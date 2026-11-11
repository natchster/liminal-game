//! The player (`docs/CUTSCENES.md`, the design): one cutscene at a time,
//! on a clock of its own (time played: it stops with the pause), its camera
//! a function of that time and its anchors, mixed over the gameplay camera
//! by its blend envelope, so it begins where you were looking and its last
//! frame is the gameplay camera exactly.
//!
//! Each frame, around play's own update (`session::frame`):
//! `cutscene_pre` puts back the gameplay camera of the frame before (the
//! player's update eases its roll and field of view from the camera as it
//! was, and must never ease from a cutscene's pose), play draws its camera,
//! then `cutscene_post` reads that camera, advances the clock, runs the
//! cues that came due (each once, even across a long frame) and writes the
//! mix. While one plays your hands are taken (`hands_taken`): keys and the
//! mouse do nothing to your body, nothing may strike you, your asides wait.
//! Nothing here draws from a random stream.
use super::cues;
use super::curve::{catmull, lerp, lerp_angle, look_angles, smooth, sway};
use super::model::*;
use crate::base::game::Game;
use std::f64::consts::PI;

/// Seconds the skip key is held to skip
pub const SKIP_HOLD: f64 = 0.6;
/// A skip near your eyes blends back over this
pub const SKIP_BLEND: f64 = 0.3;
/// ... near: within this many metres and this turn (radians); further, it
/// dips through black instead of sweeping the camera across the place
pub const SKIP_NEAR: f64 = 1.0;
pub const SKIP_NEAR_TURN: f64 = 0.61;
/// The dip through black on a far skip: closing, then opening on your eyes
pub const DIP_OUT: f64 = 0.2;
pub const DIP_IN: f64 = 0.25;
/// A cutscene broken off by danger blends back over this
pub const BREAK_BLEND: f64 = 0.35;
/// The bars come in over this, and go over that
pub const BARS_IN: f64 = 0.4;
pub const BARS_OUT: f64 = 0.5;
/// The skip line shows this long after a key
pub const HINT_FOR: f64 = 2.5;
/// A repeat that starts part-way blends in over at least this
pub const REPEAT_BLEND: f64 = 0.2;

/// A camera's pose: where it is, its Euler angles (YXZ: yaw 0 looks down
/// -z, pitch up positive) and its vertical field of view in degrees
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pose {
    pub pos: [f64; 3],
    pub yaw: f64,
    pub pitch: f64,
    pub roll: f64,
    pub fov: f64,
}

impl Pose {
    /// From `a` toward `b` by `w` (0: `a` exactly; 1: `b` exactly), the
    /// angles the short way round
    pub fn mix(a: &Pose, b: &Pose, w: f64) -> Pose {
        if w <= 0.0 {
            return *a;
        }
        if w >= 1.0 {
            return *b;
        }
        Pose {
            pos: [lerp(a.pos[0], b.pos[0], w), lerp(a.pos[1], b.pos[1], w), lerp(a.pos[2], b.pos[2], w)],
            yaw: lerp_angle(a.yaw, b.yaw, w),
            pitch: lerp(a.pitch, b.pitch, w),
            roll: lerp(a.roll, b.roll, w),
            fov: lerp(a.fov, b.fov, w),
        }
    }
    /// How far apart two poses are: metres, and the larger turn (radians)
    pub fn apart(a: &Pose, b: &Pose) -> (f64, f64) {
        let d = ((a.pos[0] - b.pos[0]).powi(2) + (a.pos[1] - b.pos[1]).powi(2) + (a.pos[2] - b.pos[2]).powi(2)).sqrt();
        let turn = super::curve::wrap(a.yaw - b.yaw).abs().max((a.pitch - b.pitch).abs()).max((a.roll - b.roll).abs());
        (d, turn)
    }
}

/// A frame keys are written in: its origin, its yaw (x right, y up, z out
/// of its face), and a pitch an `Ahead` look starts from
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Frame {
    pub o: [f64; 3],
    pub yaw: f64,
    pub pitch: f64,
}

