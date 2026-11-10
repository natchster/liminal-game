//! The pipeline (`docs/VOICES.md`, V3): a persona's reference voice, its
//! lines' takes, and the chosen takes into the game's assets.
//!
//! ```text
//! cargo xtask voice reference --persona P [--candidates N] [--write]
//! cargo xtask voice build --persona P [--model M] [--only LINE,…]
//! cargo xtask voice write --persona P [--model M] [--take K]
//! ```
//!
//! Everything made goes under `target/voice/<persona>/` until `--write`:
//! the reference into `assets/src/voice/personas/<persona>.wav` (the voice
//! every line is cloned from), the chosen takes into
//! `crates/game/assets/voice/<persona>/<line>.wav` with their entries in
//! `MANIFEST.json`. A take is made again the same: the model at its pinned
//! revision, the take's seed its number.
use super::{models, read_toml, s, tools, uv_run, voice_src};
use crate::{flag, opt, root};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

/// A persona and its lines, as written
pub struct Persona {
    pub id: String,
    pub t: toml::Table,
    pub lines: Vec<toml::Table>,
    /// The persona whose voice it speaks with (`voice.same_as`), as written
    pub base: Option<toml::Table>,
}

impl Persona {
    pub fn load(id: &str) -> Result<Persona, String> {
        let problems = super::check_sources();
        if !problems.is_empty() {
            return Err(format!("the persona and line files are not right:\n  {}", problems.join("\n  ")));
        }
        let t = read_toml(&voice_src().join(format!("personas/{id}.toml")))?;
        let lt = read_toml(&voice_src().join(format!("lines/{id}.toml")))?;
        let lines = lt.get("line").and_then(|v| v.as_array()).into_iter().flatten().filter_map(|l| l.as_table().cloned()).collect();
        let same_as = t.get("voice").and_then(|v| v.as_table()).and_then(|v| s(v, "same_as"));
        let base = match same_as {
            Some(other) => Some(read_toml(&voice_src().join(format!("personas/{other}.toml")))?),
            None => None,
        };
        Ok(Persona { id: id.to_string(), t, lines, base })
    }
    fn table(&self, k: &str) -> toml::Table {
        self.t.get(k).and_then(|v| v.as_table()).cloned().unwrap_or_default()
    }
    /// A field of its voice: its own, else (a persona with another's voice)
    /// that one's
    pub fn voice(&self, k: &str) -> String {
        s(&self.table("voice"), k)
            .or_else(|| self.base.as_ref().and_then(|b| b.get("voice")).and_then(|v| v.as_table()).and_then(|v| s(v, k)))
            .unwrap_or_default()
    }
    /// Whose voice it is: its own id, or the persona it speaks like
    /// (`voice.same_as`)
    pub fn voice_of(&self) -> String {
        s(&self.table("voice"), "same_as").unwrap_or_else(|| self.id.clone())
    }
    pub fn target(&self, k: &str) -> f64 {
        super::num(self.table("targets").get(k))
    }
    pub fn emotion_words(&self, e: &str) -> String {
        s(&self.table("emotions"), e).unwrap_or_default()
    }
    /// The reference clip every line is cloned from (kept with the sources):
    /// its own, or the one of the persona whose voice it is
    pub fn reference(&self) -> PathBuf {
        voice_src().join(format!("personas/{}.wav", self.voice_of()))
    }
    /// An emotion's own reference, if the persona has one
    pub fn emotion_ref(&self, emotion: &str) -> Option<(PathBuf, toml::Table)> {
        let r = self.table("references").get(emotion).and_then(|v| v.as_table()).cloned()?;
        Some((voice_src().join(format!("personas/{}.{emotion}.wav", self.id)), r))
    }
    /// What a line of `emotion` is cloned from: its emotion's reference if
    /// there is one, else the persona's; and what that clip says
    pub fn clone_from(&self, emotion: &str) -> (PathBuf, String) {
        match self.emotion_ref(emotion) {
            Some((path, r)) if path.is_file() => (path, s(&r, "text").unwrap_or_default()),
            _ => (self.reference(), self.voice("reference_text")),
        }
    }
    pub fn work(&self) -> PathBuf {
        root().join("target/voice").join(&self.id)
    }
}

