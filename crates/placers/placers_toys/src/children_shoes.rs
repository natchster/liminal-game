//! `place_children_shoes`: Small children's shoe: 10% chance per level. Small shoe silhouette (elongated body + toe bump + sole).
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

/// Small children's shoe: 10% chance per level. Small shoe
/// silhouette (elongated body + toe bump + sole).
pub(crate) fn place_children_shoes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let shoe_colors = [0xd82020, 0x2050d8, 0xf0f0f0, 0x181818, 0xd8b820];
    let shoe_mat = lambert!({ color: pick(&shoe_colors) });
    let sole_mat = lambert!({ color: 0x181410 });
    let shoe = group();
    // Body: elongated box.
    let body = mesh(pg("ChildrenShoes__upper", box_geo!(0.14, 0.05, 0.06)), &shoe_mat);
    body.position().y = 0.025;
    shoe.add(&body);
    // Toe bump: rounded box at front.
    let toe = mesh(pg("ChildrenShoes__toe", sphere_geo!(0.03, 6, 4)), &shoe_mat);
    toe.scale().set(1, 0.7, 1);
    toe.position().set(0.04, 0.025, 0); // ends flush with the sole's front (x 0.07)
    shoe.add(&toe);
    // Sole: thin dark box at bottom.
    let sole = mesh(pg("ChildrenShoes__sole", box_geo!(0.14, 0.008, 0.06)), &sole_mat);
    sole.position().y = 0.004;
    shoe.add(&sole);
    shoe.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    shoe.rotation().y = rnd() * PI * 2.0;
    g.world.add(&shoe);
}
