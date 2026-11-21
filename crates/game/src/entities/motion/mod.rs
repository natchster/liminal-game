//! The creatures' motion: clips of a real body's movement, made from briefs
//! by an open motion model in the lab (`lab/motion/`: NVIDIA Kimodo on MLX,
//! `docs/research/character-motion.md`), retargeted onto each creature's own
//! body as it moves, and what makes the motion the creature's laid on top
//! by its kind (`smiler::pose`, `hound::pose`, ...; `docs/design/ENTITIES.md` §6).
//!
//! - [`Library`]: a creature's clips (`assets/motion/<kind>.motion`, embedded):
//!   each frame NVIDIA's 30 SOMA joints in the creature's frame (+Y up, +Z
//!   ahead, its left at +X, the root at the origin), the root's path, and
//!   six contacts (each foot's heel and toe, each hand); and its bridges
//!   ([`Bridge`]: the motion from one clip into another, made by NVIDIA
//!   MotionBricks in the lab, `lab/motion/bricks.py`).
//! - [`retarget::Retarget`]: a frame laid on a body as each segment's turn away from
//!   the library's neutral stance (so a body bound in a stoop keeps it), the
//!   feet (and on all fours the hands) solved onto where they stand.
//! - [`anim::Anim`]: a creature's motion as it plays: the clip its kind picks,
//!   blended from the last, played to the speed it moves at, held and
//!   snapped, its feet locked to the floor while they bear weight.
//!
//! Nothing here draws from the game's streams or reads a clock: a pose is a
//! function of the creature's state and its own seeded stream (invariant 1).
pub mod anim;
pub mod retarget;

use crate::entities::body::BodyRig;
use crate::entities::brain::Mode;
use crate::prelude::*;
use anim::{Anim, Drive};
use std::sync::OnceLock;

/// What the creature's mind is on, as its body shows it.
#[derive(Clone, Copy, Debug)]
pub struct Mind {
    pub mode: Mode,
    /// seconds in it
    pub mode_t: f64,
    /// it has you: it stands over your body
    pub killer: bool,
    /// your eyes (world): what it watches
    pub eye: Vec3,
    /// where it last heard something (world, about a body's height over the
    /// floor): only a rough guess, as its rule has it, never your eyes
    pub heard: Option<Vec3>,
    /// how long it has chased you, s
    pub chase_t: f64,
}

impl Mind {
    pub fn new(mode: Mode, mode_t: f64, eye: Vec3) -> Mind {
        Mind { mode, mode_t, killer: false, eye, heard: None, chase_t: 0.0 }
    }
}

/// How high over the floor a heard sound is taken to be, m (a body's middle:
/// what it heard is a step, a breath, a thing knocked).
pub const HEARD_Y: f64 = 0.7;

/// Creature `ei`'s motion this frame (made the first time: its kind's
/// clips, `fours` if its hands are feet, starting on `first`), handed to
/// `f` with its body, what drives it and its mind. Its own stream is seeded
/// by where it first stood: nothing is drawn from the game's.
pub fn with_anim(g: &mut Game, ei: usize, fours: bool, first: &'static str, dt: f64, f: impl FnOnce(&mut Anim, &BodyRig, &Drive, &Mind)) {
    let e = &g.entities[ei];
    // (as its body is turned: a killer is turned to you)
    let yaw = e.g.rotation().y;
    let pos = e.pos();
    let kind = e.kind.clone();
    let killer = g.director.killer == Some(ei) && g.state != "playing";
    let eye = g.camera.pos();
    let Some(c) = g.creature_mut(ei) else { return };
    let heard = c.brain.mem.heard.map(|(x, z, _)| v3(x, HEARD_Y, z));
    let mind = Mind { mode: c.brain.mode, mode_t: c.brain.mode_t, killer, eye, heard, chase_t: c.chase_t };
    let rig = c.body.clone();
    if c.anim.is_none() {
        let Some(lib) = library(&kind) else { return };
        let seed = pos.x.to_bits()
            ^ pos.z.to_bits().rotate_left(29)
            ^ kind.bytes().fold(0xcbf2_9ce4_8422_2325u64, |h, b| (h ^ b as u64).wrapping_mul(0x100_0000_01b3));
        c.anim = Anim::new(&rig, lib, fours, first, seed).map(Box::new);
    }
    let Some(a) = c.anim.as_deref_mut() else { return };
    let d = a.drive(pos, yaw, dt);
    f(a, &rig, &d, &mind);
}

