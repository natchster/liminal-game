//! `place_smoke_detectors`
use game::prelude::*;
use placers_kit::place::jitter;
use placers_kit::place::pick_open;

// Ceiling smoke detector: 6% chance per level. Round puck on ceiling + red status LED.
pub(crate) fn place_smoke_detectors(g: &mut Game, _dist: &[i32]) {
    g.pp.smoke_detector_positions.clear(); // reset per level
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let Some((x, z)) = pick_open(g) else { return };
    let body_mat = lambert!({ color: 0xf0efe8 });
    let grill_mat = lambert!({ color: 0x808070 });
    let det = group();
    let ceil_y = WALL_H; // puck top on the ceiling
    let puck = mesh(pg("SmokeDetectors__body", cyl_geo!(0.075, 0.075, 0.028, 16)), &body_mat);
    puck.position().y = ceil_y - 0.014;
    det.add(&puck);
    let grill = mesh(pg("SmokeDetectors__grill", cyl_geo!(0.045, 0.045, 0.010, 16)), &grill_mat);
    grill.position().y = ceil_y - 0.030;
    det.add(&grill);
    // Red status LED (self-lit).
    let led = mesh(sphere_geo!(0.003, 6, 4), basic_mat!({ color: 0xff2020 }));
    led.position().set(0.030, ceil_y - 0.030, 0.030);
    det.add(&led);
    // Slow LED throb (0.4 Hz).
    let phase = rnd() * PI * 2.0;
    g.animate(move |_g, _dt, t| {
        let pulse = 0.7 + 0.3 * (t * 2.5 + phase).dsin();
        led.scale().set_scalar(pulse);
    });
    let (px, pz) = jitter(g, x, z, 1.0);
    det.position().set(px, 0, pz);
    g.world.add(&det);
    let p = det.pos();
    g.pp.smoke_detector_positions.push(v3(p.x, ceil_y, p.z)); // track for chirp
}
