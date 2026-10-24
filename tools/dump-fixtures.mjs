#!/usr/bin/env node
/*
 * dump-fixtures — runs the original JS modules (src/props.js,
 * src/creatures.js and the wtGeo block of backrooms.js) under node with the
 * vendored three.js r160 and writes reference outputs for the Rust tests:
 *
 *   game/tests/fixtures/props_fixtures.bin     ~40 (part, primitive) cases
 *   game/tests/fixtures/creature_fixtures.bin  creature parts, cavity shade, Watcher morphs
 *   game/tests/fixtures/to_fixed.txt           Number.prototype.toFixed(4) samples
 *
 *   node native/tools/dump-fixtures.mjs [/path/to/OpenCarole]   (default: ../.. from here)
 *
 * File layout (little-endian): magic (8 bytes), u32 case count, then per
 * case: str name, str spec ("k=v;k=v"), str result ("k=v;..."), u32 record
 * count, records. A record is: str label, u8 type (0 f32, 1 u32, 2 f64),
 * u32 itemSize, u32 stride, u32 itemCount, then the values of every item
 * whose index is a multiple of stride. str = u16 length + utf-8.
 */
import { writeFileSync, mkdirSync } from 'node:fs';
import { dirname, resolve } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
const root = resolve(process.argv[2] || process.env.OPENCAROLE_ROOT || resolve(here, '../..'));
const outDir = resolve(here, '../game/tests/fixtures');
const imp = (p) => import(pathToFileURL(resolve(root, p)).href);
const THREE = await imp('vendor/three.module.min.js');
const props = await imp('src/props.js');
const creatures = await imp('src/creatures.js');

class Out {
  constructor() { this.chunks = []; }
  push(b) { this.chunks.push(b); }
  u8(v) { const b = Buffer.alloc(1); b.writeUInt8(v); this.push(b); }
  u16(v) { const b = Buffer.alloc(2); b.writeUInt16LE(v); this.push(b); }
  u32(v) { const b = Buffer.alloc(4); b.writeUInt32LE(v >>> 0); this.push(b); }
  str(s) { const b = Buffer.from(s, 'utf8'); this.u16(b.length); this.push(b); }
  bytes(s) { this.push(Buffer.from(s)); }
  done() { return Buffer.concat(this.chunks); }
}
// one record: a typed array sampled every `stride` items
function rec(out, label, arr, itemSize, type = 0, stride = 1) {
  out.str(label); out.u8(type); out.u32(itemSize); out.u32(stride);
  const items = arr ? arr.length / itemSize : 0;
  out.u32(items);
  const vals = [];
  for (let i = 0; i < items; i += stride) for (let k = 0; k < itemSize; k++) vals.push(arr[i * itemSize + k]);
  const b = Buffer.alloc(vals.length * (type === 2 ? 8 : 4));
  vals.forEach((v, i) => { if (type === 0) b.writeFloatLE(v, i * 4); else if (type === 1) b.writeUInt32LE(v, i * 4); else b.writeDoubleLE(v, i * 8); });
  out.push(b);
}
const kv = (o) => Object.entries(o).map(([k, v]) => `${k}=${Array.isArray(v) ? v.map(String).join(',') : String(v)}`).join(';');

