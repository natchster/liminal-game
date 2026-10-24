#!/usr/bin/env node
// Capture what the browser build makes of (run seed, level) pairs, for
// comparison with the native build (`backrooms --dump-level`).
//
//   node native/tools/levelcheck/js-capture.mjs --seed 4242 --levels 1,2,3 [--out dir] [--shots] [--dpr 2]
//
// Serves the repository read-only on a local port, opens backrooms.html in
// headless Chromium (Playwright from the repo's node_modules), and for each
// level: turns the draw trace on, loads the level, and writes
// <out>/js_<seed>_L<level>.json with the per-phase / per-placer draw marks,
// the grid, the exit and key props, counts, entities, the spawn pose and every
// placed object's transform and bounds. With --shots it also freezes the sim
// and saves the frame after one fixed 1/60 s step at the spawn pose (js_<seed>_L<level>.png).
import { createRequire } from 'module';
import { fileURLToPath } from 'url';
import http from 'http';
import fs from 'fs';
import path from 'path';

const HERE = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(HERE, '..', '..', '..');
const require = createRequire(path.join(ROOT, 'package.json'));
const { chromium } = require('playwright');

const args = process.argv.slice(2);
const opt = (k, d) => { const i = args.indexOf('--' + k); return i >= 0 ? args[i + 1] : d; };
const seed = +opt('seed', '4242');
const levels = opt('levels', '1').split(',').map(Number);
const hashExtra = opt('hash', '');
const out = path.resolve(opt('out', path.join(ROOT, 'native', 'target', 'levelcheck')));
const shots = args.includes('--shots');
const width = +opt('width', '1280'), height = +opt('height', '800'), dpr = +opt('dpr', '1');
fs.mkdirSync(out, { recursive: true });

const MIME = { '.html': 'text/html', '.js': 'text/javascript', '.mjs': 'text/javascript', '.css': 'text/css', '.json': 'application/json', '.png': 'image/png', '.bin': 'application/octet-stream', '.wasm': 'application/wasm' };
const server = http.createServer((req, res) => {
  const p = path.join(ROOT, decodeURIComponent(new URL(req.url, 'http://x').pathname));
  if (!p.startsWith(ROOT) || !fs.existsSync(p) || fs.statSync(p).isDirectory()) { res.writeHead(404); res.end(); return; }
  res.writeHead(200, { 'content-type': MIME[path.extname(p)] || 'application/octet-stream' });
  fs.createReadStream(p).pipe(res);
});
await new Promise((r) => server.listen(0, '127.0.0.1', r));
const port = server.address().port;

const browser = await chromium.launch({ headless: true, args: ['--enable-gpu', '--ignore-gpu-blocklist', '--use-angle=metal'] });
const page = await browser.newPage({ viewport: { width, height }, deviceScaleFactor: dpr });
const errors = [];
page.on('pageerror', (e) => errors.push(String(e.stack || e)));
const hash = `seed=${seed}` + (hashExtra ? '&' + hashExtra : '');
await page.goto(`http://127.0.0.1:${port}/backrooms.html#${hash}`);
await page.waitForFunction(() => window.__game && window.__game.perf, null, { timeout: 120000 });
await page.evaluate(() => window.__game.begin());

for (const L of levels) {
  const r = await page.evaluate((L) => {
    const g = window.__game;
    g.randTrace(true);
    g.loadLevel(L);
    const marks = g.randMarks;
    g.randTrace(false);
    const r6 = (x) => (typeof x === 'number' ? +x.toFixed(6) : x);
    const v = (a) => a.map(r6);
    return {
      seed: undefined, level: g.level, marks,
      grid: g.grid, arch: g.arch, scp: g.scp.key, weather: g.status.weather,
      exit: g.exit, props: g.props, counts: g.counts,
      entities: g.entities,
      player: { x: g.player.pos.x, y: g.player.pos.y, z: g.player.pos.z, yaw: g.player.yaw, pitch: g.player.pitch },
      propStats: g.propStats,
      objects: g.propList().map((o) => ({ k: o.k, placer: o.placer, pos: v(o.pos), rot: v(o.rot), min: v(o.min), max: v(o.max), meshes: o.meshes, interact: o.interact })),
      barricades: g.barricades,
      buildProfile: g.buildProfile,
    };
  }, L);
  r.seed = seed;
  const base = path.join(out, `js_${seed}_L${L}`);
  fs.writeFileSync(base + '.json', JSON.stringify(r));
  let line = `level ${L}: ${r.marks.length} marks, ${r.marks.length ? r.marks[r.marks.length - 1][1] : 0} draws, ${r.objects.length} objects, scp=${r.scp}, weather=${r.weather}`;
  if (shots) {
    // rebuild, one fixed 1/60 s step (updatePlayer puts the camera at the spawn pose), freeze, draw
    await page.evaluate((L) => { const g = window.__game; g.loadLevel(L); g.step(1, 1 / 60); g.freeze(true); g.renderOnly(); }, L);
    await page.screenshot({ path: base + '.png' });
    await page.evaluate(() => window.__game.freeze(false));
    line += ' (+png)';
  }
  console.log(line);
}
if (errors.length) console.log('page errors:\n' + errors.slice(0, 5).join('\n'));
await browser.close();
server.close();
