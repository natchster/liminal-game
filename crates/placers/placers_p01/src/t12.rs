//! Batch `t12`'s prop in this crate, made real (`docs/design/PROPS.md` P3, "the
//! party and the shrine"): the party horn left on the floor
//! (`assets/src/blender/props/t12.py`, `t12.notes.md`). Its placer and what
//! only it uses; it keeps the old placer's draws, call for call.
use crate::{open_tiles, pick};
use game::prelude::*;
use kit::*;

/// The horn's cone (`t12.py` HORN_*): its length, its bell's and its narrow
/// end's radii, where its glued flap's edge is (rad round from +z towards
/// +x)
const HORN_L: f64 = 0.200;
const HORN_RB: f64 = 0.024;
const HORN_RT: f64 = 0.0048;
const HORN_SEAM1: f64 = 2.5;
/// Laid down: the turn about z that rests it on its bell's rim and its
/// mouthpiece, and the lift onto the floor, whole and trodden flat, as its
/// parts were built lying (`t12.py` prints both); undone to lay its print on
const HORN_LIE: [f64; 2] = [1.646_985, 1.585_626];
const HORN_LIFT: [f64; 2] = [0.024_163, 0.015_483];
/// The print's share of the texture; above it the card of its inside
const PRINT_V: f64 = 0.92;

/// Small party horn: 8% chance per level. A 9 in party horn (Blender
/// batch `t12`): a cone of card covered in printed foil, a white plastic
/// mouthpiece with its reed, a foil fringe round the bell; lying where it
/// was dropped, one in three trodden flat at its bell; the foil dulled,
/// scuffed through to the card, dusty, grimy where it lies.
pub(crate) fn place_small_party_horns(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = pick(&open_tiles);
    // the old colour: the foil's
    let horn_colors = [0xd82060, 0x2050d8, 0xd8b820, 0x30a030, 0xa020a0];
    let hc = pick(&horn_colors);
    let design = horn_colors.iter().position(|&c| c == hc).unwrap_or(0);
    let horn = group();
    let px = g.wx(x) + (rnd() - 0.5) * 2.4;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.4;
    horn.position().set(px, 0, pz);
    horn.rotation().y = rnd() * PI * 2.0;
    let salt = salt_of(px, pz);
    let trod = usize::from(salt % 3 == 0);
    // the foil: dark in its diffuse, its colour in the highlight
    let spec = [0x8a5a6a, 0x5a6a8e, 0x8e7a46, 0x5a7a5a, 0x7a5a84][design];
    let foil = phong!({ map: &horn_print_tex(design, trod == 1, salt), specular: spec, shininess: 34 });
    // built lying (see HORN_LIE); its print laid on by where each vertex is
    // on the cone as it stands
    put_with(&horn, ["Real__partyHornCone", "Real__partyHornConeTrod"][trod], &foil, |gg| {
        cone_uv(gg, HORN_LIE[trod], HORN_LIFT[trod], HORN_L, PRINT_V)
    });
    // the fringe in gold or silver foil, the mouthpiece white plastic gone
    // cream
    let (fd, fs) = if salt & 4 == 0 { (0x7c6636, 0x9c8858) } else { (0x767678, 0x9a9a9e) };
    put(
        &horn,
        ["Real__partyHornFringe", "Real__partyHornFringeTrod"][trod],
        &phong!({ color: fd, specular: fs, shininess: 24, side: DoubleSide }),
    );
    put(&horn, ["Real__partyHornMouth", "Real__partyHornMouthTrod"][trod], &phong!({ color: 0xbdb5a0, specular: 0x262626, shininess: 30 }));
    g.world.add(&horn);
}

