//! The human rig (the Showroom's staff and the Borrowed House's shadows)
//! and its walk: `HUMAN_RIGS`, `human_finger_pts`, `human_rig`, the staff
//! (`make_staff`, `spawn_staff`, `update_staff`), `gait_noise` and
//! `human_gait`.
//!
//! The rig lives in `Entity.parts` by bone name ("pelvis", "abdomen", "chest",
//! "neck", "head", "torso" (= pelvis), "thigh.L", "shin.L", "foot.L",
//! "toe.L", "clavicle.L", "upperarm.L", "forearm.L", "hand.L",
//! "finger{k}{j}.L" and the same with ".R", plus "skin" / "edge" when the
//! skinned body is bound). The walk's per-entity state is in `Entity.num`
//! under the keys `gSeed`, `gX`, `gZ`, `gYaw`, `gSpd`, `gTurn`, `gPh`, `gHip`,
//! `gLook`, `gLookP` and `headYaw`; the per-leg state of `RigParts::legs[i]`
//! under `legs.{i}.{name}` (`fz`, `ay`, `az`, `ph`, `w`, `y0`,
//! `z0`, `p0`, `ty`, `tz`, `tph`, `ttoe` in `num`; `down`, `sw` in `flag`).
use crate::anomalies::or0;
use crate::base::numeric::{hypot2, hypot3};
use crate::base::records::*;
use crate::entities::{ent_smooth, wrap_pi};
use crate::prelude::*;
use std::collections::HashMap;

/// One of `HUMAN_RIGS`: bone offsets (parent frame, +Y up, +Z forward, the
/// figure's left at +X) and lengths, plus the arms' and fingers' rest pose.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct HumanRig {
    pub hip_y: f64,
    pub hip_x: f64,
    pub hip_out: f64,
    pub thigh: f64,
    pub shin: f64,
    pub ball: [f64; 3],
    pub toe: f64,
    pub heel: f64,
    pub abdomen: [f64; 3],
    pub chest: [f64; 3],
    pub neck: [f64; 3],
    pub head: [f64; 3],
    pub crown: f64,
    pub clav: [f64; 3],
    pub shoulder: [f64; 3],
    pub up_arm: f64,
    pub fore: f64,
    pub arm_out: f64,
    pub elbow: f64,
    pub palm: f64,
    pub fingers: [f64; 4],
    pub spread: [f64; 4],
    pub curl: [f64; 3],
    pub thumb: [f64; 3],
}
/// The staff rig: 2.14 m to the crown, the head a little too long for the body
pub const HUMAN_RIG_STAFF: HumanRig = HumanRig {
    hip_y: 1.075,
    hip_x: 0.094,
    hip_out: 0.03,
    thigh: 0.485,
    shin: 0.49,
    ball: [0.0, -0.072, 0.148],
    toe: 0.075,
    heel: 0.08,
    abdomen: [0.0, 0.115, -0.006],
    chest: [0.0, 0.2, 0.004],
    neck: [0.0, 0.385, -0.03],
    head: [0.0, 0.125, 0.03],
    crown: 0.24,
    clav: [0.024, 0.305, 0.012],
    shoulder: [0.178, 0.035, -0.036],
    up_arm: 0.335,
    fore: 0.27,
    arm_out: 0.22,
    elbow: 0.16,
    palm: 0.098,
    fingers: [0.098, 0.108, 0.101, 0.082],
    spread: [0.026, 0.008, -0.01, -0.027],
    curl: [0.2, 0.32, 0.24],
    thumb: [0.05, 0.042, 0.034],
};
/// The shadow rig: the same shape, gaunt and a little too long in every bone
pub const HUMAN_RIG_SHADOW: HumanRig = HumanRig {
    hip_y: 1.11,
    hip_x: 0.082,
    hip_out: 0.03,
    thigh: 0.51,
    shin: 0.525,
    ball: [0.0, -0.053, 0.132],
    toe: 0.066,
    heel: 0.062,
    abdomen: [0.0, 0.125, -0.01],
    chest: [0.0, 0.215, 0.0],
    neck: [0.0, 0.39, -0.035],
    head: [0.0, 0.16, 0.035],
    crown: 0.24,
    clav: [0.02, 0.31, 0.01],
    shoulder: [0.15, 0.03, -0.035],
    up_arm: 0.37,
    fore: 0.325,
    arm_out: 0.16,
    elbow: 0.2,
    palm: 0.108,
    fingers: [0.118, 0.13, 0.122, 0.1],
    spread: [0.024, 0.008, -0.008, -0.023],
    curl: [0.24, 0.36, 0.28],
    thumb: [0.055, 0.046, 0.037],
};
/// One of `HUMAN_RIGS` by name ("staff", "shadow").
pub fn human_rig_named(name: &str) -> Option<&'static HumanRig> {
    match name {
        "staff" => Some(&HUMAN_RIG_STAFF),
        "shadow" => Some(&HUMAN_RIG_SHADOW),
        _ => None,
    }
}

/// The hand's joints (hand frame: the wrist at the
/// origin, hanging -Y, the palm turned in to the thigh, i.e. -X for the left
/// hand (s = 1), the thumb forward): four fingers of three bones (index
/// first), then the thumb. Each entry is the chain's points, base first.
pub fn human_finger_pts(r: &HumanRig, s: f64) -> Vec<Vec<[f64; 3]>> {
    let mut out = vec![];
    for (k, &len) in r.fingers.iter().enumerate() {
        let l = [len * 0.46, len * 0.3, len * 0.24];
        let mut p = [-s * 0.004, -r.palm * [1.0, 1.02, 0.97, 0.9][k], r.spread[k]];
        let mut ang = 0.0;
        let mut pts = vec![p];
        for j in 0..3 {
            ang += r.curl[j];
            let d = [-s * f64::sin(ang), -f64::cos(ang), r.spread[k] * 0.6];
            let ll = hypot3(d[0], d[1], d[2]);
            p = [p[0] + d[0] / ll * l[j], p[1] + d[1] / ll * l[j], p[2] + d[2] / ll * l[j]];
            pts.push(p);
        }
        out.push(pts);
    }
    let mut p = [-s * 0.012, -0.026, r.spread[0] + 0.004];
    let mut pts = vec![p];
    for (j, d) in [[-s * 0.15, -0.8, 0.55], [-s * 0.12, -0.92, 0.3], [-s * 0.1, -0.95, 0.12]].iter().enumerate() {
        let l = hypot3(d[0], d[1], d[2]);
        p = [p[0] + d[0] / l * r.thumb[j], p[1] + d[1] / l * r.thumb[j], p[2] + d[2] / l * r.thumb[j]];
        pts.push(p);
    }
    out.push(pts);
    out
}