impl Frame {
    /// A point of the frame in the world
    pub fn at(&self, p: [f64; 3]) -> [f64; 3] {
        let (s, c) = self.yaw.sin_cos();
        [self.o[0] + p[0] * c + p[2] * s, self.o[1] + p[1], self.o[2] - p[0] * s + p[2] * c]
    }
    /// A thing's frame: `pos` its foot, `yaw` its turn (its front, +z, faces
    /// `(sin yaw, cos yaw)`)
    pub fn thing(pos: [f64; 3], yaw: f64) -> Frame {
        Frame { o: pos, yaw, pitch: 0.0 }
    }
}

/// What a cutscene is started from, besides your eyes and the camera
#[derive(Clone, Debug, Default)]
pub struct Anchors {
    pub thing: Option<Frame>,
    /// the creature that killed you (its index)
    pub killer: Option<usize>,
}

/// The cutscene's own veil (its fades): kept on the game, so a death's
/// black stays behind its card and a far skip opens on your eyes
#[derive(Clone, Debug, PartialEq)]
pub struct Veil {
    pub opacity: f64,
    pub colour: String,
    /// after a far skip: opening at this rate (opacity a second)
    pub opening: f64,
}

impl Default for Veil {
    fn default() -> Self {
        Veil { opacity: 0.0, colour: "#000".into(), opening: 0.0 }
    }
}

/// The cutscene playing
#[derive(Clone, Debug)]
pub struct Playing {
    pub c: Cutscene,
    /// its clock
    pub t: f64,
    /// where its clock started (its short form on a repeat)
    pub t0: f64,
    pub start: Frame,
    pub eyes: Frame,
    pub thing: Option<Frame>,
    pub killer: Option<usize>,
    /// the gameplay camera, as play drew it this frame
    pub gameplay: Option<Pose>,
    /// what was drawn
    pub shown: Option<Pose>,
    /// the blend weight drawn (0 the gameplay camera, 1 the keys)
    pub w: f64,
    /// the next cue of each track to run: fade, sound, event, say
    next: [usize; 4],
    /// broken off or skipped near your eyes: back from here over this
    pub out: Option<(f64, f64)>,
    /// skipped from far: the veil closes from here, then the end
    pub dip: Option<f64>,
    pub skip_down: bool,
    pub skip_held: f64,
    /// the skip line shows for this much longer
    pub hint: f64,
    /// how it ended or is ending: "played", "skipped", "broken off"
    pub how: &'static str,
}

