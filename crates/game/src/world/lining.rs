//! The Utility Halls' lining (`docs/plan/FACILITY.md` F5; the real tunnels:
//! `docs/research/facility.md`): concrete sprayed on, walls and crown
//! (`shotcrete`, `visuals::photo`), a haunch rounding every corner where a
//! wall meets the crown, bolts set in a row along each haunch, and the
//! runs on the crown of each straight corridor three tiles long or more: a
//! ladder tray of cables, a pair of pipes on trapeze hangers, or both, one
//! either side of the crown's middle; the pipes turn up through the crown
//! at a run's ends on their elbows, and the tray's cables turn up with them.
//! None where the moss has papered the place (the Lobby's look) and no
//! runs where a place was left empty (`assets/src/props/service_run/`).
//!
//! Architecture, made once the floor has settled (`Game::lay_lining`,
//! after the lamps): from the solids as they are, with no random draw (a
//! run's kind and side come from its place, on a stream of its own). A
//! haunch springs above any door's head (`HAUNCH_SPRING`), so a low
//! corridor's haunch is small, and stops short of a door or a window; a
//! run keeps to a person's headroom and off the lamps, the props on the
//! crown and whatever stands in the way, and ends where they begin.

use crate::base::records::Lamp;
use crate::base::rng::SplitMix64;
use crate::level::archgen::{BOX_FLOOR, BOX_VOID, BOX_WALL, DIRS, Solids, ray_axis};
use crate::prelude::*;
use crate::visuals::lighting::RF_MOSS;
use crate::visuals::prop_look::{PropBox, settle_material};
use crate::world::arch::ARCH_CHUNK;
use std::collections::HashMap;

/// No haunch comes lower than this (m): a door's head and its frame stay
/// clear of it
pub const HAUNCH_SPRING: f64 = 2.2;
/// A haunch's largest radius (m)
pub const HAUNCH_R_MAX: f64 = 0.45;
/// Under this a haunch is not worth its corner (m)
pub const HAUNCH_R_MIN: f64 = 0.12;
/// Rock bolts along a haunch, about this far apart (m), as a pattern is set
pub const BOLT_EVERY: f64 = 1.25;
/// No run hangs lower than this (m): a person's headroom
pub const RUN_HEADROOM: f64 = 2.05;
/// A run hangs every so far (m): three to a 2 in pipe's 20 ft length, as
/// grooved pipe is hung (`assets/src/props/service_run/references.toml`)
pub const HANGER_EVERY: f64 = 2.4;
/// A grooved coupling joins a pipe's lengths, a length apart (m: 20 ft)
pub const PIPE_LENGTH: f64 = 6.1;
/// A run keeps this far out from the corridor's middle line, clear of the
/// jar lamps on the crown (m)
const RUN_INNER: f64 = 0.2;
/// A run's tile is clear across this far either side of its middle line (m):
/// the runs' width, inside the narrowest corridor's walls (0.85 m)
const RUN_STRIP: f64 = 0.65;
/// The runs' own stream, so no other draw moves
const RUN_SALT: u64 = 0x5EC7_10A1_0F9C_0DE5;
/// Segments of a haunch's quarter circle
const SEG: usize = 9;
/// How far a haunch is set into the wall and the crown, so no seam shows (m)
const SET_IN: f64 = 0.002;

/// The archetype the facility lines (`docs/plan/FACILITY.md` F5): the
/// Utility Halls. Its generator's table keeps the old game's poured concrete
/// and slab (`level::archgen::ARCHETYPES`, and what the placers read of a
/// zone); the world draws it sprayed.
pub const LINED: &str = "tunnels";

/// The surface a zone's walls are drawn in: sprayed concrete where it is
/// lined, the Habitable Zone's halls painted two-tone (F6, `world::halls`),
/// else its archetype's own
pub fn wall_mat_of(zn: &crate::level::archgen::Zone) -> &'static str {
    match zn.arch {
        LINED => "shotcrete",
        crate::world::halls::HALL => "twotone",
        _ => zn.wall_mat,
    }
}

/// The surface a zone's ceiling is drawn in (see `wall_mat_of`): the
/// Habitable Zone's flat slab bare, cast against steel forms (the blind
/// review: its painted slab's blotches "read more like sky than slab")
pub fn ceil_mat_of(zn: &crate::level::archgen::Zone) -> &'static str {
    match zn.arch {
        LINED => "shotcrete",
        crate::world::halls::HALL => "formwork",
        _ => zn.ceil_mat,
    }
}

/// The radius of a haunch under a crown `ceil` high, or none
pub fn haunch_r(ceil: f64) -> Option<f64> {
    let r = (ceil - HAUNCH_SPRING).min(HAUNCH_R_MAX);
    (r >= HAUNCH_R_MIN).then_some(r)
}

/// The haunch's arc meets the wall and the crown at this angle (rad), a
/// crease: never tangent to them, whose grazing faces flickered (the visual
/// playtest)
pub const CREASE: f64 = std::f64::consts::PI / 12.0;

