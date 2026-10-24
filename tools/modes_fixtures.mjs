// Differential fixtures for the Stairwell / Loop glue (backrooms.js lines
// 68246–68603, ported to game/src/modes.rs).
//
// The glue's own source is cut out of backrooms.js and run as a module under
// node, with the real src/stairwell.js and src/loop.js and the vendored
// three.js r160; every other game global it touches is a stub that behaves
// like the game (camera, lights, fog, the DOM elements, the player record,
// timers) or records the call (sfx, soundscape, showMsg, onDeath, initLevel,
// ...). A scripted bot plays each scenario through keys and mouse look; the
// per-frame inputs, sampled snapshots of everything the glue writes, and the
// recorded calls go to game/tests/fixtures/modes.json, which
// game/tests/modes_glue_js.rs replays against the Rust glue.
//
// Random streams: the glue's Math.random is mulberry32(glueSeed); the two
// modules get their own `nativeRandom` = mulberry32(moduleSeed); three.js
// UUIDs draw from a third stream. (In the game all three are the native
// generator; separate streams keep the comparison exact.)
//
//   node tools/modes_fixtures.mjs [/path/to/OpenCarole]
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const HERE = path.dirname(fileURLToPath(import.meta.url));
const ROOT = process.argv[2] || '/Users/nohwan/Documents/Personal_Projects/OpenCarole';
const OUT = path.join(HERE, '..', 'game', 'tests', 'fixtures');
const TMP = fs.mkdtempSync(path.join(process.env.TMPDIR || '/tmp', 'modes-glue-'));

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
function counted(rng) { const f = () => { f.n++; return rng(); }; f.n = 0; return f; }
globalThis.__threeUuidRandom = mulberry32(0xC0FFEE);

const THREE = await import(path.join(ROOT, 'vendor/three.module.min.js'));
const SW = await import(path.join(ROOT, 'src/stairwell.js'));
const LP = await import(path.join(ROOT, 'src/loop.js'));

// ------------------------------------------------------- the JS sources ---
const SRC = fs.readFileSync(path.join(ROOT, 'backrooms.js'), 'utf8').split('\n');
const lineOf = (pred, from = 0) => { for (let i = from; i < SRC.length; i++) if (pred(SRC[i])) return i; throw new Error('marker not found'); };
const g0 = lineOf((l) => l.startsWith('// ============================== the Stairwell'));
const g1 = lineOf((l) => l.startsWith('// ============================== game flow'), g0);
const GLUE = SRC.slice(g0, g1).join('\n');
const block = (startPred, endPred) => { const a = lineOf(startPred); const b = lineOf(endPred, a); return SRC.slice(a, b + 1).join('\n'); };
const UTILS = block((l) => l.startsWith('const rand = (a = 1, b)'), (l) => l.startsWith('const lerp = '));
const SEED_FOR_LEVEL = block((l) => l.startsWith('function seedForLevel('), (l) => l === '}');
const TEXT_POOLS_SRC = block((l) => l.startsWith('const TEXT_POOLS = {'), (l) => l === '};');
const PICK_TEXT = block((l) => l.startsWith('const pickText = '), (l) => l.startsWith('const pickText = '));
const BEAM = block((l) => l.startsWith('function beamBatteryScale('), (l) => l === '}');
const RELEASE = block((l) => l.startsWith('function releasePooledLights('), (l) => l === '}');
console.log(`glue: lines ${g0 + 1}–${g1} of backrooms.js`);

const MODULE_SRC = `
const H = globalThis.__H;
const { THREE, creatureGeometry, createStairwell, STAIR, createLoop, loopReverb, withCavityShade, canvasTex, nativeRandom,
  scene, document, scpOnLevel, sfx, fadeEl, flashEl, promptEl, world, entities, pooled, hallucShape, ambLight, hemiLight,
  playerLight, flashlight, flashlightTarget, player, showMsg, initLevel, saveGame, levelTitle, LL, keys, touchInput, camera,
  onDeath, makeWallpaper, WALL_PALETTES, carpetVariants, ceilingVariants, lightTex, renderer, nextLevel,
  hintFirstUse, setTimeout, log } = H;
const EYE = 1.62, CROUCH_EYE = 1.05;
const WALK = 3.6, SPRINT = 6.4, CROUCH_WALK = 2.0;
const STAM_MAX = 100, STAM_DRAIN = 20, STAM_REGEN = 14;
const SAN_MAX = 100;
const BAT_MAX = 100, BAT_DRAIN = 4.0;
let state = 'playing', perfT = 0, shakeT = 0, levelNum = H.init.levelNum, runSeed = H.init.runSeed, soundscape = H.init.soundscape, AC = H.init.AC, poolT = 0;
${UTILS}
${SEED_FOR_LEVEL}
${TEXT_POOLS_SRC}
const pickText0 = (key) => choice(TEXT_POOLS[key]);
const pickText = (key) => { log('pickText:' + key); return pickText0(key); };
${BEAM}
function releasePooledLights_() {}
${RELEASE.replace('function releasePooledLights(', 'function releasePooledLightsReal(')}
function releasePooledLights() { log('releasePooledLights'); releasePooledLightsReal(); }
${GLUE}
export const api = {
  enterStairwell, exitStairwell, leaveStairwellMode, updateStairwell, enterLoop, leaveLoopMode, exitLoop, updateLoop,
  stairFlightsFor, loopEntryLap,
  get: () => ({ state, perfT, shakeT, levelNum, inStairwell, inLoop, stairNearExit, loopNearExit, stairAudio, loopAudio, loopRooms, stairwell, poolT }),
  set: (o) => {
    if ('state' in o) state = o.state; if ('perfT' in o) perfT = o.perfT; if ('shakeT' in o) shakeT = o.shakeT;
    if ('levelNum' in o) levelNum = o.levelNum;
  },
};
`;
const MODULE_PATH = path.join(TMP, 'glue.mjs');
fs.writeFileSync(MODULE_PATH, MODULE_SRC);

