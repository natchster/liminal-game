//! A knock at a sealed door (`docs/design/MEGASTRUCTURE.md`, G5;
//! `game::world::knock`): in the quiet an ordinary door now and then
//! answers, once, and gives its thing at its foot; a door the cleaners
//! marked red knocks back in your own rhythm and the siren comes within
//! seconds; a door taped round never answers; nobody answers (and no hatch
//! opens) while the siren sounds or the fog is up; what was taken stays taken; and
//! the plan comes from a stream of its own after the build, the same for a
//! seed whatever was built before, moving no other draw.
mod common;

use engine::dmath::DMath;
use game::Game;
use game::base::config::EYE;
use game::world::assembly::Stage;
use game::world::knock::{ANSWER_AFTER, ANSWERING, BACK_AFTER, Due, Gift, KnockDoor, Mark, OUT_AFTER, WRONG_WAKE};

const DT: f64 = 1.0 / 60.0;
/// The floors the rules are held on (the Well, level 1, has the doors)
const SEEDS: [u32; 4] = [4242, 77, 1999, 7];

fn floor(seed: u32, level: i32) -> Game {
    game::base::rng::seed_native(0x4B0C_0000 ^ u64::from(seed) ^ ((level as u64) << 32));
    let mut g = common::new_game();
    // a new run picks its own run seed: the floor asked for is built after it
    g.start_new_game();
    g.run_seed = seed;
    g.init_level(level);
    g.state = "playing".into();
    g.hide_overlay();
    g.pointer_locked = true;
    g
}

fn step(g: &mut Game, secs: f64) {
    for _ in 0..(secs / DT).round() as usize {
        g.perf_t += DT;
        g.now_ms += DT * 1000.0;
        g.fixed_step_dt = DT;
        g.frame(DT);
        g.fixed_step_dt = 0.0;
    }
}

/// Stand at door `i` to knock, facing it
fn stand_at(g: &mut Game, i: usize) {
    let d = g.knocks.doors[i].clone();
    let p = d.stand();
    g.player.pos = engine::math::Vec3::new(p.x, g.floor_y_at(p.x, p.z) + EYE, p.z);
    g.player.yaw = d.normal.0.datan2(d.normal.1);
    g.player.pitch = 0.0;
}

/// Knock at door `i` as a player does: stand there, face it, E
fn knock(g: &mut Game, i: usize) {
    stand_at(g, i);
    step(g, DT);
    let t = g.interact_target().expect("a door in reach");
    assert_eq!(g.interactables[t].prompt, "[E] KNOCK", "facing door {i}, E knocks");
    g.try_interact();
}

fn door_where(g: &Game, f: impl Fn(&KnockDoor) -> bool) -> usize {
    g.knocks.doors.iter().position(f).expect("such a door")
}

/// A long quiet ahead (her page not taken: the siren is minutes off)
fn quiet(g: &mut Game) {
    assert_eq!(g.assembly.stage, Stage::Quiet);
    g.assembly.t = 200.0;
}

#[test]
fn the_well_has_doors_to_knock_at_and_a_few_answer() {
    let (mut doors, mut red, mut taped, mut answer) = (0, 0, 0, 0);
    let mut gifts = std::collections::BTreeMap::new();
    let floors = 24;
    for seed in 1..=floors {
        let g = floor(seed * 7919, 1);
        let n = g.knocks.doors.len();
        assert!(n >= 20, "seed {seed}: the Well's galleries have sealed doors ({n})");
        let a: Vec<_> = g.knocks.doors.iter().filter(|d| d.gift.is_some()).collect();
        let plain = g.knocks.doors.iter().filter(|d| d.mark == Mark::Plain).count();
        assert_eq!(a.len(), ANSWERING.min(plain), "seed {seed}: {} answer", a.len());
        assert!(a.iter().all(|d| d.mark == Mark::Plain), "seed {seed}: only an ordinary door answers");
        for d in &a {
            *gifts.entry(format!("{:?}", d.gift.unwrap())).or_insert(0) += 1;
            if d.gift.unwrap().is_note() {
                if seed <= 6 {
                    println!("  seed {} door {}: {}", seed * 7919, d.number, d.words);
                }
                assert!(
                    d.words.contains(" from ") && (d.words.contains(" 139") || d.words.contains(" 140")),
                    "seed {seed}: a note says the way and the storey: {}",
                    d.words
                );
            }
        }
        doors += n;
        red += g.knocks.doors.iter().filter(|d| d.mark == Mark::Red).count();
        taped += g.knocks.doors.iter().filter(|d| d.mark == Mark::Taped).count();
        answer += a.len();
    }
    let f = floors as f64;
    println!(
        "{floors} Well floors: {:.1} sealed doors a floor, {:.1} marked red, {:.1} taped, {:.1} answer ({:.1}% of the ordinary); what they give {gifts:?}",
        doors as f64 / f,
        red as f64 / f,
        taped as f64 / f,
        answer as f64 / f,
        100.0 * answer as f64 / (doors - red - taped) as f64
    );
    // most doors stay silent
    assert!((answer as f64) < 0.1 * (doors - red - taped) as f64);
    // the other places have no residents' doors
    for level in [2, 3] {
        assert!(floor(4242, level).knocks.doors.is_empty(), "level {level}: no sealed doors");
    }
}

