//! `place_small_scales`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Small vintage brass scales: 5% chance per level, gentle sway.
pub(crate) fn place_small_scales(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = pick(&open_tiles);
    let brass_mat = lambert!({ color: 0xc89040 });
    let scales = group();
    // Base disc.
    let base = mesh(pg("SmallScales__base", cyl_geo!(0.045, 0.055, 0.010, 10)), &brass_mat);
    base.position().y = 0.005;
    scales.add(&base);
    // Central pillar.
    let pillar = mesh(pg("SmallScales__pillar", cyl_geo!(0.005, 0.005, 0.10, 6)), &brass_mat);
    pillar.position().y = 0.060;
    scales.add(&pillar);
    // Cross-beam group (rotates as a whole).
    let beam = group();
    let beam_bar = mesh(pg("SmallScales__beam", cyl_geo!(0.003, 0.003, 0.14, 4)), &brass_mat);
    beam_bar.rotation().z = PI / 2.0;
    beam.add(&beam_bar);
    // 2 chains hanging + 2 pans.
    for s in [-1.0f64, 1.0] {
        for ci in 0..3 {
            let chain = mesh(pg("SmallScales__chain", cyl_geo!(0.001, 0.001, 0.030, 3)), &brass_mat);
            // One straight run from inside the beam end down into the pan.
            chain.position().set(s * 0.065, -0.013 - ci as f64 * 0.030, 0);
            beam.add(&chain);
        }
        let pan = mesh(pg("SmallScales__pan", cyl_geo!(0.024, 0.026, 0.004, 10)), &brass_mat);
        // Hung 1.1 cm higher: the chains reach it and it clears the base.
        pan.position().set(s * 0.065, -0.087, 0);
        beam.add(&pan);
    }
    beam.position().y = 0.112;
    scales.add(&beam);
    // Very slight balance sway.
    let sway_phase = rnd() * PI * 2.0;
    let sway_period = 6.0 + rnd() * 4.0;
    let beam2 = beam.clone();
    g.animate(move |_g, _dt, t| {
        beam2.rotation().z = (t * (PI * 2.0 / sway_period) + sway_phase).sin() * 0.05;
    });
    scales.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    scales.rotation().y = rnd() * PI * 2.0;
    g.world.add(&scales);
}
