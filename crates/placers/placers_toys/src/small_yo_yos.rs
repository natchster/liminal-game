//! `place_small_yo_yos`: Small yo-yo: 10% chance per level. 2 flat colored discs + connecting axle + short dangling string.
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

/// Small yo-yo: 10% chance per level. 2 flat colored discs +
/// connecting axle + short dangling string.
pub(crate) fn place_small_yo_yos(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let yoyo_colors = [0xd82020, 0x2050d8, 0xd8b820, 0x30a030, 0x8020d8];
    let yoyo_mat = lambert!({ color: pick(&yoyo_colors) });
    let axle_mat = lambert!({ color: 0x2a2418 });
    let string_mat = lambert!({ color: 0xe8e0d0 });
    let yoyo = group();
    // 2 flat discs (yo-yo halves).
    for x_off in [-0.010, 0.010] {
        let disc = mesh(pg("SmallYoYos__half", cyl_geo!(0.024, 0.024, 0.006, 12)), &yoyo_mat);
        disc.rotation().z = PI / 2.0;
        disc.position().set(x_off, 0.024, 0);
        yoyo.add(&disc);
    }
    // Central axle cylinder.
    let axle = mesh(pg("SmallYoYos__axle", cyl_geo!(0.005, 0.005, 0.020, 6)), &axle_mat);
    axle.rotation().z = PI / 2.0;
    axle.position().y = 0.024;
    yoyo.add(&axle);
    // Short dangling string: leaves the underside of the axle between the
    // halves and slopes down to the floor 10 cm out.
    let string = mesh(pg("SmallYoYos__string", cyl_geo!(0.001, 0.001, 0.10, 4)), &string_mat);
    string.position().set(0, 0.0105, 0.049);
    string.rotation().x = PI / 2.0 + 0.18;
    yoyo.add(&string);
    yoyo.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    yoyo.rotation().y = rnd() * PI * 2.0;
    g.world.add(&yoyo);
}
