//! Batch `t12`'s prop in this crate, made real (`docs/PROPS.md` P3, "the
//! party and the shrine"): the party hat, a printed card cone left on the
//! floor (`assets/src/blender/props/t12.py`, `t12.notes.md`). Its placer
//! and what only it uses; it keeps the old placer's draws, call for call.
use crate::{open_tiles, pick};
use game::prelude::*;
use kit::*;

/// The hat's height up its slant over its height (`t12.py` HAT_*): where a
/// texel of its print is on the flat sector it was rolled from
const HAT_H: f64 = 0.175;
const HAT_R: f64 = 0.0575;
/// the glued flap's edge (rad round from +z towards +x), and the crease
const HAT_SEAM1: f64 = 0.61;
const HAT_DENT: f64 = 2.50;
/// The cone's bounds' height as built (its texture's v runs over it)
const HAT_YMAX: f64 = 0.1724;
/// A hat lying: its turn about z, and the lift that puts it on the floor
/// (`t12.py` prints both as it builds)
const HAT_LIE: f64 = 1.888_255;
const HAT_LIFT: f64 = 0.056_378;
/// The print's share of the texture (v below it); above it, the plain card
/// of its inside
const PRINT_V: f64 = 0.92;

/// R426 birthday party hat: 8% chance per level. A printed card party hat
/// (Blender batch `t12`): a cone of board 175 mm tall over a 115 mm rim,
/// rolled from a printed sector and glued along its flap, a pompom of foil
/// fringe in its point, its round elastic knotted through two holes; the
/// print faded, the card dented and creased, the point bent, dust on it
/// and grime at the rim. Three in ten lie on their side.
pub fn place_birthday_hats(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    // the old hat colour: which print the card has
    let hat_colors = [0xd82060, 0x2050d8, 0xd8b820, 0x30a030, 0xd050a0];
    let hc = pick(&hat_colors);
    let design = hat_colors.iter().position(|&c| c == hc).unwrap_or(0);
    // the old band colour: the foil of the fringe in its point
    let band_colors = [0xf0f0f0, 0xd8b820, 0x2050d8, 0xd050a0];
    let bc = pick(&band_colors);
    let foil = band_colors.iter().position(|&c| c == bc).unwrap_or(0);
    let hat = group();
    let px = g.wx(x) + (rnd() - 0.5) * 2.4;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.4;
    hat.position().set(px, 0, pz);
    hat.rotation().y = rnd() * PI * 2.0;
    // the old 30% tipped on its side
    let lying = rnd() < 0.30;
    let salt = salt_of(px, pz);
    let print = hat_print_tex(design, salt);
    put_with(&hat, "Real__partyHatCone", &lambert!({ map: &print }), |gg| {
        // outside: the print where the cylinder mapping put it (u round,
        // v up); inside: the plain card, by how deep in
        let n = gg.position.len() / 3;
        for i in 0..n {
            let (x, z) = (f64::from(gg.position[i * 3]), f64::from(gg.position[i * 3 + 2]));
            let (nx, nz) = (f64::from(gg.normal[i * 3]), f64::from(gg.normal[i * 3 + 2]));
            let l = x.hypot(z).max(1e-6);
            let inside = (nx * x + nz * z) / l < -0.3;
            let v = f64::from(gg.uv[i * 2 + 1]);
            gg.uv[i * 2 + 1] = if inside { PRINT_V + 0.02 + 0.06 * v } else { (v * PRINT_V).min(PRINT_V - 0.002) } as f32;
        }
    });
    let (diffuse, spec) = [(0x5c5c5e, 0x9a9a9e), (0x6a5428, 0x9a8450), (0x44506a, 0x6c7c9c), (0x6c4c5a, 0x9c7c8c)][foil];
    let foil_mat = phong!({ color: diffuse, specular: spec, shininess: 26, side: DoubleSide });
    put(&hat, if lying { "Real__partyHatTasselLying" } else { "Real__partyHatTassel" }, &foil_mat);
    // round elastic, white gone grey with dust
    put(&hat, if lying { "Real__partyHatElasticLying" } else { "Real__partyHatElastic" }, &lambert!({ color: 0x8e897e }));
    if lying {
        hat.rotation().z = HAT_LIE;
        hat.position().y = HAT_LIFT;
    }
    g.world.add(&hat);
}

