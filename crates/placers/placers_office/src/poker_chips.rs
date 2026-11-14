//! `place_poker_chips`
use game::prelude::*;
use placers_kit::place::jitter;
use placers_kit::place::pick_open;

// Poker chip cluster: 6% chance per level. 4-8 chips in 1-3 small stacks.
pub(crate) fn place_poker_chips(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    // (body, ring)
    let chip_colors: [(u32, u32); 5] = [
        (0xc02020, 0xffffff), // red
        (0x202020, 0xffffff), // black
        (0x2060c0, 0xffffff), // blue
        (0x60c060, 0xffffff), // green
        (0xffffff, 0x202020), // white
    ];
    let cluster = group();
    let count = 4 + (rnd() * 5.0).floor() as i32;
    let stack_count = 1 + (rnd() * 3.0).floor() as i32;
    for s in 0..stack_count {
        let stack_size = count / stack_count + if s == 0 { count % stack_count } else { 0 };
        let sx = (rnd() - 0.5) * 0.10;
        let sz = (rnd() - 0.5) * 0.10;
        for c in 0..stack_size {
            let cc = choice(&chip_colors);
            let chip_mat = lambert!({ color: cc.0 });
            let ring_mat = lambert!({ color: cc.1 });
            let chip = mesh(pg("PokerChips__chip", cyl_geo!(0.015, 0.015, 0.003, 12)), &chip_mat);
            chip.position().set(sx, 0.002 + c as f64 * 0.0035, sz);
            cluster.add(&chip);
            // Contrast stripe ring on edge.
            let ring = mesh(pg("PokerChips__stripe", torus_geo!(0.015, 0.0006, 3, 12)), &ring_mat);
            ring.rotation().x = PI / 2.0;
            ring.position().set(sx, 0.002 + c as f64 * 0.0035, sz);
            cluster.add(&ring);
        }
    }
    let (px, pz) = jitter(g, x, z, 2.4);
    cluster.position().set(px, 0, pz);
    g.world.add(&cluster);
}
