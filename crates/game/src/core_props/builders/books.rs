//! The stack of books (family `book_stack_suitcase_coat_stand_and_pile`): cloth hardbacks and a paperback,
//! each stack its own painted atlas of covers, spines and page edges.
use super::*;

/// A stack of old books, hardbacks and paperbacks, laid askew
pub fn build_book_stack(g: &mut Game, cx: &mut PropsCtx) -> Built {
    let _ = (g, cx);
    // each book modelled in the family `book_stack_suitcase_coat_stand_and_pile` at its real size (cloth
    // hardbacks of three sizes, their boards over the page block, their
    // spines rounded; a trade paperback), its covers, spine and pages
    // painted in one atlas for the stack (docs/design/PROPS.md)
    use crate::assets::props::prop_part_as_modelled;
    const CLOTHS: [usize; 8] = [0, 1, 2, 3, 4, 5, 6, 7];
    // (boards, spine, pages, thickness, board width): hardbacks A, B, C, the paperback
    const KINDS: [(&str, &str, &str, f64, f64); 4] = [
        ("Real__bookHbABoards", "Real__bookHbASpine", "Real__bookHbAPages", 0.0352, 0.146),
        ("Real__bookHbBBoards", "Real__bookHbBSpine", "Real__bookHbBPages", 0.0492, 0.164),
        ("Real__bookHbCBoards", "Real__bookHbCSpine", "Real__bookHbCPages", 0.0262, 0.128),
        ("Real__bookPbCovers", "Real__bookPbSpine", "Real__bookPbPages", 0.0234, 0.140),
    ];
    // the old draws, call for call: how many, each book's size, cloth,
    // place, turn and ribbon (now a slip of paper kept in it), then the one
    // that stood on top (now one more laid there, turned across the rest)
    let mut books: Vec<(usize, usize, f64, f64, f64, f64, Option<f64>)> = vec![];
    let n = irand(3, 6);
    for _ in 0..n {
        let w = rand2(0.18, 0.26);
        let h = rand2(0.035, 0.055);
        let _ = rand2(0.14, 0.2); // (its depth: the kind says it now)
        let cloth = choice(&CLOTHS);
        let px = rand2(-0.03, 0.03);
        let pz = rand2(-0.03, 0.03);
        let yaw = rand2(-0.3, 0.3);
        let slip = if rnd() < 0.25 { Some(rand2(-0.05, 0.05)) } else { None };
        // which book its size made it: a small hardback, a paperback, a
        // novel, a big one; its thickness scales it a little
        let u = (w - 0.18) / 0.08;
        let kind = if u < 0.22 {
            2
        } else if u < 0.47 {
            3
        } else if u < 0.78 {
            0
        } else {
            1
        };
        books.push((kind, cloth, 0.94 + (h - 0.035) * 6.0, px, pz, yaw, slip));
    }
    if rnd() < 0.5 {
        let cloth = choice(&CLOTHS);
        let px = rand2(-0.06, 0.06);
        let pz = rand2(-0.06, 0.06);
        let lean = rand2(-0.5, 0.5);
        let turn = rand2(0.0, PI * 2.0);
        books.push((if lean < 0.0 { 3 } else { 0 }, cloth, 1.0, px, pz, turn, None));
    }
    let atlas = book_atlas(books.iter().map(|b| (b.0 == 3, b.1)).collect());
    // one cell of the atlas (pixels, from its top left), inset a pixel
    let cell = |x0: f64, y0: f64, w: f64, h: f64| {
        let t = atlas.duplicate();
        t.set_offset((x0 + 1.0) / 512.0, 1.0 - (y0 + h - 1.0) / 512.0);
        t.set_repeat((w - 2.0) / 512.0, (h - 2.0) / 512.0);
        lambert!({ map: &t })
    };
    let pages = cell(0.0, 384.0, 512.0, 128.0);
    let paper = lambert!({ color: 0xcdc4ad });
    let bg = group();
    let mut y = 0.0;
    for (j, &(kind, _, s, px, pz, yaw, slip)) in books.iter().enumerate() {
        let (boards, spine, block, thick, width) = KINDS[kind];
        let book = group();
        book.position().set(px, y, pz);
        book.rotation().y = yaw;
        book.scale().set(s, s, s);
        let cover = cell((j % 4) as f64 * 128.0, (j / 4) as f64 * 128.0, 128.0, 128.0);
        let back = cell((j % 2) as f64 * 256.0, 256.0 + (j / 2) as f64 * 32.0, 256.0, 32.0);
        for (name, mat) in [(boards, &cover), (spine, &back), (block, &pages)] {
            if let Some(geo) = prop_part_as_modelled(name) {
                book.add(&mesh(geo, mat));
            }
        }
        if let (Some(x), Some(geo)) = (slip, prop_part_as_modelled("Real__bookSlip")) {
            let m = mesh(geo, &paper);
            m.position().set(x, thick * 0.5, -width / 2.0 + 0.04);
            book.add(&m);
        }
        bg.add(&book);
        y += thick * s;
    }
    Built::new(bg, y)
}

