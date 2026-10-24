// Fixture generator for the Rust ports of src/stairwell.js, src/loop.js and
// src/store.js (game/src/{stairwell,loop_mode,store}.rs).
//
// Runs the original JS modules under node with the vendored three.js r160 and
// deterministic stub deps (mulberry32 rng, a canvasTex that runs the draw on a
// no-op context so the rng draws inside it still happen), and writes what
// they compute to game/tests/fixtures/*.json. The Rust tests replay the same
// inputs and assert the same results.
//
//   node tools/sl_fixtures.mjs [/path/to/OpenCarole]
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const HERE = path.dirname(fileURLToPath(import.meta.url));
const ROOT = process.argv[2] || '/Users/nohwan/Documents/Personal_Projects/OpenCarole';
const OUT = path.join(HERE, '..', 'game', 'tests', 'fixtures');
fs.mkdirSync(OUT, { recursive: true });

const THREE = await import(path.join(ROOT, 'vendor/three.module.min.js'));
const SW = await import(path.join(ROOT, 'src/stairwell.js'));
const LP = await import(path.join(ROOT, 'src/loop.js'));
const ST = await import(path.join(ROOT, 'src/store.js'));

// ------------------------------------------------------------------ stubs ---
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
const stubCtx = new Proxy({}, {
  get(t, k) {
    if (k in t) return t[k];
    if (typeof k === 'string' && k.startsWith('create')) return () => ({ addColorStop() {} });
    if (k === 'measureText') return () => ({ width: 0 });
    return () => {};
  },
  set(t, k, v) { t[k] = v; return true; },
});
const texLog = [];
function stubCanvasTex(w, h, draw, repeat = true) {
  draw(stubCtx, w, h);
  const t = new THREE.Texture();
  t.userData.size = [w, h, repeat];
  texLog.push([w, h, repeat]);
  return t;
}
const r6 = (v) => v;   // numbers go out at full precision
const v3 = (v) => [v.x, v.y, v.z];
const q4 = (q) => [q.x, q.y, q.z, q.w];
const eul = (e) => [e.x, e.y, e.z];

function attrSum(attr) {
  if (!attr) return null;
  let t = 0, ta = 0;
  const a = attr.array;
  for (let i = 0; i < a.length; i++) { t += a[i]; ta += Math.abs(a[i]) * ((i % 7) + 1); }
  return [a.length, t, ta];
}
function geoSum(g) {
  let idx = null;
  if (g.index) { let s = 0, sw = 0; const a = g.index.array; for (let i = 0; i < g.index.count; i++) { s += a[i]; sw += a[i] * ((i % 5) + 1); } idx = [g.index.count, s, sw]; }
  return { type: g.type, pos: attrSum(g.attributes.position), nor: attrSum(g.attributes.normal), uv: attrSum(g.attributes.uv), col: attrSum(g.attributes.color), idx };
}
function matInfo(m) {
  return {
    type: m.type, color: m.color ? m.color.getHex() : null, emissive: m.emissive ? m.emissive.getHex() : null,
    map: !!m.map, side: m.side, vc: !!m.vertexColors, fog: !!m.fog, tr: !!m.transparent, op: m.opacity, dw: !!m.depthWrite,
    po: !!m.polygonOffset, pof: m.polygonOffsetFactor, pou: m.polygonOffsetUnits,
  };
}
function geoParams(g) {
  const p = g.parameters || {};
  switch (g.type) {
    case 'BoxGeometry': return ['box', p.width, p.height, p.depth];
    case 'CylinderGeometry': return ['cyl', p.radiusTop, p.radiusBottom, p.height, p.radialSegments, p.heightSegments, p.openEnded, p.thetaStart, p.thetaLength];
    case 'PlaneGeometry': return ['plane', p.width, p.height];
    case 'SphereGeometry': return ['sphere', p.radius, p.widthSegments, p.heightSegments];
    case 'CircleGeometry': return ['circle', p.radius, p.segments];
    case 'TorusGeometry': return ['torus', p.radius, p.tube, p.radialSegments, p.tubularSegments, p.arc];
    default: return [g.type];
  }
}
function tree(o) {
  const n = { k: o.isInstancedMesh ? 'inst' : o.isMesh ? 'mesh' : 'group', v: o.visible, p: v3(o.position), r: eul(o.rotation) };
  if (o.isMesh) {
    n.g = geoParams(o.geometry); n.vc = o.geometry.attributes.position.count; n.ic = o.geometry.index ? o.geometry.index.count : null;
    n.m = matInfo(o.material);
  }
  if (o.isInstancedMesh) {
    n.count = o.count; n.mats = [];
    for (let i = 0; i < o.count; i++) n.mats.push(Array.from(o.instanceMatrix.array.slice(i * 16, i * 16 + 16)));
  }
  if (o.children.length) n.c = o.children.map(tree);
  return n;
}

