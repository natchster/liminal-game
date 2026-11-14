//! The windows of the rooms (`docs/design/WINDOWS.md`): real windows cut into real
//! walls, onto an outside the rooms do not have.
//!
//! A row of windows is chosen in the level's build (`place_windows`, a few
//! draws from its stream, where an office, a hall or a home would have
//! them) and finished once the props have settled and the solids are final
//! (`Game::windows_settled`, no draws): its wall's real face and depth read
//! from the solids, its opening cut out of what `build_walls` draws (the
//! solids, what you walk into, stay whole: the glass is solid), and the
//! window built in it (`model`) with what it looks out on (`view`). Its wear
//! comes from a stream of its own.
//!
//! A window's frame: x across the opening (to your right as you face it from
//! the room), y up from the floor, z out of the wall into the room from the
//! back of the cut (z = 0: where the outside begins).
pub mod blind;
pub mod model;
pub mod view;

use super::{Home, PropsCtx, home_zones};
use crate::base::rng::SplitMix64;
use crate::level::archgen::{DIRS, Solids, ray_axis};
use crate::prelude::*;

/// Where a real building would have windows: offices and office wings,
/// halls, living quarters, parlours, party rooms; never a zone left bare
pub const WINDOW_HOME: Home =
    Home { arch: &["office", "hall"], themes: &["office", "domestic", "victorian", "party"], elsewhere: 0.0, skip_bare: true };

/// The salts of a window's own streams: its row's plan, and its wear and blinds
const PLAN_SALT: u64 = 0x57_1D_0E_9A_17;
const WIN_SALT: u64 = 0x57_1D_0E_5A_11;

/// What is left of the wall behind a window's cut (the far room sees an
/// unbroken wall), m; and how deep a cut goes into a wall that fills its tile
pub const SLIVER: f64 = 0.012;
pub const DEEP_REVEAL: f64 = 0.30;
/// The lamp outside is heard through a window's glass within this, m
pub const HUM_REACH: f64 = 7.0;
/// How far in front of the view what is beyond the glass is drawn, at most
/// (`Material::portal`'s slab): behind the outermost glass and bars, m
pub const SLAB: f64 = 0.018;

/// The two kinds of window (`docs/design/WINDOWS.md` 1)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// fixed lights between aluminium mullions over convectors (offices)
    Strip,
    /// wood double-hung sash windows (halls and homes)
    DoubleHung,
}

/// A row chosen in the build: the open tile before its wall, the step from
/// that tile to the wall, how many lights or windows, its kind, its stream
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Plan {
    pub x: i32,
    pub z: i32,
    pub dx: i32,
    pub dz: i32,
    pub n: usize,
    pub kind: Kind,
    pub seed: u32,
}

/// How a light's blind hangs
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Blind {
    /// how far down it is let (0 raised, its slats stacked at the head; 1 down)
    pub down: f64,
    /// how open its slats are (0 shut, 1 flat open)
    pub open: f64,
    /// how much its bottom rail tilts (one cord let out), m across its width
    pub skew: f64,
}

/// One light of a window: its glass, the opening its light comes in by, and
/// the lit floor it throws
pub struct Light {
    /// the daylight opening (window frame): x0, x1, y0, y1, and the glass's z
    pub rect: [f64; 4],
    pub glass_z: f64,
    /// its glass (a `Patch::Glass` material of its own)
    pub glass: Obj,
    /// its blind, if it has one
    pub blind: Option<Blind>,
    /// paper taped over it from inside (someone covered it)
    pub papered: bool,
    /// a muntin down its middle (a two-light sash)
    pub muntin: bool,
    /// the floor its lamp lights through it: the lit cells (world x, z
    /// corners, in order round each), and that light drawn on the floor
    pub patch: Vec<[(f64, f64); 4]>,
    pub glow: Option<Obj>,
}

/// A blind as it hangs (its own, so it can be let down and drawn up): the
/// lights it covers, where it hangs (window frame: x0, x1, its head's and
/// the sill's height, z), what it is drawn with, and its slats on their way
pub struct Hang {
    pub lights: Vec<usize>,
    pub at: [f64; 5],
    pub obj: Obj,
    pub mats: (Mat, Mat),
    /// let down or drawn up: from, to, and when it began (game time)
    pub moving: Option<(Blind, Blind, f64)>,
}

