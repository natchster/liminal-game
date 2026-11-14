//! `place_small_trophies`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::ri as pick;

// Small trophy: 10% chance per level. Gold cup + base on floor.
pub(crate) fn place_small_trophies(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[pick(open.len())];
    let gold_mat = lambert!({ color: 0xc8a028 });
    let base_mat = lambert!({ color: 0x3a2018 });
    let plaque_mat = lambert!({ color: 0xc8a028 });
    let trophy = group();
    // Base: dark wooden square.
    let base = mesh(pg("SmallTrophies__base", box_geo!(0.10, 0.03, 0.10)), &base_mat);
    base.position().y = 0.015;
    trophy.add(&base);
    // Small gold plaque on front face with tiny dark engraving lines.
    let plaque = mesh(pg("SmallTrophies__plaque", plane_geo!(0.07, 0.020)), &plaque_mat);
    plaque.position().set(0, 0.020, 0.051);
    trophy.add(&plaque);
    // Tiny dark engraving lines on plaque.
    let engrave_mat = basic_mat!({ color: 0x2a2018, transparent: true, opacity: 0.85, depthWrite: false });
    for el in 0..3 {
        let e_line = mesh(plane_geo!(0.04, 0.002), &engrave_mat);
        e_line.position().set((rnd() - 0.5) * 0.010, 0.015 + el as f64 * 0.005, 0.0515);
        trophy.add(&e_line);
    }
    // Stem: narrow gold cylinder.
    let stem = mesh(pg("SmallTrophies__stem", cyl_geo!(0.015, 0.015, 0.10, 6)), &gold_mat);
    stem.position().y = 0.08;
    trophy.add(&stem);
    // Cup: inverted cone gold.
    let cup = mesh(pg("SmallTrophies__cup", cyl_geo!(0.05, 0.025, 0.06, 8)), &gold_mat);
    cup.position().y = 0.16;
    trophy.add(&cup);
    // 2 handles: small torus sections on cup sides.
    for h_off in [-0.06f64, 0.06] {
        let handle = mesh(pg("SmallTrophies__handle", torus_geo!(0.020, 0.003, 4, 8, PI)), &gold_mat);
        // A C in the XZ-radial plane (XY): ends on the cup wall, bow outward.
        // The chord leans 0.395 rad with the cup's flare (r 0.025 -> 0.05).
        let hs = sign_of(h_off);
        handle.rotation().z = -hs * (PI / 2.0 + 0.395);
        handle.position().set(hs * 0.0365, 0.16, 0);
        trophy.add(&handle);
    }
    trophy.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    trophy.rotation().y = rnd() * PI * 2.0;
    // 30% chance: trophy lies on side (Z-rotated 90°).
    if rnd() < 0.30 {
        trophy.rotation().z = PI / 2.0 * (if rnd() < 0.5 { -1.0 } else { 1.0 });
        trophy.position().y = 0.10; // roughly cup diameter above floor
    }
    g.world.add(&trophy);
}
