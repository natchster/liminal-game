//! `cargo xtask assets`: rebuild and check the embedded meshes.
//!
//! ```text
//! cargo xtask assets props [--batch b01,b02] [--write [--rebuild-frozen]]
//! cargo xtask assets import-js --props FILE.js --creatures FILE.js
//! cargo xtask assets textures [--write]
//! cargo xtask assets fingerprint
//! ```
//!
//! **Props** come from the bpy scripts in `assets/src/blender/` (`props.py`
//! runs every batch module in `props/`, each part a `propkit` build).
//! `assets props` runs them in Blender headless (`$BLENDER`, else `blender`
//! on the path, else the app bundle), packs every part the way the game
//! reads it (int16 positions, int8 normals, uint16 indices; planar,
//! delta-coded, raw-deflated) and compares each with
//! `crates/game/assets/prop_meshes.bin` by its decoded geometry. With
//! `--write` (a full build only) it writes the file, keeping the stored bytes
//! of every unchanged part so the diff is only what changed, and updates the
//! file's fingerprint in `crates/game/assets/MANIFEST.json`.
//!
//! *Frozen* parts (`frozen_parts` in the manifest) were built before the
//! build was repeatable (their noise came from `noise_vector`, which differs
//! in every Blender process). A rebuild gives them new shapes; they are
//! reported apart and `--write` keeps their stored bytes unless
//! `--rebuild-frozen` is given (then re-record the prop fixtures, which
//! sample some of them, and take them out of the list).
//!
//! **Creatures** are rebuilt by `creatures.py` / `skinned.py`, which need the
//! rig dumps of the HTML version (`tools/rig-dump.mjs` there); until that is
//! ported, rebuild them in the HTML version and convert its
//! `src/creature-meshes.js` with `assets import-js` (which also converts a
//! `src/prop-meshes.js`; both conversions reproduce the committed files byte
//! for byte).
//!
//! **Textures** (the Building's photographed surfaces, `textures/*.jpg`) come
//! from `assets/src/textures/build.py`, which fetches the CC0 photographs its
//! `sources.json` lists, checks each by its SHA-1, and builds every texture.
//! `assets textures` runs it (`$PYTHON`, else `python3`; it needs Pillow,
//! NumPy and SciPy) and compares; with `--write` it replaces the files and
//! records their fingerprints in the manifest.
use crate::shots::{read_json, write_json};
use crate::{flag, opt, root};
use base64::Engine as _;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::Command;

pub fn assets(args: &[String]) -> Result<(), String> {
    match args.first().map(String::as_str) {
        Some("props") => props(&args[1..]),
        Some("import-js") => import_js(&args[1..]),
        Some("textures") => textures(&args[1..]),
        Some("creatures") => crate::bodies::creatures(&args[1..], &blender(), &asset_dir()),
        Some("fingerprint") => {
            for f in ["prop_meshes.bin", "creature_meshes.bin", crate::bodies::FILE] {
                println!("{f}: {}", fingerprint(&std::fs::read(asset_dir().join(f)).map_err(|e| e.to_string())?));
            }
            for f in texture_files()? {
                println!("{f}: {}", fingerprint(&std::fs::read(asset_dir().join(&f)).map_err(|e| e.to_string())?));
            }
            Ok(())
        }
        _ => Err(
            "usage: cargo xtask assets props [--batch …] [--write] | creatures [--only …] [--write] | textures [--write] | import-js --props F --creatures F | fingerprint"
                .into(),
        ),
    }
}

fn asset_dir() -> PathBuf {
    root().join("crates/game/assets")
}

/// FNV-1a 64 of a file's bytes, as hex: the manifest's change check.
pub fn fingerprint(bytes: &[u8]) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        h ^= *b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{h:016x}")
}

fn blender() -> String {
    if let Ok(b) = std::env::var("BLENDER") {
        return b;
    }
    let on_path = Command::new("blender").arg("--version").output().is_ok_and(|o| o.status.success());
    if on_path {
        "blender".into()
    } else {
        "/Applications/Blender.app/Contents/MacOS/Blender".into()
    }
}

// ------------------------------------------------------------ the prop file

