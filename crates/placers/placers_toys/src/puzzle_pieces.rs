//! `place_puzzle_pieces`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Puzzle piece: 9% chance per level. A piece of a children's jigsaw
// (Blender family `toys_on_the_floor`): 2 mm greyboard faced with printed paper, die cut
// with round knobs, 5 cm across; a border piece, or (the old second tab) an
// inner one with two. Its face a fragment of the picture, faded, rubbed
// back to the grey board at its corners and edges, dirty.
pub(crate) fn place_puzzle_pieces(g: &mut Game, _dist: &[i32]) {
    use game::assets::props::prop_part_as_modelled;
    if rnd() < 0.91 {
        return;
    } // 9% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    // the old colours stand for what the fragment shows: sky, a fence, grass,
    // straw, a tiled roof
    let piece_colors: [u32; 5] = [0x8ba0c0, 0xa0805a, 0x60a060, 0xd0a850, 0xc06070];
    let col = pick(&piece_colors);
    let k = piece_colors.iter().position(|&c| c == col).unwrap_or(0);
    let piece = group();
    // the old second tab: an inner piece (two tabs), else a border piece
    let inner = rnd() < 0.5;
    let (board_n, face_n) =
        if inner { ("Real__puzzlePieceB", "Real__puzzlePieceFaceB") } else { ("Real__puzzlePieceA", "Real__puzzlePieceFaceA") };
    if let Some(geo) = prop_part_as_modelled(board_n) {
        piece.add(&mesh(geo, lambert!({ color: 0xffffff, map: &puzzle_board_tex() })));
    }
    if let Some(geo) = prop_part_as_modelled(face_n) {
        let bb = geo.bounding_box();
        let lo = (bb.min.x, bb.min.z);
        let ext = (bb.max.x - bb.min.x, bb.max.z - bb.min.z);
        piece.add(&mesh(geo, lambert!({ color: 0xffffff, map: &puzzle_face_tex(k, inner, lo, ext) })));
    }
    piece.position().set(g.wx(x) + (rnd() - 0.5) * 2.5, 0, g.wz(z) + (rnd() - 0.5) * 2.5);
    piece.rotation().y = rnd() * PI * 2.0;
    g.world.add(&piece);
}

/// The jigsaw piece's body: half its side (the knobs stand out of it), as
/// modelled (toys/toys_on_the_floor.py PZ_B)
const PZ_HALF: f64 = 0.019;

