//! Motion clips (`lab/motion/clips/*.motion`, made by `lab/motion/motion.py`)
//! and their retargeting onto a body of `bodies.bin`.
//!
//! A clip is mocap-like motion on NVIDIA's 30-joint SOMA skeleton: each
//! frame the joints' positions in the creature's own frame (+Y up, +Z
//! ahead, its left at +X, the root at the origin: the game's body frame),
//! the root's path, and the foot contacts.
//!
//! **Retargeting by deltas.** Every segment of the mocap gets a frame built
//! from joint positions alone (a direction and a side: hip to hip, shoulder
//! to shoulder, eye to eye, thumb to finger), so no rotation convention of
//! the source matters. Its rotation away from the mocap's neutral stance
//! (`neutral`, the idle clip's most typical frame) is laid on the creature's
//! own rest pose: a body bound in a stoop keeps its stoop and gets the
//! mocap's weight, sway and swing on top of it. The trunk and head take whole
//! frames (twist included); limbs take the least turn of their direction;
//! legs are then solved onto their feet (two bones and a pole), the feet put
//! where the mocap's are, scaled by the creature's legs over the mocap's.
use engine::prelude::*;
use game::entities::body::{BodyRig, from_to, qinv, qmul, two_bone};

pub const N: usize = 30;

/// The SOMA joints the retarget reads (indices into a frame's joints).
#[allow(dead_code, reason = "the whole skeleton is named, the retarget reads most of it")]
pub mod j {
    pub const HIPS: usize = 0;
    pub const SPINE1: usize = 1;
    pub const SPINE2: usize = 2;
    pub const CHEST: usize = 3;
    pub const NECK1: usize = 4;
    pub const NECK2: usize = 5;
    pub const HEAD: usize = 6;
    pub const JAW: usize = 7;
    pub const EYE_L: usize = 8;
    pub const EYE_R: usize = 9;
    /// per side: shoulder, arm, forearm, hand, thumb end, middle end
    pub const ARM: [[usize; 6]; 2] = [[10, 11, 12, 13, 14, 15], [16, 17, 18, 19, 20, 21]];
    /// per side: leg (hip), shin (knee), foot (ankle), toe base
    pub const LEG: [[usize; 4]; 2] = [[22, 23, 24, 25], [26, 27, 28, 29]];
}

#[derive(Clone, Debug)]
pub struct Frame {
    pub p: Vec<Vec3>,
    /// the root's path from the clip's start: x, z, yaw
    pub root: [f64; 3],
    /// left heel, left toe, right heel, right toe (0..1)
    pub contact: [f64; 4],
}

impl Frame {
    /// `a` toward `b` by `t` (positions and contacts; the root by its parts).
    pub fn lerp(a: &Frame, b: &Frame, t: f64) -> Frame {
        let p = a.p.iter().zip(&b.p).map(|(x, y)| v3(x.x + (y.x - x.x) * t, x.y + (y.y - x.y) * t, x.z + (y.z - x.z) * t)).collect();
        let mut root = [0.0; 3];
        let mut contact = [0.0; 4];
        for k in 0..3 {
            root[k] = a.root[k] + (b.root[k] - a.root[k]) * t;
        }
        for k in 0..4 {
            contact[k] = a.contact[k] + (b.contact[k] - a.contact[k]) * t;
        }
        Frame { p, root, contact }
    }
}

#[derive(Clone, Debug)]
pub struct Clip {
    pub name: String,
    pub prompt: String,
    pub looped: bool,
    /// the root's mean speed on the mocap's scale (m/s)
    pub speed: f64,
    pub frames: Vec<Frame>,
}

pub struct Library {
    pub fps: f64,
    pub neutral: Vec<Vec3>,
    pub clips: Vec<Clip>,
}

