//! `place_missing_photo_hooks`
use game::prelude::*;
use placers_kit::place::shuffle;
use placers_kit::place::wall_candidates;

// Missing photo hook: 25% chance per level, 1-2 hooks with a faded
// rectangle where a photo used to hang.
pub(crate) fn place_missing_photo_hooks(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.75 {
        return;
    } // 25% chance
    let count = 1 + (rnd() * 2.0).floor() as usize;
    let mut candidates = wall_candidates(g);
    shuffle(&mut candidates);
    let faded_mat = basic_mat!({ color: 0xe0d8c0, transparent: true, opacity: 0.45, depthWrite: false, side: DoubleSide });
    let hook_mat = lambert!({ color: 0x2a2018 });
    let mut i = 0;
    while i < count && i < candidates.len() {
        let c = candidates[i];
        // Faded rectangle showing photo absence.
        let rect_w = 0.22 + rnd() * 0.10;
        let rect_h = 0.28 + rnd() * 0.10;
        let faded = mesh(plane_geo!(rect_w, rect_h), &faded_mat);
        // Tiny hook (small dark box near top edge of faded rectangle).
        let hook = mesh(pg("MissingPhotoHooks__hook", box_geo!(0.015, 0.02, 0.012)), &hook_mat);
        let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.008);
        let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.008);
        let center_h = 1.5 + rnd() * 0.5; // 1.5 - 2.0 m
        if c.dx.abs() == 1 {
            let rot = if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 };
            faded.rotation().y = rot;
            hook.rotation().y = rot;
            let jit_z = (rnd() - 0.5) * 1.2;
            faded.position().set(wall_x, center_h, wall_z + jit_z);
            hook.position().set(wall_x + c.dx as f64 * 0.008, center_h + rect_h * 0.42, wall_z + jit_z);
        } else {
            let rot = if c.dz > 0 { PI } else { 0.0 };
            faded.rotation().y = rot;
            hook.rotation().y = rot;
            let jit_x = (rnd() - 0.5) * 1.2;
            faded.position().set(wall_x + jit_x, center_h, wall_z);
            hook.position().set(wall_x + jit_x, center_h + rect_h * 0.42, wall_z + c.dz as f64 * 0.008);
        }
        g.world.add(&faded);
        g.world.add(&hook);
        i += 1;
    }
}