/// A row of windows (or the lights of one strip window) on one wall
pub struct Window {
    pub kind: Kind,
    /// at the back of the cut, the span's middle, the floor; turned so its
    /// +z goes into the room
    pub root: Obj,
    /// the way into the room, and the way to the right seen from it (world)
    pub normal: Vec3,
    pub right: Vec3,
    /// the cut's depth (the reveal), the opening's sill and head (m)
    pub depth: f64,
    pub sill: f64,
    pub head: f64,
    pub lights: Vec<Light>,
    /// its blinds, each its own
    pub blinds: Vec<Hang>,
    /// what is cut out of the drawn walls (world boxes x0 y0 z0 x1 y1 z1)
    pub cuts: Vec<[f64; 6]>,
    /// its view's material (one for the row) and what is drawn beyond it
    pub view: Mat,
    pub portal: std::rc::Rc<std::cell::RefCell<PortalU>>,
    /// the lamp's head outside (world), the lamp itself, and how much
    /// lower than a streetlamp's it hangs (what the one behind its glass
    /// learns), m
    pub lamp: Vec3,
    pub lamp_obj: Option<Obj>,
    pub lamp_drop: f64,
    /// its lamp's ballast heard through the glass (made when first heard)
    pub hum: Option<crate::audio::soundscape::MachineHum>,
    /// its own stream's seed
    pub seed: u32,
}

impl Window {
    /// A point of the window's frame, in the world
    pub fn world(&self, x: f64, y: f64, z: f64) -> Vec3 {
        let o = self.root.pos();
        Vec3::new(o.x + self.right.x * x + self.normal.x * z, o.y + y, o.z + self.right.z * x + self.normal.z * z)
    }
    /// A world point in the window's frame
    pub fn local(&self, p: &Vec3) -> Vec3 {
        let o = self.root.pos();
        let (dx, dz) = (p.x - o.x, p.z - o.z);
        Vec3::new(dx * self.right.x + dz * self.right.z, p.y - o.y, dx * self.normal.x + dz * self.normal.z)
    }
    /// The middle of light `k`'s glass, in the world
    pub fn light_centre(&self, k: usize) -> Vec3 {
        let l = &self.lights[k];
        self.world((l.rect[0] + l.rect[1]) / 2.0, (l.rect[2] + l.rect[3]) / 2.0, l.glass_z)
    }
    /// Whether the floor at (x, z) is in the light of any of its lights (a
    /// covered light throws none)
    pub fn lit_at(&self, x: f64, z: f64) -> bool {
        (0..self.lights.len()).any(|k| !self.covered(k) && self.lights[k].patch.iter().any(|q| in_quad(q, x, z)))
    }
    /// The floor light `k` lights now (none while it is covered)
    pub fn lit_cells(&self, k: usize) -> &[[(f64, f64); 4]] {
        if self.covered(k) { &[] } else { &self.lights[k].patch }
    }
    /// Whether light `k` is covered: its blind down and shut, or papered over
    pub fn covered(&self, k: usize) -> bool {
        let l = &self.lights[k];
        l.papered || l.blind.map(|b| b.down > 0.9 && b.open < 0.2).unwrap_or(false)
    }
}

/// Whether this floor has windows: the rooms below the Lobby (level 5 on),
/// never the Showroom or the Borrowed House (their light is their own)
pub fn windows_here(g: &Game) -> bool {
    g.building.is_none() && g.level_num >= 5 && !matches!(g.level_scp.as_deref(), Some("store" | "house")) && !g.in_loop && !g.in_stairwell
}

