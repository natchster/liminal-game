//! Wall crafts, hand mirror, plants and garden tools.
use crate::{face_yaw, open_tiles, pick, set_rot_order, wall_tiles};
use game::prelude::*;

// Hanging plant: made real by batch `t06` (`t06.rs`)
pub use crate::t06::place_hanging_plants;

// Gardening gloves, garden rake: made real by batch `t17`
// (`garden/t17.rs`)
mod t17;
pub use t17::{place_garden_rakes, place_gardening_gloves};

// Framed cross-stitch: 4% chance per level. An alphabet sampler on
// linen, framed (batch r13a).
pub fn place_cross_stitches(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let walls = wall_tiles(g);
    if walls.is_empty() {
        return;
    }
    let (x, z, dx, dz) = pick(&walls);
    // An alphabet sampler (Blender batch `r13a`): cotton cross stitches on
    // linen, 14 to the inch, stretched and framed in a 30 mm oak scoop
    // moulding, 22 x 28 cm; its border, the alphabet, the numerals, a house
    // between two trees, a motto, the maker's initials and the year; the
    // colours faded, the linen yellowed and foxed; hung a little askew
    // (the old tilt's draw) (docs/design/PROPS.md)
    use game::assets::props::prop_part_as_modelled;
    let stitch = group();
    let mount_y = 1.55;
    let tilt = (rnd() - 0.5) * 0.06;
    stitch.rotation().y = face_yaw(dx, dz);
    stitch.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - 0.013), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - 0.013));
    let p = stitch.pos();
    let vary = sp_vary(p.x, p.z, tilt);
    // askew as the old one hung (turned about the same point, so it hangs
    // where it did); the parts are modelled with the wall's face at z = 0:
    // 13 mm behind the spot
    stitch.rotation().z = tilt;
    let body = group();
    body.position().set(0, mount_y, -0.013);
    stitch.add(&body);
    if let Some(geo) = prop_part_as_modelled("Real__samplerFrame") {
        let (fw, fh) = (SP_LINEN[0] + 2.0 * SP_MOULD, SP_LINEN[1] + 2.0 * SP_MOULD);
        let tex = sp_grain_tex(fw, fh, SP_MOULD, [0x76, 0x58, 0x3a], vary, 1000.0);
        body.add(&mesh(geo, lambert!({ map: &tex })));
    }
    let (cw, ch) = (SP_CELLS[0] as f64 * SP_PITCH, SP_CELLS[1] as f64 * SP_PITCH);
    let linen = mesh(plane_geo!(cw, ch), lambert!({ map: &sampler_tex(vary) }));
    linen.position().z = SP_LINEN[2] - 0.0002;
    body.add(&linen);
    g.world.add(&stitch);
}

/// The linen seen in the frame (w, h) and its face's distance from the
/// wall, the moulding's width (`Real__samplerFrame`, r13a.py)
const SP_LINEN: [f64; 3] = [0.170, 0.230, 0.0080];
const SP_MOULD: f64 = 0.025;
/// 14 stitches to the inch, and the stitched linen's cells (a little more
/// than is seen: its edges run under the frame's lip)
const SP_PITCH: f64 = 0.0254 / 14.0;
const SP_CELLS: [usize; 2] = [96, 129];

use placers_kit::place::vary as sp_vary;

/// A sampler's alphabet: 5 x 7 cells a letter, each row's bits left to right
fn sp_letter(c: char) -> [u8; 7] {
    match c {
        'A' => [0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001],
        'B' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10001, 0b10001, 0b11110],
        'C' => [0b01110, 0b10001, 0b10000, 0b10000, 0b10000, 0b10001, 0b01110],
        'D' => [0b11110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11110],
        'E' => [0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111],
        'F' => [0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000],
        'G' => [0b01110, 0b10001, 0b10000, 0b10111, 0b10001, 0b10001, 0b01111],
        'H' => [0b10001, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001],
        'I' => [0b01110, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110],
        'J' => [0b00111, 0b00010, 0b00010, 0b00010, 0b00010, 0b10010, 0b01100],
        'K' => [0b10001, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b10001],
        'L' => [0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111],
        'M' => [0b10001, 0b11011, 0b10101, 0b10101, 0b10001, 0b10001, 0b10001],
        'N' => [0b10001, 0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001],
        'O' => [0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110],
        'P' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000],
        'Q' => [0b01110, 0b10001, 0b10001, 0b10001, 0b10101, 0b10010, 0b01101],
        'R' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001],
        'S' => [0b01111, 0b10000, 0b10000, 0b01110, 0b00001, 0b00001, 0b11110],
        'T' => [0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100],
        'U' => [0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110],
        'V' => [0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01010, 0b00100],
        'W' => [0b10001, 0b10001, 0b10001, 0b10101, 0b10101, 0b10101, 0b01010],
        'X' => [0b10001, 0b10001, 0b01010, 0b00100, 0b01010, 0b10001, 0b10001],
        'Y' => [0b10001, 0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b00100],
        'Z' => [0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b10000, 0b11111],
        '0' => [0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110],
        '1' => [0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110],
        '2' => [0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b01000, 0b11111],
        '3' => [0b11111, 0b00010, 0b00100, 0b00010, 0b00001, 0b10001, 0b01110],
        '4' => [0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010],
        '5' => [0b11111, 0b10000, 0b11110, 0b00001, 0b00001, 0b10001, 0b01110],
        '6' => [0b00110, 0b01000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110],
        '7' => [0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000],
        '8' => [0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110],
        '9' => [0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00010, 0b01100],
        '.' => [0, 0, 0, 0, 0, 0b00100, 0],
        _ => [0; 7],
    }
}

