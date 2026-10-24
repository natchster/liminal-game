//! backrooms.js "entities", animation (lines 61706–61739, 61900–62238):
//! everything procedural — a gait phase advanced by how fast the thing is
//! actually travelling, per-state posture, and two-bone IK that plants the
//! hands and feet. Joint angles are set once per frame.
use crate::entities::*;
use crate::prelude::*;
use crate::records::Entity;
use crate::stairwell::js_hypot2;

// line 61710
/// `houndSpine(P)` → (P.mPelvis, P.mThorax): the spine's matrices in the
/// creature's frame. Call after posing the spine.
pub fn hound_spine(p: &HoundRig) -> (Mat4, Mat4) {
    p.spine.update_matrix(); p.pelvis.update_matrix(); p.lumbar.update_matrix(); p.thorax.update_matrix();
    let mp = Mat4::mul(&p.spine.matrix(), &p.pelvis.matrix());
    let mt = Mat4::mul(&Mat4::mul(&mp, &p.lumbar.matrix()), &p.thorax.matrix());
    (mp, mt)
}

// line 61722
/// `houndLimb(P, L, x, y, z, roll, yaw)` — one limb's IK in the creature's
/// frame. (x, y, z): the contact on the floor (under the knuckles, or the
/// ball of the foot); `roll` pitches the hand or foot about it (heel up +),
/// `yaw` turns it out. The limb's plane tips sideways onto the target, and
/// near full reach it eases in instead of locking straight.
pub fn hound_limb(m_pelvis: &Mat4, m_thorax: &Mat4, l: &HdLeg, x: f64, y: f64, z: f64, roll: f64, yaw: f64) {
    let m = match (&l.scap, l.front) {
        (Some(sc), true) => { sc.update_matrix(); Mat4::mul(m_thorax, &sc.matrix()) }
        _ => *m_pelvis,
    };
    let q = Quat::from_euler(&Euler::with_order(roll, yaw, 0.0, EulerOrder::YXZ));
    let mut v = v3(l.off[0], l.off[1], l.off[2]);
    v.apply_quaternion(&q);   // the wrist or ankle over its pivot
    v.x += x; v.y += y; v.z += z;
    v.apply_matrix4(&m.inverted());
    let h = l.hip.pos();
    let (dx, dy, dz) = (v.x - h.x, v.y - h.y, v.z - h.z);
    let a = if l.front { HD_RIG.up_arm } else { HD_RIG.thigh };
    let b = if l.front { HD_RIG.fore } else { HD_RIG.shin };
    let mut py = -js_hypot2(dx, dy);
    let mut pz = dz;
    let d = js_hypot2(py, pz);
    let soft = (a + b) * 0.998 - 0.025;
    if d > soft { let k = (soft + 0.025 * (1.0 - ((soft - d) / 0.025).exp())) / d; py *= k; pz *= k; }
    let (t1, t2) = solve_limb(0.0, 0.0, py, pz, a, b, if l.front { 0.0 } else { 1.0 });
    l.hip.set_rot(-t1, 0.0, dx.atan2(-dy));   // order ZYX: swing in the plane, then tip it
    l.knee.rotation().x = t1 - t2;
    let mut q2 = Quat::IDENTITY;
    q2.set_from_rotation_matrix(&m);
    q2.multiply(&l.hip.quat()).multiply(&l.knee.quat()).invert();
    let mut fq = Quat::IDENTITY;
    fq.multiply_quaternions(&q2, &q);
    l.foot.set_quat(&fq);
}

/// JS `e.k || 0`
#[inline] fn nz(e: &Entity, k: &str) -> f64 { let v = e.n(k); if v.is_nan() { 0.0 } else { v } }
/// JS `e.k ?? d`
#[inline] fn nd(e: &Entity, k: &str, d: f64) -> f64 { e.num.get(k).copied().unwrap_or(d) }

impl Game {
    // line 61910
    /// `entityTics(e, dt, twitch)` — close-range twitching: every so often
    /// (more often the nearer it is) the head and shoulders snap to a new small
    /// offset along a stiff, critically damped spring, and hold it.
    pub fn entity_tics(&mut self, ei: usize, dt: f64, twitch: f64) -> [f64; 3] {
        let id = self.entities[ei].g.id();
        let ext = self.ent_ext.entry(id).or_default();
        let t = ext.tic.get_or_insert_with(Tic::default);
        t.next -= dt;
        if t.next <= 0.0 {
            t.next = lerp(2.4, 0.3, twitch) * (0.6 + rnd() * 0.8);
            let k = 0.1 * twitch;
            for i in 0..3 { t.g[i] = (rnd() - 0.5) * 2.0 * k; }
        }
        let (w, h) = (30.0, dt.min(0.05) / 2.0);
        for _ in 0..2 {
            for i in 0..3 {
                t.v[i] += (w * w * (t.g[i] - t.p[i]) - 2.0 * w * t.v[i]) * h;
                t.p[i] += t.v[i] * h;
            }
        }
        t.p
    }

