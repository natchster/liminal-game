//! Indexed triangle geometry (positions, normals, uvs, colours, skin weights,
//! morph targets, groups) and the primitive generators: box, plane, circle,
//! cylinder, cone, sphere, torus, ring, lathe, capsule, polyhedra and shapes.

use crate::math::*;
use std::cell::{Ref, RefCell, RefMut};
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_ID: AtomicU64 = AtomicU64::new(1);
pub fn next_id() -> u64 {
    NEXT_ID.fetch_add(1, Ordering::Relaxed)
}

/// The parameters a generated geometry was built with (`assets::props` reads them).
#[derive(Clone, Debug, PartialEq, Default)]
pub enum GeoParams {
    #[default]
    Buffer,
    Box {
        width: f64,
        height: f64,
        depth: f64,
        ws: u32,
        hs: u32,
        ds: u32,
    },
    Cylinder {
        radius_top: f64,
        radius_bottom: f64,
        height: f64,
        radial: u32,
        height_seg: u32,
        open_ended: bool,
        theta_start: f64,
        theta_length: f64,
    },
    Cone {
        radius: f64,
        height: f64,
        radial: u32,
        height_seg: u32,
        open_ended: bool,
        theta_start: f64,
        theta_length: f64,
    },
    Sphere {
        radius: f64,
        ws: u32,
        hs: u32,
        phi_start: f64,
        phi_length: f64,
        theta_start: f64,
        theta_length: f64,
    },
    Plane {
        width: f64,
        height: f64,
        ws: u32,
        hs: u32,
    },
    Circle {
        radius: f64,
        segments: u32,
        theta_start: f64,
        theta_length: f64,
    },
    Torus {
        radius: f64,
        tube: f64,
        radial: u32,
        tubular: u32,
        arc: f64,
    },
    Capsule {
        radius: f64,
        length: f64,
        cap_segments: u32,
        radial: u32,
    },
    Octahedron {
        radius: f64,
        detail: u32,
    },
    Icosahedron {
        radius: f64,
        detail: u32,
    },
    Dodecahedron {
        radius: f64,
        detail: u32,
    },
    Tetrahedron {
        radius: f64,
        detail: u32,
    },
    Ring {
        inner: f64,
        outer: f64,
        theta_seg: u32,
        phi_seg: u32,
        theta_start: f64,
        theta_length: f64,
    },
    Lathe {
        segments: u32,
        phi_start: f64,
        phi_length: f64,
    },
    Shape {
        curve_segments: u32,
    },
}
impl GeoParams {
    pub fn type_name(&self) -> &'static str {
        match self {
            GeoParams::Buffer => "buffer",
            GeoParams::Box { .. } => "box",
            GeoParams::Cylinder { .. } => "cylinder",
            GeoParams::Cone { .. } => "cone",
            GeoParams::Sphere { .. } => "sphere",
            GeoParams::Plane { .. } => "plane",
            GeoParams::Circle { .. } => "circle",
            GeoParams::Torus { .. } => "torus",
            GeoParams::Capsule { .. } => "capsule",
            GeoParams::Octahedron { .. } => "octahedron",
            GeoParams::Icosahedron { .. } => "icosahedron",
            GeoParams::Dodecahedron { .. } => "dodecahedron",
            GeoParams::Tetrahedron { .. } => "tetrahedron",
            GeoParams::Ring { .. } => "ring",
            GeoParams::Lathe { .. } => "lathe",
            GeoParams::Shape { .. } => "shape",
        }
    }
    // Convenience accessors for single parameters (0 when the kind has none).
    pub fn width(&self) -> f64 {
        match self {
            GeoParams::Box { width, .. } | GeoParams::Plane { width, .. } => *width,
            _ => 0.0,
        }
    }
    pub fn height(&self) -> f64 {
        match self {
            GeoParams::Box { height, .. }
            | GeoParams::Plane { height, .. }
            | GeoParams::Cylinder { height, .. }
            | GeoParams::Cone { height, .. } => *height,
            _ => 0.0,
        }
    }
    pub fn depth(&self) -> f64 {
        match self {
            GeoParams::Box { depth, .. } => *depth,
            _ => 0.0,
        }
    }
    pub fn radius(&self) -> f64 {
        match self {
            GeoParams::Sphere { radius, .. }
            | GeoParams::Circle { radius, .. }
            | GeoParams::Cone { radius, .. }
            | GeoParams::Torus { radius, .. }
            | GeoParams::Capsule { radius, .. }
            | GeoParams::Octahedron { radius, .. }
            | GeoParams::Icosahedron { radius, .. } => *radius,
            GeoParams::Cylinder { radius_top, radius_bottom, .. } => radius_top.max(*radius_bottom),
            _ => 0.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GeoGroup {
    pub start: u32,
    pub count: u32,
    pub material_index: u32,
}

#[derive(Clone, Debug, Default)]
pub struct Geometry {
    pub id: u64,
    pub params: GeoParams,
    pub position: Vec<f32>,
    pub normal: Vec<f32>,
    pub uv: Vec<f32>,
    /// vertex colours; `color_size` is 3 or 4
    pub color: Vec<f32>,
    pub color_size: u8,
    pub skin_index: Vec<f32>,
    pub skin_weight: Vec<f32>,
    /// morph targets (relative offsets when `morph_relative`)
    pub morph_position: Vec<Vec<f32>>,
    pub morph_normal: Vec<Vec<f32>>,
    pub morph_relative: bool,
    pub index: Option<Vec<u32>>,
    pub groups: Vec<GeoGroup>,
    pub draw_start: u32,
    pub draw_count: u32,
    pub bounding_box: Option<Box3>,
    pub bounding_sphere: Option<Sphere>,
    /// bumped whenever vertex data changes (the renderer re-uploads)
    pub version: u64,
    pub is_prop_part: bool,
    /// generic per-geometry flags some tools set
    pub user_tag: u32,
    /// bone names a skinned body binds to
    pub bone_names: Vec<String>,
}

impl Geometry {
    pub fn vertex_count(&self) -> usize {
        self.position.len() / 3
    }
    pub fn touch(&mut self) {
        self.version += 1;
        self.bounding_box = None;
        self.bounding_sphere = None;
    }
    pub fn add_group(&mut self, start: u32, count: u32, mi: u32) {
        self.groups.push(GeoGroup { start, count, material_index: mi });
    }
    /// Recompute the bounding box (morph targets included)
    pub fn compute_bounding_box(&mut self) -> Box3 {
        let b = self.position_box_with_morphs();
        self.bounding_box = Some(b);
        b
    }
    fn position_box_with_morphs(&self) -> Box3 {
        let bbox = |a: &[f32]| {
            let mut b = Box3::empty();
            for p in a.as_chunks::<3>().0 {
                b.expand_by_point(&Vec3::new(p[0] as f64, p[1] as f64, p[2] as f64));
            }
            b
        };
        let mut b = bbox(&self.position);
        for m in &self.morph_position {
            let bm = bbox(m);
            if self.morph_relative {
                let mut v = b.min;
                v.add(&bm.min);
                b.expand_by_point(&v);
                let mut v = b.max;
                v.add(&bm.max);
                b.expand_by_point(&v);
            } else {
                b.expand_by_point(&bm.min);
                b.expand_by_point(&bm.max);
            }
        }
        b
    }
    /// Recompute the bounding sphere (recomputes the box; morph targets included)
    pub fn compute_bounding_sphere(&mut self) -> Sphere {
        let center = if self.position.is_empty() { Vec3::ZERO } else { self.position_box_with_morphs().get_center() };
        let mut max_r2: f64 = 0.0;
        for p in self.position.as_chunks::<3>().0 {
            max_r2 = max_r2.max(center.distance_to_squared(&Vec3::new(p[0] as f64, p[1] as f64, p[2] as f64)));
        }
        for m in &self.morph_position {
            for (j, q) in m.as_chunks::<3>().0.iter().enumerate() {
                let mut v = Vec3::new(q[0] as f64, q[1] as f64, q[2] as f64);
                if self.morph_relative && j * 3 + 2 < self.position.len() {
                    v.add(&Vec3::new(self.position[j * 3] as f64, self.position[j * 3 + 1] as f64, self.position[j * 3 + 2] as f64));
                }
                max_r2 = max_r2.max(center.distance_to_squared(&v));
            }
        }
        let s = Sphere { center, radius: max_r2.sqrt() };
        self.bounding_sphere = Some(s);
        s
    }
    pub fn apply_matrix4(&mut self, m: &Mat4) {
        for p in self.position.as_chunks_mut::<3>().0 {
            let mut v = Vec3::new(p[0] as f64, p[1] as f64, p[2] as f64);
            v.apply_matrix4(m);
            p[0] = v.x as f32;
            p[1] = v.y as f32;
            p[2] = v.z as f32;
        }
        if !self.normal.is_empty() {
            let mut nm = Mat3::default();
            nm.get_normal_matrix(m);
            for n in self.normal.as_chunks_mut::<3>().0 {
                let mut v = Vec3::new(n[0] as f64, n[1] as f64, n[2] as f64);
                v.apply_matrix3(&nm).normalize();
                n[0] = v.x as f32;
                n[1] = v.y as f32;
                n[2] = v.z as f32;
            }
        }
        if self.bounding_box.is_some() {
            self.compute_bounding_box();
        }
        if self.bounding_sphere.is_some() {
            self.compute_bounding_sphere();
        }
        self.version += 1;
    }
    pub fn compute_vertex_normals(&mut self) {
        let n = self.vertex_count();
        let mut nrm = vec![0f32; n * 3];
        let p = &self.position;
        let get = |i: usize| Vec3::new(p[i * 3] as f64, p[i * 3 + 1] as f64, p[i * 3 + 2] as f64);
        let face = |a: usize, b: usize, c: usize, acc: &mut Vec<f32>, accumulate: bool| {
            let (pa, pb, pc) = (get(a), get(b), get(c));
            let mut cb = pc;
            cb.sub(&pb);
            let mut ab = pa;
            ab.sub(&pb);
            cb.cross(&ab);
            for &v in &[a, b, c] {
                if accumulate {
                    // accumulate in f32 storage, rounding after every face (deliberately)
                    acc[v * 3] = (acc[v * 3] as f64 + cb.x) as f32;
                    acc[v * 3 + 1] = (acc[v * 3 + 1] as f64 + cb.y) as f32;
                    acc[v * 3 + 2] = (acc[v * 3 + 2] as f64 + cb.z) as f32;
                } else {
                    acc[v * 3] = cb.x as f32;
                    acc[v * 3 + 1] = cb.y as f32;
                    acc[v * 3 + 2] = cb.z as f32;
                }
            }
        };
        if let Some(idx) = &self.index {
            for t in idx.as_chunks::<3>().0 {
                face(t[0] as usize, t[1] as usize, t[2] as usize, &mut nrm, true);
            }
        } else {
            for i in (0..n.saturating_sub(n % 3)).step_by(3) {
                face(i, i + 1, i + 2, &mut nrm, false);
            }
        }
        for v in nrm.as_chunks_mut::<3>().0 {
            let mut x = Vec3::new(v[0] as f64, v[1] as f64, v[2] as f64);
            x.normalize();
            v[0] = x.x as f32;
            v[1] = x.y as f32;
            v[2] = x.z as f32;
        }
        self.normal = nrm;
        self.version += 1;
    }
    pub fn normalize_normals(&mut self) {
        for v in self.normal.as_chunks_mut::<3>().0 {
            let mut x = Vec3::new(v[0] as f64, v[1] as f64, v[2] as f64);
            x.normalize();
            v[0] = x.x as f32;
            v[1] = x.y as f32;
            v[2] = x.z as f32;
        }
    }
    /// Expand an indexed geometry so every triangle has its own vertices.
    pub fn to_non_indexed(&self) -> Geometry {
        let Some(idx) = &self.index else {
            let mut g = self.clone();
            g.id = next_id();
            return g;
        };
        let pick = |src: &Vec<f32>, size: usize| -> Vec<f32> {
            if src.is_empty() {
                return vec![];
            }
            let mut o = Vec::with_capacity(idx.len() * size);
            for &i in idx {
                let i = i as usize;
                o.extend_from_slice(&src[i * size..i * size + size]);
            }
            o
        };
        let mut g = Geometry { id: next_id(), params: GeoParams::Buffer, ..Default::default() };
        g.position = pick(&self.position, 3);
        g.normal = pick(&self.normal, 3);
        g.uv = pick(&self.uv, 2);
        g.color = pick(&self.color, self.color_size.max(3) as usize);
        g.color_size = self.color_size;
        g.skin_index = pick(&self.skin_index, 4);
        g.skin_weight = pick(&self.skin_weight, 4);
        g.morph_position = self.morph_position.iter().map(|m| pick(m, 3)).collect();
        g.morph_normal = self.morph_normal.iter().map(|m| pick(m, 3)).collect();
        g.morph_relative = self.morph_relative;
        g.groups = self.groups.clone();
        g
    }
    pub fn index_count(&self) -> usize {
        match &self.index {
            Some(i) => i.len(),
            None => self.vertex_count(),
        }
    }
}

/// Shared, mutable geometry handle (clones of the handle share the geometry).
#[derive(Clone, Debug)]
pub struct Geo(pub Rc<RefCell<Geometry>>);
impl PartialEq for Geo {
    fn eq(&self, o: &Geo) -> bool {
        Rc::ptr_eq(&self.0, &o.0)
    }
}

impl Geo {
    pub fn from_geometry(mut g: Geometry) -> Geo {
        if g.id == 0 {
            g.id = next_id();
        }
        if g.draw_count == 0 {
            g.draw_count = u32::MAX;
        }
        Geo(Rc::new(RefCell::new(g)))
    }
    /// An empty geometry.
    pub fn new() -> Geo {
        Geo::from_geometry(Geometry::default())
    }
    pub fn g(&self) -> RefMut<'_, Geometry> {
        self.0.borrow_mut()
    }
    pub fn r(&self) -> Ref<'_, Geometry> {
        self.0.borrow()
    }
    pub fn id(&self) -> u64 {
        self.0.borrow().id
    }
    pub fn params(&self) -> GeoParams {
        self.0.borrow().params.clone()
    }
    /// The generator's name (`"box"`, `"torus"`, ...); fitted prop parts report `"buffer"`
    pub fn type_name(&self) -> &'static str {
        let g = self.0.borrow();
        if g.is_prop_part { "buffer" } else { g.params.type_name() }
    }
    pub fn ptr_eq(&self, o: &Geo) -> bool {
        Rc::ptr_eq(&self.0, &o.0)
    }
    /// A deep copy.
    pub fn duplicate(&self) -> Geo {
        let mut g = self.0.borrow().clone();
        g.id = next_id();
        Geo::from_geometry(g)
    }
    pub fn set_position(&self, p: Vec<f32>) -> &Self {
        let mut g = self.g();
        g.position = p;
        g.touch();
        drop(g);
        self
    }
    pub fn set_normal(&self, p: Vec<f32>) -> &Self {
        let mut g = self.g();
        g.normal = p;
        g.version += 1;
        drop(g);
        self
    }
    pub fn set_uv(&self, p: Vec<f32>) -> &Self {
        let mut g = self.g();
        g.uv = p;
        g.version += 1;
        drop(g);
        self
    }
    pub fn set_color(&self, p: Vec<f32>, size: u8) -> &Self {
        let mut g = self.g();
        g.color = p;
        g.color_size = size;
        g.version += 1;
        drop(g);
        self
    }
    pub fn set_index(&self, i: Vec<u32>) -> &Self {
        let mut g = self.g();
        g.index = Some(i);
        g.version += 1;
        drop(g);
        self
    }
    pub fn translate(&self, x: impl Num, y: impl Num, z: impl Num) -> &Self {
        let mut m = Mat4::IDENTITY;
        m.make_translation(x, y, z);
        self.g().apply_matrix4(&m);
        self
    }
    pub fn rotate_x(&self, a: impl Num) -> &Self {
        let mut m = Mat4::IDENTITY;
        m.make_rotation_x(a);
        self.g().apply_matrix4(&m);
        self
    }
    pub fn rotate_y(&self, a: impl Num) -> &Self {
        let mut m = Mat4::IDENTITY;
        m.make_rotation_y(a);
        self.g().apply_matrix4(&m);
        self
    }
    pub fn rotate_z(&self, a: impl Num) -> &Self {
        let mut m = Mat4::IDENTITY;
        m.make_rotation_z(a);
        self.g().apply_matrix4(&m);
        self
    }
    pub fn scale(&self, x: impl Num, y: impl Num, z: impl Num) -> &Self {
        let mut m = Mat4::IDENTITY;
        m.make_scale(x, y, z);
        self.g().apply_matrix4(&m);
        self
    }
    pub fn apply_matrix4(&self, m: &Mat4) -> &Self {
        self.g().apply_matrix4(m);
        self
    }
    pub fn apply_quaternion(&self, q: &Quat) -> &Self {
        self.g().apply_matrix4(&Mat4::from_quat(q));
        self
    }
    pub fn center(&self) -> &Self {
        let c = self.g().compute_bounding_box().get_center();
        self.translate(-c.x, -c.y, -c.z)
    }
    pub fn compute_vertex_normals(&self) -> &Self {
        self.g().compute_vertex_normals();
        self
    }
    pub fn compute_bounding_box(&self) -> Box3 {
        self.g().compute_bounding_box()
    }
    pub fn compute_bounding_sphere(&self) -> Sphere {
        self.g().compute_bounding_sphere()
    }
    /// The bounding box (computing it if needed).
    pub fn bounding_box(&self) -> Box3 {
        let b = self.0.borrow().bounding_box;
        match b {
            Some(b) => b,
            None => self.compute_bounding_box(),
        }
    }
    pub fn bounding_sphere(&self) -> Sphere {
        let b = self.0.borrow().bounding_sphere;
        match b {
            Some(b) => b,
            None => self.compute_bounding_sphere(),
        }
    }
    pub fn touch(&self) {
        self.g().version += 1;
    }
    pub fn vertex_count(&self) -> usize {
        self.0.borrow().vertex_count()
    }
    pub fn to_non_indexed(&self) -> Geo {
        Geo::from_geometry(self.0.borrow().to_non_indexed())
    }
    pub fn dispose(&self) {}
    /// A position-only geometry from 3D points.
    pub fn from_points(pts: &[Vec3]) -> Geo {
        let g = Geo::new();
        g.set_position(pts.iter().flat_map(|p| [p.x as f32, p.y as f32, p.z as f32]).collect());
        g
    }
    /// A position-only geometry from 2D points (z = 0).
    pub fn from_points2(pts: &[Vec2]) -> Geo {
        let g = Geo::new();
        g.set_position(pts.iter().flat_map(|p| [p.x as f32, p.y as f32, 0.0]).collect());
        g
    }
}
impl Default for Geo {
    fn default() -> Self {
        Geo::new()
    }
}
impl From<Geometry> for Geo {
    fn from(g: Geometry) -> Geo {
        Geo::from_geometry(g)
    }
}

