//! `place_rusted_hammers`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::scatter;

// Rusted hammer: 8% chance per level. Metal head + wood
// handle on floor.
pub(crate) fn place_rusted_hammers(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = choice(&open);
    let rust_mat = lambert!({ color: 0x6a4028 });
    let handle_mat = lambert!({ color: 0x8a5030 });
    let hammer = group();
    // Handle.
    let handle = mesh(pg("RustedHammers__handle", cyl_geo!(0.008, 0.010, 0.20, 6)), &handle_mat);
    handle.rotation().z = PI / 2.0;
    handle.position().set(0, 0.010, 0);
    hammer.add(&handle);
    // Head (rectangular block).
    let head = mesh(pg("RustedHammers__head", box_geo!(0.030, 0.020, 0.028)), &rust_mat);
    head.position().set(0.10, 0.014, 0);
    hammer.add(&head);
    // Claw prongs (2 curved forward extensions).
    for i in 0..2 {
        let claw = mesh(pg("RustedHammers__claw", box_geo!(0.008, 0.020, 0.008)), &rust_mat);
        // on the back of the head (+z, opposite the face), prongs pointing out
        // and curling back toward the handle; split vertically (hammer on its cheek)
        claw.position().set(0.10, 0.014 + (if i == 0 { -1.0 } else { 1.0 }) * 0.004, 0.022);
        claw.rotation().x = -PI / 2.0;
        claw.rotation().z = PI;
        hammer.add(&claw);
    }
    scatter(g, &hammer, x, z, 2.4);
    g.world.add(&hammer);
}
