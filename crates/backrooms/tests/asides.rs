//! Your asides in the whole game (`docs/design/VOICES.md`, *Your asides*;
//! `game::audio::asides`): on real floors of the rooms, with creatures
//! held still, the voice director answers the place: sudden sounds are
//! answered and said, each kind at most once a run, never two asides within
//! the gap, never a line twice; a floor's arrival comes with your first
//! steps, and a retry or a Continue of the floor never says it again; the
//! same run says the same; and what you come upon, use, pick up or meet first
//! tells the director (GitHub issue #10), each once, a thing out of place
//! only the first of a floor, a word about a sound only once it has sounded.
mod common;

use engine::audio::AudioContext;
use game::Game;
use game::audio::asides::{GAP, Use};

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

/// A whole run's floors in one game, the run's memory carried down (review
/// B): from the Lobby to level 13, each floor walked into and looked round
/// for 40 s. Many floors have their word on arriving (an arrival fades only
/// by its place), the first always, and no line is said twice in the run.
#[test]
fn a_run_down_ten_floors_arrives_with_a_word_on_many_and_never_says_one_twice() {
    for seed in [4242u32, 77, 9001] {
        let (mut g, ac) = start(seed, 4);
        let mut arrived = vec![];
        for level in 4..=13 {
            if level > 4 {
                g.init_level(level);
                g.state = "playing".into();
            }
            let before = g.asides.memory.said.len();
            play(&mut g, &ac, 40.0, true);
            let new: Vec<String> = g.asides.memory.said[before..].to_vec();
            arrived.push(new.iter().any(|id| id.starts_with("arrive_")));
            println!("seed {seed} L{level} ({}): {new:?}", g.arrival_subject());
        }
        let ids = &g.asides.memory.said;
        let mut unique = ids.clone();
        unique.sort();
        unique.dedup();
        assert_eq!(unique.len(), ids.len(), "seed {seed}: a line said twice in the run: {ids:?}");
        assert!(arrived[0], "seed {seed}: the run's first floor here has its word");
        let n = arrived.iter().filter(|a| **a).count();
        println!("seed {seed}: an arrival on {n} of 10 floors");
        assert!(n >= 4, "seed {seed}: an arrival on only {n} of 10 floors: {arrived:?}");
    }
}

/// Whether your asides were told of `k` (`use:subject`): weighed, or owed
fn told(g: &Game, k: &str) -> bool {
    g.asides.memory.weighed.iter().any(|w| w == k) || g.asides.memory.owed.contains_key(k)
}

/// Stand `back` metres before `at`, facing it, the lamp on
fn stand_before(g: &mut Game, at: engine::math::Vec3, back: f64) {
    use engine::dmath::DMath;
    let p = g.player.pos;
    let (dx, dz) = (at.x - p.x, at.z - p.z);
    let d = dx.dhypot(dz).max(1e-6);
    g.player.pos.x = at.x - dx / d * back;
    g.player.pos.z = at.z - dz / d * back;
    g.player.yaw = (-dx).datan2(-dz);
    g.player.flashlight_on = true;
}

/// What you come upon tells your asides (GitHub issue #10): a window, the
/// floor's door, the working lift, the broken one, a thing out of place
/// (only the first of a floor), each once; not from where you arrive.
#[test]
fn coming_upon_things_tells_your_asides_once_and_one_odd_thing_a_floor() {
    let mut seen: std::collections::BTreeSet<String> = Default::default();
    let mut odd_floor = false;
    'floors: for seed in [4242u32, 77, 202, 9001] {
        for level in 5..=9 {
            let (mut g, ac) = start(seed, level);
            play(&mut g, &ac, 0.5, false);
            let things = g.noticing.things.clone();
            for n in &things {
                let k = format!("{}:{}", n.use_.name(), n.subject);
                if seen.contains(&k) || !g.noticing.things.contains(n) {
                    continue;
                }
                stand_before(&mut g, n.at, 1.0);
                play(&mut g, &ac, 0.5, false);
                assert!(!g.noticing.things.contains(n), "{seed}/{level}: {k} at {:?} not come upon from 1 m", n.at);
                if told(&g, &k) {
                    seen.insert(k.clone());
                }
                // a thing out of place: the floor's others are let go
                if n.use_ == Use::Odd {
                    assert!(g.noticing.odd_done && !g.noticing.things.iter().any(|t| t.use_ == Use::Odd), "{seed}/{level}");
                    if let Some(other) = things.iter().find(|t| t.use_ == Use::Odd && t.subject != n.subject) {
                        let after = g.asides.memory.events.get("odd").copied().unwrap_or(0);
                        stand_before(&mut g, other.at, 1.0);
                        play(&mut g, &ac, 0.5, false);
                        assert_eq!(g.asides.memory.events.get("odd").copied().unwrap_or(0), after, "{seed}/{level}: a second odd thing");
                        odd_floor = true;
                    }
                }
            }
            let want = ["noticed:window", "noticed:exit", "noticed:lift", "noticed:lift_broken"];
            if want.iter().all(|k| seen.contains(*k)) && seen.iter().any(|k| k.starts_with("odd:")) && odd_floor {
                break 'floors;
            }
        }
    }
    println!("told: {seen:?}");
    for k in ["noticed:window", "noticed:exit", "noticed:lift", "noticed:lift_broken"] {
        assert!(seen.contains(k), "no {k} told your asides: {seen:?}");
    }
    assert!(seen.iter().any(|k| k.starts_with("odd:")), "no thing out of place told your asides: {seen:?}");
    assert!(odd_floor, "no floor with two kinds of odd things to hold to one");
}