// ---------------------------------------------------------------- props
// three's constructor parameters in a fixed order (the Rust test rebuilds the
// same engine primitive from them)
const ORDER = {
  BoxGeometry: ['width', 'height', 'depth', 'widthSegments', 'heightSegments', 'depthSegments'],
  CylinderGeometry: ['radiusTop', 'radiusBottom', 'height', 'radialSegments', 'heightSegments', 'openEnded', 'thetaStart', 'thetaLength'],
  ConeGeometry: ['radius', 'height', 'radialSegments', 'heightSegments', 'openEnded', 'thetaStart', 'thetaLength'],
  SphereGeometry: ['radius', 'widthSegments', 'heightSegments', 'phiStart', 'phiLength', 'thetaStart', 'thetaLength'],
  TorusGeometry: ['radius', 'tube', 'radialSegments', 'tubularSegments', 'arc'],
  CapsuleGeometry: ['radius', 'length', 'capSegments', 'radialSegments'],
  CircleGeometry: ['radius', 'segments', 'thetaStart', 'thetaLength'],
  PlaneGeometry: ['width', 'height', 'widthSegments', 'heightSegments'],
  OctahedronGeometry: ['radius', 'detail'],
  RingGeometry: ['innerRadius', 'outerRadius', 'thetaSegments', 'phiSegments', 'thetaStart', 'thetaLength'],
};
const P = Math.PI;
// [part, ctor, args, { translate, own, disabled, nested }] — mostly real call sites of backrooms.js
const CASES = [
  ['DeadVendingMachines__cabinet', 'Box', [0.90, 1.85, 0.55]],
  ['Radiators__body', 'Box', [0.70, 0.50, 0.10]],
  ['Suitcases__body', 'Box', [0.40, 0.28, 0.14]],
  ['Suitcases__body', 'Box', [0.40, 0.28, 0.14]],                 // cached: the same geometry
  ['Suitcases__body', 'Box', [0.40000001, 0.28, 0.14]],           // same key to 0.1 mm
  ['Suitcases__body', 'Box', [0.40, 0.28, 0.14, 2, 2, 2], { own: 1 }],   // cached, private copy
  ['Suitcases__body', 'Box', [0.40, 0, 0.14]],                    // a flat axis (scale clamp)
  ['RolledCarpets__roll', 'Cylinder', [0.12, 0.12, 2.4, 8]],
  ['RolledCarpets__roll', 'Cylinder', [0.09, 0.09, 1.9, 8], { translate: [0, 0.5, 0] }],
  ['Trowels__blade', 'Cylinder', [0.020, 0.014, 0.080, 6, 1, false, 0, P]],
  ['IroningBoards__backEnd', 'Cylinder', [0.08, 0.08, 0.020, 8, 1, false, P, P]],
  ['CheeseWedges__body', 'Cylinder', [0.055, 0.055, 0.040, 3, 1, false, 0, P * 0.5]],
  ['Nest__flamp_pole', 'Cylinder', [0.014, 0.014, 1.4, 8]],
  ['WagonWheels__band', 'Torus', [0.128, 0.014, 4, 20]],
  ['FireHoseReels__hose', 'Torus', [0.14, 0.020, 6, 16]],
  ['Horseshoes__arch', 'Torus', [0.055, 0.010, 6, 14, P * 1.3]],
  ['Teapots__handle', 'Torus', [0.030, 0.006, 4, 10, P]],
  ['BiohazardSigns__plate', 'Plane', [0.20, 0.20]],
  ['WetFloorSigns__panel', 'Plane', [0.28, 0.42]],
  ['Props__balloon_body', 'Sphere', [0.3, 18, 14]],
  ['CastIronKettles__body', 'Sphere', [0.060, 12, 10]],
  ['SpoonRests__dish', 'Sphere', [0.045, 12, 8, 0, P * 2, 0, P / 2]],
  ['CastIronLadles__bowl', 'Sphere', [0.035, 10, 6, 0, P * 2, P / 2, P / 2]],
  ['CuckooClocks__roof', 'Cone', [0.14, 0.10, 3]],
  ['SmallPhonographs__horn', 'Cone', [0.08, 0.20, 12, 1, true]],
  ['SafetyCones__body', 'Cone', [0.06, 0.28, 8]],
  ['SafetyCones__body', 'Cone', [0.06, 0.28, 8, 1, false, 0, P]],   // a half cone (hi-res bounds)
  ['Props__mannequin_torso', 'Capsule', [0.21, 0.62, 4, 10]],
  ['Props__mannequin_leg', 'Capsule', [0.075, 0.62, 4, 8]],
  ['JewelryBoxes__stone', 'Octahedron', [0.004, 0]],
  ['SingleEarrings__gem', 'Octahedron', [0.005], { translate: [0.01, 0, 0] }],
  ['DeadCeilingLights__canopy', 'Circle', [0.11, 12]],
  ['MossPatches__mat', 'Circle', [0.14, 8]],
  ['MossPatches__mat', 'Circle', [0.14, 8, 0, P]],                 // a half disc (hi-res bounds)
  ['FloorDrainGrates__plate', 'Ring', [0.085, 0.11, 14]],
  ['Nonexistent__part', 'Box', [1, 1, 1]],                         // missing → the fallback
  ['Radiators__body', 'Box', [0.7, 0.5, 0.1], { disabled: 1 }],    // #noprops → the fallback
  ['Radiators__body', 'Box', [0.2, 0.3, 0.4], { nested: 'Suitcases__body' }],   // a part as the fallback
];

