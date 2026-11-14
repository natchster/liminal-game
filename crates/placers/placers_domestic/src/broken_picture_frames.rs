//! `place_broken_picture_frames`
use game::prelude::*;
use placers_kit::geometry::frame_bars;
use placers_kit::geometry::tri_geo;
use placers_kit::place::open_tiles;
use placers_kit::place::pick_hex;
use placers_kit::place::scatter_on_floor;

// Broken picture frame: 6% chance per level. Frame face-down + glass shards.
pub(crate) fn place_broken_picture_frames(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let frame_colors = [0x603820, 0x8a5030, 0x2a1810, 0xd8b040];
    let frame_mat = lambert!({ color: pick_hex(&frame_colors) });
    let glass_mat = basic_mat!({ color: 0xc0d0e0, transparent: true, opacity: 0.5 });
    let back_mat = lambert!({ color: 0xe8dcc0 });
    let grp = group();
    let (w, h, t, fw) = (0.22, 0.28, 0.015, 0.024);
    // Frame face-down (positive Y = back visible).
    let frame = group();
    for (bar_w, bar_h, bx, by) in frame_bars(w, h, fw) {
        let bar = mesh(pg("BrokenPictureFrames__bar", box_geo!(bar_w, bar_h, t)), &frame_mat);
        bar.position().set(bx, by, 0);
        frame.add(&bar);
    }
    // Paper backing.
    let back = mesh(plane_geo!(w, h), &back_mat);
    back.rotation().y = PI; // faces out of the back (-z)
    back.position().set(0, 0, -t / 2.0 - 0.001);
    frame.add(&back);
    // face-down as intended: front (+z) to the floor, backing on top
    frame.rotation().x = PI / 2.0;
    frame.position().y = 0.008;
    grp.add(&frame);
    // 5-8 scattered glass shards around frame.
    let n_shards = 5 + (rnd() * 4.0).floor() as i32;
    for i in 0..n_shards {
        let s1 = 0.020 + rnd() * 0.020;
        let shard_geo = tri_geo([0.0, 0.0, 0.0, s1, 0.0, s1 * (rnd() - 0.5) * 0.5, s1 * 0.5, 0.0, s1 * (0.5 + rnd() * 0.5)]);
        let shard = mesh(shard_geo, &glass_mat);
        shard.position().set((rnd() - 0.5) * 0.36, 0.002, (rnd() - 0.5) * 0.36);
        shard.rotation().y = rnd() * PI * 2.0;
        grp.add(&shard);
    }
    scatter_on_floor(g, &grp, x, z, 1.8);
    g.world.add(&grp);
}
