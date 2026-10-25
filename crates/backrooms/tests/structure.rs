//! Structural tests: the architecture and conventions that `CLAUDE.md` and
//! `docs/DEVELOPMENT.md` state, checked mechanically, each failure saying how
//! to fix it. Cheap (no level is built); part of the fast check.
use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().unwrap()
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(root().join(rel)).unwrap_or_else(|e| panic!("{rel}: {e}"))
}

/// Every `.rs` file under `dir` (relative to the root), skipping build output.
fn rust_files(dir: &str) -> Vec<PathBuf> {
    let mut out = vec![];
    let mut stack = vec![root().join(dir)];
    while let Some(d) = stack.pop() {
        for e in std::fs::read_dir(&d).into_iter().flatten().flatten() {
            let p = e.path();
            if p.is_dir() {
                if p.file_name().is_some_and(|n| n != "target") {
                    stack.push(p);
                }
            } else if p.extension().is_some_and(|x| x == "rs") {
                out.push(p);
            }
        }
    }
    out.sort();
    out
}

fn rel(p: &Path) -> String {
    p.strip_prefix(root()).unwrap_or(p).display().to_string()
}

/// The `[dependencies]` names of a crate's manifest.
fn dependencies(manifest: &str) -> Vec<String> {
    let text = read(manifest);
    let Some(i) = text.find("[dependencies]") else { return vec![] };
    text[i + 14..]
        .lines()
        .take_while(|l| !l.starts_with('['))
        .filter_map(|l| l.split('=').next())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty() && !s.starts_with('#'))
        .collect()
}

#[test]
fn layers_depend_downward_only() {
    let engine = dependencies("crates/engine/Cargo.toml");
    assert!(
        !engine.iter().any(|d| d == "game"),
        "crates/engine depends on game: the engine must not know the game — move the shared code into engine, or pass it in from the game"
    );
    let game = dependencies("crates/game/Cargo.toml");
    let platform = ["winit", "cpal", "objc2-metal", "objc2-app-kit", "objc2-quartz-core", "raw-window-handle"];
    let bad: Vec<&String> = game.iter().filter(|d| platform.contains(&d.as_str())).collect();
    assert!(bad.is_empty(), "crates/game depends on {bad:?}: the game never touches the window, Metal or CoreAudio — go through engine (that keeps it testable headless)");
}

#[test]
fn level_builds_take_no_clock_or_foreign_randomness() {
    let dirs = [
        "crates/game/src/world",
        "crates/game/src/level",
        "crates/game/src/core_props",
        "crates/game/src/placers",
        "crates/game/src/anomalies",
        "crates/game/src/modes",
        "crates/placers",
    ];
    let banned = ["Instant::now", "SystemTime", "thread_rng", "rand::", "RandomState"];
    let mut hits = vec![];
    for d in dirs {
        for f in rust_files(d) {
            for (n, line) in std::fs::read_to_string(&f).unwrap().lines().enumerate() {
                let code = line.split("//").next().unwrap_or("");
                for b in banned {
                    if code.contains(b) {
                        hits.push(format!("{}:{}: {b}", rel(&f), n + 1));
                    }
                }
            }
        }
    }
    assert!(
        hits.is_empty(),
        "level-build code must be a pure function of (run seed, level): draw from rnd() / with_native() instead of {hits:#?}\n(see docs/DEVELOPMENT.md, Determinism rules)"
    );
}

#[test]
fn every_themed_placer_has_a_theme() {
    let themes: std::collections::BTreeSet<&str> = game::world::tables::PLACER_THEME.iter().map(|(n, _)| *n).collect();
    let missing: Vec<&str> = game::world::tables::THEMED_ORDER.iter().copied().filter(|n| !themes.contains(n)).collect();
    assert!(missing.is_empty(), "placers in THEMED_ORDER without an entry in PLACER_THEME (crates/game/src/world/tables.rs): {missing:?} — add one, or the zone purposes cannot place them");
}

