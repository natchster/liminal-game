//! `place_backpacks`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Small backpack: 12% chance per level.
pub(crate) fn place_backpacks(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.88 {
        return;
    } // 12% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = pick(&open_tiles);
    let body_colors = [0x203040, 0x2a1810, 0x4a3018, 0x484028, 0x381818];
    let body_mat = lambert!({ color: pick(&body_colors) });
    let strap_mat = lambert!({ color: 0x181410 });
    let bp = group();
    // Body: upright box.
    let body = mesh(pg("Backpacks__body", box_geo!(0.28, 0.36, 0.16)), &body_mat);
    body.position().y = 0.18;
    bp.add(&body);
    // 2 vertical shoulder straps on back.
    for sx_off in [-0.08, 0.08] {
        let strap = mesh(pg("Backpacks__strap", plane_geo!(0.04, 0.35)), &strap_mat);
        strap.rotation().y = PI; // face out of the back (it faced into the body)
        strap.position().set(sx_off, 0.18, -0.081);
        bp.add(&strap);
    }
    // Top handle: half-torus.
    let handle = mesh(pg("Backpacks__handle", torus_geo!(0.04, 0.006, 4, 8, PI)), &strap_mat);
    handle.rotation().z = 0.0; // loop over the top, ends on it (was a sideways C)
    handle.position().y = 0.36;
    bp.add(&handle);
    // Small front pocket flap.
    let pocket = mesh(pg("Backpacks__pocket", plane_geo!(0.16, 0.10)), &strap_mat);
    pocket.position().set(0, 0.08, 0.081);
    bp.add(&pocket);
    bp.position().set(g.wx(x) + (rnd() - 0.5) * 2.2, 0, g.wz(z) + (rnd() - 0.5) * 2.2);
    bp.rotation().y = rnd() * PI * 2.0;
    // Slight tilt for organic slump.
    bp.rotation().z = (rnd() - 0.5) * 0.10;
    g.world.add(&bp);
}
