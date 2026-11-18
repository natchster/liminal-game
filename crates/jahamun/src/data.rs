//! The set file (`assets/set.bin`, written by `assets/src/jahamun/set.py`): a JSON
//! manifest and the meshes' arrays, deflated.
//!
//! ```text
//! "JHMSET\0\x01"  u32 raw length  zlib( u32 json length, json, padding to 4,
//!                                       u32 vertices, u32 indices,
//!                                       f32 position[3v], f32 normal[3v], f32 uv[2v], f32 light[3v],
//!                                       u32 index[i] )   (indices count from each mesh's first vertex)
//! ```
//!
//! Everything is in the engine's frame and in metres: x across the bore
//! (the walkway at -x), y up, -z into the tunnel, the portal at z = 0. A
//! piece's meshes are in its own frame; a placement puts it in the set's.
//! `light` is Cycles' diffuse light at each corner (direct and indirect,
//! the surface's own colour left out), linear and unbounded: the set
//! multiplies it by the photographed albedo (`look`).
use serde_json::Value;
use std::collections::BTreeMap;

/// What a material is drawn as (the manifest's `kind`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MatKind {
    /// a photographed albedo under its baked light
    Baked,
    /// a sign: its picture is painted here (`signs`), under its baked light
    Sign,
    /// glass: what is behind it shows, with the lamps' reflections
    Glass,
    /// a light's own face
    Emissive,
}

#[derive(Clone, Debug)]
pub struct MatDef {
    pub kind: MatKind,
    /// the albedo photograph (`assets/textures/`), if any
    pub texture: Option<String>,
    /// linear rgb, multiplying the photograph (or the colour, without one)
    pub tint: [f64; 3],
}

#[derive(Clone, Debug)]
pub struct MeshData {
    pub material: String,
    pub vertices: (usize, usize),
    pub indices: (usize, usize),
    /// min x, y, z, max x, y, z (the piece's frame)
    pub bounds: [f64; 6],
}

/// A light source as it was baked (the set lights its highlights and glows
/// from these; the light on the surfaces is the bake's).
#[derive(Clone, Debug)]
pub struct LightData {
    /// "led" (a walkway bar), "lamp" (the road's), "street", "rail", "red", "signal", "window"
    pub kind: String,
    pub at: [f64; 3],
    pub dir: [f64; 3],
    /// the emitting rectangle: across, along `axis`
    pub size: [f64; 2],
    pub axis: [f64; 3],
    pub colour: [f64; 3],
    pub watts: f64,
}

#[derive(Clone, Debug, Default)]
pub struct Piece {
    pub meshes: Vec<MeshData>,
    pub lights: Vec<LightData>,
}

#[derive(Clone, Debug)]
pub struct Placement {
    pub piece: String,
    pub at: [f64; 3],
    /// the module's index along the bore (0 at the portal), if a module
    pub index: Option<i64>,
    /// mirrored about x = mirror / 2 (the other bore)
    pub mirror: Option<f64>,
    /// 1 the walkway's bore, 2 the other
    pub bore: i64,
}

/// A box of walkable ground: x0..x1, z0..z1 at height y.
#[derive(Clone, Debug)]
pub struct WalkBox {
    pub x: [f64; 2],
    pub z: [f64; 2],
    pub y: f64,
    pub name: String,
}

/// A lit place (for the fog): its box and how lit (0..1).
#[derive(Clone, Debug)]
pub struct LitBox {
    pub x: [f64; 2],
    pub z: [f64; 2],
    pub lit: f64,
}

pub struct SetData {
    pub module: f64,
    pub tunnel: f64,
    pub mirror: f64,
    pub materials: BTreeMap<String, MatDef>,
    pub pieces: BTreeMap<String, Piece>,
    pub placements: Vec<Placement>,
    pub walk: Vec<WalkBox>,
    pub lit: Vec<LitBox>,
    /// name -> x, y (feet), z, heading (radians; 0 looks into the tunnel, -z)
    pub spawns: BTreeMap<String, [f64; 4]>,
    pub position: Vec<f32>,
    pub normal: Vec<f32>,
    pub uv: Vec<f32>,
    pub light: Vec<f32>,
    pub index: Vec<u32>,
}

fn f(v: &Value) -> f64 {
    v.as_f64().unwrap_or(0.0)
}
fn arr<const N: usize>(v: &Value) -> [f64; N] {
    let mut out = [0.0; N];
    if let Some(a) = v.as_array() {
        for (i, x) in a.iter().take(N).enumerate() {
            out[i] = f(x);
        }
    }
    out
}

fn le_u32(b: &[u8], at: usize) -> u32 {
    u32::from_le_bytes([b[at], b[at + 1], b[at + 2], b[at + 3]])
}