#[test]
fn an_ordinary_door_answers_once_and_gives_its_thing() {
    let mut kinds = std::collections::BTreeSet::new();
    for seed in SEEDS {
        let mut g = floor(seed, 1);
        quiet(&mut g);
        // a door with nothing behind it stays silent
        let silent = door_where(&g, |d| d.mark == Mark::Plain && d.gift.is_none());
        knock(&mut g, silent);
        step(&mut g, ANSWER_AFTER.1 + OUT_AFTER + 2.5);
        assert!(g.knocks.given.is_empty() && g.knocks.due.is_empty(), "seed {seed}: most doors stay silent");
        let answering: Vec<usize> = (0..g.knocks.doors.len()).filter(|&i| g.knocks.doors[i].gift.is_some()).collect();
        for i in answering {
            let gift = g.knocks.doors[i].gift.unwrap();
            kinds.insert(format!("{gift:?}"));
            let (bottles, batteries) = (g.player.bottles, g.player.batteries);
            knock(&mut g, i);
            assert!(g.knocks.doors[i].answered, "seed {seed} door {i}: it will answer");
            // someone comes to the door, the thing comes out at its foot
            step(&mut g, ANSWER_AFTER.0 - 0.1);
            assert!(g.knocks.given.iter().all(|gv| gv.door != i), "seed {seed}: not before {} s", ANSWER_AFTER.0);
            step(&mut g, ANSWER_AFTER.1 - ANSWER_AFTER.0 + OUT_AFTER + 1.2);
            let gv = g.knocks.given.iter().find(|gv| gv.door == i).expect("it answered").clone();
            let (d, p) = (&g.knocks.doors[i], gv.obj.pos());
            let foot = d.point(0.5, 0.0, 0.0);
            assert!(
                ((p.x - foot.x).dpowi(2) + (p.z - foot.z).dpowi(2)).sqrt() < 0.3 && (p.y - foot.y).abs() < 0.15,
                "seed {seed} door {i}: it lies at the door's foot ({p:?} vs {foot:?})"
            );
            // standing there, it is picked up once it lies still
            step(&mut g, 1.5);
            assert!(g.knocks.given.iter().find(|gv| gv.door == i).unwrap().taken, "seed {seed} door {i}: {gift:?} picked up");
            match gift {
                Gift::Water => assert_eq!(g.player.bottles, bottles + 1, "seed {seed}: a bottle of almond water"),
                Gift::Battery => assert_eq!(g.player.batteries, batteries + 1, "seed {seed}: a battery"),
                _ => assert!(g.msg_text.contains(&g.knocks.doors[i].words), "seed {seed}: the note read: {}", g.msg_text),
            }
            assert!(g.secrets_taken.contains_key(&KnockDoor::key(1, i)), "seed {seed}: kept for the run");
            // once a door: knocked again, nothing more
            let n = g.knocks.given.len();
            knock(&mut g, i);
            step(&mut g, ANSWER_AFTER.1 + OUT_AFTER + 2.0);
            assert_eq!(g.knocks.given.len(), n, "seed {seed} door {i}: a door answers once");
        }
        assert_eq!(g.assembly.stage, Stage::Quiet, "seed {seed}: an ordinary door calls nothing");
        assert!(g.assembly.t > 100.0);
        // your asides were told (`audio::asides`, Use::Answered): weighed or owed
        let told = |k: &str| g.asides.memory.weighed.iter().any(|w| w == k) || g.asides.memory.owed.contains_key(k);
        assert!(told("answered:hatch") || told("answered:note"), "seed {seed}: {:?}", g.asides.memory);
    }
    assert!(kinds.len() >= 3, "water, a battery and a note among them: {kinds:?}");
}

