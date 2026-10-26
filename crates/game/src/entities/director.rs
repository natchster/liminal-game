//! The pacing director. Creatures alone either never find you or find you
//! all the time; the director shapes a level into a rhythm of quiet, dread,
//! a close call and relief:
//!
//! * **Calm**: nothing is sent after you; the place is just empty.
//! * **Build**: one creature (the stalker) is steered toward your part of
//!   the level, never to your exact spot, arriving from out of sight, in a
//!   circle round you that tightens as the build-up goes on. The world
//!   announces it: its footsteps somewhere, lights failing near it, then the
//!   ambience going quiet as it closes in. If it has not found you when the
//!   time is up, it is told where you are, and again every few seconds
//!   for a last stretch. A Watcher sent this way
//!   comes on its own; its touch is the build-up's climax.
//! * **Peak**: something is hunting you, or searching close by. A menace
//!   gauge fills while it is close or in view; once full, and once you have
//!   broken away, everything after you withdraws.
//! * **Relax**: the creatures keep their distance and their senses dull for
//!   a while, then the cycle starts again, shorter on deeper levels.
//!
//! It runs per frame on the native random stream and never touches a
//! level's build. It also owns the first-sight sting (`entity_sighted`).
use crate::base::numeric::hypot2;
use crate::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Phase {
    #[default]
    Calm,
    Build,
    Peak,
    Relax,
}
impl Phase {
    pub fn name(self) -> &'static str {
        match self {
            Phase::Calm => "calm",
            Phase::Build => "build",
            Phase::Peak => "peak",
            Phase::Relax => "relax",
        }
    }
}

/// The most events one level's log keeps.
pub const LOG_CAP: usize = 4096;

#[derive(Clone, Debug, Default)]
pub struct Director {
    pub phase: Phase,
    /// Seconds left in the phase (Calm, Build, Relax)
    pub t: f64,
    /// The menace gauge (Peak): fills with closeness and visibility
    pub menace: f64,
    /// Smoothed 0..1: how close the threat feels (drives sound and screen)
    pub tension: f64,
    /// The creature being sent toward you (Build)
    pub stalker: Option<usize>,
    /// The one sent last time (the next build-up sends another if it can)
    pub last_sent: Option<usize>,
    /// Seconds to the stalker's next goal near you
    pub steer_t: f64,
    /// How long this build-up was given, s
    pub build_len: f64,
    /// The build-up's last stretch: the stalker has been told where you are
    pub final_push: bool,
    /// Seconds to the next sign of the stalker
    pub cue_t: f64,
    /// Seconds since a hunter was last close or in sight of you (Peak)
    pub lost_t: f64,
    /// Kinds already seen on this level (the first sight gets a sting)
    pub sighted: Vec<String>,
    /// Seconds before another sting may play
    pub sting_cd: f64,
    /// The cycle count on this level
    pub cycles: u32,
    /// Peaks and strikes you came through alive
    pub close_calls: u32,
    /// 0..1: how quiet the ambience should be (1 = only what matters)
    pub hush: f64,
    /// The creature that killed you
    pub killer: Option<usize>,
    /// Where the stalker was at the last progress check, and the time to the next
    pub stuck_at: Option<(f64, f64)>,
    /// What happened on this level, by game time: phase changes (`build`,
    /// `peak`, `relax`, `withdraw`), the stalker's signs (`cue:…`), `sting`,
    /// `notice`, `strike` and `death`. For run analysis; capped at `LOG_CAP`.
    pub log: Vec<(f64, &'static str)>,
    pub stuck_t: f64,
}

/// The first quiet on the first floor while her camp is there to find, s
pub const FIRST_FLOOR_CALM: (f64, f64) = (110.0, 150.0);
/// How soon a build-up follows the taking of her page, s
pub const WAKE: (f64, f64) = (4.0, 9.0);

/// How long the quiet lasts before the first creature is sent, by level:
/// long on the first floors, shorter deeper down.
fn calm_len(level: i32, first: bool) -> f64 {
    let depth = (1.0 - (level - 1) as f64 * 0.05).max(0.55);
    (if first { rand2(45.0, 80.0) } else { rand2(35.0, 65.0) }) * depth
}

impl Game {
    /// Start a level's cycle (called when the level is entered).
    /// Add an event to the director's log.
    pub fn director_note(&mut self, what: &'static str) {
        if self.director.log.len() < LOG_CAP {
            let t = self.perf_t;
            self.director.log.push((t, what));
        }
    }

