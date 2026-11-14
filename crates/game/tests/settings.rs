//! Settings (`session/settings.rs`): the defaults are the game as it was,
//! stored values are clamped, each setting does what it says the moment it
//! changes, rebinding keeps every action reachable, and they persist.
use game::prelude::*;
use game::session::input::Act;
use game::session::settings::{FOV_DEFAULT, H_FOV_MAX, Row, SETTINGS_KEY, Settings, vertical_fov};

const DT: f64 = 1.0 / 60.0;

fn rig() -> Game {
    let (mut g, _) = Game::test_level(4242, 21);
    g.init_player();
    g.state = "playing".into();
    g.pointer_locked = true;
    g.hide_overlay();
    g.player.pos = Vec3::new(g.wx(2), 0.0, g.wz(2));
    g
}
fn step(g: &mut Game, n: usize) {
    for _ in 0..n {
        g.perf_t += DT;
        g.now_ms += DT * 1000.0;
        g.update_player(DT);
    }
}

#[test]
fn the_defaults_are_the_game_as_it_was() {
    let s = Settings::default();
    // (the field of view was 74 until wide screens stopped stretching the walls)
    assert_eq!((s.look, s.invert_y, s.fov, s.bob, s.shake, s.grain), (1.0, false, 64.0, 1.0, true, true));
    assert_eq!((s.run_toggle, s.volume, s.brightness, s.focus_pause), (false, 1.0, 0, true));
    assert_eq!(s.keys(Act::Drink), ["KeyG", "Digit1"]);
    // a round trip, and nonsense from the store is clamped or ignored
    assert_eq!(Settings::from_json(&s.to_json()), s);
    let bad = serde_json::json!({ "look": 40.0, "fov": 10.0, "volume": -3.0, "brightness": 99, "bob": "x", "binds": { "Drink": [], "Nope": ["KeyQ"] } });
    let t = Settings::from_json(&bad);
    assert_eq!((t.look, t.fov, t.volume, t.brightness, t.bob), (3.0, 50.0, 0.0, 5, 1.0));
    assert_eq!(t.keys(Act::Drink), ["KeyG", "Digit1"], "an empty binding from the store left Drink without a key");
}

#[test]
fn each_setting_does_what_it_says() {
    // look speed and invert
    let mut g = rig();
    g.mouse_move(100.0, 0.0);
    let base = g.player.yaw;
    g.player.yaw = 0.0;
    g.settings.look = 2.0;
    g.mouse_move(100.0, 0.0);
    assert!((g.player.yaw - 2.0 * base).abs() < 1e-12, "look speed 2 did not turn twice as far");
    g.player.pitch = 0.0;
    g.mouse_move(0.0, 50.0);
    let down = g.player.pitch;
    g.player.pitch = 0.0;
    g.settings.invert_y = true;
    g.mouse_move(0.0, 50.0);
    assert!((g.player.pitch + down).abs() < 1e-12 && down != 0.0, "invert did not flip the look");
    // field of view
    let mut g = rig();
    g.settings.fov = 88.0;
    step(&mut g, 120);
    assert!((g.camera.camera().fov - 88.0).abs() < 0.2, "field of view {}", g.camera.camera().fov);
    // head bob and camera shake off: walking, the eye stays level
    let mut g = rig();
    g.settings.bob = 0.0;
    g.settings.shake = false;
    g.shake_t = 1.0;
    g.key_down("KeyW", false);
    let mut ys = vec![];
    for _ in 0..90 {
        step(&mut g, 1);
        ys.push(g.camera.pos().y);
    }
    let (lo, hi) = ys.iter().fold((f64::MAX, f64::MIN), |(a, b), &y| (a.min(y), b.max(y)));
    assert!(hi - lo < 0.012, "the eye moved {:.3} m walking with head bob and shake off", hi - lo);
    let mut g = rig();
    g.key_down("KeyW", false);
    let mut ys = vec![];
    for _ in 0..90 {
        step(&mut g, 1);
        ys.push(g.camera.pos().y);
    }
    let (lo, hi) = ys.iter().fold((f64::MAX, f64::MIN), |(a, b), &y| (a.min(y), b.max(y)));
    assert!(hi - lo > 0.03, "full head bob moved the eye only {:.3} m", hi - lo);
    // press to run: a press runs until you stop
    let mut g = rig();
    g.settings.run_toggle = true;
    g.key_down("KeyW", false);
    g.key_down("ShiftLeft", false);
    g.key_up("ShiftLeft");
    step(&mut g, 5);
    assert!((g.player.audible_range - 13.0).abs() < 1e-9, "a press of the run key did not run");
    g.key_up("KeyW");
    step(&mut g, 2);
    g.key_down("KeyW", false);
    step(&mut g, 2);
    assert!((g.player.audible_range - 6.5).abs() < 1e-9, "the run outlasted a stop");
}

