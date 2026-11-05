//! Batch `t17`'s props in this crate, made real (`docs/PROPS.md` P3): the
//! acorns, the conkers, the pine cone, the pine needles and the fern
//! (Blender batch `t17`; what each real thing is and where its size comes
//! from: `assets/src/blender/props/t17.notes.md`). Each placer keeps its old
//! draws call for call (the comments at each say what a draw now chooses,
//! or that it is kept for its place in the stream).
use super::*;
use game::assets::props::{prop_part_as_modelled, prop_part_info};

// ---------------------------------------------------------------- the kit
/// The Blender part `name` as modelled (its own frame, a scale of one)
fn part(name: &str) -> Option<Geo> {
    prop_part_as_modelled(name)
}

/// The bounds (frame) part `name` was modelled in
fn frame(name: &str) -> ([f64; 3], [f64; 3]) {
    prop_part_info(name).map(|m| (m.fmin, m.fmax)).unwrap_or(([-0.05; 3], [0.05; 3]))
}

/// Part `name` with texture coordinates of its own: `f(p, n)` gives each
/// vertex's (u, v) from where it is and which way it faces in the part's
/// frame. Where `wrap_u`, u runs round the thing (0..1): a triangle across
/// the seam gets its low corners again at u + 1. Made once a thread.
fn uv_part(name: &str, wrap_u: bool, f: impl Fn([f64; 3], [f64; 3]) -> (f64, f64)) -> Option<Geo> {
    thread_local! { static C: RefCell<Vec<(String, Geo)>> = const { RefCell::new(vec![]) }; }
    if let Some(g) = C.with(|c| c.borrow().iter().find(|(n, _)| n == name).map(|(_, g)| g.clone())) {
        return Some(g);
    }
    let g = part(name)?.duplicate();
    {
        let mut gg = g.g();
        let n = gg.position.len() / 3;
        let mut pos = gg.position.clone();
        let mut nrm = gg.normal.clone();
        let mut uv = Vec::with_capacity(n * 2);
        for i in 0..n {
            let p = [f64::from(pos[i * 3]), f64::from(pos[i * 3 + 1]), f64::from(pos[i * 3 + 2])];
            let q = [f64::from(nrm[i * 3]), f64::from(nrm[i * 3 + 1]), f64::from(nrm[i * 3 + 2])];
            let (u, v) = f(p, q);
            uv.push(u as f32);
            uv.push(v as f32);
        }
        let mut idx: Vec<u32> = gg.index.clone().unwrap_or_else(|| (0..n as u32).collect());
        if wrap_u {
            let mut dup: Vec<(u32, u32)> = vec![];
            for t in idx.chunks_exact_mut(3) {
                let us = [uv[t[0] as usize * 2], uv[t[1] as usize * 2], uv[t[2] as usize * 2]];
                let (lo, hi) = (us.iter().cloned().fold(9.0f32, f32::min), us.iter().cloned().fold(-9.0f32, f32::max));
                if hi - lo <= 0.5 {
                    continue;
                }
                for k in t.iter_mut() {
                    if uv[*k as usize * 2] < 0.5 {
                        let d = match dup.iter().find(|(a, _)| *a == *k) {
                            Some(&(_, d)) => d,
                            None => {
                                let d = (pos.len() / 3) as u32;
                                let s = *k as usize;
                                pos.extend_from_slice(&[pos[s * 3], pos[s * 3 + 1], pos[s * 3 + 2]]);
                                nrm.extend_from_slice(&[nrm[s * 3], nrm[s * 3 + 1], nrm[s * 3 + 2]]);
                                uv.extend_from_slice(&[uv[s * 2] + 1.0, uv[s * 2 + 1]]);
                                dup.push((*k, d));
                                d
                            }
                        };
                        *k = d;
                    }
                }
            }
        }
        gg.position = pos;
        gg.normal = nrm;
        gg.uv = uv;
        gg.index = Some(idx);
        gg.touch();
    }
    C.with(|c| c.borrow_mut().push((name.to_string(), g.clone())));
    Some(g)
}

/// Cached once per thread: a texture every placement shares (its painter
/// draws on a stream of its own, so it comes out the same whenever it is made)
macro_rules! t17_kind_tex {
    ($n:expr, $k:expr, $make:expr) => {{
        thread_local! { static T: std::cell::RefCell<Vec<Option<Tex>>> = std::cell::RefCell::new(vec![None; $n]); }
        let k = ($k).min($n - 1);
        T.with(|t| {
            if let Some(x) = &t.borrow()[k] {
                return x.clone();
            }
            let tex = $make(k);
            t.borrow_mut()[k] = Some(tex.clone());
            tex
        })
    }};
}

/// A hash of an integer lattice point, in [0, 1)
fn hash(seed: u32, i: i32, j: i32) -> f64 {
    let mut h = seed ^ (i as u32).wrapping_mul(0x27d4_eb2d) ^ (j as u32).wrapping_mul(0x1656_67b1);
    h = (h ^ (h >> 15)).wrapping_mul(0x85eb_ca6b);
    h = (h ^ (h >> 13)).wrapping_mul(0xc2b2_ae35);
    h ^= h >> 16;
    f64::from(h) / 4_294_967_296.0
}

/// Value noise in [0, 1]
fn vn(seed: u32, x: f64, y: f64) -> f64 {
    let (xf, yf) = (x.floor(), y.floor());
    let (i, j) = (xf as i32, yf as i32);
    let (u, v) = (x - xf, y - yf);
    let (u, v) = (u * u * (3.0 - 2.0 * u), v * v * (3.0 - 2.0 * v));
    let (a, b) = (hash(seed, i, j), hash(seed, i + 1, j));
    let (c, d) = (hash(seed, i, j + 1), hash(seed, i + 1, j + 1));
    a + (b - a) * u + (c - a) * v + (a - b - c + d) * u * v
}

/// Fractal value noise of `oct` octaves, in [0, 1]
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

fn sstep(e0: f64, e1: f64, x: f64) -> f64 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn mix(a: [f64; 3], b: [f64; 3], t: f64) -> [f64; 3] {
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
}

fn scale3(a: [f64; 3], k: f64) -> [f64; 3] {
    [a[0] * k, a[1] * k, a[2] * k]
}

