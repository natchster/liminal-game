//! `cargo xtask hook <event>`: the Claude Code hooks in `.claude/settings.json`.
//!
//! Each reads the hook's JSON from stdin and answers in the hook protocol:
//!
//! - `pre-edit` (PreToolUse on file edits): denies edits to anything outside
//!   this repository, and hand edits to recorded fixtures and goldens — those
//!   change only through the REGEN / rerecord / accept commands — and, while
//!   the session works from words that are not all maintainers' (the task
//!   marker of `.github/trust/trust.py`), edits to the protected paths.
//! - `post-edit` (PostToolUse on file edits): notes an edited code file in
//!   the session's ledger (below), and runs rustfmt on an edited `.rs` file;
//!   a file rustfmt cannot parse is reported back (it does not block).
//! - `stop` (Stop): runs `xtask check --fast` in each checkout where this
//!   session's own edits are uncommitted and the code has not passed as it
//!   stands, and blocks the stop with the `ERROR` lines when it fails — at
//!   most `MAX_BLOCKS` times in a row, then lets the stop through with a
//!   warning so a stuck session can still hand back.
//! - `session` (SessionStart after compaction): re-injects where the work
//!   stands — recent commits, uncommitted files, the roadmap's progress table.
//!
//! Several sessions may work at once, in one checkout or in several worktrees
//! of the repository, and a session answers only for what it wrote:
//! `post-edit` keeps a ledger per session in the checkout holding the file
//! (`target/xtask/hook/sessions/<session>.edited`), and a session whose
//! ledger names no uncommitted code stops without a check, whatever another
//! session leaves half done beside it. A file changed only through the shell
//! (a `sed -i`, a re-record) is in no ledger: the session runs the check
//! itself. One check runs at a time in a checkout (`check.lock`), and any
//! check of the same code is taken, a failure as much as a pass
//! (`last-check.json`). A stop has `STOP_BUDGET` to wait and check, under the
//! hook's own timeout; what it could not finish it says, and the session's
//! ledger is kept for the next stop.
use crate::root;
use serde_json::{Value, json};
use std::collections::BTreeSet;
use std::hash::{Hash, Hasher};
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// Consecutive stop blocks before a failing check lets the session stop.
const MAX_BLOCKS: u32 = 3;

/// A session's ledger and counter untouched this long are a finished
/// session's, and are swept.
const SESSION_DAYS: u64 = 14;

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
    if !path.starts_with(&root) {
        return Some(format!(
            "{} is outside this repository ({}): edits stay inside it (CLAUDE.md, invariant 4).",
            path.display(),
            root.display()
        ));
    }
    // measured from the checkout holding it: a worktree inside this one
    // (`.claude/worktrees/…`) has fixtures and protected paths of its own
    let mut all = checkouts(&root);
    all.push(root.clone());
    let (_, rel) = checkout_of(&path, &all)?;
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
    if let Some(session) = session_id(input) {
        note_edit(&path, &session, &checkouts(&root()));
    }
    if path.extension().is_none_or(|e| e != "rs") || !path.starts_with(root()) || !path.exists() {
        return Ok(());
    }
    // the edition and style from `rustfmt.toml`, as `cargo fmt` reads them
    let out = Command::new("rustfmt").arg(&path).output().map_err(|e| format!("rustfmt: {e}"))?;
    if !out.status.success() {
        // exit 2: shown to Claude, does not undo the edit
        eprintln!("rustfmt could not format {}:\n{}", path.display(), String::from_utf8_lossy(&out.stderr));
        std::process::exit(2);
    }
    Ok(())
}

fn git(args: &[&str]) -> String {
    git_in(&root(), args)
}

fn git_in(dir: &Path, args: &[&str]) -> String {
    // no optional locks: another session's `git add` or commit must not find
    // the index taken by a hook's look at it
    Command::new("git")
        .current_dir(dir)
        .arg("--no-optional-locks")
        .args(args)
        .output()
        .map_or_default(|o| String::from_utf8_lossy(&o.stdout).into_owned())
}

