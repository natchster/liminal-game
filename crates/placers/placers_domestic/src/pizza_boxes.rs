//! `place_pizza_boxes`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::scatter;

// Pizza box: 7% chance per level. Empty square box w/
// grease stain interior.
pub(crate) fn place_pizza_boxes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.93 {
        return;
    } // 7% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = choice(&open);
    let box_mat = lambert!({ color: 0xd8b070 });
    let grease_mat = lambert!({ color: 0x8a5030 });
    let bx = group();
    let bs = 0.28;
    // Body.
    let body = mesh(pg("PizzaBoxes__body", box_geo!(bs, 0.024, bs)), &box_mat);
    body.position().y = 0.012;
    bx.add(&body);
    // Grease stain on top.
    let grease = mesh(circle_geo!(0.10, 12), &grease_mat);
    grease.rotation().x = -PI / 2.0;
    grease.position().y = 0.025;
    bx.add(&grease);
    // 40% chance open (lid tilted back).
    if rnd() < 0.40 {
        let lid = mesh(pg("PizzaBoxes__lid", box_geo!(bs, 0.008, bs)), &box_mat);
        // Hinged on the body's top back edge (y 0.024, z -bs/2), flopped back 120 degrees.
        lid.position().set(0, 0.143, -bs / 2.0 - 0.0735);
        lid.rotation().x = -2.0 * PI / 3.0;
        bx.add(&lid);
    }
    scatter(g, &bx, x, z, 1.8);
    g.world.add(&bx);
}