/// The haunch's circle for a haunch of radius 1: its centre `c` out from the
/// wall and `c` down from the crown, its radius; the arc runs from the wall a
/// radius down to the crown a radius out, `CREASE` to each
fn circle() -> (f64, f64) {
    let (s, co) = (CREASE.dsin(), CREASE.dcos());
    let c = 1.0 / (1.0 - s / co);
    (c, c / co)
}

/// How high a haunch of radius `r` under a crown `ceil` high stands `d`
/// metres out from its wall (the crown itself beyond it)
pub fn haunch_y(ceil: f64, r: f64, d: f64) -> f64 {
    if d >= r {
        return ceil;
    }
    let (c, rr) = circle();
    let d = d.max(0.0) / r;
    ceil - c * r + r * (rr * rr - (d - c) * (d - c)).max(0.0).sqrt()
}

/// One straight haunch along a wall's face, in tile units: the floor tile in
/// front, the face's normal (`DIRS` index, out of the wall), the face's
/// coordinate across, and its extent along, each end set into what it meets
#[derive(Clone, Debug)]
pub struct Haunch {
    pub t: usize,
    pub dir: usize,
    pub at: f64,
    pub a: f64,
    pub b: f64,
    pub ceil: f64,
    pub r: f64,
    pub key: String,
    /// its ends at `a` and `b` meet a wall across them (an inside corner,
    /// where the other wall's haunch meets this one): mitred, not closed
    pub meet_a: bool,
    pub meet_b: bool,
}

/// A mesh being laid: positions, normals, UVs, triangles (the facility's
/// curved and flared architecture, mapped by its own UVs: `Patch::ArchUV`
/// maps only instanced faces, by their normal)
#[derive(Default)]
pub(crate) struct Face {
    pos: Vec<f32>,
    nrm: Vec<f32>,
    uv: Vec<f32>,
    pub(crate) idx: Vec<u32>,
}

impl Face {
    pub(crate) fn vert(&mut self, p: Vec3, n: Vec3, uv: (f64, f64)) -> u32 {
        let i = (self.pos.len() / 3) as u32;
        self.pos.extend([p.x as f32, p.y as f32, p.z as f32]);
        self.nrm.extend([n.x as f32, n.y as f32, n.z as f32]);
        self.uv.extend([uv.0 as f32, uv.1 as f32]);
        i
    }

    /// The mesh of it in `mat`, bounded
    pub(crate) fn mesh(self, mat: &Mat) -> Obj {
        let geo = Geo::new();
        geo.set_position(self.pos).set_normal(self.nrm).set_uv(self.uv).set_index(self.idx);
        geo.compute_bounding_box();
        geo.g().compute_bounding_sphere();
        mesh(&geo, mat)
    }
}

/// One haunch laid into `f`: its corner at `o` (a world point on the wall's
/// face, at the crown, each a little into them), along `u` for `len` metres,
/// out of the wall along `n`; its arc (`circle`, `arc` its points: angle, up,
/// out, how far round, for a radius of 1) scaled to `r`. An end that meets a
/// wall is mitred where the other wall's haunch meets it (as far along from
/// the corner as it stands out from its own wall), an open one closed. Its
/// photograph is mapped in metres (`scale`): along it as its wall's is, and
/// up its wall and on round its arc.
#[expect(
    clippy::too_many_arguments,
    reason = "a haunch is its frame, extent, shape and mapping; a struct for one call would only rename them"
)]
fn haunch_face(
    f: &mut Face,
    arc: &[(f64, f64, f64, f64)],
    o: Vec3,
    u: Vec3,
    n: Vec3,
    len: f64,
    r: f64,
    ceil: f64,
    meet: (bool, bool),
    scale: (f64, f64),
) {
    let at = |along: f64, y: f64, out: f64| -> Vec3 { Vec3::new(o.x + u.x * along + n.x * out, o.y + y, o.z + u.z * along + n.z * out) };
    // the wall's own mapping along it (`Patch::ArchUV`: by the way it faces)
    let along_uv = |p: &Vec3| (n.z * p.x - n.x * p.z) / scale.0;
    let base = (f.pos.len() / 3) as u32;
    for &(a, y, z, round) in arc {
        let (y, out) = (y * r, z * r);
        let s0 = if meet.0 { out.min(len / 2.0) } else { 0.0 };
        let s1 = if meet.1 { (len - out).max(len / 2.0) } else { len };
        let nn = Vec3::new(-n.x * a.dcos(), -a.dsin(), -n.z * a.dcos());
        let v = (ceil - r + round * r) / scale.1;
        for along in [s0, s1] {
            let p = at(along, y, out);
            f.vert(p, nn, (along_uv(&p), v));
        }
    }
    for i in 0..SEG as u32 {
        let (a0, a1, b0, b1) = (base + 2 * i, base + 2 * i + 1, base + 2 * i + 2, base + 2 * i + 3);
        f.idx.extend([a0, b1, b0, a0, a1, b1]);
    }
    // an open end closed: a fan from the corner
    for (end, along, sign) in [(meet.0, 0.0, -1.0), (meet.1, len, 1.0)] {
        if end {
            continue;
        }
        let nn = Vec3::new(u.x * sign, 0.0, u.z * sign);
        let corner = f.vert(at(along, 0.0, 0.0), nn, (0.0, ceil / scale.1));
        for &(_, y, z, _) in arc {
            let (y, out) = (y * r, z * r);
            f.vert(at(along, y, out), nn, (out / scale.0, (ceil + y) / scale.1));
        }
        for i in 0..SEG as u32 {
            if sign < 0.0 {
                f.idx.extend([corner, corner + 1 + i, corner + 2 + i]);
            } else {
                f.idx.extend([corner, corner + 2 + i, corner + 1 + i]);
            }
        }
    }
}

