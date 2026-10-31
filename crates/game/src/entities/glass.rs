//! The one behind the glass (`docs/WINDOWS.md` 4): someone in a pale
//! raincoat, the hood up, standing far out on the track one window of a
//! floor looks out on. *Do not stare out of a window too long.*
//!
//! - **Placed**, never sent (`place`): behind one window of about four in
//!   ten of the floors with windows (by the level's seed, no draws), never
//!   where an anomaly is; its window off the way from the arrival to the
//!   door, far from the arrival, her camp and the door.
//! - **Out there** it stands on the track, `REST` m out, and **always faces
//!   you**. It is drawn beyond the glass (`Material::portal`: what is past
//!   the window's plane is seen only through its opening).
//! - **Your look** (`eyes_on_glass`): the line of your view through one of
//!   its window's lights, within `LOOK_REACH`, nothing between, the glass
//!   uncovered. It adds up (a second a second while you look, draining
//!   `STARE_DRAIN` as fast when you do not).
//! - At `NOTICE` it has seen you: the pane ticks (a tell). At `COME` it
//!   comes up the track while your eyes are on its glass, and stands when
//!   they are not; it never goes back. In its lamp's light (`LAMP_LIGHT` m
//!   out) it no longer needs your look and comes to the glass: its face to
//!   the pane, the glass creaking, its breath misting it (tells). After
//!   `PRESS_T`, unless the glass is covered or the place lies quiet after a
//!   close call, it comes through: the pane bows in round it like film and
//!   does not break, and it comes down onto the floor inside, into its
//!   lamp's light.
//! - **Inside** it can be only where its lamp's light falls through its
//!   window (`Window::lit_at`): it goes to the edge of the light nearest you
//!   and takes you if you are in it (`creature_contact`: its touch takes
//!   your mind, `touch_nerve`), and you are thrown down out of its light.
//!   Out of its sight `LOSES` s, or `FAR` off, it goes back out through the
//!   glass and stands in the lamp's light, close.
//! - **Its sound** is glass, and its coat: the tick (a knock and the pane's
//!   ring), the creak, wet skin squeaking on the pane, the long groan of
//!   the glass bowing; inside, its wet raincoat rustling (`call`).
//! - **Body**: `bodies.bin`'s `glass` (`assets/src/creatures/body_glass.py`).
//!
//! After SCP-2885 "Window to Another You" (Pythy, rewritten by Zyn),
//! SCP-3052 (Witryso), SCP-L682 (CRAt26) and the Backrooms wiki's Entity 2
//! "The Windows" (u/glassydude101, rewritten by DrAkimoto), all CC BY-SA
//! 3.0: this file and what it draws on them are CC BY-SA 3.0 and keep this
//! attribution (`docs/LORE.md`, Attribution). Kunekune (a motif); Gogol's
//! *Viy* and Brontë's *Wuthering Heights* (public domain).
use crate::base::numeric::hypot2;
use crate::core_props::window::view::GROUND;
use crate::entities::body::two_bone;
use crate::entities::brain::*;
use crate::entities::creature::*;
use crate::entities::{with_skin_detail, EntVoice};
use crate::prelude::*;

pub const DATA: KindData = KindData {
    sight: 0.0,
    sight_lit: 0.0,
    cone: 0.0,
    hearing: 0.0,
    smell: 0.0,
    radius: 0.3,
    reach: 0.85,
    sight_gain: 0.0,
    sound_gain: 0.0,
    forget: 0.0,
    touch_nerve: 40.0,
};

/// Your eyes' time on its glass drains this share as fast when they are
/// not on it
pub const STARE_DRAIN: f64 = 0.25;
/// Your look counts within this of its glass (along your view), m
pub const LOOK_REACH: f64 = 8.0;
/// At this much look it has seen you (the tick); at this it comes, s
pub const NOTICE: f64 = 3.0;
pub const COME: f64 = 5.0;
/// It stands this far out on the track at first, m (in the fog there, a
/// smudge you are not sure of: at 55 m the fog had taken it); it comes this
/// fast while you look, m/s; within this of the glass (its lamp's light) it
/// comes on its own, m
pub const REST: f64 = 32.0;
pub const WALK: f64 = 2.6;
pub const LAMP_LIGHT: f64 = 7.0;
/// At the glass its feet are this far out, m (leaning, its hood's rim on
/// the pane and its face just behind it), and it presses this long before
/// it comes through, s
pub const AT_GLASS: f64 = 0.20;
pub const PRESS_T: f64 = 2.0;
/// Its coming through, and its going back, s
pub const THROUGH_T: f64 = 2.4;
pub const BACK_T: f64 = 2.0;
/// Inside it moves this fast within its light, m/s
pub const PACE: f64 = 1.8;
/// Out of its sight this long, or this far, it goes back out, s / m; and
/// when it has seen you keep out of its light this long, s
pub const LOSES: f64 = 4.0;
pub const KEPT_OUT: f64 = 12.0;
pub const FAR: f64 = 12.0;
/// Its sounds, and its tells, carry this far from its glass, m
pub const HEARD: f64 = 9.0;
/// Once it has learned, its lamp hangs this much lower, m, and the first
/// time comes down over this long, s
pub const LAMP_DROP: f64 = 1.0;
pub const LOWERING: f64 = 3.0;
/// Its face on the glass: how high over the ground outside, m
const FACE_H: f64 = 1.52;

/// Its materials: pale, wet skin; a pale raincoat, wet; rubber boots; dark
/// trousers; the eyes. Its own (not shared: `place` draws them beyond its
/// window's glass, and a showcase's are not).
pub fn material(slot: &str) -> Mat {
    match slot {
        // rubberised cotton, wet: a sheen, not a matte habit
        "coat" => phong!({ color: 0xffffff, vertexColors: true, specular: 0x6a6a64, shininess: 64, side: DoubleSide }),
        "boots" => phong!({ color: 0xffffff, vertexColors: true, specular: 0x2c2c2a, shininess: 30 }),
        "cloth" => lambert!({ color: 0xffffff, vertexColors: true }),
        "eye" => phong!({ color: 0xffffff, vertexColors: true, specular: 0x666666, shininess: 110 }),
        // (wet)
        _ => with_skin_detail(phong!({ color: 0xffffff, vertexColors: true, specular: 0x5c5e60, shininess: 58 }), 3.2, 1.0),
    }
}