// ===================================================================== generators
fn new_geo(params: GeoParams) -> Geometry {
    Geometry { id: next_id(), params, draw_count: u32::MAX, ..Default::default() }
}

/// Box of `width × height × depth` centred on the origin, `ws × hs × ds` segments,
/// one group per face (+x, -x, +y, -y, +z, -z).
pub fn box_geometry(width: f64, height: f64, depth: f64, ws: f64, hs: f64, ds: f64) -> Geo {
    let (ws, hs, ds) = (ws.floor() as u32, hs.floor() as u32, ds.floor() as u32);
    let mut g = new_geo(GeoParams::Box { width, height, depth, ws, hs, ds });
    let mut idx: Vec<u32> = vec![];
    let (mut pos, mut nrm, mut uv) = (vec![], vec![], vec![]);
    let mut nv: u32 = 0;
    let mut group_start = 0u32;
    // one face: plane(u, v, w, udir, vdir, width, height, depth, grid x, grid y, material index, g)
    let mut plane = |u: usize,
                     v: usize,
                     w: usize,
                     udir: f64,
                     vdir: f64,
                     width: f64,
                     height: f64,
                     depth: f64,
                     gx: u32,
                     gy: u32,
                     mi: u32,
                     g: &mut Geometry| {
        let sw = width / gx as f64;
        let sh = height / gy as f64;
        let (wh, hh, dh) = (width / 2.0, height / 2.0, depth / 2.0);
        let (gx1, gy1) = (gx + 1, gy + 1);
        let mut vc = 0u32;
        let mut gc = 0u32;
        for iy in 0..gy1 {
            let y = iy as f64 * sh - hh;
            for ix in 0..gx1 {
                let x = ix as f64 * sw - wh;
                let mut a = [0.0f64; 3];
                a[u] = x * udir;
                a[v] = y * vdir;
                a[w] = dh;
                pos.extend_from_slice(&[a[0] as f32, a[1] as f32, a[2] as f32]);
                let mut n = [0.0f64; 3];
                n[w] = if depth > 0.0 { 1.0 } else { -1.0 };
                nrm.extend_from_slice(&[n[0] as f32, n[1] as f32, n[2] as f32]);
                uv.push((ix as f64 / gx as f64) as f32);
                uv.push((1.0 - iy as f64 / gy as f64) as f32);
                vc += 1;
            }
        }
        for iy in 0..gy {
            for ix in 0..gx {
                let a = nv + ix + gx1 * iy;
                let b = nv + ix + gx1 * (iy + 1);
                let c = nv + (ix + 1) + gx1 * (iy + 1);
                let d = nv + (ix + 1) + gx1 * iy;
                idx.extend_from_slice(&[a, b, d, b, c, d]);
                gc += 6;
            }
        }
        g.add_group(group_start, gc, mi);
        group_start += gc;
        nv += vc;
    };
    // axes: x=0, y=1, z=2
    plane(2, 1, 0, -1.0, -1.0, depth, height, width, ds, hs, 0, &mut g);
    plane(2, 1, 0, 1.0, -1.0, depth, height, -width, ds, hs, 1, &mut g);
    plane(0, 2, 1, 1.0, 1.0, width, depth, height, ws, ds, 2, &mut g);
    plane(0, 2, 1, 1.0, -1.0, width, depth, -height, ws, ds, 3, &mut g);
    plane(0, 1, 2, 1.0, -1.0, width, height, depth, ws, hs, 4, &mut g);
    plane(0, 1, 2, -1.0, -1.0, width, height, -depth, ws, hs, 5, &mut g);
    g.position = pos;
    g.normal = nrm;
    g.uv = uv;
    g.index = Some(idx);
    Geo::from_geometry(g)
}

