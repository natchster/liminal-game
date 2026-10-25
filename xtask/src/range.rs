//! `cargo xtask range`: the expressive-range report (Smith and Whitehead;
//! Summerville): does the generator still make the spread of levels it made,
//! neither drifting nor collapsing into sameness?
//!
//! Builds a fixed set of levels (`SEEDS` × levels 1–6, as runs build them)
//! headless with `backrooms --metrics`, measures each on properties other
//! than the generator's inputs (`game::debug::hooks::level_metrics`: loops,
//! dead ends, depth, integration, rooms, sightlines, light, props, themes,
//! darkness, grime), and compares the set with the accepted baseline in
//! `crates/backrooms/tests/recorded/range.json`:
//!
//! - **drift**: the energy distance between the two sets (each measure
//!   standardised by the baseline), with a permutation test; it fails when
//!   the sets differ beyond chance (`P_DRIFT`);
//! - **sameness / chaos**: a measure whose spread shrank below half, or grew
//!   past twice, the baseline's.
//!
//! It writes `target/xtask/range/report.md`, scatter plots of three pairs of
//! measures (baseline grey, now yellow) and a MAP-Elites style **seed
//! archive** (`archive.json`): a grid over depth × light density holding, in
//! each cell, the level with the longest views. Empty cells are levels the
//! generator does not make; one crowded cell is sameness.
//!
//! `--look` also renders every zone view of `LOOK_SEEDS` and reports each
//! zone purpose's look (colourfulness, luminance) against its band in
//! `docs/STYLE.md`, failing a purpose with enough views of which fewer than
//! `BAND_HIT` lie inside: the band no longer describes the purpose.
//!
//! ```text
//! cargo xtask range [--look]
//! cargo xtask range --accept --reason "…"     the current set becomes the baseline
//! ```
use crate::look::look_of;
use crate::shots::{read_json, read_png, write_json};
use crate::{cargo, flag, opt, root, run};
use serde_json::{json, Value};
use std::path::PathBuf;
use std::process::Command;

const SEEDS: &[u32] = &[4242, 7, 1999, 77, 9001, 31337, 2024, 555, 123456, 8080, 271828, 314159];
const LEVELS: &str = "1,2,3,4,5,6";
/// The measures compared (in the report's order).
const MEASURES: &[&str] = &[
    "loops",
    "dead_ends",
    "depth_mean",
    "integration",
    "room_area",
    "sight_mean",
    "light_density",
    "prop_density",
    "themes_per_zone",
    "dark_share",
    "grime",
];
/// The two sets differ beyond chance below this p-value.
const P_DRIFT: f64 = 0.01;
/// A measure's spread (standard deviation) relative to the baseline's.
const SPREAD: (f64, f64) = (0.5, 2.0);
const PLOTS: &[(&str, &str)] = &[("loops", "dead_ends"), ("depth_mean", "sight_mean"), ("light_density", "prop_density")];
/// The archive's grid: these two measures, this many bins each.
const ARCHIVE: (&str, &str, usize) = ("depth_mean", "light_density", 6);
const LOOK_SEEDS: &[u32] = &[4242, 7, 1999, 77];
/// A purpose needs this many views before its band is judged (fewer, and
/// `docs/STYLE.md` gives it the band of all views)…
const BAND_MIN_VIEWS: usize = 12;
/// … and this share of them inside the band.
const BAND_HIT: f64 = 0.7;

fn out_dir() -> PathBuf {
    root().join("target/xtask/range")
}
fn baseline_path() -> PathBuf {
    root().join("crates/backrooms/tests/recorded/range.json")
}

