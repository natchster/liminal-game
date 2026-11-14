//! Creatures: decodes the organic parts modelled in Blender (embedded as
//! `game/assets/creature_meshes.bin`) into engine geometry. Positions arrive
//! as int16 quantized to each part's bounding box;
//! normals are rebuilt smooth, and UVs are a cylindrical projection around Y
//! (enough for a mottled skin texture). Parts that were painted in Blender
//! carry sRGB vertex colours (albedo), decoded to linear into `color`; their
//! materials use vertex colours and white. Skinned bodies also carry four
//! bone indices and weights per vertex and the names of their bones
//! (`bone_names`).
//!
//! Also here: [`with_cavity_shade`] and the Watcher's morph-target sheet
//! ([`watcher_geometry`]), which is built from these parts.
use crate::assets::props::{Rd, inflate_raw, or_num};
use crate::base::numeric::hypot;
use engine::prelude::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::OnceLock;

/// Every creature part, as one embedded asset file.
static CREATURE_BIN: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/creature_meshes.bin"));

/// One creature part's metadata (payloads still compressed when `packed`).
#[derive(Clone, Debug)]
pub struct CreaturePartInfo {
    pub name: &'static str,
    /// Every buffer raw-deflated, positions delta-coded per axis
    pub packed: bool,
    /// Carries sRGB vertex colours
    pub has_color: bool,
    /// Carries four bone slots per vertex
    pub skinned: bool,
    /// Vertices (`n`) and triangles (`t`)
    pub n: usize,
    pub t: usize,
    pub min: [f64; 3],
    pub max: [f64; 3],
    /// Bone names
    pub bones: Vec<&'static str>,
    v: &'static [u8],
    i: &'static [u8],
    c: &'static [u8],
    j: &'static [u8],
    w: &'static [u8],
}

struct CreatureIndex {
    parts: Vec<CreaturePartInfo>,
    by_name: HashMap<&'static str, usize>,
    names: Vec<&'static str>,
}

fn index() -> &'static CreatureIndex {
    static IDX: OnceLock<CreatureIndex> = OnceLock::new();
    IDX.get_or_init(|| {
        let mut rd = Rd::new(CREATURE_BIN);
        assert_eq!(rd.bytes(8), b"CREAMSH1", "creature_meshes.bin: bad magic (corrupt or outdated asset file)");
        let count = rd.u32() as usize;
        let mut parts = Vec::with_capacity(count);
        for _ in 0..count {
            let name = rd.str();
            let flags = rd.u8();
            let (n, t) = (rd.u32() as usize, rd.u32() as usize);
            let (min, max) = (rd.f64x3(), rd.f64x3());
            let nb = rd.u16() as usize;
            let bones = (0..nb).map(|_| rd.str()).collect();
            let (v, i, c, j, w) = (rd.blob(), rd.blob(), rd.blob(), rd.blob(), rd.blob());
            parts.push(CreaturePartInfo {
                name,
                packed: flags & 1 != 0,
                has_color: flags & 2 != 0,
                skinned: flags & 4 != 0,
                n,
                t,
                min,
                max,
                bones,
                v,
                i,
                c,
                j,
                w,
            });
        }
        let names: Vec<&'static str> = parts.iter().map(|p| p.name).collect();
        let by_name = names.iter().enumerate().map(|(i, n)| (*n, i)).collect();
        CreatureIndex { parts, by_name, names }
    })
}

/// Every creature part name, in file order.
pub fn creature_part_names() -> &'static [&'static str] {
    &index().names
}
/// The metadata of part `name`.
pub fn creature_part_info(name: &str) -> Option<&'static CreaturePartInfo> {
    index().by_name.get(name).map(|&i| &index().parts[i])
}

/// sRGB byte → linear lookup table (built once).
fn srgb_table() -> &'static [f32; 256] {
    static T: OnceLock<[f32; 256]> = OnceLock::new();
    T.get_or_init(|| {
        let mut t = [0f32; 256];
        for (i, v) in t.iter_mut().enumerate() {
            let c = i as f64 / 255.0;
            *v = (if c <= 0.04045 { c / 12.92 } else { ((c + 0.055) / 1.055).dpowf(2.4) }) as f32;
        }
        t
    })
}

