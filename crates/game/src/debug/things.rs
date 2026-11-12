//! The visual playtest's geometric detectors (`docs/VISUAL_PLAYTEST.md`
//! VP3): what the scene itself says about a view, no pixel read. *Clip*:
//! the eye or a corner of the near plane's face inside a thing (in the
//! thing's own frame, against its own box), or inside what stops a look;
//! and the body standing inside a thing (a table walked into: most things
//! do not stop a body). *Plain*: a thing in view and near whose material
//! has no picture, no vertex colour and no wear (a flat colour on
//! photographed or painted surfaces), is unlit, or glows. *Coplanar*: faces
//! of two different things on one plane, facing alike and overlapping (two
//! surfaces that will fight for one depth). A thing is named by its placer
//! (a core prop by its kind) and its mesh. Pure: the scene in, findings out.
use crate::base::game::Game;
use crate::debug::sight::Eye;
use engine::material::MatKind;
use engine::math::{Box3, Mat4, Vec3};
use engine::scene::Obj;

/// The camera's near plane (metres), as `perspective_camera` makes it
pub const NEAR: f64 = 0.05;
/// A near face's corner is padded out this much (its half-size times this)
const NEAR_PAD: f64 = 1.5;
/// A thing nearer than this to the eye is looked at for clipping (metres)
const CLIP_REACH: f64 = 3.0;
/// ... and for being plain, if in view
pub const PLAIN_REACH: f64 = 5.0;
/// A thing's box is taken this much inside (metres): a camera grazing a
/// face is not inside it
const INSET: f64 = 0.01;
/// A thing shorter than this stops no look at a body's height (a rug, a
/// decal, a puddle): never "the body inside"
const LOW: f64 = 0.3;
/// Faces are on one plane within this (metres) and parallel within this
/// (the cosine of the angle between them)
pub const COPLANAR: f64 = 0.002;
const PARALLEL: f64 = 0.9995;
/// Two faces overlap by at least this much to fight visibly (m²)
const OVERLAP: f64 = 0.0004;

/// One mesh of a thing (or one instance of an instanced mesh), as the
/// detectors see it
#[derive(Clone, Debug)]
pub struct Part {
    /// the thing (its placer, or a core prop's kind; `level` for the
    /// architecture) and an id per thing (a placed thing's index among the
    /// world's children; each mesh or instance of the architecture its own)
    pub label: String,
    pub thing: usize,
    /// the mesh's name (or its parent's), and `#i` for an instance
    pub mesh: String,
    pub world: Mat4,
    pub inverse: Mat4,
    /// how long a unit of the part's own x, y and z is in the world (metres)
    pub scale: [f64; 3],
    /// the geometry's own box, and the world box
    pub local: Box3,
    pub bounds: Box3,
    /// small enough to be judged by its box (no side over `THING`): a
    /// merged mesh of the architecture is not
    pub sized: bool,
    pub obj: Obj,
}

/// A thing found
#[derive(Clone, Debug, PartialEq)]
pub struct Finding {
    /// clip, body, plain, unlit, glow, coplanar
    pub kind: &'static str,
    pub label: String,
    pub mesh: String,
    /// where (world): the thing's middle, or the overlap's
    pub at: (f64, f64, f64),
    /// how much: metres inside, the share of the frame, the overlap in m²
    pub size: f64,
}

/// A part is judged by its box when no side of it is longer (metres)
const THING: f64 = 6.0;

