//! Replays in the app (`docs/engineering/REPLAYS.md`): what a recording
//! keeps of the build that made it, where recordings go and how they are
//! written, `--replay-verify`, `--dump-replay` and `--bench-replay`.
use game::Game;
use game::replay::{Player, ReplayFile, Start};
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, Sender, channel};
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

static BUILD: std::sync::OnceLock<Value> = std::sync::OnceLock::new();

/// What a recording keeps of the build that made it: a replay repeats only
/// on the same build, so a player is told when it was another. Worked out
/// once (it hashes the executable and asks the system its version: about
/// 60 ms), off the frame thread when `warm` was called at the start.
pub fn build() -> Value {
    BUILD
        .get_or_init(|| {
            json!({
                "version": env!("CARGO_PKG_VERSION"),
                "commit": env!("BACKROOMS_COMMIT"),
                "rustc": env!("BACKROOMS_RUSTC"),
                "exe": exe_id(),
                "macos": macos(),
            })
        })
        .clone()
}

/// Starts working out `build` on a thread of its own, so a recording begun
/// later (F9) does not wait for it on the frame.
pub fn warm() {
    std::thread::spawn(|| drop(build()));
}

/// A new recording's header: the build, when, the launch flags, the floor
/// it begins on.
pub fn header(flags: &str, level: i32) -> Value {
    let created = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map_or(0, |d| d.as_millis() as u64);
    json!({ "created_ms": created, "build": build(), "flags": flags, "level": level })
}

/// What to tell a player when `f` was made by another build than this one
/// (it may not repeat), or nothing when the executable is the same.
pub fn other_build(f: &ReplayFile) -> Option<String> {
    let (was, here) = (&f.header["build"], build());
    let s = |v: &Value| v.as_str().unwrap_or("?").to_string();
    (was["exe"] != here["exe"]).then(|| {
        format!(
            "Made by another build ({} {}; this is {} {}): it may not play as it was played",
            s(&was["version"]),
            s(&was["commit"]),
            s(&here["version"]),
            s(&here["commit"])
        )
    })
}

/// Seconds since 1970 as (year, month, day, seconds of the day): Howard
/// Hinnant's days-to-civil.
fn civil(secs: i64) -> (i64, i64, i64, i64) {
    let (days, sod) = (secs.div_euclid(86_400), secs.rem_euclid(86_400));
    let z = days + 719_468;
    let (era, doe) = (z.div_euclid(146_097), z.rem_euclid(146_097));
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (yoe + era * 400 + i64::from(m <= 2), m, d, sod)
}

/// `ms` since 1970 as a UTC date and time a file name can hold
/// (`2026-11-14_20-43-05`).
pub fn utc_stamp(ms: u64) -> String {
    let (y, m, d, s) = civil((ms / 1000) as i64);
    format!("{y:04}-{m:02}-{d:02}_{:02}-{:02}-{:02}", s / 3600, s % 3600 / 60, s % 60)
}

/// This Mac's offset from UTC, in seconds (`date +%z`, asked once).
fn utc_offset() -> i64 {
    static OFFSET: std::sync::OnceLock<i64> = std::sync::OnceLock::new();
    *OFFSET.get_or_init(|| {
        let z = std::process::Command::new("date")
            .arg("+%z")
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            .unwrap_or_default();
        let (sign, digits) = if let Some(d) = z.strip_prefix('-') { (-1, d) } else { (1, z.trim_start_matches('+')) };
        let n: i64 = digits.parse().unwrap_or(0);
        sign * (n / 100 * 3600 + n % 100 * 60)
    })
}

/// `ms` since 1970 as the Replays screen says it, in this Mac's time
/// (`14 Nov, 20:43`).
pub fn local_day(ms: u64) -> String {
    const MONTHS: [&str; 12] = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
    let (_, m, d, s) = civil((ms / 1000) as i64 + utc_offset());
    format!("{d} {}, {:02}:{:02}", MONTHS[(m - 1) as usize], s / 3600, s % 3600 / 60)
}

