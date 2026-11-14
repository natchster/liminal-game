//! `place_small_castanets`: Small castanets: 8% chance per level. Pair of clamshell castanets. Music family (percussion).
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

/// Small castanets: 8% chance per level. Pair of clamshell
/// castanets. Music family (percussion).
pub(crate) fn place_small_castanets(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let shell_colors = [0x8a3820, 0x4a2818, 0x201a10, 0xd8b820];
    let shell_mat = lambert!({ color: pick(&shell_colors) });
    let cord_mat = lambert!({ color: 0xf0e0c8 });
    let pair = group();
    for i in 0..2 {
        let cast = group();
        // 2 half-spheres (top + bottom shell) facing each other, hollows together.
        let top = mesh(pg("SmallCastanets__shell", sphere_geo!(0.030, 10, 4, 0.0, PI * 2.0, 0.0, PI / 2.0)), &shell_mat);
        top.scale().y = 0.6;
        top.position().y = 0.019; // dome up, hollow face down onto the bottom shell
        cast.add(&top);
        let bot = mesh(pg("SmallCastanets__shell", sphere_geo!(0.030, 10, 4, 0.0, PI * 2.0, 0.0, PI / 2.0)), &shell_mat);
        bot.rotation().x = PI; // dome down on the floor, hollow face up
        bot.scale().y = 0.6;
        bot.position().y = 0.018;
        cast.add(&bot);
        // Cord loop, wrapped round the two rims at the ear (+x).
        let lp = mesh(torus_geo!(0.006, 0.001, 4, 8), &cord_mat);
        lp.position().set(0.030, 0.0185, 0);
        cast.add(&lp);
        // Position side by side.
        cast.position().x = (if i == 0 { -0.040 } else { 0.040 }) + (rnd() - 0.5) * 0.02;
        cast.position().z = (rnd() - 0.5) * 0.02;
        cast.rotation().y = rnd() * PI * 2.0;
        pair.add(&cast);
    }
    pair.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    pair.rotation().y = rnd() * PI * 2.0;
    g.world.add(&pair);
}