/// Which window it is behind (`Game::behind_glass`)
#[derive(Clone, Debug, Default)]
pub struct Bind {
    /// the creature, its window (`Game::windows`) and that window's lights
    /// (the one or two it presses on and comes through), and its row's
    /// lights (it is seen through all of them: your look counts through
    /// any, and its light is all of theirs)
    pub ei: usize,
    pub win: usize,
    pub lights: Vec<usize>,
    pub row: Vec<usize>,
    /// the middle of its lights across (the window's frame): its track
    pub x: f64,
    /// each of its lights' panes at rest (a fine grid, so it can bow)
    pub panes: Vec<Vec<f32>>,
}

fn set(g: &mut Game, ei: usize, to: Mode, why: &'static str) {
    let now = g.perf_t;
    if let Some(c) = g.creature_mut(ei) {
        c.brain.set(to, why, now);
    }
}

fn put(g: &mut Game, ei: usize, key: &'static str, v: f64) {
    if let Some(c) = g.creature_mut(ei) {
        c.ease.insert(key, v);
    }
}

fn get(g: &Game, ei: usize, key: &'static str) -> f64 {
    g.creature(ei).map(|c| c.get(key)).unwrap_or(0.0)
}

/// Its binding, if `ei` is the one behind the glass of this level
fn bound(g: &Game, ei: usize) -> Option<Bind> {
    g.behind_glass.as_ref().filter(|b| b.ei == ei).cloned()
}

/// Whether creature `ei` is the one behind the glass and still out there
/// (nothing of it reaches the room: the screen's pressure, a sighting)
pub fn outside(g: &Game, ei: usize) -> bool {
    g.entities[ei].kind == "glass" && get(g, ei, "inside") <= 0.0
}

/// How the director reads it: coming to the glass, through it and in its
/// light is a hunt; coming up the track while you look, a search; standing
/// out there, or waiting at a covered pane or while the place lies quiet,
/// nothing (a hunt waiting behind a blind had held the floor in a peak
/// with no end, and a search standing unwatched every peak open)
pub fn legacy_state(mode: Mode, moving: bool) -> &'static str {
    match mode {
        Mode::Hunt | Mode::Strike => "hunt",
        Mode::Stalk if moving => "search",
        _ => "wander",
    }
}

/// Your view's way (with its pitch)
fn view_dir(g: &Game) -> Vec3 {
    let f = g.player.fwd();
    let (pc, ps) = (g.player.pitch.cos(), g.player.pitch.sin());
    Vec3::new(f.x * pc, ps, f.z * pc)
}

/// Whether your eyes are on its glass: the line of your view through one of
/// its window's lights within `LOOK_REACH`, nothing between, uncovered
pub fn eyes_on_glass(g: &Game) -> bool {
    let Some(b) = g.behind_glass.as_ref() else { return false };
    let Some(w) = g.windows.get(b.win) else { return false };
    let eye = g.camera.pos();
    let f = view_dir(g);
    // into the window's frame (its +z goes into the room)
    let le = w.local(&eye);
    let fl = Vec3::new(f.x * w.right.x + f.z * w.right.z, f.y, f.x * w.normal.x + f.z * w.normal.z);
    for &k in &b.row {
        if w.covered(k) {
            continue;
        }
        let l = &w.lights[k];
        if fl.z >= -1e-6 || le.z <= l.glass_z {
            continue;
        }
        let t = (l.glass_z - le.z) / fl.z;
        if t > LOOK_REACH {
            continue;
        }
        let (x, y) = (le.x + fl.x * t, le.y + fl.y * t);
        if x < l.rect[0] - 0.02 || x > l.rect[1] + 0.02 || y < l.rect[2] - 0.02 || y > l.rect[3] + 0.02 {
            continue;
        }
        // nothing between: a line to just before the wall's face
        let front = w.world(x, y, w.depth + 0.06);
        if g.los(eye.x, eye.z, front.x, front.z) {
            return true;
        }
    }
    false
}

/// Whether you are where its glass is heard: within `HEARD` of it, and in
/// the room it looks into
fn near_glass(g: &Game, b: &Bind) -> bool {
    let w = &g.windows[b.win];
    let at = w.world(b.x, 1.2, w.depth + 0.1);
    let pp = g.player.pos;
    hypot2(at.x - pp.x, at.z - pp.z) < HEARD && g.los(pp.x, pp.z, at.x, at.z)
}

/// A tell at its glass (`creature_tell` measures along the way a creature
/// walks, which out there is no way at all): perceived if you are where its
/// glass is heard, or your eyes are on it, or it is in the room
fn tell(g: &mut Game, ei: usize, b: &Bind) {
    let perceived = near_glass(g, b) || eyes_on_glass(g) || get(g, ei, "inside") > 0.0;
    let now = g.perf_t;
    if let Some(c) = g.creature_mut(ei) {
        if perceived {
            c.tells.retain(|t| now - t < TELL_WINDOW);
            c.tells.push(now);
        }
    }
}

/// Where it stands `out` m beyond the glass, on the ground outside (world)
fn spot(g: &Game, b: &Bind, out: f64) -> Vec3 {
    g.windows[b.win].world(b.x, GROUND, -out)
}