pub fn range(args: &[String]) -> Result<(), String> {
    let accept = flag(args, "--accept");
    let reason = opt(args, "--reason");
    if accept && reason.is_none_or(|r| r.trim().is_empty()) {
        return Err("--accept needs --reason \"why the generator's range is meant to change\"".into());
    }
    run(cargo().args(["build", "--release", "-q", "-p", "backrooms"]))?;
    let out = out_dir();
    let _ = std::fs::remove_dir_all(&out);
    std::fs::create_dir_all(&out).map_err(|e| e.to_string())?;
    let rows = measure_all(&out)?;
    println!("measured {} levels ({} seeds × levels {LEVELS})", rows.len(), SEEDS.len());

    if accept {
        write_json(&baseline_path(), &json!({ "seeds": SEEDS, "levels": LEVELS, "rows": rows }))?;
        log(reason.unwrap_or(""))?;
        println!("wrote {}; commit it with the reason", baseline_path().display());
        return Ok(());
    }
    let base = read_json(&baseline_path());
    let base_rows: Vec<Value> = base["rows"].as_array().cloned().unwrap_or_default();
    let mut failures = vec![];
    let mut report = String::from("# Expressive range\n\n");
    if base_rows.is_empty() {
        failures.push("no baseline: record one with `cargo xtask range --accept --reason …`".to_string());
    } else {
        let a = matrix(&base_rows);
        let b = matrix(&rows);
        // standardise by the baseline
        let (mean, sd) = stats(&a);
        let z = |m: &[Vec<f64>]| {
            m.iter().map(|r| r.iter().enumerate().map(|(i, v)| (v - mean[i]) / sd[i].max(1e-9)).collect()).collect::<Vec<Vec<f64>>>()
        };
        let (za, zb) = (z(&a), z(&b));
        let e = energy(&za, &zb);
        let p = permutation_p(&za, &zb, e, 999);
        report.push_str(&format!(
            "{} levels now, {} in the baseline. Energy distance {e:.4}, permutation p = {p:.3}.\n\n",
            rows.len(),
            base_rows.len()
        ));
        report.push_str("| Measure | Baseline mean ± sd | Now mean ± sd | Spread now / baseline |\n|---|---|---|---|\n");
        let (bm, bsd) = stats(&b);
        for (i, name) in MEASURES.iter().enumerate() {
            let ratio = if sd[i] > 1e-9 { bsd[i] / sd[i] } else { 1.0 };
            report.push_str(&format!("| {name} | {:.4} ± {:.4} | {:.4} ± {:.4} | {ratio:.2} |\n", mean[i], sd[i], bm[i], bsd[i]));
            if sd[i] > 1e-9 && ratio < SPREAD.0 {
                failures.push(format!("{name}: spread fell to {ratio:.2} of the baseline (sameness)"));
            }
            if sd[i] > 1e-9 && ratio > SPREAD.1 {
                failures.push(format!("{name}: spread grew to {ratio:.2} of the baseline (chaos)"));
            }
        }
        if p < P_DRIFT {
            failures.push(format!(
                "the levels drifted from the baseline (energy distance {e:.4}, p = {p:.3}); see target/xtask/range/report.md"
            ));
        }
        println!("energy distance {e:.4}, p = {p:.3}");
        for (x, y) in PLOTS {
            let f = out.join(format!("{x}-{y}.png"));
            scatter(&base_rows, &rows, x, y, &f)?;
            report.push_str(&format!("\n![{x} × {y}]({x}-{y}.png)\n"));
        }
    }
    // the seed archive
    let (filled, cells, crowd) = archive(&rows, base_rows.as_slice(), &out.join("archive.json"))?;
    report.push_str(&format!(
        "\n## Seed archive\n\n{filled} of {cells} cells of {} × {} filled; the most crowded cell holds {:.0}% of the levels (`archive.json`).\n",
        ARCHIVE.0,
        ARCHIVE.1,
        crowd * 100.0
    ));
    println!("seed archive: {filled}/{cells} cells, most crowded {:.0}%", crowd * 100.0);
    if flag(args, "--look") {
        let (text, bad) = look_by_purpose(&out)?;
        report.push_str(&text);
        failures.extend(bad);
    }
    std::fs::write(out.join("report.md"), &report).map_err(|e| e.to_string())?;
    println!("report: target/xtask/range/report.md");
    if failures.is_empty() {
        println!("the generator's range holds");
        Ok(())
    } else {
        for f in &failures {
            println!("FAIL  {f}");
        }
        Err(format!("{} range check(s) failed", failures.len()))
    }
}

