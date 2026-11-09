//! The canvas-drawn textures: wallpaper, carpet, ceiling, light-panel, dust,
//! face and TV-static textures, the hand-drawn stroke alphabet and `text_tex`.
//!
//! `Textures::new()` builds every startup texture in a fixed order (wallpaper
//! variants, carpet variants, ceiling variants, light panel, dust sprite, face,
//! TV-static canvas/texture + `draw_static()`), all on the native RNG.
//! Everything other code calls at run time is a free function.
//!
//! Random draws happen in a fixed order and under fixed conditions (loop
//! conditions that call `irand` per iteration included), so every texture is
//! reproducible for a given RNG stream.

use crate::base::numeric::hypot2;
use crate::base::numeric::*;
use crate::base::rng::{choice, irand, rand1, rand2, rnd, with_native};
use crate::placers::kit::{canvas_tex, canvas_tex_opt};
use engine::prelude::*;
use std::f64::consts::PI;

/// Iteration count of `i = 0; while i < n { i += 1 }` for a fractional bound
/// (`ceil(n)`, 0 when `n` ≤ 0 or NaN).
#[inline]
fn times(n: f64) -> u64 {
    if n > 0.0 {
        n.ceil() as u64
    } else {
        0
    }
}

// ============================================================================
// helpers
// ============================================================================

/// `n` dark translucent ellipses, each drawn at
/// x and x ± w so the texture wraps horizontally.
pub fn grime(ctx: &mut Canvas, w: f64, h: f64, n: f64, alpha: f64) {
    for _ in 0..times(n) {
        let cx = rand1(w);
        let cy = rand1(h);
        let rx = rand2(6.0, 52.0);
        let ry = rand2(4.0, 30.0);
        let rot = rand1(PI);
        ctx.set_fill_style(format!("rgba(28,20,6,{})", rand2(0.03, alpha)));
        // draw at center + horizontal copies so features that cross the seam
        // complete on the other side, making the texture wrap-seamless
        for dx in [0.0, -w, w] {
            ctx.begin_path();
            ctx.ellipse(cx + dx, cy, rx, ry, rot, 0, 7);
            ctx.fill();
        }
    }
}

/// A radial gradient
/// filled over the whole canvas at cx and cx ± w (horizontal wrap).
pub fn radial_wrap(ctx: &mut Canvas, w: f64, h: f64, cx: f64, cy: f64, r0: f64, r1: f64, stop0: &str, stop1: &str) {
    for dx in [0.0, -w, w] {
        let grad = ctx.create_radial_gradient(cx + dx, cy, r0, cx + dx, cy, r1);
        grad.add_color_stop(0, stop0);
        grad.add_color_stop(1, stop1);
        ctx.set_fill_style(&grad);
        ctx.fill_rect(0, 0, w, h);
    }
}

// ============================================================================
// WALL_PATTERNS: each takes (ctx, w, h, motif, motif2)
// ============================================================================

/// A `WALL_PATTERNS` renderer: `(ctx, w, h, motif, motif2)`.
pub type WallPatternFn = fn(&mut Canvas, f64, f64, &str, &str);

/// `WALL_PATTERNS` entry `"plain"`
pub fn pattern_plain(_ctx: &mut Canvas, _w: f64, _h: f64, _m: &str, _m2: &str) {}
/// `WALL_PATTERNS` entry `"stripe"`
pub fn pattern_stripe(ctx: &mut Canvas, w: f64, h: f64, m: &str, _m2: &str) {
    let mut x = 0.0;
    while x < w {
        ctx.set_fill_style(m);
        ctx.fill_rect(x, 0, 12, h);
        x += 24.0;
    }
}
/// `WALL_PATTERNS` entry `"stripeDouble"`
pub fn pattern_stripe_double(ctx: &mut Canvas, w: f64, h: f64, m: &str, m2: &str) {
    let mut x = 0.0;
    while x < w {
        ctx.set_fill_style(m);
        ctx.fill_rect(x, 0, 20, h);
        ctx.set_fill_style(m2);
        ctx.fill_rect(x + 22.0, 0, 6, h);
        x += 48.0;
    }
}
/// `WALL_PATTERNS` entry `"stripeHoriz"`
pub fn pattern_stripe_horiz(ctx: &mut Canvas, w: f64, h: f64, m: &str, _m2: &str) {
    let mut y = 0.0;
    while y < h {
        ctx.set_fill_style(m);
        ctx.fill_rect(0, y, w, 14);
        y += 30.0;
    }
}
/// `WALL_PATTERNS` entry `"grid"`
pub fn pattern_grid(ctx: &mut Canvas, w: f64, h: f64, m: &str, _m2: &str) {
    ctx.set_fill_style(m);
    let mut x = 0.0;
    while x < w {
        ctx.fill_rect(x, 0, 1, h);
        x += 48.0;
    }
    let mut y = 0.0;
    while y < h {
        ctx.fill_rect(0, y, w, 1);
        y += 48.0;
    }
}
/// `WALL_PATTERNS` entry `"diamond"`
pub fn pattern_diamond(ctx: &mut Canvas, w: f64, h: f64, m: &str, m2: &str) {
    let mut x = 0.0;
    while x < w {
        ctx.set_fill_style(m2);
        ctx.fill_rect(x, 0, 22, h);
        ctx.set_fill_style("rgba(255,240,180,0.05)");
        ctx.fill_rect(x + 22.0, 0, 4, h);
        let mut y = 0.0;
        while y < h {
            ctx.set_fill_style(m);
            ctx.begin_path();
            ctx.move_to(x + 11.0, y);
            ctx.line_to(x + 22.0, y + 48.0);
            ctx.line_to(x + 11.0, y + 96.0);
            ctx.line_to(x, y + 48.0);
            ctx.close_path();
            ctx.fill();
            y += 96.0;
        }
        x += 40.0;
    }
}
/// `WALL_PATTERNS` entry `"flock"`
pub fn pattern_flock(ctx: &mut Canvas, w: f64, h: f64, m: &str, _m2: &str) {
    let mut y = 32.0;
    while y < h {
        let mut x = 32.0;
        while x < w {
            let ox = if (y / 96.0).floor() % 2.0 == 0.0 { 0.0 } else { 48.0 };
            ctx.set_fill_style(m);
            ctx.begin_path();
            ctx.ellipse(x + ox, y, 22, 32, 0, 0, 7);
            ctx.fill();
            ctx.begin_path();
            ctx.ellipse(x + ox, y - 30.0, 8, 14, 0, 0, 7);
            ctx.fill();
            ctx.begin_path();
            ctx.ellipse(x + ox, y + 30.0, 8, 14, 0, 0, 7);
            ctx.fill();
            // small side leaves
            ctx.begin_path();
            ctx.ellipse(x + ox - 22.0, y, 6, 10, 0.6, 0, 7);
            ctx.fill();
            ctx.begin_path();
            ctx.ellipse(x + ox + 22.0, y, 6, 10, -0.6, 0, 7);
            ctx.fill();
            x += 96.0;
        }
        y += 96.0;
    }
}
/// `WALL_PATTERNS` entry `"chevron"`
pub fn pattern_chevron(ctx: &mut Canvas, w: f64, h: f64, m: &str, _m2: &str) {
    ctx.set_stroke_style(m);
    ctx.set_line_width(3);
    let mut y = -h;
    while y < h * 2.0 {
        ctx.begin_path();
        let mut x = 0.0;
        while x <= w {
            let yy = y + if (x / 32.0) % 2.0 == 0.0 { 0.0 } else { 14.0 };
            if x == 0.0 {
                ctx.move_to(x, yy);
            } else {
                ctx.line_to(x, yy);
            }
            x += 32.0;
        }
        ctx.stroke();
        y += 28.0;
    }
}
/// `WALL_PATTERNS` entry `"herringbone"`
pub fn pattern_herringbone(ctx: &mut Canvas, w: f64, h: f64, m: &str, _m2: &str) {
    ctx.set_stroke_style(m);
    ctx.set_line_width(2);
    let mut y = -w;
    while y < h + w {
        ctx.begin_path();
        ctx.move_to(0, y);
        ctx.line_to(w, y + w);
        ctx.stroke();
        ctx.begin_path();
        ctx.move_to(0, y + w);
        ctx.line_to(w, y);
        ctx.stroke();
        y += 24.0;
    }
}
/// `WALL_PATTERNS` entry `"polka"`
pub fn pattern_polka(ctx: &mut Canvas, w: f64, h: f64, m: &str, _m2: &str) {
    ctx.set_fill_style(m);
    let mut y = 30.0;
    while y < h {
        let mut x = 30.0;
        while x < w {
            let ox = if (y / 44.0).floor() % 2.0 == 0.0 { 0.0 } else { 22.0 };
            ctx.begin_path();
            ctx.arc(x + ox, y, 6, 0, 7);
            ctx.fill();
            x += 44.0;
        }
        y += 44.0;
    }
}
/// `WALL_PATTERNS` entry `"hex"`
pub fn pattern_hex(ctx: &mut Canvas, w: f64, h: f64, m: &str, _m2: &str) {
    ctx.set_stroke_style(m);
    ctx.set_line_width(1.5);
    let r = 22.0;
    let mut y = 0.0;
    while y < h + r {
        let ox = if to_i32(y / (r * 1.5)) % 2 == 0 { 0.0 } else { r * 0.87 };
        let mut x = -r;
        while x < w + r {
            ctx.begin_path();
            for i in 0..=6 {
                let a = i as f64 * PI / 3.0;
                let px = x + ox + a.cos() * r;
                let py = y + a.sin() * r;
                if i == 0 {
                    ctx.move_to(px, py);
                } else {
                    ctx.line_to(px, py);
                }
            }
            ctx.stroke();
            x += r * 1.74;
        }
        y += r * 1.5;
    }
}
/// `WALL_PATTERNS` entry `"artDeco"`
pub fn pattern_art_deco(ctx: &mut Canvas, w: f64, h: f64, m: &str, _m2: &str) {
    ctx.set_stroke_style(m);
    ctx.set_line_width(2);
    let mut cy = 60.0;
    while cy < h {
        let mut cx = 60.0;
        while cx < w {
            let mut a = -PI / 2.0;
            while a <= PI / 2.0 {
                ctx.begin_path();
                ctx.move_to(cx, cy);
                ctx.line_to(cx + a.cos() * 55.0, cy + a.sin() * 55.0);
                ctx.stroke();
                a += PI / 12.0;
            }
            cx += 140.0;
        }
        cy += 140.0;
    }
}
/// `WALL_PATTERNS` entry `"subway"`
pub fn pattern_subway(ctx: &mut Canvas, w: f64, h: f64, m: &str, _m2: &str) {
    ctx.set_stroke_style(m);
    ctx.set_line_width(1.5);
    let mut y = 0.0;
    while y < h {
        ctx.begin_path();
        ctx.move_to(0, y);
        ctx.line_to(w, y);
        ctx.stroke();
        let row = to_i32(y / 40.0);
        let ox = if row % 2 == 0 { 0.0 } else { 40.0 };
        let mut x = ox;
        while x < w {
            ctx.begin_path();
            ctx.move_to(x, y);
            ctx.line_to(x, y + 40.0);
            ctx.stroke();
            x += 80.0;
        }
        y += 40.0;
    }
}
/// `WALL_PATTERNS` entry `"wave"`
pub fn pattern_wave(ctx: &mut Canvas, w: f64, h: f64, m: &str, _m2: &str) {
    ctx.set_stroke_style(m);
    ctx.set_line_width(2);
    let mut y = 0.0;
    while y < h {
        ctx.begin_path();
        let mut x = 0.0;
        while x <= w {
            let yy = y + (x * 0.08).sin() * 6.0;
            if x == 0.0 {
                ctx.move_to(x, yy);
            } else {
                ctx.line_to(x, yy);
            }
            x += 6.0;
        }
        ctx.stroke();
        y += 22.0;
    }
}
/// `WALL_PATTERNS` entry `"tartan"`
pub fn pattern_tartan(ctx: &mut Canvas, w: f64, h: f64, m: &str, m2: &str) {
    let mut x = 0.0;
    while x < w {
        ctx.set_fill_style(m);
        ctx.fill_rect(x, 0, 12, h);
        x += 42.0;
    }
    let mut y = 0.0;
    while y < h {
        ctx.set_fill_style(m2);
        ctx.fill_rect(0, y, w, 8);
        y += 42.0;
    }
    // fine highlight
    ctx.set_fill_style("rgba(255,255,255,0.05)");
    let mut x = 21.0;
    while x < w {
        ctx.fill_rect(x, 0, 1, h);
        x += 42.0;
    }
}
/// `WALL_PATTERNS` entry `"trellis"`
pub fn pattern_trellis(ctx: &mut Canvas, w: f64, h: f64, m: &str, _m2: &str) {
    ctx.set_stroke_style(m);
    ctx.set_line_width(1.5);
    let mut d = -h;
    while d < w {
        ctx.begin_path();
        ctx.move_to(d, 0);
        ctx.line_to(d + h, h);
        ctx.stroke();
        ctx.begin_path();
        ctx.move_to(d + h, 0);
        ctx.line_to(d, h);
        ctx.stroke();
        d += 34.0;
    }
}
/// `WALL_PATTERNS` entry `"ovals"`
pub fn pattern_ovals(ctx: &mut Canvas, w: f64, h: f64, m: &str, _m2: &str) {
    ctx.set_stroke_style(m);
    ctx.set_line_width(1.5);
    let mut y = 40.0;
    while y < h {
        let mut x = 40.0;
        while x < w {
            let ox = if to_i32(y / 80.0) % 2 == 0 { 0.0 } else { 30.0 };
            ctx.begin_path();
            ctx.ellipse(x + ox, y, 18, 32, 0, 0, 7);
            ctx.stroke();
            x += 60.0;
        }
        y += 80.0;
    }
}
/// `WALL_PATTERNS` entry `"paisley"`
pub fn pattern_paisley(ctx: &mut Canvas, w: f64, h: f64, m: &str, _m2: &str) {
    ctx.set_fill_style(m);
    let mut y = 40.0;
    while y < h {
        let mut x = 40.0;
        while x < w {
            let ox = if to_i32(y / 96.0) % 2 == 0 { 0.0 } else { 48.0 };
            // teardrop
            ctx.begin_path();
            ctx.move_to(x + ox, y);
            ctx.quadratic_curve_to(x + ox + 34.0, y + 8.0, x + ox + 24.0, y + 32.0);
            ctx.quadratic_curve_to(x + ox + 4.0, y + 40.0, x + ox - 8.0, y + 24.0);
            ctx.quadratic_curve_to(x + ox - 8.0, y + 8.0, x + ox, y);
            ctx.fill();
            x += 96.0;
        }
        y += 96.0;
    }
}
/// `WALL_PATTERNS` entry `"stars"` (draws 120 randoms)
pub fn pattern_stars(ctx: &mut Canvas, w: f64, h: f64, m: &str, _m2: &str) {
    ctx.set_fill_style(m);
    for _ in 0..40 {
        let cx = rand1(w);
        let cy = rand1(h);
        let r = rand2(3.0, 6.0);
        ctx.begin_path();
        for s in 0..10 {
            let rr = if s % 2 == 0 { r } else { r * 0.45 };
            let a = (s as f64 / 10.0) * PI * 2.0 - PI / 2.0;
            let px = cx + a.cos() * rr;
            let py = cy + a.sin() * rr;
            if s == 0 {
                ctx.move_to(px, py);
            } else {
                ctx.line_to(px, py);
            }
        }
        ctx.close_path();
        ctx.fill();
    }
}
/// `WALL_PATTERNS` entry `"dotsLine"`
pub fn pattern_dots_line(ctx: &mut Canvas, w: f64, h: f64, m: &str, _m2: &str) {
    ctx.set_fill_style(m);
    let mut y = 24.0;
    while y < h {
        let mut x = 8.0;
        while x < w {
            ctx.begin_path();
            ctx.arc(x, y, 1.8, 0, 7);
            ctx.fill();
            x += 14.0;
        }
        y += 36.0;
    }
}
/// `WALL_PATTERNS` entry `"arches"`
pub fn pattern_arches(ctx: &mut Canvas, w: f64, h: f64, m: &str, _m2: &str) {
    ctx.set_stroke_style(m);
    ctx.set_line_width(2);
    let mut y = 0.0;
    while y < h {
        let mut x = -40.0;
        while x < w {
            ctx.begin_path();
            ctx.arc(x + 40.0, y + 80.0, 40, PI, 0);
            ctx.stroke();
            x += 80.0;
        }
        y += 80.0;
    }
}
/// `WALL_PATTERNS` entry `"crosses"`
pub fn pattern_crosses(ctx: &mut Canvas, w: f64, h: f64, m: &str, _m2: &str) {
    ctx.set_fill_style(m);
    let mut y = 24.0;
    while y < h {
        let mut x = 24.0;
        while x < w {
            ctx.fill_rect(x - 6.0, y - 1.0, 12, 2);
            ctx.fill_rect(x - 1.0, y - 6.0, 2, 12);
            x += 48.0;
        }
        y += 48.0;
    }
}
/// `WALL_PATTERNS` entry `"vines"`
pub fn pattern_vines(ctx: &mut Canvas, w: f64, h: f64, m: &str, _m2: &str) {
    ctx.set_stroke_style(m);
    ctx.set_line_width(1.5);
    let mut x = 40.0;
    while x < w {
        ctx.begin_path();
        let mut y = 0.0;
        while y <= h {
            let xx = x + (y * 0.06).sin() * 22.0;
            if y == 0.0 {
                ctx.move_to(xx, y);
            } else {
                ctx.line_to(xx, y);
            }
            y += 6.0;
        }
        ctx.stroke();
        // little leaves
        ctx.set_fill_style(m);
        let mut y = 40.0;
        while y < h {
            let xx = x + (y * 0.06).sin() * 22.0;
            ctx.begin_path();
            ctx.ellipse(xx + 10.0, y, 6, 3, 0.6, 0, 7);
            ctx.fill();
            ctx.begin_path();
            ctx.ellipse(xx - 10.0, y + 20.0, 6, 3, -0.6, 0, 7);
            ctx.fill();
            y += 60.0;
        }
        x += 100.0;
    }
}