/// One frame of its mind.
pub fn think(g: &mut Game, ei: usize, dt: f64) {
    let Some(b) = bound(g, ei) else {
        // (a showcase's: nothing to be behind; it stands and faces you)
        let pp = g.player.pos;
        let pos = g.entities[ei].pos();
        g.entities[ei].yaw = (pp.x - pos.x).atan2(pp.z - pos.z);
        return;
    };
    let Some(c) = g.creature(ei) else { return };
    let (mode, mode_t, order) = (c.brain.mode, c.brain.mode_t, c.brain.order);
    let now = g.perf_t;
    let looking = eyes_on_glass(g);
    // the look: it adds up, and drains slowly
    let stare = {
        let s = get(g, ei, "stare");
        // (a red envelope picked up: it notices your look sooner, `secrets::keeps`)
        let s = if looking { s + dt * g.body.look } else { (s - STARE_DRAIN * dt).max(0.0) };
        put(g, ei, "stare", s);
        s
    };
    let out = get(g, ei, "out");
    let inside = get(g, ei, "inside") > 0.0;
    let covered = b.lights.iter().all(|&k| g.windows[b.win].covered(k));
    // the place lies quiet after a close call: it does not come through
    let quiet = g.director.phase == crate::entities::director::Phase::Relax || matches!(order, Order::Away(t) if t > 0.0);
    match mode {
        Mode::Idle | Mode::Roam | Mode::Hold => {
            if mode != Mode::Hold {
                set(g, ei, Mode::Hold, "out on the track");
            }
            if stare >= NOTICE {
                set(g, ei, Mode::Alert, "it has seen you");
                if let Some(c) = g.creature_mut(ei) {
                    c.noticed_at = Some(now);
                }
                tell(g, ei, &b);
                call(g, ei, "tick");
            }
        }
        Mode::Alert => {
            if get(g, ei, "coming") > 0.0 {
                // come into its lamp's light it needs your look no longer:
                // stopped by a cover, or waiting out the quiet, it goes on
                // when they end (at the glass, two more seconds there)
                if !covered && !quiet {
                    set(g, ei, Mode::Hunt, "its glass uncovered");
                    if out <= AT_GLASS + 1e-6 {
                        put(g, ei, "pressed_at", now);
                        tell(g, ei, &b);
                        call(g, ei, "creak");
                    }
                }
            } else if stare >= COME {
                set(g, ei, Mode::Stalk, "it comes while you look");
            }
        }
        Mode::Stalk => {
            // while your eyes are on its glass; never back
            if looking {
                let o = (out - WALK * dt).max(AT_GLASS);
                put(g, ei, "out", o);
                put(g, ei, "moving", 1.0);
                if o <= LAMP_LIGHT {
                    set(g, ei, Mode::Hunt, "in its lamp's light");
                    put(g, ei, "coming", 1.0);
                    tell(g, ei, &b);
                }
            } else {
                put(g, ei, "moving", 0.0);
            }
        }
        Mode::Hunt if !inside && get(g, ei, "through") <= 0.0 && covered => {
            // its glass covered: it stands where it was (at the glass, there
            // when the blind goes up)
            put(g, ei, "moving", 0.0);
            set(g, ei, Mode::Alert, "its glass covered");
        }
        Mode::Hunt if !inside && get(g, ei, "through") <= 0.0 => {
            // (come on its own: a cover or the quiet only holds it)
            put(g, ei, "coming", 1.0);
            if out > AT_GLASS + 1e-6 {
                // to the glass on its own
                let o = (out - WALK * dt).max(AT_GLASS);
                put(g, ei, "out", o);
                put(g, ei, "moving", 1.0);
                if o <= AT_GLASS + 1e-6 {
                    put(g, ei, "pressed_at", now);
                    put(g, ei, "moving", 0.0);
                    tell(g, ei, &b);
                    call(g, ei, "creak");
                }
            } else {
                let since = now - get(g, ei, "pressed_at");
                if since > 0.8 && get(g, ei, "misted") <= 0.0 {
                    put(g, ei, "misted", 1.0);
                    tell(g, ei, &b);
                    call(g, ei, "squeak");
                }
                if quiet {
                    // the place lies quiet after a close call: it waits at
                    // the glass (nothing to the director while it does)
                    set(g, ei, Mode::Alert, "it waits at the glass");
                } else if since >= PRESS_T {
                    // through
                    put(g, ei, "through", now);
                    tell(g, ei, &b);
                    call(g, ei, "groan");
                    if near_glass(g, &b) {
                        g.dread_sting(0.8);
                    }
                }
            }
        }
        Mode::Hunt if !inside => {
            // coming through: up over the sill, through the glass, down inside
            let k = ((now - get(g, ei, "through")) / THROUGH_T).clamp(0.0, 1.0);
            put(g, ei, "pass", k);
            if k >= 1.0 {
                put(g, ei, "inside", 1.0);
                put(g, ei, "through", 0.0);
                put(g, ei, "lost_t", 0.0);
                let land = entry(g, &b);
                g.entities[ei].g.set_pos(land.x, land.y, land.z);
            }
        }
        Mode::Hunt => hunt_inside(g, ei, &b, dt),
        Mode::Strike => {
            if mode_t > 1.0 {
                set(g, ei, Mode::Withdraw, "it goes back out");
            }
        }
        Mode::Withdraw if inside || get(g, ei, "pass") > 0.0 => {
            // back out through the glass (after a contact too:
            // `creature_contact` sets this mode)
            let k = (mode_t / BACK_T).clamp(0.0, 1.0);
            put(g, ei, "pass", 1.0 - k);
            put(g, ei, "inside", 0.0);
            put(g, ei, "moving", 0.0);
            if k >= 1.0 {
                put(g, ei, "out", AT_GLASS + 1.2);
                put(g, ei, "stare", NOTICE);
                put(g, ei, "misted", 0.0);
                put(g, ei, "coming", 0.0);
                put(g, ei, "saw_you", 0.0);
                put(g, ei, "kept_out", 0.0);
                set(g, ei, Mode::Alert, "back out, close");
            }
        }
        Mode::Withdraw => set(g, ei, Mode::Alert, "back out, close"),
        _ => set(g, ei, Mode::Hold, "out on the track"),
    }
    // always facing you (at the glass, never more than its face allows)
    let pos = g.entities[ei].pos();
    let pp = g.player.pos;
    let mut yaw = (pp.x - pos.x).atan2(pp.z - pos.z);
    let at_glass = !inside && get(g, ei, "out") <= AT_GLASS + 0.3;
    if at_glass || get(g, ei, "pass") > 0.0 {
        let n = g.windows[b.win].normal;
        let inward = n.x.atan2(n.z);
        let d = crate::entities::wrap_pi(yaw - inward);
        yaw = inward + d.clamp(-0.6, 0.6);
    }
    g.entities[ei].yaw = yaw;
    // where it stands, out there or on its way through (inside it moves itself)
    if get(g, ei, "inside") <= 0.0 {
        let p = place_now(g, ei, &b);
        g.entities[ei].g.set_pos(p.x, p.y, p.z);
    }
    glass_frame(g, ei, &b, dt);
    lamp_learned(g, &b);
}