/// Plane of `width × height` in XY facing +Z, `ws × hs` segments.
pub fn plane_geometry(width: f64, height: f64, ws: f64, hs: f64) -> Geo {
    let (gx, gy) = (ws.floor() as u32, hs.floor() as u32);
    let mut g = new_geo(GeoParams::Plane { width, height, ws: gx, hs: gy });
    let (wh, hh) = (width / 2.0, height / 2.0);
    let (gx1, gy1) = (gx + 1, gy + 1);
    let (sw, sh) = (width / gx as f64, height / gy as f64);
    for iy in 0..gy1 {
        let y = iy as f64 * sh - hh;
        for ix in 0..gx1 {
            let x = ix as f64 * sw - wh;
            g.position.extend_from_slice(&[x as f32, (-y) as f32, 0.0]);
            g.normal.extend_from_slice(&[0.0, 0.0, 1.0]);
            g.uv.push((ix as f64 / gx as f64) as f32);
            g.uv.push((1.0 - iy as f64 / gy as f64) as f32);
        }
    }
    let mut idx = vec![];
    for iy in 0..gy {
        for ix in 0..gx {
            let a = ix + gx1 * iy;
            let b = ix + gx1 * (iy + 1);
            let c = ix + 1 + gx1 * (iy + 1);
            let d = ix + 1 + gx1 * iy;
            idx.extend_from_slice(&[a, b, d, b, c, d]);
        }
    }
    g.index = Some(idx);
    Geo::from_geometry(g)
}

/// Disc (or sector) in XY facing +Z, from `theta_start` over `theta_length`.
pub fn circle_geometry(radius: f64, segments: f64, theta_start: f64, theta_length: f64) -> Geo {
    let segs = (segments.max(3.0)) as u32;
    let mut g = new_geo(GeoParams::Circle { radius, segments: segs, theta_start, theta_length });
    g.position.extend_from_slice(&[0.0, 0.0, 0.0]);
    g.normal.extend_from_slice(&[0.0, 0.0, 1.0]);
    g.uv.extend_from_slice(&[0.5, 0.5]);
    for s in 0..=segs {
        let seg = theta_start + s as f64 / segs as f64 * theta_length;
        let x = (radius * seg.cos()) as f32;
        let y = (radius * seg.sin()) as f32;
        g.position.extend_from_slice(&[x, y, 0.0]);
        g.normal.extend_from_slice(&[0.0, 0.0, 1.0]);
        // from the f32-rounded stored position (deliberately)
        g.uv.push(((x as f64 / radius + 1.0) / 2.0) as f32);
        g.uv.push(((y as f64 / radius + 1.0) / 2.0) as f32);
    }
    let mut idx = vec![];
    for i in 1..=segs {
        idx.extend_from_slice(&[i, i + 1, 0]);
    }
    g.index = Some(idx);
    Geo::from_geometry(g)
}

