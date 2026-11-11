//! Cutscenes (`docs/CUTSCENES.md`) on real floors. The player: a cutscene
//! begins where you were looking and its last frame is the gameplay camera
//! exactly; play never eases from its pose (a twin run without it draws the
//! same camera, frame for frame, and the same random numbers); your body
//! never moves; each cue runs once, even across a long frame; the pause
//! stops it; a skip lands where watching to the end would; nothing strikes
//! you; danger breaks it off; a held world holds the creatures.
mod common;

use game::cutscene::play::{Anchors, Frame, Pose, SKIP_HOLD};
use game::cutscene::Cutscene;
use game::Game;
use serde_json::{json, Value};

const DT: f64 = 1.0 / 60.0;
const FLOOR: (u32, i32) = (4242, 6);

/// A new run on a floor of the rooms, its creatures held where they are
fn start(seed: u32, level: i32) -> Game {
    game::base::rng::seed_native((u64::from(seed) << 8) | level as u64);
    let mut g = Game::new_bare();
    common::register_placers(&mut g.placers);
    g.launch_flags = format!("seed={seed}&level={level}&nosound");
    g.boot();
    g.start_new_game();
    g.god_mode = true;
    g.pointer_locked = true;
    // one frame of play: the camera at your eyes, as a cutscene finds it
    step(&mut g, DT);
    g
}

/// One frame of play at `dt`
fn step(g: &mut Game, dt: f64) {
    for e in g.entities.iter_mut() {
        e.freeze = 1e9;
    }
    g.now_ms += dt * 1000.0;
    g.fixed_step_dt = dt;
    g.frame(dt);
    g.fixed_step_dt = 0.0;
}

/// A cutscene as the bundle would hold it, with `over` laid on top
fn cut(over: Value) -> Cutscene {
    let mut v = json!({
        "id": "test", "about": "a test", "world": "live", "then": "back", "skip": "hold", "breaks_off": true,
        "length": 3.0, "blend_in": 0.0, "blend_out": 1.0, "repeat_from": 0.0, "stages": ["test"],
        "bars": { "in": 0.0, "out": 2.5 },
        "camera": [
            key(0.0, "start", [0.0, 0.0, 0.0], None),
            key(1.0, "thing", [0.0, 1.4, 0.9], Some([0.0, 1.0, 0.0])),
            key(2.0, "eyes", [0.3, 0.0, 0.5], None),
        ],
        "fade": [], "sound": [],
        "event": [ { "t": 0.5, "name": "mark", "on_skip": true }, { "t": 1.5, "name": "mark", "on_skip": true }, { "t": 2.5, "name": "mark", "on_skip": false } ],
        "say": [],
    });
    for (k, x) in over.as_object().unwrap() {
        v[k] = x.clone();
    }
    Cutscene::from_json(&v).unwrap_or_else(|e| panic!("{e}"))
}

fn key(t: f64, anchor: &str, pos: [f64; 3], look: Option<[f64; 3]>) -> Value {
    json!({ "t": t, "anchor": anchor, "pos": pos, "look": look, "look_at": null, "yaw": 0.0, "pitch": 0.0, "roll": 0.0,
            "fov": null, "ease": "in_out", "cut": false, "sway": 0.0, "jolt": false })
}

/// Two metres ahead of you, facing you
fn ahead(g: &Game) -> Anchors {
    let p = g.player.pos;
    let (s, c) = g.player.yaw.sin_cos();
    let at = [p.x - s * 2.0, g.floor_y_at(p.x, p.z), p.z - c * 2.0];
    Anchors { thing: Some(Frame::thing(at, g.player.yaw + std::f64::consts::PI)), killer: None }
}

fn marks(g: &Game) -> usize {
    g.cut_log.iter().filter(|l| l.starts_with("event mark")).count()
}

/// The frames of a run: what play drew (the gameplay camera) and what was
/// shown, and the native draws so far; with a cutscene from frame `at`
fn run(c: Option<Cutscene>, at: usize, frames: usize) -> Vec<(Pose, Pose, u64)> {
    let mut g = start(FLOOR.0, FLOOR.1);
    let draws = game::base::rng::native_draws();
    let mut out = vec![];
    for i in 0..frames {
        if i == at {
            if let Some(c) = c.clone() {
                let a = ahead(&g);
                assert!(g.play_cutscene_data(c, a));
            }
        }
        step(&mut g, DT);
        let shown = g.camera_pose();
        let play = g.cut.as_ref().and_then(|p| p.gameplay).unwrap_or(shown);
        out.push((play, shown, game::base::rng::native_draws() - draws));
    }
    out
}

