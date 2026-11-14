//! `place_candle_lanterns`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Metal candle lantern: 4% chance per level. Frame + glass panels +
// candle + handle.
pub(crate) fn place_candle_lanterns(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let frame_mat = lambert!({ color: 0x303030 });
    let glass_mat = basic_mat!({ color: 0xe8e0c0, transparent: true, opacity: 0.35 });
    let candle_mat = lambert!({ color: 0xf0e8d0 });
    let is_lit = rnd() < 0.4;
    let flame_mat = basic_mat!({ color: 0xffa040 });
    let lantern = group();
    // Base disc.
    let base = mesh(pg("CandleLanterns__base", cyl_geo!(0.045, 0.045, 0.010, 6)), &frame_mat);
    base.position().y = 0.005;
    lantern.add(&base);
    // 4 corner posts.
    for i in 0..4 {
        let a = (i as f64 / 4.0) * PI * 2.0 + PI / 4.0;
        let post = mesh(pg("CandleLanterns__post", cyl_geo!(0.003, 0.003, 0.14, 4)), &frame_mat);
        post.position().set(a.cos() * 0.032, 0.075, a.sin() * 0.032);
        lantern.add(&post);
    }
    // 4 glass panels (planes between posts).
    for i in 0..4 {
        let a = (i as f64 / 4.0) * PI * 2.0;
        let panel = mesh(plane_geo!(0.045, 0.12), &glass_mat);
        // on the line between two diagonal posts (0.032 · cos 45° from the centre)
        panel.position().set(a.cos() * 0.032 * std::f64::consts::FRAC_1_SQRT_2, 0.075, a.sin() * 0.032 * std::f64::consts::FRAC_1_SQRT_2);
        panel.rotation().y = -a + PI / 2.0;
        lantern.add(&panel);
    }
    // Top cap.
    let top = mesh(pg("CandleLanterns__top", cyl_geo!(0.045, 0.045, 0.010, 6)), &frame_mat);
    top.position().y = 0.145;
    lantern.add(&top);
    // Candle inside.
    let candle = mesh(pg("CandleLanterns__candle", cyl_geo!(0.012, 0.012, 0.060, 8)), &candle_mat);
    candle.position().y = 0.045;
    candle.scale().y = 0.070 / 0.060; // from the plinth top (0.010) up to the flame (0.081)
    lantern.add(&candle);
    if is_lit {
        // Wick + flame.
        let flame = mesh(cone_geo!(0.005, 0.014, 5), &flame_mat);
        flame.position().y = 0.088;
        lantern.add(&flame);
        // Light source.
        let light = point_light!(0xffa040, 0.18, 1.5);
        light.position().y = 0.088;
        lantern.add(&light);
        // Flame flicker animator (rapid noise-like intensity + scale wobble).
        let phase = rnd() * PI * 2.0;
        g.animate(move |_g, _dt, t| {
            // Rapid flicker via combined sines (like turbulent flame).
            let f = 0.7 + 0.15 * (t * 7.0 + phase).sin() + 0.15 * (t * 11.0 + phase * 1.7).sin();
            flame.scale().set_scalar(f);
            light.set_intensity(0.18 * f);
        });
    }
    // Handle loop on top (torus arch).
    let handle = mesh(pg("CandleLanterns__handle", torus_geo!(0.020, 0.002, 3, 8, PI)), &frame_mat);
    handle.rotation().y = PI / 2.0;
    handle.position().y = 0.149; // hooked ends in the cap's ears (y 0.145–0.150), not 5 mm above
    lantern.add(&handle);
    lantern.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    lantern.rotation().y = rnd() * PI * 2.0;
    g.world.add(&lantern);
}
