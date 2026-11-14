//! `place_water_puddles`
use crate::shared::mark_tex;
use crate::shared::vary_by;
use game::prelude::*;
use placers_kit::noise::fbm;
use placers_kit::noise::smin;
use placers_kit::noise::sstep;
use placers_kit::noise::tide;
use placers_kit::noise::vn as vnoise;
use placers_kit::place::open_tiles;
use placers_kit::place::scatter_on_floor;
use placers_kit::wet_floor::floor_kind as puddle_floor_kind;
use placers_kit::wet_floor::Px;

// Water puddle: 8% chance per level. Where a drip from the ceiling
// lands, and has gone on landing (`docs/research/props-remodelling/floor_marks_and_dust.md`):
// on carpet the water has soaked in, a dark patch with a soft ragged edge and
// a darker rim where the dirt wicked out to it, and where the drops land the
// pile is so wet that water stands on it, with a sheen; on a hard floor it
// stands, a thin film darker than the floor round it, its edge sharp and
// lobed, the dried tide lines of where it reached before outside it, and a
// sheen. Each drop sends a ring out across the standing water, gone within a
// second. Painted on transparent planes flat on the floor, as wide as the old
// disc.
pub(crate) fn place_water_puddles(g: &mut Game, _dist: &[i32]) {
    g.pp.puddle_positions.clear(); // reset per level
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let puddle = group();
    // the old disc's radius: the plane is as wide as the disc was, the water
    // inside it
    let p_radius = 0.10 + rnd() * 0.10;
    let floor = puddle_floor_kind(g, x, z);
    let hard = floor != 0;
    let vary = vary_by(x, z, (p_radius * 1.0e4) as i32);
    let mut layers = vec![water_puddle_mat(p_radius, hard, false, vary)];
    if !hard {
        // the water standing on the soaked pile where the drops land
        layers.push(water_puddle_mat(p_radius, false, true, vary));
    }
    for (k, mat) in layers.iter().enumerate() {
        let geo = plane_geo!(p_radius * 2.0, p_radius * 2.0);
        geo.rotate_x(-PI / 2.0);
        let water = mesh(geo, mat);
        water.position().y = 0.003 + k as f64 * 0.0005;
        puddle.add(&water);
    }
    // the ring a drop sends out (the old torus): a soft bright crest and its
    // trough, on a plane over the water
    let ring_mat = drip_ring_mat();
    let rgeo = plane_geo!(p_radius * 0.6, p_radius * 0.6);
    rgeo.rotate_x(-PI / 2.0);
    let ripple = mesh(rgeo, &ring_mat);
    ripple.position().y = 0.0042;
    ripple.set_visible(false);
    puddle.add(&ripple);
    // the old ripple's phase and speed: when each drop lands (every 5-8 s)
    let ripple_phase = rnd() * PI * 2.0;
    let ripple_speed = 0.5 + rnd() * 0.3;
    {
        let ripple = ripple.clone();
        // the ring's radius on the plane at a scale of one (its crest at 0.4
        // of the plane's width), and how far the standing water reaches
        let r1 = p_radius * 0.6 * DRIP_RING_AT;
        let reach = if hard { p_radius * 0.62 } else { p_radius * 0.26 };
        g.animate(move |_g, _dt, t| {
            // seconds since the last drop landed; the ring spreads at about
            // the speed of the slowest ripples on water (0.23 m/s) and dies
            // away as it goes
            let since = ((t * ripple_speed + ripple_phase) % 4.0) / ripple_speed;
            let r = 0.008 + since * 0.23;
            let life = (1.0 - r / reach).min(1.0 - since / 1.1);
            if life <= 0.0 {
                ripple.set_visible(false);
                return;
            }
            ripple.set_visible(true);
            ripple.scale().set_scalar(r / r1);
            ripple.mat().set_opacity(life.min(1.0).powf(1.5));
        });
    }
    scatter_on_floor(g, &puddle, x, z, 2.4);
    g.world.add(&puddle);
    // Track for drip audio.
    let p = puddle.pos();
    g.pp.puddle_positions.push(v3(p.x, 0.003, p.z));
}

