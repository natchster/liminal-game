//! `place_shoe_boxes`
use game::prelude::*;
use placers_kit::place::jitter;
use placers_kit::place::pick_open;

// Shoe box: 5% chance per level. A two-piece chipboard box a pair of
// men's shoes came in, its lid on or (35%) off and lying beside it, tissue
// paper left crumpled in it (Blender batch r14a: the doubled walls, a
// corner of the lid trodden in, its end label, the lid's print).
pub(crate) fn place_shoe_boxes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    // the old colours stand for the boards shoe boxes are made of: kraft,
    // a lighter kraft, an old brown kraft, grey board, white board gone
    // cream; (board, its ink)
    let box_colors = [0xc8a878, 0xd8b078, 0xa08858, 0x606060, 0xf0e0d0];
    let col = choice(&box_colors);
    let k = box_colors.iter().position(|&c| c == col).unwrap_or(0);
    let (board, ink) = [(0xa88a62, 0x2a2420), (0xb69468, 0x3a2418), (0x8a7252, 0x22201c), (0x5e5c58, 0xc8c0b0), (0xc4bcac, 0x2c3440)][k];
    let lid_off = rnd() < 0.35;
    let (px, pz) = jitter(g, x, z, 2.0);
    let vary = ((px * 7919.0).abs() as u32) ^ ((pz * 104_729.0).abs() as u32).rotate_left(5);
    let bx = group();
    let card = lambert!({ color: 0xffffff, map: &r14a_card_tex(board, vary) });
    r14a_box_add(&bx, "Real__shoeBoxBase", &card);
    let label = lambert!({ color: 0xffffff, map: &r14a_box_label_tex(k, vary ^ 0x3c) });
    r14a_box_add(&bx, "Real__shoeBoxLabel", &label);
    let lid = group();
    r14a_box_add(&lid, "Real__shoeBoxLid", &card);
    let print = lambert!({ color: 0xffffff, map: &r14a_box_print_tex(board, ink, vary ^ 0x71) });
    r14a_box_add(&lid, "Real__shoeBoxLidPrint", &print);
    // which way the lid lies, how askew it sits: from where the box is (no
    // new draw)
    let f = (px * 13.17 + pz * 7.31).abs().fract();
    if lid_off {
        // the tissue paper it held, crumpled in it
        r14a_box_add(&bx, "Real__shoeBoxTissue", &lambert!({ color: 0xd2cec4 }));
        lid.position().set(0.40, 0, 0.10); // clear of the box
        lid.rotation().y = 0.4;
        if f < 0.5 {
            // face down: its skirt up (the lid's frame is 40.2 mm tall)
            lid.rotation().x = PI;
            lid.position().y = 0.0402;
        }
    } else {
        // on the base: its inside top on the base's rim (r14a.py SB_LID_Y)
        lid.position().y = 0.0822;
        lid.rotation().y = (f - 0.5) * 0.03;
    }
    bx.add(&lid);
    bx.position().set(px, 0, pz);
    bx.rotation().y = rnd() * PI * 2.0;
    g.world.add(&bx);
}

/// The part `name` of batch r14a, as modelled, added to `grp` in `mat`.
fn r14a_box_add(grp: &Obj, name: &str, mat: &Mat) {
    if let Some(geo) = game::assets::props::prop_part_as_modelled(name) {
        grp.add(&mesh(geo, mat.clone()));
    }
}

fn r14a_box_css(c: u32, a: f64) -> String {
    format!("rgba({},{},{},{:.3})", (c >> 16) & 255, (c >> 8) & 255, c & 255, a)
}

/// A soft spot at `at`, `r` px, `k` times as tall as wide, turned `turn`
/// (the context squashed, so no edge cuts its fade).
fn r14a_box_soft(ctx: &mut Canvas, at: (f64, f64), r: f64, k: f64, turn: f64, c: u32, a: f64) {
    ctx.save();
    ctx.translate(at.0, at.1);
    ctx.rotate(turn);
    ctx.scale(1, k);
    let gr = ctx.create_radial_gradient(0, 0, 0, 0, 0, r);
    gr.add_color_stop(0, &r14a_box_css(c, a));
    gr.add_color_stop(1, &r14a_box_css(c, 0.0));
    ctx.set_fill_style(&gr);
    ctx.begin_path();
    ctx.arc(0, 0, r, 0, 7);
    ctx.fill();
    ctx.restore();
}

