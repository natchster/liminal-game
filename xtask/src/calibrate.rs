//! `cargo xtask calibrate`: how far a reviewer can be trusted, measured on
//! pairs whose answer is known.
//!
//! ```text
//! cargo xtask calibrate prepare [--mild]
//! cargo xtask calibrate score ANSWERS.json --reviewer NAME
//! ```
//!
//! `prepare` takes `SHOTS` from the goldens and makes, for each, a damaged
//! copy with one of the `KINDS` of regression the gates exist to catch —
//! cartoon colour, washed-out light, a hole where geometry is missing,
//! smeared detail — plus `CONTROLS` identical pairs (nothing to prefer). It
//! hands them to `cargo xtask ab prepare` (blind, both orders; the golden is
//! the *after* side), and writes the known answers to
//! `target/xtask/calibrate/truth.json`. `--mild` makes each damage about
//! half as strong: where a reviewer stops seeing a regression.
//!
//! `score` scores the answers with `ab score` and then against the truth:
//! accuracy and Cohen's kappa (agreement beyond chance, which raw agreement
//! overstates), by kind, appended to
//! `crates/backrooms/tests/review/calibration.jsonl`. Until people's verdicts
//! exist this is the reviewer's calibration; once they do, their verdicts are
//! the truth (roadmap 4.5).
use crate::shots::{read_json, read_png, write_json};
use crate::{opt, root};
use serde_json::{Value, json};
use std::io::Write;
use std::path::{Path, PathBuf};

/// Goldens to damage: level views of several purposes, and creatures.
const SHOTS: &[&str] = &[
    "4242_L1_well",
    "4242_L7_office_wing",
    "4242_L7_living_quarters",
    "4242_L7_storage_hall",
    "7_L5_play_area",
    "4242_L10_showroom",
    "smiler_4m",
    "hound_4m",
    "watcher_2m",
    "4242_L2_stair_halls",
];
const KINDS: &[&str] = &["cartoon", "washed", "hole", "smear"];
/// Identical pairs: the right answer is a tie.
const CONTROLS: &[&str] = &["4242_L3_works", "smiler_8m"];

fn dir() -> PathBuf {
    root().join("target/xtask/calibrate")
}

pub fn calibrate(args: &[String]) -> Result<(), String> {
    match args.first().map(String::as_str) {
        Some("prepare") => prepare(crate::flag(args, "--mild")),
        Some("score") => score(&args[1..]),
        _ => Err("usage: cargo xtask calibrate prepare | calibrate score ANSWERS.json --reviewer NAME".into()),
    }
}

fn prepare(mild: bool) -> Result<(), String> {
    let gold = root().join("crates/backrooms/tests/golden");
    let out = dir();
    let _ = std::fs::remove_dir_all(&out);
    let (before, after) = (out.join("before"), out.join("after"));
    std::fs::create_dir_all(&before).map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&after).map_err(|e| e.to_string())?;
    let mut truth = serde_json::Map::new();
    for (i, s) in SHOTS.iter().enumerate() {
        let kind = KINDS[i % KINDS.len()];
        let (w, h, px) = read_png(&gold.join(format!("{s}.png")))?;
        let name = format!("{s}__{kind}{}", if mild { "_mild" } else { "" });
        write_png(&before.join(format!("{name}.png")), w, h, &damage(kind, mild, w, h, &px))?;
        std::fs::copy(gold.join(format!("{s}.png")), after.join(format!("{name}.png"))).map_err(|e| e.to_string())?;
        truth.insert(name, json!({ "truth": "after", "kind": if mild { format!("{kind} (mild)") } else { kind.to_string() } }));
    }
    for s in CONTROLS {
        let name = format!("{s}__same");
        for d in [&before, &after] {
            std::fs::copy(gold.join(format!("{s}.png")), d.join(format!("{name}.png"))).map_err(|e| e.to_string())?;
        }
        truth.insert(name, json!({ "truth": "tie", "kind": "same" }));
    }
    write_json(&out.join("truth.json"), &Value::Object(truth))?;
    let args: Vec<String> = [
        "prepare",
        "--before",
        before.to_str().unwrap(),
        "--after",
        after.to_str().unwrap(),
        "--question",
        "Which side looks more like the game should look (docs/design/STYLE.md): a real, worn building or creature in weak light, with nothing broken? If neither is better, answer tie.",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    crate::ab::ab(&args)?;
    println!(
        "calibration set: {} damaged + {} identical; known answers in target/xtask/calibrate/truth.json (keep them from the reviewer)",
        SHOTS.len(),
        CONTROLS.len()
    );
    Ok(())
}

