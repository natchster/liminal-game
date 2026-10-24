#!/usr/bin/env node
// Differential test: ported Rust placers (native/placers/pNN) vs. the original
// JS placers in backrooms.js. See README.md.
//
//   node tools/placer-diff/run.mjs [crates|all] [--seeds N] [--jobs K] [--only placerA,placerB]
//                                  [--sizes 21,29,37] [--details K] [--no-uses] [--no-build] [--out DIR]
import { execFileSync, spawn } from 'child_process';
import fs from 'fs';
import os from 'os';
import path from 'path';
import vm from 'vm';
import { fileURLToPath, pathToFileURL } from 'url';
import { Bundle } from './jsbundle.mjs';
import { makeLevels, writeLevels } from './levels.mjs';
import { uncovered } from './coverage.mjs';
import * as C from './canon.mjs';

const HERE = path.dirname(fileURLToPath(import.meta.url));
const WS = path.resolve(HERE, '../..');                       // the cargo workspace (native/)
const REPO = process.env.BACKROOMS_REPO || '/Users/nohwan/Documents/Personal_Projects/OpenCarole';
const BACKROOMS_JS = process.env.BACKROOMS_JS || path.join(REPO, 'backrooms.js');
const THREE_JS = process.env.THREE_JS || path.join(REPO, 'vendor/three.module.min.js');
const TARGET = process.env.CARGO_TARGET_DIR || path.join(WS, 'target');
const BIN = process.env.PLACER_DIFF_BIN || path.join(TARGET, 'release/placer-diff');

// ------------------------------------------------------------------ args
const argv = process.argv.slice(2);
const opt = { crates: [], seeds: 12, jobs: Math.max(1, Math.min(8, os.cpus().length - 2)), only: null, sizes: null, details: 3, uses: true, build: true,
  out: path.join(os.tmpdir(), 'placer-diff'), worker: false, quiet: false, levels: null, realLevels: true };
for (let i = 0; i < argv.length; i++) {
  const a = argv[i];
  const v = () => argv[++i];
  if (a === '--seeds') opt.seeds = +v();
  else if (a === '--jobs') opt.jobs = +v();
  else if (a === '--only') opt.only = v();
  else if (a === '--sizes') opt.sizes = v();
  else if (a === '--details') opt.details = +v();
  else if (a === '--no-uses') opt.uses = false;
  else if (a === '--no-build') opt.build = false;
  else if (a === '--out') opt.out = v();
  else if (a === '--worker') opt.worker = true;
  else if (a === '--levels') opt.levels = v();
  else if (a === '--synthetic-only') opt.realLevels = false;
  else if (a === '--coverage') opt.coverage = true;
  else if (a === '--strict') opt.strict = true;
  else if (a === '--quiet') opt.quiet = true;
  else if (a === '--dump') opt.dump = v().split(',').map(Number);
  else opt.crates.push(...a.split(','));
}

function rustArgs(crate, extra = []) {
  const r = [crate, '--seeds', String(opt.seeds)];
  if (opt.sizes) r.push('--sizes', opt.sizes);
  if (opt.only) r.push('--only', opt.only);
  if (!opt.uses) r.push('--no-uses');
  if (opt.levels) r.push('--levels', opt.levels);
  return r.concat(extra);
}
function runRust(crate, extra) {
  return execFileSync(BIN, rustArgs(crate, extra), { maxBuffer: 1 << 30, encoding: 'utf8', stdio: ['ignore', 'pipe', 'pipe'] });
}

