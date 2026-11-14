//! The Reader (`docs/design/ENTITIES.md`, design 1): a thin man in a brown cardigan
//! buttoned wrong, paper stuffed in its pockets and down its front, ink on
//! its fingers, the fingers too long; it walks bent over a sheet held close
//! to its face, reading under its breath. *Give it paper.*
//!
//! - **Senses.** Sight only, over the page: 12 m, in its view, and only
//!   where you are lit or moving (still in the dark you are nothing to it,
//!   even close; it feels you if it walks into you). It has no ears.
//! - **The compulsion** (`paper_near`). Any sheet of paper on the floor
//!   within 4 m of it, where it can see it, stops it whatever it is doing:
//!   it goes to it, stoops, picks it up and reads it to the end (3 s), blind
//!   to you. The sheets you drop it takes; a pile lying about it reads the
//!   top sheet of, once. It never touches her folded sheets nor the flat
//!   ones: they are not paper it reads, and nothing here looks at them.
//! - **Modes.** Roam (reading as it walks; the whisper) → Alert (it looks up
//!   over the page: the whisper stops) → Hunt (it snaps the sheet flat and
//!   comes at 4.8 m/s, faster than you walk, slower than you run) → Read
//!   (the compulsion) → Strike (at arm's length it lowers the sheet, 0.8 s:
//!   then the touch) → Withdraw. Sent by the director it Stalks toward your
//!   part of the level, still reading.
//! - **Placed** (`place`) on levels with paper lying about, from depth 2,
//!   near the paper (in an office wing when there is one); it roams there.
//! - **Body**: `bodies.bin`'s `reader` (`assets/src/creatures/body_reader.py`):
//!   its stoop over the sheet is the rest pose; its eyes are on bones of
//!   their own and read the page, line by line.
use crate::base::numeric::hypot2;
use crate::entities::body::{BodyRig, two_bone};
use crate::entities::brain::*;
use crate::entities::creature::*;
use crate::entities::{EntVoice, with_skin_detail, wrap_pi};
use crate::prelude::*;

pub const DATA: KindData = KindData {
    sight: SIGHT,
    sight_lit: SIGHT,
    cone: 0.42,
    hearing: 0.0,
    smell: 0.0,
    radius: 0.3,
    reach: 1.1,
    sight_gain: 1.4,
    sound_gain: 0.0,
    forget: 0.3,
    touch_nerve: 0.0,
};

/// How far it sees you, m (only lit or moving).
pub const SIGHT: f64 = 12.0;
/// You are lit where the light map is over this (or your lamp is on).
pub const LIT: f64 = 0.35;
/// You are moving faster than this, m/s.
pub const MOVING: f64 = 0.6;
/// A sheet on the floor this near stops it, m.
pub const PAPER_REACH: f64 = 4.0;
/// It reads a sheet to the end: this long, s (after stooping for it).
pub const READ_TIME: f64 = 3.0;
/// Stooping to pick a sheet up, s.
pub const STOOP: f64 = 0.8;
/// Once it has learned (you escaped it with paper twice, design 4),
/// it tears a sheet in half unread instead: this long, s.
pub const TEAR_TIME: f64 = 1.0;
/// Its pace when it comes, m/s (faster than you walk, slower than you run).
pub const HUNT: f64 = 4.8;
/// It lowers its sheet before it touches you, s (a tell you can see).
pub const WINDUP: f64 = 0.8;
const ROAM: f64 = 0.9;
const STALK: f64 = 2.2;
const SEARCH: f64 = 2.4;
/// Sent, it reads its way toward you until this near, m.
const STALK_NEAR: f64 = 9.0;
/// Told where you are, it takes a new hint at most this often, s.
const HINT_EVERY: f64 = 5.0;
/// It feels you this close, whatever it sees, m.
const TOUCH: f64 = 0.8;
const WHISPER_REACH: f64 = 10.0;
const SNAP_REACH: f64 = 18.0;
/// Coming, its quick soft steps behind you carry this far, m (a tell).
const STEPS_REACH: f64 = 9.0;

thread_local! {
    static MATS: std::cell::RefCell<std::collections::HashMap<String, Mat>> = Default::default();
}