impl Playing {
    /// When it ends
    pub fn end(&self) -> f64 {
        match (self.out, self.dip) {
            (_, Some(d)) => d + DIP_OUT,
            (Some((s, l)), _) => (s + l).min(self.c.length),
            _ => self.c.length,
        }
    }
    /// The weight of the keys over the gameplay camera at its time
    pub fn weight(&self) -> f64 {
        let c = &self.c;
        let blend_in = if self.t0 > 0.0 { c.blend_in.max(REPEAT_BLEND) } else { c.blend_in };
        let w_in = if blend_in > 0.0 { smooth(self.t - self.t0, 0.0, blend_in) } else { 1.0 };
        let mut w_out = if c.then == Then::Back { 1.0 - smooth(self.t, c.length - c.blend_out, c.length) } else { 1.0 };
        if let Some((s, l)) = self.out {
            w_out = w_out.min(1.0 - smooth(self.t, s, s + l));
        }
        if self.dip.is_some() && self.t >= self.end() {
            w_out = 0.0;
        }
        w_in * w_out
    }
    /// How far the bars are in (0 out, 1 in)
    pub fn bars(&self) -> f64 {
        let b = self.c.bars;
        let at_in = smooth(self.t, b.in_at, b.in_at + BARS_IN);
        let mut out = smooth(self.t, b.out_at, b.out_at + BARS_OUT);
        if let Some((s, l)) = self.out {
            out = out.max(smooth(self.t, s, s + l.min(BARS_OUT)));
        }
        if let Some(d) = self.dip {
            out = out.max(smooth(self.t, d, d + DIP_OUT));
        }
        at_in * (1.0 - out)
    }
    /// The veil its fades make at its time (and a far skip's dip)
    pub fn veil(&self) -> (f64, String) {
        let (mut v, colour) = self.c.veil_at(self.t);
        if let Some(d) = self.dip {
            v = v.max(smooth(self.t, d, d + DIP_OUT));
        }
        (v, colour)
    }
    fn frame(&self, g: &Game, a: Anchor) -> Frame {
        match a {
            Anchor::Start => self.start,
            Anchor::Eyes => self.eyes,
            Anchor::Thing => self.thing.unwrap_or(self.eyes),
            Anchor::Killer => g.cutscene_killer_frame(self.killer).unwrap_or(self.eyes),
        }
    }
    /// A key in the world: its pose (the gameplay camera's field of view
    /// where it names none) and the live point it looks at, if any
    fn resolve(&self, g: &Game, k: &Key, fov_play: f64, aspect: f64) -> (Pose, Option<[f64; 3]>) {
        let f = self.frame(g, k.anchor);
        let pos = f.at(k.pos);
        let (yaw, pitch, target) = match k.look {
            Look::Ahead => (f.yaw, f.pitch, None),
            Look::Point(p) => {
                let (y, pt) = look_angles(pos, f.at(p));
                (y, pt, None)
            }
            Look::At(a) => {
                let o = self.frame(g, a).o;
                let (y, pt) = look_angles(pos, o);
                (y, pt, Some(o))
            }
        };
        let fov = k.fov.map(|h| crate::session::settings::vertical_fov(h, aspect)).unwrap_or(fov_play);
        let pose = Pose { pos, yaw: yaw + k.yaw.to_radians(), pitch: pitch + k.pitch.to_radians(), roll: k.roll.to_radians(), fov };
        (pose, target)
    }
    /// Its own camera at its time, before the blend (`shake_k`: Settings'
    /// camera shake, 1 or 0, on its sway)
    pub fn keyed(&self, g: &Game, fov_play: f64, aspect: f64, shake_k: f64) -> Pose {
        let keys = &self.c.camera;
        let t = self.t;
        let n = keys.len();
        let last = n - 1;
        let i = keys.iter().rposition(|k| k.t <= t).unwrap_or(0);
        if t <= keys[0].t || i >= last {
            let k = if t <= keys[0].t { &keys[0] } else { &keys[last] };
            let (mut p, _) = self.resolve(g, k, fov_play, aspect);
            let (sy, sp) = sway(t, k.sway * shake_k);
            p.yaw += sy;
            p.pitch += sp;
            return p;
        }
        let (a, b) = (&keys[i], &keys[i + 1]);
        let (pa, ta) = self.resolve(g, a, fov_play, aspect);
        if b.cut {
            // held until the cut
            let (sy, sp) = sway(t, a.sway * shake_k);
            return Pose { yaw: pa.yaw + sy, pitch: pa.pitch + sp, ..pa };
        }
        let (pb, tb) = self.resolve(g, b, fov_play, aspect);
        let u = (t - a.t) / (b.t - a.t);
        let e = b.ease.at(u);
        // the neighbours (a cut ends a run of keys: nothing bends across it)
        let p0 = if i > 0 && !a.cut { self.resolve(g, &keys[i - 1], fov_play, aspect).0.pos } else { pa.pos };
        let p3 = if i + 2 <= last && !keys[i + 2].cut { self.resolve(g, &keys[i + 2], fov_play, aspect).0.pos } else { pb.pos };
        let pos = catmull(p0, pa.pos, pb.pos, p3, e);
        let (mut yaw, mut pitch) = (lerp_angle(pa.yaw, pb.yaw, e), lerp(pa.pitch, pb.pitch, e));
        // both look at the same live point: follow it from where the camera is
        if let (Some(x), Some(y)) = (ta, tb) {
            if a.look == b.look {
                let target = super::curve::lerp3(x, y, e);
                let (ly, lp) = look_angles(pos, target);
                yaw = ly + lerp(a.yaw, b.yaw, e).to_radians();
                pitch = lp + lerp(a.pitch, b.pitch, e).to_radians();
            }
        }
        let (sy, sp) = sway(t, lerp(a.sway, b.sway, e) * shake_k);
        Pose {
            pos,
            yaw: yaw + sy,
            pitch: (pitch + sp).clamp(-PI / 2.0 + 0.01, PI / 2.0 - 0.01),
            roll: lerp(pa.roll, pb.roll, e),
            fov: lerp(pa.fov, pb.fov, e),
        }
    }
}

