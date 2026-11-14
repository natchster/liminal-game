//! `place_chalk_arrows`
use crate::shared::mark_tex;
use crate::shared::vary_by;
use game::prelude::*;
use placers_kit::noise::hash;
use placers_kit::noise::sstep;
use placers_kit::noise::vn;
use placers_kit::place::wall_candidates;

// Chalk arrow: 5% chance per level. A way-mark someone left on a wall at
// eye height (`docs/research/props-remodelling/floor_marks_and_dust.md`): an arrow drawn
// quickly in blackboard chalk, its head a closed triangle scribbled in (never
// her open arrow with the longer left barb, docs/design/SECRETS.md), a word over it
// in capitals in a hurried hand; the grain of the chalk caught on the wall's
// tooth, the strokes broken where it skipped, its dust under them; white
// chalk gone a little grey on the wall. Painted on a transparent plane the
// size of the old one, facing the room.
pub(crate) fn place_chalk_arrows(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = candidates[(rnd() * candidates.len() as f64).floor() as usize];
    // the old flip (the plane turned to the wall): now which way it points
    let left = rnd() < 0.5;
    let tex = chalk_arrow_tex(left, vary_by(c.x * 4 + c.dx, c.z * 4 + c.dz, 0x513));
    let mat = lambert!({
        map: &tex,
        color: CHALK_ARROW_WHITE,
        transparent: true,
        depthWrite: false,
        polygonOffset: true,
        polygonOffsetFactor: -2,
        polygonOffsetUnits: -2
    });
    let decal = mesh(plane_geo!(0.50, 0.25), &mat);
    let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.008);
    let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.008);
    let center_y = 1.55;
    if c.dx.abs() == 1 {
        decal.rotation().y = if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 };
        decal.position().set(wall_x, center_y, wall_z + (rnd() - 0.5) * 0.6);
    } else {
        decal.rotation().y = if c.dz > 0 { PI } else { 0.0 };
        decal.position().set(wall_x + (rnd() - 0.5) * 0.6, center_y, wall_z);
    }
    g.world.add(&decal);
}

/// Blackboard chalk on a wall, years on (sRGB)
const CHALK_ARROW_WHITE: u32 = 0xdcd8ce;

/// A stroke of chalk: its points (px) and how hard it was pressed at each
struct ChalkLine {
    pts: Vec<(f64, f64)>,
    press: Vec<f64>,
}

