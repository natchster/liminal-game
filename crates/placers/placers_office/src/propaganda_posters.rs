//! `place_propaganda_posters`
use crate::shared::on_nail;
use game::base::rng::SplitMix64;
use game::prelude::*;
use game::visuals::pictures;
use game::visuals::pictures::Age;
use game::visuals::pictures::Finish;
use game::visuals::pictures::Outline;
use placers_kit::place::face_wall;
use placers_kit::place::wall_tiles;

pub(crate) fn place_propaganda_posters(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.975 {
        return;
    }
    let wt = wall_tiles(g);
    if wt.is_empty() {
        return;
    }
    let (x, z, dx, dz) = wt[(rnd() * wt.len() as f64).floor() as usize];
    // the frame (the old frame colour's draw): walnut stain, black (ebonised, rubbed), varnished pine, grey paint
    let finish = match choice(&[0x604020, 0x101010, 0x804a20, 0x2a2a2a]) {
        0x604020 => Finish::Wood([0x50, 0x34, 0x20]),
        0x101010 => Finish::Ebony,
        0x804a20 => Finish::Wood([0x8e, 0x5e, 0x30]),
        _ => Finish::Paint([0x56, 0x56, 0x52]),
    };
    let pp = group();
    let mount_y = 1.60;
    pp.rotation().y = face_wall(dx, dz);
    pp.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - 0.015), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - 0.015));
    // askew on its nail (the old tilt's draw)
    let tilt = (rnd() - 0.5) * 0.03;
    let p = pp.pos();
    let vary = pictures::vary_of(p.x, p.z, tilt);
    let body = on_nail(&pp, mount_y, 0.015, WP_SIZE.1, tilt);
    // cheap paper: browned more than a plate, the inks faded, a tide line
    // where damp got under the glass now and then
    let mut age = Age::engraving(vary);
    age.tone = [age.tone[0], age.tone[1] - 0.04, age.tone[2] - 0.1];
    age.fade += 0.1;
    age.edge += 0.2;
    let design = vary / 7;
    let dirt = 0.3 + 0.3 * (vary % 53) as f32 / 53.0;
    pictures::add_parts(
        &body,
        &[
            ("Real__warPosterFrame", lambert!({ map: &pictures::moulding_tex(WP_SIZE, &WP_FACE, Outline::Rect, finish, 1300.0, vary) })),
            ("Real__warPosterGlass", pictures::glass_mat(vary, dirt)),
            (
                "Real__warPosterPaper",
                lambert!({ map: &pictures::sheet_tex(WP_PAPER, 1500.0, age, vary, move |ctx, w, h| paint_poster(ctx, w, h, design, vary)) }),
            ),
            ("Real__warPosterBack", lambert!({ color: 0x4a3a2a })),
        ],
    );
    g.world.add(&pp);
}

// ----------------------------------------------------------- the poster ---
/// The frame's outside and the poster under its glass (`t02.py`: WP_*)
const WP_SIZE: (f64, f64) = (0.200, 0.280);

const WP_PAPER: (f64, f64) = (0.184, 0.264);

/// The cheap moulding's face, outer edge to sight (d, z), as `poster_frame`
static WP_FACE: [(f64, f64); 7] =
    [(0.0, 0.0118), (0.0008, 0.0134), (0.0024, 0.0140), (0.0092, 0.0140), (0.0106, 0.0134), (0.0115, 0.0124), (0.012, 0.0112)];

/// A line of a poster: its words, its face, its share of the type area's
/// width it fills at most, its cap height (px) and its ink
struct Line {
    text: &'static str,
    face: &'static str,
    width: f64,
    size: f64,
    ink: &'static str,
}

