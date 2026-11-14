//! `place_cotton_pads`
use game::prelude::*;
use placers_kit::place::jitter;
use placers_kit::place::pick_open;

// Cotton pads cluster: 8% chance per level. 3-6 flat circular pads.
pub(crate) fn place_cotton_pads(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let cluster = group();
    let count = 3 + (rnd() * 4.0).floor() as i32;
    for _i in 0..count {
        // Sometimes tinted (light beige if 'used', white if fresh).
        let used = rnd() < 0.4;
        let pad_mat = lambert!({ color: if used { 0xe8dcbc } else { 0xf8f4e8 } });
        let pad = mesh(pg("CottonPads__pad", cyl_geo!(0.018, 0.018, 0.003, 10)), &pad_mat);
        pad.position().set((rnd() - 0.5) * 0.13, 0.002 + rnd() * 0.003, (rnd() - 0.5) * 0.13);
        pad.rotation().z = (rnd() - 0.5) * 0.4; // slight tilt for stack
        cluster.add(&pad);
    }
    let (px, pz) = jitter(g, x, z, 2.4);
    cluster.position().set(px, 0, pz);
    g.world.add(&cluster);
}