/// Book cloth, faded (sRGB): maroon, bottle green, navy, brown, tan
/// buckram, slate, olive, black gone brown
const BOOK_CLOTH: [(f64, f64, f64); 8] = [
    (98.0, 46.0, 42.0),
    (50.0, 76.0, 60.0),
    (46.0, 56.0, 78.0),
    (94.0, 68.0, 50.0),
    (152.0, 130.0, 98.0),
    (88.0, 100.0, 108.0),
    (98.0, 94.0, 62.0),
    (48.0, 44.0, 40.0),
];
/// A paperback's printed cover: its ground and its band
const BOOK_PRINT: [((f64, f64, f64), (f64, f64, f64)); 8] = [
    ((172.0, 118.0, 76.0), (204.0, 194.0, 170.0)),
    ((86.0, 106.0, 88.0), (202.0, 192.0, 168.0)),
    ((80.0, 96.0, 118.0), (200.0, 190.0, 168.0)),
    ((198.0, 190.0, 168.0), (128.0, 64.0, 54.0)),
    ((112.0, 60.0, 54.0), (198.0, 186.0, 160.0)),
    ((72.0, 98.0, 100.0), (200.0, 190.0, 168.0)),
    ((158.0, 136.0, 82.0), (52.0, 48.0, 44.0)),
    ((118.0, 116.0, 110.0), (196.0, 188.0, 170.0)),
];
const BOOK_TITLES: [&str; 20] = [
    "THE SILENT HOUSE",
    "COLLECTED POEMS",
    "A WINTER JOURNEY",
    "HOUSEHOLD MANAGEMENT",
    "ELEMENTS OF GEOMETRY",
    "THE NORTHERN SHORE",
    "LETTERS",
    "TALES OF THE SEA",
    "PRACTICAL ARITHMETIC",
    "THE OLD ROAD",
    "BIRDS OF THE FIELD",
    "GENERAL CHEMISTRY",
    "THE GARDEN YEAR",
    "SHORT STORIES",
    "THE HOLLOW TREE",
    "ROADS AND BRIDGES",
    "A SHORT HISTORY",
    "THE LONG AFTERNOON",
    "SELECTED ESSAYS",
    "THE MAP ROOM",
];
const BOOK_AUTHORS: [&str; 10] =
    ["H. MARLOWE", "E. M. GRAY", "J. ASHBY", "R. T. HALE", "M. WREN", "A. C. FORD", "L. DUNMORE", "S. KEMP", "F. ORME", "D. LATHAM"];

fn rgb(c: (f64, f64, f64), a: f64) -> String {
    format!("rgba({},{},{},{a})", c.0 as u8, c.1 as u8, c.2 as u8)
}

/// A stack's books painted in one 512 px atlas: each book's front cover
/// (128 px cells, two rows; its head to the left, its fore-edge up, as its
/// board's top takes it), its spine (256 x 32 cells, four rows; head to the
/// left, the front board up) and the page edges all of them share (the
/// bottom 128 px)
fn book_atlas(looks: Vec<(bool, usize)>) -> Tex {
    canvas_tex_opt(512, 512, false, move |ctx, _, _| {
        // the ground under what no book uses: a dull board grey, never clear
        // (an opaque material samples a clear texel as black)
        ctx.set_fill_style("#5a5248");
        ctx.fill_rect(0, 0, 512, 512);
        for (i, &(paperback, c)) in looks.iter().enumerate() {
            let title = BOOK_TITLES[((rnd() * 20.0) as usize).min(19)];
            let author = BOOK_AUTHORS[((rnd() * 10.0) as usize).min(9)];
            let (x0, y0) = ((i % 4) as f64 * 128.0, (i / 4) as f64 * 128.0);
            let (sx, sy) = ((i % 2) as f64 * 256.0, 256.0 + (i / 2) as f64 * 32.0);
            ctx.save();
            ctx.begin_path();
            ctx.rect(x0, y0, 128, 128);
            ctx.clip();
            if paperback {
                book_print_cover(ctx, x0, y0, c, title, author);
            } else {
                book_cloth_cover(ctx, x0, y0, c, title);
            }
            ctx.restore();
            ctx.save();
            ctx.begin_path();
            ctx.rect(sx, sy, 256, 32);
            ctx.clip();
            book_spine(ctx, sx, sy, paperback, c, title, author);
            ctx.restore();
        }
        book_pages(ctx, 0.0, 384.0, 512.0, 128.0);
    })
}

