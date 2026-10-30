//! The one behind the glass (`docs/WINDOWS.md` 4), on real floors (its
//! window is a real one, cut into a real wall, with its lamp's light on the
//! floor): one test a line of its design, read back from its brain's log.
mod common;
use game::core_props::window::Blind;
use game::entities::brain::{Mode, Order};
use game::entities::director::Phase;
use game::entities::glass::{self, AT_GLASS, COME, LAMP_LIGHT, LOSES, NOTICE, PRESS_T, REST, STARE_DRAIN, THROUGH_T, WALK};
use game::prelude::*;
use game::Game;
use std::f64::consts::PI;

const DT: f64 = 1.0 / 60.0;

/// A floor with the one behind the glass: (game, it)
fn level_with_one() -> (Game, usize) {
    let mut g = common::new_game();
    for k in 0..40u32 {
        g.run_seed = k.wrapping_mul(2654435761) ^ 0x51_D0;
        for level in 5..=12 {
            g.init_level(level);
            if let Some(b) = g.behind_glass.clone() {
                g.state = "playing".into();
                g.level_start_t = g.perf_t - 100.0;
                g.director.phase = Phase::Calm;
                return (g, b.ei);
            }
        }
    }
    panic!("no floor with the one behind the glass");
}

/// You, `back` m in from its window, before its first light, your eyes on
/// its glass (or turned away from it)
fn stand(g: &mut Game, back: f64, look: bool) {
    let b = g.behind_glass.clone().unwrap();
    let w = &g.windows[b.win];
    let l = &w.lights[b.lights[0]];
    let floor = w.root.pos().y;
    let eye = w.world(b.x, 1.6, w.depth + back);
    let target = w.world((l.rect[0] + l.rect[1]) / 2.0, (l.rect[2] + l.rect[3]) / 2.0, l.glass_z);
    let (dx, dy, dz) = (target.x - eye.x, target.y - eye.y, target.z - eye.z);
    g.player.pos = Vec3::new(eye.x, floor + 1.6, eye.z);
    g.camera.set_pos(eye.x, eye.y, eye.z);
    if look {
        g.player.yaw = (-dx).atan2(-dz);
        g.player.pitch = dy.atan2(dx.hypot(dz));
    } else {
        g.player.yaw = (-dx).atan2(-dz) + PI;
        g.player.pitch = 0.0;
    }
}

/// `secs` of its mind and body, in steps of 1/60 s
fn run(g: &mut Game, ei: usize, secs: f64) {
    for _ in 0..(secs / DT).round() as usize {
        g.perf_t += DT;
        g.update_creature(ei, DT);
    }
}

fn get(g: &Game, ei: usize, k: &'static str) -> f64 {
    g.creature(ei).unwrap().get(k)
}

fn mode(g: &Game, ei: usize) -> Mode {
    g.creature(ei).unwrap().brain.mode
}

fn why_into(g: &Game, ei: usize, m: Mode) -> Option<&'static str> {
    g.creature(ei).unwrap().brain.last_into(m).map(|c| c.3)
}

/// Put it `out` m beyond its glass in `m`, its look at `stare`
fn put_out(g: &mut Game, ei: usize, out: f64, m: Mode, stare: f64) {
    let now = g.perf_t;
    let c = g.creature_mut(ei).unwrap();
    c.ease.insert("out", out);
    c.ease.insert("stare", stare);
    c.brain.set(m, "the test", now);
}

