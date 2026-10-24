// Fixture generator for the Rust port of placeProps' first half
// (game/src/core_props.rs + core_props_builders.rs, backrooms.js 57003–59034).
//
// Runs the original JS — placeProps up to the hand-off to its second half,
// plus wallAdjSpot, circleHitsWall, rand/irand/choice/clamp and the shared
// materials, all cut from backrooms.js — under node with the vendored three.js
// r160, src/props.js (the real PG), a counted mulberry32 Math.random and stub
// canvas contexts (the draws still run, on a separate "native" generator, as
// canvasTex does). Each case is a synthetic level; the fixture records the
// seeded draw count at the hand-off, the shared locals (cands / taken /
// exitTile), the world tree, interactables, animators, the gameplay records,
// and then the state after a scripted run of the animators and a use of
// every interactable (under a second seeded stream).
//
//   node tools/core_props_fixtures.mjs [/path/to/OpenCarole]
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const HERE = path.dirname(fileURLToPath(import.meta.url));
const ROOT = process.argv[2] || '/Users/nohwan/Documents/Personal_Projects/OpenCarole';
const OUT = path.join(HERE, '..', 'game', 'tests', 'fixtures');
fs.mkdirSync(OUT, { recursive: true });
const imp = (p) => import(pathToFileURL(path.join(ROOT, p)).href);
const THREE = await imp('vendor/three.module.min.js');
const props = await imp('src/props.js');

const SRC = fs.readFileSync(path.join(ROOT, 'backrooms.js'), 'utf8').split('\n');
const lines = (a, b) => SRC.slice(a - 1, b).join('\n');
const expect = (n, s) => { if (!SRC[n - 1].startsWith(s)) throw new Error(`backrooms.js:${n} moved: ${SRC[n - 1]}`); };
expect(42, 'const rand = ');
expect(45, 'const clamp = ');
expect(1513, 'function circleHitsWall(px, pz, r) {');
expect(1540, '}');
expect(3032, 'const darkMat = ');
expect(3044, 'const glassMat = ');
expect(3618, 'function wallAdjSpot(x, z) {');
expect(3627, '}');
expect(57003, 'function placeProps(dist, m4) {');
expect(59034, '  scatter(buildFloorPlant, 1);');
expect(59036, '  // ---- WEATHER-DRIVEN EXTRA PROPS ----');

// ------------------------------------------------------------------ rng ---
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
let seeded = null, draws = 0;
const nativeRandom = mulberry32(0xC0FFEE);
// the vendored three is patched (tools/patch-three.mjs) so generateUUID never touches the level stream
globalThis.__threeUuidRandom = mulberry32(0xBADC0DE);
Math.random = () => { if (!seeded) throw new Error('unseeded Math.random'); draws++; return seeded(); };

// ------------------------------------------------------------------ stubs ---
const stubCtx = new Proxy({}, {
  get(t, k) {
    if (k in t) return t[k];
    if (typeof k === 'string' && k.startsWith('create')) return () => ({ addColorStop() {} });
    if (k === 'measureText') return () => ({ width: 0 });
    return () => {};
  },
  set(t, k, v) { t[k] = v; return true; },
});
function canvasTex(w, h, draw, repeat = true) {
  const s = seeded; seeded = nativeRandom; const d = draws;
  try { draw(stubCtx, w, h); } finally { seeded = s; draws = d; }
  const t = new THREE.Texture({ width: w, height: h });
  t.colorSpace = THREE.SRGBColorSpace;
  if (repeat) t.wrapS = t.wrapT = THREE.RepeatWrapping;
  t.anisotropy = 4;
  t.userData.tag = 'canvas';
  return t;
}
const textTex = (text, { w = 256, h = 128 } = {}) => canvasTex(w, h, () => {}, false);
const handwrittenTex = (text, opts = {}) => { const o = Object.assign({ w: 1024, h: 220 }, opts); return canvasTex(o.w, o.h, () => {}, false); };
const staticTex = new THREE.Texture({ width: 96, height: 64 });
staticTex.colorSpace = THREE.SRGBColorSpace; staticTex.userData.tag = 'static';
const rainStreakTex = canvasTex(128, 256, () => {}); rainStreakTex.userData.tag = 'rain';