/// A chalk arrow and its word on a plane 0.50 × 0.25 m (1.3 mm a px),
/// pointing left or right; white (the material tints it)
fn chalk_arrow_tex(left: bool, vary: u32) -> Tex {
    mark_tex(384.0, 192.0, [1.0, 1.0, 1.0], vary, move |ctx, w, h| {
        let seed = (rnd() * 4.0e9) as u32;
        let px_m = 0.5 / w;
        let mut lines: Vec<ChalkLine> = vec![];
        let jig = |p: (f64, f64), k: f64| (p.0 + (rnd() - 0.5) * k, p.1 + (rnd() - 0.5) * k);
        // a pressure along a stroke (its points every 3 px or less): heavier in
        // its middle, lifting at its ends
        let pressed = |n: usize, base: f64| -> Vec<f64> {
            let wob = rnd() * 6.0;
            (0..n)
                .map(|k| {
                    let t = k as f64 / (n.max(2) - 1) as f64;
                    base * (0.75 + 0.2 * (t * 7.0 + wob).sin()) * (t / 0.08).min(1.0).max(0.35) * ((1.0 - t) / 0.12).min(1.0).max(0.25)
                })
                .collect()
        };
        // the arrow, drawn rightward then mirrored if it points left: a shaft
        // 26-34 cm, a closed head 5 cm long and 5 cm across, filled by scribbling
        let shaft_y = h * (0.66 + 0.06 * rnd());
        let x0 = w * (0.1 + 0.05 * rnd());
        let x1 = w * (0.8 + 0.04 * rnd());
        let sag = (rnd() - 0.5) * 8.0;
        let tilt = (rnd() - 0.5) * 10.0;
        let shaft: Vec<(f64, f64)> = (0..=8)
            .map(|k| k as f64 / 8.0)
            .map(|t| jig((x0 + (x1 - x0) * t, shaft_y + tilt * (t - 0.5) + sag * 4.0 * t * (1.0 - t)), 1.5))
            .collect();
        let tip_y = shaft_y + tilt * 0.5;
        let (hl, hh) = (0.05 / px_m * (0.9 + 0.2 * rnd()), 0.025 / px_m * (0.9 + 0.2 * rnd()));
        let tip = (x1 + hl * 0.55, tip_y);
        let (b0, b1) = ((x1 - hl * 0.45, tip_y - hh), (x1 - hl * 0.45, tip_y + hh));
        let shaft = chalk_dense(&chalk_smooth(&shaft), 3.0);
        lines.push(ChalkLine { press: pressed(shaft.len(), 1.0), pts: shaft });
        // the head's outline in one go, then scribbled in, back and forth
        let outline = chalk_dense(&[jig(b0, 2.0), jig(tip, 1.5), jig(b1, 2.0), jig(b0, 3.0)], 3.0);
        lines.push(ChalkLine { press: pressed(outline.len(), 1.0), pts: outline });
        let mut fill = vec![];
        let rows = 7 + (rnd() * 3.0) as usize;
        for k in 0..=rows {
            let f = k as f64 / rows as f64;
            let y = b0.1 + (b1.1 - b0.1) * f;
            // the head's width at this height
            let reach = (1.0 - (2.0 * f - 1.0).abs()) * (tip.0 - b0.0);
            let a = (b0.0 + 2.0 + rnd() * 2.0, y + (rnd() - 0.5) * 2.0);
            let b = (b0.0 + reach - 3.0 - rnd() * 3.0, y + (rnd() - 0.5) * 2.0);
            if k % 2 == 0 {
                fill.push(a);
                fill.push(b);
            } else {
                fill.push(b);
                fill.push(a);
            }
        }
        let fill = chalk_dense(&fill, 3.0);
        lines.push(ChalkLine { press: pressed(fill.len(), 0.8), pts: fill });
        if left {
            for l in &mut lines {
                for p in &mut l.pts {
                    p.0 = w - p.0;
                }
            }
        }
        // the word over it, in capitals 5 cm tall, slanting and drifting
        const WORDS: [&str; 5] = ["THIS WAY", "RUN", "HELP", "HIDE", "NO"];
        let word = WORDS[(rnd() * WORDS.len() as f64) as usize % WORDS.len()];
        let lh = 0.05 / px_m * (0.9 + 0.2 * rnd());
        let slant = 0.05 + 0.12 * rnd();
        let width: f64 = word.chars().map(|c| if c == ' ' { 0.55 } else { chalk_letter_w(c) + 0.28 } * lh).sum::<f64>() - 0.28 * lh;
        let mut cx = (w - width) / 2.0 + (rnd() - 0.5) * 20.0;
        let base = shaft_y - h * 0.18 - (rnd() * 8.0);
        let drift = (rnd() - 0.5) * 0.08;
        for ch in word.chars() {
            if ch == ' ' {
                cx += 0.55 * lh;
                continue;
            }
            let lw = chalk_letter_w(ch) * lh;
            let y0 = base - lh + drift * (cx - w / 2.0) + (rnd() - 0.5) * 3.0;
            let sz = 0.94 + 0.12 * rnd();
            for st in chalk_letter(ch) {
                let pts: Vec<(f64, f64)> =
                    st.iter().map(|&(u, v)| jig((cx + u * lw * sz + (1.0 - v) * lh * slant, y0 + v * lh * sz), 2.2)).collect();
                let pts = chalk_dense(&chalk_smooth(&pts), 3.0);
                lines.push(ChalkLine { press: pressed(pts.len(), 0.9), pts });
            }
            cx += lw + 0.28 * lh;
        }
        let alpha = chalk_raster(&lines, w, h, 6.5, seed);
        let mut img = ctx.create_image_data(w, h);
        for (k, a) in alpha.iter().enumerate() {
            img.data[k * 4..k * 4 + 4].copy_from_slice(&[255, 255, 255, (a.clamp(0.0, 1.0) * 255.0).round() as u8]);
        }
        ctx.put_image_data(&img, 0, 0);
    })
}

/// A letter's width for its height (capitals in a quick hand)
fn chalk_letter_w(c: char) -> f64 {
    match c {
        'I' => 0.12,
        'W' => 0.85,
        'L' | 'E' | 'P' | 'S' => 0.5,
        _ => 0.62,
    }
}

