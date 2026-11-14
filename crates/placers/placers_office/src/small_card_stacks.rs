//! `place_small_card_stacks`
use game::prelude::*;
use placers_kit::paint::rgba;
use placers_kit::place::pick_open;
use placers_kit::place::scatter_pos;

// Small playing card stack: 8% chance per level. A pack of
// poker-size playing cards dropped (Blender family `paper_and_small_things_on_office_floors`, 63.5 x 88.9 mm, 0.3
// mm a card): three in ten spread in a fan, face down but the one under
// them; else the deck put down unsquared, its top few cards slid, the top
// one turned up half the time. Backs and faces printed, the type a serif;
// grimy at the edges. The old draws are kept call for call.
pub(crate) fn place_small_card_stacks(g: &mut Game, _dist: &[i32]) {
    use game::assets::props::prop_part_as_modelled;
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let Some((x, z)) = pick_open(g) else { return };
    let cards = group();
    let is_fanned = rnd() < 0.30;
    let n_visible = 5 + (rnd() * 4.0).floor() as i32;
    // the loose cards' slips (the old jitter draws, kept as they were)
    let mut slip = vec![];
    for _ in 0..n_visible {
        if !is_fanned {
            let cx = (rnd() - 0.5) * 0.004;
            let cz = (rnd() - 0.5) * 0.004;
            let ry = (rnd() - 0.5) * 0.05;
            slip.push((cx, cz, ry));
        }
    }
    // the old suit draws: the top card turned up, a red or a black one
    let face_up = if !is_fanned && rnd() < 0.50 { Some(rnd() < 0.5) } else { None };
    scatter_pos(g, &cards, x, z, 2.4);
    cards.rotation().y = rnd() * PI * 2.0;
    let salt = salt(&cards);
    let back = lambert!({ map: &back_tex(salt) });
    let card_geo = prop_part_as_modelled("Real__cardSingle");
    let lay = |m: &Mat, x: f64, y: f64, z: f64, ry: f64| {
        if let Some(geo) = &card_geo {
            let c = mesh(geo.clone(), m);
            c.position().set(x, y, z);
            c.rotation().y = ry;
            cards.add(&c);
        }
    };
    if is_fanned {
        // spread in a fan on the floor, the one under them face up
        let face = lambert!({ map: &face_tex(salt.is_multiple_of(2), salt) });
        for i in 0..n_visible {
            let rot = -PI / 6.0 + (i as f64 / (n_visible - 1) as f64) * (PI / 3.0);
            let dx = i as f64 * 0.0115;
            lay(if i == 0 { &face } else { &back }, dx - 0.042, i as f64 * 0.00033, 0.0, rot);
        }
    } else {
        // the deck, and the loose cards slid on it
        if let Some(geo) = prop_part_as_modelled("Real__cardDeck") {
            cards.add(&mesh(geo, &lambert!({ map: &edges_tex(salt) })));
        }
        let top = DECK_H;
        let face = face_up.map(|red| lambert!({ map: &face_tex(red, salt) }));
        for (i, &(cx, cz, ry)) in slip.iter().enumerate() {
            let last = i + 1 == slip.len();
            let m = match (&face, last) {
                (Some(f), true) => f,
                _ => &back,
            };
            lay(m, cx * 1.4, top + i as f64 * 0.00031, cz * 1.4, ry);
        }
    }
    g.world.add(&cards);
}

/// The deck under the loose cards is 46 cards, 0.3 mm each
const DECK_H: f64 = 46.0 * 0.0003;

fn salt(o: &Obj) -> u32 {
    let p = o.pos();
    ((p.x * 977.0).floor() as i64 ^ ((p.z * 1181.0).floor() as i64).rotate_left(13)) as u32
}

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

/// The card's size on the canvas: 4 px a millimetre
const W: f64 = 254.0;

const H: f64 = 356.0;

