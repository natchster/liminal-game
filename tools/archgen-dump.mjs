#!/usr/bin/env node
/*
 * archgen-dump — fixtures for game/tests/archgen.rs (the bit-exact check of
 * game/src/archgen.rs and game/src/scp.rs against the JS originals).
 *
 *   node tools/archgen-dump.mjs [outDir]          (default game/tests/fixtures)
 *   OPENCAROLE_SRC=/path/to/src node tools/archgen-dump.mjs
 *
 * Imports src/archgen.js and src/scp.js directly (pure ES modules) and builds
 * each case exactly the way backrooms.js does:
 *
 *   currentSeed = seedForLevel(runSeed, n)
 *   withSeed(currentSeed, () => genLayoutLevel(n, Math.min(31 + n * 4, 53)))
 *     layout   = generateLayout({ size, level: n, rng: Math.random, cell: CELL, scp: levelScp })
 *     tileCeil = ceilingHeights(layout)
 *     solids   = buildSolids(layout, { ceil: tileCeil })
 *
 * plus an anchored rebuild (settleProps' buildSolids(layout, { anchors, ceil })
 * with some tiles' ceilings raised or lowered: `raised` = [tile, f32
 * ceiling] over tileCeil), a default build (no opt), the query
 * probes (circleHits / pointInSolid / segmentClear / rayAxis) with their
 * answers, layoutStats, storeCells, and scpPlan for 500+ seeds.
 *
 * Encoding: typed arrays are base64 of their little-endian bytes; numbers are
 * JSON numbers (shortest round-trip), except -0, NaN and ±Infinity, which are
 * the strings "-0", "NaN", "Infinity", "-Infinity".
 */
import { existsSync, mkdirSync, writeFileSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));
const SRC = [process.env.OPENCAROLE_SRC, path.resolve(here, '../../src'), '/Users/nohwan/Documents/Personal_Projects/OpenCarole/src']
  .find((p) => p && existsSync(path.join(p, 'archgen.js')) && existsSync(path.join(p, 'scp.js')));
if (!SRC) { console.error('archgen-dump: cannot find src/archgen.js (set OPENCAROLE_SRC)'); process.exit(1); }
const A = await import(pathToFileURL(path.join(SRC, 'archgen.js')).href);
const P = await import(pathToFileURL(path.join(SRC, 'scp.js')).href);
const OUT = path.resolve(process.argv[2] || path.join(here, '../game/tests/fixtures'));
mkdirSync(OUT, { recursive: true });

// ---- verbatim from backrooms.js ----
const CELL = 4;
function mulberry32(seed) {
  let t = seed >>> 0;
  return function () {
    t = (t + 0x6D2B79F5) >>> 0;
    let x = t;
    x = Math.imul(x ^ (x >>> 15), x | 1);
    x ^= x + Math.imul(x ^ (x >>> 7), x | 61);
    return ((x ^ (x >>> 14)) >>> 0) / 4294967296;
  };
}
function seedForLevel(rs, n) {
  return ((rs >>> 0) + Math.imul(n | 0, 0x9E3779B9)) >>> 0;
}
let draws = 0;
function withSeed(seed, fn) {
  const prng = mulberry32(seed);
  const orig = Math.random;
  Math.random = () => { draws++; return prng(); };
  try { return fn(); } finally { Math.random = orig; }
}

// ---- encoding ----
const num = (x) => (Number.isNaN(x) ? 'NaN' : x === Infinity ? 'Infinity' : x === -Infinity ? '-Infinity' : Object.is(x, -0) ? '-0' : x);
const b64 = (ta) => Buffer.from(ta.buffer, ta.byteOffset, ta.byteLength).toString('base64');
const zoneJson = (z) => ({
  id: z.id, arch: z.arch, x0: z.x0, z0: z.z0, x1: z.x1, z1: z.z1, title: z.title, lore: z.lore,
  ceilH: num(z.ceilH), wall: z.wall, wallT: num(z.wallT), pillar: num(z.pillar), protrude: num(z.protrude),
  jamb: num(z.jamb), lintelH: num(z.lintelH), lowH: num(z.lowH),
  wallMat: z.wallMat, floorMat: z.floorMat, ceilMat: z.ceilMat, lightEvery: z.lightEvery,
});
const solidsJson = (S) => ({ count: S.count, W: S.W, H: S.H, cell: S.cell, boxes: b64(S.boxes), start: b64(S.start), items: b64(S.items) });
const statsJson = (s) => ({ ...s, openShare: num(s.openShare), arch: Object.entries(s.arch) });

