//! What is printed on the vending machine (`docs/VENDING.md`): one atlas of
//! wrappers for everything it sells, the price strips on its trays, the
//! stickers on its door, the grime on its glass, and the words on its
//! display.
//!
//! The goods are a vending route of the 1990s as nobody made it: plain
//! makers' names and plain words ("POTATO CHIPS", "PEANUT BAR"), printed as
//! such bags and wrappers were (a bold wordmark, a picture of the product,
//! a flavour band, a net weight, a sheen of foil). No real brand. Each is
//! painted once a process (the same for every machine) from its painter's
//! own stream (`canvas_tex`).
use crate::prelude::*;

/// The goods' atlas: its size in pixels
pub const ATLAS: f64 = 1024.0;

/// A kind of thing a coil holds
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Ware {
    /// a bag of crisps, puffs, pretzels or nuts (wide coils)
    Bag,
    /// a bar or a flat packet (narrow coils)
    Bar,
    /// a wrapped pastry or a pack of cookies (wide coils)
    Pastry,
    /// almond water in a plastic bottle (the bottom tray)
    Almond,
    /// a blister card of two batteries (narrow coils)
    Battery,
}

/// One printed design: its kind and its cell (x, y, w, h in atlas pixels,
/// from the top left)
#[derive(Clone, Copy, Debug)]
pub struct Design {
    pub kind: Ware,
    pub cell: [f64; 4],
    /// what the price strip says it is (a few words)
    pub name: &'static str,
}

/// Bags: (maker, product, flavour, ground, ink, band)
const BAGS: [(&str, &str, &str, &str, &str, &str); 10] = [
    ("MIDWAY", "POTATO CHIPS", "ORIGINAL", "#c8a020", "#7a1a10", "#e8e0c8"),
    ("MIDWAY", "POTATO CHIPS", "SALT & VINEGAR", "#3c78a8", "#f2efe2", "#e8e0c8"),
    ("COUNTY FAIR", "CHEESE CURLS", "BAKED", "#d8601a", "#2a1608", "#f0d050"),
    ("HOLLIS", "PRETZEL TWISTS", "LIGHTLY SALTED", "#1e4e86", "#f4e6b0", "#c03020"),
    ("HOLLIS", "CORN CHIPS", "ORIGINAL", "#a83020", "#f6e8c0", "#e0b030"),
    ("BAYSIDE", "SALTED PEANUTS", "ROASTED", "#e4d6b0", "#7a2414", "#2a5a30"),
    ("COUNTY FAIR", "BBQ CHIPS", "HICKORY", "#6a2a18", "#f0c060", "#d8a040"),
    ("MIDWAY", "SOUR CREAM & ONION", "CHIPS", "#2a7040", "#f2f0e8", "#e8e0c8"),
    ("SUNNYVALE", "PORK RINDS", "HOT", "#b01818", "#f8e8a0", "#202020"),
    ("BAYSIDE", "TRAIL MIX", "NUTS & RAISINS", "#5a3a20", "#f0e0b0", "#c89040"),
];
/// Bars: (product, small line, ground, ink)
const BARS: [(&str, &str, &str, &str); 8] = [
    ("PEANUT BAR", "MILK CHOCOLATE", "#5a2a14", "#f0d8a0"),
    ("CARAMEL CHEW", "NOUGAT & CARAMEL", "#c88a20", "#3a1808"),
    ("COCONUT", "DARK CHOCOLATE", "#1c3a6a", "#f2f2f0"),
    ("CHOC WAFER", "CRISP WAFER", "#9a1a1a", "#f4e8c8"),
    ("MINT ROLLS", "PEPPERMINT", "#2a8a6a", "#f8f8f0"),
    ("FRUIT CHEWS", "ASSORTED", "#c02a6a", "#fff0a0"),
    ("TOFFEE", "BUTTER TOFFEE", "#d0b060", "#6a2010"),
    ("PEANUT BUTTER CUPS", "TWO CUPS", "#e06a10", "#3a1406"),
];
/// Pastries: (product, small line, ground, ink)
const PASTRIES: [(&str, &str, &str, &str); 4] = [
    ("HONEY BUN", "GLAZED", "#f2e8d0", "#a04010"),
    ("OATMEAL COOKIES", "2 COOKIES", "#d8c8a0", "#5a2a10"),
    ("PEANUT BUTTER CRACKERS", "6 SANDWICHES", "#e8a030", "#5a1808"),
    ("CINNAMON ROLL", "ICED", "#f0e0c0", "#7a3010"),
];

