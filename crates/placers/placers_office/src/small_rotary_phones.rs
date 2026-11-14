//! `place_small_rotary_phones`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Small rotary phone: 6% chance per level.
pub(crate) fn place_small_rotary_phones(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = pick(&open_tiles);
    let body_colors = [0x181008, 0x502818, 0xa02040];
    let body_mat = lambert!({ color: pick(&body_colors) });
    let dial_mat = lambert!({ color: 0xc0c0c0 });
    let cord_mat = lambert!({ color: 0x201a10 });
    let phone = group();
    // Base body.
    let base = mesh(pg("SmallRotaryPhones__base", box_geo!(0.13, 0.05, 0.10)), &body_mat);
    base.position().y = 0.025;
    phone.add(&base);
    // Rotary dial (disc on top).
    let dial = mesh(pg("SmallRotaryPhones__dial", cyl_geo!(0.028, 0.028, 0.006, 14)), &dial_mat);
    dial.position().set(0, 0.053, 0.008);
    phone.add(&dial);
    // 10 dial finger holes.
    for i in 0..10 {
        let a = (i as f64 / 10.0) * PI * 2.0;
        let hole = mesh(pg("SmallRotaryPhones__hole", cyl_geo!(0.003, 0.003, 0.008, 5)), &body_mat);
        hole.position().set(a.dcos() * 0.020, 0.055, 0.008 + a.dsin() * 0.020);
        phone.add(&hole);
    }
    // 2 handset cradle posts.
    for s in [-1.0f64, 1.0] {
        let post = mesh(pg("SmallRotaryPhones__post", cyl_geo!(0.008, 0.010, 0.014, 6)), &body_mat);
        post.position().set(s * 0.045, 0.056, -0.028);
        phone.add(&post);
    }
    // Handset sitting on cradle.
    let handset = mesh(pg("SmallRotaryPhones__handset", cyl_geo!(0.010, 0.010, 0.11, 8)), &body_mat);
    handset.rotation().z = PI / 2.0;
    handset.position().set(0, 0.070, -0.028);
    phone.add(&handset);
    // Ear/mouth cups.
    for s in [-1.0f64, 1.0] {
        let cup = mesh(pg("SmallRotaryPhones__cup", cyl_geo!(0.014, 0.012, 0.010, 8)), &body_mat);
        cup.rotation().z = PI / 2.0;
        cup.position().set(s * 0.048, 0.070, -0.028);
        phone.add(&cup);
    }
    // Curly cord (torus loop).
    let cord = mesh(pg("SmallRotaryPhones__cord", torus_geo!(0.020, 0.002, 4, 12, PI * 1.5)), &cord_mat);
    cord.rotation().x = PI / 2.0;
    cord.rotation().z = PI / 2.0;
    cord.position().set(0.070, 0.030, -0.028);
    phone.add(&cord);
    phone.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    phone.rotation().y = rnd() * PI * 2.0;
    g.world.add(&phone);
}
