//! `place_small_chalk_sticks`
use game::prelude::*;
use placers_kit::place::pick_color;
use placers_kit::place::pick_open_tile;
use placers_kit::place::scatter_pos;

// Small chalk sticks: 12% chance per level. 2-4 colored
// chalk sticks scattered on floor. Playground family.
pub(crate) fn place_small_chalk_sticks(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.88 {
        return;
    } // 12% chance
    let Some((x, z)) = pick_open_tile(g) else { return };
    let chalk_colors = [0xf0ece0, 0xd0d0d0, 0xd08080, 0x80d080, 0x8080d0, 0xd0d080, 0xd080d0];
    let grp = group();
    let n_sticks = 2 + (rnd() * 3.0).floor() as i32;
    for i in 0..n_sticks {
        let chalk_mat = lambert!({ color: pick_color(&chalk_colors) });
        let chalk = mesh(pg("SmallChalkSticks__stick", cyl_geo!(0.006, 0.006, 0.055, 6)), &chalk_mat);
        chalk.rotation().z = PI / 2.0;
        let cx = (rnd() - 0.5) * 0.12;
        let cz = (rnd() - 0.5) * 0.10;
        chalk.position().set(cx, 0.006, cz);
        chalk.rotation().y = rnd() * PI * 2.0;
        grp.add(&chalk);
    }
    scatter_pos(g, &grp, x, z, 2.4);
    grp.rotation().y = rnd() * PI * 2.0;
    g.world.add(&grp);
}
