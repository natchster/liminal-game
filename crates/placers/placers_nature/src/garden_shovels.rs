//! `place_garden_shovels`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Garden shovel: 3% chance per level. Long handle + D-grip + spade blade.
pub(crate) fn place_garden_shovels(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    } // 3% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let wood_mat = lambert!({ color: 0x8b5a2b });
    let blade_mat = lambert!({ color: 0x505050 });
    let shovel = group();
    // Long handle.
    let handle = mesh(pg("GardenShovels__handle", cyl_geo!(0.014, 0.014, 0.55, 8)), &wood_mat);
    handle.rotation().z = PI / 2.0;
    handle.position().y = 0.014;
    shovel.add(&handle);
    // Collar at blade end.
    let collar = mesh(pg("GardenHoes__collar", cyl_geo!(0.016, 0.016, 0.020, 8)), &blade_mat);
    collar.rotation().z = PI / 2.0;
    collar.position().set(-0.265, 0.014, 0);
    shovel.add(&collar);
    // Spade blade (rectangular w/ tapered tip via cone).
    let blade = mesh(pg("GardenShovels__blade", box_geo!(0.070, 0.008, 0.090)), &blade_mat);
    // Socket strap 2 mm into the collar (end -0.275); the blade lies on the floor.
    blade.position().set(-0.308, 0.004, 0);
    shovel.add(&blade);
    // Blade tip taper.
    // A flat triangular point in the blade's plane: apex out (-X), the
    // pyramid squashed (0.12) to the blade's thickness instead of standing
    // 9 cm tall with its lowest corner 3 cm under the floor.
    let tip = mesh(cone_geo!(0.045, 0.030, 3), &blade_mat);
    tip.rotation().z = PI / 2.0;
    tip.rotation().x = PI / 2.0;
    tip.scale().z = 0.12;
    tip.position().set(-0.308 - 0.035 - 0.014, 0.0054, 0);
    shovel.add(&tip);
    // D-grip at handle end (torus + 2 side bars).
    // The D lies in the blade's plane (XZ): chord across the handle's end
    // (narrowed 0.6 so its ends meet the shaft), crown bulging out along +X.
    let d_grip = mesh(pg("GardenShovels__grip", torus_geo!(0.024, 0.006, 5, 12, PI)), &wood_mat);
    set_rot_order(&d_grip, -PI / 2.0, -PI / 2.0, 0.0, EulerOrder::YXZ);
    d_grip.scale().x = 0.6;
    d_grip.position().set(0.273, 0.014, 0);
    shovel.add(&d_grip);
    shovel.position().set(g.wx(x) + (rnd() - 0.5) * 1.6, 0, g.wz(z) + (rnd() - 0.5) * 1.6);
    shovel.rotation().y = rnd() * PI * 2.0;
    g.world.add(&shovel);
}

/// `o.rotation.set(x, y, z, order)`
fn set_rot_order(o: &Obj, x: f64, y: f64, z: f64, order: EulerOrder) {
    o.set_rotation_order(order);
    o.rotation().set(x, y, z);
}
