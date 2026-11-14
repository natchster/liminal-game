//! `place_tissue_boxes`: Tissue box: 12% chance per level. A flat box of facial tissues (Blender batch r15, the 160-sheet carton at its real size, 225 x 121 x 88 mm): its printed board, the oval torn out along its perforation, a top corner trodden down, the last tissue drawn up gone limp over the top, all of it dusty. The old draws are kept call for call.
use game::prelude::*;
use placers_kit::paint::rgba;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

/// Tissue box: 12% chance per level. A flat box of facial tissues
/// (Blender batch r15, the 160-sheet carton at its real size, 225 x 121 x
/// 88 mm): its printed board, the oval torn out along its perforation, a
/// top corner trodden down, the last tissue drawn up gone limp over the
/// top, all of it dusty. The old draws are kept call for call.
pub(crate) fn place_tissue_boxes(g: &mut Game, _dist: &[i32]) {
    use game::assets::props::prop_part_as_modelled;
    if rnd() < 0.88 {
        return;
    } // 12% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    // the old colour draw picks the carton's print: cream, pale blue, lilac, grey
    let box_colors = [0xe8dcc0, 0xa0d0e8, 0xd0a0d8, 0xa0a0a0];
    let col = pick(&box_colors);
    let look = box_colors.iter().position(|&c| c == col).unwrap_or(0);
    // the old tissue's tilt: now the limp tissue's turn in the slot
    let lean = (rnd() - 0.5) * 0.3;
    let tissue = group();
    let px = g.wx(x) + (rnd() - 0.5) * 2.4;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.4;
    tissue.position().set(px, 0, pz);
    tissue.rotation().y = rnd() * PI * 2.0;
    // what is printed and how worn differs floor to floor: from where it lies
    let salt = salt(px, pz);
    let atlas = carton_atlas(look, salt);
    let cell = |c: (f64, f64, f64, f64)| {
        let t = atlas.duplicate();
        let (w, h) = (ATLAS.0, ATLAS.1);
        t.set_offset((c.0 + 1.0) / w, 1.0 - (c.1 + c.3 - 1.0) / h);
        t.set_repeat((c.2 - 2.0) / w, (c.3 - 2.0) / h);
        lambert!({ map: &t })
    };
    let parts = [
        ("Real__tissueBoxTop", cell(TOP)),
        ("Real__tissueBoxSides", cell(SIDE)),
        ("Real__tissueBoxEnds", cell(END)),
        ("Real__tissueBoxInside", lambert!({ color: 0x7e776a })),
        ("Real__tissueBoxStack", lambert!({ color: 0xcfcabd })),
    ];
    for (name, mat) in &parts {
        if let Some(geo) = prop_part_as_modelled(name) {
            tissue.add(&mesh(geo, mat));
        }
    }
    if let Some(geo) = prop_part_as_modelled("Real__tissueBoxSheet") {
        let sheet = mesh(geo, &lambert!({ map: &sheet_tex(salt), side: DoubleSide }));
        sheet.rotation().y = lean;
        tissue.add(&sheet);
    }
    g.world.add(&tissue);
}

/// The carton's atlas (px) and its cells (x, y, w, h): the top panel
/// (225 x 121 mm), a long side (225 x 88 mm, both sides print alike), an
/// end (121 x 88 mm)
const ATLAS: (f64, f64) = (1024.0, 512.0);

const TOP: (f64, f64, f64, f64) = (0.0, 0.0, 560.0, 302.0);

const SIDE: (f64, f64, f64, f64) = (0.0, 302.0, 560.0, 210.0);

const END: (f64, f64, f64, f64) = (560.0, 0.0, 300.0, 216.0);

/// A salt from where it lies
fn salt(px: f64, pz: f64) -> u32 {
    ((px * 977.0).floor() as i64 ^ ((pz * 1181.0).floor() as i64).rotate_left(13)) as u32
}

/// The print's ground and its pattern's ink, by the old colour (sRGB):
/// cream with grey-green sprigs, pale blue with white waves, lilac
/// with white flowers, grey with a fine stripe
const GROUND: [(f64, f64, f64); 4] = [(214.0, 202.0, 178.0), (146.0, 170.0, 182.0), (170.0, 148.0, 172.0), (156.0, 154.0, 150.0)];

const INK: [(f64, f64, f64); 4] = [(132.0, 146.0, 120.0), (226.0, 230.0, 228.0), (228.0, 222.0, 226.0), (112.0, 112.0, 116.0)];

