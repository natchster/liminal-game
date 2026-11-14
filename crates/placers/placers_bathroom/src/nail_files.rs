//! `place_nail_files`
use game::prelude::*;
use placers_kit::place::jitter;
use placers_kit::place::pick_open;

// Nail file (emery board): 7% chance per level.
pub(crate) fn place_nail_files(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.93 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let board_colors = [0xd050a0, 0xf0c060, 0x8060c0, 0xe0e0e0];
    let board_mat = lambert!({ color: choice(&board_colors) });
    let grain_mat = lambert!({ color: 0x606050 });
    let file = group();
    let board = mesh(pg("NailFiles__board", box_geo!(0.100, 0.003, 0.014)), &board_mat);
    board.position().y = 0.0015;
    file.add(&board);
    let grain = mesh(pg("NailFiles__grit", box_geo!(0.088, 0.001, 0.012)), &grain_mat);
    grain.position().y = 0.0035;
    file.add(&grain);
    let (px, pz) = jitter(g, x, z, 2.4);
    file.position().set(px, 0, pz);
    file.rotation().y = rnd() * PI * 2.0;
    g.world.add(&file);
}
