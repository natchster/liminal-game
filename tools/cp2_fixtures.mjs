// Fixture generator for placeProps' second half (game/src/core_props_tail*.rs).
//
// Runs the original `placeProps(dist, m4)` of backrooms.js (lines 57003–61151,
// read-only, evaluated from its source text) under node with the vendored
// three.js r160, a seeded mulberry32 `Math.random`, src/props.js for PG, the
// real JS helpers it calls (wallAdjSpot, altPathExists, the weather tables,
// the lore tables, ...) and stubs for the browser (canvasTex draws with a
// throwaway generator like the game's nativeRandom; handwrittenTex/textTex
// record their arguments). Two hooks are spliced into the source text:
//   * at line 59036 (the hand-off to the second half) the shuffled `cands`,
//     `taken`, the draw count, the exit position and the pickups are recorded;
//   * every builder call in the second half (`buildX(...)`) is wrapped to log
//     its name, arguments, draws used and `top`, and to mark what it made.
// Then it dumps everything the second half built (world subtree, materials,
// textures, lights, interactables, animators, records) and runs a scripted
// play session (animators, interactables, flipSwitch, timers) on a second
// seeded stream, dumping the state again.
//
//   node tools/cp2_fixtures.mjs [/path/to/OpenCarole]
// writes game/tests/fixtures/cp2_tail.json
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const HERE = path.dirname(fileURLToPath(import.meta.url));
const ROOT = process.argv[2] || '/Users/nohwan/Documents/Personal_Projects/OpenCarole';
const OUT = path.join(HERE, '..', 'game', 'tests', 'fixtures');
fs.mkdirSync(OUT, { recursive: true });

globalThis.__threeUuidRandom = Math.random;
const THREE = await import(pathToFileURL(path.join(ROOT, 'vendor/three.module.min.js')).href);
const props = await import(pathToFileURL(path.join(ROOT, 'src/props.js')).href);
const SRC = fs.readFileSync(path.join(ROOT, 'backrooms.js'), 'utf8');
const LINES = SRC.split('\n');

// ------------------------------------------------------------ source slicing
// end index of a balanced {...}/[...]/(...) block starting at the first opener at or after i
function blockEnd(s, i) {
  let depth = 0, started = false;
  for (; i < s.length; i++) {
    const c = s[i];
    if (c === '/' && s[i + 1] === '/') { while (i < s.length && s[i] !== '\n') i++; continue; }
    if (c === '/' && s[i + 1] === '*') { i = s.indexOf('*/', i + 2) + 1; continue; }
    if (c === '"' || c === "'" || c === '`') {
      const q = c; i++;
      while (i < s.length && s[i] !== q) { if (s[i] === '\\') i++; i++; }
      continue;
    }
    if (c === '{' || c === '[' || c === '(') { depth++; started = true; }
    else if (c === '}' || c === ']' || c === ')') { depth--; if (started && depth === 0) return i + 1; }
  }
  throw new Error('unbalanced');
}
// the full top-level declaration `function NAME ...` / `const NAME = ...;`
function decl(name) {
  const re = new RegExp(`^(function ${name}\\b|const ${name}\\b|let ${name}\\b)`, 'm');
  const m = re.exec(SRC);
  if (!m) throw new Error('no decl ' + name);
  const start = m.index;
  if (m[0].startsWith('function')) return SRC.slice(start, blockEnd(SRC, SRC.indexOf('{', start)));
  const eq = SRC.indexOf('=', start);
  let j = eq + 1; while (SRC[j] === ' ') j++;
  if (SRC[j] === '{' || SRC[j] === '[') { const e = blockEnd(SRC, j); return SRC.slice(start, SRC.indexOf(';', e) + 1); }
  return SRC.slice(start, SRC.indexOf('\n', start));
}
const lineAt = (n) => LINES[n - 1];
const PP_START = 57003, PP_END = 61151, HANDOFF = 59036;
if (!lineAt(PP_START).startsWith('function placeProps(dist, m4) {')) throw new Error('placeProps moved');
if (lineAt(PP_END) !== '}') throw new Error('placeProps end moved');
if (!lineAt(HANDOFF).includes('WEATHER-DRIVEN EXTRA PROPS')) throw new Error('hand-off moved');
const BUILDERS = ['buildFilingCabinet', 'buildBookStack', 'buildCardboardStack', 'buildTrafficCone', 'buildWheelchair',
  'buildBabyDoll', 'buildPottedPlant', 'buildFloorPlant', 'buildWindow', 'buildPaperPile', 'buildFloorHole',
  'buildBrokenMirror', 'buildElevator'];
