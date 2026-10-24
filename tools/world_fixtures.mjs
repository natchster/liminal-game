#!/usr/bin/env node
// Fixture generator for the Rust port of backrooms.js' world sections
// (game/src/world*.rs): the maze, the grid queries, genLayoutLevel, the zone /
// ceiling / nav queries, buildRegions, wallAdjSpot and friends, and a whole
// buildWorld() run with stub placers (every themed() pass, settleProps,
// settlePlacement, the architecture meshes, layPanels, instancing).
//
// The JS functions are not copied: they are cut out of backrooms.js by name
// and evaluated in one sandbox, with the vendored three r160, src/archgen.js,
// src/placement.js and src/instancing.js, under a counted mulberry32.
//
//   node tools/world_fixtures.mjs [/path/to/OpenCarole]
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const HERE = path.dirname(fileURLToPath(import.meta.url));
const ROOT = process.argv[2] || '/Users/nohwan/Documents/Personal_Projects/OpenCarole';
const OUT = path.join(HERE, '..', 'game', 'tests', 'fixtures');
fs.mkdirSync(OUT, { recursive: true });
const imp = (p) => import(pathToFileURL(path.join(ROOT, p)).href);
// three's UUIDs draw from the native generator in the game (backrooms.js line 63)
const nativeRandom = Math.random;
globalThis.__threeUuidRandom = () => nativeRandom();
const THREE = await imp('vendor/three.module.min.js');
const AG = await imp('src/archgen.js');
const PL = await imp('src/placement.js');
const IN = await imp('src/instancing.js');
const SRC = fs.readFileSync(path.join(ROOT, 'backrooms.js'), 'utf8');
const LINES = SRC.split('\n');

