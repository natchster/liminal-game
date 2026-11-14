//! `place_stall_walls`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Bathroom stall wall: 3% chance per level, free-standing partition.
pub(crate) fn place_stall_walls(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    } // 3% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = pick(&open_tiles);
    let stall_colors = [0xa0a0a8, 0x6a7080, 0x808070, 0x505060];
    let stall_mat = lambert!({ color: pick(&stall_colors), side: DoubleSide });
    let stall = group();
    // Main panel: 1.4 m wide, 1.7 m tall, 15 cm floor gap.
    let panel = mesh(box_geo!(1.40, 1.70, 0.020), &stall_mat);
    panel.position().y = 1.00; // Bottom at 15 cm.
    stall.add(&panel);
    // 2 small support feet.
    let foot_mat = lambert!({ color: 0x404040 });
    for i in 0..2 {
        let foot = mesh(pg("StallWalls__foot", box_geo!(0.10, 0.15, 0.08)), &foot_mat);
        foot.position().set(if i == 0 { -0.55 } else { 0.55 }, 0.075, 0);
        stall.add(&foot);
    }
    stall.rotation().y = rnd() * PI * 2.0;
    stall.position().set(g.wx(x) + (rnd() - 0.5) * 1.2, 0, g.wz(z) + (rnd() - 0.5) * 1.2);
    g.world.add(&stall);
}