// ------------------------------------------------------------- the code ---
const CODE = [
  lines(42, 45),
  lines(3032, 3044),
  lines(1513, 1540),
  lines(3618, 3627),
  lines(57003, 59034) + `
  __handoff({ cands, taken, exitTile, builders: {
    buildFilingCabinet, buildCardboardStack, buildWetFloorSign, buildVending, buildLocker, buildFireExtinguisher,
    buildTrashCan, buildRubberDuck, buildTrafficCone, buildWheelchair, buildBabyDoll, buildSuitcase,
    buildBrokenMirror, buildTVOnFloor, buildBookStack, buildCoatRack, buildFluorescentTube, buildPile,
    buildPottedPlant, buildFloorPlant, buildWindow, buildElevator, buildPaperPile, buildFloorHole, makeLeafShape,
  } });
}`,
  'return { placeProps, wallAdjSpot, circleHitsWall, rand };',
].join('\n');
const make = new Function('env', 'with (env) {\n' + CODE + '\n}');

function makeEnv(lv) {
  const W = lv.W, H = lv.H, CELL = 4;
  const env = {
    THREE, W, H, CELL, EYE: 1.62, SAN_MAX: 100, tiles: lv.tiles,
    tIdx: (x, z) => z * W + x,
    inGrid: (x, z) => x >= 0 && z >= 0 && x < W && z < H,
    wx: (t) => (t - W / 2 + 0.5) * CELL, wz: (t) => (t - H / 2 + 0.5) * CELL,
    tx: (w) => Math.floor(w / CELL + W / 2), tz: (w) => Math.floor(w / CELL + H / 2),
    toU: (w) => w / CELL + W / 2, toV: (w) => w / CELL + H / 2,
    solids: null, barricadeTiles: [], circleHits: () => false,
    PG: (name, geo, own) => props.propPart(THREE, name, geo, !!own),
    canvasTex, textTex, handwrittenTex, staticTex, rainStreakTex,
    world: new THREE.Group(),
    exitData: null, tvData: null, phoneData: null, clockData: null, manData: null,
    interactables: [], animators: [], almonds: [], batteryItems: [], papers: [], drawerProps: [], rainWindows: [],
    levelNum: lv.levelNum, levelScp: lv.levelScp,
    levelHasStairwell: () => lv.toStairs,
    useExitDoor: (toStairs) => { env.log.push('useExitDoor:' + toStairs); },
    showMsg: (text, dur) => { env.log.push('msg:' + text + '|' + dur); env.lastMsg = text; },
    perfT: 0, AC: null,
    noiseBurst: () => { env.log.push('noiseBurst'); }, blip: () => { env.log.push('blip'); },
    entities: [
      { type: 'smiler', state: 'wander', loseT: 3 },
      { type: 'watcher', state: 'wander', loseT: 2 },
      { type: 'hound', state: 'hunt', loseT: 5 },
    ],
    flashEl: { style: { opacity: '' } },
    setTimeout: (fn, ms) => { env.timeouts.push([fn, ms]); },
    player: {
      pos: new THREE.Vector3(), yaw: 0, sanity: 100,
      fwd() { return new THREE.Vector3(-Math.sin(this.yaw), 0, -Math.cos(this.yaw)); },
    },
    log: [], timeouts: [], lastMsg: '',
    __handoff: (h) => { env.handoff = h; },
  };
  env.isWallTile = (x, z) => !env.inGrid(x, z) || env.tiles[env.tIdx(x, z)] === 1;
  env.sfx = {
    whisper: () => { env.log.push('whisper'); },
    clank: () => { env.noiseBurst(0.12, env.rand(1800, 3200), 6, 0.10); },
  };
  return env;
}

