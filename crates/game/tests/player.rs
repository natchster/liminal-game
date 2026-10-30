//! The player module on a synthetic level:
//! movement, stamina, battery, items, pickups, markers, crying, the way shown,
//! the vacuum bot, the broken elevator, safe room / dark zones, pools, faulty
//! light switches and the keydown actions.
//!
//! `collide_move` belongs to world.rs and may still be a stub (it moves the
//! point without collision). The tests detect that and then re-apply the
//! frame's movement through local grid versions of `collide_move` /
//! `circle_hits_wall` (the no-solids path), so long runs stay inside the
//! level either way.

use engine::audio::AudioContext;
use game::base::records::*;
use game::player::{WAY_PACE, WAY_PRINTS, WAY_PRINT_LIFE, WAY_PRINT_STEP};
use game::prelude::*;
use game::visuals::lighting::RF_DARK;
use std::collections::HashMap;

const DT: f64 = 1.0 / 60.0;

// ------------------------------------------------------------ test rig
struct Rig {
    g: Game,
    stub_collide: bool,
}

/// `circle_hits_wall` on the plain grid (no solids, no barricades)
fn local_circle_hits_wall(g: &Game, px: f64, pz: f64, r: f64) -> bool {
    let (x0, x1, z0, z1) = (g.tx(px - r), g.tx(px + r), g.tz(pz - r), g.tz(pz + r));
    for z in z0..=z1 {
        for x in x0..=x1 {
            if !g.is_wall_tile(x, z) {
                continue;
            }
            let cx = clamp(px, g.wx(x) - CELL / 2.0, g.wx(x) + CELL / 2.0);
            let cz = clamp(pz, g.wz(z) - CELL / 2.0, g.wz(z) + CELL / 2.0);
            if (px - cx).powi(2) + (pz - cz).powi(2) < r * r {
                return true;
            }
        }
    }
    false
}
/// `collide_move(p, dx, dz, r)` — slide along walls, per axis
fn local_collide_move(g: &Game, p: &mut Vec3, dx: f64, dz: f64, r: f64) {
    if dx != 0.0 && !local_circle_hits_wall(g, p.x + dx, p.z, r) {
        p.x += dx;
    }
    if dz != 0.0 && !local_circle_hits_wall(g, p.x, p.z + dz, r) {
        p.z += dz;
    }
}
/// world.rs's `collide_move` still a stub? (a real one refuses to walk into the border)
fn collide_is_stub(g: &Game) -> bool {
    let mut p = Vec3::new(g.wx(1), 0.0, g.wz(1));
    g.collide_move(&mut p, -CELL, 0.0, PLAYER_R);
    p.x != g.wx(1)
}

fn rig(seed: u32, weather: usize) -> Rig {
    let (mut g, _dist) = Game::test_level(seed, 21);
    g.level_weather = &game::base::state::WEATHER_BIASES[weather];
    g.init_player();
    g.state = "playing".into();
    g.pointer_locked = true;
    let stub_collide = collide_is_stub(&g);
    Rig { g, stub_collide }
}
fn with_audio(r: &mut Rig) {
    let ac = AudioContext::new();
    ac.set_muted(true);
    r.g.audio_init_with(ac);
}
/// pooled scuff sprites / ripple meshes (lighting's init_render builds the real ones)
fn add_fx_pools(g: &mut Game) {
    for _ in 0..6 {
        let s = sprite(&sprite_mat!({ color: 0xa08060, transparent: true, opacity: 0 }));
        s.set_visible(false);
        g.scuffs.push(PooledFx { obj: s, t: 0.0 });
        let m = mesh(plane_geo!(0.6, 0.6), basic_mat!({ transparent: true, opacity: 0 }));
        m.set_visible(false);
        g.ripples.push(PooledFx { obj: m, t: 0.0 });
    }
}
fn press(g: &mut Game, code: &str) {
    g.key_down(code, false);
}
fn release(g: &mut Game, code: &str) {
    g.key_up(code);
}

/// One frame of the player-side loop: `update_player`, the way's prints / cry
/// timers, the clocks, the timeouts.
fn frame(r: &mut Rig, dt: f64) {
    let g = &mut r.g;
    let before = g.player.pos;
    g.perf_t += dt;
    g.now_ms += dt * 1000.0;
    g.run_scheduled();
    g.update_player(dt);
    if r.stub_collide {
        let after = g.player.pos;
        let (dx, dz) = (after.x - before.x, after.z - before.z);
        g.player.pos = before;
        let mut p = before;
        local_collide_move(g, &mut p, dx, dz, PLAYER_R);
        g.player.pos = p;
    }
    let t = g.perf_t;
    g.update_way(dt);
    g.update_vacuum_bot(dt);
    g.update_broken_elevator(dt, t);
    for s in g.scuffs.iter_mut().chain(g.ripples.iter_mut()) {
        if s.t > 0.0 {
            s.t -= dt;
        }
    }
    g.mouse_dx = 0.0;
    g.mouse_dy = 0.0;
}
fn frames(r: &mut Rig, n: usize) {
    for _ in 0..n {
        frame(r, DT);
    }
}

fn finite(v: Vec3) -> bool {
    v.x.is_finite() && v.y.is_finite() && v.z.is_finite()
}
fn assert_sane(r: &Rig) {
    let g = &r.g;
    let p = &g.player;
    assert!(finite(p.pos) && p.yaw.is_finite() && p.pitch.is_finite(), "player pose not finite: {:?}", p);
    assert!((0.0..=STAM_MAX).contains(&p.stamina), "stamina {}", p.stamina);
    assert!((0.0..=SAN_MAX).contains(&p.sanity), "sanity {}", p.sanity);
    assert!((0.0..=BAT_MAX).contains(&p.battery), "battery {}", p.battery);
    assert!(p.eye_h > 0.2 && p.eye_h < 2.0, "eye {}", p.eye_h);
    assert!(p.pitch.abs() <= 1.45 + 1e-9);
    let cp = g.camera.pos();
    assert!(finite(cp), "camera {:?}", cp);
    let fov = g.camera.camera().fov;
    // (the setting's range, less crouching's 6° and more running's 7°, and
    // the reality shift's nudge of up to 4°)
    use game::session::settings::{FOV_MAX, FOV_MIN};
    assert!((FOV_MIN - 10.0..=FOV_MAX + 11.0).contains(&fov), "fov {}", fov);
    // never inside a wall
    let (tx, tz) = (g.tx(p.pos.x), g.tz(p.pos.z));
    assert!(!g.is_wall_tile(tx, tz), "player in wall tile ({tx},{tz})");
    assert!(!local_circle_hits_wall(g, p.pos.x, p.pos.z, PLAYER_R - 1e-9), "player circle overlaps a wall at {:?}", p.pos);
}
/// face +z (fwd = (-sin yaw, -cos yaw)); tile column x=1 of the test level is open
fn face_south(g: &mut Game) {
    g.player.yaw = PI;
}