impl Library {
    pub fn load(path: &str) -> Result<Library, String> {
        let bytes = std::fs::read(path).map_err(|e| format!("{path}: {e}"))?;
        if bytes.len() < 12 || &bytes[..8] != b"OCLABMO1" {
            return Err(format!("{path}: not a motion library (OCLABMO1)"));
        }
        let hl = u32::from_le_bytes([bytes[8], bytes[9], bytes[10], bytes[11]]) as usize;
        let header: serde_json::Value = serde_json::from_slice(&bytes[12..12 + hl]).map_err(|e| e.to_string())?;
        let data = &bytes[12 + hl..];
        let f = |i: usize| f32::from_le_bytes([data[i * 4], data[i * 4 + 1], data[i * 4 + 2], data[i * 4 + 3]]) as f64;
        let fps = header["fps"].as_f64().unwrap_or(30.0);
        let neutral = header["neutral"]
            .as_array()
            .ok_or("no neutral pose")?
            .iter()
            .map(|p| v3(p[0].as_f64().unwrap_or(0.0), p[1].as_f64().unwrap_or(0.0), p[2].as_f64().unwrap_or(0.0)))
            .collect();
        let width = N * 3 + 3 + 4;
        let mut clips = vec![];
        for c in header["clips"].as_array().ok_or("no clips")? {
            let n = c["frames"].as_u64().unwrap_or(0) as usize;
            let at = c["offset"].as_u64().unwrap_or(0) as usize / 4;
            let mut frames = Vec::with_capacity(n);
            for k in 0..n {
                let b = at + k * width;
                let p = (0..N).map(|i| v3(f(b + i * 3), f(b + i * 3 + 1), f(b + i * 3 + 2))).collect();
                let r = b + N * 3;
                frames.push(Frame { p, root: [f(r), f(r + 1), f(r + 2)], contact: [f(r + 3), f(r + 4), f(r + 5), f(r + 6)] });
            }
            let mut clip = Clip {
                name: c["name"].as_str().unwrap_or("?").to_string(),
                prompt: c["prompt"].as_str().unwrap_or("").to_string(),
                looped: c["loop"].as_bool().unwrap_or(false),
                speed: 0.0,
                frames,
            };
            // its speed as it plays (a loop's, not the whole take's with its
            // start from standing): the root's path, frame to frame
            let n = clip.frames.len();
            let path: f64 = (1..n).map(|k| (clip.frames[k].root[0] - clip.frames[k - 1].root[0]).dhypot(clip.frames[k].root[1] - clip.frames[k - 1].root[1])).sum();
            clip.speed = if n > 1 { path * fps / (n - 1) as f64 } else { 0.0 };
            clips.push(clip);
        }
        Ok(Library { fps, neutral, clips })
    }

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

    /// The root's velocity at `t`, in the body's own frame (m/s on the
    /// mocap's scale, sideways x and ahead z) and its turn (rad/s).
    pub fn root_velocity(&self, t: f64, fps: f64) -> (f64, f64, f64) {
        let (a, b, _) = self.index(t, fps);
        let n = self.frames.len();
        let (p, q) = if b > a || !self.looped {
            (&self.frames[a].root, &self.frames[b].root)
        } else {
            (&self.frames[n - 2].root, &self.frames[n - 1].root)
        };
        if a == b {
            return (0.0, 0.0, 0.0);
        }
        let (dx, dz, dyaw) = (q[0] - p[0], q[1] - p[1], q[2] - p[2]);
        // the path is in the clip's start frame: turn the step into this frame's
        let (s, c) = p[2].dsin_cos();
        (fps * (dx * c - dz * s), fps * (dx * s + dz * c), fps * dyaw)
    }
}

// ---------------------------------------------------------------- frames

fn sub(a: Vec3, b: Vec3) -> Vec3 {
    v3(a.x - b.x, a.y - b.y, a.z - b.z)
}
fn add(a: Vec3, b: Vec3) -> Vec3 {
    v3(a.x + b.x, a.y + b.y, a.z + b.z)
}
fn scale(a: Vec3, k: f64) -> Vec3 {
    v3(a.x * k, a.y * k, a.z * k)
}
fn unit(a: Vec3) -> Vec3 {
    let l = a.length();
    if l < 1e-9 { v3(0, 1, 0) } else { scale(a, 1.0 / l) }
}
fn cross(a: Vec3, b: Vec3) -> Vec3 {
    v3(a.y * b.z - a.z * b.y, a.z * b.x - a.x * b.z, a.x * b.y - a.y * b.x)
}
fn mid(a: Vec3, b: Vec3) -> Vec3 {
    scale(add(a, b), 0.5)
}

