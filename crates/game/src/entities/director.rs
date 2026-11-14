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
/// A stalker further off than this (straight, m) gives no sign.
const CUE_REACH: f64 = 40.0;
/// Something searching within this (straight, m) is still after you (a
/// peak goes on), and you hold your breath for it
pub const ENGAGED_NEAR: f64 = 25.0;
/// You hold your breath while the one sent in a build-up is this near, m
pub const STALKER_NEAR: f64 = 20.0;
/// ... and while the place holds its own this much (`amb_hush`: something
/// coming, a search near, a silent machine, the fog up): silence is a
/// warning, and you keep it (PILLARS 3; `lore-keeper`, asides A4)
pub const HUSHED: f64 = 0.3;
/// ... and this long after your cry, s: you listen for what answers (the
/// place answers in 0.6-0.9 s, `Game::cry_for_help`)
pub const AFTER_CRY: f64 = 3.0;
/// ... and while something that hears, not yet after you, is this near, m
/// (a Hound hears your step from about this far; the store's staff by
/// night): your silence is what keeps it so (review B)
pub const HEARS_NEAR: f64 = 15.0;
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
    /// A creature's strikes that reached you in this peak (the second kills:
    /// `creature::creature_contact`)
    pub contacts: u32,
    /// This build-up ends in nothing: a sign, then quiet (`EMPTY_SHARE`)
    pub empty: bool,
    /// The next build-up was called (her page, a false trail's end): it is real
    pub called: bool,
    /// Signs given in this build-up, and the quiet since the last
    pub cues: u32,
    pub quiet_t: f64,
}

/// The share of build-ups that end in nothing: a sign, then quiet (design 5:
/// a sign is never a promise).
pub const EMPTY_SHARE: f64 = 0.3;
/// After its sign, an empty build-up stays quiet this long, then the stalker
/// goes its way, s.
pub const EMPTY_QUIET: f64 = 6.0;
/// A stalker further than this along the level is brought nearer before it
/// is sent (out of your sight), to this far from you along the way, m.
pub const FAR_SEND: f64 = 60.0;
pub const SEND_FROM: (f64, f64) = (30.0, 45.0);

