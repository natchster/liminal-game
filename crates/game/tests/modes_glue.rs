//! The Stairwell / Loop glue (`game::modes`) against recorded reference data
//! (fixtures in `tests/fixtures/`).
//!
//! Each scenario is replayed frame by frame from the recorded inputs (keys,
//! mouse look, debug actions, dt); after every frame the calls the glue made
//! (game::modes::trace_take) and, on the sampled frames, every value it
//! writes are compared with the recording.
#[path = "json_common/mod.rs"]
mod common;
use common::*;
use engine::prelude::*;
use game::base::records::Entity;
use game::base::rng::{rnd, seeded_draws, with_seed, Mulberry32};
use game::modes::stairwell::{game_canvas_tex, StairDeps, Stairwell};
use game::modes::*;
use game::Game;
use std::cell::{Cell, RefCell};
use std::rc::Rc;

const TOL: f64 = 1e-9;
/// The recorded key mask's bits. Bit 5 is the reference's C, which crouched
/// while held; the held crouch is Ctrl now (C toggles, `session/input.rs`),
/// so a recorded C is replayed as Ctrl held: the same crouch, frame for frame.
const KEYS: [&str; 8] = ["KeyW", "KeyS", "KeyA", "KeyD", "ShiftLeft", "ControlLeft", "ControlLeft", "ArrowUp"];
/// the per-frame sound updates the fixture keeps only on sampled frames
const BORING: [&str; 7] = ["ss.set_listener", "slot.at", "voice.set_level", "hum.update", "hum.silence", "hum.detune", "ss.set_zone:loop"];
const BORING_SRC: &str = r"^(ss\.setListener|slot\.at|voice\.setLevel|hum\.update|hum\.silence|hum\.detune|ss\.setZone:loop)";
fn boring(name: &str) -> bool {
    BORING.iter().any(|p| name.starts_with(p))
}

/// What the comparison still covers once a call into another module's big
/// function (death, a new level) has run: from then on only the glue's own state.
#[derive(Clone, Copy, PartialEq, Debug)]
enum Phase {
    Full,
    AfterDeath,
    AfterLevel,
}
const AFTER_DEATH: [&str; 24] = [
    "inStairwell",
    "stairNearExit",
    "inLoop",
    "loopNearExit",
    "sw.groupVis",
    "sw.faceVis",
    "worldVis",
    "lp.present",
    "lp.inScene",
    "fl.dist",
    "pl.dist",
    "pl.i",
    "amb.r",
    "amb.g",
    "amb.b",
    "hemi.r",
    "hemi.g",
    "hemi.b",
    "poolT",
    "sanity",
    "levelNum",
    "pool0.i",
    "pool3.i",
    "pool17.i",
];
const AFTER_LEVEL: [&str; 9] =
    ["inStairwell", "inLoop", "sw.groupVis", "sw.faceVis", "worldVis", "lp.present", "lp.inScene", "levelNum", "loopNearExit"];

fn fixture() -> &'static Json {
    thread_local! { static F: &'static Json = Box::leak(Box::new(Json::load("modes.json"))); }
    F.with(|f| *f)
}

/// Does `pick_text` (hud.rs) draw from `rnd()` yet? (The recorded one draws once.)
fn pick_text_draws() -> u64 {
    let g = Game::new_bare();
    with_seed(1, || {
        let b = seeded_draws();
        let _ = g.pick_text("almondPickup");
        seeded_draws() - b
    })
}

struct Ctx {
    module_draws: Rc<Cell<u64>>,
    base_draws: u64,
    rec_say: f64,
    rust_say: f64,
}

