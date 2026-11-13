//! Batch `r15`'s props in this crate, made real (`docs/design/PROPS.md` P2): its
//! placers and what only they use.
use super::*;

// Paperback book: 8% chance per level. A trade paperback dropped on
// the floor (Blender batches r02 and r15, 216 x 140 mm): six in ten lie
// open face up, their pages printed in a serif and gone brown and dusty
// where they lay open; the rest shut, its printed cover worn. The old draws
// are kept call for call.
pub(super) fn place_paperback_books(g: &mut Game, _dist: &[i32]) {
    use game::assets::props::prop_part_as_modelled;
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = choice(&open);
    // the old cover colour draw: a brick red, blue, green or mustard cover
    let cover_colors = [0xa02040, 0x2050c0, 0x30a030, 0xd8b040];
    let col = choice(&cover_colors);
    let look = cover_colors.iter().position(|&c| c == col).unwrap_or(0);
    // the old draw: six in ten lie open (the pages showing)
    let is_open = rnd() < 0.60;
    let book = group();
    scatter(g, &book, x, z, 2.4);
    let salt = r15_kit::salt(&book);
    let parts: Vec<(&str, Mat)> = if is_open {
        vec![
            ("Real__pbOpenCovers", lambert!({ color: 0xcfc7b4 })),
            ("Real__pbOpenBlock", lambert!({ map: &r15_kit::page_edges_tex() })),
            ("Real__pbOpenSpread", lambert!({ map: &r15_kit::spread_tex(look, salt) })),
        ]
    } else {
        // r02's paperback, its own cover painted: the cells of one atlas
        let atlas = r15_kit::pb_atlas(look, salt);
        let cell = |c: (f64, f64, f64, f64)| {
            let t = atlas.duplicate();
            t.set_offset((c.0 + 1.0) / 256.0, 1.0 - (c.1 + c.3 - 1.0) / 384.0);
            t.set_repeat((c.2 - 2.0) / 256.0, (c.3 - 2.0) / 384.0);
            lambert!({ map: &t })
        };
        vec![
            ("Real__bookPbCovers", cell(r15_kit::PB_COVER)),
            ("Real__bookPbSpine", cell(r15_kit::PB_SPINE)),
            ("Real__bookPbPages", cell(r15_kit::PB_PAGES)),
        ]
    };
    for (name, mat) in &parts {
        if let Some(geo) = prop_part_as_modelled(name) {
            book.add(&mesh(geo, mat));
        }
    }
    g.world.add(&book);
}

// Tape roll: 10% chance per level. A roll of PVC insulating tape,
// half used (Blender batch r15: 19 mm wide on a 1.5 in paperboard core,
// 60 mm across), lying on its face, its loose end peeled up and curled;
// lint and grit stuck to its edges. The old draws are kept call for call.
pub(super) fn place_tape_rolls(g: &mut Game, _dist: &[i32]) {
    use game::assets::props::prop_part_as_modelled;
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = choice(&open);
    // the old colour draw: white, red, yellow or blue tape
    let tape_colors = [0xf0ece0, 0xa02040, 0xd8b040, 0x2050c0];
    let col = choice(&tape_colors);
    let look = tape_colors.iter().position(|&c| c == col).unwrap_or(0);
    let tape = group();
    scatter(g, &tape, x, z, 2.4);
    let salt = r15_kit::salt(&tape);
    let pvc = r15_kit::PVC[look];
    let vinyl = |map: Tex| phong!({ map: &map, color: 0xffffff, specular: 0x1c1c1c, shininess: 34 });
    let parts = [
        ("Real__tapeRollFaces", vinyl(r15_kit::tape_faces_tex(look, salt))),
        ("Real__tapeRollWrap", vinyl(r15_kit::tape_wrap_tex(look, salt))),
        ("Real__tapeRollCore", lambert!({ map: &r15_kit::tape_core_tex(salt) })),
        ("Real__tapeRollTail", phong!({ color: r15_kit::hex(pvc, 0.9), specular: 0x1c1c1c, shininess: 34 })),
    ];
    for (name, mat) in &parts {
        if let Some(geo) = prop_part_as_modelled(name) {
            tape.add(&mesh(geo, mat));
        }
    }
    g.world.add(&tape);
}