#[test]
fn it_stands_behind_one_window_of_four_in_ten_floors_with_windows_far_from_the_arrival() {
    let mut g = common::new_game();
    let (mut eligible, mut placed) = (0, 0);
    for k in 0..14u32 {
        g.run_seed = k.wrapping_mul(2654435761) ^ 0x51_D0;
        for level in 5..=12 {
            g.init_level(level);
            let seed = g.run_seed;
            if g.windows.is_empty() || g.level_scp.is_some() {
                assert!(g.behind_glass.is_none(), "seed {seed} level {level}: behind a window where it may not be");
                continue;
            }
            eligible += 1;
            let Some(b) = g.behind_glass.clone() else { continue };
            placed += 1;
            let ei = b.ei;
            assert_eq!(g.entities[ei].kind, "glass");
            assert!(glass::outside(&g, ei), "seed {seed} level {level}: it begins inside");
            // out on the track, REST m beyond its glass, below the floor
            let w = &g.windows[b.win];
            let l = w.local(&g.entities[ei].pos());
            assert!((l.z + REST).abs() < 1e-6 && (l.x - b.x).abs() < 1e-6, "seed {seed} level {level}: it stands at {l:?}");
            assert!(
                b.lights.iter().all(|&k| !w.covered(k) && !w.lights[k].patch.is_empty()),
                "seed {seed} level {level}: behind covered glass"
            );
            // drawn beyond its glass (every part of it)
            let mut all = true;
            g.entities[ei].g.traverse(&mut |o: &Obj| {
                for m in o.mats() {
                    all &= m.r().portal.is_some();
                }
            });
            assert!(all, "seed {seed} level {level}: a part of it is drawn as if in the room");
            // far from the arrival (8 steps at least)
            let from_start = g.bfs(g.tx(g.spawn_pos.x), g.tz(g.spawn_pos.z));
            let t = glass::tile_before(&g, w, b.x).expect("no open tile before its window");
            assert!(from_start[t] >= 8, "seed {seed} level {level}: its window {} steps from the arrival", from_start[t]);
            // the same on every build
            let (win, lights) = (b.win, b.lights.clone());
            g.init_level(level);
            let again = g.behind_glass.clone().expect("gone on the second build");
            assert_eq!((again.win, again.lights), (win, lights), "seed {seed} level {level}: another window on the second build");
        }
    }
    println!("the one behind the glass: {placed} of {eligible} floors with windows");
    assert!(placed >= 8, "only {placed} floors");
    let share = placed as f64 / eligible as f64;
    assert!((0.3..=0.6).contains(&share), "on {placed} of {eligible} floors");
}

#[test]
fn your_look_on_its_glass_adds_up_and_at_three_seconds_it_has_seen_you() {
    let (mut g, ei) = level_with_one();
    stand(&mut g, 1.6, true);
    assert!(glass::eyes_on_glass(&g), "standing before its glass, looking at it, the look does not count");
    run(&mut g, ei, NOTICE - 0.5);
    assert_eq!(mode(&g, ei), Mode::Hold, "it saw you before three seconds");
    // looking away drains it slowly
    stand(&mut g, 1.6, false);
    assert!(!glass::eyes_on_glass(&g));
    run(&mut g, ei, 4.0);
    let left = get(&g, ei, "stare");
    let want = NOTICE - 0.5 - STARE_DRAIN * 4.0;
    assert!((left - want).abs() < 0.05, "after looking away 4 s the look is {left:.2}, not {want:.2}");
    // and it adds up across glances
    stand(&mut g, 1.6, true);
    run(&mut g, ei, NOTICE - left + 0.1);
    assert_eq!(mode(&g, ei), Mode::Alert);
    assert_eq!(why_into(&g, ei, Mode::Alert), Some("it has seen you"));
    let c = g.creature(ei).unwrap();
    assert!(c.noticed_at.is_some() && !c.tells.is_empty(), "the tick is not a tell");
    // beyond its reach the look does not count
    stand(&mut g, glass::LOOK_REACH + 1.0, true);
    assert!(!glass::eyes_on_glass(&g) || !g.los(g.player.pos.x, g.player.pos.z, g.entities[ei].pos().x, g.entities[ei].pos().z));
}

