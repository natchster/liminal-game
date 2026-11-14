//! `place_coffee_cups`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;
use placers_kit::place::shuffle;

// Coffee cups: 0-3 small paper cups per level (30% chance).
pub(crate) fn place_coffee_cups(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.70 {
        return;
    } // 30% chance
    let count = 1 + (rnd() * 3.0).floor() as usize;
    let mut open_tiles = open_tiles(g);
    shuffle(&mut open_tiles);
    // Cup colors: white paper, brown paper, styrofoam, disposable red.
    let cup_colors = [0xece0d0, 0x8a6a48, 0xf0eee4, 0xa03018];
    let mut i = 0;
    while i < count && i < open_tiles.len() {
        let (x, z) = open_tiles[i];
        let cup = group();
        let cup_color = pick(&cup_colors);
        let cup_mat = lambert!({ color: cup_color });
        // Tapered cylinder (top-heavy for takeaway coffee cup look).
        let body = mesh(pg("CoffeeCups__cup", cyl_geo!(0.038, 0.028, 0.09, 10)), &cup_mat);
        body.position().y = 0.045;
        cup.add(&body);
        // Coffee inside (dark disc at top when standing).
        let coffee = mesh(circle_geo!(0.032, 10), lambert!({ color: 0x2a1810 }));
        coffee.rotation().x = -PI / 2.0;
        coffee.position().y = 0.091;
        cup.add(&coffee);
        // 40% chance the cup is knocked over.
        let knocked_over = rnd() < 0.40;
        if knocked_over {
            cup.rotation().z = PI / 2.0;
            cup.position().y = 0.038;
        }
        let ox = (rnd() - 0.5) * 2.6;
        let oz = (rnd() - 0.5) * 2.6;
        cup.position().x = g.wx(x) + ox;
        cup.position().z = g.wz(z) + oz;
        cup.rotation().y = rnd() * PI * 2.0;
        g.world.add(&cup);
        // Coffee spill: knocked-over cups 60% chance spill brown stain.
        if knocked_over && rnd() < 0.60 {
            let r = 0.10 + rnd() * 0.06;
            let spill_geo = circle_geo!(r, 8);
            spill_geo.scale(1.6, 0.7, 1); // stretched away from cup mouth
            let spill_mat = basic_mat!({ color: 0x2a1810, transparent: true, opacity: 0.65, depthWrite: false });
            let spill = mesh(spill_geo, &spill_mat);
            let cup_ry = cup.rot().y;
            spill.rotation().x = -PI / 2.0;
            spill.rotation().z = cup_ry; // aligned with cup direction
            // Position stretched in cup's forward direction.
            let fx = cup_ry.sin();
            let fz = cup_ry.cos();
            let cp = cup.pos();
            spill.position().set(cp.x + fx * 0.10, 0.005, cp.z + fz * 0.10);
            g.world.add(&spill);
        }
        i += 1;
    }
}
