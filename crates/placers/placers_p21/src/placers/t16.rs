//! Batch `t16`'s mark in this crate, made real (`docs/PROPS.md` P3, "toys
//! stay toys"; `t16.notes.md`): the crayon graffiti, a small child's drawing
//! in wax crayon on a wall, the wall's tooth showing through its strokes.
//! Its placer and what only it uses; it keeps the old placer's draws, call
//! for call.
use crate::{mount_on_wall, wall_candidates};
use game::prelude::*;

/// The drawing's plane on the wall (m), as the old decal's (settling probes
/// a wall face across it, so it keeps its width)
const PLANE_W: f64 = 0.60;
const PLANE_H: f64 = 0.50;
/// Its texture: 1.17 mm a pixel
const TEX_W: usize = 512;
const TEX_H: usize = 427;

/// R512 crayon drawing on a wall: 5% chance per level. A small child's
/// drawing in wax crayon (a stick of 8 mm: strokes 5-8 mm wide, broader
/// where its side filled a shape), low on the wall where the child could
/// reach: the ground scribbled green, a sun in a corner, the family holding
/// hands, a house or a tree, flowers; never words, never a sign. The wax
/// caught on the wall's tooth (its pits show through), streaked along each
/// stroke, dusty; sometimes half scrubbed off. Each drawing its own, by
/// where it is.
pub(crate) fn place_crayon_graffiti(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = candidates[(rnd() * candidates.len() as f64).floor() as usize];
    // the drawing, by the wall it is on and the floor (no draw)
    let salt = (c.x as u32).wrapping_mul(0x9E37_79B1)
        ^ (c.z as u32).wrapping_mul(0x85EB_CA77)
        ^ ((c.dx + 2) as u32 * 5 + (c.dz + 2) as u32).wrapping_mul(0xC2B2_AE3D);
    let tex = crayon_tex(salt ^ g.current_seed);
    let mat = lambert!({
        map: &tex,
        transparent: true,
        depthWrite: false,
        polygonOffset: true,
        polygonOffsetFactor: -2,
        polygonOffsetUnits: -2
    });
    let decal = mesh(plane_geo!(PLANE_W, PLANE_H), &mat);
    mount_on_wall(g, &decal, &c, 0.008, 0.9, 0.6); // low-mounted (child-height)
    g.world.add(&decal);
}

/// The crayons (sRGB): red, blue, green, yellow, orange, brown, black,
/// purple, pink
const CRAYONS: [[f64; 3]; 9] = [
    [176.0, 54.0, 46.0],
    [50.0, 84.0, 156.0],
    [60.0, 122.0, 60.0],
    [218.0, 180.0, 56.0],
    [212.0, 112.0, 46.0],
    [112.0, 74.0, 46.0],
    [40.0, 36.0, 34.0],
    [104.0, 66.0, 130.0],
    [204.0, 114.0, 142.0],
];
const RED: u8 = 0;
const BLUE: u8 = 1;
const GREEN: u8 = 2;
const YELLOW: u8 = 3;
const ORANGE: u8 = 4;
const BROWN: u8 = 5;
const BLACK: u8 = 6;
const PURPLE: u8 = 7;
const PINK: u8 = 8;

/// A small stream of its own for the drawing (xorshift), seeded by where it is
struct Hand(u32);

impl Hand {
    fn f(&mut self) -> f64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.0 = x;
        f64::from(x) / 4_294_967_296.0
    }
    fn range(&mut self, a: f64, b: f64) -> f64 {
        a + (b - a) * self.f()
    }
    fn pick(&mut self, c: &[u8]) -> u8 {
        c[((self.f() * c.len() as f64) as usize).min(c.len() - 1)]
    }
}

/// The wax laid so far: how much (0..1) and whose colour, a pixel each
struct Wax {
    ink: Vec<f32>,
    col: Vec<u8>,
    strokes: u32,
    /// where the drawing's left edge is on the texture (px)
    ox: f64,
}

