//! Batch `t16`'s prop in this module, made real (`docs/PROPS.md` P3, "toys
//! stay toys"): the skipping rope, a PVC licorice rope with moulded
//! handles, dropped in a springy loop (`assets/src/blender/props/t16.py`,
//! `t16.notes.md`). Its placer and what only it uses; it keeps the old
//! placer's draws, call for call.
use crate::{open_tiles, pick};
use game::assets::props::prop_part_as_modelled;
use game::prelude::*;

/// R422 skipping rope: 8% chance per level. A PVC "licorice" skipping rope
/// (Blender batch `t16`): 2.4 m of 4.8 mm solid PVC through two moulded
/// polypropylene handles (114 mm), dropped with its handles together: the
/// rope springs out in one loop that crossed itself. The PVC faded and
/// dulled, grey with dust; the handles scuffed.
pub fn place_skipping_ropes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    // the old rope colour: the PVC's, faded; its handles moulded to match
    let rope_colors = [0xd82020, 0x2050d8, 0xd8b820, 0xf0f0f0];
    let old = pick(&rope_colors);
    let k = rope_colors.iter().position(|&c| c == old).unwrap_or(0);
    let pvc = [0x8a3c34, 0x3c5084, 0xae9a4c, 0xb8b4a8][k];
    let handles = [0xa48a46, 0x8a3a32, 0x3e5280, 0x8a3a32][k];
    let rope = group();
    let px = g.wx(x) + (rnd() - 0.5) * 2.2;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.2;
    let salt = t16_salt(px, pz);
    let dust = dust_tex(salt);
    t16_put(&rope, "Real__licoriceRopeCord", &phong!({ color: pvc, map: &dust, specular: 0x262626, shininess: 34 }));
    t16_put(&rope, "Real__licoriceRopeHandles", &phong!({ color: handles, map: &dust, specular: 0x1c1c1c, shininess: 22 }));
    rope.position().set(px, 0, pz);
    rope.rotation().y = rnd() * PI * 2.0;
    g.world.add(&rope);
}

/// Dust and grime over the plastic, laid on from above (a multiplier of
/// the plastic's colour): grey film in patches, darker where it lay in dirt
fn dust_tex(salt: u32) -> Tex {
    t16_paint(128, 128, move |u, v| {
        let film = t16_ss(0.4, 0.75, t16_fbm(salt, u * 6.0, v * 6.0, 3));
        let dirt = t16_ss(0.6, 0.85, t16_fbm(salt ^ 0x5d, u * 11.0, v * 11.0, 2));
        let speck = t16_vn(salt ^ 0x33, u * 120.0, v * 120.0);
        let c = t16_mix([0.92, 0.91, 0.90], [1.0, 0.99, 0.97], film);
        let c = t16_mix(c, [0.70, 0.68, 0.65], 0.6 * dirt);
        t16_mix(c, [0.80, 0.79, 0.77], 0.3 * t16_ss(0.7, 0.95, speck))
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
