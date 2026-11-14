//! `place_soap_dispensers`
use game::prelude::*;
use placers_kit::place::mount_slid as mount;
use placers_kit::place::wall_candidates;
use placers_kit::place::wall_xz;

// Wall soap dispenser: 6% chance per level. Push-plunger
// wall unit w/ pink soap visible.
pub(crate) fn place_soap_dispensers(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = choice(&candidates);
    let body_mat = basic_mat!({ color: 0xf0e0e0, transparent: true, opacity: 0.75 });
    let soap_mat = basic_mat!({ color: 0xf0a080 });
    let nozzle_mat = lambert!({ color: 0x606060 });
    let dispenser = group();
    // Body (translucent).
    let body = mesh(box_geo!(0.12, 0.20, 0.08), &body_mat);
    body.position().set(0, 0, -0.04);
    dispenser.add(&body);
    // Soap inside (partial fill).
    let soap = mesh(box_geo!(0.10, 0.12, 0.06), &soap_mat);
    soap.position().set(0, -0.020, -0.04);
    dispenser.add(&soap);
    // Nozzle at bottom.
    let nozzle = mesh(pg("SoapDispensers__nozzle", cyl_geo!(0.010, 0.012, 0.020, 6)), &nozzle_mat);
    nozzle.position().set(0, -0.11, -0.010);
    dispenser.add(&nozzle);
    // Push plunger on top.
    let plunger = mesh(pg("SoapDispensers__plunger", box_geo!(0.08, 0.014, 0.06)), &nozzle_mat);
    plunger.position().set(0, 0.11, -0.04);
    dispenser.add(&plunger);
    // Origin 8 cm off the wall: the 8 cm deep body (local z -0.08..0) backs onto
    // the wall face (at 4 cm its back half was inside the wall).
    let (wall_x, wall_z) = wall_xz(g, c, 0.080);
    mount(&dispenser, c, wall_x, wall_z, 1.30, 0.4);
    g.world.add(&dispenser);
}