/// Claude Code's `session_id`, kept to the characters a file name may hold.
fn session_id(input: &Value) -> Option<String> {
    let id: String = input["session_id"].as_str()?.chars().filter(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_')).collect();
    (!id.is_empty()).then_some(id)
}

/// The hooks' own files in `checkout` (made only when written): the last
/// check and its lock, and the sessions' ledgers and counters.
fn hook_dir(checkout: &Path) -> PathBuf {
    checkout.join("target/xtask/hook")
}

/// The code files `session` edited in `checkout` that it has not yet seen
/// pass, one path (relative to the checkout) a line.
fn ledger(checkout: &Path, session: &str) -> PathBuf {
    hook_dir(checkout).join(format!("sessions/{session}.edited"))
}

/// Every checkout of this repository, resolved: the main one and its
/// worktrees, wherever they lie.
fn checkouts(any: &Path) -> Vec<PathBuf> {
    let mut all: Vec<PathBuf> = git_in(any, &["worktree", "list", "--porcelain"])
        .lines()
        .filter_map(|l| l.strip_prefix("worktree "))
        .filter_map(|p| Path::new(p).canonicalize().ok())
        .collect();
    if all.is_empty() {
        all.push(resolve(any));
    }
    all
}

/// The checkout holding `path` (the deepest: a worktree may lie inside
/// another checkout) and the path within it.
fn checkout_of(path: &Path, checkouts: &[PathBuf]) -> Option<(PathBuf, String)> {
    let path = resolve(path);
    let c = checkouts.iter().filter(|c| path.starts_with(c)).max_by_key(|c| c.components().count())?;
    let rel = path.strip_prefix(c).ok()?.to_string_lossy().replace('\\', "/");
    Some((c.clone(), rel))
}

fn is_code(rel: &str) -> bool {
    CODE_EXT.iter().any(|e| rel.ends_with(e))
}

/// Notes in the ledger of the checkout holding `path` that `session` edited
/// it, if it is code.
fn note_edit(path: &Path, session: &str, checkouts: &[PathBuf]) {
    let Some((checkout, rel)) = checkout_of(path, checkouts) else { return };
    if !is_code(&rel) {
        return;
    }
    let ledger = ledger(&checkout, session);
    let _ = std::fs::create_dir_all(ledger.parent().unwrap_or(&checkout));
    // a whole line in one append: edits made in parallel each add their own
    // (`writeln!` could write the path and its newline apart, interleaved)
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(&ledger) {
        let _ = f.write_all(format!("{rel}\n").as_bytes());
    }
}

/// The code files of `checkout` that differ from its HEAD, tracked or new.
fn dirty_code(checkout: &Path) -> Vec<String> {
    git_in(checkout, &["status", "--porcelain=v1", "-uall"])
        .lines()
        .filter_map(|l| l.get(3..))
        .map(|p| p.rsplit(" -> ").next().unwrap_or(p).trim_matches('"').to_string())
        .filter(|p| is_code(p))
        .collect()
}

/// A fingerprint of `checkout` as a check would find it: its HEAD, its
/// uncommitted diff and its changed code files (`dirty`, new ones included).
fn fingerprint(checkout: &Path, dirty: &[String]) -> String {
    let mut h = std::collections::hash_map::DefaultHasher::new();
    git_in(checkout, &["rev-parse", "HEAD"]).hash(&mut h);
    git_in(checkout, &["diff", "HEAD", "--binary"]).hash(&mut h);
    for p in dirty {
        p.hash(&mut h);
        if let Ok(bytes) = std::fs::read(checkout.join(p)) {
            bytes.hash(&mut h);
        }
    }
    format!("{:016x}", h.finish())
}

/// How long a stop may wait and check before it says what it could not
/// finish: Claude Code kills the Stop hook at 900 s (`.claude/settings.json`).
const STOP_BUDGET: Duration = Duration::from_secs(840);

/// "this checkout", or which worktree, for a message.
fn place(checkout: &Path) -> String {
    if *checkout == resolve(&root()) { "this checkout".into() } else { format!("the worktree {}", checkout.display()) }
}

/// `cargo xtask check --fast` in `checkout`, given until `deadline`: its
/// `ERROR` and `FAIL` lines, none when it passes.
fn run_check(checkout: &Path, deadline: Instant) -> Result<Vec<String>, String> {
    use std::os::unix::process::CommandExt;
    // this binary checks the checkout it was built in; another checkout is
    // checked by its own xtask
    let mut cmd = if *checkout == resolve(&root()) {
        Command::new(std::env::current_exe().map_err(|e| e.to_string())?)
    } else {
        let mut c = Command::new("cargo");
        c.arg("xtask");
        c
    };
    // the report goes to a file: a pipe nobody reads while it runs could fill
    let report = hook_dir(checkout).join("check.out");
    let out = std::fs::File::create(&report).map_err(|e| format!("{}: {e}", report.display()))?;
    let err = out.try_clone().map_err(|e| e.to_string())?;
    // a process group of its own, so that running out of time stops its
    // builds and tests too (and with them the check's lock)
    let mut child = cmd
        .current_dir(checkout)
        .args(["check", "--fast"])
        .stdin(Stdio::null())
        .stdout(out)
        .stderr(err)
        .process_group(0)
        .spawn()
        .map_err(|e| format!("check: {e}"))?;
    let status = loop {
        if let Some(status) = child.try_wait().map_err(|e| format!("check: {e}"))? {
            break status;
        }
        if Instant::now() >= deadline {
            let _ = Command::new("kill").args(["-TERM", "--", &format!("-{}", child.id())]).status();
            let _ = child.wait();
            return Err("the check did not finish in the stop's time (a cold build?): run `cargo xtask check --fast` there".into());
        }
        std::thread::sleep(Duration::from_millis(500));
    };
    if status.success() {
        return Ok(vec![]);
    }
    let text = std::fs::read_to_string(&report).unwrap_or_default();
    let mut errors: Vec<String> = text.lines().filter(|l| l.starts_with("ERROR") || l.starts_with("FAIL")).map(String::from).collect();
    if errors.is_empty() {
        errors.push(format!("ERROR check exited with {status}: {}", text.trim().lines().last().unwrap_or("")));
    }
    Ok(errors)
}

/// Takes `lock` for this stop, waiting for another session's check until
/// `deadline`.
fn lock_until(lock: &std::fs::File, deadline: Instant) -> Result<(), String> {
    loop {
        match lock.try_lock() {
            Ok(()) => return Ok(()),
            Err(std::fs::TryLockError::WouldBlock) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(500)),
            Err(std::fs::TryLockError::WouldBlock) => {
                return Err("another session's check there was still running when the stop's time ran out: run \
                            `cargo xtask check --fast` there"
                    .into());
            }
            Err(std::fs::TryLockError::Error(e)) => return Err(format!("check.lock: {e}")),
        }
    }
}

