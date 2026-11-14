//! The Hound (`docs/design/ENTITIES.md`, design 1): an old person on all fours,
//! starved, blind, that hunts by sound. *Move when it moves; freeze when it
//! listens.*
//!
//! - **Senses.** No sight. It hears your step along the way it would walk to
//!   you, twice as far as anything else, but less well while it moves (its
//!   own scrabble covers yours); stopped to listen, it hears everything. Within 1.5 m it smells you after 2 s
//!   however still you are.
//! - **Modes.** Roam (a lope, stopping to listen every 4–8 s: Idle) →
//!   Alert (the listening crouch, head cocked, 1.5 s: the tell; silent
//!   then, it goes to look; heard again, it is sure) → Search (to the sound
//!   at 3.4 m/s, sniffing) → Hunt (a bay, then a charge at 6 m/s for at most
//!   6 s, then a stop to listen) → Strike (a growl and a crouch, 0.35 s,
//!   then a spring)
//!   → Withdraw. Sent by the director it Stalks: toward your part of the
//!   level, listening as it goes.
//! - **Body.** `bodies.bin`'s `hound` (MakeHuman's old, thin man posed on
//!   all fours; `assets/src/creatures/body_hound.py`), its rest pose the
//!   pose it stands in. Hands and feet are planted in the world and the
//!   limbs solved onto them (`body::two_bone`); the left hind drags.
use crate::base::numeric::hypot2;
use crate::entities::body::{BodyRig, two_bone};
use crate::entities::brain::*;
use crate::entities::creature::*;
use crate::entities::{EntVoice, with_skin_detail};
use crate::prelude::*;

pub const DATA: KindData = KindData {
    sight: 0.0,
    sight_lit: 0.0,
    cone: -1.0,
    hearing: 2.0,
    smell: 0.9,
    radius: 0.3,
    reach: 1.2,
    sight_gain: 0.0,
    sound_gain: 1.6,
    forget: 0.08,
    touch_nerve: 0.0,
};

/// How well it hears while it moves (its own scrabble covers your steps),
/// and how fast is moving, m/s.
pub const MOVING_EARS: f64 = 0.6;
pub const MOVING: f64 = 0.8;
/// How long a still player within smelling distance takes to be found, s;
/// a search sniffs each spot for less, so only one that lingers by you finds you.
pub const SMELL_T: f64 = 3.0;
const SNIFF_T: f64 = 1.4;
/// The director's last stretch tells it where you are at most this often, s.
const HINT_EVERY: f64 = 8.0;
/// Speeds, m/s.
const LOPE: f64 = 2.0;
/// Sent by the director: unhurried far off, careful near (by distance to you).
fn stalk_speed(d: f64) -> f64 {
    if d > 20.0 {
        4.2
    } else if d > 10.0 {
        3.6
    } else {
        2.6
    }
}
const TROT: f64 = 3.4;
const CHARGE: f64 = 6.0;
/// It lost you this far off: you outran it (what it learns from), m.
const OUTRAN: f64 = 15.0;
/// Learned, it charges for where you will be this long from now, s.
const AHEAD_T: f64 = 1.2;
const SPRING: f64 = 7.0;
/// How long a charge lasts before it stops to listen, and the stop, s.
const CHARGE_T: f64 = 6.0;
const RELISTEN_T: f64 = 2.5;
/// The listening crouch, s; for its first `CROUCH_GRACE` it only listens
/// (what it hears then does not make it sure): your moment to freeze.
const CROUCH_T: f64 = 1.5;
pub const CROUCH_GRACE: f64 = 0.6;
/// The spring's wind-up (a growl, a crouch), s, and how near it springs from, m.
const WINDUP: f64 = 0.35;
const SPRING_FROM: f64 = 1.5;
/// How far its sounds carry (a tell counts if you are within this along the way), m.
const BAY_REACH: f64 = 40.0;
const SCRABBLE_REACH: f64 = 14.0;

thread_local! {
    static MATS: std::cell::RefCell<std::collections::HashMap<String, Mat>> = Default::default();
}