#[test]
fn every_creature_has_goldens_and_behaviour_tests() {
    let src = read("crates/game/src/entities/mod.rs");
    let mut kinds: Vec<String> =
        src.match_indices("kind = \"").map(|(i, m)| src[i + m.len()..].split('"').next().unwrap().to_string()).collect();
    kinds.sort();
    kinds.dedup();
    assert!(kinds.len() >= 3, "found the creature kinds in entities/mod.rs: {kinds:?}");
    let shots = read("xtask/src/shots.rs");
    let tests = read("crates/game/tests/entities.rs");
    for k in &kinds {
        assert!(shots.contains(&format!("\"{k}\"")), "creature {k} has no golden shots: add it to CREATURES in xtask/src/shots.rs, then `cargo xtask shots --accept {k}_2m,{k}_4m,{k}_8m,{k}_death --reason …`");
        assert!(
            tests.contains(&format!("\"{k}\"")),
            "creature {k} is never named in crates/game/tests/entities.rs: give it build/animation fixtures and senses tests"
        );
    }
}

/// Files that were already past the cap when it was set, with their size
/// then: they may shrink, not grow. Split them when you next work in them.
const OVERSIZE: &[(&str, usize)] = &[];
/// No source file grows past this many lines.
const MAX_LINES: usize = 2400;

#[test]
fn source_files_stay_a_readable_size() {
    let mut big = vec![];
    for d in ["crates", "xtask"] {
        for f in rust_files(d) {
            let n = std::fs::read_to_string(&f).unwrap().lines().count();
            let r = rel(&f);
            let cap = OVERSIZE.iter().find(|(p, _)| *p == r).map(|(_, c)| *c).unwrap_or(MAX_LINES);
            if n > cap {
                big.push(format!("{r}: {n} lines (cap {cap})"));
            }
        }
    }
    assert!(
        big.is_empty(),
        "split these into modules — one subsystem per folder, a mod.rs saying what it owns (docs/DEVELOPMENT.md, Modules): {big:#?}"
    );
}

#[test]
fn docs_point_at_files_that_exist() {
    let docs = [
        "CLAUDE.md",
        "README.md",
        "docs/ARCHITECTURE.md",
        "docs/DEVELOPMENT.md",
        ".claude/rules/world.md",
        ".claude/rules/audio.md",
        ".claude/rules/visuals.md",
        ".claude/rules/entities.md",
    ];
    // every file in the repository, by its path relative to the root
    let mut all: Vec<String> = vec![];
    let mut stack = vec![root()];
    while let Some(d) = stack.pop() {
        for e in std::fs::read_dir(&d).into_iter().flatten().flatten() {
            let p = e.path();
            let name = p.file_name().unwrap().to_string_lossy().to_string();
            if p.is_dir() {
                if !["target", ".git", "dist"].contains(&name.as_str()) {
                    all.push(format!("{}/", rel(&p)));
                    stack.push(p);
                }
            } else {
                all.push(rel(&p));
            }
        }
    }
    // the skills and the reviewers name code too
    let mut docs: Vec<String> = docs.iter().map(|d| d.to_string()).collect();
    for dir in [".claude/skills", ".claude/agents"] {
        for e in std::fs::read_dir(root().join(dir)).into_iter().flatten().flatten() {
            let p = e.path();
            let f = if p.is_dir() { p.join("SKILL.md") } else { p };
            if f.extension().is_some_and(|x| x == "md") && f.exists() {
                docs.push(rel(&f));
            }
        }
    }
    let mut missing = vec![];
    for doc in &docs {
        let text = read(doc);
        for token in text.split('`').skip(1).step_by(2) {
            let t = token.trim();
            let pathlike = (t.ends_with(".rs") || t.ends_with(".md") || t.ends_with(".json") || t.ends_with('/'))
                && !t.contains(' ')
                && !t.contains('*')
                && !t.contains('<');
            if !pathlike
                || t.starts_with("target/")
                || t.starts_with("../")
                || t.starts_with('.') && !t.starts_with(".claude")
                || t.len() < 4
            {
                continue;
            }
            let found =
                all.iter().any(|f| f == t || f.ends_with(&format!("/{t}")) || (!t.contains('/') && f.rsplit('/').next() == Some(t)));
            if !found {
                missing.push(format!("{doc}: `{t}`"));
            }
        }
    }
    assert!(missing.is_empty(), "docs name files that do not exist — fix the path or the doc: {missing:#?}");
}

