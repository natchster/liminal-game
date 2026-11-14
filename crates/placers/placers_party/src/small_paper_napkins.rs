//! `place_small_paper_napkins`: Small paper napkin: 12% chance per level. A two-ply luncheon napkin left from a party (Blender family `paper_and_small_things_on_office_floors`: 33 cm, folded in four): with three of the old crumples dropped half open, with four scrunched into a wad; its embossed border and printed spots, a smear of cake, dust. The old draws are kept call for call.
use game::prelude::*;
use placers_kit::paint::rgba;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

/// Small paper napkin: 12% chance per level. A two-ply luncheon
/// napkin left from a party (Blender family `paper_and_small_things_on_office_floors`: 33 cm, folded in four): with
/// three of the old crumples dropped half open, with four scrunched into a
/// wad; its embossed border and printed spots, a smear of cake, dust. The
/// old draws are kept call for call.
pub(crate) fn place_small_paper_napkins(g: &mut Game, _dist: &[i32]) {
    use game::assets::props::prop_part_as_modelled;
    if rnd() < 0.88 {
        return;
    } // 12% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    // the old colour draw: white, cream, pink, blue, yellow or green napkins
    let nap_colors = [0xf0ece4, 0xe8d8c8, 0xf0c0d0, 0xd0d8f0, 0xf0d8a0, 0xd8f0d0];
    let col = pick(&nap_colors);
    let look = nap_colors.iter().position(|&c| c == col).unwrap_or(0);
    let napkin = group();
    let n_crumples = 3 + (rnd() * 2.0).floor() as i32;
    for _ in 0..n_crumples {
        // the old crumple's size, heights, corners, turn and place: kept as draws
        for _ in 0..8 {
            let _ = rnd();
        }
    }
    let px = g.wx(x) + (rnd() - 0.5) * 2.4;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.4;
    napkin.position().set(px, 0, pz);
    napkin.rotation().y = rnd() * PI * 2.0;
    let salt = ((px * 977.0).floor() as i64 ^ ((pz * 1181.0).floor() as i64).rotate_left(13)) as u32;
    let name = if n_crumples == 3 { "Real__napkinCrumpled" } else { "Real__napkinWad" };
    if let Some(geo) = prop_part_as_modelled(name) {
        napkin.add(&mesh(geo, &lambert!({ map: &napkin_tex(look, salt), side: DoubleSide })));
    }
    g.world.add(&napkin);
}

/// Tissue napkins (sRGB), by the old colour, gone dull
const TISSUE: [(f64, f64, f64); 6] = [
    (224.0, 220.0, 210.0),
    (214.0, 198.0, 178.0),
    (212.0, 174.0, 180.0),
    (178.0, 190.0, 210.0),
    (218.0, 196.0, 146.0),
    (186.0, 208.0, 180.0),
];

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

