//! The figures across the void (`docs/MEGASTRUCTURE.md`, design 7; after the
//! figures on the far ridge of SCP-3333 "Tower" by Jekeled, CC BY-SA 3.0):
//! now and then, off across the void on a floor of the Building, someone
//! stands where no walk reaches (on a gallery of another storey in the Well,
//! on a flight that crosses the dark in the Stair Halls, on top of a machine
//! or on a gantry in the Works), turned toward you. Look at it a moment and
//! it turns its back; look away and it is gone (watch it long, or come
//! nearer, and it is gone anyway). Long after, it stands somewhere else,
//! always across the void, never while the siren sounds or the fog is up.
//! It never comes near, and it does nothing; nothing sends it.
//!
//! Its choices come from a stream of its own, seeded from the floor's seed,
//! so it moves no other draw (`DEVELOPMENT.md`): not the build's, not the
//! native stream's. Its perches are gathered with the dressing and draw
//! nothing.
use crate::level::archgen::{segment_clear_3d, DIRS, GROUND_FLOOR, GROUND_MASS, GROUND_STAIR, GROUND_VOID};
use crate::level::building::{BuildingPlan, Style, MASS_TOP, STOREY, VOID_DEPTH};
use crate::prelude::*;
use crate::world::assembly::Stage;

/// How far a figure stands from you, at least and at most, metres (level)
pub const NEAR: f64 = 22.0;
pub const FAR: f64 = 60.0;
/// Come this near a standing figure and it turns away, metres
pub const TOO_NEAR: f64 = 14.0;
/// How long it is looked at before it turns away, seconds
pub const SEEN: f64 = 1.2;
/// How long it takes to turn its back, seconds
pub const TURN: f64 = 1.1;
/// The first wait on a floor, and the waits after, seconds (long: the same
/// thing seen again soon is no longer dread). The first is half what it was
/// when a floor took twice as long to cross (G1): it comes about once a floor
pub const FIRST: f64 = 35.0;
pub const AWAY: (f64, f64) = (100.0, 220.0);
/// In the middle of the view (looked at), and anywhere in it
pub const LOOKED: f64 = 0.96;
pub const IN_VIEW: f64 = 0.55;
/// How far under your eye a figure may stand, and how far over it, as the
/// rise of the line to it a metre: never much below (your own parapet or
/// rail, the dressing's and not a solid, would hide it), never overhead
pub const SLOPE: (f64, f64) = (-0.07, 0.45);
/// What stands behind a figure for it to show against: a face or a floor no
/// further behind it than this (nearer, the fog has not darkened it), or
/// nothing for this far (the haze), metres
pub const BACKDROP: (f64, f64) = (35.0, 140.0);
/// Its own stream's salt
pub const FIGURE_SALT: u64 = 0xF16E_0000_0A5C;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FigureState {
    /// waiting to stand somewhere
    #[default]
    Gone,
    /// standing, turned toward you
    Standing,
    /// turning its back, and gone when you look away
    Leaving,
}

/// The one figure of a floor.
#[derive(Clone, Debug)]
pub struct FarFigure {
    pub body: Obj,
    pub state: FigureState,
    /// seconds left gone / seconds looked at / seconds into leaving
    pub t: f64,
    pub at: Vec3,
    pub yaw: f64,
    /// how many times it has stood on this floor
    pub stood: u32,
    pub rng: SplitMix64,
}

