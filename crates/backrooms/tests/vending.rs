//! The vending machine's rules (`docs/VENDING.md`, the design): one test a
//! rule, each played on a machine of a real level, and what holds them all
//! (it is the same machine on every build, its uses draw nothing from the
//! build's stream, it is solid).
mod common;
use game::core_props::vending::model::{CoilPlace, D, FACE, GLASS_Z, H, TRAY_Y};
use game::core_props::vending::paint::{designs, Ware};
use game::core_props::vending::rules::*;
use game::prelude::*;
use game::Game;

/// A level of the rooms with a vending machine still on (or dead, with
/// `powered` false): (game, its index)
fn level_with(powered: bool) -> (Game, usize) {
    let mut g = common::new_game();
    for k in 0..40u32 {
        g.run_seed = k.wrapping_mul(2654435761) ^ 0x7E2D;
        for level in [5, 6, 8] {
            g.init_level(level);
            if let Some(i) = g.vending.iter().position(|m| m.powered == powered) {
                g.state = "playing".into();
                return (g, i);
            }
        }
    }
    panic!("no level with a machine {}", if powered { "on" } else { "dead" });
}

/// A level of the rooms with a dead machine that has something in it
/// (rule 9), or with `inhabited` false one that has nothing
fn level_with_dead(inhabited: bool) -> (Game, usize) {
    let mut g = common::new_game();
    for k in 0..60u32 {
        g.run_seed = k.wrapping_mul(2654435761) ^ 0x7E2D;
        for level in [5, 6, 8] {
            g.init_level(level);
            let found = (0..g.vending.len()).find(|&i| {
                let m = &g.vending[i];
                !m.powered && m.state.inhabited == inhabited && stand_before(&g, i).is_some()
            });
            if let Some(i) = found {
                g.state = "playing".into();
                return (g, i);
            }
        }
    }
    panic!("no level with a dead machine (inhabited {inhabited})");
}

/// Where to stand before machine `i`'s glass, within rule 9's reach: the
/// farthest open point, with nothing between (None in a tight spot)
fn stand_before(g: &Game, i: usize) -> Option<Vec3> {
    let m = &g.vending[i];
    let glass = m.world(0.0, 0.9, D / 2.0);
    [1.4, 1.2, 1.0, 0.8]
        .into_iter()
        .map(|d| m.world(0.0, 0.0, D / 2.0 + d))
        .find(|p| !g.circle_hits_wall(p.x, p.z, 0.25) && g.los(p.x, p.z, glass.x, glass.z))
}

/// Stand at the machine's face and look at a point of it (its frame)
fn look_at(g: &mut Game, i: usize, x: f64, y: f64, z: f64) {
    let m = &g.vending[i];
    let eye = m.world(x * 0.6, EYE, D / 2.0 + 0.9);
    let target = m.world(x, y, z);
    g.player.pos = Vec3::new(eye.x, eye.y - EYE + EYE, eye.z);
    g.camera.set_pos(eye.x, eye.y, eye.z);
    g.camera.look_at(target.x, target.y, target.z);
    g.camera.update_matrix_world(true);
}

/// `secs` of play in steps of 1/60 s (the machine's animator and the rest)
fn run(g: &mut Game, secs: f64) {
    let dt = 1.0 / 60.0;
    for _ in 0..(secs / dt).round() as usize {
        g.perf_t += dt;
        let t = g.perf_t;
        g.run_animators(dt, t);
    }
}

fn verb(g: &Game, i: usize) -> Option<Verb> {
    g.vending_verb(i)
}

/// A stocked coil and where to look to choose it
fn stocked(g: &Game, i: usize, want: Option<Ware>) -> (usize, CoilPlace) {
    let m = &g.vending[i];
    let d = designs();
    let k =
        (0..m.places.len()).find(|&k| m.stock[k].count > 0 && want.is_none_or(|w| d[m.stock[k].design].kind == w)).expect("a stocked coil");
    (k, m.places[k].clone())
}