/// `rgba(…)` of an sRGB colour (0-1) at alpha `a`
fn css(c: [f64; 3], a: f64) -> String {
    format!("rgba({:.0},{:.0},{:.0},{:.3})", c[0] * 255.0, c[1] * 255.0, c[2] * 255.0, a.clamp(0.0, 1.0))
}

/// A soft round blob, squashed by the context (never an ellipse: no rim)
fn blob(ctx: &mut Canvas, x: f64, y: f64, r: f64, k: f64, turn: f64, c: [f64; 3], a: f64) {
    ctx.save();
    ctx.translate(x, y);
    ctx.rotate(turn);
    ctx.scale(1.0, k);
    let gr = ctx.create_radial_gradient(0, 0, 0, 0, 0, r);
    gr.add_color_stop(0, &css(c, a));
    gr.add_color_stop(1, &css(c, 0.0));
    ctx.set_fill_style(&gr);
    ctx.begin_path();
    ctx.arc(0, 0, r, 0, 7);
    ctx.fill();
    ctx.restore();
}

/// Fill the canvas pixel by pixel: `f(x, y)` (pixel centres) gives a
/// straight sRGB colour and alpha. Pure (no draws, no canvas calls), so its
/// rows are filled on every core
fn paint_px(ctx: &mut Canvas, w: f64, h: f64, f: impl Fn(f64, f64) -> [f64; 4] + Sync) {
    let n = w as usize;
    let mut img = ctx.create_image_data(w, h);
    par_rows(&mut img.data, n * 4, |j, out| {
        for i in 0..n {
            let c = f(i as f64 + 0.5, j as f64 + 0.5);
            for k in 0..4 {
                out[i * 4 + k] = (c[k].clamp(0.0, 1.0) * 255.0).round() as u8;
            }
        }
    });
    ctx.put_image_data(&img, 0, 0);
}

/// Where a texture is clear, its colour becomes `rgb` (sRGB 0-1), so its
/// smaller mips keep the thing's colour at a cut-out edge instead of black
fn bleed(tex: &Tex, rgb: [f64; 3]) {
    let img = tex.image();
    let mut im = img.borrow_mut();
    for px in im.data.chunks_exact_mut(4) {
        let k = sstep(0.0, 0.5, f64::from(px[3]) / 255.0);
        for (c, b) in px.iter_mut().zip(rgb) {
            *c = (b * 255.0 + (f64::from(*c) - b * 255.0) * k).round().clamp(0.0, 255.0) as u8;
        }
    }
    im.version += 1;
}

/// A salt from where a thing lies (no draw): a painter's own stream starts
/// the same on every floor, so what it paints for each thing mixes this in
fn salt_of(x: f64, z: f64) -> u32 {
    let (a, b) = ((x * 1000.0).round() as i64 as u64, (z * 1000.0).round() as i64 as u64);
    let mut h = a.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ b.wrapping_mul(0xC2B2_AE3D_27D4_EB4F);
    h ^= h >> 29;
    (h ^ (h >> 32)) as u32
}

/// A small deterministic stream of its own (for a painter's strokes, mixed
/// with where the thing lies): no level draw
struct Mini(u64);

impl Mini {
    fn new(seed: u32) -> Mini {
        Mini(u64::from(seed).wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ 0xD1B5_4A32_D192_ED03)
    }
    fn next(&mut self) -> f64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^= z >> 31;
        (z >> 11) as f64 / (1u64 << 53) as f64
    }
    fn range(&mut self, a: f64, b: f64) -> f64 {
        a + (b - a) * self.next()
    }
}

/// The decal material of a mark painted in `tex`, lit as the floor under it
fn mark_mat(tex: &Tex) -> Mat {
    lambert!({ map: tex, transparent: true, depthWrite: false, polygonOffset: true, polygonOffsetFactor: -2, polygonOffsetUnits: -2 })
}

/// Things laid in a cluster keep a little apart: `at` moved out from those
/// already down (each `r` round) until it is clear of them (no draw)
fn clear_of(at: (f64, f64), r: f64, down: &[(f64, f64, f64)]) -> (f64, f64) {
    let (mut x, mut z) = at;
    for _ in 0..6 {
        let mut moved = false;
        for &(ox, oz, or) in down {
            let (dx, dz) = (x - ox, z - oz);
            let d = dx.hypot(dz);
            let need = (r + or) * 0.92;
            if d < need {
                let (ux, uz) = if d > 1e-6 { (dx / d, dz / d) } else { (0.6, 0.8) };
                x = ox + ux * need;
                z = oz + uz * need;
                moved = true;
            }
        }
        if !moved {
            break;
        }
    }
    (x, z)
}

/// Rotate (x, y) by `a` about the origin
fn rot2(x: f64, y: f64, a: f64) -> (f64, f64) {
    let (c, s) = (a.cos(), a.sin());
    (x * c - y * s, x * s + y * c)
}

// ================================================================= acorns
/// The acorn (`t17.py`): the nut's length and widest radius, its base over
/// the cup's foot, the cup's rim; its profile (a fraction of NUT_R along it)
const NUT_L: f64 = 0.025;
const NUT_R: f64 = 0.0076;
const CUP_FLOOR: f64 = 0.0022;
const CUP_RIM: f64 = 0.0084;
/// The cupped acorn laid down: its turn about z and where the cup's foot
/// went (`t17.py` acorn_pair, printed by the build)
const PAIR_TILT: f64 = -1.810796;
const PAIR_PIVOT: (f64, f64) = (-0.013319, 0.010592);
/// The empty cup: tipped about x, lifted onto the floor
const CUP_LOOSE_TILT: f64 = 0.22;
const CUP_LOOSE_LIFT: f64 = 0.001185;
/// The nut's map runs from the middle of its base (the scar) round its
/// side to its style: the scar's share of it
const NUT_U_SCAR: f64 = 0.6 * NUT_R / (0.6 * NUT_R + NUT_L + 0.0016);