/// What it learns (design 4): got away from twice out of its light, the
/// lamp outside its window hangs lower (`LAMP_DROP`), so its light lies
/// further into the room. The first time it comes down where you see it
/// (in its window's room, near: its light creeps in across the floor over
/// `LOWERING` s); after that it hangs low from the first.
fn lamp_learned(g: &mut Game, b: &Bind) {
    if !g.creature_learned("glass") {
        return;
    }
    let w = &g.windows[b.win];
    if w.lamp_drop >= LAMP_DROP {
        return;
    }
    let now = g.perf_t;
    let started = get(g, b.ei, "lowering");
    if started <= 0.0 {
        if g.creature_run.shown.iter().any(|k| k == "glass") {
            lamps_drop(g, LAMP_DROP, true);
            return;
        }
        if !near_glass(g, b) {
            return;
        }
        g.creature_run.shown.push("glass".into());
        g.director_note("shown");
        put(g, b.ei, "lowering", now);
        return;
    }
    let k = smoothstep(0.0, LOWERING, now - started);
    // its light creeping in, a few times a second, and where it comes to rest
    let step = ((now - started) / 0.2).floor();
    let again = k >= 1.0 || get(g, b.ei, "lowered_step") != step;
    put(g, b.ei, "lowered_step", step);
    lamps_drop(g, LAMP_DROP * k, again);
}

/// Every lamp outside this floor's windows hung `drop` m lower (every
/// window looks out on the same place: its lamp is all of theirs), their
/// light thrown again if `rethrow`
fn lamps_drop(g: &mut Game, drop: f64, rethrow: bool) {
    for i in 0..g.windows.len() {
        crate::core_props::window::view::set_lamp_drop(g, i, drop);
        if rethrow {
            crate::core_props::window::view::rethrow(g, i);
        }
    }
}

/// Its place out there, or along its way through the window (`pass` 0..1)
fn place_now(g: &Game, ei: usize, b: &Bind) -> Vec3 {
    let pass = get(g, ei, "pass");
    if pass <= 0.0 {
        return spot(g, b, get(g, ei, "out"));
    }
    // up onto the sill, through the glass, down onto the floor inside
    let w = &g.windows[b.win];
    let land = entry(g, b);
    let ll = w.local(&land);
    let (z0, z1) = (-AT_GLASS, ll.z);
    let z = z0 + (z1 - z0) * smoothstep(0.15, 0.85, pass);
    let top = w.sill - 0.45;
    let y = if pass < 0.4 {
        GROUND + (top - GROUND) * smoothstep(0.0, 0.4, pass)
    } else if pass < 0.7 {
        top
    } else {
        top * (1.0 - smoothstep(0.7, 1.0, pass))
    };
    w.world(b.x + (ll.x - b.x) * smoothstep(0.5, 1.0, pass), y, z)
}

fn smoothstep(e0: f64, e1: f64, x: f64) -> f64 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// The middle of a lit cell
fn cell_mid(q: &[(f64, f64); 4]) -> (f64, f64) {
    ((q[0].0 + q[2].0) / 2.0, (q[0].1 + q[2].1) / 2.0)
}

/// Where it comes down inside: the middle of the lit cell nearest its glass
fn entry(g: &Game, b: &Bind) -> Vec3 {
    let w = &g.windows[b.win];
    let mut best: Option<(f64, f64, f64)> = None;
    for &k in &b.row {
        for q in w.lit_cells(k) {
            let (mx, mz) = cell_mid(q);
            let l = w.local(&Vec3::new(mx, 0.0, mz));
            let d = (l.z - w.depth - 0.5).abs() + (l.x - b.x).abs() * 0.5;
            if best.map(|bb| d < bb.0).unwrap_or(true) {
                best = Some((d, mx, mz));
            }
        }
    }
    match best {
        Some((_, x, z)) => Vec3::new(x, w.root.pos().y, z),
        None => w.world(b.x, 0.0, w.depth + 0.6),
    }
}

/// Inside: only where its lamp's light falls; to its edge nearest you, and
/// you, if you are in it and within its reach
fn hunt_inside(g: &mut Game, ei: usize, b: &Bind, dt: f64) {
    // its window's light covered (blinds let down): nowhere is its
    if b.row.iter().all(|&k| g.windows[b.win].covered(k)) {
        set(g, ei, Mode::Withdraw, "its light is gone");
        return;
    }
    // the place lies quiet after a close call: it backs off, as they all do
    let order = g.creature(ei).map(|c| c.brain.order).unwrap_or_default();
    if g.director.phase == crate::entities::director::Phase::Relax || matches!(order, Order::Away(t) if t > 0.0) {
        set(g, ei, Mode::Withdraw, "the place lies quiet");
        return;
    }
    let pos = g.entities[ei].pos();
    let pp = g.player.pos;
    let d = hypot2(pp.x - pos.x, pp.z - pos.z);
    let sees = d < FAR && g.e_los(pos.x, pos.z, pp.x, pp.z);
    if sees {
        put(g, ei, "saw_you", 1.0);
    }
    let lost = if sees { 0.0 } else { get(g, ei, "lost_t") + dt };
    put(g, ei, "lost_t", lost);
    let you_lit = g.windows[b.win].lit_at(pp.x, pp.z);
    // you in its sight and out of its light, long: it gives you up
    let kept = if sees && !you_lit { get(g, ei, "kept_out") + dt } else { 0.0 };
    put(g, ei, "kept_out", kept);
    if lost > LOSES || kept > KEPT_OUT {
        set(g, ei, Mode::Withdraw, if kept > KEPT_OUT { "you kept out of its light" } else { "out of its sight" });
        // you got away from it out of its light (what it learns from; only
        // if it saw you in here at all)
        if get(g, ei, "saw_you") > 0.0 {
            g.creature_habit("glass");
        }
        return;
    }
    if you_lit && d < DATA.reach && g.strike_allowed(ei) {
        set(g, ei, Mode::Strike, "you stood in its light");
        let dead = g.creature_contact(ei);
        if !dead {
            throw_out(g, b);
        }
        return;
    }
    // the point of its light nearest you
    let w = &g.windows[b.win];
    let mut best = (f64::INFINITY, pos.x, pos.z);
    for &k in &b.row {
        for q in w.lit_cells(k) {
            let (mx, mz) = cell_mid(q);
            let dd = hypot2(pp.x - mx, pp.z - mz);
            if dd < best.0 {
                best = (dd, mx, mz);
            }
        }
    }
    let (dx, dz) = (best.1 - pos.x, best.2 - pos.z);
    let dl = hypot2(dx, dz);
    let step = (PACE * dt).min(dl);
    let mut moving = false;
    if dl > 0.02 {
        let (nx, nz) = (pos.x + dx / dl * step, pos.z + dz / dl * step);
        // never out of its light
        if w.lit_at(nx, nz) {
            g.entities[ei].g.set_pos(nx, pos.y, nz);
            moving = true;
        }
    }
    put(g, ei, "moving", if moving { 1.0 } else { 0.0 });
    // it keeps giving you its tells while it is in here: its wet coat
    // rustling
    let now = g.perf_t;
    if now - get(g, ei, "rustle_at") > 2.0 {
        put(g, ei, "rustle_at", now);
        tell(g, ei, b);
        call(g, ei, "rustle");
    }
}