    pub fn reset_director(&mut self) {
        let mut t = calm_len(self.level_num, true);
        // the first floor waits for you to find her: a long first quiet, cut
        // short when you take her page (`director_wake`)
        if self.level_num == 1 && self.story.camp.is_some() {
            t = rand2(FIRST_FLOOR_CALM.0, FIRST_FLOOR_CALM.1);
        }
        self.director = Director { t, ..Default::default() };
        self.amb_hush = 0.0;
    }

    /// Her page was taken: the place wakes. A quiet is cut short, so a
    /// build-up comes within `WAKE` seconds; already building or worse,
    /// nothing changes (PILLARS 5: you chose when the dread starts).
    pub fn director_wake(&mut self) {
        if self.director.phase == Phase::Calm {
            self.director.t = self.director.t.min(rand2(WAKE.0, WAKE.1));
            self.director_note("wake");
        }
    }

    /// One step of the director, after the creatures have moved.
    pub fn update_director(&mut self, dt: f64) {
        if self.state != "playing" || self.in_stairwell || self.in_loop {
            return;
        }
        self.director.sting_cd = (self.director.sting_cd - dt).max(0.0);
        let pp = self.player.pos;
        let mut hunters = 0;
        // still after you: hunting, or searching close by
        let mut engaged = 0;
        let mut nearest_hunt = 1e9f64;
        // pressed: a hunter is right behind you or has you in sight
        let mut pressed = false;
        for e in &self.entities {
            if e.state == "search" {
                let p = e.pos();
                if hypot2(p.x - pp.x, p.z - pp.z) < 25.0 {
                    engaged += 1;
                }
            }
            if e.state == "hunt" {
                hunters += 1;
                engaged += 1;
                let p = e.pos();
                let dh = hypot2(p.x - pp.x, p.z - pp.z);
                nearest_hunt = nearest_hunt.min(dh);
                if dh < 8.0 || (dh < 20.0 && self.e_los(p.x, p.z, pp.x, pp.z)) {
                    pressed = true;
                }
            }
        }
        // fear draws them: with your nerve gone, the next one comes now
        if self.player.sanity < 6.0 && matches!(self.director.phase, Phase::Calm | Phase::Relax) && self.director.t > 4.0 {
            self.director.t = 4.0;
        }
        match self.director.phase {
            Phase::Calm => {
                self.director.t -= dt;
                if hunters > 0 {
                    self.enter_peak();
                } else if self.director.t <= 0.0 {
                    self.enter_build();
                }
            }
            Phase::Build => {
                self.director.t -= dt;
                if hunters > 0 {
                    self.enter_peak();
                } else if self.director.t <= 0.0 && !self.director.final_push && self.final_push() {
                    // the stalker knows where you are now
                } else if self.director.t <= 0.0 || self.director.stalker.is_none() {
                    self.enter_relax(false);
                } else {
                    self.steer_stalker(dt);
                }
            }
            Phase::Peak => {
                if engaged == 0 {
                    // they have given up on you: the relief, and any straggler
                    // goes its way
                    self.enter_relax(true);
                } else {
                    let d = &mut self.director;
                    let near = clamp(1.0 - nearest_hunt / 12.0, 0.0, 1.0);
                    d.menace += dt * (0.045 + 0.1 * near + if pressed { 0.045 } else { 0.0 });
                    d.lost_t = if pressed { 0.0 } else { d.lost_t + dt };
                    // it has had its chance: once you break away (out of its
                    // sight and reach), they all back off; a little past the
                    // gauge they back off anyway unless right on you
                    if (d.menace >= 1.0 && d.lost_t > 2.5) || (d.menace >= 1.2 && nearest_hunt > 2.5) {
                        self.enter_relax(true);
                    }
                }
            }
            // blundering into one while they keep away is still a peak
            Phase::Relax if hunters > 0 => self.enter_peak(),
            Phase::Relax => {
                let level = self.level_num;
                let d = &mut self.director;
                d.t -= dt;
                d.menace = (d.menace - dt * 0.1).max(0.0);
                if d.t <= 0.0 {
                    d.phase = Phase::Calm;
                    d.t = calm_len(level, false);
                }
            }
        }
        // the felt tension: how close the nearest creature that matters is
        let mut target: f64 = 0.0;
        for (i, e) in self.entities.iter().enumerate() {
            if e.kind == "gone" || (e.kind == "staff" && self.store_state.phase != "night") {
                continue;
            }
            let p = e.pos();
            let dd = hypot2(p.x - pp.x, p.z - pp.z);
            let k = match e.state.as_str() {
                "hunt" => clamp(1.25 - dd / 22.0, 0.35, 1.0),
                "search" => clamp(1.1 - dd / 20.0, 0.2, 0.85),
                _ if Some(i) == self.director.stalker => clamp(1.0 - dd / 30.0, 0.0, 0.7),
                _ => clamp(1.0 - dd / 14.0, 0.0, 0.4),
            };
            target = target.max(k);
        }
        let rate = if target > self.director.tension { 1.6 } else { 0.35 };
        self.director.tension += (target - self.director.tension) * (dt * rate).min(1.0);
        // the place holds its breath while something comes, and while
        // something searches close by; in a chase its own noise is enough
        let mut hush: f64 = 0.0;
        if let Some(si) = self.director.stalker {
            if let Some(e) = self.entities.get(si) {
                let p = e.pos();
                hush = clamp((24.0 - hypot2(p.x - pp.x, p.z - pp.z)) / 14.0, 0.0, 1.0);
            }
        }
        for e in &self.entities {
            let p = e.pos();
            let dd = hypot2(p.x - pp.x, p.z - pp.z);
            match e.state.as_str() {
                "search" => hush = hush.max(clamp((20.0 - dd) / 12.0, 0.0, 0.85)),
                "hunt" => hush = hush.max(0.5),
                _ => {}
            }
        }
        self.director.hush += (hush - self.director.hush) * (dt * if hush > self.director.hush { 0.8 } else { 0.25 }).min(1.0);
        self.amb_hush = self.director.hush;
    }

