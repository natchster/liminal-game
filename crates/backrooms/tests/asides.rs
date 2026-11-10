//! Your asides in the whole game (`docs/VOICES.md`, *Your asides*;
//! `game::audio::asides`): on real floors of the rooms, with a player who
//! keeps looking round and creatures held still, the voice director answers
//! the place: each kind of sudden sound at most once a run, never two
//! asides within the gap, never a line twice, and the same run says the
//! same.
mod common;

use engine::audio::AudioContext;
use game::audio::asides::{Use, GAP};
use game::Game;

const DT: f64 = 1.0 / 60.0;
/// The floors (run seed, level), as the loudness scenarios
const FLOORS: [(u32, i32); 3] = [(4242, 6), (77, 8), (9001, 11)];

/// `secs` on a new run's `level`, its sound rendered offline, looking round
fn run(seed: u32, level: i32, secs: f64) -> Game {
    game::base::rng::seed_native((u64::from(seed) << 8) | level as u64);
    let mut g = Game::new_bare();
    common::register_placers(&mut g.placers);
    g.launch_flags = format!("seed={seed}&level={level}");
    let ac = AudioContext::new_offline(48000.0);
    g.audio_init_with(ac.clone());
    g.boot();
    g.start_new_game();
    g.god_mode = true;
    for _ in 0..(secs / DT).round() as usize {
        for e in g.entities.iter_mut() {
            e.freeze = 1e9;
        }
        // looking round: a person is playing
        g.player.yaw += 0.004;
        g.now_ms += DT * 1000.0;
        g.fixed_step_dt = DT;
        g.frame(DT);
        g.fixed_step_dt = 0.0;
        ac.render_offline(800);
    }
    g
}

fn says(g: &Game) -> Vec<(f64, String)> {
    g.asides.log.iter().filter(|(_, w)| w.starts_with("say:")).cloned().collect()
}

#[test]
fn a_sudden_sound_is_answered_once_a_kind_and_the_asides_keep_their_gap() {
    let mut weighed = 0;
    for (seed, level) in FLOORS {
        let g = run(seed, level, 150.0);
        println!("seed {seed} level {level}:");
        for (t, w) in &g.asides.log {
            println!("  {t:7.1} {w}");
        }
        let s = says(&g);
        // never two within the gap (the urgent ones are not posted here)
        for p in s.windows(2) {
            assert!(p[1].0 - p[0].0 >= GAP - 1e-6, "{seed}/{level}: {:?} then {:?}", p[0], p[1]);
        }
        // each kind of sound at most once, and never a line twice
        let mut kinds = vec![];
        let mut lines = vec![];
        for (_, w) in &s {
            let f: Vec<&str> = w.split(':').collect();
            if f[1] == Use::Startle.name() {
                assert!(!kinds.contains(&f[2]), "{seed}/{level}: a second startle at {}: {w}", f[2]);
                kinds.push(f[2]);
            }
            assert!(!lines.contains(&f[3]), "{seed}/{level}: {} said twice", f[3]);
            lines.push(f[3]);
        }
        weighed += g.asides.memory.events.get("startle").copied().unwrap_or(0);
    }
    assert!(weighed > 0, "no sudden sound was weighed on any floor: the hooks are dead");
}

#[test]
fn the_same_run_says_the_same() {
    let (seed, level) = FLOORS[0];
    let a = run(seed, level, 90.0);
    let b = run(seed, level, 90.0);
    assert_eq!(a.asides.log, b.asides.log);
    assert_eq!(a.asides.memory, b.asides.memory);
}
