//! Arenas (`docs/design/ENTITIES.md`, design 7): does each creature's counter
//! work, and does it matter? A room with pillars, a creature roaming it, and
//! the director sending it after you as on a level, and
//! you touring the room (its corners and its middle), stopping now and then
//! to look round as an explorer does, 100 times with the creature's
//! counter and 100 times without. Any contact (a knock-down or a death)
//! counts as a failure.
//!
//! - with the counter, at least 95% go round untouched, and 90% get round;
//! - without it, at most 70% go untouched (the counter matters) and at most
//!   60% are killed (the creature is not a death sentence: the first meeting
//!   knocks you down, a death needs a second contact in a peak).
//!
//!   cargo test --release -p game --test creatures_arenas -- --nocapture
//!
//! And once it has learned (design 4: its one new behaviour, met before and
//! already shown): the counter must still work as well as before (no
//! learned behaviour breaks its creature's rule), and the creature must
//! have done what it learned in most trials. The Hound learns in its
//! charge, which its counter seldom lets happen: its arena has you heard
//! first (walking on, minding nothing, until it comes), and is held only
//! to the comparison.
//!
//! To see why a counter fails: `debug_trials` lists the seeds where the
//! counter was touched, and `ARENA_DEBUG=kind:true:seed` prints that trial's
//! mode changes (`ARENA_LEARNED=1` for the learned creature, the Hound heard
//! first):
//!
//!   ARENA_KIND=smiler cargo test --release -p game --test creatures_arenas debug_trials -- --ignored --nocapture
mod arena;
use arena::*;
use game::base::numeric::hypot2;
use game::base::rng::seed_native;
use game::entities::brain::Mode;
use game::entities::EntityNav;
use game::prelude::*;

const DT: f64 = 1.0 / 30.0;
const LIMIT: f64 = 600.0;
const SEEDS: u64 = 100;

/// How a trial is played: with the creature's counter or not; against a
/// creature that has learned (design 4); and, for the Hound, heard first
/// (you walk on, minding nothing, until it first comes after you: with
/// its counter it seldom does, and what it learns is in its charge).
#[derive(Clone, Copy)]
struct Setup {
    counter: bool,
    learned: bool,
    heard_first: bool,
}
const PLAIN: Setup = Setup { counter: true, learned: false, heard_first: false };

#[derive(Debug, Default, Clone, Copy)]
struct Outcome {
    contacts: u32,
    killed: bool,
    reached: bool,
    secs: f64,
    /// how many times it did what it learned (design 4)
    uses: u32,
}

/// How many times creature `ei` did what its kind learns (each leaves a mark:
/// a mode change's reason, a note to the director, or the Hound's lead).
fn learned_uses(g: &Game, ei: usize) -> u32 {
    let c = g.creature(ei).unwrap();
    let reasons = |why: &str| c.brain.log.iter().filter(|l| l.3 == why).count() as u32;
    let notes = |what: &str| g.director.log.iter().filter(|(_, w)| *w == what).count() as u32;
    match g.entities[ei].kind.as_str() {
        "smiler" => reasons("it waits where you went dark"),
        "hound" => (c.get("ahead") > 0.0) as u32,
        "watcher" => notes("lights fail"),
        "corner" => reasons("it turns to look behind it"),
        "reader" => notes("tear"),
        _ => 0,
    }
}

/// What you can tell of the creature without seeing it: where its sound
/// comes from, whether it is moving (its steps) or still (listening), and
/// whether it has bayed (it is after you).
struct Heard {
    d: f64,
    moving: bool,
    hunting: bool,
    /// it growls, crouched close (a spring is coming)
    growling: bool,
    /// you have heard it stir (the Smiler's click, hiss or breath), or seen it
    stirred: bool,
}

