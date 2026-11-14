//! `cargo xtask bench`: performance against budgets and against its own
//! history.
//!
//! ```text
//! cargo xtask bench [--no-record]
//! ```
//!
//! Runs `backrooms --bench` (build, first frame, then `FRAMES` frames along a
//! fixed path through each zone; GPU time from Metal's command buffers) on
//! `RUNS` × levels 1–6, `REPEATS` times, and takes each measure's median over
//! the repeats (a level over a budget gets a second look: `REPEATS` more
//! runs, and the median of all of them). It then
//!
//! - holds every level to the budgets in `docs/engineering/PERF.md` (a JSON block, like
//!   the style bands), and
//! - looks for a **change point** in each measure's history on this machine
//!   (`perf/history.csv`): the split of the series that best separates an
//!   earlier and a later mean, kept only when a permutation test says it is
//!   not chance (`P_CHANGE`) and the later mean is slower by more than
//!   `SLOWER`. One noisy run cannot fail it; a real regression shows as a
//!   lasting step (Daly et al.; Hunter).
//!
//! Without `--no-record` the medians are appended to the history (one row per
//! level), with the commit and the machine.
//!
//! Then the **creature pack** (`backrooms --bench-creatures`: on level 4,
//! two of every kind that roams the rooms, all hunting you) is timed the
//! same way, held to the `creatures` budgets and to its own history,
//! `perf/creatures.csv` (one row a run), and **recording and replaying**
//! (`backrooms --bench-replay`: a minute of the restless bot from the title,
//! recorded and replayed headless) to the `replays` budgets and
//! `perf/replays.csv`.
use crate::shots::{machine, read_json};
use crate::{cargo, flag, root, run};
use serde_json::Value;
use std::process::Command;

const RUNS: &[u32] = &[4242, 7];
const LEVELS: &str = "1,2,3,4,5,6";
const FRAMES: &str = "240";
const REPEATS: usize = 3;
/// Measures recorded (higher is worse for all of them).
const MEASURES: &[&str] =
    &["build_ms", "first_frame_ms", "update_p50_ms", "update_p99_ms", "gpu_p50_ms", "gpu_p99_ms", "calls_max", "tris_max"];
const P_CHANGE: f64 = 0.01;
/// A change point counts as a slowdown above this ratio of the means.
const SLOWER: f64 = 1.10;
/// Change points are searched only with at least this many runs on record.
const MIN_HISTORY: usize = 8;

