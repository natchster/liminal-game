//! `place_kneeling_pads`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Garden kneeling pad: 7% chance per level. Foam pad w/ handle cutout.
pub(crate) fn place_kneeling_pads(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.93 {
        return;
    } // 7% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let pad_colors: [u32; 5] = [0x40a040, 0x408040, 0x6080c0, 0xc0a040, 0x804020];
    let pad_mat = lambert!({ color: pick(&pad_colors) });
    let pad = group();
    // Main foam pad body.
    let body = mesh(pg("KneelingPads__body", box_geo!(0.35, 0.030, 0.20)), &pad_mat);
    body.position().y = 0.015;
    pad.add(&body);
    // Handle cutout — 2 side blocks + gap (visual only).
    let cutout_mat = lambert!({ color: 0x201818 });
    let cutout = mesh(pg("KneelingPads__cutout", box_geo!(0.080, 0.032, 0.020)), &cutout_mat);
    cutout.position().set(0.115, 0.014, 0);
    pad.add(&cutout);
    pad.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    pad.rotation().y = rnd() * PI * 2.0;
    g.world.add(&pad);
}
