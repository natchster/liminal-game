//! Canvas textures shared by this crate's placers (built once, on the native RNG).
#![allow(unused_variables, unused_mut, unused_assignments, unused_imports, dead_code, clippy::all)]
use game::prelude::*;

/// A uniformly random element (one `rnd()` draw).
fn pick<'a>(a: &[&'a str]) -> &'a str {
    a[(rnd() * a.len() as f64).floor() as usize]
}

game::lazy_static_tex!(
    constellation_embroidery_texture,
    canvas_tex(200, 240, |ctx, w, h| {
        // Deep blue linen background w/ subtle woven texture.
        ctx.set_fill_style("#101830");
        ctx.fill_rect(0, 0, w, h);
        // Subtle weave (grid of tiny lighter dots for linen texture).
        ctx.set_fill_style("rgba(80,100,140,0.15)");
        let (wi, hi) = (w as i32, h as i32);
        let mut y = 0;
        while y < hi {
            let mut x = 0;
            while x < wi {
                if (x + y) % 6 == 0 {
                    ctx.fill_rect(x, y, 1, 1);
                }
                x += 3;
            }
            y += 3;
        }
        // Title top.
        ctx.set_fill_style("#f0e0a0");
        ctx.set_font("italic bold 14px serif");
        ctx.set_text_align("center");
        let chosen = pick(&["ORION", "URSA MAJOR", "CASSIOPEIA", "CYGNUS", "DRACO", "LEO"]);
        ctx.fill_text(chosen, w / 2.0, 24);
        // 30-50 tiny thread "stars" scattered around.
        ctx.set_fill_style("#f0f0e8");
        let star_count = 30 + (rnd() * 20.0).floor() as i32;
        let mut stars: Vec<(f64, f64)> = vec![];
        for _ in 0..star_count {
            let sx = 20.0 + rnd() * (w - 40.0);
            let sy = 40.0 + rnd() * (h - 80.0);
            let sr = 0.8 + rnd() * 1.2;
            stars.push((sx, sy));
            // Cross-stitch style star.
            ctx.begin_path();
            ctx.arc(sx, sy, sr, 0, PI * 2.0);
            ctx.fill();
            if sr > 1.5 {
                ctx.set_stroke_style("rgba(240,240,232,0.8)");
                ctx.set_line_width(0.5);
                ctx.begin_path();
                ctx.move_to(sx - sr - 1.0, sy);
                ctx.line_to(sx + sr + 1.0, sy);
                ctx.move_to(sx, sy - sr - 1.0);
                ctx.line_to(sx, sy + sr + 1.0);
                ctx.stroke();
            }
        }
        // Constellation lines — connect 5-8 nearby stars.
        let line_count = 5 + (rnd() * 4.0).floor() as i32;
        ctx.set_stroke_style("rgba(240,240,232,0.5)");
        ctx.set_line_width(0.6);
        let mut prev = (rnd() * stars.len() as f64).floor() as usize;
        for _ in 0..line_count {
            // Find a nearby star (identity compare: by index).
            let mut best: Option<usize> = None;
            let mut best_dist = f64::INFINITY;
            for (j, s) in stars.iter().enumerate() {
                if j == prev {
                    continue;
                }
                let d = hypot2(s.0 - stars[prev].0, s.1 - stars[prev].1);
                if d > 8.0 && d < best_dist {
                    best_dist = d;
                    best = Some(j);
                }
            }
            let Some(b) = best else { break };
            // Draw stitched line (dashed for cross-stitch feel).
            ctx.save();
            ctx.set_line_dash(&[2.0, 2.0]);
            ctx.begin_path();
            ctx.move_to(stars[prev].0, stars[prev].1);
            ctx.line_to(stars[b].0, stars[b].1);
            ctx.stroke();
            ctx.restore();
            prev = b;
        }
        // Border stitching — 4-side gold thread line at edges.
        ctx.set_stroke_style("#c0a060");
        ctx.set_line_width(1.5);
        ctx.save();
        ctx.set_line_dash(&[4.0, 2.0]);
        ctx.stroke_rect(10, 10, w - 20.0, h - 20.0);
        ctx.restore();
        // Bottom signature/date.
        ctx.set_fill_style("#c0a060");
        ctx.set_font("italic 8px cursive");
        ctx.set_text_align("center");
        let years = [1978, 1985, 1962, 1993, 1971];
        let who = pick(&["E.M.", "A.R.", "S.C.", "M.L."]);
        let year = years[(rnd() * years.len() as f64).floor() as usize];
        ctx.fill_text(&format!("Stitched by {}, {}", who, year), w / 2.0, h - 12.0);
    })
);

