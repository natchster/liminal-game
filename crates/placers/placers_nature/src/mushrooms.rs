//! `place_mushrooms`
use game::prelude::*;
use placers_kit::place::jitter;
use placers_kit::place::pick_open;

// Mushroom cluster: 5% chance per level.
pub(crate) fn place_mushrooms(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let cap_colors = [0xa02020, 0xc08040, 0xd0b060, 0x604020, 0xf0f0e0];
    let stem_mat = lambert!({ color: 0xf0e8d0 });
    let cluster = group();
    let count = 2 + (rnd() * 4.0).floor() as i32;
    let cap_color = choice(&cap_colors);
    for _i in 0..count {
        let mushroom = group();
        let s = 0.7 + rnd() * 0.5; // scale variance
        let stem_h = 0.030 * s;
        let stem = mesh(pg("Mushrooms__stem", cyl_geo!(0.005 * s, 0.007 * s, stem_h, 6)), &stem_mat);
        stem.position().y = stem_h / 2.0;
        mushroom.add(&stem);
        let cap_mat = lambert!({ color: cap_color });
        let cap = mesh(pg("Mushrooms__cap", sphere_geo!(0.014 * s, 8, 5, 0, PI * 2.0, 0, PI / 2.0)), &cap_mat);
        cap.position().y = stem_h;
        mushroom.add(&cap);
        mushroom.position().set((rnd() - 0.5) * 0.12, 0, (rnd() - 0.5) * 0.12);
        cluster.add(&mushroom);
    }
    let (px, pz) = jitter(g, x, z, 2.5);
    cluster.position().set(px, 0, pz);
    g.world.add(&cluster);
}