/// A Hound standing `dist` m out in front of machine `i`
fn hound_before(g: &mut Game, i: usize, dist: f64) -> usize {
    let p = g.vending[i].world(0.0, 0.0, D / 2.0 + dist);
    let (x, z) = g.open_point(p.x, p.z);
    let ei = g.add_entity("hound", g.tx(x), g.tz(z), 0);
    g.entities[ei].g.set_pos(x, 0.0, z);
    ei
}

#[test]
fn it_reads_your_eyes() {
    // rule 3: looking through the glass at a coil, its display shows its code
    let (mut g, i) = level_with(true);
    let (k, c) = stocked(&g, i, None);
    look_at(&mut g, i, c.x, TRAY_Y[c.tray] + 0.08, GLASS_Z);
    run(&mut g, 0.1);
    assert_eq!(g.vending[i].state.gaze, Some(k), "the coil under the eyes");
    assert!(g.vending[i].state.shown.0.starts_with(&c.code), "the display shows {:?}, not {}", g.vending[i].state.shown.0, c.code);
    // looking away, it runs its own line again
    look_at(&mut g, i, 2.5, 0.5, 0.0);
    run(&mut g, 0.1);
    assert_eq!(g.vending[i].state.gaze, None);
    assert!(g.vending[i].state.shown.0.contains("HAVE A NICE DAY"));
}

#[test]
fn insert_nothing_get_coin() {
    // rule 4: the coin return with nothing in it gives a coin, once; and it is heard
    let (mut g, i) = level_with(true);
    g.player.coins = 0;
    look_at(&mut g, i, 0.35, 1.2, D / 2.0);
    assert_eq!(verb(&g, i), Some(Verb::CoinReturn));
    let near = hound_before(&mut g, i, 6.0);
    g.vending_use(i, Verb::CoinReturn);
    run(&mut g, RETURN_T + 0.1);
    assert_eq!(g.player.coins, 1, "a coin in the cup");
    let heard = g.creature(near).and_then(|c| c.brain.mem.heard);
    assert!(heard.is_some(), "what hears, near, heard it ring");
    // once a machine: with the coin spent it offers no second one
    g.player.coins = 0;
    assert_ne!(verb(&g, i), Some(Verb::CoinReturn));
}

#[test]
fn insert_coin_get_nothing() {
    // rule 5: a coin in, the coil turns, its front good hangs at the glass,
    // nothing falls; then a silence: steps near it are not heard
    let (mut g, i) = level_with(true);
    let (k, c) = stocked(&g, i, None);
    let before = g.vending[i].stock[k].count;
    g.player.coins = 1;
    look_at(&mut g, i, c.x, TRAY_Y[c.tray] + 0.08, GLASS_Z);
    run(&mut g, 0.1);
    assert_eq!(verb(&g, i), Some(Verb::InsertCoin));
    let d0 = seeded_draws();
    g.vending_use(i, Verb::InsertCoin);
    run(&mut g, VEND_T + 1.0);
    let s = &g.vending[i].state;
    assert_eq!(s.hanging, Some(k), "its front good hangs");
    assert!(s.bin.is_none(), "nothing fell");
    assert_eq!(g.vending[i].stock[k].count, before, "nothing left the coil");
    assert_eq!(g.player.coins, 0, "it kept the coin");
    assert!(s.shown.0.contains("MAKE ANOTHER"), "it says {:?}", s.shown.0);
    assert!(s.silent_until > g.perf_t + SILENCE_T - 5.0, "the silence");
    // in the silence a body walking beside it is not heard
    let p = g.vending[i].world(0.0, 0.0, D / 2.0 + 1.0);
    assert!(g.vending_muffles(p.x, p.z));
    assert!(g.vending_hush(p.x, p.z) > 0.99, "the room round it holds its breath");
    let far = g.vending[i].world(0.0, 0.0, D / 2.0 + SILENCE_REACH + 3.0);
    assert!(!g.vending_muffles(far.x, far.z), "only near it");
    assert_eq!(g.vending_hush(far.x, far.z), 0.0, "and the room is hushed only round it");
    let at = g.vending[i].pos();
    g.vending_phantom(at);
    assert!(g.vending[i].state.spin.is_none(), "no coil turns by itself in its silence");
    assert_eq!(seeded_draws(), d0, "a use draws nothing from the build's stream");
}

