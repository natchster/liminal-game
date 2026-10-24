#!/usr/bin/env node
// Fixture generator for the Rust port of backrooms.js "entities"
// (lines 61153–62850: game/src/entities*.rs).
//
// Runs the original section verbatim in a node vm context with the vendored
// three.js r160, src/creatures.js, a seeded mulberry32 Math.random and the
// JS's own no-solids grid versions of bfs / los / circleHitsWall /
// collideMove / navTileAt / ceilAtWorld (extracted from backrooms.js), and
// writes what it computes to game/tests/fixtures/entities.json. The Rust test
// (game/tests/entities_js.rs) replays the same inputs and compares.
//
//   node tools/en_fixtures.mjs [/path/to/OpenCarole]
import fs from 'node:fs';
import path from 'node:path';
import vm from 'node:vm';
import { fileURLToPath, pathToFileURL } from 'node:url';

const HERE = path.dirname(fileURLToPath(import.meta.url));
const ROOT = process.argv[2] || '/Users/nohwan/Documents/Personal_Projects/OpenCarole';
const OUT = path.join(HERE, '..', 'game', 'tests', 'fixtures');
fs.mkdirSync(OUT, { recursive: true });
const imp = (p) => import(pathToFileURL(path.join(ROOT, p)).href);
const THREE = await imp('vendor/three.module.min.js');
const creatures = await imp('src/creatures.js');

const SRC = fs.readFileSync(path.join(ROOT, 'backrooms.js'), 'utf8');
const LINES = SRC.split('\n');
const lines = (a, b) => LINES.slice(a - 1, b).join('\n');
// a top-level declaration by name, by brace matching (skips strings and comments)
function decl(head) {
  const i = SRC.indexOf(head);
  if (i < 0) throw new Error('no ' + head);
  let j = SRC.indexOf('{', i), depth = 0;
  for (; j < SRC.length; j++) {
    const c = SRC[j];
    if (c === '/' && SRC[j + 1] === '/') { j = SRC.indexOf('\n', j); continue; }
    if (c === '/' && SRC[j + 1] === '*') { j = SRC.indexOf('*/', j) + 1; continue; }
    if (c === "'" || c === '"' || c === '`') { const q = c; j++; while (SRC[j] !== q) { if (SRC[j] === '\\') j++; j++; } continue; }
    if (c === '{') depth++;
    else if (c === '}') { depth--; if (depth === 0) break; }
  }
  let end = j + 1;
  if (SRC[end] === ';') end++;
  return SRC.slice(i, end);
}
if (!LINES[61152].startsWith('// ============================== entities')) throw new Error('entities range moved');
if (LINES[62849].trim() !== 'let heartT = 0;') throw new Error('entities range end moved');

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
let rngFn = mulberry32(1), draws = 0;
const seed = (s) => { rngFn = mulberry32(s); draws = 0; };

