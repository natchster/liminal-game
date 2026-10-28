//! Arenas (`docs/ENTITIES.md`, design 7): does each creature's counter
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
mod arena;
use arena::*;
use game::base::numeric::hypot2;
use game::base::rng::seed_native;
use game::entities::brain::Mode;
use game::prelude::*;

const DT: f64 = 1.0 / 30.0;
const LIMIT: f64 = 600.0;
const SEEDS: u64 = 100;

#[derive(Debug, Default, Clone, Copy)]
struct Outcome {
    contacts: u32,
    killed: bool,
    reached: bool,
    secs: f64,
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
}

fn heard(g: &Game, ei: usize) -> Heard {
    let e = &g.entities[ei];
    let p = e.pos();
    Heard {
        d: hypot2(p.x - g.player.pos.x, p.z - g.player.pos.z),
        moving: hypot2(e.vx, e.vz) > game::entities::hound::MOVING,
        hunting: matches!(mode(g, ei), Mode::Hunt | Mode::Strike),
        growling: mode(g, ei) == Mode::Strike,
    }
}

/// A 21 × 21 room (84 m) with pillars.
fn pillared() -> Game {
    room(21, 0.5, |x, z| x % 4 == 0 && z % 4 == 0 && x > 1 && z > 1 && x < 19 && z < 19)
}

fn trial(kind: &str, counter: bool, seed: u64) -> Outcome {
    seed_native(seed);
    let mut g = pillared();
    let mut r = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15);
    let mut next = || {
        r ^= r << 13;
        r ^= r >> 7;
        r ^= r << 17;
        (r % 1000) as f64 / 1000.0
    };
    // it starts somewhere about the middle of the room, facing anywhere
    let (cx, cz) = (7 + (next() * 7.0) as i32, 7 + (next() * 7.0) as i32);
    let ei = creature(&mut g, kind, cx, cz, next() * PI * 2.0);
    g.player.pos = v3(g.wx(2), 0.0, g.wz(2));
    // a tour of the room's corners, as you would explore a level
    let tour =
        [(g.wx(18), g.wz(2)), (g.wx(10), g.wz(10)), (g.wx(18), g.wz(18)), (g.wx(2), g.wz(18)), (g.wx(10), g.wz(10)), (g.wx(2), g.wz(2))];
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
    while t < LIMIT {
        let goal = tour[leg];
        let h = heard(&g, ei);
        let gait = if !counter {
            // the naive: walk; when it bays, run for it
            if h.hunting {
                Gait::Run
            } else {
                Gait::Walk
            }
        } else {
            match kind {
                "hound" => hound_counter(&h, stamina),
                other => panic!("no counter for {other}"),
            }
        };
        let gait = if look_left > 0.0 && gait != Gait::Run {
            look_left -= DT;
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
        // a sprint costs breath, as yours does (`player.rs`)
        let gait = if gait == Gait::Run && stamina <= 0.0 { Gait::Walk } else { gait };
        stamina = if gait == Gait::Run { (stamina - STAM_DRAIN * DT).max(0.0) } else { (stamina + STAM_REGEN * DT).min(STAM_MAX) };
        // backing away: straight away from it; otherwise on toward the goal
        let away = {
            let p = g.entities[ei].pos();
            let (dx, dz) = (g.player.pos.x - p.x, g.player.pos.z - p.z);
            let l = hypot2(dx, dz).max(1e-6);
            let (ux, uz) = (dx / l, dz / l);
            // out of its line when it comes at you; straight away otherwise
            let (ax, az) = if h.hunting { (ux * 0.5 - uz, uz * 0.5 + ux) } else { (ux, uz) };
            (g.player.pos.x + ax * 3.0, g.player.pos.z + az * 3.0)
        };
        let toward = if gait == Gait::Still {
            None
        } else if counter && (h.d < 6.0 || h.hunting) {
            Some(away)
        } else {
            Some(goal)
        };
        step(&mut g, DT, toward, gait);
        g.update_director(DT);
        t += DT;
        // a contact (the director logs each)
        out.contacts = g.director.log.iter().filter(|(_, w)| *w == "contact").count() as u32;
        if hypot2(g.player.pos.x - goal.0, g.player.pos.z - goal.1) < 2.0 {
            leg += 1;
            if leg == tour.len() {
                out.reached = true;
                break;
            }
        }
    }
    out.secs = t;
    out.killed = g.god_deaths > 0.0;
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

fn sweep(kind: &str, counter: bool) -> Sweep {
    let mut w = Sweep::default();
    for s in 0..SEEDS {
        let o = trial(kind, counter, 1000 + s);
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
    }
    if w.across > 0.0 {
        w.secs /= w.across;
    }
    let n = SEEDS as f64;
    w.untouched /= n;
    w.killed /= n;
    w.across /= n;
    w
}

#[derive(Debug, Default)]
struct Sweep {
    untouched: f64,
    killed: f64,
    across: f64,
    secs: f64,
}

fn report(kind: &str) -> (Sweep, Sweep) {
    let with = sweep(kind, true);
    let without = sweep(kind, false);
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

/// The bands every creature's arena is held to (`docs/ENTITIES.md`, design 7).
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
fn the_hounds_counter_works_and_matters() {
    let (with, without) = report("hound");
    hold_to_bands("hound", &with, &without);
}