fn heard(g: &Game, ei: usize) -> Heard {
    let e = &g.entities[ei];
    let p = e.pos();
    let d = hypot2(p.x - g.player.pos.x, p.z - g.player.pos.z);
    let m = mode(g, ei);
    // what you see of it: its eyes give your lamp back when your beam finds
    // it facing you; under a working panel, a tall shape
    let seen = {
        let pc = g.perceive(ei);
        let f = g.player.fwd();
        let (ux, uz) = ((p.x - g.player.pos.x) / d.max(1e-6), (p.z - g.player.pos.z) / d.max(1e-6));
        let toward = f.x * ux + f.z * uz;
        let faces = -(e.yaw.sin() * ux + e.yaw.cos() * uz) > 0.3;
        let eyes = g.player.flashlight_on && toward > 0.86 && d < 20.0 && faces && pc.line;
        let shape = toward > 0.5 && d < 20.0 && pc.line && pc.it_lit > 0.35;
        eyes || shape
    };
    Heard {
        d,
        moving: hypot2(e.vx, e.vz) > game::entities::hound::MOVING,
        hunting: matches!(m, Mode::Hunt | Mode::Strike),
        growling: m == Mode::Strike,
        // the Smiler: its clicks (alert) and hiss carry 16 m; close by,
        // its breathing (it is silent only waiting, further off)
        stirred: (matches!(m, Mode::Alert | Mode::Hunt | Mode::Strike) && d < 16.0)
            || d < 6.0
            || seen
            || (e.kind == "watcher" && g.path_distance(p.x, p.z, d) < game::entities::watcher::RING_REACH)
            // the one in the corner: its hum has stopped
            || (e.kind == "corner" && m == Mode::Alert && d < 16.0),
    }
}

fn pillar(x: i32, z: i32) -> bool {
    x % 4 == 0 && z % 4 == 0 && x > 1 && z > 1 && x < 19 && z < 19
}

/// The Smiler's room's light: dark (0.08), with three bright patches under
/// working panels (0.8: its middle and two corners), as its regions are on
/// a level.
fn smiler_light(x: i32, z: i32) -> f64 {
    let bright = ((9..=11).contains(&x) && (9..=11).contains(&z))
        || ((15..=19).contains(&x) && (1..=4).contains(&z))
        || ((1..=4).contains(&x) && (15..=19).contains(&z));
    if bright {
        0.8
    } else {
        0.08
    }
}

/// A 21 × 21 room (84 m) with pillars, lit for `kind`: the Hound's evenly,
/// the Smiler's by `smiler_light`.
fn pillared(kind: &str) -> Game {
    match kind {
        "smiler" => room_lit(21, smiler_light, pillar),
        _ => room(21, 0.5, pillar),
    }
}

/// Where to slip away to from creature `ei`, as a player who can see the
/// room picks it: a tile a few steps off that you reach well before it
/// does, the darker the better.
fn escape(g: &Game, ei: usize) -> (f64, f64) {
    let nav = GridNav { lit: vec![] };
    let me = nav.bfs(g, g.tx(g.player.pos.x), g.tz(g.player.pos.z));
    let p = g.entities[ei].pos();
    let it = nav.bfs(g, g.tx(p.x), g.tz(p.z));
    let mut best = (g.player.pos.x, g.player.pos.z);
    let mut top = f64::MIN;
    for z in 0..g.h {
        for x in 0..g.w {
            let i = g.t_idx(x, z);
            if !(1..=5).contains(&me[i]) || it[i] < 0 {
                continue;
            }
            let s = (it[i] - me[i]) as f64 - 4.0 * smiler_light(x, z) + me[i] as f64 * 0.1;
            if s > top {
                top = s;
                best = (g.wx(x), g.wz(z));
            }
        }
    }
    best
}