// ------------------------------------------------------------------ env ---
const stubCtx = new Proxy({}, {
  get(t, k) {
    if (k in t) return t[k];
    if (typeof k === 'string' && k.startsWith('create')) return () => ({ addColorStop() {} });
    if (k === 'measureText') return () => ({ width: 0 });
    return () => {};
  },
  set(t, k, v) { t[k] = v; return true; },
});
const TEXS = [];
function canvasTex(w, h, draw, repeat = true) {
  draw(stubCtx, w, h);
  const t = new THREE.Texture();
  t.userData.texIndex = TEXS.length;
  t.userData.size = [w, h, repeat];
  TEXS.push(t);
  return t;
}
const timeouts = [];
const env = {
  THREE, canvasTex, creatureGeometry: creatures.creatureGeometry, console,
  CELL: 4, WALL_H: 3.1, SAN_MAX: 100, SAN_REGEN: 3.0, KILL_DIST: 1.05,
  W: 0, H: 0, tiles: null, tileCeil: null, solids: null, barricadeTiles: [],
  scene: new THREE.Scene(), world: new THREE.Group(), camera: new THREE.PerspectiveCamera(74, 1, 0.05, 90),
  levelNum: 1, levelWeather: { key: 'normal' }, state: 'playing', godMode: true, godDeaths: 0,
  player: null,
  soundscape: null, AC: null, droneGain: null, safeRoomLFO: null, safeRoomTremCarrier: null,
  staminaBreathGain: null, staminaBreathLFO: null, staminaBreathLp: null, staminaBreathSubGain: null,
  staminaBreathSubOsc: null, staminaBreathPanner: null, roomToneLp: null,
  sfx: { moan() {}, breath() {}, thump() {}, whisper() {} },
  noiseBurst() {},
  flashEl: { style: {} }, vignetteEl: { style: {} },
  setTimeout: (f, ms) => { timeouts.push([f, ms]); return timeouts.length; },
  shakeT: 0, storeState: { phase: 'day' },
  updateStaff: () => false, updateShadow: () => false,
  humPulsePhase: 0, humPulseAmpCurrent: 0.015, cryDimLerp: 0, knockFlashT: 0, doorOpenedFlashT: 0, elevatorDingFlashT: 0,
  voiceSampleFlashT: 0, staticBurstFlashT: 0, realityShiftT: 0, patternBreakT: 0, clockSkipT: 0,
  chimeFlashT: 0, electricSurgeT: 0, chimeFlashColor: null, chimeFlashLife: 0.9, chimeFlashPeak: 0.25, staticChromaticT: 0,
  currentSeed: 0, perfT: 0, nowMs: 12345,
  regionLit: null,
};
env.performance = { now: () => env.nowMs };
env.onDeath = (cause = 'entity') => {
  if (env.state !== 'playing') return;
  if (env.godMode) { env.godDeaths++; env.player.sanity = Math.max(env.player.sanity, 40); return; }
};
// how lit a region is: a fixed function of the tile (the Rust test uses the same)
env.regionLightAt = (x, z) => (env.regionLit ? env.regionLit[z * env.W + x] ?? 0.5 : 0.5);
const ctx = vm.createContext(env);
vm.runInContext('Math.random = () => { __draw(); return __rng(); };', Object.assign(ctx, { __draw: () => { draws++; }, __rng: () => rngFn() }));
const code = [
  lines(42, 46),   // rand, irand, choice, clamp, lerp
  lines(1386, 1392), 'const toU = (w) => w / CELL + W / 2;\nconst toV = (w) => w / CELL + H / 2;',
  decl('function bfs('), decl('function los('), decl('function circleHitsWall('),
  decl('function ceilAtWorld('), decl('function navTileAt('), decl('function collideMove('),
  decl('const LFO_WEATHER_MUL = '), decl('const CARRIER_WEATHER_DETUNE = '), decl('const ROOM_TONE_BY_WEATHER = '),
  decl('const VIGNETTE_PULSE_AMP_BY_WEATHER = '), 'const VIGNETTE_PULSE_AMP_DEFAULT = 0.015;',
  decl('const CRY_VIGNETTE_PEAK_BY_WEATHER = '), decl('const STATIC_CHROM_BY_WEATHER = '), decl('const STAMINA_BREATH_BY_WEATHER = '),
  decl('const LEVEL_DEPTH_SLOPE_BY_WEATHER = '), decl('const SANITY_HUE_BY_WEATHER = '), decl('const SANITY_DREAD_PERIOD_MS_BY_WEATHER = '),
  decl('function levelDepthMul('), decl('function descentBoostMul('), decl('function descentFreqBoost('),
  'sfx.moan = () => rand(80, 110);',   // the only sfx the section calls that draws
  lines(61153, 62850),
].join('\n');
vm.runInContext(code, ctx, { filename: 'backrooms-entities.js' });
const G = (src) => vm.runInContext(src, ctx);