pub fn bench(args: &[String]) -> Result<(), String> {
    run(cargo().args(["build", "--release", "-q", "-p", "backrooms"]))?;
    let bin = root().join("target/release/backrooms");
    let tmp = root().join("target/xtask/bench");
    std::fs::create_dir_all(&tmp).map_err(|e| e.to_string())?;
    // seed → level → measure → samples
    type Samples<'a> = std::collections::BTreeMap<(u32, i64), std::collections::BTreeMap<&'a str, Vec<f64>>>;
    let mut samples: Samples = Default::default();
    let measure = |samples: &mut Samples, seed: u32, levels: &str, tag: &str| -> Result<(), String> {
        let f = tmp.join(format!("b{seed}_{tag}.json"));
        let st = Command::new(&bin)
            .args(["--fresh", "--width", "1280", "--height", "800", "--scale", "1", "--bench"])
            .arg(&f)
            .args(["--seed", &seed.to_string(), "--levels", levels, "--bench-frames", FRAMES])
            .stdout(std::process::Stdio::null())
            .status()
            .map_err(|e| e.to_string())?;
        if !st.success() {
            return Err(format!("bench of seed {seed} failed"));
        }
        for r in read_json(&f).as_array().into_iter().flatten() {
            let key = (seed, r["level"].as_i64().unwrap_or(0));
            for m in MEASURES {
                samples.entry(key).or_default().entry(m).or_default().push(r[*m].as_f64().unwrap_or(0.0));
            }
        }
        Ok(())
    };
    for rep in 0..REPEATS {
        for &seed in RUNS {
            measure(&mut samples, seed, LEVELS, &rep.to_string())?;
        }
    }
    let median = |v: &[f64]| {
        let mut s = v.to_vec();
        s.sort_by(|a, b| a.total_cmp(b));
        s[s.len() / 2]
    };
    let medians = |samples: &Samples| -> Vec<((u32, i64), Vec<f64>)> {
        samples.iter().map(|(k, m)| (*k, MEASURES.iter().map(|x| median(&m[x])).collect())).collect()
    };
    let mut now = medians(&samples);

    // budgets
    let doc = std::fs::read_to_string(root().join("docs/engineering/PERF.md")).map_err(|e| format!("docs/engineering/PERF.md: {e}"))?;
    let budgets: Value = doc
        .split("<!-- perf-budgets:begin -->")
        .nth(1)
        .and_then(|r| r.split("```json").nth(1))
        .and_then(|r| r.split("```").next())
        .and_then(|j| serde_json::from_str(j).ok())
        .ok_or("docs/engineering/PERF.md: no perf-budgets block")?;
    let over = |v: &[f64]| MEASURES.iter().enumerate().any(|(i, m)| budgets["levels"][m].as_f64().is_some_and(|b| v[i] > b));
    // a second look: a level over a budget is measured `REPEATS` more times
    // and judged on the median of all its runs (a p99 of 240 frames is the
    // third-worst frame: one hitch of the machine's can carry it)
    for &seed in RUNS {
        let again: Vec<String> = now.iter().filter(|((s, _), v)| *s == seed && over(v)).map(|((_, l), _)| l.to_string()).collect();
        if again.is_empty() {
            continue;
        }
        println!("a second look at seed {seed} level(s) {}", again.join(","));
        for rep in 0..REPEATS {
            measure(&mut samples, seed, &again.join(","), &format!("again{rep}"))?;
        }
    }
    now = medians(&samples);
    let mut failures = vec![];
    println!("{:<10} {}", "level", MEASURES.join("  "));
    for ((seed, level), v) in &now {
        println!("{:<10} {}", format!("{seed} L{level}"), v.iter().map(|x| format!("{x:.2}")).collect::<Vec<_>>().join("  "));
        for (i, m) in MEASURES.iter().enumerate() {
            if let Some(b) = budgets["levels"][m].as_f64()
                && v[i] > b
            {
                failures.push(format!("{seed} L{level}: {m} {:.2} over its budget {b} (docs/engineering/PERF.md)", v[i]));
            }
        }
    }

    // history
    let hist_path = root().join("perf/history.csv");
    // the model identifier has a comma in it ("Mac16,11"): not in a CSV column
    let here = machine().replace(',', ";");
    let text = std::fs::read_to_string(&hist_path).unwrap_or_default();
    let header = format!("date,commit,machine,seed,level,{}", MEASURES.join(","));
    let mut series: std::collections::BTreeMap<(u32, i64, usize), Vec<(String, f64)>> = Default::default();
    for line in text.lines().skip(1) {
        let c: Vec<&str> = line.split(',').collect();
        if c.len() < 5 + MEASURES.len() || c[2] != here {
            continue;
        }
        let (Ok(seed), Ok(level)) = (c[3].parse::<u32>(), c[4].parse::<i64>()) else { continue };
        for i in 0..MEASURES.len() {
            if let Ok(v) = c[5 + i].parse::<f64>() {
                series.entry((seed, level, i)).or_default().push((c[1].to_string(), v));
            }
        }
    }
    let commit = Command::new("git")
        .current_dir(root())
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .map_or_default(|o| String::from_utf8_lossy(&o.stdout).trim().to_string());
    let dirty = Command::new("git").current_dir(root()).args(["status", "--porcelain"]).output().is_ok_and(|o| !o.stdout.is_empty());
    let commit = if dirty { format!("{commit}+") } else { commit };
    for ((seed, level), v) in &now {
        for (i, m) in MEASURES.iter().enumerate() {
            let mut s = series.get(&(*seed, *level, i)).cloned().unwrap_or_default();
            s.push((commit.clone(), v[i]));
            if s.len() < MIN_HISTORY {
                continue;
            }
            let xs: Vec<f64> = s.iter().map(|x| x.1).collect();
            if let Some((at, before, after, p)) = change_point(&xs)
                && after > before * SLOWER
                && p < P_CHANGE
                && at + 3 >= xs.len()
            {
                failures.push(format!(
                    "{seed} L{level}: {m} stepped from {before:.2} to {after:.2} at commit {} (p = {p:.3}); see perf/history.csv",
                    s[at].0
                ));
            }
        }
    }
    if !flag(args, "--no-record") {
        let mut out = if text.is_empty() { format!("{header}\n") } else { text };
        let date = Command::new("date").arg("+%Y-%m-%d").output().map_or_default(|o| String::from_utf8_lossy(&o.stdout).trim().to_string());
        for ((seed, level), v) in &now {
            out.push_str(&format!(
                "{date},{commit},{here},{seed},{level},{}\n",
                v.iter().map(|x| format!("{x:.3}")).collect::<Vec<_>>().join(",")
            ));
        }
        std::fs::create_dir_all(hist_path.parent().unwrap()).map_err(|e| e.to_string())?;
        std::fs::write(&hist_path, out).map_err(|e| e.to_string())?;
        println!("recorded in perf/history.csv");
    }
    bench_pack(args, &bin, &tmp, &budgets, &here, &commit, &mut failures)?;
    bench_replays(args, &bin, &tmp, &budgets, &here, &commit, &mut failures)?;
    if failures.is_empty() {
        println!("within budget, no slowdown in the history");
        Ok(())
    } else {
        for f in &failures {
            println!("FAIL  {f}");
        }
        Err(format!("{} performance check(s) failed", failures.len()))
    }
}