/// One leg of a rig: hip, knee, ankle and toe joints (`side` +1 = left).
#[derive(Clone, Debug)]
pub struct RigLeg {
    pub hip: Obj,
    pub knee: Obj,
    pub ankle: Obj,
    pub toe: Obj,
    pub side: f64,
}
/// One arm of a rig: clavicle to wrist plus the finger chains (`side` +1 = left).
#[derive(Clone, Debug)]
pub struct RigArm {
    pub clav: Obj,
    pub shoulder: Obj,
    pub elbow: Obj,
    pub wrist: Obj,
    pub fingers: Vec<Vec<Obj>>,
    pub side: f64,
}
/// What `human_rig` returns, plus `skin` / `edge` once a body is bound, and
/// the body's other skinned meshes (its clothes, its hair), by name.
#[derive(Clone, Debug)]
pub struct RigParts {
    pub pelvis: Obj,
    pub abdomen: Obj,
    pub chest: Obj,
    pub neck: Obj,
    pub head: Obj,
    pub legs: Vec<RigLeg>,
    pub arms: Vec<RigArm>,
    pub skin: Option<Obj>,
    pub edge: Option<Obj>,
    pub skins: Vec<(String, Obj)>,
}
fn side_name(s: f64) -> &'static str {
    if s > 0.0 { "L" } else { "R" }
}
impl RigParts {
    /// `Entity.parts`: every bone by its name, `torso` (= pelvis), `skin`,
    /// `edge`, and `skin.<name>` for each of `skins`.
    pub fn to_map(&self) -> HashMap<String, Obj> {
        let mut m = HashMap::new();
        for (k, o) in [
            ("pelvis", &self.pelvis),
            ("abdomen", &self.abdomen),
            ("chest", &self.chest),
            ("neck", &self.neck),
            ("head", &self.head),
            ("torso", &self.pelvis),
        ] {
            m.insert(k.to_string(), o.clone());
        }
        for l in &self.legs {
            let n = side_name(l.side);
            for (k, o) in [("thigh", &l.hip), ("shin", &l.knee), ("foot", &l.ankle), ("toe", &l.toe)] {
                m.insert(format!("{}.{}", k, n), o.clone());
            }
        }
        for a in &self.arms {
            let n = side_name(a.side);
            for (k, o) in [("clavicle", &a.clav), ("upperarm", &a.shoulder), ("forearm", &a.elbow), ("hand", &a.wrist)] {
                m.insert(format!("{}.{}", k, n), o.clone());
            }
            for (k, chain) in a.fingers.iter().enumerate() {
                for (j, b) in chain.iter().enumerate() {
                    m.insert(format!("finger{}{}.{}", k, j, n), b.clone());
                }
            }
        }
        if let Some(s) = &self.skin {
            m.insert("skin".into(), s.clone());
        }
        if let Some(e) = &self.edge {
            m.insert("edge".into(), e.clone());
        }
        for (k, o) in &self.skins {
            m.insert(format!("skin.{k}"), o.clone());
        }
        m
    }
    /// The rig back from `Entity.parts` (None without a pelvis or a missing bone).
    pub fn from_map(m: &HashMap<String, Obj>) -> Option<RigParts> {
        let g = |k: &str| m.get(k).cloned();
        let mut legs = vec![];
        let mut arms = vec![];
        for s in [1.0, -1.0] {
            let n = side_name(s);
            legs.push(RigLeg {
                hip: g(&format!("thigh.{n}"))?,
                knee: g(&format!("shin.{n}"))?,
                ankle: g(&format!("foot.{n}"))?,
                toe: g(&format!("toe.{n}"))?,
                side: s,
            });
        }
        for s in [1.0, -1.0] {
            let n = side_name(s);
            let mut fingers = vec![];
            for k in 0..5 {
                let mut chain = vec![];
                for j in 0.. {
                    match g(&format!("finger{k}{j}.{n}")) {
                        Some(b) => chain.push(b),
                        None => break,
                    }
                }
                fingers.push(chain);
            }
            arms.push(RigArm {
                clav: g(&format!("clavicle.{n}"))?,
                shoulder: g(&format!("upperarm.{n}"))?,
                elbow: g(&format!("forearm.{n}"))?,
                wrist: g(&format!("hand.{n}"))?,
                fingers,
                side: s,
            });
        }
        let mut skins: Vec<(String, Obj)> =
            m.iter().filter_map(|(k, o)| k.strip_prefix("skin.").map(|n| (n.to_string(), o.clone()))).collect();
        skins.sort_by(|a, b| a.0.cmp(&b.0));
        Some(RigParts {
            pelvis: g("pelvis")?,
            abdomen: g("abdomen")?,
            chest: g("chest")?,
            neck: g("neck")?,
            head: g("head")?,
            legs,
            arms,
            skin: g("skin"),
            edge: g("edge"),
            skins,
        })
    }
}

