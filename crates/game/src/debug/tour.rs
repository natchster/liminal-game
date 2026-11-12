//! The visual playtest's tour (`docs/VISUAL_PLAYTEST.md`): where a body
//! can stand nearest every edge of the walkable floor, and the views from
//! there a player would get (at the edge, along it both ways, up at its top,
//! down at its foot, crouched). An edge is where a body walking from a
//! tile's middle (and from two lines either side of it) along an axis is
//! stopped before it reaches open floor: a wall, a parapet over the void, a
//! partition, a step, a thing that stops a body, the grid's end. Stands are
//! chosen spread over the level (each the farthest from those chosen) to a
//! budget, inside an area when one is given. Pure: no random draw, no clock.
use crate::base::game::Game;
use crate::level::archgen::{GROUND_MASS, GROUND_VOID};
use crate::prelude::*;

/// The body's circle as play moves it (`update_player`)
pub const BODY: f64 = 0.20;
/// The walk's step when finding an edge (metres)
const STEP: f64 = 0.02;
/// Lines a tile is walked from, across the walk (metres from its middle)
const LANES: [f64; 3] = [-1.2, 0.0, 1.2];
/// Two stands this close facing alike are one
const SAME_STAND: f64 = 0.6;
/// A corner: stopped again within this of the edge's stand, sideways
const CORNER_REACH: f64 = 0.6;

/// A place a body can stand nearest an edge, facing it
#[derive(Clone, Debug, PartialEq)]
pub struct Stand {
    pub x: f64,
    pub z: f64,
    pub floor: f64,
    /// facing the edge (the camera's yaw)
    pub yaw: f64,
    /// what stopped the walk: wall, void, mass, step, grid or solid
    pub edge: &'static str,
    pub corner: bool,
    pub tile: (i32, i32),
}

/// A view of the tour: a stand's camera
#[derive(Clone, Debug, PartialEq)]
pub struct View {
    /// `L<level>_<stand>_<look>`
    pub name: String,
    pub stand: usize,
    /// at, left, right, up, down, crouch
    pub look: &'static str,
    /// its stand's edge (`Stand::edge`), and whether it is a corner
    pub edge: &'static str,
    pub corner: bool,
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub yaw: f64,
    pub pitch: f64,
}

/// What to tour
#[derive(Clone, Copy, Debug)]
pub struct TourOpts {
    /// views at most (a level's stands are chosen to fill it)
    pub budget: usize,
    /// world x0, z0, x1, z1: only stands inside
    pub area: Option<[f64; 4]>,
}

impl Default for TourOpts {
    fn default() -> Self {
        TourOpts { budget: 180, area: None }
    }
}

/// The looks from an edge's stand: (name, turn from facing it, pitch, crouched)
const EDGE_LOOKS: [(&str, f64, f64, bool); 6] = [
    ("at", 0.0, 0.0, false),
    ("left", 70.0, 0.0, false),
    ("right", -70.0, 0.0, false),
    ("up", 0.0, 40.0, false),
    ("down", 0.0, -55.0, false),
    ("crouch", 0.0, -10.0, true),
];
/// ... and from a corner's
const CORNER_LOOKS: [(&str, f64, f64, bool); 3] = [("at", 0.0, 0.0, false), ("up", 0.0, 45.0, false), ("down", 0.0, -50.0, false)];

impl Game {
    /// Whether a body can stand at (x, z): on the grid, off the wall tiles,
    /// on ground that is walked (not the void, not a mass), and with its
    /// circle clear of every solid as play moves it
    pub fn standable(&self, x: f64, z: f64) -> bool {
        let (tx, tz) = (self.tx(x), self.tz(z));
        self.in_grid(tx, tz)
            && !self.is_wall_tile(tx, tz)
            && self.ground.get(self.t_idx(tx, tz)).is_none_or(|g| g.walkable())
            && !self.circle_hits_wall(x, z, BODY)
    }