/// Every mesh of everything in the world (the placed things, the core
/// props, the architecture), an instanced mesh's instances each a part
pub fn parts(g: &Game) -> Vec<Part> {
    let mut out = vec![];
    g.world.update_matrix_world(true);
    let mut level = 1usize << 40;
    for (k, root) in g.world.children().iter().enumerate() {
        let placed = {
            let u = root.user();
            match (&u.placer, u.s.get("kind")) {
                (_, Some(kind)) => Some(format!("core_{kind}")),
                (Some(p), None) => Some(p.clone()),
                (None, None) => None,
            }
        };
        root.traverse_visible(&mut |o| {
            if !o.is_mesh() {
                return;
            }
            let Some(geo) = o.try_geo() else { return };
            let local = geo.bounding_box();
            if local.is_empty() {
                return;
            }

            let mw = o.matrix_world();
            let n = if o.is_instanced() { o.instance_count() } else { 1 };
            level += 1;
            for i in 0..n {
                let world = if o.is_instanced() { Mat4::mul(&mw, &o.get_matrix_at(i)) } else { mw };
                let mut bounds = local;
                bounds.apply_matrix4(&world);
                let s = bounds.get_size();
                let (label, thing) = match &placed {
                    Some(l) => (l.clone(), k),
                    None => ("level".to_string(), level),
                };
                let name = describe(o, &bounds);
                let mesh = if o.is_instanced() { format!("{name} #{i}") } else { name };
                let axis = |x: f64, y: f64, z: f64| {
                    let (mut a, mut b) = (Vec3::new(0.0, 0.0, 0.0), Vec3::new(x, y, z));
                    a.apply_matrix4(&world);
                    b.apply_matrix4(&world);
                    ((b.x - a.x).powi(2) + (b.y - a.y).powi(2) + (b.z - a.z).powi(2)).sqrt()
                };
                let scale = [axis(1.0, 0.0, 0.0), axis(0.0, 1.0, 0.0), axis(0.0, 0.0, 1.0)];
                out.push(Part {
                    label,
                    thing,
                    mesh,
                    world,
                    inverse: world.inverted(),
                    scale,
                    local,
                    bounds,
                    sized: s.x.max(s.y).max(s.z) <= THING,
                    obj: o.clone(),
                });
            }
        });
    }
    out
}

/// What names a mesh for a reader: its own name or its nearest named
/// ancestor's, its geometry's kind, its size in the world, and its
/// material's colour and whether it has a picture (`box 4.00x3.10x0.30
/// #2b2a22 map`)
fn describe(o: &Obj, world: &Box3) -> String {
    let mut name = o.name();
    let mut up = o.parent();
    while name.is_empty() {
        match up {
            Some(p) => {
                name = p.name();
                up = p.parent();
            }
            None => break,
        }
    }
    let s = world.get_size();
    let kind = o.try_geo().map(|g| g.type_name()).unwrap_or("?");
    let mat = o.try_mat().map(|m| {
        let m = m.r();
        format!(" #{:06x}{}", m.color.get_hex(), if m.map.is_some() { " map" } else { "" })
    });
    format!("{name} {kind} {:.2}x{:.2}x{:.2}{}", s.x, s.y, s.z, mat.unwrap_or_default()).trim().to_string()
}

/// How far a point is from a box (0 inside)
fn dist(b: &Box3, p: &Vec3) -> f64 {
    let d = |lo: f64, hi: f64, v: f64| (lo - v).max(v - hi).max(0.0);
    let (x, y, z) = (d(b.min.x, b.max.x, p.x), d(b.min.y, b.max.y, p.y), d(b.min.z, b.max.z, p.z));
    (x * x + y * y + z * z).sqrt()
}

/// How deep the world point is inside the part's own box (metres, `INSET`
/// or more; None when it is not)
fn inside(p: &Part, w: Vec3) -> Option<f64> {
    let mut l = w;
    l.apply_matrix4(&p.inverse);
    let b = &p.local;
    let [sx, sy, sz] = p.scale;
    let d = [
        (l.x - b.min.x) * sx,
        (b.max.x - l.x) * sx,
        (l.y - b.min.y) * sy,
        (b.max.y - l.y) * sy,
        (l.z - b.min.z) * sz,
        (b.max.z - l.z) * sz,
    ];
    let depth = d.iter().cloned().fold(f64::MAX, f64::min);
    (depth > INSET).then_some(depth)
}