/// Measures of the creature pack (higher is worse).
const PACK_MEASURES: &[&str] = &["update_p50_ms", "update_p99_ms", "gpu_p50_ms", "gpu_p99_ms", "calls_max", "tris_max"];

/// The creature pack: `REPEATS` runs, medians, the `creatures` budgets, and
/// a change point in `perf/creatures.csv`.
fn bench_pack(
    args: &[String],
    bin: &std::path::Path,
    tmp: &std::path::Path,
    budgets: &Value,
    here: &str,
    commit: &str,
    failures: &mut Vec<String>,
) -> Result<(), String> {
    let mut samples: Vec<Vec<f64>> = vec![vec![]; PACK_MEASURES.len()];
    let mut creatures = 0;
    for rep in 0..REPEATS {
        let f = tmp.join(format!("pack_{rep}.json"));
        let st = Command::new(bin)
            .args(["--fresh", "--width", "1280", "--height", "800", "--scale", "1", "--bench-creatures"])
            .arg(&f)
            .args(["--seed", "4242", "--bench-frames", FRAMES])
            .stdout(std::process::Stdio::null())
            .status()
            .map_err(|e| e.to_string())?;
        if !st.success() {
            return Err("the creature pack bench failed".into());
        }
        let r = read_json(&f);
        creatures = r["creatures"].as_i64().unwrap_or(0);
        for (i, m) in PACK_MEASURES.iter().enumerate() {
            samples[i].push(r[*m].as_f64().unwrap_or(0.0));
        }
    }
    let series =
        Series { name: "creature pack", budgets: "creatures", csv: "perf/creatures.csv", key: ("creatures", creatures.to_string()) };
    series.hold(args, PACK_MEASURES, &medians_of(&samples), budgets, here, commit, failures)
}

/// Measures of recording and replaying (higher is worse): a replayed
/// frame's time, the log's work a frame, the log's memory and the file's
/// size a minute of play.
const REPLAY_MEASURES: &[&str] = &["replay_frame_us", "record_frame_us", "log_kb_min", "file_kb_min"];
/// The bot's frames a replay bench plays (a minute, after the way in).
const REPLAY_FRAMES: &str = "3600";

/// Recording and replaying (`backrooms --bench-replay`): `REPEATS` runs,
/// medians, the `replays` budgets, and a change point in
/// `perf/replays.csv`.
fn bench_replays(
    args: &[String],
    bin: &std::path::Path,
    tmp: &std::path::Path,
    budgets: &Value,
    here: &str,
    commit: &str,
    failures: &mut Vec<String>,
) -> Result<(), String> {
    let mut samples: Vec<Vec<f64>> = vec![vec![]; REPLAY_MEASURES.len()];
    for rep in 0..REPEATS {
        let f = tmp.join(format!("replay_{rep}.json"));
        let o = Command::new(bin)
            .arg("--bench-replay")
            .arg(&f)
            .args(["--seed", "4242", "--bench-frames", REPLAY_FRAMES])
            .output()
            .map_err(|e| e.to_string())?;
        if !o.status.success() {
            return Err(format!("the replay bench failed: {}", String::from_utf8_lossy(&o.stderr).trim()));
        }
        let r = read_json(&f);
        for (i, m) in REPLAY_MEASURES.iter().enumerate() {
            samples[i].push(r[*m].as_f64().unwrap_or(0.0));
        }
    }
    let series = Series { name: "replays", budgets: "replays", csv: "perf/replays.csv", key: ("frames", REPLAY_FRAMES.to_string()) };
    series.hold(args, REPLAY_MEASURES, &medians_of(&samples), budgets, here, commit, failures)
}

/// Each measure's median over its runs.
fn medians_of(samples: &[Vec<f64>]) -> Vec<f64> {
    samples
        .iter()
        .map(|v| {
            let mut s = v.clone();
            s.sort_by(|a, b| a.total_cmp(b));
            s[s.len() / 2]
        })
        .collect()
}