/// The napkin from above (165 mm, folded in four): the tissue's fine
/// emboss all over, the border pressed in a band 12 mm from the edges,
/// spots printed on a coloured one, the folds' lines; a smear of
/// chocolate cake and a grease spot where it was used; the floor's dust
fn napkin_tex(look: usize, salt: u32) -> Tex {
    canvas_tex_opt(256, 256, false, move |ctx, w, h| {
        let mut r = SplitMix64::new(salt, 0x4a9c);
        let c = TISSUE[look];
        ctx.set_fill_style(rgba(c, 1.0));
        ctx.fill_rect(0, 0, w, h);
        // the emboss: a fine grid of pressed dots, lit from one side (a
        // 64 px repeat, laid as the embossing roller lays it)
        let mut tile = Canvas::new(64, 64);
        let mut y = 1.5;
        while y < 64.0 {
            let mut x = if (y as i32 / 4) % 2 == 0 { 1.5 } else { 3.5 };
            while x < 64.0 {
                tile.set_fill_style("rgba(255,255,250,0.10)");
                tile.fill_rect(x, y, 1.0, 1.0);
                tile.set_fill_style("rgba(0,0,0,0.06)");
                tile.fill_rect(x + 1.0, y + 1.0, 1.0, 1.0);
                x += 4.0;
            }
            y += 4.0;
        }
        for ty in 0..4 {
            for tx in 0..4 {
                ctx.draw_image(&tile, tx as f64 * 64.0, ty as f64 * 64.0);
            }
        }
        // the border's pressed band: a wave between two lines
        let b = 18.0;
        ctx.set_line_width(1.0);
        for (dx, col) in [(0.0, "rgba(255,255,250,0.22)"), (1.0, "rgba(0,0,0,0.14)")] {
            ctx.set_stroke_style(col);
            for inset in [b - 6.0, b + 6.0] {
                ctx.stroke_rect(inset + dx, inset + dx, w - 2.0 * inset, h - 2.0 * inset);
            }
            ctx.begin_path();
            let mut t = 0.0;
            let per = 4.0 * (w - 2.0 * b);
            while t <= per {
                let (side, s) = ((t / (w - 2.0 * b)) as i32, t % (w - 2.0 * b));
                let off = 3.5 * (t / 6.0).dsin();
                let (x, y) = match side {
                    0 => (b + s, b + off),
                    1 => (w - b + off, b + s),
                    2 => (w - b - s, h - b + off),
                    _ => (b + off, h - b - s),
                };
                if t == 0.0 {
                    ctx.move_to(x + dx, y + dx);
                } else {
                    ctx.line_to(x + dx, y + dx);
                }
                t += 2.0;
            }
            ctx.stroke();
        }
        // spots printed on the coloured ones (a white one is plain)
        if look >= 2 {
            let ink = [(240.0, 236.0, 228.0), (200.0, 120.0, 120.0), (110.0, 140.0, 190.0), (230.0, 230.0, 220.0)][(salt % 4) as usize];
            ctx.set_fill_style(rgba(ink, 0.55));
            let mut j = 0;
            let mut y = 30.0;
            while y < h - 26.0 {
                let mut x = if j % 2 == 0 { 30.0 } else { 44.0 };
                while x < w - 26.0 {
                    ctx.begin_path();
                    ctx.arc(x, y, 3.2, 0, 7);
                    ctx.fill();
                    x += 28.0;
                }
                y += 24.0;
                j += 1;
            }
        }
        // the quarter folds, crossing it
        ctx.set_stroke_style("rgba(0,0,0,0.10)");
        ctx.set_line_width(1.4);
        ctx.begin_path();
        ctx.move_to(w / 2.0, 0);
        ctx.line_to(w / 2.0 + 3.0, h);
        ctx.move_to(0, h / 2.0);
        ctx.line_to(w, h / 2.0 - 2.0);
        ctx.stroke();
        // used: a smear of chocolate cake, a grease spot gone translucent
        let (sx, sy) = (60.0 + r.next() * 130.0, 60.0 + r.next() * 130.0);
        for k in 0..5 {
            let at = (sx + k as f64 * 6.0 + r.next() * 4.0, sy + k as f64 * 3.0 + r.next() * 4.0);
            soft(ctx, at, 7.0 + r.next() * 6.0, 0.6, 0.4, (86.0, 58.0, 40.0), 0.35 + 0.2 * r.next());
        }
        soft(
            ctx,
            (40.0 + r.next() * 170.0, 40.0 + r.next() * 170.0),
            18.0 + r.next() * 10.0,
            0.8,
            r.next() * 3.0,
            (150.0, 136.0, 110.0),
            0.22,
        );
        // the floor's dust
        for _ in 0..14 {
            let at = (r.next() * w, r.next() * h);
            soft(ctx, at, 12.0 + r.next() * 30.0, 0.5 + 0.5 * r.next(), r.next() * 3.0, (110.0, 100.0, 84.0), 0.08 + 0.12 * r.next());
        }
        ctx.set_fill_style("rgba(120,112,98,0.16)");
        ctx.fill_rect(0, 0, w, h);
        for _ in 0..400 {
            let (x, y) = (r.next() * w, r.next() * h);
            ctx.set_fill_style(rgba((90.0, 82.0, 70.0), 0.15 + 0.25 * r.next()));
            ctx.fill_rect(x, y, 1.0, 1.0);
        }
    })
}
