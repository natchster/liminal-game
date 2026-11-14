//! `place_spoon_rests`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Spoon rest w/ spoon: 7% chance per level.
pub(crate) fn place_spoon_rests(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.93 {
        return;
    } // 7% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let rest_colors: [u32; 5] = [0xffffff, 0xf0e0d0, 0x8080a0, 0x408080, 0xd050a0];
    let rest_mat = lambert!({ color: pick(&rest_colors) });
    let wood_mat = lambert!({ color: 0x8b5a2b });
    let rest = group();
    let dish = mesh(pg("SpoonRests__dish", sphere_geo!(0.045, 12, 8, 0, PI * 2.0, 0, PI / 2.0)), &rest_mat);
    // Flipped into a shallow dish standing on the plate (0.004..0.0139), not half under the floor.
    dish.scale().set(1.6, 0.22, 1);
    dish.rotation().x = PI;
    dish.position().y = 0.0139;
    rest.add(&dish);
    let plate = mesh(pg("SpoonRests__plate", cyl_geo!(0.070, 0.070, 0.004, 14)), &rest_mat);
    plate.scale().set(1, 1, 0.65);
    plate.position().y = 0.002;
    rest.add(&plate);
    if rnd() < 0.7 {
        let handle = mesh(pg("SpoonRests__handle", cyl_geo!(0.006, 0.006, 0.13, 6)), &wood_mat);
        handle.rotation().z = PI / 2.0;
        handle.position().set(0.016, 0.0195, 0); // resting on the dish rim (top 0.0139)
        rest.add(&handle);
        let bowl = mesh(pg("SpoonRests__bowl", sphere_geo!(0.014, 8, 6)), &wood_mat);
        bowl.scale().set(1, 0.4, 0.75);
        bowl.position().set(-0.056, 0.0165, 0); // lying in the dish, inside its rim
        rest.add(&bowl);
    }
    rest.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    rest.rotation().y = rnd() * PI * 2.0;
    g.world.add(&rest);
}