// ============================================================== stairwell ===
function stairwellFixture() {
  const out = {};
  const { STAIR } = SW;
  const FL = STAIR.STEPS * STAIR.RUN, DROP = STAIR.STEPS * STAIR.RISE;
  out.consts = { FL, DROP, STEPS: STAIR.STEPS, RISE: STAIR.RISE, RUN: STAIR.RUN, HALF: STAIR.HALF, SPINE: STAIR.SPINE, R: STAIR.R, LIGHT_STEPS: STAIR.LIGHT_STEPS, FLIGHT_DROP: SW.STAIR_FLIGHT_DROP };

  // ---- construction: rng draws, the tree, the face
  texLog.length = 0;
  const rng = counted(mulberry32(1234));
  const s = SW.createStairwell(THREE, { canvasTex: stubCanvasTex, rng });
  out.build = { draws: rng.n, texs: texLog.slice(), tree: tree(s.group), face: geoSum(s.face.geometry), faceMat: matInfo(s.face.material) };

  // ---- locate / segY / landing probes on a 10-flight descent
  s.enter({ flights: 10 });
  out.enter10Draws = rng.n;
  const xs = [], zs = [];
  for (let i = 0; i <= 16; i++) xs.push(-1.6 + i * 0.2);
  for (let i = 0; i <= 30; i++) zs.push(-1.7 + i * 0.25);
  const segs = []; for (let k = -1; k <= 21; k++) segs.push(k);
  const loc = [];
  for (const r of [0.3, 0.05]) for (const seg of segs) for (const x of xs) for (const z of zs) { const v = s.locate(seg, x, z, r); loc.push(v === null ? -99 : v); }
  const segy = [];
  for (const seg of segs) for (const z of zs) segy.push(s.segY(seg, 0, z));
  out.probes = { xs, zs, segs, rs: [0.3, 0.05], locate: loc, segY: segy };

  // ---- enter() for many depths, one stairwell in sequence
  const enters = [];
  for (const flights of [1, 2, 3, 5, 8, 10, 13, 16, 23, 31, 40, 7.9]) {
    s.enter({ flights });
    const st = s.state;
    const C = s.group.children;
    enters.push({
      flights, draws: rng.n, flightsOut: s.flights, encounterAt: st.encounterAt.slice(),
      leds: [6, 7, 8, 9].map((i) => ({ v: C[i].visible, p: v3(C[i].position), on: C[i].material.type === 'MeshBasicMaterial', blink: !!C[i].userData.blink })),
      misc: [10, 11, 12, 13].map((i) => ({ v: C[i].visible, p: v3(C[i].position), r: eul(C[i].rotation) })),
      bottom: C[5].children.map((o) => ({ p: v3(o.position), r: eul(o.rotation) })),
      chunks: [0, 1, 2, 3].map((i) => ({ v: C[i].visible, y: C[i].position.y, right: C[i].userData.right.visible, near: C[i].userData.near.visible })),
      top: C[4].visible, landing: s.landing, depth: s.depthM, voice: (({ x, y, z }) => [x, y, z])(s.voicePos()),
    });
  }
  out.enters = enters;

  // ---- chunk placement by segment (update with no motion)
  s.enter({ flights: 40 });
  const cam0 = new THREE.PerspectiveCamera(74, 1, 0.05, 90); cam0.rotation.order = 'YXZ';
  cam0.position.set(0, 1.62, -0.5); cam0.rotation.set(0, Math.PI, 0);
  const chunkBySeg = [];
  for (const seg of [-1, 0, 1, 2, 3, 4, 5, 7, 8, 11, 12, 15, 16, 30, 50, 63, 70, 77, 79]) {
    s.state.seg = seg; s.state.encounterAt = []; s.state.behindDone = true;
    s.update(0, { fwd: 0, strafe: 0, speed: 0, yaw: 0, pitch: 0, camera: cam0 });
    const C = s.group.children;
    chunkBySeg.push({ seg, chunks: [0, 1, 2, 3].map((i) => [C[i].visible, C[i].position.y, C[i].userData.right.visible, C[i].userData.near.visible]), top: C[4].visible, landing: s.landing });
  }
  out.chunkBySeg = chunkBySeg;

  // ---- a scripted descent: down 6 flights, back up until the Face is
  // behind you, turn round, then all the way down to the door
  {
    const rng2 = counted(mulberry32(99));
    const sw = SW.createStairwell(THREE, { canvasTex: stubCanvasTex, rng: rng2 });
    sw.enter({ flights: 10 });
    const cam = new THREE.PerspectiveCamera(74, 1, 0.05, 90); cam.rotation.order = 'YXZ';
    const F = 10, LAST = 2 * (F - 1) + 1;
    const fx = (odd) => (odd ? 1 : -1) * (STAIR.SPINE + (STAIR.HALF - STAIR.SPINE) / 2);
    let x = 0.55, z = -0.5, y = 1.62, seg = -1;
    let yaw = Math.PI, pitch = -0.25;
    cam.position.set(x, y, z); cam.rotation.set(pitch, yaw, 0);
    const steps = [];
    let phase = 'down', turnT = 0, holdT = 0, armedT = 0;
    const target = () => {
      const goingDown = phase === 'down' || phase === 'final';
      if (seg === LAST && phase === 'final') return [0, -0.9];
      if (seg % 2 === 0) {          // flight k
        const k = seg / 2, odd = k % 2 !== 0;
        if (goingDown) return odd ? [fx(true), -0.6] : [fx(false), FL + 0.6];
        return odd ? [fx(true), FL + 0.6] : [fx(false), -0.6];
      }
      const k = (seg - 1) / 2, far = k % 2 === 0;
      if (goingDown) {
        if (far) return x < 0.6 ? [fx(true), FL + 0.55] : [fx(true), FL - 0.6];
        return x > -0.6 ? [fx(false), -0.55] : [fx(false), 0.6];
      }
      // climbing: back onto flight k
      if (k < 0) return [x, z];
      if (far) return x > -0.6 ? [fx(false), FL + 0.55] : [fx(false), FL - 0.6];
      return x < 0.6 ? [fx(true), -0.55] : [fx(true), 0.6];
    };
    for (let f = 0; f < 9000; f++) {
      const dt = 1 / 60;
      let fwd = 1, strafe = 0, speed = 2.1;
      const st = sw.state;
      if (phase === 'down' && st.deepestSeg >= 12) phase = 'up';
      if (phase === 'up' && st.face && st.face.kind === 'behind') { armedT += dt; if (armedT > 0.6) phase = 'turn'; }
      if (phase === 'up' && st.seg <= 0) phase = 'turn';
      if (phase === 'turn') { fwd = 0; turnT += dt; if (turnT > 1.4) phase = 'final'; }
      const [tx, tz] = target();
      const dx = tx - x, dz = tz - z;
      if (phase === 'turn') yaw += Math.PI / 60 / 0.9 * (turnT < 0.9 ? 1 : 0);
      else if (Math.hypot(dx, dz) > 1e-3) yaw = Math.atan2(-dx, -dz);
      if (phase === 'final' && seg === LAST && Math.hypot(dx, dz) < 0.1) { fwd = 0; holdT += dt; }
      if (phase === 'final' && holdT > 0.5) break;
      // a little variety: crouch speed now and then, a sprint, a strafe wobble
      if (f % 400 > 350) speed = 1.2; else if (f % 700 > 640) speed = 3.4;
      if (f % 300 > 280) strafe = 0.3;
      pitch = phase === 'up' ? 0.1 : -0.25;
      // the first two Faces: stop and stare back until it jerks and goes
      if (st.face && st.face.kind === 'below' && st.faceCount <= 2 && phase !== 'turn') {
        const fp = sw.face.position, ddx = fp.x - cam.position.x, ddy = fp.y - cam.position.y, ddz = fp.z - cam.position.z;
        if (Math.hypot(ddx, ddy, ddz) < 8.5) { fwd = 0; strafe = 0; yaw = Math.atan2(-ddx, -ddz); pitch = Math.atan2(ddy, Math.hypot(ddx, ddz)); }
      }
      const inp = { fwd, strafe, speed, yaw, pitch, camera: cam };
      const cp = cam.position.clone(); const cd = new THREE.Vector3(); cam.getWorldDirection(cd);
      const o = sw.update(dt, inp);
      x = o.x; z = o.z; y = o.y; seg = o.seg;
      cam.position.set(o.x, o.y, o.z); cam.rotation.set(pitch, yaw, 0);
      const S = sw.state;
      const rec = { i: [dt, fwd, strafe, speed, yaw, pitch, cp.x, cp.y, cp.z, cd.x, cd.y, cd.z], o: [o.x, o.y, o.z, o.seg, o.nearExit ? 1 : 0, S.deepestSeg, S.stepPhase, S.time] };
      if (o.events.length) rec.e = o.events.map((e) => [e.type, e.dist ?? null, e.n ?? null, e.reason ?? null]);
      if (S.face || sw.face.visible) rec.f = [S.face ? S.face.kind : null, sw.face.visible ? 1 : 0, ...v3(sw.face.position), ...q4(sw.face.quaternion), S.face ? (S.face.stare ?? 0) : 0, S.face ? (S.face.jerks ?? 0) : 0];
      if (f % 20 === 0) rec.c = [0, 1, 2, 3].map((i) => sw.group.children[i].visible ? sw.group.children[i].position.y : null).concat([sw.group.children[4].visible ? 1 : 0]);
      steps.push(rec);
    }
    out.walk = { seed: 99, flights: 10, steps, draws: rng2.n };
  }

  // ---- with a painted face geometry
  {
    const g = new THREE.BoxGeometry(0.2, 0.3, 0.2);
    g.setAttribute('color', new THREE.Float32BufferAttribute(new Array(g.attributes.position.count * 3).fill(0.5), 3));
    const s2 = SW.createStairwell(THREE, { canvasTex: stubCanvasTex, rng: mulberry32(5), faceGeometry: g });
    const g2 = new THREE.BoxGeometry(0.2, 0.3, 0.2);
    const s3 = SW.createStairwell(THREE, { canvasTex: stubCanvasTex, rng: mulberry32(5), faceGeometry: g2 });
    out.faceGeometry = { painted: matInfo(s2.face.material), plain: matInfo(s3.face.material) };
  }
  return out;
}

