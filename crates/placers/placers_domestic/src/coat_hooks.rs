//! `place_coat_hooks`
use crate::shared::buffer_geo;
use game::prelude::*;
use placers_kit::place::mount_on_wall;
use placers_kit::place::pick_hex;
use placers_kit::place::wall_candidates;

// Wall coat hooks with robe: 5% chance per level. 3 hooks + optional robe.
pub(crate) fn place_coat_hooks(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = candidates[(rnd() * candidates.len() as f64).floor() as usize];
    let board_mat = lambert!({ color: 0x603820 });
    let hook_mat = lambert!({ color: 0x606870 });
    let robe_colors = [0x2a2418, 0x502010, 0x201a10, 0x8a4030, 0x304050];
    let hook_group = group();
    // Wood board.
    let board = mesh(pg("CoatHooks__board", box_geo!(0.30, 0.05, 0.014)), &board_mat);
    board.position().y = 0.025;
    hook_group.add(&board);
    // 3 hook posts extending forward.
    for i in 0..3 {
        let hx = -0.10 + i as f64 * 0.10;
        // Base post.
        let post = mesh(pg("CoatHooks__post", cyl_geo!(0.004, 0.004, 0.030, 4)), &hook_mat);
        post.rotation().x = PI / 2.0;
        post.position().set(hx, 0.025, 0.020);
        hook_group.add(&post);
        // Hook tip (small angled), rising from the post's front end.
        let tip = mesh(pg("CoatHooks__tip", cyl_geo!(0.003, 0.003, 0.014, 4)), &hook_mat);
        tip.position().set(hx, 0.0315, 0.0357);
        tip.rotation().x = 0.4;
        hook_group.add(&tip);
    }
    // Optional robe on middle hook (70% chance).
    if rnd() < 0.70 {
        let robe_mat = lambert!({ color: pick_hex(&robe_colors), side: DoubleSide });
        // Robe body: trapezoid plane.
        let robe_geo = buffer_geo(&[-0.06, 0.0, 0.0, 0.06, 0.0, 0.0, 0.10, -0.35, 0.0, -0.10, -0.35, 0.0], vec![0, 1, 2, 0, 2, 3]);
        let robe = mesh(robe_geo, &robe_mat);
        robe.position().set(0, 0.020, 0.035);
        hook_group.add(&robe);
    }
    mount_on_wall(g, &hook_group, &c, 0.007, 2.10, 0.6);
    g.world.add(&hook_group);
}
