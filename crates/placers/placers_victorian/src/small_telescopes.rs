//! `place_small_telescopes`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Small brass telescope: 6% chance per level.
pub(crate) fn place_small_telescopes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = pick(&open_tiles);
    let brass_mat = lambert!({ color: 0xc89040 });
    let leather_mat = lambert!({ color: 0x6a3820 });
    let lens_mat = basic_mat!({ color: 0x304050, transparent: true, opacity: 0.75 });
    let scope = group();
    // 3 sections (largest to smallest, telescoped): (len, r, x).
    let sections: [(f64, f64, f64); 3] = [(0.10, 0.020, 0.05), (0.09, 0.015, 0.145), (0.08, 0.011, 0.230)];
    for &(len, r, sx) in sections.iter() {
        let body = mesh(pg("SmallTelescopes__tube", cyl_geo!(r, r, len, 10)), &brass_mat);
        body.rotation().z = PI / 2.0;
        body.position().set(sx, 0.020, 0);
        scope.add(&body);
    }
    // Leather grip on largest section.
    let grip = mesh(pg("SmallTelescopes__grip", cyl_geo!(0.021, 0.021, 0.020, 10)), &leather_mat);
    grip.rotation().z = PI / 2.0;
    grip.position().set(0.028, 0.020, 0);
    scope.add(&grip);
    // Eyepiece at large end.
    let eye = mesh(circle_geo!(0.018, 8), &lens_mat);
    eye.rotation().y = -PI / 2.0;
    eye.position().set(-0.001, 0.020, 0);
    scope.add(&eye);
    // Lens at small end.
    let lens = mesh(circle_geo!(0.010, 8), &lens_mat);
    lens.rotation().y = PI / 2.0;
    lens.position().set(0.271, 0.020, 0);
    scope.add(&lens);
    scope.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    scope.rotation().y = rnd() * PI * 2.0;
    g.world.add(&scope);
}
