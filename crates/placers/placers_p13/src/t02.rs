//! Batch `t02`'s props in this crate, made real (`docs/PROPS.md` P3): the
//! office's framed map, the anatomy chart and the home-front poster, and
//! what only they use. Each keeps its old placer's draws, call for call.
//!
//! - The framed map: a reprint of an old map (CC0, `visuals::pictures`: the
//!   world with Cook's tracks, 1780-1796; a hand-coloured plan of Antwerp,
//!   1830) under glass in an office's frame: a flat walnut or oak gallery
//!   moulding, or a black or brass-anodised aluminium section (the old frame
//!   colour's draw), hung flush from its sawtooth on a nail.
//! - The anatomy chart: a plate of Bidloo's anatomy (1685, CC0: a skeleton
//!   from the front or from behind) mounted on linen between two turned
//!   rails (walnut, beech, black or grey: the old frame colour's draw), hung
//!   by its cord from a nail, varnished, its foot curling.
//! - The poster: a home-front window bill set in Gill Sans or Rockwell,
//!   printed in one or two inks on cheap paper (a real wartime slogan that
//!   names no country, party or person), faded, behind glass in a cheap
//!   frame (stained or painted: the old frame colour's draw).
use super::*;
use game::base::rng::SplitMix64;
use game::visuals::pictures::{self, Age, Finish, Outline};

/// A picture hung flush from the sawtooth on its back over a nail 16 mm
/// below its top (`h` its height), the wall `wall` behind the spot, askew by
/// `tilt`: the body its parts go in (z = 0 the wall)
fn on_nail(o: &Obj, mount_y: f64, wall: f64, h: f64, tilt: f64) -> Obj {
    let nail = group();
    nail.position().set(0, mount_y + h / 2.0 - 0.016, -wall);
    nail.rotation().z = tilt;
    o.add(&nail);
    let body = group();
    body.position().y = -(h / 2.0 - 0.016);
    nail.add(&body);
    body
}

// ------------------------------------------------------- the framed map ---
/// The print and the two frames' outsides (`t02.py`: OM_*)
const OM_PRINT: (f64, f64) = (0.274, 0.178);
const OM_WOOD: (f64, f64) = (0.300, 0.204);
const OM_METAL: (f64, f64) = (0.286, 0.190);
/// The gallery moulding's face, outer edge to sight (d, z), as `office_frame_wood`
static OM_FACE: [(f64, f64); 9] = [
    (0.0, 0.0182),
    (0.0005, 0.0196),
    (0.0016, 0.0200),
    (0.0122, 0.0200),
    (0.0130, 0.0196),
    (0.0134, 0.0184),
    (0.0142, 0.0180),
    (0.0158, 0.0176),
    (0.017, 0.0168),
];

pub(crate) fn place_framed_maps(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.975 {
        return;
    }
    let wt = wall_tiles(g);
    if wt.is_empty() {
        return;
    }
    let (x, z, dx, dz) = wt[(rnd() * wt.len() as f64).floor() as usize];
    // the frame (the old frame colour's draw): walnut, oak, black aluminium, brass-anodised aluminium
    let frame_colors = [0x604020, 0x8b5a2b, 0x2a2a2a, 0xd0a040];
    let color = choice(&frame_colors);
    let kind = frame_colors.iter().position(|&c| c == color).unwrap_or(0);
    let fm = group();
    let mount_y = 1.60;
    fm.rotation().y = face_wall(dx, dz);
    fm.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - 0.015), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - 0.015));
    // askew on its nail (the old tilt's draw)
    let tilt = (rnd() - 0.5) * 0.03;
    let p = fm.pos();
    let vary = pictures::vary_of(p.x, p.z, tilt);
    let size = if kind < 2 { OM_WOOD } else { OM_METAL };
    let body = on_nail(&fm, mount_y, 0.015, size.1, tilt);
    let frame = match kind {
        0 => (
            "Real__officeMapFrameWood",
            lambert!({ map: &pictures::moulding_tex(OM_WOOD, &OM_FACE, Outline::Rect, Finish::Wood([0x50, 0x34, 0x20]), 1200.0, vary) }),
        ),
        1 => (
            "Real__officeMapFrameWood",
            lambert!({ map: &pictures::moulding_tex(OM_WOOD, &OM_FACE, Outline::Rect, Finish::Wood([0x8a, 0x68, 0x40]), 1200.0, vary) }),
        ),
        2 => ("Real__officeMapFrameMetal", phong!({ color: 0x1c1c1e, specular: 0x3a3a3c, shininess: 44 })),
        _ => ("Real__officeMapFrameMetal", phong!({ color: 0x6a5634, specular: 0x6a5a3a, shininess: 44 })),
    };
    // a reprint on modern paper: faded by the office's light more than foxed
    let mut age = Age::engraving(vary);
    age.spots /= 3;
    age.fade += 0.03;
    let pic = ["map_cook", "map_antwerp"][(vary % 2) as usize];
    let dirt = 0.25 + 0.3 * (vary % 59) as f32 / 59.0;
    pictures::add_parts(
        &body,
        &[
            frame,
            ("Real__officeMapGlass", pictures::glass_mat(vary, dirt)),
            ("Real__officeMapPrint", lambert!({ map: &pictures::print_tex(pic, OM_PRINT, 1660.0, age, vary) })),
            ("Real__officeMapBack", lambert!({ color: 0x4a3a2a })),
        ],
    );
    g.world.add(&fm);
}