// ------------------------------------------------------------ the stubs ---
const stubCtx = new Proxy({}, {
  get(t, k) {
    if (k in t) return t[k];
    if (typeof k === 'string' && k.startsWith('create')) return () => ({ addColorStop() {} });
    if (k === 'measureText') return () => ({ width: 0 });
    return () => {};
  },
  set(t, k, v) { t[k] = v; return true; },
});
function stubCanvasTex(w, h, draw, repeat = true) { draw(stubCtx, w, h); const t = new THREE.Texture(); t.userData.size = [w, h, repeat]; return t; }
function el(init = {}) { return { textContent: init.text ?? '', style: { opacity: '0', background: '#000', ...(init.style || {}) } }; }
function classList() { const s = new Set(); return { add: (c) => s.add(c), remove: (c) => s.delete(c), contains: (c) => s.has(c), list: () => [...s].sort().join(' ') }; }
// the sound updates every frame makes
const BORING = /^(ss\.setListener|slot\.at|voice\.setLevel|hum\.update|hum\.silence|hum\.detune|ss\.setZone:loop)/;
const KEYS = ['KeyW', 'KeyS', 'KeyA', 'KeyD', 'ShiftLeft', 'KeyC', 'ControlLeft', 'ArrowUp'];

// one session: a fresh copy of the glue and of every game global it uses
let sessionN = 0;
async function session(opts) {
  const trace = [];
  const log = (name, ...nums) => trace.push([name, nums.map((v) => (v === null || v === undefined ? NaN : +v))]);
  const glueRng = counted(mulberry32(opts.glueSeed));
  const moduleRng = counted(mulberry32(opts.moduleSeed));
  const camera = new THREE.PerspectiveCamera(74, 1, 0.05, 90); camera.rotation.order = 'YXZ';
  const scene = new THREE.Scene();
  scene.fog = new THREE.FogExp2(0x070604, 0.052); scene.background = new THREE.Color(0x070604);
  const world = new THREE.Group(); scene.add(world);
  const flashlight = new THREE.SpotLight(0xfff2cf, 0, 28, Math.PI / 5.5, 0.35, 1.2);
  const flashlightTarget = new THREE.Object3D(); flashlight.target = flashlightTarget;
  flashlight.position.set(0, 0, 0);   // (three puts a SpotLight at DEFAULT_UP)
  scene.add(flashlight); scene.add(flashlightTarget);
  const playerLight = new THREE.PointLight(0xfff2cf, 2.4, 7, 1.6); scene.add(playerLight);
  const ambLight = new THREE.AmbientLight(0xa89a70, 0.5);
  const hemiLight = new THREE.HemisphereLight(0xd0c48c, 0x2a2418, 0.6);
  scene.add(ambLight); scene.add(hemiLight);
  const pooled = [];
  for (let i = 0; i < 18; i++) {
    const l = new THREE.SpotLight(0xffeec4, i < 4 ? 1.5 : 0, 10, Math.PI / 2, 1, 1.6);
    l.target = new THREE.Object3D(); l.userData.panel = i < 4 ? i : null; l.userData.w = i < 4 ? 1 : 0;
    l.position.set(i, 2.9, -i); l.target.position.set(i, 0, -i);
    scene.add(l); scene.add(l.target); pooled.push(l);
  }
  const hallucShape = new THREE.Mesh(new THREE.PlaneGeometry(0.9, 1.9), new THREE.MeshBasicMaterial()); hallucShape.visible = true;
  const entities = [{ face: new THREE.Mesh() }, { face: null }, { face: new THREE.Mesh() }];
  const dom = { stairHud: el(), fade: el(), flash: el(), prompt: el() };
  const body = { classList: classList() };
  const player = {
    pos: new THREE.Vector3(3, 1.62, 4), yaw: 0.4, pitch: 0, roll: 0, stamina: 100, sanity: 100, battery: 100, flashlightOn: false,
    crouch: false, bobPhase: 0, bobAmp: 0, bottles: 0, batteries: 0,
    fwd() { return new THREE.Vector3(-Math.sin(this.yaw), 0, -Math.cos(this.yaw)); },
    ...(opts.player || {}),
  };
  const keys = {};
  let nowMs = 0, tseq = 0;
  const timeouts = [];
  const setTimeout = (fn, ms) => { timeouts.push({ at: nowMs + Math.max(0, ms), fn, seq: tseq++ }); return tseq; };
  const runTimeouts = () => {
    for (;;) {
      let bi = -1;
      for (let i = 0; i < timeouts.length; i++) if (timeouts[i].at <= nowMs && (bi < 0 || timeouts[i].at < timeouts[bi].at)) bi = i;
      if (bi < 0) break;
      const [t] = timeouts.splice(bi, 1); t.fn();
    }
  };
  // the soundscape: records what the glue asks of it
  let AC = null, soundscape = null;
  if (opts.sound) {
    AC = { currentTime: 0 };
    let sayN = 0;
    const slot = () => ({ input: { slotInput: true }, at: (x, y, z) => log('slot.at', x, y, z), occlude: (a) => log('slot.occlude', a) });
    soundscape = {
      world: { world: true },
      setZone: (n, f = 1.2) => log('ss.setZone:' + n, f),
      defineZone: (n, p) => log('ss.defineZone:' + n, p.t60, p.damp, p.wet, p.pre, p.er),
      hum: { silence: () => log('hum.silence'), update: (dt, x, y, z, s) => log('hum.update', dt, x, y, z, s), detune: (c, tc = 0.6) => log('hum.detune', c, tc) },
      voiceSlot: (r = 2) => { log('ss.voiceSlot', r); return slot(); },
      createChildVoice: () => {
        log('ss.createChildVoice');
        return {
          // the Rust test plays a real voice on a clock that barely moves while it
          // runs: once it has spoken it stays busy for the rest of the scenario
          get busy() { return sayN > 0; },
          say: (name, t0, pitch = 1, loud = 1) => { sayN++; const dur = 1.75; log('voice.say:' + name, t0 - AC.currentTime, pitch, loud, dur); return dur; },
          setLevel: (v, tc = 0.4) => log('voice.setLevel', v, tc),
          stop: () => log('voice.stop'),
        };
      },
      setListener: (x, y, z, fx, fy, fz) => log('ss.setListener', x, y, z, fx, fy, fz),
      footstep: (s, w = 0.6) => log('ss.footstep:' + s, w),
      screech: (i = 1, d = 0.9) => log('ss.screech', i, d),
      heartbeat: (bpm = 80, beats = 4, wrong = 0) => log('ss.heartbeat', bpm, beats, wrong),
      knock: (count = 3, x = 0, y = 0, z = 0, gain = 0.5) => log('ss.knock', count, x, y, z, gain),
      burst: (dest, t0, dur, type, f0, q, gain) => log('ss.burst:' + type, t0 - AC.currentTime, dur, f0, q, gain),
      tone: (dest, t0, dur, freq, gain, type = 'sine', f1 = null) => log('ss.tone:' + type, t0 - AC.currentTime, dur, freq, gain, f1),
    };
  }
  let api = null;
  const LL = { uLLOn: { value: 1 } };
  const H = {
    THREE, creatureGeometry: () => null, createStairwell: SW.createStairwell, STAIR: SW.STAIR, createLoop: LP.createLoop, loopReverb: LP.loopReverb,
    withCavityShade: (T, g) => g, canvasTex: stubCanvasTex, nativeRandom: moduleRng,
    scene, document: { getElementById: (id) => dom[id], body }, scpOnLevel: () => null,
    sfx: { doorSlam: () => log('sfx.doorSlam'), door: () => log('sfx.door'), pickup: () => log('sfx.pickup') },
    fadeEl: dom.fade, flashEl: dom.flash, promptEl: dom.prompt, world, entities, pooled, hallucShape, ambLight, hemiLight,
    playerLight, flashlight, flashlightTarget, player, LL, keys, touchInput: { active: false, x: 0, z: 0, sprint: false, crouch: false }, camera,
    showMsg: (t, d = 2) => log('msg:' + t, d),
    initLevel: (n) => { log('initLevel', n); api.leaveStairwellMode(); api.leaveLoopMode(); api.set({ levelNum: n }); },
    saveGame: () => log('saveGame'),
    levelTitle: () => 'TITLE',
    onDeath: (c) => { log('onDeath:' + c); if (api.get().state === 'playing') api.set({ state: 'dead' }); },
    makeWallpaper: () => new THREE.Texture(), WALL_PALETTES: [{}], carpetVariants: [new THREE.Texture(), new THREE.Texture()],
    ceilingVariants: [new THREE.Texture()], lightTex: new THREE.Texture(),
    renderer: { compile: () => log('renderer.compile') },
    nextLevel: () => { log('nextLevel'); if (api.get().state === 'playing') api.set({ state: 'transition' }); },
    hintFirstUse: (k) => log('hintFirstUse:' + k),
    setTimeout, log,
    init: { levelNum: opts.levelNum, runSeed: opts.runSeed, soundscape, AC },
  };
  globalThis.__H = H;
  const realRandom = Math.random;
  Math.random = glueRng;
  try {
    api = (await import(pathToFileURL(MODULE_PATH).href + '?s=' + (++sessionN))).api;
  } finally { Math.random = realRandom; }

  // ---- the snapshot: everything the glue writes, by name
  const N = {
    'cam.x': () => camera.position.x, 'cam.y': () => camera.position.y, 'cam.z': () => camera.position.z,
    'cam.rx': () => camera.rotation.x, 'cam.ry': () => camera.rotation.y, 'cam.rz': () => camera.rotation.z,
    'pos.x': () => player.pos.x, 'pos.y': () => player.pos.y, 'pos.z': () => player.pos.z, yaw: () => player.yaw, pitch: () => player.pitch,
    sanity: () => player.sanity, stamina: () => player.stamina, battery: () => player.battery, flashlightOn: () => +player.flashlightOn,
    crouch: () => +player.crouch, bobPhase: () => player.bobPhase, bobAmp: () => player.bobAmp, bottles: () => player.bottles, batteries: () => player.batteries,
    shakeT: () => api.get().shakeT, perfT: () => api.get().perfT, levelNum: () => api.get().levelNum,
    'fl.i': () => flashlight.intensity, 'fl.dist': () => flashlight.distance, 'fl.x': () => flashlight.position.x, 'fl.y': () => flashlight.position.y, 'fl.z': () => flashlight.position.z,
    'tg.x': () => flashlightTarget.position.x, 'tg.y': () => flashlightTarget.position.y, 'tg.z': () => flashlightTarget.position.z,
    'pl.dist': () => playerLight.distance, 'pl.i': () => playerLight.intensity, 'pl.x': () => playerLight.position.x, 'pl.y': () => playerLight.position.y, 'pl.z': () => playerLight.position.z,
    'amb.i': () => ambLight.intensity, 'amb.r': () => ambLight.color.r, 'amb.g': () => ambLight.color.g, 'amb.b': () => ambLight.color.b,
    'hemi.i': () => hemiLight.intensity, 'hemi.r': () => hemiLight.color.r, 'hemi.g': () => hemiLight.color.g, 'hemi.b': () => hemiLight.color.b,
    'gnd.r': () => hemiLight.groundColor.r, 'gnd.g': () => hemiLight.groundColor.g, 'gnd.b': () => hemiLight.groundColor.b,
    'fog.d': () => scene.fog.density, 'fog.r': () => scene.fog.color.r, 'fog.g': () => scene.fog.color.g, 'fog.b': () => scene.fog.color.b,
    'bg.r': () => scene.background.r, 'bg.g': () => scene.background.g, 'bg.b': () => scene.background.b, llOn: () => LL.uLLOn.value,
    inStairwell: () => +api.get().inStairwell, stairNearExit: () => +api.get().stairNearExit, inLoop: () => +api.get().inLoop, loopNearExit: () => +api.get().loopNearExit,
    'sa.nextPhrase': () => api.get().stairAudio.nextPhrase, 'sa.nextKnock': () => api.get().stairAudio.nextKnock, 'sa.heartT': () => api.get().stairAudio.heartT,
    'sa.stepAcc': () => api.get().stairAudio.stepAcc, 'la.stepAcc': () => api.get().loopAudio.stepAcc, 'la.touchT': () => api.get().loopAudio.touchT,
    worldVis: () => +world.visible, hallucVis: () => +hallucShape.visible, e0Vis: () => +entities[0].face.visible, e2Vis: () => +entities[2].face.visible,
    'sw.groupVis': () => +api.get().stairwell.group.visible, 'sw.faceVis': () => +api.get().stairwell.face.visible,
    'sw.seg': () => api.get().stairwell.state.seg, 'sw.x': () => api.get().stairwell.state.x, 'sw.z': () => api.get().stairwell.state.z, 'sw.y': () => api.get().stairwell.state.y,
    'sw.landing': () => api.get().stairwell.landing, 'sw.depth': () => api.get().stairwell.depthM, 'sw.faceCount': () => api.get().stairwell.state.faceCount,
    'lp.present': () => +!!api.get().loopRooms, 'lp.lap': () => (api.get().loopRooms ? api.get().loopRooms.lap : -99), 'lp.room': () => (api.get().loopRooms ? api.get().loopRooms.room : -99),
    'lp.x': () => (api.get().loopRooms ? api.get().loopRooms.state.x : 0), 'lp.z': () => (api.get().loopRooms ? api.get().loopRooms.state.z : 0),
    'lp.seams': () => (api.get().loopRooms ? api.get().loopRooms.state.seams : -1), 'lp.inScene': () => +!!(api.get().loopRooms && api.get().loopRooms.group.parent === scene),
    poolT: () => api.get().poolT,
    glueDraws: () => glueRng.n, moduleDraws: () => moduleRng.n,
  };
  for (const i of [0, 1, 2, 3, 4, 5, 6, 17]) {
    const l = pooled[i], p = `pool${i}.`;
    Object.assign(N, {
      [p + 'i']: () => l.intensity, [p + 'dist']: () => l.distance, [p + 'x']: () => l.position.x, [p + 'y']: () => l.position.y, [p + 'z']: () => l.position.z,
      [p + 'tx']: () => l.target.position.x, [p + 'ty']: () => l.target.position.y, [p + 'tz']: () => l.target.position.z,
      [p + 'r']: () => l.color.r, [p + 'g']: () => l.color.g, [p + 'b']: () => l.color.b,
    });
  }
  const S = {
    state: () => api.get().state, stairHud: () => dom.stairHud.textContent, prompt: () => dom.prompt.textContent,
    fadeOp: () => String(dom.fade.style.opacity), fadeBg: () => String(dom.fade.style.background), flashOp: () => String(dom.flash.style.opacity),
    body: () => body.classList.list(), faceKind: () => { const f = api.get().stairwell.state.face; return f ? f.kind : ''; },
  };
  // each kind of scenario keeps the fields its mode writes
  const keep = (k) => opts.kind === 'stair'
    ? !/^pool/.test(k) || k === 'pool0.i' || k === 'pool17.i' || k === 'pool3.i'
    : !/^(sw\.|sa\.)/.test(k) || k === 'sw.groupVis' || k === 'sw.faceVis';
  const numNames = Object.keys(N).filter(keep), strNames = Object.keys(S);
  const r15 = (v) => (typeof v === 'number' && Number.isFinite(v) && !Number.isInteger(v) ? +v.toPrecision(15) : v);

  // ---- the frame driver
  const script = [], snaps = [];
  let cur = { k: 0, y: player.yaw, p: player.pitch };
  let frame = 0, traceAt = 0;
  const doAction = (a) => {
    const [kind, ...args] = a;
    if (kind === 'set') {
      const [k, v] = args;
      if (k in player) player[k] = typeof player[k] === 'boolean' ? !!v : v;
      else if (k === 'nextKnock' || k === 'nextPhrase' || k === 'heartT') api.get().stairAudio[k] = v;
      else if (k === 'shakeT' || k === 'state') api.set({ [k]: v });
      else throw new Error('set ' + k);
    } else if (kind === 'stairPlace') {
      const [seg, x, z] = args; const sw = api.get().stairwell, st = sw.state;
      st.seg = seg; st.x = x; st.z = z; st.y = sw.segY(seg, x, z); if (seg > st.deepestSeg) st.deepestSeg = seg;
    } else if (kind === 'loopPlace') {
      const [lap, z, x, yaw, pitch] = args; const g = api.get();
      if (!g.inLoop || !g.loopRooms) return;
      player.yaw = yaw; player.pitch = pitch;
      g.loopRooms.place(lap, z, x, yaw);
      Math.random = glueRng; try { api.updateLoop(0); } finally { Math.random = realRandom; }
    } else {
      Math.random = glueRng; try { api[kind](); } finally { Math.random = realRandom; }
    }
  };
  // one frame: input, actions, timers, the clock, the update; `full` snapshots everything
  let events = 0;
  const evSeen = new Map();
  const watch = (m) => { if (m && !m.__watched) { const u = m.update; m.update = (...a) => {
    const o = u(...a); events += o.events.length + (o.touch > 0 ? 1 : 0);
    for (const e of o.events) { const k = e.type + (e.n ? e.n : '') + (e.reason ? ':' + e.reason : ''); evSeen.set(k, (evSeen.get(k) || 0) + 1); }
    if (o.touch > 0) evSeen.set('touch', (evSeen.get('touch') || 0) + 1);
    return o; }; m.__watched = true; } };
  const step = ({ keys: km = cur.k, yaw = cur.y, pitch = cur.p, dt = 1 / 60, actions = [], snap = null } = {}) => {
    const rec = {};
    watch(api.get().stairwell); watch(api.get().loopRooms);
    const ev0 = events;
    if (km !== cur.k) rec.k = km;
    if (yaw !== cur.y) rec.y = yaw;
    if (pitch !== cur.p) rec.p = pitch;
    if (dt !== 1 / 60) rec.dt = dt;
    if (actions.length) rec.a = actions;
    cur = { k: km, y: yaw, p: pitch };
    for (let b = 0; b < KEYS.length; b++) keys[KEYS[b]] = !!(km & (1 << b));
    player.yaw = yaw; player.pitch = pitch;
    for (const a of actions) doAction(a);
    nowMs += dt * 1000;
    Math.random = glueRng;
    try {
      runTimeouts();
      const g = api.get();
      api.set({ perfT: g.perfT + dt });
      if (g.state === 'playing' && g.inStairwell) api.updateStairwell(dt);
      else if (g.state === 'playing' && g.inLoop) api.updateLoop(dt);
    } finally { Math.random = realRandom; }
    // the player may have been moved by the frame (loopPlace sets yaw)
    cur.y = player.yaw; cur.p = player.pitch;
    script.push(rec);
    const tr = trace.slice(traceAt); traceAt = trace.length;
    // a full snapshot every fifth frame and wherever something happens; the
    // per-frame sound updates in between are not kept (the test checks that
    // nothing else was called on the frames that are not here)
    const want = snap ?? (frame % (opts.every || 5) === 0 || tr.some((e) => !BORING.test(e[0])) || actions.length > 0 || events !== ev0);
    if (want) snaps.push({ f: frame, n: numNames.map((k) => r15(N[k]())), s: strNames.map((k) => S[k]()), t: tr.map(([n, a]) => [n, a.map(r15)]) });
    frame++;
  };
  const get = () => api.get();
  const finish = (extra = {}) => ({ ...opts.meta, kind: opts.kind, sound: !!opts.sound, glueSeed: opts.glueSeed, moduleSeed: opts.moduleSeed, levelNum: opts.levelNum, runSeed: opts.runSeed,
    player0: opts.player || {}, numNames, strNames, script, snaps, frames: frame, ...extra });
  return { step, get, player, finish, camera, trace, keys, evSeen };
}

