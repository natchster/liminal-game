//! Placement: the last word on where props sit. Runs once per level after
//! `settle_props`, over everything the placers put in the world,
//! using each mesh's oriented box:
//!
//!   1. floor things rest on the floor, or on whatever is under them (a table,
//!      a crate): nothing sunk into the carpet, nothing hovering
//!   2. nothing is buried in a wall, pillar or jamb: it is pushed out along
//!      the face it went through, or where that only pushes it into another
//!      face (a corner, a gap too narrow for it) moved to the nearest clear
//!      spot, along its own wall if it hangs on one, or left out
//!   3. no two props interpenetrate: the smaller one steps aside, and if
//!      there is no room it is left out (scenery only)
//!
//! Gameplay objects (pickups, the exit, switches, notes...) are never moved
//! sideways or removed; others yield to them. Wall- and ceiling-mounted things
//! keep their height. Pure geometry — the caller supplies the level queries.
//!
//! Every expression keeps a fixed evaluation order (and [`hypot`] is the
//! compensated one), so the result is reproducible to the last bit.
use crate::assets::props::or_num;
use crate::base::numeric::{hypot, nan_max, nan_min};
use engine::prelude::*;
use std::collections::{HashMap, HashSet};

type V3 = [f64; 3];
const I3: [V3; 3] = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
#[inline]
fn dot(a: &V3, b: &V3) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
#[inline]
fn cross(a: &V3, b: &V3) -> V3 {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}
#[inline]
fn rad(a: &[V3; 3], h: &V3, l: &V3) -> f64 {
    dot(&a[0], l).abs() * h[0] + dot(&a[1], l).abs() * h[1] + dot(&a[2], l).abs() * h[2]
}

/// The result of [`sat_depth`]: penetration depth and its axis.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sat {
    pub depth: f64,
    pub axis: Option<V3>,
}

/// Minimum penetration of two oriented boxes
/// and the axis (from a to b) it is along.
pub fn sat_depth(ca: &V3, a: &[V3; 3], ha: &V3, cb: &V3, b: &[V3; 3], hb: &V3) -> Sat {
    let t = [cb[0] - ca[0], cb[1] - ca[1], cb[2] - ca[2]];
    let mut best = f64::INFINITY;
    let mut axis: Option<V3> = None;
    let mut test = |l: &V3| -> bool {
        let len = hypot(l);
        if len < 1e-6 {
            return true;
        }
        let n = [l[0] / len, l[1] / len, l[2] / len];
        let o = rad(a, ha, &n) + rad(b, hb, &n) - dot(&t, &n).abs();
        if o < best {
            best = o;
            let s = if dot(&t, &n) < 0.0 { -1.0 } else { 1.0 };
            axis = Some([n[0] * s, n[1] * s, n[2] * s]);
        }
        o > 0.0
    };
    let done = 'sep: {
        for x in a {
            if !test(x) {
                break 'sep true;
            }
        }
        for y in b {
            if !test(y) {
                break 'sep true;
            }
        }
        for x in a {
            for y in b {
                if !test(&cross(x, y)) {
                    break 'sep true;
                }
            }
        }
        false
    };
    let _ = done;
    Sat { depth: best, axis }
}

/// One oriented box of `mesh_boxes`: centre, unit axes, half extents (world).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MeshBox {
    pub c: V3,
    pub a: [V3; 3],
    pub h: V3,
}