    fn enter_peak(&mut self) {
        self.director_note("peak");
        let d = &mut self.director;
        d.phase = Phase::Peak;
        d.menace = 0.0;
        d.lost_t = 0.0;
        d.stalker = None;
    }

    fn enter_relax(&mut self, withdraw: bool) {
        self.director_note(if withdraw { "withdraw" } else { "relax" });
        let level = self.level_num;
        let came_through = self.director.phase == Phase::Peak && self.state == "playing";
        let d = &mut self.director;
        if came_through {
            d.close_calls += 1;
        }
        d.phase = Phase::Relax;
        d.cycles += 1;
        d.t = rand2(25.0, 45.0) * (1.0 - (level - 1) as f64 * 0.04).max(0.6);
        if let Some(si) = d.stalker.take() {
            if let Some(e) = self.entities.get_mut(si) {
                e.set_fl("stalk", false);
                e.w_field = None;
            }
        }
        if withdraw {
            let pp = self.player.pos;
            for i in 0..self.entities.len() {
                let e = &self.entities[i];
                let p = e.pos();
                let after_you = matches!(e.state.as_str(), "hunt" | "search") || hypot2(p.x - pp.x, p.z - pp.z) < 25.0;
                if after_you && matches!(e.kind.as_str(), "smiler" | "hound") {
                    self.withdraw_entity(i);
                }
            }
        }
    }

