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
