//! Replays: a recorded run, fed back frame by frame into a fresh game, must
//! reproduce the game's state exactly. A mismatch names the first frame that
//! differs and in which part (player, creatures, director, world), the frame
//! where a nondeterminism entered — instead of a fixture failing three
//! changes later. Recordings are written to `target/replays/` to inspect.
mod common;

use common::bot_inputs;
use game::Game;
use game::replay::{FrameInput, Player, Recorder, ReplayFile, StateHash};
use serde_json::json;

/// (run seed, level, seconds)
const SCENARIOS: &[(u32, i32, f64)] = &[(4242, 2, 40.0), (7, 5, 40.0)];

fn new_game(seed: u32, level: i32) -> Game {
    game::base::rng::seed_native(0x5EED_0000 ^ u64::from(seed) ^ ((level as u64) << 40));
    let mut g = Game::new_bare();
    placers::register_all(&mut g.placers);
    g.launch_flags = format!("seed={seed}&level={level}&nosound");
    g.boot();
    g.start_new_game();
    g.state = "playing".into();
    g.hide_overlay();
    g.pointer_locked = true;
    g
}

/// One run in a thread of its own: the game's caches and the per-frame random
/// stream are thread-local, so every run starts from the same state, as a
/// fresh process would (a second game on one thread would find the first
/// one's textures cached, draw less randomness while building, and start with
/// a different director timer).
fn play(seed: u32, level: i32, inputs: &[FrameInput]) -> Vec<StateHash> {
    let inputs = inputs.to_vec();
    std::thread::Builder::new().stack_size(256 << 20).spawn(move || play_here(seed, level, &inputs)).unwrap().join().unwrap()
}

fn play_here(seed: u32, level: i32, inputs: &[FrameInput]) -> Vec<StateHash> {
    let mut g = new_game(seed, level);
    let mut hashes = Vec::with_capacity(inputs.len());
    for i in inputs {
        g.apply_input(i);
        g.now_ms += 1000.0 / 60.0;
        g.fixed_step_dt = 1.0 / 60.0;
        g.frame(1.0 / 60.0);
        g.fixed_step_dt = 0.0;
        hashes.push(g.state_hash());
    }
    hashes
}

#[test]
fn a_replay_reproduces_the_run() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/replays");
    std::fs::create_dir_all(&dir).unwrap();
    for &(seed, level, secs) in SCENARIOS {
        let inputs = bot_inputs((secs * 60.0) as usize, seed);
        let recorded = play(seed, level, &inputs);
        let replayed = play(seed, level, &inputs);
        let rec = json!({ "seed": seed, "level": level, "inputs": inputs.iter().map(|i| i.to_json()).collect::<Vec<_>>() });
        std::fs::write(dir.join(format!("bot_{seed}_L{level}.json")), rec.to_string()).unwrap();
        // the recording round-trips through its file format
        let back: Vec<FrameInput> = rec["inputs"].as_array().unwrap().iter().map(FrameInput::from_json).collect();
        assert_eq!(back, inputs, "the replay file format loses input");
        if let Some(k) = (0..recorded.len()).find(|&k| recorded[k] != replayed[k]) {
            panic!(
                "seed {seed} L{level}: the replay diverges at frame {k} ({:.2} s) in {:?}; something per-frame is not deterministic (clock, unseeded randomness, HashMap order)",
                k as f64 / 60.0,
                recorded[k].diff(&replayed[k])
            );
        }
        // the run went somewhere: a replay of a player standing still proves nothing
        let moved: std::collections::HashSet<u64> = recorded.iter().map(|h| h.player).collect();
        assert!(moved.len() > recorded.len() / 4, "seed {seed} L{level}: the bot barely moved ({} distinct states)", moved.len());
        println!("seed {seed} L{level}: {} frames replayed identically ({} distinct player states)", recorded.len(), moved.len());
    }
}