{
  const out = new Out();
  out.bytes('PFIX0001');
  out.u32(CASES.length);
  const results = [];
  for (const [name, ctor, args, opt = {}] of CASES) {
    const type = ctor + 'Geometry';
    const make = () => {
      const g = new THREE[type](...args);
      if (opt.translate) g.translate(...opt.translate);
      return g;
    };
    let fb = make();
    if (opt.nested) fb = props.propPart(THREE, opt.nested, fb);
    const params = ORDER[type].map((k) => { const v = fb.parameters[k]; return typeof v === 'boolean' ? (v ? 1 : 0) : v; });
    if (opt.disabled) props.setPropsEnabled(false);
    const before = { ...props.propStats };
    const g = props.propPart(THREE, name, fb, !!opt.own);
    if (opt.disabled) props.setPropsEnabled(true);
    const cached = props.propStats.cached > before.cached ? 1 : 0;
    const same = results.indexOf(g);
    results.push(g);
    out.str(name);
    out.str(kv({ type, params, translate: opt.translate || [0, 0, 0], own: opt.own ? 1 : 0, disabled: opt.disabled ? 1 : 0, nested: opt.nested || '' }));
    out.str(kv({ fallback: g === fb ? 1 : 0, cached, same, gtype: g.type, part: g.userData.propPart ? 1 : 0, keepsParams: g.parameters === fb.parameters ? 1 : 0 }));
    g.computeBoundingBox(); g.computeBoundingSphere();
    const bb = g.boundingBox, bs = g.boundingSphere;
    out.u32(6);
    rec(out, 'position', g.attributes.position.array, 3);
    rec(out, 'normal', g.attributes.normal.array, 3);
    rec(out, 'uv', g.attributes.uv.array, 2);
    rec(out, 'index', g.index ? g.index.array : null, 1, 1);
    rec(out, 'bbox', [bb.min.x, bb.min.y, bb.min.z, bb.max.x, bb.max.y, bb.max.z], 6, 2);
    rec(out, 'sphere', [bs.center.x, bs.center.y, bs.center.z, bs.radius], 4, 2);
  }
  // the counters after all of it, and every part's metadata for the decode test
  const tail = new Out();
  tail.str(kv(props.propStats));
  // every part, fitted to its own frame through a plain two-point
  // BufferGeometry (the bounding-box branch of idealBounds): counts and sums
  const names = props.PROP_PART_NAMES;
  tail.u32(names.length);
  const { PROP_MESHES } = await imp('src/prop-meshes.js');
  const sum = (a) => { let t = 0; for (let i = 0; i < a.length; i++) t += a[i]; return t; };
  for (const name of names) {
    const m = PROP_MESHES[name];
    const fb = new THREE.BufferGeometry();
    fb.setAttribute('position', new THREE.BufferAttribute(new Float32Array([...m.fmin, ...m.fmax]), 3));
    const g = props.propPart(THREE, name, fb);
    tail.str(name); tail.str(m.f);
    tail.u32(m.n); tail.u32(m.t); tail.u32(g.attributes.position.count); tail.u32(g.index.count);
    const b = Buffer.alloc(32);
    b.writeDoubleLE(sum(g.attributes.position.array), 0); b.writeDoubleLE(sum(g.attributes.normal.array), 8);
    b.writeDoubleLE(sum(g.attributes.uv.array), 16); b.writeDoubleLE(sum(g.index.array), 24);
    tail.push(b);
  }
  mkdirSync(outDir, { recursive: true });
  writeFileSync(resolve(outDir, 'props_fixtures.bin'), Buffer.concat([out.done(), tail.done()]));
  console.log(`props_fixtures.bin: ${CASES.length} cases`, props.propStats);
}