const head = LINES.slice(PP_START - 1, HANDOFF - 1).join('\n');
let tail = LINES.slice(HANDOFF - 1, PP_END).join('\n');
for (const b of BUILDERS) tail = tail.replace(new RegExp(`\\b${b}\\(`, 'g'), `__B('${b}', ${b})(`);
const placePropsSrc = head + '\n  __handoff(cands, taken);\n' + tail;

const DECLS = ['rand', 'irand', 'choice', 'clamp', 'lerp', 'CELL', 'WALL_H', 'EYE',
  'darkMat', 'woodMat', 'metalMat', 'chromeMat', 'rustMat', 'plasticYellowMat', 'plasticOrangeMat', 'plasticPinkMat',
  'plasticRedMat', 'plasticWhiteMat', 'cardboardMat', 'paperMat', 'glassMat',
  'wallAdjSpot', 'altPathExists', 'weatherPick', 'levelDepthMul', 'descentBoostMul', 'descentFreqBoost', 'beamBatteryScale',
  'LEVEL_DEPTH_SLOPE_BY_WEATHER', 'FLAME_BREATH_AMP_BY_WEATHER', 'BATTERY_HINT_DIPS', 'TAIL_BEAT_SPECS', 'RELIGHT_SPECS',
  'FIZZLE_PROB_BY_WEATHER', 'FIZZLE_ECHO_PROB_BY_WEATHER', 'BLOWOUT_MILESTONE_AT', 'RELIGHT_MILESTONE_AT',
  'LORE_ENTRIES', 'LORE_ARCS', 'loreArcFor', 'loreNoteBoosted'];
let declSrc = DECLS.map(decl).join('\n');
declSrc += "\nfor (const id of [36, 37, 38, 39, 40, 41]) LORE_ENTRIES[id].scp = 'stairwell';\n";
// SAN_MAX / BAT_MAX share a line with other constants
declSrc += 'const SAN_MAX = 100, BAT_MAX = 100;\n';
// the two sfx entries placeProps calls (sfx.chime draws three rand()s even without audio)
if (!lineAt(1268).trim().startsWith('breath:') || !lineAt(1274).trim().startsWith('chime:')) throw new Error('sfx moved');
declSrc += `const sfx = {\n${lineAt(1268)}\n${LINES.slice(1273, 1279).join('\n')}\n};\n`;

// ------------------------------------------------------------------- stubs
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
const unknown = new Set();
function anything(name) {
  const f = function () {};
  return new Proxy(f, {
    get(t, k) {
      if (k === Symbol.toPrimitive) return () => 0;
      if (k === Symbol.iterator) return function* () {};
      if (k === 'then') return undefined;
      if (k === 'length') return 0;
      return anything(name + '.' + String(k));
    },
    apply() { return anything(name + '()'); },
    construct() { return anything('new ' + name); },
    set() { return true; },
  });
}
const stubCtx = new Proxy({}, {
  get(t, k) {
    if (k in t) return t[k];
    if (typeof k === 'string' && k.startsWith('create')) return () => ({ addColorStop() {} });
    if (k === 'measureText') return () => ({ width: 10 });
    if (k === 'getImageData' || k === 'createImageData') return (x, y, w, h) => ({ width: w || x, height: h || y, data: new Uint8ClampedArray(4 * (w || x) * (h || y)) });
    return () => {};
  },
  set(t, k, v) { t[k] = v; return true; },
});