/// After its touch you are thrown down out of its light: along the way from
/// its glass into the room, to the first place out of the light a body fits
fn throw_out(g: &mut Game, b: &Bind) {
    let w = &g.windows[b.win];
    let pp = g.player.pos;
    let (nx, nz) = (w.normal.x, w.normal.z);
    let mut s = 0.25;
    while s <= 4.0 {
        let (x, z) = (pp.x + nx * s, pp.z + nz * s);
        if !w.lit_at(x, z) && !g.e_circle_hits_wall(x, z, 0.3) && g.los(pp.x, pp.z, x, z) {
            g.player.pos.x = x;
            g.player.pos.z = z;
            return;
        }
        s += 0.25;
    }
}

// ------------------------------------------------------------ the glass

/// Its glass this frame: its breath's mist on the pane, and the pane bowing
/// in round it like film as it comes through (and out as it goes back)
fn glass_frame(g: &mut Game, ei: usize, b: &Bind, dt: f64) {
    let inside = get(g, ei, "inside") > 0.0;
    let pass = get(g, ei, "pass");
    let pressing = !inside && pass <= 0.0 && get(g, ei, "out") <= AT_GLASS + 1e-3;
    let t = g.perf_t;
    // the breath: in and out on the pane while its face is to it, fading after
    let mist = {
        let m = get(g, ei, "mist");
        let want = if pressing { 0.45 + 0.35 * (0.5 + 0.5 * (t * 1.7).sin()) } else { 0.0 };
        let m = if want > m { m + (want - m) * (dt * 2.0).min(1.0) } else { (m - 0.12 * dt).max(want) };
        put(g, ei, "mist", m);
        m
    };
    // the bow: a little under its face as it presses; deep round it as it
    // comes through, the film closing behind it
    let going_back = g.creature(ei).map(|c| c.brain.mode == Mode::Withdraw).unwrap_or(false);
    let (depth, radius) = if pressing {
        (0.012 + 0.006 * (t * 1.7).sin(), 0.16)
    } else if pass > 0.0 && pass < 1.0 {
        let k = if pass < 0.55 { smoothstep(0.0, 0.55, pass) } else { 1.0 - smoothstep(0.55, 0.85, pass) };
        let wobble = if pass > 0.55 { 0.03 * ((pass - 0.55) * 40.0).sin() * (1.0 - smoothstep(0.55, 1.0, pass)) } else { 0.0 };
        ((0.02 + 0.30 * k + wobble) * if going_back { -1.0 } else { 1.0 }, 0.18 + 0.30 * k)
    } else {
        (0.0, 0.16)
    };
    let pos = g.entities[ei].pos();
    let w = &g.windows[b.win];
    let face = {
        let l = w.local(&pos);
        (l.x, GROUND + FACE_H - 0.25 * smoothstep(0.0, 0.4, pass))
    };
    for (j, &k) in b.lights.iter().enumerate() {
        let l = &w.lights[k];
        let (cx, cy) = ((l.rect[0] + l.rect[1]) / 2.0, (l.rect[2] + l.rect[3]) / 2.0);
        let (pw, ph) = (l.rect[1] - l.rect[0], l.rect[3] - l.rect[2]);
        // the mist where its face is on this pane
        {
            let m = l.glass.mat();
            let pu = m.r().patch_uniforms.clone();
            let f = &mut pu.borrow_mut().f;
            let (u, v) = ((face.0 - l.rect[0]) / pw, (face.1 - l.rect[2]) / ph);
            let on = (-0.1..=1.1).contains(&u) && (-0.1..=1.1).contains(&v);
            f[4] = u as f32;
            f[5] = v as f32;
            f[6] = 0.2;
            f[7] = if on { mist as f32 } else { 0.0 };
        }
        let Some(rest) = b.panes.get(j) else { continue };
        let Some(geo) = l.glass.try_geo() else { continue };
        let was = get(g, ei, "bowed") > 0.0;
        if depth.abs() < 1e-4 && !was {
            continue;
        }
        let mut p = rest.clone();
        for v in p.chunks_mut(3) {
            let (dx, dy) = (v[0] as f64 + cx - face.0, v[1] as f64 + cy - face.1);
            let r2 = (dx * dx + dy * dy) / (radius * radius);
            // pinned at the frame
            let ex = 1.0 - ((v[0] as f64).abs() / (pw / 2.0)).powi(6);
            let ey = 1.0 - ((v[1] as f64).abs() / (ph / 2.0)).powi(6);
            v[2] += (depth * (-r2).exp() * ex.max(0.0) * ey.max(0.0)) as f32;
        }
        geo.set_position(p);
    }
    put(g, ei, "bowed", if depth.abs() < 1e-4 { 0.0 } else { 1.0 });
}

// ------------------------------------------------------------ placing it

/// The first open tile in the room before window `w`, at `x` across it
pub fn tile_before(g: &Game, w: &crate::core_props::window::Window, x: f64) -> Option<usize> {
    let mut out = w.depth + 0.1;
    while out < w.depth + 4.5 {
        let p = w.world(x, 0.0, out);
        let (tx, tz) = (g.tx(p.x), g.tz(p.z));
        if !g.in_grid(tx, tz) {
            return None;
        }
        if !g.is_wall_tile(tx, tz) {
            return Some(g.t_idx(tx, tz));
        }
        out += 0.25;
    }
    None
}

