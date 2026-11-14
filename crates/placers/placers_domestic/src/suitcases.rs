//! `place_suitcases`: Small suitcase: 10% chance per level. Larger box + top handle + 2 side latches.
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

/// Small suitcase: 10% chance per level. Larger box + top handle
/// + 2 side latches.
pub(crate) fn place_suitcases(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let body_mat = lambert!({ color: 0x4a2010 });
    let handle_mat = lambert!({ color: 0x2a1810 });
    let latch_mat = lambert!({ color: 0xa08040 });
    let suitcase = group();
    // Body: taller than briefcase.
    let body = mesh(pg("Suitcases__body", box_geo!(0.40, 0.28, 0.14)), &body_mat);
    body.position().y = 0.14;
    suitcase.add(&body);
    // Top handle: half-torus.
    let handle = mesh(pg("Suitcases__handle", torus_geo!(0.06, 0.008, 4, 8, PI)), &handle_mat);
    handle.rotation().z = 0.0; // arch over the top, end blocks on the lid
    handle.position().y = 0.286;
    suitcase.add(&handle);
    // 2 brass latches on top edge.
    for lx_off in [-0.12, 0.12] {
        let latch = mesh(pg("Suitcases__latch", box_geo!(0.03, 0.012, 0.024)), &latch_mat);
        latch.position().set(lx_off, 0.27, 0.075);
        suitcase.add(&latch);
    }
    // Small "sticker" plate on front-right.
    let sticker = mesh(pg("Suitcases__plate", plane_geo!(0.06, 0.04)), &latch_mat);
    sticker.position().set(0.10, 0.18, 0.0715); // plate back on the front face
    suitcase.add(&sticker);
    suitcase.position().set(g.wx(x) + (rnd() - 0.5) * 2.2, 0, g.wz(z) + (rnd() - 0.5) * 2.2);
    suitcase.rotation().y = rnd() * PI * 2.0;
    g.world.add(&suitcase);
}
