//! `cargo xtask voice …`: the voice-over pipeline (`docs/VOICES.md`).
//!
//! The models run in Python environments of their own (`tools/voice/synth`
//! for synthesis on MLX, `tools/voice/eval` for the checks), each locked with
//! `uv`; this module runs them, and checks what they need is in place.
//!
//! ```text
//! cargo xtask voice doctor        uv, the environments, the models at their revisions, the licences
//! cargo xtask voice fetch [--only ID,…]   the models into the Hugging Face cache
//! cargo xtask voice reference|build|eval|write …   the pipeline (`pipeline.rs`)
//! ```
mod pipeline;

use crate::{flag, opt, root};
use std::path::{Path, PathBuf};
use std::process::Command;

const USAGE: &str = "usage: cargo xtask voice <doctor|fetch|reference|build|eval|selftest|write> …
  doctor [--smoke]          uv, the two environments, every model in the cache at its
                            pinned revision, each synthesis model's licence kept and
                            allowed in MANIFEST.json; --smoke also says a sentence and
                            hears it back
  fetch [--only ID,…]       download the models of tools/voice/models.toml
  reference --persona P [--candidates N] [--write]
                            candidates for P's voice from its description (seeds
                            design_seed on), each heard back; --write keeps the
                            design_seed one as assets/src/voice/personas/P.wav
  build --persona P [--model M] [--only LINE,…]
                            every take of P's lines, raw then cleaned, into
                            target/voice/P/M/<line>/take<k>.wav
  eval --persona P [--model M]
                            every take judged (words, emotion, tone of voice, voice);
                            each line's best passing take chosen; the gate:
                            target/voice/P/M/report.md
  selftest [--persona you]  the evaluation against known answers: wrong words, a calm
                            read of a terrified line, two other voices must fail
                            their check; three good takes must pass
  write --persona P [--model M] [--take K]
                            each line's chosen take (the evaluation's, or take K)
                            into crates/game/assets/voice/P/ and MANIFEST.json";

pub fn voice(args: &[String]) -> Result<(), String> {
    let rest = args.get(1..).unwrap_or(&[]);
    match args.first().map(String::as_str) {
        Some("doctor") => doctor(rest),
        Some("fetch") => fetch(rest),
        Some("reference") => pipeline::reference(rest),
        Some("build") => pipeline::build(rest),
        Some("write") => pipeline::write(rest),
        Some("eval") => pipeline::eval(rest),
        Some("selftest") => pipeline::selftest(rest),
        _ => Err(USAGE.into()),
    }
}

/// A model of `tools/voice/models.toml`
#[derive(Debug, Clone)]
pub struct Model {
    pub id: String,
    pub role: String,
    pub env: String,
    pub repo: String,
    pub revision: String,
    pub include: Vec<String>,
    pub upstream: Option<String>,
    pub licence: String,
}

fn tools() -> PathBuf {
    root().join("tools/voice")
}

fn s(t: &toml::Table, k: &str) -> Option<String> {
    t.get(k).and_then(|v| v.as_str()).map(String::from)
}

/// The models, read from `tools/voice/models.toml`
pub fn models() -> Result<Vec<Model>, String> {
    let path = tools().join("models.toml");
    let text = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let t: toml::Table = text.parse().map_err(|e| format!("{}: {e}", path.display()))?;
    let list = t.get("model").and_then(|v| v.as_array()).ok_or("models.toml: no [[model]]")?;
    list.iter()
        .map(|m| {
            let m = m.as_table().ok_or("models.toml: a [[model]] that is not a table")?;
            let need = |k: &str| s(m, k).ok_or_else(|| format!("models.toml: a model without {k}"));
            Ok(Model {
                id: need("id")?,
                role: need("role")?,
                env: need("env")?,
                repo: need("repo")?,
                revision: need("revision")?,
                include: m
                    .get("include")
                    .and_then(|v| v.as_array())
                    .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                    .unwrap_or_default(),
                upstream: s(m, "upstream"),
                licence: need("licence")?,
            })
        })
        .collect()
}

/// `uv run` in environment `env` (`synth` or `eval`), the lock as it is
pub fn uv_run(env: &str) -> Command {
    let mut c = Command::new("uv");
    c.args(["run", "--locked", "--project"]).arg(tools().join(env)).arg("python").current_dir(root());
    c
}