fn trial(kind: &str, setup: Setup, seed: u64) -> Outcome {
    let Setup { counter, learned, heard_first } = setup;
    seed_native(seed);
    // the one in the corner stands in a corner of a smaller lit room your
    // way keeps crossing; the others roam the pillared room
    let small = kind == "corner";
    let mut g = if small { room(6, 0.5, |_, _| false) } else { pillared(kind) };
    let mut r = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    let mut next = || {
        r ^= r << 13;
        r ^= r >> 7;
        r ^= r << 17;
        (r % 1000) as f64 / 1000.0
    };
    // it starts somewhere about the middle of the room, facing anywhere
    let (cx, cz) = (7 + (next() * 7.0) as i32, 7 + (next() * 7.0) as i32);
    let yaw0 = next() * PI * 2.0;
    let ei = if small {
        let ei = creature(&mut g, kind, 1, 1, 0.0);
        game::entities::corner::stand_in(&mut g, ei, 1, 1, -1, -1);
        ei
    } else {
        creature(&mut g, kind, cx, cz, yaw0)
    };
    if learned {
        // you have met it, it has learned, and you have seen it do it once
        let run = &mut g.creature_run;
        run.met.push(kind.to_string());
        run.learned.push(kind.to_string());
        run.shown.push(kind.to_string());
    }
    g.player.pos = if small { v3(g.wx(4), 0.0, g.wz(4)) } else { v3(g.wx(2), 0.0, g.wz(2)) };
    // a tour of the room's corners, as you would explore a level (in the
    // small room, round it and across it, never into its corner)
    // (in the small room five times round, a minute and a half: as long as
    // you would be in it)
    let tour: Vec<(f64, f64)> = if small {
        let lap = [(4, 1), (3, 3), (1, 4), (4, 4), (3, 2), (4, 1)];
        lap.iter().cycle().take(lap.len() * 5).map(|&(x, z)| (g.wx(x), g.wz(z))).collect()
    } else {
        vec![(g.wx(18), g.wz(2)), (g.wx(10), g.wz(10)), (g.wx(18), g.wz(18)), (g.wx(2), g.wz(18)), (g.wx(10), g.wz(10)), (g.wx(2), g.wz(2))]
    };
    let mut leg = 0;
    // the director sends it, as on a level: its quiet cut short
    g.reset_director();
    g.director.t = 3.0;
    let mut out = Outcome::default();
    let mut t = 0.0;
    let mut stamina = STAM_MAX;
    // an explorer stops now and then to look round (both bots alike)
    let mut walk_left = 6.0 + next() * 4.0;
    let mut look_left = 0.0;
    // your lamp: on in the dark, as anyone's (the Smiler's counter turns it
    // off for a while when it hears it stir)
    let mut wary = 0.0;
    // the one in the corner: once its hum has stopped, your eyes stay away
    // a while (a player who heard it stop does not turn to look a step
    // later, out of the hum's reach, as its head comes round)
    let mut eyes_away = 0.0;
    let mut hunted = false;
    let mut glance = 0.0;
    let mut out_since = 0.0;
    // the paper in your pockets (a pile's worth), and when you last dropped some
    g.player.papers = 8;
    let mut dropped = -10.0;
    while t < LIMIT {
        let goal = tour[leg];
        let h = heard(&g, ei);
        if h.stirred {
            wary = 15.0;
            eyes_away = 2.0 * game::entities::corner::TURN;
        }
        wary = f64::max(wary - DT, 0.0);
        eyes_away = f64::max(eyes_away - DT, 0.0);
        g.player.flashlight_on = !(counter && kind == "smiler" && wary > 0.0);
        let gait = if !counter {
            // the naive: walk; when it comes (as far as you can tell: the
            // Watcher gives no sign but a ring close by), run for it
            if h.hunting && kind != "watcher" {
                Gait::Run
            } else {
                Gait::Walk
            }
        } else {
            match kind {
                "hound" => hound_counter(&h, stamina),
                "smiler" => smiler_counter(&h, g.perceive(ei).you_lit),
                // walk on, never run: what holds it is your eyes
                "watcher" => Gait::Walk,
                // walk on, away from it: paper stops it, not your legs
                "reader" => Gait::Walk,
                // walk on; if it comes after all, run
                "corner" => {
                    if h.hunting {
                        Gait::Run
                    } else {
                        Gait::Walk
                    }
                }
                other => panic!("no counter for {other}"),
            }
        };
        // heard first: you walk on, minding nothing, until it first comes
        // after you (your breath kept: what meets its charge is your sprint)
        if h.hunting {
            hunted = true;
        }
        let gait = if heard_first && !hunted { Gait::Walk } else { gait };
        // (only when nothing is about, as far as you know)
        let calm = gait == Gait::Walk && !h.hunting && !(counter && h.stirred);
        let gait = if look_left > 0.0 && calm {
            look_left -= DT;
            // looking round: the view sweeps
            g.player.yaw += 0.9 * DT;
            Gait::Still
        } else {
            if gait != Gait::Still {
                walk_left -= DT;
            }
            if walk_left <= 0.0 {
                walk_left = 6.0 + next() * 4.0;
                look_left = 3.0 + next() * 3.0;
            }
            gait
        };
        // your nerve comes back, as it does outside a dark zone (`player.rs`)
        g.player.sanity = (g.player.sanity + SAN_REGEN * DT).min(SAN_MAX);
        // a sprint costs breath, as yours does (`player.rs`)
        let gait = if gait == Gait::Run && stamina <= 0.0 { Gait::Walk } else { gait };
        stamina = if gait == Gait::Run { (stamina - STAM_DRAIN * DT).max(0.0) } else { (stamina + STAM_REGEN * DT).min(STAM_MAX) };
        // backing away: straight away from it; otherwise on toward the goal
        let away = {
            let p = g.entities[ei].pos();
            let (dx, dz) = (g.player.pos.x - p.x, g.player.pos.z - p.z);
            let l = hypot2(dx, dz).max(1e-6);
            let (ux, uz) = (dx / l, dz / l);
            (g.player.pos.x + ux * 3.0, g.player.pos.z + uz * 3.0)
        };
        let toward = if gait == Gait::Still {
            None
        } else if counter && kind == "smiler" && (h.hunting || (h.stirred && h.d < 8.0)) {
            Some(escape(&g, ei))
        } else if counter && kind == "hound" && h.hunting {
            // its charge (slower than your sprint): to where you get well
            // before it does (sidestepping it, it ran you down; learned, it
            // runs for where you are heading)
            Some(escape(&g, ei))
        } else if counter && kind == "hound" && h.d < 6.0 {
            Some(away)
        } else if counter && kind == "reader" && h.hunting {
            Some(away)
        } else {
            Some(goal)
        };
        let before = mode(&g, ei);
        // where you look: the Watcher's counter keeps it in sight once it
        // is about (its ring, or seen), and glances back now and then
        glance -= DT;
        if glance < -2.5 {
            glance = 0.5;
        }
        let face = if counter && kind == "watcher" && (h.stirred || glance > 0.0) {
            let p = g.entities[ei].pos();
            Some((p.x, p.z))
        } else if counter && kind == "corner" && (h.stirred || h.hunting || eyes_away > 0.0) {
            // the hum has stopped: look anywhere but at it
            let p = g.entities[ei].pos();
            let pp = g.player.pos;
            Some((2.0 * pp.x - p.x, 2.0 * pp.z - p.z))
        } else {
            None
        };
        // the Reader's counter: it comes, so a sheet goes down behind you
        if counter && kind == "reader" && h.hunting && h.d < 9.0 && t - dropped > 2.0 && g.player.papers > 0 {
            let (ax, az) = away;
            look_at(&mut g, ax, az);
            g.drop_paper_marker();
            dropped = t;
        }
        step_facing(&mut g, DT, toward, gait, face);
        g.update_director(DT);
        t += DT;
        let debug = std::env::var("ARENA_DEBUG").map(|v| v == format!("{kind}:{counter}:{seed}")).unwrap_or(false);
        if debug && (t / 2.0).floor() != ((t - DT) / 2.0).floor() {
            let c = g.creature(ei).unwrap();
            println!(
                "{t:6.1}   ({:?} d {:.1} gait {gait:?} stamina {stamina:.0} tells {} strike {} papers {})",
                mode(&g, ei),
                h.d,
                c.tells.len(),
                g.strike_allowed(ei),
                g.player.papers
            );
        }
        if debug && mode(&g, ei) != before {
            let p = g.perceive(ei);
            println!(
                "{t:6.1} {:?} -> {:?} d {:.1} lamp {} gait {gait:?} lit {:.2} it_lit {:.2} aw {:.2} wary {wary:.1}",
                before,
                mode(&g, ei),
                h.d,
                g.player.flashlight_on,
                p.you_lit,
                p.it_lit,
                g.creature(ei).unwrap().brain.mem.awareness
            );
        }
        // a contact (the director logs each)
        out.contacts = g.director.log.iter().filter(|(_, w)| *w == "contact").count() as u32;
        if !out.reached && hypot2(g.player.pos.x - goal.0, g.player.pos.z - goal.1) < 2.0 {
            leg += 1;
            if leg == tour.len() {
                out.reached = true;
                leg -= 1;
            }
        }
        // round, but not while something is still after you: the chase
        // goes on (up to 30 s) and its end counts
        if out.reached {
            let after_you = matches!(mode(&g, ei), Mode::Hunt | Mode::Strike);
            if !after_you || t > out_since + 30.0 {
                break;
            }
        } else {
            out_since = t;
        }
    }
    out.secs = if out.reached { out_since } else { t };
    out.killed = g.god_deaths > 0.0;
    out.uses = learned_uses(&g, ei);
    out
}

