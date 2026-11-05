//! Batch `t13`'s paper plate, made real (`docs/PROPS.md` P3, "the party
//! and the shrine"): a 7 in coated paper dessert plate left from a party,
//! years on (`assets/src/blender/props/t13.py`; what the real one is and
//! where its size comes from: `t13.notes.md`). It keeps the old placer's
//! draws, call for call (the comments at each say what a draw now chooses).
use crate::{open_tiles, pick};
use game::assets::props::prop_part_as_modelled;
use game::prelude::*;
use kit::*;

/// The plate as modelled (`t13.py`): its half width and its height (the
/// top part's bounds), its well's radius and its flange's
const PLATE_R: f64 = 0.0892;
const PLATE_H: f64 = 0.0124;
const PLATE_WELL: f64 = 0.0556;
const PLATE_FLANGE: (f64, f64) = (0.0745, 0.0880);
/// The crumbs (`r16.py`'s `Real__breadCrumbs`, modelled on the bread
/// board): moved off the board onto the plate's well
const CRUMBS_AT: (f64, f64, f64) = (-0.066, -0.0172, 0.011);

// R428 small paper plate: 12% chance per level. A 7 in coated paper dessert
// plate, pressed and pleated, white or printed round its flange in one of
// the party inks; eaten from (a smear of frosting gone hard, grease soaked
// in, crumbs), stacked three or four as they came, dropped face down, or
// trodden on and bent up across a crease; dust over all of it.
pub fn place_small_paper_plates(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.88 {
        return;
    } // 12% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    // the old colour draws: the board's white, and (60%) the ink it is
    // printed in round its flange
    let plate_colors = [0xf0ece4, 0xece0d0, 0xe8d8c8, 0xf4e8d8];
    let pattern_colors = [0xd82060, 0x2050d8, 0xd8b820, 0x30a030, 0xa020a0];
    let col = pick(&plate_colors);
    let board = plate_colors.iter().position(|&c| c == col).unwrap_or(0);
    let ink = if rnd() < 0.60 {
        let c = pick(&pattern_colors);
        pattern_colors.iter().position(|&p| p == c)
    } else {
        None
    };
    let px = g.wx(x) + (rnd() - 0.5) * 2.4;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.4;
    let yaw = rnd() * PI * 2.0;
    // what became of it: from where it is (no draw)
    let salt = salt(px, pz, 28);
    let how = match salt % 8 {
        0..=3 => Plate::Eaten,
        4 | 5 => Plate::Stack(3 + (salt >> 8) % 2),
        6 => Plate::FaceDown,
        _ => Plate::Trodden,
    };
    let used = !matches!(how, Plate::Stack(_));
    let top = lambert!({ map: &top_tex(board, ink, salt, how) });
    let under = lambert!({ map: &under_tex(board, salt, used) });
    let plate = group();
    match how {
        Plate::Stack(n) => {
            for i in 0..n {
                let one = group();
                let k = f64::from(i);
                one.position().set(0.0015 * (k * 2.3 + f64::from(salt % 5)).sin(), k * 0.0016, 0.0015 * (k * 1.7).cos());
                one.rotation().y = k * 0.7 + f64::from((salt >> 3) % 9) * 0.1;
                put(&one, "Real__paperPlateTop", &top);
                put(&one, "Real__paperPlateUnder", &under);
                plate.add(&one);
            }
        }
        Plate::FaceDown => {
            let flip = group();
            flip.rotation().x = PI;
            flip.position().y = PLATE_H;
            put(&flip, "Real__paperPlateTop", &top);
            put(&flip, "Real__paperPlateUnder", &under);
            plate.add(&flip);
        }
        Plate::Eaten | Plate::Trodden => {
            let bent = how == Plate::Trodden;
            put(&plate, if bent { "Real__paperPlateBentTop" } else { "Real__paperPlateTop" }, &top);
            put(&plate, if bent { "Real__paperPlateBentUnder" } else { "Real__paperPlateUnder" }, &under);
            if !bent {
                if let Some(geo) = prop_part_as_modelled("Real__breadCrumbs") {
                    let crumbs = mesh(geo, lambert!({ color: 0x8a7458 }));
                    crumbs.position().set(CRUMBS_AT.0, CRUMBS_AT.1, CRUMBS_AT.2);
                    plate.add(&crumbs);
                }
            }
        }
    }
    plate.position().set(px, 0, pz);
    plate.rotation().y = yaw;
    g.world.add(&plate);
}

#[derive(Clone, Copy, PartialEq)]
enum Plate {
    Eaten,
    Stack(u32),
    FaceDown,
    Trodden,
}

