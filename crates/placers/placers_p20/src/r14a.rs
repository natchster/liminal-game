//! Batch `r14a`'s props in this crate, made real (`docs/PROPS.md` P2): its
//! placers and what only they use.
use super::*;

// Single glove: 8% chance per level. One work glove lying limp on the
// floor, back up (Blender batch r14a, the glove of `place_dropped_gloves`
// too): a leather palm glove (canvas back, leather tips, knuckle strap and
// thumb, a knitted wrist), an all-leather one, or a cotton jersey one.
pub(super) fn place_single_gloves(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = choice(&open);
    // the old colours stand for the work gloves people wear out: a brown
    // split-leather one, a grey cotton jersey one gone dark, a leather palm
    // one with a red canvas back and a striped wrist, one with a blue-grey
    // back and a blue wrist, a tan cowhide driver
    let glove_colors = [0x6a3820, 0x2a2418, 0x8a3020, 0x304050, 0xa08050];
    let col = choice(&glove_colors);
    let spec = match col {
        0x6a3820 => R14aGlove {
            back: (R14aCloth::Leather, 0x6a4a34),
            tips: None,
            cuff: (R14aCloth::Leather, 0x654531),
            stripe: None,
            under: 0x5e4230,
            points: false,
        },
        0x2a2418 => R14aGlove {
            back: (R14aCloth::Jersey, 0x5c5852),
            tips: None,
            cuff: (R14aCloth::Knit, 0x55524c),
            stripe: None,
            under: 0x4e4a45,
            points: false,
        },
        0x8a3020 => R14aGlove {
            back: (R14aCloth::Canvas, 0x7a4234),
            tips: Some(0x8e6e50),
            cuff: (R14aCloth::Knit, 0x6e3a32),
            stripe: Some(0xb0a690),
            under: 0x86684c,
            points: false,
        },
        0x304050 => R14aGlove {
            back: (R14aCloth::Canvas, 0x4c5660),
            tips: Some(0x8a6c4e),
            cuff: (R14aCloth::Knit, 0x3a4658),
            stripe: None,
            under: 0x806448,
            points: false,
        },
        _ => R14aGlove {
            back: (R14aCloth::Leather, 0x9a7c58),
            tips: None,
            cuff: (R14aCloth::Leather, 0x947654),
            stripe: None,
            under: 0x8c7050,
            points: false,
        },
    };
    let glove = group();
    scatter(g, &glove, x, z, 2.4);
    let p = glove.pos();
    let vary = ((p.x * 7919.0).abs() as u32) ^ ((p.z * 104_729.0).abs() as u32).rotate_left(13);
    glove.add(&r14a_glove(&spec, vary));
    g.world.add(&glove);
}

// Single worn boot: 8% chance per level. A 6 in lace-up work boot left on
// the floor, standing or fallen on its side (Blender batch r14a: the shaft
// and padded collar, the quarters lapped over the vamp, the lug sole, the
// lace undone from its hooks).
pub(super) fn place_single_boots(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = choice(&open);
    // the old colours stand for a work boot's leathers: black, dark brown
    // oiled, wheat nubuck faded, oxblood; (leather, lining, sole, laces, metal)
    let boot_colors = [0x201a10, 0x503020, 0x6a3820, 0x2a1810];
    let cols = match choice(&boot_colors) {
        0x201a10 => (0x2e2b28, 0x2a2522, 0x262422, 0x2a2724, 0x3c3832),
        0x503020 => (0x4e3626, 0x3a2c22, 0x2a2826, 0x7a6448, 0x4a4236),
        0x6a3820 => (0x8a6a48, 0x5a4836, 0x3a332c, 0x8c7a5c, 0x5c5040),
        _ => (0x46282a, 0x34262a, 0x262422, 0x3a2c24, 0x3c3832),
    };
    // 25% fallen over: as it lies on its outer side (modelled so)
    let tipped = rnd() < 0.25;
    let boot = group();
    scatter_xz(g, &boot, x, z, 2.4);
    let p = boot.pos();
    let vary = ((p.x * 7919.0).abs() as u32) ^ ((p.z * 104_729.0).abs() as u32).rotate_left(11);
    boot.add(&r14a_footwear("boot", if tipped { "Side" } else { "" }, cols, vary));
    g.world.add(&boot);
}