// ---- query probes (inputs drawn from their own stream, stored with answers) ----
function probes(S, W, H, seed, n) {
  const r = mulberry32(seed);
  const pick = (a) => a[Math.floor(r() * a.length)];
  const B = S.boxes;
  const boxCoord = () => {          // an exact box coordinate (f32 as double), maybe nudged
    const b = Math.floor(r() * S.count), o = b * 8;
    const u = pick([B[o], B[o + 2], (B[o] + B[o + 2]) / 2, B[o] + r() * (B[o + 2] - B[o])]);
    const v = pick([B[o + 1], B[o + 3], (B[o + 1] + B[o + 3]) / 2, B[o + 1] + r() * (B[o + 3] - B[o + 1])]);
    const nudge = pick([0, 0, 1e-7, -1e-7, 1e-6, -1e-6, 0.01, -0.01, 0.3, -0.3]);
    return [u + nudge * pick([1, 0]), v + nudge * pick([1, 0]), b];
  };
  const point = () => {
    const k = r();
    if (k < 0.45) return boxCoord().slice(0, 2);
    if (k < 0.6) return [Math.floor(r() * W) + pick([0, 0.5, 1]), Math.floor(r() * H) + pick([0, 0.5, 1])];
    if (k < 0.97) return [-0.6 + r() * (W + 1.2), -0.6 + r() * (H + 1.2)];
    return [pick([NaN, Infinity, -Infinity, -0, W, H, -1e-9]), r() * H];
  };
  const yIn = () => pick([1, 1.5, 1.8, 0, 3, 0.15 + r() * 3, r() * 8]);
  const circle = [], pis = [], seg = [], ray = [];
  for (let i = 0; i < n; i++) {
    const [u, v] = point();
    const rr = pick([0.35 / CELL, 0.14 / CELL, 0.05, 0.5, r() * 0.6, 0.0875]);
    const yMax = pick([1.8, 1.6, 3, r() * 5]);
    circle.push([num(u), num(v), num(rr), num(yMax), A.circleHits(S, u, v, rr, yMax)]);
  }
  for (let i = 0; i < n; i++) {
    let u, v, y;
    if (r() < 0.6) { const [bu, bv, b] = boxCoord(); u = bu; v = bv; y = pick([B[b * 8 + 4], B[b * 8 + 5], 1, (B[b * 8 + 4] + B[b * 8 + 5]) / 2]); }
    else { [u, v] = point(); y = yIn(); }
    pis.push([num(u), num(v), num(y), A.pointInSolid(S, u, v, y)]);
  }
  for (let i = 0; i < n; i++) {
    const [u0, v0] = point();
    let u1, v1;
    const k = r();
    if (k < 0.3) { u1 = -0.6 + r() * (W + 1.2); v1 = -0.6 + r() * (H + 1.2); }             // long
    else if (k < 0.55) { u1 = u0 + (r() - 0.5) * 8; v1 = v0 + (r() - 0.5) * 8; }           // short
    else if (k < 0.7) { u1 = u0 + (r() - 0.5) * 12; v1 = v0; }                             // axis aligned
    else if (k < 0.8) { u1 = u0; v1 = v0 + (r() - 0.5) * 12; }
    else if (k < 0.9) { u1 = Math.floor(u0) + pick([-3, -1, 1, 2, 5]); v1 = Math.floor(v0) + pick([-4, -1, 0, 1, 3]); }   // tile corners: DDA ties
    else { [u1, v1] = boxCoord(); }
    const y = yIn();
    seg.push([num(u0), num(v0), num(u1), num(v1), num(y), A.segmentClear(S, u0, v0, u1, v1, y)]);
  }
  for (let i = 0; i < n; i++) {
    const dir = Math.floor(r() * 4);
    let u, v;
    if (r() < 0.5) {
      // like settleProps' faceDelta: start 0.6 tiles out from a wall-tile bound
      const [bu, bv] = boxCoord();
      const out = dir === 0 || dir === 2 ? -1 : 1;
      if (dir < 2) { u = Math.round(bu) + out * 0.6; v = bv; } else { u = bu; v = Math.round(bv) + out * 0.6; }
    } else [u, v] = point();
    let maxT = pick([1.8, 1.8, 0.5, r() * 6, 20, Infinity, NaN]);
    // a NaN start with an infinite maxT never leaves the loop (in JS too)
    if (Number.isNaN(u + v) && maxT === Infinity) maxT = 1.8;
    const y = yIn();
    ray.push([num(u), num(v), dir, num(maxT), num(y), num(A.rayAxis(S, u, v, dir, maxT, y))]);
  }
  return { circleHits: circle, pointInSolid: pis, segmentClear: seg, rayAxis: ray };
}