// ---------------------------------------------------------------- creatures
{
  const out = new Out();
  out.bytes('CFIX0001');
  const cases = [];
  const geoRecs = (o, g, stride) => {
    const a = g.attributes;
    const recs = [
      ['position', a.position.array, 3, 0], ['normal', a.normal.array, 3, 0], ['uv', a.uv.array, 2, 0],
      ['color', a.color ? a.color.array : null, 3, 0],
      ['skinIndex', a.skinIndex ? a.skinIndex.array : null, 4, 1], ['skinWeight', a.skinWeight ? a.skinWeight.array : null, 4, 0],
      ['index', g.index.array, 1, 1],
      ['sphere', [g.boundingSphere.center.x, g.boundingSphere.center.y, g.boundingSphere.center.z, g.boundingSphere.radius], 4, 2],
    ];
    o.u32(recs.length);
    for (const [l, arr, size, type] of recs) rec(o, l, arr, size, type, l === 'index' || l === 'sphere' ? 1 : stride);
  };
  const add = (name, spec, result, fill) => cases.push({ name, spec, result, fill });
  for (const name of ['smThigh', 'hdTail', 'shBody', 'wtPose1', 'hdTeethUp']) {
    const g = creatures.creatureGeometry(THREE, name);
    const stride = g.attributes.position.count > 3000 ? 3 : 1;
    add(name, kv({ kind: 'geometry' }), kv({ bones: (g.userData.bones || []).join(','), n: g.attributes.position.count, t: g.index.count / 3 }), (o) => geoRecs(o, g, stride));
  }
  for (const [name, strength] of [['stairFace', 0.3], ['smHead', 0.75], ['hdTail', 0.5]]) {
    const g = creatures.withCavityShade(THREE, creatures.creatureGeometry(THREE, name), strength);
    add(name, kv({ kind: 'cavity', strength }), '', (o) => { o.u32(1); rec(o, 'color', g.attributes.color.array, 3, 0, 1); });
  }
  {
    // backrooms.js `wtGeo`, verbatim
    const body = creatures.creatureGeometry(THREE, 'wtBody');
    const n = body.attributes.position.count, bp = body.attributes.position.array, bn = body.attributes.normal.array;
    const bc = body.attributes.color && body.attributes.color.array;
    const tmp = new THREE.BufferGeometry();
    tmp.setIndex(body.index);
    const mp = [], mn = [], mc = [];
    for (const name of ['wtPose1', 'wtPose2', 'wtPose3', 'wtPose4', 'wtBow']) {
      const g = creatures.creatureGeometry(THREE, name);
      if (!g || g.attributes.position.count !== n) throw new Error('watcher pose ' + name);
      const p = g.attributes.position.array;
      tmp.setAttribute('position', new THREE.BufferAttribute(p, 3));
      tmp.deleteAttribute('normal');
      tmp.computeVertexNormals();
      const nn = tmp.attributes.normal.array, c = g.attributes.color && g.attributes.color.array;
      const dp = new Float32Array(n * 3), dn = new Float32Array(n * 3), dc = new Float32Array(n * 3);
      for (let i = 0; i < n * 3; i++) {
        dp[i] = p[i] - bp[i]; dn[i] = nn[i] - bn[i];
        if (c && bc) dc[i] = c[i] - bc[i];
      }
      mp.push(new THREE.BufferAttribute(dp, 3)); mn.push(new THREE.BufferAttribute(dn, 3)); mc.push(new THREE.BufferAttribute(dc, 3));
    }
    body.morphAttributes.position = mp; body.morphAttributes.normal = mn;
    if (bc) body.morphAttributes.color = mc;
    body.morphTargetsRelative = true;
    body.computeBoundingSphere();
    const s = body.boundingSphere;
    add('wtBody', kv({ kind: 'watcher' }), kv({ morphs: mp.length }), (o) => {
      o.u32(1 + mp.length * 3);
      rec(o, 'sphere', [s.center.x, s.center.y, s.center.z, s.radius], 4, 2);
      mp.forEach((m, i) => { rec(o, `mp${i}`, m.array, 3, 0, 5); rec(o, `mn${i}`, mn[i].array, 3, 0, 5); rec(o, `mc${i}`, mc[i].array, 3, 0, 5); });
    });
  }
  out.u32(cases.length);
  for (const c of cases) { out.str(c.name); out.str(c.spec); out.str(c.result); c.fill(out); }
  // every part: counts and sums (wtBody now carries the morphs; its own buffers are unchanged)
  const sum = (a) => { let t = 0; if (a) for (let i = 0; i < a.length; i++) t += a[i]; return t; };
  out.u32(creatures.CREATURE_PART_NAMES.length);
  for (const name of creatures.CREATURE_PART_NAMES) {
    const g = creatures.creatureGeometry(THREE, name), a = g.attributes;
    out.str(name);
    out.u32(a.position.count); out.u32(g.index.count / 3);
    const b = Buffer.alloc(56);
    [a.position, a.normal, a.uv, a.color, a.skinIndex, a.skinWeight].forEach((x, i) => b.writeDoubleLE(sum(x && x.array), i * 8));
    b.writeDoubleLE(sum(g.index.array), 48);
    out.push(b);
  }
  writeFileSync(resolve(outDir, 'creature_fixtures.bin'), out.done());
  console.log(`creature_fixtures.bin: ${cases.length} cases`);
}