// Crumpled receipt: 10% chance per level. A till receipt (80 mm
// thermal paper) balled in a hand and dropped: batch r08's balled sheet
// (`place_scrap_papers`), the size a strip of receipt makes of it, its
// thermal print gone brown and faded. The old draws are kept call for call.
pub(super) fn place_crumpled_receipts(g: &mut Game, _dist: &[i32]) {
    use game::assets::props::prop_part_as_modelled;
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = choice(&open);
    let receipt = group();
    scatter(g, &receipt, x, z, 2.4);
    let salt = r15_kit::salt(&receipt);
    // a strip of 80 x 200 mm is a quarter of A4: its ball a little over half
    // as big (a crumpled ball's size goes as its area to the 0.4)
    let (name, k) = if salt % 3 == 0 { ("Real__paperWad", 0.55) } else { ("Real__paperBall", 0.57) };
    let geo = prop_part_as_modelled(name).unwrap_or_else(|| box_geo!(0.040, 0.025, 0.040));
    let ball = mesh(geo, &lambert!({ map: &r15_kit::receipt_tex(salt) }));
    ball.scale().set(k, k, k);
    receipt.add(&ball);
    g.world.add(&receipt);
}

/// Batch r15's painters for the tape roll, the crumpled receipt and the
/// paperback (`place_tape_rolls`, `place_crumpled_receipts`,
/// `place_paperback_books`): each seeded by where its thing lies, so it
/// differs floor to floor without a level draw.
mod r15_kit {
    use game::prelude::*;

    /// A salt from where a placed group lies
    pub fn salt(o: &Obj) -> u32 {
        let p = o.pos();
        ((p.x * 977.0).floor() as i64 ^ ((p.z * 1181.0).floor() as i64).rotate_left(13)) as u32
    }

    use placers_kit::paint::rgba;

    /// An sRGB colour as a hex, darkened by `k`
    pub fn hex(c: (f64, f64, f64), k: f64) -> u32 {
        let q = |v: f64| ((v * k).round().clamp(0.0, 255.0)) as u32;
        (q(c.0) << 16) | (q(c.1) << 8) | q(c.2)
    }

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

    // ------------------------------------------------------------ the tape
    /// PVC tape gone dull (sRGB): white, red, yellow, blue
    pub const PVC: [(f64, f64, f64); 4] = [(210.0, 204.0, 190.0), (138.0, 38.0, 34.0), (186.0, 150.0, 46.0), (42.0, 66.0, 122.0)];

    /// The roll's wound faces (mapped as a disc: the tape from 0.72 of the
    /// radius out): the turns' fine rings, grit and lint stuck to the
    /// adhesive edges, thickest at the outer turns
    pub fn tape_faces_tex(look: usize, salt: u32) -> Tex {
        canvas_tex_opt(256, 256, false, move |ctx, w, h| {
            let mut r = SplitMix64::new(salt, 0x7a9e);
            let c = PVC[look];
            ctx.set_fill_style(rgba(c, 1.0));
            ctx.fill_rect(0, 0, w, h);
            let (cx, cy) = (w / 2.0, h / 2.0);
            ctx.set_line_width(0.7);
            let mut rr = 92.0;
            while rr < 128.0 {
                let dark = r.next() < 0.5;
                ctx.set_stroke_style(if dark {
                    rgba((0.0, 0.0, 0.0), 0.05 + 0.08 * r.next())
                } else {
                    rgba((255.0, 250.0, 240.0), 0.04 + 0.06 * r.next())
                });
                ctx.begin_path();
                ctx.arc(cx, cy, rr, 0, 7);
                ctx.stroke();
                rr += 0.8 + r.next() * 1.4;
            }
            // where the roll was put down and picked up: a turn or two darker
            for _ in 0..2 {
                let rr = 96.0 + r.next() * 28.0;
                ctx.set_stroke_style("rgba(30,24,18,0.22)");
                ctx.set_line_width(1.1);
                ctx.begin_path();
                ctx.arc(cx, cy, rr, 0, 7);
                ctx.stroke();
            }
            // grime on the adhesive edges: a band at the outer turns, patchy
            let gr = ctx.create_radial_gradient(cx, cy, 104, cx, cy, 128);
            gr.add_color_stop(0, "rgba(58,48,36,0)");
            gr.add_color_stop(0.6, "rgba(58,48,36,0.25)");
            gr.add_color_stop(1, "rgba(52,42,32,0.62)");
            ctx.set_fill_style(&gr);
            ctx.fill_rect(0, 0, w, h);
            for _ in 0..60 {
                let a = r.next() * PI * 2.0;
                let rr = 96.0 + r.next() * 32.0;
                soft(ctx, (cx + a.cos() * rr, cy + a.sin() * rr), 3.0 + r.next() * 9.0, 0.6, a, (60.0, 50.0, 38.0), 0.15 + 0.3 * r.next());
            }
            // lint: fibres and a hair caught on the glue
            ctx.set_line_width(0.6);
            for _ in 0..14 {
                let a = r.next() * PI * 2.0;
                let rr = 95.0 + r.next() * 30.0;
                let (x, y) = (cx + a.cos() * rr, cy + a.sin() * rr);
                let b = r.next() * PI * 2.0;
                let l = 4.0 + r.next() * 16.0;
                ctx.set_stroke_style(rgba(if r.next() < 0.5 { (70.0, 64.0, 58.0) } else { (190.0, 184.0, 172.0) }, 0.6));
                ctx.begin_path();
                ctx.move_to(x, y);
                ctx.quadratic_curve_to(x + (b + 0.6).cos() * l * 0.6, y + (b + 0.6).sin() * l * 0.6, x + b.cos() * l, y + b.sin() * l);
                ctx.stroke();
            }
            // and the floor's dust over the face it lay on its back on
            for _ in 0..200 {
                let (x, y) = (r.next() * w, r.next() * h);
                ctx.set_fill_style(rgba((96.0, 86.0, 70.0), 0.2 + 0.3 * r.next()));
                ctx.fill_rect(x, y, 1.0, 1.0);
            }
        })
    }

