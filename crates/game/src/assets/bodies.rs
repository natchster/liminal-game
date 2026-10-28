//! The creatures' bodies (`game/assets/bodies.bin`): each one a skeleton and
//! the meshes bound to it, built by the Blender scripts in
//! `assets/src/creatures/` and packed by `cargo xtask assets creatures`.
//!
//! A **skeleton** is a list of bones, each with its parent (always earlier in
//! the list), its head and its tail in the body's own frame (metres, +Y up,
//! +Z the way it faces, its left at +X). Every bone's rest frame is axis
//! aligned: its origin at its head, no rotation, so a body is bound in the
//! pose it was modelled in and an animation turns a bone by the rotation
//! that takes its rest direction (head to tail) to where it should point.
//!
//! A **mesh** is either skinned (four bone slots and weights per vertex,
//! indices into the body's bones) or rigid on one bone (positions in the
//! body's frame, moved into the bone's frame when it is built), and names a
//! material slot (`skin`, `teeth`, `eye`, `cloth`, ...) the game fills.
//! Meshes may carry sRGB vertex colours, UVs and morph targets (offsets).
//!
//! The file: `BODIES01`, a little-endian u32 header length, the header (JSON),
//! then the buffers, each raw-deflated; the header gives each buffer as
//! `[offset, packed length, length]` from the end of the header. Encodings:
//! positions int16 per axis over the mesh's box; normals int8 (/127);
//! colours u8 sRGB; UVs u16 (/65535); bone slots u8 and weights u8 (summing
//! to 255); indices u16, or u32 when the mesh has more than 65535 vertices;
//! a morph's offsets int16 over its own box and its normal offsets int8
//! (/63).
use crate::assets::props::inflate_raw;
use engine::prelude::*;
use serde_json::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::OnceLock;

static BODIES_BIN: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/bodies.bin"));

/// One bone of a skeleton (body frame).
#[derive(Clone, Debug, PartialEq)]
pub struct BoneDef {
    pub name: String,
    /// Index of the parent bone (always smaller), or `None` for a root
    pub parent: Option<usize>,
    pub head: [f64; 3],
    pub tail: [f64; 3],
}

/// One buffer in the file: where its packed bytes are and how long it is unpacked.
#[derive(Clone, Copy, Debug)]
struct Buf {
    off: usize,
    packed: usize,
    len: usize,
}

#[derive(Clone, Debug)]
pub struct MorphDef {
    pub name: String,
    min: [f64; 3],
    max: [f64; 3],
    pos: Buf,
    nrm: Option<Buf>,
}

/// One mesh of a body, still packed.
#[derive(Clone, Debug)]
pub struct MeshDef {
    pub name: String,
    /// The material slot the game fills (`skin`, `teeth`, `eye`, ...)
    pub material: String,
    /// `Some(bone)`: rigid on that bone; `None`: skinned
    pub bone: Option<usize>,
    /// Vertices and triangles
    pub n: usize,
    pub t: usize,
    min: [f64; 3],
    max: [f64; 3],
    pos: Buf,
    nrm: Buf,
    col: Option<Buf>,
    uv: Option<Buf>,
    jnt: Option<Buf>,
    wgt: Option<Buf>,
    idx: Buf,
    pub morphs: Vec<MorphDef>,
}

/// A body: its skeleton and meshes.
#[derive(Clone, Debug)]
pub struct BodyDef {
    pub name: String,
    pub bones: Vec<BoneDef>,
    pub meshes: Vec<MeshDef>,
    by_bone: HashMap<String, usize>,
}

impl BodyDef {
    /// The index of bone `name`.
    pub fn bone(&self, name: &str) -> Option<usize> {
        self.by_bone.get(name).copied()
    }
    /// Triangles over all meshes.
    pub fn triangles(&self) -> usize {
        self.meshes.iter().map(|m| m.t).sum()
    }
}

struct Index {
    bodies: Vec<BodyDef>,
    data: &'static [u8],
}

fn arr3(v: &Value) -> [f64; 3] {
    let a = |i: usize| v.get(i).and_then(Value::as_f64).unwrap_or(0.0);
    [a(0), a(1), a(2)]
}

fn buf(v: &Value) -> Option<Buf> {
    let a = v.as_array()?;
    let u = |i: usize| a.get(i).and_then(Value::as_u64).unwrap_or(0) as usize;
    Some(Buf { off: u(0), packed: u(1), len: u(2) })
}

fn parse(bytes: &'static [u8]) -> Index {
    assert!(bytes.len() >= 12 && &bytes[..8] == b"BODIES01", "bodies.bin: bad magic (corrupt or outdated asset file)");
    let hl = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]) as usize;
    let header: Value = serde_json::from_slice(&bytes[12..12 + hl]).expect("bodies.bin: header");
    let data = &bytes[12 + hl..];
    let mut bodies = vec![];
    for b in header["bodies"].as_array().into_iter().flatten() {
        let bones: Vec<BoneDef> = b["bones"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|x| BoneDef {
                name: x["name"].as_str().unwrap_or("").to_string(),
                parent: x["parent"].as_i64().filter(|p| *p >= 0).map(|p| p as usize),
                head: arr3(&x["head"]),
                tail: arr3(&x["tail"]),
            })
            .collect();
        let meshes = b["meshes"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|m| MeshDef {
                name: m["name"].as_str().unwrap_or("").to_string(),
                material: m["material"].as_str().unwrap_or("skin").to_string(),
                bone: m["bone"].as_i64().filter(|p| *p >= 0).map(|p| p as usize),
                n: m["n"].as_u64().unwrap_or(0) as usize,
                t: m["t"].as_u64().unwrap_or(0) as usize,
                min: arr3(&m["min"]),
                max: arr3(&m["max"]),
                pos: buf(&m["pos"]).expect("bodies.bin: a mesh without positions"),
                nrm: buf(&m["nrm"]).expect("bodies.bin: a mesh without normals"),
                col: buf(&m["col"]),
                uv: buf(&m["uv"]),
                jnt: buf(&m["jnt"]),
                wgt: buf(&m["wgt"]),
                idx: buf(&m["idx"]).expect("bodies.bin: a mesh without indices"),
                morphs: m["morphs"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .map(|k| MorphDef {
                        name: k["name"].as_str().unwrap_or("").to_string(),
                        min: arr3(&k["min"]),
                        max: arr3(&k["max"]),
                        pos: buf(&k["pos"]).expect("bodies.bin: a morph without offsets"),
                        nrm: buf(&k["nrm"]),
                    })
                    .collect(),
            })
            .collect();
        let by_bone = bones.iter().enumerate().map(|(i, b)| (b.name.clone(), i)).collect();
        bodies.push(BodyDef { name: b["name"].as_str().unwrap_or("").to_string(), bones, meshes, by_bone });
    }
    Index { bodies, data }
}

