//! `place_wall_sockets`
use game::prelude::*;
use placers_kit::place::shuffle;
use placers_kit::place::wall_candidates;

// Wall sockets: 0-4 small floor-level outlets on random walls.
pub(crate) fn place_wall_sockets(g: &mut Game, _dist: &[i32]) {
    g.pp.socket_positions.clear(); // reset per level for spark events
    if rnd() < 0.30 {
        return;
    } // 30% chance NO sockets
    let wk = g.weather_key();
    // Electric biome gets more (charge-heavy identity).
    let density_mul = if wk == "electric" { 1.6 } else { 1.0 };
    let count = 6.0f64.min(1.0f64.max(((1.0 + rnd() * 3.0) * density_mul).floor())) as usize;
    let mut candidates = wall_candidates(g);
    shuffle(&mut candidates);
    let socket_mat = lambert!({ color: 0x2a2822 });
    let hole_mat = basic_mat!({ color: 0x000000 });
    let mut i = 0;
    while i < count && i < candidates.len() {
        let c = candidates[i];
        // 0.10 × 0.07 outlet plate.
        let plate = mesh(pg("WallSockets__plate", box_geo!(0.10, 0.07, 0.015)), &socket_mat);
        let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.008);
        let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.008);
        let socket_height = 0.28 + rnd() * 0.15; // 0.28 - 0.43 m
                                                 // Two tiny plug holes.
        let hole1 = mesh(plane_geo!(0.010, 0.020), &hole_mat);
        let hole2 = mesh(plane_geo!(0.010, 0.020), &hole_mat);
        if c.dx.abs() == 1 {
            let rot = if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 };
            plate.rotation().y = rot;
            hole1.rotation().y = rot;
            hole2.rotation().y = rot;
            let pz = wall_z + (rnd() - 0.5) * 1.4;
            plate.position().set(wall_x, socket_height, pz);
            // 4.5 mm behind the plate face: dark bottoms of the modelled slots
            let hx = wall_x - c.dx as f64 * 0.003;
            hole1.position().set(hx, socket_height, plate.pos().z - 0.018);
            hole2.position().set(hx, socket_height, plate.pos().z + 0.018);
        } else {
            let rot = if c.dz > 0 { PI } else { 0.0 };
            plate.rotation().y = rot;
            hole1.rotation().y = rot;
            hole2.rotation().y = rot;
            let px = wall_x + (rnd() - 0.5) * 1.4;
            plate.position().set(px, socket_height, wall_z);
            let hz = wall_z - c.dz as f64 * 0.003;
            hole1.position().set(plate.pos().x - 0.018, socket_height, hz);
            hole2.position().set(plate.pos().x + 0.018, socket_height, hz);
        }
        g.world.add(&plate);
        g.world.add(&hole1);
        g.world.add(&hole2);
        // Track for socket-spark events (electric biome only).
        let p = plate.pos();
        g.pp.socket_positions.push(v3(p.x, socket_height, p.z));
        i += 1;
    }
}
