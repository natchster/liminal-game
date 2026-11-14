//! `place_cell_phones`
use game::prelude::*;
use placers_kit::place::jitter;
use placers_kit::place::pick_open;

// Cell phone (brick style): 6% chance per level.
pub(crate) fn place_cell_phones(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let body_colors = [0x202020, 0x606060, 0x2040a0, 0xa02020, 0x408060];
    let body_mat = lambert!({ color: choice(&body_colors) });
    let screen_mat = lambert!({ color: 0x203020 });
    let key_mat = lambert!({ color: 0x808080 });
    let phone = group();
    let body = mesh(pg("CellPhones__body", box_geo!(0.045, 0.014, 0.100)), &body_mat);
    body.position().y = 0.007;
    phone.add(&body);
    let screen = mesh(box_geo!(0.032, 0.001, 0.024), &screen_mat);
    screen.position().set(0, 0.015, -0.028);
    phone.add(&screen);
    // 12 keypad buttons (3x4 grid).
    for r in 0..4 {
        for c in 0..3 {
            let key = mesh(pg("CellPhones__key", box_geo!(0.010, 0.002, 0.010)), &key_mat);
            key.position().set(-0.012 + c as f64 * 0.012, 0.015, -0.008 + r as f64 * 0.014);
            phone.add(&key);
        }
    }
    let antenna = mesh(pg("CellPhones__antenna", cyl_geo!(0.002, 0.002, 0.020, 4)), &body_mat);
    antenna.position().set(0.020, 0.020, -0.055);
    phone.add(&antenna);
    let (px, pz) = jitter(g, x, z, 2.4);
    phone.position().set(px, 0, pz);
    phone.rotation().y = rnd() * PI * 2.0;
    g.world.add(&phone);
}
