// HUD port check: put the original page (backrooms.html + backrooms.css, its
// script blocked) into the DOM states that `cargo test -p game --test hud`
// dumped as target/hud-<name>.json, screenshot it in Chromium at 1280×800 @2x,
// and diff against our CPU composite target/hud-<name>.png.
//
//   (cd <OpenCarole> && python3 -m http.server 8777 --bind 127.0.0.1) &
//   PLAYWRIGHT=<OpenCarole>/node_modules/playwright/index.mjs \
//     node game/tests/js/hud_compare.mjs target menu hud journal journal-cover journal-medals debug effects dead achievement
//
// → target/ref-<name>.png (Chromium) and target/cmp-<name>.png (ours | ref | |diff|×4),
// and the mean abs difference per state. Transitions are disabled and keyframe
// animations held at 0% in the reference; the #gl canvas filter is not a page
// layer (scene_fx.rs owns it), so it is switched off too.
const { chromium } = await import(process.env.PLAYWRIGHT || '/Users/nohwan/Documents/Personal_Projects/OpenCarole/node_modules/playwright/index.mjs');
import fs from 'fs';
const PAGE = process.env.HUD_PAGE || 'http://127.0.0.1:8777/backrooms.html';
const [dir, ...names] = process.argv.slice(2);
const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1280, height: 800 }, deviceScaleFactor: 2 });
await page.route('**/backrooms.js', r => r.fulfill({ status: 200, contentType: 'text/javascript', body: '' }));
for (const name of names) {
  const state = JSON.parse(fs.readFileSync(`${dir}/hud-${name}.json`, 'utf8'));
  await page.goto(PAGE);
  await page.evaluate((st) => {
    const css = document.createElement('style');
    css.textContent = '*, *::after, *::before { transition: none !important; animation-play-state: paused !important; animation-delay: 0s !important; } body.low-sanity #gl, body.crit-sanity #gl { filter: none !important; } .cover-fade, .medal-replay { animation: none !important; }';
    document.head.appendChild(css);
    const apply = (el, s) => {
      if (!el) return;
      if (s.html) el.innerHTML = s.html; else el.textContent = s.text;
      const keep = el.dataset.m ? ['menu-medal'] : [];
      el.className = [...new Set([...keep, ...s.classes])].join(' ');
      el.removeAttribute('style');
      for (const [k, v] of Object.entries(s.style)) el.style.setProperty(k, v);
      for (const [k, v] of Object.entries(s.attrs)) if (k === 'disabled') el.disabled = v === 'true'; else el.setAttribute(k, v);
    };
    for (const [id, s] of Object.entries(st.elements)) {
      const el = document.getElementById(id);
      if (!el) continue;
      // containers: keep their children, only classes/styles
      if (['hud', 'compass', 'journal', 'overlay', 'overlay-buttons', 'menu-medals', 'sanity', 'stamina', 'battery', 'grain'].includes(id)) {
        el.className = s.classes.join(' '); el.removeAttribute('style');
        for (const [k, v] of Object.entries(s.style)) el.style.setProperty(k, v);
        continue;
      }
      apply(el, s);
    }
    for (const [m, s] of Object.entries(st.menuMedals)) apply(document.querySelector(`.menu-medal[data-m="${m}"]`), s);
    document.body.className = st.body.classes.join(' ');
    const gl = document.getElementById('gl');
    gl.width = 1280; gl.height = 800;
    const ctx = gl.getContext('2d'); ctx.fillStyle = st.bg; ctx.fillRect(0, 0, 1280, 800);
  }, state);
  await page.waitForTimeout(100);
  await page.screenshot({ path: `${dir}/ref-${name}.png` });
  // compare in a canvas
  const ours = 'data:image/png;base64,' + fs.readFileSync(`${dir}/hud-${name}.png`).toString('base64');
  const ref = 'data:image/png;base64,' + fs.readFileSync(`${dir}/ref-${name}.png`).toString('base64');
  const res = await page.evaluate(async ([a, b]) => {
    const load = (src) => new Promise((ok) => { const i = new Image(); i.onload = () => ok(i); i.src = src; });
    const [ia, ib] = await Promise.all([load(a), load(b)]);
    const w = ia.width, h = ia.height;
    const c = document.createElement('canvas'); c.width = w * 3; c.height = h;
    const x = c.getContext('2d');
    x.drawImage(ia, 0, 0); x.drawImage(ib, w, 0);
    const da = x.getImageData(0, 0, w, h).data, db = x.getImageData(w, 0, w, h).data;
    const out = x.createImageData(w, h);
    let sum = 0, big = 0;
    for (let i = 0; i < da.length; i += 4) {
      const d = (Math.abs(da[i] - db[i]) + Math.abs(da[i + 1] - db[i + 1]) + Math.abs(da[i + 2] - db[i + 2])) / 3;
      sum += d; if (d > 24) big++;
      out.data[i] = Math.min(255, d * 4); out.data[i + 1] = Math.min(255, d * 4); out.data[i + 2] = Math.min(255, d * 4); out.data[i + 3] = 255;
    }
    x.putImageData(out, w * 2, 0);
    return { mean: sum / (w * h), big: big / (w * h), url: c.toDataURL('image/png') };
  }, [ours, ref]);
  fs.writeFileSync(`${dir}/cmp-${name}.png`, Buffer.from(res.url.split(',')[1], 'base64'));
  console.log(name, 'mean abs diff', res.mean.toFixed(3), ' >24:', (res.big * 100).toFixed(3) + '%');
}
await browser.close();