/// One series of measures with its own history (a CSV of `date, commit,
/// machine, <key>, measures…`, a row a run) and its own budgets.
struct Series {
    name: &'static str,
    /// the section of the budgets in `PERF.md`
    budgets: &'static str,
    csv: &'static str,
    /// a column that tells one measurement from another (another value is
    /// another series)
    key: (&'static str, String),
}

impl Series {
    /// `now` printed, held to the budgets and to a change point in the
    /// history on this machine, and (without `--no-record`) appended to it.
    #[expect(clippy::too_many_arguments, reason = "the bench's shared state, passed as the levels' bench passes it")]
    fn hold(
        &self,
        args: &[String],
        measures: &[&str],
        now: &[f64],
        budgets: &Value,
        here: &str,
        commit: &str,
        failures: &mut Vec<String>,
    ) -> Result<(), String> {
        println!("{:<10} {}", self.name, measures.join("  "));
        println!(
            "{:<10} {}",
            format!("{} {}", self.key.1, self.key.0),
            now.iter().map(|x| format!("{x:.2}")).collect::<Vec<_>>().join("  ")
        );
        for (i, m) in measures.iter().enumerate() {
            if let Some(b) = budgets[self.budgets][m].as_f64()
                && now[i] > b
            {
                failures.push(format!("{}: {m} {:.2} over its budget {b} (docs/engineering/PERF.md)", self.name, now[i]));
            }
        }
        let path = root().join(self.csv);
        let text = std::fs::read_to_string(&path).unwrap_or_default();
        let mut series: Vec<Vec<(String, f64)>> = vec![vec![]; measures.len()];
        for line in text.lines().skip(1) {
            let c: Vec<&str> = line.split(',').collect();
            if c.len() < 4 + measures.len() || c[2] != here || c[3] != self.key.1 {
                continue;
            }
            for i in 0..measures.len() {
                if let Ok(v) = c[4 + i].parse::<f64>() {
                    series[i].push((c[1].to_string(), v));
                }
            }
        }
        for (i, m) in measures.iter().enumerate() {
            let mut s = series[i].clone();
            s.push((commit.to_string(), now[i]));
            if s.len() < MIN_HISTORY {
                continue;
            }
            let xs: Vec<f64> = s.iter().map(|x| x.1).collect();
            if let Some((at, before, after, p)) = change_point(&xs)
                && after > before * SLOWER
                && p < P_CHANGE
                && at + 3 >= xs.len()
            {
                failures.push(format!(
                    "{}: {m} stepped from {before:.2} to {after:.2} at commit {} (p = {p:.3}); see {}",
                    self.name, s[at].0, self.csv
                ));
            }
        }
        if !flag(args, "--no-record") {
            let mut out = if text.is_empty() { format!("date,commit,machine,{},{}\n", self.key.0, measures.join(",")) } else { text };
            let date =
                Command::new("date").arg("+%Y-%m-%d").output().map_or_default(|o| String::from_utf8_lossy(&o.stdout).trim().to_string());
            out.push_str(&format!(
                "{date},{commit},{here},{},{}\n",
                self.key.1,
                now.iter().map(|x| format!("{x:.3}")).collect::<Vec<_>>().join(",")
            ));
            std::fs::write(&path, out).map_err(|e| e.to_string())?;
            println!("recorded in {}", self.csv);
        }
        Ok(())
    }
}

/// The best single split of `x` (each side at least 3 long): its index, the
/// two means, and a permutation p-value for the size of the difference.
fn change_point(x: &[f64]) -> Option<(usize, f64, f64, f64)> {
    let mean = |s: &[f64]| s.iter().sum::<f64>() / s.len() as f64;
    let stat = |x: &[f64]| -> Option<(usize, f64)> {
        (3..=x.len().saturating_sub(3))
            .map(|k| (k, (mean(&x[k..]) - mean(&x[..k])).abs() * ((k * (x.len() - k)) as f64 / x.len() as f64).sqrt()))
            .max_by(|a, b| a.1.total_cmp(&b.1))
    };
    let (at, s) = stat(x)?;
    let mut y = x.to_vec();
    let mut r: u64 = 0x2545_F491_4F6C_DD1D;
    let mut hits = 0;
    for _ in 0..199 {
        for i in (1..y.len()).rev() {
            r ^= r << 13;
            r ^= r >> 7;
            r ^= r << 17;
            y.swap(i, (r % (i as u64 + 1)) as usize);
        }
        if stat(&y).is_some_and(|(_, t)| t >= s) {
            hits += 1;
        }
    }
    Some((at, mean(&x[..at]), mean(&x[at..]), (hits as f64 + 1.0) / 200.0))
}

#[cfg(test)]
mod tests {
    use super::change_point;

    #[test]
    fn a_lasting_step_is_found_and_noise_is_not() {
        let step: Vec<f64> = (0..12).map(|i| if i < 7 { 2.0 + 0.01 * (i % 3) as f64 } else { 3.0 + 0.01 * (i % 2) as f64 }).collect();
        let (at, before, after, p) = change_point(&step).unwrap();
        assert_eq!(at, 7);
        assert!(after > before * 1.4 && p < 0.01);
        let noise: Vec<f64> = (0..12).map(|i| 2.0 + 0.05 * ((i * 7919) % 5) as f64).collect();
        let (_, _, _, p) = change_point(&noise).unwrap();
        assert!(p > 0.01, "noise is not a change point (p {p})");
    }
}