    /// The outer wrap (u round it, v up it): grime at both edges, scuffs
    /// where it rolled, the floor's dust
    pub fn tape_wrap_tex(look: usize, salt: u32) -> Tex {
        canvas_tex_opt(256, 32, false, move |ctx, w, h| {
            let mut r = SplitMix64::new(salt, 0x3c1d);
            ctx.set_fill_style(rgba(PVC[look], 1.0));
            ctx.fill_rect(0, 0, w, h);
            for (y0, y1) in [(0.0, 7.0), (h, h - 7.0)] {
                let gr = ctx.create_linear_gradient(0, y0, 0, y1);
                gr.add_color_stop(0, "rgba(52,42,32,0.55)");
                gr.add_color_stop(1, "rgba(52,42,32,0)");
                ctx.set_fill_style(&gr);
                ctx.fill_rect(0, y0.min(y1), w, 7);
            }
            for _ in 0..30 {
                let (x, y) = (r.next() * w, r.next() * h);
                ctx.set_fill_style(rgba((240.0, 236.0, 226.0), 0.08 + 0.1 * r.next()));
                ctx.fill_rect(x, y, 4.0 + r.next() * 18.0, 0.8);
            }
            for _ in 0..120 {
                let (x, y) = (r.next() * w, r.next() * h);
                ctx.set_fill_style(rgba((90.0, 80.0, 66.0), 0.2 + 0.3 * r.next()));
                ctx.fill_rect(x, y, 1.0, 1.0);
            }
            // the outer turn's cut end, a faint step across the wrap
            ctx.set_fill_style("rgba(20,16,12,0.25)");
            ctx.fill_rect(2, 0, 1.2, h);
        })
    }

    /// The paperboard core: grey-brown kraft, mottled, its ends dark with
    /// handling
    pub fn tape_core_tex(salt: u32) -> Tex {
        canvas_tex_opt(128, 64, false, move |ctx, w, h| {
            let mut r = SplitMix64::new(salt, 0xc02e);
            ctx.set_fill_style("#8e7656");
            ctx.fill_rect(0, 0, w, h);
            for _ in 0..90 {
                let at = (r.next() * w, r.next() * h);
                soft(
                    ctx,
                    at,
                    3.0 + r.next() * 8.0,
                    0.5,
                    0.0,
                    if r.next() < 0.5 { (70.0, 56.0, 40.0) } else { (170.0, 150.0, 120.0) },
                    0.15 + 0.2 * r.next(),
                );
            }
            for (y0, y1) in [(0.0, 10.0), (h, h - 10.0)] {
                let gr = ctx.create_linear_gradient(0, y0, 0, y1);
                gr.add_color_stop(0, "rgba(40,32,24,0.6)");
                gr.add_color_stop(1, "rgba(40,32,24,0)");
                ctx.set_fill_style(&gr);
                ctx.fill_rect(0, y0.min(y1), w, 10);
            }
        })
    }

    // --------------------------------------------------------- the receipt
    const TILL: [&str; 22] = [
        "TAKEAWAY",
        "TABLE 4",
        "COFFEE WHITE      1.80",
        "TEA               1.20",
        "TOAST & JAM       1.60",
        "SOUP OF THE DAY   2.95",
        "ROLL BUTTERED     0.85",
        "WATER 500ML       0.90",
        "2 x BISCUIT       0.60",
        "SCONE             1.40",
        "SUBTOTAL          9.75",
        "TOTAL             9.75",
        "CASH             10.00",
        "CHANGE            0.25",
        "ITEMS 7",
        "THANK YOU",
        "VAT INCLUDED",
        "REG 02  OP 11",
        "14:22",
        "No. 004718",
        "--------------------",
        "********************",
    ];

