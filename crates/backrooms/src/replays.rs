//! Replays in the app (`docs/engineering/REPLAYS.md`): what a recording
//! keeps of the build that made it, where recordings go and how they are
//! written, `--replay-verify`, `--dump-replay` and `--bench-replay`.
use game::Game;
use game::replay::{Player, ReplayFile, Start};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Sender, channel};
use std::thread::JoinHandle;

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

/// `ms` since 1970 as a UTC date and time a file name can hold
/// (`2026-11-14_20-43-05`; Howard Hinnant's days-to-civil).
pub fn utc_stamp(ms: u64) -> String {
    let secs = ms / 1000;
    let (days, sod) = ((secs / 86_400) as i64, secs % 86_400);
    let z = days + 719_468;
    let (era, doe) = (z.div_euclid(146_097), z.rem_euclid(146_097));
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = yoe + era * 400 + i64::from(m <= 2);
    format!("{y:04}-{m:02}-{d:02}_{:02}-{:02}-{:02}", sod / 3600, sod % 3600 / 60, sod % 60)
}

/// Where the game keeps the recordings it makes (beside its storage).
pub fn folder(storage: &Path) -> PathBuf {
    storage.parent().unwrap_or(Path::new(".")).join("replays")
}

/// A new recording's file: when it began (UTC) and the floor it began on.
pub fn new_path(storage: &Path, level: i32) -> PathBuf {
    let ms = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_millis() as u64);
    folder(storage).join(format!("{}_L{level}.ocreplay", utc_stamp(ms)))
}

/// Writes a recording whole: beside it first, then in its place, so a
/// reader never sees half of one.
pub fn write(path: &Path, f: &ReplayFile) -> Result<(), String> {
    if let Some(d) = path.parent() {
        std::fs::create_dir_all(d).map_err(|e| format!("{}: {e}", d.display()))?;
    }
    let part = path.with_extension("ocreplay.part");
    std::fs::write(&part, f.encode()).map_err(|e| format!("{}: {e}", part.display()))?;
    std::fs::rename(&part, path).map_err(|e| format!("{}: {e}", path.display()))
}

/// Writes recordings off the frame, one after another, in the order given
/// (encoding a long one takes longer than a frame has).
pub struct Writer {
    tx: Option<Sender<(PathBuf, ReplayFile)>>,
    thread: Option<JoinHandle<()>>,
}

impl Writer {
    pub fn new() -> Writer {
        let (tx, rx) = channel::<(PathBuf, ReplayFile)>();
        let thread = std::thread::spawn(move || {
            for (p, f) in rx {
                if let Err(e) = write(&p, &f) {
                    eprintln!("recording not saved: {e}");
                }
            }
        });
        Writer { tx: Some(tx), thread: Some(thread) }
    }
    pub fn send(&self, path: PathBuf, f: ReplayFile) {
        if let Some(tx) = &self.tx {
            let _ = tx.send((path, f));
        }
    }
    /// Everything sent is written: the thread ends (when the app quits).
    pub fn finish(&mut self) {
        self.tx.take();
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}

/// The window's view a recording starts in: its session's, or the view its
/// first floor's anchor kept.
pub fn view_of(f: &ReplayFile) -> Option<[f64; 3]> {
    match &f.segments.first()?.start {
        Start::Session { view, .. } => Some(*view),
        Start::Floor(a) => {
            let v: Vec<f64> = a.state["view"].as_array()?.iter().filter_map(Value::as_u64).map(f64::from_bits).collect();
            <[f64; 3]>::try_from(v).ok()
        }
    }
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
pub fn verify(path: &str, make: fn(&str) -> Game) -> Result<(), String> {
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

/// `--bench-replay OUT.json` (`cargo xtask bench`): what recording and
/// replaying cost. A session as the app records one (a fresh process at the
/// title, New descent, the way in, then `frames` frames of the restless bot
/// at an even 60 a second), the log's work timed after every frame and its
/// memory weighed at the end; then the session replayed headless from its
/// start, timed, and checked.
pub fn bench(out: &str, seed: u32, frames: usize, make: fn(&str) -> Game) -> Result<(), String> {
    use game::replay::{FrameInput, InputEvent, Recorder};
    use std::time::Instant;
    let flags = format!("seed={seed}&nosound");
    let native = 0x0BE4_C4ED;
    game::base::rng::seed_native(native);
    let mut g = make(&flags);
    let storage = g.storage.clone();
    g.boot_in_view(Some([1280.0, 800.0, 1.0]));
    let mut log = Recorder::session(native, &flags, storage, &g);
    let bot = game::debug::bot::restless(frames, seed);
    let ticks = game::base::game::clock_ticks(1.0 / 60.0);
    let (mut k, mut played, mut play_s, mut log_s) = (0usize, None::<usize>, 0.0, 0.0);
    loop {
        let mut i = FrameInput::default();
        match k {
            0 => i.events.push(InputEvent::Click("menu:new".into())),
            1 => i.events.push(InputEvent::Lock(true)),
            _ if played.is_none() && (g.state != "playing" || g.hands_taken()) => {}
            _ => {
                let b = *played.get_or_insert(k);
                let Some(next) = bot.get(k - b) else { break };
                i = next.clone();
            }
        }
        if k > frames + 5000 {
            return Err("the way in never gave the bot its hands".into());
        }
        let t = Instant::now();
        g.apply_input(&i);
        let dt = g.advance_clock(ticks);
        g.frame(dt);
        let t1 = Instant::now();
        for e in i.events {
            log.push(e);
        }
        log.end_frame(&mut g, ticks);
        play_s += (t1 - t).as_secs_f64();
        log_s += t1.elapsed().as_secs_f64();
        k += 1;
    }
    let minutes = k as f64 / 3600.0;
    let log_kb_min = log.heap_bytes() as f64 / 1024.0 / minutes;
    let file = log.snapshot(header(&flags));
    let bytes = file.encode().len();
    let (mut r, mut p) = Player::start(file, 0, make)?;
    let t = Instant::now();
    let mut n = 0u64;
    while p.step(&mut r) {
        n += 1;
    }
    let replay_s = t.elapsed().as_secs_f64();
    if let Some(d) = p.diverged {
        return Err(format!("the bench's replay left its recording at frame {} in {:?}", d.frame, d.parts));
    }
    let us = |s: f64, n: f64| s * 1e6 / n;
    let v = json!({
        "frames": k,
        "play_frame_us": us(play_s, k as f64),
        "record_frame_us": us(log_s, k as f64),
        "replay_frame_us": us(replay_s, n as f64),
        "log_kb_min": log_kb_min,
        "file_kb_min": bytes as f64 / 1024.0 / minutes,
    });
    println!("{v}");
    std::fs::write(out, v.to_string()).map_err(|e| format!("{out}: {e}"))
}
