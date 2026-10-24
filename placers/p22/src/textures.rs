//! Module-level canvas textures of chunk p22 (built on first use, native RNG).
use crate::u8_clamped;
use game::prelude::*;

// ------------------------------------------------------------------ line 40265
game::lazy_static_tex!(crt_static_texture, canvas_tex(128, 96, |ctx, w, h| {
    // Gray background.
    ctx.set_fill_style("#404448");
    ctx.fill_rect(0, 0, w, h);
    // Random static noise pixels.
    let mut img = ctx.get_image_data(0, 0, w, h);
    let n = img.data.len();
    let mut i = 0;
    while i < n {
        let v = u8_clamped(40.0 + rnd() * 180.0);
        img.data[i] = v;
        img.data[i + 1] = v;
        img.data[i + 2] = v;
        img.data[i + 3] = 255;
        i += 4;
    }
    ctx.put_image_data(&img, 0, 0);
    // Horizontal scan lines.
    ctx.set_fill_style("rgba(0,0,0,0.15)");
    let mut y = 0.0;
    while y < h {
        ctx.fill_rect(0, y, w, 1);
        y += 3.0;
    }
    // Faint color banding.
    ctx.set_fill_style("rgba(80,120,180,0.10)");
    let mut band_y = 20.0;
    while band_y < h {
        ctx.fill_rect(0, band_y, w, 8);
        band_y += 30.0;
    }
}));

// ------------------------------------------------------------------ line 40418
game::lazy_static_tex!(wallpaper_texture, canvas_tex(192, 320, |ctx, w, h| {
    // Base cream color.
    ctx.set_fill_style("#e8d8c0");
    ctx.fill_rect(0, 0, w, h);
    // Faded vertical stripes pattern.
    ctx.set_fill_style("rgba(160,110,80,0.35)");
    let mut x2 = 0.0;
    while x2 < w {
        ctx.fill_rect(x2, 0, 4, h);
        x2 += 30.0;
    }
    // Faded flower rosettes.
    ctx.set_fill_style("rgba(140,80,90,0.4)");
    let mut ry = 20.0;
    while ry < h {
        let mut rx = 15.0;
        while rx < w {
            let jx = rx + rnd() * 6.0;
            let jy = ry + rnd() * 6.0;
            ctx.begin_path();
            for p in 0..6 {
                let a = (p as f64 / 6.0) * PI * 2.0;
                ctx.arc(jx + a.cos() * 4.0, jy + a.sin() * 4.0, 4, 0, PI * 2.0);
            }
            ctx.fill();
            rx += 42.0;
        }
        ry += 55.0;
    }
    // Water stains.
    ctx.set_fill_style("rgba(80,50,20,0.25)");
    for _ in 0..8 {
        ctx.begin_path();
        ctx.ellipse(rnd() * w, rnd() * h, 20.0 + rnd() * 30.0, 10.0 + rnd() * 20.0, rnd() * PI, 0, PI * 2.0);
        ctx.fill();
    }
}));

// ------------------------------------------------------------------ line 40559
game::lazy_static_tex!(triptych_texture, canvas_tex(320, 256, |ctx, w, h| {
    // Dark gold background.
    ctx.set_fill_style("#a08040");
    ctx.fill_rect(0, 0, w, h);
    // 3 dark panel wells.
    let panel_w = w / 3.0;
    for p in 0..3 {
        let p = p as f64;
        ctx.set_fill_style("#2a1810");
        ctx.fill_rect(p * panel_w + 8.0, 12, panel_w - 16.0, h - 24.0);
        // Halo circle.
        ctx.set_fill_style("#d8b040");
        ctx.begin_path();
        ctx.arc(p * panel_w + panel_w / 2.0, 60, 22, 0, PI * 2.0);
        ctx.fill();
        // Figure body (upright robe silhouette).
        ctx.set_fill_style("#4a2818");
        ctx.begin_path();
        ctx.move_to(p * panel_w + panel_w / 2.0 - 30.0, h - 24.0);
        ctx.line_to(p * panel_w + panel_w / 2.0 + 30.0, h - 24.0);
        ctx.line_to(p * panel_w + panel_w / 2.0 + 20.0, 90);
        ctx.line_to(p * panel_w + panel_w / 2.0 - 20.0, 90);
        ctx.close_path();
        ctx.fill();
        // Face (small oval).
        ctx.set_fill_style("#f0d8c0");
        ctx.begin_path();
        ctx.ellipse(p * panel_w + panel_w / 2.0, 60, 12, 15, 0, 0, PI * 2.0);
        ctx.fill();
    }
}));

