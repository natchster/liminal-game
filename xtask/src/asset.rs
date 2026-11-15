//! `cargo xtask asset`: the asset pipeline (`docs/design/ASSETS.md`).
//!
//! ```text
//! cargo xtask asset new NAME      start a dossier, assets/src/props/NAME/, from the templates
//! cargo xtask asset check [NAME]  every dossier (or one) against the format: the brief's
//!                                 sections, each reference's licence class (a free one with its
//!                                 SHA-1, a limited one never committed), each rule's why
//! cargo xtask asset refs NAME     its references fetched into target/asset-refs/NAME/ to look
//!                                 at (a free one checked by its SHA-1)
//! cargo xtask asset write         every dossier's placement rules compiled into
//!                                 crates/game/assets/placement.json and its MANIFEST.json entry
//! cargo xtask asset rules         every asset's rules as one table (for an agent making a place)
//! cargo xtask asset sheet NAME [--seeds 4242,77] [--levels 5,6,7,8,9]
//!                                 frames of it where the game put it (the census), and a
//!                                 questions file, in target/asset-sheet/NAME/: the playtest's
//!                                 "why could this be here?"
//! ```
//!
//! This file knows the *format*; whether a kind or a purpose a rule names
//! exists is the game's to say (`tests/usable.rs`, on the written rules).
use crate::{cargo, root, run};
use serde_json::{Value, json};
use std::path::PathBuf;

const USAGE: &str =
    "usage: cargo xtask asset new NAME | check [NAME] | refs NAME | write | rules | sheet NAME [--seeds S,…] [--levels L,…]";

/// The brief's sections, in order
pub const BRIEF: [&str; 5] = ["What it is", "How it came here", "How it fits", "What a hand does", "Where it would be found"];
/// Where a rule puts a thing
/// Where a rule may put a thing (hung on a wall: not yet, the game refuses it)
pub const ON: [&str; 2] = ["floor", "surface"];
/// The first floor the rules place on (`game::world::placement_rules::FIRST_LEVEL`:
/// the rooms'; the Building's floors place their own)
const FIRST_LEVEL: i64 = 4;
/// Near what, on the floor
pub const NEAR: [&str; 3] = ["wall", "corner", "open"];

pub fn asset(args: &[String]) -> Result<(), String> {
    match args.first().map(String::as_str) {
        Some("new") => new(args.get(1).ok_or(USAGE)?),
        Some("check") => check(args.get(1).map(String::as_str)),
        Some("refs") => refs(args.get(1).ok_or(USAGE)?),
        Some("write") => write(),
        Some("rules") => rules(),
        Some("sheet") => sheet(args.get(1).ok_or(USAGE)?, &args[2..]),
        _ => Err(USAGE.into()),
    }
}

fn src_dir() -> PathBuf {
    root().join("assets/src/props")
}

/// Every dossier's name, sorted
fn names() -> Vec<String> {
    let mut v: Vec<String> = std::fs::read_dir(src_dir())
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.path().join("brief.md").exists() && e.file_name() != "TEMPLATE")
        .filter_map(|e| e.file_name().to_str().map(String::from))
        .collect();
    v.sort();
    v
}

fn read(p: &std::path::Path) -> Result<String, String> {
    std::fs::read_to_string(p).map_err(|e| format!("{}: {e}", p.display()))
}

fn new(name: &str) -> Result<(), String> {
    if name.is_empty() || !name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_') {
        return Err(format!("{name}: a name is lower case letters, digits and _"));
    }
    let dir = src_dir().join(name);
    if dir.exists() {
        return Err(format!("{} exists", dir.display()));
    }
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let t = src_dir().join("TEMPLATE");
    for f in ["brief.md", "references.toml", "placement.toml", "playtest.md"] {
        let s = read(&t.join(f))?.replace("NAME", name);
        std::fs::write(dir.join(f), s).map_err(|e| e.to_string())?;
    }
    println!("started {} (docs/design/ASSETS.md, the skill make-asset)", dir.strip_prefix(root()).unwrap_or(&dir).display());
    Ok(())
}

