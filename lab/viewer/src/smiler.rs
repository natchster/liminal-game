//! The Smiler, animated anew: Kimodo's clips on its own body, and on top of
//! them what makes the motion its own (`docs/design/ENTITIES.md`, the Smiler
//! and §6: two vocabularies only, stillness and wrong mechanics).
//!
//! - **The body is a real one.** Its weight shifts, its steps land, its feet
//!   stay where they are put (each foot locked to the floor while the clip
//!   says it touches); a mocap model gives what procedural code could not.
//! - **Its stoop stays.** The retarget lays the mocap's motion on the body's
//!   rest pose, which is the stoop (`motion.rs`).
//! - **Stillness.** At rest it takes only part of the mocap's sway; it
//!   breathes slowly; nothing else.
//! - **The head first, the body after.** Its head goes to what it wants in a
//!   snap and holds there, the body turning round to it later.
//! - **Held frames and snaps.** Searching, it stops dead mid-step, holds,
//!   and lurches on to where it would have been.
//! - **Arms that hang dead.** When it runs, the legs run and the arms swing
//!   as dead weight (a part of the mocap's swing), until the lunge reaches.
//!
//! [`SCENE`] walks it through its modes on the stage toward a viewer, as the
//! game's director and brain would send it.
use crate::motion::{Gains, Library, Retarget, slerp};
use crate::plants::kit::Rng;
use engine::prelude::*;
use game::entities::body::{BodyRig, build_body, qmul};

/// The modes of its design, as the scene uses them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Idle,
    Alert,
    Search,
    Listen,
    Hunt,
    Strike,
    Over,
    Withdraw,
}

/// How the root moves during a beat.
#[derive(Clone, Copy, Debug)]
pub enum Root {
    /// where it is
    Still,
    /// as the clip moves it (its path at the creature's size, times a gain)
    Clip(f64),
    /// toward a point on the floor at a speed (m/s), the clip played to match
    Steer { to: (f64, f64), speed: f64 },
}

#[derive(Clone, Copy, Debug)]
pub struct Beat {
    pub mode: Mode,
    pub clip: &'static str,
    /// where in the clip the beat starts, s
    pub from: f64,
    pub secs: f64,
    pub root: Root,
}

/// The scene: in its dark corner, facing the wall; your light; its head,
/// then its body; it searches toward where you were, listens, sees you,
/// comes, coils, lunges, stands over you, and goes back into the dark.
pub const SCENE: &[Beat] = &[
    Beat { mode: Mode::Idle, clip: "idle", from: 0.0, secs: 5.0, root: Root::Still },
    Beat { mode: Mode::Alert, clip: "look", from: 0.0, secs: 1.6, root: Root::Still },
    Beat { mode: Mode::Alert, clip: "turn", from: 0.0, secs: 2.2, root: Root::Clip(1.0) },
    Beat { mode: Mode::Search, clip: "creep", from: 0.0, secs: 5.5, root: Root::Steer { to: (0.2, -0.8), speed: 0.75 } },
    Beat { mode: Mode::Listen, clip: "listen", from: 0.0, secs: 2.6, root: Root::Still },
    Beat { mode: Mode::Hunt, clip: "stalk", from: 0.0, secs: 2.0, root: Root::Steer { to: (0.0, 2.6), speed: 2.2 } },
    Beat { mode: Mode::Hunt, clip: "run", from: 0.0, secs: 1.2, root: Root::Steer { to: (0.0, 5.0), speed: 4.5 } },
    Beat { mode: Mode::Strike, clip: "lunge", from: 0.8, secs: 2.2, root: Root::Clip(1.0) },
    Beat { mode: Mode::Over, clip: "stand_over", from: 0.0, secs: 3.0, root: Root::Still },
    Beat { mode: Mode::Withdraw, clip: "back_away", from: 0.0, secs: 5.5, root: Root::Clip(1.0) },
];

pub const START: (f64, f64, f64) = (-1.6, -3.4, -2.4);

