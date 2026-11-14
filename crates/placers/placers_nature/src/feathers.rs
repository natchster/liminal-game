//! `place_feathers`
use game::prelude::*;
use placers_kit::place::jitter;
use placers_kit::place::pick_open;

// Fallen bird feather: 7% chance per level.
pub(crate) fn place_feathers(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.93 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let feather_colors = [0xf8f4e8, 0x606060, 0x203020, 0x704020, 0x202020];
    let feather_mat = lambert!({ color: choice(&feather_colors), side: DoubleSide });
    let shaft_mat = lambert!({ color: 0xe8e0d0 });
    let feather = group();
    let shaft = mesh(pg("Feathers__shaft", cyl_geo!(0.001, 0.0015, 0.10, 4)), &shaft_mat);
    shaft.rotation().z = PI / 2.0;
    shaft.position().y = 0.002;
    feather.add(&shaft);
    // Vane (elongated ellipse: a scaled circle geometry).
    let vane = mesh(circle_geo!(0.040, 8), &feather_mat);
    vane.rotation().x = -PI / 2.0;
    vane.scale().set(1.2, 0.35, 1);
    vane.position().set(0.005, 0.003, 0);
    feather.add(&vane);
    let (px, pz) = jitter(g, x, z, 2.5);
    feather.position().set(px, 0, pz);
    feather.rotation().y = rnd() * PI * 2.0;
    feather.rotation().z = (rnd() - 0.5) * 0.4; // slight curl
    g.world.add(&feather);
}