/// A soft round spot: `r` px, `k` times as tall as wide, turned `turn`
/// (the context squashed, so the fade is never cut by an edge)
fn soft(ctx: &mut Canvas, at: (f64, f64), r: f64, k: f64, turn: f64, c: (f64, f64, f64), a: f64) {
    ctx.save();
    ctx.translate(at.0, at.1);
    ctx.rotate(turn);
    ctx.scale(1, k);
    let gr = ctx.create_radial_gradient(0, 0, 0, 0, 0, r);
    gr.add_color_stop(0, &rgba(c, a));
    gr.add_color_stop(1, &rgba(c, 0.0));
    ctx.set_fill_style(&gr);
    ctx.begin_path();
    ctx.arc(0, 0, r, 0, 7);
    ctx.fill();
    ctx.restore();
}

/// The pattern's repeat, drawn once and laid like a printing cylinder's
/// (sprigs in a half-drop, waves, five-petalled flowers); the grey
/// look's stripe is drawn straight
fn repeat_tile(look: usize) -> Option<(Canvas, f64, f64)> {
    let ink = INK[look];
    let (tw, th) = match look {
        0 => (72.0, 60.0),
        1 => (48.0, 28.0),
        2 => (52.0, 52.0),
        _ => return None,
    };
    let mut t = Canvas::new(tw as u32, th as u32);
    let wraps = |cx: f64, cy: f64| {
        let mut v = vec![];
        for ox in [-tw, 0.0, tw] {
            for oy in [-th, 0.0, th] {
                v.push((cx + ox, cy + oy));
            }
        }
        v
    };
    match look {
        0 => {
            for (k, (mx, my)) in [(18.0, 15.0), (54.0, 45.0)].into_iter().enumerate() {
                for (cx, cy) in wraps(mx, my) {
                    t.save();
                    t.translate(cx, cy);
                    t.rotate(-0.42 + 0.18 * k as f64);
                    t.set_stroke_style(rgba(ink, 0.7));
                    t.set_line_width(1.0);
                    t.begin_path();
                    t.move_to(0, 7);
                    t.quadratic_curve_to(1.5, 0, 0, -8);
                    t.stroke();
                    t.set_fill_style(rgba(ink, 0.65));
                    for (ly, sd) in [(-5.0, 1.0), (-1.0, -1.0), (3.0, 1.0), (-8.0, -1.0)] {
                        t.save();
                        t.translate(0, ly);
                        t.rotate(sd * 0.9);
                        t.scale(1, 0.42);
                        t.begin_path();
                        t.arc(sd * 3.4, 0, 3.4, 0, 7);
                        t.fill();
                        t.restore();
                    }
                    t.restore();
                }
            }
        }
        1 => {
            t.set_stroke_style(rgba(ink, 0.55));
            t.set_line_width(1.6);
            for y in [7.0, 21.0] {
                t.begin_path();
                let mut x = -24.0;
                t.move_to(x, y);
                while x < tw + 24.0 {
                    t.quadratic_curve_to(x + 6.0, y - 4.5, x + 12.0, y);
                    t.quadratic_curve_to(x + 18.0, y + 4.5, x + 24.0, y);
                    x += 24.0;
                }
                t.stroke();
            }
        }
        _ => {
            for (k, (mx, my)) in [(13.0, 13.0), (39.0, 39.0)].into_iter().enumerate() {
                for (cx, cy) in wraps(mx, my) {
                    let rot = 0.3 + 0.5 * k as f64;
                    t.set_fill_style(rgba(ink, 0.7));
                    for p in 0..5 {
                        let a = rot + p as f64 * PI * 2.0 / 5.0;
                        t.begin_path();
                        t.arc(cx + a.cos() * 3.2, cy + a.sin() * 3.2, 2.4, 0, 7);
                        t.fill();
                    }
                    t.set_fill_style(rgba((196.0, 170.0, 110.0), 0.8));
                    t.begin_path();
                    t.arc(cx, cy, 1.5, 0, 7);
                    t.fill();
                }
            }
        }
    }
    Some((t, tw, th))
}