// ---- cases: (runSeed, level, scp) — 'plan' asks scpForLevel like scpOnLevel does ----
const CASES = [
  [0x1234ABCD, 1, 'plan'], [0x1234ABCD, 2, 'plan'], [0x1234ABCD, 3, 'plan'], [0x1234ABCD, 4, 'plan'], [0x1234ABCD, 5, 'plan'],
  [2024, 6, 'plan'], [2024, 7, 'plan'], [2024, 8, 'plan'], [2024, 9, 'plan'], [2024, 10, 'plan'],
  [0xDEADBEEF, 11, 'plan'], [0xDEADBEEF, 12, 'plan'], [0xDEADBEEF, 13, 'plan'],
  [777, 3, 'store'], [1, 2, 'store'],
  [5, 3, 'house'], [99, 8, 'house'], [4000000000, 12, 'house'],
  [123456789, 1, null], [987654321, 20, null],
];

let total = 0;
CASES.forEach(([runSeed, n, scpOpt], ci) => {
  const scp = scpOpt === 'plan' ? P.scpForLevel(runSeed, n) : scpOpt;
  const seed = seedForLevel(runSeed, n);
  const size = Math.min(31 + n * 4, 53);
  draws = 0;
  const { layout, tileCeil, solids } = withSeed(seed, () => {
    // genLayoutLevel(n, size)
    const layout = A.generateLayout({ size, level: n, rng: Math.random, cell: CELL, scp });
    const tileCeil = A.ceilingHeights(layout);
    const solids = A.buildSolids(layout, { ceil: tileCeil });
    return { layout, tileCeil, solids };
  });
  const L = layout, W = L.W, H = L.H;
  // settleProps-style rebuild: pinned faces next to open tiles, a few ceilings raised
  const r = mulberry32(0xA5C0 + ci * 7919);
  const anchors = new Set();
  for (let t = 0; t < W * H; t++) {
    if (L.tiles[t] !== 1) continue;
    for (let d = 0; d < 4; d++) if (r() < 0.12) anchors.add(t * 4 + d);
  }
  const ceil2 = tileCeil.slice(), raised = [];
  for (let t = 0; t < W * H; t++) {
    if (r() < 0.03) { ceil2[t] = Math.max(ceil2[t], +(r() * 4.2 + 0.12).toFixed(2)); raised.push([t, ceil2[t]]); }
    // (not something settleProps does) a few ceilings dropped to about the
    // zone's lintel, so both sides of `lintelH < top - 0.05` are exercised
    else if (r() < (L.tiles[t] === 0 && L.zones[L.zoneOf[t]].jamb > 0 ? 0.5 : 0.02)) { ceil2[t] = L.zones[L.zoneOf[t]].lintelH + (r() - 0.5) * 0.2; raised.push([t, ceil2[t]]); }
  }
  const anchored = A.buildSolids(L, { anchors, ceil: ceil2 });
  const plain = A.buildSolids(L);
  const fx = {
    runSeed, level: n, scpOpt, scp, seed, size, draws,
    layout: {
      W, H, cell: L.cell, level: L.level, scp: L.scp, dominant: L.dominant,
      tiles: b64(L.tiles), zoneOf: b64(L.zoneOf), tileFlags: b64(L.tileFlags),
      zones: L.zones.map(zoneJson),
    },
    // store zones, plus the first three zones as a plain-rect check
    storeCells: L.zones.filter((z, i) => z.arch === 'store' || i < 3).map((z) => ({ zone: z.id, cells: A.storeCells(z) })),
    ceil: b64(tileCeil),
    solids: solidsJson(solids),
    stats: statsJson(A.layoutStats(L, solids)),
    statsNoSolids: statsJson(A.layoutStats(L)),
    anchored: { anchors: [...anchors].sort((a, b) => a - b), raised, solids: solidsJson(anchored), stats: statsJson(A.layoutStats(L, anchored)) },
    // the default build (no opt: Z.ceilH, f32-rounded) is the same boxes as
    // the game's (tileCeil = f32 of Z.ceilH); kept as a flag, not a copy
    plainSame: plain.count === solids.count && b64(plain.boxes) === b64(solids.boxes) && b64(plain.start) === b64(solids.start) && b64(plain.items) === b64(solids.items),
    probes: probes(solids, W, H, 0x9E37 + ci, 300),
    probesAnchored: probes(anchored, W, H, 0x7F4A + ci, 120),
  };
  const name = `archgen_case_${String(ci).padStart(2, '0')}.json`;
  const s = JSON.stringify(fx);
  writeFileSync(path.join(OUT, name), s);
  total += s.length;
  console.log(`${name}  rs=${runSeed} L${n} scp=${scp} ${W}x${H} zones=${L.zones.length} boxes=${solids.count} draws=${draws}  ${(s.length / 1024).toFixed(0)} KB`);
});