/// Oriented boxes of the solid meshes under `root`
/// (world space). Planes, circles, rings, sheets thinner than 1.5 mm, see-through
/// (opacity < 0.6), additive and depth-write-off pieces are left out.
pub fn mesh_boxes(root: &Obj) -> Vec<MeshBox> {
    let mut out = vec![];
    root.update_matrix_world(true);
    root.traverse(&mut |c: &Obj| {
        if !c.is_mesh() || c.is_instanced() || !c.visible() {
            return;
        }
        let Some(g) = c.try_geo() else { return };
        let bb = g.bounding_box();
        if bb.is_empty() {
            return;
        }
        let t = g.type_name();
        if t == "plane" || t == "circle" || t == "ring" {
            return;
        }
        if let Some(mt) = c.try_mat() {
            let m = mt.r();
            if m.transparent && m.opacity < 0.6 || m.blending == Blending::Additive || !m.depth_write {
                return;
            }
        }
        let e = c.matrix_world().e;
        let mut a = [[0.0; 3]; 3];
        let mut s = [0.0; 3];
        for i in 0..3 {
            let (x, y, z) = (e[i * 4], e[i * 4 + 1], e[i * 4 + 2]);
            let l = or_num(hypot(&[x, y, z]), 1.0);
            a[i] = [x / l, y / l, z / l];
            s[i] = l;
        }
        let h = [(bb.max.x - bb.min.x) / 2.0 * s[0], (bb.max.y - bb.min.y) / 2.0 * s[1], (bb.max.z - bb.min.z) / 2.0 * s[2]];
        if nan_min(nan_min(h[0], h[1]), h[2]) < 0.0015 {
            return;
        } // a sheet: a decal in all but name
        let mut ctr = bb.get_center();
        ctr.apply_matrix4(&c.matrix_world());
        out.push(MeshBox { c: [ctr.x, ctr.y, ctr.z], a, h });
    });
    out
}

fn low_y(m: &MeshBox) -> f64 {
    m.c[1] - rad(&m.a, &m.h, &I3[1])
}
fn high_y(m: &MeshBox) -> f64 {
    m.c[1] + rad(&m.a, &m.h, &I3[1])
}
fn aabb(ms: &[MeshBox]) -> [f64; 6] {
    let mut b = [f64::INFINITY, f64::INFINITY, f64::INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY];
    for m in ms {
        for k in 0..3 {
            let r = rad(&m.a, &m.h, &I3[k]);
            b[k] = nan_min(b[k], m.c[k] - r);
            b[k + 3] = nan_max(b[k + 3], m.c[k] + r);
        }
    }
    b
}

/// One top-level prop to settle: the object, whether it is a protected
/// gameplay object, and its placer's name.
#[derive(Clone, Debug)]
pub struct PlaceObject {
    pub obj: Obj,
    pub protect: bool,
    pub placer: String,
}

/// The penetration `settle_placement` tolerates by default (metres).
pub const DEFAULT_TOL: f64 = 0.015;

/// The options of [`resolve_placement`].
pub struct PlacementOpts<'a> {
    /// top-level props to settle
    pub objects: Vec<PlaceObject>,
    /// `bb` → the `[x0, y0, z0, x1, y1, z1]` world boxes near it, distinct and
    /// in first-seen order (see [`WallGrid`])
    pub walls: &'a dyn Fn(&[f64; 6]) -> Vec<[f64; 6]>,
    /// `(x, z)` → ceiling height
    pub ceil_at: &'a dyn Fn(f64, f64) -> f64,
    /// `(x, z)` → true inside the pool basin (left alone)
    pub in_pool: Option<&'a dyn Fn(f64, f64) -> bool>,
    /// Penetration tolerated (metres); normally [`DEFAULT_TOL`]
    pub tol: f64,
}

/// Counts from one placement pass.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PlacementStats {
    pub props: usize,
    pub lifted: usize,
    pub dropped: usize,
    pub stacked: usize,
    pub unburied: usize,
    pub nudged: usize,
    pub removed: usize,
    pub resettled: usize,
    pub skipped: usize,
}

struct Prop {
    obj: Obj,
    protect: bool,
    placer: String,
    ms: Vec<MeshBox>,
    bb: [f64; 6],
    lo: f64,
    hi: f64,
    vol: f64,
    mv: V3,
    wall_contact: bool,
    ceiling: bool,
    floor: bool,
    wall_axis: Option<[f64; 2]>,
    remove: bool,
}