/// (u, v) on a nut whose axis point is `s` along it from its base, `ox, oy`
/// across it (`oy` toward the side turned up when it lies), the base's
/// middle `rho` from the axis when `s` is 0: u from the scar's middle round
/// to the tip, v the angle from its top (both sides alike)
fn nut_uv(s: f64, ox: f64, oy: f64) -> (f64, f64) {
    let tot = 0.6 * NUT_R + NUT_L + 0.0016;
    let rho = ox.hypot(oy);
    let u = if s < 2e-5 { rho / tot } else { (0.6 * NUT_R + s) / tot };
    (u.clamp(0.0, 1.0), ox.abs().atan2(oy) / PI)
}

/// The bare nut: its axis along x at y = NUT_R, its base at x = -NUT_L / 2
fn acorn_nut_geo() -> Option<Geo> {
    uv_part("Real__acornNut", false, |p, _| nut_uv(p[0] + NUT_L / 2.0, p[2], p[1] - NUT_R))
}

/// A point of the cupped acorn's frame in the cup's own (standing: its foot
/// at the origin, its axis +y)
fn pair_local(p: [f64; 3]) -> [f64; 3] {
    let (x, y) = rot2(p[0] - PAIR_PIVOT.0, p[1] - PAIR_PIVOT.1, -PAIR_TILT);
    [x, y, p[2]]
}

fn acorn_pair_nut_geo() -> Option<Geo> {
    uv_part("Real__acornPairNut", false, |p, _| {
        let q = pair_local(p);
        // the side turned up as it lies is the standing frame's -x
        nut_uv(q[1] - CUP_FLOOR, q[2], -q[0])
    })
}

/// (u, v) on a cup in its standing frame: u round it, v up its outside
/// (0..0.5) or its inside (0.5..1) from its foot to its rim
fn cup_uv(q: [f64; 3], n: [f64; 3]) -> (f64, f64) {
    let rho = q[0].hypot(q[2]);
    let u = (q[2].atan2(q[0]) / (2.0 * PI) + 1.0) % 1.0;
    let h = (q[1] / CUP_RIM).clamp(0.0, 1.0);
    let inward = if rho > 1e-6 { (n[0] * q[0] + n[2] * q[2]) / rho } else { 0.0 };
    let inside = inward < -0.25 && q[1] > CUP_FLOOR * 0.6;
    (u, if inside { 0.5 + 0.5 * h } else { 0.5 * h })
}

fn acorn_pair_cup_geo() -> Option<Geo> {
    uv_part("Real__acornPairCup", true, |p, n| {
        let (nx, ny) = rot2(n[0], n[1], -PAIR_TILT);
        cup_uv(pair_local(p), [nx, ny, n[2]])
    })
}

fn acorn_cup_geo() -> Option<Geo> {
    uv_part("Real__acornCup", true, |p, n| {
        // undo the tip about x and the lift
        let (y, z) = rot2(p[1] - CUP_LOOSE_LIFT, p[2], -CUP_LOOSE_TILT);
        let (ny, nz) = rot2(n[1], n[2], -CUP_LOOSE_TILT);
        cup_uv([p[0], y, z], [n[0], ny, nz])
    })
}

/// An acorn's nut (u from its scar round to its style, v its angle from its
/// top): `k` its colour (0 ripe brown, 1 old and dark, 2 dried pale), `holed`
/// a weevil's exit hole in it. Fine stripes along it, the scar pale and
/// rough within a dark ring, the tip's felt grey, blotches of decay, dust
#[allow(clippy::excessive_precision)]
fn nut_tex(k: usize, holed: bool) -> Tex {
    t17_kind_tex!(6, k * 2 + holed as usize, |kk: usize| {
        let (k, holed) = (kk / 2, kk % 2 == 1);
        let base = [[0.50, 0.32, 0.16], [0.33, 0.21, 0.12], [0.55, 0.45, 0.33]][k];
        let seed = 0xac0 + kk as u32 * 13;
        canvas_tex_opt(128, 64, false, move |ctx, w, h| {
            paint_px(ctx, w, h, |px, py| {
                let (u, v) = (px / w, 1.0 - py / h);
                let mut c = base;
                // fine stripes along it (lines of one angle), a little wavy
                let st = fbm(seed, u * 6.0, v * 26.0 + 2.0 * fbm(seed ^ 3, u * 3.0, v * 3.0, 2), 3);
                c = scale3(c, 0.86 + 0.26 * st);
                // its shoulder toward the tip a shade darker, the tip's felt
                // greyer and paler round the style
                c = mix(c, scale3(c, 0.8), sstep(0.70, 0.85, u));
                c = mix(c, [0.55, 0.50, 0.42], 0.75 * sstep(0.90, 0.97, u));
                c = mix(c, scale3(c, 0.55), 0.6 * (1.0 - ((u - 0.88) / 0.02).abs()).max(0.0));
                // the scar where it sat in the cup: pale, rough, a dark ring
                let scar = 1.0 - sstep(NUT_U_SCAR - 0.02, NUT_U_SCAR + 0.01, u);
                let rough = fbm(seed ^ 9, u * 40.0, v * 40.0, 3);
                c = mix(c, mix([0.62, 0.55, 0.42], [0.48, 0.41, 0.30], rough), scar);
                let ring = (1.0 - ((u - NUT_U_SCAR - 0.012) / 0.012).abs()).max(0.0);
                c = mix(c, scale3(base, 0.45), 0.7 * ring);
                // the part that was in the cup: paler, never browned in the sun
                c = mix(c, mix(c, [0.62, 0.55, 0.38], 0.35), sstep(NUT_U_SCAR + 0.25, NUT_U_SCAR + 0.05, u) * (1.0 - scar));
                // decay: dark blotches, a dull bloom of dust on the top
                let rot = sstep(0.68, 0.78, fbm(seed ^ 0x51, u * 9.0, v * 5.0, 4));
                c = mix(c, scale3(base, 0.5), 0.6 * rot);
                c = mix(c, [0.56, 0.52, 0.46], 0.18 * (1.0 - sstep(0.0, 0.5, v)));
                if holed {
                    // the weevil larva's exit hole: round, 2.5 mm, on the top
                    let d = ((u - 0.52) / 0.040).hypot(v / 0.050);
                    c = mix(c, [0.36, 0.26, 0.17], 0.8 * (1.0 - sstep(1.0, 1.5, d)));
                    c = mix(c, [0.05, 0.035, 0.03], 1.0 - sstep(0.75, 0.95, d));
                }
                [c[0], c[1], c[2], 1.0]
            });
        })
    })
}

