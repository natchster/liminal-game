//! Replays: a recorded run, fed back frame by frame into a fresh game, must
//! reproduce the game's state exactly. A mismatch names the first frame that
//! differs and in which part (player, creatures, director, world), the frame
//! where a nondeterminism entered — instead of a fixture failing three
//! changes later. Recordings are written to `target/replays/` to inspect.
use game::Game;
use game::debug::replay::{FrameInput, Recorder, Recording, StateHash, replay_frame};
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

/// A restless player: mostly walking, turning, now and then running,
/// crouching or switching the flashlight (a fixed pseudo-random script).
fn bot_inputs(frames: usize, seed: u32) -> Vec<FrameInput> {
    let mut s = u64::from(seed) | 1;
    let mut next = move || {
        s ^= s << 13;
        s ^= s >> 7;
        s ^= s << 17;
        (s % 1000) as f64 / 1000.0
    };
    let mut out = vec![FrameInput::default(); frames];
    let mut held: Vec<&str> = vec![];
    for (k, f) in out.iter_mut().enumerate() {
        if k % 30 == 0 {
            for h in held.drain(..) {
                f.up(h);
            }
            let r = next();
            let keys: &[&str] = if r < 0.55 {
                &["KeyW"]
            } else if r < 0.7 {
                &["KeyW", "ShiftLeft"]
            } else if r < 0.8 {
                &["KeyA"]
            } else if r < 0.88 {
                &["KeyS"]
            } else {
                &["KeyW", "ControlLeft"]
            };
            for key in keys {
                f.down(key);
                held.push(key);
            }
            if next() < 0.15 {
                f.down("KeyF");
                f.up("KeyF");
            }
        }
        let (dx, dy) = ((next() - 0.5) * 18.0, (next() - 0.5) * 2.0);
        f.look(dx, dy);
    }
    out
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

/// A session recorded the way the game records one (`--record`: a varying
/// time step from the window, its clock, telemetry, the director's events)
/// survives its file and replays to the same states.
#[test]
fn a_recording_survives_its_file_and_replays() {
    let (seed, level) = (77u32, 3);
    let inputs = bot_inputs(1800, seed);
    let native = 0xC0FFEE;
    // the window's frame times are not even
    let dts: Vec<f64> = (0..inputs.len()).map(|k| 1.0 / 60.0 * (1.0 + 0.3 * ((k * 7919 % 13) as f64 / 13.0 - 0.5))).collect();
    let (rec, recorded) = {
        let (inputs, dts) = (inputs.clone(), dts.clone());
        std::thread::Builder::new()
            .stack_size(256 << 20)
            .spawn(move || {
                game::base::rng::seed_native(native);
                let mut g = Game::new_bare();
                placers::register_all(&mut g.placers);
                g.launch_flags = format!("seed={seed}&level={level}&nosound");
                g.boot();
                g.start_new_game();
                g.state = "playing".into();
                g.hide_overlay();
                g.pointer_locked = true;
                let mut r = Recorder::new(native);
                let (mut now, mut hashes) = (0.0, vec![]);
                for (i, dt) in inputs.iter().zip(&dts) {
                    for e in &i.events {
                        r.push(e.clone());
                    }
                    g.apply_input(i);
                    now += dt * 1000.0;
                    g.now_ms = now;
                    g.frame(*dt);
                    r.end_frame(&g, *dt, now);
                    hashes.push(g.state_hash());
                }
                (r.rec.to_json().to_string(), hashes)
            })
            .unwrap()
            .join()
            .unwrap()
    };
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/replays");
    std::fs::create_dir_all(&dir).unwrap();
    // a sample for `cargo xtask heatmap` and `backrooms --replay`
    std::fs::write(dir.join(format!("recorded_{seed}_L{level}.json")), &rec).unwrap();
    let rec = Recording::from_json(&serde_json::from_str(&rec).unwrap());
    assert!(rec.rows.len() >= 25, "a telemetry row a second ({} rows)", rec.rows.len());
    let replayed = std::thread::Builder::new()
        .stack_size(256 << 20)
        .spawn(move || {
            game::base::rng::seed_native(rec.native_seed);
            let mut g = Game::new_bare();
            placers::register_all(&mut g.placers);
            g.launch_flags = format!("seed={seed}&level={level}&nosound");
            g.boot();
            g.start_new_game();
            g.state = "playing".into();
            g.hide_overlay();
            g.pointer_locked = true;
            rec.frames
                .iter()
                .map(|(dt, now, i)| {
                    replay_frame(&mut g, *dt, *now, i);
                    g.state_hash()
                })
                .collect::<Vec<_>>()
        })
        .unwrap()
        .join()
        .unwrap();
    if let Some(k) = (0..recorded.len()).find(|&k| recorded[k] != replayed[k]) {
        panic!("the recording replays differently from frame {k} in {:?}", recorded[k].diff(&replayed[k]));
    }
}
