//! `place_whisks`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Wire whisk: 7% chance per level. Handle + bulb of
// curved wire loops.
pub(crate) fn place_whisks(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.93 {
        return;
    } // 7% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let wire_mat = lambert!({ color: 0xc0c0c0 });
    let handle_colors: [u32; 4] = [0x606060, 0x8b5a2b, 0x202020, 0xc02020];
    let handle_mat = lambert!({ color: pick(&handle_colors) });
    let whisk = group();
    // Handle laid flat.
    let handle = mesh(pg("Whisks__handle", cyl_geo!(0.010, 0.010, 0.10, 8)), &handle_mat);
    handle.rotation().z = PI / 2.0;
    handle.position().set(-0.030, 0.010, 0);
    whisk.add(&handle);
    // Handle end cap.
    let cap = mesh(pg("Whisks__cap", sphere_geo!(0.011, 6, 5)), &handle_mat);
    cap.position().set(-0.084, 0.010, 0);
    whisk.add(&cap);
    // Metal ferrule between handle + wires.
    let ferrule = mesh(pg("Whisks__ferrule", cyl_geo!(0.011, 0.011, 0.010, 8)), &wire_mat);
    ferrule.rotation().z = PI / 2.0;
    ferrule.position().set(0.024, 0.010, 0);
    whisk.add(&ferrule);
    // 6 curved wire loops forming teardrop bulb.
    for i in 0..6 {
        let a = (i as f64 / 6.0) * PI * 2.0;
        let wire = mesh(pg("Whisks__wire", torus_geo!(0.020, 0.0012, 3, 10, PI)), &wire_mat);
        // Loops fanned round the handle axis (x): their ends meet at the poles,
        // the near pole (x 0.027) inside the ferrule.
        wire.rotation().x = a;
        wire.position().set(0.047, 0.010, 0);
        whisk.add(&wire);
    }
    whisk.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    whisk.rotation().y = rnd() * PI * 2.0;
    // The wire ball (r 0.021) is fatter than the handle: tip the whisk so the
    // ball and the end cap both rest on the floor.
    whisk.rotation().z = 0.078;
    whisk.position().y = 0.0076;
    g.world.add(&whisk);
}