/// The vacuum met, a note picked up: your asides are told, the note once
/// its chime has rung.
#[test]
fn meeting_the_vacuum_and_a_note_tell_your_asides() {
    let (mut vacuum, mut note) = (false, false);
    'floors: for seed in [4242u32, 77, 202, 9001, 7] {
        for level in 5..=10 {
            let (mut g, ac) = start(seed, level);
            play(&mut g, &ac, 0.5, false);
            if !vacuum && let Some(at) = g.vacuum_bot.as_ref().map(|v| v.g.pos()) {
                stand_before(&mut g, at, 1.0);
                play(&mut g, &ac, 0.5, false);
                assert!(g.noticing.vacuum_met, "{seed}/{level}: the vacuum not met from 1 m");
                vacuum |= told(&g, "noticed:vacuum");
            }
            if !note && let Some(at) = g.lore_notes.iter().find(|n| !n.taken).map(|n| n.g.pos()) {
                g.player.pos.x = at.x;
                g.player.pos.z = at.z;
                play(&mut g, &ac, 0.4, false);
                assert!(!told(&g, "found:note"), "{seed}/{level}: told before the chime");
                play(&mut g, &ac, 1.5, false);
                note |= told(&g, "found:note");
            }
            if vacuum && note {
                break 'floors;
            }
        }
    }
    assert!(vacuum, "no vacuum met told your asides");
    assert!(note, "no note picked up told your asides");
}

/// Using things (GitHub issue #10): the phone answered, the first coin in a
/// machine, a dead lift's button that brings nothing; each told once what
/// it set off has sounded, never before.
#[test]
fn using_the_phone_a_machine_and_a_dead_lift_tells_your_asides_once_they_have_sounded() {
    use game::core_props::vending::rules::Verb;
    let mut found: std::collections::BTreeSet<&str> = Default::default();
    'floors: for seed in [4242u32, 77, 202, 7, 9001, 31337] {
        for level in 5..=12 {
            let (mut g, ac) = start(seed, level);
            play(&mut g, &ac, 0.5, false);
            if !found.contains("used:phone") && g.phone_data.is_some() {
                g.answer_payphone();
                play(&mut g, &ac, 1.5, false);
                assert!(!told(&g, "used:phone"), "{seed}/{level}: told before it had its say");
                play(&mut g, &ac, 3.0, false);
                if told(&g, "used:phone") {
                    found.insert("used:phone");
                }
            }
            if !found.contains("used:vending") && !g.vending.is_empty() {
                g.vending[0].state.gaze = Some(0);
                g.player.coins = 1;
                g.vending_use(0, Verb::InsertCoin);
                play(&mut g, &ac, 1.0, false);
                assert!(!told(&g, "used:vending"), "{seed}/{level}: told before the good turned out");
                play(&mut g, &ac, 3.0, false);
                if told(&g, "used:vending") {
                    found.insert("used:vending");
                }
            }
            if !found.contains("used:lift_dead") {
                let buttons: Vec<usize> =
                    (0..g.interactables.len()).filter(|&i| g.interactables[i].prompt == "[E] PRESS THE BUTTON").collect();
                for i in buttons {
                    use_it(&mut g, i);
                    play(&mut g, &ac, 0.6, false);
                    assert!(!told(&g, "used:lift_dead"), "{seed}/{level}: told before the button clicked");
                    play(&mut g, &ac, 2.0, false);
                    if told(&g, "used:lift_dead") {
                        found.insert("used:lift_dead");
                        break;
                    }
                }
            }
            if found.len() == 3 {
                break 'floors;
            }
        }
    }
    assert_eq!(found.len(), 3, "told: {found:?}");
}

/// The run's first creature is a meeting (GitHub issue #10), whatever its
/// kind; a kind after it is a sighting of its own.
#[test]
fn the_first_creature_of_a_run_is_a_meeting_and_the_next_kinds_are_sightings() {
    // the kinds a first sight can be (`Use::Sighted`'s subjects: never the
    // Watcher, nor an anomaly's figures)
    let kinds_of = |g: &Game| -> Vec<(usize, String)> {
        let mut seen: Vec<(usize, String)> = vec![];
        for (i, e) in g.entities.iter().enumerate() {
            if Use::Sighted.subjects().contains(&e.kind.as_str()) && !seen.iter().any(|(_, k)| *k == e.kind) {
                seen.push((i, e.kind.clone()));
            }
        }
        seen
    };
    let mut floors = 0;
    'floors: for seed in [4242u32, 77, 202, 9001] {
        for level in 6..=12 {
            let (mut g, ac) = start(seed, level);
            let kinds = kinds_of(&g);
            if kinds.len() < 2 {
                continue;
            }
            play(&mut g, &ac, 0.5, false);
            g.entity_sighted(kinds[0].0, 10.0);
            assert!(told(&g, "met:"), "{seed}/{level}: the first creature was no meeting: {:?}", g.asides.memory);
            assert!(!g.asides.memory.events.contains_key("sighted"));
            g.entity_sighted(kinds[1].0, 10.0);
            assert_eq!(g.asides.memory.events.get("met").copied(), Some(1), "{seed}/{level}: a second meeting");
            assert!(told(&g, &format!("sighted:{}", kinds[1].1)), "{seed}/{level}: the second kind was no sighting");
            floors += 1;
            if floors == 2 {
                break 'floors;
            }
        }
    }
    assert_eq!(floors, 2, "two floors with two kinds of creature to meet");
}