impl Game {
    /// A figure: tall, thin, grey, its head a little forward (unlit, a
    /// little paler than the concrete: a shape against the dark far off and
    /// against a lit face alike, whatever the light)
    fn make_far_figure(&self) -> Obj {
        let g = group();
        let dark = basic_mat!({ color: 0x6b6a66 });
        // read far off as a person: two legs from the ground, shoulders
        // wider than the head, the head apart over a neck
        for s in [-1.0, 1.0] {
            let leg = mesh(capsule_geo!(0.075, 0.72, 2, 6), &dark);
            leg.position().set(s * 0.11, 0.075 + 0.36, 0.0);
            g.add(&leg);
        }
        let body = mesh(capsule_geo!(0.18, 0.5, 3, 8), &dark);
        body.position().y = 0.9 + 0.43;
        body.scale().set(1.15, 1.0, 0.75);
        g.add(&body);
        let head = mesh(sphere_geo!(0.12, 8, 6), &dark);
        head.position().set(0, 1.94, 0.04);
        g.add(&head);
        for s in [-1.0, 1.0] {
            let arm = mesh(capsule_geo!(0.05, 0.78, 2, 6), &dark);
            arm.position().set(s * 0.29, 1.2, 0.0);
            g.add(&arm);
        }
        // a little taller than anyone
        g.scale().set(1.12, 1.12, 1.12);
        g.set_visible(false);
        g.set_name("far-figure");
        g.user().gameplay = true;
        g
    }

    /// Where a figure may stand on this floor, gathered after the dressing:
    /// the Works' machine tops (the Well's galleries of other storeys at
    /// their parapets, the Stair Halls' flights and the Works' gantries are
    /// added by their dressing, where it lays them). Only places no walk is
    /// within a storey of.
    pub(crate) fn gather_figure_perches(&mut self, plan: &BuildingPlan) {
        let mut out = vec![];
        for z in 0..self.h {
            for x in 0..self.w {
                let g = self.ground_or_mass(x, z);
                let (cx, cz) = (self.wx(x), self.wz(z));
                match plan.style {
                    // (the Well's are its galleries' decks, gathered where
                    // the dressing lays them)
                    Style::Well => {}
                    Style::Works => {
                        // on top of a machine housing
                        if g.kind == GROUND_MASS && (g.h1 as f64) < MASS_TOP as f64 - 1.0 && (x + z) % 2 == 0 {
                            out.push(Vec3::new(cx, g.h1 as f64, cz));
                        }
                    }
                    Style::Halls => {}
                }
            }
        }
        self.figure_perches.extend(out);
        let unreached: Vec<Vec3> = self.figure_perches.iter().copied().filter(|p| self.no_walk_near(*p)).collect();
        self.figure_perches = unreached;
        if !self.figure_perches.is_empty() {
            let body = self.make_far_figure();
            self.world.add(&body);
            self.far_figure = Some(FarFigure {
                body,
                state: FigureState::Gone,
                t: FIRST,
                at: Vec3::new(0.0, 0.0, 0.0),
                yaw: 0.0,
                stood: 0,
                rng: SplitMix64::new(self.current_seed, FIGURE_SALT),
            });
        }
    }

    /// Whether no walk comes within a storey of `p` (the tiles round it)
    pub fn no_walk_near(&self, p: Vec3) -> bool {
        let (u, v) = (self.to_u(p.x).floor() as i32, self.to_v(p.z).floor() as i32);
        (-1..=1).all(|dz| {
            (-1..=1).all(|dx| {
                let (x, z) = (u + dx, v + dz);
                let g = self.ground_or_mass(x, z);
                !g.walkable() || (self.floor_y_of(x, z) - p.y).abs() >= STOREY - 0.2
            })
        })
    }

    /// How squarely you look at `p` (a figure's chest): the cosine to the
    /// middle of the view
    fn look_dot(&self, p: Vec3) -> f64 {
        let c = self.player.pos;
        let (dx, dy, dz) = (p.x - c.x, p.y + 1.4 - c.y, p.z - c.z);
        let d = (dx * dx + dy * dy + dz * dz).sqrt().max(1e-6);
        let f = self.player.fwd();
        let pc = self.player.pitch.cos();
        (dx * f.x * pc + dy * self.player.pitch.sin() + dz * f.z * pc) / d
    }

    /// Whether nothing stands between the eye at `c` and a figure at `p`,
    /// head, chest and knees: no solid, no flight's body nor floor's slab,
    /// and in the Well none of the galleries' decks of the other storeys
    /// (the dressing's, not solids)
    fn sight_from(&self, c: Vec3, p: Vec3) -> bool {
        [0.5, 1.4, 2.1].iter().all(|&h| self.view_clear(c, Vec3::new(p.x, p.y + h, p.z)))
    }