/// Every pattern renderer by key, in declaration order.
pub static WALL_PATTERNS: &[(&str, WallPatternFn)] = &[
    ("plain", pattern_plain),
    ("stripe", pattern_stripe),
    ("stripeDouble", pattern_stripe_double),
    ("stripeHoriz", pattern_stripe_horiz),
    ("grid", pattern_grid),
    ("diamond", pattern_diamond),
    ("flock", pattern_flock),
    ("chevron", pattern_chevron),
    ("herringbone", pattern_herringbone),
    ("polka", pattern_polka),
    ("hex", pattern_hex),
    ("artDeco", pattern_art_deco),
    ("subway", pattern_subway),
    ("wave", pattern_wave),
    ("tartan", pattern_tartan),
    ("trellis", pattern_trellis),
    ("ovals", pattern_ovals),
    ("paisley", pattern_paisley),
    ("stars", pattern_stars),
    ("dotsLine", pattern_dots_line),
    ("arches", pattern_arches),
    ("crosses", pattern_crosses),
    ("vines", pattern_vines),
];

/// The `WALL_PATTERNS` renderer for `kind`, `pattern_plain` when unknown
pub fn wall_pattern(kind: &str) -> WallPatternFn {
    WALL_PATTERNS.iter().find(|p| p.0 == kind).map(|p| p.1).unwrap_or(pattern_plain)
}

/// Patches of damp green over any wallpaper.
pub fn draw_moss_overlay(ctx: &mut Canvas, w: f64, h: f64) {
    // large patches concentrated near the top (grows down from ceiling)
    for _ in 0..30 {
        let cx = rand1(w);
        let cy = rand2(0.0, h * 0.7);
        let r = rand2(20.0, 90.0);
        let shade = irand(0, 30);
        // triple-draw to wrap seamlessly across the horizontal seam
        for dx in [0.0, -w, w] {
            let g = ctx.create_radial_gradient(cx + dx, cy, 2, cx + dx, cy, r);
            g.add_color_stop(0, &format!("rgba({},{},{},0.55)", 40 + shade, 70 + shade, 30 + shade));
            g.add_color_stop(0.5, &format!("rgba({},{},{},0.35)", 30 + shade, 55 + shade, 20 + shade));
            g.add_color_stop(1, "rgba(30,55,20,0)");
            ctx.set_fill_style(&g);
            ctx.fill_rect(0, 0, w, h);
        }
    }
    // fuzzy fibrous specks (moss "hair") — small 1-2 px, don't wrap
    for _ in 0..900 {
        ctx.set_fill_style(format!("rgba({},{},{},{})", irand(30, 70), irand(60, 100), irand(25, 60), rand2(0.15, 0.5)));
        ctx.fill_rect(rand1(w), rand1(h * 0.85), rand2(0.8, 2.2), rand2(0.8, 2.2));
    }
    // drips of dampness — draw once, then wrap for any that would clip the right edge
    for _ in 0..20 {
        let x = rand1(w);
        let width = rand2(3.0, 8.0);
        for dx in [0.0, -w, w] {
            let g2 = ctx.create_linear_gradient(x + dx, 0, x + dx, h * 0.5);
            g2.add_color_stop(0, "rgba(20,35,15,0.35)");
            g2.add_color_stop(1, "rgba(20,35,15,0)");
            ctx.set_fill_style(&g2);
            ctx.fill_rect(x + dx, 0, width, h * 0.5);
        }
    }
}

// ============================================================================
// palettes
// ============================================================================