/// The recordings in `folder`, newest first, as the Replays screen lists
/// them: the floor each began on, when, how long, and whether another
/// build made it. Only each header is read.
pub fn list(folder: &Path) -> Vec<game::replay::viewer::ReplayLine> {
    let here = build()["exe"].clone();
    let head = |p: &Path| game::replay::file::peek_header(std::io::BufReader::new(std::fs::File::open(p).ok()?)).ok();
    let mut found: Vec<(u64, game::replay::viewer::ReplayLine)> = std::fs::read_dir(folder)
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "ocreplay"))
        .filter_map(|path| {
            let h = head(&path)?;
            let created = h["created_ms"].as_u64().unwrap_or(0);
            let mut detail = format!("{} · {}", local_day(created), game::replay::viewer::clock(h["seconds"].as_f64().unwrap_or(0.0)));
            if h["build"]["exe"] != here {
                detail.push_str(" · another build");
            }
            let label = match h["level"].as_i64() {
                Some(l) if l > 0 => format!("Floor {l}"),
                _ => "The title".to_string(),
            };
            Some((created, game::replay::viewer::ReplayLine { path, label, detail }))
        })
        .collect();
    found.sort_by_key(|a| std::cmp::Reverse(a.0));
    found.into_iter().map(|f| f.1).collect()
}

/// Where the game keeps the recordings it makes (beside its storage).
pub fn folder(storage: &Path) -> PathBuf {
    storage.parent().unwrap_or(Path::new(".")).join("replays")
}

/// The save named `name` (`default` unless `--save NAME`), beside the
/// storage: `saves/<name>.ocsave` (`docs/engineering/SAVES.md`, V1).
pub fn save_slot(storage: &Path, name: &str) -> PathBuf {
    storage.parent().unwrap_or(Path::new(".")).join("saves").join(format!("{name}.ocsave"))
}

/// A save read, if its file is there and sound (another build's is used:
/// a save holds no frames to repeat).
pub fn read_save(path: &Path) -> Option<ReplayFile> {
    let bytes = std::fs::read(path).ok()?;
    match ReplayFile::decode(&bytes) {
        Ok(f) => Some(f),
        Err(e) => {
            eprintln!("{}: {e} (not used)", path.display());
            None
        }
    }
}

