//! `place_hard_hats`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::scatter;

// Safety hard hat: 8% chance per level. Yellow domed hat
// w/ brim + chin strap.
pub(crate) fn place_hard_hats(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = choice(&open);
    let hat_colors = [0xd8b040, 0xa02020, 0x2050c0, 0xf0e8d0];
    let hat_mat = lambert!({ color: choice(&hat_colors) });
    let strap_mat = lambert!({ color: 0x181008 });
    let hat = group();
    // Dome.
    let dome = mesh(pg("HardHats__dome", sphere_geo!(0.070, 12, 8, 0, PI * 2.0, 0, PI / 2.0)), &hat_mat);
    dome.position().y = 0.020;
    hat.add(&dome);
    // Brim.
    let brim = mesh(pg("HardHats__brim", cyl_geo!(0.070, 0.080, 0.010, 12)), &hat_mat);
    brim.position().y = 0.020;
    hat.add(&brim);
    // Front brim extension.
    let front = mesh(pg("HardHats__peak", box_geo!(0.030, 0.008, 0.030)), &hat_mat);
    front.position().set(0.075, 0.024, 0);
    hat.add(&front);
    // 2 side strap dots.
    for s in [-1.0, 1.0] {
        let strap = mesh(pg("HardHats__strapDot", cyl_geo!(0.006, 0.006, 0.004, 6)), &strap_mat);
        strap.rotation().z = PI / 2.0;
        strap.position().set(0, 0.025, s * 0.070);
        hat.add(&strap);
    }
    scatter(g, &hat, x, z, 2.4);
    g.world.add(&hat);
}
