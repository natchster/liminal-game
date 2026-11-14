//! `place_rusted_nails`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick_hex;
use placers_kit::place::scatter;

// Rusted nails: 10% chance per level. 6-12 rusted nails scattered on floor.
pub(crate) fn place_rusted_nails(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let rust_colors = [0x6a3820, 0x504028, 0x8a4030];
    let grp = group();
    let n_nails = 6 + (rnd() * 7.0).floor() as i32;
    for _i in 0..n_nails {
        let nail_mat = lambert!({ color: pick_hex(&rust_colors) });
        let nail = group();
        // Shaft.
        let shaft = mesh(pg("RustedNails__shaft", cyl_geo!(0.001, 0.001, 0.032, 4)), &nail_mat);
        shaft.rotation().z = PI / 2.0;
        shaft.position().set(0.010, 0.001, 0);
        nail.add(&shaft);
        // Head.
        let head = mesh(pg("RustedNails__head", cyl_geo!(0.003, 0.003, 0.002, 6)), &nail_mat);
        head.rotation().z = PI / 2.0;
        head.position().set(-0.007, 0.001, 0);
        nail.add(&head);
        // Tip point.
        let tip = mesh(cone_geo!(0.001, 0.005, 4), &nail_mat);
        tip.rotation().z = -PI / 2.0;
        tip.position().set(0.028, 0.001, 0);
        nail.add(&tip);
        // rests on its head rim and point: tilt the shank down toward the tip
        nail.position().set((rnd() - 0.5) * 0.25, 0.0016, (rnd() - 0.5) * 0.20);
        nail.rotation().z = -0.055;
        nail.rotation().y = rnd() * PI * 2.0;
        grp.add(&nail);
    }
    scatter(g, &grp, x, z, 2.4);
    g.world.add(&grp);
}
