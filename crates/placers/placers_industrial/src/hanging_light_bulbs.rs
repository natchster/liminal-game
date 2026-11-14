//! `place_hanging_light_bulbs`
use game::prelude::*;
use placers_kit::place::open_tiles;

// Hanging bare bulb: 5% chance per level. Bulb + socket +
// cord from ceiling + faint warm glow.
pub(crate) fn place_hanging_light_bulbs(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = choice(&open);
    let cord_mat = lambert!({ color: 0x181008 });
    let socket_mat = lambert!({ color: 0x606870 });
    let bulb_mat = basic_mat!({ color: 0xf0e8b0, transparent: true, opacity: 0.75 });
    let bulb = group();
    let cord_len = 0.5 + rnd() * 0.5;
    let bulb_y = WALL_H - cord_len - 0.05;
    // Cord.
    let cord = mesh(pg("HangingLightBulbs__cord", cyl_geo!(0.002, 0.002, cord_len, 4)), &cord_mat);
    cord.position().y = WALL_H - cord_len / 2.0 - 0.02;
    bulb.add(&cord);
    // Socket.
    let socket = mesh(pg("HangingLightBulbs__socket", cyl_geo!(0.014, 0.014, 0.020, 6)), &socket_mat);
    socket.position().y = bulb_y + 0.020;
    bulb.add(&socket);
    // Bulb (elongated sphere).
    let bulb_body = mesh(sphere_geo!(0.024, 10, 8), &bulb_mat);
    bulb_body.scale().y = 1.3;
    bulb_body.position().y = bulb_y - 0.010;
    bulb.add(&bulb_body);
    // 40% chance lit — warm glow.
    if rnd() < 0.40 {
        let glow = point_light!(0xffe0a0, 0.4, 1.2, 2.0);
        glow.position().y = bulb_y - 0.010;
        bulb.add(&glow);
        // Subtle flicker.
        let flicker_phase = rnd() * PI * 2.0;
        g.animate(move |_g, _dt, t| {
            glow.set_intensity(0.4 + (t * 4.0 + flicker_phase).sin() * 0.06);
        });
    }
    let px = g.wx(x) + (rnd() - 0.5) * 2.0;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.0;
    bulb.position().set(px, 0, pz);
    // Slight sway.
    if rnd() < 0.50 {
        let sway_phase = rnd() * PI * 2.0;
        let sway_period = 10.0 + rnd() * 6.0;
        let b = bulb.clone();
        g.animate(move |_g, _dt, t| {
            b.rotation().z = (t * (PI * 2.0 / sway_period) + sway_phase).sin() * 0.015;
        });
    }
    g.world.add(&bulb);
}