/// A cup (u round it, v its outside then its inside, foot to rim): felted
/// grey-brown scales in rows, their tips paler and their edges dark, as
/// the model's (13 round, 6 rows); the rim's lip; the inside pale and
/// silky, a ring where the nut's base sat; the stalk's stub woody
fn cup_tex() -> Tex {
    t17_kind_tex!(1, 0usize, |_k: usize| {
        let seed = 0xc0b;
        canvas_tex_opt(256, 128, true, move |ctx, w, h| {
            paint_px(ctx, w, h, |px, py| {
                let (u, v) = (px / w, 1.0 - py / h);
                let felt = fbm(seed, u * 60.0, v * 60.0, 3);
                if v >= 0.5 {
                    let hh = (v - 0.5) * 2.0;
                    let mut c = mix([0.66, 0.58, 0.44], [0.58, 0.50, 0.37], felt);
                    c = mix(c, [0.40, 0.31, 0.21], 0.5 * (1.0 - ((hh - 0.28) / 0.06).abs()).max(0.0));
                    c = mix(c, [0.46, 0.40, 0.31], 0.5 * sstep(0.9, 1.0, hh));
                    return [c[0], c[1], c[2], 1.0];
                }
                let hh = v * 2.0;
                let rows = 6.0;
                let kk = hh * rows;
                let row = kk.floor();
                let fv = kk - row;
                let fu = (u * 13.0 + 0.5 * (row as i32 % 2) as f64).fract();
                let mut c = mix([0.56, 0.50, 0.40], [0.47, 0.41, 0.32], felt);
                // each scale paler toward its tip, its outline dark
                c = mix(c, [0.66, 0.60, 0.50], 0.5 * fv.powf(1.5));
                let edge = (1.0 - sstep(0.0, 0.12, fu.min(1.0 - fu))).max(1.0 - sstep(0.85, 0.98, 1.0 - fv));
                c = mix(c, [0.26, 0.20, 0.15], 0.65 * edge);
                // the lip of the rim, the stub at the foot
                c = mix(c, [0.52, 0.46, 0.37], sstep(0.92, 0.98, hh));
                c = mix(c, [0.30, 0.22, 0.14], 1.0 - sstep(0.05, 0.12, hh));
                [c[0], c[1], c[2], 1.0]
            });
        })
    })
}

// R593 acorn/nut cluster: 6% chance per level. Acorns of the pedunculate
// oak (Blender batch `t17`): most fallen bare from their cups, a few still in
// theirs, an empty cup; ripe brown, old and dark, or dried pale; one in
// three of the bare holed by a weevil.
pub(super) fn place_acorns(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let nut_colors = [0x8b5a2b, 0x6b3f1f, 0x9b7a3b];
    let cluster = group();
    let count = 2 + (rnd() * 4.0).floor() as i32;
    let cup_mat = lambert!({ map: &cup_tex() });
    let mut down: Vec<(f64, f64, f64)> = vec![];
    for _i in 0..count {
        // the old acorn's draws: its colour (the nut's), where it lies, its
        // turn, its tilt (now whether it lies bare, in its cup or is a cup)
        let col = choice(&nut_colors);
        let k = nut_colors.iter().position(|&c| c == col).unwrap_or(0);
        let ax = (rnd() - 0.5) * 0.13;
        let az = (rnd() - 0.5) * 0.13;
        let yaw = rnd() * PI * 2.0;
        let tilt = (rnd() - 0.5) * 0.4;
        let t = tilt / 0.4 + 0.5;
        let acorn = group();
        let r = if t < 0.55 {
            // bare: fallen from its cup; one in three holed
            let holed = t < 0.55 / 3.0;
            if let Some(geo) = acorn_nut_geo() {
                acorn.add(&mesh(geo, lambert!({ map: &nut_tex(k, holed) })));
            }
            0.013
        } else if t < 0.85 {
            let nut = lambert!({ map: &nut_tex(k, false) });
            if let (Some(n), Some(c)) = (acorn_pair_nut_geo(), acorn_pair_cup_geo()) {
                acorn.add(&mesh(n, &nut));
                acorn.add(&mesh(c, &cup_mat));
            }
            0.015
        } else {
            if let Some(geo) = acorn_cup_geo() {
                acorn.add(&mesh(geo, &cup_mat));
            }
            0.009
        };
        let (ax, az) = clear_of((ax, az), r, &down);
        down.push((ax, az, r));
        acorn.position().set(ax, 0, az);
        acorn.rotation().y = yaw;
        cluster.add(&acorn);
    }
    let (px, pz) = jitter(g, x, z, 2.5);
    cluster.position().set(px, 0, pz);
    g.world.add(&cluster);
}

// ================================================================ conkers
/// The seed's half sizes and its scar's middle (`t17.py` CK, CK_HILUM)
const CK: [f64; 3] = [0.0155, 0.0120, 0.0140];
const CK_HILUM: [f64; 3] = [0.30, 0.50, 0.81];

/// The scar's middle, a unit direction
fn ck_hilum() -> [f64; 3] {
    let l = (CK_HILUM[0] * CK_HILUM[0] + CK_HILUM[1] * CK_HILUM[1] + CK_HILUM[2] * CK_HILUM[2]).sqrt();
    [CK_HILUM[0] / l, CK_HILUM[1] / l, CK_HILUM[2] / l]
}

/// (u, v) on a conker: round it (u) and up it (v) about its middle, by the
/// direction from its middle scaled to a sphere (as the model's warp)
fn conker_geo(name: &'static str) -> Option<Geo> {
    let (lo, hi) = frame(name);
    // the ellipsoid's middle: its foot flattened, so a little under the box's
    let c = [(lo[0] + hi[0]) / 2.0, lo[1] + CK[1] * (0.45 + 0.55 * 0.45), (lo[2] + hi[2]) / 2.0];
    uv_part(name, true, move |p, _| {
        let d = [(p[0] - c[0]) / CK[0], (p[1] - c[1]) / CK[1], (p[2] - c[2]) / CK[2]];
        let l = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt().max(1e-9);
        let u = (d[2].atan2(d[0]) / (2.0 * PI) + 1.0) % 1.0;
        let v = (d[1] / l).clamp(-1.0, 1.0).acos() / PI;
        (u, 1.0 - v)
    })
}