/// One `WALL_PALETTES` entry: base gradient triple, baseboard and motif tint.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WallPalette {
    pub name: &'static str,
    pub top: &'static str,
    pub mid: &'static str,
    pub bot: &'static str,
    pub base: &'static str,
    pub shadow: &'static str,
    /// `[r, g, b]`
    pub motif_tint: [i64; 3],
}

pub static WALL_PALETTES: [WallPalette; 21] = [
    WallPalette {
        name: "classic-yellow",
        top: "#b5a352",
        mid: "#a89848",
        bot: "#8a7b34",
        base: "#4a3d24",
        shadow: "#2e2513",
        motif_tint: [70, 52, 18],
    },
    WallPalette {
        name: "pristine-yellow",
        top: "#d4c165",
        mid: "#c4b358",
        bot: "#a89844",
        base: "#5a4a2a",
        shadow: "#332615",
        motif_tint: [80, 60, 20],
    },
    WallPalette {
        name: "mustard",
        top: "#9a8028",
        mid: "#8b721f",
        bot: "#6d5615",
        base: "#3a2e14",
        shadow: "#1a1408",
        motif_tint: [40, 25, 8],
    },
    WallPalette {
        name: "olive",
        top: "#8a8848",
        mid: "#7a7838",
        bot: "#5a5828",
        base: "#333018",
        shadow: "#1c1a0c",
        motif_tint: [30, 40, 20],
    },
    WallPalette {
        name: "sage-green",
        top: "#a8b490",
        mid: "#94a07c",
        bot: "#6d7858",
        base: "#3a4028",
        shadow: "#1e2214",
        motif_tint: [40, 55, 30],
    },
    WallPalette {
        name: "salmon",
        top: "#c99a88",
        mid: "#b88872",
        bot: "#8a5c4a",
        base: "#4a2f22",
        shadow: "#28180f",
        motif_tint: [80, 40, 30],
    },
    WallPalette {
        name: "dusty-rose",
        top: "#c4a4a8",
        mid: "#a88a90",
        bot: "#7a5f66",
        base: "#3e2830",
        shadow: "#221418",
        motif_tint: [90, 50, 60],
    },
    WallPalette {
        name: "cream-beige",
        top: "#d8ceb4",
        mid: "#c8bea4",
        bot: "#a0967c",
        base: "#4e422e",
        shadow: "#2a2216",
        motif_tint: [90, 70, 50],
    },
    WallPalette {
        name: "powder-blue",
        top: "#98a8b4",
        mid: "#84939f",
        bot: "#606d78",
        base: "#2c333a",
        shadow: "#161a1e",
        motif_tint: [40, 55, 70],
    },
    WallPalette {
        name: "butter-yellow",
        top: "#e2d494",
        mid: "#d0c284",
        bot: "#a89b60",
        base: "#5c4f2a",
        shadow: "#332a12",
        motif_tint: [110, 90, 40],
    },
    WallPalette {
        name: "faded-gold",
        top: "#c4a854",
        mid: "#b39847",
        bot: "#8a7530",
        base: "#4a3d18",
        shadow: "#28200a",
        motif_tint: [90, 65, 25],
    },
    WallPalette {
        name: "rust-orange",
        top: "#a45028",
        mid: "#93441f",
        bot: "#6d3115",
        base: "#3a1c0c",
        shadow: "#1e0e05",
        motif_tint: [70, 25, 12],
    },
    WallPalette {
        name: "lavender",
        top: "#a898b8",
        mid: "#948499",
        bot: "#6a5c72",
        base: "#332a3a",
        shadow: "#1a141f",
        motif_tint: [55, 45, 75],
    },
    WallPalette {
        name: "warm-tan",
        top: "#b8956a",
        mid: "#a68455",
        bot: "#7a6238",
        base: "#3e2f18",
        shadow: "#22190c",
        motif_tint: [80, 60, 30],
    },
    WallPalette {
        name: "ochre",
        top: "#b48838",
        mid: "#a37828",
        bot: "#7c5a1a",
        base: "#42300c",
        shadow: "#241a05",
        motif_tint: [90, 55, 15],
    },
    WallPalette {
        name: "muted-teal",
        top: "#588a88",
        mid: "#4a7876",
        bot: "#345a58",
        base: "#1a2e2d",
        shadow: "#0c1a19",
        motif_tint: [30, 65, 60],
    },
    WallPalette {
        name: "peach",
        top: "#d4a488",
        mid: "#c19274",
        bot: "#946b50",
        base: "#4a3020",
        shadow: "#26180f",
        motif_tint: [90, 55, 40],
    },
    WallPalette {
        name: "sickly-green",
        top: "#7a8848",
        mid: "#68762c",
        bot: "#4a5620",
        base: "#242e10",
        shadow: "#131807",
        motif_tint: [40, 55, 20],
    },
    WallPalette {
        name: "faded-red",
        top: "#984848",
        mid: "#843a3a",
        bot: "#5c2626",
        base: "#301818",
        shadow: "#180c0c",
        motif_tint: [80, 25, 25],
    },
    WallPalette {
        name: "cornflower",
        top: "#8a94b4",
        mid: "#7684a4",
        bot: "#525e7c",
        base: "#2a3244",
        shadow: "#161a24",
        motif_tint: [50, 60, 90],
    },
    WallPalette {
        name: "mono-yellow",
        top: "#dcd88c",
        mid: "#cdc97a",
        bot: "#aeaa5b",
        base: "#615d30",
        shadow: "#353118",
        motif_tint: [125, 115, 48],
    },
];
/// Index of "mono-yellow", the last `WALL_PALETTES` entry
pub const PAL_MONO_YELLOW: usize = 20;

// ============================================================================
// wallpaper
// ============================================================================

/// `grime_lvl`: 0 = clean,
/// 1 = aged, 2 = dirty, 3 = ruined. 512×512, repeating.
pub fn make_wallpaper(palette: &WallPalette, pattern_kind: &str, grime_lvl: i64) -> Tex {
    let g = grime_lvl as f64;
    let [mr, mg, mb] = palette.motif_tint;
    // motif is *very* subtle — old faded pattern rather than fresh paint
    let motif_alpha = 0.028 + g * 0.010;
    let motif = format!("rgba({},{},{},{})", mr, mg, mb, motif_alpha);
    let motif2 =
        format!("rgba({},{},{},{})", (mr as f64 * 0.75).floor(), (mg as f64 * 0.75).floor(), (mb as f64 * 0.75).floor(), motif_alpha * 1.4);
    let palette = *palette;
    canvas_tex(512, 512, move |ctx, w, h| {
        // base gradient — slight uneven wash from ceiling to floor
        let bg = ctx.create_linear_gradient(0, 0, 0, h);
        bg.add_color_stop(0, palette.top);
        bg.add_color_stop(0.55, palette.mid);
        bg.add_color_stop(1, palette.bot);
        ctx.set_fill_style(&bg);
        ctx.fill_rect(0, 0, w, h);
        // large low-frequency splotches of shade — helps kill the "flat painted" look
        for _ in 0..10 {
            let cx = rand1(w);
            let cy = rand1(h);
            let dark = rnd() < 0.5;
            radial_wrap(
                ctx,
                w,
                h,
                cx,
                cy,
                10.0,
                rand2(120.0, 300.0),
                if dark { "rgba(0,0,0,0.06)" } else { "rgba(255,240,210,0.05)" },
                "rgba(0,0,0,0)",
            );
        }
        // pattern render (very subtle now)
        wall_pattern(pattern_kind)(ctx, w, h, &motif, &motif2);
        // fibrous noise — kept, gives the "old paper" grain
        for _ in 0..1600 {
            ctx.set_fill_style(format!("rgba({},{},{},0.05)", irand(mr - 20, mr + 30), irand(mg - 20, mg + 30), irand(0, 40)));
            ctx.fill_rect(rand1(w), rand1(h), 1.4, 1.4);
        }
        // faint horizontal scuff lines
        ctx.set_stroke_style("rgba(0,0,0,0.05)");
        ctx.set_line_width(0.5);
        for _ in 0..22 {
            let y = rand1(h);
            ctx.begin_path();
            ctx.move_to(rand1(w - 40.0), y);
            ctx.line_to(rand1(w - 40.0) + rand2(4.0, 24.0), y + rand2(-1.0, 1.0));
            ctx.stroke();
        }
        // grime — scales with dirtiness
        grime(ctx, w, h, 20.0 + g * 60.0, 0.05 + g * 0.06);
        // yellow/brown age spots — always some, more when dirty
        for _ in 0..times(8.0 + g * 12.0) {
            let cx = rand1(w);
            let cy = rand1(h);
            radial_wrap(ctx, w, h, cx, cy, 2.0, rand2(12.0, 40.0), &format!("rgba(80,50,20,{})", 0.08 + g * 0.06), "rgba(80,50,20,0)");
        }
        // torn patches only at high grime: where the paper came away, the
        // paler backing paper under it, its torn edge ragged and soft, a
        // pale fringe of the paper's core along it and the lifted edge's
        // faint shadow (the same draws; they were a hard polygon fanned from
        // its middle, outlined dark, read as a leaf or an arrow cut out and
        // stuck on: M8)
        if g >= 2.0 {
            for _ in 0..times((g - 1.0) * 3.0) {
                let cx = rand1(w);
                let cy = rand2(60.0, h - 120.0);
                let rr = rand2(14.0, 34.0);
                let pts = irand(6, 10);
                let mut outline = vec![];
                for p in 0..pts {
                    let a = (p as f64 / pts as f64) * PI * 2.0 + rand2(-0.4, 0.4);
                    let r = rr * rand2(0.55, 1.2);
                    outline.push((cx + a.cos() * r, cy + a.sin() * r * 0.7));
                }
                let torn = torn_edge(&outline, cx);
                let trace = |ctx: &mut Canvas, dy: f64| {
                    ctx.begin_path();
                    ctx.move_to(torn[0].0, torn[0].1 + dy);
                    for &(x, y) in &torn[1..] {
                        ctx.line_to(x, y + dy);
                    }
                    ctx.close_path();
                };
                ctx.save();
                ctx.set_shadow_blur(3.0);
                ctx.set_shadow_color("rgba(196,182,136,0.18)");
                ctx.set_fill_style("rgba(200,187,142,0.13)");
                trace(ctx, 0.0);
                ctx.fill();
                ctx.restore();
                ctx.set_line_width(1.1);
                ctx.set_stroke_style("rgba(40,28,10,0.09)");
                trace(ctx, 1.3);
                ctx.stroke();
                ctx.set_stroke_style("rgba(226,216,184,0.16)");
                trace(ctx, 0.0);
                ctx.stroke();
            }
        }
        // damp streaks from floor — subtle, scales with grime
        for _ in 0..times(2.0 + g * 8.0) {
            let x = rand1(w);
            let grad = ctx.create_linear_gradient(0, h, 0, h - rand2(80.0, 260.0));
            grad.add_color_stop(0, &format!("rgba(30,20,8,{})", 0.15 + g * 0.10));
            grad.add_color_stop(1, "rgba(30,20,8,0)");
            ctx.set_fill_style(&grad);
            ctx.fill_rect(x, h - 260.0, rand2(3.0, 10.0), 260);
        }
        // mold only when dirty
        if g >= 2.0 {
            for _ in 0..times(g * 8.0) {
                let cx = rand1(w);
                let cy = rand1(h * 0.7);
                radial_wrap(ctx, w, h, cx, cy, 2.0, rand2(10.0, 24.0), &format!("rgba(25,18,10,{})", 0.3 + g * 0.10), "rgba(25,18,10,0)");
            }
        }
        // baseboard
        ctx.set_fill_style(palette.base);
        ctx.fill_rect(0, 448, w, 64);
        ctx.set_fill_style(palette.shadow);
        ctx.fill_rect(0, 448, w, 6);
        ctx.set_fill_style("rgba(0,0,0,0.35)");
        ctx.fill_rect(0, 508, w, 4);
        // Subtle ceiling-corner shadow (only the top). No horizontal edge fade —
        // that would create dark vertical stripes at every tile boundary in a
        // corridor of identical tiles.
        let tg = ctx.create_linear_gradient(0, 0, 0, 38);
        tg.add_color_stop(0, "rgba(15,10,5,0.35)");
        tg.add_color_stop(1, "rgba(15,10,5,0)");
        ctx.set_fill_style(&tg);
        ctx.fill_rect(0, 0, w, 38);
    })
}