/// Buffer `k` of part `m`, unpacked to `bytes` bytes: a packed part has every buffer
/// raw-deflated, and its positions stored per axis as differences from the
/// previous vertex.
fn field(m: &CreaturePartInfo, k: char, bytes: usize) -> Vec<u8> {
    let src = match k {
        'v' => m.v,
        'i' => m.i,
        'c' => m.c,
        'j' => m.j,
        _ => m.w,
    };
    if !m.packed {
        let mut v = src.to_vec();
        if v.len() < bytes {
            v.resize(bytes, 0);
        }
        return v;
    }
    if bytes == 0 {
        return vec![];
    }
    let out = inflate_raw(src, bytes);
    if k != 'v' {
        return out;
    }
    let n = m.n;
    let d = |i: usize| i16::from_le_bytes([out[i * 2], out[i * 2 + 1]]);
    let mut q = vec![0i16; n * 3];
    for a in 0..3 {
        let mut acc: i16 = 0;
        for i in 0..n {
            acc = acc.wrapping_add(d(a * n + i));
            q[i * 3 + a] = acc;
        }
    }
    q.iter().flat_map(|x| x.to_le_bytes()).collect()
}

thread_local! {
    static CACHE: RefCell<HashMap<String, Option<Geo>>> = RefCell::new(HashMap::new());
}

/// The decoded part (cached and shared), or
/// `None` when there is no such part.
pub fn creature_geometry(name: &str) -> Option<Geo> {
    if let Some(g) = CACHE.with(|c| c.borrow().get(name).cloned()) {
        return g;
    }
    let m = creature_part_info(name)?;
    let g = Geo::from_geometry(decode_creature(m));
    CACHE.with(|c| c.borrow_mut().insert(name.to_string(), Some(g.clone())));
    Some(g)
}

/// Decode one part into a fresh geometry (what [`creature_geometry`] caches).
pub fn decode_creature(m: &CreaturePartInfo) -> Geometry {
    let n = m.n;
    let vb = field(m, 'v', n * 6);
    let ib = field(m, 'i', m.t * 6);
    let q = |i: usize| -> f64 { if i * 2 + 1 < vb.len() { i16::from_le_bytes([vb[i * 2], vb[i * 2 + 1]]) as f64 } else { 0.0 } };
    let mut pos = vec![0f32; n * 3];
    let mut uv = vec![0f32; n * 2];
    let span: [f64; 3] = [or_num(m.max[0] - m.min[0], 1.0), or_num(m.max[1] - m.min[1], 1.0), or_num(m.max[2] - m.min[2], 1.0)];
    let h = span[1];
    let tau = std::f64::consts::PI * 2.0;
    for i in 0..n {
        for k in 0..3 {
            pos[i * 3 + k] = (m.min[k] + ((q(i * 3 + k) + 32767.0) / 65534.0) * span[k]) as f32;
        }
        uv[i * 2] = ((pos[i * 3] as f64).datan2(pos[i * 3 + 2] as f64) / tau + 0.5) as f32;
        uv[i * 2 + 1] = ((pos[i * 3 + 1] as f64 - m.min[1]) / h) as f32;
    }
    let mut g = Geometry { position: pos, uv, ..Default::default() };
    if m.has_color {
        let c8 = field(m, 'c', n * 3);
        let t = srgb_table();
        g.color = (0..n * 3).map(|i| t[c8.get(i).copied().unwrap_or(0) as usize]).collect();
        g.color_size = 3;
    }
    // skinned parts: four bone slots per vertex (indices into `bones`, the bone
    // names) and their weights, which sum to 255
    if m.skinned {
        let j8 = field(m, 'j', n * 4);
        let w8 = field(m, 'w', n * 4);
        g.skin_index = (0..n * 4).map(|i| j8.get(i).copied().unwrap_or(0) as f32).collect();
        g.skin_weight = (0..n * 4).map(|i| (w8.get(i).copied().unwrap_or(0) as f64 / 255.0) as f32).collect();
        g.bone_names = m.bones.iter().map(|s| s.to_string()).collect();
    }
    let tt = m.t * 3;
    g.index = Some((0..tt).map(|i| if i * 2 + 1 < ib.len() { u16::from_le_bytes([ib[i * 2], ib[i * 2 + 1]]) as u32 } else { 0 }).collect());
    g.compute_vertex_normals();
    g.compute_bounding_box();
    g.compute_bounding_sphere();
    g
}

