//! `place_paper_towel_dispensers`: Wall paper towel dispenser: 12% chance per level. Tall gray box + horizontal dispensing slot + small hanging paper end.
use crate::shared::wall_candidates;
use game::prelude::*;
use placers_kit::place::pick;

/// Wall paper towel dispenser: 12% chance per level. Tall
/// gray box + horizontal dispensing slot + small hanging paper end.
pub(crate) fn place_paper_towel_dispensers(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.88 {
        return;
    } // 12% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let (cx, cz, cdx, cdz) = pick(&candidates);
    let body_mat = lambert!({ color: 0xa0a098 });
    let slot_mat = basic_mat!({ color: 0x0a0806 });
    let paper_mat = lambert!({ color: 0xf0e8d0 });
    let ptd = group();
    // Body: tall box.
    let body = mesh(pg("PaperTowelDispensers__body", box_geo!(0.24, 0.34, 0.10)), &body_mat);
    body.position().y = 0.0;
    ptd.add(&body);
    // Dispensing slot: dark horizontal recess near bottom.
    let slot = mesh(plane_geo!(0.18, 0.02), &slot_mat);
    slot.position().set(0, -0.15, 0.051);
    ptd.add(&slot);
    // Small paper towel hanging out of slot: top edge inside the slot recess,
    // hanging out over its lip.
    let paper_end = mesh(pg("PaperTowelDispensers__towel", plane_geo!(0.10, 0.06)), &paper_mat);
    paper_end.position().set(0, -0.178, 0.053);
    paper_end.rotation().x = -0.15;
    ptd.add(&paper_end);
    let wall_x = g.wx(cx) + cdx as f64 * (CELL * 0.5 - 0.05);
    let wall_z = g.wz(cz) + cdz as f64 * (CELL * 0.5 - 0.05);
    let ptd_height = 1.4 + rnd() * 0.15;
    if cdx.abs() == 1 {
        ptd.rotation().y = if cdx > 0 { -PI / 2.0 } else { PI / 2.0 };
        ptd.position().set(wall_x, ptd_height, wall_z + (rnd() - 0.5) * 0.6);
    } else {
        ptd.rotation().y = if cdz > 0 { PI } else { 0.0 };
        ptd.position().set(wall_x + (rnd() - 0.5) * 0.6, ptd_height, wall_z);
    }
    g.world.add(&ptd);
}
