//! The rules of the creatures of the redesign (`docs/ENTITIES.md`), one test
//! a line of their design, in small arenas (`arena/mod.rs`). What a
//! creature did is read back from its brain's log of mode changes.
mod arena;
use arena::*;
use game::base::numeric::hypot2;
use game::entities::brain::Mode;
use game::entities::creature::*;
use game::prelude::*;

const DT: f64 = 1.0 / 60.0;

fn run(g: &mut Game, secs: f64, toward: Option<(f64, f64)>, gait: Gait) {
    let n = (secs / DT).round() as usize;
    for _ in 0..n {
        step(g, DT, toward, gait);
    }
}

/// A 13 × 13 room, a hound at (6, 4) facing +z, you at (6, 6) (8 m off), still.
fn hound_room(wall: impl Fn(i32, i32) -> bool) -> (Game, usize) {
    let mut g = room(13, 0.6, wall);
    let ei = creature(&mut g, "hound", 6, 4, 0.0);
    g.player.pos = v3(g.wx(6), 0.0, g.wz(6));
    (g, ei)
}

/// Keep the hound where it is, listening (`Idle`), for a test of its ears.
fn hold_listening(g: &mut Game, ei: usize) {
    let now = g.perf_t;
    let c = g.creature_mut(ei).unwrap();
    c.brain.set(Mode::Idle, "the test holds it", now);
}

/// Walking on the spot: a goal a centimetre off.
fn on_the_spot(g: &Game) -> Option<(f64, f64)> {
    Some((g.player.pos.x + 0.01, g.player.pos.z))
}

// ------------------------------------------------------------ the Hound

#[test]
fn the_hound_is_blind() {
    // facing you, 8 m off, your lamp on it, you still: it does not know you are there
    let (mut g, ei) = hound_room(|_, _| false);
    g.player.flashlight_on = true;
    let (hx, hz) = (g.wx(6), g.wz(4));
    look_at(&mut g, hx, hz);
    for _ in 0..180 {
        step(&mut g, DT, None, Gait::Still);
        if mode(&g, ei) == Mode::Idle {
            hold_listening(&mut g, ei);
        }
    }
    let c = g.creature(ei).unwrap();
    assert!(c.brain.mem.awareness < 0.05, "it saw you: awareness {}", c.brain.mem.awareness);
    assert!(!modes(&g, ei).contains(&Mode::Hunt));
}

#[test]
fn the_hound_hears_along_the_way_not_through_walls() {
    // a wall across the room between you, open only at its far end: 8 m
    // apart in a straight line, more than 30 m along the way
    let wall = |x: i32, z: i32| z == 5 && x < 11;
    let (mut g, ei) = hound_room(wall);
    hold_listening(&mut g, ei);
    // (the player's flow field is laid by the creatures' first update)
    step(&mut g, DT, None, Gait::Still);
    hold_listening(&mut g, ei);
    let hp = g.entities[ei].pos();
    let path = g.path_distance(hp.x, hp.z, 8.0);
    assert!(path > 26.0, "the way round is {path:.1} m");
    // walking on the spot (heard 13 m by a hound standing still)
    for _ in 0..90 {
        {
            let w = on_the_spot(&g);
            step(&mut g, DT, w, Gait::Walk)
        };
        hold_listening(&mut g, ei);
    }
    assert!(g.creature(ei).unwrap().brain.mem.awareness < 0.05, "it heard you through the wall");
    // the same steps with no wall: it hears you, and stops in its listening crouch
    let (mut g, ei) = hound_room(|_, _| false);
    hold_listening(&mut g, ei);
    for _ in 0..90 {
        {
            let w = on_the_spot(&g);
            step(&mut g, DT, w, Gait::Walk)
        };
    }
    assert!(modes(&g, ei).contains(&Mode::Alert), "it did not hear you in the open: {:?}", modes(&g, ei));
}