game::lazy_static_tex!(
    mission_patch_texture,
    canvas_tex(220, 220, |ctx, w, h| {
        // Background — dark navy square.
        ctx.set_fill_style("#101830");
        ctx.fill_rect(0, 0, w, h);
        let (cx, cy) = (w / 2.0, h / 2.0);
        let patch_r = 85.0;
        // (name, year, bg, accent, craft)
        let missions: [(&str, &str, &str, &str, &str); 4] = [
            ("APOLLO 11", "1969", "#a02020", "#f0e040", "lunar"),
            ("GEMINI IV", "1965", "#404080", "#e0d090", "capsule"),
            ("MERCURY 7", "1962", "#603020", "#f0c040", "capsule"),
            ("SKYLAB", "1973", "#202070", "#e0e0e0", "station"),
        ];
        let m = missions[(rnd() * missions.len() as f64).floor() as usize];
        // Patch body.
        ctx.set_fill_style(m.2);
        ctx.begin_path();
        ctx.arc(cx, cy, patch_r, 0, PI * 2.0);
        ctx.fill();
        // Outer rim.
        ctx.set_stroke_style(m.3);
        ctx.set_line_width(4);
        ctx.begin_path();
        ctx.arc(cx, cy, patch_r, 0, PI * 2.0);
        ctx.stroke();
        // Central spacecraft icon.
        ctx.set_fill_style(m.3);
        if m.4 == "lunar" {
            ctx.fill_rect(cx - 10.0, cy - 10.0, 20, 20);
            ctx.set_stroke_style(m.3);
            ctx.set_line_width(2);
            for i in 0..4 {
                let ang = (i as f64 / 4.0) * PI * 2.0 + PI / 4.0;
                ctx.begin_path();
                ctx.move_to(cx + ang.cos() * 8.0, cy + ang.sin() * 8.0);
                ctx.line_to(cx + ang.cos() * 25.0, cy + ang.sin() * 25.0);
                ctx.stroke();
            }
        } else if m.4 == "capsule" {
            ctx.begin_path();
            ctx.move_to(cx - 15.0, cy + 15.0);
            ctx.line_to(cx - 20.0, cy + 5.0);
            ctx.line_to(cx - 8.0, cy - 20.0);
            ctx.line_to(cx + 8.0, cy - 20.0);
            ctx.line_to(cx + 20.0, cy + 5.0);
            ctx.line_to(cx + 15.0, cy + 15.0);
            ctx.close_path();
            ctx.fill();
        } else {
            // Space station — long cylinder w/ solar panels.
            ctx.fill_rect(cx - 25.0, cy - 4.0, 50, 8);
            ctx.fill_rect(cx - 45.0, cy - 2.0, 15, 4);
            ctx.fill_rect(cx + 30.0, cy - 2.0, 15, 4);
        }
        // Star field in patch background (small white dots).
        ctx.set_fill_style("#f0f0f0");
        for _ in 0..15 {
            let sang = rnd() * PI * 2.0;
            let sr = 30.0 + rnd() * 40.0;
            let sx = cx + sang.cos() * sr;
            let sy = cy + sang.sin() * sr;
            ctx.begin_path();
            ctx.arc(sx, sy, 0.8 + rnd(), 0, PI * 2.0);
            ctx.fill();
        }
        // Mission name.
        ctx.set_fill_style(m.3);
        ctx.set_font("bold 12px serif");
        ctx.set_text_align("center");
        ctx.fill_text(m.0, cx, cy - patch_r + 20.0);
        // Year at bottom.
        ctx.set_font("bold 10px serif");
        ctx.fill_text(m.1, cx, cy + patch_r - 12.0);
        // 3 astronaut names.
        ctx.set_font("bold 7px sans-serif");
        ctx.set_text_align("left");
        let initials: [[&str; 3]; 4] = [
            ["A. SHEPARD", "V. GRISSOM", "J. GLENN"],
            ["N. ARMSTRONG", "B. ALDRIN", "M. COLLINS"],
            ["J. YOUNG", "M. GRISSOM", "E. WHITE"],
            ["C. CONRAD", "J. LOUSMA", "O. GARRIOTT"],
        ];
        let crew = initials[(rnd() * initials.len() as f64).floor() as usize];
        for i in 0..3 {
            ctx.fill_text(crew[i], cx - patch_r + 10.0, cy - 20.0 + i as f64 * 14.0);
        }
        // "NASA" text below patch.
        ctx.set_fill_style("#f0f0f0");
        ctx.set_font("italic bold 10px serif");
        ctx.set_text_align("center");
        ctx.fill_text("NASA", cx, h - 20.0);
    })
);

