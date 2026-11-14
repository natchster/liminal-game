//! `place_crushed_water_bottles`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::scatter_y;

// Crushed water bottle: 10% chance per level. Deformed
// clear plastic w/ label.
pub(crate) fn place_crushed_water_bottles(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = choice(&open);
    let bottle_mat = basic_mat!({ color: 0xc0d8e0, transparent: true, opacity: 0.55 });
    let cap_mat = lambert!({ color: 0x2050c0 });
    let label_mat = lambert!({ color: 0xf0ece0 });
    let bottle = group();
    // Body (partially crushed).
    let body = mesh(cyl_geo!(0.020, 0.024, 0.06, 8), &bottle_mat);
    body.scale().set(1, 1, 0.5);
    body.rotation().z = PI / 2.0;
    body.position().set(0, 0.020, 0);
    bottle.add(&body);
    // Cap.
    let cap = mesh(pg("CrushedWaterBottles__cap", cyl_geo!(0.010, 0.010, 0.008, 8)), &cap_mat);
    // on the neck: the narrow (r 0.020) end is at −X; closed top (local −Y) faces out
    cap.rotation().z = -PI / 2.0;
    cap.position().set(-0.034, 0.020, 0);
    bottle.add(&cap);
    // Label band.
    let label = mesh(pg("CrushedWaterBottles__label", cyl_geo!(0.021, 0.021, 0.030, 8, 1, true)), &label_mat);
    label.rotation().z = PI / 2.0;
    label.position().set(-0.006, 0.020, 0);
    // wrap the outside of the tapered body (r up to 0.0226 under the label), not inside it
    label.scale().set(1.08, 1, 0.5 * 1.08);
    bottle.add(&label);
    // the wide end (r 0.024 about y 0.020) rests on the floor
    scatter_y(g, &bottle, x, z, 2.4, 0.004);
    g.world.add(&bottle);
}
