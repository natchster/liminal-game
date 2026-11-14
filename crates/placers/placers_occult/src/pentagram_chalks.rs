//! `place_pentagram_chalks`
use game::prelude::*;
use placers_kit::noise::Rng32;
use placers_kit::noise::fbm;
use placers_kit::noise::hash;
use placers_kit::noise::sstep;
use placers_kit::noise::vn;
use placers_kit::paint::fill;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;
use placers_kit::place::salt;

// Pentagram chalk decal: 3% chance per level. A circle drawn twice and
// a five-pointed star inside it, 1.3 m across, chalked on the floor by hand
// with a stick of blackboard chalk: each line a stroke of grains caught on
// the floor's tooth, heavier where pressed, its ends overshooting where the
// hand went on; scuffed through where feet crossed it, its dust about it.
// No light: chalk does not shine (the old glow is gone, its pulse's draw
// kept).
pub(crate) fn place_pentagram_chalks(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    } // 3% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = pick(&open_tiles);
    let px = g.wx(x) + (rnd() - 0.5) * 1.5;
    let pz = g.wz(z) + (rnd() - 0.5) * 1.5;
    let spin = rnd() * PI * 2.0;
    // the old glow's pulse phase (no glow: chalk does not shine)
    let _ = rnd();
    let tex = pentagram_tex(salt(px, pz, 0x5e7));
    let mat = lambert!({
        map: &tex,
        color: pale(chalk_color_by_weather(g.weather_key())),
        transparent: true,
        depthWrite: false,
        polygonOffset: true,
        polygonOffsetFactor: -2,
        polygonOffsetUnits: -2
    });
    let decal = mesh(plane_geo!(PENT_SIZE, PENT_SIZE), &mat);
    decal.rotation().x = -PI / 2.0;
    decal.position().set(px, 0.005, pz);
    decal.rotation().z = spin;
    g.world.add(&decal);
}

/// The place's chalk colour (r08's, by the weather) as a stick of it
/// leaves it on a floor: two thirds chalk white (sRGB 235, linear 0.83)
fn pale(c: u32) -> u32 {
    let ch = |s: u32| (235.0 - (235.0 - f64::from((c >> s) & 0xff).min(235.0)) * 0.35).round() as u32;
    (ch(16) << 16) | (ch(8) << 8) | ch(0)
}

/// The chalked square (m) and its texture's side (px: 1.37 mm a pixel)
const PENT_SIZE: f64 = 1.4;

const PENT_PX: f64 = 1024.0;

/// One stroke of chalk (mm, on the decal): an arc of the circle drawn
/// freehand (its centre the decal's), or a line of the star, bowed; how wide
/// the stick drew and where along it the hand pressed
enum Stroke {
    Arc { a0: f64, span: f64, rad: f64, p1: f64, p2: f64, width: f64, wob: f64 },
    Line { a: (f64, f64), d: (f64, f64), l: f64, t0: f64, t1: f64, bow: f64, width: f64, wob: f64 },
}

