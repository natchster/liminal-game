//! `place_reading_glasses_and_books`
use game::prelude::*;
use placers_kit::noise::vn;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;
use placers_kit::place::seed_of;

// Reading glasses and books (floor): 5% chance per level. A cloth
// hardback and a paperback on it (the book stack's books, batch r02), a slip
// of paper kept in the hardback for a place, and a pair of plastic reading
// glasses (batch r14b, 50-20-135) folded and put down on the books or beside
// them; the lenses smeared and dusty.
pub(crate) fn place_reading_glasses_and_books(g: &mut Game, _dist: &[i32]) {
    use game::assets::props::prop_part_as_modelled;
    if rnd() < 0.95 {
        return;
    }
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = open_tiles[(rnd() * open_tiles.len() as f64).floor() as usize];
    // the old draws, call for call: the cover, the frame, how the glasses
    // lie, the bookmark (now a slip of paper) and its colour, where and how
    // it all lies
    let cover_colors = [0x603020, 0x2040a0, 0x408060, 0x804020, 0x202050, 0xa06040];
    let cover_c = pick(&cover_colors);
    let cover = cover_colors.iter().position(|&c| c == cover_c).unwrap_or(0);
    let frame_colors = [0x101010, 0x604020, 0xa02040, 0xc0a040, 0x2060a0];
    let frame_c = pick(&frame_colors);
    let frame = frame_colors.iter().position(|&c| c == frame_c).unwrap_or(0);
    let glasses_turn = (rnd() - 0.5) * 0.8;
    let slip = if rnd() < 0.4 {
        let bookmark_colors = [0xa02040, 0x2060a0, 0xc0a040, 0x408060];
        let c = pick(&bookmark_colors);
        Some(bookmark_colors.iter().position(|&b| b == c).unwrap_or(0))
    } else {
        None
    };
    let gx = g.wx(x) + (rnd() - 0.5) * 2.0;
    let gz = g.wz(z) + (rnd() - 0.5) * 2.0;
    let yaw = rnd() * PI * 2.0;
    let seed = seed_of(&[gx, gz]);
    let grp = group();
    // the hardback: which of the three, by its cloth (boards, spine, pages,
    // thickness, board width)
    const HARDBACKS: [(&str, &str, &str, f64, f64); 3] = [
        ("Real__bookHbABoards", "Real__bookHbASpine", "Real__bookHbAPages", 0.0352, 0.146),
        ("Real__bookHbBBoards", "Real__bookHbBSpine", "Real__bookHbBPages", 0.0492, 0.164),
        ("Real__bookHbCBoards", "Real__bookHbCSpine", "Real__bookHbCPages", 0.0262, 0.128),
    ];
    let (boards, spine, block, thick, width) = HARDBACKS[cover % 3];
    let pages = lambert!({ map: &pages_tex(seed) });
    let hb = group();
    let hb_cover = lambert!({ map: &cloth_cover_tex(cover, seed) });
    let hb_spine = lambert!({ map: &spine_tex(cover, false, seed) });
    for (name, mat) in [(boards, &hb_cover), (spine, &hb_spine), (block, &pages)] {
        if let Some(geo) = prop_part_as_modelled(name) {
            hb.add(&mesh(geo, mat));
        }
    }
    if let (Some(k), Some(geo)) = (slip, prop_part_as_modelled("Real__bookSlip")) {
        let tint = [0xa8786a, 0x929ca2, 0xc6b68a, 0x9ca48c][k];
        let m = mesh(geo, lambert!({ color: tint }));
        m.position().set(0.03 - 0.06 * f64::from(seed % 7) / 6.0, thick * 0.5, -width / 2.0 + 0.04);
        hb.add(&m);
    }
    grp.add(&hb);
    // the paperback laid on it, askew
    let pb = group();
    let pb_k = ((seed >> 3) % 8) as usize;
    let pb_cover = lambert!({ map: &print_cover_tex(pb_k, seed) });
    let pb_spine = lambert!({ map: &spine_tex(pb_k, true, seed) });
    for (name, mat) in [("Real__bookPbCovers", &pb_cover), ("Real__bookPbSpine", &pb_spine), ("Real__bookPbPages", &pages)] {
        if let Some(geo) = prop_part_as_modelled(name) {
            pb.add(&mesh(geo, mat));
        }
    }
    let skew = 0.25 + 0.35 * f64::from((seed >> 6) % 16) / 15.0;
    pb.position().set(0.006 * f64::from((seed >> 10) % 5) - 0.012, thick, 0.008);
    pb.rotation().y = if seed & 2 == 0 { skew } else { -skew };
    grp.add(&pb);
    // the glasses, folded: on the paperback by its spine, or on the floor
    let glasses = group();
    let fr = frame_mat(frame, seed);
    if let Some(geo) = prop_part_as_modelled("Real__readersFrame") {
        glasses.add(&mesh(geo, &fr));
    }
    if let Some(geo) = prop_part_as_modelled("Real__readersLenses") {
        let lens = mesh(geo, lens_mat(&lens_dirt_tex(seed)));
        lens.set_render_order(2);
        glasses.add(&lens);
    }
    if (seed >> 14) % 2 == 0 {
        glasses.position().set(0.0, 0.0236, 0.034);
        glasses.rotation().y = glasses_turn * 0.5 + PI;
        pb.add(&glasses);
    } else {
        glasses.position().set(0.215, 0, 0.04);
        glasses.rotation().y = glasses_turn;
        grp.add(&glasses);
    }
    grp.position().set(gx, 0, gz);
    grp.rotation().y = yaw;
    g.world.add(&grp);
}