const K = { W: 1, S: 2, A: 4, D: 8, SHIFT: 16, C: 32, CTRL: 64, UP: 128 };
const FL = SW.STAIR.STEPS * SW.STAIR.RUN;
const fxLane = (odd) => (odd ? 1 : -1) * (SW.STAIR.SPINE + (SW.STAIR.HALF - SW.STAIR.SPINE) / 2);
// where to walk next on the stairs (the scripted descent of tools/sl_fixtures.mjs)
function stairTarget(st, phase) {
  const goingDown = phase === 'down' || phase === 'final';
  const seg = st.seg, x = st.x, z = st.z, LAST = 2 * (st.flights - 1) + 1;
  if (seg === LAST && phase === 'final') return ((LAST - 1) / 2) % 2 === 0 ? [0, FL + 1.0] : [0, -1.0];   // the door, on the last landing
  if (seg % 2 === 0) {
    const k = seg / 2, odd = k % 2 !== 0;
    if (goingDown) return odd ? [fxLane(true), -0.6] : [fxLane(false), FL + 0.6];
    return odd ? [fxLane(true), FL + 0.6] : [fxLane(false), -0.6];
  }
  const k = (seg - 1) / 2, far = k % 2 === 0;
  if (goingDown) {
    if (far) return x < 0.6 ? [fxLane(true), FL + 0.55] : [fxLane(true), FL - 0.6];
    return x > -0.6 ? [fxLane(false), -0.55] : [fxLane(false), 0.6];
  }
  if (k < 0) return [x, z];
  if (far) return x > -0.6 ? [fxLane(false), FL + 0.55] : [fxLane(false), FL - 0.6];
  return x < 0.6 ? [fxLane(true), -0.55] : [fxLane(true), 0.6];
}
const evTypes = (sess) => [...new Set(sess.trace.map((e) => e[0].split(':')[0]))].join(' ') + '\n    events: ' + JSON.stringify([...sess.evSeen]);

