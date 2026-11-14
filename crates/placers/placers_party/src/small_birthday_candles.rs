//! `place_small_birthday_candles`: Small birthday candle: 8% chance per level. Wax cylinder + wick + optional tiny flame with faint point light. Party-remnant.
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

/// Small birthday candle: 8% chance per level. Wax cylinder +
/// wick + optional tiny flame with faint point light. Party-remnant.
pub(crate) fn place_small_birthday_candles(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let wax_colors = [0xf0e8d8, 0xf0c0d0, 0xd8f0d0, 0xd0d8f0, 0xf0d8a0, 0xe0b0e0];
    let wax_mat = lambert!({ color: pick(&wax_colors) });
    let wick_mat = lambert!({ color: 0x2a2418 });
    let candle = group();
    // Body: thin tall cylinder.
    let body = mesh(pg("SmallBirthdayCandles__body", cyl_geo!(0.008, 0.008, 0.06, 8)), &wax_mat);
    body.position().y = 0.03;
    candle.add(&body);
    // Wick: tiny dark cylinder on top.
    let wick = mesh(pg("SmallBirthdayCandles__wick", cyl_geo!(0.001, 0.001, 0.008, 4)), &wick_mat);
    wick.position().y = 0.064;
    candle.add(&wick);
    // 40% chance lit: tiny teardrop flame + faint point light.
    let is_lit = rnd() < 0.40;
    if is_lit {
        let flame_mat = basic_mat!({ color: 0xffd070, transparent: true, opacity: 0.85 });
        let flame = mesh(cone_geo!(0.003, 0.010, 5), &flame_mat);
        flame.position().y = 0.074;
        candle.add(&flame);
        let flame_light = point_light!(0xffb060, 0.35, 0.9, 2);
        flame_light.position().y = 0.075;
        candle.add(&flame_light);
        // Warm halo sphere for readable bloom.
        let halo_mat = basic_mat!({ color: 0xffb060, transparent: true, opacity: 0.25, depthWrite: false });
        let halo = mesh(sphere_geo!(0.020, 6, 5), &halo_mat);
        halo.position().y = 0.076;
        candle.add(&halo);
        // Subtle flicker.
        let flicker_phase = rnd() * PI * 2.0;
        let flicker_rate = 6.0 + rnd() * 4.0;
        g.animate(move |_g, _dt, t| {
            let p = t * flicker_rate + flicker_phase;
            let wobble = p.dsin() * 0.5 + (p * 1.7).dsin() * 0.3 + (p * 3.1).dsin() * 0.2;
            flame_light.set_intensity(0.35 + wobble * 0.08);
            flame.scale().y = 1.0 + wobble * 0.12;
            halo.scale().set_scalar(1.0 + wobble * 0.15);
        });
    }
    // Occasional tipped variant.
    if !is_lit && rnd() < 0.20 {
        candle.rotation().z = PI / 2.0;
        candle.position().y = 0.008;
    }
    candle.position().x = g.wx(x) + (rnd() - 0.5) * 2.4;
    candle.position().z = g.wz(z) + (rnd() - 0.5) * 2.4;
    candle.rotation().y = rnd() * PI * 2.0;
    g.world.add(&candle);
}
