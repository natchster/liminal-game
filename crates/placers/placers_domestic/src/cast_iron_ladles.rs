//! `place_cast_iron_ladles`
use game::prelude::*;
use placers_kit::place::face_wall;
use placers_kit::place::pick;
use placers_kit::place::wall_tiles;

// Hanging cast iron ladle: 4% chance per level.
pub(crate) fn place_cast_iron_ladles(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let wt = wall_tiles(g);
    if wt.is_empty() {
        return;
    }
    let (x, z, dx, dz) = pick(&wt);
    let iron_mat = lambert!({ color: 0x181818 });
    let ladle = group();
    let mount_y = 1.80;
    let handle = mesh(pg("CastIronLadles__handle", cyl_geo!(0.008, 0.008, 0.34, 6)), &iron_mat);
    handle.position().set(0, mount_y, 0.010);
    ladle.add(&handle);
    let ring = mesh(pg("CastIronLadles__ring", torus_geo!(0.010, 0.002, 3, 8)), &iron_mat);
    // Upright hanging ring (facing the room), its lower edge through the handle top (0.17).
    ring.position().set(0, mount_y + 0.176, 0.010);
    ladle.add(&ring);
    let bowl = mesh(pg("CastIronLadles__bowl", sphere_geo!(0.035, 10, 6, 0, PI * 2.0, PI / 2.0, PI / 2.0)), &iron_mat);
    // Rim (opening at the bowl centre height) meets the handle's lower end at its back.
    bowl.position().set(0, mount_y - 0.172, 0.037);
    ladle.add(&bowl);
    ladle.rotation().y = face_wall(dx, dz);
    ladle.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - 0.015), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - 0.015));
    g.world.add(&ladle);
}