/// The pattern printed all over a cell, by the look
fn pattern(ctx: &mut Canvas, look: usize, c: (f64, f64, f64, f64), tile: &Option<(Canvas, f64, f64)>) {
    let (x0, y0, w, h) = c;
    ctx.save();
    ctx.begin_path();
    ctx.rect(x0, y0, w, h);
    ctx.clip();
    if let Some((t, tw, th)) = tile {
        let mut y = y0;
        while y < y0 + h {
            let mut x = x0;
            while x < x0 + w {
                ctx.draw_image(t, x, y);
                x += tw;
            }
            y += th;
        }
    } else {
        ctx.set_fill_style(rgba(INK[look], 0.35));
        let mut x = x0 - h;
        while x < x0 + w {
            ctx.begin_path();
            ctx.move_to(x, y0 + h);
            ctx.line_to(x + h, y0);
            ctx.line_to(x + h + 2.5, y0);
            ctx.line_to(x + 2.5, y0 + h);
            ctx.close_path();
            ctx.fill();
            x += 7.0;
        }
    }
    ctx.restore();
}

/// Board worn at a cell's edges (the folds rubbed to the grey board
/// under the print) and dust lying on it
fn worn_edges(ctx: &mut Canvas, c: (f64, f64, f64, f64), r: &mut SplitMix64) {
    let (x0, y0, w, h) = c;
    for _ in 0..90 {
        let side = (r.next() * 4.0) as i32;
        let t = r.next();
        let (x, y) = match side {
            0 => (x0 + t * w, y0 + r.next() * 3.0),
            1 => (x0 + t * w, y0 + h - r.next() * 3.0),
            2 => (x0 + r.next() * 3.0, y0 + t * h),
            _ => (x0 + w - r.next() * 3.0, y0 + t * h),
        };
        let l = 3.0 + r.next() * 16.0;
        ctx.set_fill_style(rgba((198.0, 192.0, 180.0), 0.25 + 0.35 * r.next()));
        if side < 2 {
            ctx.fill_rect(x, y - 0.8, l, 1.6);
        } else {
            ctx.fill_rect(x - 0.8, y, 1.6, l);
        }
    }
}