// ======================================================== the Stairwell ===
async function stairDescent() {
  const s = await session({ glueSeed: 5101, moduleSeed: 99, levelNum: 2, runSeed: 777, kind: 'stair', every: 10, player: { sanity: 400 }, meta: { name: 'stairDescent' } });
  s.step({ actions: [['enterStairwell'], ['enterStairwell']] });     // the second is ignored (mid-transition)
  for (let i = 0; i < 60; i++) s.step();
  let phase = 'down', turnT = 0, armedT = 0, holdT = 0, mf = 0, exited = false;
  const seen = new Set();
  for (let f = 0; f < 12000 && !exited; f++) {
    const g = s.get(), st = g.stairwell.state, sw = g.stairwell;
    if (g.state !== 'playing' || !g.inStairwell) { s.step(); continue; }
    mf++;
    let km = K.W; const actions = [];
    if (phase === 'down' && st.deepestSeg >= 12) phase = 'up';
    if (phase === 'up' && st.face && st.face.kind === 'behind') { armedT += 1 / 60; if (armedT > 0.6) phase = 'turn'; }
    if (phase === 'up' && st.seg <= 0) phase = 'turn';
    let yaw = s.player.yaw, pitch = s.player.pitch;
    if (phase === 'turn') { km = 0; turnT += 1 / 60; if (turnT > 1.4) phase = 'final'; }
    const [tx, tz] = stairTarget(st, phase);
    const dx = tx - st.x, dz = tz - st.z;
    if (phase === 'turn') yaw += Math.PI / 60 / 0.9 * (turnT < 0.9 ? 1 : 0);
    else if (Math.hypot(dx, dz) > 1e-3) yaw = Math.atan2(-dx, -dz);
    if (phase === 'final' && st.seg === 2 * (st.flights - 1) + 1 && Math.hypot(dx, dz) < 0.1) { km = 0; holdT += 1 / 60; }
    // variety: crouch now and then, a sprint, a strafe wobble, looking about
    if (km) {
      if (mf % 400 > 350) km |= K.C; else if (mf % 700 > 640) km |= K.SHIFT;
      if (mf % 300 > 280) km |= K.D;
      if (mf % 900 > 880) km = K.UP | K.A;
    }
    pitch = phase === 'up' ? 0.1 : -0.25;
    if (st.face && st.face.kind === 'below' && st.faceCount <= 2 && phase !== 'turn') {
      const fp = sw.face.position, cp = s.camera.position, ddx = fp.x - cp.x, ddy = fp.y - cp.y, ddz = fp.z - cp.z;
      if (Math.hypot(ddx, ddy, ddz) < 8.5) { km = 0; yaw = Math.atan2(-ddx, -ddz); pitch = Math.atan2(ddy, Math.hypot(ddx, ddz)); }
    }
    // the lamp: off for a while (the dark wears on you), a battery that dies, a fresh one
    if (mf === 300) actions.push(['set', 'flashlightOn', 0]);
    if (mf === 480) actions.push(['set', 'flashlightOn', 1]);
    if (mf === 900) actions.push(['set', 'battery', 0.25]);
    if (mf === 1000) { actions.push(['set', 'battery', 70]); actions.push(['set', 'flashlightOn', 1]); }
    if (mf === 1500) actions.push(['set', 'stamina', 0.5]);
    if (g.stairNearExit && holdT > 0.3) { actions.push(['exitStairwell']); exited = true; }
    if (process.env.DBG && mf % 120 === 0) console.log(mf, phase, 'seg', st.seg, 'deep', st.deepestSeg, st.x.toFixed(2), st.z.toFixed(2), 'face', st.face && st.face.kind, st.faceCount, 'san', s.player.sanity.toFixed(1), 'km', km);
    s.step({ keys: km, yaw, pitch, actions });
  }
  for (let i = 0; i < 90; i++) s.step();
  const out = s.finish();
  console.log('stairDescent', out.frames, 'frames; calls:', evTypes(s));
  return out;
}