/// Chipboard in its colour `board` (sRGB): its fibres and flecks, unevenly
/// faded, rubbed pale and scuffed, a dried tide line of damp from the floor
/// (low on its sides), dust.
fn r14a_card_tex(board: u32, vary: u32) -> Tex {
    canvas_tex(256, 256, move |ctx, w, h| {
        let seed = (rnd() * 4.0e9) as u32 ^ vary;
        with_own_stream(seed, || {
            let (r, g, b) = ((board >> 16) & 255, (board >> 8) & 255, board & 255);
            let sh = |k: f64| {
                (((r as f64 * k).min(255.0) as u32) << 16) | (((g as f64 * k).min(255.0) as u32) << 8) | ((b as f64 * k).min(255.0) as u32)
            };
            ctx.set_fill_style(&r14a_box_css(board, 1.0));
            ctx.fill_rect(0, 0, w, h);
            for _ in 0..30 {
                let (c, a) = if rnd() < 0.5 { (sh(0.8), 0.12 + rnd() * 0.12) } else { (sh(1.15), 0.10 + rnd() * 0.10) };
                r14a_box_soft(ctx, (rnd() * w, rnd() * h), 12.0 + rnd() * 40.0, 0.5 + rnd() * 0.5, rnd() * 3.0, c, a);
            }
            // the board's fibres and flecks
            for _ in 0..1400 {
                let c = if rnd() < 0.6 { sh(0.7) } else { sh(1.2) };
                ctx.set_fill_style(&r14a_box_css(c, 0.25 + rnd() * 0.3));
                let a = rnd() * PI;
                let l = 1.0 + rnd() * 4.0;
                ctx.save();
                ctx.translate(rnd() * w, rnd() * h);
                ctx.rotate(a);
                ctx.fill_rect(0, 0, l, 0.8);
                ctx.restore();
            }
            // the damp's tide line, low on the sides, and the stain under it
            let y0 = h * (0.80 + rnd() * 0.06);
            ctx.set_fill_style(&r14a_box_css(sh(0.82), 0.35));
            ctx.fill_rect(0, y0, w, h - y0);
            ctx.set_stroke_style(&r14a_box_css(sh(0.62), 0.55));
            ctx.set_line_width(1.6);
            ctx.begin_path();
            ctx.move_to(0, y0);
            let mut xx = 0.0;
            while xx <= w {
                ctx.line_to(xx, y0 + (xx * 0.09).sin() * 2.0 + (rnd() - 0.5) * 2.0);
                xx += 8.0;
            }
            ctx.stroke();
            // rubbed pale, scuffed, a few scratches
            for _ in 0..24 {
                r14a_box_soft(
                    ctx,
                    (rnd() * w, rnd() * h),
                    3.0 + rnd() * 10.0,
                    0.3 + rnd() * 0.3,
                    (rnd() - 0.5) * 0.4,
                    sh(1.25),
                    0.25 + rnd() * 0.25,
                );
            }
            for _ in 0..14 {
                ctx.set_stroke_style(&r14a_box_css(sh(1.3), 0.4));
                ctx.set_line_width(0.7);
                ctx.begin_path();
                let (sx, sy) = (rnd() * w, rnd() * h);
                ctx.move_to(sx, sy);
                ctx.line_to(sx + (rnd() - 0.5) * 30.0, sy + (rnd() - 0.5) * 8.0);
                ctx.stroke();
            }
            for _ in 0..10 {
                r14a_box_soft(ctx, (rnd() * w, rnd() * h), 20.0 + rnd() * 40.0, 0.6, rnd() * 3.0, 0x948c80, 0.10 + rnd() * 0.08);
            }
            // every face's edges (the canvas' own): the board rubbed through
            // pale at the folds and corners, grimed along the foot
            for side in 0..4 {
                let mut t = 0.0;
                while t < 1.0 {
                    let d = 1.0 + rnd() * 4.0;
                    let a = 0.15 + rnd() * 0.35;
                    let (x, y, rw, rh) = match side {
                        0 => (t * w, 0.0, 6.0, d),
                        1 => (t * w, h - d, 6.0, d),
                        2 => (0.0, t * h, d, 6.0),
                        _ => (w - d, t * h, d, 6.0),
                    };
                    ctx.set_fill_style(&r14a_box_css(sh(1.3), a));
                    ctx.fill_rect(x, y, rw, rh);
                    t += 6.0 / w;
                }
            }
            r14a_box_soft(ctx, (w * 0.5, h), w * 0.7, 0.05, 0.0, 0x3a342c, 0.45);
            for (cx, cy) in [(0.0, 0.0), (w, 0.0), (0.0, h), (w, h)] {
                r14a_box_soft(ctx, (cx, cy), 10.0 + rnd() * 8.0, 1.0, 0.0, sh(1.35), 0.5);
            }
        })
    })
}