/// The carton's atlas: the top's pattern and the oval's torn edge, the
/// sides' label in a serif and the pattern, the end's count and its
/// bar code; worn at every fold, dusty, the corner that was trodden on
/// creased, a tide line where water stood on the floor
fn carton_atlas(look: usize, salt: u32) -> Tex {
    canvas_tex_opt(ATLAS.0, ATLAS.1, false, move |ctx, w, h| {
        let mut r = SplitMix64::new(salt, 0x7155_0e5b);
        let g = GROUND[look];
        ctx.set_fill_style(rgba(g, 1.0));
        ctx.fill_rect(0, 0, w, h);
        // the top: the pattern, a printed ring round the oval, the
        // oval's perforation torn, the maker's words at one end
        let tile = repeat_tile(look);
        pattern(ctx, look, TOP, &tile);
        let (cx, cy) = (TOP.2 / 2.0, TOP.3 / 2.0);
        let (rx, ry) = (TOP.2 * 0.056 / 0.225, TOP.3 * 0.021 / 0.121);
        ctx.set_fill_style(rgba((236.0, 232.0, 222.0), 0.9));
        ctx.begin_path();
        ctx.ellipse(cx, cy, rx + 16.0, ry + 14.0, 0, 0, 7);
        ctx.fill();
        ctx.set_stroke_style(rgba(INK[look], 0.8));
        ctx.set_line_width(1.4);
        ctx.begin_path();
        ctx.ellipse(cx, cy, rx + 12.0, ry + 10.0, 0, 0, 7);
        ctx.stroke();
        // the torn perforation: the board's grey core showing at the edge
        for k in 0..160 {
            let a = k as f64 / 160.0 * PI * 2.0;
            let f = 1.0 + 0.04 * r.next();
            ctx.set_fill_style(rgba((172.0, 166.0, 154.0), 0.5 + 0.4 * r.next()));
            ctx.fill_rect(cx + a.cos() * rx * f - 1.0, cy + a.sin() * ry * f - 1.0, 1.5 + r.next() * 2.0, 1.5 + r.next() * 1.5);
        }
        ctx.set_fill_style(rgba((84.0, 78.0, 70.0), 0.75));
        ctx.set_text_align("left");
        ctx.set_font("italic 15px Times New Roman");
        ctx.fill_text("facial tissues", 18, TOP.3 - 18.0);
        worn_edges(ctx, TOP, &mut r);
        // the side: the pattern, a label panel and its words
        pattern(ctx, look, SIDE, &tile);
        let (sx, sy) = (SIDE.0 + 70.0, SIDE.1 + 38.0);
        ctx.set_fill_style(rgba((234.0, 230.0, 220.0), 0.94));
        ctx.begin_path();
        ctx.move_to(sx + 10.0, sy);
        ctx.arc_to(sx + 250.0, sy, sx + 250.0, sy + 120.0, 10);
        ctx.arc_to(sx + 250.0, sy + 120.0, sx, sy + 120.0, 10);
        ctx.arc_to(sx, sy + 120.0, sx, sy, 10);
        ctx.arc_to(sx, sy, sx + 250.0, sy, 10);
        ctx.close_path();
        ctx.fill();
        ctx.set_fill_style(rgba((70.0, 64.0, 60.0), 0.92));
        ctx.set_text_align("center");
        ctx.set_font("italic 34px Times New Roman");
        ctx.fill_text("Facial Tissues", sx + 125.0, sy + 50.0);
        ctx.set_font("15px Gill Sans");
        ctx.fill_text("SOFT  ·  2-PLY  ·  UNSCENTED", sx + 125.0, sy + 78.0);
        ctx.set_font("bold 17px Gill Sans");
        ctx.fill_text("160 TISSUES", sx + 125.0, sy + 103.0);
        ctx.set_font("11px Gill Sans");
        ctx.set_fill_style(rgba((70.0, 64.0, 60.0), 0.8));
        ctx.fill_text("2-ply facial tissue  ·  20.8 cm x 21.3 cm", SIDE.0 + 420.0, SIDE.1 + SIDE.3 - 14.0);
        worn_edges(ctx, SIDE, &mut r);
        // the end: the count, the size, the bar code
        ctx.set_fill_style(rgba((234.0, 230.0, 220.0), 0.94));
        ctx.fill_rect(END.0 + 22.0, END.1 + 24.0, END.2 - 44.0, 104);
        ctx.set_fill_style(rgba((70.0, 64.0, 60.0), 0.92));
        ctx.set_font("bold 40px Gill Sans");
        ctx.fill_text("160", END.0 + END.2 / 2.0, END.1 + 70.0);
        ctx.set_font("13px Gill Sans");
        ctx.fill_text("2-PLY FACIAL TISSUES", END.0 + END.2 / 2.0, END.1 + 92.0);
        ctx.set_font("11px Gill Sans");
        ctx.fill_text("80 double sheets  ·  20.8 x 21.3 cm", END.0 + END.2 / 2.0, END.1 + 114.0);
        // the bar code: an EAN-13's guard bars and modules over its digits
        let (bx, by) = (END.0 + 92.0, END.1 + 138.0);
        ctx.set_fill_style("rgba(236,232,224,0.95)");
        ctx.fill_rect(bx - 8.0, by - 4.0, 132, 42);
        ctx.set_fill_style("rgba(30,28,26,0.9)");
        let mut xx = bx;
        for k in 0..95 {
            let guard = k < 3 || (45..50).contains(&k) || k >= 92;
            let on = if guard { k % 2 == 0 || k == 46 || k == 48 } else { r.next() < 0.5 };
            if on {
                ctx.fill_rect(xx, by, 1.2, if guard { 32.0 } else { 28.0 });
            }
            xx += 1.2;
        }
        ctx.set_font("9px Helvetica");
        ctx.set_text_align("left");
        let digits: String = (0..13).map(|_| char::from(b'0' + (r.next() * 10.0) as u8)).collect();
        ctx.fill_text(&digits[..1], bx - 7.0, by + 37.0);
        ctx.fill_text(&digits[1..7], bx + 5.0, by + 37.0);
        ctx.fill_text(&digits[7..], bx + 60.0, by + 37.0);
        // the outer flap's edge, 14.5 mm above the foot: its shadow line
        let fy = END.1 + END.3 * (1.0 - 0.0145 / 0.088);
        ctx.set_fill_style("rgba(40,34,28,0.35)");
        ctx.fill_rect(END.0, fy, END.2, 1.5);
        ctx.set_fill_style("rgba(250,246,236,0.18)");
        ctx.fill_rect(END.0, fy - 1.5, END.2, 1.5);
        worn_edges(ctx, END, &mut r);
        // the corner trodden on (+x, +z): creases out from it on the top
        // (its right, nearest edge) and on the long side (its top right)
        ctx.set_line_width(1.0);
        for (ox, oy, a0) in [(TOP.2, TOP.3, PI), (SIDE.2, SIDE.1, PI * 0.5)] {
            for _ in 0..9 {
                let a = a0 + 0.1 + r.next() * (PI * 0.5 - 0.2);
                let l = 18.0 + r.next() * 50.0;
                let (sx0, sy0) = (ox - 4.0 * r.next(), oy + if oy < SIDE.1 + 1.0 && oy > TOP.3 { 4.0 * r.next() } else { -4.0 * r.next() });
                ctx.set_stroke_style(rgba((60.0, 52.0, 44.0), 0.35));
                ctx.begin_path();
                ctx.move_to(sx0, sy0);
                ctx.line_to(sx0 + a.cos() * l, sy0 + a.sin() * l);
                ctx.stroke();
                ctx.set_stroke_style(rgba((240.0, 234.0, 222.0), 0.25));
                ctx.begin_path();
                ctx.move_to(sx0 + 1.0, sy0 + 1.0);
                ctx.line_to(sx0 + a.cos() * l + 1.0, sy0 + a.sin() * l + 1.0);
                ctx.stroke();
            }
        }
        // the floor's grime up the sides and ends, a tide line where
        // water stood round it once
        for c in [SIDE, END] {
            let (x0, y0, cw, ch) = c;
            let gr = ctx.create_linear_gradient(0, y0 + ch, 0, y0 + ch - 40.0);
            gr.add_color_stop(0, "rgba(70,60,46,0.55)");
            gr.add_color_stop(1, "rgba(70,60,46,0)");
            ctx.set_fill_style(&gr);
            ctx.fill_rect(x0, y0 + ch - 40.0, cw, 40);
            let tide = y0 + ch - 22.0 - 6.0 * r.next();
            ctx.set_stroke_style("rgba(110,86,52,0.45)");
            ctx.set_line_width(1.6);
            ctx.begin_path();
            let mut xx = x0;
            ctx.move_to(xx, tide);
            while xx < x0 + cw {
                xx += 6.0;
                ctx.line_to(xx, tide + (r.next() - 0.5) * 4.0);
            }
            ctx.stroke();
            let gr = ctx.create_linear_gradient(0, tide, 0, y0 + ch);
            gr.add_color_stop(0, "rgba(120,96,60,0.18)");
            gr.add_color_stop(1, "rgba(120,96,60,0.05)");
            ctx.set_fill_style(&gr);
            ctx.fill_rect(x0, tide, cw, y0 + ch - tide);
        }
        // dust: a film over all of it, heaviest on the top, in drifts
        ctx.set_fill_style("rgba(120,110,94,0.20)");
        ctx.fill_rect(0, 0, w, h);
        for _ in 0..36 {
            let at = (TOP.0 + r.next() * TOP.2, TOP.1 + r.next() * TOP.3);
            soft(ctx, at, 20.0 + r.next() * 50.0, 0.5 + r.next() * 0.5, r.next() * 3.0, (128.0, 118.0, 100.0), 0.10 + 0.14 * r.next());
        }
        ctx.set_fill_style("rgba(128,118,100,0.16)");
        ctx.fill_rect(TOP.0, TOP.1, TOP.2, TOP.3);
        for _ in 0..260 {
            let (x, y) = (r.next() * w, r.next() * h);
            ctx.set_fill_style(rgba((90.0, 82.0, 70.0), 0.15 + 0.25 * r.next()));
            ctx.fill_rect(x, y, 1.0 + r.next() * 1.5, 1.0 + r.next() * 1.5);
        }
    })
}

/// The limp tissue: soft white gone grey with dust, darker in its folds
fn sheet_tex(salt: u32) -> Tex {
    canvas_tex_opt(128, 128, false, move |ctx, w, h| {
        let mut r = SplitMix64::new(salt, 0x5ee7);
        ctx.set_fill_style("#c8c2b4");
        ctx.fill_rect(0, 0, w, h);
        for _ in 0..40 {
            let at = (r.next() * w, r.next() * h);
            soft(ctx, at, 6.0 + r.next() * 20.0, 0.4 + r.next() * 0.6, r.next() * 3.0, (120.0, 110.0, 94.0), 0.12 + 0.2 * r.next());
        }
        // the embossed dots of two-ply tissue, faint
        ctx.set_fill_style("rgba(255,255,250,0.12)");
        let mut y = 2.0;
        while y < h {
            let mut x = if (y as i32 / 6) % 2 == 0 { 2.0 } else { 5.0 };
            while x < w {
                ctx.fill_rect(x, y, 1.2, 1.2);
                x += 6.0;
            }
            y += 6.0;
        }
    })
}