fn cylinder_into(
    g: &mut Geometry,
    radius_top: f64,
    radius_bottom: f64,
    height: f64,
    radial: u32,
    height_seg: u32,
    open_ended: bool,
    theta_start: f64,
    theta_length: f64,
) {
    let mut idx: Vec<u32> = vec![];
    let mut index: u32 = 0;
    let mut grid: Vec<Vec<u32>> = vec![];
    let half = height / 2.0;
    let mut group_start = 0u32;
    // torso
    {
        let mut gc = 0u32;
        let slope = (radius_bottom - radius_top) / height;
        for y in 0..=height_seg {
            let mut row = vec![];
            let v = y as f64 / height_seg as f64;
            let radius = v * (radius_bottom - radius_top) + radius_top;
            for x in 0..=radial {
                let u = x as f64 / radial as f64;
                let theta = u * theta_length + theta_start;
                let (st, ct) = (theta.sin(), theta.cos());
                g.position.extend_from_slice(&[(radius * st) as f32, (-v * height + half) as f32, (radius * ct) as f32]);
                let mut n = Vec3::new(st, slope, ct);
                n.normalize();
                g.normal.extend_from_slice(&[n.x as f32, n.y as f32, n.z as f32]);
                g.uv.extend_from_slice(&[u as f32, (1.0 - v) as f32]);
                row.push(index);
                index += 1;
            }
            grid.push(row);
        }
        for x in 0..radial as usize {
            for y in 0..height_seg as usize {
                let a = grid[y][x];
                let b = grid[y + 1][x];
                let c = grid[y + 1][x + 1];
                let d = grid[y][x + 1];
                idx.extend_from_slice(&[a, b, d, b, c, d]);
                gc += 6;
            }
        }
        g.add_group(group_start, gc, 0);
        group_start += gc;
    }
    let cap = |top: bool, g: &mut Geometry, idx: &mut Vec<u32>, index: &mut u32, group_start: &mut u32| {
        let center_start = *index;
        let radius = if top { radius_top } else { radius_bottom };
        let sign = if top { 1.0 } else { -1.0 };
        let mut gc = 0u32;
        for _ in 1..=radial {
            g.position.extend_from_slice(&[0.0, (half * sign) as f32, 0.0]);
            g.normal.extend_from_slice(&[0.0, sign as f32, 0.0]);
            g.uv.extend_from_slice(&[0.5, 0.5]);
            *index += 1;
        }
        let center_end = *index;
        for x in 0..=radial {
            let u = x as f64 / radial as f64;
            let theta = u * theta_length + theta_start;
            let (ct, st) = (theta.cos(), theta.sin());
            g.position.extend_from_slice(&[(radius * st) as f32, (half * sign) as f32, (radius * ct) as f32]);
            g.normal.extend_from_slice(&[0.0, sign as f32, 0.0]);
            g.uv.extend_from_slice(&[(ct * 0.5 + 0.5) as f32, (st * 0.5 * sign + 0.5) as f32]);
            *index += 1;
        }
        for x in 0..radial {
            let c = center_start + x;
            let i = center_end + x;
            if top {
                idx.extend_from_slice(&[i, i + 1, c]);
            } else {
                idx.extend_from_slice(&[i + 1, i, c]);
            }
            gc += 3;
        }
        g.add_group(*group_start, gc, if top { 1 } else { 2 });
        *group_start += gc;
    };
    if !open_ended {
        if radius_top > 0.0 {
            cap(true, g, &mut idx, &mut index, &mut group_start);
        }
        if radius_bottom > 0.0 {
            cap(false, g, &mut idx, &mut index, &mut group_start);
        }
    }
    g.index = Some(idx);
}

/// Cylinder (or frustum) along Y centred on the origin; caps unless `open_ended`.
pub fn cylinder_geometry(
    radius_top: f64,
    radius_bottom: f64,
    height: f64,
    radial: f64,
    height_seg: f64,
    open_ended: bool,
    theta_start: f64,
    theta_length: f64,
) -> Geo {
    let (r, h) = (radial.floor() as u32, height_seg.floor() as u32);
    let mut g =
        new_geo(GeoParams::Cylinder { radius_top, radius_bottom, height, radial: r, height_seg: h, open_ended, theta_start, theta_length });
    cylinder_into(&mut g, radius_top, radius_bottom, height, r, h, open_ended, theta_start, theta_length);
    Geo::from_geometry(g)
}
/// Cone along Y centred on the origin (a cylinder with a zero top radius).
pub fn cone_geometry(radius: f64, height: f64, radial: f64, height_seg: f64, open_ended: bool, theta_start: f64, theta_length: f64) -> Geo {
    let (r, h) = (radial.floor() as u32, height_seg.floor() as u32);
    let mut g = new_geo(GeoParams::Cone { radius, height, radial: r, height_seg: h, open_ended, theta_start, theta_length });
    cylinder_into(&mut g, 0.0, radius, height, r, h, open_ended, theta_start, theta_length);
    Geo::from_geometry(g)
}

/// UV sphere: `ws` segments around Y (phi), `hs` from +Y down (theta).
pub fn sphere_geometry(radius: f64, ws: f64, hs: f64, phi_start: f64, phi_length: f64, theta_start: f64, theta_length: f64) -> Geo {
    let ws = (ws.floor().max(3.0)) as u32;
    let hs = (hs.floor().max(2.0)) as u32;
    let mut g = new_geo(GeoParams::Sphere { radius, ws, hs, phi_start, phi_length, theta_start, theta_length });
    let theta_end = (theta_start + theta_length).min(PI);
    let mut index = 0u32;
    let mut grid: Vec<Vec<u32>> = vec![];
    for iy in 0..=hs {
        let mut row = vec![];
        let v = iy as f64 / hs as f64;
        let mut u_off = 0.0;
        if iy == 0 && theta_start == 0.0 {
            u_off = 0.5 / ws as f64;
        } else if iy == hs && theta_end == PI {
            u_off = -0.5 / ws as f64;
        }
        for ix in 0..=ws {
            let u = ix as f64 / ws as f64;
            let x = -radius * (phi_start + u * phi_length).cos() * (theta_start + v * theta_length).sin();
            let y = radius * (theta_start + v * theta_length).cos();
            let z = radius * (phi_start + u * phi_length).sin() * (theta_start + v * theta_length).sin();
            g.position.extend_from_slice(&[x as f32, y as f32, z as f32]);
            let mut n = Vec3::new(x, y, z);
            n.normalize();
            g.normal.extend_from_slice(&[n.x as f32, n.y as f32, n.z as f32]);
            g.uv.extend_from_slice(&[(u + u_off) as f32, (1.0 - v) as f32]);
            row.push(index);
            index += 1;
        }
        grid.push(row);
    }
    let mut idx = vec![];
    for iy in 0..hs as usize {
        for ix in 0..ws as usize {
            let a = grid[iy][ix + 1];
            let b = grid[iy][ix];
            let c = grid[iy + 1][ix];
            let d = grid[iy + 1][ix + 1];
            if iy != 0 || theta_start > 0.0 {
                idx.extend_from_slice(&[a, b, d]);
            }
            if iy != hs as usize - 1 || theta_end < PI {
                idx.extend_from_slice(&[b, c, d]);
            }
        }
    }
    g.index = Some(idx);
    Geo::from_geometry(g)
}

/// Torus around Z (ring radius `radius`, tube radius `tube`), swept over `arc`.
pub fn torus_geometry(radius: f64, tube: f64, radial: f64, tubular: f64, arc: f64) -> Geo {
    let (rs, ts) = (radial.floor() as u32, tubular.floor() as u32);
    let mut g = new_geo(GeoParams::Torus { radius, tube, radial: rs, tubular: ts, arc });
    for j in 0..=rs {
        for i in 0..=ts {
            let u = i as f64 / ts as f64 * arc;
            let v = j as f64 / rs as f64 * PI * 2.0;
            let x = (radius + tube * v.cos()) * u.cos();
            let y = (radius + tube * v.cos()) * u.sin();
            let z = tube * v.sin();
            g.position.extend_from_slice(&[x as f32, y as f32, z as f32]);
            let c = Vec3::new(radius * u.cos(), radius * u.sin(), 0.0);
            let mut n = Vec3::new(x, y, z);
            n.sub(&c).normalize();
            g.normal.extend_from_slice(&[n.x as f32, n.y as f32, n.z as f32]);
            g.uv.push((i as f64 / ts as f64) as f32);
            g.uv.push((j as f64 / rs as f64) as f32);
        }
    }
    let mut idx = vec![];
    for j in 1..=rs {
        for i in 1..=ts {
            let a = (ts + 1) * j + i - 1;
            let b = (ts + 1) * (j - 1) + i - 1;
            let c = (ts + 1) * (j - 1) + i;
            let d = (ts + 1) * j + i;
            idx.extend_from_slice(&[a, b, d, b, c, d]);
        }
    }
    g.index = Some(idx);
    Geo::from_geometry(g)
}

/// Flat annulus in XY facing +Z.
pub fn ring_geometry(inner: f64, outer: f64, theta_seg: f64, phi_seg: f64, theta_start: f64, theta_length: f64) -> Geo {
    let ts = theta_seg.max(3.0) as u32;
    let ps = phi_seg.max(1.0) as u32;
    let mut g = new_geo(GeoParams::Ring { inner, outer, theta_seg: ts, phi_seg: ps, theta_start, theta_length });
    let mut r = inner;
    let step = (outer - inner) / ps as f64;
    for _ in 0..=ps {
        for i in 0..=ts {
            let seg = theta_start + i as f64 / ts as f64 * theta_length;
            let x = r * seg.cos();
            let y = r * seg.sin();
            g.position.extend_from_slice(&[x as f32, y as f32, 0.0]);
            g.normal.extend_from_slice(&[0.0, 0.0, 1.0]);
            g.uv.push(((x / outer + 1.0) / 2.0) as f32);
            g.uv.push(((y / outer + 1.0) / 2.0) as f32);
        }
        r += step;
    }
    let mut idx = vec![];
    for j in 0..ps {
        let seg = j * (ts + 1);
        for i in 0..ts {
            let s = i + seg;
            let (a, b, c, d) = (s, s + ts + 1, s + ts + 2, s + 1);
            idx.extend_from_slice(&[a, b, d, b, c, d]);
        }
    }
    g.index = Some(idx);
    Geo::from_geometry(g)
}