/// A few of `paths`, for a message.
fn some_of(paths: &[String]) -> String {
    const SHOWN: usize = 8;
    let more = paths.len().saturating_sub(SHOWN);
    let shown = paths.iter().take(SHOWN).map(String::as_str).collect::<Vec<_>>().join(", ");
    if more > 0 { format!("{shown} and {more} more") } else { shown }
}

/// Takes the session's ledger for this stop, so that edits made meanwhile (a
/// subagent still at work) start a ledger of their own: what it held, with
/// what a stop cut short had taken, is now in `<session>.checking`.
fn take_ledger(ledger: &Path) -> Option<(PathBuf, BTreeSet<String>)> {
    let taken = ledger.with_extension("checking");
    let left = std::fs::read_to_string(&taken).unwrap_or_default();
    let fresh = if std::fs::rename(ledger, &taken).is_ok() { std::fs::read_to_string(&taken).unwrap_or_default() } else { String::new() };
    let edited: BTreeSet<String> = left.lines().chain(fresh.lines()).filter(|l| !l.is_empty()).map(String::from).collect();
    if edited.is_empty() {
        let _ = std::fs::remove_file(&taken);
        return None;
    }
    let _ = std::fs::write(&taken, edited.iter().map(|p| format!("{p}\n")).collect::<String>());
    Some((taken, edited))
}

/// Puts what a stop took back into the ledger, beside what was edited
/// meanwhile.
fn give_back(ledger: &Path, taken: &Path) {
    let Ok(text) = std::fs::read_to_string(taken) else { return };
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(ledger)
        && f.write_all(text.as_bytes()).is_ok()
    {
        let _ = std::fs::remove_file(taken);
    }
}

