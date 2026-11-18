//! The way in (`docs/design/WAY_IN.md`, `modes::tunnel`): New descent from
//! the title goes through the Jahamun Tunnel; its walkway has no end, and no
//! way back once the arch is lost; the escape door changes on walking time,
//! never in sight; her note goes into the notebook; the door leads onto the
//! first floor as a door does. Played headless, by keys, on the real place.
mod common;

use game::Game;
use game::modes::tunnel::{CHANGE_MAX, CHANGE_MIN, Near, change_after};

const DT: f64 = 1.0 / 30.0;

fn step(g: &mut Game) {
    g.now_ms += DT * 1000.0;
    g.fixed_step_dt = DT;
    g.frame(DT);
    g.fixed_step_dt = 0.0;
}

fn tunnel(g: &Game) -> &game::modes::tunnel::Tunnel {
    g.tunnel.as_deref().expect("in the tunnel")
}

/// A game at the title whose New descent goes through the tunnel, and the
/// descent begun.
fn through_the_tunnel(seed: u32) -> Game {
    let mut g = common::new_game_with(&format!("nosound&seed={seed}&prologue=1"));
    for _ in 0..30 {
        step(&mut g);
    }
    assert_eq!(g.state, "menu");
    g.menu_choose("menu:new");
    // the look into it (`tunnel_in`), then your hands
    let mut looked = false;
    for _ in 0..30 * 40 {
        step(&mut g);
        looked |= g.hands_taken();
        if looked && !g.hands_taken() {
            break;
        }
    }
    assert!(looked && !g.hands_taken(), "the look into the tunnel played, and gave your hands back: {:?}", g.cut_log);
    g
}

#[test]
fn new_descent_goes_through_the_tunnel_and_its_door_onto_the_first_floor() {
    let mut g = through_the_tunnel(4242);
    // in the tunnel (your hands back after the look into it), the run made
    // behind it: its first floor built, hidden
    assert!(g.in_tunnel && g.state == "playing", "{} {}", g.in_tunnel, g.state);
    assert_eq!((g.run_seed, g.level_num), (4242, 1));
    assert!(!g.world.visible(), "the floor behind is hidden");
    assert!(g.way_in_card.is_some(), "the floor's card is kept for the door");
    assert!(g.level_card.is_none());
    assert!(g.saved.is_none(), "the way in keeps no save");
    // your hands come back where the look ends: a few bays in
    let start = game::modes::tunnel::START;
    assert_eq!(tunnel(&g).feet, start, "the look moved nothing of yours");
    // walk on (left, by the wall) until the door has changed, then to it
    g.key_down("KeyA", false);
    for _ in 0..20 {
        step(&mut g);
    }
    g.key_up("KeyA");
    let mut frames = 0;
    g.key_down("KeyW", false);
    while !tunnel(&g).set.door_changed() {
        step(&mut g);
        frames += 1;
        assert!(frames < 30 * 200, "the door changes within a minute or two of walking and a lap");
    }
    g.key_up("KeyW");
    let t = tunnel(&g);
    assert!(t.walked >= t.change_after, "{} < {}", t.walked, t.change_after);
    assert!(g.saved.is_none() && g.level_card.is_none());
    // on to her note (the next door ahead, its note just short of it)
    let z = t.feet[2];
    let door = t.set.doors().filter(|d| *d < z - 1.0).fold(f64::MIN, f64::max);
    let note = door + jahamun::NOTE_FROM_DOOR;
    g.key_down("KeyW", false);
    while tunnel(&g).feet[2] > note {
        step(&mut g);
        frames += 1;
        assert!(frames < 30 * 260, "her note is reached");
    }
    g.key_up("KeyW");
    step(&mut g);
    assert_eq!(tunnel(&g).near, Near::Note, "at her note: {:?}", tunnel(&g).feet);
    assert_eq!(g.ui.prompt.text, "[E] TAKE HER NOTE");
    g.key_down("KeyE", false);
    step(&mut g);
    g.key_up("KeyE");
    assert!(g.notebook.contains("door"), "her note is in the notebook");
    assert_eq!(g.thread, "I'm going after him.");
    let ids = g.journal_entries_in_order();
    assert_eq!(ids.get(1), Some(&game::ui::hud::notebook_journal_id(0)), "her note first in the notebook: {ids:?}");
    // to the door, and through it
    g.key_down("KeyW", false);
    while tunnel(&g).feet[2] > door + 0.3 {
        step(&mut g);
    }
    g.key_up("KeyW");
    step(&mut g);
    assert_eq!(tunnel(&g).near, Near::Door);
    assert_eq!(g.ui.prompt.text, "[E] GO THROUGH");
    g.key_down("KeyE", false);
    step(&mut g);
    g.key_up("KeyE");
    assert_eq!(g.state, "transition");
    for _ in 0..90 {
        step(&mut g);
    }
    // the first floor, entered as through a door: built, saved, its card
    assert!(!g.in_tunnel && g.tunnel.is_none());
    assert_eq!((g.state.as_str(), g.level_num), ("playing", 1));
    assert!(g.world.visible());
    assert!(g.saved.is_some(), "the door saves, as a door does");
    assert!(g.level_card.is_some(), "the floor's card");
    assert!(g.notebook.contains("door"), "her note goes with you");
    assert!(g.scene.children().iter().all(|o| o.name() != "jahamun"), "the place is let go");
}

