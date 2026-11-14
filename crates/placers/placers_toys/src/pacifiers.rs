//! `place_pacifiers`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Pacifier: 7% chance per level. Nipple + shield + ring.
pub(crate) fn place_pacifiers(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.93 {
        return;
    } // 7% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let shield_colors: [u32; 4] = [0xf0c0c0, 0xc0c0f0, 0xf0e0a0, 0xa0e0c0];
    let shield_mat = lambert!({ color: pick(&shield_colors) });
    let nipple_mat = lambert!({ color: 0xf0d8c0 });
    let paci = group();
    // Shield (flat disc lying on floor).
    let shield = mesh(pg("Pacifiers__shield", cyl_geo!(0.018, 0.018, 0.004, 10)), &shield_mat);
    shield.rotation().x = PI / 2.0;
    shield.position().set(0, 0.018, 0);
    paci.add(&shield);
    // Nipple (small elongated ellipsoid).
    let nipple = mesh(pg("Pacifiers__teat", sphere_geo!(0.006, 6, 5)), &nipple_mat);
    nipple.scale().set(1, 1, 1.6);
    nipple.position().set(0, 0.018, -0.010);
    paci.add(&nipple);
    // Ring on back (grip torus).
    let ring = mesh(pg("Pacifiers__ring", torus_geo!(0.010, 0.003, 5, 12)), &shield_mat);
    // hinged at the back centre of the shield (top of the ring there), swung
    // down and back until its far edge rests on the floor
    ring.rotation().x = -0.67;
    ring.position().set(0, 0.0102, 0.0102);
    paci.add(&ring);
    paci.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    paci.rotation().y = rnd() * PI * 2.0;
    g.world.add(&paci);
}