    /// What stops a body walking from tile `t` the way (dx, dz)
    fn edge_kind(&self, t: (i32, i32), d: (i32, i32), floor: f64) -> &'static str {
        let n = (t.0 + d.0, t.1 + d.1);
        if !self.in_grid(n.0, n.1) {
            return "grid";
        }
        if let Some(g) = self.ground.get(self.t_idx(n.0, n.1)) {
            if g.kind == GROUND_VOID {
                return "void";
            }
            if g.kind == GROUND_MASS {
                return "mass";
            }
        }
        if self.is_wall_tile(n.0, n.1) {
            return "wall";
        }
        if (self.floor_y_at(self.wx(n.0), self.wz(n.1)) - floor).abs() > 0.4 {
            return "step";
        }
        "solid"
    }

    /// Every stand nearest an edge on this level (inside `area` if given)
    pub fn tour_stands(&self, area: Option<[f64; 4]>) -> Vec<Stand> {
        let inside =
            |x: f64, z: f64| area.is_none_or(|a| x >= a[0].min(a[2]) && x <= a[0].max(a[2]) && z >= a[1].min(a[3]) && z <= a[1].max(a[3]));
        let mut out: Vec<Stand> = vec![];
        for tz in 0..self.h {
            for tx in 0..self.w {
                let (cx, cz) = (self.wx(tx), self.wz(tz));
                // (a tile whose walks cannot end inside the area is not walked)
                if area.is_some_and(|a| {
                    let (dx, dz) = ((a[0].min(a[2]) - cx).max(cx - a[0].max(a[2])), (a[1].min(a[3]) - cz).max(cz - a[1].max(a[3])));
                    dx > CELL || dz > CELL
                }) {
                    continue;
                }
                for d in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                    let (dx, dz) = (d.0 as f64, d.1 as f64);
                    for lane in LANES {
                        // across the walk: (-dz, dx)
                        let (x0, z0) = (cx - dz * lane, cz + dx * lane);
                        if !self.standable(x0, z0) {
                            continue;
                        }
                        let floor = self.floor_y_at(x0, z0);
                        let limit = CELL / 2.0 + 0.6;
                        let mut s = 0.0;
                        while s < limit && self.standable(x0 + dx * (s + STEP), z0 + dz * (s + STEP)) {
                            s += STEP;
                        }
                        if s >= limit {
                            continue; // open floor that way
                        }
                        let (x, z) = (x0 + dx * s, z0 + dz * s);
                        if (self.floor_y_at(x, z) - floor).abs() > 0.3 || !inside(x, z) {
                            continue;
                        }
                        let yaw = (-dx).atan2(-dz);
                        if out.iter().any(|o| !o.corner && (o.yaw - yaw).abs() < 1e-9 && (o.x - x).hypot(o.z - z) < SAME_STAND) {
                            continue;
                        }
                        let edge = self.edge_kind((tx, tz), d, floor);
                        out.push(Stand { x, z, floor, yaw, edge, corner: false, tile: (tx, tz) });
                        // a corner beside it: stopped again sideways, close by
                        for side in [1.0, -1.0] {
                            let (sx, sz) = (-dz * side, dx * side);
                            let mut r = 0.0;
                            while r < CORNER_REACH && self.standable(x + sx * (r + STEP), z + sz * (r + STEP)) {
                                r += STEP;
                            }
                            if r < CORNER_REACH {
                                let (qx, qz) = (x + sx * r, z + sz * r);
                                let cy = (-(dx + sx)).atan2(-(dz + sz));
                                if !out.iter().any(|o| o.corner && (o.x - qx).hypot(o.z - qz) < SAME_STAND) {
                                    out.push(Stand { x: qx, z: qz, floor, yaw: cy, edge, corner: true, tile: (tx, tz) });
                                }
                            }
                        }
                    }
                }
            }
        }
        out
    }

    /// The tour's views on this level: stands spread over it (each the
    /// farthest from those already chosen, from the first in the grid's
    /// order), their looks, to `budget` views
    pub fn tour_views(&self, o: &TourOpts) -> Vec<View> {
        let stands = self.tour_stands(o.area);
        if stands.is_empty() {
            return vec![];
        }
        let per = |s: &Stand| if s.corner { CORNER_LOOKS.len() } else { EDGE_LOOKS.len() };
        if per(&stands[0]) > o.budget {
            return vec![];
        }
        let mut chosen: Vec<usize> = vec![0];
        let mut nearest: Vec<f64> = stands.iter().map(|s| (s.x - stands[0].x).hypot(s.z - stands[0].z)).collect();
        let mut views = per(&stands[0]);
        while views < o.budget && chosen.len() < stands.len() {
            let (k, d) = nearest.iter().enumerate().fold((0, -1.0), |b, (i, &d)| if d > b.1 + 1e-9 { (i, d) } else { b });
            if d <= 0.0 || views + per(&stands[k]) > o.budget {
                break;
            }
            chosen.push(k);
            views += per(&stands[k]);
            for (i, s) in stands.iter().enumerate() {
                nearest[i] = nearest[i].min((s.x - stands[k].x).hypot(s.z - stands[k].z));
            }
        }
        let mut out = vec![];
        for (n, &k) in chosen.iter().enumerate() {
            let s = &stands[k];
            let looks: &[(&str, f64, f64, bool)] = if s.corner { &CORNER_LOOKS } else { &EDGE_LOOKS };
            for &(look, turn, pitch, crouch) in looks {
                out.push(View {
                    name: format!("L{}_{n:03}{}_{look}", self.level_num, if s.corner { "c" } else { "" }),
                    stand: k,
                    look,
                    edge: s.edge,
                    corner: s.corner,
                    x: s.x,
                    y: s.floor + if crouch { CROUCH_EYE } else { EYE },
                    z: s.z,
                    yaw: s.yaw + turn.to_radians(),
                    pitch: pitch.to_radians(),
                });
            }
        }
        out
    }
}
