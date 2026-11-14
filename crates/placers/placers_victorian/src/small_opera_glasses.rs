//! `place_small_opera_glasses`: Small opera glasses: 6% chance per level. Twin cylinders on handle. Party-costume family (fancy accessory).
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

/// Small opera glasses: 6% chance per level. Twin cylinders
/// on handle. Party-costume family (fancy accessory).
pub(crate) fn place_small_opera_glasses(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let body_colors = [0x8a3820, 0x201a10, 0xd8b040, 0x606870];
    let body_mat = lambert!({ color: pick(&body_colors) });
    let lens_mat = basic_mat!({ color: 0x304040, transparent: true, opacity: 0.75 });
    let opera = group();
    // 2 lens cylinders side by side.
    for i in 0..2 {
        let s = if i == 0 { -1.0 } else { 1.0 };
        let barrel = mesh(pg("SmallOperaGlasses__barrel", cyl_geo!(0.018, 0.018, 0.045, 10)), &body_mat);
        barrel.rotation().z = PI / 2.0;
        // Side by side (parallel, either side of the bridge).
        barrel.position().set(0.030, 0.020, s * 0.023);
        opera.add(&barrel);
        // Lens disc on far end.
        let lens = mesh(circle_geo!(0.016, 8), &lens_mat);
        lens.rotation().y = PI / 2.0;
        lens.position().set(0.030 + 0.023, 0.020, s * 0.023);
        opera.add(&lens);
    }
    // Bridge connecting them.
    let bridge = mesh(pg("SmallOperaGlasses__bridge", box_geo!(0.030, 0.008, 0.010)), &body_mat);
    bridge.position().set(0.030, 0.020, 0);
    opera.add(&bridge);
    // Handle extending sideways, out from the side of the first barrel,
    // sloping down so the finial rests on the floor.
    let handle = mesh(pg("SmallOperaGlasses__handle", cyl_geo!(0.004, 0.004, 0.10, 6)), &body_mat);
    handle.rotation().x = -PI / 2.0 - 0.16;
    handle.position().set(0.030, 0.012, -0.039 - 0.05 * 0.16f64.cos());
    opera.add(&handle);
    opera.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    opera.rotation().y = rnd() * PI * 2.0;
    g.world.add(&opera);
}
