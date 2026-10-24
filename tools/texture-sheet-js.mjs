#!/usr/bin/env node
// Renders the textures section of backrooms.js (canvasTex .. drawStatic) and
// src/surfaces.js in headless Chromium and writes a contact sheet with the
// same layout as game/tests/textures.rs:
//   target/texture-sheet-js.png        (all sections stacked)
//   target/texture-sheet-js/NN.png     (one per section)
//
// usage: node tools/texture-sheet-js.mjs [path/to/OpenCarole]
// The JS sources are only read. Playwright is resolved from the repo's node_modules.

import { createRequire } from 'node:module';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const here = path.dirname(fileURLToPath(import.meta.url));
const ws = path.resolve(here, '..');
const repo = path.resolve(process.argv[2] || '/Users/nohwan/Documents/Personal_Projects/OpenCarole');
const require = createRequire(path.join(repo, 'package.json'));
const { chromium } = require('playwright');

const src = fs.readFileSync(path.join(repo, 'backrooms.js'), 'utf8').split('\n');
const find = (pred, from = 0) => { for (let i = from; i < src.length; i++) if (pred(src[i])) return i; throw new Error('marker not found'); };
const utilA = find((l) => l.startsWith('const rand = '));
const utilB = find((l) => l.startsWith('const lerp = '));
const texA = find((l) => l.startsWith('// ============================== textures'));
const texB = find((l) => l.startsWith('// ============================== audio'), texA);
const utils = src.slice(utilA, utilB + 1).join('\n');
const textures = src.slice(texA, texB).join('\n');
const surfaces = fs.readFileSync(path.join(repo, 'src/surfaces.js'), 'utf8').replace(/^export /gm, '');

