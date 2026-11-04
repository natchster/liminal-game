//! Canvas textures shared by this crate's placers (one lazily built texture each).
//! Built on first use on the native RNG (never the level's seeded stream).
use game::prelude::*;

// `clock_face_texture`: 12 tick marks + centre dot.
game::lazy_static_tex!(
    clock_face_texture,
    canvas_tex(160, 160, |ctx, w, h| {
        ctx.set_fill_style("#f0ece0");
        ctx.fill_rect(0, 0, w, h);
        let (cx, cy) = (w / 2.0, h / 2.0);
        // 12 tick marks around edge.
        ctx.set_stroke_style("#202020");
        ctx.set_line_width(3);
        for i in 0..12 {
            let a = (i as f64 / 12.0) * PI * 2.0 - PI / 2.0;
            let r1 = w * 0.44;
            let r2 = if i % 3 == 0 { w * 0.36 } else { w * 0.40 };
            ctx.begin_path();
            ctx.move_to(cx + a.cos() * r1, cy + a.sin() * r1);
            ctx.line_to(cx + a.cos() * r2, cy + a.sin() * r2);
            ctx.stroke();
        }
        // Center dot.
        ctx.set_fill_style("#202020");
        ctx.begin_path();
        ctx.arc(cx, cy, 4, 0, PI * 2.0);
        ctx.fill();
    })
);

// `photo_texture`: faded portrait (sky gradient + face oval + body triangle).
game::lazy_static_tex!(
    photo_texture,
    canvas_tex(96, 128, |ctx, w, h| {
        let grad = ctx.create_linear_gradient(0, 0, 0, h);
        grad.add_color_stop(0, "#a0b0c8");
        grad.add_color_stop(1, "#607080");
        ctx.set_fill_style(&grad);
        ctx.fill_rect(0, 0, w, h);
        // Soft head silhouette (oval).
        ctx.set_fill_style("#d8b090");
        ctx.begin_path();
        ctx.ellipse(w * 0.5, h * 0.35, w * 0.20, h * 0.16, 0, 0, PI * 2.0);
        ctx.fill();
        // Body triangle.
        ctx.set_fill_style("#405078");
        ctx.begin_path();
        ctx.move_to(w * 0.5, h * 0.45);
        ctx.line_to(w * 0.2, h);
        ctx.line_to(w * 0.8, h);
        ctx.fill();
        // Faded overlay (age).
        ctx.set_fill_style("rgba(200, 180, 140, 0.25)");
        ctx.fill_rect(0, 0, w, h);
    })
);

// `receipt_texture`: thin white strip with printed "text" lines + barcode.
game::lazy_static_tex!(
    receipt_texture,
    canvas_tex(48, 200, |ctx, w, h| {
        ctx.set_fill_style("#f8f4e8");
        ctx.fill_rect(0, 0, w, h);
        // Header dark bar.
        ctx.set_fill_style("#202020");
        ctx.fill_rect(6, 8, w - 12.0, 4);
        ctx.fill_rect(6, 16, w - 12.0, 2);
        // Text lines (uneven length).
        for i in 0..14 {
            ctx.set_fill_style("#404040");
            let y = 30.0 + i as f64 * 10.0;
            let line_w = 8.0 + rnd() * (w - 22.0);
            ctx.fill_rect(6, y, line_w, 2);
        }
        // Footer barcode-style bars.
        let bar_y = 180.0;
        ctx.set_fill_style("#202020");
        for i in 0..20 {
            ctx.fill_rect(6.0 + i as f64 * 1.8, bar_y, 1.0 + rnd() * 1.0, 12);
        }
    })
);

