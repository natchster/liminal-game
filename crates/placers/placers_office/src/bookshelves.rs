//! `place_bookshelves`
use crate::shared::fill;
use game::assets::props::prop_part_as_modelled;
use game::prelude::*;
use placers_kit::geometry::put;
use placers_kit::noise::fbm;
use placers_kit::noise::hash;
use placers_kit::noise::mix_unclamped;
use placers_kit::noise::sstep;
use placers_kit::place::mount_slid;
use placers_kit::place::vary as t10_vary;
use placers_kit::place::wall_candidates;
use placers_kit::place::wall_xz;

// Wall bookshelf: 3% chance per level. A hanging wall bookshelf of
// stained pine (Blender batch `t10`: 600 x 160 mm, 300 mm between its
// boards, its sides scrolled at the foot, a hardboard back), r02's cloth
// hardbacks standing in it, their spines faded by the years they faced the
// room, titles in tarnished gilt, the last one leaning on its neighbour.
pub(crate) fn place_bookshelves(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    } // 3% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = choice(&candidates);
    let book_colors = [0xa02040, 0x2050a0, 0x30a030, 0xd8b040, 0x604020, 0x181410, 0xd8a080];
    let shelf = group();
    // the old draws, call for call: how many books, and each one's colour,
    // spine width and height (now its kind and size) and lean
    let n_books = 12 + (rnd() * 7.0).floor() as i32;
    let mut books = vec![];
    for _ in 0..n_books {
        let col = choice(&book_colors);
        let spine_w = 0.014 + rnd() * 0.014;
        let spine_h = 0.11 + rnd() * 0.03;
        let lean = if rnd() < 0.20 { Some((rnd() - 0.5) * 0.3) } else { None };
        let (uw, uh) = ((spine_w - 0.014) / 0.014, (spine_h - 0.11) / 0.03);
        let kind = if uw < 0.33 {
            2
        } else if uw < 0.72 {
            0
        } else {
            1
        };
        let cloth = book_colors.iter().position(|&k| k == col).unwrap_or(0);
        books.push(BsBook { kind, cloth, scale: 0.94 + 0.10 * uh, lean });
    }
    let (wall_x, wall_z) = wall_xz(g, c, 0.080);
    mount_slid(&shelf, c, wall_x, wall_z, 1.4, 0.4);
    let p = shelf.pos();
    let vary = t10_vary(p.x, p.z, n_books as f64);
    // the parts are modelled with the wall at z = 0 and the bottom board's
    // underside at y = 0: their middle where the old shelf's was (y 1.48)
    let body = group();
    body.position().set(0, -0.0475, -0.080);
    shelf.add(&body);
    let stain = [0.42, 0.29, 0.19];
    put(&body, "Real__bookshelfSides", &lambert!({ map: &bs_wood_tex(vary, stain, true) }));
    put(&body, "Real__bookshelfBoards", &lambert!({ map: &bs_wood_tex(vary ^ 0x51, stain, false) }));
    put(&body, "Real__bookshelfBack", &lambert!({ color: 0x3b2c20 }));
    bs_books(&body, &books, vary);
    g.world.add(&shelf);
}

/// The shelf (victorian/parlour_and_office_big_things.py `BS_*`): its width, a board's thickness, its back's, its
/// height between the boards' outsides, how far its sides run below
const BS_W: f64 = 0.600;

const BS_T: f64 = 0.018;

/// r02's cloth hardbacks: (boards, spine, page block, thickness, length,
/// the fore-edge's z): the novel, the big one, the small one
const BS_BOOKS: [(&str, &str, &str, f64, f64, f64); 3] = [
    ("Real__bookHbABoards", "Real__bookHbASpine", "Real__bookHbAPages", 0.0363, 0.222, -0.0730),
    ("Real__bookHbBBoards", "Real__bookHbBSpine", "Real__bookHbBPages", 0.0503, 0.240, -0.0820),
    ("Real__bookHbCBoards", "Real__bookHbCSpine", "Real__bookHbCPages", 0.0273, 0.192, -0.0640),
];