/// A conker (its map by direction from its middle): glossy mahogany with
/// paler wavy lines round the scar like a grain, the scar (hilum) pale buff,
/// matt and finely pitted inside a dark edge; `k` its age (0 fresh, 1 dried
/// dull, 2 old, dark and wrinkled)
fn conker_tex(k: usize) -> Tex {
    t17_kind_tex!(3, k, |k: usize| {
        let base = [[0.40, 0.18, 0.08], [0.34, 0.17, 0.09], [0.25, 0.13, 0.08]][k];
        let seed = 0xc04 + k as u32 * 7;
        canvas_tex_opt(256, 128, true, move |ctx, w, h| {
            paint_px(ctx, w, h, |px, py| {
                let (u, v) = (px / w, 1.0 - py / h);
                let (phi, th) = (u * 2.0 * PI, (1.0 - v) * PI);
                let d = [th.sin() * phi.cos(), th.cos(), th.sin() * phi.sin()];
                let hl = ck_hilum();
                let c0 = d[0] * hl[0] + d[1] * hl[1] + d[2] * hl[2];
                let ang = c0.clamp(-1.0, 1.0).acos();
                // the grain: wavy bands round the scar
                let wob = fbm(seed, d[0] * 3.0 + 7.0, d[2] * 3.0 + d[1] * 2.0, 3);
                let band = ((ang * 26.0 + wob * 9.0).sin() * 0.5 + 0.5).powf(3.0);
                let mut c = mix(base, scale3(base, 1.45), 0.45 * band);
                c = mix(c, scale3(base, 0.75), sstep(1.8, 2.8, ang) * 0.5);
                if k == 2 {
                    // dried: fine wrinkles darker
                    let wr = fbm(seed ^ 5, d[0] * 14.0 + 3.0, d[2] * 14.0 + d[1] * 9.0, 3);
                    c = mix(c, scale3(c, 0.6), sstep(0.55, 0.62, wr) * (1.0 - sstep(0.62, 0.70, wr)));
                }
                // the scar
                let edge = 0.72f64.acos();
                let inside = 1.0 - sstep(edge - 0.03, edge, ang);
                let pit = fbm(seed ^ 0x77, d[0] * 30.0 + 1.0, d[2] * 30.0 + d[1] * 20.0, 3);
                let scar = mix([0.68, 0.60, 0.46], [0.55, 0.47, 0.35], pit);
                let scar = mix(scar, [0.42, 0.33, 0.24], 0.4 * sstep(0.62, 0.72, fbm(seed ^ 0x13, d[0] * 6.0, d[2] * 6.0 + d[1] * 4.0, 3)));
                c = mix(c, scar, inside);
                c = mix(c, scale3(base, 0.45), 0.7 * (1.0 - ((ang - edge) / 0.035).abs()).max(0.0));
                // dust on what faces up
                c = mix(c, [0.52, 0.48, 0.43], 0.12 * sstep(0.3, 1.0, d[1]));
                [c[0], c[1], c[2], 1.0]
            });
        })
    })
}

/// The husk's outside (box-mapped): dried brown leather with olive left in
/// patches, warty, darker toward the prickles' feet
fn husk_out_tex() -> Tex {
    t17_kind_tex!(1, 0usize, |_k: usize| {
        canvas_tex_opt(256, 256, true, move |ctx, w, h| {
            paint_px(ctx, w, h, |px, py| {
                let (u, v) = (px / w, py / h);
                let n = fbm(0x4a5, u * 9.0, v * 9.0, 4);
                let mut c = mix([0.38, 0.29, 0.18], [0.29, 0.22, 0.14], n);
                c = mix(c, [0.38, 0.35, 0.21], 0.55 * sstep(0.6, 0.75, fbm(0x4a6, u * 4.0, v * 4.0, 3)));
                let wart = fbm(0x4a7, u * 70.0, v * 70.0, 2);
                c = mix(c, scale3(c, 0.6), sstep(0.68, 0.76, wart));
                c = mix(c, [0.52, 0.46, 0.36], 0.25 * sstep(0.72, 0.8, fbm(0x4a8, u * 30.0, v * 30.0, 2)));
                [c[0], c[1], c[2], 1.0]
            });
        })
    })
}

/// The husk's inside: once white and soft, dried a pale tan, browned in
/// patches, fibrous
fn husk_in_tex() -> Tex {
    t17_kind_tex!(1, 0usize, |_k: usize| {
        canvas_tex_opt(256, 256, true, move |ctx, w, h| {
            paint_px(ctx, w, h, |px, py| {
                let (u, v) = (px / w, py / h);
                let n = fbm(0x1a5, u * 8.0, v * 8.0, 4);
                let mut c = mix([0.60, 0.50, 0.36], [0.48, 0.38, 0.26], n);
                c = mix(c, [0.42, 0.30, 0.18], 0.6 * sstep(0.58, 0.72, fbm(0x1a6, u * 5.0, v * 5.0, 3)));
                let fib = fbm(0x1a7, u * 90.0, v * 12.0, 2);
                c = scale3(c, 0.9 + 0.15 * fib);
                [c[0], c[1], c[2], 1.0]
            });
        })
    })
}

