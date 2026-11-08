//! Canvas textures shared by this crate's placers (built lazily on the native RNG).
#![allow(unused_variables, unused_mut, unused_assignments, unused_imports, dead_code, clippy::all)]
use game::prelude::*;

/// Laptop screen textures (4 variants: 3 garbled tints + BSOD)
pub fn laptop_screen_textures() -> Vec<Tex> {
    thread_local! { static V: std::cell::OnceCell<Vec<Tex>> = const { std::cell::OnceCell::new() }; }
    V.with(|c| {
        c.get_or_init(|| {
            with_native(|| {
                let glyphs: Vec<char> = "!@#$%^&*()_+-=[]{}|;:,.<>?/\\~`0123456789ABCDEF#*.".chars().collect();
                // (bg, fg, line, bsod)
                let variants: [(&str, &str, &str, bool); 4] = [
                    ("#0a1830", "#a0c8f0", "rgba(160,200,240,0.10)", false),
                    ("#1a1040", "#c0a0f0", "rgba(190,160,240,0.10)", false),
                    ("#301820", "#f0a0a0", "rgba(240,160,160,0.10)", false),
                    // BSOD variant — classic blue with :( face, error, fake trace.
                    ("#183090", "#f0f0f0", "rgba(255,255,255,0.05)", true),
                ];
                variants
                    .iter()
                    .map(|&(bg, fg, line, bsod)| {
                        canvas_tex_opt(128, 96, false, |ctx, w, h| {
                            ctx.set_fill_style(bg);
                            ctx.fill_rect(0, 0, w, h);
                            // Horizontal scan lines.
                            ctx.set_fill_style(line);
                            let mut y = 0.0;
                            while y < h {
                                ctx.fill_rect(0, y, w, 1);
                                y += 2.0;
                            }
                            if bsod {
                                // BSOD: sad face + error code + fake stack lines.
                                ctx.set_fill_style(fg);
                                ctx.set_font("bold 20px sans-serif");
                                ctx.set_text_align("left");
                                ctx.set_text_baseline("top");
                                ctx.fill_text(":(", 8, 4);
                                ctx.set_font("7px monospace");
                                ctx.fill_text("SYSTEM_FAILURE", 8, 32);
                                ctx.fill_text("0x000000C4", 8, 42);
                                let trace_lines = ["ntoskrnl.exe+0x1c", "kernel32+0x28", "user32.dll+0x40", "hal.dll+0x0c"];
                                for (i, tl) in trace_lines.iter().enumerate() {
                                    ctx.fill_text(tl, 8, 56.0 + i as f64 * 10.0);
                                }
                            } else {
                                // Random glyphs in a rough grid.
                                ctx.set_fill_style(fg);
                                ctx.set_font("9px monospace");
                                ctx.set_text_baseline("top");
                                let mut y = 4.0;
                                while y < h - 6.0 {
                                    let mut x = 4.0;
                                    while x < w - 6.0 {
                                        if rnd() < 0.55 {
                                            let gch = glyphs[(rnd() * glyphs.len() as f64).floor() as usize];
                                            ctx.set_global_alpha(0.5 + rnd() * 0.5);
                                            ctx.fill_text(&gch.to_string(), x, y);
                                        }
                                        x += 8.0;
                                    }
                                    y += 10.0;
                                }
                                ctx.set_global_alpha(1);
                                // A few brighter runs (dense clusters) — reads as "corrupted output".
                                for _i in 0..3 {
                                    let cy = 8.0 + rnd() * (h - 20.0);
                                    let cx = 4.0 + rnd() * (w - 40.0);
                                    ctx.set_fill_style(fg);
                                    // the loop bound draws a fresh `rnd()` every iteration
                                    let mut gi = 0;
                                    while (gi as f64) < 4.0 + rnd() * 4.0 {
                                        let gch = glyphs[(rnd() * glyphs.len() as f64).floor() as usize];
                                        ctx.fill_text(&gch.to_string(), cx + gi as f64 * 7.0, cy);
                                        gi += 1;
                                    }
                                }
                            }
                        })
                    })
                    .collect()
            })
        })
        .clone()
    })
}