/// The hound's materials by slot: old skin (a waxy, dull sheen, the skin's
/// fine detail), its clothes (matte, both sides), its hair.
pub fn material(slot: &str) -> Mat {
    MATS.with(|m| {
        m.borrow_mut()
            .entry(slot.to_string())
            .or_insert_with(|| match slot {
                "cloth" => {
                    let m = lambert!({ color: 0xffffff, vertexColors: true });
                    m.m().side = DoubleSide;
                    with_skin_detail(m, 5.0, 0.6)
                }
                "hair" => {
                    let m = phong!({ color: 0xffffff, vertexColors: true, specular: 0x1c1b1a, shininess: 16 });
                    m.m().side = DoubleSide;
                    m
                }
                _ => with_skin_detail(phong!({ color: 0xffffff, vertexColors: true, specular: 0x1e1c1a, shininess: 20 }), 3.4, 1.2),
            })
            .clone()
    })
}

fn set(g: &mut Game, ei: usize, to: Mode, why: &'static str) {
    let now = g.perf_t;
    if let Some(c) = g.creature_mut(ei) {
        c.brain.set(to, why, now);
    }
}

/// A wander goal: a far tile it can reach, a little darker than most.
fn wander_goal(g: &Game, ei: usize) -> Option<(f64, f64)> {
    let pp = g.player.pos;
    let pos = g.entities[ei].pos();
    let mut best = None;
    let mut best_score = -1e9;
    for _ in 0..12 {
        let gx = with_native(|| irand(1, g.w as i64 - 2)) as i32;
        let gz = with_native(|| irand(1, g.h as i64 - 2)) as i32;
        if g.is_wall_tile(gx, gz) {
            continue;
        }
        let (wx, wz) = (g.wx(gx), g.wz(gz));
        let from_me = hypot2(wx - pos.x, wz - pos.z);
        if !(8.0..40.0).contains(&from_me) {
            continue;
        }
        let score = -g.e_region_light_at(gx, gz) * 2.0 - (hypot2(wx - pp.x, wz - pp.z) - 30.0).abs() * 0.03;
        if score > best_score {
            best_score = score;
            best = Some((wx, wz));
        }
    }
    best
}