#[test]
fn a_red_door_brings_the_siren_within_seconds() {
    for seed in SEEDS {
        let mut g = floor(seed, 1);
        quiet(&mut g);
        let red = door_where(&g, |d| d.mark == Mark::Red);
        knock(&mut g, red);
        // it knocks back in your own rhythm
        let Some((t, Due::KnockBack(i, gaps))) = g.knocks.due.first().cloned() else { panic!("seed {seed}: the red door answers") };
        assert_eq!(i, red);
        assert_eq!(gaps, g.knocks.rhythm, "seed {seed}: your own rhythm");
        assert!((BACK_AFTER.0..=BACK_AFTER.1).contains(&t));
        let mut at = None;
        // (the knock back's time on your asides' own clock, as their log keeps it)
        let mut back_clock = None;
        for k in 1..=((BACK_AFTER.1 + WRONG_WAKE.1 + 1.0) / 0.1) as usize {
            step(&mut g, 0.1);
            if back_clock.is_none() && g.director.log.iter().any(|e| e.1 == "knock:wrong") {
                back_clock = Some(g.asides.clock);
            }
            if g.assembly.stage == Stage::Siren {
                at = Some(k as f64 * 0.1);
                break;
            }
        }
        let at = at.unwrap_or_else(|| {
            panic!("seed {seed}: no siren within seconds of the red door ({:?}, {:.0} s)", g.assembly.stage, g.assembly.t)
        });
        assert!(g.director.log.iter().any(|e| e.1 == "knock:wrong") && g.director.log.iter().any(|e| e.1 == "wake:knock"));
        // your asides were told of your own knock back (weighed or owed),
        // once its last rap had rung out, never over it
        let k = "uncanny:knock_back";
        assert!(
            g.asides.memory.weighed.iter().any(|w| w == k) || g.asides.memory.owed.contains_key(k),
            "seed {seed}: {:?}",
            g.asides.memory
        );
        // (both on one clock, the knock back's to within the 0.1 s step it fell in)
        let back_at = back_clock.expect("the knock back was logged");
        let told_at = g.asides.log.iter().find(|(_, w)| w.contains("uncanny:knock_back")).map(|(t, _)| *t).unwrap();
        let raps: f64 = g.knocks.rhythm.iter().sum();
        assert!(
            told_at >= back_at + raps + game::world::knock::KNOCK_HEARD - 0.12,
            "seed {seed}: told {:.2} s after the knock back began, its raps last {raps:.2} s",
            told_at - back_at
        );
        println!("seed {seed}: the siren {at:.1} s after the knock at red door {}", g.knocks.doors[red].number);
        // and then it is still: after the fog, it answers no more
        g.assembly.stage = Stage::Quiet;
        g.assembly.y = f64::NEG_INFINITY;
        g.assembly.t = 200.0;
        knock(&mut g, red);
        assert!(g.knocks.due.is_empty(), "seed {seed}: once it has answered, still");
        step(&mut g, 8.0);
        assert_eq!(g.assembly.stage, Stage::Quiet);
    }
}

#[test]
fn a_taped_door_never_answers() {
    for seed in SEEDS {
        let mut g = floor(seed, 1);
        quiet(&mut g);
        let taped: Vec<usize> = (0..g.knocks.doors.len()).filter(|&i| g.knocks.doors[i].mark == Mark::Taped).collect();
        assert!(!taped.is_empty(), "seed {seed}: doors taped round");
        assert!(taped.iter().all(|&i| g.knocks.doors[i].gift.is_none()), "seed {seed}: nothing behind tape is given");
        for &i in taped.iter().take(4) {
            for _ in 0..3 {
                knock(&mut g, i);
                step(&mut g, 4.0);
            }
        }
        assert!(g.knocks.due.is_empty() && g.knocks.given.is_empty(), "seed {seed}: taped round, never");
        assert_eq!(g.assembly.stage, Stage::Quiet);
        assert!(g.assembly.t > 100.0, "seed {seed}: nor does it call the siren");
    }
}

