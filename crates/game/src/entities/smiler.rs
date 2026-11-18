//! The Smiler (`docs/design/ENTITIES.md`, design 1): someone very tall, starved
//! down to tendon, gone black as if tarred, its lips dried back off its
//! teeth. *Your lamp calls it; the dark hides you; do not run near it.*
//!
//! - **Senses.** Your lamp, seen from 25 m down any clear line (in its view,
//!   or falling on it); your body only within 8 m and only while you are
//!   lit. Its ears are duller than yours (a crouch at under 3 m; a sound
//!   places you only roughly). Lamp off
//!   and still or crouched, it loses you within about 3 s: it goes on, slower,
//!   to about where your light went out, and strikes there only if it finds
//!   you (sees, hears or bumps into you).
//! - **Modes.** Idle (in its dark, facing a corner, swaying) → Alert (it has
//!   seen a light: its head snaps to it, then its body; a click of teeth) →
//!   Search (to where the light was; there it listens) → Hunt (you are lit
//!   and in its sight: it comes at 4.5 m/s, half that where it or you stand
//!   in a bright room) → Strike (you ran within 8 m of it, or let it reach
//!   you: a sharp breath in, 0.5 s, coiling as it comes, then a lunge at
//!   7 m/s over at most 4 m) → Withdraw. Sent by the director it Stalks
//!   toward your part of the level through the dark.
//! - **Eyes.** Eyeshine is your lamp reflected: each frame its eyes' light
//!   is how much of your beam falls on them and how squarely they face you
//!   (`eyeshine`); it never glows on its own.
//! - **Body.** `bodies.bin`'s `smiler` (`assets/src/creatures/body_smiler.py`),
//!   its stoop the rest pose; it moves by Kimodo's clips of a person
//!   (`assets/motion/smiler.motion`, `entities::motion`) with its wrongness
//!   laid on by rule ([`pose`]): the head first and the body after, held
//!   frames and snaps, the arms hanging dead.
use crate::base::numeric::hypot2;
use crate::entities::body::BodyRig;
use crate::entities::brain::*;
use crate::entities::creature::*;
use crate::entities::motion::Mind;
use crate::entities::motion::anim::{Anim, Drive, Pick};
use crate::entities::motion::retarget::Gains;
use crate::entities::{EntVoice, with_skin_detail, wrap_pi};
use crate::prelude::*;

pub const DATA: KindData = KindData {
    sight: 8.0,
    sight_lit: 8.0,
    cone: 0.35,
    hearing: 0.8,
    smell: 0.0,
    radius: 0.32,
    reach: 1.3,
    sight_gain: 1.6,
    sound_gain: 0.9,
    forget: 0.35,
    touch_nerve: 0.0,
};

/// How far it sees your lamp, m.
pub const LAMP_SIGHT: f64 = 25.0;
/// Within this, running makes it lunge, m.
pub const PANIC_RANGE: f64 = 8.0;
/// How bright a place it shies from (the light map).
pub const BRIGHT: f64 = 0.55;
const CREEP: f64 = 1.6;
/// Sent, it creeps toward you until this near, m (nearer than it sees your
/// lit body: sent, it finds you, and its click gives you the moment to go dark).
const STALK_NEAR: f64 = 5.0;
/// Told where you are, it takes a new hint at most this often, s.
const HINT_EVERY: f64 = 5.0;
const STALK: f64 = 3.2;
const SEARCH: f64 = 2.6;
const HUNT: f64 = 4.5;
const LUNGE: f64 = 7.0;
const WINDUP: f64 = 0.5;
const BREATH_REACH: f64 = 12.0;
/// It feels you this close, in the dark or not, m.
const TOUCH: f64 = 0.8;
const CLICK_REACH: f64 = 16.0;
/// Once it has learned (you hid from it in the dark twice, design 4),
/// it waits this long where you went dark, silent, before it searches, s.
pub const WAITS: f64 = 6.0;

