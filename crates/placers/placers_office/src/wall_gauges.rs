//! `place_wall_gauges`
use game::prelude::*;
use placers_kit::geometry::clone_obj;
use placers_kit::place::mount_slid_ref;
use placers_kit::place::ri;
use placers_kit::place::wall_candidates;

// Wall analog gauge: 15% chance per level. Small circular
// gauge on wall — pressure/temperature reading dial.
pub(crate) fn place_wall_gauges(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.85 {
        return;
    } // 15% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = candidates[ri(candidates.len())];
    let g_mat = lambert!({ map: gauge_texture(), transparent: true, side: DoubleSide });
    let gauge = mesh(circle_geo!(0.10, 20), &g_mat);
    let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.008);
    let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.008);
    let gauge_height = 1.5 + rnd() * 0.30;
    mount_slid_ref(&gauge, &c, wall_x, wall_z, gauge_height, 1.0);
    g.world.add(&gauge);
    // 30% chance: 1-2 sibling gauges 0.24-0.30 m apart in row.
    if rnd() < 0.30 {
        let sib_count = 1 + (rnd() * 2.0).floor() as i32;
        let sib_dir = if rnd() < 0.5 { -1.0 } else { 1.0 };
        for s in 1..=sib_count {
            let sib_gauge = clone_obj(&gauge);
            let sib_off = sib_dir * (0.24 + rnd() * 0.06) * s as f64;
            if c.dx.abs() == 1 {
                sib_gauge.position().z += sib_off;
            } else {
                sib_gauge.position().x += sib_off;
            }
            g.world.add(&sib_gauge);
        }
    }
}

// Wall gauge canvas — round dial face with tick marks and
// needle. Built once.
game::lazy_static_tex!(
    gauge_texture,
    canvas_tex_opt(96, 96, false, |ctx, w, h| {
        let (cx, cy) = (w / 2.0, h / 2.0);
        // Cream face bg.
        ctx.set_fill_style("#e0d4b0");
        ctx.begin_path();
        ctx.arc(cx, cy, 42, 0, PI * 2.0);
        ctx.fill();
        // Dark ring around.
        ctx.set_stroke_style("#3a2818");
        ctx.set_line_width(3);
        ctx.begin_path();
        ctx.arc(cx, cy, 42, 0, PI * 2.0);
        ctx.stroke();
        // Tick marks: 12 major spanning 3/4 circle.
        ctx.set_stroke_style("#2a2018");
        for i in 0..=12 {
            let a = -PI * 1.25 + (i as f64 / 12.0) * PI * 1.5;
            let is_major = i % 3 == 0;
            let r_in = if is_major { 32.0 } else { 34.0 };
            ctx.set_line_width(if is_major { 2.0 } else { 1.0 });
            ctx.begin_path();
            ctx.move_to(cx + a.cos() * r_in, cy + a.sin() * r_in);
            ctx.line_to(cx + a.cos() * 38.0, cy + a.sin() * 38.0);
            ctx.stroke();
        }
        // Needle at 30% of scale.
        ctx.set_stroke_style("#8a2020");
        ctx.set_line_width(3);
        let needle_ang = -PI * 1.25 + 0.3 * PI * 1.5;
        ctx.begin_path();
        ctx.move_to(cx, cy);
        ctx.line_to(cx + needle_ang.cos() * 26.0, cy + needle_ang.sin() * 26.0);
        ctx.stroke();
        // Center pivot.
        ctx.set_fill_style("#2a2018");
        ctx.begin_path();
        ctx.arc(cx, cy, 4, 0, PI * 2.0);
        ctx.fill();
    })
);
