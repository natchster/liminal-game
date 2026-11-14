//! `place_bath_mats`
use game::prelude::*;
use placers_kit::place::pick;
use placers_kit::place::wall_tiles_first;

// Bath mat: 6% chance per level, flat against a wall.
pub(crate) fn place_bath_mats(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    // Prefer tiles with an adjacent wall.
    let open_tiles = wall_tiles_first(g);
    if open_tiles.is_empty() {
        return;
    }
    let c = pick(&open_tiles);
    let mat_colors = [0x4060a0, 0x808080, 0xa07050, 0xc0a880, 0x805060];
    let mat_mat = lambert!({ color: pick(&mat_colors) });
    let mat = mesh(pg("BathMats__mat", box_geo!(0.60, 0.008, 0.40)), &mat_mat);
    mat.position().set(g.wx(c.x) + c.dx as f64 * 0.55, 0.004, g.wz(c.z) + c.dz as f64 * 0.55);
    // Align long side parallel to wall.
    if c.dx.abs() > 0 {
        mat.rotation().y = PI / 2.0;
    }
    g.world.add(&mat);
}