// ------------------------------------------------------------ movement / stamina
#[test]
fn sprint_drains_stamina_and_moves() {
    let mut r = rig(11, 5);
    face_south(&mut r.g);
    let start = r.g.player.pos;
    press(&mut r.g, "KeyW");
    press(&mut r.g, "ShiftLeft");
    frames(&mut r, 60);
    let g = &r.g;
    assert!((g.player.stamina - (STAM_MAX - STAM_DRAIN)).abs() < 0.6, "stamina after 1 s sprint: {}", g.player.stamina);
    assert!((g.player.stam_delay - 0.8).abs() < 1e-9);
    assert_eq!(g.player.audible_range, 13.0);
    let moved = g.player.pos.z - start.z;
    assert!(moved > 4.0 && moved <= SPRINT + 0.01, "moved {moved} m along +z in 1 s of sprint");
    assert!((g.player.pos.x - start.x).abs() < 1e-6, "no sideways drift");
    assert!(g.player.bob_amp > 0.03 && g.player.bob_phase > 0.0);
    // (running widens the view by 7° over the walking one)
    assert!(g.camera.camera().fov > game::session::settings::FOV_DEFAULT + 5.0, "sprint fov {}", g.camera.camera().fov);
    assert_sane(&r);
}

#[test]
fn stamina_regenerates_after_the_delay() {
    let mut r = rig(12, 5);
    face_south(&mut r.g);
    press(&mut r.g, "KeyW");
    press(&mut r.g, "ShiftLeft");
    frames(&mut r, 90);
    let s0 = r.g.player.stamina;
    release(&mut r.g, "ShiftLeft");
    release(&mut r.g, "KeyW");
    frames(&mut r, 45); // 0.75 s: still inside the 0.8 s pause
    assert_eq!(r.g.player.stamina, s0);
    frames(&mut r, 65); // past the pause, ~1 s of regen
    let gained = r.g.player.stamina - s0;
    assert!(gained > 10.0 && gained < 15.0, "regen {gained}");
    assert_eq!(r.g.player.audible_range, 0.0);
    assert!(r.g.player.bob_amp < 0.01);
}

#[test]
fn exhausted_player_cannot_sprint() {
    let mut r = rig(13, 5);
    face_south(&mut r.g);
    // `stamina > 1` gates the sprint (at exactly the threshold it flickers
    // between a regen frame and a sprint frame); hold the regen pause so the
    // stamina stays put
    r.g.player.stamina = 0.5;
    r.g.player.stam_delay = 0.8;
    let z0 = r.g.player.pos.z;
    press(&mut r.g, "KeyW");
    press(&mut r.g, "ShiftLeft");
    frames(&mut r, 30);
    let moved = r.g.player.pos.z - z0;
    assert!((moved - WALK * 0.5).abs() < 0.05, "walk speed when exhausted: {moved}");
    assert_eq!(r.g.player.audible_range, 6.5);
    assert_eq!(r.g.player.stamina, 0.5);
}

#[test]
fn crouch_lowers_the_eye_and_blocks_sprint() {
    let mut r = rig(14, 5);
    face_south(&mut r.g);
    let z0 = r.g.player.pos.z;
    for k in ["KeyW", "ShiftLeft", "ControlLeft"] {
        press(&mut r.g, k);
    }
    frames(&mut r, 60);
    let g = &r.g;
    assert!(g.player.crouch);
    assert_eq!(g.player.stamina, STAM_MAX);
    assert!((g.player.eye_h - CROUCH_EYE).abs() < 0.01, "crouched eye {}", g.player.eye_h);
    assert_eq!(g.player.audible_range, 3.5);
    let moved = g.player.pos.z - z0;
    assert!((moved - CROUCH_WALK).abs() < 0.05, "crouch walk {moved}");
    assert!(g.player.crouch_pitch_lerp > 0.03, "crouch tilt {}", g.player.crouch_pitch_lerp);
    release(&mut r.g, "ControlLeft");
    frames(&mut r, 60);
    assert!(!r.g.player.crouch);
    assert!((r.g.player.eye_h - EYE).abs() < 0.02);
    // hiding (the Showroom) pulls the eye lower still
    r.g.player.hide_eye = Some(0.4);
    frames(&mut r, 90);
    assert!((r.g.player.eye_h - 0.4).abs() < 0.01);
    assert_sane(&r);
}

#[test]
fn mouse_look_turns_and_clamps() {
    let mut r = rig(15, 5);
    let y0 = r.g.player.yaw;
    r.g.mouse_move(100.0, 0.0);
    assert!((r.g.player.yaw - (y0 - 0.22)).abs() < 1e-9);
    r.g.mouse_move(0.0, -5000.0);
    assert_eq!(r.g.player.pitch, 1.45);
    frame(&mut r, DT);
    assert_eq!(r.g.camera.rot().y, r.g.player.yaw);
    assert!((r.g.camera.rot().x - 1.45).abs() < 0.05);
    r.g.pointer_locked = false;
    r.g.mouse_move(100.0, 0.0);
    assert!((r.g.player.yaw - (y0 - 0.22)).abs() < 1e-9, "no look without pointer lock");
}