/// The rotation whose columns are (side, up, side × up), from an `up`
/// direction and a rough `side`.
pub fn basis(up: Vec3, side: Vec3) -> Quat {
    let y = unit(up);
    let x = unit(sub(side, scale(y, side.dot(&y))));
    let z = cross(x, y);
    // the matrix [x y z] (columns) to a quaternion
    let (m00, m11, m22) = (x.x, y.y, z.z);
    let (m01, m02, m10, m12, m20, m21) = (y.x, z.x, x.y, z.y, x.z, y.z);
    let tr = m00 + m11 + m22;
    let q = if tr > 0.0 {
        let s = (tr + 1.0).sqrt() * 2.0;
        Quat::new((m21 - m12) / s, (m02 - m20) / s, (m10 - m01) / s, 0.25 * s)
    } else if m00 > m11 && m00 > m22 {
        let s = (1.0 + m00 - m11 - m22).sqrt() * 2.0;
        Quat::new(0.25 * s, (m01 + m10) / s, (m02 + m20) / s, (m21 - m12) / s)
    } else if m11 > m22 {
        let s = (1.0 + m11 - m00 - m22).sqrt() * 2.0;
        Quat::new((m01 + m10) / s, 0.25 * s, (m12 + m21) / s, (m02 - m20) / s)
    } else {
        let s = (1.0 + m22 - m00 - m11).sqrt() * 2.0;
        Quat::new((m02 + m20) / s, (m12 + m21) / s, 0.25 * s, (m10 - m01) / s)
    };
    let mut q = q;
    q.normalize();
    q
}

pub fn slerp(a: &Quat, b: &Quat, t: f64) -> Quat {
    let mut q = *a;
    q.slerp(b, t);
    q
}

/// The mocap's segment frames of one pose: pelvis, chest, neck, head, and
/// each hand.
#[derive(Clone, Copy, Debug)]
pub struct Segments {
    pub pelvis: Quat,
    pub chest: Quat,
    pub neck: Quat,
    pub head: Quat,
    pub hand: [Quat; 2],
}

pub fn segments(p: &[Vec3]) -> Segments {
    let hip_side = sub(p[j::LEG[0][0]], p[j::LEG[1][0]]);
    let chest_side = sub(p[j::ARM[0][1]], p[j::ARM[1][1]]);
    let eye_side = sub(p[j::EYE_L], p[j::EYE_R]);
    let fwd = sub(mid(p[j::EYE_L], p[j::EYE_R]), p[j::HEAD]);
    let pelvis = basis(sub(p[j::SPINE2], p[j::HIPS]), hip_side);
    let chest = basis(sub(p[j::NECK1], p[j::SPINE2]), chest_side);
    let neck = basis(sub(p[j::HEAD], p[j::NECK1]), mid(unit(chest_side), unit(eye_side)));
    let head = basis(cross(fwd, eye_side), eye_side);
    let hand = [0, 1].map(|s| {
        let a = j::ARM[s];
        // the hand's length, and its thumb side (the left hand's thumb
        // points the other way: keep each side's frame right-handed)
        let along = sub(p[a[5]], p[a[3]]);
        let thumb = sub(p[a[4]], p[a[5]]);
        basis(along, if s == 0 { thumb } else { scale(thumb, -1.0) })
    });
    Segments { pelvis, chest, neck, head, hand }
}

/// The limb segments by their joints: (from, to) per side.
fn limb(p: &[Vec3], s: usize, which: usize) -> Vec3 {
    let (a, l) = (j::ARM[s], j::LEG[s]);
    let (from, to) = match which {
        0 => (a[0], a[1]), // clavicle
        1 => (a[1], a[2]), // upper arm
        2 => (a[2], a[3]), // forearm
        3 => (l[0], l[1]), // thigh
        4 => (l[1], l[2]), // shin
        _ => (l[2], l[3]), // foot
    };
    unit(sub(p[to], p[from]))
}

/// The creature's bones the retarget drives.
#[derive(Clone, Debug)]
pub struct BoneMap {
    pub hips: usize,
    pub spine: Option<usize>,
    pub chest: Option<usize>,
    pub upperchest: Option<usize>,
    pub neck: usize,
    pub head: usize,
    pub jaw: Option<usize>,
    /// per side: clavicle, upper arm, forearm, hand
    pub arm: [[usize; 4]; 2],
    /// per side: thigh, shin, foot, toes
    pub leg: [[usize; 4]; 2],
}