// ------------------------------------------------------------------ master
if (!opt.worker) {
  if (opt.build) {
    console.error('building placer-diff (release)...');
    execFileSync('cargo', ['build', '--release', '-p', 'placers_difftest'], { cwd: WS, stdio: ['ignore', 'inherit', 'inherit'], env: { ...process.env, CARGO_TARGET_DIR: TARGET } });
  }
  let crates = opt.crates;
  if (!crates.length || crates.includes('all')) crates = execFileSync(BIN, ['--list'], { encoding: 'utf8' }).trim().split('\n').map((l) => l.split('\t')[0]);
  fs.mkdirSync(opt.out, { recursive: true });
  if (!opt.levels && opt.realLevels) { opt.levels = path.join(opt.out, 'levels.txt'); writeLevels(opt.levels, await makeLevels()); }
  if (opt.dump) {
    // print both texts of the given cases of one crate (same --seeds/--levels/--only as the run that reported them)
    const args = [fileURLToPath(import.meta.url), '--worker', '--no-build', crates[0], '--seeds', String(opt.seeds), '--out', opt.out, '--dump', opt.dump.join(',')];
    if (opt.only) args.push('--only', opt.only);
    if (opt.sizes) args.push('--sizes', opt.sizes);
    if (!opt.uses) args.push('--no-uses');
    if (opt.levels) args.push('--levels', opt.levels);
    execFileSync(process.execPath, args, { stdio: 'inherit' });
    process.exit(0);
  }
  const results = [];
  let next = 0, running = 0;
  const t0 = Date.now();
  await new Promise((resolve) => {
    const launch = () => {
      while (running < opt.jobs && next < crates.length) {
        const c = crates[next++];
        running++;
        const args = [fileURLToPath(import.meta.url), '--worker', '--no-build', c, '--seeds', String(opt.seeds), '--details', String(opt.details), '--out', opt.out];
        if (opt.only) args.push('--only', opt.only);
        if (opt.sizes) args.push('--sizes', opt.sizes);
        if (!opt.uses) args.push('--no-uses');
        if (opt.levels) args.push('--levels', opt.levels);
        if (opt.strict) args.push('--strict');
        const env = { ...process.env };
        if (opt.coverage) { env.NODE_V8_COVERAGE = path.join(opt.out, 'cov', c); fs.rmSync(env.NODE_V8_COVERAGE, { recursive: true, force: true }); }
        const p = spawn(process.execPath, ['--max-old-space-size=8192', ...args], { stdio: ['ignore', 'pipe', 'inherit'], env });
        let buf = '';
        p.stdout.on('data', (d) => { buf += d; });
        p.on('close', (code) => {
          running--;
          const line = buf.trim().split('\n').filter((l) => l.startsWith('{')).pop();
          let r;
          try { r = JSON.parse(line); } catch { r = { crate: c, error: `worker exited ${code}` }; }
          results.push(r);
          const bad = r.error ? 'ERROR ' + r.error : `${r.cases} cases, ${r.mismatchCases} mismatching, ${r.placersBad.length} placers bad`;
          console.error(`[${((Date.now() - t0) / 1000).toFixed(0)}s] ${c}: ${bad}`);
          if (next >= crates.length && running === 0) resolve(); else launch();
        });
      }
    };
    launch();
  });
  results.sort((a, b) => (a.crate < b.crate ? -1 : 1));
  console.log('\ncrate  placers    cases    built  badCases  draws≠  build≠  frames≠  uses≠  jsErr  rsPanic  noise  -> placers with mismatches');
  let tot = { cases: 0, bad: 0 };
  for (const r of results) {
    if (r.error) { console.log(`${r.crate}  ERROR ${r.error}`); tot.bad++; continue; }
    const s = r.sum;
    tot.cases += r.cases; tot.bad += r.mismatchCases;
    if (r.unportedFields && r.unportedFields.length) { tot.bad++; console.log(`${r.crate}  !! JS position fields with no Rust counterpart: ${r.unportedFields.join(' ')}`); }
    if (r.missingJs.length || r.evalErrors) { tot.bad++; console.log(`${r.crate}  !! ${r.missingJs.length} placers missing in backrooms.js, ${r.evalErrors} JS declarations failed to evaluate (see ${r.crate}.log)`); }
    const col = (v, w) => String(v).padStart(w);
    console.log(`${r.crate.padEnd(6)} ${col(r.placers, 6)} ${col(r.cases, 8)} ${col(s.built, 8)} ${col(s.bad, 9)} ${col(s.draws, 7)} ${col(s.build, 7)} ${col(s.frames, 8)} ${col(s.uses, 6)} ${col(s.jsErr, 6)} ${col(s.rsPanic, 8)} ${col(s.noise, 6)}  ${r.placersBad.join(' ')}`);
  }
  console.log(`\nTOTAL ${tot.cases} cases, ${tot.bad} mismatching. Details: ${opt.out}/<crate>.log`);
  fs.writeFileSync(path.join(opt.out, 'summary.json'), JSON.stringify(results, null, 1));
  if (opt.coverage) {
    // JS lines of each placer that no case executed (the diff could not check them)
    const lst = Object.fromEntries(execFileSync(BIN, ['--list'], { encoding: 'utf8' }).trim().split('\n').map((l) => l.split('\t')).map(([k, v]) => [k, v.split(',')]));
    const rep = [];
    let totLines = 0, totUnc = 0, totPart = 0;
    for (const c of crates) {
      const u = uncovered(path.join(opt.out, 'cov', c), lst[c]);
      for (const [n, v] of Object.entries(u)) {
        totLines += v.lines; totUnc += v.uncovered.length; totPart += v.partial.length;
        if (!v.ran) rep.push(`${c} ${n}: never called`);
        else if (v.uncovered.length || v.partial.length) {
          rep.push(`${c} ${n}: ${v.uncovered.length}/${v.lines} lines never ran, ${v.partial.length} partly`);
          for (const [l, t] of v.uncovered) rep.push(`     ${l}: ${t}`);
          for (const [l, t, f] of v.partial) rep.push(`     ${l}: (never ran: ${f})  ${t}`);
        }
      }
    }
    fs.writeFileSync(path.join(opt.out, 'coverage.txt'), rep.join('\n') + '\n');
    console.log(`JS placer line coverage: ${totLines - totUnc}/${totLines} lines ran (${totPart} lines with a branch that never ran). Report: ${opt.out}/coverage.txt`);
  }
  process.exit(tot.bad ? 1 : 0);
}

