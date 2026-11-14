//! `place_dirt_clods`
use game::prelude::*;
use placers_kit::place::jitter;
use placers_kit::place::pick_open;

// Dirt clod cluster: 8% chance per level. 3-6 small brown clumps.
pub(crate) fn place_dirt_clods(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let dirt_colors = [0x604030, 0x503020, 0x705040, 0x402010];
    let cluster = group();
    let count = 3 + (rnd() * 4.0).floor() as i32;
    for _i in 0..count {
        let dirt_mat = lambert!({ color: choice(&dirt_colors) });
        let r = 0.010 + rnd() * 0.008;
        let clod = mesh(pg("DirtClods__clod", octa_geo!(r, 0)), &dirt_mat);
        clod.position().set((rnd() - 0.5) * 0.14, r * 0.7, (rnd() - 0.5) * 0.14);
        clod.rotation().set(rnd() * PI, rnd() * PI, rnd() * PI);
        clod.scale().y = 0.6 + rnd() * 0.3;
        cluster.add(&clod);
    }
    let (px, pz) = jitter(g, x, z, 2.5);
    cluster.position().set(px, 0, pz);
    g.world.add(&cluster);
}
