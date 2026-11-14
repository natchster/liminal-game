//! The visual playtest's geometric detectors (`docs/engineering/VISUAL_PLAYTEST.md`
//! VP3): what the scene itself says about a view, no pixel read. *Clip*:
//! the eye or a corner of the near plane's face inside a thing (in the
//! thing's own frame, against its own box; not a merged mesh of the
//! architecture, and not the solids); and the body standing inside a thing
//! (a table walked into: most things do not stop a body). *Plain*: a thing
//! in view and near whose material has no picture, no vertex colour and no
//! wear (a flat colour on photographed or painted surfaces), or is unlit or
//! glows where the place is dark. *Coplanar*: two faces (of one thing or
//! two) on one plane, facing alike and overlapping (two surfaces that will
//! fight for one depth). A thing is named by its placer
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
    /// small enough to be judged at all (no side over `THING`): a merged
    /// mesh of the architecture is not
    pub sized: bool,
    /// a box or a cylinder as made (its box is its shape); anything else (a
    /// Blender part, a frame with an opening) is judged by its faces
    pub boxy: bool,
    /// its faces close round a volume (every edge shared by two of them), so
    /// a point is inside it when a ray out crosses an odd number; a plane, a
    /// shell or a frame is not, and nothing is ever inside it
    pub closed: bool,
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
    /// what it is in the world (min x, y, z, max x, y, z): the thing's box,
    /// or a cube round the overlap (the sheets outline it)
    pub bounds: [f64; 6],
}

/// A part is judged by its box when no side of it is longer (metres)
const THING: f64 = 6.0;

/// Every mesh of everything in the world (the placed things, the core
/// props, the architecture), an instanced mesh's instances each a part
pub fn parts(g: &Game) -> Vec<Part> {
    let mut out = vec![];
    let mut closed_of: std::collections::HashMap<u64, bool> = Default::default();
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
            // a body posed by morphs (a creature): its box is not its shape
            if !o.r().morph_influences.is_empty() || !geo.r().morph_position.is_empty() {
                return;
            }
            let boxy = matches!(geo.type_name(), "box" | "cylinder") && !geo.r().is_prop_part;
            let closed = *closed_of.entry(geo.id()).or_insert_with(|| is_closed(&geo));
            // the static instancing's mesh says what each instance was part of
            let labels: Vec<String> =
                o.user().s.get("instance_labels").map(|l| l.split('|').map(String::from).collect()).unwrap_or_default();

            let mw = o.matrix_world();
            let n = if o.is_instanced() { o.instance_count() } else { 1 };
            for i in 0..n {
                let world = if o.is_instanced() { Mat4::mul(&mw, &o.get_matrix_at(i)) } else { mw };
                let mut bounds = local;
                bounds.apply_matrix4(&world);
                let s = bounds.get_size();
                let (label, thing) = match (&placed, labels.get(i)) {
                    (Some(l), _) => (l.clone(), k),
                    (None, Some(l)) => {
                        level += 1;
                        (l.clone(), level)
                    }
                    (None, None) => {
                        level += 1;
                        ("level".to_string(), level)
                    }
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
                    boxy,
                    closed,
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
        let bounds = [p.bounds.min.x, p.bounds.min.y, p.bounds.min.z, p.bounds.max.x, p.bounds.max.y, p.bounds.max.z];
        let found = |kind: &'static str, d: f64| Finding {
            kind,
            label: p.label.clone(),
            mesh: p.mesh.clone(),
            at: (at.x, at.y, at.z),
            size: d,
            bounds,
        };
        let body: Vec<Vec3> = (0..6).map(|k| Vec3::new(eye.x, floor + 0.45 + k as f64 * 0.2, eye.z)).collect();
        let tall = p.bounds.max.y - p.bounds.min.y >= LOW && p.bounds.max.y >= floor + LOW;
        if p.boxy {
            if let Some(d) = face.iter().filter_map(|&c| inside(p, c)).reduce(f64::max) {
                out.push(found("clip", d));
            } else if let Some(d) = body.iter().filter_map(|&c| inside(p, c)).filter(|_| tall).reduce(f64::max) {
                out.push(found("body", d));
            }
            continue;
        }
        // by its faces: the near plane cuts one (a face nearer the near
        // face's middle than its corners are), or the eye is inside a
        // closed shape (an odd number of faces crossed on a ray out)
        let tris = world_tris(p);
        let mid = Vec3::new((face[1].x + face[4].x) / 2.0, (face[1].y + face[4].y) / 2.0, (face[1].z + face[4].z) / 2.0);
        let reach = ((face[1].x - mid.x).powi(2) + (face[1].y - mid.y).powi(2) + (face[1].z - mid.z).powi(2)).sqrt();
        let near = tris.iter().map(|t| tri_dist(mid, t)).fold(f64::MAX, f64::min);
        if near < reach {
            out.push(found("clip", reach - near));
        } else if p.closed && crossings(o, &tris) % 2 == 1 {
            out.push(found("clip", INSET));
        } else if p.closed && tall && body.iter().any(|&c| crossings(c, &tris) % 2 == 1) {
            out.push(found("body", INSET));
        }
    }
    out
}

