//! A floor's anchor (`game::session::anchor`, `docs/plan/REPLAYS.md` R2b).
//! Play walks a floor, takes its door and goes on; a fresh game started
//! from the anchor that door took, fed the same frames, must stand in the
//! same state after every one of them. Each runs in a thread of its own
//! (a fresh process's caches and stream), with sound, and the fresh game's
//! stream is seeded otherwise: only the anchor can make them agree.
mod common;

use common::bot_inputs;
use game::Game;
use game::base::game::clock_ticks;
use game::debug::replay::{FrameInput, StateHash, replay_frame};
use game::session::anchor::{Anchor, How};

/// (run seed, the floor played first, frames before leaving it, frames after the next floor is entered, how it is left)
const SCENARIOS: &[(u32, i32, usize, usize, How)] =
    &[(4242, 1, 480, 1200, How::Door), (7, 4, 480, 1200, How::Door), (77, 2, 360, 900, How::Lift), (99, 5, 360, 900, How::Fall)];

fn spawn<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static) -> T {
    std::thread::Builder::new().stack_size(256 << 20).spawn(f).unwrap().join().unwrap()
}

/// The window's uneven frame intervals (seconds)
fn interval(k: usize) -> f64 {
    1.0 / 60.0 * (1.0 + 0.3 * ((k * 7919 % 13) as f64 / 13.0 - 0.5))
}

/// One frame as the window runs it: the input, then its interval in clock ticks.
fn step(g: &mut Game, k: usize, i: &FrameInput) -> u32 {
    g.apply_input(i);
    let ticks = clock_ticks(interval(k));
    let dt = g.advance_clock(ticks);
    g.frame(dt);
    ticks
}

fn booted(seed: u32, level: i32) -> Game {
    let mut g = Game::new_bare();
    placers::register_all(&mut g.placers);
    g.launch_flags = format!("seed={seed}&level={level}");
    g.boot();
    g
}

struct Played {
    anchor: Anchor,
    after: Vec<(u32, FrameInput)>,
    hashes: Vec<StateHash>,
    details: Vec<String>,
}

/// What the world part of a frame's hash is made of, to say where a replay left play
fn detail(g: &Game) -> String {
    format!(
        "level {} state {} ambient_log {} animators {} native draws {} perf_t {} now_ms {} sound {:?}",
        g.level_num,
        g.state,
        g.ambient_log.len(),
        g.animators.len(),
        game::base::rng::native_draws(),
        g.perf_t,
        g.now_ms,
        g.soundscape().map(|s| s.carried())
    )
}

fn play(seed: u32, level: i32, before: usize, after: usize, how: How) -> Played {
    spawn(move || {
        game::base::rng::seed_native(0xA4C0_0000 ^ u64::from(seed));
        let mut g = booted(seed, level);
        g.start_new_game();
        g.state = "playing".into();
        g.hide_overlay();
        g.pointer_locked = true;
        let inputs = bot_inputs(before + after + 400, seed);
        let mut k = 0;
        for _ in 0..before {
            step(&mut g, k, &inputs[k]);
            k += 1;
        }
        // off the floor as walking into its door, its lift or a hole does
        match how {
            How::Lift => g.next_level_via_elevator(),
            How::Fall => g.begin_fall_down(),
            _ => g.next_level(),
        }
        let anchor = loop {
            step(&mut g, k, &inputs[k]);
            k += 1;
            if let Some(a) = g.entered.take() {
                break a;
            }
            assert!(k < before + 400, "seed {seed}: no floor was entered");
        };
        let mut hashes = vec![g.state_hash()];
        let mut details = vec![detail(&g)];
        let mut frames = vec![];
        for _ in 0..after {
            let ticks = step(&mut g, k, &inputs[k]);
            frames.push((ticks, inputs[k].clone()));
            hashes.push(g.state_hash());
            details.push(detail(&g));
            k += 1;
        }
        Played { anchor, after: frames, hashes, details }
    })
}

fn replay(anchor: Anchor, frames: Vec<(u32, FrameInput)>, seed: u32, level: i32) -> (Vec<StateHash>, Vec<String>) {
    spawn(move || {
        // another stream than play's: only the anchor can put it back
        game::base::rng::seed_native(0x0DD5_EED5);
        let mut g = booted(seed, level);
        let dt = g.start_from_anchor(&anchor).expect("the anchor starts a game");
        g.frame(dt);
        let (mut hashes, mut details) = (vec![g.state_hash()], vec![detail(&g)]);
        for (ticks, i) in &frames {
            replay_frame(&mut g, *ticks, i);
            hashes.push(g.state_hash());
            details.push(detail(&g));
        }
        (hashes, details)
    })
}

#[test]
fn a_floor_replays_from_its_anchor() {
    for &(seed, level, before, after, how) in SCENARIOS {
        let played = play(seed, level, before, after, how);
        assert_eq!((played.anchor.level, played.anchor.how), (level + 1, how), "seed {seed}: the way led to the next floor");
        // the anchor survives its JSON (a replay's header, a save)
        let back = Anchor::from_json(&played.anchor.to_json()).expect("the anchor reads back");
        assert_eq!(back, played.anchor, "seed {seed}: the anchor changes through its JSON");
        let (replayed, details) = replay(back, played.after.clone(), seed, level);
        assert_eq!(replayed.len(), played.hashes.len());
        if let Some(k) = (0..replayed.len()).find(|&k| replayed[k] != played.hashes[k]) {
            panic!(
                "seed {seed}: from L{}'s anchor the replay leaves play at frame {k} ({:.2} s after the entry) in {:?}\n  played:   {}\n  replayed: {}",
                level + 1,
                k as f64 / 60.0,
                played.hashes[k].diff(&replayed[k]),
                played.details[k],
                details[k]
            );
        }
        let moved: std::collections::HashSet<u64> = played.hashes.iter().map(|h| h.player).collect();
        assert!(moved.len() > played.hashes.len() / 4, "seed {seed}: the bot barely moved on L{}", level + 1);
        println!("seed {seed}: L{} from its anchor, {} frames as played", level + 1, replayed.len());
    }
}