/// a cloth's weave: a fine speckle lighter and darker than its ground
fn book_weave(ctx: &mut Canvas, x0: f64, y0: f64, w: f64, h: f64, n: usize) {
    // the light flecks and the dark, each gathered into one path and filled
    // once (the same draws, in the same order): a fill a fleck was most of a
    // level's build where books are many
    let mut flecks: [Vec<(f64, f64, f64)>; 2] = [Vec::with_capacity(n / 2 + 1), Vec::with_capacity(n / 2 + 1)];
    for k in 0..n {
        let (x, y) = (x0 + rnd() * w, y0 + rnd() * h);
        flecks[k % 2].push((x, y, 1.0 + rnd()));
    }
    for (style, list) in [("rgba(255,245,225,0.10)", &flecks[0]), ("rgba(0,0,0,0.12)", &flecks[1])] {
        ctx.set_fill_style(style);
        ctx.begin_path();
        for &(x, y, fw) in list {
            ctx.rect(x, y, fw, 1);
        }
        ctx.fill();
    }
}

/// a soft blotch (a water mark, a ring, grime)
fn book_stain(ctx: &mut Canvas, x: f64, y: f64, r: f64, ring: bool, col: &str) {
    let gr = ctx.create_radial_gradient(x, y, r * if ring { 0.75 } else { 0.0 }, x, y, r);
    gr.add_color_stop(0, "rgba(0,0,0,0)");
    gr.add_color_stop(if ring { 0.85 } else { 0.0 }, col);
    gr.add_color_stop(1, "rgba(0,0,0,0)");
    ctx.set_fill_style(gr);
    ctx.fill_rect(x - r, y - r, 2.0 * r, 2.0 * r);
}

/// A cloth board: the ground faded where the light found it, the weave, a
/// blind-stamped rule, sometimes a title stamped in gilt, the corners
/// bumped through to the board, the edges rubbed, a stain
fn book_cloth_cover(ctx: &mut Canvas, x0: f64, y0: f64, c: usize, title: &str) {
    let col = BOOK_CLOTH[c];
    ctx.set_fill_style(rgb(col, 1.0));
    ctx.fill_rect(x0, y0, 128, 128);
    // faded toward the fore-edge, where it lay in the light
    let gr = ctx.create_linear_gradient(x0, y0 + 128.0, x0, y0);
    gr.add_color_stop(0, "rgba(210,200,180,0.0)");
    gr.add_color_stop(1, &format!("rgba(210,200,180,{})", 0.10 + 0.15 * rnd()));
    ctx.set_fill_style(gr);
    ctx.fill_rect(x0, y0, 128, 128);
    book_weave(ctx, x0, y0, 128.0, 128.0, 1800);
    // where another book lay on it the cloth kept its colour
    if rnd() < 0.5 {
        ctx.set_fill_style("rgba(0,0,0,0.10)");
        ctx.fill_rect(x0 + 128.0 * rnd() * 0.4, y0 + 40.0 + 60.0 * rnd(), 128, 128);
    }
    ctx.set_stroke_style("rgba(0,0,0,0.22)");
    ctx.set_line_width(1.3);
    ctx.stroke_rect(x0 + 7.0, y0 + 7.0, 114, 114);
    if rnd() < 0.45 {
        // the title stamped on the front, in the cover's own frame (its top at the head)
        ctx.save();
        ctx.translate(x0, y0 + 128.0);
        ctx.rotate(-PI / 2.0);
        ctx.scale(128.0 / 146.0, 128.0 / 222.0);
        ctx.set_fill_style("rgba(170,146,92,0.75)");
        ctx.set_text_align("center");
        ctx.set_font("bold 10px Times New Roman");
        ctx.fill_text_max(title, 73, 52, 100);
        ctx.restore();
    }
    // bumped corners (the fore-edge's, top left and right of the cell) and rubbed edges
    for (x, y) in [(x0, y0), (x0 + 128.0, y0)] {
        book_stain(ctx, x, y, 7.0 + rnd() * 6.0, false, "rgba(176,164,140,0.75)");
    }
    for (x, y, w, h) in [(x0, y0, 128.0, 3.0), (x0, y0, 3.0, 128.0), (x0 + 125.0, y0, 3.0, 128.0)] {
        ctx.set_fill_style("rgba(200,190,170,0.22)");
        ctx.fill_rect(x, y, w, h);
    }
    if rnd() < 0.6 {
        let (x, y) = (x0 + 20.0 + rnd() * 88.0, y0 + 20.0 + rnd() * 88.0);
        book_stain(ctx, x, y, 10.0 + rnd() * 18.0, rnd() < 0.5, "rgba(40,30,20,0.22)");
    }
}