// Laptop keyboard canvas texture.
game::lazy_static_tex!(
    laptop_keyboard_texture,
    canvas_tex_opt(256, 128, false, |ctx, w, h| {
        ctx.set_fill_style("#151518");
        ctx.fill_rect(0, 0, w, h);
        let (cols, rows) = (13.0, 5.0);
        let (key_w, key_h) = (15.0, 15.0);
        let margin_x = (w - cols * key_w - (cols - 1.0) * 3.0) / 2.0;
        let margin_y = (h - rows * key_h - (rows - 1.0) * 3.0) / 2.0;
        ctx.set_fill_style("#2a2a2e");
        for r in 0..5 {
            for c in 0..13 {
                let x = margin_x + c as f64 * (key_w + 3.0);
                let y = margin_y + r as f64 * (key_h + 3.0);
                ctx.fill_rect(x, y, key_w, key_h);
                ctx.set_fill_style("#3a3a3e");
                ctx.fill_rect(x + 1.0, y + 1.0, key_w - 2.0, key_h - 2.0);
                ctx.set_fill_style("#2a2a2e");
            }
        }
        // Trackpad below keyboard.
        let (tp_w, tp_h) = (90.0, 30.0);
        let tp_x = (w - tp_w) / 2.0;
        let tp_y = h - tp_h - 5.0;
        ctx.set_fill_style("#1a1a1e");
        ctx.fill_rect(tp_x, tp_y, tp_w, tp_h);
        ctx.set_stroke_style("#2a2a2e");
        ctx.set_line_width(1);
        ctx.stroke_rect(tp_x, tp_y, tp_w, tp_h);
    })
);

