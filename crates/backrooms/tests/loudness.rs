//! The mix's loudness and variety, measured the way broadcast and game audio
//! are (EBU R128 / ITU-R BS.1770, with the `ebur128` crate) on offline
//! renders of whole levels:
//!
//! * the integrated loudness of each scenario stays within `DRIFT_LU` of
//!   its recording (a change of gains or of the bed shows up here, even when
//!   every relative check in `listen.rs` still passes);
//! * its loudness range stays within `RANGE_LU` of its recording;
//! * no sample peaks above `TRUE_PEAK_DBTP` (−1 dBTP, the game-audio ceiling
//!   in the G.A.N.G. and Sony loudness recommendations);
//! * one-shot ambient events do not come back too soon: the same sound not
//!   twice within `MIN_REPEAT_S`.
//!
//! The per-frame randomness is seeded, so a scenario renders the same audio
//! every time; after an intended change to the mix, look at (and listen to)
//! the `listen.rs` renders, then re-record:
//!
//!   cargo test --release -p backrooms --test loudness -- --ignored rerecord
//!
//! (the recording lives in `tests/recorded/`, next to `tests/reference/`,
//! whose every file is a recorded level)
use engine::audio::AudioContext;
use game::Game;
use serde_json::{Value, json};

const SR: u32 = 48000;
/// The scenarios: run seed, level, seconds (creatures held still).
// (rooms levels at the depths the scenarios were made for: the Building's
// floors came above them, `docs/design/MEGASTRUCTURE.md`)
const SCENARIOS: &[(u32, i32, f64)] = &[(4242, 6, 120.0), (77, 8, 120.0), (9001, 11, 120.0)];
/// Integrated loudness may move this far from the recording.
const DRIFT_LU: f64 = 1.0;
/// Loudness range may move this far from the recording.
const RANGE_LU: f64 = 3.0;
/// No sample above this (dBTP).
const TRUE_PEAK_DBTP: f64 = -1.0;
/// The same one-shot ambient event is not heard twice within this.
const MIN_REPEAT_S: f64 = 8.0;

#[derive(Debug)]
struct Measured {
    integrated: f64,
    range: f64,
    true_peak: f64,
    momentary_max: f64,
    /// (tag, seconds between its two closest occurrences, how often)
    repeats: Vec<(String, f64, usize)>,
}

fn measure(seed: u32, level: i32, secs: f64) -> Measured {
    game::base::rng::seed_native((u64::from(seed) << 8) | level as u64);
    let mut g = Game::new_bare();
    placers::register_all(&mut g.placers);
    g.launch_flags = format!("seed={seed}&level={level}");
    let ac = AudioContext::new_offline(SR as f64);
    g.audio_init_with(ac.clone());
    g.boot();
    g.start_new_game();
    g.god_mode = true;
    let mut ebu =
        ebur128::EbuR128::new(2, SR, ebur128::Mode::I | ebur128::Mode::LRA | ebur128::Mode::TRUE_PEAK | ebur128::Mode::M).unwrap();
    let mut momentary_max = f64::NEG_INFINITY;
    let mut seen: std::collections::BTreeMap<String, Vec<f64>> = Default::default();
    let mut last_len = 0;
    let mut t = 0.0;
    for _ in 0..(secs * 60.0).round() as usize {
        for e in g.entities.iter_mut() {
            e.freeze = 1e9;
        }
        g.now_ms += 1000.0 / 60.0;
        g.fixed_step_dt = 1.0 / 60.0;
        g.frame(1.0 / 60.0);
        g.fixed_step_dt = 0.0;
        t += 1.0 / 60.0;
        let pcm = ac.render_offline(SR as usize / 60);
        ebu.add_frames_f32(&pcm).unwrap();
        if t > 0.4 {
            momentary_max = momentary_max.max(ebu.loudness_momentary().unwrap_or(f64::NEG_INFINITY));
        }
        if g.ambient_log.len() < last_len {
            last_len = 0;
        }
        for e in &g.ambient_log[last_len..] {
            seen.entry(e.tag.clone()).or_default().push(t);
        }
        last_len = g.ambient_log.len();
    }
    let true_peak = (0..2).map(|c| ebu.true_peak(c).unwrap()).fold(0.0, f64::max);
    let repeats = seen
        .into_iter()
        .map(|(tag, ts)| {
            let gap = ts.windows(2).map(|w| w[1] - w[0]).fold(f64::INFINITY, f64::min);
            (tag, gap, ts.len())
        })
        .collect();
    Measured {
        integrated: ebu.loudness_global().unwrap(),
        range: ebu.loudness_range().unwrap(),
        true_peak: 20.0 * true_peak.max(1e-9).log10(),
        momentary_max,
        repeats,
    }
}

fn reference_path() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/recorded/loudness.json")
}

fn key(seed: u32, level: i32) -> String {
    format!("{seed}_L{level}")
}

#[test]
fn the_mix_holds_its_loudness_and_varies() {
    let reference: Value =
        std::fs::read_to_string(reference_path()).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or(Value::Null);
    let mut problems = vec![];
    for &(seed, level, secs) in SCENARIOS {
        let m = measure(seed, level, secs);
        let k = key(seed, level);
        println!(
            "{k}: integrated {:.1} LUFS, range {:.1} LU, true peak {:.1} dBTP, loudest 400 ms {:.1} LUFS",
            m.integrated, m.range, m.true_peak, m.momentary_max
        );
        for (tag, gap, n) in &m.repeats {
            println!("    {tag:<28} ×{n:<3} closest repeat {gap:.1} s");
        }
        let r = &reference[&k];
        match (r["integrated"].as_f64(), r["range"].as_f64()) {
            (Some(i), Some(lra)) => {
                if (m.integrated - i).abs() > DRIFT_LU {
                    problems
                        .push(format!("{k}: integrated loudness {:.1} LUFS, recorded {i:.1} (more than {DRIFT_LU} LU off)", m.integrated));
                }
                if (m.range - lra).abs() > RANGE_LU {
                    problems.push(format!("{k}: loudness range {:.1} LU, recorded {lra:.1} (more than {RANGE_LU} LU off)", m.range));
                }
            }
            _ => problems.push(format!("{k}: no recording in tests/recorded/loudness.json (re-record)")),
        }
        if m.true_peak > TRUE_PEAK_DBTP {
            problems.push(format!("{k}: true peak {:.1} dBTP above {TRUE_PEAK_DBTP}", m.true_peak));
        }
        for (tag, gap, _) in &m.repeats {
            if !tag.starts_with("sig-") && *gap < MIN_REPEAT_S {
                problems.push(format!("{k}: {tag} came back after {gap:.1} s (under {MIN_REPEAT_S} s)"));
            }
        }
    }
    assert!(problems.is_empty(), "{}", problems.join("\n"));
}

#[test]
#[ignore]
fn rerecord() {
    let mut out = serde_json::Map::new();
    for &(seed, level, secs) in SCENARIOS {
        let m = measure(seed, level, secs);
        let r = |v: f64| (v * 100.0).round() / 100.0;
        out.insert(key(seed, level), json!({ "integrated": r(m.integrated), "range": r(m.range), "true_peak": r(m.true_peak) }));
    }
    std::fs::write(reference_path(), serde_json::to_string_pretty(&Value::Object(out)).unwrap() + "\n").unwrap();
    println!("wrote {}", reference_path().display());
}
