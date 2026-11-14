//! `place_rolling_pin_stands`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Rolling pin stand w/ 2 pins: 4% chance per level.
pub(crate) fn place_rolling_pin_stands(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let wood_mat = lambert!({ color: 0x8b5a2b });
    let pin_color1 = lambert!({ color: 0xd0b080 });
    let pin_color2 = lambert!({ color: 0xa07050 });
    let stand = group();
    // Base plate.
    let base = mesh(pg("RollingPinStands__base", box_geo!(0.30, 0.010, 0.10)), &wood_mat);
    base.position().y = 0.005;
    stand.add(&base);
    // 4 short vertical uprights.
    for sx in [-1.0, 1.0] {
        for sz in [-1.0, 1.0] {
            let post = mesh(pg("RollingPinStands__post", cyl_geo!(0.006, 0.006, 0.045, 5)), &wood_mat);
            post.position().set(sx * 0.130, 0.028, sz * 0.035);
            stand.add(&post);
        }
    }
    // 2 rolling pins resting horizontally on posts.
    for i in 0..2 {
        let pin_mat = if i == 0 { &pin_color1 } else { &pin_color2 };
        // Side by side, each pin's handles resting on the tops (0.0505) of its
        // pair of posts (z +-0.035); they were stacked 8 mm into each other.
        let pin_y = 0.057;
        let pin_z = if i == 0 { -0.030 } else { 0.030 };
        let roller = mesh(pg("RollingPinStands__roller", cyl_geo!(0.020, 0.020, 0.24, 10)), pin_mat);
        roller.rotation().z = PI / 2.0;
        roller.position().set(0, pin_y, pin_z);
        stand.add(&roller);
        for side in [-1.0, 1.0] {
            let stem = mesh(pg("RollingPinStands__handle", cyl_geo!(0.008, 0.008, 0.040, 6)), pin_mat);
            stem.rotation().z = PI / 2.0;
            stem.position().set(side * 0.140, pin_y, pin_z);
            stand.add(&stem);
            let knob = mesh(pg("RollingPinStands__knob", sphere_geo!(0.012, 6, 5)), pin_mat);
            knob.position().set(side * 0.164, pin_y, pin_z);
            stand.add(&knob);
        }
    }
    stand.position().set(g.wx(x) + (rnd() - 0.5) * 1.8, 0, g.wz(z) + (rnd() - 0.5) * 1.8);
    stand.rotation().y = rnd() * PI * 2.0;
    g.world.add(&stand);
}