thread_local! {
    /// The stand-in capsule geometries, shared by size
    static RIG_CAPS: RefCell<HashMap<String, Geo>> = RefCell::new(HashMap::new());
}
/// A named bone at (x, y, z) under `parent`, with an optional mesh (`geo` + `mat`).
fn joint_o(geo: Option<Geo>, mat: Option<&Mat>, parent: &Obj, x: f64, y: f64, z: f64, name: &str) -> Obj {
    let pivot = bone();
    pivot.set_name(name);
    pivot.set_pos(x, y, z);
    if let (Some(g), Some(m)) = (geo, mat) {
        pivot.add(&mesh(g, m));
    }
    parent.add(&pivot);
    pivot
}
/// Tag a bone with its name (`main` in its user data).
fn set_main(o: &Obj, name: &str) {
    o.user().s.insert("main".into(), name.to_string());
}
/// Store a bone's tail offset (`tail_x` / `tail_y` / `tail_z` in its user data).
fn set_tail(o: &Obj, t: [f64; 3]) {
    let mut u = o.user();
    u.set("tail_x", t[0]);
    u.set("tail_y", t[1]);
    u.set("tail_z", t[2]);
}

/// The named bones of a human body under `g`.
/// With `rigid_mat` (no skinned body), a capsule down each bone stands in.
pub fn human_rig(g: &Obj, r: &HumanRig, rigid_mat: Option<&Mat>) -> RigParts {
    // until the Blender body is built, a capsule down each bone stands in
    let cap = |len: f64, rad: f64, up: bool| -> Option<Geo> {
        rigid_mat?;
        let key = format!("{}:{}:{}", fmt_fixed(len, 3), fmt_num(rad), up);
        Some(RIG_CAPS.with(|c| {
            c.borrow_mut()
                .entry(key)
                .or_insert_with(|| {
                    let geo = capsule_geo!(rad, 0.005f64.max(len - 2.0 * rad), 3, 8);
                    geo.translate(0, (if up { 1.0 } else { -1.0 }) * len / 2.0, 0);
                    geo
                })
                .clone()
        }))
    };
    let m = rigid_mat;
    let pelvis = joint_o(cap(0.22, 0.15, true), m, g, 0.0, r.hip_y, 0.0, "pelvis");
    let abdomen = joint_o(cap(0.2, 0.13, true), m, &pelvis, r.abdomen[0], r.abdomen[1], r.abdomen[2], "abdomen");
    let chest = joint_o(cap(0.38, 0.16, true), m, &abdomen, r.chest[0], r.chest[1], r.chest[2], "chest");
    let neck = joint_o(cap(0.15, 0.05, true), m, &chest, r.neck[0], r.neck[1], r.neck[2], "neck");
    let head = joint_o(cap(r.crown, 0.09, true), m, &neck, r.head[0], r.head[1], r.head[2], "head");
    set_main(&pelvis, "abdomen");
    set_main(&chest, "neck");
    set_tail(&head, [0.0, r.crown, 0.012]);
    let mut legs = vec![];
    for s in [1.0, -1.0] {
        let n = side_name(s);
        let hip = joint_o(cap(r.thigh, 0.07, false), m, &pelvis, s * r.hip_x, 0.0, 0.0, &format!("thigh.{n}"));
        hip.rotation().z = s * r.hip_out;
        let knee = joint_o(cap(r.shin, 0.055, false), m, &hip, 0.0, -r.thigh, 0.0, &format!("shin.{n}"));
        let ankle = joint_o(None, None, &knee, 0.0, -r.shin, 0.0, &format!("foot.{n}"));
        ankle.rotation().z = -s * r.hip_out; // the sole flat on the floor
        let toe = joint_o(None, None, &ankle, r.ball[0], r.ball[1], r.ball[2], &format!("toe.{n}"));
        set_main(&ankle, &format!("toe.{n}"));
        set_tail(&toe, [0.0, 0.0, r.toe]);
        legs.push(RigLeg { hip, knee, ankle, toe, side: s });
    }
    let mut arms = vec![];
    for s in [1.0, -1.0] {
        let n = side_name(s);
        let clav = joint_o(None, None, &chest, s * r.clav[0], r.clav[1], r.clav[2], &format!("clavicle.{n}"));
        set_main(&clav, &format!("upperarm.{n}"));
        let shoulder =
            joint_o(cap(r.up_arm, 0.05, false), m, &clav, s * r.shoulder[0], r.shoulder[1], r.shoulder[2], &format!("upperarm.{n}"));
        shoulder.rotation().z = s * r.arm_out;
        let elbow = joint_o(cap(r.fore, 0.04, false), m, &shoulder, 0.0, -r.up_arm, 0.0, &format!("forearm.{n}"));
        elbow.rotation().x = -r.elbow;
        let wrist = joint_o(cap(r.palm + r.fingers[1] * 0.8, 0.035, false), m, &elbow, 0.0, -r.fore, 0.0, &format!("hand.{n}"));
        wrist.rotation().z = -s * r.arm_out * 0.5; // the hand hangs nearer plumb
        set_main(&wrist, &format!("finger10.{n}"));
        let fingers = human_finger_pts(r, s)
            .iter()
            .enumerate()
            .map(|(k, pts)| {
                let mut chain = vec![];
                let mut parent = wrist.clone();
                let mut prev = [0.0, 0.0, 0.0];
                for j in 0..pts.len() - 1 {
                    let b = joint_o(
                        None,
                        None,
                        &parent,
                        pts[j][0] - prev[0],
                        pts[j][1] - prev[1],
                        pts[j][2] - prev[2],
                        &format!("finger{k}{j}.{n}"),
                    );
                    chain.push(b.clone());
                    prev = pts[j];
                    parent = b;
                }
                let (last, before) = (pts[pts.len() - 1], pts[pts.len() - 2]);
                set_tail(&parent, [last[0] - before[0], last[1] - before[1], last[2] - before[2]]);
                chain
            })
            .collect();
        arms.push(RigArm { clav, shoulder, elbow, wrist, fingers, side: s });
    }
    RigParts { pelvis, abdomen, chest, neck, head, legs, arms, skin: None, edge: None, skins: vec![] }
}

/// What `make_staff` / `make_shadow` return.
#[derive(Clone, Debug)]
pub struct HumanBuild {
    pub g: Obj,
    pub face: Obj,
    pub parts: RigParts,
    pub face_y: f64,
}