thread_local! {
    static MATS: std::cell::RefCell<std::collections::HashMap<String, Mat>> = Default::default();
}

/// Its materials by slot: the skin black, wet and oily (a tight highlight
/// on the bone); the teeth dry ivory; the eyes dark until your lamp finds
/// them (a material of their own each, for their eyeshine).
pub fn material(slot: &str) -> Mat {
    if slot == "eye" {
        // a fresh one per eye: `eyeshine` sets its light every frame
        return phong!({ color: 0x0a0806, emissive: 0xd9a441, specular: 0x404040, shininess: 220 });
    }
    MATS.with(|m| {
        m.borrow_mut()
            .entry(slot.to_string())
            .or_insert_with(|| match slot {
                "teeth" => phong!({ color: 0xffffff, vertexColors: true, specular: 0x222018, shininess: 30 }),
                _ => with_skin_detail(phong!({ color: 0xffffff, vertexColors: true, specular: 0x262420, shininess: 40 }), 3.1, 1.05),
            })
            .clone()
    })
}

fn set(g: &mut Game, ei: usize, to: Mode, why: &'static str) {
    let now = g.perf_t;
    if let Some(c) = g.creature_mut(ei) {
        c.brain.set(to, why, now);
        c.ease.insert("scan_set", 0.0);
    }
}

/// A dark place to go: a tile in the open, darker the better, some way off
/// (`away`: far from you too, 20 m or more, the further the better).
fn dark_goal(g: &Game, ei: usize, away: bool) -> Option<(f64, f64)> {
    let pos = g.entities[ei].pos();
    let pp = g.player.pos;
    let mut best = None;
    let mut best_score = -1e9;
    for _ in 0..14 {
        let (gx, gz) = (with_native(|| irand(1, g.w as i64 - 2)) as i32, with_native(|| irand(1, g.h as i64 - 2)) as i32);
        if !g.in_grid(gx, gz) || g.is_wall_tile(gx, gz) {
            continue;
        }
        let (wx, wz) = (g.wx(gx), g.wz(gz));
        let d = hypot2(wx - pos.x, wz - pos.z);
        if !(6.0..40.0).contains(&d) {
            continue;
        }
        let dp = hypot2(wx - pp.x, wz - pp.z);
        if away && dp < 20.0 {
            continue;
        }
        let score = -g.e_region_light_at(gx, gz) * 4.0 - (d - 18.0).abs() * 0.02 + if away { dp.min(50.0) * 0.1 } else { 0.0 };
        if score > best_score {
            best_score = score;
            best = Some((wx, wz));
        }
    }
    best
}