    /// A till receipt balled up (laid on the ball by its box faces): the
    /// thermal paper's cool white gone cream, patches of its print turned
    /// every way, faded brown where the heat print has aged, the creases'
    /// grime, the floor's dust
    pub fn receipt_tex(salt: u32) -> Tex {
        canvas_tex_opt(256, 256, false, move |ctx, w, h| {
            let mut r = SplitMix64::new(salt, 0x7ec1);
            ctx.set_fill_style("#e2ded2");
            ctx.fill_rect(0, 0, w, h);
            for _ in 0..200 {
                let (x, y) = (r.next() * w, r.next() * h);
                let v = 214.0 + r.next() * 26.0;
                ctx.set_fill_style(format!("rgba({v:.0},{:.0},{:.0},0.16)", v - 3.0, v - 12.0));
                ctx.fill_rect(x, y, 3.0 + r.next() * 9.0, 3.0 + r.next() * 9.0);
            }
            // the print, on facets of the strip turned their own ways (its
            // back is blank: some facets have none)
            ctx.set_font("bold 9px Courier New");
            ctx.set_text_align("left");
            for _ in 0..14 {
                let (cx, cy) = (r.next() * w, r.next() * h);
                let rad = 18.0 + r.next() * 26.0;
                ctx.save();
                ctx.begin_path();
                let n = 5 + (r.next() * 3.0) as usize;
                for k in 0..n {
                    let t = (k as f64 + r.next() * 0.4) / n as f64 * PI * 2.0;
                    let rr = rad * (0.6 + 0.4 * r.next());
                    if k == 0 {
                        ctx.move_to(cx + t.cos() * rr, cy + t.sin() * rr);
                    } else {
                        ctx.line_to(cx + t.cos() * rr, cy + t.sin() * rr);
                    }
                }
                ctx.close_path();
                ctx.clip();
                ctx.translate(cx, cy);
                ctx.rotate(r.next() * PI * 2.0);
                if r.next() < 0.75 {
                    let fade = 0.35 + 0.4 * r.next();
                    let mut k = (r.next() * TILL.len() as f64) as usize;
                    let mut y = -rad;
                    while y < rad + 10.0 {
                        ctx.set_fill_style(format!("rgba(74,62,52,{fade:.2})"));
                        ctx.fill_text(TILL[k % TILL.len()], -rad - 4.0 + r.next() * 6.0, y);
                        k += 1;
                        y += 10.5;
                    }
                }
                ctx.restore();
            }
            // crease lines: grime in the valleys
            ctx.set_line_width(0.8);
            for _ in 0..44 {
                let (x, y) = (r.next() * w, r.next() * h);
                let a = r.next() * PI;
                let l = 8.0 + r.next() * 36.0;
                ctx.set_stroke_style(format!("rgba(72,64,52,{:.2})", 0.08 + r.next() * 0.14));
                ctx.begin_path();
                ctx.move_to(x, y);
                ctx.line_to(x + a.cos() * l, y + a.sin() * l);
                ctx.stroke();
            }
            for _ in 0..18 {
                let at = (r.next() * w, r.next() * h);
                soft(ctx, at, 8.0 + r.next() * 20.0, 0.5 + r.next() * 0.4, r.next() * 3.0, (96.0, 86.0, 66.0), 0.08 + r.next() * 0.14);
            }
        })
    }

    // ------------------------------------------------------- the paperback
    /// Paperback covers (sRGB), by the old colour: brick, blue, green, mustard
    const COVER: [(f64, f64, f64); 4] = [(146.0, 62.0, 52.0), (60.0, 80.0, 116.0), (72.0, 102.0, 74.0), (176.0, 146.0, 70.0)];
    const BOOKS: [(&str, &str, &str); 4] = [
        ("The Summer", "Lodgers", "MARGARET KELL"),
        ("A House by", "the Harbour", "J. R. ASHWORTH"),
        ("The Orchard", "Wall", "HELEN MORAY"),
        ("Rain at", "Easter", "DAVID LISLE"),
    ];

    /// The shut paperback's atlas (256 x 384): its front cover (the cover's
    /// head to the left, its fore-edge up, as r02's covers take it), its
    /// spine (head to the left, the front up), its page edges
    pub const PB_COVER: (f64, f64, f64, f64) = (0.0, 0.0, 256.0, 256.0);
    pub const PB_SPINE: (f64, f64, f64, f64) = (0.0, 256.0, 256.0, 32.0);
    pub const PB_PAGES: (f64, f64, f64, f64) = (0.0, 288.0, 256.0, 96.0);

