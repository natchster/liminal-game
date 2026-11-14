//! `place_seedlings`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Potted seedling: 7% chance per level. Small pot w/ tiny green sprout.
pub(crate) fn place_seedlings(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.93 {
        return;
    } // 7% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let pot_mat = lambert!({ color: 0xa0603c });
    let dirt_mat = lambert!({ color: 0x2d1a0d });
    let stem_mat = lambert!({ color: 0x60a040 });
    let leaf_mat = lambert!({ color: 0x408040, side: DoubleSide });
    let seedling = group();
    // Small pot.
    let pot = mesh(pg("Seedlings__pot", cyl_geo!(0.028, 0.024, 0.030, 10)), &pot_mat);
    pot.position().y = 0.015;
    seedling.add(&pot);
    // Dirt top.
    let dirt = mesh(pg("Seedlings__dirt", circle_geo!(0.026, 10)), &dirt_mat);
    dirt.rotation().x = -PI / 2.0;
    dirt.position().y = 0.031;
    seedling.add(&dirt);
    // Tiny stem stub.
    let stem = mesh(pg("Seedlings__stem", cyl_geo!(0.0015, 0.0015, 0.024, 4)), &stem_mat);
    stem.position().y = 0.043;
    seedling.add(&stem);
    // 2 tiny cotyledons (seed leaves).
    for side in [-1.0, 1.0] {
        let leaf = mesh(circle_geo!(0.006, 5), &leaf_mat);
        leaf.rotation().x = -PI / 2.0;
        leaf.rotation().z = side * 0.6;
        leaf.position().set(side * 0.005, 0.054, 0);
        seedling.add(&leaf);
    }
    seedling.position().set(g.wx(x) + (rnd() - 0.5) * 2.2, 0, g.wz(z) + (rnd() - 0.5) * 2.2);
    seedling.rotation().y = rnd() * PI * 2.0;
    g.world.add(&seedling);
}