#[derive(Clone, Copy)]
enum PhotoKind {
    Shapes(&'static [[f64; 4]]),
    Silhouette,
    Eyes,
    Doorway,
    NightSky,
    StarCluster,
    RainWindow,
    Hallway,
    MirrorRefl,
    FoundFootage,
    ClockFace,
    TvStatic,
    Negative,
    Halo,
}

/// Photo textures (16 variants)
pub fn photo_textures() -> Vec<Tex> {
    thread_local! { static V: std::cell::OnceCell<Vec<Tex>> = const { std::cell::OnceCell::new() }; }
    V.with(|c| {
        c.get_or_init(|| {
            with_native(|| {
                use PhotoKind::*;
                let variants: [(&str, PhotoKind); 16] = [
                    ("#c8b898", Shapes(&[[24.0, 20.0, 24.0, 32.0], [50.0, 20.0, 24.0, 32.0]])), // 2 vague figures
                    ("#8a8878", Shapes(&[[10.0, 40.0, 76.0, 20.0]])),                           // horizon line
                    ("#a8988a", Shapes(&[[36.0, 22.0, 24.0, 40.0]])),                           // single central subject
                    ("#b0a8a0", Silhouette),
                    ("#4a4038", Eyes),
                    ("#2a2620", Doorway),
                    ("#0a0a12", NightSky),
                    ("#100815", StarCluster),
                    ("#3a4048", RainWindow),
                    ("#3a3020", Hallway),
                    ("#28241e", MirrorRefl),
                    ("#0a0a08", FoundFootage),
                    ("#3a3630", ClockFace),
                    ("#1a1a1a", TvStatic),
                    ("#c8c0b8", Negative),
                    ("#100a10", Halo),
                ];
                let mut textures = vec![];
                for &(bg, kind) in variants.iter() {
                    textures.push(canvas_tex_opt(96, 96, false, |ctx, w, h| {
                        ctx.set_fill_style(bg);
                        ctx.fill_rect(0, 0, w, h);
                        // Grunge speckle.
                        for _ in 0..30 {
                            ctx.set_fill_style(&format!("rgba(0,0,0,{})", 0.05 + rnd() * 0.05));
                            ctx.fill_rect(rnd() * w, rnd() * h, 1, 1);
                        }
                        ctx.set_fill_style("#2a1a12");
                        match kind {
                            Silhouette => {
                                ctx.begin_path();
                                ctx.arc(w / 2.0, 32, 10, 0, PI * 2.0);
                                ctx.fill();
                                ctx.begin_path();
                                ctx.ellipse(w / 2.0, 66, 16, 26, 0, 0, PI * 2.0);
                                ctx.fill();
                            }
                            Doorway => {
                                ctx.set_fill_style("#d8c898");
                                ctx.fill_rect(30, 22, 36, 52);
                                ctx.set_stroke_style("#1a1610");
                                ctx.set_line_width(4);
                                ctx.stroke_rect(30, 22, 36, 52);
                                ctx.set_fill_style("#3a2818");
                                ctx.fill_rect(45, 44, 6, 22);
                            }
                            NightSky => {
                                ctx.set_fill_style("#e8e4c8");
                                for _ in 0..40 {
                                    let sx = rnd() * w;
                                    let sy = rnd() * h;
                                    let s_size = if rnd() < 0.15 { 2.0 } else { 1.0 };
                                    ctx.set_global_alpha(0.4 + rnd() * 0.5);
                                    ctx.fill_rect(sx, sy, s_size, s_size);
                                }
                                ctx.set_global_alpha(1);
                                ctx.set_fill_style("#f0eee0");
                                ctx.begin_path();
                                ctx.arc(68, 28, 7, 0, PI * 2.0);
                                ctx.fill();
                                ctx.set_fill_style("rgba(240,238,224,0.25)");
                                ctx.begin_path();
                                ctx.arc(68, 28, 11, 0, PI * 2.0);
                                ctx.fill();
                            }
                            Halo => {
                                let (cx, cy) = (w / 2.0, h / 2.0);
                                for r in 0..5 {
                                    let rad = 8.0 + r as f64 * 8.0;
                                    let alpha = 0.65 - r as f64 * 0.11;
                                    ctx.set_stroke_style(&format!("rgba(220,200,160,{})", alpha));
                                    ctx.set_line_width(1.6);
                                    ctx.begin_path();
                                    ctx.arc(cx, cy, rad, 0, PI * 2.0);
                                    ctx.stroke();
                                }
                                ctx.set_fill_style("rgba(240,220,180,0.85)");
                                ctx.begin_path();
                                ctx.arc(cx, cy, 3, 0, PI * 2.0);
                                ctx.fill();
                                let grd = ctx.create_radial_gradient(cx, cy, 12, cx, cy, 55);
                                grd.add_color_stop(0, "rgba(0,0,0,0)");
                                grd.add_color_stop(1, "rgba(0,0,0,0.4)");
                                ctx.set_fill_style(&grd);
                                ctx.fill_rect(0, 0, w, h);
                            }
                            Negative => {
                                ctx.set_fill_style("#f0e8d8");
                                ctx.begin_path();
                                ctx.arc(w / 2.0, 32, 10, 0, PI * 2.0);
                                ctx.fill();
                                ctx.begin_path();
                                ctx.ellipse(w / 2.0, 66, 16, 26, 0, 0, PI * 2.0);
                                ctx.fill();
                                for _ in 0..40 {
                                    ctx.set_fill_style(&format!("rgba(0,0,0,{})", 0.10 + rnd() * 0.15));
                                    ctx.fill_rect(rnd() * w, rnd() * h, 1, 1);
                                }
                                ctx.set_fill_style("rgba(30,20,15,0.55)");
                                for s in 0..6 {
                                    let sx = 6.0 + s as f64 * 15.0;
                                    ctx.fill_rect(sx, 2, 5, 4);
                                    ctx.fill_rect(sx, h - 6.0, 5, 4);
                                }
                            }
                            TvStatic => {
                                for _ in 0..800 {
                                    let sx = rnd() * w;
                                    let sy = rnd() * h;
                                    let gray = 40.0 + rnd() * 200.0;
                                    ctx.set_fill_style(&format!("rgba({},{},{},{})", gray, gray, gray, 0.4 + rnd() * 0.5));
                                    ctx.fill_rect(sx, sy, 1, 1);
                                }
                                ctx.set_fill_style("rgba(0,0,0,0.35)");
                                ctx.fill_rect(0, 40.0 + rnd() * 20.0, w, 4);
                            }
                            ClockFace => {
                                let (cx, cy) = (w / 2.0, h / 2.0);
                                ctx.set_fill_style("#d0c8b8");
                                ctx.begin_path();
                                ctx.arc(cx, cy, 34, 0, PI * 2.0);
                                ctx.fill();
                                ctx.set_stroke_style("#1a1408");
                                ctx.set_line_width(2);
                                for i in 0..12 {
                                    let a = (i as f64 / 12.0) * PI * 2.0 - PI / 2.0;
                                    ctx.begin_path();
                                    ctx.move_to(cx + a.cos() * 28.0, cy + a.sin() * 28.0);
                                    ctx.line_to(cx + a.cos() * 33.0, cy + a.sin() * 33.0);
                                    ctx.stroke();
                                }
                                ctx.set_stroke_style("#0a0805");
                                ctx.set_line_width(3);
                                let hour_ang = ((3.0 / 12.0) + 3.0 / 720.0) * PI * 2.0 - PI / 2.0;
                                ctx.begin_path();
                                ctx.move_to(cx, cy);
                                ctx.line_to(cx + hour_ang.cos() * 16.0, cy + hour_ang.sin() * 16.0);
                                ctx.stroke();
                                ctx.set_line_width(2);
                                let min_ang = (3.0 / 60.0) * PI * 2.0 - PI / 2.0;
                                ctx.begin_path();
                                ctx.move_to(cx, cy);
                                ctx.line_to(cx + min_ang.cos() * 24.0, cy + min_ang.sin() * 24.0);
                                ctx.stroke();
                                ctx.set_fill_style("#0a0805");
                                ctx.begin_path();
                                ctx.arc(cx, cy, 2.5, 0, PI * 2.0);
                                ctx.fill();
                            }
                            FoundFootage => {
                                for _ in 0..400 {
                                    let gx = rnd() * w;
                                    let gy = rnd() * h;
                                    let gv = 40.0 + rnd() * 120.0;
                                    ctx.set_fill_style(&format!("rgba({},{},{},{})", gv, gv, gv * 0.9, 0.10 + rnd() * 0.30));
                                    ctx.fill_rect(gx, gy, 1, 1);
                                }
                                let band_y = 20.0 + rnd() * 50.0;
                                ctx.set_fill_style("rgba(200,200,180,0.15)");
                                ctx.fill_rect(0, band_y, w, 3);
                                ctx.set_fill_style("rgba(15,10,10,0.55)");
                                ctx.begin_path();
                                ctx.ellipse(w / 2.0 + (rnd() - 0.5) * 12.0, h - 26.0, 14, 22, 0, 0, PI * 2.0);
                                ctx.fill();
                                ctx.set_fill_style("rgba(15,10,10,0.45)");
                                ctx.begin_path();
                                ctx.arc(w / 2.0 + (rnd() - 0.5) * 12.0, h - 50.0, 6, 0, PI * 2.0);
                                ctx.fill();
                                ctx.set_fill_style("rgba(180,180,160,0.05)");
                                let mut y = 0.0;
                                while y < h {
                                    ctx.fill_rect(0, y, w, 1);
                                    y += 3.0;
                                }
                            }
                            MirrorRefl => {
                                let (cx, cy) = (w / 2.0, h / 2.0);
                                let frames = [
                                    (12.0, 12.0, w - 24.0, h - 24.0, "#5a5044"),
                                    (22.0, 22.0, w - 44.0, h - 44.0, "#3e3a30"),
                                    (32.0, 32.0, w - 64.0, h - 64.0, "#22201c"),
                                ];
                                for &(fx, fy, fw, fh, col) in frames.iter() {
                                    ctx.set_stroke_style(col);
                                    ctx.set_line_width(2);
                                    ctx.stroke_rect(fx, fy, fw, fh);
                                }
                                ctx.set_fill_style("#0a0805");
                                ctx.begin_path();
                                ctx.arc(cx, cy - 4.0, 4, 0, PI * 2.0);
                                ctx.fill();
                                ctx.begin_path();
                                ctx.ellipse(cx, cy + 8.0, 6, 10, 0, 0, PI * 2.0);
                                ctx.fill();
                            }
                            Hallway => {
                                let (cx, cy) = (w / 2.0, h / 2.0);
                                ctx.set_stroke_style("#1a1408");
                                ctx.set_line_width(2);
                                for (ex, ey) in [(0.0, 0.0), (w, 0.0), (0.0, h), (w, h)] {
                                    ctx.begin_path();
                                    ctx.move_to(ex, ey);
                                    ctx.line_to(cx, cy);
                                    ctx.stroke();
                                }
                                let glow_pool =
                                    ["rgba(220,180,120,0.55)", "rgba(140,180,220,0.45)", "rgba(220,80,80,0.55)", "rgba(230,220,140,0.60)"];
                                let glow_inner = glow_pool[(rnd() * glow_pool.len() as f64).floor() as usize];
                                // the same colour with its alpha set to 0
                                let glow_outer = match glow_inner.rfind(',') {
                                    Some(i) => format!("{},0)", &glow_inner[..i]),
                                    None => glow_inner.to_string(),
                                };
                                let grd = ctx.create_radial_gradient(cx, cy, 4, cx, cy, 28);
                                grd.add_color_stop(0, glow_inner);
                                grd.add_color_stop(1, &glow_outer);
                                ctx.set_fill_style(&grd);
                                ctx.fill_rect(0, 0, w, h);
                                ctx.set_fill_style("#0a0805");
                                ctx.begin_path();
                                ctx.arc(cx, cy, 3, 0, PI * 2.0);
                                ctx.fill();
                            }
                            RainWindow => {
                                ctx.set_fill_style("#1a2028");
                                ctx.fill_rect(w / 2.0 - 2.0, 8, 4, h - 16.0);
                                ctx.fill_rect(8, h / 2.0 - 2.0, w - 16.0, 4);
                                ctx.set_stroke_style("#1a2028");
                                ctx.set_line_width(3);
                                ctx.stroke_rect(6, 6, w - 12.0, h - 12.0);
                                ctx.set_stroke_style("rgba(200,220,240,0.35)");
                                ctx.set_line_width(0.8);
                                for _ in 0..30 {
                                    let rx = rnd() * w;
                                    let ry0 = rnd() * (h - 20.0);
                                    let ry_len = 6.0 + rnd() * 18.0;
                                    ctx.begin_path();
                                    ctx.move_to(rx, ry0);
                                    ctx.line_to(rx - 1.0, ry0 + ry_len);
                                    ctx.stroke();
                                }
                                ctx.set_fill_style("rgba(230,240,250,0.5)");
                                for _ in 0..15 {
                                    ctx.fill_rect(rnd() * w, rnd() * h, 2, 2);
                                }
                            }
                            StarCluster => {
                                ctx.set_fill_style("#e8d8f0");
                                let (cx, cy) = (w / 2.0, h / 2.0);
                                for _ in 0..80 {
                                    let rx = (rnd() + rnd() + rnd()) / 3.0 - 0.5;
                                    let ry = (rnd() + rnd() + rnd()) / 3.0 - 0.5;
                                    let sx = cx + rx * w * 0.9;
                                    let sy = cy + ry * h * 0.9;
                                    let s_size = if rnd() < 0.20 { 2.0 } else { 1.0 };
                                    ctx.set_global_alpha(0.4 + rnd() * 0.55);
                                    ctx.fill_rect(sx, sy, s_size, s_size);
                                }
                                ctx.set_global_alpha(1);
                                ctx.set_fill_style("#fff0f8");
                                ctx.fill_rect(cx - 2.0, cy - 2.0, 3, 3);
                                ctx.fill_rect(cx + 4.0, cy - 5.0, 2, 2);
                                ctx.fill_rect(cx - 5.0, cy + 3.0, 2, 2);
                            }
                            Eyes => {
                                let eye_y = 46.0;
                                ctx.set_fill_style("#1a1008");
                                for eye_x in [34.0, 62.0] {
                                    ctx.begin_path();
                                    ctx.ellipse(eye_x, eye_y, 12, 8, 0, 0, PI * 2.0);
                                    ctx.fill();
                                }
                                ctx.set_fill_style("#050302");
                                for eye_x in [34.0, 62.0] {
                                    ctx.begin_path();
                                    ctx.arc(eye_x, eye_y, 4, 0, PI * 2.0);
                                    ctx.fill();
                                }
                                ctx.set_fill_style("rgba(240,230,210,0.85)");
                                for eye_x in [34.0, 62.0] {
                                    ctx.fill_rect(eye_x + 1.0, eye_y - 2.0, 2, 2);
                                }
                            }
                            Shapes(shapes) => {
                                ctx.set_fill_style("#3a2a20");
                                for s in shapes.iter() {
                                    ctx.fill_rect(s[0], s[1], s[2], s[3]);
                                }
                            }
                        }
                        // Corner fade.
                        let grd = ctx.create_radial_gradient(w / 2.0, h / 2.0, w * 0.3, w / 2.0, h / 2.0, w * 0.7);
                        grd.add_color_stop(0, "rgba(0,0,0,0)");
                        grd.add_color_stop(1, "rgba(0,0,0,0.4)");
                        ctx.set_fill_style(&grd);
                        ctx.fill_rect(0, 0, w, h);
                    }));
                }
                textures
            })
        })
        .clone()
    })
}

// Long-finger-streak canvas — extreme drag variant of handprint.
game::lazy_static_tex!(
    handdrag_texture,
    canvas_tex_opt(192, 96, false, |ctx, w, h| {
        ctx.clear_rect(0, 0, w, h);
        let streak_count = 4.0 + (rnd() * 2.0).floor();
        let start_x = 20.0;
        let gap_y = (h - 30.0) / (streak_count + 1.0);
        let mut s = 0.0;
        while s < streak_count {
            let y = 15.0 + gap_y * (s + 1.0);
            ctx.set_stroke_style("rgba(50, 20, 15, 0.70)");
            ctx.set_line_width(8.0 - (s - streak_count / 2.0).abs());
            ctx.set_line_cap("round");
            ctx.begin_path();
            ctx.move_to(start_x, y + (rnd() - 0.5) * 3.0);
            for step in 1..=6 {
                let px = start_x + step as f64 * 25.0;
                let py = y + (rnd() - 0.5) * 5.0 + step as f64 * 1.0;
                ctx.line_to(px, py);
            }
            ctx.stroke();
            s += 1.0;
        }
    })
);

/// Handprint colour per weather (else the normal entry)
pub fn handprint_color_by_weather(key: &str) -> u32 {
    by_weather(key, 0x684030, 0x604828, 0x503048, 0x5a3820, 0x807870, 0x684838)
}

// Handprint canvas texture — stylized 5-finger palm print.
game::lazy_static_tex!(
    handprint_texture,
    canvas_tex_opt(64, 96, false, |ctx, w, h| {
        ctx.clear_rect(0, 0, w, h);
        ctx.set_fill_style("rgba(50, 20, 15, 0.75)");
        ctx.begin_path();
        ctx.ellipse(w / 2.0, h * 0.60, 20, 24, 0, 0, PI * 2.0);
        ctx.fill();
        let finger_angles = [-0.6, -0.3, 0.0, 0.3, 0.6];
        for fa in finger_angles {
            let fx = w / 2.0 + (fa as f64).sin() * 15.0;
            let fy = h * 0.35 + (fa as f64).cos() * 5.0;
            ctx.begin_path();
            ctx.ellipse(fx, fy, 5, 12, -fa, 0, PI * 2.0);
            ctx.fill();
        }
        ctx.set_fill_style("rgba(50, 20, 15, 0.35)");
        ctx.fill_rect(w / 2.0 - 1.0, h * 0.75, 2, 12);
    })
);

// Mirror smudge canvas texture.
game::lazy_static_tex!(
    mirror_smudge_texture,
    canvas_tex_opt(128, 192, false, |ctx, w, h| {
        ctx.clear_rect(0, 0, w, h);
        ctx.set_stroke_style("rgba(200, 200, 210, 0.18)");
        ctx.set_line_width(4);
        let streak_count = 2 + (rnd() * 2.0).floor() as i32;
        for _s in 0..streak_count {
            let x0 = 20.0 + rnd() * (w - 40.0);
            let y0 = 40.0 + rnd() * (h - 80.0);
            let ang = (rnd() - 0.5) * 0.6;
            let len = 20.0 + rnd() * 30.0;
            ctx.begin_path();
            ctx.move_to(x0, y0);
            ctx.line_to(x0 + ang.cos() * len, y0 + ang.sin() * len);
            ctx.stroke();
        }
        let bx = 40.0 + rnd() * (w - 80.0);
        let by = 70.0 + rnd() * (h - 140.0);
        let grad = ctx.create_radial_gradient(bx, by, 5, bx, by, 30);
        grad.add_color_stop(0, "rgba(220, 220, 220, 0.25)");
        grad.add_color_stop(1, "rgba(220, 220, 220, 0)");
        ctx.set_fill_style(&grad);
        ctx.fill_rect(bx - 30.0, by - 30.0, 60, 60);
    })
);
