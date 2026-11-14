//! The one in the corner (`docs/design/ENTITIES.md`, design 1): someone tall
//! standing in the far corner of a quiet room, forehead to the angle of the
//! walls, in a long nightshirt, barefoot. *Do not look at its face.*
//!
//! - **Placed**, never sent (`place`): in a corner of a room off the main way,
//!   far from the arrival, her camp and the door, from depth 2; at most one a
//!   level, chosen by the place (no draws).
//! - **Its back** is harmless. Its face is a person's face with the eyes and
//!   the mouth upside down, and you see it only once, as it turns.
//! - **The rule** (`sees_its_face`): you are in front of its face, its face in
//!   your view, a line between you, and its face lit (the light where it
//!   stands, your lamp on it, or within 3 m). Then it turns its head to you
//!   over 1.5 s and the hum stops (its tell: look away, and it turns back).
//!   And now and then while you are in its room (within 20 m and its sight,
//!   every 25–45 s) the hum
//!   stops and it turns its head to look at the room behind it: if its face
//!   comes round into your view, lit, the same.
//!   Hold the look and it turns fully, screams (a tell), and comes at
//!   5.6 m/s (below your sprint) knowing where you are for 6 s; after that
//!   only while it sees you; out of its sight 6 s, it goes back to its corner.
//! - **Its sound**: it hums, low and tuneless, a room away; the hum stops
//!   when it is about to turn; the scream.
//! - **Body**: `bodies.bin`'s `corner` (`assets/src/creatures/body_corner.py`).
//!
//! Its rule (seeing its face is what wakes it) is after SCP-096 "The Shy
//! Guy" by Dr Dan, CC BY-SA 3.0: this file is CC BY-SA 3.0 and keeps this
//! attribution (`docs/design/LORE.md`, Attribution). The turned back is a motif of
//! the Rake; its face is the Thatcher illusion (Peter Thompson, 1980).
use crate::base::numeric::hypot2;
use crate::entities::body::{BodyRig, two_bone};
use crate::entities::brain::*;
use crate::entities::creature::*;
use crate::entities::{EntVoice, with_skin_detail, wrap_pi};
use crate::prelude::*;

pub const DATA: KindData = KindData {
    sight: 0.0,
    sight_lit: 0.0,
    cone: 0.0,
    hearing: 0.0,
    smell: 0.0,
    radius: 0.35,
    reach: 1.2,
    sight_gain: 0.0,
    sound_gain: 0.0,
    forget: 0.0,
    touch_nerve: 0.0,
};

/// How long its head takes to come round to you, s (your chance to look away).
pub const TURN: f64 = 1.5;
/// Its pace when it comes, m/s (below your sprint).
pub const PACE: f64 = 5.6;
/// It knows where you are this long after it screams, s.
pub const KNOWS: f64 = 6.0;
/// Out of its sight this long, it goes back to its corner, s.
pub const LOSES: f64 = 6.0;
/// Your view of its face counts within this cosine (lenient: the screen).
pub const VIEW: f64 = 0.74;
/// Its face is lit where the light map is over this.
pub const LIT: f64 = 0.35;
/// Within this its face is lit in any light, m.
pub const NEAR: f64 = 3.0;
/// While you are this near and in its sight it turns to look behind it now
/// and then, every this many seconds (the hum stops first), m and s.
pub const LOOKS_NEAR: f64 = 20.0;
pub const LOOKS_EVERY: (f64, f64) = (25.0, 45.0);
/// ... and once it has learned (you passed behind it twice, design 4),
/// this often.
pub const LOOKS_LEARNED: (f64, f64) = (10.0, 18.0);
/// Within this behind its back, you pass behind it, m.
const BEHIND: f64 = 3.0;
/// Its hum carries this far (a room away), m.
const HUM_REACH: f64 = 16.0;
const SCREAM_REACH: f64 = 30.0;

thread_local! {
    static MATS: std::cell::RefCell<std::collections::HashMap<String, Mat>> = Default::default();
}

