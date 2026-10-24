//! Module-level canvas textures of chunk p21 (built on first use, native RNG).
use game::prelude::*;

// ------------------------------------------------------------------ line 38721
game::lazy_static_tex!(rust_stain_texture, canvas_tex(128, 320, |ctx, w, h| {
    ctx.clear_rect(0, 0, w, h);
    // Main vertical drip.
    ctx.set_fill_style("rgba(120,50,20,0.65)");
    ctx.begin_path();
    ctx.move_to(w * 0.35, 0);
    // Wavy edge left.
    let mut y = 0.0;
    while y < h {
        ctx.line_to(w * (0.30 + rnd() * 0.06), y);
        y += 20.0;
    }
    ctx.line_to(w * 0.65, h);
    // Wavy edge right (going back up).
    let mut y = h;
    while y > 0.0 {
        ctx.line_to(w * (0.60 + rnd() * 0.10), y);
        y -= 20.0;
    }
    ctx.close_path();
    ctx.fill();
    // Secondary tiny drips.
    ctx.set_fill_style("rgba(140,60,20,0.55)");
    for _ in 0..6 {
        ctx.begin_path();
        ctx.ellipse(w * (0.20 + rnd() * 0.60), rnd() * h, 3.0 + rnd() * 4.0, 15.0 + rnd() * 30.0, 0, 0, PI * 2.0);
        ctx.fill();
    }
}));

// ------------------------------------------------------------------ line 38843
game::lazy_static_tex!(claw_marks_texture, canvas_tex(256, 320, |ctx, w, h| {
    ctx.clear_rect(0, 0, w, h);
    ctx.set_stroke_style("rgba(80,20,10,0.85)");
    ctx.set_line_cap("round");
    // 3 parallel diagonal scratches.
    for i in 0..3 {
        let offset_x = 40.0 + i as f64 * 40.0;
        ctx.set_line_width(3.0 + rnd() * 3.0);
        ctx.begin_path();
        // Jagged path with per-segment jitter.
        let (mut x, mut y) = (offset_x, 40.0);
        ctx.move_to(x, y);
        for _ in 0..12 {
            x += 8.0 + rnd() * 4.0;
            y += 22.0;
            ctx.line_to(x + (rnd() - 0.5) * 8.0, y);
        }
        ctx.stroke();
    }
}));

// ------------------------------------------------------------------ line 38955
game::lazy_static_tex!(biohazard_texture, canvas_tex(160, 160, |ctx, w, h| {
    ctx.set_fill_style("#e8b820");
    ctx.fill_rect(0, 0, w, h);
    ctx.set_fill_style("#181008");
    let (cx, cy) = (w / 2.0, h / 2.0);
    // 3 outer petals of biohazard trefoil (3 rings + arcs).
    for i in 0..3 {
        let a = -PI / 2.0 + (i as f64 / 3.0) * PI * 2.0;
        let px = cx + a.cos() * 40.0;
        let py = cy + a.sin() * 40.0;
        ctx.begin_path();
        ctx.arc(px, py, 26, 0, PI * 2.0);
        ctx.fill();
        // Yellow inner circle to make ring.
        ctx.set_fill_style("#e8b820");
        ctx.begin_path();
        ctx.arc(px, py, 16, 0, PI * 2.0);
        ctx.fill();
        ctx.set_fill_style("#181008");
    }
    // Central circle.
    ctx.begin_path();
    ctx.arc(cx, cy, 12, 0, PI * 2.0);
    ctx.fill();
    // "BIOHAZARD" text.
    ctx.set_font("bold 14px sans-serif");
    ctx.set_text_align("center");
    ctx.fill_text("BIOHAZARD", cx, h - 8.0);
}));

