// node tools/levelcheck/crop.mjs out.png x y w h scale a.png b.png ... — side-by-side crops (scaled, nearest)
import { createRequire } from 'module';
import fs from 'fs';
const require = createRequire('/Users/nohwan/Documents/Personal_Projects/OpenCarole/package.json');
const { chromium } = require('playwright');
const [out, x, y, w, h, sc, ...files] = process.argv.slice(2);
const br = await chromium.launch(); const p = await br.newPage();
const url = await p.evaluate(async ([srcs, x, y, w, h, sc]) => {
  const imgs = await Promise.all(srcs.map((s) => new Promise((r) => { const i = new Image(); i.onload = () => r(i); i.src = s; })));
  const c = document.createElement('canvas'); c.width = w * sc * imgs.length + 4 * (imgs.length - 1); c.height = h * sc;
  const g = c.getContext('2d'); g.imageSmoothingEnabled = false; g.fillStyle = '#f0f'; g.fillRect(0, 0, c.width, c.height);
  imgs.forEach((im, k) => g.drawImage(im, x, y, w, h, k * (w * sc + 4), 0, w * sc, h * sc));
  return c.toDataURL('image/png');
}, [files.map((f) => 'data:image/png;base64,' + fs.readFileSync(f).toString('base64')), +x, +y, +w, +h, +sc]);
fs.writeFileSync(out, Buffer.from(url.split(',')[1], 'base64'));
await br.close();
