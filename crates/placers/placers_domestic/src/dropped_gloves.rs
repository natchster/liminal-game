//! `place_dropped_gloves`
use crate::shared::add;
use crate::shared::css;
use crate::shared::leather_tex;
use crate::shared::mix;
use crate::shared::paint_leather;
use crate::shared::rgb;
use crate::shared::soft;
use crate::shared::wander;
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Single dropped glove (floor): 8% chance per level. One glove dropped
// and left, lying limp (Blender batch r14a, the glove of
// `place_single_gloves` too): a lined leather dress glove, its three points
// stitched on the back, or a knitted one; 40% with a stripe knitted round
// the wrist.
pub(crate) fn place_dropped_gloves(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    }
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = open_tiles[(rnd() * open_tiles.len() as f64).floor() as usize];
    // the old colours stand for the gloves people drop: black, brown and
    // dark brown leather, maroon and charcoal knits, camel leather, an
    // oatmeal knit
    let glove_colors = [0x101010, 0x603020, 0x201010, 0xa02040, 0x2a2a2a, 0xd0a060, 0xf0e8d0];
    let col = pick(&glove_colors);
    // the old fingers' tilts (two draws each) now vary how it is worn
    let mut vary = 0u32;
    for _ in 0..4 {
        let a = rnd();
        let b = rnd();
        vary = vary.rotate_left(7) ^ ((a * 65535.0) as u32) ^ (((b * 65535.0) as u32) << 16);
    }
    // 40%: a stripe knitted round the wrist
    let stripe = if rnd() < 0.4 {
        let trim_colors = [0xffffff, 0xd0a040, 0xa02040, 0x2060a0];
        Some(match pick(&trim_colors) {
            0xffffff => 0xd2ccbe,
            0xd0a040 => 0xa88a48,
            0xa02040 => 0x7e3036,
            _ => 0x3a5274,
        })
    } else {
        None
    };
    let (px, pz) = (g.wx(x) + (rnd() - 0.5) * 2.4, g.wz(z) + (rnd() - 0.5) * 2.4);
    let yaw = rnd() * PI * 2.0;
    let _ = rnd(); // keeps the old tilt's draw: the glove lies flat
    let (cloth, base) = match col {
        0x101010 => (Cloth::Leather, 0x2c2a28),
        0x603020 => (Cloth::Leather, 0x5a3c2a),
        0x201010 => (Cloth::Leather, 0x3a2622),
        0xa02040 => (Cloth::Knit, 0x6a2c34),
        0x2a2a2a => (Cloth::Knit, 0x403e3c),
        0xd0a060 => (Cloth::Leather, 0xa0825c),
        _ => (Cloth::Knit, 0xb4ab9a),
    };
    let leather = cloth == Cloth::Leather;
    let (r, gg, b) = rgb(base);
    let darker = (((r * 0.92) as u32) << 16) | (((gg * 0.92) as u32) << 8) | ((b * 0.92) as u32);
    let spec = Glove {
        back: (cloth, base),
        tips: None,
        // a leather glove's wrist hemmed, or (with a stripe) a knitted wristlet
        cuff: (if leather && stripe.is_none() { Cloth::Leather } else { Cloth::Knit }, darker),
        stripe,
        under: darker,
        points: leather,
    };
    let grp = glove(&spec, vary);
    grp.position().set(px, 0, pz);
    grp.rotation().y = yaw;
    g.world.add(&grp);
}

/// What a glove is made of, piece by piece.
#[derive(Clone, Copy, PartialEq)]
enum Cloth {
    Leather,
    Knit,
    Canvas,
    Jersey,
}

/// A glove: its back's cloth and colour (sRGB), the leather tips, knuckle
/// strap and thumb of a leather palm work glove, its wrist's cloth and
/// colour, a stripe knitted round the wrist, the palm's colour, and a dress
/// glove's three stitched points.
struct Glove {
    back: (Cloth, u32),
    tips: Option<u32>,
    cuff: (Cloth, u32),
    stripe: Option<u32>,
    under: u32,
    points: bool,
}

/// The glove of batch r14a: a right one lying back up, as modelled.
fn glove(s: &Glove, vary: u32) -> Obj {
    let grp = group();
    let back = lambert!({ color: 0xffffff, map: &glove_back_tex(s.back.0, s.back.1, s.points, vary) });
    add(&grp, "Real__gloveBack", &back);
    add(&grp, "Real__gloveUnder", &lambert!({ color: s.under }));
    let cuff = lambert!({ color: 0xffffff, map: &glove_cuff_tex(s.cuff.0, s.cuff.1, s.stripe, vary ^ 0x5bd1) });
    add(&grp, "Real__gloveCuff", &cuff);
    if let Some(t) = s.tips {
        let tips = lambert!({ color: 0xffffff, map: &leather_tex(t, 128, vary ^ 0x2c1b) });
        add(&grp, "Real__gloveTips", &tips);
    }
    grp
}

/// The glove's back as modelled (domestic/things_worn_and_dropped.py): its bounds in x and z (the top's
/// u runs with x, v against z), and each digit: base (x, z), length, flat
/// width, angle from +z toward +x, bow.
const GLOVE_XZ: (f64, f64, f64, f64) = (-0.0757, 0.0877, 0.0, 0.1955);

