//! Value noise and the curves the placers paint with: a lattice hash, value noise, fractal sums of it, smooth steps, mixes.

/// The distance between the nearest and next nearest of the tile's cell
/// points (`n` cells across, repeating): small along the cells' edges
pub fn cell_edge(seed: u32, u: f64, v: f64, n: i32) -> f64 {
    let (x, y) = (u * f64::from(n), v * f64::from(n));
    let (i, j) = (x.floor() as i32, y.floor() as i32);
    let (mut d1, mut d2) = (9.0f64, 9.0f64);
    for dj in -1..=1 {
        for di in -1..=1 {
            let (ci, cj) = (i + di, j + dj);
            let (wi, wj) = (ci.rem_euclid(n), cj.rem_euclid(n));
            let fx = f64::from(ci) + 0.15 + 0.7 * hash(seed, wi, wj);
            let fy = f64::from(cj) + 0.15 + 0.7 * hash(seed ^ 0x5bd1, wi, wj);
            let d = ((x - fx).powi(2) + (y - fy).powi(2)).sqrt();
            if d < d1 {
                d2 = d1;
                d1 = d;
            } else if d < d2 {
                d2 = d;
            }
        }
    }
    d2 - d1
}

/// The distance to the nearest edge of a cell of a Voronoi field (cells
/// of size 1): cracks in old leather, varnish or enamel
pub fn crack(seed: u32, x: f64, y: f64) -> f64 {
    let (cx, cy) = (x.floor() as i32, y.floor() as i32);
    let (mut d1, mut d2) = (9.0f64, 9.0f64);
    for dy in -1..=1 {
        for dx in -1..=1 {
            let (ix, iy) = (cx + dx, cy + dy);
            let px = f64::from(ix) + 0.1 + 0.8 * hash(seed, ix, iy);
            let py = f64::from(iy) + 0.1 + 0.8 * hash(seed ^ 0x55, ix, iy);
            let d = ((x - px).powi(2) + (y - py).powi(2)).sqrt();
            if d < d1 {
                d2 = d1;
                d1 = d;
            } else if d < d2 {
                d2 = d;
            }
        }
    }
    (d2 - d1) * 0.5
}

/// Fractal value noise of `oct` octaves, in [0, 1]
pub fn fbm(seed: u32, x: f64, y: f64, oct: u32) -> f64 {
    let (mut s, mut amp, mut f, mut n) = (0.0, 0.5, 1.0, 0.0);
    for k in 0..oct {
        s += amp * vn(seed.wrapping_add(k * 7919), x * f, y * f);
        n += amp;
        amp *= 0.5;
        f *= 2.03;
    }
    s / n
}

/// Fractal noise over the tile (u, v in 0..1), `p` cells across at first
pub fn fbmp(seed: u32, u: f64, v: f64, p: i32, oct: u32) -> f64 {
    let (mut s, mut amp, mut n) = (0.0, 0.5, 0.0);
    for k in 0..oct {
        let q = p << k;
        s += amp * vnp(seed.wrapping_add(k * 7919), u * f64::from(q), v * f64::from(q), q, q);
        n += amp;
        amp *= 0.5;
    }
    s / n
}

/// A hash of an integer lattice point, in [0, 1)
pub fn hash(seed: u32, i: i32, j: i32) -> f64 {
    let mut h = seed ^ (i as u32).wrapping_mul(0x27d4_eb2d) ^ (j as u32).wrapping_mul(0x1656_67b1);
    h = (h ^ (h >> 15)).wrapping_mul(0x85eb_ca6b);
    h = (h ^ (h >> 13)).wrapping_mul(0xc2b2_ae35);
    h ^= h >> 16;
    f64::from(h) / 4_294_967_296.0
}

/// A thin line's darkness at distance d from it (soft, `w` wide)
pub fn line(d: f64, w: f64) -> f64 {
    (1.0 - (d / w).abs()).max(0.0)
}

/// Where the plush is rubbed flat (0 to 1)
pub fn matted(seed: u32, u: f64, v: f64) -> f64 {
    sstep(0.6, 0.72, fbm(seed ^ 4, u * 6.0, v * 6.0, 4))
}

/// `a` to `b` by `t`, clamped to 0..1: two colours (or any three numbers) mixed
pub fn mix(a: [f64; 3], b: [f64; 3], t: f64) -> [f64; 3] {
    let t = t.clamp(0.0, 1.0);
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
}

/// The smooth minimum of two distances, blended over `k`
pub fn smin(a: f64, b: f64, k: f64) -> f64 {
    let h = (0.5 + 0.5 * (b - a) / k).clamp(0.0, 1.0);
    b + (a - b) * h - k * h * (1.0 - h)
}