// ------------------------------------------------------------------ worker
const crate = opt.crates[0];
const logLines = [];
const log = (s) => { logLines.push(s); };
const rsText = runRust(crate);
const rsLines = rsText.split('\n');
const header = JSON.parse(rsLines[0].split('\t')[1]);
const cases = [];
const panics = new Map();
for (const l of rsLines) {
  const f = l.split('\t');
  if (f[0] === 'C') cases.push({ idx: +f[1], pi: +f[2], li: +f[3], wi: +f[4], s: +f[5], seed: +f[6], d: [+f[7], +f[9], +f[11]], h: [f[8], f[10], f[12]] });
  else if (f[0] === 'P') { const c = { idx: +f[1], pi: +f[2], li: +f[3], wi: +f[4], s: +f[5], seed: +f[6], panic: JSON.parse(f[7]) }; cases.push(c); panics.set(c.idx, c.panic); }
}
cases.sort((a, b) => a.idx - b.idx);

// ---- the JS world
const THREE = await import(pathToFileURL(THREE_JS).href);
globalThis.THREE = THREE;
const nativeRandom = Math.random;
globalThis.__threeUuidRandom = nativeRandom;       // see tools/patch-three.mjs: UUIDs never draw from the level's stream
globalThis.__native = nativeRandom;
const src = fs.readFileSync(BACKROOMS_JS, 'utf8');
const bundle = new Bundle(src);
// stand-ins (never extracted from backrooms.js)
const PRELUDE = `
var PG = (name, geo, own) => geo;                 // props off on both sides
var canvasTex = function (w, h, draw, repeat = true) {   // no drawing (it would run on the native RNG anyway)
  const t = new THREE.Texture();
  t.colorSpace = THREE.SRGBColorSpace;
  if (repeat) t.wrapS = t.wrapT = THREE.RepeatWrapping;
  t.anisotropy = 4;
  return t;
};
var __lastMsg = '';
var showMsg = function (text, dur = 2) { __lastMsg = String(text); };
// virtual timers (Rust: g.set_timeout / g.run_timeouts on g.now_ms)
var __now = 0, __timeouts = [], __toId = 1;
globalThis.setTimeout = function (fn, ms = 0) { const id = __toId++; __timeouts.push({ id, at: __now + Math.max(0, +ms || 0), fn }); return id; };
globalThis.clearTimeout = function (id) { __timeouts = __timeouts.filter((t) => t.id !== id); };
globalThis.__runTimeouts = function () {
  for (;;) {
    let bi = -1;
    for (let i = 0; i < __timeouts.length; i++) if (__timeouts[i].at <= __now && (bi < 0 || __timeouts[i].at < __timeouts[bi].at)) bi = i;
    if (bi < 0) break;
    const t = __timeouts.splice(bi, 1)[0];
    t.fn();
  }
};
`;
const STUBBED = new Set(['THREE', 'PG', 'canvasTex', 'showMsg']);
const RESET_ARR = ['panels', 'almonds', 'batteryItems', 'floorHoles', 'barricadeTiles', 'papers', 'paperMarkers', 'loreNotes', 'lightSwitches', 'hauntedCandles', 'drawerProps'];
const RESET_NULL = ['poolBounds', 'safeRoom', 'brokenElevator', 'vacuumBot', 'tvData', 'phoneData', 'clockData', 'exitData', 'manData'];
const SHARED_MATS = ['darkMat', 'woodMat', 'metalMat', 'chromeMat', 'rustMat', 'plasticYellowMat', 'plasticOrangeMat', 'plasticPinkMat', 'plasticRedMat', 'plasticWhiteMat', 'cardboardMat', 'paperMat', 'glassMat'];
const roots = [...header.placers, 'wx', 'wz', 'W', 'H', 'tiles', 'world', 'animators', 'interactables', 'levelWeather', 'WEATHER_BIASES', 'player', 'perfT', 'currentSeed',
  'faultyTripCount', 'escalationToastFired', 'mulberry32', 'gameplayR1State', ...header.pp, ...RESET_ARR, ...RESET_NULL, ...SHARED_MATS];