// -------------------------------------------------------- synthetic level ---
function level(seed, size) {
  const n = size % 2 === 0 ? size + 1 : size;
  const r = mulberry32(seed);
  const tiles = new Array(n * n).fill(0);
  for (let z = 0; z < n; z++) for (let x = 0; x < n; x++) {
    const border = x === 0 || z === 0 || x === n - 1 || z === n - 1;
    const lattice = x % 4 === 0 && z % 4 === 0;
    if (border || lattice || (x > 2 && z > 2 && r() < 0.14)) tiles[z * n + x] = 1;
  }
  tiles[n + 1] = 0;
  const bfs = () => {
    const d = new Array(n * n).fill(-1);
    if (tiles[n + 1] !== 0) return d;
    d[n + 1] = 0; const q = [[1, 1]];
    while (q.length) {
      const [x, z] = q.shift();
      for (const [dx, dz] of [[1, 0], [-1, 0], [0, 1], [0, -1]]) {
        const nx = x + dx, nz = z + dz;
        if (nx < 0 || nz < 0 || nx >= n || nz >= n) continue;
        const i = nz * n + nx;
        if (tiles[i] !== 0 || d[i] >= 0) continue;
        d[i] = d[z * n + x] + 1; q.push([nx, nz]);
      }
    }
    return d;
  };
  let dist = bfs();
  for (let i = 0; i < tiles.length; i++) if (tiles[i] === 0 && dist[i] < 0) tiles[i] = 1;
  dist = bfs();
  return { W: n, H: n, tiles, dist };
}

// --------------------------------------------------------------- dumping ---
const v3 = (v) => [v.x, v.y, v.z];
const q4 = (o) => [o.quaternion.x, o.quaternion.y, o.quaternion.z, o.quaternion.w];
function geoParams(g) {
  const p = g.parameters || {};
  const b = (v) => (v ? 1 : 0);
  switch (g.parameters ? (g.type === 'BufferGeometry' ? guessType(p) : g.type) : 'BufferGeometry') {
    case 'BoxGeometry': return ['box', p.width, p.height, p.depth, p.widthSegments, p.heightSegments, p.depthSegments];
    case 'ConeGeometry': return ['cone', p.radius, p.height, p.radialSegments, p.heightSegments, b(p.openEnded), p.thetaStart, p.thetaLength];
    case 'CylinderGeometry': return ['cyl', p.radiusTop, p.radiusBottom, p.height, p.radialSegments, p.heightSegments, b(p.openEnded), p.thetaStart, p.thetaLength];
    case 'SphereGeometry': return ['sphere', p.radius, p.widthSegments, p.heightSegments, p.phiStart, p.phiLength, p.thetaStart, p.thetaLength];
    case 'PlaneGeometry': return ['plane', p.width, p.height, p.widthSegments, p.heightSegments];
    case 'CircleGeometry': return ['circle', p.radius, p.segments, p.thetaStart, p.thetaLength];
    case 'TorusGeometry': return ['torus', p.radius, p.tube, p.radialSegments, p.tubularSegments, p.arc];
    case 'CapsuleGeometry': return ['capsule', p.radius, p.length, p.capSegments, p.radialSegments];
    case 'ShapeGeometry': return ['shape', p.curveSegments];
    default: return ['buffer'];
  }
}
// a prop part keeps its fallback's `parameters` (no type): tell them apart by keys
function guessType(p) {
  if ('depth' in p) return 'BoxGeometry';
  if ('radiusTop' in p) return 'CylinderGeometry';
  if ('capSegments' in p) return 'CapsuleGeometry';
  if ('tube' in p) return 'TorusGeometry';
  if ('radius' in p && 'height' in p) return 'ConeGeometry';
  if ('widthSegments' in p && 'radius' in p) return 'SphereGeometry';
  if ('width' in p) return 'PlaneGeometry';
  if ('segments' in p) return 'CircleGeometry';
  return 'BufferGeometry';
}
function attrSum(a) {
  if (!a) return null;
  let t = 0, ta = 0;
  for (let i = 0; i < a.length; i++) { t += a[i]; ta += Math.abs(a[i]) * ((i % 7) + 1); }
  return [a.length, t, ta];
}
function texInfo(t) {
  if (!t) return null;
  return { tag: t.userData.tag || 'other', w: t.image ? t.image.width : 0, h: t.image ? t.image.height : 0,
    ws: t.wrapS === THREE.RepeatWrapping, wt: t.wrapT === THREE.RepeatWrapping, rp: [t.repeat.x, t.repeat.y] };
}
function dumpTree(root) {
  const geos = new Map(), mats = new Map();
  const gid = (g) => { if (!geos.has(g)) geos.set(g, geos.size); return geos.get(g); };
  const mid = (m) => { if (!mats.has(m)) mats.set(m, mats.size); return mats.get(m); };
  const node = (o) => {
    const k = o.isPointLight ? 'point' : o.isLine ? 'line' : o.isMesh ? 'mesh' : o.isGroup ? 'group' : o.type;
    const n = { k, p: v3(o.position), q: q4(o) };
    if (!o.visible) n.v = false;
    if (o.scale.x !== 1 || o.scale.y !== 1 || o.scale.z !== 1) n.s = v3(o.scale);
    if (o.rotation.order !== 'XYZ') n.ro = o.rotation.order;
    if (o.userData.gameplay) n.gp = true;
    if (o.userData.floats) n.fl = true;
    if (o.isMesh || o.isLine) {
      const g = o.geometry;
      n.g = { gi: gid(g), part: !!g.userData.propPart, t: geoParams(g), vc: g.attributes.position.count, ic: g.index ? g.index.count : -1,
        pos: attrSum(g.attributes.position.array), nor: attrSum(g.attributes.normal && g.attributes.normal.array) };
      const m = o.material;
      n.m = { mi: mid(m), t: m.type, c: m.color.getHex(), e: m.emissive ? m.emissive.getHex() : -1, o: m.opacity, tr: m.transparent,
        sd: m.side, dw: m.depthWrite, bl: m.blending, map: texInfo(m.map) };
    }
    if (o.isPointLight) n.l = { c: o.color.getHex(), i: o.intensity, d: o.distance, dc: o.decay };
    if (o.children.length) n.c = o.children.map(node);
    return n;
  };
  return node(root);
}
const pick = (p) => ({ p: v3(p.g.position), base: p.base, phase: p.phase, taken: p.taken });

