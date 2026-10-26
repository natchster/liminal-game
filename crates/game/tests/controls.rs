//! The controls (`session/input.rs`): what a run needs is in reach, a held
//! key fires once, running and using things go together, a spent sprint
//! comes back without a stutter, crouch holds or toggles, Escape pauses and
//! resumes, E uses what you face, the mouse buttons work in play, the
//! notebook pages without stealing a strafe, and the cards wait for their
//! key until they are up.
use game::prelude::*;
use game::session::input::{acts_for, Act, BINDINGS, LEFT_HAND};

const DT: f64 = 1.0 / 60.0;

fn rig() -> Game {
    let (mut g, _) = Game::test_level(4242, 21);
    g.init_player();
    g.state = "playing".into();
    g.pointer_locked = true;
    g.hide_overlay();
    // an open tile with room to run in
    g.player.pos = Vec3::new(g.wx(2), 0.0, g.wz(2));
    g
}
fn press(g: &mut Game, k: &str) {
    g.key_down(k, false);
}
fn release(g: &mut Game, k: &str) {
    g.key_up(k);
}
fn step(g: &mut Game, n: usize) {
    for _ in 0..n {
        g.perf_t += DT;
        g.now_ms += DT * 1000.0;
        g.run_scheduled();
        g.update_player(DT);
        g.update_hud(DT);
    }
}
/// sprinting this frame (the audible range of a run)
fn running(g: &Game) -> bool {
    (g.player.audible_range - 13.0).abs() < 1e-9
}

#[test]
fn what_a_run_needs_is_in_reach() {
    for b in &BINDINGS {
        assert!(!b.keys.is_empty(), "{:?} has no key", b.act);
        if b.on_the_run {
            assert!(
                b.keys.iter().any(|k| LEFT_HAND.contains(k)),
                "{:?} ({}) is needed while running but none of {:?} is under the left hand with W and Shift held, or on the mouse",
                b.act,
                b.label,
                b.keys
            );
        }
    }
    // no key does two things in play (the arrows' and the mouse buttons'
    // second meaning is the notebook's, only while it is up)
    let mut seen = std::collections::BTreeMap::new();
    for b in &BINDINGS {
        for k in b.keys {
            if let Some(other) = seen.insert(*k, b.act) {
                panic!("{k} is bound to both {other:?} and {:?}", b.act);
            }
        }
    }
    assert_eq!(acts_for("KeyE").collect::<Vec<_>>(), vec![Act::Interact]);
}

#[test]
fn a_held_key_fires_once() {
    let mut g = rig();
    g.player.battery = 80.0;
    press(&mut g, "KeyF");
    for _ in 0..20 {
        g.key_down("KeyF", true); // the OS key repeat
    }
    assert!(g.player.flashlight_on, "holding F strobed the flashlight");
    release(&mut g, "KeyF");
    press(&mut g, "KeyF");
    assert!(!g.player.flashlight_on);
    g.player.bottles = 3;
    g.player.stamina = 10.0;
    press(&mut g, "KeyG");
    for _ in 0..20 {
        g.key_down("KeyG", true);
    }
    assert_eq!(g.player.bottles, 2, "holding G drank more than one bottle");
}

#[test]
fn running_and_using_things_together() {
    let mut g = rig();
    g.player.bottles = 2;
    g.player.batteries = 2;
    g.player.papers = 5;
    g.player.battery = 30.0;
    press(&mut g, "KeyW");
    press(&mut g, "ShiftLeft");
    step(&mut g, 10);
    assert!(running(&g), "not running with W and Shift held");
    // every on-the-run action while running: the run goes on
    g.player.stamina = 40.0;
    for k in ["KeyG", "KeyR", "KeyQ", "KeyF", "Mouse2", "KeyE", "Mouse0", "KeyV", "KeyC"] {
        press(&mut g, k);
        step(&mut g, 1);
        release(&mut g, k);
        step(&mut g, 1);
        assert!(running(&g), "{k} stopped the run");
        g.player.stamina = g.player.stamina.max(40.0);
    }
    assert_eq!(g.player.bottles, 1);
    assert_eq!(g.player.batteries, 1);
    assert!(g.player.papers < 5);
    assert!(!g.player.crouch, "C while running crouched you (running stands you up)");
}