struct PropPart {
    name: String,
    frame: u8,
    n: u32,
    t: u32,
    min: [f64; 3],
    max: [f64; 3],
    fmin: [f64; 3],
    fmax: [f64; 3],
    /// raw-deflated planar delta-coded buffers
    z: Vec<u8>,
    /// the same, inflated (`r` bytes)
    raw: Vec<u8>,
}

const FRAMES: [&str; 5] = ["box", "cyl", "sphere", "torus", "plane"];

fn frame_code(f: &str) -> u8 {
    FRAMES.iter().position(|x| *x == f).map(|i| i as u8).unwrap_or(5)
}

struct Rd<'a> {
    b: &'a [u8],
    p: usize,
}
impl<'a> Rd<'a> {
    fn take(&mut self, n: usize) -> &'a [u8] {
        let s = &self.b[self.p..self.p + n];
        self.p += n;
        s
    }
    fn u8(&mut self) -> u8 {
        self.take(1)[0]
    }
    fn u16(&mut self) -> u16 {
        u16::from_le_bytes(self.take(2).try_into().unwrap())
    }
    fn u32(&mut self) -> u32 {
        u32::from_le_bytes(self.take(4).try_into().unwrap())
    }
    fn f3(&mut self) -> [f64; 3] {
        let mut o = [0.0; 3];
        for v in o.iter_mut() {
            *v = f64::from_le_bytes(self.take(8).try_into().unwrap());
        }
        o
    }
}

fn read_props(bytes: &[u8]) -> Result<Vec<PropPart>, String> {
    let mut rd = Rd { b: bytes, p: 0 };
    if rd.take(8) != b"PROPMSH1" {
        return Err("prop_meshes.bin: bad magic".into());
    }
    let count = rd.u32();
    let mut out = vec![];
    for _ in 0..count {
        let nl = rd.u16() as usize;
        let name = String::from_utf8_lossy(rd.take(nl)).into_owned();
        let frame = rd.u8();
        let (n, t, r) = (rd.u32(), rd.u32(), rd.u32());
        let (min, max, fmin, fmax) = (rd.f3(), rd.f3(), rd.f3(), rd.f3());
        let zl = rd.u32() as usize;
        let z = rd.take(zl).to_vec();
        let raw = miniz_oxide::inflate::decompress_to_vec(&z).map_err(|e| format!("{name}: inflate: {e:?}"))?;
        if raw.len() != r as usize {
            return Err(format!("{name}: {} raw bytes, header says {r}", raw.len()));
        }
        out.push(PropPart { name, frame, n, t, min, max, fmin, fmax, z, raw });
    }
    Ok(out)
}

fn write_props(parts: &[PropPart]) -> Vec<u8> {
    let mut b = b"PROPMSH1".to_vec();
    b.extend((parts.len() as u32).to_le_bytes());
    for p in parts {
        b.extend((p.name.len() as u16).to_le_bytes());
        b.extend(p.name.as_bytes());
        b.push(p.frame);
        for v in [p.n, p.t, p.raw.len() as u32] {
            b.extend(v.to_le_bytes());
        }
        for a in [p.min, p.max, p.fmin, p.fmax] {
            for v in a {
                b.extend(v.to_le_bytes());
            }
        }
        b.extend((p.z.len() as u32).to_le_bytes());
        b.extend(&p.z);
    }
    b
}

