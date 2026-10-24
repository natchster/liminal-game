// Which lines of the JS placers did the diff never execute? Reads the V8
// block coverage written by workers run with NODE_V8_COVERAGE (run.mjs
// --coverage does that) and lists, per placer, the lines whose code never
// ran: the branches the comparison could not check.
//
//   node coverage.mjs <coverage dir> [placerNames...]   (prints a report)
import fs from 'fs';
import path from 'path';
import { Bundle } from './jsbundle.mjs';

const REPO = process.env.BACKROOMS_REPO || '/Users/nohwan/Documents/Personal_Projects/OpenCarole';
const BACKROOMS_JS = process.env.BACKROOMS_JS || path.join(REPO, 'backrooms.js');

/** { placerName: { lines: total code lines, uncovered: [[absLine, text]] } } */
export function uncovered(covDir, names) {
  const src = fs.readFileSync(BACKROOMS_JS, 'utf8');
  const bundle = new Bundle(src);
  const want = new Map();
  for (const n of names) { const i = bundle.decl.get(n); if (i !== undefined) want.set(`backrooms.js:${bundle.stmts[i].line}`, n); }
  const paint = new Map();   // url -> Int32Array of counts per char (-1 = no info)
  for (const f of fs.readdirSync(covDir)) {
    if (!f.endsWith('.json')) continue;
    const j = JSON.parse(fs.readFileSync(path.join(covDir, f), 'utf8'));
    for (const r of j.result) {
      if (!want.has(r.url)) continue;
      const s = bundle.stmts[bundle.decl.get(want.get(r.url))];
      const len = s.text.length;
      const local = new Int32Array(len).fill(-1);
      for (const fn of r.functions) for (const rg of fn.ranges) local.fill(rg.count, Math.max(0, rg.startOffset), Math.min(len, rg.endOffset));
      // several coverage files (processes): a char is covered if any run covered it
      const acc = paint.get(r.url);
      if (!acc) paint.set(r.url, local);
      else for (let k = 0; k < len; k++) if (local[k] > 0 || acc[k] < 0) acc[k] = Math.max(acc[k], local[k]);
    }
  }
  const out = {};
  for (const [url, name] of want) {
    const s = bundle.stmts[bundle.decl.get(name)];
    const counts = paint.get(url);
    const lines = s.text.split('\n');
    const res = { lines: 0, uncovered: [], partial: [], ran: !!counts };
    let off = 0;
    for (let li = 0; li < lines.length; li++) {
      const L = lines[li];
      const code = L.replace(/\/\/.*$/, '').trim();
      const isCode = code.length && !/^[})\];,]+$/.test(code);
      if (isCode) {
        res.lines++;
        if (counts) {
          // the line is uncovered if every non-space char of its code has count 0
          let anyRun = false, anyZero = false;
          for (let k = 0; k < L.length; k++) {
            if (L[k] === ' ') continue;
            if (L.slice(k).startsWith('//')) break;
            const c = counts[off + k];
            if (c > 0) anyRun = true; else if (c === 0) anyZero = true;
          }
          if (anyZero && !anyRun) res.uncovered.push([s.line + li, L.trim()]);
          else if (anyZero) {
            // a sub-expression / branch on this line never ran
            let frag = '';
            for (let k = 0; k < L.length; k++) { if (L.slice(k).startsWith('//')) break; frag += counts[off + k] === 0 ? L[k] : (frag.endsWith(' … ') ? '' : ' … '); }
            frag = frag.replace(/^( … )+|( … )+$/g, '').trim();
            if (/[A-Za-z0-9]/.test(frag) && !/^(return|continue|break);?$/.test(frag.replace(/[{}\s]/g, ''))) res.partial.push([s.line + li, L.trim(), frag]);
          }
        }
      }
      off += L.length + 1;
    }
    out[name] = res;
  }
  return out;
}

if (process.argv[1] && process.argv[1].endsWith('coverage.mjs')) {
  const [dir, ...names] = process.argv.slice(2);
  const r = uncovered(dir, names);
  for (const [n, v] of Object.entries(r)) {
    console.log(`${n}: ${v.ran ? `${v.uncovered.length}/${v.lines} code lines never ran` : 'never called'}`);
    for (const [l, t] of v.uncovered) console.log(`   ${l}: ${t}`);
    for (const [l, t, f] of v.partial) console.log(`   ${l}: (part never ran: ${f})  ${t}`);
  }
}