// ------------------------------------------------------------------ line 39074
game::lazy_static_tex!(caution_tape_texture, canvas_tex(512, 48, |ctx, w, h| {
    ctx.set_fill_style("#e8b820");
    ctx.fill_rect(0, 0, w, h);
    ctx.set_fill_style("#181008");
    // Diagonal black stripes.
    ctx.save();
    ctx.translate(0, 0);
    let mut i = -60.0;
    while i < w {
        ctx.save();
        ctx.begin_path();
        ctx.move_to(i, 0);
        ctx.line_to(i + 20.0, 0);
        ctx.line_to(i + 20.0 + 24.0, h);
        ctx.line_to(i + 24.0, h);
        ctx.close_path();
        ctx.fill();
        ctx.restore();
        i += 30.0;
    }
    ctx.restore();
    // "CAUTION" text overlay.
    ctx.set_fill_style("#e8b820");
    ctx.fill_rect(w * 0.35, 8, w * 0.30, 32);
    ctx.set_fill_style("#181008");
    ctx.set_font("bold 22px sans-serif");
    ctx.set_text_align("center");
    ctx.set_text_baseline("middle");
    ctx.fill_text("CAUTION", w / 2.0, h / 2.0);
}));

// ------------------------------------------------------------------ line 39204
game::lazy_static_tex!(chalk_arrow_texture, canvas_tex(256, 128, |ctx, w, h| {
    ctx.clear_rect(0, 0, w, h);
    ctx.set_stroke_style("rgba(220,220,220,0.85)");
    ctx.set_fill_style("rgba(220,220,220,0.85)");
    ctx.set_line_width(6);
    ctx.set_line_cap("round");
    // Arrow shaft.
    ctx.begin_path();
    ctx.move_to(30, h / 2.0);
    ctx.line_to(w - 40.0, h / 2.0);
    ctx.stroke();
    // Arrow head.
    ctx.begin_path();
    ctx.move_to(w - 40.0, h / 2.0);
    ctx.line_to(w - 60.0, h / 2.0 - 20.0);
    ctx.move_to(w - 40.0, h / 2.0);
    ctx.line_to(w - 60.0, h / 2.0 + 20.0);
    ctx.stroke();
    // Optional word above.
    ctx.set_font("bold 24px sans-serif");
    ctx.set_text_align("center");
    let words = ["THIS WAY", "RUN", "HELP", "HIDE", "NO"];
    let word = words[(rnd() * words.len() as f64).floor() as usize];
    ctx.fill_text(word, w / 2.0, 30);
}));

// ------------------------------------------------------------------ line 39371
game::lazy_static_tex!(crayon_graffiti_texture, canvas_tex(384, 320, |ctx, w, h| {
    ctx.clear_rect(0, 0, w, h);
    // Crayon colors.
    let colors = ["#d82040", "#2050d8", "#d8b040", "#30a030", "#a020a0", "#181008"];
    ctx.set_line_width(5);
    ctx.set_line_cap("round");
    // Stick figure family.
    let n_figs = 3 + (rnd() * 2.0).floor() as i32;
    for i in 0..n_figs {
        ctx.set_stroke_style(colors[(rnd() * colors.len() as f64).floor() as usize]);
        let fx = 60.0 + i as f64 * 90.0;
        let fy = h - 60.0;
        let fh = 60.0 + rnd() * 40.0;
        // Head circle.
        ctx.begin_path();
        ctx.arc(fx, fy - fh, 12, 0, PI * 2.0);
        ctx.stroke();
        // Body line.
        ctx.begin_path();
        ctx.move_to(fx, fy - fh + 12.0);
        ctx.line_to(fx, fy - 20.0);
        ctx.stroke();
        // Arms line.
        ctx.begin_path();
        ctx.move_to(fx - 15.0, fy - fh + 30.0);
        ctx.line_to(fx + 15.0, fy - fh + 30.0);
        ctx.stroke();
        // Legs V.
        ctx.begin_path();
        ctx.move_to(fx, fy - 20.0);
        ctx.line_to(fx - 12.0, fy);
        ctx.move_to(fx, fy - 20.0);
        ctx.line_to(fx + 12.0, fy);
        ctx.stroke();
        // Frown face (creepy).
        ctx.begin_path();
        ctx.arc(fx - 4.0, fy - fh - 3.0, 1.5, 0, PI * 2.0);
        ctx.arc(fx + 4.0, fy - fh - 3.0, 1.5, 0, PI * 2.0);
        ctx.stroke();
        ctx.begin_path();
        ctx.arc(fx, fy - fh + 6.0, 3, PI, 0);
        ctx.stroke();
    }
    // Sun in upper left.
    ctx.set_stroke_style("#d8b040");
    ctx.begin_path();
    ctx.arc(50, 40, 18, 0, PI * 2.0);
    ctx.stroke();
    for i in 0..8 {
        let a = (i as f64 / 8.0) * PI * 2.0;
        ctx.begin_path();
        ctx.move_to(50.0 + a.cos() * 22.0, 40.0 + a.sin() * 22.0);
        ctx.line_to(50.0 + a.cos() * 32.0, 40.0 + a.sin() * 32.0);
        ctx.stroke();
    }
}));