/// Every design and its cell. The bags (192 × 256) fill the top half; the
/// bars (256 × 96) and pastries (192 × 160) the lower left; the almond
/// water's label (256 × 96) and the battery card (128 × 192) the right.
pub fn designs() -> Vec<Design> {
    let mut v = vec![];
    for (i, b) in BAGS.iter().enumerate() {
        v.push(Design { kind: Ware::Bag, cell: [(i % 5) as f64 * 192.0, (i / 5) as f64 * 256.0, 192.0, 256.0], name: b.1 });
    }
    for (i, b) in BARS.iter().enumerate() {
        v.push(Design { kind: Ware::Bar, cell: [(i % 2) as f64 * 256.0, 512.0 + (i / 2) as f64 * 96.0, 256.0, 96.0], name: b.0 });
    }
    for (i, p) in PASTRIES.iter().enumerate() {
        v.push(Design {
            kind: Ware::Pastry,
            cell: [512.0 + (i % 2) as f64 * 192.0, 512.0 + (i / 2) as f64 * 160.0, 192.0, 160.0],
            name: p.0,
        });
    }
    v.push(Design { kind: Ware::Almond, cell: [512.0, 832.0, 256.0, 96.0], name: "ALMOND WATER" });
    v.push(Design { kind: Ware::Battery, cell: [896.0, 512.0, 128.0, 192.0], name: "BATTERIES 2PK" });
    v
}

/// A cell in UV space (u0, v0, du, dv), the texture flipped in y
pub fn cell_uv(c: [f64; 4]) -> [f64; 4] {
    [c[0] / ATLAS, 1.0 - (c[1] + c[3]) / ATLAS, c[2] / ATLAS, c[3] / ATLAS]
}

fn rr(ctx: &mut Canvas, x: f64, y: f64, w: f64, h: f64, r: f64) {
    ctx.begin_path();
    ctx.move_to(x + r, y);
    ctx.arc_to(x + w, y, x + w, y + h, r);
    ctx.arc_to(x + w, y + h, x, y + h, r);
    ctx.arc_to(x, y + h, x, y, r);
    ctx.arc_to(x, y, x + w, y, r);
    ctx.close_path();
}

/// Text fitted to `max_w`, centred at (x, y)
fn fit_text(ctx: &mut Canvas, s: &str, x: f64, y: f64, max_w: f64, size: f64, face: &str) {
    let mut px = size;
    loop {
        ctx.set_font(&format!("{face} {px}px"));
        if ctx.measure_text(s).width <= max_w || px <= 6.0 {
            break;
        }
        px -= 1.0;
    }
    ctx.fill_text(s, x, y);
}

/// The sheen and crinkle of foil over a cell: a soft highlight down the
/// middle, darker edges, thin creases
fn foil(ctx: &mut Canvas, x: f64, y: f64, w: f64, h: f64, creases: usize) {
    let g = ctx.create_linear_gradient(x, y, x + w, y);
    g.add_color_stop(0.0, "rgba(0,0,0,0.28)");
    g.add_color_stop(0.35, "rgba(255,255,255,0.10)");
    g.add_color_stop(0.5, "rgba(255,255,255,0.16)");
    g.add_color_stop(0.65, "rgba(255,255,255,0.06)");
    g.add_color_stop(1.0, "rgba(0,0,0,0.30)");
    ctx.set_fill_style(&g);
    ctx.fill_rect(x, y, w, h);
    ctx.set_line_width(1);
    for _ in 0..creases {
        let (cx, cy) = (x + rnd() * w, y + rnd() * h);
        let a = rand2(-0.9, 0.9) + if rnd() < 0.5 { 0.0 } else { PI / 2.0 };
        let l = rand2(0.1, 0.35) * w;
        let light = rnd() < 0.5;
        ctx.set_stroke_style(if light { "rgba(255,255,255,0.16)" } else { "rgba(0,0,0,0.16)" });
        ctx.begin_path();
        ctx.move_to(cx - a.cos() * l / 2.0, cy - a.sin() * l / 2.0);
        ctx.line_to(cx + a.cos() * l / 2.0, cy + a.sin() * l / 2.0);
        ctx.stroke();
    }
}

/// Crimped seal stripes across a cell's top and bottom
fn crimp(ctx: &mut Canvas, x: f64, y: f64, w: f64, h: f64, band: f64) {
    for (y0, y1) in [(y, y + band), (y + h - band, y + h)] {
        ctx.set_fill_style("rgba(0,0,0,0.12)");
        ctx.fill_rect(x, y0, w, y1 - y0);
        let mut yy = y0 + 1.0;
        while yy < y1 {
            ctx.set_fill_style("rgba(255,255,255,0.18)");
            ctx.fill_rect(x, yy, w, 1.0);
            ctx.set_fill_style("rgba(0,0,0,0.18)");
            ctx.fill_rect(x, yy + 1.5, w, 1.0);
            yy += 4.0;
        }
    }
}