/// Put the one behind the glass on this level (after her camp is fitted:
/// `session::flow`): from depth 2, on about four in ten of the floors with
/// windows (by the level, not by chance), never where an anomaly is;
/// behind the window that is
/// off the main way and furthest from the arrival (not by the door or her
/// camp, not covered). None when there is no such window.
pub fn place(g: &mut Game) -> Option<usize> {
    place_with(g, false)
}

/// `place`, or (`force`: a showcase's) behind the best window there is,
/// whatever the level's coin and the way say
fn place_with(g: &mut Game, force: bool) -> Option<usize> {
    g.behind_glass = None;
    // what it learned, already seen: every floor's lamps hang low from the
    // first, whether it is behind one of its windows or not (fitted after
    // the build, as the run's progress is)
    if g.creature_learned("glass") && g.creature_run.shown.iter().any(|k| k == "glass") && !g.windows.is_empty() {
        lamps_drop(g, LAMP_DROP, true);
    }
    if g.depth() < 2 || g.building.is_some() || g.level_scp.is_some() || g.in_loop || g.in_stairwell || g.windows.is_empty() {
        return None;
    }
    let mut h = crate::base::rng::SplitMix64::new(g.current_seed, 0x61_A5_5B_E4_u64 ^ g.level_num as u64);
    if h.next() > 0.65 && !force {
        return None;
    }
    let exit = g.exit_data.as_ref().map(|e| (g.tx(e.pos.x), g.tz(e.pos.z)))?;
    let (sx, sz) = (g.tx(g.spawn_pos.x), g.tz(g.spawn_pos.z));
    let from_exit = g.bfs(exit.0, exit.1);
    let from_start = g.bfs(sx, sz);
    let way = crate::entities::corner::main_way(g, &from_exit, sx, sz);
    let camp = g.story.camp.as_ref().map(|c| c.tile);
    let from_camp = camp.map(|(x, z)| g.bfs(x, z));
    let mut best: Option<(f64, Bind)> = None;
    for (wi, w) in g.windows.iter().enumerate() {
        // a window's lights: a sash window's pair (or one), a strip window's one
        let groups: Vec<Vec<usize>> = match w.kind {
            crate::core_props::window::Kind::DoubleHung => (0..w.lights.len()).collect::<Vec<_>>().chunks(2).map(|c| c.to_vec()).collect(),
            crate::core_props::window::Kind::Strip => (0..w.lights.len()).map(|k| vec![k]).collect(),
        };
        for lights in groups {
            if lights.iter().any(|&k| w.covered(k) || w.lights[k].papered) || lights.iter().all(|&k| w.lit_cells(k).is_empty()) {
                continue;
            }
            let x = lights.iter().map(|&k| (w.lights[k].rect[0] + w.lights[k].rect[1]) / 2.0).sum::<f64>() / lights.len() as f64;
            // the first open tile before it (a wall that fills its tile
            // keeps the window's own tile)
            let Some(t) = tile_before(g, w, x) else { continue };
            let (ds, de) = (from_start[t], from_exit[t]);
            if !force && (way[t] || ds < 8 || de < 5 || from_camp.as_ref().is_some_and(|fc| fc[t] >= 0 && fc[t] < 5)) {
                continue;
            }
            let score = ds.min(40) as f64 + (wi * 7 + lights[0] * 3) as f64 * 0.001;
            if best.as_ref().map(|b| score > b.0).unwrap_or(true) {
                best = Some((score, Bind { ei: 0, win: wi, lights, row: (0..w.lights.len()).collect(), x, panes: vec![] }));
            }
        }
    }
    let (_, mut b) = best?;
    // nothing of it draws from the per-frame stream as the level starts
    let own = (h.next() * u32::MAX as f64) as u32;
    let t = tile_before(g, &g.windows[b.win], b.x).unwrap_or(0) as i32;
    let (tx, tz) = (t % g.w, t / g.w);
    let ei = crate::base::rng::with_seed(own, || g.add_entity("glass", tx, tz, 0));
    b.ei = ei;
    // its panes in a fine grid, so they can bow (no draws)
    for &k in &b.lights.clone() {
        let l = &g.windows[b.win].lights[k];
        let geo = plane_geometry(l.rect[1] - l.rect[0], l.rect[3] - l.rect[2], 12.0, 16.0);
        b.panes.push(geo.g().position.clone());
        l.glass.set_geo(geo);
    }
    g.behind_glass = Some(b.clone());
    let p = spot(g, &b, REST);
    g.entities[ei].g.set_pos(p.x, p.y, p.z);
    g.entities[ei].freeze = 0.0;
    // drawn beyond its window's glass
    let portal = g.windows[b.win].portal.clone();
    g.entities[ei].g.traverse(&mut |o: &Obj| {
        for m in o.mats() {
            m.m().portal = Some(portal.clone());
        }
    });
    let now = g.perf_t;
    if let Some(c) = g.creature_mut(ei) {
        c.ease.insert("out", REST);
        c.ease.insert("stare", 0.0);
        c.brain.set(Mode::Hold, "placed out on the track", now);
    }
    Some(ei)
}