/// A torn edge round `outline`: each side broken into short jagged steps,
/// out and in by up to a pixel and a half, from a hash of where the patch
/// is (no draw)
fn torn_edge(outline: &[(f64, f64)], key: f64) -> Vec<(f64, f64)> {
    let hash = |k: usize, j: usize| {
        let mut h = (key * 977.0) as u64 ^ (k as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15) ^ (j as u64).wrapping_mul(0xc2b2_ae3d_27d4_eb4f);
        h ^= h >> 31;
        h = h.wrapping_mul(0xbf58_476d_1ce4_e5b9);
        h ^= h >> 29;
        (h % 10_000) as f64 / 10_000.0
    };
    let n = outline.len();
    let mut out = vec![];
    for k in 0..n {
        let (a, b) = (outline[k], outline[(k + 1) % n]);
        let (ex, ey) = (b.0 - a.0, b.1 - a.1);
        let l = ex.hypot(ey).max(1e-6);
        let (nx, ny) = (-ey / l, ex / l);
        let steps = ((l / 3.0).ceil() as usize).clamp(3, 16);
        for j in 0..steps {
            let t = j as f64 / steps as f64;
            let o = (hash(k, j) - 0.5) * 3.0;
            out.push((a.0 + ex * t + nx * o, a.1 + ey * t + ny * o));
        }
    }
    out
}

/// Curated soft-pattern list (`[kind, weight]`); plain
/// is heavily weighted so most walls read as unadorned old paper.
pub static SOFT_PATTERN_POOL: [(&str, f64); 13] = [
    ("plain", 8.0),
    ("plain", 8.0),
    ("plain", 8.0),
    ("polka", 1.0),
    ("hex", 1.0),
    ("ovals", 1.0),
    ("dotsLine", 1.0),
    ("stars", 1.0),
    ("paisley", 1.0),
    ("flock", 1.0),
    ("vines", 1.0),
    ("wave", 1.0),
    ("trellis", 1.0),
];
/// A weighted pick from `SOFT_PATTERN_POOL`: one `rnd()` draw from whatever
/// stream is active (the game calls it at startup, i.e. on the native
/// generator).
pub fn pick_soft_pattern() -> &'static str {
    let total = SOFT_PATTERN_POOL.iter().fold(0.0, |a, p| a + p.1);
    let mut r = rnd() * total;
    for &(k, w) in SOFT_PATTERN_POOL.iter() {
        r -= w;
        if r < 0.0 {
            return k;
        }
    }
    "plain"
}

/// Same base with a moss overlay on top.
pub fn make_moss_wallpaper(palette: &WallPalette) -> Tex {
    let [mr, mg, mb] = palette.motif_tint;
    let motif = format!("rgba({},{},{},0.03)", mr, mg, mb);
    let palette = *palette;
    canvas_tex(512, 512, move |ctx, w, h| {
        // very dirty base underneath so it reads as long-abandoned
        let bg = ctx.create_linear_gradient(0, 0, 0, h);
        bg.add_color_stop(0, palette.top);
        bg.add_color_stop(0.55, palette.mid);
        bg.add_color_stop(1, palette.bot);
        ctx.set_fill_style(&bg);
        ctx.fill_rect(0, 0, w, h);
        pattern_plain(ctx, w, h, &motif, &motif);
        // dampness pass first
        grime(ctx, w, h, 140.0, 0.14);
        // moss overlay
        draw_moss_overlay(ctx, w, h);
        // dark baseboard
        ctx.set_fill_style(palette.shadow);
        ctx.fill_rect(0, 448, w, 64);
        ctx.set_fill_style("rgba(20,32,15,0.6)");
        ctx.fill_rect(0, 448, w, 20);
        // ceiling-corner shadow only
        let tg = ctx.create_linear_gradient(0, 0, 0, 38);
        tg.add_color_stop(0, "rgba(5,10,5,0.4)");
        tg.add_color_stop(1, "rgba(5,10,5,0)");
        ctx.set_fill_style(&tg);
        ctx.fill_rect(0, 0, w, 38);
    })
}

/// One `wallpaper_variants` entry.
#[derive(Clone, Debug)]
pub struct WallpaperVariant {
    pub tex: Tex,
    /// (0..3; moss variants are 3)
    pub grime_lvl: i64,
    pub moss: bool,
    /// Index into `WALL_PALETTES`
    pub palette_idx: usize,
    /// The pattern `pick_soft_pattern()` chose (`"plain"` for moss), for inspection.
    pub pattern: &'static str,
}

/// `wallpaper_variants` — for every palette one variant per grime level 0..3,
/// then two moss variants on random palettes (86 entries). Draws
/// `pick_soft_pattern` / `irand` outside any `canvas_tex`, so call it on the native
/// RNG (`Textures::new` does).
pub fn make_wallpaper_variants() -> Vec<WallpaperVariant> {
    let mut out = vec![];
    // For every palette, generate a variant at each grime level (clean/aged/dirty/ruined).
    for p_idx in 0..WALL_PALETTES.len() {
        let p = &WALL_PALETTES[p_idx];
        for g in 0..4 {
            let k = pick_soft_pattern();
            out.push(WallpaperVariant { tex: make_wallpaper(p, k, g), grime_lvl: g, moss: false, palette_idx: p_idx, pattern: k });
        }
    }
    // moss variants live in a distinct isolated look — 2 palettes chosen at random
    let moss_pals = [irand(0, WALL_PALETTES.len() as i64 - 1), irand(0, WALL_PALETTES.len() as i64 - 1)];
    for mi in moss_pals {
        let mi = mi as usize;
        out.push(WallpaperVariant {
            tex: make_moss_wallpaper(&WALL_PALETTES[mi]),
            grime_lvl: 3,
            moss: true,
            palette_idx: mi,
            pattern: "plain",
        });
    }
    out
}

// ============================================================================
// hand-drawn stroke alphabet
// ============================================================================

/// One glyph of `STROKES`: a list of polylines in a 0..1 box.
pub type Glyph = &'static [&'static [[f64; 2]]];

