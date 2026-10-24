//! Canvas textures shared by this crate's placers (built once, on the native RNG).
#![allow(unused_variables, unused_mut, unused_assignments, unused_imports, dead_code, clippy::all)]
use game::prelude::*;

// R402 tiny analog watch texture — small wristwatch-style face on
// wall. Very small circle with minimal ticks.
game::lazy_static_tex!(
    wall_watch_texture,
    canvas_tex_opt(64, 64, false, |ctx, w, h| {
        let (cx, cy) = (w / 2.0, h / 2.0);
        ctx.set_fill_style("#d8d0c0");
        ctx.begin_path();
        ctx.arc(cx, cy, 26, 0, PI * 2.0);
        ctx.fill();
        ctx.set_stroke_style("#2a2018");
        ctx.set_line_width(2);
        ctx.begin_path();
        ctx.arc(cx, cy, 26, 0, PI * 2.0);
        ctx.stroke();
        // 4 tick marks at 12/3/6/9.
        for i in 0..4 {
            let a = (i as f64 / 4.0) * PI * 2.0 - PI / 2.0;
            ctx.set_line_width(1.5);
            ctx.begin_path();
            ctx.move_to(cx + a.cos() * 20.0, cy + a.sin() * 20.0);
            ctx.line_to(cx + a.cos() * 24.0, cy + a.sin() * 24.0);
            ctx.stroke();
        }
        // Frozen at 10:10 for classic watch-face look.
        ctx.set_stroke_style("#1a1408");
        ctx.set_line_width(2);
        // Hour hand pointing 10.
        let h_ang = ((10.0f64 % 12.0) / 12.0) * PI * 2.0 - PI / 2.0;
        ctx.begin_path();
        ctx.move_to(cx, cy);
        ctx.line_to(cx + h_ang.cos() * 12.0, cy + h_ang.sin() * 12.0);
        ctx.stroke();
        // Minute hand pointing 2.
        let m_ang = (10.0 / 60.0) * PI * 2.0 - PI / 2.0;
        ctx.begin_path();
        ctx.move_to(cx, cy);
        ctx.line_to(cx + m_ang.cos() * 18.0, cy + m_ang.sin() * 18.0);
        ctx.stroke();
        ctx.set_fill_style("#1a1408");
        ctx.begin_path();
        ctx.arc(cx, cy, 2, 0, PI * 2.0);
        ctx.fill();
    })
);

// R400 digital wall clock texture — black bg with bright red LED
// segmented time display like "12:34".
game::lazy_static_tex!(
    digital_clock_texture,
    canvas_tex_opt(128, 64, false, |ctx, w, h| {
        // Black bg.
        ctx.set_fill_style("#080605");
        ctx.fill_rect(0, 0, w, h);
        // Red LED segment digits — big block font.
        ctx.set_fill_style("#e02818");
        ctx.set_font("bold 44px monospace");
        ctx.set_text_align("center");
        ctx.set_text_baseline("middle");
        // Randomize the time once, when the texture is built, for horror-value variance.
        let hours = ["12", "03", "11", "13", "99", "00"];
        let mins = ["00", "34", "11", "33", "66"];
        let hh = hours[(rnd() * hours.len() as f64).floor() as usize];
        let mm = mins[(rnd() * mins.len() as f64).floor() as usize];
        let t = format!("{}:{}", hh, mm);
        ctx.fill_text(&t, w / 2.0, h / 2.0);
    })
);

// R395 no-smoking pictogram texture — circular white bg with dark
// cigarette icon and red slash across.
game::lazy_static_tex!(
    no_smoking_texture,
    canvas_tex_opt(96, 96, false, |ctx, w, h| {
        let (cx, cy) = (w / 2.0, h / 2.0);
        // White circular bg.
        ctx.set_fill_style("#f0f0f0");
        ctx.begin_path();
        ctx.arc(cx, cy, 42, 0, PI * 2.0);
        ctx.fill();
        // Red ring around.
        ctx.set_stroke_style("#c02020");
        ctx.set_line_width(6);
        ctx.begin_path();
        ctx.arc(cx, cy, 39, 0, PI * 2.0);
        ctx.stroke();
        // Cigarette icon: horizontal white rectangle with dark tip + red ember.
        ctx.set_fill_style("#2a2018");
        ctx.fill_rect(cx - 20.0, cy - 3.0, 30, 6);
        // Red ember tip at right.
        ctx.set_fill_style("#f04020");
        ctx.fill_rect(cx + 10.0, cy - 3.0, 8, 6);
        // Smoke wisps (two small curves upward).
        ctx.set_stroke_style("#a0a0a0");
        ctx.set_line_width(2);
        ctx.begin_path();
        ctx.move_to(cx - 15.0, cy - 5.0);
        ctx.quadratic_curve_to(cx - 12.0, cy - 12.0, cx - 15.0, cy - 18.0);
        ctx.stroke();
        ctx.begin_path();
        ctx.move_to(cx - 5.0, cy - 5.0);
        ctx.quadratic_curve_to(cx - 2.0, cy - 12.0, cx - 5.0, cy - 18.0);
        ctx.stroke();
        // Red diagonal slash across everything.
        ctx.set_stroke_style("#c02020");
        ctx.set_line_width(8);
        ctx.begin_path();
        ctx.move_to(cx - 30.0, cy - 30.0);
        ctx.line_to(cx + 30.0, cy + 30.0);
        ctx.stroke();
    })
);

// R392 wall gauge canvas — round dial face with tick marks and
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

// R390 wall calendar canvas — small rectangular paper with faded
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
