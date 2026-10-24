//! Module-level canvas textures of chunk p09 (built lazily on the native RNG).
use game::prelude::*;

// ------------------------------------------------------------------ line 19219
// R690 framed graduation photo w/ tassel (wall). Formal portrait of graduate
// in cap+gown.
game::lazy_static_tex!(graduation_photo_texture, canvas_tex(180, 220, |ctx, w, h| {
    // Muted color background — vintage color print, tan/olive.
    let grad = ctx.create_linear_gradient(0, 0, 0, h);
    grad.add_color_stop(0, "#b8a888");
    grad.add_color_stop(1, "#605040");
    ctx.set_fill_style(&grad);
    ctx.fill_rect(0, 0, w, h);
    // Vignette.
    let rg = ctx.create_radial_gradient(w / 2.0, h * 0.42, 20, w / 2.0, h * 0.42, w);
    rg.add_color_stop(0, "rgba(0,0,0,0)");
    rg.add_color_stop(1, "rgba(0,0,0,0.35)");
    ctx.set_fill_style(&rg);
    ctx.fill_rect(0, 0, w, h);
    // Grain.
    ctx.set_fill_style("rgba(30,20,10,0.15)");
    for _ in 0..200 {
        ctx.begin_path();
        ctx.arc(rnd() * w, rnd() * h, 1, 0, PI * 2.0);
        ctx.fill();
    }
    // Central graduate figure.
    let (cx, cy) = (w / 2.0, h * 0.5);
    // Head — soft radial (skin tone).
    let head_grad = ctx.create_radial_gradient(cx, cy - 30.0, 4, cx, cy - 30.0, 26);
    head_grad.add_color_stop(0, "rgba(200,160,130,0.9)");
    head_grad.add_color_stop(0.55, "rgba(180,140,110,0.55)");
    head_grad.add_color_stop(1, "rgba(180,140,110,0)");
    ctx.set_fill_style(&head_grad);
    ctx.begin_path();
    ctx.arc(cx, cy - 30.0, 26, 0, PI * 2.0);
    ctx.fill();
    // Mortarboard cap — flat square on top of head.
    ctx.set_fill_style("#101010");
    ctx.fill_rect(cx - 36.0, cy - 68.0, 72, 6);
    // Cap top square (perspective).
    ctx.begin_path();
    ctx.move_to(cx - 26.0, cy - 72.0);
    ctx.line_to(cx + 26.0, cy - 72.0);
    ctx.line_to(cx + 34.0, cy - 62.0);
    ctx.line_to(cx - 34.0, cy - 62.0);
    ctx.close_path();
    ctx.fill();
    // Cap button center.
    ctx.set_fill_style("#c0a040");
    ctx.begin_path();
    ctx.arc(cx, cy - 66.0, 3, 0, PI * 2.0);
    ctx.fill();
    // Tassel — cord + strands hanging from center-top right of cap.
    ctx.set_stroke_style("#c0a040");
    ctx.set_line_width(2);
    ctx.begin_path();
    ctx.move_to(cx, cy - 66.0);
    ctx.quadratic_curve_to(cx + 22.0, cy - 40.0, cx + 30.0, cy - 20.0);
    ctx.stroke();
    // Tassel strands (short lines at the end).
    for i in 0..5 {
        ctx.begin_path();
        ctx.move_to(cx + 30.0, cy - 20.0);
        ctx.line_to(cx + 30.0 + (i as f64 - 2.0) * 2.0, cy);
        ctx.stroke();
    }
    // Gown body (dark shape below head).
    let gown_grad = ctx.create_radial_gradient(cx, cy + 40.0, 10, cx, cy + 40.0, 70);
    gown_grad.add_color_stop(0, "rgba(30,25,25,0.9)");
    gown_grad.add_color_stop(0.6, "rgba(50,45,40,0.55)");
    gown_grad.add_color_stop(1, "rgba(50,45,40,0)");
    ctx.set_fill_style(&gown_grad);
    ctx.begin_path();
    ctx.ellipse(cx, cy + 40.0, 42, 62, 0, 0, PI * 2.0);
    ctx.fill();
    // Optional diploma tube in hand.
    ctx.set_fill_style("#f0e8d0");
    ctx.fill_rect(cx - 30.0, cy + 30.0, 20, 5);
    ctx.set_stroke_style("#a02040");
    ctx.set_line_width(1);
    ctx.stroke_rect(cx - 30.0, cy + 30.0, 20, 5);
    // Handwritten caption bottom.
    let captions = [
        "Class of '89",
        "Graduation, 1975",
        "Sarah, U. of Chicago '02",
        "Congratulations! '93",
        "Michael, Valedictorian '85",
    ];
    let cap = captions[(rnd() * captions.len() as f64).floor() as usize];
    ctx.set_fill_style("#402010");
    ctx.set_font("italic 11px cursive");
    ctx.set_text_align("center");
    ctx.fill_text(cap, w / 2.0, h - 8.0);
    // Photo border.
    ctx.set_fill_style("#f0e8d0");
    ctx.fill_rect(0, 0, w, 6);
    ctx.fill_rect(0, h - 22.0, w, 6);
    ctx.fill_rect(0, 0, 6, h);
    ctx.fill_rect(w - 6.0, 0, 6, h);
}));

