//! `place_towel_bars`
use game::prelude::*;
use placers_kit::place::face_wall as face_wall_yaw;
use placers_kit::place::pick;
use placers_kit::place::wall_tiles_first;

// Towel bar with hanging towel: 5% chance per level.
pub(crate) fn place_towel_bars(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let wall_tiles = wall_tiles_first(g);
    if wall_tiles.is_empty() {
        return;
    }
    let c = pick(&wall_tiles);
    let bar_mat = lambert!({ color: 0x909090 });
    let towel_colors = [0xffffff, 0xf0e0d0, 0x80a0c0, 0xe08070, 0xa0c0a0];
    let towel_mat = lambert!({ color: pick(&towel_colors), side: DoubleSide });
    let towel = group();
    let bar_y = 1.10;
    // Two bracket ends.
    for i in 0..2 {
        let bracket = mesh(pg("TowelBars__bracket", box_geo!(0.03, 0.06, 0.06)), &bar_mat);
        bracket.position().set(if i == 0 { -0.24 } else { 0.24 }, bar_y, 0.03);
        towel.add(&bracket);
    }
    // Metal bar between brackets.
    let bar = mesh(pg("TowelBars__bar", cyl_geo!(0.010, 0.010, 0.48, 8)), &bar_mat);
    bar.rotation().z = PI / 2.0;
    bar.position().set(0, bar_y, 0.06);
    towel.add(&bar);
    // Draped towel (folded over the bar, hanging down).
    let towel_cloth = mesh(plane_geo!(0.40, 0.34), &towel_mat);
    towel_cloth.position().set(0, bar_y - 0.16, 0.075);
    towel.add(&towel_cloth);
    // Slight second panel (back side).
    let towel_back = mesh(plane_geo!(0.40, 0.32), &towel_mat);
    towel_back.position().set(0, bar_y - 0.16, 0.045);
    towel.add(&towel_back);
    towel.rotation().y = face_wall_yaw(c.dx, c.dz);
    // Origin on the wall face, so the bracket rosettes (local z 0) sit on the wall.
    towel.position().set(g.wx(c.x) + c.dx as f64 * (CELL * 0.5), 0, g.wz(c.z) + c.dz as f64 * (CELL * 0.5));
    g.world.add(&towel);
}