#[test]
fn the_hound_listens_before_it_is_sure_and_silence_sends_it_to_look() {
    // you walk where it can hear you: it stops to listen first (the tell)
    let (mut g, ei) = hound_room(|_, _| false);
    hold_listening(&mut g, ei);
    let mut t = 0.0;
    while mode(&g, ei) != Mode::Alert && t < 3.0 {
        {
            let w = on_the_spot(&g);
            step(&mut g, DT, w, Gait::Walk)
        };
        t += DT;
    }
    assert_eq!(mode(&g, ei), Mode::Alert, "it never stopped to listen");
    // you freeze within its grace: it goes to look, it is not sure
    run(&mut g, 0.4, None, Gait::Still);
    let mut t = 0.0;
    while mode(&g, ei) == Mode::Alert && t < 3.0 {
        step(&mut g, DT, None, Gait::Still);
        t += DT;
    }
    assert_eq!(mode(&g, ei), Mode::Search, "silent through its crouch: {:?}", modes(&g, ei));
    assert!(!modes(&g, ei).contains(&Mode::Hunt));
    // the same, but you keep walking through the crouch: it is sure
    let (mut g, ei) = hound_room(|_, _| false);
    hold_listening(&mut g, ei);
    {
        let w = on_the_spot(&g);
        run(&mut g, 3.0, w, Gait::Walk)
    };
    let m = modes(&g, ei);
    let a = m.iter().position(|x| *x == Mode::Alert).expect("an alert");
    let h = m.iter().position(|x| *x == Mode::Hunt).expect("a hunt");
    assert!(a < h, "the tell came first: {m:?}");
    let log = &g.creature(ei).unwrap().brain.log;
    let (ta, th) = (log[a].0, log[h].0);
    assert!(th - ta >= 0.6, "the crouch lasted {:.2} s before it was sure", th - ta);
}

#[test]
fn the_hound_smells_you_close_however_still_you_are() {
    let (mut g, ei) = hound_room(|_, _| false);
    g.player.pos = v3(g.wx(6), 0.0, g.wz(4) + 0.8);
    // it lingers beside you (held listening), you hold still
    let linger = |g: &mut Game, secs: f64| {
        for _ in 0..(secs / DT).round() as usize {
            if mode(g, ei) == Mode::Idle || mode(g, ei) == Mode::Roam {
                hold_listening(g, ei);
            }
            step(g, DT, None, Gait::Still);
        }
    };
    linger(&mut g, 2.5);
    assert!(!modes(&g, ei).contains(&Mode::Hunt), "too soon: {:?}", modes(&g, ei));
    linger(&mut g, 1.5);
    assert!(modes(&g, ei).contains(&Mode::Hunt), "it never smelled you: {:?}", modes(&g, ei));
}

#[test]
fn the_hound_runs_slower_than_you_and_springs_short() {
    // it hunts you across the room while you run: never faster than the cap,
    // and its bursts are short
    let (mut g, ei) = hound_room(|_, _| false);
    g.player.pos = v3(g.wx(6), 0.0, g.wz(10));
    let pp = g.player.pos;
    g.creature_alert(ei, pp.x, pp.z);
    let mut fastest: f64 = 0.0;
    let mut prev = g.entities[ei].pos();
    for k in 0..600 {
        let goal = if (k / 120) % 2 == 0 { (g.wx(2), g.wz(10)) } else { (g.wx(10), g.wz(10)) };
        let before = mode(&g, ei);
        step(&mut g, DT, Some(goal), Gait::Run);
        let p = g.entities[ei].pos();
        let v = hypot2(p.x - prev.x, p.z - prev.z) / DT;
        prev = p;
        // a spring is a burst, judged on its own below; the rest is the chase
        if before != Mode::Strike && mode(&g, ei) != Mode::Strike {
            fastest = fastest.max(v);
        } else {
            assert!(v <= BURST_CAP + 0.05, "a spring at {v:.2} m/s");
        }
        if let Some(c) = g.creature(ei) {
            assert!(c.burst_m <= BURST_LEN + 0.2, "a burst of {:.2} m", c.burst_m);
        }
    }
    assert!(fastest <= CHASE_CAP + 0.05, "it ran at {fastest:.2} m/s (cap {CHASE_CAP:.2})");
}

// ------------------------------------------------------------ fairness

