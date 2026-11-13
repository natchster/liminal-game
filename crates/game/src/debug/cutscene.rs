//! Cutscenes under test (`docs/design/CUTSCENES.md`, the test engine). A *stage*
//! sets a game up the way a cutscene meets it in play and starts it the
//! way play does (`cutscene_stage`, `Staged::trigger`); a *run* records it
//! frame by frame (`run_cutscene`: what was shown, what play drew, the
//! bars, the veil, your body, the random draws the engine made); the
//! *contract* is what every run must keep, whatever the cutscene
//! (`contract`, `contract_skip`, `contract_rates`). The contract suite
//! (`crates/backrooms/tests/cutscenes.rs`) runs every cutscene of the
//! library on each of its stages; `backrooms --cutscene ID` previews one
//! and prints its contract. Placers must be registered before a stage
//! builds a floor (the caller's: `backrooms`' registry).
use crate::base::game::Game;
use crate::cutscene::play::{Anchors, Frame, Pose};
use crate::cutscene::{Cutscene, Then};

/// Turns faster than this (°/s) are a jolt the cutscene must declare
pub const MAX_TURN: f64 = 150.0;
/// ... moves faster than this (m/s)
pub const MAX_SPEED: f64 = 4.0;
/// Roll beyond this (°) only in a death's fall
pub const MAX_ROLL: f64 = 5.0;
/// The field of view at most this far (°) from play's
pub const MAX_FOV: f64 = 15.0;

/// How near a wall the camera may come (metres) while it is a cutscene's
pub const CAMERA_CLEARANCE: f64 = 0.06;

/// How to run a cutscene under test
#[derive(Clone, Copy, Debug)]
pub struct RunOpts {
    pub dt: f64,
    /// the skip key held from this time of its clock
    pub skip_at: Option<f64>,
    /// a second's pause at this time
    pub pause_at: Option<f64>,
    /// the nearest creature set hunting you at this time
    pub danger_at: Option<f64>,
    /// its short form, as on a repeat
    pub repeat: bool,
    /// Settings' camera shake off and head bob 0
    pub still: bool,
}

impl Default for RunOpts {
    fn default() -> Self {
        RunOpts { dt: 1.0 / 60.0, skip_at: None, pause_at: None, danger_at: None, repeat: false, still: false }
    }
}

/// Your body: where you stand, your facing and pitch, your crouch
pub type Body = ([f64; 3], f64, f64, bool);

/// One frame of a run
#[derive(Clone, Debug)]
pub struct Row {
    /// the cutscene's clock (after this frame)
    pub t: f64,
    pub shown: Pose,
    /// the gameplay camera this frame
    pub play: Pose,
    pub w: f64,
    pub bars: f64,
    pub veil: f64,
    pub hands: bool,
    pub body: Body,
    /// native draws the engine made this frame (its cues' sounds and events not counted)
    pub engine_draws: u64,
    /// the camera this close to a wall (`CAMERA_CLEARANCE`) while a cutscene's
    pub inside: bool,
    /// a layer of the HUD painted while your hands are taken (read every
    /// `HUD_EVERY` frames)
    pub hud_up: bool,
}

/// A run, start to hands back (or the card up)
#[derive(Clone, Debug, Default)]
pub struct Trace {
    pub id: String,
    pub stage: String,
    pub dt: f64,
    pub rows: Vec<Row>,
    /// your body when it began
    pub body_before: Option<Body>,
    /// the camera the frame before it was started
    pub before_cam: Option<Pose>,
    /// your nerve after it (a skip must leave it as watching does, within
    /// what time takes or gives)
    pub nerve_after: f64,
    /// the camera the frame after the hands came back (play's own)
    pub after: Option<Pose>,
    pub state_after: String,
    pub card_up: bool,
    /// what the stage reads of the world afterwards (a lift peered into, …)
    pub signature: String,
    /// the cutscenes' log lines of this run
    pub log: Vec<String>,
}

/// A stage set: how its cutscene is started
#[derive(Clone, Debug)]
pub struct Staged {
    pub stage: String,
    pub anchors: Anchors,
}

