//! `place_cornbread_muffins`
use crate::shared::put;
use crate::shared::put_split;
use game::assets::props::prop_part_as_modelled;
use game::prelude::*;
use placers_kit::baking::mould as loaf_mould;
use placers_kit::noise::fbm;
use placers_kit::noise::mix;
use placers_kit::noise::sstep;
use placers_kit::noise::vn;
use placers_kit::paint::fill;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;
use placers_kit::place::salt;

// Cornbread muffin cluster: 6% chance per level. Three to five
// cornbread muffins left in their paper baking cups, years on: dried hard
// and shrunk from their cases, the crowns cracked, gone dull and grey,
// mould in soft colonies; the cases stained with grease; crumbs.
pub(crate) fn place_cornbread_muffins(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    // the old golden colours: which muffin (two shapes, as they split)
    let golden_colors: [u32; 4] = [0xe0b060, 0xd8a850, 0xd0a044, 0xc89840];
    let count = 3 + (rnd() * 3.0).floor() as i32;
    // each muffin's old draws: its colour, where round the middle, how far
    // out, its turn
    let mut each = vec![];
    for i in 0..count {
        let c = pick(&golden_colors);
        let k = golden_colors.iter().position(|&g| g == c).unwrap_or(0);
        let ang = (i as f64 / count as f64) * PI * 2.0 + rnd() * 0.3;
        let rr = 0.02 + rnd() * 0.06;
        let turn = rnd() * PI * 2.0;
        each.push((k, ang, rr, turn));
    }
    let px = g.wx(x) + (rnd() - 0.5) * 2.2;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.2;
    let yaw = rnd() * PI * 2.0;
    let salt = salt(px, pz, 680);
    let case = lambert!({ map: &case_tex(salt), side: DoubleSide });
    let crowns = [lambert!({ map: &muffin_tex(salt, 0) }), lambert!({ map: &muffin_tex(salt, 1) })];
    let cluster = group();
    // real muffins cannot sit 2 cm apart: the old ring opened out until
    // their cases clear each other, the old spread kept over that
    let ring = MUFFIN_CLEAR / (PI / f64::from(count)).sin();
    for &(k, ang, rr, turn) in &each {
        let muffin = group();
        put(&muffin, "Real__cornMuffinCase", &case);
        put_split(&muffin, if k % 2 == 0 { "Real__cornMuffinA" } else { "Real__cornMuffinB" }, &crowns[k % 2]);
        let r2 = ring + (rr - 0.02) * 0.3;
        muffin.position().set(ang.cos() * r2, 0, ang.sin() * r2);
        muffin.rotation().y = turn;
        cluster.add(&muffin);
    }
    if let Some(geo) = prop_part_as_modelled("Real__breadCrumbs") {
        let crumbs = mesh(geo, lambert!({ color: 0x9a7e50 }));
        crumbs.position().set(CRUMBS_AT.0, CRUMBS_AT.1, CRUMBS_AT.2);
        cluster.add(&crumbs);
    }
    cluster.position().set(px, 0, pz);
    cluster.rotation().y = yaw;
    g.world.add(&cluster);
}

/// The muffin's case: how far its ring had to open for real muffins to sit
/// side by side (a case's top is 71 mm across)
const MUFFIN_CLEAR: f64 = 0.0356;

/// The crumbs (`domestic/kitchen_and_hearth_left_behind.py`'s `Real__breadCrumbs`, on its board) moved to the
/// middle of the muffins, on the floor
const CRUMBS_AT: (f64, f64, f64) = (-0.066, -0.0178, 0.011);

/// The muffin's case as modelled (`party/a_party_tables_leavings.py`): its height
const CASE_H: f64 = 0.033;

