//! `place_small_lipsticks`: Small lipstick tube: 10% chance per level. Two-tone tube with colored bullet tip. Dressing-table family.
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

/// Small lipstick tube: 10% chance per level. Two-tone tube
/// with colored bullet tip. Dressing-table family.
pub(crate) fn place_small_lipsticks(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let tube_colors = [0x201a10, 0xd8b040, 0xa02040, 0x606870, 0xd050a0];
    let bullet_colors = [0xa02040, 0xd85090, 0x8a2030, 0xc02060, 0xd0405a];
    let tube_mat = lambert!({ color: pick(&tube_colors) });
    let bullet_mat = lambert!({ color: pick(&bullet_colors) });
    let stick = group();
    let is_open = rnd() < 0.50;
    // Body: main tube cylinder.
    let body = mesh(pg("SmallLipsticks__case", cyl_geo!(0.010, 0.010, 0.050, 8)), &tube_mat);
    body.rotation().z = PI / 2.0;
    body.position().set(0.025, 0.010, 0);
    stick.add(&body);
    if is_open {
        // Cap laid separately, bullet extended.
        let cap = mesh(pg("SmallLipsticks__cap", cyl_geo!(0.012, 0.012, 0.035, 8)), &tube_mat);
        cap.rotation().z = PI / 2.0;
        cap.position().set(-0.020, 0.010, 0.025);
        stick.add(&cap);
        // Bullet: sloped-tip cone.
        let bullet = mesh(pg("SmallLipsticks__bullet", cyl_geo!(0.008, 0.008, 0.020, 8)), &bullet_mat);
        bullet.rotation().z = PI / 2.0;
        bullet.position().set(0.060, 0.010, 0);
        stick.add(&bullet);
        let tip = mesh(pg("SmallLipsticks__tip", cone_geo!(0.008, 0.010, 8)), &bullet_mat);
        tip.rotation().z = -PI / 2.0;
        tip.position().set(0.075, 0.010, 0);
        stick.add(&tip);
    } else {
        // Closed: cap on top of body.
        let cap = mesh(pg("SmallLipsticks__cap", cyl_geo!(0.012, 0.012, 0.035, 8)), &tube_mat);
        cap.rotation().z = PI / 2.0;
        cap.position().set(0.068, 0.010, 0);
        stick.add(&cap);
    }
    stick.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    stick.rotation().y = rnd() * PI * 2.0;
    g.world.add(&stick);
}