#[test]
fn what_hangs_is_rocked_loose_and_heard() {
    // rule 6: shoves bang (heard 20 m along the way), and on its count what
    // hangs falls; reach in and take it
    let (mut g, i) = level_with(true);
    let (k, c) = stocked(&g, i, Some(Ware::Almond));
    g.player.coins = 1;
    look_at(&mut g, i, c.x, TRAY_Y[c.tray] + 0.08, GLASS_Z);
    run(&mut g, 0.1);
    g.vending_use(i, Verb::InsertCoin);
    run(&mut g, VEND_T + 1.0);
    assert_eq!(g.vending[i].state.hanging, Some(k));
    let need = g.vending[i].state.drop_on;
    assert!((2..=4).contains(&need));
    let near = hound_before(&mut g, i, 14.0);
    for n in 1..=need {
        assert_eq!(verb(&g, i), Some(Verb::Rock), "shove {n}");
        g.vending_use(i, Verb::Rock);
        run(&mut g, SHOVE_T + FALL_T + 0.2);
        if n < need {
            assert_eq!(g.vending[i].state.hanging, Some(k), "it still hangs after {n}");
        }
    }
    assert!(g.creature(near).and_then(|c| c.brain.mem.heard).is_some(), "the bang was heard 14 m off");
    assert!(g.vending[i].state.hanging.is_none());
    assert!(g.vending[i].state.bin.is_some(), "it fell into the bin");
    assert_eq!(verb(&g, i), Some(Verb::ReachIn));
    let bottles = g.player.bottles;
    g.vending_use(i, Verb::ReachIn);
    assert_eq!(g.player.bottles, bottles + 1, "almond water in the pocket");
}

#[test]
fn it_is_solid_and_the_same_machine_every_build() {
    let (mut g, i) = level_with(true);
    let m = &g.vending[i];
    let c = m.world(0.0, 0.0, 0.0);
    assert!(g.circle_hits_wall(c.x, c.z, 0.3), "a body cannot stand inside it");
    let front = m.world(0.0, 0.0, D / 2.0 + 0.6);
    assert!(!g.circle_hits_wall(front.x, front.z, 0.3), "and can stand before it");
    let key = |g: &Game| {
        g.vending
            .iter()
            .map(|m| (m.powered, m.stock.iter().map(|s| (s.design, s.count)).collect::<Vec<_>>(), m.pos().x, m.pos().z))
            .collect::<Vec<_>>()
    };
    let (seed, level) = (g.run_seed, g.level_num);
    let a = key(&g);
    g.init_level(level + 1);
    g.run_seed = seed;
    g.init_level(level);
    assert_eq!(key(&g), a, "the same machines, built after another level");
}

#[test]
fn it_keeps_its_rhythm() {
    // rule 1: lit in its rhythm, then a click, dark and out of the light
    // map for a few seconds, then lit again
    let (mut g, i) = level_with(true);
    let (on, off, phase) = g.vending[i].state.rhythm;
    assert!((40.0..=60.0).contains(&on) && (6.0..=9.0).contains(&off), "{on} {off}");
    // to the start of a dark stretch, then through it
    let t0 = g.perf_t;
    let into = (t0 + phase).rem_euclid(on + off);
    run(&mut g, (on - into).rem_euclid(on + off) + 0.5);
    assert!(!g.vending[i].state.lit, "dark in its rhythm");
    assert!(g.vending_light().is_empty() || g.vending.iter().filter(|m| m.state.lit).count() == g.vending_light().len());
    run(&mut g, off);
    assert!(g.vending[i].state.lit, "and lit again");
}

