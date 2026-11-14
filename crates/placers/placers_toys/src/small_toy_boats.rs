//! `place_small_toy_boats`
use crate::shared::tri_geo;
use game::prelude::*;
use placers_kit::place::pick_color;
use placers_kit::place::pick_open;
use placers_kit::place::scatter_pos;

// Small toy boat: 8% chance per level. Small wooden boat
// with mast + sail. Child-toy family.
pub(crate) fn place_small_toy_boats(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let Some((x, z)) = pick_open(g) else { return };
    let hull_colors = [0x8a5030, 0x603820, 0x2050d8, 0xd82040, 0xf0ece0];
    let hull_mat = lambert!({ color: pick_color(&hull_colors) });
    let mast_mat = lambert!({ color: 0x2a2418 });
    let sail_colors = [0xf0ece0, 0xd82040, 0x2050d8, 0xd8b040];
    let sail_mat = lambert!({ color: pick_color(&sail_colors), side: DoubleSide });
    let boat = group();
    // Hull: trapezoid box (wider top).
    let hull = mesh(pg("SmallToyBoats__hull", box_geo!(0.12, 0.030, 0.045)), &hull_mat);
    hull.position().y = 0.015;
    boat.add(&hull);
    // Prow: cone at front.
    let prow = mesh(pg("SmallToyBoats__bow", cone_geo!(0.022, 0.04, 4)), &hull_mat);
    prow.rotation().z = -PI / 2.0;
    prow.position().set(0.070, 0.015, 0);
    boat.add(&prow);
    // Deck plane (lighter color).
    let deck = mesh(pg("SmallToyBoats__deck", plane_geo!(0.10, 0.040)), &hull_mat);
    deck.rotation().x = -PI / 2.0;
    deck.position().y = 0.031;
    boat.add(&deck);
    // Mast.
    let mast = mesh(pg("SmallToyBoats__mast", cyl_geo!(0.003, 0.003, 0.10, 5)), &mast_mat);
    mast.position().set(0, 0.080, 0);
    boat.add(&mast);
    // Triangular sail.
    let sail_geo = tri_geo(&[0.0, 0.032, 0.0, 0.0, 0.130, 0.0, 0.055, 0.032, 0.0], vec![0, 1, 2]);
    let sail = mesh(sail_geo, &sail_mat);
    boat.add(&sail);
    scatter_pos(g, &boat, x, z, 2.4);
    boat.rotation().y = rnd() * PI * 2.0;
    // 25% chance tipped on side: rolled about the keel (x) onto the hull side,
    // mast along the floor (YXZ: roll about the boat's own axis, then yaw).
    if rnd() < 0.25 {
        boat.set_rotation_order(EulerOrder::YXZ);
        boat.rotation().x = PI / 2.0;
        boat.position().y = 0.0225;
    }
    g.world.add(&boat);
}
