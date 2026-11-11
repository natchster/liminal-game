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
        "length": 3.0, "blend_in": 0.0, "blend_out": 1.0, "repeat_from": 0.0, "stages": ["test"], "finally": [],
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
            "fov": 0.0, "ease": "in_out", "cut": false, "sway": 0.0, "walk": 0.0, "jolt": false })
}

/// Two metres ahead of you, facing you
fn ahead(g: &Game) -> Anchors {
    let p = g.player.pos;
    let (s, c) = g.player.yaw.sin_cos();
    let at = [p.x - s * 2.0, g.floor_y_at(p.x, p.z), p.z - c * 2.0];
    Anchors { thing: Some(Frame::thing(at, g.player.yaw)), killer: None }
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
    k1["fov"] = json!(-6.0);
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
    let far = cut(
        json!({ "camera": [key(0.0, "start", [0.0; 3], None), key(1.0, "thing", [0.0, 1.5, -1.0], None), key(2.0, "eyes", [0.3, 0.0, 0.5], None)] }),
    );
    for (c, at, how) in [(near, 0.3, "blend"), (far, 0.6, "dip")] {
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
    // what must hold of it held (the two marks that hold on a skip), and
    // nothing else after the break ran
    assert_eq!(marks(&g), 2, "{:?}", g.cut_log);
    assert!(!g.cut_log.iter().any(|l| l == "event mark at 2.50"));
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

/// The HUD's layers this frame, by name
fn layers(g: &mut Game) -> Vec<&'static str> {
    let _ = g.hud_layers(1280.0, 800.0, 1.0);
    g.hud_ui.painted.clone()
}

#[test]
fn the_bars_come_in_the_hud_goes_and_a_key_shows_the_way_out() {
    let mut g = start(FLOOR.0, FLOOR.1);
    // something for the HUD to say: out of breath, a line
    g.player.stamina = 10.0;
    g.show_msg("A line", 30.0);
    for _ in 0..10 {
        step(&mut g, DT);
    }
    let before = layers(&mut g);
    assert!(before.contains(&"msg") && !before.iter().any(|n| n.starts_with("cut-")), "{before:?}");
    let a = ahead(&g);
    assert!(g.play_cutscene_data(cut(json!({})), a));
    for _ in 0..30 {
        step(&mut g, DT);
    }
    let during = layers(&mut g);
    assert!(during.contains(&"cut-bar-top") && during.contains(&"cut-bar-bottom"), "{during:?}");
    // what the HUD painted is gone (the place's own: the vignette, grain, fills, stays)
    let hud: Vec<_> = before.iter().filter(|n| !["vignette", "radial", "fill", "grain"].contains(n)).collect();
    assert!(!hud.is_empty());
    for n in hud {
        assert!(!during.contains(n), "{n} is still up during a cutscene: {during:?}");
    }
    // no words until a key
    assert!(!during.contains(&"cut-way-out"));
    g.key_down("KeyW", false);
    step(&mut g, DT);
    assert!(layers(&mut g).contains(&"cut-way-out"));
    g.key_up("KeyW");
    while g.hands_taken() {
        step(&mut g, DT);
    }
    let after = layers(&mut g);
    assert!(!after.iter().any(|n| n.starts_with("cut-")), "{after:?}");
    assert!(after.contains(&"msg"), "the HUD is back");
}

// ------------------------------------------------------------ the contract
// Every cutscene of the library, on each of its stages, keeps the contract
// (`game::debug::cutscene`): played, skipped early and late, paused, and at
// 30, 60 and 120 frames a second; and the contract catches what it is for
// (known answers: broken cutscenes and tampered runs, each named).

use game::cutscene::Skip;
use game::debug::cutscene::{contract, contract_rates, contract_skip, RunOpts, Trace};

/// A run of `c` on a fresh `stage`
fn trace(c: &Cutscene, stage: &str, dt: f64, skip_at: Option<f64>, pause_at: Option<f64>) -> Trace {
    let mut g = common::new_game();
    let s = g.cutscene_stage(stage).unwrap_or_else(|e| panic!("{e}"));
    g.run_cutscene(c, &s, &RunOpts { dt, skip_at, pause_at, danger_at: None }, &mut |_, _| {})
}

/// Everything the contract asks of `c` on `stage`
fn keeps(c: &Cutscene, stage: &str) -> Vec<String> {
    let played = trace(c, stage, DT, None, None);
    let mut p = contract(c, &played);
    if c.skip == Skip::Hold {
        for frac in [0.2, 0.6] {
            let at = c.repeat_from.max(0.0) + (c.length - c.repeat_from) * frac;
            if at + game::cutscene::play::SKIP_HOLD > c.length - 0.1 {
                continue;
            }
            let skipped = trace(c, stage, DT, Some(at), None);
            p.extend(contract(c, &skipped));
            p.extend(contract_skip(&played, &skipped));
        }
    }
    // something comes for you part-way: a live one breaks off, and what
    // must hold of it holds
    if c.breaks_off {
        let mut g = common::new_game();
        let s = g.cutscene_stage(stage).unwrap_or_else(|e| panic!("{e}"));
        let at = c.repeat_from + (c.length - c.repeat_from) * 0.4;
        let broken = g.run_cutscene(c, &s, &RunOpts { danger_at: Some(at), ..Default::default() }, &mut |_, _| {});
        p.extend(contract(c, &broken));
        if !broken.log.iter().any(|l| l.starts_with("broken off")) {
            p.push(format!("{} on {stage}: something hunting you did not break it off", c.id));
        }
    }
    let paused = trace(c, stage, DT, None, Some(c.length * 0.5));
    p.extend(contract(c, &paused));
    if paused.signature != played.signature || paused.rows.last().map(|r| r.t) != played.rows.last().map(|r| r.t) {
        p.push(format!("{} on {stage}: a pause changed where it ends ({} vs {})", c.id, paused.signature, played.signature));
    }
    let rates: Vec<Trace> = [1.0 / 30.0, 1.0 / 60.0, 1.0 / 120.0].iter().map(|&dt| trace(c, stage, dt, None, None)).collect();
    // (a live killer moves by its own animation, a step at a time)
    let live = c
        .camera
        .iter()
        .any(|k| k.anchor == game::cutscene::Anchor::Killer || k.look == game::cutscene::Look::At(game::cutscene::Anchor::Killer));
    p.extend(contract_rates(&rates, if live { 0.05 } else { 1e-6 }));
    p
}

#[test]
fn every_cutscene_keeps_the_contract_on_each_of_its_stages() {
    let mut p = vec![];
    for c in game::cutscene::library() {
        for s in &c.stages {
            p.extend(keeps(c, s));
        }
    }
    assert!(p.is_empty(), "{p:#?}");
}

#[test]
fn the_contract_catches_what_it_is_for() {
    // a good one keeps it all
    let good = cut(json!({}));
    let p = keeps(&good, "ahead");
    assert!(p.is_empty(), "{p:#?}");
    // broken cutscenes
    let turned = |yaw: f64, roll: f64, fov: f64| {
        let mut k = key(1.0, "eyes", [0.0; 3], None);
        k["yaw"] = json!(yaw);
        k["roll"] = json!(roll);
        k["fov"] = json!(fov);
        cut(json!({ "camera": [key(0.0, "start", [0.0; 3], None), k, key(2.0, "eyes", [0.0; 3], None)] }))
    };
    for (c, says) in [
        (turned(170.0, 0.0, 0.0), "turns at"),
        (turned(0.0, 20.0, 0.0), "rolls"),
        (turned(0.0, 0.0, -25.0), "field of view"),
        (
            cut(
                json!({ "camera": [key(0.0, "start", [0.0; 3], None), key(0.5, "eyes", [0.0, 0.0, -4.0], None), key(2.0, "eyes", [0.0; 3], None)] }),
            ),
            "moves at",
        ),
    ] {
        let p = contract(&c, &trace(&c, "ahead", DT, None, None));
        assert!(p.iter().any(|x| x.contains(says)), "{says}: {p:#?}");
    }
    // a jolt declared lets a fast turn through
    let mut jolted = turned(170.0, 0.0, 0.0);
    jolted.camera[1].jolt = true;
    jolted.camera[2].jolt = true;
    let p = contract(&jolted, &trace(&jolted, "ahead", DT, None, None));
    assert!(!p.iter().any(|x| x.contains("turns at")), "{p:#?}");
    // tampered runs
    let tr = trace(&good, "ahead", DT, None, None);
    let tamper = |f: &dyn Fn(&mut Trace)| {
        let mut t = tr.clone();
        f(&mut t);
        t
    };
    for (t, says) in [
        (tamper(&|t| t.rows[10].body.0[0] += 1.0), "your body moved"),
        (tamper(&|t| t.rows.last_mut().unwrap().shown.pos[0] += 0.01), "not play's camera"),
        (tamper(&|t| t.rows[5].engine_draws = 3), "the engine drew"),
        (tamper(&|t| t.rows.last_mut().unwrap().hands = true), "never gave your hands back"),
        (tamper(&|t| t.rows[3].hands = false), "came back before its end"),
    ] {
        let p = contract(&good, &t);
        assert!(p.iter().any(|x| x.contains(says)), "{says}: {p:#?}");
    }
    let wrong = tamper(&|t| {
        t.signature = "elsewhere".into();
        t.log.push("skipped at 1.00 (blend)".into());
    });
    assert!(contract_skip(&tr, &wrong).iter().any(|x| x.contains("a skip leaves")));
    assert!(contract_skip(&tr, &tr).iter().any(|x| x.contains("did not skip")));
    let drifted = tamper(&|t| t.rows[20].shown.yaw += 0.01);
    assert!(contract_rates(&[tr.clone(), drifted], 1e-6).iter().any(|x| x.contains("the camera at")));
}

// ------------------------------------------------------------ review A
// What the code and design reviews after C4 found, each held here.

#[test]
fn a_focus_lost_mid_hold_does_not_skip() {
    let mut g = start(FLOOR.0, FLOOR.1);
    let a = ahead(&g);
    assert!(g.play_cutscene_data(cut(json!({})), a));
    g.key_down("KeyE", false);
    for _ in 0..10 {
        step(&mut g, DT);
    }
    // the window loses focus: the keys are forgotten with no key-up
    g.blur();
    g.pointer_locked = true;
    for _ in 0..60 {
        step(&mut g, DT);
    }
    assert!(!g.cut_log.iter().any(|l| l.starts_with("skipped")), "{:?}", g.cut_log);
}

#[test]
fn an_early_end_in_a_fade_leaves_the_view_clear() {
    // a near skip, and a break-off, while its veil is half down
    let fading = || {
        cut(json!({
            "camera": [key(0.0, "start", [0.0; 3], None), key(2.0, "eyes", [0.2, -0.1, 0.2], None)],
            "fade": [ { "t": 0.2, "to": 0.8, "over": 1.0, "colour": "#000" }, { "t": 2.2, "to": 0.0, "over": 0.3, "colour": "#000" } ],
        }))
    };
    for danger in [false, true] {
        let mut g = start(FLOOR.0, FLOOR.1);
        let a = ahead(&g);
        assert!(g.play_cutscene_data(fading(), a));
        if danger {
            for _ in 0..40 {
                step(&mut g, DT);
            }
            g.stage_hunt();
        } else {
            g.key_down("KeyE", false);
        }
        while g.hands_taken() {
            step(&mut g, DT);
        }
        g.key_up("KeyE");
        for _ in 0..30 {
            step(&mut g, DT);
        }
        assert_eq!(g.cut_veil.opacity, 0.0, "danger {danger}: {:?}", g.cut_log);
    }
}

#[test]
fn danger_refuses_one_that_gives_your_hands_back() {
    let mut g = start(FLOOR.0, FLOOR.1);
    g.stage_hunt();
    let a = ahead(&g);
    assert!(!g.play_cutscene_data(cut(json!({})), a));
    assert!(g.cut_log.iter().any(|l| l.ends_with("danger")), "{:?}", g.cut_log);
}

#[test]
fn a_blend_in_starts_from_the_camera_as_it_was() {
    let mut g = start(FLOOR.0, FLOOR.1);
    // the camera somewhere play does not draw it (a title, a death)
    let mut away = g.camera_pose();
    away.pos[1] += 1.0;
    away.yaw += 0.8;
    away.roll = 0.05;
    g.set_camera_pose(&away);
    let a = ahead(&g);
    assert!(g.play_cutscene_data(cut(json!({ "blend_in": 0.6 })), a));
    step(&mut g, DT);
    let (d, turn) = Pose::apart(&away, &g.camera_pose());
    assert!(d < 0.02 && turn < 0.02, "the first frame left the camera as it was by {d:.3} m, {turn:.3} rad");
}

#[test]
fn nothing_told_while_your_hands_are_taken_counts() {
    let mut g = start(FLOOR.0, FLOOR.1);
    let ki = (0..g.entities.len()).find(|&i| g.creature(i).is_some()).unwrap();
    let a = ahead(&g);
    assert!(g.play_cutscene_data(cut(json!({ "breaks_off": false })), a));
    let before = g.creature(ki).unwrap().tells.len();
    g.creature_tell(ki, 1e6);
    assert_eq!(g.creature(ki).unwrap().tells.len(), before);
}

#[test]
fn a_death_plays_to_its_card_and_holds_the_camera() {
    let mut g = start(FLOOR.0, FLOOR.1);
    g.god_mode = false;
    g.on_death("sanity");
    let c = cut(
        json!({ "then": "card", "skip": "never", "breaks_off": false, "length": 1.2, "blend_out": 0.0, "bars": { "in": 0.3, "out": 1.2 },
        "camera": [key(0.0, "start", [0.0; 3], None), key(1.0, "eyes", [0.0, -1.0, 0.3], None)],
        "fade": [ { "t": 0.6, "to": 1.0, "over": 0.5, "colour": "#000" } ], "event": [] }),
    );
    assert!(g.play_cutscene_data(c, Anchors::default()));
    while g.hands_taken() {
        step(&mut g, DT);
    }
    let held = g.camera_pose();
    assert!(g.live_menu().is_some(), "the card is up at its end");
    for _ in 0..30 {
        step(&mut g, DT);
    }
    assert_eq!(g.camera_pose(), held, "the camera holds behind the card");
    assert_eq!(g.cut_veil.opacity, 1.0, "its black stays behind the card");
}

// ------------------------------------------------------------ the way in
// New descent from the title (C5): one shot from the title's view into
// your eyes, in the same place, the card after.

/// A game booted to its title (a fixed boot), the title live for a second
fn at_the_title(native: u64) -> Game {
    game::base::rng::seed_native(native);
    let mut g = common::new_game();
    for _ in 0..60 {
        g.now_ms += DT * 1000.0;
        g.fixed_step_dt = DT;
        g.frame(DT);
    }
    g
}

#[test]
fn new_descent_draws_the_title_into_your_eyes_with_no_cut() {
    for native in [0x7171u64, 0x5EED_5EED, 99] {
        let mut g = at_the_title(native);
        assert_eq!(g.state, "menu");
        let (seed, title, view) = (g.run_seed, g.camera_pose(), g.title_view());
        g.menu_choose("menu:new");
        assert!(g.hands_taken(), "{native:x}: the way in plays: {:?}", g.cut_log);
        // the same place: the title's run, facing the way the title looked
        assert_eq!(g.run_seed, seed, "{native:x}: New descent keeps the title's place");
        assert_eq!(g.player.yaw, view.1);
        assert!(g.level_card.is_none(), "{native:x}: the card waits for the end");
        let (mut last, mut worst) = (title, (0.0f64, 0.0f64));
        let mut frames = 0;
        while g.hands_taken() {
            assert!(frames < 600, "{native:x}: the way in ends");
            step(&mut g, DT);
            let now = g.camera_pose();
            let (d, turn) = Pose::apart(&last, &now);
            worst = (worst.0.max(d), worst.1.max(turn));
            last = now;
            frames += 1;
            if g.hands_taken() {
                assert!(g.level_card.is_none(), "{native:x}: the card waits for the end");
            }
        }
        for _ in 0..30 {
            step(&mut g, DT);
            let now = g.camera_pose();
            let (d, turn) = Pose::apart(&last, &now);
            worst = (worst.0.max(d), worst.1.max(turn));
            last = now;
        }
        // no frame from the title to your hands and after jumps: under a
        // walk's step and a slow turn a frame
        assert!(worst.0 < 0.04 && worst.1 < 0.03, "{native:x}: a frame jumped {:.3} m, {:.3} rad", worst.0, worst.1);
        assert!(g.level_card.is_some(), "{native:x}: the card after");
        assert_eq!(g.state, "playing");
        // the camera moved from the title to your eyes
        let (d, _) = Pose::apart(&title, &g.camera_pose());
        assert!(d > 1.0, "{native:x}: the title stood back from your eyes ({d:.2} m)");
        // your first arrival is still to be said
        assert!(g.asides.arrival.is_some() || g.asides.log.iter().any(|(_, l)| l.contains("arrive")), "{native:x}");
    }
}

#[test]
fn new_descent_over_another_runs_floor_goes_through_black() {
    let mut g = at_the_title(0x7171);
    g.start_new_game();
    for _ in 0..30 {
        step(&mut g, DT);
    }
    let old = g.run_seed;
    g.pause();
    g.leave_for_title();
    assert_eq!(g.state, "menu");
    assert!(g.title_place.is_none());
    g.new_descent();
    assert_eq!(g.ui.fade.opacity(), Some(1.0));
    for _ in 0..60 {
        g.now_ms += DT * 1000.0;
        g.fixed_step_dt = DT;
        g.frame(DT);
    }
    assert!(g.hands_taken(), "{:?}", g.cut_log);
    assert_ne!(g.run_seed, old, "a new run");
    while g.hands_taken() {
        step(&mut g, DT);
    }
    assert!(g.level_card.is_some());
}

// ------------------------------------------------------------ the shaft
// A broken lift's shaft (C6): looked into once, through the cutscene; it
// leaves nothing behind but what it did to you; in danger it waits.

#[test]
fn the_shaft_is_looked_into_once_and_leaves_nothing_behind() {
    for (seed, level) in [(4242, 4), (9001, 7)] {
        let mut g = common::new_game();
        g.cutscene_stage(&format!("broken_elevator:{seed}:{level}")).unwrap();
        let nerve = g.player.sanity;
        assert!(g.interact_target().is_some(), "{seed}:{level}: its prompt, where you stand");
        g.msg_text.clear();
        g.try_interact();
        assert!(g.hands_taken(), "{seed}:{level}: {:?}", g.cut_log);
        let eyes = |g: &Game| g.broken_elevator.as_ref().is_some_and(|b| b.objs.contains_key("eyes"));
        let (mut saw, mut lowest) = (false, nerve);
        while g.hands_taken() {
            step(&mut g, DT);
            saw |= eyes(&g);
            lowest = lowest.min(g.player.sanity);
        }
        assert!(saw, "{seed}:{level}: something looked back");
        assert!(!eyes(&g), "{seed}:{level}: nothing left in the black");
        assert!(g.broken_elevator.as_ref().unwrap().peered);
        assert!((nerve - lowest - 12.0).abs() < 0.5, "{seed}:{level}: the look shakes your nerve ({nerve} → {lowest})");
        // (the old look told you in capitals what you had seen)
        for told in ["SOMETHING WAS DOWN THERE.", "IT LOOKED BACK.", "YOU SHOULD NOT HAVE LOOKED.", "THE DARK BREATHED."] {
            assert_ne!(g.msg_text, told, "{seed}:{level}: no line tells you what you saw");
        }
        // it is not offered again
        g.try_interact();
        assert!(!g.hands_taken());
    }
    // a second lift in a run: the black and the breath, and nothing looks back
    let mut g = common::new_game();
    g.cutscene_stage("broken_elevator:9001:7").unwrap();
    g.cut_seen.insert("shaft".into(), 1);
    g.try_interact();
    assert!(g.hands_taken());
    let mut saw = false;
    while g.hands_taken() {
        step(&mut g, DT);
        saw |= g.broken_elevator.as_ref().is_some_and(|b| b.objs.contains_key("eyes"));
    }
    assert!(!saw, "the same scare, soon again");
    assert!(g.broken_elevator.as_ref().unwrap().peered);
    // in danger nothing happens, and the lift waits
    let mut g = common::new_game();
    g.cutscene_stage("broken_elevator:4242:4").unwrap();
    g.stage_hunt();
    g.try_interact();
    assert!(!g.hands_taken() && !g.broken_elevator.as_ref().unwrap().peered, "{:?}", g.cut_log);
}