/// The piece's printed face (laid on it from above: its x across the
/// canvas, its z down it, over the face's bounds `lo` + `ext`): the
/// fragment `k` of a child's picture in flat colours with dark outlines
/// (sky and a cloud, a fence's planks, grass and daisies, straw, a tiled
/// roof), faded, dirty, the print rubbed back to the grey board at the
/// body's corners and along its straight edges (a border piece's straight
/// side too)
fn puzzle_face_tex(k: usize, inner: bool, lo: (f64, f64), ext: (f64, f64)) -> Tex {
    canvas_tex_opt(128, 128, false, move |ctx, w, h| {
        let px = |x: f64, z: f64| ((x - lo.0) / ext.0 * w, (z - lo.1) / ext.1 * h);
        let line = |ctx: &mut Canvas, pts: &[(f64, f64)], lw: f64, style: &str| {
            ctx.set_line_width(lw);
            ctx.set_stroke_style(style);
            ctx.begin_path();
            for (i, &(x, y)) in pts.iter().enumerate() {
                if i == 0 {
                    ctx.move_to(x, y);
                } else {
                    ctx.line_to(x, y);
                }
            }
            ctx.stroke();
        };
        let ink = "rgba(46,40,38,0.75)";
        match k {
            0 => {
                // sky, and a cloud's edge coming in from one side
                let gr = ctx.create_linear_gradient(0, 0, 0, h);
                gr.add_color_stop(0, "#7b93b0");
                gr.add_color_stop(1, "#9db0c4");
                ctx.set_fill_style(gr);
                ctx.fill_rect(0, 0, w, h);
                let (cx, cy) = (w * (0.55 + rnd() * 0.3), h * (0.45 + rnd() * 0.3));
                let puffs = [(0.0, 0.0, 0.26), (-0.22, 0.08, 0.2), (0.2, 0.1, 0.22), (0.05, -0.16, 0.18)];
                for &(dx, dy, r) in &puffs {
                    ctx.set_fill_style("#d9d6cc");
                    ctx.begin_path();
                    ctx.arc(cx + dx * w, cy + dy * h, r * w, 0, 7);
                    ctx.fill();
                }
                for &(dx, dy, r) in &puffs {
                    ctx.set_line_width(2.0);
                    ctx.set_stroke_style(ink);
                    ctx.begin_path();
                    ctx.arc(cx + dx * w, cy + dy * h, r * w, 3.6, 6.0);
                    ctx.stroke();
                }
            }
            1 => {
                // a fence's planks, their gaps, the grain, a nail
                ctx.set_fill_style("#9b7a52");
                ctx.fill_rect(0, 0, w, h);
                let pw = w / 3.2;
                let off = rnd() * pw;
                for i in -1..5 {
                    let x0 = off + i as f64 * pw;
                    ctx.set_fill_style(if i % 2 == 0 { "rgba(120,90,58,0.5)" } else { "rgba(176,140,96,0.35)" });
                    ctx.fill_rect(x0, 0, pw, h);
                    for _ in 0..5 {
                        let gx = x0 + rnd() * pw;
                        line(ctx, &[(gx, 0.0), (gx + (rnd() - 0.5) * 6.0, h)], 1.0, "rgba(90,64,40,0.35)");
                    }
                    line(ctx, &[(x0, 0.0), (x0, h)], 2.6, ink);
                    let (nx, ny) = (x0 + pw * 0.5, h * (0.2 + rnd() * 0.6));
                    ctx.set_fill_style("rgba(40,36,34,0.7)");
                    ctx.begin_path();
                    ctx.arc(nx, ny, 1.8, 0, 7);
                    ctx.fill();
                }
            }
            2 => {
                // grass, its blades, a few daisies
                ctx.set_fill_style("#5b8a4e");
                ctx.fill_rect(0, 0, w, h);
                for _ in 0..140 {
                    let (x, y) = (rnd() * w, rnd() * h);
                    let l = 5.0 + rnd() * 9.0;
                    let style = if rnd() < 0.5 { "rgba(52,92,48,0.6)" } else { "rgba(120,160,96,0.5)" };
                    line(ctx, &[(x, y), (x + (rnd() - 0.5) * 4.0, y - l)], 1.2, style);
                }
                for _ in 0..4 {
                    let (x, y) = (rnd() * w, rnd() * h);
                    for p in 0..8 {
                        let a = p as f64 * PI / 4.0;
                        ctx.set_fill_style("#d8d2c2");
                        ctx.begin_path();
                        ctx.ellipse(x + a.cos() * 4.0, y + a.sin() * 4.0, 3.2, 1.6, a, 0, 7);
                        ctx.fill();
                    }
                    ctx.set_fill_style("#c09a3c");
                    ctx.begin_path();
                    ctx.arc(x, y, 2.4, 0, 7);
                    ctx.fill();
                }
            }
            3 => {
                // straw: thin strokes every way, ochre and brown
                ctx.set_fill_style("#c39f52");
                ctx.fill_rect(0, 0, w, h);
                for _ in 0..170 {
                    let (x, y) = (rnd() * w, rnd() * h);
                    let a = -0.5 + rnd() * 1.0 + if rnd() < 0.3 { 1.2 } else { 0.0 };
                    let l = 10.0 + rnd() * 18.0;
                    let style = if rnd() < 0.5 { "rgba(150,112,48,0.55)" } else { "rgba(224,196,128,0.5)" };
                    line(ctx, &[(x, y), (x + a.cos() * l, y + a.sin() * l)], 1.0, style);
                }
                line(ctx, &[(0.0, h * (0.3 + rnd() * 0.4)), (w, h * (0.2 + rnd() * 0.6))], 2.4, ink);
            }
            _ => {
                // a tiled roof: rows of round-ended tiles, outlined
                ctx.set_fill_style("#ab5a4e");
                ctx.fill_rect(0, 0, w, h);
                let tw = w / 4.0;
                let th = h / 5.0;
                for row in 0..7 {
                    let y = row as f64 * th - th * 0.5;
                    let off = if row % 2 == 0 { 0.0 } else { tw / 2.0 };
                    for c in -1..6 {
                        let x = off + c as f64 * tw;
                        ctx.set_fill_style(if (row + c) % 3 == 0 { "rgba(140,64,52,0.6)" } else { "rgba(196,110,92,0.35)" });
                        ctx.begin_path();
                        ctx.arc(x + tw / 2.0, y + th, tw / 2.0, 0, PI);
                        ctx.line_to(x, y);
                        ctx.line_to(x + tw, y);
                        ctx.fill();
                        ctx.set_line_width(2.0);
                        ctx.set_stroke_style(ink);
                        ctx.begin_path();
                        ctx.arc(x + tw / 2.0, y + th, tw / 2.0, 0, PI);
                        ctx.stroke();
                    }
                }
            }
        }
        // years: faded toward the board's grey-white, rubbed, dirty
        ctx.set_fill_style("rgba(214,208,194,0.16)");
        ctx.fill_rect(0, 0, w, h);
        for _ in 0..8 {
            let (x, y) = (rnd() * w, rnd() * h);
            let rr = 10.0 + rnd() * 22.0;
            let gr = ctx.create_radial_gradient(x, y, 0, x, y, rr);
            gr.add_color_stop(0, if rnd() < 0.5 { "rgba(220,214,200,0.22)" } else { "rgba(70,62,50,0.22)" });
            gr.add_color_stop(1, "rgba(0,0,0,0)");
            ctx.set_fill_style(gr);
            ctx.fill_rect(x - rr, y - rr, rr * 2.0, rr * 2.0);
        }
        for _ in 0..80 {
            ctx.set_fill_style(format!("rgba(58,52,44,{:.2})", 0.2 + rnd() * 0.4));
            ctx.fill_rect(rnd() * w, rnd() * h, 1, 1);
        }
        // the print rubbed off the body's straight edges (not at the knobs'
        // necks, a side's middle) and its corners, showing the grey board
        let board = "rgba(176,168,154,0.8)";
        let hh = PZ_HALF;
        let sides = [((-hh, hh), (hh, hh)), ((hh, -hh), (hh, hh)), ((-hh, -hh), (hh, -hh)), ((-hh, -hh), (-hh, hh))];
        for &((x0, z0), (x1, z1)) in &sides {
            for half in [(0.0, 0.28), (0.72, 1.0)] {
                let a = px(x0 + (x1 - x0) * half.0, z0 + (z1 - z0) * half.0);
                let b = px(x0 + (x1 - x0) * half.1, z0 + (z1 - z0) * half.1);
                line(ctx, &[a, b], 2.0 + rnd() * 1.5, board);
            }
        }
        // a border piece's straight side (-z) is worn all along
        if !inner {
            line(ctx, &[px(-hh, -hh), px(hh, -hh)], 2.4, board);
        }
        for &(cx, cz) in &[(-hh, -hh), (hh, -hh), (hh, hh), (-hh, hh)] {
            let (x, y) = px(cx, cz);
            let rr = 3.0 + rnd() * 4.0;
            let gr = ctx.create_radial_gradient(x, y, 0, x, y, rr);
            gr.add_color_stop(0, "rgba(176,168,154,0.95)");
            gr.add_color_stop(1, "rgba(176,168,154,0)");
            ctx.set_fill_style(gr);
            ctx.fill_rect(x - rr, y - rr, rr * 2.0, rr * 2.0);
        }
    })
}

/// The greyboard of its edges and back: grey-brown recycled card, its
/// fibres flecked, darker with dirt
fn puzzle_board_tex() -> Tex {
    canvas_tex(64, 64, |ctx, w, h| {
        ctx.set_fill_style("#8a8274");
        ctx.fill_rect(0, 0, w, h);
        for _ in 0..140 {
            let (x, y) = (rnd() * w, rnd() * h);
            ctx.set_fill_style(if rnd() < 0.5 { "rgba(170,162,146,0.45)" } else { "rgba(60,54,46,0.4)" });
            ctx.fill_rect(x, y, 1.0 + rnd() * 2.0, 1.0);
        }
    })
}
