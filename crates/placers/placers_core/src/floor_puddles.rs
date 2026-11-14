//! `place_floor_puddles`
use game::prelude::*;
use placers_kit::noise::fbm;
use placers_kit::noise::smin;
use placers_kit::noise::sstep;
use placers_kit::noise::tide;
use placers_kit::noise::vn as vnoise;
use placers_kit::paint::mark_tex;
use placers_kit::paint::paint_px_indexed as paint_px;
use placers_kit::place::open_tiles;
use placers_kit::place::ri as pick;
use placers_kit::wet_floor::floor_kind;
use placers_kit::wet_floor::Px;

// Floor puddle: 15% chance per level (25% in wet biome). Water on the
// floor (`docs/research/props-remodelling/marks_on_the_rooms_surfaces.md`): on carpet it has soaked in, a dark wet patch with a
// soft, ragged edge and a darker ring where the dirt wicked out to it; on a
// hard floor it stands, a thin film darker than the dry floor round it, its
// edge sharp and irregular, a damp margin outside it and a sheen of the
// lights on it. Painted on a transparent plane flat on the floor.
pub(crate) fn place_floor_puddles(g: &mut Game, _dist: &[i32]) {
    let chance = if g.weather_key() == "wet" { 0.25 } else { 0.15 };
    if rnd() > chance {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[pick(open.len())];
    // the old puddle's draws: its radius (16-30 cm) and how much longer
    // than wide it was (1-1.3)
    let puddle_r = 0.16 + rnd() * 0.14;
    let long = 1.0 + rnd() * 0.3;
    let (rx, rz) = (puddle_r, puddle_r * long);
    let floor = floor_kind(g, x, z);
    let (puddle_mat, sheen) = puddle_mat(rx, rz, floor);
    let geo = plane_geo!(rx * 2.5, rz * 2.5);
    geo.rotate_x(-PI / 2.0);
    let puddle = mesh(geo, &puddle_mat);
    puddle.position().set(g.wx(x) + (rnd() - 0.5) * 2.2, 0.003, g.wz(z) + (rnd() - 0.5) * 2.2);
    puddle.rotation().y = rnd() * PI * 2.0;
    g.world.add(&puddle);
    // 25% chance: where it is fed by a drip, the sheen on it trembles
    // (period 4-7 s; it was a swell of the old disc)
    if rnd() < 0.25 {
        let r_phase = rnd() * PI * 2.0;
        let r_period = 4.0 + rnd() * 3.0;
        let m2 = puddle_mat.clone();
        g.animate(move |_g, _dt, t| {
            let s = 1.0 + (t * (PI * 2.0 / r_period) + r_phase).sin() * 0.15;
            m2.m().specular.set_rgb(sheen * s, sheen * s, sheen * s);
        });
    }
}

/// Water on a floor of `floor` (`floor_kind`), about `rx × rz` m, on a plane
/// `2.5 rx × 2.5 rz`: soaked into carpet, a dark patch with a soft ragged
/// edge and a darker rim where the dirt wicked out; standing on a hard floor,
/// a dark film with a sharp, irregular edge, a damp margin outside it and a
/// sheen. Its material, and the sheen (linear specular) it was given.
fn puddle_mat(rx: f64, rz: f64, floor: u8) -> (Mat, f64) {
    let (pw, ph) = (rx * 2.5, rz * 2.5);
    let (tw, th) = (192.0, (192.0 * rz / rx).round().clamp(96.0, 256.0));
    let hard = floor != 0;
    let wet = if hard { [0.09, 0.09, 0.095] } else { [0.12, 0.1, 0.08] };
    let tex = mark_tex(tw, th, wet, move |ctx, cw, ch| {
        let seed = (rnd() * 4.0e9) as u32;
        let (sx, sy) = (pw / cw, ph / ch);
        let aa = sx.max(sy) * 1.2;
        let r0 = rx.min(rz);
        // on a hard floor, smaller pools beside it, joined to it or not
        let sats: Vec<(f64, f64, f64)> = if hard {
            (0..(rnd() * 3.5) as usize)
                .map(|_| {
                    let (a, rr) = (rnd() * PI * 2.0, 0.65 + 0.25 * rnd());
                    (a.cos() * rx * rr, a.sin() * rz * rr, r0 * (0.1 + 0.15 * rnd()))
                })
                .collect()
        } else {
            vec![]
        };
        paint_px(ctx, cw, ch, |i, j| {
            let (x, y) = ((i as f64 + 0.5) * sx - pw / 2.0, (j as f64 + 0.5) * sy - ph / 2.0);
            let (u, v) = (x / rx, y / rz);
            let rho = u.hypot(v);
            let (cu, cv) = (u / rho.max(1e-6), v / rho.max(1e-6));
            let lobe = 1.0
                + 0.5 * (fbm(seed, cu * 1.2 + 4.0, cv * 1.2 + 4.0, 3) - 0.5)
                + 0.16 * (vnoise(seed ^ 0x9, cu * 4.0 + 2.0, cv * 4.0 + 2.0) - 0.5)
                + 0.07 * (vnoise(seed ^ 0xa, cu * 10.0 + 1.0, cv * 10.0 + 1.0) - 0.5);
            let mut d = (rho - 0.88 * lobe) * r0;
            for &(qx, qz, qr) in &sats {
                d = smin(d, (x - qx).hypot(y - qz) - qr, 0.012);
            }
            let mut px = Px::CLEAR;
            if hard {
                d += (vnoise(seed ^ 0x4d, x * 90.0, y * 90.0) - 0.5) * 0.004;
                // the damp margin outside it, the film, and the dark line of
                // its edge
                if d > 0.0 {
                    px.lay([0.11, 0.105, 0.1], 0.2 * (-d / 0.015).exp() * (0.6 + 0.4 * vnoise(seed ^ 0x5e, x * 50.0, y * 50.0)));
                }
                px.lay(wet, (0.5 - d / aa).clamp(0.0, 1.0) * (0.4 + 0.08 * vnoise(seed ^ 0x3c, x * 25.0, y * 25.0)));
                px.lay([0.035, 0.035, 0.04], tide(d, aa, 0.0015) * 0.35);
            } else {
                // soaked in: the edge ragged and soft, a darker rim at it
                let dd = d + (fbm(seed ^ 0x77, x * 14.0, y * 14.0, 3) - 0.5) * 0.035;
                px.lay(wet, sstep(0.012, -0.025, dd) * (0.55 + 0.15 * vnoise(seed ^ 0x3c, x * 30.0, y * 30.0)));
                px.lay([0.09, 0.07, 0.05], 0.22 * (-(dd + 0.004).abs() / 0.008).exp());
            }
            px.rgba()
        });
    });
    let (sheen, shine) = if hard { (0.3, 40.0) } else { (0.03, 18.0) };
    let m = phong!({ map: &tex, transparent: true, depthWrite: false, polygonOffset: true, polygonOffsetFactor: -2, polygonOffsetUnits: -2, shininess: shine });
    m.m().specular.set_rgb(sheen, sheen, sheen);
    (m, sheen)
}