/// A heap of the product as printed on a bag: chips, curls, twists, nuts
fn heap(ctx: &mut Canvas, cx: f64, cy: f64, r: f64, what: &str) {
    let (base, dark) = match what {
        "CHEESE CURLS" => ("#f08a20", "#b05010"),
        "PRETZEL TWISTS" => ("#8a4a18", "#4a2208"),
        "CORN CHIPS" => ("#e0b040", "#a07018"),
        "SALTED PEANUTS" | "TRAIL MIX" => ("#c88a48", "#7a4a20"),
        "PORK RINDS" => ("#e8c890", "#a07848"),
        "BBQ CHIPS" => ("#c87830", "#7a3a10"),
        _ => ("#f0d070", "#b89030"),
    };
    // a soft shadow under the heap
    ctx.set_fill_style("rgba(0,0,0,0.22)");
    ctx.begin_path();
    ctx.ellipse(cx, cy + r * 0.55, r * 1.1, r * 0.35, 0, 0, PI * 2.0);
    ctx.fill();
    for i in 0..26 {
        let t = i as f64 / 26.0;
        let a = rnd() * PI * 2.0;
        let d = r * (1.0 - t) * rand2(0.2, 0.9);
        let (x, y) = (cx + a.cos() * d * 1.2, cy + a.sin() * d * 0.6 - t * r * 0.3);
        let (rx, ry) = (r * rand2(0.22, 0.36), r * rand2(0.14, 0.24));
        let rot = rnd() * PI;
        let g = ctx.create_linear_gradient(x - rx, y - ry, x + rx, y + ry);
        g.add_color_stop(0.0, base);
        g.add_color_stop(1.0, dark);
        ctx.set_fill_style(&g);
        ctx.begin_path();
        ctx.ellipse(x, y, rx, ry, rot, 0, PI * 2.0);
        ctx.fill();
        ctx.set_fill_style("rgba(255,255,240,0.25)");
        ctx.begin_path();
        ctx.ellipse(x - rx * 0.25, y - ry * 0.3, rx * 0.4, ry * 0.3, rot, 0, PI * 2.0);
        ctx.fill();
    }
}

fn paint_bag(ctx: &mut Canvas, c: [f64; 4], b: &(&str, &str, &str, &str, &str, &str)) {
    let [x, y, w, h] = c;
    let (maker, product, flavour, ground, ink, band) = *b;
    ctx.set_fill_style(ground);
    ctx.fill_rect(x, y, w, h);
    // a darker field behind the picture
    let g = ctx.create_radial_gradient(x + w / 2.0, y + h * 0.62, 4.0, x + w / 2.0, y + h * 0.62, w * 0.7);
    g.add_color_stop(0.0, "rgba(255,255,255,0.22)");
    g.add_color_stop(1.0, "rgba(0,0,0,0.0)");
    ctx.set_fill_style(&g);
    ctx.fill_rect(x, y, w, h);
    // the maker's oval
    ctx.set_fill_style(band);
    ctx.begin_path();
    ctx.ellipse(x + w / 2.0, y + h * 0.17, w * 0.40, h * 0.075, 0, 0, PI * 2.0);
    ctx.fill();
    ctx.set_fill_style(ground);
    ctx.set_text_align("center");
    ctx.set_text_baseline("middle");
    fit_text(ctx, maker, x + w / 2.0, y + h * 0.17, w * 0.66, 26.0, "bold italic Futura");
    // the product's name
    ctx.set_fill_style(ink);
    fit_text(ctx, product, x + w / 2.0, y + h * 0.33, w * 0.86, 34.0, "bold Impact");
    heap(ctx, x + w / 2.0, y + h * 0.62, w * 0.28, product);
    // the flavour band
    ctx.set_fill_style(band);
    ctx.fill_rect(x, y + h * 0.80, w, h * 0.07);
    ctx.set_fill_style(ground);
    fit_text(ctx, flavour, x + w / 2.0, y + h * 0.835, w * 0.8, 17.0, "bold Helvetica");
    // the net weight
    ctx.set_fill_style(ink);
    fit_text(ctx, "NET WT 1 OZ (28.3g)", x + w / 2.0, y + h * 0.91, w * 0.7, 10.0, "bold Helvetica");
    foil(ctx, x, y, w, h, 14);
    crimp(ctx, x, y, w, h, h * 0.06);
}

