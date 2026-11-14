//! `place_mouse_traps`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::scatter_on_floor;

// Wooden mouse trap: 7% chance per level. Wood base + spring bar + cheese.
pub(crate) fn place_mouse_traps(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.93 {
        return;
    } // 7% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let wood_mat = lambert!({ color: 0xa08050 });
    let wire_mat = lambert!({ color: 0xc0c0c0 });
    let cheese_mat = lambert!({ color: 0xf0d040 });
    let trap = group();
    // Base plank.
    let base = mesh(pg("MouseTraps__base", box_geo!(0.06, 0.006, 0.038)), &wood_mat);
    base.position().y = 0.003;
    trap.add(&base);
    // Spring bar (U-shape wire).
    let spring_loop = mesh(pg("MouseTraps__snap", torus_geo!(0.016, 0.001, 4, 8)), &wire_mat);
    spring_loop.position().set(-0.015, 0.010, 0);
    spring_loop.rotation().x = PI / 2.0;
    trap.add(&spring_loop);
    // Bar wire across base.
    let bar = mesh(pg("MouseTraps__bar", cyl_geo!(0.001, 0.001, 0.036, 4)), &wire_mat);
    bar.rotation().x = PI / 2.0;
    bar.position().set(0.010, 0.007, 0);
    trap.add(&bar);
    // Trigger tab.
    let trigger = mesh(pg("MouseTraps__trigger", box_geo!(0.006, 0.002, 0.014)), &wire_mat);
    trigger.position().set(0.020, 0.008, 0);
    trap.add(&trigger);
    // Cheese bait cube.
    let cheese = mesh(pg("MouseTraps__cheese", box_geo!(0.006, 0.005, 0.005)), &cheese_mat);
    cheese.position().set(0.020, 0.009, 0);
    trap.add(&cheese);
    scatter_on_floor(g, &trap, x, z, 2.4);
    g.world.add(&trap);
}
