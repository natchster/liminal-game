//! `place_dropped_clipboards`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::scatter;

// Dropped clipboard: 8% chance per level. Wooden clipboard
// + white paper w/ ink lines + clip.
pub(crate) fn place_dropped_clipboards(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = choice(&open);
    let board_mat = lambert!({ color: 0x8a5030 });
    let paper_mat = lambert!({ color: 0xf0ece0 });
    let clip_mat = lambert!({ color: 0xc0c0c0 });
    let ink_mat = lambert!({ color: 0x181410 });
    let clipboard = group();
    // Board.
    let board = mesh(pg("DroppedClipboards__board", box_geo!(0.20, 0.006, 0.28)), &board_mat);
    board.position().y = 0.003;
    clipboard.add(&board);
    // Paper on top.
    let paper = mesh(plane_geo!(0.18, 0.26), &paper_mat);
    paper.rotation().x = -PI / 2.0;
    paper.position().y = 0.007;
    clipboard.add(&paper);
    // Metal clip at top.
    let clip = mesh(pg("DroppedClipboards__clip", box_geo!(0.10, 0.008, 0.030)), &clip_mat);
    clip.position().set(0, 0.010, -0.115);
    clipboard.add(&clip);
    // 8 ink lines on paper.
    for i in 0..8 {
        let line = mesh(plane_geo!(0.14, 0.001), &ink_mat);
        line.rotation().x = -PI / 2.0;
        line.position().set(0, 0.008, -0.10 + i as f64 * 0.024);
        clipboard.add(&line);
    }
    scatter(g, &clipboard, x, z, 2.0);
    g.world.add(&clipboard);
}