/// The walk's style (`st` of `human_gait`; all eased by the caller,
/// 0..1): hunt — long strides, a stiff upper body, arms hanging; glide — a
/// drifting, barely lifted step; reach — the arms up in front; look — the
/// head on the player; shiver — the smoke's tremor; breath; tilt — a head
/// tilt (radians).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GaitStyle {
    pub hunt: f64,
    pub glide: f64,
    pub reach: f64,
    pub look: f64,
    pub shiver: f64,
    pub tilt: f64,
    pub breath: f64,
}

/// Smooth noise (a sum of sines)
pub fn gait_noise(x: f64) -> f64 {
    x.sin() * 0.6 + (x * 2.13 + 1.3).sin() * 0.3 + (x * 4.37 + 2.6).sin() * 0.1
}

/// Two-bone IK in a limb's plane: hip (hy, hz) and target (ty, tz), bone
/// lengths `a` / `b`, `fwd` = the knee bends forward → the two joint angles
/// (t1, t2). The same function as `entities::solve_limb`; kept here so the
/// rig's walk does not depend on that module.
pub fn solve_limb_ik(hy: f64, hz: f64, ty: f64, tz: f64, a: f64, b: f64, fwd: bool) -> (f64, f64) {
    let (mut dy, mut dz) = (ty - hy, tz - hz);
    let mut d = hypot2(dy, dz);
    let d_max = (a + b) * 0.998;
    if d > d_max {
        dy *= d_max / d;
        dz *= d_max / d;
        d = d_max;
    }
    d = d.max((a - b).abs() + 1e-3);
    let aa = clamp((a * a + d * d - b * b) / (2.0 * a * d), -1.0, 1.0).acos();
    let t1 = dz.atan2(-dy) + if fwd { aa } else { -aa };
    let t2 = (dz - t1.sin() * a).atan2(-(dy + t1.cos() * a));
    (t1, t2)
}

fn lk(i: usize, k: &str) -> String {
    format!("legs.{i}.{k}")
}

