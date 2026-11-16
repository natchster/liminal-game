//! The test step of `cargo xtask check`: every test of the workspace run as
//! a process of its own, on a pool as wide as the machine, the longest first.
//!
//! `cargo test` runs one test binary after another, and most of ours hold one
//! to four tests that play levels for a minute or two, so it kept one or two
//! cores busy for most of its twenty minutes. Here every test is a job: each
//! binary is built once (`cargo test --no-run`), asked for its tests
//! (`--list`), and each test run alone (`NAME --exact`) in its crate's
//! directory, as cargo runs it. The last run's times
//! (`target/xtask/check/test-times.tsv`) order the queue, a test not timed
//! yet first; the whole suite then takes about the longest test or its
//! total time over the cores, whichever is more.
use crate::{cargo, root};
use std::collections::{HashMap, HashSet, VecDeque};
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::sync::{Arc, Mutex};
use std::time::Instant;

struct Binary {
    pkg: String,
    target: String,
    kind: String,
    exe: PathBuf,
    dir: PathBuf,
}

struct Job {
    bin: usize,
    name: String,
    key: String,
    est: f64,
}

/// What the pool did
pub struct Outcome {
    pub built: bool,
    pub passed: usize,
    pub failed: usize,
    pub ignored: usize,
    pub skipped: usize,
    pub workers: usize,
}

/// The profile the tests are built and run with (`Cargo.toml`: release
/// without link-time optimisation; the same bits, a tenth of the relinking)
pub const PROFILE: &str = "fasttest";

