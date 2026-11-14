//! `place_broken_flashlights`
use game::prelude::*;
use placers_kit::place::open_tiles;

// Broken flashlight: 8% chance per level. Metal cylinder
// w/ head lens (cracked) + faint dim glow inside.
pub(crate) fn place_broken_flashlights(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = choice(&open);
    let body_colors = [0x181410, 0x606870, 0xa02020];
    let body_mat = lambert!({ color: choice(&body_colors) });
    let lens_mat = basic_mat!({ color: 0xe0e8f0, transparent: true, opacity: 0.5 });
    let flashlight = group();
    // Handle body.
    let body = mesh(pg("BrokenFlashlights__body", cyl_geo!(0.014, 0.014, 0.12, 8)), &body_mat);
    body.rotation().z = PI / 2.0;
    body.position().set(0, 0.014, 0);
    flashlight.add(&body);
    // Head (wider).
    let head = mesh(pg("BrokenFlashlights__head", cyl_geo!(0.020, 0.014, 0.024, 8)), &body_mat);
    head.rotation().z = -PI / 2.0;
    head.position().set(0.070, 0.014, 0);
    flashlight.add(&head);
    // Cracked lens.
    let lens = mesh(circle_geo!(0.018, 8), &lens_mat);
    lens.rotation().y = PI / 2.0;
    lens.position().set(0.083, 0.014, 0);
    flashlight.add(&lens);
    // 40% chance faint dim glow (dying beam).
    if rnd() < 0.40 {
        let glow = point_light!(0xffffe0, 0.20, 0.5, 2.0);
        glow.position().set(0.09, 0.014, 0);
        flashlight.add(&glow);
        // Very unsteady dying flicker.
        let flicker_phase = rnd() * PI * 2.0;
        g.animate(move |_g, _dt, t| {
            let flick = if (t * 12.0 + flicker_phase).dsin() > 0.5 { 1.0 } else { 0.15 };
            glow.set_intensity(0.20 * flick);
        });
    }
    // 30% chance tipped/rolling.
    if rnd() < 0.30 {
        flashlight.rotation().y = PI / 2.0 + (rnd() - 0.5) * 0.5;
    }
    flashlight.position().x = g.wx(x) + (rnd() - 0.5) * 2.4;
    flashlight.position().z = g.wz(z) + (rnd() - 0.5) * 2.4;
    flashlight.rotation().y += rnd() * PI * 2.0;
    // The head (r 0.020) is fatter than the barrel (r 0.014): tip it so the head
    // rim and the tail both rest on the floor instead of the head sinking 6 mm.
    flashlight.rotation().z = 0.042;
    flashlight.position().y = 0.0025;
    g.world.add(&flashlight);
}
