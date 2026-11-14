//! `place_ferns`
use game::prelude::*;
use placers_kit::geometry::frame;
use placers_kit::geometry::part;
use placers_kit::noise::fbm;
use placers_kit::noise::mix_unclamped;
use placers_kit::noise::sstep;
use placers_kit::paint::bleed;
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

// Fern frond: 6% chance per level. A frond of common polypody broken
// off and dropped (Blender family `a_garden_and_a_wood_indoors`): its blade cut nearly to the rachis
// into narrow blunt segments, lying on its face or its back (the sori in
// rows), green browning from its tips, withered or dead.
pub(crate) fn place_ferns(g: &mut Game, _dist: &[i32]) {
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
    // A draught stirs it (the old sway's rate and its first leaflet's
    // phase): it turns a hair on the floor about its stalk's end
    let phase = ph[2] * PI * 2.0;
    g.animate(move |_g, _dt, t| {
        body.rotation().y = (t * 0.4 + phase).sin() * 0.012;
    });
}

// =================================================================== fern
/// The frond (`nature/a_garden_and_a_wood_indoors.py` FERN_*): the blade's length, its segment pairs, a
/// segment's angle forward from the rachis
const FERN_L: f64 = 0.150;

const FERN_N: usize = 13;

const FERN_ANG: f64 = 64.0 * PI / 180.0;

/// Segment k (from the base) on side ±1: its root's x on the rachis, its
/// length, its half width (as `nature/a_garden_and_a_wood_indoors.py` fern_seg)
fn fern_seg(k: usize, side: f64) -> (f64, f64, f64) {
    let t = (k as f64 + if side > 0.0 { 0.0 } else { 0.45 }) / FERN_N as f64;
    let x = FERN_L * (0.02 + 0.86 * t);
    let k = if t < 0.5 { 0.90 + 0.10 * (t * 4.0).min(1.0) } else { ((t - 0.5) / 0.5 * PI / 2.0).cos().max(0.0).powf(0.8) };
    let l = 0.032 * k + 0.004;
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
    cached_kind_tex!(6, state * 2 + back as usize, |kk: usize| {
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
                let mut c = mix_unclamped(green, mul(green, 0.8), n);
                // drying from the tips and the margins inward
                let (a, across) = seg.map(|(a, c, _)| (a, c)).unwrap_or((0.0, 0.0));
                let dry = match state {
                    0 => sstep(0.72, 0.98, a) * 0.8,
                    1 => 0.35 + 0.5 * sstep(0.5, 0.95, a),
                    _ => 0.9,
                };
                let blot = fbm(seed ^ 0x33, x * 120.0, z * 120.0, 3);
                c = mix_unclamped(c, brown, (dry + 0.3 * (blot - 0.5)).clamp(0.0, 1.0));
                c = mix_unclamped(c, mul(brown, 0.7), 0.6 * sstep(0.55, 0.95, across.abs()) * (0.4 + 0.6 * dry));
                // the midvein, the side veins forking out to the margin
                if let Some((a, cr, k)) = seg {
                    let mid = 1.0 - sstep(0.06, 0.14, cr.abs());
                    let q = (a * 18.0 - cr.abs() * 1.4 + k as f64 * 0.37).fract();
                    let side = (1.0 - sstep(0.04, 0.12, (q - 0.5).abs())) * sstep(0.12, 0.25, cr.abs()) * (1.0 - sstep(0.7, 0.9, cr.abs()));
                    let pale = if back { [0.62, 0.62, 0.48] } else { [0.52, 0.54, 0.36] };
                    c = mix_unclamped(c, mix_unclamped(c, pale, 0.6), mid.max(side * 0.35));
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
                            c = mix_unclamped(c, sori, 1.0 - sstep(0.7, 1.0, d));
                        }
                    }
                } else {
                    // the rachis's wing
                    c = mix_unclamped(c, mul(c, 1.1), 0.3);
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