    pub fn pb_atlas(look: usize, salt: u32) -> Tex {
        canvas_tex_opt(256, 384, false, move |ctx, _, _| {
            let mut r = SplitMix64::new(salt, 0xb00c);
            let (t1, t2, author) = BOOKS[(salt as usize / 7) % BOOKS.len()];
            let g = COVER[look];
            let light = (226.0, 218.0, 198.0);
            // the cover, drawn upright in its own frame (140 x 216 mm)
            ctx.save();
            ctx.translate(0, 256);
            ctx.rotate(-PI / 2.0);
            ctx.scale(256.0 / 140.0, 256.0 / 216.0);
            ctx.set_fill_style(rgba(g, 1.0));
            ctx.fill_rect(0, 0, 140, 216);
            ctx.set_fill_style(rgba(light, 1.0));
            ctx.fill_rect(10, 64, 120, 92);
            ctx.set_stroke_style(rgba(g, 0.9));
            ctx.set_line_width(0.8);
            ctx.stroke_rect(13, 67, 114, 86);
            ctx.set_text_align("center");
            ctx.set_fill_style(rgba((52.0, 44.0, 40.0), 0.95));
            ctx.set_font("bold 15px Times New Roman");
            ctx.fill_text_max(t1, 70, 100, 108);
            ctx.fill_text_max(t2, 70, 119, 108);
            ctx.set_font("italic 8px Times New Roman");
            ctx.fill_text("a novel", 70, 138);
            ctx.set_fill_style(rgba(light, 0.95));
            ctx.set_font("10px Gill Sans");
            ctx.fill_text_max(author, 70, 40, 120);
            // the publisher's mark: a small tree in a ring
            ctx.set_stroke_style(rgba(light, 0.9));
            ctx.set_line_width(0.9);
            ctx.begin_path();
            ctx.arc(70, 192, 7, 0, 7);
            ctx.stroke();
            ctx.begin_path();
            ctx.move_to(70, 197);
            ctx.line_to(70, 188);
            ctx.move_to(70, 191);
            ctx.line_to(66.5, 187.5);
            ctx.move_to(70, 190);
            ctx.line_to(73.5, 186.5);
            ctx.stroke();
            // sun-faded from the head down, a crease where it was bent back
            // at the spine, a corner across, the edges worn to the card
            let gr = ctx.create_linear_gradient(0, 0, 0, 216);
            gr.add_color_stop(0, "rgba(232,226,210,0.30)");
            gr.add_color_stop(1, "rgba(232,226,210,0.0)");
            ctx.set_fill_style(&gr);
            ctx.fill_rect(0, 0, 140, 216);
            ctx.set_stroke_style("rgba(235,228,212,0.7)");
            ctx.set_line_width(0.8);
            for x in [8.0, 11.0] {
                ctx.begin_path();
                ctx.move_to(x, 0);
                ctx.line_to(x + r.next() * 1.5, 216);
                ctx.stroke();
            }
            ctx.begin_path();
            ctx.move_to(140.0 - 20.0 - 20.0 * r.next(), 216);
            ctx.line_to(140, 216.0 - 18.0 - 20.0 * r.next());
            ctx.stroke();
            ctx.set_fill_style("rgba(232,226,210,0.4)");
            for (x, y, w, h) in [(0.0, 0.0, 140.0, 2.5), (0.0, 213.5, 140.0, 2.5), (137.5, 0.0, 2.5, 216.0)] {
                ctx.fill_rect(x, y, w, h);
            }
            // a cup's ring, and the second-hand price in pencil
            if r.next() < 0.7 {
                let (x, y, rr) = (30.0 + 80.0 * r.next(), 150.0 + 50.0 * r.next(), 18.0 + 6.0 * r.next());
                ctx.set_stroke_style("rgba(70,48,28,0.35)");
                ctx.set_line_width(1.6);
                ctx.begin_path();
                ctx.arc(x, y, rr, 0.3, 5.9);
                ctx.stroke();
            }
            ctx.set_fill_style("rgba(70,70,74,0.55)");
            ctx.set_font("italic 9px Times New Roman");
            ctx.fill_text("40p", 124, 13);
            ctx.restore();
            for _ in 0..500 {
                let (x, y) = (r.next() * 256.0, r.next() * 256.0);
                ctx.set_fill_style(if r.next() < 0.5 { "rgba(255,248,232,0.10)" } else { "rgba(0,0,0,0.10)" });
                ctx.fill_rect(x, y, 1.0, 1.0);
            }
            // the spine (256 x 32: head at the left, the front cover up)
            let (sx, sy) = (PB_SPINE.0, PB_SPINE.1);
            ctx.set_fill_style(rgba(g, 1.0));
            ctx.fill_rect(sx, sy, 256, 32);
            ctx.set_fill_style(rgba(light, 0.95));
            ctx.set_text_align("left");
            ctx.set_text_baseline("middle");
            ctx.set_font("bold 10px Times New Roman");
            ctx.fill_text_max(&format!("{t1} {t2}"), sx + 14.0, sy + 16.0, 140);
            ctx.set_font("8px Gill Sans");
            ctx.fill_text_max(author, sx + 168.0, sy + 16.0, 64);
            ctx.set_text_baseline("alphabetic");
            ctx.set_stroke_style("rgba(236,230,214,0.7)");
            ctx.set_line_width(0.8);
            for _ in 0..(3.0 + r.next() * 4.0) as usize {
                let y = sy + 4.0 + r.next() * 24.0;
                let x = sx + r.next() * 50.0;
                ctx.begin_path();
                ctx.move_to(x, y);
                ctx.line_to(x + 80.0 + r.next() * 120.0, y + r.next() * 2.0 - 1.0);
                ctx.stroke();
            }
            let gr = ctx.create_linear_gradient(0, sy, 0, sy + 8.0);
            gr.add_color_stop(0, "rgba(150,140,120,0.4)");
            gr.add_color_stop(1, "rgba(150,140,120,0)");
            ctx.set_fill_style(&gr);
            ctx.fill_rect(sx, sy, 256, 8);
            page_edges(ctx, PB_PAGES.0, PB_PAGES.1, PB_PAGES.2, PB_PAGES.3, &mut r);
        })
    }

