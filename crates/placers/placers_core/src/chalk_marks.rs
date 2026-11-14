//! `place_chalk_marks`
use game::prelude::*;
use placers_kit::place::shuffle;

// Chalk marks: 0-3 marks in chalk on random walls, in a stranger's hand:
// a number underlined, a looping scribble where someone tried the chalk, a
// fitter's measure (a line with a tick at each end, its length over it);
// never a lone stroke, a tally, an arrow or a circle (hers and the place's
// are those), the grain of the chalk dragged over the wall, its dust under
// it. The mark is laid on the wall in upright strips each no wider than the
// old streak's settling probe takes in at once (`CHALK_STRIP`), so settling
// finds the same wall face for it and pins nothing new.
pub(crate) fn place_chalk_marks(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.35 {
        return;
    } // 35% chance of NO chalk this level
    let count = 1 + (rnd() * 3.0).floor() as usize; // 1-3 marks
    let mut candidates = wall_candidates(g);
    shuffle(&mut candidates);
    let chalk_color = chalk_color_by_weather(g.weather_key());
    let mut i = 0;
    while i < count && i < candidates.len() {
        let c = candidates[i];
        // the old streak's width and height: which scrawl, and its size
        let w = 0.012 + rnd() * 0.006;
        let h = 0.10 + rnd() * 0.06;
        let u = (w - 0.012) / 0.006;
        let kind = ((u * 3.0).floor() as usize).min(2);
        let code = ((u * 1.0e6) as u64) ^ (((h - 0.10) / 0.06 * 1.0e6) as u64).rotate_left(17);
        let full = CHALK_PLANE * (0.88 + (h - 0.10) / 0.06 * 0.24);
        let tex = chalk_scrawl_tex(kind, code);
        let chalk_mat = lambert!({
            map: &tex,
            color: chalk_color,
            transparent: true,
            depthWrite: false,
            side: DoubleSide,
            polygonOffset: true,
            polygonOffsetFactor: -2,
            polygonOffsetUnits: -2
        });
        let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.02);
        let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.02);
        let mid_height = 0.6 + rnd() * 0.8; // 0.6 - 1.4 m
        let (turn, mx, mz) = if c.dx.abs() == 1 {
            (if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 }, wall_x, wall_z + (rnd() - 0.5) * 1.4)
        } else {
            (if c.dz > 0 { PI } else { 0.0 }, wall_x + (rnd() - 0.5) * 1.4, wall_z)
        };
        // where the wall's face steps (a jamb, a pillar, a corner) within its
        // width, a smaller one, so none of it hangs off an edge
        let (ax, az) = (turn.dcos(), -turn.dsin());
        let (px, pz) = (mx - c.dx as f64 * 0.01, mz - c.dz as f64 * 0.01);
        let inside = |off: f64| g.circle_hits_wall(px + ax * off, pz + az * off, 0.01);
        let mid = inside(0.0);
        let mut size = full;
        while size > 0.09 && (inside(-size / 2.0) != mid || inside(size / 2.0) != mid) {
            size *= 0.8;
        }
        // the strips, left to right across the wall, each at the old streak's
        // place with its slice of the scrawl beside it (a group at the origin:
        // settling takes each strip by itself, where it took the streak)
        let mark = group();
        let n = (size / CHALK_STRIP).ceil().max(1.0) as usize;
        let sw = size / n as f64;
        for k in 0..n {
            let geo = plane_geo!(sw, size);
            let (u0, u1) = (k as f32 / n as f32, (k + 1) as f32 / n as f32);
            geo.set_uv(vec![u0, 1.0, u1, 1.0, u0, 0.0, u1, 0.0]);
            geo.translate((k as f64 + 0.5) * sw - size / 2.0, 0, 0);
            let strip = mesh(geo, &chalk_mat);
            strip.rotation().y = turn;
            strip.position().set(mx, mid_height, mz);
            mark.add(&strip);
        }
        g.world.add(&mark);
        i += 1;
    }
}

