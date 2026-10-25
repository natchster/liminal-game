//! `cargo xtask hook <event>`: the Claude Code hooks in `.claude/settings.json`.
//!
//! Each reads the hook's JSON from stdin and answers in the hook protocol:
//!
//! - `pre-edit` (PreToolUse on file edits): denies edits to the HTML/JS version
//!   in the parent folder, and hand edits to recorded fixtures and goldens —
//!   those change only through the REGEN / rerecord / accept commands.
//! - `post-edit` (PostToolUse on file edits): runs rustfmt on an edited `.rs`
//!   file; a file rustfmt cannot parse is reported back (it does not block).
//! - `stop` (Stop): runs `xtask check --fast` when code changed since the
//!   last passing check, and blocks the stop with the `ERROR` lines when it
//!   fails — at most `MAX_BLOCKS` times in a row, then lets the stop through
//!   with a warning so a stuck session can still hand back.
//! - `session` (SessionStart after compaction): re-injects where the work
//!   stands — recent commits, uncommitted files, the roadmap's progress table.
use crate::root;
use serde_json::{json, Value};
use std::hash::{Hash, Hasher};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Consecutive stop blocks before a failing check lets the session stop.
const MAX_BLOCKS: u32 = 3;

/// Paths (relative to the workspace) that only re-recorders may write.
const RECORDED: &[&str] = &["crates/game/tests/fixtures/", "crates/backrooms/tests/reference/", "crates/backrooms/tests/golden/"];

/// Changes to these make the stop hook run the check (docs alone do not).
const CODE_EXT: &[&str] = &[".rs", ".toml", ".metal", ".lock", ".json", ".bin", ".txt"];

pub fn hook(args: &[String]) -> Result<(), String> {
    let mut input = String::new();
    std::io::stdin().read_to_string(&mut input).map_err(|e| format!("reading hook input: {e}"))?;
    let input: Value = serde_json::from_str(&input).unwrap_or(Value::Null);
    match args.first().map(String::as_str) {
        Some("pre-edit") => pre_edit(&input),
        Some("post-edit") => post_edit(&input),
        Some("stop") => stop(&input),
        Some("session") => session(),
        _ => Err("usage: cargo xtask hook pre-edit|post-edit|stop|session".into()),
    }
}

fn hook_dir() -> PathBuf {
    let d = root().join("target/xtask/hook");
    let _ = std::fs::create_dir_all(&d);
    d
}

/// The edited file, absolute.
fn edited_path(input: &Value) -> Option<PathBuf> {
    let ti = &input["tool_input"];
    let p = ti["file_path"].as_str().or_else(|| ti["notebook_path"].as_str())?;
    let p = Path::new(p);
    Some(if p.is_absolute() { p.to_path_buf() } else { Path::new(input["cwd"].as_str().unwrap_or(".")).join(p) })
}

/// Why an edit of `path` must not happen, if it must not.
fn edit_refusal(path: &Path, root: &Path) -> Option<String> {
    let parent = root.parent()?;
    if path.starts_with(parent) && !path.starts_with(root) {
        return Some(format!(
            "{} is part of the HTML/JS version in the parent folder, which is a read-only reference (CLAUDE.md, invariant 4).",
            path.display()
        ));
    }
    let rel = path.strip_prefix(root).ok()?.to_string_lossy().replace('\\', "/");
    if RECORDED.iter().any(|r| rel.starts_with(r)) {
        return Some(format!(
            "{rel} is recorded reference data: change it only by re-recording after an intended change \
             (REGEN=1 / `-- --ignored rerecord` / `xtask shots --accept`; see CLAUDE.md), then review the diff."
        ));
    }
    None
}

fn pre_edit(input: &Value) -> Result<(), String> {
    let Some(path) = edited_path(input) else { return Ok(()) };
    if let Some(reason) = edit_refusal(&path, &root()) {
        let out = json!({ "hookSpecificOutput": {
            "hookEventName": "PreToolUse",
            "permissionDecision": "deny",
            "permissionDecisionReason": reason,
        }});
        println!("{out}");
    }
    Ok(())
}

fn post_edit(input: &Value) -> Result<(), String> {
    let Some(path) = edited_path(input) else { return Ok(()) };
    if path.extension().is_none_or(|e| e != "rs") || !path.starts_with(root()) || !path.exists() {
        return Ok(());
    }
    let out = Command::new("rustfmt").args(["--edition", "2021"]).arg(&path).output().map_err(|e| format!("rustfmt: {e}"))?;
    if !out.status.success() {
        // exit 2: shown to Claude, does not undo the edit
        eprintln!("rustfmt could not format {}:\n{}", path.display(), String::from_utf8_lossy(&out.stderr));
        std::process::exit(2);
    }
    Ok(())
}

