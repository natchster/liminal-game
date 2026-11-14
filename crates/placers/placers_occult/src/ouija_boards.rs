//! `place_ouija_boards`
use game::prelude::*;
use placers_kit::geometry::put;
use placers_kit::noise::Rng32;
use placers_kit::noise::fbm;
use placers_kit::noise::mix;
use placers_kit::noise::sstep;
use placers_kit::noise::vn;
use placers_kit::paint::fill;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;
use placers_kit::place::salt;

// Ouija board decal: 4% chance per level. A printed talking board of
// the old kind (no maker's name on it): a lithographed face on 1/8 in
// hardboard, 18 x 12 in, its letters in two arcs, its numerals and the
// words in a serif face, a sun by YES and a moon by NO; warped, its print
// rubbed through where the planchette went round and round, grimed at its
// edges, a corner knocked off. Its planchette a wooden teardrop on three
// felted legs, lying still where it was left.
pub(crate) fn place_ouija_boards(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = pick(&open_tiles);
    let bx = g.wx(x) + (rnd() - 0.5) * 2.0;
    let bz = g.wz(z) + (rnd() - 0.5) * 2.0;
    let spin = rnd() * PI * 2.0;
    let salt = salt(bx, bz, 0x0b1);
    let board = group();
    put(&board, "Real__talkBoardCore", &lambert!({ map: &hardboard_tex(salt) }));
    put(
        &board,
        "Real__talkBoardFace",
        &lambert!({ map: &talk_board_tex(salt), polygonOffset: true, polygonOffsetFactor: -1, polygonOffsetUnits: -1 }),
    );
    board.position().set(bx, 0, bz);
    board.rotation().y = spin;
    g.world.add(&board);
    // the old planchette's offsets: where it lies on the board (in the
    // board's own frame now, so it always lies on the letters), its turn
    let lx = (rnd() - 0.5) * 0.30 * 0.8;
    let lz = (rnd() - 0.5) * 0.15 * 0.9;
    let pturn = rnd() * PI * 2.0;
    let (c, s) = (spin.dcos(), spin.dsin());
    let base_x = bx + lx * c + lz * s;
    let base_z = bz - lx * s + lz * c;
    let pl = group();
    put(&pl, "Real__talkPlanchette", &phong!({ map: &planchette_tex(salt), specular: 0x2a2620, shininess: 24 }));
    put(&pl, "Real__talkPlanchetteFelt", &lambert!({ color: 0x24261f }));
    pl.position().set(base_x, board_face_y(lx, lz), base_z);
    pl.rotation().y = pturn;
    g.world.add(&pl);
    // the old drift's two phases and its period (the planchette lies still:
    // one that moves by itself would be a working board)
    let _ = rnd();
    let _ = rnd();
    let _ = rnd();
}

/// The board (mm, `occult/shrine_and_seance_things.py`): its size and how it has warped, as modelled
const BOARD: (f64, f64) = (457.0, 305.0);

/// The height of the board's printed face at (x, z) of its frame (m)
fn board_face_y(x: f64, z: f64) -> f64 {
    let (u, w) = (x * 1000.0 / (BOARD.0 / 2.0), z * 1000.0 / (BOARD.1 / 2.0));
    (3.46 + 2.6 * u * u + 0.9 * u * w + 0.6 * w * w) / 1000.0
}

