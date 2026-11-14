//! `place_wall_chalkboards`
use game::prelude::*;
use placers_kit::mirror_frame::moulding_grain_tex;
use placers_kit::noise::hash as cb_hash01;
use placers_kit::noise::sstep as cb_sstep;
use placers_kit::noise::vn as cb_vnoise;
use placers_kit::place::mount_on_wall;
use placers_kit::place::vary as cm_vary;
use placers_kit::place::wall_candidates;

// Wall chalkboard: 4% chance per level. A 30 x 40 cm wall board in a
// pine frame, a note left on it in chalk (batch r13a).
pub(crate) fn place_wall_chalkboards(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = candidates[(rnd() * candidates.len() as f64).floor() as usize];
    // The common 30 x 40 cm wall chalkboard (Blender batch `r13a`): a
    // painted board in a varnished pine frame 22 mm wide, flat on the wall;
    // on it a note in someone's chalk capitals over the ghost of the one
    // wiped before it, the eraser's swipes, dust (docs/design/PROPS.md)
    use game::assets::props::prop_part_as_modelled;
    let board = group();
    mount_on_wall(g, &board, &c, 0.010, 1.60, 0.6);
    let p = board.pos();
    let vary = cm_vary(p.x, p.z, 0.5);
    // the parts are modelled with the wall's face at z = 0: 10 mm behind the spot
    let body = group();
    body.position().z = -0.010;
    board.add(&body);
    if let Some(geo) = prop_part_as_modelled("Real__chalkboardFrame") {
        let tex = moulding_grain_tex(CB_SIZE[0], CB_SIZE[1], CB_MOULD, [0x9c, 0x76, 0x4c], vary, 1000.0);
        body.add(&mesh(geo, lambert!({ map: &tex })));
    }
    let (fw, fh) = (CB_SIZE[0] - 2.0 * CB_MOULD, CB_SIZE[1] - 2.0 * CB_MOULD);
    let face = mesh(plane_geo!(fw + 0.002, fh + 0.002), lambert!({ map: &chalkboard_face_tex(fw, fh, vary) }));
    face.position().z = CB_FACE;
    body.add(&face);
    g.world.add(&board);
}

/// The chalkboard: its size, its frame's width, its painted face's distance
/// from the wall (`Real__chalkboardFrame`, industrial/six_things_hung_on_a_wall.py)
const CB_SIZE: [f64; 2] = [0.400, 0.300];

const CB_MOULD: f64 = 0.022;

const CB_FACE: f64 = 0.0071;

/// What was left on it, in capitals: the week's business, nothing more
const CB_NOTES: [&[&str]; 6] = [
    &["MON 3/11", "STAFF MTG 9:30", "ROOM 2B"],
    &["ORDER:", "TONER x2", "A4 PAPER x5", "STAPLES"],
    &["J. HALE OUT", "TILL THE 14TH", "CALLS TO EXT 214"],
    &["PHOTOCOPIER", "OUT OF ORDER", "TECH WEDS"],
    &["MILK", "COFFEE", "SUGAR", "CUPS x20"],
    &["DELIVERY THURS", "CHAIRS x12", "TABLES x4"],
];

/// An arc of an ellipse (centre, radii, from a0 to a1 radians, y up)
fn cb_arc(cx: f64, cy: f64, rx: f64, ry: f64, a0: f64, a1: f64) -> Vec<(f64, f64)> {
    let n = ((a1 - a0).abs() / 0.35).ceil().max(2.0) as usize;
    (0..=n).map(|k| a0 + (a1 - a0) * k as f64 / n as f64).map(|a| (cx + rx * a.cos(), cy + ry * a.sin())).collect()
}