/// The Hound's counter, as a player who knows it plays it: move when it
/// moves, freeze when it listens, keep out of a charge's line.
fn hound_counter(h: &Heard, stamina: f64) -> Gait {
    if h.growling || (h.hunting && h.moving && h.d < 10.0) {
        // a spring coming, or a charge at you: get out of its line
        if stamina > 25.0 {
            Gait::Run
        } else {
            Gait::Walk
        }
    } else if !h.moving {
        // it has stopped to listen: within a crouch's reach of its
        // ears freeze; within a step's, creep away; else walk
        if h.d < 7.5 {
            Gait::Still
        } else if h.d < 14.0 {
            Gait::Crouch
        } else {
            Gait::Walk
        }
    } else if h.d < 4.5 {
        Gait::Still
    } else if h.d < 8.5 {
        Gait::Crouch
    } else {
        Gait::Walk
    }
}

/// The Smiler's counter, as a player who knows it: lamp off once you hear
/// it (above); never run near it; when it comes, keep still in the dark,
/// and creep (crouched, never running) out of a bright patch or away from
/// it when it is all but on you.
fn smiler_counter(h: &Heard, light_here: f64) -> Gait {
    if h.hunting {
        if light_here > 0.35 {
            // in a bright patch it sees you: walk (never run) out of it
            Gait::Walk
        } else if h.d < 1.2 {
            Gait::Crouch
        } else {
            Gait::Still
        }
    } else if h.stirred && h.d < 8.0 && light_here > 0.35 {
        // it is about and you stand in the light: walk out of it
        Gait::Walk
    } else if h.stirred && h.d < 4.0 {
        // it is about, very close: keep still
        Gait::Still
    } else if h.stirred && h.d < 8.0 {
        // close: creep away from it
        Gait::Crouch
    } else {
        Gait::Walk
    }
}