/// The eye and the four corners of its near plane's face (padded), world
pub fn near_face(eye: &Eye) -> [Vec3; 5] {
    let t = (eye.fov / 2.0).tan();
    let a = eye.w as f64 / eye.h as f64;
    let (hh, hw) = (NEAR * t * NEAR_PAD, NEAR * t * a * NEAR_PAD);
    let (sp, cp) = eye.pitch.sin_cos();
    let (sy, cy) = eye.yaw.sin_cos();
    // the camera's axes in the world (YXZ: yaw, then pitch)
    let fwd = Vec3::new(-sy * cp, sp, -cy * cp);
    let right = Vec3::new(cy, 0.0, -sy);
    let up = Vec3::new(sy * sp, cp, cy * sp);
    let o = Vec3::new(eye.x, eye.y, eye.z);
    let c = |r: f64, u: f64| {
        Vec3::new(
            o.x + fwd.x * NEAR + right.x * r + up.x * u,
            o.y + fwd.y * NEAR + right.y * r + up.y * u,
            o.z + fwd.z * NEAR + right.z * r + up.z * u,
        )
    };
    [o, c(-hw, -hh), c(hw, -hh), c(-hw, hh), c(hw, hh)]
}

/// Clip at a view: the eye or its near face inside a thing; the body (its
/// column from the knee to the eye, at the stand) inside a thing that
/// stands taller than `LOW`
pub fn clip(g: &Game, parts: &[Part], eye: &Eye) -> Vec<Finding> {
    let o = Vec3::new(eye.x, eye.y, eye.z);
    let face = near_face(eye);
    let floor = g.floor_y_at(eye.x, eye.z);
    let mut out = vec![];
    for p in parts {
        if !p.sized || dist(&p.bounds, &o) > CLIP_REACH {
            continue;
        }
        let at = p.bounds.get_center();
        if let Some(d) = face.iter().filter_map(|&c| inside(p, c)).reduce(f64::max) {
            out.push(Finding { kind: "clip", label: p.label.clone(), mesh: p.mesh.clone(), at: (at.x, at.y, at.z), size: d });
            continue;
        }
        if p.bounds.max.y - p.bounds.min.y < LOW || p.bounds.max.y < floor + LOW {
            continue;
        }
        let body = (0..6).map(|k| Vec3::new(eye.x, floor + 0.45 + k as f64 * 0.2, eye.z));
        if let Some(d) = body.filter_map(|c| inside(p, c)).reduce(f64::max) {
            out.push(Finding { kind: "body", label: p.label.clone(), mesh: p.mesh.clone(), at: (at.x, at.y, at.z), size: d });
        }
    }
    out
}

/// The share of the frame a box covers, roughly (its corners projected;
/// None when it is not in view)
pub fn screen_share(eye: &Eye, b: &Box3) -> Option<f64> {
    let t = (eye.fov / 2.0).tan();
    let a = eye.w as f64 / eye.h as f64;
    let (sp, cp) = eye.pitch.sin_cos();
    let (sy, cy) = eye.yaw.sin_cos();
    let fwd = (-sy * cp, sp, -cy * cp);
    let right = (cy, 0.0, -sy);
    let up = (sy * sp, cp, cy * sp);
    let (mut x0, mut x1, mut y0, mut y1) = (f64::MAX, f64::MIN, f64::MAX, f64::MIN);
    let mut ahead = 0;
    for k in 0..8 {
        let p = (
            if k & 1 == 0 { b.min.x } else { b.max.x } - eye.x,
            if k & 2 == 0 { b.min.y } else { b.max.y } - eye.y,
            if k & 4 == 0 { b.min.z } else { b.max.z } - eye.z,
        );
        let z = p.0 * fwd.0 + p.1 * fwd.1 + p.2 * fwd.2;
        if z < NEAR {
            continue;
        }
        ahead += 1;
        let x = (p.0 * right.0 + p.1 * right.1 + p.2 * right.2) / (z * t * a);
        let y = (p.0 * up.0 + p.1 * up.1 + p.2 * up.2) / (z * t);
        x0 = x0.min(x);
        x1 = x1.max(x);
        y0 = y0.min(y);
        y1 = y1.max(y);
    }
    if ahead == 0 {
        return None;
    }
    let (cx0, cx1, cy0, cy1) = (x0.max(-1.0), x1.min(1.0), y0.max(-1.0), y1.min(1.0));
    (cx1 > cx0 && cy1 > cy0).then(|| (cx1 - cx0) * (cy1 - cy0) / 4.0)
}