#[test]
fn the_walkway_has_no_end_and_once_the_arch_is_lost_no_way_back() {
    use jahamun::ring::{far_seam, near_seam};
    let mut g = through_the_tunnel(7);
    // on, a long way: more than the tunnel is long
    g.key_down("KeyW", false);
    let mut deepest = 0.0f64;
    for _ in 0..30 * 190 {
        step(&mut g);
        let t = tunnel(&g);
        assert!(t.feet[2] > far_seam() - 1.0 && t.feet[2] < 0.5, "built ground: {:?}", t.feet);
        deepest = deepest.max(t.along);
    }
    g.key_up("KeyW");
    assert!(deepest > 600.0, "walked on {deepest} m");
    assert!(tunnel(&g).set.ring.closed);
    // back, as far again: the arch never comes
    g.key_down("KeyS", false);
    for _ in 0..30 * 190 {
        step(&mut g);
        assert!(tunnel(&g).feet[2] < near_seam() + 0.5, "the arch came back: {:?}", tunnel(&g).feet);
    }
    g.key_up("KeyS");
    assert!(tunnel(&g).back > 500.0, "walked back {} m", tunnel(&g).back);
}

#[test]
fn the_door_changes_on_walking_time_and_never_in_sight() {
    // the run's own time, 60-120 s, different from run to run
    let times: Vec<f64> = (1..=40).map(change_after).collect();
    assert!(times.iter().all(|t| (CHANGE_MIN..CHANGE_MAX).contains(t)));
    assert!(times.windows(2).any(|w| (w[0] - w[1]).abs() > 5.0));
    let mut g = through_the_tunnel(9001);
    // standing still changes nothing
    for _ in 0..30 * 150 {
        step(&mut g);
    }
    assert!(!tunnel(&g).set.door_changed(), "the door's time is walking time");
    // walking, it changes, and at that moment no copy of it can be seen
    g.key_down("KeyW", false);
    let mut frames = 0;
    loop {
        let before = tunnel(&g).set.door_changed();
        step(&mut g);
        frames += 1;
        let t = tunnel(&g);
        if !before && t.set.door_changed() {
            assert!(t.walked >= t.change_after);
            let eye = engine::prelude::Vec3::new(t.feet[0], t.feet[1] + game::base::config::EYE, t.feet[2]);
            assert!(t.door_unseen(eye, g.player.yaw), "changed in sight: {:?}", t.feet);
            break;
        }
        assert!(frames < 30 * 200, "it changes");
    }
}

#[test]
fn a_run_on_a_chosen_seed_or_level_begins_on_its_floor() {
    for (flags, wanted) in [
        ("nosound", true),
        ("nosound&seed=4242", false),
        ("nosound&level=3", false),
        ("nosound&seed=4242&prologue=1", true),
        ("nosound&prologue=0", false),
    ] {
        let g = common::new_game_with(flags);
        assert_eq!(g.way_in_wanted(), wanted, "{flags}");
    }
    let mut g = common::new_game_with("nosound&seed=4242");
    g.menu_choose("menu:new");
    assert!(!g.in_tunnel, "the tools' and tests' runs keep the old look from the title");
}

#[test]
fn a_new_floor_leaves_the_tunnel_cleanly() {
    let mut g = through_the_tunnel(4242);
    for _ in 0..10 {
        step(&mut g);
    }
    g.init_level(1);
    assert!(!g.in_tunnel && g.tunnel.is_none() && g.world.visible());
    assert!(g.scene.children().iter().all(|o| o.name() != "jahamun"));
}