/// One frame of the hound's mind.
pub fn think(g: &mut Game, ei: usize, dt: f64) {
    let Some(c) = g.creature(ei) else { return };
    let (mode, mode_t, order, p, aw) = (c.brain.mode, c.brain.mode_t, c.brain.order, c.brain.last, c.brain.mem.awareness);
    let pos = g.entities[ei].pos();
    let pp = g.player.pos;
    let now = g.perf_t;
    // its own steps cover yours; stopped, it hears everything
    let moving = hypot2(g.entities[ei].vx, g.entities[ei].vz) > MOVING;
    // ---- the senses feed its awareness
    let relaxed = g.director.phase == crate::entities::director::Phase::Relax || matches!(order, Order::Away(_));
    let unaware = aw < 0.2 && matches!(mode, Mode::Roam | Mode::Idle | Mode::Stalk);
    // how far it hears your step now: twice anyone's, less while it moves,
    // unaware or sent away
    let ears = DATA.hearing * if moving { MOVING_EARS } else { 1.0 } * if unaware { 0.8 } else { 1.0 } * if relaxed { 0.55 } else { 1.0 };
    let range = g.steps_heard() * ears;
    // (the cap: you are overlooked standing still; the yellow water: found sooner)
    let still = g.body.still;
    let mut sense = if range > 0.0 && p.path < range { ((range - p.path) / (range * 0.35).max(0.5)).clamp(0.0, 1.0) } else { 0.0 };
    if mode == Mode::Withdraw {
        sense = 0.0;
    }
    let smelling = p.dist < DATA.smell && p.line;
    if mode == Mode::Alert && mode_t < CROUCH_GRACE {
        sense = 0.0;
    }
    if sense > 0.02 {
        // blind, it places a sound roughly, never closer than a metre (the
        // fainter, the rougher), and never in a wall
        let (a, r) = with_native(|| (rnd() * PI * 2.0, 1.0 + rnd() * (1.0 - sense) * 3.0));
        let (hx, hz) = g.open_point(pp.x + a.cos() * r, pp.z + a.sin() * r);
        let c = g.creature_mut(ei).unwrap();
        c.brain.feed(DATA.sound_gain * sense, dt, 1.0);
        c.brain.mem.heard = Some((hx, hz, now));
    } else if mode != Mode::Alert {
        g.creature_mut(ei).unwrap().brain.drain(DATA.forget, dt);
    }
    {
        let c = g.creature_mut(ei).unwrap();
        c.smell_t = if smelling && mode != Mode::Withdraw { c.smell_t + dt } else { 0.0 };
        if c.smell_t >= SMELL_T * still {
            c.brain.mem.awareness = 1.0;
            c.brain.mem.heard = Some((pp.x, pp.z, now));
        }
    }
    let aw = g.creature(ei).unwrap().brain.mem.awareness;
    let heard = g.creature(ei).unwrap().brain.mem.heard;
    // where you are heading (your way over the last moments): learned, it
    // charges for there, not for where you are
    let ahead = {
        let c = g.creature_mut(ei).unwrap();
        let (lx, lz, had) = (c.get("ppx"), c.get("ppz"), c.get("pp_set") > 0.0);
        c.ease.insert("ppx", pp.x);
        c.ease.insert("ppz", pp.z);
        c.ease.insert("pp_set", 1.0);
        let (vx, vz) = if had && dt > 0.0 { ((pp.x - lx) / dt, (pp.z - lz) / dt) } else { (0.0, 0.0) };
        let (vx, vz) = (c.eased("pvx", vx.clamp(-8.0, 8.0), 4.0, dt), c.eased("pvz", vz.clamp(-8.0, 8.0), 4.0, dt));
        if c.get("ahead") > 0.0 && hypot2(vx, vz) > 1.0 { Some(g.open_point(pp.x + vx * AHEAD_T, pp.z + vz * AHEAD_T)) } else { None }
    };

    match mode {
        Mode::Withdraw => {
            let goal = g.creature(ei).unwrap().brain.goal.or_else(|| wander_goal(g, ei));
            g.creature_mut(ei).unwrap().brain.goal = goal;
            g.creature_go(ei, goal, false, TROT, dt);
            let away = matches!(order, Order::Away(t) if t > 0.0);
            if !away && mode_t > 8.0 {
                g.creature_mut(ei).unwrap().brain.goal = None;
                set(g, ei, Mode::Roam, "it has gone far enough");
            }
        }
        Mode::Roam | Mode::Stalk | Mode::Idle => {
            if aw >= SUSPECT {
                set(g, ei, Mode::Alert, "it heard something");
                g.creature_tell(ei, SCRABBLE_REACH);
                g.entity_call(ei, "listen");
                return;
            }
            if let Order::Away(_) = order {
                g.creature_mut(ei).unwrap().brain.goal = None;
                set(g, ei, Mode::Withdraw, "the director sent it away");
                return;
            }
            if let Order::Seek(x, z) = order {
                // told where you are, it is blind: it has the direction, not the
                // spot, and it takes a new hint only now and then
                let last = g.creature(ei).unwrap().get("hint_t");
                if last == 0.0 || now - last > HINT_EVERY {
                    let (a, r) = with_native(|| (rnd() * PI * 2.0, rand2(4.0, 7.0)));
                    let (hx, hz) = g.open_point(x + a.cos() * r, z + a.sin() * r);
                    let c = g.creature_mut(ei).unwrap();
                    c.ease.insert("hint_t", now);
                    c.brain.mem.heard = Some((hx, hz, now));
                    set(g, ei, Mode::Search, "it was told where you are");
                    return;
                }
            }
            let stalk = order == Order::Stalk;
            if mode == Mode::Idle {
                // the pause to listen
                g.entity_drive(ei, dt, 0.0);
                if mode_t > 1.6 {
                    set(g, ei, if stalk { Mode::Stalk } else { Mode::Roam }, "nothing heard");
                }
                return;
            }
            if stalk && mode == Mode::Roam {
                set(g, ei, Mode::Stalk, "the director sent it");
            } else if !stalk && mode == Mode::Stalk {
                set(g, ei, Mode::Roam, "no longer sent");
            }
            // where it goes: the director's goal when sent, else its own
            let goal = if stalk {
                let wt = g.entities[ei].w_target;
                wt.map(|(x, z)| (g.wx(x), g.wz(z)))
            } else {
                let c = g.creature(ei).unwrap();
                let reached = c.brain.goal.is_none_or(|(x, z)| hypot2(x - pos.x, z - pos.z) < 2.0);
                if reached || c.brain.goal_t <= 0.0 {
                    let gl = wander_goal(g, ei);
                    let c = g.creature_mut(ei).unwrap();
                    c.brain.goal = gl;
                    c.brain.goal_t = with_native(|| rand2(10.0, 18.0));
                }
                g.creature(ei).unwrap().brain.goal
            };
            g.creature_mut(ei).unwrap().brain.goal_t -= dt;
            g.creature_go(ei, goal, false, if stalk { stalk_speed(p.dist) } else { LOPE }, dt);
            // it stops to listen every few seconds; near you, every two to four
            let c = g.creature_mut(ei).unwrap();
            if c.get("pause_in") <= 0.0 {
                let near = stalk && p.dist < 15.0;
                let v = with_native(|| if near { rand2(2.0, 4.0) } else { rand2(4.0, 8.0) });
                c.ease.insert("pause_in", v);
            }
            if mode_t > c.get("pause_in") {
                c.ease.insert("pause_in", 0.0);
                set(g, ei, Mode::Idle, "it stops to listen");
            }
        }
        Mode::Alert => {
            // the listening crouch: still; silent through it and it goes to look
            g.entity_drive(ei, dt, 0.0);
            if sure(aw) {
                notice(g, ei);
            } else if mode_t > CROUCH_T {
                // it knows the sound's direction, not its spot: it goes near it
                if let Some((x, z, t)) = g.creature(ei).unwrap().brain.mem.heard {
                    let (a, r) = with_native(|| (rnd() * PI * 2.0, rand2(1.5, 3.5)));
                    let (hx, hz) = g.open_point(x + a.cos() * r, z + a.sin() * r);
                    g.creature_mut(ei).unwrap().brain.mem.heard = Some((hx, hz, t));
                }
                set(g, ei, Mode::Search, "silence: it goes to where it heard");
            }
        }
        Mode::Search => {
            if sure(aw) {
                notice(g, ei);
                return;
            }
            let goal = heard.map(|(x, z, _)| (x, z));
            let at = goal.is_none_or(|(x, z)| hypot2(x - pos.x, z - pos.z) < 1.5);
            if at {
                // sniffing about where it heard you
                g.entity_drive(ei, dt, 0.0);
                if g.creature(ei).unwrap().brain.t > SNIFF_T {
                    // a step or two further, or give up
                    g.creature_mut(ei).unwrap().brain.t = 0.0;
                    if let Some((x, z, t)) = g.creature(ei).unwrap().brain.mem.heard {
                        let a = with_native(rnd) * PI * 2.0;
                        let (hx, hz) = g.open_point(x + a.cos() * 2.5, z + a.sin() * 2.5);
                        g.creature_mut(ei).unwrap().brain.mem.heard = Some((hx, hz, t));
                    }
                }
            } else {
                g.creature_go(ei, goal, false, TROT, dt);
            }
            if mode_t > 18.0 && aw < SUSPECT {
                set(g, ei, Mode::Roam, "it gave up looking");
            }
        }
        Mode::Hunt => {
            let c = g.creature(ei).unwrap();
            let charge_t = c.chase_t;
            let listening = charge_t > CHARGE_T;
            if listening {
                // the stop in the charge: it listens; heard now, it comes again;
                // quiet through it, and it has lost you
                g.entity_drive(ei, dt, 0.0);
                let heard_now = (sense > 0.05 || smelling) && charge_t > CHARGE_T + 0.4;
                if heard_now || charge_t > CHARGE_T + RELISTEN_T {
                    if heard_now {
                        g.creature_mut(ei).unwrap().chase_t = 0.0;
                        bay(g, ei);
                    } else {
                        let c = g.creature_mut(ei).unwrap();
                        c.chase_t = 0.0;
                        c.brain.mem.awareness = 0.45;
                        set(g, ei, Mode::Search, "lost you in the stop");
                        // you got well away from it: you outran it (what it learns from)
                        if p.dist > OUTRAN {
                            g.creature_habit("hound");
                        }
                        return;
                    }
                }
            } else {
                // charge at where it hears you (it is blind: where you were, if you are quiet)
                let allowed = g.strike_allowed(ei);
                if p.dist < SPRING_FROM && allowed {
                    set(g, ei, Mode::Strike, "close enough to spring");
                    g.creature_mut(ei).unwrap().burst_m = 0.0;
                    g.entity_call(ei, "growl");
                    g.creature_tell(ei, SCRABBLE_REACH);
                    return;
                }
                if p.dist < 1.2 {
                    // not yet: it holds off, snarling (a tell)
                    g.entity_drive(ei, dt, 0.0);
                    if g.creature(ei).unwrap().brain.t > 1.2 {
                        g.creature_mut(ei).unwrap().brain.t = 0.0;
                        g.entity_call(ei, "growl");
                        g.creature_tell(ei, SCRABBLE_REACH);
                    }
                } else {
                    let goal = heard.map(|(x, z, _)| (x, z));
                    let fresh = heard.is_some_and(|(_, _, t)| now - t < 0.3);
                    if let Some(at) = ahead.filter(|_| fresh && p.dist > SPRING_FROM) {
                        // learned: for where you are heading, to meet you from ahead
                        g.creature_go(ei, Some(at), false, CHARGE, dt);
                    } else if fresh && p.dist < 14.0 && p.line {
                        g.creature_go(ei, None, true, CHARGE, dt);
                    } else {
                        g.creature_go(ei, goal, false, CHARGE, dt);
                    }
                }
            }
            g.creature_mut(ei).unwrap().chase_t += dt;
            if aw < 0.15 && !listening {
                set(g, ei, Mode::Search, "nothing to go on");
                if p.dist > OUTRAN {
                    g.creature_habit("hound");
                }
            }
        }
        Mode::Strike => {
            let t = g.creature(ei).unwrap().brain.mode_t;
            if t < WINDUP {
                g.entity_drive(ei, dt, 0.0);
            } else {
                if g.creature(ei).unwrap().burst_m == 0.0 {
                    // a pounce leaves at full speed, aimed at you
                    let (pos, pp) = (g.entities[ei].pos(), g.player.pos);
                    let (dx, dz) = (pp.x - pos.x, pp.z - pos.z);
                    let l = hypot2(dx, dz).max(1e-6);
                    let e = &mut g.entities[ei];
                    e.vx = dx / l * SPRING;
                    e.vz = dz / l * SPRING;
                }
                g.creature_burst(ei, SPRING, dt);
                let p = g.perceive(ei);
                if p.dist < DATA.reach {
                    // it reached you (the gate decides what that costs you)
                    g.creature_contact(ei);
                } else if g.creature(ei).unwrap().burst_m >= BURST_LEN || t > WINDUP + 1.0 {
                    // it lands and stops to listen (its stop in a charge)
                    let e = &mut g.entities[ei];
                    e.vx = 0.0;
                    e.vz = 0.0;
                    set(g, ei, Mode::Hunt, "the spring missed");
                    g.creature_mut(ei).unwrap().chase_t = CHARGE_T;
                }
            }
        }
        _ => set(g, ei, Mode::Roam, "back to its ground"),
    }
}

