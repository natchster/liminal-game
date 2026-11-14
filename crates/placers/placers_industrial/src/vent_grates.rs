//! `place_vent_grates`
use game::prelude::*;
use placers_kit::place::face_wall;
use placers_kit::place::pick;
use placers_kit::place::wall_tiles_first;

// Bathroom vent grate: 5% chance per level, high on the wall.
pub(crate) fn place_vent_grates(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let wall_tiles = wall_tiles_first(g);
    if wall_tiles.is_empty() {
        return;
    }
    let c = pick(&wall_tiles);
    let frame_mat = lambert!({ color: 0x707070 });
    let shadow_mat = lambert!({ color: 0x101010 });
    let vent = group();
    let mount_y = 2.20; // High on wall.
                        // Outer frame rectangle.
    let frame = mesh(pg("VentGrates__frame", box_geo!(0.30, 0.20, 0.010)), &frame_mat);
    frame.position().y = mount_y;
    vent.add(&frame);
    // Dark inset (shadow behind louvers).
    let inset = mesh(box_geo!(0.26, 0.16, 0.005), &shadow_mat);
    inset.position().set(0, mount_y, 0.006);
    vent.add(&inset);
    // 6 horizontal louvers.
    for l in 0..6 {
        let louver = mesh(pg("VentGrates__louver", box_geo!(0.26, 0.014, 0.008)), &frame_mat);
        louver.position().set(0, mount_y - 0.07 + l as f64 * 0.028, 0.010);
        louver.rotation().x = -0.4; // Angled downward.
        vent.add(&louver);
    }
    vent.rotation().y = face_wall(c.dx, c.dz);
    vent.position().set(g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.006), 0, g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.006));
    g.world.add(&vent);
}
