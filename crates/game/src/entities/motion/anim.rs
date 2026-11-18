//! A creature's motion as it plays: the clip its kind picks ([`Pick`]),
//! blended from the last, played to the speed it moves at, held and snapped,
//! its feet (and on all fours its hands) locked to the floor while they bear
//! weight and stepped, never slid, when they let go.
//!
//! The game moves the creature (its brain and steering); the motion follows:
//! a clip that travels is played at the pace and stride that make its feet
//! keep up with the ground ([`Anim::advance`]). A held frame stops the body
//! where it is while the root goes on; the snap after it catches up, so the
//! creature stops dead and lurches on to where it would have been (wrong
//! mechanics, `docs/design/ENTITIES.md` §6), and nothing it does reaches its
//! place in the world.
//!
//! What a pose depends on: the creature's state ([`Drive`]), the clips, and
//! its own stream ([`Anim::new`]'s seed): nothing per frame is drawn from
//! the game's streams (invariant 1).
use super::retarget::{Ends, Gains, Retarget, set_global, slerp};
use super::{Frame, Library, body_to_world, mix, sub, world_to_body};
use crate::entities::body::{BodyRig, qmul};
use crate::prelude::*;
use std::collections::BTreeMap;

/// Where the creature is and how it moves this frame (the world's frame).
#[derive(Clone, Copy, Debug)]
pub struct Drive {
    pub pos: Vec3,
    pub yaw: f64,
    /// its velocity, m/s
    pub vel: (f64, f64),
    pub dt: f64,
}

impl Drive {
    /// Its speed ahead (negative: backing away) and sideways, m/s.
    pub fn ahead(&self) -> (f64, f64) {
        let (s, c) = self.yaw.dsin_cos();
        let (vx, vz) = self.vel;
        (vx * s + vz * c, vx * c - vz * s)
    }

    pub fn speed(&self) -> f64 {
        self.vel.0.dhypot(self.vel.1)
    }
}

/// What a kind wants played this frame.
#[derive(Clone, Copy, Debug)]
pub struct Pick {
    pub clip: &'static str,
    /// where in it to start when it is switched to, s
    pub from: f64,
    /// how much of the mocap each part takes
    pub gains: Gains,
    /// how long to blend from the last clip, s
    pub fade: f64,
    /// played to the speed the creature moves at (else at its own pace)
    pub travel: bool,
    /// its own pace, times
    pub rate: f64,
    /// held frames: the gaps between them, s (none: it never holds)
    pub holds: Option<(f64, f64)>,
    /// the feet (and hands) locked to the floor while they touch
    pub lock: bool,
    /// played at this time (s) whatever its pace: a clip scrubbed by a rule's own clock
    pub at: Option<f64>,
    /// going the way it faces away from, it walks backward (the clip played
    /// back to front); else a body moving off its heading is turning, and
    /// goes on forward
    pub backs: bool,
    /// how much of a travelling clip's speed-up is its pace (the rest its
    /// stride's): 0.42, a heavy body's, its strides lengthening more than its
    /// steps quicken; more, quicker steps and shorter strides
    pub cadence: f64,
    /// the most it is played past its own pace
    pub max_pace: f64,
}

impl Pick {
    pub fn new(clip: &'static str) -> Pick {
        Pick {
            clip,
            from: 0.0,
            gains: Gains::default(),
            fade: 0.3,
            travel: false,
            rate: 1.0,
            holds: None,
            lock: true,
            at: None,
            backs: false,
            cadence: 0.42,
            max_pace: 2.2,
        }
    }
    pub fn travel(mut self) -> Pick {
        self.travel = true;
        self
    }
    pub fn gains(mut self, g: Gains) -> Pick {
        self.gains = g;
        self
    }
    pub fn fade(mut self, s: f64) -> Pick {
        self.fade = s;
        self
    }
    pub fn from(mut self, s: f64) -> Pick {
        self.from = s;
        self
    }
    pub fn rate(mut self, r: f64) -> Pick {
        self.rate = r;
        self
    }
    pub fn holds(mut self, lo: f64, hi: f64) -> Pick {
        self.holds = Some((lo, hi));
        self
    }
    pub fn cadence(mut self, c: f64) -> Pick {
        self.cadence = c;
        self
    }
    pub fn max_pace(mut self, m: f64) -> Pick {
        self.max_pace = m;
        self
    }
    pub fn backs(mut self) -> Pick {
        self.backs = true;
        self
    }
    pub fn unlocked(mut self) -> Pick {
        self.lock = false;
        self
    }
    pub fn at(mut self, t: f64) -> Pick {
        self.at = Some(t);
        self
    }
}

