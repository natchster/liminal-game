//! `place_small_hula_hoops`
use game::prelude::*;
use placers_kit::place::pick_color;
use placers_kit::place::pick_open;
use placers_kit::place::scatter_pos;

// Small hula hoop: 6% chance per level. Large colorful
// torus lying on floor. Playground family.
pub(crate) fn place_small_hula_hoops(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let Some((x, z)) = pick_open(g) else { return };
    let hoop_colors = [0xd82040, 0x2050d8, 0xd8b040, 0x30a030, 0xa020a0, 0xd050a0, 0xe07020];
    let hoop_color = pick_color(&hoop_colors);
    let hoop_mat = lambert!({ color: hoop_color });
    let stripe_colors = [0xf0ece0, 0x181008, 0xd8b040];
    let hoop = group();
    // Main torus (large: 30 cm r, 1.2 cm tube).
    let body = mesh(pg("SmallHulaHoops__hoop", torus_geo!(0.30, 0.012, 6, 24)), &hoop_mat);
    body.rotation().x = PI / 2.0;
    body.position().y = 0.012;
    hoop.add(&body);
    // Optional stripe overlay: 4 small torus segments in accent color.
    if rnd() < 0.60 {
        let stripe_mat = lambert!({ color: pick_color(&stripe_colors) });
        for i in 0..4 {
            let stripe = mesh(pg("SmallHulaHoops__stripe", torus_geo!(0.30, 0.0125, 4, 4, PI / 12.0)), &stripe_mat);
            stripe.rotation().x = PI / 2.0;
            stripe.rotation().z = (i as f64 / 4.0) * PI * 2.0;
            stripe.position().y = 0.012;
            hoop.add(&stripe);
        }
    }
    scatter_pos(g, &hoop, x, z, 1.5);
    hoop.rotation().y = rnd() * PI * 2.0;
    g.world.add(&hoop);
}
