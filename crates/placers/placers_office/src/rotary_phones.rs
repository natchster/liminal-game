//! `place_rotary_phones`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::ri as pick;

// Rotary desk phone: 12% chance per level. Vintage-style
// phone on floor: base box + rotary disc + handset with coiled cord.
pub(crate) fn place_rotary_phones(g: &mut Game, _dist: &[i32]) {
    g.pp.off_hook_rotary_positions = vec![]; // reset per level
    if rnd() < 0.88 {
        return;
    } // 12% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[pick(open.len())];
    let body_colors = [0x201818, 0x282a20, 0x1a1a20];
    let body_mat = lambert!({ color: body_colors[pick(body_colors.len())] });
    let dial_mat = lambert!({ color: 0xa8a8a0 });
    let cord_mat = lambert!({ color: 0x080604 });
    let phone = group();
    // Base: wide short box.
    let base = mesh(pg("RotaryPhones__base", box_geo!(0.18, 0.05, 0.14)), &body_mat);
    base.position().y = 0.025;
    phone.add(&base);
    // Rotary dial: circular disc on top-front.
    let dial = mesh(pg("RotaryPhones__dial", cyl_geo!(0.045, 0.045, 0.008, 12)), &dial_mat);
    dial.position().set(0, 0.054, 0.02);
    phone.add(&dial);
    // Small darker inner center on dial.
    let dial_center = mesh(pg("RotaryPhones__dialCenter", cyl_geo!(0.010, 0.010, 0.010, 8)), &body_mat);
    dial_center.position().set(0, 0.060, 0.02);
    phone.add(&dial_center);
    // 25% chance: handset off-hook — placed on floor beside base.
    let off_hook = rnd() < 0.25;
    let handset = group();
    let bar = mesh(pg("RotaryPhones__handsetBar", box_geo!(0.16, 0.03, 0.03)), &body_mat);
    handset.add(&bar);
    for off in [-0.075, 0.075] {
        let ear = mesh(pg("RotaryPhones__ear", sphere_geo!(0.025, 8, 6)), &body_mat);
        ear.scale().set(1, 0.7, 1);
        ear.position().set(off, 0, 0);
        handset.add(&ear);
    }
    if off_hook {
        // Handset placed on floor 0.15-0.20 m away from base.
        let oh_ang = rnd() * PI * 2.0;
        let oh_rad = 0.15 + rnd() * 0.05;
        handset.position().set(oh_ang.cos() * oh_rad, 0.017, oh_ang.sin() * oh_rad);
        handset.rotation().y = rnd() * PI * 2.0;
        handset.rotation().z = 0.0;
    } else {
        handset.rotation().z = (rnd() - 0.5) * 0.15;
        // lower ear cup rests on its cradle horn (base top 0.05) whatever the tilt
        let hz = handset.rot().z;
        handset.position().set(0, 0.0685 + hz.abs() * 0.075, -0.045);
    }
    phone.add(&handset);
    // Coiled cord: single short cylinder curling from handset to base.
    let cord = mesh(pg("RotaryPhones__cord", cyl_geo!(0.004, 0.004, if off_hook { 0.20 } else { 0.15 }, 4)), &cord_mat);
    if off_hook {
        // lie along the floor from the base toward the handset
        let hp = handset.pos();
        cord.rotation().z = -PI / 2.0;
        cord.rotation().y = -hp.z.atan2(hp.x);
        cord.position().set(hp.x * 0.5, 0.005, hp.z * 0.5);
    } else {
        cord.rotation().z = PI / 2.0 + (rnd() - 0.5) * 0.3;
        cord.position().set(0.05, 0.06, -0.05);
    }
    phone.add(&cord);
    phone.position().set(g.wx(x) + (rnd() - 0.5) * 2.2, 0, g.wz(z) + (rnd() - 0.5) * 2.2);
    phone.rotation().y = rnd() * PI * 2.0;
    g.world.add(&phone);
    // Track off-hook variants for dial-tone audio.
    if off_hook {
        let pp = phone.pos();
        g.pp.off_hook_rotary_positions.push(v3(pp.x, 0.10, pp.z));
    }
}