/// The game the fixtures were recorded on: lights, fog, the HUD elements,
/// three entities, the player record, and the Stairwell made at startup (no
/// Blender face, the modules on their own rng stream).
fn setup(sc: &Json) -> (Game, Rc<Cell<u64>>) {
    let mut g = Game::new_bare();
    let dust = g.tex.dust_sprite.clone();
    g.init_render(&dust);
    if sc.get("sound").b() {
        let ac = AudioContext::new_silent();
        ac.set_muted(true);
        g.audio_init_with(ac);
        assert!(g.soundscape().is_some());
    }
    let n = Rc::new(Cell::new(0u64));
    let m = Rc::new(RefCell::new(Mulberry32::new(sc.get("moduleSeed").i() as u32)));
    let f: Rc<dyn Fn() -> f64> = {
        let (n, m) = (n.clone(), m.clone());
        Rc::new(move || {
            n.set(n.get() + 1);
            m.borrow_mut().next()
        })
    };
    set_modes_rng(Some(f.clone()));
    let s = Stairwell::new(StairDeps { canvas_tex: game_canvas_tex(), rng: Box::new(move || f()), face_geometry: None });
    g.scene.add(&s.group);
    g.stairwell = Some(s);
    // lights and fog as the fixtures have them
    g.amb_light.light().color.set_hex(0xa89a70);
    g.amb_light.set_intensity(0.5);
    g.hemi_light.light().color.set_hex(0xd0c48c);
    g.hemi_light.light().ground_color.set_hex(0x2a2418);
    g.hemi_light.set_intensity(0.6);
    g.fog_color = Color::hex(0x070604);
    g.background = Color::hex(0x070604);
    g.fog_density = 0.052;
    g.player_light.light().color.set_hex(0xfff2cf);
    g.player_light.set_intensity(2.4);
    g.player_light.light().distance = 7.0;
    g.flashlight.set_intensity(0);
    g.flashlight.light().distance = 28.0;
    for (i, l) in g.pooled.iter().enumerate() {
        l.light().color.set_hex(0xffeec4);
        l.light().distance = 10.0;
        l.set_intensity(if i < 4 { 1.5 } else { 0.0 });
        l.position().set(i as f64, 2.9, -(i as f64));
        l.target().unwrap().position().set(i as f64, 0, -(i as f64));
    }
    for i in 0..g.pooled.len() {
        g.pooled_panel[i] = if i < 4 { Some(i) } else { None };
        g.pooled_w[i] = if i < 4 { 1.0 } else { 0.0 };
    }
    g.pool_t = 0.0;
    g.halluc_shape.set_visible(true);
    g.world.set_visible(true);
    g.entities = (0..3)
        .map(|i| {
            let mut e = Entity::new("smiler", group());
            if i != 1 {
                e.face = Some(mesh(box_geo!(0.1, 0.1, 0.1), basic_mat!()));
            }
            e
        })
        .collect();
    g.ll_on = 1.0;
    // the HUD elements
    for el in [&mut g.ui.fade, &mut g.ui.flash] {
        el.set_opacity(0.0);
        el.set_style("background", "#000");
    }
    g.ui.prompt.set_text("");
    g.ui.stair_hud.set_text("");
    g.ui.body.classes.clear();
    // the player
    g.player.pos = Vec3::new(3.0, 1.62, 4.0);
    g.player.yaw = 0.4;
    g.player.pitch = 0.0;
    g.player.stamina = 100.0;
    g.player.sanity = 100.0;
    g.player.battery = 100.0;
    g.player.flashlight_on = false;
    g.player.crouch = false;
    g.player.bob_phase = 0.0;
    g.player.bob_amp = 0.0;
    g.player.bottles = 0;
    g.player.batteries = 0;
    for (k, v) in sc.get("player0").entries() {
        match k.as_str() {
            "sanity" => g.player.sanity = v.f(),
            "battery" => g.player.battery = v.f(),
            "stamina" => g.player.stamina = v.f(),
            "flashlightOn" => g.player.flashlight_on = v.b(),
            o => panic!("player0.{}", o),
        }
    }
    g.state = "playing".into();
    g.perf_t = 0.0;
    g.shake_t = 0.0;
    g.now_ms = 0.0;
    g.level_num = sc.get("levelNum").i() as i32;
    g.run_seed = sc.get("runSeed").i() as u32;
    (g, n)
}

