//! `place_nail_clippers`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Nail clippers: 8% chance per level.
pub(crate) fn place_nail_clippers(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = pick(&open_tiles);
    let metal_mat = lambert!({ color: 0xc0c0c0 });
    let clippers = group();
    // Bottom jaw plate.
    let bottom = mesh(pg("NailClippers__bottom", box_geo!(0.055, 0.007, 0.020)), &metal_mat);
    bottom.position().y = 0.004;
    clippers.add(&bottom);
    // Top jaw plate (slightly shorter).
    let top = mesh(pg("NailClippers__top", box_geo!(0.048, 0.005, 0.018)), &metal_mat);
    top.position().set(0.002, 0.010, 0);
    clippers.add(&top);
    // Lever arm (pivots at rear, extends forward over top).
    let lever = mesh(pg("NailClippers__lever", box_geo!(0.050, 0.004, 0.008)), &metal_mat);
    lever.position().set(0, 0.016, 0);
    clippers.add(&lever);
    // Pivot rivet.
    let rivet = mesh(pg("NailClippers__rivet", cyl_geo!(0.003, 0.003, 0.018, 6)), &metal_mat);
    rivet.position().set(-0.020, 0.010, 0);
    clippers.add(&rivet);
    clippers.position().set(g.wx(x) + (rnd() - 0.5) * 2.5, 0, g.wz(z) + (rnd() - 0.5) * 2.5);
    clippers.rotation().y = rnd() * PI * 2.0;
    g.world.add(&clippers);
}