/// One frame of the Smiler's mind.
pub fn think(g: &mut Game, ei: usize, dt: f64) {
    let Some(c) = g.creature(ei) else { return };
    let (mode, mode_t, order, p) = (c.brain.mode, c.brain.mode_t, c.brain.order, c.brain.last);
    let pos = g.entities[ei].pos();
    let pp = g.player.pos;
    let now = g.perf_t;
    let relaxed = g.director.phase == crate::entities::director::Phase::Relax || matches!(order, Order::Away(_));
    let dull = if relaxed { 0.55 } else { 1.0 };
    // ---- the senses: your lamp from far off, your lit body near, your steps
    let lamp_seen = p.lamp && p.line && p.dist < LAMP_SIGHT * g.body.lamp_seen * dull && (p.in_view || p.lamp_on_it);
    let lit = p.lamp || p.you_lit > 0.35;
    let body_seen = lit && p.line && p.in_view && p.dist < PANIC_RANGE;
    let hear_range = g.steps_heard() * DATA.hearing * dull;
    let heard = hear_range > 0.0 && p.path < hear_range;
    // it walked into you in the dark
    let touched = p.dist < TOUCH;
    let sighted = lamp_seen || body_seen || touched;
    let mut feed = 0.0;
    if lamp_seen {
        feed += DATA.sight_gain * (1.0 - p.dist / (LAMP_SIGHT * g.body.lamp_seen)).max(0.25);
    }
    if body_seen || touched {
        feed += DATA.sight_gain;
    }
    if heard {
        feed += DATA.sound_gain * ((hear_range - p.path) / (hear_range * 0.35).max(0.5)).clamp(0.0, 1.0);
    }
    if mode == Mode::Withdraw {
        feed = 0.0;
    }
    // where it thinks you are: where it saw you; a sound only roughly; and
    // when your light goes out, only roughly where it last was (the dark
    // hides you: it has to look)
    let was_sighted = g.creature(ei).unwrap().get("sighted") > 0.0;
    let rough = |g: &Game, near: f64, far: f64| {
        let (a, r) = with_native(|| (rnd() * PI * 2.0, near + rnd() * (far - near)));
        g.open_point(pp.x + a.dcos() * r, pp.z + a.dsin() * r)
    };
    let guess = if feed > 0.0 && sighted {
        Some((pp.x, pp.z))
    } else if feed > 0.0 {
        Some(rough(g, 1.0, 3.0))
    } else if was_sighted {
        Some(rough(g, 1.5, 3.0))
    } else {
        None
    };
    {
        let c = g.creature_mut(ei).unwrap();
        c.ease.insert("sighted", if sighted && feed > 0.0 { 1.0 } else { 0.0 });
        if let Some((x, z)) = guess {
            c.brain.mem.seen = Some((x, z, now));
        }
        if feed > 0.0 {
            c.brain.feed(feed, dt, 1.0);
        } else {
            c.brain.drain(DATA.forget, dt);
        }
    }
    let sensing = feed > 0.0;
    let aw = g.creature(ei).unwrap().brain.mem.awareness;
    let seen = g.creature(ei).unwrap().brain.mem.seen;
    // in bright places it is slow: where it stands, or coming at you into one
    let bright = p.it_lit > BRIGHT || p.you_lit > BRIGHT;
    let pace = |v: f64| if bright { v * 0.5 } else { v };

    match mode {
        Mode::Withdraw => {
            let goal = g.creature(ei).unwrap().brain.goal.or_else(|| dark_goal(g, ei, true));
            g.creature_mut(ei).unwrap().brain.goal = goal;
            g.creature_go(ei, goal, false, pace(STALK), dt);
            let away = matches!(order, Order::Away(t) if t > 0.0);
            if !away && mode_t > 8.0 {
                g.creature_mut(ei).unwrap().brain.goal = None;
                set(g, ei, Mode::Idle, "back in the dark");
            }
        }
        Mode::Idle | Mode::Roam | Mode::Stalk => {
            if aw >= SUSPECT {
                set(g, ei, Mode::Alert, "it saw a light");
                g.creature_tell(ei, CLICK_REACH);
                call(g, ei, "click");
                return;
            }
            if let Order::Away(_) = order {
                g.creature_mut(ei).unwrap().brain.goal = None;
                set(g, ei, Mode::Withdraw, "the director sent it away");
                return;
            }
            if let Order::Seek(x, z) = order {
                let last = g.creature(ei).unwrap().get("hint_t");
                if last == 0.0 || now - last > HINT_EVERY {
                    hint(g, ei, x, z);
                    set(g, ei, Mode::Search, "it was told where you are");
                    return;
                }
            }
            let stalk = order == Order::Stalk;
            if stalk && mode != Mode::Stalk {
                set(g, ei, Mode::Stalk, "the director sent it");
            } else if !stalk && mode == Mode::Stalk {
                set(g, ei, Mode::Roam, "no longer sent");
            }
            // sent, it goes to where the director would have it wait, out of
            // your sight; there it creeps on toward you through the dark (it
            // hunts by sight: from a place you cannot see, it cannot see you)
            let lurk = stalk && {
                let wt = g.entities[ei].w_target.map(|(x, z)| (g.wx(x), g.wz(z)));
                wt.is_none_or(|(x, z)| hypot2(x - pos.x, z - pos.z) < 2.5)
            };
            if lurk {
                if p.dist > STALK_NEAR {
                    g.creature_go(ei, None, true, pace(CREEP), dt);
                } else {
                    scan(g, ei, mode_t, dt);
                }
                return;
            }
            let goal = if stalk {
                g.entities[ei].w_target.map(|(x, z)| (g.wx(x), g.wz(z)))
            } else {
                let c = g.creature(ei).unwrap();
                let reached = c.brain.goal.is_none_or(|(x, z)| hypot2(x - pos.x, z - pos.z) < 2.0);
                if mode == Mode::Idle {
                    // it stands in its dark a while, then drifts to another
                    if mode_t > 12.0 {
                        set(g, ei, Mode::Roam, "it drifts");
                    }
                    None
                } else {
                    // (a goal it cannot get to is given up after a while)
                    if reached || c.brain.goal_t <= 0.0 {
                        let gl = dark_goal(g, ei, false);
                        let c = g.creature_mut(ei).unwrap();
                        c.brain.goal = gl;
                        c.brain.goal_t = with_native(|| rand2(14.0, 22.0));
                        if mode_t > 4.0 {
                            set(g, ei, Mode::Idle, "it stands in the dark");
                        }
                    }
                    g.creature_mut(ei).unwrap().brain.goal_t -= dt;
                    g.creature(ei).unwrap().brain.goal
                }
            };
            if goal.is_some() {
                g.creature_go(ei, goal, false, pace(if stalk { STALK } else { CREEP }), dt);
            } else {
                g.entity_drive(ei, dt, 0.0);
            }
        }
        Mode::Alert => {
            // its head is already on the light; the body comes round after
            g.entity_drive(ei, dt, 0.0);
            let h = (pp.x - pos.x).datan2(pp.z - pos.z);
            if mode_t > 0.5 {
                g.entity_turn(ei, h, dt, Some(2.0));
            }
            if sure(aw) && mode_t > 0.6 {
                notice(g, ei);
            } else if mode_t > 1.4 && aw < SUSPECT + 0.2 {
                set(g, ei, Mode::Search, "the light went out");
            } else if mode_t > 2.5 {
                set(g, ei, Mode::Search, "it goes to look");
            }
        }
        Mode::Search => {
            if sure(aw) {
                notice(g, ei);
                return;
            }
            if let Order::Seek(x, z) = order
                && now - g.creature(ei).unwrap().get("hint_t") > HINT_EVERY
            {
                hint(g, ei, x, z);
                set(g, ei, Mode::Search, "told again where you are");
                return;
            }
            if aw >= SUSPECT && mode_t > 1.0 && (lamp_seen || body_seen) {
                set(g, ei, Mode::Alert, "the light again");
                g.creature_tell(ei, CLICK_REACH);
                call(g, ei, "click");
                return;
            }
            let goal = g.creature(ei).unwrap().brain.goal.or(seen.map(|(x, z, _)| (x, z)));
            let at = goal.is_none_or(|(x, z)| hypot2(x - pos.x, z - pos.z) < 1.5);
            let told = matches!(order, Order::Seek(..));
            if at {
                // there it waits in the dark, listening, looking round
                scan(g, ei, mode_t, dt);
            } else {
                // (told where you are, it goes at its stalking pace)
                g.creature_go(ei, goal, false, pace(if told { STALK } else { SEARCH }), dt);
            }
            if mode_t > 16.0 {
                g.creature_mut(ei).unwrap().brain.goal = None;
                set(g, ei, Mode::Roam, "it gave up looking");
            }
        }
        Mode::Hunt => {
            if aw < SUSPECT {
                // you hid from it in the dark, your lamp out: what it learns from
                if !p.lamp {
                    g.creature_habit("smiler");
                }
                g.creature_mut(ei).unwrap().brain.goal = seen.map(|(x, z, _)| (x, z));
                if g.creature_may_show(ei) {
                    set(g, ei, Mode::Hold, "it waits where you went dark");
                } else {
                    set(g, ei, Mode::Search, "it lost you in the dark");
                }
                return;
            }
            let allowed = g.strike_allowed(ei);
            let running = g.player.audible_range >= 12.0;
            if allowed && ((running && p.dist < PANIC_RANGE) || (p.dist < 1.8 && sensing)) {
                set(g, ei, Mode::Strike, if running { "you ran" } else { "you let it reach you" });
                g.creature_mut(ei).unwrap().burst_m = 0.0;
                call(g, ei, "inhale");
                g.creature_tell(ei, BREATH_REACH);
                return;
            }
            if p.dist < 1.4 && sensing {
                g.entity_drive(ei, dt, 0.0);
            } else if body_seen || lamp_seen {
                g.creature_go(ei, None, true, pace(HUNT), dt);
            } else {
                // it has lost sight of you: to where it thinks you are, slower
                let goal = seen.map(|(x, z, _)| (x, z));
                g.creature_go(ei, goal, false, pace(if sensing { HUNT } else { SEARCH }), dt);
            }
            // its breath, close: a tell
            let c = g.creature_mut(ei).unwrap();
            if c.get("breath_t") <= 0.0 {
                c.ease.insert("breath_t", 2.2);
                if p.dist < BREATH_REACH {
                    g.creature_tell(ei, BREATH_REACH);
                }
            } else {
                let v = c.get("breath_t") - dt;
                c.ease.insert("breath_t", v);
            }
        }
        Mode::Strike => {
            if mode_t < WINDUP {
                // the inhale: it coils, still coming
                g.creature_go(ei, None, true, pace(SEARCH), dt);
            } else {
                if g.creature(ei).unwrap().burst_m == 0.0 {
                    let (dx, dz) = (pp.x - pos.x, pp.z - pos.z);
                    let l = hypot2(dx, dz).max(1e-6);
                    let e = &mut g.entities[ei];
                    e.vx = dx / l * LUNGE;
                    e.vz = dz / l * LUNGE;
                    call(g, ei, "lunge");
                }
                g.creature_burst(ei, LUNGE, dt);
                let p = g.perceive(ei);
                if p.dist < DATA.reach {
                    g.creature_contact(ei);
                } else if g.creature(ei).unwrap().burst_m >= BURST_LEN || mode_t > WINDUP + 1.0 {
                    let e = &mut g.entities[ei];
                    e.vx = 0.0;
                    e.vz = 0.0;
                    set(g, ei, Mode::Hunt, "the lunge missed");
                }
            }
        }
        Mode::Hold => {
            // learned: it waits where you went dark, silent, no clicks; your
            // lamp on now, anywhere it could see it, and it has you
            g.entity_drive(ei, dt, 0.0);
            if (p.lamp && p.line && p.dist < LAMP_SIGHT * g.body.lamp_seen) || body_seen || touched {
                g.creature_mut(ei).unwrap().brain.mem.awareness = 1.0;
                notice(g, ei);
            } else if mode_t > WAITS {
                set(g, ei, Mode::Search, "it gave up waiting");
            }
        }
        _ => set(g, ei, Mode::Idle, "back in the dark"),
    }
}

