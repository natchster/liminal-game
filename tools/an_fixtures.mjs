// Fixture generator for the Rust port of backrooms.js lines 1581–2530 (the
// anomalies, the Showroom, the human rig and its walk, the Borrowed House):
// game/src/anomalies*.rs.
//
// It runs the ORIGINAL source text of those lines (sliced out of
// backrooms.js) inside a sandbox whose globals are deterministic stand-ins:
// the vendored three.js r160, src/archgen.js / store.js / scp.js /
// creatures.js, a seeded mulberry32 Math.random with a draw counter, a
// canvasTex that runs its draw on a recording no-op context, god-mode
// onDeath, no audio. The helpers other modules own (bfs, los,
// circleHitsWall, navTileAt, the entity steering, skinBody) come in two
// variants:
//   stub — what the Rust stubs of world.rs / entities.rs do today
//   real — the original functions, sliced out of backrooms.js too
// and every scenario is dumped for both, so the Rust tests compare against
// whichever matches the state of those ports.
//
//   node tools/an_fixtures.mjs [/path/to/OpenCarole]
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const HERE = path.dirname(fileURLToPath(import.meta.url));
const ROOT = process.argv[2] || '/Users/nohwan/Documents/Personal_Projects/OpenCarole';
const OUT = path.join(HERE, '..', 'game', 'tests', 'fixtures');
fs.mkdirSync(OUT, { recursive: true });
const imp = (p) => import(pathToFileURL(path.join(ROOT, p)).href);

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
globalThis.__threeUuidRandom = mulberry32(4242);   // three's UUIDs never touch Math.random
const THREE = await imp('vendor/three.module.min.js');
const AG = await imp('src/archgen.js');
const ST = await imp('src/store.js');
const SCP = await imp('src/scp.js');
const CR = await imp('src/creatures.js');

// ------------------------------------------------------------ the source ---
const SRC = fs.readFileSync(path.join(ROOT, 'backrooms.js'), 'utf8').split('\n');
const L = (n) => SRC[n - 1];
if (!L(1581).includes('anomalies') || !L(2527).startsWith('function levelTitle') || L(2531) !== '}') throw new Error('backrooms.js moved: re-check the anomaly line range');
const ANOM = SRC.slice(1580, 2531).join('\n');
// a top-level declaration, by its first line, to the end of its braces
function block(head) {
  const i = SRC.findIndex((l) => l.startsWith(head));
  if (i < 0) throw new Error('not found: ' + head);
  const out = [];
  let depth = 0, opened = false;
  for (let j = i; j < SRC.length; j++) {
    const l = SRC[j];
    out.push(l);
    for (const ch of l) { if (ch === '{') { depth++; opened = true; } else if (ch === '}') depth--; }
    if (opened && depth === 0) break;
    if (!opened && /;\s*(\/\/.*)?$/.test(l)) break;
  }
  return out.join('\n');
}
const COMMON = [
  block('const entSmooth ='), block('const wrapPi ='), block('function joint('), block('const IK ='), block('function solveLimb('),
  block('function textTex('), block('function regionLightAt('),
].join('\n');
const REAL = [
  block('function bfs('), block('function los('), block('function circleHitsWall('), block('function navTileAt('), block('function collideMove('),
  block('const ENT_R ='), block('const ENT_MOTION ='), block('const STEER8 ='), block('function lineWalkable('), block('function entityTurn('),
  block('function entityAim('), block('function entityStepToward('), block('function entityDrive('), block('function skinBody('),
].join('\n');
// what world.rs / entities.rs's stubs do
const STUB = `
function bfs(sx, sz) { return new Int32Array(W * H).fill(-1); }
function los() { return true; }
function circleHitsWall() { return false; }
function navTileAt(px, pz) { return [tx(px), tz(pz)]; }
function collideMove(p, dx, dz) { p.x += dx; p.z += dz; }
function entityTurn() {}
function entityStepToward() { return false; }
function entityDrive() {}
function skinBody() { return null; }
`;
// the game globals the anomaly code calls, as the JS defines them (or as the
// harness needs them: no audio, god mode)
const GLUE = `
const rand = (a = 1, b) => (b === undefined ? Math.random() * a : a + Math.random() * (b - a));
const irand = (a, b) => Math.floor(rand(a, b + 1));
const clamp = (v, a, b) => (v < a ? a : v > b ? b : v);
const lerp = (a, b, t) => a + (b - a) * t;
const tIdx = (x, z) => z * W + x;
const inGrid = (x, z) => x >= 0 && z >= 0 && x < W && z < H;
const isWallTile = (x, z) => !inGrid(x, z) || tiles[tIdx(x, z)] === 1;
const wx = (t) => (t - W / 2 + 0.5) * CELL;
const wz = (t) => (t - H / 2 + 0.5) * CELL;
const tx = (w) => Math.floor(w / CELL + W / 2);
const tz = (w) => Math.floor(w / CELL + H / 2);
const toU = (w) => w / CELL + W / 2;
const toV = (w) => w / CELL + H / 2;
function blip(freq, dur, type = 'sine', gain = 0.2, slideTo = null, delay = 0, pan = 0) { if (!AC) return; }
function noiseBurst(dur, freq, q, gain, type = 'bandpass', slideTo = null, delay = 0, pan = 0) { if (!AC) return; }
const sfx = {
  door: () => { blip(523, 0.5, 'sine', 0.16); blip(784, 0.8, 'sine', 0.13, null, 0.14); },
  clank: () => noiseBurst(0.12, rand(1800, 3200), 6, 0.10),
};
function showMsg(text, dur = 2) { msgLog.push([text, dur]); }
function onDeath(cause = 'entity') {
  if (state !== 'playing') return;
  if (godMode) { godDeaths++; godCauses[cause] = (godCauses[cause] || 0) + 1; player.sanity = Math.max(player.sanity, 40); return; }
  state = 'dead';
}
function withSkinDetail(mat) { return mat; }
`;