/// Seconds of play before a cutscene starts (the camera settled, the card gone)
const SETTLE: f64 = 1.0;

impl Game {
    fn stage_frame(&mut self, dt: f64) {
        for e in self.entities.iter_mut() {
            e.freeze = 1e9;
        }
        self.now_ms += dt * 1000.0;
        self.fixed_step_dt = dt;
        self.frame(dt);
        self.fixed_step_dt = 0.0;
    }

    /// A new run on `level` of run `seed`, settled, the creatures held
    fn stage_floor(&mut self, seed: u32, level: i32) {
        crate::base::rng::seed_native((u64::from(seed) << 8) | level as u64);
        self.launch_flags = format!("seed={seed}&level={level}&nosound");
        self.start_new_game();
        self.god_mode = true;
        self.pointer_locked = true;
        for _ in 0..(SETTLE * 60.0) as usize {
            self.stage_frame(1.0 / 60.0);
        }
    }

    /// Set up `stage` (on a booted game whose placers are registered):
    /// `ahead` a floor of the rooms with a thing two metres ahead of you
    /// (cutscenes under construction, and the known answers); `title` the
    /// title over the Well, live for a second (started by New descent);
    /// `title:SEED` the same over run SEED's Well
    pub fn cutscene_stage(&mut self, stage: &str) -> Result<Staged, String> {
        if let Some(seed) = stage.strip_prefix("title:") {
            let seed: u32 = seed.parse().map_err(|_| format!("stage {stage}: a run seed"))?;
            if self.state != "menu" {
                return Err("the title stage needs a game booted to its title".into());
            }
            self.run_seed = seed;
            self.init_level(1);
            self.title_place = Some((seed, 1));
            self.menu.view_t = 0.0;
        }
        match stage.split(':').next().unwrap_or("") {
            // `death:KIND` (a creature of that kind in your face, run 4242's
            // level 7), `death:sanity` (level 6), `death:fog` (the Well)
            "death" => {
                let what = stage.split(':').nth(1).unwrap_or("");
                let (seed, level) = match what {
                    "fog" => (4242, 1),
                    "sanity" => (4242, 6),
                    _ => (4242, 7),
                };
                self.stage_floor(seed, level);
                self.god_mode = false;
                // `death:KIND:wall`: backed against a wall (you die fleeing)
                if stage.ends_with(":wall") {
                    self.stage_back_to_wall();
                }
                if !matches!(what, "fog" | "sanity") {
                    self.showcase_entity(what, 1.0);
                    if !self.entities.iter().any(|e| e.kind == what) {
                        return Err(format!("stage {stage}: no {what}"));
                    }
                }
                Ok(Staged { stage: stage.into(), anchors: Anchors::default() })
            }
            // `broken_elevator:SEED:LEVEL[:side|:away]`: before a broken lift,
            // where its prompt is, facing it (or along its wall, or away: E has no
            // facing cone, so a look can start from any facing)
            "broken_elevator" => {
                let mut it = stage.split(':').skip(1).map(|x| x.parse::<i64>());
                let (Some(Ok(seed)), Some(Ok(level))) = (it.next(), it.next()) else {
                    return Err(format!("stage {stage}: broken_elevator:SEED:LEVEL"));
                };
                self.stage_floor(seed as u32, level as i32);
                let Some(f) = self.shaft_frame() else {
                    return Err(format!("stage {stage}: no broken lift on that floor"));
                };
                // a metre out from its doors, facing them
                let (w, n) = (f.o, [f.yaw.sin(), f.yaw.cos()]);
                // (your place is kept at eye height, as play keeps it)
                self.player.pos = engine::math::Vec3::new(w[0] + n[0], self.floor_y_at(w[0], w[2]) + crate::prelude::EYE, w[2] + n[1]);
                self.player.yaw = f.yaw
                    + match stage.rsplit(':').next() {
                        Some("side") => std::f64::consts::FRAC_PI_2,
                        Some("away") => std::f64::consts::PI,
                        _ => 0.0,
                    };
                self.player.pitch = 0.0;
                for _ in 0..30 {
                    self.stage_frame(1.0 / 60.0);
                }
                Ok(Staged { stage: stage.into(), anchors: Anchors::default() })
            }
            "title" => {
                if self.state != "menu" {
                    return Err("the title stage needs a game booted to its title".into());
                }
                for _ in 0..(SETTLE * 60.0) as usize {
                    self.stage_frame(1.0 / 60.0);
                }
                Ok(Staged { stage: stage.into(), anchors: Anchors::default() })
            }
            "ahead" => {
                self.stage_floor(4242, 6);
                let p = self.player.pos;
                let (s, c) = self.player.yaw.sin_cos();
                let at = [p.x - s * 2.0, self.floor_y_at(p.x, p.z), p.z - c * 2.0];
                let thing = Frame::thing(at, self.player.yaw);
                Ok(Staged { stage: stage.into(), anchors: Anchors { thing: Some(thing), killer: None, seen_as: None } })
            }
            _ => Err(format!("no stage \"{stage}\" (debug::cutscene::cutscene_stage)")),
        }
    }