/// A card's own wear: grime at the edges where hands held it, the
/// corners rubbed, the floor's dust, a crease across one corner
fn worn(ctx: &mut Canvas, r: &mut SplitMix64) {
    for (x0, y0, x1, y1) in [(0.0, 0.0, 14.0, 0.0), (W, 0.0, W - 14.0, 0.0), (0.0, 0.0, 0.0, 14.0), (0.0, H, 0.0, H - 14.0)] {
        let gr = ctx.create_linear_gradient(x0, y0, x1, y1);
        gr.add_color_stop(0, "rgba(96,84,66,0.35)");
        gr.add_color_stop(1, "rgba(96,84,66,0)");
        ctx.set_fill_style(&gr);
        ctx.fill_rect(0, 0, W, H);
    }
    for _ in 0..14 {
        let at = (r.next() * W, r.next() * H);
        soft(ctx, at, 10.0 + r.next() * 30.0, 0.5 + r.next() * 0.5, r.next() * 3.0, (110.0, 100.0, 84.0), 0.08 + 0.1 * r.next());
    }
    let (cx, cy) = (if r.next() < 0.5 { 0.0 } else { W }, if r.next() < 0.5 { 0.0 } else { H });
    let l = 30.0 + 26.0 * r.next();
    let (sx, sy) = (if cx > 0.0 { -1.0 } else { 1.0 }, if cy > 0.0 { -1.0 } else { 1.0 });
    ctx.set_line_width(1.2);
    ctx.set_stroke_style("rgba(70,60,48,0.35)");
    ctx.begin_path();
    ctx.move_to(cx + sx * l, cy);
    ctx.line_to(cx, cy + sy * l);
    ctx.stroke();
    ctx.set_stroke_style("rgba(250,246,236,0.35)");
    ctx.begin_path();
    ctx.move_to(cx + sx * (l + 1.5), cy);
    ctx.line_to(cx, cy + sy * (l + 1.5));
    ctx.stroke();
    for _ in 0..120 {
        let (x, y) = (r.next() * W, r.next() * H);
        ctx.set_fill_style(rgba((96.0, 88.0, 74.0), 0.15 + 0.25 * r.next()));
        ctx.fill_rect(x, y, 1.0, 1.0);
    }
}

/// The back: a white border, a red field printed with a fine lattice
/// and a rosette in an oval; no maker's mark
fn back_tex(salt: u32) -> Tex {
    canvas_tex_opt(W, H, false, move |ctx, w, h| {
        let mut r = SplitMix64::new(salt, 0xbac4);
        ctx.set_fill_style("#e4ddcc");
        ctx.fill_rect(0, 0, w, h);
        let red = (136.0, 44.0, 46.0);
        let (bx, by, bw, bh) = (18.0, 18.0, w - 36.0, h - 36.0);
        ctx.save();
        ctx.begin_path();
        ctx.move_to(bx + 10.0, by);
        ctx.arc_to(bx + bw, by, bx + bw, by + bh, 10);
        ctx.arc_to(bx + bw, by + bh, bx, by + bh, 10);
        ctx.arc_to(bx, by + bh, bx, by, 10);
        ctx.arc_to(bx, by, bx + bw, by, 10);
        ctx.close_path();
        ctx.set_fill_style(rgba(red, 1.0));
        ctx.fill();
        ctx.clip();
        // the lattice: two sets of fine diagonals, a dot where they cross
        ctx.set_stroke_style("rgba(226,214,200,0.75)");
        ctx.set_line_width(1.1);
        let step = 9.0;
        let mut k = -h;
        ctx.begin_path();
        while k < w + h {
            ctx.move_to(k, 0);
            ctx.line_to(k + h, h);
            ctx.move_to(k + h, 0);
            ctx.line_to(k, h);
            k += step;
        }
        ctx.stroke();
        // the oval and its rosette
        let (cx, cy) = (w / 2.0, h / 2.0);
        ctx.set_fill_style(rgba(red, 1.0));
        ctx.begin_path();
        ctx.ellipse(cx, cy, 52, 70, 0, 0, 7);
        ctx.fill();
        ctx.set_stroke_style("rgba(226,214,200,0.85)");
        for (rx, ry, lw) in [(50.0, 68.0, 2.0), (44.0, 62.0, 1.0)] {
            ctx.set_line_width(lw);
            ctx.begin_path();
            ctx.ellipse(cx, cy, rx, ry, 0, 0, 7);
            ctx.stroke();
        }
        ctx.set_line_width(1.0);
        ctx.begin_path();
        for k in 0..16 {
            let a = k as f64 / 16.0 * PI * 2.0;
            ctx.move_to(cx + a.dcos() * 34.0, cy + a.dsin() * 34.0);
            ctx.ellipse(cx + a.dcos() * 18.0, cy + a.dsin() * 18.0, 16, 5, a, 0, 7);
        }
        ctx.stroke();
        ctx.begin_path();
        ctx.arc(cx, cy, 6, 0, 7);
        ctx.stroke();
        ctx.restore();
        // the ink rubbed off at the field's edges
        for _ in 0..60 {
            let t = r.next();
            let (x, y) = match (r.next() * 4.0) as i32 {
                0 => (bx + t * bw, by + r.next() * 3.0),
                1 => (bx + t * bw, by + bh - r.next() * 3.0),
                2 => (bx + r.next() * 3.0, by + t * bh),
                _ => (bx + bw - r.next() * 3.0, by + t * bh),
            };
            ctx.set_fill_style("rgba(220,208,192,0.5)");
            ctx.fill_rect(x - 1.0, y - 1.0, 2.0 + r.next() * 4.0, 2.0 + r.next() * 4.0);
        }
        worn(ctx, &mut r);
    })
}