/// The director's hint of where you are: a place 4–7 m from you (never in
/// a wall), taken at most every `HINT_EVERY` s; it goes there to look.
fn hint(g: &mut Game, ei: usize, x: f64, z: f64) {
    let (a, r) = with_native(|| (rnd() * PI * 2.0, rand2(4.0, 7.0)));
    let (hx, hz) = g.open_point(x + a.dcos() * r, z + a.dsin() * r);
    let now = g.perf_t;
    let c = g.creature_mut(ei).unwrap();
    c.ease.insert("hint_t", now);
    c.brain.goal = Some((hx, hz));
}

/// It stands and looks round: its body turns slowly one way and back, so
/// its eyes sweep the dark.
fn scan(g: &mut Game, ei: usize, t: f64, dt: f64) {
    g.entity_drive(ei, dt, 0.0);
    let yaw = g.entities[ei].yaw;
    let Some(c) = g.creature_mut(ei) else { return };
    // (the way it faced when it began, kept until its mode changes: `set`)
    if c.get("scan_set") == 0.0 {
        c.ease.insert("scan_from", yaw);
        c.ease.insert("scan_set", 1.0);
    }
    let base = c.get("scan_from");
    g.entity_turn(ei, wrap_pi(base + 1.6 * (t * 0.45).dsin()), dt, Some(0.9));
}

