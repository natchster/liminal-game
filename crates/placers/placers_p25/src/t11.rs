//! Batch `t11`'s folding fan, made real (`docs/PROPS.md` P3; the Blender
//! batch `assets/src/blender/props/t11.py`, its notes `t11.notes.md`): its
//! placer and what only it uses. It keeps the old placer's draws, call for
//! call.
use crate::{open_tiles, pick};
use game::prelude::*;
use t11_kit::*;

/// The leaf's bounds as modelled (`Real__fanLeaf`, x and z): where each
/// texel lies on the flat leaf
const LEAF_X: (f64, f64) = (-0.159_754, 0.159_741);
const LEAF_Z: (f64, f64) = (0.048_685, 0.194_998);
/// The leaf's inner and outer radii, its opening (`t11.py` `FAN_*`)
const LEAF_R: (f64, f64) = (0.085, 0.195);
const FAN_OPEN: f64 = 110.0 * PI / 180.0;
/// The pleats: twice the ribs less one
const PANELS: f64 = 48.0;

// R441 small folding fan: 6% chance per level. A lady's paper folding fan
// (a sensu, 19.5 cm): 25 bamboo ribs riveted at their heads, the two
// lacquered guards, the leaf of washi folded in pleats over them, printed
// (the old colour pick: cherry blossom on rose, waves on indigo, maple
// leaves on gold, bamboo on celadon, wisteria on violet, a mountain in ink
// on cream); dropped open on the floor, its paper yellowed, foxed and
// rubbed along its folds, torn between two ribs.
pub fn place_small_folding_fans(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    // the old colour pick: the leaf's print
    let design = pick(&[0usize, 1, 2, 3, 4, 5]);
    let fan = group();
    let px = g.wx(x) + (rnd() - 0.5) * 2.4;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.4;
    let s = salt(px, pz, 25);
    put(&fan, "Real__fanLeaf", &lambert!({ map: &leaf_tex(s, design) }));
    // smoked bamboo for the darker prints, natural for the pale
    let rib = if matches!(design, 1 | 4) { 0x3a2a1c } else { 0x7a6040 };
    put(&fan, "Real__fanRibs", &phong!({ color: rib, specular: 0x1a1814, shininess: 24 }));
    put(&fan, "Real__fanRivet", &phong!({ color: 0x6a5838, specular: 0x5a5040, shininess: 50 }));
    fan.position().set(px, 0, pz);
    fan.rotation().y = rnd() * PI * 2.0;
    g.world.add(&fan);
}

/// Motifs scattered over the flat leaf: how much of a motif covers (x, z)
/// (cells of `cell` m), its own distance, angle and random value
fn motif(seed: u32, x: f64, z: f64, cell: f64, keep: f64) -> (f64, f64, f64) {
    let (gx, gz) = (x / cell, z / cell);
    let (ix, iz) = (gx.floor() as i32, gz.floor() as i32);
    let mut best = (9.0, 0.0, 0.0);
    for j in -1..=1 {
        for i in -1..=1 {
            let (cx, cz) = (ix + i, iz + j);
            if hash(seed, cx, cz) > keep {
                continue;
            }
            let ox = cx as f64 + 0.15 + 0.7 * hash(seed ^ 0x51, cx, cz);
            let oz = cz as f64 + 0.15 + 0.7 * hash(seed ^ 0x93, cx, cz);
            let (dx, dz) = ((gx - ox) * cell, (gz - oz) * cell);
            let d = dx.hypot(dz);
            if d < best.0 {
                best = (d, dz.atan2(dx) + hash(seed ^ 0x3c, cx, cz) * 6.283, hash(seed ^ 0x2d, cx, cz));
            }
        }
    }
    best
}