/// A frame from three axes and an origin (the axes as columns; right-handed:
/// `z = x × y`)
fn frame(x: Vec3, y: Vec3, z: Vec3, o: Vec3) -> Mat4 {
    let mut m = Mat4::IDENTITY;
    m.make_basis(&x, &y, &z);
    m.set_position(o.x, o.y, o.z);
    m
}

/// The turn of a frame's axes
fn turn(x: Vec3, y: Vec3, z: Vec3) -> Quat {
    let mut q = Quat::IDENTITY;
    q.set_from_rotation_matrix(&frame(x, y, z, Vec3::ZERO));
    q
}

fn neg(v: Vec3) -> Vec3 {
    Vec3::new(-v.x, -v.y, -v.z)
}

/// One straight run of a corridor's crown, in world metres: its middle
/// line from `p0` to `p1` (along x or z), the crown's height, how far out
/// its walls stand (from the middle line, each side, the least along it)
/// and their haunch
struct Run {
    p0: (f64, f64),
    p1: (f64, f64),
    along_x: bool,
    ceil: f64,
    half: f64,
    r: f64,
    /// where it starts, for its kind and side
    seed_tile: usize,
}

impl Game {
    /// The floor tile `t` is the Utility Halls' and bare (a region the moss
    /// has papered is the Lobby's look, and keeps it)
    fn lined(&self, t: usize) -> bool {
        if self.tiles[t] != 0 {
            return false;
        }
        let Some(l) = self.layout.as_ref() else { return false };
        let zn = &l.zones[l.zone_of[t] as usize];
        zn.arch == LINED && self.region_info[self.region_map[t] as usize].flags & RF_MOSS == 0
    }

