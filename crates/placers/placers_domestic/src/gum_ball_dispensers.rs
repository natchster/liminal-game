//! `place_gum_ball_dispensers`
use game::prelude::*;
use placers_kit::place::face_wall;
use placers_kit::place::pick;
use placers_kit::place::wall_tiles;

// Gum-ball dispenser: 3% chance per level. Wall-mounted glass globe on
// stand + coin slot.
pub(crate) fn place_gum_ball_dispensers(g: &mut Game, _dist: &[i32]) {
    g.pp.gumball_positions.clear(); // reset per level
    if rnd() < 0.97 {
        return;
    } // 3% chance
    let walls = wall_tiles(g);
    if walls.is_empty() {
        return;
    }
    let (x, z, dx, dz) = pick(&walls);
    let base_mat = lambert!({ color: 0xa02020 });
    let glass_mat = basic_mat!({ color: 0xe8f0f8, transparent: true, opacity: 0.35 });
    let gum_colors: [u32; 7] = [0xc02020, 0x2060c0, 0x40a040, 0xf0e020, 0xd050a0, 0xe08040, 0xffffff];
    let dispenser = group();
    let mount_y = 1.30;
    // Base pedestal (red metal).
    let base = mesh(pg("GumBallDispensers__base", box_geo!(0.14, 0.24, 0.14)), &base_mat);
    base.position().y = mount_y;
    dispenser.add(&base);
    // Glass sphere on top w/ gumballs inside.
    let globe = mesh(sphere_geo!(0.09, 12, 10), &glass_mat);
    globe.position().set(0, mount_y + 0.20, 0);
    dispenser.add(&globe);
    // 15 colored gumballs inside sphere.
    for _i in 0..15 {
        let gc = pick(&gum_colors);
        let gum_mat = lambert!({ color: gc });
        let ball = mesh(pg("GumBallDispensers__ball", sphere_geo!(0.012, 6, 5)), &gum_mat);
        // Random position inside globe (bias downward — gravity settled).
        let r = rnd() * 0.065;
        let theta = rnd() * PI * 2.0;
        let phi = PI * 0.5 + rnd() * PI * 0.4; // biased lower hemisphere
        ball.position().set(phi.sin() * theta.cos() * r, mount_y + 0.20 + phi.cos() * r, phi.sin() * theta.sin() * r);
        dispenser.add(&ball);
    }
    // Coin slot (small dark rectangle on base).
    let slot = mesh(box_geo!(0.014, 0.004, 0.002), lambert!({ color: 0x100808 }));
    slot.position().set(0, mount_y + 0.04, 0.072);
    dispenser.add(&slot);
    // Dispensing knob.
    let knob = mesh(pg("GumBallDispensers__knob", cyl_geo!(0.010, 0.010, 0.015, 8)), &base_mat);
    knob.rotation().x = PI / 2.0;
    knob.position().set(0, mount_y - 0.04, 0.078);
    dispenser.add(&knob);
    dispenser.rotation().y = face_wall(dx, dz);
    dispenser.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - 0.060), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - 0.060));
    g.world.add(&dispenser);
    let p = dispenser.pos();
    g.pp.gumball_positions.push(v3(p.x, 0, p.z));
}