/// `STROKES[c]` — the hand-drawn stroke alphabet (uppercase, digits and basic
/// punctuation). `None` for characters without a glyph; `' '` is empty.
pub fn strokes(c: char) -> Option<Glyph> {
    Some(match c {
        '0' => &[
            &[[0.5, 0.02], [0.85, 0.15], [0.95, 0.5], [0.85, 0.85], [0.5, 0.98], [0.15, 0.85], [0.05, 0.5], [0.15, 0.15], [0.5, 0.02]],
            &[[0.15, 0.15], [0.85, 0.85]],
        ],
        '1' => &[&[[0.25, 0.25], [0.5, 0.05], [0.5, 1.0]], &[[0.25, 1.0], [0.75, 1.0]]],
        '2' => &[&[[0.1, 0.25], [0.3, 0.05], [0.7, 0.05], [0.9, 0.25], [0.9, 0.45], [0.1, 1.0], [0.9, 1.0]]],
        '3' => &[
            &[[0.1, 0.1], [0.7, 0.05], [0.9, 0.25], [0.7, 0.5], [0.4, 0.5]],
            &[[0.4, 0.5], [0.75, 0.55], [0.95, 0.75], [0.7, 0.98], [0.1, 0.9]],
        ],
        '4' => &[&[[0.7, 0.05], [0.05, 0.65], [0.95, 0.65]], &[[0.7, 0.05], [0.7, 1.0]]],
        '5' => &[&[[0.9, 0.05], [0.15, 0.05], [0.1, 0.5], [0.7, 0.5], [0.9, 0.7], [0.7, 0.98], [0.15, 0.92]]],
        '6' => &[&[[0.85, 0.1], [0.4, 0.15], [0.15, 0.5], [0.15, 0.85], [0.5, 1.0], [0.85, 0.85], [0.85, 0.65], [0.5, 0.5], [0.15, 0.6]]],
        '7' => &[&[[0.05, 0.05], [0.95, 0.05], [0.3, 1.0]]],
        '8' => &[&[[0.5, 0.05], [0.2, 0.2], [0.5, 0.5], [0.2, 0.85], [0.5, 0.98], [0.8, 0.85], [0.5, 0.5], [0.8, 0.2], [0.5, 0.05]]],
        '9' => &[&[[0.85, 0.9], [0.5, 1.0], [0.15, 0.85], [0.15, 0.5], [0.5, 0.4], [0.85, 0.5], [0.85, 0.2], [0.5, 0.05], [0.15, 0.2]]],
        'A' => &[&[[0.05, 1.0], [0.5, 0.05], [0.95, 1.0]], &[[0.22, 0.6], [0.78, 0.6]]],
        'B' => &[
            &[[0.1, 0.0], [0.1, 1.0]],
            &[[0.1, 0.02], [0.7, 0.05], [0.9, 0.25], [0.68, 0.48], [0.1, 0.5]],
            &[[0.1, 0.5], [0.72, 0.52], [0.95, 0.75], [0.7, 0.98], [0.1, 1.0]],
        ],
        'C' => &[&[[0.92, 0.15], [0.55, 0.02], [0.2, 0.2], [0.05, 0.5], [0.2, 0.82], [0.55, 1.0], [0.92, 0.85]]],
        'D' => &[&[[0.1, 0.0], [0.1, 1.0]], &[[0.1, 0.02], [0.6, 0.06], [0.9, 0.35], [0.9, 0.68], [0.6, 0.96], [0.1, 1.0]]],
        'E' => &[&[[0.9, 0.05], [0.1, 0.05], [0.1, 1.0], [0.9, 1.0]], &[[0.1, 0.52], [0.7, 0.52]]],
        'F' => &[&[[0.9, 0.05], [0.1, 0.05], [0.1, 1.0]], &[[0.1, 0.52], [0.7, 0.52]]],
        'G' => {
            &[&[[0.92, 0.15], [0.55, 0.02], [0.2, 0.2], [0.05, 0.5], [0.2, 0.82], [0.55, 1.0], [0.92, 0.85], [0.92, 0.55], [0.55, 0.55]]]
        }
        'H' => &[&[[0.1, 0.0], [0.1, 1.0]], &[[0.9, 0.0], [0.9, 1.0]], &[[0.1, 0.5], [0.9, 0.5]]],
        'I' => &[&[[0.5, 0.0], [0.5, 1.0]], &[[0.32, 0.0], [0.68, 0.0]], &[[0.32, 1.0], [0.68, 1.0]]],
        'J' => &[&[[0.7, 0.0], [0.7, 0.82], [0.5, 0.98], [0.25, 0.92], [0.15, 0.72]]],
        'K' => &[&[[0.1, 0.0], [0.1, 1.0]], &[[0.9, 0.0], [0.1, 0.5], [0.95, 1.0]]],
        'L' => &[&[[0.1, 0.0], [0.1, 1.0], [0.9, 1.0]]],
        'M' => &[&[[0.05, 1.0], [0.05, 0.0], [0.5, 0.7], [0.95, 0.0], [0.95, 1.0]]],
        'N' => &[&[[0.1, 1.0], [0.1, 0.0], [0.9, 1.0], [0.9, 0.0]]],
        'O' => {
            &[&[[0.5, 0.02], [0.85, 0.15], [0.98, 0.5], [0.85, 0.85], [0.5, 0.98], [0.15, 0.85], [0.02, 0.5], [0.15, 0.15], [0.5, 0.02]]]
        }
        'P' => &[&[[0.1, 1.0], [0.1, 0.0], [0.7, 0.05], [0.9, 0.25], [0.7, 0.48], [0.1, 0.5]]],
        'Q' => &[
            &[[0.5, 0.02], [0.85, 0.15], [0.98, 0.5], [0.85, 0.85], [0.5, 0.98], [0.15, 0.85], [0.02, 0.5], [0.15, 0.15], [0.5, 0.02]],
            &[[0.6, 0.7], [1.0, 1.05]],
        ],
        'R' => &[&[[0.1, 1.0], [0.1, 0.0], [0.7, 0.05], [0.9, 0.25], [0.7, 0.48], [0.1, 0.5]], &[[0.4, 0.5], [0.95, 1.0]]],
        'S' => {
            &[&[[0.9, 0.15], [0.5, 0.02], [0.15, 0.18], [0.15, 0.38], [0.5, 0.5], [0.85, 0.62], [0.85, 0.82], [0.5, 0.98], [0.15, 0.85]]]
        }
        'T' => &[&[[0.05, 0.05], [0.95, 0.05]], &[[0.5, 0.05], [0.5, 1.0]]],
        'U' => &[&[[0.1, 0.0], [0.1, 0.7], [0.3, 0.95], [0.7, 0.95], [0.9, 0.7], [0.9, 0.0]]],
        'V' => &[&[[0.05, 0.0], [0.5, 1.0], [0.95, 0.0]]],
        'W' => &[&[[0.05, 0.0], [0.2, 1.0], [0.5, 0.3], [0.8, 1.0], [0.95, 0.0]]],
        'X' => &[&[[0.05, 0.0], [0.95, 1.0]], &[[0.95, 0.0], [0.05, 1.0]]],
        'Y' => &[&[[0.05, 0.0], [0.5, 0.55], [0.95, 0.0]], &[[0.5, 0.55], [0.5, 1.0]]],
        'Z' => &[&[[0.05, 0.05], [0.95, 0.05], [0.05, 0.95], [0.95, 0.95]]],
        '!' => &[&[[0.5, 0.05], [0.5, 0.72]], &[[0.5, 0.9], [0.5, 0.98]]],
        '?' => &[&[[0.15, 0.15], [0.4, 0.02], [0.7, 0.05], [0.9, 0.22], [0.7, 0.42], [0.5, 0.55], [0.5, 0.7]], &[[0.5, 0.9], [0.5, 0.98]]],
        '.' => &[&[[0.5, 0.9], [0.5, 0.98]]],
        ',' => &[&[[0.5, 0.82], [0.4, 1.02]]],
        '\'' => &[&[[0.5, 0.02], [0.4, 0.22]]],
        '-' => &[&[[0.15, 0.5], [0.85, 0.5]]],
        '/' => &[&[[0.85, 0.05], [0.15, 1.0]]],
        '(' => &[&[[0.7, 0.05], [0.35, 0.3], [0.35, 0.7], [0.7, 0.95]]],
        ')' => &[&[[0.3, 0.05], [0.65, 0.3], [0.65, 0.7], [0.3, 0.95]]],
        ' ' => &[],
        _ => return None,
    })
}

/// The options `draw_hand_stroke` reads.
#[derive(Clone, Debug)]
pub struct StrokeOpts<'a> {
    pub color: &'a str,
    pub thickness: f64,
    pub jitter: f64,
    pub gap: f64,
    pub wobble: f64,
}

/// One polyline with a jittered pen:
/// sub-segments with random offsets, variable width and occasional gaps.
pub fn draw_hand_stroke(ctx: &mut Canvas, points: &[[f64; 2]], opts: &StrokeOpts) {
    let StrokeOpts { color, thickness, jitter, gap, wobble } = *opts;
    ctx.set_stroke_style(color);
    ctx.set_line_cap("round");
    ctx.set_line_join("round");
    for i in 0..points.len().saturating_sub(1) {
        let [x0, y0] = points[i];
        let [x1, y1] = points[i + 1];
        let seg_len = hypot2(x1 - x0, y1 - y0);
        let subs = 2f64.max((seg_len / 4.0).ceil()) as u64;
        for s in 0..subs {
            // occasional gap (pen lift)
            if rnd() < gap {
                continue;
            }
            let t0 = s as f64 / subs as f64;
            let t1 = (s + 1) as f64 / subs as f64;
            let px0 = x0 + (x1 - x0) * t0 + rand2(-jitter, jitter) + (t0 * 9.0).sin() * wobble;
            let py0 = y0 + (y1 - y0) * t0 + rand2(-jitter, jitter) + (t0 * 7.0).cos() * wobble;
            let px1 = x0 + (x1 - x0) * t1 + rand2(-jitter, jitter) + (t1 * 9.0).sin() * wobble;
            let py1 = y0 + (y1 - y0) * t1 + rand2(-jitter, jitter) + (t1 * 7.0).cos() * wobble;
            ctx.set_line_width(thickness * (0.6 + rnd() * 0.7));
            ctx.begin_path();
            ctx.move_to(px0, py0);
            ctx.line_to(px1, py1);
            ctx.stroke();
        }
    }
}

/// `handwritten_tex(text, opts)` options. `Default`: ink
/// `rgba(140,20,15,0.95)`, size 90, thickness 5, jitter 1.4, gap 0.02,
/// wobble 1.1, ink blobs on, no drip, single stroke, padding 24, 1024×220.
#[derive(Clone, Debug)]
pub struct HandOpts {
    pub color: String,
    pub size: f64,
    pub thickness: f64,
    pub jitter: f64,
    pub gap: f64,
    pub wobble: f64,
    pub ink_blobs: bool,
    pub drip: bool,
    pub double_stroke: bool,
    pub padding: f64,
    pub w: f64,
    pub h: f64,
}
impl Default for HandOpts {
    fn default() -> Self {
        HandOpts {
            color: "rgba(140,20,15,0.95)".into(),
            size: 90.0,
            thickness: 5.0,
            jitter: 1.4,
            gap: 0.02,
            wobble: 1.1,
            ink_blobs: true,
            drip: false,
            double_stroke: false,
            padding: 24.0,
            w: 1024.0,
            h: 220.0,
        }
    }
}

/// A line of hand-drawn text on a transparent,
/// non-repeating `o.w × o.h` canvas texture.
pub fn handwritten_tex(text: &str, o: &HandOpts) -> Tex {
    let o = o.clone();
    let chars: Vec<char> = text.to_uppercase().chars().collect();
    // pre-compute per-glyph width in unit space (all letters ~1 unit wide; space is 0.6)
    let widths: Vec<f64> = chars.iter().map(|&c| if c == ' ' { 0.55 } else { 0.9 }).collect();
    let total_w = widths.iter().fold(0.0, |a, b| a + b) + widths.len() as f64 * 0.15;
    let scale_x = (o.w - o.padding * 2.0) / total_w;
    let scale = nan_min(o.size, scale_x);
    let start_x = (o.w - total_w * scale) / 2.0;
    let base_y = (o.h - scale) / 2.0;
    canvas_tex_opt(o.w, o.h, false, move |ctx, w, h| {
        ctx.clear_rect(0, 0, w, h);
        let mut cursor = start_x;
        for i in 0..chars.len() {
            let glyph = strokes(chars[i]);
            let cw_unit = widths[i];
            if let Some(glyph) = glyph.filter(|g| !g.is_empty()) {
                // per-glyph baseline wobble + rotation
                let gy = base_y + rand2(-scale * 0.06, scale * 0.06);
                let gx = cursor + rand2(-scale * 0.03, scale * 0.03);
                let rot = rand2(-0.12, 0.12);
                let gscale_x = 1.0 + rand2(-0.08, 0.08);
                let gscale_y = 1.0 + rand2(-0.08, 0.14);
                ctx.save();
                ctx.translate(gx + (cw_unit * scale) / 2.0, gy + scale / 2.0);
                ctx.rotate(rot);
                ctx.scale(gscale_x, gscale_y);
                ctx.translate(-(cw_unit * scale) / 2.0, -scale / 2.0);
                // shadow for the ink glow feel: the glyph's strokes drawn as
                // one layer, its glow cast once (a blur a stroke was most of
                // the cost of a page of writing)
                ctx.set_shadow_color(&o.color);
                ctx.set_shadow_blur(2);
                // (the glyph's box, and room for its jitter, wobble and glow)
                let m = scale * 0.25 + o.thickness * 2.0 + 6.0;
                ctx.begin_layer_in(-m, -m, cw_unit * scale + 2.0 * m, scale + 2.0 * m);
                for poly in glyph.iter() {
                    // map unit coords to glyph-local pixel coords
                    let scaled: Vec<[f64; 2]> = poly.iter().map(|&[x, y]| [x * cw_unit * scale, y * scale]).collect();
                    draw_hand_stroke(
                        ctx,
                        &scaled,
                        &StrokeOpts { color: &o.color, thickness: o.thickness, jitter: o.jitter, gap: o.gap, wobble: o.wobble },
                    );
                    if o.double_stroke && rnd() < 0.6 {
                        // second pass — writer went over the letter
                        draw_hand_stroke(
                            ctx,
                            &scaled,
                            &StrokeOpts {
                                color: &o.color,
                                thickness: o.thickness * 0.7,
                                jitter: o.jitter * 1.4,
                                gap: o.gap * 2.0,
                                wobble: o.wobble,
                            },
                        );
                    }
                }
                // ink blob at random stroke endpoint (pooled ink)
                if o.ink_blobs && rnd() < 0.28 {
                    let poly = choice(glyph);
                    let [ex, ey] = poly[irand(0, poly.len() as i64 - 1) as usize];
                    ctx.set_fill_style(&o.color);
                    ctx.begin_path();
                    ctx.arc(ex * cw_unit * scale, ey * scale, rand2(1.5, 3.5), 0, 7);
                    ctx.fill();
                }
                ctx.end_layer();
                ctx.restore();
            }
            cursor += (cw_unit + 0.15) * scale;
        }
        // drips
        if o.drip {
            ctx.set_fill_style(&o.color);
            for _ in 0..6 {
                let dx = rand2(start_x + 20.0, start_x + total_w * scale - 20.0);
                let dy = base_y + scale * 0.65;
                let dh = rand2(10.0, 36.0);
                ctx.fill_rect(dx, dy, rand2(1.2, 2.4), dh);
                ctx.begin_path();
                ctx.arc(dx + 1.0, dy + dh, rand2(1.5, 3.0), 0, 7);
                ctx.fill();
            }
        }
        // occasional splatter
        if rnd() < 0.5 {
            ctx.set_fill_style(&o.color);
            // the loop bound is re-drawn on every test `i < irand(4, 12)`
            let mut i = 0;
            while i < irand(4, 12) {
                ctx.begin_path();
                ctx.arc(rand1(o.w), rand2(base_y - 10.0, base_y + scale + 10.0), rand2(0.4, 1.6), 0, 7);
                ctx.fill();
                i += 1;
            }
        }
    })
}

