//! `place_cleaning_mop_buckets`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Mop + bucket w/ water puddle (floor): 4% chance per level.
pub(crate) fn place_cleaning_mop_buckets(g: &mut Game, _dist: &[i32]) {
    g.pp.mop_bucket_positions = vec![]; // reset per level
    if rnd() < 0.96 {
        return;
    }
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = open_tiles[(rnd() * open_tiles.len() as f64).floor() as usize];
    // Bucket color pool.
    let bucket_colors = [0xa02020, 0x2060a0, 0xd0a040, 0x606060, 0x808080];
    let bucket_mat = lambert!({ color: pick(&bucket_colors) });
    // Mop handle color pool.
    let handle_colors = [0x8b5a2b, 0x604020, 0x505050, 0xa07040];
    let handle_mat = lambert!({ color: pick(&handle_colors) });
    // Mop head color — dirty/wet gray or off-white.
    let mop_head_colors = [0xc0b8a0, 0xa09888, 0xe0d8c8, 0x808070];
    let mop_head_mat = lambert!({ color: pick(&mop_head_colors) });
    let metal_mat = lambert!({ color: 0xa0a0a8 });
    let mb = group();
    // Bucket — trapezoidal (tapered cylinder).
    let bucket_h = 0.20;
    let (bucket_r_top, bucket_r_bot) = (0.13, 0.11);
    let bucket = mesh(pg("CleaningMopBuckets__bucket", cyl_geo!(bucket_r_top, bucket_r_bot, bucket_h, 12, 1, true)), &bucket_mat);
    bucket.position().y = bucket_h / 2.0 + 0.002;
    mb.add(&bucket);
    // Bottom disc.
    let bottom = mesh(circle_geo!(bucket_r_bot, 12), &bucket_mat);
    bottom.rotation().x = -PI / 2.0;
    bottom.position().y = 0.002;
    mb.add(&bottom);
    // Rim torus.
    let rim = mesh(pg("CleaningMopBuckets__rim", torus_geo!(bucket_r_top, 0.006, 4, 14)), &bucket_mat);
    rim.rotation().x = PI / 2.0;
    rim.position().y = bucket_h + 0.002;
    mb.add(&rim);
    // Metal handle arch — semi-torus arching over top, ends on the side nubs.
    let bucket_handle = mesh(pg("CleaningMopBuckets__bail", torus_geo!(bucket_r_top * 1.02, 0.004, 3, 10, PI)), &metal_mat);
    bucket_handle.rotation().z = 0.0;
    bucket_handle.position().y = bucket_h * 0.90;
    mb.add(&bucket_handle);
    // Handle-attach pivots (small nubs at bucket rim sides).
    for side in [-1.0, 1.0] {
        let nub = mesh(pg("CleaningMopBuckets__nub", sphere_geo!(0.005, 5, 4)), &metal_mat);
        nub.position().set(side * bucket_r_top, bucket_h * 0.90, 0);
        mb.add(&nub);
    }
    // Water inside bucket — dark surface disc slightly below rim.
    if rnd() < 0.7 {
        let water_mat = basic_mat!({ color: 0x304060, transparent: true, opacity: 0.75 });
        let water = mesh(circle_geo!(bucket_r_top * 0.92, 12), &water_mat);
        water.rotation().x = -PI / 2.0;
        water.position().y = bucket_h * 0.72;
        mb.add(&water);
    }
    // Mop — long handle leaning at an angle, foot in the head down in the bucket.
    let mop_handle_len = 1.10;
    let lean_angle: f64 = -0.35;
    let mop_handle = mesh(pg("CleaningMopBuckets__stick", cyl_geo!(0.010, 0.010, mop_handle_len, 6)), &handle_mat);
    // rotation.z = lean_angle leans the top toward +x, so the end points follow that axis.
    mop_handle.rotation().z = lean_angle;
    let mop_bottom_x = 0.0;
    let mop_bottom_y = bucket_h * 0.6;
    let mop_top_x = mop_bottom_x - lean_angle.sin() * mop_handle_len;
    let mop_top_y = mop_bottom_y + lean_angle.cos() * mop_handle_len;
    mop_handle.position().set((mop_bottom_x + mop_top_x) / 2.0, (mop_bottom_y + mop_top_y) / 2.0, 0);
    mb.add(&mop_handle);
    // Grip band at top of handle (small colored torus), ring axis along the stick.
    let grip_mat = lambert!({ color: 0xa02040 });
    let grip = mesh(pg("CleaningMopBuckets__grip", torus_geo!(0.011, 0.003, 3, 8)), &grip_mat);
    grip.rotation().y = -lean_angle;
    grip.rotation().x = -PI / 2.0;
    grip.position().set(mop_top_x + lean_angle.sin() * 0.05, mop_top_y - lean_angle.cos() * 0.05, 0);
    mb.add(&grip);
    // Mop head — bunch of thin strands radiating from bottom (inside bucket); the
    // clamp (centre mop_head_y + 0.02) takes the stick's foot.
    let mop_head_x = mop_bottom_x;
    let mop_head_y = mop_bottom_y - 0.025; // Inside bucket.
    let mop_head_z = 0.0;
    // 12 strands radiating out + down.
    let strand_count = 12;
    for i in 0..strand_count {
        let ang = (i as f64 / strand_count as f64) * PI * 2.0;
        let strand_len = 0.06 + rnd() * 0.04;
        let strand = mesh(pg("CleaningMopBuckets__strand", cyl_geo!(0.002, 0.001, strand_len, 3)), &mop_head_mat);
        // Angle downward + outward.
        strand.rotation().x = ang.sin() * 0.6;
        strand.rotation().z = ang.cos() * 0.6;
        // Position at mop head center.
        strand.position().set(mop_head_x, mop_head_y, mop_head_z);
        // Offset along direction.
        let dx = ang.cos() * strand_len * 0.35;
        let dz = ang.sin() * strand_len * 0.35;
        strand.position().set(mop_head_x + dx, mop_head_y - strand_len * 0.35, mop_head_z + dz);
        mb.add(&strand);
    }
    // Metal band clamping strands to handle.
    let clamp2 = mesh(pg("CleaningMopBuckets__clamp", cyl_geo!(0.014, 0.014, 0.020, 8)), &metal_mat);
    clamp2.position().set(mop_head_x, mop_head_y + 0.020, mop_head_z);
    mb.add(&clamp2);
    // 60% chance: water puddle on floor beside bucket.
    if rnd() < 0.6 {
        let puddle_mat = basic_mat!({ color: 0x304060, transparent: true, opacity: 0.55 });
        let puddle = mesh(circle_geo!(0.11, 14), &puddle_mat);
        puddle.rotation().x = -PI / 2.0;
        puddle.scale().set(1.4, 1.0, 0.9);
        puddle.position().set(-bucket_r_top - 0.05, 0.001, 0.04);
        puddle.rotation().z = rnd() * PI * 2.0;
        mb.add(&puddle);
    }
    mb.position().set(g.wx(x) + (rnd() - 0.5) * 1.6, 0, g.wz(z) + (rnd() - 0.5) * 1.6);
    mb.rotation().y = rnd() * PI * 2.0;
    g.world.add(&mb);
    // Register mop bucket position for water-slosh audio.
    let p = mb.pos();
    g.pp.mop_bucket_positions.push(v3(p.x, 0, p.z));
}
