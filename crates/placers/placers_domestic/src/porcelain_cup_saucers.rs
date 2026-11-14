//! `place_porcelain_cup_saucers`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick_hex;
use placers_kit::place::scatter;

// Porcelain cup + saucer: 7% chance per level.
pub(crate) fn place_porcelain_cup_saucers(g: &mut Game, _dist: &[i32]) {
    g.pp.teacup_positions.clear(); // reset per level
    if rnd() < 0.93 {
        return;
    } // 7% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let porcelain_colors = [0xe8dcc0, 0xf0d8c0, 0xd8d8e0, 0xe8c0a0];
    let porcelain_mat = lambert!({ color: pick_hex(&porcelain_colors) });
    let tea_mat = lambert!({ color: 0x603820 });
    let pattern_mat = lambert!({ color: 0x4030a0 });
    let set = group();
    // Saucer (flat disc + slight rim).
    let saucer = mesh(pg("PorcelainCupSaucers__saucer", cyl_geo!(0.045, 0.045, 0.004, 14)), &porcelain_mat);
    saucer.position().y = 0.002;
    set.add(&saucer);
    let saucer_rim = mesh(pg("PorcelainCupSaucers__saucerRim", torus_geo!(0.043, 0.002, 5, 14)), &porcelain_mat);
    saucer_rim.rotation().x = PI / 2.0;
    saucer_rim.position().y = 0.005;
    set.add(&saucer_rim);
    // Cup (short cylinder).
    let cup = mesh(pg("PorcelainCupSaucers__cup", cyl_geo!(0.024, 0.020, 0.030, 12, 1, true)), &porcelain_mat);
    cup.position().y = 0.019;
    set.add(&cup);
    // Cup bottom.
    let cup_bottom = mesh(circle_geo!(0.020, 12), &porcelain_mat);
    cup_bottom.rotation().x = -PI / 2.0;
    cup_bottom.position().y = 0.005;
    set.add(&cup_bottom);
    // Cup handle (torus arc): loop in the radial plane with its gap on the cup wall.
    let handle = mesh(pg("PorcelainCupSaucers__handle", torus_geo!(0.010, 0.002, 4, 8, PI * 1.4)), &porcelain_mat);
    handle.rotation().z = -2.2;
    handle.rotation().y = 0.0;
    handle.position().set(0.028, 0.020, 0);
    set.add(&handle);
    // Tea residue (60% chance).
    if rnd() < 0.60 {
        let tea = mesh(circle_geo!(0.018, 12), &tea_mat);
        tea.rotation().x = -PI / 2.0;
        tea.position().y = 0.026;
        set.add(&tea);
    }
    // Small blue pattern band on cup (60% chance).
    if rnd() < 0.60 {
        let band = mesh(cyl_geo!(0.0245, 0.0245, 0.003, 12, 1, true), &pattern_mat);
        band.scale().set(0.935, 1, 0.935); // painted on the cup wall, not floating off it
        band.position().y = 0.010;
        set.add(&band);
    }
    scatter(g, &set, x, z, 2.4);
    g.world.add(&set);
    // Track for clink audio.
    let p = set.pos();
    g.pp.teacup_positions.push(v3(p.x, 0.02, p.z));
}