#[test]
fn pool_slows_and_splashes() {
    let mut r = rig(16, 1);
    face_south(&mut r.g);
    let p = r.g.player.pos;
    r.g.pool_bounds = Some(PoolBounds { min_x: p.x - 3.0, max_x: p.x + 3.0, min_z: p.z - 3.0, max_z: p.z + 30.0, ..Default::default() });
    press(&mut r.g, "KeyW");
    frames(&mut r, 60);
    assert!(r.g.player.was_in_pool);
    assert_eq!(r.g.player.audible_range, 6.5 + 3.0);
    let moved = r.g.player.pos.z - p.z;
    assert!((moved - WALK * 0.55).abs() < 0.05, "pool walk {moved}");
    r.g.pool_bounds = None;
    frame(&mut r, DT);
    assert!(!r.g.player.was_in_pool);
}

// ------------------------------------------------------------ flashlight / items
#[test]
fn flashlight_drains_the_battery() {
    let mut r = rig(21, 5);
    press(&mut r.g, "KeyF");
    assert!(r.g.player.flashlight_on);
    frames(&mut r, 120);
    assert!((r.g.player.battery - (BAT_MAX - BAT_DRAIN * 2.0)).abs() < 0.2, "battery {}", r.g.player.battery);
    assert!((r.g.flashlight.intensity() - 14.0).abs() < 0.01);
    // the rig follows the camera
    assert_eq!(r.g.flashlight.pos(), r.g.camera.pos());
    release(&mut r.g, "KeyF");
    press(&mut r.g, "KeyF");
    assert!(!r.g.player.flashlight_on);
    let b = r.g.player.battery;
    frames(&mut r, 60);
    assert_eq!(r.g.player.battery, b);
    assert!(r.g.flashlight.intensity() < 0.01);
    // run dry: auto-off with a message
    r.g.player.battery = 0.05;
    r.g.toggle_flashlight();
    frames(&mut r, 5);
    assert!(!r.g.player.flashlight_on);
    assert_eq!(r.g.player.battery, 0.0);
    assert_eq!(r.g.msg_text, "BATTERY DEAD");
    // a dead cell won't switch on
    r.g.msg_text.clear();
    r.g.toggle_flashlight();
    assert!(!r.g.player.flashlight_on);
    assert_eq!(r.g.msg_text, "BATTERY DEAD");
}

#[test]
fn drinking_and_swapping() {
    let mut r = rig(22, 5);
    let g = &mut r.g;
    assert!(!g.drink_almond_water());
    assert_eq!(g.msg_text, "NO ALMOND WATER");
    g.player.bottles = 2;
    assert!(!g.drink_almond_water());
    assert_eq!(g.msg_text, "YOU ARE NOT TIRED YET");
    g.player.stamina = 30.0;
    g.player.stam_delay = 0.8;
    press(g, "KeyG");
    assert_eq!(g.player.bottles, 1);
    assert_eq!(g.player.stamina, 75.0);
    assert_eq!(g.player.stam_delay, 0.0);
    g.player.stamina = 80.0;
    press(g, "Digit1");
    assert_eq!(g.player.stamina, STAM_MAX);
    assert_eq!(g.player.bottles, 0);

    assert!(!g.swap_battery());
    assert_eq!(g.msg_text, "NO SPARE BATTERIES");
    g.player.batteries = 1;
    assert!(!g.swap_battery());
    assert_eq!(g.msg_text, "THE CELL IS STILL GOOD");
    g.player.battery = 10.0;
    press(g, "KeyR");
    assert_eq!(g.player.batteries, 0);
    assert_eq!(g.player.battery, 65.0);
    // not while paused
    g.state = "paused".into();
    g.player.bottles = 3;
    g.player.stamina = 10.0;
    assert!(!g.drink_almond_water());
    assert_eq!(g.player.bottles, 3);
}

// ------------------------------------------------------------ pickups
#[test]
fn pickups_and_first_use_hints() {
    let mut r = rig(31, 5);
    let p = r.g.player.pos;
    let at = |x: f64, z: f64| {
        let o = group();
        o.set_pos(x, 1.0, z);
        o
    };
    r.g.almonds.push(Pickup { g: at(p.x + 0.3, p.z), taken: false, base: 1.0, phase: 0.0 });
    r.g.almonds.push(Pickup { g: at(p.x + 0.3, p.z + 0.2), taken: false, base: 1.0, phase: 0.0 });
    r.g.battery_items.push(Pickup { g: at(p.x, p.z + 0.3), taken: false, base: 1.0, phase: 0.0 });
    r.g.papers.push(PaperPile { g: at(p.x - 0.3, p.z), taken: false });
    r.g.papers.push(PaperPile { g: at(p.x + 30.0, p.z), taken: false });
    frame(&mut r, DT);
    let g = &r.g;
    assert_eq!(g.player.bottles, 2);
    assert_eq!(g.player.batteries, 1);
    assert!((3..=8).contains(&g.player.papers), "papers {}", g.player.papers);
    assert!(g.almonds.iter().all(|a| a.taken && !a.g.visible()));
    assert!(g.battery_items[0].taken && !g.battery_items[0].g.visible());
    assert!(g.papers[0].taken && !g.papers[1].taken);
    assert!(g.msg_text.starts_with("PAPER STACK — "), "{}", g.msg_text);
    assert!(g.hinted_bottle && g.hinted_battery);
    // the hints land 1.7 s later (one per kind, the bottle one first)
    assert_eq!(g.timeouts.len(), 2);
    frames(&mut r, 103);
    assert_eq!(r.g.msg_text, "R to change it.");
    assert!(r.g.timeouts.is_empty());
    // a second bottle never re-hints
    r.g.hint_first_use("bottle");
    assert!(r.g.timeouts.is_empty());
}