/// Choose this floor's rows of windows where the old window zone took its
/// tiles (in the core props' tail): three tiles beside a wall from the
/// shuffled pool and a wall beside each (one draw each), taken as they were
/// on every floor of the rooms, so nothing placed after them moves. A row is
/// planned on one only on a floor that has windows and in a zone a building
/// would have them in; how many lights, and its own stream's seed, come from
/// a stream of its own. Nothing is built here: `windows_settled` finishes
/// them on the final walls.
pub fn place_windows(g: &mut Game, cx: &mut PropsCtx) {
    let here = windows_here(g);
    let zones = home_zones(g, &WINDOW_HOME);
    for _ in 0..3 {
        let Some(c) = super::seam::s_take(g, cx, true) else { break };
        let Some(spot) = super::seam::s_wall_adj_spot(g, c.x, c.z) else { continue };
        let home = zones.as_ref().zip(g.layout.as_ref()).map(|(z, l)| z[l.zone_of[g.t_idx(c.x, c.z)] as usize]).unwrap_or(false);
        if !here || !home {
            continue;
        }
        let mut r =
            SplitMix64::new(g.current_seed ^ (c.x as u32).wrapping_mul(0x9E37_79B1) ^ (c.z as u32).wrapping_mul(0x85EB_CA77), PLAN_SALT);
        let n = 1 + ((r.next() * 3.0) as usize).min(2);
        let seed = (r.next() * u32::MAX as f64) as u32;
        let (dx, dz) = (-spot.normal.x.round() as i32, -spot.normal.z.round() as i32);
        let office = g
            .layout
            .as_ref()
            .zip(g.zone_ref(c.x, c.z))
            .map(|(l, zn)| {
                zn.arch == "office" || g.zone_purposes.get(l.zone_of[g.t_idx(c.x, c.z)] as usize).is_some_and(|p| p.has_any(&["office"]))
            })
            .unwrap_or(false);
        let kind = if office { Kind::Strip } else { Kind::DoubleHung };
        g.window_plans.push(Plan { x: c.x, z: c.z, dx, dz, n, kind, seed });
    }
}

/// What a row of windows was fitted to: its wall's face, the cut's depth,
/// and the span's middle along the wall (world)
#[derive(Clone, Copy, Debug)]
struct Fit {
    /// the face at the span's middle, at the floor (world)
    face: Vec3,
    depth: f64,
    sill: f64,
    head: f64,
    n: usize,
}

/// Whether the point (u, v) (tile units) at height y is inside a solid
pub(crate) fn solid_at(s: &Solids, u: f64, v: f64, y: f64) -> bool {
    if u < 0.0 || v < 0.0 || u >= s.w as f64 || v >= s.h as f64 {
        return true;
    }
    let t = (v.floor() as i32 * s.w + u.floor() as i32) as usize;
    s.tile_items(t).iter().any(|&b| {
        let x = s.box_at(b as usize);
        let kind = x[7] as i32;
        if kind == crate::level::archgen::BOX_VOID || kind == crate::level::archgen::BOX_FLOOR {
            return false;
        }
        u >= x[0] as f64 && u <= x[2] as f64 && v >= x[1] as f64 && v <= x[3] as f64 && y >= x[4] as f64 && y <= x[5] as f64
    })
}

/// The width of a row of `n` and how far apart its openings are (centre to
/// centre), with its trim either side
fn span_of(kind: Kind, n: usize) -> (f64, f64) {
    match kind {
        Kind::Strip => (n as f64 * model::ST_W + model::MULL + 0.06, 0.0),
        Kind::DoubleHung => ((n - 1) as f64 * model::DH_PITCH + model::DH_W + 2.0 * (model::CASE_REVEAL + model::CASE_W), model::DH_PITCH),
    }
}

