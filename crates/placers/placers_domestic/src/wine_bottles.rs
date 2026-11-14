//! `place_wine_bottles`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Wine bottle: 6% chance per level.
pub(crate) fn place_wine_bottles(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let glass_colors: [u32; 3] = [0x304018, 0x603010, 0x201820];
    let glass_mat = lambert!({ color: pick(&glass_colors) });
    let foil_colors: [u32; 3] = [0xa02020, 0xd0a040, 0x606060];
    let foil_mat = lambert!({ color: pick(&foil_colors) });
    let label_mat = lambert!({ color: 0xf0e8c8 });
    let bottle = group();
    let body = mesh(pg("WineBottles__body", cyl_geo!(0.036, 0.036, 0.18, 12)), &glass_mat);
    body.position().y = 0.09;
    bottle.add(&body);
    let shoulder = mesh(pg("WineBottles__shoulder", cyl_geo!(0.014, 0.036, 0.030, 12)), &glass_mat);
    shoulder.position().y = 0.195;
    bottle.add(&shoulder);
    let neck = mesh(pg("WineBottles__neck", cyl_geo!(0.014, 0.014, 0.06, 10)), &glass_mat);
    neck.position().y = 0.24;
    bottle.add(&neck);
    let foil = mesh(pg("WineBottles__foil", cyl_geo!(0.016, 0.016, 0.020, 10)), &foil_mat);
    foil.position().y = 0.28;
    bottle.add(&foil);
    let label = mesh(plane_geo!(0.11, 0.070), &label_mat);
    label.scale().x = 0.5; // flat label kept inside the bottle's 0.072 m silhouette (was 0.11 wide)
    label.position().set(0, 0.10, 0.0365);
    bottle.add(&label);
    if rnd() < 0.20 {
        bottle.rotation().z = PI / 2.0;
    }
    let px = g.wx(x) + (rnd() - 0.5) * 2.0;
    let py = if bottle.rot().z != 0.0 { 0.036 } else { 0.0 }; // lying: axis one radius up, not on the floor
    let pz = g.wz(z) + (rnd() - 0.5) * 2.0;
    bottle.position().set(px, py, pz);
    bottle.rotation().y = rnd() * PI * 2.0;
    g.world.add(&bottle);
}
