//! `place_vinyl_record_discs`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick_hex;
use placers_kit::place::scatter;

// Loose vinyl record: 8% chance per level. Black disc w/ coloured label.
pub(crate) fn place_vinyl_record_discs(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let vinyl_mat = lambert!({ color: 0x101008 });
    let label_colors = [0xa02040, 0x2050d8, 0xd8b040, 0x30a030];
    let label_mat = lambert!({ color: pick_hex(&label_colors) });
    let disc = group();
    // Main vinyl.
    let body = mesh(pg("VinylRecordDiscs__disc", cyl_geo!(0.078, 0.078, 0.002, 20)), &vinyl_mat);
    body.position().y = 0.001;
    disc.add(&body);
    // Label.
    let label = mesh(circle_geo!(0.025, 12), &label_mat);
    label.rotation().x = -PI / 2.0;
    label.position().y = 0.0025;
    disc.add(&label);
    // Center hole (tiny dark spot).
    let hole = mesh(circle_geo!(0.003, 6), &vinyl_mat);
    hole.rotation().x = -PI / 2.0;
    hole.position().y = 0.0028;
    disc.add(&hole);
    scatter(g, &disc, x, z, 2.4);
    g.world.add(&disc);
}