/// SOMA's joints, as the clips keep them.
pub const N: usize = 30;

/// The SOMA joints the retarget reads (indices into a frame's joints).
pub mod j {
    pub const HIPS: usize = 0;
    pub const SPINE2: usize = 2;
    pub const NECK1: usize = 4;
    pub const HEAD: usize = 6;
    pub const EYE_L: usize = 8;
    pub const EYE_R: usize = 9;
    /// per side: shoulder, arm, forearm, hand, thumb end, middle end
    pub const ARM: [[usize; 6]; 2] = [[10, 11, 12, 13, 14, 15], [16, 17, 18, 19, 20, 21]];
    /// per side: leg (hip), shin (knee), foot (ankle), toe base
    pub const LEG: [[usize; 4]; 2] = [[22, 23, 24, 25], [26, 27, 28, 29]];
}

/// The contacts: left heel, left toe, right heel, right toe, left hand, right hand.
pub const CONTACTS: usize = 6;

#[derive(Clone, Debug)]
pub struct Frame {
    pub p: Vec<Vec3>,
    /// the root's path from the clip's start: x, z, yaw
    pub root: [f64; 3],
    /// 0..1, as [`CONTACTS`]
    pub contact: [f64; CONTACTS],
}

impl Frame {
    /// `a` toward `b` by `t` (positions and contacts; the root by its parts).
    pub fn lerp(a: &Frame, b: &Frame, t: f64) -> Frame {
        let p = a.p.iter().zip(&b.p).map(|(x, y)| v3(x.x + (y.x - x.x) * t, x.y + (y.y - x.y) * t, x.z + (y.z - x.z) * t)).collect();
        let mut root = [0.0; 3];
        let mut contact = [0.0; CONTACTS];
        for k in 0..3 {
            root[k] = a.root[k] + (b.root[k] - a.root[k]) * t;
        }
        for k in 0..CONTACTS {
            contact[k] = a.contact[k] + (b.contact[k] - a.contact[k]) * t;
        }
        Frame { p, root, contact }
    }

    /// Foot `s` touches the floor (its heel or its toe).
    pub fn foot_down(&self, s: usize) -> bool {
        self.contact[s * 2].max(self.contact[s * 2 + 1]) > 0.5
    }

    /// Hand `s` is on the floor.
    pub fn hand_down(&self, s: usize) -> bool {
        self.contact[4 + s] > 0.5
    }
}

#[derive(Clone, Debug)]
pub struct Clip {
    pub name: String,
    /// the brief, in its own words
    pub prompt: String,
    pub looped: bool,
    /// the root's mean speed as it plays, on the mocap's scale (m/s)
    pub speed: f64,
    pub frames: Vec<Frame>,
    /// a bridge: the way from one clip into another
    pub bridge: Option<Bridge>,
}

/// The motion from one clip into another (`lab/motion/bricks.py`: NVIDIA
/// MotionBricks, given the four frames of `from` that lead to `from_t` and
/// the four of `to` from `to_t`): its first frame is `from`'s at `from_t`,
/// its last `to`'s at `to_t`. A body going from the one clip to the other
/// plays it (`anim::Anim`), where it blended the two.
#[derive(Clone, Debug)]
pub struct Bridge {
    pub from: String,
    pub to: String,
    /// where in `from` it leaves, s
    pub from_t: f64,
    /// where in `to` it lands, s
    pub to_t: f64,
}

#[derive(Debug)]
pub struct Library {
    pub fps: f64,
    /// the stance every clip is measured from: as the body is bound
    pub neutral: Vec<Vec3>,
    pub clips: Vec<Clip>,
}

const MAGIC: &[u8; 8] = b"OCMOTIO2";

