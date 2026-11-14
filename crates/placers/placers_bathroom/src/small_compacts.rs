//! `place_small_compacts`: Small makeup compact: 8% chance per level. Round clamshell compact with mirror lid + powder pan. Dressing-table family.
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

/// Small makeup compact: 8% chance per level. Round clamshell
/// compact with mirror lid + powder pan. Dressing-table family.
pub(crate) fn place_small_compacts(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let body_colors = [0xd8b040, 0xf0d0d0, 0x201a10, 0xa02040, 0x606870];
    let body_mat = lambert!({ color: pick(&body_colors) });
    let mirror_mat = basic_mat!({ color: 0xc0d0e0 });
    let powder_mat = lambert!({ color: 0xe8c0a0 });
    let compact = group();
    let is_open = rnd() < 0.60;
    let r = 0.038;
    // Hinge (two knuckles): on the back rim when closed, joining the two halves when open.
    let hinge = mesh(pg("SmallCompacts__hinge", sphere_geo!(0.003, 6, 4)), &body_mat);
    if is_open {
        // Bottom half (powder pan).
        let bot = mesh(pg("SmallCompacts__base", cyl_geo!(r, r, 0.010, 14)), &body_mat);
        bot.position().y = 0.005;
        compact.add(&bot);
        let powder = mesh(circle_geo!(r * 0.85, 14), &powder_mat);
        powder.rotation().x = -PI / 2.0;
        powder.position().y = 0.0105;
        compact.add(&powder);
        // Top half laid open to the side.
        let top = mesh(pg("SmallCompacts__lid", cyl_geo!(r, r, 0.008, 14)), &body_mat);
        top.position().set(r * 2.05, 0.004, 0);
        compact.add(&top);
        let mir = mesh(circle_geo!(r * 0.85, 14), &mirror_mat);
        mir.rotation().x = -PI / 2.0;
        mir.position().set(r * 2.05, 0.0085, 0);
        compact.add(&mir);
        // Hinge across the gap between the two rims, pin along Z.
        hinge.rotation().y = PI / 2.0;
        hinge.position().set(r * 1.025, 0.006, 0);
        compact.add(&hinge);
    } else {
        // Closed clamshell.
        let body = mesh(pg("SmallCompacts__body", cyl_geo!(r, r, 0.018, 14)), &body_mat);
        body.position().y = 0.009;
        compact.add(&body);
        // Small hinge dot on rim.
        hinge.position().set(0, 0.018, r);
        compact.add(&hinge);
    }
    compact.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    compact.rotation().y = rnd() * PI * 2.0;
    g.world.add(&compact);
}
