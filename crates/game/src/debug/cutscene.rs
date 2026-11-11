//! Cutscenes under test (`docs/CUTSCENES.md`, the test engine). A *stage*
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
use crate::cutscene::play::{Anchors, Frame, Pose, BREAK_BLEND, SKIP_BLEND};
use crate::cutscene::{Cutscene, Then};

/// Turns faster than this (°/s) are a jolt the cutscene must declare
pub const MAX_TURN: f64 = 150.0;
/// ... moves faster than this (m/s)
pub const MAX_SPEED: f64 = 4.0;
/// Roll beyond this (°) only in a death's fall
pub const MAX_ROLL: f64 = 5.0;
/// The field of view at most this far (°) from play's
pub const MAX_FOV: f64 = 15.0;

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
    /// (cutscenes under construction, and the known answers)
    pub fn cutscene_stage(&mut self, stage: &str) -> Result<Staged, String> {
        match stage {
            "ahead" => {
                self.stage_floor(4242, 6);
                let p = self.player.pos;
                let (s, c) = self.player.yaw.sin_cos();
                let at = [p.x - s * 2.0, self.floor_y_at(p.x, p.z), p.z - c * 2.0];
                let thing = Frame::thing(at, self.player.yaw);
                Ok(Staged { stage: stage.into(), anchors: Anchors { thing: Some(thing), killer: None } })
            }
            _ => Err(format!("no stage \"{stage}\" (debug::cutscene::cutscene_stage)")),
        }
    }

    /// Start `c` on its stage as play would
    pub fn cutscene_trigger(&mut self, s: &Staged, c: &Cutscene) -> bool {
        self.play_cutscene_data(c.clone(), s.anchors.clone())
    }

    /// What the stage reads of the world after a run (a skip must leave the same)
    pub fn cutscene_signature(&self, s: &Staged) -> String {
        let p = &self.player;
        let r = |x: f64| (x * 1000.0).round() / 1000.0;
        let _ = s;
        format!("{} {} {} {} {} {}", self.state, r(p.pos.x), r(p.pos.z), r(p.yaw), p.crouch, r(p.sanity))
    }

    /// Record `c` on its stage at a step of `dt`, holding the skip key from
    /// `skip_at` seconds and pausing for a second at `pause_at` if given;
    /// `on_frame` sees each frame after it is played (a preview renders it)
    pub fn run_cutscene(
        &mut self,
        c: &Cutscene,
        s: &Staged,
        dt: f64,
        skip_at: Option<f64>,
        pause_at: Option<f64>,
        on_frame: &mut dyn FnMut(&mut Game, &Row),
    ) -> Trace {
        let log_from = self.cut_log.len();
        let mut tr = Trace { id: c.id.clone(), stage: s.stage.clone(), dt, ..Default::default() };
        let p = &self.player;
        tr.body_before = Some(([p.pos.x, p.pos.y, p.pos.z], p.yaw, p.pitch, p.crouch));
        if !self.cutscene_trigger(s, c) {
            tr.log = self.cut_log[log_from.min(self.cut_log.len())..].to_vec();
            return tr;
        }
        let (mut skipping, mut paused) = (false, false);
        let limit = ((c.length + 5.0) / dt) as usize;
        for _ in 0..limit {
            let t = self.cut.as_ref().map(|p| p.t).unwrap_or(0.0);
            if let (Some(at), false) = (skip_at, skipping) {
                if t >= at {
                    self.key_down("KeyE", false);
                    skipping = true;
                }
            }
            if let (Some(at), false) = (pause_at, paused) {
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
            self.stage_frame(dt);
            let p = &self.player;
            let body = ([p.pos.x, p.pos.y, p.pos.z], p.yaw, p.pitch, p.crouch);
            let shown = self.camera_pose();
            // (the frame it ended in: its clock went on by a step)
            let last_t = tr.rows.last().map(|r| r.t).unwrap_or(0.0);
            let (t, play, w, bars) = match &self.cut {
                Some(q) => (q.t, q.gameplay.unwrap_or(shown), q.w, q.bars()),
                None => (last_t + dt, shown, 0.0, 0.0),
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
        tr.state_after = self.state.clone();
        tr.signature = self.cutscene_signature(s);
        tr.log = self.cut_log[log_from.min(self.cut_log.len())..].to_vec();
        tr
    }
}

/// Whether time `t` of `c` lies in a segment into a key marked as a jolt
fn in_jolt(c: &Cutscene, t: f64) -> bool {
    c.camera.windows(2).any(|w| w[1].jolt && t > w[0].t && t <= w[1].t + 1e-9)
}

/// What a run broke of the contract (`docs/CUTSCENES.md`, the design)
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
    // 5: time, not frames; the engine draws nothing
    if let Some(r) = tr.rows.iter().find(|r| r.engine_draws > 0) {
        p.push(format!("{}: the engine drew {} random numbers", at(r), r.engine_draws));
    }
    // 7: comfortable
    let after_break = |t: f64| {
        tr.log
            .iter()
            .filter_map(|l| l.strip_prefix("skipped at ").or_else(|| l.strip_prefix("broken off at ")))
            .filter_map(|s| s.split(' ').next()?.parse::<f64>().ok())
            .any(|s| t >= s - 1e-9 && t <= s + SKIP_BLEND.max(BREAK_BLEND) + 0.3)
    };
    for w in tr.rows.windows(2) {
        let (a, b) = (&w[0], &w[1]);
        let dt = tr.dt;
        // (a jolt declared, the quick way back after a skip or a break-off,
        // and a cut made under the veil, which nobody sees)
        if in_jolt(c, b.t) || after_break(b.t) || a.veil > 0.99 || b.veil > 0.99 {
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
/// the same length, and at each time they share the same camera (within
/// `tol` metres and radians; a live killer moves with its own animation)
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