    /// Send a creature away: a far, dark goal, and a while before it
    /// notices anything again.
    pub fn withdraw_entity(&mut self, ei: usize) {
        let pp = self.player.pos;
        let mut best: Option<(i32, i32)> = None;
        let mut best_score = -1e9;
        for _ in 0..16 {
            let gx = irand(1, self.w as i64 - 2) as i32;
            let gz = irand(1, self.h as i64 - 2) as i32;
            if self.is_wall_tile(gx, gz) {
                continue;
            }
            let dp = hypot2(self.wx(gx) - pp.x, self.wz(gz) - pp.z);
            let score = dp.min(60.0) - self.e_region_light_at(gx, gz) * 10.0;
            if score > best_score {
                best_score = score;
                best = Some((gx, gz));
            }
        }
        let field = best.map(|(gx, gz)| Rc::new(self.e_bfs(gx, gz)));
        let e = &mut self.entities[ei];
        e.state = "wander".into();
        e.w_target = best;
        e.w_field = field;
        e.w_timer = 14.0;
        e.lose_t = 0.0;
        e.set_n("retreatT", 12.0);
        e.set_fl("stalk", false);
    }

    fn enter_build(&mut self) {
        self.director_note("build");
        let pp = self.player.pos;
        // the nearest of the ones that hunt (the Watcher now and then), and
        // not the one sent last time when another can go
        let mut pick: Option<(usize, f64, f64)> = None;
        let last = self.director.last_sent;
        let others = self.entities.iter().enumerate().filter(|(i, e)| Some(*i) != last && e.state == "wander").count() > 0;
        for (i, e) in self.entities.iter().enumerate() {
            if !matches!(e.kind.as_str(), "smiler" | "hound" | "watcher") || e.state != "wander" {
                continue;
            }
            if others && Some(i) == last {
                continue;
            }
            let p = e.pos();
            let dd = hypot2(p.x - pp.x, p.z - pp.z);
            let score = dd + if e.kind == "watcher" { 60.0 } else { 0.0 };
            if pick.map(|(_, _, b)| score < b).unwrap_or(true) {
                pick = Some((i, dd, score));
            }
        }
        let pick = pick.map(|(i, dd, _)| (i, dd));
        let d = &mut self.director;
        let Some((si, dist)) = pick else {
            // nothing to send: stay quiet a little longer
            d.t = rand2(15.0, 25.0);
            return;
        };
        d.phase = Phase::Build;
        // time to arrive, and a while to prowl once it has
        d.t = clamp(dist / 3.4, 10.0, 60.0) + rand2(25.0, 40.0);
        d.build_len = d.t;
        d.final_push = false;
        d.stalker = Some(si);
        d.last_sent = Some(si);
        d.steer_t = 0.0;
        d.cue_t = rand2(3.0, 6.0);
        d.stuck_at = None;
        d.stuck_t = 6.0;
        self.entities[si].set_fl("stalk", true);
        self.entities[si].set_n("retreatT", 0.0);
    }

