//! Constants that several placers (or the core) share: per-weather palettes
//! and textures built once on first use.

use crate::base::rng::rnd;
use crate::placers::kit::canvas_tex;
use crate::placers::kit::canvas_tex_opt;
use engine::prelude::*;
use std::f64::consts::PI;

/// A silvered mirror's highlight (`Patch::Glass`, Blinn-Phong): the lamp seen
/// in it as a lamp is in a mirror, a small hot spot about the size of its
/// lens, not a disc of glare round it (at 4000 it read as bloom:
/// `docs/design/PROPS.md`, M8)
pub const MIRROR_SHINE: f64 = 120000.0;

/// Picks the value for weather `key` from one value per weather; any other
/// key gets `normal`.
pub fn by_weather<T: Copy>(key: &str, wet: T, electric: T, haunted: T, derelict: T, still: T, normal: T) -> T {
    match key {
        "wet" => wet,
        "electric" => electric,
        "haunted" => haunted,
        "derelict" => derelict,
        "still" => still,
        _ => normal,
    }
}

#[derive(Clone, Copy, Debug)]
pub struct StainSpec {
    pub color: u32,
    pub opacity_base: f64,
    pub density_mul: f64,
}
/// Floor stain colour, opacity and density by weather.
pub fn floor_stain_by_weather(key: &str) -> StainSpec {
    by_weather(
        key,
        StainSpec { color: 0x0a1a20, opacity_base: 0.42, density_mul: 1.4 },
        StainSpec { color: 0x1a1006, opacity_base: 0.55, density_mul: 0.8 },
        StainSpec { color: 0x0e1a0c, opacity_base: 0.35, density_mul: 1.0 },
        StainSpec { color: 0x2a1608, opacity_base: 0.45, density_mul: 1.2 },
        StainSpec { color: 0x1a1614, opacity_base: 0.22, density_mul: 0.6 },
        StainSpec { color: 0x141210, opacity_base: 0.30, density_mul: 1.0 },
    )
}
/// Chalk mark colour by weather.
pub fn chalk_color_by_weather(key: &str) -> u32 {
    by_weather(key, 0x9ea4a2, 0xe8dc70, 0xa04040, 0xc0b090, 0xf0eee8, 0xe4e0d8)
}
/// Glass shard colour by weather.
pub fn shard_color_by_weather(key: &str) -> u32 {
    by_weather(key, 0xb0c0c8, 0xd8d0a8, 0xc0a0a0, 0xa89890, 0xd0d8dc, 0xd0d8dc)
}
/// Newspaper tint by weather.
pub fn newspaper_tint_by_weather(key: &str) -> u32 {
    by_weather(key, 0x9ea0a0, 0xd8b070, 0xa88080, 0xa89880, 0xc4b898, 0xffffff)
}
/// Cobweb colour by weather.
pub fn cobweb_color_by_weather(key: &str) -> u32 {
    by_weather(key, 0xa8b0b8, 0xc8c0a0, 0xa89890, 0x98908c, 0xd8d0c8, 0xd8d0c8)
}
/// Indicator LED colour by weather.
pub fn led_color_by_weather(key: &str) -> u32 {
    by_weather(key, 0xff4020, 0xffe040, 0xc040ff, 0xff6020, 0xf0f0f0, 0xff2020)
}
/// Wall vent colour by weather.
pub fn vent_color_by_weather(key: &str) -> u32 {
    by_weather(key, 0x554a3a, 0x606058, 0x504840, 0x5a3820, 0x808070, 0x6a6058)
}
/// Splash sprite colour by weather.
pub fn splash_sprite_by_weather(key: &str) -> u32 {
    by_weather(key, 0x80c0f0, 0xf0e090, 0xe06050, 0x88908a, 0x60a0f0, 0x60a0f0)
}

crate::lazy_static_tex!(
    watch_face_texture,
    canvas_tex_opt(128, 128, false, |ctx, w, h| {
        ctx.set_fill_style("#f0ece0");
        ctx.begin_path();
        ctx.arc(w / 2.0, h / 2.0, w / 2.0 - 4.0, 0, PI * 2.0);
        ctx.fill();
        ctx.set_stroke_style("#181008");
        ctx.set_line_width(2);
        ctx.stroke();
        ctx.set_fill_style("#181008");
        ctx.set_font("bold 12px serif");
        ctx.set_text_align("center");
        ctx.set_text_baseline("middle");
        let nums = ["XII", "I", "II", "III", "IV", "V", "VI", "VII", "VIII", "IX", "X", "XI"];
        for i in 0..12 {
            let a = -PI / 2.0 + (i as f64 / 12.0) * PI * 2.0;
            ctx.fill_text(nums[i], w / 2.0 + a.dcos() * 46.0, h / 2.0 + a.dsin() * 46.0);
        }
        ctx.set_line_width(4);
        ctx.begin_path();
        ctx.move_to(w / 2.0, h / 2.0);
        ctx.line_to(w / 2.0 + (-PI / 2.0 + PI * 0.6).dcos() * 26.0, h / 2.0 + (-PI / 2.0 + PI * 0.6).dsin() * 26.0);
        ctx.stroke();
        ctx.set_line_width(3);
        ctx.begin_path();
        ctx.move_to(w / 2.0, h / 2.0);
        ctx.line_to(w / 2.0 + (-PI / 2.0 + PI * 1.35).dcos() * 40.0, h / 2.0 + (-PI / 2.0 + PI * 1.35).dsin() * 40.0);
        ctx.stroke();
        ctx.begin_path();
        ctx.arc(w / 2.0, h / 2.0, 5, 0, PI * 2.0);
        ctx.fill();
    })
);