fn shift(p: &mut Prop, d: V3) {
    for m in p.ms.iter_mut() {
        m.c[0] += d[0];
        m.c[1] += d[1];
        m.c[2] += d[2];
    }
    for k in 0..3 {
        p.bb[k] += d[k];
        p.bb[k + 3] += d[k];
    }
    p.lo += d[1];
    p.hi += d[1];
}

#[derive(Clone, Copy, Debug)]
struct Hit {
    depth: f64,
    axis: Option<V3>,
}

fn wall_hits(p: &Prop, walls: &dyn Fn(&[f64; 6]) -> Vec<[f64; 6]>) -> Option<Hit> {
    let mut worst: Option<Hit> = None;
    let q = [p.bb[0] - 0.05, p.bb[1], p.bb[2] - 0.05, p.bb[3] + 0.05, p.bb[4], p.bb[5] + 0.05];
    for w in walls(&q) {
        let wc = [(w[0] + w[3]) / 2.0, (w[1] + w[4]) / 2.0, (w[2] + w[5]) / 2.0];
        let wh = [(w[3] - w[0]) / 2.0, (w[4] - w[1]) / 2.0, (w[5] - w[2]) / 2.0];
        for m in &p.ms {
            let r = sat_depth(&m.c, &m.a, &m.h, &wc, &I3, &wh);
            if worst.is_none() || r.depth > worst.unwrap().depth {
                worst = Some(Hit { depth: r.depth, axis: r.axis });
            }
        }
    }
    worst
}

