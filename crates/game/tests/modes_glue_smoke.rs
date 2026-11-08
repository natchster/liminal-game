//! Smoke test of the Stairwell / Loop glue on a booted game: in and out of
//! both modes many times through the main loop, on wandering input, with and
//! without sound, by every way out (the door, death, a new level, a restart).
use engine::prelude::*;
use game::base::rng::Mulberry32;
use game::Game;

const KEYS: [&str; 8] = ["KeyW", "KeyS", "KeyA", "KeyD", "ShiftLeft", "KeyC", "ControlLeft", "ArrowUp"];

/// a booted game (`boot()`: `init_modes` builds the Stairwell there)
fn booted() -> Game {
    let mut g = Game::new_bare();
    g.boot();
    g.state = "playing".into();
    g.fixed_step_dt = 1.0 / 60.0;
    g
}
/// one main-loop frame of `dt` (the app advances the clock, then `frame()`)
fn frame(g: &mut Game, dt: f64) {
    g.now_ms += dt * 1000.0;
    g.frame(dt);
}
fn wander(g: &mut Game, r: &mut Mulberry32, f: usize) {
    if f % 30 == 0 {
        for k in KEYS {
            g.keys.remove(k);
        }
        for (i, k) in KEYS.iter().enumerate() {
            if r.next() < [0.7, 0.15, 0.2, 0.2, 0.25, 0.15, 0.05, 0.1][i] {
                g.keys.insert(k.to_string());
            }
        }
        g.player.yaw += (r.next() - 0.5) * 2.0;
        g.player.pitch = (r.next() - 0.5) * 1.2;
    }
    if f % 97 == 0 {
        g.player.flashlight_on = !g.player.flashlight_on;
    }
}
/// scene children other than the entities' bodies / faces (their number follows the level)
fn non_entity_kids(g: &Game) -> usize {
    let ent: Vec<u64> = g.entities.iter().flat_map(|e| [Some(e.g.id()), e.face.as_ref().map(|f| f.id())]).flatten().collect();
    g.scene.children().iter().filter(|o| !ent.contains(&o.id())).count()
}
fn assert_out(g: &Game, scene_kids: usize, what: &str) {
    assert!(!g.in_stairwell && !g.in_loop, "{}: still in a mode", what);
    assert!(g.loop_rooms.is_none() && g.loop_paper.is_empty(), "{}: the Loop was not disposed", what);
    assert!(g.stair_voice.is_none(), "{}: the child's voice still runs", what);
    let s = g.stairwell.as_ref().unwrap();
    assert!(!s.group.visible() && !s.face.visible(), "{}: the Stairwell still shows", what);
    assert!(g.world.visible(), "{}: the level is hidden", what);
    assert!(!g.ui.body.has_class("in-stairwell") && !g.ui.body.has_class("in-loop"), "{}: body classes", what);
    assert_eq!(g.ui.prompt.text, "", "{}: prompt", what);
    assert_eq!(non_entity_kids(g), scene_kids, "{}: objects left in the scene", what);
    assert_eq!(g.flashlight.light().distance, 28.0, "{}: flashlight reach", what);
}