/// Build, list and run every test of the workspace, the names `skip` matches
/// (as `--skip` does: a part of the name) left out; everything said goes to
/// `log`
pub fn run(skip: &[&str], log: &mut std::fs::File) -> Result<Outcome, String> {
    let mut out = Outcome { built: false, passed: 0, failed: 0, ignored: 0, skipped: 0, workers: 0 };
    let built = cargo()
        .args(["test", "--workspace", "--profile", PROFILE, "--no-run", "--message-format=json-render-diagnostics"])
        .output()
        .map_err(|e| format!("cargo test --no-run: {e}"))?;
    log.write_all(&built.stderr).map_err(|e| e.to_string())?;
    if !built.status.success() {
        return Ok(out);
    }
    out.built = true;
    let bins = binaries(&String::from_utf8_lossy(&built.stdout));

    let times_path = root().join("target/xtask/check/test-times.tsv");
    let times: HashMap<String, f64> = std::fs::read_to_string(&times_path)
        .unwrap_or_default()
        .lines()
        .filter_map(|l| l.split_once('\t').and_then(|(k, v)| v.parse().ok().map(|v| (k.to_string(), v))))
        .collect();
    // every binary asked for its tests at once: one just built is checked by
    // the system at its first launch (0.8 s for one of ours, alone), and
    // the 90 of them one after another had taken a minute or more
    let listed: Vec<Result<(Vec<String>, Vec<String>), String>> = std::thread::scope(|s| {
        let asked: Vec<_> = bins.iter().map(|b| s.spawn(move || Ok((list(b, true)?, list(b, false)?)))).collect();
        asked.into_iter().map(|h| h.join().unwrap_or_else(|_| Err("a listing thread panicked".to_string()))).collect()
    });
    let mut jobs = vec![];
    // every test there is (skipped or not): the times kept are theirs
    let mut known = HashSet::new();
    for (bi, (b, l)) in bins.iter().zip(listed).enumerate() {
        let (ignored, names) = l?;
        out.ignored += ignored.len();
        for name in names {
            known.insert(format!("{}::{}::{name}", b.pkg, b.target));
            if ignored.contains(&name) {
                continue;
            }
            if skip.iter().any(|s| name.contains(s)) {
                out.skipped += 1;
                continue;
            }
            let key = format!("{}::{}::{name}", b.pkg, b.target);
            // a test not timed yet goes first: it may be a long one
            let est = times.get(&key).copied().unwrap_or(f64::INFINITY);
            jobs.push(Job { bin: bi, name, key, est });
        }
    }
    jobs.sort_by(|a, b| b.est.total_cmp(&a.est));

    let workers = std::thread::available_parallelism().map_or(8, |n| n.get());
    out.workers = workers;
    let queue = Arc::new(Mutex::new(VecDeque::from(jobs)));
    let bins = Arc::new(bins);
    let shared_log = Arc::new(Mutex::new(log.try_clone().map_err(|e| e.to_string())?));
    let results = Arc::new(Mutex::new(vec![]));
    let handles: Vec<_> = (0..workers)
        .map(|_| {
            let (queue, bins, shared_log, results) = (queue.clone(), bins.clone(), shared_log.clone(), results.clone());
            std::thread::spawn(move || {
                loop {
                    let Some(job) = queue.lock().unwrap().pop_front() else { break };
                    let b = &bins[job.bin];
                    let t0 = Instant::now();
                    let res = Command::new(&b.exe)
                        .args([job.name.as_str(), "--exact", "--test-threads=1"])
                        .current_dir(&b.dir)
                        .env("CARGO_MANIFEST_DIR", &b.dir)
                        .env("CARGO_PKG_NAME", &b.pkg)
                        .output();
                    let secs = t0.elapsed().as_secs_f64();
                    let (ok, text) = match res {
                        Ok(o) => {
                            (o.status.success(), format!("{}{}", String::from_utf8_lossy(&o.stdout), String::from_utf8_lossy(&o.stderr)))
                        }
                        Err(e) => (false, format!("could not run {}: {e}\n", b.exe.display())),
                    };
                    let mut s = format!("     Running {} {} ({secs:.1} s)\n{text}", b.target, job.name);
                    if !ok {
                        let which = match b.kind.as_str() {
                            "lib" => "--lib".to_string(),
                            "bin" => format!("--bin {}", b.target),
                            _ => format!("--test {}", b.target),
                        };
                        s += &format!("error: test failed, to rerun pass `-p {} {which} -- {} --exact`\n", b.pkg, job.name);
                    }
                    let _ = shared_log.lock().unwrap().write_all(s.as_bytes());
                    results.lock().unwrap().push((job.key, ok, secs));
                }
            })
        })
        .collect();
    for h in handles {
        h.join().map_err(|_| "a test pool worker panicked".to_string())?;
    }
    let results = results.lock().unwrap();
    for (_, ok, _) in results.iter() {
        if *ok {
            out.passed += 1;
        } else {
            out.failed += 1;
        }
    }
    let ran: Vec<(String, f64)> = results.iter().map(|(k, _, s)| (k.clone(), *s)).collect();
    let _ = std::fs::write(&times_path, kept_times(times, &ran, &known));
    Ok(out)
}

/// The times file after a run: each test that ran with its new time, each
/// one that did not (skipped by `--fast`) with its last, and none of a test
/// that is gone (a stale time ordered the queue, and read as work there was
/// not: a test split long ago kept its six minutes)
fn kept_times(mut times: HashMap<String, f64>, ran: &[(String, f64)], known: &HashSet<String>) -> String {
    for (key, secs) in ran {
        times.insert(key.clone(), *secs);
    }
    let mut tsv: Vec<String> = times.into_iter().filter(|(k, _)| known.contains(k)).map(|(k, v)| format!("{k}\t{v:.2}")).collect();
    tsv.sort();
    tsv.join("\n") + "\n"
}

