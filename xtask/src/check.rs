//! `cargo xtask check`: the quality gate, with a report short enough to read.
//!
//! Each step runs to completion with its full output written to
//! `target/xtask/check/<step>.log`; the report is one line per step plus one
//! `ERROR <step> …` line per problem found, so a failure can be found with
//! `grep ERROR` and nothing else floods the terminal (or an agent's context).
//!
//! The steps: `fmt`, `clippy`, `test`, `shots` (the golden frames and look
//! statistics, `cargo xtask shots`) and, in the full check, `range` (the
//! generator's expressive range, `cargo xtask range --look`) and `bench`
//! (budgets and the performance history, `cargo xtask bench --no-record`).
//!
//! `--fast` leaves out the slow suites that play whole levels in real time
//! (`FAST_SKIP`); they are filtered by name, so the report counts them as
//! skipped rather than passed.
use crate::{cargo, flag, opt, root};
use std::path::Path;
use std::process::Command;
use std::time::Instant;

/// At most this many `ERROR` lines per step; the log has the rest.
const MAX_ERRORS: usize = 20;

/// Tests that play levels for minutes of game time (about 2½ minutes of the
/// suite's run): the full check runs them, `--fast` does not.
const FAST_SKIP: &[&str] = &[
    "the_director_paces_a_level",
    "playthrough_from_new_game",
    "a_level_opens_quietly",
    "signatures_are_heard",
    "the_notice_is_loud",
    "the_hush_quiets_the_ambience",
    "the_door_is_heard",
    "her_trail_leads_through_her_camp_to_the_door",
    "her_marks_change_as_she_goes_down",
    "lit_surfaces_have_plausible_albedo",
    "the_mix_holds_its_loudness_and_varies",
    "generated_levels_are_walkable_and_repeatable",
    "a_replay_reproduces_the_run",
    "a_recording_survives_its_file_and_replays",
    "personas_play_and_the_fear_has_a_shape",
    "who_lives_where",
    "the_one_in_the_corner_hums",
    "the_reader_whispers",
    "the_rule_follower_gets_through_and_no_death_breaks_the_rules",
    "the_explorer_gets_through_some",
    "the_rusher_gets_through_some",
    "the_whole_run_live",
];

struct Step {
    name: &'static str,
    cmd: Command,
    /// Turns the step's output into one line per problem.
    errors: fn(&str) -> Vec<String>,
}

