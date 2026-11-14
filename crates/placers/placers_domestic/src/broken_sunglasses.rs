//! `place_broken_sunglasses`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::scatter;

// Broken sunglasses: 7% chance per level. Frame + 2 lenses
// (one cracked, one intact) + snapped temple.
pub(crate) fn place_broken_sunglasses(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.93 {
        return;
    } // 7% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = choice(&open);
    let frame_colors = [0x181008, 0x603820, 0x2050a0, 0xd82040];
    let frame_mat = lambert!({ color: choice(&frame_colors) });
    let lens_mat = basic_mat!({ color: 0x2a2818, transparent: true, opacity: 0.7 });
    let glasses = group();
    // 2 lens frames.
    for s in [-1.0, 1.0] {
        let rim = mesh(pg("BrokenSunglasses__rim", torus_geo!(0.024, 0.003, 4, 12)), &frame_mat);
        rim.rotation().x = PI / 2.0;
        rim.position().set(s * 0.028, 0.003, 0);
        glasses.add(&rim);
        // Lens (dark tinted).
        let lens = mesh(circle_geo!(0.022, 12), &lens_mat);
        lens.rotation().x = -PI / 2.0;
        lens.position().set(s * 0.028, 0.004, 0);
        glasses.add(&lens);
    }
    // Bridge.
    let bridge = mesh(pg("BrokenSunglasses__bridge", cyl_geo!(0.002, 0.002, 0.014, 4)), &frame_mat);
    bridge.rotation().z = PI / 2.0;
    bridge.position().y = 0.003;
    glasses.add(&bridge);
    // 1 intact temple + 1 broken temple.
    let attached_side = if rnd() < 0.5 { -1.0 } else { 1.0 };
    let temple = mesh(pg("BrokenSunglasses__temple", cyl_geo!(0.002, 0.002, 0.10, 4)), &frame_mat);
    temple.rotation().x = PI / 2.0;
    temple.rotation().z = 0.1;
    temple.position().set(attached_side * 0.052, 0.003, -0.05);
    glasses.add(&temple);
    // Broken temple lying separately.
    let broken = mesh(pg("BrokenSunglasses__broken", cyl_geo!(0.002, 0.002, 0.06, 4)), &frame_mat);
    // Lying flat on the floor (axis horizontal), turned 45 degrees, not stuck half into it.
    broken.set_rotation_order(EulerOrder::YXZ);
    broken.rotation().set(PI / 2.0, PI / 4.0, 0);
    broken.position().set(-attached_side * 0.08, 0.003, 0.06);
    glasses.add(&broken);
    scatter(g, &glasses, x, z, 2.4);
    g.world.add(&glasses);
}
