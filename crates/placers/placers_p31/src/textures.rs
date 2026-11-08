//! Canvas textures shared by this crate's placers (built once, on the native RNG).
use game::prelude::*;

// Exit sign canvas texture — bright green EXIT text on dark background.
game::lazy_static_tex!(
    exit_sign_texture_2,
    canvas_tex_opt(96, 40, false, |ctx, w, h| {
        ctx.set_fill_style("#1a1a12");
        ctx.fill_rect(0, 0, w, h);
        ctx.set_fill_style("#40c860");
        ctx.set_font("bold 28px sans-serif");
        ctx.set_text_align("center");
        ctx.set_text_baseline("middle");
        ctx.fill_text("EXIT", w / 2.0, h / 2.0 + 2.0);
        // Slight scan-line grain.
        let mut y = 0.0;
        while y < h {
            ctx.set_fill_style("rgba(0,0,0,0.06)");
            ctx.fill_rect(0, y, w, 1);
            y += 2.0;
        }
    })
);

// Thermostat canvas texture — analog dial with tick marks + needle
// frozen at random temp. One-per-load random reading.
game::lazy_static_tex!(
    thermostat_texture,
    canvas_tex_opt(64, 64, false, |ctx, w, h| {
        let (cx, cy, r) = (w / 2.0, h / 2.0, 24.0);
        // Cream background.
        ctx.set_fill_style("#d8ccb0");
        ctx.fill_rect(0, 0, w, h);
        // Dial ring.
        ctx.set_stroke_style("#2a1810");
        ctx.set_line_width(2);
        ctx.begin_path();
        ctx.arc(cx, cy, r, 0, PI * 2.0);
        ctx.stroke();
        // Tick marks (270° span from 135° to 45° via bottom).
        for i in 0..=10 {
            let a = PI * 0.75 + (PI * 1.5) * (i as f64 / 10.0);
            ctx.set_line_width(if i % 5 == 0 { 2 } else { 1 });
            ctx.begin_path();
            ctx.move_to(cx + a.cos() * (r - 2.0), cy + a.sin() * (r - 2.0));
            ctx.line_to(cx + a.cos() * (r - 6.0), cy + a.sin() * (r - 6.0));
            ctx.stroke();
        }
        // Needle at random reading in valid arc.
        let needle_ang = PI * 0.75 + (PI * 1.5) * (0.2 + rnd() * 0.6);
        ctx.set_stroke_style("#a02010");
        ctx.set_line_width(2);
        ctx.begin_path();
        ctx.move_to(cx, cy);
        ctx.line_to(cx + needle_ang.cos() * (r - 4.0), cy + needle_ang.sin() * (r - 4.0));
        ctx.stroke();
        // Center pin.
        ctx.set_fill_style("#2a1810");
        ctx.begin_path();
        ctx.arc(cx, cy, 2, 0, PI * 2.0);
        ctx.fill();
    })
);

/// Poster textures (3 variants) — distressed poster rectangles generated once,
/// on first use.
pub fn poster_textures() -> Vec<Tex> {
    thread_local! { static V: std::cell::OnceCell<Vec<Tex>> = const { std::cell::OnceCell::new() }; }
    V.with(|c| {
        c.get_or_init(|| {
            game::base::rng::with_native(|| {
                // (bg, ink, banner)
                let variants: [(&str, &str, &str); 3] = [
                    ("#c8b880", "#403020", "#8a5030"), // yellowed with red-brown banner
                    ("#a8a09a", "#302820", "#4a4028"), // gray with olive banner
                    ("#b8a488", "#403428", "#603820"), // cream with rust banner
                ];
                variants
                    .iter()
                    .map(|&(bg, ink, banner)| {
                        canvas_tex_opt(128, 192, false, |ctx, w, h| {
                            // Background paper.
                            ctx.set_fill_style(bg);
                            ctx.fill_rect(0, 0, w, h);
                            // Grunge speckles.
                            for _ in 0..60 {
                                ctx.set_fill_style(&format!("rgba(0,0,0,{})", 0.03 + rnd() * 0.08));
                                let (px, py) = (rnd() * w, rnd() * h);
                                let sz = 1.0 + rnd() * 3.0;
                                ctx.fill_rect(px, py, sz, sz);
                            }
                            // Top banner.
                            ctx.set_fill_style(banner);
                            ctx.fill_rect(6, 12, w - 12.0, 28);
                            // Faux-title text as pixel blocks.
                            ctx.set_fill_style(bg);
                            let mut x = 12.0;
                            while x < w - 12.0 {
                                if rnd() < 0.65 {
                                    ctx.fill_rect(x, 18, 4, 16);
                                }
                                x += 6.0;
                            }
                            // Body content: 6 lines of text streaks.
                            ctx.set_fill_style(ink);
                            for line in 0..6 {
                                let y = 60.0 + line as f64 * 18.0;
                                let mut x = 12.0;
                                while x < w - 20.0 {
                                    if rnd() < 0.55 {
                                        ctx.fill_rect(x, y, 2, 3);
                                    }
                                    x += 3.0;
                                }
                            }
                            // Bottom rectangle (photo/logo placeholder).
                            ctx.set_fill_style(ink);
                            ctx.fill_rect(24, 140, w - 48.0, 38);
                            ctx.set_fill_style(bg);
                            for _ in 0..25 {
                                let px = 26.0 + rnd() * (w - 52.0);
                                let py = 142.0 + rnd() * 34.0;
                                let sw = 1.0 + rnd() * 2.0;
                                let sh = 1.0 + rnd() * 2.0;
                                ctx.fill_rect(px, py, sw, sh);
                            }
                            // Barely-legible text hint over the banner.
                            let hints = ["EXIT", "13", "SAFE", "WAIT", "DO NOT", "???", "7", "HOLD", "BACK"];
                            let hint = hints[(rnd() * hints.len() as f64).floor() as usize];
                            ctx.set_fill_style(bg);
                            ctx.set_global_alpha(0.65);
                            ctx.set_font("bold 16px sans-serif");
                            ctx.set_text_align("center");
                            ctx.set_text_baseline("middle");
                            ctx.fill_text(hint, w / 2.0, 28);
                            ctx.set_global_alpha(1.0);
                            // Fade edges via corner alpha.
                            let grd = ctx.create_linear_gradient(0, 0, w, h);
                            grd.add_color_stop(0, "rgba(80,60,40,0.20)");
                            grd.add_color_stop(0.5, "rgba(0,0,0,0)");
                            grd.add_color_stop(1, "rgba(80,60,40,0.25)");
                            ctx.set_fill_style(&grd);
                            ctx.fill_rect(0, 0, w, h);
                        })
                    })
                    .collect()
            })
        })
        .clone()
    })
}