/// It is sure: it bays, and comes.
fn notice(g: &mut Game, ei: usize) {
    let now = g.perf_t;
    {
        let c = g.creature_mut(ei).unwrap();
        // each notice: no strike in the 2 s after it
        c.noticed_at = Some(now);
        c.chase_t = 0.0;
    }
    set(g, ei, Mode::Hunt, "it is sure where you are");
    g.director_note("notice");
    bay(g, ei);
    let d = g.perceive(ei).dist;
    if d < 30.0 {
        g.dread_sting((1.3 - d / 30.0).clamp(0.5, 1.0));
    }
}

// ------------------------------------------------------------ the body

/// Its four limbs: (upper, lower, end bone, the end's joint, side).
const LIMBS: [(&str, &str, &str, f64, f64); 4] = [
    // name stems, phase offset in the cycle, side
    ("thigh.L", "shin.L", "foot.L", 0.0, 1.0),
    ("upperarm.L", "forearm.L", "hand.L", 0.25, 1.0),
    ("thigh.R", "shin.R", "foot.R", 0.5, -1.0),
    ("upperarm.R", "forearm.R", "hand.R", 0.75, -1.0),
];

fn world_to_body(root: &Obj, p: Vec3) -> Vec3 {
    let o = root.pos();
    let yaw = root.rotation().y;
    let (dx, dz) = (p.x - o.x, p.z - o.z);
    let (s, c) = (yaw.sin(), yaw.cos());
    v3(dx * c - dz * s, p.y - o.y, dx * s + dz * c)
}

