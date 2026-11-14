//! `place_switch_plates`
use game::prelude::*;
use placers_kit::place::face_wall;
use placers_kit::place::pick;
use placers_kit::place::wall_tiles_first;

// Dead light switch plate: 7% chance per level (does nothing).
pub(crate) fn place_switch_plates(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.93 {
        return;
    } // 7% chance
    let wall_tiles = wall_tiles_first(g);
    if wall_tiles.is_empty() {
        return;
    }
    let c = pick(&wall_tiles);
    let plate_mat = lambert!({ color: 0xe8e4d8 });
    let toggle_mat = lambert!({ color: 0xf8f4e8 });
    let screw_mat = lambert!({ color: 0x808080 });
    let sw = group();
    let mount_y = 1.25;
    // Rectangular face plate.
    let plate = mesh(pg("SwitchPlates__plate", box_geo!(0.075, 0.115, 0.008)), &plate_mat);
    plate.position().y = mount_y;
    sw.add(&plate);
    // Toggle switch (random up/down orientation).
    let flipped = rnd() < 0.5;
    let toggle = mesh(pg("SwitchPlates__toggle", box_geo!(0.010, 0.024, 0.014)), &toggle_mat);
    toggle.position().set(0, mount_y, 0.010);
    toggle.rotation().x = if flipped { 0.35 } else { -0.35 };
    sw.add(&toggle);
    // 2 screws (top + bottom): head's underside on the plate face, shank in the plate.
    for i in 0..2 {
        let screw = mesh(pg("SwitchPlates__screw", cyl_geo!(0.003, 0.003, 0.006, 6)), &screw_mat);
        screw.rotation().x = PI / 2.0;
        screw.position().set(0, mount_y + if i == 0 { 0.048 } else { -0.048 }, 0.0028);
        sw.add(&screw);
    }
    sw.rotation().y = face_wall(c.dx, c.dz);
    sw.position().set(g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.005), 0, g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.005));
    g.world.add(&sw);
}
