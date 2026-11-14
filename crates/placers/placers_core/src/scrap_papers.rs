//! `place_scrap_papers`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::shuffle;

// Discarded scrap papers: 1-3 sheets of office paper per level, balled up
// and thrown away or smoothed out again and dropped (Blender family `everyday_litter`: a
// sheet of A4 balled tight, one crumpled loosely, one flattened out; printed).
pub(crate) fn place_scrap_papers(g: &mut Game, _dist: &[i32]) {
    use game::assets::props::prop_part_as_modelled;
    let count = 1 + (rnd() * 3.0).floor() as usize; // 1-3 scraps
    let mut open_tiles = open_tiles(g);
    shuffle(&mut open_tiles);
    let mut wad_mat: Option<Mat> = None;
    let mut page_mat: Option<Mat> = None;
    let mut i = 0;
    while i < count && i < open_tiles.len() {
        let (x, z) = open_tiles[i];
        // the old crumple's sizes: which scrap it is (x), how big (z)
        let size_x = 0.06 + rnd() * 0.05;
        let size_y = 0.04 + rnd() * 0.03;
        let size_z = 0.06 + rnd() * 0.05;
        let u = (size_x - 0.06) / 0.05;
        let (name, sheet) = if u < 0.42 {
            ("Real__paperBall", false)
        } else if u < 0.78 {
            ("Real__paperWad", false)
        } else {
            ("Real__paperSheet", true)
        };
        let mat =
            if sheet { page_mat.get_or_insert_with(scrap_page_mat).clone() } else { wad_mat.get_or_insert_with(scrap_wad_mat).clone() };
        let geo = prop_part_as_modelled(name).unwrap_or_else(|| box_geo!(size_x, size_y, size_z));
        let scrap = mesh(geo, &mat);
        // the old tilts are kept as draws: a ball rests on its flattest side,
        // a sheet lies where it fell
        let _rx = rnd();
        let ry = rnd() * PI * 2.0;
        let _rz = rnd();
        // a ball a little bigger or smaller (a sheet is A4)
        let k = if sheet { 1.0 } else { 0.9 + (size_z - 0.06) / 0.05 * 0.2 };
        scrap.scale().set(k, k, k);
        scrap.rotation().y = ry;
        let ox = (rnd() - 0.5) * 2.6;
        let oz = (rnd() - 0.5) * 2.6;
        scrap.position().set(g.wx(x) + ox, 0, g.wz(z) + oz);
        g.world.add(&scrap);
        i += 1;
    }
}

/// Office paper balled up: fragments of its print turned every way, the
/// creases' grime, a footprint's dust (laid on the ball by its box faces).
fn scrap_wad_mat() -> Mat {
    let tex = canvas_tex(256, 256, |ctx, w, h| {
        ctx.set_fill_style("#e4e0d4");
        ctx.fill_rect(0, 0, w, h);
        // the paper's own unevenness
        for _ in 0..260 {
            let (x, y) = (rnd() * w, rnd() * h);
            let v = 212.0 + rnd() * 30.0;
            ctx.set_fill_style(format!("rgba({v:.0},{:.0},{:.0},0.18)", v - 4.0, v - 14.0));
            ctx.fill_rect(x, y, 3.0 + rnd() * 9.0, 3.0 + rnd() * 9.0);
        }
        // the print: patches of type, each a facet of the sheet turned its own way
        for _ in 0..16 {
            let (cx, cy) = (rnd() * w, rnd() * h);
            let r = 20.0 + rnd() * 30.0;
            ctx.save();
            ctx.begin_path();
            let n = 5 + (rnd() * 3.0).floor() as usize;
            for k in 0..n {
                let t = (k as f64 + rnd() * 0.4) / n as f64 * PI * 2.0;
                let rr = r * (0.6 + 0.4 * rnd());
                if k == 0 {
                    ctx.move_to(cx + t.dcos() * rr, cy + t.dsin() * rr);
                } else {
                    ctx.line_to(cx + t.dcos() * rr, cy + t.dsin() * rr);
                }
            }
            ctx.close_path();
            ctx.clip();
            ctx.translate(cx, cy);
            ctx.rotate(rnd() * PI * 2.0);
            paper_type_lines(ctx, -r - 10.0, -r - 10.0, 2.0 * r + 20.0, 2.0 * r + 20.0, 9.0, 0.35 + rnd() * 0.3);
            ctx.restore();
        }
        // crease lines: grime in the valleys
        ctx.set_line_width(0.8);
        for _ in 0..40 {
            let (x, y) = (rnd() * w, rnd() * h);
            let a = rnd() * PI;
            let l = 10.0 + rnd() * 40.0;
            ctx.set_stroke_style(format!("rgba(70,64,52,{:.2})", 0.08 + rnd() * 0.12));
            ctx.begin_path();
            ctx.move_to(x, y);
            ctx.line_to(x + a.dcos() * l, y + a.dsin() * l);
            ctx.stroke();
        }
        // dust from the floor
        for _ in 0..16 {
            let at = (rnd() * w, rnd() * h);
            scrap_soft(ctx, at, 8.0 + rnd() * 20.0, 0.5 + rnd() * 0.4, rnd() * 3.0, (96, 86, 66, 0.06 + rnd() * 0.12));
        }
    });
    lambert!({ map: &tex, color: 0xffffff })
}