/// One dossier's problems (empty: it holds)
pub fn problems(name: &str) -> Vec<String> {
    problems_in(&src_dir().join(name), name)
}

/// The problems of the dossier in `dir`
fn problems_in(dir: &std::path::Path, name: &str) -> Vec<String> {
    let mut out = vec![];
    let mut say = |s: String| out.push(format!("{name}: {s}"));
    // the brief: its sections, each with something in it
    match read(&dir.join("brief.md")) {
        Err(e) => say(e),
        Ok(b) => {
            let heads: Vec<&str> = b.lines().filter_map(|l| l.strip_prefix("## ")).map(str::trim).collect();
            if heads != BRIEF {
                say(format!("brief.md's sections are {heads:?}, not {BRIEF:?}"));
            }
            for (k, h) in BRIEF.iter().enumerate() {
                let body = b.split(&format!("## {h}")).nth(1).map(|r| r.split("\n## ").next().unwrap_or("")).unwrap_or("");
                if body.trim().is_empty() || body.contains("TODO") {
                    say(format!("brief.md: section {} ({h}) is empty or TODO", k + 1));
                }
            }
        }
    }
    // the references: each with what it shows and its licence class
    match read(&dir.join("references.toml")).and_then(|s| s.parse::<toml::Table>().map_err(|e| format!("references.toml: {e}"))) {
        Err(e) => say(e),
        Ok(t) => {
            let refs = t.get("ref").and_then(|v| v.as_array()).cloned().unwrap_or_default();
            if refs.is_empty() {
                say("references.toml: no [[ref]]".into());
            }
            for (i, r) in refs.iter().enumerate() {
                let get = |k: &str| r.get(k).and_then(|v| v.as_str()).unwrap_or("");
                for k in ["url", "shows", "licence", "class"] {
                    if get(k).is_empty() {
                        say(format!("references.toml: ref {} has no {k}", i + 1));
                    }
                }
                match get("class") {
                    "free" => {
                        // one that may be sampled (into a texture) is fetched by its checksum
                        let sampled = r.get("sampled").and_then(|v| v.as_bool()).unwrap_or(true);
                        let sha = get("sha1");
                        if sampled && (sha.len() != 40 || !sha.chars().all(|c| c.is_ascii_hexdigit())) {
                            say(format!(
                                "references.toml: ref {} is free and has no SHA-1 (one that may be sampled is fetched by its checksum; `sampled = false` if it never is)",
                                i + 1
                            ));
                        }
                    }
                    "limited" => {}
                    c => say(format!("references.toml: ref {}'s class is {c:?} (free or limited)", i + 1)),
                }
            }
            for (k, what) in [("palette", "ref"), ("measure", "ref")] {
                for (i, p) in t.get(k).and_then(|v| v.as_array()).cloned().unwrap_or_default().iter().enumerate() {
                    let n = p.get(what).and_then(|v| v.as_integer()).unwrap_or(0);
                    if n < 1 || n as usize > refs.len() {
                        say(format!("references.toml: {k} {} names no reference ({what} = {n})", i + 1));
                    }
                }
            }
        }
    }
    // nothing but its four files and the free references' own folder in it
    // (a limited reference is never committed)
    for e in std::fs::read_dir(dir).into_iter().flatten().flatten() {
        let f = e.file_name().to_string_lossy().to_string();
        if !["brief.md", "references.toml", "placement.toml", "playtest.md"].contains(&f.as_str()) {
            say(format!("{f}: a dossier holds its four files; a reference is fetched (`asset refs`), never kept here"));
        }
    }
    // the rules: each with its why
    match read(&dir.join("placement.toml")).and_then(|s| s.parse::<toml::Table>().map_err(|e| format!("placement.toml: {e}"))) {
        Err(e) => say(e),
        Ok(t) => {
            let count = t.get("count").and_then(|v| v.as_array()).map(|a| a.iter().filter_map(|x| x.as_integer()).collect::<Vec<_>>());
            if !matches!(count.as_deref(), Some([a, b]) if 0 <= *a && a <= b) {
                say("placement.toml: count = [least, most] a floor".into());
            }
            let levels = t.get("levels").and_then(|v| v.as_array()).map(|a| a.iter().filter_map(|x| x.as_integer()).collect::<Vec<_>>());
            if !matches!(levels.as_deref(), Some([a, b]) if FIRST_LEVEL <= *a && a <= b) {
                say(format!("placement.toml: levels = [first, last], from {FIRST_LEVEL} down (the rules place on the rooms' floors)"));
            }
            let rules = t.get("rule").and_then(|v| v.as_array()).cloned().unwrap_or_default();
            if rules.is_empty() {
                say("placement.toml: no [[rule]]".into());
            }
            for (i, r) in rules.iter().enumerate() {
                let s = |k: &str| r.get(k).and_then(|v| v.as_str()).unwrap_or("");
                if !ON.contains(&s("on")) {
                    say(format!("placement.toml: rule {}: on = {:?} (one of {ON:?})", i + 1, s("on")));
                }
                if s("on") == "surface" && r.get("of").and_then(|v| v.as_array()).is_none_or(|a| a.is_empty()) {
                    say(format!("placement.toml: rule {}: on a surface, of what (`of`)", i + 1));
                }
                if r.get("dark").is_some_and(|v| v.as_bool().is_none()) {
                    say(format!("placement.toml: rule {}: dark = true or false", i + 1));
                }
                if !s("near").is_empty() && !NEAR.contains(&s("near")) {
                    say(format!("placement.toml: rule {}: near = {:?} (one of {NEAR:?})", i + 1, s("near")));
                }
                if r.get("purposes").and_then(|v| v.as_array()).is_none_or(|a| a.is_empty()) {
                    say(format!("placement.toml: rule {}: no purposes (\"*\" for any)", i + 1));
                }
                if r.get("weight").and_then(|v| v.as_float().or(v.as_integer().map(|i| i as f64))).is_none_or(|w| w <= 0.0) {
                    say(format!("placement.toml: rule {}: weight > 0", i + 1));
                }
                if s("why").trim().len() < 12 {
                    say(format!("placement.toml: rule {}: its why (the story a player could tell)", i + 1));
                }
            }
            if let Some(n) = t.get("never")
                && n.get("why").and_then(|v| v.as_str()).is_none_or(|w| w.trim().len() < 12)
            {
                say("placement.toml: [never] without its why".into());
            }
        }
    }
    match read(&dir.join("playtest.md")) {
        Err(e) => say(e),
        Ok(p) if !p.contains("## ") => say("playtest.md: no sections".into()),
        Ok(_) => {}
    }
    out
}