/// The Hugging Face cache (`HF_HUB_CACHE`, `HF_HOME/hub`, or `~/.cache/huggingface/hub`)
fn hf_cache() -> PathBuf {
    if let Some(p) = std::env::var_os("HF_HUB_CACHE") {
        return PathBuf::from(p);
    }
    if let Some(p) = std::env::var_os("HF_HOME") {
        return PathBuf::from(p).join("hub");
    }
    PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(".cache/huggingface/hub")
}

/// Where `m` is in the cache at its revision, if it is
pub fn snapshot(m: &Model) -> Option<PathBuf> {
    let dir = hf_cache().join(format!("models--{}", m.repo.replace('/', "--"))).join("snapshots").join(&m.revision);
    dir.is_dir().then_some(dir)
}

/// Does a file name match an `include` pattern (`*` any run of characters,
/// within a directory or across, as Hugging Face's `allow_patterns`)
fn glob(pat: &str, name: &str) -> bool {
    match pat.split_once('*') {
        None => pat == name,
        Some((head, tail)) => name.starts_with(head) && (0..=name.len() - head.len()).any(|i| glob(tail, &name[head.len() + i..])),
    }
}

fn files_under(dir: &Path, base: &Path, out: &mut Vec<String>) {
    for e in std::fs::read_dir(dir).into_iter().flatten().flatten() {
        let p = e.path();
        if p.is_dir() {
            files_under(&p, base, out);
        } else if let Ok(rel) = p.strip_prefix(base) {
            out.push(rel.to_string_lossy().into_owned());
        }
    }
}

fn fetch(args: &[String]) -> Result<(), String> {
    let mut c = uv_run("synth");
    c.arg("-u").arg(tools().join("fetch.py"));
    if let Some(only) = opt(args, "--only") {
        c.args(["--only", only]);
    }
    crate::run(&mut c)
}

struct Report {
    failed: usize,
}

impl Report {
    fn ok(&self, what: &str) {
        println!("ok    {what}");
    }
    fn fail(&mut self, what: &str) {
        println!("FAIL  {what}");
        self.failed += 1;
    }
    fn check(&mut self, ok: bool, what: &str) {
        if ok {
            self.ok(what)
        } else {
            self.fail(what)
        }
    }
}

fn doctor(args: &[String]) -> Result<(), String> {
    let mut r = Report { failed: 0 };
    // uv
    match Command::new("uv").arg("--version").output() {
        Ok(o) if o.status.success() => r.ok(String::from_utf8_lossy(&o.stdout).trim()),
        _ => r.fail("uv is not on PATH (https://docs.astral.sh/uv/: `brew install uv`)"),
    }
    // the environments: in step with their locks, and their main packages import
    for (env, probe) in [
        ("synth", "import mlx, mlx_audio"),
        ("eval", "import mlx_whisper, parakeet_mlx, jiwer, torch, transformers, speechbrain, funasr, parselmouth, librosa, pyloudnorm"),
    ] {
        let o = uv_run(env).args(["-c", probe]).output();
        match o {
            Ok(o) if o.status.success() => r.ok(&format!("tools/voice/{env}: synced, imports")),
            Ok(o) => {
                let err = String::from_utf8_lossy(&o.stderr);
                let last = err.lines().rev().find(|l| !l.trim().is_empty()).unwrap_or("").trim().to_string();
                r.fail(&format!("tools/voice/{env}: {last} (`uv sync --project tools/voice/{env}`)"))
            }
            Err(e) => r.fail(&format!("tools/voice/{env}: {e}")),
        }
    }
    // the personas and their lines
    let problems = check_sources();
    if problems.is_empty() {
        r.ok(&format!("assets/src/voice: {} persona(s), their lines complete", stems(&voice_src().join("personas")).len()));
    } else {
        for p in problems {
            r.fail(&p);
        }
    }
    // the models, at their revisions, with every file they need
    let models = models()?;
    let manifest: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(root().join("crates/game/assets/MANIFEST.json")).map_err(|e| e.to_string())?)
            .map_err(|e| e.to_string())?;
    let allowed: Vec<&str> = manifest["allowed_models"].as_array().into_iter().flatten().filter_map(|v| v.as_str()).collect();
    for m in &models {
        let Some(dir) = snapshot(m) else {
            r.fail(&format!("{}: not in the cache at {} (`cargo xtask voice fetch --only {}`)", m.id, &m.revision[..12], m.id));
            continue;
        };
        let mut files = vec![];
        files_under(&dir, &dir, &mut files);
        let unmatched: Vec<&String> = m.include.iter().filter(|p| !files.iter().any(|f| glob(p, f))).collect();
        // a pattern may match nothing in a repository that has no such file
        // (a LICENSE); the weights must be there
        let is_weights = |f: &str| [".safetensors", ".pt", ".bin", ".ckpt"].iter().any(|x| f.ends_with(x));
        let weights = !m.include.iter().any(|p| is_weights(p)) || files.iter().any(|f| is_weights(f));
        r.check(
            weights,
            &format!(
                "{} ({}): {}@{} in the cache ({} files{})",
                m.id,
                m.env,
                m.repo,
                &m.revision[..12],
                files.len(),
                if unmatched.is_empty() { String::new() } else { format!("; nothing for {unmatched:?}") }
            ),
        );
        if m.role == "synth" {
            let up = m.upstream.clone().unwrap_or_else(|| m.repo.clone());
            let kept = root().join("assets/src/voice/models").join(&m.id);
            let has_licence = ["LICENSE", "LICENSE.txt", "LICENSE.md", "README.md"].iter().any(|f| kept.join(f).is_file())
                && kept.join("SOURCE.toml").is_file();
            r.check(has_licence, &format!("{}: its licence ({}) and card kept in assets/src/voice/models/{}/", m.id, m.licence, m.id));
            r.check(allowed.contains(&up.as_str()), &format!("{}: {up} in MANIFEST.json's allowed_models", m.id));
            r.check(
                ["Apache-2.0", "MIT"].contains(&m.licence.as_str()),
                &format!("{}: licence {} allows selling what it makes", m.id, m.licence),
            );
        }
    }
    if flag(args, "--smoke") && r.failed == 0 {
        smoke(&mut r)?;
    }
    if r.failed > 0 {
        return Err(format!("voice doctor: {} problem(s)", r.failed));
    }
    println!("voice doctor: all well");
    Ok(())
}