/// Book cloth (sRGB), for the old colours in their order: maroon, navy,
/// bottle green, ochre buckram, brown, black gone brown, tan
const BS_CLOTH: [[f64; 3]; 7] = [
    [104.0, 44.0, 40.0],
    [44.0, 54.0, 76.0],
    [48.0, 74.0, 58.0],
    [128.0, 104.0, 62.0],
    [92.0, 66.0, 48.0],
    [46.0, 42.0, 38.0],
    [136.0, 116.0, 90.0],
];

const BS_TITLES: [&str; 24] = [
    "PROCEDURES MANUAL",
    "BUSINESS LAW",
    "PRINCIPLES OF ACCOUNTING",
    "ANNUAL REPORT",
    "MINUTES",
    "COLLECTED POEMS",
    "INDUSTRIAL SAFETY",
    "FILING AND RECORDS",
    "CONCISE DICTIONARY",
    "ATLAS OF THE WORLD",
    "TAX GUIDE",
    "OFFICE PRACTICE",
    "THE NORTHERN SHORE",
    "PERSONNEL POLICY",
    "BOOKKEEPING",
    "STANDARDS",
    "CORRESPONDENCE",
    "A SHORT HISTORY",
    "THE LONG AFTERNOON",
    "REGULATIONS",
    "ELEMENTS OF STATISTICS",
    "TYPEWRITING",
    "LEDGER",
    "SELECTED ESSAYS",
];

const BS_AUTHORS: [&str; 8] = ["HALE", "MARLOWE", "ASHBY", "WREN", "FORD", "DUNMORE", "KEMP", "ORME"];

/// One book on the shelf, from the old book's draws
struct BsBook {
    kind: usize,
    cloth: usize,
    scale: f64,
    lean: Option<f64>,
}

/// The books stood in a row from the left, as many as fit; the last leans
/// on its neighbour when it was drawn leaning and there is room
fn bs_books(body: &Obj, books: &[BsBook], vary: u32) {
    let x1 = BS_W / 2.0 - BS_T - 0.001;
    let mut x = -x1;
    let mut placed: Vec<(usize, f64, f64)> = vec![]; // (book, left x, its thickness)
    for (i, b) in books.iter().enumerate() {
        let t = BS_BOOKS[b.kind].3 * b.scale;
        if x + t > x1 {
            continue;
        }
        placed.push((i, x, t));
        x += t + 0.0006 + 0.0024 * hash(vary, i as i32, 3);
    }
    let atlas = bs_spine_atlas(books.iter().map(|b| (b.kind, b.cloth)).collect(), vary);
    let pages = lambert!({ color: 0xb5ab94 });
    let last = placed.len().saturating_sub(1);
    for (k, &(i, left, t)) in placed.iter().enumerate() {
        let b = &books[i];
        let (boards, spine, block, _, len, fore) = BS_BOOKS[b.kind];
        let h = len * b.scale;
        // stood on its tail, its front board to the right, its spine out:
        // a pivot at its foot's left corner, the book turned about it
        let pivot = group();
        pivot.position().set(left, BS_T, 0.008 - fore * b.scale + 0.010 * hash(vary, i as i32, 5));
        let mut lean = 0.0;
        if let (true, Some(l), true) = (k == last, b.lean, k > 0) {
            // its top on the neighbour's shoulder (the neighbour's right face)
            let room = x1 - left;
            let want = 0.12 + l.abs() * 2.0;
            let fit = ((room - t) / h).clamp(0.0, 0.5).asin();
            lean = want.min(fit);
            let prev = placed[k - 1].1 + placed[k - 1].2;
            pivot.position().x = prev + h * lean.sin();
        }
        pivot.rotation().z = lean;
        let book = group();
        book.rotation().z = -PI / 2.0;
        book.position().set(0, h / 2.0, 0);
        book.scale().set(b.scale, b.scale, b.scale);
        pivot.add(&book);
        let c = BS_CLOTH[b.cloth];
        let shade = 0.88 + 0.2 * hash(vary, i as i32, 7);
        let col = ((c[0] * shade).min(255.0) as u32) << 16 | ((c[1] * shade).min(255.0) as u32) << 8 | (c[2] * shade).min(255.0) as u32;
        let cell = atlas.duplicate();
        cell.set_offset(((i % 2) as f64 * 256.0 + 1.0) / 512.0, 1.0 - ((i / 2) as f64 * 32.0 + 31.0) / 512.0);
        cell.set_repeat(254.0 / 512.0, 30.0 / 512.0);
        put(&book, boards, &lambert!({ color: col }));
        put(&book, spine, &lambert!({ map: &cell }));
        put(&book, block, &pages);
        body.add(&pivot);
    }
}

