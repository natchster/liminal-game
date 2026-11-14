//! `place_exit_arrow_signs`
use game::prelude::*;
use placers_kit::place::pick_wall;
use placers_kit::place::wall_mount;

// Exit-arrow sign: 5% chance per level. Small green directional sign w/ white arrow.
pub(crate) fn place_exit_arrow_signs(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    }
    let Some((x, z, dx, dz)) = pick_wall(g) else { return };
    let bg_mat = lambert!({ color: 0x208040 });
    let arrow_mat = lambert!({ color: 0xf8f8f0 });
    let sign = group();
    let mount_y = 2.10;
    let bg = mesh(pg("ExitArrowSigns__panel", box_geo!(0.30, 0.14, 0.010)), &bg_mat);
    bg.position().y = mount_y;
    sign.add(&bg);
    let flip = rnd() < 0.5;
    let dir: f64 = if flip { -1.0 } else { 1.0 };
    let shaft = mesh(pg("ExitArrowSigns__shaft", box_geo!(0.16, 0.028, 0.006)), &arrow_mat);
    shaft.position().set(-0.02 * dir, mount_y, 0.008);
    sign.add(&shaft);
    let head = mesh(pg("ExitArrowSigns__head", cone_geo!(0.036, 0.060, 3)), &arrow_mat);
    head.rotation().z = if dir > 0.0 { -PI / 2.0 } else { PI / 2.0 };
    head.position().set(0.080 * dir, mount_y, 0.010);
    sign.add(&head);
    wall_mount(g, &sign, x, z, dx, dz, 0.006);
    g.world.add(&sign);
}