// Layout and items: keep in sync with game/tests/textures.rs.
const sheet = String.raw`
const PAD = 4, TITLE_H = 24, LABEL_H = 16;
function renderSection(s) {
  const n = s.items.length, rows = Math.ceil(n / s.cols);
  const w = PAD + s.cols * (s.cw + PAD), h = TITLE_H + rows * (s.ch + LABEL_H + PAD) + PAD;
  const c = document.createElement('canvas'); c.width = w; c.height = h;
  const ctx = c.getContext('2d');
  ctx.fillStyle = '#202020'; ctx.fillRect(0, 0, w, h);
  ctx.fillStyle = '#ffffff'; ctx.font = 'bold 14px sans-serif';
  ctx.fillText(s.name, PAD, 17);
  s.items.forEach((it, i) => {
    const col = i % s.cols, row = Math.floor(i / s.cols);
    const x0 = PAD + col * (s.cw + PAD), y0 = TITLE_H + row * (s.ch + LABEL_H + PAD);
    ctx.fillStyle = '#7a7a7a'; ctx.fillRect(x0, y0, s.cw, s.ch);
    ctx.fillStyle = '#9a9a9a';
    for (let yy = 0; yy < s.ch; yy += 16) for (let xx = 0; xx < s.cw; xx += 16)
      if ((Math.floor(xx / 16) + Math.floor(yy / 16)) % 2 === 1) ctx.fillRect(x0 + xx, y0 + yy, Math.min(16, s.cw - xx), Math.min(16, s.ch - yy));
    const img = it.tex.image, tw = img.width, th = img.height;
    const k = Math.min(s.cw / tw, s.ch / th), dw = tw * k, dh = th * k;
    ctx.drawImage(img, x0 + (s.cw - dw) / 2, y0 + (s.ch - dh) / 2, dw, dh);
    ctx.fillStyle = '#dddddd'; ctx.font = '11px sans-serif';
    ctx.fillText(it.label, x0 + 2, y0 + s.ch + 12);
  });
  return c;
}
const item = (label, tex) => ({ label, tex });
const hand = (color, thickness, jitter, gap, wobble, doubleStroke, drip, w, h) => ({ color, thickness, jitter, gap, wobble, doubleStroke, drip, w, h });
const txt = (w, h, font, color, glow) => ({ w, h, font, color, glow });
window.buildSheets = () => {
  const surf = makeSurfaces(canvasTex, nativeRandom);
  const surfItems = [];
  for (const k of ['drywall', 'paint', 'concrete', 'brick', 'tile']) for (let g = 0; g < 4; g++) surfItems.push(item('wall ' + k + ' ' + g, surf.wall(k, g)));
  for (const k of ['concrete', 'office', 'tile']) for (let g = 0; g < 4; g++) surfItems.push(item('floor ' + k + ' ' + g, surf.floor(k, g)));
  for (const k of ['slab', 'plaster']) for (let g = 0; g < 4; g++) surfItems.push(item('ceil ' + k + ' ' + g, surf.ceil(k, g)));
  const secs = [];
  secs.push({ name: 'patterns', cw: 256, ch: 256, cols: 6, items: Object.entries(WALL_PATTERNS).map(([name, f]) => item(name, canvasTex(512, 512, (ctx, w, h) => {
    ctx.fillStyle = '#f4f0e0'; ctx.fillRect(0, 0, w, h);
    f(ctx, w, h, 'rgba(0,0,0,0.85)', 'rgba(170,20,20,0.7)');
  }))) });
  const wp = [];
  for (let p = 0; p < 21; p++) for (let g = 0; g < 4; g++) wp.push(item(p + ':' + g, makeWallpaper(WALL_PALETTES[p], 'plain', g)));
  wp.push(item('moss 0', makeMossWallpaper(WALL_PALETTES[0])));
  wp.push(item('moss 20', makeMossWallpaper(WALL_PALETTES[20])));
  secs.push({ name: 'wallpaper (plain, palette:grime)', cw: 128, ch: 128, cols: 12, items: wp });
  const detail = [[20, 'plain', 0], [20, 'plain', 3], [0, 'flock', 1], [5, 'trellis', 2], [12, 'hex', 1], [16, 'paisley', 0], [1, 'stars', 3], [9, 'vines', 2]];
  const wd = detail.map(([p, k, g]) => item(p + ' ' + k + ' ' + g, makeWallpaper(WALL_PALETTES[p], k, g)));
  wd.splice(2, 0, item('moss 20', makeMossWallpaper(WALL_PALETTES[20])));
  secs.push({ name: 'wallpaper detail', cw: 256, ch: 256, cols: 5, items: wd });
  const fc = [];
  for (let g = 0; g < 5; g++) fc.push(item('carpet ' + g, makeCarpet(g)));
  ceilingVariants.forEach((t, g) => fc.push(item('ceiling ' + g, t)));
  fc.push(item('lightTex', lightTex));
  fc.push(item('dustSprite', dustSprite));
  fc.push(item('faceTex', faceTex));
  fc.push(item('staticTex', staticTex));
  secs.push({ name: 'floors, ceilings, sprites', cw: 256, ch: 256, cols: 5, items: fc });
  const handItems = [
    ['CALL ME', hand('rgba(10,10,10,0.9)', 4, 1.6, 0.02, 1.2, false, false, 256, 64)],
    ['OUT OF SERVICE', hand('rgba(160,30,20,0.98)', 8, 2.2, 0.02, 1.6, true, true, 1024, 220)],
    ['DO NOT ENTER', hand('rgba(160,30,20,0.98)', 6, 1.9, 0.02, 1.4, true, true, 1024, 220)],
    ['OUT OF ORDER', hand('rgba(20,15,10,0.98)', 6, 1.8, 0.02, 1.4, false, false, 512, 128)],
    ['M.KANE', hand('rgba(20,15,10,0.9)', 3, 1.5, 0.02, 1.2, false, false, 256, 128)],
    ['FRAGILE', hand('rgba(20,10,5,0.95)', 4, 1.6, 0.02, 1.2, false, false, 512, 96)],
    ["DON'T LOOK AT IT", hand('rgba(140,20,15,0.95)', 6, 1.6, 0.02, 1.2, true, true, 1024, 220)],
    ['IT HUNTS BY SOUND', hand('rgba(230,225,210,0.90)', 4, 1.9, 0.05, 1.5, false, false, 1024, 220)],
    ['HELP ME', hand('rgba(15,10,8,0.95)', 3, 1.2, 0.01, 0.8, true, false, 1024, 220)],
    ['DAY 47', hand('rgba(220,180,40,0.92)', 5, 1.4, 0.03, 1.4, false, false, 1024, 220)],
    ['RUN', hand('rgba(100,10,10,0.98)', 8, 2.4, 0.06, 2.0, false, true, 1024, 220)],
    ['HELLO ?', hand('rgba(30,50,120,0.95)', 4, 1.5, 0.03, 1.1, false, false, 1024, 220)],
    ['I MADE IT TO LEVEL 3', hand('rgba(35,30,25,0.88)', 3, 1.7, 0.04, 1.6, false, false, 1024, 220)],
    ['ABCDEFGHIJKLM', {}],
    ['NOPQRSTUVWXYZ', {}],
    ["0123456789!?.,'-/()", {}],
  ];
  secs.push({ name: 'handwrittenTex', cw: 512, ch: 128, cols: 3, items: handItems.map(([t, o]) => item(t, handwrittenTex(t, o))) });
  const textItems = [
    ['EXIT', txt(512, 128, 'bold 60px Helvetica, Arial, sans-serif', '#1c2a44', 0)],
    ['STAIRS', txt(256, 76, 'bold 44px Helvetica, Arial, sans-serif', '#e8e2d0', 2)],
    ['COLA-Z', txt(1024, 192, 'bold 140px "Impact", sans-serif', '#c11515', 6)],
    ['$1.25', txt(128, 32, 'bold 22px monospace', '#c11515', 0)],
    ['E - -', txt(256, 64, 'bold 40px "Courier New", monospace', '#ff4020', 6)],
    ['INSERT COIN', txt(256, 32, 'bold 18px sans-serif', '#e8e8e8', 0)],
    ['7', txt(128, 64, 'bold 44px "Courier New", monospace', '#f04010', 6)],
    ['CLEAN 3000', txt(256, 64, 'bold 26px sans-serif', '#4a5a70', 2)],
    ['DEFAULT', {}],
    ['TILT', { rot: 0.3 }],
  ];
  secs.push({ name: 'textTex', cw: 512, ch: 128, cols: 3, items: textItems.map(([t, o]) => item(t, textTex(t, o))) });
  secs.push({ name: 'surfaces', cw: 256, ch: 256, cols: 8, items: surfItems });
  secs.push({ name: 'wallpaperVariants (random patterns / moss palettes)', cw: 128, ch: 128, cols: 12, items: wallpaperVariants.map((v) => item(v.paletteIdx + ':' + v.grimeLvl + (v.moss ? 'm' : ''), v.tex)) });
  const canvases = secs.map(renderSection);
  const sw = Math.max(...canvases.map((c) => c.width)), sh = canvases.reduce((a, c) => a + c.height, 0);
  const all = document.createElement('canvas'); all.width = sw; all.height = sh;
  const actx = all.getContext('2d');
  actx.fillStyle = '#101010'; actx.fillRect(0, 0, sw, sh);
  let y = 0;
  for (const c of canvases) { actx.drawImage(c, 0, y); y += c.height; }
  // per-texture alpha-weighted mean colour, same as mean_rgba in game/tests/textures.rs
  const stats = [];
  secs.forEach((s, si) => s.items.forEach((it) => {
    const im = it.tex.image, c = document.createElement('canvas'); c.width = im.width; c.height = im.height;
    const x = c.getContext('2d'); x.drawImage(im, 0, 0);
    const d = x.getImageData(0, 0, im.width, im.height).data;
    let r = 0, g = 0, b = 0, a = 0;
    for (let i = 0; i < d.length; i += 4) { const w = d[i + 3]; r += d[i] * w; g += d[i + 1] * w; b += d[i + 2] * w; a += w; }
    const k = a > 0 ? 1 / a : 0;
    stats.push([si, it.label, (r * k).toFixed(2), (g * k).toFixed(2), (b * k).toFixed(2), (a / (d.length / 4)).toFixed(2)].join('\t'));
  }));
  return { sections: canvases.map((c) => c.toDataURL('image/png')), all: all.toDataURL('image/png'), stats: stats.join('\n') + '\n' };
};
`;