/// It is sure: a hiss through its teeth, and it comes.
fn notice(g: &mut Game, ei: usize) {
    let now = g.perf_t;
    if let Some(c) = g.creature_mut(ei) {
        // each notice: no strike in the 2 s after it
        c.noticed_at = Some(now);
    }
    set(g, ei, Mode::Hunt, "it is sure where you are");
    g.director_note("notice");
    call(g, ei, "hiss");
    g.creature_tell(ei, CLICK_REACH);
    let d = g.perceive(ei).dist;
    if d < 30.0 {
        g.dread_sting((1.3 - d / 30.0).clamp(0.5, 1.0));
    }
}

// ------------------------------------------------------------ the body

/// One frame of its body: Kimodo's clips on its own body (`entities::motion`)
/// and what makes the motion its own laid on top ([`pose`]).
pub fn animate(g: &mut Game, ei: usize, dt: f64) {
    eyeshine(g, ei);
    crate::entities::motion::with_anim(g, ei, false, "idle", dt, pose);
}

/// How much of its bound stoop it stands out of, and how heavy it moves
/// (`motion::retarget::Retarget::upright`, `motion::anim::Anim::mass`):
/// tall, a little stooped under the ceilings, heavy on its feet.
const UPRIGHT: f64 = 0.55;
const MASS: f64 = 1.0;