impl Cutscene {
    /// Its veil at time `t`: each fade eases from what the veil was when it
    /// began to its `to` over its `over`; the colour is the last begun's
    pub fn veil_at(&self, t: f64) -> (f64, String) {
        let mut v = 0.0;
        let mut colour = "#000".to_string();
        for (i, f) in self.fade.iter().enumerate() {
            if f.t > t {
                break;
            }
            // what the veil was when this fade began
            let until = self.fade.get(i + 1).map(|n| n.t.min(t)).unwrap_or(t);
            let k = if f.over > 0.0 { smooth(until, f.t, f.t + f.over) } else { 1.0 };
            v = lerp(v, f.to, k);
            colour = f.colour.clone();
        }
        (v, colour)
    }
}

impl Game {
    /// Your hands are taken: a cutscene has the camera
    pub fn hands_taken(&self) -> bool {
        self.cut.is_some()
    }

    /// Play the cutscene `id` from the library; false when it cannot be
    /// played now (unknown, one already playing, not in play, in the
    /// Stairwell or the Loop, or an anchor it names not given)
    pub fn play_cutscene(&mut self, id: &str, anchors: Anchors) -> bool {
        match super::get(id) {
            Some(c) => self.play_cutscene_data(c.clone(), anchors),
            None => {
                self.cut_note(format!("no cutscene {id}"));
                false
            }
        }
    }

    /// Play a cutscene given as data (tests make their own)
    pub fn play_cutscene_data(&mut self, c: Cutscene, anchors: Anchors) -> bool {
        let refuse = if self.cut.is_some() {
            Some("one is playing")
        } else if self.in_stairwell || self.in_loop {
            Some("the Stairwell and the Loop have their own camera")
        } else if self.state != "playing" && self.state != "dead" {
            Some("not in play")
        } else if anchors.thing.is_none()
            && (c.camera.iter().any(|k| k.anchor == Anchor::Thing || k.look == Look::At(Anchor::Thing))
                || c.sound.iter().any(|s| s.anchor == Anchor::Thing))
        {
            Some("it names a thing and none was given")
        } else {
            None
        };
        if let Some(why) = refuse {
            self.cut_note(format!("refused {}: {why}", c.id));
            return false;
        }
        let seen = self.cut_seen.entry(c.id.clone()).or_default();
        let t0 = if *seen > 0 { c.repeat_from } else { 0.0 };
        *seen += 1;
        let cam = self.camera.clone();
        let cp = cam.pos();
        let start = Frame { o: [cp.x, cp.y, cp.z], yaw: cam.rot().y, pitch: cam.rot().x };
        let pp = self.player.pos;
        let eyes =
            Frame { o: [pp.x, self.floor_y_at(pp.x, pp.z) + self.player.eye_h, pp.z], yaw: self.player.yaw, pitch: self.player.pitch };
        // empty hands: no early E, the notebook put away
        self.input.interact_buffer = 0.0;
        if self.journal_open {
            self.journal_open = false;
            self.ui.journal.remove_class("on");
        }
        self.cut_note(format!("play {}{}", c.id, if t0 > 0.0 { format!(" from {t0:.2} (a repeat)") } else { String::new() }));
        self.cut = Some(Playing {
            c,
            t: t0,
            t0,
            start,
            eyes,
            thing: anchors.thing,
            killer: anchors.killer,
            gameplay: None,
            shown: None,
            w: 0.0,
            next: [0; 4],
            out: None,
            dip: None,
            skip_down: false,
            skip_held: 0.0,
            hint: 0.0,
            how: "played",
        });
        // (cues at its first instant run with its first frame)
        if let Some(p) = self.cut.as_mut() {
            let t0 = p.t0;
            for (i, list) in [
                p.c.fade.iter().map(|x| x.t).collect::<Vec<_>>(),
                p.c.sound.iter().map(|x| x.t).collect(),
                p.c.event.iter().map(|x| x.t).collect(),
                p.c.say.iter().map(|x| x.t).collect(),
            ]
            .iter()
            .enumerate()
            {
                // a repeat starts part-way: what came before its start is not run
                p.next[i] = list.iter().take_while(|&&t| t < t0).count();
            }
        }
        true
    }

