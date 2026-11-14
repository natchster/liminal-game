//! `place_unrolled_tp_rolls`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::scatter;

// Unrolled TP: 8% chance per level. Cardboard tube +
// unrolled paper trail.
pub(crate) fn place_unrolled_tp_rolls(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = choice(&open);
    let tube_mat = lambert!({ color: 0x8a5030 });
    let paper_mat = lambert!({ color: 0xf0ece0, side: DoubleSide });
    let tp = group();
    // Cardboard tube.
    let tube = mesh(pg("UnrolledTPRolls__tube", cyl_geo!(0.024, 0.024, 0.10, 10)), &tube_mat);
    tube.rotation().z = PI / 2.0;
    tube.position().y = 0.024;
    tp.add(&tube);
    // Unrolled paper trail (bent plane strip).
    let trail = mesh(plane_geo!(0.30, 0.10), &paper_mat);
    trail.rotation().x = -PI / 2.0;
    trail.position().set(0.20, 0.001, 0);
    tp.add(&trail);
    // Small second trail patch further out.
    let trail2 = mesh(plane_geo!(0.20, 0.10), &paper_mat);
    trail2.rotation().x = -PI / 2.0;
    trail2.position().set(0.45, 0.001, 0.03);
    trail2.rotation().z = 0.2;
    tp.add(&trail2);
    scatter(g, &tp, x, z, 2.0);
    g.world.add(&tp);
}