/// What a session's stop owes `checkout`: what `judge` says of the code its
/// ledger there names. Whatever it took from the ledger goes back unless the
/// stop found nothing to answer for (its edits committed, or passed).
/// `session` is None for a stop run by hand: then every uncommitted change
/// is its own.
fn stop_in(
    checkout: &Path,
    session: Option<&str>,
    deadline: Instant,
    run: &dyn Fn(&Path, Instant) -> Result<Vec<String>, String>,
) -> Result<Option<String>, String> {
    let Some(session) = session else { return judge(checkout, None, deadline, run) };
    let ledger = ledger(checkout, session);
    let Some((taken, edited)) = take_ledger(&ledger) else { return Ok(None) };
    let verdict = judge(checkout, Some(&edited), deadline, run);
    if matches!(verdict, Ok(None)) {
        let _ = std::fs::remove_file(&taken);
    } else {
        give_back(&ledger, &taken);
    }
    verdict
}

/// Nothing when no uncommitted code of `checkout` is the session's own
/// (`edited`; None: all of it is), or when the code as it stands has been
/// checked and passed; otherwise the failure, as the reason to block. Any
/// session's check of the same code counts, a failure as much as a pass:
/// the same code fails the same way, and a session blocked by others' work
/// beside its own is not made to wait for a check again each time it stops.
fn judge(
    checkout: &Path,
    edited: Option<&BTreeSet<String>>,
    deadline: Instant,
    run: &dyn Fn(&Path, Instant) -> Result<Vec<String>, String>,
) -> Result<Option<String>, String> {
    let own_of = |dirty: &[String]| -> Vec<String> { dirty.iter().filter(|p| edited.is_none_or(|e| e.contains(*p))).cloned().collect() };
    if own_of(&dirty_code(checkout)).is_empty() {
        return Ok(None);
    }
    // one check at a time in a checkout; the code is read again once it is
    // this stop's turn
    let dir = hook_dir(checkout);
    std::fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let lock = std::fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(dir.join("check.lock"))
        .map_err(|e| format!("check.lock: {e}"))?;
    lock_until(&lock, deadline)?;
    let dirty = dirty_code(checkout);
    let own = own_of(&dirty);
    if own.is_empty() {
        return Ok(None);
    }
    let fp = fingerprint(checkout, &dirty);
    let last_file = dir.join("last-check.json");
    let last: Value = std::fs::read_to_string(&last_file).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or(Value::Null);
    let errors: Vec<String> = if last["fingerprint"].as_str() == Some(fp.as_str()) {
        last["errors"].as_array().into_iter().flatten().filter_map(Value::as_str).map(String::from).collect()
    } else {
        let errors = run(checkout, deadline)?;
        let _ = std::fs::write(&last_file, json!({ "fingerprint": fp, "errors": errors }).to_string());
        errors
    };
    drop(lock);
    if errors.is_empty() {
        return Ok(None);
    }
    let mut text = format!(
        "`cargo xtask check --fast` fails in {}:\n{}\nUncommitted there, edited by this session: {}.",
        place(checkout),
        errors.join("\n"),
        some_of(&own)
    );
    let others: Vec<String> = dirty.into_iter().filter(|p| !own.contains(p)).collect();
    if !others.is_empty() {
        text += &format!(
            "\nAlso uncommitted there, not edited by this session (another session's work in progress, or changed through \
             the shell): {}. An error in those may not be yours: do not change another session's files to make the check \
             pass; say so instead.",
            some_of(&others)
        );
    }
    Ok(Some(text))
}

/// Ledgers and counters of sessions that ended long ago.
fn sweep(checkout: &Path) {
    let Ok(entries) = std::fs::read_dir(hook_dir(checkout).join("sessions")) else { return };
    for e in entries.flatten() {
        let age = e.metadata().and_then(|m| m.modified()).ok().and_then(|t| t.elapsed().ok());
        if age.is_some_and(|a| a.as_secs() > SESSION_DAYS * 86_400) {
            let _ = std::fs::remove_file(e.path());
        }
    }
}