// ------------------------------------------------------------ the sandbox ---
const CELL = 4, WALL_H = 3.1, EYE = 1.62;
function elem() { return { style: {}, textContent: '' }; }
function counted(rng) { const f = () => { f.n++; return rng(); }; f.n = 0; return f; }
const native = mulberry32(777);
const recCtx = new Proxy({}, {
  get(t, k) {
    if (k in t) return t[k];
    if (typeof k === 'string' && k.startsWith('create')) return () => ({ addColorStop() {} });
    if (k === 'measureText') return () => ({ width: 0 });
    return () => {};
  },
  set(t, k, v) { t[k] = v; return true; },
});
function makeSandbox({ variant = 'stub', hash = '', layout = null, levelNum = 3, runSeed = 12345 } = {}) {
  const C = {
    THREE, CELL, WALL_H, EYE, KILL_DIST: 1.05, SAN_MAX: 100, RF_DARK: 1,
    W: layout ? layout.W : 21, H: layout ? layout.H : 21, tiles: layout ? layout.tiles : new Uint8Array(21 * 21), layout, solids: null, barricadeTiles: [],
    exitData: null, world: new THREE.Group(), scene: new THREE.Scene(), interactables: [], entities: [], panels: [],
    player: { pos: new THREE.Vector3(0, EYE, 0), sanity: 100, crouch: false, flashlightOn: false, audibleRange: 0, hideEye: null },
    levelNum, runSeed, state: 'playing', godMode: true, godDeaths: 0, godCauses: {}, msgLog: [], shakeT: 0, playerField: null,
    AC: null, soundscape: null, master: null, regionInfo: null, regionMap: null,
    scpForLevel: SCP.scpForLevel, storeKit: ST.storeKit, storePhase: ST.storePhase, storeClock: ST.storeClock, productName: ST.productName,
    ROOM_SETS: ST.ROOM_SETS, ROOM_SET_WEIGHTS: ST.ROOM_SET_WEIGHTS, DEPARTMENT_SIGNS: ST.DEPARTMENT_SIGNS, storeCells: AG.storeCells, ARCHETYPES: AG.ARCHETYPES,
    creatureGeometry: CR.creatureGeometry,
    PG: (name, g) => g,
    canvasTex(w, h, draw, repeat = true) {
      const seeded = Math.random; Math.random = native;
      try { draw(recCtx, w, h); } finally { Math.random = seeded; }
      const t = new THREE.Texture(); t.userData.size = [w, h, repeat];
      return t;
    },
    pickText: (k) => 'PICK:' + k,
    enterLoop() { C.modeLog.push('loop'); }, enterStairwell() { C.modeLog.push('stairs'); }, nextLevel() { C.modeLog.push('next'); }, modeLog: [],
    document: { getElementById: (id) => C.els[id] || (C.els[id] = elem()) }, els: {},
    location: { hash },
    timers: [], setTimeout(f, ms) { C.timers.push([f, ms]); return C.timers.length; }, clearTimeout() {},
    crouchIndEl: Object.assign(elem(), { textContent: '— CROUCHING —' }), flashEl: elem(),
    console,
  };
  const exportsList = ['scpOnLevel', 'showOmen', 'storeState', 'storeReset', 'placeStore', 'drawStoreBoard', 'storeMusic', 'storeAnnounce', 'onStorePhase', 'updateStore',
    'HUMAN_RIGS', 'humanFingerPts', 'humanRig', 'makeStaff', 'spawnStaff', 'updateStaff', 'gaitNoise', 'humanGait',
    'houseState', 'houseReset', 'houseExitBlocked', 'SHADOW_U', 'shadowSmoke', 'shadowMat', 'shadowEdgeMat', 'placeHouse', 'putOutNestLamp', 'houseLetGo',
    'makeShadow', 'spawnShadows', 'playerLitForShadows', 'updateShadow', 'shadowFade', 'nearestShadow', 'updateHouse', 'useExitDoor', 'updateScp', 'levelTitle',
    'bfs', 'navTileAt'];
  const code = `with (C) {\n${GLUE}\n${COMMON}\n${variant === 'real' ? REAL : STUB}\n${ANOM}\nreturn { ${exportsList.join(', ')}, get levelScp() { return levelScp; }, set levelScp(v) { levelScp = v; } };\n}`;
  const before = new Set(Object.keys(globalThis));
  const api = new Function('C', code)(C);
  for (const k of Object.keys(globalThis)) if (!before.has(k)) throw new Error('the sandbox leaked a global: ' + k);
  return { C, api };
}

