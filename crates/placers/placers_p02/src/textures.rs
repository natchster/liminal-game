//! Canvas textures shared by this crate's placers.
use game::prelude::*;

// Open journal pages: simulated handwriting on aged paper.
game::lazy_static_tex!(
    journal_texture,
    canvas_tex_opt(384, 256, false, |ctx, w, h| {
        ctx.set_fill_style("#e8dcc0");
        ctx.fill_rect(0, 0, w, h);
        // Slight yellowing at edges.
        let grad = ctx.create_radial_gradient(w / 2.0, h / 2.0, 100, w / 2.0, h / 2.0, 250);
        grad.add_color_stop(0, "rgba(232,220,192,0)");
        grad.add_color_stop(1, "rgba(160,120,80,0.4)");
        ctx.set_fill_style(&grad);
        ctx.fill_rect(0, 0, w, h);
        // Center spine crease line.
        ctx.set_stroke_style("rgba(80,50,30,0.5)");
        ctx.set_line_width(1);
        ctx.begin_path();
        ctx.move_to(w / 2.0, 20);
        ctx.line_to(w / 2.0, h - 20.0);
        ctx.stroke();
        // Simulated handwriting: wavy lines of "words".
        ctx.set_stroke_style("rgba(40,20,10,0.7)");
        ctx.set_line_width(2);
        for side in 0..2 {
            let start_x = if side == 0 { 30.0 } else { w / 2.0 + 20.0 };
            let end_x = if side == 0 { w / 2.0 - 20.0 } else { w - 30.0 };
            for line in 0..12 {
                let y = 30.0 + line as f64 * 18.0;
                let mut x = start_x;
                while x < end_x {
                    let word_w = 20.0 + rnd() * 30.0;
                    ctx.begin_path();
                    ctx.move_to(x, y);
                    // Squiggly word line.
                    ctx.bezier_curve_to(x + word_w / 3.0, y - 3.0, x + word_w * 2.0 / 3.0, y + 3.0, x + word_w, y);
                    ctx.stroke();
                    x += word_w + 8.0;
                }
            }
        }
    })
);

// Tarot card back: deep purple with gold pattern.
game::lazy_static_tex!(
    tarot_back_texture,
    canvas_tex_opt(128, 200, false, |ctx, w, h| {
        ctx.set_fill_style("#4030c0");
        ctx.fill_rect(0, 0, w, h);
        ctx.set_stroke_style("#d8b040");
        ctx.set_line_width(2);
        // Border.
        ctx.stroke_rect(6, 6, w - 12.0, h - 12.0);
        ctx.stroke_rect(12, 12, w - 24.0, h - 24.0);
        // Center diamond.
        ctx.begin_path();
        ctx.move_to(w / 2.0, 30);
        ctx.line_to(w - 30.0, h / 2.0);
        ctx.line_to(w / 2.0, h - 30.0);
        ctx.line_to(30, h / 2.0);
        ctx.close_path();
        ctx.stroke();
        // Central 8-point star.
        ctx.begin_path();
        for i in 0..8 {
            let a = (i as f64 / 8.0) * PI * 2.0;
            let r = if i % 2 == 0 { 30.0 } else { 15.0 };
            let px = w / 2.0 + a.cos() * r;
            let py = h / 2.0 + a.sin() * r;
            if i == 0 {
                ctx.move_to(px, py);
            } else {
                ctx.line_to(px, py);
            }
        }
        ctx.close_path();
        ctx.stroke();
        ctx.set_fill_style("#d8b040");
        ctx.fill();
    })
);

// Tarot face-up card: IX THE MOON.
game::lazy_static_tex!(
    tarot_face_texture,
    canvas_tex_opt(128, 200, false, |ctx, w, h| {
        ctx.set_fill_style("#e8dcc0");
        ctx.fill_rect(0, 0, w, h);
        ctx.set_stroke_style("#4030c0");
        ctx.set_line_width(2);
        ctx.stroke_rect(6, 6, w - 12.0, h - 12.0);
        // Roman numeral top.
        ctx.set_fill_style("#4030c0");
        ctx.set_font("bold 22px serif");
        ctx.set_text_align("center");
        ctx.set_text_baseline("top");
        ctx.fill_text("IX", w / 2.0, 12);
        // Central symbol (moon crescent).
        ctx.set_line_width(3);
        ctx.begin_path();
        ctx.arc(w / 2.0, h / 2.0 - 10.0, 30, 0.2 * PI, 1.8 * PI);
        ctx.stroke();
        ctx.begin_path();
        ctx.arc(w / 2.0 + 10.0, h / 2.0 - 12.0, 25, 0.4 * PI, 1.6 * PI);
        ctx.stroke();
        // Card name.
        ctx.set_font("bold 14px serif");
        ctx.set_text_baseline("bottom");
        ctx.fill_text("THE MOON", w / 2.0, h - 12.0);
    })
);

