//! `place_hair_claws`
use game::prelude::*;
use placers_kit::place::jitter;
use placers_kit::place::pick_open;

// Hair claw clip: 7% chance per level. Two-jaw claw shape on floor.
pub(crate) fn place_hair_claws(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.93 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let claw_colors = [0x202020, 0xd050a0, 0xa0a0e0, 0xc0a060, 0x408060];
    let claw_mat = lambert!({ color: choice(&claw_colors) });
    let clip = group();
    let bottom = mesh(pg("HairClaws__jaw", box_geo!(0.048, 0.012, 0.024)), &claw_mat);
    bottom.position().y = 0.006;
    clip.add(&bottom);
    let top = mesh(pg("HairClaws__topJaw", box_geo!(0.044, 0.010, 0.022)), &claw_mat);
    top.position().set(0.002, 0.018, 0);
    clip.add(&top);
    // 4-6 comb teeth on bottom jaw front edge.
    let teeth_count = 4 + (rnd() * 3.0).floor() as i32;
    for i in 0..teeth_count {
        let tooth = mesh(pg("HairClaws__tooth", box_geo!(0.003, 0.004, 0.010)), &claw_mat);
        // Long axis along +X, sticking 8 mm out of the jaw's front rim.
        tooth.rotation().y = PI / 2.0;
        tooth.position().set(0.027, 0.008, -0.011 + i as f64 * (0.022 / (teeth_count - 1) as f64));
        clip.add(&tooth);
    }
    // Hinge pin at rear.
    let hinge = mesh(pg("HairClaws__hinge", cyl_geo!(0.003, 0.003, 0.024, 6)), lambert!({ color: 0xc0c0c0 }));
    hinge.rotation().x = PI / 2.0;
    hinge.position().set(-0.020, 0.012, 0);
    clip.add(&hinge);
    let (px, pz) = jitter(g, x, z, 2.4);
    clip.position().set(px, 0, pz);
    clip.rotation().y = rnd() * PI * 2.0;
    g.world.add(&clip);
}
