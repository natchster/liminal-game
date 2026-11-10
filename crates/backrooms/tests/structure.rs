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

/// Playtesting never sounds through the speakers. The output device is off
/// unless the windowed game turns it on: one call, in `main`, after every
/// headless mode has returned and just before the window opens. A mode
/// added later is silent without remembering to be. `cargo xtask` also
/// silences everything it runs (`BACKROOMS_SILENT`), a test binary can't
/// open it (`engine::audio::device_output`), and no test uses the device's
/// context
#[test]
fn automated_runs_never_open_the_output_device() {
    // spelt in pieces, so this file does not match itself
    let on = concat!("set_device_output(", "true)");
    let audio = read("crates/engine/src/audio.rs");
    assert!(
        audio.contains(concat!("static DEVICE_OUTPUT: AtomicBool = AtomicBool::new(", "false);")),
        "engine::audio: the output device must start off (DEVICE_OUTPUT = false); only the windowed game turns it on"
    );
    // the one place it is turned on
    let mut ons = vec![];
    for dir in ["crates", "xtask/src"] {
        for f in rust_files(dir) {
            let s = std::fs::read_to_string(&f).unwrap();
            if rel(&f) != "crates/backrooms/tests/structure.rs" {
                ons.extend(s.matches(on).map(|_| rel(&f)));
            }
        }
    }
    assert_eq!(
        ons,
        vec!["crates/backrooms/src/main.rs".to_string()],
        "the speakers are turned on in exactly one place, main.rs before the window opens; found {on} in {ons:?}"
    );
    let main = read("crates/backrooms/src/main.rs");
    let body = &main[main.find("fn main()").expect("main.rs has fn main")..];
    let at_on = body.find(on).expect("main turns the speakers on for the window");
    let at_run = body.find("run(\n        AppConfig").expect("main opens the window with run(AppConfig …)");
    let last_return = body[..at_run].rfind("return;").expect("main's headless modes return before the window");
    assert!(
        last_return < at_on && at_on < at_run,
        "main.rs: {on} must come after every headless mode has returned and just before run(AppConfig …), so no headless run can reach it"
    );
    for mode in ["bench(&o", "bench_creatures(&o", "metrics(&o", "level_check(&o", "screenshot(&o"] {
        let at = body.find(mode).unwrap_or_else(|| panic!("main.rs: the headless mode {mode} …) moved; update this test"));
        assert!(at < at_on, "main.rs: the headless mode {mode} …) runs after the speakers are turned on");
    }
    let xtask = read("xtask/src/main.rs");
    let xmain = &xtask[xtask.find("fn main()").expect("xtask has fn main")..];
    let first_task = xmain.find("match args.first()").expect("xtask dispatches its tasks");
    assert!(
        xtask.contains("const SILENT_ENV: &str = \"BACKROOMS_SILENT\";")
            && xmain[..first_task].contains("std::env::set_var(SILENT_ENV, \"1\")"),
        "xtask/src/main.rs: set BACKROOMS_SILENT before any task runs, so nothing a task starts sounds through the speakers"
    );
    let device = concat!("AudioContext::", "new()");
    let guard = concat!("AudioContext::", "new().plays_on_device()");
    for dir in ["crates/game/tests", "crates/backrooms/tests", "crates/engine/tests"] {
        for f in rust_files(dir) {
            // only the guard itself asks the device context whether it plays
            let s = std::fs::read_to_string(&f).unwrap().replace(guard, "");
            assert!(!s.contains(device), "{}: a test uses AudioContext::new_silent() or new_offline(), never the device", rel(&f));
        }
    }
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
        "crates/game/src/story",
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
    // the creatures: the kinds on the brain (`creature::on_new_brain`)
    let src = read("crates/game/src/entities/creature.rs");
    let at = src.find("pub fn on_new_brain").expect("creature::on_new_brain");
    let body = &src[at..at + src[at..].find('}').unwrap()];
    let mut kinds: Vec<String> = body.split('"').skip(1).step_by(2).map(|k| k.to_string()).collect();
    kinds.sort();
    kinds.dedup();
    assert!(kinds.len() >= 5, "found the creature kinds in creature::on_new_brain: {kinds:?}");
    let shots = read("xtask/src/shots.rs");
    // (a kind that lives behind a real window, the one behind the glass:
    // its window is cut into a real wall and lit by its lamp, which no arena
    // room has, so its rules and its counter are held on real floors, in
    // crates/backrooms/tests/<kind>.rs)
    const ON_REAL_FLOORS: &[&str] = &["glass"];
    for k in &kinds {
        let (rules, arenas) = if ON_REAL_FLOORS.contains(&k.as_str()) {
            let f = read(&format!("crates/backrooms/tests/{k}.rs"));
            (f.clone(), f)
        } else {
            (read("crates/game/tests/creatures_rules.rs"), read("crates/game/tests/creatures_arenas.rs"))
        };
        assert!(shots.contains(&format!("\"{k}\"")), "creature {k} has no golden shots: add it to CREATURES in xtask/src/shots.rs, then `cargo xtask shots --accept {k}_2m,{k}_4m,{k}_8m,{k}_death --reason …`");
        assert!(
            rules.contains(&format!("\"{k}\"")),
            "creature {k} is never named in crates/game/tests/creatures_rules.rs: one test a line of its design"
        );
        assert!(
            arenas.contains(&format!("report(\"{k}\")")),
            "creature {k} has no arena in crates/game/tests/creatures_arenas.rs: its counter held to the bands"
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
                // build output, and the voice tools' Python environments
                // (third-party code, ignored by git: tools/voice/*/.venv)
                if !["target", ".git", "dist", ".venv", "__pycache__"].contains(&name.as_str()) {
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

/// FNV-1a 64, as `cargo xtask assets` writes it into the manifest.
fn fnv64(bytes: &[u8]) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in bytes {
        h ^= *b as u64;
        h = h.wrapping_mul(0x0000_0100_0000_01b3);
    }
    format!("{h:016x}")
}

#[test]
fn every_embedded_asset_has_a_current_licensed_manifest_entry() {
    let m: serde_json::Value =
        serde_json::from_str(&read("crates/game/assets/MANIFEST.json")).expect("crates/game/assets/MANIFEST.json is JSON");
    let allowed = |k: &str| -> Vec<String> { m[k].as_array().into_iter().flatten().filter_map(|v| v.as_str().map(String::from)).collect() };
    let (licences, generators, models) = (allowed("allowed_licences"), allowed("allowed_generators"), allowed("allowed_models"));
    let mut problems = vec![];
    // the meshes, the photographed textures (named `textures/<file>`) and
    // the voice lines (`voice/<persona>/<line>.wav`)
    let voices: Vec<(String, std::fs::DirEntry)> = std::fs::read_dir(root().join("crates/game/assets/voice"))
        .into_iter()
        .flatten()
        .flatten()
        .filter(|d| d.path().is_dir())
        .flat_map(|d| {
            let persona = d.file_name().to_string_lossy().to_string();
            std::fs::read_dir(d.path())
                .into_iter()
                .flatten()
                .flatten()
                .map(move |e| (format!("voice/{persona}/{}", e.file_name().to_string_lossy()), e))
                .filter(|(n, _)| n.ends_with(".wav"))
        })
        .collect();
    // a voice line's entry with no file: a line renamed or dropped by hand
    // (`cargo xtask voice write` removes both)
    for name in m["files"].as_object().into_iter().flatten().map(|(k, _)| k).filter(|k| k.starts_with("voice/")) {
        if !voices.iter().any(|(n, _)| n == name) {
            problems.push(format!(
                "{name}: in MANIFEST.json but not in crates/game/assets (rewrite its persona with `cargo xtask voice write`)"
            ));
        }
    }
    let files = std::fs::read_dir(root().join("crates/game/assets"))
        .unwrap()
        .flatten()
        .map(|e| (e.file_name().to_string_lossy().to_string(), e))
        .filter(|(n, _)| n.ends_with(".bin"))
        .chain(
            std::fs::read_dir(root().join("crates/game/assets/textures"))
                .unwrap()
                .flatten()
                .map(|e| (format!("textures/{}", e.file_name().to_string_lossy()), e))
                .filter(|(n, _)| n.ends_with(".jpg")),
        )
        .chain(voices);
    for (name, e) in files {
        let entry = &m["files"][&name];
        if entry.is_null() {
            problems.push(format!("{name}: no entry in MANIFEST.json (where it comes from, its licence, its generator)"));
            continue;
        }
        let fp = fnv64(&std::fs::read(e.path()).unwrap());
        if entry["fingerprint"].as_str() != Some(fp.as_str()) {
            let how = if name.starts_with("voice/") { "cargo xtask voice write" } else { "cargo xtask assets …" };
            problems.push(format!("{name}: the file changed but its manifest entry did not (fingerprint {fp}); rebuild it with `{how}`, which records it, and say what changed"));
        }
        let lic = entry["licence"].as_str().unwrap_or("");
        if !licences.iter().any(|l| l == lic) {
            problems.push(format!("{name}: licence \"{lic}\" is not in allowed_licences"));
        }
        let gen = entry["generator"].as_str().unwrap_or("");
        if !generators.iter().any(|g| g == gen) {
            problems.push(format!("{name}: generator \"{gen}\" is not in allowed_generators"));
        }
        // the model that made it, and (a voice line) the one that designed the voice it is cloned from
        for key in ["generative_model", "reference_model"] {
            if let Some(model) = entry[key].as_str() {
                if !models.iter().any(|x| x == model) {
                    problems.push(format!("{name}: {key} \"{model}\" is not in allowed_models (check its licence: non-commercial and territory-limited weights are not allowed)"));
                }
            }
        }
        if name.starts_with("voice/") && entry["generative_model"].as_str().is_none() {
            problems.push(format!("{name}: a voice line names the model that said it (generative_model)"));
        }
        for s in entry["sources"].as_array().into_iter().flatten().filter_map(|s| s.as_str()) {
            if !root().join(s).exists() {
                problems.push(format!("{name}: source {s} does not exist"));
            }
        }
    }
    assert!(problems.is_empty(), "{problems:#?}");
}

/// What the earlier version the game grew from was called, in the forms the
/// repository once used for it. Spelled in pieces (`concat!`) so that this
/// file does not match itself.
const EARLIER_VERSION: &[&str] = &[
    concat!("html", " version"),
    concat!("html", "/js"),
    concat!("html", "-recorded"),
    concat!("js", " version"),
    concat!("web", " version"),
    concat!("backrooms", ".js"),
    concat!("three", ".js"),
    concat!("three", "js"),
    concat!("ported", " from"),
];

/// Deliberate mentions, each with its reason: (file, the form it uses, why).
const EARLIER_VERSION_EXCEPTIONS: &[(&str, &str, &str)] = &[
    (
        "docs/SOURCES.md",
        concat!("three", ".js"),
        "cites the source of the windows' ground-projected sky: the GroundedSkybox of that library, research, not this game's past",
    ),
    (
        "docs/research/windows-real.md",
        concat!("three", ".js"),
        "research on the ground-projected sky: quotes that library's GroundedSkybox and links its docs and source",
    ),
    ("docs/research/windows-real.md", concat!("three", "js"), "the same research's link to that library's docs (its domain)"),
];

/// The development-round tags of the earlier version in a text: `R` and
/// three or four digits standing alone (not inside a word, a path or a
/// number), as (line number from 1, tag). EBU R-numbers (loudness) and the
/// IRC's R-sections (building codes) are standards, not rounds.
fn round_tags(text: &str) -> Vec<(usize, String)> {
    let b = text.as_bytes();
    let mut out = vec![];
    for i in 0..b.len() {
        if b[i] != b'R' {
            continue;
        }
        let before_ok = i == 0 || !(b[i - 1].is_ascii_alphanumeric() || b"/_-.#".contains(&b[i - 1]));
        let digits = b[i + 1..].iter().take_while(|c| c.is_ascii_digit()).count();
        let after = i + 1 + digits;
        let after_ok = after >= b.len() || !(b[after].is_ascii_alphanumeric() || b[after] == b'_');
        let lead = text[..i].trim_end();
        let standard = lead.ends_with("EBU") || lead.ends_with("IRC");
        if before_ok && (3..=4).contains(&digits) && after_ok && !standard {
            out.push((text[..i].matches('\n').count() + 1, text[i..after].to_string()));
        }
    }
    out
}

/// The game stands on its own: no text file names the earlier version it
/// grew from (`EARLIER_VERSION`, but for `EARLIER_VERSION_EXCEPTIONS`), and
/// no Rust or Markdown file carries one of that version's round tags
/// (`round_tags`). Say what a thing is or does in this game instead.
#[test]
fn nothing_names_the_earlier_version() {
    const TEXT: &[&str] = &["rs", "md", "py", "toml", "json", "jsonl", "yml", "yaml", "sh", "metal", "txt", "tsv", "csv"];
    let mut files = vec![];
    let mut stack = vec![root()];
    while let Some(d) = stack.pop() {
        for e in std::fs::read_dir(&d).into_iter().flatten().flatten() {
            let p = e.path();
            let name = p.file_name().unwrap().to_string_lossy().to_string();
            if p.is_dir() {
                // build output, and the voice tools' Python environments
                // (third-party code, ignored by git: tools/voice/*/.venv)
                if !["target", ".git", "dist", ".venv", "__pycache__"].contains(&name.as_str()) {
                    stack.push(p);
                }
            } else if p.extension().is_some_and(|x| TEXT.contains(&x.to_string_lossy().as_ref())) {
                files.push(p);
            }
        }
    }
    files.sort();
    let mut found = vec![];
    for f in &files {
        let Ok(text) = std::fs::read_to_string(f) else { continue };
        let file = rel(f);
        let ext = f.extension().unwrap().to_string_lossy().to_string();
        for (n, line) in text.lines().enumerate() {
            let lower = line.to_lowercase();
            for form in EARLIER_VERSION {
                let excused = EARLIER_VERSION_EXCEPTIONS.iter().any(|(ef, eform, _)| *ef == file && eform == form);
                if lower.contains(form) && !excused {
                    found.push(format!("{file}:{}: \"{form}\"", n + 1));
                }
            }
        }
        if ext == "rs" || ext == "md" {
            for (n, tag) in round_tags(&text) {
                found.push(format!("{file}:{n}: round tag {tag}"));
            }
        }
    }
    assert!(
        found.is_empty(),
        "these lines name the earlier version the game grew from; say what the thing is or does in this game \
         (or, for a citation that must stay, add it to EARLIER_VERSION_EXCEPTIONS with its reason): {found:#?}"
    );
    // every exception is still needed
    for (file, form, why) in EARLIER_VERSION_EXCEPTIONS {
        let text = read(file).to_lowercase();
        assert!(text.contains(form), "{file} no longer mentions \"{form}\": drop its exception ({why})");
    }
}

#[test]
fn round_tags_are_found_and_standards_are_not() {
    let tag = concat!("R", "653");
    assert_eq!(round_tags(&format!("// x\n// {tag} fertilizer spikes")), vec![(2, tag.to_string())]);
    assert_eq!(round_tags(&format!("(R{}–R{})", 220, 227)).len(), 2);
    assert!(round_tags(concat!("EBU R", "128 loudness, EBU\n  R", "128 again")).is_empty());
    assert!(round_tags(concat!("(IRC R", "312.2.1)")).is_empty());
    assert!(round_tags(concat!("frames/R", "0020 and R8Unorm, R", "2D, rooms R4, place_gameplay_r", "1_trigger")).is_empty());
}