fn cycle(g: &mut Game, sound: bool, seed: u32) {
    if sound {
        let ac = AudioContext::new_silent();
        ac.set_muted(true);
        g.audio_init_with(ac);
        assert!(g.soundscape().is_some());
    }
    let mut r = Mulberry32::new(seed);
    // the Stairwell was built at boot: hidden, in the scene
    let s = g.stairwell.as_ref().expect("init_modes builds the Stairwell");
    assert!(!s.group.visible());
    assert!(s.group.parent().map(|p| p == g.scene).unwrap_or(false));
    let kids = non_entity_kids(g);
    let mut exits = [0usize; 6];
    for round in 0..12 {
        let loop_mode = round % 2 == 1;
        g.state = "playing".into();
        g.player.sanity = 100.0;
        g.player.battery = 100.0;
        g.player.stamina = 100.0;
        if loop_mode {
            g.enter_loop();
        } else {
            g.enter_stairwell();
        }
        assert_eq!(g.state, "transition");
        for _ in 0..60 {
            frame(g, 1.0 / 60.0);
        }
        assert!(if loop_mode { g.in_loop && g.loop_rooms.is_some() } else { g.in_stairwell }, "round {}: not inside", round);
        assert_eq!(g.state, "playing");
        assert!(!g.world.visible());
        if loop_mode {
            assert!(g.ui.body.has_class("in-loop"));
            assert!(g.loop_rooms.as_ref().unwrap().group.parent().is_some());
        } else {
            assert!(g.ui.body.has_class("in-stairwell") && g.stairwell.as_ref().unwrap().group.visible());
            assert!(g.flashlight.light().distance < 4.0, "the lamp's reach is capped in the Stairwell");
            if sound {
                assert!(g.stair_voice.is_some() && g.stair_slot.is_some());
            }
        }
        // a while inside on wandering input, now and then jumping deep
        let n = 300 + (r.next() * 900.0) as usize;
        for f in 0..n {
            wander(g, &mut r, f);
            if f == n / 2 {
                if loop_mode {
                    let lap = [0, 3, 5, 6, 9][(r.next() * 5.0) as usize];
                    g.loop_rooms.as_mut().unwrap().place(lap, r.next() * game::modes::loop_rooms::LOOP_STRIP, 0.0, None);
                    g.update_loop(0.0);
                } else {
                    let s = g.stairwell.as_mut().unwrap();
                    let seg = 2 * ((r.next() * (s.flights() - 1) as f64) as i32) + 1;
                    let lp = s.landing_point(seg, 0.5);
                    let y = s.seg_y(seg, 0.0, lp.z);
                    let st = s.state_mut();
                    st.seg = seg;
                    st.x = 0.0;
                    st.z = lp.z;
                    st.y = y;
                    st.deepest_seg = st.deepest_seg.max(seg);
                }
            }
            frame(g, if f % 211 == 0 { 0.05 } else { 1.0 / 60.0 });
            assert!(g.camera.pos().x.is_finite() && g.camera.pos().y.is_finite() && g.camera.pos().z.is_finite());
            assert!(g.player.sanity >= 0.0 && g.player.battery >= 0.0 && g.player.stamina >= 0.0);
            if g.state != "playing" {
                break;
            }
        }
        // out, one way or another
        let way = (round / 2) % 6;
        for k in KEYS {
            g.keys.remove(k);
        }
        match (way, g.state.as_str()) {
            (_, "dead") => {
                exits[1] += 1;
                g.leave_stairwell_mode();
                g.leave_loop_mode();
            }
            (0, _) | (3, _) => {
                // walk to the way out and take it
                if loop_mode {
                    let lr = g.loop_rooms.as_mut().unwrap();
                    let (x, z) = lr.exit_door_at();
                    lr.place(0, z, x + 0.3, None);
                    g.update_loop(0.0);
                    assert!(g.loop_near_exit, "at the exit door");
                    assert_eq!(g.ui.prompt.text, "[E] OPEN THE EXIT DOOR");
                    g.exit_loop();
                } else {
                    let s = g.stairwell.as_mut().unwrap();
                    let seg = 2 * (s.flights() - 1) + 1;
                    let lp = s.landing_point(seg, 0.5);
                    let y = s.seg_y(seg, 0.0, lp.z);
                    let st = s.state_mut();
                    st.seg = seg;
                    st.x = 0.0;
                    st.z = lp.z;
                    st.y = y;
                    frame(g, 1.0 / 60.0);
                    assert!(g.stair_near_exit, "at the door at the bottom");
                    // (no prompt check after a whole frame: `update_hud` runs after
                    // `update_stairwell` and overwrites the "[E] OPEN THE DOOR" prompt)
                    g.exit_stairwell();
                }
                let lvl = g.level_num;
                for _ in 0..90 {
                    frame(g, 1.0 / 60.0);
                }
                assert_eq!(g.level_num, lvl + 1, "on to the next level");
                exits[0] += 1;
            }
            (1, _) => {
                g.player.sanity = 0.0;
                g.player.flashlight_on = false;
                frame(g, 1.0 / 60.0);
                assert_eq!(g.state, "dead");
                exits[1] += 1;
                g.leave_stairwell_mode();
                g.leave_loop_mode();
            }
            (2, _) => {
                g.init_level(g.level_num);
                exits[2] += 1;
            }
            (4, _) => {
                g.leave_stairwell_mode();
                g.leave_loop_mode();
                g.leave_stairwell_mode();
                g.leave_loop_mode();
                exits[4] += 1;
            }
            _ => {
                g.restart();
                exits[5] += 1;
            }
        }
        assert_out(g, kids, &format!("round {} (way {})", round, way));
    }
    assert!(exits[0] > 0 && exits[1] > 0, "exits {:?}", exits);
    // nothing happens from outside the modes
    g.state = "playing".into();
    g.exit_stairwell();
    g.exit_loop();
    g.leave_stairwell_mode();
    g.leave_loop_mode();
    for _ in 0..120 {
        frame(g, 1.0 / 60.0);
    }
    assert_out(g, kids, "after the rounds");
}

#[test]
fn modes_glue_smoke() {
    let mut g = booted();
    cycle(&mut g, false, 7); // without sound (`nosound`, or before the first click)
    cycle(&mut g, true, 11); // and with the soundscape
}