/// The posters: (paper, field, its margin (share of the width), the lines,
/// the leading after each, the printer's line)
#[allow(clippy::type_complexity)]
fn poster(k: u32) -> (&'static str, &'static str, f64, Vec<(Line, f64)>, &'static str) {
    let l = |text, face, width, size, ink| Line { text, face, width, size, ink };
    match k % 3 {
        // the 1939 type-only bill, its crown gone: cream on red
        0 => (
            "#e2d6bc",
            "#9c3a2c",
            0.05,
            vec![
                (l("FREEDOM", "Gill Sans", 0.9, 44.0, "#eadfc6"), 0.45),
                (l("IS IN", "Gill Sans", 0.5, 34.0, "#eadfc6"), 0.45),
                (l("PERIL", "Gill Sans", 0.75, 44.0, "#eadfc6"), 1.6),
                (l("DEFEND IT", "Gill Sans", 0.7, 22.0, "#eadfc6"), 0.6),
                (l("WITH ALL", "Gill Sans", 0.55, 22.0, "#eadfc6"), 0.6),
                (l("YOUR MIGHT", "Gill Sans", 0.75, 22.0, "#eadfc6"), 0.0),
            ],
            "51-4317   50M   3/40",
        ),
        // black and red on the paper
        1 => (
            "#e4dbc4",
            "#e4dbc4",
            0.07,
            vec![
                (l("CARELESS", "Gill Sans", 0.92, 40.0, "#26221f"), 0.4),
                (l("TALK", "Gill Sans", 0.62, 52.0, "#26221f"), 0.9),
                (l("COSTS", "Gill Sans", 0.7, 40.0, "#9a3a2c"), 0.4),
                (l("LIVES", "Gill Sans", 0.7, 52.0, "#9a3a2c"), 1.4),
                (l("Keep it under your hat", "Gill Sans", 0.8, 13.0, "#26221f"), 0.0),
            ],
            "P.S. 1204   25M   9/41",
        ),
        // slab capitals on an ochre field
        _ => (
            "#e0d4b6",
            "#a87a30",
            0.05,
            vec![
                (l("DIG", "Rockwell", 0.5, 56.0, "#2e2418"), 0.35),
                (l("FOR", "Rockwell", 0.4, 34.0, "#2e2418"), 0.35),
                (l("VICTORY", "Rockwell", 0.9, 48.0, "#2e2418"), 1.5),
                (l("GROW YOUR OWN", "Rockwell", 0.7, 16.0, "#2e2418"), 0.5),
                (l("VEGETABLES", "Rockwell", 0.6, 16.0, "#2e2418"), 0.0),
            ],
            "51-6022   100M   2/42",
        ),
    }
}

/// A poster, its type set as a printer would: each line centred, its size
/// fitted to its share of the width, the inks laid a hair out of register,
/// unevenly; a white margin round the field
fn paint_poster(ctx: &mut Canvas, w: f64, h: f64, k: u32, vary: u32) {
    let mut r = SplitMix64::new(vary, 0x9057);
    let (paper, field, margin, lines, imprint) = poster(k);
    ctx.set_fill_style(paper);
    ctx.fill_rect(0, 0, w, h);
    let m = w * margin;
    ctx.set_fill_style(field);
    ctx.fill_rect(m, m, w - 2.0 * m, h - 2.0 * m);
    // the ink lies unevenly: a few soft lighter patches where it starved
    for _ in 0..14 {
        let (x, y, rr) = (r.next() * w, r.next() * h, w * r.range(0.05, 0.17));
        let gr = ctx.create_radial_gradient(x, y, 0, x, y, rr);
        gr.add_color_stop(0, "rgba(236,226,204,0.10)");
        gr.add_color_stop(1, "rgba(236,226,204,0)");
        ctx.set_fill_style(&gr);
        ctx.fill_rect(x - rr, y - rr, 2.0 * rr, 2.0 * rr);
    }
    // the block of type, its height summed, centred a little above the middle
    let scale = w / 276.0;
    let sizes: Vec<f64> = lines
        .iter()
        .map(|(l, _)| {
            ctx.set_font(&format!("bold {}px {}", l.size * scale, l.face));
            let tw = ctx.measure_text(l.text).width.max(1.0);
            (l.size * scale).min(l.size * scale * (w - 2.0 * m) * l.width / tw)
        })
        .collect();
    let total: f64 = lines.iter().zip(&sizes).map(|((_, lead), s)| s * (0.72 + lead)).sum();
    let mut y = (h - total) * 0.44;
    ctx.set_text_align("center");
    ctx.set_text_baseline("alphabetic");
    for ((l, lead), s) in lines.iter().zip(&sizes) {
        y += s * 0.72;
        ctx.set_font(&format!("bold {s}px {}", l.face));
        let off = r.range(-0.4, 0.4);
        ctx.set_fill_style(l.ink);
        ctx.fill_text(l.text, w / 2.0 + off, y);
        y += s * lead;
    }
    // the printer's line in the margin
    ctx.set_font(&format!("{}px Helvetica", (6.0 * scale).max(5.0)));
    ctx.set_fill_style("rgba(40,34,30,0.75)");
    ctx.set_text_align("left");
    ctx.fill_text(imprint, m, h - m * 0.35);
}