fn check(one: Option<&str>) -> Result<(), String> {
    let list: Vec<String> = one.map_or_else(names, |n| vec![n.to_string()]);
    let all: Vec<String> = list.iter().flat_map(|n| problems(n)).collect();
    if all.is_empty() {
        println!("{} dossier(s) hold", list.len());
        Ok(())
    } else {
        Err(all.join("\n"))
    }
}

fn refs(name: &str) -> Result<(), String> {
    let t: toml::Table = read(&src_dir().join(name).join("references.toml"))?.parse().map_err(|e| format!("{e}"))?;
    let out = root().join("target/asset-refs").join(name);
    std::fs::create_dir_all(&out).map_err(|e| e.to_string())?;
    let mut bad = vec![];
    for (i, r) in t.get("ref").and_then(|v| v.as_array()).cloned().unwrap_or_default().iter().enumerate() {
        let url = r.get("url").and_then(|v| v.as_str()).unwrap_or("");
        let class = r.get("class").and_then(|v| v.as_str()).unwrap_or("");
        let ext = url.rsplit('.').next().filter(|e| e.len() <= 4 && e.chars().all(char::is_alphanumeric)).unwrap_or("html");
        let f = out.join(format!("{:02}_{class}.{ext}", i + 1));
        let ok = std::process::Command::new("curl")
            .args(["-sfL", "-A", "OpenCarole-asset-refs/1.0 (cargo xtask asset refs)", "-o"])
            .arg(&f)
            .arg(url)
            .status()
            .is_ok_and(|s| s.success());
        if !ok {
            bad.push(format!("ref {}: could not fetch {url}", i + 1));
            continue;
        }
        if class == "free"
            && let Some(want) = r.get("sha1").and_then(|v| v.as_str())
        {
            let got = std::process::Command::new("shasum")
                .arg(&f)
                .output()
                .map(|o| String::from_utf8_lossy(&o.stdout).split_whitespace().next().unwrap_or("").to_string())
                .unwrap_or_default();
            if got != want {
                bad.push(format!("ref {}: SHA-1 {got}, not {want}", i + 1));
            }
        }
    }
    println!("fetched into {}", out.strip_prefix(root()).unwrap_or(&out).display());
    if bad.is_empty() { Ok(()) } else { Err(bad.join("\n")) }
}

