//! `cargo xtask cutscene`: the cutscene pipeline (`docs/design/CUTSCENES.md`).
//!
//! ```text
//! cargo xtask cutscene check     the sources against the format, the bundle against the sources,
//!                                and the game's own rules (`backrooms --cutscene-check`)
//! cargo xtask cutscene write     compile assets/src/cutscenes/*.toml into crates/game/assets/cutscenes.json
//!                                (every field written out) and its MANIFEST.json entry
//! cargo xtask cutscene list      what there is: id, length, world, stages, about
//! cargo xtask cutscene preview ID [--stage S] [--every SECONDS]
//!                                the sources compiled (not written) and ID played on its stage
//!                                (each stage when none is named): its frames, a contact sheet,
//!                                its timeline and its contract in target/xtask/cutscene/ID/STAGE/
//! ```
//!
//! This file knows the *format* (the fields, their kinds and their
//! defaults); the *rules* (times in order, the blend envelope, the first
//! key, the events and sounds the game knows, the comfort limits) are the
//! game's (`game::cutscene`), run on the compiled bundle through the game's
//! binary, so there is one copy of them.
use crate::{cargo, root, run};
use serde_json::{json, Map, Value};
use std::path::{Path, PathBuf};

const USAGE: &str = "usage: cargo xtask cutscene check | write | list | preview ID [--stage S] [--every SECONDS]";

/// The bundle's own line
const ABOUT: &str = "Every cutscene, compiled by `cargo xtask cutscene write` from assets/src/cutscenes/*.toml (every field written out). Do not edit: change the source and write it again (docs/design/CUTSCENES.md).";

pub fn cutscene(args: &[String]) -> Result<(), String> {
    match args.first().map(String::as_str) {
        Some("check") => check(),
        Some("write") => write(),
        Some("list") => list(),
        Some("preview") => preview(&args[1..]),
        _ => Err(USAGE.into()),
    }
}

fn src_dir() -> PathBuf {
    root().join("assets/src/cutscenes")
}

fn bundle_path() -> PathBuf {
    root().join("crates/game/assets/cutscenes.json")
}

/// The sources, by file name
fn sources(dir: &Path) -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "toml"))
        .collect();
    v.sort();
    v
}

#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Str,
    Num,
    Bool,
    Vec3,
    StrList,
    /// a list of words, maybe empty
    Names,
    Table,
    Tables,
}