const DIGITS: [(f64, f64, f64, f64, f64, f64); 5] = [
    (0.0335, 0.094, 0.079, 0.0232, 0.13, 0.006),
    (0.0110, 0.098, 0.087, 0.0238, 0.03, -0.003),
    (-0.0120, 0.096, 0.080, 0.0228, -0.08, -0.007),
    (-0.0335, 0.088, 0.064, 0.0205, -0.23, -0.010),
    (0.045, 0.030, 0.068, 0.0265, 0.72, -0.0085),
];

/// A digit's axis at t (0 its base, 1 its tip): the point and its direction.
fn digit_at(d: &(f64, f64, f64, f64, f64, f64), t: f64) -> ((f64, f64), (f64, f64)) {
    let (bx, bz, l, _, ang, bow) = *d;
    let am = ang + bow / l * t;
    let a = ang + bow / l * 2.0 * t;
    ((bx + am.sin() * l * t, bz + am.cos() * l * t), (a.sin(), a.cos()))
}

/// A cloth's ground in colour `base`: knit wales, a canvas weave, a jersey's
/// fine knit, or leather; faded unevenly.
fn paint_cloth(ctx: &mut Canvas, w: f64, h: f64, cloth: Cloth, base: u32) {
    if cloth == Cloth::Leather {
        paint_leather(ctx, w, h, base);
        return;
    }
    let c = rgb(base);
    ctx.set_fill_style(&css(c, 1.0));
    ctx.fill_rect(0, 0, w, h);
    let dark = (c.0 * 0.66, c.1 * 0.64, c.2 * 0.62);
    let pale = mix(c, (200.0, 196.0, 186.0), 0.35);
    match cloth {
        Cloth::Canvas => {
            // the plain weave: warp and weft every 2 px, each thread uneven
            for i in (0..w as i32).step_by(2) {
                ctx.set_fill_style(&css(dark, 0.10 + rnd() * 0.10));
                ctx.fill_rect(i as f64, 0, 1, h);
            }
            for j in (0..h as i32).step_by(2) {
                ctx.set_fill_style(&css(pale, 0.08 + rnd() * 0.10));
                ctx.fill_rect(0, j as f64, w, 1);
            }
        }
        _ => {
            // knitted: columns of V stitches (a jersey's finer)
            let step = if cloth == Cloth::Knit { 4.0 } else { 3.0 };
            ctx.set_stroke_style(&css(dark, 0.22));
            ctx.set_line_width(0.8);
            let mut y = 0.0;
            while y < h {
                // a row of Vs, one path
                ctx.begin_path();
                let mut x = 0.0;
                ctx.move_to(x, y);
                while x < w {
                    ctx.line_to(x + step / 2.0, y + step * 0.8);
                    ctx.line_to(x + step, y);
                    x += step;
                }
                ctx.stroke();
                y += step * 0.8;
            }
            // fuzz and pills
            for _ in 0..500 {
                wander(ctx, (rnd() * w, rnd() * h), 2.0 + rnd() * 4.0, rnd() * 6.3, pale, 0.25, 0.5);
            }
            for _ in 0..90 {
                ctx.set_fill_style(&css(pale, 0.4));
                ctx.fill_rect(rnd() * w, rnd() * h, 1.5, 1.5);
            }
        }
    }
    // faded and soiled unevenly
    for _ in 0..30 {
        let (col, a) = if rnd() < 0.5 { ((c.0 * 0.6, c.1 * 0.56, c.2 * 0.5), 0.10 + rnd() * 0.12) } else { (pale, 0.08 + rnd() * 0.1) };
        soft(ctx, (rnd() * w, rnd() * h), 10.0 + rnd() * 34.0, 0.6 + rnd() * 0.4, rnd() * 3.0, col, a);
    }
}