impl Wax {
    /// A stroke of the crayon along `pts` (px): a hand's line (its points
    /// wobbled, then smoothed), `width` px across, pressed `press` (0..1),
    /// lighter at its ends; the wax streaked along it by the crayon's worn
    /// facets
    fn stroke(&mut self, hand: &mut Hand, pts: &[(f64, f64)], width: f64, press: f64, col: u8) {
        if pts.len() < 2 {
            return;
        }
        self.strokes += 1;
        let id = self.strokes;
        let mut p: Vec<(f64, f64)> = pts.iter().map(|&(x, y)| (x + hand.range(-1.2, 1.2), y + hand.range(-1.2, 1.2))).collect();
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
        self.lay(hand, &p, width, press, col, id);
    }

    /// A stroke with corners: straight runs between `pts`, each wavering a
    /// little as a hand's does, the corners kept
    fn lines(&mut self, hand: &mut Hand, pts: &[(f64, f64)], width: f64, press: f64, col: u8) {
        if pts.len() < 2 {
            return;
        }
        self.strokes += 1;
        let id = self.strokes;
        let mut q = vec![];
        for k in 0..pts.len() - 1 {
            let (a, b) = (pts[k], pts[k + 1]);
            let n = ((b.0 - a.0).hypot(b.1 - a.1) / 10.0).ceil().max(1.0) as usize;
            for j in 0..n {
                let t = j as f64 / n as f64;
                let wob = if j == 0 { 0.0 } else { 1.0 };
                q.push((a.0 + (b.0 - a.0) * t + wob * hand.range(-1.0, 1.0), a.1 + (b.1 - a.1) * t + wob * hand.range(-1.0, 1.0)));
            }
        }
        q.push(pts[pts.len() - 1]);
        self.lay(hand, &q, width, press, col, id);
    }

    /// The wax along the line `p`, dab by dab
    #[allow(clippy::too_many_arguments)]
    fn lay(&mut self, hand: &mut Hand, p: &[(f64, f64)], width: f64, press: f64, col: u8, id: u32) {
        let len: f64 = p.windows(2).map(|w| (w[1].0 - w[0].0).hypot(w[1].1 - w[0].1)).sum();
        let wob = hand.range(0.0, 6.0);
        let mut s = 0.0;
        for k in 1..p.len() {
            let (a, b) = (p[k - 1], p[k]);
            let seg = (b.0 - a.0).hypot(b.1 - a.1);
            if seg < 1e-6 {
                continue;
            }
            let (tx, ty) = ((b.0 - a.0) / seg, (b.1 - a.1) / seg);
            let steps = (seg / 0.7).ceil() as usize;
            for j in 0..steps {
                let t = j as f64 / steps as f64;
                let (cx, cy) = (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t);
                let at = (s + seg * t) / len.max(1e-6);
                let ends = (at / 0.05).min(1.0) * ((1.0 - at) / 0.08).min(1.0);
                let pr = press * (0.82 + 0.12 * (at * 11.0 + wob).sin() + 0.06 * (at * 29.0 + wob).sin()) * (0.55 + 0.45 * ends);
                let r = width / 2.0 * (0.9 + 0.1 * (at * 7.0 + wob).cos());
                self.dab(cx + self.ox, cy, r, pr, -ty, tx, id, col);
            }
            s += seg;
        }
    }

    /// One dab of the stroke at (cx, cy), radius r, across (nx, ny)
    #[allow(clippy::too_many_arguments)]
    fn dab(&mut self, cx: f64, cy: f64, r: f64, pr: f64, nx: f64, ny: f64, id: u32, col: u8) {
        let (x0, x1) = ((cx - r - 1.0).floor().max(0.0) as usize, ((cx + r + 1.0).ceil() as usize).min(TEX_W - 1));
        let (y0, y1) = ((cy - r - 1.0).floor().max(0.0) as usize, ((cy + r + 1.0).ceil() as usize).min(TEX_H - 1));
        if cx + r < 0.0 || cy + r < 0.0 || x0 > x1 || y0 > y1 {
            return;
        }
        for y in y0..=y1 {
            for x in x0..=x1 {
                let (dx, dy) = (x as f64 + 0.5 - cx, y as f64 + 0.5 - cy);
                let d = dx.hypot(dy);
                let cov = (r - d + 0.5).clamp(0.0, 1.0);
                if cov <= 0.0 {
                    continue;
                }
                // the streaks: fixed across the stroke, running along it
                let across = ((dx * nx + dy * ny) * 1.3 + 40.0).floor() as i32;
                let streak = 0.72 + 0.28 * t16_hash(id.wrapping_mul(0x2545_F491), across, 0);
                let v = (pr * cov * streak) as f32;
                let i = y * TEX_W + x;
                if v > self.ink[i] * 0.6 && v > 0.12 {
                    self.col[i] = col;
                }
                self.ink[i] = self.ink[i].max(v);
            }
        }
    }

