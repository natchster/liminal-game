//! `place_small_hourglasses`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Small hourglass: 5% chance per level, animated falling stream.
pub(crate) fn place_small_hourglasses(g: &mut Game, _dist: &[i32]) {
    g.pp.hourglass_positions.clear(); // reset per level
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = pick(&open_tiles);
    let frame_mat = lambert!({ color: 0x8a5030 });
    let glass_mat = basic_mat!({ color: 0xf0f0f0, transparent: true, opacity: 0.20 });
    let sand_mat = lambert!({ color: 0xd8b060 });
    let hg = group();
    // Top frame plate.
    let top_plate = mesh(pg("SmallHourglasses__topPlate", cyl_geo!(0.035, 0.035, 0.006, 8)), &frame_mat);
    top_plate.position().y = 0.157;
    hg.add(&top_plate);
    // Bottom frame plate.
    let bot_plate = mesh(pg("SmallHourglasses__botPlate", cyl_geo!(0.035, 0.035, 0.006, 8)), &frame_mat);
    bot_plate.position().y = 0.003;
    hg.add(&bot_plate);
    // 4 corner posts.
    for i in 0..4 {
        let a = (i as f64 / 4.0) * PI * 2.0 + PI / 4.0;
        let post = mesh(pg("SmallHourglasses__post", cyl_geo!(0.003, 0.003, 0.15, 4)), &frame_mat);
        post.position().set(a.dcos() * 0.028, 0.080, a.dsin() * 0.028);
        hg.add(&post);
    }
    // The two bulbs meet at their apexes (the waist, y 0.080) and run to the
    // plates: stretched 1.233 in height.
    // Top glass bulb (upside-down cone).
    let top_bulb = mesh(cone_geo!(0.026, 0.060, 12, 1, true), &glass_mat);
    top_bulb.rotation().x = PI;
    top_bulb.scale().y = 1.233;
    top_bulb.position().y = 0.117;
    hg.add(&top_bulb);
    // Bottom glass bulb.
    let bot_bulb = mesh(cone_geo!(0.026, 0.060, 12, 1, true), &glass_mat);
    bot_bulb.scale().y = 1.233;
    bot_bulb.position().y = 0.043;
    hg.add(&bot_bulb);
    // Top sand: turned point-down into the upper bulb's funnel, narrowed to
    // stay inside the glass.
    let top_sand = mesh(pg("SmallHourglasses__sand", cone_geo!(0.020, 0.035, 8)), &sand_mat);
    top_sand.rotation().x = PI;
    top_sand.scale().set(0.6, 1, 0.6);
    top_sand.position().y = 0.0995;
    hg.add(&top_sand);
    // Bottom sand: a heap on the bottom plate.
    let bot_sand = mesh(pg("SmallHourglasses__sand", cone_geo!(0.020, 0.028, 8)), &sand_mat);
    bot_sand.position().y = 0.020;
    hg.add(&bot_sand);
    // Falling sand stream: from the waist (0.080) down to the heap's crest (0.034).
    let stream_mat = lambert!({ color: 0xd8b060 });
    let stream = mesh(cyl_geo!(0.001, 0.001, 0.030, 4), &stream_mat);
    stream.scale().y = 1.53;
    stream.position().y = 0.057;
    hg.add(&stream);
    // Subtle sand shimmer animator.
    let shim_phase = rnd() * PI * 2.0;
    let stream2 = stream.clone();
    g.animate(move |_g, _dt, t| {
        stream2.position().y = 0.057 + (t * 8.0 + shim_phase).dsin() * 0.001;
        stream2.rotation().y = t * 2.0;
    });
    hg.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    hg.rotation().y = rnd() * PI * 2.0;
    g.world.add(&hg);
    // Track for sand whoosh audio.
    let hp = hg.pos();
    g.pp.hourglass_positions.push(v3(hp.x, 0.07, hp.z));
}