// `barometer_texture`: 270 degree arc of ticks + RAIN / CHANGE / FAIR labels.
game::lazy_static_tex!(
    barometer_texture,
    canvas_tex(160, 160, |ctx, w, h| {
        ctx.set_fill_style("#e0d8c0");
        ctx.fill_rect(0, 0, w, h);
        let (cx, cy) = (w / 2.0, h / 2.0);
        ctx.set_stroke_style("#302818");
        ctx.set_line_width(2);
        for i in 0..=24 {
            let t = i as f64 / 24.0;
            let a = -PI * 1.25 + t * PI * 1.5; // 270 degree arc.
            let r1 = w * 0.42;
            let r2 = if i % 4 == 0 { w * 0.33 } else { w * 0.38 };
            ctx.begin_path();
            ctx.move_to(cx + a.cos() * r1, cy + a.sin() * r1);
            ctx.line_to(cx + a.cos() * r2, cy + a.sin() * r2);
            ctx.stroke();
        }
        // Labels: RAIN CHANGE FAIR.
        ctx.set_fill_style("#302818");
        ctx.set_font("bold 12px sans-serif");
        ctx.set_text_align("center");
        ctx.fill_text("RAIN", cx - w * 0.30, cy);
        ctx.fill_text("FAIR", cx + w * 0.30, cy);
        ctx.fill_text("CHANGE", cx, cy - h * 0.28);
    })
);

// `card_texture`: business card (name line, subtitle, contact lines, logo dot).
game::lazy_static_tex!(
    card_texture,
    canvas_tex(96, 56, |ctx, w, h| {
        ctx.set_fill_style("#f8f4e8");
        ctx.fill_rect(0, 0, w, h);
        // Top name line (bold thick).
        ctx.set_fill_style("#202020");
        ctx.fill_rect(6, 12, w * 0.55, 3);
        // Subtitle.
        ctx.set_fill_style("#606060");
        ctx.fill_rect(6, 19, w * 0.35, 2);
        // Contact lines.
        for i in 0..3 {
            ctx.fill_rect(6, 30.0 + i as f64 * 6.0, w * (0.4 + rnd() * 0.3), 1);
        }
        // Small dark logo circle.
        ctx.set_fill_style("#802020");
        ctx.begin_path();
        ctx.arc(w - 12.0, 12, 4, 0, PI * 2.0);
        ctx.fill();
    })
);

// `diploma_texture`: parchment, double border, title, script lines, seal, signature.
game::lazy_static_tex!(
    diploma_texture,
    canvas_tex(160, 200, |ctx, w, h| {
        // Cream parchment.
        ctx.set_fill_style("#f0e8c8");
        ctx.fill_rect(0, 0, w, h);
        // Decorative border (double-line rectangle).
        ctx.set_stroke_style("#604020");
        ctx.set_line_width(2);
        ctx.stroke_rect(8, 8, w - 16.0, h - 16.0);
        ctx.stroke_rect(14, 14, w - 28.0, h - 28.0);
        // Header title bar.
        ctx.set_fill_style("#302010");
        ctx.set_font("bold 14px serif");
        ctx.set_text_align("center");
        ctx.fill_text("CERTIFICATE", w / 2.0, 40);
        // 6 script-line placeholders.
        ctx.set_fill_style("#403020");
        for i in 0..6 {
            let y = 60.0 + i as f64 * 14.0;
            let line_w = w * (0.5 + rnd() * 0.3);
            ctx.fill_rect((w - line_w) / 2.0, y, line_w, 2);
        }
        // Circle seal (red).
        ctx.set_fill_style("#a02020");
        ctx.begin_path();
        ctx.arc(w * 0.25, h * 0.85, 14, 0, PI * 2.0);
        ctx.fill();
        // Signature line.
        ctx.set_stroke_style("#403020");
        ctx.set_line_width(1);
        ctx.begin_path();
        ctx.move_to(w * 0.5, h * 0.85);
        ctx.line_to(w * 0.9, h * 0.85);
        ctx.stroke();
    })
);

