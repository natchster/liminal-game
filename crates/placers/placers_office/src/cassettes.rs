//! `place_cassettes`
use game::prelude::*;
use placers_kit::place::jitter;
use placers_kit::place::pick_open;

// Cassette tape: 6% chance per level. Shell w/ 2 reels + label.
pub(crate) fn place_cassettes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let shell_colors = [0x202020, 0xf0e8d0, 0x804060, 0x408060, 0xc0a040];
    let shell_mat = lambert!({ color: choice(&shell_colors) });
    let reel_mat = lambert!({ color: 0x101010 });
    let tape_mat = lambert!({ color: 0x402010 });
    let label_colors = [0xffffff, 0xf8d040, 0xd0a0e0, 0x80c0e0];
    let label_mat = lambert!({ color: label_colors[(rnd() * 4.0).floor() as usize] });
    let cassette = group();
    let shell = mesh(pg("Cassettes__shell", box_geo!(0.100, 0.006, 0.062)), &shell_mat);
    shell.position().y = 0.003;
    cassette.add(&shell);
    let label = mesh(plane_geo!(0.084, 0.030), &label_mat);
    label.rotation().x = -PI / 2.0;
    label.position().set(0, 0.007, 0.008);
    cassette.add(&label);
    // 2 reels.
    for side in [-1.0f64, 1.0] {
        let reel = mesh(pg("Cassettes__reel", cyl_geo!(0.017, 0.017, 0.001, 12)), &reel_mat);
        reel.position().set(side * 0.024, 0.007, -0.012);
        cassette.add(&reel);
        // Small brown tape strip between reels.
        if side == 1.0 {
            let tape = mesh(box_geo!(0.048, 0.001, 0.006), &tape_mat);
            tape.position().set(0, 0.007, -0.024);
            cassette.add(&tape);
        }
        // 6 sprocket teeth around each reel hub.
        for t in 0..6 {
            let a = (t as f64 / 6.0) * PI * 2.0;
            let tooth = mesh(box_geo!(0.002, 0.001, 0.002), &reel_mat);
            tooth.position().set(side * 0.024 + a.cos() * 0.006, 0.008, -0.012 + a.sin() * 0.006);
            cassette.add(&tooth);
        }
    }
    let (px, pz) = jitter(g, x, z, 2.4);
    cassette.position().set(px, 0, pz);
    cassette.rotation().y = rnd() * PI * 2.0;
    g.world.add(&cassette);
}
