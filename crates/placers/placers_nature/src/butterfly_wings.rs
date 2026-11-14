//! `place_butterfly_wings`
use crate::shared::blob;
use crate::shared::css;
use game::prelude::*;
use placers_kit::geometry::frame;
use placers_kit::geometry::part;
use placers_kit::noise::hash;
use placers_kit::paint::bleed;
use placers_kit::place::jitter;
use placers_kit::place::pick_open;

/// One of `N` textures of a kind, made the first time it is asked for
macro_rules! cached_kind_tex {
    ($n:expr, $k:expr, $make:expr) => {{
        thread_local! { static T: std::cell::RefCell<Vec<Option<Tex>>> = std::cell::RefCell::new(vec![None; $n]); }
        let k = ($k).min($n - 1);
        T.with(|t| {
            if let Some(x) = &t.borrow()[k] {
                return x.clone();
            }
            let tex = $make(k);
            t.borrow_mut()[k] = Some(tex.clone());
            tex
        })
    }};
}

// Butterfly wing: 5% chance per level. A single wing left on the floor
// (Blender batch `t05`): a peacock's or a small tortoiseshell's, hibernating
// indoors and eaten by a mouse over the winter but for its wings; its scales
// rubbed to the membrane at the edges, its root torn.
pub(crate) fn place_butterfly_wings(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let wing_colors = [0xd08040, 0x4060c0, 0x8040c0, 0xd0c040, 0x804020];
    // the old wing's colour: which wing, and which side up
    let col = choice(&wing_colors);
    let k = wing_colors.iter().position(|&c| c == col).unwrap_or(0);
    let wing = group();
    let name = if k == 2 || k == 3 { "Real__wingHind" } else { "Real__wingFore" };
    if let Some(geo) = part(name) {
        let m = mesh(geo, lambert!({ map: &wing_tex(k), alphaTest: 0.5, side: DoubleSide }));
        // where the old wing's middle was
        m.position().set(0.025, 0, 0.022);
        m.rotation().y = 0.6;
        wing.add(&m);
    }
    let (px, pz) = jitter(g, x, z, 2.4);
    wing.position().set(px, 0, pz);
    wing.rotation().y = rnd() * PI * 2.0;
    g.world.add(&wing);
}

// =================================================================== wing
/// A forewing's outline (mm, x from the root along the costa, z back from
/// the costa), Aglais's: the costa, the apex, the outer margin with its
/// angle, the tornus, the inner margin back to the root (`nature/bits_of_the_outside_indoors.py` WING_FORE:
/// 30.5 x 19 mm)
const FORE: [(f64, f64); 17] = [
    (0.6, 1.6),
    (6.0, 0.4),
    (14.0, 0.0),
    (22.0, 0.3),
    (27.5, 1.0),
    (30.3, 2.3),
    (29.6, 5.4),
    (28.2, 7.9),
    (29.0, 9.6),
    (26.8, 12.6),
    (24.6, 15.6),
    (22.4, 18.7),
    (17.0, 15.8),
    (10.0, 10.4),
    (5.0, 6.4),
    (2.0, 3.8),
    (0.6, 3.0),
];

/// The veins (mm, from, to): the forewing's from its discal cell to the
/// costa, the outer margin and the tornus, and the anal vein
const FORE_VEINS: [((f64, f64), (f64, f64)); 10] = [
    ((1.0, 2.6), (13.0, 3.6)),
    ((1.0, 4.0), (12.0, 7.6)),
    ((8.0, 3.2), (20.0, 0.8)),
    ((11.0, 3.4), (25.0, 1.2)),
    ((13.0, 3.6), (29.0, 2.2)),
    ((13.2, 5.0), (29.6, 5.0)),
    ((13.0, 6.4), (28.4, 8.4)),
    ((12.5, 7.6), (27.2, 11.2)),
    ((10.0, 7.0), (25.4, 14.6)),
    ((1.0, 4.0), (21.6, 18.2)),
];

/// The hindwing's veins (mm)
const HIND_VEINS: [((f64, f64), (f64, f64)); 10] = [
    ((1.0, 3.8), (11.0, 6.0)),
    ((1.0, 6.0), (11.6, 10.0)),
    ((1.0, 3.2), (16.0, 0.8)),
    ((11.0, 6.0), (23.0, 4.5)),
    ((11.4, 7.6), (24.0, 8.5)),
    ((11.6, 9.4), (23.0, 12.5)),
    ((11.4, 10.0), (21.5, 15.5)),
    ((9.0, 9.0), (18.0, 19.2)),
    ((6.0, 8.0), (13.5, 21.0)),
    ((1.0, 6.0), (8.0, 20.5)),
];