/// Plain at a view: in view and within `PLAIN_REACH`, a lit material with
/// no picture, no vertex colour and no wear (`plain`), an unlit one
/// (`unlit`), or one that glows (`glow`); its size is the share of the
/// frame the thing's box covers
pub fn plain(parts: &[Part], eye: &Eye) -> Vec<Finding> {
    let o = Vec3::new(eye.x, eye.y, eye.z);
    let mut out = vec![];
    for p in parts {
        if !p.sized || dist(&p.bounds, &o) > PLAIN_REACH {
            continue;
        }
        let Some(share) = screen_share(eye, &p.bounds) else { continue };
        if share < 0.002 {
            continue;
        }
        let at = p.bounds.get_center();
        for m in p.obj.mats() {
            let m = m.r();
            if !m.visible || m.opacity < 0.5 || !m.color_write {
                continue;
            }
            let lit = matches!(m.kind, MatKind::Lambert | MatKind::Phong | MatKind::Standard);
            let glow = m.emissive.r.max(m.emissive.g).max(m.emissive.b) * m.emissive_intensity;
            let kind = if glow > 0.25 || m.emissive_map.is_some() {
                "glow"
            } else if m.kind == MatKind::Basic && !m.transparent {
                "unlit"
            } else if lit && m.map.is_none() && !m.vertex_colors && m.wear <= 0.0 {
                "plain"
            } else {
                continue;
            };
            out.push(Finding { kind, label: p.label.clone(), mesh: p.mesh.clone(), at: (at.x, at.y, at.z), size: share });
            break;
        }
    }
    out
}

/// A face, world: its corners, its unit normal and its plane's offset
#[derive(Clone, Debug)]
struct Face {
    v: [Vec3; 3],
    n: Vec3,
    d: f64,
    part: usize,
}

fn sub(a: Vec3, b: Vec3) -> Vec3 {
    Vec3::new(a.x - b.x, a.y - b.y, a.z - b.z)
}
fn dot(a: Vec3, b: Vec3) -> f64 {
    a.x * b.x + a.y * b.y + a.z * b.z
}
fn cross(a: Vec3, b: Vec3) -> Vec3 {
    Vec3::new(a.y * b.z - a.z * b.y, a.z * b.x - a.x * b.z, a.x * b.y - a.y * b.x)
}