    /// A shape filled the way a child fills it: the crayon on its side,
    /// back and forth across it at a slant, inside `inside`
    #[allow(clippy::too_many_arguments)]
    fn hatch(
        &mut self,
        hand: &mut Hand,
        bx: (f64, f64, f64, f64),
        inside: &dyn Fn(f64, f64) -> bool,
        gap: f64,
        slant: f64,
        press: f64,
        col: u8,
    ) {
        let (x0, y0, x1, y1) = bx;
        let mut y = y0 + gap * 0.5;
        let mut fwd = true;
        let mut pts = vec![];
        while y < y1 {
            // the row's run inside the shape
            let mut run: Vec<f64> = vec![];
            let mut x = x0;
            while x <= x1 {
                if inside(x, y + (x - x0) * slant) {
                    run.push(x);
                }
                x += 1.5;
            }
            if let (Some(&a), Some(&b)) = (run.first(), run.last()) {
                let (a, b) = (a + hand.range(-2.0, 3.0), b + hand.range(-3.0, 2.0));
                let (s, e) = if fwd { (a, b) } else { (b, a) };
                pts.push((s, y + (s - x0) * slant));
                pts.push((e, y + (e - x0) * slant));
                fwd = !fwd;
            }
            y += gap * hand.range(0.8, 1.25);
        }
        self.stroke(hand, &pts, 9.0, press, col);
    }
}

/// A closed loop round (cx, cy), radii rx, ry, a child's: not quite closed,
/// overlapping where it began
fn loop_pts(hand: &mut Hand, cx: f64, cy: f64, rx: f64, ry: f64) -> Vec<(f64, f64)> {
    let a0 = hand.range(0.0, PI * 2.0);
    let over = hand.range(0.15, 0.45);
    let n = 18;
    (0..=n)
        .map(|k| {
            let a = a0 + (PI * 2.0 + over) * k as f64 / n as f64;
            let w = 1.0 + 0.06 * (a * 3.0 + a0).sin();
            (cx + rx * w * a.cos(), cy + ry * w * a.sin())
        })
        .collect()
}