const THREE_STUB = `
const THREE = {
  SRGBColorSpace: 'srgb', RepeatWrapping: 1000,
  CanvasTexture: class { constructor(c) { this.image = c; this.needsUpdate = true; } dispose() {} },
};`;

const script = `(() => {\n${THREE_STUB}\n${utils}\nconst nativeRandom = Math.random;\n${textures}\n${surfaces}\n${sheet}\n})();`;

const browser = await chromium.launch();
try {
  const page = await browser.newPage();
  page.on('pageerror', (e) => { console.error('page error:', e.message); });
  await page.setContent('<!doctype html><html><body></body></html>');
  await page.addScriptTag({ content: script });
  const out = await page.evaluate(() => window.buildSheets());
  const dir = path.join(ws, 'target', 'texture-sheet-js');
  fs.mkdirSync(dir, { recursive: true });
  const save = (file, url) => fs.writeFileSync(file, Buffer.from(url.split(',')[1], 'base64'));
  out.sections.forEach((u, i) => save(path.join(dir, String(i).padStart(2, '0') + '.png'), u));
  save(path.join(ws, 'target', 'texture-sheet-js.png'), out.all);
  fs.writeFileSync(path.join(ws, 'target', 'texture-stats-js.tsv'), out.stats);
  // compare with the Rust stats if the test has run
  const rustStats = path.join(ws, 'target', 'texture-stats.tsv');
  if (fs.existsSync(rustStats)) {
    const parse = (t) => t.trim().split('\n').map((l) => l.split('\t'));
    const js = parse(out.stats), rs = parse(fs.readFileSync(rustStats, 'utf8'));
    const rows = js.map((j, i) => {
      const r = rs[i] || [];
      const d = [2, 3, 4, 5].map((k) => +r[k] - +j[k]);
      return { sec: j[0], label: j[1], same: r[1] === j[1], d, max: Math.max(...d.map(Math.abs)) };
    });
    console.log('mean colour diff (rust - js), worst 25: [dR dG dB dA]');
    for (const r of rows.filter((r) => r.sec !== '7').sort((a, b) => b.max - a.max).slice(0, 25)) console.log(r.sec, r.label.padEnd(24), r.same ? '' : 'LABEL MISMATCH', r.d.map((v) => v.toFixed(1).padStart(6)).join(' '));
  }
  console.log('wrote', path.join(ws, 'target', 'texture-sheet-js.png'), 'and', out.sections.length, 'section sheets in', dir);
} finally {
  await browser.close();
}
