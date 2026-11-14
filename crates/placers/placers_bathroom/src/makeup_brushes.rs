//! `place_makeup_brushes`
use game::prelude::*;
use placers_kit::place::jitter;
use placers_kit::place::pick_open;

// Makeup brush: 7% chance per level. Handle + ferrule + bristle head.
pub(crate) fn place_makeup_brushes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.93 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let handle_colors = [0xd0a040, 0x202020, 0xc0a080, 0x804060, 0xe0d0a0];
    let handle_mat = lambert!({ color: choice(&handle_colors) });
    let ferrule_mat = lambert!({ color: 0xc0c0c0 });
    let bristle_colors = [0xf0e8d8, 0x604030, 0x202020, 0xa08060];
    let bristle_mat = lambert!({ color: choice(&bristle_colors) });
    let brush = group();
    let handle = mesh(pg("MakeupBrushes__handle", cyl_geo!(0.005, 0.008, 0.085, 8)), &handle_mat);
    handle.rotation().z = PI / 2.0;
    handle.position().set(-0.030, 0.008, 0);
    brush.add(&handle);
    let ferrule = mesh(pg("MakeupBrushes__ferrule", cyl_geo!(0.009, 0.009, 0.014, 8)), &ferrule_mat);
    ferrule.rotation().z = PI / 2.0;
    ferrule.position().set(0.018, 0.009, 0);
    brush.add(&ferrule);
    // Bristle head (dome or cone depending on type).
    let bristle_type = rnd();
    let bristle_head;
    if bristle_type < 0.5 {
        // Dome (powder brush).
        bristle_head = mesh(pg("MakeupBrushes__dome", sphere_geo!(0.011, 8, 5)), &bristle_mat);
        bristle_head.scale().x = 1.4;
        bristle_head.position().set(0.038, 0.011, 0); // rests on the floor
    } else {
        // Angled cone (blush/contour).
        bristle_head = mesh(pg("MakeupBrushes__cone", cone_geo!(0.010, 0.024, 8)), &bristle_mat);
        bristle_head.rotation().z = -PI / 2.0;
        bristle_head.position().set(0.036, 0.010, 0); // base 1 mm inside the ferrule mouth
    }
    brush.add(&bristle_head);
    let (px, pz) = jitter(g, x, z, 2.4);
    brush.position().set(px, 0, pz);
    brush.rotation().y = rnd() * PI * 2.0;
    g.world.add(&brush);
}