const missingJs = header.placers.filter((n) => !bundle.decl.has(n));
const { indices } = bundle.collect(roots, STUBBED);
const evalErrors = [];
vm.runInThisContext(PRELUDE, { filename: 'prelude.js' });
for (const s of bundle.emit(indices)) {
  try { vm.runInThisContext(s.text, { filename: `backrooms.js:${s.line}` }); }
  catch (e) { evalErrors.push(`${s.names.join(',') || s.text.slice(0, 40)} (line ${s.line}): ${e.message}`); }
}
const declared = (n) => bundle.decl.has(n) && !STUBBED.has(n);
const ppNames = header.pp.filter(declared);
const DRIVER = `
globalThis.__playerInit = Object.fromEntries(Object.entries(player).filter(([k, v]) => typeof v !== 'object' && typeof v !== 'function'));
globalThis.__weathers = ${JSON.stringify(header.weathers)}.map((k) => WEATHER_BIASES.find((w) => w.key === k));
globalThis.__pp = () => [${header.pp.map((n) => (declared(n) ? `[${JSON.stringify(n)}, ${n}]` : `[${JSON.stringify(n)}, null]`)).join(', ')}];
globalThis.__reset = function (lv, wi, seed, li, s) {
  W = lv.w; H = lv.h; tiles = Uint8Array.from(lv.tiles);
  world = new THREE.Group(); animators = []; interactables = [];
  ${ppNames.map((n) => `${n} = [];`).join(' ')}
  // odd seeds: every *Positions list starts with two positions (lib.rs prefill)
  if (s % 2 === 1) {
    const open = [];
    for (let z = 0; z < lv.h; z++) for (let x = 0; x < lv.w; x++) if (lv.tiles[z * lv.w + x] === 0) open.push([x, z]);
    if (open.length) {
      const pre = (j) => [0, 1].map((k) => { const [x, z] = open[(j * 7919 + k * 104729 + s * 31 + li * 17) % open.length]; return { x: (x - lv.w / 2 + 0.5) * CELL, y: 0.25 + 0.5 * k, z: (z - lv.h / 2 + 0.5) * CELL }; });
      ${header.pp.map((n, j) => `${n} = pre(${j});`).join(' ')}
    }
  }
  ${RESET_ARR.filter(declared).map((n) => `${n} = [];`).join(' ')}
  ${RESET_NULL.filter(declared).map((n) => `${n} = null;`).join(' ')}
  faultyTripCount = 0; escalationToastFired = false;
  Object.assign(player, __playerInit); player.pos.set(wx(1), 0, wz(1));
  __lastMsg = ''; perfT = 0; currentSeed = seed; __now = 0; __timeouts = [];
  levelWeather = __weathers[wi];
};
globalThis.__get = () => ({ world, animators, interactables, player, msg: __lastMsg, timeouts: __timeouts.length, r1: gameplayR1State, tiles });
globalThis.__fireTimeouts = () => { __now += 1e7; __runTimeouts(); };
globalThis.__setPerfT = (t) => { perfT = t; };
`;
vm.runInThisContext(DRIVER, { filename: 'driver.js' });

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
let drawN = 0;
function seeded(seed) { const r = mulberry32(seed); Math.random = () => { drawN++; return r(); }; }
function unseed() { Math.random = nativeRandom; }

