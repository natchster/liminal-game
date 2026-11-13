//! Batch `t14`'s teddy's torn-off arm, made real (`docs/design/PROPS.md` P3; the
//! Blender batch `assets/src/blender/props/t14.py`, its notes
//! `t14.notes.md`): its placer and what only it uses. It keeps the old
//! placer's draws, call for call.
use super::*;
use t14_kit::{fbm, fill, mix, put, rgb, salt, ss, vn};

/// The old fur colours, and the plush each stands for (sRGB): a caramel, a
/// dark brown, a chocolate
const FUR_COLORS: [u32; 3] = [0x8a5030, 0x604020, 0x503020];
const PLUSH: [u32; 3] = [0x86603f, 0x5c4331, 0x4a3628];

// A teddy's arm: 6% chance per level. A plush teddy's arm (a 40 cm
// bear's: acrylic pile sewn round its edge, a velour pad on its paw) torn
// off at its shoulder seam, lying on its side: the plush ragged at the
// tear, the seam's threads broken, the polyester fibre stuffing pulled out
// of it, three tufts of it lying near. Matted, faded, grimed.
pub(crate) fn place_teddy_paws(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let col = pick_hex(&FUR_COLORS);
    let k = FUR_COLORS.iter().position(|&c| c == col).unwrap_or(0);
    let s = salt(g.wx(x), g.wz(z), 515);
    let base = rgb(PLUSH[k]);
    let paw = group();
    let arm = group();
    put(&arm, "Real__teddyArmFur", &lambert!({ map: &plush_tex(s, base) }));
    put(&arm, "Real__teddyArmPile", &lambert!({ map: &plush_pile_tex(s, base), alphaTest: 0.5 }));
    put(&arm, "Real__teddyArmPad", &lambert!({ map: &velour_tex(s, [0.70, 0.62, 0.51]) }));
    let fibre = lambert!({ map: &stuffing_tex(s) });
    let wisps = lambert!({ map: &wisps_tex(s), alphaTest: 0.5 });
    put(&arm, "Real__teddyArmStuffing", &fibre);
    put(&arm, "Real__teddyArmStuffingWisps", &wisps);
    paw.add(&arm);
    // the stuffing's tufts (the old puffs' draws: their size and turn, and
    // where they lie, past the torn end)
    for _ in 0..3 {
        let (a, b) = (rnd(), rnd());
        let (tx, tz) = ((rnd() - 0.5) * 0.08, 0.04 + (rnd() - 0.5) * 0.04);
        let tuft = group();
        put(&tuft, "Real__teddyArmTuft", &fibre);
        put(&tuft, "Real__teddyArmTuftWisps", &wisps);
        tuft.scale().set_scalar(0.8 + 0.4 * a);
        tuft.rotation().y = b * PI * 2.0;
        tuft.position().set(-0.115 + tx * 0.9, 0, (tz - 0.04) * 3.0);
        paw.add(&tuft);
    }
    scatter_on_floor(g, &paw, x, z, 2.4);
    // (the old tilt's draw: the arm turned a little from its tufts)
    arm.rotation().y = (rnd() - 0.5) * 0.4;
    g.world.add(&paw);
}