fn paint_bar(ctx: &mut Canvas, c: [f64; 4], b: &(&str, &str, &str, &str)) {
    let [x, y, w, h] = c;
    let (product, line, ground, ink) = *b;
    ctx.set_fill_style(ground);
    ctx.fill_rect(x, y, w, h);
    // a stripe and a burst
    ctx.set_fill_style("rgba(255,255,255,0.14)");
    ctx.fill_rect(x, y + h * 0.70, w, h * 0.10);
    ctx.set_fill_style(ink);
    ctx.set_text_align("center");
    ctx.set_text_baseline("middle");
    fit_text(ctx, product, x + w * 0.5, y + h * 0.40, w * 0.78, 40.0, "bold Futura");
    fit_text(ctx, line, x + w * 0.5, y + h * 0.75, w * 0.6, 13.0, "bold Helvetica");
    fit_text(ctx, "1.86 OZ", x + w * 0.88, y + h * 0.88, w * 0.2, 9.0, "Helvetica");
    foil(ctx, x, y, w, h, 8);
    // the crimped ends at left and right
    for x0 in [x, x + w - w * 0.05] {
        let mut xx = x0 + 1.0;
        while xx < x0 + w * 0.05 {
            ctx.set_fill_style("rgba(255,255,255,0.2)");
            ctx.fill_rect(xx, y, 1.0, h);
            ctx.set_fill_style("rgba(0,0,0,0.22)");
            ctx.fill_rect(xx + 1.5, y, 1.0, h);
            xx += 4.0;
        }
    }
}

fn paint_pastry(ctx: &mut Canvas, c: [f64; 4], p: &(&str, &str, &str, &str)) {
    let [x, y, w, h] = c;
    let (product, line, ground, ink) = *p;
    // clear film over the product: the product's colour, a printed band
    ctx.set_fill_style(ground);
    ctx.fill_rect(x, y, w, h);
    let g = ctx.create_radial_gradient(x + w / 2.0, y + h / 2.0, 5.0, x + w / 2.0, y + h / 2.0, w * 0.6);
    g.add_color_stop(0.0, "rgba(200,140,60,0.55)");
    g.add_color_stop(1.0, "rgba(120,70,20,0.15)");
    ctx.set_fill_style(&g);
    ctx.begin_path();
    ctx.ellipse(x + w / 2.0, y + h * 0.55, w * 0.42, h * 0.36, 0, 0, PI * 2.0);
    ctx.fill();
    ctx.set_fill_style(ink);
    ctx.fill_rect(x, y + h * 0.06, w, h * 0.22);
    ctx.set_fill_style(ground);
    ctx.set_text_align("center");
    ctx.set_text_baseline("middle");
    fit_text(ctx, product, x + w / 2.0, y + h * 0.17, w * 0.9, 26.0, "bold Rockwell");
    ctx.set_fill_style(ink);
    fit_text(ctx, line, x + w / 2.0, y + h * 0.88, w * 0.6, 13.0, "bold Helvetica");
    // the film's shine
    ctx.set_fill_style("rgba(255,255,255,0.18)");
    ctx.fill_rect(x + w * 0.12, y, w * 0.08, h);
}

/// The almond water's label: the plain label of the bottles found on the
/// floors, wrapped round the bottle
fn paint_almond(ctx: &mut Canvas, c: [f64; 4]) {
    let [x, y, w, h] = c;
    ctx.set_fill_style("#e8e4d8");
    ctx.fill_rect(x, y, w, h);
    ctx.set_fill_style("#3a5a6a");
    ctx.fill_rect(x, y + h * 0.10, w, h * 0.08);
    ctx.fill_rect(x, y + h * 0.82, w, h * 0.08);
    ctx.set_fill_style("#2a3a44");
    ctx.set_text_align("center");
    ctx.set_text_baseline("middle");
    for k in [0.25, 0.75] {
        fit_text(ctx, "ALMOND WATER", x + w * k, y + h * 0.45, w * 0.44, 22.0, "bold Gill Sans");
        fit_text(ctx, "16.9 FL OZ", x + w * k, y + h * 0.66, w * 0.3, 11.0, "Gill Sans");
    }
}

/// Two batteries on a blister card
fn paint_battery(ctx: &mut Canvas, c: [f64; 4]) {
    let [x, y, w, h] = c;
    ctx.set_fill_style("#1a1a1a");
    ctx.fill_rect(x, y, w, h);
    ctx.set_fill_style("#c89820");
    ctx.fill_rect(x, y, w, h * 0.16);
    ctx.set_fill_style("#1a1a1a");
    ctx.set_text_align("center");
    ctx.set_text_baseline("middle");
    fit_text(ctx, "ALKALINE", x + w / 2.0, y + h * 0.08, w * 0.8, 16.0, "bold Helvetica");
    // the cells under the blister
    for k in 0..2 {
        let cx = x + w * (0.3 + 0.4 * k as f64);
        let g = ctx.create_linear_gradient(cx - w * 0.14, 0, cx + w * 0.14, 0);
        g.add_color_stop(0.0, "#6a5010");
        g.add_color_stop(0.45, "#e0b840");
        g.add_color_stop(1.0, "#5a400c");
        ctx.set_fill_style(&g);
        rr(ctx, cx - w * 0.14, y + h * 0.24, w * 0.28, h * 0.56, 6.0);
        ctx.fill();
        ctx.set_fill_style("#222");
        ctx.fill_rect(cx - w * 0.14, y + h * 0.60, w * 0.28, h * 0.20);
    }
    ctx.set_fill_style("#e8e0c0");
    fit_text(ctx, "2 PACK  D", x + w / 2.0, y + h * 0.90, w * 0.8, 14.0, "bold Helvetica");
}