/// A suit's pip, `s` px high, centred at (x, y): 0 hearts, 1 diamonds,
/// 2 spades, 3 clubs
fn pip(ctx: &mut Canvas, suit: usize, x: f64, y: f64, s: f64, down: bool) {
    ctx.save();
    ctx.translate(x, y);
    if down {
        ctx.rotate(PI);
    }
    ctx.scale(s / 20.0, s / 20.0);
    ctx.begin_path();
    match suit {
        0 => {
            ctx.move_to(0, 10);
            ctx.bezier_curve_to(-4, 5, -10, 1, -10, -4);
            ctx.bezier_curve_to(-10, -9, -4, -11, 0, -5);
            ctx.bezier_curve_to(4, -11, 10, -9, 10, -4);
            ctx.bezier_curve_to(10, 1, 4, 5, 0, 10);
            ctx.fill();
        }
        1 => {
            ctx.move_to(0, -10);
            ctx.quadratic_curve_to(3, -4, 8, 0);
            ctx.quadratic_curve_to(3, 4, 0, 10);
            ctx.quadratic_curve_to(-3, 4, -8, 0);
            ctx.quadratic_curve_to(-3, -4, 0, -10);
            ctx.fill();
        }
        2 => {
            ctx.move_to(0, -10);
            ctx.bezier_curve_to(-4, -5, -10, -1, -10, 3);
            ctx.bezier_curve_to(-10, 8, -4, 9, 0, 4);
            ctx.bezier_curve_to(4, 9, 10, 8, 10, 3);
            ctx.bezier_curve_to(10, -1, 4, -5, 0, -10);
            ctx.fill();
            ctx.begin_path();
            ctx.move_to(0, 3);
            ctx.quadratic_curve_to(1, 8, 4, 10);
            ctx.line_to(-4, 10);
            ctx.quadratic_curve_to(-1, 8, 0, 3);
            ctx.fill();
        }
        _ => {
            for (cx, cy) in [(0.0, -5.0), (-5.2, 2.0), (5.2, 2.0)] {
                ctx.begin_path();
                ctx.arc(cx, cy, 4.8, 0, 7);
                ctx.fill();
            }
            ctx.begin_path();
            ctx.move_to(0, 0);
            ctx.quadratic_curve_to(1, 7, 4, 10);
            ctx.line_to(-4, 10);
            ctx.quadratic_curve_to(-1, 7, 0, 0);
            ctx.fill();
        }
    }
    ctx.restore();
}

