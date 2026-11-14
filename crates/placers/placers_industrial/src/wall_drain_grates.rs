//! `place_wall_drain_grates`
use game::prelude::*;
use placers_kit::place::mount;
use placers_kit::place::wall_candidates;
use placers_kit::place::wall_xz;

// Wall drain grate: 5% chance per level. Rectangular grate
// w/ 5 horizontal slots + dark cavity.
pub(crate) fn place_wall_drain_grates(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = choice(&candidates);
    let frame_mat = lambert!({ color: 0x707078 });
    let dark_mat = basic_mat!({ color: 0x080604 });
    let grate = group();
    // Dark cavity behind.
    let cavity = mesh(plane_geo!(0.10, 0.14), &dark_mat);
    cavity.position().z = 0.001;
    grate.add(&cavity);
    // Frame.
    let fw = 0.006;
    for (bar_w, bar_h, bx, by) in [
        (0.10 + fw * 2.0, fw, 0.0, 0.07 + fw / 2.0),
        (0.10 + fw * 2.0, fw, 0.0, -0.07 - fw / 2.0),
        (fw, 0.14, -0.05 - fw / 2.0, 0.0),
        (fw, 0.14, 0.05 + fw / 2.0, 0.0),
    ] {
        let bar = mesh(box_geo!(bar_w, bar_h, 0.004), &frame_mat);
        bar.position().set(bx, by, 0.002);
        grate.add(&bar);
    }
    // 5 horizontal slot bars.
    for i in 0..5 {
        let slot = mesh(pg("WallDrainGrates__slot", box_geo!(0.08, 0.004, 0.005)), &frame_mat);
        slot.position().set(0, -0.04 + i as f64 * 0.02, 0.003);
        grate.add(&slot);
    }
    let (wall_x, wall_z) = wall_xz(g, c, 0.005);
    mount(&grate, c, wall_x, wall_z, 0.25, Some(0.6)); // Near floor.
    g.world.add(&grate);
}