// ------------------------------------------------------------------ dump helpers ---
const GEO_NAMES = new Map();
for (const n of creatures.CREATURE_PART_NAMES) { const g = creatures.creatureGeometry(THREE, n); if (g) GEO_NAMES.set(g, n); }
const wtGeo = G('wtGeo');
GEO_NAMES.set(wtGeo.body, 'wtBody');
const v3 = (v) => [v.x, v.y, v.z];
const q4 = (q) => [q.x, q.y, q.z, q.w];
function geoId(g) {
  if (GEO_NAMES.has(g)) return ['part', GEO_NAMES.get(g)];
  const p = g.parameters || {};
  switch (g.type) {
    case 'SphereGeometry': return ['sphere', p.radius, p.widthSegments, p.heightSegments];
    case 'PlaneGeometry': return ['plane', p.width, p.height];
    default: return [g.type];
  }
}
const hex = (c) => (c ? c.getHex() : null);
function matInfo(m) {
  return {
    type: m.type, color: hex(m.color), emissive: hex(m.emissive), specular: hex(m.specular), shininess: m.shininess ?? null,
    vc: !!m.vertexColors, tr: !!m.transparent, op: m.opacity, dw: !!m.depthWrite, fog: !!m.fog,
    map: m.map ? m.map.userData.texIndex : null, bump: m.bumpMap ? m.bumpMap.userData.texIndex : null, bumpScale: m.bumpScale ?? null,
    key: m.customProgramCacheKey ? (m.customProgramCacheKey === THREE.Material.prototype.customProgramCacheKey ? '' : m.customProgramCacheKey()) : '',
    vis: m.visible,
  };
}
function tree(o) {
  const n = {
    k: o.isSkinnedMesh ? 'skinned' : o.isMesh ? 'mesh' : o.isBone ? 'bone' : o.isGroup ? 'group' : 'object3d',
    name: o.name, p: v3(o.position), q: q4(o.quaternion), s: v3(o.scale), order: o.rotation.order,
  };
  if (o.userData.tail) n.tail = o.userData.tail;
  if (o.userData.main) n.main = o.userData.main;
  if (o.isMesh) { n.g = geoId(o.geometry); n.m = matInfo(o.material); if (o.morphTargetInfluences) n.inf = o.morphTargetInfluences.slice(); }
  if (o.isSkinnedMesh) {
    n.bones = o.skeleton.bones.map((b) => b.name);
    n.bind = Array.from(o.bindMatrix.elements);
    n.inv = o.skeleton.boneInverses.map((m) => Array.from(m.elements));
  }
  if (o.children.length) n.c = o.children.map(tree);
  return n;
}
// local transforms of every node under g, in traverse order
function pose(o) { const out = []; o.traverse((c) => { out.push([...q4(c.quaternion), ...v3(c.position), ...v3(c.scale)]); }); return out; }

// ------------------------------------------------------------------ grid ---
// an odd grid: border walls, a few pillars and a partition with a gap; tile
// ceilings vary (some low, for the Watcher's bow) and regions are lit unevenly
function setGrid(n, wallFn, ceilFn, litFn) {
  env.W = n; env.H = n;
  env.tiles = new Uint8Array(n * n);
  env.tileCeil = new Float32Array(n * n);
  env.regionLit = new Array(n * n);
  for (let z = 0; z < n; z++) for (let x = 0; x < n; x++) {
    const i = z * n + x;
    env.tiles[i] = (x === 0 || z === 0 || x === n - 1 || z === n - 1 || wallFn(x, z)) ? 1 : 0;
    env.tileCeil[i] = ceilFn(x, z);
    env.regionLit[i] = litFn(x, z);
  }
  return { n, tiles: Array.from(env.tiles), ceil: Array.from(env.tileCeil), lit: env.regionLit.slice() };
}
const GRID_A = (n) => setGrid(n,
  (x, z) => (x % 6 === 3 && z % 6 === 3) || (x === 9 && z > 2 && z < n - 3 && z !== 8),
  (x, z) => ((x + 2 * z) % 7 === 0 ? 2.4 : 3.1),
  (x, z) => ((x * 7 + z * 3) % 5) / 4);
function makePlayer(x, z, yaw) {
  return {
    pos: new THREE.Vector3(x, 0, z), yaw, pitch: 0, sanity: 100, stamina: 100, audibleRange: 0, flashlightOn: false,
    fwd() { return new THREE.Vector3(-Math.sin(this.yaw), 0, -Math.cos(this.yaw)); },
  };
}
const resetWorld = () => { env.scene = ctx.scene = new THREE.Scene(); env.world = ctx.world = new THREE.Group(); G('entities = []; playerField = null; playerFieldT = 0; heartT = 0;'); };

const out = {};

