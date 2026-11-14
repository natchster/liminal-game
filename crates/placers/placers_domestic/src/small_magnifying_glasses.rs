//! `place_small_magnifying_glasses`
use game::prelude::*;
use placers_kit::place::pick_color;
use placers_kit::place::pick_open_tile;
use placers_kit::place::scatter_pos;

// Small magnifying glass: 8% chance per level. Handle +
// lens ring + tinted lens disc. Detective/kid family.
pub(crate) fn place_small_magnifying_glasses(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let Some((x, z)) = pick_open_tile(g) else { return };
    let handle_colors = [0x8a5030, 0x201a10, 0xd8b040, 0x181008];
    let handle_mat = lambert!({ color: pick_color(&handle_colors) });
    let ring_mat = lambert!({ color: 0xd8b040 });
    let lens_mat = basic_mat!({ color: 0xd0e8f0, transparent: true, opacity: 0.35 });
    let glass = group();
    let lens_r = 0.040;
    // Lens ring (torus).
    let ring = mesh(pg("SmallMagnifyingGlasses__rim", torus_geo!(lens_r, 0.005, 6, 16)), &ring_mat);
    ring.rotation().x = PI / 2.0;
    ring.position().y = 0.006;
    glass.add(&ring);
    // Lens disc (tinted glass).
    let lens = mesh(circle_geo!(lens_r - 0.002, 16), &lens_mat);
    lens.rotation().x = -PI / 2.0;
    lens.position().y = 0.007;
    glass.add(&lens);
    // Handle: extending sideways from lens ring.
    let handle_len = 0.10;
    let handle = mesh(pg("SmallMagnifyingGlasses__handle", cyl_geo!(0.008, 0.010, handle_len, 8)), &handle_mat);
    handle.rotation().z = PI / 2.0;
    handle.position().set(lens_r + handle_len / 2.0, 0.006, 0);
    glass.add(&handle);
    // End knob.
    let knob = mesh(pg("SmallMagnifyingGlasses__knob", sphere_geo!(0.012, 8, 6)), &handle_mat);
    knob.position().set(lens_r + handle_len + 0.005, 0.006, 0);
    glass.add(&knob);
    scatter_pos(g, &glass, x, z, 2.4);
    glass.rotation().y = rnd() * PI * 2.0;
    g.world.add(&glass);
}