/// A paperback's front: its printed ground and band, the title, the
/// author, the publisher's mark; creased, its corners and edges worn white
fn book_print_cover(ctx: &mut Canvas, x0: f64, y0: f64, c: usize, title: &str, author: &str) {
    let (ground, band) = BOOK_PRINT[c];
    ctx.save();
    ctx.translate(x0, y0 + 128.0);
    ctx.rotate(-PI / 2.0);
    ctx.scale(128.0 / 140.0, 128.0 / 216.0);
    // the cover, 140 x 216 (mm): its spine at the left
    ctx.set_fill_style(rgb(ground, 1.0));
    ctx.fill_rect(0, 0, 140, 216);
    ctx.set_fill_style(rgb(band, 1.0));
    ctx.fill_rect(0, 70, 140, 70);
    ctx.set_fill_style(rgb(ground, 0.9));
    ctx.set_text_align("center");
    ctx.set_font("bold 13px Gill Sans");
    for (k, line) in title.splitn(2, ' ').enumerate() {
        ctx.fill_text_max(line, 70, 98.0 + k as f64 * 17.0, 120);
    }
    ctx.set_fill_style(rgb(band, 0.9));
    ctx.set_font("10px Gill Sans");
    ctx.fill_text_max(author, 70, 40, 120);
    ctx.begin_path();
    ctx.arc(70, 185, 9, 0, 7);
    ctx.fill();
    // a crease where it was bent back at the spine, and across a corner
    ctx.set_stroke_style("rgba(235,228,212,0.7)");
    ctx.set_line_width(1);
    ctx.begin_path();
    ctx.move_to(9, 0);
    ctx.line_to(10, 216);
    ctx.stroke();
    ctx.begin_path();
    ctx.move_to(140.0 - 30.0 * rnd() - 10.0, 216);
    ctx.line_to(140, 216.0 - 30.0 * rnd() - 12.0);
    ctx.stroke();
    // the edges worn white
    ctx.set_fill_style("rgba(235,228,212,0.35)");
    for (x, y, w, h) in [(0.0, 0.0, 140.0, 3.0), (0.0, 213.0, 140.0, 3.0), (137.0, 0.0, 3.0, 216.0)] {
        ctx.fill_rect(x, y, w, h);
    }
    ctx.restore();
    for (x, y) in [(x0, y0), (x0 + 128.0, y0)] {
        book_stain(ctx, x, y, 6.0 + rnd() * 5.0, false, "rgba(236,230,214,0.7)");
    }
    book_weave(ctx, x0, y0, 128.0, 128.0, 500);
    if rnd() < 0.6 {
        let (x, y) = (x0 + 20.0 + rnd() * 88.0, y0 + 20.0 + rnd() * 88.0);
        book_stain(ctx, x, y, 12.0 + rnd() * 16.0, rnd() < 0.6, "rgba(60,44,24,0.25)");
    }
}