    /// Keep the stalker coming: every few seconds a new goal 7–15 m from
    /// you, preferring tiles you cannot see, and a sign of it now and then.
    fn steer_stalker(&mut self, dt: f64) {
        let Some(si) = self.director.stalker else { return };
        if si >= self.entities.len() || !matches!(self.entities[si].state.as_str(), "wander" | "search") {
            self.director.stalker = None;
            return;
        }
        // it heard you and is looking: let it look; in the last stretch it is
        // told again every few seconds where you are now
        if self.entities[si].state == "search" {
            if self.director.final_push {
                self.director.steer_t -= dt;
                if self.director.steer_t <= 0.0 {
                    self.director.steer_t = 4.0;
                    let pp = self.player.pos;
                    let e = &mut self.entities[si];
                    e.set_n("seenX", pp.x + rand2(-2.0, 2.0));
                    e.set_n("seenZ", pp.z + rand2(-2.0, 2.0));
                    e.set_n("searchR", 3.0);
                    e.set_n("searchT", e.n("searchT").max(6.0));
                    e.set_n("lookT", 0.0);
                    e.w_target = None;
                }
            }
            return;
        }
        // the Watcher needs no leading: unwatched, it comes on its own
        let watcher = self.entities[si].kind == "watcher";
        let pp = self.player.pos;
        let sp = self.entities[si].pos();
        let ds = hypot2(sp.x - pp.x, sp.z - pp.z);
        // wedged on something on its way: it gets loose while nobody watches
        self.director.stuck_t -= dt;
        if self.director.stuck_t <= 0.0 {
            self.director.stuck_t = 6.0;
            let moved = self.director.stuck_at.map(|(x, z)| hypot2(sp.x - x, sp.z - z)).unwrap_or(1e9);
            if moved < 0.5 && ds > 8.0 && !self.e_los(sp.x, sp.z, pp.x, pp.z) {
                self.unstick_entity(si);
            }
            let p = self.entities[si].pos();
            self.director.stuck_at = Some((p.x, p.z));
        }
        self.director.steer_t -= dt;
        if self.director.steer_t <= 0.0 && !watcher {
            self.director.steer_t = rand2(5.0, 8.0);
            // the circle it prowls tightens as the build-up goes on
            let k = clamp(1.0 - self.director.t / self.director.build_len.max(1.0), 0.0, 1.0);
            let (near, far) = (lerp(6.0, 3.0, k), lerp(16.0, 8.0, k));
            let (ptx, ptz) = (self.tx(pp.x), self.tz(pp.z));
            let mut best: Option<(i32, i32)> = None;
            let mut best_score = -1e9;
            for _ in 0..20 {
                let gx = ptx + irand(-4, 4) as i32;
                let gz = ptz + irand(-4, 4) as i32;
                if !self.in_grid(gx, gz) || self.is_wall_tile(gx, gz) {
                    continue;
                }
                let (wx, wz) = (self.wx(gx), self.wz(gz));
                let dp = hypot2(wx - pp.x, wz - pp.z);
                if !(near..=far).contains(&dp) {
                    continue;
                }
                let hidden = !self.e_los(pp.x, pp.z, wx, wz);
                // out of your sight, on its side of you, in the dark
                let toward = -hypot2(wx - sp.x, wz - sp.z) * 0.05;
                let score = if hidden { 2.0 } else { 0.0 } + toward - self.e_region_light_at(gx, gz) + rnd() * 0.5;
                if score > best_score {
                    best_score = score;
                    best = Some((gx, gz));
                }
            }
            if let Some((gx, gz)) = best {
                let f = Rc::new(self.e_bfs(gx, gz));
                let e = &mut self.entities[si];
                e.w_target = Some((gx, gz));
                e.w_field = Some(f);
                e.w_timer = 12.0;
            }
        }
        self.director.cue_t -= dt;
        if self.director.cue_t <= 0.0 {
            self.director.cue_t = lerp(4.0, 9.0, clamp(ds / 30.0, 0.0, 1.0)) * rand2(0.8, 1.2);
            self.presence_cue(si, ds);
        }
    }

    /// A sign that something is coming, from where it really is: steps far
    /// off, then the lights near it failing, then nothing at all.
    fn presence_cue(&mut self, si: usize, ds: f64) {
        let sp = self.entities[si].pos();
        // the Watcher makes no sound: only the lights over it tell
        if self.entities[si].kind == "watcher" && ds > 9.0 {
            self.director_note("cue:lights");
            self.flicker_near(sp.x, sp.z);
            return;
        }
        if ds > 20.0 {
            self.director_note("cue:steps");
            self.entity_steps(si, irand(3, 5) as i32, 0.9);
        } else if ds > 9.0 {
            self.flicker_near(sp.x, sp.z);
            if rnd() < 0.6 {
                self.director_note("cue:lights+steps");
                self.entity_steps(si, 2, 0.6);
            } else {
                self.director_note("cue:lights");
            }
        } else if rnd() < 0.5 {
            // close: one sound, and then nothing
            self.director_note("cue:one step");
            self.entity_steps(si, 1, 0.45);
        } else {
            self.director_note("cue:silence");
        }
    }