// ------------------------------------------------------------------ line 19817
// R693 crumpled letter w/ opening greeting: handwritten letter with
// "Dear X..." visible + signature.
game::lazy_static_tex!(letter_texture, canvas_tex(180, 220, |ctx, w, h| {
    // Aged paper background.
    ctx.set_fill_style("#f4ecd0");
    ctx.fill_rect(0, 0, w, h);
    // Faint blue-ruled lines.
    ctx.set_stroke_style("rgba(120,140,180,0.28)");
    ctx.set_line_width(0.7);
    let mut y = 24.0;
    while y < h - 10.0 {
        ctx.begin_path();
        ctx.move_to(8, y);
        ctx.line_to(w - 8.0, y);
        ctx.stroke();
        y += 14.0;
    }
    // Red left margin line.
    ctx.set_stroke_style("rgba(200,60,60,0.35)");
    ctx.set_line_width(1);
    ctx.begin_path();
    ctx.move_to(18, 8);
    ctx.line_to(18, h - 8.0);
    ctx.stroke();
    // Date top right.
    let dates = ["October 14, 1987", "March 22, 1974", "July 4, 1962", "December 18, 1993"];
    ctx.set_fill_style("#302010");
    ctx.set_font("italic 11px cursive");
    ctx.set_text_align("right");
    ctx.fill_text(dates[(rnd() * dates.len() as f64).floor() as usize], w - 12.0, 20);
    // Greeting: "Dear X,"
    let names = ["Sarah", "Michael", "Emma", "James", "Rose", "Thomas", "Anna", "David"];
    let greeting = format!("Dear {},", names[(rnd() * names.len() as f64).floor() as usize]);
    ctx.set_text_align("left");
    ctx.set_font("italic 16px cursive");
    ctx.fill_text(&greeting, 24, 44);
    // Body — 6-8 wavy lines of cursive script.
    ctx.set_stroke_style("rgba(50,35,20,0.75)");
    ctx.set_line_width(1.2);
    for i in 0..8 {
        let y = 66.0 + i as f64 * 14.0;
        if y > h - 40.0 { break; }
        // Random line length (some short as if paragraph endings).
        let line_len = 100.0 + rnd() * 40.0;
        // Wavy cursive simulation — many quadratic segments.
        ctx.begin_path();
        ctx.move_to(24, y);
        let mut cx = 24.0;
        while cx < 24.0 + line_len {
            let seg_len = 6.0 + rnd() * 8.0;
            let nx = cx + seg_len;
            let dy = (rnd() - 0.5) * 4.0;
            ctx.quadratic_curve_to(cx + seg_len / 2.0, y + dy * 1.5, nx, y + dy);
            cx = nx;
        }
        ctx.stroke();
    }
    // Sign-off "Love," on its own line + signature.
    ctx.set_font("italic 15px cursive");
    ctx.set_fill_style("#302010");
    let signoffs = ["Love always,", "Yours,", "With love,", "Missing you,"];
    ctx.fill_text(signoffs[(rnd() * signoffs.len() as f64).floor() as usize], 24, h - 40.0);
    // Signature — big wavy scribble.
    ctx.set_stroke_style("rgba(30,20,10,0.85)");
    ctx.set_line_width(1.6);
    ctx.begin_path();
    ctx.move_to(30, h - 18.0);
    ctx.quadratic_curve_to(45, h - 30.0, 60, h - 20.0);
    ctx.quadratic_curve_to(75, h - 10.0, 90, h - 22.0);
    ctx.quadratic_curve_to(105, h - 34.0, 120, h - 18.0);
    ctx.stroke();
    // Fold crease horizontal.
    ctx.set_stroke_style("rgba(120,90,60,0.30)");
    ctx.set_line_width(1.5);
    ctx.begin_path();
    ctx.move_to(0, h * 0.5);
    ctx.line_to(w, h * 0.5);
    ctx.stroke();
}));

