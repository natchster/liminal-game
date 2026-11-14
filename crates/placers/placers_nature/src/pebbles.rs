//! `place_pebbles`
use game::prelude::*;
use placers_kit::place::jitter;
use placers_kit::place::pick_open;

// Pebble cluster: 8% chance per level.
pub(crate) fn place_pebbles(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let stone_colors = [0x707068, 0x8a8888, 0x605850, 0x504840, 0x9a9080];
    let cluster = group();
    let count = 4 + (rnd() * 5.0).floor() as i32;
    for _i in 0..count {
        let stone_mat = lambert!({ color: choice(&stone_colors) });
        let r = 0.006 + rnd() * 0.008;
        let pebble = mesh(pg("Pebbles__pebble", sphere_geo!(r, 6, 4)), &stone_mat);
        pebble.position().set((rnd() - 0.5) * 0.14, r * 0.7, (rnd() - 0.5) * 0.14);
        pebble.scale().set(1.0 + rnd() * 0.4, 0.5 + rnd() * 0.3, 1.0 + rnd() * 0.4);
        pebble.rotation().set(rnd() * PI, rnd() * PI, rnd() * PI);
        cluster.add(&pebble);
    }
    let (px, pz) = jitter(g, x, z, 2.5);
    cluster.position().set(px, 0, pz);
    g.world.add(&cluster);
}