/// The flattened sheet: a stock count printed on it (a title, a ruled table
/// of stationery), its creases, a shoe's dusty print across it.
fn scrap_page_mat() -> Mat {
    let tex = canvas_tex_opt(384, 544, false, |ctx, w, h| {
        ctx.set_fill_style("#e2ded2");
        ctx.fill_rect(0, 0, w, h);
        let ink = "rgba(36,34,32,0.86)";
        ctx.set_fill_style(ink);
        ctx.set_text_align("left");
        ctx.set_text_baseline("alphabetic");
        ctx.set_font("bold 17px Helvetica");
        ctx.fill_text("Stock Count Sheet", 34, 58);
        ctx.set_font("10px Helvetica");
        ctx.fill_text("Stationery store, ground floor", 34, 74);
        ctx.fill_text("Counted by: ____________   Date: ___ / ___ / ______", 34, 92);
        // the table
        let rows = [
            ("1043-A", "Copy paper A4, 80 g/m2, ream", "112"),
            ("1043-B", "Copy paper A3, ream", "14"),
            ("2210", "Toner cartridge, black", "6"),
            ("2211", "Toner cartridge, cyan", "2"),
            ("3105", "Staples 26/6, box of 5000", "38"),
            ("3112", "Binder clips 32 mm, box", "21"),
            ("3120", "Paper clips, box of 100", "45"),
            ("4001", "Folders, manila, pack of 50", "9"),
            ("4007", "Lever arch files, A4", "27"),
            ("5020", "Ballpoint pens, blue, box", "16"),
            ("5021", "Ballpoint pens, black, box", "11"),
            ("5040", "Highlighters, assorted", "7"),
            ("6110", "Envelopes C4, box of 250", "4"),
            ("6115", "Envelopes DL, box of 500", "5"),
            ("7002", "Labels, address, sheet", "300"),
            ("7010", "Tape, clear, 19 mm", "24"),
            ("7015", "Sticky notes 76 x 76", "60"),
            ("8001", "Correction fluid", "13"),
        ];
        let (x0, x1, x2, x3, x4) = (34.0, 92.0, 270.0, 312.0, w - 30.0);
        let top = 112.0;
        let rh = 19.0;
        ctx.set_stroke_style("rgba(40,38,36,0.7)");
        ctx.set_line_width(1.0);
        for k in 0..=rows.len() + 1 {
            let y = top + k as f64 * rh;
            ctx.begin_path();
            ctx.move_to(x0, y);
            ctx.line_to(x4, y);
            ctx.stroke();
        }
        for x in [x0, x1, x2, x3, x4] {
            ctx.begin_path();
            ctx.move_to(x, top);
            ctx.line_to(x, top + (rows.len() + 1) as f64 * rh);
            ctx.stroke();
        }
        ctx.set_font("bold 10px Helvetica");
        for (x, t) in [(x0, "Item"), (x1, "Description"), (x2, "Qty"), (x3, "Counted")] {
            ctx.fill_text(t, x + 4.0, top + 13.5);
        }
        ctx.set_font("10px Times");
        for (k, (a, b, c)) in rows.iter().enumerate() {
            let y = top + (k + 1) as f64 * rh + 13.5;
            ctx.fill_text(a, x0 + 4.0, y);
            ctx.fill_text(b, x1 + 4.0, y);
            ctx.fill_text(c, x2 + 4.0, y);
        }
        ctx.set_font("9px Helvetica");
        ctx.fill_text("Page 2 of 3", w / 2.0 - 22.0, h - 26.0);
        // the creases: thin lines darker on one side, lighter on the other
        for _ in 0..34 {
            let (x, y) = (rnd() * w, rnd() * h);
            let a = rnd() * PI;
            let l = 30.0 + rnd() * 140.0;
            let (dx, dy) = (a.dcos() * l, a.dsin() * l);
            ctx.set_line_width(1.0);
            ctx.set_stroke_style(format!("rgba(80,72,58,{:.2})", 0.10 + rnd() * 0.15));
            ctx.begin_path();
            ctx.move_to(x, y);
            ctx.line_to(x + dx, y + dy);
            ctx.stroke();
            ctx.set_stroke_style("rgba(255,255,250,0.10)");
            ctx.begin_path();
            ctx.move_to(x + 1.2, y + 1.2);
            ctx.line_to(x + dx + 1.2, y + dy + 1.2);
            ctx.stroke();
        }
        // a shoe stepped on it: the dusty print of a sole's tread, half of it
        let (sx, sy, sa) = (w * (0.35 + rnd() * 0.3), h * (0.45 + rnd() * 0.3), rnd() * PI);
        ctx.save();
        ctx.translate(sx, sy);
        ctx.rotate(sa);
        ctx.begin_path();
        ctx.ellipse(0, -40.0, 46.0, 62.0, 0, 0, 7);
        ctx.ellipse(0, 70.0, 34.0, 38.0, 0, 0, 7);
        ctx.clip();
        for k in 0..26 {
            let y = -110.0 + k as f64 * 8.0;
            ctx.set_fill_style(format!("rgba(92,82,64,{:.2})", 0.10 + rnd() * 0.10));
            ctx.fill_rect(-50.0 + rnd() * 6.0, y, 100.0, 4.0);
        }
        ctx.restore();
        // dirt at its edges, where it lay longest on the floor
        for _ in 0..40 {
            let (x, y) = if rnd() < 0.5 {
                (rnd() * w, if rnd() < 0.5 { rnd() * 30.0 } else { h - rnd() * 30.0 })
            } else {
                (if rnd() < 0.5 { rnd() * 24.0 } else { w - rnd() * 24.0 }, rnd() * h)
            };
            scrap_soft(ctx, (x, y), 8.0 + rnd() * 18.0, 0.5 + rnd() * 0.4, rnd() * 3.0, (96, 86, 66, 0.08 + rnd() * 0.14));
        }
    });
    lambert!({ map: &tex, color: 0xffffff, side: DoubleSide })
}