pub fn resolve_placement(o: &PlacementOpts) -> PlacementStats {
    let tol = o.tol;
    let walls = o.walls;
    let mut st = PlacementStats::default();
    let mut pp: Vec<Prop> = vec![];
    for it in &o.objects {
        if it.obj.r().user.floats {
            st.skipped += 1;
            continue;
        } // balloons and the like hover on purpose
        let ms = mesh_boxes(&it.obj);
        if ms.is_empty() {
            continue;
        }
        let bb = aabb(&ms);
        let (cx, cz) = ((bb[0] + bb[3]) / 2.0, (bb[2] + bb[5]) / 2.0);
        if let Some(f) = o.in_pool {
            if f(cx, cz) {
                st.skipped += 1;
                continue;
            }
        }
        let (mut lo, mut hi) = (f64::INFINITY, f64::NEG_INFINITY);
        for m in &ms {
            lo = nan_min(lo, low_y(m));
            hi = nan_max(hi, high_y(m));
        }
        let vol = (bb[3] - bb[0]) * (bb[4] - bb[1]) * (bb[5] - bb[2]);
        pp.push(Prop {
            obj: it.obj.clone(),
            protect: it.protect,
            placer: it.placer.clone(),
            ms,
            bb,
            lo,
            hi,
            vol,
            mv: [0.0; 3],
            wall_contact: false,
            ceiling: false,
            floor: false,
            wall_axis: None,
            remove: false,
        });
    }
    st.props = pp.len();
    // --- classify: mounted on a wall / the ceiling, or standing on the floor
    for p in pp.iter_mut() {
        let (cx, cz) = ((p.bb[0] + p.bb[3]) / 2.0, (p.bb[2] + p.bb[5]) / 2.0);
        let ceil = (o.ceil_at)(cx, cz);
        let w = wall_hits(p, walls);
        p.wall_contact = matches!(w, Some(h) if h.depth > -0.04);
        p.ceiling = p.hi > ceil - 0.12 && p.lo > 1.2;
        p.floor = !p.ceiling && p.lo < 0.6 && !(p.wall_contact && p.lo > 0.08);
        // a wall-mounted thing may only ever slide along its wall
        if p.wall_contact && !p.floor {
            if let Some(ax) = w.and_then(|h| h.axis) {
                let l = hypot(&[ax[0], ax[2]]);
                if l > 0.5 {
                    p.wall_axis = Some([ax[0] / l, ax[2] / l]);
                }
            }
        }
    }
    // --- 1. out of the walls (horizontally, along the face it went through)
    for p in pp.iter_mut() {
        for _pass in 0..3 {
            let Some(w) = wall_hits(p, walls) else { break };
            if w.depth <= tol {
                break;
            }
            let Some(ax) = w.axis else { break }; // from the prop towards the wall
            let (mut dx, mut dz) = (-ax[0], -ax[2]);
            let l = hypot(&[dx, dz]);
            if l < 0.3 {
                break;
            } // a floor/ceiling slab: nothing sideways helps
            dx /= l;
            dz /= l;
            let d = (w.depth + 0.004) / l;
            if p.protect && d > 0.25 {
                break;
            } // gameplay things stay where the game put them
            shift(p, [dx * d, 0.0, dz * d]);
            p.mv[0] += dx * d;
            p.mv[2] += dz * d;
            st.unburied += 1;
        }
    }
    // --- 1b. still in a wall (a gap narrower than the prop, a corner, between
    // two jambs): pushing out of one face only pushed it into the other. Look for
    // the nearest clear spot instead, along its own wall for a wall-mounted
    // thing. Scenery with nowhere to go is dropped; a wall thing only when it is
    // deep in (a few centimetres into the plaster reads as flush).
    const DIRS8: [[f64; 2]; 8] =
        [[1.0, 0.0], [-1.0, 0.0], [0.0, 1.0], [0.0, -1.0], [0.7071, 0.7071], [-0.7071, 0.7071], [0.7071, -0.7071], [-0.7071, -0.7071]];
    for p in pp.iter_mut() {
        if p.protect {
            continue;
        }
        let Some(w0) = wall_hits(p, walls) else { continue };
        if w0.depth <= tol {
            continue;
        }
        let dirs: Vec<[f64; 2]> = match p.wall_axis {
            Some(wa) => vec![[-wa[1], wa[0]], [wa[1], -wa[0]]],
            None => DIRS8.to_vec(),
        };
        let mut found: Option<V3> = None;
        let mut r = 0.1;
        while r < 0.85 && found.is_none() {
            for &[ux, uz] in &dirs {
                let d = [ux * r, 0.0, uz * r];
                shift(p, d);
                let w = wall_hits(p, walls);
                shift(p, [-d[0], 0.0, -d[2]]);
                if w.is_none_or(|w| w.depth <= tol) {
                    found = Some(d);
                    break;
                }
            }
            r += 0.1;
        }
        if let Some(f) = found {
            shift(p, f);
            p.mv[0] += f[0];
            p.mv[2] += f[2];
            st.unburied += 1;
        } else if p.wall_axis.is_none() || w0.depth > 0.08 {
            p.remove = true;
            st.removed += 1;
        }
    }
    // --- 2. rest on the floor or on what is below
    let mut by_volume: Vec<usize> = (0..pp.len()).collect();
    by_volume.sort_by(|&a, &b| ordering_of(pp[b].vol - pp[a].vol));
    rest(&mut pp, &by_volume, false, &mut st);
    // --- 3. props apart: protected and bigger first, the rest yield
    let mut order: Vec<usize> = (0..pp.len()).collect();
    order.sort_by(|&a, &b| {
        let d = pp[b].protect as i32 - pp[a].protect as i32;
        if d != 0 {
            ordering_of(d as f64)
        } else {
            ordering_of(pp[b].vol - pp[a].vol)
        }
    });
    let mut grid: HashMap<i64, Vec<usize>> = HashMap::new();
    for &pi in &order {
        if pp[pi].remove {
            continue;
        }
        let mut c = if pp[pi].protect { None } else { clash(&pp, pi, &grid, tol) };
        let mut tries = 0;
        while let Some(cc) = c {
            if tries >= 6 {
                break;
            }
            // step out along the separating axis flattened onto the floor; when
            // that axis is vertical (one on top of the other), go sideways, away
            // from the other prop's centre (every other try at right angles to that)
            let ax = cc.axis.unwrap_or([0.0; 3]);
            let (mut ux, mut uz) = (-ax[0], -ax[2]);
            let lh = hypot(&[ux, uz]);
            let mut dist;
            let p = &pp[pi];
            if lh >= 0.3 {
                ux /= lh;
                uz /= lh;
                dist = (cc.depth + 0.01) / lh;
            } else {
                let q = &pp[cc.q];
                ux = (p.bb[0] + p.bb[3]) - (q.bb[0] + q.bb[3]);
                uz = (p.bb[2] + p.bb[5]) - (q.bb[2] + q.bb[5]);
                let l = or_num(hypot(&[ux, uz]), 1.0);
                ux /= l;
                uz /= l;
                dist = nan_min(p.bb[3] - p.bb[0], p.bb[5] - p.bb[2]) * 0.5 + cc.depth + 0.01;
            }
            if tries % 2 == 1 {
                let t = ux;
                ux = -uz;
                uz = t;
            }
            if let Some([wx, wz]) = p.wall_axis {
                // keep it on its wall: drop the part of the step that leaves the wall
                let k = ux * wx + uz * wz;
                ux -= k * wx;
                uz -= k * wz;
                let l = hypot(&[ux, uz]);
                if l < 0.2 {
                    ux = -wz;
                    uz = wx;
                    if tries % 2 == 1 {
                        ux = -ux;
                        uz = -uz;
                    }
                } else {
                    ux /= l;
                    uz /= l;
                }
                dist = cc.depth + 0.02 + tries as f64 * 0.1;
            }
            dist *= 1.0 + tries as f64 * 0.4;
            if dist > 1.5 {
                break;
            }
            let d = [ux * dist, 0.0, uz * dist];
            shift(&mut pp[pi], d);
            let w = wall_hits(&pp[pi], walls);
            if matches!(w, Some(w) if w.depth > tol) {
                shift(&mut pp[pi], [-d[0], 0.0, -d[2]]);
                tries += 1;
                continue;
            }
            pp[pi].mv[0] += d[0];
            pp[pi].mv[2] += d[2];
            st.nudged += 1;
            c = clash(&pp, pi, &grid, tol);
            tries += 1;
        }
        if c.is_some() {
            pp[pi].remove = true;
            st.removed += 1;
            continue;
        }
        add_to_grid(&pp[pi], pi, &mut grid);
    }
    // --- things that stood on something that moved or went: settle them again
    rest(&mut pp, &by_volume, true, &mut st);
    // --- apply
    for p in &pp {
        if p.remove {
            p.obj.remove_from_parent();
            continue;
        }
        if p.mv[0] != 0.0 || p.mv[1] != 0.0 || p.mv[2] != 0.0 {
            // the object's own parent is the world (unrotated), so a world shift is a local one
            {
                let mut pos = p.obj.position();
                pos.x += p.mv[0];
                pos.y += p.mv[1];
                pos.z += p.mv[2];
            }
            p.obj.update_matrix_world(true);
        }
    }
    st
}

