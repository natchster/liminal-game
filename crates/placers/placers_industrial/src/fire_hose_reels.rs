//! `place_fire_hose_reels`
use game::prelude::*;
use placers_kit::place::mount_slid;
use placers_kit::place::wall_candidates;
use placers_kit::place::wall_xz;

// Fire hose reel: 4% chance per level. Red drum reel +
// coiled hose + nozzle valve.
pub(crate) fn place_fire_hose_reels(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = choice(&candidates);
    let red_mat = lambert!({ color: 0xa02020 });
    let hose_mat = lambert!({ color: 0xf0e8d8 });
    let brass_mat = lambert!({ color: 0xc89040 });
    let reel = group();
    // Mounting plate.
    let plate = mesh(pg("FireHoseReels__plate", box_geo!(0.42, 0.42, 0.010)), &red_mat);
    plate.position().z = -0.005;
    reel.add(&plate);
    // Drum cylinder (thick).
    let drum = mesh(pg("FireHoseReels__drum", cyl_geo!(0.10, 0.10, 0.08, 12)), &red_mat);
    drum.rotation().x = PI / 2.0;
    drum.scale().set(1.2, 1, 1.2); // r 0.12: the hose coils (inner r 0.12) now wrap it (was a 2 cm gap)
    drum.position().set(0, 0, 0.040);
    reel.add(&drum);
    // Coiled hose (thick torus around drum).
    let hose = mesh(pg("FireHoseReels__hose", torus_geo!(0.14, 0.020, 6, 16)), &hose_mat);
    hose.position().set(0, 0, 0.030);
    reel.add(&hose);
    // Second wrap for depth.
    let hose2 = mesh(pg("FireHoseReels__hose", torus_geo!(0.14, 0.018, 6, 16)), &hose_mat);
    hose2.position().set(0, 0, 0.058);
    reel.add(&hose2);
    // Brass nozzle valve on side of drum.
    let nozzle = mesh(pg("FireHoseReels__nozzle", cyl_geo!(0.014, 0.010, 0.030, 6)), &brass_mat);
    nozzle.position().set(0.14, -0.14, 0.055);
    nozzle.rotation().z = 0.4;
    reel.add(&nozzle);
    let (wall_x, wall_z) = wall_xz(g, c, 0.005);
    mount_slid(&reel, c, wall_x, wall_z, 1.50, 0.4);
    g.world.add(&reel);
}
