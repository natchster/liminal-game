//! The visual playtest's pixel detectors (`docs/VISUAL_PLAYTEST.md` VP2):
//! what a view's bare renders say. Two renders cleared to different colours
//! differ only where the background shows (the colour filter runs over the
//! whole frame, so the background is found by the difference, never by its
//! colour); where the background shows and a ray from the eye meets the
//! level's own walls, floors, masses or ceilings within reach, the level
//! has a hole (*gaps*). Two renders with the camera turned by a hair
//! (`JITTER`, about a tenth of a pixel), one each way, move an edge or a
//! texture too little to change a pixel by much, and a hard bright edge that
//! crosses a sample point does so on one side only; where two surfaces fight
//! for one depth the winner flips either way (*flicker*: changed in both).
//! No edge mask is used: a fight's speckle is itself all edges. Pure:
//! buffers in, masks and blobs out; the renders are `backrooms --tour`'s.
use crate::base::game::Game;
use crate::level::archgen::{Solids, BOX_VOID};

/// A pixel shows the background where the two clear colours differ by more
/// than this in some channel (0–255)
pub const BACKGROUND_DIFF: u8 = 40;
/// A view is flagged for gaps, or for flicker, at this many pixels or more
pub const FLAG_PX: usize = 12;
/// The jittered render's turn, in yaw and in pitch (radians: 0.02°)
pub const JITTER: f64 = 0.02 * std::f64::consts::PI / 180.0;
/// A pixel changed between the render and the jittered one by more than this
pub const FLICKER_DIFF: u8 = 48;
/// How far a ray from the eye is followed for the level's own surfaces
/// (metres): the far plane is 90, the fog closes far nearer
pub const GAP_REACH: f64 = 40.0;
/// The ray's step (metres)
const RAY_STEP: f64 = 0.05;
/// The level's own surfaces are taken this much inside their solids
/// (metres): a solid may stand a few centimetres proud of what is drawn of
/// it (a slab, a parapet), and a ray at a background pixel beside its edge
/// would graze that; a hole or a crack sends its rays far deeper
const RIM: f64 = 0.03;

/// A connected patch of a mask
#[derive(Clone, Debug, PartialEq)]
pub struct Blob {
    pub n: usize,
    pub x0: usize,
    pub y0: usize,
    pub x1: usize,
    pub y1: usize,
}

/// The camera a view was rendered with: the eye, its yaw and pitch (the
/// game's: forward is (-sin yaw, -cos yaw) at pitch 0), the vertical field
/// of view (radians) and the frame's size
#[derive(Clone, Copy, Debug)]
pub struct Eye {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f64,
    pub pitch: f64,
    pub fov: f64,
    pub w: usize,
    pub h: usize,
}

impl Eye {
    /// The world direction through pixel (px, py) (unit length)
    pub fn ray(&self, px: f64, py: f64) -> (f64, f64, f64) {
        let t = (self.fov / 2.0).tan();
        let a = self.w as f64 / self.h as f64;
        let cx = (2.0 * (px + 0.5) / self.w as f64 - 1.0) * t * a;
        let cy = (1.0 - 2.0 * (py + 0.5) / self.h as f64) * t;
        // camera space (x right, y up, -z forward), pitched about x, then yawed about y
        let (sp, cp) = self.pitch.sin_cos();
        let (y1, z1) = (cy * cp - (-1.0) * sp, cy * sp + (-1.0) * cp);
        let (sy, cyw) = self.yaw.sin_cos();
        let (x2, z2) = (cx * cyw + z1 * sy, -cx * sy + z1 * cyw);
        let l = (x2 * x2 + y1 * y1 + z2 * z2).sqrt();
        (x2 / l, y1 / l, z2 / l)
    }
}

/// Max channel difference of pixel `i` between two RGBA buffers
#[inline]
fn diff(a: &[u8], b: &[u8], i: usize) -> u8 {
    let o = i * 4;
    a[o].abs_diff(b[o]).max(a[o + 1].abs_diff(b[o + 1])).max(a[o + 2].abs_diff(b[o + 2]))
}

/// Where the background shows: the renders cleared to two colours differ
pub fn background_mask(a: &[u8], b: &[u8]) -> Vec<bool> {
    (0..a.len() / 4).map(|i| diff(a, b, i) > BACKGROUND_DIFF).collect()
}

/// A mask grown by `r` pixels (a square)
pub fn grow_mask(m: &[bool], w: usize, h: usize, r: usize) -> Vec<bool> {
    if r == 0 {
        return m.to_vec();
    }
    // rows, then columns
    let mut rows = vec![false; w * h];
    for y in 0..h {
        for x in 0..w {
            if m[y * w + x] {
                for xx in x.saturating_sub(r)..(x + r + 1).min(w) {
                    rows[y * w + xx] = true;
                }
            }
        }
    }
    let mut out = vec![false; w * h];
    for y in 0..h {
        for x in 0..w {
            if rows[y * w + x] {
                for yy in y.saturating_sub(r)..(y + r + 1).min(h) {
                    out[yy * w + x] = true;
                }
            }
        }
    }
    out
}

