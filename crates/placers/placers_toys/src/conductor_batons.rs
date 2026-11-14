//! `place_conductor_batons`: Small conductor's baton: 8% chance per level. Thin white baton with cork tip. Music family (conducting).
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

/// Small conductor's baton: 8% chance per level. Thin white
/// baton with cork tip. Music family (conducting).
pub(crate) fn place_conductor_batons(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let shaft_mat = lambert!({ color: 0xf0ece0 });
    let cork_mat = lambert!({ color: 0xb08050 });
    let baton = group();
    // Shaft: thin white tapered cylinder.
    let shaft = mesh(pg("ConductorBatons__shaft", cyl_geo!(0.002, 0.003, 0.30, 6)), &shaft_mat);
    shaft.rotation().z = PI / 2.0;
    shaft.position().set(0.15, 0.010, 0);
    baton.add(&shaft);
    // Cork handle at base.
    let cork = mesh(pg("ConductorBatons__cork", cyl_geo!(0.008, 0.008, 0.030, 8)), &cork_mat);
    cork.rotation().z = PI / 2.0;
    cork.position().set(0.015, 0.010, 0);
    baton.add(&cork);
    baton.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    baton.rotation().y = rnd() * PI * 2.0;
    g.world.add(&baton);
}
