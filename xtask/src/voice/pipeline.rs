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
        Ok(Persona { id: id.to_string(), t, lines })
    }
    fn table(&self, k: &str) -> toml::Table {
        self.t.get(k).and_then(|v| v.as_table()).cloned().unwrap_or_default()
    }
    pub fn voice(&self, k: &str) -> String {
        s(&self.table("voice"), k).unwrap_or_default()
    }
    pub fn target(&self, k: &str) -> f64 {
        super::num(self.table("targets").get(k))
    }
    pub fn emotion_words(&self, e: &str) -> String {
        s(&self.table("emotions"), e).unwrap_or_default()
    }
    /// The reference clip every line is cloned from (kept with the sources)
    pub fn reference(&self) -> PathBuf {
        voice_src().join(format!("personas/{}.wav", self.id))
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
    let text = p.voice("reference_text");
    for seed in seed0..seed0 + n {
        let wav = dir.join(format!("seed{seed}.wav"));
        let o = uv_run("eval")
            .arg(tools().join("evaluate.py"))
            .args(["hear", "--text", &text])
            .arg(&wav)
            .output()
            .map_err(|e| e.to_string())?;
        println!("seed {seed}: {}\n  {}", wav.display(), String::from_utf8_lossy(&o.stdout).trim().replace('\n', "\n  "));
    }
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
    let refer = |j: &mut Value| {
        j["ref_audio"] = json!(p.reference());
        j["ref_text"] = json!(p.voice("reference_text"));
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
    if !models()?.iter().any(|x| x.id == m && x.role == "synth") {
        return Err(format!("{m} is not a synthesis model of tools/voice/models.toml"));
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
    let forced: Option<i64> = opt(args, "--take").map(|v| v.parse().map_err(|_| "--take K")).transpose()?;
    let ms = models()?;
    let upstream = |id: &str| ms.iter().find(|m| m.id == id).and_then(|m| m.upstream.clone()).unwrap_or_default();
    let assets = root().join("crates/game/assets");
    for l in &p.lines {
        let id = s(l, "id").unwrap_or_default();
        let take = match (forced, &report) {
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
                "sources": [
                    format!("assets/src/voice/personas/{}.toml", p.id),
                    format!("assets/src/voice/personas/{}.wav", p.id),
                    format!("assets/src/voice/lines/{}.toml", p.id),
                    format!("assets/src/voice/models/{model}/SOURCE.toml"),
                    "tools/voice/models.toml",
                    "tools/voice/synth.py",
                    "tools/voice/evaluate.py",
                ],
            }),
        )?;
        println!("{name}: take {take} ({} bytes)", bytes.len());
    }
    Ok(())
}