/// A signed comparator value as an Ordering (NaN counts as equal).
fn ordering_of(v: f64) -> std::cmp::Ordering {
    if v < 0.0 {
        std::cmp::Ordering::Less
    } else if v > 0.0 {
        std::cmp::Ordering::Greater
    } else {
        std::cmp::Ordering::Equal
    }
}

/// highest top of a bigger prop under p's footprint, below p's middle
fn under(pp: &[Prop], by_volume: &[usize], pi: usize) -> Option<f64> {
    let p = &pp[pi];
    let mut top: Option<f64> = None;
    for &qi in by_volume {
        let q = &pp[qi];
        if qi == pi || q.remove || q.vol < p.vol * 0.8 {
            continue;
        }
        if q.bb[0] > p.bb[3] - 0.02 || q.bb[3] < p.bb[0] + 0.02 || q.bb[2] > p.bb[5] - 0.02 || q.bb[5] < p.bb[2] + 0.02 {
            continue;
        }
        if q.hi > (p.lo + p.hi) / 2.0 || q.hi < p.lo - 0.6 {
            continue;
        }
        if top.is_none() || q.hi > top.unwrap() {
            top = Some(q.hi);
        }
    }
    top
}

fn rest(pp: &mut [Prop], by_volume: &[usize], final_: bool, st: &mut PlacementStats) {
    for &pi in by_volume {
        {
            let p = &pp[pi];
            if p.remove || p.protect && p.lo > 0.0 {
                continue;
            } // pickups hover on purpose
        }
        // on the last pass anything left hovering with nothing under it (its
        // cabinet was moved or left out) comes down too, unless a wall or the
        // ceiling holds it
        let (floor, lo, wall_contact) = {
            let p = &pp[pi];
            (p.floor, p.lo, p.wall_contact)
        };
        let hovering = final_ && !floor && !pp[pi].ceiling && !wall_contact && lo > 0.02;
        if !floor && !hovering {
            continue;
        }
        let sup = if lo > 0.02 { under(pp, by_volume, pi) } else { None };
        let mut target = 0.0;
        if let Some(s) = sup {
            if s > 0.03 {
                target = s;
            }
        }
        let dy = target - lo;
        if dy.abs() < 0.006 {
            continue;
        }
        if dy < 0.0 && target == 0.0 && lo > 0.02 && wall_contact {
            continue;
        } // hung on a wall
        let p = &mut pp[pi];
        shift(p, [0.0, dy, 0.0]);
        p.mv[1] += dy;
        if final_ {
            st.resettled += 1;
        } else if dy > 0.0 {
            st.lifted += 1;
        } else if target > 0.0 {
            st.stacked += 1;
        } else {
            st.dropped += 1;
        }
    }
}