    /// Start `c` on its stage as play would
    pub fn cutscene_trigger(&mut self, s: &Staged, c: &Cutscene) -> bool {
        let started = self.cutscene_start(s, c);
        // (the one asked for, or it did not start: a stage whose path plays
        // another, say a death of another cause, must not record it as this)
        let ours = self.cut.as_ref().is_some_and(|p| p.c.id == c.id) && self.cut_override.is_none();
        self.cut_override = None;
        started && ours
    }

    fn cutscene_start(&mut self, s: &Staged, c: &Cutscene) -> bool {
        match s.stage.split(':').next().unwrap_or("") {
            // as the title's list does (`menu:new`)
            "title" => {
                self.new_descent_with(Some(c.clone()));
                self.hands_taken()
            }
            // as death comes: what killed you, and on_death
            "death" => {
                let what = s.stage.split(':').nth(1).unwrap_or("");
                self.cut_override = Some(c.clone());
                let cause = match what {
                    "fog" => "assembly",
                    "sanity" => "sanity",
                    kind => {
                        self.director.killer = self.entities.iter().position(|e| e.kind == kind);
                        "entity"
                    }
                };
                self.on_death(cause);
                self.hands_taken()
            }
            // as your hand does: E on the lift
            "broken_elevator" => {
                self.cut_override = Some(c.clone());
                self.try_interact();
                self.hands_taken()
            }
            _ => self.play_cutscene_data(c.clone(), s.anchors.clone()),
        }
    }

    /// What the stage reads of the world after a run (a skip must leave the same)
    pub fn cutscene_signature(&self, s: &Staged) -> String {
        let p = &self.player;
        let r = |x: f64| (x * 1000.0).round() / 1000.0;
        // (what is the same whenever it is read: not the nerve, which moves with time)
        let base = format!("{} {} {} {} {}", self.state, r(p.pos.x), r(p.pos.z), r(p.yaw), p.crouch);
        match s.stage.split(':').next().unwrap_or("") {
            "title" => format!("{base} run {} level {} card {}", self.run_seed, self.level_num, self.level_card.is_some()),
            "broken_elevator" => format!("{base} peered {}", self.broken_elevator.as_ref().is_some_and(|b| b.peered)),
            _ => base,
        }
    }

