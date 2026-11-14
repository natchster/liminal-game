//! `place_mixing_bowls`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Ceramic mixing bowl: 6% chance per level.
pub(crate) fn place_mixing_bowls(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let bowl_colors: [u32; 6] = [0xf0e8d8, 0xd8b090, 0x8080a0, 0x408090, 0xa0d0e0, 0xe0e0e0];
    let bowl_mat = lambert!({ color: pick(&bowl_colors) });
    let bowl = group();
    let body = mesh(pg("MixingBowls__body", cyl_geo!(0.075, 0.048, 0.055, 14)), &bowl_mat);
    body.position().y = 0.028;
    bowl.add(&body);
    let rim = mesh(pg("MixingBowls__rim", torus_geo!(0.075, 0.005, 4, 16)), &bowl_mat);
    rim.rotation().x = PI / 2.0;
    rim.position().y = 0.056;
    bowl.add(&rim);
    if rnd() < 0.5 {
        let dough_mat = lambert!({ color: 0xf0e0b8 });
        let dough = mesh(pg("MixingBowls__dough", sphere_geo!(0.055, 10, 6)), &dough_mat);
        dough.scale().y = 0.4;
        dough.position().y = 0.046;
        bowl.add(&dough);
    }
    bowl.position().set(g.wx(x) + (rnd() - 0.5) * 2.2, 0, g.wz(z) + (rnd() - 0.5) * 2.2);
    bowl.rotation().y = rnd() * PI * 2.0;
    g.world.add(&bowl);
}