// ------------------------------------------------------------------ tables ---
out.smFingers = G('SM_FINGERS');
out.hdDigits = G('HD_DIGITS');
out.grin = [];
for (let i = 0; i <= 64; i++) { const a = -1.4 + i * (2.8 / 64); out.grin.push([a, ...G(`smGrinAt(${a})`)]); }
out.wrapPi = [-20, -7, -3.2, -1, 0, 1, 3.15, 6.3, 9.5, 31].map((a) => [a, G(`wrapPi(${a})`)]);
out.entSmooth = [];
for (let i = 0; i < 40; i++) { const x = -0.5 + i * 0.05; out.entSmooth.push([x, G(`entSmooth(0.2, 1.1, ${x})`)]); }
out.textures = TEXS.map((t) => t.userData.size);

// ------------------------------------------------------------------ solveLimb ---
{
  seed(777);
  const r = () => rngFn();
  const cases = [];
  for (let i = 0; i < 400; i++) {
    const a = 0.1 + r() * 0.6, b = 0.1 + r() * 0.6;
    const hy = (r() - 0.5) * 2, hz = (r() - 0.5) * 2;
    const reach = (a + b) * (i % 5 === 0 ? 1.4 : i % 7 === 0 ? 0.01 : r());
    const ang = r() * Math.PI * 2;
    const ty = hy + Math.cos(ang) * reach, tz = hz + Math.sin(ang) * reach, fwd = i % 2 === 0;
    G(`solveLimb(${hy}, ${hz}, ${ty}, ${tz}, ${a}, ${b}, ${fwd})`);
    const ik = G('IK');
    cases.push([hy, hz, ty, tz, a, b, fwd ? 1 : 0, ik.t1, ik.t2]);
  }
  out.solveLimb = cases;
}

// ------------------------------------------------------------------ builds ---
{
  out.grid = GRID_A(23);
  resetWorld();
  env.levelNum = 3;
  seed(4242);
  const builds = {};
  for (const [type, x, z] of [['smiler', 5, 6], ['hound', 12, 4], ['watcher', 15, 17]]) {
    const b = G(`(${type === 'hound' ? 'makeHound' : type === 'watcher' ? 'makeWatcher' : 'makeSmiler'})(${x}, ${z})`);
    builds[type] = { x, z, tree: tree(b.g), face: tree(b.face), faceY: b.faceY, inScene: env.scene.children.indexOf(b.face), inWorld: env.world.children.indexOf(b.g) };
  }
  builds.draws = draws;
  out.builds = builds;
  // the shared materials, once
  out.mats = {
    smilerSkin: matInfo(G('smilerSkinMat')), houndSkin: matInfo(G('houndSkinMat')), watcherSheet: matInfo(G('watcherSheetMat')),
    houndHair: matInfo(G('houndHairMat')), smilerTooth: matInfo(G('smilerToothMat')), smilerMaw: matInfo(G('smilerMawMat')),
    houndTooth: matInfo(G('houndToothMat')), houndEye: matInfo(G('houndEyeMat')), smilerEye: matInfo(G('smilerEyeMaterial()')),
  };
  out.texIndex = { skinDetail: G('skinDetailTex').userData.texIndex, cloth: G('watcherClothTex').userData.texIndex,
    smilerFace: G('smilerFaceTex').userData.texIndex, smilerHalo: G('smilerHaloTex').userData.texIndex, houndFace: G('houndFaceTex').userData.texIndex };
}

// ------------------------------------------------------------------ spawnEntities ---
{
  const cases = [];
  for (const [n, lvl, count, s] of [[23, 1, 1, 11], [23, 2, 2, 12], [27, 3, 3, 13], [27, 5, 3, 14], [31, 7, 3, 15], [31, 9, 3, 99], [23, 4, 6, 7], [23, 3, 40, 8]]) {
    GRID_A(n);
    resetWorld();
    env.levelNum = lvl;
    const sx = 1, sz = 1;
    const dist = G(`bfs(${sx}, ${sz})`);
    seed(s);
    G(`spawnEntities(${count}, bfs(1, 1))`);
    const ents = G('entities').map((e) => ({ type: e.type, x: e.pos.x, z: e.pos.z, yaw: e.yaw, speed: e.speed, freeze: e.freeze, eyeY: e.eyeYOffset, faceY: e.faceY }));
    cases.push({ n, level: lvl, count, seed: s, draws, ents, dist: Array.from(dist) });
  }
  out.spawn = cases;
}

