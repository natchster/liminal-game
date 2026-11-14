//! `place_small_potions`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick_hex;
use placers_kit::place::pick_tile;

// Small potion bottle: 8% chance per level. Glass vial
// with colored liquid + cork stopper. Halloween/mystical family.
pub(crate) fn place_small_potions(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick_tile(&open);
    let glass_mat = basic_mat!({ color: 0xc0e0d0, transparent: true, opacity: 0.35 });
    let potion_colors: [u32; 5] = [0x30d040, 0xa020d0, 0xd82040, 0x2050d8, 0xd8b040];
    let potion_color = pick_hex(&potion_colors);
    let potion_mat = basic_mat!({ color: potion_color, transparent: true, opacity: 0.85 });
    let cork_mat = lambert!({ color: 0xa87050 });
    let potion = group();
    // Body: round-shouldered bottle.
    let body = mesh(sphere_geo!(0.024, 10, 8, 0, PI * 2.0, 0.4, PI - 0.4), &glass_mat);
    body.position().y = 0.028;
    potion.add(&body);
    // Neck.
    let neck = mesh(cyl_geo!(0.008, 0.008, 0.020, 6), &glass_mat);
    neck.position().y = 0.056;
    potion.add(&neck);
    // Cork.
    let cork = mesh(pg("SmallPotions__cork", cyl_geo!(0.007, 0.007, 0.010, 6)), &cork_mat);
    cork.position().y = 0.070;
    potion.add(&cork);
    // Colored liquid inside (smaller sphere).
    let liquid = mesh(sphere_geo!(0.018, 8, 6), &potion_mat);
    liquid.position().y = 0.026;
    liquid.scale().y = 0.7;
    potion.add(&liquid);
    // 15% chance tipped.
    if rnd() < 0.15 {
        potion.rotation().z = PI / 2.0;
        potion.position().y = 0.018;
    }
    potion.position().x = g.wx(x) + (rnd() - 0.5) * 2.4;
    potion.position().z = g.wz(z) + (rnd() - 0.5) * 2.4;
    potion.rotation().y = rnd() * PI * 2.0;
    g.world.add(&potion);
}