impl Stroke {
    fn width(&self) -> f64 {
        match self {
            Stroke::Arc { width, .. } | Stroke::Line { width, .. } => *width,
        }
    }
    fn wob(&self) -> f64 {
        match self {
            Stroke::Arc { wob, .. } | Stroke::Line { wob, .. } => *wob,
        }
    }
    /// The arc's radius at angle q
    fn radius(rad: f64, p1: f64, p2: f64, q: f64) -> f64 {
        rad + 9.0 * (2.0 * q + p1).dsin() + 5.0 * (5.0 * q + p2).dsin()
    }
    /// The distance from (x, y) to the stroke (mm) and how far along it it
    /// is (0..1), given its polar coordinates about the decal's middle
    /// (`rho`, `q`); None when far from it
    fn at(&self, x: f64, y: f64, rho: f64, q: f64) -> Option<(f64, f64)> {
        match *self {
            Stroke::Arc { a0, span, rad, p1, p2, .. } => {
                if (rho - rad).abs() > 60.0 {
                    return None;
                }
                let rel = (q - a0).rem_euclid(PI * 2.0);
                if rel <= span {
                    let r = Stroke::radius(rad, p1, p2, a0 + rel);
                    Some(((rho - r).abs(), rel / span))
                } else {
                    // past an end: the distance to the nearer end
                    let (e0, e1) = (a0, a0 + span);
                    let (r0, r1) = (Stroke::radius(rad, p1, p2, e0), Stroke::radius(rad, p1, p2, e1));
                    let d0 = ((x - r0 * e0.dcos()).dpowi(2) + (y - r0 * e0.dsin()).dpowi(2)).sqrt();
                    let d1 = ((x - r1 * e1.dcos()).dpowi(2) + (y - r1 * e1.dsin()).dpowi(2)).sqrt();
                    Some(if d0 < d1 { (d0, 0.0) } else { (d1, 1.0) })
                }
            }
            Stroke::Line { a, d, l, t0, t1, bow, .. } => {
                let (rx, ry) = (x - a.0, y - a.1);
                let t = (rx * d.0 + ry * d.1) / l;
                let perp = -rx * d.1 + ry * d.0;
                let tc = t.clamp(t0, t1);
                let off = bow * (PI * tc.clamp(0.0, 1.0)).dsin();
                let along = (tc - t0) / (t1 - t0);
                let dist = (((t - tc) * l).dpowi(2) + (perp - off).dpowi(2)).sqrt();
                if dist > 60.0 {
                    return None;
                }
                Some((dist, along))
            }
        }
    }
}