    /// A line in the cutscenes' log (kept short: the last 64)
    pub fn cut_note(&mut self, s: String) {
        if self.cut_log.len() >= 64 {
            self.cut_log.remove(0);
        }
        self.cut_log.push(s);
    }

    /// The killer's frame: its head, facing as it faces
    pub fn cutscene_killer_frame(&self, killer: Option<usize>) -> Option<Frame> {
        let ki = killer.filter(|&i| i < self.entities.len())?;
        let e = &self.entities[ki];
        let head = match e.parts.get("head") {
            Some(h) => h.world_position(),
            None => {
                let p = e.pos();
                engine::math::Vec3::new(p.x, e.face_y, p.z)
            }
        };
        Some(Frame { o: [head.x, head.y, head.z], yaw: e.yaw, pitch: 0.0 })
    }

    /// The camera as a pose
    pub fn camera_pose(&self) -> Pose {
        let cam = &self.camera;
        let p = cam.pos();
        let r = cam.rot();
        Pose { pos: [p.x, p.y, p.z], yaw: r.y, pitch: r.x, roll: r.z, fov: cam.camera().fov }
    }

    /// Put the camera at `p`
    pub fn set_camera_pose(&mut self, p: &Pose) {
        let cam = self.camera.clone();
        cam.position().set(p.pos[0], p.pos[1], p.pos[2]);
        cam.rotation().y = p.yaw;
        cam.rotation().x = p.pitch;
        cam.rotation().z = p.roll;
        cam.camera().fov = p.fov;
    }

    /// Before play's update: the gameplay camera of the frame before, put
    /// back, so play eases from its own camera and never from a cutscene's
    pub fn cutscene_pre(&mut self) {
        if let Some(g) = self.cut.as_ref().and_then(|p| p.gameplay) {
            self.set_camera_pose(&g);
        }
    }

    /// After play's update: the clock, the skip, a break-off, the cues, the
    /// camera, the veil; and the end. What the engine drew from the native
    /// stream is counted apart from what its cues drew (`cut_engine_draws`:
    /// it must be 0, rule 5)
    pub fn cutscene_post(&mut self, dt: f64) {
        let before = crate::base::rng::native_draws();
        self.cut_cue_draws = 0;
        self.cutscene_step(dt);
        self.cut_engine_draws = crate::base::rng::native_draws() - before - self.cut_cue_draws;
    }

    fn cutscene_step(&mut self, dt: f64) {
        // a far skip's veil opening on your eyes
        if self.cut.is_none() {
            if self.cut_veil.opening > 0.0 {
                self.cut_veil.opacity = (self.cut_veil.opacity - self.cut_veil.opening * dt).max(0.0);
                if self.cut_veil.opacity == 0.0 {
                    self.cut_veil.opening = 0.0;
                }
            }
            return;
        }
        let mut p = self.cut.take().unwrap();
        let gameplay = self.camera_pose();
        p.gameplay = Some(gameplay);
        p.t += dt;
        p.hint = (p.hint - dt).max(0.0);
        if p.skip_down && p.c.skip == Skip::Hold && p.out.is_none() && p.dip.is_none() {
            p.skip_held += dt;
            if p.skip_held >= SKIP_HOLD {
                self.cut_skip(&mut p);
            }
        }
        // something comes: back to your eyes, now
        if p.c.breaks_off && p.c.world == World::Live && p.out.is_none() && p.dip.is_none() && self.cutscene_danger() {
            p.out = Some((p.t, BREAK_BLEND));
            p.how = "broken off";
            self.cut_note(format!("broken off at {:.2}", p.t));
        }
        self.cut_cues(&mut p);
        let aspect = self.camera.camera().aspect;
        let shake_k = if self.settings.shake { 1.0 } else { 0.0 };
        let end = p.end();
        let done = p.t >= end - 1e-9;
        p.w = if done && p.c.then == Then::Back { 0.0 } else { p.weight() };
        let shown = if p.w <= 0.0 { gameplay } else { Pose::mix(&gameplay, &p.keyed(self, gameplay.fov, aspect, shake_k), p.w) };
        self.set_camera_pose(&shown);
        p.shown = Some(shown);
        let (v, colour) = p.veil();
        self.cut_veil = Veil { opacity: v, colour, opening: 0.0 };
        if done {
            if p.dip.is_some() {
                // opened on your eyes after the hands are back
                self.cut_veil.opacity = 1.0;
                self.cut_veil.opening = 1.0 / DIP_IN;
            }
            self.cut_note(format!("end {} ({}) at {:.2}", p.c.id, p.how, p.t));
            if p.c.then == Then::Card {
                self.cut_hold = Some(shown);
            }
            return;
        }
        self.cut = Some(p);
    }