// =================================================================== loop ===
function loopFixture() {
  const out = {};
  const { LOOP, FURN } = LP;
  out.consts = { LOOP, LOOP_PITCH: LP.LOOP_PITCH, LOOP_STRIP: LP.LOOP_STRIP, LAP_MIN: LP.LAP_MIN, FURN };
  // ---- pure functions
  out.lapLook = [];
  for (let L = -3; L <= 14; L++) { const k = LP.lapLook(L); out.lapLook.push([L, k.lap, ...k.tint, ...k.light, k.bright, k.flicker, +k.turned, +k.red, +k.stains, +k.dirty, +k.figure, +k.bare, +k.pickups, +k.exit]); }
  out.pickupAt = [];
  for (let lap = -1; lap <= 40; lap++) for (let room = 0; room < 5; room++) { const p = LP.pickupAt(lap, room); out.pickupAt.push([lap, room, p === null ? 0 : p === 'almond' ? 1 : 2]); }
  out.loopReverb = [-1, 0, 0.4, 0.5, 1, 2.5, 3, 6, 7, 9.6].map((l) => { const r = LP.loopReverb(l); return [l, r.t60, r.damp, r.wet, r.pre, r.er]; });
  out.loopHumCents = []; for (let i = 0; i <= 64; i++) { const lam = i * 0.125; out.loopHumCents.push([lam, LP.loopHumCents(lam)]); }
  out.loopDrain = []; for (let L = -2; L <= 14; L++) out.loopDrain.push([L, LP.loopDrain(L)]);

  // ---- construction
  const T = () => new THREE.Texture();
  texLog.length = 0;
  const rng = counted(mulberry32(4242));
  const lp = LP.createLoop(THREE, { canvasTex: stubCanvasTex, rng, textures: { wall: T(), wallDirty: T(), carpet: T(), ceiling: T(), panel: T() } });
  const meshes = lp.group.children.map((m) => ({ name: m.name, cap: m.isInstancedMesh ? m.instanceMatrix.count : null, geo: geoSum(m.geometry), mat: matInfo(m.material) }));
  out.build = { draws: rng.n, texs: texLog.slice(), meshes, groupName: lp.group.name, groupVisible: lp.group.visible };
  {
    const bare = LP.createLoop(THREE, {});
    out.buildPlain = bare.group.children.map((m) => ({ name: m.name, mat: matInfo(m.material) }));
    const half = LP.createLoop(THREE, { canvasTex: stubCanvasTex, rng: mulberry32(1), textures: { wall: T() } });
    out.buildHalf = half.group.children.map((m) => ({ name: m.name, mat: matInfo(m.material) }));
  }

  const dumpInst = () => lp.group.children.filter((m) => m.isInstancedMesh).map((m) => {
    const r = { name: m.name, count: m.count, m: [], c: [] };
    for (let i = 0; i < m.count; i++) {
      r.m.push(Array.from(m.instanceMatrix.array.slice(i * 16, i * 16 + 16)));
      r.c.push(Array.from(m.instanceColor.array.slice(i * 3, i * 3 + 3)));
    }
    return r;
  });
  const ghost = () => { const g = lp.group.children[lp.group.children.length - 1]; return [g.visible ? 1 : 0, ...v3(g.position), ...eul(g.rotation)]; };
  const counts = () => { const c = lp.counts(); return Object.keys(c).map((k) => [k, c[k]]); };

  // ---- enter and placements: what is drawn
  lp.enter({ lap: 2 });
  const snaps = [];
  const snap = (label) => snaps.push({ label, st: { lap: lp.state.lap, x: lp.state.x, z: lp.state.z, yaw: lp.state.yaw, entryDoor: lp.state.entryDoor, started: lp.state.started }, lap: lp.lap, room: lp.room, counts: counts(), drawCalls: lp.drawCalls, inst: dumpInst(), ghost: ghost() });
  snap('enter2');
  const places = [[5, 13, 0], [-3, 1, 0], [7, 25.9, 0.1], [0, 2.5, 0], [3, 0.01, -0.4], [12, 14.5, 1.2], [-5, 20, 0], [4, 7.8, 0.3]];
  for (const [lap, z, x] of places) { lp.place(lap, z, x, 0.3); snap(`place ${lap} ${z} ${x}`); }
  out.snaps = snaps;

  // ---- collision and sight probes
  const probes = { blocked: [], clear: [], lookLap: [] };
  for (const lap of [0, 2, 4, 5, 6, 8]) {
    lp.place(lap, 13, 0, 0);
    for (let ix = 0; ix <= 26; ix++) for (let iz = 0; iz <= 66; iz++) {
      const x = -2.6 + ix * 0.2, z = -0.3 + iz * 0.4;
      probes.blocked.push(lp.blocked(x, z) ? 1 : (lp.blocked(x, z, 0.1) ? 2 : 0));
    }
    for (let o = -12; o <= 12; o++) probes.lookLap.push(lp.lookLapOf(o));
  }
  const pr = mulberry32(77);
  for (let i = 0; i < 600; i++) {
    const ax = (pr() - 0.5) * 4.8, az = pr() * 26, bx = (pr() - 0.5) * 4.8, bz = i % 7 === 0 ? az : pr() * 26;
    probes.clear.push([ax, az, bx, bz, lp.clearLine(ax, az, bx, bz) ? 1 : 0]);
  }
  probes.clear.push([0, 5.2, 0.3, 5.2, lp.clearLine(0, 5.2, 0.3, 5.2) ? 1 : 0]);
  out.probes = probes;

  // ---- the scripted walk (dt 1/30): inputs, and what came out
  {
    const rng3 = counted(mulberry32(31337));
    const L = LP.createLoop(THREE, { canvasTex: stubCanvasTex, rng: rng3, textures: { wall: T(), wallDirty: T(), carpet: T(), ceiling: T(), panel: T() } });
    const P = LP.LOOP_PITCH, W = LOOP.W;
    const script = [];
    const recs = [];
    let fr = 0;
    const record = (o) => {
      const S = L.state, Fg = L.figure;
      const rec = { o: [o.x, o.z, o.lap, o.room, o.nearExit ? 1 : 0, o.touch, o.lightCount, S.lap, S.seams, S.entryDoor ? 1 : 0] };
      if (o.events.length) rec.e = o.events.map((e) => [e.type, e.dir ?? null, e.lap ?? null, e.kind ?? null, e.room ?? null, e.dist ?? null, e.moved ?? null]);
      if (fr % 5 === 0 || o.events.length) {
        const e = o.env;
        rec.env = [e.fogR, e.fogG, e.fogB, e.fog, e.amb, e.hemi, e.skyR, e.skyG, e.skyB, e.groundR, e.groundG, e.groundB, e.hum, e.humCents, e.reverb, e.drain, e.glow, e.lam];
        rec.l = o.lights.map((q) => [q.x, q.y, q.z, q.i, q.r, q.g, q.b]);
        rec.fig = [Fg.active ? 1 : 0, Fg.lap, Fg.gz, Fg.x, Fg.yaw, Fg.seen ? 1 : 0, Fg.unseenT, Fg.moved, Fg.inst, Fg.sightings];
        rec.st = [S.time, S.entryT, S.entryUnseen, ...Array.from(S.taken).sort((a, b) => a - b)];
      }
      if (fr % 50 === 0 || o.events.length) { const c = L.counts(); rec.c = Object.keys(c).map((k) => c[k]); rec.dc = L.drawCalls; }
      recs.push(rec);
      fr++;
    };
    const u = (dt, fwd, strafe, speed, yaw) => { script.push({ u: [dt, fwd, strafe, speed, yaw] }); record(L.update(dt, { fwd, strafe, speed, yaw })); };
    const place = (lap, z, x, yaw) => { script.push({ place: [lap, z, x, yaw] }); L.place(lap, z, x, yaw); recs.push({ placed: [L.state.lap, L.state.z, L.state.x] }); };
    script.push({ enter: 2 }); L.enter({ lap: 2 }); recs.push({ entered: [L.state.lap, L.state.x, L.state.z, L.state.yaw] });
    const dt = 1 / 30, WALK = 3.6;
    const yawTo = (dx, dz) => Math.atan2(-dx, -dz);
    const goTo = (tx, tz, maxF = 120, speed = WALK) => {
      for (let i = 0; i < maxF; i++) {
        const dx = tx - L.state.x, dz = tz - L.state.z;
        if (Math.hypot(dx, dz) < 0.12) return true;
        u(dt, 1, 0, Math.min(speed, Math.hypot(dx, dz) / dt), yawTo(dx, dz));
      }
      return false;
    };
    const stand = (n, yaw) => { for (let i = 0; i < n; i++) u(dt, 0, 0, WALK, yaw); };
    u(0, 0, 0, WALK, Math.PI / 2);           // the glue's updateLoop(0) on entry
    stand(10, Math.PI / 2);
    // off the side door to the axis, then forward through the laps
    goTo(0, L.state.z - 0.4);
    let lastG = -1e9, lookedBack = new Set();
    for (let step = 0; step < 3000; step++) {
      const S = L.state, lapR = L.lap, room = L.room, zc = room * P + P / 2;
      if (lapR >= 7 && room >= 1) break;
      const g = S.lap * 5 + room;
      if (g !== lastG) {
        lastG = g;
        const kind = LP.pickupAt(lapR, room);
        const key = lapR * 8 + room;
        if (kind && !S.taken.has(key) && S.z < zc - 1.9) {
          const bare = LP.lapLook(lapR).bare;
          if (kind === 'almond') { if (bare) { goTo(-1.3, zc - 0.2); goTo(-0.2, zc + 0.6); } else { goTo(-1.35, zc - 1.0); goTo(-0.3, zc - 1.2); goTo(0, zc + 0.6); } }
          else { if (bare) { goTo(1.35, zc - 1.3); goTo(0.2, zc + 0.6); } else { goTo(1.5, zc - 1.75); goTo(0, zc + 0.6); } }
          continue;
        }
        if (lapR >= 5 && (room === 3 || room === 4 || room === 0) && !lookedBack.has(g)) {
          lookedBack.add(g);
          stand(8, Math.PI);
          for (let i = 0; i < 15; i++) u(dt, 0, 0, WALK, Math.PI + (i + 1) * (Math.PI / 15));
          stand(20, 0);
          for (let i = 0; i < 15; i++) u(dt, 0, 0, WALK, (i + 1) * (Math.PI / 15));
        }
      }
      const zBefore = S.z;
      u(dt, 1, 0, WALK, yawTo(Math.max(-0.3, Math.min(0.3, -S.x)), 1));
      if (Math.abs(L.state.z - zBefore) < 1e-9 && L.state.lap === S.lap) {
        // blocked (by the figure, most likely): step round it
        u(dt, 0, 1, WALK, Math.PI); u(dt, 0, 1, WALK, Math.PI);
      }
    }
    // back from lap 1 to lap 0 and out of the exit door
    place(1, 24.0, 0, 0);
    for (let i = 0; i < 2000; i++) {
      const S = L.state;
      if (L.lap === 0 && L.room === 0) break;
      u(dt, 1, 0, WALK, yawTo(Math.max(-0.3, Math.min(0.3, -S.x)), -1));
    }
    const ex = L.exitDoorAt();
    goTo(ex.x + 0.3, ex.z, 200);
    stand(20, Math.PI / 2);
    // through the seam backwards, laps below 0, down past LAP_MIN
    goTo(0, ex.z - 0.5, 100);
    for (let i = 0; i < 900; i++) u(dt, 1, 0, WALK, yawTo(Math.max(-0.3, Math.min(0.3, -L.state.x)), -1));
    // the figure: placed in lap 5, looking around, away, back
    place(5, 13 - 3.2, 0, Math.PI);
    stand(30, Math.PI);
    stand(60, 0);
    stand(30, Math.PI);
    stand(60, 0);
    for (let i = 0; i < 90; i++) u(dt, 1, 0, 1.0, Math.PI);
    stand(60, 0.3);
    for (let i = 0; i < 60; i++) u(dt, 1, 0, 1.0, Math.PI - 0.2);
    stand(40, -0.8);
    // walls, furniture and sliding: strafing and walking into things
    place(4, 2 * P + 0.5, 0, Math.PI);
    for (let i = 0; i < 60; i++) u(dt, 0, -1, WALK, Math.PI);      // strafe towards the desk side
    for (let i = 0; i < 60; i++) u(dt, 1, 0, WALK, Math.PI / 2 + 0.3);
    for (let i = 0; i < 60; i++) u(dt, 1, 0.5, WALK, -Math.PI / 2);
    for (let i = 0; i < 60; i++) u(dt, 1, 0, WALK, Math.PI - 0.9);
    for (let i = 0; i < 40; i++) u(dt, -1, 0, WALK, Math.PI);
    // big steps and odd frames
    u(0.5, 1, 0, WALK, Math.PI);
    u(0, 1, 0, WALK, Math.PI);
    place(6, 25.95, 0, Math.PI);
    for (let i = 0; i < 30; i++) u(dt, 1, 0, WALK, Math.PI);
    place(9, 13, 0, Math.PI);
    for (let i = 0; i < 200; i++) u(dt, 1, 0, WALK, Math.PI + 0.05 * Math.sin(i * 0.1));
    out.walk = { seed: 31337, script, recs, draws: rng3.n, frames: fr };
  }
  return out;
}

