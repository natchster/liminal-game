//! `place_marbles`
use game::prelude::*;
use placers_kit::place::jitter;
use placers_kit::place::pick_open;

// Marbles cluster: 8% chance per level. 5-10 small translucent glass spheres.
pub(crate) fn place_marbles(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let marble_colors = [0xd8e8f0, 0xf0d8a0, 0xd0c0e0, 0xa0e0c0, 0xe0a0a0, 0xc0e0f0];
    let cluster = group();
    let count = 5 + (rnd() * 6.0).floor() as i32;
    for _i in 0..count {
        let m_mat = lambert!({ color: choice(&marble_colors), transparent: true, opacity: 0.7 });
        let r = 0.008 + rnd() * 0.003;
        let marble = mesh(sphere_geo!(r, 8, 6), &m_mat);
        marble.position().set((rnd() - 0.5) * 0.14, r, (rnd() - 0.5) * 0.14);
        cluster.add(&marble);
    }
    let (px, pz) = jitter(g, x, z, 2.4);
    cluster.position().set(px, 0, pz);
    g.world.add(&cluster);
}