// ------------------------------------------------------------------ line 41205
game::lazy_static_tex!(cracked_mirror_texture, canvas_tex(192, 256, |ctx, w, h| {
    // Mirror surface base (cool gray-blue).
    ctx.set_fill_style("#506878");
    ctx.fill_rect(0, 0, w, h);
    // Random dark specks.
    ctx.set_fill_style("rgba(20,10,5,0.35)");
    for _ in 0..30 {
        ctx.begin_path();
        ctx.arc(rnd() * w, rnd() * h, 2.0 + rnd() * 3.0, 0, PI * 2.0);
        ctx.fill();
    }
    // Central impact point.
    let cx = w * 0.35 + rnd() * w * 0.3;
    let cy = h * 0.35 + rnd() * h * 0.3;
    // Radial cracks (10 lines from center).
    ctx.set_stroke_style("rgba(0,0,0,0.85)");
    ctx.set_line_width(2);
    for i in 0..10 {
        let a = (i as f64 / 10.0) * PI * 2.0 + rnd() * 0.3;
        let len = 80.0 + rnd() * 60.0;
        ctx.begin_path();
        ctx.move_to(cx, cy);
        // Zigzag path.
        let (mut x, mut y) = (cx, cy);
        for _ in 0..5 {
            let step_len = len / 5.0;
            let jitter = (rnd() - 0.5) * 15.0;
            let aa = a + (jitter / len);
            x += aa.cos() * step_len;
            y += aa.sin() * step_len;
            ctx.line_to(x, y);
        }
        ctx.stroke();
    }
    // Concentric crack rings (3 rings).
    ctx.set_line_width(1.5);
    let mut r = 30.0;
    while r < 100.0 {
        ctx.begin_path();
        let mut a = 0.0;
        while a < PI * 2.0 {
            let jr = r + (rnd() - 0.5) * 8.0;
            let px = cx + a.cos() * jr;
            let py = cy + a.sin() * jr;
            if a == 0.0 { ctx.move_to(px, py); } else { ctx.line_to(px, py); }
            a += PI / 12.0;
        }
        ctx.stroke();
        r += 25.0;
    }
    // Impact center dark hole.
    ctx.set_fill_style("rgba(0,0,0,0.95)");
    ctx.begin_path();
    ctx.arc(cx, cy, 6, 0, PI * 2.0);
    ctx.fill();
}));

// ------------------------------------------------------------------ line 41391
game::lazy_static_tex!(scattered_page_texture, canvas_tex_opt(128, 160, false, |ctx, w, h| {
    ctx.set_fill_style("#f0e8d8");
    ctx.fill_rect(0, 0, w, h);
    // Age tint corner.
    let grad = ctx.create_radial_gradient(w / 2.0, h / 2.0, 30, w / 2.0, h / 2.0, 90);
    grad.add_color_stop(0, "rgba(240,232,216,0)");
    grad.add_color_stop(1, "rgba(160,120,80,0.35)");
    ctx.set_fill_style(&grad);
    ctx.fill_rect(0, 0, w, h);
    // 8 lines of squiggly text.
    ctx.set_stroke_style("rgba(40,20,10,0.7)");
    ctx.set_line_width(1.5);
    for line in 0..8 {
        let y = 12.0 + line as f64 * 18.0;
        let mut x = 8.0;
        while x < w - 8.0 {
            let word_w = 12.0 + rnd() * 24.0;
            ctx.begin_path();
            ctx.move_to(x, y);
            ctx.bezier_curve_to(x + word_w / 3.0, y - 2.0, x + word_w * 2.0 / 3.0, y + 2.0, x + word_w, y);
            ctx.stroke();
            x += word_w + 6.0;
        }
    }
}));