#[inline]
fn cell_of(x: f64) -> i64 {
    (x / 1.5).floor() as i64
}

fn add_to_grid(p: &Prop, pi: usize, grid: &mut HashMap<i64, Vec<usize>>) {
    for x in cell_of(p.bb[0])..=cell_of(p.bb[3]) {
        for z in cell_of(p.bb[2])..=cell_of(p.bb[5]) {
            grid.entry(x * 65536 + z).or_default().push(pi);
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Clash {
    depth: f64,
    axis: Option<V3>,
    q: usize,
}

/// Two objects from the same placer are usually one composition (flowers in a
/// vase, a canister by its camera): only `place_props`, which puts down many
/// unrelated things, is checked against itself, and then only for deep hits.
fn clash(pp: &[Prop], pi: usize, grid: &HashMap<i64, Vec<usize>>, tol: f64) -> Option<Clash> {
    let p = &pp[pi];
    let mut worst: Option<Clash> = None;
    let mut seen: HashSet<usize> = HashSet::new();
    for x in cell_of(p.bb[0])..=cell_of(p.bb[3]) {
        for z in cell_of(p.bb[2])..=cell_of(p.bb[5]) {
            let Some(list) = grid.get(&(x * 65536 + z)) else { continue };
            for &qi in list {
                if !seen.insert(qi) {
                    continue;
                }
                let q = &pp[qi];
                if q.bb[0] > p.bb[3]
                    || q.bb[3] < p.bb[0]
                    || q.bb[1] > p.bb[4]
                    || q.bb[4] < p.bb[1]
                    || q.bb[2] > p.bb[5]
                    || q.bb[5] < p.bb[2]
                {
                    continue;
                }
                let same = q.placer == p.placer;
                if same && p.placer != "place_props" {
                    continue;
                }
                let lim = if same { 0.05 } else { tol };
                for a in &p.ms {
                    for b in &q.ms {
                        let r = sat_depth(&a.c, &a.a, &a.h, &b.c, &b.a, &b.h);
                        if r.depth > lim && (worst.is_none() || r.depth > worst.unwrap().depth) {
                            worst = Some(Clash { depth: r.depth, axis: r.axis, q: qi });
                        }
                    }
                }
            }
        }
    }
    worst
}

/// The `walls` query `settle_placement` builds from
/// `solids.boxes`: every solid box (tile units x0, z0, x1, z1 and heights
/// y0, y1) is filed under each tile cell it covers; a query returns the
/// distinct boxes of the cells under `bb`, in first-seen order.
#[derive(Clone, Debug, Default)]
pub struct WallGrid {
    w: i32,
    h: i32,
    cell: f64,
    boxes: Vec<[f64; 6]>,
    cells: HashMap<i64, Vec<usize>>,
}
impl WallGrid {
    /// Grid size `w`, `h` (tiles) and `cell` (metres per tile).
    pub fn new(w: i32, h: i32, cell: f64) -> WallGrid {
        WallGrid { w, h, cell, boxes: vec![], cells: HashMap::new() }
    }
    /// One `solids.boxes` entry: `b[q], b[q+1], b[q+2], b[q+3]` = x0, z0, x1, z1
    /// (tiles), `b[q+4], b[q+5]` = y0, y1 (metres).
    pub fn add_tile_box(&mut self, x0: f64, z0: f64, x1: f64, z1: f64, y0: f64, y1: f64) {
        let (wh, hh) = (self.w as f64 / 2.0, self.h as f64 / 2.0);
        let wb = [(x0 - wh) * self.cell, y0, (z0 - hh) * self.cell, (x1 - wh) * self.cell, y1, (z1 - hh) * self.cell];
        let i = self.boxes.len();
        self.boxes.push(wb);
        let mut x = x0.floor() as i64;
        while x <= x1.floor() as i64 {
            let mut z = z0.floor() as i64;
            while z <= z1.floor() as i64 {
                self.cells.entry(z * self.w as i64 + x).or_default().push(i);
                z += 1;
            }
            x += 1;
        }
    }
    /// All of `solids.boxes[0 .. count*8]` (8 numbers per box).
    pub fn from_solids(w: i32, h: i32, cell: f64, boxes: &[f64], count: usize) -> WallGrid {
        let mut g = WallGrid::new(w, h, cell);
        for i in 0..count {
            let q = i * 8;
            g.add_tile_box(boxes[q], boxes[q + 1], boxes[q + 2], boxes[q + 3], boxes[q + 4], boxes[q + 5]);
        }
        g
    }
    /// `walls(bb)`: the world boxes near `bb` = [x0, y0, z0, x1, y1, z1].
    pub fn query(&self, bb: &[f64; 6]) -> Vec<[f64; 6]> {
        let to_u = |v: f64| v / self.cell + self.w as f64 / 2.0;
        let to_v = |v: f64| v / self.cell + self.h as f64 / 2.0;
        let mut seen: HashSet<usize> = HashSet::new();
        let mut out = vec![];
        let (x0, x1) = (to_u(bb[0]).floor() as i64, to_u(bb[3]).floor() as i64);
        let (z0, z1) = (to_v(bb[2]).floor() as i64, to_v(bb[5]).floor() as i64);
        for x in x0..=x1 {
            for z in z0..=z1 {
                if let Some(list) = self.cells.get(&(z * self.w as i64 + x)) {
                    for &i in list {
                        if seen.insert(i) {
                            out.push(self.boxes[i]);
                        }
                    }
                }
            }
        }
        out
    }
    pub fn len(&self) -> usize {
        self.boxes.len()
    }
    pub fn is_empty(&self) -> bool {
        self.boxes.is_empty()
    }
}