/// `--showcase glass:OUT` (headless evidence): you stand before its window
/// looking out, and it stands `out` m beyond the glass (0: its face to the
/// pane; below 0: inside, in its light). A level where no window has one
/// gets one (`place_with`, forced). → it, or None on a level without windows.
pub fn showcase(g: &mut Game, out: f64) -> Option<usize> {
    if g.behind_glass.is_none() {
        place_with(g, true);
    }
    let b = g.behind_glass.clone()?;
    let ei = b.ei;
    let now = g.perf_t;
    {
        let w = &g.windows[b.win];
        // (its face at the glass seen from a metre, at it)
        let pressed = (0.0..=AT_GLASS + 0.01).contains(&out);
        let back = if out < 0.0 {
            3.4
        } else if pressed {
            1.1
        } else {
            1.9
        };
        let eye = w.world(b.x + 0.2, 0.0, w.depth + back);
        g.player.pos.x = eye.x;
        g.player.pos.z = eye.z;
        g.player.yaw = w.normal.x.atan2(w.normal.z);
        g.player.pitch = if out < 0.0 {
            -0.12
        } else if pressed {
            -(1.62 - (GROUND + FACE_H)).atan2(back + w.depth + AT_GLASS)
        } else {
            -0.03
        };
    }
    let mode = if out < 0.0 || out <= AT_GLASS + 0.01 { Mode::Hunt } else { Mode::Hold };
    if let Some(c) = g.creature_mut(ei) {
        if c.brain.mode != mode {
            c.brain.set(mode, "the showcase", now);
        }
        c.ease.insert("out", out.max(AT_GLASS));
        c.ease.insert("inside", if out < 0.0 { 1.0 } else { 0.0 });
        c.ease.insert("pass", 0.0);
        c.ease.insert("pressed_at", now);
        c.ease.insert("moving", 0.0);
    }
    let p = if out < 0.0 { entry(g, &b) } else { place_now(g, ei, &b) };
    g.entities[ei].g.set_pos(p.x, p.y, p.z);
    let pp = g.player.pos;
    g.entities[ei].yaw = (pp.x - p.x).atan2(pp.z - p.z);
    g.entities[ei].freeze = 1.0;
    glass_frame(g, ei, &b, 1.0 / 60.0);
    for (k, e) in g.entities.iter_mut().enumerate() {
        if k != ei {
            e.g.set_pos(-1e4, 0.0, -1e4);
            e.freeze = 1.0;
        }
    }
    Some(ei)
}

// ------------------------------------------------------------ the body

/// A point in the world in the body's frame (its root turned by its yaw)
fn world_to_body(g: &Game, ei: usize, p: Vec3) -> Vec3 {
    let o = g.entities[ei].pos();
    let yaw = g.entities[ei].yaw;
    let (dx, dz) = (p.x - o.x, p.z - o.z);
    let (s, c) = (yaw.sin(), yaw.cos());
    v3(dx * c - dz * s, p.y - o.y, dx * s + dz * c)
}

/// One frame of its body: standing still, its arms at its sides, facing
/// you; walking evenly, too smoothly, its arms still; at the glass its face
/// to the pane and its hands either side of it; through the window bent low
/// over the sill; inside, its arms out toward you.
pub fn animate(g: &mut Game, ei: usize, dt: f64) {
    let b = bound(g, ei);
    // its hands on the glass, if it is at it (world, then the body's frame)
    let hands = b.as_ref().map(|b| {
        let w = &g.windows[b.win];
        let pos = g.entities[ei].pos();
        let l = w.local(&pos);
        let gz = b.lights.first().map(|&k| w.lights[k].glass_z).unwrap_or(0.0) - 0.02;
        // (at its chin, either side: its arms up from the elbow, the upper
        // arms hanging; lifted out from the shoulder, linear skinning had
        // swung the coat's shoulders up into two fins beside its hood)
        let y = GROUND + FACE_H - 0.17;
        [w.world(l.x + 0.22, y, gz), w.world(l.x - 0.22, y, gz)]
    });
    let hands = hands.map(|h| [world_to_body(g, ei, h[0]), world_to_body(g, ei, h[1])]);
    let t = g.perf_t;
    let Some(c) = g.creature_mut(ei) else { return };
    let rig = c.body.clone();
    let moving = c.get("moving") > 0.0;
    let pass = c.get("pass");
    let inside = c.get("inside") > 0.0;
    let pressing = b.is_some() && !inside && pass <= 0.0 && c.get("out") <= AT_GLASS + 1e-3;
    rig.rest();
    // its face to the glass: the morph, its body leaning to it
    let press = c.eased("press", if pressing { 1.0 } else { 0.0 }, 3.0, dt);
    if let Some(m) = rig.meshes.get("skin") {
        let mut inf = m.morph_influences();
        if !inf.is_empty() && (inf[0] - press).abs() > 1e-4 {
            inf[0] = press;
        }
    }
    let crouch = c.eased("crouch", if pass > 0.0 && pass < 1.0 { 1.0 } else { 0.0 }, 5.0, dt);
    let reach = c.eased("reach", if inside { 1.0 } else { 0.0 }, 3.0, dt);
    let breath = (t * 0.8).sin() * 0.008;
    if let Some(ch) = rig.ix("chest") {
        rig.turn(ch, &Quat::from_euler(&Euler::new(0.35 * crouch + 0.12 * press + breath, 0.0, 0.0)));
    }
    if let Some(sp) = rig.ix("spine") {
        rig.turn(sp, &Quat::from_euler(&Euler::new(0.45 * crouch + 0.08 * press, 0.0, 0.0)));
    }
    if let Some(hd) = rig.ix("head") {
        rig.turn(hd, &Quat::from_euler(&Euler::new(-0.25 * crouch - 0.12 * press, 0.0, 0.0)));
    }
    // ---- the arms: still at its sides; up to the glass; out toward you
    for (k, (side, sx)) in [(".L", 1.0), (".R", -1.0)].into_iter().enumerate() {
        let (Some(iu), Some(il), Some(ih)) =
            (rig.ix(&format!("upperarm{side}")), rig.ix(&format!("forearm{side}")), rig.ix(&format!("hand{side}")))
        else {
            continue;
        };
        let up = press.max(reach * 0.9).max(crouch * 0.7);
        if up < 0.01 {
            continue;
        }
        let base = rig.head(iu);
        let rest = v(rig.def.bones[ih].head);
        // on the glass either side of its face; out toward you; on the sill
        let glass = hands.map(|h| h[k]).unwrap_or(v3(sx * 0.19, 1.45, 0.4));
        let toward = v3(sx * 0.16, 1.25, 0.62);
        let sill = v3(sx * 0.24, base.y - 0.25, 0.45);
        let mix = |a: Vec3, b: Vec3, k: f64| v3(a.x + (b.x - a.x) * k, a.y + (b.y - a.y) * k, a.z + (b.z - a.z) * k);
        let want = if crouch > 0.5 {
            sill
        } else if reach > press {
            toward
        } else {
            glass
        };
        let target = mix(rest, want, up);
        // the elbows down and a little out and back (palms flat on the glass,
        // nothing of it through the pane)
        let pole = v3(base.x + sx * 0.15, base.y - 0.6, base.z - 0.25);
        let (du, dl) = two_bone(base, target, pole, rig.rest_len[iu], rig.rest_len[il]);
        rig.aim(iu, du, 0.0);
        rig.aim(il, dl, 0.0);
    }
    // ---- the legs: an even, stiff walk; drawn up over the sill
    if moving {
        c.phase = (c.phase + WALK * dt / 1.1 / 2.0).fract();
    }
    let phase = c.phase;
    for (k, side) in [".L", ".R"].iter().enumerate() {
        let (Some(it), Some(is), Some(ifo)) =
            (rig.ix(&format!("thigh{side}")), rig.ix(&format!("shin{side}")), rig.ix(&format!("foot{side}")))
        else {
            continue;
        };
        let rest_end = v(rig.def.bones[ifo].head);
        let lp = (phase + k as f64 * 0.5).fract() * std::f64::consts::TAU;
        // a step's swing and lift, toward you (it faces you)
        let (fwd, lift) = if moving { (lp.sin() * 0.26, lp.cos().max(0.0) * 0.08) } else { (0.0, 0.0) };
        let target = v3(rest_end.x, rest_end.y + lift + crouch * 0.42, rest_end.z + fwd + crouch * 0.25);
        let base = rig.head(it);
        let pole = v3(base.x, base.y, base.z + 1.0);
        let (du, dl) = two_bone(base, target, pole, rig.rest_len[it], rig.rest_len[is]);
        rig.aim(it, du, 0.0);
        rig.aim(is, dl, 0.0);
        rig.aim(ifo, rig.rest_dir[ifo], 0.0);
    }
}

