//! `place_fire_extinguishers`
use game::prelude::*;
use placers_kit::place::pick;
use placers_kit::place::wall_candidates;

// Fire extinguisher: 0-1 red cylinder mounted in bracket on a random wall.
pub(crate) fn place_fire_extinguishers(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.75 {
        return;
    } // 25% chance per level
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = pick(&candidates);
    let red_mat = lambert!({ color: 0xa02010 });
    let metal_mat = lambert!({ color: 0x3a3028 });
    let black_mat = lambert!({ color: 0x1a1410 });
    let extg = group();
    // Body: cylinder 0.09 r × 0.42 tall.
    let body = mesh(pg("FireExtinguishers__body", cyl_geo!(0.09, 0.09, 0.42, 12)), &red_mat);
    body.position().y = 0.21;
    extg.add(&body);
    // Neck/valve: smaller metal cylinder on top.
    let neck = mesh(pg("FireExtinguishers__valve", cyl_geo!(0.03, 0.05, 0.06, 8)), &metal_mat);
    neck.position().y = 0.45;
    extg.add(&neck);
    // Handle bar (horizontal short box on top of neck).
    let handle = mesh(pg("FireExtinguishers__lever", box_geo!(0.10, 0.02, 0.03)), &black_mat);
    handle.position().y = 0.485;
    extg.add(&handle);
    // Nozzle short cylinder on the front.
    let nozzle = mesh(pg("FireExtinguishers__hose", cyl_geo!(0.008, 0.008, 0.14, 6)), &black_mat);
    nozzle.rotation().z = PI / 2.0;
    nozzle.position().set(0.09, 0.32, 0);
    extg.add(&nozzle);
    // Wall bracket (small dark box behind body).
    let bracket = mesh(pg("FireExtinguishers__bracket", box_geo!(0.15, 0.12, 0.03)), &metal_mat);
    bracket.position().set(-0.09, 0.25, 0);
    extg.add(&bracket);
    // the group's front is +X (nozzle) and the bracket plate is its back
    // (local x −0.165): turn −X to the wall and put that plate on it
    let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.165);
    let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.165);
    extg.rotation().y = (c.dz as f64).atan2(-(c.dx as f64));
    if c.dx.abs() == 1 {
        extg.position().set(wall_x, 0, wall_z + (rnd() - 0.5) * 1.2);
    } else {
        extg.position().set(wall_x + (rnd() - 0.5) * 1.2, 0, wall_z);
    }
    g.world.add(&extg);
}