/// A spine (head at the left, the front board up): cloth faded more than
/// its boards, the title and author stamped (gilt, or ink on a light
/// cloth), the rules at head and tail, the ends frayed; or a paperback's
/// printed spine, cracked along its length where it was read
fn book_spine(ctx: &mut Canvas, x0: f64, y0: f64, paperback: bool, c: usize, title: &str, author: &str) {
    let (ink, ground) = if paperback {
        let (g0, b0) = BOOK_PRINT[c];
        ctx.set_fill_style(rgb(g0, 1.0));
        ctx.fill_rect(x0, y0, 256, 32);
        ctx.set_fill_style(rgb(b0, 1.0));
        ctx.fill_rect(x0 + 70.0, y0, 110, 32);
        (rgb(g0, 0.95), b0)
    } else {
        let col = BOOK_CLOTH[c];
        let f = 0.18 + 0.2 * rnd();
        let faded = (col.0 + (196.0 - col.0) * f, col.1 + (186.0 - col.1) * f, col.2 + (166.0 - col.2) * f);
        ctx.set_fill_style(rgb(faded, 1.0));
        ctx.fill_rect(x0, y0, 256, 32);
        book_weave(ctx, x0, y0, 256.0, 32.0, 900);
        let light = col.0 + col.1 + col.2 > 300.0;
        ctx.set_fill_style(if light { "rgba(40,30,24,0.8)" } else { "rgba(180,152,92,0.8)" });
        for x in [x0 + 12.0, x0 + 15.0, x0 + 241.0, x0 + 244.0] {
            ctx.fill_rect(x, y0, 1.2, 32);
        }
        ((if light { "rgba(40,30,24,0.85)" } else { "rgba(184,156,96,0.85)" }).to_string(), faded)
    };
    ctx.set_fill_style(ink.as_str());
    ctx.set_text_align("left");
    ctx.set_text_baseline("middle");
    ctx.set_font(if paperback { "bold 9px Gill Sans" } else { "bold 9px Times New Roman" });
    ctx.fill_text_max(title, x0 + if paperback { 78.0 } else { 30.0 }, y0 + 17.0, if paperback { 96.0 } else { 140.0 });
    ctx.set_font(if paperback { "7px Gill Sans" } else { "7px Times New Roman" });
    if paperback {
        ctx.set_fill_style(rgb(ground, 0.9));
        ctx.fill_text_max(author, x0 + 186.0, y0 + 17.0, 64);
    } else {
        ctx.fill_text_max(author, x0 + 182.0, y0 + 17.0, 54);
    }
    ctx.set_text_baseline("alphabetic");
    // the stamping rubbed: specks of the ground over it
    for _ in 0..140 {
        ctx.set_fill_style(rgb(ground, 0.6));
        ctx.fill_rect(x0 + rnd() * 256.0, y0 + rnd() * 32.0, 1.5, 1.5);
    }
    if paperback {
        ctx.set_stroke_style("rgba(236,230,214,0.75)");
        ctx.set_line_width(0.8);
        for _ in 0..(2.0 + rnd() * 4.0) as usize {
            let y = y0 + 4.0 + rnd() * 24.0;
            let x = x0 + rnd() * 60.0;
            ctx.begin_path();
            ctx.move_to(x, y);
            ctx.line_to(x + 60.0 + rnd() * 140.0, y + rnd() * 2.0 - 1.0);
            ctx.stroke();
        }
    } else {
        // frayed at head and tail
        for x in [x0, x0 + 250.0] {
            ctx.set_fill_style("rgba(200,188,166,0.35)");
            ctx.fill_rect(x, y0, 6, 32);
        }
    }
    // dust settled on its upper edge
    let gr = ctx.create_linear_gradient(x0, y0, x0, y0 + 8.0);
    gr.add_color_stop(0, "rgba(150,140,120,0.35)");
    gr.add_color_stop(1, "rgba(150,140,120,0)");
    ctx.set_fill_style(gr);
    ctx.fill_rect(x0, y0, 256, 8);
}

/// Page edges: old paper gone yellow, the leaves' fine lines, tanned
/// darker toward the boards, foxed, grimy
fn book_pages(ctx: &mut Canvas, x0: f64, y0: f64, w: f64, h: f64) {
    ctx.set_fill_style("#d2c4a0");
    ctx.fill_rect(x0, y0, w, h);
    let mut y = y0;
    while y < y0 + h {
        ctx.set_fill_style(format!("rgba(90,70,40,{})", 0.05 + 0.12 * rnd()));
        ctx.fill_rect(x0, y, w, 0.6 + rnd() * 0.8);
        y += 1.0 + rnd() * 2.0;
    }
    for (ya, yb) in [(y0, y0 + 16.0), (y0 + h, y0 + h - 16.0)] {
        let gr = ctx.create_linear_gradient(x0, ya, x0, yb);
        gr.add_color_stop(0, "rgba(120,86,40,0.45)");
        gr.add_color_stop(1, "rgba(120,86,40,0)");
        ctx.set_fill_style(gr);
        ctx.fill_rect(x0, y0, w, h);
    }
    for _ in 0..60 {
        let (x, y) = (x0 + rnd() * w, y0 + rnd() * h);
        book_stain(ctx, x, y, 1.0 + rnd() * 3.0, false, "rgba(130,84,40,0.35)");
    }
    let gr = ctx.create_linear_gradient(x0, y0, x0 + w, y0);
    gr.add_color_stop(0, "rgba(60,50,40,0.15)");
    gr.add_color_stop(0.5, "rgba(60,50,40,0)");
    gr.add_color_stop(1, "rgba(60,50,40,0.15)");
    ctx.set_fill_style(gr);
    ctx.fill_rect(x0, y0, w, h);
}
