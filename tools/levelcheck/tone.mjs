// node tools/levelcheck/tone.mjs a.png b.png — compare images at 1/16 scale (40×25 cells of the
// average colour) so per-pixel noise (film grain, random wallpaper speckle) cancels out and only
// systematic differences in brightness / colour / geometry remain. Prints the signed mean per
// channel (b − a), the mean |Δ| and the worst cell.
import { createRequire } from 'module';
import fs from 'fs';
const require = createRequire('/Users/nohwan/Documents/Personal_Projects/OpenCarole/package.json');
const { chromium } = require('playwright');
const [a, b] = process.argv.slice(2);
const br = await chromium.launch(); const p = await br.newPage();
const data = (f) => 'data:image/png;base64,' + fs.readFileSync(f).toString('base64');
const r = await p.evaluate(async ([da, db]) => {
  const load = (src) => new Promise((res) => { const i = new Image(); i.onload = () => res(i); i.src = src; });
  const [ia, ib] = await Promise.all([load(da), load(db)]);
  const W = 40, H = 25;
  const avg = (img) => {
    // box-average: draw at full size, then sum 32×32 blocks
    const c = document.createElement('canvas'); c.width = img.width; c.height = img.height;
    const x = c.getContext('2d'); x.drawImage(img, 0, 0);
    const d = x.getImageData(0, 0, img.width, img.height).data;
    const bw = img.width / W, bh = img.height / H, out = [];
    for (let j = 0; j < H; j++) for (let i = 0; i < W; i++) {
      let s = [0, 0, 0], n = 0;
      for (let y = Math.floor(j * bh); y < Math.floor((j + 1) * bh); y++) for (let xx = Math.floor(i * bw); xx < Math.floor((i + 1) * bw); xx++) {
        const k = (y * img.width + xx) * 4; s[0] += d[k]; s[1] += d[k + 1]; s[2] += d[k + 2]; n++;
      }
      out.push(s.map((v) => v / n));
    }
    return out;
  };
  const A = avg(ia), B = avg(ib);
  const sd = [0, 0, 0]; let mad = 0, worst = 0, wi = 0;
  A.forEach((ca, k) => { const cb = B[k]; let m = 0; for (let c = 0; c < 3; c++) { const d = cb[c] - ca[c]; sd[c] += d; m = Math.max(m, Math.abs(d)); mad += Math.abs(d); } if (m > worst) { worst = m; wi = k; } });
  const n = A.length;
  const map = [];
  for (let j = 0; j < H; j++) { let row = ''; for (let i = 0; i < W; i++) { const k = j * W + i; const d = (B[k][0] + B[k][1] + B[k][2] - A[k][0] - A[k][1] - A[k][2]) / 3; row += d > 12 ? '#' : d > 6 ? '+' : d > 2 ? '.' : d < -12 ? '@' : d < -6 ? '=' : d < -2 ? '-' : ' '; } map.push(row); }
  return { signed: sd.map((v) => v / n), mad: mad / (n * 3), worst, cell: [wi % W, Math.floor(wi / W)], a: A[wi].map(Math.round), b: B[wi].map(Math.round), map };
}, [data(a), data(b)]);
if (process.argv.includes('--map')) console.log('native − js luminance per cell (# > +12, + > +6, . > +2, - < -2, = < -6, @ < -12):\n' + r.map.map((l) => '|' + l + '|').join('\n'));
console.log(`1/32-cell tone: signed Δ r ${r.signed[0].toFixed(2)} g ${r.signed[1].toFixed(2)} b ${r.signed[2].toFixed(2)}  mean|Δ| ${r.mad.toFixed(2)}  worst ${r.worst.toFixed(1)} at cell ${r.cell} (${r.a} vs ${r.b})`);
await br.close();