game::lazy_static_tex!(
    vintage_ad_texture,
    canvas_tex(200, 260, |ctx, w, h| {
        // (name, tag, bg, accent, icon)
        let variants: [(&str, &str, &str, &str, &str); 6] = [
            ("COLA", "REFRESHING & DELICIOUS", "#a02020", "#f0e8c0", "bottle"),
            ("SMOOTHER", "DOCTORS RECOMMEND", "#603030", "#f0e0a0", "cigarette"),
            ("BREAKFAST OF CHAMPIONS", "START YOUR DAY RIGHT", "#f0c040", "#a02020", "cereal"),
            ("FILL 'ER UP", "HAPPY MOTORING", "#204080", "#f0e040", "pump"),
            ("SEE THE USA", "IN YOUR CHEVROLET", "#c04020", "#f0f0e0", "car"),
            ("BEAUTY SLEEP", "FEEL YOUR BEST", "#a06090", "#f0e0f0", "face"),
        ];
        let v = variants[(rnd() * variants.len() as f64).floor() as usize];
        let (name, tag, bg, accent, icon) = v;
        ctx.set_fill_style(bg);
        ctx.fill_rect(0, 0, w, h);
        // Border stripe.
        ctx.set_stroke_style(accent);
        ctx.set_line_width(5);
        ctx.stroke_rect(10, 10, w - 20.0, h - 20.0);
        // Product name.
        ctx.set_fill_style(accent);
        ctx.set_font("bold 26px serif");
        ctx.set_text_align("center");
        ctx.fill_text(name, w / 2.0, 50);
        // Tagline.
        ctx.set_font("italic bold 12px serif");
        ctx.fill_text(tag, w / 2.0, 72);
        // Central product illustration.
        let (cx, cy) = (w / 2.0, h * 0.55);
        ctx.set_fill_style(accent);
        if icon == "bottle" {
            ctx.begin_path();
            ctx.move_to(cx - 15.0, cy - 40.0);
            ctx.line_to(cx - 15.0, cy - 25.0);
            ctx.quadratic_curve_to(cx - 25.0, cy - 15.0, cx - 25.0, cy + 20.0);
            ctx.line_to(cx - 22.0, cy + 45.0);
            ctx.line_to(cx + 22.0, cy + 45.0);
            ctx.line_to(cx + 25.0, cy + 20.0);
            ctx.quadratic_curve_to(cx + 25.0, cy - 15.0, cx + 15.0, cy - 25.0);
            ctx.line_to(cx + 15.0, cy - 40.0);
            ctx.close_path();
            ctx.fill();
            // Bottle cap.
            ctx.set_fill_style(bg);
            ctx.fill_rect(cx - 12.0, cy - 42.0, 24, 6);
            // Label band.
            ctx.set_fill_style(bg);
            ctx.fill_rect(cx - 26.0, cy + 5.0, 52, 20);
        } else if icon == "cigarette" {
            ctx.fill_rect(cx - 32.0, cy - 30.0, 64, 60);
            ctx.set_stroke_style(bg);
            ctx.set_line_width(3);
            ctx.stroke_rect(cx - 32.0, cy - 30.0, 64, 60);
            ctx.set_fill_style(bg);
            ctx.set_font("bold 14px serif");
            ctx.set_text_align("center");
            ctx.fill_text("CAMEL", cx, cy - 5.0);
            ctx.set_font("9px serif");
            ctx.fill_text("20 CLASS A", cx, cy + 10.0);
            // Single cigarette poking out.
            ctx.set_fill_style(accent);
            ctx.fill_rect(cx + 30.0, cy - 40.0, 4, 20);
            ctx.set_fill_style("#a06030");
            ctx.fill_rect(cx + 30.0, cy - 30.0, 4, 5);
        } else if icon == "cereal" {
            ctx.fill_rect(cx - 30.0, cy - 35.0, 60, 70);
            ctx.set_stroke_style(bg);
            ctx.set_line_width(3);
            ctx.stroke_rect(cx - 30.0, cy - 35.0, 60, 70);
            ctx.set_fill_style(bg);
            ctx.set_font("bold 12px serif");
            ctx.set_text_align("center");
            ctx.fill_text("WHEATIES", cx, cy - 10.0);
            ctx.begin_path();
            ctx.arc(cx, cy + 15.0, 12, 0, PI * 2.0);
            ctx.fill();
        } else if icon == "pump" {
            ctx.fill_rect(cx - 20.0, cy - 45.0, 40, 70);
            ctx.set_stroke_style(bg);
            ctx.set_line_width(3);
            ctx.stroke_rect(cx - 20.0, cy - 45.0, 40, 70);
            // Nozzle handle.
            ctx.set_fill_style(accent);
            ctx.fill_rect(cx + 15.0, cy - 20.0, 20, 8);
            // Dial.
            ctx.set_fill_style(bg);
            ctx.begin_path();
            ctx.arc(cx, cy - 25.0, 12, 0, PI * 2.0);
            ctx.fill();
        } else if icon == "car" {
            ctx.begin_path();
            ctx.move_to(cx - 40.0, cy + 10.0);
            ctx.line_to(cx - 30.0, cy - 10.0);
            ctx.line_to(cx - 10.0, cy - 18.0);
            ctx.line_to(cx + 20.0, cy - 18.0);
            ctx.line_to(cx + 35.0, cy - 8.0);
            ctx.line_to(cx + 40.0, cy + 10.0);
            ctx.line_to(cx - 40.0, cy + 10.0);
            ctx.close_path();
            ctx.fill();
            // Wheels.
            ctx.set_fill_style(bg);
            ctx.begin_path();
            ctx.arc(cx - 25.0, cy + 12.0, 8, 0, PI * 2.0);
            ctx.arc(cx + 25.0, cy + 12.0, 8, 0, PI * 2.0);
            ctx.fill();
        } else if icon == "face" {
            ctx.begin_path();
            ctx.arc(cx, cy - 10.0, 25, 0, PI * 2.0);
            ctx.fill();
            // Eyes (2 dots).
            ctx.set_fill_style(bg);
            ctx.begin_path();
            ctx.arc(cx - 8.0, cy - 12.0, 2, 0, PI * 2.0);
            ctx.arc(cx + 8.0, cy - 12.0, 2, 0, PI * 2.0);
            ctx.fill();
            // Smile.
            ctx.set_stroke_style(bg);
            ctx.set_line_width(2);
            ctx.begin_path();
            ctx.arc(cx, cy - 8.0, 8, 0.4, PI - 0.4);
            ctx.stroke();
            // Bottle/product.
            ctx.set_fill_style(accent);
            ctx.fill_rect(cx - 8.0, cy + 25.0, 16, 25);
        }
        // Price/CTA at bottom.
        ctx.set_fill_style(accent);
        ctx.set_font("bold 14px serif");
        ctx.set_text_align("center");
        ctx.fill_text(
            pick(&["NOW ONLY 5¢", "ASK YOUR GROCER", "AVAILABLE EVERYWHERE", "GET YOURS TODAY", "BUY AMERICAN", "ORDER NOW"]),
            w / 2.0,
            h - 25.0,
        );
        // Year — small text bottom.
        ctx.set_font("italic 9px serif");
        ctx.fill_text(pick(&["© 1948", "© 1953", "© 1957", "© 1962", "© 1965"]), w / 2.0, h - 12.0);
    })
);