pub fn check(args: &[String]) -> Result<(), String> {
    let fast = flag(args, "--fast");
    let only: Option<Vec<&str>> = opt(args, "--only").map(|s| s.split(',').collect());
    let skip: Vec<&str> = opt(args, "--skip").map(|s| s.split(',').collect()).unwrap_or_default();
    let logs = root().join("target/xtask/check");
    std::fs::create_dir_all(&logs).map_err(|e| format!("{}: {e}", logs.display()))?;

    let mut steps = vec![];
    let mut fmt = cargo();
    fmt.args(["fmt", "--all", "--", "--check"]);
    steps.push(Step { name: "fmt", cmd: fmt, errors: fmt_errors });
    let mut clippy = cargo();
    clippy.args(["clippy", "--workspace", "--all-targets", "--release", "--", "-D", "warnings"]);
    steps.push(Step { name: "clippy", cmd: clippy, errors: rustc_errors });
    let mut test = cargo();
    test.args(["test", "--workspace", "--release", "--no-fail-fast"]);
    if fast {
        test.arg("--");
        for t in FAST_SKIP {
            test.args(["--skip", t]);
        }
    }
    steps.push(Step { name: "test", cmd: test, errors: test_errors });
    let mut shots = cargo();
    shots.args(["xtask", "shots"]);
    steps.push(Step { name: "shots", cmd: shots, errors: shot_errors });
    if !fast {
        let mut range = cargo();
        range.args(["xtask", "range", "--look"]);
        steps.push(Step { name: "range", cmd: range, errors: range_errors });
        let mut bench = cargo();
        bench.args(["xtask", "bench", "--no-record"]);
        steps.push(Step { name: "bench", cmd: bench, errors: range_errors });
    }

    let started = Instant::now();
    let mut failed = vec![];
    for mut s in steps {
        if only.as_ref().is_some_and(|o| !o.contains(&s.name)) || skip.contains(&s.name) {
            continue;
        }
        let t0 = Instant::now();
        // stdout and stderr into one file, interleaved as written (cargo
        // prints a test binary's name on stderr and its results on stdout)
        let log = logs.join(format!("{}.log", s.name));
        let file = std::fs::File::create(&log).map_err(|e| format!("{}: {e}", log.display()))?;
        let file2 = file.try_clone().map_err(|e| e.to_string())?;
        let status = s.cmd.stdout(file).stderr(file2).status().map_err(|e| format!("{}: {e}", s.name))?;
        let text = String::from_utf8_lossy(&std::fs::read(&log).unwrap_or_default()).into_owned();
        let secs = t0.elapsed().as_secs_f64();
        if status.success() {
            println!("ok    {:<8} {secs:6.1} s{}", s.name, summary(s.name, &text));
            continue;
        }
        println!("FAIL  {:<8} {secs:6.1} s  log: {}", s.name, rel(&log));
        let mut errs = (s.errors)(&text);
        if errs.is_empty() {
            errs.push(format!("exited with {status} (no recognisable error lines; read the log)"));
        }
        let n = errs.len();
        for e in errs.into_iter().take(MAX_ERRORS) {
            println!("ERROR {} {e}", s.name);
        }
        if n > MAX_ERRORS {
            println!("ERROR {} … {} more in the log", s.name, n - MAX_ERRORS);
        }
        failed.push(s.name);
    }
    let mode = if fast { "fast" } else { "full" };
    if failed.is_empty() {
        println!("check ({mode}) passed in {:.0} s", started.elapsed().as_secs_f64());
        Ok(())
    } else {
        Err(format!("check ({mode}) failed: {}", failed.join(", ")))
    }
}

fn rel(p: &Path) -> String {
    p.strip_prefix(root()).unwrap_or(p).display().to_string()
}

/// A short tail for a passing step (the test count).
fn summary(step: &str, text: &str) -> String {
    if step != "test" {
        return String::new();
    }
    let (mut passed, mut ignored, mut filtered) = (0u32, 0u32, 0u32);
    for l in text.lines().filter(|l| l.starts_with("test result:")) {
        let num = |key: &str| -> u32 {
            l.split(';').find(|p| p.contains(key)).and_then(|p| p.split_whitespace().filter_map(|t| t.parse().ok()).last()).unwrap_or(0)
        };
        passed += num(" passed");
        ignored += num(" ignored");
        filtered += num(" filtered out");
    }
    let skipped = if filtered > 0 { format!(", {filtered} skipped (fast)") } else { String::new() };
    format!("  {passed} passed, {ignored} ignored{skipped}")
}

/// `FAIL  …` lines of `cargo xtask range` and `cargo xtask bench`.
fn range_errors(text: &str) -> Vec<String> {
    text.lines().filter_map(|l| l.strip_prefix("FAIL  ")).map(String::from).collect()
}

/// `name  frame FAIL …` / `look  FAIL …` / `whole set: FAIL …` lines.
fn shot_errors(text: &str) -> Vec<String> {
    let mut out = vec![];
    let mut last = "";
    for l in text.lines() {
        let name = l.split_whitespace().next().unwrap_or("");
        if !l.starts_with(' ') && !name.is_empty() {
            last = name;
        }
        if let Some(i) = l.find("FAIL") {
            let what = if l.trim_start().starts_with("look") { "look" } else { "frame" };
            let who = if l.starts_with("whole set") { "whole set" } else { last };
            out.push(format!("{who} {what}: {}", l[i + 4..].trim()));
        }
    }
    if out.is_empty() {
        out.extend(text.lines().filter(|l| l.starts_with("error")).map(String::from));
    }
    out
}