    // line 61926
    /// `animateEntity(e, dt, t, distToPlayer)` — joints, gait, jaw, head tracking
    pub fn animate_entity(&mut self, ei: usize, dt: f64, t: f64, dist_to_player: f64) {
        let Some(rig) = self.entity_rig(ei) else { return };
        let pp = self.player.pos;
        let pyaw = self.player.yaw;
        // how far it moved this frame tells us whether — and how fast — it walks
        let (moved, speed, stride, hunt, twitch, walk) = {
            let e = &mut self.entities[ei];
            let pos = e.pos();
            let moved = js_hypot2(pos.x - nd(e, "lastX", pos.x), pos.z - nd(e, "lastZ", pos.z));
            e.set_n("lastX", pos.x); e.set_n("lastZ", pos.z);
            let speed = if dt > 0.0 { (moved / dt).min(12.0) } else { 0.0 };
            // the gait advances by the distance actually walked (a jump of more
            // than 0.6 m in one frame is a relocation, not a stride)
            let stride = if moved > 0.6 { 0.0 } else { moved };
            let hunt = lerp(nz(e, "hunt"), if e.state == "hunt" { 1.0 } else { 0.0 }, (dt * 2.2).min(1.0));
            e.set_n("hunt", hunt);
            // close range makes it twitch — the stillness breaks down as it gets near
            let twitch = clamp(1.0 - dist_to_player / 7.0, 0.0, 1.0) * (0.4 + 0.6 * hunt);
            let walk = lerp(nz(e, "walk"), if speed > 0.15 { 1.0 } else { 0.0 }, (dt * 4.0).min(1.0));
            e.set_n("walk", walk);
            // head turns to keep the player in view independent of where the body goes
            let want_head = clamp(wrap_pi((pp.x - pos.x).atan2(pp.z - pos.z) - e.yaw), -1.95, 1.95);
            let hy = lerp(nz(e, "headYaw"), want_head, (dt * (2.0 + 4.0 * hunt)).min(1.0));
            e.set_n("headYaw", hy);
            (moved, speed, stride, hunt, twitch, walk)
        };
        let kind = self.entities[ei].kind.clone();
        match (kind.as_str(), &*rig) {
            ("hound", Rig::Hound(p)) => self.animate_hound(ei, p, dt, t, dist_to_player, speed, stride, hunt, twitch, walk),
            ("watcher", Rig::Watcher(p)) => self.animate_watcher(ei, p, dt, moved, pp, pyaw),
            ("hound", _) | ("watcher", _) => {}
            (_, Rig::Smiler(p)) => self.animate_smiler(ei, p, dt, t, dist_to_player, speed, stride, hunt, twitch, walk),
            _ => {}
        }
    }