/// Cheap cavity shading as vertex colours (`strength` is usually 0.75;
/// `center` defaults to the bounding sphere's): a vertex whose normal turns away from
/// the direction out of the centre sits in a hollow (an eye socket, the gap
/// under a brow) and is darkened. Multiplies any painted colours. Returns a
/// new geometry (a duplicate of `geo`) with its own `color`.
pub fn with_cavity_shade(geo: &Geo, strength: f64, center: Option<Vec3>) -> Geo {
    let g = geo.duplicate();
    let c = match center {
        Some(c) => c,
        None => g.bounding_sphere().center,
    };
    let base: Option<Vec<f32>> = {
        let s = geo.r();
        if s.color.is_empty() { None } else { Some(s.color.clone()) }
    };
    let col = {
        let gg = g.r();
        let count = gg.vertex_count();
        let (pos, nrm) = (&gg.position, &gg.normal);
        let nr = |i: usize| nrm.get(i).copied().unwrap_or(0.0) as f64;
        let mut col = vec![0f32; count * 3];
        for i in 0..count {
            let dx = pos[i * 3] as f64 - c.x;
            let dy = pos[i * 3 + 1] as f64 - c.y;
            let dz = pos[i * 3 + 2] as f64 - c.z;
            let l = or_num(hypot(&[dx, dy, dz]), 1.0);
            let facing = (nr(i * 3) * dx + nr(i * 3 + 1) * dy + nr(i * 3 + 2) * dz) / l; // 1 on a smooth dome
            let t = crate::base::numeric::nan_min(1.0, crate::base::numeric::nan_max(0.0, (0.9 - facing) / 0.9));
            let k = 1.0 - strength * t.dpowf(1.4);
            for j in 0..3 {
                let b = match &base {
                    Some(b) => b.get(i * 3 + j).copied().unwrap_or(f32::NAN) as f64,
                    None => 1.0,
                };
                col[i * 3 + j] = (k * b) as f32;
            }
        }
        col
    };
    g.set_color(col, 3);
    g
}

/// The Watcher's sheet. `body` is the shared `wtBody` part geometry (or the
/// cylinder stand-in), `morphs` the number of morph targets. The parts
/// `wtPose1`..`wtPose4` (the further poses) and `wtBow` (its head bowed
/// under a low ceiling) are the same vertices moved, so they become relative
/// morph targets on `body` (`morph_position`, `morph_normal`,
/// `morph_relative = true`) and the colour deltas (their baked shadow moves
/// with the folds) are in `morph_color` — the engine's Geometry has no
/// colour morph attribute, so the renderer side has to apply those itself.
#[derive(Clone, Debug)]
pub struct WatcherGeo {
    pub body: Geo,
    pub morphs: usize,
    pub morph_color: Vec<Vec<f32>>,
}

pub const WATCHER_POSES: [&str; 5] = ["wtPose1", "wtPose2", "wtPose3", "wtPose4", "wtBow"];

thread_local! {
    static WATCHER: std::cell::OnceCell<WatcherGeo> = const { std::cell::OnceCell::new() };
}

/// The Watcher's sheet (built once; it adds the morph targets to the cached
/// `wtBody` part).
pub fn watcher_geometry() -> WatcherGeo {
    WATCHER.with(|c| c.get_or_init(build_watcher).clone())
}