/// A capital as a hand writes it in chalk: its width, and its strokes in the
/// order and direction they are made (x right, y up, the cap height 1)
fn cb_glyph(c: char) -> (f64, Vec<Vec<(f64, f64)>>) {
    let v = |p: &[(f64, f64)]| p.to_vec();
    match c {
        'A' => (0.66, vec![v(&[(0.0, 0.0), (0.33, 1.0), (0.66, 0.0)]), v(&[(0.14, 0.38), (0.52, 0.4)])]),
        'B' => (
            0.58,
            vec![
                v(&[(0.0, 0.0), (0.0, 1.0)]),
                v(&[(0.0, 1.0), (0.34, 1.0), (0.5, 0.93), (0.53, 0.78), (0.45, 0.6), (0.28, 0.54), (0.02, 0.54)]),
                v(&[(0.28, 0.54), (0.5, 0.47), (0.6, 0.3), (0.56, 0.12), (0.4, 0.02), (0.0, 0.0)]),
            ],
        ),
        'C' => (0.62, vec![cb_arc(0.36, 0.5, 0.36, 0.5, 0.75, 5.55)]),
        'D' => (
            0.62,
            vec![
                v(&[(0.0, 0.0), (0.0, 1.0)]),
                v(&[(0.0, 1.0), (0.24, 1.0), (0.48, 0.9), (0.62, 0.62), (0.6, 0.3), (0.45, 0.08), (0.2, 0.0), (0.0, 0.0)]),
            ],
        ),
        'E' => (0.55, vec![v(&[(0.55, 1.0), (0.0, 1.0), (0.0, 0.0), (0.56, 0.0)]), v(&[(0.0, 0.52), (0.4, 0.53)])]),
        'F' => (0.52, vec![v(&[(0.55, 1.0), (0.0, 1.0), (0.0, 0.0)]), v(&[(0.0, 0.52), (0.4, 0.53)])]),
        'G' => (0.68, vec![cb_arc(0.36, 0.5, 0.36, 0.5, 0.75, 5.9), v(&[(0.4, 0.42), (0.7, 0.42), (0.68, 0.06)])]),
        'H' => (0.6, vec![v(&[(0.0, 1.0), (0.0, 0.0)]), v(&[(0.6, 1.0), (0.6, 0.0)]), v(&[(0.0, 0.52), (0.6, 0.53)])]),
        'I' => (0.14, vec![v(&[(0.07, 1.0), (0.07, 0.0)])]),
        'J' => (0.5, vec![v(&[(0.5, 1.0), (0.5, 0.26), (0.42, 0.06), (0.25, 0.0), (0.08, 0.06), (0.0, 0.22)])]),
        'K' => (0.58, vec![v(&[(0.0, 1.0), (0.0, 0.0)]), v(&[(0.56, 1.0), (0.02, 0.42)]), v(&[(0.2, 0.6), (0.6, 0.0)])]),
        'L' => (0.5, vec![v(&[(0.0, 1.0), (0.0, 0.0), (0.5, 0.0)])]),
        'M' => (0.76, vec![v(&[(0.0, 0.0), (0.04, 1.0), (0.38, 0.28), (0.71, 1.0), (0.76, 0.0)])]),
        'N' => (0.62, vec![v(&[(0.0, 0.0), (0.0, 1.0), (0.62, 0.0), (0.62, 1.0)])]),
        'O' => (0.72, vec![cb_arc(0.36, 0.5, 0.36, 0.5, 1.75, 1.75 + 6.45)]),
        'P' => {
            (0.56, vec![v(&[(0.0, 0.0), (0.0, 1.0)]), v(&[(0.0, 1.0), (0.36, 1.0), (0.54, 0.88), (0.55, 0.66), (0.4, 0.52), (0.02, 0.5)])])
        }
        'Q' => (0.74, vec![cb_arc(0.36, 0.5, 0.36, 0.5, 1.75, 1.75 + 6.4), v(&[(0.42, 0.24), (0.74, -0.06)])]),
        'R' => (
            0.6,
            vec![
                v(&[(0.0, 0.0), (0.0, 1.0)]),
                v(&[(0.0, 1.0), (0.36, 1.0), (0.54, 0.88), (0.55, 0.66), (0.4, 0.52), (0.02, 0.5)]),
                v(&[(0.22, 0.5), (0.6, 0.0)]),
            ],
        ),
        'S' => (
            0.58,
            vec![v(&[
                (0.55, 0.86),
                (0.42, 0.99),
                (0.2, 1.0),
                (0.04, 0.88),
                (0.06, 0.66),
                (0.3, 0.53),
                (0.52, 0.4),
                (0.6, 0.2),
                (0.5, 0.04),
                (0.28, 0.0),
                (0.08, 0.06),
                (0.0, 0.17),
            ])],
        ),
        'T' => (0.64, vec![v(&[(0.0, 1.0), (0.64, 1.0)]), v(&[(0.32, 1.0), (0.32, 0.0)])]),
        'U' => (0.6, vec![v(&[(0.0, 1.0), (0.0, 0.3), (0.08, 0.08), (0.29, 0.0), (0.5, 0.08), (0.59, 0.3), (0.6, 1.0)])]),
        'V' => (0.62, vec![v(&[(0.0, 1.0), (0.31, 0.0), (0.62, 1.0)])]),
        'W' => (0.86, vec![v(&[(0.0, 1.0), (0.2, 0.0), (0.43, 0.72), (0.65, 0.0), (0.86, 1.0)])]),
        'X' => (0.58, vec![v(&[(0.0, 1.0), (0.58, 0.0)]), v(&[(0.58, 1.0), (0.0, 0.0)])]),
        'Y' => (0.6, vec![v(&[(0.0, 1.0), (0.3, 0.5)]), v(&[(0.6, 1.0), (0.3, 0.5), (0.3, 0.0)])]),
        'Z' => (0.58, vec![v(&[(0.0, 1.0), (0.58, 1.0), (0.0, 0.0), (0.6, 0.0)])]),
        '0' => (0.58, vec![cb_arc(0.29, 0.5, 0.29, 0.5, 1.7, 1.7 + 6.4)]),
        '1' => (0.32, vec![v(&[(0.02, 0.8), (0.24, 1.0), (0.24, 0.0)])]),
        '2' => {
            (0.58, vec![v(&[(0.03, 0.78), (0.12, 0.95), (0.32, 1.0), (0.5, 0.92), (0.55, 0.72), (0.42, 0.46), (0.0, 0.0), (0.58, 0.0)])])
        }
        '3' => (
            0.56,
            vec![v(&[
                (0.04, 0.9),
                (0.24, 1.0),
                (0.46, 0.95),
                (0.52, 0.77),
                (0.38, 0.58),
                (0.2, 0.54),
                (0.38, 0.54),
                (0.55, 0.42),
                (0.58, 0.2),
                (0.42, 0.03),
                (0.2, 0.0),
                (0.02, 0.1),
            ])],
        ),
        '4' => (0.62, vec![v(&[(0.46, 0.0), (0.46, 1.0), (0.0, 0.3), (0.62, 0.3)])]),
        '5' => (
            0.56,
            vec![v(&[
                (0.52, 1.0),
                (0.1, 1.0),
                (0.06, 0.56),
                (0.3, 0.62),
                (0.5, 0.52),
                (0.58, 0.3),
                (0.48, 0.08),
                (0.26, 0.0),
                (0.04, 0.08),
            ])],
        ),
        '6' => (
            0.56,
            vec![v(&[
                (0.5, 0.95),
                (0.3, 1.0),
                (0.1, 0.85),
                (0.02, 0.5),
                (0.04, 0.2),
                (0.2, 0.01),
                (0.42, 0.02),
                (0.56, 0.2),
                (0.54, 0.4),
                (0.38, 0.55),
                (0.18, 0.52),
                (0.04, 0.38),
            ])],
        ),
        '7' => (0.56, vec![v(&[(0.0, 1.0), (0.58, 1.0), (0.2, 0.0)])]),
        '8' => (
            0.58,
            vec![v(&[
                (0.3, 0.54),
                (0.08, 0.66),
                (0.07, 0.88),
                (0.3, 1.0),
                (0.52, 0.88),
                (0.5, 0.66),
                (0.3, 0.54),
                (0.06, 0.4),
                (0.04, 0.14),
                (0.3, 0.0),
                (0.56, 0.14),
                (0.54, 0.4),
                (0.3, 0.54),
            ])],
        ),
        '9' => (
            0.56,
            vec![v(&[
                (0.54, 0.62),
                (0.4, 0.48),
                (0.18, 0.48),
                (0.04, 0.64),
                (0.06, 0.88),
                (0.26, 1.0),
                (0.48, 0.93),
                (0.56, 0.7),
                (0.54, 0.3),
                (0.42, 0.05),
                (0.2, 0.0),
                (0.06, 0.08),
            ])],
        ),
        '.' => (0.12, vec![v(&[(0.05, 0.0), (0.07, 0.03)])]),
        ':' => (0.14, vec![v(&[(0.06, 0.62), (0.08, 0.65)]), v(&[(0.06, 0.06), (0.08, 0.09)])]),
        '-' => (0.36, vec![v(&[(0.0, 0.45), (0.36, 0.47)])]),
        '/' => (0.42, vec![v(&[(0.0, -0.04), (0.42, 1.04)])]),
        'x' => (0.4, vec![v(&[(0.0, 0.52), (0.38, 0.0)]), v(&[(0.38, 0.52), (0.0, 0.0)])]),
        _ => (0.42, vec![]),
    }
}