/// The faces of the parts within `reach` of any of `near` (world points)
fn faces(parts: &[Part], near: &[Vec3], reach: f64) -> Vec<Face> {
    let mut out = vec![];
    for (pi, p) in parts.iter().enumerate() {
        if !near.iter().any(|q| dist(&p.bounds, q) < reach) {
            continue;
        }
        let ms = p.obj.mats();
        if ms.iter().any(|m| {
            let m = m.r();
            m.transparent || !m.depth_write || m.polygon_offset || m.opacity < 0.5
        }) {
            continue;
        }
        let geo = p.obj.geo();
        let g = geo.r();
        let pos = &g.position;
        let vert = |i: usize| {
            let mut v = Vec3::new(pos[i * 3] as f64, pos[i * 3 + 1] as f64, pos[i * 3 + 2] as f64);
            v.apply_matrix4(&p.world);
            v
        };
        let tri: Vec<[usize; 3]> = match &g.index {
            Some(ix) => ix.chunks_exact(3).map(|c| [c[0] as usize, c[1] as usize, c[2] as usize]).collect(),
            None => (0..pos.len() / 9).map(|t| [t * 3, t * 3 + 1, t * 3 + 2]).collect(),
        };
        for [a, b, c] in tri {
            let v = [vert(a), vert(b), vert(c)];
            let n = cross(sub(v[1], v[0]), sub(v[2], v[0]));
            let l = dot(n, n).sqrt();
            if l < 1e-9 {
                continue;
            }
            let n = Vec3::new(n.x / l, n.y / l, n.z / l);
            out.push(Face { v, n, d: dot(n, v[0]), part: pi });
        }
    }
    out
}

/// The area two coplanar triangles share (Sutherland–Hodgman on the plane)
fn overlap(a: &Face, b: &Face) -> f64 {
    // a 2D frame on a's plane
    let e = sub(a.v[1], a.v[0]);
    let el = dot(e, e).sqrt();
    let u = Vec3::new(e.x / el, e.y / el, e.z / el);
    let w = cross(a.n, u);
    let to2 = |p: Vec3| {
        let q = sub(p, a.v[0]);
        (dot(q, u), dot(q, w))
    };
    let mut poly: Vec<(f64, f64)> = b.v.iter().map(|&p| to2(p)).collect();
    let clip_tri: Vec<(f64, f64)> = a.v.iter().map(|&p| to2(p)).collect();
    // counter-clockwise clip triangle
    let area2 = |t: &[(f64, f64)]| (t[1].0 - t[0].0) * (t[2].1 - t[0].1) - (t[2].0 - t[0].0) * (t[1].1 - t[0].1);
    let mut ct = clip_tri.clone();
    if area2(&ct) < 0.0 {
        ct.swap(1, 2);
    }
    for k in 0..3 {
        let (p0, p1) = (ct[k], ct[(k + 1) % 3]);
        let side = |q: (f64, f64)| (p1.0 - p0.0) * (q.1 - p0.1) - (p1.1 - p0.1) * (q.0 - p0.0);
        let mut next = vec![];
        for i in 0..poly.len() {
            let (s, e) = (poly[i], poly[(i + 1) % poly.len()]);
            let (ss, se) = (side(s), side(e));
            if se >= 0.0 {
                if ss < 0.0 {
                    let t = ss / (ss - se);
                    next.push((s.0 + (e.0 - s.0) * t, s.1 + (e.1 - s.1) * t));
                }
                next.push(e);
            } else if ss >= 0.0 {
                let t = ss / (ss - se);
                next.push((s.0 + (e.0 - s.0) * t, s.1 + (e.1 - s.1) * t));
            }
        }
        poly = next;
        if poly.len() < 3 {
            return 0.0;
        }
    }
    let mut a2 = 0.0;
    for i in 0..poly.len() {
        let (p, q) = (poly[i], poly[(i + 1) % poly.len()]);
        a2 += p.0 * q.1 - q.0 * p.1;
    }
    (a2 / 2.0).abs()
}

/// A plane bucket's width (metres): a face goes in its bucket and, near
/// its upper edge, the next one too, so two faces within `COPLANAR` share one
const BUCKET: f64 = 4.0 * COPLANAR;