/// Every dossier's rules as the game reads them
fn compile() -> Result<Value, String> {
    let mut assets = serde_json::Map::new();
    let mut probs = vec![];
    for n in names() {
        probs.extend(problems(&n));
        let t: toml::Table = read(&src_dir().join(&n).join("placement.toml"))?.parse().map_err(|e| format!("{n}: {e}"))?;
        let v = serde_json::to_value(&t).map_err(|e| e.to_string())?;
        assets.insert(n, v);
    }
    if !probs.is_empty() {
        return Err(probs.join("\n"));
    }
    Ok(json!({
        "about": "Where each asset may lie and why, compiled by `cargo xtask asset write` from assets/src/props/*/placement.toml. Do not edit: change the source and write it again (docs/design/ASSETS.md).",
        "assets": assets,
    }))
}

fn write() -> Result<(), String> {
    let v = compile()?;
    let text = serde_json::to_string_pretty(&v).map_err(|e| e.to_string())? + "\n";
    let dest = root().join("crates/game/assets/placement.json");
    std::fs::write(&dest, &text).map_err(|e| e.to_string())?;
    let sources: Vec<String> = names().iter().map(|n| format!("assets/src/props/{n}/placement.toml")).collect();
    crate::assets::update_manifest(
        "placement.json",
        text.as_bytes(),
        json!({
            "authors": "written by coding agents under the developer's direction",
            "built_with": "cargo xtask asset write (TOML sources, every field written out)",
            "generative_model": null,
            "generator": "written by hand, compiled by scripts",
            "licence": "MIT",
            "rebuild": "cargo xtask asset write",
            "sources": sources,
        }),
    )?;
    println!("wrote crates/game/assets/placement.json ({} assets) and its manifest entry", sources.len());
    Ok(())
}

fn rules() -> Result<(), String> {
    let v = compile()?;
    println!("| asset | count | on | of / near | purposes | weight | why |");
    println!("|---|---|---|---|---|---|---|");
    for (n, a) in v["assets"].as_object().into_iter().flatten() {
        for r in a["rule"].as_array().into_iter().flatten() {
            let list =
                |k: &str| r[k].as_array().map(|a| a.iter().filter_map(|x| x.as_str()).collect::<Vec<_>>().join(", ")).unwrap_or_default();
            let of = if r["on"] == "surface" { list("of") } else { r["near"].as_str().unwrap_or("").to_string() };
            println!(
                "| {n} | {} | {} | {of} | {} | {} | {} |",
                a["count"],
                r["on"].as_str().unwrap_or(""),
                list("purposes"),
                r["weight"],
                r["why"].as_str().unwrap_or("")
            );
        }
        if let Some(nv) = a.get("never") {
            println!("| {n} | | never | | {} | | {} |", nv["purposes"], nv["why"].as_str().unwrap_or(""));
        }
    }
    Ok(())
}