fn num(g: &Game, c: &Ctx, k: &str) -> f64 {
    let b = |v: bool| if v { 1.0 } else { 0.0 };
    let cam = g.camera.pos();
    let cr = g.camera.rot();
    let sw = g.stairwell.as_ref().unwrap();
    let lr = g.loop_rooms.as_ref();
    if let Some((i, f)) = k.strip_prefix("pool").and_then(|r| r.split_once('.')) {
        let l = &g.pooled[i.parse::<usize>().unwrap()];
        let (p, t, col) = (l.pos(), l.target().unwrap().pos(), l.light_color());
        return match f {
            "i" => l.intensity(),
            "dist" => l.light().distance,
            "x" => p.x,
            "y" => p.y,
            "z" => p.z,
            "tx" => t.x,
            "ty" => t.y,
            "tz" => t.z,
            "r" => col.r,
            "g" => col.g,
            "b" => col.b,
            _ => panic!("{}", k),
        };
    }
    match k {
        "cam.x" => cam.x,
        "cam.y" => cam.y,
        "cam.z" => cam.z,
        "cam.rx" => cr.x,
        "cam.ry" => cr.y,
        "cam.rz" => cr.z,
        "pos.x" => g.player.pos.x,
        "pos.y" => g.player.pos.y,
        "pos.z" => g.player.pos.z,
        "yaw" => g.player.yaw,
        "pitch" => g.player.pitch,
        "sanity" => g.player.sanity,
        "stamina" => g.player.stamina,
        "battery" => g.player.battery,
        "flashlightOn" => b(g.player.flashlight_on),
        "crouch" => b(g.player.crouch),
        "bobPhase" => g.player.bob_phase,
        "bobAmp" => g.player.bob_amp,
        "bottles" => g.player.bottles as f64,
        "batteries" => g.player.batteries as f64,
        "shakeT" => g.shake_t,
        "perfT" => g.perf_t,
        "levelNum" => g.level_num as f64,
        "fl.i" => g.flashlight.intensity(),
        "fl.dist" => g.flashlight.light().distance,
        "fl.x" => g.flashlight.pos().x,
        "fl.y" => g.flashlight.pos().y,
        "fl.z" => g.flashlight.pos().z,
        "tg.x" => g.flashlight_target.pos().x,
        "tg.y" => g.flashlight_target.pos().y,
        "tg.z" => g.flashlight_target.pos().z,
        "pl.dist" => g.player_light.light().distance,
        "pl.i" => g.player_light.intensity(),
        "pl.x" => g.player_light.pos().x,
        "pl.y" => g.player_light.pos().y,
        "pl.z" => g.player_light.pos().z,
        "amb.i" => g.amb_light.intensity(),
        "amb.r" => g.amb_light.light_color().r,
        "amb.g" => g.amb_light.light_color().g,
        "amb.b" => g.amb_light.light_color().b,
        "hemi.i" => g.hemi_light.intensity(),
        "hemi.r" => g.hemi_light.light_color().r,
        "hemi.g" => g.hemi_light.light_color().g,
        "hemi.b" => g.hemi_light.light_color().b,
        "gnd.r" => g.hemi_light.light().ground_color.r,
        "gnd.g" => g.hemi_light.light().ground_color.g,
        "gnd.b" => g.hemi_light.light().ground_color.b,
        "fog.d" => g.fog_density,
        "fog.r" => g.fog_color.r,
        "fog.g" => g.fog_color.g,
        "fog.b" => g.fog_color.b,
        "bg.r" => g.background.r,
        "bg.g" => g.background.g,
        "bg.b" => g.background.b,
        "llOn" => g.ll_on,
        "inStairwell" => b(g.in_stairwell),
        "stairNearExit" => b(g.stair_near_exit),
        "inLoop" => b(g.in_loop),
        "loopNearExit" => b(g.loop_near_exit),
        "sa.nextPhrase" => g.stair_audio.next_phrase,
        "sa.nextKnock" => g.stair_audio.next_knock,
        "sa.heartT" => g.stair_audio.heart_t,
        "sa.stepAcc" => g.stair_audio.step_acc,
        "la.stepAcc" => g.loop_audio.step_acc,
        "la.touchT" => g.loop_audio.touch_t,
        "worldVis" => b(g.world.visible()),
        "hallucVis" => b(g.halluc_shape.visible()),
        "e0Vis" => b(g.entities[0].face.as_ref().unwrap().visible()),
        "e2Vis" => b(g.entities[2].face.as_ref().unwrap().visible()),
        "sw.groupVis" => b(sw.group.visible()),
        "sw.faceVis" => b(sw.face.visible()),
        "sw.seg" => sw.st.seg as f64,
        "sw.x" => sw.st.x,
        "sw.z" => sw.st.z,
        "sw.y" => sw.st.y,
        "sw.landing" => sw.landing() as f64,
        "sw.depth" => sw.depth_m(),
        "sw.faceCount" => sw.st.face_count as f64,
        "lp.present" => b(lr.is_some()),
        "lp.lap" => lr.map(|l| l.lap() as f64).unwrap_or(-99.0),
        "lp.room" => lr.map(|l| l.room() as f64).unwrap_or(-99.0),
        "lp.x" => lr.map(|l| l.state().x).unwrap_or(0.0),
        "lp.z" => lr.map(|l| l.state().z).unwrap_or(0.0),
        "lp.seams" => lr.map(|l| l.state().seams as f64).unwrap_or(-1.0),
        "lp.inScene" => b(lr.map(|l| l.group.parent().map(|p| p.id() == g.scene.id()).unwrap_or(false)).unwrap_or(false)),
        "poolT" => g.pool_t,
        "glueDraws" => (seeded_draws() - c.base_draws) as f64,
        "moduleDraws" => c.module_draws.get() as f64,
        _ => panic!("unknown field {}", k),
    }
}
fn strs(g: &Game, k: &str) -> String {
    match k {
        "state" => g.state.clone(),
        "stairHud" => g.ui.stair_hud.text.clone(),
        "prompt" => g.ui.prompt.text.clone(),
        "fadeOp" => g.ui.fade.style("opacity").unwrap_or("").to_string(),
        "fadeBg" => g.ui.fade.style("background").unwrap_or("").to_string(),
        "flashOp" => g.ui.flash.style("opacity").unwrap_or("").to_string(),
        "body" => g.ui.body.classes.iter().cloned().collect::<Vec<_>>().join(" "),
        "faceKind" => g.stairwell.as_ref().unwrap().st.face.as_ref().map(|f| f.kind.as_str().to_string()).unwrap_or_default(),
        _ => panic!("unknown string field {}", k),
    }
}

