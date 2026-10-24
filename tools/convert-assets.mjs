#!/usr/bin/env node
/*
 * convert-assets — packs the generated Blender meshes of the JS game into the
 * compact binaries the native game embeds (include_bytes!):
 *
 *   src/prop-meshes.js     → game/assets/prop_meshes.bin     (game/src/props.rs)
 *   src/creature-meshes.js → game/assets/creature_meshes.bin (game/src/creatures.rs)
 *
 * Regenerate after `npm run creatures` / tools/prop-assemble.mjs changed the
 * JS data:
 *
 *   node native/tools/convert-assets.mjs            # reads ../src next to native/
 *   node tools/convert-assets.mjs /path/to/OpenCarole/src
 *   OPENCAROLE_SRC=/path/to/src node tools/convert-assets.mjs
 *
 * The compressed payloads are kept as they are (raw DEFLATE, decoded in Rust
 * with miniz_oxide), so the binaries are the base64-decoded JS data plus a
 * small header per part. All numbers little-endian; bounds are f64 (the exact
 * JS numbers).
 *
 * prop_meshes.bin
 *   "PROPMSH1" u32 count, then per part (in PROP_MESHES key order):
 *   u16 nameLen, name (utf-8), u8 kind (0 box 1 cyl 2 sphere 3 torus 4 plane 5 other),
 *   u32 n (vertices), u32 t (triangles), u32 r (inflated bytes),
 *   f64 min[3], max[3], fmin[3], fmax[3], u32 zLen, z (raw deflate)
 *
 * creature_meshes.bin
 *   "CREAMSH1" u32 count, then per part (in CREATURE_MESHES key order):
 *   u16 nameLen, name, u8 flags (1 packed `z`, 2 colours `c`, 4 skin `j`&&`w`, 8 bones `b`),
 *   u32 n, u32 t, f64 min[3], max[3], u16 boneCount, per bone u16 len + name,
 *   then the fields v, i, c, j, w: u32 len + bytes (base64-decoded; deflated
 *   when packed, raw otherwise; empty when absent)
 */
import { writeFileSync, mkdirSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
const srcDir = resolve(process.argv[2] || process.env.OPENCAROLE_SRC || resolve(here, '../../src'));
const outDir = resolve(here, '../game/assets');

const { PROP_MESHES } = await import(pathToFileURL(resolve(srcDir, 'prop-meshes.js')).href);
const { CREATURE_MESHES } = await import(pathToFileURL(resolve(srcDir, 'creature-meshes.js')).href);

class Out {
  constructor() { this.chunks = []; this.len = 0; }
  push(b) { this.chunks.push(b); this.len += b.length; }
  bytes(b) { this.push(Buffer.from(b)); }
  u8(v) { const b = Buffer.alloc(1); b.writeUInt8(v); this.push(b); }
  u16(v) { if (v > 0xffff) throw new Error('u16 overflow'); const b = Buffer.alloc(2); b.writeUInt16LE(v); this.push(b); }
  u32(v) { const b = Buffer.alloc(4); b.writeUInt32LE(v); this.push(b); }
  f64(v) { const b = Buffer.alloc(8); b.writeDoubleLE(v); this.push(b); }
  str(s) { const b = Buffer.from(s, 'utf8'); this.u16(b.length); this.push(b); }
  blob(b) { this.u32(b.length); this.push(b); }
  done() { return Buffer.concat(this.chunks, this.len); }
}
const b64 = (s) => Buffer.from(s || '', 'base64');
const KINDS = { box: 0, cyl: 1, sphere: 2, torus: 3, plane: 4, other: 5 };

// ---------------------------------------------------------------- props
{
  const o = new Out();
  const names = Object.keys(PROP_MESHES);
  o.bytes('PROPMSH1');
  o.u32(names.length);
  for (const name of names) {
    const m = PROP_MESHES[name];
    if (!(m.f in KINDS)) throw new Error(`${name}: unknown frame kind ${m.f}`);
    o.str(name);
    o.u8(KINDS[m.f]);
    o.u32(m.n); o.u32(m.t); o.u32(m.r);
    for (const a of [m.min, m.max, m.fmin, m.fmax]) for (let k = 0; k < 3; k++) o.f64(a[k]);
    o.blob(b64(m.z));
  }
  mkdirSync(outDir, { recursive: true });
  const buf = o.done();
  writeFileSync(resolve(outDir, 'prop_meshes.bin'), buf);
  console.log(`prop_meshes.bin: ${names.length} parts, ${buf.length} bytes`);
}

// ---------------------------------------------------------------- creatures
{
  const o = new Out();
  const names = Object.keys(CREATURE_MESHES);
  o.bytes('CREAMSH1');
  o.u32(names.length);
  for (const name of names) {
    const m = CREATURE_MESHES[name];
    o.str(name);
    // JS truthiness, exactly as src/creatures.js tests them
    const flags = (m.z ? 1 : 0) | (m.c ? 2 : 0) | (m.j && m.w ? 4 : 0) | (m.b ? 8 : 0);
    o.u8(flags);
    o.u32(m.n); o.u32(m.t);
    for (const a of [m.min, m.max]) for (let k = 0; k < 3; k++) o.f64(a[k]);
    const bones = m.b || [];
    o.u16(bones.length);
    for (const b of bones) o.str(b);
    for (const k of ['v', 'i', 'c', 'j', 'w']) o.blob(b64(m[k]));
  }
  const buf = o.done();
  writeFileSync(resolve(outDir, 'creature_meshes.bin'), buf);
  console.log(`creature_meshes.bin: ${names.length} parts, ${buf.length} bytes`);
}