// one level setup + run; returns the fixture case
function runCase(spec) {
  const { name, size, seed, weather, level, scp, loreFound, pool, runtimeSeed } = spec;
  // ---- synthetic level (border walls, a lattice of wall stubs, random interior walls)
  const W = size, H = size;
  const tiles = new Uint8Array(W * H);
  const lrng = mulberry32(seed ^ 0x5bd1e995);
  for (let z = 0; z < H; z++) for (let x = 0; x < W; x++) {
    const border = x === 0 || z === 0 || x === W - 1 || z === H - 1;
    const lattice = x % 4 === 0 && z % 4 === 0;
    if (border || lattice || (x > 2 && z > 2 && lrng() < 0.12)) tiles[z * W + x] = 1;
  }
  tiles[W + 1] = 0;
  const bfs = () => {
    const d = new Int32Array(W * H).fill(-1); const q = [[1, 1]]; d[W + 1] = 0;
    for (let qi = 0; qi < q.length; qi++) {
      const [x, z] = q[qi];
      for (const [dx, dz] of [[1, 0], [-1, 0], [0, 1], [0, -1]]) {
        const nx = x + dx, nz = z + dz;
        if (nx < 0 || nz < 0 || nx >= W || nz >= H) continue;
        const i = nz * W + nx; if (tiles[i] !== 0 || d[i] >= 0) continue;
        d[i] = d[z * W + x] + 1; q.push([nx, nz]);
      }
    }
    return d;
  };
  let dist = bfs();
  for (let i = 0; i < tiles.length; i++) if (tiles[i] === 0 && dist[i] < 0) tiles[i] = 1;
  dist = bfs();
  const wx = (t) => (t - W / 2 + 0.5) * 4, wz = (t) => (t - H / 2 + 0.5) * 4;
  // ---- buildWorld's pool pre-scan (line 4525)
  let poolPlan = null;
  if (pool) {
    const isWall = (x, z) => x < 0 || z < 0 || x >= W || z >= H || tiles[z * W + x] === 1;
    outer: for (const [pw, ph] of [[3, 3], [4, 3], [3, 4], [4, 4], [3, 2], [2, 3]]) {
      for (let z = 2; z <= H - 2 - ph; z++) for (let x = 2; x <= W - 2 - pw; x++) {
        let allOpen = true;
        for (let zz = 0; zz < ph && allOpen; zz++) for (let xx = 0; xx < pw && allOpen; xx++) if (isWall(x + xx, z + zz)) allOpen = false;
        if (!allOpen) continue;
        const cxT = wx(x + pw / 2 - 0.5), czT = wz(z + ph / 2 - 0.5);
        if (Math.hypot(cxT - wx(1), czT - wz(1)) < 8) continue;
        poolPlan = { x, z, pw, ph }; break outer;
      }
    }
  }
  // ---- panels: one per open tile with (x + z) % 3 == 0
  const panels = [];
  for (let z = 1; z < H - 1; z++) for (let x = 1; x < W - 1; x++) if (tiles[z * W + x] === 0 && (x * 7 + z * 3) % 5 === 0) panels.push({ wx: wx(x), wz: wz(z), switchedOff: false });

  // ---- the environment every free identifier of placeProps resolves through
  const seeded = mulberry32(seed);
  let draws = 0;
  const seededCounted = () => { draws++; return seeded(); };
  const native = mulberry32(0xdeadbeef ^ seed);
  const envMath = Object.create(Math);
  envMath.random = seededCounted;
  const texLog = [];
  const mkTex = (info) => { const t = new THREE.Texture(); t.userData.info = info; texLog.push(info); return t; };
  let now = 0, timerId = 1; const timers = [];
  const msgs = [];
  const simMsgs = []; let simFrame = -1, simPhase = 'build';
  const camera = new THREE.PerspectiveCamera(74, 1, 0.05, 90);
  const player = { pos: new THREE.Vector3(wx(1), 0, wz(1)), sanity: 100, battery: 100, flashlightOn: false };
  const flashEl = { style: {} }, aberrationEl = { style: {} };
  const handoff = {};
  const builderLog = [];
  const env = {
    THREE, Math: envMath, W, H, tiles,
    tIdx: (x, z) => z * W + x,
    inGrid: (x, z) => x >= 0 && z >= 0 && x < W && z < H,
    isWallTile: (x, z) => !(x >= 0 && z >= 0 && x < W && z < H) || tiles[z * W + x] === 1,
    wx, wz, tx: (w) => Math.floor(w / 4 + W / 2), tz: (w) => Math.floor(w / 4 + H / 2),
    PG: (n, g, own) => props.propPart(THREE, n, g, own),
    canvasTex: (w, h, draw, repeat = true) => {
      const prev = envMath.random; envMath.random = native;
      try { draw(stubCtx, w, h); } finally { envMath.random = prev; }
      const t = mkTex({ kind: 'canvas', w, h, repeat });
      if (repeat) t.wrapS = t.wrapT = THREE.RepeatWrapping;
      return t;
    },
    handwrittenTex: (text, opts = {}) => mkTex({ kind: 'hand', text, opts: JSON.parse(JSON.stringify(opts)) }),
    textTex: (text, opts = {}) => mkTex({ kind: 'text', text, opts: { ...opts } }),
    world: new THREE.Group(), scene: new THREE.Scene(), camera, player,
    animators: [], interactables: [], panels, almonds: [], batteryItems: [], papers: [], loreNotes: [], lightSwitches: [],
    hauntedCandles: [], floorHoles: [], barricadeTiles: [], drawerProps: [], paperMarkers: [], entities: [],
    levelWeather: { key: weather }, levelNum: level, levelScp: scp, loreFound: new Set(loreFound), loreArcsCompleted: new Set(),
    window: { __poolPlan: poolPlan }, location: { hash: '' }, AC: null, safeRoomLFO: null,
    spawnPos: new THREE.Vector3(wx(1), 0, wz(1)),
    exitData: null, poolBounds: null, vacuumBot: null, safeRoom: null, brokenElevator: null,
    candleBlowoutTally: 0, candleRelightTally: 0, candleFizzleTally: 0, blowoutMilestoneShown: false, relightMilestoneShown: false,
    state: 'playing', shakeT: 0, cameraDipT: 0, flashEl, aberrationEl,
    showMsg: (t, d) => { msgs.push(`${t}|${d}`); simMsgs.push([simFrame, simPhase, t, d]); },
    nextLevelViaElevator: () => msgs.push('#nextLevelViaElevator'),
    setTimeout: (fn, ms) => { const id = timerId++; timers.push({ id, at: now + Math.max(0, ms || 0), fn }); return id; },
    clearTimeout: (id) => { const i = timers.findIndex((t) => t.id === id); if (i >= 0) timers.splice(i, 1); },
    noiseBurst: () => {}, blip: () => {},
    console,
    __handoff: (cands, taken) => {
      handoff.draws = draws;
      handoff.cands = cands.map((c) => [c.x, c.z, c.d]);
      handoff.taken = taken.map((c) => [c.x, c.z, c.d ?? -1]);
      handoff.world = env.world.children.length;
      handoff.interactables = env.interactables.length;
      handoff.animators = env.animators.length;
      handoff.exit = env.exitData ? [env.exitData.pos.x, env.exitData.pos.y, env.exitData.pos.z] : null;
      handoff.almonds = env.almonds.map((a) => ({ p: v3(a.g.position), base: a.base, phase: a.phase, taken: a.taken }));
      handoff.batteries = env.batteryItems.map((a) => ({ p: v3(a.g.position), base: a.base, phase: a.phase, taken: a.taken }));
      handoff.papers = env.papers.length; handoff.loreNotes = env.loreNotes.length; handoff.lightSwitches = env.lightSwitches.length;
      handoff.floorHoles = env.floorHoles.length; handoff.hauntedCandles = env.hauntedCandles.length;
      handoff.tail = true;
    },
    __B: (bname, fn) => (...args) => {
      const d0 = draws, w0 = env.world.children.length, i0 = env.interactables.length, a0 = env.animators.length;
      const r = fn(...args);
      const sideW = env.world.children.slice(w0);
      for (const o of sideW) o.userData.__side = bname;
      for (const it of env.interactables.slice(i0)) it.__side = bname;
      for (const an of env.animators.slice(a0)) an.__side = bname;
      r.g.userData.__builder = bname;
      builderLog.push({ name: bname, args: args.map((a) => (typeof a === 'boolean' ? (a ? 1 : 0) : a)), at: d0 - handoff.draws, used: draws - d0, top: r.top ?? 0 });
      return r;
    },
  };
  const proxy = new Proxy(env, {
    has: (t, k) => typeof k === 'string',
    get(t, k) {
      if (k === Symbol.unscopables) return undefined;
      if (k in t) return t[k];
      if (k in globalThis) return globalThis[k];
      unknown.add(k);
      return (t[k] = anything(k));
    },
    set(t, k, v) { t[k] = v; return true; },
  });
  const factory = new Function('env', `with (env) { return (function () {\n${declSrc}\n${placePropsSrc}\nreturn { placeProps, LORE_ENTRIES, LORE_ARCS };\n})(); }`);
  props.clearPropCache();
  const api = factory(proxy);
  const drawsBefore = draws;
  api.placeProps(Array.from(dist), new THREE.Matrix4());
  if (!handoff.tail) throw new Error('hand-off not reached');
  const out = { name, spec, W, H, tiles: Array.from(tiles), panels: panels.map((p) => [p.wx, p.wz]), poolPlan: poolPlan && [poolPlan.x, poolPlan.z, poolPlan.pw, poolPlan.ph] };
  out.handoff = handoff;
  out.tailDraws = draws - handoff.draws;
  out.builders = builderLog;
  out.lore = { scp: api.LORE_ENTRIES.map((e, i) => { if (e.id !== i) throw new Error('lore id != index'); return e.scp || null; }), arcs: api.LORE_ARCS.map((a) => [a.ids, a.color]) };
  void drawsBefore;
  // ---- what the second half made
  const tailWorld = env.world.children.slice(handoff.world).filter((o) => !o.userData.__side);
  out.build = dumpState(env, tailWorld, handoff, camera);
  const buildSet = new Set(tailWorld);
  out.texs = texLog.length;
  out.msgs = msgs.slice();
  // ---- a scripted play session on a second seeded stream
  const rrng = mulberry32(runtimeSeed);
  let rdraws = 0;
  envMath.random = () => { rdraws++; return rrng(); };
  // drop what builders made (the Rust test's stub builders make none of it)
  env.animators = env.animators.slice(handoff.animators).filter((a) => !a.__side);
  env.interactables = env.interactables.slice(handoff.interactables).filter((i) => !i.__side);
  const sim = spec.sim;
  // where the player parks during sim.stick: the first candle / the first sparking wire bundle
  let stickPos = null;
  if (sim.stickTo === 'candle') stickPos = out.build.candlePos[0] || null;
  if (sim.stickTo === 'wire') {
    const w = tailWorld.find((o) => o.isGroup && o.children.some((c) => c.isPointLight && c.color.getHex() === 0xfff2a0));
    stickPos = w ? v3(w.position) : null;
  }
  out.stickPos = stickPos;
  const frames = [];
  let t = 10;
  for (let f = 0; f < sim.frames; f++) {
    simFrame = f;
    const dt = f % 7 === 3 ? 0.05 : 1 / 60;
    t += dt; now += dt * 1000;
    // the player walks a loop through the level; the camera follows
    const k = f / sim.frames;
    player.pos.set(wx(1) + Math.sin(k * 6.283) * W * 1.2, 0, wz(1) + (1 - Math.cos(k * 6.283)) * H * 0.9);
    if (sim.stick && stickPos && f >= sim.stick[0] && f < sim.stick[1]) player.pos.set(stickPos[0] + 0.3, 0, stickPos[2] + 0.2);
    player.flashlightOn = f > sim.frames / 3;
    if (f === sim.lowBattery) player.battery = 2.5;
    if (f === sim.lowBattery + 40) player.battery = 60;
    camera.position.set(player.pos.x, 1.62, player.pos.z);
    // timers due
    simPhase = 'timer';
    for (;;) {
      let bi = -1;
      for (let i = 0; i < timers.length; i++) if (timers[i].at <= now && (bi < 0 || timers[i].at < timers[bi].at)) bi = i;
      if (bi < 0) break;
      const tm = timers.splice(bi, 1)[0]; tm.fn();
    }
    simPhase = 'anim';
    for (const fn of env.animators.slice()) fn(dt, t);
    simPhase = 'use';
    if (sim.use.includes(f)) {
      for (let i = 0; i < env.interactables.length; i++) {
        const it = env.interactables[i];
        if (it.prompt === '[E] CALL THE ELEVATOR') continue;   // leaves the level (tested on its own)
        if (it.enabled()) it.use();
      }
    }
    simPhase = 'flip';
    if (sim.flip.includes(f)) for (let i = 0; i < env.lightSwitches.length; i++) env.lightSwitches[i].flip(i % 2 ? 'motion' : 'hand');
    if (sim.snap.includes(f)) frames.push({ f, t, now, state: dumpRuntime(env, handoff, camera) });
  }
  // the last message of each (frame, phase) that showed one
  const last = [];
  for (const m of simMsgs) {
    if (m[0] < 0) continue;
    if (last.length && last[last.length - 1][0] === m[0] && last[last.length - 1][1] === m[1]) last[last.length - 1] = m;
    else last.push(m);
  }
  out.sim = { frames, msgs: last, draws: rdraws };
  out.final = dumpRuntime(env, handoff);
  // objects the session added (scorches, smoke still alive), in full
  out.final.added = env.world.children.slice(handoff.world).filter((o) => !o.userData.__side && !buildSet.has(o)).map(tree);
  out.final.panelsOff = env.panels.map((p) => (p.switchedOff ? 1 : 0));
  out.finalPlayer = { sanity: player.sanity, battery: player.battery };
  out.finalGlobals = { shakeT: env.shakeT, cameraDipT: env.cameraDipT, flash: { ...flashEl.style }, aberration: { ...aberrationEl.style },
    tallies: [env.candleBlowoutTally, env.candleRelightTally, env.candleFizzleTally], timers: timers.length };
  return out;
}

