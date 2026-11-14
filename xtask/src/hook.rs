//! `cargo xtask hook <event>`: the Claude Code hooks in `.claude/settings.json`.
//!
//! Each reads the hook's JSON from stdin and answers in the hook protocol:
//!
//! - `pre-edit` (PreToolUse on file edits): denies edits to anything outside
//!   this repository, and hand edits to recorded fixtures and goldens — those
//!   change only through the REGEN / rerecord / accept commands — and, while
//!   the session works from words that are not all maintainers' (the task
//!   marker of `.github/trust/trust.py`), edits to the protected paths.
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
use std::path::{Component, Path, PathBuf};
use std::process::Command;

/// Consecutive stop blocks before a failing check lets the session stop.
const MAX_BLOCKS: u32 = 3;

/// Paths (relative to the workspace) that only re-recorders may write.
const RECORDED: &[&str] = &[
    "crates/game/tests/fixtures/",
    "crates/backrooms/tests/reference/",
    "crates/backrooms/tests/recorded/",
    "crates/backrooms/tests/golden/",
    "crates/placers/tests/fixtures/",
];

/// Changes to these make the stop hook run the check (docs alone do not).
const CODE_EXT: &[&str] = &[".rs", ".toml", ".metal", ".lock", ".json", ".bin", ".jpg", ".txt"];

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

/// `p` with `.` and `..` taken out and its longest existing ancestor
/// resolved (symlinks such as macOS's `/tmp`), so a path can be compared with
/// the repository's whatever way it was written. A file not yet written
/// keeps its own name under its resolved folder.
fn resolve(p: &Path) -> PathBuf {
    let mut lexical = PathBuf::new();
    for c in p.components() {
        match c {
            Component::CurDir => {}
            Component::ParentDir => {
                lexical.pop();
            }
            c => lexical.push(c),
        }
    }
    let mut rest = vec![];
    let mut base = lexical.as_path();
    loop {
        if let Ok(real) = base.canonicalize() {
            return rest.iter().rev().fold(real, |acc: PathBuf, c| acc.join(c));
        }
        match (base.parent(), base.file_name()) {
            (Some(up), Some(name)) => {
                rest.push(name.to_os_string());
                base = up;
            }
            _ => return lexical,
        }
    }
}

/// Why an edit of `path` must not happen, if it must not: it is outside
/// this repository, or it is recorded reference data.
fn edit_refusal(path: &Path, root: &Path) -> Option<String> {
    let (path, root) = (resolve(path), resolve(root));
    let Ok(rel) = path.strip_prefix(&root) else {
        return Some(format!(
            "{} is outside this repository ({}): edits stay inside it (CLAUDE.md, invariant 4).",
            path.display(),
            root.display()
        ));
    };
    let rel = rel.to_string_lossy().replace('\\', "/");
    if RECORDED.iter().any(|r| rel.starts_with(r)) {
        return Some(format!(
            "{rel} is recorded reference data: change it only by re-recording after an intended change \
             (REGEN=1 / `-- --ignored rerecord` / `xtask shots --accept`; see CLAUDE.md), then review the diff."
        ));
    }
    untrusted_task_refusal(&rel, &root)
}

/// What `.github/trust/trust.py issue N --mark` writes when a session works
/// from words that are not all maintainers' (docs/engineering/TRUST.md): the
/// issue, why, and the protected paths, which it then may not edit.
const TRUST_TASK: &str = "target/trust/task.json";

/// Why `rel` must not be edited while the task marker stands: it is a
/// protected path. Only the owner clears the marker; the trust gate holds
/// the same line on GitHub, this only stops the edit where it starts.
fn untrusted_task_refusal(rel: &str, root: &Path) -> Option<String> {
    let task: Value = serde_json::from_str(&std::fs::read_to_string(root.join(TRUST_TASK)).ok()?).ok()?;
    let folded = rel.to_lowercase();
    let glob = task["protected"]
        .as_array()?
        .iter()
        .filter_map(Value::as_str)
        .find(|g| glob_match(g.to_lowercase().as_bytes(), folded.as_bytes()))?;
    let issues = task["issues"].as_array()?;
    let numbers: Vec<String> = issues.iter().map(|i| format!("#{}", i["number"])).collect();
    let why: Vec<&str> = issues.iter().flat_map(|i| i["why"].as_array().into_iter().flatten().filter_map(Value::as_str)).collect();
    let numbers = numbers.join(", ");
    Some(format!(
        "{rel} is a protected path ({glob} in .github/trust/policy.json), and this session works on {numbers}, whose \
         words are not all maintainers' ({}). Protected paths change only by the owner, or after a maintainer's \
         `/trust-protected <sha>` on the pull request: stop and tell the user. If this session is not working on \
         {numbers}, the owner clears the marker ({TRUST_TASK}) with `python3 .github/trust/trust.py clear`.",
        why.join("; ")
    ))
}

