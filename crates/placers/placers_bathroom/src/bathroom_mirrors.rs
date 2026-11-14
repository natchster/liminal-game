//! `place_bathroom_mirrors`
use game::prelude::*;
use placers_kit::place::mount_on_wall;
use placers_kit::place::wall_candidates;

// Bathroom mirror: 5% chance per level.
pub(crate) fn place_bathroom_mirrors(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = candidates[(rnd() * candidates.len() as f64).floor() as usize];
    let frame_mat = lambert!({ color: 0xc0c8d0 });
    let mirror_mat = basic_mat!({ color: 0x506878 });
    let mirror = group();
    let (w, h, t, fw) = (0.30, 0.40, 0.012, 0.014);
    // Frame.
    for (bar_w, bar_h, bx, by) in [
        (w + fw * 2.0, fw, 0.0, h / 2.0 + fw / 2.0),
        (w + fw * 2.0, fw, 0.0, -h / 2.0 - fw / 2.0),
        (fw, h, -w / 2.0 - fw / 2.0, 0.0),
        (fw, h, w / 2.0 + fw / 2.0, 0.0),
    ] {
        let bar = mesh(pg("BathroomMirrors__bar", box_geo!(bar_w, bar_h, t)), &frame_mat);
        bar.position().set(bx, by, 0);
        mirror.add(&bar);
    }
    // Mirror face.
    let face = mesh(plane_geo!(w, h), &mirror_mat);
    face.position().set(0, 0, t / 2.0 + 0.001);
    mirror.add(&face);
    mount_on_wall(g, &mirror, &c, 0.008, 1.55, 0.6);
    g.world.add(&mirror);
}
