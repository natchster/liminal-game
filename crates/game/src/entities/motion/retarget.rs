//! A clip's frame laid on a creature's body.
//!
//! **By deltas.** Every segment of the mocap gets a frame built from joint
//! positions alone (a direction and a side: hip to hip, shoulder to
//! shoulder, eye to eye, thumb to finger), so no rotation convention of the
//! source matters. Its rotation away from the library's neutral stance (a
//! clip of the creature's own brief, standing as its body is bound) is laid
//! on the body's rest pose: a body bound in a stoop keeps its stoop, one
//! bound over a page keeps the page before its face, one bound on all fours
//! stays on them. The trunk and head take whole frames (twist included);
//! limbs the least turn of their direction; legs (and on all fours arms) are
//! then solved onto where their ends stand (two bones, the mocap's own knee
//! or elbow for the pole), scaled by the creature's legs over the mocap's.
use super::{Library, N, add, j, mid_of, scale, sub, unit};
use crate::entities::body::{BodyRig, from_to, qinv, qmul, two_bone};
use crate::prelude::*;

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

fn cross(a: Vec3, b: Vec3) -> Vec3 {
    v3(a.y * b.z - a.z * b.y, a.z * b.x - a.x * b.z, a.x * b.y - a.y * b.x)
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
    let fwd = sub(mid_of(p[j::EYE_L], p[j::EYE_R]), p[j::HEAD]);
    let pelvis = basis(sub(p[j::SPINE2], p[j::HIPS]), hip_side);
    let chest = basis(sub(p[j::NECK1], p[j::SPINE2]), chest_side);
    let neck = basis(sub(p[j::HEAD], p[j::NECK1]), mid_of(unit(chest_side), unit(eye_side)));
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
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Gains {
    pub trunk: f64,
    pub head: f64,
    pub arms: f64,
    pub legs: f64,
    /// the hips' rise and fall with each step (0: a walk too even)
    pub bob: f64,
}

impl Default for Gains {
    fn default() -> Gains {
        Gains { trunk: 1.0, head: 1.0, arms: 1.0, legs: 1.0, bob: 1.0 }
    }
}

impl Gains {
    /// `self` toward `o` by `t`.
    pub fn mix(&self, o: &Gains, t: f64) -> Gains {
        let m = |a: f64, b: f64| a + (b - a) * t;
        Gains {
            trunk: m(self.trunk, o.trunk),
            head: m(self.head, o.head),
            arms: m(self.arms, o.arms),
            legs: m(self.legs, o.legs),
            bob: m(self.bob, o.bob),
        }
    }
}

/// Where the ends of the limbs go this frame (the body's frame): a planted
/// foot or hand, or `None` for where the mocap puts it; the whole body
/// shifted by `shift` (held behind its root); the mocap's steps lengthened
/// `stride` times along the way it faces.
#[derive(Clone, Copy, Debug)]
pub struct Ends {
    pub ankles: [Option<Vec3>; 2],
    pub wrists: [Option<Vec3>; 2],
    pub shift: Vec3,
    pub stride: f64,
}

impl Default for Ends {
    fn default() -> Ends {
        Ends { ankles: [None; 2], wrists: [None; 2], shift: v3(0, 0, 0), stride: 1.0 }
    }
}

/// A body's retarget from one motion library.
#[derive(Clone, Debug)]
pub struct Retarget {
    pub map: BoneMap,
    /// the creature's legs over the mocap's
    pub k: f64,
    /// on all fours: the arms are solved onto the hands, as the legs onto the feet
    pub fours: bool,
    /// how much of its bound stoop it stands out of (0: as bound; 1: each
    /// bone of its trunk turned to the vertical); its kind's posture
    pub upright: f64,
    /// how much of that its neck and head take (0: still bowed as bound,
    /// the Reader's to its page)
    pub upright_head: f64,
    neutral: Vec<Vec3>,
    n_seg: Segments,
    n_limb: [[Vec3; 6]; 2],
    rest_hips: Vec3,
    rest_ankle: [Vec3; 2],
    rest_wrist: [Vec3; 2],
    limb_len: [f64; 4],
    /// each hip to its ankle at rest
    rest_reach: [f64; 2],
}

fn rest_head(rig: &BodyRig, i: usize) -> Vec3 {
    let h = rig.def.bones[i].head;
    v3(h[0], h[1], h[2])
}

/// Which way a middle joint (`joint`) bends, away from the middle of its
/// limb (`mid`): 0.5 m that way when it is bent, less as the limb comes
/// straight (a straight limb's bend is noise, and a pole on noise swivels
/// the knee about), so the pole's fixed lean decides.
fn bend_toward(joint: Vec3, mid: Vec3) -> Vec3 {
    let b = sub(joint, mid);
    let l = b.length();
    scale(b, 0.5 / l.max(0.05))
}

/// Bone `i` turned `g` from rest in the body's frame, whatever its parents do.
pub fn set_global(rig: &BodyRig, i: usize, g: &Quat) {
    let parent = rig.def.bones[i].parent.map_or(Quat::IDENTITY, |p| rig.body_rot(p));
    rig.bones[i].set_quat(&qmul(&qinv(&parent), g));
}

/// Bone `i` turned further by `q` in the body's frame (on top of its pose).
pub fn turn_global(rig: &BodyRig, i: usize, q: &Quat) {
    let now = rig.body_rot(i);
    set_global(rig, i, &qmul(q, &now));
}

impl Retarget {
    pub fn new(rig: &BodyRig, lib: &Library, fours: bool) -> Option<Retarget> {
        let map = BoneMap::humanoid(rig)?;
        let n = &lib.neutral;
        if n.len() != N {
            return None;
        }
        let mocap_leg = |s: usize| sub(n[j::LEG[s][1]], n[j::LEG[s][0]]).length() + sub(n[j::LEG[s][2]], n[j::LEG[s][1]]).length();
        let body_leg = |s: usize| rig.rest_len[map.leg[s][0]] + rig.rest_len[map.leg[s][1]];
        let k = (body_leg(0) + body_leg(1)) / (mocap_leg(0) + mocap_leg(1));
        let n_limb = [0, 1].map(|s| [0, 1, 2, 3, 4, 5].map(|w| limb(n, s, w)));
        let arm = |s: usize| rig.rest_len[map.arm[s][1]] + rig.rest_len[map.arm[s][2]];
        let limb_len = [body_leg(0), body_leg(1), arm(0), arm(1)];
        let rest_reach = [0, 1].map(|s| sub(rest_head(rig, map.leg[s][2]), rest_head(rig, map.leg[s][0])).length() + 1e-4);
        Some(Retarget {
            limb_len,
            rest_reach,
            k,
            fours,
            upright: 0.0,
            upright_head: 1.0,
            neutral: n.clone(),
            n_seg: segments(n),
            n_limb,
            rest_hips: rest_head(rig, map.hips),
            rest_ankle: [rest_head(rig, map.leg[0][2]), rest_head(rig, map.leg[1][2])],
            rest_wrist: [rest_head(rig, map.arm[0][3]), rest_head(rig, map.arm[1][3])],
            map,
        })
    }

    /// Bone `b` turned `share` of `upright` of the way from its rest
    /// direction to the vertical (in the body's frame): its posture.
    fn posture(&self, rig: &BodyRig, b: usize, share: f64) -> Quat {
        let u = (self.upright * share).clamp(0.0, 1.0);
        if u <= 0.0 {
            return Quat::IDENTITY;
        }
        let d = rig.rest_dir[b];
        from_to(&d, &unit(add(scale(d, 1.0 - u), v3(0.0, u, 0.0))))
    }

    /// How much taller it stands with its legs straight than as bound, m
    /// (the shorter leg: neither is stretched).
    fn straighten(&self) -> f64 {
        (0..2).map(|s| (0.985 * self.limb_len[s] - self.rest_reach[s]).max(0.0)).fold(f64::MAX, f64::min)
    }

    fn delta(now: &Quat, rest: &Quat, gain: f64) -> Quat {
        let d = qmul(now, &qinv(rest));
        slerp(&Quat::IDENTITY, &d, gain)
    }

    /// End `e`'s limb's length: each leg, then each arm (upper and fore).
    pub fn leg_len_of(&self, e: usize) -> f64 {
        self.limb_len[e]
    }

    /// Foot `s`'s ankle at rest, in the body's frame (on the floor).
    pub fn rest_ankle(&self, s: usize) -> Vec3 {
        self.rest_ankle[s]
    }

    /// Hand `s`'s wrist at rest, in the body's frame.
    pub fn rest_wrist(&self, s: usize) -> Vec3 {
        self.rest_wrist[s]
    }

    /// Where the mocap puts foot `s`'s ankle, in the body's frame, its step
    /// ahead lengthened `stride` times. (On all fours the mocap crawls on its
    /// knees: the knee is what meets the floor in step, and the foot goes
    /// where it goes.)
    pub fn ankle_target(&self, p: &[Vec3], s: usize, gain: f64, stride: f64) -> Vec3 {
        let end = j::LEG[s][if self.fours { 1 } else { 2 }];
        let d = sub(p[end], self.neutral[end]);
        let k = self.k * gain;
        add(self.rest_ankle[s], v3(d.x * k, d.y * k, d.z * k * stride))
    }

    /// Where the mocap puts hand `s`'s wrist, in the body's frame.
    pub fn wrist_target(&self, p: &[Vec3], s: usize, gain: f64, stride: f64) -> Vec3 {
        let d = sub(p[j::ARM[s][3]], self.neutral[j::ARM[s][3]]);
        let k = self.k * gain;
        add(self.rest_wrist[s], v3(d.x * k, d.y * k, d.z * k * stride))
    }

    /// Pose `rig` as frame `p`, with `gains`; `ends` overrides where each
    /// ankle (and on all fours each wrist) goes.
    pub fn pose(&self, rig: &BodyRig, p: &[Vec3], gains: &Gains, ends: &Ends) {
        let m = &self.map;
        rig.rest();
        // the hips: the mocap's offset from its stance, at the creature's size
        // (and the whole body where it is held, behind its root)
        let dh = sub(p[j::HIPS], self.neutral[j::HIPS]);
        let k = self.k * gains.legs;
        // (standing out of its stoop it stands taller too: its legs, bent at
        // rest, straighten part of the way)
        let taller = if self.fours { 0.0 } else { self.upright * self.straighten() };
        let hip = add(add(self.rest_hips, v3(dh.x * k, dh.y * k * gains.bob + taller, dh.z * k)), ends.shift);
        rig.bones[m.hips].set_pos(hip.x, hip.y, hip.z);
        let seg = segments(p);
        let n = &self.n_seg;
        let dp = Self::delta(&seg.pelvis, &n.pelvis, gains.trunk);
        let dc = Self::delta(&seg.chest, &n.chest, gains.trunk);
        let dn = Self::delta(&seg.neck, &n.neck, gains.head);
        let dhd = Self::delta(&seg.head, &n.head, gains.head);
        // its posture: each bone of its trunk turned part of the way out of
        // the stoop it is bound in (on all fours its chest and head raised a
        // little, never reared), the mocap's turn laid on that
        let up = |b: usize, share: f64| self.posture(rig, b, share);
        let pelvis = if self.fours { 0.3 } else { 1.0 };
        let dp = qmul(&dp, &up(m.hips, pelvis));
        set_global(rig, m.hips, &dp);
        if let Some(b) = m.spine {
            set_global(rig, b, &qmul(&slerp(&dp, &dc, 0.4), &up(b, 1.0)));
        }
        if let Some(b) = m.chest {
            set_global(rig, b, &qmul(&slerp(&dp, &dc, 0.75), &up(b, 1.0)));
        }
        if let Some(b) = m.upperchest {
            set_global(rig, b, &qmul(&dc, &up(b, 1.0)));
        }
        set_global(rig, m.neck, &qmul(&slerp(&dc, &dn, gains.head), &up(m.neck, self.upright_head)));
        set_global(rig, m.head, &qmul(&slerp(&dc, &dhd, gains.head), &up(m.head, self.upright_head)));
        for s in 0..2 {
            let a = m.arm[s];
            for (w, &b) in a.iter().take(3).enumerate() {
                let d = from_to(&self.n_limb[s][w], &limb(p, s, w));
                set_global(rig, b, &slerp(&dc, &d, if w == 0 { 1.0 } else { gains.arms }));
            }
            // (half of it: the mocap's hand is framed by its thumb's and its
            // middle finger's ends, and they twitch)
            let dhand = Self::delta(&seg.hand[s], &n.hand[s], gains.arms * 0.5);
            set_global(rig, a[3], &dhand);
        }
        if self.fours {
            // the arms are legs: solved onto where the hands stand, the
            // elbow bending where the mocap's does
            for s in 0..2 {
                let a = m.arm[s];
                let target = ends.wrists[s].unwrap_or_else(|| add(self.wrist_target(p, s, gains.arms, ends.stride), ends.shift));
                let base = rig.head(a[1]);
                let elbow = rig.tail(a[1]);
                let pole = add(add(elbow, bend_toward(elbow, mid_of(base, target))), v3(0.0, 0.0, -0.2));
                let hand = rig.body_rot(a[3]);
                let (du, dl) = two_bone(base, target, pole, rig.rest_len[a[1]], rig.rest_len[a[2]]);
                rig.aim(a[1], du, 0.0);
                rig.aim(a[2], dl, 0.0);
                set_global(rig, a[3], &hand);
            }
        }
        // the pelvis drops as far as a foot needs to stay in reach: a long
        // stride on these legs dips the body, as it does a person's
        let mut drop: f64 = 0.0;
        for s in 0..2 {
            let l = m.leg[s];
            let target = ends.ankles[s].unwrap_or_else(|| add(self.ankle_target(p, s, gains.legs, ends.stride), ends.shift));
            let hip = rig.head(l[0]);
            // (a body bound with straight legs stands as straight: never
            // dropped below its own rest for the margin a bent one is given)
            let reach = (0.97 * (rig.rest_len[l[0]] + rig.rest_len[l[1]])).max(self.rest_reach[s] + taller);
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
            // (on all fours the foot keeps the set it is bound in: the mocap's
            // lies behind its knee)
            let df =
                if self.fours { Quat::IDENTITY } else { slerp(&Quat::IDENTITY, &from_to(&self.n_limb[s][5], &limb(p, s, 5)), gains.legs) };
            set_global(rig, l[0], &dt);
            set_global(rig, l[1], &ds);
            let target = ends.ankles[s].unwrap_or_else(|| add(self.ankle_target(p, s, gains.legs, ends.stride), ends.shift));
            let base = rig.head(l[0]);
            // the knee bends where the mocap's does
            let knee = rig.tail(l[0]);
            // (and forward, so a leg the mocap holds straight still bends the right way)
            let pole = add(add(knee, bend_toward(knee, mid_of(base, target))), v3(0.0, 0.0, 0.35));
            let (du, dl) = two_bone(base, target, pole, rig.rest_len[l[0]], rig.rest_len[l[1]]);
            rig.aim(l[0], du, 0.0);
            rig.aim(l[1], dl, 0.0);
            set_global(rig, l[2], &df);
        }
    }
}