#[test]
fn lore_notes_pickup_and_whisper() {
    let mut r = rig(32, 5);
    with_audio(&mut r);
    let p = r.g.player.pos;
    let note = |x: f64, z: f64| {
        let o = group();
        o.set_pos(x, 0.0, z);
        o
    };
    r.g.lore_notes.push(LoreNote { g: note(p.x + 0.5, p.z), taken: false, id: 4 });
    frame(&mut r, DT);
    assert!(r.g.lore_found.contains(&4));
    assert!(r.g.lore_notes[0].taken && !r.g.lore_notes[0].g.visible());
    assert!(r.g.msg_text.starts_with("NOTE FOUND — "), "{}", r.g.msg_text);
    // a re-spawned copy of a read note whispers its title once per approach
    let copy = note(p.x + 1.2, p.z);
    r.g.lore_notes.push(LoreNote { g: copy.clone(), taken: false, id: 4 });
    r.g.msg_t = 0.0;
    frame(&mut r, DT);
    assert!(!r.g.lore_notes[1].taken, "a read note is not picked up again");
    assert_eq!(copy.r().user.get("wasNearRead"), 1.0);
    assert_eq!(copy.r().user.get("whisperCooldown"), 15.0);
    frames(&mut r, 10);
    assert!(copy.r().user.get("whisperCooldown") < 15.0);
    // walking away re-arms it
    r.g.player.pos.z += 6.0;
    frame(&mut r, DT);
    assert_eq!(copy.r().user.get("wasNearRead"), 0.0);
}

#[test]
fn floor_hole_ends_the_update() {
    let mut r = rig(33, 5);
    let p = r.g.player.pos;
    r.g.floor_holes.push(FloorHole { x: p.x, z: p.z, r: 0.8 });
    let o = group();
    o.set_pos(p.x, 0.0, p.z);
    r.g.papers.push(PaperPile { g: o, taken: false });
    frame(&mut r, DT);
    // `begin_fall_down()` returns from `update_player` before the paper pickup
    assert!(!r.g.papers[0].taken);
}

// ------------------------------------------------------------ paper markers
#[test]
fn paper_markers() {
    let mut r = rig(41, 5);
    face_south(&mut r.g);
    press(&mut r.g, "KeyQ");
    assert_eq!(r.g.msg_text, "NO PAPER LEFT");
    r.g.player.papers = 3;
    let kids = r.g.world.child_count();
    r.g.drop_paper_marker();
    assert_eq!(r.g.paper_markers.len(), 1);
    assert_eq!(r.g.player.papers, 2);
    assert_eq!(r.g.world.child_count(), kids + 1);
    let m = &r.g.paper_markers[0];
    let mp = m.mesh.pos();
    assert!((mp.z - (r.g.player.pos.z + 0.35)).abs() < 1e-9 && (mp.y - 0.008).abs() < 1e-12);
    assert_eq!((m.world_x, m.world_z), (mp.x, mp.z));
    assert!((m.mesh.rot().y - PI).abs() <= 0.4);
    // too close to the last one: refused, nothing spent
    r.g.drop_paper_marker();
    assert_eq!(r.g.paper_markers.len(), 1);
    assert_eq!(r.g.player.papers, 2);
    press(&mut r.g, "KeyW");
    frames(&mut r, 30);
    release(&mut r.g, "KeyW");
    release(&mut r.g, "KeyQ");
    press(&mut r.g, "KeyQ");
    assert_eq!(r.g.paper_markers.len(), 2);
    assert_eq!(r.g.player.papers, 1);
}

// ------------------------------------------------------------ crying
#[test]
fn cry_for_help_cooldown_and_outcomes() {
    let mut r = rig(51, 5);
    r.g.exit_data = Some(ExitData { pos: Vec3::new(r.g.wx(19), 0.0, r.g.wz(19)), ..Default::default() });
    let watcher = Entity::new("watcher", group());
    r.g.entities.push(watcher);
    // a smiler on the new brain, waiting in its dark
    let (sx, sz) = (r.g.tx(r.g.player.pos.x) + 3, r.g.tz(r.g.player.pos.z));
    let si = r.g.add_entity("smiler", sx, sz, 0);
    r.g.entities[si].freeze = 2.0;
    press(&mut r.g, "KeyH");
    assert_eq!(r.g.cry_cooldown, 22.0);
    assert_eq!(r.g.msg_text, "YOU CRY OUT...");
    assert_eq!(r.g.timeouts.len(), 1);
    r.g.cry_for_help();
    assert_eq!(r.g.msg_text, "WAIT 22s");
    assert_eq!(r.g.timeouts.len(), 1);
    // the cooldown ticks in `update_way` (61 frames: 60 × 1/60 leaves 21.000…01)
    frames(&mut r, 61);
    r.g.cry_for_help();
    assert_eq!(r.g.msg_text, "WAIT 21s");

    // many cries (seeded): both outcomes happen; the way is shown without a word
    let (mut arrows, mut alerts) = (0, 0);
    with_seed(777, || {
        for _ in 0..24 {
            r.g.cry_cooldown = 0.0;
            r.g.cry_dim_t = 0.0;
            let n0 = r.g.entities.len();
            let shown = r.g.ways_shown;
            r.g.cry_for_help();
            r.g.now_ms += 1000.0;
            r.g.run_scheduled();
            match r.g.msg_text.as_str() {
                "YOU CRY OUT..." if r.g.ways_shown == shown + 1 => {
                    arrows += 1;
                    assert_eq!(r.g.cry_dim_t, 8.0);
                }
                "SOMETHING HEARD YOU" => {
                    alerts += 1;
                    assert!(r.g.entities.len() <= n0 + 1);
                    assert_eq!(r.g.entities[0].state, "wander", "the watcher keeps its own way");
                    // every creature on the new brain is sure where you cried out
                    for i in 1..r.g.entities.len() {
                        let aw = r.g.creature(i).map(|c| c.brain.mem.awareness).unwrap_or(0.0);
                        assert!(aw >= 0.98, "{} {i} did not hear the cry ({aw:.2})", r.g.entities[i].kind);
                    }
                }
                m => panic!("unexpected cry outcome {m:?}"),
            }
        }
    });
    assert!(arrows > 0 && alerts > 0, "arrows {arrows}, alerts {alerts}");
    // spawned hunters are smilers on level 1 (on the new brain), 10–26 m
    // away, held a moment (0.5 s) before they come
    let pp = r.g.player.pos;
    for i in 2..r.g.entities.len() {
        let e = &r.g.entities[i];
        assert_eq!(e.kind, "smiler");
        assert!(r.g.creature(i).is_some(), "a spawned smiler has its brain");
        let d = hypot2(e.pos().x - pp.x, e.pos().z - pp.z);
        assert!((10.0..26.0 + CELL).contains(&d), "spawned {d:.1} m off");
        assert!(e.freeze == 0.5 || e.freeze == 0.0);
    }
    // deeper levels favour the entities: level 8+ clamps to 5% arrows
    // (nine floors into the rooms: the rooms begin on level 4)
    r.g.level_num = 9 + game::level::building::BUILDING_FLOORS;
    let (mut arrows9, mut hounds) = (0, 0);
    with_seed(4242, || {
        for _ in 0..40 {
            r.g.cry_cooldown = 0.0;
            let n0 = r.g.entities.len();
            let shown = r.g.ways_shown;
            r.g.cry_for_help();
            r.g.now_ms += 1000.0;
            r.g.run_scheduled();
            if r.g.ways_shown > shown {
                arrows9 += 1;
            }
            if r.g.entities.len() > n0 && r.g.entities[n0].kind == "hound" {
                hounds += 1;
                // a creature of the redesign, sure where the cry came from
                let c = r.g.creature(n0).expect("the hound a cry brings is on the new brain");
                assert!(c.brain.mem.awareness >= 0.99, "it heard the cry");
            }
        }
    });
    assert!(arrows9 <= 8, "arrows on level 9: {arrows9}/40");
    assert!(hounds > 0);
}