async function stairDeath() {
  const s = await session({ glueSeed: 77, moduleSeed: 5, levelNum: 2, runSeed: 1, player: { sanity: 3.2, battery: 0.6, stamina: 40 }, kind: 'stair', meta: { name: 'stairDeath' } });
  s.step({ actions: [['exitStairwell'], ['leaveStairwellMode'], ['enterStairwell']] });   // not inside yet: no-ops, then in
  for (let i = 0; i < 60; i++) s.step();
  for (let i = 0; i < 400 && s.get().state === 'playing'; i++) s.step({ keys: i % 50 < 30 ? K.W | K.SHIFT : K.S, yaw: Math.PI + 0.3 * Math.sin(i * 0.05), pitch: -0.2 });
  s.step({ actions: [['leaveStairwellMode']], snap: true });
  s.step({ actions: [['leaveStairwellMode']], snap: true });
  const out = s.finish();
  console.log('stairDeath', out.frames, 'frames; calls:', evTypes(s));
  return out;
}

async function stairSound() {
  const s = await session({ sound: true, glueSeed: 4242, moduleSeed: 314, levelNum: 23, runSeed: 9, kind: 'stair', meta: { name: 'stairSound' } });
  s.step({ actions: [['enterStairwell']] });
  for (let i = 0; i < 60; i++) s.step();
  // walk a little (footsteps), sprinting and crouching; the child holds its
  // first phrase until near the end (its voice stays busy once it has spoken)
  for (let i = 0; i < 120; i++) s.step({ keys: i < 40 ? K.W : i < 80 ? K.W | K.SHIFT : K.W | K.C, yaw: Math.PI, pitch: -0.25, actions: i === 0 ? [['set', 'nextPhrase', 60]] : [] });
  // deep down at once: the knocking from above and the Face on the landing below
  const sw = s.get().stairwell;
  const k = Math.max(17, sw.state.encounterAt[0] ?? 17), seg = 2 * k + 1;
  console.log('stairSound: encounters at', JSON.stringify(sw.state.encounterAt), '→ seg', seg);
  s.step({ actions: [['stairPlace', seg, 0.0, k % 2 === 0 ? FL + 0.7 : -0.7], ['set', 'nextKnock', 0.3]], yaw: k % 2 === 0 ? Math.PI : 0, pitch: -0.3 });
  let stared = 0;
  for (let f = 0; f < 700; f++) {
    const st = sw.state;
    let yaw = s.player.yaw, pitch = s.player.pitch, km = 0;
    if (st.face && st.face.kind === 'below') {
      const fp = sw.face.position, cp = s.camera.position, ddx = fp.x - cp.x, ddy = fp.y - cp.y, ddz = fp.z - cp.z;
      yaw = Math.atan2(-ddx, -ddz); pitch = Math.atan2(ddy, Math.hypot(ddx, ddz)); stared++;
    } else if (f > 200) { km = K.W; const [tx, tz] = stairTarget(st, 'down'); yaw = Math.atan2(-(tx - st.x), -(tz - st.z)); pitch = -0.25; }
    const actions = f === 600 ? [['set', 'nextPhrase', 0.05]] : [];
    s.step({ keys: km, yaw, pitch, actions });
  }
  s.step({ actions: [['leaveStairwellMode']], snap: true });
  const out = s.finish();
  console.log('stairSound', out.frames, 'frames; stared', stared, '; calls:', evTypes(s));
  return out;
}