fn persona_arg(args: &[String]) -> Result<Persona, String> {
    Persona::load(opt(args, "--persona").ok_or("say which persona: --persona ID")?)
}

/// Run synthesis jobs (`tools/voice/synth.py jobs`)
fn synthesize(jobs: &[Value], name: &Path) -> Result<(), String> {
    std::fs::create_dir_all(name.parent().unwrap()).map_err(|e| e.to_string())?;
    std::fs::write(name, serde_json::to_string_pretty(&json!({ "jobs": jobs })).unwrap()).map_err(|e| e.to_string())?;
    crate::run(uv_run("synth").arg("-u").arg(tools().join("synth.py")).arg("jobs").arg(name))
}

/// Clean raw takes (`tools/voice/evaluate.py clean-all`): what each became
fn clean_all(items: &[Value], name: &Path) -> Result<Vec<Value>, String> {
    std::fs::write(name, serde_json::to_string_pretty(&json!({ "items": items })).unwrap()).map_err(|e| e.to_string())?;
    let o = uv_run("eval").arg(tools().join("evaluate.py")).arg("clean-all").arg(name).output().map_err(|e| e.to_string())?;
    if !o.status.success() {
        return Err(format!("cleaning failed: {}", String::from_utf8_lossy(&o.stderr)));
    }
    String::from_utf8_lossy(&o.stdout)
        .lines()
        .filter(|l| l.starts_with('{'))
        .map(|l| serde_json::from_str(l).map_err(|e| e.to_string()))
        .collect()
}

/// `voice reference`: candidates for the persona's voice from its
/// description (seeds `design_seed` on), each heard back; `--write` keeps
/// the `design_seed` one, cleaned, as the persona's reference
pub fn reference(args: &[String]) -> Result<(), String> {
    let p = persona_arg(args)?;
    if p.voice_of() != p.id && opt(args, "--emotion").is_none() {
        return Err(format!(
            "{} speaks with {}'s voice (voice.same_as): its reference is {}; make that one's (--persona {})",
            p.id,
            p.voice_of(),
            p.reference().display(),
            p.voice_of()
        ));
    }
    if let Some(e) = opt(args, "--emotion") {
        return emotion_reference(&p, e, args);
    }
    let n: i64 = opt(args, "--candidates").map_or(Ok(1), |v| v.parse().map_err(|_| "--candidates N"))?;
    let seed0 = p.table("voice").get("design_seed").and_then(|v| v.as_integer()).unwrap_or(1);
    let dir = p.work().join("reference");
    let jobs: Vec<Value> = (seed0..seed0 + n)
        .map(|seed| {
            json!({
                "model": p.voice("design_model"),
                "text": p.voice("reference_text"),
                "instruct": p.voice("description").split_whitespace().collect::<Vec<_>>().join(" "),
                "seed": seed,
                "out": dir.join(format!("seed{seed}.wav")),
            })
        })
        .collect();
    synthesize(&jobs, &dir.join("jobs.json"))?;
    // each candidate judged with no emotion asked of it, against itself:
    // what matters is its words, how natural it is and its pitch
    let text = p.voice("reference_text");
    let takes: Vec<Value> = (seed0..seed0 + n)
        .map(|seed| {
            let wav = dir.join(format!("seed{seed}.wav"));
            json!({"line": format!("seed{seed}"), "take": seed, "path": wav, "reference": wav, "text": text, "emotion": "", "intensity": 0.0})
        })
        .collect();
    judge(&p, "reference", takes, &dir)?;
    print_candidates(&dir, &["words", "pitch", "not_synthetic"])?;
    if flag(args, "--write") {
        let chosen = dir.join(format!("seed{seed0}.wav"));
        let item = json!({ "src": chosen, "dst": p.reference(), "lufs": -20.0, "peak": -1.0 });
        let got = clean_all(&[item], &dir.join("clean.json"))?;
        println!("{}: the reference ({})", p.reference().display(), got.first().cloned().unwrap_or_default());
    } else {
        println!("listen, choose a seed, set voice.design_seed to it and run again with --write");
    }
    Ok(())
}