// ---------------------------------------------------------------- toFixed(4)
{
  // deterministic samples: ties and near-ties, negatives, tiny and large values
  let s = 12345;
  const rnd = () => { s = (s * 1103515245 + 12345) % 2147483648; return s / 2147483648; };
  const vals = [0, -0, 1e-5, -1e-5, 0.00005, -0.00005, 1.00005, 0.03125, -0.03125, 2.5, 123.456789, 1e20, -1e20];
  for (let i = 0; i < 400; i++) {
    const k = Math.floor(rnd() * 400000) - 200000;
    const t = (k + 0.5) / 10000;
    vals.push(t, t + Number.EPSILON * Math.abs(t), t - Number.EPSILON * Math.abs(t), (k * 2 + 1) / 20000);
    vals.push((rnd() - 0.5) * 10 ** Math.floor(rnd() * 8 - 4));
  }
  const lines = vals.map((v) => `${Object.is(v, -0) ? '-0' : String(v)} ${v.toFixed(4)}`);
  writeFileSync(resolve(outDir, 'to_fixed.txt'), lines.join('\n') + '\n');
  console.log(`to_fixed.txt: ${vals.length} values`);
}

// ---------------------------------------------------------------- placement
// A synthetic room run through src/placement.js. The spec lines rebuild the
// same scene in Rust; the R/S lines are what JS made of it.
{
  const { resolvePlacement } = await imp('src/placement.js');
  let seed = 20241024;
  const rnd = () => (seed = (seed * 16807) % 2147483647) / 2147483647;
  const lines = [];
  const walls = [
    [-6.2, 0, -6.2, 6.2, 3, -6], [-6.2, 0, 6, 6.2, 3, 6.2], [-6.2, 0, -6, -6, 3, 6], [6, 0, -6, 6.2, 3, 6],
    [1.7, 0, 1.7, 2.3, 3, 2.3], [-3, 0, 0.9, 0, 3, 1.1], [3.4, 0, -4, 3.5, 3, -1],
    [-1, 0, -3, -0.9, 3, -1], [-0.6, 0, -3, -0.5, 3, -1],                         // a 0.3 m gap
  ];
  for (const w of walls) lines.push('W ' + w.join(' '));
  const world = new THREE.Group();
  const objects = [];
  const placers = ['placeProps', 'placeCrates', 'placeLamps', 'placeSigns', 'placeChairs'];
  const N = 170;
  for (let i = 0; i < N; i++) {
    const kind = rnd();
    const o = new THREE.Group();
    let x = -5.8 + rnd() * 11.6, z = -5.8 + rnd() * 11.6, y;
    if (kind < 0.08) { x = -0.75 + (rnd() - 0.5) * 0.1; z = -3 + rnd() * 2; y = 0; }   // stuck in the gap
    else if (kind < 0.45) y = [-0.05, 0, 0.004, 0.03, 0.3][Math.floor(rnd() * 5)];       // floor things, some sunk or hovering
    else if (kind < 0.65) y = 0.7 + rnd() * 0.2;                                     // on a table, or hovering
    else if (kind < 0.85) {                                                          // wall things
      y = 1.2 + rnd();
      const side = Math.floor(rnd() * 4);
      if (side === 0) z = -6 + 0.02 + rnd() * 0.1; else if (side === 1) z = 6 - 0.02 - rnd() * 0.1;
      else if (side === 2) x = -6 + 0.02 + rnd() * 0.1; else x = 6 - 0.02 - rnd() * 0.1;
    } else y = 2.7 + rnd() * 0.25;                                                   // ceiling things
    const ry = rnd() < 0.5 ? 0 : [Math.PI / 2, Math.PI, rnd() * Math.PI * 2][Math.floor(rnd() * 3)];
    o.position.set(x, y, z); o.rotation.y = ry;
    const placer = placers[Math.floor(rnd() * placers.length)];
    const protect = rnd() < 0.08 ? 1 : 0;
    const floats = rnd() < 0.03 ? 1 : 0;
    o.userData.placer = placer; o.userData.floats = !!floats;
    const nm = 1 + Math.floor(rnd() * 3);
    lines.push(`O ${placer} ${protect} ${floats} ${x} ${y} ${z} ${ry} ${nm}`);
    for (let k = 0; k < nm; k++) {
      const t = rnd();
      const type = t < 0.7 ? 'box' : t < 0.9 ? 'cyl' : 'plane';
      const a = 0.04 + rnd() * 0.7, b = 0.02 + rnd() * 0.6, c = 0.04 + rnd() * 0.6;
      const mx = (rnd() - 0.5) * 0.3, my = k === 0 ? b / 2 : rnd() * 0.5, mz = (rnd() - 0.5) * 0.3;
      const rx = rnd() < 0.8 ? 0 : rnd() * 0.6, rz = rnd() < 0.8 ? 0 : rnd() * 0.6;
      const m = rnd();
      const transparent = m < 0.06 ? 1 : 0, opacity = transparent ? (rnd() < 0.5 ? 0.3 : 0.8) : 1;
      const depthWrite = m > 0.97 ? 0 : 1, additive = m > 0.94 && m <= 0.97 ? 1 : 0;
      const geo = type === 'box' ? new THREE.BoxGeometry(a, b, c) : type === 'cyl' ? new THREE.CylinderGeometry(a / 2, a / 2, b, 12) : new THREE.PlaneGeometry(a, c);
      const mat = new THREE.MeshLambertMaterial({ transparent: !!transparent, opacity, depthWrite: !!depthWrite, blending: additive ? THREE.AdditiveBlending : THREE.NormalBlending });
      const mesh = new THREE.Mesh(geo, mat);
      mesh.position.set(mx, my, mz); mesh.rotation.set(rx, 0, rz);
      o.add(mesh);
      lines.push(`M ${type} ${a} ${b} ${c} ${mx} ${my} ${mz} ${rx} ${rz} ${transparent} ${opacity} ${depthWrite} ${additive}`);
    }
    world.add(o);
    objects.push({ obj: o, placer, protect: !!protect });
  }
  const ceilAt = (x, z) => (x > 3 ? 2.4 : 3.0);
  const inPool = (x, z) => x < -4.5 && z < -4.5;
  const wallsQ = (bb) => walls.filter((w) => !(w[0] > bb[3] || w[3] < bb[0] || w[2] > bb[5] || w[5] < bb[2]));
  const st = resolvePlacement(THREE, { objects, walls: wallsQ, ceilAt, inPool, tol: 0.015 });
  objects.forEach((it, i) => lines.push(`R ${i} ${it.obj.parent ? 0 : 1} ${it.obj.position.x} ${it.obj.position.y} ${it.obj.position.z}`));
  lines.push('S ' + ['props', 'lifted', 'dropped', 'stacked', 'unburied', 'nudged', 'removed', 'resettled', 'skipped'].map((k) => st[k]).join(' '));
  writeFileSync(resolve(outDir, 'placement_world.txt'), lines.join('\n') + '\n');
  console.log('placement_world.txt', st);
}