/// The walk on one entity with rig `r` and style `st` (`player` = the player's position).
pub fn human_gait_entity(e: &mut Entity, player: Vec3, dt: f64, t: f64, r: &HumanRig, st: &GaitStyle) {
    if !(dt > 0.0) {
        return;
    }
    let Some(p) = RigParts::from_map(&e.parts) else { return };
    let tau = PI * 2.0;
    if !e.num.contains_key("gSeed") {
        e.set_n("gSeed", rnd() * 100.0);
    } // once: its own rhythm
    let seed = e.n("gSeed");
    let pos = e.pos();
    let mut moved = hypot2(pos.x - e.num.get("gX").copied().unwrap_or(pos.x), pos.z - e.num.get("gZ").copied().unwrap_or(pos.z));
    let turned = wrap_pi(e.yaw - e.num.get("gYaw").copied().unwrap_or(e.yaw)).abs();
    e.set_n("gX", pos.x);
    e.set_n("gZ", pos.z);
    e.set_n("gYaw", e.yaw);
    if moved > 0.6 {
        moved = 0.0;
    } // a relocation, not a stride
    let k_s = 1.0 - (-dt * 6.0).exp();
    let g_spd = or0(e.n("gSpd")) + (moved / dt - or0(e.n("gSpd"))) * k_s;
    e.set_n("gSpd", g_spd);
    let g_turn = or0(e.n("gTurn")) + ((turned / dt).min(6.0) - or0(e.n("gTurn"))) * k_s;
    e.set_n("gTurn", g_turn);
    let (spd, hunt, glide, reach_k, shiv) = (g_spd, st.hunt, st.glide, st.reach, st.shiver);
    let walk_k = ent_smooth(0.04, 0.5, spd);
    let idle = 1.0 - walk_k;
    let step_k = walk_k.max(0.75 * ent_smooth(0.25, 1.0, g_turn) * idle); // feet lifting: walking, or turning on the spot
    // cadence (cycles/s) and the cycle it makes at this speed
    let f = lerp(lerp(0.52 + 0.31 * spd, 0.45 + 0.35 * spd, hunt), 0.42 + 0.3 * spd, glide);
    let c = 0.3f64.max(spd / f);
    // the share of a cycle a foot is down
    let d = clamp(
        lerp(lerp(0.63, 0.53, clamp(spd / 3.0, 0.0, 1.0)), 0.43, hunt).min(lerp(0.6, 0.36, clamp((spd - 2.0) / 3.0, 0.0, 1.0))),
        0.34,
        0.66,
    );
    let g_ph = (or0(e.n("gPh")) + (moved + turned * 0.3 * idle) * tau / c) % (tau * 64.0);
    e.set_n("gPh", g_ph);
    let u_l = (g_ph / tau) % 1.0;
    let s_cyc = d * c;
    let off = -0.02 * walk_k;
    let ank_y = r.hip_y - r.thigh - r.shin;
    let bz = r.ball[2];
    // the feet first: where each one is (the body's frame), heel to toe
    let lift = (0.06 + 0.035 * hunt - 0.04 * glide) * step_k;
    let ph_h = (0.24 + 0.08 * hunt) * (1.0 - 0.6 * glide) * walk_k; // toes up at heel strike
    let ph_t = (0.42 + 0.04 * spd.min(3.0)) * (1.0 - 0.6 * glide) * walk_k; // heel up at toe-off
    let foot_x = r.hip_x + (r.thigh + r.shin) * r.hip_out.sin() - 0.03 * hunt; // at night, feet nearly in a line
    let land_y = ank_y * ph_h.cos() + r.heel * ph_h.sin();
    let land_z = s_cyc / 2.0 + off - r.heel - ank_y * ph_h.sin() + r.heel * ph_h.cos();
    let rest = 1.0 - step_k;
    for (li, lg) in p.legs.iter().enumerate() {
        let s = lg.side;
        let u = (u_l + if s > 0.0 { 0.0 } else { 0.5 }) % 1.0;
        let (az, ay, ph, toe);
        let down = u < d;
        e.set_fl(&lk(li, "down"), down);
        if down {
            // stance: anchored where it landed
            let fz = match e.num.get(&lk(li, "fz")).copied() {
                Some(fz) if !e.fl(&lk(li, "sw")) => fz - moved,
                _ => s_cyc / 2.0 + off - s_cyc * u / d, // touch-down: on the mark, less how far it has come since
            };
            e.set_n(&lk(li, "fz"), fz);
            e.set_fl(&lk(li, "sw"), false);
            let v = u / d;
            ph = ph_h * (1.0 - ent_smooth(0.0, 0.2, v)) - ph_t * ent_smooth(0.45, 1.0, v);
            if ph >= 0.0 {
                ay = ank_y * ph.cos() + r.heel * ph.sin();
                az = fz - r.heel - ank_y * ph.sin() + r.heel * ph.cos();
            } else {
                ay = ank_y * ph.cos() - bz * ph.sin();
                az = fz + bz - ank_y * ph.sin() - bz * ph.cos();
            }
            toe = 0f64.min(ph);
            e.set_n(&lk(li, "ay"), ay);
            e.set_n(&lk(li, "az"), az);
            e.set_n(&lk(li, "ph"), ph);
        } else {
            // swing: off the ball, through, onto the heel at the next mark
            let w = (u - d) / (1.0 - d);
            e.set_n(&lk(li, "w"), w);
            if !e.fl(&lk(li, "sw")) {
                e.set_fl(&lk(li, "sw"), true);
                let y0 = e.num.get(&lk(li, "ay")).copied().unwrap_or(ank_y);
                let z0 = e.num.get(&lk(li, "az")).copied().unwrap_or(0.0);
                let p0 = e.num.get(&lk(li, "ph")).copied().unwrap_or(0.0);
                e.set_n(&lk(li, "y0"), y0);
                e.set_n(&lk(li, "z0"), z0);
                e.set_n(&lk(li, "p0"), p0);
            }
            let (y0, z0, p0) = (e.n(&lk(li, "y0")), e.n(&lk(li, "z0")), e.n(&lk(li, "p0")));
            let m = -c * (1.0 - d) * walk_k; // leave and land at the ground's speed
            let w2 = w * w;
            let w3 = w2 * w;
            az = (2.0 * w3 - 3.0 * w2 + 1.0) * z0 + (w3 - 2.0 * w2 + w) * m + (3.0 * w2 - 2.0 * w3) * land_z + (w3 - w2) * m;
            ay = lerp(y0, land_y, ent_smooth(0.0, 1.0, w)) + lift * (PI * w).sin() * (1.15 - 0.3 * w);
            ph = lerp(p0, ph_h, ent_smooth(0.1, 0.85, w)) - 0.12 * walk_k * (PI * w).sin() * (1.0 - w);
            toe = 0f64.min(p0) * (1.0 - ent_smooth(0.0, 0.35, w));
        }
        // stopping (or never started): the foot settles flat where it is
        e.set_n(&lk(li, "ty"), lerp(ay, ank_y, rest));
        e.set_n(&lk(li, "tz"), az);
        e.set_n(&lk(li, "tph"), lerp(ph, 0.0, rest));
        e.set_n(&lk(li, "ttoe"), lerp(toe, 0.0, rest));
    }
    // the pelvis: as high as the planted legs let it ride (knees a little
    // soft), swaying onto the stance foot, dropping on the swing side, turning
    // with the forward leg. Standing, a slow shift of weight from foot to foot.
    let mid_l = (tau * (u_l - d / 2.0)).cos(); // +1 at the left foot's mid-stance, -1 at the right's
    let shift = gait_noise(t * 0.31 + seed) * idle * (1.0 - hunt);
    let sway = 0.022 * (1.0 - 0.5 * hunt) * (1.0 - 0.3 * glide) * walk_k * mid_l + 0.028 * shift;
    let psi = -(0.045 + 0.025 * spd.min(2.0) + 0.03 * hunt) * walk_k * (tau * u_l).cos();
    let roll = 0.032 * (1.0 - 0.5 * glide) * walk_k * mid_l + 0.035 * shift;
    let pitch = 0.02 + (0.0125 * spd.min(2.0) + 0.09 * hunt) * walk_k + 0.03 * hunt + 0.08 * reach_k;
    let l_sum = r.thigh + r.shin;
    let l_soft = l_sum * (0.988 + 0.008 * idle);
    let l_max = l_sum * 0.997;
    let mut h_soft = r.hip_y - 0.006 - 0.008 * shift.abs();
    let mut h_hard = 9.0f64;
    // a planted leg holds the hips; a swinging one lets go gradually after it
    // leaves the floor and takes hold gradually as it comes in to land
    for pass in 0..2 {
        for (li, lg) in p.legs.iter().enumerate() {
            let down = e.fl(&lk(li, "down"));
            let planted = down || step_k <= 0.05;
            if planted != (pass == 0) {
                continue;
            }
            let w = e.n(&lk(li, "w"));
            let hold = if planted { 1.0 } else { (1.0 - ent_smooth(0.0, 0.3, w)).max(ent_smooth(0.35, 0.95, w)) };
            if hold <= 0.0 {
                continue;
            }
            let s = lg.side;
            let (ty, tz) = (e.n(&lk(li, "ty")), e.n(&lk(li, "tz")));
            let dz = tz + s * r.hip_x * psi.sin();
            let dx = s * foot_x - (sway + s * r.hip_x * psi.cos());
            let lift0 = s * r.hip_x * roll.sin();
            let hi = ty - lift0 + (l_soft * l_soft - dz * dz - dx * dx).max(0.0).sqrt();
            h_soft = lerp(h_soft, h_soft.min(hi), hold);
            if down {
                h_hard = h_hard.min(ty - lift0 + (l_max * l_max - dz * dz - dx * dx).max(0.0).sqrt());
            }
        }
    }
    let reach_z = (s_cyc / 2.0 - 0.03).max(0.0);
    let h_low = ank_y + 0.014 + (l_soft * l_soft - reach_z * reach_z).max(0.0).sqrt() - 0.03 * hunt; // level: low enough for the longest reach
    let h_want = lerp(h_soft - 0.02 * hunt, h_soft.min(h_low), clamp(0.85 * glide + 0.35 * hunt, 0.0, 1.0));
    // it settles at once and rises eased
    let g_hip = match e.num.get("gHip").copied() {
        Some(gh) if !(h_want < gh) => gh + (h_want - gh) * (1.0 - (-dt * 12.0).exp()),
        _ => h_want,
    };
    e.set_n("gHip", g_hip);
    let hip_y = g_hip.min(h_hard);
    p.pelvis.position().set(sway, hip_y, 0);
    p.pelvis.rotation().set(pitch, psi, roll);
    p.pelvis.update_matrix();
    let gait_m = p.pelvis.matrix().inverted();
    // the legs onto the feet
    for (li, lg) in p.legs.iter().enumerate() {
        let s = lg.side;
        let mut v = v3(s * foot_x, e.n(&lk(li, "ty")), e.n(&lk(li, "tz")));
        v.apply_matrix4(&gait_m); // the ankle's target, in the pelvis frame
        let (dx, dy, dz) = (v.x - s * r.hip_x, v.y, v.z);
        let rho = dx.atan2(-dy);
        let (t1, t2) = solve_limb_ik(0.0, 0.0, -hypot2(dx, dy), dz, r.thigh, r.shin, true);
        lg.hip.rotation().set(-t1, 0, rho);
        lg.knee.rotation().x = t1 - t2;
        lg.ankle.rotation().set(-e.n(&lk(li, "tph")) + t2 - pitch, -psi + s * 0.05 * (1.0 - hunt), -roll - rho);
        lg.toe.rotation().x = e.n(&lk(li, "ttoe"));
    }
    // the head: eased round to you, and down (or up) to your eyes
    let d_p = hypot2(player.x - pos.x, player.z - pos.z);
    let want = clamp(wrap_pi((player.x - pos.x).atan2(player.z - pos.z) - e.yaw), -1.3, 1.3) * st.look;
    let eye_y = hip_y + r.abdomen[1] + r.chest[1] + r.neck[1] + r.head[1] + 0.08;
    let want_p = clamp((eye_y - player.y).atan2(0.6f64.max(d_p)), -0.4, 0.6) * st.look;
    let k_l = 1.0 - (-dt * 3.5).exp();
    let g_look = or0(e.n("gLook")) + (want - or0(e.n("gLook"))) * k_l;
    e.set_n("gLook", g_look);
    let g_look_p = or0(e.n("gLookP")) + (want_p - or0(e.n("gLookP"))) * k_l;
    e.set_n("gLookP", g_look_p);
    e.set_n("headYaw", g_look);
    // the spine: against the pelvis, leaning into a hunt, breathing
    let breath = (t * tau / 4.4 + seed).sin() * st.breath;
    let lean = (0.03 + 0.05 * hunt) * walk_k + 0.02 * hunt + 0.12 * reach_k;
    let drift = 0.035 * glide;
    let ab = [lean * 0.5 - 0.006 * breath, -0.6 * psi * (1.0 - 0.4 * hunt), -0.55 * roll + drift * gait_noise(t * 0.7 + seed)];
    p.abdomen.rotation().set(ab[0], ab[1], ab[2]);
    let ch = [
        lean * 0.5 - 0.014 * breath + shiv * 0.02 * gait_noise(t * 8.3 + seed),
        -0.8 * psi * (1.0 - 0.5 * hunt) + 0.22 * g_look,
        -0.3 * roll + drift * gait_noise(t * 0.53 + seed * 2.0) + shiv * 0.02 * gait_noise(t * 10.1 + seed),
    ];
    p.chest.rotation().set(ch[0], ch[1], ch[2]);
    let yaw_c = psi + ab[1] + ch[1];
    let pitch_c = pitch + ab[0] + ch[0];
    let roll_c = roll + ab[2] + ch[2];
    let (hy, hp) = (g_look - yaw_c, g_look_p - pitch_c);
    p.neck.rotation().set(0.45 * hp, 0.4 * hy, -0.4 * roll_c);
    p.head.rotation().set(
        0.55 * hp + shiv * 0.03 * gait_noise(t * 7.1 + seed * 3.0),
        0.6 * hy + shiv * 0.04 * gait_noise(t * 5.3 + seed),
        -0.6 * roll_c + st.tilt,
    );
    // the arms: against the legs, the elbows easing more on the forward swing;
    // hunting they hang and only sway; reaching, they come up in front
    let arm_a = (0.2 + 0.05 * spd.min(2.5)) * walk_k * (1.0 - 0.8 * hunt) * (1.0 - 0.5 * glide);
    for a in &p.arms {
        let s = a.side;
        let u = (u_l + if s > 0.0 { 0.0 } else { 0.5 }) % 1.0;
        let fwd = -(tau * (u - 0.05 - 0.12 * hunt)).cos(); // +1: this arm forward (its leg back)
        let sw = arm_a * fwd + 0.03 * walk_k + 0.03 * hunt * walk_k * gait_noise(t * 1.1 + seed + s);
        a.clav.rotation().set(0, -s * 0.12 * sw - s * 0.08 * reach_k, s * (0.012 * breath - 0.035 * hunt + 0.06 * reach_k));
        // reaching, the arms come up nearly level, one a little ahead of the other
        let rk = reach_k * (1.0 + 0.08 * s + 0.06 * gait_noise(t * 1.7 + seed + s));
        a.shoulder.rotation().set(
            -sw * (1.0 - reach_k) - 1.32 * rk + shiv * 0.04 * gait_noise(t * 9.0 + s * 3.0 + seed),
            0,
            s * (r.arm_out - 0.07 * walk_k - 0.07 * hunt - 0.17 * reach_k),
        );
        a.elbow.rotation().x =
            -(r.elbow * (1.0 - 0.7 * hunt) * (1.0 - 0.6 * reach_k) + 0.32 * 0f64.max(fwd) * arm_a + 0.05 * walk_k * (1.0 - hunt));
        a.wrist.rotation().set(0.06 * fwd * walk_k * (1.0 - hunt) + 0.25 * reach_k, 0, -s * r.arm_out * 0.5 * (1.0 - reach_k));
        let cc = 0.05 + 0.2 * hunt * (1.0 - reach_k) - 0.1 * reach_k
            + 0.04 * gait_noise(t * 0.8 + seed + s * 5.0)
            + shiv * 0.06 * gait_noise(t * 13.0 + s);
        for (k, fk) in a.fingers.iter().enumerate() {
            let kk = if k == 4 { 0.4 } else { 1.0 };
            for (j, b) in fk.iter().enumerate() {
                b.rotation().z = -s * cc * kk * (0.7 + 0.3 * j as f64);
            }
        }
    }
}