#[test]
fn no_strike_without_two_tells_nor_in_the_first_seconds() {
    let (mut g, ei) = hound_room(|_, _| false);
    // just noticed, no tells: not yet
    {
        let now = g.perf_t;
        let c = g.creature_mut(ei).unwrap();
        c.noticed_at = Some(now - 5.0);
        c.tells.clear();
    }
    assert!(!g.strike_allowed(ei), "no tells");
    let now = g.perf_t;
    g.creature_mut(ei).unwrap().tells = vec![now - 3.0];
    assert!(!g.strike_allowed(ei), "one tell");
    g.creature_mut(ei).unwrap().tells = vec![now - 3.0, now - 1.0];
    assert!(g.strike_allowed(ei), "two tells, noticed 5 s ago");
    // the tells are old: not any more
    g.creature_mut(ei).unwrap().tells = vec![now - 30.0, now - 25.0];
    assert!(!g.strike_allowed(ei), "tells from half a minute ago");
    // a level's first seconds
    g.creature_mut(ei).unwrap().tells = vec![now - 3.0, now - 1.0];
    g.level_start_t = now - 10.0;
    assert!(!g.strike_allowed(ei), "ten seconds into the level");
    // within two seconds of noticing you
    g.level_start_t = 0.0;
    g.creature_mut(ei).unwrap().noticed_at = Some(now - 1.0);
    assert!(!g.strike_allowed(ei), "a second after its notice");
}

#[test]
fn the_first_meeting_does_not_kill_the_second_contact_in_a_peak_does() {
    let (mut g, ei) = hound_room(|_, _| false);
    let nerve = g.player.sanity;
    assert!(!g.creature_contact(ei), "the first contact");
    assert_eq!(g.god_deaths, 0.0, "the first meeting killed");
    assert!(g.player.sanity < nerve, "it took your nerve");
    assert!(g.creature_run.met.contains(&"hound".to_string()));
    assert_eq!(mode(&g, ei), Mode::Withdraw, "it lets you go");
    // the next contact in the same peak kills
    g.director.contacts = 1;
    g.creature_contact(ei);
    assert_eq!(g.god_deaths, 1.0, "the second contact in a peak");
    // a new peak, a full nerve: a contact knocks you down
    g.director.contacts = 0;
    g.player.sanity = SAN_MAX;
    g.creature_contact(ei);
    assert_eq!(g.god_deaths, 1.0, "a first contact in a new peak, nerve full");
    // ... but not on a spent nerve
    g.director.contacts = 0;
    g.player.sanity = 20.0;
    g.creature_contact(ei);
    assert_eq!(g.god_deaths, 2.0, "a contact on a spent nerve");
}

#[test]
fn the_hounds_hair_hangs_whatever_its_head_does() {
    let (mut g, ei) = hound_room(|_, _| false);
    // over your body it lifts its face to you
    g.director.killer = Some(ei);
    g.state = "dead".into();
    for _ in 0..60 {
        game::entities::hound::animate(&mut g, ei, DT);
    }
    let c = g.creature(ei).unwrap();
    let b = &c.body;
    let (h0, h2) = (b.ix("hair.0").unwrap(), b.ix("hair.2").unwrap());
    let (root, tip) = (b.head(h0), b.tail(h2));
    assert!(tip.y < root.y - 0.2, "the hair's tips at {:.2} m, its roots at {:.2} m", tip.y, root.y);
    // and the head did lift
    let head = b.ix("head").unwrap();
    let (hh, ht) = (b.head(head), b.tail(head));
    assert!(ht.y > hh.y, "the crown is not above the neck: {hh:?} → {ht:?}");
}

// ------------------------------------------------------------ the Smiler

/// A dark 13 × 13 room (light 0.08), a smiler at (6, 2) facing +z, you at (6, 7) (20 m off).
fn smiler_room(light: f64) -> (Game, usize) {
    let mut g = room(13, light, |_, _| false);
    let ei = creature(&mut g, "smiler", 6, 2, 0.0);
    g.player.pos = v3(g.wx(6), 0.0, g.wz(7));
    let (sx, sz) = (g.wx(6), g.wz(2));
    look_at(&mut g, sx, sz);
    (g, ei)
}

fn hold_idle(g: &mut Game, ei: usize) {
    let now = g.perf_t;
    let c = g.creature_mut(ei).unwrap();
    if c.brain.mode == Mode::Roam {
        c.brain.set(Mode::Idle, "the test holds it", now);
    }
}

#[test]
fn the_smiler_sees_your_lamp_from_far_and_not_you_in_the_dark() {
    // your lamp on, 20 m off, still: it turns to the light, then it comes
    let (mut g, ei) = smiler_room(0.08);
    g.player.flashlight_on = true;
    for _ in 0..180 {
        step(&mut g, DT, None, Gait::Still);
        hold_idle(&mut g, ei);
    }
    let m = modes(&g, ei);
    assert!(m.contains(&Mode::Alert), "it never saw your lamp: {m:?}");
    // the same with your lamp off: it does not know you are there
    let (mut g, ei) = smiler_room(0.08);
    g.player.flashlight_on = false;
    for _ in 0..180 {
        step(&mut g, DT, None, Gait::Still);
        hold_idle(&mut g, ei);
    }
    assert!(g.creature(ei).unwrap().brain.mem.awareness < 0.05, "it saw you in the dark");
    assert!(!modes(&g, ei).contains(&Mode::Alert));
}