#[test]
fn a_spent_sprint_comes_back_without_a_stutter() {
    let mut g = rig();
    press(&mut g, "KeyW");
    press(&mut g, "ShiftLeft");
    // the running stretches while the run key stays held, and the stamina each began at
    let (mut stretches, mut cur, mut was, mut back_at) = (vec![], 0usize, true, None);
    for f in 0..60 * 14 {
        // keep inside the level: turn round now and then
        if f % 90 == 0 {
            g.player.yaw += PI;
        }
        step(&mut g, 1);
        let now = running(&g);
        if now {
            if !was && back_at.is_none() {
                back_at = Some(g.player.stamina);
            }
            cur += 1;
        } else if was {
            stretches.push(cur);
            cur = 0;
        }
        was = now;
    }
    // run out, walk until there is breath for a real run, run again: never a
    // run of a few frames (before: three frames of running every 0.9 s)
    assert!(stretches.len() >= 2, "the run never ran out and came back: {stretches:?}");
    assert!(stretches.iter().all(|&n| n >= 60), "the run stuttered (frames per running stretch): {stretches:?}");
    let back = back_at.expect("the run never came back");
    assert!(back >= game::session::input::SPRINT_BACK_AT - 1.0, "the run came back at {back:.1} stamina");
}

#[test]
fn crouch_holds_with_ctrl_and_toggles_with_c() {
    let mut g = rig();
    press(&mut g, "ControlLeft");
    step(&mut g, 1);
    assert!(g.player.crouch);
    release(&mut g, "ControlLeft");
    step(&mut g, 1);
    assert!(!g.player.crouch);
    press(&mut g, "KeyC");
    release(&mut g, "KeyC");
    step(&mut g, 30);
    assert!(g.player.crouch, "C did not keep you crouched");
    press(&mut g, "KeyC");
    release(&mut g, "KeyC");
    step(&mut g, 1);
    assert!(!g.player.crouch);
    // a run stands you up from a C crouch
    press(&mut g, "KeyC");
    release(&mut g, "KeyC");
    press(&mut g, "KeyW");
    press(&mut g, "ShiftLeft");
    step(&mut g, 2);
    assert!(!g.player.crouch && running(&g), "running did not stand you up");
}

#[test]
fn escape_pauses_and_resumes() {
    let mut g = rig();
    press(&mut g, "Escape");
    release(&mut g, "Escape");
    assert_eq!(g.state, "paused");
    assert_eq!(g.want_pointer_lock.take(), Some(false));
    assert_eq!(g.live_menu(), Some(game::ui::menu::Screen::Pause), "no pause list");
    // the window answers: the pointer is free
    g.pointer_lock_changed(false);
    assert_eq!(g.state, "paused");
    press(&mut g, "Escape");
    release(&mut g, "Escape");
    assert_eq!(g.state, "playing", "Escape did not resume");
    assert_eq!(g.want_pointer_lock.take(), Some(true));
    assert!(g.live_menu().is_none());
    g.pointer_lock_changed(true);
    // losing the window pauses; unless Settings says to keep the world running
    g.blur();
    assert_eq!(g.state, "paused");
    g.resume();
    g.pointer_lock_changed(true);
    g.settings.focus_pause = false;
    g.blur();
    g.pointer_lock_changed(false); // (the window reports it once more)
    assert_eq!(g.state, "playing", "the window was left and the world stopped, with Settings saying not to");
}

#[test]
fn e_uses_what_you_face() {
    let mut g = rig();
    g.interactables.clear();
    let p = g.player.pos;
    // a drawer ahead, a switch to the left, both in reach
    g.interact(Vec3::new(p.x, 0.0, p.z - 1.0), 1.6, "DRAWER", |_| true, |g| g.show_msg("drawer", 1.0));
    g.interact(Vec3::new(p.x - 1.0, 0.0, p.z), 1.6, "SWITCH", |_| true, |g| g.show_msg("switch", 1.0));
    g.player.yaw = 0.0; // facing -z
    step(&mut g, 1);
    assert_eq!(g.ui.prompt.text, "DRAWER");
    press(&mut g, "KeyE");
    release(&mut g, "KeyE");
    assert_eq!(g.msg_text, "drawer");
    g.player.yaw = PI / 2.0; // facing -x
    step(&mut g, 1);
    assert_eq!(g.ui.prompt.text, "SWITCH");
    press(&mut g, "Mouse0");
    release(&mut g, "Mouse0");
    assert_eq!(g.msg_text, "switch", "a left click did not use what you face");
    // the one behind you still counts when it is the only one
    g.interactables.remove(1);
    step(&mut g, 1);
    assert_eq!(g.ui.prompt.text, "DRAWER");
}