// ----------------------------------------------------------------- cases ---
const CASES = [
  { seed: 101, lseed: 11, size: 21, levelNum: 1, levelScp: null, toStairs: false },
  { seed: 202, lseed: 22, size: 29, levelNum: 3, levelScp: 'loop', toStairs: false },
  { seed: 303, lseed: 33, size: 37, levelNum: 2, levelScp: null, toStairs: true },
  { seed: 404, lseed: 44, size: 45, levelNum: 5, levelScp: 'store', toStairs: false },
  { seed: 505, lseed: 55, size: 33, levelNum: 7, levelScp: null, toStairs: false },
  { seed: 606, lseed: 66, size: 25, levelNum: 4, levelScp: null, toStairs: true },
  { seed: 707, lseed: 77, size: 53, levelNum: 9, levelScp: null, toStairs: false },
  { seed: 808, lseed: 88, size: 9, levelNum: 1, levelScp: null, toStairs: false },   // no candidates at all
  { seed: 909, lseed: 99, size: 13, levelNum: 2, levelScp: null, toStairs: false },  // a handful
];

const out = [];
for (const cs of CASES) {
  props.clearPropCache();
  const lv = { ...level(cs.lseed, cs.size), ...cs };
  const env = makeEnv(lv);
  const fns = make(env); env.rand = fns.rand;
  seeded = mulberry32(cs.seed); draws = 0;
  fns.placeProps(lv.dist, null);
  const res = { case: cs, W: lv.W, H: lv.H, tiles: lv.tiles, dist: lv.dist };
  res.draws = draws;
  const h = env.handoff;
  const cd = (c) => [c.x, c.z, c.d === undefined ? 0 : c.d];
  res.handoff = { cands: h.cands.map(cd), taken: h.taken.map(cd), exitTile: cd(h.exitTile) };
  res.tree = dumpTree(env.world);
  res.interactables = env.interactables.map((it) => ({ p: v3(it.pos), r: it.r, prompt: it.prompt, en: !!it.enabled() }));
  res.animators = env.animators.length;
  res.exitData = env.exitData && { pos: v3(env.exitData.pos), normal: v3(env.exitData.normal) };
  res.tvData = env.tvData && { world: v3(env.tvData.world), normal: v3(env.tvData.normal), acc: env.tvData.acc, light: env.tvData.light.intensity };
  res.clockData = env.clockData && { world: v3(env.clockData.world), normal: v3(env.clockData.normal), acc: env.clockData.acc, tock: env.clockData.tock };
  res.phoneData = env.phoneData && { world: v3(env.phoneData.world), normal: v3(env.phoneData.normal), ringing: env.phoneData.ringing, answered: env.phoneData.answered, ringT: env.phoneData.ringT, handset: v3(env.phoneData.handset.position) };
  res.manData = env.manData && { p: v3(env.manData.g.position), timer: env.manData.timer };
  res.almonds = env.almonds.map(pick);
  res.batteryItems = env.batteryItems.map(pick);
  res.drawerProps = env.drawerProps.map((dp) => ({ cab: env.world.children.indexOf(dp.cabinet), drawers: dp.drawers.map((d) => d.contentType || '') }));
  res.rainWindows = env.rainWindows.length;
  res.worldChildren = env.world.children.length;

  // ---- scripted run of the animators (a second seeded stream)
  seeded = mulberry32(cs.seed ^ 0x5eed); draws = 0;
  const sim = [];
  const man = env.manData, ph = env.phoneData;
  for (let k = 0; k < 720; k++) {
    // 600 frames of 50 ms around the mannequin, then 120 of 1/60 s at the phone
    const dt = k < 600 ? 0.05 : 1 / 60;
    const t = k < 600 ? 1 + k * 0.05 : 31 + (k - 600) / 60;
    if (man && k < 600) {
      // circle the mannequin at 3..12 m, looking alternately at it and away
      const a = k * 0.03, rr = 3 + (k % 90) / 10;
      env.player.pos.set(man.g.position.x + Math.cos(a) * rr, 1.62, man.g.position.z + Math.sin(a) * rr);
      env.player.yaw = (Math.floor(k / 40) % 2) ? a : a + Math.PI;
    } else if (ph) {
      env.player.pos.set(ph.world.x + 1, 1.62, ph.world.z + 1);
    }
    if (ph && k === 650) ph.ringing = true;
    env.perfT = t;
    for (const fn of env.animators) fn(dt, t);
    if (k % 30 === 29) {
      sim.push({
        k, draws,
        man: man ? { p: v3(man.g.position), ry: man.g.rotation.y, timer: man.timer } : null,
        handset: ph ? v3(ph.handset.position) : null,
        floats: env.world.children.filter((c) => c.userData.floats).map((c) => v3(c.children[0].position)),
      });
    }
  }
  res.sim = sim;
  res.simDraws = draws;
  res.simPlayer = { p: v3(env.player.pos), yaw: env.player.yaw };

  // ---- use every enabled interactable, in order
  const uses = [];
  env.perfT = 50;
  for (const it of env.interactables) {
    if (!it.enabled()) { uses.push(null); continue; }
    env.log.length = 0; env.lastMsg = '';
    it.use();
    uses.push({ draws, msg: env.lastMsg, log: env.log.slice(), sanity: env.player.sanity, flash: String(env.flashEl.style.opacity) });
  }
  res.uses = uses;
  res.after = {
    draws,
    almonds: env.almonds.map(pick), batteryItems: env.batteryItems.map(pick),
    papers: env.papers.map((p) => ({ p: v3(p.g.position), taken: p.taken, n: p.g.children.length })),
    drawers: env.drawerProps.map((dp) => dp.drawers.map((d) => [d.opened, v3(d.mesh.position)])),
    worldChildren: env.world.children.length,
    entities: env.entities.map((e) => [e.state, e.loseT]),
    phone: ph ? { answered: ph.answered, ringing: ph.ringing, hintUntil: ph.hintUntil === undefined ? null : ph.hintUntil } : null,
    timeouts: env.timeouts.map(([, ms]) => ms),
    enabled: env.interactables.map((it) => !!it.enabled()),
    added: env.world.children.slice(res.worldChildren).map((c) => dumpTree(c)),
  };
  for (const [fn] of env.timeouts) fn();
  res.after.flashAfterTimeouts = String(env.flashEl.style.opacity);
  seeded = null;
  out.push(res);
  console.log(`case ${cs.seed}: ${lv.W}x${lv.H}, draws ${res.draws}, world ${res.worldChildren}, interactables ${res.interactables.length}, animators ${res.animators}, sim draws ${res.simDraws}, after ${res.after.draws}`);
}
fs.writeFileSync(path.join(OUT, 'core_props_head.json'), JSON.stringify(out));