/// Acrylic plush pile (`b`): short, in fine streaks, matted flat in
/// patches, faded, grimed
fn plush_tex(salt: u32, b: [f64; 3]) -> Tex {
    canvas_tex_opt(128, 128, false, move |ctx, w, h| {
        let seed = salt ^ 0x9b1;
        fill(ctx, w, h, |u, v| {
            let streak = fbm(seed, u * 70.0, v * 35.0, 3);
            let mut c = mix(b, [b[0] * 0.66, b[1] * 0.64, b[2] * 0.62], ss(0.42, 0.72, streak) * 0.45);
            let tip = vn(seed ^ 2, u * 110.0, v * 90.0);
            c = mix(c, [b[0] * 1.18 + 0.03, b[1] * 1.16 + 0.03, b[2] * 1.14 + 0.03], ss(0.7, 0.95, tip) * 0.4);
            c = mix(c, [b[0] * 0.78, b[1] * 0.76, b[2] * 0.74], matted(seed, u, v) * 0.6);
            let fade = fbm(seed ^ 3, u * 2.0, v * 2.0, 3);
            c = mix(c, [0.60, 0.57, 0.52], ss(0.5, 0.8, fade) * 0.25);
            let grime = fbm(seed ^ 6, u * 3.0, v * 3.0, 3);
            c = mix(c, [0.32, 0.28, 0.24], ss(0.55, 0.85, grime) * 0.4);
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// Where the plush is matted flat (0 to 1)
fn matted(seed: u32, u: f64, v: f64) -> f64 {
    ss(0.6, 0.72, fbm(seed ^ 4, u * 3.0, v * 3.0, 4))
}

/// The pile's tips (the stippled layer): a third of the fibres' ends, none
/// where it is matted flat
fn plush_pile_tex(salt: u32, b: [f64; 3]) -> Tex {
    canvas_tex_opt(128, 128, false, move |ctx, w, h| {
        let seed = salt ^ 0x9b1;
        fill(ctx, w, h, |u, v| {
            let n = vn(seed ^ 0x31, u * 85.0, v * 75.0);
            let m = fbm(seed ^ 0x32, u * 10.0, v * 10.0, 2);
            let a = if n > 0.6 - 0.12 * (m - 0.5) + matted(seed, u, v) * 0.6 { 1.0 } else { 0.0 };
            let c = mix([b[0] * 1.1 + 0.02, b[1] * 1.08 + 0.02, b[2] * 1.06 + 0.02], [0.55, 0.52, 0.47], 0.12 + 0.2 * m);
            [c[0], c[1], c[2], a]
        });
    })
}

/// The paw pad's velour (`b`): its nap in patches, grimed
fn velour_tex(salt: u32, b: [f64; 3]) -> Tex {
    canvas_tex_opt(64, 64, false, move |ctx, w, h| {
        let seed = salt ^ 0x7e1;
        fill(ctx, w, h, |u, v| {
            let nap = fbm(seed, u * 6.0, v * 6.0, 3);
            let mut c = mix(b, [b[0] * 0.82, b[1] * 0.8, b[2] * 0.78], ss(0.4, 0.7, nap) * 0.6);
            let grime = fbm(seed ^ 1, u * 4.0, v * 4.0, 3);
            c = mix(c, [0.38, 0.33, 0.28], ss(0.45, 0.8, grime) * 0.5);
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// Polyester fibre stuffing: white gone grey with the floor's dust, its
/// fibres in fine curled wisps, darker in its folds
fn stuffing_tex(salt: u32) -> Tex {
    canvas_tex_opt(128, 128, false, move |ctx, w, h| {
        let seed = salt ^ 0x5f1;
        fill(ctx, w, h, |u, v| {
            let b = [0.84, 0.83, 0.80];
            let wisp = fbm(seed, u * 40.0 + 6.0 * fbm(seed ^ 1, u * 6.0, v * 6.0, 2), v * 40.0, 3);
            let mut c = mix(b, [0.70, 0.68, 0.64], ss(0.35, 0.7, wisp) * 0.5);
            let dust = fbm(seed ^ 2, u * 4.0, v * 4.0, 3);
            c = mix(c, [0.55, 0.52, 0.47], ss(0.55, 0.8, dust) * 0.45);
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// The stuffing's loose fibres (its stippled outer layer): wisps, most of
/// it open
fn wisps_tex(salt: u32) -> Tex {
    canvas_tex_opt(128, 128, false, move |ctx, w, h| {
        let seed = salt ^ 0x5f3;
        fill(ctx, w, h, |u, v| {
            let f = fbm(seed, u * 30.0 + 8.0 * fbm(seed ^ 1, u * 5.0, v * 5.0, 2), v * 90.0, 2);
            let a = if (f - 0.5).abs() < 0.06 { 1.0 } else { 0.0 };
            let c = mix([0.86, 0.85, 0.82], [0.62, 0.60, 0.56], vn(seed ^ 2, u * 8.0, v * 8.0) * 0.7);
            [c[0], c[1], c[2], a]
        });
    })
}

/// What this batch's painters share in this crate
mod t14_kit {
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
    pub(super) fn ss(e0: f64, e1: f64, x: f64) -> f64 {
        let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    }
    pub(super) fn mix(a: [f64; 3], b: [f64; 3], t: f64) -> [f64; 3] {
        let t = t.clamp(0.0, 1.0);
        [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
    }
    /// A colour (0xrrggbb) as fractions
    pub(super) fn rgb(c: u32) -> [f64; 3] {
        [f64::from((c >> 16) & 255) / 255.0, f64::from((c >> 8) & 255) / 255.0, f64::from(c & 255) / 255.0]
    }
    /// Every pixel from where it is (u across, v up), on every core
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
    /// A salt from where a thing is (no draw): its painters differ from
    /// floor to floor
    pub(super) fn salt(x: f64, z: f64, k: u32) -> u32 {
        ((x * 7919.0).abs() as u32) ^ ((z * 104_729.0).abs() as u32).rotate_left(13) ^ k.wrapping_mul(0x9e37_79b9)
    }
    /// The part `name` of a Blender batch as modelled, added to `grp`
    pub(super) fn put(grp: &Obj, name: &str, mat: &Mat) -> Option<Obj> {
        let geo = game::assets::props::prop_part_as_modelled(name)?;
        let m = mesh(geo, mat.clone());
        grp.add(&m);
        Some(m)
    }
}