/// A fresh process's game as the app makes one for a replay (`main.rs`,
/// `replay_game`): nothing read, nothing written, not yet booted.
fn app_game(flags: &str) -> Game {
    let mut g = Game::new_bare();
    g.launch_flags = flags.to_string();
    g.storage_path = None;
    placers::register_all(&mut g.placers);
    g
}

/// What happens at a frame of the bot's (counted from its first).
#[derive(Clone, Copy, PartialEq)]
enum Do {
    /// the floor's door, taken as walking into it takes it
    Door,
    /// F9: a recording begins, or the one being made ends
    Record,
    /// the recording so far, weighed (how fast it grows)
    Weigh,
}

/// A session as the app plays one.
struct Played {
    log: Recorder,
    /// the state after every frame
    states: Vec<StateHash>,
    /// the frames (whole-session) the doors were taken before
    doors: Vec<u64>,
    /// the recordings ended by Record
    made: Vec<ReplayFile>,
    /// (bytes, minutes) at each `Weigh`, and at the end
    weighed: Vec<(usize, f64)>,
    /// the frame the bot was given its hands at
    began: usize,
}

/// The window's frame times are not even.
fn dt_of(k: usize) -> f64 {
    1.0 / 60.0 * (1.0 + 0.3 * ((k * 7919 % 13) as f64 / 13.0 - 0.5))
}

/// A session played as the app plays one: a fresh process at the title,
/// booted from `storage` (as the app reads it) in a window's view, its log
/// kept (recording from the first frame if `whole`, as `--record` does);
/// New descent chosen, the pointer taken, the way in played; then the bot,
/// and at its frames `script` says, a door or the Record key.
fn play_session(flags: &str, storage: &[(&str, &str)], whole: bool, bot: &[FrameInput], script: &[(usize, Do)]) -> Played {
    let (flags, bot, script) = (flags.to_string(), bot.to_vec(), script.to_vec());
    let storage: std::collections::BTreeMap<String, String> = storage.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect();
    std::thread::Builder::new()
        .stack_size(256 << 20)
        .spawn(move || {
            let native = 0xC0FFEE;
            game::base::rng::seed_native(native);
            let mut g = app_game(&flags);
            g.storage.clone_from(&storage);
            g.boot_in_view(Some([1280.0, 800.0, 2.0]));
            let mut log = Recorder::session(native, &flags, storage, &g);
            if whole {
                log.start_recording();
            }
            let header = json!({ "flags": flags });
            let mut p = Played { log, states: vec![], doors: vec![], made: vec![], weighed: vec![], began: 0 };
            let (mut k, mut played, mut minutes) = (0usize, None::<usize>, 0.0);
            loop {
                let mut i = FrameInput::default();
                let mut now = vec![];
                match k {
                    0 => i.events.push(game::replay::InputEvent::Click("menu:new".into())),
                    1 => i.events.push(game::replay::InputEvent::Lock(true)),
                    _ if played.is_none() && (g.state != "playing" || g.hands_taken()) => {}
                    _ => {
                        let b = *played.get_or_insert(k);
                        let Some(next) = bot.get(k - b) else { break };
                        i = next.clone();
                        now = script.iter().filter(|s| s.0 == k - b).map(|s| s.1).collect();
                    }
                }
                assert!(k < bot.len() + 5000, "the way in never gave the bot its hands");
                if now.contains(&Do::Door) {
                    g.next_level();
                    p.doors.push(k as u64);
                }
                for e in &i.events {
                    p.log.push(e.clone());
                }
                g.apply_input(&i);
                let ticks = game::base::game::clock_ticks(dt_of(k));
                let dt = g.advance_clock(ticks);
                g.frame(dt);
                p.log.end_frame(&mut g, ticks);
                minutes += dt_of(k) / 60.0;
                p.states.push(g.state_hash());
                // (between frames, as the app answers the key)
                if now.contains(&Do::Record) {
                    match p.log.take_recording(header.clone()) {
                        Some(f) => p.made.push(f),
                        None => p.log.start_recording(),
                    }
                }
                if now.contains(&Do::Weigh) {
                    p.weighed.push((p.log.snapshot(header.clone()).encode().len(), minutes));
                }
                k += 1;
            }
            p.weighed.push((p.log.snapshot(header).encode().len(), minutes));
            p.began = played.unwrap_or(0);
            p
        })
        .unwrap()
        .join()
        .unwrap()
}