/// The sampler's chart: a colour (1..) or none (0) for each cell. Who
/// stitched it and when, and its motto, from `pick`
fn sp_chart(pick: u32) -> Vec<u8> {
    let (w, h) = (SP_CELLS[0], SP_CELLS[1]);
    let mut c = vec![0u8; w * h];
    let mut set = |x: usize, y: usize, k: u8, c: &mut Vec<u8>| {
        if x < w && y < h {
            c[y * w + x] = k;
        }
    };
    // colours: 1 madder red, 2 indigo, 3 green, 4 ochre, 5 brown, 6 black
    // the border: two red lines, a green zigzag between them
    for y in 0..h {
        for x in 0..w {
            let e = x.min(w - 1 - x).min(y).min(h - 1 - y);
            let t = if y.min(h - 1 - y) <= x.min(w - 1 - x) { x } else { y };
            if e == 3 || e == 9 {
                set(x, y, 1, &mut c);
            } else if (4..=8).contains(&e) && e == 4 + (t % 8).abs_diff(4) {
                set(x, y, 3, &mut c);
            }
        }
    }
    let text = |s: &str, y: usize, k: u8, c: &mut Vec<u8>| {
        let n = s.chars().count();
        let x0 = (w - (n * 6 - 1)) / 2;
        for (i, ch) in s.chars().enumerate() {
            let rows = sp_letter(ch);
            for (r, bits) in rows.iter().enumerate() {
                for b in 0..5 {
                    if bits >> (4 - b) & 1 == 1 {
                        set(x0 + i * 6 + b, y + r, k, c);
                    }
                }
            }
        }
    };
    let band = |y: usize, k: u8, c: &mut Vec<u8>| {
        for x in (13..w - 13).step_by(2) {
            set(x, y, k, c);
        }
    };
    text("ABCDEFGHI", 12, 1, &mut c);
    text("JKLMNOPQR", 21, 2, &mut c);
    text("STUVWXYZ", 30, 3, &mut c);
    band(39, 4, &mut c);
    text("1234567890", 42, 1, &mut c);
    band(51, 4, &mut c);
    // the house: its chimney stack first, the roof's slope over its foot,
    // the walls, two windows and a door
    let (hx, hy) = (w / 2 - 12, 55);
    for y in 0..5 {
        for x in 16..19 {
            set(hx + x, hy + y, 5, &mut c);
        }
    }
    for r in 0..10 {
        for x in (12 - r - 2).max(0)..=(12 + r + 2) {
            set(hx + x, hy + r + 1, 1, &mut c);
        }
    }
    for y in 10..23 {
        for x in 2..23 {
            let win = (5..9).contains(&x) && (13..17).contains(&y) || (16..20).contains(&x) && (13..17).contains(&y);
            let door = (10..15).contains(&x) && y >= 16;
            set(
                hx + x,
                hy + y,
                if win {
                    6
                } else if door {
                    5
                } else {
                    4
                },
                &mut c,
            );
        }
    }
    // the trees either side, and the ground
    for &tx in &[hx - 14, hx + 38] {
        for y in 0..12 {
            let half = 5 - (y as i32 - 5).abs().min(5);
            for x in -(half + 1)..=(half + 1) {
                set((tx as i32 + x) as usize, hy + 4 + y, 3, &mut c);
            }
        }
        for y in 16..23 {
            set(tx, hy + y, 5, &mut c);
            set(tx + 1, hy + y, 5, &mut c);
        }
    }
    for x in 13..w - 13 {
        set(x, hy + 23, 3, &mut c);
    }
    const WHO: [&str; 6] = ["E.A.B.", "J.R.W.", "A.L.T.", "R.S.K.", "D.E.P.", "H.J.C."];
    const MOTTO: [&str; 2] = ["GOD IS LOVE", "HOME IS BEST"];
    let motto = MOTTO[(pick >> 4) as usize % MOTTO.len()];
    text(motto, 82, 2, &mut c);
    let year = 1946 + (pick >> 9) % 27;
    text(&format!("{}  {year}", WHO[pick as usize % WHO.len()]), 92, 1, &mut c);
    // a row of tulips
    for i in 0..6 {
        let fx = 17 + i * 11;
        let k = [1u8, 2, 1, 4, 1, 2][i];
        for (y, row) in ["0101010", "0111110", "0111110", "0011100", "0001000", "0001000", "0011100", "0001000"].iter().enumerate() {
            for (x, b) in row.chars().enumerate() {
                if b == '1' {
                    set(fx + x, 102 + y, if y < 4 { k } else { 3 }, &mut c);
                }
            }
        }
    }
    band(113, 4, &mut c);
    c
}

use placers_kit::noise::hash as sp_hash;

use placers_kit::noise::vn as sp_noise;

