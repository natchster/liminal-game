//! Batch `t16`'s prop in this crate, made real (`docs/PROPS.md` P3, "toys
//! stay toys"): the small skateboard, a moulded plastic cruiser board on
//! its wheels or upside down (`assets/src/blender/props/t16.py`,
//! `t16.notes.md`). Its placer and what only it uses; it keeps the old
//! placer's draws, call for call.
use crate::{pick_color, pick_open_tile, scatter_pos};
use game::assets::props::prop_part_as_modelled;
use game::prelude::*;

/// R451 small skateboard: 8% chance per level. A plastic cruiser board
/// (Blender batch `t16`): a 22 x 6 in moulded polypropylene deck, its top
/// textured for grip and its tail kicked, on 3 in cast aluminium trucks
/// and 59 mm urethane wheels; one in seven lies upside down, resting on its
/// tail and nose (built so: the game turns it only about y). The plastic
/// faded and scuffed white at its edges, the wheels yellowed, grime.
pub fn place_small_skateboards(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let Some((x, z)) = pick_open_tile(g) else { return };
    // the old deck colour: its plastic's, faded
    let deck_colors = [0xd82040, 0x2050d8, 0x30a030, 0xa020a0, 0xd8b040, 0x201a10];
    let old = pick_color(&deck_colors);
    let k = deck_colors.iter().position(|&c| c == old).unwrap_or(0);
    let plastic =
        [[0.55, 0.22, 0.20], [0.24, 0.33, 0.52], [0.30, 0.46, 0.32], [0.42, 0.30, 0.45], [0.70, 0.60, 0.30], [0.17, 0.16, 0.15]][k];
    let skate = group();
    scatter_pos(g, &skate, x, z, 2.0);
    skate.rotation().y = rnd() * PI * 2.0;
    // the old 15% upside down
    let flip = if rnd() < 0.15 { "Flip" } else { "" };
    let p = skate.pos();
    let salt = t16_salt(p.x, p.z);
    t16_put(&skate, &format!("Real__cruiserDeck{flip}"), &phong!({ map: &deck_tex(plastic, salt), specular: 0x161616, shininess: 20 }));
    t16_put(&skate, &format!("Real__cruiserTrucks{flip}"), &phong!({ color: 0x74787c, specular: 0x2c2c2c, shininess: 28 }));
    t16_put(&skate, &format!("Real__cruiserBushings{flip}"), &lambert!({ color: 0x8e6434 }));
    t16_put(&skate, &format!("Real__cruiserWheels{flip}"), &lambert!({ map: &wheel_tex(salt) }));
    g.world.add(&skate);
}

/// The deck's plastic, laid on its bounds (u along it, v across): the fine
/// moulded grip of its top, faded, scuffed white where it scraped (its
/// nose, its tail, its rails), dark where feet wore it, grime
fn deck_tex(plastic: [f64; 3], salt: u32) -> Tex {
    t16_paint(512, 128, move |u, v| {
        let mut c = t16_mix(plastic, [0.60, 0.58, 0.54], 0.22 * t16_ss(0.4, 0.8, t16_fbm(salt, u * 6.0, v * 2.0, 3)));
        // the moulded grip: a fine raised lattice
        let (gx, gy) = (u * 180.0, v * 50.0);
        let lat = ((gx + gy).sin() * (gx - gy).sin()).abs();
        c = t16_mix(c, [c[0] * 0.82, c[1] * 0.82, c[2] * 0.82], 0.5 * (1.0 - t16_ss(0.05, 0.35, lat)));
        // scraped white at the ends and the rails, in streaks along it
        let ends = t16_ss(0.14, 0.02, u).max(t16_ss(0.88, 0.99, u)).max(t16_ss(0.12, 0.0, v)).max(t16_ss(0.88, 1.0, v));
        let streak = t16_ss(0.55, 0.75, t16_fbm(salt ^ 0x2d, u * 60.0, v * 6.0, 3));
        c = t16_mix(c, [0.74, 0.72, 0.68], 0.75 * ends * streak);
        // where feet stood: darker, the grip worn smooth
        let feet = (-(((u - 0.30) / 0.10).powi(2) + ((v - 0.5) / 0.35).powi(2)))
            .exp()
            .max((-(((u - 0.72) / 0.10).powi(2) + ((v - 0.5) / 0.35).powi(2))).exp());
        c = t16_mix(c, [c[0] * 0.70, c[1] * 0.70, c[2] * 0.70], 0.5 * feet);
        let dirt = t16_ss(0.5, 0.85, t16_fbm(salt ^ 0x4d, u * 10.0, v * 3.0, 3));
        t16_mix(c, [0.28, 0.26, 0.23], 0.45 * dirt)
    })
}

/// The urethane of the wheels: off-white gone yellow, the tread grey with
/// grime
fn wheel_tex(salt: u32) -> Tex {
    t16_paint(64, 64, move |u, v| {
        let c = t16_mix([0.76, 0.71, 0.56], [0.66, 0.60, 0.44], t16_fbm(salt ^ 0x6d, u * 5.0, v * 5.0, 3));
        t16_mix(c, [0.36, 0.33, 0.29], 0.55 * t16_ss(0.45, 0.8, t16_fbm(salt ^ 0x7d, u * 9.0, v * 9.0, 2)))
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