impl Library {
    /// A library from its bytes (`lab/motion/motion.py bake`).
    pub fn parse(bytes: &[u8]) -> Result<Library, String> {
        if bytes.len() < 12 || &bytes[..8] != MAGIC {
            return Err("not a motion library (OCMOTIO2)".into());
        }
        let hl = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]) as usize;
        let header: serde_json::Value = serde_json::from_slice(&bytes[12..12 + hl]).map_err(|e| e.to_string())?;
        let data = &bytes[12 + hl..];
        let fps = header["fps"].as_f64().unwrap_or(30.0);
        let xyz = |p: &serde_json::Value| v3(p[0].as_f64().unwrap_or(0.0), p[1].as_f64().unwrap_or(0.0), p[2].as_f64().unwrap_or(0.0));
        let neutral: Vec<Vec3> = header["neutral"].as_array().ok_or("no neutral pose")?.iter().map(xyz).collect();
        if neutral.len() != N {
            return Err(format!("a neutral of {} joints", neutral.len()));
        }
        let contacts = header["contacts"].as_array().map_or(4, |c| c.len());
        let width = N * 3 + 3 + contacts;
        // the bridges, deflated (zlib) after the clips: each value's track
        // over its frames in 16-bit steps from the frame before (`motion.py bake`)
        let block = match header["bridges"].as_object() {
            Some(b) => {
                let at = |k: &str| b.get(k).and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                let (o, len) = (at("offset"), at("length"));
                let z = data.get(o..o + len).ok_or("the bridges run past the data")?;
                miniz_oxide::inflate::decompress_to_vec_zlib_with_limit(z, at("size").max(1))
                    .map_err(|e| format!("the bridges: {:?}", e.status))?
            }
            None => vec![],
        };
        let mut clips = vec![];
        for c in header["clips"].as_array().ok_or("no clips")? {
            let n = c["frames"].as_u64().unwrap_or(0) as usize;
            if n < 2 {
                return Err(format!("clip {} has {n} frames (a clip plays two at least)", c["name"]));
            }
            let offset = c["offset"].as_u64().unwrap_or(0) as usize;
            // its values, frame by frame: float32, or (a bridge) summed from its steps
            let vals: Vec<f64> = match c["scale"].as_f64() {
                Some(scale) => {
                    if offset + n * width * 2 > block.len() {
                        return Err(format!("bridge {} runs past the bridges", c["name"]));
                    }
                    let mut v = vec![0.0; n * width];
                    for j in 0..width {
                        let mut acc = 0i32;
                        for k in 0..n {
                            let b = offset + (j * n + k) * 2;
                            acc += i16::from_le_bytes([block[b], block[b + 1]]) as i32;
                            v[k * width + j] = acc as f64 / scale;
                        }
                    }
                    v
                }
                None => {
                    if offset + n * width * 4 > data.len() {
                        return Err(format!("clip {} runs past the data", c["name"]));
                    }
                    let d = &data[offset..offset + n * width * 4];
                    d.as_chunks::<4>().0.iter().map(|b| f32::from_le_bytes(*b) as f64).collect()
                }
            };
            let f = |i: usize| vals[i];
            let mut frames = Vec::with_capacity(n);
            for k in 0..n {
                let b = k * width;
                let p = (0..N).map(|i| v3(f(b + i * 3), f(b + i * 3 + 1), f(b + i * 3 + 2))).collect();
                let r = b + N * 3;
                let mut contact = [0.0; CONTACTS];
                for (q, c) in contact.iter_mut().enumerate().take(contacts.min(CONTACTS)) {
                    *c = f(r + 3 + q);
                }
                frames.push(Frame { p, root: [f(r), f(r + 1), f(r + 2)], contact });
            }
            let bridge = c["bridge"].as_object().map(|b| Bridge {
                from: b.get("from").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                to: b.get("to").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                from_t: b.get("from_t").and_then(|v| v.as_f64()).unwrap_or(0.0),
                to_t: b.get("to_t").and_then(|v| v.as_f64()).unwrap_or(0.0),
            });
            let mut clip = Clip {
                name: c["name"].as_str().unwrap_or("?").to_string(),
                prompt: c["prompt"].as_str().unwrap_or("").to_string(),
                looped: c["loop"].as_bool().unwrap_or(false),
                speed: 0.0,
                frames,
                bridge,
            };
            // its speed as it plays (a loop's, not the whole take's with its
            // start from standing): the root's path, frame to frame
            let n = clip.frames.len();
            let path: f64 = (1..n)
                .map(|k| (clip.frames[k].root[0] - clip.frames[k - 1].root[0]).dhypot(clip.frames[k].root[1] - clip.frames[k - 1].root[1]))
                .sum();
            clip.speed = if n > 1 { path * fps / (n - 1) as f64 } else { 0.0 };
            clips.push(clip);
        }
        Ok(Library { fps, neutral, clips })
    }

    /// Clip `name`'s index.
    pub fn clip(&self, name: &str) -> Option<usize> {
        self.clips.iter().position(|c| c.name == name)
    }

    /// The way from clip `from` (now at `t` s) into `to` at `to_t` s (or
    /// up to [`BRIDGE_LANDS`] later), if a bridge was made for it: going
    /// on (`wait`: it still moves, and `from` goes on with it), the next to
    /// leave `from` (round a loop) if one leaves within [`BRIDGE_WAIT`] (it
    /// starts on the very frame it was made from); else the nearest within
    /// [`BRIDGE_REACH`] (now, blended in from where `from` is: a body that
    /// has stopped does not walk on under it while it waits).
    pub fn bridge(&self, from: &str, t: f64, to: &str, to_t: f64, wait: bool) -> Option<Way> {
        let a = &self.clips[self.clip(from)?];
        let dur = a.duration(self.fps);
        let end = (a.frames.len() - 1) as f64 / self.fps;
        let t = if a.looped { t.rem_euclid(dur) } else { t.clamp(0.0, end) };
        let mine = || {
            self.clips.iter().enumerate().filter_map(|(i, c)| {
                let b = c.bridge.as_ref()?;
                (b.from == from && b.to == to && (to_t - 0.05..=to_t + BRIDGE_LANDS).contains(&b.to_t)).then_some((i, b.from_t))
            })
        };
        // how long until it comes round (a clip played once: never, once past)
        let ahead = |from_t: f64| if a.looped { (from_t - t).rem_euclid(dur) } else { from_t - t };
        let next = mine()
            .map(|(i, f)| (i, ahead(f)))
            .filter(|&(_, w)| (-0.5 / self.fps..=if wait { BRIDGE_WAIT } else { 0.5 / self.fps }).contains(&w))
            .min_by(|x, y| x.1.total_cmp(&y.1));
        if let Some((i, w)) = next {
            return Some(if w <= 0.5 / self.fps { Way::Now(i) } else { Way::Wait(i, w) });
        }
        mine()
            .map(|(i, f)| {
                let d = (f - t).abs();
                (i, if a.looped { d.min(dur - d) } else { d })
            })
            .filter(|&(_, d)| d <= BRIDGE_REACH)
            .min_by(|x, y| x.1.total_cmp(&y.1))
            .map(|(i, _)| Way::Blend(i))
    }
}

