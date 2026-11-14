//! `place_hair_dryers`
use game::prelude::*;
use placers_kit::place::face_wall as face_wall_yaw;
use placers_kit::place::pick;
use placers_kit::place::wall_tiles_first;

// Hair dryer wall mount: 4% chance per level.
pub(crate) fn place_hair_dryers(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let wall_tiles = wall_tiles_first(g);
    if wall_tiles.is_empty() {
        return;
    }
    let c = pick(&wall_tiles);
    let bracket_mat = lambert!({ color: 0x606060 });
    let dryer_colors = [0xe0e0e0, 0x404040, 0xc0a0a0];
    let dryer_mat = lambert!({ color: pick(&dryer_colors) });
    let nozzle_mat = lambert!({ color: 0x303030 });
    let dryer = group();
    let mount_y = 1.30;
    // Wall bracket plate. Group origin is 2 cm off the wall: the plate's back
    // is on the wall face and the dryer rests on the plate face.
    let plate = mesh(pg("HairDryers__plate", box_geo!(0.10, 0.14, 0.02)), &bracket_mat);
    plate.position().set(0, mount_y, -0.010);
    dryer.add(&plate);
    let dryer_z = 0.045;
    // Dryer body (hanging horizontally in bracket).
    let body = mesh(pg("HairDryers__body", cyl_geo!(0.045, 0.045, 0.16, 10)), &dryer_mat);
    body.rotation().z = PI / 2.0;
    body.position().set(0.06, mount_y - 0.02, dryer_z);
    dryer.add(&body);
    // Nozzle end (narrower).
    let nozzle = mesh(pg("HairDryers__nozzle", cyl_geo!(0.030, 0.038, 0.04, 10)), &nozzle_mat);
    nozzle.rotation().z = PI / 2.0;
    nozzle.position().set(0.16, mount_y - 0.02, dryer_z);
    dryer.add(&nozzle);
    // Handle (small perpendicular stem).
    let handle = mesh(pg("HairDryers__handle", cyl_geo!(0.025, 0.025, 0.10, 8)), &dryer_mat);
    handle.position().set(0.06, mount_y - 0.11, dryer_z);
    dryer.add(&handle);
    // Cord loop: a U hanging from the handle's butt, both sleeved ends 4 mm up into it.
    let cord = mesh(pg("HairDryers__cord", torus_geo!(0.025, 0.005, 4, 10, PI)), &nozzle_mat);
    cord.position().set(0.06, mount_y - 0.156, dryer_z);
    cord.rotation().z = PI;
    cord.scale().x = 0.65;
    dryer.add(&cord);
    // Align + push against wall.
    dryer.rotation().y = face_wall_yaw(c.dx, c.dz);
    dryer.position().set(g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.020), 0, g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.020));
    g.world.add(&dryer);
}