// ============================================================================
// floors
// ============================================================================

/// A carpet matching a grime level; 4 = moss.
pub fn make_carpet(grime_lvl: i64) -> Tex {
    let bases = ["#8a7752", "#7a6748", "#6d5c40", "#4c3f2a"]; // cleanest -> filthiest
    let base = if grime_lvl >= 4 { "#3a4a28" } else { bases[grime_lvl.min(3).max(0) as usize] };
    let gl = grime_lvl as f64;
    canvas_tex(512, 512, move |ctx, w, h| {
        ctx.set_fill_style(base);
        ctx.fill_rect(0, 0, w, h);
        let mut img = ctx.get_image_data(0, 0, w, h);
        {
            let d = &mut img.data;
            // per-pixel fiber noise, biased toward slight vertical grain
            let noise_amp = 24 + grime_lvl * 3;
            let (wi, hi) = (w as usize, h as usize);
            for y in 0..hi {
                for x in 0..wi {
                    let i = (y * wi + x) * 4;
                    let n = irand(-noise_amp, noise_amp) as f64;
                    let g = (if x % 3 == 0 { -6.0 } else { 0.0 }) + (if y % 4 == 0 { -4.0 } else { 0.0 });
                    d[i] = clamp(d[i] as f64 + n + g, 0.0, 255.0) as u8;
                    d[i + 1] = clamp(d[i + 1] as f64 + n + g, 0.0, 255.0) as u8;
                    d[i + 2] = clamp(d[i + 2] as f64 + round_half_up(n * 0.7) + g, 0.0, 255.0) as u8;
                }
            }
        }
        ctx.put_image_data(&img, 0, 0);
        // matted patches (worn traffic paths) — wrap seamlessly
        let path_count = 3.0 + gl * 3.0;
        for _ in 0..times(path_count) {
            let cx = rand1(w);
            let cy = rand1(h);
            let r = rand2(60.0, 140.0);
            let stop0 = format!("rgba(50,42,28,{})", 0.25 + gl * 0.08);
            for dx in [0.0, -w, w] {
                for dy in [0.0, -h, h] {
                    let g = ctx.create_radial_gradient(cx + dx, cy + dy, 5, cx + dx, cy + dy, r);
                    g.add_color_stop(0, &stop0);
                    g.add_color_stop(1, "rgba(50,42,28,0)");
                    ctx.set_fill_style(&g);
                    ctx.fill_rect(0, 0, w, h);
                }
            }
        }
        // moist stains — many more when dirty
        grime(ctx, w, h, 40.0 + gl * 45.0, 0.10 + gl * 0.05);
        // darker splotches at higher grime
        for _ in 0..times(2.0 + gl * 4.0) {
            ctx.set_fill_style(format!("rgba(20,14,6,{})", rand2(0.2, 0.45)));
            ctx.begin_path();
            ctx.ellipse(rand1(w), rand1(h), rand2(8.0, 26.0), rand2(5.0, 18.0), rand1(PI), 0, 7);
            ctx.fill();
        }
        // torn/curling edges at ruined
        if grime_lvl >= 3 {
            for _ in 0..6 {
                ctx.set_fill_style("rgba(120,95,55,0.55)");
                let cx = rand1(w);
                let cy = rand1(h);
                ctx.begin_path();
                ctx.move_to(cx, cy);
                for p in 0..8 {
                    let a = (p as f64 / 8.0) * PI * 2.0 + rand2(-0.4, 0.4);
                    let r = rand2(6.0, 18.0);
                    ctx.line_to(cx + a.cos() * r, cy + a.sin() * r);
                }
                ctx.close_path();
                ctx.fill();
            }
        }
        // moss variant: green fuzzy overlay + puddles
        if grime_lvl >= 4 {
            for _ in 0..20 {
                let cx = rand1(w);
                let cy = rand1(h);
                let g2 = ctx.create_radial_gradient(cx, cy, 4, cx, cy, rand2(30.0, 100.0));
                g2.add_color_stop(0, "rgba(40,70,25,0.55)");
                g2.add_color_stop(1, "rgba(40,70,25,0)");
                ctx.set_fill_style(&g2);
                ctx.fill_rect(0, 0, w, h);
            }
            for _ in 0..1200 {
                ctx.set_fill_style(format!("rgba({},{},{},{})", irand(30, 60), irand(60, 110), irand(20, 55), rand2(0.2, 0.5)));
                ctx.fill_rect(rand1(w), rand1(h), rand2(0.8, 2.0), rand2(0.8, 2.0));
            }
            // puddle
            for _ in 0..3 {
                let cx = rand1(w);
                let cy = rand1(h);
                let g3 = ctx.create_radial_gradient(cx, cy, 2, cx, cy, rand2(20.0, 60.0));
                g3.add_color_stop(0, "rgba(10,20,15,0.7)");
                g3.add_color_stop(1, "rgba(10,20,15,0)");
                ctx.set_fill_style(&g3);
                ctx.fill_rect(0, 0, w, h);
            }
        }
    })
}

// ============================================================================
// ceilings
// ============================================================================

/// `make_ceiling(opts)` options. `Default` (the "aged" ceiling): base
/// `#c6c0ae`, grid `#7d7768`, tile shade 0.03–0.12, speckle 40, 5 stains at
/// alpha 0.5, no cracks / missing tiles / holes, grime 40 at alpha 0.09.
#[derive(Clone, Debug)]
pub struct CeilingOpts {
    pub base_color: String,
    pub grid_color: String,
    /// `[min, max]` darkening per tile
    pub tile_shade: [f64; 2],
    pub speckle: f64,
    pub stains: f64,
    pub stain_alpha: f64,
    pub cracks: f64,
    pub missing: f64,
    pub holes: f64,
    pub grime_amount: f64,
    pub grime_alpha: f64,
}
impl Default for CeilingOpts {
    fn default() -> Self {
        CeilingOpts {
            base_color: "#c6c0ae".into(),
            grid_color: "#7d7768".into(),
            tile_shade: [0.03, 0.12],
            speckle: 40.0,
            stains: 5.0,
            stain_alpha: 0.5,
            cracks: 0.0,
            missing: 0.0,
            holes: 0.0,
            grime_amount: 40.0,
            grime_alpha: 0.09,
        }
    }
}

/// Cells of the ceiling's grid along a tile (4 m): 0.67 m, an acoustic
/// tile's size (2 ft is 0.61)
pub const CEIL_CELLS: usize = 6;

/// The ceiling's T-bar grid over a tile's texture (laid from the tile's
/// edge, `arch::plane_material`; its centre is three cells in): bars every
/// cell along u through the edge, and along v half a cell off it, so a
/// panel hung at a tile's centre (2 × 1 cells, `panel_geo`) fills cells
/// exactly
fn ceiling_grid(ctx: &mut Canvas, w: f64, h: f64, bar: &str) {
    let cell = w / CEIL_CELLS as f64;
    ctx.set_fill_style(bar);
    for k in 0..CEIL_CELLS {
        let x = k as f64 * cell;
        ctx.fill_rect(x - 1.5, 0, 3, h);
        if k == 0 {
            ctx.fill_rect(w - 1.5, 0, 3, h);
        }
        let y = (k as f64 + 0.5) * cell;
        ctx.fill_rect(0, y - 1.5, w, 3);
    }
    // the bars' shadow on the tiles beside them
    ctx.set_fill_style("rgba(0,0,0,0.22)");
    for k in 0..CEIL_CELLS {
        let x = k as f64 * cell;
        ctx.fill_rect(x + 1.5, 0, 1, h);
        if k == 0 {
            ctx.fill_rect(w - 2.5, 0, 1, h);
        }
        let y = (k as f64 + 0.5) * cell;
        ctx.fill_rect(0, y + 1.5, w, 1);
    }
}