fn rgb(c: (f64, f64, f64), a: f64) -> String {
    format!("rgba({},{},{},{a})", c.0 as u8, c.1 as u8, c.2 as u8)
}

/// The hardback's cloth (sRGB), faded, for the old cover colours:
/// brown, navy, bottle green, red-brown, dark navy, tan buckram
const CLOTH: [(f64, f64, f64); 6] =
    [(92.0, 50.0, 40.0), (50.0, 58.0, 82.0), (52.0, 76.0, 60.0), (110.0, 62.0, 44.0), (40.0, 44.0, 62.0), (150.0, 126.0, 94.0)];

/// A paperback's printed ground and band
const PRINT: [((f64, f64, f64), (f64, f64, f64)); 8] = [
    ((172.0, 118.0, 76.0), (204.0, 194.0, 170.0)),
    ((86.0, 106.0, 88.0), (202.0, 192.0, 168.0)),
    ((80.0, 96.0, 118.0), (200.0, 190.0, 168.0)),
    ((198.0, 190.0, 168.0), (128.0, 64.0, 54.0)),
    ((112.0, 60.0, 54.0), (198.0, 186.0, 160.0)),
    ((72.0, 98.0, 100.0), (200.0, 190.0, 168.0)),
    ((158.0, 136.0, 82.0), (52.0, 48.0, 44.0)),
    ((118.0, 116.0, 110.0), (196.0, 188.0, 170.0)),
];

/// Titles and authors, none real
const TITLES: [&str; 12] = [
    "THE QUIET YEARS",
    "A FIELD GUIDE TO MOSSES",
    "LETTERS HOME",
    "THE LONG CORRIDOR",
    "COLLECTED STORIES",
    "NIGHT TRAINS",
    "THE ATLAS OF ROOMS",
    "MIDDLEMARCH ROAD",
    "BIRDS IN WINTER",
    "A HOUSE OF DOORS",
    "THE LAST TENANT",
    "SEA AND SKY",
];

const AUTHORS: [&str; 8] = ["E. L. BRANT", "MARGARET ASH", "J. W. COLE", "H. PEMBER", "A. DORAN", "R. LISLE", "C. M. FROST", "O. TARRANT"];

fn title(seed: u32, k: u32) -> (&'static str, &'static str) {
    (TITLES[((seed >> k) % TITLES.len() as u32) as usize], AUTHORS[((seed >> (k + 5)) % AUTHORS.len() as u32) as usize])
}

fn weave(ctx: &mut Canvas, r: &mut SplitMix64, w: f64, h: f64, n: usize) {
    for k in 0..n {
        ctx.set_fill_style(if k % 2 == 0 { "rgba(255,245,225,0.10)" } else { "rgba(0,0,0,0.12)" });
        ctx.fill_rect(r.range(0.0, w), r.range(0.0, h), 1.0 + r.next(), 1);
    }
}

fn blot(ctx: &mut Canvas, x: f64, y: f64, rad: f64, ring: bool, col: &str) {
    let gr = ctx.create_radial_gradient(x, y, rad * if ring { 0.75 } else { 0.0 }, x, y, rad);
    gr.add_color_stop(0, "rgba(0,0,0,0)");
    gr.add_color_stop(if ring { 0.85 } else { 0.0 }, col);
    gr.add_color_stop(1, "rgba(0,0,0,0)");
    ctx.set_fill_style(gr);
    ctx.fill_rect(x - rad, y - rad, 2.0 * rad, 2.0 * rad);
}