/// A field: its name, kind, and default (`None`: required; `Some(Null)`: absent is null)
type Field = (&'static str, Kind, Option<Value>);

fn top() -> Vec<Field> {
    vec![
        ("id", Kind::Str, None),
        ("about", Kind::Str, None),
        ("world", Kind::Str, Some(json!("live"))),
        ("then", Kind::Str, Some(json!("back"))),
        ("skip", Kind::Str, Some(json!("hold"))),
        // (its default depends on the world: filled in by `compile`)
        ("breaks_off", Kind::Bool, Some(Value::Null)),
        ("length", Kind::Num, None),
        ("blend_in", Kind::Num, Some(json!(0.5))),
        ("blend_out", Kind::Num, Some(json!(1.0))),
        ("repeat_from", Kind::Num, Some(json!(0.0))),
        ("stages", Kind::StrList, None),
        ("finally", Kind::Names, Some(json!([]))),
        ("bars", Kind::Table, Some(json!({}))),
        ("camera", Kind::Tables, None),
        ("fade", Kind::Tables, Some(json!([]))),
        ("sound", Kind::Tables, Some(json!([]))),
        ("event", Kind::Tables, Some(json!([]))),
        ("say", Kind::Tables, Some(json!([]))),
    ]
}

fn bars() -> Vec<Field> {
    // (`out` defaults to half a second before the end: filled in by `compile`)
    vec![("in", Kind::Num, Some(json!(0.0))), ("out", Kind::Num, Some(Value::Null))]
}

fn key() -> Vec<Field> {
    vec![
        ("t", Kind::Num, None),
        ("anchor", Kind::Str, None),
        ("pos", Kind::Vec3, Some(json!([0.0, 0.0, 0.0]))),
        ("look", Kind::Vec3, Some(Value::Null)),
        ("look_at", Kind::Str, Some(Value::Null)),
        ("yaw", Kind::Num, Some(json!(0.0))),
        ("pitch", Kind::Num, Some(json!(0.0))),
        ("roll", Kind::Num, Some(json!(0.0))),
        ("fov", Kind::Num, Some(json!(0.0))),
        ("ease", Kind::Str, Some(json!("in_out"))),
        ("cut", Kind::Bool, Some(json!(false))),
        ("sway", Kind::Num, Some(json!(0.0))),
        ("walk", Kind::Num, Some(json!(0.0))),
        ("jolt", Kind::Bool, Some(json!(false))),
    ]
}

fn fade() -> Vec<Field> {
    vec![("t", Kind::Num, None), ("to", Kind::Num, None), ("over", Kind::Num, Some(json!(0.4))), ("colour", Kind::Str, Some(json!("#000")))]
}

fn sound() -> Vec<Field> {
    vec![
        ("t", Kind::Num, None),
        ("name", Kind::Str, None),
        ("anchor", Kind::Str, Some(json!("eyes"))),
        ("pos", Kind::Vec3, Some(json!([0.0, 0.0, 0.0]))),
        ("gain", Kind::Num, Some(json!(1.0))),
    ]
}

fn event() -> Vec<Field> {
    vec![("t", Kind::Num, None), ("name", Kind::Str, None), ("on_skip", Kind::Bool, Some(json!(true)))]
}

fn say() -> Vec<Field> {
    vec![("t", Kind::Num, None), ("use", Kind::Str, None), ("subject", Kind::Str, Some(json!("")))]
}

/// The words each enumerated field may take
const WORDS: &[(&str, &[&str])] = &[
    ("world", &["live", "held"]),
    ("then", &["back", "card"]),
    ("skip", &["hold", "never"]),
    ("anchor", &["start", "eyes", "thing", "killer"]),
    ("look_at", &["start", "eyes", "thing", "killer"]),
    ("ease", &["linear", "in", "out", "in_out"]),
];

fn num(v: &toml::Value) -> Option<f64> {
    v.as_float().or_else(|| v.as_integer().map(|i| i as f64)).filter(|x| x.is_finite())
}

/// One table against its fields: the JSON object with every field written
/// out, or what is wrong with it
fn table(t: &toml::Table, spec: &[Field], what: &str, problems: &mut Vec<String>) -> Map<String, Value> {
    let mut out = Map::new();
    for k in t.keys() {
        if !spec.iter().any(|(n, _, _)| n == k) {
            problems.push(format!("{what}: unknown field `{k}`"));
        }
    }
    for (name, kind, default) in spec {
        let Some(v) = t.get(*name) else {
            match default {
                Some(d) => {
                    out.insert(name.to_string(), d.clone());
                }
                None => problems.push(format!("{what}: no `{name}`")),
            }
            continue;
        };
        let got = match kind {
            Kind::Str => v.as_str().filter(|s| !s.trim().is_empty()).map(|s| json!(s)),
            Kind::Num => num(v).map(|x| json!(x)),
            Kind::Bool => v.as_bool().map(|b| json!(b)),
            Kind::Vec3 => {
                v.as_array().filter(|a| a.len() == 3).and_then(|a| a.iter().map(num).collect::<Option<Vec<f64>>>()).map(|p| json!(p))
            }
            Kind::StrList => v
                .as_array()
                .filter(|a| !a.is_empty())
                .and_then(|a| a.iter().map(|x| x.as_str().filter(|s| !s.trim().is_empty()).map(String::from)).collect::<Option<Vec<_>>>())
                .map(|l| json!(l)),
            // (read by the caller with their own fields)
            Kind::Names => v
                .as_array()
                .and_then(|a| a.iter().map(|x| x.as_str().filter(|s| !s.trim().is_empty()).map(String::from)).collect::<Option<Vec<_>>>())
                .map(|l| json!(l)),
            Kind::Table => v.as_table().map(|_| Value::Null),
            Kind::Tables => v.as_array().filter(|a| a.iter().all(|x| x.as_table().is_some())).map(|_| Value::Null),
        };
        let Some(got) = got else {
            let says = match kind {
                Kind::Str => "a word or line",
                Kind::Num => "a number",
                Kind::Bool => "true or false",
                Kind::Vec3 => "[x, y, z]",
                Kind::StrList => "a list of words",
                Kind::Names => "a list of words (maybe empty)",
                Kind::Table => "a table",
                Kind::Tables => "a list of tables ([[...]])",
            };
            problems.push(format!("{what}: `{name}` is not {says}"));
            continue;
        };
        if let (Some(s), Some((_, allowed))) = (got.as_str(), WORDS.iter().find(|(w, _)| w == name)) {
            if !allowed.contains(&s) {
                problems.push(format!("{what}: `{name}` \"{s}\" is not one of {}", allowed.join(", ")));
            }
        }
        out.insert(name.to_string(), got);
    }
    out
}

/// One source compiled: every field written out, in the bundle's form
fn compile(text: &str, stem: &str) -> Result<Value, Vec<String>> {
    let t: toml::Table = text.parse().map_err(|e| vec![format!("{stem}.toml: {e}")])?;
    let mut problems = vec![];
    let what = format!("{stem}.toml");
    let mut out = table(&t, &top(), &what, &mut problems);
    if out.get("id").and_then(Value::as_str).is_some_and(|id| id != stem) {
        problems.push(format!("{what}: its id must be its file's name ({stem})"));
    }
    if out.get("breaks_off").is_some_and(Value::is_null) {
        let live = out.get("world").and_then(Value::as_str) == Some("live");
        out.insert("breaks_off".into(), json!(live));
    }
    let length = out.get("length").and_then(Value::as_f64).unwrap_or(0.0);
    let b = t.get("bars").and_then(|b| b.as_table()).cloned().unwrap_or_default();
    let mut bars_out = table(&b, &bars(), &format!("{what}: bars"), &mut problems);
    if bars_out.get("out").is_some_and(Value::is_null) {
        let at_in = bars_out.get("in").and_then(Value::as_f64).unwrap_or(0.0);
        bars_out.insert("out".into(), json!((length - 0.5).max(at_in)));
    }
    out.insert("bars".into(), Value::Object(bars_out));
    for (list, spec, each) in [
        ("camera", key(), "camera key"),
        ("fade", fade(), "fade"),
        ("sound", sound(), "sound"),
        ("event", event(), "event"),
        ("say", say(), "say"),
    ] {
        let items: Vec<Value> = t
            .get(list)
            .and_then(|v| v.as_array())
            .into_iter()
            .flatten()
            .enumerate()
            .filter_map(|(i, x)| x.as_table().map(|x| Value::Object(table(x, &spec, &format!("{what}: {each} {}", i + 1), &mut problems))))
            .collect();
        out.insert(list.into(), Value::Array(items));
    }
    if problems.is_empty() {
        Ok(Value::Object(out))
    } else {
        Err(problems)
    }
}

/// Every source compiled into the bundle's text, or every problem found
fn compile_all(dir: &Path) -> Result<String, Vec<String>> {
    let mut list = vec![];
    let mut problems = vec![];
    for p in sources(dir) {
        let stem = p.file_stem().unwrap_or_default().to_string_lossy().to_string();
        match std::fs::read_to_string(&p) {
            Ok(text) => match compile(&text, &stem) {
                Ok(v) => list.push(v),
                Err(mut e) => problems.append(&mut e),
            },
            Err(e) => problems.push(format!("{}: {e}", p.display())),
        }
    }
    if !problems.is_empty() {
        return Err(problems);
    }
    let bundle = json!({ "about": ABOUT, "cutscenes": list });
    Ok(serde_json::to_string_pretty(&bundle).map_err(|e| vec![e.to_string()])? + "\n")
}

/// The game's own rules on a bundle (`backrooms --cutscene-check FILE`)
fn game_rules(bundle: &Path) -> Result<(), String> {
    run(cargo().args(["build", "--release", "-q", "-p", "backrooms"]))?;
    let out = std::process::Command::new(root().join("target/release/backrooms"))
        .arg("--cutscene-check")
        .arg(bundle)
        .output()
        .map_err(|e| e.to_string())?;
    print!("{}", String::from_utf8_lossy(&out.stdout));
    if out.status.success() {
        Ok(())
    } else {
        Err(format!("the game's rules: {}", String::from_utf8_lossy(&out.stderr).trim()))
    }
}

fn check() -> Result<(), String> {
    let text = compile_all(&src_dir()).map_err(|p| p.join("\n"))?;
    let on_disk = std::fs::read_to_string(bundle_path()).unwrap_or_default();
    if on_disk != text {
        return Err("crates/game/assets/cutscenes.json is not what the sources compile to: `cargo xtask cutscene write`".into());
    }
    game_rules(&bundle_path())?;
    println!("cutscenes: {} sources, the bundle current, the game's rules kept", sources(&src_dir()).len());
    Ok(())
}

fn write() -> Result<(), String> {
    let text = compile_all(&src_dir()).map_err(|p| p.join("\n"))?;
    // the game's rules first, on a copy: a bundle that breaks them is never written
    let tmp = root().join("target/xtask/cutscene/cutscenes.json");
    std::fs::create_dir_all(tmp.parent().unwrap()).map_err(|e| e.to_string())?;
    std::fs::write(&tmp, &text).map_err(|e| e.to_string())?;
    game_rules(&tmp)?;
    std::fs::write(bundle_path(), &text).map_err(|e| e.to_string())?;
    let sources: Vec<String> =
        sources(&src_dir()).iter().map(|p| p.strip_prefix(root()).unwrap_or(p).to_string_lossy().to_string()).collect();
    crate::assets::update_manifest(
        "cutscenes.json",
        text.as_bytes(),
        json!({
            "authors": "written by coding agents under the developer's direction",
            "built_with": "cargo xtask cutscene write (TOML sources, every field written out)",
            "generative_model": null,
            "generator": "written by hand, compiled by scripts",
            "licence": "MIT",
            "rebuild": "cargo xtask cutscene write",
            "sources": sources,
        }),
    )?;
    println!("wrote crates/game/assets/cutscenes.json ({} cutscenes) and its manifest entry", sources.len());
    Ok(())
}

/// ID played on its stages from the sources as they stand (compiled to a
/// copy, not written): frames, a contact sheet, a timeline and the
/// contract, to look at before writing
fn preview(args: &[String]) -> Result<(), String> {
    let id = args.first().filter(|a| !a.starts_with("--")).ok_or(USAGE)?;
    let text = compile_all(&src_dir()).map_err(|p| p.join("\n"))?;
    let v: Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
    let c = v["cutscenes"].as_array().into_iter().flatten().find(|c| c["id"] == id.as_str()).ok_or(format!("no source for {id}"))?.clone();
    let tmp = root().join("target/xtask/cutscene/cutscenes.json");
    std::fs::create_dir_all(tmp.parent().unwrap()).map_err(|e| e.to_string())?;
    std::fs::write(&tmp, &text).map_err(|e| e.to_string())?;
    game_rules(&tmp)?;
    let stages: Vec<String> = match crate::opt(args, "--stage") {
        Some(s) => vec![s.to_string()],
        None => c["stages"].as_array().into_iter().flatten().filter_map(|s| s.as_str().map(String::from)).collect(),
    };
    let every = crate::opt(args, "--every").unwrap_or("0.25");
    let mut broken = vec![];
    for stage in &stages {
        let dir = root().join(format!("target/xtask/cutscene/{id}/{}", stage.replace(':', "_")));
        let _ = std::fs::remove_dir_all(&dir);
        let out = std::process::Command::new(root().join("target/release/backrooms"))
            .args(["--cutscene", id, "--stage", stage, "--bundle"])
            .arg(&tmp)
            .args(["--every", every, "--sheet"])
            .arg(&dir)
            .output()
            .map_err(|e| e.to_string())?;
        print!("{}", String::from_utf8_lossy(&out.stdout));
        if !out.status.success() {
            return Err(format!("{id} on {stage}: {}", String::from_utf8_lossy(&out.stderr).trim()));
        }
        let contract = std::fs::read_to_string(dir.join("contract.txt")).unwrap_or_default();
        if contract.trim() != "ok" {
            broken.push(format!("{id} on {stage}:\n{contract}"));
        }
        println!("  look at {}", dir.join("sheet.png").strip_prefix(root()).unwrap_or(&dir).display());
    }
    if broken.is_empty() {
        Ok(())
    } else {
        Err(format!("the contract is broken:\n{}", broken.join("\n")))
    }
}

fn list() -> Result<(), String> {
    let text = compile_all(&src_dir()).map_err(|p| p.join("\n"))?;
    let v: Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
    for c in v["cutscenes"].as_array().into_iter().flatten() {
        let stages: Vec<&str> = c["stages"].as_array().into_iter().flatten().filter_map(Value::as_str).collect();
        println!(
            "{:<14} {:>5.1} s  {:<4}  then {:<4}  stages {:<40}  {}",
            c["id"].as_str().unwrap_or("?"),
            c["length"].as_f64().unwrap_or(0.0),
            c["world"].as_str().unwrap_or("?"),
            c["then"].as_str().unwrap_or("?"),
            stages.join(","),
            c["about"].as_str().unwrap_or("")
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The sample with every field: its compiled form is the game's
    /// fixture (`crates/game/tests/data/cutscene/sample.json`, parsed by the
    /// game's tests), so the format here and the parse there agree.
    /// `REGEN=1 cargo test -p xtask cutscene` writes it again.
    #[test]
    fn the_sample_compiles_to_the_games_fixture() {
        let dir = root().join("crates/game/tests/data/cutscene");
        let text = std::fs::read_to_string(dir.join("sample.toml")).unwrap();
        let v = compile(&text, "sample").unwrap_or_else(|p| panic!("{p:#?}"));
        let bundle = serde_json::to_string_pretty(&json!({ "about": ABOUT, "cutscenes": [v] })).unwrap() + "\n";
        let fixture = dir.join("sample.json");
        if std::env::var("REGEN").is_ok() {
            std::fs::write(&fixture, &bundle).unwrap();
        }
        assert_eq!(std::fs::read_to_string(&fixture).unwrap(), bundle, "REGEN=1 cargo test -p xtask cutscene, after an intended change");
    }

    /// The bundle the game embeds is what its sources compile to: a source
    /// edited and not written fails the fast check, not only `cutscene check`
    #[test]
    fn the_bundle_is_what_the_sources_compile_to() {
        let text = compile_all(&src_dir()).unwrap_or_else(|p| panic!("{p:#?}"));
        let on_disk = std::fs::read_to_string(bundle_path()).unwrap();
        assert!(
            on_disk == text,
            "crates/game/assets/cutscenes.json is not what assets/src/cutscenes compiles to: `cargo xtask cutscene write`"
        );
    }

    #[test]
    fn defaults_are_written_out() {
        let v =
            compile("id = \"a\"\nabout = \"x\"\nlength = 3\nstages = [\"title\"]\n[[camera]]\nt = 0\nanchor = \"start\"\n", "a").unwrap();
        assert_eq!(v["world"], "live");
        assert_eq!(v["breaks_off"], true);
        assert_eq!(v["blend_out"], 1.0);
        assert_eq!(v["bars"], json!({ "in": 0.0, "out": 2.5 }));
        assert_eq!(v["camera"][0]["ease"], "in_out");
        assert_eq!(v["camera"][0]["look"], Value::Null);
        assert_eq!(v["fade"], json!([]));
        let held = compile(
            "id = \"a\"\nabout = \"x\"\nworld = \"held\"\nlength = 3\nstages = [\"title\"]\n[[camera]]\nt = 0\nanchor = \"start\"\n",
            "a",
        )
        .unwrap();
        assert_eq!(held["breaks_off"], false, "a held world has nothing to break it off");
    }

    /// One broken source per kind of fault: each is named
    #[test]
    fn every_fault_is_named() {
        let base = "id = \"a\"\nabout = \"x\"\nlength = 3\nstages = [\"title\"]\n[[camera]]\nt = 0\nanchor = \"start\"\n";
        let cases: &[(&str, &str)] = &[
            (&base.replace("id = \"a\"", "id = \"b\""), "its id must be its file's name"),
            (&base.replace("length = 3\n", ""), "no `length`"),
            (&format!("{base}colour = \"red\"\n"), "unknown field `colour`"),
            (&base.replace("anchor = \"start\"", "anchor = \"floor\""), "`anchor` \"floor\" is not one of"),
            (&base.replace("length = 3", "length = \"long\""), "`length` is not a number"),
            (&format!("{base}pos = [1, 2]\n"), "`pos` is not [x, y, z]"),
            (&base.replace("stages = [\"title\"]", "stages = []"), "`stages` is not a list of words"),
            (&format!("{base}[[event]]\nname = \"x\"\n"), "event 1: no `t`"),
            (&format!("{base}[bars]\nwhen = 1\n"), "bars: unknown field `when`"),
            ("id = ", "a.toml"),
        ];
        for (src, says) in cases {
            let p = compile(src, "a").expect_err(says);
            assert!(p.iter().any(|x| x.contains(says)), "{says}: {p:?}");
        }
    }
}