#[test]
fn in_the_dark_the_smiler_loses_you() {
    let (mut g, ei) = smiler_room(0.08);
    g.player.pos = v3(g.wx(6), 0.0, g.wz(5));
    let pp = g.player.pos;
    g.creature_alert(ei, pp.x, pp.z);
    g.player.flashlight_on = true;
    run(&mut g, 1.0, None, Gait::Still);
    assert_eq!(mode(&g, ei), Mode::Hunt, "{:?}", modes(&g, ei));
    // lamp off, crouched and still: it loses you
    g.player.flashlight_on = false;
    let mut t = 0.0;
    while mode(&g, ei) == Mode::Hunt && t < 6.0 {
        step(&mut g, DT, None, Gait::Still);
        t += DT;
    }
    assert_eq!(mode(&g, ei), Mode::Search, "after {t:.1} s in the dark: {:?}", modes(&g, ei));
    assert!(t < 4.0, "it took {t:.1} s to lose you");
}

#[test]
fn running_near_the_smiler_makes_it_lunge() {
    let (mut g, ei) = smiler_room(0.08);
    g.player.pos = v3(g.wx(6), 0.0, g.wz(3) + 2.0);
    let pp = g.player.pos;
    g.creature_alert(ei, pp.x, pp.z);
    g.player.flashlight_on = true;
    run(&mut g, 2.5, None, Gait::Still);
    let now = g.perf_t;
    g.creature_mut(ei).unwrap().tells = vec![now - 2.0, now - 1.0];
    // you run (away along the room)
    let goal = (g.wx(11), g.wz(11));
    run(&mut g, 0.5, Some(goal), Gait::Run);
    assert!(modes(&g, ei).contains(&Mode::Strike), "it did not lunge: {:?}", modes(&g, ei));
}

#[test]
fn the_smiler_is_slow_in_bright_rooms() {
    for (light, cap) in [(0.08, 4.6), (0.8, 2.4)] {
        let (mut g, ei) = smiler_room(light);
        g.player.flashlight_on = true;
        let pp = g.player.pos;
        g.creature_alert(ei, pp.x, pp.z);
        let mut fastest: f64 = 0.0;
        let mut prev = g.entities[ei].pos();
        for _ in 0..120 {
            step(&mut g, DT, None, Gait::Still);
            let p = g.entities[ei].pos();
            if mode(&g, ei) == Mode::Hunt {
                fastest = fastest.max(hypot2(p.x - prev.x, p.z - prev.z) / DT);
            }
            prev = p;
        }
        assert!(fastest <= cap, "light {light}: it came at {fastest:.2} m/s");
        assert!(fastest > 0.5, "light {light}: it never came");
    }
}

#[test]
fn the_smilers_eyes_shine_only_in_your_beam() {
    let (mut g, ei) = smiler_room(0.08);
    g.player.pos = v3(g.wx(6), 0.0, g.wz(5));
    let (sx, sz) = (g.wx(6), g.wz(2));
    // it faces you (+z), you look at it, your lamp on
    g.entities[ei].yaw = 0.0;
    look_at(&mut g, sx, sz);
    g.camera.set_pos(g.player.pos.x, EYE, g.player.pos.z);
    let shine = |g: &mut Game| {
        game::entities::smiler::eyeshine(g, ei);
        let c = g.creature(ei).unwrap();
        let m = c.body.meshes.get("eye.L").unwrap().mats()[0].clone();
        let v = m.m().emissive_intensity;
        v
    };
    g.player.flashlight_on = true;
    g.world.update_matrix_world(true);
    assert!(shine(&mut g) > 0.05, "no eyeshine in the beam");
    g.player.flashlight_on = false;
    assert_eq!(shine(&mut g), 0.0, "eyeshine with your lamp off");
    g.player.flashlight_on = true;
    // you look away: out of the beam
    look_at(&mut g, sx + 30.0, sz);
    assert_eq!(shine(&mut g), 0.0, "eyeshine outside the beam");
}