    /// Whether the eye at `c` sees the point `b`: no solid between, and none
    /// of what the dressing builds over the walks (flights' bodies, floors'
    /// slabs, the Well's decks, lamps' heads) either
    pub fn view_clear(&self, c: Vec3, b: Vec3) -> bool {
        let clear = match &self.solids {
            Some(s) => segment_clear_3d(s, self.to_u(c.x), self.to_v(c.z), c.y, self.to_u(b.x), self.to_v(b.z), b.y),
            None => true,
        };
        clear && self.decks_clear(c, b) && self.lamps_clear(c, b)
    }

    /// Whether the line from `a` to `b` passes no lamp's head (a lamp near
    /// the eye hides a figure far off)
    fn lamps_clear(&self, a: Vec3, b: Vec3) -> bool {
        let (dx, dy, dz) = (b.x - a.x, b.y - a.y, b.z - a.z);
        let len2 = (dx * dx + dy * dy + dz * dz).max(1e-9);
        self.panels.iter().all(|l| {
            let (px, py, pz) = (l.wx - a.x, l.y - a.y, l.wz - a.z);
            let t = ((px * dx + py * dy + pz * dz) / len2).clamp(0.0, 1.0);
            let (ex, ey, ez) = (px - dx * t, py - dy * t, pz - dz * t);
            ex * ex + ey * ey + ez * ez > 0.7 * 0.7
        })
    }

    /// The Well's gallery decks: over every walk along a block face, one a
    /// storey up and down (`building_dress`); the deck under a tile's
    /// storey `k` lies at its floor + `k` storeys
    fn decked(&self, x: i32, z: i32) -> Option<f64> {
        let g = self.ground_or_mass(x, z);
        if g.kind != GROUND_FLOOR || !self.in_grid(x, z) {
            return None;
        }
        let face = (0..4).any(|d| {
            self.in_grid(x + DIRS[d][0], z + DIRS[d][1]) && self.ground_or_mass(x + DIRS[d][0], z + DIRS[d][1]).kind == GROUND_MASS
        });
        face.then_some(g.h0 as f64)
    }

    /// Whether the line from `a` to `b` passes under no flight and through
    /// no floor, and (in the Well) crosses no gallery deck of another storey;
    /// on the tile it ends on, it may (a figure stands on its deck)
    pub(crate) fn decks_clear(&self, a: Vec3, b: Vec3) -> bool {
        let well = self.building.as_ref().is_some_and(|p| p.style == Style::Well);
        let end = (self.to_u(b.x).floor() as i32, self.to_v(b.z).floor() as i32);
        let len = ((b.x - a.x).powi(2) + (b.z - a.z).powi(2)).sqrt();
        let n = (len / 0.25).ceil().max(1.0) as usize;
        let mut prev = a.y;
        for i in 1..=n {
            let t = i as f64 / n as f64;
            let (qx, qy, qz) = (a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t, a.z + (b.z - a.z) * t);
            let (x, z) = (self.to_u(qx).floor() as i32, self.to_v(qz).floor() as i32);
            if (x, z) == end {
                break;
            }
            let g = self.ground_or_mass(x, z);
            let floor = self.floor_y_at(qx, qz);
            if (g.kind == GROUND_STAIR && qy < floor) || (g.kind == GROUND_FLOOR && qy < floor && qy > floor - 0.4) {
                return false;
            }
            if !well {
                prev = qy;
                continue;
            }
            if let Some(h) = self.decked(x, z) {
                let (lo, hi) = (prev.min(qy), prev.max(qy));
                // the decks this step passes through (each 0.3 m thick, its
                // top at its storey, and over the void its parapet on it),
                // other than the walk's own
                let rail = if (0..4).any(|d| self.ground_or_mass(x + DIRS[d][0], z + DIRS[d][1]).kind == GROUND_VOID) { 1.05 } else { 0.0 };
                let k0 = ((lo - rail - h) / STOREY).floor() as i32;
                let k1 = ((hi + 0.3 - h) / STOREY).ceil() as i32;
                if (k0..=k1).any(|k| {
                    let top = h + k as f64 * STOREY;
                    k != 0 && k.abs() <= 9 && hi >= top - 0.3 && lo <= top + rail
                }) {
                    return false;
                }
            }
            prev = qy;
        }
        true
    }

