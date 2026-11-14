//! `place_hair_ties`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Hair tie band: 10% chance per level.
pub(crate) fn place_hair_ties(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = pick(&open_tiles);
    let tie_colors = [0x202020, 0xe04040, 0x4080e0, 0x40c060, 0xe0c040, 0xd050a0];
    let tie_mat = lambert!({ color: pick(&tie_colors) });
    // Squashed torus lying flat.
    let tie = mesh(pg("HairTies__tie", torus_geo!(0.020, 0.004, 5, 14)), &tie_mat);
    tie.rotation().x = PI / 2.0;
    tie.scale().y = 0.5; // slightly squashed
    tie.position().set(g.wx(x) + (rnd() - 0.5) * 2.5, 0.004, g.wz(z) + (rnd() - 0.5) * 2.5);
    tie.rotation().z = rnd() * PI;
    g.world.add(&tie);
}
