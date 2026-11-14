//! `place_crumpled_tissues`
use game::prelude::*;
use placers_kit::place::jitter;
use placers_kit::place::pick_open;

// Crumpled tissue paper: 10% chance per level.
pub(crate) fn place_crumpled_tissues(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let tissue_mat = lambert!({ color: 0xf8f4e8 });
    let tissue = group();
    // Irregular ball of tissue: 3 overlapping cubes rotated randomly.
    for _i in 0..3 {
        let s = 0.020 + rnd() * 0.010;
        let chunk = mesh(pg("CrumpledTissues__chunk", box_geo!(s, s, s)), &tissue_mat);
        chunk.position().set((rnd() - 0.5) * 0.020, s / 2.0 + (rnd() - 0.5) * 0.008, (rnd() - 0.5) * 0.020);
        chunk.rotation().set(rnd() * PI, rnd() * PI, rnd() * PI);
        tissue.add(&chunk);
    }
    let (px, pz) = jitter(g, x, z, 2.5);
    tissue.position().set(px, 0, pz);
    g.world.add(&tissue);
}
