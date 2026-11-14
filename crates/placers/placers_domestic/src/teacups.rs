//! `place_teacups`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Teacup + saucer: 7% chance per level.
pub(crate) fn place_teacups(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.93 {
        return;
    } // 7% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let cup_colors: [u32; 6] = [0xffffff, 0xf0e0d0, 0x8080a0, 0xd050a0, 0xc0e0a0, 0xa0d0e0];
    let cup_mat = lambert!({ color: pick(&cup_colors) });
    let cup = group();
    let saucer = mesh(pg("Teacups__saucer", cyl_geo!(0.055, 0.050, 0.006, 14)), &cup_mat);
    saucer.position().y = 0.003;
    cup.add(&saucer);
    let indent = mesh(cyl_geo!(0.028, 0.028, 0.002, 12), lambert!({ color: 0xf0e0d0 }));
    indent.position().y = 0.007;
    cup.add(&indent);
    let body = mesh(pg("Teacups__cup", cyl_geo!(0.028, 0.022, 0.032, 12)), &cup_mat);
    body.position().y = 0.022;
    cup.add(&body);
    let handle = mesh(pg("Teacups__handle", torus_geo!(0.012, 0.003, 4, 8, PI)), &cup_mat);
    // C looping out from the cup wall (+x), both ends on the wall.
    handle.rotation().z = -PI / 2.0;
    handle.position().set(0.0245, 0.022, 0);
    cup.add(&handle);
    if rnd() < 0.6 {
        let tea_mat = lambert!({ color: 0x40200a });
        let tea = mesh(pg("Teacups__tea", cyl_geo!(0.024, 0.024, 0.003, 10)), &tea_mat);
        tea.position().y = 0.036;
        cup.add(&tea);
    }
    cup.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    cup.rotation().y = rnd() * PI * 2.0;
    g.world.add(&cup);
    let p = cup.pos();
    g.pp.teacup_positions.push(v3(p.x, 0, p.z));
}
