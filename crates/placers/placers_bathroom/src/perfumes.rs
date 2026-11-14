//! `place_perfumes`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Perfume bottle w/ atomizer bulb: 6% chance per level.
// Squat glass bottle w/ neck + rubber squeeze bulb.
pub(crate) fn place_perfumes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    // Perfume liquid pool — amber/gold/rose tones.
    let liquid_colors: [u32; 5] = [0xd0a040, 0xf0d090, 0xf0c0a0, 0xa08040, 0xf0a080];
    let liquid_mat = lambert!({ color: pick(&liquid_colors), transparent: true, opacity: 0.75 });
    let glass_mat = lambert!({ color: 0xd8d0e8, transparent: true, opacity: 0.30 });
    // Bulb color pool — vintage rubber tones.
    let bulb_colors: [u32; 4] = [0xa02040, 0x603020, 0x201020, 0x4a3020];
    let bulb_mat = lambert!({ color: pick(&bulb_colors) });
    let metal_mat = lambert!({ color: 0xc0a060 });
    let p = group();
    // Bottle body — sphere flattened slightly for classic
    // Art Deco perfume silhouette.
    let body_geom = sphere_geo!(0.035, 12, 8);
    let body = mesh(&body_geom, &liquid_mat);
    body.scale().y = 0.85;
    body.position().y = 0.030;
    p.add(&body);
    // Glass overlay (slightly larger).
    let glass = mesh(&body_geom, &glass_mat);
    glass.scale().set(1.08, 0.90, 1.08);
    glass.position().y = 0.030;
    p.add(&glass);
    // Neck (short thin cylinder).
    let neck = mesh(pg("Perfumes__neck", cyl_geo!(0.008, 0.010, 0.016, 8)), &metal_mat);
    neck.position().y = 0.066;
    p.add(&neck);
    // Atomizer connector (horizontal metal ring).
    let connector = mesh(pg("Perfumes__ring", torus_geo!(0.010, 0.002, 4, 8)), &metal_mat);
    connector.rotation().x = PI / 2.0;
    connector.position().y = 0.077;
    p.add(&connector);
    // Squeeze bulb — larger squashed sphere off to the side.
    let bulb = mesh(pg("Perfumes__bulb", sphere_geo!(0.020, 8, 7)), &bulb_mat);
    bulb.scale().set(1.2, 1.0, 1.2);
    bulb.position().set(0.048, 0.075, 0);
    p.add(&bulb);
    // Thin flexible connector tube (short cylinder aligned).
    let tube_mat = lambert!({ color: 0x503020 });
    let tube = mesh(pg("Perfumes__tube", cyl_geo!(0.003, 0.003, 0.030, 5)), &tube_mat);
    tube.rotation().z = PI / 2.0;
    tube.position().set(0.025, 0.075, 0);
    p.add(&tube);
    // Small tassel hanging from bulb — 3-segment tiny cord.
    let tassel_mat = lambert!({ color: 0xd0a040 });
    for i in 0..3 {
        let seg = mesh(pg("Perfumes__cord", cyl_geo!(0.001, 0.001, 0.006, 3)), &tassel_mat);
        seg.position().set(0.048, 0.053 - i as f64 * 0.006, 0); // top segment 1 mm into the bulb (bottom y 0.055)
        p.add(&seg);
    }
    let tassel_knot = mesh(pg("Perfumes__tassel", sphere_geo!(0.003, 4, 4)), &tassel_mat);
    tassel_knot.position().set(0.048, 0.035, 0);
    p.add(&tassel_knot);
    p.position().set(g.wx(x) + (rnd() - 0.5) * 2.2, 0, g.wz(z) + (rnd() - 0.5) * 2.2);
    p.rotation().y = rnd() * PI * 2.0;
    g.world.add(&p);
}
