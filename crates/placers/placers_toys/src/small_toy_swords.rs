//! `place_small_toy_swords`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick_hex;
use placers_kit::place::pick_tile;

// Small toy sword: 8% chance per level. Kid's foam/plastic
// sword with blade + guard + hilt. Party-costume family.
pub(crate) fn place_small_toy_swords(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick_tile(&open);
    let blade_colors: [u32; 3] = [0xc0c8d0, 0xd8b040, 0xa8b0b8];
    let blade_mat = lambert!({ color: pick_hex(&blade_colors) });
    let grip_colors: [u32; 4] = [0xd82040, 0x2050d8, 0x8a5030, 0x30a030];
    let grip_mat = lambert!({ color: pick_hex(&grip_colors) });
    let gold_mat = lambert!({ color: 0xd8b040 });
    let sword = group();
    // Blade: long thin flattened box.
    let blade_len = 0.28;
    let blade = mesh(pg("SmallToySwords__blade", box_geo!(blade_len, 0.020, 0.005)), &blade_mat);
    blade.position().set(blade_len / 2.0 + 0.030, 0.010, 0);
    sword.add(&blade);
    // Blade tip point.
    let tip = mesh(pg("SmallToySwords__tip", cone_geo!(0.010, 0.020, 4)), &blade_mat);
    tip.rotation().z = -PI / 2.0;
    tip.position().set(blade_len + 0.040, 0.010, 0);
    sword.add(&tip);
    // Cross guard (horizontal bar).
    let guard = mesh(pg("SmallToySwords__guard", box_geo!(0.008, 0.036, 0.010)), &gold_mat);
    guard.position().set(0.030, 0.010, 0);
    sword.add(&guard);
    // Grip.
    let grip = mesh(pg("SmallToySwords__grip", cyl_geo!(0.008, 0.008, 0.05, 8)), &grip_mat);
    grip.rotation().z = PI / 2.0;
    grip.position().set(0.005, 0.010, 0);
    sword.add(&grip);
    // Pommel (gold ball at grip end).
    let pommel = mesh(pg("SmallToySwords__pommel", sphere_geo!(0.010, 8, 6)), &gold_mat);
    pommel.position().set(-0.020, 0.010, 0);
    sword.add(&pommel);
    sword.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    sword.rotation().y = rnd() * PI * 2.0;
    // Lie it flat (it stood on the blade's edge with the 3.6 cm guard through
    // the floor): roll 90° about the blade axis, then dip the point 0.02 rad so
    // it rests on the pommel and the blade end. 'YZX' = roll, dip, then yaw.
    sword.set_rotation_order(EulerOrder::YZX);
    sword.rotation().x = PI / 2.0;
    sword.rotation().z = -0.02;
    sword.position().y = 0.010;
    g.world.add(&sword);
}
