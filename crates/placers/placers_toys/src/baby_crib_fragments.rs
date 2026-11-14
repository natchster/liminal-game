//! `place_baby_crib_fragments`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick_hex;
use placers_kit::place::scatter;

// Baby crib fragment: 3% chance per level. Wooden crib section with bars.
pub(crate) fn place_baby_crib_fragments(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    } // 3% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let wood_colors = [0xf0e0d0, 0xe8dcc0, 0x8a5030];
    let wood_mat = lambert!({ color: pick_hex(&wood_colors) });
    let crib = group();
    // Base rail (bottom).
    let base_rail = mesh(pg("BabyCribFragments__baserail", box_geo!(0.44, 0.016, 0.020)), &wood_mat);
    base_rail.position().set(0, 0.008, 0);
    crib.add(&base_rail);
    // Top rail.
    let top_rail = mesh(pg("BabyCribFragments__toprail", box_geo!(0.44, 0.016, 0.020)), &wood_mat);
    top_rail.position().set(0, 0.42, 0);
    crib.add(&top_rail);
    // 10 vertical bars.
    let n_bars = 10;
    for i in 0..n_bars {
        let bx = -0.20 + i as f64 * (0.40 / (n_bars - 1) as f64);
        let bar = mesh(pg("BabyCribFragments__spindle", cyl_geo!(0.005, 0.005, 0.40, 5)), &wood_mat);
        bar.position().set(bx, 0.22, 0);
        crib.add(&bar);
    }
    // 2 corner posts (taller).
    for s in [-1.0, 1.0] {
        let post = mesh(pg("BabyCribFragments__post", cyl_geo!(0.014, 0.014, 0.50, 5)), &wood_mat);
        post.position().set(s * 0.22, 0.25, 0);
        crib.add(&post);
        // Post cap knob.
        let knob = mesh(pg("BabyCribFragments__finial", sphere_geo!(0.018, 8, 6)), &wood_mat);
        knob.position().set(s * 0.22, 0.51, 0);
        crib.add(&knob);
    }
    scatter(g, &crib, x, z, 1.6);
    g.world.add(&crib);
}
