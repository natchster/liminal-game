//! `place_emergency_lanterns`
use game::prelude::*;
use placers_kit::place::pick_wall;
use placers_kit::place::wall_mount;

// Hanging emergency lantern: 3% chance per level. Wall-mounted lantern w/ warm light.
pub(crate) fn place_emergency_lanterns(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    }
    let Some((x, z, dx, dz)) = pick_wall(g) else { return };
    let bracket_mat = lambert!({ color: 0x404040 });
    let body_mat = lambert!({ color: 0x505050 });
    let lens_mat = basic_mat!({ color: 0xffe0a0 }); // self-lit warm.
    let lantern = group();
    let mount_y = 1.75;
    let bracket = mesh(pg("EmergencyLanterns__bracket", box_geo!(0.010, 0.02, 0.08)), &bracket_mat);
    bracket.position().set(0, mount_y + 0.05, 0.05);
    lantern.add(&bracket);
    // Chain link (2 small torus rings), alternate links at 90 degrees.
    for i in 0..2 {
        let link = mesh(pg("EmergencyLanterns__link", torus_geo!(0.006, 0.001, 3, 6)), &bracket_mat);
        link.rotation().y = if i % 2 == 0 { 0.0 } else { PI / 2.0 };
        link.position().set(0, mount_y + 0.034 - i as f64 * 0.012, 0.08);
        lantern.add(&link);
    }
    let body = mesh(pg("EmergencyLanterns__body", box_geo!(0.065, 0.09, 0.055)), &body_mat);
    body.position().set(0, mount_y - 0.03, 0.10);
    lantern.add(&body);
    // Warm lens on front (glowing), seated on the housing front.
    let lens = mesh(box_geo!(0.050, 0.055, 0.006), &lens_mat);
    lens.position().set(0, mount_y - 0.030, 0.129);
    lantern.add(&lens);
    let lamp_light = point_light!(0xffe4a0, 0.30, 2.2);
    lamp_light.position().set(0, mount_y - 0.03, 0.15);
    lantern.add(&lamp_light);
    // Subtle flicker animator: slow drift + very rare dip.
    let phase = rnd() * PI * 2.0;
    let lens_m = lens.mat();
    g.animate(move |_g, _dt, t| {
        let drift = 0.85 + 0.15 * (t * 0.7 + phase).dsin();
        let dip = if (t * 0.3 + phase * 2.0).dsin() > 0.94 { 0.55 } else { 1.0 };
        lamp_light.set_intensity(0.30 * drift * dip);
        let mut m = lens_m.m();
        m.opacity = drift * dip;
        m.transparent = true;
    });
    wall_mount(g, &lantern, x, z, dx, dz, 0.060);
    g.world.add(&lantern);
}