/// A clip playing: which, and where in it.
#[derive(Clone, Copy, Debug)]
struct Play {
    clip: usize,
    t: f64,
}

/// A foot or hand held to the floor (world).
#[derive(Clone, Copy, Debug, Default)]
struct Lock {
    at: Option<Vec3>,
    /// a step after a release: where it was, the time left, its length
    release: Option<(Vec3, f64, f64)>,
    /// where the clip put it last frame (world)
    prev: Option<Vec3>,
}

/// One creature's motion.
#[derive(Clone, Debug)]
pub struct Anim {
    pub lib: &'static Library,
    pub rt: Retarget,
    /// how much of its kind's wrongness is laid on (1 in the game; the lab
    /// shows the mocap bare at 0)
    pub wrong: f64,
    /// seconds it has been animated
    pub time: f64,
    /// what the kind eases between (by name)
    pub ease: BTreeMap<&'static str, f64>,
    /// the head's held turn toward what it watches (yaw, pitch)
    pub head: (f64, f64),
    /// the finger bones: (bone, side 0 left 1 right, finger 1..5, joint 1..3)
    pub fingers: Vec<(usize, usize, usize, usize)>,
    /// the clip now, and the one it is fading from (and the time left)
    cur: Play,
    cur_name: &'static str,
    prev: Option<(Play, f64)>,
    fade: f64,
    gains: Gains,
    /// held: the time left and its length
    hold: Option<(f64, f64)>,
    next_hold: f64,
    /// the body behind its root while held (world, x and z)
    lag: (f64, f64),
    last_pos: Option<Vec3>,
    /// each foot, then each hand
    locks: [Lock; 4],
    /// the distance from each lock to where the clip wants its end (a trace)
    pub lock_gap: [f64; 4],
    rng: u64,
    /// the stride and pace the last travelling clip was played at
    pub stride: f64,
    pub pace: f64,
    /// a clip its kind asked for that its library has not (a test reads it:
    /// the motion would quietly go on with the last one)
    pub missing: Option<&'static str>,
    /// how heavy it moves (0..1): how far its pelvis sinks onto each footfall
    pub mass: f64,
    /// the pelvis sunk onto its footfalls: how far (m, below 0 down) and how fast
    dip: (f64, f64),
    /// its velocity as measured from where it was put each frame (world, smoothed)
    vel: (f64, f64),
    seen_at: Option<Vec3>,
}

/// The lurch after a held frame: what it missed made up in this long, s
/// (quick enough to be wrong, slow enough to be a body's: in 80 ms it was a
/// film skipping, a cartoon's).
const SNAP: f64 = 0.22;
/// Its footfalls' weight: the pelvis sprung on its legs, its frequency
/// (Hz) and damping, and the most it sinks, m. Damped critically: the weight
/// comes down and settles, never springs back up (a dip that rebounded read
/// as a hop in the blind review).
const DIP_HZ: f64 = 2.4;
const DIP_DAMP: f64 = 1.0;
const DIP_MAX: f64 = 0.045;
/// A held body falls at most this far behind its root, m.
const MAX_LAG: f64 = 0.5;

impl Anim {
    /// A motion for `rig` from `lib` (`fours`: its hands are feet too),
    /// starting on `first`; `seed` is its own stream (choose it from where
    /// the creature was put, never from the game's streams).
    pub fn new(rig: &BodyRig, lib: &'static Library, fours: bool, first: &'static str, seed: u64) -> Option<Anim> {
        let rt = Retarget::new(rig, lib, fours)?;
        let clip = lib.clip(first).unwrap_or(0);
        let mut fingers = vec![];
        for (side, suf) in [".L", ".R"].iter().enumerate() {
            for f in 1..=5 {
                for k in 1..=3 {
                    if let Some(b) = rig.ix(&format!("finger{f}{k}{suf}")) {
                        fingers.push((b, side, f, k));
                    }
                }
            }
        }
        Some(Anim {
            lib,
            rt,
            wrong: 1.0,
            time: 0.0,
            ease: BTreeMap::new(),
            head: (0.0, 0.0),
            fingers,
            cur: Play { clip, t: 0.0 },
            cur_name: first,
            prev: None,
            fade: 0.3,
            gains: Gains::default(),
            hold: None,
            next_hold: 2.5,
            lag: (0.0, 0.0),
            last_pos: None,
            locks: [Lock::default(); 4],
            lock_gap: [0.0; 4],
            rng: seed | 1,
            stride: 1.0,
            pace: 1.0,
            missing: None,
            mass: 1.0,
            dip: (0.0, 0.0),
            vel: (0.0, 0.0),
            seen_at: None,
        })
    }

