//! The self-assembly on the Building's floors (`docs/design/MEGASTRUCTURE.md`,
//! B5; `game::world::assembly`): the siren before the fog, the fog rising
//! out of the void and sinking again in the director's phases, a sealed
//! cell keeps it out, out in it you are gone, her page sounds the siren,
//! and every floor tile has a cell within reach of the warning at a walk
//! (G2: each of these half as long as the first pass's). What knocks at a
//! sealed cell (G6, `game::world::cell_knock`): unanswered it goes; answered
//! it tries the wheel, and the door gives unless you hold it (a visit that
//! does not give leaves the wheel at rest shut); a door that gives, or a
//! cell left open, lets the fog in. Its prompts follow what is heard and
//! seen at the door (`MILESTONES.md` 13): the hold from the wheel's first
//! pull, the door to open once its going, heard, is over.
mod common;

use game::base::config::{CELL, EYE, SAN_REGEN, WALK};
use game::entities::director::Phase;
use game::level::building::CELL_REACH;
use game::session::input::{Act, INTERACT_BUFFER};
use game::world::assembly::{Stage, DRAIN, FIRST_FLOOR_QUIET, FIRST_QUIET, KNOCK, LAST, PAGE_WAKE, QUIET, WARNING, WHEEL_OPEN, WHEEL_SHUT};
use game::world::cell_knock::{
    Visit, BREATH, GAVE_OPEN, GIVE, GOING_BREATHS, GOING_TAIL, KNOCK_RINGS, PATIENCE, PLAY, SLACK, TRIES, WHEEL_AFTER,
};
use game::Game;

const DT: f64 = 1.0 / 60.0;

/// A player's reaction to the wheel's first pull (its squeal, the wheel
/// moving in its play, the prompt): a simple reaction to a sound or a light
/// is 0.15–0.3 s; here the slow end of it, with a frame for the screen
const REACTION: f64 = 0.3;

/// The cell door's verbs
const OPEN: &str = "[E] OPEN THE DOOR";
const HOLD: &str = "[E] HOLD THE WHEEL";
const KNOCK_BACK: &str = "[E] KNOCK";
const DOOR_VERBS: [&str; 4] = ["[E] TURN THE WHEEL", OPEN, KNOCK_BACK, HOLD];

/// The shortest a visit's going lasts, from when it is first heard: its
/// breaths from the first to the last at the quickest pace, and the last
/// dying away
const GOING_LEAST: f64 = (GOING_BREATHS - 1) as f64 * BREATH.0 + GOING_TAIL;