/// Coplanar: faces within `reach` of the tour's stands that lie on one
/// plane, face alike and overlap by `OVERLAP` or more (a face against
/// another of the same mesh too: a doubled face fights as well); one
/// finding a pair of things, at its largest overlap
pub fn coplanar(parts: &[Part], near: &[Vec3], reach: f64) -> Vec<Finding> {
    let fs = faces(parts, near, reach);
    // buckets by the plane: its normal's dominant axis and sign, its offset
    let mut buckets: std::collections::BTreeMap<(usize, bool, i64), Vec<usize>> = Default::default();
    for (i, f) in fs.iter().enumerate() {
        let (ax, ay, az) = (f.n.x.abs(), f.n.y.abs(), f.n.z.abs());
        let axis = if ax >= ay && ax >= az {
            0
        } else if ay >= az {
            1
        } else {
            2
        };
        let sign = [f.n.x, f.n.y, f.n.z][axis] >= 0.0;
        let k = (f.d / BUCKET).floor();
        buckets.entry((axis, sign, k as i64)).or_default().push(i);
        if f.d - k * BUCKET > BUCKET - COPLANAR {
            buckets.entry((axis, sign, k as i64 + 1)).or_default().push(i);
        }
    }
    let lo = |f: &Face, a: usize| f.v.iter().map(|v| [v.x, v.y, v.z][a]).fold(f64::MAX, f64::min);
    let hi = |f: &Face, a: usize| f.v.iter().map(|v| [v.x, v.y, v.z][a]).fold(f64::MIN, f64::max);
    let mut best: std::collections::BTreeMap<(usize, usize), (f64, Vec3, usize, usize)> = Default::default();
    for (&(axis, _, _), list) in &buckets {
        // sweep along an axis of the plane
        let t = (axis + 1) % 3;
        let mut l = list.clone();
        l.sort_by(|&i, &j| lo(&fs[i], t).total_cmp(&lo(&fs[j], t)));
        for (x, &i) in l.iter().enumerate() {
            let a = &fs[i];
            let end = hi(a, t);
            for &j in &l[x + 1..] {
                let b = &fs[j];
                if lo(b, t) > end {
                    break;
                }
                let u = (axis + 2) % 3;
                if lo(b, u) > hi(a, u) || lo(a, u) > hi(b, u) {
                    continue;
                }
                if dot(a.n, b.n) < PARALLEL || (a.d - b.d).abs() > COPLANAR {
                    continue;
                }
                let ov = overlap(a, b);
                if ov < OVERLAP {
                    continue;
                }
                let (ta, tb) = (parts[a.part].thing, parts[b.part].thing);
                let pair = (ta.min(tb), ta.max(tb));
                let c = centroid(a, b);
                let e = best.entry(pair).or_insert((0.0, c, a.part, b.part));
                if ov > e.0 {
                    *e = (ov, c, a.part, b.part);
                }
            }
        }
    }
    best.into_values()
        .map(|(ov, c, pa, pb)| {
            let (a, b) = (&parts[pa], &parts[pb]);
            // one material on both: drawn alike where they overlap, a fight
            // seldom seen (listed, not flagged: the flicker detector has the say)
            let alike = match (a.obj.try_mat(), b.obj.try_mat()) {
                (Some(x), Some(y)) => x.ptr_eq(&y),
                _ => false,
            };
            Finding {
                kind: if alike { "coplanar_alike" } else { "coplanar" },
                label: format!("{} + {}", a.label, b.label),
                mesh: format!("{} + {}", a.mesh, b.mesh),
                at: (c.x, c.y, c.z),
                size: ov,
            }
        })
        .collect()
}

/// Between two faces' middles
fn centroid(a: &Face, b: &Face) -> Vec3 {
    let m = |f: &Face| {
        Vec3::new((f.v[0].x + f.v[1].x + f.v[2].x) / 3.0, (f.v[0].y + f.v[1].y + f.v[2].y) / 3.0, (f.v[0].z + f.v[1].z + f.v[2].z) / 3.0)
    };
    let (p, q) = (m(a), m(b));
    Vec3::new((p.x + q.x) / 2.0, (p.y + q.y) / 2.0, (p.z + q.z) / 2.0)
}

/// Faces this near a stand are tried for coplanar pairs (metres)
pub const COPLANAR_REACH: f64 = 6.0;
/// A thing plain, unlit or glowing is flagged when this many views find it
/// (one view can catch a thing at its worst; several agree it is so)
pub const PLAIN_VIEWS: usize = 2;