/// The horn's printed foil, laid out flat as it was printed (a texel's
/// place on the flat sector from where it is on the cone) and worn: scuffed
/// through to the card, creased white where it was trodden, dust, grime
/// along the side it lies on; above `PRINT_V` the card of its inside
fn horn_print_tex(design: usize, trod: bool, salt: u32) -> Tex {
    canvas_tex_opt(256, 512, false, move |ctx, w, h| {
        let a = (HORN_RB - HORN_RT).atan2(HORN_L);
        let card = [0.80, 0.78, 0.72];
        // the foil's diffuse (dark: a metal) and the ink printed on it
        let (ground, ink) = match design {
            0 => ([0.50, 0.25, 0.31], [0.82, 0.79, 0.72]),
            1 => ([0.26, 0.32, 0.47], [0.82, 0.79, 0.72]),
            2 => ([0.55, 0.45, 0.22], [0.26, 0.16, 0.20]),
            3 => ([0.27, 0.40, 0.28], [0.82, 0.79, 0.72]),
            _ => ([0.40, 0.27, 0.44], [0.70, 0.70, 0.72]),
        };
        let sheet = |u: f64, v: f64| -> (f64, f64, f64, f64) {
            let y = v / PRINT_V * HORN_L;
            let r = HORN_RB + (HORN_RT - HORN_RB) * y / HORN_L;
            let rho = r / a.sin();
            let d = (u * PI * 2.0 - HORN_SEAM1).rem_euclid(PI * 2.0);
            let psi = d * a.sin();
            (rho * psi.cos(), rho * psi.sin(), y, r)
        };
        fill(ctx, w, h, |u, v| {
            if v > PRINT_V {
                // the card inside, darker the deeper in (the light hardly
                // gets there)
                let depth = ((v - PRINT_V - 0.02) / 0.06).clamp(0.0, 1.0);
                let n = fbm(salt ^ 0x51, u * 9.0, v * 40.0, 3);
                let c = mix([0.62, 0.60, 0.55], [0.50, 0.48, 0.43], n);
                let c = mix(c, [0.16, 0.15, 0.13], sstep(0.0, 0.7, depth));
                return [c[0], c[1], c[2], 1.0];
            }
            let (sx, sy, y, r) = sheet(u, v);
            let mut c = ground;
            let m = match design {
                0 | 4 => {
                    // stripes across the flat sheet: spirals on the cone
                    let k = (sx * 0.7 + sy) / 0.011;
                    let f = k - k.floor();
                    sstep(0.0, 0.03, f) * (1.0 - sstep(0.40, 0.43, f))
                }
                1 => {
                    // five-pointed stars
                    let sp = 0.016;
                    let (i, j) = ((sx / sp).floor(), (sy / sp).floor());
                    let off = if (j as i64) % 2 == 0 { 0.0 } else { 0.5 };
                    let (cx, cy) = ((i + 0.5 + off).min(i + 1.0) * sp, (j + 0.5) * sp);
                    let (dx, dy) = (sx - cx, sy - cy);
                    let ang = dy.atan2(dx) + hash(17, i as i32, j as i32) * 6.0;
                    let rr = 0.0032 * (0.62 + 0.38 * (2.5 * ang).cos().abs().powf(3.0));
                    1.0 - sstep(rr - 0.0004, rr + 0.0004, dx.hypot(dy))
                }
                2 => {
                    // a fine diamond hatch in dark ink
                    let k1 = (sx + sy) / 0.005;
                    let k2 = (sx - sy) / 0.005;
                    let f1 = (k1 - k1.floor() - 0.5).abs();
                    let f2 = (k2 - k2.floor() - 0.5).abs();
                    (1.0 - sstep(0.06, 0.12, f1)).max(1.0 - sstep(0.06, 0.12, f2)) * 0.8
                }
                _ => {
                    let sp = 0.010;
                    let j = (sy / sp).floor();
                    let off = if (j as i64) % 2 == 0 { 0.0 } else { 0.5 };
                    let i = (sx / sp - off).floor();
                    let (cx, cy) = ((i + 0.5 + off) * sp, (j + 0.5) * sp);
                    1.0 - sstep(0.0019, 0.0025, (sx - cx).hypot(sy - cy))
                }
            };
            c = mix(c, ink, m);
            let ux = u * 2.0 * PI * r;
            // the foil dulled and patchy, scuffed through to the card
            c = mix(c, [0.46, 0.44, 0.40], 0.10 + 0.14 * fbm(salt, ux * 40.0, y * 40.0, 3));
            let sc = vn(salt ^ 3, ux * 1100.0, y * 1100.0) * 0.55 + fbm(salt ^ 4, ux * 150.0, y * 300.0, 2) * 0.45;
            if sc > 0.70 {
                c = mix(c, card, ((sc - 0.70) * 6.0).min(0.9));
            }
            // trodden: the foil cracked white along its two folds
            if trod && y < 0.13 {
                for fold in [0.0, 0.5, 1.0] {
                    let dd = (u - fold).abs() * 2.0 * PI * r;
                    let crack = (1.0 - sstep(0.0002, 0.0012, dd)) * (1.0 - sstep(0.08, 0.13, y));
                    if vn(salt ^ 5, y * 1500.0, fold * 7.0) > 0.2 {
                        c = mix(c, card, crack * 0.8);
                    }
                }
            }
            // the side it lies on (round to -x): grime; dust over the rest
            let du = ((u - 0.75 + 0.5).rem_euclid(1.0) - 0.5).abs();
            let grime = (1.0 - sstep(0.03, 0.12, du)) * (0.6 + 0.4 * fbm(salt ^ 6, ux * 50.0, y * 50.0, 2));
            c = mix(c, [0.16, 0.13, 0.10], grime * 0.6);
            let dust = fbm(salt ^ 7, ux * 18.0, y * 18.0, 4);
            c = mix(c, [0.52, 0.49, 0.44], (0.10 + 0.25 * sstep(0.35, 0.75, dust)) * sstep(0.12, 0.30, du));
            [c[0], c[1], c[2], 1.0]
        });
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
    /// Texture coordinates for a cone of card (a hat, a horn), from where
    /// each vertex is on it as it stands (`beta`, `lift`: the turn about z
    /// and the lift it was built lying with, undone): u round its axis, v up
    /// it over `ymax` into the print's share `print_v`; its inside (by the
    /// normal) the band above, by how deep in. Triangles across u's wrap
    /// get their low corners again at u + 1.
    pub(super) fn cone_uv(gg: &mut Geometry, beta: f64, lift: f64, ymax: f64, print_v: f64) {
        let (c, s) = (beta.cos(), beta.sin());
        let n = gg.position.len() / 3;
        let mut uv = vec![0f32; n * 2];
        for i in 0..n {
            let (x, y, z) = (f64::from(gg.position[i * 3]), f64::from(gg.position[i * 3 + 1]) - lift, f64::from(gg.position[i * 3 + 2]));
            let (px, py) = (x * c + y * s, -x * s + y * c);
            let (nx0, ny0, nz) = (f64::from(gg.normal[i * 3]), f64::from(gg.normal[i * 3 + 1]), f64::from(gg.normal[i * 3 + 2]));
            let nx = nx0 * c + ny0 * s;
            let l = px.hypot(z).max(1e-6);
            let inside = (nx * px + nz * z) / l < -0.3;
            let u = px.atan2(z).rem_euclid(2.0 * PI) / (2.0 * PI);
            let f = (py / ymax).clamp(0.0, 1.0);
            uv[i * 2] = u as f32;
            uv[i * 2 + 1] = if inside { print_v + 0.02 + 0.06 * f } else { (f * print_v).min(print_v - 0.002) } as f32;
        }
        let mut idx: Vec<u32> = gg.index.clone().unwrap_or_else(|| (0..n as u32).collect());
        let mut dup: Vec<Option<u32>> = vec![None; n];
        for t in idx.chunks_mut(3) {
            let us = [uv[t[0] as usize * 2], uv[t[1] as usize * 2], uv[t[2] as usize * 2]];
            let (lo, hi) = (us.iter().cloned().fold(f32::MAX, f32::min), us.iter().cloned().fold(f32::MIN, f32::max));
            if hi - lo <= 0.5 {
                continue;
            }
            for k in 0..3 {
                let v = t[k] as usize;
                if v < n && uv[v * 2] < 0.5 {
                    let d = match dup[v] {
                        Some(d) => d,
                        None => {
                            let d = (gg.position.len() / 3) as u32;
                            for a in 0..3 {
                                let (p, q) = (gg.position[v * 3 + a], gg.normal[v * 3 + a]);
                                gg.position.push(p);
                                gg.normal.push(q);
                            }
                            uv.push(uv[v * 2] + 1.0);
                            uv.push(uv[v * 2 + 1]);
                            dup[v] = Some(d);
                            d
                        }
                    };
                    t[k] = d;
                }
            }
        }
        gg.uv = uv;
        gg.index = Some(idx);
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