/// The goods' atlas (once a process)
pub fn goods_atlas() -> Tex {
    thread_local! { static T: std::cell::OnceCell<Tex> = const { std::cell::OnceCell::new() }; }
    T.with(|t| {
        t.get_or_init(|| {
            canvas_tex_opt(ATLAS, ATLAS, false, |ctx, _w, _h| {
                ctx.set_fill_style("#808080");
                ctx.fill_rect(0, 0, ATLAS, ATLAS);
                let d = designs();
                let (mut bi, mut ci, mut pi) = (0, 0, 0);
                for de in &d {
                    match de.kind {
                        Ware::Bag => {
                            paint_bag(ctx, de.cell, &BAGS[bi]);
                            bi += 1;
                        }
                        Ware::Bar => {
                            paint_bar(ctx, de.cell, &BARS[ci]);
                            ci += 1;
                        }
                        Ware::Pastry => {
                            paint_pastry(ctx, de.cell, &PASTRIES[pi]);
                            pi += 1;
                        }
                        Ware::Almond => paint_almond(ctx, de.cell),
                        Ware::Battery => paint_battery(ctx, de.cell),
                    }
                }
            })
        })
        .clone()
    })
}

// ------------------------------------------------------------ price strips
/// The price strips' atlas: one strip a tray (1024 × 32), each coil's code
/// and price under it, on the machine's paper label stock
pub const STRIP_H: f64 = 32.0;
pub fn strips_tex(codes: &[Vec<(String, String)>]) -> Tex {
    let rows = codes.len().max(1) as f64;
    let codes = codes.to_vec();
    canvas_tex_opt(1024, STRIP_H * rows, false, move |ctx, w, _h| {
        for (t, row) in codes.iter().enumerate() {
            let y = t as f64 * STRIP_H;
            ctx.set_fill_style("#1c1c1c");
            ctx.fill_rect(0, y, w, STRIP_H);
            let n = row.len().max(1) as f64;
            for (i, (code, price)) in row.iter().enumerate() {
                let cw = w / n;
                let x = i as f64 * cw;
                // a white tab with the code, a black one with the price beside it
                ctx.set_fill_style("#e8e6dc");
                ctx.fill_rect(x + cw * 0.20, y + 5.0, cw * 0.20, STRIP_H - 10.0);
                ctx.set_fill_style("#0c0c0c");
                ctx.fill_rect(x + cw * 0.42, y + 5.0, cw * 0.38, STRIP_H - 10.0);
                ctx.set_text_baseline("middle");
                ctx.set_text_align("center");
                ctx.set_fill_style("#141414");
                ctx.set_font("bold 16px Helvetica");
                ctx.fill_text(code, x + cw * 0.30, y + STRIP_H / 2.0);
                ctx.set_fill_style("#f0f0e8");
                ctx.fill_text(price, x + cw * 0.61, y + STRIP_H / 2.0);
            }
        }
    })
}

