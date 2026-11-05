//! Batch `t16`'s prop in this crate, made real (`docs/PROPS.md` P3, "toys
//! stay toys"): the small xylophone, a child's eight note metallophone,
//! enamelled bars pinned on a pine frame, its mallet beside it
//! (`assets/src/blender/props/t16.py`, `t16.notes.md`). Its placer and what
//! only it uses; it keeps the old placer's draws, call for call.
use crate::{open_tiles, pick};
use game::assets::props::prop_part_as_modelled;
use game::prelude::*;

/// The bars as built (`t16.py` XY_*): eight, 28 mm apart along x from the
/// low C, 22 mm wide, 100 mm stepping down to 68 mm long across z
const N_BARS: usize = 8;
const PITCH: f64 = 0.028;
const BAR_W: f64 = 0.022;
/// The bars' bounds in plan (x, z), over which their enamel is laid
const X0: f64 = -0.109;
const X1: f64 = 0.109;
const Z0: f64 = -0.050;
const Z1: f64 = 0.050;

fn bar_len(i: usize) -> f64 {
    0.100 - i as f64 * (0.032 / (N_BARS - 1) as f64)
}

fn bar_x(i: usize) -> f64 {
    -PITCH * (N_BARS - 1) as f64 / 2.0 + i as f64 * PITCH
}

/// R436 small xylophone: 6% chance per level. A child's eight note
/// "xylophone" (Blender batch `t16`): a metallophone of 3 mm aluminium bars
/// enamelled in eight colours, C to C (100 to 68 mm), each pinned through
/// its two nodes over a felt strip on a splayed pine frame (265 x 75 mm),
/// its note's letter printed at its foot; a wooden mallet lying in front.
/// The enamel faded, chipped at the corners and worn to the metal where
/// the mallet struck, the pine grey with handling, dust.
pub fn place_small_xylophones(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let xylo = group();
    let px = g.wx(x) + (rnd() - 0.5) * 2.0;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.0;
    let salt = t16_salt(px, pz);
    t16_put(&xylo, "Real__xyloFrame", &lambert!({ map: &pine_tex(salt) }));
    t16_put(&xylo, "Real__xyloFelt", &lambert!({ color: 0x4a2a28 }));
    if let Some(geo) = prop_part_as_modelled("Real__xyloBars") {
        // the enamel laid on from above, every face (the bars' sides take
        // their own bar's colour)
        let geo = geo.duplicate();
        {
            let mut gg = geo.g();
            let n = gg.position.len() / 3;
            for i in 0..n {
                let (bx, bz) = (f64::from(gg.position[i * 3]), f64::from(gg.position[i * 3 + 2]));
                gg.uv[i * 2] = ((bx - X0) / (X1 - X0)) as f32;
                gg.uv[i * 2 + 1] = (1.0 - (bz - Z0) / (Z1 - Z0)) as f32;
            }
            gg.touch();
        }
        xylo.add(&mesh(geo, phong!({ map: &enamel_tex(salt), specular: 0x2a2a2a, shininess: 30 })));
    }
    t16_put(&xylo, "Real__xyloPins", &phong!({ color: 0x6a6a68, specular: 0x303030, shininess: 30 }));
    t16_put(&xylo, "Real__xyloMallet", &lambert!({ color: 0x96785a }));
    t16_put(&xylo, "Real__xyloMalletHead", &phong!({ color: 0x3a2e26, specular: 0x1e1e1e, shininess: 20 }));
    xylo.position().set(px, 0, pz);
    xylo.rotation().y = rnd() * PI * 2.0;
    g.world.add(&xylo);
}

