//! `place_single_child_shoes`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick_hex;
use placers_kit::place::scatter_on_floor;

// Single child shoe: 6% chance per level. One small child's shoe on floor.
pub(crate) fn place_single_child_shoes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let shoe_colors = [0xd82040, 0x2050d8, 0xd8b040, 0xf0ece0, 0x201a10];
    let shoe_mat = lambert!({ color: pick_hex(&shoe_colors) });
    let sole_mat = lambert!({ color: 0x201a10 });
    let shoe = group();
    // Sole.
    let sole = mesh(pg("SingleChildShoes__sole", box_geo!(0.10, 0.010, 0.05)), &sole_mat);
    sole.position().y = 0.005;
    shoe.add(&sole);
    // Upper toe (rounded).
    let toe = mesh(pg("SingleChildShoes__toe", sphere_geo!(0.028, 8, 6)), &shoe_mat);
    toe.scale().set(1, 0.6, 0.9);
    toe.position().set(0.02, 0.020, 0);
    shoe.add(&toe);
    // Heel.
    let heel = mesh(pg("SingleChildShoes__heel", box_geo!(0.040, 0.024, 0.048)), &shoe_mat);
    heel.position().set(-0.024, 0.022, 0);
    shoe.add(&heel);
    // 2 lace X marks on top.
    for i in 0..2 {
        for j in 0..2 {
            let lace_mat = lambert!({ color: 0xf0ece0 });
            let lace = mesh(pg("SingleChildShoes__lace", box_geo!(0.020, 0.001, 0.002)), &lace_mat);
            lace.position().set(0.01 + i as f64 * 0.010, 0.030, 0);
            lace.rotation().y = if j == 0 { 0.3 } else { -0.3 };
            shoe.add(&lace);
        }
    }
    scatter_on_floor(g, &shoe, x, z, 2.4);
    // 25% tipped on its side ('YXZ': roll about the shoe's own axis, then yaw).
    if rnd() < 0.25 {
        shoe.set_rotation_order(EulerOrder::YXZ);
        shoe.rotation().x = PI / 2.0;
        shoe.position().y = 0.025;
    }
    g.world.add(&shoe);
}
