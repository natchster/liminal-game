//! `place_nail_polishes`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Nail polish bottle: 8% chance per level. Small
// squat glass bottle w/ colored polish + black cap.
pub(crate) fn place_nail_polishes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    // Polish color pool — vivid + pastel mix.
    let polish_colors: [u32; 7] = [0xa02020, 0xf0a0c0, 0x203060, 0x804080, 0xd0a040, 0xf0e0e8, 0x101010];
    let polish_mat = lambert!({ color: pick(&polish_colors), transparent: true, opacity: 0.85 });
    // Glass (translucent tint over polish).
    let glass_mat = lambert!({ color: 0xd0d0e8, transparent: true, opacity: 0.35 });
    // Cap — usually black.
    let cap_mat = lambert!({ color: 0x0a0a0a });
    let bot = group();
    // Squat body — box (feels like classic OPI-style bottle).
    let body = mesh(box_geo!(0.040, 0.038, 0.040), &polish_mat);
    body.position().y = 0.019;
    bot.add(&body);
    // Glass overlay slightly larger.
    let glass = mesh(box_geo!(0.044, 0.040, 0.044), &glass_mat);
    glass.position().y = 0.020;
    bot.add(&glass);
    // Neck (short cylinder).
    let neck = mesh(pg("NailPolishes__neck", cyl_geo!(0.008, 0.010, 0.008, 8)), &cap_mat);
    neck.position().y = 0.044;
    bot.add(&neck);
    // Cap — tall cylinder.
    let cap = mesh(pg("NailPolishes__cap", cyl_geo!(0.012, 0.012, 0.036, 10)), &cap_mat);
    cap.position().y = 0.066;
    bot.add(&cap);
    // Tiny cap-top disc.
    let cap_top = mesh(circle_geo!(0.012, 10), &cap_mat);
    cap_top.rotation().x = -PI / 2.0;
    cap_top.position().y = 0.084;
    bot.add(&cap_top);
    // 30% chance: bottle tipped over.
    if rnd() < 0.3 {
        bot.rotation().z = PI / 2.0 + (rnd() - 0.5) * 0.3;
        bot.position().y = 0.022;
    }
    let px = g.wx(x) + (rnd() - 0.5) * 2.6;
    let py = bot.pos().y;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.6;
    bot.position().set(px, py, pz);
    bot.rotation().y = rnd() * PI * 2.0;
    g.world.add(&bot);
}