/// The part `name` of a Blender batch, as modelled, added to `grp` in `mat`.
fn r14a_add(grp: &Obj, name: &str, mat: &Mat) {
    if let Some(geo) = game::assets::props::prop_part_as_modelled(name) {
        grp.add(&mesh(geo, mat.clone()));
    }
}

fn r14a_rgb(c: u32) -> (f64, f64, f64) {
    (((c >> 16) & 255) as f64, ((c >> 8) & 255) as f64, (c & 255) as f64)
}

fn r14a_css(c: (f64, f64, f64), a: f64) -> String {
    let k = |v: f64| v.clamp(0.0, 255.0).round();
    format!("rgba({},{},{},{:.3})", k(c.0), k(c.1), k(c.2), a.clamp(0.0, 1.0))
}

fn r14a_mix(a: (f64, f64, f64), b: (f64, f64, f64), t: f64) -> (f64, f64, f64) {
    (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t, a.2 + (b.2 - a.2) * t)
}

/// A soft spot at `at`, `r` px, `k` times as tall as wide, turned `turn`,
/// fading out from its middle (the context squashed, so no edge cuts it).
fn r14a_soft(ctx: &mut Canvas, at: (f64, f64), r: f64, k: f64, turn: f64, c: (f64, f64, f64), a: f64) {
    ctx.save();
    ctx.translate(at.0, at.1);
    ctx.rotate(turn);
    ctx.scale(1, k);
    let gr = ctx.create_radial_gradient(0, 0, 0, 0, 0, r);
    gr.add_color_stop(0, &r14a_css(c, a));
    gr.add_color_stop(1, &r14a_css(c, 0.0));
    ctx.set_fill_style(&gr);
    ctx.begin_path();
    ctx.arc(0, 0, r, 0, 7);
    ctx.fill();
    ctx.restore();
}

/// A short wandering line from `at`, `len` px long at angle `ang`.
fn r14a_wander(ctx: &mut Canvas, at: (f64, f64), len: f64, ang: f64, c: (f64, f64, f64), a: f64, lw: f64) {
    ctx.set_stroke_style(&r14a_css(c, a));
    ctx.set_line_width(lw);
    ctx.begin_path();
    ctx.move_to(at.0, at.1);
    let (mut x, mut y, mut t) = (at.0, at.1, ang);
    for _ in 0..5 {
        t += (rnd() - 0.5) * 0.5;
        x += t.cos() * len / 5.0;
        y += t.sin() * len / 5.0;
        ctx.line_to(x, y);
    }
    ctx.stroke();
}

/// Leather after years, in its colour `base` (sRGB): mottled where it was
/// oiled and dried, its grain, creased, rubbed pale and scuffed (most low
/// on it, v < 0.35: the welt's edge all round, the toe's foot), a film of
/// dust. Painted on a stream of its own varied by `vary`.
fn r14a_leather_tex(base: u32, size: u32, vary: u32) -> Tex {
    canvas_tex(size, size, move |ctx, w, h| {
        let seed = (rnd() * 4.0e9) as u32 ^ vary;
        with_own_stream(seed, || r14a_paint_leather(ctx, w, h, base))
    })
}