    /// Page edges: the leaves' fine lines, tanned darker toward the covers,
    /// foxed, grimy
    fn page_edges(ctx: &mut Canvas, x0: f64, y0: f64, w: f64, h: f64, r: &mut SplitMix64) {
        ctx.set_fill_style("#cdbf9c");
        ctx.fill_rect(x0, y0, w, h);
        let mut y = y0;
        while y < y0 + h {
            ctx.set_fill_style(format!("rgba(90,70,40,{:.2})", 0.05 + 0.12 * r.next()));
            ctx.fill_rect(x0, y, w, 0.6 + r.next() * 0.8);
            y += 1.0 + r.next() * 2.0;
        }
        for (ya, yb) in [(y0, y0 + h * 0.15), (y0 + h, y0 + h * 0.85)] {
            let gr = ctx.create_linear_gradient(x0, ya, x0, yb);
            gr.add_color_stop(0, "rgba(120,86,40,0.45)");
            gr.add_color_stop(1, "rgba(120,86,40,0)");
            ctx.set_fill_style(&gr);
            ctx.fill_rect(x0, y0, w, h);
        }
        for _ in 0..40 {
            let (x, y) = (x0 + r.next() * w, y0 + r.next() * h);
            soft(ctx, (x, y), 1.0 + r.next() * 3.0, 1.0, 0.0, (130.0, 84.0, 40.0), 0.35);
        }
    }

    /// The open book's page edges (its halves' head, tail and fore-edge)
    pub fn page_edges_tex() -> Tex {
        canvas_tex_opt(128, 64, false, |ctx, w, h| {
            let mut r = SplitMix64::new(0x9e37, 0xed9e);
            page_edges(ctx, 0.0, 0.0, w, h, &mut r);
            ctx.set_fill_style("rgba(120,110,94,0.25)");
            ctx.fill_rect(0, 0, w, h);
        })
    }