// ------------------------------------------------------------ the display
/// The display's 5 × 7 font: rows top first, 5 bits a row (bit 4 left)
fn glyph(c: char) -> [u8; 7] {
    match c {
        'A' => [14, 17, 17, 31, 17, 17, 17],
        'B' => [30, 17, 17, 30, 17, 17, 30],
        'C' => [14, 17, 16, 16, 16, 17, 14],
        'D' => [30, 17, 17, 17, 17, 17, 30],
        'E' => [31, 16, 16, 30, 16, 16, 31],
        'F' => [31, 16, 16, 30, 16, 16, 16],
        'G' => [14, 17, 16, 23, 17, 17, 15],
        'H' => [17, 17, 17, 31, 17, 17, 17],
        'I' => [14, 4, 4, 4, 4, 4, 14],
        'J' => [7, 2, 2, 2, 2, 18, 12],
        'K' => [17, 18, 20, 24, 20, 18, 17],
        'L' => [16, 16, 16, 16, 16, 16, 31],
        'M' => [17, 27, 21, 21, 17, 17, 17],
        'N' => [17, 17, 25, 21, 19, 17, 17],
        'O' => [14, 17, 17, 17, 17, 17, 14],
        'P' => [30, 17, 17, 30, 16, 16, 16],
        'Q' => [14, 17, 17, 17, 21, 18, 13],
        'R' => [30, 17, 17, 30, 20, 18, 17],
        'S' => [15, 16, 16, 14, 1, 1, 30],
        'T' => [31, 4, 4, 4, 4, 4, 4],
        'U' => [17, 17, 17, 17, 17, 17, 14],
        'V' => [17, 17, 17, 17, 17, 10, 4],
        'W' => [17, 17, 17, 21, 21, 21, 10],
        'X' => [17, 17, 10, 4, 10, 17, 17],
        'Y' => [17, 17, 10, 4, 4, 4, 4],
        'Z' => [31, 1, 2, 4, 8, 16, 31],
        '0' => [14, 17, 19, 21, 25, 17, 14],
        '1' => [4, 12, 4, 4, 4, 4, 14],
        '2' => [14, 17, 1, 2, 4, 8, 31],
        '3' => [31, 2, 4, 2, 1, 17, 14],
        '4' => [2, 6, 10, 18, 31, 2, 2],
        '5' => [31, 16, 30, 1, 1, 17, 14],
        '6' => [6, 8, 16, 30, 17, 17, 14],
        '7' => [31, 1, 2, 4, 8, 8, 8],
        '8' => [14, 17, 17, 14, 17, 17, 14],
        '9' => [14, 17, 17, 15, 1, 2, 12],
        '.' => [0, 0, 0, 0, 0, 12, 12],
        '-' => [0, 0, 0, 31, 0, 0, 0],
        '$' => [4, 15, 20, 14, 5, 30, 4],
        '\'' => [4, 4, 8, 0, 0, 0, 0],
        '?' => [14, 17, 1, 2, 4, 0, 4],
        ':' => [0, 12, 12, 0, 12, 12, 0],
        _ => [0; 7],
    }
}

/// Columns of the display (10 characters of 5 dots, a dot apart)
pub const DISPLAY_CHARS: usize = 10;
/// Pixels a dot, and the texture's size
const DOT: f64 = 8.0;
pub const DISPLAY_W: f64 = (DISPLAY_CHARS * 6 + 1) as f64 * DOT;
pub const DISPLAY_H: f64 = 9.0 * DOT;

/// Paints `text` (ten characters from `scroll`) on the display: lit
/// dots red-orange with a glow, the unlit dots faintly there behind the
/// smoked window; `on` 0..1 is how much the dots are lit (0: dead).
pub fn paint_display(ctx: &mut Canvas, text: &str, scroll: usize, on: f64) {
    ctx.set_fill_style("#060302");
    ctx.fill_rect(0, 0, DISPLAY_W, DISPLAY_H);
    let chars: Vec<char> = text.chars().collect();
    let lit_rgb = (255.0 * on, 70.0 * on + 8.0, 24.0 * on + 4.0);
    for col in 0..DISPLAY_CHARS {
        let c = chars.get(col + scroll).copied().unwrap_or(' ');
        let g = glyph(c);
        for (row, bits) in g.iter().enumerate() {
            for b in 0..5 {
                let lit = bits & (1 << (4 - b)) != 0;
                let x = ((col * 6 + 1 + b) as f64 + 0.5) * DOT;
                let y = ((row + 1) as f64 + 0.5) * DOT;
                if lit && on > 0.0 {
                    ctx.set_fill_style(format!("rgba({:.0},{:.0},{:.0},0.30)", lit_rgb.0, lit_rgb.1, lit_rgb.2));
                    ctx.begin_path();
                    ctx.arc(x, y, DOT * 0.75, 0, PI * 2.0);
                    ctx.fill();
                    ctx.set_fill_style(format!("rgb({:.0},{:.0},{:.0})", lit_rgb.0, lit_rgb.1, lit_rgb.2));
                } else {
                    ctx.set_fill_style("#1a0a06");
                }
                ctx.begin_path();
                ctx.arc(x, y, DOT * 0.36, 0, PI * 2.0);
                ctx.fill();
            }
        }
    }
}