/// One sentence said by the first synthesis model and heard back
fn smoke(r: &mut Report) -> Result<(), String> {
    let out = root().join("target/voice/smoke");
    std::fs::create_dir_all(&out).map_err(|e| e.to_string())?;
    let text = "Is anybody there? I can hear the lights humming.";
    let wav = out.join("smoke.wav");
    let o = uv_run("synth")
        .arg(tools().join("synth.py"))
        .args(["say", "--model", "qwen3-tts-voicedesign", "--seed", "1", "--text", text])
        .args(["--instruct", "A man in his thirties, plain American voice, speaking calmly."])
        .arg("--out")
        .arg(&wav)
        .output()
        .map_err(|e| e.to_string())?;
    if !o.status.success() {
        r.fail(&format!("smoke: synthesis failed: {}", String::from_utf8_lossy(&o.stderr).lines().last().unwrap_or("")));
        return Ok(());
    }
    let o = uv_run("eval").arg(tools().join("evaluate.py")).args(["hear", "--text", text]).arg(&wav).output().map_err(|e| e.to_string())?;
    let said = String::from_utf8_lossy(&o.stdout);
    let wer = said.lines().find_map(|l| l.strip_prefix("wer ")).and_then(|v| v.trim().parse::<f64>().ok());
    r.check(
        o.status.success() && wer.is_some_and(|w| w <= 0.15),
        &format!("smoke: said and heard back ({}); {}", wav.display(), said.trim().replace('\n', "; ")),
    );
    Ok(())
}

// ------------------------------------------------------------ personas and lines

/// What a field must hold
#[derive(Clone, Copy, PartialEq)]
enum Kind {
    Str,
    Int,
    Num,
    StrList,
    NumPair,
    IntPair,
    /// a table of names to non-empty strings (the emotions)
    Words,
    Table,
}

/// A persona (`assets/src/voice/personas/<id>.toml`): each field, what it
/// holds, whether it must be there
const PERSONA: &[(&str, Kind, bool)] = &[
    ("id", Kind::Str, true),
    ("name", Kind::Str, true),
    ("used_for", Kind::Str, true),
    ("person", Kind::Table, true),
    ("person.summary", Kind::Str, true),
    ("person.age", Kind::Int, true),
    ("person.sex", Kind::Str, true),
    ("person.from", Kind::Str, true),
    ("person.here", Kind::Str, true),
    ("person.never_said", Kind::Str, true),
    ("voice", Kind::Table, true),
    ("voice.description", Kind::Str, true),
    ("voice.reference_text", Kind::Str, true),
    ("voice.design_model", Kind::Str, true),
    ("voice.design_seed", Kind::Int, true),
    ("voice.line_model", Kind::Str, true),
    ("tone", Kind::Table, true),
    ("tone.direction", Kind::StrList, true),
    ("tone.like", Kind::StrList, true),
    ("tone.never", Kind::StrList, true),
    ("emotions", Kind::Words, true),
    ("targets", Kind::Table, true),
    ("targets.f0_median_hz", Kind::NumPair, true),
    ("targets.loudness_lufs", Kind::Num, true),
    ("targets.peak_dbtp", Kind::Num, true),
    ("targets.wer_max", Kind::Num, true),
    ("targets.similarity_min", Kind::Num, true),
    ("targets.similarity_min_loud", Kind::Num, true),
];

