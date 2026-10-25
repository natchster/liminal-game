//! Project automation (`cargo xtask <task>`).
//!
//! ```text
//! cargo xtask bundle            release build → dist/Backrooms.app and dist/backrooms
//! cargo xtask bundle --debug    debug build (faster to compile, slower to run)
//! cargo xtask bundle --run      … and open the app
//! cargo xtask check [--fast]    the quality gate: fmt, clippy, tests (terse report)
//! cargo xtask hook <event>      the Claude Code hooks (.claude/settings.json)
//! ```
mod bundle;
mod check;
mod hook;

use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

const USAGE: &str = "usage: cargo xtask <task>
  bundle [--debug] [--run]   build and sign dist/Backrooms.app
  check [--fast] [--only STEP,…]
                             fmt, clippy and tests with a terse report;
                             logs in target/xtask/check/
  hook pre-edit|post-edit|stop|session
                             Claude Code hooks; read the hook JSON on stdin";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let rest = args.get(1..).unwrap_or(&[]);
    let result = match args.first().map(String::as_str) {
        Some("bundle") => bundle::bundle(rest),
        Some("check") => check::check(rest),
        Some("hook") => hook::hook(rest),
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
