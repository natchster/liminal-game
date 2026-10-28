//! `cargo xtask assets creatures`: build the creatures' bodies in Blender
//! and pack them into `crates/game/assets/bodies.bin`.
//!
//! ```text
//! cargo xtask assets creatures [--only hound,smiler] [--write]
//! ```
//!
//! Blender runs `assets/src/creatures/build.py` headless; each body it
//! builds is written as `target/xtask/assets/creatures/<name>.json` (the
//! skeleton, and per mesh flat lists of positions, normals, colours, UVs,
//! bone slots, weights, indices and morph offsets, plus a 4-view preview
//! PNG beside it). They are packed the way the game reads them (see
//! `crates/game/src/assets/bodies.rs` for the format) and compared, body by
//! body, with the committed file. With `--write` (a full build) the file is
//! replaced and its fingerprint recorded in `MANIFEST.json`.
use crate::shots::read_json;
use crate::{flag, opt, root};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

pub const FILE: &str = "bodies.bin";

fn f3(v: &Value) -> [f64; 3] {
    let a = |i: usize| v.get(i).and_then(Value::as_f64).unwrap_or(0.0);
    [a(0), a(1), a(2)]
}

fn floats(v: &Value) -> Vec<f64> {
    v.as_array().map(|a| a.iter().map(|x| x.as_f64().unwrap_or(0.0)).collect()).unwrap_or_default()
}

/// The packed buffers and the header entry of one body.
struct Packer {
    data: Vec<u8>,
}

impl Packer {
    fn put(&mut self, raw: &[u8]) -> Value {
        let z = miniz_oxide::deflate::compress_to_vec(raw, 9);
        let off = self.data.len();
        self.data.extend_from_slice(&z);
        json!([off, z.len(), raw.len()])
    }

    fn quant(&mut self, v: &[f64]) -> (Value, [f64; 3], [f64; 3]) {
        let (mut lo, mut hi) = ([f64::INFINITY; 3], [f64::NEG_INFINITY; 3]);
        for (i, x) in v.iter().enumerate() {
            lo[i % 3] = lo[i % 3].min(*x);
            hi[i % 3] = hi[i % 3].max(*x);
        }
        for k in 0..3 {
            if !lo[k].is_finite() {
                lo[k] = 0.0;
                hi[k] = 0.0;
            }
            // round the box outward to 1e-5 m so the header's numbers are short
            lo[k] = (lo[k] * 1e5).floor() / 1e5;
            hi[k] = (hi[k] * 1e5).ceil() / 1e5;
        }
        let mut raw = Vec::with_capacity(v.len() * 2);
        for (i, x) in v.iter().enumerate() {
            let k = i % 3;
            let span = if hi[k] > lo[k] { hi[k] - lo[k] } else { 1.0 };
            let q = (((x - lo[k]) / span) * 65534.0).round() as i64 - 32767;
            raw.extend_from_slice(&(q.clamp(-32767, 32767) as i16).to_le_bytes());
        }
        (self.put(&raw), lo, hi)
    }

    fn snorm8(&mut self, v: &[f64], scale: f64) -> Value {
        let raw: Vec<u8> = v.iter().map(|x| ((x * scale).round().clamp(-127.0, 127.0) as i8) as u8).collect();
        self.put(&raw)
    }

    fn unorm8(&mut self, v: &[f64], scale: f64) -> Value {
        let raw: Vec<u8> = v.iter().map(|x| (x * scale).round().clamp(0.0, 255.0) as u8).collect();
        self.put(&raw)
    }