impl BoneMap {
    /// The standard humanoid names (`assets/src/creatures/human.py`).
    pub fn humanoid(rig: &BodyRig) -> Option<BoneMap> {
        let i = |n: &str| rig.ix(n);
        let side = |s: &str| -> Option<([usize; 4], [usize; 4])> {
            Some((
                [i(&format!("clavicle{s}"))?, i(&format!("upperarm{s}"))?, i(&format!("forearm{s}"))?, i(&format!("hand{s}"))?],
                [i(&format!("thigh{s}"))?, i(&format!("shin{s}"))?, i(&format!("foot{s}"))?, i(&format!("toes{s}"))?],
            ))
        };
        let (al, ll) = side(".L")?;
        let (ar, lr) = side(".R")?;
        Some(BoneMap {
            hips: i("hips")?,
            spine: i("spine"),
            chest: i("chest"),
            upperchest: i("upperchest"),
            neck: i("neck")?,
            head: i("head")?,
            jaw: i("jaw"),
            arm: [al, ar],
            leg: [ll, lr],
        })
    }
}

/// How much of the mocap each part of the body takes (0 keeps the rest pose).
#[derive(Clone, Copy, Debug)]
pub struct Gains {
    pub trunk: f64,
    pub head: f64,
    pub arms: f64,
    pub legs: f64,
}

impl Default for Gains {
    fn default() -> Gains {
        Gains { trunk: 1.0, head: 1.0, arms: 1.0, legs: 1.0 }
    }
}

/// A body's retarget from one motion library.
pub struct Retarget {
    pub map: BoneMap,
    /// the creature's legs over the mocap's
    pub k: f64,
    neutral: Vec<Vec3>,
    n_seg: Segments,
    n_limb: [[Vec3; 6]; 2],
    rest_hips: Vec3,
    rest_ankle: [Vec3; 2],
}

fn rest_head(rig: &BodyRig, i: usize) -> Vec3 {
    let h = rig.def.bones[i].head;
    v3(h[0], h[1], h[2])
}

impl Retarget {
    pub fn new(rig: &BodyRig, lib: &Library) -> Option<Retarget> {
        let map = BoneMap::humanoid(rig)?;
        let n = &lib.neutral;
        let mocap_leg = |s: usize| sub(n[j::LEG[s][1]], n[j::LEG[s][0]]).length() + sub(n[j::LEG[s][2]], n[j::LEG[s][1]]).length();
        let body_leg = |s: usize| rig.rest_len[map.leg[s][0]] + rig.rest_len[map.leg[s][1]];
        let k = (body_leg(0) + body_leg(1)) / (mocap_leg(0) + mocap_leg(1));
        let n_limb = [0, 1].map(|s| [0, 1, 2, 3, 4, 5].map(|w| limb(n, s, w)));
        Some(Retarget {
            k,
            neutral: n.clone(),
            n_seg: segments(n),
            n_limb,
            rest_hips: rest_head(rig, map.hips),
            rest_ankle: [rest_head(rig, map.leg[0][2]), rest_head(rig, map.leg[1][2])],
            map,
        })
    }

    /// Bone `i` turned `g` from rest in the body's frame, whatever its parents do.
    fn global(rig: &BodyRig, i: usize, g: &Quat) {
        let parent = rig.def.bones[i].parent.map_or(Quat::IDENTITY, |p| rig.body_rot(p));
        rig.bones[i].set_quat(&qmul(&qinv(&parent), g));
    }

    fn delta(now: &Quat, rest: &Quat, gain: f64) -> Quat {
        let d = qmul(now, &qinv(rest));
        slerp(&Quat::IDENTITY, &d, gain)
    }

    /// Leg `s`'s length (thigh and shin).
    pub fn leg_len(&self, rig: &BodyRig, s: usize) -> f64 {
        rig.rest_len[self.map.leg[s][0]] + rig.rest_len[self.map.leg[s][1]]
    }

    /// Foot `s`'s ankle at rest, in the body's frame (on the floor).
    pub fn rest_ankle(&self, s: usize) -> Vec3 {
        self.rest_ankle[s]
    }

