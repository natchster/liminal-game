//! `place_wall_barometers`
use game::prelude::*;
use placers_kit::place::mount_slid_ref as wall_mount;
use placers_kit::place::ri as pick;
use placers_kit::place::wall_candidates;

// Wall barometer: 10% chance per level. Tall dark-wood case
// with dial face + 2 hanging weights below.
pub(crate) fn place_wall_barometers(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = candidates[pick(candidates.len())];
    let case_mat = lambert!({ color: 0x4a2818 });
    let dial_face_mat = lambert!({ color: 0xd8d0c0 });
    let weight_mat = lambert!({ color: 0xa08040 });
    let barometer = group();
    // Case: tall thin dark-wood rectangle.
    let case_b = mesh(pg("WallBarometers__case", box_geo!(0.18, 0.60, 0.05)), &case_mat);
    case_b.position().y = 0.0;
    barometer.add(&case_b);
    // Dial face: cream circle near top.
    let dial_f = mesh(circle_geo!(0.070, 16), &dial_face_mat);
    dial_f.position().set(0, 0.20, 0.026);
    barometer.add(&dial_f);
    // Small dark ring around dial.
    let dial_ring = mesh(pg("WallBarometers__ring", torus_geo!(0.070, 0.003, 4, 16)), &case_mat);
    dial_ring.rotation().x = 0.0;
    dial_ring.position().set(0, 0.20, 0.028);
    barometer.add(&dial_ring);
    // Tiny needle inside dial.
    let needle_mat = basic_mat!({ color: 0x8a2020 });
    let needle = mesh(plane_geo!(0.003, 0.055), &needle_mat);
    needle.position().set(0.010, 0.213, 0.030);
    needle.rotation().z = 0.5;
    barometer.add(&needle);
    // 2 hanging brass weights (small cylinders) below dial.
    for w_off in [-0.04, 0.04] {
        let w_cord = mesh(pg("WallBarometers__cord", cyl_geo!(0.002, 0.002, 0.28, 4)), &case_mat);
        w_cord.position().set(w_off, -0.08, 0.026);
        barometer.add(&w_cord);
        let weight = mesh(pg("WallBarometers__weight", cyl_geo!(0.013, 0.013, 0.05, 8)), &weight_mat);
        weight.position().set(w_off, -0.24, 0.026);
        barometer.add(&weight);
    }
    // Case back (0.05 deep) flush on the wall; it was 5 mm off it.
    let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.025);
    let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.025);
    let bar_height = 1.5 + rnd() * 0.15;
    wall_mount(&barometer, &c, wall_x, wall_z, bar_height, 0.8);
    g.world.add(&barometer);
}
