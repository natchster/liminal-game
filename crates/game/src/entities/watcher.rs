//! The Watcher (`docs/design/ENTITIES.md`, design 1): someone standing under an old
//! dust sheet, where furniture under sheets stands; at a glance it is
//! furniture. *Keep it in sight.*
//!
//! - **Seen** (`seen`): in your view (a lenient cone, about the screen), lit
//!   (the light map over 0.35 where it stands, your lamp on it, or within
//!   4 m), and not behind a wall. Seen, it holds: not a fold of the sheet
//!   moves.
//! - **Unseen**, and only when the director has sent it, it comes at 4.4 m/s (faster than
//!   you walk) along the way to you, facing you; after a while out of your sight it may
//!   simply be nearer when you look back (up to 6 m along the way, never
//!   within 10 m of you, never where you could see it). Each time you look
//!   back it stands differently: its pose (a morph of its one sheet) changes
//!   only while unseen.
//! - **Its touch** takes your nerve (−45) and knocks you down
//!   (`KindData::touch_nerve`, through `creature_contact` like every strike:
//!   two tells first, and not in the 2 s after you looked away); it falls
//!   back a few steps; it kills only a mind already gone.
//! - **Silent.** Close to it your ears ring (a faint high tone from where it
//!   stands, its tell; `voice`).
//! - **Body.** `bodies.bin`'s `watcher` (`assets/src/creatures/body_watcher.py`):
//!   one sheet draped by Blender's cloth over a hidden body, its poses the
//!   same sheet draped again.
use crate::base::numeric::hypot2;
use crate::entities::brain::*;
use crate::entities::creature::*;
use crate::entities::{EntVoice, wrap_pi};
use crate::prelude::*;

pub const DATA: KindData = KindData {
    sight: 0.0,
    sight_lit: 0.0,
    cone: 0.0,
    hearing: 0.0,
    smell: 0.0,
    radius: 0.4,
    reach: 0.9,
    sight_gain: 0.0,
    sound_gain: 0.0,
    forget: 0.0,
    touch_nerve: 45.0,
};

/// Its pace when no one is looking, m/s: faster than you walk, slower than
/// you run.
pub const PACE: f64 = 4.4;
/// Cosine of half the view in which it counts as seen (lenient: most of the screen).
pub const VIEW: f64 = 0.74;
/// Lit enough to be seen where the light map is over this.
pub const LIT: f64 = 0.35;
/// Within this it is seen in any light, m.
pub const CLOSE: f64 = 4.0;
/// How much nearer it can be when you look back, m along the way.
pub const NEARER: f64 = 6.0;
/// It is never put nearer than this, m.
pub const NEVER_WITHIN: f64 = 10.0;
/// Out of sight this long before it can be nearer, s.
const NEARER_AFTER: f64 = 2.5;
/// Unseen and nearer than this, it is after you (`Mode::Hunt`: the
/// director's peak, which is short; the long way to you is the build-up), m.
const HUNT_NEAR: f64 = 6.0;
/// Your ears ring within this (along the way), m: its tell.
pub const RING_REACH: f64 = 8.0;
/// A stare at it longer than this is what it learns from, s.
pub const LONG_STARE: f64 = 8.0;
/// Once it has learned (three long stares, design 4), while you watch it
/// within this the panels over it fail, every `FAILS_EVERY` s, and its place
/// is dark for `DARK_FOR` s after each (it takes the light you see it by).
pub const FAILS_WITHIN: f64 = 15.0;
pub const FAILS_EVERY: f64 = 2.5;
pub const DARK_FOR: f64 = 5.0;
/// The poses of its sheet, as morphs (the first is the rest).
pub const POSES: [&str; 6] = ["stand", "listen", "bowed", "reach", "lean", "over"];
/// The pose it bends over you in.
const OVER: usize = 5;

