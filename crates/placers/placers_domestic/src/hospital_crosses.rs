//! `place_hospital_crosses`
use game::prelude::*;
use placers_kit::place::mount_on_wall;
use placers_kit::place::wall_candidates;

// Hospital cross: 4% chance per level. Red + cross box wall-mounted.
pub(crate) fn place_hospital_crosses(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = candidates[(rnd() * candidates.len() as f64).floor() as usize];
    let white_mat = basic_mat!({ color: 0xf0ece0 });
    let red_mat = basic_mat!({ color: 0xd82030 });
    let cross = group();
    let s = 0.14;
    // White backing.
    let back = mesh(pg("HospitalCrosses__back", box_geo!(s, s, 0.010)), &white_mat);
    back.position().z = -0.005;
    cross.add(&back);
    // Red vertical bar.
    let v_bar = mesh(pg("HospitalCrosses__vbar", box_geo!(s * 0.3, s * 0.9, 0.008)), &red_mat);
    v_bar.position().z = 0.002;
    cross.add(&v_bar);
    // Red horizontal bar.
    let h_bar = mesh(pg("HospitalCrosses__hbar", box_geo!(s * 0.9, s * 0.3, 0.008)), &red_mat);
    h_bar.position().z = 0.002;
    cross.add(&h_bar);
    mount_on_wall(g, &cross, &c, 0.005, 2.0, 0.6);
    g.world.add(&cross);
}