/// Its materials: sallow skin; the knit and the wool dull; shoe leather a
/// little shiny; paper matt, both sides drawn; the eyes and the teeth.
pub fn material(slot: &str) -> Mat {
    MATS.with(|m| {
        m.borrow_mut()
            .entry(slot.to_string())
            .or_insert_with(|| match slot {
                "cloth" => with_skin_detail(lambert!({ color: 0xffffff, vertexColors: true, emissive: 0x060504 }), 3.6, 0.7),
                "shoe" => phong!({ color: 0xffffff, vertexColors: true, specular: 0x2a2824, shininess: 30 }),
                "paper" => lambert!({ color: 0xffffff, vertexColors: true, emissive: 0x0c0b09, side: DoubleSide }),
                "eye" => phong!({ color: 0xffffff, vertexColors: true, specular: 0x555555, shininess: 90 }),
                "teeth" => lambert!({ color: 0xffffff, vertexColors: true }),
                _ => with_skin_detail(phong!({ color: 0xffffff, vertexColors: true, specular: 0x2a2826, shininess: 24 }), 3.2, 1.0),
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

/// A sheet on the floor it would read.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Paper {
    /// One you dropped (`Game::paper_markers`): it takes it
    Dropped,
    /// A pile lying about (`Game::papers`, by index): it reads the top sheet, once
    Pile(usize),
}

/// The nearest paper on the floor within `PAPER_REACH` of creature `ei`
/// that it can see: the sheets you dropped, and the piles it has not read.
/// Her folded sheets and the flat ones are neither, so it never reads them.
pub fn paper_near(g: &Game, ei: usize) -> Option<(Paper, f64, f64)> {
    let pos = g.entities[ei].pos();
    let read = g.entities[ei].n("readPiles") as u64;
    let mut best: Option<(Paper, f64, f64)> = None;
    let mut bd = PAPER_REACH;
    for m in &g.paper_markers {
        let d = hypot2(m.world_x - pos.x, m.world_z - pos.z);
        if d < bd && g.e_los(pos.x, pos.z, m.world_x, m.world_z) {
            bd = d;
            best = Some((Paper::Dropped, m.world_x, m.world_z));
        }
    }
    for (i, p) in g.papers.iter().enumerate() {
        if p.taken || (i < 52 && read & (1u64 << i) != 0) {
            continue;
        }
        let q = p.g.pos();
        let d = hypot2(q.x - pos.x, q.z - pos.z);
        if d < bd && g.e_los(pos.x, pos.z, q.x, q.z) {
            bd = d;
            best = Some((Paper::Pile(i), q.x, q.z));
        }
    }
    best
}

/// It has the sheet at (x, z): a dropped one is taken off the floor (into
/// its pockets), a pile keeps its sheets but is not read again.
fn take_paper(g: &mut Game, ei: usize, what: Paper, x: f64, z: f64) {
    match what {
        Paper::Dropped => {
            let near = g
                .paper_markers
                .iter()
                .enumerate()
                .map(|(i, m)| (i, hypot2(m.world_x - x, m.world_z - z)))
                .min_by(|a, b| a.1.total_cmp(&b.1));
            if let Some((i, d)) = near
                && d < 0.5
            {
                let m = g.paper_markers.remove(i);
                g.world.remove(&m.mesh);
            }
        }
        Paper::Pile(i) => {
            if i < 52 {
                let read = g.entities[ei].n("readPiles") as u64 | (1u64 << i);
                g.entities[ei].set_n("readPiles", read as f64);
            }
        }
    }
    call(g, ei, "rustle");
}

/// Whether you are moving (over the last few frames).
fn you_moving(g: &mut Game, ei: usize, dt: f64) -> bool {
    let pp = g.player.pos;
    let Some(c) = g.creature_mut(ei) else { return false };
    let (lx, lz) = (c.get("ppx"), c.get("ppz"));
    let had = c.get("pp_set") > 0.0;
    c.ease.insert("ppx", pp.x);
    c.ease.insert("ppz", pp.z);
    c.ease.insert("pp_set", 1.0);
    let v = if had && dt > 0.0 { (hypot2(pp.x - lx, pp.z - lz) / dt).min(20.0) } else { 0.0 };
    // (a step's pause does not hide you: it remembers the motion a moment)
    let m = c.eased("moving", if v > MOVING { 1.0 } else { 0.0 }, if v > MOVING { 30.0 } else { 3.0 }, dt);
    m > 0.5
}

/// Its own stream for what it chooses while it goes about its reading (from
/// its home and how many choices it has made): a Reader far from you draws
/// nothing from the per-frame stream, so it moves nothing else in the level.
fn own_stream(g: &mut Game, ei: usize) -> crate::base::rng::SplitMix64 {
    let Some(c) = g.creature_mut(ei) else { return crate::base::rng::SplitMix64::new(0, 0) };
    let k = c.get("choices") + 1.0;
    c.ease.insert("choices", k);
    let (hx, hz) = (c.get("home_x"), c.get("home_z"));
    let seed = (hx.to_bits() ^ hz.to_bits().rotate_left(17)) as u32 ^ (k as u32).wrapping_mul(0x9E37_79B9);
    crate::base::rng::SplitMix64::new(seed, 0x2EAD_60A1)
}

/// An open tile of the level, from stream `r`.
fn any_tile(g: &Game, r: &mut crate::base::rng::SplitMix64) -> (i32, i32) {
    (1 + (r.next() * (g.w - 2).max(1) as f64) as i32, 1 + (r.next() * (g.h - 2).max(1) as f64) as i32)
}

/// A place to walk while reading: an open tile 6–20 m off, near its home
/// (in the office wing, if it has one).
fn roam_goal(g: &mut Game, ei: usize) -> Option<(f64, f64)> {
    let mut r = own_stream(g, ei);
    let pos = g.entities[ei].pos();
    let c = g.creature(ei)?;
    let home = (c.get("home_x"), c.get("home_z"));
    let mut best = None;
    let mut best_score = -1e9;
    let (cx, cz) = (g.tx(pos.x), g.tz(pos.z));
    for _ in 0..14 {
        // (a tile within five of where it stands)
        let (gx, gz) = (cx - 5 + (r.next() * 11.0) as i32, cz - 5 + (r.next() * 11.0) as i32);
        if !g.in_grid(gx, gz) || g.is_wall_tile(gx, gz) {
            continue;
        }
        let (wx, wz) = (g.wx(gx), g.wz(gz));
        let d = hypot2(wx - pos.x, wz - pos.z);
        if !(6.0..20.0).contains(&d) {
            continue;
        }
        let office = g.zone_purpose_at(gx, gz).map(|p| p.label == "office wing").unwrap_or(false);
        let score = -hypot2(wx - home.0, wz - home.1) * 0.15 + if office { 3.0 } else { 0.0 };
        if score > best_score {
            best_score = score;
            best = Some((wx, wz));
        }
    }
    best
}

/// A place far from you to go to (20 m or more), its home when that is.
fn away_goal(g: &mut Game, ei: usize) -> Option<(f64, f64)> {
    let mut r = own_stream(g, ei);
    let pp = g.player.pos;
    let c = g.creature(ei)?;
    let home = (c.get("home_x"), c.get("home_z"));
    if hypot2(home.0 - pp.x, home.1 - pp.z) >= 20.0 {
        return Some(home);
    }
    let mut best = None;
    let mut bd = 0.0;
    for _ in 0..16 {
        let (gx, gz) = any_tile(g, &mut r);
        if !g.in_grid(gx, gz) || g.is_wall_tile(gx, gz) {
            continue;
        }
        let (wx, wz) = (g.wx(gx), g.wz(gz));
        let dp = hypot2(wx - pp.x, wz - pp.z).min(60.0);
        if dp > bd {
            bd = dp;
            best = Some((wx, wz));
        }
    }
    best
}

/// One frame of the Reader's mind.
pub fn think(g: &mut Game, ei: usize, dt: f64) {
    let moving = you_moving(g, ei, dt);
    let Some(c) = g.creature(ei) else { return };
    let (mode, mode_t, order, p) = (c.brain.mode, c.brain.mode_t, c.brain.order, c.brain.last);
    let pos = g.entities[ei].pos();
    let pp = g.player.pos;
    let now = g.perf_t;
    let relaxed = g.director.phase == crate::entities::director::Phase::Relax || matches!(order, Order::Away(_));
    let range = SIGHT * if relaxed { 0.6 } else { 1.0 };
    // ---- the compulsion: a sheet on the floor near it, whatever it was doing
    if mode != Mode::Read
        && !(mode == Mode::Strike && mode_t >= WINDUP)
        && let Some((what, x, z)) = paper_near(g, ei)
    {
        let c = g.creature_mut(ei).unwrap();
        c.ease.insert("paper_x", x);
        c.ease.insert("paper_z", z);
        c.ease.insert("paper_pile", if let Paper::Pile(i) = what { i as f64 + 1.0 } else { 0.0 });
        c.ease.insert("read_stage", 0.0);
        c.ease.insert("was_after_you", if mode.after_you() { 1.0 } else { 0.0 });
        let e = &mut g.entities[ei];
        e.vx = 0.0;
        e.vz = 0.0;
        set(g, ei, Mode::Read, "a sheet on the floor");
        // a sheet dropped while it was after you: you escaped it with paper
        if mode.after_you() {
            g.creature_habit("reader");
        }
        return;
    }
    // ---- its eyes: over the page, only what is lit or moves; and touch
    let lit = p.lamp || p.you_lit > LIT;
    let reading = mode == Mode::Read;
    let seen = !reading && p.line && p.in_view && p.dist < range && (lit || moving);
    let touched = p.dist < TOUCH;
    let feed = if mode == Mode::Withdraw {
        0.0
    } else if touched {
        DATA.sight_gain * 2.0
    } else if seen {
        DATA.sight_gain * (1.0 - p.dist / range).max(0.3)
    } else {
        0.0
    };
    {
        let c = g.creature_mut(ei).unwrap();
        if feed > 0.0 {
            c.brain.mem.seen = Some((pp.x, pp.z, now));
            c.brain.feed(feed, dt, 1.0);
        } else {
            c.brain.drain(DATA.forget, dt);
        }
    }
    let sensing = feed > 0.0;
    let aw = g.creature(ei).unwrap().brain.mem.awareness;
    let last_seen = g.creature(ei).unwrap().brain.mem.seen.map(|(x, z, _)| (x, z));

    match mode {
        Mode::Read => read(g, ei, dt),
        Mode::Withdraw => {
            let goal = g.creature(ei).unwrap().brain.goal.or_else(|| away_goal(g, ei));
            g.creature_mut(ei).unwrap().brain.goal = goal;
            g.creature_go(ei, goal, false, STALK, dt);
            let away = matches!(order, Order::Away(t) if t > 0.0);
            if !away && mode_t > 8.0 {
                g.creature_mut(ei).unwrap().brain.goal = None;
                set(g, ei, Mode::Roam, "back to its reading");
            }
        }
        Mode::Idle | Mode::Roam | Mode::Stalk => {
            if aw >= SUSPECT {
                look_up(g, ei);
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
            // sent, it goes where the director would have it, then reads its
            // way on toward you (it finds you by sight)
            let lurk = stalk && {
                let wt = g.entities[ei].w_target.map(|(x, z)| (g.wx(x), g.wz(z)));
                wt.map(|(x, z)| hypot2(x - pos.x, z - pos.z) < 2.5).unwrap_or(true)
            };
            if lurk {
                if p.dist > STALK_NEAR {
                    g.creature_go(ei, None, true, ROAM * 1.5, dt);
                } else {
                    g.entity_drive(ei, dt, 0.0);
                }
                return;
            }
            let goal = if stalk {
                g.entities[ei].w_target.map(|(x, z)| (g.wx(x), g.wz(z)))
            } else if mode == Mode::Idle {
                // it stands and reads a while (a page turned), then walks on
                if mode_t > g.creature(ei).unwrap().get("idle_for").max(3.0) {
                    set(g, ei, Mode::Roam, "it reads as it walks");
                }
                None
            } else {
                let c = g.creature(ei).unwrap();
                let reached = c.brain.goal.map(|(x, z)| hypot2(x - pos.x, z - pos.z) < 1.5).unwrap_or(true);
                if reached || c.brain.goal_t <= 0.0 {
                    let gl = roam_goal(g, ei);
                    let mut r = own_stream(g, ei);
                    let c = g.creature_mut(ei).unwrap();
                    c.brain.goal = gl;
                    c.brain.goal_t = r.range(20.0, 30.0);
                    if mode_t > 6.0 {
                        let f = r.range(4.0, 8.0);
                        let c = g.creature_mut(ei).unwrap();
                        c.ease.insert("idle_for", f);
                        set(g, ei, Mode::Idle, "it stops to read");
                    }
                }
                g.creature_mut(ei).unwrap().brain.goal_t -= dt;
                g.creature(ei).unwrap().brain.goal
            };
            if goal.is_some() {
                g.creature_go(ei, goal, false, if stalk { STALK } else { ROAM }, dt);
            } else {
                g.entity_drive(ei, dt, 0.0);
            }
        }
        Mode::Alert => {
            // it looks up over the page; its body comes round after
            g.entity_drive(ei, dt, 0.0);
            let h = (pp.x - pos.x).atan2(pp.z - pos.z);
            if mode_t > 0.4 {
                g.entity_turn(ei, h, dt, Some(2.5));
            }
            if sure(aw) && mode_t > 0.6 {
                notice(g, ei);
            } else if mode_t > 1.5 && aw < SUSPECT {
                set(g, ei, Mode::Roam, "back to its page");
            } else if mode_t > 3.0 {
                g.creature_mut(ei).unwrap().brain.goal = last_seen;
                set(g, ei, Mode::Search, "it goes to look");
            }
        }
        Mode::Search => {
            if sure(aw) {
                notice(g, ei);
                return;
            }
            if aw >= SUSPECT && sensing && mode_t > 1.0 {
                look_up(g, ei);
                return;
            }
            if let Order::Seek(x, z) = order
                && now - g.creature(ei).unwrap().get("hint_t") > HINT_EVERY
            {
                hint(g, ei, x, z);
                set(g, ei, Mode::Search, "told again where you are");
                return;
            }
            let goal = g.creature(ei).unwrap().brain.goal.or(last_seen);
            let at = goal.map(|(x, z)| hypot2(x - pos.x, z - pos.z) < 1.5).unwrap_or(true);
            if at {
                g.entity_drive(ei, dt, 0.0);
                let yaw = g.entities[ei].yaw;
                g.entity_turn(ei, wrap_pi(yaw + 0.9 * dt), dt, Some(0.9));
            } else {
                g.creature_go(ei, goal, false, SEARCH, dt);
            }
            if mode_t > 14.0 {
                g.creature_mut(ei).unwrap().brain.goal = None;
                set(g, ei, Mode::Roam, "it gave up looking");
            }
        }
        Mode::Hunt => {
            if aw < SUSPECT {
                g.creature_mut(ei).unwrap().brain.goal = last_seen;
                set(g, ei, Mode::Search, "it lost you");
                return;
            }
            if p.dist < DATA.reach + 0.15 && sensing && g.strike_allowed(ei) {
                set(g, ei, Mode::Strike, "it reached you");
                // it lowers the sheet: you see its face (a tell)
                g.creature_tell(ei, 0.0);
                return;
            }
            if p.dist < 1.0 && sensing {
                g.entity_drive(ei, dt, 0.0);
            } else if sensing {
                g.creature_go(ei, None, true, HUNT, dt);
            } else {
                g.creature_go(ei, last_seen, false, HUNT, dt);
            }
            // its steps, quick and soft, close behind you: a tell
            let c = g.creature_mut(ei).unwrap();
            let left = c.get("steps_t") - dt;
            if left <= 0.0 {
                c.ease.insert("steps_t", 2.0);
                if p.dist < STEPS_REACH {
                    g.creature_tell(ei, STEPS_REACH);
                }
            } else {
                c.ease.insert("steps_t", left);
            }
        }
        Mode::Strike => {
            // the sheet comes down as it comes on (a walk does not outpace
            // it; a run does)
            if p.dist > 0.9 {
                g.creature_go(ei, None, true, HUNT, dt);
            } else {
                g.entity_drive(ei, dt, 0.0);
            }
            if mode_t >= WINDUP {
                if p.dist < DATA.reach + 0.3 {
                    g.creature_contact(ei);
                } else {
                    set(g, ei, Mode::Hunt, "you stepped back");
                }
            }
        }
        _ => set(g, ei, Mode::Roam, "it reads as it walks"),
    }
}

/// The compulsion: to the sheet, stoop and take it, read it to the end;
/// then it looks up, and goes on with what it was doing.
fn read(g: &mut Game, ei: usize, dt: f64) {
    let pos = g.entities[ei].pos();
    let c = g.creature(ei).unwrap();
    let (x, z) = (c.get("paper_x"), c.get("paper_z"));
    let stage = c.get("read_stage");
    let (mode_t, t) = (c.brain.mode_t, c.brain.t);
    let pile = c.get("paper_pile");
    let what = if pile > 0.0 { Paper::Pile(pile as usize - 1) } else { Paper::Dropped };
    let after = c.get("was_after_you") > 0.0;
    if stage == 0.0 {
        // gone (a pile you took) or out of its reach: nothing to read
        let there = match what {
            Paper::Dropped => g.paper_markers.iter().any(|m| hypot2(m.world_x - x, m.world_z - z) < 0.5),
            Paper::Pile(i) => g.papers.get(i).map(|p| !p.taken).unwrap_or(false),
        };
        if !there || mode_t > 8.0 {
            set(g, ei, Mode::Roam, "the sheet was gone");
            return;
        }
        if hypot2(x - pos.x, z - pos.z) < 0.55 {
            g.entity_drive(ei, dt, 0.0);
            let c = g.creature_mut(ei).unwrap();
            c.ease.insert("read_stage", 1.0);
            c.brain.t = 0.0;
        } else {
            g.creature_go(ei, Some((x, z)), false, if after { HUNT } else { STALK }, dt);
        }
        return;
    }
    g.entity_drive(ei, dt, 0.0);
    if stage == 1.0 {
        // stooping for it; at the bottom it has it
        if t >= STOOP * 0.6 && g.creature(ei).unwrap().get("took") == 0.0 {
            take_paper(g, ei, what, x, z);
            g.creature_mut(ei).unwrap().ease.insert("took", 1.0);
        }
        if t >= STOOP {
            // learned, it tears it in half unread (you hear it tear)
            let tears = g.creature_may_show(ei);
            let c = g.creature_mut(ei).unwrap();
            c.ease.insert("read_stage", 2.0);
            c.ease.insert("took", 0.0);
            c.ease.insert("tears", if tears { 1.0 } else { 0.0 });
            c.brain.t = 0.0;
            if tears {
                call(g, ei, "tear");
                g.director_note("tear");
            }
        }
        return;
    }
    // reading it, to the end, blind to you (or tearing it)
    let tears = g.creature(ei).unwrap().get("tears") > 0.0;
    if t >= if tears { TEAR_TIME } else { READ_TIME } {
        let seen = g.creature(ei).unwrap().brain.mem.seen.map(|(x, z, _)| (x, z));
        if after {
            g.creature_mut(ei).unwrap().brain.goal = seen;
            set(g, ei, Mode::Search, "it read it to the end");
        } else {
            set(g, ei, Mode::Roam, "it read it to the end");
        }
    }
}

/// It looks up from its page: the whisper stops (its tell).
fn look_up(g: &mut Game, ei: usize) {
    set(g, ei, Mode::Alert, "it saw you over its page");
    g.creature_tell(ei, WHISPER_REACH);
}

/// The director's hint of where you are: 4–7 m from you, at most every
/// `HINT_EVERY` s; it goes there to look.
fn hint(g: &mut Game, ei: usize, x: f64, z: f64) {
    let (a, r) = with_native(|| (rnd() * PI * 2.0, rand2(4.0, 7.0)));
    let (hx, hz) = g.open_point(x + a.cos() * r, z + a.sin() * r);
    let now = g.perf_t;
    let c = g.creature_mut(ei).unwrap();
    c.ease.insert("hint_t", now);
    c.brain.goal = Some((hx, hz));
}

/// It is sure: it snaps its sheet flat, and comes.
fn notice(g: &mut Game, ei: usize) {
    let now = g.perf_t;
    if let Some(c) = g.creature_mut(ei) {
        c.noticed_at = Some(now);
    }
    set(g, ei, Mode::Hunt, "it is sure where you are");
    g.director_note("notice");
    call(g, ei, "snap");
    g.creature_tell(ei, SNAP_REACH);
    let d = g.perceive(ei).dist;
    if d < 25.0 {
        g.dread_sting((1.2 - d / 25.0).clamp(0.5, 1.0));
    }
}

// ------------------------------------------------------------ placing it

/// Put the Reader on this level (after the build: `session::flow`): from
/// depth 2, on most levels with paper lying about (by the level, not by
/// chance), near the paper, in an office wing when there is one, far from
/// the arrival and her camp. None when there is no paper.
pub fn place(g: &mut Game) -> Option<usize> {
    if g.in_loop || g.in_stairwell || !g.ecology().contains(&"reader") {
        return None;
    }
    let mut h = crate::base::rng::SplitMix64::new(g.current_seed, 0x2EAD_E2E5_u64 ^ g.level_num as u64);
    let piles: Vec<(i32, i32)> = g.papers.iter().filter(|p| !p.taken).map(|p| (g.tx(p.g.pos().x), g.tz(p.g.pos().z))).collect();
    if piles.is_empty() {
        return None;
    }
    let (sx, sz) = (g.tx(g.spawn_pos.x), g.tz(g.spawn_pos.z));
    let from_start = g.bfs(sx, sz);
    let from_camp = g.story.camp.as_ref().map(|c| g.bfs(c.tile.0, c.tile.1));
    let mut best: Option<(f64, i32, i32)> = None;
    for &(px, pz) in &piles {
        for dz in -6..=6 {
            for dx in -6..=6 {
                let (x, z) = (px + dx, pz + dz);
                // (beside the paper, not standing on it)
                if (dx == 0 && dz == 0) || !g.in_grid(x, z) || g.is_wall_tile(x, z) {
                    continue;
                }
                let i = g.t_idx(x, z);
                if from_start[i] < 18 {
                    continue;
                }
                if let Some(fc) = &from_camp
                    && fc[i] >= 0
                    && fc[i] < 5
                {
                    continue;
                }
                let office = g.zone_purpose_at(x, z).map(|p| p.label == "office wing").unwrap_or(false);
                let score = if office { 3.0 } else { 0.0 } - (dx.abs() + dz.abs()) as f64 * 0.2
                    + (from_start[i] as f64).min(40.0) * 0.02
                    + ((x * 7 + z * 13) % 5) as f64 * 0.01;
                if best.map(|b| score > b.0).unwrap_or(true) {
                    best = Some((score, x, z));
                }
            }
        }
    }
    let (_, x, z) = best?;
    // (its making draws on a stream of its own: nothing moves on the
    // per-frame stream as the level starts)
    let own = (h.next() * u32::MAX as f64) as u32;
    let ei = crate::base::rng::with_seed(own, || g.add_entity("reader", x, z, 0));
    settle_in(g, ei, x, z);
    Some(ei)
}

/// Creature `ei` lives at tile (x, z): its home, where it reads.
pub fn settle_in(g: &mut Game, ei: usize, x: i32, z: i32) {
    let (wx, wz) = (g.wx(x), g.wz(z));
    let e = &mut g.entities[ei];
    e.g.set_pos(wx, 0.0, wz);
    e.freeze = 0.0;
    let now = g.perf_t;
    if let Some(c) = g.creature_mut(ei) {
        c.ease.insert("home_x", wx);
        c.ease.insert("home_z", wz);
        c.ease.insert("idle_for", 5.0);
        c.brain.set(Mode::Idle, "it reads where the paper is", now);
    }
}

// ------------------------------------------------------------ the body

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

/// One frame of its body: bent over its page, its eyes going along the
/// lines; looking up over the page at you; stooping for a sheet; the sheet
/// lowered before it touches you; shuffling on planted feet.
pub fn animate(g: &mut Game, ei: usize, dt: f64) {
    let (vx, vz) = (g.entities[ei].vx, g.entities[ei].vz);
    let speed = hypot2(vx, vz);
    let t = g.perf_t;
    let pp = g.player.pos;
    let killer = g.director.killer == Some(ei) && g.state != "playing";
    let Some(c) = g.creature_mut(ei) else { return };
    let rig = c.body.clone();
    let (mode, mode_t) = (c.brain.mode, c.brain.mode_t);
    let stage = c.get("read_stage");
    let bt = c.brain.t;
    let root = rig.root.clone();
    rig.rest();
    if killer {
        let p = root.pos();
        root.rotation().y = (pp.x - p.x).atan2(pp.z - p.z);
    }
    let yaw = root.rotation().y;
    let pos = root.pos();
    let want = wrap_pi((pp.x - pos.x).atan2(pp.z - pos.z) - yaw).clamp(-1.2, 1.2);
    let at_you = matches!(mode, Mode::Alert | Mode::Hunt | Mode::Strike) || killer;
    // ---- how it holds itself
    let up = c.eased("up", if at_you { 1.0 } else { 0.0 }, 7.0, dt);
    let lowered = c.eased("lowered", if killer || (mode == Mode::Strike) { 1.0 } else { 0.0 }, 1.0 / WINDUP * 2.2, dt);
    let stoop_w = if mode == Mode::Read && stage == 1.0 { (bt / STOOP * PI).sin().max(0.0) } else { 0.0 };
    let stoop = c.eased("stoop", stoop_w, 12.0, dt);
    let hunt = c.eased("hunt", if mode == Mode::Hunt { 1.0 } else { 0.0 }, 4.0, dt);
    let hips = rig.ix("hips").unwrap();
    let h0 = rest_head(&rig, hips);
    rig.bones[hips].set_pos(h0.x, h0.y - 0.16 * stoop - 0.02 * hunt, h0.z);
    if let Some(sp) = rig.ix("spine") {
        rig.turn(sp, &Quat::from_euler(&Euler::new(0.45 * stoop + 0.1 * hunt, 0.0, 0.0)));
    }
    if let Some(ch) = rig.ix("chest") {
        rig.turn(ch, &Quat::from_euler(&Euler::new(0.35 * stoop + (t * 1.2).sin() * 0.012, 0.0, 0.0)));
    }
    // ---- the head: down on the page; up over it at you, eyes first
    let held = c.eased("head_yaw", if at_you { want } else { 0.0 }, 5.0, dt);
    if let Some(nk) = rig.ix("neck") {
        rig.turn(nk, &Quat::from_euler(&Euler::new(-0.45 * up + 0.2 * stoop, held * 0.45, 0.0)));
    }
    if let Some(hd) = rig.ix("head") {
        let nod = if at_you { 0.0 } else { (t * 0.9).sin() * 0.03 };
        rig.turn(hd, &Quat::from_euler(&Euler::new(-0.35 * up + nod, held * 0.5, 0.0)));
    }
    // the whisper moves its jaw a little, fast; still when it looks at you
    let whisper = if matches!(mode, Mode::Roam | Mode::Idle | Mode::Stalk | Mode::Read | Mode::Search) { 1.0 } else { 0.0 };
    if let Some(jw) = rig.ix("jaw") {
        let k = ((t * 23.0).sin() * 0.5 + 0.5) * ((t * 7.3).sin() * 0.5 + 0.5);
        // (the sheet down, its mouth hangs open on the word it was reading)
        rig.turn(jw, &Quat::from_euler(&Euler::new(0.05 * k * whisper + 0.26 * lowered, 0.0, 0.0)));
    }
    // ---- the eyes: along the lines of the page, left to right and back;
    // on you when it looks up
    let line = (t * 1.9).fract();
    let (ey, ex) = if at_you {
        (held * 0.4, -0.25)
    } else if line < 0.8 {
        (-0.32 + 0.64 * (line / 0.8), 0.06 * ((t * 1.9).floor() % 6.0) / 6.0)
    } else {
        (0.32 - 0.64 * ((line - 0.8) / 0.2), 0.06 * ((t * 1.9).floor() % 6.0) / 6.0)
    };
    for side in ["eye.L", "eye.R"] {
        if let Some(i) = rig.ix(side) {
            rig.turn(i, &Quat::from_euler(&Euler::new(ex, ey, 0.0)));
        }
    }
    // ---- the arms: the page before its face; lowered before it touches
    // you; one hand to the floor when it stoops
    for side in [".L", ".R"] {
        let sx = if side == ".L" { 1.0 } else { -1.0 };
        let (Some(ua), Some(fa)) = (rig.ix(&format!("upperarm{side}")), rig.ix(&format!("forearm{side}"))) else { continue };
        let ru = rig.rest_dir[ua];
        let rf = rig.rest_dir[fa];
        let down_u = v3(sx * 0.12, -1.0, 0.12);
        let down_f = v3(sx * 0.02, -1.0, 0.35);
        let reach = if side == ".R" { stoop } else { 0.0 };
        let floor_u = v3(-0.1, -0.7, 0.7);
        let floor_f = v3(-0.05, -0.9, 0.4);
        let k = lowered.max(reach);
        let (tu, tf) = if reach > lowered { (floor_u, floor_f) } else { (down_u, down_f) };
        let mix = |a: Vec3, b: Vec3, k: f64| v3(a.x + (b.x - a.x) * k, a.y + (b.y - a.y) * k, a.z + (b.z - a.z) * k);
        rig.aim(ua, mix(ru, tu, k), 0.0);
        rig.aim(fa, mix(rf, tf, k), 0.0);
    }
    // ---- the legs: planted; a shuffle reading, long quick steps coming
    if c.limbs.len() != 2 {
        c.limbs = vec![Limb::default(); 2];
    }
    let stride = if speed > 2.5 { 1.4 } else { 0.7 };
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
        if !limb.planted || hypot2(limb.at[0] - home_w.x, limb.at[2] - home_w.z) > 1.6 {
            limb.at = [home_w.x, rest_end.y, home_w.z];
            limb.planted = true;
        }
        if speed > 0.15 && lp > 0.5 {
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
            // a shuffle barely lifts the foot
            limb.at[1] = rest_end.y + (s * PI).sin() * if speed > 2.5 { 0.14 } else { 0.035 };
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
    let _ = mode_t;
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
    let pos = g.entities[ei].pos();
    slot.at(pos.x, 1.5, pos.z);
    Some(slot)
}

/// Its calls: `snap` (the sheet snapped flat: it comes), `rustle` (a sheet
/// picked up).
pub fn call(g: &mut Game, ei: usize, what: &str) {
    let Some(s) = g.soundscape() else { return };
    let Some(v) = voice_slot(g, ei).map(|slot| slot.input.clone()) else { return };
    let t = g.ac_time();
    match what {
        "snap" => {
            // a crack of paper pulled taut, and its flutter
            s.burst(&v, t, 0.025, "highpass", 2200.0, 1.0, 0.42, None, 0.001);
            s.burst(&v, t + 0.004, 0.08, "bandpass", 1500.0, 1.2, 0.16, Some(900.0), 0.003);
            s.burst(&v, t + 0.05, 0.25, "bandpass", 4200.0, 0.9, 0.04, Some(3000.0), 0.02);
        }
        "tear" => {
            // a sheet torn across: a ripping that rises, and the halves
            s.burst(&v, t, 0.45, "bandpass", 1400.0, 1.0, 0.16, Some(3200.0), 0.01);
            for k in 0..6 {
                s.burst(&v, t + 0.04 + k as f64 * 0.06, 0.02, "highpass", 2600.0, 1.0, 0.07, None, 0.001);
            }
        }
        "rustle" => {
            // (fixed: it reads far from you too, and draws nothing then)
            for (k, f) in [3100.0, 2700.0, 3900.0, 3300.0].into_iter().enumerate() {
                s.burst(&v, t + k as f64 * 0.07 + [0.0, 0.02, 0.01, 0.03][k], 0.06, "bandpass", f, 1.1, 0.05, None, 0.01);
            }
        }
        _ => {}
    }
}

/// Its whisper while it reads: fast, without a voice, under its breath, in
/// runs of syllables with a breath between, one run at a time (so it stops
/// within a syllable when it looks up); quick soft steps when it comes.
pub fn voice(g: &mut Game, ei: usize, dt: f64, d: f64) {
    if g.ac.is_none() || d > WHISPER_REACH + 6.0 {
        return;
    }
    let Some(s) = g.soundscape() else { return };
    let Some(slot) = voice_slot(g, ei) else { return };
    let pos = g.entities[ei].pos();
    let pp = g.player.pos;
    let clear = g.e_los(pos.x, pos.z, pp.x, pp.z);
    slot.occlude(if clear { 0.0 } else { 0.7 });
    let v = slot.input.clone();
    let t = g.ac_time();
    let Some(c) = g.creature_mut(ei) else { return };
    let mode = c.brain.mode;
    c.voice_t -= dt;
    if c.voice_t > 0.0 {
        return;
    }
    match mode {
        Mode::Roam | Mode::Idle | Mode::Stalk | Mode::Search | Mode::Withdraw | Mode::Read => {
            // a run of 4–8 syllables, each a breath of shaped noise (the
            // hiss of the consonant over the breath under it); a burst
            // falls away over its length, so each is given long enough to
            // be heard as a syllable, and they run into each other
            let n = with_native(|| irand(4, 8));
            let mut at = 0.0;
            for _ in 0..n {
                let f = with_native(|| rand2(2600.0, 4800.0));
                s.burst(&v, t + at, 0.42, "bandpass", f, 1.2, 0.12, Some(f * with_native(|| rand2(0.8, 1.15))), 0.015);
                s.burst(&v, t + at, 0.35, "bandpass", with_native(|| rand2(900.0, 1400.0)), 0.9, 0.05, None, 0.02);
                at += with_native(|| rand2(0.09, 0.15));
            }
            c.voice_t = at + with_native(|| rand2(0.2, 0.5));
        }
        Mode::Hunt => {
            // soft soles, quick
            s.burst(&v, t, 0.05, "bandpass", 1300.0, 1.1, 0.07, Some(800.0), 0.004);
            c.voice_t = 0.29;
        }
        _ => c.voice_t = 0.3,
    }
}