/// The bars' enamel laid on from above (u along x, v along z): each bar its
/// colour, faded; chipped to the aluminium round its edges and corners, a
/// patch worn to the metal where the mallet struck it, scratches, grime
/// round its pins; its note's letter printed white at its foot, rubbed
fn enamel_tex(salt: u32) -> Tex {
    const COLS: [[f64; 3]; N_BARS] = [
        [0.62, 0.24, 0.21],
        [0.70, 0.42, 0.22],
        [0.76, 0.64, 0.30],
        [0.32, 0.50, 0.32],
        [0.28, 0.48, 0.58],
        [0.25, 0.31, 0.54],
        [0.42, 0.30, 0.50],
        [0.66, 0.38, 0.46],
    ];
    let (w, h) = (512usize, 256usize);
    canvas_tex_opt(w as f64, h as f64, false, move |ctx, cw, ch| {
        let mut img = ctx.create_image_data(cw, ch);
        par_rows(&mut img.data, w * 4, |j, out| {
            for i in 0..w {
                let (u, v) = ((i as f64 + 0.5) / w as f64, (j as f64 + 0.5) / h as f64);
                let (x, z) = (X0 + (X1 - X0) * u, Z0 + (Z1 - Z0) * v);
                let k = (((x - bar_x(0)) / PITCH).round().max(0.0) as usize).min(N_BARS - 1);
                let (dx, dz) = ((x - bar_x(k)).abs(), z.abs());
                // how far in from the bar's edge (m)
                let edge = (BAR_W / 2.0 - dx).min(bar_len(k) / 2.0 - dz);
                let metal = t16_mix([0.66, 0.67, 0.68], [0.52, 0.53, 0.54], t16_fbm(salt ^ 0x15, u * 30.0, v * 15.0, 2));
                let mut c = t16_mix(COLS[k], [0.70, 0.68, 0.64], 0.15 * t16_ss(0.45, 0.8, t16_fbm(salt, u * 8.0, v * 4.0, 3)));
                let chip_n = t16_fbm(salt ^ 0x25, u * 60.0, v * 30.0, 3);
                let corner = t16_ss(0.006, 0.0, (BAR_W / 2.0 - dx).max(0.0).hypot((bar_len(k) / 2.0 - dz).max(0.0)) - 0.002);
                let chip = t16_ss(0.0035, 0.0005, edge).max(corner) * t16_ss(0.45, 0.6, chip_n);
                // where the mallet struck: the middle of the bar, a little to the player
                let (sx, sz) = ((x - bar_x(k)) / 0.008, (z - 0.004) / 0.011);
                let strike = (-(sx * sx + sz * sz)).exp() * t16_ss(0.3, 0.6, t16_fbm(salt ^ 0x35, u * 90.0, v * 45.0, 2));
                c = t16_mix(c, metal, chip.max(strike * 0.85));
                let scratch = (t16_fbm(salt ^ 0x45, u * 4.0, v * 70.0, 2) - 0.5).abs();
                c = t16_mix(c, [0.78, 0.77, 0.74], 0.25 * (1.0 - t16_ss(0.0, 0.006, scratch)));
                // grime round the pins and dust
                let pin = (x - bar_x(k)).hypot(dz - bar_len(k) * (0.5 - 0.224));
                c = t16_mix(c, [0.22, 0.20, 0.18], 0.5 * t16_ss(0.006, 0.0025, pin));
                let dust = t16_ss(0.5, 0.85, t16_fbm(salt ^ 0x55, u * 12.0, v * 6.0, 3));
                c = t16_mix(c, [0.55, 0.53, 0.50], 0.2 * dust);
                for q in 0..3 {
                    out[i * 4 + q] = (c[q].clamp(0.0, 1.0) * 255.0).round() as u8;
                }
                out[i * 4 + 3] = 255;
            }
        });
        ctx.put_image_data(&img, 0, 0);
        // the notes' letters at the bars' feet (+z), printed and rubbed
        ctx.set_font("bold 15px sans-serif");
        ctx.set_text_align("center");
        for (k, l) in ["C", "D", "E", "F", "G", "A", "B", "C"].iter().enumerate() {
            let px = (bar_x(k) - X0) / (X1 - X0) * cw;
            let pz = (bar_len(k) / 2.0 - 0.006 - Z0) / (Z1 - Z0) * ch;
            ctx.set_fill_style("rgba(226,222,210,0.72)");
            ctx.fill_text(l, px, pz);
        }
    })
}

/// The pine of the frame laid on its bounds: its grain along the rails,
/// greyed and darkened with handling, dust
fn pine_tex(salt: u32) -> Tex {
    t16_paint(256, 64, move |u, v| {
        let grain = t16_fbm(salt ^ 0x65, u * 3.0, v * 40.0, 3);
        let mut c = t16_mix([0.66, 0.53, 0.36], [0.55, 0.42, 0.28], grain);
        c = t16_mix(c, [0.52, 0.48, 0.42], 0.35 * t16_ss(0.4, 0.8, t16_fbm(salt ^ 0x75, u * 6.0, v * 3.0, 3)));
        t16_mix(c, [0.30, 0.26, 0.22], 0.3 * t16_ss(0.6, 0.9, t16_fbm(salt ^ 0x85, u * 14.0, v * 8.0, 2)))
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