#[test]
fn it_goes_dark_when_something_comes() {
    // rule 2: a creature after you near it puts it out, until a moment after it has gone
    let (mut g, i) = level_with(true);
    let front = g.vending[i].world(0.0, 0.0, D / 2.0 + 3.0);
    g.player.pos = Vec3::new(front.x, EYE, front.z);
    g.vending[i].state.rhythm = (1e9, 0.0, 0.0);
    run(&mut g, 0.2);
    assert!(g.vending[i].state.lit);
    let ei = hound_before(&mut g, i, 8.0);
    if let Some(c) = g.creature_mut(ei) {
        c.brain.mode = game::entities::brain::Mode::Hunt;
    }
    run(&mut g, 0.2);
    assert!(!g.vending[i].state.lit, "dark while it comes");
    assert_eq!(g.vending[i].state.last_sound, None, "out without a sound (its own beat clicks)");
    let far = g.vending[i].world(0.0, 0.0, D / 2.0 + DARK_REACH + 30.0);
    g.entities[ei].g.set_pos(far.x, 0.0, far.z);
    run(&mut g, DARK_HOLD - 1.0);
    assert!(!g.vending[i].state.lit, "still dark just after");
    run(&mut g, 1.5);
    assert!(g.vending[i].state.lit, "lit again once it has gone");
    assert_eq!(g.vending[i].state.last_sound, None, "and back without a sound");
    // its own beat, by contrast: a click
    g.vending[i].state.rhythm = (0.001, 1e9, 0.0);
    run(&mut g, 0.2);
    assert!(!g.vending[i].state.lit);
    assert!(matches!(g.vending[i].state.last_sound, Some((Sfx::Click, _))), "its beat clicks: {:?}", g.vending[i].state.last_sound);
}

#[test]
fn never_rock_or_tilt() {
    // rule 7: with nothing hanging, the second shove creaks and the third
    // fells it: heard all over the floor, it throws you down (never to
    // death), it lies solid where it fell, and its almond water and
    // batteries lie by it
    let (mut g, i) = level_with(true);
    g.player.coins = 0;
    g.vending[i].state.coin_given = true;
    let m = &g.vending[i];
    let front = m.world(0.0, 0.0, D / 2.0 + 0.6);
    g.player.pos = Vec3::new(front.x, EYE, front.z);
    g.player.sanity = 40.0;
    let far = hound_before(&mut g, i, 60.0);
    let (almonds, batteries) = (g.almonds.len(), g.battery_items.len());
    for n in 1..=FALL_ON {
        assert_eq!(verb(&g, i), Some(Verb::Rock), "shove {n}");
        g.vending_use(i, Verb::Rock);
        run(&mut g, SHOVE_T + 0.1);
        assert_eq!(g.vending[i].state.creaked, n >= CREAK_ON, "the creak on shove {CREAK_ON}");
    }
    run(&mut g, TOPPLE_T + 0.2);
    let m = &g.vending[i];
    assert!(m.state.down, "it fell");
    assert_eq!(verb(&g, i), None, "nothing more to do with it");
    assert!(g.creature(far).and_then(|c| c.brain.mem.heard).is_some(), "the crash was heard 60 m off");
    assert!(
        g.player.sanity >= game::entities::creature::KNOCKED_LEAVES && g.player.sanity < 40.0,
        "thrown down, alive: {}",
        g.player.sanity
    );
    let lies = m.world(0.0, 0.0, FACE + H / 2.0);
    assert!(g.circle_hits_wall(lies.x, lies.z, 0.2), "it lies solid before its wall");
    let stood = m.world(0.0, 0.0, -D / 4.0);
    assert!(!g.circle_hits_wall(stood.x, stood.z, 0.05), "and not where it stood");
    assert!(!g.circle_hits_wall(g.player.pos.x, g.player.pos.z, 0.3), "you are out from under it");
    let d = designs();
    let held = |w: Ware| m.stock.iter().any(|s| s.count > 0 && d[s.design].kind == w);
    if held(Ware::Almond) {
        assert!(g.almonds.len() > almonds, "its almond water spilled");
    }
    if held(Ware::Battery) {
        assert!(g.battery_items.len() > batteries, "its batteries spilled");
    }
    assert!(g.vending_light().is_empty(), "dark for good");
}

