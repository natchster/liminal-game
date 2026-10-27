//! `cargo xtask shots`: golden frames and look statistics.
//!
//! Renders a fixed shot list headless with `--fresh` (no saved settings, a
//! fixed per-frame seed: the same frame bit for bit on the same machine):
//!
//! - level views: the spawn view of a few `(seed, level)` pairs and one zone
//!   view for every zone purpose the earlier levels did not show yet, so
//!   between them they cover every purpose, weather and anomaly;
//! - each creature in front of the camera at 2, 4 and 8 m, and its death scene.
//!
//! Each frame is compared with its golden in `crates/backrooms/tests/golden/`:
//! identical pixels pass; otherwise ꟻLIP (the `nv-flip` crate) measures the
//! perceived difference — the mean, the 99th percentile and the worst 32×32
//! tile, since a missing wall is a small, local change a mean hides — and
//! anything above `TINY` fails with an error map next to the frame in
//! `target/xtask/shots/`. A missing golden fails too.
//!
//! Every frame is also held to the art bible: `docs/STYLE.md` ends with a
//! JSON block of ceilings for all frames and a colourfulness / luminance band
//! for each zone purpose, which the purpose's zone view must stay inside.
//!
//! The look statistics of every frame (colourfulness, mean luminance,
//! contrast, deep-shadow share) are held to *anchors* in `look.json`, which
//! move only with `--accept-look`: re-accepting a frame never moves its look
//! anchor, so a slow slide back to bright, saturated frames still fails.
//!
//! ```text
//! cargo xtask shots                         render and compare
//! cargo xtask shots --list                  print the shot names
//! cargo xtask shots --accept NAME,…|all --reason "…"       new goldens
//! cargo xtask shots --accept-look NAME,…|all --reason "…"  new look anchors
//! ```
//!
//! Goldens are bound to the machine that recorded them (model and macOS
//! version in `shots.json`); elsewhere the comparison is advisory.
use crate::look::{look_of, Look};
use crate::{cargo, flag, opt, root, run};
use serde_json::{json, Map, Value};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

pub const WIDTH: u32 = 480;
pub const HEIGHT: u32 = 300;

/// Level views: these levels, in this order; each adds the purposes not seen yet.
/// (The Building's three floors first; then the rooms, from the Lobby on
/// level 4; 4242's Showroom is on level 10, 7's Borrowed House on level 5.)
const LEVELS: &[(u32, i32)] = &[(4242, 1), (4242, 2), (4242, 3), (4242, 7), (7, 5), (4242, 4), (7, 6), (4242, 10)];
const CREATURES: &[&str] = &["smiler", "hound", "watcher"];
const DISTANCES: &[u32] = &[2, 4, 8];

/// A difference below all of these is reported but passes: below what a
/// person notices in a side-by-side flip.
const TINY: Flip = Flip { mean: 0.002, p99: 0.05, tile: 0.02 };

/// How far one frame's look may move from its anchor before it fails.
const LOOK_BANDS: Look = Look { colourfulness: 3.0, luminance: 0.02, contrast: 0.02, shadow: 0.04 };
/// How far the *median* shift over all frames may go: a change of light or
/// grade moves every frame a little, far inside the per-frame bands (putting
/// the ambient back from 0.62 to 0.78 raised the median luminance by 0.009).
const SET_BANDS: Look = Look { colourfulness: 0.4, luminance: 0.005, contrast: 0.005, shadow: 0.004 };

#[derive(Clone, Copy, Debug, Default)]
struct Flip {
    mean: f64,
    p99: f64,
    tile: f64,
}

struct Shot {
    name: String,
    path: PathBuf,
}

fn golden_dir() -> PathBuf {
    root().join("crates/backrooms/tests/golden")
}
fn out_dir() -> PathBuf {
    root().join("target/xtask/shots")
}