    /// Whether, seen from `c`, a figure at `p` has something behind it to
    /// show against: on along the line past it, a face or a floor before
    /// the fog has darkened it, or nothing at all (the haze); not the void's
    /// depths, nor the Works' ceiling (as dark as it is)
    fn backed(&self, c: Vec3, p: Vec3) -> bool {
        let (ax, ay, az) = (p.x, p.y + 1.4, p.z);
        let (dx, dy, dz) = (ax - c.x, ay - c.y, az - c.z);
        let len = (dx * dx + dy * dy + dz * dz).sqrt().max(1e-6);
        let (ux, uy, uz) = (dx / len, dy / len, dz / len);
        let roof = match &self.building {
            Some(b) if b.style == Style::Works => self.works_ceiling(),
            _ => f64::INFINITY,
        };
        let mut t = 1.0;
        while t <= BACKDROP.1 {
            let (qx, qy, qz) = (ax + ux * t, ay + uy * t, az + uz * t);
            let (x, z) = (self.to_u(qx).floor() as i32, self.to_v(qz).floor() as i32);
            if !self.in_grid(x, z) {
                // past the floor's edge: its outer wall, near or far
                return t <= BACKDROP.0;
            }
            let g = self.ground[self.t_idx(x, z)];
            let hit = qy >= roof
                || (g.kind == GROUND_MASS && qy >= g.h0 as f64 && qy <= g.h1 as f64)
                || (g.walkable() && qy <= self.floor_y_of(x, z))
                || (g.kind == GROUND_VOID && qy < VOID_DEPTH as f64 + 40.0);
            if hit {
                return t <= BACKDROP.0 && qy < roof && !(g.kind == GROUND_VOID && qy < VOID_DEPTH as f64 + 40.0);
            }
            t += 1.0;
        }
        true
    }

    /// Whether a figure at `p` could be seen from an eye at `c`: far enough
    /// and not too far, not far under you nor overhead, nothing in between,
    /// and something behind it to show against
    pub fn perch_seen_from(&self, c: Vec3, p: Vec3) -> bool {
        let d = hypot2(p.x - c.x, p.z - c.z);
        let rise = (p.y + 1.4 - c.y) / d.max(1e-6);
        // on top of a machine, seen from below, its own edge hides it
        let (x, z) = (self.to_u(p.x).floor() as i32, self.to_v(p.z).floor() as i32);
        let on_top = self.ground_or_mass(x, z).kind == GROUND_MASS;
        (NEAR..FAR).contains(&d)
            && (SLOPE.0..=SLOPE.1).contains(&rise)
            && (!on_top || c.y >= p.y + 0.3)
            && self.sight_from(c, p)
            && self.backed(c, p)
    }

    /// A perch to stand on: one you could see from where you are if you
    /// turned, out of your view now, and not the one it last stood on
    fn pick_perch(&self, r: &mut SplitMix64, last: Option<Vec3>) -> Option<Vec3> {
        let n = self.figure_perches.len();
        if n == 0 {
            return None;
        }
        let pp = self.player.pos;
        let start = (r.next() * n as f64) as usize % n;
        let again = |p: Vec3| last.is_some_and(|l| hypot2(p.x - l.x, p.z - l.z) < 6.0 && (p.y - l.y).abs() < 1.0);
        (0..n)
            .map(|k| self.figure_perches[(start + k) % n])
            .find(|&p| !again(p) && self.look_dot(p) < IN_VIEW && self.perch_seen_from(pp, p))
    }