thread_local! {
    static STAFF_MATS: RefCell<HashMap<String, Mat>> = Default::default();
}

/// The staff's materials by slot: pale skin with a little waxy sheen (its
/// fine detail); the uniform's cloth matte, its weave close up; the shoes
/// polished; the hair matte; the badge a plain card.
pub fn staff_material(slot: &str) -> Mat {
    STAFF_MATS.with(|m| {
        m.borrow_mut()
            .entry(slot.to_string())
            .or_insert_with(|| match slot {
                "cloth" => {
                    crate::entities::with_skin_detail(lambert!({ color: 0xffffff, vertexColors: true, emissive: 0x050403 }), 3.8, 0.6)
                }
                "shoe" => phong!({ color: 0xffffff, vertexColors: true, specular: 0x34312c, shininess: 40 }),
                "hair" => lambert!({ color: 0xffffff, vertexColors: true }),
                "badge" => lambert!({ color: 0xffffff, vertexColors: true, emissive: 0x0a0a09 }),
                _ => crate::entities::with_skin_detail(
                    phong!({ color: 0xffffff, vertexColors: true, specular: 0x2c2a27, shininess: 26 }),
                    3.4,
                    0.6,
                ),
            })
            .clone()
    })
}

/// Bind body `name` of `bodies.bin` to the rig under `g` (its bones by
/// name, as they stand now): its "skin" as `parts.skin`, the rest as
/// `parts.skins`. A body made for the rig (`assets/src/creatures/figure.py`)
/// has the rig's bones; nothing is bound when it is missing.
pub fn bind_figure(g: &Obj, name: &str, parts: &mut RigParts, mat: &dyn Fn(&str) -> Mat) {
    let Some(def) = crate::assets::bodies::body(name) else { return };
    for m in &def.meshes {
        let Some(geo) = crate::assets::bodies::body_geometry(name, &m.name) else { continue };
        let Some(sm) = crate::entities::skin_body(g, &geo, &mat(&m.material)) else { continue };
        sm.set_name(&format!("{name}:{}", m.name));
        if m.name == "skin" {
            parts.skin = Some(sm);
        } else {
            parts.skins.push((m.name.clone(), sm));
        }
    }
}