// R602 chestnut cluster: 5% chance per level. Horse chestnuts (Blender batch
// `t17`): conkers fallen bare, and others still sitting in their husks split
// open into three valves, the husks dried brown; fresh and glossy, dried
// dull or old and wrinkled.
pub(super) fn place_chestnuts(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let cluster = group();
    let count = 2 + (rnd() * 3.0).floor() as i32;
    let out_mat = lambert!({ map: &husk_out_tex() });
    let in_mat = lambert!({ map: &husk_in_tex() });
    let mut down: Vec<(f64, f64, f64)> = vec![];
    let mut placed: Vec<(Obj, f64, f64, bool)> = vec![];
    for _i in 0..count {
        // the old chestnut's draws: its husk (60%: now a husk split open with
        // the conker in it), where it lies, its turn
        let husked = rnd() < 0.60;
        let cx = (rnd() - 0.5) * 0.13;
        let cz = (rnd() - 0.5) * 0.13;
        let yaw = rnd() * PI * 2.0;
        let chestnut = group();
        chestnut.rotation().y = yaw;
        placed.push((chestnut, cx, cz, husked));
    }
    let (px, pz) = jitter(g, x, z, 2.5);
    // spread as they fell (twice the old spacing: a husk is 6 cm across)
    for (chestnut, cx, cz, husked) in placed {
        let r = if husked { 0.036 } else { 0.017 };
        let (cx, cz) = clear_of((cx * 2.0, cz * 2.0), r, &down);
        down.push((cx, cz, r));
        let salt = salt_of(px + cx, pz + cz);
        let age = (salt % 3) as usize;
        let seed_mat = if age == 0 {
            phong!({ map: &conker_tex(age), specular: 0x262019, shininess: 26 })
        } else {
            lambert!({ map: &conker_tex(age) })
        };
        if husked {
            for (name, mat) in [("Real__conkerHuskOut", &out_mat), ("Real__conkerHuskIn", &in_mat)] {
                if let Some(geo) = part(name) {
                    chestnut.add(&mesh(geo, mat));
                }
            }
            if let Some(geo) = conker_geo("Real__conkerSeated") {
                chestnut.add(&mesh(geo, &seed_mat));
            }
        } else if let Some(geo) = conker_geo("Real__conker") {
            chestnut.add(&mesh(geo, &seed_mat));
        }
        chestnut.position().set(cx, 0, cz);
        cluster.add(&chestnut);
    }
    cluster.position().set(px, 0, pz);
    g.world.add(&cluster);
}

// ============================================================== pine cone
/// The cone's woody scales (box-mapped): dark red-brown, darker in their
/// grooves; its scales' ends weathered grey-brown
fn cone_tex(apo: bool) -> Tex {
    t17_kind_tex!(2, apo as usize, |k: usize| {
        canvas_tex_opt(128, 128, true, move |ctx, w, h| {
            paint_px(ctx, w, h, |px, py| {
                let (u, v) = (px / w, py / h);
                let n = fbm(0x9c0 + k as u32, u * 14.0, v * 14.0, 4);
                let c = if k == 1 {
                    let mut c = mix([0.47, 0.40, 0.31], [0.36, 0.30, 0.23], n);
                    c = mix(c, [0.30, 0.22, 0.15], 0.5 * sstep(0.62, 0.75, fbm(0x9c5, u * 40.0, v * 40.0, 2)));
                    c
                } else {
                    let mut c = mix([0.36, 0.22, 0.13], [0.26, 0.16, 0.10], n);
                    c = mix(c, [0.44, 0.33, 0.22], 0.3 * sstep(0.6, 0.8, fbm(0x9c6, u * 30.0, v * 6.0, 2)));
                    c
                };
                [c[0], c[1], c[2], 1.0]
            });
        })
    })
}

// R594 pine cone: 6% chance per level. A Scots pine's cone, open (Blender
// batch `t17`): its woody scales spread from the axis, their rhombic ends
// keeled with the small umbo; weathered grey-brown, the scales' hidden faces
// darker.
pub(super) fn place_pine_cones(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let cone_colors = [0x604020, 0x503018, 0x705030, 0x402010];
    // the old colour's draw: how weathered it is (brown, darker, grey and
    // dry, dark with damp)
    let col = choice(&cone_colors);
    const TINT: [(u32, u32); 4] = [(0xffffff, 0xffffff), (0xd8ccc0, 0xd8d0c8), (0xf0e8e0, 0xffffff), (0xa89a8c, 0xa8a098)];
    let (ts, ta) = TINT[cone_colors.iter().position(|&c| c == col).unwrap_or(0)];
    let cone = group();
    for (name, mat) in [
        ("Real__pineCone", lambert!({ map: &cone_tex(false), color: ts })),
        ("Real__pineConeApo", lambert!({ map: &cone_tex(true), color: ta })),
    ] {
        if let Some(geo) = part(name) {
            cone.add(&mesh(geo, &mat));
        }
    }
    let (px, pz) = jitter(g, x, z, 2.5);
    cone.position().set(px, 0, pz);
    cone.rotation().y = rnd() * PI * 2.0;
    g.world.add(&cone);
}

// =========================================================== pine needles
/// A drift of shed needles on the floor, `size` m across: broken needles
/// and their bits lying every way, finer toward its edge, in a film of the
/// grit and dust that came in with them; soft and uneven (a mark, `r07`)
fn needle_drift_tex(salt: u32, size: f64) -> Tex {
    let px = 384.0;
    canvas_tex_opt(px, px, false, move |ctx, w, h| {
        let mut r = Mini::new(salt ^ 0x9ee);
        ctx.clear_rect(0, 0, w, h);
        let ppm = w / size;
        // the dust and grit under it, in soft patches
        for _ in 0..14 {
            let (a, d) = (r.range(0.0, 2.0 * PI), r.range(0.0, 0.32) * w);
            let (x, y) = (w / 2.0 + a.cos() * d, h / 2.0 + a.sin() * d);
            blob(ctx, x, y, r.range(0.05, 0.16) * w, r.range(0.5, 1.0), r.range(0.0, PI), [0.30, 0.26, 0.20], r.range(0.10, 0.22));
        }
        // the bits: needles broken short, every way, densest in the middle
        ctx.set_line_cap("round");
        for i in 0..240 {
            let (a, d) = (r.range(0.0, 2.0 * PI), r.next().powf(0.8) * 0.45 * w);
            let (x, y) = (w / 2.0 + a.cos() * d, h / 2.0 + a.sin() * d);
            let len = r.range(0.004, 0.030) * ppm * (1.0 - 0.5 * d / (0.45 * w));
            let t = r.range(0.0, 2.0 * PI);
            let tone = r.next();
            let c = mix([0.42, 0.26, 0.13], [0.30, 0.24, 0.17], tone);
            let fade = 1.0 - sstep(0.25, 0.45, d / w);
            ctx.set_stroke_style(&css(c, 0.85 * fade));
            ctx.set_line_width(if i % 3 == 0 { 1.6 } else { 1.1 });
            ctx.begin_path();
            ctx.move_to(x, y);
            let bend = r.range(-0.15, 0.15);
            ctx.quadratic_curve_to(
                x + (t + bend).cos() * len * 0.5,
                y + (t + bend).sin() * len * 0.5,
                x + t.cos() * len,
                y + t.sin() * len,
            );
            ctx.stroke();
        }
        // a few scale flakes and crumbs of bark
        for _ in 0..30 {
            let (a, d) = (r.range(0.0, 2.0 * PI), r.next() * 0.4 * w);
            blob(
                ctx,
                w / 2.0 + a.cos() * d,
                h / 2.0 + a.sin() * d,
                r.range(1.0, 2.6),
                r.range(0.4, 1.0),
                r.range(0.0, PI),
                [0.33, 0.22, 0.13],
                0.7,
            );
        }
    })
}