// ================================================================== store ===
function storeFixture() {
  const out = {};
  out.hours = ST.STORE_HOURS;
  const ts = [];
  for (let t = 0; t <= 1200; t += 0.7) ts.push(t);
  ts.push(90.99, 91, 104.999, 105, 152.99, 153, 279, 293, 340.9, 341, 1e4, 12345.678);
  out.phase = ts.map((t) => { const p = ST.storePhase(t); return [t, p.phase, p.left, p.into, p.cycle, p.len, ST.storeClock(p)]; });
  const H2 = { firstDay: 10, day: 20, warn: 3, night: 5 };
  out.phase2 = [];
  for (let t = 0; t <= 120; t += 0.37) { const p = ST.storePhase(t, H2); out.phase2.push([t, p.phase, p.left, p.into, p.cycle, p.len, ST.storeClock(p)]); }
  const nr = mulberry32(2024);
  out.names = []; for (let i = 0; i < 300; i++) out.names.push(ST.productName(nr));
  const sr = mulberry32(7);
  out.sets = [];
  for (let rep = 0; rep < 5; rep++) for (const theme of Object.keys(ST.ROOM_SETS)) {
    const list = ST.ROOM_SETS[theme](sr).map(([name, x, z, iy = 0, yOff = 0, a1, a2]) => [name, x, z, iy, yOff, a1 ?? null, a2 ?? null]);
    out.sets.push([theme, list]);
  }
  out.weights = Object.entries(ST.ROOM_SET_WEIGHTS);
  out.signs = Object.entries(ST.DEPARTMENT_SIGNS);

  // ---- the kit
  const kr = counted(mulberry32(555));
  const matLog = [];
  const mats = new Map();
  const mat = (hex) => { let m = mats.get(hex); if (!m) { m = new THREE.MeshLambertMaterial({ color: hex }); m.userData.hex = hex; mats.set(hex, m); matLog.push(hex); } return m; };
  const tagLog = [];
  const tag = (text) => { tagLog.push([text.name, text.what, text.price]); const m = new THREE.MeshBasicMaterial(); m.userData.tag = tagLog.length - 1; return m; };
  const PG = (name, geo) => { geo.userData.pg = name; return geo; };
  const kit = ST.storeKit({ THREE, PG, mat, rng: kr, tag });
  const dump = (G) => ({
    hide: G.userData.hide ? [G.userData.hide.eye, G.userData.hide.r, G.userData.hide.kind, G.userData.hide.front ?? null] : null,
    c: G.children.map((o) => [o.geometry.userData.pg ?? null, geoParams(o.geometry), o.material.userData.hex ?? (o.material.userData.tag !== undefined ? -1 - o.material.userData.tag : null), ...v3(o.position), ...eul(o.rotation)]),
  });
  const names = ['bed', 'nightstand', 'tableLamp', 'wardrobe', 'rug', 'sofa', 'armchair', 'coffeeTable', 'floorLamp', 'bookcase', 'tvBench', 'kitchenRun', 'fridge', 'diningTable', 'chair', 'desk', 'officeChair', 'bunkBed', 'toyBox', 'rack', 'trolley'];
  out.kit = [];
  for (let rep = 0; rep < 3; rep++) for (const n of names) out.kit.push([n, null, null, dump(kit[n]())]);
  for (const [n, a1, a2] of [['rug', 2.6, 1.9], ['rug', 2.0, null], ['kitchenRun', 3.6, null], ['kitchenRun', 2.0, null], ['kitchenRun', 1.0, null]]) out.kit.push([n, a1, a2, dump(kit[n](a1 ?? undefined, a2 ?? undefined))]);
  for (let i = 0; i < 4; i++) { const t = { name: ST.productName(kr), what: 'Bookcase', price: `${i * 10 + 9}.–` }; out.kit.push(['priceTag', t.name, t.price, dump(kit.priceTag(t))]); }
  out.kitDraws = kr.n; out.matLog = matLog; out.tagLog = tagLog;
  return out;
}

const write = (name, obj) => { const p = path.join(OUT, name); fs.writeFileSync(p, JSON.stringify(obj)); console.log(name, (fs.statSync(p).size / 1024).toFixed(0), 'KB'); };
write('stairwell.json', stairwellFixture());
write('loop.json', loopFixture());
write('store.json', storeFixture());