#[test]
fn cry_dim_envelope() {
    let mut r = rig(52, 5);
    r.g.cry_dim_t = 8.0;
    frames(&mut r, 60);
    assert!(r.g.cry_dim_lerp > 0.75, "rise {}", r.g.cry_dim_lerp);
    frames(&mut r, 7 * 60 + 10);
    assert!(r.g.cry_dim_t <= 0.0);
    let peak = r.g.cry_dim_lerp;
    frames(&mut r, 60);
    assert!(r.g.cry_dim_lerp < peak && r.g.cry_dim_lerp > 0.3, "slow fall {}", r.g.cry_dim_lerp);
}

#[test]
fn the_ways_prints_come_up_wet_and_dry_away() {
    let mut r = rig(53, 5);
    r.g.init_player();
    for i in 1..4 {
        let m = mesh(r.g.print_geo.clone(), r.g.print_mat.duplicate());
        m.set_pos(r.g.wx(1), 0.006, r.g.wz(i));
        r.g.world.add(&m);
        r.g.way_prints.push(WayPrint { mesh: m, life: WAY_PRINT_LIFE - i as f64, index: i as usize });
    }
    let kids = r.g.world.child_count();
    // a breath in, each at its wettest; never lit, never pulsing
    frames(&mut r, 30);
    let o: Vec<f64> = r.g.way_prints.iter().map(|a| a.mesh.mat().opacity_v()).collect();
    assert!(o.iter().all(|&o| (0.5..=0.85).contains(&o)), "wet {o:?}");
    assert!(r.g.way_prints.iter().all(|a| a.mesh.mat().m().emissive.get_hex() == 0), "a print that glows");
    // drying: fainter as they go, gone at the end of their life
    frames(&mut r, 10 * 60);
    let o2: Vec<f64> = r.g.way_prints.iter().map(|a| a.mesh.mat().opacity_v()).collect();
    assert!(o2.iter().zip(&o).all(|(b, a)| b < a), "drying {o2:?} after {o:?}");
    frames(&mut r, 10 * 60);
    assert!(r.g.way_prints.is_empty());
    assert_eq!(r.g.world.child_count(), kids - 3);
}

#[test]
fn the_way_is_shown_along_the_way_to_the_door() {
    let mut r = rig(54, 5);
    r.g.init_player();
    let (ex, ez) = (1, 17);
    r.g.exit_data = Some(ExitData { pos: Vec3::new(r.g.wx(ex), 0.0, r.g.wz(ez)), ..Default::default() });
    if !r.g.bfs(ex, ez).iter().any(|&d| d >= 0) {
        eprintln!("bfs is still a stub: no path, no way");
        return;
    }
    // no panels over the way: footprints, a pace apart, walking toward the door
    r.g.panels.clear();
    r.g.msg_text.clear();
    assert_eq!(r.g.show_the_way(), Some(WayAnswer::Prints));
    assert_eq!(r.g.cry_dim_t, 8.0);
    assert!(r.g.msg_text.is_empty(), "the place said something: {:?}", r.g.msg_text);
    assert!(r.g.way_prints.is_empty(), "the prints come one after another, not at once");
    let mut seen = 0;
    for _ in 0..(WAY_PRINTS as f64 * WAY_PRINT_STEP / DT) as usize + 10 {
        frame(&mut r, DT);
        assert!(r.g.way_prints.len() >= seen, "a print went before its time");
        seen = r.g.way_prints.len();
    }
    assert!(seen >= WAY_PRINTS - 2, "{seen} prints");
    let (px, pz) = (r.g.player.pos.x, r.g.player.pos.z);
    let mut last = -1.0;
    for (k, a) in r.g.way_prints.iter().enumerate() {
        let p = a.mesh.pos();
        // down the open column x = 1 toward the door, each further on, a pace apart
        assert!((p.x - r.g.wx(1)).abs() < 0.2, "print {k} off the way at x {:.2}", p.x);
        let along = (p.x - px).hypot(p.z - pz);
        assert!(along > last, "print {k} not further on");
        if k > 0 {
            assert!((along - last - WAY_PACE).abs() < 0.25, "print {k}: {:.2} m from the last", along - last);
        }
        last = along;
        assert!(r.g.exit_data.as_ref().unwrap().pos.z > p.z - 0.5);
    }
    // lit panels over the way: they answer next, in the order you would walk under them
    for z in [3, 5, 7, 9] {
        let mut p = Panel::default();
        p.wx = r.g.wx(1);
        p.wz = r.g.wz(z);
        r.g.panels.push(p);
    }
    assert_eq!(r.g.show_the_way(), Some(WayAnswer::Lights));
    assert!(r.g.way_prints.is_empty(), "the last answer's prints dry at once");
    let mut order = vec![];
    for _ in 0..(1.5 / DT) as usize {
        frame(&mut r, DT);
        for (i, p) in r.g.panels.iter().enumerate() {
            if p.flicker > 0.0 && !order.contains(&i) {
                order.push(i);
            }
        }
    }
    assert_eq!(order, vec![0, 1, 2, 3], "the lights did not go toward the door in turn");
    // and not the lights twice running, while prints can answer
    assert_eq!(r.g.way_answer_here(), Some(WayAnswer::Prints));
    // no exit, no way
    r.g.exit_data = None;
    assert_eq!(r.g.show_the_way(), None);
}