/// The share of the frame a box covers, roughly (its edges clipped at the
/// near plane, their ends projected; None when it is not in view)
pub fn screen_share(eye: &Eye, b: &Box3) -> Option<f64> {
    screen_rect(eye, b).map(|(x0, y0, x1, y1)| (x1 - x0) * (y1 - y0) / 4.0)
}

/// Where a box is in the frame: (left, bottom, right, top) from -1 to 1,
/// clipped to the frame (None when it is not in view)
pub fn screen_rect(eye: &Eye, b: &Box3) -> Option<(f64, f64, f64, f64)> {
    let t = (eye.fov / 2.0).tan();
    let a = eye.w as f64 / eye.h as f64;
    let (sp, cp) = eye.pitch.sin_cos();
    let (sy, cy) = eye.yaw.sin_cos();
    let fwd = (-sy * cp, sp, -cy * cp);
    let right = (cy, 0.0, -sy);
    let up = (sy * sp, cp, cy * sp);
    let (mut x0, mut x1, mut y0, mut y1) = (f64::MAX, f64::MIN, f64::MAX, f64::MIN);
    let mut ahead = 0;
    // each corner in the camera's frame: (right, up, depth)
    let cam = |k: usize| {
        let p = (
            if k & 1 == 0 { b.min.x } else { b.max.x } - eye.x,
            if k & 2 == 0 { b.min.y } else { b.max.y } - eye.y,
            if k & 4 == 0 { b.min.z } else { b.max.z } - eye.z,
        );
        (p.0 * right.0 + p.1 * right.1 + p.2 * right.2, p.0 * up.0 + p.1 * up.1 + p.2 * up.2, p.0 * fwd.0 + p.1 * fwd.1 + p.2 * fwd.2)
    };
    let mut take = |c: (f64, f64, f64)| {
        ahead += 1;
        let (x, y) = (c.0 / (c.2 * t * a), c.1 / (c.2 * t));
        x0 = x0.min(x);
        x1 = x1.max(x);
        y0 = y0.min(y);
        y1 = y1.max(y);
    };
    // the twelve edges: corners that differ in one bit
    for k in 0..8usize {
        for bit in [1usize, 2, 4] {
            let j = k | bit;
            if j == k {
                continue;
            }
            let (p, q) = (cam(k), cam(j));
            match (p.2 >= NEAR, q.2 >= NEAR) {
                (true, true) => {
                    take(p);
                    take(q);
                }
                (false, false) => {}
                (pa, _) => {
                    let (ahead_end, behind) = if pa { (p, q) } else { (q, p) };
                    let s = (ahead_end.2 - NEAR) / (ahead_end.2 - behind.2);
                    take(ahead_end);
                    take((ahead_end.0 + (behind.0 - ahead_end.0) * s, ahead_end.1 + (behind.1 - ahead_end.1) * s, NEAR));
                }
            }
        }
    }
    if ahead == 0 {
        return None;
    }
    let (cx0, cx1, cy0, cy1) = (x0.max(-1.0), x1.min(1.0), y0.max(-1.0), y1.min(1.0));
    (cx1 > cx0 && cy1 > cy0).then_some((cx0, cy0, cx1, cy1))
}

/// The place's light under which an unlit or glowing thing stands out (the
/// light map, 0 dark to 1 lit): where it is lit, a thing lit by itself is a
/// lamp, a panel, a lit window, a screen, as meant (`selflit`, listed only)
pub const DARK: f64 = 0.25;

