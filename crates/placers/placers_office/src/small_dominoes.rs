//! `place_small_dominoes`
use game::prelude::*;
use placers_kit::place::pick_open;
use placers_kit::place::scatter_pos;

// Small dominoes: 10% chance per level. 2-4 dominoes with
// pip dots scattered. Games family.
pub(crate) fn place_small_dominoes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let Some((x, z)) = pick_open(g) else { return };
    let bone_mat = lambert!({ color: 0xf0ece0 });
    let pip_mat = lambert!({ color: 0x181008 });
    let divider_mat = lambert!({ color: 0x8a7060 });
    let grp = group();
    let n_dominoes = 2 + (rnd() * 3.0).floor() as i32;
    for i in 0..n_dominoes {
        let dom = group();
        // Body: rectangle 4×2 cm.
        let body = mesh(pg("SmallDominoes__tile", box_geo!(0.040, 0.006, 0.020)), &bone_mat);
        body.position().y = 0.003;
        dom.add(&body);
        // Central divider line.
        let div = mesh(box_geo!(0.001, 0.001, 0.018), &divider_mat);
        div.position().set(0, 0.0065, 0); // resting on the tile top (y 0.006)
        dom.add(&div);
        // 2 half-face pip patterns.
        let left_count = (rnd() * 6.0).floor() as i32 + 1;
        let right_count = (rnd() * 6.0).floor() as i32 + 1;
        for (half, count) in [(-1.0, left_count), (1.0, right_count)] {
            let half_x = half * 0.010;
            for &(px, pz) in domino_pips(count) {
                let pip = mesh(circle_geo!(0.0018, 6), &pip_mat);
                pip.rotation().x = -PI / 2.0;
                pip.position().set(half_x + px, 0.0062, pz); // 0.2 mm over the tile top, not 1 mm
                dom.add(&pip);
            }
        }
        // Scatter placement.
        let dx = (rnd() - 0.5) * 0.10;
        let dz = (rnd() - 0.5) * 0.10;
        dom.position().set(dx, 0, dz);
        dom.rotation().y = rnd() * PI * 2.0;
        grp.add(&dom);
    }
    scatter_pos(g, &grp, x, z, 2.4);
    grp.rotation().y = rnd() * PI * 2.0;
    g.world.add(&grp);
}

/// The pip table of `place_small_dominoes` (half-face layouts, 1-6).
fn domino_pips(count: i32) -> &'static [(f64, f64)] {
    match count {
        1 => &[(0.0, 0.0)],
        2 => &[(-0.005, -0.004), (0.005, 0.004)],
        3 => &[(-0.005, -0.004), (0.0, 0.0), (0.005, 0.004)],
        4 => &[(-0.005, -0.004), (-0.005, 0.004), (0.005, -0.004), (0.005, 0.004)],
        5 => &[(-0.005, -0.004), (-0.005, 0.004), (0.005, -0.004), (0.005, 0.004), (0.0, 0.0)],
        _ => &[(-0.005, -0.005), (-0.005, 0.0), (-0.005, 0.005), (0.005, -0.005), (0.005, 0.0), (0.005, 0.005)],
    }
}
