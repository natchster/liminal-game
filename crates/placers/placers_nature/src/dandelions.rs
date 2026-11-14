//! `place_dandelions`
use game::prelude::*;
use placers_kit::place::jitter;
use placers_kit::place::pick_open;

// Dandelion puff: 6% chance per level. Fluffy white sphere w/ stem.
pub(crate) fn place_dandelions(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let stem_mat = lambert!({ color: 0x608040 });
    let puff_mat = lambert!({ color: 0xf8f8f0, transparent: true, opacity: 0.7 });
    let dandelion = group();
    let stem_h = 0.06;
    let stem = mesh(pg("Dandelions__stem", cyl_geo!(0.001, 0.001, stem_h, 4)), &stem_mat);
    stem.position().y = stem_h / 2.0;
    dandelion.add(&stem);
    let puff = mesh(sphere_geo!(0.020, 10, 8), &puff_mat);
    puff.position().y = stem_h + 0.014;
    dandelion.add(&puff);
    // 8 tiny outer wisp spikes radiating (small cylinders).
    let wisp_mat = lambert!({ color: 0xf8f8f0 });
    for i in 0..8 {
        let a = (i as f64 / 8.0) * PI * 2.0;
        let wisp = mesh(pg("Dandelions__wisp", cyl_geo!(0.0004, 0.0004, 0.010, 3)), &wisp_mat);
        // Radiate from the puff's centre: achene (-Y) 1 mm inside the puff, pappus (+Y) outward.
        let el = -0.3 + 0.9 * (a * 0.5).dsin();
        let w_dir = v3(el.dcos() * a.dcos(), el.dsin(), el.dcos() * a.dsin());
        wisp.quaternion().set_from_unit_vectors(&v3(0, 1, 0), &w_dir);
        wisp.position().set(0, stem_h + 0.014, 0).add_scaled_vector(&w_dir, 0.024);
        dandelion.add(&wisp);
    }
    let (px, pz) = jitter(g, x, z, 2.5);
    dandelion.position().set(px, 0, pz);
    dandelion.rotation().y = rnd() * PI * 2.0;
    let base_z = (rnd() - 0.5) * 0.2;
    dandelion.rotation().z = base_z; // slight lean
    g.world.add(&dandelion);
    // Gentle dandelion sway animator.
    let phase = rnd() * PI * 2.0;
    g.animate(move |_g, _dt, t| {
        dandelion.rotation().z = base_z + (t * 0.35 + phase).dsin() * 0.05;
    });
}