impl Game {
    /// A Showroom staff member at tile (x, z): the rig, and the body
    /// (`bodies.bin` "staff", `assets/src/creatures/body_staff.py`) bound to
    /// its bones as they stand at rest, a skinned mesh for each of its parts.
    pub fn make_staff(&mut self, x: i32, z: i32) -> HumanBuild {
        let g = group();
        let mut parts = human_rig(&g, &HUMAN_RIG_STAFF, None);
        bind_figure(&g, "staff", &mut parts, &staff_material);
        g.position().set(self.wx(x), 0, self.wz(z));
        self.world.add(&g);
        let face = node(); // no glowing face: the bookkeeping expects one
        self.scene.add(&face);
        HumanBuild { g, face, parts, face_y: 2.0 }
    }

    pub fn spawn_staff(&mut self, count: i32, dist_from_spawn: &[i32]) {
        let mut open: Vec<(i32, i32)> = vec![];
        for z in 1..self.h - 1 {
            for x in 1..self.w - 1 {
                let i = self.t_idx(x, z);
                if self.tiles[i] == 0 && dist_from_spawn[i] as f64 > self.w as f64 * 0.35 && self.zone_arch_of(i) == "store" {
                    open.push((x, z));
                }
            }
        }
        let mut i = 0;
        while i < count && !open.is_empty() {
            let (x, z) = open.remove((rnd() * open.len() as f64).floor() as usize);
            let b = self.make_staff(x, z);
            let mut e = Entity::new("staff", b.g.clone());
            e.face = Some(b.face);
            e.face_y = b.face_y;
            e.eye_y_offset = 0.1;
            e.parts = b.parts.to_map();
            e.yaw = rand1(PI * 2.0);
            e.state = "wander".into();
            e.speed = 4.5 + (self.depth() as f64 * 0.08).min(0.8);
            e.set_n("lookK", 0.0);
            e.set_n("huntK", 0.0);
            self.entities.push(e);
            i += 1;
        }
    }

    pub fn human_gait(&mut self, ei: usize, dt: f64, t: f64, r: &HumanRig, st: &GaitStyle) {
        let pp = self.player.pos;
        human_gait_entity(&mut self.entities[ei], pp, dt, t, r, st);
    }

