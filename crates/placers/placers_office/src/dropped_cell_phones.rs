//! `place_dropped_cell_phones`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::scatter;

// Dropped cell phone: 10% chance per level. Small
// rectangular phone (flip or slab).
pub(crate) fn place_dropped_cell_phones(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = choice(&open);
    let body_colors = [0x181410, 0xa02040, 0x2050c0, 0xd8b040];
    let body_mat = lambert!({ color: choice(&body_colors) });
    let screen_mat = basic_mat!({ color: 0x101820 });
    let phone = group();
    let is_flip = rnd() < 0.5;
    if is_flip {
        // Flip-phone: 2 halves stacked.
        let bottom = mesh(pg("DroppedCellPhones__flipBase", box_geo!(0.030, 0.006, 0.055)), &body_mat);
        bottom.position().y = 0.003;
        phone.add(&bottom);
        let top = mesh(pg("DroppedCellPhones__flipLid", box_geo!(0.030, 0.006, 0.055)), &body_mat);
        top.position().y = 0.009;
        phone.add(&top);
        // Keypad dots.
        for ky in 0..3 {
            for kx in 0..3 {
                let key = mesh(cyl_geo!(0.002, 0.002, 0.001, 4), &screen_mat);
                key.position().set(-0.008 + kx as f64 * 0.008, 0.007, 0.006 + ky as f64 * 0.008);
                phone.add(&key);
            }
        }
    } else {
        // Slab phone.
        let body = mesh(pg("DroppedCellPhones__slab", box_geo!(0.040, 0.008, 0.075)), &body_mat);
        body.position().y = 0.004;
        phone.add(&body);
        // Dark screen.
        let screen = mesh(plane_geo!(0.032, 0.055), &screen_mat);
        screen.rotation().x = -PI / 2.0;
        screen.position().y = 0.009;
        phone.add(&screen);
    }
    scatter(g, &phone, x, z, 2.4);
    g.world.add(&phone);
}