pub fn shots(args: &[String]) -> Result<(), String> {
    let accept = opt(args, "--accept");
    let accept_look = opt(args, "--accept-look");
    let reason = opt(args, "--reason");
    if (accept.is_some() || accept_look.is_some()) && reason.is_none_or(|r| r.trim().is_empty()) {
        return Err("--accept / --accept-look need --reason \"why this change is intended\"".into());
    }
    let gdir = golden_dir();
    std::fs::create_dir_all(&gdir).map_err(|e| e.to_string())?;
    let mut manifest = read_json(&gdir.join("shots.json"));
    let mut anchors = read_json(&gdir.join("look.json"));

    let shots = if accept.is_some() || accept_look.is_some() { existing_renders()? } else { render_all()? };
    if flag(args, "--list") {
        for s in &shots {
            println!("{}", s.name);
        }
        return Ok(());
    }
    let pick = |list: Option<&str>, name: &str| list.is_some_and(|l| l == "all" || l.split(',').any(|n| n == name));

    let here = machine();
    let recorded_on = manifest["machine"].as_str().unwrap_or("").to_string();
    let binding = recorded_on.is_empty() || recorded_on == here;
    if !binding {
        println!("note: goldens were recorded on {recorded_on}; this is {here}, so differences are advisory");
    }

    let mut failures = 0;
    let mut accepted = vec![];
    let mut anchored = vec![];
    let mut table = BTreeMap::new();
    let mut shifts: Vec<(Look, Look)> = vec![];
    let style = style_bands()?;
    for s in &shots {
        let (w, h, px) = read_png(&s.path)?;
        let look = look_of(w, h, &px);
        let golden = gdir.join(format!("{}.png", s.name));
        if pick(accept, &s.name) {
            std::fs::copy(&s.path, &golden).map_err(|e| e.to_string())?;
            manifest["shots"][&s.name] = json!({ "size": [w, h] });
            accepted.push(s.name.clone());
        }
        if pick(accept_look, &s.name) {
            anchors[&s.name] = look.to_json();
            anchored.push(s.name.clone());
        }
        // ---- the frame
        let verdict = if !golden.exists() {
            failures += 1;
            "FAIL  no golden (record one with --accept NAME --reason …)".to_string()
        } else {
            let (gw, gh, gpx) = read_png(&golden)?;
            if (gw, gh) != (w, h) {
                failures += 1;
                format!("FAIL  size {w}×{h}, golden {gw}×{gh}")
            } else if gpx == px {
                "ok".to_string()
            } else {
                let f = flip(w, h, &gpx, &px, &out_dir().join(format!("{}-diff.png", s.name)))?;
                let msg = format!("mean {:.4}  p99 {:.3}  worst tile {:.3}", f.mean, f.p99, f.tile);
                if f.mean <= TINY.mean && f.p99 <= TINY.p99 && f.tile <= TINY.tile {
                    format!("ok    tiny change ({msg})")
                } else {
                    failures += usize::from(binding);
                    format!("FAIL  {msg}; error map: target/xtask/shots/{}-diff.png", s.name)
                }
            }
        };
        // ---- its look
        let look_verdict = match Look::from_json(&anchors[&s.name]) {
            None => {
                failures += 1;
                "FAIL  no look anchor (set one with --accept-look NAME --reason …)".to_string()
            }
            Some(a) => {
                shifts.push((look, a));
                let off = look.outside(&a, &LOOK_BANDS);
                if off.is_empty() {
                    "ok".into()
                } else {
                    failures += usize::from(binding);
                    format!("FAIL  {}", off.join(", "))
                }
            }
        };
        // ---- the art bible
        let off = style_check(&style, &s.name, &look);
        let look_verdict = if off.is_empty() {
            look_verdict
        } else {
            failures += usize::from(binding);
            let style = format!("FAIL  outside docs/STYLE.md: {}", off.join(", "));
            if look_verdict.starts_with("ok") {
                style
            } else {
                format!("{look_verdict}; {style}")
            }
        };
        table.insert(s.name.clone(), (verdict, look_verdict, look));
    }
    let name_w = table.keys().map(|k| k.len()).max().unwrap_or(10);
    for (name, (v, lv, look)) in &table {
        let line = format!("{name:<name_w$}  frame {v}");
        println!("{line}");
        if !lv.starts_with("ok") {
            println!("{:<name_w$}  look  {lv}", "");
        } else if std::env::var_os("SHOTS_VERBOSE").is_some() {
            println!("{:<name_w$}  look  {}", "", look.describe());
        }
    }
    // ---- the whole set
    if shifts.len() >= 5 && accept_look.is_none() {
        let median = |f: fn(&Look) -> f64| {
            let mut v: Vec<f64> = shifts.iter().map(|(l, a)| f(l) - f(a)).collect();
            v.sort_by(|a, b| a.total_cmp(b));
            v[v.len() / 2]
        };
        let shift = Look {
            colourfulness: median(|l| l.colourfulness),
            luminance: median(|l| l.luminance),
            contrast: median(|l| l.contrast),
            shadow: median(|l| l.shadow),
        };
        let zero = Look { colourfulness: 0.0, luminance: 0.0, contrast: 0.0, shadow: 0.0 };
        let off = shift.outside(&zero, &SET_BANDS);
        if off.is_empty() {
            println!("whole set: median look shift within bands ({})", shift.describe_shift());
        } else {
            failures += usize::from(binding);
            println!("whole set: FAIL  median shift {} — every frame moved the same way", off.join(", ").replace(" vs anchor 0.000", ""));
        }
    }
    if !accepted.is_empty() || !anchored.is_empty() {
        manifest["machine"] = json!(here);
        manifest["size"] = json!([WIDTH, HEIGHT]);
        write_json(&gdir.join("shots.json"), &manifest)?;
        write_json(&gdir.join("look.json"), &anchors)?;
        log_acceptance(&accepted, &anchored, reason.unwrap_or(""))?;
        println!(
            "accepted {} frame(s) and {} look anchor(s); review `git diff --stat crates/backrooms/tests/golden` and commit with the reason",
            accepted.len(),
            anchored.len()
        );
        return Ok(());
    }
    if failures > 0 {
        return Err(format!("{failures} shot check(s) failed"));
    }
    println!("{} shots match their goldens and look anchors", shots.len());
    Ok(())
}

