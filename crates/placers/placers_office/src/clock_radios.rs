//! `place_clock_radios`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::ri;

// Small clock radio: 12% chance per level. Block with tuning
// dial + LED digital display. Classic bedside device.
pub(crate) fn place_clock_radios(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.88 {
        return;
    } // 12% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[ri(open.len())];
    let case_mat = lambert!({ color: 0x2a2418 });
    let dial_mat = lambert!({ color: 0xd8d0c0 });
    let led_mat = basic_mat!({ color: 0xe02818 });
    let knob_mat = lambert!({ color: 0x8a7828 });
    let radio = group();
    // Case: box.
    let body = mesh(pg("ClockRadios__case", box_geo!(0.20, 0.08, 0.12)), &case_mat);
    body.position().y = 0.04;
    radio.add(&body);
    // Tuning dial: cream horizontal rectangle on front.
    let dial = mesh(plane_geo!(0.12, 0.03), &dial_mat);
    dial.position().set(-0.03, 0.055, 0.061);
    radio.add(&dial);
    // Small LED digital display on right side of front.
    let led = mesh(plane_geo!(0.04, 0.025), &led_mat);
    led.position().set(0.07, 0.055, 0.061);
    radio.add(&led);
    // Small knob (right side of case).
    let knob = mesh(pg("ClockRadios__knob", cyl_geo!(0.012, 0.012, 0.006, 6)), &knob_mat);
    knob.rotation().x = PI / 2.0;
    knob.position().set(0.09, 0.025, 0.061);
    radio.add(&knob);
    radio.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    radio.rotation().y = rnd() * PI * 2.0;
    g.world.add(&radio);
}