/// A cloth board (its head to the left, its fore-edge up, as the
/// boards' top takes it): faded toward the fore-edge, the weave, a
/// blind rule, the title stamped, the corners bumped, a ring where a cup
/// stood on it
fn cloth_cover_tex(c: usize, seed: u32) -> Tex {
    let (t, _) = title(seed, 1);
    canvas_tex_opt(128, 128, false, move |ctx, w, h| {
        let mut r = SplitMix64::new(seed, 0xc0e1);
        let col = CLOTH[c];
        ctx.set_fill_style(rgb(col, 1.0));
        ctx.fill_rect(0, 0, w, h);
        let gr = ctx.create_linear_gradient(0, h, 0, 0);
        gr.add_color_stop(0, "rgba(210,200,180,0.0)");
        gr.add_color_stop(1, &format!("rgba(210,200,180,{:.2})", r.range(0.12, 0.26)));
        ctx.set_fill_style(gr);
        ctx.fill_rect(0, 0, w, h);
        weave(ctx, &mut r, w, h, 1800);
        ctx.set_stroke_style("rgba(0,0,0,0.22)");
        ctx.set_line_width(1.3);
        ctx.stroke_rect(7, 7, 114, 114);
        ctx.save();
        ctx.translate(0, h);
        ctx.rotate(-PI / 2.0);
        ctx.scale(128.0 / 146.0, 128.0 / 222.0);
        ctx.set_fill_style("rgba(170,146,92,0.7)");
        ctx.set_text_align("center");
        ctx.set_font("bold 10px Times New Roman");
        ctx.fill_text_max(t, 73, 50, 100);
        ctx.restore();
        for (x, y) in [(0.0, 0.0), (w, 0.0)] {
            blot(ctx, x, y, r.range(7.0, 13.0), false, "rgba(176,164,140,0.75)");
        }
        for (x, y, ww, hh) in [(0.0, 0.0, w, 3.0), (0.0, 0.0, 3.0, h), (w - 3.0, 0.0, 3.0, h)] {
            ctx.set_fill_style("rgba(200,190,170,0.22)");
            ctx.fill_rect(x, y, ww, hh);
        }
        let (x, y) = (r.range(30.0, 98.0), r.range(30.0, 98.0));
        blot(ctx, x, y, r.range(16.0, 24.0), true, "rgba(40,30,20,0.28)");
    })
}

/// A paperback's front (as the cloth board): its printed ground and
/// band, title and author, the publisher's mark; creased at the spine,
/// its edges worn white
fn print_cover_tex(c: usize, seed: u32) -> Tex {
    let (t, a) = title(seed, 9);
    canvas_tex_opt(128, 128, false, move |ctx, w, h| {
        let mut r = SplitMix64::new(seed, 0x9b1c);
        let (ground, band) = PRINT[c];
        ctx.save();
        ctx.translate(0, h);
        ctx.rotate(-PI / 2.0);
        ctx.scale(128.0 / 140.0, 128.0 / 216.0);
        ctx.set_fill_style(rgb(ground, 1.0));
        ctx.fill_rect(0, 0, 140, 216);
        ctx.set_fill_style(rgb(band, 1.0));
        ctx.fill_rect(0, 72, 140, 66);
        ctx.set_fill_style(rgb(ground, 0.9));
        ctx.set_text_align("center");
        ctx.set_font("bold 13px Gill Sans");
        let mut parts = t.splitn(2, ' ');
        if let Some(l) = parts.next() {
            ctx.fill_text_max(l, 70, 100, 124);
        }
        if let Some(l) = parts.next() {
            ctx.fill_text_max(l, 70, 117, 124);
        }
        ctx.set_fill_style(rgb(band, 0.9));
        ctx.set_font("10px Gill Sans");
        ctx.fill_text_max(a, 70, 42, 120);
        ctx.begin_path();
        ctx.arc(70, 186, 8, 0, 7);
        ctx.fill();
        ctx.set_stroke_style("rgba(235,228,212,0.7)");
        ctx.set_line_width(1);
        for x in [9.0, 13.0] {
            ctx.begin_path();
            ctx.move_to(x, 0);
            ctx.line_to(x + 1.0, 216);
            ctx.stroke();
        }
        ctx.set_fill_style("rgba(235,228,212,0.35)");
        for (x, y, ww, hh) in [(0.0, 0.0, 140.0, 3.0), (0.0, 213.0, 140.0, 3.0), (137.0, 0.0, 3.0, 216.0)] {
            ctx.fill_rect(x, y, ww, hh);
        }
        ctx.restore();
        for (x, y) in [(0.0, 0.0), (w, 0.0)] {
            blot(ctx, x, y, r.range(6.0, 11.0), false, "rgba(236,230,214,0.7)");
        }
        weave(ctx, &mut r, w, h, 500);
        let (x, y) = (r.range(20.0, 108.0), r.range(20.0, 108.0));
        blot(ctx, x, y, r.range(12.0, 26.0), r.next() < 0.6, "rgba(60,44,24,0.25)");
    })
}