/// The bands at the end of `docs/STYLE.md`.
fn style_bands() -> Result<Value, String> {
    let text = std::fs::read_to_string(root().join("docs/STYLE.md")).map_err(|e| format!("docs/STYLE.md: {e}"))?;
    let (Some(a), Some(b)) = (text.find("<!-- style-bands:begin -->"), text.find("<!-- style-bands:end -->")) else {
        return Err("docs/STYLE.md has no style-bands block".into());
    };
    let block = &text[a..b];
    let json = block
        .split("```json")
        .nth(1)
        .and_then(|r| r.split("```").next())
        .ok_or("docs/STYLE.md: the style-bands block needs a ```json fence")?;
    serde_json::from_str(json).map_err(|e| format!("docs/STYLE.md style bands: {e}"))
}

/// What a frame breaks of the art bible: the ceiling for all frames, the one
/// for level views, and for a zone view (`<seed>_L<level>_<purpose>`) its
/// purpose's band.
fn style_check(style: &Value, name: &str, look: &Look) -> Vec<String> {
    let mut out = vec![];
    let all = &style["all"];
    if let Some(c) = all["colourfulness_max"].as_f64() {
        if look.colourfulness > c {
            out.push(format!("colourfulness {:.1} > {c} (all frames)", look.colourfulness));
        }
    }
    let purpose = name.split_once("_L").and_then(|(_, rest)| rest.split_once('_')).map(|(_, p)| p.replace('_', " "));
    if let Some(l) = style["levels"]["luminance_max"].as_f64().filter(|_| purpose.is_some()) {
        if look.luminance > l {
            out.push(format!("luminance {:.3} > {l} (level views)", look.luminance));
        }
    }
    if let Some(p) = purpose.filter(|p| p != "spawn") {
        let band = &style["purposes"][&p];
        if band.is_null() {
            out.push(format!("no band for the purpose \"{p}\" (add one)"));
        }
        if let Some(c) = band["colourfulness_max"].as_f64() {
            if look.colourfulness > c {
                out.push(format!("colourfulness {:.1} > {c} ({p})", look.colourfulness));
            }
        }
        if let (Some(lo), Some(hi)) = (band["luminance"][0].as_f64(), band["luminance"][1].as_f64()) {
            if look.luminance < lo || look.luminance > hi {
                out.push(format!("luminance {:.3} outside {lo}–{hi} ({p})", look.luminance));
            }
        }
    }
    out
}