/// Plain at a view: in view and within `PLAIN_REACH`, a lit material with
/// no picture, no vertex colour and no wear (`plain`), an unlit one in the
/// dark (`unlit`), or one that glows in the dark (`glow`); unlit or glowing
/// where the place is lit is `selflit`; its size is the share of the frame
/// the thing's box covers
pub fn plain(g: &Game, parts: &[Part], eye: &Eye) -> Vec<Finding> {
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
            let dark = g.light_map_at(at.x, at.z) < DARK;
            let kind = if glow > 0.25 || m.emissive_map.is_some() {
                if dark { "glow" } else { "selflit" }
            } else if m.kind == MatKind::Basic && !m.transparent {
                if dark { "unlit" } else { "selflit" }
            } else if lit && m.map.is_none() && !m.vertex_colors && m.wear <= 0.0 {
                "plain"
            } else {
                continue;
            };
            let bounds = [p.bounds.min.x, p.bounds.min.y, p.bounds.min.z, p.bounds.max.x, p.bounds.max.y, p.bounds.max.z];
            out.push(Finding { kind, label: p.label.clone(), mesh: p.mesh.clone(), at: (at.x, at.y, at.z), size: share, bounds });
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

/// The stands bucketed on a grid of `reach` (metres): what is within
/// `reach` of a point is in its cell's 3 × 3
struct Near<'a> {
    reach: f64,
    cells: std::collections::HashMap<(i64, i64), Vec<&'a Vec3>>,
}

impl<'a> Near<'a> {
    fn new(near: &'a [Vec3], reach: f64) -> Near<'a> {
        let mut cells: std::collections::HashMap<(i64, i64), Vec<&'a Vec3>> = Default::default();
        for q in near {
            cells.entry(((q.x / reach).floor() as i64, (q.z / reach).floor() as i64)).or_default().push(q);
        }
        Near { reach, cells }
    }
    /// The stand nearest a triangle, among those within reach of its box
    fn nearest(&self, t: &[Vec3; 3]) -> Option<&'a Vec3> {
        let mut b = Box3::empty();
        for c in t {
            b.expand_by_point(c);
        }
        let (x0, x1) = ((b.min.x / self.reach).floor() as i64 - 1, (b.max.x / self.reach).floor() as i64 + 1);
        let (z0, z1) = ((b.min.z / self.reach).floor() as i64 - 1, (b.max.z / self.reach).floor() as i64 + 1);
        let cands: Box<dyn Iterator<Item = &&'a Vec3>> = if (x1 - x0 + 1) * (z1 - z0 + 1) > 4 * self.cells.len() as i64 {
            Box::new(self.cells.values().flatten())
        } else {
            Box::new((x0..=x1).flat_map(move |x| (z0..=z1).filter_map(move |z| self.cells.get(&(x, z)))).flatten())
        };
        cands.map(|q| (tri_dist(**q, t), *q)).filter(|(d, _)| *d < self.reach).min_by(|a, b| a.0.total_cmp(&b.0)).map(|(_, q)| q)
    }

    /// Whether a box is within reach of a stand
    fn by(&self, b: &Box3) -> bool {
        let c = b.get_center();
        let (cx, cz) = ((c.x / self.reach).floor() as i64, (c.z / self.reach).floor() as i64);
        // a large box: every cell under it, grown by one
        let (x0, x1) = (((b.min.x / self.reach).floor() as i64 - 1).min(cx - 1), ((b.max.x / self.reach).floor() as i64 + 1).max(cx + 1));
        let (z0, z1) = (((b.min.z / self.reach).floor() as i64 - 1).min(cz - 1), ((b.max.z / self.reach).floor() as i64 + 1).max(cz + 1));
        if (x1 - x0 + 1) * (z1 - z0 + 1) > 4 * self.cells.len() as i64 {
            return self.cells.values().flatten().any(|q| dist(b, q) < self.reach);
        }
        (x0..=x1).any(|x| (z0..=z1).any(|z| self.cells.get(&(x, z)).is_some_and(|l| l.iter().any(|q| dist(b, q) < self.reach))))
    }
}