/// A line (`[[line]]` of `assets/src/voice/lines/<persona>.toml`)
const LINE: &[(&str, Kind, bool)] = &[
    ("id", Kind::Str, true),
    ("text", Kind::Str, true),
    ("emotion", Kind::Str, true),
    ("intensity", Kind::Num, true),
    ("use", Kind::Str, true),
    ("levels", Kind::IntPair, true),
    ("takes", Kind::Int, true),
    // the words a model is directed with, when the emotion's are not enough
    ("direction", Kind::Str, false),
];

/// Where the game plays a voice line (`use`)
pub const USES: &[&str] = &["cry"];

fn kind_ok(v: &toml::Value, k: Kind) -> bool {
    let num = |v: &toml::Value| v.as_float().is_some() || v.as_integer().is_some();
    match k {
        Kind::Str => v.as_str().is_some_and(|s| !s.trim().is_empty()),
        Kind::Int => v.as_integer().is_some(),
        Kind::Num => num(v),
        Kind::StrList => v.as_array().is_some_and(|a| !a.is_empty() && a.iter().all(|x| x.as_str().is_some_and(|s| !s.trim().is_empty()))),
        Kind::NumPair => v.as_array().is_some_and(|a| a.len() == 2 && a.iter().all(num)),
        Kind::IntPair => v.as_array().is_some_and(|a| a.len() == 2 && a.iter().all(|x| x.as_integer().is_some())),
        Kind::Words => v.as_table().is_some_and(|t| !t.is_empty() && t.values().all(|x| x.as_str().is_some_and(|s| !s.trim().is_empty()))),
        Kind::Table => v.as_table().is_some(),
    }
}

/// Every field of `t` against `spec`: missing, of the wrong kind, or unknown
fn check_fields(t: &toml::Table, spec: &[(&str, Kind, bool)], what: &str, problems: &mut Vec<String>) {
    for &(path, kind, required) in spec {
        let mut v = None;
        let mut here = Some(t);
        for (i, part) in path.split('.').enumerate() {
            let got = here.and_then(|h| h.get(part));
            if i == path.split('.').count() - 1 {
                v = got;
            } else {
                here = got.and_then(|g| g.as_table());
            }
        }
        match v {
            None if required => problems.push(format!("{what}: no {path}")),
            None => {}
            Some(v) if !kind_ok(v, kind) => problems.push(format!(
                "{what}: {path} is not {}",
                match kind {
                    Kind::Str => "a non-empty string",
                    Kind::Int => "an integer",
                    Kind::Num => "a number",
                    Kind::StrList => "a list of non-empty strings",
                    Kind::NumPair => "a pair of numbers",
                    Kind::IntPair => "a pair of integers",
                    Kind::Words => "a table of names to non-empty strings",
                    Kind::Table => "a table",
                }
            )),
            Some(_) => {}
        }
    }
    // unknown fields, at the top and in each table the spec describes
    let known = |prefix: &str, k: &str| {
        let full = if prefix.is_empty() { k.to_string() } else { format!("{prefix}.{k}") };
        spec.iter().any(|(p, _, _)| *p == full)
    };
    for (k, v) in t {
        if !known("", k) {
            problems.push(format!("{what}: unknown field {k}"));
        } else if spec.iter().any(|(p, kind, _)| p == k && *kind == Kind::Table) {
            for k2 in v.as_table().into_iter().flat_map(|t| t.keys()) {
                if !known(k, k2) {
                    problems.push(format!("{what}: unknown field {k}.{k2}"));
                }
            }
        }
    }
}

fn read_toml(path: &Path) -> Result<toml::Table, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    text.parse().map_err(|e| format!("{}: {e}", path.display()))
}