/// A child's drawing in crayon (TEX_W x TEX_H: 1.17 mm a pixel), its wax
/// caught on the wall's tooth; transparent where there is none
fn crayon_tex(salt: u32) -> Tex {
    canvas_tex_opt(TEX_W as f64, TEX_H as f64, false, move |ctx, cw, ch| {
        let mut hand = Hand(salt | 1);
        for _ in 0..4 {
            hand.f();
        }
        // the drawing kept to the middle of the plane, clear of a corner it
        // may run into (the plane is the old decal's width)
        let x0 = TEX_W as f64 * 0.06;
        let mut wax = Wax { ink: vec![0.0; TEX_W * TEX_H], col: vec![0; TEX_W * TEX_H], strokes: 0, ox: x0 };
        let (w, h) = (TEX_W as f64 - 2.0 * x0, TEX_H as f64);
        let ground = h * hand.range(0.80, 0.86);
        let sun_left = hand.f() < 0.5;
        let house = hand.f() < 0.6;
        // the sun, in its corner: round, filled, its rays
        {
            let (cx, cy) = (if sun_left { hand.range(62.0, 92.0) } else { w - hand.range(62.0, 92.0) }, hand.range(58.0, 78.0));
            let r = hand.range(32.0, 42.0);
            let col = if hand.f() < 0.7 { YELLOW } else { ORANGE };
            let fill = move |x: f64, y: f64| (x - cx).hypot(y - cy) < r - 2.0;
            wax.hatch(&mut hand, (cx - r, cy - r, cx + r, cy + r), &fill, 6.0, 0.25, 0.55, col);
            let ring = loop_pts(&mut hand, cx, cy, r, r);
            wax.stroke(&mut hand, &ring, 6.5, 0.9, if col == YELLOW { ORANGE } else { col });
            let n = 8 + (hand.f() * 5.0) as usize;
            for k in 0..n {
                let a = PI * 2.0 * k as f64 / n as f64 + hand.range(-0.12, 0.12);
                let (r0, r1) = (r + hand.range(5.0, 9.0), r + hand.range(20.0, 32.0));
                wax.stroke(&mut hand, &[(cx + r0 * a.cos(), cy + r0 * a.sin()), (cx + r1 * a.cos(), cy + r1 * a.sin())], 6.0, 0.85, col);
            }
        }
        // the house on the ground, its roof, door and window
        let (hx0, hx1) = if house {
            let hw = hand.range(150.0, 185.0);
            let x0 = if sun_left { w * hand.range(0.52, 0.60) } else { w * hand.range(0.14, 0.22) };
            let hh = hand.range(118.0, 145.0);
            let top = ground - hh;
            let wall = hand.pick(&[BROWN, BLACK, BLUE, RED]);
            let tilt = hand.range(-5.0, 5.0);
            let lean = hand.range(-4.0, 4.0);
            wax.lines(&mut hand, &[(x0, ground), (x0 + tilt, top), (x0 + hw + tilt, top + lean), (x0 + hw, ground)], 6.0, 0.9, wall);
            let roof = hand.pick(&[RED, RED, BROWN, ORANGE]);
            let apex = (x0 + hw / 2.0 + hand.range(-14.0, 14.0), top - hand.range(70.0, 96.0));
            let rp = [(x0 - 14.0, top + 2.0), apex, (x0 + hw + 14.0, top + 2.0), (x0 - 10.0, top + 1.0)];
            let (ax, ay) = apex;
            let inroof = move |x: f64, y: f64| {
                let t = (y - ay) / (top - ay);
                (0.0..1.0).contains(&t) && (x - ax).abs() < t * (hw / 2.0 + 12.0)
            };
            wax.hatch(&mut hand, (x0 - 14.0, ay, x0 + hw + 14.0, top), &inroof, 7.0, -0.35, 0.6, roof);
            wax.lines(&mut hand, &rp, 6.5, 0.9, roof);
            // the door, its knob
            let (dw, dh) = (hand.range(32.0, 42.0), hand.range(56.0, 70.0));
            let dx = x0 + hw * hand.range(0.18, 0.55);
            wax.lines(&mut hand, &[(dx, ground), (dx, ground - dh), (dx + dw, ground - dh), (dx + dw, ground)], 5.5, 0.85, BROWN);
            wax.stroke(&mut hand, &[(dx + dw - 7.0, ground - dh / 2.0), (dx + dw - 6.0, ground - dh / 2.0 + 1.0)], 5.0, 0.9, BLACK);
            // a window with its cross
            let ws = hand.range(30.0, 38.0);
            let wx = if dx - x0 > hw * 0.4 { x0 + 14.0 } else { x0 + hw - ws - 14.0 };
            let wy = top + hand.range(14.0, 22.0);
            wax.lines(&mut hand, &[(wx, wy), (wx + ws, wy), (wx + ws, wy + ws), (wx, wy + ws), (wx, wy - 1.0)], 5.0, 0.85, BLUE);
            wax.stroke(&mut hand, &[(wx + ws / 2.0, wy), (wx + ws / 2.0, wy + ws)], 4.5, 0.8, BLUE);
            wax.stroke(&mut hand, &[(wx, wy + ws / 2.0), (wx + ws, wy + ws / 2.0)], 4.5, 0.8, BLUE);
            (x0 - 20.0, x0 + hw + 20.0)
        } else {
            // else a tree: its trunk and a scribbled crown
            let tx = if sun_left { w * hand.range(0.70, 0.80) } else { w * hand.range(0.18, 0.28) };
            let th = hand.range(90.0, 125.0);
            wax.stroke(&mut hand, &[(tx - 9.0, ground), (tx - 7.0, ground - th)], 6.0, 0.9, BROWN);
            wax.stroke(&mut hand, &[(tx + 9.0, ground), (tx + 8.0, ground - th)], 6.0, 0.9, BROWN);
            let (cx, cy, r) = (tx, ground - th - 44.0, hand.range(50.0, 64.0));
            let crown = move |x: f64, y: f64| ((x - cx) / r).hypot((y - cy) / (r * 0.85)) < 0.95;
            wax.hatch(&mut hand, (cx - r, cy - r, cx + r, cy + r), &crown, 6.0, 0.4, 0.6, GREEN);
            let mut pts = vec![];
            for k in 0..=40 {
                let a = PI * 2.0 * k as f64 / 40.0;
                let bump = 1.0 + 0.12 * (a * 7.0).sin().abs();
                pts.push((cx + r * bump * a.cos(), cy + r * 0.85 * bump * a.sin()));
            }
            wax.stroke(&mut hand, &pts, 6.0, 0.9, GREEN);
            (tx - r - 10.0, tx + r + 10.0)
        };
        // the family, holding hands, on the ground beside it
        let n = 2 + (hand.f() * 3.0) as usize;
        let span_l = if hx0 > w * 0.4 { 30.0 } else { hx1 + 10.0 };
        let span_r = if hx0 > w * 0.4 { hx0 - 10.0 } else { w - 30.0 };
        // no more of them than stand side by side there
        let n = n.min(((span_r - span_l) / 64.0).floor().max(1.0) as usize);
        let step = ((span_r - span_l) / n as f64).min(96.0);
        let mut prev_hand: Option<(f64, f64)> = None;
        for k in 0..n {
            let fx = span_l + step * (k as f64 + 0.5) + hand.range(-6.0, 6.0);
            let tall = if k < 2 && n > 2 { hand.range(150.0, 180.0) } else { hand.range(100.0, 140.0) };
            let col = hand.pick(&[RED, BLUE, PURPLE, BLACK, GREEN, ORANGE, PINK, BROWN]);
            let head_r = tall * hand.range(0.13, 0.16);
            let (hx, hy) = (fx, ground - tall + head_r);
            let oval = head_r * hand.range(0.95, 1.1);
            let head = loop_pts(&mut hand, hx, hy, head_r, oval);
            wax.stroke(&mut hand, &head, 5.0, 0.9, col);
            // the face: two eyes, a smile
            wax.stroke(&mut hand, &[(hx - head_r * 0.38, hy - head_r * 0.2), (hx - head_r * 0.36, hy - head_r * 0.15)], 4.5, 0.9, BLACK);
            wax.stroke(&mut hand, &[(hx + head_r * 0.36, hy - head_r * 0.2), (hx + head_r * 0.38, hy - head_r * 0.15)], 4.5, 0.9, BLACK);
            let smile: Vec<(f64, f64)> = (0..=6)
                .map(|q| {
                    let a = PI * (0.2 + 0.6 * q as f64 / 6.0);
                    (hx + head_r * 0.5 * a.cos(), hy + head_r * 0.15 + head_r * 0.38 * a.sin())
                })
                .collect();
            wax.stroke(&mut hand, &smile, 4.0, 0.85, RED);
            // hair
            let hair = hand.pick(&[BROWN, YELLOW, BLACK, ORANGE]);
            let hp: Vec<(f64, f64)> = (0..9)
                .map(|q| {
                    let a = PI * (1.1 + 0.8 * q as f64 / 8.0);
                    let rr = head_r * if q % 2 == 0 { 1.05 } else { 1.35 };
                    (hx + rr * a.cos(), hy + rr * a.sin())
                })
                .collect();
            wax.stroke(&mut hand, &hp, 4.5, 0.85, hair);
            // the body: a line, or a dress
            let neck = hy + head_r;
            let hip = ground - tall * 0.36;
            if hand.f() < 0.45 {
                let dw = tall * 0.2;
                wax.lines(&mut hand, &[(fx, neck), (fx - dw, hip + 4.0), (fx + dw, hip + 4.0), (fx, neck)], 5.5, 0.9, col);
            } else {
                let lean = hand.range(-2.0, 2.0);
                wax.stroke(&mut hand, &[(fx, neck), (fx + lean, hip)], 5.5, 0.9, col);
            }
            // the legs and the feet
            let lx = tall * 0.12;
            wax.stroke(&mut hand, &[(fx - 2.0, hip), (fx - lx, ground - 2.0), (fx - lx - 9.0, ground - 1.0)], 5.0, 0.9, col);
            wax.stroke(&mut hand, &[(fx + 2.0, hip), (fx + lx, ground - 2.0), (fx + lx + 9.0, ground - 1.0)], 5.0, 0.9, col);
            // the arms out, reaching to the next one's hand
            let sh = neck + tall * 0.12;
            let reach = (step * 0.5 - 4.0).min(tall * 0.42);
            let lh = prev_hand.unwrap_or((fx - reach, sh + hand.range(-12.0, 10.0)));
            let rh = (fx + reach, sh + hand.range(-6.0, 14.0));
            wax.stroke(&mut hand, &[lh, (fx - reach * 0.4, sh + 2.0), (fx, sh)], 5.0, 0.9, col);
            wax.stroke(&mut hand, &[(fx, sh), (fx + reach * 0.4, sh + 2.0), rh], 5.0, 0.9, col);
            if k == n - 1 || prev_hand.is_none() {
                // a hand with its fingers spread at the free ends
                let (ex, ey, sgn) = if prev_hand.is_none() { (lh.0, lh.1, -1.0) } else { (rh.0, rh.1, 1.0) };
                for f in 0..4 {
                    let a = (-0.9 + 0.6 * f as f64) + if sgn < 0.0 { PI } else { 0.0 };
                    wax.stroke(&mut hand, &[(ex, ey), (ex + 9.0 * a.cos(), ey + 9.0 * a.sin())], 3.5, 0.8, col);
                }
            }
            prev_hand = Some(rh);
        }
        // the ground: grass scribbled along it
        {
            let mut pts = vec![];
            let mut x = hand.range(14.0, 30.0);
            let x1 = w - hand.range(14.0, 30.0);
            let mut up = false;
            while x < x1 {
                pts.push((x, ground + if up { -hand.range(9.0, 20.0) } else { hand.range(0.0, 4.0) }));
                x += hand.range(5.0, 9.0);
                up = !up;
            }
            wax.stroke(&mut hand, &pts, 5.5, 0.8, GREEN);
        }
        // flowers in the grass
        let nf = (hand.f() * 4.0) as usize;
        for _ in 0..nf {
            let fx = hand.range(30.0, w - 30.0);
            if fx > hx0 && fx < hx1 {
                continue;
            }
            let top = ground - hand.range(26.0, 44.0);
            let lean = hand.range(-3.0, 3.0);
            wax.stroke(&mut hand, &[(fx, ground + 2.0), (fx + lean, top)], 4.5, 0.85, GREEN);
            let pc = hand.pick(&[RED, PINK, PURPLE, YELLOW, ORANGE]);
            for q in 0..5 {
                let a = PI * 2.0 * q as f64 / 5.0;
                let petal = loop_pts(&mut hand, fx + 7.0 * a.cos(), top + 7.0 * a.sin(), 5.0, 5.0);
                wax.stroke(&mut hand, &petal, 4.0, 0.85, pc);
            }
        }
        // a cloud or two, a bird
        if hand.f() < 0.45 {
            let (cx, cy) = (if sun_left { w * hand.range(0.45, 0.7) } else { w * hand.range(0.3, 0.55) }, hand.range(40.0, 70.0));
            let mut pts = vec![];
            for q in 0..=4 {
                let bx = cx - 40.0 + 20.0 * q as f64;
                for k in 0..=6 {
                    let a = PI + PI * k as f64 / 6.0;
                    pts.push((bx + 11.0 * a.cos(), cy + 11.0 * a.sin()));
                }
            }
            pts.push((cx + 46.0, cy + 8.0));
            pts.push((cx - 46.0, cy + 8.0));
            pts.push((cx - 50.0, cy));
            wax.stroke(&mut hand, &pts, 5.0, 0.8, BLUE);
        }
        if hand.f() < 0.35 {
            let (bx, by) = (hand.range(w * 0.35, w * 0.65), hand.range(90.0, 130.0));
            wax.stroke(
                &mut hand,
                &[(bx - 12.0, by), (bx - 6.0, by - 6.0), (bx, by), (bx + 6.0, by - 6.0), (bx + 12.0, by)],
                4.0,
                0.85,
                BLACK,
            );
        }
        // a younger hand's scribble over a corner of it
        if hand.f() < 0.3 {
            let (cx, cy) = (hand.range(w * 0.2, w * 0.8), hand.range(h * 0.35, h * 0.65));
            let col = hand.pick(&[PURPLE, RED, BLUE]);
            let mut pts = vec![];
            for k in 0..60 {
                let a = k as f64 * 0.7;
                pts.push((cx + (14.0 + 8.0 * (k as f64 * 0.23).sin()) * a.cos() + k as f64 * 0.9, cy + 10.0 * a.sin()));
            }
            wax.stroke(&mut hand, &pts, 5.5, 0.75, col);
        }
        // someone tried to scrub it off: the wax thinned in a band, smeared
        let scrub = if hand.f() < 0.4 {
            Some((x0 + hand.range(w * 0.2, w * 0.8), hand.range(h * 0.3, h * 0.7), hand.range(60.0, 120.0)))
        } else {
            None
        };
        let seed = salt;
        let (ink, col) = (wax.ink, wax.col);
        let mut img = ctx.create_image_data(cw, ch);
        par_rows(&mut img.data, TEX_W * 4, |j, out| {
            for i in 0..TEX_W {
                let k = j * TEX_W + i;
                let (x, y) = (i as f64, j as f64);
                // the wall's tooth: what the wax catches on (high: a pit it skips)
                let tooth = 0.62 * t16_vn(0x7a11, x / 1.4, y / 1.4) + 0.38 * t16_vn(0x7a13, x / 5.0, y / 5.0);
                let mut a = f64::from(ink[k]);
                if let Some((sx, sy, sr)) = scrub {
                    let d = ((x - sx) / sr).hypot((y - sy) / (sr * 0.5));
                    let rub = (1.0 - d).clamp(0.0, 1.0) * (0.55 + 0.35 * t16_vn(seed ^ 0x31, x / 30.0, y / 3.0));
                    a *= 1.0 - rub;
                }
                let alpha = ((a - tooth * 0.55 - 0.05) / 0.22).clamp(0.0, 1.0) * 0.94;
                let c = CRAYONS[col[k] as usize];
                // dust over the wax
                let dust = 0.10 + 0.08 * t16_vn(seed ^ 0x55, x / 12.0, y / 12.0);
                for q in 0..3 {
                    out[i * 4 + q] = (c[q] * (1.0 - dust) + 168.0 * dust).round().clamp(0.0, 255.0) as u8;
                }
                out[i * 4 + 3] = (alpha * 255.0).round() as u8;
            }
        });
        ctx.put_image_data(&img, 0, 0);
    })
}

fn t16_hash(seed: u32, i: i32, j: i32) -> f64 {
    let mut h = seed ^ (i as u32).wrapping_mul(0x27d4_eb2d) ^ (j as u32).wrapping_mul(0x1656_67b1);
    h = (h ^ (h >> 15)).wrapping_mul(0x85eb_ca6b);
    h = (h ^ (h >> 13)).wrapping_mul(0xc2b2_ae35);
    h ^= h >> 16;
    f64::from(h) / 4_294_967_296.0
}

fn t16_vn(seed: u32, x: f64, y: f64) -> f64 {
    let (xf, yf) = (x.floor(), y.floor());
    let (i, j) = (xf as i32, yf as i32);
    let (u, v) = (x - xf, y - yf);
    let (u, v) = (u * u * (3.0 - 2.0 * u), v * v * (3.0 - 2.0 * v));
    let (a, b) = (t16_hash(seed, i, j), t16_hash(seed, i + 1, j));
    let (c, d) = (t16_hash(seed, i, j + 1), t16_hash(seed, i + 1, j + 1));
    a + (b - a) * u + (c - a) * v + (a - b - c + d) * u * v
}