/// A hindwing's outline (mm, x from the root out, z back from its costa;
/// 24.5 x 21.5 mm)
const HIND: [(f64, f64); 15] = [
    (0.6, 2.2),
    (8.0, 0.3),
    (16.0, 0.6),
    (21.6, 2.6),
    (24.2, 6.0),
    (23.6, 10.0),
    (22.4, 13.6),
    (21.0, 16.2),
    (17.4, 19.6),
    (13.0, 21.3),
    (9.0, 21.0),
    (5.0, 16.0),
    (2.0, 10.0),
    (0.6, 5.0),
    (0.4, 3.4),
];

/// The five wings the old colour chose: a small tortoiseshell's forewing, a
/// peacock's forewing, a peacock's hindwing, a small tortoiseshell's
/// hindwing (all their upper sides), a peacock's forewing from beneath
fn wing_tex(k: usize) -> Tex {
    cached_kind_tex!(5, k, |k: usize| {
        let hind = k == 2 || k == 3;
        let (name, size, outline): (&str, (f64, f64), &[(f64, f64)]) =
            if hind { ("Real__wingHind", (24.5, 21.5), &HIND) } else { ("Real__wingFore", (30.5, 19.0), &FORE) };
        let (lo, hi) = frame(name);
        let th = (256.0 * (hi[2] - lo[2]) / (hi[0] - lo[0])).round();
        let tex = canvas_tex_opt(256, th, false, move |ctx, w, h| {
            let (sx, sy) = (w / size.0, h / size.1);
            let p = |x: f64, z: f64| (x * sx, z * sy);
            let path = |ctx: &mut Canvas| {
                ctx.begin_path();
                for (i, &(x, z)) in outline.iter().enumerate() {
                    let (a, b) = p(x, z);
                    if i == 0 {
                        ctx.move_to(a, b);
                    } else {
                        ctx.line_to(a, b);
                    }
                }
                ctx.close_path();
            };
            let fill_poly = |ctx: &mut Canvas, pts: &[(f64, f64)], c: [f64; 3], a: f64| {
                ctx.set_fill_style(css(c, a));
                ctx.begin_path();
                for (i, &(x, z)) in pts.iter().enumerate() {
                    let (a, b) = p(x, z);
                    if i == 0 {
                        ctx.move_to(a, b);
                    } else {
                        ctx.line_to(a, b);
                    }
                }
                ctx.close_path();
                ctx.fill();
            };
            let spot = |ctx: &mut Canvas, x: f64, z: f64, r: f64, k2: f64, c: [f64; 3], a: f64| {
                let (cx, cy) = p(x, z);
                blob(ctx, cx, cy, r * sx, k2, 0.3, c, a);
            };
            // a scale-covered disc: crisp at the edge, as the eyespots' rings are
            let disc = |ctx: &mut Canvas, x: f64, z: f64, r: f64, c: [f64; 3], a: f64| {
                let (cx, cy) = p(x, z);
                ctx.set_fill_style(css(c, a));
                ctx.begin_path();
                ctx.ellipse(cx, cy, r * sx, r * sy * 0.92, 0.2, 0, 7);
                ctx.fill();
            };
            ctx.save();
            path(ctx);
            ctx.clip();
            let (rust, black, cream, blue) = ([0.55, 0.21, 0.15], [0.10, 0.08, 0.07], [0.80, 0.72, 0.50], [0.38, 0.44, 0.58]);
            match k {
                // small tortoiseshell, forewing
                0 => {
                    let orange = [0.74, 0.42, 0.17];
                    ctx.set_fill_style(css(orange, 1.0));
                    ctx.fill_rect(0, 0, w, h);
                    spot(ctx, 0.0, 6.0, 9.0, 1.0, [0.22, 0.14, 0.09], 0.95);
                    // the costa's blocks: black, yellow, black, yellow, black
                    for &(x0, x1, c) in &[
                        (5.0, 9.0, black),
                        (9.0, 12.5, [0.82, 0.70, 0.30]),
                        (12.5, 17.0, black),
                        (17.0, 20.5, [0.82, 0.70, 0.30]),
                        (20.5, 26.0, black),
                    ] {
                        fill_poly(ctx, &[(x0, -1.0), (x1, -1.0), (x1 - 0.8, 5.5), (x0 - 0.4, 5.0)], c, 1.0);
                    }
                    spot(ctx, 27.2, 2.6, 1.3, 1.0, [0.86, 0.84, 0.78], 0.95);
                    for &(x, z, r) in &[(9.5, 9.0, 1.9), (12.5, 12.0, 1.5), (14.8, 16.4, 2.4)] {
                        spot(ctx, x, z, r, 1.0, black, 1.0);
                        spot(ctx, x, z, r * 0.7, 1.0, black, 1.0);
                    }
                }
                // peacock, forewing
                1 => {
                    ctx.set_fill_style(css(rust, 1.0));
                    ctx.fill_rect(0, 0, w, h);
                    spot(ctx, 0.0, 7.0, 10.0, 1.0, [0.24, 0.12, 0.09], 0.95);
                    fill_poly(ctx, &[(4.0, -1.0), (22.0, -1.0), (21.0, 2.2), (4.0, 1.6)], black, 0.95);
                    fill_poly(ctx, &[(13.5, -1.0), (16.0, -1.0), (15.5, 3.0), (13.2, 2.6)], cream, 0.9);
                    // the eyespot: black patches round it, cream on its inner
                    // side, rust and black within, blue-violet scales on it
                    disc(ctx, 25.6, 5.4, 4.6, black, 0.9);
                    disc(ctx, 24.6, 6.8, 4.0, black, 0.9);
                    spot(ctx, 22.8, 4.6, 3.6, 0.8, cream, 0.95);
                    spot(ctx, 23.6, 7.6, 2.2, 1.0, cream, 0.8);
                    spot(ctx, 25.4, 6.0, 2.6, 0.9, [0.58, 0.28, 0.20], 0.9);
                    disc(ctx, 25.8, 6.2, 1.6, black, 0.9);
                    for i in 0..9 {
                        let (a, d) = (hash(0x3a5, i, 0) * 6.3, 0.8 + 2.4 * hash(0x3a5, i, 1));
                        spot(ctx, 25.8 + a.cos() * d, 6.2 + a.sin() * d * 0.8, 0.8 + 0.6 * hash(0x3a5, i, 2), 1.0, blue, 0.7);
                    }
                    spot(ctx, 28.4, 2.4, 0.7, 1.0, [0.86, 0.84, 0.80], 0.9);
                }
                // peacock, hindwing
                2 => {
                    ctx.set_fill_style(css(rust, 1.0));
                    ctx.fill_rect(0, 0, w, h);
                    // its base dark brown, hairy
                    disc(ctx, -2.0, 12.0, 10.5, [0.20, 0.12, 0.09], 0.9);
                    spot(ctx, 0.0, 11.0, 17.0, 1.0, [0.18, 0.11, 0.08], 0.9);
                    // the eyespot toward the front of the wing: a pale ring, black,
                    // the blue-grey scales scattered in it, its black heart
                    disc(ctx, 16.8, 7.4, 5.6, [0.56, 0.52, 0.46], 0.9);
                    disc(ctx, 16.8, 7.6, 4.6, black, 0.97);
                    // the blue-grey scales: a broken ring of them, thickest toward the wing's edge
                    let (cx, cy) = p(17.0, 7.7);
                    ctx.set_stroke_style(css([0.42, 0.48, 0.58], 0.55));
                    ctx.set_line_width(1.3 * sx);
                    ctx.begin_path();
                    ctx.arc(cx, cy, 2.9 * sx, -0.6, 3.6);
                    ctx.stroke();
                    for i in 0..14 {
                        let a = -0.6 + 4.2 * hash(0x3a1, i, 0);
                        let d = 2.3 + 1.4 * hash(0x3a1, i, 1);
                        spot(ctx, 17.0 + a.cos() * d, 7.7 + a.sin() * d, 0.7, 1.0, [0.50, 0.56, 0.66], 0.6);
                    }
                    disc(ctx, 17.0, 7.6, 1.4, black, 0.95);
                }
                // small tortoiseshell, hindwing
                3 => {
                    ctx.set_fill_style(css([0.74, 0.42, 0.17], 1.0));
                    ctx.fill_rect(0, 0, w, h);
                    spot(ctx, 0.0, 6.0, 16.0, 1.0, [0.14, 0.10, 0.08], 1.0);
                    spot(ctx, 3.0, 9.0, 12.0, 1.0, [0.14, 0.10, 0.08], 1.0);
                }
                // peacock, forewing from beneath: near black, finely lined like bark
                _ => {
                    ctx.set_fill_style(css([0.17, 0.14, 0.11], 1.0));
                    ctx.fill_rect(0, 0, w, h);
                    ctx.set_line_width(0.7);
                    for i in 0..90 {
                        let x0 = rnd() * size.0;
                        let z0 = rnd() * size.1;
                        let c = if i % 3 == 0 { [0.30, 0.25, 0.20] } else { [0.08, 0.07, 0.06] };
                        ctx.set_stroke_style(css(c, 0.5));
                        ctx.begin_path();
                        let (a, b) = p(x0, z0);
                        ctx.move_to(a, b);
                        let (c2, d2) = p(x0 + 0.8, z0 + 2.0 + rnd());
                        let (e2, f2) = p(x0 - 0.5, z0 + 4.0);
                        ctx.quadratic_curve_to(c2, d2, e2, f2);
                        ctx.stroke();
                    }
                    spot(ctx, 16.0, 9.0, 7.0, 0.6, [0.26, 0.21, 0.16], 0.6);
                }
            }
            // the dark border along the outer margin, and the blue lunules in it
            ctx.set_line_join("round");
            let n = outline.len();
            let margin: Vec<(f64, f64)> = if hind { outline[3..11].to_vec() } else { outline[5..12].to_vec() };
            ctx.set_stroke_style(css(if k == 4 { [0.32, 0.27, 0.22] } else { [0.16, 0.11, 0.08] }, 0.95));
            ctx.set_line_width(2.2 * sx);
            ctx.begin_path();
            for (i, &(x, z)) in margin.iter().enumerate() {
                let (a, b) = p(x, z);
                if i == 0 {
                    ctx.move_to(a, b);
                } else {
                    ctx.line_to(a, b);
                }
            }
            ctx.stroke();
            if k == 0 || k == 3 {
                for win in margin.windows(2) {
                    let (mx, mz) = ((win[0].0 + win[1].0) / 2.0, (win[0].1 + win[1].1) / 2.0);
                    let (cx, cz) = (size.0 * 0.45, size.1 * 0.45);
                    let d = (mx - cx).hypot(mz - cz).max(1e-6);
                    spot(ctx, mx - (mx - cx) / d * 1.0, mz - (mz - cz) / d * 1.0, 0.7, 0.6, [0.40, 0.48, 0.62], 0.9);
                }
            }
            // the veins: the discal cell's edges and the veins out of it,
            // darker lines a little bowed
            ctx.set_line_width(0.3 * sx);
            ctx.set_stroke_style(css([0.12, 0.09, 0.07], 0.3));
            for &((x0, z0), (x1, z1)) in if hind { &HIND_VEINS } else { &FORE_VEINS } {
                let (a, b) = p(x0, z0);
                let (c2, d2) = p((x0 + x1) / 2.0 + 0.3, (z0 + z1) / 2.0 - 0.5);
                let (e2, f2) = p(x1, z1);
                ctx.begin_path();
                ctx.move_to(a, b);
                ctx.quadratic_curve_to(c2, d2, e2, f2);
                ctx.stroke();
            }
            // worn: scales rubbed off to the grey membrane at the edges and in patches
            for _ in 0..14 {
                let (i, t) = ((rnd() * n as f64) as usize % n, rnd());
                let (a, b) = (outline[i], outline[(i + 1) % n]);
                let (x, z) = (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t);
                spot(ctx, x, z, 1.2 + 2.0 * rnd(), 0.6 + 0.4 * rnd(), [0.52, 0.48, 0.42], 0.35 + 0.3 * rnd());
            }
            for _ in 0..4 {
                spot(ctx, size.0 * (0.25 + 0.6 * rnd()), size.1 * (0.2 + 0.6 * rnd()), 1.0 + 2.0 * rnd(), 0.7, [0.50, 0.46, 0.40], 0.25);
            }
            // dust on it
            for _ in 0..120 {
                let (x, y) = (rnd() * w, rnd() * h);
                ctx.set_fill_style(css([0.55, 0.52, 0.46], 0.15 + 0.2 * rnd()));
                ctx.fill_rect(x, y, 1.0, 1.0);
            }
            ctx.restore();
            // torn: bites out of the outer margin, the root torn away ragged
            ctx.set_global_composite_operation("destination-out");
            for _ in 0..5 {
                let (i, t) = ((rnd() * n as f64) as usize % n, rnd());
                let (a, b) = (outline[i], outline[(i + 1) % n]);
                let (x, z) = (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t);
                if x < 4.0 {
                    continue;
                }
                let (cx, cy) = p(x, z);
                ctx.set_fill_style("rgba(0,0,0,1)");
                ctx.begin_path();
                ctx.arc(cx, cy, (0.5 + 1.1 * rnd()) * sx, 0, 7);
                ctx.fill();
            }
            ctx.begin_path();
            let (r0x, r0y) = p(0.0, 0.0);
            ctx.move_to(r0x, r0y);
            for i in 0..=8 {
                let z = size.1 * 0.45 * i as f64 / 8.0;
                let x = 1.6 + 1.6 * rnd();
                let (a, b) = p(x, z);
                ctx.line_to(a, b);
            }
            let (a, b) = p(0.0, size.1 * 0.5);
            ctx.line_to(a, b);
            ctx.close_path();
            ctx.set_fill_style("rgba(0,0,0,1)");
            ctx.fill();
            ctx.set_global_composite_operation("source-over");
        });
        bleed(&tex, [0.30, 0.22, 0.17]);
        tex
    })
}