/// `file` replayed from its segment `seg` (to its in-point by simulating,
/// then to its end), the doors taken where play took them: every kept
/// state as recorded, and the state after every frame as play's. Returns
/// how many frames ran from the in-point.
fn replays_as_played(file: &ReplayFile, seg: usize, p: &Played) -> u64 {
    let (f, doors) = (file.clone(), p.doors.clone());
    let (states, diverged, from) = std::thread::Builder::new()
        .stack_size(256 << 20)
        .spawn(move || {
            let (mut g, mut pl) = Player::start(f, seg, app_game).expect("the recording starts a game");
            let mut states = vec![];
            loop {
                if doors.contains(&pl.frame) {
                    g.next_level();
                }
                if !pl.step(&mut g) {
                    break;
                }
                // (the frame just run; a floor's entry is the rest of the frame before its first)
                states.push((pl.frame - 1, g.state_hash()));
            }
            let from = pl.in_point();
            (states, pl.diverged, from)
        })
        .unwrap()
        .join()
        .unwrap();
    assert_eq!(diverged, None, "segment {seg}: a kept state differs");
    for (k, h) in &states {
        assert!(*h == p.states[*k as usize], "segment {seg}: frame {k} replays differently, in {:?}", p.states[*k as usize].diff(h));
    }
    let last = states.last().map_or(0, |s| s.0 + 1);
    assert_eq!(last, file.out_point(), "segment {seg} plays to the recording's out-point");
    last - from.max(states.first().map_or(0, |s| s.0))
}

/// A session recorded whole, the way `--record` records one (from the
/// title, the window's uneven intervals in clock ticks, the look rounded
/// where it enters, a floor entered on the way), survives its file
/// (`OCREPLAY`) and replays to the same states, from its start and from the
/// floor's anchor, here and in the game itself (`--replay-verify`); and a
/// minute of it adds little to its file.
#[test]
fn a_recording_survives_its_file_and_replays() {
    let (seed, level) = (77u32, 3);
    let flags = format!("seed={seed}&level={level}&nosound");
    let p = play_session(&flags, &[], true, &bot_inputs(2400, seed), &[(900, Do::Door), (1500, Do::Weigh)]);
    let file = p.log.snapshot(json!({ "flags": flags }));
    let bytes = file.encode();
    println!("the way in took {} frames; {} frames in all, {:.2} min", p.began, p.states.len(), p.weighed[1].1);
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../../target/replays/recorded_{seed}_L{level}.ocreplay"));
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    // a sample for `backrooms --replay`, `--replay-verify` and `cargo xtask heatmap`
    std::fs::write(&path, &bytes).unwrap();
    // what a minute of play adds (a start and a floor's anchor are paid once:
    // here 6 KB, most of it the dust's motes)
    let ((mid, at), (end, until)) = (p.weighed[0], p.weighed[1]);
    let per_minute = (end - mid) as f64 / (until - at) / 1024.0;
    println!("{} bytes; past the door it grows by {per_minute:.1} KB a minute", bytes.len());
    assert!(per_minute <= 25.0, "a replay grows too fast: {per_minute:.1} KB a minute");
    let file = ReplayFile::decode(&bytes).expect("the recording reads back");
    assert_eq!(file.segments.len(), 2, "its start and the floor the door led to");
    assert_eq!((file.in_point(), file.out_point()), (0, p.states.len() as u64), "the whole session");
    assert!(file.telemetry["rows"].as_array().is_some_and(|r| r.len() >= 25), "a telemetry row a second");
    for seg in 0..2 {
        replays_as_played(&file, seg, &p);
    }
    // and in the game itself: the floor's segment from its anchor to the end,
    // and from the start up to the door (which this test took by hand, so
    // the recording does not have it: the first state kept after it differs)
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_backrooms")).arg("--replay-verify").arg(&path).output().unwrap();
    let said = String::from_utf8_lossy(&out.stdout);
    println!("{said}");
    assert!(said.contains("segment 1, from L4's anchor (door): ") && said.contains("to the end, every kept state as recorded"), "{said}");
    let left = said
        .lines()
        .find_map(|l| l.strip_prefix("segment 0, from the start: leaves the recording at frame "))
        .and_then(|l| l.split(' ').next()?.parse::<u64>().ok());
    let first_kept_after_door = (p.doors[0] / 60 + 1) * 60 - 1;
    assert_eq!(left, Some(first_kept_after_door), "from the start, the game repeats the recording up to the door: {said}");
}