/// Acoustic ceiling tiles, 6 × 6 a tile, on their grid (the tiles' face is
/// a photograph laid over them: `photo::arch_detail("acoustic")`).
pub fn make_ceiling(o: &CeilingOpts) -> Tex {
    let o = o.clone();
    canvas_tex(512, 512, move |ctx, w, h| {
        ctx.set_fill_style(&o.base_color);
        ctx.fill_rect(0, 0, w, h);
        let cell = w / CEIL_CELLS as f64;
        // each tile a little off its neighbours (sagged, yellowed, replaced)
        for i in 0..CEIL_CELLS {
            for j in 0..CEIL_CELLS {
                let (x, y) = (i as f64 * cell, (j as f64 + 0.5) * cell);
                ctx.set_fill_style(format!("rgba(0,0,0,{})", rand2(o.tile_shade[0], o.tile_shade[1]) * 0.7));
                // (the last row runs over the edge: the rest of it at the top)
                ctx.fill_rect(x, y, cell, (h - y).min(cell));
                if y + cell > h {
                    ctx.fill_rect(x, 0, cell, y + cell - h);
                }
            }
        }
        for _ in 0..times(o.speckle) {
            ctx.set_fill_style(format!("rgba(0,0,0,{})", rand2(0.1, 0.25)));
            ctx.fill_rect(rand1(w), rand1(h), 1, 1);
        }
        ceiling_grid(ctx, w, h, &o.grid_color);
        for _ in 0..times(o.stains) {
            let cx = rand1(w);
            let cy = rand1(h);
            let g = ctx.create_radial_gradient(cx, cy, 4, cx, cy, rand2(32.0, 80.0));
            g.add_color_stop(0, &format!("rgba(120,80,30,{})", o.stain_alpha));
            g.add_color_stop(0.6, &format!("rgba(90,60,22,{})", o.stain_alpha * 0.5));
            g.add_color_stop(1, "rgba(90,60,22,0)");
            ctx.set_fill_style(&g);
            ctx.fill_rect(0, 0, w, h);
        }
        // cracks
        for _ in 0..times(o.cracks) {
            ctx.set_stroke_style("rgba(0,0,0,0.55)");
            ctx.set_line_width(1.2);
            ctx.begin_path();
            let mut x = rand1(w);
            let mut y = rand1(h);
            ctx.move_to(x, y);
            // the loop bound is re-drawn on every test `s < irand(6, 14)`
            let mut s = 0;
            while s < irand(6, 14) {
                x += rand2(-24.0, 24.0);
                y += rand2(-16.0, 16.0);
                ctx.line_to(x, y);
                s += 1;
            }
            ctx.stroke();
        }
        // a missing tile: the dark behind it, a pipe across (never the
        // middle cells, where a panel hangs)
        for _ in 0..times(o.missing) {
            let cell = w / CEIL_CELLS as f64;
            let tx = irand(1, CEIL_CELLS as i64 - 2) as f64;
            let ty = irand(1, CEIL_CELLS as i64 - 3) as f64;
            let ox = tx * cell + 2.0;
            let oy = (ty + 0.5) * cell + 2.0;
            ctx.set_fill_style("rgba(0,0,0,0.85)");
            ctx.fill_rect(ox, oy, cell - 4.0, cell - 4.0);
            ctx.set_stroke_style("rgba(80,70,55,0.6)");
            ctx.set_line_width(3);
            ctx.begin_path();
            ctx.move_to(ox + 4.0, oy + cell * 0.35);
            ctx.line_to(ox + cell - 8.0, oy + cell * 0.35);
            ctx.stroke();
        }
        // dark round holes
        for _ in 0..times(o.holes) {
            let cx = rand2(60.0, w - 60.0);
            let cy = rand2(60.0, h - 60.0);
            let g = ctx.create_radial_gradient(cx, cy, 2, cx, cy, rand2(16.0, 34.0));
            g.add_color_stop(0, "rgba(0,0,0,0.9)");
            g.add_color_stop(1, "rgba(0,0,0,0)");
            ctx.set_fill_style(&g);
            ctx.fill_rect(0, 0, w, h);
        }
        grime(ctx, w, h, o.grime_amount, o.grime_alpha);
    })
}

/// Green fuzzy tiles with drips.
pub fn make_moss_ceiling() -> Tex {
    canvas_tex(512, 512, |ctx, w, h| {
        ctx.set_fill_style("#5a6a48");
        ctx.fill_rect(0, 0, w, h);
        // grid still present but darkened
        ceiling_grid(ctx, w, h, "#2c3524");
        for _ in 0..40 {
            let cx = rand1(w);
            let cy = rand1(h);
            let g = ctx.create_radial_gradient(cx, cy, 4, cx, cy, rand2(30.0, 90.0));
            g.add_color_stop(0, &format!("rgba({},{},{},0.6)", irand(40, 70), irand(70, 100), irand(25, 55)));
            g.add_color_stop(1, "rgba(40,70,25,0)");
            ctx.set_fill_style(&g);
            ctx.fill_rect(0, 0, w, h);
        }
        for _ in 0..1400 {
            ctx.set_fill_style(format!("rgba({},{},{},{})", irand(30, 60), irand(55, 95), irand(20, 45), rand2(0.15, 0.45)));
            ctx.fill_rect(rand1(w), rand1(h), rand2(0.7, 1.8), rand2(0.7, 1.8));
        }
        // dark hanging drip spots
        for _ in 0..15 {
            let cx = rand1(w);
            let cy = rand2(h * 0.6, h);
            let g = ctx.create_radial_gradient(cx, cy, 1, cx, cy, rand2(6.0, 14.0));
            g.add_color_stop(0, "rgba(0,0,0,0.7)");
            g.add_color_stop(1, "rgba(0,0,0,0)");
            ctx.set_fill_style(&g);
            ctx.fill_rect(0, 0, w, h);
        }
    })
}

/// `ceiling_variants` — by grime level: clean, aged (defaults), dirty, ruined, moss.
pub fn make_ceiling_variants() -> Vec<Tex> {
    vec![
        // clean — like new
        make_ceiling(&CeilingOpts {
            base_color: "#d8d2c0".into(),
            grid_color: "#8f887a".into(),
            tile_shade: [0.01, 0.05],
            stains: 0.0,
            cracks: 0.0,
            grime_amount: 6.0,
            grime_alpha: 0.05,
            ..Default::default()
        }),
        // aged (default)
        make_ceiling(&CeilingOpts::default()),
        // dirty — more stains, some cracks
        make_ceiling(&CeilingOpts {
            base_color: "#b8b09a".into(),
            grid_color: "#6a6558".into(),
            tile_shade: [0.04, 0.14],
            stains: 6.0,
            stain_alpha: 0.55,
            cracks: 2.0,
            grime_amount: 60.0,
            grime_alpha: 0.12,
            ..Default::default()
        }),
        // ruined — cracks, holes, one missing tile
        make_ceiling(&CeilingOpts {
            base_color: "#a89f8a".into(),
            grid_color: "#5b5548".into(),
            tile_shade: [0.06, 0.18],
            stains: 9.0,
            stain_alpha: 0.65,
            cracks: 4.0,
            missing: 1.0,
            holes: 3.0,
            grime_amount: 90.0,
            grime_alpha: 0.16,
            ..Default::default()
        }),
        // moss
        make_moss_ceiling(),
    ]
}

// ============================================================================
// small sprites
// ============================================================================

/// `light_tex` — the fluorescent ceiling panel (128×64, not repeating).
pub fn make_light_tex() -> Tex {
    canvas_tex_opt(128, 64, false, |ctx, w, h| {
        ctx.set_fill_style("#8f8a7a");
        ctx.fill_rect(0, 0, w, h);
        let g = ctx.create_radial_gradient(w / 2.0, h / 2.0, 4, w / 2.0, h / 2.0, w / 1.6);
        g.add_color_stop(0, "#fffbe8");
        g.add_color_stop(0.75, "#f3e9bd");
        g.add_color_stop(1, "#d9cf9f");
        ctx.set_fill_style(&g);
        ctx.fill_rect(5, 5, w - 10.0, h - 10.0);
        // faint bulb tubes
        ctx.set_fill_style("rgba(255,255,220,0.35)");
        ctx.fill_rect(12, 26, w - 24.0, 4);
        ctx.fill_rect(12, 34, w - 24.0, 4);
    })
}

/// `dust_sprite` — soft radial sprite for dust motes (64×64).
pub fn make_dust_sprite() -> Tex {
    canvas_tex_opt(64, 64, false, |ctx, w, h| {
        ctx.clear_rect(0, 0, w, h);
        let g = ctx.create_radial_gradient(w / 2.0, h / 2.0, 1, w / 2.0, h / 2.0, w / 2.0);
        g.add_color_stop(0, "rgba(255,240,200,1)");
        g.add_color_stop(0.4, "rgba(255,220,160,0.35)");
        g.add_color_stop(1, "rgba(255,220,160,0)");
        ctx.set_fill_style(&g);
        ctx.fill_rect(0, 0, w, h);
    })
}

/// `face_tex` — the glowing grinning face (256×256).
pub fn make_face_tex() -> Tex {
    canvas_tex_opt(256, 256, false, |ctx, _w, _h| {
        ctx.clear_rect(0, 0, 256, 256);
        // outer soft glow halo
        let halo = ctx.create_radial_gradient(128, 128, 20, 128, 128, 128);
        halo.add_color_stop(0, "rgba(255,220,200,0.35)");
        halo.add_color_stop(0.5, "rgba(255,180,150,0.10)");
        halo.add_color_stop(1, "rgba(255,180,150,0)");
        ctx.set_fill_style(&halo);
        ctx.fill_rect(0, 0, 256, 256);
        // glowing eyes
        ctx.set_shadow_color("#ffe8b0");
        ctx.set_shadow_blur(30);
        ctx.set_fill_style("#fff8d0");
        ctx.begin_path();
        ctx.ellipse(84, 92, 14, 24, 0, 0, 7);
        ctx.fill();
        ctx.begin_path();
        ctx.ellipse(172, 92, 14, 24, 0, 0, 7);
        ctx.fill();
        // sharper inner pupils
        ctx.set_shadow_blur(8);
        ctx.set_fill_style("#ffffff");
        ctx.begin_path();
        ctx.ellipse(84, 90, 5, 10, 0, 0, 7);
        ctx.fill();
        ctx.begin_path();
        ctx.ellipse(172, 90, 5, 10, 0, 0, 7);
        ctx.fill();
        // grinning mouth with teeth
        ctx.set_shadow_color("#ffffff");
        ctx.set_shadow_blur(18);
        ctx.set_stroke_style("#fff");
        ctx.set_line_width(12);
        ctx.set_line_cap("round");
        ctx.begin_path();
        ctx.move_to(48, 148);
        ctx.quadratic_curve_to(128, 232, 208, 148);
        ctx.stroke();
        // teeth zigzag inside the smile
        ctx.set_shadow_blur(4);
        ctx.set_stroke_style("#111");
        ctx.set_line_width(3);
        ctx.begin_path();
        ctx.move_to(60, 168);
        for i in 0..=12 {
            let x = 60 + i * 12;
            let y = 168 + if i % 2 == 0 { 6 } else { 16 };
            ctx.line_to(x, y);
        }
        ctx.stroke();
    })
}

/// `text_tex(text, opts)` options. `Default`: 256×128,
/// `bold 34px "Courier New", monospace`, white, glow 14, no rotation.
#[derive(Clone, Debug)]
pub struct TextTexOpts {
    pub w: f64,
    pub h: f64,
    pub font: String,
    pub color: String,
    pub glow: f64,
    pub rot: f64,
}
impl Default for TextTexOpts {
    fn default() -> Self {
        TextTexOpts { w: 256.0, h: 128.0, font: "bold 34px \"Courier New\", monospace".into(), color: "#fff".into(), glow: 14.0, rot: 0.0 }
    }
}

