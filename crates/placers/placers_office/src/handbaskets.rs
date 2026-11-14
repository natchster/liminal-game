//! `place_handbaskets`: Small handbasket: 10% chance per level. Shallow open box + 2 arched handles. Reads as shopping-style handbasket.
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

/// Small handbasket: 10% chance per level. Shallow open box +
/// 2 arched handles. Reads as shopping-style handbasket.
pub(crate) fn place_handbaskets(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let body_colors = [0x8a2020, 0x2050a0, 0x686050, 0x2a2a2a];
    let body_mat = lambert!({ color: pick(&body_colors) });
    let inside_mat = basic_mat!({ color: 0x0a0806 });
    let handle_mat = lambert!({ color: 0x1a1610 });
    let basket = group();
    // Body: shallow open box.
    let body = mesh(pg("Handbaskets__body", box_geo!(0.22, 0.12, 0.16)), &body_mat);
    body.position().y = 0.06;
    basket.add(&body);
    // Inside dark disc (open top).
    let inside = mesh(plane_geo!(0.20, 0.14), &inside_mat);
    inside.rotation().x = -PI / 2.0;
    inside.position().y = 0.121;
    basket.add(&inside);
    // 2 arched handles across top: each half-ring (in XY, arching up) stands
    // on the rim of a long wall (z ±0.075), pivot hubs on the rim.
    for hx_off in [-0.09, 0.09] {
        let handle = mesh(pg("Handbaskets__handle", torus_geo!(0.05, 0.006, 4, 8, PI)), &handle_mat);
        handle.position().set(0, 0.118, hx_off * (0.075 / 0.09));
        basket.add(&handle);
    }
    basket.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    basket.rotation().y = rnd() * PI * 2.0;
    g.world.add(&basket);
}
