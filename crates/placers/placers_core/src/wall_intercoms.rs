//! `place_wall_intercoms`
use game::prelude::*;
use placers_kit::place::mount;
use placers_kit::place::wall_candidates;
use placers_kit::place::wall_xz;

// Wall intercom: 4% chance per level. Small speaker box +
// grille + button.
pub(crate) fn place_wall_intercoms(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = choice(&candidates);
    let body_mat = lambert!({ color: 0x808088 });
    let dark_mat = lambert!({ color: 0x181410 });
    let button_mat = lambert!({ color: 0xa02020 });
    let intercom = group();
    // Body plate.
    let body = mesh(pg("WallIntercoms__body", box_geo!(0.10, 0.16, 0.020)), &body_mat);
    body.position().set(0, 0, -0.010);
    intercom.add(&body);
    // Speaker grille (dark disc).
    let grille = mesh(pg("WallIntercoms__grille", circle_geo!(0.032, 12)), &dark_mat);
    grille.position().set(0, 0.03, 0.002);
    intercom.add(&grille);
    // 4 grille slot lines.
    for i in 0..4 {
        let slot = mesh(pg("WallIntercoms__slot", box_geo!(0.040, 0.002, 0.001)), &body_mat);
        slot.position().set(0, 0.02 + i as f64 * 0.008, 0.005);
        intercom.add(&slot);
    }
    // Big talk button.
    let btn = mesh(pg("WallIntercoms__button", cyl_geo!(0.010, 0.010, 0.005, 8)), &button_mat);
    btn.rotation().x = PI / 2.0;
    btn.position().set(0, -0.05, 0.004);
    intercom.add(&btn);
    let (wall_x, wall_z) = wall_xz(g, c, 0.010);
    mount(&intercom, c, wall_x, wall_z, 1.55, Some(0.6));
    g.world.add(&intercom);
}