    /// Record `c` on its stage as `o` asks (its step; the skip key held, a
    /// second's pause, something set hunting you, each from a time of its
    /// clock); `on_frame` sees each frame after it is played (a preview
    /// renders it)
    pub fn run_cutscene(&mut self, c: &Cutscene, s: &Staged, o: &RunOpts, on_frame: &mut dyn FnMut(&mut Game, &Row)) -> Trace {
        let dt = o.dt;
        let log_from = self.cut_log.len();
        let mut tr = Trace { id: c.id.clone(), stage: s.stage.clone(), dt, ..Default::default() };
        tr.before_cam = Some(self.camera_pose());
        if o.still {
            self.settings.shake = false;
            self.settings.bob = 0.0;
        }
        self.cut_force_repeat = o.repeat;
        if !self.cutscene_trigger(s, c) {
            tr.log = self.cut_log[log_from.min(self.cut_log.len())..].to_vec();
            return tr;
        }
        // (as it starts: a trigger may begin a run, which places you)
        let p = &self.player;
        tr.body_before = Some(([p.pos.x, p.pos.y, p.pos.z], p.yaw, p.pitch, p.crouch));
        let (mut skipping, mut paused, mut endangered) = (false, false, false);
        let limit = ((c.length + 5.0) / dt) as usize;
        for _ in 0..limit {
            let t = self.cut.as_ref().map(|p| p.t).unwrap_or(0.0);
            if let (Some(at), false) = (o.skip_at, skipping) {
                if t >= at {
                    self.key_down("KeyE", false);
                    skipping = true;
                }
            }
            if let (Some(at), false) = (o.pause_at, paused) {
                if t >= at {
                    self.key_down("Escape", false);
                    self.key_up("Escape");
                    for _ in 0..(1.0 / dt) as usize {
                        self.stage_frame(dt);
                    }
                    self.key_down("Escape", false);
                    self.key_up("Escape");
                    paused = true;
                }
            }
            if let (Some(at), false) = (o.danger_at, endangered) {
                if t >= at {
                    self.stage_hunt();
                    endangered = true;
                }
            }
            self.stage_frame(dt);
            let p = &self.player;
            let body = ([p.pos.x, p.pos.y, p.pos.z], p.yaw, p.pitch, p.crouch);
            let shown = self.camera_pose();
            // (the frame it ended in: what play drew, and the bars, kept)
            let (t, play, w, bars) = match (&self.cut, self.cut_end) {
                (Some(q), _) => (q.t, q.gameplay.unwrap_or(shown), q.w, q.bars()),
                (None, Some(e)) => (e.t, e.play, 0.0, e.bars),
                (None, None) => (tr.rows.last().map(|r| r.t).unwrap_or(0.0) + dt, shown, 0.0, 0.0),
            };
            let inside = self.hands_taken() && self.circle_hits_wall(shown.pos[0], shown.pos[2], CAMERA_CLEARANCE);
            let hud_up = self.hands_taken() && tr.rows.len() % HUD_EVERY == 0 && {
                let _ = self.hud_layers(1280.0, 800.0, 1.0);
                self.hud_ui.painted.iter().any(|n| !NOT_HUD.contains(n) && !n.starts_with("cut-") && !n.starts_with("menu"))
            };
            tr.rows.push(Row {
                t,
                shown,
                play,
                w,
                bars,
                veil: self.cut_veil.opacity,
                hands: self.hands_taken(),
                body,
                engine_draws: self.cut_engine_draws,
                inside,
                hud_up,
            });
            let row = tr.rows.last().unwrap().clone();
            on_frame(self, &row);
            if !self.hands_taken() {
                break;
            }
        }
        if skipping {
            self.key_up("KeyE");
        }
        // the frame after: play's own camera, and the card if it ends on one
        self.stage_frame(dt);
        tr.after = Some(self.camera_pose());
        for _ in 0..(2.0 / dt) as usize {
            if self.live_menu().is_some() || self.state != "dead" {
                break;
            }
            self.stage_frame(dt);
        }
        tr.card_up = self.live_menu().is_some();
        // (read when watching it to its end would be a second over: a
        // skipped run and a watched one at the same moment, so what the
        // place takes or gives over time is the same in both)
        let mut elapsed = tr.rows.len() as f64 * dt;
        while elapsed < c.length + 1.0 {
            self.stage_frame(dt);
            elapsed += dt;
        }
        tr.state_after = self.state.clone();
        tr.nerve_after = self.player.sanity;
        tr.signature = self.cutscene_signature(s);
        tr.log = self.cut_log[log_from.min(self.cut_log.len())..].to_vec();
        tr
    }