impl Game {
    /// Fit plan `p` to the final walls, as many of its lights as fit (no draws)
    fn window_fit(&self, p: &Plan, taken: &[[f64; 6]]) -> Option<Fit> {
        let s = self.solids.as_ref()?;
        let dir = DIRS.iter().position(|d| d[0] == p.dx && d[1] == p.dz)?;
        let (sill, head) = match p.kind {
            Kind::Strip => (model::ST_SILL, model::ST_SILL + model::ST_H + 2.0 * model::MULL),
            Kind::DoubleHung => (model::DH_SILL, model::DH_SILL + model::DH_H),
        };
        let ceil = self.tile_ceil.get(self.t_idx(p.x, p.z)).copied().unwrap_or(0.0) as f64;
        if head + 0.12 > ceil {
            return None;
        }
        let (fx, fz) = (p.dx as f64, p.dz as f64);
        // along the wall (tile units), and the open tile's middle
        let (ax, az) = (fz.abs(), fx.abs());
        let (uc, vc) = (p.x as f64 + 0.5, p.z as f64 + 0.5);
        let t0 = ray_axis(s, uc, vc, dir, 2.0, (sill + head) / 2.0);
        if !(0.05..1.6).contains(&t0) {
            return None;
        }
        'n: for n in (1..=p.n).rev() {
            let (span, _) = span_of(p.kind, n);
            let half = span / 2.0 / CELL;
            // the face over the whole span and half a metre past it either
            // side (a window wants a wall round it: never a pillar's face, nor
            // a block standing out of its wall), at the sill, the middle, the head
            let margin = 0.5 / CELL;
            let mut a = -half - margin;
            while a <= half + margin {
                for y in [sill + 0.04, (sill + head) / 2.0, head - 0.04] {
                    let (u, v) = (uc + ax * a, vc + az * a);
                    if solid_at(s, u, v, y) {
                        continue 'n;
                    }
                    let t = ray_axis(s, u, v, dir, 2.0, y);
                    if (t - t0).abs() > 0.004 {
                        continue 'n;
                    }
                    // never a partition that leans
                    let (wu, wv) = (u + fx * (t + 0.01), v + fz * (t + 0.01));
                    let wt = self.t_idx(wu.floor() as i32, wv.floor() as i32);
                    if self.angled.iter().any(|w| w.owns(wt)) {
                        continue 'n;
                    }
                    // open floor before it
                    if solid_at(s, u + fx * (t - 0.15), v + fz * (t - 0.15), 1.0) {
                        continue 'n;
                    }
                }
                a += 0.06;
            }
            // the wall's thickness (its least over the span)
            let mut thick = f64::INFINITY;
            for a in [-half, 0.0, half] {
                let (u, v) = (uc + ax * a + fx * t0, vc + az * a + fz * t0);
                let mut d = 0.0;
                while d < 0.8 && solid_at(s, u + fx * (d + 0.005) / CELL, v + fz * (d + 0.005) / CELL, (sill + head) / 2.0) {
                    d += 0.005;
                }
                thick = thick.min(d);
            }
            if thick < 0.10 {
                return None;
            }
            let depth = if thick < 0.75 { thick - SLIVER } else { DEEP_REVEAL };
            let face = Vec3::new(self.to_world_u(uc + fx * t0), 0.0, self.to_world_v(vc + fz * t0));
            let fit = Fit { face, depth, sill, head, n };
            // clear of every other window's cut (and the wall's far side)
            let me = self.fit_cuts(p, &fit);
            let grow = |b: &[f64; 6], m: f64| [b[0] - m, b[1] - m, b[2] - m, b[3] + m, b[4] + m, b[5] + m];
            if me.iter().any(|c| taken.iter().any(|o| boxes_meet(&grow(c, 0.3), o))) {
                continue 'n;
            }
            // and of whatever already hangs on that wall or stands before it
            if !self.window_room(p, &fit) {
                continue 'n;
            }
            return Some(fit);
        }
        None
    }

    /// Tile u / v (fractional) to world x / z
    fn to_world_u(&self, u: f64) -> f64 {
        (u - self.w as f64 / 2.0) * CELL
    }
    fn to_world_v(&self, v: f64) -> f64 {
        (v - self.h as f64 / 2.0) * CELL
    }

    /// The boxes a fitted row cuts out of the drawn walls (world)
    fn fit_cuts(&self, p: &Plan, f: &Fit) -> Vec<[f64; 6]> {
        let (fx, fz) = (p.dx as f64, p.dz as f64);
        let (rx, rz) = (-fz, fx);
        let mut out = vec![];
        let openings: Vec<(f64, f64)> = match p.kind {
            Kind::Strip => {
                let w = f.n as f64 * model::ST_W + model::MULL;
                vec![(-w / 2.0, w / 2.0)]
            }
            Kind::DoubleHung => (0..f.n)
                .map(|i| {
                    let cx = (i as f64 - (f.n - 1) as f64 / 2.0) * model::DH_PITCH;
                    (cx - model::DH_W / 2.0, cx + model::DH_W / 2.0)
                })
                .collect(),
        };
        for (x0, x1) in openings {
            // from just in front of the face to the cut's back
            let a = Vec3::new(f.face.x + rx * x0 - fx * 0.002, f.sill, f.face.z + rz * x0 - fz * 0.002);
            let b = Vec3::new(f.face.x + rx * x1 + fx * f.depth, f.head, f.face.z + rz * x1 + fz * f.depth);
            out.push([a.x.min(b.x), f.sill, a.z.min(b.z), a.x.max(b.x), f.head, a.z.max(b.z)]);
        }
        out
    }

    /// Whether nothing already hangs on the wall where a row would go, or
    /// stands where its trim, stool and convectors would (props placed before
    /// it, the vending machines)
    fn window_room(&self, p: &Plan, f: &Fit) -> bool {
        let (fx, fz) = (p.dx as f64, p.dz as f64);
        let (rx, rz) = (-fz, fx);
        let (span, _) = span_of(p.kind, f.n);
        let reach = if p.kind == Kind::Strip { 0.27 } else { 0.08 };
        let y0 = if p.kind == Kind::Strip { 0.02 } else { f.sill - 0.12 };
        let corners = [
            (f.face.x + rx * span / 2.0 + fx * 0.05, f.face.z + rz * span / 2.0 + fz * 0.05),
            (f.face.x - rx * span / 2.0 - fx * reach, f.face.z - rz * span / 2.0 - fz * reach),
        ];
        let zone = Box3::new(
            Vec3::new(corners[0].0.min(corners[1].0), y0, corners[0].1.min(corners[1].1)),
            Vec3::new(corners[0].0.max(corners[1].0), f.head + 0.1, corners[0].1.max(corners[1].1)),
        );
        for o in self.world.children() {
            if o.is_instanced() || o.is_light() || o.r().user.arch {
                continue;
            }
            let mut b = Box3::default();
            b.set_from_object(&o);
            if !b.is_empty() && b.intersects_box(&zone) {
                return false;
            }
        }
        let mid = Vec3::new(f.face.x - fx * 0.4, 0.0, f.face.z - fz * 0.4);
        !self.hits_vending(mid.x, mid.z, span / 2.0)
    }

    /// Finish the rows chosen in the build on the final walls (after the
    /// props settle, before the walls are drawn): fit each, cut its
    /// openings, build it and its view. No draws: what it is made of comes
    /// from its own stream.
    pub fn windows_settled(&mut self) {
        let plans = std::mem::take(&mut self.window_plans);
        let mut taken: Vec<[f64; 6]> = vec![];
        for p in &plans {
            let Some(fit) = self.window_fit(p, &taken) else { continue };
            let cuts = self.fit_cuts(p, &fit);
            taken.extend(cuts.iter().copied());
            let mut r = SplitMix64::new(p.seed, WIN_SALT);
            let w = model::build(self, p, &fit_to_frame(p, &fit), cuts, &mut r);
            self.windows.push(w);
        }
        self.window_plans = plans;
        for i in 0..self.windows.len() {
            view::finish(self, i);
            self.window_blinds_register(i);
        }
        if !self.windows.is_empty() {
            self.animate(|g, dt, _t| g.windows_frame(dt));
        }
    }

    /// Per frame: blinds on their way (`blind`), and each window's lamp
    /// heard through its glass, near it (made the first time it is near,
    /// silent and far it is nothing)
    pub fn windows_frame(&mut self, dt: f64) {
        self.blinds_frame();
        let pp = self.player.pos;
        let hush = self.amb_hush.clamp(0.0, 1.0);
        for i in 0..self.windows.len() {
            let w = &self.windows[i];
            let k = w.lights.len() / 2;
            let at = w.light_centre(k);
            let d = crate::base::numeric::hypot2(at.x - pp.x, at.z - pp.z);
            let open = (0..w.lights.len()).any(|k| !w.covered(k));
            // (in its room: round the other side of its wall it is a wall)
            let front = w.world(w.local(&at).x, 1.0, w.depth + 0.1);
            let level = if d < HUM_REACH && open && self.los(pp.x, pp.z, front.x, front.z) { 1.0 - 0.75 * hush } else { 0.0 };
            if w.hum.is_none() {
                if level <= 0.0 {
                    continue;
                }
                let Some(ss) = self.soundscape() else { continue };
                self.windows[i].hum = Some(ss.lamp_hum());
            }
            if let Some(h) = &self.windows[i].hum {
                h.update(dt, at.x, at.y + 0.3, at.z, level);
            }
        }
    }

    /// Every opening cut out of the drawn walls (`build_walls`)
    pub fn window_cut_boxes(&self) -> Vec<[f64; 6]> {
        self.windows.iter().flat_map(|w| w.cuts.iter().copied()).collect()
    }
}