/// The Smiler's motion, the mocap's weight under it and its wrongness on
/// top (design §6: two vocabularies only, stillness and wrong mechanics):
///
/// - **Stillness.** Waiting it keeps a part of the mocap's sway; it
///   breathes slowly; nothing else.
/// - **Its stoop stays.** The clips are laid on its rest pose, the stoop.
/// - **The head first, the body after.** Its head goes to what it watches
///   in a snap and holds there; the body comes round later.
/// - **Held frames and snaps.** Searching (and now and then waiting) it
///   stops dead mid-step, holds, and lurches on to where it would have been.
/// - **Arms that hang dead.** Hunting, the legs run and the arms swing as
///   dead weight, until the lunge reaches.
/// - Its jaw opens as it comes; its fingers clench as it coils, splay as it
///   reaches, and twitch one by one while it stands over you.
pub fn pose(a: &mut Anim, rig: &BodyRig, d: &Drive, m: &Mind) {
    a.rt.upright = UPRIGHT;
    a.mass = MASS;
    let w = a.wrong;
    let moving = a.eased("moving", if d.speed() > 0.25 { 1.0 } else { 0.0 }, 8.0, d.dt) > 0.5;
    let still = Gains { trunk: 1.0 - 0.6 * w, head: 1.0 - 0.5 * w, arms: 1.0 - 0.7 * w, legs: 1.0, bob: 1.0 };
    let dead_arms = Gains { trunk: 1.0, head: 1.0 - 0.6 * w, arms: 1.0 - 0.65 * w, legs: 1.0, bob: 1.0 };
    let listening = m.mode == Mode::Search && !moving;
    let pick = if m.killer {
        // out of a strike into stillness slowly: no pop
        Pick::new("stand_over").fade(0.7)
    } else {
        match m.mode {
            Mode::Strike if m.mode_t < WINDUP => Pick::new("lunge").from(0.3).gains(dead_arms).travel(),
            Mode::Strike => Pick::new("lunge").from(0.8).travel(),
            Mode::Hunt if moving && d.speed() > STALK + 0.3 => Pick::new("run").gains(dead_arms).travel(),
            Mode::Hunt | Mode::Stalk if moving => Pick::new("stalk").gains(dead_arms).travel(),
            Mode::Search if moving => Pick::new("creep").travel().holds(1.2, 2.4),
            Mode::Search => Pick::new("listen").gains(still),
            Mode::Roam | Mode::Withdraw if moving => Pick::new("creep").travel(),
            Mode::Alert => Pick::new("look").gains(still),
            _ => Pick::new("idle").gains(still).holds(3.0, 5.0),
        }
    };
    a.play(rig, &pick, d);
    // ---- the head on what it watches, in snaps; tilted as it listens
    let watching = matches!(m.mode, Mode::Alert | Mode::Hunt | Mode::Strike) || m.killer || (listening && m.mode_t > 1.6);
    let tilt = if m.killer {
        0.3 * w
    } else if listening {
        0.38 * w * (m.mode_t / 0.6).min(1.0)
    } else {
        0.0
    };
    a.breathe(rig, 0.02, 1.4);
    a.look(rig, watching.then_some(m.eye), d, 0.25, (1.5, 0.7), 0.4, tilt);
    // ---- the fingers: loose; clenching as it coils, splayed past straight
    // as it reaches, twitching one by one while it stands over you
    let curl = match m.mode {
        _ if m.killer => 0.35,
        Mode::Strike if m.mode_t < WINDUP => 0.25 + 0.5 * (m.mode_t / WINDUP),
        Mode::Strike => -0.18,
        Mode::Hunt => 0.12,
        _ => 0.28,
    };
    let t = a.time;
    let killer = m.killer;
    a.curl(rig, curl, &|f| if killer { 0.35 * (t * 2.3 + f as f64 * 1.7).dsin().max(0.0).dpowi(3) * w } else { 0.0 });
    // ---- the jaw a little open as it comes, wide in the lunge
    let open = match m.mode {
        _ if m.killer => 0.25,
        Mode::Hunt => 0.12,
        Mode::Strike => 0.32,
        _ => 0.0,
    } * w;
    let open = a.eased("jaw", open, 8.0, d.dt);
    a.jaw(rig, open);
}