    /// Every cue of `p` that came due, in order of time, each once
    fn cut_cues(&mut self, p: &mut Playing) {
        // (nothing after a break-off or a skip: only what came before it)
        let until = match (p.out, p.dip) {
            (Some((s, _)), _) | (_, Some(s)) => s.min(p.t),
            _ => p.t,
        };
        // (the veil is a function of time, `veil_at`: only counted here)
        while p.c.fade.get(p.next[0]).is_some_and(|f| f.t <= until) {
            p.next[0] += 1;
        }
        while let Some(s) = p.c.sound.get(p.next[1]).filter(|s| s.t <= until).cloned() {
            p.next[1] += 1;
            if let Some(play) = cues::sound(&s.name) {
                let at = p.frame(self, s.anchor).at(s.pos);
                let d = crate::base::rng::native_draws();
                self.play_from(at[0], at[1], at[2], 2.0, s.gain, 0.6, play);
                self.cut_cue_draws += crate::base::rng::native_draws() - d;
                self.cut_note(format!("sound {} at {:.2}", s.name, s.t));
            }
        }
        while let Some(e) = p.c.event.get(p.next[2]).filter(|e| e.t <= until).cloned() {
            p.next[2] += 1;
            self.cut_event(&e.name, e.t);
        }
        while let Some(s) = p.c.say.get(p.next[3]).filter(|s| s.t <= until).cloned() {
            p.next[3] += 1;
            if let Some((u, subject)) = cues::say_subject(&s.use_, &s.subject) {
                let d = crate::base::rng::native_draws();
                self.aside(u, subject);
                self.cut_cue_draws += crate::base::rng::native_draws() - d;
                self.cut_note(format!("say {} {} at {:.2}", s.use_, s.subject, s.t));
            }
        }
    }

    fn cut_event(&mut self, name: &str, t: f64) {
        if let Some(run) = cues::event(name) {
            let d = crate::base::rng::native_draws();
            run(self);
            self.cut_cue_draws += crate::base::rng::native_draws() - d;
            self.cut_note(format!("event {name} at {t:.2}"));
        }
    }

    /// The skip: the end evaluated, never the world fast-forwarded. Every
    /// event not yet run whose effect must not be lost runs now, in order
    /// (its sounds and asides do not); then back to your eyes, quickly if
    /// the camera is near them, through black if it is far
    fn cut_skip(&mut self, p: &mut Playing) {
        let rest: Vec<Event> = p.c.event[p.next[2]..].to_vec();
        p.next = [p.c.fade.len(), p.c.sound.len(), p.c.event.len(), p.c.say.len()];
        for e in rest.iter().filter(|e| e.on_skip) {
            self.cut_event(&e.name, e.t);
        }
        p.how = "skipped";
        let aspect = self.camera.camera().aspect;
        let near = match p.gameplay {
            Some(g) => {
                let k = p.keyed(self, g.fov, aspect, 0.0);
                let (d, turn) = Pose::apart(&g, &Pose::mix(&g, &k, p.weight()));
                d < SKIP_NEAR && turn < SKIP_NEAR_TURN
            }
            None => true,
        };
        if near || p.c.then == Then::Card {
            p.out = Some((p.t, SKIP_BLEND));
        } else {
            p.dip = Some(p.t);
        }
        self.cut_note(format!("skipped at {:.2} ({})", p.t, if near { "blend" } else { "dip" }));
    }

    /// Something hunting you, or searching near: a live cutscene breaks off
    pub fn cutscene_danger(&self) -> bool {
        matches!(self.holding_breath(), Some("hunted" | "searched"))
    }