#[test]
fn it_begins_where_you_look_and_ends_on_play_exactly() {
    let (at, frames) = (30, 30 + 200 + 30);
    // (a roll and a field of view of its own: play eases both from the
    // camera as it was, and must ease from its own)
    let mut k1 = key(1.0, "thing", [0.0, 1.4, 0.9], Some([0.0, 1.0, 0.0]));
    k1["roll"] = json!(8.0);
    k1["fov"] = json!(58.0);
    let c = cut(json!({ "camera": [key(0.0, "start", [0.0; 3], None), k1, key(2.0, "eyes", [0.3, 0.0, 0.5], None)] }));
    let with = run(Some(c), at, frames);
    let without = run(None, at, frames);
    let mut moved = 0.0f64;
    for i in 0..frames {
        let (play, shown, draws) = with[i];
        let (bare, _, bare_draws) = without[i];
        // play never eases from a cutscene's pose: its camera is the twin's
        assert_eq!(play, bare, "frame {i}: play's camera differs from a run without the cutscene");
        // and the cutscene draws nothing
        assert_eq!(draws, bare_draws, "frame {i}: the cutscene drew from the native stream");
        moved = moved.max(Pose::apart(&shown, &play).0);
        if i < at || i >= at + 180 {
            assert_eq!(shown, play, "frame {i}: outside the cutscene the camera is play's");
        }
    }
    assert!(moved > 0.5, "the camera went somewhere ({moved:.2} m)");
    // its first frame is where you were looking (it starts on the keys)
    let (first, before) = (with[at].1, with[at - 1].1);
    assert!(Pose::apart(&first, &before).0 < 0.02, "the first frame jumped: {first:?} from {before:?}");
}

#[test]
fn your_body_never_moves_and_the_mouse_turns_nothing() {
    let mut g = start(FLOOR.0, FLOOR.1);
    step(&mut g, DT);
    let (pos, yaw, pitch, crouch) = (g.player.pos, g.player.yaw, g.player.pitch, g.player.crouch);
    let a = ahead(&g);
    assert!(g.play_cutscene_data(cut(json!({})), a));
    g.key_down("KeyW", false);
    g.key_down("ShiftLeft", false);
    g.key_down("KeyC", false);
    for _ in 0..120 {
        g.mouse_move(40.0, -25.0);
        step(&mut g, DT);
    }
    assert!(g.hands_taken());
    assert_eq!((g.player.pos, g.player.yaw, g.player.pitch, g.player.crouch), (pos, yaw, pitch, crouch));
    // a key showed the way out
    assert!(g.cut.as_ref().unwrap().hint > 0.0);
    // ... and once the hands are back, W walks again
    for _ in 0..120 {
        step(&mut g, DT);
    }
    assert!(!g.hands_taken());
    for _ in 0..30 {
        step(&mut g, DT);
    }
    assert_ne!(g.player.pos, pos, "W held walks once the hands are back");
}

#[test]
fn each_cue_runs_once_even_across_a_long_frame() {
    let mut g = start(FLOOR.0, FLOOR.1);
    let a = ahead(&g);
    assert!(g.play_cutscene_data(cut(json!({})), a));
    // one frame of a second and a half: two marks came due in it
    step(&mut g, 1.6);
    assert_eq!(marks(&g), 2);
    step(&mut g, 0.05);
    assert_eq!(marks(&g), 2);
    for _ in 0..40 {
        step(&mut g, 0.05);
    }
    assert_eq!(marks(&g), 3);
    assert!(!g.hands_taken());
}

#[test]
fn the_pause_stops_it() {
    let mut g = start(FLOOR.0, FLOOR.1);
    let a = ahead(&g);
    assert!(g.play_cutscene_data(cut(json!({})), a));
    for _ in 0..30 {
        step(&mut g, DT);
    }
    g.key_down("Escape", false);
    g.key_up("Escape");
    assert_eq!(g.state, "paused");
    let t = g.cut.as_ref().unwrap().t;
    for _ in 0..120 {
        step(&mut g, DT);
    }
    assert_eq!(g.cut.as_ref().unwrap().t, t, "paused, its clock stands");
    g.key_down("Escape", false);
    g.key_up("Escape");
    assert_eq!(g.state, "playing");
    step(&mut g, DT);
    assert!(g.cut.as_ref().unwrap().t > t);
}

/// The skip key pressed at `at` seconds of `c`; → the game after its hands
/// are back, and how the skip went back to your eyes
fn skip_at(c: Cutscene, at: f64) -> (Game, String) {
    let mut g = start(FLOOR.0, FLOOR.1);
    let a = ahead(&g);
    assert!(g.play_cutscene_data(c, a));
    while g.cut.as_ref().unwrap().t < at {
        step(&mut g, DT);
    }
    g.key_down("KeyE", false);
    let mut frames = 0;
    while g.hands_taken() {
        step(&mut g, DT);
        frames += 1;
        assert!(frames < 120, "a skip ends it");
    }
    g.key_up("KeyE");
    let how = g.cut_log.iter().find(|l| l.starts_with("skipped")).cloned().unwrap_or_default();
    (g, how)
}