fn do_action(g: &mut Game, a: &Json) {
    let a = a.arr();
    match a[0].s() {
        "set" => {
            let v = &a[2];
            match a[1].s() {
                "flashlightOn" => g.player.flashlight_on = v.b(),
                "battery" => g.player.battery = v.f(),
                "sanity" => g.player.sanity = v.f(),
                "stamina" => g.player.stamina = v.f(),
                "nextKnock" => g.stair_audio.next_knock = v.f(),
                "nextPhrase" => g.stair_audio.next_phrase = v.f(),
                "heartT" => g.stair_audio.heart_t = v.f(),
                "shakeT" => g.shake_t = v.f(),
                "state" => g.state = v.s().to_string(),
                o => panic!("set {}", o),
            }
        }
        "stairPlace" => {
            let (seg, x, z) = (a[1].i() as i32, a[2].f(), a[3].f());
            let s = g.stairwell.as_mut().unwrap();
            let y = s.seg_y(seg, x, z);
            let st = s.state_mut();
            st.seg = seg;
            st.x = x;
            st.z = z;
            st.y = y;
            if seg > st.deepest_seg {
                st.deepest_seg = seg;
            }
        }
        "loopPlace" => {
            if !g.in_loop || g.loop_rooms.is_none() {
                return;
            }
            let (lap, z, x, yaw, pitch) = (a[1].i() as i32, a[2].f(), a[3].f(), a[4].f(), a[5].f());
            g.player.yaw = yaw;
            g.player.pitch = pitch;
            g.loop_rooms.as_mut().unwrap().place(lap, z, x, Some(yaw));
            g.update_loop(0.0);
        }
        "enterStairwell" => g.enter_stairwell(),
        "exitStairwell" => g.exit_stairwell(),
        "leaveStairwellMode" => g.leave_stairwell_mode(),
        "enterLoop" => g.enter_loop(),
        "exitLoop" => g.exit_loop(),
        "leaveLoopMode" => g.leave_loop_mode(),
        o => panic!("action {}", o),
    }
}