/// The stitched linen, 6 px a stitch: the linen's weave (two threads a
/// stitch each way, the holes between them), each stitch a cross of two
/// strands of stranded cotton laid corner to corner (the top one always
/// the same way, as a careful hand works), round and lit along their length;
/// the colours faded, most where the light reached; the linen yellowed,
/// foxed with brown spots, darker under the frame's lip. Each pixel from
/// its cell, no canvas call in the loop
fn sampler_tex(vary: u32) -> Tex {
    const S: usize = 6;
    let (n, m) = (SP_CELLS[0] * S, SP_CELLS[1] * S);
    canvas_tex_opt(n as f64, m as f64, false, move |ctx, w, h| {
        let seed = (rnd() * 4.0e9) as u32 ^ vary;
        let chart = sp_chart(vary >> 5);
        let fade = 0.25 + 0.25 * sp_hash(seed, 1, 2);
        let pal: [[f64; 3]; 7] = [
            [0.0; 3],
            [150.0, 56.0, 46.0],
            [56.0, 66.0, 104.0],
            [70.0, 98.0, 62.0],
            [184.0, 146.0, 64.0],
            [96.0, 64.0, 42.0],
            [44.0, 40.0, 38.0],
        ];
        let linen = [208.0, 194.0, 160.0];
        let mut img = ctx.create_image_data(w, h);
        par_rows(&mut img.data, n * 4, |j, out| {
            let (cy, py) = (j / S, j % S);
            for i in 0..n {
                let (cx, px) = (i / S, i % S);
                let (u, v) = ((px as f64 + 0.5) / S as f64, (py as f64 + 0.5) / S as f64);
                // the linen: threads 3 px apart each way, the holes between them
                let (tx, ty) = (i % 3, j % 3);
                let slub = 0.92 + 0.12 * sp_hash(seed ^ 0x31, (i / 3) as i32, 0) * 0.5 + 0.06 * sp_hash(seed ^ 0x32, 0, (j / 3) as i32);
                let weave = if tx == 0 && ty == 0 {
                    0.7
                } else if tx == 0 || ty == 0 {
                    0.9
                } else {
                    1.0
                };
                let mut col = [linen[0] * weave * slub, linen[1] * weave * slub, linen[2] * weave * slub];
                let k = chart[cy * SP_CELLS[0] + cx] as usize;
                if k > 0 {
                    // a stitch: the bottom strand "\", the top one "/" over it
                    let jit = (sp_hash(seed ^ 0x41, cx as i32, cy as i32) - 0.5) * 0.08;
                    let hw = 0.19 + 0.04 * sp_hash(seed ^ 0x42, cx as i32, cy as i32);
                    let d_top = (u + v - 1.0 + jit).abs() / 2f64.sqrt();
                    let d_bot = (u - v - jit).abs() / 2f64.sqrt();
                    let shade = if d_top < hw {
                        let tw = 0.07 * (((u - v) * 7.0 + cx as f64) * PI).sin();
                        Some(1.02 - 0.5 * (d_top / hw).powi(2) + tw)
                    } else if d_bot < hw {
                        let tw = 0.06 * (((u + v) * 7.0 + cy as f64) * PI).sin();
                        Some(0.88 - 0.45 * (d_bot / hw).powi(2) + tw)
                    } else {
                        None
                    };
                    if let Some(s) = shade {
                        let c = pal[k];
                        // faded: toward the linen, more toward the top where the light fell
                        let f = (fade * (1.15 - 0.4 * cy as f64 / SP_CELLS[1] as f64)).min(0.7);
                        for q in 0..3 {
                            col[q] = (c[q] + (linen[q] * 0.85 - c[q]) * f) * s;
                        }
                    }
                }
                // yellowed and foxed; darker in the margin under the lip
                let (x, y) = (i as f64, j as f64);
                let fox =
                    (sp_noise(seed ^ 0x51, x * 0.05, y * 0.05) * 0.6 + sp_noise(seed ^ 0x52, x * 0.2, y * 0.2) * 0.4 - 0.64).max(0.0) * 6.0;
                let spot = if sp_hash(seed ^ 0x53, (x / 7.0) as i32, (y / 7.0) as i32) > 0.985 { 0.5 } else { 0.0 };
                let foxing = (fox + spot * sp_noise(seed ^ 0x54, x * 0.3, y * 0.3)).min(0.6);
                let edge = (i.min(n - 1 - i).min(j).min(m - 1 - j)) as f64;
                let rim = 1.0 - 0.22 * (1.0 - (edge / 14.0).min(1.0));
                let yel = [0.98, 0.95, 0.86];
                for q in 0..3 {
                    let fx = [146.0, 104.0, 62.0][q];
                    let c = col[q] * yel[q] * rim;
                    out[i * 4 + q] = (c + (fx - c) * foxing * 0.5).round().clamp(0.0, 255.0) as u8;
                }
                out[i * 4 + 3] = 255;
            }
        });
        ctx.put_image_data(&img, 0, 0);
    })
}

/// A soft blob, squashed whole (`k`) and turned, so its fade is never cut
fn sp_blob(ctx: &mut Canvas, x: f64, y: f64, r: f64, k: f64, turn: f64, stops: &[(f64, String)]) {
    ctx.save();
    ctx.translate(x, y);
    ctx.rotate(turn);
    ctx.scale(1.0, k);
    let g = ctx.create_radial_gradient(0, 0, 0, 0, 0, r);
    for (at, c) in stops {
        g.add_color_stop(*at, c);
    }
    ctx.set_fill_style(&g);
    ctx.begin_path();
    ctx.arc(0, 0, r, 0, 7);
    ctx.fill();
    ctx.restore();
}

