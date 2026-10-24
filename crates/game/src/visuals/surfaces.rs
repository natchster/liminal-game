//! `visuals::surfaces` — canvas-drawn materials for the non-wallpaper parts of
//! the Backrooms: painted drywall, poured concrete, brick, pool tile, office
//! carpet tiles, concrete slab and plaster ceilings.
//!
//! Every texture is seamless in both directions (features that cross an edge
//! are drawn again on the far side) so it can be mapped in world space onto
//! boxes of any size. Textures are built lazily and cached, so a level only
//! pays for the surfaces it actually uses.
//!
//! `make_surfaces(rng)`: the texture factory is always the game's own
//! `canvas_tex` (colour space, wrapping,
//! anisotropy); `rng` is used for the texture noise (the game passes the
//! native generator: textures are cached across levels, never level-seeded).
//!
//! grime: 0 clean · 1 aged · 2 dirty · 3 ruined

use crate::placers::kit::canvas_tex;
use engine::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;

/// Metres covered by one texture repeat `[u, v]`.
pub static SURFACE_SCALE: [(&str, [f64; 2]); 10] = [
    ("drywall", [4.0, 3.1]),
    ("paint", [4.0, 3.1]),
    ("concrete", [3.2, 3.2]),
    ("brick", [2.0, 2.0]),
    ("tile", [1.2, 1.2]),
    ("floorConcrete", [4.0, 4.0]),
    ("floorOffice", [4.0, 4.0]),
    ("floorTile", [1.2, 1.2]),
    ("ceilSlab", [4.0, 4.0]),
    ("ceilPlaster", [4.0, 4.0]),
];

/// The random source the surfaces draw from (`rng`).
pub type Rng = fn() -> f64;

/// Uniform in [0, a): `rng() * a`
#[inline]
fn r1(rng: Rng, a: f64) -> f64 {
    rng() * a
}
/// Uniform in [a, b): `a + rng() * (b - a)`
#[inline]
fn r2(rng: Rng, a: f64, b: f64) -> f64 {
    a + rng() * (b - a)
}
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

/// Store a number as a clamped byte (clamp to 0..=255, NaN → 0, round half
/// to even).
fn u8_clamped(v: f64) -> u8 {
    if !(v > 0.0) {
        return 0;
    }
    if v >= 255.0 {
        return 255;
    }
    let f = v.floor();
    let d = v - f;
    let r = if d > 0.5 {
        f + 1.0
    } else if d < 0.5 {
        f
    } else if f % 2.0 == 0.0 {
        f
    } else {
        f + 1.0
    };
    r as u8
}

/// Call `f` at (x, y) and at every wrapped
/// copy that touches the canvas.
pub fn wrap_draw(w: f64, h: f64, x: f64, y: f64, r: f64, mut f: impl FnMut(f64, f64)) {
    for dx in [-w, 0.0, w] {
        for dy in [-h, 0.0, h] {
            let (xx, yy) = (x + dx, y + dy);
            if xx + r < 0.0 || yy + r < 0.0 || xx - r > w || yy - r > h {
                continue;
            }
            f(xx, yy);
        }
    }
}

/// A wrapped radial fade from `rgba`.
pub fn blotch(ctx: &mut Canvas, w: f64, h: f64, x: f64, y: f64, r: f64, rgba: &str) {
    wrap_draw(w, h, x, y, r, |xx, yy| {
        let g = ctx.create_radial_gradient(xx, yy, 1, xx, yy, r);
        g.add_color_stop(0, rgba);
        g.add_color_stop(1, "rgba(0,0,0,0)");
        ctx.set_fill_style(&g);
        ctx.fill_rect(xx - r, yy - r, r * 2.0, r * 2.0);
    });
}

