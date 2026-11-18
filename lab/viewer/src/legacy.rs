//! The Smiler as the game animates it today: `entities::smiler::animate`
//! (crates/game/src/entities/smiler.rs), copied here line for line with the
//! game's state replaced by plain inputs, so the lab can show the old and the
//! new on the same path, the same body and the same light. Nothing here
//! changes the game.
use engine::prelude::*;
use game::entities::body::{BodyRig, build_body, two_bone};
use std::collections::HashMap;

use crate::smiler::Mode;

const WINDUP: f64 = 0.5;

#[derive(Clone, Debug, Default)]
struct Limb {
    at: [f64; 3],
    from: [f64; 3],
    to: [f64; 3],
    swing: f64,
    planted: bool,
}

pub struct Legacy {
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

impl Legacy {
    pub fn new() -> Option<Legacy> {
        let rig = build_body("smiler", &game::entities::smiler::material)?;
        Some(Legacy { rig, ease: HashMap::new(), limbs: vec![], phase: 0.0 })
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

    /// One frame: at `pos`, heading `yaw`, moving at (`vx`, `vz`), in `mode`
    /// for `mode_t` seconds, the player at `pp`; `t` the game's clock.
    #[expect(clippy::too_many_arguments, reason = "the game's state, passed plainly")]
    pub fn animate(&mut self, pos: Vec3, yaw: f64, vx: f64, vz: f64, mode: Mode, mode_t: f64, pp: Vec3, t: f64, dt: f64) {
        let rig = self.rig.clone();
        let root = rig.root.clone();
        root.set_pos(pos.x, pos.y, pos.z);
        root.rotation().y = yaw;
        root.update_matrix_world(true);
        let speed = hypot2(vx, vz);
        let killer = mode == Mode::Over;
        rig.rest();
        let lunge = self.eased("lunge", if mode == Mode::Strike && mode_t >= WINDUP && !killer { 1.0 } else { 0.0 }, 9.0, dt);
        let stare = self.eased("stare", if killer { 1.0 } else { 0.0 }, 6.0, dt);
        let coil = self.eased("coil", if mode == Mode::Strike && mode_t < WINDUP { 1.0 } else { 0.0 }, 10.0, dt);
        let hunt = self.eased("hunt", if mode == Mode::Hunt { 1.0 } else { 0.0 }, 3.0, dt);
        // ---- the trunk: a slow sway at rest, lower and forward when it comes
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
        // ---- the head: on you (or your light) first; it turns in snaps
        let want = wrap_pi((pp.x - pos.x).datan2(pp.z - pos.z) - yaw).clamp(-1.4, 1.4);
        let look = if matches!(mode, Mode::Alert | Mode::Hunt | Mode::Strike) || killer { want } else { (t * 0.23).dsin() * 0.3 };
        let held = self.ease.get("head_yaw").copied().unwrap_or(0.0);
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
        // ---- the arms: hanging dead; forward, reaching, in a lunge
        for side in [".L", ".R"] {
            let sx = if side == ".L" { 1.0 } else { -1.0 };
            if let (Some(ua), Some(fa)) = (rig.ix(&format!("upperarm{side}")), rig.ix(&format!("forearm{side}"))) {
                let swing = (speed * 0.08).min(0.2) * (t * 3.0 + sx).dsin();
                let r = rig.rest_dir[ua];
                let reach = v3(sx * 0.15, -0.1, 1.0);
                let hang = v3(r.x, r.y, r.z + swing);
                rig.aim(
                    ua,
                    v3(hang.x + (reach.x - hang.x) * lunge, hang.y + (reach.y - hang.y) * lunge, hang.z + (reach.z - hang.z) * lunge),
                    0.0,
                );
                let rf = rig.rest_dir[fa];
                rig.aim(fa, v3(rf.x + (reach.x - rf.x) * lunge, rf.y + (reach.y - rf.y) * lunge, rf.z + (reach.z - rf.z) * lunge), 0.0);
            }
        }
        // ---- the legs: feet planted; each step taken in three snaps
        if self.limbs.len() != 2 {
            self.limbs = vec![Limb::default(); 2];
        }
        let stride = 1.1;
        self.phase = (self.phase + speed * dt / stride).fract();
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
            if !limb.planted || hypot2(limb.at[0] - home_w.x, limb.at[2] - home_w.z) > 1.4 {
                limb.at = [home_w.x, rest_end.y, home_w.z];
                limb.planted = true;
            }
            let swinging = speed > 0.15 && lp > 0.6;
            if swinging {
                let s = (lp - 0.6) / 0.4;
                if limb.swing == 0.0 {
                    limb.from = limb.at;
                    let l = speed.max(1e-6);
                    limb.to = [home_w.x + vx / l * stride * 0.5, rest_end.y, home_w.z + vz / l * stride * 0.5];
                }
                limb.swing = s.max(1e-3);
                // wrong mechanics: the foot moves in three held snaps
                let st = ((s * 3.0).floor() / 3.0 + 0.15 * (s * 3.0).fract()).min(1.0);
                for a in 0..3 {
                    limb.at[a] = limb.from[a] + (limb.to[a] - limb.from[a]) * st;
                }
                limb.at[1] = rest_end.y + (s * PI).dsin() * 0.1;
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
        rig.sync();
    }
}
