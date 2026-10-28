//! Entity behaviour: steering, the lore look-rules, the entity voices and
//! `update_entities`.
//!
//! The flow fields are per 4 m tile. A creature follows the field's descent a
//! few tiles ahead (8-way, never across a corner), aims at the furthest of
//! those points it can reach in a straight line with its body's width (or at
//! you, if you are in such a line), and drives a velocity toward the aim with
//! limited acceleration; its facing follows the velocity at a limited turn rate.
use crate::base::numeric::hypot2;
use crate::base::tables::*;
use crate::entities::director::Phase;
use crate::entities::*;
use crate::prelude::*;

pub const ENT_R: f64 = 0.3;
/// Per-kind motion limits (any other kind moves like the Smiler) → (acceleration m/s², turn rate rad/s)
pub fn ent_motion(kind: &str) -> (f64, f64) {
    match kind {
        "hound" => (16.0, 6.5),
        "watcher" => (40.0, 12.0),
        "staff" => (7.0, 3.6),
        "shadow" => (6.0, 3.2),
        _ => (9.0, 4.2),
    }
}
pub const STEER8: [(i32, i32); 8] = [(1, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (-1, 1), (1, -1), (-1, -1)];

/// `field[i]` (None past the end or without a field)
#[inline]
fn fld(field: Option<&Rc<Vec<i32>>>, i: usize) -> Option<i32> {
    field.and_then(|f| f.get(i).copied())
}

impl Game {
    pub fn line_walkable(&self, ax: f64, az: f64, bx: f64, bz: f64, r: f64) -> bool {
        let (dx, dz) = (bx - ax, bz - az);
        let n = (hypot2(dx, dz) / 0.3).ceil() as i64;
        for i in 1..=n {
            let i = i as f64;
            let n = n as f64;
            if self.e_circle_hits_wall(ax + dx * i / n, az + dz * i / n, r) {
                return false;
            }
        }
        true
    }

    /// Turn toward a heading: eased,
    /// wrapped, never faster than the creature can
    pub fn entity_turn(&mut self, ei: usize, target: f64, dt: f64, rate: Option<f64>) {
        let e = &mut self.entities[ei];
        let diff = wrap_pi(target - e.yaw);
        let max = rate.unwrap_or_else(|| ent_motion(&e.kind).1) * dt;
        e.yaw = wrap_pi(e.yaw + clamp(diff * (dt * 8.0).min(1.0), -max, max));
    }

    /// Pick the aim point; false when the field
    /// has nothing more to give (a wander goal reached, or no way down)
    pub fn entity_aim(&mut self, ei: usize, field: Option<&Rc<Vec<i32>>>, direct: Option<Vec3>) -> bool {
        let pos = self.entities[ei].pos();
        if let Some(d) = direct {
            if hypot2(d.x - pos.x, d.z - pos.z) < 14.0 && self.line_walkable(pos.x, pos.z, d.x, d.z, ENT_R) {
                let e = &mut self.entities[ei];
                e.set_n("aimX", d.x);
                e.set_n("aimZ", d.z);
                e.set_fl("aimFinal", true);
                return true;
            }
        }
        let (cx, cz) = self.e_nav_tile_at(pos.x, pos.z);
        if fld(field, self.t_idx(cx, cz)).map(|v| v <= 0).unwrap_or(false) {
            let Some(d) = direct else { return false };
            let e = &mut self.entities[ei];
            e.set_n("aimX", d.x);
            e.set_n("aimZ", d.z);
            e.set_fl("aimFinal", true);
            return true;
        }
        // the steering points (ten f32 slots): every slot read below is written first
        let mut pts = [0f32; 10];
        let (mut x, mut z) = (cx, cz);
        let mut d = fld(field, self.t_idx(cx, cz));
        let mut n = 0usize;
        while n < 5 && d.map(|v| v > 0).unwrap_or(false) {
            let (mut bx, mut bz, mut bd) = (-1, -1, d.unwrap());
            for (dx, dz) in STEER8 {
                let (nx, nz) = (x + dx, z + dz);
                if !self.in_grid(nx, nz) || self.is_wall_tile(nx, nz) {
                    continue;
                }
                if dx != 0 && dz != 0 && (self.is_wall_tile(x + dx, z) || self.is_wall_tile(x, z + dz)) {
                    continue;
                }
                if let Some(nd) = fld(field, self.t_idx(nx, nz)) {
                    if nd >= 0 && nd < bd {
                        bd = nd;
                        bx = nx;
                        bz = nz;
                    }
                }
            }
            if bx < 0 {
                break;
            }
            x = bx;
            z = bz;
            d = Some(bd);
            pts[n * 2] = self.wx(x) as f32;
            pts[n * 2 + 1] = self.wz(z) as f32;
            n += 1;
        }
        if n == 0 {
            return false;
        }
        // the path reaches your tile: the last point is you, not the tile's centre
        let to_target = d.map(|v| v <= 0).unwrap_or(false) && direct.is_some();
        if to_target {
            let dd = direct.unwrap();
            pts[(n - 1) * 2] = dd.x as f32;
            pts[(n - 1) * 2 + 1] = dd.z as f32;
        }
        let mut k = 0;
        let mut i = n - 1;
        while i > 0 {
            if self.line_walkable(pos.x, pos.z, pts[i * 2] as f64, pts[i * 2 + 1] as f64, ENT_R) {
                k = i;
                break;
            }
            i -= 1;
        }
        let e = &mut self.entities[ei];
        e.set_n("aimX", pts[k * 2] as f64);
        e.set_n("aimZ", pts[k * 2 + 1] as f64);
        e.set_fl("aimFinal", to_target && k == n - 1);
        true
    }

    /// One step of
    /// travel toward the field's goal (or `direct`, the thing it is after).
    /// Returns true when a wander goal is reached.
    pub fn entity_step_toward(
        &mut self,
        ei: usize,
        field: Option<std::rc::Rc<Vec<i32>>>,
        dt: f64,
        speed: f64,
        direct: Option<Vec3>,
    ) -> bool {
        let id = self.entities[ei].g.id();
        let same_field = match (self.ent_ext.get(&id).and_then(|x| x.aim_field.as_ref()), field.as_ref()) {
            (Some(a), Some(b)) => Rc::ptr_eq(a, b),
            (None, None) => true,
            _ => false,
        };
        let reaim = {
            let e = &self.entities[ei];
            let pos = e.pos();
            !same_field || !(e.n("aimT") > 0.0) || (!e.fl("aimFinal") && hypot2(e.n("aimX") - pos.x, e.n("aimZ") - pos.z) < 1.2)
        };
        if reaim {
            self.ent_ext.entry(id).or_default().aim_field = field.clone();
            self.entities[ei].set_n("aimT", 0.18);
            let ok = self.entity_aim(ei, field.as_ref(), direct);
            self.entities[ei].set_fl("aimOk", ok);
        } else {
            let e = &mut self.entities[ei];
            let v = e.n("aimT") - dt;
            e.set_n("aimT", v);
        }
        let aim_ok = self.entities[ei].fl("aimOk");
        if let Some(d) = direct {
            if aim_ok && self.entities[ei].fl("aimFinal") {
                // it keeps after you between re-aims
                let e = &mut self.entities[ei];
                e.set_n("aimX", d.x);
                e.set_n("aimZ", d.z);
            }
        }
        self.entity_drive(ei, dt, if aim_ok { speed } else { 0.0 });
        !aim_ok && direct.is_none()
    }

    /// Drive a velocity toward the aim with
    /// limited acceleration; what a wall takes off the step comes off the
    /// velocity too; the facing follows the velocity.
    pub fn entity_drive(&mut self, ei: usize, dt: f64, speed: f64) {
        let (accel, _) = ent_motion(&self.entities[ei].kind);
        let (mut tvx, mut tvz) = (0.0, 0.0);
        let pos = self.entities[ei].pos();
        if speed > 0.0 {
            let e = &self.entities[ei];
            let (dx, dz) = (e.n("aimX") - pos.x, e.n("aimZ") - pos.z);
            let dl = hypot2(dx, dz);
            // corners are taken at speed; a final target is approached, not overrun
            let want = if dl < 1e-3 { 0.0 } else { speed * (if e.fl("aimFinal") { 1f64.min(dl / 0.6) } else { 1.0 }) };
            if want > 0.0 {
                tvx = dx / dl * want;
                tvz = dz / dl * want;
            }
        }
        let (vx, vz, burst) = {
            let e = &self.entities[ei];
            (e.vx, e.vz, e.n("lungeT") > 0.0 || e.state == "hunt" && e.kind == "hound")
        };
        // a lunge or a charge is a burst: twice the push
        let a = accel * (if burst { 2.0 } else { 1.0 }) * dt;
        let (mut ax, mut az) = (tvx - vx, tvz - vz);
        let al = hypot2(ax, az);
        if al > a {
            ax *= a / al;
            az *= a / al;
        }
        let (nvx, nvz) = (vx + ax, vz + az);
        let (ox, oz) = (pos.x, pos.z);
        let mut p = pos;
        if nvx != 0.0 || nvz != 0.0 {
            self.e_collide_move(&mut p, nvx * dt, nvz * dt, ENT_R);
        }
        let e = &mut self.entities[ei];
        e.g.set_pos_v(&p);
        e.vx = nvx;
        e.vz = nvz;
        if dt > 0.0 {
            e.vx = (p.x - ox) / dt;
            e.vz = (p.z - oz) / dt;
        }
        if hypot2(e.vx, e.vz) > 0.15 {
            let h = e.vx.atan2(e.vz);
            self.entity_turn(ei, h, dt, None);
        }
    }

    /// → speed multiplier. Smiler: hold its
    /// gaze and back away slowly and it barely creeps; turn your back on it, or
    /// run, and it lunges. Hound: staring it down buys a few seconds (not
    /// often) while it growls.
    pub fn entity_look_rule(&mut self, ei: usize, d: f64, seen: bool, dt: f64) -> f64 {
        let f = self.player.fwd();
        let pos = self.entities[ei].pos();
        let pp = self.player.pos;
        let (tx0, tz0) = (pos.x - pp.x, pos.z - pp.z);
        let l = {
            let h = hypot2(tx0, tz0);
            if h != 0.0 && !h.is_nan() {
                h
            } else {
                1.0
            }
        };
        // whether *you* are looking at it: your eyes, your range
        let in_cone = (tx0 * f.x + tz0 * f.z) / l > 0.82;
        let watched = in_cone && d < 26.0 && (seen || self.e_los(pp.x, pp.z, pos.x, pos.z));
        let running = self.player.audible_range >= 12.0;
        let kind = self.entities[ei].kind.clone();
        if kind == "smiler" {
            let e = &mut self.entities[ei];
            let was = e.fl("wasWatched");
            e.set_fl("wasWatched", watched);
            let lt = 0f64.max(e.n("lungeT") - dt);
            e.set_n("lungeT", lt);
            if watched && !running {
                return 0.28;
            }
            if was && !watched && d < 11.0 {
                e.set_n("lungeT", 1.4);
                self.entity_call(ei, "lunge");
            }
            let e = &self.entities[ei];
            return if e.n("lungeT") > 0.0 {
                1.7
            } else if running {
                1.25
            } else {
                1.0
            };
        }
        if kind == "hound" {
            let e = &mut self.entities[ei];
            let cd = 0f64.max(e.n("stareCd") - dt);
            e.set_n("stareCd", cd);
            if e.n("holdT") > 0.0 {
                let h = e.n("holdT") - dt;
                e.set_n("holdT", h);
                if h <= 0.0 {
                    e.set_n("stareCd", 12.0);
                }
                return 0.0;
            }
            if watched && d < 9.0 && e.n("stareCd") <= 0.0 {
                e.set_n("holdT", 2.8);
                self.entity_call(ei, "growl");
                return 0.0;
            }
            return 1.0;
        }
        1.0
    }

    /// 'lunge' | 'growl' | 'bark', from the entity's voice slot
    pub fn entity_call(&mut self, ei: usize, what: &str) {
        if crate::entities::creature::on_new_brain(&self.entities[ei].kind) {
            return crate::entities::creature::kind_call(self, ei, what);
        }
        let Some(s) = self.soundscape() else { return };
        let id = self.entities[ei].g.id();
        let Some(v) = self.ent_ext.get(&id).and_then(|x| x.voice.as_ref()).map(|v| v.slot.input.clone()) else { return };
        let t = self.ac_time();
        match what {
            "lunge" => {
                s.burst(&v, t, 0.5, "bandpass", 900.0, 2.0, 0.2, Some(2400.0), 0.01);
                s.tone(&v, t, 0.4, 180.0, 0.08, "sawtooth", Some(90.0), 0.003);
            }
            "growl" => {
                s.tone(&v, t, 2.4, 52.0, 0.14, "sawtooth", Some(44.0), 0.2);
                s.burst(&v, t, 2.4, "lowpass", 320.0, 1.5, 0.09, Some(220.0), 0.2);
            }
            "shriek" => {
                if self.entities[ei].kind == "hound" {
                    // a wet snarl that climbs into a yelp
                    s.tone(&v, t, 0.7, 96.0, 0.2, "sawtooth", Some(70.0), 0.02);
                    s.burst(&v, t, 0.7, "lowpass", 700.0, 1.2, 0.18, Some(420.0), 0.02);
                    s.burst(&v, t + 0.5, 0.35, "bandpass", 1600.0, 4.0, 0.22, Some(2600.0), 0.01);
                } else {
                    // a sharp wet inhale through the teeth, then the scream
                    s.burst(&v, t, 0.3, "bandpass", 700.0, 2.5, 0.16, Some(2600.0), 0.25);
                    s.tone(&v, t + 0.32, 0.9, 340.0, 0.1, "sawtooth", Some(980.0), 0.01);
                    s.tone(&v, t + 0.32, 0.9, 347.0, 0.08, "square", Some(1010.0), 0.01);
                    s.burst(&v, t + 0.3, 1.1, "bandpass", 2100.0, 1.4, 0.32, Some(3900.0), 0.01);
                    for k in 0..6 {
                        s.burst(&v, t + 1.25 + k as f64 * 0.055, 0.02, "highpass", 3400.0, 1.0, 0.07, None, 0.001);
                    }
                }
            }
            "listen" => {
                if self.entities[ei].kind == "hound" {
                    // sniffing
                    for k in 0..3 {
                        s.burst(&v, t + k as f64 * 0.16, 0.08, "bandpass", 2600.0, 2.0, 0.05, Some(1800.0), 0.01);
                    }
                } else {
                    // teeth, and a slow breath in
                    for k in 0..2 {
                        s.burst(&v, t + k as f64 * 0.09, 0.02, "highpass", 3200.0, 1.0, 0.06, None, 0.001);
                    }
                    s.burst(&v, t + 0.3, 1.2, "lowpass", 380.0, 0.9, 0.06, Some(520.0), 0.6);
                }
            }
            "bark" => {
                for k in 0..2 {
                    let k = k as f64;
                    s.burst(&v, t + k * 0.28, 0.16, "bandpass", 700.0, 3.0, 0.3, Some(380.0), 0.005);
                    s.tone(&v, t + k * 0.28, 0.14, 240.0, 0.12, "square", Some(150.0), 0.003);
                }
            }
            _ => {}
        }
    }

    /// Each thing has its own sound,
    /// placed in 3D and muffled when a wall is between you
    pub fn update_entity_voice(&mut self, ei: usize, d: f64, dt: f64, seen: bool) {
        let Some(s) = self.soundscape() else { return };
        if self.ac.is_none() {
            return;
        }
        let id = self.entities[ei].g.id();
        if self.ent_ext.get(&id).and_then(|x| x.voice.as_ref()).is_none() {
            let slot = s.voice_slot(2.2);
            let next = rand2(0.5, 3.0);
            self.ent_ext.entry(id).or_default().voice = Some(EntVoice { slot, next, was_hunting: false });
        }
        let pos = self.entities[ei].pos();
        let pp = self.player.pos;
        let kind = self.entities[ei].kind.clone();
        let slot = self.ent_ext[&id].voice.as_ref().unwrap().slot.clone();
        slot.at(pos.x, if kind == "hound" { 0.8 } else { 1.9 }, pos.z);
        if d < 26.0 {
            let clear = seen || (d < 12.0 && self.e_los(pos.x, pos.z, pp.x, pp.z));
            slot.occlude(if clear { 0.0 } else { 1.0 });
        }
        let hunting = self.entities[ei].state == "hunt";
        let was = self.ent_ext[&id].voice.as_ref().unwrap().was_hunting;
        if kind == "hound" && hunting && !was {
            self.entity_call(ei, "bark");
        }
        let next = {
            let v = self.ent_ext.get_mut(&id).unwrap().voice.as_mut().unwrap();
            v.was_hunting = hunting;
            v.next -= dt;
            v.next
        };
        if next > 0.0 || d > 30.0 {
            return;
        }
        let (v, t) = (slot.input.clone(), self.ac_time());
        let new_next;
        if kind == "smiler" {
            // slow wet breathing, and teeth when it is coming
            new_next = if hunting { rand2(1.1, 1.6) } else { rand2(2.4, 3.8) };
            s.burst(&v, t, 0.9, "lowpass", 420.0, 0.9, 0.05, Some(260.0), 0.35);
            s.burst(&v, t + 1.0, 0.7, "lowpass", 520.0, 0.9, 0.035, Some(300.0), 0.12);
            if hunting {
                for k in 0..3 {
                    s.burst(&v, t + 0.2 + k as f64 * 0.07, 0.025, "highpass", 3200.0, 1.0, 0.05, None, 0.002);
                }
            }
        } else if kind == "hound" {
            // panting when it runs, a low whine when it circles
            if hunting {
                new_next = 0.34;
                s.burst(&v, t, 0.18, "bandpass", 1300.0, 1.4, 0.07, Some(800.0), 0.02);
            } else {
                new_next = rand2(4.0, 8.0);
                s.tone(&v, t, 1.1, rand2(620.0, 760.0), 0.03, "sine", Some(480.0), 0.2);
            }
        } else {
            // the watcher makes no sound; close up, your ears ring
            new_next = rand2(4.0, 7.0);
            if d < 7.0 {
                s.tone(&s.dry, t, 2.2, 5200.0 + rand2(-200.0, 200.0), 0.006 * (1.0 - d / 7.0), "sine", None, 0.6);
            }
        }
        self.ent_ext.get_mut(&id).unwrap().voice.as_mut().unwrap().next = new_next;
    }

    /// Whether you are inside creature `ei`'s field of view (a wide cone
    /// ahead of it, or right beside it). A hunting creature is locked on to
    /// you and tracks you without it; a lit flashlight aimed at it is seen
    /// from any side.
    pub fn entity_faces_player(&self, ei: usize, d: f64) -> bool {
        let e = &self.entities[ei];
        if e.state == "hunt" || d <= 3.5 {
            return true;
        }
        let pos = e.pos();
        let pp = self.player.pos;
        let (dx, dz) = ((pp.x - pos.x) / d, (pp.z - pos.z) / d);
        if e.yaw.sin() * dx + e.yaw.cos() * dz >= 0.42 {
            return true;
        }
        let f = self.player.fwd();
        self.player.flashlight_on && -(f.x * dx + f.z * dz) > 0.9
    }

    /// Footfalls from where the creature walks, one per stride, loud in a
    /// chase and soft while it searches or prowls.
    fn entity_footfalls(&mut self, ei: usize, dt: f64, speed: f64, d: f64) {
        if d > 34.0 {
            return;
        }
        let stride = if self.entities[ei].kind == "hound" { 0.9 } else { 1.6 };
        self.entities[ei].step_acc += dt * speed;
        if self.entities[ei].step_acc > stride {
            self.entities[ei].step_acc = 0.0;
            let hunting = self.entities[ei].state == "hunt";
            let loud = if hunting { clamp(1.25 - d / 22.0, 0.35, 1.0) } else { clamp(0.75 - d / 40.0, 0.15, 0.5) };
            self.entity_steps(ei, 1, loud);
        }
    }

    pub fn update_entities(&mut self, dt: f64, t: f64) {
        self.update_entities_body(dt, t);
        self.sync_entity_skins();
    }

    /// The body of `update_entities` (its early `return`s land here).
    fn update_entities_body(&mut self, dt: f64, t: f64) {
        self.player_field_t -= dt;
        if self.player_field_t <= 0.0 {
            self.player_field_t = 0.35;
            let (ptx, ptz) = self.e_nav_tile_at(self.player.pos.x, self.player.pos.z);
            self.player_field = Some(Rc::new(self.e_bfs(ptx, ptz)));
        }
        let mut nearest: f64 = 1e9;
        let mut any_hunt = false;
        let mut next = 0;
        while next < self.entities.len() {
            let ei = next;
            next += 1;
            {
                let e = &self.entities[ei];
                let y = e.yaw;
                e.g.rotation().y = y;
            }
            self.update_face(ei); // billboard face, always watching you

            let pos = self.entities[ei].pos();
            let d = hypot2(pos.x - self.player.pos.x, pos.z - self.player.pos.z);
            let kind = self.entities[ei].kind.clone();
            // the Showroom's staff run their own rules (`update_staff`) and rig
            if (kind == "staff" || kind == "shadow") && self.entities[ei].freeze > 0.0 {
                self.entities[ei].freeze -= dt;
                continue;
            }
            if kind == "staff" {
                if self.update_staff(ei, dt, t, d) {
                    return;
                }
                if self.entities[ei].state == "hunt" {
                    any_hunt = true;
                    nearest = nearest.min(d);
                }
                continue;
            }
            // the Borrowed House's shadows (`update_shadow`); 'gone' ones have come apart
            if kind == "shadow" || kind == "gone" {
                if self.entities[ei].kind == "shadow" && self.update_shadow(ei, dt, t, d) {
                    return;
                }
                if self.entities[ei].kind == "shadow" && self.entities[ei].state == "hunt" {
                    any_hunt = true;
                    nearest = nearest.min(d);
                }
                continue;
            }
            nearest = nearest.min(d);
            // the creatures of the redesign run their own brain (`creature`)
            if crate::entities::creature::on_new_brain(&kind) {
                if self.entities[ei].freeze > 0.0 {
                    // held still (a showcase, a level's first seconds): it only breathes
                    self.entities[ei].freeze -= dt;
                    self.entity_drive(ei, dt, 0.0);
                    crate::entities::creature::kind_animate(self, ei, dt);
                    if let Some(c) = self.creature(ei) {
                        c.body.sync();
                    }
                    continue;
                }
                self.update_creature(ei, dt);
                if self.state != "playing" {
                    return;
                }
                if self.entities[ei].state == "hunt" {
                    any_hunt = true;
                }
                let los = d < 26.0 && self.e_los(pos.x, pos.z, self.player.pos.x, self.player.pos.z);
                if los && d < 22.0 && !self.director.sighted.contains(&kind) {
                    let mut to_e = v3(pos.x - self.player.pos.x, 0.0, pos.z - self.player.pos.z);
                    to_e.normalize();
                    let visible = d < 8.0 || self.player.flashlight_on || self.e_region_light_at(self.tx(pos.x), self.tz(pos.z)) > 0.25;
                    if visible && self.player.fwd().dot(&to_e) > 0.8 {
                        self.entity_sighted(ei, d);
                    }
                }
                crate::entities::creature::kind_voice(self, ei, dt, d);
                continue;
            }
            {
                let e = &mut self.entities[ei];
                let nt = e.n("noticeT");
                if nt > 0.0 {
                    e.set_n("noticeT", (nt - dt).max(0.0));
                }
            }
            self.animate_entity(ei, dt, t, d); // joints, gait, jaw, head tracking
            if self.entities[ei].freeze > 0.0 {
                self.entities[ei].freeze -= dt;
                self.entity_drive(ei, dt, 0.0);
                continue;
            } // it comes to a stop, not a halt

            // drawn to light: a lit lamp is seen from much further off in the dark
            let sight_range = if kind != "watcher" && self.player.flashlight_on { 17.0 } else { 12.0 };
            let los = d < 26.0 && self.e_los(pos.x, pos.z, self.player.pos.x, self.player.pos.z);
            // its eyes look forward: out of the cone you can pass it unnoticed
            let seen = d < sight_range && los && (kind == "watcher" || self.entity_faces_player(ei, d));
            self.update_entity_voice(ei, d, dt, d < sight_range && los);
            // the first time you lay eyes on one, it hits you
            if los && d < 22.0 && !self.director.sighted.contains(&kind) {
                let mut to_e = v3(pos.x - self.player.pos.x, 0.0, pos.z - self.player.pos.z);
                to_e.normalize();
                let visible = d < 8.0 || self.player.flashlight_on || self.e_region_light_at(self.tx(pos.x), self.tz(pos.z)) > 0.25;
                if visible && self.player.fwd().dot(&to_e) > 0.8 {
                    self.entity_sighted(ei, d);
                }
            }
            // ---- watcher: only moves when NOT observed, doesn't kill, drains sanity ----
            if kind == "watcher" {
                let pp = self.player.pos;
                let mut to_e = v3(pos.x - pp.x, 0.0, pos.z - pp.z);
                to_e.normalize();
                let in_view = self.player.fwd().dot(&to_e) > 0.35;
                let observed = seen && in_view;
                {
                    let e = &mut self.entities[ei];
                    e.observed = observed || in_view; // its pose may only change out of view (`animate_entity`)
                    e.set_fl("observed", observed || in_view);
                }
                if observed {
                    let e = &mut self.entities[ei];
                    e.unseen_t = 0.0;
                    // face the player, don't move
                    e.vx = 0.0;
                    e.vz = 0.0;
                    let h = (pp.x - pos.x).atan2(pp.z - pos.z);
                    self.entity_turn(ei, h, dt, None);
                } else {
                    self.entities[ei].unseen_t += dt;
                    // creep toward player while unobserved; unless the director
                    // has sent it, slowly, and never closer than a room away
                    let between = self.director.stalker != Some(ei);
                    if let Some(pf) = self.player_field.clone() {
                        let sp = self.entities[ei].speed * if between { 0.35 } else { 1.0 };
                        if between && d < 18.0 {
                            self.entity_drive(ei, dt, 0.0);
                        } else {
                            self.entity_step_toward(ei, Some(pf), dt, sp, Some(self.player.pos));
                        }
                    }
                    // teleport closer if unseen too long (very unnerving), but
                    // only once it has been sent: it keeps the rhythm too
                    if self.entities[ei].unseen_t > 4.5 && d > 3.0 && !between {
                        let (ep, pp) = (self.entities[ei].pos(), self.player.pos);
                        let mut dir = v3(pp.x - ep.x, 0.0, pp.z - ep.z);
                        dir.normalize().multiply_scalar((d - 2.0).min(6.0));
                        let (nx, nz) = (ep.x + dir.x, ep.z + dir.z);
                        if !self.e_circle_hits_wall(nx, nz, 0.4) {
                            let e = &self.entities[ei];
                            e.g.position().x = nx;
                            e.g.position().z = nz;
                        }
                        self.entities[ei].unseen_t = 0.0;
                        if rnd() < 0.5 {
                            self.sfx_breath();
                        }
                    }
                }
                // extra sanity pressure when close (half of it unless it was sent)
                if d < 6.0 {
                    let k = if self.director.stalker != Some(ei) { 0.5 } else { 1.0 };
                    self.player.sanity = clamp(self.player.sanity - dt * 8.0 * k * (1.0 - d / 6.0), 0.0, SAN_MAX);
                }
                // touch = big sanity crash + hurt flash but not instant death
                if d < KILL_DIST + 0.2 {
                    self.player.sanity = clamp(self.player.sanity - 45.0, 0.0, SAN_MAX);
                    self.ui.flash.set_opacity(1.0);
                    self.schedule(350.0, |g| {
                        g.ui.flash.set_opacity(0.0);
                    });
                    // push the watcher back a bit so it doesn't spam-drain
                    let (ep, pp) = (self.entities[ei].pos(), self.player.pos);
                    let mut back = v3(ep.x - pp.x, 0.0, ep.z - pp.z);
                    back.normalize().multiply_scalar(3.0);
                    let (nx, nz) = (ep.x + back.x, ep.z + back.z);
                    if !self.e_circle_hits_wall(nx, nz, 0.4) {
                        let e = &self.entities[ei];
                        e.g.position().x = nx;
                        e.g.position().z = nz;
                    }
                    self.shake_t = 0.6;
                    if self.player.sanity <= 0.0 {
                        self.on_death("sanity");
                    } else {
                        self.director_strike(ei);
                    }
                }
                continue;
            }
            let pp = self.player.pos;
            let relaxed = self.director.phase == Phase::Relax;
            let hear_mul = (if kind == "hound" { 1.8 } else { 1.0 }) * if relaxed { 0.55 } else { 1.0 };
            if self.entities[ei].state == "wander" {
                // sent away by the director: it doesn't turn back for a while
                let retreat = self.entities[ei].n("retreatT");
                if retreat > 0.0 {
                    self.entities[ei].set_n("retreatT", retreat - dt);
                }
                let calm = retreat > 0.0;
                // seen → it has you; only heard → it comes to look. A bright-tile
                // player is harder to lock on to.
                let audible = !calm && d < self.player.audible_range * hear_mul;
                let player_lit = self.e_region_light_at(self.tx(pp.x), self.tz(pp.z));
                let dazzle = player_lit > 0.65 && d > 5.0 && rnd() < (player_lit - 0.5);
                if !calm && seen && !dazzle {
                    self.entity_notice(ei, d);
                    continue;
                } else if audible {
                    let a = rnd() * PI * 2.0;
                    let off = rand2(1.5, 4.0);
                    self.entity_search(ei, pp.x + a.cos() * off, pp.z + a.sin() * off, 6.0, 14.0);
                    self.entity_call(ei, "listen");
                } else {
                    let stalk = self.entities[ei].fl("stalk");
                    self.entities[ei].w_timer -= dt;
                    let reached = {
                        let e = &self.entities[ei];
                        let p = e.pos();
                        e.w_target.map(|(gx, gz)| self.tx(p.x) == gx && self.tz(p.z) == gz).unwrap_or(false)
                    };
                    let need_goal = !stalk && (self.entities[ei].w_field.is_none() || self.entities[ei].w_timer <= 0.0 || reached);
                    if need_goal {
                        // a wandering goal that prefers darker tiles, far from the
                        // player: up to 12 tries, the best score wins
                        self.entities[ei].w_timer = rand2(8.0, 14.0);
                        let mut best_score = -1e9;
                        let mut best_g: Option<(i32, i32)> = None;
                        for _ in 0..12 {
                            let gx = irand(1, self.w as i64 - 2) as i32;
                            let gz = irand(1, self.h as i64 - 2) as i32;
                            if self.is_wall_tile(gx, gz) {
                                continue;
                            }
                            let d_from_p = hypot2(self.wx(gx) - pp.x, self.wz(gz) - pp.z);
                            if d_from_p < 24.0 {
                                continue;
                            }
                            let lit = self.e_region_light_at(gx, gz);
                            // prefer darker (higher score when lit low), penalize far
                            let score = -lit * 3.0 - (d_from_p - 40.0).abs() * 0.05;
                            if score > best_score {
                                best_score = score;
                                best_g = Some((gx, gz));
                            }
                        }
                        if let Some((gx, gz)) = best_g {
                            let f = Rc::new(self.e_bfs(gx, gz));
                            let e = &mut self.entities[ei];
                            e.w_target = Some((gx, gz));
                            e.w_field = Some(f);
                        }
                    }
                    // a stalker that has arrived waits for its next goal, listening
                    let pace = if stalk && reached {
                        0.0
                    } else if calm {
                        self.entities[ei].speed * 0.7
                    } else if stalk {
                        // unhurried far off, slow and careful once near
                        if d > 30.0 {
                            3.8
                        } else if d > 12.0 {
                            2.5
                        } else {
                            1.8
                        }
                    } else {
                        1.7
                    };
                    if let Some(wf) = self.entities[ei].w_field.clone() {
                        if pace > 0.0 {
                            self.entity_step_toward(ei, Some(wf), dt, pace, None);
                        } else {
                            self.entity_drive(ei, dt, 0.0);
                        }
                    }
                    if pace > 0.0 {
                        self.entity_footfalls(ei, dt, pace, d);
                    }
                    // occasionally stop and stare
                    self.entities[ei].stare_t -= dt;
                    if self.entities[ei].stare_t <= 0.0 {
                        self.entities[ei].stare_t = rand2(5.0, 11.0);
                        if d < 15.0 && los && !stalk {
                            self.entities[ei].freeze = rand2(1.2, 2.4);
                        }
                    }
                }
            }
            if self.entities[ei].state == "search" {
                let audible = d < self.player.audible_range * hear_mul;
                if seen {
                    self.entity_notice(ei, d);
                    continue;
                }
                if audible {
                    // closer now: it knows roughly where
                    let e = &mut self.entities[ei];
                    e.set_n("seenX", pp.x + rand2(-1.5, 1.5));
                    e.set_n("seenZ", pp.z + rand2(-1.5, 1.5));
                    e.set_n("searchR", e.n("searchR").min(3.0));
                    e.set_n("searchT", e.n("searchT").max(7.0));
                }
                let st = self.entities[ei].n("searchT") - dt;
                self.entities[ei].set_n("searchT", st);
                if st <= 0.0 || relaxed && d > 14.0 {
                    let e = &mut self.entities[ei];
                    e.state = "wander".into();
                    e.w_field = None;
                    e.w_timer = 0.0;
                } else {
                    let before = self.entities[ei].pos();
                    self.search_step(ei, dt);
                    let after = self.entities[ei].pos();
                    let moved = hypot2(after.x - before.x, after.z - before.z);
                    if dt > 0.0 {
                        self.entity_footfalls(ei, dt, moved / dt, d);
                    }
                }
            }
            if self.entities[ei].state == "hunt" {
                any_hunt = true;
                // R769 hunt persists while the player is audibly moving or seen;
                // fully silent and unseen, interest decays faster
                let audible = d < self.player.audible_range;
                if seen || audible {
                    let e = &mut self.entities[ei];
                    e.lose_t = 0.0;
                    e.set_n("seenX", pp.x);
                    e.set_n("seenZ", pp.z);
                } else {
                    let decay_mul = if self.player.audible_range < 1.0 { 1.7 } else { 1.0 };
                    self.entities[ei].lose_t += dt * decay_mul;
                }
                if self.entities[ei].lose_t > 5.0 {
                    // it lost you: it goes where it last had you, and looks
                    let (lx, lz) = (self.entities[ei].n("seenX"), self.entities[ei].n("seenZ"));
                    self.entity_search(ei, lx, lz, 7.0, 16.0);
                    self.entity_drive(ei, dt, 0.0);
                } else if let Some(pf) = self.player_field.clone() {
                    // R769 speed penalty in bright tiles
                    let p = self.entities[ei].pos();
                    let lit_here = self.e_region_light_at(self.tx(p.x), self.tz(p.z));
                    // the first seconds after it finds you, it comes at you hard
                    let charge = self.entities[ei].n("chargeT");
                    if charge > 0.0 {
                        self.entities[ei].set_n("chargeT", charge - dt);
                    }
                    let burst = if charge > 0.0 { 1.2 } else { 1.0 };
                    let speed_mul = (1.0 - 0.35 * lit_here) * self.entity_look_rule(ei, d, seen, dt) * burst;
                    let sp = self.entities[ei].speed;
                    self.entity_step_toward(ei, Some(pf), dt, sp * speed_mul, Some(self.player.pos));
                    self.entity_footfalls(ei, dt, sp * speed_mul, d);
                }
                if d < KILL_DIST {
                    self.director.killer = Some(ei);
                    self.on_death("entity");
                    return;
                }
            }
        }
        self.update_entity_pressure(dt, nearest, any_hunt);
    }

    /// The tail of `update_entities`: audio and screen pressure
    /// from the nearest entity, the vignette's pulses, the sanity hue shift,
    /// the sagging hum, the stamina breath, the heartbeat and the sanity drain.
    fn update_entity_pressure(&mut self, dt: f64, nearest: f64, any_hunt: bool) {
        let wk = self.weather_key();
        if let Some(dg) = &self.audio.drone_gain {
            let target = if any_hunt {
                clamp(0.55 - nearest * 0.035, 0.06, 0.5)
            } else if nearest < 9.0 {
                0.05
            } else {
                0.0
            }
            .max(self.director.tension * 0.22);
            let g = dg.gain();
            g.set_value(g.value() + (target - g.value()) * (dt * 3.0).min(1.0));
        }
        let pressure = if any_hunt { clamp(1.0 - nearest / 16.0, 0.0, 1.0) } else { clamp(1.0 - nearest / 10.0, 0.0, 0.35) }
            .max(self.director.tension * 0.55);
        // R92/R95 hum pulse: subtle vignette breath driven by the safe-room LFO
        let lfo_hz = self.audio.safe_room_lfo.as_ref().map(|n| n.frequency().value()).filter(|v| *v != 0.0 && !v.is_nan()).unwrap_or(0.35);
        self.hum_pulse_phase += dt * lfo_hz * PI * 2.0 * self.descent_freq_boost();
        let pulse_amp_target = vignette_pulse_amp_by_weather(wk).unwrap_or(VIGNETTE_PULSE_AMP_DEFAULT) * self.descent_boost_mul(0.4);
        self.hum_pulse_amp_current += (pulse_amp_target - self.hum_pulse_amp_current) * (dt * 1.5).min(1.0);
        let hum_pulse = self.hum_pulse_amp_current * (self.hum_pulse_phase.sin() * 0.5 + 0.5);
        // R146 sanity dread pulse: "the darkness breathes with you"
        let mut sanity_dread = 0.0;
        let san_for_vig = self.player.sanity / SAN_MAX;
        if san_for_vig < 0.30 {
            let dread_mag = 1.0 - san_for_vig / 0.30;
            let dread_period =
                sanity_dread_period_ms_by_weather(wk).or(sanity_dread_period_ms_by_weather("normal")).unwrap() / self.descent_freq_boost();
            sanity_dread = 0.18 * self.descent_boost_mul(0.4) * dread_mag * ((self.clock_ms() / dread_period).sin() * 0.5 + 0.5);
        }
        // R164 level-descent vignette baseline
        let descent_dread = (1.0 - self.level_depth_mul()) * 0.8;
        sanity_dread += descent_dread;
        // R197/R198 cry-dim vignette pulse
        let cry_vig_peak = cry_vignette_peak_by_weather(wk).or(cry_vignette_peak_by_weather("normal")).unwrap();
        let cry_vignette = cry_vig_peak * self.cry_dim_lerp;
        // R220 distant knock pulse
        if self.knock_flash_t > 0.0 {
            self.knock_flash_t -= dt;
        }
        let knock_pulse = if self.knock_flash_t > 0.0 { 0.10 * (self.knock_flash_t / 0.6) * (self.knock_flash_t / 0.6) } else { 0.0 };
        // R223 door-opened pulse
        if self.door_opened_flash_t > 0.0 {
            self.door_opened_flash_t -= dt;
        }
        let door_pulse = if self.door_opened_flash_t > 0.0 { 0.08 * (PI * self.door_opened_flash_t / 1.0).sin() } else { 0.0 };
        // R225 elevator ding pulse
        if self.elevator_ding_flash_t > 0.0 {
            self.elevator_ding_flash_t -= dt;
        }
        let ding_pulse = if self.elevator_ding_flash_t > 0.0 { 0.06 * (self.elevator_ding_flash_t / 1.4) } else { 0.0 };
        // R228 voice-sample pulse
        if self.voice_sample_flash_t > 0.0 {
            self.voice_sample_flash_t -= dt;
        }
        let voice_pulse = if self.voice_sample_flash_t > 0.0 {
            0.05 * (self.voice_sample_flash_t / 0.8) * (self.voice_sample_flash_t / 0.8)
        } else {
            0.0
        };
        // R230 static-burst pulse
        if self.static_burst_flash_t > 0.0 {
            self.static_burst_flash_t -= dt;
        }
        let static_pulse = if self.static_burst_flash_t > 0.0 {
            0.07 * (self.static_burst_flash_t / 0.5) * (self.static_burst_flash_t / 0.5)
        } else {
            0.0
        };
        // R237 reality shift vignette pulse
        let shift_pulse = if self.reality_shift_t > 0.0 {
            0.12 * (PI * (2.0 - self.reality_shift_t) / 2.0).sin() * (self.reality_shift_t / 2.0)
        } else {
            0.0
        };
        // R243 pattern break vignette pulse
        let pattern_pulse = if self.pattern_break_t > 0.0 { 0.03 * (PI * (1.0 - self.pattern_break_t / 4.0)).sin() } else { 0.0 };
        // R243 clock skip vignette pulse
        let skip_pulse = if self.clock_skip_t > 0.0 { 0.06 * (self.clock_skip_t / 0.5) } else { 0.0 };
        let ip = 1.0 - pressure;
        self.ui.vignette.set_opacity(clamp(
            pressure * 0.95
                + hum_pulse * ip
                + sanity_dread * ip
                + cry_vignette * ip
                + knock_pulse * ip
                + door_pulse * ip
                + ding_pulse * ip
                + voice_pulse * ip
                + static_pulse * ip
                + shift_pulse * ip
                + pattern_pulse * ip
                + skip_pulse * ip,
            0.0,
            0.95,
        ));
        // R151 chime visual flash: brief coloured pulse via inset box-shadow
        if self.chime_flash_t > 0.0 {
            self.chime_flash_t -= dt;
            if self.chime_flash_t <= 0.0 {
                // only clear the R151 flash, not the R103 electric surge tint
                if self.electric_surge_t <= 0.0 {
                    self.ui.vignette.set_style("boxShadow", "");
                }
            } else if let Some(c) = self.chime_flash_color.clone().filter(|c| !c.is_empty()) {
                let p = self.chime_flash_t / self.chime_flash_life; // 1 → 0 over lifetime
                let alpha = fmt_fixed(self.chime_flash_peak * p * p, 3);
                self.ui.vignette.set_style("boxShadow", &format!("inset 0 0 240px 60px {c}, {alpha})"));
            }
        }
        // R140/R141 sanity hue-shift; R232 static chromatic
        let s01 = self.player.sanity / SAN_MAX;
        if self.static_chromatic_t > 0.0 {
            self.static_chromatic_t -= dt;
        }
        let chromatic_mag = if self.static_chromatic_t > 0.0 { self.static_chromatic_t / 0.35 } else { 0.0 };
        let chrom_spec = static_chrom_by_weather(wk).or(static_chrom_by_weather("normal")).unwrap();
        let chrom_contrast = 1.0 + chrom_spec.contrast * chromatic_mag;
        let chrom_saturate = 1.0 + chrom_spec.saturate * chromatic_mag;
        if s01 < 0.20 && self.state == "playing" {
            let shift_mag = 1.0 - s01 / 0.20; // 0 → 1 as sanity drops
            let hue_spec = sanity_hue_by_weather(wk).or(sanity_hue_by_weather("normal")).unwrap();
            let hue_deg = (hue_spec.spread * (self.clock_ms() / 700.0).sin() + hue_spec.bias) * shift_mag;
            let f = format!(
                "hue-rotate({}deg) saturate({}) contrast({})",
                fmt_fixed(hue_deg, 1),
                fmt_fixed((1.0 + shift_mag * 1.5) * chrom_saturate, 2),
                fmt_fixed(chrom_contrast, 2)
            );
            self.ui.vignette.set_style("filter", &f);
        } else if chromatic_mag > 0.0 {
            let f = format!("saturate({}) contrast({})", fmt_fixed(chrom_saturate, 2), fmt_fixed(chrom_contrast, 2));
            self.ui.vignette.set_style("filter", &f);
        } else if self.ui.vignette.style("filter").is_some() {
            self.ui.vignette.set_style("filter", "");
        }
        // R141/R142 sanity safe-room detune: the hum sags at very low sanity
        if let (Some(lfo), Some(car)) = (self.audio.safe_room_lfo.clone(), self.audio.safe_room_trem_carrier.clone()) {
            let lfo_jit = ((self.current_seed & 0xff) as f64 / 255.0 - 0.5) * 0.20;
            let lfo_weather_mul = lfo_weather_mul(wk).unwrap_or(1.0);
            let level_depth = self.level_depth_mul();
            let nominal = (0.35 + lfo_jit) * lfo_weather_mul * level_depth;
            let sanity_mag = if s01 < 0.20 { 1.0 - s01 / 0.20 } else { 0.0 };
            lfo.frequency().set_value(nominal + (-0.10 * sanity_mag));
            let car_jit = (((self.current_seed >> 8) & 0xff) as f64 / 255.0 - 0.5) * 40.0;
            let car_weather_off = carrier_weather_detune(wk).unwrap_or(0.0);
            let car_nominal = (330.0 + car_jit + car_weather_off) * level_depth;
            car.frequency().set_value(car_nominal - 15.0 * sanity_mag);
        }
        // R200–R207 stamina breath layer: below 30% stamina, breath becomes audible
        if let (Some(bgn), Some(blfo), Some(blp)) =
            (self.audio.stamina_breath_gain.clone(), self.audio.stamina_breath_lfo.clone(), self.audio.stamina_breath_lp.clone())
        {
            let stam = self.player.stamina / 100.0;
            let breath_mag = if stam >= 0.30 || self.state != "playing" { 0.0 } else { 1.0 - stam / 0.30 };
            let br_spec = stamina_breath_by_weather(wk).or(stamina_breath_by_weather("normal")).unwrap();
            let target_gain = br_spec.gain * breath_mag;
            let bg = bgn.gain();
            bg.set_value(bg.value() + (target_gain - bg.value()) * (dt * 3.0).min(1.0));
            // R202 sanity boost: panicked breath cadence without exhaustion
            let san_frac = self.player.sanity / SAN_MAX;
            let sanity_breath_boost = if san_frac < 0.30 { (1.0 - san_frac / 0.30) * 0.10 } else { 0.0 };
            let target_freq = br_spec.rate_base + br_spec.rate_gain * breath_mag + sanity_breath_boost;
            let bf = blfo.frequency();
            bf.set_value(bf.value() + (target_freq - bf.value()) * (dt * 2.0).min(1.0));
            // R202 filter sweep 220 → 400 Hz at peak fatigue
            let target_lp = 220.0 + 180.0 * breath_mag;
            let bl = blp.frequency();
            bl.set_value(bl.value() + (target_lp - bl.value()) * (dt * 2.0).min(1.0));
            // R203 Q sweep 0.4 → 0.7
            let target_q = 0.4 + 0.3 * breath_mag;
            let bq = blp.q();
            bq.set_value(bq.value() + (target_q - bq.value()) * (dt * 2.0).min(1.0));
            // R204/R205 sub-osc chest thump gain
            if let Some(sg) = &self.audio.stamina_breath_sub_gain {
                let sub_mag = 0f64.max(breath_mag - 0.15) / 0.85;
                let target_sub = br_spec.sub_gain * sub_mag;
                let g = sg.gain();
                g.set_value(g.value() + (target_sub - g.value()) * (dt * 3.0).min(1.0));
            }
            // R206 sub-osc frequency toward the weather target
            if let Some(so) = &self.audio.stamina_breath_sub_osc {
                let sf = so.frequency();
                sf.set_value(sf.value() + (br_spec.sub_freq - sf.value()) * (dt * 0.8).min(1.0));
            }
            // R204/R206/R207 pan wander at peak fatigue
            if let Some(pn) = &self.audio.stamina_breath_panner {
                let pan_depth = br_spec.pan_depth * breath_mag;
                let target_pan = (self.perf_t * br_spec.pan_rate).sin() * pan_depth;
                let pp = pn.pan();
                pp.set_value(pp.value() + (target_pan - pp.value()) * (dt * 2.0).min(1.0));
            }
        }
        // (R142, the low-sanity room-tone brightening, is part of the room-tone
        // lerp in `update_ambient`)
        // R103 electric surge tint: inset blue-white halo synced with the surge dim
        if self.electric_surge_t > 0.0 {
            let fade = self.electric_surge_t / 0.35;
            let alpha = fmt_fixed(0.55 * fade, 2);
            self.ui.vignette.set_style("boxShadow", &format!("inset 0 0 220px 40px rgba(180, 220, 255, {alpha})"));
            // opacity floor so the flash is visible even when pressure is 0
            let cur = self.ui.vignette.opacity().filter(|v| !v.is_nan()).unwrap_or(0.0);
            self.ui.vignette.set_opacity(cur.max(0.55 * fade));
        } else if self.ui.vignette.style("boxShadow").is_some() {
            self.ui.vignette.set_style("boxShadow", "");
        }
        let dread = self.director.tension;
        if (any_hunt && nearest < 14.0) || dread > 0.45 {
            self.heart_t -= dt;
            if self.heart_t <= 0.0 {
                let k = (1.0 - clamp(nearest / 14.0, 0.0, 1.0)).max(dread);
                self.heart_t = lerp(1.05, 0.42, k);
                self.sfx_thump(clamp(0.2 + k, 0.25, 1.0));
            }
        }
        // sanity: drains near entities (heavier when hunted or looking at them), regens otherwise
        let mut drain = 0.0;
        let pp = self.player.pos;
        let fwd = self.player.fwd();
        for e in &self.entities {
            let p = e.pos();
            let d = hypot2(p.x - pp.x, p.z - pp.z);
            if d > 14.0 {
                continue;
            }
            if e.kind == "staff" && self.store_state.phase != "night" {
                continue;
            } // by day they only look
            if e.kind == "gone" {
                continue;
            }
            let mut to_e = v3(p.x - pp.x, 0.0, p.z - pp.z);
            to_e.normalize();
            let looking = fwd.dot(&to_e) > 0.5;
            let proximity = clamp(1.0 - d / 14.0, 0.0, 1.0);
            drain += proximity * (if looking { 5.5 } else { 2.2 }) * (if e.state == "hunt" { 1.7 } else { 1.0 });
        }
        // (it comes back on its own in `update_player`)
        if drain > 0.0 {
            self.player.sanity = clamp(self.player.sanity - drain * dt, 0.0, SAN_MAX);
        }
        // ambient whisper when sanity is very low
        if self.player.sanity < 25.0 && rnd() < dt * 0.35 {
            self.sfx_whisper();
        }
    }
}