/// Builds the game and renders every shot into `target/xtask/shots/`.
fn render_all() -> Result<Vec<Shot>, String> {
    run(cargo().args(["build", "--release", "-q", "-p", "backrooms"]))?;
    let bin = root().join("target/release/backrooms");
    let out = out_dir();
    let _ = std::fs::remove_dir_all(&out);
    std::fs::create_dir_all(&out).map_err(|e| e.to_string())?;
    let size = |c: &mut Command| {
        c.args(["--fresh", "--width", &WIDTH.to_string(), "--height", &HEIGHT.to_string(), "--scale", "1"]);
    };
    // the creatures: one process each, a few at a time
    let mut jobs: Vec<(String, Command)> = vec![];
    for kind in CREATURES {
        for d in DISTANCES {
            let mut c = Command::new(&bin);
            size(&mut c);
            let name = format!("{kind}_{d}m");
            c.args(["--seed", "4242", "--level", "4", "--play", "--frames", "90", "--showcase", &format!("{kind}:{d}")]);
            c.arg("--screenshot").arg(out.join(format!("{name}.png")));
            jobs.push((name, c));
        }
        let mut c = Command::new(&bin);
        size(&mut c);
        let name = format!("{kind}_death");
        c.args(["--seed", "4242", "--level", "4", "--play", "--frames", "90", "--showcase", &format!("{kind}:2"), "--death"]);
        c.arg("--screenshot").arg(out.join(format!("{name}.png")));
        jobs.push((name, c));
    }
    // the levels: every zone view, then the selection
    let raw = out.join("levels");
    for &(seed, level) in LEVELS {
        let mut c = Command::new(&bin);
        size(&mut c);
        c.arg("--dump-level").arg(&raw).args(["--seed", &seed.to_string(), "--levels", &level.to_string(), "--screenshot", "x", "--views"]);
        jobs.push((format!("{seed}_L{level}"), c));
    }
    run_parallel(jobs, 4)?;

    let mut shots: Vec<Shot> = vec![];
    for kind in CREATURES {
        for name in DISTANCES.iter().map(|d| format!("{kind}_{d}m")).chain([format!("{kind}_death")]) {
            shots.push(Shot { path: out.join(format!("{name}.png")), name });
        }
    }
    let mut seen = std::collections::BTreeSet::new();
    for &(seed, level) in LEVELS {
        let base = raw.join(format!("rs_{seed}_L{level}"));
        let spawn = out.join(format!("{seed}_L{level}_spawn.png"));
        std::fs::rename(base.with_extension("png"), &spawn).map_err(|e| format!("{seed} L{level}: {e}"))?;
        shots.push(Shot { name: format!("{seed}_L{level}_spawn"), path: spawn });
        let dump = read_json(&base.with_extension("json"));
        for p in dump["purposes"].as_array().into_iter().flatten() {
            let label = p["label"].as_str().unwrap_or("?").to_string();
            if !seen.insert(label.clone()) {
                continue;
            }
            let z = p["zone"].as_u64().unwrap_or(0);
            let name = format!("{seed}_L{level}_{}", label.replace(' ', "_"));
            let src = raw.join(format!("rs_{seed}_L{level}_z{z:02}.png"));
            let dst = out.join(format!("{name}.png"));
            if std::fs::rename(&src, &dst).is_ok() {
                shots.push(Shot { name, path: dst });
            }
        }
    }
    std::fs::write(out.join("list.txt"), shots.iter().map(|s| s.name.clone()).collect::<Vec<_>>().join("\n")).map_err(|e| e.to_string())?;
    Ok(shots)
}

/// The renders of the last run (to accept without rendering again).
fn existing_renders() -> Result<Vec<Shot>, String> {
    let list = std::fs::read_to_string(out_dir().join("list.txt"))
        .map_err(|_| "no renders to accept: run `cargo xtask shots` first".to_string())?;
    Ok(list.lines().map(|n| Shot { name: n.to_string(), path: out_dir().join(format!("{n}.png")) }).collect())
}

fn run_parallel(jobs: Vec<(String, Command)>, at_once: usize) -> Result<(), String> {
    let mut pending = jobs.into_iter();
    let mut running: Vec<(String, std::process::Child)> = vec![];
    loop {
        while running.len() < at_once {
            let Some((name, mut c)) = pending.next() else { break };
            let child = c.stdout(std::process::Stdio::null()).spawn().map_err(|e| format!("{name}: {e}"))?;
            running.push((name, child));
        }
        if running.is_empty() {
            return Ok(());
        }
        let (name, mut child) = running.remove(0);
        let st = child.wait().map_err(|e| format!("{name}: {e}"))?;
        if !st.success() {
            return Err(format!("rendering {name} failed ({st})"));
        }
    }
}