#[test]
fn gone_dark_and_still_close_to_it_you_are_not_found() {
    // it comes at you, 5 m off; your lamp goes out and you keep still. It
    // goes on to about where your light was, and rarely walks into you
    let mut found = 0;
    for s in 0..40 {
        game::base::rng::seed_native(500 + s);
        let (mut g, ei) = smiler_room(0.08);
        g.player.pos = v3(g.wx(6), 0.0, g.wz(2) + 5.0);
        let pp = g.player.pos;
        g.creature_alert(ei, pp.x, pp.z);
        g.player.flashlight_on = true;
        run(&mut g, 1.0, None, Gait::Still);
        assert_eq!(mode(&g, ei), Mode::Hunt, "{:?}", modes(&g, ei));
        g.player.flashlight_on = false;
        run(&mut g, 12.0, None, Gait::Still);
        if modes(&g, ei).contains(&Mode::Strike) {
            found += 1;
        }
    }
    assert!(found <= 6, "it found you {found} times in 40");
}

#[test]
fn the_smiler_sees_ahead_not_behind() {
    // its back to you 12 m off, your lamp on but pointed away from it: it
    // does not know you are there
    let (mut g, ei) = smiler_room(0.08);
    g.player.pos = v3(g.wx(6), 0.0, g.wz(5));
    g.entities[ei].yaw = PI;
    g.player.flashlight_on = true;
    let (sx, sz) = (g.wx(6), g.wz(2));
    look_at(&mut g, sx + 30.0, sz);
    for _ in 0..120 {
        step(&mut g, DT, None, Gait::Still);
        hold_idle(&mut g, ei);
    }
    assert!(!modes(&g, ei).contains(&Mode::Alert), "it saw a lamp behind it: {:?}", modes(&g, ei));
    // your beam on its back: it sees its own shadow thrown ahead of it
    look_at(&mut g, sx, sz);
    for _ in 0..120 {
        step(&mut g, DT, None, Gait::Still);
        hold_idle(&mut g, ei);
    }
    assert!(modes(&g, ei).contains(&Mode::Alert), "your beam on it went unseen: {:?}", modes(&g, ei));
}

// ------------------------------------------------------------ the Watcher

/// A 13 × 13 room lit at `light`, a watcher at (6, 2) sent by the director,
/// you at (6, `z`) looking at it.
fn watcher_room(light: f64, z: i32) -> (Game, usize) {
    let mut g = room(13, light, |_, _| false);
    let ei = creature(&mut g, "watcher", 6, 2, 0.0);
    g.player.pos = v3(g.wx(6), 0.0, g.wz(z));
    g.director.stalker = Some(ei);
    let p = g.entities[ei].pos();
    look_at(&mut g, p.x, p.z);
    (g, ei)
}

/// You keep looking at it (wherever it is) for `secs`.
fn watch(g: &mut Game, ei: usize, secs: f64) {
    for _ in 0..(secs / DT) as usize {
        let p = g.entities[ei].pos();
        look_at(g, p.x, p.z);
        step(g, DT, None, Gait::Still);
    }
}

/// You look the other way for `secs`.
fn look_away(g: &mut Game, ei: usize, secs: f64) {
    for _ in 0..(secs / DT) as usize {
        let p = g.entities[ei].pos();
        let pp = g.player.pos;
        look_at(g, 2.0 * pp.x - p.x, 2.0 * pp.z - p.z);
        step(g, DT, None, Gait::Still);
    }
}

fn dist_to(g: &Game, ei: usize) -> f64 {
    let p = g.entities[ei].pos();
    hypot2(p.x - g.player.pos.x, p.z - g.player.pos.z)
}

#[test]
fn the_watcher_moves_only_unseen() {
    let (mut g, ei) = watcher_room(0.6, 10);
    let p0 = g.entities[ei].pos();
    watch(&mut g, ei, 4.0);
    let p1 = g.entities[ei].pos();
    assert!(hypot2(p1.x - p0.x, p1.z - p0.z) < 1e-9, "it moved while you watched it");
    assert_eq!(mode(&g, ei), Mode::Hold, "{:?}", modes(&g, ei));
    let d0 = dist_to(&g, ei);
    look_away(&mut g, ei, 2.0);
    assert!(dist_to(&g, ei) < d0 - 3.0, "unseen 2 s, it came only {:.1} m", d0 - dist_to(&g, ei));
    // and the moment you look back, it holds
    let d1 = dist_to(&g, ei);
    watch(&mut g, ei, 2.0);
    assert!((dist_to(&g, ei) - d1).abs() < 1e-9, "it moved after you looked back");
}