/// Flicker: pixels both jittered renders (one turned each way) changed, off
/// the background (and a pixel round it: its rim is the geometry's edge)
pub fn flicker_mask(a: &[u8], plus: &[u8], minus: &[u8], background: &[bool], w: usize, h: usize) -> Vec<bool> {
    let bg = grow_mask(background, w, h, 1);
    (0..w * h).map(|i| !bg[i] && diff(a, plus, i) > FLICKER_DIFF && diff(a, minus, i) > FLICKER_DIFF).collect()
}

/// The 4-connected patches of a mask, largest first (ties by place)
pub fn blobs(m: &[bool], w: usize, h: usize) -> Vec<Blob> {
    let mut seen = vec![false; w * h];
    let mut out = vec![];
    let mut stack = vec![];
    for s in 0..w * h {
        if !m[s] || seen[s] {
            continue;
        }
        let mut b = Blob { n: 0, x0: usize::MAX, y0: usize::MAX, x1: 0, y1: 0 };
        seen[s] = true;
        stack.push(s);
        while let Some(i) = stack.pop() {
            let (x, y) = (i % w, i / w);
            b.n += 1;
            b.x0 = b.x0.min(x);
            b.y0 = b.y0.min(y);
            b.x1 = b.x1.max(x);
            b.y1 = b.y1.max(y);
            let mut push = |j: usize| {
                if m[j] && !seen[j] {
                    seen[j] = true;
                    stack.push(j);
                }
            };
            if x > 0 {
                push(i - 1);
            }
            if x + 1 < w {
                push(i + 1);
            }
            if y > 0 {
                push(i - w);
            }
            if y + 1 < h {
                push(i + w);
            }
        }
        out.push(b);
    }
    out.sort_by(|a, b| b.n.cmp(&a.n).then((a.y0, a.x0).cmp(&(b.y0, b.x0))));
    out
}

impl Game {
    /// How far along the ray from (x, y, z) the level's own surfaces stand
    /// (a wall, a solid, a mass, a floor, a ceiling), within `reach`; None
    /// where it meets none (over the void, out of the grid, up into the open)
    pub fn ray_meets_level(&self, o: (f64, f64, f64), d: (f64, f64, f64), reach: f64) -> Option<f64> {
        let mut t = RAY_STEP;
        while t < reach {
            let (x, y, z) = (o.0 + d.0 * t, o.1 + d.1 * t, o.2 + d.2 * t);
            let (tx, tz) = (self.tx(x), self.tz(z));
            if !self.in_grid(tx, tz) {
                return None;
            }
            if self.surface_at(tx, tz, x, y, z) {
                return Some(t);
            }
            t += RAY_STEP;
        }
        None
    }

    /// Whether the point is inside the level's own surfaces: a solid that
    /// stops a look (a mass, a wall, a floor's slab, a partition; not the
    /// void's, which stops a body only); in the rooms also a wall tile, the
    /// floor and the ceiling
    fn surface_at(&self, tx: i32, tz: i32, x: f64, y: f64, z: f64) -> bool {
        if let Some(s) = &self.solids {
            if stops_a_look(s, self.to_u(x), self.to_v(z), y) {
                return true;
            }
        }
        self.ground.is_empty() && (self.is_wall_tile(tx, tz) || y < -RIM || y > self.ceil_at_world(x, z) + RIM)
    }
}

/// Whether a solid that stops a look holds the point (tile units, metres
/// up), `RIM` inside its faces
fn stops_a_look(s: &Solids, u: f64, v: f64, y: f64) -> bool {
    let r = RIM / s.cell;
    let (x, z) = (u.floor(), v.floor());
    if x < 0.0 || z < 0.0 || x >= s.w as f64 || z >= s.h as f64 {
        return false;
    }
    s.tile_items((z as i32 * s.w + x as i32) as usize).iter().any(|&b| {
        let o = s.box_at(b as usize);
        o[7] as i32 != BOX_VOID
            && u > o[0] as f64 + r
            && u < o[2] as f64 - r
            && v > o[1] as f64 + r
            && v < o[3] as f64 - r
            && y > o[4] as f64 + RIM
            && y < o[5] as f64 - RIM
    })
}

/// Gaps: background pixels (every `step`-th, both ways) whose ray meets the
/// level within `GAP_REACH`; the mask is the frame's size, each sampled
/// pixel standing for the background pixels of its step × step block
pub fn gap_mask(g: &Game, eye: &Eye, background: &[bool], step: usize) -> Vec<bool> {
    let (w, h) = (eye.w, eye.h);
    let mut m = vec![false; w * h];
    let mut y = step / 2;
    while y < h {
        let mut x = step / 2;
        while x < w {
            if background[y * w + x] && g.ray_meets_level((eye.x, eye.y, eye.z), eye.ray(x as f64, y as f64), GAP_REACH).is_some() {
                for yy in y.saturating_sub(step / 2)..(y + step - step / 2).min(h) {
                    for xx in x.saturating_sub(step / 2)..(x + step - step / 2).min(w) {
                        m[yy * w + xx] = background[yy * w + xx];
                    }
                }
            }
            x += step;
        }
        y += step;
    }
    m
}