/// Whether (x, z) is inside the convex quad `q` (either winding)
pub fn in_quad(q: &[(f64, f64); 4], x: f64, z: f64) -> bool {
    let mut sign = 0.0;
    for i in 0..4 {
        let (a, b) = (q[i], q[(i + 1) % 4]);
        let c = (b.0 - a.0) * (z - a.1) - (b.1 - a.1) * (x - a.0);
        if c.abs() < 1e-12 {
            continue;
        }
        if sign == 0.0 {
            sign = c.signum();
        } else if c.signum() != sign {
            return false;
        }
    }
    true
}

/// Whether two world boxes overlap
fn boxes_meet(a: &[f64; 6], b: &[f64; 6]) -> bool {
    (0..3).all(|k| a[k] < b[k + 3] && b[k] < a[k + 3])
}

/// A fitted row as `model` builds it
pub struct Frame {
    /// the back of the cut at the span's middle, at the floor (world)
    pub origin: Vec3,
    pub normal: Vec3,
    pub right: Vec3,
    pub yaw: f64,
    pub depth: f64,
    pub sill: f64,
    pub head: f64,
    pub n: usize,
}

fn fit_to_frame(p: &Plan, f: &Fit) -> Frame {
    let (fx, fz) = (p.dx as f64, p.dz as f64);
    let normal = Vec3::new(-fx, 0.0, -fz);
    let right = Vec3::new(-fz, 0.0, fx);
    Frame {
        origin: Vec3::new(f.face.x + fx * f.depth, 0.0, f.face.z + fz * f.depth),
        normal,
        right,
        yaw: normal.x.atan2(normal.z),
        depth: f.depth,
        sill: f.sill,
        head: f.head,
        n: f.n,
    }
}