// R598 pine needle cluster: 7% chance per level. Needles a Scots pine shed
// (Blender batch `t17`): pairs still in their papery sheaths and single
// needles broken from them, rust brown, grey-brown or dark with damp, on a
// drift of their own broken bits and dust.
pub(super) fn place_pine_needles(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.93 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let needle_colors = [0x304018, 0x405020, 0x203010];
    // shed: rust brown, grey-brown, dark brown with damp (the old greens)
    const SHED: [u32; 3] = [0x7a4a24, 0x6a5a44, 0x4a3420];
    let sheath = lambert!({ color: 0x6e645a });
    let mats: Vec<Mat> = SHED.iter().map(|&c| lambert!({ color: c })).collect();
    let cluster = group();
    let count = 10 + (rnd() * 10.0).floor() as i32;
    for i in 0..count {
        // the old needle's draws: its colour, its turn, where it lies
        let col = choice(&needle_colors);
        let k = needle_colors.iter().position(|&c| c == col).unwrap_or(0);
        let yaw = rnd() * PI * 2.0;
        let (nx, nz) = ((rnd() - 0.5) * 0.14, (rnd() - 0.5) * 0.14);
        // pairs mostly, a single broken from its pair every third
        let w = ["A", "B", "C"][(i % 3) as usize];
        let nd = group();
        if let Some(geo) = part(&format!("Real__pineNeedles{w}")) {
            nd.add(&mesh(geo, &mats[k]));
        }
        if w != "C" {
            if let Some(geo) = part(&format!("Real__pineSheath{w}")) {
                nd.add(&mesh(geo, &sheath));
            }
        }
        // one lying over another sits on it: a needle's thickness higher
        // for each laid before it near it (no draw)
        nd.position().set(nx, 0.00035 * (i % 4) as f64, nz);
        nd.rotation().y = yaw;
        cluster.add(&nd);
    }
    let (px, pz) = jitter(g, x, z, 2.4);
    let size = 0.30;
    let drift = mesh(plane_geo!(size, size), mark_mat(&needle_drift_tex(salt_of(px, pz), size)));
    drift.rotation().x = -PI / 2.0;
    drift.position().y = 0.0006;
    cluster.add(&drift);
    cluster.position().set(px, 0, pz);
    g.world.add(&cluster);
}

// =================================================================== fern
/// The frond (`t17.py` FERN_*): the blade's length, its segment pairs, a
/// segment's angle forward from the rachis
const FERN_L: f64 = 0.150;
const FERN_N: usize = 13;
const FERN_ANG: f64 = 64.0 * PI / 180.0;

/// Segment k (from the base) on side ±1: its root's x on the rachis, its
/// length, its half width (as `t17.py` fern_seg)
fn fern_seg(k: usize, side: f64) -> (f64, f64, f64) {
    let t = (k as f64 + if side > 0.0 { 0.0 } else { 0.45 }) / FERN_N as f64;
    let x = FERN_L * (0.02 + 0.86 * t);
    let l = 0.036 * (PI * (0.16 + 0.84 * t).min(1.0).powf(0.72)).sin().max(0.0).powf(0.9) * (0.88 + 0.12 * (t * 5.0).min(1.0)) + 0.004;
    let hw = 0.0036 + 0.0006 * (1.0 - t);
    (x, l, hw)
}

/// Where (x, z) lies on the blade: (how far inside its outline, m; the
/// nearest segment's along (0..1) and across (-1..1), or None on the
/// rachis's wing or the tip)
fn fern_at(x: f64, z: f64) -> (f64, Option<(f64, f64, usize)>) {
    let (ca, sa) = (FERN_ANG.cos(), FERN_ANG.sin());
    let side = if z >= 0.0 { 1.0 } else { -1.0 };
    let mut best = (-1.0f64, None);
    for k in 0..FERN_N {
        let (x0, l, hw) = fern_seg(k, side);
        let (dx, dz) = (x - x0, z * side);
        let s = (dx * ca + dz * sa).clamp(0.0, l);
        let (qx, qz) = (x0 + s * ca, s * sa);
        let across = (x - qx) * (-sa) + (z * side - qz) * ca;
        let d = (x - qx).hypot(z * side - qz);
        // narrowing a little to its blunt tip, its margin finely notched
        let a = s / l;
        let w = hw * (1.0 - 0.18 * a) * (1.0 - 0.06 * (0.5 + 0.5 * (s * 2600.0).cos()));
        let tipd = if s >= l - 1e-9 { d } else { across.abs() };
        let inside = w - tipd;
        if inside > best.0 {
            best = (inside, Some((a, (across / w).clamp(-1.0, 1.0), k)));
        }
    }
    // the wing along the rachis that joins them
    if x > 0.0 && x < FERN_L * 0.97 {
        let wing = 0.0019 * (1.0 - 0.4 * x / FERN_L) - z.abs();
        if wing > best.0 {
            best = (wing, None);
        }
    }
    // the tip: a last pointed lobe
    if x > FERN_L * 0.86 {
        let t = (x - FERN_L * 0.86) / (FERN_L * 0.14);
        let tip = 0.0055 * (1.0 - t).max(0.0).powf(0.8) - z.abs();
        if tip > best.0 {
            best = (tip, None);
        }
    }
    best
}

