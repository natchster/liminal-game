//! `place_cutting_boards`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Cutting board: 6% chance per level. Wooden board
// w/ hole handle.
pub(crate) fn place_cutting_boards(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let wood_colors: [u32; 4] = [0xc0a070, 0x8b5a2b, 0xa08050, 0xd0b080];
    let board_mat = lambert!({ color: pick(&wood_colors) });
    let board = group();
    let plank = mesh(pg("CuttingBoards__plank", box_geo!(0.30, 0.018, 0.20)), &board_mat);
    plank.position().y = 0.009;
    board.add(&plank);
    let hole = mesh(cyl_geo!(0.012, 0.012, 0.022, 8), lambert!({ color: 0x201810 }));
    hole.scale().y = 0.8; // 0..0.0176: just under the board top (0.018), not proud of it
    hole.position().set(-0.130, 0.0088, 0);
    board.add(&hole);
    for i in 0..3 {
        let score = mesh(box_geo!(0.10 + rnd() * 0.06, 0.001, 0.001), lambert!({ color: 0x503020 }));
        score.position().set(0.02 + i as f64 * 0.02, 0.019, -0.06 + i as f64 * 0.03);
        score.rotation().y = (rnd() - 0.5) * 0.2;
        board.add(&score);
    }
    board.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    board.rotation().y = rnd() * PI * 2.0;
    g.world.add(&board);
}
