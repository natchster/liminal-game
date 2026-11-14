//! What a recording keeps of the build that made it (`src/replays.rs`): the
//! commit (with `+` when the tree differed from it) and the compiler.
use std::process::Command;

fn out(cmd: &mut Command) -> String {
    cmd.output().ok().filter(|o| o.status.success()).map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string()).unwrap_or_default()
}

fn main() {
    let commit = out(Command::new("git").args(["rev-parse", "--short=12", "HEAD"]));
    let dirty = !out(Command::new("git").args(["status", "--porcelain", "--untracked-files=no"])).is_empty();
    let commit = if commit.is_empty() {
        "unknown".to_string()
    } else if dirty {
        format!("{commit}+")
    } else {
        commit
    };
    println!("cargo:rustc-env=BACKROOMS_COMMIT={commit}");
    let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".into());
    println!("cargo:rustc-env=BACKROOMS_RUSTC={}", out(Command::new(rustc).arg("--version")));
    // a new commit, or a change in this crate, builds it again
    println!("cargo:rerun-if-changed=../../.git/HEAD");
    println!("cargo:rerun-if-changed=../../.git/refs/heads");
    println!("cargo:rerun-if-changed=src");
}
