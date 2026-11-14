//! `place_acorns`
use crate::shared::clear_of;
use crate::shared::uv_part;
use game::assets::props::prop_part_as_modelled;
use game::assets::props::prop_part_info;
use game::prelude::*;
use placers_kit::noise::fbm;
use placers_kit::noise::hash;
use placers_kit::noise::mix_unclamped;
use placers_kit::noise::sstep;
use placers_kit::paint::mul;
use placers_kit::paint::paint_px;
use placers_kit::place::jitter;
use placers_kit::place::pick_open;

/// Cached once per thread: a texture every placement shares (its painter
/// draws on a stream of its own, so it comes out the same whenever it is made)
macro_rules! cached_kind_tex {
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

// Acorn/nut cluster: 6% chance per level. Acorns of the pedunculate
// oak (Blender batch `t17`): fallen bare from their cups or still in them,
// now and then an empty cup; ripe brown and glossy, old and dark, or dried
// pale; one in three of the bare holed by a weevil.
pub(crate) fn place_acorns(g: &mut Game, _dist: &[i32]) {
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
        // turn, its tilt (now whether it lies bare (two in five), in its cup
        // (two in five) or is an empty cup)
        let col = choice(&nut_colors);
        let k = nut_colors.iter().position(|&c| c == col).unwrap_or(0);
        let ax = (rnd() - 0.5) * 0.13;
        let az = (rnd() - 0.5) * 0.13;
        let yaw = rnd() * PI * 2.0;
        let tilt = (rnd() - 0.5) * 0.4;
        let t = tilt / 0.4 + 0.5;
        let acorn = group();
        let r = if t < 0.40 {
            // bare: fallen from its cup; one in three holed
            let holed = t < 0.40 / 3.0;
            if let Some(geo) = acorn_nut_geo() {
                acorn.add(&mesh(geo, nut_mat(k, holed)));
            }
            0.013
        } else if t < 0.80 {
            let nut = nut_mat(k, false);
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

/// Rotate (x, y) by `a` about the origin
fn rot2(x: f64, y: f64, a: f64) -> (f64, f64) {
    let (c, s) = (a.cos(), a.sin());
    (x * c - y * s, x * s + y * c)
}

// ================================================================= acorns
/// The acorn (`nature/a_garden_and_a_wood_indoors.py`): the nut's length and widest radius, its base over
/// the cup's foot, the cup's rim; its profile (a fraction of NUT_R along it)
const NUT_L: f64 = 0.025;

const NUT_R: f64 = 0.0076;

const CUP_FLOOR: f64 = 0.0022;

const CUP_RIM: f64 = 0.0084;

/// The cupped acorn laid down: its turn about z and where the cup's foot
/// went (`nature/a_garden_and_a_wood_indoors.py` acorn_pair, printed by the build)
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
    cached_kind_tex!(6, k * 2 + holed as usize, |kk: usize| {
        let (k, holed) = (kk / 2, kk % 2 == 1);
        let base = [[0.50, 0.32, 0.16], [0.33, 0.21, 0.12], [0.50, 0.40, 0.28]][k];
        let seed = 0xac0 + kk as u32 * 13;
        canvas_tex_opt(128, 64, false, move |ctx, w, h| {
            paint_px(ctx, w, h, |px, py| {
                let (u, v) = (px / w, 1.0 - py / h);
                let mut c = base;
                // fine stripes along it (lines of one angle), a little wavy
                let st = fbm(seed, u * 6.0, v * 34.0 + 2.0 * fbm(seed ^ 3, u * 3.0, v * 3.0, 2), 3);
                c = mul(c, 0.80 + 0.40 * st);
                // its shoulder toward the tip a shade darker, the tip's felt
                // greyer and paler round the style
                c = mix_unclamped(c, mul(c, 0.8), sstep(0.70, 0.85, u));
                c = mix_unclamped(c, [0.55, 0.50, 0.42], 0.75 * sstep(0.90, 0.97, u));
                c = mix_unclamped(c, mul(c, 0.55), 0.6 * (1.0 - ((u - 0.88) / 0.02).abs()).max(0.0));
                // the scar where it sat in the cup: pale, rough, a dark ring
                let scar = 1.0 - sstep(NUT_U_SCAR - 0.02, NUT_U_SCAR + 0.01, u);
                let rough = fbm(seed ^ 9, u * 40.0, v * 40.0, 3);
                c = mix_unclamped(c, mix_unclamped([0.62, 0.55, 0.42], [0.48, 0.41, 0.30], rough), scar);
                let ring = (1.0 - ((u - NUT_U_SCAR - 0.012) / 0.012).abs()).max(0.0);
                c = mix_unclamped(c, mul(base, 0.45), 0.7 * ring);
                // the part that was in the cup: paler, never browned in the sun
                c = mix_unclamped(
                    c,
                    mix_unclamped(c, [0.62, 0.55, 0.38], 0.35),
                    sstep(NUT_U_SCAR + 0.25, NUT_U_SCAR + 0.05, u) * (1.0 - scar),
                );
                // decay: dark blotches, a dull bloom of dust on the top
                let rot = sstep(0.68, 0.78, fbm(seed ^ 0x51, u * 9.0, v * 5.0, 4));
                c = mix_unclamped(c, mul(base, 0.5), 0.6 * rot);
                c = mix_unclamped(c, [0.56, 0.52, 0.46], 0.18 * (1.0 - sstep(0.0, 0.5, v)));
                if holed {
                    // the weevil larva's exit hole: round, 2 mm, on the top
                    let d = ((u - 0.52) / 0.033).hypot(v / 0.042);
                    c = mix_unclamped(c, [0.36, 0.26, 0.17], 0.8 * (1.0 - sstep(1.0, 1.5, d)));
                    c = mix_unclamped(c, [0.05, 0.035, 0.03], 1.0 - sstep(0.75, 0.95, d));
                }
                [c[0], c[1], c[2], 1.0]
            });
        })
    })
}

/// A nut's material: a ripe one keeps a little of its gloss
fn nut_mat(k: usize, holed: bool) -> Mat {
    if k == 0 {
        phong!({ map: &nut_tex(k, holed), specular: 0x1c1813, shininess: 28 })
    } else {
        lambert!({ map: &nut_tex(k, holed) })
    }
}

/// A cup (u round it, v its outside then its inside, foot to rim): felted
/// grey-brown scales in rows, their tips paler and their edges dark, as
/// the model's (13 round, 6 rows); the rim's lip; the inside pale and
/// silky, a ring where the nut's base sat; the stalk's stub woody
fn cup_tex() -> Tex {
    cached_kind_tex!(1, 0usize, |_k: usize| {
        let seed = 0xc0b;
        canvas_tex_opt(256, 128, true, move |ctx, w, h| {
            paint_px(ctx, w, h, |px, py| {
                let (u, v) = (px / w, 1.0 - py / h);
                let felt = fbm(seed, u * 60.0, v * 60.0, 3);
                if v >= 0.5 {
                    let hh = (v - 0.5) * 2.0;
                    let mut c = mix_unclamped([0.66, 0.58, 0.44], [0.58, 0.50, 0.37], felt);
                    c = mix_unclamped(c, [0.40, 0.31, 0.21], 0.5 * (1.0 - ((hh - 0.28) / 0.06).abs()).max(0.0));
                    c = mix_unclamped(c, [0.46, 0.40, 0.31], 0.5 * sstep(0.9, 1.0, hh));
                    return [c[0], c[1], c[2], 1.0];
                }
                let hh = v * 2.0;
                let rows = 6.0;
                let kk = hh * rows;
                let row = kk.floor();
                let fv = kk - row;
                let col = (u * 13.0 + 0.5 * (row as i32 % 2) as f64).floor();
                // each scale a little out of line with its neighbours
                let jit = hash(seed ^ 0x5c, col as i32, row as i32) - 0.5;
                let fu = (u * 13.0 + 0.5 * (row as i32 % 2) as f64 + 0.25 * jit * fv).fract();
                let fv = (fv + 0.2 * (hash(seed ^ 0x5d, col as i32, row as i32) - 0.5) * (1.0 - fv)).clamp(0.0, 1.0);
                let tone = 0.9 + 0.2 * hash(seed ^ 0x5e, col as i32, row as i32);
                let mut c = mul(mix_unclamped([0.50, 0.44, 0.34], [0.42, 0.36, 0.27], felt), tone);
                // each scale paler and felted toward its tip, its lower edge
                // shadowed by the one over it
                c = mix_unclamped(c, [0.60, 0.55, 0.45], 0.45 * fv.powf(1.5));
                let edge = (1.0 - sstep(0.0, 0.12, fu.min(1.0 - fu))) * 0.8 + (1.0 - sstep(0.0, 0.22, fv));
                c = mix_unclamped(c, [0.22, 0.17, 0.13], 0.75 * edge.min(1.0));
                // the lip of the rim, the stub at the foot
                c = mix_unclamped(c, [0.52, 0.46, 0.37], sstep(0.92, 0.98, hh));
                c = mix_unclamped(c, [0.30, 0.22, 0.14], 1.0 - sstep(0.05, 0.12, hh));
                [c[0], c[1], c[2], 1.0]
            });
        })
    })
}
