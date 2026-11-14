//! `place_gameboys`
use game::prelude::*;
use placers_kit::place::jitter;
use placers_kit::place::pick_open;

// Gameboy-style handheld: 5% chance per level.
pub(crate) fn place_gameboys(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let body_colors = [0xe0e0e0, 0x40a0e0, 0xe08040, 0xa0d080];
    let body_mat = lambert!({ color: choice(&body_colors) });
    let screen_frame_mat = lambert!({ color: 0x404040 });
    let screen_mat = lambert!({ color: 0x506830 });
    let dpad_mat = lambert!({ color: 0x101010 });
    let ab_mat = lambert!({ color: 0x802020 });
    let gb = group();
    let body = mesh(pg("Gameboys__body", box_geo!(0.060, 0.014, 0.090)), &body_mat);
    body.position().y = 0.007;
    gb.add(&body);
    let screen_frame = mesh(pg("Gameboys__bezel", box_geo!(0.048, 0.002, 0.038)), &screen_frame_mat);
    screen_frame.position().set(0, 0.015, -0.020);
    gb.add(&screen_frame);
    let screen = mesh(pg("Gameboys__screen", box_geo!(0.036, 0.001, 0.028)), &screen_mat);
    screen.position().set(0, 0.0164, -0.020); // on the bezel (top 0.016)
    gb.add(&screen);
    // D-pad plus (2 cross bars), on the body (top 0.014).
    let dpad_h = mesh(pg("Gameboys__dpadH", box_geo!(0.014, 0.003, 0.005)), &dpad_mat);
    dpad_h.position().set(-0.016, 0.0153, 0.020);
    gb.add(&dpad_h);
    let dpad_v = mesh(pg("Gameboys__dpadV", box_geo!(0.005, 0.003, 0.014)), &dpad_mat);
    dpad_v.position().set(-0.016, 0.0153, 0.020);
    gb.add(&dpad_v);
    // A + B buttons.
    let btn_a = mesh(pg("Gameboys__button", cyl_geo!(0.005, 0.005, 0.003, 8)), &ab_mat);
    btn_a.position().set(0.020, 0.0153, 0.014);
    gb.add(&btn_a);
    let btn_b = mesh(pg("Gameboys__button", cyl_geo!(0.005, 0.005, 0.003, 8)), &ab_mat);
    btn_b.position().set(0.012, 0.0153, 0.024);
    gb.add(&btn_b);
    // Start + Select bars (2 small pills).
    for i in 0..2 {
        let bar = mesh(pg("Gameboys__pill", cyl_geo!(0.002, 0.002, 0.008, 5)), &screen_frame_mat);
        bar.rotation().z = PI / 2.0;
        bar.position().set(-0.006 + i as f64 * 0.012, 0.016, 0.036);
        gb.add(&bar);
    }
    // Speaker dots (5 tiny holes) on the body top.
    for d in 0..5 {
        let dot = mesh(cyl_geo!(0.001, 0.001, 0.001, 4), &screen_frame_mat);
        dot.position().set(0.014 + d as f64 * 0.003, 0.0143, 0.038);
        gb.add(&dot);
    }
    let (px, pz) = jitter(g, x, z, 2.4);
    gb.position().set(px, 0, pz);
    gb.rotation().y = rnd() * PI * 2.0;
    g.world.add(&gb);
}