/// The frond seen from above in its blade's frame (`state` 0 green and
/// browning at its tips, 1 withered olive, 2 dead brown; `back` its back,
/// the sori in a row each side of every segment's midvein): the outline
/// cut out, the midveins and their forked side veins paler, the margins
/// browned and the tips dried, a few holes
fn fern_tex(state: usize, back: bool) -> Tex {
    t17_kind_tex!(6, state * 2 + back as usize, |kk: usize| {
        let (state, back) = (kk / 2, kk % 2 == 1);
        let (lo, hi) = frame("Real__fernFrond");
        let (ex, ez) = (hi[0] - lo[0], hi[2] - lo[2]);
        let th = (512.0 * ez / ex).round().max(32.0);
        let seed = 0xfe0 + kk as u32 * 31;
        let (green, brown) = if back {
            ([[0.38, 0.44, 0.24], [0.48, 0.46, 0.27], [0.52, 0.41, 0.25]][state], [0.50, 0.38, 0.22])
        } else {
            ([[0.27, 0.35, 0.14], [0.40, 0.40, 0.18], [0.44, 0.32, 0.18]][state], [0.42, 0.29, 0.15])
        };
        let tex = canvas_tex_opt(512, th, false, move |ctx, w, h| {
            let px_m = ex / w;
            paint_px(ctx, w, h, |px, py| {
                let x = lo[0] + px / w * ex;
                let z = lo[2] + py / h * ez;
                let (inside, seg) = fern_at(x, z);
                let edge = (inside / px_m + 0.5).clamp(0.0, 1.0);
                let n = fbm(seed, x * 300.0, z * 300.0, 3);
                let mut c = mix(green, scale3(green, 0.8), n);
                // drying from the tips and the margins inward
                let (a, across) = seg.map(|(a, c, _)| (a, c)).unwrap_or((0.0, 0.0));
                let dry = match state {
                    0 => sstep(0.72, 0.98, a) * 0.8,
                    1 => 0.35 + 0.5 * sstep(0.5, 0.95, a),
                    _ => 0.9,
                };
                let blot = fbm(seed ^ 0x33, x * 120.0, z * 120.0, 3);
                c = mix(c, brown, (dry + 0.3 * (blot - 0.5)).clamp(0.0, 1.0));
                c = mix(c, scale3(brown, 0.7), 0.6 * sstep(0.55, 0.95, across.abs()) * (0.4 + 0.6 * dry));
                // the midvein, the side veins forking out to the margin
                if let Some((a, cr, k)) = seg {
                    let mid = 1.0 - sstep(0.06, 0.14, cr.abs());
                    let q = (a * 18.0 - cr.abs() * 1.4 + k as f64 * 0.37).fract();
                    let side = (1.0 - sstep(0.04, 0.12, (q - 0.5).abs())) * sstep(0.12, 0.25, cr.abs()) * (1.0 - sstep(0.7, 0.9, cr.abs()));
                    let pale = if back { [0.62, 0.62, 0.48] } else { [0.52, 0.54, 0.36] };
                    c = mix(c, mix(c, pale, 0.6), mid.max(side * 0.35));
                    if back {
                        // the sori: round, naked, orange-brown gone dark,
                        // halfway between the midvein and the margin
                        let row = (a * 7.0 - 0.6).round();
                        if (1.0..=6.0).contains(&row) {
                            let da = (a * 7.0 - 0.6 - row) / 7.0;
                            let dc = cr.abs() - 0.55;
                            let (_, l, hw) = fern_seg(k, if z >= 0.0 { 1.0 } else { -1.0 });
                            let d = (da * l).hypot(dc * hw) / 0.0009;
                            let sori = [[0.60, 0.36, 0.12], [0.45, 0.28, 0.12], [0.30, 0.20, 0.12]][state];
                            c = mix(c, sori, 1.0 - sstep(0.7, 1.0, d));
                        }
                    }
                } else {
                    // the rachis's wing
                    c = mix(c, scale3(c, 1.1), 0.3);
                }
                // holes eaten through
                let hole = sstep(0.79, 0.81, fbm(seed ^ 0x40, x * 260.0, z * 260.0, 3));
                [c[0], c[1], c[2], edge * (1.0 - hole)]
            });
        });
        bleed(&tex, green);
        tex
    })
}

// R596 fern frond: 6% chance per level. A frond of common polypody broken
// off and dropped (Blender batch `t17`): its blade cut nearly to the rachis
// into narrow blunt segments, lying on its face or its back (the sori in
// rows), green browning from its tips, withered or dead.
pub(super) fn place_ferns(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let leaflet_colors = [0x407030, 0x306028, 0x507838];
    // the old leaflets' colour: now how far the frond has dried (browning
    // at its tips, withered, dead)
    let col = choice(&leaflet_colors);
    let state = leaflet_colors.iter().position(|&c| c == col).unwrap_or(0);
    // the old twelve leaflets' sway phases: kept for their draws; the first
    // turns the frond onto its back (one in three), the second shades it
    let mut ph = [0.0f64; 12];
    for p in ph.iter_mut() {
        *p = rnd();
    }
    let back = ph[0] < 0.35;
    let shade = 0.88 + 0.12 * ph[1];
    let tint = ((shade * 255.0) as u32) * 0x010101;
    let fern = group();
    let body = group();
    if let Some(geo) = part("Real__fernFrond") {
        body.add(&mesh(geo, lambert!({ map: &fern_tex(state, back), color: tint, alphaTest: 0.5, side: DoubleSide })));
    }
    if let Some(geo) = part("Real__fernStipe") {
        let c = [0x5a5a2c, 0x5e5430, 0x5a4428][state];
        body.add(&mesh(geo, lambert!({ color: c })));
    }
    if back {
        // on its back: turned over about its rachis
        let top = frame("Real__fernFrond").1[1].max(frame("Real__fernStipe").1[1]);
        body.rotation().x = PI;
        body.position().y = top;
    }
    // the frond's middle where the old one's was
    body.position().x = -FERN_L * 0.4;
    fern.add(&body);
    let (px, pz) = jitter(g, x, z, 2.4);
    fern.position().set(px, 0, pz);
    fern.rotation().y = rnd() * PI * 2.0;
    g.world.add(&fern);
    // R608 a draught stirs it (the old sway's rate and its first leaflet's
    // phase): it turns a hair on the floor about its stalk's end
    let phase = ph[2] * PI * 2.0;
    g.animate(move |_g, _dt, t| {
        body.rotation().y = (t * 0.4 + phase).sin() * 0.012;
    });
}