// ------------------------------------------------------------------ animateEntity ---
// scripted frames: the entity walks a curve, the state changes, it is stared
// down, it lunges, the player circles it; every 4th frame the whole rig's pose
{
  GRID_A(23);
  const cases = [];
  for (const [type, s] of [['smiler', 31], ['hound', 32], ['watcher', 33], ['smiler', 34], ['hound', 35]]) {
    resetWorld();
    env.levelNum = 4;
    env.player = ctx.player = makePlayer(0, 0, 0);
    seed(s);
    const e = G(`addEntity('${type}', 11, 11, 0)`);
    const frames = [];
    let t = 3.7;
    const N = 360;
    const x0 = e.pos.x, z0 = e.pos.z;
    for (let k = 0; k < N; k++) {
      const dt = [1 / 60, 1 / 30, 1 / 144, 0.05, 0][k % 37 === 5 ? 4 : k % 11 === 0 ? 1 : k % 13 === 0 ? 2 : k % 29 === 0 ? 3 : 0];
      t += dt;
      // the path: still, then a walk, a run, a jump (relocation), a stop
      const ph = k / N;
      const sp = ph < 0.15 ? 0 : ph < 0.4 ? 1.4 : ph < 0.6 ? (s % 2 ? 6.5 : 4.0) : ph < 0.62 ? 0 : ph < 0.85 ? 2.2 : 0;
      const ang = k * 0.013 + s;
      let ex = e.pos.x + Math.sin(ang) * sp * dt, ez = e.pos.z + Math.cos(ang) * sp * dt;
      if (k === 250) { ex += 1.5; }
      const yaw = ang;
      const state = (ph > 0.3 && ph < 0.75) ? 'hunt' : 'wander';
      const holdT = (ph > 0.5 && ph < 0.58) ? 1 : 0;
      const lungeT = (ph > 0.66 && ph < 0.7) ? 0.5 : 0;
      const freeze = (ph < 0.1) ? 1 : 0;
      const px = x0 + Math.sin(k * 0.021) * (3 + 9 * ph), pz = z0 + Math.cos(k * 0.021) * (3 + 9 * ph), pyaw = k * 0.03;
      const observed = type === 'watcher' ? (k < 20 ? undefined : (Math.floor(k / 45) % 2 === 0)) : undefined;
      const inp = { dt, t, ex, ez, yaw, state, holdT, lungeT, freeze, px, pz, pyaw, observed: observed === undefined ? -1 : observed ? 1 : 0 };
      e.pos.x = ex; e.pos.z = ez; e.yaw = yaw; e.state = state; e.holdT = holdT; e.lungeT = lungeT; e.freeze = freeze;
      if (observed !== undefined) e.observed = observed; else delete e.observed;
      env.player.pos.set(px, 0, pz); env.player.yaw = pyaw;
      const d = Math.hypot(e.pos.x - px, e.pos.z - pz);
      inp.d = d;
      G('animateEntity')(e, dt, t, d);
      const rec = { inp };
      if (k % 6 === 0 || k === N - 1) {
        rec.pose = pose(e.g);
        rec.num = {};
        for (const key of ['hunt', 'walk', 'headYaw', 'hdSpeed', 'bound', 'gait', 'crouch', 'listen', 'breath', 'cock', 'hairVX', 'hairVZ', 'hairX', 'hairZ',
          'lunge', 'flinch', 'eyeK', 'breathPh', 'flexPh', 'poseYaw', 'poseStep', 'poseFrom', 'poseS', 'poseMove', 'bow', 'lastX', 'lastZ']) if (typeof e[key] === 'number') rec.num[key] = e[key];
        if (typeof e.poseTaken === 'boolean') rec.poseTaken = e.poseTaken;
        if (e.tic) rec.tic = [...e.tic.p, ...e.tic.v, ...e.tic.g, e.tic.next];
        const P = e.parts;
        if (P.halo) rec.halo = [P.halo.opacity, P.halo.visible ? 1 : 0];
        if (P.far) rec.far = [P.far.opacity, P.far.visible ? 1 : 0];
        if (P.eyeMat) rec.eye = P.eyeMat.emissive.r;
        if (P.sheet && P.sheet.morphTargetInfluences) rec.inf = P.sheet.morphTargetInfluences.slice();
        rec.gry = e.g.rotation.y; rec.gsy = e.g.scale.y;
      }
      rec.draws = draws;
      frames.push(rec);
    }
    cases.push({ type, seed: s, frames });
  }
  out.anim = cases;
}

