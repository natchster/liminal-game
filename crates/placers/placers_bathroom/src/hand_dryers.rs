//! `place_hand_dryers`: Wall electric hand dryer: 12% chance per level. Large white box + curved bottom nozzle — restroom institutional fixture.
use crate::shared::wall_candidates;
use game::prelude::*;
use placers_kit::place::pick;

/// Wall electric hand dryer: 12% chance per level. Large white
/// box + curved bottom nozzle — restroom institutional fixture.
pub(crate) fn place_hand_dryers(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.88 {
        return;
    } // 12% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let (cx, cz, cdx, cdz) = pick(&candidates);
    let body_mat = lambert!({ color: 0xd0c8b8 });
    let nozzle_mat = lambert!({ color: 0x2a2418 });
    let dryer = group();
    // Main body: box.
    let body = mesh(pg("HandDryers__body", box_geo!(0.32, 0.30, 0.14)), &body_mat);
    body.position().y = 0.0;
    dryer.add(&body);
    // Curved bottom nozzle: short slanted box at bottom.
    let nozzle = mesh(pg("HandDryers__nozzle", box_geo!(0.12, 0.04, 0.06)), &nozzle_mat);
    nozzle.rotation().x = -0.5;
    nozzle.position().set(0, -0.17, 0.06);
    dryer.add(&nozzle);
    // Small round emergency button on right side.
    let btn = mesh(pg("HandDryers__button", cyl_geo!(0.015, 0.015, 0.005, 8)), &nozzle_mat);
    btn.rotation().x = PI / 2.0;
    btn.position().set(0.12, 0.05, 0.071);
    dryer.add(&btn);
    let wall_x = g.wx(cx) + cdx as f64 * (CELL * 0.5 - 0.07);
    let wall_z = g.wz(cz) + cdz as f64 * (CELL * 0.5 - 0.07);
    let dryer_height = 1.2 + rnd() * 0.15;
    if cdx.abs() == 1 {
        dryer.rotation().y = if cdx > 0 { -PI / 2.0 } else { PI / 2.0 };
        dryer.position().set(wall_x, dryer_height, wall_z + (rnd() - 0.5) * 0.6);
    } else {
        dryer.rotation().y = if cdz > 0 { PI } else { 0.0 };
        dryer.position().set(wall_x + (rnd() - 0.5) * 0.6, dryer_height, wall_z);
    }
    g.world.add(&dryer);
}