// ---- tables + small exports ----
const levels = [];
for (const lv of [-3, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 30, 101]) {
  for (const scp of [null, 'store', 'house', 'loop']) {
    const s = A.levelSchedule(lv, scp);
    levels.push({ level: lv, scp, dominant: s.dominant, patchwork: !!s.patchwork, weights: Object.entries(s.weights).map(([k, w]) => [k, num(w)]) });
  }
}
const mul = [0, 1, 0xFFFFFFFF, 0x80000000, 12345].map((seed) => { const r = mulberry32(seed), a = A.mulberry32(seed); return { seed, values: Array.from({ length: 16 }, () => { const x = r(); if (x !== a()) throw new Error('mulberry32 mismatch'); return x; }) }; });
const tables = {
  DIRS: A.DIRS, TF_LOW: A.TF_LOW,
  BOX: { WALL: A.BOX_WALL, PILLAR: A.BOX_PILLAR, JAMB: A.BOX_JAMB, LINTEL: A.BOX_LINTEL, ANCHOR: A.BOX_ANCHOR },
  ARCHETYPES: Object.entries(A.ARCHETYPES).map(([key, a]) => ({ key, ...a })),
  levelSchedule: levels,
  mulberry32: mul,
};
writeFileSync(path.join(OUT, 'archgen_tables.json'), JSON.stringify(tables));

// ---- scp: 500 plans (250 small seeds, 250 full-range u32) + rosters ----
const seedStream = mulberry32(0x5CB);
const seeds = [...Array.from({ length: 250 }, (_, i) => i), ...Array.from({ length: 250 }, () => Math.floor(seedStream() * 4294967296) >>> 0)];
const plan = (s, ...a) => [...P.scpPlan(s, ...a)];
const custom = [
  { name: 'weighted', roster: [{ key: 'stairwell', from: 2, weight: 0.5 }, { key: 'store', from: 3, weight: 1.5 }, { key: 'house', from: 4, weight: 2 }, { key: 'loop', from: 2, weight: 1 }], horizon: 14 },
  { name: 'crowded', roster: Array.from({ length: 7 }, (_, i) => ({ key: ['stairwell', 'store', 'house', 'loop', 'stairwell', 'store', 'house'][i], from: 2 + (i % 3), weight: 1 })), horizon: 12 },
  { name: 'collide', roster: ['stairwell', 'store', 'house', 'loop', 'store'].map((key) => ({ key, from: 9, weight: 1 })), horizon: 10 },
  { name: 'impossible', roster: Array.from({ length: 6 }, (_, i) => ({ key: ['stairwell', 'store', 'house', 'loop', 'store', 'loop'][i], from: 2, weight: 1 })), horizon: 8 },
];
const scp = {
  SCP_ROSTER: P.SCP_ROSTER, SCP_KEYS: P.SCP_KEYS, SCP_HORIZON: P.SCP_HORIZON,
  plans: seeds.map((s) => ({ seed: s, plan: plan(s) })),
  forLevel: seeds.slice(0, 40).map((s) => ({ seed: s, levels: Array.from({ length: 16 }, (_, n) => P.scpForLevel(s, n)) })),
  custom: custom.map((c) => ({ ...c, plans: seeds.slice(0, 120).map((s) => ({ seed: s, plan: plan(s, c.roster, c.horizon) })) })),
};
writeFileSync(path.join(OUT, 'scp_plan.json'), JSON.stringify(scp));
console.log(`wrote ${CASES.length} cases (${(total / 1048576).toFixed(1)} MB), archgen_tables.json, scp_plan.json to ${OUT}`);