/// The strokes of a capital in its box (u across, v down, 0..1), as a quick
/// hand draws it
fn chalk_letter(c: char) -> Vec<Vec<(f64, f64)>> {
    let arc = |cx: f64, cy: f64, rx: f64, ry: f64, a0: f64, a1: f64| -> Vec<(f64, f64)> {
        (0..=12).map(|k| a0 + (a1 - a0) * k as f64 / 12.0).map(|a| (cx + rx * a.cos(), cy + ry * a.sin())).collect()
    };
    match c {
        'A' => vec![vec![(0.0, 1.0), (0.5, 0.0), (1.0, 1.0)], vec![(0.22, 0.62), (0.8, 0.6)]],
        'D' => {
            let mut s = vec![(0.08, 1.0), (0.08, 0.0)];
            s.extend(arc(0.2, 0.5, 0.75, 0.5, -1.57, 1.57));
            s.push((0.06, 1.0));
            vec![s]
        }
        'E' => vec![vec![(0.95, 0.0), (0.05, 0.0), (0.05, 1.0), (1.0, 1.0)], vec![(0.05, 0.5), (0.75, 0.48)]],
        'H' => vec![vec![(0.05, 0.0), (0.05, 1.0)], vec![(0.95, 0.0), (0.95, 1.0)], vec![(0.05, 0.52), (0.95, 0.5)]],
        'I' => vec![vec![(0.5, 0.0), (0.5, 1.0)]],
        'L' => vec![vec![(0.05, 0.0), (0.05, 1.0), (0.95, 1.0)]],
        'N' => vec![vec![(0.05, 1.0), (0.05, 0.0), (0.95, 1.0), (0.95, 0.0)]],
        'O' => vec![arc(0.5, 0.5, 0.5, 0.5, -1.8, 4.6)],
        'P' => {
            let mut s = vec![(0.05, 1.0), (0.05, 0.0)];
            s.extend(arc(0.3, 0.27, 0.65, 0.27, -1.57, 1.57));
            s.push((0.05, 0.54));
            vec![s]
        }
        'R' => {
            let mut s = vec![(0.05, 1.0), (0.05, 0.0)];
            s.extend(arc(0.3, 0.26, 0.65, 0.26, -1.57, 1.57));
            s.extend([(0.05, 0.52), (0.4, 0.52), (0.98, 1.0)]);
            vec![s]
        }
        'S' => {
            let mut s = arc(0.5, 0.25, 0.48, 0.25, -0.3, -4.4);
            s.extend(arc(0.5, 0.74, 0.5, 0.26, -1.6, 2.6));
            vec![s]
        }
        'T' => vec![vec![(0.0, 0.0), (1.0, 0.02)], vec![(0.5, 0.0), (0.5, 1.0)]],
        'U' => {
            let mut s = vec![(0.05, 0.0)];
            s.extend(arc(0.5, 0.55, 0.45, 0.45, 3.14, 0.0));
            s.push((0.95, 0.0));
            vec![s]
        }
        'W' => vec![vec![(0.0, 0.0), (0.24, 1.0), (0.5, 0.3), (0.76, 1.0), (1.0, 0.0)]],
        'Y' => vec![vec![(0.0, 0.0), (0.5, 0.5), (1.0, 0.0)], vec![(0.5, 0.5), (0.5, 1.0)]],
        _ => vec![],
    }
}

/// A hand's line through the points (Chaikin, twice)
fn chalk_smooth(p: &[(f64, f64)]) -> Vec<(f64, f64)> {
    let mut p = p.to_vec();
    for _ in 0..2 {
        if p.len() < 3 {
            break;
        }
        let mut q = vec![p[0]];
        for k in 0..p.len() - 1 {
            let (a, b) = (p[k], p[k + 1]);
            q.push((a.0 * 0.75 + b.0 * 0.25, a.1 * 0.75 + b.1 * 0.25));
            q.push((a.0 * 0.25 + b.0 * 0.75, a.1 * 0.25 + b.1 * 0.75));
        }
        q.push(p[p.len() - 1]);
        p = q;
    }
    p
}

/// The side of the cells a chalk raster sorts its strokes' segments into (px)
const CHALK_CELL: usize = 16;

/// For each `CHALK_CELL` square of an `n × m` px canvas, the segments (their
/// index in `segs`) that come within `reach` px of it
fn chalk_cells(segs: &[((f64, f64), (f64, f64), f64, f64, u32)], n: usize, m: usize, reach: f64) -> Vec<Vec<u32>> {
    let (cw, chh) = (n.div_ceil(CHALK_CELL), m.div_ceil(CHALK_CELL));
    let mut cells = vec![vec![]; cw * chh];
    let cell = |v: f64, hi: usize| ((v / CHALK_CELL as f64).floor().max(0.0) as usize).min(hi - 1);
    for (k, &(a, b, ..)) in segs.iter().enumerate() {
        let (x0, x1) = (cell(a.0.min(b.0) - reach, cw), cell(a.0.max(b.0) + reach, cw));
        let (y0, y1) = (cell(a.1.min(b.1) - reach, chh), cell(a.1.max(b.1) + reach, chh));
        for cy in y0..=y1 {
            for cx in x0..=x1 {
                cells[cy * cw + cx].push(k as u32);
            }
        }
    }
    cells
}