/// `n` small square dots written into a transparent w×h layer, then
/// composited at (0, 0) in one `draw_image`. Every call starts from a fresh
/// layer.
pub fn speckle(rng: Rng, ctx: &mut Canvas, w: f64, h: f64, n: u32, rgb: [u8; 3], a_min: f64, a_max: f64, s_min: f64, s_max: f64) {
    let mut layer = Canvas::new(w as u32, h as u32);
    let mut img = layer.create_image_data(w, h);
    {
        let d = &mut img.data;
        let (wi, hi) = (w as i64, h as i64);
        for _ in 0..n {
            let a = r2(rng, a_min, a_max) * 255.0;
            let s = 1f64.max(round_half_up(r2(rng, s_min, s_max))) as i64;
            let x0 = r1(rng, w).floor() as i64;
            let y0 = r1(rng, h).floor() as i64;
            for yy in 0..s {
                for xx in 0..s {
                    let o = ((((y0 + yy) % hi) * wi + ((x0 + xx) % wi)) * 4) as usize;
                    d[o] = rgb[0];
                    d[o + 1] = rgb[1];
                    d[o + 2] = rgb[2];
                    d[o + 3] = u8_clamped((d[o + 3] as f64).max(a));
                }
            }
        }
    }
    layer.put_image_data(&img, 0, 0);
    ctx.draw_image(&layer, 0, 0);
}

/// Vertical damp streaks rising from the floor (walls only).
pub fn damp(rng: Rng, ctx: &mut Canvas, w: f64, h: f64, n: f64, alpha: f64) {
    for _ in 0..times(n) {
        let x = r1(rng, w);
        let top = r2(rng, h * 0.45, h * 0.9);
        let wd = r2(rng, 4.0, 16.0);
        let g = ctx.create_linear_gradient(0, h, 0, h - top);
        g.add_color_stop(0, &format!("rgba(40,30,12,{})", alpha));
        g.add_color_stop(1, "rgba(40,30,12,0)");
        ctx.set_fill_style(&g);
        for dx in [-w, 0.0, w] {
            ctx.fill_rect(x + dx, h - top, wd, top);
        }
    }
}

/// `#rrggbb` scaled by `1 + k`, as `rgb(r,g,b)`.
pub fn shade(hex: &str, k: f64) -> String {
    let n = i64::from_str_radix(&hex[1..], 16).unwrap_or(0);
    let f = |v: i64| 0f64.max(255f64.min(round_half_up(v as f64 * (1.0 + k))));
    format!("rgb({},{},{})", f(n >> 16), f((n >> 8) & 255), f(n & 255))
}

// ---------------------------------------------------------------- walls ---

/// Drywall paint colour per grime level.
pub const PAINT: [&str; 4] = ["#d9d3c2", "#cfc9b4", "#c2bba2", "#aea68a"];

