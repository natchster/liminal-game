//! `place_small_dice_pairs`
use game::prelude::*;
use placers_kit::place::pick_open;
use placers_kit::place::scatter_pos;

// Small dice pair: 10% chance per level. 2 white cubes
// with random-face pip dots. Games family.
pub(crate) fn place_small_dice_pairs(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let Some((x, z)) = pick_open(g) else { return };
    let dice_mat = lambert!({ color: 0xf0ece0 });
    let pip_mat = lambert!({ color: 0x181008 });
    let pair = group();
    for i in 0..2 {
        let die = group();
        let size = 0.028;
        let body = mesh(pg("SmallDicePairs__die", box_geo!(size, size, size)), &dice_mat);
        body.position().y = size / 2.0;
        die.add(&body);
        // Top face pips (1-6 count based on random).
        let top_count = 1 + (rnd() * 6.0).floor() as i32;
        let pip_r = 0.003;
        let off = size * 0.28;
        let pip_positions: Vec<(f64, f64)> = match top_count {
            1 => vec![(0.0, 0.0)],
            2 => vec![(-off, -off), (off, off)],
            3 => vec![(-off, -off), (0.0, 0.0), (off, off)],
            4 => vec![(-off, -off), (-off, off), (off, -off), (off, off)],
            5 => vec![(-off, -off), (-off, off), (off, -off), (off, off), (0.0, 0.0)],
            _ => vec![(-off, -off), (-off, 0.0), (-off, off), (off, -off), (off, 0.0), (off, off)],
        };
        for (px, pz) in pip_positions {
            let pip = mesh(circle_geo!(pip_r, 8), &pip_mat);
            pip.rotation().x = -PI / 2.0;
            pip.position().set(px, size + 0.0005, pz);
            die.add(&pip);
        }
        // Position side by side.
        let dx = (if i == 0 { -0.024 } else { 0.024 }) + (rnd() - 0.5) * 0.010;
        die.position().x = dx;
        let dz = (rnd() - 0.5) * 0.024;
        die.position().z = dz;
        die.rotation().y = rnd() * PI * 2.0;
        pair.add(&die);
    }
    scatter_pos(g, &pair, x, z, 2.4);
    pair.rotation().y = rnd() * PI * 2.0;
    g.world.add(&pair);
}