// ------------------------------------------------------------ glass, stickers
/// The glass's grime: a clear pane with finger smears at hand height, a
/// film of dust thicker at the bottom, fine scratches, and a soft
/// reflection of the room's light (the renderer has no reflections)
pub fn glass_tex() -> Tex {
    canvas_tex_opt(256, 448, false, |ctx, w, h| {
        ctx.clear_rect(0, 0, w, h);
        // a broad soft reflection, brighter at the top
        let g = ctx.create_linear_gradient(0, 0, w, h);
        g.add_color_stop(0.0, "rgba(255,255,255,0.10)");
        g.add_color_stop(0.4, "rgba(255,255,255,0.02)");
        g.add_color_stop(0.55, "rgba(255,255,255,0.07)");
        g.add_color_stop(1.0, "rgba(255,255,255,0.0)");
        ctx.set_fill_style(&g);
        ctx.fill_rect(0, 0, w, h);
        // dust, settling toward the bottom
        let d = ctx.create_linear_gradient(0, 0, 0, h);
        d.add_color_stop(0.0, "rgba(200,190,170,0.04)");
        d.add_color_stop(1.0, "rgba(200,190,170,0.14)");
        ctx.set_fill_style(&d);
        ctx.fill_rect(0, 0, w, h);
        // smears where hands rest and point (the middle third)
        for _ in 0..14 {
            let (x, y) = (rand2(0.15, 0.95) * w, rand2(0.35, 0.8) * h);
            let r = rand2(6.0, 16.0);
            ctx.set_fill_style(format!("rgba(230,225,210,{:.3})", rand2(0.05, 0.12)));
            ctx.begin_path();
            ctx.ellipse(x, y, r, r * 1.3, rand2(-0.5, 0.5), 0, PI * 2.0);
            ctx.fill();
        }
        ctx.set_line_width(0.7);
        for _ in 0..40 {
            let (x, y) = (rnd() * w, rnd() * h);
            let a = rand2(-0.6, 0.6);
            let l = rand2(6.0, 40.0);
            ctx.set_stroke_style(format!("rgba(255,255,255,{:.3})", rand2(0.08, 0.22)));
            ctx.begin_path();
            ctx.move_to(x, y);
            ctx.line_to(x + a.cos() * l, y + a.sin() * l);
            ctx.stroke();
        }
    })
}

/// The door's stickers, as cells of `decal_tex` (x, y, w, h in its pixels)
pub const EXACT: [f64; 4] = [0.0, 0.0, 128.0, 64.0];
pub const PUSH: [f64; 4] = [128.0, 0.0, 128.0, 64.0];
pub const SERVICE: [f64; 4] = [0.0, 64.0, 256.0, 64.0];
pub const SELECT: [f64; 4] = [0.0, 128.0, 128.0, 24.0];
pub const RETURN: [f64; 4] = [0.0, 152.0, 128.0, 24.0];
pub const BILLS: [f64; 4] = [128.0, 128.0, 128.0, 48.0];
pub const NEVER: [f64; 4] = [0.0, 176.0, 256.0, 80.0];
/// A cell of the 256 px decal texture in UV space (flipped in y)
pub fn decal_uv(c: [f64; 4]) -> [f64; 4] {
    [c[0] / 256.0, 1.0 - (c[1] + c[3]) / 256.0, c[2] / 256.0, c[3] / 256.0]
}

