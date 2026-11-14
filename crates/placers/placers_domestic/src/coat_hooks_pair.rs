//! `place_coat_hooks_pair`
use game::prelude::*;
use placers_kit::place::face_yaw;
use placers_kit::place::pick;
use placers_kit::place::wall_tiles;

// Coat hooks pair: 6% chance per level. Small backboard w/ 2 J-hooks.
pub(crate) fn place_coat_hooks_pair(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let walls = wall_tiles(g);
    if walls.is_empty() {
        return;
    }
    let (x, z, dx, dz) = pick(&walls);
    let board_mat = lambert!({ color: 0x8b5a2b });
    let hook_mat = lambert!({ color: 0xa08040 });
    let pair = group();
    let mount_y = 1.60;
    // Small board.
    let board = mesh(pg("CoatHooksPair__board", box_geo!(0.16, 0.05, 0.015)), &board_mat);
    board.position().y = mount_y;
    pair.add(&board);
    // 2 J-hooks.
    for i in 0..2 {
        let hook_x = -0.040 + i as f64 * 0.080;
        let base = mesh(pg("CoatHooksPair__rosette", cyl_geo!(0.008, 0.008, 0.006, 8)), &hook_mat);
        base.rotation().x = PI / 2.0;
        base.position().set(hook_x, mount_y, 0.010);
        pair.add(&base);
        let arm = mesh(pg("CoatHooksPair__arm", cyl_geo!(0.005, 0.005, 0.040, 6)), &hook_mat);
        arm.rotation().x = PI / 2.0;
        arm.position().set(hook_x, mount_y, 0.030);
        pair.add(&arm);
        let curve = mesh(pg("CoatHooksPair__curve", torus_geo!(0.012, 0.004, 3, 8, PI)), &hook_mat);
        // U open upward (a J): one end on the arm tip, free ball-tip end out in the room.
        curve.rotation().y = PI / 2.0;
        curve.rotation().z = PI;
        curve.position().set(hook_x, mount_y, 0.062);
        pair.add(&curve);
    }
    pair.rotation().y = face_yaw(dx, dz);
    pair.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - 0.010), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - 0.010));
    g.world.add(&pair);
}