/// Surface of revolution: 2D points (x = radius, y = height) swept around Y.
pub fn lathe_geometry(points: &[Vec2], segments: f64, phi_start: f64, phi_length: f64) -> Geo {
    let segs = segments.floor() as u32;
    let phi_length = clamp(phi_length, 0.0, PI * 2.0);
    let mut g = new_geo(GeoParams::Lathe { segments: segs, phi_start, phi_length });
    lathe_into(&mut g, points, segs, phi_start, phi_length);
    Geo::from_geometry(g)
}
fn lathe_into(g: &mut Geometry, points: &[Vec2], segs: u32, phi_start: f64, phi_length: f64) {
    let inv = 1.0 / segs as f64;
    let n = points.len();
    let mut init_normals: Vec<f64> = vec![];
    let mut prev = Vec3::ZERO;
    for j in 0..n {
        if j == 0 {
            let dx = points[1].x - points[0].x;
            let dy = points[1].y - points[0].y;
            let mut nn = Vec3::new(dy, -dx, 0.0);
            prev = nn;
            nn.normalize();
            init_normals.extend_from_slice(&[nn.x, nn.y, nn.z]);
        } else if j == n - 1 {
            init_normals.extend_from_slice(&[prev.x, prev.y, prev.z]);
        } else {
            let dx = points[j + 1].x - points[j].x;
            let dy = points[j + 1].y - points[j].y;
            let cur = Vec3::new(dy, -dx, 0.0);
            let mut nn = cur;
            nn.x += prev.x;
            nn.y += prev.y;
            nn.z += prev.z;
            nn.normalize();
            init_normals.extend_from_slice(&[nn.x, nn.y, nn.z]);
            prev = cur;
        }
    }
    for i in 0..=segs {
        let phi = phi_start + i as f64 * inv * phi_length;
        let (s, c) = (phi.sin(), phi.cos());
        for j in 0..n {
            g.position.extend_from_slice(&[(points[j].x * s) as f32, points[j].y as f32, (points[j].x * c) as f32]);
            g.uv.extend_from_slice(&[(i as f64 / segs as f64) as f32, (j as f64 / (n - 1) as f64) as f32]);
            g.normal.extend_from_slice(&[
                (init_normals[3 * j] * s) as f32,
                init_normals[3 * j + 1] as f32,
                (init_normals[3 * j] * c) as f32,
            ]);
        }
    }
    let mut idx = vec![];
    for i in 0..segs as usize {
        for j in 0..n - 1 {
            let base = j + i * n;
            let (a, b, c, d) = (base, base + n, base + n + 1, base + 1);
            idx.extend_from_slice(&[a as u32, b as u32, d as u32, c as u32, d as u32, b as u32]);
        }
    }
    g.index = Some(idx);
}

/// Capsule along Y: a cylinder of `length` with hemispherical caps (a lathe).
pub fn capsule_geometry(radius: f64, length: f64, cap_segments: f64, radial: f64) -> Geo {
    let mut path = Path2::new();
    path.absarc(0.0, -length / 2.0, radius, PI * 1.5, 0.0, false);
    path.absarc(0.0, length / 2.0, radius, 0.0, PI * 0.5, false);
    let pts = path.get_points(cap_segments as u32);
    let mut g = new_geo(GeoParams::Capsule { radius, length, cap_segments: cap_segments as u32, radial: radial as u32 });
    lathe_into(&mut g, &pts, radial.floor() as u32, 0.0, PI * 2.0);
    Geo::from_geometry(g)
}

fn polyhedron(vertices: &[f64], indices: &[u32], radius: f64, detail: u32, params: GeoParams) -> Geo {
    let mut buf: Vec<f64> = vec![];
    let mut uvb: Vec<f64> = vec![];
    let vtx = |i: u32| Vec3::new(vertices[i as usize * 3], vertices[i as usize * 3 + 1], vertices[i as usize * 3 + 2]);
    // subdivide
    for t in indices.as_chunks::<3>().0 {
        let (a, b, c) = (vtx(t[0]), vtx(t[1]), vtx(t[2]));
        let cols = detail + 1;
        let mut v: Vec<Vec<Vec3>> = vec![];
        for i in 0..=cols {
            let mut aj = a;
            aj.lerp(&c, i as f64 / cols as f64);
            let mut bj = b;
            bj.lerp(&c, i as f64 / cols as f64);
            let rows = cols - i;
            let mut row = vec![];
            for j in 0..=rows {
                if j == 0 && i == cols {
                    row.push(aj);
                } else {
                    let mut p = aj;
                    p.lerp(&bj, j as f64 / rows as f64);
                    row.push(p);
                }
            }
            v.push(row);
        }
        let mut push = |p: Vec3| buf.extend_from_slice(&[p.x, p.y, p.z]);
        for i in 0..cols as usize {
            for j in 0..(2 * (cols as usize - i) - 1) {
                let k = j / 2;
                if j % 2 == 0 {
                    push(v[i][k + 1]);
                    push(v[i + 1][k]);
                    push(v[i][k]);
                } else {
                    push(v[i][k + 1]);
                    push(v[i + 1][k + 1]);
                    push(v[i + 1][k]);
                }
            }
        }
    }
    // apply radius
    for p in buf.as_chunks_mut::<3>().0 {
        let mut v = Vec3::new(p[0], p[1], p[2]);
        v.normalize().multiply_scalar(radius);
        p[0] = v.x;
        p[1] = v.y;
        p[2] = v.z;
    }
    let azimuth = |v: &Vec3| v.z.atan2(-v.x);
    let inclination = |v: &Vec3| (-v.y).atan2((v.x * v.x + v.z * v.z).sqrt());
    for p in buf.as_chunks::<3>().0 {
        let v = Vec3::new(p[0], p[1], p[2]);
        let u = azimuth(&v) / 2.0 / PI + 0.5;
        let w = inclination(&v) / PI + 0.5;
        uvb.push(u);
        uvb.push(1.0 - w);
    }
    // correct uvs
    let mut i = 0;
    let mut j = 0;
    while i < buf.len() {
        let a = Vec3::new(buf[i], buf[i + 1], buf[i + 2]);
        let b = Vec3::new(buf[i + 3], buf[i + 4], buf[i + 5]);
        let c = Vec3::new(buf[i + 6], buf[i + 7], buf[i + 8]);
        let centroid = a.added(b).added(c).scaled(1.0 / 3.0);
        let az = azimuth(&centroid);
        let fix = |uvb: &mut Vec<f64>, st: usize, vec: &Vec3, az: f64| {
            if az < 0.0 && uvb[st] == 1.0 {
                uvb[st] -= 1.0;
            }
            if vec.x == 0.0 && vec.z == 0.0 {
                uvb[st] = az / 2.0 / PI + 0.5;
            }
        };
        fix(&mut uvb, j, &a, az);
        fix(&mut uvb, j + 2, &b, az);
        fix(&mut uvb, j + 4, &c, az);
        i += 9;
        j += 6;
    }
    // correct seam
    let mut k = 0;
    while k < uvb.len() {
        let (x0, x1, x2) = (uvb[k], uvb[k + 2], uvb[k + 4]);
        let max = x0.max(x1).max(x2);
        let min = x0.min(x1).min(x2);
        if max > 0.9 && min < 0.1 {
            if x0 < 0.2 {
                uvb[k] += 1.0;
            }
            if x1 < 0.2 {
                uvb[k + 2] += 1.0;
            }
            if x2 < 0.2 {
                uvb[k + 4] += 1.0;
            }
        }
        k += 6;
    }
    let mut g = new_geo(params);
    g.position = buf.iter().map(|&x| x as f32).collect();
    g.normal = g.position.clone();
    g.uv = uvb.iter().map(|&x| x as f32).collect();
    if detail == 0 {
        g.compute_vertex_normals();
    } else {
        g.normalize_normals();
    }
    Geo::from_geometry(g)
}