/// Painted drywall: off-white/greige, roller
/// texture, scuffs at chair height, a dark vinyl cove base. Office walls,
/// bulkheads (`paint` = no base).
pub fn drywall(rng: Rng, g: i32, base: bool) -> Tex {
    let gf = g as f64;
    canvas_tex(512, 512, move |ctx, w, h| {
        let bg = ctx.create_linear_gradient(0, 0, 0, h);
        bg.add_color_stop(0, PAINT[g as usize]);
        bg.add_color_stop(1, &shade(PAINT[g as usize], -0.08));
        ctx.set_fill_style(&bg);
        ctx.fill_rect(0, 0, w, h);
        for _ in 0..14 {
            blotch(
                ctx,
                w,
                h,
                r1(rng, w),
                r1(rng, h),
                r2(rng, 60.0, 200.0),
                if rng() < 0.5 { "rgba(0,0,0,0.035)" } else { "rgba(255,250,235,0.04)" },
            );
        }
        speckle(rng, ctx, w, h, 5000, [90, 85, 70], 0.03, 0.08, 0.7, 1.6); // roller stipple
                                                                           // board seams every 1.2 m (4 m texture → 153.6 px), barely there
        ctx.set_fill_style("rgba(0,0,0,0.035)");
        let mut x = 0.0;
        while x < w {
            ctx.fill_rect(round_half_up(x), 0, 1, h);
            x += w / (4.0 / 1.2);
        }
        // scuffs and chair-rail knocks at 0.7–1.1 m
        for _ in 0..times(18.0 + gf * 22.0) {
            let y = h - (r2(rng, 0.7, 1.15) / 3.1) * h;
            let x = r1(rng, w);
            ctx.set_stroke_style(format!("rgba(30,28,24,{})", r2(rng, 0.05, 0.14 + gf * 0.05)));
            ctx.set_line_width(r2(rng, 0.5, 1.6));
            ctx.begin_path();
            ctx.move_to(x, y);
            ctx.line_to(x + r2(rng, 4.0, 26.0), y + r2(rng, -2.0, 2.0));
            ctx.stroke();
        }
        for _ in 0..times(gf * 6.0) {
            blotch(ctx, w, h, r1(rng, w), r2(rng, h * 0.1, h * 0.6), r2(rng, 20.0, 70.0), &format!("rgba(110,85,40,{})", 0.07 + gf * 0.04));
        }
        if g >= 2 {
            damp(rng, ctx, w, h, gf * 5.0, 0.1 + gf * 0.05);
        }
        if base {
            // 10 cm vinyl cove base
            let bh = round_half_up(h * (0.1 / 3.1));
            ctx.set_fill_style("#3a3935");
            ctx.fill_rect(0, h - bh, w, bh);
            ctx.set_fill_style("rgba(255,255,255,0.06)");
            ctx.fill_rect(0, h - bh, w, 2);
            // (kept on purpose: the w×bh layer is composited at (0, 0), i.e. the top rows)
            speckle(rng, ctx, w, bh, 300, [0, 0, 0], 0.05, 0.2, 0.8, 2.2);
        }
        // ceiling-line shadow
        let tg = ctx.create_linear_gradient(0, 0, 0, 30);
        tg.add_color_stop(0, "rgba(10,8,4,0.22)");
        tg.add_color_stop(1, "rgba(10,8,4,0)");
        ctx.set_fill_style(&tg);
        ctx.fill_rect(0, 0, w, 30);
    })
}

/// Poured concrete colour per grime level.
pub const CONC: [&str; 4] = ["#8e8a82", "#7f7b72", "#6c685f", "#58554d"];

/// Poured concrete: form-panel seams every 1.6 m, snap-tie
/// holes on a grid, efflorescence and rust bleed. Level 1 halls, Level 2 tunnels.
pub fn concrete(rng: Rng, g: i32) -> Tex {
    let gf = g as f64;
    canvas_tex(512, 512, move |ctx, w, h| {
        ctx.set_fill_style(CONC[g as usize]);
        ctx.fill_rect(0, 0, w, h);
        for _ in 0..26 {
            blotch(
                ctx,
                w,
                h,
                r1(rng, w),
                r1(rng, h),
                r2(rng, 40.0, 160.0),
                if rng() < 0.5 { "rgba(0,0,0,0.07)" } else { "rgba(255,255,255,0.05)" },
            );
        }
        speckle(rng, ctx, w, h, 9000, [30, 30, 28], 0.05, 0.16, 0.6, 1.8);
        speckle(rng, ctx, w, h, 2500, [210, 205, 195], 0.04, 0.12, 0.6, 1.4);
        // air pockets (bug holes)
        for _ in 0..160 {
            let r = r2(rng, 0.8, 2.4);
            let (x, y) = (r1(rng, w), r1(rng, h));
            wrap_draw(w, h, x, y, r, |xx, yy| {
                ctx.set_fill_style("rgba(20,20,18,0.45)");
                ctx.begin_path();
                ctx.arc(xx, yy, r, 0, 7);
                ctx.fill();
            });
        }
        // form seams: 3.2 m texture, panels 1.6 m → two per axis
        ctx.set_fill_style("rgba(25,24,22,0.35)");
        for f in [0.0, 0.5] {
            ctx.fill_rect(round_half_up(f * w), 0, 2, h);
            ctx.fill_rect(0, round_half_up(f * h), w, 2);
        }
        ctx.set_fill_style("rgba(255,255,255,0.08)");
        for f in [0.0, 0.5] {
            ctx.fill_rect(round_half_up(f * w) + 2.0, 0, 1, h);
            ctx.fill_rect(0, round_half_up(f * h) + 2.0, w, 1);
        }
        // snap-tie holes on a 0.8 m grid
        let step = w / 4.0;
        let mut y = step / 2.0;
        while y < h {
            let mut x = step / 2.0;
            while x < w {
                ctx.set_fill_style("rgba(15,14,12,0.7)");
                ctx.begin_path();
                ctx.arc(x, y, 3.2, 0, 7);
                ctx.fill();
                if g >= 1 && rng() < 0.25 + gf * 0.15 {
                    let len = r2(rng, 20.0, 90.0);
                    let gr = ctx.create_linear_gradient(0, y, 0, y + len);
                    gr.add_color_stop(0, &format!("rgba(120,60,20,{})", 0.25 + gf * 0.08));
                    gr.add_color_stop(1, "rgba(120,60,20,0)");
                    ctx.set_fill_style(&gr);
                    ctx.fill_rect(x - 1.5, y, 3, len);
                }
                x += step;
            }
            y += step;
        }
        // efflorescence + water tide marks
        for _ in 0..times(4.0 + gf * 5.0) {
            blotch(ctx, w, h, r1(rng, w), r1(rng, h), r2(rng, 30.0, 110.0), &format!("rgba(225,220,205,{})", 0.05 + gf * 0.03));
        }
        for _ in 0..times(gf * 7.0) {
            blotch(ctx, w, h, r1(rng, w), r1(rng, h), r2(rng, 20.0, 80.0), &format!("rgba(20,24,16,{})", 0.08 + gf * 0.05));
        }
    })
}