/// `glob` matches `path` the way `.github/trust/trust.py` matches them:
/// `**/` any folders or none, `**` anything, `*` anything but `/`, `?` one
/// character but `/`.
fn glob_match(glob: &[u8], path: &[u8]) -> bool {
    match glob {
        [] => path.is_empty(),
        [b'*', b'*', b'/', rest @ ..] => {
            glob_match(rest, path) || path.iter().enumerate().any(|(i, c)| *c == b'/' && glob_match(rest, &path[i + 1..]))
        }
        [b'*', b'*', rest @ ..] => (0..=path.len()).any(|i| glob_match(rest, &path[i..])),
        [b'*', rest @ ..] => (0..=path.len()).take_while(|&i| i == 0 || path[i - 1] != b'/').any(|i| glob_match(rest, &path[i..])),
        [b'?', rest @ ..] => matches!(path, [c, ..] if *c != b'/') && glob_match(rest, &path[1..]),
        [g, rest @ ..] => path.first() == Some(g) && glob_match(rest, &path[1..]),
    }
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
    let roadmap = std::fs::read_to_string(root().join("docs/plan/TOOLING_ROADMAP.md")).unwrap_or_default();
    let progress: String = roadmap
        .split("\n## ")
        .find(|s| s.starts_with("Progress"))
        .map(|s| s.lines().skip(1).collect::<Vec<_>>().join("\n"))
        .unwrap_or_default();
    let ctx = format!(
        "Where the work stands (re-injected after compaction):\nRecent commits:\n{log}Uncommitted files: {dirty}\nTooling roadmap progress (docs/plan/TOOLING_ROADMAP.md):{progress}\nCLAUDE.md has the commands and invariants."
    );
    println!("{}", json!({ "hookSpecificOutput": { "hookEventName": "SessionStart", "additionalContext": ctx } }));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edits_outside_the_repository_and_to_fixtures_are_refused() {
        let root = Path::new("/p/work/game");
        // anything outside: the folder above, a sibling, anywhere else
        for outside in ["/p/work/notes.md", "/p/work/game-old/src/lib.rs", "/p/work/game/../other/x.rs", "/elsewhere/scratch/notes.md"] {
            assert!(edit_refusal(Path::new(outside), root).unwrap().contains("outside this repository"), "{outside}");
        }
        assert!(edit_refusal(Path::new("/p/work/game/crates/game/tests/fixtures/entities.json"), root).unwrap().contains("re-recording"));
        assert!(edit_refusal(Path::new("/p/work/game/crates/backrooms/tests/reference/rs_1_L1.json"), root).is_some());
        assert!(edit_refusal(Path::new("/p/work/game/crates/placers/tests/fixtures/fingerprints.tsv"), root).is_some());
        assert_eq!(edit_refusal(Path::new("/p/work/game/crates/game/src/lib.rs"), root), None);
        assert_eq!(edit_refusal(Path::new("/p/work/game/./docs/x/../DEVELOPMENT.md"), root), None);
    }

    #[test]
    fn the_repository_itself_is_inside_however_it_is_written() {
        let root = root();
        assert_eq!(edit_refusal(&root.join("crates/game/src/lib.rs"), &root), None);
        assert_eq!(edit_refusal(&root.join("xtask/../docs/new-note.md"), &root), None);
        let beside = root.parent().unwrap().join("beside-the-repository.txt");
        assert!(edit_refusal(&beside, &root).is_some(), "{} is outside", beside.display());
        assert!(edit_refusal(&root.join("../beside-the-repository.txt"), &root).is_some());
    }

    #[test]
    fn globs_match_as_the_trust_tool_matches_them() {
        let m = |g: &str, p: &str| glob_match(g.as_bytes(), p.as_bytes());
        assert!(m("**/claude.md", "claude.md") && m("**/claude.md", "crates/game/claude.md"));
        assert!(!m("**/claude.md", "crates/gameclaude.md"));
        assert!(m(".claude/**", ".claude/skills/x/SKILL.md") && !m(".claude/**", ".claudex/a"));
        assert!(m("a/*.rs", "a/x.rs") && !m("a/*.rs", "a/b/x.rs"));
        assert!(m("a?c", "abc") && !m("a?c", "a/c") && !m("a?c", "ac"));
        assert!(m("xtask/src/hook.rs", "xtask/src/hook.rs") && !m("xtask/src/hook.rs", "xtask/src/check.rs"));
    }

    #[test]
    fn protected_paths_are_refused_while_an_untrusted_task_stands() {
        let root = std::env::temp_dir().join(format!("xtask-hook-trust-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("target/trust")).unwrap();
        let root = resolve(&root);
        // no marker: nothing refused
        assert_eq!(edit_refusal(&root.join(".claude/settings.json"), &root), None);
        let task = json!({ "issues": [{ "number": 12, "why": ["#12 was opened by @someone"] }], "protected": [".claude/**", "**/CLAUDE.md", "xtask/src/hook.rs"] });
        std::fs::write(root.join(TRUST_TASK), task.to_string()).unwrap();
        for p in [".claude/settings.json", ".Claude/skills/x/SKILL.md", "CLAUDE.md", "crates/game/CLAUDE.md", "xtask/src/hook.rs"] {
            let why = edit_refusal(&root.join(p), &root).unwrap_or_default();
            assert!(why.contains("protected path") && why.contains("#12"), "{p}: {why}");
        }
        for p in ["crates/game/src/lib.rs", "docs/engineering/TRUST.md", "xtask/src/check.rs"] {
            assert_eq!(edit_refusal(&root.join(p), &root), None, "{p}");
        }
        // an unreadable marker refuses nothing (the gate on GitHub still holds)
        std::fs::write(root.join(TRUST_TASK), "not json").unwrap();
        assert_eq!(edit_refusal(&root.join(".claude/settings.json"), &root), None);
        let _ = std::fs::remove_dir_all(&root);
    }
}