    /// A human bear-crawl: a lateral-sequence walk (hind, then the hand on the
    /// same side) that turns into a lurching bound as it speeds up. Hands and
    /// feet are solved onto their contacts, so they plant; the spine carries
    /// the gait between them. The right hind leg is lame.
    fn animate_hound(&mut self, ei: usize, p: &HoundRig, dt: f64, t: f64, dist: f64, speed: f64, stride: f64, hunt: f64, twitch: f64, walk: f64) {
        let (v, bound, gp, cyc, hold, listen) = {
            let e = &mut self.entities[ei];
            let v = lerp(nz(e, "hdSpeed"), speed, 1.0 - (-dt * 5.0).exp());
            e.set_n("hdSpeed", v);
            let bound = lerp(nz(e, "bound"), ent_smooth(2.4, 5.0, v), 1.0 - (-dt * 3.0).exp());
            e.set_n("bound", bound);
            let rate = lerp(10.0, 3.1, bound);   // gait radians per metre
            let gait = nz(e, "gait") + stride * rate;
            e.set_n("gait", gait);
            let (gp, cyc) = (gait, (PI * 2.0) / rate);   // metres per cycle
            let hold = lerp(nz(e, "crouch"), if e.n("holdT") > 0.0 { 1.0 } else { 0.0 }, 1.0 - (-dt * 6.0).exp());   // stared down
            e.set_n("crouch", hold);
            let listen = lerp(nz(e, "listen"), if e.state == "wander" && e.freeze > 0.0 { 1.0 } else { 0.0 }, 1.0 - (-dt * 3.0).exp());
            e.set_n("listen", listen);
            (v, bound, gp, cyc, hold, listen)
        };
        let (charge, duty) = (hunt * bound, lerp(0.62, 0.27, bound));
        let tic = self.entity_tics(ei, dt, twitch);
        // each limb: its contact point, how far the heel has peeled up over the
        // knuckles or the ball (roll), how much weight it carries, the curl
        #[derive(Clone, Copy, Default)]
        struct LS { cz: f64, cy: f64, swing: f64, roll: f64, peel: f64, load: f64 }
        let mut ls = [LS::default(); 4];
        let mut land: f64 = 0.0;
        for (li, l) in p.legs.iter().enumerate() {
            let diag = if l.front == (l.side < 0.0) { 0.0 } else { PI };           // diagonal pairs, LF+RH / RF+LH
            let walk_off = diag + if l.front { PI / 2.0 } else { PI };             // lateral sequence: RH, RF, LH, LF
            let off = walk_off + wrap_pi((if l.front { PI } else { 0.0 }) - walk_off) * bound;   // bound: feet, then hands
            let d = if l.lame { duty * 0.85 } else { duty };
            let u = (((gp + off) / (PI * 2.0)) % 1.0 + 1.0) % 1.0;
            let (s, r) = (cyc * d * walk, if l.front { 0.55 } else { 0.4 });   // what the body travels while it is down: planted
            let (cz, roll);
            let (mut cy, mut swing, mut load, mut peel) = (0.0, 0.0, 0.0, 0.0);
            if u < d {   // down: q 0..1 through the stance
                let q = u / d;
                cz = s * (0.5 - q);
                roll = r * ent_smooth(0.55, 1.0, q);
                load = ent_smooth(0.0, 0.2, q) * (1.0 - ent_smooth(0.5, 1.0, q));
            } else {     // in the air: q 0..1 through the swing
                let q = (u - d) / (1.0 - d);
                cz = s * (-0.5 + q * q * (3.0 - 2.0 * q));
                swing = (PI * q).sin();
                cy = swing * (if l.front { 0.1 } else { 0.09 }) * (0.6 + 0.25 * bound) * walk;
                roll = r * (1.0 - ent_smooth(0.0, 0.7, q)) + (if l.front { -0.18 } else { 0.12 }) * swing;   // reaching, fingers up
                peel = ent_smooth(0.0, 0.35, q);
            }
            ls[li] = LS { cz, cy, swing: swing * walk, roll: roll * walk, peel, load: lerp(1.0, load, walk) };
            if l.front { let w = wrap_pi(PI * 2.0 * u - 0.25); land = land.max((-8.0 * w * w).exp() * walk); }
            l.hip.position().y = l.base_y + 0f64.max(swing) * (if l.front { 0.0 } else if l.lame { 0.014 } else { 0.004 });   // the lame hip hikes
        }
        // the spine between them: it bends toward the side whose hand and foot
        // are closing, rolls up over a lifted limb and so twists, arches as the
        // bound gathers, and hunches when stared down while the rump drops
        let (rf, lf, rh, lh) = (ls[0], ls[1], ls[2], ls[3]);
        let bend_y = 0.1 * ((rf.cz - rh.cz) - (lf.cz - lh.cz));
        let arch = ((gp).cos() * 0.13 * bound + (gp * 2.0).sin() * 0.025 * (1.0 - bound)) * walk + 0.1 * hold;
        let pitch = -0.16 * hold + gp.sin() * 0.06 * charge;
        let (roll_p, roll_t) = (0.05 * lh.swing - 0.1 * rh.swing, 0.07 * (lf.swing - rf.swing));
        let s_y = HD_RIG.spine_y - 0.045 * hold - 0.02 * walk - 0.05 * bound + gp.sin().powi(2) * (0.008 + 0.02 * bound) * walk - 0.008 * land;
        p.spine.position().y = s_y;
        p.spine.rotation().x = pitch;
        let (pel, lum, tho) = ([-0.45 * arch, -0.6 * bend_y, roll_p], [0.6 * arch, bend_y, 0.5 * (roll_t - roll_p)], [0.35 * arch + tic[0] * 0.3, bend_y, 0.5 * (roll_t - roll_p)]);
        p.pelvis.set_rot(pel[0], pel[1], pel[2]);
        p.lumbar.set_rot(lum[0], lum[1], lum[2]);
        p.thorax.set_rot(tho[0], tho[1], tho[2]);
        // breathing: slow and deep when it wanders, panting when it hunts or runs
        let pant = hunt.max(bound);
        let breath = { let e = &mut self.entities[ei]; let b = nz(e, "breath") + dt * lerp(1.6, 7.0, pant); e.set_n("breath", b); b };
        let br = (0.5 + 0.5 * breath.sin()) * lerp(0.045, 0.025, pant);
        p.ribs.set_scale(1.0 + br, 1.0 + br * 1.3, 1.0 + br * 0.3);
        let (mp, mt) = hound_spine(p);
        let dg = hd_digits();
        for (li, l) in p.legs.iter().enumerate() {
            let s = ls[li];
            // the shoulder blade slides back as the body passes over its hand and
            // stands up off the ribs as the weight comes on
            if l.front {
                if let Some(sc) = &l.scap {
                    sc.set_rot(-0.5 * s.cz + 0.08 * hold, l.side * 0.05 * s.swing, -l.side * (0.15 * (s.load - 0.6) + 0.1 * hold + tic[2] * 0.6));
                }
            }
            hound_limb(&mp, &mt, l, l.side * (if l.front { HD_RIG.hand_x } else { HD_RIG.foot_x }), s.cy,
                (if l.front { HD_RIG.hand_z + HD_RIG.knuckle[2] } else { HD_RIG.foot_z }) + s.cz, s.roll, l.side * (if l.front { 0.12 } else { 0.05 }));
            // fingers splay as the hand takes the weight, stay flat while the heel
            // peels up, curl in the air; stared down they claw at the floor
            let flat = -s.roll * (1.0 - s.peel);
            for (k, dd) in l.digits.iter().enumerate() {
                let tw = tic[k % 3] * 2.5;
                if l.front {
                    dd[0].set_rot(flat + s.swing * 0.5 - 0.3 * hold + tw, dg.splay[k] * (0.04 + 0.18 * s.load), 0.0);
                    dd[1].rotation().x = s.swing * (if k == 4 { 0.4 } else { 0.85 }) + 0.55 * hold + tw * 0.6;
                    dd[2].rotation().x = s.swing * (if k == 4 { 0.3 } else { 0.65 }) + 0.4 * hold;
                } else {
                    dd[0].rotation().x = flat + s.swing * 0.25 - 0.1 * hold;
                    dd[1].rotation().x = 0.12 * s.load + s.swing * 0.35 + 0.25 * hold;
                }
            }
        }
        // the head hangs low, bobs on the hands' footfalls, cocks to listen,
        // drops and trembles when stared down, and comes up to charge
        let sp = pitch + pel[0] + lum[0] + tho[0];   // the shoulders' pitch
        let neck_x = HD_RIG.neck_pitch + 0.12 * hold - 0.35 * charge + land * 0.07;
        p.neck.rotation().x = neck_x;
        let head_x = -(neck_x + sp) + 0.12 * hold - 0.2 * charge;
        p.head.rotation().x = head_x;
        let head_yaw = self.entities[ei].n("headYaw");
        p.head.rotation().y = clamp(head_yaw * 0.6, -0.7, 0.7) + tic[1];
        if self.entities[ei].n("cock") == 0.0 || self.entities[ei].n("cock").is_nan() {
            let c = if rnd() < 0.5 { -1.0 } else { 1.0 };
            self.entities[ei].set_n("cock", c);
        }
        let cock = self.entities[ei].n("cock");
        let head_z = (t * 1.3).sin() * 0.04 + 0.45 * listen * cock;
        p.head.rotation().z = head_z;
        p.head.position().x = (t * 57.0).sin() * 0.0015 * hold;   // a 1–2 mm tremor
        // breathing through the mouth; it hangs open on the hunt and gapes in a charge
        p.jaw.rotation().x = 0.04 + (1.0 - hunt) * (0.5 + 0.5 * breath.sin()) * 0.05
            + hunt * (1.0 - hold) * (1.0 - bound) * (0.3 + (t * 9.0).sin() * 0.08) + 0.42 * hold + 0.92 * charge;
        for (i, ear) in p.ears.iter().enumerate() { ear.rotation().y = (if i != 0 { 1.0 } else { -1.0 }) * (0.55 * listen + 0.25 * hunt); }
        // the hair on a spring: it lags the head, streams back in a charge and
        // parts when the head is cocked, so the eyeshine flickers through it
        let hdt = dt.min(0.05);
        let tx0 = -(head_x + neck_x + sp) * 0.8 + 0.55f64.min(v * 0.07) + land * 0.05;
        let tz0 = -head_z * 0.8 - (pel[2] + lum[2] + tho[2]) * 0.6;
        let (hx, hz) = {
            let e = &mut self.entities[ei];
            let vx = nz(e, "hairVX") + ((tx0 - nz(e, "hairX")) * 70.0 - nz(e, "hairVX") * 9.0) * hdt;
            e.set_n("hairVX", vx);
            let vz = nz(e, "hairVZ") + ((tz0 - nz(e, "hairZ")) * 70.0 - nz(e, "hairVZ") * 9.0) * hdt;
            e.set_n("hairVZ", vz);
            let hx = nz(e, "hairX") + vx * hdt;
            let hz = nz(e, "hairZ") + vz * hdt;
            e.set_n("hairX", hx); e.set_n("hairZ", hz);
            (hx, hz)
        };
        p.hair.rotation().x = hx; p.hair.rotation().z = hz;
        if let Some([root, mid, tip]) = &p.tail {   // the trailing clump sways and streams
            let sway = (gp * 0.5 + t * 1.3).sin();
            root.rotation().x = 0.35 + 0.6 * charge + 0.4f64.min(v * 0.05);
            root.rotation().z = sway * 0.12 * (0.4 + walk);
            mid.rotation().x = 0.12 + 0.2 * charge; mid.rotation().z = -sway * 0.1;
            tip.rotation().x = 0.18; tip.rotation().z = sway * 0.12;
        }
        // far off, the eyeshine billboard, flickering as the hair sways
        let flicker = 0.5 + 0.5 * (t * 6.3 + (t * 2.1).sin() * 2.0).sin();
        let op = ent_smooth(7.0, 10.0, dist) * (0.45 + 0.55 * flicker) * (1.0 - ent_smooth(30.0, 38.0, dist));
        let mut fm = p.far.m();
        fm.opacity = op;
        fm.visible = op > 0.004;   // not drawn at all when it would not show
    }