/// Every level's measures, one `backrooms --metrics` process per seed.
fn measure_all(out: &std::path::Path) -> Result<Vec<Value>, String> {
    let bin = root().join("target/release/backrooms");
    let mut pending: Vec<(u32, std::process::Child)> = vec![];
    let mut rows = vec![];
    let mut todo = SEEDS.iter();
    loop {
        while pending.len() < 4 {
            let Some(&seed) = todo.next() else { break };
            let child = Command::new(&bin)
                .args(["--fresh", "--metrics"])
                .arg(out.join(format!("m{seed}.json")))
                .args(["--seed", &seed.to_string(), "--levels", LEVELS])
                .stdout(std::process::Stdio::null())
                .spawn()
                .map_err(|e| e.to_string())?;
            pending.push((seed, child));
        }
        if pending.is_empty() {
            break;
        }
        let (seed, mut c) = pending.remove(0);
        if !c.wait().map_err(|e| e.to_string())?.success() {
            return Err(format!("measuring seed {seed} failed"));
        }
    }
    for &seed in SEEDS {
        let v = read_json(&out.join(format!("m{seed}.json")));
        rows.extend(v.as_array().cloned().unwrap_or_default());
    }
    Ok(rows)
}

fn matrix(rows: &[Value]) -> Vec<Vec<f64>> {
    rows.iter().map(|r| MEASURES.iter().map(|m| r[m].as_f64().unwrap_or(0.0)).collect()).collect()
}

fn stats(m: &[Vec<f64>]) -> (Vec<f64>, Vec<f64>) {
    let n = m.len().max(1) as f64;
    let k = MEASURES.len();
    let mean: Vec<f64> = (0..k).map(|i| m.iter().map(|r| r[i]).sum::<f64>() / n).collect();
    let sd: Vec<f64> = (0..k).map(|i| (m.iter().map(|r| (r[i] - mean[i]).powi(2)).sum::<f64>() / n).sqrt()).collect();
    (mean, sd)
}

fn dist(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b).map(|(x, y)| (x - y) * (x - y)).sum::<f64>().sqrt()
}

/// Energy distance: 2·E|X−Y| − E|X−X′| − E|Y−Y′|.
fn energy(a: &[Vec<f64>], b: &[Vec<f64>]) -> f64 {
    let mean = |x: &[Vec<f64>], y: &[Vec<f64>]| {
        let mut s = 0.0;
        for p in x {
            for q in y {
                s += dist(p, q);
            }
        }
        s / (x.len() * y.len()).max(1) as f64
    };
    2.0 * mean(a, b) - mean(a, a) - mean(b, b)
}

/// The share of random relabellings of the pooled levels whose energy
/// distance is at least `e` (a fixed generator, so the report repeats).
fn permutation_p(a: &[Vec<f64>], b: &[Vec<f64>], e: f64, n: usize) -> f64 {
    if e <= 1e-12 {
        return 1.0;
    }
    let pool: Vec<Vec<f64>> = a.iter().chain(b).cloned().collect();
    let mut s: u64 = 0x9E37_79B9_7F4A_7C15;
    let mut next = || {
        s ^= s << 13;
        s ^= s >> 7;
        s ^= s << 17;
        s
    };
    let mut hits = 0;
    let mut idx: Vec<usize> = (0..pool.len()).collect();
    for _ in 0..n {
        for i in (1..idx.len()).rev() {
            let j = (next() % (i as u64 + 1)) as usize;
            idx.swap(i, j);
        }
        let x: Vec<Vec<f64>> = idx[..a.len()].iter().map(|&i| pool[i].clone()).collect();
        let y: Vec<Vec<f64>> = idx[a.len()..].iter().map(|&i| pool[i].clone()).collect();
        if energy(&x, &y) >= e {
            hits += 1;
        }
    }
    (hits as f64 + 1.0) / (n as f64 + 1.0)
}

