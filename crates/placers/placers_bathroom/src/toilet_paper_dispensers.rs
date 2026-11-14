//! `place_toilet_paper_dispensers`
use game::prelude::*;
use placers_kit::place::face_wall as face_wall_yaw;
use placers_kit::place::pick;
use placers_kit::place::wall_tiles_first;

// Toilet paper dispenser: 5% chance per level, wall-mounted.
pub(crate) fn place_toilet_paper_dispensers(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let wall_tiles = wall_tiles_first(g);
    if wall_tiles.is_empty() {
        return;
    }
    let c = pick(&wall_tiles);
    let housing_mat = lambert!({ color: 0x606060 });
    let paper_mat = lambert!({ color: 0xf0e8d8 });
    let tp = group();
    let mount_y = 0.85;
    // Housing plate.
    let housing = mesh(pg("ToiletPaperDispensers__housing", box_geo!(0.24, 0.20, 0.14)), &housing_mat);
    housing.position().y = mount_y;
    tp.add(&housing);
    // Visible roll (horizontal cylinder) hanging out of the open bottom: its
    // axis is 1 cm under the housing.
    let roll = mesh(pg("ToiletPaperDispensers__roll", cyl_geo!(0.055, 0.055, 0.10, 12)), &paper_mat);
    roll.rotation().z = PI / 2.0;
    roll.position().set(0.070, mount_y - 0.11, 0);
    tp.add(&roll);
    // Roll core (dark inner).
    let core = mesh(pg("ToiletPaperDispensers__core", cyl_geo!(0.015, 0.015, 0.11, 8)), &housing_mat);
    core.rotation().z = PI / 2.0;
    core.position().set(0.070, mount_y - 0.11, 0);
    tp.add(&core);
    // Tear bar along the front bottom edge, over the roll.
    let lip = mesh(pg("ToiletPaperDispensers__lip", box_geo!(0.020, 0.010, 0.14)), &housing_mat);
    lip.rotation().y = PI / 2.0;
    lip.position().set(0.050, mount_y - 0.10, 0.062);
    tp.add(&lip);
    tp.rotation().y = face_wall_yaw(c.dx, c.dz);
    tp.position().set(g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.060), 0, g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.060));
    g.world.add(&tp);
}