    /// The text the open book is open at: a chapter of a quiet novel
    /// (written for the game), two pages of it
    const PROSE: [&str; 13] = [
        "had boiled twice before Mrs Pryor came down, and by then the light had moved off the table and onto the dresser, where it found the row of blue plates and stayed there. She took the end chair, as she always did, and read the post with her glasses pushed up into her hair. There was a card from her sister in Whitby, a bill from the coal merchant, and a long envelope that she set aside without opening.",
        "\u{2018}You\u{2019}ll want to see to the gutters before the weather turns,\u{2019} she said. \u{2018}Tom Ellery said he\u{2019}d come on Saturday, but he says that every year.\u{2019}",
        "I said I would ask him again on my way to the station. She nodded and poured the tea, and for a while neither of us said anything. Outside, a cart went by with milk churns rattling on the back of it, and the dog from next door ran after it as far as the corner and then thought better of it.",
        "In those years the house took lodgers from April to the end of September. Most of them came back each summer, so that by the third or fourth year we knew which of them liked the window open at night and which of them wanted an egg done hard. Mr Petrie, who taught music at a school somewhere in the Midlands, came for the whole of August and practised scales in the front room every morning between nine and ten. Nobody minded. The Coopers came with their two boys and a great deal of luggage, and stayed a fortnight, and left sand in every room of the house.",
        "My own room was at the top, under the slope of the roof, and from the small window I could see the harbour wall and, on a clear day, the line of the far shore. I had a table there, and a chair that had come out of the kitchen, and I did my reading in the evenings when the lodgers had gone out to the pier or to the pictures. Mrs Pryor had given me a lamp with a green shade, and it is the light of that lamp I remember best from those summers, more than the sea or the long afternoons.",
        "On Saturdays I went into town for her. The list was always the same, give or take a few things: bread, a pound of sugar, soap, a tin of peas, a quarter of tea, string. I would walk the long way back along the front, with the bag cutting into my fingers, and stop at the end of the jetty to watch the boats come in. The men threw the fish up onto the boards in crates and the gulls stood about on the rails waiting for what was left.",
        "It was on one of those Saturdays, late in the season, that I first met Ruth Ellery. She was sitting on the steps of the lifeboat house with a book open on her knees, not reading it, and when I went past she asked me the time. I told her, and she said that was all right then, she had another hour, and went back to not reading her book. I walked on a little way and then came back and asked her what it was.",
        "\u{2018}It\u{2019}s about a man who builds a boat in his garden,\u{2019} she said. \u{2018}I\u{2019}ve read the first chapter four times. Every time I get to the end of it I forget how it started.\u{2019}",
        "I said that perhaps it was not a very good book, and she laughed and said that it was a very good book, the best she had ever read, and that the fault was all hers. Then she closed it and put it in her coat pocket, and we walked up the hill together as far as the chapel, where she turned off toward the farms.",
        "The next Saturday it rained, and I took the bus into town instead of walking, and came back with the shopping wrapped in newspaper because the bag had split. Mrs Pryor put the kettle on and told me that Mr Petrie had written to say he would not be coming in August after all, his sister being unwell, and that she had let the front room to a couple from Leeds who wanted it for the whole month and had asked whether there was a piano.",
        "\u{2018}I told them there was,\u{2019} she said. \u{2018}I didn\u{2019}t tell them it wants tuning.\u{2019}",
        "In the evenings that week I helped her with the accounts. She kept them in a long green book with a cloth spine, in pencil, and every column added up to the penny. She wrote the lodgers\u{2019} names down the left-hand side and what they owed across, and when they had paid she drew a line through the figure and wrote the date beside it in very small numbers. Some of the names went back twenty years. I asked her once whether she remembered them all, and she said she remembered most of them, and the ones she did not remember she had no wish to.",
        "On the Friday the couple from Leeds arrived on the late train. He was tall and quiet and carried both of their cases up the hill himself; she talked all the way from the station about the view, which in the dark there was no way of seeing. Mrs Pryor had left the hall light on for them and a plate of sandwiches under a cloth on the kitchen table, and by the time I came down in the morning they had eaten the sandwiches and gone out to look at the sea.",
    ];

    thread_local! {
        /// The open book's text, set once: every copy of it lies open at
        /// the same two pages (its running heads, folios and wear are its own)
        static SPREAD_TEXT: std::cell::OnceCell<Canvas> = const { std::cell::OnceCell::new() };
    }

    /// The two pages' text (1024 x 790 for 280 x 216 mm, the verso left, the
    /// recto right, the head up), justified in a serif, on a clear ground
    fn spread_text() -> Canvas {
        let mut ctx = Canvas::new(1024, 790);
        let ink = "rgba(44,38,32,0.86)";
        let (pitch, lines) = (17.6, 36);
        let mut words: Vec<(String, bool)> = vec![];
        for (k, para) in PROSE.iter().enumerate() {
            for (j, wd) in para.split(' ').enumerate() {
                words.push((wd.to_string(), j == 0 && k > 0));
            }
        }
        let mut wi = 0;
        ctx.set_fill_style(ink);
        ctx.set_text_align("left");
        ctx.set_font("14px Times New Roman");
        let space = ctx.measure_text(" ").width;
        let widths: Vec<f64> = words.iter().map(|w| ctx.measure_text(&w.0).width).collect();
        for page in 0..2 {
            let (x0, tw) = if page == 0 { (52.0, 396.0) } else { (512.0 + 64.0, 396.0) };
            for l in 0..lines {
                if wi >= words.len() {
                    break;
                }
                let y = 80.0 + l as f64 * pitch;
                let indent = if words[wi].1 { 20.0 } else { 0.0 };
                let mut j = wi;
                let mut used = indent;
                while j < words.len() {
                    if j > wi && words[j].1 {
                        break;
                    }
                    let need = if j == wi { used + widths[j] } else { used + space + widths[j] };
                    if j > wi && need > tw {
                        break;
                    }
                    used = need;
                    j += 1;
                }
                // justified, but the last line of a paragraph
                let last = j >= words.len() || words[j].1;
                let gaps = (j - wi).saturating_sub(1) as f64;
                let sum: f64 = widths[wi..j].iter().sum();
                let gap = if last || gaps < 1.0 { space } else { (tw - indent - sum) / gaps };
                let mut x = x0 + indent;
                for k in wi..j {
                    ctx.fill_text(&words[k].0, x, y);
                    x += widths[k] + gap;
                }
                wi = j;
            }
        }
        ctx
    }