/// Running bond, per-brick tone, sooty mortar. Level 3.
pub fn brick(rng: Rng, g: i32) -> Tex {
    let gf = g as f64;
    canvas_tex(512, 512, move |ctx, w, h| {
        ctx.set_fill_style(["#8d8474", "#7a7263", "#655e52", "#4e4940"][g as usize]);
        ctx.fill_rect(0, 0, w, h); // mortar
        let (cols, rows) = (8i32, 24i32);
        let (bw, bh, m) = (w / cols as f64, h / rows as f64, 3.0);
        let tones: [[f64; 3]; 5] = [[120.0, 58.0, 40.0], [104.0, 50.0, 36.0], [134.0, 70.0, 48.0], [92.0, 46.0, 34.0], [112.0, 64.0, 50.0]];
        for r in 0..rows {
            let off = (r % 2) as f64 * bw / 2.0;
            for c in -1..cols + 1 {
                let t = tones[(rng() * tones.len() as f64).floor() as usize];
                let k = 1.0 - gf * 0.12 + r2(rng, -0.08, 0.08);
                let x = c as f64 * bw + off;
                ctx.set_fill_style(format!("rgb({},{},{})", to_i32(t[0] * k), to_i32(t[1] * k), to_i32(t[2] * k)));
                for dx in [0.0, -w, w] {
                    ctx.fill_rect(x + dx + m / 2.0, r as f64 * bh + m / 2.0, bw - m, bh - m);
                }
            }
        }
        speckle(rng, ctx, w, h, 7000, [20, 10, 6], 0.05, 0.18, 0.6, 1.8);
        speckle(rng, ctx, w, h, 1500, [220, 190, 170], 0.03, 0.1, 0.6, 1.4);
        for _ in 0..times(6.0 + gf * 10.0) {
            blotch(ctx, w, h, r1(rng, w), r1(rng, h), r2(rng, 30.0, 120.0), &format!("rgba(10,8,6,{})", 0.08 + gf * 0.05));
        }
        for _ in 0..times(3.0 + gf * 4.0) {
            blotch(ctx, w, h, r1(rng, w), r1(rng, h), r2(rng, 20.0, 70.0), &format!("rgba(230,225,210,{})", 0.06 + gf * 0.03));
        }
    })
}