/// A 400×400 scatter plot: the baseline grey, now yellow.
fn scatter(base: &[Value], now: &[Value], x: &str, y: &str, path: &std::path::Path) -> Result<(), String> {
    let (w, h, pad) = (400usize, 400usize, 24usize);
    let mut px = vec![0u8; w * h * 4];
    for p in px.chunks_mut(4) {
        p.copy_from_slice(&[18, 18, 18, 255]);
    }
    let all: Vec<(f64, f64)> = base.iter().chain(now).map(|r| (r[x].as_f64().unwrap_or(0.0), r[y].as_f64().unwrap_or(0.0))).collect();
    let (x0, x1) = all.iter().fold((f64::MAX, f64::MIN), |(a, b), p| (a.min(p.0), b.max(p.0)));
    let (y0, y1) = all.iter().fold((f64::MAX, f64::MIN), |(a, b), p| (a.min(p.1), b.max(p.1)));
    let mut dot = |vx: f64, vy: f64, c: [u8; 4]| {
        let fx = if x1 > x0 { (vx - x0) / (x1 - x0) } else { 0.5 };
        let fy = if y1 > y0 { (vy - y0) / (y1 - y0) } else { 0.5 };
        let cx = pad + (fx * (w - 2 * pad) as f64) as usize;
        let cy = h - pad - (fy * (h - 2 * pad) as f64) as usize;
        for dy in 0..3 {
            for dx in 0..3 {
                let (qx, qy) = (cx + dx - 1, cy + dy - 1);
                if qx < w && qy < h {
                    px[(qy * w + qx) * 4..(qy * w + qx) * 4 + 4].copy_from_slice(&c);
                }
            }
        }
    };
    for r in base {
        dot(r[x].as_f64().unwrap_or(0.0), r[y].as_f64().unwrap_or(0.0), [120, 120, 120, 255]);
    }
    for r in now {
        dot(r[x].as_f64().unwrap_or(0.0), r[y].as_f64().unwrap_or(0.0), [235, 200, 60, 255]);
    }
    for i in pad..w - pad {
        for &yy in &[h - pad, pad] {
            px[(yy * w + i) * 4..(yy * w + i) * 4 + 4].copy_from_slice(&[70, 70, 70, 255]);
        }
    }
    let f = std::fs::File::create(path).map_err(|e| e.to_string())?;
    let mut enc = png::Encoder::new(std::io::BufWriter::new(f), w as u32, h as u32);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    enc.write_header().and_then(|mut wr| wr.write_image_data(&px)).map_err(|e| e.to_string())
}

/// MAP-Elites style: per cell of the archive grid, the level with the longest
/// mean sightline. Returns (filled cells, cells, share of the most crowded).
fn archive(rows: &[Value], base: &[Value], path: &std::path::Path) -> Result<(usize, usize, f64), String> {
    let (xa, ya, n) = ARCHIVE;
    let span = |m: &str| {
        let src = if base.is_empty() { rows } else { base };
        let v: Vec<f64> = src.iter().filter_map(|r| r[m].as_f64()).collect();
        (v.iter().cloned().fold(f64::MAX, f64::min), v.iter().cloned().fold(f64::MIN, f64::max))
    };
    let ((x0, x1), (y0, y1)) = (span(xa), span(ya));
    let bin = |v: f64, lo: f64, hi: f64| {
        if hi > lo {
            (((v - lo) / (hi - lo)) * n as f64).floor().clamp(0.0, n as f64 - 1.0) as usize
        } else {
            0
        }
    };
    let mut cells: std::collections::BTreeMap<(usize, usize), (f64, Value, usize)> = Default::default();
    for r in rows {
        let key = (bin(r[xa].as_f64().unwrap_or(0.0), x0, x1), bin(r[ya].as_f64().unwrap_or(0.0), y0, y1));
        let fit = r["sight_mean"].as_f64().unwrap_or(0.0);
        let e = cells.entry(key).or_insert((f64::MIN, Value::Null, 0));
        e.2 += 1;
        if fit > e.0 {
            e.0 = fit;
            e.1 = json!({ "seed": r["seed"], "level": r["level"], "sight_mean": fit });
        }
    }
    let crowd = cells.values().map(|c| c.2).max().unwrap_or(0) as f64 / rows.len().max(1) as f64;
    let list: Vec<Value> =
        cells.iter().map(|((i, j), (_, best, count))| json!({ "cell": [i, j], "levels": count, "best": best })).collect();
    write_json(path, &json!({ "axes": [xa, ya], "bins": n, "cells": list }))?;
    Ok((cells.len(), n * n, crowd))
}