/// How a clip goes into the next by a bridge ([`Library::bridge`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Way {
    /// it leaves now, from where it is
    Now(usize),
    /// it leaves when it has played this much more of its clip, s
    Wait(usize, f64),
    /// it leaves now, blended in from where it is
    Blend(usize),
}

/// How much later in a clip than its pick starts it a bridge may land, s:
/// a clip that begins with a person stopping (a listen) is landed on where
/// they have stopped, the bridge being the stop.
pub const BRIDGE_LANDS: f64 = 2.5;
/// The longest a clip goes on before its bridge comes round, s (bridges
/// leave a clip that travels every 0.25 s: a step's time).
pub const BRIDGE_WAIT: f64 = 0.3;
/// How far from where a clip is a bridge may leave it and be blended in, s
/// (over [`anim::BRIDGE_BLEND`]: a clip left every 0.25 s if it travels, every
/// 0.6 s if it stands).
pub const BRIDGE_REACH: f64 = 0.3;

/// The kinds that move by clips, and their libraries' bytes. (The Watcher
/// is never seen to move: its poses are its body's, `watcher::POSES`.)
pub const KINDS: [&str; 5] = ["smiler", "hound", "corner", "reader", "glass"];

macro_rules! motion_bytes {
    ($k:literal) => {
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/motion/", $k, ".motion"))
    };
}

fn bytes(kind: &str) -> Option<&'static [u8]> {
    Some(match kind {
        "smiler" => motion_bytes!("smiler"),
        "hound" => motion_bytes!("hound"),
        "corner" => motion_bytes!("corner"),
        "reader" => motion_bytes!("reader"),
        "glass" => motion_bytes!("glass"),
        _ => return None,
    })
}