/// The squared distance between two points
pub fn d2(a: (f64, f64, f64), b: (f64, f64, f64)) -> f64 {
    (a.0 - b.0).powi(2) + (a.1 - b.1).powi(2) + (a.2 - b.2).powi(2)
}

/// Whether a world point is in the eye's frame and within 8 m
pub fn in_sight(eye: &Eye, p: (f64, f64, f64)) -> bool {
    if d2((eye.x, eye.y, eye.z), p) > 64.0 {
        return false;
    }
    let b = Box3::new(Vec3::new(p.0 - 0.01, p.1 - 0.01, p.2 - 0.01), Vec3::new(p.0 + 0.01, p.1 + 0.01, p.2 + 0.01));
    screen_share(eye, &b).is_some()
}

/// The findings of a level, gathered: a row a thing and kind (how many
/// views found it, the first, where, the largest size), and how many are
/// flagged of each kind: clip, body and coplanar at once, plain, unlit and
/// glow when `PLAIN_VIEWS` views agree; `coplanar_alike` listed only
pub fn gather(found: &[(String, Finding)]) -> (String, Vec<(&'static str, usize)>) {
    let mut by: std::collections::BTreeMap<(&'static str, String, String, i64, i64, i64), (usize, String, (f64, f64, f64), f64)> =
        Default::default();
    for (view, f) in found {
        let q = |v: f64| (v * 10.0).round() as i64;
        let e =
            by.entry((f.kind, f.label.clone(), f.mesh.clone(), q(f.at.0), q(f.at.1), q(f.at.2))).or_insert((0, view.clone(), f.at, 0.0));
        e.0 += 1;
        e.3 = e.3.max(f.size);
    }
    let mut rows = String::from("kind\tlabel\tmesh\tviews\tview\tx\ty\tz\tsize\tflagged\n");
    let mut flagged: std::collections::BTreeMap<&'static str, usize> = Default::default();
    let mut list: Vec<_> = by.into_iter().collect();
    // the flagged first, then by kind, the most seen and the largest
    let is_flagged = |k: &str, n: usize| match k {
        "plain" | "unlit" | "glow" => n >= PLAIN_VIEWS,
        "coplanar_alike" => false,
        _ => true,
    };
    list.sort_by(|a, b| {
        is_flagged(b.0 .0, b.1 .0)
            .cmp(&is_flagged(a.0 .0, a.1 .0))
            .then(a.0 .0.cmp(b.0 .0))
            .then(b.1 .0.cmp(&a.1 .0))
            .then(b.1 .3.total_cmp(&a.1 .3))
    });
    for ((kind, label, mesh, ..), (n, view, at, size)) in list {
        let f = is_flagged(kind, n);
        if f {
            *flagged.entry(kind).or_default() += 1;
        }
        rows.push_str(&format!("{kind}\t{label}\t{mesh}\t{n}\t{view}\t{:.3}\t{:.3}\t{:.3}\t{size:.4}\t{f}\n", at.0, at.1, at.2));
    }
    (rows, flagged.into_iter().collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn face(v: [(f64, f64, f64); 3], part: usize) -> Face {
        let v = v.map(|p| Vec3::new(p.0, p.1, p.2));
        let n = cross(sub(v[1], v[0]), sub(v[2], v[0]));
        let l = dot(n, n).sqrt();
        let n = Vec3::new(n.x / l, n.y / l, n.z / l);
        Face { v, n, d: dot(n, v[0]), part }
    }

    #[test]
    fn two_triangles_on_one_plane_share_their_overlap() {
        // a unit right triangle, and the same moved half along x: they share
        // a right triangle of legs 0.5 (area 0.125)
        let a = face([(0.0, 0.0, 0.0), (1.0, 0.0, 0.0), (0.0, 1.0, 0.0)], 0);
        let b = face([(0.5, 0.0, 0.0), (1.5, 0.0, 0.0), (0.5, 1.0, 0.0)], 1);
        assert!((overlap(&a, &b) - 0.125).abs() < 1e-9, "{}", overlap(&a, &b));
        // apart, and wound the other way round
        let c = face([(2.0, 0.0, 0.0), (3.0, 0.0, 0.0), (2.0, 1.0, 0.0)], 1);
        assert_eq!(overlap(&a, &c), 0.0);
        let d = face([(0.0, 0.0, 0.0), (0.0, 1.0, 0.0), (1.0, 0.0, 0.0)], 1);
        assert!((overlap(&a, &d) - 0.5).abs() < 1e-9);
    }

    #[test]
    fn the_near_face_stands_before_the_eye() {
        let eye = Eye { x: 1.0, y: 1.6, z: -2.0, yaw: 0.9, pitch: -0.4, fov: 0.8, w: 640, h: 400 };
        let f = near_face(&eye);
        assert_eq!((f[0].x, f[0].y, f[0].z), (1.0, 1.6, -2.0));
        let fwd = (-(0.9f64).sin() * (0.4f64).cos(), -(0.4f64).sin(), -(0.9f64).cos() * (0.4f64).cos());
        for c in &f[1..] {
            let d = (c.x - 1.0, c.y - 1.6, c.z + 2.0);
            // every corner NEAR ahead along the view, and a few centimetres off it
            assert!(((d.0 * fwd.0 + d.1 * fwd.1 + d.2 * fwd.2) - NEAR).abs() < 1e-9);
            assert!((d.0 * d.0 + d.1 * d.1 + d.2 * d.2).sqrt() < 0.1);
        }
        // a box straight ahead is in sight and covers some of the frame; behind, none
        let ahead = Box3::new(
            Vec3::new(1.0 + fwd.0 * 3.0 - 0.5, 1.6 + fwd.1 * 3.0 - 0.5, -2.0 + fwd.2 * 3.0 - 0.5),
            Vec3::new(1.0 + fwd.0 * 3.0 + 0.5, 1.6 + fwd.1 * 3.0 + 0.5, -2.0 + fwd.2 * 3.0 + 0.5),
        );
        let share = screen_share(&eye, &ahead).unwrap();
        assert!(share > 0.02 && share < 0.5, "{share}");
        let behind = Box3::new(
            Vec3::new(1.0 - fwd.0 * 3.0 - 0.5, 1.6 - fwd.1 * 3.0 - 0.5, -2.0 - fwd.2 * 3.0 - 0.5),
            Vec3::new(1.0 - fwd.0 * 3.0 + 0.5, 1.6 - fwd.1 * 3.0 + 0.5, -2.0 - fwd.2 * 3.0 + 0.5),
        );
        assert!(screen_share(&eye, &behind).is_none());
    }

    #[test]
    fn a_plain_thing_is_flagged_when_two_views_agree() {
        let f = |kind: &'static str, label: &str| Finding { kind, label: label.into(), mesh: "m".into(), at: (1.0, 0.5, 2.0), size: 0.1 };
        let once = [("v1".to_string(), f("plain", "a")), ("v1".to_string(), f("clip", "b")), ("v2".to_string(), f("coplanar_alike", "c"))];
        let (_, flagged) = gather(&once);
        assert_eq!(flagged, vec![("clip", 1)], "one view is not enough for a plain thing; a clip is a clip");
        let twice = [("v1".to_string(), f("plain", "a")), ("v2".to_string(), f("plain", "a"))];
        let (rows, flagged) = gather(&twice);
        assert_eq!(flagged, vec![("plain", 1)]);
        assert!(rows.lines().nth(1).unwrap().starts_with("plain\ta\tm\t2\tv1\t"), "{rows}");
    }
}