// ------------------------------------------------------------------ steering ---
{
  GRID_A(23);
  resetWorld();
  env.levelNum = 2;
  env.player = ctx.player = makePlayer(0, 0, 0);
  const res = {};
  // lineWalkable over many segments
  seed(55);
  const lw = [];
  for (let i = 0; i < 300; i++) {
    const ax = (rngFn() - 0.5) * 80, az = (rngFn() - 0.5) * 80, bx = ax + (rngFn() - 0.5) * 30, bz = az + (rngFn() - 0.5) * 30, r = [0.3, 0.4, 0.05][i % 3];
    lw.push([ax, az, bx, bz, r, G(`lineWalkable(${ax}, ${az}, ${bx}, ${bz}, ${r})`) ? 1 : 0]);
  }
  res.lineWalkable = lw;
  // the world queries themselves on this grid (the Rust test's port must agree)
  const q = [];
  for (let i = 0; i < 300; i++) {
    const ax = (rngFn() - 0.5) * 92, az = (rngFn() - 0.5) * 92, bx = (rngFn() - 0.5) * 92, bz = (rngFn() - 0.5) * 92, r = 0.3 + rngFn() * 0.5;
    const p = new THREE.Vector3(ax, 0, az);
    G('collideMove')(p, (bx - ax) * 0.1, (bz - az) * 0.1, r);
    q.push([ax, az, bx, bz, r, G(`los(${ax}, ${az}, ${bx}, ${bz})`) ? 1 : 0, G(`circleHitsWall(${ax}, ${az}, ${r})`) ? 1 : 0, ...G(`navTileAt(${ax}, ${az})`), G(`ceilAtWorld(${ax}, ${az})`), p.x, p.z]);
  }
  res.queries = q;
  res.bfs = [[3, 4], [9, 8], [20, 20], [9, 5]].map(([x, z]) => [x, z, Array.from(G(`bfs(${x}, ${z})`))]);
  // entityTurn
  seed(56);
  const e = G(`addEntity('smiler', 4, 4, 0)`);
  const turns = [];
  for (let i = 0; i < 200; i++) {
    const yaw = (rngFn() - 0.5) * 14, target = (rngFn() - 0.5) * 14, dt = rngFn() * 0.1, rate = i % 3 === 0 ? rngFn() * 5 : undefined;
    e.yaw = yaw; e.type = ['smiler', 'hound', 'watcher', 'staff', 'shadow', 'gone'][i % 6];
    G('entityTurn')(e, target, dt, rate);
    turns.push([yaw, target, dt, rate === undefined ? -1 : rate, i % 6, e.yaw]);
  }
  e.type = 'smiler';
  res.turns = turns;
  // walks: stepToward with a field and a moving target, the drive, re-aims
  const walks = [];
  for (const [type, sx, sz, gx, gz, direct, speed, s] of [
    ['smiler', 2, 2, 17, 15, true, 5.5, 61], ['hound', 18, 4, 2, 19, true, 7.2, 62], ['watcher', 4, 18, 15, 6, true, 3.2, 63],
    ['smiler', 12, 12, 3, 4, false, 1.7, 64], ['hound', 2, 10, 20, 10, false, 1.7, 65],
    ['smiler', 2, 2, 7, 5, true, 5.5, 66], ['hound', 12, 12, 14, 16, true, 7.2, 67], ['smiler', 12, 12, 8, 11, false, 1.7, 68],
    ['watcher', 13, 5, 10, 8, true, 3.2, 69], ['staff', 16, 16, 20, 20, false, 1.05, 70], ['shadow', 20, 2, 16, 4, true, 2.1, 71],
    ['smiler', 12, 12, 13, 14, false, 1.7, 72], ['hound', 16, 10, 17, 11, false, 1.7, 73],
  ]) {
    resetWorld();
    seed(s);
    const en = G(`addEntity('${type === 'staff' || type === 'shadow' ? 'smiler' : type}', ${sx}, ${sz}, 0)`);
    en.type = type;   // staff / shadow steer with their own motion constants
    en.freeze = 0;
    const field = G(`bfs(${gx}, ${gz})`);
    let field2 = field;
    const tgt = new THREE.Vector3(G(`wx(${gx})`), 0, G(`wz(${gz})`));
    const steps = [];
    for (let k = 0; k < 420; k++) {
      const dt = k % 9 === 0 ? 1 / 30 : 1 / 60;
      if (k === 200) field2 = G(`bfs(${gx}, ${gz})`);   // a new field object: a re-aim
      tgt.x += Math.sin(k * 0.05) * 0.03; tgt.z += Math.cos(k * 0.04) * 0.03;
      if (k === 150) en.lungeT = 0.6;
      if (k > 150) en.lungeT = Math.max(0, (en.lungeT || 0) - dt);
      if (type === 'hound') en.state = k > 100 ? 'hunt' : 'wander';
      const done = G('entityStepToward')(en, field2, dt, speed, direct ? tgt : null);
      steps.push([dt, tgt.x, tgt.z, en.lungeT || 0, en.state === 'hunt' ? 1 : 0, en.pos.x, en.pos.z, en.vx, en.vz, en.yaw, en.aimX, en.aimZ, en.aimFinal ? 1 : 0, en.aimOk ? 1 : 0, en.aimT, done ? 1 : 0]);
    }
    walks.push({ type, sx, sz, gx, gz, direct: direct ? 1 : 0, speed, seed: s, steps });
  }
  res.walks = walks;
  // look rules: a smiler and a hound, the player turning to and from them
  const looks = [];
  for (const [type, s] of [['smiler', 71], ['hound', 72]]) {
    resetWorld();
    seed(s);
    const en = G(`addEntity('${type}', 11, 11, 0)`);
    const steps = [];
    for (let k = 0; k < 600; k++) {
      const dt = 1 / 60;
      const px = en.pos.x + Math.sin(k * 0.01) * (4 + (k % 200) * 0.05), pz = en.pos.z + Math.cos(k * 0.01) * (4 + (k % 200) * 0.05);
      env.player.pos.set(px, 0, pz);
      env.player.yaw = Math.atan2(px - en.pos.x, pz - en.pos.z) + (Math.floor(k / 60) % 2 ? 0 : Math.PI) + Math.sin(k * 0.1) * 0.3;
      env.player.audibleRange = (Math.floor(k / 150) % 2) ? 14 : 3;
      const d = Math.hypot(en.pos.x - px, en.pos.z - pz), seen = (k % 7) !== 0;
      const m = G('entityLookRule')(en, d, seen, dt);
      steps.push([px, pz, env.player.yaw, env.player.audibleRange, d, seen ? 1 : 0, m, en.lungeT || 0, en.holdT || 0, en.stareCd || 0, en.wasWatched ? 1 : 0]);
    }
    looks.push({ type, seed: s, steps });
  }
  res.looks = looks;
  out.steer = res;
}