/// Octahedron; `detail` subdivides each face.
pub fn octahedron_geometry(radius: f64, detail: f64) -> Geo {
    let v = [1.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, -1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, -1.0];
    let i = [0, 2, 4, 0, 4, 3, 0, 3, 5, 0, 5, 2, 1, 2, 5, 1, 5, 3, 1, 3, 4, 1, 4, 2];
    polyhedron(&v, &i, radius, detail as u32, GeoParams::Octahedron { radius, detail: detail as u32 })
}
/// Tetrahedron; `detail` subdivides each face.
pub fn tetrahedron_geometry(radius: f64, detail: f64) -> Geo {
    let v = [1.0, 1.0, 1.0, -1.0, -1.0, 1.0, -1.0, 1.0, -1.0, 1.0, -1.0, -1.0];
    let i = [2, 1, 0, 0, 3, 2, 1, 3, 0, 2, 3, 1];
    polyhedron(&v, &i, radius, detail as u32, GeoParams::Tetrahedron { radius, detail: detail as u32 })
}
/// Icosahedron; `detail` subdivides each face.
pub fn icosahedron_geometry(radius: f64, detail: f64) -> Geo {
    let t = (1.0 + 5f64.sqrt()) / 2.0;
    let v = [
        -1.0, t, 0.0, 1.0, t, 0.0, -1.0, -t, 0.0, 1.0, -t, 0.0, 0.0, -1.0, t, 0.0, 1.0, t, 0.0, -1.0, -t, 0.0, 1.0, -t, t, 0.0, -1.0, t,
        0.0, 1.0, -t, 0.0, -1.0, -t, 0.0, 1.0,
    ];
    let i = [
        0, 11, 5, 0, 5, 1, 0, 1, 7, 0, 7, 10, 0, 10, 11, 1, 5, 9, 5, 11, 4, 11, 10, 2, 10, 7, 6, 7, 1, 8, 3, 9, 4, 3, 4, 2, 3, 2, 6, 3, 6,
        8, 3, 8, 9, 4, 9, 5, 2, 4, 11, 6, 2, 10, 8, 6, 7, 9, 8, 1,
    ];
    polyhedron(&v, &i, radius, detail as u32, GeoParams::Icosahedron { radius, detail: detail as u32 })
}
/// Dodecahedron; `detail` subdivides each face.
pub fn dodecahedron_geometry(radius: f64, detail: f64) -> Geo {
    let t = (1.0 + 5f64.sqrt()) / 2.0;
    let r = 1.0 / t;
    let v = [
        -1.0, -1.0, -1.0, -1.0, -1.0, 1.0, -1.0, 1.0, -1.0, -1.0, 1.0, 1.0, 1.0, -1.0, -1.0, 1.0, -1.0, 1.0, 1.0, 1.0, -1.0, 1.0, 1.0, 1.0,
        0.0, -r, -t, 0.0, -r, t, 0.0, r, -t, 0.0, r, t, -r, -t, 0.0, -r, t, 0.0, r, -t, 0.0, r, t, 0.0, -t, 0.0, -r, t, 0.0, -r, -t, 0.0,
        r, t, 0.0, r,
    ];
    let i = [
        3, 11, 7, 3, 7, 15, 3, 15, 13, 7, 19, 17, 7, 17, 6, 7, 6, 15, 17, 4, 8, 17, 8, 10, 17, 10, 6, 8, 0, 16, 8, 16, 2, 8, 2, 10, 0, 12,
        1, 0, 1, 18, 0, 18, 16, 6, 10, 2, 6, 2, 13, 6, 13, 15, 2, 16, 18, 2, 18, 3, 2, 3, 13, 18, 1, 9, 18, 9, 11, 18, 11, 3, 4, 14, 12, 4,
        12, 0, 4, 0, 8, 11, 9, 5, 11, 5, 19, 11, 19, 7, 19, 5, 14, 19, 14, 4, 19, 4, 17, 1, 12, 14, 1, 14, 5, 1, 5, 9,
    ];
    polyhedron(&v, &i, radius, detail as u32, GeoParams::Dodecahedron { radius, detail: detail as u32 })
}

// ===================================================================== 2D paths / shapes
#[derive(Clone, Debug)]
enum Curve2 {
    Line(Vec2, Vec2),
    Quad(Vec2, Vec2, Vec2),
    Cubic(Vec2, Vec2, Vec2, Vec2),
    Ellipse { x: f64, y: f64, rx: f64, ry: f64, a0: f64, a1: f64, cw: bool, rot: f64 },
}
impl Curve2 {
    fn point(&self, t: f64) -> Vec2 {
        match *self {
            Curve2::Line(a, b) => {
                if t == 1.0 {
                    b
                } else {
                    Vec2::new(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t)
                }
            }
            Curve2::Quad(p0, p1, p2) => {
                let k = 1.0 - t;
                Vec2::new(k * k * p0.x + 2.0 * k * t * p1.x + t * t * p2.x, k * k * p0.y + 2.0 * k * t * p1.y + t * t * p2.y)
            }
            Curve2::Cubic(p0, p1, p2, p3) => {
                let b = |p0: f64, p1: f64, p2: f64, p3: f64| {
                    let k = 1.0 - t;
                    k * k * k * p0 + 3.0 * k * k * t * p1 + 3.0 * k * t * t * p2 + t * t * t * p3
                };
                Vec2::new(b(p0.x, p1.x, p2.x, p3.x), b(p0.y, p1.y, p2.y, p3.y))
            }
            Curve2::Ellipse { x, y, rx, ry, a0, a1, cw, rot } => {
                let two_pi = PI * 2.0;
                let mut delta = a1 - a0;
                let same = delta.abs() < f64::EPSILON;
                while delta < 0.0 {
                    delta += two_pi;
                }
                while delta > two_pi {
                    delta -= two_pi;
                }
                if delta < f64::EPSILON {
                    delta = if same { 0.0 } else { two_pi };
                }
                if cw && !same {
                    if delta == two_pi {
                        delta = -two_pi;
                    } else {
                        delta -= two_pi;
                    }
                }
                let angle = a0 + t * delta;
                let mut px = x + rx * angle.cos();
                let mut py = y + ry * angle.sin();
                if rot != 0.0 {
                    let (c, s) = (rot.cos(), rot.sin());
                    let tx = px - x;
                    let ty = py - y;
                    px = tx * c - ty * s + x;
                    py = tx * s + ty * c + y;
                }
                Vec2::new(px, py)
            }
        }
    }
    fn points(&self, divisions: u32) -> Vec<Vec2> {
        (0..=divisions).map(|d| self.point(d as f64 / divisions as f64)).collect()
    }
}

/// 2D outline of lines, curves and arcs, with optional holes (used by shape and
/// capsule geometries).
#[derive(Clone, Debug, Default)]
pub struct Path2 {
    curves: Vec<Curve2>,
    current: Vec2,
    pub holes: Vec<Path2>,
    pub auto_close: bool,
}
pub type Shape = Path2;
impl Path2 {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn from_points(pts: &[Vec2]) -> Self {
        let mut p = Path2::new();
        if let Some(f) = pts.first() {
            p.move_to(f.x, f.y);
            for q in &pts[1..] {
                p.line_to(q.x, q.y);
            }
        }
        p
    }
    pub fn move_to(&mut self, x: impl Num, y: impl Num) -> &mut Self {
        self.current = Vec2::new(x.f(), y.f());
        self
    }
    pub fn line_to(&mut self, x: impl Num, y: impl Num) -> &mut Self {
        let p = Vec2::new(x.f(), y.f());
        self.curves.push(Curve2::Line(self.current, p));
        self.current = p;
        self
    }
    pub fn quadratic_curve_to(&mut self, cx: impl Num, cy: impl Num, x: impl Num, y: impl Num) -> &mut Self {
        let c = Vec2::new(cx.f(), cy.f());
        let p = Vec2::new(x.f(), y.f());
        self.curves.push(Curve2::Quad(self.current, c, p));
        self.current = p;
        self
    }
    pub fn bezier_curve_to(&mut self, c1x: impl Num, c1y: impl Num, c2x: impl Num, c2y: impl Num, x: impl Num, y: impl Num) -> &mut Self {
        let c1 = Vec2::new(c1x.f(), c1y.f());
        let c2 = Vec2::new(c2x.f(), c2y.f());
        let p = Vec2::new(x.f(), y.f());
        self.curves.push(Curve2::Cubic(self.current, c1, c2, p));
        self.current = p;
        self
    }
    pub fn absellipse(&mut self, x: f64, y: f64, rx: f64, ry: f64, a0: f64, a1: f64, cw: bool, rot: f64) -> &mut Self {
        let c = Curve2::Ellipse { x, y, rx, ry, a0, a1, cw, rot };
        if !self.curves.is_empty() {
            let f = c.point(0.0);
            if f != self.current {
                self.line_to(f.x, f.y);
            }
        }
        let last = c.point(1.0);
        self.curves.push(c);
        self.current = last;
        self
    }
    pub fn absarc(&mut self, x: f64, y: f64, r: f64, a0: f64, a1: f64, cw: bool) -> &mut Self {
        self.absellipse(x, y, r, r, a0, a1, cw, 0.0)
    }
    pub fn arc(&mut self, x: f64, y: f64, r: f64, a0: f64, a1: f64, cw: bool) -> &mut Self {
        let (cx, cy) = (self.current.x, self.current.y);
        self.absarc(x + cx, y + cy, r, a0, a1, cw)
    }
    pub fn ellipse(&mut self, x: f64, y: f64, rx: f64, ry: f64, a0: f64, a1: f64, cw: bool, rot: f64) -> &mut Self {
        let (cx, cy) = (self.current.x, self.current.y);
        self.absellipse(x + cx, y + cy, rx, ry, a0, a1, cw, rot)
    }
    pub fn close_path(&mut self) -> &mut Self {
        if let (Some(first), Some(last)) = (self.curves.first().map(|c| c.point(0.0)), self.curves.last().map(|c| c.point(1.0)))
            && first != last
        {
            self.curves.push(Curve2::Line(last, first));
        }
        self
    }
    pub fn get_points(&self, divisions: u32) -> Vec<Vec2> {
        let mut pts: Vec<Vec2> = vec![];
        let mut last: Option<Vec2> = None;
        for c in &self.curves {
            let res = match c {
                Curve2::Ellipse { .. } => divisions * 2,
                Curve2::Line(..) => 1,
                _ => divisions,
            };
            for p in c.points(res) {
                if let Some(l) = last
                    && l == p
                {
                    continue;
                }
                pts.push(p);
                last = Some(p);
            }
        }
        if self.auto_close && pts.len() > 1 && pts[pts.len() - 1] != pts[0] {
            pts.push(pts[0]);
        }
        pts
    }
}

