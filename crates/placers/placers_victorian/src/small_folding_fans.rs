//! `place_small_folding_fans`
use game::prelude::*;
use placers_kit::noise::fbm;
use placers_kit::noise::hash;
use placers_kit::noise::mix;
use placers_kit::noise::sstep;
use placers_kit::noise::vn;
use placers_kit::paint::fill;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;
use placers_kit::place::salt;

// Small folding fan: 6% chance per level. A lady's paper folding fan
// (a sensu, 19.5 cm): 25 bamboo ribs riveted at their heads, the two
// lacquered guards, the leaf of washi folded in pleats over them, printed
// (the old colour pick: cherry blossom on rose, waves on indigo, maple
// leaves on gold, bamboo on celadon, wisteria on violet, a mountain in ink
// on cream); dropped open on the floor, its paper yellowed, foxed and
// rubbed along its folds, torn between two ribs.
pub(crate) fn place_small_folding_fans(g: &mut Game, _dist: &[i32]) {
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

/// The leaf's bounds as modelled (`Real__fanLeaf`, x and z): where each
/// texel lies on the flat leaf
const LEAF_X: (f64, f64) = (-0.159_754, 0.159_741);

const LEAF_Z: (f64, f64) = (0.048_685, 0.194_998);

/// The leaf's inner and outer radii, its opening (`t11.py` `FAN_*`)
const LEAF_R: (f64, f64) = (0.085, 0.195);

const FAN_OPEN: f64 = 110.0 * PI / 180.0;

/// The pleats: twice the ribs less one
const PANELS: f64 = 48.0;

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
                    // white waves (seigaiha) on indigo: rows of nested arcs
                    c = [0.15, 0.19, 0.32];
                    let cell = 0.028;
                    let (gx, gz) = (x / cell, z / (cell * 0.5));
                    let row = gz.floor();
                    let off = if (row as i64).rem_euclid(2) == 0 { 0.0 } else { 0.5 };
                    let cx = (gx + off).floor() - off + 0.5;
                    let (dx, dz) = ((gx - cx) * cell, (gz - row) * cell * 0.5);
                    let d = dx.hypot(dz);
                    let ring = ((d / 0.0042).fract() - 0.5).abs();
                    let inside = 1.0 - sstep(0.0128, 0.0136, d);
                    c = mix(c, [0.70, 0.74, 0.80], (1.0 - sstep(0.16, 0.26, ring)) * inside * 0.85);
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
                    // wisteria on violet: racemes hanging from the leaf's top
                    // edge in toward the rivet, each a tapering drop of florets
                    c = [0.36, 0.27, 0.40];
                    let n = 9.0;
                    let slot = ((th + FAN_OPEN / 2.0) / FAN_OPEN * n).floor();
                    let mut m = 0.0f64;
                    for k in [slot - 1.0, slot, slot + 1.0] {
                        let ki = k as i32;
                        let tk = -FAN_OPEN / 2.0 + (k + 0.3 + 0.4 * hash(seed, ki, 1)) / n * FAN_OPEN;
                        let len = 0.035 + 0.03 * hash(seed, ki, 2);
                        let along = (LEAF_R.1 - 0.004 - r) / len;
                        if !(0.0..=1.0).contains(&along) {
                            continue;
                        }
                        let across = ((th - tk) * r).abs();
                        let wid = 0.0075 * (1.0 - along).powf(0.7) + 0.0012;
                        m = m.max(1.0 - sstep(wid * 0.8, wid, across));
                    }
                    let floret = vn(seed ^ 7, x * 800.0, z * 800.0);
                    c = mix(c, [0.70, 0.64, 0.80], m * sstep(0.3, 0.45, floret));
                    c = mix(c, [0.86, 0.84, 0.86], m * sstep(0.75, 0.85, floret));
                    // the vine's leaves along the top edge
                    let lf = vn(seed ^ 9, th * 40.0, rr * 6.0);
                    c = mix(c, [0.30, 0.38, 0.24], sstep(0.88, 0.97, rr) * sstep(0.45, 0.6, lf));
                }
                _ => {
                    // mountains in ink on cream: a far range pale and high,
                    // a near one dark and low, each wash darkest at its ridge
                    // and fading down into mist; a pine's dots on the near
                    c = [0.84, 0.80, 0.70];
                    for (k, (base, hgt, ink)) in [(0.52f64, 0.34f64, 0.35f64), (0.22, 0.30, 0.72)].into_iter().enumerate() {
                        let f = 2.2 + k as f64 * 1.3;
                        let tri = 1.0 - (2.0 * (th * f + 0.37 * k as f64 + hash(seed, k as i32, 3)).rem_euclid(1.0) - 1.0).abs();
                        let ridge = base + hgt * tri.powf(1.6) + 0.06 * (fbm(seed ^ (k as u32 * 13 + 1), th * 9.0, 0.5, 3) - 0.5);
                        let wash = sstep(ridge - 0.32, ridge - 0.015, rr) * (1.0 - sstep(ridge - 0.015, ridge + 0.005, rr));
                        c = mix(c, [0.24, 0.24, 0.24], wash * ink);
                    }
                    let p = vn(seed ^ 9, x * 500.0, z * 500.0);
                    c = mix(c, [0.16, 0.16, 0.15], sstep(0.86, 0.92, p) * (1.0 - sstep(0.3, 0.4, rr)));
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

/// The part `name` of a Blender batch as modelled, added to `grp` in `mat`
fn put(grp: &Obj, name: &str, mat: &Mat) -> Option<Obj> {
    let geo = game::assets::props::prop_part_as_modelled(name)?;
    let m = mesh(geo, mat.clone());
    grp.add(&m);
    Some(m)
}
