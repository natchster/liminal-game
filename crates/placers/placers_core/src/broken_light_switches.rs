//! `place_broken_light_switches`
use game::prelude::*;
use placers_kit::place::mount_slid;
use placers_kit::place::wall_candidates;
use placers_kit::place::wall_xz;

// Broken wall light switch: 6% chance per level. Switch
// plate w/ hanging switch bat + exposed wires.
pub(crate) fn place_broken_light_switches(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = choice(&candidates);
    let plate_mat = lambert!({ color: 0xe8e0c8 });
    let bat_mat = lambert!({ color: 0xf0ece0 });
    let wire_colors = [0xd82040, 0x2050d8, 0xd8b040];
    let sw = group();
    // Plate.
    let plate = mesh(pg("BrokenLightSwitches__plate", box_geo!(0.060, 0.10, 0.006)), &plate_mat);
    plate.position().set(0, 0, -0.003);
    sw.add(&plate);
    // Switch bat (tilted hanging - broken).
    let bat = mesh(pg("BrokenLightSwitches__toggle", box_geo!(0.014, 0.028, 0.010)), &bat_mat);
    bat.position().set(0, 0.010, 0.005);
    bat.rotation().z = 0.6; // hanging tilted
    sw.add(&bat);
    // 2 exposed wires poking out from side.
    for i in 0..2 {
        let fi = i as f64;
        let wire_mat = lambert!({ color: choice(&wire_colors) });
        let wire = mesh(pg("BrokenLightSwitches__wire", cyl_geo!(0.001, 0.001, 0.05, 3)), &wire_mat);
        wire.position().set(-0.030, -0.020 + fi * 0.008, 0.005);
        wire.rotation().z = -0.5 + fi * 0.3;
        sw.add(&wire);
    }
    let (wall_x, wall_z) = wall_xz(g, c, 0.003);
    mount_slid(&sw, c, wall_x, wall_z, 1.30, 0.6);
    g.world.add(&sw);
}
