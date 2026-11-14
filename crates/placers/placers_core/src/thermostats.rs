//! `place_thermostats`
use game::prelude::*;
use placers_kit::place::pick;
use placers_kit::place::wall_candidates;

// Thermostats: 20% chance per level, 1 small wall unit (1.3-1.5 m).
pub(crate) fn place_thermostats(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.80 {
        return;
    } // 20% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = pick(&candidates);
    // Housing box + dial face.
    let housing_mat = lambert!({ color: 0x9a8a70 });
    let housing = mesh(pg("Thermostats__housing", box_geo!(0.12, 0.12, 0.03)), &housing_mat);
    let dial_mat = lambert!({ map: thermostat_texture(), transparent: true, side: DoubleSide });
    let dial = mesh(plane_geo!(0.10, 0.10), &dial_mat);
    // housing back on the wall (3 cm deep); dial 0.5 mm in front of its face
    let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.015);
    let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.015);
    let th_height = 1.3 + rnd() * 0.2;
    if c.dx.abs() == 1 {
        let rot = if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 };
        housing.rotation().y = rot;
        dial.rotation().y = rot;
        let jit_z = (rnd() - 0.5) * 1.2;
        housing.position().set(wall_x, th_height, wall_z + jit_z);
        dial.position().set(wall_x - c.dx as f64 * 0.0155, th_height, wall_z + jit_z);
    } else {
        let rot = if c.dz > 0 { PI } else { 0.0 };
        housing.rotation().y = rot;
        dial.rotation().y = rot;
        let jit_x = (rnd() - 0.5) * 1.2;
        housing.position().set(wall_x + jit_x, th_height, wall_z);
        dial.position().set(wall_x + jit_x, th_height, wall_z - c.dz as f64 * 0.0155);
    }
    g.world.add(&housing);
    g.world.add(&dial);
}

// Thermostat canvas texture — analog dial with tick marks + needle
// frozen at random temp. One-per-load random reading.
game::lazy_static_tex!(
    thermostat_texture,
    canvas_tex_opt(64, 64, false, |ctx, w, h| {
        let (cx, cy, r) = (w / 2.0, h / 2.0, 24.0);
        // Cream background.
        ctx.set_fill_style("#d8ccb0");
        ctx.fill_rect(0, 0, w, h);
        // Dial ring.
        ctx.set_stroke_style("#2a1810");
        ctx.set_line_width(2);
        ctx.begin_path();
        ctx.arc(cx, cy, r, 0, PI * 2.0);
        ctx.stroke();
        // Tick marks (270° span from 135° to 45° via bottom).
        for i in 0..=10 {
            let a = PI * 0.75 + (PI * 1.5) * (i as f64 / 10.0);
            ctx.set_line_width(if i % 5 == 0 { 2 } else { 1 });
            ctx.begin_path();
            ctx.move_to(cx + a.cos() * (r - 2.0), cy + a.sin() * (r - 2.0));
            ctx.line_to(cx + a.cos() * (r - 6.0), cy + a.sin() * (r - 6.0));
            ctx.stroke();
        }
        // Needle at random reading in valid arc.
        let needle_ang = PI * 0.75 + (PI * 1.5) * (0.2 + rnd() * 0.6);
        ctx.set_stroke_style("#a02010");
        ctx.set_line_width(2);
        ctx.begin_path();
        ctx.move_to(cx, cy);
        ctx.line_to(cx + needle_ang.cos() * (r - 4.0), cy + needle_ang.sin() * (r - 4.0));
        ctx.stroke();
        // Center pin.
        ctx.set_fill_style("#2a1810");
        ctx.begin_path();
        ctx.arc(cx, cy, 2, 0, PI * 2.0);
        ctx.fill();
    })
);
