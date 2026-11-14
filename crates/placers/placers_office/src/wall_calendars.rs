//! `place_wall_calendars`
use game::prelude::*;
use placers_kit::place::mount_slid_ref as wall_mount;
use placers_kit::place::ri as pick;
use placers_kit::place::wall_candidates;

// Wall calendar: 20% chance per level, 1 unit. Small
// paper-hanging calendar with visible date grid.
pub(crate) fn place_wall_calendars(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.80 {
        return;
    } // 20% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = candidates[pick(candidates.len())];
    let cal_mat = lambert!({ map: calendar_texture_2(), transparent: true, side: DoubleSide });
    let cal = mesh(plane_geo!(0.24, 0.32), &cal_mat);
    let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.005);
    let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.005);
    let cal_height = 1.6 + rnd() * 0.20;
    wall_mount(&cal, &c, wall_x, wall_z, cal_height, 1.0);
    // Slight crooked hang.
    cal.rotation().z = (rnd() - 0.5) * 0.08;
    g.world.add(&cal);
    // 30% chance: X marks across some day cells + torn bottom-
    // corner triangle. X marks read as "counting down days".
    if rnd() < 0.30 {
        // 4-6 X marks in random cells.
        let x_mat = basic_mat!({ color: 0x1a0a08, transparent: true, opacity: 0.85, depthWrite: false, side: DoubleSide });
        let x_count = 4 + (rnd() * 3.0).floor() as i32;
        let cell_w = 0.24 / 7.0;
        let cell_h = 0.32 / 5.0; // approx grid size
        for _xi in 0..x_count {
            let cell_col = (rnd() * 7.0).floor() - 3.0; // -3..3
            let cell_row = (rnd() * 5.0).floor() - 2.0; // -2..2
            let cell_x = cell_col * cell_w + cell_w * 0.5;
            let cell_y = -cell_row * cell_h - cell_h * 0.5;
            // 2 diagonal strokes = X.
            for stroke in 0..2 {
                let line = mesh(plane_geo!(0.020, 0.002), &x_mat);
                let cry = cal.rot().y;
                line.rotation().z = if stroke == 0 { PI / 4.0 } else { -PI / 4.0 };
                line.rotation().y = cry;
                // Local offset in calendar frame; nudge slightly outward.
                let cp = cal.pos();
                let world_x = cp.x + cry.cos() * 0.0 - cry.sin() * cell_x;
                let world_z = cp.z - cry.sin() * 0.0 - cry.cos() * cell_x;
                line.position().set(world_x - c.dx as f64 * 0.001, cp.y + cell_y, world_z - c.dz as f64 * 0.001);
                g.world.add(&line);
            }
        }
        // Torn bottom-right corner triangle.
        let torn_mat = basic_mat!({ color: 0x1a1610, transparent: true, opacity: 0.8, depthWrite: false, side: DoubleSide });
        let size = 0.05 + rnd() * 0.03;
        let torn_geo = tri_geo([0.12 - size, -0.16, 0.0, 0.12, -0.16, 0.0, 0.12, -0.16 + size, 0.0], [0, 2, 1]);
        let torn = mesh(torn_geo, &torn_mat);
        let cry = cal.rot().y;
        let cp = cal.pos();
        torn.rotation().y = cry;
        torn.position().copy(&cp);
        torn.position().x -= c.dx as f64 * 0.001;
        torn.position().z -= c.dz as f64 * 0.001;
        g.world.add(&torn);
    }
}

/// A one-triangle geometry: f32 positions `v`, `set_index(idx)`,
/// `compute_vertex_normals()`.
fn tri_geo(v: [f64; 9], idx: [u32; 3]) -> Geo {
    let geo = Geo::new();
    geo.set_position(v.iter().map(|&a| a as f32).collect());
    geo.set_index(idx.to_vec());
    geo.compute_vertex_normals();
    geo
}

// Wall calendar canvas — small rectangular paper with faded
// month title and grid. Built once, reused across levels.
game::lazy_static_tex!(
    calendar_texture_2,
    canvas_tex_opt(96, 128, false, |ctx, w, h| {
        // Cream paper bg.
        ctx.set_fill_style("#e0d8b8");
        ctx.fill_rect(0, 0, w, h);
        // Dark month title bar at top.
        ctx.set_fill_style("#2a2015");
        ctx.fill_rect(4, 4, w - 8.0, 20);
        // Dark title text — fake month word (indecipherable at distance).
        ctx.set_fill_style("#c8b090");
        ctx.set_font("bold 12px sans-serif");
        ctx.set_text_align("center");
        ctx.fill_text("JULY 19—", w / 2.0, 18);
        // 5×7 grid of day cells with dark borders.
        ctx.set_stroke_style("#5a4830");
        ctx.set_line_width(1);
        let grid_top = 32.0;
        let grid_h = h - grid_top - 8.0;
        let cell_w = (w - 8.0) / 7.0;
        let cell_h = grid_h / 5.0;
        for r in 0..=5 {
            ctx.begin_path();
            ctx.move_to(4, grid_top + r as f64 * cell_h);
            ctx.line_to(w - 4.0, grid_top + r as f64 * cell_h);
            ctx.stroke();
        }
        for c in 0..=7 {
            ctx.begin_path();
            ctx.move_to(4.0 + c as f64 * cell_w, grid_top);
            ctx.line_to(4.0 + c as f64 * cell_w, grid_top + grid_h);
            ctx.stroke();
        }
        // Tiny random dark marks in some cells (day numbers indicated).
        ctx.set_fill_style("#3a2c1a");
        for r in 0..5 {
            for c in 0..7 {
                if rnd() < 0.7 {
                    ctx.fill_rect(6.0 + c as f64 * cell_w, grid_top + r as f64 * cell_h + 2.0, 4, 3);
                }
            }
        }
        // Grunge speckle.
        for _ in 0..15 {
            ctx.set_fill_style(&format!("rgba(0,0,0,{})", 0.06 + rnd() * 0.1));
            let (x, y) = (rnd() * w, rnd() * h);
            ctx.fill_rect(x, y, 1, 1);
        }
    })
);