/// A built part (the batch JSON of `props.py`) as the game stores it.
fn pack_part(name: &str, d: &Value) -> Result<PropPart, String> {
    let b64 =
        |k: &str| base64::engine::general_purpose::STANDARD.decode(d[k].as_str().unwrap_or("")).map_err(|e| format!("{name}.{k}: {e}"));
    let (v, m, i) = (b64("v")?, b64("m")?, b64("i")?);
    let n = d["n"].as_u64().unwrap_or(0) as usize;
    let v: Vec<i16> = v.chunks(2).map(|c| i16::from_le_bytes([c[0], c[1]])).collect();
    let m: Vec<i8> = m.iter().map(|&x| x as i8).collect();
    let idx: Vec<u16> = i.chunks(2).map(|c| u16::from_le_bytes([c[0], c[1]])).collect();
    let (mut pv, mut pm) = (vec![0i16; n * 3], vec![0i8; n * 3]);
    for k in 0..3 {
        let (mut prev, mut prev_n) = (0i16, 0i8);
        for j in 0..n {
            pv[k * n + j] = v[j * 3 + k].wrapping_sub(prev);
            prev = v[j * 3 + k];
            pm[k * n + j] = m[j * 3 + k].wrapping_sub(prev_n);
            prev_n = m[j * 3 + k];
        }
    }
    let mut prev = 0u16;
    let pi: Vec<u16> = idx
        .iter()
        .map(|&x| {
            let d = x.wrapping_sub(prev);
            prev = x;
            d
        })
        .collect();
    let mut raw = Vec::with_capacity(n * 9 + pi.len() * 2);
    raw.extend(pv.iter().flat_map(|x| x.to_le_bytes()));
    raw.extend(pm.iter().map(|&x| x as u8));
    raw.extend(pi.iter().flat_map(|x| x.to_le_bytes()));
    let f3 = |k: &str| -> [f64; 3] {
        let a = d[k].as_array().cloned().unwrap_or_default();
        [0, 1, 2].map(|i| a.get(i).and_then(|v| v.as_f64()).unwrap_or(0.0))
    };
    Ok(PropPart {
        name: name.to_string(),
        frame: frame_code(d["f"].as_str().unwrap_or("")),
        n: n as u32,
        t: d["t"].as_u64().unwrap_or(0) as u32,
        min: f3("min"),
        max: f3("max"),
        fmin: f3("fmin"),
        fmax: f3("fmax"),
        z: miniz_oxide::deflate::compress_to_vec(&raw, 9),
        raw,
    })
}

