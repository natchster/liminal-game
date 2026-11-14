//! `place_small_poker_chips`
use game::prelude::*;
use placers_kit::place::pick_color;
use placers_kit::place::pick_open;
use placers_kit::place::scatter_pos;

// Small poker chip stack: 8% chance per level. Stack of
// colored casino chips. Games family.
pub(crate) fn place_small_poker_chips(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let Some((x, z)) = pick_open(g) else { return };
    let chip_colors = [0xd82040, 0x2050d8, 0x30a030, 0x181008, 0xf0ece0];
    let stack = group();
    let n_chips = 4 + (rnd() * 6.0).floor() as i32;
    let chip_r = 0.019;
    let chip_t = 0.004;
    for i in 0..n_chips {
        let chip_mat = lambert!({ color: pick_color(&chip_colors) });
        let chip = mesh(pg("SmallPokerChips__chip", cyl_geo!(chip_r, chip_r, chip_t, 12)), &chip_mat);
        let cx = (rnd() - 0.5) * 0.002;
        let cy = chip_t / 2.0 + i as f64 * chip_t;
        let cz = (rnd() - 0.5) * 0.002;
        chip.position().set(cx, cy, cz);
        chip.rotation().y = rnd() * PI * 2.0;
        stack.add(&chip);
    }
    scatter_pos(g, &stack, x, z, 2.4);
    stack.rotation().y = rnd() * PI * 2.0;
    g.world.add(&stack);
}