/// Where a pip card's pips go (as fractions of the pip field: x, y)
fn pips_of(n: usize) -> Vec<(f64, f64)> {
    let (l, m, rr) = (0.0, 0.5, 1.0);
    match n {
        1 => vec![(m, 0.5)],
        2 => vec![(m, 0.0), (m, 1.0)],
        3 => vec![(m, 0.0), (m, 0.5), (m, 1.0)],
        4 => vec![(l, 0.0), (rr, 0.0), (l, 1.0), (rr, 1.0)],
        5 => vec![(l, 0.0), (rr, 0.0), (m, 0.5), (l, 1.0), (rr, 1.0)],
        6 => vec![(l, 0.0), (rr, 0.0), (l, 0.5), (rr, 0.5), (l, 1.0), (rr, 1.0)],
        7 => vec![(l, 0.0), (rr, 0.0), (m, 0.25), (l, 0.5), (rr, 0.5), (l, 1.0), (rr, 1.0)],
        8 => vec![(l, 0.0), (rr, 0.0), (m, 0.25), (l, 0.5), (rr, 0.5), (m, 0.75), (l, 1.0), (rr, 1.0)],
        9 => vec![(l, 0.0), (rr, 0.0), (l, 1.0 / 3.0), (rr, 1.0 / 3.0), (m, 0.5), (l, 2.0 / 3.0), (rr, 2.0 / 3.0), (l, 1.0), (rr, 1.0)],
        _ => vec![
            (l, 0.0),
            (rr, 0.0),
            (m, 1.0 / 6.0),
            (l, 1.0 / 3.0),
            (rr, 1.0 / 3.0),
            (l, 2.0 / 3.0),
            (rr, 2.0 / 3.0),
            (m, 5.0 / 6.0),
            (l, 1.0),
            (rr, 1.0),
        ],
    }
}

/// A pip card's face (an ace to a ten, red or black): its indices in a
/// serif in two corners, its pips laid out as a deck's are, those in its
/// lower half turned
fn face_tex(red: bool, salt: u32) -> Tex {
    canvas_tex_opt(W, H, false, move |ctx, w, h| {
        let mut r = SplitMix64::new(salt, 0xface);
        ctx.set_fill_style("#e6dfcf");
        ctx.fill_rect(0, 0, w, h);
        let n = 1 + (r.next() * 10.0) as usize;
        let suit = if red { (r.next() * 2.0) as usize } else { 2 + (r.next() * 2.0) as usize };
        let ink = if red { "rgba(150,40,40,0.95)" } else { "rgba(34,30,28,0.95)" };
        let rank = if n == 1 { "A".to_string() } else { n.to_string() };
        ctx.set_fill_style(ink);
        ctx.set_text_align("center");
        for flip in [false, true] {
            ctx.save();
            if flip {
                ctx.translate(w, h);
                ctx.rotate(PI);
            }
            ctx.set_font("bold 30px Times New Roman");
            ctx.fill_text_max(&rank, 22, 38, 34);
            pip(ctx, suit, 22.0, 54.0, 17.0, false);
            ctx.restore();
        }
        let (fx, fy, fw, fh) = (64.0, 52.0, w - 128.0, h - 104.0);
        for (px, py) in pips_of(n) {
            let s = if n == 1 { 80.0 } else { 40.0 };
            pip(ctx, suit, fx + px * fw, fy + py * fh, s, py > 0.5);
        }
        worn(ctx, &mut r);
    })
}

/// The deck's edges (v up the deck): a line for each of the 46 cards,
/// greyer toward the bottom where it lay in the dust
fn edges_tex(salt: u32) -> Tex {
    canvas_tex_opt(32, 128, false, move |ctx, w, h| {
        let mut r = SplitMix64::new(salt, 0xed6e);
        ctx.set_fill_style("#d6cfbf");
        ctx.fill_rect(0, 0, w, h);
        for k in 0..46 {
            let y = (k as f64 + 0.5) * h / 46.0;
            ctx.set_fill_style(format!("rgba(80,64,58,{:.2})", 0.25 + 0.25 * r.next()));
            ctx.fill_rect(0, y, w, 0.9);
        }
        let gr = ctx.create_linear_gradient(0, h, 0, 0);
        gr.add_color_stop(0, "rgba(80,70,54,0.5)");
        gr.add_color_stop(1, "rgba(80,70,54,0.1)");
        ctx.set_fill_style(&gr);
        ctx.fill_rect(0, 0, w, h);
    })
}