fn index() -> &'static Index {
    static IDX: OnceLock<Index> = OnceLock::new();
    IDX.get_or_init(|| parse(BODIES_BIN))
}

/// Every body in the file, in file order.
pub fn body_names() -> Vec<&'static str> {
    index().bodies.iter().map(|b| b.name.as_str()).collect()
}

/// Body `name`'s skeleton and (packed) meshes.
pub fn body(name: &str) -> Option<&'static BodyDef> {
    index().bodies.iter().find(|b| b.name == name)
}

fn unpack(b: Buf) -> Vec<u8> {
    let data = index().data;
    inflate_raw(&data[b.off..b.off + b.packed], b.len)
}

fn i16s(b: &[u8]) -> impl Iterator<Item = f64> + '_ {
    b.chunks_exact(2).map(|c| i16::from_le_bytes([c[0], c[1]]) as f64)
}

fn dequant(q: impl Iterator<Item = f64>, min: [f64; 3], max: [f64; 3]) -> Vec<f32> {
    q.enumerate()
        .map(|(i, v)| {
            let k = i % 3;
            let span = if max[k] > min[k] { max[k] - min[k] } else { 1.0 };
            (min[k] + (v + 32767.0) / 65534.0 * span) as f32
        })
        .collect()
}

/// sRGB byte → linear.
fn srgb_lin(c: u8) -> f32 {
    let c = c as f64 / 255.0;
    (if c <= 0.04045 { c / 12.92 } else { ((c + 0.055) / 1.055).powf(2.4) }) as f32
}

/// Mesh `m` of `body` decoded: positions in the body's frame, or, for a rigid
/// mesh, in its bone's frame (the rest frame is the head, unrotated).
pub fn decode_mesh(body: &BodyDef, m: &MeshDef) -> Geometry {
    let mut position = dequant(i16s(&unpack(m.pos)), m.min, m.max);
    if let Some(b) = m.bone.and_then(|i| body.bones.get(i)) {
        for (i, p) in position.iter_mut().enumerate() {
            *p -= b.head[i % 3] as f32;
        }
    }
    let normal: Vec<f32> = unpack(m.nrm).iter().map(|&q| q as i8 as f32 / 127.0).collect();
    let mut g = Geometry { position, normal, ..Default::default() };
    if let Some(c) = m.col {
        g.color = unpack(c).into_iter().map(srgb_lin).collect();
        g.color_size = 3;
    }
    g.uv = match m.uv {
        Some(u) => unpack(u).chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]]) as f32 / 65535.0).collect(),
        None => vec![0.0; m.n * 2],
    };
    if let (Some(j), Some(w)) = (m.jnt, m.wgt) {
        g.skin_index = unpack(j).into_iter().map(|v| v as f32).collect();
        g.skin_weight = unpack(w).into_iter().map(|v| v as f32 / 255.0).collect();
        g.bone_names = body.bones.iter().map(|b| b.name.clone()).collect();
    }
    let ib = unpack(m.idx);
    g.index = Some(if m.n > 65535 {
        ib.chunks_exact(4).map(|c| u32::from_le_bytes([c[0], c[1], c[2], c[3]])).collect()
    } else {
        ib.chunks_exact(2).map(|c| u16::from_le_bytes([c[0], c[1]]) as u32).collect()
    });
    for k in &m.morphs {
        g.morph_position.push(dequant(i16s(&unpack(k.pos)), k.min, k.max));
        g.morph_normal.push(match k.nrm {
            Some(nb) => unpack(nb).iter().map(|&q| q as i8 as f32 / 63.0).collect(),
            None => vec![0.0; m.n * 3],
        });
    }
    g.morph_relative = !m.morphs.is_empty();
    g.compute_bounding_box();
    g.compute_bounding_sphere();
    g
}

thread_local! {
    static CACHE: RefCell<HashMap<(String, String), Geo>> = RefCell::new(HashMap::new());
}

/// Mesh `mesh` of body `body`, decoded once and shared.
pub fn body_geometry(body: &str, mesh: &str) -> Option<Geo> {
    let key = (body.to_string(), mesh.to_string());
    if let Some(g) = CACHE.with(|c| c.borrow().get(&key).cloned()) {
        return Some(g);
    }
    let b = self::body(body)?;
    let m = b.meshes.iter().find(|m| m.name == mesh)?;
    let g = Geo::from_geometry(decode_mesh(b, m));
    CACHE.with(|c| c.borrow_mut().insert(key, g.clone()));
    Some(g)
}