/// A moulded frame's wood as its front is mapped (the frame's bounds, `w` x
/// `h` m, its rails `mould` m wide, `ppm` px a metre): each rail's grain
/// along it, mitred at the corners, the joints opened a hair; stained `base`
/// (sRGB), its figure darker, the stain rubbed thin along its outer edge and
/// its sight edge, knocks and scratches, dust along the top rail. `vary`
/// mixes the frame's place into its stream
fn sp_grain_tex(w: f64, h: f64, mould: f64, base: [u8; 3], vary: u32, ppm: f64) -> Tex {
    let (pw, ph) = ((w * ppm).round(), (h * ppm).round());
    canvas_tex_opt(pw, ph, false, move |ctx, cw, ch| {
        let seed = (rnd() * 4.0e9) as u32 ^ vary;
        with_own_stream(seed, || {
            let m = mould * ppm;
            let [r0, g0, b0] = base.map(f64::from);
            let col = |k: f64, a: f64| format!("rgba({:.0},{:.0},{:.0},{a:.3})", r0 * k, g0 * k, b0 * k);
            ctx.set_fill_style(col(1.0, 1.0));
            ctx.fill_rect(0, 0, cw, ch);
            // the four rails: (outline, along x?)
            let rails: [([(f64, f64); 4], bool); 4] = [
                ([(0.0, 0.0), (cw, 0.0), (cw - m, m), (m, m)], true),
                ([(0.0, ch), (cw, ch), (cw - m, ch - m), (m, ch - m)], true),
                ([(0.0, 0.0), (m, m), (m, ch - m), (0.0, ch)], false),
                ([(cw, 0.0), (cw - m, m), (cw - m, ch - m), (cw, ch)], false),
            ];
            for (poly, along_x) in rails.iter() {
                ctx.save();
                ctx.begin_path();
                ctx.move_to(poly[0].0, poly[0].1);
                for q in &poly[1..] {
                    ctx.line_to(q.0, q.1);
                }
                ctx.close_path();
                ctx.clip();
                // each rail its own board: a shade of its own
                ctx.set_fill_style(col(0.88 + 0.24 * rnd(), 0.5));
                ctx.fill_rect(0, 0, cw, ch);
                let len = if *along_x { cw } else { ch };
                let (lo, hi) = match (poly[0], along_x) {
                    ((_, y), true) if y < 1.0 => (0.0, m),
                    (_, true) => (ch - m, ch),
                    ((x, _), false) if x < 1.0 => (0.0, m),
                    _ => (cw - m, cw),
                };
                // the grain: long wavering lines along the rail, darker figure among them
                let lines = (m * 2.2) as usize;
                for _ in 0..lines {
                    let across = lo + rnd() * (hi - lo);
                    let (amp, fr, ph) = (0.4 + rnd() * 1.6, 1.0 + rnd() * 3.0, rnd() * 6.3);
                    let dark = rnd() < 0.3;
                    ctx.set_stroke_style(col(if dark { 0.55 } else { 0.72 + 0.2 * rnd() }, if dark { 0.35 } else { 0.18 + 0.2 * rnd() }));
                    ctx.set_line_width(if dark { 0.8 + rnd() * 1.4 } else { 0.4 + rnd() * 0.8 });
                    ctx.begin_path();
                    let n = (len / 8.0) as usize + 2;
                    for k in 0..=n {
                        let t = k as f64 / n as f64 * len;
                        let a = across + amp * (t / len * fr * 6.28 + ph).sin();
                        let (x, y) = if *along_x { (t, a) } else { (a, t) };
                        if k == 0 {
                            ctx.move_to(x, y);
                        } else {
                            ctx.line_to(x, y);
                        }
                    }
                    ctx.stroke();
                }
                // pores, short dark flecks along the grain
                for _ in 0..(len * m / 60.0) as usize {
                    let (t, a) = (rnd() * len, lo + rnd() * (hi - lo));
                    let l = 1.5 + rnd() * 4.0;
                    ctx.set_stroke_style(col(0.45, 0.25 + 0.2 * rnd()));
                    ctx.set_line_width(0.6);
                    ctx.begin_path();
                    let (x, y, dx, dy) = if *along_x { (t, a, l, 0.0) } else { (a, t, 0.0, l) };
                    ctx.move_to(x, y);
                    ctx.line_to(x + dx, y + dy);
                    ctx.stroke();
                }
                ctx.restore();
            }
            // the mitres: each joint opened a hair, grime in it
            ctx.set_stroke_style("rgba(20,14,10,0.55)");
            ctx.set_line_width(0.9);
            for (a, b) in [((0.0, 0.0), (m, m)), ((cw, 0.0), (cw - m, m)), ((0.0, ch), (m, ch - m)), ((cw, ch), (cw - m, ch - m))] {
                ctx.begin_path();
                ctx.move_to(a.0, a.1);
                ctx.line_to(b.0, b.1);
                ctx.stroke();
            }
            // the stain rubbed thin along the outer edge and the sight edge
            ctx.set_line_width(2.4);
            for inset in [1.2, m - 1.4] {
                ctx.set_stroke_style("rgba(222,196,160,0.14)");
                ctx.stroke_rect(inset, inset, cw - 2.0 * inset, ch - 2.0 * inset);
            }
            // knocks and scratches
            for _ in 0..(cw + ch) as usize / 14 {
                let edge = rnd() < 0.7;
                let (x, y) = if rnd() < 0.5 {
                    (
                        rnd() * cw,
                        if rnd() < 0.5 {
                            if edge {
                                1.0
                            } else {
                                rnd() * m
                            }
                        } else if edge {
                            ch - 1.0
                        } else {
                            ch - rnd() * m
                        },
                    )
                } else {
                    (
                        if rnd() < 0.5 {
                            if edge {
                                1.0
                            } else {
                                rnd() * m
                            }
                        } else if edge {
                            cw - 1.0
                        } else {
                            cw - rnd() * m
                        },
                        rnd() * ch,
                    )
                };
                if rnd() < 0.5 {
                    let stops = [(0.0, "rgba(214,182,140,0.45)".to_string()), (1.0, "rgba(214,182,140,0)".into())];
                    sp_blob(ctx, x, y, 0.8 + rnd() * 2.2, 0.5 + 0.5 * rnd(), rnd() * 3.0, &stops);
                } else {
                    let a = rnd() * 6.3;
                    let l = 3.0 + rnd() * 12.0;
                    ctx.set_stroke_style(format!("rgba(200,170,130,{:.2})", 0.12 + 0.18 * rnd()));
                    ctx.set_line_width(0.5);
                    ctx.begin_path();
                    ctx.move_to(x, y);
                    ctx.line_to(x + a.cos() * l, y + a.sin() * l);
                    ctx.stroke();
                }
            }
            // dust along the top rail, and grime where hands set it straight
            let gd = ctx.create_linear_gradient(0, 0, 0, m);
            gd.add_color_stop(0, "rgba(168,162,150,0.32)");
            gd.add_color_stop(1, "rgba(168,162,150,0.08)");
            ctx.set_fill_style(&gd);
            ctx.fill_rect(m * 0.4, 0, cw - m * 0.8, m);
            for x in [m * 0.6, cw - m * 0.6] {
                let stops = [(0.0, "rgba(28,20,14,0.22)".to_string()), (1.0, "rgba(28,20,14,0)".into())];
                sp_blob(ctx, x, ch * (0.55 + 0.2 * rnd()), m * 1.4, 0.6, PI / 2.0, &stops);
            }
        })
    })
}