fn body_to_world(root: &Obj, p: Vec3) -> Vec3 {
    let o = root.pos();
    let yaw = root.rotation().y;
    let (s, c) = (yaw.sin(), yaw.cos());
    v3(o.x + p.x * c + p.z * s, o.y + p.y, o.z - p.x * s + p.z * c)
}

fn rest_head(rig: &BodyRig, i: usize) -> Vec3 {
    let h = rig.def.bones[i].head;
    v3(h[0], h[1], h[2])
}

/// One frame of the hound's body: the trunk and head by mode, the limbs
/// onto their plants.
pub fn animate(g: &mut Game, ei: usize, dt: f64) {
    let pos = g.entities[ei].pos();
    let (vx, vz) = (g.entities[ei].vx, g.entities[ei].vz);
    let speed = hypot2(vx, vz);
    let t = g.perf_t;
    let killer = g.director.killer == Some(ei) && g.state != "playing";
    let Some(c) = g.creature_mut(ei) else { return };
    let rig = c.body.clone();
    let mode = c.brain.mode;
    let mode_t = c.brain.mode_t;
    let run = c.eased("run", ((speed - 2.0) / 3.5).clamp(0.0, 1.0), 4.0, dt);
    let crouch = c.eased(
        "crouch",
        if mode == Mode::Alert || (mode == Mode::Strike && mode_t < WINDUP) || mode == Mode::Idle { 1.0 } else { 0.0 },
        6.0,
        dt,
    );
    // it bays when it is sure; and it lifts its face to you over your body
    let bay = c.eased("bay", if (mode == Mode::Hunt && c.chase_t < 0.9) || killer { 1.0 } else { 0.0 }, 8.0, dt);
    // the gait's phase advances with the distance walked
    let stride = 0.9 + 0.7 * run;
    c.phase = (c.phase + speed * dt / stride).fract();
    let phase = c.phase;
    let duty = 0.72 - 0.25 * run;
    if c.limbs.len() != 4 {
        c.limbs = vec![Limb::default(); 4];
    }
    let root = rig.root.clone();
    rig.rest();
    // ---- trunk: a bob with the gait, lower in the crouch, a lurch when it runs
    let hips = rig.ix("hips").unwrap();
    let h0 = rest_head(&rig, hips);
    let bob = (phase * PI * 4.0).sin() * (0.012 + 0.03 * run);
    rig.bones[hips].set_pos(h0.x, h0.y + bob - 0.11 * crouch, h0.z - 0.05 * crouch);
    let pitch = -0.06 * run + 0.18 * crouch;
    rig.turn(hips, &Quat::from_euler(&Euler::new(pitch, 0.0, (phase * PI * 2.0).sin() * 0.05)));
    if let Some(sp) = rig.ix("spine") {
        rig.turn(sp, &Quat::from_euler(&Euler::new(-0.5 * pitch, 0.0, -(phase * PI * 2.0).sin() * 0.04)));
    }
    // ---- head: hanging and swaying; raised and cocked when it listens; up when it bays
    let cock = if crouch > 0.5 { ((t * 1.3).floor() * 1.7).sin() * 0.35 } else { 0.0 }; // it snaps between cocks
    if let Some(nk) = rig.ix("neck") {
        rig.turn(nk, &Quat::from_euler(&Euler::new(-0.5 * crouch - 0.9 * bay + (phase * PI * 4.0).sin() * 0.05, 0.0, 0.0)));
    }
    if let Some(hd) = rig.ix("head") {
        rig.turn(hd, &Quat::from_euler(&Euler::new(-0.3 * bay, 0.0, cock * crouch)));
    }
    if let Some(jw) = rig.ix("jaw") {
        rig.turn(jw, &Quat::from_euler(&Euler::new(0.45 * bay, 0.0, 0.0)));
    }
    // ---- hair: it hangs to the floor whatever the head does, trailing a little as it moves
    let yaw = root.rotation().y;
    let (sy, cy) = (yaw.sin(), yaw.cos());
    let (bvx, bvz) = (vx * cy - vz * sy, vx * sy + vz * cy);
    for k in 0..3 {
        if let Some(hb) = rig.ix(["hair.0", "hair.1", "hair.2"][k]) {
            let sway = (t * 2.3 + k as f64).sin() * 0.04 * (1.0 + run);
            rig.aim(hb, v3(-bvx * 0.05 + sway, -1.0, -bvz * 0.05), 0.0);
        }
    }
    // ---- limbs: plant, swing, solve
    let fwd_w = v3(vx, 0.0, vz);
    for (k, (up, lo, end, off, side)) in LIMBS.iter().enumerate() {
        let (iu, il, ie) = (rig.ix(up).unwrap(), rig.ix(lo).unwrap(), rig.ix(end).unwrap());
        let rest_end = rest_head(&rig, ie);
        let home_w = body_to_world(&root, v3(rest_end.x, 0.0, rest_end.z));
        let lame = k == 0; // the left hind drags
        let lp = (phase + off).fract();
        let limb = &mut c.limbs[k];
        if !limb.planted || hypot2(limb.at[0] - home_w.x, limb.at[2] - home_w.z) > 1.2 {
            limb.at = [home_w.x, rest_end.y, home_w.z];
            limb.planted = true;
        }
        let swinging = speed > 0.15 && lp > duty;
        if swinging {
            let s = (lp - duty) / (1.0 - duty);
            if limb.swing == 0.0 {
                limb.from = limb.at;
                let ahead = stride * 0.5 * if lame { 0.7 } else { 1.0 };
                let mut d = fwd_w;
                let l = d.length().max(1e-6);
                d.multiply_scalar(ahead / l);
                limb.to = [home_w.x + d.x, rest_end.y, home_w.z + d.z];
            }
            limb.swing = s.max(1e-3);
            let lift = if lame { 0.0 } else { (s * PI).sin() * (0.07 + 0.05 * run) };
            for a in 0..3 {
                limb.at[a] = limb.from[a] + (limb.to[a] - limb.from[a]) * s;
            }
            limb.at[1] = rest_end.y + lift;
        } else {
            if limb.swing > 0.0 {
                limb.at = limb.to;
            }
            limb.swing = 0.0;
            limb.at[1] = rest_end.y;
        }
        let target = world_to_body(&root, v3(limb.at[0], limb.at[1], limb.at[2]));
        let a = rig.rest_len[iu];
        let b = rig.rest_len[il];
        let base = rig.head(iu);
        let front = up.starts_with("upperarm");
        let pole = if front { v3(base.x + side * 0.4, base.y, base.z - 0.6) } else { v3(base.x, base.y + 0.2, base.z + 1.0) };
        let (du, dl) = two_bone(base, target, pole, a, b);
        rig.aim(iu, du, 0.0);
        rig.aim(il, dl, 0.0);
        // the hand flat, the foot as it stood (the dragging one on its toes)
        rig.aim(ie, rig.rest_dir[ie], 0.0);
    }
    let _ = pos;
}