/// The test binaries `cargo test --no-run --message-format=json` built
fn binaries(json: &str) -> Vec<Binary> {
    let mut out = vec![];
    for l in json.lines() {
        let Ok(v) = serde_json::from_str::<serde_json::Value>(l) else { continue };
        if v["reason"] != "compiler-artifact" || v["profile"]["test"] != true {
            continue;
        }
        let Some(exe) = v["executable"].as_str() else { continue };
        let manifest = PathBuf::from(v["manifest_path"].as_str().unwrap_or_default());
        let dir = manifest.parent().map_or_else(root, PathBuf::from);
        out.push(Binary {
            pkg: package_name(v["package_id"].as_str().unwrap_or_default(), &dir),
            target: v["target"]["name"].as_str().unwrap_or_default().to_string(),
            kind: v["target"]["kind"][0].as_str().unwrap_or("test").to_string(),
            exe: PathBuf::from(exe),
            dir,
        });
    }
    out
}

/// A package's name from its id (`path+file:///…/name#name@1.0.0`, or
/// `…/name#1.0.0` when the directory is named after it)
fn package_name(id: &str, dir: &std::path::Path) -> String {
    match id.rsplit_once('#') {
        Some((_, tail)) if tail.contains('@') => tail.split('@').next().unwrap_or_default().to_string(),
        _ => dir.file_name().map_or_default(|n| n.to_string_lossy().into_owned()),
    }
}

/// The tests of a binary (only the ignored ones with `ignored`)
fn list(b: &Binary, ignored: bool) -> Result<Vec<String>, String> {
    let mut cmd = Command::new(&b.exe);
    cmd.args(["--list", "--format", "terse"]).current_dir(&b.dir).env("CARGO_MANIFEST_DIR", &b.dir);
    if ignored {
        cmd.arg("--ignored");
    }
    let o = cmd.output().map_err(|e| format!("{} --list: {e}", b.exe.display()))?;
    Ok(String::from_utf8_lossy(&o.stdout).lines().filter_map(|l| l.strip_suffix(": test")).map(String::from).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The tests' profile is the release one without link-time optimisation,
    /// and the release one keeps it: the game, the goldens and the bench are
    /// measured as they ship, and the tests relink in seconds (`Cargo.toml`)
    #[test]
    fn the_tests_profile_is_release_without_lto() {
        let text = std::fs::read_to_string(crate::root().join("Cargo.toml")).unwrap();
        let cargo: toml::Value = toml::from_str(&text).unwrap();
        let profile = |name: &str| cargo["profile"].get(name).unwrap_or_else(|| panic!("no profile {name}")).clone();
        let tests = profile(PROFILE);
        assert_eq!(tests["inherits"].as_str(), Some("release"), "{PROFILE} is not release's");
        assert_eq!(tests["lto"].as_bool(), Some(false), "{PROFILE} links with LTO again");
        let keys: Vec<_> = tests.as_table().unwrap().keys().cloned().collect();
        assert_eq!(keys, ["inherits", "lto"], "{PROFILE} differs from release in more than LTO: {keys:?}");
        assert_eq!(profile("release")["lto"].as_str(), Some("thin"), "release lost its LTO");
    }

    #[test]
    fn the_times_kept_are_the_tests_there_are() {
        let old: HashMap<String, f64> =
            [("a::t::ran", 9.0), ("a::t::skipped", 30.0), ("a::t::gone", 352.0)].iter().map(|(k, v)| (k.to_string(), *v)).collect();
        let known: HashSet<String> = ["a::t::ran", "a::t::skipped", "a::t::new"].iter().map(|k| k.to_string()).collect();
        let ran = [("a::t::ran".to_string(), 2.5), ("a::t::new".to_string(), 1.0)];
        assert_eq!(kept_times(old, &ran, &known), "a::t::new\t1.00\na::t::ran\t2.50\na::t::skipped\t30.00\n");
    }

    #[test]
    fn a_package_is_named_from_its_id() {
        let d = std::path::Path::new("/x/crates/placers/placers_office");
        assert_eq!(package_name("path+file:///x/crates/game#0.1.0", std::path::Path::new("/x/crates/game")), "game");
        assert_eq!(package_name("path+file:///x/crates/placers/placers_office#placers_office@1.0.0", d), "placers_office");
    }
}
