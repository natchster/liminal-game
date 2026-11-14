//! `place_ceiling_fans`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Ceiling fan: 3% chance per level, spinning blades.
pub(crate) fn place_ceiling_fans(g: &mut Game, _dist: &[i32]) {
    g.pp.fan_positions.clear(); // reset per level
    if rnd() < 0.97 {
        return;
    } // 3% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = pick(&open_tiles);
    let motor_mat = lambert!({ color: 0x504a40 });
    let blade_colors: [u32; 4] = [0x8a5030, 0x201810, 0x606870, 0xe0d8c0];
    let blade_base_color = pick(&blade_colors);
    let fan = group();
    let fan_y = WALL_H - 0.08; // whole fan 2 cm up so the downrod meets the ceiling
                               // Stem to ceiling.
    let stem = mesh(pg("CeilingFans__downrod", cyl_geo!(0.010, 0.010, 0.08, 6)), &motor_mat);
    stem.position().y = WALL_H - 0.04;
    fan.add(&stem);
    // Motor housing (cylinder).
    let motor = mesh(pg("CeilingFans__motor", cyl_geo!(0.045, 0.045, 0.030, 8)), &motor_mat);
    motor.position().y = fan_y;
    fan.add(&motor);
    // Blade group (rotates).
    let blade_group = group();
    for i in 0..4 {
        // Per-blade tint variance (±8% brightness for wear/dust).
        let shade = 0.92 + rnd() * 0.16;
        let r2 = to_i32(255f64.min(((blade_base_color >> 16) & 0xff) as f64 * shade));
        let g2 = to_i32(255f64.min(((blade_base_color >> 8) & 0xff) as f64 * shade));
        let b2 = to_i32(255f64.min((blade_base_color & 0xff) as f64 * shade));
        let per_blade_mat = lambert!({ color: (r2 << 16) | (g2 << 8) | b2, side: DoubleSide });
        let blade = mesh(pg("CeilingFans__blade", box_geo!(0.30, 0.008, 0.06)), &per_blade_mat);
        blade.position().x = 0.15;
        blade.rotation().z = 0.14; // 8 deg tilt for realism (blade pitch)
        let arm = group();
        arm.rotation().y = (i as f64 / 4.0) * PI * 2.0;
        arm.add(&blade);
        blade_group.add(&arm);
    }
    blade_group.position().y = fan_y - 0.014;
    fan.add(&blade_group);
    // Subtle warm downlight from motor (30% of fans get it).
    if rnd() < 0.30 {
        let fan_light = point_light!(0xffe4b5, 0.12, 1.4);
        fan_light.position().y = fan_y - 0.05;
        fan.add(&fan_light);
    }
    // Slow rotation animator (fan spinning).
    let spin_rate = 1.5 + rnd() * 1.5;
    let bg = blade_group.clone();
    g.animate(move |_g, _dt, t| {
        bg.rotation().y = t * spin_rate;
    });
    fan.position().set(g.wx(x) + (rnd() - 0.5) * 1.0, 0, g.wz(z) + (rnd() - 0.5) * 1.0);
    g.world.add(&fan);
    let fp = fan.pos();
    g.pp.fan_positions.push(v3(fp.x, fan_y, fp.z)); // track for whoosh audio
}