// ------------------------------------------------------------- the dumps ---
const v3 = (v) => [v.x, v.y, v.z];
const eul = (e) => [e.x, e.y, e.z];
function kindOf(o) {
  if (o.isInstancedMesh) return 'inst';
  if (o.isSkinnedMesh) return 'skinned';
  if (o.isMesh) return 'mesh';
  if (o.isBone) return 'bone';
  if (o.isPointLight) return 'pointLight';
  if (o.isGroup) return 'group';
  return 'object3d';
}
function geoParams(g) {
  const p = g.parameters || {};
  switch (g.type) {
    case 'BoxGeometry': return ['box', p.width, p.height, p.depth];
    case 'CylinderGeometry': return ['cyl', p.radiusTop, p.radiusBottom, p.height, p.radialSegments, p.heightSegments, p.openEnded, p.thetaStart, p.thetaLength];
    case 'PlaneGeometry': return ['plane', p.width, p.height];
    case 'CapsuleGeometry': return ['capsule', p.radius, p.length, p.capSegments, p.radialSegments];
    default: return [g.type];
  }
}
// a tree, materials and geometries numbered in order of first appearance
function dumper() {
  const mats = [], matIds = new Map(), geoIds = new Map();
  let geoN = 0;
  function matIndex(m) {
    if (!matIds.has(m)) {
      matIds.set(m, mats.length);
      mats.push({
        type: m.type, color: m.color ? m.color.getHex() : null, emissive: m.emissive ? m.emissive.getHex() : null, ei: m.emissiveIntensity ?? null,
        map: m.map ? (m.map.userData.size ? m.map.userData.size.slice(0, 2) : [0, 0]) : null,
        tr: !!m.transparent, op: m.opacity, dw: !!m.depthWrite, side: m.side, po: !!m.polygonOffset, pof: m.polygonOffsetFactor, vc: !!m.vertexColors,
        spec: m.specular ? m.specular.getHex() : null, shin: m.shininess ?? null,
      });
    }
    return matIds.get(m);
  }
  function tree(o) {
    // defaults left out: name '', scale 1, visible, renderOrder 0, no userData, no children
    const n = { k: kindOf(o), p: v3(o.position), r: eul(o.rotation) };
    if (o.name) n.n = o.name;
    if (o.scale.x !== 1 || o.scale.y !== 1 || o.scale.z !== 1) n.s = v3(o.scale);
    if (!o.visible) n.v = false;
    if (o.renderOrder) n.ro = o.renderOrder;
    const ud = {};
    if (o.userData.hide) ud.hide = [o.userData.hide.eye, o.userData.hide.r, o.userData.hide.kind, o.userData.hide.front ?? null];
    if (o.userData.gameplay) ud.gameplay = true;
    if (o.userData.arch) ud.arch = true;
    if (o.userData.main) ud.main = o.userData.main;
    if (o.userData.tail) ud.tail = o.userData.tail;
    if (Object.keys(ud).length) n.ud = ud;
    if (o.isMesh) {
      if (!geoIds.has(o.geometry)) geoIds.set(o.geometry, geoN++);
      n.gid = geoIds.get(o.geometry);
      n.g = geoParams(o.geometry);
      if (n.g[0] === 'capsule' || o.isSkinnedMesh) {   // translated capsules, the creature bodies
        o.geometry.computeBoundingBox();
        const b = o.geometry.boundingBox;
        n.bb = [b.min.x, b.min.y, b.min.z, b.max.x, b.max.y, b.max.z];
      }
      n.m = matIndex(o.material);
    }
    if (o.isInstancedMesh) {
      n.count = o.count; n.im = [];
      for (let i = 0; i < o.count; i++) n.im.push(Array.from(o.instanceMatrix.array.slice(i * 16, i * 16 + 16)));
    }
    if (o.isSkinnedMesh) n.bones = o.skeleton ? o.skeleton.bones.map((b) => b.name) : null;
    if (o.isPointLight) n.light = [o.color.getHex(), o.intensity, o.distance, o.decay];
    if (o.children.length) n.c = o.children.map(tree);
    return n;
  }
  return { tree, mats, matIndex };
}
function bones(root) {   // every bone of a rig in DFS order: [x, y, z, rx, ry, rz]
  const out = [];
  root.traverse((o) => { if (o.isBone) out.push([...v3(o.position), ...eul(o.rotation)]); });
  return out;
}
const GAIT_KEYS = ['gSeed', 'gX', 'gZ', 'gYaw', 'gSpd', 'gTurn', 'gPh', 'gHip', 'gLook', 'gLookP', 'headYaw'];
const LEG_KEYS = ['fz', 'ay', 'az', 'ph', 'w', 'y0', 'z0', 'p0', 'ty', 'tz', 'tph', 'ttoe'];
function gaitState(e) {
  const o = {};
  for (const k of GAIT_KEYS) o[k] = e[k] ?? null;
  o.legs = e.parts.legs.map((Lg) => { const l = { down: !!Lg.down, sw: !!Lg.sw }; for (const k of LEG_KEYS) l[k] = Lg[k] ?? null; return l; });
  return o;
}
// one entity in one frame
const STATES = ['wander', 'hunt'];
function entFrame(e) {
  return [e.type, e.pos.x, e.pos.z, e.yaw, e.state, e.loseT, e.wTimer, e.stepAcc, e.lookK ?? null, e.huntK ?? null, e.solid ?? null, e.agit ?? null, e.touchCD ?? null,
    e.dying ?? 0, e.g.scale.x, e.wTarget, !!e.wField, e.parts.edge ? e.parts.edge.visible : null, !!e.g.parent];
}
function entityRec(e) {
  const keys = ['lookK', 'huntK', 'solid', 'agit', 'phase', 'touchCD'];
  const num = {};
  for (const k of keys) if (e[k] !== undefined) num[k] = e[k];
  return {
    type: e.type, x: e.pos.x, z: e.pos.z, yaw: e.yaw, state: e.state, speed: e.speed, faceY: e.faceY, eyeYOffset: e.eyeYOffset,
    wTimer: e.wTimer, loseT: e.loseT, stepAcc: e.stepAcc, freeze: e.freeze, dying: e.dying ?? 0, wTarget: e.wTarget, hasField: !!e.wField,
    vx: e.vx ?? 0, vz: e.vz ?? 0, num, scale: e.g.scale.x, inWorld: !!e.g.parent, faceInScene: !!e.face.parent, edgeVisible: e.parts.edge ? e.parts.edge.visible : null,
  };
}