/// Pool tile: 15 cm glazed squares, pale grout that
/// yellows with grime. Level 37.
pub fn tile(rng: Rng, g: i32, floor: bool) -> Tex {
    let gf = g as f64;
    canvas_tex(512, 512, move |ctx, w, h| {
        let grout = ["#cfd4d2", "#c3c6bf", "#aeae9f", "#8f8d7a"][g as usize];
        ctx.set_fill_style(grout);
        ctx.fill_rect(0, 0, w, h);
        let n = 8;
        let (s, m) = (w / n as f64, 3.0);
        for r in 0..n {
            for c in 0..n {
                let (rf, cf) = (r as f64, c as f64);
                let v = 236.0 - gf * 9.0 + r2(rng, -5.0, 5.0);
                ctx.set_fill_style(&if floor {
                    format!("rgb({},{},{})", to_i32(v - 8.0), to_i32(v), to_i32(v + 2.0))
                } else {
                    format!("rgb({},{},{})", to_i32(v), to_i32(v + 3.0), to_i32(v + 2.0))
                });
                ctx.fill_rect(cf * s + m / 2.0, rf * s + m / 2.0, s - m, s - m);
                // glaze highlight
                let gr = ctx.create_linear_gradient(cf * s, rf * s, cf * s + s, rf * s + s);
                gr.add_color_stop(0, "rgba(255,255,255,0.18)");
                gr.add_color_stop(0.5, "rgba(255,255,255,0)");
                gr.add_color_stop(1, "rgba(0,0,0,0.05)");
                ctx.set_fill_style(&gr);
                ctx.fill_rect(cf * s + m / 2.0, rf * s + m / 2.0, s - m, s - m);
            }
        }
        for _ in 0..times(gf * 8.0) {
            blotch(ctx, w, h, r1(rng, w), r1(rng, h), r2(rng, 15.0, 60.0), &format!("rgba(110,100,60,{})", 0.05 + gf * 0.04));
        }
        speckle(rng, ctx, w, h, (800 + g * 1200) as u32, [60, 60, 50], 0.03, 0.1, 0.6, 1.4);
    })
}

// ---------------------------------------------------------------- floors ---

pub fn floor_concrete(rng: Rng, g: i32) -> Tex {
    let gf = g as f64;
    canvas_tex(512, 512, move |ctx, w, h| {
        ctx.set_fill_style(["#8a8680", "#7b776f", "#67645c", "#54514a"][g as usize]);
        ctx.fill_rect(0, 0, w, h);
        for _ in 0..30 {
            blotch(
                ctx,
                w,
                h,
                r1(rng, w),
                r1(rng, h),
                r2(rng, 40.0, 180.0),
                if rng() < 0.55 { "rgba(0,0,0,0.07)" } else { "rgba(255,255,255,0.05)" },
            );
        }
        speckle(rng, ctx, w, h, 12000, [25, 25, 22], 0.04, 0.14, 0.6, 1.6);
        // power-trowel swirls
        ctx.set_stroke_style("rgba(255,255,255,0.035)");
        ctx.set_line_width(1);
        for _ in 0..40 {
            let (x, y, r) = (r1(rng, w), r1(rng, h), r2(rng, 20.0, 70.0));
            ctx.begin_path();
            ctx.arc(x, y, r, r1(rng, 7.0), r1(rng, 7.0));
            ctx.stroke();
        }
        // saw-cut control joint on the 4 m grid (texture border)
        ctx.set_fill_style("rgba(20,18,15,0.55)");
        ctx.fill_rect(0, 0, w, 2);
        ctx.fill_rect(0, 0, 2, h);
        // hairline cracks
        for _ in 0..times(1.0 + gf * 2.0) {
            let mut x = r1(rng, w);
            let mut y = r1(rng, h);
            ctx.set_stroke_style(format!("rgba(15,14,12,{})", 0.35 + gf * 0.1));
            ctx.set_line_width(0.8);
            ctx.begin_path();
            ctx.move_to(x, y);
            for _ in 0..12 {
                x += r2(rng, -18.0, 18.0);
                y += r2(rng, -18.0, 18.0);
                ctx.line_to(x, y);
            }
            ctx.stroke();
        }
        // puddle rings and oil stains
        for _ in 0..times(2.0 + gf * 5.0) {
            blotch(ctx, w, h, r1(rng, w), r1(rng, h), r2(rng, 30.0, 110.0), &format!("rgba(15,15,12,{})", 0.1 + gf * 0.05));
        }
    })
}