    /// The open book's two pages: the text (set once), running heads and
    /// folios; the paper tanned and darker at its edges, foxed, a tide mark
    /// where damp got at it, dust over all of it where it lay open
    pub fn spread_tex(look: usize, salt: u32) -> Tex {
        let _ = look;
        canvas_tex_opt(1024, 790, false, move |ctx, w, h| {
            let mut r = SplitMix64::new(salt, 0x5b2e);
            let (t1, t2, author) = BOOKS[(salt as usize / 7) % BOOKS.len()];
            ctx.set_fill_style("#ddd2b8");
            ctx.fill_rect(0, 0, w, h);
            for _ in 0..36 {
                let at = (r.next() * w, r.next() * h);
                soft(
                    ctx,
                    at,
                    12.0 + r.next() * 40.0,
                    0.5 + r.next() * 0.5,
                    r.next() * 3.0,
                    if r.next() < 0.5 { (236.0, 228.0, 206.0) } else { (196.0, 180.0, 146.0) },
                    0.12,
                );
            }
            SPREAD_TEXT.with(|c| ctx.draw_image(c.get_or_init(spread_text), 0, 0));
            let ink = "rgba(44,38,32,0.86)";
            let first_page = 86 + (salt % 120) as i32 * 2;
            ctx.set_fill_style(ink);
            ctx.set_text_align("center");
            for page in 0..2 {
                let cx = if page == 0 { 52.0 + 198.0 } else { 576.0 + 198.0 };
                ctx.set_font("italic 12px Times New Roman");
                let head = if page == 0 { author.to_string() } else { format!("{t1} {t2}") };
                ctx.fill_text(&head, cx, 44);
                ctx.set_font("12px Times New Roman");
                ctx.fill_text(&format!("{}", first_page + page), cx, h - 40.0);
            }
            ctx.set_text_align("left");
            // the paper browned toward its edges, and in the gutter
            for (x0, x1) in [(0.0, 60.0), (w, w - 60.0), (512.0, 470.0), (512.0, 554.0)] {
                let gr = ctx.create_linear_gradient(x0, 0, x1, 0);
                gr.add_color_stop(0, "rgba(130,96,52,0.40)");
                gr.add_color_stop(1, "rgba(130,96,52,0)");
                ctx.set_fill_style(&gr);
                ctx.fill_rect(x0.min(x1), 0, 60.0f64.max((x1 - x0).abs()), h);
            }
            for (y0, y1) in [(0.0, 50.0), (h, h - 50.0)] {
                let gr = ctx.create_linear_gradient(0, y0, 0, y1);
                gr.add_color_stop(0, "rgba(130,96,52,0.35)");
                gr.add_color_stop(1, "rgba(130,96,52,0)");
                ctx.set_fill_style(&gr);
                ctx.fill_rect(0, y0.min(y1), w, 50);
            }
            // foxing
            for _ in 0..70 {
                let at = (r.next() * w, r.next() * h);
                soft(ctx, at, 1.5 + r.next() * 5.0, 1.0, 0.0, (140.0, 92.0, 46.0), 0.3 + 0.3 * r.next());
            }
            // a tide mark where damp came up into the recto's foot
            let (tx, ty) = (512.0 + 140.0 + r.next() * 260.0, h - 30.0);
            ctx.save();
            ctx.translate(tx, ty);
            ctx.scale(1, 0.55);
            let gr = ctx.create_radial_gradient(0, 0, 90, 0, 0, 160);
            gr.add_color_stop(0, "rgba(150,110,60,0.10)");
            gr.add_color_stop(0.9, "rgba(120,84,40,0.40)");
            gr.add_color_stop(1, "rgba(120,84,40,0)");
            ctx.set_fill_style(&gr);
            ctx.begin_path();
            ctx.arc(0, 0, 160, 0, 7);
            ctx.fill();
            ctx.restore();
            // the dust it gathered lying open: a grey film, in drifts
            ctx.set_fill_style("rgba(126,118,104,0.26)");
            ctx.fill_rect(0, 0, w, h);
            for _ in 0..14 {
                let at = (r.next() * w, r.next() * h);
                soft(ctx, at, 30.0 + r.next() * 90.0, 0.4 + r.next() * 0.6, r.next() * 3.0, (120.0, 112.0, 98.0), 0.08 + 0.12 * r.next());
            }
            for _ in 0..300 {
                let (x, y) = (r.next() * w, r.next() * h);
                ctx.set_fill_style(rgba((90.0, 82.0, 70.0), 0.15 + 0.3 * r.next()));
                ctx.fill_rect(x, y, 1.0 + r.next(), 1.0 + r.next());
            }
        })
    }
}