    /// What drives it this frame, from where it is (world) and how it is
    /// turned: its velocity measured from where it was last frame (whatever
    /// moved it: steering, or put there), smoothed; a jump of more than 2 m
    /// (put somewhere else) plants its feet anew.
    pub fn drive(&mut self, pos: Vec3, yaw: f64, dt: f64) -> Drive {
        if let Some(p) = self.seen_at {
            let (dx, dz) = (pos.x - p.x, pos.z - p.z);
            if dx.dhypot(dz) > 2.0 {
                self.replant();
                self.vel = (0.0, 0.0);
            } else if dt > 1e-6 {
                let k = (dt * 12.0).min(1.0);
                self.vel = (self.vel.0 + (dx / dt - self.vel.0) * k, self.vel.1 + (dz / dt - self.vel.1) * k);
            }
        }
        self.seen_at = Some(pos);
        Drive { pos, yaw, vel: self.vel, dt }
    }

    /// Its own stream: 0..1.
    fn draw(&mut self) -> f64 {
        // xorshift64*
        let mut x = self.rng;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.rng = x;
        (x.wrapping_mul(0x2545_f491_4f6c_dd1d) >> 11) as f64 / (1u64 << 53) as f64
    }

    /// A value eased toward `target` at `rate` (per second).
    pub fn eased(&mut self, key: &'static str, target: f64, rate: f64, dt: f64) -> f64 {
        let v = self.ease.entry(key).or_insert(target);
        *v += (target - *v) * (dt * rate).min(1.0);
        *v
    }

    pub fn get(&self, key: &'static str) -> f64 {
        self.ease.get(key).copied().unwrap_or(0.0)
    }

    /// The clip playing, and how far into it (s).
    pub fn playing(&self) -> (&'static str, f64) {
        (self.cur_name, self.cur.t)
    }

    /// Held this frame (a trace).
    pub fn held(&self) -> bool {
        matches!(self.hold, Some((left, _)) if left > 0.0)
    }

    /// Each foot, then each hand: planted (held where it stands, not
    /// stepping to a new place), and how far from where the clip wants it.
    pub fn locks(&self) -> [(bool, f64); 4] {
        [0, 1, 2, 3].map(|s| (self.locks[s].at.is_some() && self.locks[s].release.is_none(), self.lock_gap[s]))
    }

    /// Forget where the feet stood (the creature was put somewhere else).
    pub fn replant(&mut self) {
        self.locks = [Lock::default(); 4];
        self.lag = (0.0, 0.0);
        self.last_pos = None;
        self.hold = None;
    }

    /// One frame of `pick`: the clip advanced and sampled, the feet locked,
    /// the body posed. The kind lays its own on top after.
    pub fn play(&mut self, rig: &BodyRig, pick: &Pick, d: &Drive) -> Frame {
        let fr = self.advance(pick, d);
        let ends = self.plant(&fr, pick, d);
        self.rt.pose(rig, &fr.p, &self.gains, &ends);
        fr
    }

