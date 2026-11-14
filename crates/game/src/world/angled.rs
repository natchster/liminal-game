//! Walls not quite on the grid (rooms R4).
//!
//! Archgen stands every partition on the tile grid. A straight run of a thin
//! partition that ends free on at least one side (an opening, a doorway with
//! no frame) may lean a few degrees: about the corner or the wall it starts
//! from, or about its middle when both its ends are free. Partitions put up
//! by hand, slightly off-square.
//!
//! A leaning run is drawn as the boxes it was, turned (`build_walls`). What
//! collides, blocks a look or a sound and shapes the light map is a
//! staircase of short boxes on the grid inside the drawn wall
//! ([`BOX_ANGLED`]), so every query over the solids reads it as it reads any
//! wall; whatever hangs on or stands against it turns with it
//! (`settle_props`). A run stays inside its own wall tiles (its free end at
//! most [`DEV_MAX`] off its line, its wall tile 2 m either side of it), so
//! the tiles, the paths and the tiles the placers chose are as they were. The choice is pure: a stream of its own
//! for each run, from the level's seed and the run's place ([`LEAN_SALT`]),
//! so no draw of the build moves.
use crate::base::rng::SplitMix64;
use crate::level::archgen::{BOX_ANGLED, BOX_JAMB, BOX_LINTEL, BOX_WALL, LayoutView, Solids, make_solids};
use std::collections::HashSet;

pub const LEAN_SALT: u64 = 0x1EA2_0FF5_90A2_E004;
/// The zones whose partitions may lean: thin walls put up by hand (the
/// store's shelves and the pool's basin are fitted to theirs)
const ZONES: [&str; 4] = ["lobby", "pillars", "office", "hall"];
/// The share of the runs that could lean which do
const CHANCE: f64 = 0.45;
/// How far a run leans (degrees): most a little off-square...
const LEAN: [f64; 2] = [2.0, 4.5];
/// ...its free end at most this far off its line (m)
const DEV: f64 = 0.6;
/// ...and of those that lean, now and then one at an angle
const STEEP_CHANCE: f64 = 0.18;
const LEAN_STEEP: [f64; 2] = [6.0, 10.0];
/// The farthest a run's free end moves off its line (m): a steep one's.
/// Its wall tile reaches 2 m either side of its line
pub const DEV_MAX: f64 = 1.2;
/// Less than this (degrees) a run stays straight: a long run with its free
/// end held to its `DEV` or `DEV_MAX`
const LEAN_MIN: f64 = 1.2;
/// The longest step of the staircase (tile units: 0.5 m)
const STEP: f64 = 0.125;

/// A run of partition that leans. Coordinates in tile units: `along` the
/// run's axis, `across` it
#[derive(Clone, Debug, PartialEq)]
pub struct Wall {
    /// the run lies along x (else along z)
    pub along_x: bool,
    /// its centre line (across)
    pub line: f64,
    /// what it turns about (along, on the line)
    pub pivot: f64,
    /// how far it turns (radians; + takes the far end of the run toward + across)
    pub theta: f64,
    /// half its thickness
    pub half: f64,
    /// what it is made of, as it stood on the grid: (owner tile, `[x0 z0 x1 z1 y0 y1]`)
    pub parts: Vec<(usize, [f64; 6])>,
}