/// Its materials: pale, waxy skin; washed flannel; the eyes and teeth.
pub fn material(slot: &str) -> Mat {
    MATS.with(|m| {
        m.borrow_mut()
            .entry(slot.to_string())
            .or_insert_with(|| match slot {
                "cloth" => {
                    with_skin_detail(lambert!({ color: 0xffffff, vertexColors: true, emissive: 0x0a0a09, side: DoubleSide }), 3.4, 0.6)
                }
                "eye" => phong!({ color: 0xffffff, vertexColors: true, specular: 0x555555, shininess: 90 }),
                "teeth" => lambert!({ color: 0xffffff, vertexColors: true }),
                _ => with_skin_detail(phong!({ color: 0xffffff, vertexColors: true, specular: 0x2a2826, shininess: 26 }), 3.2, 1.0),
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

/// Where its face looks (its body's way, its head turned by `head_yaw`).
fn face_dir(g: &Game, ei: usize) -> (f64, f64) {
    let yaw = g.entities[ei].yaw + g.creature(ei).map_or(0.0, |c| c.get("head_yaw"));
    (yaw.dsin(), yaw.dcos())
}

/// Whether you see its face: in its front half, its face in your view, a
/// line between you, and its face lit.
pub fn sees_its_face(g: &Game, ei: usize) -> bool {
    let pos = g.entities[ei].pos();
    let pp = g.player.pos;
    let d = hypot2(pos.x - pp.x, pos.z - pp.z).max(1e-6);
    let (fx, fz) = face_dir(g, ei);
    // you are in front of its face
    if fx * (pp.x - pos.x) + fz * (pp.z - pos.z) <= 0.0 {
        return false;
    }
    // you look at it
    let f = g.player.fwd();
    let toward = (f.x * (pos.x - pp.x) + f.z * (pos.z - pp.z)) / d;
    if toward < VIEW || !g.e_los(pp.x, pp.z, pos.x, pos.z) {
        return false;
    }
    let lamp = g.player.flashlight_on && toward > 0.9 && d < 18.0;
    lamp || d < NEAR || g.e_region_light_at(g.tx(pos.x), g.tz(pos.z)) > LIT
}

/// Whether it sees you (after its scream): a line between you, within 30 m.
fn sees_you(g: &Game, ei: usize) -> bool {
    let pos = g.entities[ei].pos();
    let pp = g.player.pos;
    hypot2(pos.x - pp.x, pos.z - pp.z) < 30.0 && g.e_los(pos.x, pos.z, pp.x, pp.z)
}

/// One frame of its mind.
pub fn think(g: &mut Game, ei: usize, dt: f64) {
    let Some(c) = g.creature(ei) else { return };
    let (mode, mode_t) = (c.brain.mode, c.brain.mode_t);
    let pos = g.entities[ei].pos();
    let pp = g.player.pos;
    let d = hypot2(pos.x - pp.x, pos.z - pp.z);
    let now = g.perf_t;
    let home = (c.get("home_x"), c.get("home_z"), c.get("home_yaw"));
    match mode {
        Mode::Idle | Mode::Roam | Mode::Hold => {
            if mode != Mode::Hold {
                set(g, ei, Mode::Hold, "in its corner");
            }
            g.entity_drive(ei, dt, 0.0);
            // you pass behind it (counted once a pass): what it learns from
            let (fx, fz) = face_dir(g, ei);
            let behind = d < BEHIND && fx * (pp.x - pos.x) + fz * (pp.z - pos.z) < 0.0;
            let passing = g.creature(ei).unwrap().get("passing") > 0.0;
            if behind && !passing {
                g.creature_mut(ei).unwrap().ease.insert("passing", 1.0);
                g.creature_habit("corner");
            } else if passing && d > 2.0 * BEHIND {
                g.creature_mut(ei).unwrap().ease.insert("passing", 0.0);
            }
            // now and then, while you are in its room, it turns to look at
            // the room behind it (the hum stops first); learned, more often.
            // Never while the place relaxes after a close call (pillar 5: the
            // threat backs off; its chase had begun a peak in the relief)
            let relaxing = g.director.phase == crate::entities::director::Phase::Relax;
            let near = d < LOOKS_NEAR && g.e_los(pos.x, pos.z, pp.x, pp.z) && !relaxing;
            let every = if g.creature_learned("corner") { LOOKS_LEARNED } else { LOOKS_EVERY };
            let turn_in = {
                let c = g.creature_mut(ei).unwrap();
                if c.get("turn_in") <= 0.0 {
                    let v = with_native(|| rand2(every.0, every.1));
                    c.ease.insert("turn_in", v);
                }
                let v = c.get("turn_in") - if near { dt } else { 0.0 };
                c.ease.insert("turn_in", v.max(1e-3));
                v
            };
            let why = if sees_its_face(g, ei) {
                Some("you saw its face")
            } else if near && turn_in <= 1e-3 {
                Some("it turns to look behind it")
            } else {
                None
            };
            if let Some(why) = why {
                set(g, ei, Mode::Alert, why);
                // the hum stops: its tell
                g.creature_tell(ei, HUM_REACH);
                let c = g.creature_mut(ei).unwrap();
                c.ease.insert("humming", 0.0);
                c.ease.insert("turn_in", 0.0);
            }
        }
        Mode::Alert => {
            g.entity_drive(ei, dt, 0.0);
            let looked_round = g.creature(ei).unwrap().brain.log.last().is_some_and(|l| l.3 == "it turns to look behind it");
            if !looked_round && !sees_its_face(g, ei) {
                // you looked away in time: it turns back to its corner
                set(g, ei, Mode::Hold, "you looked away");
                g.creature_mut(ei).unwrap().ease.insert("humming", 1.0);
            } else if looked_round && mode_t >= TURN && !sees_its_face(g, ei) {
                // its head came round, and you were not looking
                set(g, ei, Mode::Hold, "you were not looking");
                g.creature_mut(ei).unwrap().ease.insert("humming", 1.0);
            } else if mode_t >= TURN {
                // it has your face: it turns, and screams
                set(g, ei, Mode::Hunt, "you held its look");
                let c = g.creature_mut(ei).unwrap();
                c.noticed_at = Some(now);
                c.ease.insert("knows_until", now + KNOWS);
                c.ease.insert("lost_t", 0.0);
                g.creature_tell(ei, SCREAM_REACH);
                g.director_note("notice");
                call(g, ei, "scream");
                g.dread_sting(1.0);
            }
        }
        Mode::Hunt => {
            let knows = now < g.creature(ei).unwrap().get("knows_until");
            let sees = sees_you(g, ei);
            let lost = if knows || sees { 0.0 } else { g.creature(ei).unwrap().get("lost_t") + dt };
            g.creature_mut(ei).unwrap().ease.insert("lost_t", lost);
            if lost > LOSES {
                set(g, ei, Mode::Withdraw, "it lost you");
                return;
            }
            if d < DATA.reach && g.strike_allowed(ei) {
                set(g, ei, Mode::Strike, "it reached you");
                g.creature_contact(ei);
                return;
            }
            if knows || sees {
                g.creature_go(ei, None, true, PACE, dt);
                let h = (pp.x - pos.x).datan2(pp.z - pos.z);
                g.entity_turn(ei, h, dt, Some(8.0));
            } else {
                g.entity_drive(ei, dt, 0.0);
            }
        }
        Mode::Strike => {
            g.entity_drive(ei, dt, 0.0);
            if mode_t > 1.0 {
                set(g, ei, Mode::Withdraw, "it goes back");
            }
        }
        Mode::Withdraw => {
            // back to its corner, and its face to the walls
            let at = hypot2(pos.x - home.0, pos.z - home.1) < 0.3;
            if at {
                g.entity_drive(ei, dt, 0.0);
                g.entity_turn(ei, home.2, dt, Some(2.5));
                if wrap_pi(g.entities[ei].yaw - home.2).abs() < 0.05 {
                    g.entities[ei].yaw = home.2;
                    g.creature_mut(ei).unwrap().ease.insert("humming", 1.0);
                    set(g, ei, Mode::Hold, "back in its corner");
                }
            } else {
                g.creature_go(ei, Some((home.0, home.1)), false, 2.0, dt);
            }
        }
        _ => set(g, ei, Mode::Hold, "in its corner"),
    }
}

// ------------------------------------------------------------ placing it

/// The main way: the walk from the arrival (sx, sz) down the door's field
/// `from_exit`, and two tiles round it (what a placed creature keeps off).
pub fn main_way(g: &Game, from_exit: &[i32], sx: i32, sz: i32) -> Vec<bool> {
    let mut way = vec![false; (g.w * g.h) as usize];
    let (mut x, mut z) = (sx, sz);
    for _ in 0..(g.w * g.h) {
        for dz in -2..=2 {
            for dx in -2..=2 {
                if g.in_grid(x + dx, z + dz) {
                    way[g.t_idx(x + dx, z + dz)] = true;
                }
            }
        }
        let here = from_exit[g.t_idx(x, z)];
        if here <= 0 {
            break;
        }
        let next = [(1, 0), (-1, 0), (0, 1), (0, -1)]
            .into_iter()
            .map(|(dx, dz)| (x + dx, z + dz))
            .find(|&(nx, nz)| g.in_grid(nx, nz) && from_exit[g.t_idx(nx, nz)] == here - 1);
        let Some((nx, nz)) = next else { break };
        x = nx;
        z = nz;
    }
    way
}

/// Where the one in the corner will stand on this level (its tile), if it
/// is placed: pure, from the level as built (`place`'s choice). What is
/// put away (`secrets`) keeps clear of it.
pub fn corner_spot(g: &Game) -> Option<(i32, i32)> {
    chosen(g).map(|((x, z, _, _), _)| (x, z))
}

/// Put the one in the corner on this level (after her camp is fitted:
/// `session::flow`): from depth 2, on about half the levels (by the level,
/// not by chance), in the quietest inside corner off the main way, far from
/// the arrival, her camp and the door. None when there is no such corner.
pub fn place(g: &mut Game) -> Option<usize> {
    let ((x, z, cx, cz), mut h) = chosen(g)?;
    // nothing of it draws from the per-frame stream as the level starts (a
    // draw there would shift every frame after it): its making draws on a
    // stream of its own, and its first look round is set by the place
    let own = (h.next() * u32::MAX as f64) as u32;
    let ei = crate::base::rng::with_seed(own, || g.add_entity("corner", x, z, 0));
    stand_in(g, ei, x, z, cx, cz);
    let first = LOOKS_EVERY.0 + (LOOKS_EVERY.1 - LOOKS_EVERY.0) * h.next();
    if let Some(c) = g.creature_mut(ei) {
        c.ease.insert("turn_in", first);
    }
    Some(ei)
}

/// The corner `place` chooses (its tile and the way into the corner), and
/// the level's own stream after the choice
fn chosen(g: &Game) -> Option<((i32, i32, i32, i32), crate::base::rng::SplitMix64)> {
    if g.depth() < 2 || g.building.is_some() || g.level_scp.is_some() || g.in_loop || g.in_stairwell {
        return None;
    }
    // about half the levels: from the level's own seed, not the stream
    let mut h = crate::base::rng::SplitMix64::new(g.current_seed, 0xC0_4E_E1_u64 ^ g.level_num as u64);
    if h.next() > 0.55 {
        return None;
    }
    let exit = g.exit_data.as_ref().map(|e| (g.tx(e.pos.x), g.tz(e.pos.z)))?;
    let (sx, sz) = (g.tx(g.spawn_pos.x), g.tz(g.spawn_pos.z));
    let from_exit = g.bfs(exit.0, exit.1);
    let from_start = g.bfs(sx, sz);
    let camp = g.story.camp.as_ref().map(|c| c.tile);
    let from_camp = camp.map(|(x, z)| g.bfs(x, z));
    let way = main_way(g, &from_exit, sx, sz);
    let mut best: Option<(f64, i32, i32, i32, i32)> = None;
    for z in 1..g.h - 1 {
        for x in 1..g.w - 1 {
            let i = g.t_idx(x, z);
            if g.is_wall_tile(x, z) || way[i] || from_start[i] < 8 || from_exit[i] < 5 || from_exit[i] < 0 {
                continue;
            }
            if let Some(fc) = &from_camp
                && fc[i] >= 0
                && fc[i] < 5
            {
                continue;
            }
            for (cx, cz) in [(1, 1), (1, -1), (-1, 1), (-1, -1)] {
                if !(g.is_wall_tile(x + cx, z) && g.is_wall_tile(x, z + cz) && g.is_wall_tile(x + cx, z + cz)) {
                    continue;
                }
                // quieter the further from the way (its open neighbours few)
                let open = [(1, 0), (-1, 0), (0, 1), (0, -1)].iter().filter(|(dx, dz)| !g.is_wall_tile(x + dx, z + dz)).count();
                let score = (from_start[i] as f64).min(40.0) * 0.1 - open as f64 + ((x * 7 + z * 13) % 5) as f64 * 0.01;
                if best.is_none_or(|b| score > b.0) {
                    best = Some((score, x, z, cx, cz));
                }
            }
        }
    }
    let (_, x, z, cx, cz) = best?;
    Some(((x, z, cx, cz), h))
}

/// Stand creature `ei` in the corner of tile (x, z) toward (cx, cz) (each
/// ±1), close to the angle of the walls, its face to it: its home.
pub fn stand_in(g: &mut Game, ei: usize, x: i32, z: i32, cx: i32, cz: i32) {
    let back = CELL / 2.0 - 0.42;
    let (wx, wz) = (g.wx(x) + cx as f64 * back, g.wz(z) + cz as f64 * back);
    let yaw = (cx as f64).datan2(cz as f64);
    let e = &mut g.entities[ei];
    e.g.set_pos(wx, 0.0, wz);
    e.yaw = yaw;
    e.freeze = 0.0;
    let now = g.perf_t;
    if let Some(c) = g.creature_mut(ei) {
        c.ease.insert("home_x", wx);
        c.ease.insert("home_z", wz);
        c.ease.insert("home_yaw", yaw);
        c.ease.insert("humming", 1.0);
        c.brain.set(Mode::Hold, "placed in its corner", now);
    }
}

// ------------------------------------------------------------ the body

fn world_to_body(root: &Obj, p: Vec3) -> Vec3 {
    let o = root.pos();
    let yaw = root.rotation().y;
    let (dx, dz) = (p.x - o.x, p.z - o.z);
    let (s, c) = (yaw.dsin(), yaw.dcos());
    v3(dx * c - dz * s, p.y - o.y, dx * s + dz * c)
}

fn body_to_world(root: &Obj, p: Vec3) -> Vec3 {
    let o = root.pos();
    let yaw = root.rotation().y;
    let (s, c) = (yaw.dsin(), yaw.dcos());
    v3(o.x + p.x * c + p.z * s, o.y + p.y, o.z - p.x * s + p.z * c)
}

fn rest_head(rig: &BodyRig, i: usize) -> Vec3 {
    let h = rig.def.bones[i].head;
    v3(h[0], h[1], h[2])
}

/// One frame of its body: utterly still in its corner (a breath); its head
/// coming round to you, alone, while it turns; then all of it, running at
/// you with its arms dead at its sides, its jaw open.
pub fn animate(g: &mut Game, ei: usize, dt: f64) {
    let (vx, vz) = (g.entities[ei].vx, g.entities[ei].vz);
    let speed = hypot2(vx, vz);
    let t = g.perf_t;
    let pp = g.player.pos;
    let killer = g.director.killer == Some(ei) && g.state != "playing";
    let Some(c) = g.creature_mut(ei) else { return };
    let rig = c.body.clone();
    let (mode, mode_t) = (c.brain.mode, c.brain.mode_t);
    let root = rig.root.clone();
    rig.rest();
    if killer {
        // it has you: all of it turned to you
        let p = root.pos();
        let y = (pp.x - p.x).datan2(pp.z - p.z);
        root.rotation().y = y;
    }
    let yaw = root.rotation().y;
    let pos = root.pos();
    let want = wrap_pi((pp.x - pos.x).datan2(pp.z - pos.z) - yaw);
    // the head: round to you over `TURN` (as far as a neck goes, and further)
    let turning = matches!(mode, Mode::Alert);
    let head_yaw = if turning {
        want.clamp(-2.6, 2.6) * (mode_t / TURN).clamp(0.0, 1.0)
    } else if matches!(mode, Mode::Hunt | Mode::Strike) || killer {
        want.clamp(-1.0, 1.0)
    } else {
        // back to its corner, slowly
        let h = c.get("head_yaw");
        h - h.signum() * (dt * 1.2).min(h.abs())
    };
    c.ease.insert("head_yaw", head_yaw);
    let scream = c.eased("scream", if matches!(mode, Mode::Hunt | Mode::Strike) || killer { 1.0 } else { 0.0 }, 6.0, dt);
    if let Some(nk) = rig.ix("neck") {
        rig.turn(nk, &Quat::from_euler(&Euler::new(-0.15 * scream, head_yaw * 0.45, 0.0)));
    }
    if let Some(hd) = rig.ix("head") {
        // at rest its forehead to the corner; turning, its face comes round
        rig.turn(hd, &Quat::from_euler(&Euler::new(0.12 * (1.0 - scream) - 0.1 * scream, head_yaw * 0.55, 0.0)));
    }
    if let Some(jw) = rig.ix("jaw") {
        rig.turn(jw, &Quat::from_euler(&Euler::new(0.55 * scream, 0.0, 0.0)));
    }
    if let Some(ch) = rig.ix("chest") {
        // a slow breath, nothing else
        rig.turn(ch, &Quat::from_euler(&Euler::new((t * 0.9).dsin() * 0.012, 0.0, 0.0)));
    }
    // ---- the legs: planted; it runs in long strides, its arms do not move
    if c.limbs.len() != 2 {
        c.limbs = vec![Limb::default(); 2];
    }
    let stride = 1.6;
    c.phase = (c.phase + speed * dt / stride).fract();
    let phase = c.phase;
    for (k, side) in [".L", ".R"].iter().enumerate() {
        let (Some(it), Some(is), Some(ifo)) =
            (rig.ix(&format!("thigh{side}")), rig.ix(&format!("shin{side}")), rig.ix(&format!("foot{side}")))
        else {
            continue;
        };
        let rest_end = rest_head(&rig, ifo);
        let home_w = body_to_world(&root, v3(rest_end.x, 0.0, rest_end.z));
        let lp = (phase + k as f64 * 0.5).fract();
        let limb = &mut c.limbs[k];
        if !limb.planted || hypot2(limb.at[0] - home_w.x, limb.at[2] - home_w.z) > 1.8 {
            limb.at = [home_w.x, rest_end.y, home_w.z];
            limb.planted = true;
        }
        if speed > 0.2 && lp > 0.5 {
            let s = (lp - 0.5) / 0.5;
            if limb.swing == 0.0 {
                limb.from = limb.at;
                let l = speed.max(1e-6);
                limb.to = [home_w.x + vx / l * stride * 0.5, rest_end.y, home_w.z + vz / l * stride * 0.5];
            }
            limb.swing = s.max(1e-3);
            for a in 0..3 {
                limb.at[a] = limb.from[a] + (limb.to[a] - limb.from[a]) * s;
            }
            limb.at[1] = rest_end.y + (s * PI).dsin() * 0.18;
        } else {
            if limb.swing > 0.0 {
                limb.at = limb.to;
            }
            limb.swing = 0.0;
            limb.at[1] = rest_end.y;
        }
        let target = world_to_body(&root, v3(limb.at[0], limb.at[1], limb.at[2]));
        let base = rig.head(it);
        let pole = v3(base.x, base.y, base.z + 1.0);
        let (du, dl) = two_bone(base, target, pole, rig.rest_len[it], rig.rest_len[is]);
        rig.aim(it, du, 0.0);
        rig.aim(is, dl, 0.0);
        rig.aim(ifo, rig.rest_dir[ifo], 0.0);
    }
}

// ------------------------------------------------------------ the voice

fn voice_slot(g: &mut Game, ei: usize) -> Option<crate::audio::soundscape::VoiceSlot> {
    let s = g.soundscape()?;
    let id = g.entities[ei].g.id();
    if g.ent_ext.get(&id).and_then(|x| x.voice.as_ref()).is_none() {
        let slot = s.voice_slot(4.0);
        g.ent_ext.entry(id).or_default().voice = Some(EntVoice { slot, next: 1.0, was_hunting: false });
    }
    let slot = g.ent_ext[&id].voice.as_ref()?.slot.clone();
    let pos = g.entities[ei].pos();
    slot.at(pos.x, 1.75, pos.z);
    Some(slot)
}

/// Its calls: `scream`.
pub fn call(g: &mut Game, ei: usize, what: &str) {
    let Some(s) = g.soundscape() else { return };
    let Some(v) = voice_slot(g, ei).map(|slot| slot.input.clone()) else { return };
    let t = g.ac_time();
    if what == "scream" {
        // a long hoarse shriek, rising and breaking
        s.burst(&v, t, 1.6, "bandpass", 1900.0, 1.4, 0.34, Some(2900.0), 0.04);
        s.tone(&v, t + 0.02, 1.5, 610.0, 0.06, "sawtooth", Some(980.0), 0.05);
        s.burst(&v, t + 1.4, 0.9, "bandpass", 1200.0, 1.1, 0.2, Some(700.0), 0.1);
    }
}

/// How loud its hum is (a room away, over the room's bed in its band:
/// `the_one_in_the_corner_hums`)
const HUM_GAIN: f64 = 0.15;

/// It hums while it stands in its corner (low, tuneless, a room away, not
/// through walls clearly); silent when it turns; breathing hard when it comes.
pub fn voice(g: &mut Game, ei: usize, dt: f64, d: f64) {
    if g.ac.is_none() || d > HUM_REACH + 4.0 {
        return;
    }
    let Some(s) = g.soundscape() else { return };
    let Some(slot) = voice_slot(g, ei) else { return };
    let pos = g.entities[ei].pos();
    let pp = g.player.pos;
    let clear = g.e_los(pos.x, pos.z, pp.x, pp.z);
    slot.occlude(if clear { 0.0 } else { 0.6 });
    let v = slot.input.clone();
    let t = g.ac_time();
    let Some(c) = g.creature_mut(ei) else { return };
    let (mode, humming) = (c.brain.mode, c.get("humming") > 0.0);
    c.voice_t -= dt;
    if c.voice_t > 0.0 {
        return;
    }
    if mode == Mode::Hold && humming {
        // no tune: one held note at a time, wandering round where the last
        // one was, now and then a breath between (one at a time, so the hum
        // stops within a note when it is about to turn)
        let last = c.get("hum_f");
        // (a man's hum, held: over the lamps' 120 Hz and under its octave.
        // Each note had been a pluck (`tone` falls away over its length), at
        // 98-140 Hz under the lamps: heard only as each began, it could not
        // be heard to stop; E10)
        let f =
            if last > 0.0 { (last * with_native(|| rand2(0.93, 1.08))).clamp(160.0, 210.0) } else { with_native(|| rand2(170.0, 200.0)) };
        c.ease.insert("hum_f", f);
        let len = with_native(|| rand2(0.5, 0.9));
        s.note(&v, t, len, f, HUM_GAIN, "triangle", Some(f * with_native(|| rand2(0.98, 1.02))), 0.12, 0.15);
        let breath = if with_native(rnd) < 0.2 { with_native(|| rand2(0.8, 1.8)) } else { 0.0 };
        c.voice_t = len * 0.95 + breath;
    } else if matches!(mode, Mode::Hunt) {
        s.burst(&v, t, 0.35, "lowpass", 700.0, 0.8, 0.05, Some(400.0), 0.05);
        c.voice_t = 0.45;
    } else {
        c.voice_t = 0.5;
    }
}