fn sweep(kind: &str, setup: Setup) -> Sweep {
    let mut w = Sweep::default();
    for s in 0..SEEDS {
        let o = trial(kind, setup, 1000 + s);
        if o.contacts == 0 {
            w.untouched += 1.0;
        }
        if o.killed {
            w.killed += 1.0;
        }
        if o.reached {
            w.across += 1.0;
            w.secs += o.secs;
        }
        if o.uses > 0 {
            w.used += 1.0;
        }
        w.uses += o.uses;
    }
    if w.across > 0.0 {
        w.secs /= w.across;
    }
    let n = SEEDS as f64;
    w.untouched /= n;
    w.killed /= n;
    w.across /= n;
    w.used /= n;
    w
}

#[derive(Debug, Default)]
struct Sweep {
    untouched: f64,
    killed: f64,
    across: f64,
    secs: f64,
    /// the share of trials where it did what it learned, and how often in all
    used: f64,
    uses: u32,
}

fn report(kind: &str) -> (Sweep, Sweep) {
    let with = sweep(kind, PLAIN);
    let without = sweep(kind, Setup { counter: false, ..PLAIN });
    println!(
        "{kind}: with the counter {:.0}% untouched, {:.0}% killed, {:.0}% round ({:.0} s); without {:.0}% untouched, {:.0}% killed, {:.0}% round ({:.0} s)",
        with.untouched * 100.0,
        with.killed * 100.0,
        with.across * 100.0,
        with.secs,
        without.untouched * 100.0,
        without.killed * 100.0,
        without.across * 100.0,
        without.secs
    );
    (with, without)
}

