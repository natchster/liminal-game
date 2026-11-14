//! `place_sippy_cups`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Sippy cup: 6% chance per level. Colored cup + lid + spout + 2 handles.
pub(crate) fn place_sippy_cups(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let body_colors: [u32; 5] = [0xc02020, 0x2060c0, 0x40a040, 0xd050a0, 0xf0e020];
    let body_mat = lambert!({ color: pick(&body_colors) });
    let lid_mat = lambert!({ color: 0xf8f4e8 });
    let spout_mat = lambert!({ color: 0xf0d8c0 });
    let cup = group();
    // Cup body (tapered cylinder).
    let body = mesh(pg("SippyCups__body", cyl_geo!(0.030, 0.026, 0.070, 12)), &body_mat);
    body.position().y = 0.035;
    cup.add(&body);
    // Lid (wider disc on top).
    let lid = mesh(pg("SippyCups__lid", cyl_geo!(0.032, 0.032, 0.010, 12)), &lid_mat);
    lid.position().y = 0.075;
    cup.add(&lid);
    // Spout (small extended cylinder).
    let spout = mesh(pg("SippyCups__spout", cyl_geo!(0.006, 0.008, 0.020, 6)), &spout_mat);
    spout.rotation().z = 0.4;
    spout.position().set(0.020, 0.088, 0);
    cup.add(&spout);
    // 2 side handles (half-torus each).
    for side in [-1.0, 1.0] {
        let handle = mesh(pg("SippyCups__handle", torus_geo!(0.014, 0.005, 5, 8, PI)), &body_mat);
        // C in the radial plane, bulging out sideways; both ends in the wall
        handle.rotation().z = -side * PI / 2.0;
        handle.position().set(side * 0.026, 0.035, 0);
        cup.add(&handle);
    }
    // 40% chance tipped on side: laid over about X so the handles stay level,
    // resting on the lid rim and the foot (Y last keeps the random heading).
    let tipped = rnd() < 0.4;
    if tipped {
        cup.set_rotation_order(EulerOrder::YXZ);
        cup.rotation().x = PI / 2.0 - 0.074;
    }
    cup.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, if tipped { 0.026 } else { 0.0 }, g.wz(z) + (rnd() - 0.5) * 2.4);
    cup.rotation().y = rnd() * PI * 2.0;
    g.world.add(&cup);
}