// ------------------------------------------------------------ vacuum bot / elevator
fn vacuum(g: &Game, x: i32, z: i32) -> VacuumBot {
    let o = group();
    o.set_pos(g.wx(x), 0.0, g.wz(z));
    let btn = mesh(box_geo!(0.02, 0.01, 0.02), lambert!({ color: 0x60ff90 }));
    o.add(&btn);
    let mut objs = HashMap::new();
    objs.insert("homeBtn".to_string(), btn);
    let mut f = HashMap::new();
    f.insert("speed".to_string(), 0.45);
    VacuumBot { g: o, world_pos: Vec3::ZERO, dir: 0.0, timer: 0.0, follow_t: 0.0, objs, f, b: HashMap::new() }
}

#[test]
fn vacuum_bot_modes() {
    let mut r = rig(61, 5);
    with_audio(&mut r);
    let v = vacuum(&r.g, 1, 5);
    r.g.vacuum_bot = Some(v);
    // wander: picks a heading at once, then rolls slowly
    frames(&mut r, 240);
    {
        let v = r.g.vacuum_bot.as_ref().unwrap();
        assert!(finite(v.g.pos()));
        assert!(v.timer > -DT && v.timer <= 5.5);
        assert!(v.f["spinPhase"] > 0.0);
        let q = (v.dir / (PI / 4.0)).round() * (PI / 4.0);
        assert!((v.dir - q).abs() < 1e-9, "wander heading is a multiple of 45°: {}", v.dir);
    }
    // follow: closes in on a player farther than 1.8 m
    let d = |g: &Game| {
        let p = g.vacuum_bot.as_ref().unwrap().g.pos();
        hypot2(p.x - g.player.pos.x, p.z - g.player.pos.z)
    };
    {
        let (px, pz) = (r.g.wx(1), r.g.wz(5));
        let v = r.g.vacuum_bot.as_mut().unwrap();
        v.g.set_pos(px, 0.0, pz);
        v.follow_t = 3.0;
    }
    let d0 = d(&r.g);
    frames(&mut r, 120);
    let d1 = d(&r.g);
    assert!((d0 - d1 - 0.9 * 2.0).abs() < 0.05, "follow approach {d0} -> {d1}");
    // stun: sits still
    {
        let v = r.g.vacuum_bot.as_mut().unwrap();
        v.follow_t = 0.0;
        v.f.insert("stunT".into(), 1.0);
    }
    let p0 = r.g.vacuum_bot.as_ref().unwrap().g.pos();
    frames(&mut r, 50);
    let p1 = r.g.vacuum_bot.as_ref().unwrap().g.pos();
    assert_eq!((p0.x, p0.z), (p1.x, p1.z));
    // chase: runs at the player at 1.7 m/s, LED blinks yellow, then wanders again
    frames(&mut r, 20);
    {
        let (px, pz) = (r.g.wx(1), r.g.wz(6));
        let v = r.g.vacuum_bot.as_mut().unwrap();
        v.g.set_pos(px, 0.0, pz);
        v.f.insert("chaseT".into(), 1.0);
    }
    let d0 = d(&r.g);
    frames(&mut r, 30);
    let d1 = d(&r.g);
    assert!((d0 - d1 - 1.7 * 0.5).abs() < 0.05, "chase approach {d0} -> {d1}");
    let c = r.g.vacuum_bot.as_ref().unwrap().objs["homeBtn"].mat().color_v();
    assert!(c.r > 0.7 && (c.b - 0.10).abs() < 1e-9, "chase LED {:?}", c);
    frames(&mut r, 40);
    let v = r.g.vacuum_bot.as_ref().unwrap();
    assert!(v.f["chaseT"] <= 0.0);
    assert_eq!(v.objs["homeBtn"].mat().color_v(), Color::hex(0x60ff90));
    // the hum follows the distance (0.11 at the player, silent past 10 m)
    {
        let (px, pz) = (r.g.player.pos.x, r.g.player.pos.z + 2.0);
        r.g.vacuum_bot.as_mut().unwrap().g.set_pos(px, 0.0, pz);
    }
    frames(&mut r, 90);
    let hum = r.g.audio.vacuum_gain.as_ref().unwrap().gain().value();
    assert!(hum > 0.06 && hum <= 0.11, "hum {hum}");
    r.g.vacuum_bot = None;
    frames(&mut r, 120);
    assert!(r.g.audio.vacuum_gain.as_ref().unwrap().gain().value() < hum * 0.01);
}