fn props(args: &[String]) -> Result<(), String> {
    let batches = opt(args, "--batch");
    let write = flag(args, "--write");
    if write && batches.is_some() {
        return Err("--write needs a full build (no --batch): every part comes from this build".into());
    }
    let out = root().join("target/xtask/assets/props");
    let _ = std::fs::remove_dir_all(&out);
    std::fs::create_dir_all(&out).map_err(|e| e.to_string())?;
    let mut cmd = Command::new(blender());
    // (a script error exits 1: Blender's own exit code is 0 after one)
    cmd.args(["--background", "--factory-startup", "--python-exit-code", "1", "--python"])
        .arg(root().join("assets/src/blender/props.py"))
        .arg("--")
        .arg("--out")
        .arg(&out);
    if let Some(b) = batches {
        cmd.args(["--batch", b]);
    }
    let o = cmd.output().map_err(|e| format!("running Blender ({}): {e}; set $BLENDER", blender()))?;
    let log = String::from_utf8_lossy(&o.stdout);
    for l in log.lines().filter(|l| l.starts_with("[props]")) {
        println!("{l}");
    }
    if !o.status.success() || log.contains("Traceback") {
        let _ = std::fs::write(out.join("blender.log"), format!("{log}{}", String::from_utf8_lossy(&o.stderr)));
        return Err(format!("the Blender build failed; full output in {}", out.join("blender.log").display()));
    }
    // the built parts
    let mut built: BTreeMap<String, PropPart> = BTreeMap::new();
    let mut files: Vec<PathBuf> = std::fs::read_dir(&out).map_err(|e| e.to_string())?.flatten().map(|e| e.path()).collect();
    files.sort();
    for f in files {
        let name = f.file_name().unwrap().to_string_lossy().to_string();
        if !name.ends_with(".json") || name.ends_with(".report.json") || (name.starts_with("zz_") && batches.is_none()) {
            continue;
        }
        let parts = read_json(&f);
        for (pname, d) in parts.as_object().into_iter().flatten() {
            if built.contains_key(pname) {
                return Err(format!("part {pname} is built twice (in {name} and an earlier batch)"));
            }
            built.insert(pname.clone(), pack_part(pname, d)?);
        }
    }
    let path = asset_dir().join("prop_meshes.bin");
    let committed = read_props(&std::fs::read(&path).map_err(|e| e.to_string())?)?;
    let by_name: BTreeMap<&str, &PropPart> = committed.iter().map(|p| (p.name.as_str(), p)).collect();
    let manifest = read_json(&asset_dir().join("MANIFEST.json"));
    let frozen: std::collections::BTreeSet<String> = manifest["files"]["prop_meshes.bin"]["frozen_parts"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|v| v.as_str().map(String::from))
        .collect();
    let rebuild_frozen = flag(args, "--rebuild-frozen");
    let (mut same, mut changed, mut new, mut kept) = (0, vec![], vec![], 0);
    for (name, p) in &built {
        match by_name.get(name.as_str()) {
            Some(c) if c.raw == p.raw && c.n == p.n && c.t == p.t && c.frame == p.frame && c.min == p.min && c.max == p.max => same += 1,
            Some(_) if frozen.contains(name) => kept += 1,
            Some(_) => changed.push(name.clone()),
            None => new.push(name.clone()),
        }
    }
    let _ = std::fs::write(out.join("changed.txt"), changed.join("\n"));
    let gone: Vec<&str> =
        if batches.is_none() { committed.iter().map(|p| p.name.as_str()).filter(|n| !built.contains_key(*n)).collect() } else { vec![] };
    println!(
        "{} parts built: {same} identical to prop_meshes.bin, {} changed, {} new{}, {kept} frozen (they differ; kept unless --rebuild-frozen)",
        built.len(),
        changed.len(),
        new.len(),
        if batches.is_none() { format!(", {} no longer built", gone.len()) } else { String::new() }
    );
    for (what, list) in [("changed", &changed), ("new", &new)] {
        if !list.is_empty() {
            println!("  {what}: {}", list.iter().take(20).cloned().collect::<Vec<_>>().join(", "));
        }
    }
    if !gone.is_empty() {
        println!("  no longer built: {}", gone.iter().take(20).cloned().collect::<Vec<_>>().join(", "));
    }
    if write {
        // keep the stored bytes of unchanged parts: the diff is only what changed
        let parts: Vec<PropPart> = built
            .into_values()
            .map(|p| match by_name.get(p.name.as_str()) {
                // the stored part, byte for byte: unchanged, or frozen
                Some(c) if c.raw == p.raw || (frozen.contains(&p.name) && !rebuild_frozen) => PropPart {
                    name: c.name.clone(),
                    frame: c.frame,
                    n: c.n,
                    t: c.t,
                    min: c.min,
                    max: c.max,
                    fmin: c.fmin,
                    fmax: c.fmax,
                    z: c.z.clone(),
                    raw: c.raw.clone(),
                },
                _ => p,
            })
            .collect();
        let bytes = write_props(&parts);
        std::fs::write(&path, &bytes).map_err(|e| e.to_string())?;
        update_manifest("prop_meshes.bin", &bytes, json!({ "parts": parts.len(), "rebuilt_with": blender_version() }))?;
        println!(
            "wrote {} ({} parts); review the parts table diff (REGEN=1 cargo test --release -p game --test asset_lint)",
            path.display(),
            parts.len()
        );
    } else if !changed.is_empty() || !new.is_empty() || !gone.is_empty() {
        println!("run with --write (a full build) to replace prop_meshes.bin");
    }
    Ok(())
}

pub(crate) fn blender_version() -> String {
    Command::new(blender())
        .arg("--version")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).lines().next().unwrap_or("").trim().to_string())
        .unwrap_or_default()
}

/// The photographed textures, as the manifest names them (`textures/<name>.jpg`)
fn texture_files() -> Result<Vec<String>, String> {
    let mut v: Vec<String> = std::fs::read_dir(asset_dir().join("textures"))
        .map_err(|e| e.to_string())?
        .flatten()
        .map(|e| e.file_name().to_string_lossy().to_string())
        .filter(|n| n.ends_with(".jpg"))
        .map(|n| format!("textures/{n}"))
        .collect();
    v.sort();
    Ok(v)
}

/// `assets textures [--write]`: build the photographed textures (and, with
/// `--write`, replace them and record their fingerprints)
fn textures(args: &[String]) -> Result<(), String> {
    let write = flag(args, "--write");
    let python = std::env::var("PYTHON").unwrap_or_else(|_| "python3".into());
    let mut cmd = Command::new(&python);
    cmd.arg("-I").arg(root().join("assets/src/textures/build.py"));
    if write {
        cmd.arg("--write");
    }
    if let Some(only) = opt(args, "--only") {
        cmd.arg("--only").arg(only);
    }
    let st = cmd.status().map_err(|e| format!("{python}: {e} (set $PYTHON to a Python with Pillow, NumPy and SciPy)"))?;
    if !st.success() {
        return Err(format!("build.py failed ({st})"));
    }
    if write {
        for f in texture_files()? {
            let bytes = std::fs::read(asset_dir().join(&f)).map_err(|e| e.to_string())?;
            update_manifest(&f, &bytes, json!({}))?;
            println!("{f}: fingerprint {}", fingerprint(&bytes));
        }
    }
    Ok(())
}