/// The polyline `p` with points added so no step is longer than `step`
fn chalk_dense(p: &[(f64, f64)], step: f64) -> Vec<(f64, f64)> {
    let mut out = vec![];
    for k in 0..p.len().saturating_sub(1) {
        let (a, b) = (p[k], p[k + 1]);
        let n = ((a.0 - b.0).hypot(a.1 - b.1) / step).ceil().max(1.0) as usize;
        for i in 0..n {
            let t = i as f64 / n as f64;
            out.push((a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t));
        }
    }
    if let Some(&l) = p.last() {
        out.push(l);
    }
    out
}

/// The chalk left on a wall by `lines`, a stick `width` px wide: how much each
/// pixel of a `w × h` canvas holds (0..1). The chalk catches on the wall's
/// tooth (fixed grains about a millimetre across, coarser patches of the
/// wall's texture), heavier where it was pressed, gone where it skipped; its
/// dust has fallen down the wall under the strokes. No draws.
fn chalk_raster(lines: &[ChalkLine], w: f64, h: f64, width: f64, seed: u32) -> Vec<f64> {
    let (n, m) = (w as usize, h as usize);
    // each segment with its bounds, so a pixel looks only at those near it
    let mut segs: Vec<((f64, f64), (f64, f64), f64, f64, u32)> = vec![];
    for (li, l) in lines.iter().enumerate() {
        for k in 0..l.pts.len().saturating_sub(1) {
            segs.push((l.pts[k], l.pts[k + 1], l.press[k], l.press[k + 1], seed.wrapping_add(li as u32 * 97)));
        }
    }
    let r = width / 2.0;
    let cells = chalk_cells(&segs, n, m, r + 1.0);
    let cw = n.div_ceil(CHALK_CELL);
    let mut dep = vec![0.0f64; n * m];
    let mut bytes = vec![0u8; n * m * 4];
    par_rows(&mut bytes, n * 4, |j, out| {
        let y = j as f64 + 0.5;
        for i in 0..n {
            let x = i as f64 + 0.5;
            let mut keep = 0.0f64;
            for &si in &cells[(j / CHALK_CELL) * cw + i / CHALK_CELL] {
                let (a, b, pa, pb, ls) = segs[si as usize];
                if x < a.0.min(b.0) - r - 1.0 || x > a.0.max(b.0) + r + 1.0 || y < a.1.min(b.1) - r - 1.0 || y > a.1.max(b.1) + r + 1.0 {
                    continue;
                }
                let (ex, ey) = (b.0 - a.0, b.1 - a.1);
                let l2 = (ex * ex + ey * ey).max(1e-9);
                let t = (((x - a.0) * ex + (y - a.1) * ey) / l2).clamp(0.0, 1.0);
                let d = (x - a.0 - ex * t).hypot(y - a.1 - ey * t);
                // the stick's edge is uneven; it skips along the stroke
                let rr = r * (0.85 + 0.3 * vn(ls ^ 0x5a, x * 0.35, y * 0.35));
                let cover = sstep(rr + 0.8, rr - 0.8, d);
                if cover <= 0.0 {
                    continue;
                }
                let skip = sstep(0.2, 0.5, vn(ls ^ 0x77, (a.0 + ex * t) * 0.11, (a.1 + ey * t) * 0.11));
                let press = (pa + (pb - pa) * t) * (0.3 + 0.7 * skip) * (1.0 - 0.45 * (d / rr).powi(2));
                keep = 1.0 - (1.0 - keep) * (1.0 - cover * press);
            }
            // the wall's tooth: grains ~1.3 px, and coarser patches of it
            let grain = hash(seed ^ 0x1d, (x / 1.3) as i32, (y / 1.3) as i32);
            let patch = vn(seed ^ 0x2e, x * 0.12, y * 0.12);
            let tooth = 0.7 * grain + 0.3 * patch;
            // chalk on a wall holds only on the tooth's peaks
            let a = sstep(tooth - 0.06, tooth + 0.05, keep * 0.82);
            out[i * 4] = (a * 255.0).round() as u8;
            out[i * 4 + 1] = (keep * 255.0).round() as u8;
        }
    });
    for k in 0..n * m {
        dep[k] = f64::from(bytes[k * 4]) / 255.0;
    }
    // the dust: what fell from each stroke, down the wall under it, fading
    let mut out = dep.clone();
    for i in 0..n {
        let mut acc = 0.0f64;
        for j in 0..m {
            let k = j * n + i;
            let held = f64::from(bytes[k * 4 + 1]) / 255.0;
            acc = acc * 0.93 + held * 0.1;
            let speck = hash(seed ^ 0x3f, i as i32, j as i32);
            let dust = acc * (0.12 + 0.5 * (speck > 0.93) as u8 as f64);
            out[k] = 1.0 - (1.0 - out[k]) * (1.0 - dust.min(0.5));
        }
    }
    out
}