/// The faces within `reach` of any of `near` (world points), each face
/// judged by itself (a long part reaches far from a stand at one end only)
fn faces(parts: &[Part], near: &[Vec3], reach: f64) -> Vec<Face> {
    let grid = Near::new(near, reach);
    let mut out = vec![];
    for (pi, p) in parts.iter().enumerate() {
        if !grid.by(&p.bounds) {
            continue;
        }
        let ms = p.obj.mats();
        if ms.iter().any(|m| {
            let m = m.r();
            m.transparent || !m.depth_write || m.polygon_offset || m.opacity < 0.5
        }) {
            continue;
        }
        for v in world_tris(p) {
            let n = cross(sub(v[1], v[0]), sub(v[2], v[0]));
            let l = dot(n, n).sqrt();
            if l < 1e-9 {
                continue;
            }
            let mut b = Box3::empty();
            for c in &v {
                b.expand_by_point(c);
            }
            if !grid.by(&b) {
                continue;
            }
            let n = Vec3::new(n.x / l, n.y / l, n.z / l);
            out.push(Face { v, n, d: dot(n, v[0]), part: pi });
        }
    }
    out
}

/// A part's triangles in the world
fn world_tris(p: &Part) -> Vec<[Vec3; 3]> {
    let geo = p.obj.geo();
    let g = geo.r();
    let pos = &g.position;
    let vert = |i: usize| {
        let mut v = Vec3::new(pos[i * 3] as f64, pos[i * 3 + 1] as f64, pos[i * 3 + 2] as f64);
        v.apply_matrix4(&p.world);
        v
    };
    let tri: Vec<[usize; 3]> = match &g.index {
        Some(ix) => ix.as_chunks::<3>().0.iter().map(|c| [c[0] as usize, c[1] as usize, c[2] as usize]).collect(),
        None => (0..pos.len() / 9).map(|t| [t * 3, t * 3 + 1, t * 3 + 2]).collect(),
    };
    tri.into_iter().map(|[a, b, c]| [vert(a), vert(b), vert(c)]).collect()
}

/// Whether a geometry's faces close round a volume: every edge (its
/// vertices welded by place, to the millimetre) shared by exactly two faces
fn is_closed(geo: &engine::geometry::Geo) -> bool {
    let g = geo.r();
    let pos = &g.position;
    let key = |i: usize| {
        ((pos[i * 3] * 1000.0).round() as i64, (pos[i * 3 + 1] * 1000.0).round() as i64, (pos[i * 3 + 2] * 1000.0).round() as i64)
    };
    let tri: Vec<[usize; 3]> = match &g.index {
        Some(ix) => ix.as_chunks::<3>().0.iter().map(|c| [c[0] as usize, c[1] as usize, c[2] as usize]).collect(),
        None => (0..pos.len() / 9).map(|t| [t * 3, t * 3 + 1, t * 3 + 2]).collect(),
    };
    if tri.is_empty() {
        return false;
    }
    let mut edges: std::collections::HashMap<((i64, i64, i64), (i64, i64, i64)), u32> = Default::default();
    for [a, b, c] in tri {
        for (p, q) in [(a, b), (b, c), (c, a)] {
            let (kp, kq) = (key(p), key(q));
            if kp == kq {
                continue;
            }
            *edges.entry(if kp < kq { (kp, kq) } else { (kq, kp) }).or_default() += 1;
        }
    }
    edges.values().all(|&n| n == 2)
}

/// The distance from a point to a triangle (Ericson's closest point)
fn tri_dist(p: Vec3, t: &[Vec3; 3]) -> f64 {
    let (a, b, c) = (t[0], t[1], t[2]);
    let (ab, ac, ap) = (sub(b, a), sub(c, a), sub(p, a));
    let (d1, d2) = (dot(ab, ap), dot(ac, ap));
    let len = |v: Vec3| dot(v, v).sqrt();
    if d1 <= 0.0 && d2 <= 0.0 {
        return len(ap);
    }
    let bp = sub(p, b);
    let (d3, d4) = (dot(ab, bp), dot(ac, bp));
    if d3 >= 0.0 && d4 <= d3 {
        return len(bp);
    }
    let vc = d1 * d4 - d3 * d2;
    if vc <= 0.0 && d1 >= 0.0 && d3 <= 0.0 {
        let v = d1 / (d1 - d3);
        return len(sub(p, Vec3::new(a.x + ab.x * v, a.y + ab.y * v, a.z + ab.z * v)));
    }
    let cp = sub(p, c);
    let (d5, d6) = (dot(ab, cp), dot(ac, cp));
    if d6 >= 0.0 && d5 <= d6 {
        return len(cp);
    }
    let vb = d5 * d2 - d1 * d6;
    if vb <= 0.0 && d2 >= 0.0 && d6 <= 0.0 {
        let w = d2 / (d2 - d6);
        return len(sub(p, Vec3::new(a.x + ac.x * w, a.y + ac.y * w, a.z + ac.z * w)));
    }
    let va = d3 * d6 - d5 * d4;
    if va <= 0.0 && (d4 - d3) >= 0.0 && (d5 - d6) >= 0.0 {
        let w = (d4 - d3) / ((d4 - d3) + (d5 - d6));
        return len(sub(p, Vec3::new(b.x + (c.x - b.x) * w, b.y + (c.y - b.y) * w, b.z + (c.z - b.z) * w)));
    }
    let den = 1.0 / (va + vb + vc);
    let (v, w) = (vb * den, vc * den);
    len(sub(p, Vec3::new(a.x + ab.x * v + ac.x * w, a.y + ab.y * v + ac.y * w, a.z + ab.z * v + ac.z * w)))
}