    /// The haunches of the floor, from the solids: every face of a wall
    /// that stands from the floor to the crown in front of lined floor,
    /// less what covers it, joined along its line
    pub fn haunches(&self, s: &Solids) -> Vec<Haunch> {
        let (w, h) = (self.w, self.h);
        let mut out: Vec<(Haunch, bool, bool)> = vec![];
        const PROBE: f64 = 0.005;
        // what a haunch stops short of (tile units: u0, v0, u1, v1): a door
        // the layout made, its frame and sign; a window's cut, its casing
        let mut gaps: Vec<[f64; 4]> = vec![];
        let door = 0.9 / CELL;
        for t in self.layout_things.iter().filter(|t| t.asset.ends_with("_door") || t.asset.ends_with("_doors")) {
            let (u, v) = (self.to_u(t.x), self.to_v(t.z));
            gaps.push([u - door, v - door, u + door, v + door]);
        }
        let casing = 0.2 / CELL;
        for win in &self.windows {
            for c in &win.cuts {
                gaps.push([self.to_u(c[0]) - casing, self.to_v(c[2]) - casing, self.to_u(c[3]) + casing, self.to_v(c[5]) + casing]);
            }
        }
        for bi in 0..s.count {
            let o = bi * 8;
            let bx = |k: usize| s.boxes[o + k] as f64;
            if bx(7) as i32 != BOX_WALL || bx(4) > 0.01 {
                continue;
            }
            for dir in 0..4 {
                let [dx, dz] = DIRS[dir];
                let along_x = dx == 0;
                let at = match dir {
                    0 => bx(2),
                    1 => bx(0),
                    2 => bx(3),
                    _ => bx(1),
                };
                let (lo, hi) = if along_x { (bx(0), bx(2)) } else { (bx(1), bx(3)) };
                let probe = at + (dx + dz) as f64 * PROBE;
                // its pieces, one a tile in front
                let mut k = lo.floor();
                while k < hi - 1e-9 {
                    let (a, b) = (lo.max(k), hi.min(k + 1.0));
                    k += 1.0;
                    if b - a < 1e-6 {
                        continue;
                    }
                    let mid = (a + b) / 2.0;
                    let (tx, tz) =
                        if along_x { (mid.floor() as i32, probe.floor() as i32) } else { (probe.floor() as i32, mid.floor() as i32) };
                    if tx < 0 || tz < 0 || tx >= w || tz >= h {
                        continue;
                    }
                    let t = (tz * w + tx) as usize;
                    if !self.lined(t) {
                        continue;
                    }
                    let ceil = self.tile_ceil[t] as f64;
                    let Some(r) = haunch_r(ceil) else { continue };
                    if bx(5) < ceil - 0.01 {
                        continue;
                    }
                    // less what stands in front of it at the haunch's height
                    let mut keep: Vec<(f64, bool, f64, bool)> = vec![(a, false, b, false)];
                    for kk in s.start[t]..s.start[t + 1] {
                        let c = s.items[kk as usize] as usize;
                        if c == bi {
                            continue;
                        }
                        let q = c * 8;
                        let cb = |k: usize| s.boxes[q + k] as f64;
                        let kind = cb(7) as i32;
                        if kind == BOX_VOID || kind == BOX_FLOOR || cb(5) < ceil - r || cb(4) > ceil {
                            continue;
                        }
                        let (c_lo, c_hi, x_lo, x_hi) = if along_x { (cb(0), cb(2), cb(1), cb(3)) } else { (cb(1), cb(3), cb(0), cb(2)) };
                        if probe <= x_lo || probe >= x_hi {
                            continue;
                        }
                        let mut next = vec![];
                        for (p, pa, q2, qb) in keep {
                            if c_hi <= p || c_lo >= q2 {
                                next.push((p, pa, q2, qb));
                                continue;
                            }
                            if c_lo > p {
                                next.push((p, pa, c_lo, true));
                            }
                            if c_hi < q2 {
                                next.push((c_hi, true, q2, qb));
                            }
                        }
                        keep = next;
                    }
                    for gp in &gaps {
                        let (c_lo, c_hi, x_lo, x_hi) = if along_x { (gp[0], gp[2], gp[1], gp[3]) } else { (gp[1], gp[3], gp[0], gp[2]) };
                        if probe <= x_lo || probe >= x_hi {
                            continue;
                        }
                        let mut next = vec![];
                        for (p, pa, q2, qb) in keep {
                            if c_hi <= p || c_lo >= q2 {
                                next.push((p, pa, q2, qb));
                                continue;
                            }
                            if c_lo > p {
                                next.push((p, pa, c_lo, false));
                            }
                            if c_hi < q2 {
                                next.push((c_hi, false, q2, qb));
                            }
                        }
                        keep = next;
                    }
                    let (zn_x, zn_z) = (tx, tz);
                    let key = self.wall_face_key(zn_x, zn_z).unwrap_or_default();
                    for (p, pa, q2, qb) in keep {
                        if q2 - p > 1e-4 {
                            out.push((Haunch { t, dir, at, a: p, b: q2, ceil, r, key: key.clone(), meet_a: pa, meet_b: qb }, pa, qb));
                        }
                    }
                }
            }
        }
        // joined along their line: same face, crown and surface, end to end
        out.sort_by(|x, y| {
            (x.0.dir, (x.0.at * 1e4).round() as i64, (x.0.a * 1e6).round() as i64).cmp(&(
                y.0.dir,
                (y.0.at * 1e4).round() as i64,
                (y.0.a * 1e6).round() as i64,
            ))
        });
        let mut joined: Vec<(Haunch, bool, bool)> = vec![];
        for (hh, pa, qb) in out {
            if let Some(last) = joined.last_mut()
                && last.0.dir == hh.dir
                && (last.0.at - hh.at).abs() < 1e-4
                && (last.0.b - hh.a).abs() < 1e-4
                && (last.0.ceil - hh.ceil).abs() < 1e-3
                && last.0.key == hh.key
            {
                last.0.b = hh.b;
                last.2 = qb;
                continue;
            }
            joined.push((hh, pa, qb));
        }
        // an end that meets a wall is mitred where the other wall's haunch
        // meets it (`lay_lining`); an open end stops just short of its wall's
        // end, so its end and the wall's (set into the wall by `SET_IN`) are
        // not one face
        let short = 0.004 / CELL;
        joined
            .into_iter()
            .map(|(mut hh, pa, qb)| {
                hh.meet_a = pa;
                hh.meet_b = qb;
                if !pa {
                    hh.a += short;
                }
                if !qb {
                    hh.b -= short;
                }
                hh
            })
            .filter(|hh| hh.b - hh.a > 0.05 / CELL)
            .collect()
    }