#[test]
fn it_restocks_in_its_dark() {
    // rule 8: a coil you emptied is full again when its light comes back
    // after a dark beat; not before, however long you look away
    let (mut g, i) = level_with(true);
    let (k, c) = stocked(&g, i, None);
    let full = g.vending[i].stock[k].count;
    g.player.coins = 1;
    look_at(&mut g, i, c.x, TRAY_Y[c.tray] + 0.08, GLASS_Z);
    run(&mut g, 0.1);
    g.vending_use(i, Verb::InsertCoin);
    run(&mut g, VEND_T + 1.0);
    for _ in 0..g.vending[i].state.drop_on {
        g.vending_use(i, Verb::Rock);
        run(&mut g, SHOVE_T + FALL_T + 0.2);
    }
    g.vending_use(i, Verb::ReachIn);
    assert_eq!(g.vending[i].stock[k].count, full - 1);
    // lit, looked at or not, nothing changes
    g.vending[i].state.rhythm = (1e9, 0.0, 0.0);
    let face = g.vending[i].world(0.0, 0.0, D / 2.0 + 2.0);
    g.player.pos = Vec3::new(face.x, EYE, face.z);
    g.player.yaw = g.vending[i].yaw();
    run(&mut g, 20.0);
    g.player.yaw += PI;
    run(&mut g, 40.0);
    assert_eq!(g.vending[i].stock[k].count, full - 1, "not while it is lit");
    // a dark beat: still short in the dark, full as the light comes back
    g.vending[i].state.rhythm = (0.001, 1e9, 0.0);
    run(&mut g, 2.0);
    assert!(!g.vending[i].state.lit);
    assert_eq!(g.vending[i].stock[k].count, full - 1, "not in the dark itself");
    g.vending[i].state.rhythm = (1e9, 0.0, 0.0);
    run(&mut g, 0.1);
    assert!(g.vending[i].state.lit);
    assert_eq!(g.vending[i].stock[k].count, full, "full again with its light");
}

#[test]
fn something_is_inside_the_dead_ones() {
    // rule 9: stand close before a dead one's glass a moment (whichever way
    // you face) and, once, its flap lifts from inside, is held and let
    // fall, and a hand print comes up on the inside of its glass
    let (mut g, i) = level_with_dead(true);
    let face = stand_before(&g, i).unwrap();
    g.player.pos = Vec3::new(face.x, EYE, face.z);
    g.player.yaw = g.vending[i].yaw();
    let print = g.vending[i].print.clone().expect("a dead one has its print");
    let away = g.vending[i].world(0.0, 0.0, D / 2.0 + 4.0);
    // passing is not enough: it waits for you to stand there
    run(&mut g, INSIDE_WAIT * 0.6);
    g.player.pos = Vec3::new(away.x, EYE, away.z);
    run(&mut g, 0.5);
    g.player.pos = Vec3::new(face.x, EYE, face.z);
    run(&mut g, INSIDE_WAIT * 0.6);
    assert!(g.vending[i].state.lifted.is_none() && !print.visible(), "not in passing");
    // your back to it, standing there, it is the same
    g.player.yaw += PI;
    run(&mut g, INSIDE_WAIT * 0.5 + LIFT_T * 0.5);
    g.player.yaw -= PI;
    let t = g.vending[i].state.lifted;
    assert!(t.is_some(), "its flap lifted");
    assert!(g.vending[i].flap.rotation().x < -0.3, "held up from inside");
    run(&mut g, LIFT_T);
    assert!(g.vending[i].flap.rotation().x.abs() < 1e-9, "and let fall");
    assert!(print.visible() && print.mat().m().opacity > 0.99, "a hand on the inside of the glass");
    // once
    g.player.pos = Vec3::new(away.x, EYE, away.z);
    run(&mut g, 1.0);
    g.player.pos = Vec3::new(face.x, EYE, face.z);
    run(&mut g, INSIDE_WAIT + LIFT_T);
    assert_eq!(g.vending[i].state.lifted, t, "once");
    // and never from behind it: another dead one, met from behind its wall
    let (mut g, i) = level_with_dead(true);
    let back = g.vending[i].world(0.0, 0.0, -D / 2.0 - 0.6);
    g.player.pos = Vec3::new(back.x, EYE, back.z);
    g.player.yaw = g.vending[i].yaw() + PI;
    run(&mut g, INSIDE_WAIT + LIFT_T);
    assert!(g.vending[i].state.lifted.is_none(), "not through its wall");
    // and not every dead one: one with nothing in it stays still
    let (mut g, i) = level_with_dead(false);
    let face = stand_before(&g, i).unwrap();
    g.player.pos = Vec3::new(face.x, EYE, face.z);
    run(&mut g, INSIDE_WAIT + LIFT_T);
    assert!(g.vending[i].state.lifted.is_none(), "an empty one stays still");
}