    /// It never moves while it is seen: no idle, no breathing, no sway, no
    /// head tracking. Each time it has moved unseen (a metre or more since you
    /// last saw it) it takes the next, more attentive pose, one step per look
    /// away, and only while nobody is looking. Unseen, it also turns to face
    /// you; under a low ceiling it bows.
    fn animate_watcher(&mut self, ei: usize, p: &WatcherRig, dt: f64, moved: f64, pp: Vec3, pyaw: f64) {
        let pos = self.entities[ei].pos();
        let ceil = self.e_ceil_at_world(pos.x, pos.z);
        if !self.entities[ei].num.contains_key("poseYaw") {
            let e = &mut self.entities[ei];
            let y = e.yaw;
            e.set_n("poseYaw", y); e.set_n("poseStep", 0.0); e.set_n("poseFrom", 0.0); e.set_n("poseS", 1.0); e.set_n("poseMove", 0.0); e.set_fl("poseTaken", false);
            apply_watcher_pose_with(e, p, ceil, 1.0);
        }
        let (ex, ez) = (pos.x - pp.x, pos.z - pp.z);
        let hl = js_hypot2(ex, ez);
        let ahead = (-pyaw.sin() * ex - pyaw.cos() * ez) / (if hl != 0.0 && !hl.is_nan() { hl } else { 1.0 });
        let e = &mut self.entities[ei];
        let observed = e.flag.get("observed").copied();
        if observed == Some(false) && ahead <= 0.35 {
            let pm = e.n("poseMove") + moved;
            e.set_n("poseMove", pm);
            if !e.fl("poseTaken") && pm > 1.0 && e.n("poseStep") < (WT_POSES.len() - 1) as f64 {
                let st = e.n("poseStep");
                e.set_n("poseFrom", st); e.set_n("poseStep", st + 1.0); e.set_n("poseS", 0.0); e.set_fl("poseTaken", true);
            }
            let ps = 1f64.min(e.n("poseS") + moved / 0.9 + dt / 0.6);
            e.set_n("poseS", ps);
            e.set_n("poseYaw", (pp.x - pos.x).atan2(pp.z - pos.z));
            apply_watcher_pose_with(e, p, ceil, dt + moved);
        } else if observed == Some(true) { e.set_n("poseMove", 0.0); e.set_fl("poseTaken", false); }
        let py = e.n("poseYaw");
        e.g.rotation().y = py;
    }

