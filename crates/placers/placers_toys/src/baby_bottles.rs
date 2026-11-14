//! `place_baby_bottles`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Baby bottle: 6% chance per level. Body + nipple + cap ring.
pub(crate) fn place_baby_bottles(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let body_mat = lambert!({ color: 0xf8f0e0, transparent: true, opacity: 0.65 });
    let content_mat = lambert!({ color: 0xf0e8d8 });
    let ring_colors: [u32; 4] = [0xf0c0c0, 0xc0c0f0, 0xa0e0c0, 0xf0e0a0];
    let ring_mat = lambert!({ color: pick(&ring_colors) });
    let nipple_mat = lambert!({ color: 0xf0d8c0 });
    let bottle = group();
    // Translucent body.
    let body = mesh(cyl_geo!(0.020, 0.024, 0.075, 12), &body_mat);
    body.position().y = 0.037;
    bottle.add(&body);
    // Interior content (milk column visible through glass).
    let content = mesh(pg("BabyBottles__milk", cyl_geo!(0.017, 0.021, 0.045, 10)), &content_mat);
    content.position().y = 0.024;
    bottle.add(&content);
    // Colored cap ring on top.
    let ring = mesh(pg("BabyBottles__collar", cyl_geo!(0.022, 0.022, 0.010, 12)), &ring_mat);
    ring.position().y = 0.080;
    bottle.add(&ring);
    // Nipple on top.
    let nipple = mesh(pg("BabyBottles__teat", sphere_geo!(0.010, 6, 5)), &nipple_mat);
    nipple.scale().y = 1.4;
    nipple.position().y = 0.096;
    bottle.add(&nipple);
    // 30% tipped over.
    let is_tipped = rnd() < 0.30;
    if is_tipped {
        bottle.rotation().z = PI / 2.0;
    }
    bottle.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    bottle.rotation().y = rnd() * PI * 2.0;
    g.world.add(&bottle);
    // Milk sloshing animator (tipped bottles slosh more).
    let slphase = rnd() * PI * 2.0;
    let amp = if is_tipped { 0.06 } else { 0.02 }; // tipped bottles slosh visibly more
    g.animate(move |_g, _dt, t| {
        // Wobble the milk cylinder scale x/z slightly, and tilt scale y a bit.
        let s = 1.0 + (t * 1.8 + slphase).dsin() * amp * 0.5;
        let sy = 1.0 + (t * 1.8 + slphase + PI).dsin() * amp * 0.3;
        content.scale().set(s, sy, s);
    });
}