    /// You, backed up against the nearest wall behind you (0.3 m from it),
    /// facing out
    fn stage_back_to_wall(&mut self) {
        let p = self.player.pos;
        let (s, c) = self.player.yaw.sin_cos();
        let mut d = 0.0;
        while d < 12.0 && !self.circle_hits_wall(p.x + s * (d + 0.1), p.z + c * (d + 0.1), 0.3) {
            d += 0.1;
        }
        self.player.pos.x = p.x + s * d;
        self.player.pos.z = p.z + c * d;
        for _ in 0..10 {
            self.stage_frame(1.0 / 60.0);
        }
    }

    /// The nearest creature set hunting you (held where it is, it keeps its
    /// mode): danger, for a cutscene that must break off at it
    pub fn stage_hunt(&mut self) {
        let pp = self.player.pos;
        let d = |g: &Game, i: usize| {
            let q = g.entities[i].pos();
            (q.x - pp.x).hypot(q.z - pp.z)
        };
        let near = (0..self.entities.len()).filter(|&i| self.creature(i).is_some()).min_by(|&a, &b| d(self, a).total_cmp(&d(self, b)));
        if let Some(i) = near {
            self.entities[i].state = "hunt".into();
        }
    }
}

/// The HUD read every this many frames of a run
pub const HUD_EVERY: usize = 15;
/// Layers that are the place's, not the HUD's
const NOT_HUD: &[&str] = &["vignette", "radial", "fill", "grain"];
/// A skip's nerve within this of watching's (what time takes or gives
/// over the seconds a skip saves)
pub const SKIP_NERVE: f64 = 4.0;

/// A frame's move at a start or an end that reads as a jump (metres, radians)
pub const JUMP: f64 = 0.05;
pub const JUMP_TURN: f64 = 0.05;

/// Whether time `t` of `c` lies in a segment into a key marked as a jolt
fn in_jolt(c: &Cutscene, t: f64) -> bool {
    c.camera.windows(2).any(|w| w[1].jolt && t > w[0].t && t <= w[1].t + 1e-9)
}