// -------------------------------------------------------------- dumping
const v3 = (v) => [v.x, v.y, v.z];
function texInfo(t) {
  if (!t) return null;
  const i = t.userData.info || { kind: 'other' };
  return { ...i, ws: t.wrapS === THREE.RepeatWrapping ? 1 : 0, wt: t.wrapT === THREE.RepeatWrapping ? 1 : 0, rep: [t.repeat.x, t.repeat.y] };
}
function matInfo(m) {
  return {
    type: m.type, color: m.color ? m.color.getHex() : null, emissive: m.emissive ? m.emissive.getHex() : null,
    op: m.opacity, tr: m.transparent ? 1 : 0, dw: m.depthWrite ? 1 : 0, side: m.side, bl: m.blending, map: texInfo(m.map),
  };
}
function geoInfo(g) {
  const p = g.parameters || {};
  const a = g.attributes.position.array;
  let s = 0, sa = 0; for (let i = 0; i < a.length; i++) { s += a[i]; sa += Math.abs(a[i]) * ((i % 7) + 1); }
  return { type: g.type, part: g.userData.propPart ? 1 : 0, params: Object.keys(p).sort().map((k) => [k, typeof p[k] === 'boolean' ? (p[k] ? 1 : 0) : p[k]]), n: g.attributes.position.count, idx: g.index ? g.index.count : 0, s, sa };
}
function tree(o) {
  const n = { k: o.isMesh ? 'mesh' : o.isPointLight ? 'plight' : o.isGroup ? 'group' : o.type, p: v3(o.position), r: [o.rotation.x, o.rotation.y, o.rotation.z, o.rotation.order], s: v3(o.scale), v: o.visible ? 1 : 0 };
  if (o.userData.__builder) { n.k = 'builder'; n.b = o.userData.__builder; return n; }
  if (o.isMesh) { n.g = geoInfo(o.geometry); n.m = matInfo(o.material); }
  if (o.isLight) { n.l = [o.color.getHex(), o.intensity, o.distance ?? 0, o.decay ?? 0]; }
  if (o.children.length) n.c = o.children.map(tree);
  return n;
}
function dumpState(env, tailWorld, handoff, camera) {
  void camera;
  const r = {};
  r.tree = tailWorld.map(tree);
  r.interactables = env.interactables.slice(handoff.interactables).filter((i) => !i.__side).map((i) => ({ p: v3(i.pos), r: i.r, prompt: i.prompt, en: i.enabled() ? 1 : 0 }));
  r.animators = env.animators.slice(handoff.animators).filter((a) => !a.__side).length;
  const rec = {};
  rec.switches = env.lightSwitches.slice(handoff.lightSwitches).map((s) => ({ x: s.wx, z: s.wz, on: s.on ? 1 : 0, faulty: s.faulty ? 1 : 0, panels: s.panelIndices, label: s.label, rocker: s.rocker.rotation.x, mc: s.motionCooldown }));
  rec.papers = env.papers.slice(handoff.papers).map((p) => ({ p: v3(p.g.position), taken: p.taken ? 1 : 0 }));
  rec.notes = env.loreNotes.slice(handoff.loreNotes).map((n) => ({ p: v3(n.g.position), id: n.id, taken: n.taken ? 1 : 0 }));
  rec.pool = env.poolBounds ? [env.poolBounds.minX, env.poolBounds.maxX, env.poolBounds.minZ, env.poolBounds.maxZ, env.poolBounds.waterY, ...env.poolBounds.tiles] : null;
  rec.poolPlanLeft = env.window.__poolPlan ? 1 : 0;
  const vb = env.vacuumBot;
  rec.vacuum = vb ? { p: v3(vb.g.position), yaw: vb.yaw, speed: vb.speed, pick: vb.pickTimer, spin: vb.spinPhase, chase: vb.chaseT, follow: vb.followT, stun: vb.stunT } : null;
  rec.safe = env.safeRoom ? [env.safeRoom.x, env.safeRoom.z, env.safeRoom.r, env.safeRoom.discovered ? 1 : 0] : null;
  const be = env.brokenElevator;
  rec.broken = be ? { world: v3(be.world), normal: v3(be.normal), next: be.nextScrape, peered: be.peered ? 1 : 0, li: be.light.intensity } : null;
  rec.holes = env.floorHoles.slice(handoff.floorHoles).map((h) => [h.x, h.z, h.r]);
  rec.barricades = env.barricadeTiles.slice();
  rec.candles = env.hauntedCandles.slice(handoff.hauntedCandles).map((c) => ({ out: c.extinguished ? 1 : 0, fadeT: c.fadeT, hint: c.hintCooldown, near: c.wasNear ? 1 : 0, streak: c.fizzleStreak, wick: c.wickGlowT }));
  rec.pickups = [...env.almonds.map((a) => [...v3(a.g.position), a.g.rotation.y, a.g.rotation.z]), ...env.batteryItems.map((a) => [...v3(a.g.position), a.g.rotation.y, a.g.rotation.z])];
  r.records = rec;
  r.candlePos = env.hauntedCandles.length ? env.interactables.filter((i) => i.prompt === '[E] BLOW OUT').map((i) => v3(i.pos)) : [];
  r.panelsOff = env.panels.map((p) => (p.switchedOff ? 1 : 0));
  return r;
}
// the run-time view of an object (what animators change), 12 significant digits
const rp = (x) => (typeof x === 'number' ? +x.toPrecision(12) : x);
function rtree(o) {
  const n = [o.isMesh ? 'mesh' : o.isPointLight ? 'plight' : o.userData.__builder ? 'builder' : o.isGroup ? 'group' : o.type,
    [...v3(o.position), o.rotation.x, o.rotation.y, o.rotation.z, ...v3(o.scale)].map(rp), o.visible ? 1 : 0,
    o.isMesh ? rp(o.material.opacity) : o.isLight ? rp(o.intensity) : 0,
    o.isMesh && o.material.emissive ? o.material.emissive.getHex() : 0,
    o.isMesh && o.geometry.attributes.position.count === 625 ? rp(o.geometry.attributes.position.array.reduce((a, b) => a + b, 0)) : 0];
  if (o.children.length && !o.userData.__builder) n.push(o.children.map(rtree));
  return n;
}
function dumpRuntime(env, handoff) {
  const w = env.world.children.slice(handoff.world).filter((o) => !o.userData.__side);
  return { tree: w.map(rtree), records: dumpState(env, w, handoff).records, interactables: env.interactables.map((i) => ({ p: v3(i.pos), en: i.enabled() ? 1 : 0 })) };
}