fn flip(w: u32, h: u32, golden: &[u8], actual: &[u8], diff_png: &Path) -> Result<Flip, String> {
    let rgb = |px: &[u8]| px.chunks(4).flat_map(|p| [p[0], p[1], p[2]]).collect::<Vec<u8>>();
    let r = nv_flip::FlipImageRgb8::with_data(w, h, &rgb(golden));
    let t = nv_flip::FlipImageRgb8::with_data(w, h, &rgb(actual));
    let err = nv_flip::flip(r, t, nv_flip::DEFAULT_PIXELS_PER_DEGREE);
    let mut pool = nv_flip::FlipPool::from_image(&err);
    let values = err.to_vec();
    // the worst 32×32 tile (mean error inside it)
    let mut tile = 0.0f64;
    for ty in (0..h).step_by(32) {
        for tx in (0..w).step_by(32) {
            let (mut s, mut n) = (0.0, 0.0);
            for y in ty..(ty + 32).min(h) {
                for x in tx..(tx + 32).min(w) {
                    s += values[(y * w + x) as usize] as f64;
                    n += 1.0;
                }
            }
            tile = tile.max(s / n);
        }
    }
    let map = err.apply_color_lut(&nv_flip::magma_lut()).to_vec();
    let rgba: Vec<u8> = map.chunks(3).flat_map(|p| [p[0], p[1], p[2], 255]).collect();
    write_png(diff_png, w, h, &rgba)?;
    Ok(Flip { mean: pool.mean() as f64, p99: pool.get_percentile(0.99, true) as f64, tile })
}

pub fn read_png(p: &Path) -> Result<(u32, u32, Vec<u8>), String> {
    let f = std::fs::File::open(p).map_err(|e| format!("{}: {e}", p.display()))?;
    let mut dec = png::Decoder::new(f);
    dec.set_transformations(png::Transformations::EXPAND);
    let mut r = dec.read_info().map_err(|e| format!("{}: {e}", p.display()))?;
    let mut buf = vec![0; r.output_buffer_size()];
    let info = r.next_frame(&mut buf).map_err(|e| format!("{}: {e}", p.display()))?;
    buf.truncate(info.buffer_size());
    let px = match info.color_type {
        png::ColorType::Rgba => buf,
        png::ColorType::Rgb => buf.chunks(3).flat_map(|c| [c[0], c[1], c[2], 255]).collect(),
        other => return Err(format!("{}: unsupported colour type {other:?}", p.display())),
    };
    Ok((info.width, info.height, px))
}

fn write_png(p: &Path, w: u32, h: u32, rgba: &[u8]) -> Result<(), String> {
    let f = std::fs::File::create(p).map_err(|e| format!("{}: {e}", p.display()))?;
    let mut enc = png::Encoder::new(std::io::BufWriter::new(f), w, h);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    let mut wr = enc.write_header().map_err(|e| e.to_string())?;
    wr.write_image_data(rgba).map_err(|e| e.to_string())
}

pub fn read_json(p: &Path) -> Value {
    std::fs::read_to_string(p).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_else(|| Value::Object(Map::new()))
}

pub fn write_json(p: &Path, v: &Value) -> Result<(), String> {
    let s = serde_json::to_string_pretty(v).map_err(|e| e.to_string())?;
    std::fs::write(p, s + "\n").map_err(|e| format!("{}: {e}", p.display()))
}

/// The Mac model and macOS version: goldens are exact only on the same pair.
pub fn machine() -> String {
    let out = |cmd: &str, args: &[&str]| {
        Command::new(cmd).args(args).output().map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string()).unwrap_or_default()
    };
    format!("{} macOS {}", out("sysctl", &["-n", "hw.model"]), out("sw_vers", &["-productVersion"]))
}

fn log_acceptance(frames: &[String], looks: &[String], reason: &str) -> Result<(), String> {
    let p = golden_dir().join("ACCEPTED.md");
    let mut s = std::fs::read_to_string(&p).unwrap_or_else(|_| {
        "# Accepted goldens\n\nEvery change to a golden frame or a look anchor, newest last. Written by\n`cargo xtask shots --accept / --accept-look`; the reason is the author's.\n".into()
    });
    let date =
        Command::new("date").arg("+%Y-%m-%d").output().map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string()).unwrap_or_default();
    let head = Command::new("git")
        .current_dir(root())
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();
    let list = |v: &[String]| if v.len() > 6 { format!("{} shots", v.len()) } else { v.join(", ") };
    s.push_str(&format!("\n- {date}, on top of {head}: "));
    if !frames.is_empty() {
        s.push_str(&format!("frames: {}. ", list(frames)));
    }
    if !looks.is_empty() {
        s.push_str(&format!("look anchors: {}. ", list(looks)));
    }
    s.push_str(&format!("Reason: {}\n", reason.trim()));
    std::fs::write(&p, s).map_err(|e| e.to_string())
}