/// Its material: an old linen dust sheet, matte, grey with handling, the
/// weave in it; one layer, seen from both sides.
pub fn material(_slot: &str) -> Mat {
    thread_local! {
        static MAT: std::cell::RefCell<Option<Mat>> = const { std::cell::RefCell::new(None) };
    }
    MAT.with(|m| {
        m.borrow_mut()
            .get_or_insert_with(|| {
                let tex = crate::entities::watcher_cloth_tex();
                let mat = crate::entities::with_skin_detail(
                    lambert!({ color: 0xffffff, vertexColors: true, emissive: 0x0b0a08, side: DoubleSide }),
                    3.1,
                    0.85,
                );
                {
                    let mut mm = mat.m();
                    mm.bump_map = Some(tex.clone());
                    mm.patch = Patch::WatcherSheet { map: tex, scale: 3.1, strength: 0.85 };
                    mm.version += 1;
                }
                mat
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

/// Whether you see it: in your view, lit, not behind a wall (the rule it
/// keeps; lenient, as SCP: Containment Breach's).
pub fn seen(g: &Game, ei: usize) -> bool {
    let e = &g.entities[ei];
    let pos = e.pos();
    let pp = g.player.pos;
    let d = hypot2(pos.x - pp.x, pos.z - pp.z);
    // (not "too close not to see": behind your back, at your shoulder, it is
    // unseen, and it touches you)
    if !g.e_los(pp.x, pp.z, pos.x, pos.z) {
        return false;
    }
    let f = g.player.fwd();
    let toward = (f.x * (pos.x - pp.x) + f.z * (pos.z - pp.z)) / d.max(1e-6);
    if toward < VIEW {
        return false;
    }
    let lamp = g.player.flashlight_on && toward > 0.9 && d < 18.0;
    // (learned: the panels over it have failed, its place is dark a while)
    let dark = g.creature(ei).is_some_and(|c| c.get("dark_until") > g.perf_t);
    lamp || d < CLOSE || (!dark && g.e_region_light_at(g.tx(pos.x), g.tz(pos.z)) > LIT)
}

/// A tile up to `NEARER` m along the way to you from it, never within
/// `NEVER_WITHIN` m of you and never where you could see it there.
fn nearer_place(g: &Game, ei: usize) -> Option<(f64, f64)> {
    let pf = g.player_field.clone()?;
    let pos = g.entities[ei].pos();
    let pp = g.player.pos;
    let (mut x, mut z) = (g.tx(pos.x), g.tz(pos.z));
    let mut best = None;
    let mut gone = 0.0;
    loop {
        let here = pf.get(g.t_idx(x, z)).copied().unwrap_or(-1);
        if here <= 0 {
            break;
        }
        let mut next = None;
        for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let (nx, nz) = (x + dx, z + dz);
            if g.in_grid(nx, nz) && pf[g.t_idx(nx, nz)] == here - 1 && !g.is_wall_tile(nx, nz) {
                next = Some((nx, nz));
                break;
            }
        }
        let Some((nx, nz)) = next else { break };
        gone += CELL;
        if gone > NEARER {
            break;
        }
        let (wx, wz) = (g.wx(nx), g.wz(nz));
        if hypot2(wx - pp.x, wz - pp.z) < NEVER_WITHIN {
            break;
        }
        // where you could not see it from where you stand
        let f = g.player.fwd();
        let d = hypot2(wx - pp.x, wz - pp.z).max(1e-6);
        let toward = (f.x * (wx - pp.x) + f.z * (wz - pp.z)) / d;
        if !g.e_los(pp.x, pp.z, wx, wz) || toward < VIEW - 0.15 {
            best = Some((wx, wz));
        }
        x = nx;
        z = nz;
    }
    best
}

/// One frame of the Watcher's mind.
pub fn think(g: &mut Game, ei: usize, dt: f64) {
    let Some(c) = g.creature(ei) else { return };
    let (mode, mode_t, order) = (c.brain.mode, c.brain.mode_t, c.brain.order);
    let pos = g.entities[ei].pos();
    let pp = g.player.pos;
    let d = hypot2(pos.x - pp.x, pos.z - pp.z);
    let now = g.perf_t;
    let is_seen = seen(g, ei);
    // sent by the director, or already after you in the peak it began
    let in_peak = g.director.phase == crate::entities::director::Phase::Peak;
    let sent = matches!(order, Order::Stalk | Order::Seek(..)) || (in_peak && matches!(mode, Mode::Hunt | Mode::Hold | Mode::Stalk));
    // its tell: your ears ring when it is near (and seeing it nearer than
    // you left it)
    {
        let path = g.path_distance(pos.x, pos.z, d);
        let c = g.creature_mut(ei).unwrap();
        let ring = c.get("ring_t") - dt;
        c.ease.insert("ring_t", ring);
        if path < RING_REACH && ring <= 0.0 {
            c.ease.insert("ring_t", 3.0);
            g.creature_tell(ei, RING_REACH);
        }
    }
    {
        let c = g.creature_mut(ei).unwrap();
        let unseen = if is_seen { 0.0 } else { c.get("unseen") + dt };
        let was = c.get("was_seen") > 0.0;
        c.ease.insert("unseen", unseen);
        c.ease.insert("was_seen", if is_seen { 1.0 } else { 0.0 });
        // looked back at after it moved: you see it nearer, a tell
        let moved = c.get("moved_unseen");
        if is_seen && !was && moved > 1.0 {
            c.ease.insert("moved_unseen", 0.0);
            g.creature_tell(ei, 0.0);
        }
    }
    // a long stare, held while it is after you: what it learns from
    {
        let c = g.creature_mut(ei).unwrap();
        let stare = if is_seen { c.get("stare") + dt } else { 0.0 };
        let was_stare = c.get("stare");
        c.ease.insert("stare", stare);
        if !is_seen && was_stare > LONG_STARE && sent {
            g.creature_habit("watcher");
        }
    }
    // learned: while you watch it, the panels over it fail
    if is_seen && sent && d < FAILS_WITHIN && g.creature(ei).unwrap().get("fail_t") <= now && g.creature_may_show(ei) {
        let c = g.creature_mut(ei).unwrap();
        c.ease.insert("fail_t", now + FAILS_EVERY);
        c.ease.insert("dark_until", now + DARK_FOR);
        g.flicker_near(pos.x, pos.z);
        g.director_note("lights fail");
    }
    if is_seen {
        // seen, it holds: nothing moves
        let e = &mut g.entities[ei];
        e.vx = 0.0;
        e.vz = 0.0;
        if sent && !matches!(mode, Mode::Hold | Mode::Withdraw) {
            set(g, ei, Mode::Hold, "you see it");
        } else if !sent && !matches!(mode, Mode::Idle | Mode::Hold | Mode::Withdraw) {
            set(g, ei, Mode::Idle, "it stands where it is");
        }
        return;
    }
    // ---- unseen
    let unseen_t = g.creature(ei).unwrap().get("unseen");
    // it stands differently each time you look back
    if unseen_t > 0.6 && g.creature(ei).unwrap().get("posed") == 0.0 {
        let k = with_native(|| irand(0, (POSES.len() - 2) as i64)) as usize;
        let c = g.creature_mut(ei).unwrap();
        c.ease.insert("pose", k as f64);
        c.ease.insert("posed", 1.0);
    }
    if unseen_t < 0.6 {
        g.creature_mut(ei).unwrap().ease.insert("posed", 0.0);
    }
    match mode {
        Mode::Withdraw => {
            g.entity_drive(ei, dt, 0.0);
            if !matches!(order, Order::Away(t) if t > 0.0) && mode_t > 6.0 {
                set(g, ei, Mode::Idle, "it stands again");
            }
        }
        Mode::Idle | Mode::Hold | Mode::Stalk | Mode::Hunt => {
            if let Order::Away(_) = order {
                set(g, ei, Mode::Withdraw, "the director sent it away");
                return;
            }
            if !sent {
                // not sent, it stays where it stands (furniture)
                g.entity_drive(ei, dt, 0.0);
                if mode != Mode::Idle {
                    set(g, ei, Mode::Idle, "it stands where it is");
                }
                return;
            }
            // out of your sight it comes; near (6 m), it is after you (the
            // director's peak), further, it is on its way
            let want = if d < HUNT_NEAR || (mode == Mode::Hunt && d < HUNT_NEAR + 4.0) { Mode::Hunt } else { Mode::Stalk };
            if mode != want {
                let from_sight = !matches!(mode, Mode::Stalk | Mode::Hunt);
                set(
                    g,
                    ei,
                    want,
                    if from_sight {
                        "out of your sight"
                    } else if want == Mode::Hunt {
                        "near you, unseen"
                    } else {
                        "on its way"
                    },
                );
                // its notice is each time it starts to come: no touch in the
                // 2 s after you look away (`strike_allowed`)
                if from_sight {
                    g.creature_mut(ei).unwrap().noticed_at = Some(now);
                }
            }
            // within reach, and it has given its tells: it touches you
            if d < DATA.reach && g.strike_allowed(ei) {
                set(g, ei, Mode::Strike, "it reached you unseen");
                strike(g, ei);
                return;
            }
            // nearer when you look back (it has been out of sight a while)
            if unseen_t > NEARER_AFTER
                && d > NEVER_WITHIN + 2.0
                && let Some((x, z)) = nearer_place(g, ei)
            {
                let moved = hypot2(x - pos.x, z - pos.z);
                g.entities[ei].g.set_pos(x, 0.0, z);
                let c = g.creature_mut(ei).unwrap();
                c.ease.insert("unseen", 0.0);
                let m = c.get("moved_unseen") + moved;
                c.ease.insert("moved_unseen", m);
                face_you(g, ei);
                return;
            }
            let before = g.entities[ei].pos();
            if d > DATA.reach * 0.8 {
                g.creature_go(ei, None, true, PACE, dt);
            } else {
                g.entity_drive(ei, dt, 0.0);
            }
            let after = g.entities[ei].pos();
            face_you(g, ei);
            let c = g.creature_mut(ei).unwrap();
            let m = c.get("moved_unseen") + hypot2(after.x - before.x, after.z - before.z);
            c.ease.insert("moved_unseen", m);
        }
        Mode::Strike => {
            g.entity_drive(ei, dt, 0.0);
            if mode_t > 1.0 {
                set(g, ei, Mode::Withdraw, "it falls back");
            }
        }
        _ => set(g, ei, Mode::Idle, "it stands where it is"),
    }
}

/// It turns to you (only ever while you cannot see it).
fn face_you(g: &mut Game, ei: usize) {
    let pos = g.entities[ei].pos();
    let pp = g.player.pos;
    g.entities[ei].yaw = (pp.x - pos.x).datan2(pp.z - pos.z);
    let yaw = g.entities[ei].yaw;
    g.entities[ei].g.rotation().y = wrap_pi(yaw);
}

/// Its touch: through `creature_contact` (nerve −45, knocked down; a death
/// only for a mind already gone), then it falls back a few steps.
fn strike(g: &mut Game, ei: usize) {
    {
        let c = g.creature_mut(ei).unwrap();
        c.ease.insert("pose", OVER as f64);
    }
    g.creature_contact(ei);
    if g.state != "playing" {
        return;
    }
    let pos = g.entities[ei].pos();
    let pp = g.player.pos;
    let (dx, dz) = (pos.x - pp.x, pos.z - pp.z);
    let l = hypot2(dx, dz).max(1e-6);
    let (nx, nz) = (pos.x + dx / l * 3.0, pos.z + dz / l * 3.0);
    if !g.e_circle_hits_wall(nx, nz, DATA.radius) {
        g.entities[ei].g.set_pos(nx, 0.0, nz);
    }
}

/// One frame of its body: the pose its sheet stands in (changed only while
/// you cannot see it; bent over you as it touches you), and standing close,
/// facing you, when it has you.
pub fn animate(g: &mut Game, ei: usize, _dt: f64) {
    let killer = g.director.killer == Some(ei) && g.state != "playing";
    let Some(c) = g.creature(ei) else { return };
    // when it has you it stands close, facing you, its face in the cloth at
    // your eyes' height (bent over you is the moment of its touch)
    let pose = if killer { 0 } else { c.get("pose") as usize };
    // its face's mark (what the camera over your body looks at) where its
    // face is: bent over you, or standing
    if let (Some(h), Some(o)) = (c.body.ix("head"), c.body.ix("face.over")) {
        let at = if pose == OVER { c.body.def.bones[o].head } else { c.body.def.bones[h].head };
        c.body.bones[h].set_pos(at[0], at[1], at[2]);
    }
    let Some(sheet) = c.body.meshes.get("sheet") else { return };
    let mut inf = sheet.morph_influences();
    if inf.len() + 1 < POSES.len() {
        return;
    }
    let mut want = vec![0.0; inf.len()];
    if pose > 0 {
        want[pose - 1] = 1.0;
    }
    if *inf != want {
        *inf = want;
    }
}

// ------------------------------------------------------------ its company

/// The furniture it stands among: up to three pieces under the same old
/// sheets on the open tiles round (x, z), facing about, so that at a glance
/// it is one of them. Chosen by the place, not by chance: the level's draws
/// are untouched.
pub fn company(g: &mut Game, x: i32, z: i32) {
    const PIECES: [&str; 3] = ["sheet_wingback", "sheet_chair", "sheet_table"];
    let mut k = (x + z).rem_euclid(3) as usize;
    let mut placed = 0;
    for (dx, dz) in [(1, 0), (-1, 1), (0, -1), (1, 1), (-1, -1), (0, 1), (-1, 0), (1, -1)] {
        if placed >= 3 {
            break;
        }
        let (tx, tz) = (x + dx, z + dz);
        if !g.in_grid(tx, tz) || g.is_wall_tile(tx, tz) {
            continue;
        }
        let Some(rig) = crate::entities::body::build_body(PIECES[k % 3], &material) else { continue };
        let h = |a: i32, b: i32, m: i32| (tx * a + tz * b).rem_euclid(m) as f64;
        let root = rig.root.clone();
        root.position().set(g.wx(tx) + (h(7, 3, 5) - 2.0) * 0.25, 0.0, g.wz(tz) + (h(3, 5, 5) - 2.0) * 0.25);
        root.rotation().y = h(13, 7, 8) * PI / 4.0;
        g.world.add(&root);
        k += 1;
        placed += 1;
    }
}

// ------------------------------------------------------------ the voice

/// Its place in the soundscape (created on first use).
fn voice_slot(g: &mut Game, ei: usize) -> Option<crate::audio::soundscape::VoiceSlot> {
    let s = g.soundscape()?;
    let id = g.entities[ei].g.id();
    if g.ent_ext.get(&id).and_then(|x| x.voice.as_ref()).is_none() {
        let slot = s.voice_slot(1.5);
        g.ent_ext.entry(id).or_default().voice = Some(EntVoice { slot, next: 1.0, was_hunting: false });
    }
    let slot = g.ent_ext[&id].voice.as_ref()?.slot.clone();
    let pos = g.entities[ei].pos();
    slot.at(pos.x, 1.5, pos.z);
    Some(slot)
}

/// It has no calls.
pub fn call(_g: &mut Game, _ei: usize, _what: &str) {}

/// Silent; but close to it your ears ring: a faint, high, beating tone from
/// where it stands, louder as you near it (none past `RING_REACH`).
pub fn voice(g: &mut Game, ei: usize, dt: f64, d: f64) {
    if g.ac.is_none() || d > RING_REACH {
        return;
    }
    let Some(s) = g.soundscape() else { return };
    let Some(slot) = voice_slot(g, ei) else { return };
    slot.occlude(0.0);
    let v = slot.input.clone();
    let t = g.ac_time();
    let Some(c) = g.creature_mut(ei) else { return };
    c.voice_t -= dt;
    if c.voice_t > 0.0 {
        return;
    }
    c.voice_t = 2.4;
    let k = (1.0 - d / RING_REACH).clamp(0.0, 1.0);
    // two close high tones beating, swelling and fading over 2.6 s
    s.tone(&v, t, 2.6, 7120.0, 0.010 * k, "sine", None, 0.9);
    s.tone(&v, t + 0.05, 2.5, 7133.0, 0.008 * k, "sine", None, 0.9);
}
