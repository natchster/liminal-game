//! Replays from the command line (`docs/engineering/REPLAYS.md`): what a
//! recording keeps of the build that made it, `--replay-verify` and
//! `--dump-replay`.
use game::Game;
use game::replay::{Player, ReplayFile, Start};
use serde_json::{Value, json};

/// The executable's own bytes, hashed (FNV-1a 64): the build exactly.
fn exe_id() -> String {
    let Some(bytes) = std::env::current_exe().ok().and_then(|p| std::fs::read(p).ok()) else { return String::new() };
    let h = bytes.iter().fold(0xcbf2_9ce4_8422_2325u64, |h, b| (h ^ u64::from(*b)).wrapping_mul(0x0000_0100_0000_01b3));
    format!("{h:016x}")
}

fn macos() -> String {
    std::process::Command::new("sw_vers")
        .arg("-productVersion")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default()
}

/// What a recording keeps of the build that made it: a replay repeats only
/// on the same build, so a player is told when it was another.
pub fn build() -> Value {
    json!({
        "version": env!("CARGO_PKG_VERSION"),
        "commit": env!("BACKROOMS_COMMIT"),
        "rustc": env!("BACKROOMS_RUSTC"),
        "exe": exe_id(),
        "macos": macos(),
    })
}

/// A new recording's header: the build, when, the launch flags.
pub fn header(flags: &str) -> Value {
    let created = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_millis() as u64);
    json!({ "created_ms": created, "build": build(), "flags": flags })
}

/// What to tell a player when `f` was made by another build than this one
/// (it may not repeat), or nothing when the executable is the same.
pub fn other_build(f: &ReplayFile) -> Option<String> {
    let (was, here) = (&f.header["build"], build());
    (was["exe"] != here["exe"]).then(|| {
        format!(
            "made by another build ({} {}, this is {} {}): it may not repeat",
            was["version"], was["commit"], here["version"], here["commit"]
        )
    })
}

pub fn read(path: &str) -> Result<ReplayFile, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("{path}: {e}"))?;
    ReplayFile::decode(&bytes).map_err(|e| format!("{path}: {e}"))
}

/// `--dump-replay FILE`: the recording as JSON, on stdout.
pub fn dump(path: &str) -> Result<(), String> {
    let f = read(path)?;
    println!("{}", serde_json::to_string_pretty(&f.to_json()).map_err(|e| e.to_string())?);
    Ok(())
}

/// `--replay-verify FILE`: the recording played headless from its start and
/// from each floor's anchor in it, each in a thread of its own (a fresh
/// game), every kept state compared. Ok if every one repeats.
pub fn verify(path: &str, make: fn(&str, Option<[f64; 3]>) -> Game) -> Result<(), String> {
    let f = read(path)?;
    if let Some(told) = other_build(&f) {
        println!("{told}");
    }
    let total = f.frame_count();
    let mut failed = 0;
    for (k, seg) in f.segments.iter().enumerate() {
        let what = match &seg.start {
            Start::Session { .. } => "from the start".to_string(),
            Start::Floor(a) => format!("from L{}'s anchor ({})", a.level, a.how.id()),
        };
        let file = f.clone();
        let result = std::thread::Builder::new()
            .stack_size(256 << 20)
            .spawn(move || -> Result<(u64, Option<game::replay::Divergence>), String> {
                let (mut g, mut p) = Player::start(file, k, make)?;
                let mut n = 0;
                while p.step(&mut g) {
                    n += 1;
                }
                Ok((n, p.diverged))
            })
            .map_err(|e| e.to_string())?
            .join()
            .map_err(|_| "the replay panicked".to_string())?;
        match result {
            Ok((n, None)) => println!("segment {k}, {what}: {n} frames to the end, every kept state as recorded"),
            Ok((_, Some(d))) => {
                failed += 1;
                println!("segment {k}, {what}: leaves the recording at frame {} of {total}, in {:?}", d.frame, d.parts);
            }
            Err(e) => {
                failed += 1;
                println!("segment {k}, {what}: {e}");
            }
        }
    }
    if failed == 0 { Ok(()) } else { Err(format!("{failed} of {} segments did not repeat", f.segments.len())) }
}
