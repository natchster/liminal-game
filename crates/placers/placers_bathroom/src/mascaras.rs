//! `place_mascaras`
use game::prelude::*;
use placers_kit::place::jitter;
use placers_kit::place::pick_open;

// Mascara wand: 7% chance per level. Tube + optional separated brush wand.
pub(crate) fn place_mascaras(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.93 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let tube_mat = lambert!({ color: 0x202020 });
    let bristle_mat = lambert!({ color: 0x101010 });
    let mascara = group();
    let wand_out = rnd() < 0.55;
    let tube = mesh(pg("Mascaras__tube", cyl_geo!(0.007, 0.007, 0.060, 8)), &tube_mat);
    tube.rotation().z = PI / 2.0;
    tube.position().y = 0.007;
    mascara.add(&tube);
    if wand_out {
        // Cap, wand and brush share one axis (local -X from the cap centre); the set is
        // turned 0.3 rad and tipped 0.166 rad so the cap and the brush rest on the floor.
        let wand_set = group();
        wand_set.position().set(0.055, 0.0101, 0.020);
        wand_set.rotation().set(0, 0.3, 0.166);
        mascara.add(&wand_set);
        let cap = mesh(pg("Mascaras__cap", cyl_geo!(0.009, 0.009, 0.015, 8)), &tube_mat);
        cap.rotation().z = PI / 2.0;
        wand_set.add(&cap);
        // Wand stem: collar 3 mm into the cap's socket.
        let wand = mesh(pg("Mascaras__wand", cyl_geo!(0.0015, 0.0015, 0.030, 4)), &tube_mat);
        wand.rotation().z = PI / 2.0;
        wand.position().set(-0.0195, 0, 0);
        wand_set.add(&wand);
        // Bristle brush, centred on the wand tip.
        let brush = mesh(pg("Mascaras__brush", sphere_geo!(0.006, 6, 5)), &bristle_mat);
        brush.scale().set(1.5, 0.7, 0.7);
        brush.position().set(-0.0345, 0, 0);
        wand_set.add(&brush);
    } else {
        let cap = mesh(pg("Mascaras__cap", cyl_geo!(0.009, 0.009, 0.015, 8)), &tube_mat);
        cap.rotation().z = PI / 2.0;
        cap.position().set(0.038, 0.007, 0);
        mascara.add(&cap);
    }
    let (px, pz) = jitter(g, x, z, 2.4);
    mascara.position().set(px, 0, pz);
    mascara.rotation().y = rnd() * PI * 2.0;
    g.world.add(&mascara);
}