/// The coated board's whites (sRGB 0..1), by the old colour, gone dull
const BOARD: [[f64; 3]; 4] = [[0.86, 0.84, 0.79], [0.84, 0.80, 0.73], [0.82, 0.77, 0.70], [0.87, 0.82, 0.74]];
/// The party inks, by the old colour (a raspberry, a royal blue, a gold, a
/// green, a purple), as printed on board and faded
const INK: [[f64; 3]; 5] = [[0.70, 0.28, 0.40], [0.30, 0.40, 0.66], [0.80, 0.66, 0.30], [0.36, 0.56, 0.38], [0.52, 0.34, 0.58]];

/// The top face from above (u along x, v against z, as the game lays a
/// box's top): the coated white, the pleats' light and shade across the
/// wall and flange; printed: dots on an ink band, ink rays, or confetti;
/// eaten from: frosting smeared and dried, grease soaked in round it, the
/// fork's scrapes, crumbs; trodden: a sole's grime across it; dust.
fn top_tex(board: usize, ink: Option<usize>, salt: u32, how: Plate) -> Tex {
    canvas_tex_opt(256, 256, false, move |ctx, w, h| {
        let seed = salt ^ 0x9a7e;
        let mut r = SplitMix64::new(salt, 0x2b1);
        let design = (salt >> 5) % 3;
        let smear = (r.range(-0.03, 0.03), r.range(-0.03, 0.03), r.range(0.0, PI), r.range(0.012, 0.022));
        let frost = [[0.80, 0.74, 0.62], [0.78, 0.62, 0.62], [0.36, 0.24, 0.17]][(salt >> 9) as usize % 3];
        let grease = (r.range(-0.03, 0.03), r.range(-0.03, 0.03), r.range(0.018, 0.03));
        let sole = (r.range(0.0, PI), r.range(-0.02, 0.02));
        let eaten = matches!(how, Plate::Eaten | Plate::Trodden | Plate::FaceDown);
        let b = BOARD[board];
        fill(ctx, w, h, |u, v| {
            let x = (u * 2.0 - 1.0) * PLATE_R;
            let z = ((1.0 - v) * 2.0 - 1.0) * PLATE_R;
            let rr = (x * x + z * z).sqrt();
            let a = x.atan2(z);
            let n = fbm(seed, u * 24.0, v * 24.0, 3);
            let mut c = b;
            if let Some(k) = ink {
                let ic = INK[k];
                let on_flange = rr > PLATE_FLANGE.0 && rr < PLATE_FLANGE.1;
                let ink_here = match design {
                    // a band of ink round the flange, white dots on it
                    0 => {
                        let s = 0.0105;
                        let (ci, cj) = (((x / s) + 0.5).floor(), ((z / s) + 0.5).floor());
                        let (dx, dz) = (x - ci * s, z - cj * s);
                        on_flange && (dx * dx + dz * dz).sqrt() > 0.0022 || (rr - PLATE_WELL - 0.002).abs() < 0.0008
                    }
                    // rays of ink across the wall and flange, between the pleats
                    1 => rr > PLATE_WELL + 0.004 && rr < PLATE_FLANGE.1 && (a * 16.0 / PI).rem_euclid(2.0) < 0.8,
                    // confetti over all of it, and the flange's edge in ink
                    _ => {
                        let s = 0.009;
                        let (ci, cj) = ((x / s).floor(), (z / s).floor());
                        let hx = hash(seed ^ 1, ci as i32, cj as i32);
                        let hz = hash(seed ^ 2, ci as i32, cj as i32);
                        let (dx, dz) = (x - (ci + 0.25 + 0.5 * hx) * s, z - (cj + 0.25 + 0.5 * hz) * s);
                        let ang = hash(seed ^ 3, ci as i32, cj as i32) * PI;
                        let al = dx * ang.cos() + dz * ang.sin();
                        let ac = -dx * ang.sin() + dz * ang.cos();
                        (al.abs() < 0.0018 && ac.abs() < 0.0007 && hash(seed ^ 4, ci as i32, cj as i32) < 0.6) || (rr > 0.084 && rr < PLATE_FLANGE.1)
                    }
                };
                if ink_here {
                    c = ic;
                }
            } else if (rr - 0.081).abs() < 0.0006 {
                // a plain one: the flange's embossed line
                c = mix(c, [0.70, 0.68, 0.63], 0.5);
            }
            // the ink and the coating faded
            c = mix(c, [0.86, 0.84, 0.79], 0.22);
            // the pleats: light on one side of each, shade on the other
            let pl = sstep(PLATE_WELL, PLATE_WELL + 0.008, rr);
            c = mix(c, [0.0, 0.0, 0.0], pl * 0.06 * (1.0 + (a * 32.0).sin()));
            if eaten {
                // the frosting: smeared with a fork, dried hard and cracked
                let (sx, sz, sa, sl) = smear;
                let (dx, dz) = (x - sx, z - sz);
                let al = dx * sa.cos() + dz * sa.sin();
                let ac = -dx * sa.sin() + dz * sa.cos();
                let body = sstep(1.0, 0.7, ((al / sl).powi(2) + (ac / (sl * 0.45)).powi(2)).sqrt() + 0.3 * (n - 0.5));
                let tines = (0.5 + 0.5 * (ac * 2600.0).sin()).powi(3) * 0.3;
                let crack = (1.0 - ((vn(seed ^ 6, x * 900.0, z * 900.0) - 0.5).abs() * 14.0).min(1.0)) * 0.5;
                c = mix(c, mix(frost, [0.30, 0.26, 0.22], crack), body * (0.85 - tines));
                // grease soaked into the coating round it
                let (gx, gz, gr) = grease;
                let gd = (((x - gx).powi(2) + (z - gz).powi(2)).sqrt() / gr) + 0.25 * (n - 0.5);
                c = mix(c, [0.62, 0.56, 0.42], sstep(1.0, 0.6, gd) * 0.35);
                // crumbs and specks
                let sp = vn(seed ^ 8, x * 1400.0, z * 1400.0);
                if sp > 0.86 && rr < PLATE_WELL {
                    c = mix(c, [0.48, 0.38, 0.26], (sp - 0.86) * 5.0);
                }
            }
            if how == Plate::Trodden {
                // a sole's grime: the tread's bars across it
                let (sa, so) = sole;
                let along = x * sa.cos() + z * sa.sin();
                let across = -x * sa.sin() + z * sa.cos() - so;
                let in_sole = sstep(0.05, 0.04, across.abs()) * sstep(0.10, 0.07, along.abs());
                let bars = sstep(0.3, 0.6, (along * 260.0).sin()) * (0.6 + 0.4 * n);
                c = mix(c, [0.32, 0.29, 0.25], in_sole * bars * 0.55);
            }
            // dust, and grime at the edge where it met the floor
            c = mix(c, [0.60, 0.58, 0.53], 0.12 + 0.18 * n + if matches!(how, Plate::Stack(_)) { 0.1 } else { 0.0 });
            c = mix(c, [0.45, 0.42, 0.37], sstep(0.084, 0.089, rr) * 0.4);
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// The underside (u along x, v along z: the top's map seen through the
/// board): plain grey-white board, its fibres, the pleats; grease soaked
/// through under where it was eaten from; floor grime
fn under_tex(board: usize, salt: u32, used: bool) -> Tex {
    canvas_tex_opt(128, 128, false, move |ctx, w, h| {
        let seed = salt ^ 0x3d0;
        let mut r = SplitMix64::new(salt, 0x2b1);
        // the top's own draws, to find its grease spot again
        let _ = (r.next(), r.next(), r.next(), r.next());
        let grease = (r.range(-0.03, 0.03), r.range(-0.03, 0.03), r.range(0.018, 0.03));
        let b = mix(BOARD[board], [0.70, 0.68, 0.62], 0.4);
        fill(ctx, w, h, |u, v| {
            let x = (u * 2.0 - 1.0) * PLATE_R;
            let z = (v * 2.0 - 1.0) * PLATE_R;
            let rr = (x * x + z * z).sqrt();
            let n = fbm(seed, u * 30.0, v * 30.0, 3);
            let mut c = mix(b, [0.62, 0.60, 0.55], 0.4 * n);
            c = mix(c, [0.0, 0.0, 0.0], sstep(PLATE_WELL, PLATE_WELL + 0.008, rr) * 0.05 * (1.0 + (x.atan2(z) * 32.0).sin()));
            if used {
                let (gx, gz, gr) = grease;
                let gd = (((x - gx).powi(2) + (z - gz).powi(2)).sqrt() / (gr * 0.8)) + 0.3 * (n - 0.5);
                c = mix(c, [0.56, 0.50, 0.38], sstep(1.0, 0.5, gd) * 0.35);
            }
            c = mix(c, [0.46, 0.43, 0.38], 0.15 + 0.25 * sstep(0.04, 0.0, rr - 0.03) * n);
            [c[0], c[1], c[2], 1.0]
        });
    })
}

mod kit {
    use game::assets::props::prop_part_as_modelled;
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
    pub(super) fn salt(x: f64, z: f64, k: u32) -> u32 {
        ((x * 7919.0).abs() as u32) ^ ((z * 104_729.0).abs() as u32).rotate_left(13) ^ k.wrapping_mul(0x9e37_79b9)
    }
    /// The part `name` of a Blender batch as modelled, added to `grp` in `mat`
    pub(super) fn put(grp: &Obj, name: &str, mat: &Mat) -> Option<Obj> {
        let geo = prop_part_as_modelled(name)?;
        let m = mesh(geo, mat.clone());
        grp.add(&m);
        Some(m)
    }
}
