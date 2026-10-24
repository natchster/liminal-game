// Compare with: cargo run --release -p placers_difftest --example trig_repro > trig.txt; node trig_repro.mjs trig.txt
import fs from 'fs';
const names = ['sin', 'cos', 'atan2', 'pow', 'tan', 'asin'];
const cnt = Object.fromEntries(names.map((n) => [n, 0])); let n = 0, ex = {};
for (const l of fs.readFileSync(process.argv[2], 'utf8').trim().split('\n')) {
  const [a, b, ...r] = l.split(' ').map(Number); n++;
  const js = [Math.sin(a), Math.cos(a), Math.atan2(a, b), Math.pow(b, 2.4), Math.tan(a), Math.asin(b * 0.3)];
  js.forEach((v, i) => { if (v !== r[i] && !(isNaN(v) && isNaN(r[i]))) { cnt[names[i]]++; ex[names[i]] ??= [a, b, v, r[i]]; } });
}
console.log(n, 'inputs; results that differ from Rust std:', JSON.stringify(cnt)); console.log(JSON.stringify(ex));
