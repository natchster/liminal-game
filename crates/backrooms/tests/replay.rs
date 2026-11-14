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

/// A game as the app makes one for a recorded session (`main.rs`,
/// `replay_game`): a fresh install, booted in the window's view, at the title.
fn app_game(flags: &str, view: Option<[f64; 3]>) -> Game {
    let mut g = Game::new_bare();
    g.launch_flags = flags.to_string();
    g.storage_path = None;
    placers::register_all(&mut g.placers);
    g.boot_in_view(view);
    g
}

/// A session recorded the way the app records one (`--record`: from the
/// title, the window's uneven intervals in clock ticks, the look rounded
/// where it enters, a floor entered on the way) survives its file
/// (`OCREPLAY`) and replays to the same states, from its start and from the
/// floor's anchor, here and in the game itself (`--replay-verify`); and a
/// minute of it adds little to its file.
#[test]
fn a_recording_survives_its_file_and_replays() {
    let (seed, level) = (77u32, 3);
    let flags = format!("seed={seed}&level={level}&nosound");
    let bot = bot_inputs(2400, seed);
    let native = 0xC0FFEE;
    // the window's frame times are not even
    let dt_of = |k: usize| 1.0 / 60.0 * (1.0 + 0.3 * ((k * 7919 % 13) as f64 / 13.0 - 0.5));
    let (bytes, recorded, door_call, door_at, mid) = {
        let flags = flags.clone();
        std::thread::Builder::new()
            .stack_size(256 << 20)
            .spawn(move || {
                game::base::rng::seed_native(native);
                let mut g = app_game(&flags, Some([1280.0, 800.0, 2.0]));
                let mut r = Recorder::session(native, &flags, &g);
                let (mut hashes, mut door_call, mut door_at, mut mid) = (vec![], 0, None, (0, 0.0));
                let (mut k, mut played, mut minutes) = (0usize, None::<usize>, 0.0);
                loop {
                    // from the title, as a player begins: the new descent
                    // chosen and the pointer taken; the way in plays, and
                    // then the bot plays
                    let mut i = FrameInput::default();
                    match k {
                        0 => i.events.push(game::replay::InputEvent::Click("menu:new".into())),
                        1 => i.events.push(game::replay::InputEvent::Lock(true)),
                        _ if played.is_none() && (g.state != "playing" || g.hands_taken()) => {}
                        _ => {
                            let b = *played.get_or_insert(k);
                            let Some(next) = bot.get(k - b) else { break };
                            i = next.clone();
                            // a while in, the floor's door, as walking into it does
                            if k - b == 900 {
                                g.next_level();
                                door_call = k;
                            }
                        }
                    }
                    assert!(k < 5000, "the way in never gave the bot its hands");
                    for e in &i.events {
                        r.push(e.clone());
                    }
                    g.apply_input(&i);
                    let ticks = game::base::game::clock_ticks(dt_of(k));
                    let dt = g.advance_clock(ticks);
                    g.frame(dt);
                    r.end_frame(&mut g, ticks);
                    minutes += dt_of(k) / 60.0;
                    if door_at.is_none() && r.segments.len() > 1 {
                        door_at = Some(r.segments[1].first_frame as usize);
                    }
                    hashes.push(g.state_hash());
                    // past the door (whose anchor is paid once): how big so far
                    if played.is_some_and(|b| k - b == 1500) {
                        mid = (r.snapshot(json!({ "flags": flags })).encode().len(), minutes);
                    }
                    k += 1;
                }
                let bytes = r.snapshot(json!({ "flags": flags })).encode();
                println!("the way in took {} frames; {} frames in all, {minutes:.2} min", played.unwrap_or(0), hashes.len());
                (bytes, hashes, door_call, door_at.expect("the door was taken"), (mid.0, minutes - mid.1))
            })
            .unwrap()
            .join()
            .unwrap()
    };
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("../../target/replays/recorded_{seed}_L{level}.ocreplay"));
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    // a sample for `backrooms --replay`, `--replay-verify` and `cargo xtask heatmap`
    std::fs::write(&path, &bytes).unwrap();
    // what a minute of play adds (a start and a floor's anchor are paid once:
    // here 6 KB, most of it the dust's motes)
    let per_minute = (bytes.len() - mid.0) as f64 / mid.1 / 1024.0;
    println!("{} bytes; past the door it grows by {per_minute:.1} KB a minute", bytes.len());
    assert!(per_minute <= 25.0, "a replay grows too fast: {per_minute:.1} KB a minute");
    let file = ReplayFile::decode(&bytes).expect("the recording reads back");
    assert_eq!(file.segments.len(), 2, "its start and the floor the door led to");
    assert!(file.telemetry["rows"].as_array().is_some_and(|r| r.len() >= 25), "a telemetry row a second");
    for seg in 0..2 {
        let f = file.clone();
        let (replayed, diverged) = std::thread::Builder::new()
            .stack_size(256 << 20)
            .spawn(move || {
                let (mut g, mut p) = Player::start(f, seg, app_game).expect("the recording starts a game");
                let mut hashes = vec![];
                loop {
                    // the door, taken as the recording took it: from the
                    // start, play crosses into the floor's segment by itself
                    if p.frame == door_call as u64 {
                        g.next_level();
                    }
                    if !p.step(&mut g) {
                        break;
                    }
                    hashes.push(g.state_hash());
                }
                (hashes, p.diverged)
            })
            .unwrap()
            .join()
            .unwrap();
        // from the floor's anchor, the first frame run is the rest of the
        // frame the floor was entered in: play's state after that frame
        let from = if seg == 0 { 0 } else { door_at - 1 };
        assert_eq!(diverged, None, "segment {seg}: a kept state differs");
        if let Some(k) = (0..replayed.len()).find(|&k| replayed[k] != recorded[from + k]) {
            panic!(
                "segment {seg}: the recording replays differently from frame {} in {:?}",
                from + k,
                recorded[from + k].diff(&replayed[k])
            );
        }
        assert_eq!(from + replayed.len(), recorded.len(), "segment {seg} plays to the recording's end");
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
        .and_then(|l| l.split(' ').next()?.parse::<usize>().ok());
    let first_kept_after_door = (door_call / 60 + 1) * 60 - 1;
    assert_eq!(left, Some(first_kept_after_door), "from the start, the game repeats the recording up to the door: {said}");
}
