//! `place_shattered_eyeglasses`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::scatter_on_floor;

// Shattered eyeglasses: 6% chance per level. Frame + lenses + broken temple.
pub(crate) fn place_shattered_eyeglasses(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let frame_mat = lambert!({ color: 0x181008 });
    let lens_mat = basic_mat!({ color: 0xc0d0e0, transparent: true, opacity: 0.4 });
    let glasses = group();
    // 2 lens frames (torus).
    for s in [-1.0, 1.0] {
        let rim = mesh(pg("ShatteredEyeglasses__rim", torus_geo!(0.020, 0.002, 4, 12)), &frame_mat);
        rim.rotation().x = PI / 2.0;
        rim.position().set(s * 0.024, 0.002, 0);
        glasses.add(&rim);
        // Lens disc.
        let lens = mesh(circle_geo!(0.018, 12), &lens_mat);
        lens.rotation().x = -PI / 2.0;
        lens.position().set(s * 0.024, 0.003, 0);
        glasses.add(&lens);
    }
    // Bridge between lenses.
    let bridge = mesh(pg("ShatteredEyeglasses__bridge", cyl_geo!(0.001, 0.001, 0.010, 3)), &frame_mat);
    bridge.rotation().z = PI / 2.0;
    bridge.position().set(0, 0.002, 0);
    glasses.add(&bridge);
    // 1 temple attached (other broken off).
    let temple_side = if rnd() < 0.5 { -1.0 } else { 1.0 };
    let temple = mesh(pg("ShatteredEyeglasses__temple", cyl_geo!(0.001, 0.001, 0.10, 3)), &frame_mat);
    temple.rotation().x = PI / 2.0;
    temple.rotation().z = 0.1;
    temple.position().set(temple_side * 0.044, 0.002, -0.05);
    glasses.add(&temple);
    // Broken temple laying separately.
    let broken = mesh(pg("ShatteredEyeglasses__broken", cyl_geo!(0.001, 0.001, 0.06, 3)), &frame_mat);
    broken.rotation().z = PI / 3.0;
    broken.position().set(-temple_side * 0.10, 0.002, 0.04);
    glasses.add(&broken);
    scatter_on_floor(g, &glasses, x, z, 2.4);
    g.world.add(&glasses);
}