// Hand mirror: 6% chance per level. Round mirror w/ long handle laid flat
// on floor.
pub fn place_hand_mirrors(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let frame_colors: [u32; 5] = [0xd0a040, 0x606060, 0x804060, 0xc02020, 0x101010];
    let frame_mat = lambert!({ color: pick(&frame_colors) });
    let glass_mat = lambert!({ color: 0xe0e8f0 });
    let mirror = group();
    // Round frame disc.
    let frame = mesh(pg("HandMirrors__frame", cyl_geo!(0.030, 0.030, 0.005, 12)), &frame_mat);
    frame.position().y = 0.003;
    mirror.add(&frame);
    // Glass surface (slightly smaller silver disc on top).
    let glass = mesh(circle_geo!(0.026, 12), &glass_mat);
    glass.rotation().x = -PI / 2.0;
    glass.position().y = 0.006;
    mirror.add(&glass);
    // Handle (long thin rectangle extending).
    let handle = mesh(pg("HandMirrors__handle", box_geo!(0.008, 0.006, 0.070)), &frame_mat);
    handle.position().set(0, 0.003, 0.060);
    mirror.add(&handle);
    // Small grip knob at end.
    let knob = mesh(pg("HandMirrors__knob", sphere_geo!(0.008, 6, 5)), &frame_mat);
    knob.scale().set_scalar(0.625); // r 5 mm: a finial just wider than the 8 mm handle, resting on the floor
    knob.position().set(0, 0.005, 0.098);
    mirror.add(&knob);
    mirror.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    mirror.rotation().y = rnd() * PI * 2.0;
    g.world.add(&mirror);
}

// Framed watercolor: 4% chance per level. Ornate frame + pastel abstract
// wash inside.
game::lazy_static_tex!(
    watercolor_texture,
    canvas_tex(128, 96, |ctx, w, h| {
        // Soft off-white paper.
        ctx.set_fill_style("#f0ecdc");
        ctx.fill_rect(0, 0, w, h);
        // 5 large soft blobs of pastel color.
        let pastels = ["#a0c0e8", "#e0b0c0", "#c0d8a8", "#e8d090", "#c0a0d0"];
        for i in 0..8 {
            ctx.set_fill_style(&format!("{}a0", pastels[i % pastels.len()])); // alpha ~62%.
            ctx.begin_path();
            let cx = rnd() * w;
            let cy = rnd() * h;
            let r = 8.0 + rnd() * 24.0;
            ctx.arc(cx, cy, r, 0, PI * 2.0);
            ctx.fill();
        }
        // Thin dark accent lines (brush strokes).
        ctx.set_stroke_style("#403020");
        ctx.set_line_width(1);
        for _i in 0..3 {
            ctx.begin_path();
            let (mx, my) = (rnd() * w, rnd() * h);
            ctx.move_to(mx, my);
            let (c1x, c1y) = (rnd() * w, rnd() * h);
            let (c2x, c2y) = (rnd() * w, rnd() * h);
            let (ex, ey) = (rnd() * w, rnd() * h);
            ctx.bezier_curve_to(c1x, c1y, c2x, c2y, ex, ey);
            ctx.stroke();
        }
    })
);

pub fn place_watercolors(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let walls = wall_tiles(g);
    if walls.is_empty() {
        return;
    }
    let (x, z, dx, dz) = pick(&walls);
    let frame_mat = lambert!({ color: 0xd0a040 });
    let matting_mat = lambert!({ color: 0xf0eddb });
    let paint_mat = lambert!({ map: watercolor_texture() });
    let painting = group();
    let mount_y = 1.65;
    // Gold outer frame.
    let frame = mesh(pg("Watercolors__frame", box_geo!(0.34, 0.26, 0.014)), &frame_mat);
    frame.position().y = mount_y;
    painting.add(&frame);
    // White matting inset.
    let matting = mesh(pg("Watercolors__mat", box_geo!(0.28, 0.20, 0.010)), &matting_mat);
    matting.position().set(0, mount_y, 0.008);
    painting.add(&matting);
    // Watercolor paper surface.
    let art = mesh(plane_geo!(0.22, 0.16), &paint_mat);
    art.position().set(0, mount_y, 0.014);
    painting.add(&art);
    painting.rotation().z = (rnd() - 0.5) * 0.05; // slight tilt
    painting.rotation().y = face_yaw(dx, dz);
    painting.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - 0.015), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - 0.015));
    g.world.add(&painting);
}

// Potted seedling: 7% chance per level. Small pot w/ tiny green sprout.
pub fn place_seedlings(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.93 {
        return;
    } // 7% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let pot_mat = lambert!({ color: 0xa0603c });
    let dirt_mat = lambert!({ color: 0x2d1a0d });
    let stem_mat = lambert!({ color: 0x60a040 });
    let leaf_mat = lambert!({ color: 0x408040, side: DoubleSide });
    let seedling = group();
    // Small pot.
    let pot = mesh(pg("Seedlings__pot", cyl_geo!(0.028, 0.024, 0.030, 10)), &pot_mat);
    pot.position().y = 0.015;
    seedling.add(&pot);
    // Dirt top.
    let dirt = mesh(pg("Seedlings__dirt", circle_geo!(0.026, 10)), &dirt_mat);
    dirt.rotation().x = -PI / 2.0;
    dirt.position().y = 0.031;
    seedling.add(&dirt);
    // Tiny stem stub.
    let stem = mesh(pg("Seedlings__stem", cyl_geo!(0.0015, 0.0015, 0.024, 4)), &stem_mat);
    stem.position().y = 0.043;
    seedling.add(&stem);
    // 2 tiny cotyledons (seed leaves).
    for side in [-1.0, 1.0] {
        let leaf = mesh(circle_geo!(0.006, 5), &leaf_mat);
        leaf.rotation().x = -PI / 2.0;
        leaf.rotation().z = side * 0.6;
        leaf.position().set(side * 0.005, 0.054, 0);
        seedling.add(&leaf);
    }
    seedling.position().set(g.wx(x) + (rnd() - 0.5) * 2.2, 0, g.wz(z) + (rnd() - 0.5) * 2.2);
    seedling.rotation().y = rnd() * PI * 2.0;
    g.world.add(&seedling);
}