fn r14a_paint_leather(ctx: &mut Canvas, w: f64, h: f64, base: u32) {
    let c = r14a_rgb(base);
    let s = w / 256.0;
    ctx.set_fill_style(&r14a_css(c, 1.0));
    ctx.fill_rect(0, 0, w, h);
    let dark = (c.0 * 0.62, c.1 * 0.58, c.2 * 0.55);
    let pale = r14a_mix(c, (196.0, 184.0, 166.0), 0.42);
    for _ in 0..36 {
        let at = (rnd() * w, rnd() * h);
        let (col, a) = if rnd() < 0.55 { (dark, 0.10 + rnd() * 0.14) } else { (pale, 0.06 + rnd() * 0.10) };
        r14a_soft(ctx, at, (10.0 + rnd() * 34.0) * s, 0.5 + rnd() * 0.5, rnd() * 3.0, col, a);
    }
    // the grain: pores and a fine pebble
    for _ in 0..(2600.0 * s * s) as i32 {
        let col = if rnd() < 0.7 { dark } else { pale };
        ctx.set_fill_style(&r14a_css(col, 0.12 + rnd() * 0.16));
        ctx.fill_rect(rnd() * w, rnd() * h, 1.0, 1.0);
    }
    // creases, each with its pale lip
    for _ in 0..(48.0 * s) as i32 {
        let at = (rnd() * w, rnd() * h);
        let len = (6.0 + rnd() * 24.0) * s;
        let ang = (rnd() - 0.5) * 1.0 + if rnd() < 0.25 { PI / 2.0 } else { 0.0 };
        r14a_wander(ctx, at, len, ang, (c.0 * 0.4, c.1 * 0.36, c.2 * 0.34), 0.45, 0.8 * s.max(0.7));
        r14a_wander(ctx, (at.0, at.1 + 1.0), len * 0.8, ang, pale, 0.18, 0.7);
    }
    // rubbed pale and scuffed, most of it low: the dye worn through to the
    // crust in ragged streaks along the rub, scratched
    for i in 0..30 {
        let y = if i < 22 { h * (0.66 + rnd() * 0.34) } else { rnd() * h };
        let at = (rnd() * w, y);
        let crust = r14a_mix(c, (190.0, 178.0, 160.0), 0.40 + rnd() * 0.2);
        let ang = (rnd() - 0.5) * 0.5;
        for _ in 0..(3 + (rnd() * 5.0) as i32) {
            let a2 = (at.0 + (rnd() - 0.5) * 10.0 * s, at.1 + (rnd() - 0.5) * 4.0 * s);
            r14a_wander(
                ctx,
                a2,
                (2.0 + rnd() * 9.0) * s,
                ang + (rnd() - 0.5) * 0.3,
                crust,
                0.18 + rnd() * 0.3,
                (0.5 + rnd() * 0.9) * s.max(0.6),
            );
        }
        r14a_soft(ctx, at, (4.0 + rnd() * 8.0) * s, 0.25, ang, crust, 0.10 + rnd() * 0.10);
    }
    // dust, and dried water's tide marks
    for _ in 0..12 {
        r14a_soft(ctx, (rnd() * w, rnd() * h), (18.0 + rnd() * 40.0) * s, 0.6, rnd() * 3.0, (150.0, 144.0, 132.0), 0.08 + rnd() * 0.08);
    }
    for _ in 0..3 {
        let (x, y, r) = (rnd() * w, rnd() * h, (10.0 + rnd() * 26.0) * s);
        ctx.set_stroke_style(&r14a_css(pale, 0.22));
        ctx.set_line_width(1.2 * s);
        ctx.begin_path();
        ctx.arc(x, y, r, rnd() * 3.0, 3.0 + rnd() * 3.0);
        ctx.stroke();
    }
}

/// What a glove is made of, piece by piece.
#[derive(Clone, Copy, PartialEq)]
enum R14aCloth {
    Leather,
    Knit,
    Canvas,
    Jersey,
}

/// A glove: its back's cloth and colour (sRGB), the leather tips, knuckle
/// strap and thumb of a leather palm work glove, its wrist's cloth and
/// colour, a stripe knitted round the wrist, the palm's colour, and a dress
/// glove's three stitched points.
struct R14aGlove {
    back: (R14aCloth, u32),
    tips: Option<u32>,
    cuff: (R14aCloth, u32),
    stripe: Option<u32>,
    under: u32,
    points: bool,
}

