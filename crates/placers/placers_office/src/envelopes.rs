//! `place_envelopes`
use crate::shared::tri_geo;
use game::prelude::*;
use placers_kit::place::jitter;
use placers_kit::place::pick_open;

// Dropped envelope: 7% chance per level. Optional letter poking out.
pub(crate) fn place_envelopes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.93 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let env_colors = [0xf0e8d0, 0xf8f4e8, 0xe8dcbc, 0xc0a898];
    let env_mat = lambert!({ color: choice(&env_colors), side: DoubleSide });
    let letter_mat = lambert!({ color: 0xffffff, side: DoubleSide });
    let stamp_mat = lambert!({ color: 0xc02040 });
    let env = group();
    let body = mesh(plane_geo!(0.14, 0.09), &env_mat);
    body.rotation().x = -PI / 2.0;
    body.position().y = 0.002;
    env.add(&body);
    // Diagonal seal flap crease (triangle overlay).
    let flap_geom = tri_geo(vec![-0.07, 0.0, -0.045, 0.07, 0.0, -0.045, 0.0, 0.0, 0.01], vec![0, 1, 2]);
    let flap = mesh(flap_geom, &env_mat);
    flap.position().y = 0.003;
    env.add(&flap);
    // Stamp corner.
    let stamp = mesh(plane_geo!(0.020, 0.024), &stamp_mat);
    stamp.rotation().x = -PI / 2.0;
    stamp.position().set(0.052, 0.004, -0.028);
    env.add(&stamp);
    // 40% chance letter poking out.
    if rnd() < 0.4 {
        let letter = mesh(plane_geo!(0.11, 0.075), &letter_mat);
        letter.rotation().x = -PI / 2.0;
        letter.position().set(0.04, 0.003, 0.020);
        letter.rotation().z = 0.15;
        env.add(&letter);
    }
    let (px, pz) = jitter(g, x, z, 2.4);
    env.position().set(px, 0, pz);
    env.rotation().y = rnd() * PI * 2.0;
    g.world.add(&env);
}