/// The bands every creature's arena is held to (`docs/design/ENTITIES.md`, design 7).
fn hold_to_bands(kind: &str, with: &Sweep, without: &Sweep) {
    assert!(with.untouched >= 0.95, "{kind}: with its counter only {:.0}% went untouched", with.untouched * 100.0);
    assert!(with.across >= 0.90, "{kind}: with its counter only {:.0}% got round", with.across * 100.0);
    assert!(
        without.untouched <= 0.70,
        "{kind}: without its counter {:.0}% went untouched: the counter does not matter",
        without.untouched * 100.0
    );
    assert!(without.killed <= 0.60, "{kind}: without its counter {:.0}% were killed: a death sentence", without.killed * 100.0);
}

#[test]
fn the_smilers_counter_works_and_matters() {
    let (with, without) = report("smiler");
    hold_to_bands("smiler", &with, &without);
}

#[test]
fn the_watchers_counter_works_and_matters() {
    let (with, without) = report("watcher");
    hold_to_bands("watcher", &with, &without);
}

#[test]
fn the_readers_counter_works_and_matters() {
    let (with, without) = report("reader");
    hold_to_bands("reader", &with, &without);
}

#[test]
fn the_ones_in_the_corner_counter_works_and_matters() {
    let (with, without) = report("corner");
    hold_to_bands("corner", &with, &without);
}

#[test]
fn the_hounds_counter_works_and_matters() {
    let (with, without) = report("hound");
    hold_to_bands("hound", &with, &without);
}

/// Once it has learned, its counter still works as well (design 4).
fn hold_when_learned(kind: &str) {
    let heard_first = kind == "hound";
    let with = sweep(kind, Setup { learned: true, heard_first, ..PLAIN });
    let before = sweep(kind, Setup { heard_first, ..PLAIN });
    println!(
        "{kind}, learned{}: with the counter {:.0}% untouched, {:.0}% killed, {:.0}% round ({:.0} s); it did what it learned in {:.0}% of the trials ({} times; {} before it learned); before it learned {:.0}% untouched",
        if heard_first { ", heard first" } else { "" },
        with.untouched * 100.0,
        with.killed * 100.0,
        with.across * 100.0,
        with.secs,
        with.used * 100.0,
        with.uses,
        before.uses,
        before.untouched * 100.0
    );
    // (it must have done it: an arena it never learned in shows nothing)
    assert!(with.used >= 0.5, "{kind}, learned: it did what it learned in only {:.0}% of the trials", with.used * 100.0);
    // what it learned takes nothing from its counter
    assert!(
        with.untouched >= before.untouched - 0.05,
        "{kind}, learned: {:.0}% untouched against {:.0}% before it learned",
        with.untouched * 100.0,
        before.untouched * 100.0
    );
    assert!(with.across >= 0.90, "{kind}, learned: with its counter only {:.0}% got round", with.across * 100.0);
    // and it holds the counter's band (heard first, the Hound's counter
    // has already been half lost: there only the comparison)
    if !heard_first {
        assert!(with.untouched >= 0.95, "{kind}, learned: with its counter only {:.0}% went untouched", with.untouched * 100.0);
    }
}

#[test]
fn what_each_learns_leaves_its_counter_working() {
    for kind in ["smiler", "hound", "watcher", "corner", "reader"] {
        hold_when_learned(kind);
    }
}

/// The seeds where `ARENA_KIND`'s counter was touched (see the top).
#[test]
#[ignore]
fn debug_trials() {
    let kind = std::env::var("ARENA_KIND").unwrap_or("smiler".into());
    let counter = std::env::var("ARENA_COUNTER").map(|v| v != "false").unwrap_or(true);
    let learned = std::env::var("ARENA_LEARNED").is_ok();
    let setup = Setup { counter, learned, heard_first: std::env::var("ARENA_HEARD").is_ok() || (learned && kind == "hound") };
    for s in 0..SEEDS {
        let o = trial(&kind, setup, 1000 + s);
        if o.contacts > 0 || std::env::var("ARENA_ALL").is_ok() {
            println!("seed {} contacts {} killed {}", 1000 + s, o.contacts, o.killed);
        }
    }
}
