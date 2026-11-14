//! `place_radiators`
use game::prelude::*;
use placers_kit::place::pick;
use placers_kit::place::wall_candidates;

// Radiator: 0-1 cast-iron ridged heater against a random wall.
pub(crate) fn place_radiators(g: &mut Game, _dist: &[i32]) {
    g.pp.radiator_positions.clear(); // reset per level
    if rnd() < 0.75 {
        return;
    } // 25% chance per level
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = pick(&candidates);
    let body_color = vent_color_by_weather(g.weather_key());
    let body_mat = lambert!({ color: body_color });
    let radiator = group();
    // Main body: 0.7 wide × 0.5 tall × 0.10 deep flat box.
    let body = mesh(pg("Radiators__body", box_geo!(0.70, 0.50, 0.10)), &body_mat);
    body.position().y = 0.30;
    radiator.add(&body);
    // 8 vertical fins on front face — each a thin tall box.
    for f in 0..8 {
        let fin = mesh(pg("Radiators__fin", box_geo!(0.06, 0.48, 0.02)), &body_mat);
        fin.position().set(-0.31 + f as f64 * 0.085, 0.30, 0.06);
        radiator.add(&fin);
    }
    // Top slat (cap piece).
    let cap = mesh(pg("Radiators__cap", box_geo!(0.72, 0.03, 0.12)), &body_mat);
    cap.position().y = 0.565;
    radiator.add(&cap);
    // Position: against the wall.
    let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.10);
    let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.10);
    if c.dx.abs() == 1 {
        radiator.rotation().y = if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 };
        radiator.position().set(wall_x, 0, wall_z + (rnd() - 0.5) * 1.2);
    } else {
        radiator.rotation().y = if c.dz > 0 { PI } else { 0.0 };
        radiator.position().set(wall_x + (rnd() - 0.5) * 1.2, 0, wall_z);
    }
    g.world.add(&radiator);
    // Track for steam-hiss audio events.
    let rp = radiator.pos();
    g.pp.radiator_positions.push(v3(rp.x, 0.30, rp.z));
}
