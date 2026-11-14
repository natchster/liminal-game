//! `place_cheese_wedges`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Cheese wheel wedge: 7% chance per level.
pub(crate) fn place_cheese_wedges(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.93 {
        return;
    } // 7% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let cheese_colors: [u32; 4] = [0xf0d040, 0xd0a020, 0xe8c060, 0xf0e090];
    let cheese_mat = lambert!({ color: pick(&cheese_colors) });
    let rind_mat = lambert!({ color: 0xa08020 });
    let wedge = group();
    let body = mesh(pg("CheeseWedges__body", cyl_geo!(0.055, 0.055, 0.040, 3, 1, false, 0, PI * 0.5)), &cheese_mat);
    // Lying on its triangular side (quarter disc flat, x/z 0..0.055), rind lips
    // resting on the floor; was stood on its rind arc, half under the floor.
    body.position().y = 0.0221;
    wedge.add(&body);
    let rind = mesh(pg("CheeseWedges__rind", cyl_geo!(0.056, 0.056, 0.042, 8, 1, false, 0, PI * 0.5)), &rind_mat);
    rind.position().y = 0.0221;
    rind.scale().y = 1.05;
    wedge.add(&rind);
    for i in 0..3 {
        let hole = mesh(circle_geo!(0.005 + rnd() * 0.004, 6), lambert!({ color: 0xa08020 }));
        hole.rotation().x = -PI / 2.0;
        // on the wedge's top face (0.0421), inside the quarter disc
        hole.position().set(0.010 + i as f64 * 0.013, 0.0426, 0.010 + rnd() * 0.012);
        wedge.add(&hole);
    }
    wedge.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    wedge.rotation().y = rnd() * PI * 2.0;
    g.world.add(&wedge);
}