/// Where the drip ring's crest lies on its plane (a fraction of its width)
const DRIP_RING_AT: f64 = 0.4;

/// Water a drip has left on the floor, on a plane `2r` square: on a hard
/// floor a film with a sharp lobed edge, a damp margin, the dried rims of
/// where it reached before and a pool or two beside it; on carpet the patch
/// it soaked (or, `standing`, the water standing on the pile in the middle
/// of it). Its material, with the sheen of wet things.
fn water_puddle_mat(r: f64, hard: bool, standing: bool, vary: u32) -> Mat {
    let pw = r * 2.0;
    let wet = if hard {
        [0.1, 0.104, 0.11]
    } else if standing {
        [0.07, 0.062, 0.055]
    } else {
        [0.12, 0.1, 0.08]
    };
    let tex = mark_tex(192.0, 192.0, wet, vary, move |ctx, cw, ch| {
        let seed = (rnd() * 4.0e9) as u32;
        let s = pw / cw;
        let aa = s * 1.2;
        // how far the water reaches now, and before (the dried rims)
        let reach = if hard {
            r * (0.6 + 0.08 * rnd())
        } else if standing {
            r * (0.3 + 0.08 * rnd())
        } else {
            r * (0.74 + 0.08 * rnd())
        };
        let rims: Vec<(f64, u32, f64)> = if hard {
            (0..1 + (rnd() * 2.2) as usize)
                .map(|k| (reach * (1.12 + 0.12 * k as f64 + 0.06 * rnd()), seed ^ (0x51 + k as u32), 0.1 + 0.12 * rnd()))
                .collect()
        } else {
            vec![]
        };
        let sats: Vec<(f64, f64, f64)> = if hard {
            (0..(rnd() * 3.0) as usize)
                .map(|_| {
                    let (a, rr) = (rnd() * PI * 2.0, reach * (0.95 + 0.25 * rnd()));
                    (a.cos() * rr, a.sin() * rr, reach * (0.08 + 0.12 * rnd()))
                })
                .collect()
        } else {
            vec![]
        };
        // the outline of a wetting that reached `rr` (signed distance, m):
        // lobed where the floor dips, finer on a hard floor, where the edge
        // stops at every grain
        let fine = if hard { 0.06 } else { 0.02 };
        let outline = move |x: f64, y: f64, rr: f64, sd: u32| -> f64 {
            let rho = x.hypot(y) / rr;
            let (cu, cv) = if rho > 1e-6 { (x / rr / rho, y / rr / rho) } else { (1.0, 0.0) };
            let lobe = 1.0
                + 0.42 * (fbm(sd, cu * 1.2 + 4.0, cv * 1.2 + 4.0, 3) - 0.5)
                + 0.14 * (vnoise(sd ^ 0x9, cu * 4.0 + 2.0, cv * 4.0 + 2.0) - 0.5)
                + fine * (vnoise(sd ^ 0xa, cu * 10.0 + 1.0, cv * 10.0 + 1.0) - 0.5);
            (rho - lobe) * rr
        };
        let mut img = ctx.create_image_data(cw, ch);
        let n = cw as usize;
        par_rows(&mut img.data, n * 4, |j, out| {
            let y = (j as f64 + 0.5) * s - r;
            for i in 0..n {
                let x = (i as f64 + 0.5) * s - r;
                let mut px = Px::CLEAR;
                let mut d = outline(x, y, reach, seed);
                for &(qx, qz, qr) in &sats {
                    d = smin(d, (x - qx).hypot(y - qz) - qr, 0.01);
                }
                if hard {
                    d += (vnoise(seed ^ 0x4d, x * 90.0, y * 90.0) - 0.5) * 0.003;
                    // where it reached before: a dried rim of what it carried
                    for &(rr, sd, k) in &rims {
                        let dr = outline(x, y, rr, sd) + (vnoise(sd, x * 70.0, y * 70.0) - 0.5) * 0.004;
                        px.lay([0.2, 0.17, 0.13], tide(dr, aa, 0.004) * k * (0.5 + 0.5 * fbm(sd ^ 0x3, x * 20.0, y * 20.0, 2)));
                    }
                    // the damp margin, the film, the dark line of its edge
                    if d > 0.0 {
                        px.lay([0.11, 0.105, 0.1], 0.22 * (-d / 0.012).exp() * (0.6 + 0.4 * vnoise(seed ^ 0x5e, x * 50.0, y * 50.0)));
                    }
                    px.lay(wet, (0.5 - d / aa).clamp(0.0, 1.0) * (0.36 + 0.08 * vnoise(seed ^ 0x3c, x * 25.0, y * 25.0)));
                    px.lay([0.035, 0.035, 0.04], tide(d, aa, 0.0015) * 0.35);
                } else if standing {
                    // water on the pile: its edge soft over a few fibres
                    let dd = d + (vnoise(seed ^ 0x71, x * 160.0, y * 160.0) - 0.5) * 0.004;
                    px.lay(wet, sstep(0.004, -0.008, dd) * (0.26 + 0.08 * vnoise(seed ^ 0x3c, x * 40.0, y * 40.0)));
                } else {
                    // soaked in: the edge ragged and soft, a darker rim at it
                    let dd = d
                        + (fbm(seed ^ 0x77, x * 12.0, y * 12.0, 2) - 0.5) * 0.016
                        + (fbm(seed ^ 0x78, x * 45.0, y * 45.0, 2) - 0.5) * 0.01;
                    px.lay(wet, sstep(0.012, -0.024, dd) * (0.55 + 0.15 * vnoise(seed ^ 0x3c, x * 30.0, y * 30.0)));
                    px.lay([0.09, 0.07, 0.05], 0.24 * (-(dd + 0.004).abs() / 0.007).exp());
                }
                // never a square's edge
                px.a *= sstep(0.0, 0.025, r - x.abs().max(y.abs()));
                let c = px.rgba();
                for (k, v) in c.iter().enumerate() {
                    out[i * 4 + k] = (v.clamp(0.0, 1.0) * 255.0).round() as u8;
                }
            }
        });
        ctx.put_image_data(&img, 0, 0);
    });
    let (sheen, shine) = if hard {
        (0.3, 40.0)
    } else if standing {
        (0.3, 34.0)
    } else {
        (0.03, 18.0)
    };
    let m = phong!({ map: &tex, transparent: true, depthWrite: false, polygonOffset: true, polygonOffsetFactor: -2, polygonOffsetUnits: -2, shininess: shine });
    m.m().specular.set_rgb(sheen, sheen, sheen);
    m
}

