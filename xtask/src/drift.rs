//! `cargo xtask drift`: the drift sweep — every gate, read together, written
//! where people and the next session will see it.
//!
//! ```text
//! cargo xtask drift [--skip-check]
//! ```
//!
//! Runs the full `cargo xtask check` (or, with `--skip-check`, reads the logs
//! of the last one in `target/xtask/check/`), then:
//!
//! - writes each content entry's pass state into `docs/plan/content.json`
//!   (`passes`, `last_checked`) from the test, shot and range results;
//! - rewrites the *Latest sweep* section of `docs/plan/QUALITY.md` (between the
//!   `drift:begin` / `drift:end` markers): every step, the shots, the range,
//!   the performance, the files nearing the size cap, the content that is
//!   failing, and how many regressions people caught that no gate did (the
//!   steering loop's count, from the table in `QUALITY.md`);
//! - writes the full report to `target/xtask/drift/report.md`.
//!
//! It never changes a grade: grades move only with a person's reason in
//! `QUALITY.md`'s history (the `drift-sweep` skill says how to read it).
use crate::shots::{read_json, write_json};
use crate::{cargo, flag, root};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::process::Command;

/// The cap in `tests/structure.rs`.
const MAX_LINES: usize = 2400;

pub fn drift(args: &[String]) -> Result<(), String> {
    let logs = root().join("target/xtask/check");
    let mut check_summary = String::new();
    if !flag(args, "--skip-check") {
        let out = cargo().args(["xtask", "check"]).output().map_err(|e| e.to_string())?;
        check_summary = String::from_utf8_lossy(&out.stdout).into_owned();
    }
    let read = |name: &str| std::fs::read_to_string(logs.join(format!("{name}.log"))).unwrap_or_default();
    let (test, shots, range, bench) = (read("test"), read("shots"), read("range"), read("bench"));
    if test.is_empty() {
        return Err("no check logs in target/xtask/check/: run without --skip-check".into());
    }

    // ---- test files: tests/NAME.rs → ok?
    let mut file_ok: BTreeMap<String, bool> = BTreeMap::new();
    let mut placers_ok = true;
    let mut current: Option<String> = None;
    for l in test.lines() {
        let l = l.trim();
        if let Some(rest) = l.strip_prefix("Running ") {
            current = rest.split_whitespace().next().map(String::from);
            if rest.contains("deps/placers_") {
                current = Some("placers".into());
            }
        } else if let Some(r) = l.strip_prefix("test result: ") {
            let ok = r.starts_with("ok");
            match current.as_deref() {
                Some("placers") => placers_ok &= ok,
                Some(f) if f.starts_with("tests/") => {
                    let name = f.trim_start_matches("tests/").trim_end_matches(".rs").to_string();
                    let e = file_ok.entry(name).or_insert(true);
                    *e &= ok;
                }
                _ => {}
            }
            current = None;
        }
    }
    // ---- shots: name → ok?
    let mut shot_ok: BTreeMap<String, bool> = BTreeMap::new();
    let mut last = String::new();
    for l in shots.lines() {
        if !l.starts_with(' ') {
            if let Some(name) = l.split_whitespace().next() {
                last = name.to_string();
                if l.contains(" frame ") {
                    shot_ok.insert(last.clone(), !l.contains("FAIL"));
                }
            }
        } else if l.contains("FAIL") {
            shot_ok.insert(last.clone(), false);
        }
    }
    let range_fails: Vec<&str> = range.lines().filter(|l| l.starts_with("FAIL")).collect();
    let bench_fails: Vec<&str> = bench.lines().filter(|l| l.starts_with("FAIL")).collect();

    // ---- the ledger
    let date =
        Command::new("date").arg("+%Y-%m-%d").output().map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string()).unwrap_or_default();
    let ledger_path = root().join("docs/plan/content.json");
    let mut ledger = read_json(&ledger_path);
    let mut failing = vec![];
    if let Some(entries) = ledger["entries"].as_array_mut() {
        for e in entries {
            let mut passes = serde_json::Map::new();
            for c in e["checks"].as_array().cloned().unwrap_or_default().iter().filter_map(|c| c.as_str()) {
                let ok = match c.split_once(':') {
                    Some(("shots", n)) => shot_ok.get(n).copied(),
                    Some(("style", p)) => Some(
                        !range_fails.iter().any(|f| f.contains(&format!("\"{p}\"")))
                            && !shots.lines().any(|l| l.contains("STYLE") && l.contains(&format!("({p})"))),
                    ),
                    Some(("test", t)) => t.split_once('/').and_then(|(_, f)| file_ok.get(f).copied()),
                    Some(("lint", "placer-smoke")) => Some(placers_ok),
                    _ => None,
                };
                if let Some(ok) = ok {
                    passes.insert(c.to_string(), json!(ok));
                    if !ok {
                        failing.push(format!("{}: {c}", e["id"].as_str().unwrap_or("?")));
                    }
                }
            }
            e["passes"] = Value::Object(passes);
            e["last_checked"] = json!(date);
        }
    }
    write_json(&ledger_path, &ledger)?;

    // ---- files nearing the cap
    let mut sizes: Vec<(usize, String)> = vec![];
    let mut stack = vec![root().join("crates"), root().join("xtask")];
    while let Some(d) = stack.pop() {
        for e in std::fs::read_dir(&d).into_iter().flatten().flatten() {
            let p = e.path();
            if p.is_dir() {
                if p.file_name().is_some_and(|n| n != "target") {
                    stack.push(p);
                }
            } else if p.extension().is_some_and(|x| x == "rs") {
                let n = std::fs::read_to_string(&p).map(|s| s.lines().count()).unwrap_or(0);
                sizes.push((n, p.strip_prefix(root()).unwrap_or(&p).display().to_string()));
            }
        }
    }
    sizes.sort_by(|a, b| b.0.cmp(&a.0));
    let big: Vec<String> =
        sizes.iter().take(5).map(|(n, f)| format!("`{f}` {n} ({:.0}% of the cap)", *n as f64 / MAX_LINES as f64 * 100.0)).collect();

    // ---- the steering loop: regressions people caught that no gate did
    let quality_path = root().join("docs/plan/QUALITY.md");
    let quality = std::fs::read_to_string(&quality_path).map_err(|e| e.to_string())?;
    let caught = quality
        .split("## Caught by people")
        .nth(1)
        .map(|s| s.split("\n## ").next().unwrap_or(""))
        .map(|s| s.lines().filter(|l| l.starts_with("| 20")).count())
        .unwrap_or(0);

    // ---- the summary
    let step_lines: Vec<&str> = check_summary.lines().filter(|l| l.starts_with("ok ") || l.starts_with("FAIL")).collect();
    let steps = if step_lines.is_empty() { "(from the last check's logs)".to_string() } else { step_lines.join("; ").replace('|', "/") };
    let range_line = range.lines().find(|l| l.starts_with("energy distance")).unwrap_or("not run").to_string();
    let archive_line = range.lines().find(|l| l.starts_with("seed archive")).unwrap_or("").to_string();
    let shots_bad: Vec<&String> = shot_ok.iter().filter(|(_, ok)| !**ok).map(|(n, _)| n).collect();
    let mut sweep = format!("Last sweep: {date}.\n\n| Gate | Result |\n|---|---|\n");
    sweep.push_str(&format!("| Check steps | {steps} |\n"));
    sweep.push_str(&format!(
        "| Tests | {} of {} test files pass{} |\n",
        file_ok.values().filter(|v| **v).count(),
        file_ok.len(),
        if placers_ok { "; placer smoke tests pass" } else { "; **placer smoke tests fail**" }
    ));
    sweep.push_str(&format!(
        "| Golden frames | {} of {} match{} |\n",
        shot_ok.values().filter(|v| **v).count(),
        shot_ok.len(),
        if shots_bad.is_empty() { String::new() } else { format!("; failing: {shots_bad:?}") }
    ));
    sweep.push_str(&format!(
        "| Expressive range | {range_line}; {archive_line}{} |\n",
        if range_fails.is_empty() { String::new() } else { format!("; {}", range_fails.join("; ")) }
    ));
    sweep.push_str(&format!(
        "| Performance | {} |\n",
        if bench_fails.is_empty() { "within budget, no step in the history".to_string() } else { bench_fails.join("; ") }
    ));
    sweep.push_str(&format!("| Largest files | {} |\n", big.join(", ")));
    sweep
        .push_str(&format!("| Content failing a check | {} |\n", if failing.is_empty() { "none".to_string() } else { failing.join(", ") }));
    sweep.push_str(&format!("| Regressions people caught that no gate did | {caught} (see *Caught by people*) |\n"));
    let (a, b) = ("<!-- drift:begin -->", "<!-- drift:end -->");
    let new_quality = match (quality.find(a), quality.find(b)) {
        (Some(i), Some(j)) => format!("{}{a}\n{sweep}{}", &quality[..i], &quality[j..]),
        _ => quality.replacen("## Subsystems", &format!("## Latest sweep\n\n{a}\n{sweep}{b}\n\n## Subsystems"), 1),
    };
    std::fs::write(&quality_path, new_quality).map_err(|e| e.to_string())?;
    let out = root().join("target/xtask/drift");
    std::fs::create_dir_all(&out).map_err(|e| e.to_string())?;
    let report = format!("# Drift sweep {date}\n\n{sweep}\n## Check output\n\n```\n{check_summary}\n```\n\n## Range\n\n```\n{range}\n```\n\n## Performance\n\n```\n{bench}\n```\n");
    std::fs::write(out.join("report.md"), &report).map_err(|e| e.to_string())?;
    print!("{sweep}");
    println!("\nwrote docs/plan/content.json (passes), docs/plan/QUALITY.md (Latest sweep) and target/xtask/drift/report.md");
    Ok(())
}

/// `cargo xtask nightly`: the drift sweep (with the full check), a long level
/// sweep, and a recorded bench run — the jobs too slow for every change.
pub fn nightly(_args: &[String]) -> Result<(), String> {
    let mut failed = vec![];
    if let Err(e) = drift(&[]) {
        failed.push(format!("drift sweep: {e}"));
    }
    let sweep = cargo()
        .env("SWEEP_CASES", "200")
        .args(["test", "--release", "-q", "-p", "backrooms", "--test", "level_sweep", "generated"])
        .status()
        .map_err(|e| e.to_string())?;
    if !sweep.success() {
        failed.push(
            "the 200-case level sweep failed (its failing seeds are in crates/backrooms/tests/level_sweep.proptest-regressions)".into(),
        );
    }
    if !cargo().args(["xtask", "bench"]).status().map_err(|e| e.to_string())?.success() {
        failed.push("bench (recorded in perf/history.csv)".into());
    }
    if failed.is_empty() {
        println!("nightly: all passed; commit docs/plan/content.json, docs/plan/QUALITY.md and perf/history.csv");
        Ok(())
    } else {
        Err(failed.join("; "))
    }
}