/// The back of the glove: its cloth; the fingers' joints creased across, the
/// knuckles and fingertips rubbed and soiled where they took the work; a
/// dress glove's three stitched points; the dirt of the floor.
fn glove_back_tex(cloth: Cloth, base: u32, points: bool, vary: u32) -> Tex {
    canvas_tex(256, 256, move |ctx, w, h| {
        let seed = (rnd() * 4.0e9) as u32 ^ vary;
        with_own_stream(seed, || {
            paint_cloth(ctx, w, h, cloth, base);
            let (x0, x1, z0, z1) = GLOVE_XZ;
            let px = |x: f64, z: f64| ((x - x0) / (x1 - x0) * w, (z - z0) / (z1 - z0) * h);
            let c = rgb(base);
            let crease = (c.0 * 0.42, c.1 * 0.38, c.2 * 0.35);
            let dirt = (70.0, 62.0, 52.0);
            for (k, d) in DIGITS.iter().enumerate() {
                let hw = d.3 / 2.0;
                let joints: &[f64] = if k == 4 { &[0.45] } else { &[0.30, 0.62] };
                for &tj in joints {
                    for dt in [-0.03, 0.0, 0.03] {
                        let ((x, z), (dx, dz)) = digit_at(d, tj + dt);
                        let (nx, nz) = (-dz, dx);
                        let a = px(x - nx * hw, z - nz * hw);
                        let m = px(x + dx * 0.002, z + dz * 0.002);
                        let b = px(x + nx * hw, z + nz * hw);
                        ctx.set_stroke_style(&css(crease, if dt == 0.0 { 0.45 } else { 0.22 }));
                        ctx.set_line_width(1.1);
                        ctx.begin_path();
                        ctx.move_to(a.0, a.1);
                        ctx.quadratic_curve_to(m.0, m.1, b.0, b.1);
                        ctx.stroke();
                    }
                }
                // the tip soiled and rubbed
                let ((x, z), _) = digit_at(d, 0.93);
                let p = px(x, z);
                soft(ctx, p, 9.0, 1.0, 0.0, dirt, 0.35);
                if cloth == Cloth::Leather {
                    soft(ctx, p, 5.0, 0.7, rnd(), mix(c, (200.0, 190.0, 172.0), 0.5), 0.3);
                }
                // the knuckle
                if k < 4 {
                    let ((x, z), _) = digit_at(d, -0.05);
                    soft(ctx, px(x, z), 8.0, 0.6, 0.0, mix(c, (196.0, 188.0, 170.0), 0.5), 0.22);
                }
            }
            if points {
                // three lines of stitching from between the fingers' bases
                // toward the wrist, a little converging
                for (xa, xb) in [(0.022, 0.016), (-0.0005, -0.0005), (-0.023, -0.017)] {
                    let (a, b) = (px(xa, 0.088), px(xb, 0.036));
                    ctx.set_stroke_style(&css(crease, 0.6));
                    ctx.set_line_width(0.9);
                    ctx.set_line_dash(&[2.0, 1.6]);
                    for off in [-1.4, 1.4] {
                        ctx.begin_path();
                        ctx.move_to(a.0 + off, a.1);
                        ctx.line_to(b.0 + off, b.1);
                        ctx.stroke();
                    }
                    ctx.set_line_dash(&[]);
                }
            }
            // dirt of the floor, and where it was gripped
            for _ in 0..20 {
                soft(ctx, (rnd() * w, rnd() * h), 6.0 + rnd() * 22.0, 0.6 + rnd() * 0.4, rnd() * 3.0, dirt, 0.08 + rnd() * 0.12);
            }
        })
    })
}

/// The wrist: a ribbed knit (its ribs along the arm, its mouth at the
/// canvas' top), a stripe knitted round it, or the leather hemmed.
fn glove_cuff_tex(cloth: Cloth, base: u32, stripe: Option<u32>, vary: u32) -> Tex {
    canvas_tex(128, 128, move |ctx, w, h| {
        let seed = (rnd() * 4.0e9) as u32 ^ vary;
        with_own_stream(seed, || {
            let c = rgb(base);
            if cloth == Cloth::Leather {
                paint_leather(ctx, w, h, base);
                // the hem's stitching by the mouth
                ctx.set_stroke_style(&css((c.0 * 0.45, c.1 * 0.4, c.2 * 0.38), 0.6));
                ctx.set_line_width(0.8);
                ctx.set_line_dash(&[2.0, 1.5]);
                ctx.begin_path();
                ctx.move_to(0, h * 0.10);
                ctx.line_to(w, h * 0.10);
                ctx.stroke();
                ctx.set_line_dash(&[]);
                return;
            }
            ctx.set_fill_style(&css(c, 1.0));
            ctx.fill_rect(0, 0, w, h);
            // the stripes, across it (the cuff runs z -0.066 at its mouth to 0)
            if let Some(sc) = stripe {
                let s = rgb(sc);
                for (a, b) in [(0.18, 0.26), (0.36, 0.44)] {
                    ctx.set_fill_style(&css(s, 1.0));
                    ctx.fill_rect(0, h * a, w, h * (b - a));
                }
            }
            // the ribs: a ridge and a valley every 5 px
            let mut x = 0.0;
            while x < w {
                ctx.set_fill_style("rgba(255,250,240,0.16)");
                ctx.fill_rect(x, 0, 2.0, h);
                ctx.set_fill_style("rgba(20,16,12,0.22)");
                ctx.fill_rect(x + 3.0, 0, 1.5, h);
                x += 5.0;
            }
            for _ in 0..200 {
                wander(ctx, (rnd() * w, rnd() * h), 2.0 + rnd() * 3.0, rnd() * 6.3, mix(c, (210.0, 204.0, 192.0), 0.4), 0.22, 0.5);
            }
            // grubby where it was pulled on, the mouth stretched and darker
            soft(ctx, (w * 0.5, 0.0), w * 0.7, 0.18, 0.0, (60.0, 54.0, 46.0), 0.35);
            for _ in 0..10 {
                soft(ctx, (rnd() * w, rnd() * h), 8.0 + rnd() * 16.0, 0.6, rnd() * 3.0, (70.0, 62.0, 52.0), 0.10 + rnd() * 0.1);
            }
        })
    })
}