/// A wall-facing slot: open tile (x, z) with a wall at (x + dx, z + dz).
#[derive(Clone, Copy)]
struct WallCand {
    x: i32,
    z: i32,
    dx: i32,
    dz: i32,
}

/// Every (open tile, wall direction) pair whose neighbour is a wall or off the grid.
fn wall_candidates(g: &Game) -> Vec<WallCand> {
    let mut v = vec![];
    for z in 1..g.h - 1 {
        for x in 1..g.w - 1 {
            if g.tiles[g.t_idx(x, z)] != 0 {
                continue;
            }
            for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let (nx, nz) = (x + dx, z + dz);
                if g.is_wall_tile(nx, nz) {
                    v.push(WallCand { x, z, dx, dz });
                }
            }
        }
    }
    v
}

/// The chalk mark's square (m): its scrawl in the upper part, the dust below.
const CHALK_PLANE: f64 = 0.22;

/// The widest strip a mark is laid in (m): settling probes a wall face only
/// at the middle of anything narrower than 8 cm, as it did the old streak.
const CHALK_STRIP: f64 = 0.07;

/// A scrawl in chalk on a wall (`kind` 0 a number underlined, 1 a looping
/// scribble, 2 a fitter's measure), drawn with a stick of blackboard chalk
/// (10 mm: a stroke 6-8 mm wide), its grain broken by the wall's tooth, its
/// dust fallen below it. White, tinted by the material.
fn chalk_scrawl_tex(kind: usize, code: u64) -> Tex {
    canvas_tex_opt(256, 256, false, move |ctx, w, h| {
        ctx.clear_rect(0, 0, w, h);
        // 1 px = 0.86 mm; the scrawl in the upper two thirds
        let mut strokes: Vec<Vec<(f64, f64)>> = vec![];
        let jig = |p: (f64, f64), k: f64| (p.0 + (rnd() - 0.5) * k, p.1 + (rnd() - 0.5) * k);
        // a number in a quick upright hand, its digits `dw` x `dh`, centred on cx
        let number = |strokes: &mut Vec<Vec<(f64, f64)>>, n: u64, cx: f64, y0: f64, dw: f64, dh: f64, slant: f64| -> f64 {
            let digits: Vec<u32> = n.to_string().chars().filter_map(|c| c.to_digit(10)).collect();
            let gap = dw * 0.28;
            let total = digits.len() as f64 * (dw + gap) - gap;
            let mut x0 = cx - total / 2.0;
            for d in digits {
                for st in chalk_digit(d) {
                    strokes.push(st.iter().map(|&(u, v)| jig((x0 + u * dw + (1.0 - v) * dh * slant, y0 + v * dh), 2.0)).collect());
                }
                x0 += dw + gap;
            }
            total
        };
        match kind {
            0 => {
                // a number, 9 cm tall, underlined quickly
                let slant = 0.06 + (code % 7) as f64 * 0.015;
                let total = number(&mut strokes, 2 + code % 97, w / 2.0, 28.0, 56.0, 100.0, slant);
                let ul_y = 28.0 + 100.0 + 14.0;
                strokes.push(vec![((w - total) / 2.0 - 10.0, ul_y + 3.0), (w / 2.0, ul_y), ((w + total) / 2.0 + 14.0, ul_y - 4.0)]);
            }
            1 => {
                // a looping scribble along a line, as a hand tries the chalk
                let loops = 5.0 + (code % 3) as f64;
                let mut pts = vec![];
                let (x0, x1, cy) = (34.0, w - 34.0, 92.0 + (code % 13) as f64);
                for k in 0..=220 {
                    let t = k as f64 / 220.0;
                    let a = t * loops * PI * 2.0;
                    let r = 24.0 + 9.0 * (t * 7.0 + code as f64 * 0.1).dsin();
                    pts.push((x0 + (x1 - x0) * t + r * 0.55 * a.dcos(), cy - r * 0.8 * a.dsin() + 10.0 * (t * 3.0).dsin()));
                }
                strokes.push(pts);
            }
            _ => {
                // a fitter's measure: a line across with a tick at each end,
                // its length in centimetres written over it
                let (x0, x1) = (30.0 + (code % 11) as f64, w - 28.0 - (code % 7) as f64);
                let y = 132.0 + (code % 9) as f64;
                strokes.push(vec![jig((x0, y), 2.0), jig(((x0 + x1) / 2.0, y + 1.5), 2.0), jig((x1, y - 2.0), 2.0)]);
                strokes.push(vec![jig((x0 + 1.5, y - 17.0), 1.5), jig((x0 - 1.0, y + 15.0), 1.5)]);
                strokes.push(vec![jig((x1 + 1.5, y - 19.0), 1.5), jig((x1 - 1.0, y + 13.0), 1.5)]);
                number(&mut strokes, 60 + code % 140, (x0 + x1) / 2.0, y - 78.0, 34.0, 58.0, 0.08);
            }
        }
        let mut low = 0.0f64;
        let (mut lx, mut hx) = (w, 0.0f64);
        for st in &strokes {
            let pts = chalk_smooth(st);
            chalk_stroke(ctx, &pts, 9.5);
            for &(x, y) in &pts {
                low = low.max(y);
                lx = lx.min(x);
                hx = hx.max(x);
            }
        }
        // its dust: a faint powder down the wall under the strokes, and specks
        for _ in 0..14 {
            let x = lx + rnd() * (hx - lx);
            let y = low + 4.0 + rnd() * 26.0;
            chalk_soft(ctx, x, y, 7.0 + rnd() * 10.0, 1.6 + rnd(), 0.05 + rnd() * 0.06);
        }
        for _ in 0..160 {
            let x = lx - 6.0 + rnd() * (hx - lx + 12.0);
            let y = low - 10.0 + rnd() * rnd() * (h - low + 6.0);
            ctx.set_fill_style(format!("rgba(255,255,255,{:.2})", 0.15 + rnd() * 0.45));
            ctx.fill_rect(x, y, 0.8 + rnd(), 0.8 + rnd());
        }
    })
}