// ============================================================= the Loop ===
async function loopWalk() {
  const s = await session({ glueSeed: 8080, moduleSeed: 31337, levelNum: 4, runSeed: 123457, player: { flashlightOn: true, battery: 30 }, kind: 'loop', every: 8, meta: { name: 'loopWalk' } });
  s.step({ actions: [['leaveLoopMode'], ['exitLoop'], ['enterLoop']] });
  for (let i = 0; i < 60; i++) s.step();
  const L = () => s.get().loopRooms;
  const P = LP.LOOP_PITCH;
  const yawTo = (dx, dz) => Math.atan2(-dx, -dz);
  let mf = 0;
  const keysFor = (base) => { mf++; if (mf % 500 > 470) return base | K.SHIFT; if (mf % 650 > 630) return base | K.C; return base; };
  const goTo = (tx, tz, maxF = 240) => {
    for (let i = 0; i < maxF; i++) {
      if (s.get().state !== 'playing' || !L()) return false;
      const st = L().state, dx = tx - st.x, dz = tz - st.z;
      if (Math.hypot(dx, dz) < 0.15) return true;
      s.step({ keys: K.W, yaw: yawTo(dx, dz) });
    }
    return false;
  };
  const stand = (n, yaw) => { for (let i = 0; i < n; i++) s.step({ keys: 0, yaw }); };
  stand(10, s.player.yaw);
  goTo(0, L().state.z - 0.4);
  let lastG = -1e9; const lookedBack = new Set();
  for (let step = 0; step < 6000; step++) {
    if (s.get().state !== 'playing') break;
    const lr = L(), S = lr.state, lapR = lr.lap, room = lr.room, zc = room * P + P / 2;
    if (lapR >= 7 && room >= 1) break;
    const gIdx = S.lap * 5 + room;
    if (gIdx !== lastG) {
      lastG = gIdx;
      const kind = LP.pickupAt(lapR, room), key = lapR * 8 + room;
      if (kind && !S.taken.has(key) && S.z < zc - 1.9) {
        const bare = LP.lapLook(lapR).bare;
        if (kind === 'almond') { if (bare) { goTo(-1.3, zc - 0.2); goTo(-0.2, zc + 0.6); } else { goTo(-1.35, zc - 1.0); goTo(-0.3, zc - 1.2); goTo(0, zc + 0.6); } }
        else { if (bare) { goTo(1.35, zc - 1.3); goTo(0.2, zc + 0.6); } else { goTo(1.5, zc - 1.75); goTo(0, zc + 0.6); } }
        continue;
      }
      if (lapR >= 5 && (room === 3 || room === 4 || room === 0) && !lookedBack.has(gIdx)) {
        lookedBack.add(gIdx);
        stand(8, Math.PI);
        for (let i = 0; i < 15; i++) s.step({ keys: 0, yaw: Math.PI + (i + 1) * (Math.PI / 15) });
        stand(20, 0);
        for (let i = 0; i < 15; i++) s.step({ keys: 0, yaw: (i + 1) * (Math.PI / 15) });
      }
    }
    const zBefore = S.z, lapBefore = S.lap;
    s.step({ keys: keysFor(K.W), yaw: yawTo(Math.max(-0.3, Math.min(0.3, -S.x)), 1) });
    if (L() && Math.abs(L().state.z - zBefore) < 1e-9 && L().state.lap === lapBefore) { s.step({ keys: K.D, yaw: Math.PI }); s.step({ keys: K.D, yaw: Math.PI }); }
  }
  // the figure, up close: walk into it
  s.step({ actions: [['loopPlace', 5, 13 - 2.2, 0, Math.PI, 0]] });
  for (let i = 0; i < 90 && s.get().state === 'playing'; i++) s.step({ keys: K.W, yaw: Math.PI });
  stand(30, 0);
  // back from lap 1 to lap 0 and out of the exit door
  s.step({ actions: [['loopPlace', 1, 24.0, 0, 0, 0], ['set', 'sanity', 90]] });
  for (let i = 0; i < 3000; i++) {
    const lr = L(); if (!lr) break;
    if (lr.lap === 0 && lr.room === 0) break;
    s.step({ keys: K.W, yaw: yawTo(Math.max(-0.3, Math.min(0.3, -lr.state.x)), -1) });
  }
  const ex = L().exitDoorAt();
  goTo(ex.x + 0.3, ex.z, 300);
  stand(5, Math.PI / 2);
  s.step({ actions: s.get().loopNearExit ? [['exitLoop'], ['exitLoop']] : [], snap: true });
  const out = s.finish({ nearExitAtEnd: s.get().loopNearExit });
  console.log('loopWalk', out.frames, 'frames; exit', out.nearExitAtEnd, '; calls:', evTypes(s));
  return out;
}