/// Every zone view of `LOOK_SEEDS`, grouped by zone purpose and held to the
/// purpose's band in `docs/STYLE.md`.
fn look_by_purpose(out: &std::path::Path) -> Result<(String, Vec<String>), String> {
    let bin = root().join("target/release/backrooms");
    let dir = out.join("views");
    let mut kids = vec![];
    for &seed in LOOK_SEEDS {
        kids.push(
            Command::new(&bin)
                .args(["--fresh", "--width", "480", "--height", "300", "--scale", "1", "--dump-level"])
                .arg(dir.join(seed.to_string()))
                .args(["--seed", &seed.to_string(), "--levels", LEVELS, "--screenshot", "x", "--views"])
                .stdout(std::process::Stdio::null())
                .spawn()
                .map_err(|e| e.to_string())?,
        );
    }
    for mut k in kids {
        if !k.wait().map_err(|e| e.to_string())?.success() {
            return Err("rendering the zone views failed".into());
        }
    }
    let style = std::fs::read_to_string(root().join("docs/STYLE.md")).unwrap_or_default();
    let bands: Value =
        style.split("```json").nth(1).and_then(|r| r.split("```").next()).and_then(|j| serde_json::from_str(j).ok()).unwrap_or(Value::Null);
    let mut by: std::collections::BTreeMap<String, Vec<(f64, f64)>> = Default::default();
    for &seed in LOOK_SEEDS {
        for level in LEVELS.split(',') {
            let base = dir.join(seed.to_string()).join(format!("rs_{seed}_L{level}"));
            let dump = read_json(&base.with_extension("json"));
            for p in dump["purposes"].as_array().into_iter().flatten() {
                let z = p["zone"].as_u64().unwrap_or(0);
                let img = dir.join(seed.to_string()).join(format!("rs_{seed}_L{level}_z{z:02}.png"));
                if let Ok((w, h, px)) = read_png(&img) {
                    let l = look_of(w, h, &px);
                    by.entry(p["label"].as_str().unwrap_or("?").to_string()).or_default().push((l.colourfulness, l.luminance));
                }
            }
        }
    }
    write_json(
        &out.join("look_by_purpose.json"),
        &json!(by.iter().map(|(k, v)| (k.clone(), json!(v))).collect::<serde_json::Map<String, Value>>()),
    )?;
    let q = |v: &mut Vec<f64>, f: f64| {
        v.sort_by(|a, b| a.total_cmp(b));
        v[((v.len() - 1) as f64 * f).round() as usize]
    };
    let mut text = String::from("\n## Look by zone purpose\n\n| Purpose | Views | Colourfulness p50 / p95 | Luminance p5 / p50 / p95 | Inside the STYLE band |\n|---|---|---|---|---|\n");
    let mut bad = vec![];
    for (label, v) in &by {
        let mut c: Vec<f64> = v.iter().map(|x| x.0).collect();
        let mut l: Vec<f64> = v.iter().map(|x| x.1).collect();
        let band = &bands["purposes"][label];
        let inside = v
            .iter()
            .filter(|(cc, ll)| {
                band["colourfulness_max"].as_f64().is_none_or(|m| *cc <= m)
                    && band["luminance"][0].as_f64().is_none_or(|lo| *ll >= lo)
                    && band["luminance"][1].as_f64().is_none_or(|hi| *ll <= hi)
            })
            .count();
        let share = inside as f64 / v.len() as f64;
        text.push_str(&format!(
            "| {label} | {} | {:.1} / {:.1} | {:.3} / {:.3} / {:.3} | {:.0}% |\n",
            v.len(),
            q(&mut c, 0.5),
            q(&mut c, 0.95),
            q(&mut l, 0.05),
            q(&mut l, 0.5),
            q(&mut l, 0.95),
            share * 100.0
        ));
        if band.is_null() {
            bad.push(format!("zone purpose \"{label}\" has no band in docs/STYLE.md"));
        } else if v.len() >= BAND_MIN_VIEWS && share < BAND_HIT {
            bad.push(format!("only {:.0}% of {} \"{label}\" views lie inside its STYLE band", share * 100.0, v.len()));
        }
    }
    print!("{text}");
    Ok((text, bad))
}

fn log(reason: &str) -> Result<(), String> {
    let p = root().join("crates/backrooms/tests/recorded/ACCEPTED.md");
    let mut s = std::fs::read_to_string(&p)
        .unwrap_or_else(|_| "# Accepted recordings\n\nEvery new baseline in this folder, newest last, with the author's reason.\n".into());
    let date =
        Command::new("date").arg("+%Y-%m-%d").output().map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string()).unwrap_or_default();
    let head = Command::new("git")
        .current_dir(root())
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default();
    s.push_str(&format!("\n- {date}, on top of {head}: range.json. Reason: {}\n", reason.trim()));
    std::fs::write(&p, s).map_err(|e| e.to_string())
}
