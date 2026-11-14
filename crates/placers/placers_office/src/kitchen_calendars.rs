//! `place_kitchen_calendars`
use game::prelude::*;
use placers_kit::place::wall_mount as mount_on_wall;
use placers_kit::place::wall_tiles;

pub(crate) fn place_kitchen_calendars(g: &mut Game, _dist: &[i32]) {
    g.pp.kitchen_calendar_positions = vec![]; // reset per level
    if rnd() < 0.97 {
        return;
    }
    let walls = wall_tiles(g);
    if walls.is_empty() {
        return;
    }
    let (x, z, dx, dz) = walls[(rnd() * walls.len() as f64).floor() as usize];
    let paper_mat = lambert!({ map: kitchen_calendar_texture() });
    let kc = group();
    let mount_y = 1.60;
    // Wall-hung calendar — no frame, just paper w/ small hanging nail.
    let (cal_w, cal_h) = (0.24, 0.28);
    let paper = mesh(plane_geo!(cal_w, cal_h), &paper_mat);
    paper.position().set(0, mount_y, 0.010);
    kc.add(&paper);
    // Small dark nail head visible at top center.
    let nail_mat = lambert!({ color: 0x606060 });
    let nail = mesh(pg("KitchenCalendars__nail", cyl_geo!(0.003, 0.003, 0.004, 5)), &nail_mat);
    nail.rotation().x = PI / 2.0;
    nail.position().set(0, mount_y + cal_h * 0.5 + 0.002, 0.012);
    kc.add(&nail);
    // Wire/ring loop just below nail (small torus).
    let wire_mat = lambert!({ color: 0x808080 });
    let wire = mesh(pg("KitchenCalendars__wire", torus_geo!(0.005, 0.0008, 3, 8)), &wire_mat);
    wire.rotation().x = PI / 2.0;
    wire.position().set(0, mount_y + cal_h * 0.5 - 0.005, 0.011);
    kc.add(&wire);
    mount_on_wall(g, &kc, x, z, dx, dz, 0.015);
    kc.rotation().z = (rnd() - 0.5) * 0.03;
    g.world.add(&kc);
    let p = kc.pos();
    g.pp.kitchen_calendar_positions.push(v3(p.x, 0, p.z));
}

// Kitchen calendar w/ marked dates (wall).
game::lazy_static_tex!(
    kitchen_calendar_texture,
    canvas_tex(220, 260, |ctx, w, h| {
        // White paper background.
        ctx.set_fill_style("#f8f4e8");
        ctx.fill_rect(0, 0, w, h);
        // Top decorative image band (a simple gradient — pastoral scene).
        let top_band = h * 0.36;
        let grad = ctx.create_linear_gradient(0, 0, 0, top_band);
        grad.add_color_stop(0, "#a0d0e8");
        grad.add_color_stop(1, "#d0e0a0");
        ctx.set_fill_style(&grad);
        ctx.fill_rect(0, 0, w, top_band);
        // Simple silhouettes in top image — hills + tree.
        ctx.set_fill_style("#405030");
        ctx.begin_path();
        ctx.move_to(0, top_band);
        ctx.line_to(w * 0.20, top_band * 0.65);
        ctx.line_to(w * 0.45, top_band * 0.75);
        ctx.line_to(w * 0.70, top_band * 0.55);
        ctx.line_to(w, top_band * 0.70);
        ctx.line_to(w, top_band);
        ctx.close_path();
        ctx.fill();
        // Small tree.
        ctx.set_fill_style("#304020");
        ctx.begin_path();
        ctx.arc(w * 0.15, top_band * 0.60, 12, 0, PI * 2.0);
        ctx.fill();
        ctx.set_fill_style("#603020");
        ctx.fill_rect(w * 0.15 - 1.5, top_band * 0.60, 3, 12);
        // Month name.
        ctx.set_fill_style("#302020");
        ctx.set_font("bold 20px serif");
        ctx.set_text_align("center");
        let months = ["JANUARY", "MARCH", "JUNE", "SEPTEMBER", "NOVEMBER", "DECEMBER"];
        let month = choice(&months);
        ctx.fill_text(month, w / 2.0, top_band + 26.0);
        // Year.
        ctx.set_font("italic 14px serif");
        let year = 1985 + (rnd() * 12.0).floor() as i64;
        ctx.fill_text(&year.to_string(), w / 2.0, top_band + 46.0);
        // Days of week header.
        ctx.set_font("bold 10px sans-serif");
        let days = ["S", "M", "T", "W", "T", "F", "S"];
        let grid_top = top_band + 60.0;
        let cell_w = w / 8.0;
        for i in 0..7 {
            ctx.fill_text(days[i], cell_w * (i + 1) as f64, grid_top);
        }
        // Grid lines + date numbers (5 weeks, mostly-filled).
        ctx.set_font("11px sans-serif");
        ctx.set_stroke_style("rgba(100,80,60,0.25)");
        ctx.set_line_width(0.5);
        let cell_h = (h - grid_top - 20.0) / 5.0;
        let start_offset = (rnd() * 4.0).floor() as i32; // Start day of week for the 1st.
        let mut day_num = 1;
        for row in 0..5 {
            for col in 0..7 {
                let cell_x = cell_w * (col + 1) as f64;
                let cell_y = grid_top + 12.0 + row as f64 * cell_h;
                // Grid line to bottom of cell.
                ctx.begin_path();
                ctx.move_to(cell_x - cell_w / 2.0, cell_y + cell_h * 0.85);
                ctx.line_to(cell_x + cell_w / 2.0, cell_y + cell_h * 0.85);
                ctx.stroke();
                // Fill number if in valid range.
                let cell_idx = row * 7 + col;
                if cell_idx >= start_offset && day_num <= 30 {
                    ctx.set_fill_style("#302020");
                    ctx.fill_text(&day_num.to_string(), cell_x, cell_y + cell_h * 0.3);
                    day_num += 1;
                }
            }
        }
        // 2-4 red-circled dates (implies marked events).
        ctx.set_stroke_style("#a02040");
        ctx.set_line_width(2);
        let mark_count = 2 + (rnd() * 3.0).floor() as i32;
        for _m in 0..mark_count {
            let row = (rnd() * 5.0).floor() as i32;
            let col = (rnd() * 7.0).floor() as i32;
            let cell_idx = row * 7 + col;
            if cell_idx >= start_offset && cell_idx - start_offset < 30 {
                let cx = cell_w * (col + 1) as f64;
                let cy = grid_top + 12.0 + row as f64 * cell_h + cell_h * 0.3;
                ctx.begin_path();
                ctx.arc(cx, cy, 8, 0, PI * 2.0);
                ctx.stroke();
            }
        }
        // Bottom company/brand text.
        ctx.set_font("italic 9px serif");
        ctx.set_fill_style("rgba(70,50,40,0.6)");
        ctx.set_text_align("center");
        let brands = ["Farmer's Almanac", "Country Living", "Home & Garden", "Nature's Beauty"];
        ctx.fill_text(choice(&brands), w / 2.0, h - 8.0);
    })
);