// ------------------------------------------------------------------ cases
const CASES = [];
const W8 = ['normal', 'wet', 'derelict', 'still', 'electric', 'haunted'];
let n = 0;
for (const weather of W8) {
  for (const [size, pool] of [[27, true], [35, false], [43, true]]) {
    n++;
    CASES.push({ name: `${weather}_${size}`, size, seed: 1000 + n * 7919, weather, level: 1 + (n % 9), scp: n % 4 === 1 ? 'store' : n % 4 === 2 ? 'stairwell' : null,
      loreFound: n % 3 === 0 ? [7, 3, 1, 2, 9, 14] : n % 3 === 1 ? [] : Array.from({ length: 36 }, (_, i) => i), pool, runtimeSeed: 77 + n,
      sim: { frames: 600, use: [40, 140, 141, 142, 300, 420, 421, 422, 423, 424, 500], flip: [60, 61, 250], snap: [1, 45, 90, 150, 299, 450, 599], stick: [180, 260], stickTo: 'candle', lowBattery: 200 } });
  }
}
// a tiny level where the candidates run out early
CASES.push({ name: 'haunted_small', size: 13, seed: 4242, weather: 'haunted', level: 3, scp: null, loreFound: [], pool: true, runtimeSeed: 5,
  sim: { frames: 300, use: [10, 11, 12, 13, 14, 15, 100, 101, 102, 103], flip: [5], snap: [1, 50, 299], stick: [20, 120], stickTo: 'candle', lowBattery: 60 } });