async function loopSound() {
  const s = await session({ sound: true, glueSeed: 2468, moduleSeed: 1357, levelNum: 7, runSeed: 424242, kind: 'loop', meta: { name: 'loopSound' } });
  s.step({ actions: [['enterLoop']] });
  for (let i = 0; i < 60; i++) s.step();
  const L = () => s.get().loopRooms;
  // to the axis, then forward across a seam, sprinting and crouching
  for (let i = 0; i < 40; i++) s.step({ keys: K.W, yaw: Math.atan2(L().state.x, 0.4) });
  for (let i = 0; i < 420; i++) s.step({ keys: i % 140 < 60 ? K.W | K.SHIFT : i % 140 < 100 ? K.W : K.W | K.C, yaw: Math.atan2(-Math.max(-0.3, Math.min(0.3, -L().state.x)), -1) });
  // the figure: placed in lap 5, looking around, away, back, then into it
  s.step({ actions: [['loopPlace', 5, 13 - 3.2, 0, Math.PI, 0]] });
  const stand = (n, yaw) => { for (let i = 0; i < n; i++) s.step({ keys: 0, yaw }); };
  stand(30, Math.PI); stand(60, 0); stand(30, Math.PI); stand(60, 0);
  for (let i = 0; i < 90 && s.get().state === 'playing'; i++) s.step({ keys: K.W, yaw: Math.PI });
  stand(40, 0.3);
  // deep: lamps out, the hum gone
  s.step({ actions: [['loopPlace', 9, 13, 0, Math.PI, 0.2]] });
  for (let i = 0; i < 80 && s.get().state === 'playing'; i++) s.step({ keys: K.W, yaw: Math.PI });
  s.step({ actions: [['leaveLoopMode']], snap: true });
  const out = s.finish();
  console.log('loopSound', out.frames, 'frames; calls:', evTypes(s));
  return out;
}

