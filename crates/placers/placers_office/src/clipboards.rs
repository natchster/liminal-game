//! `place_clipboards`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::shuffle;

// Clipboards: 0-2 dropped clipboards per level (30% chance).
pub(crate) fn place_clipboards(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.70 {
        return;
    } // 30% chance
    let count = 1 + (rnd() * 2.0).floor() as usize;
    let mut open_tiles = open_tiles(g);
    shuffle(&mut open_tiles);
    let board_mat = lambert!({ color: 0x9a7048 });
    let paper_mat = lambert!({ color: 0xece0d0 });
    let clip_mat = lambert!({ color: 0x606060 });
    let mut i = 0;
    while i < count && i < open_tiles.len() {
        let (x, z) = open_tiles[i];
        let clipboard = group();
        // Board: 0.20 × 0.28 × 0.008 flat rectangle.
        let board = mesh(pg("Clipboards__board", box_geo!(0.20, 0.008, 0.28)), &board_mat);
        board.position().y = 0.004;
        clipboard.add(&board);
        // Paper on top (slightly smaller).
        let paper = mesh(pg("Clipboards__paper", plane_geo!(0.18, 0.24)), &paper_mat);
        paper.rotation().x = -PI / 2.0;
        paper.position().y = 0.009;
        clipboard.add(&paper);
        // Wire clip at top edge.
        let clip = mesh(pg("Clipboards__clip", box_geo!(0.06, 0.012, 0.02)), &clip_mat);
        clip.position().set(0, 0.014, -0.11);
        clipboard.add(&clip);
        let ox = (rnd() - 0.5) * 2.4;
        let oz = (rnd() - 0.5) * 2.4;
        clipboard.position().x = g.wx(x) + ox;
        clipboard.position().z = g.wz(z) + oz;
        clipboard.rotation().y = rnd() * PI * 2.0;
        g.world.add(&clipboard);
        // 30% chance a pen is dropped next to the clipboard.
        if rnd() < 0.30 {
            let pen_mat = lambert!({ color: 0x1a1a1a });
            let pen = mesh(pg("Clipboards__pen", cyl_geo!(0.004, 0.004, 0.11, 6)), &pen_mat);
            pen.rotation().z = PI / 2.0;
            pen.rotation().y = rnd() * PI * 2.0;
            // Position within 0.20 m of clipboard.
            let p_dx = (rnd() - 0.5) * 0.4;
            let p_dz = (rnd() - 0.5) * 0.4;
            let cp = clipboard.pos();
            pen.position().set(cp.x + p_dx, 0.004, cp.z + p_dz);
            g.world.add(&pen);
        }
        i += 1;
    }
}
