//! `place_ceiling_cables`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::ri;

// Ceiling hanging cable: 15% chance per level, 1 unit.
// Vertical thin cylinder dangling from ceiling + small plug head at bottom.
pub(crate) fn place_ceiling_cables(g: &mut Game, _dist: &[i32]) {
    g.pp.ceiling_cable_plug_positions = vec![]; // reset per level
    if rnd() < 0.85 {
        return;
    } // 15% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[ri(open.len())];
    let cable_mat = lambert!({ color: 0x181410 });
    let plug_mat = lambert!({ color: 0x080604 });
    let cable = group();
    // Cable length varies — 0.5-1.2 m, dangling from ceiling.
    let cable_len = 0.5 + rnd() * 0.7;
    let shaft = mesh(pg("CeilingCables__cable", cyl_geo!(0.006, 0.006, cable_len, 5)), &cable_mat);
    // Slight tilt for organic hang.
    let tilt = (rnd() - 0.5) * 0.2;
    shaft.rotation().z = tilt;
    // tilted about its centre: top end 3 mm into the ceiling
    shaft.position().y = WALL_H + 0.003 - (cable_len / 2.0) * tilt.dcos();
    cable.add(&shaft);
    // Plug at bottom (small dark box), the cable entering 4 mm into its top.
    let plug = mesh(pg("CeilingCables__plug", box_geo!(0.035, 0.045, 0.028)), &plug_mat);
    plug.position().set(tilt.dsin() * cable_len / 2.0, WALL_H + 0.003 - cable_len * tilt.dcos() - 0.0185, 0);
    cable.add(&plug);
    cable.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    cable.rotation().y = rnd() * PI * 2.0;
    g.world.add(&cable);
    // Track cable plug tip world position for spark events.
    let cp = cable.pos();
    g.pp.ceiling_cable_plug_positions.push(v3(cp.x, plug.pos().y, cp.z));
    // 30% chance: very slow pendulum sway on the cable group.
    // Small ±0.03 rad amplitude around Z, period 8-14 s.
    if rnd() < 0.30 {
        let sway_phase = rnd() * PI * 2.0;
        let sway_period = 8.0 + rnd() * 6.0;
        let base_rot_z = cable.rot().z;
        let c2 = cable.clone();
        g.animate(move |_g, _dt, t| {
            c2.rotation().z = base_rot_z + (t * (PI * 2.0 / sway_period) + sway_phase).dsin() * 0.03;
        });
    }
}
