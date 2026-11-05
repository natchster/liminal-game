//! Batch `t16`'s prop in this crate, made real (`docs/PROPS.md` P3, "toys
//! stay toys"): the jump rope, a braided cotton skipping rope with turned
//! beech handles, dropped in loose loops (`assets/src/blender/props/t16.py`,
//! `t16.notes.md`). Its placer and what only it uses; it keeps the old
//! placer's draws, call for call.
use super::{jitter, pick_open};
use game::assets::props::prop_part_as_modelled;
use game::prelude::*;

/// R624 jump rope: 5% chance per level. A child's skipping rope (Blender
/// batch `t16`): 2.3 m of 8 mm braided cotton between two turned beech
/// handles (120 x 25 mm), wound round a hand once and dropped: three slack
/// loops come apart and cross each other, the handles out of the coil side
/// by side. The dye faded and grimed, the lacquer of the handles worn to
/// grey wood where they were held.
pub(super) fn place_jump_ropes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    // the old rope colour: the cotton's dye, faded
    let rope_colors = [0xc04020, 0x2060c0, 0x40a040, 0xf0e020, 0xd050a0];
    let old = choice(&rope_colors);
    let k = rope_colors.iter().position(|&c| c == old).unwrap_or(0);
    let dye = [[0.55, 0.30, 0.24], [0.30, 0.37, 0.50], [0.36, 0.45, 0.31], [0.70, 0.62, 0.38], [0.64, 0.44, 0.49]][k];
    let rope = group();
    let (px, pz) = jitter(g, x, z, 2.4);
    let salt = t16_salt(px, pz);
    t16_put(&rope, "Real__jumpRopeCord", &lambert!({ map: &cotton_tex(dye, salt) }));
    t16_put(&rope, "Real__jumpRopeHandles", &lambert!({ map: &beech_tex(salt) }));
    rope.position().set(px, 0, pz);
    rope.rotation().y = rnd() * PI * 2.0;
    g.world.add(&rope);
}

/// The cotton, laid on the cord from above (its bounds): the dye mottled
/// where it faded and where it was wet, the braid's fine light and dark,
/// grey grime and floor dust in patches
fn cotton_tex(dye: [f64; 3], salt: u32) -> Tex {
    t16_paint(128, 128, move |u, v| {
        let n = t16_fbm(salt, u * 7.0, v * 7.0, 3);
        let braid = t16_vn(salt ^ 0x51, u * 160.0, v * 160.0);
        let mut c = t16_mix(dye, [0.62, 0.60, 0.55], 0.25 * t16_ss(0.45, 0.75, n));
        c = t16_mix(c, [0.0, 0.0, 0.0], 0.10 + 0.18 * braid);
        let grime = t16_ss(0.5, 0.8, t16_fbm(salt ^ 0x77, u * 4.0, v * 4.0, 3));
        t16_mix(c, [0.30, 0.28, 0.25], 0.45 * grime)
    })
}

/// Beech under a worn lacquer: its fine grain, grey where hands wore the
/// lacquer off, dark grime in the turnings
fn beech_tex(salt: u32) -> Tex {
    t16_paint(128, 128, move |u, v| {
        let grain = t16_fbm(salt ^ 0x3c, u * 6.0, v * 90.0, 3);
        let mut c = t16_mix([0.62, 0.48, 0.33], [0.50, 0.37, 0.24], grain);
        let worn = t16_ss(0.55, 0.75, t16_fbm(salt ^ 0x19, u * 5.0, v * 5.0, 3));
        c = t16_mix(c, [0.55, 0.52, 0.47], 0.55 * worn);
        t16_mix(c, [0.24, 0.20, 0.16], 0.35 * t16_ss(0.6, 0.85, t16_fbm(salt ^ 0x2b, u * 12.0, v * 12.0, 2)))
    })
}

// ---- what only these use ----

/// A salt from where the rope lies (no draw): its wear differs from floor
/// to floor
fn t16_salt(x: f64, z: f64) -> u32 {
    let (a, b) = ((x * 1000.0).round() as i64 as u64, (z * 1000.0).round() as i64 as u64);
    let mut h = a.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ b.wrapping_mul(0xC2B2_AE3D_27D4_EB4F);
    h ^= h >> 29;
    (h ^ (h >> 32)) as u32
}

/// The part `name` of batch `t16` as modelled, added to `grp` in `mat`
fn t16_put(grp: &Obj, name: &str, mat: &Mat) {
    if let Some(geo) = prop_part_as_modelled(name) {
        grp.add(&mesh(geo, mat.clone()));
    }
}

/// A texture painted pixel by pixel (sRGB) from `f(u, v)` (v down the
/// image), on every core: no canvas call and no draw inside
fn t16_paint(w: usize, h: usize, f: impl Fn(f64, f64) -> [f64; 3] + Sync + Send + 'static) -> Tex {
    canvas_tex_opt(w as f64, h as f64, false, move |ctx, cw, ch| {
        let mut img = ctx.create_image_data(cw, ch);
        par_rows(&mut img.data, w * 4, |j, out| {
            for i in 0..w {
                let c = f((i as f64 + 0.5) / w as f64, (j as f64 + 0.5) / h as f64);
                for k in 0..3 {
                    out[i * 4 + k] = (c[k].clamp(0.0, 1.0) * 255.0).round() as u8;
                }
                out[i * 4 + 3] = 255;
            }
        });
        ctx.put_image_data(&img, 0, 0);
    })
}

fn t16_ss(e0: f64, e1: f64, x: f64) -> f64 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn t16_mix(a: [f64; 3], b: [f64; 3], t: f64) -> [f64; 3] {
    let t = t.clamp(0.0, 1.0);
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
}

fn t16_hash(seed: u32, i: i32, j: i32) -> f64 {
    let mut h = seed ^ (i as u32).wrapping_mul(0x27d4_eb2d) ^ (j as u32).wrapping_mul(0x1656_67b1);
    h = (h ^ (h >> 15)).wrapping_mul(0x85eb_ca6b);
    h = (h ^ (h >> 13)).wrapping_mul(0xc2b2_ae35);
    h ^= h >> 16;
    f64::from(h) / 4_294_967_296.0
}

fn t16_vn(seed: u32, x: f64, y: f64) -> f64 {
    let (xf, yf) = (x.floor(), y.floor());
    let (i, j) = (xf as i32, yf as i32);
    let (u, v) = (x - xf, y - yf);
    let (u, v) = (u * u * (3.0 - 2.0 * u), v * v * (3.0 - 2.0 * v));
    let (a, b) = (t16_hash(seed, i, j), t16_hash(seed, i + 1, j));
    let (c, d) = (t16_hash(seed, i, j + 1), t16_hash(seed, i + 1, j + 1));
    a + (b - a) * u + (c - a) * v + (a - b - c + d) * u * v
}

fn t16_fbm(seed: u32, x: f64, y: f64, oct: u32) -> f64 {
    let (mut s, mut amp, mut f, mut n) = (0.0, 0.5, 1.0, 0.0);
    for k in 0..oct {
        s += amp * t16_vn(seed.wrapping_add(k * 7919), x * f, y * f);
        n += amp;
        amp *= 0.5;
        f *= 2.03;
    }
    s / n
}