/// The door's stickers: the coin instructions under the coin entry (USE
/// EXACT CHANGE), the flap's PUSH, the operator's service sticker with a
/// telephone number, the panel's legends, and the industry's warning label
/// against rocking (after the CPSC campaign of 1995: rule 7's warning)
pub fn decal_tex() -> Tex {
    canvas_tex_opt(256, 256, false, |ctx, _w, _h| {
        ctx.clear_rect(0, 0, 256, 256);
        ctx.set_text_align("center");
        ctx.set_text_baseline("middle");
        let [x, y, w, h] = EXACT;
        ctx.set_fill_style("#d8d2c0");
        rr(ctx, x + 2.0, y + 2.0, w - 4.0, h - 4.0, 6.0);
        ctx.fill();
        ctx.set_fill_style("#a01010");
        ctx.fill_rect(x + 2.0, y + 2.0, w - 4.0, 18);
        ctx.set_fill_style("#f0ece0");
        ctx.set_font("bold 13px Helvetica");
        ctx.fill_text("USE EXACT CHANGE", x + w / 2.0, y + 11.0);
        ctx.set_fill_style("#1a1a1a");
        ctx.set_font("bold 11px Helvetica");
        ctx.fill_text("NICKELS DIMES QUARTERS", x + w / 2.0, y + 32.0);
        ctx.fill_text("NO PENNIES", x + w / 2.0, y + 48.0);
        let [x, y, w, h] = PUSH;
        ctx.set_fill_style("rgba(235,230,215,0.92)");
        ctx.set_font("bold 44px Helvetica");
        ctx.fill_text("PUSH", x + w / 2.0, y + h / 2.0 + 2.0);
        let [x, y, w, h] = SERVICE;
        ctx.set_fill_style("#e6e0c8");
        rr(ctx, x + 4.0, y + 4.0, w - 8.0, h - 8.0, 4.0);
        ctx.fill();
        ctx.set_fill_style("#1a3a6a");
        ctx.fill_rect(x + 4.0, y + 4.0, w - 8.0, 16);
        ctx.set_fill_style("#f2efe4");
        ctx.set_font("bold 12px Helvetica");
        ctx.fill_text("FOR SERVICE OR REFUND CALL", x + w / 2.0, y + 12.0);
        ctx.set_fill_style("#1a1a1a");
        ctx.set_font("bold 22px Helvetica");
        ctx.fill_text("1-800-555-0147", x + w / 2.0, y + 40.0);
        ctx.set_fill_style("#e8e4d8");
        ctx.set_font("bold 13px Helvetica");
        let [x, y, w, h] = SELECT;
        ctx.fill_text("MAKE SELECTION", x + w / 2.0, y + h / 2.0);
        let [x, y, w, h] = RETURN;
        ctx.fill_text("COIN RETURN", x + w / 2.0, y + h / 2.0);
        let [x, y, w, _] = BILLS;
        ctx.fill_text("$1   $5", x + w / 2.0, y + 14.0);
        ctx.set_font("bold 11px Helvetica");
        ctx.fill_text("FACE UP", x + w / 2.0, y + 34.0);
        // the warning label: yellow, a black band, a pictogram of a falling machine
        let [x, y, w, h] = NEVER;
        ctx.set_fill_style("#e2c12a");
        ctx.fill_rect(x + 2.0, y + 2.0, w - 4.0, h - 4.0);
        ctx.set_fill_style("#141414");
        ctx.fill_rect(x + 2.0, y + 2.0, w - 4.0, 18);
        ctx.set_fill_style("#e2c12a");
        ctx.set_font("bold 14px Helvetica");
        ctx.fill_text("WARNING", x + w / 2.0 + 18.0, y + 11.0);
        // the pictogram: a machine leaning over a figure
        ctx.set_fill_style("#141414");
        ctx.save();
        ctx.translate(x + 26.0, y + 70.0);
        ctx.rotate(-0.45);
        ctx.fill_rect(-9, -40, 18, 40);
        ctx.restore();
        ctx.begin_path();
        ctx.arc(x + 40.0, y + 52.0, 4, 0, PI * 2.0);
        ctx.fill();
        ctx.fill_rect(x + 37.0, y + 56.0, 6, 14);
        ctx.set_text_align("left");
        ctx.set_font("bold 10px Helvetica");
        for (i, l) in
            ["NEVER ROCK OR TILT.", "MACHINE WILL NOT DISPENSE", "FREE PRODUCT. TIPPING CAN", "CAUSE INJURY OR DEATH."].iter().enumerate()
        {
            ctx.fill_text(l, x + 58.0, y + 30.0 + i as f64 * 12.0);
        }
    })
}

/// The keypad's legends: one cell of 32 px a key, four to a row (21 keys
/// in six rows)
pub fn keys_tex() -> Tex {
    canvas_tex_opt(128, 256, false, |ctx, _w, _h| {
        ctx.clear_rect(0, 0, 128, 256);
        ctx.set_text_align("center");
        ctx.set_text_baseline("middle");
        ctx.set_fill_style("#1a1a1a");
        ctx.set_font("bold 20px Helvetica");
        for (i, k) in super::model::KEYS.iter().enumerate() {
            let (cx, cy) = ((i % 4) as f64 * 32.0 + 16.0, (i / 4) as f64 * 32.0 + 17.0);
            ctx.fill_text(k, cx, cy);
        }
    })
}

/// Rust and scuffs along the bottom of a door, where feet and mops and
/// carts have caught it for years (thicker toward the bottom edge)
pub fn rust_tex() -> Tex {
    canvas_tex_opt(256, 64, false, |ctx, w, h| {
        ctx.clear_rect(0, 0, w, h);
        for _ in 0..70 {
            let (x, y) = (rnd() * w, h - rnd() * rnd() * h);
            let r = rand2(1.5, 7.0);
            let warm = rnd() < 0.6;
            let a = rand2(0.15, 0.55) * (0.4 + 0.6 * y / h);
            ctx.set_fill_style(if warm { format!("rgba(110,58,24,{a:.3})") } else { format!("rgba(170,160,140,{:.3})", a * 0.6) });
            ctx.begin_path();
            ctx.ellipse(x, y, r * rand2(1.0, 3.0), r, rand2(-0.3, 0.3), 0, PI * 2.0);
            ctx.fill();
        }
        // scuffs: short grey strokes at kick height
        ctx.set_line_width(1.2);
        for _ in 0..24 {
            let (x, y) = (rnd() * w, rand2(0.2, 0.8) * h);
            ctx.set_stroke_style(format!("rgba(160,158,150,{:.3})", rand2(0.15, 0.4)));
            ctx.begin_path();
            ctx.move_to(x, y);
            ctx.line_to(x + rand2(6.0, 24.0), y + rand2(-2.0, 2.0));
            ctx.stroke();
        }
    })
}