fn build_watcher() -> WatcherGeo {
    let Some(body) = creature_geometry("wtBody") else {
        let g = cylinder_geometry(0.16, 0.36, 2.06, 16.0, 1.0, false, 0.0, std::f64::consts::TAU);
        g.translate(0.0, 1.03, 0.0);
        return WatcherGeo { body: g, morphs: 0, morph_color: vec![] };
    };
    let (n, bp, bn, bc, index) = {
        let b = body.r();
        (
            b.vertex_count(),
            b.position.clone(),
            b.normal.clone(),
            if b.color.is_empty() { None } else { Some(b.color.clone()) },
            b.index.clone(),
        )
    };
    let mut mp = vec![];
    let mut mn = vec![];
    let mut mc = vec![];
    for name in WATCHER_POSES {
        let Some(g) = creature_geometry(name) else { return WatcherGeo { body, morphs: 0, morph_color: vec![] } };
        if g.vertex_count() != n {
            return WatcherGeo { body, morphs: 0, morph_color: vec![] };
        }
        let (p, c) = {
            let r = g.r();
            (r.position.clone(), if r.color.is_empty() { None } else { Some(r.color.clone()) })
        };
        let mut tmp = Geometry { position: p.clone(), index: index.clone(), ..Default::default() };
        tmp.compute_vertex_normals();
        let nn = &tmp.normal;
        let mut dp = vec![0f32; n * 3];
        let mut dn = vec![0f32; n * 3];
        let mut dc = vec![0f32; n * 3];
        for i in 0..n * 3 {
            dp[i] = (p[i] as f64 - bp[i] as f64) as f32;
            dn[i] = (nn[i] as f64 - bn[i] as f64) as f32;
            if let (Some(c), Some(bc)) = (&c, &bc) {
                dc[i] = (c[i] as f64 - bc[i] as f64) as f32;
            }
        }
        mp.push(dp);
        mn.push(dn);
        mc.push(dc);
    }
    let morphs = mp.len();
    {
        let mut b = body.g();
        b.morph_position = mp;
        b.morph_normal = mn;
        b.morph_relative = true;
        b.version += 1;
        // a bounding sphere that covers every pose
        let s = bounding_sphere_with_morphs(&b);
        b.bounding_sphere = Some(s);
    }
    WatcherGeo { body, morphs, morph_color: if bc.is_some() { mc } else { vec![] } }
}

/// Bounding sphere of a geometry including its morph targets (the
/// engine's own only looks at `position`): centred on the box of every pose,
/// with the radius of the farthest vertex in any pose.
pub fn bounding_sphere_with_morphs(g: &Geometry) -> Sphere {
    if g.position.is_empty() {
        return Sphere { center: Vec3::ZERO, radius: 0.0 };
    }
    let bbox = |a: &[f32]| {
        let mut b = Box3::empty();
        for p in a.as_chunks::<3>().0 {
            b.expand_by_point(&Vec3::new(p[0] as f64, p[1] as f64, p[2] as f64));
        }
        b
    };
    let mut b = bbox(&g.position);
    for m in &g.morph_position {
        let bm = bbox(m);
        if g.morph_relative {
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
    let center = b.get_center();
    let mut max_r2: f64 = 0.0;
    for p in g.position.as_chunks::<3>().0 {
        max_r2 = max_r2.max(center.distance_to_squared(&Vec3::new(p[0] as f64, p[1] as f64, p[2] as f64)));
    }
    for m in &g.morph_position {
        for (j, q) in m.as_chunks::<3>().0.iter().enumerate() {
            let mut v = Vec3::new(q[0] as f64, q[1] as f64, q[2] as f64);
            if g.morph_relative {
                v.add(&Vec3::new(g.position[j * 3] as f64, g.position[j * 3 + 1] as f64, g.position[j * 3 + 2] as f64));
            }
            max_r2 = max_r2.max(center.distance_to_squared(&v));
        }
    }
    Sphere { center, radius: max_r2.sqrt() }
}