/// A soft round of chalk dust at (x, y), `r` px, drawn out `k` times as tall
/// as it is wide (the context squashed, so its fade is never cut).
fn chalk_soft(ctx: &mut Canvas, x: f64, y: f64, r: f64, k: f64, alpha: f64) {
    ctx.save();
    ctx.translate(x, y);
    ctx.scale(1, k);
    let gr = ctx.create_radial_gradient(0, 0, 0, 0, 0, r);
    gr.add_color_stop(0, &format!("rgba(255,255,255,{alpha:.3})"));
    gr.add_color_stop(1, "rgba(255,255,255,0)");
    ctx.set_fill_style(&gr);
    ctx.begin_path();
    ctx.arc(0, 0, r, 0, 7);
    ctx.fill();
    ctx.restore();
}

/// The strokes of a digit in a unit box (u across, v down), as a quick hand
/// draws it.
fn chalk_digit(d: u32) -> Vec<Vec<(f64, f64)>> {
    let arc = |cx: f64, cy: f64, rx: f64, ry: f64, a0: f64, a1: f64| -> Vec<(f64, f64)> {
        (0..=12).map(|k| a0 + (a1 - a0) * k as f64 / 12.0).map(|a| (cx + rx * a.dcos(), cy + ry * a.dsin())).collect()
    };
    match d {
        0 => vec![arc(0.5, 0.5, 0.48, 0.5, -1.4, 4.95)],
        1 => vec![vec![(0.22, 0.2), (0.55, 0.0), (0.55, 1.0)]],
        2 => {
            let mut s = arc(0.5, 0.28, 0.45, 0.28, 3.4, 6.6);
            s.extend([(0.05, 1.0), (1.0, 0.98)]);
            vec![s]
        }
        3 => {
            let mut s = arc(0.45, 0.25, 0.45, 0.25, 3.6, 7.6);
            s.extend(arc(0.45, 0.73, 0.5, 0.27, -1.4, 2.6));
            vec![s]
        }
        4 => vec![vec![(0.7, 1.0), (0.7, 0.0), (0.0, 0.68), (1.0, 0.68)]],
        5 => {
            let mut s = vec![(0.9, 0.0), (0.18, 0.0), (0.12, 0.43)];
            s.extend(arc(0.45, 0.68, 0.5, 0.31, -2.4, 2.7));
            vec![s]
        }
        6 => {
            let mut s = vec![(0.85, 0.05), (0.45, 0.2), (0.08, 0.62)];
            s.extend(arc(0.5, 0.72, 0.43, 0.27, 3.1, 9.4));
            vec![s]
        }
        7 => vec![vec![(0.0, 0.02), (0.95, 0.0), (0.35, 1.0)]],
        8 => {
            let mut s = arc(0.5, 0.25, 0.38, 0.25, 1.6, 7.6);
            s.extend(arc(0.5, 0.74, 0.46, 0.26, -1.6, 4.7));
            vec![s]
        }
        _ => {
            let mut s = arc(0.48, 0.3, 0.44, 0.3, 0.2, 6.6);
            s.extend([(0.9, 0.4), (0.62, 1.0)]);
            vec![s]
        }
    }
}

