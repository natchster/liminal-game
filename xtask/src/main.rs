//! Project automation (`cargo xtask <task>`).
//!
//! ```text
//! cargo xtask bundle            release build → dist/Backrooms.app and dist/backrooms
//! cargo xtask bundle --debug    debug build (faster to compile, slower to run)
//! cargo xtask bundle --run      … and open the app
//! cargo xtask check [--fast]    the quality gate: fmt, clippy, tests (terse report)
//! cargo xtask hook <event>      the Claude Code hooks (.claude/settings.json)
//! cargo xtask shots             golden frames and look statistics
//! cargo xtask range [--look]    the generator's expressive range against its baseline
//! cargo xtask ab prepare|score  blind A/B reviews of frames
//! cargo xtask calibrate prepare|score   a reviewer's agreement with known answers
//! cargo xtask assets props|creatures|textures|fingerprint   rebuild and check the embedded assets
//! cargo xtask bench [--no-record]   performance against budgets and its history
//! cargo xtask heatmap REC.json…    where players went, from recorded sessions
//! cargo xtask drift [--skip-check]   the drift sweep: gates → ledger and QUALITY.md
//! cargo xtask nightly             drift sweep + long level sweep + recorded bench
//! cargo xtask voice doctor|fetch  the voice-over pipeline (docs/VOICES.md)
//! ```
mod ab;
mod assets;
mod bench;
mod bodies;
mod bundle;
mod calibrate;
mod check;
mod drift;
mod heatmap;
mod hook;
mod look;
mod range;
mod shots;
mod testpool;
mod voice;

use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

const USAGE: &str = "usage: cargo xtask <task>
  bundle [--debug] [--run]   build and sign dist/Backrooms.app
  check [--fast] [--only STEP,…] [--skip STEP,…]
                             fmt, clippy, tests and shots with a terse report;
                             logs in target/xtask/check/
  shots [--list] [--accept NAME,…|all --reason \"…\"] [--accept-look NAME,…|all --reason \"…\"]
                             render the golden shot list, compare frames (ꟻLIP)
                             and look statistics with crates/backrooms/tests/golden/
  range [--look] [--accept --reason \"…\"]
                             level measures over fixed seeds against
                             crates/backrooms/tests/recorded/range.json
  ab prepare --before DIR --after DIR [--names …] [--question \"…\"]
  ab score ANSWERS.json [--reviewer NAME]
                             blind A/B review pairs (both orders) and their scoring
  calibrate prepare | calibrate score ANSWERS.json --reviewer NAME
                             damaged goldens with known answers; accuracy and
                             Cohen's kappa of a reviewer
  assets props [--batch …] [--write] | assets creatures [--only …] [--write]
  assets textures [--write] | assets fingerprint
                             rebuild the prop meshes, the creatures' bodies (Blender)
                             or the photographed textures, and compare with crates/game/assets
  bench [--no-record]        frame, build and first-frame times against docs/PERF.md
                             and perf/history.csv (change points)
  heatmap REC.json …         heatmaps of recorded sessions (backrooms --record)
  drift [--skip-check]       run the gates, write passes into docs/content.json and
                             the Latest sweep into docs/QUALITY.md
  nightly                    drift sweep, a 200-case level sweep, a recorded bench
  voice doctor [--smoke] | voice fetch [--only ID,…]
                             the voice-over pipeline (docs/VOICES.md): its toolchain
                             and models
  hook pre-edit|post-edit|stop|session
                             Claude Code hooks; read the hook JSON on stdin";

fn main() -> ExitCode {
    // nothing a task runs (a test, a shot, a bench, a build of the game)
    // sounds through the speakers: every child inherits this
    // (`engine::audio::device_output`; `bundle --run` opens the app through
    // Launch Services, which does not pass it on)
    std::env::set_var(SILENT_ENV, "1");
    let args: Vec<String> = std::env::args().skip(1).collect();
    let rest = args.get(1..).unwrap_or(&[]);
    let result = match args.first().map(String::as_str) {
        Some("bundle") => bundle::bundle(rest),
        Some("check") => check::check(rest),
        Some("hook") => hook::hook(rest),
        Some("shots") => shots::shots(rest),
        Some("range") => range::range(rest),
        Some("ab") => ab::ab(rest),
        Some("calibrate") => calibrate::calibrate(rest),
        Some("assets") => assets::assets(rest),
        Some("bench") => bench::bench(rest),
        Some("heatmap") => heatmap::heatmap(rest),
        Some("drift") => drift::drift(rest),
        Some("nightly") => drift::nightly(rest),
        Some("voice") => voice::voice(rest),
        _ => {
            eprintln!("{USAGE}");
            return ExitCode::from(2);
        }
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

/// Silences the game wherever it runs from a task (read by
/// `engine::audio::device_output`)
const SILENT_ENV: &str = "BACKROOMS_SILENT";

/// The workspace root (the parent of this crate's directory).
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).parent().expect("xtask lives in the workspace root").to_path_buf()
}

/// The cargo that runs this task (so toolchain overrides carry over).
fn cargo() -> Command {
    let mut c = Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".into()));
    c.current_dir(root());
    c
}

fn run(cmd: &mut Command) -> Result<(), String> {
    let status = cmd.status().map_err(|e| format!("{cmd:?}: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{cmd:?} failed with {status}"))
    }
}

/// The value after `--name`, if given.
fn opt<'a>(args: &'a [String], name: &str) -> Option<&'a str> {
    args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).map(String::as_str)
}

fn flag(args: &[String], name: &str) -> bool {
    args.iter().any(|a| a == name)
}