/// How many triangles a ray from `p` crosses (a direction off every axis,
/// so it seldom meets an edge): odd inside a closed shape
fn crossings(p: Vec3, tris: &[[Vec3; 3]]) -> usize {
    let d = Vec3::new(0.8017, 0.4123, 0.4327);
    tris.iter()
        .filter(|t| {
            let (e1, e2) = (sub(t[1], t[0]), sub(t[2], t[0]));
            let h = cross(d, e2);
            let a = dot(e1, h);
            if a.abs() < 1e-12 {
                return false;
            }
            let f = 1.0 / a;
            let s = sub(p, t[0]);
            let u = f * dot(s, h);
            if !(0.0..=1.0).contains(&u) {
                return false;
            }
            let q = cross(s, e1);
            let v = f * dot(d, q);
            if v < 0.0 || u + v > 1.0 {
                return false;
            }
            f * dot(e2, q) > 1e-9
        })
        .count()
}

/// The area two coplanar triangles share (Sutherland–Hodgman on the plane)
/// and its middle (the world point at the shared polygon's centroid); with
/// a window (a point and a half-size, metres), only what lies within it on
/// the plane (a facade's face runs a hundred metres from the stand that
/// sees it)
#[cfg(test)]
fn overlap(a: &Face, b: &Face) -> (f64, Vec3) {
    overlap_in(a, b, None)
}

fn overlap_in(a: &Face, b: &Face, window: Option<(Vec3, f64)>) -> (f64, Vec3) {
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
    // the clipping edges, counter-clockwise: the triangle's, then the window's square
    let mut edges: Vec<((f64, f64), (f64, f64))> = (0..3).map(|k| (ct[k], ct[(k + 1) % 3])).collect();
    if let Some((c, r)) = window {
        let (cu, cw) = to2(c);
        let sq = [(cu - r, cw - r), (cu + r, cw - r), (cu + r, cw + r), (cu - r, cw + r)];
        edges.extend((0..4).map(|k| (sq[k], sq[(k + 1) % 4])));
    }
    for (p0, p1) in edges {
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
            return (0.0, a.v[0]);
        }
    }
    let (mut a2, mut cx, mut cy) = (0.0, 0.0, 0.0);
    for i in 0..poly.len() {
        let (p, q) = (poly[i], poly[(i + 1) % poly.len()]);
        let c = p.0 * q.1 - q.0 * p.1;
        a2 += c;
        cx += (p.0 + q.0) * c;
        cy += (p.1 + q.1) * c;
    }
    let (mx, my) = if a2.abs() > 1e-15 { (cx / (3.0 * a2), cy / (3.0 * a2)) } else { (poly[0].0, poly[0].1) };
    let at = Vec3::new(a.v[0].x + u.x * mx + w.x * my, a.v[0].y + u.y * mx + w.y * my, a.v[0].z + u.z * mx + w.z * my);
    ((a2 / 2.0).abs(), at)
}

/// A plane bucket's width (metres): a face goes in its bucket and, near
/// its upper edge, the next one too, so two faces within `COPLANAR` share one
const BUCKET: f64 = 4.0 * COPLANAR;

