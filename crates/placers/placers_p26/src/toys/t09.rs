//! Batch `t09`'s carnival mask, made real (`docs/design/PROPS.md` P3, "the party
//! and the shrine"; `assets/src/blender/props/t09.py`, `t09.notes.md`): a
//! Venetian volto of papier-mache, worn. Its placer and what only it uses;
//! it keeps the old placer's draws, call for call.
use crate::{open_tiles, pick};
use game::prelude::*;

/// The face part's bounds across (x) and from its brow (-z) to its chin
/// (+z), as built (`t09.py`, `Real__maskFace`), in mm of the face's own frame:
/// X = u * MASK_X - MASK_X / 2, Y (up the face) = v * MASK_Y - MASK_Y0
const MASK_X: f64 = 150.0;
const MASK_Y: f64 = 215.8;
const MASK_Y0: f64 = 111.5;

/// Small face mask: 6% chance per level. A Venetian volto (15 x 21.5
/// cm), a full-face carnival mask of papier-mache, gessoed and painted in
/// one colour, gilt round the eyes and its edge, two satin ribbons from its
/// temples; lying face up where it was dropped. Its gesso crazed, its paint
/// rubbed through to the brown paper on the nose, the brow and the chin,
/// grime in the eyes' hollows, dust.
pub fn place_small_masks(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance (rarest)
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    // the old face colours: the paint over the gesso
    let face_colors = [0xd0b898, 0xa08060, 0xd0d0c0, 0xe8ccb0, 0xa0a0a0];
    let paint = pick(&face_colors);
    let px = g.wx(x) + (rnd() - 0.5) * 2.4;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.4;
    let turn = rnd() * PI * 2.0;
    let salt = t09_salt(px, pz);
    let mask = group();
    t09_put(&mask, "Real__maskFace", &phong!({ map: &mask_tex(salt, paint), specular: 0x2a2620, shininess: 18 }));
    // satin, faded: black or a dark red, by where it lies
    let ribbon = if salt & 1 == 0 { 0x2c2628 } else { 0x5a2a2c };
    t09_put(&mask, "Real__maskRibbons", &phong!({ color: ribbon, specular: 0x5a5654, shininess: 22 }));
    mask.position().set(px, 0, pz);
    mask.rotation().y = turn;
    g.world.add(&mask);
}

/// The face's relief (`t09.py`, `mask_height`): Z (mm) at (X, Y)
fn mask_height(x: f64, y: f64) -> f64 {
    let gs = |cx: f64, cy: f64, sx: f64, sy: f64, ax: f64| (-(((ax - cx) / sx).powi(2) + ((y - cy) / sy).powi(2))).exp();
    let ax = x.abs();
    let mut z = -(x * x) / (2.0 * 64.0) - (y * y) / (2.0 * 260.0);
    z += 4.5 * gs(0.0, 72.0, 42.0, 30.0, x);
    z += 5.0 * (-((y - 37.0) / 9.0).powi(2)).exp() * (-(x / 48.0).powi(4)).exp();
    z -= 7.0 * gs(32.0, 21.0, 19.0, 11.0, ax);
    z += 7.5 * gs(38.0, -10.0, 19.0, 20.0, ax);
    let t = ss(32.0, -16.0, y) * (1.0 - ss(-16.0, -27.0, y));
    let w = 5.5 + 7.0 * ss(32.0, -16.0, y);
    z += (3.0 + 21.0 * t.powf(1.3)) * (-(x / w).powi(2)).exp() * (1.0 - ss(-22.0, -30.0, y) * 0.6) * (1.0 - ss(30.0, 44.0, y));
    z += 6.5 * gs(11.0, -21.0, 6.0, 6.0, ax);
    z += 5.5 * gs(0.0, -44.0, 20.0, 5.5, x);
    z += 5.0 * gs(0.0, -55.5, 17.0, 5.0, x);
    z += 8.0 * gs(0.0, -82.0, 19.0, 14.0, x);
    z
}