/// The board's printed face (its top: u across, v up the board), 2.24 px a
/// mm: the lithograph in a serif face on a tan ground, then its years
fn talk_board_tex(salt: u32) -> Tex {
    let k = 1024.0 / BOARD.0;
    canvas_tex_opt(1024.0, (BOARD.1 * k).round(), false, move |ctx, w, h| {
        let ink = "rgb(38,28,22)";
        ctx.set_fill_style("rgb(184,160,118)");
        ctx.fill_rect(0, 0, w, h);
        ctx.save();
        ctx.scale(k, k);
        // the border: a double rule
        ctx.set_stroke_style(ink);
        ctx.set_line_width(1.6);
        ctx.stroke_rect(9.0, 9.0, BOARD.0 - 18.0, BOARD.1 - 18.0);
        ctx.set_line_width(0.6);
        ctx.stroke_rect(12.5, 12.5, BOARD.0 - 25.0, BOARD.1 - 25.0);
        ctx.set_fill_style(ink);
        ctx.set_text_align("center");
        ctx.set_text_baseline("middle");
        // the sun by YES, the moon by NO
        ctx.begin_path();
        ctx.arc(38.0, 40.0, 11.0, 0, PI * 2.0);
        ctx.fill();
        ctx.set_line_width(1.4);
        for i in 0..16 {
            let a = f64::from(i) * PI / 8.0;
            let (r0, r1) = (14.0, if i % 2 == 0 { 22.0 } else { 19.0 });
            ctx.begin_path();
            ctx.move_to(38.0 + a.dcos() * r0, 40.0 + a.dsin() * r0);
            ctx.line_to(38.0 + a.dcos() * r1, 40.0 + a.dsin() * r1);
            ctx.stroke();
        }
        ctx.begin_path();
        ctx.arc(420.0, 40.0, 15.0, 0, PI * 2.0);
        ctx.fill();
        ctx.set_fill_style("rgb(184,160,118)");
        ctx.begin_path();
        ctx.arc(427.0, 35.0, 13.5, 0, PI * 2.0);
        ctx.fill();
        ctx.set_fill_style(ink);
        ctx.set_font("bold 22px serif");
        ctx.fill_text("YES", 82.0, 42.0);
        ctx.fill_text("NO", 378.0, 42.0);
        // the letters in two arcs
        ctx.set_font("bold 27px serif");
        let arc = |ctx: &mut Canvas, letters: &str, cy: f64, rad: f64, span: f64| {
            let n = letters.chars().count();
            for (i, ch) in letters.chars().enumerate() {
                let a = -span + 2.0 * span * i as f64 / (n - 1) as f64;
                ctx.save();
                ctx.translate(BOARD.0 / 2.0 + rad * a.dsin(), cy - rad * a.dcos());
                ctx.rotate(a);
                ctx.fill_text(&ch.to_string(), 0.0, 0.0);
                ctx.restore();
            }
        };
        arc(ctx, "ABCDEFGHIJKLM", 560.0, 438.0, 0.36);
        arc(ctx, "NOPQRSTUVWXYZ", 604.0, 428.0, 0.35);
        ctx.set_font("bold 24px serif");
        for (i, d) in "1234567890".chars().enumerate() {
            ctx.fill_text(&d.to_string(), 118.0 + 24.6 * i as f64, 232.0);
        }
        ctx.set_font("bold 21px serif");
        ctx.fill_text("GOOD BYE", BOARD.0 / 2.0, 272.0);
        ctx.restore();
        // its years: the tan gone uneven, the print rubbed through in a band
        // where the planchette went round, scratches, grime at the edges, a
        // tide line, dust; the corner knocked off to the hardboard
        let src = ctx.get_image_data(0, 0, w, h);
        let iw = w as usize;
        let mut img = ctx.create_image_data(w, h);
        let seed = salt ^ 0xb0a;
        let scratches: Vec<(f64, f64, f64, f64)> = {
            let mut r = Rng32(seed | 1);
            (0..9).map(|_| (r.f() * BOARD.0, r.f() * BOARD.1, r.f() * PI, 30.0 + 120.0 * r.f())).collect()
        };
        par_rows(&mut img.data, iw * 4, |j, out| {
            for i in 0..iw {
                let (x, y) = ((i as f64 + 0.5) / k, (j as f64 + 0.5) / k);
                let o = (j * iw + i) * 4;
                let px = [f64::from(src.data[o]) / 255.0, f64::from(src.data[o + 1]) / 255.0, f64::from(src.data[o + 2]) / 255.0];
                let paper = [184.0 / 255.0, 160.0 / 255.0, 118.0 / 255.0];
                let inkness = (1.0 - (px[0] - 0.149) / (paper[0] - 0.149)).clamp(0.0, 1.0);
                // the ground mottled, the ink lifted where rubbed
                let n = fbm(seed, x * 0.03, y * 0.03, 4);
                let mut c = mix(paper, [0.62, 0.52, 0.37], n * 0.7);
                let (rx, ry) = ((x - BOARD.0 / 2.0) / 175.0, (y - 185.0) / 78.0);
                let rub = (1.0 - sstep(0.6, 1.15, (rx * rx + ry * ry).sqrt())) * (0.55 + 0.45 * vn(seed ^ 5, x * 0.4, y * 0.04));
                let ink_left = inkness * (1.0 - 0.55 * rub);
                c = mix(c, [0.16, 0.12, 0.09], ink_left * (0.85 + 0.15 * vn(seed ^ 6, x * 2.0, y * 2.0)));
                // rubbed to the brown under the print in the worst of it
                c = mix(c, [0.52, 0.42, 0.30], rub * 0.35 * sstep(0.55, 0.8, vn(seed ^ 7, x * 0.25, y * 0.9)));
                for &(sx, sy, sa, sl) in &scratches {
                    let (dx, dy) = (x - sx, y - sy);
                    let along = dx * sa.dcos() + dy * sa.dsin();
                    let across = (-dx * sa.dsin() + dy * sa.dcos()).abs();
                    if along.abs() < sl / 2.0 && across < 0.45 {
                        c = mix(c, [0.78, 0.72, 0.60], 0.55 * (1.0 - across / 0.45));
                    }
                }
                // grime at the edges, darker at the near edge where hands held it
                let edge = x.min(BOARD.0 - x).min(y).min(BOARD.1 - y);
                let grime =
                    (1.0 - sstep(2.0, 26.0, edge)) * (0.4 + 0.4 * fbm(seed ^ 8, x * 0.08, y * 0.08, 2)) + sstep(255.0, 305.0, y) * 0.12;
                c = mix(c, [0.24, 0.19, 0.13], grime);
                // a tide line where water stood on its far left
                let tq = ((x - 70.0).dpowi(2) / 1.6 + (y - 70.0).dpowi(2)).sqrt() / 92.0 + (vn(seed ^ 9, x * 0.05, y * 0.05) - 0.5) * 0.25;
                c = mix(
                    c,
                    [0.46, 0.36, 0.24],
                    sstep(0.84, 0.97, tq) * (1.0 - sstep(0.97, 1.03, tq)) * 0.6 + (1.0 - sstep(0.0, 0.97, tq)) * 0.08,
                );
                // the knocked corner (near right): the face gone to the board
                let cd = ((x - (BOARD.0 - 4.0)).dpowi(2) + (y - (BOARD.1 - 6.0)).dpowi(2)).sqrt()
                    + (vn(seed ^ 10, x * 0.5, y * 0.5) - 0.5) * 5.0;
                if cd < 12.0 {
                    c = mix(c, [0.33, 0.24, 0.15], 0.9);
                }
                // dust
                let d = 0.10 + 0.08 * fbm(seed ^ 11, x * 0.05, y * 0.05, 2);
                c = mix(c, [0.62, 0.60, 0.56], d);
                for kk in 0..3 {
                    out[i * 4 + kk] = (c[kk].clamp(0.0, 1.0) * 255.0).round() as u8;
                }
                out[i * 4 + 3] = 255;
            }
        });
        ctx.put_image_data(&img, 0, 0);
    })
}