#[test]
fn an_early_e_lands_on_what_comes_into_reach() {
    let mut g = rig();
    g.interactables.clear();
    let p = g.player.pos;
    g.interact(Vec3::new(p.x, 0.0, p.z - 2.0), 1.0, "DOOR", |_| true, |g| g.show_msg("door", 1.0));
    g.player.yaw = 0.0;
    // E a step before the door is in reach, running at it
    press(&mut g, "KeyE");
    release(&mut g, "KeyE");
    assert_ne!(g.msg_text, "door");
    g.player.pos.z -= 1.05;
    g.update_input(DT);
    assert_eq!(g.msg_text, "door", "the early E was lost");
    // too early is too early
    g.show_msg("", 1.0);
    g.player.pos = p;
    press(&mut g, "KeyE");
    release(&mut g, "KeyE");
    for _ in 0..12 {
        g.update_input(DT);
    }
    g.player.pos.z -= 1.05;
    g.update_input(DT);
    assert_ne!(g.msg_text, "door", "an E from 0.2 s ago opened the door");
}

#[test]
fn letting_go_of_command_forgets_held_keys() {
    let mut g = rig();
    press(&mut g, "MetaLeft");
    press(&mut g, "KeyW"); // its key-up never comes while Command is held
    release(&mut g, "MetaLeft");
    assert!(!g.key("KeyW"), "W walks on by itself after a Command shortcut");
}

#[test]
fn the_notebook_pages_without_stealing_a_strafe() {
    let mut g = rig();
    g.lore_found.extend([1usize, 2, 3, 4]);
    press(&mut g, "Tab");
    release(&mut g, "Tab");
    assert!(g.journal_open, "Tab did not open the notebook");
    let c0 = g.journal_cursor;
    press(&mut g, "KeyA");
    step(&mut g, 10);
    release(&mut g, "KeyA");
    assert_eq!(g.journal_cursor, c0, "A turned a page (it strafes)");
    press(&mut g, "ArrowLeft");
    release(&mut g, "ArrowLeft");
    assert_eq!(g.journal_cursor, c0 - 1.0);
    g.wheel(40.0);
    assert_eq!(g.journal_cursor, c0, "the wheel did not turn the page");
    press(&mut g, "Mouse2");
    release(&mut g, "Mouse2");
    assert_eq!(g.journal_cursor, c0 - 1.0, "a right click did not turn back");
    assert!(!g.player.flashlight_on, "a right click in the notebook lit the flashlight");
    press(&mut g, "Escape");
    release(&mut g, "Escape");
    assert!(!g.journal_open && g.state == "playing", "Escape did not just close the notebook");
}

#[test]
fn the_cards_wait_until_they_are_up() {
    let mut g = rig();
    g.state = "dead".into();
    press(&mut g, "KeyE");
    release(&mut g, "KeyE");
    assert_eq!(g.state, "dead", "E skipped the death before its card was up");
    g.show_overlay("x", "y", "z");
    press(&mut g, "KeyE");
    release(&mut g, "KeyE");
    assert_eq!(g.state, "playing", "E did not wake you once the card was up");
}

#[test]
fn reading_keeps_the_view_and_the_walk() {
    let mut g = rig();
    g.lore_found.extend([1usize, 2]);
    press(&mut g, "Tab");
    release(&mut g, "Tab");
    assert!(g.journal_open);
    assert_eq!(g.want_pointer_lock, None, "opening the notebook took the mouse");
    let yaw = g.player.yaw;
    g.mouse_move(80.0, 0.0);
    assert!(g.player.yaw != yaw, "the view did not turn while reading");
    let p0 = g.player.pos;
    press(&mut g, "KeyW");
    step(&mut g, 30);
    release(&mut g, "KeyW");
    assert!(g.player.pos.distance_to(&p0) > 0.5, "you could not walk while reading");
    // leaving the window pauses, and the notebook goes back in the pocket
    g.blur();
    assert_eq!(g.state, "paused");
    assert!(!g.journal_open && !g.ui.journal.has_class("on"));
}