    /// The clip's time on, held frames and the blend: the frame to pose.
    pub fn advance(&mut self, pick: &Pick, d: &Drive) -> Frame {
        let dt = d.dt;
        self.time += dt;
        let lib = self.lib;
        // ---- the clip: switched to with a blend from the last
        if pick.clip != self.cur_name && lib.clip(pick.clip).is_none() {
            self.missing = Some(pick.clip);
        }
        if pick.clip != self.cur_name
            && let Some(c) = lib.clip(pick.clip)
        {
            self.prev = Some((self.cur, pick.fade.max(1e-3)));
            self.fade = pick.fade.max(1e-3);
            self.cur = Play { clip: c, t: pick.from };
            self.cur_name = pick.clip;
        }
        // its gains ease over the blend (no pop when a part is let go)
        let k = (dt / self.fade.max(0.05)).min(1.0);
        self.gains = self.gains.mix(&pick.gains, k);
        // ---- held frames: it stops dead, holds, and lurches on
        let speed = d.speed();
        let held = matches!(self.hold, Some((left, _)) if left > 0.0);
        if let Some((lo, hi)) = pick.holds.filter(|_| self.wrong > 0.0) {
            if self.next_hold > self.time + hi {
                self.next_hold = self.time + lo;
            }
            if !held && self.hold.is_none() && self.time >= self.next_hold {
                // never so long that the body falls far behind its root
                let len = (0.35 + 0.45 * self.draw()).min(MAX_LAG / speed.max(0.1));
                self.hold = Some((len, len));
                self.next_hold = self.time + len + lo + (hi - lo) * self.draw();
            }
        } else if let Some((left, len)) = self.hold.filter(|_| held) {
            // a clip that never holds ends a hold begun before it: the snap
            // now (what was held is made up at once)
            self.hold = Some((0.0, len - left));
        }
        let mut rate = pick.rate;
        let mut snap = false;
        if let Some((left, len)) = self.hold {
            if left > 0.0 {
                self.hold = Some((left - dt, len));
                rate = 0.0;
            } else if left > -SNAP {
                // the snap: what it missed, in 80 ms
                rate = pick.rate * (1.0 + len / SNAP);
                snap = true;
                self.hold = Some((left - dt, len));
            } else {
                self.hold = None;
            }
        }
        // ---- the lag: while held the body stays, the root goes on
        let moved = self.last_pos.map_or((0.0, 0.0), |p| (d.pos.x - p.x, d.pos.z - p.z));
        self.last_pos = Some(d.pos);
        if rate == 0.0 {
            self.lag = (self.lag.0 - moved.0, self.lag.1 - moved.1);
            let l = self.lag.0.dhypot(self.lag.1);
            if l > MAX_LAG {
                self.lag = (self.lag.0 * MAX_LAG / l, self.lag.1 * MAX_LAG / l);
            }
        } else if snap || self.lag.0 != 0.0 || self.lag.1 != 0.0 {
            let keep = if snap { (1.0 - dt / SNAP).max(0.0) } else { (1.0 - dt * 20.0).max(0.0) };
            self.lag = (self.lag.0 * keep, self.lag.1 * keep);
            if self.lag.0.abs() + self.lag.1.abs() < 1e-4 {
                self.lag = (0.0, 0.0);
            }
        }
        // ---- the pace: a travelling clip played to the ground's speed
        let clip = &lib.clips[self.cur.clip];
        let (ahead, _) = d.ahead();
        let (mut stride, mut pace) = (1.0, 1.0);
        if pick.travel && clip.speed > 0.05 {
            let going = if pick.backs { ahead.abs() } else { d.speed() };
            let want = going / (clip.speed * self.rt.k);
            // a faster pace and a longer stride, shared (a run is not a jog played fast)
            // (never played much past twice its pace: a clip played fast
            // reads as a cartoon's, not as a body's)
            pace = want.max(1e-3).dpowf(pick.cadence).clamp(0.45, pick.max_pace);
            stride = (want / pace).clamp(0.4, 3.2);
            if pick.backs && ahead < -0.05 {
                pace = -pace;
            }
        }
        self.stride = self.stride + (stride - self.stride) * (dt * 6.0).min(1.0);
        self.pace = pace;
        let prev_rate = if rate > 0.0 { 1.0 } else { 0.0 };
        match pick.at {
            Some(t) => self.cur.t = t,
            None => self.cur.t += dt * rate * pace,
        }
        if let Some((p, left)) = self.prev.as_mut() {
            p.t += dt * prev_rate;
            *left -= dt;
            if *left <= 0.0 {
                self.prev = None;
            }
        }
        // ---- the pose: the current clip, faded in over the last
        let mut fr = clip.sample(self.cur.t, lib.fps);
        if let Some((p, left)) = self.prev {
            let old = lib.clips[p.clip].sample(p.t, lib.fps);
            let w = 1.0 - left / self.fade;
            fr = Frame::lerp(&old, &fr, w * w * (3.0 - 2.0 * w));
        }
        fr
    }

