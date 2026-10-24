#!/usr/bin/env node
// Compare browser and native dumps of the same levels.
//
//   node native/tools/levelcheck/compare.mjs [--dir native/target/levelcheck] [--seed 4242] [--levels 1,2,3] [-v]
//
// Reads js_<seed>_L<n>.json (tools/levelcheck/js-capture.mjs) and
// rs_<seed>_L<n>.json (`backrooms --dump-level`), and reports per level: the
// first draw-trace mark where the two builds part (the phase or placer that
// drew a different number of times), the grid, zones, exit, key props,
// counts, entities, and placed objects matched placer by placer.
import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';

const HERE = path.dirname(fileURLToPath(import.meta.url));
const args = process.argv.slice(2);
const opt = (k, d) => { const i = args.indexOf('--' + k); return i >= 0 ? args[i + 1] : d; };
const dir = path.resolve(opt('dir', path.join(HERE, '..', '..', 'target', 'levelcheck')));
const seed = opt('seed', '4242');
const verbose = args.includes('-v');
let levels = opt('levels', null);
levels = levels ? levels.split(',').map(Number)
  : fs.readdirSync(dir).map((f) => (f.match(new RegExp(`^js_${seed}_L(\\d+)\\.json$`)) || [])[1]).filter(Boolean).map(Number).sort((a, b) => a - b);

const close = (a, b, eps = 1e-4) => Math.abs(a - b) <= eps * Math.max(1, Math.abs(a), Math.abs(b));
const vclose = (a, b, eps) => a && b && a.length === b.length && a.every((x, i) => close(x, b[i], eps));
let allOk = true;

