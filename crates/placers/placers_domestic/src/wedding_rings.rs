//! `place_wedding_rings`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick_hex;
use placers_kit::place::scatter;

// Wedding ring: 6% chance per level. Small gold torus on floor.
pub(crate) fn place_wedding_rings(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let gold_colors = [0xd8b040, 0xe8c860, 0xd0d0d0, 0xf0d8d8];
    let gold_mat = lambert!({ color: pick_hex(&gold_colors) });
    let gem_mat = basic_mat!({ color: 0xe0f0ff, transparent: true, opacity: 0.9 });
    let ring = group();
    // Band torus.
    let band = mesh(pg("WeddingRings__band", torus_geo!(0.010, 0.002, 5, 14)), &gold_mat);
    band.rotation().x = PI / 2.0;
    band.position().y = 0.002;
    ring.add(&band);
    // 40% chance small diamond gem on top (an octahedron).
    if rnd() < 0.40 {
        let gem = mesh(octa_geo!(0.003), &gem_mat);
        gem.position().y = 0.006;
        ring.add(&gem);
    }
    scatter(g, &ring, x, z, 2.4);
    g.world.add(&ring);
}