// Pentagram chalk drawing (transparent background).
game::lazy_static_tex!(
    pentagram_texture,
    canvas_tex_opt(384, 384, false, |ctx, w, h| {
        ctx.clear_rect(0, 0, w, h);
        ctx.set_stroke_style("rgba(220,220,220,0.85)");
        ctx.set_line_width(4);
        ctx.set_line_cap("round");
        // Outer circle.
        ctx.begin_path();
        ctx.arc(w / 2.0, h / 2.0, w * 0.42, 0, PI * 2.0);
        ctx.stroke();
        // Inner circle.
        ctx.begin_path();
        ctx.arc(w / 2.0, h / 2.0, w * 0.35, 0, PI * 2.0);
        ctx.stroke();
        // 5-point pentagram (star inside).
        let (cx, cy) = (w / 2.0, h / 2.0);
        let r = w * 0.32;
        let mut points: Vec<(f64, f64)> = vec![];
        for i in 0..5 {
            let a = -PI / 2.0 + (i as f64 / 5.0) * PI * 2.0;
            points.push((cx + a.cos() * r, cy + a.sin() * r));
        }
        // Draw pentagram as 5-star (0 -> 2 -> 4 -> 1 -> 3 -> 0).
        ctx.begin_path();
        ctx.move_to(points[0].0, points[0].1);
        ctx.line_to(points[2].0, points[2].1);
        ctx.line_to(points[4].0, points[4].1);
        ctx.line_to(points[1].0, points[1].1);
        ctx.line_to(points[3].0, points[3].1);
        ctx.close_path();
        ctx.stroke();
        // Small runes at 5 outer points.
        ctx.set_font("bold 18px serif");
        ctx.set_text_align("center");
        ctx.set_text_baseline("middle");
        ctx.set_fill_style("rgba(220,220,220,0.75)");
        let runes = ["ᚠ", "ᚢ", "ᚦ", "ᚨ", "ᚱ"];
        for i in 0..5 {
            let a = -PI / 2.0 + (i as f64 / 5.0) * PI * 2.0;
            ctx.fill_text(runes[i], cx + a.cos() * (r + 20.0), cy + a.sin() * (r + 20.0));
        }
    })
);

// Ouija board: wood grain, YES/NO, alphabet, numbers, GOODBYE.
game::lazy_static_tex!(
    ouija_texture,
    canvas_tex_opt(512, 320, false, |ctx, w, h| {
        // Wood-grain background.
        ctx.set_fill_style("#8a5030");
        ctx.fill_rect(0, 0, w, h);
        // Grain streaks.
        for _ in 0..30 {
            ctx.set_stroke_style(&format!("rgba(60,30,20,{})", 0.15 + rnd() * 0.15));
            ctx.set_line_width(1.0 + rnd() * 2.0);
            ctx.begin_path();
            ctx.move_to(0, rnd() * h);
            ctx.quadratic_curve_to(w * 0.5, rnd() * h, w, rnd() * h);
            ctx.stroke();
        }
        ctx.set_fill_style("#e8dcc0");
        ctx.set_font("bold 24px serif");
        ctx.set_text_align("center");
        ctx.set_text_baseline("middle");
        // "YES" top-left, "NO" top-right.
        ctx.fill_text("YES", 60, 30);
        ctx.fill_text("NO", w - 60.0, 30);
        // Alphabet in 2 rows.
        let abc = "ABCDEFGHIJKLM";
        let nop = "NOPQRSTUVWXYZ";
        ctx.set_font("bold 28px serif");
        for i in 0..13 {
            ctx.fill_text(&abc[i..i + 1], 40.0 + i as f64 * 34.0, 90);
            ctx.fill_text(&nop[i..i + 1], 40.0 + i as f64 * 34.0, 145);
        }
        // Numbers 0-9.
        ctx.set_font("bold 24px serif");
        for i in 0..10 {
            ctx.fill_text(&i.to_string(), 90.0 + i as f64 * 34.0, 200);
        }
        // "GOODBYE" at bottom.
        ctx.set_font("bold 26px serif");
        ctx.fill_text("GOODBYE", w / 2.0, 260);
        // Sun/moon symbols in corners (stroked with the last grain style).
        ctx.begin_path();
        ctx.arc(50, 200, 20, 0, PI * 2.0);
        ctx.stroke();
        ctx.begin_path();
        ctx.arc(w - 50.0, 200, 18, 0.2, PI * 1.8);
        ctx.stroke();
    })
);
