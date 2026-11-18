//! The creatures as the game animated them before their motion came from
//! clips: each kind's `animate` at 1718a5b (crates/game/src/entities/*.rs),
//! copied here line for line with the game's state replaced by plain inputs
//! ([`Inputs`]), so the lab can show the old and the new on the same path,
//! the same body and the same light. Nothing here is the game's any more.
use engine::prelude::*;
use game::entities::body::{BodyRig, build_body, two_bone};
use game::entities::brain::Mode;
use game::entities::glass::AtGlass;
use game::entities::reader::Page;
use std::collections::HashMap;

/// What the old animation read of the game, each frame.
#[derive(Clone, Copy, Debug)]
pub struct Inputs {
    pub pos: Vec3,
    pub yaw: f64,
    pub vel: (f64, f64),
    pub mode: Mode,
    pub mode_t: f64,
    /// the player (what it watches)
    pub pp: Vec3,
    pub killer: bool,
    /// the game's clock
    pub t: f64,
    pub dt: f64,
    pub chase_t: f64,
    pub page: Page,
    pub glass: AtGlass,
}

#[derive(Clone, Debug, Default)]
struct Limb {
    at: [f64; 3],
    from: [f64; 3],
    to: [f64; 3],
    swing: f64,
    planted: bool,
}

pub struct Legacy {
    pub kind: &'static str,
    pub rig: BodyRig,
    ease: HashMap<&'static str, f64>,
    limbs: Vec<Limb>,
    phase: f64,
}

fn hypot2(a: f64, b: f64) -> f64 {
    a.dhypot(b)
}

fn wrap_pi(a: f64) -> f64 {
    (a + PI).rem_euclid(2.0 * PI) - PI
}

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

fn mix(a: Vec3, b: Vec3, k: f64) -> Vec3 {
    v3(a.x + (b.x - a.x) * k, a.y + (b.y - a.y) * k, a.z + (b.z - a.z) * k)
}

impl Legacy {
    pub fn new(kind: &'static str) -> Option<Legacy> {
        let rig = build_body(kind, &crate::creatures::material(kind))?;
        Some(Legacy { kind, rig, ease: HashMap::new(), limbs: vec![], phase: 0.0 })
    }

    pub fn reset(&mut self) {
        self.ease.clear();
        self.limbs.clear();
        self.phase = 0.0;
    }

    fn eased(&mut self, key: &'static str, target: f64, rate: f64, dt: f64) -> f64 {
        let v = self.ease.entry(key).or_insert(target);
        *v += (target - *v) * (dt * rate).min(1.0);
        *v
    }

    fn get(&self, key: &'static str) -> f64 {
        self.ease.get(key).copied().unwrap_or(0.0)
    }

    /// One frame of the old animation.
    pub fn animate(&mut self, i: &Inputs) {
        let root = self.rig.root.clone();
        root.set_pos(i.pos.x, i.pos.y, i.pos.z);
        root.rotation().y = i.yaw;
        root.update_matrix_world(true);
        match self.kind {
            "smiler" => self.smiler(i),
            "hound" => self.hound(i),
            "corner" => self.corner(i),
            "reader" => self.reader(i),
            "glass" => self.glass(i),
            _ => {}
        }
        self.rig.sync();
    }

