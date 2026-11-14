//! `place_contact_lens_cases`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Contact lens case: 7% chance per level, L/R colored caps.
pub(crate) fn place_contact_lens_cases(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.93 {
        return;
    } // 7% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = pick(&open_tiles);
    let base_mat = lambert!({ color: 0xf0f0e8 });
    // L=blue R=green (classic contact-lens color convention).
    let l_mat = lambert!({ color: 0x40a0e0 });
    let r_mat = lambert!({ color: 0x40c060 });
    let c_case = group();
    // Base plate.
    let base = mesh(pg("ContactLensCases__base", box_geo!(0.070, 0.012, 0.036)), &base_mat);
    base.position().y = 0.006;
    c_case.add(&base);
    // Left cap.
    let l_cap = mesh(pg("ContactLensCases__cap", cyl_geo!(0.013, 0.013, 0.010, 10)), &l_mat);
    l_cap.position().set(-0.016, 0.017, 0);
    c_case.add(&l_cap);
    // Right cap.
    let r_cap = mesh(pg("ContactLensCases__cap", cyl_geo!(0.013, 0.013, 0.010, 10)), &r_mat);
    r_cap.position().set(0.016, 0.017, 0);
    c_case.add(&r_cap);
    c_case.position().set(g.wx(x) + (rnd() - 0.5) * 2.5, 0, g.wz(z) + (rnd() - 0.5) * 2.5);
    c_case.rotation().y = rnd() * PI * 2.0;
    g.world.add(&c_case);
}
