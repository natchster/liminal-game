//! `place_small_hairbrushes`
use game::prelude::*;
use placers_kit::place::pick_color;
use placers_kit::place::pick_open;
use placers_kit::place::scatter_pos;

// Small hairbrush: 10% chance per level. Oval brush pad
// with bristle bumps + handle. Dressing-table family.
pub(crate) fn place_small_hairbrushes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let Some((x, z)) = pick_open(g) else { return };
    let handle_colors = [0xd050a0, 0x2050d8, 0x201a10, 0xd8b040, 0x8a3820];
    let handle_mat = lambert!({ color: pick_color(&handle_colors) });
    let bristle_mat = lambert!({ color: 0x2a2418 });
    let brush = group();
    // Pad: oval box (scaled cube).
    let pad = mesh(pg("SmallHairbrushes__pad", box_geo!(0.055, 0.012, 0.035)), &handle_mat);
    pad.position().set(0.030, 0.006, 0);
    brush.add(&pad);
    // 5×3 bristle bumps on top of pad.
    for bx in 0..5 {
        for bz in 0..3 {
            let bristle = mesh(pg("SmallHairbrushes__bristle", cyl_geo!(0.002, 0.002, 0.010, 4)), &bristle_mat);
            bristle.position().set(0.030 - 0.020 + bx as f64 * 0.010, 0.017, -0.010 + bz as f64 * 0.010);
            brush.add(&bristle);
        }
    }
    // Handle: tapered cylinder extending sideways.
    let handle = mesh(pg("SmallHairbrushes__handle", cyl_geo!(0.008, 0.012, 0.10, 8)), &handle_mat);
    handle.rotation().z = PI / 2.0;
    handle.position().set(-0.046, 0.012, 0); // flared end 1.5 mm into the pad, lying on the floor
    brush.add(&handle);
    scatter_pos(g, &brush, x, z, 2.4);
    brush.rotation().y = rnd() * PI * 2.0;
    g.world.add(&brush);
}
