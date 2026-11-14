//! `place_cross_stitches`
use game::prelude::*;
use placers_kit::noise::hash as sp_hash;
use placers_kit::noise::vn as sp_noise;
use placers_kit::place::face_yaw;
use placers_kit::place::pick;
use placers_kit::place::vary as sp_vary;
use placers_kit::place::wall_tiles;

// Framed cross-stitch: 4% chance per level. An alphabet sampler on
// linen, framed (batch r13a).
pub(crate) fn place_cross_stitches(g: &mut Game, _dist: &[i32]) {
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
/// wall, the moulding's width (`Real__samplerFrame`, industrial/six_things_hung_on_a_wall.py)
const SP_LINEN: [f64; 3] = [0.170, 0.230, 0.0080];

const SP_MOULD: f64 = 0.025;

/// 14 stitches to the inch, and the stitched linen's cells (a little more
/// than is seen: its edges run under the frame's lip)
const SP_PITCH: f64 = 0.0254 / 14.0;

const SP_CELLS: [usize; 2] = [96, 129];

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