/// The ring a drop sends across standing water, on a plane: its crest (the
/// lights' sheen caught on its slope) at `DRIP_RING_AT` of the width, the
/// trough inside it, and a fainter second ring behind.
fn drip_ring_mat() -> Mat {
    let tex = mark_tex(128.0, 128.0, [0.6, 0.62, 0.64], 0, |ctx, cw, ch| {
        let mut img = ctx.create_image_data(cw, ch);
        let n = cw as usize;
        par_rows(&mut img.data, n * 4, |j, out| {
            let v = (j as f64 + 0.5) / ch - 0.5;
            for i in 0..n {
                let u = (i as f64 + 0.5) / cw - 0.5;
                let q = u.hypot(v);
                let band = |at: f64, w: f64| (-((q - at) / w).powi(2)).exp();
                let mut px = Px::CLEAR;
                px.lay([0.04, 0.045, 0.05], 0.22 * band(DRIP_RING_AT - 0.035, 0.018) + 0.1 * band(DRIP_RING_AT - 0.14, 0.014));
                px.lay([0.62, 0.64, 0.66], 0.42 * band(DRIP_RING_AT, 0.014) + 0.16 * band(DRIP_RING_AT - 0.1, 0.012));
                let c = px.rgba();
                for (k, w) in c.iter().enumerate() {
                    out[i * 4 + k] = (w.clamp(0.0, 1.0) * 255.0).round() as u8;
                }
            }
        });
        ctx.put_image_data(&img, 0, 0);
    });
    lambert!({ map: &tex, transparent: true, depthWrite: false, polygonOffset: true, polygonOffsetFactor: -3, polygonOffsetUnits: -3 })
}
