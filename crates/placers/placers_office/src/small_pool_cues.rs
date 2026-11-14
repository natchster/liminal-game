//! `place_small_pool_cues`
use game::prelude::*;
use placers_kit::place::pick_color;
use placers_kit::place::pick_open;
use placers_kit::place::scatter_pos;

// Small pool cue: 6% chance per level. Long tapered wooden
// cue with tip cap. Games family.
pub(crate) fn place_small_pool_cues(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let Some((x, z)) = pick_open(g) else { return };
    let shaft_colors = [0xc8a870, 0x8a6540, 0x603820, 0x201a10];
    let shaft_mat = lambert!({ color: pick_color(&shaft_colors) });
    let tip_mat = lambert!({ color: 0x2050c0 });
    let grip_mat = lambert!({ color: 0x201a10 });
    let cue = group();
    let cue_len = 0.90;
    // Shaft: tapered cylinder (thicker at butt).
    let shaft = mesh(pg("SmallPoolCues__shaft", cyl_geo!(0.006, 0.014, cue_len, 8)), &shaft_mat);
    shaft.rotation().z = PI / 2.0;
    shaft.position().set(cue_len / 2.0, 0.010, 0);
    cue.add(&shaft);
    // Tip: small blue chalk pad on far end.
    let tip = mesh(pg("SmallPoolCues__tip", cyl_geo!(0.006, 0.006, 0.008, 6)), &tip_mat);
    tip.rotation().z = PI / 2.0;
    tip.position().set(-0.004, 0.010, 0);
    cue.add(&tip);
    // Grip: dark wrap on butt end.
    let grip = mesh(pg("SmallPoolCues__grip", cyl_geo!(0.015, 0.015, 0.12, 8)), &grip_mat);
    grip.rotation().z = PI / 2.0;
    grip.position().set(cue_len - 0.06, 0.010, 0);
    cue.add(&grip);
    scatter_pos(g, &cue, x, z, 1.6);
    cue.rotation().y = rnd() * PI * 2.0;
    g.world.add(&cue);
}