// ---------------------------------------------------------------- instancing
// Repeated static meshes through src/instancing.js; the result lines describe
// the world's children afterwards.
{
  const { instancifyStatic } = await imp('src/instancing.js');
  let seed = 99991;
  const rnd = () => (seed = (seed * 16807) % 2147483647) / 2147483647;
  const lines = [];
  const geos = [new THREE.BoxGeometry(0.1, 0.2, 0.3), new THREE.CylinderGeometry(0.05, 0.05, 0.4, 8), new THREE.SphereGeometry(0.1, 8, 6)];
  lines.push('GEOS 3');
  // materials: 0 and 1 look alike (two placers' identical materials), 2 differs
  // in colour, 3 is transparent, 4 is basic, 5 has depthWrite off, 6 emissive
  const mdefs = [
    ['lambert', 0x808080, 0, 0, 1], ['lambert', 0x808080, 0, 0, 1], ['lambert', 0x406020, 0, 0, 1], ['lambert', 0x808080, 0, 1, 1],
    ['basic', 0x808080, 0, 0, 1], ['lambert', 0x808080, 0, 0, 0], ['lambert', 0x808080, 0x200000, 0, 1],
  ];
  const mats = mdefs.map(([k, c, e, tr, dw]) => {
    const m = k === 'basic' ? new THREE.MeshBasicMaterial({ color: c }) : new THREE.MeshLambertMaterial({ color: c, emissive: e });
    m.transparent = !!tr; m.depthWrite = !!dw;
    return m;
  });
  mdefs.forEach((d, i) => lines.push(`MAT ${i} ${d.join(' ')}`));
  const world = new THREE.Group();
  const meshMeta = new Map();
  for (let i = 0; i < 40; i++) {
    const g = new THREE.Group();
    const st = rnd() < 0.85 ? 1 : 0, noi = rnd() < 0.08 ? 1 : 0;
    const x = (rnd() - 0.5) * 16, z = (rnd() - 0.5) * 16, ry = rnd() < 0.5 ? 0 : rnd() * 6;
    g.position.set(x, 0, z); g.rotation.y = ry;
    g.userData.staticProp = !!st; g.userData.noInstance = !!noi;
    lines.push(`T ${st} ${noi} ${x} ${z} ${ry}`);
    const sub = rnd() < 0.3;
    let parent = g;
    if (sub) { parent = new THREE.Group(); parent.position.set(0, 0.5, 0); g.add(parent); lines.push('SUB'); }
    const nm = 1 + Math.floor(rnd() * 3);
    for (let k = 0; k < nm; k++) {
      const gi = Math.floor(rnd() * 3), mi = Math.floor(rnd() * mats.length), vis = rnd() < 0.95 ? 1 : 0;
      const m = new THREE.Mesh(geos[gi], mats[mi]);
      m.position.set(rnd() * 0.5, rnd() * 0.5, rnd() * 0.5); m.visible = !!vis;
      parent.add(m);
      meshMeta.set(m, [gi, mi]);
      lines.push(`M ${gi} ${mi} ${m.position.x} ${m.position.y} ${m.position.z} ${vis}`);
    }
    world.add(g);
  }
  const chunkOf = (x, z) => Math.floor(x / 4) * 64 + Math.floor(z / 4);
  const st = instancifyStatic(THREE, world, chunkOf);
  lines.push(`S ${st.meshes} ${st.instanced} ${st.buckets} ${st.removedGroups}`);
  const metaOf = (geo, mat) => [geos.indexOf(geo), mats.indexOf(mat)];
  for (const c of world.children) {
    if (c.isInstancedMesh) {
      const [gi, mi] = metaOf(c.geometry, c.material);
      let sx = 0, sy = 0, sz = 0;
      const mm = new THREE.Matrix4();
      for (let i = 0; i < c.count; i++) { c.getMatrixAt(i, mm); sx += mm.elements[12]; sy += mm.elements[13]; sz += mm.elements[14]; }
      lines.push(`I ${c.count} ${gi} ${mi} ${sx} ${sy} ${sz}`);
    } else {
      let n = 0; c.traverse((o) => { if (o.isMesh) n++; });
      let groups = 0; c.traverse((o) => { if (!o.isMesh) groups++; });
      lines.push(`G ${n} ${groups}`);
    }
  }
  writeFileSync(resolve(outDir, 'instancing_world.txt'), lines.join('\n') + '\n');
  console.log('instancing_world.txt', st);
}