/// The baking cups (round and up; the base below): plain white paper, kraft
/// brown, or printed with small dots; grease soaked through in blotches
/// with dark tide edges, most low down; the pleats' creases; dust on the
/// rim, grime at the foot
fn case_tex(salt: u32) -> Tex {
    canvas_tex_opt(128, 64, false, move |ctx, w, h| {
        let seed = salt ^ 0xcace;
        let kind = (salt >> 7) % 4;
        let paper = match kind {
            2 => [0.60, 0.48, 0.34],
            _ => [0.86, 0.82, 0.72],
        };
        fill(ctx, w, h, |u, v| {
            let y = v * CASE_H;
            let n = fbm(seed, u * 12.0, v * 6.0, 3);
            let mut c = mix(paper, [0.70, 0.67, 0.60], 0.25 * n);
            if kind == 3 {
                let (fx, fy) = ((u * 48.0).fract() - 0.5, (v * 6.0).fract() - 0.5);
                if fx * fx + fy * fy * 0.5 < 0.04 {
                    c = [0.74, 0.46, 0.44];
                }
            }
            // the pleats' creases
            c = mix(c, [0.0, 0.0, 0.0], 0.08 * (0.5 + 0.5 * (u * PI * 48.0).sin()));
            // grease: blotches, darker at their tide edges, most low down
            let gb = fbm(seed ^ 4, u * 9.0, v * 3.0, 3) + 0.25 * (1.0 - v);
            let blot = sstep(0.52, 0.62, gb);
            let tide = sstep(0.03, 0.0, (gb - 0.56).abs());
            c = mix(c, [0.66, 0.54, 0.32], blot * 0.55);
            c = mix(c, [0.46, 0.36, 0.20], tide * 0.4);
            // dust on the rim, grime at the foot
            c = mix(c, [0.60, 0.57, 0.50], 0.08 + 0.2 * sstep(0.025, 0.033, y));
            c = mix(c, [0.38, 0.35, 0.30], 0.4 * sstep(0.004, 0.0, y));
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// The cracks across each muffin's crown (`party/a_party_tables_leavings.py`'s MUFFIN_CRACKS: x0,
/// z0, x1, z1 in m) and the crown's half width
const MUFFIN_CRACKS: [&[(f64, f64, f64, f64)]; 2] = [
    &[(-0.024, 0.006, 0.022, -0.004), (0.004, 0.0, 0.010, 0.020)],
    &[(-0.018, -0.016, 0.0, 0.002), (0.0, 0.002, 0.021, -0.006), (0.0, 0.002, -0.004, 0.022)],
];

const CROWN_R: f64 = 0.033;

/// The muffin, laid as `put_split` lays it (its sides round and up below,
/// its crown from above in the upper left): cornbread's crust, golden gone
/// dull and dusty, pebbled with the coarse meal, browner at the shoulder;
/// the cracks across the crown (shape `k`'s) dark, the paler crumb at
/// their lips; mould colonies (the loaf's); dust on the crown
fn muffin_tex(salt: u32, k: usize) -> Tex {
    canvas_tex_opt(128, 128, false, move |ctx, w, h| {
        let seed = salt ^ 0x3fb ^ (k as u32 * 0x91);
        let mut r = SplitMix64::new(seed, 0x10af);
        let cols: Vec<(f64, f64, f64)> = (0..7)
            .map(|i| {
                if i < 4 {
                    (r.range(0.1, 0.9) * w * 0.5, r.range(0.05, 0.45) * h, if i < 1 { r.range(5.0, 8.0) } else { r.range(2.0, 4.0) })
                } else {
                    (r.next() * w, h * r.range(0.5, 0.62), r.range(2.0, 5.0))
                }
            })
            .collect();
        let bake = [0.74, 0.56, 0.28];
        fill(ctx, w, h, |u, v| {
            let (px, py) = (u * w, (1.0 - v) * h);
            let top = v >= 0.5;
            let n = fbm(seed, u * 10.0, v * 10.0, 3);
            let mut c = mix(bake, [0.56, 0.47, 0.34], 0.25 + 0.15 * n);
            // the coarse meal: flecks paler and darker
            let g1 = vn(seed ^ 2, px * 0.9, py * 0.9);
            let g2 = vn(seed ^ 3, px * 1.3 + 7.0, py * 1.3);
            c = mix(c, [0.84, 0.70, 0.42], sstep(0.70, 0.9, g1) * 0.5);
            c = mix(c, [0.36, 0.24, 0.12], sstep(0.74, 0.92, g2) * 0.45);
            if top {
                let (x, z) = (((v - 0.5) * 2.0 * 2.0 - 1.0) * CROWN_R, ((u * 2.0).fract() * 2.0 - 1.0) * CROWN_R);
                let rr = (x * x + z * z).sqrt() / CROWN_R;
                // browner toward the shoulder
                c = mix(c, [0.46, 0.32, 0.17], sstep(0.6, 0.95, rr) * 0.4);
                // the cracks: dark down in them, the crumb paler at their lips
                let mut dist = 1.0f64;
                for &(x0, z0, x1, z1) in MUFFIN_CRACKS[k] {
                    let (ax, az) = (x1 - x0, z1 - z0);
                    let t = (((x - x0) * ax + (z - z0) * az) / (ax * ax + az * az)).clamp(0.0, 1.0);
                    let d =
                        ((x - x0 - ax * t).powi(2) + (z - z0 - az * t).powi(2)).sqrt() + 0.0015 * (1.0 - (PI * (0.15 + 0.85 * t)).sin());
                    dist = dist.min(d);
                }
                let wob = 0.0005 * (vn(seed ^ 12, x * 900.0, z * 900.0) - 0.5);
                c = mix(c, [0.82, 0.68, 0.42], sstep(0.0028, 0.0016, dist + wob) * 0.6);
                c = mix(c, [0.24, 0.16, 0.08], sstep(0.0016, 0.0006, dist + wob) * 0.85);
                c = mix(c, [0.60, 0.57, 0.51], 0.16 + 0.12 * n);
            } else {
                // the side: the crumb paler down in the case, the crust above
                let y = v * 2.0;
                c = mix(c, [0.72, 0.60, 0.38], sstep(0.62, 0.5, y) * 0.5);
                c = mix(c, [0.60, 0.57, 0.51], 0.08);
            }
            let (s, wr, st) = loaf_mould(&cols, px, py, seed ^ 7);
            c = mix(c, [0.56, 0.50, 0.32], st * 0.3);
            c = mix(c, [0.82, 0.82, 0.78], wr * 0.6);
            let sp = vn(seed ^ 9, px / 1.2, py / 1.2);
            c = mix(c, [0.34 + 0.1 * sp, 0.40 + 0.08 * sp, 0.33 + 0.05 * sp], s * 0.9);
            [c[0], c[1], c[2], 1.0]
        });
    })
}