async function loopDeath() {
  const s = await session({ glueSeed: 999, moduleSeed: 4, levelNum: 9, runSeed: 31, player: { sanity: 4.5 }, kind: 'loop', meta: { name: 'loopDeath' } });
  s.step({ actions: [['enterLoop']] });
  for (let i = 0; i < 60; i++) s.step();
  s.step({ actions: [['loopPlace', 8, 13, 0, Math.PI, 0]] });
  for (let i = 0; i < 600 && s.get().state === 'playing'; i++) s.step({ keys: i % 90 < 60 ? K.W : 0, yaw: Math.PI + 0.4 * Math.sin(i * 0.03) });
  s.step({ actions: [['leaveLoopMode']], snap: true });
  s.step({ actions: [['leaveLoopMode'], ['enterStairwell']], snap: true });   // the game is dead: neither does anything
  const out = s.finish();
  console.log('loopDeath', out.frames, 'frames; calls:', evTypes(s));
  return out;
}

// a few pure values
const pure = {
  stairFlightsFor: [], loopEntryLap: [], phrase: [],
};
{
  // the child's next phrase: the glue's own expression, cut out and run on a grid of draws
  const line = GLUE.split('\n').find((l) => l.includes('const phrase = faceOut'));
  const f = new Function('faceOut', 'r', line.trim() + ' return phrase;');
  const rs = [];
  for (let i = 0; i <= 200; i++) rs.push(i / 200);
  for (const b of [0.45, 0.62, 0.8, 0.92]) rs.push(b - 1e-12, b, b + 1e-12);
  for (const r of rs) for (const fo of [false, true]) pure.phrase.push([+fo, r, f(fo, r)]);
}
for (let n = -2; n <= 60; n++) pure.stairFlightsFor.push([n, Math.min(40, 8 + Math.floor(n / 3) * 5)]);
{
  const f = new Function(`${SEED_FOR_LEVEL}; return seedForLevel;`)();
  const r = mulberry32(17);
  for (let i = 0; i < 200; i++) { const rs = (r() * 4294967296) >>> 0, n = 1 + Math.floor(r() * 40); pure.loopEntryLap.push([rs, n, 2 + ((f(rs, n) >>> 7) % 2)]); }
}

const fixture = { pure, scenarios: [] };
for (const fn of [stairDescent, stairDeath, stairSound, loopWalk, loopSound, loopDeath]) fixture.scenarios.push(await fn());
const p = path.join(OUT, 'modes.json');
// snapshot numbers go out at 15 significant digits (the test compares at 1e-9); the inputs exactly
fixture.boring = BORING.source;
const json = JSON.stringify(fixture);
fs.writeFileSync(p, json);
for (const sc of fixture.scenarios) console.log(' ', sc.name, (JSON.stringify(sc).length / 1024).toFixed(0), 'KB');
console.log('modes.json', (fs.statSync(p).size / 1024).toFixed(0), 'KB');
fs.rmSync(TMP, { recursive: true, force: true });