/// Specks: a dot in each `cell` (m) with likelihood `like`, its radius
/// `r0` + up to `r1`, anti-aliased over `aa`: (coverage, the dot's hash)
pub fn specks(seed: u32, x: f64, y: f64, cell: f64, like: f64, r0: f64, r1: f64, aa: f64) -> (f64, f64) {
    let (gi, gj) = ((x / cell).floor() as i32, (y / cell).floor() as i32);
    let (mut best, mut hk) = (0.0f64, 0.0);
    for k in 0..9 {
        let (ci, cj) = (gi + k % 3 - 1, gj + k / 3 - 1);
        let h = hash(seed, ci, cj);
        if h >= like {
            continue;
        }
        let (px, py) = ((f64::from(ci) + hash(seed ^ 0x77, ci, cj)) * cell, (f64::from(cj) + hash(seed ^ 0x78, ci, cj)) * cell);
        let r = r0 + r1 * hash(seed ^ 0x79, ci, cj);
        let cov = (0.5 - ((x - px).hypot(y - py) - r) / aa).clamp(0.0, 1.0);
        if cov > best {
            best = cov;
            hk = hash(seed ^ 0x7a, ci, cj);
        }
    }
    (best, hk)
}

/// 0 below `e0`, 1 above `e1`, smooth between
pub fn sstep(e0: f64, e1: f64, x: f64) -> f64 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// How a dried water mark shades at signed distance `d` (m, < 0 inside) from
/// one of its rims: sharp on its outside (over `aa`), what the water left
/// there fading inward over `ew`
pub fn tide(d: f64, aa: f64, ew: f64) -> f64 {
    (0.5 - d / aa).clamp(0.0, 1.0) * (d.min(0.0) / ew).exp()
}

/// Value noise in [0, 1]
pub fn vn(seed: u32, x: f64, y: f64) -> f64 {
    let (xf, yf) = (x.floor(), y.floor());
    let (i, j) = (xf as i32, yf as i32);
    let (u, v) = (x - xf, y - yf);
    let (u, v) = (u * u * (3.0 - 2.0 * u), v * v * (3.0 - 2.0 * v));
    let (a, b) = (hash(seed, i, j), hash(seed, i + 1, j));
    let (c, d) = (hash(seed, i, j + 1), hash(seed, i + 1, j + 1));
    a + (b - a) * u + (c - a) * v + (a - b - c + d) * u * v
}

/// Value noise over a lattice repeating every (px, py) cells
pub fn vnp(seed: u32, x: f64, y: f64, px: i32, py: i32) -> f64 {
    let (xf, yf) = (x.floor(), y.floor());
    let (i, j) = (xf as i32, yf as i32);
    let (u, v) = (x - xf, y - yf);
    let (u, v) = (u * u * (3.0 - 2.0 * u), v * v * (3.0 - 2.0 * v));
    let (i0, i1, j0, j1) = (i.rem_euclid(px), (i + 1).rem_euclid(px), j.rem_euclid(py), (j + 1).rem_euclid(py));
    let (a, b) = (hash(seed, i0, j0), hash(seed, i1, j0));
    let (c, d) = (hash(seed, i0, j1), hash(seed, i1, j1));
    a + (b - a) * u + (c - a) * v + (a - b - c + d) * u * v
}

/// The two nearest distances to a jittered lattice (cells of 1)
pub fn worley(seed: u32, x: f64, y: f64) -> (f64, f64) {
    let (mut d1, mut d2) = (9.0f64, 9.0f64);
    for dj in -1..=1 {
        for di in -1..=1 {
            let (ii, jj) = (x.floor() as i32 + di, y.floor() as i32 + dj);
            let fx = ii as f64 + hash(seed, ii, jj);
            let fy = jj as f64 + hash(seed ^ 77, ii, jj);
            let d = (fx - x).hypot(fy - y);
            if d < d1 {
                d2 = d1;
                d1 = d;
            } else if d < d2 {
                d2 = d;
            }
        }
    }
    (d1, d2)
}

pub fn mix_unclamped(a: [f64; 3], b: [f64; 3], t: f64) -> [f64; 3] {
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
}

/// A small generator for a mark's own layout, seeded from where it is
pub struct Rng32(pub u32);

impl Rng32 {
    pub fn u(&mut self) -> u32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.0 = x;
        x
    }
    /// in [0, 1)
    pub fn f(&mut self) -> f64 {
        f64::from(self.u()) / 4_294_967_296.0
    }
}
