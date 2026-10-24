// Compare with: cargo run --release -p placers_difftest --example euler_repro > rs.txt; node euler_repro.mjs rs.txt
const THREE = await import((process.env.BACKROOMS_REPO || '/Users/nohwan/Documents/Personal_Projects/OpenCarole') + '/vendor/three.module.min.js');
import fs from 'fs';
globalThis.__threeUuidRandom = Math.random;
const lines = fs.readFileSync(process.argv[2], 'utf8').trim().split('\n');
let qdiff = 0, zsign = 0, sinUlp = 0, n = 0, ex = null;
for (const l of lines) {
  const [x, y, qx, qy, qz, qw, ex_, ez] = l.split(' ').map(Number);
  const o = new THREE.Object3D(); o.rotation.y = y; o.rotation.x = x;
  const c = o.clone();
  n++;
  const q = o.quaternion;
  if (q.x !== qx || q.y !== qy || q.z !== qz || q.w !== qw) qdiff++;
  if (Math.sign(1 / c.rotation.z) !== Math.sign(1 / ez) || Math.abs(c.rotation.z - ez) > 1) { zsign++; if (!ex) ex = { x, y, js: [c.rotation.x, c.rotation.z, q.x, q.y, q.z, q.w], rs: [ex_, ez, qx, qy, qz, qw] }; }
}
console.log(`${n} inputs: quaternion bits differ in ${qdiff}, clone() Euler z differs (±pi or ±0) in ${zsign}`);
console.log(JSON.stringify(ex));
