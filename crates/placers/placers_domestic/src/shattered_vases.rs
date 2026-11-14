//! `place_shattered_vases`
use crate::shared::buffer_geo;
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick_hex;
use placers_kit::place::scatter_on_floor;

// Shattered vase: 6% chance per level. Broken pottery + upright base.
pub(crate) fn place_shattered_vases(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let pot_colors = [0xe8dcc0, 0x8a5030, 0x4a2818, 0xa02040];
    let pot_mat = lambert!({ color: pick_hex(&pot_colors) });
    let grp = group();
    // Intact base stub (bottom of vase remaining).
    let base = mesh(pg("ShatteredVases__base", cyl_geo!(0.030, 0.035, 0.020, 12)), &pot_mat);
    base.position().y = 0.010;
    grp.add(&base);
    // Broken jagged rim on base (short irregular tabs, tangent to the rim).
    for i in 0..5 {
        let a = (i as f64 / 5.0) * PI * 2.0 + rnd() * 0.4;
        let tab = mesh(pg("ShatteredVases__shard", box_geo!(0.010, 0.014 + rnd() * 0.010, 0.004)), &pot_mat);
        tab.position().set(a.cos() * 0.028, 0.024, a.sin() * 0.028);
        tab.rotation().y = PI / 2.0 - a;
        grp.add(&tab);
    }
    // 6-8 scattered shard triangles on floor.
    let n_shards = 6 + (rnd() * 3.0).floor() as i32;
    for i in 0..n_shards {
        let s = 0.020 + rnd() * 0.020;
        let shard_geo = buffer_geo(&[0.0, 0.0, 0.0, s, 0.0, s * 0.5 * (rnd() - 0.5), s * 0.5, 0.0, s * (0.5 + rnd() * 0.5)], vec![0, 1, 2]);
        let shard = mesh(shard_geo, &pot_mat);
        shard.position().set((rnd() - 0.5) * 0.30, 0.002, (rnd() - 0.5) * 0.30);
        shard.rotation().y = rnd() * PI * 2.0;
        grp.add(&shard);
    }
    scatter_on_floor(g, &grp, x, z, 2.0);
    g.world.add(&grp);
}