    /// The Smiler: a walk that is almost right. Upright and slow when it
    /// wanders; stooped and creeping when it hunts and you hold its gaze;
    /// torso thrown forward 40 degrees with the arms reaching when it lunges.
    /// IK plants the balls of the feet; the hock always folds backwards. In
    /// bright light it flinches: head turned away, a forearm up.
    fn animate_smiler(&mut self, ei: usize, p: &SmilerRig, dt: f64, t: f64, dist: f64, speed: f64, stride: f64, hunt: f64, twitch: f64, walk: f64) {
        let pos = self.entities[ei].pos();
        let (gp, lunge) = {
            let e = &mut self.entities[ei];
            let gait = nz(e, "gait") + stride * 3.4;
            e.set_n("gait", gait);
            // state blends ease exponentially, the same at any frame rate
            let lunge = lerp(nz(e, "lunge"), if e.n("lungeT") > 0.0 || speed > 7.0 { 1.0 } else { 0.0 }, 1.0 - (-5.0 * dt).exp());
            e.set_n("lunge", lunge);
            (gait, lunge)
        };
        let stalk = hunt * (1.0 - lunge);
        let lit = self.e_region_light_at(self.tx(pos.x), self.tz(pos.z));
        let flinch = {
            let e = &mut self.entities[ei];
            let f = lerp(nz(e, "flinch"), if hunt > 0.5 && lit > 0.65 { 1.0 } else { 0.0 }, 1.0 - (-3.0 * dt).exp());
            e.set_n("flinch", f);
            f
        };
        let tic = self.entity_tics(ei, dt, twitch);
        let (eye_k, breath_ph) = {
            let e = &mut self.entities[ei];
            let k = lerp(nd(e, "eyeK", 1.0), if lit > 0.65 { 0.4 } else { 1.0 }, 1.0 - (-3.0 * dt).exp());   // its eyes dim in bright light
            e.set_n("eyeK", k);
            // breathing: a slow rise of the chest and shoulders, quicker once it hunts
            let ph = (nz(e, "breathPh") + dt * (1.25 + 1.2 * hunt + 1.4 * lunge)) % (PI * 2.0);
            e.set_n("breathPh", ph);
            (k, ph)
        };
        // crowns: about 2.28 m wandering (a slight stoop), 2.1 stalking, 1.7 lunging
        let hip_y = SM_LEG.hip_y - 0.04 - 0.1 * stalk - 0.18 * lunge + gp.sin().abs() * 0.03 * walk;
        let breath = 0.5 - 0.5 * breath_ph.cos();
        // the stoop is a curve, not a hinge at the hips: pelvis, abdomen and chest
        // each take a share (their sum is the pitch the arms and head undo)
        let p_pel = 0.1 + 0.08 * stalk + 0.3 * lunge + (gp * 2.0).sin() * 0.025 * walk;
        let p_abd = 0.04 + 0.07 * stalk + 0.12 * lunge;
        let p_che = 0.04 + 0.07 * stalk + 0.1 * lunge - 0.035 * breath * (1.0 - 0.5 * lunge);
        let pitch = p_pel + p_abd + p_che;
        p.torso.position().y = hip_y;
        p.torso.rotation().x = p_pel;
        p.abdomen.rotation().x = p_abd;
        p.chest.rotation().x = p_che;
        p.chest.position().y = SM_SPINE_CHEST[1] + 0.005 * breath;
        // the hips turn with the legs and the shoulders against them; the sway runs up the spine
        let sw = gp.sin() * walk;
        p.torso.rotation().y = -0.07 * sw;
        p.abdomen.rotation().y = 0.03 * sw;
        p.chest.rotation().y = 0.11 * sw;
        p.torso.rotation().z = 0.05 * sw + (t * 1.2566).sin() * 0.009;   // 0.5 degrees of sway at 0.2 Hz
        p.abdomen.rotation().z = -0.015 * sw;
        p.chest.rotation().z = -0.025 * sw + (t * 1.2566 - 0.6).sin() * 0.006;
        let s_len = (PI / 3.4) * walk;   // a stance covers what the body travels, so the feet do not slide
        for l in &p.legs {
            let u = ((gp + if l.side > 0.0 { PI } else { 0.0 }) / (PI * 2.0)) % 1.0;
            let (mut bz, by, mu);
            let mut curl = 0.0;
            if u < 0.5 { bz = s_len * (0.5 - u * 2.0); by = SM_LEG.ball; mu = 0.525 + 0.35 * ent_smooth(0.3, 0.5, u) * walk; }
            else {
                let v = (u - 0.5) * 2.0;
                bz = s_len * (-0.5 + v * v * (3.0 - 2.0 * v));
                by = SM_LEG.ball + (PI * v).sin() * (0.1 + 0.08 * lunge) * walk;
                mu = 0.525 + 0.35 * (1.0 - v) * walk;
                curl = 0.35 * (PI * v).sin() * walk;
            }
            bz += 0.075 - 0.12 * lunge;
            let (t1, t2) = solve_limb(hip_y, 0.0, by + SM_LEG.meta * mu.cos(), bz - SM_LEG.meta * mu.sin(), SM_LEG.thigh, SM_LEG.shin, 1.0);
            l.hip.position().y = hip_y;
            l.hip.rotation().x = -t1;
            l.knee.rotation().x = t1 - t2;
            l.ankle.rotation().x = t2 - mu;   // the hock
            l.toes.rotation().x = mu + curl;
            l.hip.rotation().z = l.side * 0.04;
        }

        // the fingers' slow clawing runs on its own phase, faster the closer it gets
        let flex_ph = {
            let e = &mut self.entities[ei];
            let f = (nz(e, "flexPh") + dt * (0.5 + 1.3 * stalk + 2.2 * twitch)) % (PI * 2.0);
            e.set_n("flexPh", f);
            f
        };
        for a in &p.arms {
            // each arm swings with the opposite leg, a beat behind its shoulder
            let ph = gp + (if a.side > 0.0 { PI } else { 0.0 }) - 0.35;
            let swing = ph.sin() * (0.2 + 0.1 * stalk) * walk;
            let fwd = (1.0 + (ph - 0.4).sin()).powi(2) * 0.25;   // a smooth 0..1, peaking on the forward swing
            let raise = if a.side > 0.0 { flinch } else { 0.0 };   // a forearm comes up across the face
            // wandering the arms hang past the knees; stalking they come half up; lunging they reach
            {
                let mut r = a.shoulder.rotation();
                r.x = lerp(-pitch - 0.1 - swing - 0.45 * stalk - 1.05 * lunge, -pitch - 1.45, raise);
                r.y = lerp(-a.side * (0.5 - 0.3 * lunge), -a.side * 1.15, raise);   // raised: the forearm swings across
                r.z = lerp(a.side * (0.16 + 0.08 * stalk + 0.22 * lunge) + tic[2] * a.side * 0.6, a.side * 0.05, raise);
            }
            a.shoulder.position().y = a.y0 + 0.006 * breath;   // the shoulders lift with each breath
            a.elbow.rotation().x = lerp(-(0.25 + 0.45 * stalk + fwd * 0.25 * walk) * (1.0 - 0.75 * lunge), -1.95, raise);
            a.wrist.rotation().x = 0.1 + (t * 2.1 + a.side).sin() * 0.15 * (0.3 + stalk + twitch) - 0.45 * lunge;
            a.wrist.rotation().z = (t * 1.7 + a.side * 2.0).sin() * 0.1 * (0.3 + twitch);
            // fingers (curl is -x): loosely curled and drifting while it wanders;
            // stalking they flex one after another into a claw; splayed and fanned
            // in a lunge or over the face; close up they twitch
            let splay = lunge.max(raise);
            for (k, f) in a.fingers.iter().enumerate() {
                let kf = k as f64;
                let c = 0.5 - 0.5 * (flex_ph - kf * 0.7 + a.side).cos();   // 0 open .. 1 clawed
                let idle = (t * 0.6 + kf * 1.3 + a.side * 2.0).sin() * 0.05;
                let tw = tic[(k + if a.side > 0.0 { 1 } else { 2 }) % 3] * 2.6 * twitch;
                for (j, b) in f.iter().enumerate() {
                    let loose = (if j != 0 { -0.14 } else { -0.06 }) + idle;
                    let claw = if j != 0 { -(0.12 + 0.6 * c) } else { 0.12 - 0.22 * c };
                    b.rotation().x = lerp(lerp(loose, claw, stalk), if j != 0 { 0.14 } else { 0.24 }, splay) + if j != 0 { tw } else { tw * 0.4 };
                }
                f[0].rotation().z = (if k == 4 { 0.25 } else { (kf - 1.5) * 0.11 }) * splay;
            }
        }

        let neck_x = 0.55 + 0.1 * stalk + 0.2 * lunge + (t * 0.7).sin() * 0.02;
        p.neck.rotation().x = neck_x;
        p.neck.rotation().z = (t * 0.53).sin() * 0.04;
        // it looks down at you from just above your eye line; the face stays level against the stoop
        let gaze = clamp((hip_y + 0.95 - self.player.pos.y).atan2(0.6f64.max(dist)), -0.2, 0.55);
        p.head.rotation().x = -(neck_x + pitch) + gaze * (1.0 - 0.6 * lunge) + tic[0];
        let head_yaw = self.entities[ei].n("headYaw");
        p.head.rotation().y = head_yaw + tic[1] + flinch * 0.5;
        // a slow tilt, twelve degrees each way over about four seconds, and more the closer it gets
        p.head.rotation().z = (t * 1.57).sin() * 0.21 * (1.0 - 0.5 * hunt) + twitch * 0.3;
        // the jaw breathes with the chest; once it has seen you it drops and the
        // crescent opens into a lens, the slit splitting back along the cheeks
        p.jaw.rotation().x = 0.06 * breath + hunt * (0.55 + (t * 7.3).sin() * 0.08) + 0.1 * lunge;
        // the face: brighter by a third when it hunts; past 8 m only the drawn face is left
        p.eye_mat.m().emissive.set_scalar(eye_k * (1.0 + 0.3 * hunt));
        let glow = (1.0 + 0.3 * hunt) * eye_k;
        let ho = 1f64.min(glow * ent_smooth(1.2, 3.0, dist));
        let fo = 1f64.min(glow * ent_smooth(6.0, 9.0, dist) * (1.0 - ent_smooth(30.0, 38.0, dist)));
        { let mut m = p.halo.m(); m.opacity = ho; m.visible = ho > 0.004; }   // skip invisible billboards
        { let mut m = p.far.m(); m.opacity = fo; m.visible = fo > 0.004; }
    }