#[test]
fn it_stands_where_it_belongs() {
    // none on the Lobby (its emptiness is the point), nor in the Showroom
    // or the Borrowed House (their light is their own); on other floors of
    // the rooms it does stand
    let mut g = common::new_game();
    let (mut lobby, mut store, mut house, mut elsewhere, mut stood) = (0, 0, 0, 0, 0);
    for k in 0..40u32 {
        g.run_seed = k.wrapping_mul(2654435761) ^ 0x7E2D;
        for level in 4..=12 {
            let scp = game::level::scp::scp_for_level(g.run_seed, level);
            let (count, want) = match (level, scp) {
                (4, _) if lobby < 3 => (&mut lobby, false),
                (_, Some("store")) if store < 1 => (&mut store, false),
                (_, Some("house")) if house < 1 => (&mut house, false),
                (5 | 6, None) if elsewhere < 3 => (&mut elsewhere, true),
                _ => continue,
            };
            g.init_level(level);
            if want {
                stood += usize::from(!g.vending.is_empty());
            } else {
                assert!(g.vending.is_empty(), "seed {} level {level} ({scp:?}) has a machine", g.run_seed);
            }
            *count += 1;
        }
        if lobby >= 3 && store >= 1 && house >= 1 && elsewhere >= 3 {
            assert!(stood >= 2, "it stands on the other floors: {stood} of 3");
            return;
        }
    }
    panic!("not every kind of floor met: lobby {lobby} store {store} house {house} elsewhere {elsewhere}");
}

#[test]
fn it_stands_where_an_office_or_a_hall_would_have_put_one() {
    // where it stands, it stands in an office, a hall or the Poolrooms (or
    // an office wing or a party room), never in a zone left bare (pillar 1)
    let mut g = common::new_game();
    let mut met = 0;
    for k in 0..8u32 {
        g.run_seed = k.wrapping_mul(2654435761) ^ 0x7E2D;
        for level in 5..=10 {
            g.init_level(level);
            let Some(l) = g.layout.as_ref() else { continue };
            for m in &g.vending {
                // the floor it stands on: the first open tile before its face
                // (moved onto a thin wall's face, it stands over the wall's
                // own tile, whose zone is a neighbour's)
                let (tx, tz) = (1..=12)
                    .map(|k| m.world(0.0, 0.0, D / 2.0 + 0.25 * k as f64))
                    .map(|p| (g.tx(p.x), g.tz(p.z)))
                    .find(|&(x, z)| !g.is_wall_tile(x, z))
                    .expect("open floor before it");
                let z = l.zone_of[g.t_idx(tx, tz)] as usize;
                let (arch, purpose) = (l.zones[z].arch, &g.zone_purposes[z]);
                let at = format!("seed {} level {level}: zone {z} {arch} {:?}", g.run_seed, purpose.label);
                assert!(purpose.primary.is_some(), "never in a zone left bare: {at}");
                assert!(["office", "hall", "pool"].contains(&arch) || purpose.has_any(&["office", "party"]), "its home: {at}");
                met += 1;
            }
        }
    }
    assert!(met >= 20, "machines met: {met}");
}