/// The note's strokes laid out on a board `w` x `h` px: lines of capitals
/// `cap` px high from (x0, y0), as one hand writes them (each letter a
/// little bigger or smaller, leaning, off the line; the lines drifting)
fn cb_layout(lines: &[&str], w: f64, cap: f64, x0: f64, y0: f64, pitch: f64, r: &mut SplitMix64) -> Vec<Vec<(f64, f64)>> {
    let mut out = vec![];
    let lean = r.range(0.04, 0.14);
    for (li, line) in lines.iter().enumerate() {
        let mut x = x0 + r.range(-0.15, 0.25) * cap;
        let base = y0 + pitch * li as f64 + r.range(-0.08, 0.08) * cap;
        let drift = r.range(-0.035, 0.03);
        let size = cap * r.range(0.94, 1.06);
        for ch in line.chars() {
            if ch == ' ' {
                x += size * r.range(0.45, 0.62);
                continue;
            }
            let (gw, strokes) = cb_glyph(ch);
            let k = size * r.range(0.93, 1.07);
            let (dy, turn) = (r.range(-0.05, 0.05) * cap + (x - x0) * drift, r.range(-0.05, 0.05));
            let (sn, cs) = turn.sin_cos();
            for s in strokes {
                let mut p: Vec<(f64, f64)> = s
                    .iter()
                    .map(|&(gx, gy)| {
                        let (lx, ly) = ((gx + gy * lean) * k, gy * k);
                        let (rx, ry) = (lx * cs - ly * sn, lx * sn + ly * cs);
                        (x + rx, base + dy - ry)
                    })
                    .collect();
                // the hand overshoots where a stroke starts, now and then
                if p.len() > 1 && r.next() < 0.3 {
                    let (a, b) = (p[0], p[1]);
                    let e = r.range(0.04, 0.1);
                    p.insert(0, (a.0 - (b.0 - a.0) * e, a.1 - (b.1 - a.1) * e));
                }
                out.push(p);
            }
            x += (gw + r.range(0.16, 0.26)) * k;
            if x > w - cap * 0.6 {
                break;
            }
        }
    }
    out
}