#[test]
fn in_the_dark_only_your_lamp_or_nearness_holds_it() {
    // dark, your lamp off, 16 m off: you look at it, and it comes all the same
    let (mut g, ei) = watcher_room(0.08, 6);
    g.player.flashlight_on = false;
    let d0 = dist_to(&g, ei);
    watch(&mut g, ei, 1.5);
    assert!(dist_to(&g, ei) < d0 - 2.0, "unlit, it held in the dark");
    // your lamp on it: it holds
    let (mut g, ei) = watcher_room(0.08, 6);
    g.player.flashlight_on = true;
    let d0 = dist_to(&g, ei);
    watch(&mut g, ei, 2.0);
    assert!((dist_to(&g, ei) - d0).abs() < 1e-9, "it moved in your beam");
    // within 4 m it is seen in any light
    let (mut g, ei) = watcher_room(0.08, 3);
    let p = g.entities[ei].pos();
    g.player.pos = v3(p.x, 0.0, p.z + 3.0);
    g.player.flashlight_on = false;
    let d0 = dist_to(&g, ei);
    watch(&mut g, ei, 2.0);
    assert!(d0 < game::entities::watcher::CLOSE && (dist_to(&g, ei) - d0).abs() < 1e-9, "it moved {d0:.1} m from you");
}

#[test]
fn behind_a_wall_it_is_not_seen() {
    // a wall between you (a gap at its end): looking toward it does not hold it
    let mut g = room(13, 0.6, |x, z| z == 5 && x > 2);
    let ei = creature(&mut g, "watcher", 6, 2, 0.0);
    g.player.pos = v3(g.wx(6), 0.0, g.wz(8));
    g.director.stalker = Some(ei);
    let d0 = dist_to(&g, ei);
    watch(&mut g, ei, 3.0);
    assert!(dist_to(&g, ei) < d0 - 1.0 || modes(&g, ei).contains(&Mode::Stalk), "behind the wall it held: {:?}", modes(&g, ei));
}

#[test]
fn it_stands_differently_only_while_unseen() {
    let (mut g, ei) = watcher_room(0.6, 10);
    let pose = |g: &Game| g.creature(ei).unwrap().get("pose");
    let mut changed = 0;
    let mut last = pose(&g);
    for round in 0..8 {
        // while you watch, its pose holds
        for _ in 0..60 {
            let p = g.entities[ei].pos();
            look_at(&mut g, p.x, p.z);
            step(&mut g, DT, None, Gait::Still);
            assert_eq!(pose(&g), last, "its pose changed while you watched (round {round})");
        }
        look_away(&mut g, ei, 1.0);
        if pose(&g) != last {
            changed += 1;
        }
        last = pose(&g);
        // (kept far: it may not reach you in this test)
        let pp = g.player.pos;
        g.entities[ei].g.set_pos(pp.x, 0.0, pp.z - 20.0);
    }
    assert!(changed >= 3, "it stood the same way {} of 8 times you looked back", 8 - changed);
}

#[test]
fn it_is_never_put_nearer_than_10_m_nor_where_you_see_it() {
    // a long hall; you look away and wait: each time it is simply nearer,
    // it is at least 10 m from you and out of your sight
    let mut g = room(30, 0.6, |x, _| x > 3);
    let ei = creature(&mut g, "watcher", 2, 2, 0.0);
    g.player.pos = v3(g.wx(2), 0.0, g.wz(27));
    g.director.stalker = Some(ei);
    let mut jumps = 0;
    let mut prev = g.entities[ei].pos();
    let mut prev_d = dist_to(&g, ei);
    for _ in 0..(20.0 / DT) as usize {
        let pp = g.player.pos;
        look_at(&mut g, pp.x, pp.z + 10.0);
        step(&mut g, DT, None, Gait::Still);
        let p = g.entities[ei].pos();
        let nearer = dist_to(&g, ei) < prev_d;
        prev_d = dist_to(&g, ei);
        // (a jump nearer: not its falling back after a touch)
        if nearer && hypot2(p.x - prev.x, p.z - prev.z) > 1.0 {
            jumps += 1;
            let d = dist_to(&g, ei);
            assert!(d >= game::entities::watcher::NEVER_WITHIN - 1e-6, "put {d:.1} m from you");
            assert!(!game::entities::watcher::seen(&g, ei), "put where you see it");
        }
        prev = p;
    }
    assert!(jumps > 0, "it was never nearer when you looked back");
}