// ------------------------------------------------------------- levels ---
function genLevel(seed, size, level, scp) {
  const layout = AG.generateLayout({ size, level, rng: mulberry32(seed), cell: CELL, scp });
  return layout;
}
// the spawn (first open tile from the top-left) and its BFS distances
function spawnAndDist(layout) {
  const { W, H, tiles } = layout;
  let sx = -1, sz = -1;
  for (let z = 1; z < H - 1 && sx < 0; z++) for (let x = 1; x < W - 1; x++) if (tiles[z * W + x] === 0) { sx = x; sz = z; break; }
  const dist = new Int32Array(W * H).fill(-1);
  const q = [sz * W + sx]; dist[q[0]] = 0;
  for (let qi = 0; qi < q.length; qi++) {
    const cur = q[qi], cx = cur % W, cz = (cur / W) | 0;
    for (const [dx, dz] of [[1, 0], [-1, 0], [0, 1], [0, -1]]) {
      const nx = cx + dx, nz = cz + dz;
      if (nx < 0 || nz < 0 || nx >= W || nz >= H) continue;
      const n = nz * W + nx;
      if (tiles[n] === 0 && dist[n] === -1) { dist[n] = dist[cur] + 1; q.push(n); }
    }
  }
  return { sx, sz, dist };
}
// the exit: the reachable open tile farthest from the spawn (ties: last in scan order)
function exitTile(layout, dist) {
  let best = -1, bi = -1;
  for (let i = 0; i < dist.length; i++) if (dist[i] >= best && layout.tiles[i] === 0) { best = dist[i]; bi = i; }
  return [bi % layout.W, (bi / layout.W) | 0];
}
function layoutRec(layout) { return { W: layout.W, H: layout.H, dominant: layout.dominant, tiles: Array.from(layout.tiles), zoneOf: Array.from(layout.zoneOf), zones: layout.zones.map((z) => z.arch) }; }

// ================================================================= scp ===
function scpFixture() {
  const cases = [];
  for (const [hash, runSeed] of [['', 12345], ['', 987654321], ['#scp=store@3,house@5', 12345], ['#scp=none@4', 12345], ['#nosound&scp=loop@2,stairwell@3&level=4', 777],
    ['#scp=store%403%2Chouse%407', 31337], ['#scp=house@0,store@-1,loop@2.5,stairwell@0x3', 4242], ['#scp=store@3,store@6,house@3', 1], ['#scp=', 99], ['#xscp=store@3', 99], ['#scp=@3,foo@4', 5],
    ['#scp=store@ 3 ,house@1e1', 2024], ['#scp=none@2,none@3,none@4,none@5,none@6', 8080]]) {
    const { api } = makeSandbox({ hash, runSeed });
    const levels = [];
    for (let n = 1; n <= 14; n++) levels.push(api.scpOnLevel(n));
    cases.push({ hash, runSeed, levels });
  }
  return cases;
}