#[test]
fn a_skip_lands_where_play_lands() {
    // near your eyes (a cutscene that stays by them), and far from them (at the thing)
    let near = cut(json!({ "camera": [key(0.0, "start", [0.0; 3], None), key(2.0, "eyes", [0.2, -0.1, 0.2], None)] }));
    for (c, at, how) in [(near, 0.3, "blend"), (cut(json!({})), 1.0, "dip")] {
        let (mut g, said) = skip_at(c, at);
        assert!(said.contains(how), "skipped at {at}: {said}");
        // the events that hold on a skip ran, once; the one that does not, not
        assert_eq!(marks(&g), 2, "at {at}: {:?}", g.cut_log);
        // the camera is play's
        step(&mut g, DT);
        let shown = g.camera_pose();
        let p = g.player.pos;
        assert!((shown.pos[0] - p.x).abs() < 0.05 && (shown.pos[2] - p.z).abs() < 0.05, "at {at}: {shown:?} {p:?}");
        // a far skip opens on your eyes
        for _ in 0..30 {
            step(&mut g, DT);
        }
        assert_eq!(g.cut_veil.opacity, 0.0);
    }
    // held under the time, nothing happens
    let mut g = start(FLOOR.0, FLOOR.1);
    let a = ahead(&g);
    assert!(g.play_cutscene_data(cut(json!({})), a));
    g.key_down("KeyE", false);
    for _ in 0..((SKIP_HOLD / DT) as usize - 4) {
        step(&mut g, DT);
    }
    g.key_up("KeyE");
    for _ in 0..10 {
        step(&mut g, DT);
    }
    assert!(g.hands_taken() && !g.cut_log.iter().any(|l| l.starts_with("skipped")));
}

#[test]
fn nothing_strikes_while_your_hands_are_taken() {
    let mut g = start(FLOOR.0, FLOOR.1);
    g.god_mode = false;
    for _ in 0..60 {
        step(&mut g, DT);
    }
    let a = ahead(&g);
    assert!(g.play_cutscene_data(cut(json!({ "breaks_off": false })), a));
    let ki = (0..g.entities.len()).find(|&i| g.creature(i).is_some()).expect("a creature on the floor");
    assert!(!g.strike_allowed(ki));
    let before = (g.state.clone(), g.player.sanity);
    assert!(!g.creature_contact(ki));
    assert_eq!((g.state.clone(), g.player.sanity), before);
}

#[test]
fn something_hunting_you_breaks_it_off() {
    let mut g = start(FLOOR.0, FLOOR.1);
    let a = ahead(&g);
    assert!(g.play_cutscene_data(cut(json!({})), a));
    for _ in 0..20 {
        step(&mut g, DT);
    }
    let i = (0..g.entities.len()).find(|&i| g.creature(i).is_some()).unwrap();
    g.entities[i].state = "hunt".into();
    let mut frames = 0;
    while g.hands_taken() {
        step(&mut g, DT);
        frames += 1;
    }
    assert!(frames as f64 * DT < 0.4, "back in {frames} frames");
    assert!(g.cut_log.iter().any(|l| l.starts_with("broken off")));
    // nothing after the break ran
    assert_eq!(marks(&g), 0);
}

#[test]
fn a_held_world_holds_the_creatures_and_one_plays_at_a_time() {
    let mut g = start(FLOOR.0, FLOOR.1);
    let a = ahead(&g);
    assert!(g.play_cutscene_data(cut(json!({ "world": "held", "breaks_off": false })), a.clone()));
    assert!(!g.play_cutscene_data(cut(json!({})), a), "one at a time");
    for e in g.entities.iter_mut() {
        e.freeze = 0.0;
    }
    let at: Vec<_> = g.entities.iter().map(|e| e.pos()).collect();
    for _ in 0..60 {
        g.now_ms += DT * 1000.0;
        g.fixed_step_dt = DT;
        g.frame(DT);
    }
    let now: Vec<_> = g.entities.iter().map(|e| e.pos()).collect();
    assert_eq!(at, now, "a held world: nothing walks");
    // a thing it names must be given
    let mut h = start(FLOOR.0, FLOOR.1);
    assert!(!h.play_cutscene_data(cut(json!({})), Anchors::default()));
}

#[test]
fn a_death_ends_it_where_it_is() {
    let mut g = start(FLOOR.0, FLOOR.1);
    g.god_mode = false;
    let a = ahead(&g);
    assert!(g.play_cutscene_data(cut(json!({})), a));
    for _ in 0..30 {
        step(&mut g, DT);
    }
    g.on_death("sanity");
    assert_eq!(g.state, "dead");
    assert!(!g.hands_taken());
    assert!(g.cut_log.iter().any(|l| l.contains("(a death)")));
}