fn pick<'a>(a: &[&'a str]) -> &'a str {
    a[(rnd() * a.len() as f64).floor() as usize]
}

crate::lazy_static_tex!(
    newspaper_texture,
    canvas_tex(220, 280, |ctx, w, h| {
        ctx.set_fill_style("#e8dcb8");
        ctx.fill_rect(0, 0, w, h);
        ctx.set_fill_style("rgba(140,100,60,0.10)");
        for _ in 0..40 {
            ctx.begin_path();
            let (x, y, r) = (rnd() * w, rnd() * h, 8.0 + rnd() * 15.0);
            ctx.arc(x, y, r, 0, PI * 2.0);
            ctx.fill();
        }
        ctx.set_fill_style("#101010");
        ctx.set_text_align("center");
        ctx.set_font("bold 22px serif");
        ctx.fill_text(pick(&["THE DAILY HERALD", "MORNING GAZETTE", "CITY CHRONICLE", "EVENING SENTINEL"]), w / 2.0, 22);
        ctx.set_stroke_style("#101010");
        ctx.set_line_width(1);
        ctx.begin_path();
        ctx.move_to(10, 30);
        ctx.line_to(w - 10.0, 30);
        ctx.move_to(10, 34);
        ctx.line_to(w - 10.0, 34);
        ctx.stroke();
        ctx.set_font("9px serif");
        ctx.set_text_align("left");
        ctx.fill_text(pick(&["Tuesday, October 14", "Sunday, March 22", "Friday, July 4", "Saturday, Dec. 18"]), 10, 44);
        ctx.set_text_align("right");
        let year = pick(&["1987", "1974", "1962", "1993", "1958"]);
        let price = pick(&["25¢", "10¢", "15¢", "50¢"]);
        ctx.fill_text(&format!("{year}   {price}"), w - 10.0, 44);
        ctx.set_text_align("center");
        ctx.set_font("bold 18px serif");
        let headline = pick(&[
            "MYSTERY CONTINUES",
            "NO ANSWERS FOUND",
            "TOWN LEFT SILENT",
            "FAMILIES GONE MISSING",
            "STRANGE EMPTINESS",
            "ALL RESIDENTS VANISH",
        ]);
        ctx.fill_text(headline, w / 2.0, 68);
        ctx.set_font("italic 11px serif");
        ctx.fill_text(
            pick(&[
                "Investigators baffled by mass disappearance",
                "Empty homes, warm meals, no explanations",
                "Officials speak of unexplained events",
                "Search parties return with no findings",
            ]),
            w / 2.0,
            88,
        );
        ctx.set_stroke_style("#303030");
        ctx.set_line_width(0.7);
        let col_w = 90.0;
        for cx in [15.0, 118.0] {
            for row in 0..24 {
                let y = 105.0 + row as f64 * 6.5;
                if y > h - 25.0 {
                    break;
                }
                let line_w = col_w - rnd() * 20.0;
                ctx.begin_path();
                ctx.move_to(cx, y);
                ctx.line_to(cx + line_w, y);
                ctx.stroke();
            }
        }
        ctx.set_fill_style("#8a7050");
        ctx.fill_rect(w / 2.0 - 25.0, h * 0.55, 50, 36);
        let fig = ctx.create_radial_gradient(w / 2.0, h * 0.55 + 18.0, 3, w / 2.0, h * 0.55 + 18.0, 22);
        fig.add_color_stop(0, "rgba(40,25,15,0.85)");
        fig.add_color_stop(1, "rgba(60,45,30,0)");
        ctx.set_fill_style(&fig);
        ctx.begin_path();
        ctx.ellipse(w / 2.0, h * 0.55 + 22.0, 15, 16, 0, 0, PI * 2.0);
        ctx.fill();
        ctx.set_fill_style("#101010");
        ctx.set_font("italic 8px serif");
        ctx.set_text_align("center");
        ctx.fill_text("scene from empty street", w / 2.0, h * 0.55 + 46.0);
        ctx.set_stroke_style("rgba(80,60,40,0.35)");
        ctx.set_line_width(1.5);
        ctx.begin_path();
        ctx.move_to(0, h * 0.5);
        ctx.line_to(w, h * 0.5 + 8.0);
        ctx.stroke();
    })
);