#[test]
fn broken_elevator_pulse_and_linger_scare() {
    let mut r = rig(62, 5);
    let p = r.g.player.pos;
    r.g.broken_elevator = Some(BrokenElevator {
        light: point_light!(0xff2010, 0, 6, 2),
        warn_bar: None,
        world: Vec3::new(p.x + 1.0, 1.2, p.z),
        normal: Vec3::new(1.0, 0.0, 0.0),
        next_scrape: 0.5,
        peered: false,
        objs: HashMap::new(),
        f: HashMap::new(),
        b: HashMap::new(),
    });
    let san0 = r.g.player.sanity;
    frames(&mut r, 60);
    {
        let be = r.g.broken_elevator.as_ref().unwrap();
        assert!((0.0..=1.4).contains(&be.light.intensity()));
        assert!(be.next_scrape > 4.0 && be.next_scrape < 14.0, "rescheduled scrape {}", be.next_scrape);
        assert!((be.f["lingerT"] - 1.0).abs() < 0.02);
        assert!(!be.b.get("descentFired").copied().unwrap_or(false));
    }
    frames(&mut r, 14 * 60 + 2);
    assert!(r.g.broken_elevator.as_ref().unwrap().b["descentFired"]);
    assert_eq!(r.g.msg_text, "SOMETHING FELL. FAR DOWN.");
    assert!(r.g.player.sanity < san0, "sanity dip");
    assert_eq!(r.g.camera_dip_t, 0.0);
    frames(&mut r, 40); // ~0.67 s: the dip timeout ran
    assert!(r.g.camera_dip_t > 0.0 && r.g.camera_dip_t <= 0.35);
    frames(&mut r, 60);
    assert!(r.g.timeouts.is_empty());
    // walking away decays the linger, never fires twice
    r.g.player.pos.z += 12.0;
    frames(&mut r, 60);
    assert!(r.g.broken_elevator.as_ref().unwrap().b["descentFired"]);
}

// ------------------------------------------------------------ sanity: dark / safe room
#[test]
fn dark_regions_drain_and_safe_rooms_mend() {
    let mut r = rig(71, 5);
    let n = (r.g.w * r.g.h) as usize;
    r.g.region_map = vec![1; n];
    r.g.region_info = vec![RegionInfo::default(), RegionInfo { flags: RF_DARK, ..Default::default() }];
    frames(&mut r, 60);
    assert!((r.g.player.sanity - (SAN_MAX - 6.5)).abs() < 0.1, "dark drain {}", r.g.player.sanity);
    // the flashlight stops the drain; normal regen takes over
    r.g.player.flashlight_on = true;
    let s0 = r.g.player.sanity;
    frames(&mut r, 60);
    assert!((r.g.player.sanity - (s0 + SAN_REGEN)).abs() < 0.1);
    r.g.player.flashlight_on = false;
    // a lit region + safe room: discovered once, mends at 3.2x, glow on
    r.g.region_map = vec![0; n];
    let p = r.g.player.pos;
    r.g.safe_room = Some(SafeRoom { x: p.x, z: p.z, r: 2.0, ..Default::default() });
    r.g.player.sanity = 50.0;
    r.g.crit_sanity_accum = 3.0;
    frame(&mut r, DT);
    assert!(r.g.safe_room.as_ref().unwrap().discovered);
    assert_eq!(r.g.msg_text, "SAFE ROOM — SANITY MENDS HERE");
    assert!(r.g.ui.safe_room_glow.has_class("on"));
    assert_eq!(r.g.crit_sanity_accum, 0.0);
    frames(&mut r, 59);
    assert!((r.g.player.sanity - (50.0 + SAN_REGEN * 3.2)).abs() < 0.1, "safe regen {}", r.g.player.sanity);
    r.g.player.pos.z += 8.0;
    frame(&mut r, DT);
    assert!(!r.g.ui.safe_room_glow.has_class("on"));
}

// ------------------------------------------------------------ light switches
#[test]
fn faulty_switch_arms_then_flips() {
    let mut r = rig(81, 5);
    let p = r.g.player.pos;
    r.g.panels = vec![Panel::default(), Panel::default()];
    r.g.light_switches.push(LightSwitch { x: p.x + 1.0, z: p.z, faulty: true, on: true, panels: vec![1], ..Default::default() });
    r.g.light_switches.push(LightSwitch { x: p.x + 1.0, z: p.z, faulty: false, on: true, panels: vec![0], ..Default::default() });
    frame(&mut r, DT);
    {
        let sw = &r.g.light_switches[0];
        assert!((sw.f["armT"] - (0.75 - DT)).abs() < 1e-9);
        assert!(sw.b["wasInRange"]);
    }
    assert_eq!(r.g.panels[1].flicker, 0.75);
    assert_eq!(r.g.panels[0].flicker, 0.0, "the good switch never triggers");
    frames(&mut r, 45);
    assert_eq!(r.g.faulty_trip_count, 1.0);
    assert!(r.g.light_switches[0].f["motionCooldown"] > 7.0);
    // escalated (3+ trips): flips at once, with a shake
    r.g.faulty_trip_count = 3.0;
    r.g.light_switches[0].f.insert("motionCooldown".into(), 0.0);
    r.g.player.pos.z += 5.0;
    frame(&mut r, DT);
    r.g.player.pos.z -= 5.0;
    r.g.shake_t = 0.0;
    frame(&mut r, DT);
    assert_eq!(r.g.faulty_trip_count, 4.0);
    assert!(r.g.shake_t > 0.2, "escalation shake {}", r.g.shake_t);
    assert_eq!(r.g.light_switches[0].f["motionCooldown"], 8.0);
}

// ------------------------------------------------------------ keys
#[test]
fn keydown_actions() {
    let mut r = rig(91, 5);
    let g = &mut r.g;
    press(g, "KeyM");
    assert!(g.muted);
    assert_eq!(g.msg_text, "Sound off");
    release(g, "KeyM");
    press(g, "KeyM");
    assert!(!g.muted);
    assert_eq!(g.msg_text, "Sound on");
    // a held key fires once: its auto-repeat only keeps it held
    g.key_down("KeyM", true);
    assert!(!g.muted);
    release(g, "KeyM");
    // the other keys dispatch without panicking (their targets may be stubs)
    for k in ["KeyE", "F3", "Tab", "KeyJ", "ArrowLeft", "ArrowRight", "KeyA", "KeyD", "Escape", "Escape", "Enter", "Digit2"] {
        press(g, k);
        release(g, k);
    }
    g.journal_open = true;
    for k in ["ArrowLeft", "KeyA", "ArrowRight", "KeyD", "Escape"] {
        press(g, k);
        release(g, k);
    }
    g.journal_open = false;
    // (the death card is up)
    g.show_overlay("", "", "");
    g.state = "dead".into();
    press(g, "KeyR");
    release(g, "KeyR");
    // R while dead restarts the level (`restart()`)
    assert_eq!(g.state, "playing");
    g.show_overlay("", "", "");
    g.state = "dead".into();
    press(g, "Enter");
    assert_eq!(g.state, "playing");
    g.state = "dead".into();
    // F / Q / G / H do nothing while dead
    g.player.battery = 50.0;
    press(g, "KeyF");
    assert!(!g.player.flashlight_on);
    press(g, "KeyH");
    assert_eq!(g.cry_cooldown, 0.0);
}