// ------------------------------------------------------ the anatomy chart ---
/// The plate, the rails' axes (above and below its middle, out from the
/// wall) and the nail above its middle (`t02.py`: AC_*)
const AC_SHEET: (f64, f64) = (0.215, 0.300);
const AC_RAIL: (f64, f64) = (0.154, 0.0075);
const AC_NAIL: f64 = 0.305;

pub(crate) fn place_anatomy_charts(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.975 {
        return;
    }
    let wt = wall_tiles(g);
    if wt.is_empty() {
        return;
    }
    let (x, z, dx, dz) = wt[(rnd() * wt.len() as f64).floor() as usize];
    // the rails (the old frame colour's draw): walnut, beech, black paint, weathered grey
    let rail = match choice(&[0x604020, 0x8b5a2b, 0x202020, 0x504030]) {
        0x604020 => phong!({ color: 0x4e3420, specular: 0x1e1a14, shininess: 20 }),
        0x8b5a2b => phong!({ color: 0x8a6a46, specular: 0x1e1a14, shininess: 20 }),
        0x202020 => phong!({ color: 0x1e1c1a, specular: 0x262626, shininess: 26 }),
        _ => lambert!({ color: 0x5e564a }),
    };
    let ac = group();
    let mount_y = 1.60;
    ac.rotation().y = face_wall(dx, dz);
    ac.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - 0.015), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - 0.015));
    // swung on its cord (the old tilt's draw)
    let tilt = (rnd() - 0.5) * 0.03;
    let p = ac.pos();
    let vary = pictures::vary_of(p.x, p.z, tilt);
    // the nail, and the chart hanging from it
    let nail = group();
    nail.position().set(0, mount_y + AC_NAIL, -0.015);
    nail.rotation().z = tilt;
    ac.add(&nail);
    let body = group();
    body.position().y = -AC_NAIL;
    nail.add(&body);
    let pic = ["anat_front", "anat_back"][(vary % 2) as usize];
    // varnished, as wall charts were: gone amber, darkest where hands rolled it
    let mut age = Age::engraving(vary);
    age.tone = [age.tone[0], age.tone[1] - 0.03, age.tone[2] - 0.08];
    age.grime += 0.06;
    pictures::add_parts(
        &body,
        &[
            (
                "Real__anatomyChartSheet",
                phong!({ map: &pictures::print_tex(pic, AC_SHEET, 1330.0, age, vary), specular: 0x24221c, shininess: 16 }),
            ),
            ("Real__anatomyChartCord", lambert!({ color: 0x6a604e })),
            ("Real__anatomyChartNail", phong!({ color: 0x3a3936, specular: 0x404040, shininess: 30 })),
        ],
    );
    if let Some(geo) = game::assets::props::prop_part_as_modelled("Real__anatomyChartRail") {
        for y in [AC_RAIL.0, -AC_RAIL.0] {
            let r = mesh(geo.clone(), &rail);
            r.position().set(0, y, AC_RAIL.1);
            body.add(&r);
        }
    }
    g.world.add(&ac);
}

// ----------------------------------------------------------- the poster ---
/// The frame's outside and the poster under its glass (`t02.py`: WP_*)
const WP_SIZE: (f64, f64) = (0.200, 0.280);
const WP_PAPER: (f64, f64) = (0.184, 0.264);
/// The cheap moulding's face, outer edge to sight (d, z), as `poster_frame`
static WP_FACE: [(f64, f64); 7] = [
    (0.0, 0.0118),
    (0.0008, 0.0134),
    (0.0024, 0.0140),
    (0.0092, 0.0140),
    (0.0106, 0.0134),
    (0.0115, 0.0124),
    (0.012, 0.0112),
];

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
            ("Real__warPosterPaper", lambert!({ map: &pictures::sheet_tex(WP_PAPER, 1500.0, age, vary, move |ctx, w, h| paint_poster(ctx, w, h, design, vary)) })),
            ("Real__warPosterBack", lambert!({ color: 0x4a3a2a })),
        ],
    );
    g.world.add(&pp);
}
