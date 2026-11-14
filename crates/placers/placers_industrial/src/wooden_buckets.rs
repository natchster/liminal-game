//! `place_wooden_buckets`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Wooden staved bucket: 4% chance per level. Barrel-style bucket w/
// vertical staves + metal bands.
pub(crate) fn place_wooden_buckets(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let wood_mat = lambert!({ color: 0x8b5a2b });
    let band_mat = lambert!({ color: 0x505050 });
    let bucket = group();
    // Main body (slightly tapered cylinder — barrel shape).
    let body = mesh(pg("WoodenBuckets__body", cyl_geo!(0.080, 0.070, 0.14, 12)), &wood_mat);
    body.position().y = 0.070;
    bucket.add(&body);
    // 12 vertical stave lines (darker thin cylinders on outside).
    for i in 0..12 {
        let a = (i as f64 / 12.0) * PI * 2.0;
        let stave = mesh(cyl_geo!(0.0015, 0.0015, 0.14, 3), lambert!({ color: 0x603818 }));
        // follow the body's taper (r ≈ 0.072 at the foot, 0.081 at the rim)
        stave.rotation().y = -a;
        stave.rotation().z = -0.0655;
        stave.position().set(a.dcos() * 0.0770, 0.070, a.dsin() * 0.0770);
        bucket.add(&stave);
    }
    // 2 metal bands (top + bottom).
    for i in 0..2 {
        let band_y = 0.020 + i as f64 * 0.100;
        let band = mesh(pg("WoodenBuckets__band", torus_geo!(0.079 - i as f64 * 0.005, 0.004, 4, 16)), &band_mat);
        // sized to the body at this height (it widens upward): the bottom band
        // floated off it and the top one was buried
        let band_s = (0.070 + 0.010 * band_y / 0.14 + 0.0015) / (0.079 - i as f64 * 0.005);
        band.scale().set(band_s, band_s, 1);
        band.rotation().x = PI / 2.0;
        band.position().y = band_y;
        bucket.add(&band);
    }
    // Curved wire handle.
    let handle = mesh(pg("WoodenBuckets__handle", torus_geo!(0.085, 0.003, 4, 12, PI)), &band_mat);
    handle.rotation().y = PI / 2.0;
    handle.position().y = 0.138;
    bucket.add(&handle);
    bucket.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    bucket.rotation().y = rnd() * PI * 2.0;
    g.world.add(&bucket);
}