// ---- every builder on its own (the second half calls several of them)
{
  props.clearPropCache();
  const lv = { ...level(1, 21), seed: 1, levelNum: 1, levelScp: null, toStairs: false };
  const env = makeEnv(lv);
  const fns = make(env); env.rand = fns.rand;
  seeded = mulberry32(1); draws = 0;
  fns.placeProps(lv.dist, null);
  const B = env.handoff.builders;
  const calls = [
    ['buildFilingCabinet', []], ['buildCardboardStack', []], ['buildWetFloorSign', []], ['buildVending', []],
    ['buildLocker', []], ['buildFireExtinguisher', []], ['buildTrashCan', []], ['buildRubberDuck', []],
    ['buildTrafficCone', []], ['buildWheelchair', []], ['buildBabyDoll', []], ['buildSuitcase', []],
    ['buildBrokenMirror', [0.7]], ['buildTVOnFloor', []], ['buildBookStack', []], ['buildCoatRack', []],
    ['buildFluorescentTube', []], ['buildPile', []], ['buildPottedPlant', []], ['buildFloorPlant', []],
    ['buildWindow', []], ['buildElevator', [true]], ['buildElevator', [false]], ['buildPaperPile', []], ['buildFloorHole', []],
  ];
  const res = [];
  for (const [name, args] of calls) {
    for (let rep = 0; rep < 6; rep++) {
      const s = 1000 + res.length * 7;
      props.clearPropCache();
      seeded = mulberry32(s); draws = 0;
      const rw = env.rainWindows.length;
      const b = B[name](...args);
      res.push({ name, args, seed: s, draws, top: b.top, tree: dumpTree(b.g),
        drawers: b.drawers ? b.drawers.map((d) => [d.contentType || '', env.world.children.indexOf(d.mesh), b.g.children.indexOf(d.mesh)]) : null,
        rain: env.rainWindows.length - rw,
        rainIdx: env.rainWindows.length > rw ? b.g.children.indexOf(env.rainWindows[env.rainWindows.length - 1]) : -1 });
    }
  }
  for (const [len, wid, color, curl] of [[0.2, 0.07, 0x2a5a2a, 0.2], [0.31, 0.12, 0x357a30, 0.3], [0.13, 0.05, 0x8a7a2a, 0.45]]) {
    seeded = mulberry32(5); draws = 0;
    res.push({ name: 'makeLeafShape', args: [len, wid, color, curl], seed: 5, draws, top: 0, tree: dumpTree(B.makeLeafShape(len, wid, color, curl)), drawers: null, rain: 0, rainIdx: -1 });
  }
  seeded = null;
  fs.writeFileSync(path.join(OUT, 'core_props_builders.json'), JSON.stringify(res));
  console.log('builders:', res.length, 'calls');
}
console.log('wrote', path.join(OUT, 'core_props_head.json'));