fn floor(seed: u32, level: i32) -> Game {
    game::base::rng::seed_native(0xA55E_0000 ^ u64::from(seed) ^ ((level as u64) << 32));
    let mut g = common::new_game();
    // a new run picks its own run seed (from the native stream): the floor
    // asked for is built after it, with the seed asked for
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

/// Stand the body on tile (x, z), on its floor
fn stand(g: &mut Game, x: i32, z: i32) {
    let (wx, wz) = (g.wx(x), g.wz(z));
    g.player.pos = engine::math::Vec3::new(wx, g.floor_y_at(wx, wz) + EYE, wz);
}

#[test]
fn the_siren_comes_first_then_the_fog_rises_and_sinks() {
    let mut g = floor(4242, 1);
    g.god_mode = true;
    // stand in a cell, sealed, out of the way
    let c = g.building.as_ref().unwrap().cells[0].tile;
    stand(&mut g, c.0, c.1);
    g.turn_cell_wheel(0, true);
    g.assembly.t = 1.0;
    let (mut stages, mut last_y) = (vec![], f64::NEG_INFINITY);
    let mut rose_while = (true, true);
    for _ in 0..(400.0 / 0.25) as usize {
        step(&mut g, 0.25);
        let a = &g.assembly;
        if stages.last() != Some(&a.stage) {
            stages.push(a.stage);
        }
        match a.stage {
            Stage::Siren | Stage::Rising => rose_while.0 &= a.y >= last_y - 1e-9,
            Stage::Falling => rose_while.1 &= a.y <= last_y + 1e-9,
            _ => {}
        }
        last_y = a.y;
        if stages.len() >= 6 && a.stage == Stage::Quiet {
            break;
        }
    }
    assert_eq!(
        stages,
        vec![Stage::Quiet, Stage::Siren, Stage::Rising, Stage::Crest, Stage::Falling, Stage::Quiet],
        "the order: {stages:?}"
    );
    assert!(rose_while.0 && rose_while.1, "it rises, then it sinks");
    // the director's log reads as on the rooms: build, peak, relax
    let log: Vec<&str> = g.director.log.iter().map(|e| e.1).filter(|w| ["build", "peak", "relax"].contains(w)).collect();
    assert_eq!(log, vec!["build", "peak", "relax"]);
    assert_eq!(g.director.phase, Phase::Calm);
    assert_eq!(g.state, "playing");
}

#[test]
fn a_sealed_cell_keeps_it_out_and_out_in_it_you_are_gone() {
    for sealed in [true, false] {
        let mut g = floor(4242, 1);
        let c = g.building.as_ref().unwrap().cells[0].tile;
        stand(&mut g, c.0, c.1);
        if sealed {
            g.turn_cell_wheel(0, true);
        }
        // the fog stands over the cell's floor
        let h = g.floor_y_of(c.0, c.1);
        g.assembly.stage = Stage::Crest;
        g.assembly.y = h + 6.0;
        g.assembly.crest = h + 6.0;
        g.assembly.t = 30.0;
        let s0 = g.player.sanity;
        step(&mut g, 14.0);
        if sealed {
            assert_eq!(g.state, "playing", "sealed in, it does not come in");
            assert!(g.player.sanity >= s0 - 1e-9, "sealed in, the mind holds: {s0} → {}", g.player.sanity);
        } else {
            assert_eq!(g.state, "dead", "out in it");
            assert_eq!(g.last_death_cause.as_deref(), Some("assembly"));
        }
    }
}

#[test]
fn a_shut_door_stops_a_body_and_an_open_one_lets_it_by() {
    let mut g = floor(4242, 2);
    let d = g.cell_doors[0].clone();
    let (mx, mz) = ((d.span[0] + d.span[2]) / 2.0, (d.span[1] + d.span[3]) / 2.0);
    assert!(!g.hits_cell_door(mx, mz, 0.35), "open, the doorway is free");
    g.turn_cell_wheel(0, true);
    assert!(g.hits_cell_door(mx, mz, 0.35), "shut, it stops a body");
    assert!(g.circle_hits_wall(mx, mz, 0.35));
    g.turn_cell_wheel(0, false);
    assert!(!g.hits_cell_door(mx, mz, 0.35));
}

#[test]
fn her_page_sounds_the_siren() {
    for level in 1..=3 {
        let mut g = floor(4242, level);
        assert_eq!(g.assembly.stage, Stage::Quiet);
        assert!(g.assembly.t >= FIRST_QUIET.0 - 1e-9, "level {level}: a long quiet first ({:.0} s)", g.assembly.t);
        g.take_page();
        step(&mut g, PAGE_WAKE.1 + 0.5);
        assert!(g.assembly.stage == Stage::Siren, "level {level}: the siren within seconds of her page ({:?})", g.assembly.stage);
    }
}

#[test]
fn a_cell_is_in_reach_of_the_warning() {
    // the farthest any floor tile is from a cell, at a walk, inside the
    // siren's warning (before the fog reaches even the lowest walk), with
    // half as long again to spare
    let walk_s = |tiles: i32| tiles as f64 * CELL / WALK;
    assert!(walk_s(CELL_REACH) * 1.5 < WARNING, "{:.0} s to the farthest cell at a walk, {WARNING:.0} s of warning", walk_s(CELL_REACH));
    // and `CELL_REACH` follows the warning: what is walked in 0.43 of it
    // (the first pass's 16 tiles were 17.8 s of 42 at a walk; G2's 8 are
    // 8.9 s of 21)
    const SHARE: f64 = 0.43;
    assert!(
        walk_s(CELL_REACH) <= SHARE * WARNING && walk_s(CELL_REACH + 1) > SHARE * WARNING,
        "CELL_REACH {CELL_REACH} tiles ({:.1} s at a walk) should be what is walked in {SHARE} of the {WARNING} s warning ({:.1} s)",
        walk_s(CELL_REACH),
        SHARE * WARNING
    );
    for level in 1..=3 {
        for seed in [4242u32, 7, 77, 1999] {
            let (l, p) = game::base::rng::with_seed(game::base::rng::seed_for_level(seed, level), || {
                game::level::building::generate_building(level, &mut || game::base::rng::rnd())
            });
            let m = game::level::building::navigation(&l, &p);
            assert!(m.cell_reach <= CELL_REACH, "seed {seed} level {level}: a floor {} tiles from a cell", m.cell_reach);
        }
    }
}

/// The Building's floors paced: on each, six minutes standing sealed in
/// the first cell. The first siren comes within its first quiet (the first
/// floor waits longer for her camp to be found), the quiet between two is
/// in its length, and the fog comes again. A log, like `tests/pacing.rs`'s
/// (which crosses the floors).
#[test]
fn the_building_paces_a_floor() {
    for level in 1..=3 {
        let mut g = floor(4242, level);
        g.god_mode = true;
        let c = g.building.as_ref().unwrap().cells[0].tile;
        stand(&mut g, c.0, c.1);
        g.turn_cell_wheel(0, true);
        let mut spans: Vec<(Phase, f64)> = vec![(g.director.phase, 0.0)];
        let (mut first_siren, mut t) = (None, 0.0);
        for _ in 0..(360.0 / 0.5) as usize {
            step(&mut g, 0.5);
            t += 0.5;
            if g.assembly.stage == Stage::Siren && first_siren.is_none() {
                first_siren = Some(t);
            }
            match spans.last_mut() {
                Some(s) if s.0 == g.director.phase => s.1 += 0.5,
                _ => spans.push((g.director.phase, 0.5)),
            }
        }
        let text: Vec<String> = spans.iter().map(|(p, s)| format!("{} {s:.0}s", p.name())).collect();
        println!("seed 4242 L{level}: {} assemblies, first siren at {first_siren:?} s\n  {}", g.assembly.count, text.join(" → "));
        let fs = first_siren.unwrap_or(f64::INFINITY);
        let most = if level == 1 { FIRST_FLOOR_QUIET.1 } else { FIRST_QUIET.1 } + 1.0;
        assert!(fs <= most, "L{level}: the first siren within {most} s ({fs:.0} s)");
        assert!(g.assembly.count >= 3, "L{level}: it comes again ({} in 6 minutes)", g.assembly.count);
        for (p, s) in &spans[1..spans.len() - 1] {
            if *p == Phase::Calm {
                assert!(*s >= QUIET.0 - 0.5 && *s <= QUIET.1 + 0.5, "L{level}: a quiet between two of {s:.0} s");
            }
        }
        assert_eq!(g.state, "playing");
    }
}

/// Her camp on the Building's floors is a cell (the shelter there), and her
/// first four pages lie on levels 1–4 whatever the run: the Building's three
/// floors and the Lobby, where no anomaly comes.
#[test]
fn her_camp_is_a_cell_and_her_first_pages_are_the_buildings() {
    for (seed, level) in [(4242u32, 1), (4242, 2), (4242, 3), (7, 1), (7, 2), (7, 3)] {
        let g = floor(seed, level);
        let camp = g.story.camp.as_ref().expect("a camp").tile;
        assert!(g.building.as_ref().unwrap().cells.iter().any(|c| c.tile == camp), "seed {seed} L{level}: her camp {camp:?} is not a cell");
    }
    for seed in [1u32, 7, 77, 4242, 1999, 31337, 9001, 12345, 2026, 555] {
        for (n, id) in [(1, "f1"), (2, "f2"), (3, "f3"), (4, "f4")] {
            assert_eq!(game::story::notebook::page_for_level(seed, n).map(|p| p.id), Some(id), "seed {seed} level {n}");
        }
    }
}

/// The first cell that is not her camp (her page lies in that one), and
/// you standing in its middle, facing its door
fn into_a_cell(g: &mut Game) -> usize {
    let camp = g.story.camp.as_ref().map(|c| c.tile);
    let cells = g.building.as_ref().unwrap().cells.clone();
    let ci = cells.iter().position(|c| Some(c.tile) != camp).expect("a cell that is not her camp");
    let c = cells[ci].tile;
    stand(g, c.0, c.1);
    let d = g.cell_doors[ci].clone();
    let (dx, dz) = (d.at.x - g.player.pos.x, d.at.z - g.player.pos.z);
    g.player.yaw = (-dx).atan2(-dz);
    g.player.pitch = 0.0;
    ci
}

/// The fog standing at its crest `over` metres over cell `ci`'s floor for
/// `secs` more, the next knock due at once
fn fog_over(g: &mut Game, ci: usize, over: f64, secs: f64) {
    let (x, z) = g.cell_doors[ci].tile;
    let h = g.floor_y_of(x, z);
    let a = &mut g.assembly;
    a.stage = Stage::Crest;
    a.y = h + over;
    a.crest = h + over;
    a.t = secs;
    a.count = a.count.max(1);
}

/// What E does where you stand, if anything
fn prompt(g: &Game) -> Option<String> {
    g.interact_target().map(|i| g.interactables[i].prompt.clone())
}

/// Whether the verb `text` is shown where you stand, whichever way you face
/// (in its reach, and its condition true)
fn shown(g: &Game, text: &str) -> bool {
    let p = g.player.pos;
    g.interactables.iter().any(|i| i.prompt == text && i.pos.distance_to_squared(&p) < i.r * i.r && (i.enabled)(g))
}

fn logged(g: &Game, what: &str) -> usize {
    g.director.log.iter().filter(|e| e.1 == what).count()
}

/// Sealed in, the fog over the cell, until something knocks; then you
/// knock back at the door, as a player does (E, facing it). It heard you:
/// nothing of the door's is shown until its first pull
fn answer(g: &mut Game, ci: usize) {
    for _ in 0..600 {
        if g.assembly.door.visit == Visit::Knocking {
            break;
        }
        step(g, DT);
    }
    assert_eq!(g.assembly.door.visit, Visit::Knocking, "something knocks at the door");
    assert_eq!(prompt(g).as_deref(), Some("[E] KNOCK"), "while it knocks, E at the door knocks back");
    assert!(!shown(g, OPEN), "while it knocks, the door is not opened");
    g.try_interact();
    assert_eq!(g.assembly.door.visit, Visit::Trying, "answered");
    assert_eq!(g.assembly.door.cell, Some(ci));
    assert_eq!(logged(g, "cell:answered"), 1);
    for v in DOOR_VERBS {
        assert!(!shown(g, v), "answered, before the wheel turns: {v} shown");
    }
}

/// From the answer to the wheel's first pull, a frame at a time (`each`
/// before each frame, with the seconds since the answer): nothing of the
/// door's shown, nothing held, the wheel at rest; at the first pull (the
/// frame `cell:wheel` is logged) the hold shows, and nothing else of the
/// door's. The seconds it took
fn to_the_first_pull(g: &mut Game, ci: usize, mut each: impl FnMut(&mut Game, f64)) -> f64 {
    let rest = g.cell_doors[ci].turned;
    let mut t = 0.0;
    while logged(g, "cell:wheel") == 0 {
        assert!(t < WHEEL_AFTER.1 + 2.0 * DT, "the wheel tried within {:?} s of the answer ({t:.2} s)", WHEEL_AFTER);
        for v in DOOR_VERBS {
            assert!(!shown(g, v), "{t:.2} s after the answer, before its first pull: {v} shown");
        }
        let d = &g.assembly.door;
        assert_eq!((d.grip, d.play, d.give), (0.0, 0.0, 0.0), "{t:.2} s after the answer: nothing held, nothing turned");
        assert_eq!(g.cell_doors[ci].turned, rest, "{t:.2} s after the answer: the wheel at rest");
        each(g, t);
        step(g, DT);
        t += DT;
    }
    for v in DOOR_VERBS {
        assert_eq!(shown(g, v), v == HOLD, "at its first pull ({t:.2} s after the answer): {v}");
    }
    t
}

/// Silent is safe: sealed in, the fog over the cell, nobody answering.
/// Something knocks a few times (`PATIENCE`), and then the knocking stops
/// and it goes while the fog still stands, heard going once its last knock
/// has rung out (`KNOCK_RINGS`): its knock is to be answered until then,
/// and the door to open shows only once its going is over (`GOING_LEAST`
/// at the least; `MILESTONES.md` 13);
/// nothing opens, the mind holds, and it does not come back before the
/// all-clear. How many times it knocks is its own choice: the same
/// whatever the per-frame stream drew.
#[test]
fn silent_is_safe() {
    for level in 1..=3 {
        let mut patience = vec![];
        for native in [1u64, 2] {
            let mut g = floor(4242, level);
            game::base::rng::seed_native(native * 0x9E37);
            let ci = into_a_cell(&mut g);
            g.turn_cell_wheel(ci, true);
            let longest = (PATIENCE.1 + 1) as f64 * KNOCK.1 + 1.0;
            fog_over(&mut g, ci, 6.0, longest + 5.0);
            let s0 = g.player.sanity;
            let (mut knocks, mut going_in, mut gone_in, mut last_knock) = (0, None, None, 0.0);
            let mut t = 0.0;
            // through the crest, the fall and into the quiet
            for _ in 0..(120.0 / 0.1) as usize {
                step(&mut g, 0.1);
                t += 0.1;
                assert_eq!(g.assembly.sealed, Some(ci), "L{level}: nothing unseals it ({t:.1} s)");
                assert!(
                    g.cell_doors[ci].target < 0.5 && g.hits_cell_door(g.cell_doors[ci].at.x, g.cell_doors[ci].at.z, 0.35),
                    "L{level}: the door stays shut"
                );
                assert!(g.player.sanity >= s0 - 1e-9, "L{level}: sealed in, silent, the mind holds: {s0} → {}", g.player.sanity);
                assert_eq!(g.state, "playing");
                let visit = g.assembly.door.visit;
                if g.assembly.door.knocks > knocks && going_in.is_none() {
                    knocks = g.assembly.door.knocks;
                    last_knock = t;
                }
                if visit == Visit::Going && going_in.is_none() {
                    going_in = Some((t, g.assembly.stage));
                }
                if visit == Visit::Gone && gone_in.is_none() {
                    gone_in = Some(t);
                }
                if going_in.is_some() {
                    assert_eq!(g.assembly.door.knocks, knocks, "L{level}: once it goes, no more knocks");
                }
                // its knock to answer until it is heard going, and the door
                // to open only once it has gone
                assert_eq!(shown(&g, KNOCK_BACK), visit == Visit::Knocking, "L{level}: {t:.1} s in, {visit:?}: its knock to answer");
                assert_eq!(shown(&g, OPEN), visit == Visit::Gone, "L{level}: {t:.1} s in, {visit:?}: the door to open");
                if g.assembly.stage == Stage::Quiet {
                    break;
                }
            }
            let (going, stage) = going_in.expect("it went");
            let gone = gone_in.expect("its going was over");
            println!(
                "seed 4242 L{level} (native {native}): {knocks} knocks, the last {last_knock:.1} s in; heard going {going:.1} s in ({:.1} s after it), the fog at {stage:?}; gone, the door to open, {:.1} s after that",
                going - last_knock,
                gone - going
            );
            assert_eq!(stage, Stage::Crest, "L{level}: it went on its own, the fog still up");
            assert!(
                going - last_knock >= KNOCK_RINGS - 0.15,
                "L{level}: heard going once its last knock had rung out ({:.1} s)",
                going - last_knock
            );
            assert!(gone - going >= GOING_LEAST - 0.1, "L{level}: at the door until it had gone ({:.1} s)", gone - going);
            assert!((PATIENCE.0..=PATIENCE.1).contains(&knocks), "L{level}: {knocks} knocks");
            assert_eq!(g.assembly.stage, Stage::Quiet, "L{level}: the all-clear");
            assert_eq!(
                (logged(&g, "cell:knocks"), logged(&g, "cell:going"), logged(&g, "cell:gone")),
                (1, 1, 1),
                "L{level}: it came once and went once"
            );
            assert_eq!(logged(&g, "cell:wheel") + logged(&g, "cell:gave"), 0, "L{level}: nobody tried the wheel");
            patience.push(knocks);
        }
        assert_eq!(patience[0], patience[1], "L{level}: how often it knocks is its own choice, not the per-frame stream's");
    }
}

/// Its knock is to be answered until it is heard going (`MILESTONES.md`
/// 13): knocking, it decides to go (its patience out, or the fog starting
/// down) before it is heard going, and until its last knock has rung out
/// (`KNOCK_RINGS`) it is at the door as it was: `[E] KNOCK` shown, the door
/// not to be opened, and answered then, it tries the wheel. With the fog
/// starting down while it knocks, its knock is shown until it is heard
/// going, the door to open only once it has gone.
#[test]
fn its_knock_is_answerable_until_it_is_heard_going() {
    for (level, how) in [(1, "answered"), (2, "answered"), (1, "the fog falls"), (3, "the fog falls")] {
        let mut g = floor(4242, level);
        let ci = into_a_cell(&mut g);
        g.turn_cell_wheel(ci, true);
        // (the door shut, and the wheel at rest from your own turn)
        step(&mut g, 3.5);
        fog_over(&mut g, ci, 6.0, 60.0);
        let mut t = 0.0;
        while g.assembly.door.visit != Visit::Knocking {
            step(&mut g, DT);
            t += DT;
            assert!(t < 10.0, "L{level}: something knocks");
        }
        if how == "the fog falls" {
            // the crest's time is up: the fog starts down
            g.assembly.t = 0.0;
            step(&mut g, DT);
        } else {
            while g.assembly.door.leaving.is_none() {
                step(&mut g, 0.05);
                t += 0.05;
                assert!(t < 60.0, "L{level}: unanswered, it goes");
            }
        }
        let left = g.assembly.door.leaving.expect("it decided to go");
        assert_eq!(g.assembly.door.visit, Visit::Knocking, "L{level} {how}: not yet heard going");
        assert!(shown(&g, KNOCK_BACK) && !shown(&g, OPEN), "L{level} {how}: its knock to answer, the door shut to you");
        if how == "answered" {
            println!(
                "seed 4242 L{level}: its patience out after {} knocks, answered {left:.1} s before it would be heard going",
                g.assembly.door.knocks
            );
            assert_eq!(prompt(&g).as_deref(), Some(KNOCK_BACK));
            g.try_interact();
            assert_eq!(g.assembly.door.visit, Visit::Trying, "L{level}: answered while it was still there");
            to_the_first_pull(&mut g, ci, |_, _| {});
            assert_eq!(logged(&g, "cell:going"), 0, "L{level}: it did not go");
            continue;
        }
        // until it is heard going: its knock to answer, the door shut to you
        let mut heard = 0.0;
        while g.assembly.door.visit == Visit::Knocking {
            assert!(shown(&g, KNOCK_BACK) && !shown(&g, OPEN), "L{level}: {heard:.2} s after the fog turned");
            step(&mut g, DT);
            heard += DT;
            assert!(heard < KNOCK_RINGS + 0.1, "L{level}: heard going once its knock has rung out");
        }
        assert_eq!(g.assembly.door.visit, Visit::Going);
        assert!((heard - left).abs() < 2.0 * DT, "L{level}: heard going {heard:.2} s after the fog turned, {left:.2} s due");
        let mut gone = 0.0;
        while g.assembly.door.visit == Visit::Going {
            assert!(!shown(&g, KNOCK_BACK) && !shown(&g, OPEN), "L{level}: going");
            step(&mut g, DT);
            gone += DT;
        }
        println!("seed 4242 L{level}: the fog turned while it knocked; heard going {heard:.1} s after, gone {gone:.1} s after that");
        assert_eq!(g.assembly.door.visit, Visit::Gone);
        assert!(gone >= GOING_LEAST - 2.0 * DT, "L{level}: at the door until it had gone ({gone:.1} s)");
        assert!(shown(&g, OPEN), "L{level}: gone, the door to open");
    }
}

/// An answer brings the wheel turning: sealed in, the fog up, you knock
/// back at the door while it knocks; a moment later (`WHEEL_AFTER`) the
/// wheel turns from outside, its first pull taking up its play (`SLACK`),
/// then on toward giving, the door straining in its frame; the wheel is
/// yours to hold (`[E] HOLD THE WHEEL`) from its first pull, not before
/// (`to_the_first_pull`), and the door is not opened from inside while it
/// is at it.
#[test]
fn an_answer_brings_the_wheel_turning() {
    for level in 1..=3 {
        let mut g = floor(4242, level);
        g.god_mode = true;
        let ci = into_a_cell(&mut g);
        g.turn_cell_wheel(ci, true);
        // (the door shut, and the wheel at rest from your own turn)
        step(&mut g, 3.5);
        fog_over(&mut g, ci, 6.0, 40.0);
        answer(&mut g, ci);
        let before = g.cell_doors[ci].turned;
        // not yet: it heard you; then its first pull, the hold with it
        let first = to_the_first_pull(&mut g, ci, |_, _| {});
        assert!(first >= WHEEL_AFTER.0 - DT, "L{level}: a moment before it tries ({first:.2} s)");
        step(&mut g, SLACK + 0.6);
        let turned = g.cell_doors[ci].turned - before;
        println!(
            "seed 4242 L{level}: the wheel first pulled {first:.2} s after the answer; {:.2} s later it had turned {turned:.2} rad, {:.0}% of the way to giving",
            SLACK + 0.6,
            g.assembly.door.give * 100.0
        );
        assert_eq!(logged(&g, "cell:wheel"), 1, "L{level}: it tries the wheel");
        assert_eq!(g.assembly.door.play, 1.0, "L{level}: its play taken up");
        assert!(turned > 0.5, "L{level}: the wheel turns from outside ({turned:.2} rad)");
        assert!(g.assembly.door.give > 0.0 && g.assembly.door.give < 1.0, "L{level}: on its way to giving");
        assert!(g.cell_doors[ci].strain > 0.0, "L{level}: the door strains in its frame");
        assert_eq!(g.assembly.sealed, Some(ci), "L{level}: not given yet");
        assert_eq!(prompt(&g).as_deref(), Some(HOLD), "L{level}: the wheel to hold, not the door to open");
        assert!(!shown(&g, OPEN));
    }
}

/// The hold shows when the wheel turns (`MILESTONES.md` 13): answered,
/// nothing of the door's is shown until its first pull, and E there holds
/// nothing, pressed again and again (each press more than the early E's
/// grace, `INTERACT_BUFFER`, before the pull) and kept down: at the first
/// pull the wheel is not held, it turns in its play, and kept down E still
/// holds nothing (a press begins a hold), so past its play it turns toward
/// giving. Before the hold shows, nothing on the screen or in the wheel
/// tells that it is coming.
#[test]
fn the_hold_shows_when_the_wheel_turns() {
    for level in 1..=3 {
        let mut g = floor(4242, level);
        g.god_mode = true;
        let ci = into_a_cell(&mut g);
        g.turn_cell_wheel(ci, true);
        step(&mut g, 3.5);
        fog_over(&mut g, ci, 6.0, 40.0);
        answer(&mut g, ci);
        let wait = g.assembly.door.wait;
        g.keys.insert("KeyE".into());
        let mut presses = 0;
        let first = to_the_first_pull(&mut g, ci, |g, t| {
            if ((t / DT).round() as usize) % 9 == 0 && t + INTERACT_BUFFER + 2.0 * DT < wait {
                g.on_action(Act::Interact);
                presses += 1;
            }
        });
        let d = g.assembly.door.clone();
        println!("seed 4242 L{level}: E pressed {presses} times and kept down from the answer; the first pull {first:.2} s after it, unheld (its play {:.0}% taken up)", d.play * 100.0);
        assert!(presses >= 4, "L{level}: E pressed in the gap ({presses})");
        assert_eq!((d.grip, d.held_for), (0.0, 0.0), "L{level}: E before the pull holds nothing");
        assert!(d.play > 0.0 && d.give == 0.0, "L{level}: its first pull turns the wheel in its play");
        step(&mut g, SLACK + 0.3);
        let d = &g.assembly.door;
        assert_eq!(d.held_for, 0.0, "L{level}: E kept down holds nothing: a press begins a hold");
        assert!(d.give > 0.0, "L{level}: let be past its play, it turns toward giving ({:.3})", d.give);
        g.keys.remove("KeyE");
    }
}

/// Held is safe: answered, and then the wheel held from its first pull (E
/// pressed on it a player's reaction after the hold shows, `REACTION`, and
/// kept down, or pressed again and again): within its play (`SLACK`), so
/// it strains and never turns toward giving, the door stays sealed through
/// the fog, the mind holds, and after a while (`TRIES`) it lets go and goes,
/// heard going: the door to open shows only once it has gone.
#[test]
fn held_is_safe() {
    for (level, how) in [(1, "held"), (2, "held"), (3, "held"), (1, "pressed")] {
        let mut g = floor(4242, level);
        let ci = into_a_cell(&mut g);
        g.turn_cell_wheel(ci, true);
        // (the door shut, and the wheel at rest from your own turn)
        step(&mut g, 3.5);
        let rest = g.cell_doors[ci].turned;
        fog_over(&mut g, ci, 6.0, TRIES.1 + WHEEL_AFTER.1 + 6.0);
        answer(&mut g, ci);
        let first = to_the_first_pull(&mut g, ci, |_, _| {});
        // its squeal, the wheel moving, the prompt: a player's reaction, then E
        step(&mut g, REACTION);
        assert_eq!(prompt(&g).as_deref(), Some(HOLD));
        let play = g.assembly.door.play;
        g.try_interact();
        if how == "held" {
            g.keys.insert("KeyE".into());
        }
        let s0 = g.player.sanity;
        let (mut t, mut going, mut gone, mut most) = (0.0, None, None, 0.0f64);
        for k in 0..(120.0 / 0.1) as usize {
            // (pressed: E again every 0.4 s while the wheel is to hold)
            if how == "pressed" && k % 4 == 3 && prompt(&g).as_deref() == Some(HOLD) {
                g.try_interact();
            }
            step(&mut g, 0.1);
            t += 0.1;
            assert_eq!(g.assembly.sealed, Some(ci), "L{level}: held, the door stays sealed ({t:.1} s)");
            assert!(g.cell_doors[ci].target < 0.5, "L{level}: held, the door stays shut");
            assert!(g.player.sanity >= s0 - 1e-9, "L{level}: held, the fog stays out: {s0} → {}", g.player.sanity);
            assert_eq!(g.state, "playing");
            most = most.max(g.cell_doors[ci].turned - rest);
            let visit = g.assembly.door.visit;
            if visit == Visit::Going && going.is_none() {
                going = Some(t);
            }
            if visit == Visit::Gone && gone.is_none() {
                gone = Some(t);
            }
            assert_eq!(shown(&g, HOLD), visit == Visit::Trying, "L{level}: {t:.1} s, {visit:?}: the wheel to hold");
            assert_eq!(shown(&g, OPEN), visit == Visit::Gone, "L{level}: {t:.1} s, {visit:?}: the door to open");
            if g.assembly.stage == Stage::Quiet {
                break;
            }
        }
        let d = &g.assembly.door;
        let (going, gone) = (going.expect("it let go"), gone.expect("its going was over"));
        println!(
            "seed 4242 L{level} ({how}): the hold shown {first:.2} s after the answer, E {REACTION} s after that (its play {:.0}% taken up); held {:.1} s, it let go after {:.1} s at the wheel, gone {:.1} s after; the wheel never {:.3} rad from rest",
            play * 100.0,
            d.held_for,
            d.t,
            gone - going,
            most
        );
        assert_eq!(g.assembly.stage, Stage::Quiet, "L{level}: through to the all-clear");
        assert_eq!(logged(&g, "cell:wheel"), 1, "L{level}: it tried the wheel");
        assert_eq!(logged(&g, "cell:gave"), 0, "L{level}: held, it did not give");
        assert_eq!((logged(&g, "cell:going"), logged(&g, "cell:gone")), (1, 1), "L{level}: it let go and went");
        assert!(play < 1.0, "L{level}: a reaction of {REACTION} s is within its play");
        assert_eq!(d.give, 0.0, "L{level}: held from the first pull, it never turned toward giving");
        assert!(d.held_for >= TRIES.0 - REACTION - 0.2, "L{level}: held all the while it tried ({:.1} s)", d.held_for);
        assert!(most <= PLAY + 1e-9, "L{level}: the wheel never past its play ({most:.3} rad)");
        assert!(gone - going >= GOING_LEAST - 0.1, "L{level}: at the door until it had gone ({:.1} s)", gone - going);
        assert_eq!(g.cell_doors[ci].turned, rest, "L{level}: the wheel back at rest");
        g.keys.remove("KeyE");
    }
}

/// A visit that does not give leaves the wheel shut: answered, the wheel
/// turned part of the way from outside, and then the visit ends without
/// the door giving (held until it lets go, or the fog going down while it
/// tries): the wheel turns back to where it rests on a shut door, not left
/// partly open on it for the rest of the floor (code review 2). Shut and
/// opened again after the all-clear, it rests where an open door's does.
#[test]
fn a_visit_that_does_not_give_leaves_the_wheel_shut() {
    for (level, how) in [(1, "held"), (2, "held"), (3, "held"), (1, "the fog falls"), (2, "the fog falls")] {
        let mut g = floor(4242, level);
        let ci = into_a_cell(&mut g);
        g.turn_cell_wheel(ci, true);
        // (the door shut, and the wheel at rest from your own turn)
        step(&mut g, 3.5);
        assert_eq!(g.cell_doors[ci].turned, WHEEL_SHUT, "L{level}: shut, the wheel rests shut");
        fog_over(&mut g, ci, 6.0, TRIES.1 + WHEEL_AFTER.1 + 6.0);
        answer(&mut g, ci);
        // not held: it turns the wheel part of the way (its play, and less
        // than GIVE s more)
        step(&mut g, WHEEL_AFTER.1 + SLACK + 0.6);
        let part = g.assembly.door.give;
        assert!(part > 0.1 && part < 1.0, "L{level} {how}: on its way to giving ({part:.2})");
        assert!(g.cell_doors[ci].turned > WHEEL_SHUT + 0.5, "L{level} {how}: the wheel turned from outside");
        if how == "held" {
            assert_eq!(prompt(&g).as_deref(), Some(HOLD));
            g.try_interact();
            g.keys.insert("KeyE".into());
        } else {
            // the crest's time is up: the fog goes down
            g.assembly.t = 0.0;
        }
        // (it lets go and goes, heard going: the door to open only once it
        // has gone)
        let mut going = 0.0;
        for _ in 0..((TRIES.1 + 8.0) / 0.1) as usize {
            step(&mut g, 0.1);
            let visit = g.assembly.door.visit;
            if visit == Visit::Going {
                going += 0.1;
            }
            assert_eq!(shown(&g, OPEN), visit == Visit::Gone, "L{level} {how}: {visit:?}: the door to open");
            if visit == Visit::Gone {
                break;
            }
        }
        g.keys.remove("KeyE");
        assert_eq!(g.assembly.door.visit, Visit::Gone, "L{level} {how}: it went");
        assert!(going >= GOING_LEAST - 0.15, "L{level} {how}: heard going until it had gone ({going:.1} s)");
        assert_eq!(logged(&g, "cell:going"), 1, "L{level} {how}: it went once");
        assert_eq!(logged(&g, "cell:gave"), 0, "L{level} {how}: the door did not give");
        assert_eq!(g.assembly.door.give, part, "L{level} {how}: held, it turned no further");
        // (the wheel eases back)
        step(&mut g, 3.0);
        let d = &g.cell_doors[ci];
        println!(
            "seed 4242 L{level} ({how}): turned {:.0}% of the way to giving, then it went; the wheel at {:.3} rad",
            part * 100.0,
            d.turned
        );
        assert!(d.target < 0.5, "L{level} {how}: the door still shut");
        assert_eq!((d.turn_to, d.turned), (WHEEL_SHUT, WHEEL_SHUT), "L{level} {how}: the wheel back where it rests shut");
        if how == "the fog falls" {
            // through to the all-clear, then opened: it rests open
            for _ in 0..1200 {
                if g.assembly.stage == Stage::Quiet {
                    break;
                }
                step(&mut g, 0.1);
            }
            assert_eq!(g.assembly.stage, Stage::Quiet, "L{level}: the all-clear");
            g.turn_cell_wheel(ci, false);
            step(&mut g, 3.5);
            assert_eq!(g.cell_doors[ci].turned, WHEEL_OPEN, "L{level}: opened, the wheel rests open");
        }
    }
}

/// Let go is the fog: answered and not held, the wheel turns, and the door
/// gives (its play, `SLACK`, and `GIVE` seconds of turning past it): it
/// swings in, the cell is no longer
/// sealed and will not seal again while the fog is up, and the fog's harm
/// reaches you inside, as it does outside: your mind goes at `DRAIN`, and
/// you are gone.
#[test]
fn let_go_is_the_fog() {
    for level in 1..=3 {
        let mut g = floor(4242, level);
        let ci = into_a_cell(&mut g);
        g.turn_cell_wheel(ci, true);
        // (the door shut, and the wheel at rest from your own turn)
        step(&mut g, 3.5);
        fog_over(&mut g, ci, 6.0, 40.0);
        answer(&mut g, ci);
        let s0 = g.player.sanity;
        let mut gave = None;
        let mut t = 0.0;
        while t < WHEEL_AFTER.1 + SLACK + GIVE + 1.0 && gave.is_none() {
            step(&mut g, DT);
            t += DT;
            if g.assembly.door.visit == Visit::Gave {
                gave = Some(t);
            } else {
                assert!(g.player.sanity >= s0 - 1e-9, "L{level}: before it gives, the fog stays out");
            }
        }
        let gave = gave.expect("the door gave");
        println!("seed 4242 L{level}: the door gave {gave:.2} s after the answer");
        assert!(gave >= WHEEL_AFTER.0 + SLACK + GIVE - 0.05, "L{level}: it gives after its play and {GIVE} s of turning ({gave:.2} s)");
        assert_eq!(logged(&g, "cell:gave"), 1);
        assert_eq!(g.assembly.sealed, None, "L{level}: no longer sealed");
        assert!((g.cell_doors[ci].target - GAVE_OPEN).abs() < 1e-9, "L{level}: the door swings in");
        let d = g.cell_doors[ci].clone();
        assert!(!g.hits_cell_door(d.at.x, d.at.z, 0.35), "L{level}: the doorway is open");
        // its seal is broken: the wheel will not shut it while the fog is up
        assert!(g.cell_door_gave(ci));
        assert_ne!(prompt(&g).as_deref(), Some("[E] TURN THE WHEEL"), "L{level}: the wheel will not shut it");
        // the fog's harm reaches you inside, as outside: the mind goes at
        // DRAIN (less the little it mends of itself, as anywhere)
        let s1 = g.player.sanity;
        step(&mut g, 0.5);
        let lost = s1 - g.player.sanity;
        let want = (DRAIN - SAN_REGEN) * 0.5;
        assert!(
            (lost - want).abs() < 0.05 * want,
            "L{level}: inside, the mind goes as it does out in the fog ({lost:.2} in 0.5 s, {want:.2} out in it)"
        );
        step(&mut g, LAST);
        assert_eq!(g.state, "dead", "L{level}: let go, the fog");
        assert_eq!(g.last_death_cause.as_deref(), Some("assembly"));
    }
}

/// An unsealed cell lets the fog in: a cell whose door was never shut, and
/// one opened while the fog is up, are as the walk outside: with the fog
/// over you, the mind goes at the same rate as outside, and you are gone.
#[test]
fn an_unsealed_cell_lets_the_fog_in() {
    for level in 1..=3 {
        let mut drains = vec![];
        for how in ["outside", "never shut", "opened"] {
            let mut g = floor(4242, level);
            let ci = into_a_cell(&mut g);
            if how == "outside" {
                // on the walk before its door
                let d = g.cell_doors[ci].clone();
                let (cx, cz) = (g.wx(d.tile.0), g.wz(d.tile.1));
                let (ox, oz) = (d.at.x + (d.at.x - cx) / 2.0 * 0.7, d.at.z + (d.at.z - cz) / 2.0 * 0.7);
                g.player.pos = engine::math::Vec3::new(ox, g.floor_y_at(ox, oz) + EYE, oz);
                assert_ne!((g.tx(ox), g.tz(oz)), d.tile, "L{level}: outside the cell");
            }
            if how == "opened" {
                g.turn_cell_wheel(ci, true);
                step(&mut g, 2.0);
            }
            fog_over(&mut g, ci, 6.0, 40.0);
            // (the fog as far over the walk as over the cell's floor)
            let p = g.player.pos;
            g.assembly.y = g.floor_y_at(p.x, p.z) + 6.0;
            g.assembly.crest = g.assembly.y;
            if how == "opened" {
                step(&mut g, 1.0);
                assert_eq!(g.player.sanity, 100.0, "L{level}: sealed, the fog stays out");
                g.turn_cell_wheel(ci, false);
            }
            let s0 = g.player.sanity;
            step(&mut g, 0.5);
            drains.push((how, s0 - g.player.sanity));
            step(&mut g, LAST);
            assert_eq!(g.state, "dead", "L{level} {how}: the fog comes in");
            assert_eq!(g.last_death_cause.as_deref(), Some("assembly"));
        }
        println!("seed 4242 L{level}: the mind lost in 0.5 s {drains:?}");
        let out = drains[0].1;
        assert!(out > 0.0);
        for (how, lost) in &drains[1..] {
            assert!((lost - out).abs() < 1e-6, "L{level}: {how}, the fog's harm as outside ({lost:.2} against {out:.2})");
        }
    }
}