/// A spine (head at the left, the front board up): the cloth faded
/// more than its boards, title and author stamped in gilt, rules at head
/// and tail; or a paperback's printed spine, cracked where it was read
fn spine_tex(c: usize, paperback: bool, seed: u32) -> Tex {
    let (t, a) = title(seed, if paperback { 9 } else { 1 });
    canvas_tex_opt(256, 32, false, move |ctx, w, h| {
        let mut r = SplitMix64::new(seed, if paperback { 0x5b2 } else { 0x5b1 });
        let (ink, ground) = if paperback {
            let (g0, b0) = PRINT[c];
            ctx.set_fill_style(rgb(g0, 1.0));
            ctx.fill_rect(0, 0, w, h);
            ctx.set_fill_style(rgb(b0, 1.0));
            ctx.fill_rect(70, 0, 110, h);
            (rgb(g0, 0.95), b0)
        } else {
            let col = CLOTH[c];
            let f = r.range(0.18, 0.38);
            let faded = (col.0 + (196.0 - col.0) * f, col.1 + (186.0 - col.1) * f, col.2 + (166.0 - col.2) * f);
            ctx.set_fill_style(rgb(faded, 1.0));
            ctx.fill_rect(0, 0, w, h);
            weave(ctx, &mut r, w, h, 900);
            ctx.set_fill_style("rgba(180,152,92,0.8)");
            for x in [12.0, 15.0, 241.0, 244.0] {
                ctx.fill_rect(x, 0, 1.2, h);
            }
            ("rgba(184,156,96,0.85)".to_string(), faded)
        };
        ctx.set_fill_style(ink.as_str());
        ctx.set_text_align("left");
        ctx.set_text_baseline("middle");
        ctx.set_font(if paperback { "bold 9px Gill Sans" } else { "bold 9px Times New Roman" });
        ctx.fill_text_max(t, if paperback { 78.0 } else { 30.0 }, 17, if paperback { 96.0 } else { 140.0 });
        ctx.set_font(if paperback { "7px Gill Sans" } else { "7px Times New Roman" });
        if paperback {
            ctx.set_fill_style(rgb(ground, 0.9));
        }
        ctx.fill_text_max(a, if paperback { 186.0 } else { 182.0 }, 17, if paperback { 64.0 } else { 54.0 });
        ctx.set_text_baseline("alphabetic");
        for _ in 0..140 {
            ctx.set_fill_style(rgb(ground, 0.6));
            ctx.fill_rect(r.range(0.0, w), r.range(0.0, h), 1.5, 1.5);
        }
        if paperback {
            ctx.set_stroke_style("rgba(236,230,214,0.75)");
            ctx.set_line_width(0.8);
            for _ in 0..(2.0 + r.next() * 4.0) as usize {
                let y = r.range(4.0, 28.0);
                let x = r.range(0.0, 60.0);
                ctx.begin_path();
                ctx.move_to(x, y);
                ctx.line_to(x + r.range(60.0, 200.0), y + r.range(-1.0, 1.0));
                ctx.stroke();
            }
        } else {
            for x in [0.0, 250.0] {
                ctx.set_fill_style("rgba(200,188,166,0.35)");
                ctx.fill_rect(x, 0, 6, h);
            }
        }
        let gr = ctx.create_linear_gradient(0, 0, 0, 8);
        gr.add_color_stop(0, "rgba(150,140,120,0.35)");
        gr.add_color_stop(1, "rgba(150,140,120,0)");
        ctx.set_fill_style(gr);
        ctx.fill_rect(0, 0, w, 8);
    })
}

