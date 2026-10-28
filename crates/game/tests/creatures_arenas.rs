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
//!
//! To see why a counter fails: `debug_trials` lists the seeds where the
//! counter was touched, and `ARENA_DEBUG=kind:true:seed` prints that trial's
//! mode changes:
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
            || (e.kind == "watcher" && g.path_distance(p.x, p.z, d) < game::entities::watcher::RING_REACH),
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

fn trial(kind: &str, counter: bool, seed: u64) -> Outcome {
    seed_native(seed);
    let mut g = pillared(kind);
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
    // your lamp: on in the dark, as anyone's (the Smiler's counter turns it
    // off for a while when it hears it stir)
    let mut wary = 0.0;
    let mut glance = 0.0;
    while t < LIMIT {
        let goal = tour[leg];
        let h = heard(&g, ei);
        if h.stirred {
            wary = 15.0;
        }
        wary = f64::max(wary - DT, 0.0);
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
                other => panic!("no counter for {other}"),
            }
        };
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
            // out of its line when the Hound comes at you; straight away
            // otherwise (from the Smiler, through the light: it is slow there)
            let (ax, az) = if h.hunting { (ux * 0.5 - uz, uz * 0.5 + ux) } else { (ux, uz) };
            (g.player.pos.x + ax * 3.0, g.player.pos.z + az * 3.0)
        };
        let toward = if gait == Gait::Still {
            None
        } else if counter && kind == "smiler" && (h.hunting || (h.stirred && h.d < 8.0)) {
            Some(escape(&g, ei))
        } else if counter && kind == "hound" && (h.d < 6.0 || h.hunting) {
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
        } else {
            None
        };
        step_facing(&mut g, DT, toward, gait, face);
        g.update_director(DT);
        t += DT;
        if std::env::var("ARENA_DEBUG").map(|v| v == format!("{kind}:{counter}:{seed}")).unwrap_or(false) && mode(&g, ei) != before {
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
fn the_hounds_counter_works_and_matters() {
    let (with, without) = report("hound");
    hold_to_bands("hound", &with, &without);
}

/// The seeds where `ARENA_KIND`'s counter was touched (see the top).
#[test]
#[ignore]
fn debug_trials() {
    let kind = std::env::var("ARENA_KIND").unwrap_or("smiler".into());
    let counter = std::env::var("ARENA_COUNTER").map(|v| v != "false").unwrap_or(true);
    for s in 0..SEEDS {
        let o = trial(&kind, counter, 1000 + s);
        if o.contacts > 0 || std::env::var("ARENA_ALL").is_ok() {
            println!("seed {} contacts {} killed {}", 1000 + s, o.contacts, o.killed);
        }
    }
}