fn shape_area(c: &[Vec2]) -> f64 {
    let n = c.len();
    let mut a = 0.0;
    let mut p = n.wrapping_sub(1);
    let mut q = 0;
    while q < n {
        a += c[p].x * c[q].y - c[q].x * c[p].y;
        p = q;
        q += 1;
    }
    a * 0.5
}
fn is_clockwise(pts: &[Vec2]) -> bool {
    shape_area(pts) < 0.0
}
fn remove_dup_end_pts(pts: &mut Vec<Vec2>) {
    let l = pts.len();
    if l > 2 && pts[l - 1] == pts[0] {
        pts.pop();
    }
}

/// Flat triangulated shapes in XY facing +Z (one group per shape when several).
pub fn shape_geometry(shapes: &[Shape], curve_segments: f64) -> Geo {
    let cs = curve_segments as u32;
    let mut g = new_geo(GeoParams::Shape { curve_segments: cs });
    let mut idx = vec![];
    let mut group_start = 0u32;
    let multi = shapes.len() > 1;
    for (si, shape) in shapes.iter().enumerate() {
        let index_offset = (g.position.len() / 3) as u32;
        let mut contour = shape.get_points(cs);
        let mut holes: Vec<Vec<Vec2>> = shape.holes.iter().map(|h| h.get_points(cs)).collect();
        if !is_clockwise(&contour) {
            contour.reverse();
        }
        for h in holes.iter_mut() {
            if is_clockwise(h) {
                h.reverse();
            }
        }
        let faces = triangulate_shape(&contour, &holes);
        let mut all = contour.clone();
        remove_dup_end_pts(&mut all);
        for h in &holes {
            let mut h = h.clone();
            remove_dup_end_pts(&mut h);
            all.extend(h);
        }
        // vertices: the contour then the holes, each without a repeated end point
        // (the order the triangulation indexes)
        for p in &all {
            g.position.extend_from_slice(&[p.x as f32, p.y as f32, 0.0]);
            g.normal.extend_from_slice(&[0.0, 0.0, 1.0]);
            g.uv.extend_from_slice(&[p.x as f32, p.y as f32]);
        }
        let mut gc = 0;
        for f in faces {
            idx.extend_from_slice(&[f[0] + index_offset, f[1] + index_offset, f[2] + index_offset]);
            gc += 3;
        }
        if multi {
            g.add_group(group_start, gc, si as u32);
            group_start += gc;
        }
    }
    g.index = Some(idx);
    Geo::from_geometry(g)
}

fn triangulate_shape(contour: &[Vec2], holes: &[Vec<Vec2>]) -> Vec<[u32; 3]> {
    let mut contour = contour.to_vec();
    remove_dup_end_pts(&mut contour);
    let mut verts: Vec<f64> = vec![];
    let mut hole_idx: Vec<usize> = vec![];
    for p in &contour {
        verts.push(p.x);
        verts.push(p.y);
    }
    let mut start = contour.len();
    for h in holes {
        let mut h = h.clone();
        remove_dup_end_pts(&mut h);
        hole_idx.push(start);
        start += h.len();
        for p in &h {
            verts.push(p.x);
            verts.push(p.y);
        }
    }
    let tris = crate::earcut::earcut(&verts, &hole_idx, 2);
    tris.as_chunks::<3>().0.iter().map(|t| [t[0] as u32, t[1] as u32, t[2] as u32]).collect()
}

