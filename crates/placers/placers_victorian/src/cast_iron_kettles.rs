//! `place_cast_iron_kettles`
use game::prelude::*;
use placers_kit::place::face_wall;
use placers_kit::place::pick;
use placers_kit::place::wall_tiles;

// Hanging cast iron kettle: 3% chance per level. Wall-hung tea kettle w/
// spout + handle.
pub(crate) fn place_cast_iron_kettles(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    } // 3% chance
    let walls = wall_tiles(g);
    if walls.is_empty() {
        return;
    }
    let (x, z, dx, dz) = pick(&walls);
    let iron_mat = lambert!({ color: 0x181818 });
    let kettle = group();
    let mount_y = 1.75;
    // Bulbous body (sphere).
    let body = mesh(pg("CastIronKettles__body", sphere_geo!(0.060, 12, 10)), &iron_mat);
    body.scale().y = 0.85;
    body.position().set(0, mount_y, 0.055);
    kettle.add(&body);
    // Angled spout extending up-forward.
    let spout = mesh(pg("CastIronKettles__spout", cyl_geo!(0.010, 0.014, 0.070, 6)), &iron_mat);
    spout.rotation().z = PI / 3.0; // narrow mouth up and out at -X, wide root in the body
    spout.position().set(-0.055, mount_y + 0.028, 0.060);
    kettle.add(&spout);
    // Lid on top (short disc).
    let lid = mesh(pg("CastIronKettles__lid", cyl_geo!(0.020, 0.024, 0.008, 8)), &iron_mat);
    lid.position().set(0, mount_y + 0.056, 0.055);
    kettle.add(&lid);
    // Lid knob.
    let knob = mesh(pg("CastIronKettles__knob", sphere_geo!(0.005, 5, 4)), &iron_mat);
    knob.position().set(0, mount_y + 0.064, 0.055);
    kettle.add(&knob);
    // Curved handle arch over top.
    let handle = mesh(pg("CastIronKettles__handle", torus_geo!(0.055, 0.004, 4, 10, PI)), &iron_mat);
    handle.rotation().z = 0.0;
    handle.rotation().y = PI / 2.0;
    // ends on the body's shoulders (surface y ≈ +0.020 at 5.5 cm out), arch clears the knob
    handle.position().set(0, mount_y + 0.018, 0.055);
    kettle.add(&handle);
    kettle.rotation().y = face_wall(dx, dz);
    kettle.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - 0.060), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - 0.060));
    g.world.add(&kettle);
}