/// A mask painted over a frame (for the report's strips): masked pixels in
/// `color`, the rest dimmed
pub fn paint(px: &[u8], m: &[bool], color: [u8; 3]) -> Vec<u8> {
    let mut out = px.to_vec();
    for (i, &on) in m.iter().enumerate() {
        let o = i * 4;
        if on {
            out[o..o + 3].copy_from_slice(&color);
        } else {
            for c in 0..3 {
                out[o + c] /= 3;
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame(w: usize, h: usize, f: impl Fn(usize, usize) -> [u8; 3]) -> Vec<u8> {
        let mut v = Vec::with_capacity(w * h * 4);
        for y in 0..h {
            for x in 0..w {
                let c = f(x, y);
                v.extend_from_slice(&[c[0], c[1], c[2], 255]);
            }
        }
        v
    }

    #[test]
    fn the_background_is_where_two_clears_differ_whatever_its_colour() {
        // a magenta wall stays a wall: only the cleared hole differs
        let hole = |x: usize, y: usize| (10..14).contains(&x) && (5..8).contains(&y);
        let a = frame(32, 16, |x, y| if hole(x, y) { [255, 0, 255] } else { [250, 5, 250] });
        let b = frame(32, 16, |x, y| if hole(x, y) { [0, 255, 0] } else { [250, 5, 250] });
        let m = background_mask(&a, &b);
        let bl = blobs(&m, 32, 16);
        assert_eq!(bl.len(), 1);
        assert_eq!((bl[0].n, bl[0].x0, bl[0].y0, bl[0].x1, bl[0].y1), (12, 10, 5, 13, 7));
    }

    #[test]
    fn flicker_is_a_fight_flipping_not_an_edge_moving() {
        let (w, h) = (48, 32);
        // a hard edge at x = 20 a tenth of a pixel over (its column a tenth
        // nearer the other side), and a fighting patch: a speckle that flips
        let speckle = |x: usize, y: usize, k: usize| (x + y + k) % 2 == 0;
        let patch = |x: usize, y: usize| (30..36).contains(&x) && (10..16).contains(&y);
        let a = frame(w, h, |x, y| {
            if patch(x, y) {
                if speckle(x, y, 0) {
                    [200, 180, 150]
                } else {
                    [90, 60, 40]
                }
            } else if x < 20 {
                [10, 10, 10]
            } else {
                [250, 240, 230]
            }
        });
        // turned one way the edge's pixel takes a sample of the other side,
        // turned the other way the pixel beside it does (a quarter: 60 levels)
        let turned = |edge_x: usize| {
            frame(w, h, move |x, y| {
                if patch(x, y) {
                    if speckle(x, y, 1) {
                        [200, 180, 150]
                    } else {
                        [90, 60, 40]
                    }
                } else if x == edge_x && edge_x < 20 {
                    [70, 67, 65]
                } else if x == edge_x {
                    [190, 182, 175]
                } else if x < 20 {
                    [10, 10, 10]
                } else {
                    [250, 240, 230]
                }
            })
        };
        let (plus, minus) = (turned(19), turned(20));
        // the background beside it is left out, and a pixel round it
        let bg: Vec<bool> = (0..w * h).map(|i| i % w >= 44).collect();
        let m = flicker_mask(&a, &plus, &minus, &bg, w, h);
        let bl = blobs(&m, w, h);
        assert_eq!(m.iter().filter(|&&b| b).count(), 36, "{bl:?}");
        assert!(bl.iter().all(|b| b.x0 >= 30 && b.x1 < 36), "{bl:?}");
        // the same render twice flickers nowhere
        assert!(!flicker_mask(&a, &a, &a, &bg, w, h).iter().any(|&b| b));
        // an edge that crosses a sample one way only is no flicker: with one
        // jitter alone it would be
        assert!(flicker_mask(&a, &plus, &plus, &bg, w, h).iter().filter(|&&b| b).count() > 36);
    }

    #[test]
    fn a_ray_through_the_middle_looks_where_the_eye_does() {
        let e = Eye { x: 0.0, y: 1.6, z: 0.0, yaw: 0.7, pitch: 0.3, fov: 1.1, w: 641, h: 401 };
        let (dx, dy, dz) = e.ray(320.0, 200.0);
        let f = (-(0.7f64).sin() * (0.3f64).cos(), (0.3f64).sin(), -(0.7f64).cos() * (0.3f64).cos());
        assert!((dx - f.0).abs() < 1e-9 && (dy - f.1).abs() < 1e-9 && (dz - f.2).abs() < 1e-9, "{:?} {:?}", (dx, dy, dz), f);
        // the top row looks up, the right column right of forward
        assert!(e.ray(320.0, 0.0).1 > dy);
        let r = e.ray(640.0, 200.0);
        let right = ((0.7f64).cos(), -(0.7f64).sin());
        assert!(r.0 * right.0 + r.2 * right.1 > 0.0);
    }
}