    /// `n` of a creature's footfalls from where it is (spatial, muffled
    /// through walls), at its own gait.
    pub fn entity_steps(&mut self, ei: usize, n: i32, loud: f64) {
        let Some(s) = self.soundscape() else { return };
        let id = self.entities[ei].g.id();
        let Some(v) = self.ent_ext.get(&id).and_then(|x| x.voice.as_ref()).map(|v| v.slot.input.clone()) else { return };
        let kind = self.entities[ei].kind.clone();
        let t0 = self.ac_time();
        let mut t = t0;
        for k in 0..n {
            match kind.as_str() {
                "hound" => {
                    // a quick padded scramble, claws on the floor
                    s.burst(&v, t, 0.06, "lowpass", 300.0, 0.8, 0.22 * loud, None, 0.002);
                    s.burst(&v, t + 0.01, 0.03, "highpass", 3500.0, 1.0, 0.05 * loud, None, 0.001);
                    t += if k % 2 == 0 { 0.13 } else { 0.21 };
                }
                _ => {
                    // long-legged and uneven: a heavy heel, a dragged toe
                    s.burst(&v, t, 0.1, "lowpass", 220.0, 0.9, 0.32 * loud, None, 0.002);
                    s.tone(&v, t, 0.12, 70.0, 0.16 * loud, "sine", Some(42.0), 0.003);
                    s.burst(&v, t + 0.09, 0.22, "bandpass", 900.0, 1.6, 0.035 * loud, Some(500.0), 0.03);
                    t += if k % 2 == 0 { 0.62 } else { 0.94 };
                }
            }
        }
    }

    /// You see creature `ei` for the first time on this level: a sting,
    /// your heart, and a blow to your nerve.
    pub fn entity_sighted(&mut self, ei: usize, d: f64) {
        let kind = self.entities[ei].kind.clone();
        if self.director.sighted.contains(&kind) {
            return;
        }
        self.director.sighted.push(kind);
        self.player.sanity = clamp(self.player.sanity - 6.0 - 6.0 * clamp(1.0 - d / 20.0, 0.0, 1.0), 0.0, SAN_MAX);
        self.dread_sting(0.7);
        self.sfx_thump(0.9);
    }

    /// A dissonant hit: low clustered strings, a sub drop and a scrape of
    /// high noise, never the same chord twice in a row.
    pub fn dread_sting(&mut self, intensity: f64) {
        self.director_note("sting");
        if self.director.sting_cd > 0.0 {
            return;
        }
        self.director.sting_cd = 5.0;
        let Some(s) = self.soundscape() else { return };
        let t = self.ac_time();
        let root = rand2(98.0, 131.0);
        for (k, ratio) in [1.0, 1.059, 1.414, 1.498 * 1.059].iter().enumerate() {
            let f = root * ratio;
            s.tone(&s.dry, t, 2.6, f, 0.045 * intensity, "sawtooth", Some(f * 0.985), 0.01 + k as f64 * 0.012);
        }
        s.tone(&s.dry, t, 1.4, 52.0, 0.3 * intensity, "sine", Some(31.0), 0.004);
        s.burst(&s.dry, t, 1.1, "highpass", 4200.0, 0.7, 0.05 * intensity, Some(1800.0), 0.004);
    }

    /// Creature `ei` has found you: it stops dead with its head snapped to
    /// you, screams from where it stands, and then it comes.
    pub fn entity_notice(&mut self, ei: usize, d: f64) {
        self.director_note("notice");
        let pp = self.player.pos;
        let pos = self.entities[ei].pos();
        let kind = self.entities[ei].kind.clone();
        {
            let e = &mut self.entities[ei];
            e.state = "hunt".into();
            e.lose_t = 0.0;
            e.yaw = (pp.x - pos.x).atan2(pp.z - pos.z);
            e.g.rotation().y = e.yaw;
            e.vx = 0.0;
            e.vz = 0.0;
            e.freeze = if kind == "hound" { 0.35 } else { 0.6 };
            e.set_n("noticeT", 1.0);
            e.set_n("chargeT", 2.2);
            e.set_n("seenX", pp.x);
            e.set_n("seenZ", pp.z);
            e.set_fl("stalk", false);
        }
        self.entity_call(ei, "shriek");
        // the lights around you stutter as it turns
        for p in self.panels.iter_mut() {
            if hypot2(p.wx - pp.x, p.wz - pp.z) < 10.0 && !p.dead {
                p.flicker = p.flicker.max(rand2(0.35, 0.8));
            }
        }
        if d < 30.0 {
            self.dread_sting(clamp(1.3 - d / 30.0, 0.5, 1.0));
            self.shake_t = self.shake_t.max(0.3);
            self.player.sanity = clamp(self.player.sanity - 8.0, 0.0, SAN_MAX);
        }
    }