/// `a` less `c` (world boxes `[x0 y0 z0 x1 y1 z1]`): up to six boxes
pub fn box_minus(a: &[f64; 6], c: &[f64; 6]) -> Vec<[f64; 6]> {
    if !boxes_meet(a, c) {
        return vec![*a];
    }
    let mut out = vec![];
    let mut rest = *a;
    for k in 0..3 {
        if c[k] > rest[k] {
            let mut lo = rest;
            lo[k + 3] = c[k];
            out.push(lo);
            rest[k] = c[k];
        }
        if c[k + 3] < rest[k + 3] {
            let mut hi = rest;
            hi[k] = c[k + 3];
            out.push(hi);
            rest[k + 3] = c[k + 3];
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_box_less_a_hole_through_it_is_four_boxes_round_the_hole() {
        let wall = [0.0, 0.0, 0.0, 4.0, 3.0, 0.2];
        let hole = [1.0, 0.8, -0.1, 2.0, 2.2, 0.3];
        let parts = box_minus(&wall, &hole);
        assert_eq!(parts.len(), 4);
        let vol = |b: &[f64; 6]| (b[3] - b[0]) * (b[4] - b[1]) * (b[5] - b[2]);
        let total: f64 = parts.iter().map(vol).sum();
        assert!((total - (vol(&wall) - 1.0 * 1.4 * 0.2)).abs() < 1e-9, "{total}");
        // a hole short of the back leaves the back whole
        let niche = [1.0, 0.8, 0.012, 2.0, 2.2, 0.3];
        let parts = box_minus(&wall, &niche);
        assert_eq!(parts.len(), 5);
        assert!(parts.iter().any(|b| b[2] == 0.0 && b[5] == 0.012 && b[0] == 1.0 && b[3] == 2.0 && b[1] == 0.8 && b[4] == 2.2));
        let total: f64 = parts.iter().map(vol).sum();
        assert!((total - (vol(&wall) - 1.0 * 1.4 * 0.188)).abs() < 1e-9, "{total}");
        // and one it does not reach is left alone
        assert_eq!(box_minus(&wall, &[5.0, 0.0, 0.0, 6.0, 1.0, 1.0]), vec![wall]);
    }
}