const levels = header.levels.map((l) => ({ w: l.w, h: l.h, tiles: Array.from(l.tiles, (c) => c.charCodeAt(0) - 48), dist: Int32Array.from(l.dist) }));
const fns = header.placers.map((n) => (bundle.decl.has(n) ? vm.runInThisContext(n) : null));
const text = (lines) => lines.map((l) => l + '\n').join('');

const unportedFields = new Set();   // JS per-entry fields of *Positions lists with no Rust counterpart
function runJsCase(c) {
  const lv = levels[c.li];
  __reset(lv, c.wi, c.seed, c.li, c.s);
  const out = [[], [], []], d = [0, 0, 0];
  let err = null;
  const fn = fns[c.pi];
  drawN = 0; seeded(c.seed);
  try { fn(lv.dist); } catch (e) { err = 'build: ' + (e && e.stack ? e.stack.split('\n').slice(0, 3).join(' | ') : e); }
  finally { unseed(); }
  d[0] = drawN;
  let st = __get();
  out[0].push(`#build anim${st.animators.length} inter${st.interactables.length}`);
  C.sceneS(st.world, out[0]); C.ppS(__pp(), header.ppx, out[0], unportedFields); C.interS(st.interactables, out[0]); C.stateS(st.r1, st.tiles, lv.tiles, out[0]);
  drawN = 0; seeded((c.seed ^ 0xA5A5A5A5) >>> 0);
  try {
    header.frames.forEach(([dt, t], k) => {
      __setPerfT(t);
      for (const a of __get().animators.slice()) a(dt, t);
      if (header.frameDumps.includes(k)) { out[1].push(`#frame${k + 1}`); C.sceneS(__get().world, out[1]); }
    });
  } catch (e) { err = err || 'frames: ' + (e && e.stack ? e.stack.split('\n').slice(0, 3).join(' | ') : e); }
  finally { unseed(); }
  d[1] = drawN;
  if (header.uses) {
    drawN = 0;
    header.useFrames.forEach(([dt, t], round) => {
      seeded((c.seed ^ 0x3C3C3C3C ^ round) >>> 0);
      try {
        st = __get();
        for (const it of st.interactables.slice()) { if (it.enabled && !it.enabled()) continue; it.use(); }
        __setPerfT(t);
        for (const a of __get().animators.slice()) a(dt, t);
      } catch (e) { err = err || 'use: ' + (e && e.stack ? e.stack.split('\n').slice(0, 3).join(' | ') : e); }
      finally { unseed(); }
      st = __get();
      out[2].push(`#use${round + 1}`);
      C.sceneS(st.world, out[2]); C.interS(st.interactables, out[2]); C.playerS(st.player, st.msg, out[2]); C.stateS(st.r1, st.tiles, lv.tiles, out[2]);
    });
    d[2] = drawN;
    out[2].push(`TO ${st.timeouts}`);
    if (st.timeouts) {
      seeded((c.seed ^ 0x77777777) >>> 0);
      try { __fireTimeouts(); } catch (e) { err = err || 'timeouts: ' + (e && e.stack ? e.stack.split('\n').slice(0, 3).join(' | ') : e); }
      finally { unseed(); }
      st = __get();
      out[2].push('#timeouts');
      C.sceneS(st.world, out[2]); C.interS(st.interactables, out[2]); C.playerS(st.player, st.msg, out[2]);
    }
  }
  return { d, t: out.map(text), err, objs: st.world.children.length };
}