/// Compare one frame's calls. Returns the boundary they cross, if any.
fn compare_calls(what: &str, rec: &[(String, Vec<f64>)], rust: &[TraceEntry], ctx: &mut Ctx, pick_stub: bool) {
    let names = |v: &[TraceEntry]| v.iter().map(|(n, _)| n.clone()).collect::<Vec<_>>();
    assert_eq!(
        rec.len(),
        rust.len(),
        "{}: calls\n  recorded: {:?}\n  got:      {:?}",
        what,
        rec.iter().map(|(n, _)| n.clone()).collect::<Vec<_>>(),
        names(rust)
    );
    for (i, ((jn, ja), (rn, ra))) in rec.iter().zip(rust.iter()).enumerate() {
        let after_pick = i > 0 && rec[i - 1].0.starts_with("pick_text:");
        if jn.starts_with("msg:") && after_pick && pick_stub {
            assert_eq!(rn, "msg:", "{}: message from pick_text (stub)", what);
        } else if let (Some(j), true) = (jn.strip_prefix("msg:LEVEL "), rn.starts_with("msg:LEVEL ")) {
            // `LEVEL n — <level_title()>`: the title is anomalies.rs's
            let jp = j.split(" — ").next().unwrap();
            let rp = rn["msg:LEVEL ".len()..].split(" — ").next().unwrap();
            assert_eq!(jp, rp, "{}: level message", what);
            assert!(rn.contains(" — "), "{}: level message {}", what, rn);
        } else {
            assert_eq!(jn, rn, "{}: call {}", what, i);
        }
        let say = jn.starts_with("voice.say:");
        assert_eq!(ja.len(), ra.len(), "{}: {} argument count", what, jn);
        for (k, (a, b)) in ja.iter().zip(ra.iter()).enumerate() {
            if say && k == 3 {
                continue;
            } // the phrase's length: the recorded voice was a stub
            assert_close(*b, *a, TOL, &format!("{}: {} arg {}", what, jn, k));
        }
        if say {
            ctx.rec_say = ja[3];
            ctx.rust_say = ra[3];
        }
    }
}