/// The pentagram in chalk (white, its grains in alpha; the material's
/// colour tints it as the place's chalk is tinted)
fn pentagram_tex(salt: u32) -> Tex {
    canvas_tex_opt(PENT_PX, PENT_PX, false, move |ctx, w, h| {
        let seed = salt ^ 0x9e57;
        let mut r = Rng32(seed | 1);
        let mut strokes = vec![];
        // the circle, drawn twice round (an outer ring and an inner), each in
        // two strokes whose ends overlap where the hand went on
        for rad in [655.0, 585.0] {
            let (p1, p2) = (r.f() * 6.3, r.f() * 6.3);
            let mut a = r.f() * 6.3;
            for part in 0..2 {
                let span = PI + if part == 0 { 0.1 + 0.1 * r.f() } else { 0.05 + 0.12 * r.f() };
                let off = (r.f() - 0.5) * 6.0;
                strokes.push(Stroke::Arc { a0: a, span, rad: rad + off, p1, p2, width: 15.0 + 5.0 * r.f(), wob: r.f() * 6.0 });
                a += span - 0.08;
            }
        }
        // the star: its five points on the inner ring, each line drawn from a
        // point to the next but one, bowed a little, overshooting its ends
        let top = PI / 2.0 + (r.f() - 0.5) * 0.08;
        let pts5: Vec<(f64, f64)> = (0..5)
            .map(|k| {
                let q = top + f64::from(k) * 2.0 * PI / 5.0;
                let rr = 572.0 + (r.f() - 0.5) * 16.0;
                (rr * q.dcos(), rr * q.dsin())
            })
            .collect();
        for k in 0..5 {
            let (a, b) = (pts5[k * 2 % 5], pts5[(k * 2 + 2) % 5]);
            let (dx, dy) = (b.0 - a.0, b.1 - a.1);
            let l = (dx * dx + dy * dy).sqrt();
            let t0 = -(2.0 + 20.0 * r.f()) / l;
            let t1 = 1.0 + (6.0 + 30.0 * r.f()) / l;
            let bow = (r.f() - 0.5) * 22.0;
            strokes.push(Stroke::Line { a, d: (dx / l, dy / l), l, t0, t1, bow, width: 15.0 + 5.0 * r.f(), wob: r.f() * 6.0 });
        }
        // where feet crossed it: scuffs through the lines, the chalk smeared
        let scuffs: Vec<(f64, f64, f64, f64)> =
            (0..4).map(|_| ((r.f() - 0.5) * 1100.0, (r.f() - 0.5) * 1100.0, 40.0 + 70.0 * r.f(), r.f() * PI)).collect();
        let mm_px = PENT_SIZE * 1000.0 / w;
        fill(ctx, w, h, move |u, v| {
            let (x, y) = ((u - 0.5) * PENT_SIZE * 1000.0, (v - 0.5) * PENT_SIZE * 1000.0);
            let (rho, q) = ((x * x + y * y).sqrt(), y.datan2(x));
            let mut a = 0.0f64;
            let mut haze = 0.0f64;
            for st in &strokes {
                if let Some((d, along)) = st.at(x, y, rho, q) {
                    let ends = (along / 0.04).min(1.0) * ((1.0 - along) / 0.06).min(1.0);
                    let wob = st.wob();
                    // pressed harder and lighter along it, and broken where it
                    // skipped over the pile
                    let press = (0.35 + 0.65 * vn(seed ^ 0x61, along * 18.0 + wob * 7.0, 0.5)) * (0.45 + 0.55 * ends);
                    let press = press * sstep(0.1, 0.24, vn(seed ^ 0x62, along * 30.0 + wob * 3.0, 1.5));
                    // its edge ragged where the stick rolled and the floor's pile caught it
                    let half = st.width() * 0.5 * (0.75 + 0.25 * ends) * (0.8 + 0.4 * vn(seed ^ 0x21, x / 5.0, y / 5.0));
                    if d < half + mm_px {
                        let edge = 1.0 - (d / half).min(1.0);
                        // the floor's tooth: grains caught on the pile's tips (1.4
                        // mm), in clumps (3 mm), skipping where the stick skipped
                        let tooth =
                            hash(seed, (x / 1.4).floor() as i32, (y / 1.4).floor() as i32) * 0.5 + vn(seed ^ 0x33, x / 3.0, y / 3.0) * 0.5;
                        let skip = 0.75 + 0.25 * vn(seed ^ 0x44, along * 140.0 + wob * 10.0, 0.5);
                        let k = press * skip * (0.9 + 0.5 * edge) - tooth * 0.85;
                        let grain = sstep(-0.05, 0.3, k) * sstep(half + mm_px, half - mm_px, d);
                        a = a.max(grain * (0.45 + 0.55 * press) * (0.7 + 0.3 * edge));
                    }
                    haze = haze.max(0.13 * (-d / 24.0).dexp() * press * (0.6 + 0.4 * vn(seed ^ 0x12, x / 20.0, y / 20.0)));
                }
            }
            if a <= 0.0 && haze <= 0.004 {
                return [1.0, 1.0, 1.0, 0.0];
            }
            // scuffed: the chalk worn off in patches and smeared about them
            let mut keep = 1.0;
            for &(sx, sy, sr, sa) in &scuffs {
                let (dx, dy) = (x - sx, y - sy);
                let (ex, ey) = (dx * sa.dcos() + dy * sa.dsin(), (-dx * sa.dsin() + dy * sa.dcos()) * 2.2);
                let q = (ex * ex + ey * ey).sqrt() / sr;
                if q < 1.4 {
                    let n = fbm(seed ^ 0x51, x * 0.02, y * 0.02, 3);
                    let gone = (1.0 - sstep(0.55, 1.25, q + (n - 0.5) * 0.6)).clamp(0.0, 1.0);
                    keep = f64::min(keep, 1.0 - 0.85 * gone);
                    haze = haze.max(0.07 * gone * a.max(0.3));
                }
            }
            let lift = 0.78 + 0.22 * fbm(seed ^ 0x77, x * 0.004, y * 0.004, 2);
            let alpha = (a * keep * lift).max(haze);
            [1.0, 1.0, 1.0, alpha.clamp(0.0, 1.0)]
        });
    })
}
