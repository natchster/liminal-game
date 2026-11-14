//! `place_calculators`
use game::prelude::*;
use placers_kit::place::jitter;
use placers_kit::place::pick_open;

// Pocket calculator: 6% chance per level. Solar strip + display + button grid.
pub(crate) fn place_calculators(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let body_colors = [0x202020, 0x404060, 0x604040, 0xe0e0e0];
    let body_mat = lambert!({ color: choice(&body_colors) });
    let screen_mat = lambert!({ color: 0x506830 });
    let solar_mat = lambert!({ color: 0x201830 });
    let btn_mat = lambert!({ color: 0x808080 });
    let op_mat = lambert!({ color: 0xc0a020 });
    let calc = group();
    let body = mesh(pg("Calculators__body", box_geo!(0.055, 0.010, 0.085)), &body_mat);
    body.position().y = 0.005;
    calc.add(&body);
    let screen = mesh(box_geo!(0.042, 0.001, 0.018), &screen_mat);
    screen.position().set(0, 0.011, -0.028);
    calc.add(&screen);
    let solar = mesh(box_geo!(0.020, 0.001, 0.004), &solar_mat);
    solar.position().set(-0.014, 0.011, -0.0395); // clear of the LCD so the two don't z-fight
    calc.add(&solar);
    // 4x4 button grid.
    for r in 0..4 {
        for c in 0..4 {
            let is_op = c == 3;
            let key = mesh(pg("Calculators__key", box_geo!(0.009, 0.002, 0.009)), if is_op { &op_mat } else { &btn_mat });
            key.position().set(-0.018 + c as f64 * 0.012, 0.011, -0.010 + r as f64 * 0.014);
            calc.add(&key);
        }
    }
    let (px, pz) = jitter(g, x, z, 2.4);
    calc.position().set(px, 0, pz);
    calc.rotation().y = rnd() * PI * 2.0;
    g.world.add(&calc);
}