#[test]
fn it_comes_up_the_track_only_while_you_look_and_never_goes_back() {
    let (mut g, ei) = level_with_one();
    stand(&mut g, 1.6, true);
    run(&mut g, ei, COME + 0.2);
    assert_eq!(mode(&g, ei), Mode::Stalk, "after {COME} s of your look it does not come");
    let a = get(&g, ei, "out");
    run(&mut g, ei, 2.0);
    let b = get(&g, ei, "out");
    assert!(((a - b) - 2.0 * WALK).abs() < 0.1, "it came {:.2} m in 2 s", a - b);
    // looked away from, it stands
    stand(&mut g, 1.6, false);
    run(&mut g, ei, 3.0);
    assert!((get(&g, ei, "out") - b).abs() < 1e-9, "it moved while unwatched");
    assert_eq!(mode(&g, ei), Mode::Stalk);
    // it never goes back: its way out only shrinks
    let mut last = get(&g, ei, "out");
    for k in 0..6 {
        stand(&mut g, 1.6, k % 2 == 0);
        run(&mut g, ei, 0.5);
        let now = get(&g, ei, "out");
        assert!(now <= last + 1e-9, "it went back {:.2} m", now - last);
        last = now;
    }
}

#[test]
fn in_its_lamps_light_it_comes_to_the_glass_and_through() {
    let (mut g, ei) = level_with_one();
    put_out(&mut g, ei, LAMP_LIGHT + 0.3, Mode::Stalk, COME);
    stand(&mut g, 1.6, true);
    run(&mut g, ei, 0.3);
    assert_eq!(mode(&g, ei), Mode::Hunt);
    assert_eq!(why_into(&g, ei, Mode::Hunt), Some("in its lamp's light"));
    // now it needs no look
    stand(&mut g, 1.6, false);
    let to_glass = (LAMP_LIGHT - AT_GLASS) / WALK;
    run(&mut g, ei, to_glass + 0.2);
    assert!((get(&g, ei, "out") - AT_GLASS).abs() < 1e-6, "it is not at the glass ({:.2} m out)", get(&g, ei, "out"));
    assert!(glass::outside(&g, ei));
    // its face to the pane, its breath on it: tells you hear standing there
    run(&mut g, ei, 1.0);
    assert!(g.creature(ei).unwrap().tells.len() >= 2, "at the glass it gave {} tells", g.creature(ei).unwrap().tells.len());
    run(&mut g, ei, PRESS_T + THROUGH_T);
    assert!(!glass::outside(&g, ei), "it did not come through");
    let w = &g.windows[g.behind_glass.as_ref().unwrap().win];
    let p = g.entities[ei].pos();
    assert!(w.lit_at(p.x, p.z), "it came down out of its light");
}

#[test]
fn covered_or_in_a_quiet_place_it_does_not_come_through() {
    for how in ["covered", "quiet"] {
        let (mut g, ei) = level_with_one();
        let b = g.behind_glass.clone().unwrap();
        put_out(&mut g, ei, AT_GLASS + 0.5, Mode::Hunt, COME);
        stand(&mut g, 1.6, false);
        if how == "covered" {
            for &k in &b.lights {
                g.windows[b.win].lights[k].blind = Some(Blind { down: 1.0, open: 0.0, skew: 0.0 });
            }
        } else {
            g.creature_mut(ei).unwrap().brain.order = Order::Away(20.0);
        }
        run(&mut g, ei, 8.0);
        assert!(glass::outside(&g, ei), "{how}, it came through");
        assert!((get(&g, ei, "out") - AT_GLASS).abs() < 1e-6, "{how}, it is not at the glass");
        // a covered glass takes no look
        if how == "covered" {
            stand(&mut g, 1.6, true);
            assert!(!glass::eyes_on_glass(&g), "a look counted through a covered glass");
        }
    }
}

/// It inside, in its light, as come through; you `d` m from it
fn inside(g: &mut Game, ei: usize) {
    put_out(g, ei, AT_GLASS, Mode::Hunt, COME);
    let now = g.perf_t;
    let c = g.creature_mut(ei).unwrap();
    c.ease.insert("pressed_at", now - 10.0);
    stand(g, 1.6, false);
    run(g, ei, PRESS_T + THROUGH_T + 0.2);
    assert!(!glass::outside(g, ei), "it did not come in");
}

