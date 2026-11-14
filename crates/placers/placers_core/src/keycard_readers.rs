//! `place_keycard_readers`
use game::prelude::*;
use placers_kit::place::shuffle;
use placers_kit::place::wall_candidates as wall_cands;

// Keycard reader: 25% chance per level, 1-2 units.
pub(crate) fn place_keycard_readers(g: &mut Game, _dist: &[i32]) {
    g.pp.keycard_reader_positions.clear(); // reset per level
    if rnd() < 0.75 {
        return;
    } // 25% chance
    let count = 1 + (rnd() * 2.0).floor() as usize;
    let mut cands = wall_cands(g);
    shuffle(&mut cands);
    let body_mat = lambert!({ color: 0x2a2828 });
    let slot_mat = lambert!({ color: 0x8a8880 });
    // Per-weather LED color for readers (`led_color_by_weather`, as the motion sensors').
    let led_color = led_color_by_weather(g.weather_key());
    let red_led_mat = basic_mat!({ color: led_color });
    let mut i = 0;
    while i < count && i < cands.len() {
        let c = cands[i];
        let reader = group();
        let body = mesh(pg("KeycardReaders__body", box_geo!(0.10, 0.14, 0.03)), &body_mat);
        reader.add(&body);
        // Card slot (horizontal thin band, upper portion).
        let slot = mesh(plane_geo!(0.08, 0.008), &slot_mat);
        slot.position().set(0, 0.03, 0.016);
        reader.add(&slot);
        // Tiny red LED near bottom.
        let led = mesh(box_geo!(0.008, 0.008, 0.008), &red_led_mat);
        led.position().set(0, -0.04, 0.017);
        reader.add(&led);
        let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.02);
        let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.02);
        let reader_height = 1.2 + rnd() * 0.2;
        if c.dx.abs() == 1 {
            reader.rotation().y = if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 };
            reader.position().set(wall_x, reader_height, wall_z + (rnd() - 0.5) * 1.2);
        } else {
            reader.rotation().y = if c.dz > 0 { PI } else { 0.0 };
            reader.position().set(wall_x + (rnd() - 0.5) * 1.2, reader_height, wall_z);
        }
        g.world.add(&reader);
        // Track for beep audio events.
        let rp = reader.pos();
        g.pp.keycard_reader_positions.push(v3(rp.x, reader_height, rp.z));
        i += 1;
    }
}