/// The glove of batch r14a: a right one lying back up, as modelled.
fn r14a_glove(s: &R14aGlove, vary: u32) -> Obj {
    let grp = group();
    let back = lambert!({ color: 0xffffff, map: &r14a_glove_back_tex(s.back.0, s.back.1, s.points, vary) });
    r14a_add(&grp, "Real__gloveBack", &back);
    r14a_add(&grp, "Real__gloveUnder", &lambert!({ color: s.under }));
    let cuff = lambert!({ color: 0xffffff, map: &r14a_glove_cuff_tex(s.cuff.0, s.cuff.1, s.stripe, vary ^ 0x5bd1) });
    r14a_add(&grp, "Real__gloveCuff", &cuff);
    if let Some(t) = s.tips {
        let tips = lambert!({ color: 0xffffff, map: &r14a_leather_tex(t, 128, vary ^ 0x2c1b) });
        r14a_add(&grp, "Real__gloveTips", &tips);
    }
    grp
}

/// The glove's back as modelled (r14a.py): its bounds in x and z (the top's
/// u runs with x, v against z), and each digit: base (x, z), length, flat
/// width, angle from +z toward +x, bow.
const R14A_GLOVE_XZ: (f64, f64, f64, f64) = (-0.0757, 0.0877, 0.0, 0.1955);

const R14A_DIGITS: [(f64, f64, f64, f64, f64, f64); 5] = [
    (0.0335, 0.094, 0.079, 0.0232, 0.13, 0.006),
    (0.0110, 0.098, 0.087, 0.0238, 0.03, -0.003),
    (-0.0120, 0.096, 0.080, 0.0228, -0.08, -0.007),
    (-0.0335, 0.088, 0.064, 0.0205, -0.23, -0.010),
    (0.045, 0.030, 0.068, 0.0265, 0.72, -0.0085),
];

/// A digit's axis at t (0 its base, 1 its tip): the point and its direction.
fn r14a_digit_at(d: &(f64, f64, f64, f64, f64, f64), t: f64) -> ((f64, f64), (f64, f64)) {
    let (bx, bz, l, _, ang, bow) = *d;
    let am = ang + bow / l * t;
    let a = ang + bow / l * 2.0 * t;
    ((bx + am.sin() * l * t, bz + am.cos() * l * t), (a.sin(), a.cos()))
}

/// A cloth's ground in colour `base`: knit wales, a canvas weave, a jersey's
/// fine knit, or leather; faded unevenly.
fn r14a_paint_cloth(ctx: &mut Canvas, w: f64, h: f64, cloth: R14aCloth, base: u32) {
    if cloth == R14aCloth::Leather {
        r14a_paint_leather(ctx, w, h, base);
        return;
    }
    let c = r14a_rgb(base);
    ctx.set_fill_style(&r14a_css(c, 1.0));
    ctx.fill_rect(0, 0, w, h);
    let dark = (c.0 * 0.66, c.1 * 0.64, c.2 * 0.62);
    let pale = r14a_mix(c, (200.0, 196.0, 186.0), 0.35);
    match cloth {
        R14aCloth::Canvas => {
            // the plain weave: warp and weft every 2 px, each thread uneven
            for i in (0..w as i32).step_by(2) {
                ctx.set_fill_style(&r14a_css(dark, 0.10 + rnd() * 0.10));
                ctx.fill_rect(i as f64, 0, 1, h);
            }
            for j in (0..h as i32).step_by(2) {
                ctx.set_fill_style(&r14a_css(pale, 0.08 + rnd() * 0.10));
                ctx.fill_rect(0, j as f64, w, 1);
            }
        }
        _ => {
            // knitted: columns of V stitches (a jersey's finer)
            let step = if cloth == R14aCloth::Knit { 4.0 } else { 3.0 };
            ctx.set_stroke_style(&r14a_css(dark, 0.22));
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
                r14a_wander(ctx, (rnd() * w, rnd() * h), 2.0 + rnd() * 4.0, rnd() * 6.3, pale, 0.25, 0.5);
            }
            for _ in 0..90 {
                ctx.set_fill_style(&r14a_css(pale, 0.4));
                ctx.fill_rect(rnd() * w, rnd() * h, 1.5, 1.5);
            }
        }
    }
    // faded and soiled unevenly
    for _ in 0..30 {
        let (col, a) = if rnd() < 0.5 { ((c.0 * 0.6, c.1 * 0.56, c.2 * 0.5), 0.10 + rnd() * 0.12) } else { (pale, 0.08 + rnd() * 0.1) };
        r14a_soft(ctx, (rnd() * w, rnd() * h), 10.0 + rnd() * 34.0, 0.6 + rnd() * 0.4, rnd() * 3.0, col, a);
    }
}

