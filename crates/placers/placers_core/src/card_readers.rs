//! `place_card_readers`
use game::prelude::*;
use placers_kit::place::mount;
use placers_kit::place::wall_candidates;
use placers_kit::place::wall_xz;

// Swipe card reader: 4% chance per level. Dark box w/
// slot + status LED.
pub(crate) fn place_card_readers(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = choice(&candidates);
    let body_mat = lambert!({ color: 0x181410 });
    let led_mat = basic_mat!({ color: 0x30d030 });
    let slot_mat = basic_mat!({ color: 0x080604 });
    let reader = group();
    // Body.
    let body = mesh(pg("CardReaders__body", box_geo!(0.08, 0.12, 0.020)), &body_mat);
    body.position().set(0, 0, -0.010);
    reader.add(&body);
    // Card slot (dark horizontal bar).
    let slot = mesh(box_geo!(0.05, 0.006, 0.008), &slot_mat);
    slot.position().set(0, 0.02, 0.001);
    reader.add(&slot);
    // Status LED.
    let led = mesh(sphere_geo!(0.003, 5, 4), &led_mat);
    led.position().set(0.020, -0.02, 0.003);
    reader.add(&led);
    // Slow LED blink.
    let blink_phase = rnd() * PI * 2.0;
    {
        let led = led.clone();
        g.animate(move |_g, _dt, t| {
            led.set_visible((t * 1.5 + blink_phase).dsin() > 0.6);
        });
    }
    let (wall_x, wall_z) = wall_xz(g, c, 0.010);
    mount(&reader, c, wall_x, wall_z, 1.30, Some(0.6));
    g.world.add(&reader);
}