// Gardening trowel: 6% chance per level. Metal scoop blade + handle.
pub fn place_trowels(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let blade_mat = lambert!({ color: 0x606060 });
    let handle_colors: [u32; 4] = [0x8b5a2b, 0x404040, 0xc02020, 0x2060c0];
    let handle_mat = lambert!({ color: pick(&handle_colors) });
    let trowel = group();
    // Scoop blade (curved via 2 rotated planes + a cylinder half).
    // Simpler: a slightly tapered half cylinder.
    let blade = mesh(pg("Trowels__blade", cyl_geo!(0.020, 0.014, 0.080, 6, 1, false, 0, PI)), &blade_mat);
    // Scoop lying hollow-up: the half-cylinder's round back (+X) down, its
    // width across (Z), the tip end (+Y) toward -X; the belly rests on the
    // floor and the rim is at y 0.020.
    blade.rotation().x = PI;
    blade.rotation().z = PI / 2.0;
    blade.position().set(-0.030, 0.020, 0);
    trowel.add(&blade);
    // Blade tip cone (pointed end): flattened (0.35) to a point continuing
    // the scoop's floor, lying on the floor.
    let tip = mesh(pg("Trowels__tip", cone_geo!(0.015, 0.020, 6)), &blade_mat);
    tip.rotation().z = -PI / 2.0;
    tip.scale().x = 0.35;
    tip.position().set(-0.080, 0.0055, 0);
    trowel.add(&tip);
    // Shaft (thin cylinder from blade to handle).
    let shaft = mesh(pg("Trowels__shaft", cyl_geo!(0.005, 0.005, 0.040, 6)), &blade_mat);
    shaft.rotation().z = PI / 2.0;
    shaft.position().set(0.008, 0.010, 0);
    trowel.add(&shaft);
    // Wooden/plastic handle grip.
    let handle = mesh(pg("Trowels__handle", cyl_geo!(0.012, 0.014, 0.060, 8)), &handle_mat);
    handle.rotation().z = PI / 2.0;
    handle.position().set(0.056, 0.012, 0);
    trowel.add(&handle);
    // Small end cap.
    let cap = mesh(pg("Trowels__cap", sphere_geo!(0.014, 6, 5)), &handle_mat);
    cap.position().set(0.090, 0.012, 0);
    trowel.add(&cap);
    trowel.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    trowel.rotation().y = rnd() * PI * 2.0;
    g.world.add(&trowel);
}

// Seed packet: 8% chance per level. Colored paper envelope w/ printed
// flower image.
game::lazy_static_tex!(
    seed_packet_texture,
    canvas_tex(80, 120, |ctx, w, h| {
        // Random pastel background.
        let bgs = ["#f0e0a0", "#e0b0a0", "#a0c0e0", "#c0e0a0"];
        ctx.set_fill_style(bgs[(rnd() * bgs.len() as f64).floor() as usize]);
        ctx.fill_rect(0, 0, w, h);
        // Border stripe at top.
        ctx.set_fill_style("#404020");
        ctx.fill_rect(0, 0, w, 10);
        // "Seed name" title bar (dark bar).
        ctx.set_fill_style("#202010");
        ctx.fill_rect(6, 16, w - 12.0, 6);
        // Center flower illustration (5 petal circle + center).
        let (cx, cy) = (w / 2.0, h * 0.55);
        let flower_colors = ["#c02020", "#f0c020", "#d060c0", "#c04060", "#e08040"];
        ctx.set_fill_style(flower_colors[(rnd() * flower_colors.len() as f64).floor() as usize]);
        for p in 0..5 {
            let a = (p as f64 / 5.0) * PI * 2.0;
            ctx.begin_path();
            ctx.arc(cx + a.cos() * 12.0, cy + a.sin() * 12.0, 8, 0, PI * 2.0);
            ctx.fill();
        }
        // Center yellow dot.
        ctx.set_fill_style("#f0e020");
        ctx.begin_path();
        ctx.arc(cx, cy, 5, 0, PI * 2.0);
        ctx.fill();
        // Bottom info bar.
        ctx.set_fill_style("#404020");
        for i in 0..3 {
            ctx.fill_rect(6, h - 24.0 + i as f64 * 6.0, w - 12.0 - rnd() * 20.0, 2);
        }
    })
);

pub fn place_seed_packets(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let packet_mat = lambert!({ map: seed_packet_texture(), side: DoubleSide });
    let packet = mesh(plane_geo!(0.055, 0.080), &packet_mat);
    packet.rotation().x = -PI / 2.0;
    packet.position().set(g.wx(x) + (rnd() - 0.5) * 2.5, 0.003, g.wz(z) + (rnd() - 0.5) * 2.5);
    packet.rotation().z = rnd() * PI * 2.0;
    g.world.add(&packet);
}

