//! `place_metal_buckets`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Metal bucket: 5% chance per level. Tapered cylinder pail + curved handle.
pub(crate) fn place_metal_buckets(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let bucket_colors: [u32; 5] = [0xa0a0a0, 0x808080, 0x606060, 0x2060c0, 0xc02020];
    let bucket_mat = lambert!({ color: pick(&bucket_colors) });
    let handle_mat = lambert!({ color: 0x404040 });
    let bucket = group();
    // Tapered body (narrower at bottom).
    let body = mesh(pg("MetalBuckets__body", cyl_geo!(0.075, 0.058, 0.11, 14)), &bucket_mat);
    body.position().y = 0.055;
    bucket.add(&body);
    // Rim (slight lip torus).
    let rim = mesh(pg("MetalBuckets__rim", torus_geo!(0.075, 0.005, 4, 16)), &bucket_mat);
    rim.rotation().x = PI / 2.0;
    rim.position().y = 0.110;
    bucket.add(&rim);
    // Curved wire handle (half-torus over top).
    let handle = mesh(pg("MetalBuckets__handle", torus_geo!(0.082, 0.003, 4, 12, PI)), &handle_mat);
    handle.rotation().z = 0.0;
    handle.rotation().y = 0.0; // ends at ±x on the ears (was turned 90° away from them)
    handle.position().y = 0.106;
    bucket.add(&handle);
    // 2 handle attachment ears.
    for side in [-1.0, 1.0] {
        let ear = mesh(pg("MetalBuckets__ear", box_geo!(0.008, 0.010, 0.006)), &bucket_mat);
        ear.position().set(side * 0.076, 0.100, 0);
        bucket.add(&ear);
    }
    // 30% tipped on side (rolling): tilted so the rim and the base edge both
    // touch the floor (was pivoted on the base centre, half sunk), bail folded
    // down over the upper side, yaw applied last so it can lie any way.
    let tipped = rnd() < 0.30;
    if tipped {
        bucket.set_rotation_order(EulerOrder::YXZ);
        bucket.rotation().x = PI / 2.0 - 0.197;
        handle.rotation().x = -2.15;
    }
    bucket.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, if tipped { 0.057 } else { 0.0 }, g.wz(z) + (rnd() - 0.5) * 2.0);
    bucket.rotation().y = rnd() * PI * 2.0;
    g.world.add(&bucket);
}