/// A recording begun at any moment (F9) replays from that moment: its
/// file holds the segment its in-point is in (the session's start, with what
/// storage held, or a floor's anchor) and every one after, and its replay
/// reaches the in-point by simulating, then plays to its out-point as play
/// went. Between recordings the log keeps only the current floor's segment.
#[test]
fn a_recording_begun_mid_floor_replays_from_its_in_point() {
    let (seed, level) = (4242u32, 2);
    let flags = format!("seed={seed}&level={level}&nosound");
    // a player's settings, as the app reads them before it boots
    let mut settings = game::session::settings::Settings::default();
    settings.fov = 90.0;
    settings.look = 1.4;
    let settings = settings.to_json().to_string();
    let script = [(300, Do::Record), (900, Do::Door), (1500, Do::Record), (1700, Do::Door), (1900, Do::Record), (2300, Do::Record)];
    let p = play_session(&flags, &[(game::session::settings::SETTINGS_KEY, &settings)], false, &bot_inputs(2500, seed), &script);
    let [a, b] = p.made.as_slice() else { panic!("two recordings made, not {}", p.made.len()) };
    // the first: begun on the first floor (from the session's start, what
    // storage held kept), across a door, ended on the next
    assert_eq!(a.segments.len(), 2, "the session's start and the floor after the door");
    let game::replay::Start::Session { storage, .. } = &a.segments[0].start else { panic!("the first floor starts at the session") };
    assert_eq!(storage.get(game::session::settings::SETTINGS_KEY), Some(&settings), "what storage held at the boot");
    assert_eq!(a.in_point(), (p.began + 300 + 1) as u64);
    assert_eq!(a.out_point(), (p.began + 1500 + 1) as u64);
    let shown = replays_as_played(&ReplayFile::decode(&a.encode()).unwrap(), 0, &p);
    assert_eq!(shown, 1200, "from its in-point to its out-point");
    replays_as_played(a, 1, &p);
    // the second: begun mid-floor, two doors later; the log had let the
    // floors before go
    assert_eq!(b.segments.len(), 1, "only the floor it was made on");
    assert!(matches!(b.segments[0].start, game::replay::Start::Floor(_)));
    assert!(b.in_point() > b.segments[0].first_frame, "begun mid-floor: {} after its start", b.in_point() - b.segments[0].first_frame);
    let shown = replays_as_played(&ReplayFile::decode(&b.encode()).unwrap(), 0, &p);
    assert_eq!(shown, 400);
    assert_eq!(p.log.segments.len(), 1, "between recordings, the log keeps the current floor alone");
    let minutes = (p.states.len() - p.log.segments[0].first_frame as usize) as f64 / 3600.0;
    println!("the log: {} KB for the floor's {minutes:.2} min", p.log.heap_bytes() / 1024);
}