fn score(args: &[String]) -> Result<(), String> {
    let answers = args.first().ok_or("calibrate score ANSWERS.json --reviewer NAME")?;
    let reviewer = opt(args, "--reviewer").unwrap_or("agent").to_string();
    let ab_args: Vec<String> = vec!["score".into(), answers.clone(), "--reviewer".into(), format!("{reviewer} (calibration)")];
    crate::ab::ab(&ab_args)?;
    // the per-image verdicts just logged
    let log = std::fs::read_to_string(root().join("crates/backrooms/tests/review/verdicts.jsonl")).map_err(|e| e.to_string())?;
    let truth = read_json(&dir().join("truth.json"));
    let mut verdict: std::collections::BTreeMap<String, String> = Default::default();
    for l in log.lines().rev() {
        let v: Value = serde_json::from_str(l).unwrap_or(Value::Null);
        let img = v["image"].as_str().unwrap_or("").to_string();
        if truth.get(&img).is_some() && !verdict.contains_key(&img) && v["reviewer"].as_str() == Some(&format!("{reviewer} (calibration)"))
        {
            verdict.insert(img, v["verdict"].as_str().unwrap_or("tie").to_string());
        }
    }
    let cats = ["after", "before", "tie"];
    let mut pairs = vec![];
    let mut by_kind: std::collections::BTreeMap<String, (usize, usize)> = Default::default();
    for (img, t) in truth.as_object().into_iter().flatten() {
        let tv = t["truth"].as_str().unwrap_or("tie");
        let rv = verdict.get(img).map_or("tie", String::as_str);
        pairs.push((tv, rv));
        let e = by_kind.entry(t["kind"].as_str().unwrap_or("?").to_string()).or_default();
        e.0 += 1;
        e.1 += usize::from(tv == rv);
    }
    let n = pairs.len() as f64;
    let po = pairs.iter().filter(|(a, b)| a == b).count() as f64 / n;
    let pe: f64 = cats
        .iter()
        .map(|c| (pairs.iter().filter(|(a, _)| a == c).count() as f64 / n) * (pairs.iter().filter(|(_, b)| b == c).count() as f64 / n))
        .sum();
    let kappa = if (1.0 - pe).abs() < 1e-9 { 1.0 } else { (po - pe) / (1.0 - pe) };
    println!("{reviewer}: {:.0}% of {} known answers, Cohen's kappa {kappa:.2}", po * 100.0, pairs.len());
    for (k, (t, ok)) in &by_kind {
        println!("  {k:<8} {ok}/{t}");
    }
    let row = json!({
        "date": std::process::Command::new("date").arg("+%Y-%m-%d").output().map_or_default(|o| String::from_utf8_lossy(&o.stdout).trim().to_string()),
        "reviewer": reviewer, "n": pairs.len(), "accuracy": (po * 1000.0).round() / 1000.0, "kappa": (kappa * 1000.0).round() / 1000.0,
        "by_kind": by_kind.iter().map(|(k, (t, ok))| (k.clone(), json!([ok, t]))).collect::<serde_json::Map<String, Value>>(),
    });
    let p = root().join("crates/backrooms/tests/review/calibration.jsonl");
    let mut f = std::fs::OpenOptions::new().create(true).append(true).open(&p).map_err(|e| e.to_string())?;
    writeln!(f, "{row}").map_err(|e| e.to_string())?;
    println!("logged to crates/backrooms/tests/review/calibration.jsonl");
    Ok(())
}

/// A damaged copy of a frame: the regressions the gates exist for.
fn damage(kind: &str, mild: bool, w: u32, h: u32, px: &[u8]) -> Vec<u8> {
    let mut out = px.to_vec();
    let (w, h) = (w as usize, h as usize);
    let (sat, bright, contrast, lift, hole, blur) =
        if mild { (1.35, 1.05, 0.8, 8.0, 0.4, 1usize) } else { (1.9, 1.12, 0.55, 18.0, 1.0, 3usize) };
    match kind {
        // toy colour: saturation ×1.9 (×1.35 mild) around the pixel's grey, a little brighter
        "cartoon" => {
            for p in out.chunks_mut(4) {
                let l = 0.2126 * p[0] as f64 + 0.7152 * p[1] as f64 + 0.0722 * p[2] as f64;
                for c in p.iter_mut().take(3) {
                    *c = ((l + (*c as f64 - l) * sat) * bright).clamp(0.0, 255.0) as u8;
                }
            }
        }
        // flat, washed-out light: contrast ×0.55 (×0.8 mild) toward mid-grey, shadows lifted
        "washed" => {
            for p in out.chunks_mut(4) {
                for c in p.iter_mut().take(3) {
                    *c = (118.0 + (*c as f64 - 118.0) * contrast + lift).clamp(0.0, 255.0) as u8;
                }
            }
        }
        // missing geometry: a hard-edged black hole left of centre
        "hole" => {
            let (hw, hh) = ((w as f64 * 0.18 * hole) as usize, (h as f64 * 0.30 * hole) as usize);
            let (x0, y0) = (w * 30 / 100, h * 32 / 100);
            let (x1, y1) = (x0 + hw, y0 + hh);
            for y in y0..y1 {
                for x in x0..x1 {
                    out[(y * w + x) * 4..(y * w + x) * 4 + 3].copy_from_slice(&[0, 0, 0]);
                }
            }
        }
        // lost detail: a 7×7 box blur (3×3 mild)
        "smear" => {
            for y in 0..h {
                for x in 0..w {
                    let mut s = [0u32; 3];
                    let mut n = 0;
                    for yy in y.saturating_sub(blur)..(y + blur + 1).min(h) {
                        for xx in x.saturating_sub(blur)..(x + blur + 1).min(w) {
                            for c in 0..3 {
                                s[c] += px[(yy * w + xx) * 4 + c] as u32;
                            }
                            n += 1;
                        }
                    }
                    for c in 0..3 {
                        out[(y * w + x) * 4 + c] = (s[c] / n) as u8;
                    }
                }
            }
        }
        _ => {}
    }
    out
}

fn write_png(p: &Path, w: u32, h: u32, rgba: &[u8]) -> Result<(), String> {
    let f = std::fs::File::create(p).map_err(|e| e.to_string())?;
    let mut enc = png::Encoder::new(std::io::BufWriter::new(f), w, h);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    enc.write_header().and_then(|mut wr| wr.write_image_data(rgba)).map_err(|e| e.to_string())
}