    /// The Utility Halls' haunches and rock bolts (the runs: `lay_runs`)
    pub fn lay_lining(&mut self) {
        let Some(s) = self.solids.take() else { return };
        let list = self.haunches(&s);
        self.solids = Some(s);
        if list.is_empty() {
            return;
        }
        let (w, h) = (self.w as f64, self.h as f64);
        let up = Vec3::new(0.0, 1.0, 0.0);
        // each haunch's face is laid into one mesh a surface and a chunk,
        // its photograph mapped in metres along it and round its arc,
        // carrying on from its wall's (`Patch::ArchUV` maps a face by its
        // normal: round a curve it stretched the photograph, the visual
        // inspector's "smeared band")
        let (c, rr) = circle();
        let span = std::f64::consts::FRAC_PI_2 - 2.0 * CREASE;
        let arc: Vec<(f64, f64, f64, f64)> = (0..=SEG)
            .map(|i| {
                let a = (std::f64::consts::PI - CREASE) - span * i as f64 / SEG as f64;
                // (its angle, up from the crown, out from the wall, how far round)
                (a, -c + rr * a.dsin(), c + rr * a.dcos(), rr * span * i as f64 / SEG as f64)
            })
            .collect();
        let mut faces: Vec<(String, (i64, i64), Face)> = vec![];
        let mut fi: HashMap<(String, (i64, i64)), usize> = HashMap::new();
        let mut bolts: Vec<Mat4> = vec![];
        for hh in &list {
            let [dx, dz] = DIRS[hh.dir];
            let n = Vec3::new(dx as f64, 0.0, dz as f64);
            let u = up.crossed(n);
            // the haunch's ends in world metres along u
            let world = |along: f64| -> Vec3 {
                if dx == 0 {
                    Vec3::new((along - w / 2.0) * CELL, 0.0, (hh.at - h / 2.0) * CELL)
                } else {
                    Vec3::new((hh.at - w / 2.0) * CELL, 0.0, (along - h / 2.0) * CELL)
                }
            };
            let (ea, eb) = (world(hh.a), world(hh.b));
            let ua = ea.x * u.x + ea.z * u.z;
            let ub = eb.x * u.x + eb.z * u.z;
            let (start, meet_s, meet_e) = if ua <= ub { (ea, hh.meet_a, hh.meet_b) } else { (eb, hh.meet_b, hh.meet_a) };
            let len = (ub - ua).abs();
            let o = Vec3::new(start.x - n.x * SET_IN, hh.ceil + SET_IN, start.z - n.z * SET_IN);
            let kind = hh.key.split(':').next().unwrap_or("");
            let [sx, sy] = self.surfaces.scale(kind);
            let chunk = ((self.to_u(start.x) / ARCH_CHUNK).floor() as i64, (self.to_v(start.z) / ARCH_CHUNK).floor() as i64);
            let k = *fi.entry((hh.key.clone(), chunk)).or_insert_with(|| {
                faces.push((hh.key.clone(), chunk, Face::default()));
                faces.len() - 1
            });
            haunch_face(&mut faces[k].2, &arc, o, u, n, len, hh.r, hh.ceil, (meet_s, meet_e), (sx, sy));
            // a row of bolts on the haunch, its middle 45 degrees round
            if len >= 0.6 && hh.r >= 0.2 {
                let nb = ((len / BOLT_EVERY).round() as usize).max(1);
                let c45 = std::f64::consts::FRAC_1_SQRT_2;
                let normal = Vec3::new(n.x * c45, -c45, n.z * c45);
                // the arc's middle, as far out from the wall as down from the crown
                let (c, rr) = circle();
                let off = c - rr * c45;
                let side = u.crossed(normal);
                for k in 0..nb {
                    let along = (k as f64 + 0.5) * len / nb as f64;
                    let p =
                        Vec3::new(start.x + u.x * along + n.x * hh.r * off, hh.ceil - hh.r * off, start.z + u.z * along + n.z * hh.r * off);
                    bolts.push(frame(u, normal, side, p));
                }
            }
        }
        for (key, _, f) in faces {
            let mat = self.arch_mat(&key);
            let m = f.mesh(&mat);
            m.user().arch = true;
            self.world.add(&m);
        }
        if !bolts.is_empty() {
            let geo = crate::assets::props::prop_part_as_modelled("Lining__rockBolt").expect("no prop part Lining__rockBolt");
            // forged steel gone dull, a little rust, sprayed over at its edges
            let mat = lambert!({ color: 0x5d5853 });
            settle_material(&mat);
            self.add_instanced(&geo, &mat, &bolts, &|m: &Mat4| *m);
        }
    }