/// A soft round spot at `at`: `r` px, `k` times as tall as it is wide, turned
/// `turn`, its colour fading out from the middle (the context squashed, so
/// the fade is never cut by an edge).
fn scrap_soft(ctx: &mut Canvas, at: (f64, f64), r: f64, k: f64, turn: f64, rgba: (u8, u8, u8, f64)) {
    let (cr, cg, cb, a) = rgba;
    ctx.save();
    ctx.translate(at.0, at.1);
    ctx.rotate(turn);
    ctx.scale(1, k);
    let gr = ctx.create_radial_gradient(0, 0, 0, 0, 0, r);
    gr.add_color_stop(0, &format!("rgba({cr},{cg},{cb},{a:.3})"));
    gr.add_color_stop(1, &format!("rgba({cr},{cg},{cb},0)"));
    ctx.set_fill_style(&gr);
    ctx.begin_path();
    ctx.arc(0, 0, r, 0, 7);
    ctx.fill();
    ctx.restore();
}

/// Lines of small printed type (words of an office's papers) filling the box
/// (x, y, w, h), `size` px, its ink `alpha`.
fn paper_type_lines(ctx: &mut Canvas, x: f64, y: f64, w: f64, h: f64, size: f64, alpha: f64) {
    const WORDS: [&str; 32] = [
        "the", "and", "of", "to", "for", "account", "total", "date", "please", "report", "per", "unit", "page", "item", "order", "review",
        "with", "that", "office", "copy", "received", "amount", "number", "dept.", "on", "is", "be", "as", "by", "this", "invoice",
        "meeting",
    ];
    ctx.set_fill_style(format!("rgba(34,32,30,{alpha:.2})"));
    ctx.set_font(&format!("{size:.0}px Times"));
    ctx.set_text_align("left");
    let mut yy = y + size;
    while yy < y + h {
        let mut xx = x + rnd() * 6.0;
        let end = x + w - rnd() * w * 0.25;
        while xx < end {
            let word = WORDS[(rnd() * WORDS.len() as f64) as usize % WORDS.len()];
            ctx.fill_text(word, xx, yy);
            xx += ctx.measure_text(word).width + size * 0.35;
        }
        yy += size * 1.3;
    }
}