/// Its eyes give back your lamp: how far inside your beam they are, how
/// squarely they face you, nearer brighter. Never a glow of their own.
pub fn eyeshine(g: &mut Game, ei: usize) {
    let Some(c) = g.creature(ei) else { return };
    let rig = c.body.clone();
    let lamp = g.player.flashlight_on;
    let cam = g.camera.pos();
    let f = g.player.fwd();
    let has_head = rig.ix("head").is_some();
    for name in ["eye.L", "eye.R"] {
        let Some(m) = rig.meshes.get(name) else { continue };
        let mut k = 0.0;
        if lamp && has_head {
            let p = m.world_position();
            let (dx, dy, dz) = (p.x - cam.x, p.y - cam.y, p.z - cam.z);
            let d = (dx * dx + dy * dy + dz * dz).sqrt().max(1e-6);
            let in_beam = ((f.x * dx + f.y * dy + f.z * dz) / d - 0.86) / 0.11;
            // how squarely it faces you: its head's forward against the way to you
            let yaw = rig.root.rotation().y;
            let facing = -(yaw.dsin() * dx + yaw.dcos() * dz) / (dx * dx + dz * dz).sqrt().max(1e-6);
            k = in_beam.clamp(0.0, 1.0) * ((facing - 0.3) / 0.6).clamp(0.0, 1.0) * 1.8 / (1.0 + d * d / 120.0);
        }
        for mat in m.mats() {
            mat.m().emissive_intensity = k;
        }
    }
}

// ------------------------------------------------------------ the voice