// ------------------------------------------------------------------ line 41457
game::lazy_static_tex!(chalkboard_texture, canvas_tex(256, 192, |ctx, w, h| {
    ctx.set_fill_style("#1a3a2a");
    ctx.fill_rect(0, 0, w, h);
    // Faint chalk smears (light wipe patterns).
    ctx.set_fill_style("rgba(200,220,200,0.10)");
    for _ in 0..20 {
        ctx.begin_path();
        ctx.ellipse(rnd() * w, rnd() * h, 30.0 + rnd() * 40.0, 8.0 + rnd() * 8.0, rnd() * PI, 0, PI * 2.0);
        ctx.fill();
    }
    // 4-5 lines of ghostly writing.
    ctx.set_stroke_style("rgba(220,220,220,0.6)");
    ctx.set_line_width(2);
    ctx.set_font("bold 22px monospace");
    ctx.set_fill_style("rgba(220,220,220,0.65)");
    let messages = ["ROOM 42", "DO NOT ENTER", "THEY WATCH", "THE END", "???"];
    for (i, msg) in messages.iter().enumerate() {
        let y = 28.0 + i as f64 * 34.0;
        ctx.fill_text(msg, 12, y);
    }
}));

// ------------------------------------------------------------------ line 41736
game::lazy_static_tex!(broken_clock_texture, canvas_tex(128, 128, |ctx, w, h| {
    ctx.set_fill_style("#e8dcc0");
    ctx.begin_path();
    ctx.arc(w / 2.0, h / 2.0, w / 2.0 - 4.0, 0, PI * 2.0);
    ctx.fill();
    ctx.set_stroke_style("#181008");
    ctx.set_line_width(3);
    ctx.stroke();
    // Age grime.
    for _ in 0..15 {
        ctx.set_fill_style(&format!("rgba(100,80,40,{})", 0.05 + rnd() * 0.15));
        ctx.begin_path();
        ctx.arc(20.0 + rnd() * (w - 40.0), 20.0 + rnd() * (h - 40.0), 4.0 + rnd() * 8.0, 0, PI * 2.0);
        ctx.fill();
    }
    // Hour marks.
    ctx.set_stroke_style("#181008");
    ctx.set_line_width(3);
    for i in 0..12 {
        let a = -PI / 2.0 + (i as f64 / 12.0) * PI * 2.0;
        let (r0, r1) = (44.0, 52.0);
        ctx.begin_path();
        ctx.move_to(w / 2.0 + a.cos() * r0, h / 2.0 + a.sin() * r0);
        ctx.line_to(w / 2.0 + a.cos() * r1, h / 2.0 + a.sin() * r1);
        ctx.stroke();
    }
    // Broken hands at 3:33 (unsettling non-round time).
    ctx.set_line_width(4);
    ctx.begin_path();
    ctx.move_to(w / 2.0, h / 2.0);
    let hour_a = -PI / 2.0 + PI / 2.0 * 1.11; // ~3:33
    ctx.line_to(w / 2.0 + hour_a.cos() * 24.0, h / 2.0 + hour_a.sin() * 24.0);
    ctx.stroke();
    ctx.set_line_width(3);
    ctx.begin_path();
    ctx.move_to(w / 2.0, h / 2.0);
    let min_a = -PI / 2.0 + PI * 1.1; // ~33 min
    ctx.line_to(w / 2.0 + min_a.cos() * 38.0, h / 2.0 + min_a.sin() * 38.0);
    ctx.stroke();
    ctx.begin_path();
    ctx.arc(w / 2.0, h / 2.0, 4, 0, PI * 2.0);
    ctx.set_fill_style("#181008");
    ctx.fill();
    // Big crack across face.
    ctx.set_stroke_style("rgba(0,0,0,0.55)");
    ctx.set_line_width(2);
    ctx.begin_path();
    ctx.move_to(30, 20);
    ctx.line_to(80, 60);
    ctx.line_to(60, 95);
    ctx.line_to(100, 110);
    ctx.stroke();
    // Small branch cracks.
    ctx.set_line_width(1);
    for _ in 0..5 {
        ctx.begin_path();
        ctx.move_to(60.0 + rnd() * 30.0, 40.0 + rnd() * 40.0);
        ctx.line_to(60.0 + rnd() * 40.0, 40.0 + rnd() * 40.0);
        ctx.stroke();
    }
}));