    /// The creature lost you or heard you without seeing you: it goes to
    /// where it thinks you are and looks, closer each time.
    pub fn entity_search(&mut self, ei: usize, x: f64, z: f64, spread: f64, secs: f64) {
        let e = &mut self.entities[ei];
        e.state = "search".into();
        e.lose_t = 0.0;
        e.set_n("seenX", x);
        e.set_n("seenZ", z);
        e.set_n("searchT", secs);
        e.set_n("searchR", spread);
        e.set_n("lookT", 0.0);
        e.w_field = None;
        e.w_target = None;
    }

    /// One step of a search: walk to the next spot near where it last
    /// knew you were, stop and listen, pick a closer spot.
    pub fn search_step(&mut self, ei: usize, dt: f64) {
        let (lx, lz, r) = {
            let e = &self.entities[ei];
            (e.n("seenX"), e.n("seenZ"), e.n("searchR"))
        };
        let pos = self.entities[ei].pos();
        let look = self.entities[ei].n("lookT");
        if look > 0.0 {
            let e = &mut self.entities[ei];
            e.set_n("lookT", look - dt);
            // it turns its head about while it listens
            let yaw = e.yaw + (look * 2.3).sin() * dt * 1.6;
            e.yaw = yaw;
            self.entity_drive(ei, dt, 0.0);
            return;
        }
        let need = match self.entities[ei].w_target {
            None => true,
            Some((gx, gz)) => self.tx(pos.x) == gx && self.tz(pos.z) == gz,
        };
        if need {
            if self.entities[ei].w_target.is_some() {
                let e = &mut self.entities[ei];
                e.set_n("lookT", rand2(1.2, 2.4));
                e.set_n("searchR", (r * 0.7).max(2.5));
            }
            let mut goal = None;
            for _ in 0..10 {
                let a = rnd() * PI * 2.0;
                let rr = rnd().sqrt() * r;
                let (gx, gz) = (self.tx(lx + a.cos() * rr), self.tz(lz + a.sin() * rr));
                if self.in_grid(gx, gz) && !self.is_wall_tile(gx, gz) {
                    goal = Some((gx, gz));
                    break;
                }
            }
            let goal = goal.unwrap_or((self.tx(lx), self.tz(lz)));
            let f = Rc::new(self.e_bfs(goal.0, goal.1));
            let e = &mut self.entities[ei];
            e.w_target = Some(goal);
            e.w_field = Some(f);
            return;
        }
        // careful while it searches; with urgency when the director has told it where you are
        let urgent = self.director.final_push && self.director.stalker == Some(ei);
        let speed = self.entities[ei].speed
            * if urgent {
                0.7
            } else if self.entities[ei].kind == "hound" {
                0.5
            } else {
                0.42
            };
        if let Some(wf) = self.entities[ei].w_field.clone() {
            if self.entity_step_toward(ei, Some(wf), dt, speed, None) {
                self.entities[ei].w_target = Some((self.tx(pos.x), self.tz(pos.z)));
            }
        }
    }

    /// The index of the creature nearest to you, if any.
    pub fn nearest_entity(&self) -> Option<usize> {
        let pp = self.player.pos;
        let mut best: Option<(usize, f64)> = None;
        for (i, e) in self.entities.iter().enumerate() {
            if e.kind == "gone" {
                continue;
            }
            let p = e.pos();
            let d = hypot2(p.x - pp.x, p.z - pp.z);
            if best.map(|(_, b)| d < b).unwrap_or(true) {
                best = Some((i, d));
            }
        }
        best.map(|(i, _)| i)
    }