// ------------------------------------------------------------------ line 39464
game::lazy_static_tex!(family_portrait_texture, canvas_tex(160, 128, |ctx, w, h| {
    // Aged sepia background.
    let grad = ctx.create_linear_gradient(0, 0, 0, h);
    grad.add_color_stop(0, "#a08060");
    grad.add_color_stop(1, "#503020");
    ctx.set_fill_style(&grad);
    ctx.fill_rect(0, 0, w, h);
    // 3-4 figure silhouettes.
    let n_figs = 3 + (rnd() * 2.0).floor() as i32;
    ctx.set_fill_style("#302010");
    for i in 0..n_figs {
        let fx = (i + 1) as f64 * (w / (n_figs + 1) as f64);
        let is_child = i > n_figs - 2 && rnd() < 0.6;
        let fh = if is_child { 40.0 } else { 60.0 };
        let fy = h - 10.0;
        // Body trapezoid.
        ctx.begin_path();
        ctx.move_to(fx - 12.0, fy);
        ctx.line_to(fx + 12.0, fy);
        ctx.line_to(fx + 8.0, fy - fh + 18.0);
        ctx.line_to(fx - 8.0, fy - fh + 18.0);
        ctx.close_path();
        ctx.fill();
        // Head circle.
        ctx.begin_path();
        ctx.arc(fx, fy - fh + 6.0, 10, 0, PI * 2.0);
        ctx.fill();
        // Face oval (skin tone).
        ctx.set_fill_style("#d8a880");
        ctx.begin_path();
        ctx.ellipse(fx, fy - fh + 6.0, 7, 8, 0, 0, PI * 2.0);
        ctx.fill();
        // Blank face (no features - unsettling).
        ctx.set_fill_style("#302010");
    }
    // Craquelure age lines.
    ctx.set_stroke_style("rgba(20,10,5,0.25)");
    ctx.set_line_width(1);
    for _ in 0..15 {
        ctx.begin_path();
        ctx.move_to(rnd() * w, rnd() * h);
        ctx.line_to(rnd() * w, rnd() * h);
        ctx.stroke();
    }
}));

// ------------------------------------------------------------------ line 39715
game::lazy_static_tex!(coloring_texture, canvas_tex(320, 200, |ctx, w, h| {
    ctx.set_fill_style("#f0e8d0");
    ctx.fill_rect(0, 0, w, h);
    // Center spine crease.
    ctx.set_stroke_style("rgba(80,60,40,0.4)");
    ctx.set_line_width(1);
    ctx.begin_path();
    ctx.move_to(w / 2.0, 10);
    ctx.line_to(w / 2.0, h - 10.0);
    ctx.stroke();
    // Line drawing (house on left side).
    ctx.set_stroke_style("#181008");
    ctx.set_line_width(2);
    // House square.
    ctx.stroke_rect(30, 80, 100, 80);
    // Roof triangle.
    ctx.begin_path();
    ctx.move_to(30, 80);
    ctx.line_to(80, 30);
    ctx.line_to(130, 80);
    ctx.stroke();
    // Door.
    ctx.stroke_rect(60, 120, 30, 40);
    // 2 windows.
    ctx.stroke_rect(35, 90, 20, 20);
    ctx.stroke_rect(105, 90, 20, 20);
    // Kid scribble in bright colors.
    let kid_colors = ["#d82040", "#2050d8", "#d8b040", "#30a030"];
    for i in 0..15 {
        ctx.set_stroke_style(kid_colors[i % kid_colors.len()]);
        ctx.set_line_width(3);
        ctx.begin_path();
        let mut x = 40.0 + rnd() * 100.0;
        let mut y = 90.0 + rnd() * 60.0;
        ctx.move_to(x, y);
        for _ in 0..5 {
            x += (rnd() - 0.5) * 20.0;
            y += (rnd() - 0.5) * 20.0;
            ctx.line_to(x, y);
        }
        ctx.stroke();
    }
    // Right page: line drawing of tree.
    ctx.set_stroke_style("#181008");
    ctx.set_line_width(2);
    ctx.stroke_rect(220, 130, 20, 40); // trunk
    ctx.begin_path();
    ctx.arc(230, 100, 40, 0, PI * 2.0); // canopy
    ctx.stroke();
    // Scribble green on tree.
    ctx.set_stroke_style("#30a030");
    ctx.set_line_width(4);
    for _ in 0..8 {
        ctx.begin_path();
        ctx.move_to(200.0 + rnd() * 60.0, 80.0 + rnd() * 40.0);
        ctx.line_to(200.0 + rnd() * 60.0, 80.0 + rnd() * 40.0);
        ctx.stroke();
    }
}));