/// The shelf's stained pine (as its faces are mapped): the grain along u
/// (the boards) or along v (the sides), a knot or two, the stain worn pale
/// on the edges a hand finds, dust
fn bs_wood_tex(vary: u32, stain: [f64; 3], up: bool) -> Tex {
    let (w, h) = if up { (128.0, 256.0) } else { (512.0, 128.0) };
    canvas_tex_opt(w, h, false, move |ctx, w, h| {
        let seed = vary ^ 0x7a1;
        let knots: Vec<(f64, f64)> = (0..3).map(|k| (hash(seed, k, 1), hash(seed, k, 2))).collect();
        fill(ctx, w, h, |u, v| {
            // along and across the grain
            let (a, b) = if up { (v * 2.0, u * 3.0) } else { (u * 6.0, v * 1.5) };
            let mut warp = fbm(seed, a * 2.0, b * 4.0, 3) * 0.6;
            for &(ku, kv) in &knots {
                let (da, db) = if up { ((v - kv) * 2.0, (u - ku) * 3.0) } else { ((u - ku) * 6.0, (v - kv) * 1.5) };
                let d = (da * da * 4.0 + db * db * 40.0).sqrt();
                warp += 0.25 / (d * 8.0 + 0.3);
            }
            let ring = ((b * 38.0 + warp * 9.0).sin() * 0.5 + 0.5).powf(3.0);
            let fleck = fbm(seed ^ 9, a * 40.0, b * 300.0, 2);
            let k = 0.86 + 0.18 * fleck - 0.22 * ring;
            let mut o = stain.map(|c| c * k);
            // the knots: dark, resinous
            for &(ku, kv) in &knots {
                let (da, db) = if up { ((v - kv) * 2.0, (u - ku) * 3.0) } else { ((u - ku) * 6.0, (v - kv) * 1.5) };
                let d = (da * da + db * db * 9.0).sqrt();
                o = mix_unclamped(o, [0.16, 0.09, 0.05], 0.8 * (1.0 - sstep(0.006, 0.02, d)));
            }
            // worn, dusty
            let wear = sstep(0.55, 0.8, fbm(seed ^ 13, a * 6.0, b * 10.0, 3));
            o = mix_unclamped(o, [0.55, 0.42, 0.30], 0.35 * wear);
            let dust = 0.10 + 0.14 * fbm(seed ^ 17, a * 9.0, b * 12.0, 3);
            mix_unclamped(o, [0.52, 0.49, 0.44], dust)
        });
    })
}