/// What a run broke of the contract (`docs/design/CUTSCENES.md`, the design)
pub fn contract(c: &Cutscene, tr: &Trace) -> Vec<String> {
    let mut p = vec![];
    let at = |r: &Row| format!("{} on {} at {:.2} s", tr.id, tr.stage, r.t);
    if tr.rows.is_empty() {
        p.push(format!("{} on {}: it did not start ({:?})", tr.id, tr.stage, tr.log));
        return p;
    }
    let ended_early = tr.log.iter().any(|l| l.starts_with("skipped") || l.starts_with("broken off"));
    // 2: your hands are taken, and you are told
    let last = tr.rows.last().unwrap();
    for r in &tr.rows[..tr.rows.len() - 1] {
        if !r.hands {
            p.push(format!("{}: your hands came back before its end", at(r)));
            break;
        }
    }
    let full_from = c.bars.in_at + crate::cutscene::play::BARS_IN;
    if c.bars.out_at > full_from && !ended_early && !tr.rows.iter().any(|r| r.bars > 0.999) {
        p.push(format!("{} on {}: the bars never came all the way in", tr.id, tr.stage));
    }
    // 3: given back in place
    let before = tr.body_before.unwrap();
    if let Some(r) = tr.rows.iter().find(|r| r.body != before) {
        p.push(format!("{}: your body moved ({:?} → {:?})", at(r), before, r.body));
    }
    match c.then {
        Then::Back => {
            if last.hands {
                p.push(format!("{}: it never gave your hands back", at(last)));
            }
            if last.shown != last.play {
                p.push(format!("{}: its last frame is not play's camera ({:?} vs {:?})", at(last), last.shown, last.play));
            }
            if last.bars > 0.0 || last.veil > 0.0 && !tr.log.iter().any(|l| l.contains("(dip)")) {
                p.push(format!("{}: the bars or the veil are still up as your hands come back", at(last)));
            }
        }
        Then::Card => {
            if !tr.card_up {
                p.push(format!("{} on {}: it ends on the card, and no card came", tr.id, tr.stage));
            }
        }
    }
    for name in &c.finally {
        let n = tr.log.iter().filter(|l| l.starts_with(&format!("event {name} at "))).count();
        let in_track = c.event.iter().filter(|e| &e.name == name).count();
        if n < 1 || n > 1 + in_track {
            p.push(format!("{} on {}: its finally {name} ran {n} times", tr.id, tr.stage));
        }
    }
    // 1: from where you were looking, and on into play's camera after
    if let (Some(b), Some(f)) = (tr.before_cam, tr.rows.first()) {
        let (d, turn) = Pose::apart(&b, &f.shown);
        // (unless it starts with a declared jolt: a death's eyes snapping to its face)
        if (d > JUMP || turn > JUMP_TURN) && !in_jolt(c, f.t) {
            p.push(format!("{}: its first frame jumps {d:.3} m and {:.1}° from the camera before it", at(f), turn.to_degrees()));
        }
    }
    if let (Then::Back, Some(a)) = (c.then, tr.after) {
        let (d, turn) = Pose::apart(&last.shown, &a);
        if d > JUMP || turn > JUMP_TURN {
            p.push(format!("{}: the frame after it ends jumps {d:.3} m and {:.1}°", at(last), turn.to_degrees()));
        }
    }
    if !ended_early && (last.t - c.length).abs() > tr.dt + 1e-9 {
        p.push(format!("{}: played through, it ended at {:.3} s, not its length {:.3}", at(last), last.t, c.length));
    }
    if let Some(r) = tr.rows.iter().find(|r| r.hud_up) {
        p.push(format!("{}: the HUD is up while your hands are taken", at(r)));
    }
    if let Some(r) = tr.rows.iter().find(|r| r.inside) {
        p.push(format!("{}: the camera is within {CAMERA_CLEARANCE} m of a wall", at(r)));
    }
    // 5: time, not frames; the engine draws nothing
    if let Some(r) = tr.rows.iter().find(|r| r.engine_draws > 0) {
        p.push(format!("{}: the engine drew {} random numbers", at(r), r.engine_draws));
    }
    // 7: comfortable
    // (a key marked `cut` jumps on purpose: the frame it lands in)
    let cut_in = |a: f64, b: f64| c.camera.iter().any(|k| k.cut && k.t > a - 1e-9 && k.t <= b + 1e-9);
    for w in tr.rows.windows(2) {
        let (a, b) = (&w[0], &w[1]);
        let dt = tr.dt;
        // (a jolt declared, a cut a key asks for, and a cut made under the
        // veil, which nobody sees; the way back after a skip or a break-off
        // keeps the limits: near, it blends slowly enough, far, it dips)
        if in_jolt(c, b.t) || cut_in(a.t, b.t) || a.veil > 0.99 || b.veil > 0.99 {
            continue;
        }
        let turn = crate::cutscene::curve::wrap(b.shown.yaw - a.shown.yaw).abs().max((b.shown.pitch - a.shown.pitch).abs()) / dt;
        if turn.to_degrees() > MAX_TURN {
            p.push(format!(
                "{}: it turns at {:.0}°/s (more than {MAX_TURN}° without a jolt): yaw {:.3} → {:.3}, pitch {:.3} → {:.3}, w {:.3} → {:.3}",
                at(b),
                turn.to_degrees(),
                a.shown.yaw,
                b.shown.yaw,
                a.shown.pitch,
                b.shown.pitch,
                a.w,
                b.w
            ));
            break;
        }
        let speed = Pose::apart(&a.shown, &b.shown).0 / dt;
        if speed > MAX_SPEED {
            p.push(format!("{}: it moves at {speed:.1} m/s (more than {MAX_SPEED} without a jolt)", at(b)));
            break;
        }
    }
    for r in &tr.rows {
        if c.then != Then::Card && r.shown.roll.to_degrees().abs() > MAX_ROLL + (r.play.roll.to_degrees().abs()) {
            p.push(format!("{}: it rolls {:.1}° (a roll is a fall's)", at(r), r.shown.roll.to_degrees()));
            break;
        }
        if (r.shown.fov - r.play.fov).abs() > MAX_FOV {
            p.push(format!("{}: its field of view is {:.1}° from play's", at(r), (r.shown.fov - r.play.fov).abs()));
            break;
        }
    }
    p
}

