//! `place_wall_flashlights`
use game::prelude::*;
use placers_kit::place::mount_slid_ref as wall_mount;
use placers_kit::place::ri as pick;
use placers_kit::place::wall_candidates;

// Wall emergency flashlight: 12% chance per level. Small dark
// bracket + flashlight cylinder held horizontally on wall.
pub(crate) fn place_wall_flashlights(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.88 {
        return;
    } // 12% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = candidates[pick(candidates.len())];
    let bracket_mat = lambert!({ color: 0x2a2418 });
    let flash_mat = lambert!({ color: 0x181820 });
    let lens_glass = lambert!({ color: 0xe8e8c0 });
    let fl = group();
    // Bracket: small dark box against wall.
    let bracket = mesh(pg("WallFlashlights__bracket", box_geo!(0.06, 0.06, 0.015)), &bracket_mat);
    bracket.position().z = 0.008;
    fl.add(&bracket);
    // Flashlight body: horizontal cylinder held by bracket.
    let fl_body = mesh(pg("WallFlashlights__barrel", cyl_geo!(0.025, 0.025, 0.16, 8)), &flash_mat);
    fl_body.rotation().z = PI / 2.0;
    fl_body.position().z = 0.040;
    fl.add(&fl_body);
    // Head: slightly wider cylinder at front tip.
    let fl_head = mesh(pg("WallFlashlights__head", cyl_geo!(0.030, 0.028, 0.03, 8)), &flash_mat);
    fl_head.rotation().z = PI / 2.0;
    fl_head.position().set(0.088, 0, 0.040);
    fl.add(&fl_head);
    // Lens glass at tip.
    let fl_lens = mesh(circle_geo!(0.025, 10), &lens_glass);
    fl_lens.rotation().y = PI / 2.0;
    fl_lens.position().set(0.104, 0, 0.040);
    fl.add(&fl_lens);
    let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.005);
    let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.005);
    let fl_height = 1.6 + rnd() * 0.15;
    wall_mount(&fl, &c, wall_x, wall_z, fl_height, 1.0);
    g.world.add(&fl);
}