// ------------------------------------------------------------ extraction ---
// A top-level declaration, cut at the line where its brackets close (strings
// and comments skipped; none of the cut code has a bracket in a regex).
function extract(name) {
  const re = new RegExp(`^(function ${name}\\(|const ${name}\\b|let ${name}\\b)`);
  const i0 = LINES.findIndex((l) => re.test(l));
  if (i0 < 0) throw new Error('no declaration of ' + name);
  let depth = 0, seen = false, out = [];
  for (let i = i0; i < LINES.length; i++) {
    const l = LINES[i];
    out.push(l);
    let q = null;
    for (let k = 0; k < l.length; k++) {
      const c = l[k];
      if (q) { if (c === '\\') k++; else if (c === q) q = null; continue; }
      if (c === '/' && l[k + 1] === '/') break;
      if (c === "'" || c === '"' || c === '`') { q = c; continue; }
      if ('([{'.includes(c)) { depth++; seen = true; }
      else if (')]}'.includes(c)) depth--;
    }
    if (depth === 0 && (seen || /;\s*(\/\/.*)?$/.test(l))) {
      if (/^function /.test(l) || /[;}\]]\s*(\/\/.*)?$/.test(l) || !seen) return out.join('\n');
    }
  }
  throw new Error('unterminated ' + name);
}
const NAMES = [
  'rand', 'irand', 'choice', 'clamp',
  'tIdx', 'inGrid', 'isWallTile', 'wx', 'wz', 'tx', 'tz',
  'genMaze', 'bfs', 'los', 'circleHitsWall', 'altPathExists', 'toU', 'toV', 'genLayoutLevel',
  'zoneAtTile', 'zoneAtWorld', 'ceilAtWorld', 'navTileAt', 'collideMove',
  'biasedSpawnAngle', 'panForWorldPos',
  'RF_DARK', 'RF_MOSS', 'DECAY_BASE', 'DECAY_STEP', 'DECAY_MAX', 'levelDecay', 'GRIME_LIGHT', 'buildRegions', 'regionAt', 'regionFlagsAt',
  'wallAdjSpot', 'wallRegionIdx', 'grimeOf', 'tileFullySolid', 'wallMatKey', 'floorMatKey', 'ceilMatKey',
  '_m4a', '_yAxis', 'ARCH_CHUNK', '_chunkPos', 'addInstanced', 'buildFloors', 'buildCeilings', 'buildWalls',
  'FIXTURE_HANG', 'layPanels', 'freezeStatic', 'collectResources',
  'PROP_FAR', 'propCull', 'buildPropCull',
  'placementStats', 'gameplayObjects', 'settlePlacement', 'nearPool', 'settleStats', 'settleProps',
  'resetPropPositionArrays',
  'PLACER_THEMES', 'THEME_ALLOW', 'PLACER_ALWAYS', 'placerThemeCache', 'placerTheme', 'themeStats', 'placerRegistry', 'placerDist',
  'tagPlaced', 'themed', 'buildProfile', 'pendingDispose', 'buildWorld',
];
const pieces = NAMES.map(extract);
// every placer buildWorld runs through themed(), in order
const bwSrc = extract('buildWorld');
const THEMED = [...bwSrc.matchAll(/^\s+themed\((place[A-Za-z0-9_]+), dist\);/gm)].map((m) => m[1]);
const PLACERS = [...new Set(THEMED)];
// globals the cut code assigns without declaring here: declare them
const declared = new Set();
for (const p of pieces) { const m = p.match(/^(?:function|const|let) ([A-Za-z_$][\w$]*)/); if (m) declared.add(m[1]); for (const mm of p.matchAll(/^(?:const|let) .*$/gm)) for (const d of mm[0].matchAll(/(?:^(?:const|let) |, )([A-Za-z_$][\w$]*) =/g)) declared.add(d[1]); }
const assigned = new Set();
for (const src of [bwSrc, extract('resetPropPositionArrays')]) for (const m of src.matchAll(/(?:^|[;{]\s*)\s*([A-Za-z_$][\w$]*) = /gm)) assigned.add(m[1]);
const PROVIDED = new Set(['W', 'H', 'tiles', 'levelNum', 'layout', 'solids', 'tileCeil', 'regionMap', 'regionInfo', 'ceilRegion', 'barricadeTiles',
  'poolBounds', 'world', 'panels', 'panelMesh', 'interactables', 'animators', 'lightSwitches', 'almonds', 'batteryItems', 'papers', 'loreNotes',
  'hauntedCandles', 'safeRoom', 'brokenElevator', 'tvData', 'phoneData', 'clockData', 'manData', 'levelScp', 'spawnPos']);
const autoDecl = [...assigned].filter((n) => !declared.has(n) && !PROVIDED.has(n));

const PRELUDE = `
let W = 0, H = 0, tiles = null, levelNum = 1;
let layout = null, solids = null, tileCeil = null;
let regionMap = null, regionInfo = null, ceilRegion = null;
let barricadeTiles = [], poolBounds = null;
let world = null, panels = [], panelMesh = null, interactables = [], animators = [], lightSwitches = [];
let almonds = [], batteryItems = [], papers = [], loreNotes = [], hauntedCandles = [];
let safeRoom = null, brokenElevator = null, tvData = null, phoneData = null, clockData = null, manData = null;
let levelScp = null;
let spawnPos = new THREE.Vector3();
const player = { pos: new THREE.Vector3(), yaw: 0 };
const CELL = 4, WALL_H = 3.1, EYE = 1.62;
let wallpaperVariants = [];
const WALL_PALETTES = { length: 21 };
const PAL_MONO_YELLOW = 20;
const rateDeltaBuf = [], ambientLog = [], rainWindows = [];
const scene = new THREE.Scene();
const location = { hash: '' };
const window = {};
const lightTex = null;
const panelGeo = new THREE.BoxGeometry(1.9, 0.06, 1.25);
const ceilTileGeo = new THREE.PlaneGeometry(CELL, CELL); ceilTileGeo.rotateX(Math.PI / 2);
const floorTileGeo = new THREE.PlaneGeometry(CELL, CELL); floorTileGeo.rotateX(-Math.PI / 2);
const unitBoxGeo = new THREE.BoxGeometry(1, 1, 1);
const unitPlaneGeo = new THREE.PlaneGeometry(1, 1);
const named = (k) => { const m = new THREE.MeshBasicMaterial(); m.name = k; return m; };
const floorMats = [0, 1, 2, 3, 4].map((i) => named('carpet:' + i));
const ceilMats = [0, 1, 2, 3, 4].map((i) => named('tiles:' + i));
const metalMat = named('metal');
const archMats = new Map();
function archMat(key) { let m = archMats.get(key); if (!m) { m = named(key); archMats.set(key, m); } return m; }
const clearPropCache = () => {};
const randTrace = { on: true, n: 0, marks: [] };
const traceMark = (label) => { randTrace.marks.push([label, H$.draws()]); };
let placeProps = () => {}, placeStore = () => {}, placeHouse = () => {};
${autoDecl.map((n) => `let ${n} = null;`).join('\n')}
`;
const PLACER_DECL = PLACERS.map((n) => `const ${n} = H$.placer('${n}');`).join('\n');
const BODY = PRELUDE + '\n' + PLACER_DECL + '\n' + pieces.join('\n') + `
return {
  get: (k) => eval(k), set: (k, v) => eval(k + ' = v'),
  genMaze, bfs, los, circleHitsWall, altPathExists, genLayoutLevel, zoneAtTile, zoneAtWorld, ceilAtWorld, navTileAt, collideMove,
  biasedSpawnAngle, panForWorldPos, levelDecay, buildRegions, regionAt, regionFlagsAt, wallAdjSpot, buildWorld, settleProps,
  placerTheme, themed, toU, toV, wx, wz, tx, tz, tIdx, isWallTile, inGrid,
};`;
// the sandbox hooks: draw counter and placer factory (set per scenario)
const H$ = { draws: () => 0, placer: (n) => { const f = { [n]: function (dist) { return H$.run(n, dist); } }[n]; return f; }, run: () => {} };
const S = new Function('THREE', 'H$', 'generateLayout', 'buildSolids', 'circleHits', 'segmentClear', 'rayAxis', 'ceilingHeights', 'ARCH_DIRS',
  'resolvePlacement', 'instancifyStatic', BODY)(THREE, H$, AG.generateLayout, AG.buildSolids, AG.circleHits, AG.segmentClear, AG.rayAxis,
  AG.ceilingHeights, AG.DIRS, PL.resolvePlacement, IN.instancifyStatic);

// ------------------------------------------------------------ rng ---
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
let DRAWS = 0;
H$.draws = () => DRAWS;
function withSeed(seed, fn) {
  const prng = mulberry32(seed), orig = Math.random;
  Math.random = () => { DRAWS++; return prng(); };
  try { return fn(); } finally { Math.random = orig; }
}
const qrng = mulberry32(0xC0FFEE);   // query inputs (not the game's stream)
const qr = (a, b) => a + qrng() * (b - a);
const qi = (a, b) => Math.floor(qr(a, b + 1));
// wallpaper variant metadata: 21 palettes × 4 grime levels, then 2 moss
const WV = [];
for (let p = 0; p < 21; p++) for (let g = 0; g < 4; g++) WV.push({ grimeLvl: g, moss: false, paletteIdx: p });
WV.push({ grimeLvl: 3, moss: true, paletteIdx: 5 }, { grimeLvl: 3, moss: true, paletteIdx: 17 });
S.set('wallpaperVariants', WV);
const hashArr = (a) => { let h = 0, s = 0; for (let i = 0; i < a.length; i++) { s += a[i]; h = (Math.imul(h, 31) + (a[i] | 0) + 7) | 0; } return [a.length, s, h]; };

// ============================================================ A. maze ===
const maze = [];
for (const [size, seed] of [[21, 1], [22, 2], [31, 3], [41, 4], [53, 5]]) {
  const d0 = DRAWS;
  withSeed(seed, () => S.genMaze(size));
  maze.push({ size, seed, draws: DRAWS - d0, w: S.get('W'), tiles: Array.from(S.get('tiles')).join('') });
}

// ============================================================ B–D. one level at a time ===
function levelQueries(n, seed, scp) {
  S.set('levelNum', n); S.set('levelScp', scp);
  S.set('barricadeTiles', []);
  const size = Math.min(31 + n * 4, 53);
  const d0 = DRAWS;
  withSeed(seed, () => S.genLayoutLevel(n, size));
  const out = { n, seed, scp, size, layoutDraws: DRAWS - d0 };
  const W = S.get('W'), H = S.get('H'), tiles = S.get('tiles'), layout = S.get('layout');
  out.w = W; out.h = H; out.tiles = Array.from(tiles).join('');
  out.tileCeil = hashArr(Array.from(S.get('tileCeil')));
  out.solidsCount = S.get('solids').count;
  out.zones = layout.zones.length;
  const open = []; for (let i = 0; i < W * H; i++) if (tiles[i] === 0) open.push(i);
  const wallT = []; for (let i = 0; i < W * H; i++) if (tiles[i] === 1) wallT.push(i);
  // bfs from (1,1), an open tile and a wall tile
  const srcs = [[1, 1], [open[qi(0, open.length - 1)] % W, (open[qi(0, open.length - 1)] / W) | 0]];
  const wt = wallT[qi(0, wallT.length - 1)]; srcs.push([wt % W, (wt / W) | 0], [-1, 3]);
  out.bfs = srcs.map(([x, z]) => ({ x, z, d: Array.from(S.bfs(x, z)) }));
  const ext = W * 4 / 2;
  const pt = () => [qr(-ext - 3, ext + 3), qr(-ext - 3, ext + 3)];
  out.los = []; for (let i = 0; i < 160; i++) { const [ax, az] = pt(); const [bx, bz] = i % 3 ? [ax + qr(-14, 14), az + qr(-14, 14)] : pt(); out.los.push([ax, az, bx, bz, S.los(ax, az, bx, bz)]); }
  // circleHitsWall with and without barricades
  const circ = (q) => { const r = []; for (let i = 0; i < 160; i++) { const [px, pz] = pt(); const rr = qr(0.05, 1.2); r.push([px, pz, rr, S.circleHitsWall(px, pz, rr)]); } return r; };
  out.circle = circ();
  const bar = []; for (let i = 0; i < 12; i++) bar.push(open[qi(0, open.length - 1)]);
  S.set('barricadeTiles', bar); out.barricades = bar; out.circleBar = circ();
  // the no-solids fallbacks
  const sol = S.get('solids'); S.set('solids', null);
  out.circleNoSolids = circ();
  out.losNoSolids = []; for (let i = 0; i < 80; i++) { const [ax, az] = pt(); const [bx, bz] = [ax + qr(-14, 14), az + qr(-14, 14)]; out.losNoSolids.push([ax, az, bx, bz, S.los(ax, az, bx, bz)]); }
  S.set('solids', sol); S.set('barricadeTiles', []);
  // altPathExists
  out.alt = [];
  for (let i = 0; i < 40; i++) {
    const a = i % 9 === 0 ? wallT[qi(0, wallT.length - 1)] : open[qi(0, open.length - 1)], b = open[qi(0, open.length - 1)];
    const blk = []; const nb = qi(0, 30); for (let k = 0; k < nb; k++) blk.push(open[qi(0, open.length - 1)]);
    out.alt.push([a % W, (a / W) | 0, b % W, (b / W) | 0, blk, S.altPathExists(a % W, (a / W) | 0, b % W, (b / W) | 0, blk)]);
  }
  // collideMove
  out.collide = [];
  for (let i = 0; i < 160; i++) {
    const t = open[qi(0, open.length - 1)];
    const p = { x: S.wx(t % W) + qr(-2, 2), z: S.wz((t / W) | 0) + qr(-2, 2) };
    const dx = i % 7 === 0 ? 0 : qr(-0.6, 0.6), dz = i % 5 === 0 ? 0 : qr(-0.6, 0.6), r = qr(0.2, 0.5);
    const p0 = [p.x, p.z]; S.collideMove(p, dx, dz, r); out.collide.push([...p0, dx, dz, r, p.x, p.z]);
  }
  // zone / ceiling / nav
  out.zone = []; for (let i = 0; i < 80; i++) { const x = qi(-3, W + 2), z = qi(-3, H + 2); out.zone.push([x, z, layout.zones.indexOf(S.zoneAtTile(x, z))]); }
  out.zoneW = []; for (let i = 0; i < 60; i++) { const [px, pz] = pt(); out.zoneW.push([px, pz, layout.zones.indexOf(S.zoneAtWorld(px, pz)), S.ceilAtWorld(px, pz)]); }
  out.nav = []; for (let i = 0; i < 80; i++) { const t = i % 2 ? wallT[qi(0, wallT.length - 1)] : open[qi(0, open.length - 1)]; const px = S.wx(t % W) + qr(-2.2, 2.2), pz = S.wz((t / W) | 0) + qr(-2.2, 2.2); out.nav.push([px, pz, ...S.navTileAt(px, pz)]); }
  // regions (on the level's own stream after the layout, as initLevel → buildWorld)
  const d1 = DRAWS;
  withSeed(seed ^ 0x5bd1e995, () => S.buildRegions());
  out.regionDraws = DRAWS - d1;
  out.regionMap = Array.from(S.get('regionMap')).join(',');
  out.ceilRegion = Array.from(S.get('ceilRegion')).join(',');
  out.regionInfo = S.get('regionInfo').map((r) => [r.wall, r.ceil, r.floor, r.flags, r.grime, r.lit]);
  out.regionAt = []; for (let i = 0; i < 60; i++) { const x = qi(-2, W + 1), z = qi(-2, H + 1); const r = S.regionAt(x, z); out.regionAt.push([x, z, r.wall, r.flags, S.regionFlagsAt(x, z)]); }
  // wallAdjSpot, biasedSpawnAngle, panForWorldPos
  const d2 = DRAWS;
  out.spots = withSeed(seed + 77, () => {
    const r = [];
    for (let i = 0; i < 60; i++) { const t = i % 4 ? open[qi(0, open.length - 1)] : qi(0, W * H - 1); const s = S.wallAdjSpot(t % W, (t / W) | 0); r.push([t % W, (t / W) | 0, s ? [s.pos.x, s.pos.y, s.pos.z, s.normal.x, s.normal.y, s.normal.z, s.yaw, Object.is(s.normal.x, -0) ? 1 : 0, Object.is(s.normal.z, -0) ? 1 : 0] : null]); }
    return r;
  });
  out.spotDraws = DRAWS - d2;
  const player = S.get('player');
  out.angles = withSeed(seed + 99, () => { const r = []; for (let i = 0; i < 60; i++) { player.yaw = i % 10 === 0 ? NaN : qr(-7, 7); r.push([Number.isNaN(player.yaw) ? null : player.yaw, S.biasedSpawnAngle()]); } return r; });
  out.pans = []; for (let i = 0; i < 60; i++) { player.pos.set(qr(-20, 20), 0, qr(-20, 20)); player.yaw = qr(-7, 7); const [x, z] = i % 15 === 0 ? [player.pos.x, player.pos.z] : pt(); out.pans.push([player.pos.x, player.pos.z, player.yaw, x, z, S.panForWorldPos(x, z)]); }
  player.yaw = 0; player.pos.set(0, 0, 0);
  return out;
}
const levels = [];
for (const [n, seed, scp] of [[1, 101, null], [2, 202, null], [3, 303, null], [5, 505, null], [7, 707, null], [9, 909, null], [12, 1212, null], [4, 404, 'store'], [8, 808, 'house']]) levels.push(levelQueries(n, seed, scp));
const decay = []; for (let n = -2; n <= 15; n++) decay.push([n, S.levelDecay(n)]);
S.set('levelNum', 6); decay.push([null, S.levelDecay()]);
const themes = PLACERS.map((n) => [n, S.placerTheme(n)]);
fs.writeFileSync(path.join(OUT, 'world_queries.json'), JSON.stringify({ maze, levels, decay, themes }));
console.log('world_queries.json:', levels.length, 'levels');

// ============================================================ E. buildWorld with probe placers ===
// Every themed() placer is the same probe (the Rust test registers the same):
// it logs how many tiles are masked (2), then with one draw decides to add a
// box (floor / wall / ceiling / tall), a group of two boxes at the origin, a
// point light, or nothing; boxes sit on wallAdjSpot faces when there is one.
const PROBE_SIZE = [[0.4, 0.3, 0.4], [0.5, 0.4, 0.05], [0.3, 0.1, 0.3], [0.8, 2.3, 0.4]];
const PROBE_Y = [0.15, 1.5, 3.02, 1.15];
const PROBE_GEOS = PROBE_SIZE.map(([w, h, d]) => new THREE.BoxGeometry(w, h, d));
const PROBE_MATS = [0x884422, 0x226688, 0xcccccc, 0x553311].map((c) => new THREE.MeshLambertMaterial({ color: c }));
let probeLog = [];
H$.run = (name, dist) => {
  const tiles = S.get('tiles'), W = S.get('W');
  let masked = 0; const open = [];
  for (let i = 0; i < tiles.length; i++) { if (tiles[i] === 2) masked++; if (tiles[i] === 0) open.push(i); }
  probeLog.push(masked);
  const r = Math.random();
  if (!open.length || r >= 0.36) return;
  const world = S.get('world');
  const pick = () => { const t = open[Math.floor(Math.random() * open.length)]; return [t % W, (t / W) | 0]; };
  if (r < 0.3) {
    const [x, z] = pick();
    const kind = Math.floor(Math.random() * 4);
    const m = new THREE.Mesh(PROBE_GEOS[kind], PROBE_MATS[kind]);
    const spot = kind === 0 ? null : S.wallAdjSpot(x, z);
    if (spot) { m.position.copy(spot.pos).addScaledVector(spot.normal, PROBE_SIZE[kind][2] / 2); m.rotation.y = spot.yaw; }
    else { m.position.set(S.wx(x) + (Math.random() - 0.5) * 3, 0, S.wz(z) + (Math.random() - 0.5) * 3); m.rotation.y = Math.random() * 6; }
    m.position.y = PROBE_Y[kind];
    world.add(m);
    if (r < 0.04) S.get('animators').push(() => {});
  } else if (r < 0.34) {
    const g = new THREE.Group();
    for (let k = 0; k < 2; k++) {
      const [x, z] = pick();
      const m = new THREE.Mesh(PROBE_GEOS[0], PROBE_MATS[0]);
      m.position.set(S.wx(x) + (Math.random() - 0.5) * 3.6, 0.15, S.wz(z) + (Math.random() - 0.5) * 3.6);
      g.add(m);
    }
    world.add(g);
  } else {
    const [x, z] = pick();
    const l = new THREE.PointLight(0xffeecc, 1, 6, 2); l.position.set(S.wx(x), 2.2, S.wz(z)); world.add(l);
  }
};
function matName(m) { return m.name || (m.type + ':' + m.color.getHexString()); }
function geoDesc(g) { const p = g.parameters || {}; return g.type === 'BoxGeometry' ? ['box', p.width, p.height, p.depth] : g.type === 'PlaneGeometry' ? ['plane', p.width, p.height] : [g.type]; }
function childSum(o) {
  const n = { k: o.isInstancedMesh ? 'inst' : o.isMesh ? 'mesh' : o.isLight ? 'light' : o.isGroup ? 'group' : 'obj',
    placer: o.userData.placer || null, sp: !!o.userData.staticProp, arch: !!o.userData.arch, ins: !!o.userData.instanced, vis: o.visible, nc: o.children.length };
  if (o.isMesh) { n.mat = matName(o.material); n.geo = geoDesc(o.geometry); }
  if (o.isInstancedMesh) {
    n.count = o.count;
    const a = o.instanceMatrix.array; let s = 0, ws = 0;
    for (let i = 0; i < o.count * 16; i++) { s += a[i]; ws += a[i] * ((i % 7) + 1); }
    n.sum = s; n.wsum = ws; n.first = Array.from(a.slice(0, 16)); n.last = Array.from(a.slice((o.count - 1) * 16, o.count * 16));
    if (o.instanceColor) n.colors = o.count;
  } else {
    n.p = [o.position.x, o.position.y, o.position.z]; n.q = [o.quaternion.x, o.quaternion.y, o.quaternion.z, o.quaternion.w];
    n.c = o.children.map((c) => [c.position.x, c.position.y, c.position.z]);
  }
  return n;
}
const builds = [];
for (const [n, seed, scp] of [[1, 0x1001, null], [2, 0x2002, null], [3, 0x3003, null], [6, 0x6006, null], [10, 0xA00A, null], [12, 0xC00C, null], [4, 0x4004, 'store'], [8, 0x8008, 'house']]) {
  S.set('levelNum', n); S.set('levelScp', scp); S.set('world', null); S.get('window').__poolPlan = null;
  probeLog = [];
  const rt = S.get('randTrace'); rt.marks.length = 0;
  const d0 = DRAWS;
  const tm = S.get('traceMark'); withSeed(seed, () => { tm('start'); S.genLayoutLevel(n, Math.min(31 + n * 4, 53)); tm('layout'); S.buildWorld(); tm('world'); });
  const out = { n, seed, scp, draws: DRAWS - d0, marks: rt.marks.map(([l, k]) => [l, k - d0]) };
  out.tiles = Array.from(S.get('tiles')).join('');
  out.tileCeil = Array.from(S.get('tileCeil'));
  out.solidsCount = S.get('solids').count;
  out.regionInfo = S.get('regionInfo').map((r) => [r.wall, r.ceil, r.floor, r.flags, r.grime, r.lit]);
  out.panels = S.get('panels').map((p) => [p.wx, p.wz, p.bright, p.seed, p.rot, p.lfoFreq, p.lfoPhase, p.y, p.hBoost, p.ceil]);
  const pp = S.get('window').__poolPlan; out.poolPlan = pp ? [pp.x, pp.z, pp.pw, pp.ph] : null;
  const sp = S.get('spawnPos'); out.spawn = [sp.x, sp.y, sp.z];
  const ts = S.get('themeStats'); out.themeStats = [ts.ran, ts.skipped, Object.entries(ts.byTheme).map(([k, v]) => [k, v.ran, v.skipped])];
  out.settle = { ...S.get('settleStats') }; out.placement = { ...S.get('placementStats') };
  out.probeLog = probeLog;
  out.frozen = S.get('buildProfile').frozen; out.instancing = S.get('buildProfile').instancing;
  const world = S.get('world');
  out.children = world.children.map(childSum);
  const pc = S.get('propCull'); out.cull = pc.list.map((e) => [world.children.indexOf(e.o), e.x, e.z, e.r]);
  builds.push(out);
  console.log(`build ${n}${scp ? ' ' + scp : ''}: ${out.draws} draws, ${world.children.length} children, themes ${ts.ran}/${ts.skipped}`);
}
fs.writeFileSync(path.join(OUT, 'world_build.json'), JSON.stringify({ builds }));

// ============================================================ F. settleProps on synthetic props ===
const settleCases = [];
for (const [n, seed] of [[2, 77], [5, 55], [9, 88]]) {
  S.set('levelNum', n); S.set('levelScp', null);
  withSeed(seed, () => S.genLayoutLevel(n, Math.min(31 + n * 4, 53)));
  const W = S.get('W'), H = S.get('H'), tiles = S.get('tiles'), tileCeil = S.get('tileCeil');
  const open = [], edgeWall = [];
  for (let i = 0; i < W * H; i++) {
    if (tiles[i] === 0) open.push(i);
    else { const x = i % W, z = (i / W) | 0; if (x > 0 && z > 0 && x < W - 1 && z < H - 1 && !S.isWallTile(x + 1, z)) edgeWall.push(i); }
  }
  // low ceilings over some open tiles (so tall things raise them)
  const lows = [];
  for (let k = 0; k < 60; k++) { const t = open[qi(0, open.length - 1)]; tileCeil[t] = 2.15 + 0.1 * (k % 4); lows.push([t, tileCeil[t]]); }
  const spec = [];
  const tileFrac = (t, fu, fv) => [((t % W) + fu - W / 2) * 4, (((t / W) | 0) + fv - H / 2) * 4];
  for (let k = 0; k < 180; k++) {
    const kind = ['floor', 'wall', 'ceil', 'tall', 'inwall', 'light', 'group', 'grouprot'][k % 8];
    const t = kind === 'inwall' ? edgeWall[qi(0, edgeWall.length - 1)] : open[qi(0, open.length - 1)];
    const edge = () => { const s = qi(0, 3); return s === 0 ? [qr(0.82, 0.99), qr(0.2, 0.8)] : s === 1 ? [qr(0.01, 0.18), qr(0.2, 0.8)] : s === 2 ? [qr(0.2, 0.8), qr(0.82, 0.99)] : [qr(0.2, 0.8), qr(0.01, 0.18)]; };
    const sz = [qr(0.08, 0.7), qr(0.05, 0.6), qr(0.04, 0.7)];
    const e = { kind, w: sz[0], h: sz[1], d: sz[2], ry: k % 3 === 0 ? qr(-3, 3) : 0, kids: [] };
    let fu, fv;
    if (kind === 'inwall') { [fu, fv] = [qr(0.95, 0.999), qr(0.2, 0.8)]; }
    else if (kind === 'floor' || kind === 'tall') { [fu, fv] = k % 2 ? edge() : [qr(0, 1), qr(0, 1)]; }
    else [fu, fv] = edge();
    const [x, z] = tileFrac(t, fu, fv);
    const y = kind === 'ceil' ? 3.05 : kind === 'wall' || kind === 'inwall' ? qr(0.9, 2.2) : kind === 'tall' ? (e.h = qr(1.9, 2.7)) / 2 : kind === 'light' ? 2.4 : e.h / 2;
    e.p = [x, y, z];
    if (kind === 'group' || kind === 'grouprot') {
      e.p = [0, 0, 0]; e.ry = kind === 'grouprot' ? 0.5 : 0;
      for (let c = 0; c < 3; c++) { const tt = open[qi(0, open.length - 1)]; const [cx, cz] = tileFrac(tt, ...edge()); e.kids.push([cx, qr(0.1, 2.0), cz, qr(0.1, 0.5), qr(0.1, 0.5), qr(0.03, 0.4)]); }
    }
    spec.push(e);
  }
  const ints = []; for (let k = 0; k < 14; k++) { const t = open[qi(0, open.length - 1)]; const [x, z] = tileFrac(t, ...[qr(0, 1), qr(0, 1)]); ints.push([x, z]); }
  const pt = open[qi(0, open.length - 1)];
  const [px, pz] = [S.wx(pt % W), S.wz((pt / W) | 0)];
  const pool = { minX: px - 3, maxX: px + 5, minZ: pz - 3, maxZ: pz + 3, waterY: 0.08, splashY: 0.1 };
  // build the world from the spec
  const world = new THREE.Group(); S.set('world', world);
  const objs = [];
  for (const e of spec) {
    let o;
    if (e.kind === 'light') o = new THREE.PointLight(0xffffff, 1, 5, 2);
    else if (e.kind === 'group' || e.kind === 'grouprot') {
      o = new THREE.Group();
      for (const [cx, cy, cz, w, h, d] of e.kids) { const m = new THREE.Mesh(new THREE.BoxGeometry(w, h, d), PROBE_MATS[0]); m.position.set(cx, cy, cz); o.add(m); }
    } else o = new THREE.Mesh(new THREE.BoxGeometry(e.w, e.h, e.d), PROBE_MATS[1]);
    o.position.set(...e.p); o.rotation.y = e.ry;
    world.add(o); objs.push(o);
  }
  S.set('interactables', ints.map(([x, z]) => ({ pos: new THREE.Vector3(x, 1.6, z) })));
  S.set('lightSwitches', []);
  const runs = [];
  for (const usePool of [false, true]) {
    // the same props, settled twice (without and with a pool)
    for (let k = 0; k < objs.length; k++) { objs[k].position.set(...spec[k].p); objs[k].children.forEach((c, j) => c.position.set(...spec[k].kids[j].slice(0, 3))); }
    for (let i = 0; i < W * H; i++) tileCeil[i] = S.get('layout').zones[S.get('layout').zoneOf[i]].ceilH;
    for (const [t, c] of lows) tileCeil[t] = c;
    S.set('poolBounds', usePool ? pool : null);
    S.settleProps(0);
    runs.push({
      pool: usePool,
      stats: { ...S.get('settleStats') },
      solids: S.get('solids').count,
      tileCeil: Array.from(tileCeil),
      pos: objs.map((o) => [o.position.x, o.position.y, o.position.z, o.children.map((c) => [c.position.x, c.position.y, c.position.z])]),
    });
  }
  S.set('interactables', []); S.set('poolBounds', null);
  settleCases.push({ n, seed, lows, spec, ints, pool: [pool.minX, pool.maxX, pool.minZ, pool.maxZ], runs });
  console.log(`settle ${n}:`, runs.map((r) => JSON.stringify(r.stats)).join(' | '));
}
fs.writeFileSync(path.join(OUT, 'world_settle.json'), JSON.stringify({ cases: settleCases }));