/// The window sign textures (8 variants).
pub fn window_sign_textures() -> Vec<Tex> {
    thread_local! { static V: std::cell::OnceCell<Vec<Tex>> = const { std::cell::OnceCell::new() }; }
    V.with(|c| {
        c.get_or_init(|| {
            crate::base::rng::with_native(|| {
                let variants: [(&str, &str, &str, f64); 8] = [
                    ("OPEN", "#0a2a0a", "#a0f0a0", 26.0),
                    ("CLOSED", "#2a0a0a", "#f0a0a0", 26.0),
                    ("OUT OF\nORDER", "#2a1808", "#f0b060", 18.0),
                    ("RESTROOM", "#08202a", "#a0d8f0", 22.0),
                    ("EXIT", "#082a10", "#40f070", 30.0),
                    ("DO NOT\nENTER", "#2a0808", "#f04040", 18.0),
                    ("STAFF\nONLY", "#2a2408", "#f0e060", 20.0),
                    ("KEEP\nOUT", "#2a1408", "#f08040", 22.0),
                ];
                variants
                    .iter()
                    .map(|&(text, bg, fg, fs)| {
                        canvas_tex_opt(128, 64, false, |ctx, w, h| {
                            ctx.set_fill_style(bg);
                            ctx.fill_rect(0, 0, w, h);
                            ctx.set_stroke_style(fg);
                            ctx.set_line_width(2);
                            ctx.stroke_rect(3, 3, w - 6.0, h - 6.0);
                            ctx.set_fill_style(fg);
                            ctx.set_font(&format!("bold {fs}px monospace"));
                            ctx.set_text_align("center");
                            ctx.set_text_baseline("middle");
                            let lines: Vec<&str> = text.split('\n').collect();
                            let line_h = fs + 2.0;
                            let start_y = h / 2.0 - ((lines.len() - 1) as f64 * line_h) / 2.0;
                            for (i, ln) in lines.iter().enumerate() {
                                ctx.fill_text(ln, w / 2.0, start_y + i as f64 * line_h);
                            }
                            for _ in 0..20 {
                                ctx.set_fill_style(format!("rgba(0,0,0,{})", 0.08 + rnd() * 0.1));
                                let (x, y) = (rnd() * w, rnd() * h);
                                ctx.fill_rect(x, y, 1, 1);
                            }
                        })
                    })
                    .collect()
            })
        })
        .clone()
    })
}

