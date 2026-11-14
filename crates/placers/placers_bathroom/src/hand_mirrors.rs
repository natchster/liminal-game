//! `place_hand_mirrors`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Hand mirror: 6% chance per level. Round mirror w/ long handle laid flat
// on floor.
pub(crate) fn place_hand_mirrors(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let frame_colors: [u32; 5] = [0xd0a040, 0x606060, 0x804060, 0xc02020, 0x101010];
    let frame_mat = lambert!({ color: pick(&frame_colors) });
    let glass_mat = lambert!({ color: 0xe0e8f0 });
    let mirror = group();
    // Round frame disc.
    let frame = mesh(pg("HandMirrors__frame", cyl_geo!(0.030, 0.030, 0.005, 12)), &frame_mat);
    frame.position().y = 0.003;
    mirror.add(&frame);
    // Glass surface (slightly smaller silver disc on top).
    let glass = mesh(circle_geo!(0.026, 12), &glass_mat);
    glass.rotation().x = -PI / 2.0;
    glass.position().y = 0.006;
    mirror.add(&glass);
    // Handle (long thin rectangle extending).
    let handle = mesh(pg("HandMirrors__handle", box_geo!(0.008, 0.006, 0.070)), &frame_mat);
    handle.position().set(0, 0.003, 0.060);
    mirror.add(&handle);
    // Small grip knob at end.
    let knob = mesh(pg("HandMirrors__knob", sphere_geo!(0.008, 6, 5)), &frame_mat);
    knob.scale().set_scalar(0.625); // r 5 mm: a finial just wider than the 8 mm handle, resting on the floor
    knob.position().set(0, 0.005, 0.098);
    mirror.add(&knob);
    mirror.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    mirror.rotation().y = rnd() * PI * 2.0;
    g.world.add(&mirror);
}