fn floats(b: &[u8], at: usize, n: usize) -> Vec<f32> {
    b[at..at + n * 4].as_chunks::<4>().0.iter().map(|c| f32::from_le_bytes(*c)).collect()
}

impl SetData {
    /// Read a set file.
    pub fn read(bytes: &[u8]) -> Result<SetData, String> {
        if bytes.len() < 12 || &bytes[0..8] != b"JHMSET\x00\x01" {
            return Err("not a Jahamun set file (JHMSET v1)".into());
        }
        let raw_len = le_u32(bytes, 8) as usize;
        let body = miniz_oxide::inflate::decompress_to_vec_zlib(&bytes[12..]).map_err(|e| format!("inflating the set: {e:?}"))?;
        if body.len() != raw_len {
            return Err(format!("the set's body is {} bytes, not {raw_len}", body.len()));
        }
        let jl = le_u32(&body, 0) as usize;
        let js: Value = serde_json::from_slice(&body[4..4 + jl]).map_err(|e| format!("the set's manifest: {e}"))?;
        let mut at = (4 + jl).div_ceil(4) * 4;
        let nv = le_u32(&body, at) as usize;
        let ni = le_u32(&body, at + 4) as usize;
        at += 8;
        let position = floats(&body, at, nv * 3);
        at += nv * 12;
        let normal = floats(&body, at, nv * 3);
        at += nv * 12;
        let uv = floats(&body, at, nv * 2);
        at += nv * 8;
        let light = floats(&body, at, nv * 3);
        at += nv * 12;
        let index: Vec<u32> = body[at..at + ni * 4].as_chunks::<4>().0.iter().map(|c| u32::from_le_bytes(*c)).collect();

        let mut materials = BTreeMap::new();
        if let Some(m) = js["materials"].as_object() {
            for (k, v) in m {
                let kind = match v["kind"].as_str().unwrap_or("baked") {
                    "sign" => MatKind::Sign,
                    "glass" => MatKind::Glass,
                    "emissive" => MatKind::Emissive,
                    _ => MatKind::Baked,
                };
                materials.insert(k.clone(), MatDef { kind, texture: v["texture"].as_str().map(String::from), tint: arr::<3>(&v["tint"]) });
            }
        }
        let mut pieces = BTreeMap::new();
        if let Some(p) = js["pieces"].as_object() {
            for (k, v) in p {
                let meshes = v["meshes"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .map(|m| {
                        let vs = arr::<2>(&m["vertices"]);
                        let is = arr::<2>(&m["indices"]);
                        MeshData {
                            material: m["material"].as_str().unwrap_or("").to_string(),
                            vertices: (vs[0] as usize, vs[1] as usize),
                            indices: (is[0] as usize, is[1] as usize),
                            bounds: arr::<6>(&m["bounds"]),
                        }
                    })
                    .collect();
                let lights = v["lights"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .map(|l| LightData {
                        kind: l["kind"].as_str().unwrap_or("").to_string(),
                        at: arr::<3>(&l["at"]),
                        dir: arr::<3>(&l["dir"]),
                        size: arr::<2>(&l["size"]),
                        axis: arr::<3>(&l["axis"]),
                        colour: arr::<3>(&l["colour"]),
                        watts: f(&l["watts"]),
                    })
                    .collect();
                pieces.insert(k.clone(), Piece { meshes, lights });
            }
        }
        let placements = js["placements"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|p| Placement {
                piece: p["piece"].as_str().unwrap_or("").to_string(),
                at: arr::<3>(&p["at"]),
                index: p["index"].as_i64(),
                mirror: p["mirror"].as_f64(),
                bore: p["bore"].as_i64().unwrap_or(1),
            })
            .collect();
        let walk = js["walk"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|w| WalkBox {
                x: arr::<2>(&w["x"]),
                z: arr::<2>(&w["z"]),
                y: f(&w["y"]),
                name: w["name"].as_str().unwrap_or("").to_string(),
            })
            .collect();
        let lit = js["lit"]
            .as_array()
            .into_iter()
            .flatten()
            .map(|w| LitBox { x: arr::<2>(&w["x"]), z: arr::<2>(&w["z"]), lit: f(&w["lit"]) })
            .collect();
        let mut spawns = BTreeMap::new();
        if let Some(s) = js["spawns"].as_object() {
            for (k, v) in s {
                spawns.insert(k.clone(), arr::<4>(v));
            }
        }
        Ok(SetData {
            module: f(&js["module"]),
            tunnel: f(&js["tunnel"]),
            mirror: f(&js["mirror"]),
            materials,
            pieces,
            placements,
            walk,
            lit,
            spawns,
            position,
            normal,
            uv,
            light,
            index,
        })
    }
}