/// How much chalk each pixel of a `w` x `h` board holds (0..1) from
/// `lines` (px; each smoothed, pressed heavier where it lands, skipping where
/// the stick rode over the board's grain), a stick `width` px wide. No draws
fn cb_chalk(lines: &[Vec<(f64, f64)>], n: usize, m: usize, width: f64, seed: u32) -> Vec<f32> {
    let mut segs: Vec<((f64, f64), (f64, f64), f64, f64, u32)> = vec![];
    for (li, l) in lines.iter().enumerate() {
        // smoothed (Chaikin, twice) between its sharp turns, which a hand
        // makes as turns (an A's apex, an E's corners)
        let mut p: Vec<(f64, f64)> = vec![];
        let mut run: Vec<(f64, f64)> = vec![];
        for k in 0..l.len() {
            run.push(l[k]);
            let sharp = k > 0 && k + 1 < l.len() && {
                let (a, b, c) = (l[k - 1], l[k], l[k + 1]);
                let (u, v) = ((b.0 - a.0, b.1 - a.1), (c.0 - b.0, c.1 - b.1));
                let cos = (u.0 * v.0 + u.1 * v.1) / (u.0.hypot(u.1) * v.0.hypot(v.1)).max(1e-9);
                cos < 0.45
            };
            if sharp || k + 1 == l.len() {
                for _ in 0..2 {
                    if run.len() < 3 {
                        break;
                    }
                    let mut q = vec![run[0]];
                    for j in 0..run.len() - 1 {
                        let (a, b) = (run[j], run[j + 1]);
                        q.push((a.0 * 0.75 + b.0 * 0.25, a.1 * 0.75 + b.1 * 0.25));
                        q.push((a.0 * 0.25 + b.0 * 0.75, a.1 * 0.25 + b.1 * 0.75));
                    }
                    q.push(run[run.len() - 1]);
                    run = q;
                }
                if !p.is_empty() {
                    run.remove(0);
                }
                p.append(&mut run);
                run.push(l[k]);
            }
        }
        let mut d = vec![];
        for k in 0..p.len().saturating_sub(1) {
            let (a, b) = (p[k], p[k + 1]);
            let c = ((a.0 - b.0).hypot(a.1 - b.1) / 2.5).ceil().max(1.0) as usize;
            for i in 0..c {
                let t = i as f64 / c as f64;
                d.push((a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t));
            }
        }
        if let Some(&e) = p.last() {
            d.push(e);
        }
        if d.len() == 1 {
            d.push((d[0].0 + 0.6, d[0].1 + 0.4));
        }
        let nd = d.len() - 1;
        let ls = seed.wrapping_add(li as u32 * 97);
        let press = |k: usize| {
            let t = k as f64 / nd as f64;
            (0.78 + 0.22 * (t * 5.0 + li as f64).sin()) * (1.0 + 0.25 * (-t * 14.0).exp()) * ((1.0 - t) / 0.1).clamp(0.45, 1.0)
        };
        for k in 0..nd {
            segs.push((d[k], d[k + 1], press(k), press(k + 1), ls));
        }
    }
    let r = width / 2.0;
    const CELL: usize = 16;
    let (cw, chh) = (n.div_ceil(CELL), m.div_ceil(CELL));
    let mut cells: Vec<Vec<u32>> = vec![vec![]; cw * chh];
    let cell = |v: f64, hi: usize| ((v / CELL as f64).floor().max(0.0) as usize).min(hi - 1);
    for (k, &(a, b, ..)) in segs.iter().enumerate() {
        let (x0, x1) = (cell(a.0.min(b.0) - r - 1.0, cw), cell(a.0.max(b.0) + r + 1.0, cw));
        let (y0, y1) = (cell(a.1.min(b.1) - r - 1.0, chh), cell(a.1.max(b.1) + r + 1.0, chh));
        for cy in y0..=y1 {
            for cx in x0..=x1 {
                cells[cy * cw + cx].push(k as u32);
            }
        }
    }
    let mut bytes = vec![0u8; n * m * 4];
    par_rows(&mut bytes, n * 4, |j, out| {
        let y = j as f64 + 0.5;
        for i in 0..n {
            let x = i as f64 + 0.5;
            let mut keep = 0.0f64;
            for &si in &cells[(j / CELL) * cw + i / CELL] {
                let (a, b, pa, pb, ls) = segs[si as usize];
                if x < a.0.min(b.0) - r - 1.0 || x > a.0.max(b.0) + r + 1.0 || y < a.1.min(b.1) - r - 1.0 || y > a.1.max(b.1) + r + 1.0 {
                    continue;
                }
                let (ex, ey) = (b.0 - a.0, b.1 - a.1);
                let l2 = (ex * ex + ey * ey).max(1e-9);
                let t = (((x - a.0) * ex + (y - a.1) * ey) / l2).clamp(0.0, 1.0);
                let d = (x - a.0 - ex * t).hypot(y - a.1 - ey * t);
                let rr = r * (0.82 + 0.3 * cb_vnoise(ls ^ 0x5a, x * 0.3, y * 0.3));
                let cover = cb_sstep(rr + 0.7, rr - 0.7, d);
                if cover <= 0.0 {
                    continue;
                }
                let skip = cb_sstep(0.18, 0.45, cb_vnoise(ls ^ 0x77, (a.0 + ex * t) * 0.09, (a.1 + ey * t) * 0.09));
                let press = (pa + (pb - pa) * t) * (0.45 + 0.55 * skip) * (1.0 - 0.35 * (d / rr).powi(2));
                keep = 1.0 - (1.0 - keep) * (1.0 - cover * press);
            }
            // chalk holds on the board's grain, finer than on a wall
            let grain = cb_hash01(seed ^ 0x1d, i as i32, j as i32);
            let tooth = 0.75 * grain + 0.25 * cb_vnoise(seed ^ 0x2e, x * 0.2, y * 0.2);
            let a = cb_sstep(tooth - 0.18, tooth + 0.12, keep * 0.9) * keep.min(1.0).powf(0.5) * 0.92;
            out[i * 4] = (a.clamp(0.0, 1.0) * 255.0).round() as u8;
        }
    });
    (0..n * m).map(|k| f32::from(bytes[k * 4]) / 255.0).collect()
}