// ------------------------------------------------------------ long scripted runs
/// A scripted walk: WASD, sprint and crouch toggles, mouse turns, low sanity,
/// shakes, lens dips, reality shifts; every frame checked.
fn scripted_run(r: &mut Rig, n: usize) {
    let mut held: Vec<&'static str> = vec![];
    for k in 0..n {
        let phase = k / 90;
        let want: Vec<&'static str> = match phase % 8 {
            0 => vec!["KeyW", "ShiftLeft"],
            1 => vec!["KeyW", "KeyD"],
            2 => vec!["KeyS"],
            3 => vec!["KeyW", "ControlLeft"],
            4 => vec!["KeyA", "ShiftLeft"],
            5 => vec!["ArrowUp", "ArrowRight", "ShiftRight"],
            6 => vec!["KeyC", "KeyW"],
            _ => vec![],
        };
        for h in held.clone() {
            if !want.contains(&h) {
                release(&mut r.g, h);
            }
        }
        for w in &want {
            if !held.contains(w) {
                press(&mut r.g, w);
            }
        }
        held = want;
        if k % 7 == 0 {
            r.g.mouse_move(((k as f64) * 0.37).sin() * 40.0, ((k as f64) * 0.11).cos() * 6.0);
        }
        if k % 400 == 13 {
            r.g.shake_t = 0.6;
        }
        if k % 500 == 77 {
            r.g.camera_dip_t = 0.35;
        }
        if k % 650 == 101 {
            r.g.reality_shift_t = 2.0;
        }
        if r.g.reality_shift_t > 0.0 {
            r.g.reality_shift_t -= DT;
        }
        if k % 300 == 150 {
            r.g.player.sanity = 12.0;
        }
        if k == n / 2 {
            r.g.player.stamina = 20.0;
        }
        frame(r, DT);
        assert_sane(r);
    }
}

#[test]
fn long_runs_in_every_weather() {
    for wi in 0..6 {
        for audio in [false, true] {
            let mut r = rig(100 + wi as u32, wi);
            if audio {
                with_audio(&mut r);
            }
            add_fx_pools(&mut r.g);
            r.g.player.flashlight_on = true;
            let start = r.g.player.pos;
            scripted_run(&mut r, 2400);
            let g = &r.g;
            let wk = g.weather_key();
            let moved = hypot2(g.player.pos.x - start.x, g.player.pos.z - start.z);
            assert!(moved > 1.0 || g.player.bob_phase > 20.0, "{wk}: the player went nowhere");
            assert!(g.player.bob_phase > 20.0, "{wk}: bob phase {}", g.player.bob_phase);
            assert!(g.player.battery < BAT_MAX - 30.0, "{wk}: battery {}", g.player.battery);
            // the deterministic per-weather footstep effects fired
            match wk {
                "wet" => assert!(g.footstep_fx_tally.wet > 0.0),
                "derelict" => assert!(g.footstep_fx_tally.derelict > 0.0),
                "still" => assert!(g.footstep_fx_tally.still > 0.0),
                "electric" => assert!(g.footstep_fx_tally.electric > 0.0),
                "haunted" => assert!(g.footstep_fx_tally.haunted > 0.0),
                _ => assert!(g.footstep_fx_tally.normal > 0.0),
            }
            if audio {
                // the crouch shuffle effects need audio
                match wk {
                    "wet" => assert!(g.crouch_fx_tally.wet > 0.0),
                    "still" => assert!(g.crouch_fx_tally.still > 0.0),
                    "derelict" => assert!(g.crouch_fx_tally.derelict > 0.0),
                    _ => {}
                }
            } else {
                assert_eq!(g.crouch_fx_tally.get(wk), 0.0, "{wk}: no crouch fx without audio");
            }
            if matches!(wk, "wet") {
                assert!(g.ripples.iter().any(|p| p.obj.visible()));
            }
            if matches!(wk, "derelict" | "electric") {
                assert!(g.scuffs.iter().any(|p| p.obj.visible()));
            }
        }
    }
}

#[test]
fn seeded_runs_are_reproducible() {
    let run = || {
        with_seed(2024, || {
            let mut r = rig(7, 3);
            add_fx_pools(&mut r.g);
            r.g.player.papers = 20;
            for k in 0..600 {
                if k == 0 {
                    press(&mut r.g, "KeyW");
                    press(&mut r.g, "ShiftLeft");
                }
                if k % 50 == 25 {
                    r.g.drop_paper_marker();
                }
                if k % 9 == 0 {
                    r.g.mouse_move(23.0, 1.0);
                }
                if k == 300 {
                    release(&mut r.g, "ShiftLeft");
                    press(&mut r.g, "ControlLeft");
                    r.g.player.sanity = 20.0;
                }
                frame(&mut r, DT);
            }
            let p = &r.g.player;
            let marks: Vec<(u64, u64)> = r.g.paper_markers.iter().map(|m| (m.world_x.to_bits(), m.world_z.to_bits())).collect();
            (
                p.pos.x.to_bits(),
                p.pos.z.to_bits(),
                p.stamina.to_bits(),
                p.bob_phase.to_bits(),
                r.g.camera.pos().y.to_bits(),
                marks,
                r.g.footstep_fx_tally.haunted.to_bits(),
            )
        })
    };
    assert_eq!(run(), run());
}