    /// A key while your hands are taken: Escape pauses, Mute mutes, the
    /// skip key (the use key, Space or Enter) held skips, any other shows
    /// the skip line
    pub fn cutscene_key(&mut self, code: &str) {
        use crate::session::input::Act;
        let acts = self.acts_of(code);
        if acts.contains(&Act::Pause) {
            return self.toggle_pause();
        }
        if acts.contains(&Act::Mute) {
            return self.on_action(Act::Mute);
        }
        let skip = self.is_skip_key(code);
        if let Some(p) = self.cut.as_mut() {
            p.hint = HINT_FOR;
            if skip {
                p.skip_down = true;
            }
        }
    }

    /// A key let go while your hands are taken
    pub fn cutscene_key_up(&mut self, code: &str) {
        if self.is_skip_key(code) {
            if let Some(p) = self.cut.as_mut() {
                p.skip_down = false;
                p.skip_held = 0.0;
            }
        }
    }

    /// The keys that skip: the use key as bound, Space and Enter
    pub fn is_skip_key(&self, code: &str) -> bool {
        code == "Space" || code == "Enter" || self.settings.keys(crate::session::input::Act::Interact).iter().any(|k| k == code)
    }

    /// End any cutscene now, where it is (a death, a new floor): no event
    /// after now runs, and the camera is play's from the next frame
    pub fn cutscene_end_now(&mut self, why: &str) {
        if let Some(p) = self.cut.take() {
            self.cut_note(format!("end {} ({why}) at {:.2}", p.c.id, p.t));
        }
        self.cut_veil = Veil::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pose(x: f64, yaw: f64) -> Pose {
        Pose { pos: [x, 1.6, 0.0], yaw, pitch: 0.1, roll: 0.0, fov: 64.0 }
    }

    #[test]
    fn a_mix_at_its_ends_is_each_pose_exactly() {
        let (a, b) = (pose(0.3, 3.0), pose(7.1, -3.0));
        assert_eq!(Pose::mix(&a, &b, 0.0), a);
        assert_eq!(Pose::mix(&a, &b, 1.0), b);
        // the short way round: from 3.0 to -3.0 passes π, not 0
        assert!(Pose::mix(&a, &b, 0.5).yaw.abs() > 3.0);
        let (d, turn) = Pose::apart(&a, &b);
        assert!((d - 6.8).abs() < 1e-9 && turn < 0.3);
    }

    #[test]
    fn a_things_front_is_its_plus_z() {
        // a thing turned to face +x: a point 1 m out of its face is 1 m along +x
        let f = Frame::thing([10.0, 0.0, 5.0], PI / 2.0);
        let p = f.at([0.0, 1.0, 1.0]);
        assert!((p[0] - 11.0).abs() < 1e-12 && (p[1] - 1.0).abs() < 1e-12 && (p[2] - 5.0).abs() < 1e-12, "{p:?}");
        // its right (+x) is -z in the world
        let r = f.at([1.0, 0.0, 0.0]);
        assert!((r[2] - 4.0).abs() < 1e-12, "{r:?}");
        // and an eye frame looks down its -z: ahead is where its yaw looks
        let eyes = Frame { o: [0.0; 3], yaw: 0.4, pitch: 0.0 };
        let ahead = eyes.at([0.0, 0.0, -1.0]);
        let (yaw, _) = super::super::curve::look_angles([0.0; 3], ahead);
        assert!((yaw - 0.4).abs() < 1e-12);
    }

    #[test]
    fn the_veil_follows_its_fades_in_time() {
        let mut c =
            crate::cutscene::model::parse_bundle(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/cutscene/sample.json")))
                .unwrap()
                .remove(0);
        c.fade =
            vec![Fade { t: 1.0, to: 1.0, over: 0.5, colour: "#000".into() }, Fade { t: 2.0, to: 0.0, over: 1.0, colour: "#200".into() }];
        assert_eq!(c.veil_at(0.5).0, 0.0);
        assert!((c.veil_at(1.25).0 - 0.5).abs() < 1e-9);
        assert_eq!(c.veil_at(1.9), (1.0, "#000".to_string()));
        assert!((c.veil_at(2.5).0 - 0.5).abs() < 1e-9);
        assert_eq!(c.veil_at(3.5), (0.0, "#200".to_string()));
        // a fade cut short by the next starts that one from where it was
        c.fade[1].t = 1.25;
        assert!((c.veil_at(1.25).0 - 0.5).abs() < 1e-9);
    }
}