// ------------------------------------------------------------------ line 39935
// (no placer in backrooms.js samples this texture; kept for completeness)
game::lazy_static_tex!(exit_sign_texture, canvas_tex(128, 64, |ctx, w, h| {
    ctx.set_fill_style("#204020");
    ctx.fill_rect(0, 0, w, h);
    ctx.set_fill_style("#40e070");
    ctx.set_font("bold 32px sans-serif");
    ctx.set_text_align("center");
    ctx.set_text_baseline("middle");
    ctx.fill_text("EXIT", w / 2.0, h / 2.0);
}));

// ------------------------------------------------------------------ line 40017
// (no placer in backrooms.js samples this texture; kept for completeness)
game::lazy_static_tex!(calendar_texture, canvas_tex(192, 256, |ctx, w, h| {
    ctx.set_fill_style("#f0e8d0");
    ctx.fill_rect(0, 0, w, h);
    // Header block.
    ctx.set_fill_style("#8a3020");
    ctx.fill_rect(0, 0, w, 60);
    // Month text.
    ctx.set_fill_style("#f0e8d0");
    ctx.set_font("bold 24px serif");
    ctx.set_text_align("center");
    ctx.set_text_baseline("middle");
    let months = ["JANUARY", "MARCH", "JULY", "OCTOBER", "NOVEMBER"];
    ctx.fill_text(months[(rnd() * months.len() as f64).floor() as usize], w / 2.0, 22);
    // Year.
    ctx.set_font("bold 18px serif");
    let years = ["1974", "1982", "1988", "1997"];
    ctx.fill_text(years[(rnd() * years.len() as f64).floor() as usize], w / 2.0, 44);
    // Weekday header.
    ctx.set_fill_style("#181008");
    ctx.set_font("bold 12px serif");
    let wdays = ["S", "M", "T", "W", "T", "F", "S"];
    for i in 0..7 {
        ctx.fill_text(wdays[i], 15.0 + i as f64 * 24.0, 72);
    }
    // 5x7 date grid.
    ctx.set_font("bold 14px serif");
    let mut d = 1;
    for row in 0..5 {
        for col in 0..7 {
            if d > 30 { break; }
            let x2 = 15.0 + col as f64 * 24.0;
            let y2 = 96.0 + row as f64 * 26.0;
            ctx.set_fill_style("#181008");
            ctx.fill_text(&d.to_string(), x2, y2);
            d += 1;
        }
    }
    // Red circle around random date (X marks the spot).
    ctx.set_stroke_style("#d82020");
    ctx.set_line_width(3);
    ctx.begin_path();
    let cd = 5 + (rnd() * 20.0).floor() as i32;
    let crow = ((cd - 1) as f64 / 7.0).floor();
    let ccol = (cd - 1) % 7;
    ctx.arc(15.0 + ccol as f64 * 24.0, 96.0 + crow * 26.0 - 2.0, 12, 0, PI * 2.0);
    ctx.stroke();
}));