/// `Diff in /abs/path.rs:12:` → `path.rs:12 not formatted (run cargo fmt)`.
fn fmt_errors(text: &str) -> Vec<String> {
    let root = root().display().to_string();
    text.lines()
        .filter_map(|l| l.strip_prefix("Diff in "))
        .map(|l| {
            let p = l.trim_end_matches(':').trim_start_matches(&root).trim_start_matches('/');
            format!("{p} not formatted (run `cargo fmt`)")
        })
        .collect()
}

/// rustc / clippy diagnostics: `error: msg` followed by ` --> file:line:col`.
fn rustc_errors(text: &str) -> Vec<String> {
    let mut out = vec![];
    let lines: Vec<&str> = text.lines().collect();
    for (i, l) in lines.iter().enumerate() {
        let msg = match l.strip_prefix("error") {
            Some(rest) if rest.starts_with(':') || rest.starts_with('[') => {
                rest.trim_start_matches(|c| c != ':').trim_start_matches(':').trim()
            }
            _ => continue,
        };
        if msg.starts_with("could not compile") || msg.starts_with("aborting due to") || msg.contains("warnings being treated as errors") {
            continue;
        }
        let at = lines[i + 1..].iter().take(4).find_map(|n| n.trim_start().strip_prefix("--> ")).unwrap_or("");
        out.push(if at.is_empty() { msg.to_string() } else { format!("{at} {msg}") });
    }
    out
}

/// Test failures: the failing test, where it panicked and why; compile
/// errors in test code come through `rustc_errors`.
fn test_errors(text: &str) -> Vec<String> {
    let mut out = rustc_errors(text);
    let lines: Vec<&str> = text.lines().collect();
    for (i, l) in lines.iter().enumerate() {
        // thread 'name' panicked at path:line:col:
        if let Some(rest) = l.strip_prefix("thread '") {
            if let Some((name, tail)) = rest.split_once("' panicked at ") {
                let at = tail.trim_end_matches(':');
                let why = lines.get(i + 1).map(|s| s.trim()).unwrap_or("");
                let why: String = why.chars().take(240).collect();
                out.push(format!("{name} panicked at {at}: {why}"));
            }
        }
        // error: test failed, to rerun pass `-p game --test entities`
        if let Some(rest) = l.strip_prefix("error: test failed, to rerun pass ") {
            out.push(format!("rerun with: cargo test --release {}", rest.trim_matches('`')));
        }
    }
    out.retain(|e| !e.starts_with("test failed"));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_failures_become_one_line_each() {
        let log = "running 2 tests\ntest a::b ... FAILED\n\nfailures:\n\n---- a::b stdout ----\n\nthread 'a::b' panicked at crates/game/tests/x.rs:10:5:\nassertion failed: left == right\nnote: run with `RUST_BACKTRACE=1`\n\nerror: test failed, to rerun pass `-p game --test x`\n";
        let e = test_errors(log);
        assert_eq!(
            e,
            vec![
                "a::b panicked at crates/game/tests/x.rs:10:5: assertion failed: left == right",
                "rerun with: cargo test --release -p game --test x"
            ]
        );
    }

    #[test]
    fn the_summary_counts_passed_ignored_and_skipped() {
        let log = "test result: ok. 3 passed; 0 failed; 1 ignored; 0 measured; 2 filtered out; finished in 0.1s\ntest result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.2s\n";
        assert_eq!(summary("test", log), "  7 passed, 1 ignored, 2 skipped (fast)");
    }

    #[test]
    fn clippy_errors_carry_their_location() {
        let log = "error: this expression creates a reference which is immediately dereferenced\n  --> crates/game/src/a.rs:3:9\n   |\nerror: could not compile `game` due to 1 previous error\n";
        assert_eq!(
            rustc_errors(log),
            vec!["crates/game/src/a.rs:3:9 this expression creates a reference which is immediately dereferenced"]
        );
    }
}
