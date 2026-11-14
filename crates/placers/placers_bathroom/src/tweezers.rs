//! `place_tweezers`
use game::prelude::*;
use placers_kit::place::jitter;
use placers_kit::place::pick_open;

// Tweezers: 7% chance per level. Two thin metal arms meeting at a point.
pub(crate) fn place_tweezers(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.93 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let metal_colors = [0xc0c0c0, 0xa0a0a0, 0xd0b060, 0x606060];
    let tweezer_mat = lambert!({ color: choice(&metal_colors) });
    let tweezers = group();
    for side in [-1.0f64, 1.0] {
        let arm = mesh(pg("Tweezers__arm", box_geo!(0.055, 0.002, 0.004)), &tweezer_mat);
        arm.position().set(-0.008, 0.002, side * 0.003);
        arm.rotation().y = side * 0.05; // slight taper toward point
        tweezers.add(&arm);
    }
    let base = mesh(pg("Tweezers__base", box_geo!(0.005, 0.003, 0.008)), &tweezer_mat);
    base.position().set(-0.032, 0.002, 0);
    tweezers.add(&base);
    let (px, pz) = jitter(g, x, z, 2.4);
    tweezers.position().set(px, 0, pz);
    tweezers.rotation().y = rnd() * PI * 2.0;
    g.world.add(&tweezers);
}