/// Kind `kind`'s clips, read once.
pub fn library(kind: &str) -> Option<&'static Library> {
    static LIBS: [OnceLock<Option<Library>>; 5] = [const { OnceLock::new() }; 5];
    let i = KINDS.iter().position(|k| *k == kind)?;
    LIBS[i]
        .get_or_init(|| {
            let lib = Library::parse(bytes(kind)?);
            debug_assert!(lib.is_ok(), "assets/motion/{kind}.motion: {:?}", lib.as_ref().err());
            lib.ok()
        })
        .as_ref()
}

impl Clip {
    pub fn duration(&self, fps: f64) -> f64 {
        self.frames.len() as f64 / fps
    }

    fn index(&self, t: f64, fps: f64) -> (usize, usize, f64) {
        let n = self.frames.len();
        let f = t * fps;
        if self.looped {
            let f = f.rem_euclid(n as f64);
            let i = f.floor() as usize % n;
            (i, (i + 1) % n, f - f.floor())
        } else {
            let f = f.clamp(0.0, (n - 1) as f64);
            let i = f.floor() as usize;
            (i, (i + 1).min(n - 1), f - f.floor())
        }
    }

    /// How fast its root goes about `t` s (over ±0.1 s), m/s on the mocap's
    /// scale: a bridge's speed changes along it.
    pub fn speed_at(&self, t: f64, fps: f64) -> f64 {
        let n = self.frames.len();
        let k = ((t * fps).round().max(0.0) as usize).min(n - 1);
        let (a, b) = (k.saturating_sub(3), (k + 3).min(n - 1));
        if b <= a {
            return self.speed;
        }
        let (p, q) = (&self.frames[a].root, &self.frames[b].root);
        (q[0] - p[0]).dhypot(q[1] - p[1]) * fps / (b - a) as f64
    }

    /// When its hips are lowest, s (a stoop's bottom, a crouch's).
    pub fn lowest(&self, fps: f64) -> f64 {
        let mut best = (0, f64::MAX);
        for (k, f) in self.frames.iter().enumerate() {
            if f.p[j::HIPS].y < best.1 {
                best = (k, f.p[j::HIPS].y);
            }
        }
        best.0 as f64 / fps
    }

    /// The pose at `t` seconds (looped, or held on its last frame).
    pub fn sample(&self, t: f64, fps: f64) -> Frame {
        let (a, b, w) = self.index(t, fps);
        let mut fr = Frame::lerp(&self.frames[a], &self.frames[b], w);
        // the root across a loop's seam: carry on from the last step
        if self.looped && b < a {
            let n = self.frames.len();
            let (l, m) = (&self.frames[n - 2].root, &self.frames[n - 1].root);
            for k in 0..3 {
                fr.root[k] = m[k] + (m[k] - l[k]) * w;
            }
        }
        fr
    }
}

// ---------------------------------------------------------------- helpers

pub(crate) fn sub(a: Vec3, b: Vec3) -> Vec3 {
    v3(a.x - b.x, a.y - b.y, a.z - b.z)
}
pub(crate) fn add(a: Vec3, b: Vec3) -> Vec3 {
    v3(a.x + b.x, a.y + b.y, a.z + b.z)
}
pub(crate) fn scale(a: Vec3, k: f64) -> Vec3 {
    v3(a.x * k, a.y * k, a.z * k)
}
pub(crate) fn mix(a: Vec3, b: Vec3, t: f64) -> Vec3 {
    v3(a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t, a.z + (b.z - a.z) * t)
}
pub(crate) fn mid_of(a: Vec3, b: Vec3) -> Vec3 {
    scale(add(a, b), 0.5)
}
pub(crate) fn unit(a: Vec3) -> Vec3 {
    let l = a.length();
    if l < 1e-9 { v3(0, 1, 0) } else { scale(a, 1.0 / l) }
}

/// A point of the body's frame in the world (its root at `pos`, turned `yaw`).
pub fn body_to_world(pos: Vec3, yaw: f64, p: Vec3) -> Vec3 {
    let (s, c) = yaw.dsin_cos();
    v3(pos.x + p.x * c + p.z * s, pos.y + p.y, pos.z - p.x * s + p.z * c)
}

/// A point of the world in the body's frame.
pub fn world_to_body(pos: Vec3, yaw: f64, p: Vec3) -> Vec3 {
    let (s, c) = yaw.dsin_cos();
    let (dx, dz) = (p.x - pos.x, p.z - pos.z);
    v3(dx * c - dz * s, p.y - pos.y, dx * s + dz * c)
}
