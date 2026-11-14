//! `place_horseshoes`
use game::prelude::*;
use placers_kit::place::face_wall;
use placers_kit::place::pick;
use placers_kit::place::wall_tiles;

// Hanging horseshoe: 5% chance per level. Wall-mounted U-shape iron shoe
// w/ nail holes.
pub(crate) fn place_horseshoes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let walls = wall_tiles(g);
    if walls.is_empty() {
        return;
    }
    let (x, z, dx, dz) = pick(&walls);
    let iron_mat = lambert!({ color: 0x504040 });
    let nail_mat = lambert!({ color: 0x707070 });
    let shoe = group();
    let mount_y = 1.75;
    // Main U-shape via 3/4 torus.
    let arch = mesh(pg("Horseshoes__arch", torus_geo!(0.055, 0.010, 6, 14, PI * 1.3)), &iron_mat);
    arch.rotation().z = -PI * 0.15;
    arch.position().set(0, mount_y, 0.005);
    shoe.add(&arch);
    // 6 nail holes (small dark dots around inside of arch).
    for i in 0..6 {
        // along the web's fuller groove (r ≈ 0.058), clear of the heels and the toe clip;
        // 0.6 mm proud of the web face (z 0.0122). The group is already turned to the room.
        let a = -PI * 0.03 + (i as f64 / 5.0) * PI * 1.06;
        let nail = mesh(circle_geo!(0.004, 5), &nail_mat);
        nail.position().set(a.dcos() * 0.058, mount_y + a.dsin() * 0.058, 0.0128);
        shoe.add(&nail);
    }
    // Small nail-hanger at very top.
    let hanger = mesh(pg("Horseshoes__hanger", cyl_geo!(0.002, 0.002, 0.010, 4)), &nail_mat);
    // driven into the wall (z -0.008), head (+Y) towards the room; the toe's inner
    // edge (r 0.045) rests on its shank
    hanger.rotation().x = PI / 2.0;
    hanger.position().set(0, mount_y + 0.043, -0.004);
    shoe.add(&hanger);
    shoe.rotation().y = face_wall(dx, dz);
    shoe.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - 0.008), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - 0.008));
    g.world.add(&shoe);
}