impl Wall {
    fn al_ac(&self, u: f64, v: f64) -> (f64, f64) {
        if self.along_x { (u, v) } else { (v, u) }
    }
    fn uv(&self, al: f64, ac: f64) -> (f64, f64) {
        if self.along_x { (al, ac) } else { (ac, al) }
    }
    /// Where a point of the run as it stood on the grid stands now
    pub fn turn(&self, u: f64, v: f64) -> (f64, f64) {
        let (al, ac) = self.al_ac(u, v);
        let (s, c) = self.theta.sin_cos();
        let (da, dc) = (al - self.pivot, ac - self.line);
        self.uv(self.pivot + da * c - dc * s, self.line + da * s + dc * c)
    }
    /// The turn in the (u, v) plane, from +u toward +v (radians)
    pub fn phi(&self) -> f64 {
        if self.along_x { self.theta } else { -self.theta }
    }
    /// Its face on side `side` (+1: the + across side) at `al` along: the
    /// face's across coordinate
    pub fn face(&self, al: f64, side: f64) -> f64 {
        self.line + (al - self.pivot) * self.theta.tan() + side * self.half / self.theta.cos()
    }
    /// The coordinate along it of a point (u, v)
    pub fn along(&self, u: f64, v: f64) -> f64 {
        self.al_ac(u, v).0
    }
    /// Tile `t` is one of the run's (or the corner it turns about)
    pub fn owns(&self, t: usize) -> bool {
        self.parts.iter().any(|p| p.0 == t)
    }
    /// The staircase that stands in for it: short boxes on the grid, each
    /// inside the drawn wall over its length, each step overlapping the next
    /// across (a step is at most half the wall's half thickness), so no look
    /// passes between them
    pub fn steps(&self, out: &mut Vec<f32>) {
        let (sn, cs) = self.theta.sin_cos();
        let tn = sn / cs;
        let seg = STEP.min(0.5 * self.half / tn.abs().max(1e-9));
        let hc = self.half / cs;
        for &(owner, p) in &self.parts {
            let (a0, a1) = (self.al_ac(p[0], p[1]).0, self.al_ac(p[2], p[3]).0);
            let (w0, w1) = (self.pivot + (a0 - self.pivot) * cs, self.pivot + (a1 - self.pivot) * cs);
            let n = ((w1 - w0) / seg).ceil().max(1.0) as usize;
            for k in 0..n {
                let u0 = w0 + (w1 - w0) * k as f64 / n as f64;
                let u1 = w0 + (w1 - w0) * (k + 1) as f64 / n as f64;
                let (c0, c1) = ((u0 - self.pivot) * tn, (u1 - self.pivot) * tn);
                let (lo, hi) = (self.line + c0.max(c1) - hc, self.line + c0.min(c1) + hc);
                let (x0, z0) = self.uv(u0, lo);
                let (x1, z1) = self.uv(u1, hi);
                for v in [x0, z0, x1, z1, p[4], p[5], owner as f64, BOX_ANGLED as f64] {
                    out.push(v as f32);
                }
            }
        }
    }
}

/// How a run ends
#[derive(Clone, Copy, PartialEq)]
enum End {
    /// in an open tile with no door frame
    Free,
    /// at a corner or a tee that the run's line stops in: turns about the
    /// corner's centre, the corner's stub (box id) turning with it
    Corner(usize),
    /// against a wall that fills its tile: turns about the tile's edge
    Block,
    /// anything else (a frame, a pinned face, a wall of another thickness,
    /// the line going on through a tee)
    Held,
}