/// Commercial carpet tiles: 50 cm squares laid quarter-turn, blue-grey.
pub fn floor_office(rng: Rng, g: i32) -> Tex {
    let gf = g as f64;
    canvas_tex(512, 512, move |ctx, w, h| {
        let n = 8;
        let s = w / n as f64;
        let base: [f64; 3] = [[92.0, 98.0, 108.0], [86.0, 90.0, 96.0], [80.0, 80.0, 78.0], [72.0, 68.0, 60.0]][g as usize];
        for r in 0..n {
            for c in 0..n {
                let (rf, cf) = (r as f64, c as f64);
                let k = (if (r + c) % 2 != 0 { 1.04 } else { 0.96 }) + r2(rng, -0.02, 0.02);
                ctx.set_fill_style(format!("rgb({},{},{})", to_i32(base[0] * k), to_i32(base[1] * k), to_i32(base[2] * k)));
                ctx.fill_rect(cf * s, rf * s, s, s);
                // pile direction stripes, rotated per tile
                ctx.set_fill_style("rgba(255,255,255,0.03)");
                for i in 0..16 {
                    let i = i as f64;
                    if (r + c) % 2 != 0 {
                        ctx.fill_rect(cf * s + i * 4.0, rf * s, 1, s);
                    } else {
                        ctx.fill_rect(cf * s, rf * s + i * 4.0, s, 1);
                    }
                }
            }
        }
        speckle(rng, ctx, w, h, 16000, [20, 22, 26], 0.08, 0.2, 0.6, 1.3);
        speckle(rng, ctx, w, h, 5000, [170, 175, 185], 0.05, 0.12, 0.6, 1.2);
        ctx.set_fill_style("rgba(0,0,0,0.18)");
        for i in 0..=n {
            let i = i as f64;
            ctx.fill_rect(i * s, 0, 1, h);
            ctx.fill_rect(0, i * s, w, 1);
        }
        for _ in 0..times(3.0 + gf * 7.0) {
            blotch(ctx, w, h, r1(rng, w), r1(rng, h), r2(rng, 20.0, 90.0), &format!("rgba(40,30,15,{})", 0.1 + gf * 0.06));
        }
    })
}

// -------------------------------------------------------------- ceilings ---

/// Board-formed concrete slab.
pub fn ceil_slab(rng: Rng, g: i32) -> Tex {
    let gf = g as f64;
    canvas_tex(512, 512, move |ctx, w, h| {
        ctx.set_fill_style(["#8f8c85", "#84817a", "#76736b", "#65625b"][g as usize]);
        ctx.fill_rect(0, 0, w, h);
        for _ in 0..24 {
            blotch(ctx, w, h, r1(rng, w), r1(rng, h), r2(rng, 40.0, 160.0), "rgba(0,0,0,0.08)");
        }
        speckle(rng, ctx, w, h, 9000, [20, 20, 18], 0.05, 0.15, 0.8, 2.2);
        // plywood form marks, 1.2 × 2.4 m sheets
        ctx.set_fill_style("rgba(20,20,18,0.3)");
        let mut x = 0.0;
        while x < w {
            ctx.fill_rect(round_half_up(x), 0, 1, h);
            x += w / (4.0 / 1.2);
        }
        ctx.fill_rect(0, 0, w, 1);
        ctx.fill_rect(0, round_half_up(h * 0.6), w, 1);
        for _ in 0..times(3.0 + gf * 6.0) {
            blotch(ctx, w, h, r1(rng, w), r1(rng, h), r2(rng, 30.0, 110.0), &format!("rgba(40,30,15,{})", 0.1 + gf * 0.06));
        }
    })
}