    /// The straight runs of the lined corridors (see the module)
    fn runs(&self, props: &[PropBox]) -> Vec<Run> {
        let Some(s) = self.solids.as_ref() else { return vec![] };
        let (w, h) = (self.w, self.h);
        let ok = |x: i32, z: i32| x >= 0 && z >= 0 && x < w && z < h && self.lined((z * w + x) as usize);
        // the tubes' panels, across a third of a corridor or more
        let tubes: Vec<(f64, f64)> = self.panels.iter().filter(|p| p.kind == Lamp::Tube).map(|p| (p.wx, p.wz)).collect();
        let mut runs = vec![];
        for along_x in [true, false] {
            // a run tile: lined, the corridor going on both ways along it,
            // and (along z) not where a run along x passes
            let run_tile = |x: i32, z: i32| -> bool {
                if !ok(x, z) {
                    return false;
                }
                let t = (z * w + x) as usize;
                let (fx, fz) = (ok(x - 1, z) && ok(x + 1, z), ok(x, z - 1) && ok(x, z + 1));
                let through = if along_x { fx } else { fz && !fx };
                if !through {
                    return false;
                }
                let ceil = self.tile_ceil[t] as f64;
                // nothing standing up into the crown's runs in its middle
                // (the corridor's own walls stand further out): a lintel, a
                // jamb, a barricade, a prop hung from the crown
                let (cx, cz) = (self.wx(x), self.wz(z));
                let half_c = CELL / 2.0;
                let mid = RUN_STRIP / CELL;
                let (u, v) = (self.to_u(cx), self.to_v(cz));
                let (su0, sv0, su1, sv1) =
                    if along_x { (u - 0.5, v - mid, u + 0.5, v + mid) } else { (u - mid, v - 0.5, u + mid, v + 0.5) };
                for kk in s.start[t]..s.start[t + 1] {
                    let q = s.items[kk as usize] as usize * 8;
                    let cb = |k: usize| s.boxes[q + k] as f64;
                    let kind = cb(7) as i32;
                    if kind == BOX_VOID || kind == BOX_FLOOR || cb(5) < ceil - 0.6 {
                        continue;
                    }
                    if cb(2) > su0 && cb(0) < su1 && cb(3) > sv0 && cb(1) < sv1 {
                        return false;
                    }
                }
                // a barricade's slab across the way (`Game::barricades`: not a
                // solid, a body is stopped by the slab itself)
                if self.barricades.iter().any(|b| b.tile == t) {
                    return false;
                }
                for p in props {
                    let bb = &p.bb;
                    if bb.max.y > ceil - 0.6
                        && bb.max.x > cx - half_c
                        && bb.min.x < cx + half_c
                        && bb.max.z > cz - half_c
                        && bb.min.z < cz + half_c
                    {
                        return false;
                    }
                }
                // a tube's panel: the runs would cross it
                !tubes.iter().any(|&(px, pz)| (px - cx).abs() < half_c && (pz - cz).abs() < half_c)
            };
            let (n_outer, n_inner) = if along_x { (h, w) } else { (w, h) };
            for o in 0..n_outer {
                let mut k = 0;
                while k < n_inner {
                    let tile = |i: i32| if along_x { (i, o) } else { (o, i) };
                    let (x0, z0) = tile(k);
                    if !run_tile(x0, z0) {
                        k += 1;
                        continue;
                    }
                    let ceil = self.tile_ceil[(z0 * w + x0) as usize] as f64;
                    let mut e = k;
                    while e + 1 < n_inner && {
                        let (x, z) = tile(e + 1);
                        run_tile(x, z) && (self.tile_ceil[(z * w + x) as usize] as f64 - ceil).abs() < 1e-3
                    } {
                        e += 1;
                    }
                    let len_tiles = e - k + 1;
                    let start = k;
                    k = e + 1;
                    if len_tiles < 3 {
                        continue;
                    }
                    let Some(r) = haunch_r(ceil) else { continue };
                    // how far out its walls stand, the least along it
                    let mut half = f64::INFINITY;
                    for i in start..=e {
                        let (x, z) = tile(i);
                        let (u, v) = (x as f64 + 0.5, z as f64 + 0.5);
                        let y = ceil - 0.3;
                        let dirs = if along_x { [2, 3] } else { [0, 1] };
                        for d in dirs {
                            half = half.min(ray_axis(s, u, v, d, 1.0, y) * CELL);
                        }
                    }
                    if !half.is_finite() || half < 0.6 {
                        continue;
                    }
                    let (xs, zs) = tile(start);
                    let (xe, ze) = tile(e);
                    let trim = 0.3;
                    let (p0, p1) = if along_x {
                        ((self.wx(xs) - CELL / 2.0 + trim, self.wz(zs)), (self.wx(xe) + CELL / 2.0 - trim, self.wz(ze)))
                    } else {
                        ((self.wx(xs), self.wz(zs) - CELL / 2.0 + trim), (self.wx(xe), self.wz(ze) + CELL / 2.0 - trim))
                    };
                    runs.push(Run { p0, p1, along_x, ceil, half, r, seed_tile: (zs * w + xs) as usize });
                }
            }
        }
        runs
    }