/// A hand's line through the points (Chaikin, twice).
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

/// The wall's tooth at (x, y) px: what catches the chalk (0..1), fixed for
/// the wall so every stroke skips in the same places.
fn chalk_tooth(x: f64, y: f64) -> f64 {
    let (i, j) = ((x / 1.6).floor() as i64, (y / 1.6).floor() as i64);
    let mut v = (i.wrapping_mul(374_761_393) ^ j.wrapping_mul(668_265_263)) as u64;
    v = (v ^ (v >> 13)).wrapping_mul(1_274_126_177);
    ((v ^ (v >> 16)) & 0xffff) as f64 / 65535.0
}

/// A stroke of chalk along the points: grains caught on the wall's tooth,
/// heavier where pressed, broken where it skipped, its ends lifting.
fn chalk_stroke(ctx: &mut Canvas, pts: &[(f64, f64)], width: f64) {
    let mut len = 0.0;
    for k in 1..pts.len() {
        len += ((pts[k].0 - pts[k - 1].0).dpowi(2) + (pts[k].1 - pts[k - 1].1).dpowi(2)).sqrt();
    }
    let mut s = 0.0;
    let wob = rnd() * 6.0;
    for k in 1..pts.len() {
        let (a, b) = (pts[k - 1], pts[k]);
        let seg = ((b.0 - a.0).dpowi(2) + (b.1 - a.1).dpowi(2)).sqrt();
        if seg < 1e-6 {
            continue;
        }
        let (tx, ty) = ((b.0 - a.0) / seg, (b.1 - a.1) / seg);
        let (nx, ny) = (-ty, tx);
        let steps = (seg / 0.7).ceil() as usize;
        for j in 0..steps {
            let t = j as f64 / steps as f64;
            let (cx, cy) = (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t);
            let at = (s + seg * t) / len.max(1e-6);
            let ends = (at / 0.06).min(1.0) * ((1.0 - at) / 0.1).min(1.0);
            let press = (0.62 + 0.25 * (at * 9.0 + wob).dsin() + 0.13 * (at * 23.0 + wob).dsin()) * (0.4 + 0.6 * ends);
            for _ in 0..6 {
                let off = (rnd() - 0.5) * width * (0.7 + 0.3 * ends);
                let (x, y) = (cx + nx * off, cy + ny * off);
                let edge = 1.0 - (2.0 * off / width).abs();
                if chalk_tooth(x, y) + 0.3 * rnd() < press * (0.75 + 0.6 * edge) {
                    ctx.set_fill_style(format!("rgba(255,255,255,{:.2})", 0.55 + 0.45 * rnd()));
                    ctx.fill_rect(x - 0.8, y - 0.8, 1.4 + rnd() * 1.0, 1.4 + rnd() * 1.0);
                }
            }
        }
        s += seg;
    }
}
