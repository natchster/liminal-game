//! `place_small_toy_phones`
use game::prelude::*;
use placers_kit::place::pick_color;
use placers_kit::place::pick_open;
use placers_kit::place::scatter_pos;

// Small toy phone: 8% chance per level. Toy phone base +
// handset with cord. Child-toy family.
pub(crate) fn place_small_toy_phones(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let Some((x, z)) = pick_open(g) else { return };
    let body_colors = [0xd82040, 0xd8b040, 0x2050d8, 0x30a030];
    let body_mat = lambert!({ color: pick_color(&body_colors) });
    let button_mat = lambert!({ color: 0xf0ece0 });
    let cord_mat = lambert!({ color: 0x201a10 });
    let phone = group();
    // Base: box.
    let base = mesh(pg("SmallToyPhones__base", box_geo!(0.09, 0.020, 0.07)), &body_mat);
    base.position().y = 0.010;
    phone.add(&base);
    // 3×4 dial button grid.
    for bx in 0..3 {
        for bz in 0..4 {
            let btn = mesh(pg("SmallToyPhones__button", cyl_geo!(0.005, 0.005, 0.002, 6)), &button_mat);
            btn.position().set(
                -0.020 + bx as f64 * 0.020,
                0.021,
                -0.029 + bz as f64 * 0.0125, // grid kept in front of the handset
            );
            phone.add(&btn);
        }
    }
    // Handset: curved cylinder on top of base.
    let handset = mesh(pg("SmallToyPhones__handset", cyl_geo!(0.010, 0.010, 0.075, 6)), &body_mat);
    handset.rotation().z = PI / 2.0;
    // Handset and cups rest wholly on the base top (back edge z 0.035, sides x ±0.045).
    handset.position().set(0, 0.030, 0.023);
    phone.add(&handset);
    // Handset earpiece + mouthpiece bulges.
    for s in [-1.0, 1.0] {
        let bulge = mesh(pg("SmallToyPhones__cup", sphere_geo!(0.012, 6, 5)), &body_mat);
        bulge.position().set(s * 0.033, 0.030, 0.023);
        phone.add(&bulge);
    }
    // Cord: thin curved cylinder loop.
    let cord = mesh(torus_geo!(0.010, 0.001, 4, 8, PI * 1.5), &cord_mat);
    cord.position().set(0.037, 0.022, 0.005); // in front of the earpiece cup, clear of the buttons (x <= 0.025)
    cord.rotation().z = PI / 2.0;
    phone.add(&cord);
    scatter_pos(g, &phone, x, z, 2.4);
    phone.rotation().y = rnd() * PI * 2.0;
    g.world.add(&phone);
}