    /// One step of the figure (the director's, on the Building's floors)
    pub fn update_far_figure(&mut self, dt: f64) {
        let Some(mut f) = self.far_figure.take() else { return };
        let pp = self.player.pos;
        let d = hypot2(pp.x - f.at.x, pp.z - f.at.z);
        match f.state {
            FigureState::Gone => {
                f.t -= dt;
                // not while the fog comes: the Building holds its breath
                if f.t <= 0.0 {
                    if self.assembly.stage != Stage::Quiet {
                        f.t = 8.0;
                    } else if let Some(p) = self.pick_perch(&mut f.rng, (f.stood > 0).then_some(f.at)) {
                        f.at = p;
                        f.state = FigureState::Standing;
                        f.t = 0.0;
                        f.stood += 1;
                        f.body.position().copy(&p);
                        f.body.set_visible(true);
                        self.director_note("figure:stood");
                    } else {
                        f.t = 6.0;
                    }
                }
            }
            FigureState::Standing => {
                // turned toward you
                f.yaw = (pp.x - f.at.x).atan2(pp.z - f.at.z);
                f.body.rotation().y = f.yaw;
                let looked = self.look_dot(f.at) >= LOOKED && self.sight_from(pp, f.at);
                // (a time under 0 holds it: the debug screen's)
                f.t = if looked {
                    f.t + dt
                } else if f.t > 0.0 {
                    (f.t - dt * 0.5).max(0.0)
                } else {
                    f.t
                };
                // looked at, come upon, or the siren: it goes
                if f.t >= SEEN || d < TOO_NEAR || self.assembly.stage != Stage::Quiet {
                    f.state = FigureState::Leaving;
                    f.t = 0.0;
                    self.director_note("figure:turned");
                }
            }
            FigureState::Leaving => {
                f.t += dt;
                let k = (f.t / TURN).min(1.0);
                f.body.rotation().y = f.yaw + PI * k * k * (3.0 - 2.0 * k);
                // its back to you, it is gone the moment you look away
                let watched = self.look_dot(f.at) >= IN_VIEW && self.sight_from(pp, f.at);
                if f.t >= TURN && (!watched || f.t > 12.0 || d < TOO_NEAR * 0.6) {
                    f.body.set_visible(false);
                    f.state = FigureState::Gone;
                    f.t = f.rng.range(AWAY.0, AWAY.1);
                }
            }
        }
        self.far_figure = Some(f);
    }

    /// For `--screen figure` (and the tests): the figure standing across the void, about
    /// 40 m off, from the walk nearest the arrival it can be seen from, and
    /// the eye turned just beside it (it does not turn away while the
    /// frames run)
    pub fn show_far_figure(&mut self) {
        let Some(plan) = self.building.clone() else { return };
        let mut best: Option<(f64, Vec3, Vec3)> = None;
        for z in 0..self.h {
            for x in 0..self.w {
                if !self.ground[self.t_idx(x, z)].walkable() {
                    continue;
                }
                let (dx, dz) = (x - plan.spawn.0, z - plan.spawn.1);
                let eye = Vec3::new(self.wx(x), self.floor_y_of(x, z) + EYE, self.wz(z));
                for &p in &self.figure_perches {
                    let d = hypot2(p.x - eye.x, p.z - eye.z);
                    let score = (d - 40.0).abs() * 0.5 + (dx.abs() + dz.abs()) as f64 * 0.3;
                    if best.is_none_or(|b| score < b.0) && self.perch_seen_from(eye, p) {
                        best = Some((score, eye, p));
                    }
                }
            }
        }
        let Some((_, eye, p)) = best else { return };
        self.player.pos = eye;
        let (dx, dy, dz) = (p.x - eye.x, p.y + 1.2 - eye.y, p.z - eye.z);
        // a little to its side, so the crosshair and a prompt leave it clear
        self.player.yaw = (-dx).atan2(-dz) - 0.1;
        self.player.pitch = dy.atan2((dx * dx + dz * dz).sqrt());
        if let Some(f) = self.far_figure.as_mut() {
            f.at = p;
            f.state = FigureState::Standing;
            f.t = -1e6;
            f.yaw = (eye.x - p.x).atan2(eye.z - p.z);
            f.body.position().copy(&p);
            f.body.rotation().y = f.yaw;
            f.body.set_visible(true);
        }
    }
}