    /// Two legs on planted feet: each step swung (or snapped) from where it
    /// stood to half a stride ahead; `lift` the swing's height, `swing_from`
    /// where in the cycle a foot leaves the floor, `reset` how far a foot may
    /// lag before it is put back under the hip, `snaps` the Smiler's held thirds.
    #[allow(clippy::too_many_arguments, reason = "the old code's knobs")]
    fn biped(&mut self, i: &Inputs, stride: f64, swing_from: f64, lift: f64, reset: f64, min_speed: f64, snaps: bool) {
        let rig = self.rig.clone();
        let root = rig.root.clone();
        let (vx, vz) = i.vel;
        let speed = hypot2(vx, vz);
        if self.limbs.len() != 2 {
            self.limbs = vec![Limb::default(); 2];
        }
        self.phase = (self.phase + speed * i.dt / stride).fract();
        let phase = self.phase;
        for (k, side) in [".L", ".R"].iter().enumerate() {
            let (Some(it), Some(is), Some(ifo)) =
                (rig.ix(&format!("thigh{side}")), rig.ix(&format!("shin{side}")), rig.ix(&format!("foot{side}")))
            else {
                continue;
            };
            let rest_end = rest_head(&rig, ifo);
            let home_w = body_to_world(&root, v3(rest_end.x, 0.0, rest_end.z));
            let lp = (phase + k as f64 * 0.5).fract();
            let limb = &mut self.limbs[k];
            if !limb.planted || hypot2(limb.at[0] - home_w.x, limb.at[2] - home_w.z) > reset {
                limb.at = [home_w.x, rest_end.y, home_w.z];
                limb.planted = true;
            }
            if speed > min_speed && lp > swing_from {
                let s = (lp - swing_from) / (1.0 - swing_from);
                if limb.swing == 0.0 {
                    limb.from = limb.at;
                    let l = speed.max(1e-6);
                    limb.to = [home_w.x + vx / l * stride * 0.5, rest_end.y, home_w.z + vz / l * stride * 0.5];
                }
                limb.swing = s.max(1e-3);
                let st = if snaps { ((s * 3.0).floor() / 3.0 + 0.15 * (s * 3.0).fract()).min(1.0) } else { s };
                for a in 0..3 {
                    limb.at[a] = limb.from[a] + (limb.to[a] - limb.from[a]) * st;
                }
                limb.at[1] = rest_end.y + (s * PI).dsin() * lift;
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

    fn smiler(&mut self, i: &Inputs) {
        const WINDUP: f64 = 0.5;
        let (mode, mode_t, t, dt, pp, killer) = (i.mode, i.mode_t, i.t, i.dt, i.pp, i.killer);
        let speed = hypot2(i.vel.0, i.vel.1);
        let rig = self.rig.clone();
        let root = rig.root.clone();
        rig.rest();
        let lunge = self.eased("lunge", if mode == Mode::Strike && mode_t >= WINDUP && !killer { 1.0 } else { 0.0 }, 9.0, dt);
        let stare = self.eased("stare", if killer { 1.0 } else { 0.0 }, 6.0, dt);
        let coil = self.eased("coil", if mode == Mode::Strike && mode_t < WINDUP { 1.0 } else { 0.0 }, 10.0, dt);
        let hunt = self.eased("hunt", if mode == Mode::Hunt { 1.0 } else { 0.0 }, 3.0, dt);
        let hips = rig.ix("hips").unwrap();
        let h0 = rest_head(&rig, hips);
        let sway = (t * 1.3).dsin() * 0.01 * (1.0 - hunt);
        rig.bones[hips].set_pos(h0.x + sway, h0.y - 0.08 * coil - 0.03 * hunt, h0.z);
        if let Some(sp) = rig.ix("spine") {
            rig.turn(sp, &Quat::from_euler(&Euler::new(0.15 * hunt + 0.5 * lunge - 0.2 * coil, 0.0, (t * 0.7).dsin() * 0.03)));
        }
        if let Some(ch) = rig.ix("chest") {
            rig.turn(ch, &Quat::from_euler(&Euler::new((t * 1.1).dsin() * 0.02 + 0.2 * lunge, 0.0, 0.0)));
        }
        let yaw = root.rotation().y;
        let pos = root.pos();
        let want = wrap_pi((pp.x - pos.x).datan2(pp.z - pos.z) - yaw).clamp(-1.4, 1.4);
        let look = if matches!(mode, Mode::Alert | Mode::Hunt | Mode::Strike) || killer { want } else { (t * 0.23).dsin() * 0.3 };
        let held = self.get("head_yaw");
        let snapped = if (look - held).abs() > 0.25 { look } else { held };
        self.ease.insert("head_yaw", snapped);
        if let Some(nk) = rig.ix("neck") {
            rig.turn(nk, &Quat::from_euler(&Euler::new(-0.3 * lunge + 0.35 * stare, snapped * 0.5, 0.0)));
        }
        if let Some(hd) = rig.ix("head") {
            rig.turn(hd, &Quat::from_euler(&Euler::new(-0.2 * lunge + 0.15 * stare, snapped * 0.5, (t * 0.4).dsin() * 0.06 + 0.3 * stare)));
        }
        if let Some(jw) = rig.ix("jaw") {
            rig.turn(jw, &Quat::from_euler(&Euler::new(0.12 * coil + 0.3 * lunge + 0.3 * stare, 0.0, 0.0)));
        }
        for side in [".L", ".R"] {
            let sx = if side == ".L" { 1.0 } else { -1.0 };
            if let (Some(ua), Some(fa)) = (rig.ix(&format!("upperarm{side}")), rig.ix(&format!("forearm{side}"))) {
                let swing = (speed * 0.08).min(0.2) * (t * 3.0 + sx).dsin();
                let r = rig.rest_dir[ua];
                let reach = v3(sx * 0.15, -0.1, 1.0);
                let hang = v3(r.x, r.y, r.z + swing);
                rig.aim(ua, mix(hang, reach, lunge), 0.0);
                let rf = rig.rest_dir[fa];
                rig.aim(fa, mix(rf, reach, lunge), 0.0);
            }
        }
        self.biped(i, 1.1, 0.6, 0.1, 1.4, 0.15, true);
    }

    fn hound(&mut self, i: &Inputs) {
        const WINDUP: f64 = 0.35;
        const LIMBS: [(&str, &str, &str, f64, f64); 4] = [
            ("thigh.L", "shin.L", "foot.L", 0.0, 1.0),
            ("upperarm.L", "forearm.L", "hand.L", 0.25, 1.0),
            ("thigh.R", "shin.R", "foot.R", 0.5, -1.0),
            ("upperarm.R", "forearm.R", "hand.R", 0.75, -1.0),
        ];
        let (mode, mode_t, t, dt, killer) = (i.mode, i.mode_t, i.t, i.dt, i.killer);
        let (vx, vz) = i.vel;
        let speed = hypot2(vx, vz);
        let rig = self.rig.clone();
        let run = self.eased("run", ((speed - 2.0) / 3.5).clamp(0.0, 1.0), 4.0, dt);
        let crouch = self.eased(
            "crouch",
            if mode == Mode::Alert || (mode == Mode::Strike && mode_t < WINDUP) || mode == Mode::Idle { 1.0 } else { 0.0 },
            6.0,
            dt,
        );
        let bay = self.eased("bay", if (mode == Mode::Hunt && i.chase_t < 0.9) || killer { 1.0 } else { 0.0 }, 8.0, dt);
        let stride = 0.9 + 0.7 * run;
        self.phase = (self.phase + speed * dt / stride).fract();
        let phase = self.phase;
        let duty = 0.72 - 0.25 * run;
        if self.limbs.len() != 4 {
            self.limbs = vec![Limb::default(); 4];
        }
        let root = rig.root.clone();
        rig.rest();
        let hips = rig.ix("hips").unwrap();
        let h0 = rest_head(&rig, hips);
        let bob = (phase * PI * 4.0).dsin() * (0.012 + 0.03 * run);
        rig.bones[hips].set_pos(h0.x, h0.y + bob - 0.11 * crouch, h0.z - 0.05 * crouch);
        let pitch = -0.06 * run + 0.18 * crouch;
        rig.turn(hips, &Quat::from_euler(&Euler::new(pitch, 0.0, (phase * PI * 2.0).dsin() * 0.05)));
        if let Some(sp) = rig.ix("spine") {
            rig.turn(sp, &Quat::from_euler(&Euler::new(-0.5 * pitch, 0.0, -(phase * PI * 2.0).dsin() * 0.04)));
        }
        let cock = if crouch > 0.5 { ((t * 1.3).floor() * 1.7).dsin() * 0.35 } else { 0.0 };
        if let Some(nk) = rig.ix("neck") {
            rig.turn(nk, &Quat::from_euler(&Euler::new(-0.5 * crouch - 0.9 * bay + (phase * PI * 4.0).dsin() * 0.05, 0.0, 0.0)));
        }
        if let Some(hd) = rig.ix("head") {
            rig.turn(hd, &Quat::from_euler(&Euler::new(-0.3 * bay, 0.0, cock * crouch)));
        }
        if let Some(jw) = rig.ix("jaw") {
            rig.turn(jw, &Quat::from_euler(&Euler::new(0.45 * bay, 0.0, 0.0)));
        }
        let yaw = root.rotation().y;
        let (sy, cy) = (yaw.dsin(), yaw.dcos());
        let (bvx, bvz) = (vx * cy - vz * sy, vx * sy + vz * cy);
        for k in 0..3 {
            if let Some(hb) = rig.ix(["hair.0", "hair.1", "hair.2"][k]) {
                let sway = (t * 2.3 + k as f64).dsin() * 0.04 * (1.0 + run);
                rig.aim(hb, v3(-bvx * 0.05 + sway, -1.0, -bvz * 0.05), 0.0);
            }
        }
        let fwd_w = v3(vx, 0.0, vz);
        for (k, (up, lo, end, off, side)) in LIMBS.iter().enumerate() {
            let (iu, il, ie) = (rig.ix(up).unwrap(), rig.ix(lo).unwrap(), rig.ix(end).unwrap());
            let rest_end = rest_head(&rig, ie);
            let home_w = body_to_world(&root, v3(rest_end.x, 0.0, rest_end.z));
            let lame = k == 0;
            let lp = (phase + off).fract();
            let limb = &mut self.limbs[k];
            if !limb.planted || hypot2(limb.at[0] - home_w.x, limb.at[2] - home_w.z) > 1.2 {
                limb.at = [home_w.x, rest_end.y, home_w.z];
                limb.planted = true;
            }
            let swinging = speed > 0.15 && lp > duty;
            if swinging {
                let s = (lp - duty) / (1.0 - duty);
                if limb.swing == 0.0 {
                    limb.from = limb.at;
                    let ahead = stride * 0.5 * if lame { 0.7 } else { 1.0 };
                    let mut d = fwd_w;
                    let l = d.length().max(1e-6);
                    d.multiply_scalar(ahead / l);
                    limb.to = [home_w.x + d.x, rest_end.y, home_w.z + d.z];
                }
                limb.swing = s.max(1e-3);
                let lift = if lame { 0.0 } else { (s * PI).dsin() * (0.07 + 0.05 * run) };
                for a in 0..3 {
                    limb.at[a] = limb.from[a] + (limb.to[a] - limb.from[a]) * s;
                }
                limb.at[1] = rest_end.y + lift;
            } else {
                if limb.swing > 0.0 {
                    limb.at = limb.to;
                }
                limb.swing = 0.0;
                limb.at[1] = rest_end.y;
            }
            let target = world_to_body(&root, v3(limb.at[0], limb.at[1], limb.at[2]));
            let (a, b) = (rig.rest_len[iu], rig.rest_len[il]);
            let base = rig.head(iu);
            let front = up.starts_with("upperarm");
            let pole = if front { v3(base.x + side * 0.4, base.y, base.z - 0.6) } else { v3(base.x, base.y + 0.2, base.z + 1.0) };
            let (du, dl) = two_bone(base, target, pole, a, b);
            rig.aim(iu, du, 0.0);
            rig.aim(il, dl, 0.0);
            rig.aim(ie, rig.rest_dir[ie], 0.0);
        }
    }

    fn corner(&mut self, i: &Inputs) {
        const TURN: f64 = 1.5;
        let (mode, mode_t, t, dt, pp, killer) = (i.mode, i.mode_t, i.t, i.dt, i.pp, i.killer);
        let rig = self.rig.clone();
        let root = rig.root.clone();
        rig.rest();
        if killer {
            let p = root.pos();
            root.rotation().y = (pp.x - p.x).datan2(pp.z - p.z);
        }
        let yaw = root.rotation().y;
        let pos = root.pos();
        let want = wrap_pi((pp.x - pos.x).datan2(pp.z - pos.z) - yaw);
        let head_yaw = if matches!(mode, Mode::Alert) {
            want.clamp(-2.6, 2.6) * (mode_t / TURN).clamp(0.0, 1.0)
        } else if matches!(mode, Mode::Hunt | Mode::Strike) || killer {
            want.clamp(-1.0, 1.0)
        } else {
            let h = self.get("head_yaw");
            h - h.signum() * (dt * 1.2).min(h.abs())
        };
        self.ease.insert("head_yaw", head_yaw);
        let scream = self.eased("scream", if matches!(mode, Mode::Hunt | Mode::Strike) || killer { 1.0 } else { 0.0 }, 6.0, dt);
        if let Some(nk) = rig.ix("neck") {
            rig.turn(nk, &Quat::from_euler(&Euler::new(-0.15 * scream, head_yaw * 0.45, 0.0)));
        }
        if let Some(hd) = rig.ix("head") {
            rig.turn(hd, &Quat::from_euler(&Euler::new(0.12 * (1.0 - scream) - 0.1 * scream, head_yaw * 0.55, 0.0)));
        }
        if let Some(jw) = rig.ix("jaw") {
            rig.turn(jw, &Quat::from_euler(&Euler::new(0.55 * scream, 0.0, 0.0)));
        }
        if let Some(ch) = rig.ix("chest") {
            rig.turn(ch, &Quat::from_euler(&Euler::new((t * 0.9).dsin() * 0.012, 0.0, 0.0)));
        }
        self.biped(i, 1.6, 0.5, 0.18, 1.8, 0.2, false);
    }

    fn reader(&mut self, i: &Inputs) {
        const STOOP: f64 = 0.8;
        const WINDUP: f64 = 0.8;
        let (mode, t, dt, pp, killer) = (i.mode, i.t, i.dt, i.pp, i.killer);
        let speed = hypot2(i.vel.0, i.vel.1);
        let rig = self.rig.clone();
        let root = rig.root.clone();
        let (stage, bt) = (i.page.stage, i.page.t);
        rig.rest();
        if killer {
            let p = root.pos();
            root.rotation().y = (pp.x - p.x).datan2(pp.z - p.z);
        }
        let yaw = root.rotation().y;
        let pos = root.pos();
        let want = wrap_pi((pp.x - pos.x).datan2(pp.z - pos.z) - yaw).clamp(-1.2, 1.2);
        let at_you = matches!(mode, Mode::Alert | Mode::Hunt | Mode::Strike) || killer;
        let up = self.eased("up", if at_you { 1.0 } else { 0.0 }, 7.0, dt);
        let lowered = self.eased("lowered", if killer || (mode == Mode::Strike) { 1.0 } else { 0.0 }, 1.0 / WINDUP * 2.2, dt);
        let stoop_w = if mode == Mode::Read && stage == 1.0 { (bt / STOOP * PI).dsin().max(0.0) } else { 0.0 };
        let stoop = self.eased("stoop", stoop_w, 12.0, dt);
        let hunt = self.eased("hunt", if mode == Mode::Hunt { 1.0 } else { 0.0 }, 4.0, dt);
        let hips = rig.ix("hips").unwrap();
        let h0 = rest_head(&rig, hips);
        rig.bones[hips].set_pos(h0.x, h0.y - 0.16 * stoop - 0.02 * hunt, h0.z);
        if let Some(sp) = rig.ix("spine") {
            rig.turn(sp, &Quat::from_euler(&Euler::new(0.45 * stoop + 0.1 * hunt, 0.0, 0.0)));
        }
        if let Some(ch) = rig.ix("chest") {
            rig.turn(ch, &Quat::from_euler(&Euler::new(0.35 * stoop + (t * 1.2).dsin() * 0.012, 0.0, 0.0)));
        }
        let held = self.eased("head_yaw", if at_you { want } else { 0.0 }, 5.0, dt);
        if let Some(nk) = rig.ix("neck") {
            rig.turn(nk, &Quat::from_euler(&Euler::new(-0.45 * up + 0.2 * stoop, held * 0.45, 0.0)));
        }
        if let Some(hd) = rig.ix("head") {
            let nod = if at_you { 0.0 } else { (t * 0.9).dsin() * 0.03 };
            rig.turn(hd, &Quat::from_euler(&Euler::new(-0.35 * up + nod, held * 0.5, 0.0)));
        }
        let whisper = if matches!(mode, Mode::Roam | Mode::Idle | Mode::Stalk | Mode::Read | Mode::Search) { 1.0 } else { 0.0 };
        if let Some(jw) = rig.ix("jaw") {
            let k = ((t * 23.0).dsin() * 0.5 + 0.5) * ((t * 7.3).dsin() * 0.5 + 0.5);
            rig.turn(jw, &Quat::from_euler(&Euler::new(0.05 * k * whisper + 0.26 * lowered, 0.0, 0.0)));
        }
        let line = (t * 1.9).fract();
        let (ey, ex) = if at_you {
            (held * 0.4, -0.25)
        } else if line < 0.8 {
            (-0.32 + 0.64 * (line / 0.8), 0.06 * ((t * 1.9).floor() % 6.0) / 6.0)
        } else {
            (0.32 - 0.64 * ((line - 0.8) / 0.2), 0.06 * ((t * 1.9).floor() % 6.0) / 6.0)
        };
        for side in ["eye.L", "eye.R"] {
            if let Some(e) = rig.ix(side) {
                rig.turn(e, &Quat::from_euler(&Euler::new(ex, ey, 0.0)));
            }
        }
        for side in [".L", ".R"] {
            let sx = if side == ".L" { 1.0 } else { -1.0 };
            let (Some(ua), Some(fa)) = (rig.ix(&format!("upperarm{side}")), rig.ix(&format!("forearm{side}"))) else { continue };
            let (ru, rf) = (rig.rest_dir[ua], rig.rest_dir[fa]);
            let down_u = v3(sx * 0.12, -1.0, 0.12);
            let down_f = v3(sx * 0.02, -1.0, 0.35);
            let reach = if side == ".R" { stoop } else { 0.0 };
            let floor_u = v3(-0.1, -0.7, 0.7);
            let floor_f = v3(-0.05, -0.9, 0.4);
            let k = lowered.max(reach);
            let (tu, tf) = if reach > lowered { (floor_u, floor_f) } else { (down_u, down_f) };
            rig.aim(ua, mix(ru, tu, k), 0.0);
            rig.aim(fa, mix(rf, tf, k), 0.0);
        }
        let stride = if speed > 2.5 { 1.4 } else { 0.7 };
        self.biped(i, stride, 0.5, if speed > 2.5 { 0.14 } else { 0.035 }, 1.6, 0.15, false);
    }

    fn glass(&mut self, i: &Inputs) {
        const WALK: f64 = 2.6;
        let (t, dt) = (i.t, i.dt);
        let at = &i.glass;
        let rig = self.rig.clone();
        rig.rest();
        let press = self.eased("press", if at.pressing { 1.0 } else { 0.0 }, 3.0, dt);
        if let Some(m) = rig.meshes.get("skin") {
            let mut inf = m.morph_influences();
            if !inf.is_empty() && (inf[0] - press).abs() > 1e-4 {
                inf[0] = press;
            }
        }
        let crouch = self.eased("crouch", if at.pass > 0.0 && at.pass < 1.0 { 1.0 } else { 0.0 }, 5.0, dt);
        let reach = self.eased("reach", if at.inside { 1.0 } else { 0.0 }, 3.0, dt);
        let breath = (t * 0.8).dsin() * 0.008;
        if let Some(ch) = rig.ix("chest") {
            rig.turn(ch, &Quat::from_euler(&Euler::new(0.35 * crouch + 0.12 * press + breath, 0.0, 0.0)));
        }
        if let Some(sp) = rig.ix("spine") {
            rig.turn(sp, &Quat::from_euler(&Euler::new(0.45 * crouch + 0.08 * press, 0.0, 0.0)));
        }
        if let Some(hd) = rig.ix("head") {
            rig.turn(hd, &Quat::from_euler(&Euler::new(-0.25 * crouch - 0.12 * press, 0.0, 0.0)));
        }
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
            let rest = rest_head(&rig, ih);
            let glass = at.hands.map_or(v3(sx * 0.19, 1.45, 0.4), |h| h[k]);
            let toward = v3(sx * 0.16, 1.25, 0.62);
            let sill = v3(sx * 0.24, base.y - 0.25, 0.45);
            let want = if crouch > 0.5 {
                sill
            } else if reach > press {
                toward
            } else {
                glass
            };
            let target = mix(rest, want, up);
            let pole = v3(base.x + sx * 0.15, base.y - 0.6, base.z - 0.25);
            let (du, dl) = two_bone(base, target, pole, rig.rest_len[iu], rig.rest_len[il]);
            rig.aim(iu, du, 0.0);
            rig.aim(il, dl, 0.0);
        }
        if at.moving {
            self.phase = (self.phase + WALK * dt / 1.1 / 2.0).fract();
        }
        let phase = self.phase;
        for (k, side) in [".L", ".R"].iter().enumerate() {
            let (Some(it), Some(is), Some(ifo)) =
                (rig.ix(&format!("thigh{side}")), rig.ix(&format!("shin{side}")), rig.ix(&format!("foot{side}")))
            else {
                continue;
            };
            let rest_end = rest_head(&rig, ifo);
            let lp = (phase + k as f64 * 0.5).fract() * std::f64::consts::TAU;
            let (fwd, lift) = if at.moving { (lp.dsin() * 0.26, lp.dcos().max(0.0) * 0.08) } else { (0.0, 0.0) };
            let target = v3(rest_end.x, rest_end.y + lift + crouch * 0.42, rest_end.z + fwd + crouch * 0.25);
            let base = rig.head(it);
            let pole = v3(base.x, base.y, base.z + 1.0);
            let (du, dl) = two_bone(base, target, pole, rig.rest_len[it], rig.rest_len[is]);
            rig.aim(it, du, 0.0);
            rig.aim(is, dl, 0.0);
            rig.aim(ifo, rig.rest_dir[ifo], 0.0);
        }
    }
}