/// The board's face, `w` x `h` m: its black-green paint, worn glossy where
/// it was wiped most; the ghost of the last note under the eraser's
/// swipes; chalk dust gathered low; the note itself in chalk capitals.
/// Which note, and the hand, from `vary` (where the board hangs)
fn chalkboard_face_tex(w: f64, h: f64, vary: u32) -> Tex {
    let ppm = 1600.0;
    let (pw, ph) = ((w * ppm).round(), (h * ppm).round());
    canvas_tex_opt(pw, ph, false, move |ctx, cw, ch| {
        let seed = (rnd() * 4.0e9) as u32 ^ vary;
        let (n, m) = (cw as usize, ch as usize);
        let mut r = SplitMix64::new(seed, 0x0c4a_1b0a);
        let note = (vary as usize) % CB_NOTES.len();
        let ghost = (note + 1 + (vary as usize >> 8) % (CB_NOTES.len() - 1)) % CB_NOTES.len();
        // the note: capitals 2.4 cm high, lines 4.6 cm apart
        let cap = 0.024 * ppm * r.range(0.9, 1.1);
        let lines = cb_layout(CB_NOTES[note], cw, cap, 0.025 * ppm, 0.045 * ppm + cap * 0.2, 0.046 * ppm * r.range(0.95, 1.05), &mut r);
        let text = cb_chalk(&lines, n, m, 0.0034 * ppm, seed ^ 0x51);
        // the last note, wiped: thicker, fainter, smeared sideways by the eraser
        let ghost_lines = cb_layout(CB_NOTES[ghost], cw, cap * 1.1, 0.03 * ppm, 0.05 * ppm, 0.05 * ppm, &mut r);
        let ghost_c = cb_chalk(&ghost_lines, n, m, 0.0062 * ppm, seed ^ 0x52);
        // the eraser's swipes: long gentle arcs across the board, some overlapping
        let swipes: Vec<(f64, f64, f64, f64, f64)> = (0..7)
            .map(|_| (r.range(0.0, ch), r.range(0.03, 0.08) * ch, r.range(-0.08, 0.08), r.range(0.4, 1.0), r.range(0.025, 0.06) * ppm))
            .collect();
        let mut img = ctx.create_image_data(cw, ch);
        par_rows(&mut img.data, n * 4, |j, out| {
            let y = j as f64 + 0.5;
            for i in 0..n {
                let x = i as f64 + 0.5;
                let k = j * n + i;
                // the paint: a dark green-black, mottled, glossier where wiped most
                let mot = cb_vnoise(seed ^ 0x61, x * 0.02, y * 0.02) * 0.6 + cb_vnoise(seed ^ 0x62, x * 0.09, y * 0.09) * 0.4;
                let base = [34.0 + 9.0 * mot, 41.0 + 9.0 * mot, 37.0 + 8.0 * mot];
                // what the eraser left: dust along its swipes, smearing the ghost sideways
                let mut haze = 0.05 + 0.05 * cb_vnoise(seed ^ 0x63, x * 0.015, y * 0.03);
                let mut wiped = 0.0f64;
                for &(sy, amp, tilt, str_, wd) in &swipes {
                    let cy = sy + tilt * (x - cw / 2.0) + amp * ((x / cw) * 3.1).sin();
                    let d = (y - cy) / wd;
                    let band = (-d * d).exp() * str_;
                    let streak = 0.6 + 0.4 * cb_vnoise(seed ^ 0x64, x * 0.01, y * 0.6);
                    haze += band * 0.07 * streak;
                    wiped += band * 0.9;
                }
                // dust settled low on the board
                haze += 0.08 * cb_sstep(0.7, 1.0, y / ch) * (0.6 + 0.4 * cb_vnoise(seed ^ 0x65, x * 0.05, y * 0.2));
                let smear = ghost_c[j * n + ((i as f64 + 6.0 * cb_vnoise(seed ^ 0x66, x * 0.02, y * 0.05)) as usize).min(n - 1)] as f64;
                // most of it gone under the swipes: what is left, in patches
                let left = cb_sstep(0.3, 0.65, cb_vnoise(seed ^ 0x67, x * 0.025, y * 0.06)) * (1.0 - 0.75 * wiped.min(1.0));
                let g = 0.24 * smear * left;
                let t = text[k] as f64 * 0.95;
                let a = (1.0 - (1.0 - haze.min(0.4)) * (1.0 - g) * (1.0 - t)).clamp(0.0, 1.0);
                let chalk = [224.0, 224.0, 216.0];
                for c in 0..3 {
                    out[i * 4 + c] = (base[c] * (1.0 - a) + chalk[c] * a).round().clamp(0.0, 255.0) as u8;
                }
                out[i * 4 + 3] = 255;
            }
        });
        ctx.put_image_data(&img, 0, 0);
    })
}