// ============================================================ the rig ===
function rigFixture() {
  const { api } = makeSandbox({});
  const out = { fingers: {}, rigs: [] };
  for (const name of ['staff', 'shadow']) {
    out.fingers[name] = [1, -1].map((s) => api.humanFingerPts(api.HUMAN_RIGS[name], s));
    for (const rigid of [false, true]) {
      const D = dumper();
      const g = new THREE.Group();
      const mat = rigid ? new THREE.MeshLambertMaterial({ color: 0xc9a23a }) : null;
      const P = api.humanRig(g, api.HUMAN_RIGS[name], mat);
      out.rigs.push({ name, rigid, tree: D.tree(g), mats: D.mats, legs: P.legs.map((l) => [l.hip.name, l.knee.name, l.ankle.name, l.toe.name, l.side]),
        arms: P.arms.map((a) => [a.clav.name, a.shoulder.name, a.elbow.name, a.wrist.name, a.fingers.map((f) => f.map((b) => b.name)), a.side]) });
    }
  }
  out.gaitNoise = []; for (let i = -40; i <= 40; i++) out.gaitNoise.push([i * 0.37, api.gaitNoise(i * 0.37)]);
  out.shadowMats = [api.shadowMat, api.shadowEdgeMat].map((m) => ({ type: m.type, color: m.color.getHex(), tr: m.transparent, op: m.opacity, dw: m.depthWrite, side: m.side, key: m.customProgramCacheKey() }));
  return out;
}

// the scripted walk for the gait test: speed stops, ambles, strides, runs,
// turns on the spot and is relocated once; the style eases through every factor
function gaitScript(n) {
  const fr = [];
  let x = 0, z = 0, yaw = 0, t = 0;
  for (let k = 0; k < n; k++) {
    const dt = k % 97 === 50 ? 0 : (k % 7 === 3 ? 1 / 30 : 1 / 60);
    t += dt;
    const ph = k / n;
    const spd = ph < 0.08 ? 0 : ph < 0.25 ? 0.9 : ph < 0.4 ? 1.8 : ph < 0.55 ? 3.6 : ph < 0.62 ? 0 : ph < 0.8 ? 5.5 : 0.4;
    const turnRate = ph > 0.55 && ph < 0.62 ? 2.2 : Math.sin(k * 0.05) * 0.6;
    yaw += turnRate * dt;
    x += Math.sin(yaw) * spd * dt; z += Math.cos(yaw) * spd * dt;
    if (k === Math.floor(n * 0.7)) { x += 3; z -= 2; }    // a relocation, not a stride
    const st = {
      hunt: Math.max(0, Math.sin(k * 0.013)), glide: Math.max(0, Math.cos(k * 0.009)) * 0.8, reach: Math.max(0, Math.sin(k * 0.021 + 1)) * 0.9,
      look: 0.5 + 0.5 * Math.sin(k * 0.017), shiver: Math.max(0, Math.sin(k * 0.031 + 2)), tilt: Math.sin(k * 0.05) * 0.2, breath: 0.5 + 0.5 * Math.cos(k * 0.011),
    };
    fr.push({ dt, t, x, z, yaw, st, px: 2 + Math.sin(k * 0.02) * 3, py: EYE - (k % 200 < 50 ? 0.6 : 0), pz: 3 + Math.cos(k * 0.015) * 2 });
  }
  return fr;
}
function gaitFixture() {
  const out = [];
  for (const name of ['staff', 'shadow']) {
    for (const variant of ['real']) {   // the walk itself calls nothing the variants change
      const { C, api } = makeSandbox({ variant });
      Math.random = counted(mulberry32(name === 'staff' ? 101 : 202));
      const b = name === 'staff' ? api.makeStaff(10, 10) : api.makeShadow(10, 10);
      const e = { type: name, g: b.g, pos: b.g.position, parts: b.parts, yaw: 0 };
      const R = api.HUMAN_RIGS[name];
      const fr = gaitScript(600);
      const snaps = [];
      fr.forEach((f, k) => {
        e.pos.x = f.x; e.pos.z = f.z; e.yaw = f.yaw;
        C.player.pos.set(f.px, f.py, f.pz);
        api.humanGait(e, f.dt, f.t, R, f.st);
        if (k % 6 === 0 || k === fr.length - 1) snaps.push({ k, bones: bones(b.g), gait: gaitState(e) });
      });
      out.push({ name, variant, script: fr, draws: Math.random.n, snaps, hasSkin: !!b.parts.skin });
    }
  }
  return out;
}

