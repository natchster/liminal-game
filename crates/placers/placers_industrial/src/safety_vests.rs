//! `place_safety_vests`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::scatter;

// Safety vest: 8% chance per level. Folded neon vest w/
// reflective silver stripes.
pub(crate) fn place_safety_vests(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = choice(&open);
    let vest_colors = [0xe0f020, 0xe07020, 0xe02020];
    let vest_mat = lambert!({ color: choice(&vest_colors) });
    let stripe_mat = basic_mat!({ color: 0xc0c0c0 });
    let vest = group();
    // Body (folded rectangle).
    let body = mesh(pg("SafetyVests__body", box_geo!(0.20, 0.020, 0.16)), &vest_mat);
    body.position().y = 0.010;
    vest.add(&body);
    // 2 reflective horizontal stripes.
    for sz in [-0.02, 0.02] {
        let stripe = mesh(plane_geo!(0.20, 0.020), &stripe_mat);
        stripe.rotation().x = -PI / 2.0;
        stripe.position().set(0, 0.021, sz);
        vest.add(&stripe);
    }
    scatter(g, &vest, x, z, 2.0);
    g.world.add(&vest);
}