#[test]
fn its_touch_takes_your_nerve_and_kills_only_a_spent_mind() {
    // it is behind you, 3 m off, and has given its tells; you look away
    let touch = |sanity: f64| {
        let (mut g, ei) = watcher_room(0.6, 4);
        let pp = g.player.pos;
        g.entities[ei].g.set_pos(pp.x, 0.0, pp.z - 3.0);
        g.player.sanity = sanity;
        let now = g.perf_t;
        g.creature_mut(ei).unwrap().tells = vec![now - 2.0, now - 1.0];
        look_away(&mut g, ei, 4.0);
        g
    };
    let g = touch(SAN_MAX);
    let ei = g.entities.iter().position(|e| e.kind == "watcher").unwrap();
    assert!(modes(&g, ei).contains(&Mode::Strike), "it never touched you: {:?}", modes(&g, ei));
    assert!(g.player.sanity <= SAN_MAX - 45.0 + 1e-9, "your nerve: {:.0}", g.player.sanity);
    assert_eq!(g.god_deaths, 0.0, "a touch on a whole mind killed");
    assert!(dist_to(&g, ei) > 2.0, "it did not fall back ({:.1} m)", dist_to(&g, ei));
    // on a mind already nearly gone, the same touch kills
    let g = touch(40.0);
    assert!(g.god_deaths > 0.0, "a spent mind survived its touch");
}

#[test]
fn your_ears_ring_near_it_and_that_is_its_tell() {
    let (mut g, ei) = watcher_room(0.6, 4);
    // 6 m off (inside its ring), unseen
    let pp = g.player.pos;
    g.entities[ei].g.set_pos(pp.x, 0.0, pp.z - 6.0);
    g.director.stalker = None;
    look_away(&mut g, ei, 4.0);
    assert!(!g.creature(ei).unwrap().tells.is_empty(), "no tell within its ring");
    let (mut g, ei) = watcher_room(0.6, 9);
    g.director.stalker = None;
    look_away(&mut g, ei, 4.0);
    assert!(g.creature(ei).unwrap().tells.is_empty(), "a tell from beyond its ring");
}

// ------------------------------------------------------------ the one in the corner

/// A lit 13 × 13 room, the one in the corner in tile (1, 1), its face to
/// the corner there; you at (x, z) tiles. It does not look round on its own
/// unless `looks` (so a test of its rule is about the rule).
fn corner_room(x: i32, z: i32, looks: bool) -> (Game, usize) {
    let mut g = room(13, 0.6, |_, _| false);
    let ei = creature(&mut g, "corner", 1, 1, 0.0);
    game::entities::corner::stand_in(&mut g, ei, 1, 1, -1, -1);
    if !looks {
        g.creature_mut(ei).unwrap().ease.insert("turn_in", 1e9);
    }
    g.player.pos = v3(g.wx(x), 0.0, g.wz(z));
    let p = g.entities[ei].pos();
    look_at(&mut g, p.x, p.z);
    (g, ei)
}

/// It is turned to face you where it stands (as though it had come round).
fn face_me(g: &mut Game, ei: usize) {
    let p = g.entities[ei].pos();
    let pp = g.player.pos;
    let yaw = (pp.x - p.x).atan2(pp.z - p.z);
    g.entities[ei].yaw = yaw;
    g.creature_mut(ei).unwrap().ease.insert("home_yaw", yaw);
}

#[test]
fn its_back_is_harmless() {
    let (mut g, ei) = corner_room(4, 4, false);
    watch(&mut g, ei, 6.0);
    assert!(!modes(&g, ei).contains(&Mode::Alert), "it turned at your look at its back: {:?}", modes(&g, ei));
    assert!(g.creature(ei).unwrap().get("humming") > 0.0, "it stopped humming");
}

#[test]
fn seeing_its_face_turns_it_and_looking_away_in_time_saves_you() {
    let (mut g, ei) = corner_room(5, 5, false);
    face_me(&mut g, ei);
    watch(&mut g, ei, 0.6);
    assert_eq!(mode(&g, ei), Mode::Alert, "{:?}", modes(&g, ei));
    // the hum stopped, and that was a tell
    assert_eq!(g.creature(ei).unwrap().get("humming"), 0.0);
    assert!(!g.creature(ei).unwrap().tells.is_empty());
    look_away(&mut g, ei, 1.0);
    assert_eq!(mode(&g, ei), Mode::Hold, "{:?}", modes(&g, ei));
}