/// The back of the glove: its cloth; the fingers' joints creased across, the
/// knuckles and fingertips rubbed and soiled where they took the work; a
/// dress glove's three stitched points; the dirt of the floor.
fn r14a_glove_back_tex(cloth: R14aCloth, base: u32, points: bool, vary: u32) -> Tex {
    canvas_tex(256, 256, move |ctx, w, h| {
        let seed = (rnd() * 4.0e9) as u32 ^ vary;
        with_own_stream(seed, || {
            r14a_paint_cloth(ctx, w, h, cloth, base);
            let (x0, x1, z0, z1) = R14A_GLOVE_XZ;
            let px = |x: f64, z: f64| ((x - x0) / (x1 - x0) * w, (z - z0) / (z1 - z0) * h);
            let c = r14a_rgb(base);
            let crease = (c.0 * 0.42, c.1 * 0.38, c.2 * 0.35);
            let dirt = (70.0, 62.0, 52.0);
            for (k, d) in R14A_DIGITS.iter().enumerate() {
                let hw = d.3 / 2.0;
                let joints: &[f64] = if k == 4 { &[0.45] } else { &[0.30, 0.62] };
                for &tj in joints {
                    for dt in [-0.03, 0.0, 0.03] {
                        let ((x, z), (dx, dz)) = r14a_digit_at(d, tj + dt);
                        let (nx, nz) = (-dz, dx);
                        let a = px(x - nx * hw, z - nz * hw);
                        let m = px(x + dx * 0.002, z + dz * 0.002);
                        let b = px(x + nx * hw, z + nz * hw);
                        ctx.set_stroke_style(&r14a_css(crease, if dt == 0.0 { 0.45 } else { 0.22 }));
                        ctx.set_line_width(1.1);
                        ctx.begin_path();
                        ctx.move_to(a.0, a.1);
                        ctx.quadratic_curve_to(m.0, m.1, b.0, b.1);
                        ctx.stroke();
                    }
                }
                // the tip soiled and rubbed
                let ((x, z), _) = r14a_digit_at(d, 0.93);
                let p = px(x, z);
                r14a_soft(ctx, p, 9.0, 1.0, 0.0, dirt, 0.35);
                if cloth == R14aCloth::Leather {
                    r14a_soft(ctx, p, 5.0, 0.7, rnd(), r14a_mix(c, (200.0, 190.0, 172.0), 0.5), 0.3);
                }
                // the knuckle
                if k < 4 {
                    let ((x, z), _) = r14a_digit_at(d, -0.05);
                    r14a_soft(ctx, px(x, z), 8.0, 0.6, 0.0, r14a_mix(c, (196.0, 188.0, 170.0), 0.5), 0.22);
                }
            }
            if points {
                // three lines of stitching from between the fingers' bases
                // toward the wrist, a little converging
                for (xa, xb) in [(0.022, 0.016), (-0.0005, -0.0005), (-0.023, -0.017)] {
                    let (a, b) = (px(xa, 0.088), px(xb, 0.036));
                    ctx.set_stroke_style(&r14a_css(crease, 0.6));
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
                r14a_soft(ctx, (rnd() * w, rnd() * h), 6.0 + rnd() * 22.0, 0.6 + rnd() * 0.4, rnd() * 3.0, dirt, 0.08 + rnd() * 0.12);
            }
        })
    })
}

/// The wrist: a ribbed knit (its ribs along the arm, its mouth at the
/// canvas' top), a stripe knitted round it, or the leather hemmed.
fn r14a_glove_cuff_tex(cloth: R14aCloth, base: u32, stripe: Option<u32>, vary: u32) -> Tex {
    canvas_tex(128, 128, move |ctx, w, h| {
        let seed = (rnd() * 4.0e9) as u32 ^ vary;
        with_own_stream(seed, || {
            let c = r14a_rgb(base);
            if cloth == R14aCloth::Leather {
                r14a_paint_leather(ctx, w, h, base);
                // the hem's stitching by the mouth
                ctx.set_stroke_style(&r14a_css((c.0 * 0.45, c.1 * 0.4, c.2 * 0.38), 0.6));
                ctx.set_line_width(0.8);
                ctx.set_line_dash(&[2.0, 1.5]);
                ctx.begin_path();
                ctx.move_to(0, h * 0.10);
                ctx.line_to(w, h * 0.10);
                ctx.stroke();
                ctx.set_line_dash(&[]);
                return;
            }
            ctx.set_fill_style(&r14a_css(c, 1.0));
            ctx.fill_rect(0, 0, w, h);
            // the stripes, across it (the cuff runs z -0.066 at its mouth to 0)
            if let Some(sc) = stripe {
                let s = r14a_rgb(sc);
                for (a, b) in [(0.18, 0.26), (0.36, 0.44)] {
                    ctx.set_fill_style(&r14a_css(s, 1.0));
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
                r14a_wander(
                    ctx,
                    (rnd() * w, rnd() * h),
                    2.0 + rnd() * 3.0,
                    rnd() * 6.3,
                    r14a_mix(c, (210.0, 204.0, 192.0), 0.4),
                    0.22,
                    0.5,
                );
            }
            // grubby where it was pulled on, the mouth stretched and darker
            r14a_soft(ctx, (w * 0.5, 0.0), w * 0.7, 0.18, 0.0, (60.0, 54.0, 46.0), 0.35);
            for _ in 0..10 {
                r14a_soft(ctx, (rnd() * w, rnd() * h), 8.0 + rnd() * 16.0, 0.6, rnd() * 3.0, (70.0, 62.0, 52.0), 0.10 + rnd() * 0.1);
            }
        })
    })
}

/// The shoe or boot of batch r14a, `kind` "shoe" or "boot": `var` "R", "L",
/// "Side" (the right one fallen on its outer side, as it lies) or "" (the
/// boot standing). Colours sRGB: the leather, the lining, the sole's
/// rubber or leather, the laces, the eyelets' and hooks' metal.
fn r14a_footwear(kind: &str, var: &str, cols: (u32, u32, u32, u32, u32), vary: u32) -> Obj {
    let (leather, lining, sole, lace, metal) = cols;
    let grp = group();
    let name = |p: &str| format!("Real__{kind}{p}{var}");
    let boot = kind == "boot";
    let upper = lambert!({ color: 0xffffff, map: &r14a_leather_tex(leather, 256, vary) });
    r14a_add(&grp, &name("Upper"), &upper);
    r14a_add(&grp, &name("Lining"), &lambert!({ color: lining }));
    let edge = lambert!({ color: 0xffffff, map: &r14a_sole_tex(sole, false, vary ^ 0x91) });
    r14a_add(&grp, &name("Sole"), &edge);
    let heel =
        if boot || var == "Side" { edge.clone() } else { lambert!({ color: 0xffffff, map: &r14a_sole_tex(0x3e2c22, true, vary ^ 0x93) }) };
    r14a_add(&grp, &name("Heel"), &heel);
    // the tread shows only on one that has fallen over
    let tread = if var == "Side" {
        lambert!({ color: 0xffffff, map: &r14a_tread_tex(boot, sole, vary ^ 0x97) })
    } else {
        lambert!({ color: sole })
    };
    r14a_add(&grp, &name("Tread"), &tread);
    r14a_add(&grp, &name("Laces"), &lambert!({ color: lace }));
    let met = phong!({ color: metal, specular: 0x4a4a46, shininess: 40 });
    r14a_add(&grp, &name(if boot { "Hooks" } else { "Eyelets" }), &met);
    grp
}

/// A sole's edge and welt (or `stack`, a heel of leather lifts on a rubber
/// top piece, its layers across): inked and burnished, scuffed pale, dirt.
fn r14a_sole_tex(base: u32, stack: bool, vary: u32) -> Tex {
    canvas_tex(128, 128, move |ctx, w, h| {
        let seed = (rnd() * 4.0e9) as u32 ^ vary;
        with_own_stream(seed, || {
            let c = r14a_rgb(base);
            ctx.set_fill_style(&r14a_css(c, 1.0));
            ctx.fill_rect(0, 0, w, h);
            if stack {
                // the lifts (each 4-5 mm of the heel's 22), the top piece below
                let mut y = 0.0;
                while y < h * 0.76 {
                    ctx.set_fill_style(&r14a_css((c.0 * 0.55, c.1 * 0.5, c.2 * 0.48), 0.7));
                    ctx.fill_rect(0, y, w, 1.0);
                    ctx.set_fill_style(&r14a_css(r14a_mix(c, (150.0, 120.0, 96.0), 0.3), 0.3));
                    ctx.fill_rect(0, y + 1.0, w, 1.0);
                    y += h * 0.19;
                }
                ctx.set_fill_style("rgb(40,38,36)");
                ctx.fill_rect(0, h * 0.77, w, h * 0.23);
            }
            for _ in 0..40 {
                ctx.set_fill_style(&r14a_css((c.0 * 0.7, c.1 * 0.68, c.2 * 0.66), 0.2));
                ctx.fill_rect(rnd() * w, rnd() * h, 4.0 + rnd() * 18.0, 1.0);
            }
            for _ in 0..22 {
                let crust = r14a_mix(c, (150.0, 140.0, 126.0), 0.5);
                r14a_soft(ctx, (rnd() * w, rnd() * h), 2.0 + rnd() * 6.0, 0.4, (rnd() - 0.5) * 0.5, crust, 0.35);
            }
            for _ in 0..10 {
                r14a_soft(ctx, (rnd() * w, rnd() * h), 8.0 + rnd() * 20.0, 0.5, 0.0, (110.0, 102.0, 90.0), 0.12);
            }
        })
    })
}

/// The tread of one fallen on its side, as it lies (u from the toe, at u =
/// 0, back to the heel; v from the outer edge, on the floor, up to the inner
/// one): a shoe's leather sole worn dark at the ball and frayed at the toe,
/// paler in the waist where it never touched, the heel's rubber top piece
/// worn at its back; a boot's lugs, worn grey on their faces, grit in the
/// grooves.
fn r14a_tread_tex(boot: bool, base: u32, vary: u32) -> Tex {
    canvas_tex(256, 128, move |ctx, w, h| {
        let seed = (rnd() * 4.0e9) as u32 ^ vary;
        with_own_stream(seed, || {
            // the heel's breast, as modelled (r14a.py): the shoe's at 78% of
            // the tread from the toe, the boot's at 77%
            let br = if boot { 0.771 } else { 0.781 };
            if !boot {
                let leather = (122.0, 96.0, 70.0);
                ctx.set_fill_style(&r14a_css(leather, 1.0));
                ctx.fill_rect(0, 0, w, h);
                // worn: the ball dark and smooth, the toe frayed paler
                r14a_soft(ctx, (w * 0.34, h * 0.5), w * 0.22, 0.6, 0.0, (64.0, 54.0, 44.0), 0.6);
                r14a_soft(ctx, (w * 0.04, h * 0.5), w * 0.07, 1.4, 0.0, (150.0, 132.0, 108.0), 0.5);
                // the waist, never on the floor: paler, the size stamped
                r14a_soft(ctx, (w * 0.66, h * 0.5), w * 0.12, 0.6, 0.0, (160.0, 132.0, 100.0), 0.45);
                ctx.save();
                ctx.translate(w * 0.66, h * 0.5);
                ctx.rotate(-PI / 2.0);
                ctx.set_fill_style("rgba(60,46,34,0.55)");
                ctx.set_font("bold 11px Helvetica");
                ctx.set_text_align("center");
                ctx.set_text_baseline("middle");
                ctx.fill_text("9", 0, -6);
                ctx.set_font("7px Helvetica");
                ctx.fill_text("LEATHER", 0, 7);
                ctx.restore();
                for _ in 0..140 {
                    r14a_wander(ctx, (rnd() * w * br, rnd() * h), 3.0 + rnd() * 9.0, (rnd() - 0.5) * 6.3, (70.0, 58.0, 46.0), 0.25, 0.6);
                }
                // the heel's top piece, black rubber, worn at the back
                ctx.set_fill_style("rgb(44,41,38)");
                ctx.fill_rect(w * br, 0, w * (1.0 - br), h);
                r14a_soft(ctx, (w * 1.0, h * 0.35), w * 0.06, 1.8, 0.0, (96.0, 88.0, 78.0), 0.6);
                for k in 0..5 {
                    ctx.set_fill_style("rgba(120,112,100,0.7)");
                    ctx.fill_rect(w * (br + 0.03 + 0.035 * (k as f64 % 2.0)), h * (0.2 + 0.15 * k as f64), 2.0, 2.0);
                }
            } else {
                let c = r14a_rgb(base);
                let groove = (c.0 * 0.55, c.1 * 0.55, c.2 * 0.55);
                ctx.set_fill_style(&r14a_css(groove, 1.0));
                ctx.fill_rect(0, 0, w, h);
                // lugs in rows across the sole, chevroned; few in the waist
                let face = r14a_mix(c, (120.0, 114.0, 106.0), 0.35);
                let mut x = 6.0;
                let mut row = 0;
                while x < w - 4.0 {
                    let waist = x / w > 0.56 && x / w < br;
                    if x / w > br - 0.012 && x / w < br + 0.012 {
                        x += 6.0;
                        continue;
                    }
                    let n = if waist { 2 } else { 4 };
                    for k in 0..n {
                        let y = h * (0.12 + 0.76 * (k as f64 + 0.5 * (row % 2) as f64) / n as f64);
                        ctx.save();
                        ctx.translate(x, y);
                        ctx.rotate(if k * 2 < n { 0.5 } else { -0.5 });
                        ctx.set_fill_style(&r14a_css(face, 1.0));
                        ctx.fill_rect(-4.0, -9.0, 8.0, 18.0);
                        ctx.restore();
                    }
                    x += if waist { 16.0 } else { 11.0 };
                    row += 1;
                }
                // the faces worn grey, grit in the grooves
                r14a_soft(ctx, (w * 0.34, h * 0.5), w * 0.2, 0.6, 0.0, (130.0, 124.0, 114.0), 0.25);
                r14a_soft(ctx, (w * 0.92, h * 0.5), w * 0.1, 0.8, 0.0, (130.0, 124.0, 114.0), 0.25);
                for _ in 0..260 {
                    ctx.set_fill_style(&r14a_css((150.0, 140.0, 124.0), 0.35));
                    ctx.fill_rect(rnd() * w, rnd() * h, 1.0, 1.0);
                }
            }
        })
    })
}
