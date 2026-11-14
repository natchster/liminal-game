//! Baked things (loaves, pies, muffins, firewood's bark): their own noise, and mould.

use engine::dmath::DMath;
/// A hash of an integer lattice point
pub fn hash(seed: u32, x: i32, y: i32) -> f64 {
    let mut h = seed ^ (x as u32).wrapping_mul(0x8da6_b343) ^ (y as u32).wrapping_mul(0xd816_3841);
    h = (h ^ (h >> 13)).wrapping_mul(0xcb1a_b31f);
    h ^= h >> 16;
    h as f64 / 4_294_967_296.0
}

/// Smooth value noise in [0, 1]
pub fn noise(seed: u32, x: f64, y: f64) -> f64 {
    let (xf, yf) = (x.floor(), y.floor());
    let (fx, fy) = (x - xf, y - yf);
    let (sx, sy) = (fx * fx * (3.0 - 2.0 * fx), fy * fy * (3.0 - 2.0 * fy));
    let (xi, yi) = (xf as i32, yf as i32);
    let (a, b) = (hash(seed, xi, yi), hash(seed, xi + 1, yi));
    let (c, d) = (hash(seed, xi, yi + 1), hash(seed, xi + 1, yi + 1));
    a + (b - a) * sx + (c - a) * sy + (a - b - c + d) * sx * sy
}

/// Three octaves of it
pub fn fbm(seed: u32, x: f64, y: f64) -> f64 {
    0.55 * noise(seed, x, y) + 0.3 * noise(seed ^ 0x51, x * 2.3, y * 2.3) + 0.15 * noise(seed ^ 0xa2, x * 5.1, y * 5.1)
}

pub fn mix(c: &mut [f64; 3], to: [f64; 3], k: f64) {
    let k = k.clamp(0.0, 1.0);
    for i in 0..3 {
        c[i] += (to[i] - c[i]) * k;
    }
}

/// Mould at a point, from the colonies [(x, y, radius)] (in px): ragged
/// patches, a grey-green spore mass in the middle, white cottony fuzz
/// round it and over it in wisps, a yellowed stain in the bread round it;
/// returns (spores, fuzz, stain)
pub fn mould(cols: &[(f64, f64, f64)], x: f64, y: f64, seed: u32) -> (f64, f64, f64) {
    let (mut s, mut wr, mut st) = (0.0f64, 0.0f64, 0.0f64);
    let warp = fbm(seed, x / 7.0, y / 7.0) - 0.5;
    let fuzz = noise(seed ^ 0x33, x / 1.6, y / 1.6);
    for &(cx, cy, cr) in cols {
        let d = ((x - cx).dpowi(2) + (y - cy).dpowi(2)).sqrt() / cr + 0.9 * warp;
        s = s.max(((0.55 - d) * 3.0).clamp(0.0, 1.0) * (0.6 + 0.4 * fuzz));
        wr = wr.max(((1.05 - d) * 2.2).clamp(0.0, 1.0) * (0.35 + 0.65 * fuzz));
        st = st.max(((1.5 - d) * 1.6).clamp(0.0, 1.0));
    }
    (s, wr, st)
}