// ========================================================== the Showroom ===
function storeCase(seed, size, level, variant, sim) {
  const layout = genLevel(seed, size, level, 'store');
  const { sx, sz, dist } = spawnAndDist(layout);
  const [ex, ez] = exitTile(layout, dist);
  const { C, api } = makeSandbox({ variant, layout, levelNum: level });
  api.levelScp = 'store';
  C.exitData = { pos: new THREE.Vector3((ex - layout.W / 2 + 0.5) * CELL, 0, (ez - layout.H / 2 + 0.5) * CELL) };
  for (let i = 0; i < 24; i++) C.panels.push({ flicker: 0, switchedOff: false });
  const rec = { seed, size, level, variant, layout: layoutRec(layout), spawn: [sx, sz], exit: [ex, ez] };
  // the build: placeStore then spawnStaff, under the level's seed
  Math.random = counted(mulberry32(seed * 7 + 1));
  api.storeReset();
  api.placeStore(Array.from(dist));
  rec.placeDraws = Math.random.n;
  const D = dumper();
  rec.world = C.world.children.map(D.tree);
  if (!sim) rec.world = rec.world.map((n) => ({ k: n.k, p: n.p, r: n.r, nc: (n.c || []).length }));   // the big level: the layout of it only
  if (variant === 'real') rec.world = rec.world.map((n, i) => (n.k === 'inst' ? Object.assign(n, { i }) : null)).filter(Boolean);
  rec.store = { on: api.storeState.on, boardText: api.storeState.boardText, hides: api.storeState.hides.map((h) => [h.kind, h.obj.position.x, h.obj.position.z, h.obj.rotation.y, h.eye, h.r, h.front ?? null]) };
  const c0 = C.world.children.length;
  api.spawnStaff(4 + Math.min(2, Math.floor(level / 5)), Array.from(dist));
  rec.spawnDraws = Math.random.n;
  rec.staffTrees = C.entities.map((e) => D.tree(e.g));
  rec.mats = D.mats;
  rec.entities = C.entities.map(entityRec);
  rec.worldAfterSpawn = C.world.children.length - c0;
  rec.sceneFaces = C.scene.children.length;
  if (!sim) return rec;
  // a run through day, closing, night and the next morning
  Math.random = counted(mulberry32(seed * 13 + 5));
  const hides = api.storeState.hides;
  const frames = [];
  const NF = 760;
  let fieldT = 0, t = 0;
  for (let k = 0; k < NF; k++) {
    const dt = k % 3 === 0 ? 0.31 : 0.17;   // uneven steps: no easing saturates every frame
    // the player: wandering the open floor, then by a staff member, then in a hiding place
    const phase = k % 120;
    let px, pz, crouch = false, flash = (k % 300) > 150, aud = (k % 50) < 10 ? 13 : 2;
    if (phase < 40) { const t = ((k * 7) % (layout.W * layout.H)); let i = t; while (layout.tiles[i] !== 0) i = (i + 1) % layout.tiles.length; px = (i % layout.W - layout.W / 2 + 0.5) * CELL; pz = (((i / layout.W) | 0) - layout.H / 2 + 0.5) * CELL; }
    else if (phase < 80) { const e = C.entities[k % C.entities.length]; px = e.pos.x + 0.6 + (k % 5) * 0.4; pz = e.pos.z - 0.3; }
    else { const h = hides[k % Math.max(1, hides.length)]; if (h) { const c = Math.cos(h.obj.rotation.y), s = Math.sin(h.obj.rotation.y); const lz = (h.kind === 'bed' ? [0.3, 0.95, -1.05, 0.5] : [0.1, 0.3, -0.35, 0.42])[k % 4], lx = (h.kind === 'bed' ? [0, 0.7, 0.2, 0.8] : [0, 0.5, 0.1, 0.6])[(k >> 2) % 4];
        px = h.obj.position.x + lx * c + lz * s; pz = h.obj.position.z - lx * s + lz * c; crouch = phase % 3 !== 0; } else { px = 0; pz = 0; } }
    C.player.pos.set(px, EYE, pz); C.player.crouch = crouch; C.player.flashlightOn = flash; C.player.audibleRange = aud;
    fieldT -= dt;
    if (fieldT <= 0) { fieldT = 0.35; const [ptx, ptz] = api.navTileAt(px, pz); C.playerField = api.bfs(ptx, ptz); }
    t += dt;
    for (const e of C.entities) {
      e.g.rotation.y = e.yaw;
      const d = Math.hypot(e.pos.x - px, e.pos.z - pz);
      if (e.freeze > 0) { e.freeze -= dt; continue; }
      if (e.type === 'staff') { if (api.updateStaff(e, dt, t, d)) break; }
    }
    api.updateScp(dt);
    if (k % 4 !== 3 && k !== NF - 1) { frames.push({ k, input: [px, pz, crouch, flash, aud, dt], draws: Math.random.n }); continue; }
    const f = { k, input: [px, pz, crouch, flash, aud, dt], draws: Math.random.n,
      store: [api.storeState.t, api.storeState.phase, api.storeState.cycle, api.storeState.lightsOut, api.storeState.hidden, api.storeState.boardText, C.player.hideEye, api.storeState.panelsOff,
        C.shakeT, C.player.sanity, C.godDeaths, C.crouchIndEl.textContent, api.storeState.arrowMat ? api.storeState.arrowMat.color.r : null],
      ents: C.entities.map(entFrame) };
    if (k % 60 === 59 || k === NF - 1) { f.gait = C.entities.map(gaitState); f.bones = C.entities.map((e) => bones(e.g)); }
    frames.push(f);
  }
  rec.frames = frames;
  rec.panels = C.panels.map((p) => [p.flicker, p.switchedOff]);
  rec.msgs = C.msgLog;
  return rec;
}
function storeFixture() {
  const out = [];
  for (const [seed, size, level, sim] of [[11, 31, 3, true], [12, 41, 6, true], [13, 53, 9, false]]) for (const variant of ['stub', 'real']) out.push(storeCase(seed, size, level, variant, sim));
  return out;
}