/// The hat's print, faded, on the card it was rolled from (a texel's place
/// on the flat sector from where it is on the cone), and the wear: dust,
/// the print rubbed off in specks, grime at the rim, the ink cracked along
/// the dent's crease; above `PRINT_V` the plain card of its inside
fn hat_print_tex(design: usize, salt: u32) -> Tex {
    canvas_tex_opt(512, 512, false, move |ctx, w, h| {
        let a = HAT_R.atan2(HAT_H);
        let slant = HAT_R.hypot(HAT_H);
        let card = [0.86, 0.84, 0.78];
        // the print: the ground colour and what is printed on it
        let (ground, ink, ink2): ([f64; 3], [f64; 3], [f64; 3]) = match design {
            0 => ([0.66, 0.31, 0.38], [0.88, 0.84, 0.77], [0.88, 0.84, 0.77]),
            1 => ([0.34, 0.41, 0.58], [0.85, 0.81, 0.73], [0.85, 0.81, 0.73]),
            2 => ([0.78, 0.67, 0.36], [0.66, 0.36, 0.42], [0.36, 0.47, 0.60]),
            3 => ([0.36, 0.51, 0.35], [0.86, 0.83, 0.75], [0.80, 0.72, 0.42]),
            _ => ([0.76, 0.52, 0.62], [0.88, 0.84, 0.78], [0.48, 0.25, 0.36]),
        };
        let print = |u: f64, v: f64| -> [f64; 3] {
            let phi = u * PI * 2.0;
            let y = v / PRINT_V * HAT_YMAX;
            let s = (y / HAT_H).clamp(0.0, 1.0);
            let rho = (1.0 - s) * slant;
            // round from the flap's edge: the sector's own angle
            let d = (phi - HAT_SEAM1).rem_euclid(PI * 2.0);
            let psi = d * a.sin();
            let (sx, sy) = (rho * psi.cos(), rho * psi.sin());
            let dots = |sp: f64, r: f64, stagger: bool| -> f64 {
                let j = (sy / sp).floor();
                let off = if stagger && (j as i64) % 2 != 0 { 0.5 } else { 0.0 };
                let i = (sx / sp - off).floor();
                let (cx, cy) = ((i + 0.5 + off) * sp, (j + 0.5) * sp);
                let dd = (sx - cx).hypot(sy - cy);
                1.0 - sstep(r - 0.0005, r + 0.0005, dd)
            };
            match design {
                0 | 4 => {
                    let m = dots(0.024, 0.0058, true);
                    mix(ground, ink, m)
                }
                1 | 3 => {
                    // stripes from the point to the rim (lines through the
                    // sector's centre), and a fine dot in the light ones
                    let k = d / (PI * 2.0 / 14.0);
                    let f = k - k.floor();
                    let st = sstep(0.47, 0.49, f) * (1.0 - sstep(0.97, 0.99, f));
                    let mut c = mix(ground, ink, st);
                    if design == 3 {
                        c = mix(c, ink2, dots(0.016, 0.0022, true) * (1.0 - st));
                    }
                    c
                }
                _ => {
                    // confetti: small round and square bits in two inks
                    let sp = 0.018;
                    let (i, j) = ((sx / sp).floor(), (sy / sp).floor());
                    let hh = hash(97, i as i32, j as i32);
                    let (ox, oy) = (hash(98, i as i32, j as i32), hash(99, i as i32, j as i32));
                    let (cx, cy) = ((i + 0.25 + 0.5 * ox) * sp, (j + 0.25 + 0.5 * oy) * sp);
                    let dd = (sx - cx).hypot(sy - cy);
                    let m = 1.0 - sstep(0.0026, 0.0034, dd);
                    mix(ground, if hh < 0.5 { ink } else { ink2 }, m)
                }
            }
        };
        fill(ctx, w, h, |u, v| {
            if v > PRINT_V {
                // the plain card inside, darker the deeper in
                let depth = ((v - PRINT_V - 0.02) / 0.06).clamp(0.0, 1.0);
                let n = fbm(salt ^ 0x51, u * 9.0, v * 40.0, 3);
                let c = mix([0.74, 0.72, 0.66], [0.62, 0.59, 0.53], n);
                let c = mix(c, [0.20, 0.19, 0.17], sstep(0.1, 0.9, depth));
                return [c[0], c[1], c[2], 1.0];
            }
            let c = print(u, v);
            [c[0], c[1], c[2], 1.0]
        });
        if design == 4 {
            // HAPPY BIRTHDAY round the rim, in the sector's arc: laid on
            // the texture stretched round as the cone takes it
            let s_text = 0.105;
            let r = HAT_R * (1.0 - s_text);
            let y = s_text * HAT_H / HAT_YMAX * PRINT_V;
            let px_per_m_u = w / (2.0 * PI * r);
            let px_per_m_v = h / (HAT_YMAX / PRINT_V);
            ctx.save();
            ctx.set_fill_style("rgb(122,62,92)");
            ctx.set_text_align("center");
            ctx.set_text_baseline("middle");
            ctx.translate(0, h * (1.0 - y));
            ctx.scale(px_per_m_u / px_per_m_v, 1.0);
            ctx.set_font(&format!("bold {:.0}px Arial Rounded MT Bold", 0.0095 * px_per_m_v));
            let span = w * px_per_m_v / px_per_m_u;
            for k in 0..3 {
                ctx.fill_text("HAPPY BIRTHDAY", span * (k as f64 + 0.5) / 3.0, 0);
            }
            ctx.restore();
        }
        // the wear, over what is printed
        let mut img = ctx.get_image_data(0, 0, w, h);
        let iw = w as usize;
        par_rows(&mut img.data, iw * 4, |j, out| {
            let v = 1.0 - (j as f64 + 0.5) / h;
            for i in 0..iw {
                let u = (i as f64 + 0.5) / w;
                let mut c = [f64::from(out[i * 4]) / 255.0, f64::from(out[i * 4 + 1]) / 255.0, f64::from(out[i * 4 + 2]) / 255.0];
                if v <= PRINT_V {
                    let y = v / PRINT_V * HAT_YMAX;
                    let s = y / HAT_H;
                    let r = HAT_R * (1.0 - s);
                    let (ux, vy) = (u * 2.0 * PI * r, y);
                    // faded unevenly, paler where the light lay
                    let fade = fbm(salt, ux * 30.0, vy * 30.0, 3);
                    c = mix(c, [0.80, 0.77, 0.70], 0.10 + 0.16 * fade);
                    // the print rubbed off in specks and scuffs (the card)
                    let sc = vn(salt ^ 3, ux * 900.0, vy * 900.0) * 0.6 + fbm(salt ^ 4, ux * 120.0, vy * 240.0, 2) * 0.4;
                    if sc > 0.72 {
                        c = mix(c, card, ((sc - 0.72) * 6.0).min(0.85));
                    }
                    // the ink cracked white along the dent's crease
                    let dphi = ((u * 2.0 * PI - HAT_DENT + PI).rem_euclid(2.0 * PI) - PI) * r;
                    let crack = (1.0 - sstep(0.0002, 0.0009, dphi.abs())) * sstep(0.10, 0.18, s) * (1.0 - sstep(0.60, 0.70, s));
                    if crack > 0.0 && vn(salt ^ 5, vy * 900.0, 1.0) > 0.25 {
                        c = mix(c, card, crack * 0.75);
                    }
                    // grime where the rim stood on the floor, dust over it
                    let grime = (1.0 - sstep(0.0, 0.022, y)) * (0.55 + 0.45 * fbm(salt ^ 6, ux * 60.0, 3.0, 2));
                    c = mix(c, [0.30, 0.26, 0.21], grime * 0.55);
                    let dust = fbm(salt ^ 7, ux * 14.0, vy * 14.0, 4);
                    c = mix(c, [0.62, 0.59, 0.53], 0.14 + 0.22 * sstep(0.35, 0.75, dust));
                }
                for k in 0..3 {
                    out[i * 4 + k] = (c[k].clamp(0.0, 1.0) * 255.0).round() as u8;
                }
            }
        });
        ctx.put_image_data(&img, 0, 0);
    })
}

