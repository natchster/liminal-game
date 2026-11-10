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

#[test]
fn coming_upon_her_camp_tells_your_asides_once() {
    // the Lobby: her camp is there to find
    let (mut g, ac) = start(4242, 4);
    let at = g.story.camp.as_ref().expect("her camp on the Lobby").at;
    play(&mut g, &ac, 1.0, false);
    assert!(!g.story.camp_come && !g.asides.memory.events.contains_key("found"), "not from where you arrive");
    g.player.pos.x = at.x + 1.5;
    g.player.pos.z = at.z;
    play(&mut g, &ac, 0.5, false);
    assert!(g.story.camp_come, "you came upon it");
    let k = "found:camp";
    assert!(g.asides.memory.weighed.iter().any(|w| w == k) || g.asides.memory.owed.contains_key(k), "{:?}", g.asides.memory);
    // away and back: once a floor
    let events = g.asides.memory.events.get("found").copied();
    g.player.pos.x = at.x + 20.0;
    play(&mut g, &ac, 0.5, false);
    g.player.pos.x = at.x + 1.5;
    play(&mut g, &ac, 0.5, false);
    assert_eq!(g.asides.memory.events.get("found").copied(), events);
}

/// Use interactable `i` once (as `tests/usable.rs` does)
fn use_it(g: &mut Game, i: usize) {
    let mut f = std::mem::replace(&mut g.interactables[i].on_use, Box::new(|_| {}));
    f(g);
    g.interactables[i].on_use = f;
}

#[test]
fn a_lifts_car_and_a_candles_notes_tell_your_asides_once_they_have_sounded() {
    let told = |g: &Game, k: &str| g.asides.memory.weighed.iter().any(|w| w == k) || g.asides.memory.owed.contains_key(k);
    let (mut lift, mut candle) = (false, false);
    'floors: for seed in [4242u32, 77, 202, 7, 9001, 31337] {
        for level in 5..=12 {
            let (mut g, ac) = start(seed, level);
            let verbs = |g: &Game, prompt: &str| -> Vec<usize> {
                (0..g.interactables.len()).filter(|&i| g.interactables[i].prompt == prompt).collect()
            };
            for (prompt, key, found) in
                [("[E] PRESS THE BUTTON", "uncanny:lift", &mut lift), ("[E] BLOW OUT", "uncanny:candle", &mut candle)]
            {
                if *found {
                    continue;
                }
                for i in verbs(&g, prompt) {
                    use_it(&mut g, i);
                    // the sound first (a car 4 s on and 2.6 s long; the notes
                    // 2.8 s on): nothing told before it has sounded
                    play(&mut g, &ac, 2.0, false);
                    assert!(!told(&g, key), "{seed}/{level}: told before it sounded");
                    play(&mut g, &ac, 6.0, false);
                    if told(&g, key) {
                        *found = true;
                        println!("{seed}/{level}: {key} told");
                        break;
                    }
                }
            }
            if lift && candle {
                break 'floors;
            }
        }
    }
    assert!(lift, "no lift's car told your asides");
    assert!(candle, "no candle's notes told your asides");
}

#[test]
fn the_lamp_dying_in_the_dark_and_the_stairwell_tell_your_asides() {
    use game::visuals::lighting::RF_DARK;
    // the lamp: dying where the place is dark, and not where it is lit
    let mut dark_told = false;
    let mut lit_seen = false;
    'floors: for seed in [4242u32, 77, 9001] {
        for level in 6..=12 {
            let (mut g, ac) = start(seed, level);
            let tile = |dark: bool| {
                (0..g.h).flat_map(|z| (0..g.w).map(move |x| (x, z))).find(|&(x, z)| {
                    !g.is_wall_tile(x, z)
                        && g.region_map
                            .get(g.t_idx(x, z))
                            .and_then(|r| g.region_info.get(*r as usize))
                            .is_some_and(|r| (r.flags & RF_DARK != 0) == dark)
                })
            };
            let tiles = [(false, tile(false)), (true, tile(true))];
            for (dark, at) in tiles {
                let Some((x, z)) = at else { continue };
                if g.asides.memory.events.contains_key("dark") {
                    break;
                }
                g.player.pos.x = g.wx(x);
                g.player.pos.z = g.wz(z);
                g.player.flashlight_on = true;
                g.player.battery = 0.001;
                play(&mut g, &ac, 0.2, false);
                assert!(!g.player.flashlight_on, "the lamp went out");
                let told = g.asides.memory.events.contains_key("dark");
                assert_eq!(told, dark, "{seed}/{level}: told {told} in a {} place", if dark { "dark" } else { "lit" });
                lit_seen |= !dark;
                dark_told |= dark && told;
                // past the cooldown for the next floor's try
                g.asides.clock += 120.0;
            }
            if dark_told && lit_seen {
                break 'floors;
            }
        }
    }
    assert!(dark_told && lit_seen, "a lamp died in the dark ({dark_told}) and in the light ({lit_seen})");
    // the Stairwell: once its screen has gone, if you are still on its stairs
    let (seed, level) = (4242u32, 5);
    let (mut g, ac) = start(seed, level);
    assert_eq!(g.level_scp.as_deref(), Some("stairwell"), "seed 4242's level 5 opens on the Stairwell");
    g.enter_stairwell();
    play(&mut g, &ac, 4.0, false);
    assert!(g.in_stairwell);
    let k = format!("arrive:stairwell:{level}");
    assert!(!g.asides.memory.weighed.contains(&k) && !g.asides.memory.owed.contains_key(&k), "not while its screen is up");
    play(&mut g, &ac, 4.0, false);
    assert!(g.asides.memory.weighed.contains(&k) || g.asides.memory.owed.contains_key(&k), "{:?}", g.asides.memory);
}