/// The walls of a level of rooms that lean, and its solids with their
/// staircases standing in for them. Pure in `(view, s, seed)`
pub fn lean(view: &LayoutView, s: Solids, seed: u32) -> (Solids, Vec<Wall>) {
    let (w, h) = (view.w, view.h);
    let idx = |x: i32, z: i32| (z * w + x) as usize;
    let inside = |x: i32, z: i32| x >= 0 && z >= 0 && x < w && z < h;
    // the boxes a tile owns, and every box over it
    let own = |t: usize| -> Vec<usize> {
        (s.start[t]..s.start[t + 1]).map(|k| s.items[k as usize] as usize).filter(|&b| s.box_at(b)[6] as usize == t).collect()
    };
    let over = |t: usize| (s.start[t]..s.start[t + 1]).map(|k| s.items[k as usize] as usize);
    let bx = |b: usize| -> [f64; 8] { std::array::from_fn(|k| s.box_at(b)[k] as f64) };
    let leans_here = |t: usize| ZONES.contains(&view.zones[view.zone_of[t] as usize].arch);
    // a tile whose one box runs straight through it along `ax` (true: x): its box
    let straight = |x: i32, z: i32, ax: bool| -> Option<usize> {
        if !inside(x, z) || view.tiles[idx(x, z)] != 1 || !leans_here(idx(x, z)) {
            return None;
        }
        let o = own(idx(x, z));
        let [b] = o[..] else { return None };
        let v = bx(b);
        let (a0, a1, c0, c1, f) = if ax { (v[0], v[2], v[1], v[3], x) } else { (v[1], v[3], v[0], v[2], z) };
        let full = (a0 - f as f64).abs() < 1e-5 && (a1 - f as f64 - 1.0).abs() < 1e-5;
        (v[7] as i32 == BOX_WALL && full && c1 - c0 < 0.3).then_some(b)
    };
    // how the run ends in tile (x, z), coming from the side `from` (-1: from
    // below along the axis), its partition `c0..c1` across
    let end = |x: i32, z: i32, ax: bool, from: i32, c: (f64, f64)| -> End {
        if !inside(x, z) {
            return End::Held;
        }
        let t = idx(x, z);
        if view.tiles[t] == 0 {
            let framed = over(t).any(|b| matches!(bx(b)[7] as i32, BOX_JAMB | BOX_LINTEL));
            return if framed { End::Held } else { End::Free };
        }
        let o = own(t);
        let (xf, zf) = (x as f64, z as f64);
        if let [b] = o[..] {
            let v = bx(b);
            let fills = v[0] <= xf + 1e-5 && v[1] <= zf + 1e-5 && v[2] >= xf + 1.0 - 1e-5 && v[3] >= zf + 1.0 - 1e-5;
            return if v[7] as i32 == BOX_WALL && fills { End::Block } else { End::Held };
        }
        if o.len() != 2 || !leans_here(t) {
            return End::Held;
        }
        // a corner or a tee: one box along the run's axis that stops at the
        // tile's middle (the line does not go on), at the run's thickness
        let f = if ax { xf } else { zf };
        let stub = o.iter().copied().find(|&b| {
            let v = bx(b);
            let (a0, a1, c0, c1) = if ax { (v[0], v[2], v[1], v[3]) } else { (v[1], v[3], v[0], v[2]) };
            let toward = if from < 0 { (a0 - f).abs() < 1e-5 && a1 < f + 0.75 } else { (a1 - f - 1.0).abs() < 1e-5 && a0 > f + 0.25 };
            v[7] as i32 == BOX_WALL && toward && (c0 - c.0).abs() < 1e-5 && (c1 - c.1).abs() < 1e-5
        });
        match stub {
            Some(b) if o.iter().all(|&k| bx(k)[7] as i32 == BOX_WALL) => End::Corner(b),
            _ => End::Held,
        }
    };
    let mut walls: Vec<Wall> = vec![];
    let mut gone: HashSet<usize> = HashSet::new();
    let mut corners: HashSet<usize> = HashSet::new();
    for ax in [true, false] {
        let (n_line, n_along) = if ax { (h, w) } else { (w, h) };
        for l in 0..n_line {
            let at = |a: i32| if ax { (a, l) } else { (l, a) };
            let mut a = 0;
            while a < n_along {
                let (x, z) = at(a);
                let Some(b0) = straight(x, z, ax) else {
                    a += 1;
                    continue;
                };
                let v0 = bx(b0);
                let c = if ax { (v0[1], v0[3]) } else { (v0[0], v0[2]) };
                // the run: tiles straight along the axis, at one thickness
                let mut run = vec![(idx(x, z), b0)];
                let mut b = a + 1;
                while b < n_along {
                    let (xb, zb) = at(b);
                    let Some(k) = straight(xb, zb, ax) else { break };
                    let v = bx(k);
                    let ck = if ax { (v[1], v[3]) } else { (v[0], v[2]) };
                    if (ck.0 - c.0).abs() > 1e-5 || (ck.1 - c.1).abs() > 1e-5 {
                        break;
                    }
                    run.push((idx(xb, zb), k));
                    b += 1;
                }
                let (lo_t, hi_t) = (at(a - 1), at(b));
                let (e0, e1) = (end(lo_t.0, lo_t.1, ax, 1, c), end(hi_t.0, hi_t.1, ax, -1, c));
                let (s0, s1) = (a as f64, b as f64);
                let first = run[0].0;
                a = b;
                // turns about: the middle when both ends are free, else the end that is held
                let pivot = match (e0, e1) {
                    (End::Free, End::Free) => (s0 + s1) / 2.0,
                    (End::Free, End::Corner(_)) => s1 + 0.5,
                    (End::Free, End::Block) => s1,
                    (End::Corner(_), End::Free) => s0 - 0.5,
                    (End::Block, End::Free) => s0,
                    _ => continue,
                };
                let corner = match (e0, e1) {
                    (End::Corner(k), _) | (_, End::Corner(k)) => Some(k),
                    _ => None,
                };
                if corner.is_some_and(|k| corners.contains(&(bx(k)[6] as usize))) {
                    continue;
                }
                // a stream of its own: the level's seed and the run's place
                let key = ((first as u64) << 1) | ax as u64;
                let mut r = SplitMix64::new(seed, LEAN_SALT ^ key.wrapping_mul(0x9E37_79B9_7F4A_7C15));
                let (go, steep, sign) = (r.next(), r.next() < STEEP_CHANCE, r.next());
                let (range, dev) = if steep { (LEAN_STEEP, DEV_MAX) } else { (LEAN, DEV) };
                let reach = if e0 == End::Free { pivot - s0 } else { s1 - pivot } * view.cell;
                let mag = r.range(range[0], range[1]).min((dev / reach).clamp(-1.0, 1.0).asin().to_degrees());
                if go >= CHANCE || mag < LEAN_MIN {
                    continue;
                }
                let theta = (if sign < 0.5 { -mag } else { mag }).to_radians();
                let half = (c.1 - c.0) / 2.0;
                let mut parts: Vec<(usize, [f64; 6])> = vec![];
                let uv = |al0: f64, al1: f64, y0: f64, y1: f64| -> [f64; 6] {
                    if ax { [al0, c.0, al1, c.1, y0, y1] } else { [c.0, al0, c.1, al1, y0, y1] }
                };
                // the corner's stub, from the corner's middle (the corner's
                // other arm covers the rest, whichever way the run turns)
                if let Some(k) = corner {
                    let v = bx(k);
                    let mid = if e0 == End::Free { s1 + 0.5 } else { s0 - 0.5 };
                    let (al0, al1) = if e0 == End::Free { (s1, mid) } else { (mid, s0) };
                    parts.push((v[6] as usize, uv(al0, al1, v[4], v[5])));
                    gone.insert(k);
                    corners.insert(v[6] as usize);
                }
                // against a wall that fills its tile: the end goes on into it,
                // so no sliver opens where it turns away
                let into = half * theta.tan().abs() + 0.003;
                for (i, &(t, k)) in run.iter().enumerate() {
                    let v = bx(k);
                    let (mut al0, mut al1) = (s0 + i as f64, s0 + i as f64 + 1.0);
                    if i == 0 && e0 == End::Block {
                        al0 -= into;
                    }
                    if i + 1 == run.len() && e1 == End::Block {
                        al1 += into;
                    }
                    parts.push((t, uv(al0, al1, v[4], v[5])));
                    gone.insert(k);
                }
                walls.push(Wall { along_x: ax, line: (c.0 + c.1) / 2.0, pivot, theta, half, parts });
            }
        }
    }
    if walls.is_empty() {
        return (s, walls);
    }
    let mut boxes: Vec<f32> = Vec::with_capacity(s.boxes.len() + walls.len() * 64 * 8);
    for b in 0..s.count {
        if !gone.contains(&b) {
            boxes.extend_from_slice(s.box_at(b));
        }
    }
    for wall in &walls {
        wall.steps(&mut boxes);
    }
    (make_solids(s.w, s.h, s.cell, boxes), walls)
}