// Mini watering can: 5% chance per level. Body + spout + handle.
pub fn place_watering_cans(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let can_colors: [u32; 5] = [0x408060, 0x606060, 0x2060c0, 0xc02020, 0xc0a040];
    let can_mat = lambert!({ color: pick(&can_colors) });
    let rose_mat = lambert!({ color: 0x505050 });
    let can = group();
    // Main body (tapered cylinder).
    let body = mesh(pg("WateringCans__body", cyl_geo!(0.045, 0.050, 0.080, 12)), &can_mat);
    body.position().y = 0.040;
    can.add(&body);
    // Spout (angled cylinder extending out).
    // The spout's narrow end (+Y) points out and up along spout_dir; its wide,
    // flanged end is set into the body low on the -X side.
    let spout = mesh(pg("WateringCans__spout", cyl_geo!(0.008, 0.010, 0.095, 6)), &can_mat);
    let spout_dir = v3(-(PI / 3.0).sin(), (PI / 3.0).cos(), 0);
    spout.rotation().z = PI / 3.0;
    spout.position().set(-0.056, 0.070, 0);
    can.add(&spout);
    let mut spout_end = spout.pos();
    spout_end.add_scaled_vector(&spout_dir, 0.0475);
    // Rose (dispenser end) on spout — small perforated disc: conical back
    // on the spout's lip, face (+Y) out along the spout.
    let rose = mesh(pg("WateringCans__rose", cyl_geo!(0.012, 0.014, 0.006, 8)), &rose_mat);
    rose.rotation().z = PI / 3.0;
    {
        let mut p = rose.position();
        p.copy(&spout_end);
        p.add_scaled_vector(&spout_dir, 0.0025);
    }
    can.add(&rose);
    let mut rose_face = rose.pos();
    rose_face.add_scaled_vector(&spout_dir, 0.0035);
    let rose_up = v3((PI / 3.0).cos(), (PI / 3.0).sin(), 0);
    // 5 tiny hole dots on rose.
    for hh in 0..5 {
        let hole = mesh(circle_geo!(0.0012, 4), lambert!({ color: 0x101010 }));
        let a = (hh as f64 / 5.0) * PI * 2.0;
        hole.quaternion().set_from_unit_vectors(&v3(0, 0, 1), &spout_dir);
        {
            let mut p = hole.position();
            p.copy(&rose_face);
            p.add_scaled_vector(&rose_up, a.cos() * 0.005);
            p.add(&v3(0, 0, a.sin() * 0.005));
        }
        can.add(&hole);
    }
    // Handle (curved arc over top): an upright strap in the spout's plane
    // (XY), both ends set into the can's top.
    let handle = mesh(pg("WateringCans__handle", torus_geo!(0.038, 0.005, 4, 8, PI)), &can_mat);
    handle.position().set(0.003, 0.081, 0);
    can.add(&handle);
    can.position().set(g.wx(x) + (rnd() - 0.5) * 2.2, 0, g.wz(z) + (rnd() - 0.5) * 2.2);
    can.rotation().y = rnd() * PI * 2.0;
    g.world.add(&can);
}

// Pruning shears: 6% chance per level. 2 curved blades + 2 handles + pivot.
pub fn place_pruning_shears(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let blade_mat = lambert!({ color: 0x606060 });
    let handle_colors: [u32; 4] = [0xc02020, 0xd0a040, 0x408040, 0x202020];
    let handle_mat = lambert!({ color: pick(&handle_colors) });
    let shears = group();
    // Upper blade (curved via short cylinder + cone tip).
    // Lying flat, jaws and handles sprung open in the floor plane about a
    // vertical pivot bolt at (0.005, 0): the blade (-Z side, edge +Z) and
    // the hook (+Z side, edge -Z) open ±0.15 rad from their roots at the bolt,
    // their tips on their ends; the handles open ±0.2 rad from the bolt.
    let upper_blade = mesh(pg("PruningShears__blade", box_geo!(0.045, 0.005, 0.010)), &blade_mat);
    upper_blade.rotation().y = -0.15;
    upper_blade.position().set(0.005 - 0.020 * 0.15f64.cos(), 0.0025, -0.004 - 0.020 * 0.15f64.sin());
    shears.add(&upper_blade);
    // Upper blade tip cone (point outward, flattened to the blade's thickness).
    let upper_tip = mesh(pg("PruningShears__tip", cone_geo!(0.006, 0.014, 4)), &blade_mat);
    upper_tip.rotation().set(0, -0.15, -PI / 2.0);
    upper_tip.scale().x = 0.45;
    upper_tip.position().set(0.005 - 0.0485 * 0.15f64.cos(), 0.0025, -0.004 - 0.0485 * 0.15f64.sin());
    shears.add(&upper_tip);
    // Lower blade (matches, angled other way).
    let lower_blade = mesh(pg("PruningShears__hook", box_geo!(0.045, 0.005, 0.010)), &blade_mat);
    lower_blade.rotation().y = 0.15;
    lower_blade.position().set(0.005 - 0.020 * 0.15f64.cos(), 0.0025, 0.004 + 0.020 * 0.15f64.sin());
    shears.add(&lower_blade);
    let lower_tip = mesh(pg("PruningShears__tip", cone_geo!(0.006, 0.014, 4)), &blade_mat);
    lower_tip.rotation().set(0, 0.15, -PI / 2.0);
    lower_tip.scale().x = 0.45;
    lower_tip.position().set(0.005 - 0.0485 * 0.15f64.cos(), 0.0025, 0.004 + 0.0485 * 0.15f64.sin());
    shears.add(&lower_tip);
    // Pivot (rivet in middle): vertical bolt through blades and handles.
    let pivot = mesh(pg("PruningShears__pivot", cyl_geo!(0.004, 0.004, 0.020, 6)), &blade_mat);
    pivot.scale().y = 0.75;
    pivot.position().set(0.005, 0.0075, 0);
    shears.add(&pivot);
    // 2 handles extending back.
    let handle_upper = mesh(pg("PruningShears__handle", box_geo!(0.055, 0.014, 0.012)), &handle_mat);
    handle_upper.rotation().y = 0.2;
    handle_upper.position().set(0.005 + 0.0245 * 0.2f64.cos(), 0.007, -0.006 - 0.0245 * 0.2f64.sin());
    shears.add(&handle_upper);
    let handle_lower = mesh(pg("PruningShears__handle", box_geo!(0.055, 0.014, 0.012)), &handle_mat);
    handle_lower.rotation().y = -0.2;
    handle_lower.position().set(0.005 + 0.0245 * 0.2f64.cos(), 0.007, 0.006 + 0.0245 * 0.2f64.sin());
    shears.add(&handle_lower);
    shears.position().set(g.wx(x) + (rnd() - 0.5) * 2.2, 0, g.wz(z) + (rnd() - 0.5) * 2.2);
    shears.rotation().y = rnd() * PI * 2.0;
    g.world.add(&shears);
}