/// Record a file's new fingerprint (and facts about it) in the manifest.
pub(crate) fn update_manifest(file: &str, bytes: &[u8], extra: Value) -> Result<(), String> {
    let p = asset_dir().join("MANIFEST.json");
    let mut m = read_json(&p);
    let e = &mut m["files"][file];
    e["fingerprint"] = json!(fingerprint(bytes));
    for (k, v) in extra.as_object().into_iter().flatten() {
        e[k] = v.clone();
    }
    write_json(&p, &m)
}

// ------------------------------------------------------------ the HTML version's modules

/// `name: { key: value, … }` entries of a generated JS module, one per line.
fn js_entries(src: &str) -> Vec<(String, BTreeMap<String, String>)> {
    let mut out = vec![];
    let mut cur: Option<(String, String)> = None;
    for line in src.lines() {
        let starts = line.starts_with("  ") && !line.starts_with("   ") && line.contains(": {");
        if starts {
            if let Some((n, b)) = cur.take() {
                out.push((n, parse_fields(&b)));
            }
            let (n, rest) = line.trim_start().split_once(": {").unwrap();
            cur = Some((n.to_string(), rest.to_string()));
        } else if let Some((_, b)) = cur.as_mut() {
            b.push(' ');
            b.push_str(line.trim());
        }
    }
    if let Some((n, b)) = cur {
        out.push((n, parse_fields(&b)));
    }
    out.retain(|(n, _)| n.chars().all(|c| c.is_ascii_alphanumeric() || c == '_'));
    out
}

fn parse_fields(body: &str) -> BTreeMap<String, String> {
    let mut m = BTreeMap::new();
    let b: Vec<char> = body.chars().collect();
    let mut i = 0;
    while i < b.len() {
        while i < b.len() && (b[i] == ' ' || b[i] == ',') {
            i += 1;
        }
        let ks = i;
        while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == '_') {
            i += 1;
        }
        if i >= b.len() || b[i] != ':' || ks == i {
            break;
        }
        let key: String = b[ks..i].iter().collect();
        i += 1;
        while i < b.len() && b[i] == ' ' {
            i += 1;
        }
        let vs = i;
        match b.get(i) {
            Some('\'') => {
                i += 1;
                while i < b.len() && b[i] != '\'' {
                    i += 1;
                }
                i += 1;
            }
            Some('[') => {
                while i < b.len() && b[i] != ']' {
                    i += 1;
                }
                i += 1;
            }
            _ => {
                while i < b.len() && b[i] != ',' && b[i] != '}' {
                    i += 1;
                }
            }
        }
        m.insert(key, b[vs..i.min(b.len())].iter().collect::<String>().trim().to_string());
    }
    m
}

