//! Project automation (`cargo xtask <task>`).
//!
//! ```text
//! cargo xtask bundle            release build → dist/Backrooms.app and dist/backrooms
//! cargo xtask bundle --debug    debug build (faster to compile, slower to run)
//! cargo xtask bundle --run      … and open the app
//! ```
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

const TARGET: &str = "aarch64-apple-darwin";
const APP_NAME: &str = "Backrooms.app";
const BIN_NAME: &str = "backrooms";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let result = match args.first().map(String::as_str) {
        Some("bundle") => bundle(&args[1..]),
        _ => {
            eprintln!("usage: cargo xtask bundle [--debug] [--run]");
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

fn run(cmd: &mut Command) -> Result<(), String> {
    let status = cmd.status().map_err(|e| format!("{cmd:?}: {e}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{cmd:?} failed with {status}"))
    }
}

/// Build the game and package it as a signed, double-clickable app bundle.
fn bundle(args: &[String]) -> Result<(), String> {
    let debug = args.iter().any(|a| a == "--debug");
    let open = args.iter().any(|a| a == "--run");
    let root = root();
    if std::env::consts::ARCH != "aarch64" {
        eprintln!("note: building on {}; the target is {TARGET}", std::env::consts::ARCH);
    }

    let mut build = Command::new(std::env::var("CARGO").unwrap_or_else(|_| "cargo".into()));
    build.current_dir(&root).args(["build", "-p", BIN_NAME, "--target", TARGET]);
    if !debug {
        build.arg("--release");
    }
    run(&mut build)?;

    let profile = if debug { "debug" } else { "release" };
    let bin = root.join("target").join(TARGET).join(profile).join(BIN_NAME);
    let dist = root.join("dist");
    let app = dist.join(APP_NAME);
    if app.exists() {
        std::fs::remove_dir_all(&app).map_err(|e| format!("removing {}: {e}", app.display()))?;
    }
    let macos = app.join("Contents/MacOS");
    let resources = app.join("Contents/Resources");
    for d in [&macos, &resources] {
        std::fs::create_dir_all(d).map_err(|e| format!("{}: {e}", d.display()))?;
    }
    let copy = |from: &Path, to: &Path| {
        std::fs::copy(from, to).map(|_| ()).map_err(|e| format!("copying {} → {}: {e}", from.display(), to.display()))
    };
    copy(&bin, &macos.join(BIN_NAME))?;
    copy(&root.join("packaging/Info.plist"), &app.join("Contents/Info.plist"))?;
    let icon = root.join("packaging/AppIcon.icns");
    if icon.exists() {
        copy(&icon, &resources.join("AppIcon.icns"))?;
    }
    // an ad-hoc signature is required to run on Apple Silicon
    if run(Command::new("codesign").args(["--force", "--deep", "--sign", "-"]).arg(&app)).is_err() {
        eprintln!("warning: codesign failed; the app may need `xattr -cr` before it runs");
    }
    copy(&bin, &dist.join(BIN_NAME))?;
    println!("built {} and {}", app.display(), dist.join(BIN_NAME).display());

    if open {
        run(Command::new("open").arg(&app))?;
    }
    Ok(())
}
