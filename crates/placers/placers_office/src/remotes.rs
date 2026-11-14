//! `place_remotes`
use game::prelude::*;
use placers_kit::place::jitter;
use placers_kit::place::pick_open;

// TV remote control: 6% chance per level.
pub(crate) fn place_remotes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let body_colors = [0x202020, 0x606060, 0x804020, 0xe0e0d0];
    let body_mat = lambert!({ color: choice(&body_colors) });
    let btn_mat = lambert!({ color: 0x404040 });
    let big_btn_mat = lambert!({ color: 0x9c3a32 });
    let ir_mat = basic_mat!({ color: 0x5a1e1a });
    let remote = group();
    let body = mesh(pg("Remotes__body", box_geo!(0.045, 0.014, 0.18)), &body_mat);
    body.position().y = 0.007;
    remote.add(&body);
    // IR emitter dot (dark red, at one end).
    let ir = mesh(circle_geo!(0.004, 6), &ir_mat);
    ir.rotation().x = -PI / 2.0;
    ir.position().set(0, 0.015, -0.088);
    remote.add(&ir);
    let power = mesh(pg("Remotes__power", cyl_geo!(0.006, 0.006, 0.003, 8)), &big_btn_mat);
    power.position().set(0, 0.016, -0.070);
    remote.add(&power);
    // Number pad 3x4 grid.
    for r in 0..4 {
        for c in 0..3 {
            let key = mesh(pg("Remotes__key", box_geo!(0.008, 0.002, 0.008)), &btn_mat);
            key.position().set(-0.010 + c as f64 * 0.010, 0.015, -0.030 + r as f64 * 0.014);
            remote.add(&key);
        }
    }
    let vol = mesh(pg("Remotes__rocker", box_geo!(0.008, 0.003, 0.020)), &btn_mat);
    vol.position().set(-0.014, 0.016, 0.048);
    remote.add(&vol);
    let ch = mesh(pg("Remotes__rocker", box_geo!(0.008, 0.003, 0.020)), &btn_mat);
    ch.position().set(0.014, 0.016, 0.048);
    remote.add(&ch);
    let menu = mesh(pg("Remotes__menu", box_geo!(0.014, 0.002, 0.008)), &btn_mat);
    menu.position().set(0, 0.015, 0.076);
    remote.add(&menu);
    let (px, pz) = jitter(g, x, z, 2.4);
    remote.position().set(px, 0, pz);
    remote.rotation().y = rnd() * PI * 2.0;
    g.world.add(&remote);
}