    /// Where each foot (and on all fours each hand) goes: held where it
    /// touches the floor (when the clip says so, or holds it low and still
    /// before it says so); let go, it steps to where the clip wants it,
    /// never slides there.
    pub fn plant(&mut self, fr: &Frame, pick: &Pick, d: &Drive) -> Ends {
        let dt = d.dt;
        let (pos, yaw) = (d.pos, d.yaw);
        let shift = world_to_body(v3(0, 0, 0), yaw, v3(self.lag.0, 0.0, self.lag.1));
        let mut ends = Ends { shift, stride: self.stride, ..Ends::default() };
        let mut fall = 0.0;
        let k = self.rt.k;
        let n = if self.rt.fours { 4 } else { 2 };
        for e in 0..n {
            let (s, hand) = (e % 2, e >= 2);
            let g = &self.gains;
            let free_b = if hand {
                self.rt.wrist_target(&fr.p, s, g.arms, self.stride)
            } else {
                self.rt.ankle_target(&fr.p, s, g.legs, self.stride)
            };
            let free_b = v3(free_b.x + shift.x, free_b.y, free_b.z + shift.z);
            if !pick.lock {
                self.locks[e] = Lock::default();
                continue;
            }
            let free = body_to_world(pos, yaw, free_b);
            let rest = if hand { self.rt.rest_wrist(s) } else { self.rt.rest_ankle(s) };
            let floor = rest.y;
            // under its hip: a lock further from here than the limb can
            // comfortably reach is stepped from, never knelt to
            let under = body_to_world(pos, yaw, v3(rest.x + shift.x, rest.y, rest.z + shift.z));
            let reach = 0.7 * self.rt.leg_len_of(e);
            let lock = &mut self.locks[e];
            let still = lock.prev.is_some_and(|p| (p.x - free.x).dhypot(p.z - free.z) < 0.35 * dt.max(1e-4));
            lock.prev = Some(free);
            let down = if hand { fr.hand_down(s) } else { fr.foot_down(s) };
            // (on all fours a foot is down only near the floor: a crawl on the
            // knees keeps its toes down while the ankle rises behind the knee)
            let near = !self.rt.fours || hand || free.y < floor + 0.05;
            let touching = (down && near) || (free.y < floor + 0.02 && still);
            // a step's time: longer for a longer step, quicker the faster it goes
            let fast = d.speed();
            let span = |a: Vec3, b: Vec3| ((a.x - b.x).dhypot(a.z - b.z) / (2.5 + fast)).clamp(0.08, 0.25);
            match (touching, lock.at) {
                (true, None) => {
                    lock.at = Some(free);
                    // a footfall: its weight comes down on it
                    if !hand || self.rt.fours {
                        fall += 1.0;
                    }
                }
                (true, Some(at)) if (at.x - free.x).dhypot(at.z - free.z) > 0.45 * k || (at.x - under.x).dhypot(at.z - under.z) > reach => {
                    // stretched too far (a turn on the spot, a change of clip): a
                    // step, to where the foot will be wanted when it lands
                    let dd = span(at, free);
                    lock.release = Some((at, dd, dd));
                    lock.at = Some(v3(free.x + d.vel.0 * dd, free.y, free.z + d.vel.1 * dd));
                }
                (false, Some(at)) => {
                    let dd = span(at, free);
                    lock.release = Some((at, dd, dd));
                    lock.at = None;
                }
                _ => {}
            }
            let mut target = match lock.at {
                Some(at) => v3(at.x, free.y, at.z),
                None => free,
            };
            if let Some((from, left, len)) = lock.release {
                let w = (left / len).clamp(0.0, 1.0);
                let gap = (from.x - target.x).dhypot(from.z - target.z);
                // a step clears the floor; a few centimetres are only eased
                let lift = if gap > 0.06 { (PI * (1.0 - w)).dsin() * (0.05 + 0.12 * gap).min(0.16) } else { 0.0 };
                target = v3(target.x + (from.x - target.x) * w, target.y + lift, target.z + (from.z - target.z) * w);
                lock.release = if left - dt > 0.0 { Some((from, left - dt, len)) } else { None };
            }
            self.lock_gap[e] = (target.x - free.x).dhypot(target.z - free.z);
            let b = Some(world_to_body(pos, yaw, target));
            if hand {
                ends.wrists[s] = b;
            } else {
                ends.ankles[s] = b;
            }
        }
        // its weight: each footfall while it moves pushes the pelvis down on its
        // legs, sprung back up (a body's mass, not a puppet's)
        let speed = d.speed();
        if fall > 0.0 && speed > 0.3 {
            self.dip.1 -= fall * self.mass * (0.16 + 0.04 * speed).min(0.4);
        }
        let w = 2.0 * PI * DIP_HZ;
        let (y, v) = self.dip;
        let v = v + (-w * w * y - 2.0 * DIP_DAMP * w * v) * dt;
        let y = (y + v * dt).clamp(-DIP_MAX, DIP_MAX * 0.4);
        self.dip = (y, v);
        ends.shift = v3(ends.shift.x, ends.shift.y + y, ends.shift.z);
        ends
    }