    // line 62221
    /// `applyWatcherPose(e, ease = 1)` — the blend from its last pose to its
    /// next (morph targets on its one sheet, eased by e.poseS), the face anchor
    /// moved to that pose's head, and a bow under low ceilings so the crown
    /// clears them by 0.15 m. `ease` is how far the bow may move this time.
    pub fn apply_watcher_pose(&mut self, ei: usize, ease: f64) {
        let Some(rig) = self.entity_rig(ei) else { return };
        let Rig::Watcher(p) = &*rig else { return };
        let pos = self.entities[ei].pos();
        let ceil = self.e_ceil_at_world(pos.x, pos.z);
        apply_watcher_pose_with(&mut self.entities[ei], p, ceil, ease);
    }
}

/// `applyWatcherPose` with the ceiling above the Watcher already looked up.
fn apply_watcher_pose_with(e: &mut Entity, p: &WatcherRig, ceil: f64, ease: f64) {
    let from = nz(e, "poseFrom");
    let step = nz(e, "poseStep");
    let a = &WT_POSES[from as usize];
    let b = &WT_POSES[step as usize];
    let s = nd(e, "poseS", 1.0);
    let k = s * s * (3.0 - 2.0 * s);
    e.g.scale().y = clamp((ceil - 0.15) / 2.31, 0.86, 1.0);
    let bow = lerp(nz(e, "bow"), if ceil < 2.6 { 1.0 } else { 0.0 }, clamp(ease * 2.0, 0.0, 1.0));
    e.set_n("bow", bow);
    let changed = {
        let mut inf = p.sheet.morph_influences();
        if inf.len() >= 5 {   // one sheet: last pose -> next pose, plus the bow
            let before = inf.clone();
            for v in inf.iter_mut() { *v = 0.0; }
            if from > 0.0 { inf[from as usize - 1] += 1.0 - k; }
            if step > 0.0 { inf[step as usize - 1] += k; }
            inf[4] = bow;
            if *inf != before || p.blended.borrow().is_empty() { Some(inf.clone()) } else { None }
        } else { None }
    };
    if let Some(inf) = changed { blend_watcher_colors(p, &inf); }
    let h0 = WT_POSES[0].head;
    p.head.set_pos(lerp(a.head[0], b.head[0], k) + (WT_BOW_HEAD[0] - h0[0]) * bow,
        lerp(a.head[1], b.head[1], k) + (WT_BOW_HEAD[1] - h0[1]) * bow, lerp(a.head[2], b.head[2], k) + (WT_BOW_HEAD[2] - h0[2]) * bow);
    p.head.set_rot(lerp(a.pitch + a.lean, b.pitch + b.lean, k) + 0.25 * bow, 0.0, lerp(a.tilt, b.tilt, k));
}

/// three r160 morphcolor_vertex on the CPU: `vColor = base + Σ dc_k · inf_k`
/// (relative targets) into this Watcher's own copy of the sheet's colours.
fn blend_watcher_colors(p: &WatcherRig, inf: &[f64]) {
    *p.blended.borrow_mut() = inf.to_vec();
    if p.morph_color.is_empty() || p.base_color.is_empty() { return; }
    let Some(geo) = p.sheet.try_geo() else { return };
    let mut gg = geo.g();
    let n = p.base_color.len().min(gg.color.len());
    for i in 0..n {
        let mut c = p.base_color[i];
        for (k, mc) in p.morph_color.iter().enumerate() {
            let w = inf.get(k).copied().unwrap_or(0.0);
            if w != 0.0 { c += mc[i] * w as f32; }
        }
        gg.color[i] = c;
    }
    gg.version += 1;
}