#[test]
fn rebinding_keeps_every_action_in_reach() {
    let mut s = Settings::default();
    s.rebind(Act::Drink, "KeyX");
    assert_eq!(s.keys(Act::Drink), ["KeyX", "Digit1"]);
    // a key another action had: the two swap
    s.rebind(Act::Drink, "KeyF");
    assert_eq!(s.keys(Act::Drink)[0], "KeyF");
    assert_eq!(s.keys(Act::Flashlight), ["KeyX", "Mouse2"]);
    s.rebind(Act::Paper, "KeyF");
    assert_eq!(s.keys(Act::Paper), ["KeyF"]);
    assert_eq!(s.keys(Act::Drink), ["KeyQ", "Digit1"], "Drink lost a key: {:?}", s.keys(Act::Drink));
    // a key it already has moves to the front
    s.rebind(Act::Drink, "Digit1");
    assert_eq!(s.keys(Act::Drink), ["Digit1", "KeyQ"]);
    for (a, k) in &s.binds {
        assert!(!k.is_empty(), "{a:?} has no key");
    }
    // in play: the new key drinks, the old one does not
    let mut g = rig();
    g.settings.rebind(Act::Drink, "KeyX");
    g.player.bottles = 2;
    g.player.stamina = 10.0;
    g.key_down("KeyG", false);
    assert_eq!(g.player.bottles, 2, "G still drinks after Drink moved to X");
    g.key_down("KeyX", false);
    assert_eq!(g.player.bottles, 1, "X does not drink");
}

#[test]
fn settings_persist_apart_from_the_save() {
    let mut g = rig();
    g.settings.step(Row::Volume, -1);
    g.settings.step(Row::Fov, 1);
    g.settings.rebind(Act::Call, "KeyZ");
    g.save_settings();
    let stored = g.storage_get(SETTINGS_KEY).expect("stored");
    g.clear_save();
    assert!(g.storage_get(SETTINGS_KEY).is_some(), "clearing the save took the settings with it");
    let mut h = rig();
    h.storage_set(SETTINGS_KEY, &stored);
    h.load_settings();
    assert_eq!(h.settings, g.settings);
    assert_eq!((h.settings.volume, h.settings.fov), (0.9, 66.0));
}

/// A wide screen sees further to the sides only up to 100°, and the
/// setting is the vertical view on a screen up to 16:9; a store from before
/// the cap that never moved the field of view (74) walks with the new one
#[test]
fn a_wide_screen_does_not_stretch_the_walls() {
    let across = |v: f64, aspect: f64| 2.0 * ((v.to_radians() / 2.0).dtan() * aspect).datan().to_degrees();
    for aspect in [4.0 / 3.0, 1.6, 16.0 / 9.0] {
        assert_eq!(vertical_fov(FOV_DEFAULT, aspect), FOV_DEFAULT, "at {aspect:.2} the setting is the view");
    }
    // 21:9 and 32:9: as wide as 100° and no wider
    for aspect in [2.39, 3.56] {
        let v = vertical_fov(FOV_DEFAULT, aspect);
        assert!((across(v, aspect) - H_FOV_MAX).abs() < 1e-6, "at {aspect:.2}: {:.1}° across", across(v, aspect));
        assert!(v < FOV_DEFAULT);
    }
    // a setting wider than the cap keeps what it gives at 16:9
    let at169 = across(90.0, 16.0 / 9.0);
    assert!((across(vertical_fov(90.0, 2.39), 2.39) - at169).abs() < 1e-6);
    // the camera follows it on a wide screen
    let mut g = rig();
    g.camera.camera().aspect = 2.39;
    step(&mut g, 120);
    assert!((across(g.camera.camera().fov, 2.39) - H_FOV_MAX).abs() < 0.5, "{:.1}° across", across(g.camera.camera().fov, 2.39));
    // the old default, never moved, becomes the new one; a chosen one stays
    assert_eq!(Settings::from_json(&serde_json::json!({ "fov": 74.0 })).fov, FOV_DEFAULT);
    assert_eq!(Settings::from_json(&serde_json::json!({ "fov": 80.0 })).fov, 80.0);
    assert_eq!(Settings::from_json(&serde_json::json!({ "fov": 74.0, "fovRev": 2 })).fov, 74.0);
}

#[test]
fn the_volume_and_the_pause_set_the_master_gain() {
    let mut g = rig();
    let ac = engine::audio::AudioContext::new_silent();
    g.audio_init_with(ac);
    let gain = |g: &Game| g.master.as_ref().unwrap().gain().value();
    assert!((gain(&g) - 0.9).abs() < 1e-6, "the default master gain moved: {}", gain(&g));
    g.settings.volume = 0.5;
    g.save_settings();
    assert!((gain(&g) - 0.45).abs() < 1e-6);
    g.pause();
    assert!((gain(&g) - 0.45 * 0.25).abs() < 1e-6, "the pause did not bring the sound down: {}", gain(&g));
    g.resume();
    assert!((gain(&g) - 0.45).abs() < 1e-6);
    g.set_muted(true);
    assert_eq!(gain(&g), 0.0);
}