/// The macOS face to draw a CSS font with when the engine canvas would resolve
/// its first family to a different face (engine `family_font_name` maps
/// `Impact` to Papyrus). `None` = the engine canvas draws it correctly.
fn override_face(font_css: &str) -> Option<&'static str> {
    let spec = engine::css::parse_css_font(font_css);
    let first = spec.family.split(',').next().unwrap_or("").trim().trim_matches(|c| c == '"' || c == '\'').to_ascii_lowercase();
    match first.as_str() {
        "impact" => Some("Impact"),
        _ => None,
    }
}

/// Local stand-in for `ctx.fill_text` in `text_tex` when the engine canvas would
/// pick the wrong face: draws `text` with CoreText exactly as the engine's
/// `fill_text` does (centre / middle alignment, shadow glow, `translate(w/2,
/// h/2) rotate(rot)`), but with the named `face`, onto a transparent w×h layer.
fn core_text_layer(w: f64, h: f64, text: &str, face: &str, font_css: &str, color: &str, glow: f64, rot: f64) -> Canvas {
    use engine::cg::*;
    let (wi, hi) = (w.max(1.0) as usize, h.max(1.0) as usize);
    let mut buf = vec![0u8; wi * hi * 4];
    let spec = engine::css::parse_css_font(font_css);
    let col = engine::css::parse_css_color(color).unwrap_or([0.0, 0.0, 0.0, 1.0]);
    unsafe {
        let cs = CGColorSpaceCreateWithName(kCGColorSpaceSRGB);
        let c = CGBitmapContextCreate(
            buf.as_mut_ptr() as *mut std::ffi::c_void,
            wi,
            hi,
            8,
            wi * 4,
            cs,
            kCGImageAlphaPremultipliedLast | kCGBitmapByteOrder32Big,
        );
        CGContextTranslateCTM(c, 0.0, hi as f64);
        CGContextScaleCTM(c, 1.0, -1.0);
        // font (same trait handling as the engine canvas)
        let nm = cfstr(face);
        let mut font = CTFontCreateWithName(nm, spec.size, std::ptr::null());
        CFRelease(nm);
        let mut traits = 0u32;
        if spec.bold {
            traits |= kCTFontBoldTrait;
        }
        if spec.italic {
            traits |= kCTFontItalicTrait;
        }
        if traits != 0 {
            let f2 = CTFontCreateCopyWithSymbolicTraits(font, spec.size, std::ptr::null(), traits, traits);
            if !f2.is_null() {
                CFRelease(font as CFTypeRef);
                font = f2;
            }
        }
        let keys = [kCTFontAttributeName, kCTForegroundColorFromContextAttributeName];
        let vals = [font as CFTypeRef, kCFBooleanTrue];
        let dict = CFDictionaryCreate(
            std::ptr::null(),
            keys.as_ptr(),
            vals.as_ptr(),
            2,
            &kCFTypeDictionaryKeyCallBacks as *const _ as _,
            &kCFTypeDictionaryValueCallBacks as *const _ as _,
        );
        let s = cfstr(text);
        let astr = CFAttributedStringCreate(std::ptr::null(), s, dict);
        let line = CTLineCreateWithAttributedString(astr);
        CFRelease(astr);
        CFRelease(s);
        CFRelease(dict);
        let (mut a, mut d, mut l) = (0.0, 0.0, 0.0);
        let tw = CTLineGetTypographicBounds(line, &mut a, &mut d, &mut l);
        let (ascent, descent) = (CTFontGetAscent(font), CTFontGetDescent(font));
        // shadow colour = text colour, shadow blur = glow
        if col[3] > 0.0 && glow > 0.0 {
            let sc = CGColorCreate(cs, col.as_ptr());
            CGContextSetShadowWithColor(c, CGSize::default(), glow, sc);
            CGColorRelease(sc);
        }
        // translate(w / 2, h / 2), rotate(rot), text align centre, baseline middle
        CGContextTranslateCTM(c, w / 2.0, h / 2.0);
        let (sn, cn) = rot.sin_cos();
        CGContextConcatCTM(c, CGAffineTransform { a: cn, b: sn, c: -sn, d: cn, tx: 0.0, ty: 0.0 });
        CGContextTranslateCTM(c, -tw / 2.0, (ascent - descent) / 2.0);
        CGContextScaleCTM(c, 1.0, -1.0);
        CGContextSetTextMatrix(c, CGAffineTransform::IDENTITY);
        CGContextSetTextPosition(c, 0.0, 0.0);
        CGContextSetRGBFillColor(c, col[0], col[1], col[2], col[3]);
        CGContextSetTextDrawingMode(c, kCGTextFill);
        CTLineDraw(line, c);
        CGContextFlush(c);
        CFRelease(line as CFTypeRef);
        CFRelease(font as CFTypeRef);
        CGContextRelease(c);
        CGColorSpaceRelease(cs);
    }
    // premultiplied → straight alpha, then into a canvas
    for px in buf.chunks_exact_mut(4) {
        let a = px[3] as u32;
        if a == 0 || a == 255 {
            continue;
        }
        for k in 0..3 {
            px[k] = ((px[k] as u32 * 255 + a / 2) / a).min(255) as u8;
        }
    }
    let mut layer = Canvas::new(wi as u32, hi as u32);
    layer.put_image_data(&ImageData { width: wi as u32, height: hi as u32, data: buf }, 0, 0);
    layer
}

/// Centred glowing text on a transparent canvas.
///
/// For a font whose family the engine canvas maps to another face (`Impact`,
/// used by the COLA-Z vending machine) the `fill_text` is drawn by a local
/// CoreText helper with the right face; everything else is drawn by the
/// engine canvas's `fill_text`.
pub fn text_tex(text: &str, o: &TextTexOpts) -> Tex {
    let o = o.clone();
    let text = text.to_string();
    let (w, h) = (o.w, o.h);
    canvas_tex_opt(w, h, false, move |ctx, _, _| {
        ctx.clear_rect(0, 0, w, h);
        if let Some(face) = override_face(&o.font) {
            let layer = core_text_layer(w, h, &text, face, &o.font, &o.color, o.glow, o.rot);
            ctx.draw_image(&layer, 0, 0);
            return;
        }
        ctx.save();
        ctx.translate(w / 2.0, h / 2.0);
        ctx.rotate(o.rot);
        ctx.set_font(&o.font);
        ctx.set_text_align("center");
        ctx.set_text_baseline("middle");
        ctx.set_shadow_color(&o.color);
        ctx.set_shadow_blur(o.glow);
        ctx.set_fill_style(&o.color);
        ctx.fill_text(&text, 0, 0);
        ctx.restore();
    })
}

/// Refill the TV-static canvas with grey noise (one `irand`
/// per pixel, from the active stream) and flag the texture for upload.
/// `Textures::update_static` runs it on the native RNG.
pub fn draw_static(static_canvas: &mut Canvas, static_tex: &Tex) {
    let mut img = static_canvas.create_image_data(96, 64);
    {
        let d = &mut img.data;
        let mut i = 0;
        while i < d.len() {
            let v = irand(0, 255) as u8;
            d[i] = v;
            d[i + 1] = v;
            d[i + 2] = v;
            d[i + 3] = 255;
            i += 4;
        }
    }
    static_canvas.put_image_data(&img, 0, 0);
    static_canvas.update_tex(static_tex);
}

// ============================================================================
// startup textures
// ============================================================================

/// The textures built once at startup.
pub struct Textures {
    pub wallpaper_variants: Vec<WallpaperVariant>,
    /// = `wallpaper_variants[0].tex` (legacy binding)
    pub wallpaper_tex: Tex,
    /// `make_carpet(0) ..= make_carpet(4)` (index 4 = moss)
    pub carpet_variants: Vec<Tex>,
    /// = `carpet_variants[2]` (legacy binding)
    pub carpet_tex: Tex,
    /// (index 4 = moss)
    pub ceiling_variants: Vec<Tex>,
    /// = `ceiling_variants[0]` (legacy binding)
    pub ceiling_tex: Tex,
    pub light_tex: Tex,
    pub dust_sprite: Tex,
    pub face_tex: Tex,
    /// The TV-static canvas (96×64; drawn on directly)
    pub static_canvas: Canvas,
    /// Texture of `static_canvas`, sRGB, clamped
    pub static_tex: Tex,
}

impl Textures {
    /// 1×1 stand-ins with the real counts (for tests and `Game::new_bare`;
    /// `boot()` swaps in `Textures::new()`)
    pub fn placeholder() -> Textures {
        let t = || Tex::data_rgba(vec![200, 190, 140, 255], 1, 1);
        let wallpaper_variants: Vec<WallpaperVariant> = (0..86)
            .map(|i| WallpaperVariant {
                tex: t(),
                grime_lvl: (i % 4) as i64,
                moss: false,
                palette_idx: i % WALL_PALETTES.len(),
                pattern: "plain",
            })
            .collect();
        let carpet_variants: Vec<Tex> = (0..5).map(|_| t()).collect();
        let ceiling_variants: Vec<Tex> = (0..5).map(|_| t()).collect();
        let static_canvas = Canvas::new(96, 64);
        let static_tex = static_canvas.to_tex();
        Textures {
            wallpaper_tex: wallpaper_variants[0].tex.clone(),
            wallpaper_variants,
            carpet_tex: carpet_variants[2].clone(),
            carpet_variants,
            ceiling_tex: ceiling_variants[0].clone(),
            ceiling_variants,
            light_tex: t(),
            dust_sprite: t(),
            face_tex: t(),
            static_canvas,
            static_tex,
        }
    }
    /// Build every startup texture in a fixed order, on the native RNG.
    pub fn new() -> Textures {
        with_native(|| {
            let wallpaper_variants = make_wallpaper_variants();
            let wallpaper_tex = wallpaper_variants[0].tex.clone();
            let carpet_variants = vec![make_carpet(0), make_carpet(1), make_carpet(2), make_carpet(3), make_carpet(4)];
            let carpet_tex = carpet_variants[2].clone();
            let ceiling_variants = make_ceiling_variants();
            let ceiling_tex = ceiling_variants[0].clone();
            let light_tex = make_light_tex();
            let dust_sprite = make_dust_sprite();
            let face_tex = make_face_tex();
            let mut static_canvas = Canvas::new(96, 64);
            let static_tex = static_canvas.to_tex();
            static_tex.t().color_space = ColorSpace::Srgb;
            draw_static(&mut static_canvas, &static_tex);
            Textures {
                wallpaper_variants,
                wallpaper_tex,
                carpet_variants,
                carpet_tex,
                ceiling_variants,
                ceiling_tex,
                light_tex,
                dust_sprite,
                face_tex,
                static_canvas,
                static_tex,
            }
        })
    }

    /// `draw_static()` — redraw the TV static on the native RNG (it is only
    /// called from the frame loop, never inside a seeded level build).
    pub fn update_static(&mut self) {
        with_native(|| draw_static(&mut self.static_canvas, &self.static_tex));
    }
}

impl Default for Textures {
    fn default() -> Self {
        Textures::new()
    }
}
