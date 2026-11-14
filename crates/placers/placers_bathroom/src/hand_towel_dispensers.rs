//! `place_hand_towel_dispensers`
use game::prelude::*;
use placers_kit::place::face_wall as face_wall_yaw;
use placers_kit::place::pick;
use placers_kit::place::wall_tiles_first;

// Hand towel dispenser: 4% chance per level, visible paper tongue.
pub(crate) fn place_hand_towel_dispensers(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let wall_tiles = wall_tiles_first(g);
    if wall_tiles.is_empty() {
        return;
    }
    let c = pick(&wall_tiles);
    let body_mat = lambert!({ color: 0x808070 });
    let paper_mat = lambert!({ color: 0xf0e8d0 });
    let disp = group();
    let mount_y = 1.35;
    // Main body (chunky rectangle).
    let body = mesh(pg("HandTowelDispensers__body", box_geo!(0.30, 0.40, 0.14)), &body_mat);
    body.position().y = mount_y;
    disp.add(&body);
    // Bottom slot lip.
    let lip = mesh(pg("HandTowelDispensers__lip", box_geo!(0.24, 0.014, 0.10)), &body_mat);
    lip.position().set(0, mount_y - 0.20, 0.04);
    disp.add(&lip);
    // Hanging paper tongue (visible torn sheet).
    let tongue = mesh(pg("HandTowelDispensers__tongue", plane_geo!(0.20, 0.14)), &paper_mat);
    tongue.position().set(0, mount_y - 0.27, 0.075);
    disp.add(&tongue);
    // Small "PULL" arrow indicator strip, on the cover face.
    let arrow = mesh(pg("HandTowelDispensers__arrow", box_geo!(0.06, 0.010, 0.005)), lambert!({ color: 0x202020 }));
    arrow.position().set(0, mount_y + 0.10, 0.0725);
    disp.add(&arrow);
    disp.rotation().y = face_wall_yaw(c.dx, c.dz);
    disp.position().set(g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.070), 0, g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.070));
    g.world.add(&disp);
}