// ===================================================================== constructor macros (trailing arguments optional)
/// `box_geo!(w=1, h=1, d=1, ws=1, hs=1, ds=1)`
#[macro_export]
macro_rules! box_geo {
    () => {
        $crate::geometry::box_geometry(1.0, 1.0, 1.0, 1.0, 1.0, 1.0)
    };
    ($w:expr_2021) => {
        $crate::geometry::box_geometry(($w) as f64, 1.0, 1.0, 1.0, 1.0, 1.0)
    };
    ($w:expr_2021, $h:expr_2021) => {
        $crate::geometry::box_geometry(($w) as f64, ($h) as f64, 1.0, 1.0, 1.0, 1.0)
    };
    ($w:expr_2021, $h:expr_2021, $d:expr_2021) => {
        $crate::geometry::box_geometry(($w) as f64, ($h) as f64, ($d) as f64, 1.0, 1.0, 1.0)
    };
    ($w:expr_2021, $h:expr_2021, $d:expr_2021, $a:expr_2021) => {
        $crate::geometry::box_geometry(($w) as f64, ($h) as f64, ($d) as f64, ($a) as f64, 1.0, 1.0)
    };
    ($w:expr_2021, $h:expr_2021, $d:expr_2021, $a:expr_2021, $b:expr_2021) => {
        $crate::geometry::box_geometry(($w) as f64, ($h) as f64, ($d) as f64, ($a) as f64, ($b) as f64, 1.0)
    };
    ($w:expr_2021, $h:expr_2021, $d:expr_2021, $a:expr_2021, $b:expr_2021, $c:expr_2021) => {
        $crate::geometry::box_geometry(($w) as f64, ($h) as f64, ($d) as f64, ($a) as f64, ($b) as f64, ($c) as f64)
    };
}
/// `cyl_geo!(rt=1, rb=1, h=1, radial=32, hs=1, open=false, ts=0, tl=2π)`
#[macro_export]
macro_rules! cyl_geo {
    () => {
        $crate::cyl_geo!(1.0, 1.0, 1.0)
    };
    ($a:expr_2021) => {
        $crate::cyl_geo!($a, 1.0, 1.0)
    };
    ($a:expr_2021, $b:expr_2021) => {
        $crate::cyl_geo!($a, $b, 1.0)
    };
    ($a:expr_2021, $b:expr_2021, $h:expr_2021) => {
        $crate::cyl_geo!($a, $b, $h, 32)
    };
    ($a:expr_2021, $b:expr_2021, $h:expr_2021, $r:expr_2021) => {
        $crate::cyl_geo!($a, $b, $h, $r, 1)
    };
    ($a:expr_2021, $b:expr_2021, $h:expr_2021, $r:expr_2021, $hs:expr_2021) => {
        $crate::cyl_geo!($a, $b, $h, $r, $hs, false)
    };
    ($a:expr_2021, $b:expr_2021, $h:expr_2021, $r:expr_2021, $hs:expr_2021, $o:expr_2021) => {
        $crate::cyl_geo!($a, $b, $h, $r, $hs, $o, 0.0)
    };
    ($a:expr_2021, $b:expr_2021, $h:expr_2021, $r:expr_2021, $hs:expr_2021, $o:expr_2021, $ts:expr_2021) => {
        $crate::cyl_geo!($a, $b, $h, $r, $hs, $o, $ts, std::f64::consts::TAU)
    };
    ($a:expr_2021, $b:expr_2021, $h:expr_2021, $r:expr_2021, $hs:expr_2021, $o:expr_2021, $ts:expr_2021, $tl:expr_2021) => {
        $crate::geometry::cylinder_geometry(
            ($a) as f64,
            ($b) as f64,
            ($h) as f64,
            ($r) as f64,
            ($hs) as f64,
            $o,
            ($ts) as f64,
            ($tl) as f64,
        )
    };
}
/// `cone_geo!(r=1, h=1, radial=32, hs=1, open=false, ts=0, tl=2π)`
#[macro_export]
macro_rules! cone_geo {
    () => {
        $crate::cone_geo!(1.0, 1.0)
    };
    ($a:expr_2021) => {
        $crate::cone_geo!($a, 1.0)
    };
    ($a:expr_2021, $h:expr_2021) => {
        $crate::cone_geo!($a, $h, 32)
    };
    ($a:expr_2021, $h:expr_2021, $r:expr_2021) => {
        $crate::cone_geo!($a, $h, $r, 1)
    };
    ($a:expr_2021, $h:expr_2021, $r:expr_2021, $hs:expr_2021) => {
        $crate::cone_geo!($a, $h, $r, $hs, false)
    };
    ($a:expr_2021, $h:expr_2021, $r:expr_2021, $hs:expr_2021, $o:expr_2021) => {
        $crate::cone_geo!($a, $h, $r, $hs, $o, 0.0)
    };
    ($a:expr_2021, $h:expr_2021, $r:expr_2021, $hs:expr_2021, $o:expr_2021, $ts:expr_2021) => {
        $crate::cone_geo!($a, $h, $r, $hs, $o, $ts, std::f64::consts::TAU)
    };
    ($a:expr_2021, $h:expr_2021, $r:expr_2021, $hs:expr_2021, $o:expr_2021, $ts:expr_2021, $tl:expr_2021) => {
        $crate::geometry::cone_geometry(($a) as f64, ($h) as f64, ($r) as f64, ($hs) as f64, $o, ($ts) as f64, ($tl) as f64)
    };
}
/// `sphere_geo!(r=1, ws=32, hs=16, phi_start=0, phi_length=2π, theta_start=0, theta_length=π)`
#[macro_export]
macro_rules! sphere_geo {
    () => {
        $crate::sphere_geo!(1.0)
    };
    ($r:expr_2021) => {
        $crate::sphere_geo!($r, 32)
    };
    ($r:expr_2021, $w:expr_2021) => {
        $crate::sphere_geo!($r, $w, 16)
    };
    ($r:expr_2021, $w:expr_2021, $h:expr_2021) => {
        $crate::sphere_geo!($r, $w, $h, 0.0)
    };
    ($r:expr_2021, $w:expr_2021, $h:expr_2021, $ps:expr_2021) => {
        $crate::sphere_geo!($r, $w, $h, $ps, std::f64::consts::TAU)
    };
    ($r:expr_2021, $w:expr_2021, $h:expr_2021, $ps:expr_2021, $pl:expr_2021) => {
        $crate::sphere_geo!($r, $w, $h, $ps, $pl, 0.0)
    };
    ($r:expr_2021, $w:expr_2021, $h:expr_2021, $ps:expr_2021, $pl:expr_2021, $ts:expr_2021) => {
        $crate::sphere_geo!($r, $w, $h, $ps, $pl, $ts, std::f64::consts::PI)
    };
    ($r:expr_2021, $w:expr_2021, $h:expr_2021, $ps:expr_2021, $pl:expr_2021, $ts:expr_2021, $tl:expr_2021) => {
        $crate::geometry::sphere_geometry(($r) as f64, ($w) as f64, ($h) as f64, ($ps) as f64, ($pl) as f64, ($ts) as f64, ($tl) as f64)
    };
}
/// `plane_geo!(w=1, h=1, ws=1, hs=1)`
#[macro_export]
macro_rules! plane_geo {
    () => {
        $crate::plane_geo!(1.0, 1.0)
    };
    ($w:expr_2021) => {
        $crate::plane_geo!($w, 1.0)
    };
    ($w:expr_2021, $h:expr_2021) => {
        $crate::plane_geo!($w, $h, 1, 1)
    };
    ($w:expr_2021, $h:expr_2021, $a:expr_2021) => {
        $crate::plane_geo!($w, $h, $a, 1)
    };
    ($w:expr_2021, $h:expr_2021, $a:expr_2021, $b:expr_2021) => {
        $crate::geometry::plane_geometry(($w) as f64, ($h) as f64, ($a) as f64, ($b) as f64)
    };
}
/// `circle_geo!(r=1, segments=32, theta_start=0, theta_length=2π)`
#[macro_export]
macro_rules! circle_geo {
    () => {
        $crate::circle_geo!(1.0)
    };
    ($r:expr_2021) => {
        $crate::circle_geo!($r, 32)
    };
    ($r:expr_2021, $s:expr_2021) => {
        $crate::circle_geo!($r, $s, 0.0)
    };
    ($r:expr_2021, $s:expr_2021, $ts:expr_2021) => {
        $crate::circle_geo!($r, $s, $ts, std::f64::consts::TAU)
    };
    ($r:expr_2021, $s:expr_2021, $ts:expr_2021, $tl:expr_2021) => {
        $crate::geometry::circle_geometry(($r) as f64, ($s) as f64, ($ts) as f64, ($tl) as f64)
    };
}
/// `torus_geo!(r=1, tube=0.4, radial=12, tubular=48, arc=2π)`
#[macro_export]
macro_rules! torus_geo {
    () => {
        $crate::torus_geo!(1.0)
    };
    ($r:expr_2021) => {
        $crate::torus_geo!($r, 0.4)
    };
    ($r:expr_2021, $t:expr_2021) => {
        $crate::torus_geo!($r, $t, 12)
    };
    ($r:expr_2021, $t:expr_2021, $rs:expr_2021) => {
        $crate::torus_geo!($r, $t, $rs, 48)
    };
    ($r:expr_2021, $t:expr_2021, $rs:expr_2021, $ts:expr_2021) => {
        $crate::torus_geo!($r, $t, $rs, $ts, std::f64::consts::TAU)
    };
    ($r:expr_2021, $t:expr_2021, $rs:expr_2021, $ts:expr_2021, $a:expr_2021) => {
        $crate::geometry::torus_geometry(($r) as f64, ($t) as f64, ($rs) as f64, ($ts) as f64, ($a) as f64)
    };
}
/// `capsule_geo!(r=1, length=1, cap_segments=4, radial_segments=8)`
#[macro_export]
macro_rules! capsule_geo {
    () => {
        $crate::capsule_geo!(1.0)
    };
    ($r:expr_2021) => {
        $crate::capsule_geo!($r, 1.0)
    };
    ($r:expr_2021, $l:expr_2021) => {
        $crate::capsule_geo!($r, $l, 4)
    };
    ($r:expr_2021, $l:expr_2021, $c:expr_2021) => {
        $crate::capsule_geo!($r, $l, $c, 8)
    };
    ($r:expr_2021, $l:expr_2021, $c:expr_2021, $s:expr_2021) => {
        $crate::geometry::capsule_geometry(($r) as f64, ($l) as f64, ($c) as f64, ($s) as f64)
    };
}
/// `octa_geo!(r=1, detail=0)`
#[macro_export]
macro_rules! octa_geo {
    () => {
        $crate::octa_geo!(1.0)
    };
    ($r:expr_2021) => {
        $crate::octa_geo!($r, 0)
    };
    ($r:expr_2021, $d:expr_2021) => {
        $crate::geometry::octahedron_geometry(($r) as f64, ($d) as f64)
    };
}
/// `icosa_geo!(r=1, detail=0)`
#[macro_export]
macro_rules! icosa_geo {
    () => {
        $crate::icosa_geo!(1.0)
    };
    ($r:expr_2021) => {
        $crate::icosa_geo!($r, 0)
    };
    ($r:expr_2021, $d:expr_2021) => {
        $crate::geometry::icosahedron_geometry(($r) as f64, ($d) as f64)
    };
}
/// `dodeca_geo!(r=1, detail=0)`
#[macro_export]
macro_rules! dodeca_geo {
    () => {
        $crate::dodeca_geo!(1.0)
    };
    ($r:expr_2021) => {
        $crate::dodeca_geo!($r, 0)
    };
    ($r:expr_2021, $d:expr_2021) => {
        $crate::geometry::dodecahedron_geometry(($r) as f64, ($d) as f64)
    };
}
/// `tetra_geo!(r=1, detail=0)`
#[macro_export]
macro_rules! tetra_geo {
    () => {
        $crate::tetra_geo!(1.0)
    };
    ($r:expr_2021) => {
        $crate::tetra_geo!($r, 0)
    };
    ($r:expr_2021, $d:expr_2021) => {
        $crate::geometry::tetrahedron_geometry(($r) as f64, ($d) as f64)
    };
}
/// `ring_geo!(inner=0.5, outer=1, theta_segs=32, phi_segs=1, theta_start=0, theta_length=2π)`
#[macro_export]
macro_rules! ring_geo {
    () => {
        $crate::ring_geo!(0.5, 1.0)
    };
    ($i:expr_2021) => {
        $crate::ring_geo!($i, 1.0)
    };
    ($i:expr_2021, $o:expr_2021) => {
        $crate::ring_geo!($i, $o, 32)
    };
    ($i:expr_2021, $o:expr_2021, $t:expr_2021) => {
        $crate::ring_geo!($i, $o, $t, 1)
    };
    ($i:expr_2021, $o:expr_2021, $t:expr_2021, $p:expr_2021) => {
        $crate::ring_geo!($i, $o, $t, $p, 0.0)
    };
    ($i:expr_2021, $o:expr_2021, $t:expr_2021, $p:expr_2021, $ts:expr_2021) => {
        $crate::ring_geo!($i, $o, $t, $p, $ts, std::f64::consts::TAU)
    };
    ($i:expr_2021, $o:expr_2021, $t:expr_2021, $p:expr_2021, $ts:expr_2021, $tl:expr_2021) => {
        $crate::geometry::ring_geometry(($i) as f64, ($o) as f64, ($t) as f64, ($p) as f64, ($ts) as f64, ($tl) as f64)
    };
}
/// `lathe_geo!(points, segments=12, phi_start=0, phi_length=2π)`
#[macro_export]
macro_rules! lathe_geo {
    ($p:expr_2021) => {
        $crate::lathe_geo!($p, 12)
    };
    ($p:expr_2021, $s:expr_2021) => {
        $crate::lathe_geo!($p, $s, 0.0)
    };
    ($p:expr_2021, $s:expr_2021, $ps:expr_2021) => {
        $crate::lathe_geo!($p, $s, $ps, std::f64::consts::TAU)
    };
    ($p:expr_2021, $s:expr_2021, $ps:expr_2021, $pl:expr_2021) => {
        $crate::geometry::lathe_geometry(&$p, ($s) as f64, ($ps) as f64, ($pl) as f64)
    };
}
/// `shape_geo!(shape, curve_segments=12)`
#[macro_export]
macro_rules! shape_geo {
    ($s:expr_2021) => {
        $crate::shape_geo!($s, 12)
    };
    ($s:expr_2021, $c:expr_2021) => {
        $crate::geometry::shape_geometry(std::slice::from_ref(&$s), ($c) as f64)
    };
}