#[test]
fn the_fast_check_skips_only_tests_that_exist() {
    let check = read("xtask/src/check.rs");
    let i = check.find("const FAST_SKIP").expect("FAST_SKIP in xtask/src/check.rs");
    let list = &check[i..check[i..].find("];").unwrap() + i];
    let names: Vec<&str> = list.split('"').skip(1).step_by(2).collect();
    let mut tests = String::new();
    for d in ["crates"] {
        for f in rust_files(d) {
            tests.push_str(&std::fs::read_to_string(&f).unwrap());
        }
    }
    let stale: Vec<&&str> = names.iter().filter(|n| !tests.contains(&format!("fn {n}()"))).collect();
    assert!(
        stale.is_empty(),
        "FAST_SKIP in xtask/src/check.rs names tests that no longer exist: {stale:?} — update the list to the new test names"
    );
}

#[test]
fn the_content_ledger_covers_the_content() {
    let ledger: serde_json::Value = serde_json::from_str(&read("docs/content.json")).expect("docs/content.json is JSON");
    let entries = ledger["entries"].as_array().expect("docs/content.json: entries");
    let ids: std::collections::BTreeSet<&str> = entries.iter().filter_map(|e| e["id"].as_str()).collect();
    let mut missing = vec![];
    // what the code has
    let src = read("crates/game/src/entities/mod.rs");
    for (i, m) in src.match_indices("kind = \"") {
        let k = src[i + m.len()..].split('"').next().unwrap();
        missing.push(format!("creature/{k}"));
    }
    for k in game::level::scp::SCP_KEYS {
        missing.push(format!("anomaly/{k}"));
    }
    let themes: std::collections::BTreeSet<&str> = game::world::tables::PLACER_THEME.iter().map(|(_, t)| *t).collect();
    for t in themes {
        missing.push(format!("props/{t}"));
    }
    let style = read("docs/STYLE.md");
    let block = style.split("```json").nth(1).and_then(|r| r.split("```").next()).expect("docs/STYLE.md: style bands");
    let bands: serde_json::Value = serde_json::from_str(block).unwrap();
    for p in bands["purposes"].as_object().unwrap().keys() {
        missing.push(format!("purpose/{}", p.replace(' ', "_")));
    }
    missing.retain(|id| !ids.contains(id.as_str()));
    missing.dedup();
    assert!(missing.is_empty(), "docs/content.json has no entry for {missing:?} — add one with the checks it must pass");
    // what the ledger names exists
    let mut dangling = vec![];
    for e in entries {
        for c in e["checks"].as_array().into_iter().flatten().filter_map(|c| c.as_str()) {
            let ok = match c.split_once(':') {
                Some(("shots", n)) => root().join(format!("crates/backrooms/tests/golden/{n}.png")).exists(),
                Some(("style", p)) => !bands["purposes"][p].is_null(),
                Some(("test", t)) => t.split_once('/').is_some_and(|(cr, f)| root().join(format!("crates/{cr}/tests/{f}.rs")).exists()),
                Some(("lint", "placer-smoke")) => true,
                _ => false,
            };
            if !ok {
                dangling.push(format!("{}: {c}", e["id"].as_str().unwrap_or("?")));
            }
        }
    }
    assert!(dangling.is_empty(), "docs/content.json names checks that do not exist: {dangling:?} — fix the name or add the check");
}
