//! `place_small_trumpets`: Small trumpet: 6% chance per level. Small brass toy trumpet with bell + tube + 3 valves.
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

/// Small trumpet: 6% chance per level. Small brass toy
/// trumpet with bell + tube + 3 valves.
pub(crate) fn place_small_trumpets(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let brass_mat = lambert!({ color: 0xc89040 });
    let dark_mat = lambert!({ color: 0x2a2418 });
    let trumpet = group();
    // Main tube: horizontal.
    let tube = mesh(pg("SmallTrumpets__tube", cyl_geo!(0.010, 0.010, 0.18, 8)), &brass_mat);
    tube.rotation().z = PI / 2.0;
    tube.position().set(0.09, 0.020, 0);
    trumpet.add(&tube);
    // Bell: flared cone at far end.
    let bell = mesh(pg("SmallTrumpets__bell", cone_geo!(0.035, 0.06, 12, 1, true)), &brass_mat);
    bell.rotation().z = PI / 2.0; // narrow end on the tube, flaring out to +X
    bell.position().set(0.21, 0.020, 0);
    trumpet.add(&bell);
    // Mouthpiece: small cup at near end.
    let mouth = mesh(pg("SmallTrumpets__mouth", cyl_geo!(0.012, 0.010, 0.012, 8)), &brass_mat);
    mouth.rotation().z = PI / 2.0;
    mouth.position().set(-0.006, 0.020, 0);
    trumpet.add(&mouth);
    // 3 valve caps on top of tube.
    for i in 0..3 {
        let valve = mesh(pg("SmallTrumpets__valve", cyl_geo!(0.008, 0.008, 0.020, 6)), &dark_mat);
        valve.position().set(0.06 + i as f64 * 0.025, 0.038, 0);
        trumpet.add(&valve);
    }
    trumpet.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    trumpet.rotation().y = rnd() * PI * 2.0;
    // Tilt so it rests on the bell rim and the mouthpiece.
    trumpet.rotation().z = 0.023f64.datan2(0.252);
    trumpet.position().y = -0.0069;
    g.world.add(&trumpet);
}