    /// Where the mocap puts foot `s`'s ankle, in the body's frame.
    pub fn ankle_target(&self, p: &[Vec3], s: usize, gain: f64) -> Vec3 {
        let d = sub(p[j::LEG[s][2]], self.neutral[j::LEG[s][2]]);
        add(self.rest_ankle[s], scale(d, self.k * gain))
    }

    /// Pose `rig` as frame `p`, with `gains`; `ankles` overrides where each
    /// ankle goes (a planted foot, in the body's frame).
    pub fn pose(&self, rig: &BodyRig, p: &[Vec3], gains: &Gains, ankles: [Option<Vec3>; 2]) {
        let m = &self.map;
        rig.rest();
        // the hips: the mocap's offset from its stance, at the creature's size
        let dh = sub(p[j::HIPS], self.neutral[j::HIPS]);
        let hip = add(self.rest_hips, scale(dh, self.k * gains.legs));
        rig.bones[m.hips].set_pos(hip.x, hip.y, hip.z);
        let seg = segments(p);
        let n = &self.n_seg;
        let dp = Self::delta(&seg.pelvis, &n.pelvis, gains.trunk);
        let dc = Self::delta(&seg.chest, &n.chest, gains.trunk);
        let dn = Self::delta(&seg.neck, &n.neck, gains.head);
        let dhd = Self::delta(&seg.head, &n.head, gains.head);
        Self::global(rig, m.hips, &dp);
        if let Some(b) = m.spine {
            Self::global(rig, b, &slerp(&dp, &dc, 0.4));
        }
        if let Some(b) = m.chest {
            Self::global(rig, b, &slerp(&dp, &dc, 0.75));
        }
        if let Some(b) = m.upperchest {
            Self::global(rig, b, &dc);
        }
        Self::global(rig, m.neck, &slerp(&dc, &dn, gains.head));
        Self::global(rig, m.head, &slerp(&dc, &dhd, gains.head));
        for s in 0..2 {
            let a = m.arm[s];
            for (w, &b) in a.iter().take(3).enumerate() {
                let d = from_to(&self.n_limb[s][w], &limb(p, s, w));
                Self::global(rig, b, &slerp(&dc, &d, if w == 0 { 1.0 } else { gains.arms }));
            }
            let dhand = Self::delta(&seg.hand[s], &n.hand[s], gains.arms);
            Self::global(rig, a[3], &dhand);
        }
        // the pelvis drops as far as a foot needs to stay in reach: a long
        // stride on these legs dips the body, as it does a person's
        let mut drop: f64 = 0.0;
        for s in 0..2 {
            let l = m.leg[s];
            let target = ankles[s].unwrap_or_else(|| self.ankle_target(p, s, gains.legs));
            let hip = rig.head(l[0]);
            let reach = 0.97 * (rig.rest_len[l[0]] + rig.rest_len[l[1]]);
            let (dx, dy, dz) = (target.x - hip.x, target.y - hip.y, target.z - hip.z);
            let room = (reach * reach - dx * dx - dz * dz).max(0.0).sqrt();
            drop = drop.max(-dy - room);
        }
        if drop > 0.0 {
            let h = rig.bones[m.hips].pos();
            rig.bones[m.hips].set_pos(h.x, h.y - drop.min(0.1), h.z);
        }
        // the legs: their turn, then solved onto the feet
        for s in 0..2 {
            let l = m.leg[s];
            let dt = slerp(&Quat::IDENTITY, &from_to(&self.n_limb[s][3], &limb(p, s, 3)), gains.legs);
            let ds = slerp(&Quat::IDENTITY, &from_to(&self.n_limb[s][4], &limb(p, s, 4)), gains.legs);
            let df = slerp(&Quat::IDENTITY, &from_to(&self.n_limb[s][5], &limb(p, s, 5)), gains.legs);
            Self::global(rig, l[0], &dt);
            Self::global(rig, l[1], &ds);
            let target = ankles[s].unwrap_or_else(|| self.ankle_target(p, s, gains.legs));
            let base = rig.head(l[0]);
            // the knee bends where the mocap's does
            let knee = rig.tail(l[0]);
            // (and forward, so a leg the mocap holds straight still bends the right way)
            let pole = add(add(knee, scale(unit(sub(knee, mid(base, target))), 0.5)), v3(0.0, 0.0, 0.35));
            let (du, dl) = two_bone(base, target, pole, rig.rest_len[l[0]], rig.rest_len[l[1]]);
            rig.aim(l[0], du, 0.0);
            rig.aim(l[1], dl, 0.0);
            Self::global(rig, l[2], &df);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A standing SOMA pose (Kimodo's, unconditioned, frame 0).
    fn stance() -> Vec<Vec3> {
        let p: [[f64; 3]; N] = [
            [-0.004, 0.997, 0.009],
            [-0.004, 1.047, 0.009],
            [-0.004, 1.118, 0.003],
            [-0.003, 1.190, -0.020],
            [0.001, 1.453, -0.002],
            [0.0, 1.528, 0.023],
            [0.0, 1.588, 0.041],
            [0.002, 1.596, 0.071],
            [0.035, 1.649, 0.111],
            [-0.029, 1.648, 0.114],
            [0.016, 1.418, 0.053],
            [0.168, 1.382, 0.014],
            [0.250, 1.109, -0.023],
            [0.207, 0.861, 0.074],
            [0.141, 0.751, 0.102],
            [0.148, 0.696, 0.083],
            [-0.014, 1.418, 0.053],
            [-0.163, 1.369, 0.011],
            [-0.230, 1.090, 0.001],
            [-0.197, 0.843, 0.110],
            [-0.124, 0.737, 0.137],
            [-0.136, 0.677, 0.117],
            [0.097, 0.912, 0.034],
            [0.116, 0.481, 0.001],
            [0.128, 0.063, -0.070],
            [0.154, 0.016, 0.060],
            [-0.105, 0.914, 0.034],
            [-0.128, 0.481, 0.012],
            [-0.149, 0.063, -0.051],
            [-0.162, 0.014, 0.081],
        ];
        p.iter().map(|a| v3(a[0], a[1], a[2])).collect()
    }

    fn library() -> Library {
        let p = stance();
        let frame =
            |dz: f64, c: f64| Frame { p: p.iter().map(|q| v3(q.x, q.y, q.z + dz)).collect(), root: [0.0, dz, 0.0], contact: [c; 4] };
        let clip = Clip {
            name: "t".into(),
            prompt: String::new(),
            looped: true,
            speed: 0.0,
            frames: vec![frame(0.0, 1.0), frame(0.1, 0.0), frame(0.2, 1.0)],
        };
        Library { fps: 30.0, neutral: p, clips: vec![clip] }
    }

    #[test]
    fn the_neutral_stance_is_the_bodys_rest_pose() {
        let lib = library();
        let rig = game::entities::body::build_body("smiler", &|_| engine::prelude::lambert!()).expect("the Smiler's body");
        let rt = Retarget::new(&rig, &lib).expect("a humanoid map");
        assert!(rt.k > 1.0 && rt.k < 1.6, "the Smiler's legs over the mocap's: {}", rt.k);
        rt.pose(&rig, &lib.neutral, &Gains::default(), [None, None]);
        let m = &rt.map;
        // the trunk, head and arms exactly at rest; the legs solved back onto their rest ankles
        let mut bones = vec![m.hips, m.neck, m.head];
        bones.extend(m.spine);
        bones.extend(m.chest);
        bones.extend(m.upperchest);
        bones.extend(m.arm.iter().flatten());
        for b in bones {
            let q = rig.body_rot(b);
            let angle = 2.0 * q.w.abs().min(1.0).dacos();
            assert!(angle < 1e-6, "{} turned {angle} rad from rest", rig.def.bones[b].name);
        }
        for leg in &m.leg {
            let foot = leg[2];
            let (at, want) = (rig.head(foot), rest_head(&rig, foot));
            assert!(sub(at, want).length() < 2e-3, "{} at {at:?}, rest {want:?}", rig.def.bones[foot].name);
        }
    }

    #[test]
    fn a_loop_carries_its_root_across_the_seam() {
        let lib = library();
        let c = &lib.clips[0];
        // forward 0.1 a frame, the seam included
        let (_, vz, _) = c.root_velocity(2.5 / 30.0, 30.0);
        assert!((vz - 3.0).abs() < 1e-9, "{vz}");
        let a = c.sample(1.0 / 30.0, 30.0);
        assert!((a.contact[0]).abs() < 1e-9 && (c.sample(0.5 / 30.0, 30.0).contact[0] - 0.5).abs() < 1e-9);
    }
}