fn import_js(args: &[String]) -> Result<(), String> {
    let b64 = |s: &str| base64::engine::general_purpose::STANDARD.decode(s.trim_matches('\'')).map_err(|e| e.to_string());
    let arr = |s: &str| -> [f64; 3] {
        let v: Vec<f64> = s.trim_matches(|c| c == '[' || c == ']').split(',').filter_map(|x| x.trim().parse().ok()).collect();
        [v.first().copied().unwrap_or(0.0), v.get(1).copied().unwrap_or(0.0), v.get(2).copied().unwrap_or(0.0)]
    };
    if let Some(p) = opt(args, "--props") {
        let src = std::fs::read_to_string(p).map_err(|e| format!("{p}: {e}"))?;
        let mut parts = vec![];
        for (name, f) in js_entries(&src) {
            let z = b64(f.get("z").map(String::as_str).unwrap_or(""))?;
            let raw = miniz_oxide::inflate::decompress_to_vec(&z).map_err(|e| format!("{name}: {e:?}"))?;
            parts.push(PropPart {
                frame: frame_code(f.get("f").map(|s| s.trim_matches('\'')).unwrap_or("")),
                n: f.get("n").and_then(|v| v.parse().ok()).unwrap_or(0),
                t: f.get("t").and_then(|v| v.parse().ok()).unwrap_or(0),
                min: arr(f.get("min").map(String::as_str).unwrap_or("")),
                max: arr(f.get("max").map(String::as_str).unwrap_or("")),
                fmin: arr(f.get("fmin").map(String::as_str).unwrap_or("")),
                fmax: arr(f.get("fmax").map(String::as_str).unwrap_or("")),
                name,
                z,
                raw,
            });
        }
        let bytes = write_props(&parts);
        std::fs::write(asset_dir().join("prop_meshes.bin"), &bytes).map_err(|e| e.to_string())?;
        update_manifest(
            "prop_meshes.bin",
            &bytes,
            json!({ "parts": parts.len(), "converted_from": p, "converted_from_fingerprint": fingerprint(src.as_bytes()) }),
        )?;
        println!("prop_meshes.bin: {} parts from {p}", parts.len());
    }
    if let Some(p) = opt(args, "--creatures") {
        let src = std::fs::read_to_string(p).map_err(|e| format!("{p}: {e}"))?;
        let entries = js_entries(&src);
        let mut b = b"CREAMSH1".to_vec();
        b.extend((entries.len() as u32).to_le_bytes());
        for (name, f) in &entries {
            b.extend((name.len() as u16).to_le_bytes());
            b.extend(name.as_bytes());
            let flags = u8::from(f.get("z").map(String::as_str) == Some("1"))
                | (u8::from(f.contains_key("c")) << 1)
                | (u8::from(f.contains_key("j")) << 2)
                | (u8::from(f.contains_key("b")) << 3);
            b.push(flags);
            for k in ["n", "t"] {
                b.extend(f.get(k).and_then(|v| v.parse::<u32>().ok()).unwrap_or(0).to_le_bytes());
            }
            for k in ["min", "max"] {
                for v in arr(f.get(k).map(String::as_str).unwrap_or("")) {
                    b.extend(v.to_le_bytes());
                }
            }
            let bones: Vec<String> = f
                .get("b")
                .map(|s| {
                    s.trim_matches(|c| c == '[' || c == ']')
                        .split(',')
                        .map(|x| x.trim().trim_matches('\'').to_string())
                        .filter(|x| !x.is_empty())
                        .collect()
                })
                .unwrap_or_default();
            b.extend((bones.len() as u16).to_le_bytes());
            for bn in &bones {
                b.extend((bn.len() as u16).to_le_bytes());
                b.extend(bn.as_bytes());
            }
            for k in ["v", "i", "c", "j", "w"] {
                let blob = match f.get(k) {
                    Some(s) => b64(s)?,
                    None => vec![],
                };
                b.extend((blob.len() as u32).to_le_bytes());
                b.extend(blob);
            }
        }
        std::fs::write(asset_dir().join("creature_meshes.bin"), &b).map_err(|e| e.to_string())?;
        update_manifest(
            "creature_meshes.bin",
            &b,
            json!({ "parts": entries.len(), "converted_from": p, "converted_from_fingerprint": fingerprint(src.as_bytes()) }),
        )?;
        println!("creature_meshes.bin: {} parts from {p}", entries.len());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_js_entry_parses_into_its_fields() {
        let src = "export const X = {\n  Box__a: { f: 'box', n: 3, t: 1, r: 9, min: [-1, 0, 2.5], max: [1, 1, 3],\n    z: 'AAAA' },\n};\n";
        let e = js_entries(src);
        assert_eq!(e.len(), 1);
        assert_eq!(e[0].0, "Box__a");
        assert_eq!(e[0].1["f"], "'box'");
        assert_eq!(e[0].1["min"], "[-1, 0, 2.5]");
        assert_eq!(e[0].1["z"], "'AAAA'");
    }

    #[test]
    fn fingerprints_differ_by_a_byte() {
        assert_ne!(fingerprint(b"abc"), fingerprint(b"abd"));
        assert_eq!(fingerprint(b""), "cbf29ce484222325");
    }
}