// ------------------------------------------------------------------ updateEntities ---
// a whole level's worth of frames: three of them and a player walking about
{
  const cases = [];
  for (const [lvl, s, weather, place, path = place ? 'circle' : 'loop'] of [[5, 91, 'normal', [[5, 3], [8, 6], [4, 6]]], [8, 92, 'wet', null], [3, 93, 'haunted', null],
    [5, 91, 'still', [[14, 14], [20, 20], [7, 4]]], [8, 92, 'electric', [[6, 4], [11, 10], [16, 5]]],
    [5, 91, 'derelict', [[20, 20], [20, 2], [5, 4]], 'still']]) {
    const grid = GRID_A(27);
    resetWorld();
    env.levelNum = lvl; env.levelWeather = ctx.levelWeather = { key: weather };
    env.player = ctx.player = makePlayer(G('wx(2)'), G('wz(2)'), 0);
    Object.assign(env, { shakeT: 0, humPulsePhase: 0, humPulseAmpCurrent: 0.015, knockFlashT: 0.5, doorOpenedFlashT: 0.7, elevatorDingFlashT: 1.0,
      voiceSampleFlashT: 0.6, staticBurstFlashT: 0.4, realityShiftT: 1.2, patternBreakT: 2, clockSkipT: 0.3, chimeFlashT: 0.8, chimeFlashColor: 'rgba(200, 180, 90',
      staticChromaticT: 0.3, electricSurgeT: 0.2, cryDimLerp: 0.5, currentSeed: 0x12345678, godDeaths: 0 });
    env.vignetteEl.style = {}; env.flashEl.style = {};
    seed(s);
    G(`spawnEntities(3, bfs(1, 1))`);
    const spawnDraws = draws;
    // the first creature starts close by, so it sees you
    const es = G('entities');
    if (place) place.forEach(([x, z], i) => es[i].pos.set(G(`wx(${x})`), 0, G(`wz(${z})`)));
    else es[0].pos.set(G('wx(5)'), 0, G('wz(3)'));
    const frames = [];
    const N = 900;
    for (let k = 0; k < N; k++) {
      const dt = k % 17 === 0 ? 1 / 30 : 1 / 60;
      const t = 10 + k / 60;
      const ph = k / N;
      // the player wanders round the room, sometimes running with the lamp on
      const px = path === 'still' ? G('wx(2)') : path === 'circle' ? G('wx(8)') + Math.cos(k * 0.005) * 14 : G('wx(2)') + Math.sin(k * 0.006) * 30 + 30;
      const pz = path === 'still' ? G('wz(2)') : path === 'circle' ? G('wz(8)') + Math.sin(k * 0.005) * 14 : G('wz(2)') + (1 - Math.cos(k * 0.004)) * 20;
      const p = new THREE.Vector3(px, 0, pz);
      G('collideMove')(env.player.pos, p.x - env.player.pos.x, p.z - env.player.pos.z, 0.35);
      // 'still': back turned on the third creature, glancing round now and then
      env.player.yaw = path === 'still'
        ? Math.atan2(es[2].pos.x - env.player.pos.x, es[2].pos.z - env.player.pos.z) + 0.2 * Math.sin(k * 0.01) + (k % 300 > 240 ? Math.PI : 0)
        : k * 0.02 + Math.sin(k * 0.05);
      env.player.audibleRange = ph < 0.3 ? 0 : ph < 0.5 ? 8 : ph < 0.6 ? 14 : 0.5;
      env.player.flashlightOn = ph > 0.4 && ph < 0.8;
      if (k === 450) env.player.sanity = 20;
      env.camera.position.set(env.player.pos.x, 1.62, env.player.pos.z);
      env.perfT = t;
      G('updateEntities')(dt, t);
      frames.push({
        full: k % 3 === 0 || k === N - 1 ? 1 : 0,
        inp: [dt, t, env.player.pos.x, env.player.pos.z, env.player.yaw, env.player.audibleRange, env.player.flashlightOn ? 1 : 0],
        ents: !(k % 3 === 0 || k === N - 1) ? [] : es.map((e) => [e.pos.x, e.pos.z, e.yaw, e.state === 'hunt' ? 1 : 0, e.loseT, e.freeze, e.stareT, e.wTimer, e.unseenT, e.vx || 0, e.vz || 0, e.stepAcc,
          e.face.position.x, e.face.position.y, e.face.position.z]),
        san: env.player.sanity, vig: Number(env.vignetteEl.style.opacity), heart: G('heartT'), shake: env.shakeT, deaths: env.godDeaths, draws,
        filter: env.vignetteEl.style.filter || '', box: env.vignetteEl.style.boxShadow || '', hum: env.humPulsePhase, flash: env.flashEl.style.opacity ?? null,
      });
    }
    cases.push({ level: lvl, seed: s, weather, place, path, grid, spawnDraws, types: es.map((e) => e.type), frames });
  }
  out.update = cases;
}

fs.writeFileSync(path.join(OUT, 'entities.json'), JSON.stringify(out));
console.log('entities.json', (fs.statSync(path.join(OUT, 'entities.json')).size / 1e6).toFixed(2), 'MB',
  'builds draws', out.builds.draws, 'spawn', out.spawn.map((c) => c.ents.map((e) => e.type[0]).join('')).join(' '),
  'update types', out.update.map((c) => c.types.join(',')).join(' | '));