/// A skip lands where watching to the end does
pub fn contract_skip(played: &Trace, skipped: &Trace) -> Vec<String> {
    let mut p = vec![];
    if !skipped.log.iter().any(|l| l.starts_with("skipped")) {
        p.push(format!("{} on {}: holding the skip key did not skip it", skipped.id, skipped.stage));
    }
    if (played.nerve_after - skipped.nerve_after).abs() > SKIP_NERVE {
        p.push(format!(
            "{} on {}: a skip leaves your nerve at {:.1}, watching at {:.1}",
            skipped.id, skipped.stage, skipped.nerve_after, played.nerve_after
        ));
    }
    if played.signature != skipped.signature {
        p.push(format!(
            "{} on {}: a skip leaves \"{}\", watching leaves \"{}\"",
            skipped.id, skipped.stage, skipped.signature, played.signature
        ));
    }
    if played.card_up != skipped.card_up {
        p.push(format!("{} on {}: the card after a skip differs", skipped.id, skipped.stage));
    }
    p
}

/// The same cutscene at frame rates whose steps all land on the times of
/// the slowest (30, 60 and 120 a second land every 1/30 s): the same cues,
/// the same length, at each time they share the same blend, and where its
/// own camera is all there is the same camera (within `tol` metres and
/// radians; a live killer moves with its own animation). Play's camera is
/// play's: its smoothing goes by the frame
pub fn contract_rates(traces: &[Trace], tol: f64) -> Vec<String> {
    let mut p = vec![];
    let cues = |tr: &Trace| -> Vec<String> {
        tr.log.iter().filter(|l| l.starts_with("event ") || l.starts_with("sound ") || l.starts_with("say ")).cloned().collect()
    };
    let first = &traces[0];
    let fps = |tr: &Trace| (1.0 / tr.dt).round();
    for tr in &traces[1..] {
        if cues(tr) != cues(first) {
            p.push(format!(
                "{} on {}: its cues at {} fps {:?} differ from {} fps {:?}",
                tr.id,
                tr.stage,
                fps(tr),
                cues(tr),
                fps(first),
                cues(first)
            ));
        }
        let (a, b) = (first.rows.last().map(|r| r.t).unwrap_or(0.0), tr.rows.last().map(|r| r.t).unwrap_or(0.0));
        if (a - b).abs() > first.dt.max(tr.dt) + 1e-9 {
            p.push(format!("{} on {}: it ends at {b:.3} s at {} fps and {a:.3} s at {} fps", tr.id, tr.stage, fps(tr), fps(first)));
        }
        let mut worst = (0.0f64, 0.0);
        for r in &first.rows {
            if let Some(q) = tr.rows.iter().find(|q| (q.t - r.t).abs() < 1e-6) {
                if (r.w - q.w).abs() > 1e-9 {
                    p.push(format!(
                        "{} on {}: at {:.3} s its blend is {:.4} at {} fps and {:.4} at {} fps",
                        tr.id,
                        tr.stage,
                        r.t,
                        q.w,
                        fps(tr),
                        r.w,
                        fps(first)
                    ));
                }
                if r.w < 0.999 || q.w < 0.999 {
                    continue;
                }
                let (d, turn) = Pose::apart(&r.shown, &q.shown);
                if d.max(turn) > worst.0 {
                    worst = (d.max(turn), r.t);
                }
            }
        }
        if worst.0 > tol {
            p.push(format!(
                "{} on {}: at {:.3} s the camera at {} fps is {:.4} from the one at {} fps",
                tr.id,
                tr.stage,
                worst.1,
                fps(tr),
                worst.0,
                fps(first)
            ));
        }
    }
    p
}