// ------------------------------------------------------------ the voice

/// Its voice: a place in the soundscape that follows it (made at its first
/// sound, wherever that comes from: a call before it has ever moved too).
fn voice_slot(g: &mut Game, ei: usize) -> Option<crate::audio::soundscape::VoiceSlot> {
    let s = g.soundscape()?;
    let id = g.entities[ei].g.id();
    if g.ent_ext.get(&id).and_then(|x| x.voice.as_ref()).is_none() {
        let slot = s.voice_slot(2.2);
        g.ent_ext.entry(id).or_default().voice = Some(EntVoice { slot, next: 1.0, was_hunting: false });
    }
    let slot = g.ent_ext[&id].voice.as_ref()?.slot.clone();
    let pos = g.entities[ei].pos();
    slot.at(pos.x, 0.6, pos.z);
    Some(slot)
}

/// It breaks into its charge: a bay (its tell); learned (you outran it
/// twice, design 4), no bay, only its nails scrabbling as it goes (the tell,
/// nearer), and it runs for where you are heading.
fn bay(g: &mut Game, ei: usize) {
    if g.creature_may_show(ei) {
        g.creature_mut(ei).unwrap().ease.insert("ahead", 1.0);
        g.entity_call(ei, "scrabble");
        g.creature_tell(ei, SCRABBLE_REACH);
    } else {
        g.entity_call(ei, "bark");
        g.creature_tell(ei, BAY_REACH);
    }
}

