//! `place_snail_shells`
use crate::shared::blob;
use crate::shared::css;
use game::prelude::*;
use placers_kit::geometry::frame;
use placers_kit::geometry::part;
use placers_kit::noise::fbm;
use placers_kit::noise::hash;
use placers_kit::noise::mix_unclamped;
use placers_kit::noise::sstep;
use placers_kit::noise::vn;
use placers_kit::paint::paint_px;
use placers_kit::place::jitter;
use placers_kit::place::pick_open;

/// One of `N` textures of a kind, made the first time it is asked for
macro_rules! cached_kind_tex {
    ($n:expr_2021, $k:expr_2021, $make:expr_2021) => {{
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

// Snail shell: 5% chance per level. An empty snail's shell (Blender
// family `bits_of_the_outside_indoors`): a garden snail's, 4.5 whorls, its bands following them, its
// white lip turned back, or a grove snail's; lying on its aperture or on its
// side.
pub(crate) fn place_snail_shells(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let shell_colors = [0xc8a888, 0xa08060, 0xd0b090, 0x604020];
    // the old colour: which shell
    let col = choice(&shell_colors);
    let k = shell_colors.iter().position(|&c| c == col).unwrap_or(0);
    let (_, _, _, _, lip, inside) = SHELLS[k];
    let posed = group();
    // the shell's wall seen from within too, deep in the aperture (its bands
    // show through it, as a real one's do)
    if let Some(geo) = part("Real__snailShell") {
        posed.add(&mesh(geo, phong!({ map: &shell_tex(k), specular: 0x1e1c18, shininess: 22, side: DoubleSide })));
    }
    if let Some(geo) = part("Real__snailLip") {
        posed.add(&mesh(geo, phong!({ color: lip, specular: 0x24221e, shininess: 30 })));
    }
    if let Some(geo) = part("Real__snailInside") {
        posed.add(&mesh(geo, lambert!({ color: inside, side: DoubleSide })));
    }
    // a grove snail's is smaller (22 mm across to the garden snail's 32)
    let s = if k == 2 { 0.7 } else { 1.0 };
    posed.scale().set_scalar(s);
    let shell = group();
    let (px, pz) = jitter(g, x, z, 2.5);
    shell.position().set(px, 0, pz);
    shell.rotation().y = rnd() * PI * 2.0;
    // the old "on its side" (40%): on its side, else on its aperture
    let side = rnd() < 0.4;
    if side {
        posed.rotation().z = PI * 0.5;
    } else {
        // the aperture's face (the sweep's way at its end) turned to the floor
        let tt = sh_t_end();
        let (a, b) = (sh_point(tt - 0.05, -PI / 2.0), sh_point(tt + sh_obl(-PI / 2.0), -PI / 2.0));
        let mut d = Vec3::new(b[0] - a[0], (b[1] - a[1]) * 0.6, b[2] - a[2]);
        d.normalize();
        let mut down = Vec3::new(0.15, -1.0, 0.0);
        down.normalize();
        let mut q = Quat::IDENTITY;
        q.set_from_unit_vectors(&d, &down);
        posed.quaternion().copy(&q);
    }
    // its lowest point on the floor, its middle where the old shell's was
    let mut bb = Box3::default();
    bb.set_from_object(&posed);
    posed.position().set(-(bb.min.x + bb.max.x) / 2.0, -bb.min.y, -(bb.min.z + bb.max.z) / 2.0);
    shell.add(&posed);
    g.world.add(&shell);
}

// ============================================================ snail shell
/// The garden snail's shell as `nature/bits_of_the_outside_indoors.py` sweeps it (SH_*): its growth a
/// whorl, whorls, the section's centre radius, radius, drop and height over
/// width at the aperture, how oblique the aperture is
const SH_W: f64 = 2.3;

const SH_N: f64 = 4.3;

const SH_RC: f64 = 0.0085;

const SH_RHO: f64 = 0.0100;

const SH_H: f64 = 0.0175;

const SH_B: f64 = 1.12;

const SH_OBL: f64 = 0.25;

fn sh_t_end() -> f64 {
    SH_N * PI * 2.0
}

/// The shell's surface at turn t and section angle phi (`nature/bits_of_the_outside_indoors.py` sh_point)
fn sh_point(t: f64, phi: f64) -> [f64; 3] {
    let tt = sh_t_end();
    let s = (SH_W.ln() / (PI * 2.0) * (t - tt)).exp();
    let drop = SH_H * s * (1.0 + 0.10 * sstep(tt - 1.4, tt, t));
    let rho = SH_RHO * s * (1.0 + 0.07 * sstep(tt - 0.3, tt, t));
    let (c, sn) = (t.cos(), t.sin());
    [SH_RC * s * c + c * rho * phi.cos(), -drop + rho * SH_B * phi.sin(), -SH_RC * s * sn - sn * rho * phi.cos()]
}

/// How much further round the last whorl goes at section angle phi
fn sh_obl(phi: f64) -> f64 {
    SH_OBL * (0.5 - 0.5 * phi.sin())
}

/// The four shells the old colour chose: (ground, bands, how strong the
/// bands are, how broken into flames, the lip, the inside; sRGB)
const SHELLS: [([f64; 3], [f64; 3], f64, f64, u32, u32); 4] = [
    // a garden snail's, old and bleached, its periostracum worn to the chalk
    ([0.72, 0.65, 0.52], [0.42, 0.32, 0.22], 0.45, 0.55, 0xd4cfc2, 0x8c8272),
    // a garden snail's, its bands dark, broken by yellow flecks
    ([0.62, 0.50, 0.33], [0.27, 0.18, 0.11], 0.9, 0.45, 0xd8d2c4, 0x6e604c),
    // a grove snail's (Cepaea nemoralis), yellow with five dark bands, its lip dark
    ([0.76, 0.66, 0.36], [0.24, 0.16, 0.10], 0.92, 0.0, 0x3e2c22, 0x8a7a50),
    // a garden snail's, dark with damp
    ([0.46, 0.37, 0.25], [0.21, 0.15, 0.10], 0.85, 0.4, 0xbab4a6, 0x5a4e3e),
];

/// Shell `k` in its part's sphere frame (u round the axis, v down from the
/// apex): the ground mottled, the spiral bands following the whorls (painted
/// along the sweep), growth lines across them, the apex worn pale
fn shell_tex(k: usize) -> Tex {
    cached_kind_tex!(4, k, |k: usize| {
        let (lo, hi) = frame("Real__snailShell");
        let (ground, band, strength, flames, _, _) = SHELLS[k];
        let cepaea = k == 2;
        canvas_tex_opt(512, 256, false, move |ctx, w, h| {
            // where a point of the shell lands in the texture (`uv_for`'s sphere)
            let uv = |p: [f64; 3]| {
                let c: Vec<f64> = (0..3).map(|i| (p[i] - lo[i]) / (hi[i] - lo[i]) * 2.0 - 1.0).collect();
                let u = (c[2].atan2(-c[0]) / (PI * 2.0) + 1.0) % 1.0;
                let r = (c[0] * c[0] + c[1] * c[1] + c[2] * c[2]).sqrt().max(1e-9);
                let v = 1.0 - (c[1] / r).clamp(-1.0, 1.0).acos() / PI;
                (u * w, (1.0 - v) * h)
            };
            paint_px(ctx, w, h, |x, y| {
                let m = fbm(0x5e11 + k as u32, x / 18.0, y / 10.0, 4);
                let streak = vn(0x5e12 + k as u32, x / 3.0, y / 40.0);
                let c = mix_unclamped(ground, mix_unclamped(ground, [0.30, 0.25, 0.18], 0.35), sstep(0.4, 0.75, m) * 0.7 + 0.15 * streak);
                [c[0], c[1], c[2], 1.0]
            });
            // a quad of the surface between turns t0..t1 and angles p0..p1
            let quad = |ctx: &mut Canvas, t0: f64, t1: f64, p0: f64, p1: f64, c: [f64; 3], a: f64| {
                let pts = [uv(sh_point(t0, p0)), uv(sh_point(t1, p0)), uv(sh_point(t1, p1)), uv(sh_point(t0, p1))];
                let minx = pts.iter().fold(f64::MAX, |m, p| m.min(p.0));
                let maxx = pts.iter().fold(f64::MIN, |m, p| m.max(p.0));
                // across the seam: drawn on both sides of it
                let wrap = maxx - minx > w * 0.5;
                for shift in [0.0, if wrap { -w } else { f64::NAN }] {
                    if shift.is_nan() {
                        continue;
                    }
                    ctx.set_fill_style(css(c, a));
                    ctx.begin_path();
                    for (i, &(px, py)) in pts.iter().enumerate() {
                        let px = if wrap && px < w * 0.5 { px + w } else { px } + shift;
                        if i == 0 {
                            ctx.move_to(px, py);
                        } else {
                            ctx.line_to(px, py);
                        }
                    }
                    ctx.close_path();
                    ctx.fill();
                }
            };
            let tt = sh_t_end();
            // the bands: (section angle, half width), on the last whorls
            let bands: &[(f64, f64)] = if cepaea {
                &[(0.95, 0.07), (0.45, 0.06), (0.05, 0.09), (-0.45, 0.10), (-0.95, 0.12)]
            } else {
                &[(1.05, 0.10), (0.55, 0.14), (0.15, 0.16), (-0.35, 0.18), (-0.85, 0.14)]
            };
            for (bi, &(pc, hw)) in bands.iter().enumerate() {
                let mut t = tt - 2.8 * PI;
                while t < tt + sh_obl(pc) {
                    let dt = 0.025;
                    // the flames: the band broken by flecks of the ground,
                    // fading in and out along it
                    let f = vn(0xf1a + bi as u32 * 7 + k as u32, t * 5.0, bi as f64 * 3.0);
                    let a = strength * (1.0 - flames * sstep(0.45, 0.7, f));
                    let t1 = (t + dt * 1.3).min(tt + sh_obl(pc));
                    // soft-edged: the band's middle, then its two edges fainter
                    quad(ctx, t, t1, pc - hw * 0.6, pc + hw * 0.6, band, a * 0.8);
                    quad(ctx, t, t1, pc - hw, pc + hw, band, a * 0.45);
                    quad(ctx, t, t1, pc - hw * 1.35, pc + hw * 1.35, band, a * 0.15);
                    t += dt;
                }
            }
            // growth lines across the whorls
            let mut t = tt - 3.0 * PI;
            let mut i = 0u32;
            while t < tt {
                let a = 0.06 + 0.14 * hash(0x9a0 + k as u32, i as i32, 0);
                let c = if i.is_multiple_of(3) { [0.86, 0.80, 0.68] } else { [0.30, 0.24, 0.17] };
                for j in 0..14 {
                    let (p0, p1) = (-PI + PI * 2.0 * j as f64 / 14.0, -PI + PI * 2.0 * (j + 1) as f64 / 14.0);
                    quad(ctx, t, t + 0.012, p0, p1, c, a);
                }
                t += 0.04 + 0.08 * hash(0x9a1 + k as u32, i as i32, 1);
                i += 1;
            }
            // the apex worn pale, and dust in the sutures' shadow at the top
            blob(ctx, w * 0.5, 0.0, h * 0.22, w / h * 0.25, 0.0, [0.78, 0.74, 0.66], 0.6);
            ctx.set_fill_style(css([0.78, 0.74, 0.66], 0.35));
            ctx.fill_rect(0, 0, w, h * 0.06);
        })
    })
}