/// The board's hardboard (its edges and its back): brown, pressed smooth
/// one side and screen-textured the other, grimed
fn hardboard_tex(salt: u32) -> Tex {
    canvas_tex_opt(128, 128, false, move |ctx, w, h| {
        let seed = salt ^ 0x4bd;
        fill(ctx, w, h, |u, v| {
            let n = fbm(seed, u * 9.0, v * 9.0, 3);
            let c = mix([0.30, 0.22, 0.15], [0.22, 0.16, 0.11], n);
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// The planchette's wood (maple, varnished once, gone amber): its grain,
/// darker with handling round its rim, dust on top
fn planchette_tex(salt: u32) -> Tex {
    canvas_tex_opt(64, 64, false, move |ctx, w, h| {
        let seed = salt ^ 0x91a;
        fill(ctx, w, h, |u, v| {
            let g = (vn(seed, u * 3.0, v * 40.0) * 0.6 + fbm(seed ^ 3, u * 8.0, v * 8.0, 2) * 0.4).clamp(0.0, 1.0);
            let mut c = mix([0.54, 0.43, 0.30], [0.43, 0.33, 0.22], g);
            let rim = (((u - 0.5).abs().max((v - 0.5).abs())) - 0.36).max(0.0) * 6.0;
            c = mix(c, [0.26, 0.18, 0.11], rim.min(0.7));
            [c[0], c[1], c[2], 1.0]
        });
    })
}
