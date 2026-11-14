//! Building a thing: triangles and buffers to geometry, the Blender-made parts and their frames, a part put on a group.
use game::assets::props::{prop_part_as_modelled, prop_part_info};
use game::prelude::*;

/// A custom geometry: f32 positions + `set_index(idx)` + `compute_vertex_normals()`
pub fn buf_geo(verts: &[f64], idx: &[u32]) -> Geo {
    let geo = Geo::new();
    geo.set_position(verts.iter().map(|&v| v as f32).collect());
    geo.set_index(idx.to_vec());
    geo.compute_vertex_normals();
    geo
}

/// A deep clone whose Euler angles are re-derived from each node's quaternion
/// (a yaw past π/2 comes back as (±π, π - θ, ±π)), so later
/// `clone.rotation().y = v` writes act on those angles (kept on purpose).
/// `Obj::duplicate` keeps the Euler as it was, hence this wrapper.
pub fn clone_obj(o: &Obj) -> Obj {
    let c = o.duplicate();
    c.traverse(&mut |n| {
        let q = n.quat();
        *n.quaternion() = q;
    });
    c
}

/// The bounds (frame) part `name` was modelled in
pub fn frame(name: &str) -> ([f64; 3], [f64; 3]) {
    prop_part_info(name).map(|m| (m.fmin, m.fmax)).unwrap_or(([-0.05; 3], [0.05; 3]))
}

/// The four bars of a picture frame, each `(bar_w, bar_h, bx, by)`.
pub fn frame_bars(w: f64, h: f64, fw: f64) -> [(f64, f64, f64, f64); 4] {
    [
        (w + fw * 2.0, fw, 0.0, h / 2.0 + fw / 2.0),
        (w + fw * 2.0, fw, 0.0, -h / 2.0 - fw / 2.0),
        (fw, h, -w / 2.0 - fw / 2.0, 0.0),
        (fw, h, w / 2.0 + fw / 2.0, 0.0),
    ]
}

/// Oriented cylinder segment between two points (`mid`, `set_from_unit_vectors`).
pub fn orient_seg(seg: &Obj, prev: &Vec3, next: &Vec3) {
    let mut mid = *prev;
    mid.add(next).multiply_scalar(0.5);
    seg.position().copy(&mid);
    let mut v = *next;
    v.sub(prev).normalize();
    seg.quaternion().set_from_unit_vectors(&v3(0, 1, 0), &v);
}

/// The Blender part `name` as modelled (its own frame, a scale of one)
pub fn part(name: &str) -> Option<Geo> {
    prop_part_as_modelled(name)
}

/// The Blender part `name`, as modelled, in `grp` with `mat`
pub fn put(grp: &Obj, name: &str, mat: &Mat) -> Option<Obj> {
    let m = mesh(prop_part_as_modelled(name)?, mat);
    grp.add(&m);
    Some(m)
}

/// The same, its texture coordinates laid from above at its real scale
/// (u along x, v against z, `half` m from its middle to the map's edge)
pub fn put_top(grp: &Obj, name: &str, mat: &Mat, half: f64) -> Option<Obj> {
    let geo = prop_part_as_modelled(name)?.duplicate();
    {
        let mut gg = geo.g();
        let n = gg.position.len() / 3;
        for i in 0..n {
            let (x, z) = (f64::from(gg.position[i * 3]), f64::from(gg.position[i * 3 + 2]));
            gg.uv[i * 2] = ((x / half + 1.0) * 0.5) as f32;
            gg.uv[i * 2 + 1] = (1.0 - (z / half + 1.0) * 0.5) as f32;
        }
        gg.touch();
    }
    let m = mesh(geo, mat.clone());
    grp.add(&m);
    Some(m)
}

/// Part `name` with its texture laid on at its real scale (`size` m a
/// repeat), from the side each vertex faces most: a worn surface the same
/// grain on every part of the thing
pub fn put_world(grp: &Obj, name: &str, mat: &Mat, size: f64) {
    let Some(geo) = prop_part_as_modelled(name) else { return };
    let geo = geo.duplicate();
    {
        let mut gg = geo.g();
        let n = gg.position.len() / 3;
        let mut uv = Vec::with_capacity(n * 2);
        for i in 0..n {
            let (x, y, z) = (f64::from(gg.position[i * 3]), f64::from(gg.position[i * 3 + 1]), f64::from(gg.position[i * 3 + 2]));
            let (ax, ay, az) = (gg.normal[i * 3].abs(), gg.normal[i * 3 + 1].abs(), gg.normal[i * 3 + 2].abs());
            let (u, v) = if ay >= ax && ay >= az {
                (x, z)
            } else if ax >= az {
                (z, y)
            } else {
                (x, y)
            };
            uv.push((u / size) as f32);
            uv.push((v / size) as f32);
        }
        gg.uv = uv;
        gg.touch();
    }
    grp.add(&mesh(geo, mat.clone()));
}

/// `o.rotation.set(x, y, z, order)`
pub fn set_rot_order(o: &Obj, x: f64, y: f64, z: f64, order: EulerOrder) {
    let mut r = o.rotation();
    r.set(x, y, z);
    r.order = order;
}

/// A single-triangle geometry with computed normals.
pub fn tri_geo(v: [f64; 9]) -> Geo {
    let geo = Geo::new();
    geo.set_position(v.iter().map(|&a| a as f32).collect());
    geo.set_index(vec![0, 1, 2]);
    geo.compute_vertex_normals();
    geo
}