/// The hound's calls (`entity_call` routes them here): `bark` (the bay),
/// `growl`, `listen` (sniffing), `scrabble` (its nails, breaking into a run).
pub fn call(g: &mut Game, ei: usize, what: &str) {
    let Some(s) = g.soundscape() else { return };
    let Some(v) = voice_slot(g, ei).map(|slot| slot.input.clone()) else { return };
    let t = g.ac_time();
    match what {
        "bark" => {
            // a hoarse bay: a voice that was a person's, falling, with breath through it
            s.tone(&v, t, 1.2, 230.0, 0.17, "sawtooth", Some(150.0), 0.04);
            s.tone(&v, t, 1.2, 460.0, 0.06, "square", Some(305.0), 0.05);
            s.burst(&v, t, 1.3, "bandpass", 950.0, 1.1, 0.3, Some(560.0), 0.05);
            s.burst(&v, t + 1.15, 0.45, "lowpass", 600.0, 0.8, 0.08, Some(300.0), 0.02);
        }
        "growl" => {
            s.tone(&v, t, 0.9, 62.0, 0.12, "sawtooth", Some(50.0), 0.08);
            s.burst(&v, t, 0.9, "lowpass", 320.0, 1.5, 0.09, Some(240.0), 0.08);
        }
        "scrabble" => {
            for k in 0..9 {
                let f = [3400.0, 2900.0, 3800.0][k % 3];
                s.burst(&v, t + k as f64 * 0.045 + [0.0, 0.012, 0.006][k % 3], 0.02, "highpass", f, 1.0, 0.09, None, 0.001);
            }
            s.burst(&v, t, 0.35, "lowpass", 300.0, 0.8, 0.12, None, 0.004);
        }
        "listen" => {
            for k in 0..3 {
                s.burst(&v, t + 0.25 + k as f64 * 0.17, 0.08, "bandpass", 2600.0, 2.0, 0.05, Some(1800.0), 0.01);
            }
        }
        _ => {}
    }
}