#[test]
fn nobody_answers_while_it_is_up() {
    for seed in SEEDS {
        let mut g = floor(seed, 1);
        g.god_mode = true;
        let gift = door_where(&g, |d| d.gift.is_some());
        let red = door_where(&g, |d| d.mark == Mark::Red);
        for stage in [Stage::Siren, Stage::Rising, Stage::Crest, Stage::Falling] {
            g.assembly.stage = stage;
            g.assembly.t = 30.0;
            g.assembly.y = g.assembly.low - 10.0;
            for i in [gift, red] {
                knock(&mut g, i);
                step(&mut g, 0.05);
                assert!(g.knocks.due.is_empty() && !g.knocks.doors[i].answered, "seed {seed} {stage:?}: nobody answers while it is up");
            }
        }
        // a knock in the last of the quiet: the siren comes before the answer,
        // and nobody opens; the door answers another time
        g.assembly.stage = Stage::Quiet;
        g.assembly.y = f64::NEG_INFINITY;
        g.assembly.t = 1.0;
        knock(&mut g, gift);
        step(&mut g, ANSWER_AFTER.1 + OUT_AFTER + 2.0);
        assert!(g.assembly.up());
        assert!(g.knocks.given.is_empty() && !g.knocks.doors[gift].answered, "seed {seed}: never opened while it was up");
        // someone already at the door when the siren starts: the hatch stays shut
        g.assembly.stage = Stage::Quiet;
        g.assembly.y = f64::NEG_INFINITY;
        g.assembly.t = 200.0;
        knock(&mut g, gift);
        for _ in 0..600 {
            step(&mut g, DT);
            if g.knocks.given.iter().any(|gv| gv.door == gift) {
                break;
            }
        }
        assert!(g.knocks.given.iter().any(|gv| gv.door == gift && gv.t < 0.0), "seed {seed}: someone at the door");
        g.assembly.t = 0.0;
        step(&mut g, OUT_AFTER + 2.0);
        assert!(g.assembly.up());
        assert!(g.knocks.given.is_empty() && !g.knocks.doors[gift].answered, "seed {seed}: no hatch opened while it was up");
        // in the quiet after, it does
        g.assembly.stage = Stage::Quiet;
        g.assembly.y = f64::NEG_INFINITY;
        g.assembly.t = 200.0;
        knock(&mut g, gift);
        step(&mut g, ANSWER_AFTER.1 + OUT_AFTER + 0.5);
        assert!(g.knocks.given.iter().any(|gv| gv.door == gift), "seed {seed}: in the quiet after, it answers");
    }
}

#[test]
fn what_was_taken_stays_taken() {
    let mut g = floor(4242, 1);
    quiet(&mut g);
    let i = door_where(&g, |d| d.gift.is_some());
    knock(&mut g, i);
    step(&mut g, ANSWER_AFTER.1 + OUT_AFTER + 3.0);
    assert!(g.knocks.given.iter().any(|gv| gv.door == i && gv.taken));
    // a retry: the floor again, as the run has it
    g.init_level(1);
    g.state = "playing".into();
    assert!(g.knocks.doors[i].answered, "the door has answered, for the rest of the run");
    assert!(g.knocks.doors.iter().enumerate().all(|(k, d)| k == i || !d.answered));
    quiet(&mut g);
    knock(&mut g, i);
    step(&mut g, ANSWER_AFTER.1 + OUT_AFTER + 2.0);
    assert!(g.knocks.given.is_empty(), "nothing is given twice");
}

#[test]
fn the_plan_is_its_own_and_the_same_for_a_seed() {
    type Plan = Vec<(u32, Mark, Option<Gift>, String)>;
    let plan = |g: &Game| -> Plan { g.knocks.doors.iter().map(|d| (d.number, d.mark, d.gift, d.words.clone())).collect() };
    for seed in SEEDS {
        let a = floor(seed, 1);
        // built after other floors, the same
        let mut b = floor(seed, 3);
        b.init_level(2);
        b.init_level(1);
        assert_eq!(plan(&a), plan(&b), "seed {seed}: the same plan whatever came before");
        // planned again: no draw from the build's stream nor the per-frame one
        let mut c = floor(seed, 1);
        let before = game::base::rng::seeded_draws();
        game::base::rng::seed_native(99);
        let want = game::base::rng::native_rnd();
        game::base::rng::seed_native(99);
        c.knocks.planned = false;
        c.fit_knocks();
        assert_eq!(game::base::rng::native_rnd(), want, "seed {seed}: no per-frame draw");
        assert_eq!(game::base::rng::seeded_draws(), before, "seed {seed}: no draw of the build");
        assert_eq!(plan(&a), plan(&c));
    }
}