// Garden hoe: 4% chance per level. Long wooden handle + metal flat blade.
pub fn place_garden_hoes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let wood_mat = lambert!({ color: 0xa07040 });
    let head_mat = lambert!({ color: 0x505050 });
    let hoe = group();
    // Long handle (laid flat).
    let handle = mesh(pg("GardenHoes__handle", cyl_geo!(0.012, 0.012, 0.55, 8)), &wood_mat);
    handle.rotation().z = PI / 2.0;
    handle.position().y = 0.012;
    hoe.add(&handle);
    // Metal collar at head end.
    let collar = mesh(pg("GardenHoes__collar", cyl_geo!(0.014, 0.014, 0.020, 8)), &head_mat);
    collar.rotation().z = PI / 2.0;
    collar.position().set(-0.265, 0.012, 0);
    hoe.add(&collar);
    // Flat blade head (perpendicular to handle).
    let head = mesh(pg("GardenHoes__head", box_geo!(0.016, 0.006, 0.070)), &head_mat);
    // Socket side 1 mm into the collar's end (-0.275), lying on the floor.
    head.position().set(-0.282, 0.003, 0);
    hoe.add(&head);
    // Small end cap on handle other end.
    let cap = mesh(pg("GardenHoes__cap", sphere_geo!(0.014, 6, 5)), &wood_mat);
    cap.position().set(0.278, 0.012, 0);
    hoe.add(&cap);
    hoe.position().set(g.wx(x) + (rnd() - 0.5) * 1.8, 0, g.wz(z) + (rnd() - 0.5) * 1.8);
    hoe.rotation().y = rnd() * PI * 2.0;
    g.world.add(&hoe);
}

// Garden hose coil: 5% chance per level. Concentric tori forming coiled hose.
pub fn place_garden_hoses(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let hose_colors: [u32; 4] = [0x408040, 0x2060c0, 0x606060, 0x804040];
    let hose_mat = lambert!({ color: pick(&hose_colors) });
    let nozzle_mat = lambert!({ color: 0xd0a040 });
    let hose = group();
    // 4 concentric torus loops for coiled read.
    // Lying flat on the floor, each turn pressed against the next (1.4 cm
    // pitch for a 1.6 cm hose), so they read as one flat coil.
    for i in 0..4 {
        let r = 0.090 - i as f64 * 0.014;
        let lp = mesh(pg("GardenHoses__loop", torus_geo!(r, 0.008, 5, 16)), &hose_mat);
        lp.rotation().x = PI / 2.0;
        lp.rotation().z = i as f64 * 1.3; // turn each loop's stripe/bump round
        lp.position().y = 0.008;
        hose.add(&lp);
    }
    // Brass spray nozzle at end.
    // The hose end leaves the outer turn tangentially at (0.09, 0): the
    // coupling (+Y) sits on the hose there, the front points along +Z.
    let nozzle = mesh(pg("GardenHoses__nozzle", cyl_geo!(0.010, 0.012, 0.060, 8)), &nozzle_mat);
    nozzle.rotation().x = -PI / 2.0;
    nozzle.position().set(0.094, 0.011, 0.028);
    hose.add(&nozzle);
    // Nozzle tip cone.
    let tip = mesh(pg("GardenHoses__tip", cone_geo!(0.008, 0.014, 6)), &nozzle_mat);
    tip.rotation().x = PI / 2.0;
    tip.position().set(0.094, 0.011, 0.064);
    hose.add(&tip);
    hose.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    hose.rotation().y = rnd() * PI * 2.0;
    g.world.add(&hose);
}

// Garden shovel: 3% chance per level. Long handle + D-grip + spade blade.
pub fn place_garden_shovels(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    } // 3% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let wood_mat = lambert!({ color: 0x8b5a2b });
    let blade_mat = lambert!({ color: 0x505050 });
    let shovel = group();
    // Long handle.
    let handle = mesh(pg("GardenShovels__handle", cyl_geo!(0.014, 0.014, 0.55, 8)), &wood_mat);
    handle.rotation().z = PI / 2.0;
    handle.position().y = 0.014;
    shovel.add(&handle);
    // Collar at blade end.
    let collar = mesh(pg("GardenHoes__collar", cyl_geo!(0.016, 0.016, 0.020, 8)), &blade_mat);
    collar.rotation().z = PI / 2.0;
    collar.position().set(-0.265, 0.014, 0);
    shovel.add(&collar);
    // Spade blade (rectangular w/ tapered tip via cone).
    let blade = mesh(pg("GardenShovels__blade", box_geo!(0.070, 0.008, 0.090)), &blade_mat);
    // Socket strap 2 mm into the collar (end -0.275); the blade lies on the floor.
    blade.position().set(-0.308, 0.004, 0);
    shovel.add(&blade);
    // Blade tip taper.
    // A flat triangular point in the blade's plane: apex out (-X), the
    // pyramid squashed (0.12) to the blade's thickness instead of standing
    // 9 cm tall with its lowest corner 3 cm under the floor.
    let tip = mesh(cone_geo!(0.045, 0.030, 3), &blade_mat);
    tip.rotation().z = PI / 2.0;
    tip.rotation().x = PI / 2.0;
    tip.scale().z = 0.12;
    tip.position().set(-0.308 - 0.035 - 0.014, 0.0054, 0);
    shovel.add(&tip);
    // D-grip at handle end (torus + 2 side bars).
    // The D lies in the blade's plane (XZ): chord across the handle's end
    // (narrowed 0.6 so its ends meet the shaft), crown bulging out along +X.
    let d_grip = mesh(pg("GardenShovels__grip", torus_geo!(0.024, 0.006, 5, 12, PI)), &wood_mat);
    set_rot_order(&d_grip, -PI / 2.0, -PI / 2.0, 0.0, EulerOrder::YXZ);
    d_grip.scale().x = 0.6;
    d_grip.position().set(0.273, 0.014, 0);
    shovel.add(&d_grip);
    shovel.position().set(g.wx(x) + (rnd() - 0.5) * 1.6, 0, g.wz(z) + (rnd() - 0.5) * 1.6);
    shovel.rotation().y = rnd() * PI * 2.0;
    g.world.add(&shovel);
}