/// A clip playing: which, and where in it.
#[derive(Clone, Copy, Debug)]
struct Play {
    clip: usize,
    t: f64,
}

/// A foot held to the floor (world), and how far it has eased back to free.
#[derive(Clone, Copy, Debug, Default)]
struct Lock {
    at: Option<Vec3>,
    /// a step after a release: where it was, the time left, its length
    release: Option<(Vec3, f64, f64)>,
    /// where the clip put the foot last frame (world)
    prev: Option<Vec3>,
}

pub struct Smiler {
    pub rig: BodyRig,
    pub rt: Retarget,
    pub pos: Vec3,
    pub yaw: f64,
    /// how much of the wrongness is laid on (0: the mocap as it is, on its body)
    pub wrong: f64,
    pub beat: usize,
    pub beat_t: f64,
    pub time: f64,
    cur: Play,
    prev: Option<(Play, f64)>,
    head: (f64, f64),
    hold: Option<(f64, f64)>,
    next_hold: f64,
    locks: [Lock; 2],
    rng: Rng,
    /// the finger bones: (bone, side 0 left 1 right, finger 1..5, joint 1..3)
    fingers: Vec<(usize, usize, usize, usize)>,
    lock_gap: [f64; 2],
    /// the current blend's length, s
    fade: f64,
}

const FADE: f64 = 0.3;

fn body_to_world(pos: Vec3, yaw: f64, p: Vec3) -> Vec3 {
    let (s, c) = yaw.dsin_cos();
    v3(pos.x + p.x * c + p.z * s, pos.y + p.y, pos.z - p.x * s + p.z * c)
}

fn world_to_body(pos: Vec3, yaw: f64, p: Vec3) -> Vec3 {
    let (s, c) = yaw.dsin_cos();
    let (dx, dz) = (p.x - pos.x, p.z - pos.z);
    v3(dx * c - dz * s, p.y - pos.y, dx * s + dz * c)
}

fn wrap(a: f64) -> f64 {
    (a + PI).rem_euclid(2.0 * PI) - PI
}

impl Smiler {
    pub fn new(lib: &Library) -> Option<Smiler> {
        let rig = build_body("smiler", &game::entities::smiler::material)?;
        let rt = Retarget::new(&rig, lib)?;
        let clip = lib.clips.iter().position(|c| c.name == SCENE[0].clip).unwrap_or(0);
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
        let s = Smiler {
            rig,
            rt,
            pos: v3(START.0, 0.0, START.1),
            yaw: START.2,
            wrong: 1.0,
            beat: 0,
            beat_t: 0.0,
            time: 0.0,
            cur: Play { clip, t: 0.0 },
            prev: None,
            head: (0.0, 0.0),
            hold: None,
            next_hold: 2.5,
            locks: [Lock::default(); 2],
            rng: Rng::new(0x534d_494c),
            fingers,
            lock_gap: [0.0; 2],
            fade: FADE,
        };
        s.place();
        Some(s)
    }

    /// The feet's locks, for a trace: (locked, how far the lock is from the
    /// clip's foot), per side.
    pub fn locks(&self) -> [(bool, f64); 2] {
        [0, 1].map(|s| (self.locks[s].at.is_some(), self.lock_gap[s]))
    }

    pub fn mode(&self) -> Mode {
        SCENE[self.beat].mode
    }

    /// Back to the scene's start.
    pub fn restart(&mut self, lib: &Library) {
        self.pos = v3(START.0, 0.0, START.1);
        self.yaw = START.2;
        self.beat = 0;
        self.beat_t = 0.0;
        self.time = 0.0;
        self.cur = Play { clip: lib.clips.iter().position(|c| c.name == SCENE[0].clip).unwrap_or(0), t: 0.0 };
        self.prev = None;
        self.head = (0.0, 0.0);
        self.hold = None;
        self.next_hold = 2.5;
        self.locks = [Lock::default(); 2];
        self.rng = Rng::new(0x534d_494c);
        self.place();
    }