fn run_scenario(name: &str) {
    let fx = fixture();
    assert_eq!(fx.get("boring").s(), BORING_SRC, "the recorded list of per-frame sound calls changed");
    let sc = fx.get("scenarios").arr().iter().find(|s| s.get("name").s() == name).unwrap_or_else(|| panic!("no scenario {}", name));
    let pick_stub = pick_text_draws() == 0;
    let (mut g, module_draws) = setup(sc);
    let num_names: Vec<String> = sc.get("numNames").arr().iter().map(|v| v.s().to_string()).collect();
    let str_names: Vec<String> = sc.get("strNames").arr().iter().map(|v| v.s().to_string()).collect();
    let snaps = sc.get("snaps").arr();
    let script = sc.get("script").arr();
    assert_eq!(script.len() as i64, sc.get("frames").i());
    let mut next_snap = 0;
    let mut phase = Phase::Full;
    let mut compared = 0usize;
    with_seed(sc.get("glueSeed").i() as u32, || {
        let mut ctx = Ctx { module_draws: module_draws.clone(), base_draws: seeded_draws(), rec_say: 0.0, rust_say: 0.0 };
        trace_begin();
        for (f, rec) in script.iter().enumerate() {
            if rec.has("k") {
                let km = rec.get("k").i();
                let down: Vec<&str> = KEYS.iter().enumerate().filter(|(b, _)| km & (1 << b) != 0).map(|(_, k)| *k).collect();
                for k in KEYS {
                    if down.contains(&k) {
                        g.keys.insert(k.to_string());
                    } else {
                        g.keys.remove(k);
                    }
                }
            }
            if rec.has("y") {
                g.player.yaw = rec.get("y").f();
            }
            if rec.has("p") {
                g.player.pitch = rec.get("p").f();
            }
            let dt = if rec.has("dt") { rec.get("dt").f() } else { 1.0 / 60.0 };
            if rec.has("a") {
                for a in rec.get("a").arr() {
                    do_action(&mut g, a);
                }
            }
            g.now_ms += dt * 1000.0;
            g.run_scheduled();
            g.perf_t += dt;
            if g.state == "playing" && g.in_stairwell {
                g.update_stairwell(dt);
            } else if g.state == "playing" && g.in_loop {
                g.update_loop(dt);
            }
            let calls = trace_take();
            trace_begin();
            // the recorded `pick_text` drew once; a stubbed one does not
            if pick_stub {
                for (n, _) in &calls {
                    if n.starts_with("pick_text:") {
                        rnd();
                    }
                }
            }
            let what = format!("{} frame {}", name, f);
            let snap = if next_snap < snaps.len() && snaps[next_snap].get("f").i() as usize == f {
                next_snap += 1;
                Some(&snaps[next_snap - 1])
            } else {
                None
            };
            let Some(snap) = snap else {
                let loud: Vec<_> = calls.iter().filter(|(n, _)| !boring(n)).map(|(n, _)| n.clone()).collect();
                assert!(loud.is_empty(), "{}: calls the recording does not have: {:?}", what, loud);
                continue;
            };
            let rec_calls: Vec<(String, Vec<f64>)> = snap.get("t").arr().iter().map(|e| (e.at(0).s().to_string(), e.at(1).fs())).collect();
            compare_calls(&what, &rec_calls, &calls, &mut ctx, pick_stub);
            for (n, _) in &rec_calls {
                if n.starts_with("on_death:") && phase == Phase::Full {
                    phase = Phase::AfterDeath;
                }
                if n == "init_level" || n == "next_level" {
                    phase = Phase::AfterLevel;
                }
            }
            if !snap.has("n") {
                continue;
            }
            let jn = snap.get("n").arr();
            for (i, k) in num_names.iter().enumerate() {
                let keep = match phase {
                    Phase::Full => true,
                    Phase::AfterDeath => AFTER_DEATH.contains(&k.as_str()),
                    Phase::AfterLevel => AFTER_LEVEL.contains(&k.as_str()),
                };
                if !keep {
                    continue;
                }
                let mut want = jn[i].f();
                let mut got = num(&g, &ctx, k);
                if k == "sa.nextPhrase" && ctx.rec_say != 0.0 {
                    want -= ctx.rec_say;
                    got -= ctx.rust_say;
                }
                assert_close(got, want, TOL, &format!("{}: {}", what, k));
            }
            let rec_s = snap.get("s").arr();
            for (i, k) in str_names.iter().enumerate() {
                let keep = match phase {
                    Phase::Full => true,
                    Phase::AfterDeath => ["state", "body", "prompt", "stairHud"].contains(&k.as_str()),
                    Phase::AfterLevel => ["state", "body"].contains(&k.as_str()),
                };
                if !keep {
                    continue;
                }
                assert_eq!(strs(&g, k), rec_s[i].s(), "{}: {}", what, k);
            }
            compared += 1;
        }
        let _ = trace_take();
    });
    set_modes_rng(None);
    assert_eq!(next_snap, snaps.len(), "{}: snapshots left over", name);
    assert!(compared > 20, "{}: only {} snapshots compared", name, compared);
    eprintln!(
        "{}: {} frames, {} snapshots compared (pick_text {})",
        name,
        script.len(),
        compared,
        if pick_stub { "stubbed" } else { "real" }
    );
}

#[test]
fn stairwell_descent_matches_reference() {
    run_scenario("stairDescent");
}
#[test]
fn stairwell_death_matches_reference() {
    run_scenario("stairDeath");
}
#[test]
fn stairwell_sound_matches_reference() {
    run_scenario("stairSound");
}
#[test]
fn loop_walk_matches_reference() {
    run_scenario("loopWalk");
}
#[test]
fn loop_sound_matches_reference() {
    run_scenario("loopSound");
}
#[test]
fn loop_death_matches_reference() {
    run_scenario("loopDeath");
}

#[test]
fn pure_helpers_match_reference() {
    let p = fixture().get("pure");
    for r in p.get("stairFlightsFor").arr() {
        assert_eq!(stair_flights_for(r.at(0).i() as i32) as i64, r.at(1).i(), "stair_flights_for({})", r.at(0).i());
    }
    let mut g = Game::new_bare();
    for r in p.get("loopEntryLap").arr() {
        g.run_seed = r.at(0).i() as u32;
        g.level_num = r.at(1).i() as i32;
        assert_eq!(g.loop_entry_lap() as i64, r.at(2).i(), "loop_entry_lap({}, {})", r.at(0).i(), r.at(1).i());
    }
    for r in p.get("phrase").arr() {
        assert_eq!(stair_phrase(r.at(0).b(), r.at(1).f()), r.at(2).s(), "phrase({}, {})", r.at(0).b(), r.at(1).f());
    }
    assert!(p.get("phrase").len() > 400);
    assert_eq!(LOOP_ZONES.len(), 7);
}