/// Frames of NAME where the game put it, from the census (`--prop-census`,
/// `--census-only core_NAME`), and the playtester's questions file
fn sheet(name: &str, args: &[String]) -> Result<(), String> {
    let seeds = crate::opt(args, "--seeds").unwrap_or("4242,77,202");
    let levels = crate::opt(args, "--levels").unwrap_or("5,6,7,8,9");
    run(cargo().args(["build", "--release", "-p", "backrooms"]))?;
    let out = root().join("target/asset-sheet").join(name);
    let _ = std::fs::remove_dir_all(&out);
    std::fs::create_dir_all(&out).map_err(|e| e.to_string())?;
    let mut frames = vec![];
    for s in seeds.split(',') {
        let d = out.join(format!("s{s}"));
        run(std::process::Command::new(root().join("target/release/backrooms")).args([
            "--dump-level",
            d.to_str().unwrap_or(""),
            "--seed",
            s,
            "--levels",
            levels,
            "--screenshot",
            "x",
            "--prop-census",
            "3",
            "--census-only",
            &format!("core_{name}"),
            "--width",
            "960",
            "--height",
            "600",
        ]))?;
        let mut f: Vec<PathBuf> = std::fs::read_dir(d.join("census"))
            .into_iter()
            .flatten()
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|x| x == "png"))
            .collect();
        f.sort();
        frames.extend(f);
    }
    let mut q = format!(
        "# Why could this be here?\n\nThe thing: **{}**. Each frame (`frameNN.png`) shows it where the game put it, framed as a player near it would see it.\n\nFor each frame: what you see, then *why could this be here?* (a person's reason, none, or the game's), and how sure (`obvious`, `could be`, `no idea`). End with JSON: `{{\"frame01\": {{\"why\": \"...\", \"sure\": \"obvious\"}}, ...}}`.\n\nFrames:",
        name.replace('_', " ")
    );
    for (i, f) in frames.iter().enumerate() {
        let to = out.join(format!("frame{:02}.png", i + 1));
        std::fs::copy(f, &to).map_err(|e| e.to_string())?;
        q.push_str(&format!(" frame{:02}", i + 1));
    }
    std::fs::write(out.join("questions.md"), q + "\n").map_err(|e| e.to_string())?;
    println!(
        "{} frames in {} (give a fresh placement-critic only that folder)",
        frames.len(),
        out.strip_prefix(root()).unwrap_or(&out).display()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every dossier in the tree holds
    #[test]
    fn the_dossiers_hold() {
        let bad: Vec<String> = names().iter().flat_map(|n| problems(n)).collect();
        assert!(bad.is_empty(), "{bad:#?}");
    }

    /// The game reads what the sources say: the bundle is the sources
    /// compiled (a `placement.toml` changed without `cargo xtask asset
    /// write` would be checked here and ignored there)
    #[test]
    fn the_bundle_is_its_sources() {
        let want = serde_json::to_string_pretty(&compile().expect("the dossiers compile")).unwrap() + "\n";
        let have = std::fs::read_to_string(root().join("crates/game/assets/placement.json")).expect("the bundle");
        assert!(want == have, "crates/game/assets/placement.json is not its sources: cargo xtask asset write");
    }

    /// A dossier just started does not hold until it is written: the
    /// template's every section, reference and rule is found wanting
    #[test]
    fn a_new_dossier_holds_only_once_written() {
        let t = src_dir().join("TEMPLATE");
        let p = problems_in(&t, "TEMPLATE");
        for want in ["section 1 (What it is) is empty or TODO", "is free and has no SHA-1", "rule 1: its why"] {
            assert!(p.iter().any(|x| x.contains(want)), "the template passes `{want}`: {p:#?}");
        }
    }
}