    // ------------------------------------------------------------ layers

    /// The head on `at` (world; `None`: ahead), held until it is off by more
    /// than `snap` radians, then snapped to it (`snap` 0: it follows
    /// smoothly), the neck taking `neck` of the turn; `roll` tilts it.
    /// → the turn laid on (yaw, pitch).
    #[allow(clippy::too_many_arguments, reason = "a look is all of these")]
    pub fn look(&mut self, rig: &BodyRig, at: Option<Vec3>, d: &Drive, snap: f64, reach: (f64, f64), neck: f64, roll: f64) -> (f64, f64) {
        let m = &self.rt.map;
        let (wy, wp) = match at {
            Some(eye) => {
                let head_at = rig.head(m.head);
                let local = world_to_body(d.pos, d.yaw, eye);
                let to = sub(local, v3(head_at.x, head_at.y + 0.15, head_at.z));
                (to.x.datan2(to.z).clamp(-reach.0, reach.0), (-to.y).datan2(to.x.dhypot(to.z)).clamp(-0.9, reach.1))
            }
            None => (0.0, 0.0),
        };
        let (hy, hp) = self.head;
        let err = (wy - hy).dhypot(wp - hp);
        let rate = if snap <= 0.0 {
            6.0
        } else if err > snap * self.wrong.max(0.05) {
            11.0
        } else {
            0.0
        };
        let a = 1.0 - (-rate * d.dt).dexp();
        self.head = (hy + (wy - hy) * a, hp + (wp - hp) * a);
        let (hy, hp) = (self.head.0 * self.wrong, self.head.1 * self.wrong);
        let turn = |share: f64, r: f64| Quat::from_euler(&Euler::new(hp * share, hy * share, r));
        let bn = rig.body_rot(m.neck);
        let bh = rig.body_rot(m.head);
        set_global(rig, m.neck, &qmul(&turn(neck, 0.0), &bn));
        set_global(rig, m.head, &qmul(&turn(1.0, roll), &bh));
        (hy, hp)
    }

    /// A slow breath in the chest (`amp` radians, `rate` per second).
    pub fn breathe(&self, rig: &BodyRig, amp: f64, rate: f64) {
        let m = &self.rt.map;
        if let Some(ch) = m.upperchest.or(m.chest) {
            let breath = Quat::from_euler(&Euler::new((self.time * rate).dsin() * amp * self.wrong, 0.0, 0.0));
            let q: Quat = *rig.bones[ch].quaternion();
            rig.bones[ch].set_quat(&qmul(&q, &breath));
        }
    }

    /// The fingers curled by `curl` (radians a joint; below 0 splayed past
    /// straight), plus `twitch(finger)`.
    pub fn curl(&self, rig: &BodyRig, curl: f64, twitch: &dyn Fn(usize) -> f64) {
        for &(b, side, f, k) in &self.fingers {
            let a = (curl + twitch(f)) * if k == 1 { 0.7 } else { 1.0 } * if f == 1 { 0.5 } else { 1.0 };
            let axis = if side == 0 { v3(0.0, 0.0, -1.0) } else { v3(0.0, 0.0, 1.0) };
            rig.bones[b].set_quat(&Quat::from_axis_angle(&axis, a));
        }
    }

    /// The jaw open by `open` (0..1 of a radian).
    pub fn jaw(&self, rig: &BodyRig, open: f64) {
        if let Some(jw) = self.rt.map.jaw {
            rig.bones[jw].set_quat(&slerp(&Quat::IDENTITY, &Quat::from_euler(&Euler::new(1.0, 0.0, 0.0)), open));
        }
    }

    /// Arm `s` (0 left, 1 right) reaching for `target` (the body's frame),
    /// `w` of the way from where the clip holds it, the elbow toward `pole`.
    pub fn reach(&self, rig: &BodyRig, s: usize, target: Vec3, pole: Vec3, w: f64) {
        if w <= 1e-3 {
            return;
        }
        let a = self.rt.map.arm[s];
        let base = rig.head(a[1]);
        let now = rig.head(a[3]);
        let to = mix(now, target, w.min(1.0));
        let hand = rig.body_rot(a[3]);
        let (du, dl) = crate::entities::body::two_bone(base, to, pole, rig.rest_len[a[1]], rig.rest_len[a[2]]);
        rig.aim(a[1], du, 0.0);
        rig.aim(a[2], dl, 0.0);
        set_global(rig, a[3], &hand);
    }
}