/// The end label (90 x 50 mm): what was in the box, as a shoe shop's label
/// gives it (no real maker): the style, its colour, the size, a barcode;
/// the paper yellowed and rubbed, a corner lifting dirty.
fn r14a_box_label_tex(k: usize, vary: u32) -> Tex {
    let (style, colour, size) =
        [("4417", "BROWN", "9"), ("2380", "TAN", "10"), ("6512", "BLACK", "8"), ("1093", "OXBLOOD", "9"), ("3304", "NAVY", "11")][k];
    canvas_tex(256, 144, move |ctx, w, h| {
        let seed = (rnd() * 4.0e9) as u32 ^ vary;
        with_own_stream(seed, || {
            ctx.set_fill_style("rgb(214,206,186)");
            ctx.fill_rect(0, 0, w, h);
            let ink = "rgba(36,32,30,0.88)";
            ctx.set_fill_style(ink);
            ctx.set_text_baseline("alphabetic");
            ctx.set_text_align("left");
            ctx.set_font("bold 15px Helvetica");
            ctx.fill_text("MEN'S DERBY", 12, 24);
            ctx.set_font("11px Helvetica");
            ctx.fill_text(&format!("STYLE  {style}"), 12, 46);
            ctx.fill_text(&format!("COLOUR  {colour}"), 12, 62);
            ctx.fill_text("UPPER  LEATHER", 12, 78);
            ctx.fill_text("WIDTH  F", 12, 94);
            // the size, boxed
            ctx.set_stroke_style(ink);
            ctx.set_line_width(2);
            ctx.stroke_rect(w - 78.0, 12, 64, 58);
            ctx.set_font("bold 40px Helvetica");
            ctx.set_text_align("center");
            ctx.fill_text(size, w - 46.0, 58);
            ctx.set_font("9px Helvetica");
            ctx.fill_text("UK SIZE", w - 46.0, 24);
            // the barcode and its digits
            let mut x = 12.0;
            let mut digits = String::new();
            for i in 0..12 {
                digits.push(char::from(b'0' + (rnd() * 10.0) as u8 % 10));
                if i == 5 {
                    digits.push(' ');
                }
            }
            while x < 150.0 {
                let bw = 1.0 + (rnd() * 3.0).floor();
                if rnd() < 0.55 {
                    ctx.fill_rect(x, 104, bw, 26);
                }
                x += bw;
            }
            ctx.set_font("9px Helvetica");
            ctx.set_text_align("left");
            ctx.fill_text(&digits, 14, 140);
            // yellowed and rubbed, the ink worn off in places, a dirty corner
            for _ in 0..16 {
                r14a_box_soft(ctx, (rnd() * w, rnd() * h), 10.0 + rnd() * 30.0, 0.6, rnd() * 3.0, 0xb8a47a, 0.10 + rnd() * 0.14);
            }
            for _ in 0..30 {
                r14a_box_soft(ctx, (rnd() * w, rnd() * h), 2.0 + rnd() * 5.0, 0.4, rnd() * 3.0, 0xd6ceb6, 0.5);
            }
            r14a_box_soft(ctx, (w, h), 40.0, 1.0, 0.0, 0x5a5044, 0.45);
        })
    })
}

/// The lid's print (220 x 130 mm, read from the box's long front): a
/// shoe shop's plain one-colour print (no real maker), a ruled border, rubbed
/// off where the lid was handled.
fn r14a_box_print_tex(board: u32, ink: u32, vary: u32) -> Tex {
    canvas_tex(256, 152, move |ctx, w, h| {
        let seed = (rnd() * 4.0e9) as u32 ^ vary;
        with_own_stream(seed, || {
            // the board shows through: the print is the only thing on it
            ctx.set_fill_style(&r14a_box_css(board, 1.0));
            ctx.fill_rect(0, 0, w, h);
            ctx.set_stroke_style(&r14a_box_css(ink, 0.85));
            ctx.set_line_width(2.5);
            ctx.stroke_rect(8, 8, w - 16.0, h - 16.0);
            ctx.set_line_width(0.8);
            ctx.stroke_rect(13, 13, w - 26.0, h - 26.0);
            ctx.set_fill_style(&r14a_box_css(ink, 0.88));
            ctx.set_text_align("center");
            ctx.set_text_baseline("middle");
            ctx.set_font("italic 30px Georgia");
            ctx.fill_text("Fine Footwear", w * 0.5, h * 0.42);
            ctx.set_font("10px Georgia");
            ctx.fill_text("GOODYEAR WELTED  \u{b7}  LEATHER LINED", w * 0.5, h * 0.66);
            // the ink rubbed off and the board's own grain over it
            for _ in 0..60 {
                r14a_box_soft(ctx, (rnd() * w, rnd() * h), 2.0 + rnd() * 9.0, 0.4 + rnd() * 0.4, rnd() * 3.0, board, 0.5 + rnd() * 0.4);
            }
            for _ in 0..500 {
                ctx.set_fill_style(&r14a_box_css(board, 0.5));
                ctx.fill_rect(rnd() * w, rnd() * h, 1.0 + rnd() * 3.0, 0.8);
            }
            for _ in 0..8 {
                r14a_box_soft(ctx, (rnd() * w, rnd() * h), 20.0 + rnd() * 40.0, 0.6, rnd() * 3.0, 0x948c80, 0.10);
            }
        })
    })
}