    fn place(&self) {
        self.rig.root.set_pos(self.pos.x, self.pos.y, self.pos.z);
        self.rig.root.rotation().y = self.yaw;
        self.rig.root.update_matrix_world(true);
    }

    fn switch(&mut self, lib: &Library, beat: Beat) {
        let Some(c) = lib.clips.iter().position(|c| c.name == beat.clip) else { return };
        if c != self.cur.clip {
            // out of a strike into stillness slowly: no pop
            let fade = if beat.mode == Mode::Over { 0.7 } else { FADE };
            self.prev = Some((self.cur, fade));
            self.fade = fade;
            self.cur = Play { clip: c, t: beat.from };
        }
    }

    /// One step of `dt` seconds; `eye` is the viewer's eye (world), what it
    /// looks at.
    pub fn update(&mut self, lib: &Library, dt: f64, eye: Vec3) {
        self.time += dt;
        self.beat_t += dt;
        if self.beat_t >= SCENE[self.beat].secs {
            self.beat_t = 0.0;
            self.beat = (self.beat + 1) % SCENE.len();
            if self.beat == 0 {
                self.pos = v3(START.0, 0.0, START.1);
                self.yaw = START.2;
                self.locks = [Lock::default(); 2];
            }
            self.switch(lib, SCENE[self.beat]);
        }
        let beat = SCENE[self.beat];
        let k = self.rt.k;
        let clip = &lib.clips[self.cur.clip];
        // ---- held frames: searching (and now and then at rest) it stops dead
        // and lurches on to where it would have been
        let wrong = self.wrong;
        let held = matches!(self.hold, Some((left, _)) if left > 0.0);
        let can_hold = matches!(beat.mode, Mode::Search | Mode::Idle) && wrong > 0.0;
        if !held && can_hold && self.time >= self.next_hold {
            let len = self.rng.range(0.35, 0.8);
            self.hold = Some((len, len));
            let gap = if beat.mode == Mode::Search { self.rng.range(1.2, 2.4) } else { self.rng.range(3.0, 5.0) };
            self.next_hold = self.time + len + gap;
        }
        let mut rate = 1.0;
        let mut root_dt = dt;
        if let Some((left, len)) = self.hold {
            if left > 0.0 {
                // held: neither the pose nor the root moves
                self.hold = Some((left - dt, len));
                rate = 0.0;
                root_dt = 0.0;
            } else if left > -0.08 {
                // the snap: what it missed, in 80 ms
                rate = 1.0 + len / 0.08;
                root_dt = dt * rate;
                self.hold = Some((left - dt, len));
            } else {
                self.hold = None;
            }
        }
        // ---- the root
        match beat.root {
            Root::Still => {}
            Root::Clip(gain) => {
                let (vx, vz, vyaw) = clip.root_velocity(self.cur.t, lib.fps);
                let d = body_to_world(v3(0, 0, 0), self.yaw, v3(vx * k * gain * root_dt, 0.0, vz * k * gain * root_dt));
                self.pos = v3(self.pos.x + d.x, 0.0, self.pos.z + d.z);
                self.yaw += vyaw * root_dt;
            }
            Root::Steer { to, speed } => {
                let (dx, dz) = (to.0 - self.pos.x, to.1 - self.pos.z);
                let dist = dx.dhypot(dz);
                let want = dx.datan2(dz);
                // the body comes round after the head: a slow turn
                let turn = wrap(want - self.yaw).clamp(-2.4 * root_dt, 2.4 * root_dt);
                self.yaw += turn;
                let step = (speed * root_dt).min(dist);
                self.pos = v3(self.pos.x + self.yaw.dsin() * step, 0.0, self.pos.z + self.yaw.dcos() * step);
                // the clip played to the speed (on its own scale)
                if clip.speed > 0.05 {
                    rate *= (speed / (clip.speed * k)).clamp(0.6, 1.8);
                }
                // there: the beat is over (a clip never runs on the spot)
                if dist - step < 0.15 {
                    self.beat_t = beat.secs;
                }
            }
        }
        // the clip fading out keeps its own pace
        let prev_rate = if rate > 0.0 { 1.0 } else { 0.0 };
        self.cur.t += dt * rate;
        if let Some((p, left)) = self.prev.as_mut() {
            p.t += dt * prev_rate;
            *left -= dt;
            if *left <= 0.0 {
                self.prev = None;
            }
        }
        self.place();
        // ---- the pose: the current clip, faded in over the last
        let mut fr = clip.sample(self.cur.t, lib.fps);
        if let Some((p, left)) = self.prev {
            let old = lib.clips[p.clip].sample(p.t, lib.fps);
            let w = 1.0 - left / self.fade;
            fr = crate::motion::Frame::lerp(&old, &fr, w * w * (3.0 - 2.0 * w));
        }
        let gains = match beat.mode {
            // stillness: a part of the sway
            Mode::Idle | Mode::Listen => Gains { trunk: 1.0 - 0.6 * wrong, head: 1.0 - 0.5 * wrong, arms: 1.0 - 0.7 * wrong, legs: 1.0 },
            // dead arms while it runs
            Mode::Hunt => Gains { trunk: 1.0, head: 1.0 - 0.6 * wrong, arms: 1.0 - 0.65 * wrong, legs: 1.0 },
            _ => Gains::default(),
        };
        // ---- the feet: held where they touch the floor (when the clip says
        // so, or holds the foot low and still before it says so); let go,
        // a foot steps to where the clip wants it, never slides there
        let mut ankles = [None, None];
        for s in 0..2 {
            let free = body_to_world(self.pos, self.yaw, self.rt.ankle_target(&fr.p, s, gains.legs));
            let floor = self.rt.rest_ankle(s).y;
            // under its hip: a lock further from here than the leg can
            // comfortably reach is stepped from, never knelt to
            let under = body_to_world(self.pos, self.yaw, self.rt.rest_ankle(s));
            let reach = 0.7 * self.rt.leg_len(&self.rig, s);
            let lock = &mut self.locks[s];
            let still = lock.prev.is_some_and(|p| (p.x - free.x).dhypot(p.z - free.z) < 0.35 * dt.max(1e-4));
            lock.prev = Some(free);
            let touching = fr.contact[s * 2].max(fr.contact[s * 2 + 1]) > 0.5 || (free.y < floor + 0.02 && still);
            let span = |a: Vec3, b: Vec3| ((a.x - b.x).dhypot(a.z - b.z) / 2.5).clamp(0.1, 0.25);
            match (touching, lock.at) {
                (true, None) => lock.at = Some(free),
                (true, Some(at)) if (at.x - free.x).dhypot(at.z - free.z) > 0.45 * k || (at.x - under.x).dhypot(at.z - under.z) > reach => {
                    // stretched too far (a turn on the spot, a change of clip): a step
                    let d = span(at, free);
                    lock.release = Some((at, d, d));
                    lock.at = Some(free);
                }
                (false, Some(at)) => {
                    let d = span(at, free);
                    lock.release = Some((at, d, d));
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
            self.lock_gap[s] = (target.x - free.x).dhypot(target.z - free.z);
            ankles[s] = Some(world_to_body(self.pos, self.yaw, target));
        }
        self.rt.pose(&self.rig, &fr.p, &gains, ankles);
        self.layer(beat, dt, eye);
        self.rig.sync();
    }

    /// What is laid on the mocap: the head on its target in snaps, the slow
    /// breath, the jaw.
    fn layer(&mut self, beat: Beat, dt: f64, eye: Vec3) {
        let m = &self.rt.map;
        let rig = &self.rig;
        let wrong = self.wrong;
        // where the eye is, from the head
        let head_at = rig.head(m.head);
        let local = world_to_body(self.pos, self.yaw, eye);
        let to = v3(local.x - head_at.x, local.y - head_at.y - 0.15, local.z - head_at.z);
        let want_yaw = to.x.datan2(to.z).clamp(-1.5, 1.5);
        let want_pitch = (-to.y).datan2(to.x.dhypot(to.z)).clamp(-0.9, 0.7);
        let watching =
            matches!(beat.mode, Mode::Alert | Mode::Hunt | Mode::Strike | Mode::Over) || (beat.mode == Mode::Listen && self.beat_t > 1.6);
        let (wy, wp) = if watching { (want_yaw, want_pitch) } else { (0.0, 0.0) };
        // it holds its head until the error is large, then snaps to it
        let (hy, hp) = self.head;
        let err = (wy - hy).dhypot(wp - hp);
        let rate = if err > 0.25 * wrong.max(0.05) { 22.0 } else { 0.0 };
        let a = 1.0 - (-rate * dt).dexp();
        self.head = (hy + (wy - hy) * a, hp + (wp - hp) * a);
        let (hy, hp) = (self.head.0 * wrong, self.head.1 * wrong);
        let turn = |share: f64, roll: f64| Quat::from_euler(&Euler::new(hp * share, hy * share, roll));
        let tilt = if beat.mode == Mode::Listen {
            0.38 * wrong * (self.beat_t / 0.6).min(1.0)
        } else if beat.mode == Mode::Over {
            0.3 * wrong
        } else {
            0.0
        };
        // the breath, slow (the neck and head ride on it)
        if let Some(ch) = m.upperchest {
            let breath = Quat::from_euler(&Euler::new((self.time * 1.4).dsin() * 0.02 * wrong, 0.0, 0.0));
            let q: Quat = *rig.bones[ch].quaternion();
            rig.bones[ch].set_quat(&qmul(&q, &breath));
        }
        // then the head on what it watches, the neck taking a part of the turn
        let bn = rig.body_rot(m.neck);
        let bh = rig.body_rot(m.head);
        set_global(rig, m.neck, &qmul(&turn(0.4, 0.0), &bn));
        set_global(rig, m.head, &qmul(&turn(1.0, tilt), &bh));
        // the fingers (the mocap has none): loose at rest, clenching as it
        // coils, splayed past straight as it reaches, twitching one by one
        // while it stands over you
        let strike_t = if beat.mode == Mode::Strike { self.beat_t } else { 0.0 };
        let curl = match beat.mode {
            Mode::Strike if strike_t < 0.5 => 0.25 + 0.5 * (strike_t / 0.5),
            Mode::Strike => -0.18,
            Mode::Hunt => 0.12,
            Mode::Over => 0.35,
            _ => 0.28,
        };
        for &(b, side, f, k) in &self.fingers {
            let twitch = if beat.mode == Mode::Over { 0.35 * ((self.time * 2.3 + f as f64 * 1.7).dsin()).max(0.0).dpowi(3) * wrong } else { 0.0 };
            let a = (curl + twitch) * if k == 1 { 0.7 } else { 1.0 } * if f == 1 { 0.5 } else { 1.0 };
            let axis = if side == 0 { v3(0.0, 0.0, -1.0) } else { v3(0.0, 0.0, 1.0) };
            rig.bones[b].set_quat(&Quat::from_axis_angle(&axis, a));
        }
        // the jaw a little open as it comes, wide in the lunge
        if let Some(jw) = m.jaw {
            let open = match beat.mode {
                Mode::Hunt => 0.12,
                Mode::Strike => 0.32,
                Mode::Over => 0.25,
                _ => 0.0,
            } * wrong;
            rig.bones[jw].set_quat(&slerp(&Quat::IDENTITY, &Quat::from_euler(&Euler::new(1.0, 0.0, 0.0)), open));
        }
    }
}

fn set_global(rig: &BodyRig, i: usize, g: &Quat) {
    let parent = rig.def.bones[i].parent.map_or(Quat::IDENTITY, |p| rig.body_rot(p));
    let mut inv = parent;
    inv.invert();
    rig.bones[i].set_quat(&qmul(&inv, g));
}