    fn mesh(&mut self, m: &Value) -> Result<Value, String> {
        let name = m["name"].as_str().unwrap_or("?");
        let pos = floats(&m["pos"]);
        let n = pos.len() / 3;
        let idx = floats(&m["idx"]);
        if pos.len() % 3 != 0 || idx.len() % 3 != 0 {
            return Err(format!("mesh {name}: positions or indices not in threes"));
        }
        if let Some(bad) = idx.iter().find(|&&i| i < 0.0 || i as usize >= n) {
            return Err(format!("mesh {name}: index {bad} out of range ({n} vertices)"));
        }
        let (pos_b, lo, hi) = self.quant(&pos);
        let nrm = self.snorm8(&floats(&m["nrm"]), 127.0);
        let col = if m["col"].is_array() { self.unorm8(&floats(&m["col"]), 255.0) } else { Value::Null };
        let uv = if m["uv"].is_array() {
            let raw: Vec<u8> = floats(&m["uv"]).iter().flat_map(|x| ((x.clamp(0.0, 1.0) * 65535.0).round() as u16).to_le_bytes()).collect();
            self.put(&raw)
        } else {
            Value::Null
        };
        let (jnt, wgt) = if m["jnt"].is_array() {
            let w = floats(&m["wgt"]);
            let j = floats(&m["jnt"]);
            if j.len() != n * 4 || w.len() != n * 4 {
                return Err(format!("mesh {name}: bone slots or weights are not four per vertex"));
            }
            for v in 0..n {
                let s: f64 = w[v * 4..v * 4 + 4].iter().sum();
                if (s - 255.0).abs() > 0.5 {
                    return Err(format!("mesh {name}: vertex {v}'s weights sum to {s}, not 255"));
                }
            }
            (self.unorm8(&j, 1.0), self.unorm8(&w, 1.0))
        } else {
            (Value::Null, Value::Null)
        };
        let idx_raw: Vec<u8> = if n > 65535 {
            idx.iter().flat_map(|&i| (i as u32).to_le_bytes()).collect()
        } else {
            idx.iter().flat_map(|&i| (i as u16).to_le_bytes()).collect()
        };
        let idx_b = self.put(&idx_raw);
        let mut morphs = vec![];
        for k in m["morphs"].as_array().into_iter().flatten() {
            let d = floats(&k["pos"]);
            if d.len() != pos.len() {
                return Err(format!("mesh {name}: morph {} has {} offsets for {} vertices", k["name"], d.len() / 3, n));
            }
            let (pb, klo, khi) = self.quant(&d);
            let nb = if k["nrm"].is_array() { self.snorm8(&floats(&k["nrm"]), 63.0) } else { Value::Null };
            morphs.push(json!({"name": k["name"], "min": klo, "max": khi, "pos": pb, "nrm": nb}));
        }
        Ok(json!({
            "name": name, "material": m["material"].as_str().unwrap_or("skin"),
            "bone": m["bone"].as_i64().unwrap_or(-1),
            "n": n, "t": idx.len() / 3, "min": lo, "max": hi,
            "pos": pos_b, "nrm": nrm, "col": col, "uv": uv, "jnt": jnt, "wgt": wgt, "idx": idx_b,
            "morphs": morphs,
        }))
    }
}

/// Pack the built bodies (in the order given) into the game's file.
pub fn pack(bodies: &[Value]) -> Result<Vec<u8>, String> {
    let mut p = Packer { data: vec![] };
    let mut out = vec![];
    for b in bodies {
        let name = b["name"].as_str().ok_or("a body without a name")?;
        let bones: Vec<Value> = b["bones"].as_array().cloned().unwrap_or_default();
        for (i, bone) in bones.iter().enumerate() {
            let par = bone["parent"].as_i64().unwrap_or(-1);
            if par >= i as i64 {
                return Err(format!("{name}: bone {} comes before its parent", bone["name"]));
            }
        }
        let bones: Vec<Value> = bones
            .iter()
            .map(|x| {
                let r = |v: [f64; 3]| v.map(|c| (c * 1e5).round() / 1e5);
                json!({"name": x["name"], "parent": x["parent"].as_i64().unwrap_or(-1), "head": r(f3(&x["head"])), "tail": r(f3(&x["tail"]))})
            })
            .collect();
        let mut meshes = vec![];
        for m in b["meshes"].as_array().into_iter().flatten() {
            let bone = m["bone"].as_i64().unwrap_or(-1);
            if bone >= bones.len() as i64 {
                return Err(format!("{name}: mesh {} is on bone {bone} of {}", m["name"], bones.len()));
            }
            if m["jnt"].is_array() && floats(&m["jnt"]).iter().any(|&j| j as usize >= bones.len()) {
                return Err(format!("{name}: mesh {} has a bone slot past the skeleton", m["name"]));
            }
            meshes.push(p.mesh(m)?);
        }
        out.push(json!({"name": name, "bones": bones, "meshes": meshes}));
    }
    let header = serde_json::to_vec(&json!({"bodies": out})).map_err(|e| e.to_string())?;
    let mut file = b"BODIES01".to_vec();
    file.extend_from_slice(&(header.len() as u32).to_le_bytes());
    file.extend_from_slice(&header);
    file.extend_from_slice(&p.data);
    Ok(file)
}