    /// The runs on the lined corridors' crowns: a tray of cables, a pair of
    /// pipes, or both, each held to its rules (`assets/src/props/service_run/`)
    pub fn lay_runs(&mut self, props: &[PropBox]) {
        let runs = self.runs(props);
        if runs.is_empty() {
            return;
        }
        let up = Vec3::new(0.0, 1.0, 0.0);
        let mut trays: Vec<Mat4> = vec![];
        let mut risers: Vec<Mat4> = vec![];
        let mut struts: Vec<Mat4> = vec![];
        let mut rods: Vec<(f64, f64, f64, f64)> = vec![];
        let mut pipes: Vec<(usize, Mat4)> = vec![];
        let mut clamps: Vec<(usize, Mat4)> = vec![];
        let mut couplings: Vec<(usize, Mat4)> = vec![];
        let mut elbows: Vec<(usize, Mat4)> = vec![];
        // the two pipes: 4 in and 2 in, their outside diameters' halves (m)
        const PIPE_R: [f64; 2] = [0.0572, 0.0302];
        for run in &runs {
            let (mx, mz) = ((run.p0.0 + run.p1.0) / 2.0, (run.p0.1 + run.p1.1) / 2.0);
            if self.place_by_layout("service_run", "run", mx, mz).is_none() {
                continue;
            }
            let mut sm = SplitMix64::new(self.current_seed, RUN_SALT ^ (run.seed_tile as u64).wrapping_mul(0x9E37_79B9));
            let pick = sm.next();
            let (tray, pair) = if pick < 0.5 {
                (true, true)
            } else if pick < 0.75 {
                (false, true)
            } else {
                (true, false)
            };
            let side = if sm.next() < 0.5 { 1.0 } else { -1.0 };
            let big_inside = sm.next() < 0.5;
            // along it (u) and across it (n), right-handed: u = y × n
            let n = if run.along_x { Vec3::new(0.0, 0.0, 1.0) } else { Vec3::new(-1.0, 0.0, 0.0) };
            let u = up.crossed(n);
            let (a, b) = if run.along_x { (run.p0.0, run.p1.0) } else { (run.p0.1, run.p1.1) };
            let (lo, hi) = if u.x + u.z > 0.0 { (a.min(b), a.max(b)) } else { (a.max(b), a.min(b)) };
            let len = (hi - lo).abs();
            let at = |along: f64, across: f64, y: f64| -> Vec3 {
                let base = if run.along_x { Vec3::new(lo, y, run.p0.1) } else { Vec3::new(run.p0.0, y, lo) };
                Vec3::new(base.x + u.x * along + n.x * across, y, base.z + u.z * along + n.z * across)
            };
            // where the haunch stands over a point `across` from the middle
            let crown = |across: f64| haunch_y(run.ceil, run.r, run.half - across.abs());
            let hangers: Vec<f64> = {
                let k = ((len - 0.8) / HANGER_EVERY).ceil().max(1.0) as usize;
                (0..=k).map(|i| 0.4 + (len - 0.8) * i as f64 / k as f64).collect()
            };
            let strut_at = |inner: f64, outer: f64, y_top: f64, sd: f64, struts: &mut Vec<Mat4>, rods: &mut Vec<(f64, f64, f64, f64)>| {
                let width = outer - inner;
                let mid = sd * (inner + outer) / 2.0;
                for &al in &hangers {
                    let p = at(al, mid, y_top);
                    struts.push(Mat4::compose(&p, &turn(u, up, n), &Vec3::new(1.0, 1.0, width)));
                    for e in [-0.46, 0.46] {
                        let q = at(al, mid + e * width, y_top);
                        rods.push((q.x, q.z, y_top - 0.041, crown(mid + e * width) + 0.01));
                    }
                }
            };
            if tray {
                let sd = side;
                let (inner, outer) = (RUN_INNER, RUN_INNER + 0.31);
                // its top under the haunch at its outer edge
                let top = (run.ceil - 0.10).min(crown(outer) - 0.03);
                // (its rails 92 mm tall: `lining.py`)
                let y = top - 0.092;
                if y - 0.041 >= RUN_HEADROOM && run.half > outer + 0.05 {
                    let mid = sd * (inner + outer) / 2.0;
                    let mut along = 0.0;
                    while along < len - 1e-6 {
                        let l = (len - along).min(1.0);
                        let p = at(along, mid, y);
                        trays.push(Mat4::compose(&p, &turn(u, up, n), &Vec3::new(l, 1.0, 1.0)));
                        along += 1.0;
                    }
                    strut_at(inner - 0.04, outer + 0.04, y, sd, &mut struts, &mut rods);
                    // its cables turn up into the crown at its ends
                    let lift = y + 0.02;
                    let rise = crown(mid) + 0.02 - lift;
                    if rise > 0.0 {
                        risers.push(Mat4::compose(&at(len, mid, lift), &turn(up, neg(u), n), &Vec3::new(rise, 1.0, 1.0)));
                        risers.push(Mat4::compose(&at(0.0, mid, lift), &turn(up, u, neg(n)), &Vec3::new(rise, 1.0, 1.0)));
                    }
                }
            }
            if pair {
                let sd = if tray { -side } else { side };
                let (ri, ro) = if big_inside { (PIPE_R[0], PIPE_R[1]) } else { (PIPE_R[1], PIPE_R[0]) };
                let (ci, co) = (RUN_INNER + 0.02 + ri, RUN_INNER + 0.02 + 2.0 * ri + 0.07 + ro);
                let outer = co + ro + 0.06;
                let y_top =
                    (run.ceil - 0.10 - 2.0 * ri.max(ro)).min(crown(co + ro) - 0.03 - 2.0 * ro).min(crown(ci + ri) - 0.03 - 2.0 * ri);
                if y_top - 0.041 >= RUN_HEADROOM && run.half > outer + 0.05 {
                    for (c, pr) in [(ci, ri), (co, ro)] {
                        let kind = if pr > 0.04 { 0 } else { 1 };
                        let yc = y_top + pr;
                        let across = sd * c;
                        // the pipe's length along the run, its elbows up at its ends
                        // (a long-radius elbow's centre to end: 1.5 times
                        // its size, 2.6 times its outside radius)
                        let bend = 2.6 * pr;
                        let p = at(bend, across, yc);
                        pipes.push((kind, Mat4::compose(&p, &turn(u, up, n), &Vec3::new(len - 2.0 * bend, pr, pr))));
                        let crown_here = crown(across);
                        for (e_at, back) in [(bend, true), (len - bend, false)] {
                            // the elbow: from along +u, turning up; at the
                            // start it runs the other way
                            let dir = if back { neg(u) } else { u };
                            let side_ax = dir.crossed(up);
                            let q = at(e_at, across, yc);
                            elbows.push((kind, Mat4::compose(&q, &turn(dir, up, side_ax), &Vec3::new(pr, pr, pr))));
                            // and up into the crown
                            let riser = at(e_at + if back { -bend } else { bend }, across, yc + bend);
                            let rise = crown_here + 0.02 - riser.y;
                            if rise > 0.0 {
                                pipes.push((kind, Mat4::compose(&riser, &turn(up, neg(u), n), &Vec3::new(rise, pr, pr))));
                            }
                        }
                        let mut cpl = PIPE_LENGTH * (0.3 + 0.4 * sm.next());
                        while cpl < len - bend - 0.3 {
                            // a joint clear of the clamps
                            let here = match hangers.iter().find(|&&hg| (hg - cpl).abs() < 0.2) {
                                Some(&hg) => hg + 0.3,
                                None => cpl,
                            };
                            if here < len - bend - 0.1 {
                                couplings.push((kind, Mat4::compose(&at(here, across, yc), &turn(u, up, n), &Vec3::new(pr, pr, pr))));
                            }
                            cpl += PIPE_LENGTH;
                        }
                        for &al in &hangers {
                            clamps.push((kind, Mat4::compose(&at(al, across, yc), &turn(u, up, n), &Vec3::new(pr, pr, pr))));
                        }
                    }
                    strut_at(RUN_INNER, outer, y_top, sd, &mut struts, &mut rods);
                }
            }
        }
        let part = |name: &str| crate::assets::props::prop_part_as_modelled(name).unwrap_or_else(|| panic!("no prop part {name}"));
        // galvanized steel, dulled; black PVC; the pipes in their primer
        let galv = lambert!({ color: 0x656b6b });
        settle_material(&galv);
        if !trays.is_empty() {
            self.add_instanced(&part("Run__tray"), &galv, &trays, &|m: &Mat4| *m);
            let pvc = lambert!({ color: 0x1e1f1f });
            settle_material(&pvc);
            self.add_instanced(&part("Run__cables"), &pvc, &trays, &|m: &Mat4| *m);
            self.add_instanced(&part("Run__cables"), &pvc, &risers, &|m: &Mat4| *m);
        }
        if !struts.is_empty() {
            self.add_instanced(&part("Run__strut"), &galv, &struts, &|m: &Mat4| *m);
        }
        if !rods.is_empty() {
            let geo = self.unit_box_geo.clone();
            let mat = self.mats.metal.clone();
            self.add_instanced(&geo, &mat, &rods, &|&(x, z, y0, y1): &(f64, f64, f64, f64)| {
                Mat4::compose(&Vec3::new(x, (y0 + y1) / 2.0, z), &Quat::IDENTITY, &Vec3::new(0.01, (y1 - y0).max(0.01), 0.01))
            });
        }
        // the pipes: a radius across, a metre along x
        let pipe_geo = part("Run__pipe");
        for (kind, colour) in [(0usize, 0x5d5a52u32), (1, 0x6b665b)] {
            let mat = lambert!({ color: colour });
            settle_material(&mat);
            let list: Vec<Mat4> = pipes.iter().filter(|(k, _)| *k == kind).map(|(_, m)| *m).collect();
            self.add_instanced(&pipe_geo, &mat, &list, &|m: &Mat4| *m);
            let list: Vec<Mat4> = elbows.iter().filter(|(k, _)| *k == kind).map(|(_, m)| *m).collect();
            self.add_instanced(&part("Run__elbow"), &mat, &list, &|m: &Mat4| *m);
        }
        let all = |v: &[(usize, Mat4)]| v.iter().map(|(_, m)| *m).collect::<Vec<Mat4>>();
        // the couplings' own enamel, an orange gone dull with dust (the
        // references' coupling in shade)
        let iron = lambert!({ color: 0x523322 });
        settle_material(&iron);
        self.add_instanced(&part("Run__coupling"), &iron, &all(&couplings), &|m: &Mat4| *m);
        self.add_instanced(&part("Run__clamp"), &galv, &all(&clamps), &|m: &Mat4| *m);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_haunch_springs_above_every_door() {
        // the lowest Utility Halls' crown has a small one, a lower one none
        assert_eq!(haunch_r(2.30), None);
        let r = haunch_r(2.35).expect("a crown 2.35 m up has a haunch");
        assert!(2.35 - r >= HAUNCH_SPRING - 1e-9, "{r}");
        let top = haunch_r(2.65).expect("the tallest crown has one");
        assert!((top - HAUNCH_R_MAX).abs() < 1e-9, "{top}");
    }

    #[test]
    fn a_haunch_meets_its_wall_and_crown_at_a_crease() {
        let (ceil, r) = (2.6, 0.4);
        assert!((haunch_y(ceil, r, 0.0) - (ceil - r)).abs() < 1e-9, "a radius down at the wall");
        assert!((haunch_y(ceil, r, r) - ceil).abs() < 1e-9, "at the crown a radius out");
        assert_eq!(haunch_y(ceil, r, r + 0.3), ceil);
        // it rises all the way, and leaves the wall at the crease, not along it
        let mut last = haunch_y(ceil, r, 0.0);
        for k in 1..=40 {
            let y = haunch_y(ceil, r, r * k as f64 / 40.0);
            assert!(y > last, "{k}");
            last = y;
        }
        let slope = (haunch_y(ceil, r, 1e-4) - haunch_y(ceil, r, 0.0)) / 1e-4;
        assert!((slope - (std::f64::consts::FRAC_PI_2 - CREASE).dtan()).abs() < 0.05, "{slope}");
    }
}
