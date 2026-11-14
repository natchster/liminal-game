//! `place_lanyards`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::scatter;

// Office lanyard w/ badge: 8% chance per level. Colored
// strap loop + ID badge card.
pub(crate) fn place_lanyards(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = choice(&open);
    let strap_colors = [0xa02040, 0x2050c0, 0x30a030, 0x181410, 0xd8b040];
    let strap_mat = lambert!({ color: choice(&strap_colors), side: DoubleSide });
    let badge_mat = lambert!({ color: 0xf0ece0 });
    let ink_mat = lambert!({ color: 0x181410 });
    let lanyard = group();
    // Loop (half-torus lying flat).
    let lp = mesh(torus_geo!(0.06, 0.003, 4, 12, PI * 1.4), &strap_mat);
    lp.rotation().x = -PI / 2.0;
    lp.position().y = 0.003;
    lanyard.add(&lp);
    // Strap extending down to badge.
    let strap = mesh(plane_geo!(0.008, 0.06), &strap_mat);
    // run along +X from the loop's open end (x 0.06, z 0) to the badge's
    // slotted end (x 0.12): it lay across Z, touching neither
    strap.rotation().set(-PI / 2.0, 0, PI / 2.0);
    strap.position().set(0.093, 0.003, 0);
    lanyard.add(&strap);
    // Badge card.
    let badge = mesh(pg("Lanyards__badge", box_geo!(0.06, 0.001, 0.09)), &badge_mat);
    // long axis along +X, its slotted −Z end turned to the strap (spans x 0.12…0.21)
    badge.rotation().y = PI / 2.0;
    badge.position().set(0.165, 0.001, 0);
    lanyard.add(&badge);
    // 3 ink lines on badge.
    for i in 0..3 {
        let line = mesh(plane_geo!(0.045, 0.003), &ink_mat);
        // turned with the badge: across its width (z), stepping along its length (x)
        line.rotation().set(-PI / 2.0, 0, PI / 2.0);
        line.position().set(0.165 - 0.03 + i as f64 * 0.020, 0.0016, 0);
        lanyard.add(&line);
    }
    scatter(g, &lanyard, x, z, 2.0);
    g.world.add(&lanyard);
}