/// The face's paint, laid on it from above (u across, v from the chin to
/// the brow): one colour over gesso, crazed; gilt in a band round each eye
/// and along the edge, rubbed; worn through to the brown paper where it
/// stands out; grime in the hollows, dust
fn mask_tex(salt: u32, paint: u32) -> Tex {
    let p = [f64::from((paint >> 16) & 255) / 255.0, f64::from((paint >> 8) & 255) / 255.0, f64::from(paint & 255) / 255.0];
    canvas_tex_opt(256, 256, false, move |ctx, w, h| {
        let seed = salt ^ 0x3a5c;
        let w_ = w as usize;
        let mut img = ctx.create_image_data(w, h);
        par_rows(&mut img.data, w_ * 4, |j, out| {
            for i in 0..w_ {
                let (u, v) = ((i as f64 + 0.5) / w, 1.0 - (j as f64 + 0.5) / h);
                let (x, y) = (u * MASK_X - MASK_X / 2.0, v * MASK_Y - MASK_Y0);
                let n = fbm(seed, u * 6.0, v * 6.0, 4);
                let mut c = mix(p, mix(p, [0.0, 0.0, 0.0], 0.25), n * 0.6);
                // gilt: round each eye (an almond 16 x 7.4 mm at (+-32, 22)) and along the edge
                let (ex, ey) = ((x.abs() - 32.0) / 16.0, (y - 22.0) / 7.4);
                let eye = (ex * ex + ey * ey).sqrt();
                let ring = ss(1.05, 1.15, eye) * (1.0 - ss(1.55, 1.75, eye));
                let (ox, oy) = (x / 75.0, if y > 0.0 { y / 106.0 } else { y / 110.0 });
                let edge = ss(0.90, 0.94, (ox * ox + oy * oy).sqrt());
                let gilt = ring.max(edge) * (0.75 + 0.25 * ss(0.3, 0.6, vn(seed ^ 3, u * 40.0, v * 40.0)));
                c = mix(c, [0.66, 0.54, 0.30], gilt);
                // the relief above the face's curve: where it is rubbed
                let base = -(x * x) / (2.0 * 64.0) - (y * y) / (2.0 * 260.0);
                let high = mask_height(x, y) - base;
                let rub = ss(9.0, 18.0, high) * ss(0.4, 0.7, fbm(seed ^ 5, u * 18.0, v * 18.0, 3));
                c = mix(c, [0.42, 0.33, 0.24], rub * 0.85);
                // the gesso crazed: a net of fine dark lines
                let cr = (fbm(seed ^ 7, u * 14.0, v * 14.0, 3) - 0.5).abs().min((fbm(seed ^ 11, u * 22.0, v * 22.0, 2) - 0.5).abs());
                c = mix(c, [0.30, 0.26, 0.21], (1.0 - ss(0.002, 0.008, cr)) * 0.35);
                // grime in the eyes' hollows and under the nose
                let hollow = ss(-1.0, -5.0, high);
                c = mix(c, [0.26, 0.22, 0.18], hollow * 0.45);
                let dust = vn(seed ^ 13, u * 150.0, v * 150.0);
                if dust > 0.82 {
                    c = mix(c, [0.55, 0.53, 0.50], (dust - 0.82) * 2.5);
                }
                for k in 0..3 {
                    out[i * 4 + k] = (c[k].clamp(0.0, 1.0) * 255.0).round() as u8;
                }
                out[i * 4 + 3] = 255;
            }
        });
        ctx.put_image_data(&img, 0, 0);
    })
}

fn ss(e0: f64, e1: f64, x: f64) -> f64 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn mix(a: [f64; 3], b: [f64; 3], t: f64) -> [f64; 3] {
    let t = t.clamp(0.0, 1.0);
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
}

fn hash(seed: u32, i: i32, j: i32) -> f64 {
    let mut h = seed ^ (i as u32).wrapping_mul(0x27d4_eb2d) ^ (j as u32).wrapping_mul(0x1656_67b1);
    h = (h ^ (h >> 15)).wrapping_mul(0x85eb_ca6b);
    h = (h ^ (h >> 13)).wrapping_mul(0xc2b2_ae35);
    h ^= h >> 16;
    f64::from(h) / 4_294_967_296.0
}

fn vn(seed: u32, x: f64, y: f64) -> f64 {
    let (xf, yf) = (x.floor(), y.floor());
    let (i, j) = (xf as i32, yf as i32);
    let (u, v) = (x - xf, y - yf);
    let (u, v) = (u * u * (3.0 - 2.0 * u), v * v * (3.0 - 2.0 * v));
    let (a, b) = (hash(seed, i, j), hash(seed, i + 1, j));
    let (c, d) = (hash(seed, i, j + 1), hash(seed, i + 1, j + 1));
    a + (b - a) * u + (c - a) * v + (a - b - c + d) * u * v
}

fn fbm(seed: u32, x: f64, y: f64, oct: u32) -> f64 {
    let (mut s, mut amp, mut f, mut n) = (0.0, 0.5, 1.0, 0.0);
    for k in 0..oct {
        s += amp * vn(seed.wrapping_add(k * 7919), x * f, y * f);
        n += amp;
        amp *= 0.5;
        f *= 2.03;
    }
    s / n
}

/// A salt from where the mask lies (no draw): its paint's wear differs from
/// floor to floor
fn t09_salt(x: f64, z: f64) -> u32 {
    ((x * 7919.0).abs() as u32) ^ ((z * 104_729.0).abs() as u32).rotate_left(13) ^ 0x9e37_79b9
}

/// The part `name` of batch `t09` as modelled, added to `grp` in `mat`
fn t09_put(grp: &Obj, name: &str, mat: &Mat) {
    if let Some(geo) = game::assets::props::prop_part_as_modelled(name) {
        grp.add(&mesh(geo, mat.clone()));
    }
}