CASES.push({ name: 'electric_big', size: 51, seed: 99, weather: 'electric', level: 12, scp: 'loop', loreFound: [], pool: true, runtimeSeed: 1234,
  sim: { frames: 1500, use: [30, 700], flip: [10], snap: [1, 400, 800, 1499], stick: null, stickTo: null, lowBattery: 100 } });
// parked next to a sparking wire bundle long enough for flares (and a permanent scorch)
CASES.push({ name: 'electric_flares', size: 43, seed: 31337, weather: 'electric', level: 5, scp: null, loreFound: [], pool: false, runtimeSeed: 4,
  sim: { frames: 24000, use: [100], flip: [], snap: [1, 4000, 8000, 12000, 16000, 20000, 23999], stick: [0, 24000], stickTo: 'wire', lowBattery: 1000 } });
// candles blown out with the battery too low to relight: the hint, then relights
CASES.push({ name: 'haunted_hint', size: 43, seed: 2718, weather: 'haunted', level: 7, scp: 'house', loreFound: [3, 8], pool: true, runtimeSeed: 9,
  sim: { frames: 900, use: [10, 300, 301, 302, 303, 600, 601, 602], flip: [700], snap: [1, 11, 60, 299, 304, 599, 899], stick: [0, 900], stickTo: 'candle', lowBattery: 5 } });

const out = { cases: CASES.map(runCase) };
out.unknown = [...unknown].sort();
fs.writeFileSync(path.join(OUT, 'cp2_tail.json'), JSON.stringify(out));
console.log(`cp2_tail.json: ${out.cases.length} cases; unknown globals: ${out.unknown.join(' ')}`);
for (const c of out.cases) console.log(`  ${c.name}: tail draws ${c.tailDraws}, builders ${c.builders.length}, tree ${c.build.tree.length}, inter ${c.build.interactables.length}, anim ${c.build.animators}, sim msgs ${c.sim.msgs.length}, sim draws ${c.sim.draws}, final tree ${c.final.tree.length}`);