// ===================================================== the Borrowed House ===
function houseCase(seed, size, level, variant) {
  const layout = genLevel(seed, size, level, 'house');
  const { sx, sz, dist } = spawnAndDist(layout);
  const [ex, ez] = exitTile(layout, dist);
  const { C, api } = makeSandbox({ variant, layout, levelNum: level });
  api.levelScp = 'house';
  C.exitData = { pos: new THREE.Vector3((ex - layout.W / 2 + 0.5) * CELL, 0, (ez - layout.H / 2 + 0.5) * CELL) };
  const rec = { seed, size, level, variant, layout: layoutRec(layout), spawn: [sx, sz], exit: [ex, ez], dist: Array.from(dist) };
  Math.random = counted(mulberry32(seed * 7 + 3));
  api.houseReset();
  api.placeHouse(Array.from(dist));
  rec.placeDraws = Math.random.n;
  const D = dumper();
  rec.world = C.world.children.map(D.tree);
  rec.house = { nest: [api.houseState.nest.x, api.houseState.nest.z], lamps: api.houseState.lamps.length };
  rec.interactables = C.interactables.map((it) => [it.pos.x, it.pos.y, it.pos.z, it.r, it.prompt]);
  const c0 = C.world.children.length;
  api.spawnShadows(3 + (level >= 8 ? 1 : 0), Array.from(dist));
  rec.spawnDraws = Math.random.n;
  rec.shadowTrees = C.entities.map((e) => D.tree(e.g));
  rec.mats = D.mats;
  rec.entities = C.entities.map(entityRec);
  rec.worldAfterSpawn = C.world.children.length - c0;
  // a run: in the dark and lit, by the nest, the lamps going out one by one
  Math.random = counted(mulberry32(seed * 13 + 9));
  const frames = [];
  let fieldT = 0, t = 0;
  const nx = api.houseState.nest.x, nz = api.houseState.nest.z;
  for (let k = 0; k < 900; k++) {
    const dt = k % 5 === 0 ? 0.05 : 1 / 30;
    let px, pz;
    const ph = k % 150;
    if (ph < 50) { const e = C.entities[(k / 150 | 0) % C.entities.length]; px = e.pos.x + 0.5 + (ph % 10) * 0.15; pz = e.pos.z + 0.4; }
    else if (ph < 100) { px = nx + Math.sin(k * 0.1) * 4; pz = nz + Math.cos(k * 0.1) * 4; }
    else { let i = (k * 11) % layout.tiles.length; while (layout.tiles[i] !== 0) i = (i + 1) % layout.tiles.length; px = (i % layout.W - layout.W / 2 + 0.5) * CELL; pz = (((i / layout.W) | 0) - layout.H / 2 + 0.5) * CELL; }
    const flash = (k % 200) >= 120;
    C.player.pos.set(px, EYE, pz); C.player.flashlightOn = flash;
    let usedLamp = -1;
    if (k >= 400 && k % 30 === 0) { const i = C.interactables.findIndex((x) => x.enabled()); if (i >= 0) { usedLamp = i; C.interactables[i].use(); } }
    fieldT -= dt;
    if (fieldT <= 0) { fieldT = 0.35; const [ptx, ptz] = api.navTileAt(px, pz); C.playerField = api.bfs(ptx, ptz); }
    t += dt;
    for (const e of C.entities) {
      e.g.rotation.y = e.yaw;
      const d = Math.hypot(e.pos.x - px, e.pos.z - pz);
      if ((e.type === 'staff' || e.type === 'shadow') && e.freeze > 0) { e.freeze -= dt; continue; }
      if (e.type === 'shadow' || e.type === 'gone') { if (e.type === 'shadow' && api.updateShadow(e, dt, t, d)) break; }
    }
    api.updateScp(dt);
    if (k % 3 !== 2 && k !== 899) { frames.push({ k, input: [px, pz, flash, dt], used: usedLamp, draws: Math.random.n }); continue; }
    const f = { k, input: [px, pz, flash, dt], used: usedLamp, draws: Math.random.n,
      house: [api.houseState.out, api.houseState.lamps.filter((l) => !l.out).length, api.houseState.nestLight.intensity, api.SHADOW_U.uTime.value, api.SHADOW_U.uSmoke.value,
        api.shadowMat.opacity, api.shadowEdgeMat.opacity, C.player.sanity, C.godDeaths, C.flashEl.style.opacity ?? null, api.playerLitForShadows()],
      ents: C.entities.map(entFrame) };
    if (k % 75 === 74 || k === 899) { f.gait = C.entities.filter((e) => e.type !== 'gone').map(gaitState); f.bones = C.entities.map((e) => bones(e.g)); }
    frames.push(f);
  }
  rec.frames = frames;
  rec.lampsEnd = api.houseState.lamps.map((l) => [l.out, l.shade.emissiveIntensity, l.shade.color.r, l.shade.color.g, l.shade.color.b]);
  rec.msgs = C.msgLog;
  rec.timers = C.timers.map((t) => t[1]);
  // the front door: blocked while the nest is lit, then the next level
  return rec;
}
function houseFixture() {
  const out = [];
  for (const [seed, size, level] of [[21, 31, 3], [22, 41, 8]]) for (const variant of ['stub', 'real']) out.push(houseCase(seed, size, level, variant));
  return out;
}