    /// After death: the camera is held on the killer's face, shaking, while
    /// it keeps moving (jaw, hands) until the screen cuts to black.
    pub fn update_death_cam(&mut self, dt: f64) {
        let Some(ki) = self.director.killer.filter(|&i| i < self.entities.len()) else { return };
        let t = self.perf_t;
        self.animate_entity(ki, dt, t, 0.8);
        if let Some(sk) = self.entities[ki].parts.get("skin") {
            crate::entities::sync_skin_bind(sk);
        }
        let head = match self.entities[ki].parts.get("head") {
            Some(h) => h.world_position(),
            None => {
                let p = self.entities[ki].pos();
                v3(p.x, self.entities[ki].face_y, p.z)
            }
        };
        let cam = self.camera.clone();
        let c = cam.pos();
        let (dx, dy, dz) = (head.x - c.x, head.y - c.y, head.z - c.z);
        self.shake_t = (self.shake_t - dt).max(0.0);
        let shake = self.shake_t.max(0.25) * 0.05;
        cam.rotation().y = (-dx).atan2(-dz) + rand2(-1.0, 1.0) * shake;
        cam.rotation().x = dy.atan2(hypot2(dx, dz)) + rand2(-1.0, 1.0) * shake;
        cam.rotation().z = rand2(-1.0, 1.0) * shake * 0.6;
    }

    /// Move creature `ei` to the centre of the nearest tile where its body
    /// fits (its own first), when something has trapped it.
    pub fn unstick_entity(&mut self, ei: usize) {
        let p = self.entities[ei].pos();
        let (cx, cz) = (self.tx(p.x), self.tz(p.z));
        for r in 0..4i32 {
            for dz in -r..=r {
                for dx in -r..=r {
                    if dx.abs().max(dz.abs()) != r {
                        continue;
                    }
                    let (x, z) = (cx + dx, cz + dz);
                    if !self.in_grid(x, z) || self.is_wall_tile(x, z) {
                        continue;
                    }
                    let (wx, wz) = (self.wx(x), self.wz(z));
                    if !self.e_circle_hits_wall(wx, wz, 0.4) {
                        let e = &mut self.entities[ei];
                        e.g.set_pos(wx, 0.0, wz);
                        e.vx = 0.0;
                        e.vz = 0.0;
                        e.set_n("aimT", 0.0);
                        return;
                    }
                }
            }
        }
    }

    /// The panels over a spot stutter.
    fn flicker_near(&mut self, x: f64, z: f64) {
        let mut near: Vec<(usize, f64)> = self.panels.iter().enumerate().map(|(i, p)| (i, hypot2(p.wx - x, p.wz - z))).collect();
        near.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        for &(i, dp) in near.iter().take(3) {
            if dp < 10.0 {
                self.panels[i].flicker = rand2(0.6, 1.1);
            }
        }
    }

    /// The Watcher the director sent has touched you: that was its moment.
    /// It falls back and keeps away while the place settles.
    pub fn director_strike(&mut self, ei: usize) {
        self.director_note("strike");
        if self.director.stalker != Some(ei) {
            return;
        }
        if self.state == "playing" {
            self.director.close_calls += 1;
        }
        self.enter_relax(false);
    }

    /// The build-up ran out without it finding you: it is told where you
    /// are (and again every few seconds) and gets a last stretch to come and
    /// look, with some urgency. False when there
    /// is no stalker that can be sent (or it is the Watcher, which needs no
    /// telling).
    fn final_push(&mut self) -> bool {
        let Some(si) = self.director.stalker.filter(|&i| i < self.entities.len()) else { return false };
        if self.entities[si].kind == "watcher" || self.entities[si].state != "wander" {
            return false;
        }
        let pp = self.player.pos;
        self.entity_search(si, pp.x + rand2(-2.0, 2.0), pp.z + rand2(-2.0, 2.0), 3.0, 15.0);
        self.entity_call(si, "listen");
        self.director.final_push = true;
        self.director.steer_t = 4.0;
        self.director.t = 25.0;
        true
    }
}
