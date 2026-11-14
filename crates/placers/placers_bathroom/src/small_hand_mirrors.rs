//! `place_small_hand_mirrors`: Small hand mirror: 8% chance per level. Round mirror on handle. Dressing-table family.
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

/// Small hand mirror: 8% chance per level. Round mirror on
/// handle. Dressing-table family.
pub(crate) fn place_small_hand_mirrors(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let frame_colors = [0xd8b040, 0xf0d0d0, 0x8a3820, 0x201a10];
    let frame_mat = lambert!({ color: pick(&frame_colors) });
    let mirror_mat = basic_mat!({ color: 0xc0d0e0 });
    let mirror = group();
    // Frame back (thin disc).
    let back = mesh(pg("SmallHandMirrors__frame", cyl_geo!(0.055, 0.055, 0.008, 14)), &frame_mat);
    back.position().y = 0.004;
    mirror.add(&back);
    // Mirror face (slightly smaller lighter disc on top).
    let face = mesh(circle_geo!(0.048, 14), &mirror_mat);
    face.rotation().x = -PI / 2.0;
    face.position().y = 0.0085;
    mirror.add(&face);
    // Handle: cylinder extending outward.
    let handle_len = 0.12;
    let handle = mesh(pg("SmallHandMirrors__handle", cyl_geo!(0.008, 0.010, handle_len, 8)), &frame_mat);
    handle.rotation().z = PI / 2.0;
    handle.position().set(0.055 + handle_len / 2.0, 0.010, 0); // thick end (r 0.010) resting on the floor
    mirror.add(&handle);
    // Handle end knob.
    let knob = mesh(pg("SmallHandMirrors__knob", sphere_geo!(0.012, 8, 6)), &frame_mat);
    knob.position().set(0.055 + handle_len + 0.008, 0.012, 0); // resting on the floor
    mirror.add(&knob);
    mirror.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    mirror.rotation().y = rnd() * PI * 2.0;
    g.world.add(&mirror);
}