// ===================================================== door, title, misc ===
function miscFixture() {
  const out = {};
  // useExitDoor on each anomaly, toStairs or not; the house before and after the nest goes out
  out.door = [];
  for (const scp of [null, 'store', 'house', 'loop', 'stairwell']) {
    for (const toStairs of [false, true]) {
      for (const houseOn of [false, true]) {
        const { C, api } = makeSandbox({});
        Math.random = counted(mulberry32(5));
        api.levelScp = scp;
        if (houseOn) api.houseState.on = true;
        api.useExitDoor(toStairs);
        out.door.push({ scp, toStairs, houseOn, modes: C.modeLog, msgs: C.msgLog, draws: Math.random.n });
      }
    }
  }
  out.title = [];
  for (const dominant of ['lobby', 'pillars', 'office', 'maze', 'tunnels', 'hall', 'electrical', 'store', 'pool', 'house']) {
    const { C, api } = makeSandbox({ layout: { W: 21, H: 21, tiles: new Uint8Array(441), dominant } });
    out.title.push([dominant, api.levelTitle()]);
  }
  // showOmen: its two timers
  {
    const { C, api } = makeSandbox({});
    api.showOmen('THE LIGHTS ARE ON IN THE SHOP', 1.7, 7);
    out.omen = { opacity: C.els.omen.style.opacity, timers: C.timers.map((t) => t[1]) };
    C.timers[0][0](); out.omen.after1 = [C.els.omen.textContent, C.els.omen.style.opacity];
    C.timers[1][0](); out.omen.after2 = C.els.omen.style.opacity;
  }
  // onStorePhase on its own (panels, music off, staff back to wandering)
  {
    const { C, api } = makeSandbox({});
    Math.random = counted(mulberry32(77));
    for (let i = 0; i < 30; i++) C.panels.push({ flicker: 0, switchedOff: false });
    C.entities.push({ type: 'staff', state: 'hunt', loseT: 3 }, { type: 'smiler', state: 'hunt', loseT: 3 });
    const seq = [];
    for (const to of ['warn', 'night', 'day']) {
      api.onStorePhase('x', to);
      seq.push({ to, draws: Math.random.n, panels: C.panels.map((p) => [p.flicker, p.switchedOff]), panelsOff: api.storeState.panelsOff, shakeT: C.shakeT, msgs: C.msgLog.slice(), ents: C.entities.map((e) => [e.state, e.loseT]) });
    }
    out.phases = seq;
  }
  return out;
}

const fixture = { scp: scpFixture(), rig: rigFixture(), gait: gaitFixture(), store: storeFixture(), house: houseFixture(), misc: miscFixture() };
// floats to 12 significant digits (the tests compare at 1e-9)
const round = (k, v) => (typeof v === 'number' && !Number.isInteger(v) ? +v.toPrecision(12) : v);
fs.writeFileSync(path.join(OUT, 'anomalies.json'), JSON.stringify(fixture, round));
console.log('wrote', path.join(OUT, 'anomalies.json'), (fs.statSync(path.join(OUT, 'anomalies.json')).size / 1e6).toFixed(2), 'MB');