for (const L of levels) {
  const jf = path.join(dir, `js_${seed}_L${L}.json`), rf = path.join(dir, `rs_${seed}_L${L}.json`);
  if (!fs.existsSync(jf) || !fs.existsSync(rf)) { console.log(`level ${L}: missing ${fs.existsSync(jf) ? rf : jf}`); allOk = false; continue; }
  const J = JSON.parse(fs.readFileSync(jf, 'utf8')), R = JSON.parse(fs.readFileSync(rf, 'utf8'));
  const issues = [];
  const check = (name, ok, detail = '') => { if (!ok) issues.push(name + (detail ? ': ' + detail : '')); };

  // draw trace: first mark that differs
  const jm = J.marks, rm = R.marks;
  let firstBad = -1;
  for (let i = 0; i < Math.max(jm.length, rm.length); i++) {
    const a = jm[i], b = rm[i];
    if (!a || !b || a[0] !== b[0] || a[1] !== b[1]) { firstBad = i; break; }
  }
  if (firstBad >= 0) {
    const a = jm[firstBad], b = rm[firstBad], pa = jm[firstBad - 1];
    check('draw trace', false, `diverges at mark #${firstBad}: js ${JSON.stringify(a)} vs native ${JSON.stringify(b)}` +
      (pa ? ` (after ${JSON.stringify(pa)}; the phase/placer ending at mark #${firstBad} drew ${a ? a[1] - pa[1] : '?'} in js, ${b && rm[firstBad - 1] ? b[1] - rm[firstBad - 1][1] : '?'} native)` : ''));
  }
  check('marks count', jm.length === rm.length, `${jm.length} vs ${rm.length}`);
  check('scp', J.scp === R.scp, `${J.scp} vs ${R.scp}`);
  check('weather', J.weather === R.weather, `${J.weather} vs ${R.weather}`);
  check('grid size', J.grid.W === R.grid.W && J.grid.H === R.grid.H, `${J.grid.W}x${J.grid.H} vs ${R.grid.W}x${R.grid.H}`);
  if (J.grid.tiles && R.grid.tiles) {
    let d = 0; for (let i = 0; i < J.grid.tiles.length; i++) if (J.grid.tiles[i] !== R.grid.tiles[i]) d++;
    check('tiles', d === 0 && J.grid.tiles.length === R.grid.tiles.length, `${d} tiles differ`);
  }
  if (J.arch && R.arch) {
    check('dominant', J.arch.dominant === R.arch.dominant, `${J.arch.dominant} vs ${R.arch.dominant}`);
    check('zones', JSON.stringify(J.arch.zones.map((z) => [z.arch, z.x0, z.z0, z.x1, z.z1])) === JSON.stringify(R.arch.zones.map((z) => [z.arch, z.x0, z.z0, z.x1, z.z1])));
  } else check('arch', !J.arch === !R.arch);
  check('exit', (!J.exit && !R.exit) || (J.exit && R.exit && close(J.exit.x, R.exit.x) && close(J.exit.z, R.exit.z)), `${JSON.stringify(J.exit)} vs ${JSON.stringify(R.exit)}`);
  for (const k of ['tv', 'phone', 'clock', 'mannequin']) {
    const a = J.props[k], b = R.props[k];
    check('prop ' + k, (!a && !b) || (a && b && close(a.x, b.x) && close(a.z, b.z)), `${JSON.stringify(a)} vs ${JSON.stringify(b)}`);
  }
  for (const k of ['almonds', 'batteries']) check(k, JSON.stringify(J.props[k].map((p) => [p.x.toFixed(3), p.z.toFixed(3)])) === JSON.stringify(R.props[k].map((p) => [p.x.toFixed(3), p.z.toFixed(3)])));
  for (const k of ['walls', 'panels', 'almonds', 'batteries', 'interactables', 'animators']) check('count ' + k, J.counts[k] === R.counts[k], `${J.counts[k]} vs ${R.counts[k]}`);
  check('entities', JSON.stringify(J.entities.map((e) => [e.type, e.x.toFixed(3), e.z.toFixed(3)])) === JSON.stringify(R.entities.map((e) => [e.type, e.x.toFixed(3), e.z.toFixed(3)])),
    `${J.entities.map((e) => e.type).join(',')} vs ${R.entities.map((e) => e.type).join(',')}`);
  check('player', close(J.player.x, R.player.x) && close(J.player.z, R.player.z) && close(J.player.yaw, R.player.yaw));
  check('barricades', JSON.stringify(J.barricades) === JSON.stringify(R.barricades));

  // placed objects, placer by placer, in order
  const byP = (list) => { const m = new Map(); for (const o of list) { if (!m.has(o.placer)) m.set(o.placer, []); m.get(o.placer).push(o); } return m; };
  const jp = byP(J.objects), rp = byP(R.objects);
  const objIssues = [];
  for (const [p, js] of jp) {
    const rs = rp.get(p) || [];
    if (rs.length !== js.length) { objIssues.push(`${p}: ${js.length} objects vs ${rs.length}`); continue; }
    for (let i = 0; i < js.length; i++) {
      const a = js[i], b = rs[i];
      if (!vclose(a.pos, b.pos, 1e-3)) { objIssues.push(`${p}[${i}] pos ${a.pos} vs ${b.pos}`); break; }
      if (!vclose(a.min, b.min, 2e-3) || !vclose(a.max, b.max, 2e-3)) { objIssues.push(`${p}[${i}] bounds ${a.min}..${a.max} vs ${b.min}..${b.max}`); break; }
      if (a.meshes !== b.meshes) { objIssues.push(`${p}[${i}] meshes ${a.meshes} vs ${b.meshes}`); break; }
      if (a.interact !== b.interact) { objIssues.push(`${p}[${i}] interact ${a.interact} vs ${b.interact}`); break; }
    }
  }
  for (const [p, rs] of rp) if (!jp.has(p)) objIssues.push(`${p}: native only (${rs.length})`);
  check('objects', objIssues.length === 0, `${objIssues.length} placers differ` + (objIssues.length ? '\n      ' + objIssues.slice(0, verbose ? 1e9 : 8).join('\n      ') : ''));

  const ok = issues.length === 0;
  allOk &&= ok;
  console.log(`level ${L} (seed ${seed}): ${ok ? 'MATCH' : issues.length + ' differences'}  [js ${jm.length} marks / ${jm.length ? jm[jm.length - 1][1] : 0} draws / ${J.objects.length} objects; native ${rm.length} / ${rm.length ? rm[rm.length - 1][1] : 0} / ${R.objects.length}]`);
  for (const i of issues) console.log('   - ' + i);
}
process.exit(allOk ? 0 : 1);
