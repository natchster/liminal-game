//! `place_small_tambourines`: Small tambourine: 8% chance per level. Torus rim + membrane surface + tiny jingle dots around rim.
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

/// Small tambourine: 8% chance per level. Torus rim + membrane
/// surface + tiny jingle dots around rim.
pub(crate) fn place_small_tambourines(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let rim_mat = lambert!({ color: 0x6a4820 });
    let mem_mat = lambert!({ color: 0xd8ccb0 });
    let jingle_mat = lambert!({ color: 0xa8a840 });
    let tamb = group();
    // Rim: torus laid flat.
    let rim = mesh(pg("SmallTambourines__rim", torus_geo!(0.09, 0.010, 4, 12)), &rim_mat);
    rim.rotation().x = -PI / 2.0;
    rim.position().y = 0.010;
    tamb.add(&rim);
    // Membrane surface: cream disc inside rim.
    let mem = mesh(circle_geo!(0.086, 12), &mem_mat);
    mem.rotation().x = -PI / 2.0;
    mem.position().y = 0.011;
    tamb.add(&mem);
    // 6 tiny brass jingle dots around rim.
    for j in 0..6 {
        let j_ang = (j as f64 / 6.0) * PI * 2.0;
        let jingle = mesh(pg("SmallTambourines__jingle", cyl_geo!(0.008, 0.008, 0.004, 6)), &jingle_mat);
        jingle.position().set(j_ang.dcos() * 0.09, 0.014, j_ang.dsin() * 0.09);
        tamb.add(&jingle);
    }
    tamb.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    tamb.rotation().y = rnd() * PI * 2.0;
    g.world.add(&tamb);
}