#[test]
fn holding_its_look_brings_it_below_your_sprint() {
    let (mut g, ei) = corner_room(9, 9, false);
    face_me(&mut g, ei);
    watch(&mut g, ei, game::entities::corner::TURN + 0.2);
    assert_eq!(mode(&g, ei), Mode::Hunt, "{:?}", modes(&g, ei));
    let mut fastest: f64 = 0.0;
    let mut prev = g.entities[ei].pos();
    for _ in 0..90 {
        step(&mut g, DT, None, Gait::Still);
        let p = g.entities[ei].pos();
        fastest = fastest.max(hypot2(p.x - prev.x, p.z - prev.z) / DT);
        prev = p;
    }
    assert!(fastest > 4.0 && fastest <= game::entities::corner::PACE + 0.05, "it came at {fastest:.2} m/s");
    assert!(fastest < SPRINT, "faster than you can run");
}

#[test]
fn it_knows_where_you_are_for_6_s_then_goes_back_to_its_corner() {
    // a wall between the halves of the room, a door at its end
    let mut g = room(13, 0.6, |x, z| x == 7 && z > 2);
    let ei = creature(&mut g, "corner", 1, 11, 0.0);
    game::entities::corner::stand_in(&mut g, ei, 1, 11, -1, 1);
    g.creature_mut(ei).unwrap().ease.insert("turn_in", 1e9);
    g.player.pos = v3(g.wx(5), 0.0, g.wz(9));
    face_me(&mut g, ei);
    watch(&mut g, ei, game::entities::corner::TURN + 0.2);
    assert_eq!(mode(&g, ei), Mode::Hunt);
    // you are gone behind the wall, still: for 6 s it comes all the same
    g.player.pos = v3(g.wx(10), 0.0, g.wz(10));
    let d0 = dist_to(&g, ei);
    run(&mut g, 3.0, None, Gait::Still);
    assert!(dist_to(&g, ei) < d0 - 3.0, "it did not know where you were");
    // then, out of its sight, it gives up and goes home, its face to the walls
    g.player.pos = v3(g.wx(11), 0.0, g.wz(3));
    run(&mut g, 30.0, None, Gait::Still);
    let m = modes(&g, ei);
    assert!(m.contains(&Mode::Withdraw), "{m:?}");
    assert_eq!(mode(&g, ei), Mode::Hold, "{m:?}");
    let c = g.creature(ei).unwrap();
    let p = g.entities[ei].pos();
    assert!(hypot2(p.x - c.get("home_x"), p.z - c.get("home_z")) < 0.4, "it is not back in its corner");
}

#[test]
fn now_and_then_it_looks_round_and_then_your_eyes_are_the_danger() {
    // you look at it as it looks round: it has your face
    let (mut g, ei) = corner_room(2, 3, true);
    g.creature_mut(ei).unwrap().ease.insert("turn_in", 0.3);
    watch(&mut g, ei, 0.5);
    assert_eq!(mode(&g, ei), Mode::Alert, "{:?}", modes(&g, ei));
    assert_eq!(g.creature(ei).unwrap().get("humming"), 0.0, "the hum went on");
    watch(&mut g, ei, game::entities::corner::TURN + 0.2);
    assert_eq!(mode(&g, ei), Mode::Hunt, "{:?}", modes(&g, ei));
    // you looked away when the hum stopped: it turns back
    let (mut g, ei) = corner_room(2, 3, true);
    g.creature_mut(ei).unwrap().ease.insert("turn_in", 0.3);
    watch(&mut g, ei, 0.5);
    look_away(&mut g, ei, game::entities::corner::TURN + 0.3);
    assert_eq!(mode(&g, ei), Mode::Hold, "{:?}", modes(&g, ei));
}

#[test]
fn it_is_never_sent() {
    let (mut g, ei) = corner_room(5, 5, false);
    let si = creature(&mut g, "smiler", 10, 10, 0.0);
    g.reset_director();
    for _ in 0..(120.0 / DT) as usize {
        g.update_director(DT);
        if g.director.stalker.is_some() {
            break;
        }
    }
    assert_eq!(g.director.stalker, Some(si), "the director sent {:?}", g.director.stalker.map(|i| g.entities[i].kind.clone()));
    assert_ne!(g.director.stalker, Some(ei));
}
