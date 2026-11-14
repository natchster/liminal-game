//! `place_8_track_tapes`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// 8-track tape cartridge: 5% chance per level. Chunky blocky cartridge w/
// label + notch cutout.
pub(crate) fn place_8_track_tapes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let shell_colors: [u32; 6] = [0x2040a0, 0xa02020, 0x408040, 0xc0a040, 0x606060, 0xe0d0a0];
    let shell_mat = lambert!({ color: pick(&shell_colors) });
    let label_colors: [u32; 4] = [0xffffff, 0xf0e020, 0xd050a0, 0x40a0e0];
    let label_mat = lambert!({ color: label_colors[(rnd() * 4.0).floor() as usize] });
    let cartridge = group();
    // Chunky main body.
    let body = mesh(pg("EightTrackTapes__body", box_geo!(0.108, 0.024, 0.140)), &shell_mat);
    body.position().y = 0.012;
    cartridge.add(&body);
    // Large paper label on top.
    let label = mesh(plane_geo!(0.090, 0.110), &label_mat);
    label.rotation().x = -PI / 2.0;
    label.position().set(0, 0.025, 0);
    cartridge.add(&label);
    // Front notch (dark rectangular cutout at one edge).
    let notch = mesh(box_geo!(0.070, 0.020, 0.014), lambert!({ color: 0x0a0a0a }));
    notch.position().set(0, 0.012, 0.063);
    cartridge.add(&notch);
    // 2 small tape hub circles visible through notch.
    for side in [-1.0, 1.0] {
        let hub = mesh(pg("EightTrackTapes__hub", cyl_geo!(0.008, 0.008, 0.002, 8)), lambert!({ color: 0x606060 }));
        hub.position().set(side * 0.024, 0.023, 0.058);
        cartridge.add(&hub);
    }
    cartridge.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    cartridge.rotation().y = rnd() * PI * 2.0;
    g.world.add(&cartridge);
}