// `card_face_texture`: playing card with a random suit + rank.
game::lazy_static_tex!(
    card_face_texture,
    canvas_tex(64, 96, |ctx, w, h| {
        ctx.set_fill_style("#fafaf4");
        ctx.fill_rect(0, 0, w, h);
        // Border.
        ctx.set_stroke_style("#606060");
        ctx.set_line_width(1);
        ctx.stroke_rect(2, 2, w - 4.0, h - 4.0);
        // Random suit + rank.
        let suits = ["\u{2665}", "\u{2666}", "\u{2660}", "\u{2663}"];
        let ranks = ["A", "K", "Q", "J", "10", "7", "3"];
        let suit = suits[(rnd() * suits.len() as f64).floor() as usize];
        let rank = ranks[(rnd() * ranks.len() as f64).floor() as usize];
        let red = suit == "\u{2665}" || suit == "\u{2666}";
        ctx.set_fill_style(if red { "#c02020" } else { "#202020" });
        ctx.set_font("bold 14px sans-serif");
        ctx.set_text_align("left");
        // Top-left corner.
        ctx.fill_text(rank, 5, 16);
        ctx.set_font("14px sans-serif");
        ctx.fill_text(suit, 5, 32);
        // Center big suit.
        ctx.set_font("bold 36px sans-serif");
        ctx.set_text_align("center");
        ctx.fill_text(suit, w / 2.0, h / 2.0 + 12.0);
        // Bottom-right corner (rotated).
        ctx.save();
        ctx.translate(w - 5.0, h - 6.0);
        ctx.rotate(PI);
        ctx.set_font("bold 14px sans-serif");
        ctx.set_text_align("left");
        ctx.fill_text(rank, 0, 8);
        ctx.set_font("14px sans-serif");
        ctx.fill_text(suit, 0, 24);
        ctx.restore();
    })
);

// `die_face_texture`: one random die face (1-6 pips).
game::lazy_static_tex!(
    die_face_texture,
    canvas_tex(64, 64, |ctx, w, h| {
        ctx.set_fill_style("#f8f4e8");
        ctx.fill_rect(0, 0, w, h);
        // Pick a random face 1-6 for display purposes.
        let pips = 1 + (rnd() * 6.0).floor() as usize;
        ctx.set_fill_style("#202020");
        let positions: [&[(f64, f64)]; 6] = [
            &[(0.5, 0.5)],
            &[(0.25, 0.25), (0.75, 0.75)],
            &[(0.25, 0.25), (0.5, 0.5), (0.75, 0.75)],
            &[(0.25, 0.25), (0.75, 0.25), (0.25, 0.75), (0.75, 0.75)],
            &[(0.25, 0.25), (0.75, 0.25), (0.5, 0.5), (0.25, 0.75), (0.75, 0.75)],
            &[(0.25, 0.25), (0.75, 0.25), (0.25, 0.5), (0.75, 0.5), (0.25, 0.75), (0.75, 0.75)],
        ];
        for &(px, py) in positions[pips - 1] {
            ctx.begin_path();
            ctx.arc(w * px, h * py, 5, 0, PI * 2.0);
            ctx.fill();
        }
    })
);

// `map_texture`: beige map, fold creases, winding roads, landmark dots.
game::lazy_static_tex!(
    map_texture,
    canvas_tex(96, 96, |ctx, w, h| {
        // Beige base.
        ctx.set_fill_style("#e8d8a0");
        ctx.fill_rect(0, 0, w, h);
        // Fold crease lines (dark).
        ctx.set_stroke_style("rgba(120, 90, 40, 0.6)");
        ctx.set_line_width(1);
        ctx.begin_path();
        ctx.move_to(w / 3.0, 0);
        ctx.line_to(w / 3.0, h);
        ctx.move_to(2.0 * w / 3.0, 0);
        ctx.line_to(2.0 * w / 3.0, h);
        ctx.move_to(0, h / 3.0);
        ctx.line_to(w, h / 3.0);
        ctx.move_to(0, 2.0 * h / 3.0);
        ctx.line_to(w, 2.0 * h / 3.0);
        ctx.stroke();
        // Road lines (winding).
        ctx.set_stroke_style("#804020");
        ctx.set_line_width(1.5);
        for _i in 0..3 {
            ctx.begin_path();
            ctx.move_to(rnd() * w, 0);
            ctx.bezier_curve_to(w * rnd(), h * 0.3, w * rnd(), h * 0.7, rnd() * w, h);
            ctx.stroke();
        }
        // Small green landmark dots.
        ctx.set_fill_style("#406030");
        for _i in 0..6 {
            ctx.begin_path();
            ctx.arc(rnd() * w, rnd() * h, 2, 0, PI * 2.0);
            ctx.fill();
        }
        // Red star landmark.
        ctx.set_fill_style("#c02020");
        ctx.begin_path();
        ctx.arc(w * 0.5, h * 0.5, 4, 0, PI * 2.0);
        ctx.fill();
    })
);
