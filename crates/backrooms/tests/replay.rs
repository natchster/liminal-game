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
    for &(seed, level, secs) in SCENARIOS {
        let inputs = bot_inputs((secs * 60.0) as usize, seed);
        let recorded = play(seed, level, &inputs);
        let replayed = play(seed, level, &inputs);
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
    /// a death (as a creature's touch makes one)
    Die,
    /// R: a key that goes on from the death card (waking)
    Wake,
}

/// A session as the app plays one.
struct Played {
    log: Recorder,
    /// the state after every frame
    states: Vec<StateHash>,
    /// the frames (whole-session) the doors were taken before, and the deaths
    doors: Vec<u64>,
    deaths: Vec<u64>,
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
            let mut p = Played { log, states: vec![], doors: vec![], deaths: vec![], made: vec![], weighed: vec![], began: 0 };
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
                if now.contains(&Do::Die) {
                    g.on_death("entity");
                    p.deaths.push(k as u64);
                }
                if now.contains(&Do::Wake) {
                    i.events.push(game::replay::InputEvent::Down("KeyR".into()));
                    i.events.push(game::replay::InputEvent::Up("KeyR".into()));
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
    let (f, doors, deaths) = (file.clone(), p.doors.clone(), p.deaths.clone());
    let (states, diverged, from) = std::thread::Builder::new()
        .stack_size(256 << 20)
        .spawn(move || {
            let (mut g, mut pl) = Player::start(f, seg, app_game).expect("the recording starts a game");
            let mut states = vec![];
            loop {
                if doors.contains(&pl.frame) {
                    g.next_level();
                }
                if deaths.contains(&pl.frame) {
                    g.on_death("entity");
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

/// A short session from the title (New descent, the way in, the bot),
/// played on `rate`'s speakers (the silent clock's 48 kHz when none) and
/// replayed in the same thread, every frame's state compared.
fn plays_and_replays_here(rate: Option<f64>) {
    std::thread::Builder::new()
        .stack_size(256 << 20)
        .spawn(move || {
            let flags = "seed=4242";
            let native = 0xFEED;
            game::base::rng::seed_native(native);
            let mut g = app_game(flags);
            g.boot_on(Some([1280.0, 800.0, 1.0]), rate.map(engine::audio::AudioContext::new_offline));
            let mut log = Recorder::session(native, flags, Default::default(), &g);
            let ticks = game::base::game::clock_ticks(1.0 / 60.0);
            let bot = bot_inputs(600, 4242);
            let mut states = vec![];
            for k in 0..900 {
                let mut i = FrameInput::default();
                match k {
                    0 => i.events.push(game::replay::InputEvent::Click("menu:new".into())),
                    1 => i.events.push(game::replay::InputEvent::Lock(true)),
                    _ if k >= 300 => i = bot[k - 300].clone(),
                    _ => {}
                }
                for e in &i.events {
                    log.push(e.clone());
                }
                g.apply_input(&i);
                let dt = g.advance_clock(ticks);
                g.frame(dt);
                log.end_frame(&mut g, ticks);
                states.push(g.state_hash());
            }
            let (mut r, mut p) = Player::start(log.snapshot(json!({})), 0, app_game).unwrap();
            let mut k = 0;
            while p.step(&mut r) {
                assert!(r.state_hash() == states[k], "frame {k} replays differently, in {:?}", states[k].diff(&r.state_hash()));
                k += 1;
            }
            assert_eq!(k, states.len());
        })
        .unwrap()
        .join()
        .unwrap();
}

/// A replay in the thread that played (as the Replays screen plays one in
/// the running game): nothing the thread kept from play (the per-frame
/// stream's draw count, its caches) reaches the replay.
#[test]
fn a_replay_in_the_thread_that_played_repeats_it() {
    plays_and_replays_here(None);
}

/// A session played on speakers at another rate (44.1 kHz, as a headset or
/// a USB DAC may be) replays on the silent clock every replay and test runs
/// on (48 kHz): what a session draws as it starts does not hang on the
/// device.
#[test]
fn a_session_replays_whatever_the_speakers_rate() {
    plays_and_replays_here(Some(44_100.0));
}

/// Watching a recording begun mid-floor (`Viewer`): it seeks to its
/// in-point, plays by the window's time at its speed, and ten seconds back
/// (a fresh game from the floor's anchor) or on lands in play's state there.
#[test]
fn a_replay_is_watched_by_the_clock_and_moved_either_way() {
    use game::replay::viewer::Viewer;
    use std::time::Duration;
    let flags = "seed=77&level=3&nosound";
    let p = play_session(flags, &[], false, &bot_inputs(2400, 77), &[(900, Do::Door), (1100, Do::Record)]);
    let file = ReplayFile::decode(&p.log.snapshot(json!({ "flags": flags })).encode()).unwrap();
    assert!(matches!(file.segments[0].start, game::replay::Start::Floor(_)), "begun after the door");
    let states = p.states.clone();
    std::thread::Builder::new()
        .stack_size(256 << 20)
        .spawn(move || {
            let (mut g, mut v) = Viewer::open(file, app_game).unwrap();
            let as_played = |g: &Game, v: &Viewer, what: &str| {
                let k = v.player.frame as usize - 1;
                assert!(g.state_hash() == states[k], "{what}: frame {k} is not play's, in {:?}", states[k].diff(&g.state_hash()));
            };
            let settle = |g: &mut Game, v: &mut Viewer| {
                while v.seeking() {
                    v.frame(g, 1.0 / 60.0, Duration::from_millis(10));
                }
            };
            settle(&mut g, &mut v);
            assert_eq!(v.bar().at, 0.0, "at its in-point");
            as_played(&g, &v, "the in-point");
            // ten seconds of the window, ten of the recording
            for _ in 0..600 {
                v.frame(&mut g, 1.0 / 60.0, Duration::from_millis(10));
            }
            assert!((v.bar().at - 10.0).abs() < 0.05, "at {}", v.bar().at);
            as_played(&g, &v, "ten seconds in");
            // twice the speed: five seconds of the window, ten of it
            v.faster();
            for _ in 0..300 {
                v.frame(&mut g, 1.0 / 60.0, Duration::from_millis(10));
            }
            assert!((v.bar().at - 20.0).abs() < 0.05, "at {}", v.bar().at);
            // back ten seconds: a fresh game from the anchor, simulated there
            v.jump(&mut g, -10.0, app_game).unwrap();
            settle(&mut g, &mut v);
            assert!((v.bar().at - 10.0).abs() < 0.05, "back at {}", v.bar().at);
            as_played(&g, &v, "ten seconds back");
            // and on five
            v.jump(&mut g, 5.0, app_game).unwrap();
            settle(&mut g, &mut v);
            assert!((v.bar().at - 15.0).abs() < 0.05, "on at {}", v.bar().at);
            as_played(&g, &v, "five seconds on");
            assert_eq!(v.player.diverged, None);
        })
        .unwrap()
        .join()
        .unwrap();
}

/// The wanderer (`--record-bot`'s bot) walks a floor as a player might,
/// not into a wall: on two floors of a run, a minute of its play covers
/// ground and stands still little; and since it chooses its inputs from
/// what it sees, its recording replays frame for frame like a person's.
#[test]
fn the_wanderer_walks_and_its_walk_replays() {
    for level in [2, 5] {
        std::thread::Builder::new()
            .stack_size(256 << 20)
            .spawn(move || {
                let flags = format!("seed=7&level={level}");
                let native = 0x0B07 ^ level as u64;
                game::base::rng::seed_native(native);
                let mut g = app_game(&flags);
                g.boot_in_view(Some([1280.0, 800.0, 1.0]));
                let mut log = Recorder::session(native, &flags, Default::default(), &g);
                log.start_recording();
                let mut bot = game::debug::bot::Wanderer::new(7 ^ ((level as u64) << 32));
                let ticks = game::base::game::clock_ticks(1.0 / 60.0);
                let (mut states, mut began, mut places) = (vec![], None::<usize>, vec![]);
                for k in 0..6000 {
                    let i = match k {
                        0 => FrameInput { events: vec![game::replay::InputEvent::Click("menu:new".into())] },
                        1 => FrameInput { events: vec![game::replay::InputEvent::Lock(true)] },
                        _ if began.is_none() && (g.state != "playing" || g.hands_taken()) => FrameInput::default(),
                        _ => {
                            let b = *began.get_or_insert(k);
                            if k - b == 3600 {
                                break;
                            }
                            if (k - b).is_multiple_of(60) {
                                places.push((g.player.pos.x, g.player.pos.z));
                            }
                            bot.next(&g)
                        }
                    };
                    for e in &i.events {
                        log.push(e.clone());
                    }
                    g.apply_input(&i);
                    let dt = g.advance_clock(ticks);
                    g.frame(dt);
                    log.end_frame(&mut g, ticks);
                    states.push(g.state_hash());
                }
                let tiles: std::collections::BTreeSet<(i64, i64)> =
                    places.iter().map(|(x, z)| ((x / 4.0).floor() as i64, (z / 4.0).floor() as i64)).collect();
                let still = places.windows(2).filter(|w| engine::dmath::DMath::dhypot(w[1].0 - w[0].0, w[1].1 - w[0].1) < 0.5).count();
                println!("floor {level}: {} tiles of 4 m in a minute, {still} s standing", tiles.len());
                assert!(tiles.len() >= 15, "floor {level}: the wanderer covered {} tiles of 4 m in a minute", tiles.len());
                assert!(still <= 12, "floor {level}: the wanderer stood {still} s of a minute");
                let (mut r, mut p) = Player::start(log.snapshot(json!({})), 0, app_game).unwrap();
                let mut k = 0;
                while p.step(&mut r) {
                    assert!(r.state_hash() == states[k], "floor {level}: frame {k} replays differently");
                    k += 1;
                }
                assert_eq!(k, states.len());
            })
            .unwrap()
            .join()
            .unwrap();
    }
}

/// A save at any frame of a replay (`ReplayFile::cut`) goes on as the run
/// went: opened, it is simulated to its frame, in play's state there, and
/// the recording's frames after it run as play ran them.
#[test]
fn a_save_at_any_frame_of_a_replay_goes_on_as_the_run_went() {
    use game::replay::viewer::Viewer;
    let flags = "seed=77&level=3&nosound";
    let p = play_session(flags, &[], true, &bot_inputs(2400, 77), &[(900, Do::Door)]);
    let file = p.log.snapshot(json!({ "flags": flags }));
    let at = p.doors[0] + 700;
    let save = ReplayFile::decode(&file.cut(at).encode()).unwrap();
    assert_eq!((save.segments.len(), save.in_point()), (1, at), "the floor it is on, up to its frame");
    let (states, rest) = (p.states.clone(), file.segments[1].clone());
    std::thread::Builder::new()
        .stack_size(256 << 20)
        .spawn(move || {
            let (mut g, mut v) = Viewer::open(save, app_game).unwrap();
            while v.seeking() {
                v.frame(&mut g, 1.0 / 60.0, std::time::Duration::from_secs(1));
            }
            assert_eq!(v.player.frame, at);
            assert!(g.state_hash() == states[at as usize - 1], "the save's place is not play's");
            // on, with the recording's own frames
            for k in at..at + 600 {
                let (ticks, input) = rest.frames.get((k - rest.first_frame) as usize).unwrap();
                game::replay::replay_frame(&mut g, *ticks, input);
                assert!(g.state_hash() == states[k as usize], "frame {k} after the save is not play's");
            }
        })
        .unwrap()
        .join()
        .unwrap();
}

/// Continue is a floor's start a replay stands on (`How::Continue`): a
/// later session, its title, Continue from the save a run made mid-floor,
/// lands on that floor with its progress, and its log begins a segment
/// there whose replay repeats play frame for frame.
#[test]
fn continue_is_a_floor_start_a_replay_stands_on() {
    std::thread::Builder::new()
        .stack_size(256 << 20)
        .spawn(|| {
            // a run: into floor 3 of run 77, a page taken, saved there
            let p = play_session("seed=77&level=3&nosound", &[], false, &bot_inputs(600, 77), &[]);
            let (mut g, mut pl) = Player::start(p.log.snapshot(json!({})), 0, app_game).unwrap();
            while pl.step(&mut g) {}
            assert_eq!(g.state, "playing");
            g.notebook.insert("f1".into());
            g.save_game();
            let save = ReplayFile::decode(&g.saved.as_ref().unwrap().encode()).unwrap();
            // a later session: the title offers Continue, and it is chosen
            let native = 0x5A7E;
            game::base::rng::seed_native(native);
            let mut h = app_game("nosound");
            h.saved = Some(save);
            h.boot_in_view(Some([1280.0, 800.0, 2.0]));
            assert_eq!(h.menu.items[0].id, "menu:continue");
            let mut log = Recorder::session(native, "nosound", Default::default(), &h);
            // (the whole session kept, as `--record` keeps it)
            log.start_recording();
            let ticks = game::base::game::clock_ticks(1.0 / 60.0);
            let bot = bot_inputs(900, 5);
            let mut states = vec![];
            for k in 0..900 {
                let i = if k == 0 {
                    FrameInput { events: vec![game::replay::InputEvent::Click("menu:continue".into())] }
                } else {
                    bot[k].clone()
                };
                for e in &i.events {
                    log.push(e.clone());
                }
                h.apply_input(&i);
                let dt = h.advance_clock(ticks);
                h.frame(dt);
                log.end_frame(&mut h, ticks);
                states.push(h.state_hash());
            }
            assert_eq!((h.level_num, h.run_seed), (3, 77), "Continue lands on the saved floor of its run");
            assert!(h.notebook.contains("f1"), "and keeps what was taken");
            let file = log.snapshot(json!({ "flags": "nosound" }));
            assert_eq!(file.segments.len(), 2, "the session's start, and Continue's");
            assert!(matches!(&file.segments[1].start, game::replay::Start::Floor(a) if a.how == game::session::anchor::How::Continue));
            let (mut r, mut pl) = Player::start(file.clone(), 1, app_game).unwrap();
            let from = file.segments[1].first_frame as usize - 1;
            let mut k = from;
            while pl.step(&mut r) {
                assert!(r.state_hash() == states[k], "from Continue's anchor, frame {k} is not play's");
                k += 1;
            }
            assert_eq!(k, states.len());
        })
        .unwrap()
        .join()
        .unwrap();
}

/// Waking after a death is a floor's start too (`How::Wake`): the floor
/// again, and a replay from there repeats play.
#[test]
fn waking_is_a_floor_start_a_replay_stands_on() {
    let p = play_session("seed=4242&level=4&nosound", &[], true, &bot_inputs(1200, 4242), &[(300, Do::Die), (500, Do::Wake)]);
    let file = ReplayFile::decode(&p.log.snapshot(json!({})).encode()).unwrap();
    let woke =
        file.segments.iter().position(|s| matches!(&s.start, game::replay::Start::Floor(a) if a.how == game::session::anchor::How::Wake));
    let seg = woke.expect("waking began a segment");
    replays_as_played(&file, seg, &p);
}

/// Taking control of a replay (Enter) after it entered a floor by itself
/// (here, a Continue): that floor's anchor is the recording's, never a new
/// start in play's log (whose first segment stays the cut's), the keys the
/// recording held are let go as play's first input, and the log taken over
/// replays as it was played.
#[test]
fn taking_control_after_a_floor_plays_on_cleanly() {
    use game::replay::viewer::Viewer;
    std::thread::Builder::new()
        .stack_size(256 << 20)
        .spawn(|| {
            // a session: a save of floor 3, the title, Continue, a walk
            let p = play_session("seed=77&level=3&nosound", &[], false, &bot_inputs(600, 77), &[]);
            let (mut g, mut pl) = Player::start(p.log.snapshot(json!({})), 0, app_game).unwrap();
            while pl.step(&mut g) {}
            g.save_game();
            let native = 0x5A7F;
            game::base::rng::seed_native(native);
            let mut h = app_game("nosound");
            h.saved = g.saved.clone();
            h.boot_in_view(Some([1280.0, 800.0, 2.0]));
            let mut log = Recorder::session(native, "nosound", Default::default(), &h);
            log.start_recording();
            let ticks = game::base::game::clock_ticks(1.0 / 60.0);
            let bot = bot_inputs(900, 5);
            for k in 0..900 {
                let i = if k == 0 {
                    FrameInput { events: vec![game::replay::InputEvent::Click("menu:continue".into())] }
                } else {
                    bot[k].clone()
                };
                for e in &i.events {
                    log.push(e.clone());
                }
                h.apply_input(&i);
                let dt = h.advance_clock(ticks);
                h.frame(dt);
                log.end_frame(&mut h, ticks);
            }
            let file = log.snapshot(json!({ "flags": "nosound" }));
            let continued = file.segments[1].first_frame;
            // watched, and taken over ten seconds after the Continue
            let (mut g, mut v) = Viewer::open(file.clone(), app_game).unwrap();
            v.seek_to(&mut g, continued as f64 / 60.0 + 10.0, app_game).unwrap();
            assert_eq!(g.entered, None, "a replay keeps no floor of its own");
            let at = v.player.frame;
            let mut log = Recorder::resume(&file, at, &g);
            let mut held: Vec<String> = g.keys.iter().cloned().collect();
            held.sort();
            for k in held {
                let e = game::replay::InputEvent::Up(k);
                log.push(e.clone());
                g.apply_event(&e);
            }
            assert!(g.keys.is_empty(), "the recording's keys let go");
            let mut states = vec![];
            for i in bot_inputs(300, 9) {
                for e in &i.events {
                    log.push(e.clone());
                }
                g.apply_input(&i);
                let dt = g.advance_clock(ticks);
                g.frame(dt);
                log.end_frame(&mut g, ticks);
                states.push(g.state_hash());
            }
            assert_eq!(log.segments.len(), 1);
            assert_eq!(log.segments[0].first_frame, continued, "play's log starts at the Continue the recording made");
            let (mut r, mut pl) = Player::start(log.snapshot(json!({ "flags": "nosound" })), 0, app_game).unwrap();
            while pl.frame < at || pl.pending_entry() {
                pl.step(&mut r);
            }
            for (k, want) in states.iter().enumerate() {
                assert!(pl.step(&mut r), "the log ends early");
                assert!(r.state_hash() == *want, "frame {k} after taking control is not play's");
            }
        })
        .unwrap()
        .join()
        .unwrap();
}

/// A seek to S seconds lands S seconds after the in-point, whatever the
/// recording begins with (`--save-from-replay` on a recording begun mid-floor).
#[test]
fn a_seek_counts_from_the_in_point() {
    use game::replay::viewer::Viewer;
    let flags = "seed=77&level=3&nosound";
    let p = play_session(flags, &[], false, &bot_inputs(2400, 77), &[(900, Do::Door), (1100, Do::Record)]);
    let file = p.log.snapshot(json!({ "flags": flags }));
    assert!(file.in_point() > file.segments[0].first_frame);
    std::thread::Builder::new()
        .stack_size(256 << 20)
        .spawn(move || {
            let (mut g, mut v) = Viewer::open(file, app_game).unwrap();
            v.seek_to(&mut g, 5.0, app_game).unwrap();
            assert!((v.bar().at - 5.0).abs() < 0.05, "at {}", v.bar().at);
        })
        .unwrap()
        .join()
        .unwrap();
}

/// The recordings the title lists come from outside the game, so they reach
/// play as an input (`Listed`): a session that went down the title past the
/// Replays line replays to the same screen.
#[test]
fn the_title_s_rows_replay_as_played() {
    std::thread::Builder::new()
        .stack_size(256 << 20)
        .spawn(|| {
            let native = 0x7171;
            game::base::rng::seed_native(native);
            let mut g = app_game("nosound");
            g.boot_in_view(Some([1280.0, 800.0, 1.0]));
            let mut log = Recorder::session(native, "nosound", Default::default(), &g);
            log.start_recording();
            let line = |n: usize| game::replay::viewer::ReplayLine {
                path: format!("r{n}.ocreplay").into(),
                label: format!("Floor {n}"),
                detail: String::new(),
            };
            let listed = game::replay::InputEvent::Listed(vec![line(1), line(2)]);
            let keys = ["ArrowDown", "ArrowDown", "Enter"];
            let ticks = game::base::game::clock_ticks(1.0 / 60.0);
            for k in 0..20 {
                let mut i = FrameInput::default();
                if k == 0 {
                    i.events.push(listed.clone());
                }
                if let Some(key) = keys.get(k / 4).filter(|_| k % 4 == 2) {
                    i.down(key);
                    i.up(key);
                }
                for e in &i.events {
                    log.push(e.clone());
                }
                g.apply_input(&i);
                let dt = g.advance_clock(ticks);
                g.frame(dt);
                log.end_frame(&mut g, ticks);
            }
            // New descent, Replays, Settings, Quit: two down is Settings
            assert_eq!(g.live_menu(), Some(game::ui::menu::Screen::Settings));
            let file = ReplayFile::decode(&log.snapshot(json!({})).encode()).unwrap();
            let (mut r, mut pl) = Player::start(file, 0, app_game).unwrap();
            while pl.step(&mut r) {}
            assert_eq!(r.replays.len(), 2, "the list came with the recording");
            assert_eq!(r.live_menu(), g.live_menu(), "the replay went down the same rows");
            assert!(r.state_hash() == g.state_hash());
        })
        .unwrap()
        .join()
        .unwrap();
}

/// A save that cannot be put back is kept, and the title stays, the
/// player's settings untouched; one an older build made, without a field
/// added since or with one taken out, continues.
#[test]
fn a_save_that_cannot_be_read_is_kept() {
    std::thread::Builder::new()
        .stack_size(256 << 20)
        .spawn(|| {
            let p = play_session("seed=4242&level=2&nosound", &[], false, &bot_inputs(300, 1), &[]);
            let (mut g, mut pl) = Player::start(p.log.snapshot(json!({})), 0, app_game).unwrap();
            while pl.step(&mut g) {}
            g.save_game();
            let good = g.saved.clone().unwrap();
            let with = |drop: &str| {
                let mut f = good.clone();
                if let game::replay::Start::Floor(a) = &mut f.segments[0].start {
                    let (part, key) = drop.split_once('.').unwrap_or((drop, ""));
                    let s = a.state.as_object_mut().unwrap();
                    if key.is_empty() {
                        s.remove(part);
                    } else {
                        s[part].as_object_mut().unwrap().remove(key);
                    }
                }
                f
            };
            // the stream gone: kept, the title stays, the settings untouched
            let mut h = app_game("nosound");
            h.boot_in_view(Some([1280.0, 800.0, 1.0]));
            h.settings.fov = 101.0;
            h.saved = Some(with("native"));
            h.continue_game();
            h.run_scheduled();
            assert_eq!(h.live_menu(), Some(game::ui::menu::Screen::Title), "the title stays");
            assert!(h.saved.is_some() && !h.save_cleared, "the save is kept");
            assert_eq!(h.settings.fov, 101.0, "the player's settings untouched");
            // a field added since (the overlay's eased opacity): it continues
            let mut h = app_game("nosound");
            h.boot_in_view(Some([1280.0, 800.0, 1.0]));
            h.saved = Some(with("f64.aberration_opacity_cur"));
            h.continue_game();
            h.run_scheduled();
            assert_eq!((h.state.as_str(), h.level_num), ("playing", 2), "an older save continues");
            // a field taken out since (the film grain's clock, issue #14): it continues
            let mut older = good.clone();
            if let game::replay::Start::Floor(a) = &mut older.segments[0].start {
                a.state["f64"]["grain_t"] = json!(0.25f64.to_bits());
            }
            let mut h = app_game("nosound");
            h.boot_in_view(Some([1280.0, 800.0, 1.0]));
            h.saved = Some(older);
            h.continue_game();
            h.run_scheduled();
            assert_eq!((h.state.as_str(), h.level_num), ("playing", 2), "a save with the grain's clock continues");
        })
        .unwrap()
        .join()
        .unwrap();
}
