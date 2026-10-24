// usage: node imgdiff.mjs a.png b.png [diff.png] — per-channel mean/max abs diff, % pixels off by > 8/255
import { chromium } from '/Users/nohwan/Documents/Personal_Projects/OpenCarole/node_modules/playwright/index.mjs';
import fs from 'fs';
const [a, b, out] = process.argv.slice(2);
const br = await chromium.launch();
const p = await br.newPage();
const toData = (f) => 'data:image/png;base64,' + fs.readFileSync(f).toString('base64');
const r = await p.evaluate(async ([da, db, wantOut]) => {
  const load = (src) => new Promise((res) => { const i = new Image(); i.onload = () => res(i); i.src = src; });
  const [ia, ib] = await Promise.all([load(da), load(db)]);
  const w = Math.min(ia.width, ib.width), h = Math.min(ia.height, ib.height);
  const get = (img) => { const c = document.createElement('canvas'); c.width = w; c.height = h; const x = c.getContext('2d'); x.drawImage(img, 0, 0); return x.getImageData(0, 0, w, h).data; };
  const A = get(ia), B = get(ib);
  let sum = 0, max = 0, off = 0; const ch = [0, 0, 0];
  const dc = document.createElement('canvas'); dc.width = w; dc.height = h; const dx = dc.getContext('2d'); const D = dx.createImageData(w, h);
  for (let i = 0; i < w * h; i++) {
    let m = 0;
    for (let k = 0; k < 3; k++) { const d = Math.abs(A[i * 4 + k] - B[i * 4 + k]); sum += d; ch[k] += d; if (d > m) m = d; }
    if (m > max) max = m; if (m > 8) off++;
    D.data[i * 4] = Math.min(255, m * 4); D.data[i * 4 + 1] = Math.min(255, m * 4); D.data[i * 4 + 2] = Math.min(255, m * 4); D.data[i * 4 + 3] = 255;
  }
  dx.putImageData(D, 0, 0);
  return { w, h, mean: sum / (w * h * 3), max, offPct: 100 * off / (w * h), ch: ch.map(v => v / (w * h)), diff: wantOut ? dc.toDataURL('image/png') : null };
}, [toData(a), toData(b), !!out]);
if (out && r.diff) fs.writeFileSync(out, Buffer.from(r.diff.split(',')[1], 'base64'));
console.log(`size ${r.w}x${r.h}  mean|Δ| ${r.mean.toFixed(2)}/255  max ${r.max}  pixels off>8: ${r.offPct.toFixed(2)}%  per-channel ${r.ch.map(v => v.toFixed(2)).join(' ')}`);
await br.close();