/// The leaf's print, laid flat (u across, v from the leaf's inner edge to
/// its outer as seen from above): its design over its ground; the pleats
/// shading in turn; rubbed pale along the mountain folds; yellowed, foxed,
/// a tear between two ribs at the edge; dust
fn leaf_tex(salt: u32, design: usize) -> Tex {
    canvas_tex_opt(256, 128, false, move |ctx, w, h| {
        let seed = salt ^ 0x4c3;
        fill(ctx, w, h, |u, v| {
            let x = LEAF_X.0 + (LEAF_X.1 - LEAF_X.0) * u;
            let z = LEAF_Z.1 - (LEAF_Z.1 - LEAF_Z.0) * v;
            let r = x.hypot(z);
            let th = x.atan2(z);
            let rr = ((r - LEAF_R.0) / (LEAF_R.1 - LEAF_R.0)).clamp(0.0, 1.0);
            let mut c;
            match design {
                0 => {
                    // cherry blossom on rose, flecks of gold
                    c = [0.52, 0.24, 0.27];
                    let (d, a, k) = motif(seed, x, z, 0.022, 0.75);
                    let petal = 0.0075 * (0.62 + 0.38 * (2.5 * a).cos().abs());
                    let fl = 1.0 - sstep(petal * 0.85, petal, d);
                    c = mix(c, if k < 0.5 { [0.86, 0.76, 0.76] } else { [0.80, 0.62, 0.64] }, fl);
                    c = mix(c, [0.62, 0.28, 0.30], 1.0 - sstep(0.0012, 0.002, d));
                    let g = vn(seed ^ 3, x * 900.0, z * 900.0);
                    c = mix(c, [0.66, 0.54, 0.30], sstep(0.88, 0.95, g) * 0.8);
                }
                1 => {
                    // blue waves (seigaiha) on indigo
                    c = [0.15, 0.19, 0.32];
                    let cell = 0.016;
                    let (gx, gz) = (x / cell, z / (cell * 0.5));
                    let row = gz.floor();
                    let off = if (row as i64).rem_euclid(2) == 0 { 0.0 } else { 0.5 };
                    let cx = (gx + off).floor() - off + 0.5;
                    let (dx, dz) = ((gx - cx) * cell, (gz - row) * cell * 0.5);
                    let d = dx.hypot(dz);
                    let ring = ((d / 0.0021).fract() - 0.5).abs();
                    let inside = 1.0 - sstep(0.0072, 0.0078, d);
                    c = mix(c, [0.70, 0.74, 0.80], (1.0 - sstep(0.12, 0.22, ring)) * inside * 0.85);
                }
                2 => {
                    // maple leaves on gold leaf gone dull
                    c = [0.62, 0.50, 0.27];
                    c = mix(c, [0.48, 0.38, 0.20], fbm(seed ^ 5, x * 60.0, z * 60.0, 3) * 0.6);
                    let (d, a, k) = motif(seed, x, z, 0.026, 0.6);
                    // a maple leaf: five pointed lobes
                    let lobe = 0.0105 * (0.42 + 0.58 * (1.0 - (2.5 * a).sin().abs()).powi(2));
                    let lf = 1.0 - sstep(lobe * 0.9, lobe, d);
                    c = mix(c, if k < 0.5 { [0.52, 0.18, 0.11] } else { [0.60, 0.30, 0.12] }, lf);
                }
                3 => {
                    // bamboo on celadon: stalks up the leaf, their nodes, leaves
                    c = [0.60, 0.65, 0.52];
                    for (k, a0) in [-0.62f64, -0.18, 0.31, 0.7].iter().enumerate() {
                        let off = (th - a0) * r;
                        let stalk = 1.0 - sstep(0.0035, 0.0045, off.abs());
                        let node = 1.0 - sstep(0.0, 0.0012, ((r + k as f64 * 0.013) % 0.034 - 0.017).abs());
                        c = mix(c, [0.22, 0.34, 0.20], stalk);
                        c = mix(c, [0.12, 0.20, 0.12], stalk * node);
                    }
                    let (d, a, _) = motif(seed, x, z, 0.02, 0.55);
                    let leaf = (d * a.cos() / 0.011).powi(2) + (d * a.sin() / 0.0028).powi(2);
                    c = mix(c, [0.20, 0.36, 0.20], 1.0 - sstep(0.8, 1.0, leaf));
                }
                4 => {
                    // wisteria hanging in clusters on violet
                    c = [0.36, 0.27, 0.40];
                    let (d, a, k) = motif(seed, x, z, 0.03, 0.8);
                    // a cluster: a drop of florets hanging out from the rivet
                    let along = d * (a - th).cos();
                    let across = d * (a - th).sin();
                    let drop = (along / 0.013).powi(2) + (across / (0.006 * (1.0 - along / 0.02).clamp(0.2, 1.4))).powi(2);
                    let floret = vn(seed ^ 7, x * 700.0, z * 700.0);
                    let m = (1.0 - sstep(0.7, 1.0, drop)) * sstep(0.35, 0.5, floret);
                    c = mix(c, if k < 0.6 { [0.74, 0.67, 0.82] } else { [0.86, 0.84, 0.86] }, m);
                }
                _ => {
                    // a mountain in ink on cream: ridges washed grey, paler further
                    c = [0.84, 0.80, 0.70];
                    for (k, base) in [0.25f64, 0.45, 0.62].iter().enumerate() {
                        let ridge = base + 0.22 * fbm(seed ^ (k as u32 * 13), th * 2.2 + k as f64, 0.5, 3) - 0.1;
                        let ink = sstep(ridge + 0.02, ridge - 0.12, rr) * (1.0 - sstep(ridge - 0.12, ridge - 0.4, rr) * 0.6);
                        c = mix(c, [0.30, 0.30, 0.30], ink * (0.75 - 0.2 * k as f64));
                    }
                    // a pine's dots on the near ridge
                    let p = vn(seed ^ 9, x * 500.0, z * 500.0);
                    c = mix(c, [0.16, 0.16, 0.15], sstep(0.85, 0.92, p) * (1.0 - sstep(0.3, 0.4, rr)));
                }
            }
            // the pleats: each panel a shade in turn, rubbed pale on the folds
            let pf = (th + FAN_OPEN / 2.0) / FAN_OPEN * PANELS;
            let pi = pf.floor() as i64;
            let along = pf.fract();
            c = if pi.rem_euclid(2) == 0 { [c[0] * 1.04, c[1] * 1.04, c[2] * 1.04] } else { [c[0] * 0.93, c[1] * 0.93, c[2] * 0.93] };
            let fold = 1.0 - sstep(0.0, 0.12, along.min(1.0 - along));
            c = mix(c, [0.80, 0.76, 0.66], fold * 0.25 * rr);
            // the paper yellowed and foxed; the edge grimy
            c = mix(c, [0.72, 0.62, 0.42], 0.12);
            let fox = vn(seed ^ 11, x * 300.0, z * 300.0);
            c = mix(c, [0.52, 0.38, 0.22], sstep(0.86, 0.94, fox) * 0.5);
            c = mix(c, [0.30, 0.25, 0.20], sstep(0.9, 1.0, rr) * 0.35);
            // a tear between two ribs, from the edge in
            let tear = (pf - 31.5).abs() * 0.0016 / 1.0;
            c = mix(c, [0.10, 0.08, 0.06], (1.0 - sstep(0.0, 0.0006, tear)) * sstep(0.62, 0.7, rr));
            // dust
            let d = fbm(seed ^ 13, x * 80.0, z * 80.0, 2);
            c = mix(c, [0.56, 0.53, 0.48], 0.08 + 0.15 * sstep(0.5, 0.8, d));
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// The painting kit of batch `t11`'s painters (as batches `t08`'s and
/// `t09`'s): value noise and a per-pixel fill (pure, so `par_rows` may run
/// it on every core), a salt from where a thing is, a part placed.
#[allow(dead_code)]
mod t11_kit {
    use game::prelude::*;

    pub(super) fn hash(seed: u32, i: i32, j: i32) -> f64 {
        let mut h = seed ^ (i as u32).wrapping_mul(0x27d4_eb2d) ^ (j as u32).wrapping_mul(0x1656_67b1);
        h = (h ^ (h >> 15)).wrapping_mul(0x85eb_ca6b);
        h = (h ^ (h >> 13)).wrapping_mul(0xc2b2_ae35);
        h ^= h >> 16;
        f64::from(h) / 4_294_967_296.0
    }
    pub(super) fn vn(seed: u32, x: f64, y: f64) -> f64 {
        let (xf, yf) = (x.floor(), y.floor());
        let (i, j) = (xf as i32, yf as i32);
        let (u, v) = (x - xf, y - yf);
        let (u, v) = (u * u * (3.0 - 2.0 * u), v * v * (3.0 - 2.0 * v));
        let (a, b) = (hash(seed, i, j), hash(seed, i + 1, j));
        let (c, d) = (hash(seed, i, j + 1), hash(seed, i + 1, j + 1));
        a + (b - a) * u + (c - a) * v + (a - b - c + d) * u * v
    }
    pub(super) fn fbm(seed: u32, x: f64, y: f64, oct: u32) -> f64 {
        let (mut s, mut amp, mut f, mut n) = (0.0, 0.5, 1.0, 0.0);
        for k in 0..oct {
            s += amp * vn(seed.wrapping_add(k * 7919), x * f, y * f);
            n += amp;
            amp *= 0.5;
            f *= 2.03;
        }
        s / n
    }
    pub(super) fn sstep(e0: f64, e1: f64, x: f64) -> f64 {
        let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    }
    pub(super) fn mix(a: [f64; 3], b: [f64; 3], t: f64) -> [f64; 3] {
        let t = t.clamp(0.0, 1.0);
        [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
    }
    /// Fills the canvas pixel by pixel on every core: `f(u, v)` (u right, v
    /// up, both 0..1, as the texture is laid) gives straight sRGB and alpha
    pub(super) fn fill(ctx: &mut Canvas, w: f64, h: f64, f: impl Fn(f64, f64) -> [f64; 4] + Sync) {
        let iw = w as usize;
        let mut img = ctx.create_image_data(w, h);
        par_rows(&mut img.data, iw * 4, |j, out| {
            for i in 0..iw {
                let c = f((i as f64 + 0.5) / w, 1.0 - (j as f64 + 0.5) / h);
                for k in 0..4 {
                    out[i * 4 + k] = (c[k].clamp(0.0, 1.0) * 255.0).round() as u8;
                }
            }
        });
        ctx.put_image_data(&img, 0, 0);
    }
    /// A salt from where a thing is (no draw): its painters' strokes differ
    /// from floor to floor
    pub(super) fn salt(x: f64, z: f64, k: u32) -> u32 {
        ((x * 7919.0).abs() as u32) ^ ((z * 104_729.0).abs() as u32).rotate_left(13) ^ k.wrapping_mul(0x9e37_79b9)
    }
    /// The part `name` of a Blender batch as modelled, added to `grp` in `mat`
    pub(super) fn put(grp: &Obj, name: &str, mat: &Mat) -> Option<Obj> {
        let geo = game::assets::props::prop_part_as_modelled(name)?;
        let m = mesh(geo, mat.clone());
        grp.add(&m);
        Some(m)
    }
}
