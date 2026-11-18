//! The plants' shape kit: a small vector, a seeded stream, a mesh builder,
//! tubes swept along a curve (parallel-transport frames) and the leaf blade
//! every species is made of (a midrib bent by its own weight, a width
//! profile, a fold along the midrib, a curl of the margins, a twist).
//!
//! Everything is a pure function of its seed: the stream is SplitMix64 and
//! the trigonometry is the engine's correctly rounded `dmath`, so a seed is
//! the same plant on every Mac.
use engine::dmath::DMath;
use engine::prelude::*;
use std::ops::{Add, AddAssign, Mul, Neg, Sub};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct V(pub f64, pub f64, pub f64);

impl Add for V {
    type Output = V;
    fn add(self, o: V) -> V {
        V(self.0 + o.0, self.1 + o.1, self.2 + o.2)
    }
}
impl AddAssign for V {
    fn add_assign(&mut self, o: V) {
        *self = *self + o;
    }
}
impl Sub for V {
    type Output = V;
    fn sub(self, o: V) -> V {
        V(self.0 - o.0, self.1 - o.1, self.2 - o.2)
    }
}
impl Mul<f64> for V {
    type Output = V;
    fn mul(self, k: f64) -> V {
        V(self.0 * k, self.1 * k, self.2 * k)
    }
}
impl Neg for V {
    type Output = V;
    fn neg(self) -> V {
        V(-self.0, -self.1, -self.2)
    }
}

pub const UP: V = V(0.0, 1.0, 0.0);
/// The floor's height: leaves that reach it lie on it.
pub const FLOOR: f64 = 0.004;

impl V {
    pub fn dot(self, o: V) -> f64 {
        self.0 * o.0 + self.1 * o.1 + self.2 * o.2
    }
    pub fn cross(self, o: V) -> V {
        V(self.1 * o.2 - self.2 * o.1, self.2 * o.0 - self.0 * o.2, self.0 * o.1 - self.1 * o.0)
    }
    pub fn len(self) -> f64 {
        self.dot(self).sqrt()
    }
    pub fn norm(self) -> V {
        let l = self.len();
        if l < 1e-12 { V(0.0, 1.0, 0.0) } else { self * (1.0 / l) }
    }
    pub fn lerp(self, o: V, t: f64) -> V {
        self + (o - self) * t
    }
    /// Turned `a` radians about unit `axis` (Rodrigues).
    pub fn rot(self, axis: V, a: f64) -> V {
        let (s, c) = a.dsin_cos();
        self * c + axis.cross(self) * s + axis * (axis.dot(self) * (1.0 - c))
    }
    /// Any unit vector square to this one.
    pub fn perp(self) -> V {
        let t = if self.1.abs() < 0.9 { UP } else { V(1.0, 0.0, 0.0) };
        self.cross(t).norm()
    }
}

/// The rotation taking unit `a` onto unit `b`, applied to `p`.
pub fn turn_onto(p: V, a: V, b: V) -> V {
    let axis = a.cross(b);
    let s = axis.len();
    if s < 1e-12 {
        return if a.dot(b) > 0.0 { p } else { p.rot(a.perp(), PI) };
    }
    p.rot(axis * (1.0 / s), s.datan2(a.dot(b)))
}

/// A seeded stream (SplitMix64): `fork` gives a part its own stream, so
/// adding a draw to one leaf does not move every leaf after it.
#[derive(Clone, Debug)]
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Rng {
        Rng(seed ^ 0x9e37_79b9_7f4a_7c15)
    }
    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }
    /// In [0, 1).
    pub fn f(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }
    pub fn range(&mut self, a: f64, b: f64) -> f64 {
        a + (b - a) * self.f()
    }
    /// A normal deviate (Box–Muller).
    pub fn normal(&mut self) -> f64 {
        let u = self.f().max(1e-12);
        let v = self.f();
        (-2.0 * u.dln()).sqrt() * (2.0 * PI * v).dcos()
    }
    pub fn chance(&mut self, p: f64) -> bool {
        self.f() < p
    }
    pub fn fork(&mut self, k: u64) -> Rng {
        Rng(self.next_u64() ^ k.wrapping_mul(0x2545_f491_4f6c_dd1d))
    }
}