/// The wall clock face textures (6 variants).
pub fn clock_textures() -> Vec<Tex> {
    thread_local! { static V: std::cell::OnceCell<Vec<Tex>> = const { std::cell::OnceCell::new() }; }
    V.with(|c| {
        c.get_or_init(|| {
            crate::base::rng::with_native(|| {
                // (hour, minute, face colour, tint r, g, b, cracked)
                let variants: [(f64, f64, &str, u32, u32, u32, bool); 6] = [
                    (3.0, 33.0, "#e0dcd4", 30, 25, 20, false),
                    (11.0, 11.0, "#d8d4c8", 20, 20, 30, false),
                    (13.0, 0.0, "#dcd0b8", 40, 30, 10, false),
                    (4.0, 4.0, "#e4d8cc", 40, 15, 10, false),
                    (12.0, 34.0, "#d0d0d0", 20, 20, 20, false),
                    (6.0, 6.0, "#d8d0c0", 15, 15, 15, true),
                ];
                variants
                    .iter()
                    .map(|&(vh, vm, face, tr, tg, tb, cracked)| {
                        canvas_tex_opt(128, 128, false, |ctx, w, h| {
                            ctx.set_fill_style(face);
                            ctx.fill_rect(0, 0, w, h);
                            let grd = ctx.create_radial_gradient(w / 2.0, h / 2.0, w * 0.3, w / 2.0, h / 2.0, w * 0.55);
                            grd.add_color_stop(0, "rgba(0,0,0,0)");
                            grd.add_color_stop(1, &format!("rgba({tr},{tg},{tb},0.35)"));
                            ctx.set_fill_style(&grd);
                            ctx.fill_rect(0, 0, w, h);
                            ctx.set_stroke_style("#302820");
                            for i in 0..60 {
                                let a = (i as f64 / 60.0) * PI * 2.0 - PI / 2.0;
                                let major = i % 5 == 0;
                                let r_in = if major { 46.0 } else { 50.0 };
                                ctx.set_line_width(if major { 2.5 } else { 1.0 });
                                ctx.begin_path();
                                ctx.move_to(w / 2.0 + a.dcos() * r_in, h / 2.0 + a.dsin() * r_in);
                                ctx.line_to(w / 2.0 + a.dcos() * 54.0, h / 2.0 + a.dsin() * 54.0);
                                ctx.stroke();
                            }
                            ctx.set_fill_style("#302820");
                            ctx.set_font("bold 12px sans-serif");
                            ctx.set_text_align("center");
                            ctx.set_text_baseline("middle");
                            ctx.fill_text("12", w / 2.0, h / 2.0 - 42.0);
                            ctx.fill_text("3", w / 2.0 + 42.0, h / 2.0);
                            ctx.fill_text("6", w / 2.0, h / 2.0 + 42.0);
                            ctx.fill_text("9", w / 2.0 - 42.0, h / 2.0);
                            let hour_ang = ((vh % 12.0) / 12.0 + vm / 720.0) * PI * 2.0 - PI / 2.0;
                            let min_ang = (vm / 60.0) * PI * 2.0 - PI / 2.0;
                            ctx.set_stroke_style("#1a1410");
                            ctx.set_line_width(4);
                            ctx.begin_path();
                            ctx.move_to(w / 2.0, h / 2.0);
                            ctx.line_to(w / 2.0 + hour_ang.dcos() * 28.0, h / 2.0 + hour_ang.dsin() * 28.0);
                            ctx.stroke();
                            ctx.set_line_width(3);
                            ctx.begin_path();
                            ctx.move_to(w / 2.0, h / 2.0);
                            ctx.line_to(w / 2.0 + min_ang.dcos() * 40.0, h / 2.0 + min_ang.dsin() * 40.0);
                            ctx.stroke();
                            ctx.set_fill_style("#1a1410");
                            ctx.begin_path();
                            ctx.arc(w / 2.0, h / 2.0, 3, 0, PI * 2.0);
                            ctx.fill();
                            if cracked {
                                let ix = w / 2.0 + (rnd() - 0.5) * 40.0;
                                let iy = h / 2.0 + (rnd() - 0.5) * 40.0;
                                ctx.set_stroke_style("rgba(20,15,10,0.85)");
                                let n = 3 + (rnd() * 3.0).floor() as i32;
                                for _ in 0..n {
                                    let ang = rnd() * PI * 2.0;
                                    let len = 28.0 + rnd() * 24.0;
                                    let mid = (rnd() - 0.5) * 8.0;
                                    ctx.set_line_width(0.8 + rnd() * 1.2);
                                    ctx.begin_path();
                                    ctx.move_to(ix, iy);
                                    ctx.line_to(ix + ang.dcos() * len * 0.5 + mid, iy + ang.dsin() * len * 0.5 + mid);
                                    ctx.line_to(ix + ang.dcos() * len, iy + ang.dsin() * len);
                                    ctx.stroke();
                                }
                                ctx.set_fill_style("rgba(20,15,10,0.6)");
                                ctx.begin_path();
                                let r = 2.0 + rnd() * 1.5;
                                ctx.arc(ix, iy, r, 0, PI * 2.0);
                                ctx.fill();
                            }
                        })
                    })
                    .collect()
            })
        })
        .clone()
    })
}

crate::lazy_static_tex!(
    cracked_glass_texture,
    canvas_tex_opt(128, 192, false, |ctx, w, h| {
        ctx.clear_rect(0, 0, w, h);
        let ix = w / 2.0 + (rnd() - 0.5) * 60.0;
        let iy = h / 2.0 + (rnd() - 0.5) * 80.0;
        ctx.set_stroke_style("rgba(15,10,8,0.85)");
        let n = 5 + (rnd() * 4.0).floor() as i32;
        for _ in 0..n {
            let ang = rnd() * PI * 2.0;
            let len = 40.0 + rnd() * 60.0;
            let mid = (rnd() - 0.5) * 12.0;
            ctx.set_line_width(0.8 + rnd() * 1.4);
            ctx.begin_path();
            ctx.move_to(ix, iy);
            ctx.line_to(ix + ang.dcos() * len * 0.55 + mid, iy + ang.dsin() * len * 0.55 + mid);
            ctx.line_to(ix + ang.dcos() * len, iy + ang.dsin() * len);
            ctx.stroke();
        }
        ctx.set_fill_style("rgba(15,10,8,0.65)");
        ctx.begin_path();
        let r = 3.0 + rnd() * 2.0;
        ctx.arc(ix, iy, r, 0, PI * 2.0);
        ctx.fill();
    })
);
