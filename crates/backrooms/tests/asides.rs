//! Your asides in the whole game (`docs/VOICES.md`, *Your asides*;
//! `game::audio::asides`): on real floors of the rooms, with creatures
//! held still, the voice director answers the place: sudden sounds are
//! answered and said, each kind at most once a run, never two asides within
//! the gap, never a line twice; a floor's arrival comes with your first
//! steps, and a retry or a Continue of the floor never says it again; and
//! the same run says the same.
mod common;

use engine::audio::AudioContext;
use game::audio::asides::{Use, GAP};
use game::Game;

const DT: f64 = 1.0 / 60.0;
/// The floors (run seed, level), as the loudness scenarios
const FLOORS: [(u32, i32); 3] = [(4242, 6), (77, 8), (9001, 11)];

/// A new run on `level`, its sound rendered offline
fn start(seed: u32, level: i32) -> (Game, AudioContext) {
    game::base::rng::seed_native((u64::from(seed) << 8) | level as u64);
    let mut g = Game::new_bare();
    common::register_placers(&mut g.placers);
    g.launch_flags = format!("seed={seed}&level={level}");
    let ac = AudioContext::new_offline(48000.0);
    g.audio_init_with(ac.clone());
    g.boot();
    g.start_new_game();
    g.god_mode = true;
    (g, ac)
}

/// `secs` of play, looking round, walking on if `walk`
fn play(g: &mut Game, ac: &AudioContext, secs: f64, walk: bool) {
    for _ in 0..(secs / DT).round() as usize {
        for e in g.entities.iter_mut() {
            e.freeze = 1e9;
        }
        // looking round: a person is playing
        g.player.yaw += 0.004;
        if walk {
            g.keys.insert("KeyW".into());
        }
        g.now_ms += DT * 1000.0;
        g.fixed_step_dt = DT;
        g.frame(DT);
        g.fixed_step_dt = 0.0;
        ac.render_offline(800);
    }
    g.keys.clear();
}

fn says(g: &Game) -> Vec<(f64, String)> {
    g.asides.log.iter().filter(|(_, w)| w.starts_with("say:")).cloned().collect()
}

#[test]
fn a_sudden_sound_is_answered_once_a_kind_and_the_asides_keep_their_gap() {
    let mut startles = 0;
    for (seed, level) in FLOORS {
        let (mut g, ac) = start(seed, level);
        play(&mut g, &ac, 240.0, false);
        println!("seed {seed} level {level}:");
        for (t, w) in &g.asides.log {
            println!("  {t:7.1} {w}");
        }
        assert!(!g.asides.log.iter().any(|(_, w)| w.ends_with("unknown subject")), "{:?}", g.asides.log);
        let s = says(&g);
        for p in s.windows(2) {
            assert!(p[1].0 - p[0].0 >= GAP - 1e-6, "{seed}/{level}: {:?} then {:?}", p[0], p[1]);
        }
        let mut kinds = vec![];
        let mut lines = vec![];
        for (_, w) in &s {
            let f: Vec<&str> = w.split(':').collect();
            if f[1] == Use::Startle.name() {
                assert!(!kinds.contains(&f[2]), "{seed}/{level}: a second startle at {}: {w}", f[2]);
                kinds.push(f[2]);
                startles += 1;
            }
            assert!(!lines.contains(&f[3]), "{seed}/{level}: {} said twice", f[3]);
            lines.push(f[3]);
        }
    }
    // said, not only weighed: the sounds of three floors bring at least two
    assert!(startles >= 2, "{startles} startles said on three floors");
}

#[test]
fn a_floors_arrival_comes_with_your_first_steps_and_a_retry_or_a_continue_never_says_it_again() {
    let (seed, level) = FLOORS[1];
    let (mut g, ac) = start(seed, level);
    play(&mut g, &ac, 20.0, true);
    let first = says(&g);
    assert!(first.iter().any(|(_, w)| w.starts_with("say:arrive:")), "the run's first floor always has its arrival: {:?}", g.asides.log);
    // you die on the floor, and try it again
    g.restart();
    play(&mut g, &ac, 20.0, true);
    assert_eq!(says(&g).iter().filter(|(_, w)| w.starts_with("say:arrive:")).count(), 1, "{:?}", g.asides.log);
    // you leave without a word to the game, and continue: the save has
    // every line said (each saved as it was said)
    let said = g.asides.memory.said.clone();
    g.continue_game();
    assert_eq!(g.asides.memory.said, said);
    play(&mut g, &ac, 20.0, true);
    let arrivals: Vec<_> = says(&g).into_iter().filter(|(_, w)| w.starts_with("say:arrive:")).collect();
    assert_eq!(arrivals.len(), 1, "{arrivals:?}");
    let ids: Vec<&String> = g.asides.memory.said.iter().collect();
    let mut unique = ids.clone();
    unique.sort();
    unique.dedup();
    assert_eq!(ids.len(), unique.len(), "a line said twice: {ids:?}");
}

#[test]
fn the_same_run_says_the_same() {
    let (seed, level) = FLOORS[0];
    let run = || {
        let (mut g, ac) = start(seed, level);
        play(&mut g, &ac, 90.0, false);
        g
    };
    let (a, b) = (run(), run());
    assert_eq!(a.asides.log, b.asides.log);
    assert_eq!(a.asides.memory, b.asides.memory);
}