/// The hound's sounds from where it is: nails as each hand and foot plants,
/// the dragging foot, panting in a chase, sniffing in a search, a slow wheeze
/// at rest; muffled through walls.
pub fn voice(g: &mut Game, ei: usize, dt: f64, d: f64) {
    let Some(s) = g.soundscape() else { return };
    if g.ac.is_none() || d > 40.0 {
        return;
    }
    let Some(slot) = voice_slot(g, ei) else { return };
    let pos = g.entities[ei].pos();
    let pp = g.player.pos;
    let clear = d < 12.0 && g.e_los(pos.x, pos.z, pp.x, pp.z);
    slot.occlude(if clear { 0.0 } else { 1.0 });
    let v = slot.input.clone();
    let t = g.ac_time();
    let Some(c) = g.creature_mut(ei) else { return };
    let mode = c.brain.mode;
    // a click as each limb comes down, the scrape of the dragging one
    let mut plants = 0;
    let mut drag = false;
    for k in 0..c.limbs.len() {
        let key: &'static str = ["sw0", "sw1", "sw2", "sw3"][k];
        let was = c.ease.get(key).copied().unwrap_or(0.0) > 0.0;
        let now = c.limbs[k].swing > 0.0;
        if was && !now {
            plants += 1;
        }
        if !was && now && k == 0 {
            drag = true;
        }
        c.ease.insert(key, if now { 1.0 } else { 0.0 });
    }
    c.voice_t -= dt;
    let due = c.voice_t <= 0.0;
    if due {
        c.voice_t = match mode {
            Mode::Hunt | Mode::Strike => 0.34,
            Mode::Search => 1.6,
            _ => with_native(|| rand2(3.0, 5.0)),
        };
    }
    for k in 0..plants {
        let tk = t + k as f64 * 0.02;
        s.burst(&v, tk, 0.016, "highpass", 3200.0, 1.0, 0.05, None, 0.001);
        s.burst(&v, tk, 0.05, "lowpass", 320.0, 0.9, 0.035, None, 0.003);
    }
    if drag {
        s.burst(&v, t, 0.38, "bandpass", 1250.0, 0.8, 0.045, Some(850.0), 0.04);
    }
    if due {
        match mode {
            Mode::Hunt | Mode::Strike => s.burst(&v, t, 0.18, "bandpass", 1300.0, 1.4, 0.07, Some(800.0), 0.02),
            Mode::Search => {
                for k in 0..3 {
                    s.burst(&v, t + k as f64 * 0.16, 0.08, "bandpass", 2600.0, 2.0, 0.045, Some(1800.0), 0.01);
                }
            }
            Mode::Alert | Mode::Idle => {}
            _ => s.burst(&v, t, 0.9, "lowpass", 520.0, 0.9, 0.03, Some(300.0), 0.3),
        }
    }
}