/// Smooth plaster with water-stain rings.
pub fn ceil_plaster(rng: Rng, g: i32) -> Tex {
    let gf = g as f64;
    canvas_tex(512, 512, move |ctx, w, h| {
        ctx.set_fill_style(["#e9e7de", "#dddace", "#cdc8b6", "#b8b19a"][g as usize]);
        ctx.fill_rect(0, 0, w, h);
        for _ in 0..18 {
            blotch(ctx, w, h, r1(rng, w), r1(rng, h), r2(rng, 60.0, 200.0), "rgba(0,0,0,0.03)");
        }
        speckle(rng, ctx, w, h, 4000, [120, 115, 100], 0.03, 0.08, 0.8, 2.2);
        for _ in 0..times(1.0 + gf * 4.0) {
            let (x, y, r) = (r1(rng, w), r1(rng, h), r2(rng, 30.0, 90.0));
            blotch(ctx, w, h, x, y, r, &format!("rgba(140,110,50,{})", 0.08 + gf * 0.05));
            // one rand(3) per wrapped copy
            wrap_draw(w, h, x, y, r, |xx, yy| {
                ctx.set_stroke_style(format!("rgba(120,90,40,{})", 0.12 + gf * 0.05));
                ctx.set_line_width(1.2);
                ctx.begin_path();
                ctx.ellipse(xx, yy, r * 0.8, r * 0.6, r1(rng, 3.0), 0, 7);
                ctx.stroke();
            });
        }
    })
}

/// Grime level truncated to an integer and clamped to 0..=3
fn clamp_g(g: f64) -> i32 {
    0.max(3.min(to_i32(g)))
}

/// What `make_surfaces` returns: lazily built, cached surface textures.
pub struct Surfaces {
    rng: Rng,
    cache: RefCell<HashMap<String, Tex>>,
}

/// See the module docs. The game passes the
/// native generator (`crate::base::rng::native_rnd`).
pub fn make_surfaces(rng: Rng) -> Surfaces {
    Surfaces { rng, cache: RefCell::new(HashMap::new()) }
}

impl Surfaces {
    /// The cached texture for `key:g`, built on first use.
    fn get(&self, key: &str, g: i32, build: impl FnOnce(Rng, i32) -> Tex) -> Tex {
        let k = format!("{}:{}", key, g);
        if let Some(t) = self.cache.borrow().get(&k) {
            return t.clone();
        }
        let t = build(self.rng, g);
        self.cache.borrow_mut().insert(k, t.clone());
        t
    }
    /// Wall texture at grime `g` — kind: drywall | paint | concrete | brick | tile.
    pub fn wall(&self, kind: &str, g: impl Num) -> Option<Tex> {
        let g = clamp_g(g.f());
        match kind {
            "drywall" => Some(self.get("drywall", g, |r, x| drywall(r, x, true))),
            "paint" => Some(self.get("paint", g, |r, x| drywall(r, x, false))),
            "concrete" => Some(self.get("concrete", g, concrete)),
            "brick" => Some(self.get("brick", g, brick)),
            "tile" => Some(self.get("tile", g, |r, x| tile(r, x, false))),
            _ => None,
        }
    }
    /// Floor texture at grime `g` — kind: concrete | office | tile.
    pub fn floor(&self, kind: &str, g: impl Num) -> Option<Tex> {
        let g = clamp_g(g.f());
        match kind {
            "concrete" => Some(self.get("floorConcrete", g, floor_concrete)),
            "office" => Some(self.get("floorOffice", g, floor_office)),
            "tile" => Some(self.get("floorTile", g, |r, x| tile(r, x, true))),
            _ => None,
        }
    }
    /// Ceiling texture at grime `g` — kind: slab | plaster.
    pub fn ceil(&self, kind: &str, g: impl Num) -> Option<Tex> {
        let g = clamp_g(g.f());
        match kind {
            "slab" => Some(self.get("ceilSlab", g, ceil_slab)),
            "plaster" => Some(self.get("ceilPlaster", g, ceil_plaster)),
            _ => None,
        }
    }
    /// Metres per texture repeat for `kind` (`SURFACE_SCALE`, else [4, 3.1])
    pub fn scale(&self, kind: &str) -> [f64; 2] {
        SURFACE_SCALE.iter().find(|s| s.0 == kind).map(|s| s.1).unwrap_or([4.0, 3.1])
    }
    /// Number of textures built so far.
    pub fn cached(&self) -> usize {
        self.cache.borrow().len()
    }
}