/// The candidates of a reference as the evaluation judged them
/// (`dir/report.json`): the measures to choose by, and which of the
/// `checks` that matter for it each fails
fn print_candidates(dir: &Path, checks: &[&str]) -> Result<(), String> {
    let rep: Value =
        serde_json::from_str(&std::fs::read_to_string(dir.join("report.json")).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    for t in rep["takes"].as_array().into_iter().flatten() {
        let failed: Vec<&str> = checks.iter().copied().filter(|c| t["checks"][*c] == json!(false)).collect();
        let f = |v: &Value| v.as_f64().map_or("-".to_string(), |x| format!("{x:.2}"));
        println!(
            "seed {}: synthetic {}, pitch {} Hz, HNR {} dB, arousal {}, voice {}, tone {}{}; words {:?}; fails {failed:?}",
            t["take"],
            f(&t["style"]["Speech synthesizer"]),
            t["prosody"]["f0_median"],
            f(&t["prosody"]["hnr_db"]),
            f(&t["emotion"]["arousal"]),
            f(&t["similarity"]),
            f(&t["tone"]["margin"]),
            if t["silent"] == json!(true) { ", silent" } else { "" },
            t["words"]["whisper"].as_str().unwrap_or("")
        );
    }
    println!("all of it: {}", dir.join("report.md").display());
    Ok(())
}

/// What a line's take is made with: the job for its model
pub fn job(p: &Persona, model: &str, line: &toml::Table, take: i64, out: &Path) -> Value {
    let emotion = s(line, "emotion").unwrap_or_default();
    let words = s(line, "direction").unwrap_or_else(|| p.emotion_words(&emotion));
    let mut j = json!({
        "model": model,
        "text": s(line, "text").unwrap_or_default(),
        "seed": take,
        "emotion": emotion,
        "intensity": super::num(line.get("intensity")),
        "out": out,
    });
    let (ref_audio, ref_text) = p.clone_from(&emotion);
    let refer = |j: &mut Value| {
        j["ref_audio"] = json!(ref_audio);
        j["ref_text"] = json!(ref_text);
    };
    match model {
        // the cloned voice, directed
        "voxcpm2" => {
            refer(&mut j);
            j["instruct"] = json!(words);
        }
        // the voice designed again with the direction (no instruction on a clone)
        "qwen3-tts-voicedesign" => {
            let d = p.voice("description").split_whitespace().collect::<Vec<_>>().join(" ");
            j["instruct"] = json!(format!("{d} He is {words}."));
        }
        // the cloned voice; intensity sets exaggeration; the label for Ming
        _ => refer(&mut j),
    }
    j
}

fn line_ids(p: &Persona, args: &[String]) -> Vec<toml::Table> {
    let only: Vec<&str> = opt(args, "--only").map(|o| o.split(',').collect()).unwrap_or_default();
    p.lines.iter().filter(|l| only.is_empty() || s(l, "id").is_some_and(|id| only.contains(&id.as_str()))).cloned().collect()
}

pub fn model_arg(p: &Persona, args: &[String]) -> Result<String, String> {
    let m = opt(args, "--model").map(String::from).unwrap_or_else(|| p.voice("line_model"));
    if !super::SPEAKERS.contains(&m.as_str()) {
        return Err(format!("{m} cannot say a line: one of {:?}", super::SPEAKERS));
    }
    Ok(m)
}

/// `voice build`: every take of every line (or `--only` some), raw from the
/// model, then cleaned: `target/voice/<persona>/<model>/<line>/take<k>.wav`
pub fn build(args: &[String]) -> Result<(), String> {
    let p = persona_arg(args)?;
    let model = model_arg(&p, args)?;
    if !p.reference().is_file() && model != "qwen3-tts-voicedesign" {
        return Err(format!("{} has no reference voice yet: cargo xtask voice reference --persona {} --candidates 6", p.id, p.id));
    }
    let dir = p.work().join(&model);
    let mut jobs = vec![];
    let mut items = vec![];
    for l in line_ids(&p, args) {
        let id = s(&l, "id").unwrap_or_default();
        for k in 1..=l.get("takes").and_then(|v| v.as_integer()).unwrap_or(1) {
            let raw = dir.join("raw").join(&id).join(format!("take{k}.wav"));
            jobs.push(job(&p, &model, &l, k, &raw));
            items.push(json!({ "src": raw, "dst": dir.join(&id).join(format!("take{k}.wav")), "lufs": p.target("loudness_lufs"), "peak": p.target("peak_dbtp") }));
        }
    }
    synthesize(&jobs, &dir.join("jobs.json"))?;
    let got = clean_all(&items, &dir.join("clean.json"))?;
    for (it, g) in items.iter().zip(&got) {
        println!("{}: {g}", it["dst"].as_str().unwrap_or_default());
    }
    println!("{} takes in {}", got.len(), dir.display());
    Ok(())
}

/// Where a persona's line goes in the game's assets (relative to them)
pub fn asset_name(persona: &str, line: &str) -> String {
    format!("voice/{persona}/{line}.wav")
}

/// `voice write`: each line's chosen take (from the evaluation's report, or
/// `--take K` for all) into the game's assets, with its manifest entry
pub fn write(args: &[String]) -> Result<(), String> {
    let p = persona_arg(args)?;
    let model = model_arg(&p, args)?;
    let dir = p.work().join(&model);
    let report: Option<Value> = std::fs::read_to_string(dir.join("report.json")).ok().and_then(|t| serde_json::from_str(&t).ok());
    // `--take K` for every line, or `--take id=K,id2=K2` for some
    let takes_given = opt(args, "--take").unwrap_or("");
    let forced_all: Option<i64> = takes_given.parse().ok();
    let mut forced_line = std::collections::HashMap::new();
    for part in takes_given.split(',').filter(|p| p.contains('=')) {
        let (id, k) = part.split_once('=').unwrap();
        forced_line.insert(id.to_string(), k.parse::<i64>().map_err(|_| format!("--take {part}: id=K"))?);
    }
    if !takes_given.is_empty() && forced_all.is_none() && forced_line.is_empty() {
        return Err("--take K, or --take id=K,…".into());
    }
    let ms = models()?;
    let upstream = |id: &str| ms.iter().find(|m| m.id == id).and_then(|m| m.upstream.clone()).unwrap_or_default();
    let assets = root().join("crates/game/assets");
    for l in &p.lines {
        let id = s(l, "id").unwrap_or_default();
        let take = match (forced_line.get(&id).copied().or(forced_all), &report) {
            (Some(k), _) => k,
            (None, Some(r)) => r["lines"][&id]["chosen"]
                .as_i64()
                .ok_or_else(|| format!("{id}: the report chose no take (it failed the gate); fix it or pass --take"))?,
            (None, None) => {
                return Err(format!(
                    "no report for {} on {model}: run `cargo xtask voice eval --persona {} --model {model}` or pass --take",
                    p.id, p.id
                ))
            }
        };
        let src = dir.join(&id).join(format!("take{take}.wav"));
        let bytes = std::fs::read(&src).map_err(|e| format!("{}: {e}", src.display()))?;
        let name = asset_name(&p.id, &id);
        let dst = assets.join(&name);
        std::fs::create_dir_all(dst.parent().unwrap()).map_err(|e| e.to_string())?;
        std::fs::write(&dst, &bytes).map_err(|e| e.to_string())?;
        let meta: Value = std::fs::read_to_string(dir.join("raw").join(&id).join(format!("take{take}.json")))
            .ok()
            .and_then(|t| serde_json::from_str(&t).ok())
            .unwrap_or_default();
        let mut sources = vec![
            format!("assets/src/voice/personas/{}.toml", p.id),
            format!("assets/src/voice/personas/{}.wav", p.voice_of()),
            format!("assets/src/voice/lines/{}.toml", p.id),
            format!("assets/src/voice/models/{model}/SOURCE.toml"),
            "tools/voice/models.toml".into(),
            "tools/voice/synth.py".into(),
            "tools/voice/evaluate.py".into(),
        ];
        // the persona whose voice it is, described
        if p.voice_of() != p.id {
            sources.insert(1, format!("assets/src/voice/personas/{}.toml", p.voice_of()));
        }
        // cloned from an emotion's own reference
        let (from, _) = p.clone_from(&s(l, "emotion").unwrap_or_default());
        if from != p.reference() {
            sources.insert(2, from.strip_prefix(root()).unwrap_or(&from).to_string_lossy().into_owned());
        }
        crate::assets::update_manifest(
            &name,
            &bytes,
            json!({
                "authors": "written by coding agents under the developer's direction; the voice designed from a description (no real person's), said by a text-to-speech model",
                "built_with": format!("{} at {} ({}), cloned from the persona's reference (designed by {}); trimmed, 48 kHz, set to {} LUFS, true peak under {} dBTP; by cargo xtask voice", meta["repo"].as_str().unwrap_or(&model), &meta["revision"].as_str().unwrap_or("?")[..12.min(meta["revision"].as_str().unwrap_or("?").len())], meta["backend"].as_str().unwrap_or("?"), upstream(&p.voice("design_model")), p.target("loudness_lufs"), p.target("peak_dbtp")),
                "generator": "text-to-speech models, processed by scripts",
                "generative_model": upstream(&model),
                "reference_model": upstream(&p.voice("design_model")),
                "licence": "MIT",
                "rebuild": format!("cargo xtask voice build --persona {} --model {model} --only {id}; voice eval; voice write (take {take}, seed {take})", p.id),
                "line": { "persona": p.id, "id": id, "text": s(l, "text"), "emotion": s(l, "emotion"), "seed": take },
                "sources": sources,
            }),
        )?;
        println!("{name}: take {take} ({} bytes)", bytes.len());
    }
    write_index(&p)?;
    Ok(())
}

/// The index the game reads (`crates/game/assets/voice/lines.tsv`): a row
/// a line, the persona's rows replaced, the others' kept, sorted
pub fn index_rows(p: &Persona) -> Vec<String> {
    p.lines
        .iter()
        .map(|l| {
            let lv: Vec<i64> =
                l.get("levels").and_then(|v| v.as_array()).map(|a| a.iter().filter_map(|x| x.as_integer()).collect()).unwrap_or_default();
            format!(
                "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
                p.id,
                s(l, "id").unwrap_or_default(),
                s(l, "use").unwrap_or_default(),
                s(l, "subject").unwrap_or_default(),
                lv.first().unwrap_or(&1),
                lv.get(1).unwrap_or(&99),
                super::num(l.get("intensity")),
                s(l, "emotion").unwrap_or_default(),
                s(l, "text").unwrap_or_default()
            )
        })
        .collect()
}

pub const INDEX_HEADER: &str = "persona\tid\tuse\tsubject\tfirst\tlast\tintensity\temotion\ttext";

pub fn index_path() -> PathBuf {
    root().join("crates/game/assets/voice/lines.tsv")
}

fn write_index(p: &Persona) -> Result<(), String> {
    let old = std::fs::read_to_string(index_path()).unwrap_or_default();
    let mut rows: Vec<String> = old.lines().skip(1).filter(|r| !r.starts_with(&format!("{}\t", p.id))).map(String::from).collect();
    rows.extend(index_rows(p));
    rows.sort();
    std::fs::write(index_path(), format!("{INDEX_HEADER}\n{}\n", rows.join("\n"))).map_err(|e| e.to_string())
}

/// The takes of a persona's lines on a model that exist, as the evaluation
/// reads them
fn takes_of(p: &Persona, dir: &Path) -> Vec<Value> {
    let mut out = vec![];
    for l in &p.lines {
        let id = s(l, "id").unwrap_or_default();
        for k in 1..=l.get("takes").and_then(|v| v.as_integer()).unwrap_or(1) {
            let path = dir.join(&id).join(format!("take{k}.wav"));
            if path.is_file() {
                out.push(json!({
                    "line": id, "take": k, "path": path,
                    "text": s(l, "text"), "emotion": s(l, "emotion"), "intensity": super::num(l.get("intensity")),
                }));
            }
        }
    }
    out
}

/// Judge takes (`tools/voice/evaluate.py report`): whether they pass
pub fn judge(p: &Persona, model: &str, takes: Vec<Value>, dir: &Path) -> Result<bool, String> {
    let job = json!({
        "persona": serde_json::to_value(&p.t).map_err(|e| e.to_string())?,
        "reference": p.reference(),
        "model": model,
        "takes": takes,
    });
    let file = dir.join("eval.json");
    std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    std::fs::write(&file, serde_json::to_string_pretty(&job).unwrap()).map_err(|e| e.to_string())?;
    let st = uv_run("eval").arg(tools().join("evaluate.py")).arg("report").arg(&file).arg(dir).status().map_err(|e| e.to_string())?;
    Ok(st.success())
}

/// `voice eval`: every take of a persona's lines on a model judged; each
/// line's best passing take chosen; the gate fails when a line has none
pub fn eval(args: &[String]) -> Result<(), String> {
    let p = persona_arg(args)?;
    let model = model_arg(&p, args)?;
    let dir = p.work().join(&model);
    let takes = takes_of(&p, &dir);
    if takes.is_empty() {
        return Err(format!("no takes in {}: cargo xtask voice build --persona {} --model {model}", dir.display(), p.id));
    }
    let n = takes.len();
    let ok = judge(&p, &model, takes, &dir)?;
    let rep: Value =
        serde_json::from_str(&std::fs::read_to_string(dir.join("report.json")).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    for (line, v) in rep["lines"].as_object().into_iter().flatten() {
        println!("{line:22} chosen {:>4}  passing {}", v["chosen"].as_i64().map_or("none".into(), |k| format!("take {k}")), v["passing"]);
    }
    println!("{n} takes judged; {}", dir.join("report.md").display());
    if ok {
        Ok(())
    } else {
        Err(format!("{} on {model}: a line has no take that passes (see the report)", p.id))
    }
}

/// A known answer for the evaluation: a take made on purpose, and which of
/// its checks must fail (`None`: it must pass them all)
struct Known {
    name: &'static str,
    expect: Option<&'static str>,
}

/// `voice selftest`: the evaluation against known answers (the first
/// persona's voice): words that are not the line's, a calm read of a
/// terrified line and two voices that are not his must each fail the check
/// that is theirs; three of his own good takes must pass every check
pub fn selftest(args: &[String]) -> Result<(), String> {
    let p = Persona::load(opt(args, "--persona").unwrap_or("you"))?;
    let dir = root().join("target/voice/selftest");
    let line = |id: &str| {
        p.lines.iter().find(|l| s(l, "id").as_deref() == Some(id)).cloned().ok_or_else(|| format!("selftest needs line {id} of {}", p.id))
    };
    let woman = "A young woman in her twenties with a bright, clear, high voice and a light British accent. She is uneasy and uncertain, calling out into the dark.";
    let old_man = "An elderly man in his seventies with a deep, slow, gravelly voice and a Scottish accent. He is uneasy and uncertain, calling out into the dark.";
    let raw = |n: &str| dir.join("raw").join(format!("{n}.wav"));
    let mut jobs = vec![
        json!({"model": "voxcpm2", "seed": 1, "text": "The lights hum the whole time.", "ref_audio": p.reference(), "instruct": p.emotion_words("fear"), "out": raw("wrong_words")}),
        json!({"model": "voxcpm2", "seed": 1, "text": "Get me out of here! Please!", "ref_audio": p.reference(), "instruct": "calm, relaxed and cheerful, speaking softly and slowly", "out": raw("calm_terror")}),
        json!({"model": "qwen3-tts-voicedesign", "seed": 1, "text": "Hello? Is anybody there?", "instruct": woman, "out": raw("other_woman")}),
        json!({"model": "qwen3-tts-voicedesign", "seed": 2, "text": "Hello? Is anybody there?", "instruct": old_man, "out": raw("other_man_calm")}),
    ];
    // what each was meant to be, as the evaluation judges it
    let own = |n: &str, t: &str, e: &str, x: f64| (n.to_string(), t.to_string(), e.to_string(), x);
    let mut judged = vec![
        own("wrong_words", "Help! Help me!", "fear", 0.75),
        own("calm_terror", "Get me out of here! Please!", "terror", 0.8),
        own("other_woman", "Hello? Is anybody there?", "unease", 0.35),
        own("other_man_calm", "Hello? Is anybody there?", "unease", 0.35),
    ];
    let mut known = vec![
        Known { name: "wrong_words", expect: Some("words") },
        Known { name: "calm_terror", expect: Some("emotion") },
        Known { name: "other_woman", expect: Some("voice") },
        Known { name: "other_man_calm", expect: Some("voice") },
    ];
    // his own good takes, made as the pipeline makes them
    for (name, id, take) in
        [("good_hello", "hello_anyone", 2), ("good_hoarse", "please_help_hoarse", 1), ("good_somebody", "somebody_help", 4)]
    {
        let l = line(id)?;
        jobs.push(job(&p, "voxcpm2", &l, take, &raw(name)));
        judged.push((
            name.to_string(),
            s(&l, "text").unwrap_or_default(),
            s(&l, "emotion").unwrap_or_default(),
            super::num(l.get("intensity")),
        ));
        known.push(Known { name, expect: None });
    }
    synthesize(&jobs, &dir.join("jobs.json"))?;
    let items: Vec<Value> = judged.iter().map(|(n, ..)| json!({"src": raw(n), "dst": dir.join(format!("{n}.wav")), "lufs": p.target("loudness_lufs"), "peak": p.target("peak_dbtp")})).collect();
    clean_all(&items, &dir.join("clean.json"))?;
    let takes: Vec<Value> = judged
        .iter()
        .map(|(n, text, emotion, x)| json!({"line": n, "take": 1, "path": dir.join(format!("{n}.wav")), "text": text, "emotion": emotion, "intensity": x}))
        .collect();
    judge(&p, "selftest", takes, &dir)?; // the gate fails by design: the bad ones fail
    let rep: Value =
        serde_json::from_str(&std::fs::read_to_string(dir.join("report.json")).map_err(|e| e.to_string())?).map_err(|e| e.to_string())?;
    let mut wrong = vec![];
    for k in &known {
        let t =
            rep["takes"].as_array().into_iter().flatten().find(|t| t["line"] == k.name).ok_or_else(|| format!("{}: not judged", k.name))?;
        let failed: Vec<&str> =
            t["checks"].as_object().into_iter().flatten().filter(|(_, v)| v == &&json!(false)).map(|(c, _)| c.as_str()).collect();
        let right = match k.expect {
            Some(c) => failed.contains(&c),
            None => failed.is_empty(),
        };
        println!(
            "{} {:16} must {:18} failed {:?}",
            if right { "ok  " } else { "FAIL" },
            k.name,
            k.expect.map_or("pass".to_string(), |c| format!("fail {c}")),
            failed
        );
        if !right {
            wrong.push(k.name);
        }
    }
    println!("{}", dir.join("report.md").display());
    if wrong.is_empty() {
        println!("voice selftest: the evaluation gives every known answer");
        Ok(())
    } else {
        Err(format!("voice selftest: wrong on {wrong:?}"))
    }
}

/// `voice reference --emotion E`: candidates for an emotion's reference,
/// cloned from the persona's voice by the reference's model with the
/// emotion's words (seeds `references.E.seed` on), each judged as a take of
/// that emotion at intensity 0.9; `--write` keeps the seed's one as
/// `assets/src/voice/personas/<id>.<E>.wav`
fn emotion_reference(p: &Persona, e: &str, args: &[String]) -> Result<(), String> {
    let (path, r) = p.emotion_ref(e).ok_or_else(|| format!("{} has no [references.{e}]", p.id))?;
    if !p.reference().is_file() {
        return Err(format!("make {}'s reference first: cargo xtask voice reference --persona {} --write", p.id, p.id));
    }
    let n: i64 = opt(args, "--candidates").map_or(Ok(1), |v| v.parse().map_err(|_| "--candidates N"))?;
    let seed0 = r.get("seed").and_then(|v| v.as_integer()).unwrap_or(1);
    let text = s(&r, "text").unwrap_or_default();
    let dir = p.work().join("reference").join(e);
    let jobs: Vec<Value> = (seed0..seed0 + n)
        .map(|seed| {
            json!({
                "model": s(&r, "model").unwrap_or_default(), "text": text, "seed": seed,
                "instruct": p.emotion_words(e), "emotion": e, "intensity": 0.9,
                "ref_audio": p.reference(), "ref_text": p.voice("reference_text"),
                "out": dir.join("raw").join(format!("seed{seed}.wav")),
            })
        })
        .collect();
    synthesize(&jobs, &dir.join("jobs.json"))?;
    let items: Vec<Value> =
        (seed0..seed0 + n).map(|seed| json!({"src": dir.join("raw").join(format!("seed{seed}.wav")), "dst": dir.join(format!("seed{seed}.wav")), "lufs": -20.0, "peak": -1.0})).collect();
    clean_all(&items, &dir.join("clean.json"))?;
    let takes: Vec<Value> = (seed0..seed0 + n)
        .map(|seed| json!({"line": format!("seed{seed}"), "take": seed, "path": dir.join(format!("seed{seed}.wav")), "text": text, "emotion": e, "intensity": 0.9}))
        .collect();
    judge(p, &format!("reference {e}"), takes, &dir)?;
    print_candidates(&dir, &["words", "emotion", "pitch", "tone", "voice", "not_synthetic"])?;
    if flag(args, "--write") {
        let chosen = dir.join(format!("seed{seed0}.wav"));
        std::fs::copy(&chosen, &path).map_err(|e| format!("{}: {e}", chosen.display()))?;
        println!("{}: the {e} reference (seed {seed0})", path.display());
    } else {
        println!("choose a seed, set references.{e}.seed to it and run again with --write");
    }
    Ok(())
}

/// `voice bakeoff`: the evaluation's reports for a persona side by side,
/// one row a model that has one (`docs/VOICES.md`, *The bake-off*)
pub fn bakeoff(args: &[String]) -> Result<(), String> {
    let p = persona_arg(args)?;
    println!("| Model | Lines with a passing take | Takes passing | Mean voice similarity | Checks failed |");
    println!("|---|---|---|---|---|");
    for m in super::SPEAKERS {
        let Ok(text) = std::fs::read_to_string(p.work().join(m).join("report.json")) else { continue };
        let rep: Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
        let takes = rep["takes"].as_array().cloned().unwrap_or_default();
        let lines = rep["lines"].as_object().map(|o| (o.values().filter(|v| !v["chosen"].is_null()).count(), o.len())).unwrap_or((0, 0));
        let passing = takes.iter().filter(|t| t["passes"] == json!(true)).count();
        let sim = takes.iter().filter_map(|t| t["similarity"].as_f64()).sum::<f64>() / takes.len().max(1) as f64;
        let mut fails: Vec<(String, usize)> = vec![];
        for t in &takes {
            for (c, v) in t["checks"].as_object().into_iter().flatten() {
                if v == &json!(false) {
                    match fails.iter_mut().find(|(k, _)| k == c) {
                        Some((_, n)) => *n += 1,
                        None => fails.push((c.clone(), 1)),
                    }
                }
            }
        }
        fails.sort_by(|a, b| b.1.cmp(&a.1));
        let fails: Vec<String> = fails.iter().map(|(c, n)| format!("{c} {n}")).collect();
        println!("| {m} | {}/{} | {passing}/{} | {sim:.2} | {} |", lines.0, lines.1, takes.len(), fails.join(", "));
    }
    Ok(())
}