/// Coplanar: faces within `reach` of the tour's stands (each face by
/// itself) that lie on one plane, face alike and overlap (a face against
/// another of the same mesh too: a doubled face fights as well), summed a
/// pair of parts to `OVERLAP` or more; one finding a pair of things, at the
/// middle of the largest overlap of its largest pair of parts
pub fn coplanar(parts: &[Part], near: &[Vec3], reach: f64) -> Vec<Finding> {
    let fs = faces(parts, near, reach);
    let grid = Near::new(near, reach);
    // buckets by the plane: its normal's dominant axis and sign, its offset;
    // a face whose two largest components nearly tie (about 45°) goes in
    // both axes' buckets
    let mut buckets: std::collections::BTreeMap<(usize, bool, i64), Vec<usize>> = Default::default();
    for (i, f) in fs.iter().enumerate() {
        let n = [f.n.x, f.n.y, f.n.z];
        let mut order = [0usize, 1, 2];
        order.sort_by(|&a, &b| n[b].abs().total_cmp(&n[a].abs()));
        let axes: &[usize] = if n[order[0]].abs() - n[order[1]].abs() < 0.05 { &order[..2] } else { &order[..1] };
        for &axis in axes {
            let sign = n[axis] >= 0.0;
            let k = (f.d / BUCKET).floor();
            buckets.entry((axis, sign, k as i64)).or_default().push(i);
            if f.d - k * BUCKET > BUCKET - COPLANAR {
                buckets.entry((axis, sign, k as i64 + 1)).or_default().push(i);
            }
        }
    }
    let lo = |f: &Face, a: usize| f.v.iter().map(|v| [v.x, v.y, v.z][a]).fold(f64::MAX, f64::min);
    let hi = |f: &Face, a: usize| f.v.iter().map(|v| [v.x, v.y, v.z][a]).fold(f64::MIN, f64::max);
    let mut seen: std::collections::HashSet<(usize, usize)> = Default::default();
    // a pair of parts: the overlap summed, and the middle of its largest
    let mut pairs: std::collections::BTreeMap<(usize, usize), (f64, f64, Vec3)> = Default::default();
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
                if dot(a.n, b.n) < PARALLEL || (a.d - b.d).abs() > COPLANAR || !seen.insert((i.min(j), i.max(j))) {
                    continue;
                }
                // only what lies within reach of the stand nearest the
                // smaller face, on which the overlap lies (the larger, a
                // facade's long sliver, may be nearest a stand far from
                // where the two meet: review B; a face whose box reaches a
                // stand where the face itself does not is out of reach)
                let area = |f: &Face| {
                    let c = cross(sub(f.v[1], f.v[0]), sub(f.v[2], f.v[0]));
                    dot(c, c)
                };
                let small = if area(a) <= area(b) { a } else { b };
                let Some(stand) = grid.nearest(&small.v) else { continue };
                let (ov, at) = overlap_in(a, b, Some((*stand, reach)));
                if ov <= 1e-9 {
                    continue;
                }
                let e = pairs.entry((a.part.min(b.part), a.part.max(b.part))).or_insert((0.0, 0.0, at));
                e.0 += ov;
                if ov > e.1 {
                    e.1 = ov;
                    e.2 = at;
                }
            }
        }
    }
    // one row a pair of things (the largest pair of their parts)
    let mut best: std::collections::BTreeMap<(usize, usize), (f64, Vec3, usize, usize)> = Default::default();
    for ((pa, pb), (sum, _, at)) in pairs {
        if sum < OVERLAP {
            continue;
        }
        let (ta, tb) = (parts[pa].thing, parts[pb].thing);
        let e = best.entry((ta.min(tb), ta.max(tb))).or_insert((0.0, at, pa, pb));
        if sum > e.0 {
            *e = (sum, at, pa, pb);
        }
    }
    best.into_values()
        .map(|(ov, c, pa, pb)| {
            let (a, b) = (&parts[pa], &parts[pb]);
            // drawn alike where they overlap (one colour, one picture, one
            // kind of material): a fight seldom seen (listed, not flagged:
            // the flicker detector has the say)
            let alike = match (a.obj.try_mat(), b.obj.try_mat()) {
                (Some(x), Some(y)) => {
                    let (x, y) = (x.r(), y.r());
                    x.kind == y.kind && x.color.get_hex() == y.color.get_hex() && x.map == y.map
                }
                _ => false,
            };
            Finding {
                kind: if alike { "coplanar_alike" } else { "coplanar" },
                label: format!("{} + {}", a.label, b.label),
                mesh: format!("{} + {}", a.mesh, b.mesh),
                at: (c.x, c.y, c.z),
                size: ov,
                bounds: [c.x - 0.3, c.y - 0.3, c.z - 0.3, c.x + 0.3, c.y + 0.3, c.z + 0.3],
            }
        })
        .collect()
}