/// Page edges: yellowed leaves, tanned toward the boards, foxed
fn pages_tex(seed: u32) -> Tex {
    canvas_tex_opt(256, 64, false, move |ctx, w, h| {
        let mut r = SplitMix64::new(seed, 0x9a6e);
        ctx.set_fill_style("#d0c19c");
        ctx.fill_rect(0, 0, w, h);
        let mut y = 0.0;
        while y < h {
            ctx.set_fill_style(format!("rgba(90,70,40,{:.2})", r.range(0.05, 0.17)));
            ctx.fill_rect(0, y, w, r.range(0.6, 1.2));
            y += r.range(1.0, 2.5);
        }
        for (ya, yb) in [(0.0, 9.0), (h, h - 9.0)] {
            let gr = ctx.create_linear_gradient(0, ya, 0, yb);
            gr.add_color_stop(0, "rgba(120,86,40,0.45)");
            gr.add_color_stop(1, "rgba(120,86,40,0)");
            ctx.set_fill_style(gr);
            ctx.fill_rect(0, 0, w, h);
        }
        for _ in 0..40 {
            let (x, y) = (r.range(0.0, w), r.range(0.0, h));
            blot(ctx, x, y, r.range(1.0, 3.0), false, "rgba(130,84,40,0.35)");
        }
    })
}

/// The frame's acetate: black, dark tortoiseshell, burgundy, honey
/// tortoiseshell, navy (for the old frame colours); tortoiseshell's
/// mottle painted, the shine of polished plastic
fn frame_mat(k: usize, seed: u32) -> Mat {
    let tortoise = k == 1 || k == 3;
    if !tortoise {
        let col = [0x161412, 0, 0x3e161c, 0, 0x1c2232][k];
        return phong!({ color: col, specular: 0x303030, shininess: 60 });
    }
    let honey = k == 3;
    let tex = canvas_tex_opt(128, 128, false, move |ctx, w, h| {
        let n = w as usize;
        let mut img = ctx.create_image_data(w, h);
        par_rows(&mut img.data, n * 4, |j, out| {
            for i in 0..n {
                let (x, y) = (i as f64 / w, j as f64 / h);
                let m = vn(seed, x * 9.0, y * 5.0) * 0.6 + vn(seed ^ 0x33, x * 23.0, y * 14.0) * 0.4;
                let spot = ((m - 0.45) * 3.0).clamp(0.0, 1.0);
                let (a, b) = if honey { ((150.0, 104.0, 46.0), (52.0, 30.0, 14.0)) } else { ((92.0, 54.0, 24.0), (26.0, 16.0, 10.0)) };
                let px = |u: f64, v: f64| (u + (v - u) * spot) as u8;
                out[i * 4..i * 4 + 4].copy_from_slice(&[px(a.0, b.0), px(a.1, b.1), px(a.2, b.2), 255]);
            }
        });
        ctx.put_image_data(&img, 0, 0);
    });
    phong!({ map: &tex, specular: 0x303030, shininess: 60 })
}

/// The lenses' dirt (grey: how much): a haze, smears, dust
fn lens_dirt_tex(seed: u32) -> Tex {
    canvas_tex_opt(128, 64, false, move |ctx, w, h| {
        let n = w as usize;
        let mut img = ctx.create_image_data(w, h);
        par_rows(&mut img.data, n * 4, |j, out| {
            for i in 0..n {
                let (x, y) = (i as f64 / w, j as f64 / h);
                let d = vn(seed, x * 7.0, y * 4.0) * 0.6 + vn(seed ^ 7, x * 26.0, y * 13.0) * 0.4;
                let smear = (vn(seed ^ 0x55, x * 3.0, y * 2.0) - 0.58).max(0.0) * 1.8;
                let v = ((0.18 + 0.36 * d + smear) * 255.0).min(255.0) as u8;
                out[i * 4..i * 4 + 4].copy_from_slice(&[v, v, v, 255]);
            }
        });
        ctx.put_image_data(&img, 0, 0);
    })
}

/// The lenses (glass, `Patch::Glass`): the room seen in them by
/// Fresnel, their dirt from the map's grey
fn lens_mat(map: &Tex) -> Mat {
    let m = phong!({ color: 0xffffff, specular: 0x9a9a9a, shininess: 220, transparent: true, depthWrite: false });
    {
        let mut mm = m.m();
        mm.map = Some(map.clone());
        mm.patch = Patch::Glass;
        mm.look_settled = true;
        mm.name = "reading_lens".into();
    }
    {
        let pu = m.r().patch_uniforms.clone();
        let f = &mut pu.borrow_mut().f;
        f[0] = 0.5;
        f[4] = 0.5;
        f[5] = 0.6;
        f[6] = 0.3;
        f[7] = 0.0;
        f[8] = 1.8;
        f[9] = 0.2;
        f[10] = 0.19;
        f[11] = 0.17;
        f[12] = 0.0;
        f[13] = 0.08;
    }
    m
}
