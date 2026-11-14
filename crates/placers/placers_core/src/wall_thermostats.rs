//! `place_wall_thermostats`
use game::prelude::*;
use placers_kit::place::mount_slid_ref as wall_mount;
use placers_kit::place::ri as pick;
use placers_kit::place::wall_candidates;

// Wall thermostat: 20% chance per level, 1 unit. Small rectangular
// beige control panel with tiny dial.
pub(crate) fn place_wall_thermostats(g: &mut Game, _dist: &[i32]) {
    g.pp.thermostat_positions = vec![]; // reset per level
    if rnd() < 0.80 {
        return;
    } // 20% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = candidates[pick(candidates.len())];
    let body_mat = lambert!({ color: 0xd8d0c0 });
    let dial_mat = lambert!({ color: 0x2a2418 });
    let readout_mat = basic_mat!({ color: 0x304028 });
    let thermo = group();
    // Body: 8×12 cm cream rectangle.
    let body = mesh(pg("WallThermostats__body", box_geo!(0.08, 0.12, 0.02)), &body_mat);
    thermo.add(&body);
    // Dial: small circular knob near top.
    let dial = mesh(pg("WallThermostats__knob", cyl_geo!(0.018, 0.018, 0.006, 8)), &dial_mat);
    dial.rotation().x = PI / 2.0;
    dial.position().set(0, 0.02, 0.013); // back on the body's face (z 0.010)
    thermo.add(&dial);
    // Tiny readout window at bottom.
    let readout = mesh(plane_geo!(0.05, 0.02), &readout_mat);
    readout.position().set(0, -0.035, 0.011);
    thermo.add(&readout);
    let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.011);
    let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.011);
    let thermo_height = 1.4 + rnd() * 0.15;
    wall_mount(&thermo, &c, wall_x, wall_z, thermo_height, 1.0);
    g.world.add(&thermo);
    // Track thermostat position for click audio events.
    let tp = thermo.pos();
    g.pp.thermostat_positions.push(v3(tp.x, thermo_height, tp.z));
    // 30% chance: subtle readout flicker animator — brief dim
    // dips simulating an aged LCD glitching.
    if rnd() < 0.30 {
        let _readout_base_op = readout_mat.opacity_v();
        let mut flicker_t = 3.0 + rnd() * 5.0;
        let mut dip_t = 0.0;
        let m = readout_mat.clone();
        g.animate(move |_g, dt, _t| {
            if dip_t > 0.0 {
                dip_t -= dt;
                let on = dip_t > 0.0;
                m.m().color.set_rgb(
                    if on { 0.10 } else { 0x30 as f64 / 255.0 },
                    if on { 0.15 } else { 0x40 as f64 / 255.0 },
                    if on { 0.10 } else { 0x28 as f64 / 255.0 },
                );
            } else {
                flicker_t -= dt;
                if flicker_t <= 0.0 {
                    dip_t = 0.04;
                    flicker_t = 3.0 + rnd() * 5.0;
                }
            }
        });
    }
}
