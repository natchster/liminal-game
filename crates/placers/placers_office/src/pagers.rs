//! `place_pagers`
use game::prelude::*;
use placers_kit::place::jitter;
use placers_kit::place::pick_open;

// Pager: 8% chance per level. Chunky body w/ tiny LCD screen + clip.
pub(crate) fn place_pagers(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let body_colors = [0x202020, 0x404040, 0x2040a0, 0x408060];
    let body_mat = lambert!({ color: choice(&body_colors) });
    let screen_mat = lambert!({ color: 0x506830 });
    let btn_mat = lambert!({ color: 0x707070 });
    let pager = group();
    let body = mesh(pg("Pagers__body", box_geo!(0.048, 0.020, 0.070)), &body_mat);
    body.position().y = 0.012; // lying on its back on the 2 mm belt clip
    pager.add(&body);
    let screen = mesh(box_geo!(0.036, 0.002, 0.020), &screen_mat);
    screen.position().set(0, 0.023, -0.020);
    pager.add(&screen);
    for i in 0..3 {
        let btn = mesh(pg("Pagers__button", cyl_geo!(0.004, 0.004, 0.002, 8)), &btn_mat);
        btn.position().set(-0.012 + i as f64 * 0.012, 0.024, 0.005);
        pager.add(&btn);
    }
    // Belt clip on back (bent metal strip).
    let clip_mat = lambert!({ color: 0x505050 });
    let clip = mesh(box_geo!(0.020, 0.002, 0.055), &clip_mat);
    clip.position().set(0, 0.001, 0);
    pager.add(&clip);
    let (px, pz) = jitter(g, x, z, 2.4);
    pager.position().set(px, 0, pz);
    pager.rotation().y = rnd() * PI * 2.0;
    g.world.add(&pager);
}