/// The painting kit of batch `t12`'s painters (as batches `t08`, `t09`):
/// value noise and a per-pixel fill (pure, so `par_rows` may run it on
/// every core), a salt from where a thing is, a part placed.
#[allow(dead_code)]
mod kit {
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
    pub(super) fn salt_of(x: f64, z: f64) -> u32 {
        let (a, b) = ((x * 1000.0).round() as i64 as u64, (z * 1000.0).round() as i64 as u64);
        let mut h = a.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ b.wrapping_mul(0xC2B2_AE3D_27D4_EB4F);
        h ^= h >> 29;
        (h ^ (h >> 32)) as u32
    }
    /// The part `name` of a Blender batch as modelled, added to `grp` in `mat`
    pub(super) fn put(grp: &Obj, name: &str, mat: &Mat) -> Option<Obj> {
        let geo = game::assets::props::prop_part_as_modelled(name)?;
        let m = mesh(geo, mat.clone());
        grp.add(&m);
        Some(m)
    }
    /// The same with its geometry (a copy) changed by `edit` first (its
    /// texture coordinates)
    pub(super) fn put_with(grp: &Obj, name: &str, mat: &Mat, edit: impl FnOnce(&mut Geometry)) -> Option<Obj> {
        let geo = game::assets::props::prop_part_as_modelled(name)?.duplicate();
        {
            let mut gg = geo.g();
            edit(&mut gg);
            gg.touch();
        }
        let m = mesh(geo, mat.clone());
        grp.add(&m);
        Some(m)
    }
}