#[test]
fn inside_it_is_only_where_its_light_falls_and_takes_you_only_in_it() {
    let (mut g, ei) = level_with_one();
    inside(&mut g, ei);
    let b = g.behind_glass.clone().unwrap();
    // you just out of its light, near: it comes to the edge and no further
    let cells: Vec<(f64, f64)> = b
        .lights
        .iter()
        .flat_map(|&k| g.windows[b.win].lights[k].patch.iter().map(|q| ((q[0].0 + q[2].0) / 2.0, (q[0].1 + q[2].1) / 2.0)))
        .collect();
    let w = &g.windows[b.win];
    let mut spot = None;
    for s in 2..30 {
        let p = w.world(b.x, 0.0, w.depth + s as f64 * 0.2);
        if !w.lit_at(p.x, p.z) && !g.circle_hits_wall(p.x, p.z, 0.3) && cells.iter().any(|c| (c.0 - p.x).hypot(c.1 - p.z) < 1.4) {
            spot = Some(p);
            break;
        }
    }
    let spot = spot.expect("no place out of its light near it");
    let sanity = g.player.sanity;
    g.player.pos = Vec3::new(spot.x, g.player.pos.y, spot.z);
    for _ in 0..240 {
        g.perf_t += DT;
        g.update_creature(ei, DT);
        let p = g.entities[ei].pos();
        assert!(g.windows[b.win].lit_at(p.x, p.z), "it stepped out of its light");
    }
    assert_eq!(g.player.sanity, sanity, "it reached you out of its light");
    // in its light, by it: it takes you, and you are thrown out of the light
    let p = g.entities[ei].pos();
    g.creature_mut(ei).unwrap().noticed_at = Some(g.perf_t - 10.0);
    g.player.pos = Vec3::new(p.x + 0.3, g.player.pos.y, p.z);
    assert!(
        g.windows[b.win].lit_at(g.player.pos.x, g.player.pos.z) || {
            // (the cell beside it may be dark: stand on it)
            g.player.pos = Vec3::new(p.x, g.player.pos.y, p.z);
            true
        }
    );
    run(&mut g, ei, 1.0);
    assert!(g.player.sanity < sanity - 30.0, "its touch took {:.0} nerve", sanity - g.player.sanity);
    assert!(g.state == "playing", "its first touch killed");
    assert!(!g.windows[b.win].lit_at(g.player.pos.x, g.player.pos.z), "you were left in its light");
}

#[test]
fn out_of_its_sight_it_goes_back_out_and_stands_close() {
    let (mut g, ei) = level_with_one();
    inside(&mut g, ei);
    // you far off, out of its sight
    let far = (0..g.h).flat_map(|z| (0..g.w).map(move |x| (x, z))).find(|&(x, z)| {
        let (wx, wz) = (g.wx(x), g.wz(z));
        let p = g.entities[ei].pos();
        !g.is_wall_tile(x, z) && (wx - p.x).hypot(wz - p.z) > 20.0 && !g.los(wx, wz, p.x, p.z)
    });
    let (x, z) = far.expect("no place out of its sight");
    g.player.pos = Vec3::new(g.wx(x), g.player.pos.y, g.wz(z));
    run(&mut g, ei, LOSES + glass::BACK_T + 0.5);
    assert!(glass::outside(&g, ei), "it stayed in");
    assert_eq!(mode(&g, ei), Mode::Alert);
    assert_eq!(why_into(&g, ei, Mode::Alert), Some("back out, close"));
    assert!((get(&g, ei, "out") - (AT_GLASS + 1.2)).abs() < 1e-6, "it stands {:.2} m out", get(&g, ei, "out"));
}

#[test]
fn out_there_it_is_nothing_to_the_director_and_it_is_never_sent() {
    let (mut g, ei) = level_with_one();
    stand(&mut g, 1.6, false);
    run(&mut g, ei, 1.0);
    assert_eq!(g.entities[ei].state, "wander", "standing out there it reads as {}", g.entities[ei].state);
    // an order to go away is not a mode for it
    g.creature_mut(ei).unwrap().brain.order = Order::Away(30.0);
    run(&mut g, ei, 1.0);
    assert_eq!(mode(&g, ei), Mode::Hold);
    put_out(&mut g, ei, 20.0, Mode::Stalk, COME);
    run(&mut g, ei, 0.1);
    assert_eq!(g.entities[ei].state, "search");
    put_out(&mut g, ei, 5.0, Mode::Hunt, COME);
    run(&mut g, ei, 0.1);
    assert_eq!(g.entities[ei].state, "hunt");
    // the director never chooses it
    assert!(g.director.stalker != Some(ei));
}