/// The committed file's header (bodies by name), if there is one.
fn committed(path: &Path) -> BTreeMap<String, Value> {
    let Ok(b) = std::fs::read(path) else { return BTreeMap::new() };
    if b.len() < 12 || &b[..8] != b"BODIES01" {
        return BTreeMap::new();
    }
    let hl = u32::from_le_bytes([b[8], b[9], b[10], b[11]]) as usize;
    let h: Value = serde_json::from_slice(&b[12..12 + hl]).unwrap_or(Value::Null);
    h["bodies"].as_array().into_iter().flatten().map(|x| (x["name"].as_str().unwrap_or("").to_string(), x.clone())).collect()
}

/// A body's shape, for the comparison: bone names and per mesh its counts.
fn summary(b: &Value) -> String {
    let bones = b["bones"].as_array().map(|a| a.len()).unwrap_or(0);
    let meshes: Vec<String> = b["meshes"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|m| format!("{}:{}v/{}t", m["name"].as_str().unwrap_or("?"), m["n"], m["t"]))
        .collect();
    format!("{bones} bones; {}", meshes.join(", "))
}

pub fn creatures(args: &[String], blender: &str, asset_dir: &Path) -> Result<(), String> {
    let only = opt(args, "--only");
    let write = flag(args, "--write");
    if write && only.is_some() {
        return Err("--write needs a full build (no --only): every body comes from this build".into());
    }
    let out = root().join("target/xtask/assets/creatures");
    let _ = std::fs::remove_dir_all(&out);
    std::fs::create_dir_all(&out).map_err(|e| e.to_string())?;
    let mut cmd = Command::new(blender);
    cmd.args(["--background", "--factory-startup", "--python"])
        .arg(root().join("assets/src/creatures/build.py"))
        .arg("--")
        .arg("--out")
        .arg(&out);
    if let Some(o) = only {
        cmd.args(["--only", o]);
    }
    let o = cmd.output().map_err(|e| format!("running Blender ({blender}): {e}; set $BLENDER"))?;
    let log = String::from_utf8_lossy(&o.stdout);
    for l in log.lines().filter(|l| l.starts_with("[creatures]")) {
        println!("{l}");
    }
    if !o.status.success() || log.contains("Traceback") {
        let _ = std::fs::write(out.join("blender.log"), format!("{log}{}", String::from_utf8_lossy(&o.stderr)));
        return Err(format!("the Blender build failed; full output in {}", out.join("blender.log").display()));
    }
    // the build lists its bodies in the order the file keeps them
    let order = read_json(&out.join("bodies.json"));
    let names: Vec<String> = order.as_array().into_iter().flatten().filter_map(|v| v.as_str().map(String::from)).collect();
    let bodies: Vec<Value> = names.iter().map(|n| read_json(&out.join(format!("{n}.json")))).collect();
    let file = pack(&bodies)?;
    let built = {
        let tmp: PathBuf = out.join(FILE);
        std::fs::write(&tmp, &file).map_err(|e| e.to_string())?;
        committed(&tmp)
    };
    let path = asset_dir.join(FILE);
    let before = committed(&path);
    for (n, b) in &built {
        match before.get(n) {
            None => println!("new      {n}: {}", summary(b)),
            Some(c) if summary(c) == summary(b) => println!("same     {n}: {}", summary(b)),
            Some(c) => println!("changed  {n}: {} -> {}", summary(c), summary(b)),
        }
    }
    for n in before.keys().filter(|n| !built.contains_key(*n)) {
        println!("{}  {n}: not built", if only.is_some() { "skipped" } else { "gone   " });
    }
    println!("{} bytes ({} committed); previews in {}", file.len(), std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0), out.display());
    if write {
        std::fs::write(&path, &file).map_err(|e| e.to_string())?;
        crate::assets::update_manifest(
            FILE,
            &file,
            json!({
                "generator": "procedural bpy scripts",
                "generative_model": null,
                "licence": "MIT",
                "authors": "written by coding agents under the developer's direction; the bodies that were people start from MakeHuman's base mesh and targets (CC0 1.0, the MakeHuman team)",
                "built_with": format!("{} headless: skeletons, bodies (MakeHuman's hm08 base and its targets, or metaballs), voxel remesh, displacement, decimation, bone-heat weights, vertex paint; packed by cargo xtask assets creatures", crate::assets::blender_version()),
                "sources": ["assets/src/creatures/build.py", "assets/src/creatures/kit.py", "assets/src/creatures/hm08.py", "assets/src/creatures/sources.json"],
                "rebuild": "cargo xtask assets creatures [--only …] [--write]; the MakeHuman files and their SHA-1s are in assets/src/creatures/sources.json (fetched once into target/creature-sources/)",
            }),
        )?;
        println!("wrote {} and its fingerprint", path.display());
    }
    Ok(())
}
