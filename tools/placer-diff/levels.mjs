// Realistic grids for the placer diff: real archgen layouts (src/archgen.js,
// the generator the game uses) at the game's level sizes, plus copies with
// some zones hidden from the placer (tiles = 2), the way `themed()` in
// backrooms.js masks the zones a placer's theme is not welcome in. The BFS
// distance field is computed before masking, as in buildWorld().
//
//   node levels.mjs <out.txt>     one level per line: "w h tiles dist"
import fs from 'fs';
import path from 'path';
import { pathToFileURL } from 'url';

const REPO = process.env.BACKROOMS_REPO || '/Users/nohwan/Documents/Personal_Projects/OpenCarole';

function mulberry32(seed) {
  let t = seed >>> 0;
  return function () {
    t = (t + 0x6D2B79F5) >>> 0;
    let x = t;
    x = Math.imul(x ^ (x >>> 15), x | 1);
    x ^= x + Math.imul(x ^ (x >>> 7), x | 61);
    return ((x ^ (x >>> 14)) >>> 0) / 4294967296;
  };
}
// backrooms.js bfs(1, 1)
function bfs(W, H, tiles, sx, sz) {
  const tIdx = (x, z) => z * W + x;
  const inGrid = (x, z) => x >= 0 && z >= 0 && x < W && z < H;
  const isWallTile = (x, z) => !inGrid(x, z) || tiles[tIdx(x, z)] === 1;
  const dist = new Int32Array(W * H).fill(-1);
  if (isWallTile(sx, sz)) {
    let found = false;
    for (const [dx, dz] of [[1, 0], [-1, 0], [0, 1], [0, -1], [1, 1], [-1, 1], [1, -1], [-1, -1]]) {
      if (!isWallTile(sx + dx, sz + dz)) { sx += dx; sz += dz; found = true; break; }
    }
    if (!found) return dist;
  }
  const q = [tIdx(sx, sz)];
  dist[q[0]] = 0;
  for (let qi = 0; qi < q.length; qi++) {
    const cur = q[qi], cx = cur % W, cz = (cur / W) | 0, d = dist[cur] + 1;
    for (const [dx, dz] of [[1, 0], [-1, 0], [0, 1], [0, -1]]) {
      const nx = cx + dx, nz = cz + dz;
      if (inGrid(nx, nz) && tiles[tIdx(nx, nz)] === 0 && dist[tIdx(nx, nz)] === -1) { dist[tIdx(nx, nz)] = d; q.push(tIdx(nx, nz)); }
    }
  }
  return dist;
}

export async function makeLevels() {
  const { generateLayout } = await import(pathToFileURL(path.join(REPO, 'src/archgen.js')).href);
  const out = [];
  // [level number, layout seed, masked?]
  const plan = [[1, 101, false], [2, 202, false], [4, 404, false], [6, 606, false], [9, 909, false], [3, 303, true], [7, 707, true]];
  for (const [n, seed, masked] of plan) {
    const L = generateLayout({ size: Math.min(31 + n * 4, 53), level: n, rng: mulberry32(seed), cell: 4, scp: null });
    const tiles = Array.from(L.tiles, (t) => (t === 0 ? 0 : 1));
    const dist = bfs(L.W, L.H, tiles, 1, 1);
    if (masked) {
      const r = mulberry32(seed ^ 0x5eed);
      const allow = L.zones.map(() => r() < 0.5);
      for (let i = 0; i < tiles.length; i++) if (tiles[i] === 0 && !allow[L.zoneOf[i]]) tiles[i] = 2;
    }
    out.push({ w: L.W, h: L.H, tiles, dist: Array.from(dist) });
  }
  return out;
}

export function writeLevels(file, levels) {
  fs.writeFileSync(file, levels.map((l) => `${l.w} ${l.h} ${l.tiles.join('')} ${l.dist.join(',')}`).join('\n') + '\n');
}

if (import.meta.url === pathToFileURL(process.argv[1]).href) {
  writeLevels(process.argv[2] || 'levels.txt', await makeLevels());
}