/// The first quiet on the first floor while her camp is there to find, s
pub const FIRST_FLOOR_CALM: (f64, f64) = (110.0, 150.0);
/// How soon a build-up follows the end of a false trail, s
pub const LURE: (f64, f64) = (1.5, 3.5);
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
        let mut t = calm_len(self.depth(), true);
        // the first floor waits for you to find her: a long first quiet, cut
        // short when you take her page (`director_wake`). The same draw, spread
        // over the longer span (another draw would shift every later one)
        if self.level_num == crate::level::building::FIRST_ROOMS_LEVEL && self.story.camp.is_some() {
            let u = (t - 45.0) / 35.0;
            t = FIRST_FLOOR_CALM.0 + u * (FIRST_FLOOR_CALM.1 - FIRST_FLOOR_CALM.0);
        }
        self.director = Director { t, ..Default::default() };
        self.amb_hush = 0.0;
        // the Building's floors: what comes is the self-assembly (`world::assembly`)
        if self.building.is_some() {
            self.reset_assembly();
        }
    }

    /// You followed a trail that is not hers to its end: in a quiet,
    /// something is sent within `LURE` s; in a build-up, the stalker is told
    /// where you are now (its last stretch).
    pub fn director_lure(&mut self) {
        self.director_note("lure");
        match self.director.phase {
            Phase::Calm => {
                self.director.t = self.director.t.min(rand2(LURE.0, LURE.1));
                self.director.called = true;
            }
            Phase::Build => self.director.t = self.director.t.min(0.0),
            _ => {}
        }
    }

    /// Her page was taken: the place wakes. A quiet is cut short, so a
    /// build-up comes within `WAKE` seconds; already building or worse,
    /// nothing changes (PILLARS 5: you chose when the dread starts).
    pub fn director_wake(&mut self) {
        if self.building.is_some() {
            if self.director.phase == Phase::Calm {
                self.assembly_wake(crate::world::assembly::PAGE_WAKE);
                self.director_note("wake");
            }
            return;
        }
        if self.director.phase == Phase::Calm {
            self.director.t = self.director.t.min(rand2(WAKE.0, WAKE.1));
            self.director.called = true;
            self.director_note("wake");
        }
    }

    /// One step of the director, after the creatures have moved.
    pub fn update_director(&mut self, dt: f64) {
        if self.state != "playing" || self.in_stairwell || self.in_loop {
            return;
        }
        // nothing walks the Building's floors: the self-assembly comes instead,
        // and the place holds its breath while the fog is up
        if self.building.is_some() {
            self.update_assembly(dt);
            self.update_cell_doors(dt);
            self.update_far_figure(dt);
            self.update_knocks(dt);
            let hush = if self.assembly.up() { 0.55 } else { 0.0 };
            self.director.hush += (hush - self.director.hush) * (dt * 0.4).min(1.0);
            self.amb_hush = self.director.hush;
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
                if hypot2(p.x - pp.x, p.z - pp.z) < ENGAGED_NEAR {
                    engaged += 1;
                }
            }
            // (one already sent away, finishing its chase, starts no new peak)
            if e.state == "hunt" && e.n("retreatT") > 0.0 {
                engaged += 1;
                continue;
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
                let d = &mut self.director;
                if d.empty && d.cues > 0 {
                    d.quiet_t += dt;
                }
                if hunters > 0 {
                    self.enter_peak();
                } else if self.director.empty && (self.director.quiet_t > EMPTY_QUIET || self.director.t <= 0.0) {
                    // a sign, then quiet: it goes its way, and nothing came
                    self.director_note("nothing");
                    if let Some(si) = self.director.stalker {
                        self.withdraw_entity(si);
                    }
                    self.enter_relax(false);
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
                let level = self.depth();
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
        if let Some(si) = self.director.stalker
            && let Some(e) = self.entities.get(si)
        {
            let p = e.pos();
            hush = clamp((24.0 - hypot2(p.x - pp.x, p.z - pp.z)) / 14.0, 0.0, 1.0);
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
        // a silent vending machine (VENDING.md rule 5) hushes the room round it
        let hush = hush.max(0.85 * self.vending_hush(pp.x, pp.z));
        self.director.hush += (hush - self.director.hush) * (dt * if hush > self.director.hush { 0.8 } else { 0.25 }).min(1.0);
        self.amb_hush = self.director.hush;
    }

    /// Whether you hold your breath now, and why (your asides' rule 4,
    /// `audio::asides`): the pacing director answers, since it knows what
    /// is coming. Just after your cry; and on a floor, while something
    /// hunts you, searches within `ENGAGED_NEAR`, or the one it sent is
    /// within `STALKER_NEAR` in a build-up, and while the place is hushed
    /// (`HUSHED`). In the Stairwell or the Loop only your cry: the floor's
    /// creatures and its hush are not there.
    pub fn holding_breath(&self) -> Option<&'static str> {
        if self.cry_last_at.is_some_and(|c| self.perf_t - c < AFTER_CRY) {
            return Some("cried");
        }
        // in the Stairwell, the Face out on its landing; in the Loop, nothing
        if self.in_stairwell {
            return self.stairwell.as_ref().is_some_and(|s| s.st.face.is_some()).then_some("faced");
        }
        if self.in_loop {
            return None;
        }
        // (the creatures before the hush: a hunt that begins in a hush is
        // still a hunt, and cuts a line off: review B)
        let pp = self.player.pos;
        for (i, e) in self.entities.iter().enumerate() {
            if e.kind == "gone" {
                continue;
            }
            let p = e.pos();
            let d = hypot2(p.x - pp.x, p.z - pp.z);
            match e.state.as_str() {
                "hunt" => return Some("hunted"),
                "search" if d < ENGAGED_NEAR => return Some("searched"),
                _ if self.director.phase == Phase::Build && self.director.stalker == Some(i) && d < STALKER_NEAR => return Some("stalked"),
                _ if d < HEARS_NEAR && (e.kind == "hound" || (e.kind == "staff" && self.store_state.phase == "night")) => {
                    return Some("near");
                }
                _ => {}
            }
        }
        if self.amb_hush >= HUSHED {
            return Some("hushed");
        }
        None
    }

    fn enter_peak(&mut self) {
        self.director_note("peak");
        let d = &mut self.director;
        d.phase = Phase::Peak;
        d.contacts = 0;
        d.menace = 0.0;
        d.lost_t = 0.0;
        d.stalker = None;
    }

    fn enter_relax(&mut self, withdraw: bool) {
        self.director_note(if withdraw { "withdraw" } else { "relax" });
        let level = self.depth();
        let came_through = self.director.phase == Phase::Peak && self.state == "playing";
        let d = &mut self.director;
        if came_through {
            d.close_calls += 1;
        }
        let relief = came_through;
        d.contacts = 0;
        d.phase = Phase::Relax;
        d.cycles += 1;
        d.t = rand2(25.0, 45.0) * (1.0 - (level - 1) as f64 * 0.04).max(0.6);
        if let Some(si) = d.stalker.take()
            && let Some(e) = self.entities.get_mut(si)
        {
            e.set_fl("stalk", false);
            e.w_field = None;
        }
        if withdraw {
            let pp = self.player.pos;
            for i in 0..self.entities.len() {
                let e = &self.entities[i];
                let p = e.pos();
                let after_you = matches!(e.state.as_str(), "hunt" | "search") || hypot2(p.x - pp.x, p.z - pp.z) < 25.0;
                if after_you && matches!(e.kind.as_str(), "smiler" | "hound" | "reader") {
                    self.withdraw_entity(i);
                }
            }
        }
        // your asides: the relief after a close call, said once your breath
        // is let go
        if relief {
            self.aside(crate::audio::asides::Use::Aftermath, "");
        }
    }

    /// The relief, now: everything after you backs off (a creature's strike
    /// was its moment).
    pub fn director_relax_now(&mut self) {
        if self.director.phase != Phase::Relax {
            self.enter_relax(true);
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

    /// When creature `si` is further than `FAR_SEND` m from you along the
    /// level, it is moved, out of your sight, to the open tile nearest it
    /// that is `SEND_FROM` m from you along the way → its new straight
    /// distance, or None when it was near enough (or nowhere fitted).
    fn bring_nearer(&mut self, si: usize) -> Option<f64> {
        let pf = self.player_field.clone()?;
        let p = self.entities[si].pos();
        let (tx, tz) = self.e_nav_tile_at(p.x, p.z);
        let steps = pf.get(self.t_idx(tx, tz)).copied().unwrap_or(-1);
        if steps >= 0 && steps as f64 * CELL <= FAR_SEND {
            return None;
        }
        let pp = self.player.pos;
        let mut best: Option<(f64, i32, i32)> = None;
        for z in 1..self.h - 1 {
            for x in 1..self.w - 1 {
                let k = pf.get(self.t_idx(x, z)).copied().unwrap_or(-1);
                if k < 0 || !(SEND_FROM.0..SEND_FROM.1).contains(&(k as f64 * CELL)) || self.is_wall_tile(x, z) {
                    continue;
                }
                let (wx, wz) = (self.wx(x), self.wz(z));
                if self.e_los(pp.x, pp.z, wx, wz) || self.e_circle_hits_wall(wx, wz, 0.4) {
                    continue;
                }
                let d = hypot2(wx - p.x, wz - p.z);
                if best.is_none_or(|b| d < b.0) {
                    best = Some((d, x, z));
                }
            }
        }
        let (_, x, z) = best?;
        let (wx, wz) = (self.wx(x), self.wz(z));
        let e = &mut self.entities[si];
        e.g.set_pos(wx, 0.0, wz);
        e.vx = 0.0;
        e.vz = 0.0;
        e.w_field = None;
        e.w_target = None;
        self.director_note("brought nearer");
        Some(hypot2(wx - pp.x, wz - pp.z))
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
            if !matches!(e.kind.as_str(), "smiler" | "hound" | "watcher" | "reader") || e.state != "wander" {
                continue;
            }
            if others && Some(i) == last {
                continue;
            }
            let p = e.pos();
            let dd = hypot2(p.x - pp.x, p.z - pp.z);
            let score = dd + if e.kind == "watcher" { 60.0 } else { 0.0 };
            if pick.is_none_or(|(_, _, b)| score < b) {
                pick = Some((i, dd, score));
            }
        }
        let pick = pick.map(|(i, dd, _)| (i, dd));
        // sent from far off, it would not arrive in time: out of your sight it
        // is brought within reach first (its signs still come from where it is)
        let pick = pick.map(|(si, dd)| (si, self.bring_nearer(si).unwrap_or(dd)));
        let d = &mut self.director;
        let Some((si, dist)) = pick else {
            // nothing to send: stay quiet a little longer
            d.t = rand2(15.0, 25.0);
            return;
        };
        d.phase = Phase::Build;
        // some build-ups end in nothing (never one her page or a trail called)
        d.empty = !d.called && rnd() < EMPTY_SHARE;
        d.called = false;
        d.cues = 0;
        d.quiet_t = 0.0;
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
        // it heard you and is looking: let it look (in the last stretch its
        // orders tell it where you are now: `Order::Seek`, taken as a hint
        // at most every few seconds by its kind)
        if self.entities[si].state == "search" {
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
            let moved = self.director.stuck_at.map_or(1e9, |(x, z)| hypot2(sp.x - x, sp.z - z));
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
        // out of earshot it gives no sign: the place holds its breath until
        // something is near enough to be heard (a sign must mean it)
        if ds > CUE_REACH {
            return;
        }
        // an empty build-up gives one sign, then nothing
        if self.director.empty && self.director.cues > 0 {
            return;
        }
        self.director.cues += 1;
        self.director.quiet_t = 0.0;
        let sp = self.entities[si].pos();
        // the Watcher makes no sound: only the lights over it tell
        if self.entities[si].kind == "watcher" && ds > 9.0 {
            self.director_note("cue:lights");
            self.flicker_near(sp.x, sp.z);
            return;
        }
        if ds > 20.0 {
            self.director_note("cue:steps");
            let n = irand(3, 5) as i32;
            self.entity_steps(si, n, 0.9);
            // steps far off: your asides may answer the first of a run, once
            // they have stopped (a word over them would hide where they are)
            let gait = match self.entities[si].kind.as_str() {
                "hound" => 0.17,
                "reader" => 0.52,
                _ => 0.78,
            };
            self.aside_later((n as f64 * gait + 0.4) * 1000.0, crate::audio::asides::Use::Startle, "creature_steps");
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
                "reader" => {
                    // soft soles, shuffling: a scuff, a page turned
                    s.burst(&v, t, 0.07, "bandpass", 1200.0, 1.1, 0.12 * loud, Some(800.0), 0.004);
                    if k == n - 1 {
                        s.burst(&v, t + 0.25, 0.18, "bandpass", 3600.0, 1.0, 0.05 * loud, Some(2800.0), 0.01);
                    }
                    t += if k % 2 == 0 { 0.48 } else { 0.55 };
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
        self.director.sighted.push(kind.clone());
        self.player.sanity = clamp(self.player.sanity - 6.0 - 6.0 * clamp(1.0 - d / 20.0, 0.0, 1.0), 0.0, SAN_MAX);
        self.dread_sting(0.7);
        self.sfx_thump(0.9);
        // your asides: the first sight of a kind in a run, from where it does
        // not have you (never the Watcher: a word would say which sheet)
        if kind != "watcher" {
            self.aside(crate::audio::asides::Use::Sighted, &kind);
        }
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

    /// Creature `ei` has found you (the bench's pack, an old caller): it is
    /// sure where you are, and comes (`creature_alert`).
    pub fn entity_notice(&mut self, ei: usize, _d: f64) {
        let pp = self.player.pos;
        self.creature_alert(ei, pp.x, pp.z);
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
            if best.is_none_or(|(_, b)| d < b) {
                best = Some((i, d));
            }
        }
        best.map(|(i, _)| i)
    }

    /// After death, what goes on while the camera is a death's cutscene's
    /// or held behind its card: the killer keeps moving (jaw, hands), and
    /// the shake draws what the death camera always drew (three a frame,
    /// with a killer), so nothing after a death moves (CLAUDE.md invariant
    /// 1). → the shake (yaw, pitch, roll), also kept in `cut_shake`
    pub fn death_world(&mut self, dt: f64) -> (f64, f64, f64) {
        self.cut_shake = (0.0, 0.0, 0.0);
        let Some(ki) = self.director.killer.filter(|&i| i < self.entities.len()) else { return self.cut_shake };
        crate::entities::creature::kind_animate(self, ki, dt);
        if let Some(c) = self.creature(ki) {
            c.body.sync();
        }
        self.shake_t = (self.shake_t - dt).max(0.0);
        let shake = self.shake_t.max(0.25) * 0.05;
        // (the death camera's three draws, kept so nothing after a death
        // moves; the shake itself is a jitter of the cutscene's clock,
        // `curve::jitter`, the same at any frame rate: these are its sizes)
        let _ = (rand2(-1.0, 1.0), rand2(-1.0, 1.0), rand2(-1.0, 1.0));
        self.cut_shake = (shake, shake, shake * 0.6);
        self.cut_shake
    }

    /// After death, with no cutscene to play it: the camera is held on the
    /// killer's face, shaking, while it keeps moving (jaw, hands) until the
    /// screen cuts to black.
    pub fn update_death_cam(&mut self, dt: f64) {
        let Some(ki) = self.director.killer.filter(|&i| i < self.entities.len()) else { return };
        // (its kind's own animation: every killer is a creature of the redesign)
        crate::entities::creature::kind_animate(self, ki, dt);
        if let Some(c) = self.creature(ki) {
            c.body.sync();
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
    pub(crate) fn flicker_near(&mut self, x: f64, z: f64) {
        let mut near: Vec<(usize, f64)> = self.panels.iter().enumerate().map(|(i, p)| (i, hypot2(p.wx - x, p.wz - z))).collect();
        near.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        for &(i, dp) in near.iter().take(3) {
            if dp < 10.0 {
                self.panels[i].flicker = rand2(0.6, 1.1);
            }
        }
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
        // (it is told through its orders: `Order::Seek`)
        self.director.final_push = true;
        self.director.steer_t = 4.0;
        self.director.t = 25.0;
        true
    }
}
