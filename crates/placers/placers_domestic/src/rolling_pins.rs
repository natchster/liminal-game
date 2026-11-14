//! `place_rolling_pins`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Wooden rolling pin: 6% chance per level. Central roll
// + 2 handle knobs.
pub(crate) fn place_rolling_pins(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let wood_colors: [u32; 4] = [0xc0a070, 0xa07050, 0xd0b080, 0x8b5a2b];
    let wood_mat = lambert!({ color: pick(&wood_colors) });
    let pin = group();
    // Main roller cylinder (28 cm).
    let roller = mesh(pg("RollingPins__roller", cyl_geo!(0.024, 0.024, 0.28, 12)), &wood_mat);
    roller.rotation().z = PI / 2.0;
    roller.position().y = 0.024;
    pin.add(&roller);
    // 2 handle stems (thinner cylinders extending each end).
    for side in [-1.0, 1.0] {
        let stem = mesh(pg("RollingPins__stem", cyl_geo!(0.010, 0.010, 0.050, 8)), &wood_mat);
        stem.rotation().z = PI / 2.0;
        stem.position().set(side * 0.165, 0.024, 0);
        pin.add(&stem);
        // Handle knob at end (small sphere).
        let knob = mesh(pg("RollingPins__knob", sphere_geo!(0.014, 6, 5)), &wood_mat);
        knob.position().set(side * 0.194, 0.024, 0);
        pin.add(&knob);
    }
    pin.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    pin.rotation().y = rnd() * PI * 2.0;
    g.world.add(&pin);
}