/// `--save-from-replay FILE --at S --save-to OUT`: the recording as a save
/// that resumes `at` seconds after its in-point (`ReplayFile::cut`).
pub fn save_from_replay(file: &str, at: f64, out: &str, make: fn(&str) -> Game) -> Result<(), String> {
    let f = read(file)?;
    let (mut g, mut v) = game::replay::viewer::Viewer::open(f, make)?;
    v.seek_to(&mut g, at, make)?;
    let s = v.save_here();
    write(Path::new(out), &s)?;
    println!("{out}: a save of {file} at {} (frame {}), floor {}", game::replay::viewer::clock(v.bar().at), v.player.frame, g.level_num);
    Ok(())
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

/// Writes recordings and saves off the frame, one after another, in the
/// order given (encoding a long one takes longer than a frame has; a save
/// let go of is removed in its turn, so a write before cannot bring it
/// back), and says how each one it was asked about went.
pub struct Writer {
    tx: Option<Sender<(PathBuf, Option<ReplayFile>, bool)>>,
    thread: Option<JoinHandle<()>>,
    done: Receiver<(PathBuf, Result<(), String>)>,
}

impl Writer {
    pub fn new() -> Writer {
        let (tx, rx) = channel::<(PathBuf, Option<ReplayFile>, bool)>();
        let (told, done) = channel();
        let thread = std::thread::spawn(move || {
            for (p, f, tell) in rx {
                let r = match f {
                    Some(f) => write(&p, &f),
                    None => std::fs::remove_file(&p)
                        .or_else(|e| if e.kind() == std::io::ErrorKind::NotFound { Ok(()) } else { Err(e) })
                        .map_err(|e| format!("{}: {e}", p.display())),
                };
                if let Err(e) = &r {
                    eprintln!("recording not saved: {e}");
                }
                if tell {
                    let _ = told.send((p, r));
                }
            }
        });
        Writer { tx: Some(tx), thread: Some(thread), done }
    }
    /// `f` to `path`; with `tell`, how it went comes back (`written`).
    pub fn send(&self, path: PathBuf, f: ReplayFile, tell: bool) {
        if let Some(tx) = &self.tx {
            let _ = tx.send((path, Some(f), tell));
        }
    }
    /// `path` removed, in its turn.
    pub fn remove(&self, path: PathBuf) {
        if let Some(tx) = &self.tx {
            let _ = tx.send((path, None, false));
        }
    }
    /// The writes asked about that have finished since the last call.
    pub fn written(&self) -> Vec<(PathBuf, Result<(), String>)> {
        self.done.try_iter().collect()
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

/// A session a bot played, as the app records one: its log, the frames it
/// ran, and (for the bench) the time the play and the log took.
pub struct BotRun {
    pub log: game::replay::Recorder,
    pub frames: usize,
    pub play_s: f64,
    pub log_s: f64,
}

/// A fresh process's session played by a bot (`game::debug::bot`): booted
/// at the title in a 1280×800 window from nothing stored, New descent
/// chosen and the pointer taken, the way in played, then the bot's frames
/// (`next` gives the input of its `k`th from where play stands, None when
/// it is done) at an even 60 a second, its log kept (from the start if
/// `whole`, as `--record` keeps one).
pub fn bot_session(
    flags: &str,
    native: u64,
    whole: bool,
    make: fn(&str) -> Game,
    mut next: impl FnMut(&Game, usize) -> Option<game::replay::FrameInput>,
) -> Result<BotRun, String> {
    use game::replay::{FrameInput, InputEvent, Recorder};
    use std::time::Instant;
    game::base::rng::seed_native(native);
    let mut g = make(flags);
    let storage = g.storage.clone();
    g.boot_in_view(Some([1280.0, 800.0, 1.0]));
    let mut log = Recorder::session(native, flags, storage, &g);
    if whole {
        log.start_recording();
    }
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
                let Some(n) = next(&g, k - b) else { break };
                i = n;
            }
        }
        if played.is_none() && k > 5000 {
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
    Ok(BotRun { log, frames: k, play_s, log_s })
}

/// `--record-bot DIR --seed S --levels 1,2,… [--bot-seconds N]`: for each
/// floor of run `seed` a session recorded whole, the floor in its launch
/// flags (a fresh process at the title, New descent, the way in to that
/// floor), the wanderer (`game::debug::bot::Wanderer`) playing `seconds`
/// after the way in (it does not look for doors: a floor a file), written
/// to `DIR/bot_<seed>_L<floor>.ocreplay`.
pub fn record_bot(dir: &str, seed: u32, levels: &[i32], seconds: f64, make: fn(&str) -> Game) -> Result<(), String> {
    for &l in levels {
        let flags = format!("seed={seed}&level={l}");
        let native = 0x0B07_0000 ^ u64::from(seed) ^ ((l as u64) << 40);
        let frames = (seconds * 60.0) as usize;
        let mut bot = game::debug::bot::Wanderer::new(u64::from(seed) ^ ((l as u64) << 32));
        let run = bot_session(&flags, native, true, make, |g, k| (k < frames).then(|| bot.next(g)))?;
        let path = Path::new(dir).join(format!("bot_{seed}_L{l}.ocreplay"));
        let f = run.log.snapshot(header(&flags, l));
        write(&path, &f)?;
        println!(
            "{}: floor {l} of run {seed}, {} frames ({:.0} s), {} KB",
            path.display(),
            run.frames,
            run.frames as f64 / 60.0,
            f.encode().len() / 1024
        );
    }
    Ok(())
}

/// `--bench-replay OUT.json` (`cargo xtask bench`): what recording and
/// replaying cost. A session as the app records one (a fresh process at the
/// title, New descent, the way in, then `frames` frames of the restless bot
/// at an even 60 a second), the log's work timed after every frame and its
/// memory weighed at the end; then the session replayed headless from its
/// start, timed, and checked.
pub fn bench(out: &str, seed: u32, frames: usize, make: fn(&str) -> Game) -> Result<(), String> {
    use std::time::Instant;
    let flags = format!("seed={seed}&nosound");
    // (the restless bot: a fixed script, the same work on every run)
    let bot = game::debug::bot::restless(frames, seed);
    let BotRun { log, frames: k, play_s, log_s, .. } = bot_session(&flags, 0x0BE4_C4ED, false, make, |_, k| bot.get(k).cloned())?;
    let minutes = k as f64 / 3600.0;
    let log_kb_min = log.heap_bytes() as f64 / 1024.0 / minutes;
    let file = log.snapshot(header(&flags, 1));
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
