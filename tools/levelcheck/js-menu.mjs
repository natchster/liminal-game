// node native/tools/levelcheck/js-menu.mjs out.png [width height dpr] — the browser build's start menu
import { createRequire } from 'module';
import { fileURLToPath } from 'url';
import http from 'http';
import fs from 'fs';
import path from 'path';
const HERE = path.dirname(fileURLToPath(import.meta.url));
const ROOT = path.resolve(HERE, '..', '..', '..');
const require = createRequire(path.join(ROOT, 'package.json'));
const { chromium } = require('playwright');
const [out, w = 1280, h = 800, dpr = 1] = process.argv.slice(2);
const MIME = { '.html': 'text/html', '.js': 'text/javascript', '.css': 'text/css', '.json': 'application/json' };
const server = http.createServer((req, res) => {
  const p = path.join(ROOT, decodeURIComponent(new URL(req.url, 'http://x').pathname));
  if (!p.startsWith(ROOT) || !fs.existsSync(p) || fs.statSync(p).isDirectory()) { res.writeHead(404); res.end(); return; }
  res.writeHead(200, { 'content-type': MIME[path.extname(p)] || 'application/octet-stream' }); fs.createReadStream(p).pipe(res);
});
await new Promise((r) => server.listen(0, '127.0.0.1', r));
const br = await chromium.launch({ headless: true, args: ['--enable-gpu', '--use-angle=metal'] });
const page = await br.newPage({ viewport: { width: +w, height: +h }, deviceScaleFactor: +dpr });
await page.goto(`http://127.0.0.1:${server.address().port}/backrooms.html#seed=4242`);
await page.waitForFunction(() => window.__game && window.__game.perf, null, { timeout: 120000 });
await page.waitForTimeout(1500);
await page.screenshot({ path: out });
await br.close(); server.close();