fn v(a: [f64; 3]) -> Vec3 {
    v3(a[0], a[1], a[2])
}

// ------------------------------------------------------------ the voice

fn voice_slot(g: &mut Game, ei: usize) -> Option<crate::audio::soundscape::VoiceSlot> {
    let s = g.soundscape()?;
    let id = g.entities[ei].g.id();
    if g.ent_ext.get(&id).and_then(|x| x.voice.as_ref()).is_none() {
        let slot = s.voice_slot(3.0);
        g.ent_ext.entry(id).or_default().voice = Some(EntVoice { slot, next: 1.0, was_hunting: false });
    }
    let slot = g.ent_ext[&id].voice.as_ref()?.slot.clone();
    // heard from its glass (or from where it is, inside)
    let p = match bound(g, ei) {
        Some(b) if get(g, ei, "inside") <= 0.0 => {
            let w = &g.windows[b.win];
            w.world(b.x, GROUND + FACE_H, w.lights[b.lights[0]].glass_z)
        }
        _ => g.entities[ei].pos() + v3(0.0, 1.5, 0.0),
    };
    slot.at(p.x, p.y, p.z);
    Some(slot)
}

/// Its calls: `tick`, `creak`, `squeak`, `groan`, and inside `rustle` (only
/// where heard: at its glass, from in its room; inside, within `HEARD` of
/// it with nothing between. Nothing is made for a sound no one could hear)
pub fn call(g: &mut Game, ei: usize, what: &str) {
    if g.ac.is_none() {
        return;
    }
    let heard = match bound(g, ei) {
        Some(_) if get(g, ei, "inside") > 0.0 => {
            let (p, pp) = (g.entities[ei].pos(), g.player.pos);
            hypot2(p.x - pp.x, p.z - pp.z) < HEARD && g.e_los(p.x, p.z, pp.x, pp.z)
        }
        Some(b) => near_glass(g, &b),
        None => true,
    };
    if !heard {
        return;
    }
    let Some(s) = g.soundscape() else { return };
    let Some(v) = voice_slot(g, ei).map(|slot| slot.input.clone()) else { return };
    let t = g.ac_time();
    match what {
        // the pane under strain: a dull knock and the plate's ring, its
        // partials inharmonic (not a click: the Smiler clicks)
        "tick" => {
            s.burst(&v, t, 0.02, "bandpass", 1300.0, 1.5, 0.5, None, 0.001);
            s.tone(&v, t, 0.6, 1170.0, 0.10, "sine", Some(1160.0), 0.001);
            s.tone(&v, t, 0.35, 2710.0, 0.05, "sine", Some(2690.0), 0.001);
        }
        // the pane under its weight: a low dry flex
        "creak" => {
            s.burst(&v, t, 0.9, "bandpass", 420.0, 1.6, 0.6, Some(260.0), 0.08);
            s.note(&v, t + 0.05, 0.8, 190.0, 0.12, "triangle", Some(150.0), 0.1, 0.3);
        }
        // wet skin dragged on the glass: a short squeal, twice
        "squeak" => {
            s.burst(&v, t, 0.22, "bandpass", 3300.0, 9.0, 0.06, Some(3900.0), 0.02);
            s.burst(&v, t + 0.42, 0.16, "bandpass", 3700.0, 9.0, 0.045, Some(3200.0), 0.02);
        }
        // its wet raincoat: rubberised cloth rustling, the stiff cloth
        // creaking as it shifts (the place drips: this is not a drip)
        "rustle" => {
            s.burst(&v, t, 0.28, "bandpass", 1100.0, 0.9, 0.16, Some(800.0), 0.03);
            s.burst(&v, t + 0.31, 0.2, "bandpass", 1300.0, 0.9, 0.12, Some(950.0), 0.03);
            s.note(&v, t + 0.05, 0.35, 260.0, 0.04, "triangle", Some(215.0), 0.05, 0.15);
        }
        // the glass bowing in, long and low, and a wet squeal of skin on it
        "groan" => {
            s.burst(&v, t, 2.2, "bandpass", 300.0, 3.0, 0.42, Some(180.0), 0.4);
            s.note(&v, t + 0.1, 2.0, 140.0, 0.08, "sawtooth", Some(95.0), 0.5, 0.6);
            s.burst(&v, t + 0.6, 0.35, "bandpass", 2600.0, 6.0, 0.18, Some(3100.0), 0.02);
        }
        _ => {}
    }
}

/// Nothing held: its calls are its sound
pub fn voice(_g: &mut Game, _ei: usize, _dt: f64, _d: f64) {}