fn git(args: &[&str]) -> String {
    Command::new("git").current_dir(root()).args(args).output().map(|o| String::from_utf8_lossy(&o.stdout).into_owned()).unwrap_or_default()
}

/// A fingerprint of the uncommitted state of the code, or None when no code
/// file differs from HEAD.
fn code_fingerprint() -> Option<u64> {
    let status = git(&["status", "--porcelain=v1", "-uall"]);
    let changed: Vec<&str> = status.lines().filter_map(|l| l.get(3..)).filter(|p| CODE_EXT.iter().any(|e| p.ends_with(e))).collect();
    if changed.is_empty() {
        return None;
    }
    let mut h = std::collections::hash_map::DefaultHasher::new();
    git(&["diff", "HEAD", "--binary"]).hash(&mut h);
    for p in &changed {
        p.hash(&mut h);
        if let Ok(bytes) = std::fs::read(root().join(p)) {
            bytes.hash(&mut h);
        }
    }
    Some(h.finish())
}

fn stop(_input: &Value) -> Result<(), String> {
    // `stop_hook_active` would allow a single retry; the counter below allows
    // MAX_BLOCKS, which is what a real fix usually needs.
    let Some(fp) = code_fingerprint() else { return Ok(()) };
    let dir = hook_dir();
    let pass_file = dir.join("last-pass");
    let blocks_file = dir.join("blocks");
    if std::fs::read_to_string(&pass_file).ok().as_deref() == Some(&fp.to_string()) {
        return Ok(());
    }
    let exe = std::env::current_exe().map_err(|e| e.to_string())?;
    let out = Command::new(exe).current_dir(root()).args(["check", "--fast"]).output().map_err(|e| format!("check: {e}"))?;
    if out.status.success() {
        let _ = std::fs::write(&pass_file, fp.to_string());
        let _ = std::fs::remove_file(&blocks_file);
        return Ok(());
    }
    let report = String::from_utf8_lossy(&out.stdout);
    let errors: Vec<&str> = report.lines().filter(|l| l.starts_with("ERROR") || l.starts_with("FAIL")).collect();
    let blocks: u32 = std::fs::read_to_string(&blocks_file).ok().and_then(|s| s.trim().parse().ok()).unwrap_or(0);
    let text = format!("`cargo xtask check --fast` fails on the uncommitted changes:\n{}", errors.join("\n"));
    if blocks < MAX_BLOCKS {
        let _ = std::fs::write(&blocks_file, (blocks + 1).to_string());
        println!(
            "{}",
            json!({ "decision": "block", "reason": format!("{text}\nFix these before stopping (attempt {} of {MAX_BLOCKS}).", blocks + 1) })
        );
    } else {
        let _ = std::fs::remove_file(&blocks_file);
        println!("{}", json!({ "systemMessage": format!("{text}\nStill failing after {MAX_BLOCKS} attempts; stopping anyway.") }));
    }
    Ok(())
}

fn session() -> Result<(), String> {
    let log = git(&["log", "--oneline", "-5"]);
    let dirty = git(&["status", "--porcelain"]).lines().count();
    let roadmap = std::fs::read_to_string(root().join("docs/TOOLING_ROADMAP.md")).unwrap_or_default();
    let progress: String = roadmap
        .split("\n## ")
        .find(|s| s.starts_with("Progress"))
        .map(|s| s.lines().skip(1).collect::<Vec<_>>().join("\n"))
        .unwrap_or_default();
    let ctx = format!(
        "Where the work stands (re-injected after compaction):\nRecent commits:\n{log}Uncommitted files: {dirty}\nTooling roadmap progress (docs/TOOLING_ROADMAP.md):{progress}\nCLAUDE.md has the commands and invariants."
    );
    println!("{}", json!({ "hookSpecificOutput": { "hookEventName": "SessionStart", "additionalContext": ctx } }));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edits_to_the_parent_folder_and_to_fixtures_are_refused() {
        let root = Path::new("/p/OpenCarole/native");
        assert!(edit_refusal(Path::new("/p/OpenCarole/backrooms.js"), root).unwrap().contains("read-only"));
        assert!(edit_refusal(Path::new("/p/OpenCarole/native/crates/game/tests/fixtures/entities.json"), root)
            .unwrap()
            .contains("re-recording"));
        assert!(edit_refusal(Path::new("/p/OpenCarole/native/crates/backrooms/tests/reference/rs_1_L1.json"), root).is_some());
        assert_eq!(edit_refusal(Path::new("/p/OpenCarole/native/crates/game/src/lib.rs"), root), None);
        assert_eq!(edit_refusal(Path::new("/tmp/scratch/notes.md"), root), None);
    }
}