/// sRGB (0..1) to linear: vertex colours are linear in the renderer.
pub fn lin(c: [f64; 3]) -> [f64; 3] {
    c.map(|x| if x <= 0.04045 { x / 12.92 } else { ((x + 0.055) / 1.055).dpowf(2.4) })
}

pub fn mix(a: [f64; 3], b: [f64; 3], t: f64) -> [f64; 3] {
    let t = t.clamp(0.0, 1.0);
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
}

pub fn smooth(e0: f64, e1: f64, x: f64) -> f64 {
    // equal edges are a step (and never 0 / 0)
    if (e1 - e0).abs() < 1e-12 {
        return if (x >= e0) == (e1 >= e0) { 1.0 } else { 0.0 };
    }
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// A value noise on one axis, smooth, from a seed: for bands and blotches.
pub fn noise1(seed: u64, x: f64) -> f64 {
    let h = |i: i64| {
        let mut r = Rng::new(seed ^ (i as u64).wrapping_mul(0x9e37_79b9));
        r.f() * 2.0 - 1.0
    };
    let i = x.floor();
    let t = x - i;
    let s = t * t * (3.0 - 2.0 * t);
    h(i as i64) * (1.0 - s) + h(i as i64 + 1) * s
}

/// Triangles with a colour at each corner (sRGB in, linear stored).
#[derive(Clone, Default)]
pub struct MeshBuf {
    pub pos: Vec<f32>,
    pub col: Vec<f32>,
    pub idx: Vec<u32>,
}

impl MeshBuf {
    pub fn vert(&mut self, p: V, c: [f64; 3]) -> u32 {
        let n = (self.pos.len() / 3) as u32;
        self.pos.extend([p.0 as f32, p.1 as f32, p.2 as f32]);
        self.col.extend(lin(c).map(|x| x as f32));
        n
    }
    pub fn tri(&mut self, a: u32, b: u32, c: u32) {
        self.idx.extend([a, b, c]);
    }
    pub fn quad(&mut self, a: u32, b: u32, c: u32, d: u32) {
        self.tri(a, b, c);
        self.tri(a, c, d);
    }
    pub fn triangles(&self) -> usize {
        self.idx.len() / 3
    }
    pub fn extend(&mut self, o: &MeshBuf) {
        let base = (self.pos.len() / 3) as u32;
        self.pos.extend_from_slice(&o.pos);
        self.col.extend_from_slice(&o.col);
        self.idx.extend(o.idx.iter().map(|i| i + base));
    }
    pub fn to_geo(&self) -> Geo {
        let g = Geo::new();
        g.set_position(self.pos.clone());
        g.set_color(self.col.clone(), 3);
        g.set_index(self.idx.clone());
        g.compute_vertex_normals();
        g
    }
}

/// A tube along `pts` with radius `radii[i]` at each, `sides` round, its
/// colour from (along 0..1, around 0..1); `cap` closes the far end.
pub fn tube(mb: &mut MeshBuf, pts: &[V], radii: &[f64], sides: usize, cap: bool, color: &dyn Fn(f64, f64) -> [f64; 3]) {
    let n = pts.len();
    if n < 2 {
        return;
    }
    let tangent = |i: usize| -> V {
        let a = pts[i.saturating_sub(1)];
        let b = pts[(i + 1).min(n - 1)];
        (b - a).norm()
    };
    let mut t_prev = tangent(0);
    let mut side = t_prev.perp();
    let mut rings: Vec<u32> = Vec::with_capacity(n);
    for i in 0..n {
        let t = tangent(i);
        // parallel transport: the side turns only as much as the tangent does
        side = turn_onto(side, t_prev, t);
        side = (side - t * side.dot(t)).norm();
        t_prev = t;
        let up = t.cross(side);
        let along = i as f64 / (n - 1) as f64;
        let first = (mb.pos.len() / 3) as u32;
        for k in 0..sides {
            let a = 2.0 * PI * k as f64 / sides as f64;
            let (s, c) = a.dsin_cos();
            let p = pts[i] + side * (c * radii[i]) + up * (s * radii[i]);
            mb.vert(p, color(along, k as f64 / sides as f64));
        }
        rings.push(first);
    }
    for i in 0..n - 1 {
        for k in 0..sides {
            let k1 = (k + 1) % sides;
            let (a, b) = (rings[i] + k as u32, rings[i] + k1 as u32);
            let (c, d) = (rings[i + 1] + k1 as u32, rings[i + 1] + k as u32);
            mb.quad(a, d, c, b);
        }
    }
    if cap {
        let tip = mb.vert(pts[n - 1] + tangent(n - 1) * radii[n - 1], color(1.0, 0.0));
        for k in 0..sides {
            let k1 = (k + 1) % sides;
            mb.tri(rings[n - 1] + k as u32, rings[n - 1] + k1 as u32, tip);
        }
    }
}

/// A leaf blade: its midrib starts at `base` along `dir`, its face toward
/// `face` (made square to the midrib), and bends under its own weight.
#[derive(Clone)]
pub struct Blade {
    pub base: V,
    pub dir: V,
    pub face: V,
    pub length: f64,
    /// half-width at `u` (0 base .. 1 tip), metres
    pub width: fn(f64, f64) -> f64,
    /// the widest half-width, passed to `width`
    pub w: f64,
    pub segs_u: usize,
    /// points across each half (1 = a flat strip edge to edge)
    pub segs_v: usize,
    /// bending toward the ground per metre, at full lever (radians/m)
    pub droop: f64,
    /// the leaf folded along its midrib: each half lifted by this (radians)
    pub fold: f64,
    /// the margins rolled toward the face (metres of lift at the margin)
    pub curl: f64,
    /// turned about the midrib from base to tip (radians)
    pub twist: f64,
    /// ruffle of the margin: amplitude (m) and waves along the leaf
    pub wave: (f64, f64),
    /// the midrib folds over here: (u, radians), the dying leaf's kink
    pub kink: Option<(f64, f64)>,
}

impl Blade {
    pub fn new(base: V, dir: V, face: V, length: f64, w: f64, width: fn(f64, f64) -> f64) -> Blade {
        Blade {
            base,
            dir: dir.norm(),
            face,
            length,
            width,
            w,
            segs_u: 8,
            segs_v: 1,
            droop: 0.0,
            fold: 0.0,
            curl: 0.0,
            twist: 0.0,
            wave: (0.0, 0.0),
            kink: None,
        }
    }

    /// The midrib: a point, its tangent and its side (to the leaf's left) at
    /// each of `segs_u + 1` steps.
    pub fn midrib(&self) -> Vec<(V, V, V)> {
        let n = self.segs_u.max(1);
        let ds = self.length / n as f64;
        let mut p = self.base;
        let mut d = self.dir;
        let mut side = self.dir.cross(self.face).norm();
        if side.len() < 0.5 {
            side = d.perp();
        }
        let mut out = Vec::with_capacity(n + 1);
        out.push((p, d, side));
        for i in 0..n {
            let u = (i as f64 + 0.5) / n as f64;
            // the weight beyond this point bends it, by its lever
            let lever = (1.0 - u).dpowf(1.5);
            let down = V(0.0, -1.0, 0.0);
            let axis = d.cross(down);
            let s = axis.len();
            if s > 1e-6 {
                let a = (self.droop * ds * lever * s).min(0.5);
                let ax = axis * (1.0 / s);
                d = d.rot(ax, a);
                side = side.rot(ax, a);
            }
            if let Some((ku, ka)) = self.kink {
                let (u0, u1) = (i as f64 / n as f64, (i + 1) as f64 / n as f64);
                if ku >= u0 && ku < u1 {
                    // folds over toward the ground, about its own side
                    let ax = if d.cross(down).len() > 1e-6 { d.cross(down).norm() } else { side };
                    d = d.rot(ax, ka);
                    side = side.rot(ax, ka);
                }
            }
            let tw = self.twist / n as f64;
            side = side.rot(d, tw);
            side = (side - d * side.dot(d)).norm();
            p += d * ds;
            // what reaches the floor lies along it
            if p.1 < FLOOR && d.1 < 0.0 {
                p.1 = FLOOR;
                let flat = V(d.0, 0.0, d.2);
                if flat.len() > 1e-6 {
                    let f = flat.norm();
                    side = turn_onto(side, d, f);
                    d = f;
                }
            }
            out.push((p, d, side));
        }
        out
    }

    /// Into `mb`, its colour from (u along, v across −1..1).
    pub fn build(&self, mb: &mut MeshBuf, color: &dyn Fn(f64, f64) -> [f64; 3]) {
        let rib = self.midrib();
        let n = rib.len() - 1;
        let m = self.segs_v.max(1);
        let cols = 2 * m + 1;
        let first = (mb.pos.len() / 3) as u32;
        for (i, &(p, d, side)) in rib.iter().enumerate() {
            let u = i as f64 / n as f64;
            let face = d.cross(side).norm();
            let hw = (self.width)(u, self.w);
            for j in 0..cols {
                let v = (j as f64 - m as f64) / m as f64;
                let a = v.abs();
                let lift = hw * a * self.fold.dsin()
                    + self.curl * a * a
                    + self.wave.0 * a * a * (u * self.wave.1 * 2.0 * PI + v.signum() * 1.3).dsin();
                let q = p + side * (hw * v * self.fold.dcos()) + face * lift;
                mb.vert(q, color(u, v));
            }
        }
        for i in 0..n {
            for j in 0..cols - 1 {
                let a = first + (i * cols + j) as u32;
                let b = a + 1;
                let c = a + cols as u32 + 1;
                let d = a + cols as u32;
                mb.quad(a, b, c, d);
            }
        }
    }
}

/// A body of revolution: `profile` is (radius, height) from the bottom up,
/// coloured by (height share 0..1, angle share 0..1).
pub fn lathe(mb: &mut MeshBuf, at: V, profile: &[(f64, f64)], sides: usize, color: &dyn Fn(f64, f64) -> [f64; 3]) {
    let first = (mb.pos.len() / 3) as u32;
    let rows = profile.len();
    for (i, &(r, y)) in profile.iter().enumerate() {
        for k in 0..sides {
            let a = 2.0 * PI * k as f64 / sides as f64;
            let (s, c) = a.dsin_cos();
            mb.vert(at + V(c * r, y, s * r), color(i as f64 / (rows - 1).max(1) as f64, k as f64 / sides as f64));
        }
    }
    for i in 0..rows - 1 {
        for k in 0..sides {
            let k1 = (k + 1) % sides;
            let a = first + (i * sides + k) as u32;
            let b = first + (i * sides + k1) as u32;
            let c = first + ((i + 1) * sides + k1) as u32;
            let d = first + ((i + 1) * sides + k) as u32;
            mb.quad(a, d, c, b);
        }
    }
}

/// A disc (a fan) at height `y` over `at`, radius `r`, its height and
/// colour at (radius share, angle) from `f`: soil.
pub fn disc(mb: &mut MeshBuf, at: V, r: f64, rings: usize, sides: usize, f: &dyn Fn(f64, f64) -> (f64, [f64; 3])) {
    let (h0, c0) = f(0.0, 0.0);
    let centre = mb.vert(at + V(0.0, h0, 0.0), c0);
    let first = (mb.pos.len() / 3) as u32;
    for i in 1..=rings {
        let rr = i as f64 / rings as f64;
        for k in 0..sides {
            let a = 2.0 * PI * k as f64 / sides as f64;
            let (s, c) = a.dsin_cos();
            let (h, col) = f(rr, a);
            mb.vert(at + V(c * r * rr, h, s * r * rr), col);
        }
    }
    for k in 0..sides {
        let k1 = (k + 1) % sides;
        mb.tri(centre, first + k1 as u32, first + k as u32);
    }
    for i in 0..rings - 1 {
        for k in 0..sides {
            let k1 = (k + 1) % sides;
            let a = first + (i * sides + k) as u32;
            let b = first + (i * sides + k1) as u32;
            let c = first + ((i + 1) * sides + k1) as u32;
            let d = first + ((i + 1) * sides + k) as u32;
            mb.quad(a, b, c, d);
        }
    }
}