/// Its voice: a place in the soundscape that follows its head.
fn voice_slot(g: &mut Game, ei: usize) -> Option<crate::audio::soundscape::VoiceSlot> {
    let s = g.soundscape()?;
    let id = g.entities[ei].g.id();
    if g.ent_ext.get(&id).and_then(|x| x.voice.as_ref()).is_none() {
        let slot = s.voice_slot(2.2);
        g.ent_ext.entry(id).or_default().voice = Some(EntVoice { slot, next: 1.0, was_hunting: false });
    }
    let slot = g.ent_ext[&id].voice.as_ref()?.slot.clone();
    let pos = g.entities[ei].pos();
    slot.at(pos.x, 2.0, pos.z);
    Some(slot)
}

/// Its sounds: `click` (teeth, when it shifts), `hiss` (sure), `inhale`
/// (before a lunge), `lunge`.
pub fn call(g: &mut Game, ei: usize, what: &str) {
    let Some(s) = g.soundscape() else { return };
    let Some(v) = voice_slot(g, ei).map(|slot| slot.input.clone()) else { return };
    let t = g.ac_time();
    match what {
        "click" => {
            for k in 0..2 {
                s.burst(&v, t + k as f64 * 0.07, 0.012, "highpass", 3600.0, 1.0, 0.09, None, 0.001);
            }
        }
        "hiss" => {
            s.burst(&v, t, 0.9, "bandpass", 4200.0, 1.6, 0.16, Some(3000.0), 0.05);
            s.burst(&v, t + 0.1, 0.8, "lowpass", 600.0, 0.9, 0.08, Some(400.0), 0.2);
        }
        "inhale" => {
            s.burst(&v, t, 0.7, "bandpass", 900.0, 2.2, 0.14, Some(2400.0), 0.5);
        }
        "lunge" => {
            s.burst(&v, t, 0.5, "bandpass", 1500.0, 1.5, 0.26, Some(800.0), 0.01);
            for k in 0..4 {
                s.burst(&v, t + 0.05 + k as f64 * 0.06, 0.015, "highpass", 3400.0, 1.0, 0.08, None, 0.001);
            }
        }
        _ => {}
    }
}

/// Slow wet breathing where it is (faster when it comes), nothing when it
/// waits; muffled through walls.
pub fn voice(g: &mut Game, ei: usize, dt: f64, d: f64) {
    if g.ac.is_none() || d > 30.0 {
        return;
    }
    let Some(s) = g.soundscape() else { return };
    let Some(slot) = voice_slot(g, ei) else { return };
    let pos = g.entities[ei].pos();
    let pp = g.player.pos;
    let clear = d < 12.0 && g.e_los(pos.x, pos.z, pp.x, pp.z);
    slot.occlude(if clear { 0.0 } else { 1.0 });
    let v = slot.input.clone();
    let t = g.ac_time();
    let Some(c) = g.creature_mut(ei) else { return };
    let mode = c.brain.mode;
    c.voice_t -= dt;
    if c.voice_t > 0.0 {
        return;
    }
    c.voice_t = match mode {
        Mode::Hunt | Mode::Strike => with_native(|| rand2(1.1, 1.6)),
        Mode::Idle | Mode::Search | Mode::Hold => with_native(|| rand2(4.0, 7.0)),
        _ => with_native(|| rand2(2.4, 3.8)),
    };
    if matches!(mode, Mode::Idle | Mode::Search) && d > 6.0 || mode == Mode::Hold {
        return; // waiting, it is silent
    }
    s.burst(&v, t, 0.9, "lowpass", 420.0, 0.9, 0.05, Some(260.0), 0.35);
    s.burst(&v, t + 1.0, 0.7, "lowpass", 520.0, 0.9, 0.035, Some(300.0), 0.12);
    if matches!(mode, Mode::Hunt | Mode::Strike) {
        for k in 0..2 {
            s.burst(&v, t + 0.2 + k as f64 * 0.07, 0.012, "highpass", 3400.0, 1.0, 0.05, None, 0.001);
        }
    }
}