fn voice_src() -> PathBuf {
    root().join("assets/src/voice")
}

fn stems(dir: &Path) -> Vec<(String, PathBuf)> {
    let mut v: Vec<(String, PathBuf)> = std::fs::read_dir(dir)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "toml"))
        .map(|p| (p.file_stem().unwrap().to_string_lossy().into_owned(), p))
        .collect();
    v.sort();
    v
}

fn num(v: Option<&toml::Value>) -> f64 {
    v.and_then(|v| v.as_float().or_else(|| v.as_integer().map(|i| i as f64))).unwrap_or(f64::NAN)
}

/// Every persona and line file under `assets/src/voice`: what is wrong with
/// each (nothing, when all is well)
pub fn check_sources() -> Vec<String> {
    let mut problems = vec![];
    let synth: Vec<String> = models().unwrap_or_default().into_iter().filter(|m| m.role == "synth").map(|m| m.id).collect();
    let mut emotions_of = std::collections::HashMap::new();
    for (stem, path) in stems(&voice_src().join("personas")) {
        let what = format!("personas/{stem}.toml");
        let t = match read_toml(&path) {
            Ok(t) => t,
            Err(e) => {
                problems.push(e);
                continue;
            }
        };
        check_fields(&t, PERSONA, &what, &mut problems);
        if s(&t, "id").as_deref() != Some(stem.as_str()) {
            problems.push(format!("{what}: id must be its file's name, {stem}"));
        }
        let voice = t.get("voice").and_then(|v| v.as_table());
        for k in ["design_model", "line_model"] {
            if let Some(m) = voice.and_then(|v| s(v, k)) {
                if !synth.contains(&m) {
                    problems.push(format!("{what}: voice.{k} {m} is not a synthesis model of tools/voice/models.toml"));
                }
            }
        }
        let targets = t.get("targets").and_then(|v| v.as_table());
        let tg = |k: &str| num(targets.and_then(|t| t.get(k)));
        let f0 = targets.and_then(|t| t.get("f0_median_hz")).and_then(|v| v.as_array()).map(|a| (num(a.first()), num(a.get(1))));
        if f0.is_some_and(|(lo, hi)| !(lo > 40.0 && lo < hi && hi < 1000.0)) {
            problems.push(format!("{what}: targets.f0_median_hz must be a low and a higher pitch in Hz"));
        }
        // (a range's `contains` is false for a missing value, NaN, too)
        if !(-40.0..-6.0).contains(&tg("loudness_lufs")) || !(-60.0..=0.0).contains(&tg("peak_dbtp")) {
            problems.push(format!("{what}: targets.loudness_lufs (-40 to -6) or peak_dbtp (at most 0) out of range"));
        }
        let fraction = |v: f64| v > 0.0 && v < 1.0;
        if !fraction(tg("wer_max"))
            || !fraction(tg("similarity_min"))
            || !fraction(tg("similarity_min_loud"))
            || tg("similarity_min_loud") > tg("similarity_min")
        {
            problems
                .push(format!("{what}: targets.wer_max, similarity_min and similarity_min_loud (at most similarity_min) are fractions"));
        }
        let emotions: Vec<String> = t.get("emotions").and_then(|v| v.as_table()).map(|e| e.keys().cloned().collect()).unwrap_or_default();
        emotions_of.insert(stem.clone(), emotions);
    }
    for (stem, path) in stems(&voice_src().join("lines")) {
        let what = format!("lines/{stem}.toml");
        let t = match read_toml(&path) {
            Ok(t) => t,
            Err(e) => {
                problems.push(e);
                continue;
            }
        };
        if s(&t, "persona").as_deref() != Some(stem.as_str()) {
            problems.push(format!("{what}: persona must be its file's name, {stem}"));
        }
        let Some(emotions) = emotions_of.get(&stem) else {
            problems.push(format!("{what}: no persona {stem} (personas/{stem}.toml)"));
            continue;
        };
        for k in t.keys().filter(|k| *k != "persona" && *k != "line") {
            problems.push(format!("{what}: unknown field {k}"));
        }
        let lines = t.get("line").and_then(|v| v.as_array()).cloned().unwrap_or_default();
        if lines.is_empty() {
            problems.push(format!("{what}: no [[line]]"));
        }
        let mut ids = std::collections::HashSet::new();
        for (i, l) in lines.iter().enumerate() {
            let Some(l) = l.as_table() else { continue };
            let id = s(l, "id").unwrap_or_else(|| format!("#{}", i + 1));
            let what = format!("{what} line {id}");
            check_fields(l, LINE, &what, &mut problems);
            if !id.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_') {
                problems.push(format!("{what}: an id is lower case, digits and _"));
            }
            if !ids.insert(id.clone()) {
                problems.push(format!("{what}: a second line with this id"));
            }
            if let Some(e) = s(l, "emotion") {
                if !emotions.contains(&e) {
                    problems.push(format!("{what}: emotion {e} is not one of the persona's ({})", emotions.join(", ")));
                }
            }
            if !(0.0..=1.0).contains(&num(l.get("intensity"))) {
                problems.push(format!("{what}: intensity is 0 to 1"));
            }
            if let Some(u) = s(l, "use") {
                if !USES.contains(&u.as_str()) {
                    problems.push(format!("{what}: use {u} is not one the game plays ({USES:?})"));
                }
            }
            let lv: Vec<i64> =
                l.get("levels").and_then(|v| v.as_array()).map(|a| a.iter().filter_map(|x| x.as_integer()).collect()).unwrap_or_default();
            if lv.len() == 2 && !(1 <= lv[0] && lv[0] <= lv[1]) {
                problems.push(format!("{what}: levels is a first and a last level, from 1"));
            }
            if !l.get("takes").and_then(|v| v.as_integer()).is_some_and(|n| (1..=16).contains(&n)) {
                problems.push(format!("{what}: takes is 1 to 16"));
            }
        }
    }
    problems
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every persona and line file is complete and consistent (V2)
    #[test]
    fn every_persona_and_line_file_is_complete() {
        assert!(!stems(&voice_src().join("personas")).is_empty(), "no persona");
        let problems = check_sources();
        assert!(problems.is_empty(), "{problems:#?}");
    }

    /// The checker finds what is wrong: a missing field, a wrong kind, an
    /// unknown field
    #[test]
    fn the_checker_finds_a_missing_a_mistyped_and_an_unknown_field() {
        let t: toml::Table =
            "id = \"x\"\ntext = 3\nemotion = \"fear\"\nintensity = 0.5\nuse = \"cry\"\nlevels = [1, 3]\nmood = \"?\"".parse().unwrap();
        let mut p = vec![];
        check_fields(&t, LINE, "line", &mut p);
        assert!(p.iter().any(|e| e.contains("no takes")), "{p:?}");
        assert!(p.iter().any(|e| e.contains("text is not")), "{p:?}");
        assert!(p.iter().any(|e| e.contains("unknown field mood")), "{p:?}");
        let t: toml::Table = "[voice]\ndescription = \"a\"\ncolour = \"b\"".parse().unwrap();
        let mut p = vec![];
        check_fields(&t, PERSONA, "persona", &mut p);
        assert!(p.iter().any(|e| e.contains("unknown field voice.colour")), "{p:?}");
        assert!(p.iter().any(|e| e.contains("no voice.reference_text")), "{p:?}");
    }

    #[test]
    fn an_include_pattern_matches_as_hugging_face_does() {
        assert!(glob("*.safetensors", "model.safetensors"));
        assert!(glob("speech_tokenizer/*", "speech_tokenizer/model.safetensors"));
        assert!(glob("tokenizer.*", "tokenizer.model"));
        assert!(!glob("*.json", "model.safetensors"));
        assert!(glob("LICENSE", "LICENSE") && !glob("LICENSE", "LICENSE.md"));
    }

    #[test]
    fn every_model_is_pinned_and_a_shipping_one_is_licensed_for_sale() {
        let ms = models().unwrap();
        assert!(ms.iter().any(|m| m.role == "synth") && ms.iter().any(|m| m.role == "eval"));
        for m in &ms {
            assert_eq!(m.revision.len(), 40, "{}: pin a full revision, not {}", m.id, m.revision);
            assert!(
                ["synth", "eval"].contains(&m.role.as_str()) && [m.role.as_str()].contains(&m.env.as_str()),
                "{}: role {} env {}",
                m.id,
                m.role,
                m.env
            );
            assert!(!m.include.is_empty(), "{}: say which files it needs", m.id);
            if m.role == "synth" {
                assert!(
                    ["Apache-2.0", "MIT"].contains(&m.licence.as_str()),
                    "{}: {} is not a licence that allows selling what it makes",
                    m.id,
                    m.licence
                );
                assert!(m.upstream.is_some(), "{}: name the original model, whose licence governs", m.id);
            }
        }
    }
}