/// Faces this near a stand are tried for coplanar pairs (metres)
pub const COPLANAR_REACH: f64 = 6.0;
/// A thing plain, unlit or glowing is flagged when this many stands see it
/// (one stand can catch a thing at its worst; several agree it is so; a
/// stand's looks share its eye, so they count once)
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

/// The stand a view belongs to (its name less its look: `L1_003_left` →
/// `L1_003`; a camera given by name is its own)
pub fn stand_of(view: &str) -> &str {
    match view.rsplit_once('_') {
        Some((s, look)) if ["at", "left", "right", "up", "down", "crouch"].contains(&look) => s,
        _ => view,
    }
}

/// The findings of a level, gathered: a row a thing and kind (how many
/// views and stands found it, the first view, where, the largest size), and
/// how many are flagged of each kind: clip, body and coplanar at once;
/// plain, unlit and glow when `PLAIN_VIEWS` stands agree; `coplanar_alike`
/// and `selflit` listed only
pub fn gather(found: &[(String, Finding)]) -> (String, Vec<(&'static str, usize)>) {
    type Seen = (usize, String, (f64, f64, f64), f64, std::collections::BTreeSet<String>);
    let mut by: std::collections::BTreeMap<(&'static str, String, String, i64, i64, i64), Seen> = Default::default();
    for (view, f) in found {
        let q = |v: f64| (v * 10.0).round() as i64;
        let e = by.entry((f.kind, f.label.clone(), f.mesh.clone(), q(f.at.0), q(f.at.1), q(f.at.2))).or_insert((
            0,
            view.clone(),
            f.at,
            0.0,
            Default::default(),
        ));
        e.0 += 1;
        e.3 = e.3.max(f.size);
        e.4.insert(stand_of(view).to_string());
    }
    let mut rows = String::from("kind\tlabel\tmesh\tviews\tview\tx\ty\tz\tsize\tflagged\tstands\n");
    let mut flagged: std::collections::BTreeMap<&'static str, usize> = Default::default();
    let mut list: Vec<_> = by.into_iter().collect();
    // the flagged first, then by kind, the most seen and the largest
    let is_flagged = |k: &str, stands: usize| match k {
        "plain" | "unlit" | "glow" => stands >= PLAIN_VIEWS,
        "coplanar_alike" | "selflit" => false,
        _ => true,
    };
    list.sort_by(|a, b| {
        is_flagged(b.0.0, b.1.4.len())
            .cmp(&is_flagged(a.0.0, a.1.4.len()))
            .then(a.0.0.cmp(b.0.0))
            .then(b.1.0.cmp(&a.1.0))
            .then(b.1.3.total_cmp(&a.1.3))
    });
    for ((kind, label, mesh, ..), (n, view, at, size, stands)) in list {
        let f = is_flagged(kind, stands.len());
        if f {
            *flagged.entry(kind).or_default() += 1;
        }
        rows.push_str(&format!(
            "{kind}\t{label}\t{mesh}\t{n}\t{view}\t{:.3}\t{:.3}\t{:.3}\t{size:.4}\t{f}\t{}\n",
            at.0,
            at.1,
            at.2,
            stands.len()
        ));
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
        let (area, at) = overlap(&a, &b);
        assert!((area - 0.125).abs() < 1e-9, "{area}");
        // its middle is the shared triangle's: (0.5 + 1/6, 1/6)
        assert!((at.x - (0.5 + 1.0 / 6.0)).abs() < 1e-9 && (at.y - 1.0 / 6.0).abs() < 1e-9 && at.z.abs() < 1e-12, "{at:?}");
        // apart, and wound the other way round
        let c = face([(2.0, 0.0, 0.0), (3.0, 0.0, 0.0), (2.0, 1.0, 0.0)], 1);
        assert_eq!(overlap(&a, &c).0, 0.0);
        let d = face([(0.0, 0.0, 0.0), (0.0, 1.0, 0.0), (1.0, 0.0, 0.0)], 1);
        assert!((overlap(&a, &d).0 - 0.5).abs() < 1e-9);
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
    fn a_point_knows_how_far_a_face_is_and_whether_it_is_inside() {
        let t = [Vec3::new(0.0, 0.0, 0.0), Vec3::new(1.0, 0.0, 0.0), Vec3::new(0.0, 1.0, 0.0)];
        assert!((tri_dist(Vec3::new(0.2, 0.2, 0.5), &t) - 0.5).abs() < 1e-12);
        assert!((tri_dist(Vec3::new(-1.0, 0.0, 0.0), &t) - 1.0).abs() < 1e-12);
        assert!((tri_dist(Vec3::new(1.0, 1.0, 0.0), &t) - 0.5f64.sqrt()).abs() < 1e-12);
        // a unit cube's twelve faces: inside crosses one, outside none or two
        let geo = engine::geometry::box_geometry(1.0, 1.0, 1.0, 1.0, 1.0, 1.0);
        let g = geo.r();
        let v = |i: usize| Vec3::new(g.position[i * 3] as f64, g.position[i * 3 + 1] as f64, g.position[i * 3 + 2] as f64);
        let tris: Vec<[Vec3; 3]> =
            g.index.as_ref().unwrap().as_chunks::<3>().0.iter().map(|c| [v(c[0] as usize), v(c[1] as usize), v(c[2] as usize)]).collect();
        assert!(is_closed(&geo), "a box is closed");
        assert!(!is_closed(&engine::geometry::plane_geometry(1.0, 1.0, 1.0, 1.0)), "a plane is not");
        assert_eq!(crossings(Vec3::new(0.1, -0.2, 0.3), &tris) % 2, 1);
        assert_eq!(crossings(Vec3::new(0.9, 0.0, 0.0), &tris) % 2, 0);
        assert_eq!(crossings(Vec3::new(-0.9, 0.1, 0.0), &tris) % 2, 0);
    }

    #[test]
    fn a_box_reaching_behind_the_eye_is_measured_to_the_near_plane() {
        // a post 4 cm thick from 1 m behind the eye to 6 cm ahead: what is
        // seen of it runs to the near plane, wider than its ahead corners say
        let eye = Eye { x: 0.0, y: 0.0, z: 0.0, yaw: 0.0, pitch: 0.0, fov: 1.0, w: 640, h: 400 };
        let post = Box3::new(Vec3::new(-0.02, -0.02, -0.06), Vec3::new(0.02, 0.02, 1.0));
        let (t, a) = ((0.5f64).tan(), 1.6);
        let corners_only = (0.04 / (0.06 * t * a)) * (0.04 / (0.06 * t)) / 4.0;
        let to_near = (0.04 / (NEAR * t * a)) * (0.04 / (NEAR * t)) / 4.0;
        let s = screen_share(&eye, &post).unwrap();
        assert!((s - to_near).abs() < 1e-9 && s > corners_only, "{s} {to_near} {corners_only}");
        // wholly behind: not in view
        assert!(screen_share(&eye, &Box3::new(Vec3::new(-1.0, -1.0, 0.5), Vec3::new(1.0, 1.0, 2.0))).is_none());
    }

    #[test]
    fn a_plain_thing_is_flagged_when_two_views_agree() {
        let f = |kind: &'static str, label: &str| Finding {
            kind,
            label: label.into(),
            mesh: "m".into(),
            at: (1.0, 0.5, 2.0),
            size: 0.1,
            bounds: [0.0; 6],
        };
        let once = [("v1".to_string(), f("plain", "a")), ("v1".to_string(), f("clip", "b")), ("v2".to_string(), f("coplanar_alike", "c"))];
        let (_, flagged) = gather(&once);
        assert_eq!(flagged, vec![("clip", 1)], "one view is not enough for a plain thing; a clip is a clip");
        // two looks of one stand are one stand; two stands agree
        let one_stand = [("L1_003_at".to_string(), f("plain", "a")), ("L1_003_down".to_string(), f("plain", "a"))];
        assert!(gather(&one_stand).1.is_empty());
        let twice = [("L1_003_at".to_string(), f("plain", "a")), ("L1_007_left".to_string(), f("plain", "a"))];
        let (rows, flagged) = gather(&twice);
        assert_eq!(flagged, vec![("plain", 1)]);
        assert!(rows.lines().nth(1).unwrap().starts_with("plain\ta\tm\t2\tL1_003_at\t"), "{rows}");
        assert!(rows.lines().nth(1).unwrap().ends_with("\ttrue\t2"), "{rows}");
        assert_eq!(stand_of("L1_001c_crouch"), "L1_001c");
        assert_eq!(stand_of("elev"), "elev");
    }
}