/// The shelf's spines in one 512 px atlas (256 x 32 a cell, two columns;
/// the head to the left, the front board up): cloth faded where the room's
/// light reached it, gilt rules and a title, a label sometimes; the head
/// rubbed and dusty
fn bs_spine_atlas(looks: Vec<(usize, usize)>, vary: u32) -> Tex {
    canvas_tex_opt(512, 512, false, move |ctx, _, _| {
        let mut r = SplitMix64::new(vary, 0x5b1e);
        ctx.set_fill_style("#4a4038");
        ctx.fill_rect(0, 0, 512, 512);
        for (i, &(kind, cloth)) in looks.iter().enumerate() {
            let (x0, y0) = ((i % 2) as f64 * 256.0, (i / 2) as f64 * 32.0);
            let c = BS_CLOTH[cloth];
            // faded toward a grey-brown, more on a light cloth's red
            let fade = r.range(0.18, 0.42);
            let base = [c[0] + (150.0 - c[0]) * fade, c[1] + (138.0 - c[1]) * fade, c[2] + (120.0 - c[2]) * fade];
            ctx.save();
            ctx.begin_path();
            ctx.rect(x0, y0, 256, 32);
            ctx.clip();
            ctx.set_fill_style(&format!("rgb({},{},{})", base[0] as u8, base[1] as u8, base[2] as u8));
            ctx.fill_rect(x0, y0, 256, 32);
            // the weave
            for k in 0..700 {
                let (x, y) = (x0 + r.next() * 256.0, y0 + r.next() * 32.0);
                ctx.set_fill_style(if k % 2 == 0 { "rgba(255,245,225,0.08)" } else { "rgba(0,0,0,0.10)" });
                ctx.fill_rect(x, y, 1.0 + r.next(), 1);
            }
            // the spine's round catches more light down its middle
            let gr = ctx.create_linear_gradient(x0, y0, x0, y0 + 32.0);
            gr.add_color_stop(0, "rgba(0,0,0,0.22)");
            gr.add_color_stop(0.5, "rgba(255,240,220,0.06)");
            gr.add_color_stop(1, "rgba(0,0,0,0.22)");
            ctx.set_fill_style(gr);
            ctx.fill_rect(x0, y0, 256, 32);
            // gilt on a dark cloth, black ink on a light one
            let gilt = if cloth == 3 || cloth == 6 { "rgba(34,26,20,0.78)" } else { "rgba(168,140,80,0.80)" };
            // the rules at its head and tail
            if r.next() < 0.75 {
                ctx.set_fill_style(gilt);
                for x in [14.0, 18.0, 236.0, 240.0] {
                    ctx.fill_rect(x0 + x, y0 + 2.0, 1.2, 28);
                }
            }
            let title = BS_TITLES[((r.next() * 24.0) as usize).min(23)];
            let author = BS_AUTHORS[((r.next() * 8.0) as usize).min(7)];
            ctx.set_text_baseline("middle");
            ctx.set_text_align("center");
            let label = r.next() < 0.4;
            let gilt = if label { "rgba(168,140,80,0.80)" } else { gilt };
            if label {
                // a leather label, the title on it
                ctx.set_fill_style("rgba(30,20,14,0.85)");
                ctx.fill_rect(x0 + 34.0, y0 + 5.0, 128, 22);
                ctx.set_stroke_style(gilt);
                ctx.set_line_width(0.8);
                ctx.stroke_rect(x0 + 36.0, y0 + 7.0, 124, 18);
            }
            ctx.set_fill_style(gilt);
            let size = if kind == 1 { 13 } else { 11 };
            ctx.set_font(&format!("bold {size}px Times New Roman"));
            ctx.fill_text_max(title, x0 + 98.0, y0 + 16.5, 120);
            ctx.set_font("9px Times New Roman");
            ctx.fill_text_max(author, x0 + 196.0, y0 + 16.5, 50);
            // a library's label at its tail now and then, a call number on it
            if r.next() < 0.25 {
                ctx.set_fill_style("rgba(206,196,170,0.92)");
                ctx.fill_rect(x0 + 214.0, y0 + 6.0, 18, 20);
                ctx.set_fill_style("rgba(40,34,30,0.8)");
                ctx.set_font("7px Helvetica");
                ctx.fill_text(&format!("{}", 100 + (r.next() * 800.0) as u32), x0 + 223.0, y0 + 16.0);
            }
            // the head and tail rubbed to the board, the head's dust
            for (x, w) in [(0.0, 5.0), (251.0, 5.0)] {
                ctx.set_fill_style("rgba(196,182,156,0.35)");
                ctx.fill_rect(x0 + x, y0, w, 32);
            }
            ctx.set_fill_style("rgba(150,140,124,0.25)");
            ctx.fill_rect(x0, y0, 9, 32);
            // a stain, scuffs
            if r.next() < 0.5 {
                let (x, y) = (x0 + 40.0 + r.next() * 180.0, y0 + 16.0);
                let rr = 6.0 + r.next() * 12.0;
                let gr = ctx.create_radial_gradient(x, y, 0, x, y, rr);
                gr.add_color_stop(0, "rgba(40,30,20,0.22)");
                gr.add_color_stop(1, "rgba(40,30,20,0)");
                ctx.set_fill_style(gr);
                ctx.fill_rect(x - rr, y - rr, 2.0 * rr, 2.0 * rr);
            }
            ctx.restore();
        }
    })
}