// ------------------------------------------------------------ its counter

/// How a run of trials went: the share untouched, and the share where it
/// came through at all
#[derive(Debug)]
struct Outcome {
    trials: usize,
    untouched: f64,
    came_in: f64,
}

/// One trial on a floor with it: you in its window's room for 75 s. With
/// the counter (a player who knows it): you glance out now and then (a
/// second and a half in twelve) and never once it has seen you, and
/// should it come in, you keep out of its light. Without: you stare out,
/// and when it comes in you go to it. → (touched, it came in)
fn trial(g: &mut Game, seed: u32, level: i32, counter: bool) -> (bool, bool) {
    g.run_seed = seed;
    g.init_level(level);
    let ei = g.behind_glass.as_ref().unwrap().ei;
    g.state = "playing".into();
    g.level_start_t = g.perf_t - 100.0;
    g.director.phase = Phase::Calm;
    let b = g.behind_glass.clone().unwrap();
    // where you stand: 1.6 m in; with the counter, the first place out of
    // its light along the same line
    let mut back = 1.6;
    if counter {
        let w = &g.windows[b.win];
        while back < 4.0 && w.lit_at(w.world(b.x, 0.0, w.depth + back).x, w.world(b.x, 0.0, w.depth + back).z) {
            back += 0.2;
        }
    }
    stand(g, back, !counter);
    let sanity = g.player.sanity;
    let mut came_in = false;
    for k in 0..(75.0 / DT) as usize {
        let t = k as f64 * DT;
        let seen = g.creature(ei).unwrap().noticed_at.is_some();
        if glass::outside(g, ei) {
            let look = !counter || (!seen && t % 12.0 < 1.5);
            stand(g, back, look);
        } else {
            came_in = true;
            let p = g.entities[ei].pos();
            let pp = g.player.pos;
            let (dx, dz) = (p.x - pp.x, p.z - pp.z);
            let d = dx.hypot(dz).max(1e-6);
            if !counter && d > 0.4 {
                // to it, at a walk
                g.player.pos.x += dx / d * 1.4 * DT;
                g.player.pos.z += dz / d * 1.4 * DT;
            }
        }
        g.perf_t += DT;
        g.update_creature(ei, DT);
    }
    (g.player.sanity < sanity - 1.0, came_in)
}

/// Its counter's trials over the floors that have it (both ways)
fn report(kind: &str) -> (Outcome, Outcome) {
    assert_eq!(kind, "glass");
    let mut floors = vec![];
    let mut g = common::new_game();
    for k in 0..40u32 {
        g.run_seed = k.wrapping_mul(2654435761) ^ 0x51_D0;
        for level in 5..=12 {
            g.init_level(level);
            if g.behind_glass.is_some() {
                floors.push((g.run_seed, level));
            }
        }
        if floors.len() >= 20 {
            break;
        }
    }
    let mut run = |counter: bool| {
        let r: Vec<(bool, bool)> = floors.iter().map(|&(s, l)| trial(&mut g, s, l, counter)).collect();
        let n = r.len() as f64;
        Outcome {
            trials: r.len(),
            untouched: r.iter().filter(|o| !o.0).count() as f64 / n,
            came_in: r.iter().filter(|o| o.1).count() as f64 / n,
        }
    };
    (run(true), run(false))
}

#[test]
fn its_counter_holds_on_real_floors() {
    let (with, without) = report("glass");
    println!("the one behind the glass: with its counter {with:?}; without {without:?}");
    assert!(with.trials >= 15, "only {} floors", with.trials);
    assert!(with.untouched >= 0.95, "with its counter only {:.0}% went untouched", with.untouched * 100.0);
    assert!(without.untouched <= 0.70, "without its counter {:.0}% went untouched", without.untouched * 100.0);
    assert!(without.came_in >= 0.9, "stared at, it came in on only {:.0}% of floors", without.came_in * 100.0);
}