// ---- run + compare
const SEC = ['build', 'frames', 'uses'];
const per = header.placers.map((n) => ({ name: n, cases: 0, built: 0, bad: 0, draws: 0, build: 0, frames: 0, uses: 0, jsErr: 0, rsPanic: 0, noise: 0, unchecked: 0, first: [], noiseEx: null }));
const want = new Map();        // idx -> js result kept for detail comparison
for (const c of cases) {
  const p = per[c.pi];
  p.cases++;
  if (!fns[c.pi]) continue;
  const r = runJsCase(c);
  if (r.objs > 0) p.built++;
  if (panics.has(c.idx)) { p.rsPanic++; p.bad++; if (p.first.length < opt.details) p.first.push({ c, kind: 'rsPanic', msg: panics.get(c.idx), jsErr: r.err }); continue; }
  if (r.err) { p.jsErr++; p.bad++; if (p.first.length < opt.details) p.first.push({ c, kind: 'jsErr', msg: r.err }); continue; }
  const bad = [];
  if (r.d[0] !== c.d[0]) bad.push('draws');
  for (let k = 0; k < 3; k++) if (C.hash(r.t[k]) !== c.h[k] || (k > 0 && r.d[k] !== c.d[k])) bad.push(SEC[k]);
  if (!bad.length) continue;
  c.bad = bad;
  c.js = r;
  want.set(c.idx, c);
}
// --dump i,j: print both texts of these cases (debugging)
if (opt.dump) {
  const txt = runRust(crate, ['--detail', opt.dump.join(',')]);
  for (const l of txt.split('\n')) {
    const f = l.split('\t');
    if (f[0] !== 'D') continue;
    const c = cases[+f[1]];
    const r = runJsCase(c);
    for (let k = 0; k < 3; k++) {
      console.log(`===== case ${c.idx} ${header.placers[c.pi]} [${SEC[k]}] draws js=${r.d[k]} rs=${c.d[k]}`);
      console.log('--- js'); console.log(r.t[k]); console.log('--- rs'); console.log(JSON.parse(f[2 + k]));
    }
  }
  process.exit(0);
}
// detail pass: re-run the Rust side with full text for the mismatching cases (capped per placer)
const detailIdx = [];
const capBy = new Map();
for (const c of want.values()) { const k = capBy.get(c.pi) || 0; if (k < 200) { capBy.set(c.pi, k + 1); detailIdx.push(c.idx); } }
const rsDetail = new Map();
if (detailIdx.length) {
  const txt = runRust(crate, ['--detail', detailIdx.join(',')]);
  for (const l of txt.split('\n')) { const f = l.split('\t'); if (f[0] === 'D') rsDetail.set(+f[1], [JSON.parse(f[2]), JSON.parse(f[3]), JSON.parse(f[4])]); }
}
// token-wise comparison tolerant to quantisation noise
const ROT = / r~(-?\d+),~(-?\d+),~(-?\d+),([A-Z]{3}) q~(-?\d+),~(-?\d+),~(-?\d+),~(-?\d+)/;
const TWO_PI = 2 * Math.PI * 1e5;
// the same orientation written differently: Euler angles 2π apart and/or the
// quaternion negated (ulp-level sin/cos differences flip atan2 across ±π)
function sameRotation(ma, mb) {
  if (ma[4] !== mb[4]) return false;
  for (let k = 1; k <= 3; k++) {
    const d = Math.abs(+ma[k] - +mb[k]);
    if (!(d <= 3 || Math.abs(d - TWO_PI) <= 3 || Math.abs(d - 2 * TWO_PI) <= 3)) return false;
  }
  const qa = [5, 6, 7, 8].map((k) => +ma[k]), qb = [5, 6, 7, 8].map((k) => +mb[k]);
  return qa.every((v, i) => Math.abs(v - qb[i]) <= 3) || qa.every((v, i) => Math.abs(v + qb[i]) <= 3);
}
function lineNoise(a, b) {
  if (opt.strict) return false;
  const ra = ROT.exec(a), rb = ROT.exec(b);
  if (ra && rb && ra[0] !== rb[0] && sameRotation(ra, rb)) b = b.replace(ROT, ra[0]);
  const ta = a.split(/([ ,|()"])/), tb = b.split(/([ ,|()"])/);
  if (ta.length !== tb.length) return false;
  for (let i = 0; i < ta.length; i++) {
    if (ta[i] === tb[i]) continue;
    const ma = /^([a-zA-Z]*)~(-?\d+)$/.exec(ta[i]), mb = /^([a-zA-Z]*)~(-?\d+)$/.exec(tb[i]);
    if (ma && mb && ma[1] === mb[1] && Math.abs(+ma[2] - +mb[2]) <= 3) continue;
    const ha = /^(c|e|sp|gc)([0-9a-f]{6})$/.exec(ta[i]), hb = /^(c|e|sp|gc)([0-9a-f]{6})$/.exec(tb[i]);
    if (ha && hb && ha[1] === hb[1]) {
      const x = parseInt(ha[2], 16), y = parseInt(hb[2], 16);
      if ([16, 8, 0].every((s) => Math.abs(((x >> s) & 255) - ((y >> s) & 255)) <= 1)) continue;
    }
    return false;
  }
  return true;
}
function firstDiff(js, rs) {
  const a = js.split('\n'), b = rs.split('\n');
  let noiseOnly = true, first = -1;
  const n = Math.max(a.length, b.length);
  for (let i = 0; i < n; i++) {
    if (a[i] === b[i]) continue;
    if (a[i] !== undefined && b[i] !== undefined && lineNoise(a[i], b[i])) continue;
    noiseOnly = false; first = i; break;
  }
  return { noiseOnly, first, a, b };
}
const lvName = (c) => `level ${c.li} (${levels[c.li].w}x${levels[c.li].h}) ${header.weathers[c.wi]} seed ${c.seed}`;
for (const c of want.values()) {
  const p = per[c.pi];
  const rs = rsDetail.get(c.idx);
  let real = c.bad.filter((k) => k === 'draws');
  const diffs = [];
  if (!rs) { p.unchecked++; p.bad++; for (const k of c.bad) p[k]++; continue; }
  {
    for (let k = 0; k < 3; k++) {
      if (!c.bad.includes(SEC[k])) continue;
      const fd = firstDiff(c.js.t[k], rs[k]);
      const drawsDiffer = k > 0 && c.js.d[k] !== c.d[k];
      if (fd.noiseOnly && !drawsDiffer) { if (!p.noiseEx) p.noiseEx = { c, sec: SEC[k], js: c.js.t[k], rs: rs[k] }; continue; }
      real.push(SEC[k]);
      diffs.push({ sec: SEC[k], fd, drawsDiffer });
    }
  }
  real = [...new Set(real)];
  if (!real.length) { p.noise++; continue; }
  for (const k of real) p[k]++;
  p.bad++;
  if (p.first.length < opt.details) p.first.push({ c, kind: real.join('+'), diffs });
}
// ---- report
let mismatchCases = 0;
const sum = { built: 0, bad: 0, draws: 0, build: 0, frames: 0, uses: 0, jsErr: 0, rsPanic: 0, noise: 0, unchecked: 0 };
const placersBad = [];
log(`# ${crate}: ${header.placers.length} placers, ${cases.length} cases (${levels.length} levels x ${header.weathers.length} weathers x ${header.seeds} seeds)`);
if (missingJs.length) log(`!! placers not found in backrooms.js: ${missingJs.join(' ')}`);
if (evalErrors.length) log(`!! JS declarations that failed to evaluate:\n   ${evalErrors.join('\n   ')}`);
if (unportedFields.size) log(`!! JS *Positions entries carry fields the Rust PropPositions has no list for (data the port cannot keep): ${[...unportedFields].join(' ')}`);
for (const p of per) {
  for (const k of Object.keys(sum)) sum[k] += p[k];
  const nbad = p.first.length ? p.cases : 0;
  const any = p.draws || p.build || p.frames || p.uses || p.jsErr || p.rsPanic;
  log(`${any ? 'FAIL' : 'ok  '} ${p.name.padEnd(36)} cases ${p.cases} built ${p.built} | bad ${p.bad} (draws≠${p.draws} build≠${p.build} frames≠${p.frames} uses≠${p.uses} jsErr ${p.jsErr} rsPanic ${p.rsPanic} unchecked ${p.unchecked}) noise ${p.noise}`);
  if (p.noiseEx) {
    const { c, sec, js, rs } = p.noiseEx;
    const a = js.split('\n'), b = rs.split('\n');
    const i = a.findIndex((l, k) => l !== b[k]);
    log(`  ~~ noise example: case ${c.idx} ${lvName(c)} [${sec}] line ${i + 1}\n     js ${a[i]}\n     rs ${b[i]}`);
  }
  if (any) placersBad.push(p.name);
  for (const f of p.first) {
    const c = f.c;
    log(`  -- case ${c.idx} ${lvName(c)}: ${f.kind}`);
    if (f.kind === 'jsErr' || f.kind === 'rsPanic') { log(`     ${f.msg}${f.jsErr ? ' (js: ' + f.jsErr + ')' : ''}`); continue; }
    log(`     draws js=${c.js.d.join('/')} rs=${c.d.join('/')}`);
    for (const d of f.diffs || []) {
      const { fd } = d;
      log(`     [${d.sec}] first difference at line ${fd.first + 1}${d.drawsDiffer ? ' (draws differ)' : ''}`);
      for (let i = Math.max(0, fd.first - 2); i < Math.min(fd.a.length, fd.first + 1); i++) if (i !== fd.first) log(`        ${fd.a[i]}`);
      log(`     js ${fd.a[fd.first]}`);
      log(`     rs ${fd.b[fd.first]}`);
      log(`        (js ${fd.a.length} lines, rs ${fd.b.length} lines)`);
    }
  }
}
mismatchCases = sum.bad;
fs.mkdirSync(opt.out, { recursive: true });
fs.writeFileSync(path.join(opt.out, `${crate}.log`), logLines.join('\n') + '\n');
if (!opt.quiet) process.stderr.write(logLines.filter((l) => !l.startsWith('ok  ')).join('\n') + '\n');
console.log(JSON.stringify({ crate, placers: header.placers.length, cases: cases.length, mismatchCases, sum, placersBad, missingJs, evalErrors: evalErrors.length, unportedFields: [...unportedFields] }));