// ------------------------------------------------------------------ line 20070
// R694 open diary page spread: ruled pages, a dated entry, and an entry
// interrupted mid-line with a trailing pen tail.
game::lazy_static_tex!(diary_page_texture, canvas_tex(240, 200, |ctx, w, h| {
    ctx.set_fill_style("#f0e8d0");
    ctx.fill_rect(0, 0, w, h);
    // Ruled blue lines.
    ctx.set_stroke_style("rgba(120,140,180,0.35)");
    ctx.set_line_width(0.7);
    let mut y = 20.0;
    while y < h - 5.0 {
        ctx.begin_path();
        ctx.move_to(6, y);
        ctx.line_to(w - 6.0, y);
        ctx.stroke();
        y += 12.0;
    }
    // Center gutter shadow (spine).
    let shadow = ctx.create_linear_gradient(w / 2.0 - 12.0, 0, w / 2.0 + 12.0, 0);
    shadow.add_color_stop(0, "rgba(0,0,0,0)");
    shadow.add_color_stop(0.5, "rgba(60,40,20,0.35)");
    shadow.add_color_stop(1, "rgba(0,0,0,0)");
    ctx.set_fill_style(&shadow);
    ctx.fill_rect(w / 2.0 - 12.0, 0, 24, h);
    // Date top-right (day entry).
    ctx.set_fill_style("#302010");
    ctx.set_font("italic 12px cursive");
    ctx.set_text_align("right");
    let dates = ["Oct. 14, 1987", "Mar. 22, 1974", "Jul. 4, 1962", "Dec. 18, 1993"];
    ctx.fill_text(dates[(rnd() * dates.len() as f64).floor() as usize], w - 12.0, 16);
    // Left column entry — cursive strokes (reused R693 pattern).
    ctx.set_stroke_style("rgba(30,20,10,0.85)");
    ctx.set_line_width(1.1);
    for row in 0..12 {
        let y = 32.0 + row as f64 * 12.0;
        if y > h - 10.0 { break; }
        let line_len = 60.0 + rnd() * 40.0;
        ctx.begin_path();
        ctx.move_to(10, y);
        let mut cx = 10.0;
        while cx < 10.0 + line_len && cx < w / 2.0 - 20.0 {
            let seg_len = 5.0 + rnd() * 7.0;
            let nx = cx + seg_len;
            let dy = (rnd() - 0.5) * 3.0;
            ctx.quadratic_curve_to(cx + seg_len / 2.0, y + dy * 1.3, nx, y + dy);
            cx = nx;
        }
        ctx.stroke();
    }
    // Right column — mostly blank (interrupted mid-entry), just short lines.
    for row in 0..3 {
        let y = 32.0 + row as f64 * 12.0;
        let line_len = 50.0 + rnd() * 30.0;
        ctx.begin_path();
        ctx.move_to(w / 2.0 + 14.0, y);
        let mut cx = w / 2.0 + 14.0;
        while cx < w / 2.0 + 14.0 + line_len && cx < w - 10.0 {
            let seg_len = 5.0 + rnd() * 7.0;
            let nx = cx + seg_len;
            let dy = (rnd() - 0.5) * 3.0;
            ctx.quadratic_curve_to(cx + seg_len / 2.0, y + dy * 1.3, nx, y + dy);
            cx = nx;
        }
        ctx.stroke();
    }
    // A partial line — starts, then abruptly ends w/ a trailing pen tail.
    ctx.begin_path();
    let partial_y = 32.0 + 3.0 * 12.0;
    ctx.move_to(w / 2.0 + 14.0, partial_y);
    let mut px = w / 2.0 + 14.0;
    let partial_len = 30.0;
    while px < w / 2.0 + 14.0 + partial_len {
        let seg_len = 5.0 + rnd() * 7.0;
        let nx = px + seg_len;
        let dy = (rnd() - 0.5) * 3.0;
        ctx.quadratic_curve_to(px + seg_len / 2.0, partial_y + dy * 1.3, nx, partial_y + dy);
        px = nx;
    }
    // Trailing pen-drag tail — long low-angle line.
    ctx.quadratic_curve_to(px + 30.0, partial_y + 10.0, px + 60.0, partial_y + 25.0);
    ctx.stroke();
    // Small ink blob at end of trailing tail.
    ctx.set_fill_style("rgba(30,20,10,0.85)");
    ctx.begin_path();
    ctx.arc(px + 60.0, partial_y + 25.0, 2.5, 0, PI * 2.0);
    ctx.fill();
}));