fn stop(input: &Value) -> Result<(), String> {
    // `stop_hook_active` would allow a single retry; the counter below allows
    // MAX_BLOCKS, which is what a real fix usually needs.
    let deadline = Instant::now() + STOP_BUDGET;
    let session = session_id(input);
    let here = resolve(&root());
    sweep(&here);
    // a session answers for its edits in any checkout of the repository; a
    // stop run by hand, with no session, for every change in this one
    let checkouts = if session.is_some() { checkouts(&here) } else { vec![here.clone()] };
    let mut failures = vec![];
    for c in &checkouts {
        match stop_in(c, session.as_deref(), deadline, &run_check) {
            Ok(failure) => failures.extend(failure),
            // one checkout left unchecked keeps what the others found
            Err(e) => failures.push(format!("Not checked in {}: {e}", place(c))),
        }
    }
    let blocks_file = hook_dir(&here).join(format!("sessions/{}.blocks", session.as_deref().unwrap_or("by-hand")));
    if failures.is_empty() {
        let _ = std::fs::remove_file(&blocks_file);
        return Ok(());
    }
    let _ = std::fs::create_dir_all(hook_dir(&here).join("sessions"));
    let blocks: u32 = std::fs::read_to_string(&blocks_file).ok().and_then(|s| s.trim().parse().ok()).unwrap_or(0);
    let text = failures.join("\n\n");
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
    let progress: String =
        roadmap.split("\n## ").find(|s| s.starts_with("Progress")).map_or_default(|s| s.lines().skip(1).collect::<Vec<_>>().join("\n"));
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

    #[test]
    fn an_edit_belongs_to_the_deepest_checkout_holding_it() {
        let all = [PathBuf::from("/p/game"), PathBuf::from("/p/game/.claude/worktrees/w"), PathBuf::from("/q/wt")];
        let of = |p: &str| checkout_of(Path::new(p), &all).map(|(c, rel)| (c.display().to_string(), rel));
        assert_eq!(of("/p/game/xtask/src/hook.rs"), Some(("/p/game".into(), "xtask/src/hook.rs".into())));
        assert_eq!(of("/p/game/.claude/worktrees/w/src/a.rs"), Some(("/p/game/.claude/worktrees/w".into(), "src/a.rs".into())));
        assert_eq!(of("/q/wt/./src/../b.rs"), Some(("/q/wt".into(), "b.rs".into())));
        assert_eq!(of("/p/gamer/a.rs"), None);
    }

    /// `git` in `dir`, which must succeed.
    fn git_ok(dir: &Path, args: &[&str]) {
        let ok = Command::new("git")
            .current_dir(dir)
            .args(["-c", "user.name=t", "-c", "user.email=t@t", "-c", "commit.gpgsign=false"])
            .args(args)
            .output()
            .is_ok_and(|o| o.status.success());
        assert!(ok, "git {args:?} in {}", dir.display());
    }

    #[test]
    fn a_session_answers_for_its_own_edits_in_any_worktree() {
        use std::cell::RefCell;
        let main = std::env::temp_dir().join(format!("xtask-hook-stop-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&main);
        std::fs::create_dir_all(&main).unwrap();
        let main = resolve(&main);
        for f in ["a.rs", "b.rs", "notes.md"] {
            std::fs::write(main.join(f), "fn f() {}\n").unwrap();
        }
        // as in the repository: the hook's own files are not code
        std::fs::write(main.join(".gitignore"), "target/\n").unwrap();
        git_ok(&main, &["init", "-q"]);
        git_ok(&main, &["add", "."]);
        git_ok(&main, &["commit", "-qm", "start"]);
        // a worktree inside the main checkout, as `claude --worktree` makes them
        git_ok(&main, &["worktree", "add", "-q", "--detach", "wt"]);
        let wt = main.join("wt");
        let all = checkouts(&main);
        assert_eq!(all, vec![main.clone(), wt.clone()]);
        // its fixtures are refused as fixtures, measured from its own checkout
        assert!(edit_refusal(&wt.join("crates/game/tests/fixtures/x.json"), &main).unwrap().contains("re-recording"));
        assert_eq!(edit_refusal(&wt.join("crates/game/src/lib.rs"), &main), None);

        // session s1 edits a.rs; another session edits b.rs; s2 edits nothing
        std::fs::write(main.join("a.rs"), "fn f() { 1 }\n").unwrap();
        std::fs::write(main.join("b.rs"), "fn f() { 2 }\n").unwrap();
        note_edit(&main.join("a.rs"), "s1", &all);
        note_edit(&main.join("notes.md"), "s1", &all); // not code: not noted
        assert_eq!(std::fs::read_to_string(ledger(&main, "s1")).unwrap(), "a.rs\n");

        let runs = RefCell::new(vec![]);
        // a check that fails or passes, while `meanwhile` is edited (a subagent at work)
        let check = |errors: &'static [&'static str], meanwhile: Option<&'static str>| {
            let (runs, main, all) = (&runs, &main, &all);
            move |c: &Path, _: Instant| -> Result<Vec<String>, String> {
                runs.borrow_mut().push(c.to_path_buf());
                if let Some(f) = meanwhile {
                    note_edit(&main.join(f), "s1", all);
                }
                Ok(errors.iter().map(|e| e.to_string()).collect())
            }
        };
        let fails = check(&["ERROR test x"], None);
        let in_time = Instant::now() + Duration::from_secs(60);
        let lines = |l: &Path| -> Vec<String> {
            let mut v: Vec<String> = std::fs::read_to_string(l).unwrap_or_default().lines().map(String::from).collect();
            v.sort();
            v.dedup();
            v
        };

        // s2 has edited nothing: it stops without a check, whatever b.rs holds
        assert_eq!(stop_in(&main, Some("s2"), in_time, &fails).unwrap(), None);
        assert!(runs.borrow().is_empty());
        // s1 is checked, and told which uncommitted code is not its own
        let why = stop_in(&main, Some("s1"), in_time, &fails).unwrap().unwrap();
        assert!(why.contains("ERROR test x") && why.contains("edited by this session: a.rs."), "{why}");
        assert!(why.contains("not edited by this session") && why.contains("b.rs"), "{why}");
        assert_eq!(runs.borrow().len(), 1);
        assert_eq!(lines(&ledger(&main, "s1")), ["a.rs"]);
        // the same code is not checked again: its failure stands
        assert!(stop_in(&main, Some("s1"), in_time, &fails).unwrap().is_some());
        assert_eq!(runs.borrow().len(), 1);

        // changed code is checked again; what was edited during the check
        // stays in the ledger after it passes, and passes as the same code
        std::fs::write(main.join("a.rs"), "fn f() { 5 }\n").unwrap();
        assert_eq!(stop_in(&main, Some("s1"), in_time, &check(&[], Some("b.rs"))).unwrap(), None);
        assert_eq!(runs.borrow().len(), 2);
        assert_eq!(lines(&ledger(&main, "s1")), ["b.rs"]);
        assert_eq!(stop_in(&main, Some("s1"), in_time, &fails).unwrap(), None);
        assert_eq!(runs.borrow().len(), 2);
        assert!(!ledger(&main, "s1").exists());

        // a failure gives the ledger back, with what was edited meanwhile
        note_edit(&main.join("a.rs"), "s1", &all);
        std::fs::write(main.join("a.rs"), "fn f() { 6 }\n").unwrap();
        assert!(stop_in(&main, Some("s1"), in_time, &check(&["ERROR test y"], Some("b.rs"))).unwrap().is_some());
        assert_eq!(lines(&ledger(&main, "s1")), ["a.rs", "b.rs"]);
        assert!(!ledger(&main, "s1").with_extension("checking").exists());

        // out of time behind another session's check: not checked, and the
        // ledger is kept for the next stop
        std::fs::write(main.join("a.rs"), "fn f() { 7 }\n").unwrap();
        let held = std::fs::File::create(hook_dir(&main).join("check.lock")).unwrap();
        held.lock().unwrap();
        let late = stop_in(&main, Some("s1"), Instant::now(), &fails).unwrap_err();
        assert!(late.contains("time ran out"), "{late}");
        drop(held);
        assert_eq!(lines(&ledger(&main, "s1")), ["a.rs", "b.rs"]);
        assert_eq!(runs.borrow().len(), 3);

        // an edit in the worktree is checked there, not in the main checkout
        std::fs::write(wt.join("a.rs"), "fn f() { 3 }\n").unwrap();
        note_edit(&wt.join("a.rs"), "s1", &all);
        assert_eq!(std::fs::read_to_string(ledger(&wt, "s1")).unwrap(), "a.rs\n");
        assert!(stop_in(&wt, Some("s1"), in_time, &fails).unwrap().is_some());
        assert_eq!(runs.borrow().last(), Some(&wt));

        // edits since committed are no longer the session's to answer for
        git_ok(&wt, &["commit", "-qam", "wt"]);
        assert_eq!(stop_in(&wt, Some("s1"), in_time, &fails).unwrap(), None);
        assert!(!ledger(&wt, "s1").exists());
        // by hand, with no session, every uncommitted change is its own
        let why = stop_in(&main, None, in_time, &fails).unwrap().unwrap();
        assert!(why.contains("a.rs, b.rs") && !why.contains("not edited"), "{why}");
        let _ = std::fs::remove_dir_all(&main);
    }
}