    /// → true when the player was caught (the frame must stop there)
    pub fn update_staff(&mut self, ei: usize, dt: f64, t: f64, d: f64) -> bool {
        let night = self.store_state.phase == "night";
        let pp = self.player.pos;
        if !night {
            // by day: browse the floor slowly; when you are close, stop and look
            self.entities[ei].state = "wander".into();
            let ep = self.entities[ei].pos();
            let near = d < 4.5 && self.los(ep.x, ep.z, pp.x, pp.z);
            {
                let e = &mut self.entities[ei];
                let k = e.n("lookK");
                e.set_n("lookK", k + ((if near { 1.0 } else { 0.0 }) - k) * 1f64.min(dt * 3.0));
            }
            if !near {
                self.entities[ei].w_timer -= dt;
                if self.entities[ei].w_field.is_none() || self.entities[ei].w_timer <= 0.0 {
                    self.entities[ei].w_timer = rand2(6.0, 12.0);
                    for _ in 0..10 {
                        let gx = irand(1, self.w as i64 - 2) as i32;
                        let gz = irand(1, self.h as i64 - 2) as i32;
                        if self.tiles[self.t_idx(gx, gz)] != 0 {
                            continue;
                        }
                        let f = Rc::new(self.bfs(gx, gz));
                        let e = &mut self.entities[ei];
                        e.w_target = Some((gx, gz));
                        e.w_field = Some(f);
                        break;
                    }
                }
                if let Some(f) = self.entities[ei].w_field.clone() {
                    let p0 = self.entities[ei].pos();
                    self.entity_step_toward(ei, Some(f), dt, 1.05, None);
                    let p1 = self.entities[ei].pos();
                    let _speed = hypot2(p1.x - p0.x, p1.z - p0.z) / dt.max(1e-4);
                    // (computed, never read)
                }
            } else {
                self.entity_drive(ei, dt, 0.0); // it slows to a stop, then turns to you
                let e = &self.entities[ei];
                let _speed = hypot2(or0(e.vx), or0(e.vz));
                let ep = e.pos();
                self.entity_turn(ei, (pp.x - ep.x).atan2(pp.z - ep.z), dt, Some(1.6));
            }
        } else {
            // by night: the lamp gives you away from across the floor, footsteps from
            // a room away; in the dark and quiet, only right next to you. Hidden = gone.
            let hidden = self.store_state.hidden;
            let range = if self.player.flashlight_on { 18.0 } else { 5.0 };
            let ep = self.entities[ei].pos();
            let seen = !hidden && d < range && self.los(ep.x, ep.z, pp.x, pp.z);
            let heard = !hidden && d < or0(self.steps_heard()) * 1.4;
            if seen || heard {
                if self.entities[ei].state != "hunt" {
                    self.entities[ei].state = "hunt".into();
                    if self.ac.is_some() {
                        self.blip(220.0, 0.25, "sawtooth", 0.05, Some(330.0), 0.0, 0.0);
                    }
                }
                self.entities[ei].lose_t = 0.0;
            } else if self.entities[ei].state == "hunt" {
                let e = &mut self.entities[ei];
                e.lose_t += dt;
                if e.lose_t > 4.0 {
                    e.state = "wander".into();
                    e.w_field = None;
                }
            }
            let p0 = self.entities[ei].pos();
            if self.entities[ei].state == "hunt" && self.player_field.is_some() {
                let (f, sp) = (self.player_field.clone(), self.entities[ei].speed);
                self.entity_step_toward(ei, f, dt, sp, Some(pp));
            } else {
                self.entities[ei].w_timer -= dt;
                if self.entities[ei].w_field.is_none() || self.entities[ei].w_timer <= 0.0 {
                    self.entities[ei].w_timer = rand2(3.0, 6.0);
                    // search near where the player was last noticed
                    let (ptx, ptz) = self.nav_tile_at(pp.x, pp.z);
                    let gx = clamp((ptx as i64 + irand(-4, 4)) as f64, 1.0, (self.w - 2) as f64) as i32;
                    let gz = clamp((ptz as i64 + irand(-4, 4)) as f64, 1.0, (self.h - 2) as f64) as i32;
                    if self.tiles[self.t_idx(gx, gz)] == 0 {
                        let f = Rc::new(self.bfs(gx, gz));
                        let e = &mut self.entities[ei];
                        e.w_target = Some((gx, gz));
                        e.w_field = Some(f);
                    }
                }
                if let Some(f) = self.entities[ei].w_field.clone() {
                    self.entity_step_toward(ei, Some(f), dt, 2.1, None);
                }
            }
            let p1 = self.entities[ei].pos();
            let speed = hypot2(p1.x - p0.x, p1.z - p0.z) / dt.max(1e-4);
            let e = &mut self.entities[ei];
            let k = e.n("lookK");
            e.set_n("lookK", k + ((if e.state == "hunt" { 1.0 } else { 0.0 }) - k) * 1f64.min(dt * 4.0));
            // footfalls on the hard floor, louder the faster it comes
            e.step_acc += dt * speed;
            if e.step_acc > 1.4 {
                e.step_acc = 0.0;
                self.noise_burst(0.06, 900.0, 1.2, clamp(0.35 - d * 0.018, 0.02, 0.35), "bandpass", None, 0.0, 0.0);
            }
            if self.entities[ei].state == "hunt" && !hidden && d < KILL_DIST && !self.hands_taken() {
                self.on_death("entity");
                return true;
            }
        }
        let e = &mut self.entities[ei];
        let yaw = e.yaw;
        e.g.rotation().y = yaw;
        // the walk (`human_gait`): a browsing amble by day; at night a long, stiff,
        // wrong stride with the arms hanging dead (searching, half of it). The head
        // turns to you when it looks, with a slow, curious tilt.
        let night_k = if night { if e.state == "hunt" { 1.0 } else { 0.55 } } else { 0.0 };
        let hk = or0(e.n("huntK"));
        e.set_n("huntK", hk + (night_k - hk) * (1.0 - (-dt * 2.5).exp()));
        let look = e.n("lookK");
        let st =
            GaitStyle { hunt: e.n("huntK"), glide: 0.0, reach: 0.0, look, shiver: 0.0, tilt: (t * 0.7).sin() * 0.12 * look, breath: 1.0 };
        self.human_gait(ei, dt, t, &HUMAN_RIG_STAFF, &st);
        false
    }
}
