//! `place_cotton_balls`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Cotton ball cluster: 8% chance per level, 3-6 puffs.
pub(crate) fn place_cotton_balls(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = pick(&open_tiles);
    let puff_mat = lambert!({ color: 0xf8f8f0 });
    let cluster = group();
    let count = 3 + (rnd() * 4.0).floor() as i32;
    for i in 0..count {
        let r = 0.010 + rnd() * 0.006;
        let puff = mesh(pg("CottonBalls__puff", sphere_geo!(r, 8, 6)), &puff_mat);
        puff.position().set((rnd() - 0.5) * 0.09, r * 0.9, (rnd() - 0.5) * 0.09);
        puff.scale().y = 0.85; // slight squash for rest-on-floor read
        cluster.add(&puff);
    }
    cluster.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    g.world.add(&cluster);
}
