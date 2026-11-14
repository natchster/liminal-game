//! The player (`docs/design/CUTSCENES.md`, the design): one cutscene at a time,
//! on a clock of its own (time played: it stops with the pause), its camera
//! a function of that time and its anchors. It blends in from the camera as
//! it was when it began (its start pose), and out into the gameplay camera,
//! so it begins where you were looking and its last frame is the gameplay
//! camera exactly.
//!
//! Each frame, around play's own update (`session::frame`):
//! `cutscene_pre` puts back the gameplay camera of the frame before (the
//! player's update eases its roll and field of view from the camera as it
//! was, and must never ease from a cutscene's pose), play draws its camera,
//! then `cutscene_post` reads that camera, advances the clock, runs the
//! cues that came due (each once, even across a long frame, with the
//! cutscene in place: one that ends it, or starts another, ends the step)
//! and writes the mix. While one plays your hands are taken
//! (`hands_taken`): keys and the mouse do nothing to your body, nothing
//! may strike you, your asides wait. However your hands come back (played,
//! skipped, broken off), its `finally` events run once. Nothing here draws
//! from a random stream.
use super::cues;
use super::curve::{catmull, lerp, lerp_angle, look_angles, smooth, sway};
use super::model::*;
use crate::base::game::Game;
use engine::dmath::DMath;
use std::f64::consts::PI;

/// Seconds the skip key is held to skip
pub const SKIP_HOLD: f64 = 0.6;
/// A skip near your eyes blends back over this
pub const SKIP_BLEND: f64 = 0.4;
/// ... near: within this many metres and this turn (radians), under a
/// veil of at most `SKIP_NEAR_VEIL`; further, it dips through black
/// instead of sweeping the camera across the place
pub const SKIP_NEAR: f64 = 1.0;
pub const SKIP_NEAR_TURN: f64 = 0.61;
pub const SKIP_NEAR_VEIL: f64 = 0.5;
/// The dip through black on a far skip: closing, then opening on your eyes
pub const DIP_OUT: f64 = 0.2;
pub const DIP_IN: f64 = 0.25;
/// A cutscene broken off by danger blends back over this
pub const BREAK_BLEND: f64 = 0.4;
/// The bars come in over this, and go over that
pub const BARS_IN: f64 = 0.4;
pub const BARS_OUT: f64 = 0.5;
/// The skip line shows this long after a key, or after this much mouse
/// travel (pixels)
pub const HINT_FOR: f64 = 2.5;
pub const HINT_MOUSE: f64 = 60.0;
/// A repeat that starts part-way blends in over at least this
pub const REPEAT_BLEND: f64 = 0.2;
/// A walked segment's step (metres a step) and its bob (metres up, and
/// radians of yaw, for `walk` 1), near your own walk's
pub const STRIDE: f64 = 0.72;
pub const WALK_BOB: f64 = 0.022;
pub const WALK_SWAY: f64 = 0.006;
/// Danger (`cutscene_danger`): a search this near, a nerve this low
pub const DANGER_SEARCH: f64 = 25.0;
pub const DANGER_NERVE: f64 = 20.0;

/// A key placed from you is kept this far (metres) from walls
pub const KEY_CLEARANCE: f64 = 0.15;

/// `to`, or the point nearest it on the way from `from` that is clear of
/// walls by `KEY_CLEARANCE` (its height kept)
fn clear_of_walls(g: &Game, from: [f64; 3], to: [f64; 3]) -> [f64; 3] {
    if !g.circle_hits_wall(to[0], to[2], KEY_CLEARANCE) {
        return to;
    }
    for k in (0..12).rev() {
        let u = k as f64 / 12.0;
        let (x, z) = (lerp(from[0], to[0], u), lerp(from[2], to[2], u));
        if !g.circle_hits_wall(x, z, KEY_CLEARANCE) {
            return [x, to[1], z];
        }
    }
    [from[0], to[1], from[2]]
}

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
        let d = ((a.pos[0] - b.pos[0]).dpowi(2) + (a.pos[1] - b.pos[1]).dpowi(2) + (a.pos[2] - b.pos[2]).dpowi(2)).sqrt();
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
        let (s, c) = self.yaw.dsin_cos();
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
    /// what it is counted as for its short form (a death by its killer's
    /// kind: "death_taken:hound"); none: its id
    pub seen_as: Option<String>,
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

/// How a cutscene's hands-back looked, kept for the frame it ended in
/// (tests: the contract reads play's camera and the bars of that frame)
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ended {
    pub t: f64,
    pub shown: Pose,
    pub play: Pose,
    pub bars: f64,
    pub veil: f64,
}

/// A cue come due, run with the cutscene in place
#[derive(Clone, Debug)]
enum Due {
    Sound(Sound),
    Event(String, f64),
    Say(Say),
}

/// The cutscene playing
#[derive(Clone, Debug)]
pub struct Playing {
    pub c: Cutscene,
    /// which cutscene this is of the game's (a cue that ends it, or starts
    /// another, is seen by its number changing)
    pub r#gen: u64,
    /// its clock
    pub t: f64,
    /// where its clock started (its short form on a repeat)
    pub t0: f64,
    /// the camera when it began, whole (its roll and field of view too):
    /// it blends in from here
    pub start_pose: Pose,
    pub start: Frame,
    pub eyes: Frame,
    pub thing: Option<Frame>,
    pub killer: Option<usize>,
    /// the gameplay camera, as play drew it this frame
    pub gameplay: Option<Pose>,
    /// what was drawn
    pub shown: Option<Pose>,
    /// the weight of its camera over play's drawn (1 its own, 0 play's)
    pub w: f64,
    /// the next cue of each track to run: fade, sound, event, say
    next: [usize; 4],
    /// broken off or skipped near your eyes: back from here over this
    pub out: Option<(f64, f64)>,
    /// skipped from far: the veil closes from here, then the end
    pub dip: Option<f64>,
    /// the skip key is down (read from the keys held, each frame)
    pub skip_down: bool,
    pub skip_held: f64,
    /// the skip line shows for this much longer
    pub hint: f64,
    /// the mouse's travel toward showing the skip line
    pub mouse_moved: f64,
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
    /// Its own keys over its start pose: 0 at its start (a blend in), 1 after
    fn w_in(&self) -> f64 {
        let blend_in = if self.t0 > 0.0 { self.c.blend_in.max(REPEAT_BLEND) } else { self.c.blend_in };
        if blend_in > 0.0 { smooth(self.t - self.t0, 0.0, blend_in) } else { 1.0 }
    }
    /// Its camera over play's: 1, then 0 at its end (a blend out, a break-off,
    /// a skip near your eyes); a far skip's cut, under the veil, at its end
    pub fn weight(&self) -> f64 {
        let c = &self.c;
        let mut w = if c.then == Then::Back { 1.0 - smooth(self.t, c.length - c.blend_out, c.length) } else { 1.0 };
        if let Some((s, l)) = self.out {
            w = w.min(1.0 - smooth(self.t, s, s + l));
        }
        if self.dip.is_some() && self.t >= self.end() {
            w = 0.0;
        }
        w
    }
    /// How far the bars are in (0 out, 1 in): in from its start (a repeat's
    /// too), out at its time, or with a break-off or a skip
    pub fn bars(&self) -> f64 {
        let b = self.c.bars;
        let from = b.in_at.max(self.t0);
        let at_in = smooth(self.t, from, from + BARS_IN);
        let mut out = smooth(self.t, b.out_at, b.out_at + BARS_OUT);
        if let Some((s, l)) = self.out {
            out = out.max(smooth(self.t, s, s + l.min(BARS_OUT)));
        }
        if let Some(d) = self.dip {
            out = out.max(smooth(self.t, d, d + DIP_OUT));
        }
        at_in * (1.0 - out)
    }
    /// The veil its fades make at its time: after a break-off or a near
    /// skip, what it was then, lifting as the camera comes back; a far
    /// skip's dip closing over it
    pub fn veil(&self) -> (f64, String) {
        let (mut v, colour) = match self.out {
            Some((s, l)) => {
                let (v, colour) = self.c.veil_at(self.t.min(s));
                (v * (1.0 - smooth(self.t, s, s + l)), colour)
            }
            None => self.c.veil_at(self.t),
        };
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
    /// A key in the world: its pose (its field of view play's, moved by the
    /// key's own) and the live point it looks at, if any
    fn resolve(&self, g: &Game, k: &Key, fov_play: f64) -> (Pose, Option<[f64; 3]>) {
        let f = self.frame(g, k.anchor);
        let mut pos = f.at(k.pos);
        // a key placed from you (your eyes, the camera as it was) comes in
        // toward you until it is clear of walls: you die backed into a corner
        if matches!(k.anchor, Anchor::Eyes | Anchor::Start) {
            pos = clear_of_walls(g, f.o, pos);
        }
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
        // (the first key, the camera as it was: its roll and field of view too)
        let (roll, fov) = if k.anchor == Anchor::Start && k.pos == [0.0; 3] && k.look == Look::Ahead && k.roll == 0.0 && k.fov == 0.0 {
            (self.start_pose.roll, self.start_pose.fov)
        } else {
            (k.roll.to_radians(), fov_play + k.fov)
        };
        let pose = Pose { pos, yaw: yaw + k.yaw.to_radians(), pitch: pitch + k.pitch.to_radians(), roll, fov };
        (pose, target)
    }
    /// Its own keys at its time (`shake_k`: Settings' camera shake, 1 or 0,
    /// on its sway; `bob_k`: Settings' head bob, on a walk)
    pub fn keyed(&self, g: &Game, fov_play: f64, shake_k: f64, bob_k: f64) -> Pose {
        let keys = &self.c.camera;
        let t = self.t;
        let last = keys.len() - 1;
        let i = keys.iter().rposition(|k| k.t <= t).unwrap_or(0);
        if t <= keys[0].t || i >= last {
            let k = if t <= keys[0].t { &keys[0] } else { &keys[last] };
            let (mut p, _) = self.resolve(g, k, fov_play);
            let (sy, sp) = sway(t, k.sway * shake_k);
            p.yaw += sy;
            p.pitch += sp;
            return p;
        }
        let (a, b) = (&keys[i], &keys[i + 1]);
        let (pa, ta) = self.resolve(g, a, fov_play);
        if b.cut {
            // held until the cut
            let (sy, sp) = sway(t, a.sway * shake_k);
            return Pose { yaw: pa.yaw + sy, pitch: pa.pitch + sp, ..pa };
        }
        let (pb, tb) = self.resolve(g, b, fov_play);
        let u = (t - a.t) / (b.t - a.t);
        let e = b.ease.at(u);
        // the neighbours (a cut ends a run of keys: nothing bends across it)
        let p0 = if i > 0 && !a.cut { self.resolve(g, &keys[i - 1], fov_play).0.pos } else { pa.pos };
        let p3 = if i + 2 <= last && !keys[i + 2].cut { self.resolve(g, &keys[i + 2], fov_play).0.pos } else { pb.pos };
        let pos = catmull(p0, pa.pos, pb.pos, p3, e);
        let (mut yaw, mut pitch) = (lerp_angle(pa.yaw, pb.yaw, e), lerp(pa.pitch, pb.pitch, e));
        // both look at the same live point: follow it from where the camera is
        if let (Some(x), Some(y)) = (ta, tb)
            && a.look == b.look
        {
            let target = super::curve::lerp3(x, y, e);
            let (ly, lp) = look_angles(pos, target);
            yaw = ly + lerp(a.yaw, b.yaw, e).to_radians();
            pitch = lp + lerp(a.pitch, b.pitch, e).to_radians();
        }
        let (sy, sp) = sway(t, lerp(a.sway, b.sway, e) * shake_k);
        // a walked segment: a step's bob by the distance covered, faded out
        // at both its ends (so nothing jumps at a key)
        let mut pos = pos;
        if b.walk > 0.0 {
            let len = Pose::apart(&pa, &pb).0;
            let phase = e * len / STRIDE * PI;
            let fade = (PI * e).dsin() * b.walk * bob_k;
            pos[1] += phase.dsin().abs() * WALK_BOB * fade;
            yaw += phase.dsin() * WALK_SWAY * fade;
        }
        Pose {
            pos,
            yaw: yaw + sy,
            pitch: (pitch + sp).clamp(-PI / 2.0 + 0.01, PI / 2.0 - 0.01),
            roll: lerp(pa.roll, pb.roll, e),
            fov: lerp(pa.fov, pb.fov, e),
        }
    }
    /// Its camera before play's comes back: its keys blended in from its
    /// start pose
    pub fn own(&self, g: &Game, fov_play: f64, shake_k: f64, bob_k: f64) -> Pose {
        let k = self.keyed(g, fov_play, shake_k, bob_k);
        Pose::mix(&self.start_pose, &k, self.w_in())
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
            let until = self.fade.get(i + 1).map_or(t, |n| n.t.min(t));
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
    /// played now (`play_cutscene_data`)
    pub fn play_cutscene(&mut self, id: &str, anchors: Anchors) -> bool {
        // (a test's or a preview's own version of it, once)
        if let Some(c) = self.cut_override.take_if(|c| c.id == id) {
            return self.play_cutscene_data(c, anchors);
        }
        match super::get(id) {
            Some(c) => self.play_cutscene_data(c.clone(), anchors),
            None => {
                self.cut_note(format!("no cutscene {id}"));
                false
            }
        }
    }

    /// Play a cutscene given as data (tests make their own); false, and
    /// nothing changed, when one is playing, outside play (a death's is
    /// played while dead), in the Stairwell or the Loop, without a thing it
    /// names, or (one that gives your hands back) in danger: the caller
    /// then does what it would have done without one
    pub fn play_cutscene_data(&mut self, c: Cutscene, anchors: Anchors) -> bool {
        let names_thing = c.camera.iter().any(|k| k.anchor == Anchor::Thing || k.look == Look::At(Anchor::Thing))
            || c.sound.iter().any(|s| s.anchor == Anchor::Thing);
        let refuse = if self.cut.is_some() {
            Some("one is playing")
        } else if self.in_stairwell || self.in_loop {
            Some("the Stairwell and the Loop have their own camera")
        } else if !(self.state == "playing" || self.state == "dead" && c.then == Then::Card) {
            Some("not in play")
        } else if anchors.thing.is_none() && names_thing {
            Some("it names a thing and none was given")
        } else if c.camera.iter().any(|k| k.anchor == Anchor::Killer || k.look == Look::At(Anchor::Killer))
            && anchors.killer.is_none_or(|i| i >= self.entities.len())
        {
            Some("it names a killer and none was given")
        } else if c.then == Then::Back && self.cutscene_danger(true) {
            Some("danger")
        } else {
            None
        };
        if let Some(why) = refuse {
            self.cut_note(format!("refused {}: {why}", c.id));
            return false;
        }
        let seen = self.cut_seen.entry(anchors.seen_as.clone().unwrap_or(c.id.clone())).or_default();
        let t0 = if *seen > 0 || std::mem::take(&mut self.cut_force_repeat) { c.repeat_from } else { 0.0 };
        *seen += 1;
        let start_pose = self.camera_pose();
        let start = Frame { o: start_pose.pos, yaw: start_pose.yaw, pitch: start_pose.pitch };
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
        // a repeat starts part-way: what came before its start is not run,
        // but what must hold of it does (as on a skip)
        let before: Vec<Event> = c.event.iter().filter(|e| e.t < t0 && e.on_skip).cloned().collect();
        let mut next = [0; 4];
        for (i, ts) in [
            c.fade.iter().map(|x| x.t).collect::<Vec<_>>(),
            c.sound.iter().map(|x| x.t).collect(),
            c.event.iter().map(|x| x.t).collect(),
            c.say.iter().map(|x| x.t).collect(),
        ]
        .iter()
        .enumerate()
        {
            next[i] = ts.iter().take_while(|&&t| t < t0).count();
        }
        self.cut_gen += 1;
        let r#gen = self.cut_gen;
        self.cut_end = None;
        self.cut_hold = None;
        self.cut_shake = (0.0, 0.0, 0.0);
        self.cut = Some(Playing {
            c,
            r#gen,
            t: t0,
            t0,
            start_pose,
            start,
            eyes,
            thing: anchors.thing,
            killer: anchors.killer,
            gameplay: None,
            shown: None,
            w: 1.0,
            next,
            out: None,
            dip: None,
            skip_down: false,
            skip_held: 0.0,
            hint: 0.0,
            mouse_moved: 0.0,
            how: "played",
        });
        for e in before {
            self.cut_event(&e.name, e.t);
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

    /// The cutscene playing is still the one numbered `gen`
    fn cut_is(&self, r#gen: u64) -> bool {
        self.cut.as_ref().is_some_and(|p| p.r#gen == r#gen)
    }

    fn cutscene_step(&mut self, dt: f64) {
        // a veil left to open (a far skip on your eyes, a break-off in a fade)
        if self.cut.is_none() {
            if self.cut_veil.opening > 0.0 {
                self.cut_veil.opacity = (self.cut_veil.opacity - self.cut_veil.opening * dt).max(0.0);
                if self.cut_veil.opacity == 0.0 {
                    self.cut_veil.opening = 0.0;
                }
            }
            return;
        }
        let gameplay = self.camera_pose();
        // (its own cost is not danger: a look that shakes your nerve does not break itself off)
        let danger = self.state == "playing" && self.cutscene_danger(false);
        let skip_key = self.keys.iter().any(|k| self.is_skip_key(k));
        // the clock, the skip key held, danger
        let (r#gen, skip_now, broke) = {
            let p = self.cut.as_mut().unwrap();
            p.gameplay = Some(gameplay);
            p.t += dt;
            p.hint = (p.hint - dt).max(0.0);
            // (the key as held now: a focus lost or Command let go forgets it)
            p.skip_down &= skip_key;
            let free = p.out.is_none() && p.dip.is_none();
            let mut skip_now = false;
            if p.skip_down && p.c.skip == Skip::Hold && free {
                p.skip_held += dt;
                skip_now = p.skip_held >= SKIP_HOLD;
            } else {
                p.skip_held = 0.0;
            }
            let broke = !skip_now && free && p.c.breaks_off && p.c.world == World::Live && danger;
            if broke {
                p.how = "broken off";
            }
            (p.r#gen, skip_now, broke)
        };
        if broke {
            // back to your eyes as a skip goes: quickly if near, through black if far
            let near = self.cut_near_eyes();
            let p = self.cut.as_mut().unwrap();
            let t = p.t;
            if near {
                p.out = Some((t, BREAK_BLEND));
            } else {
                p.dip = Some(t);
            }
            self.cut_note(format!("broken off at {t:.2} ({})", if near { "blend" } else { "dip" }));
            // what must hold holds, as on a skip
            if !self.cut_run_held(r#gen) {
                return;
            }
        }
        if skip_now && !self.cut_skip(r#gen) {
            return;
        }
        // the cues come due, run with the cutscene in place
        for cue in self.cut_due() {
            match cue {
                Due::Sound(s) => {
                    if let Some(play) = cues::sound(&s.name) {
                        let p = self.cut.as_ref().unwrap();
                        let at = p.frame(self, s.anchor).at(s.pos);
                        let d = crate::base::rng::native_draws();
                        self.play_from(at[0], at[1], at[2], 2.0, s.gain, 0.6, play);
                        self.cut_cue_draws += crate::base::rng::native_draws() - d;
                        self.cut_note(format!("sound {} at {:.2}", s.name, s.t));
                    }
                }
                Due::Event(name, t) => self.cut_event(&name, t),
                Due::Say(s) => {
                    if let Some((u, subject)) = cues::say_subject(&s.use_, &s.subject) {
                        let d = crate::base::rng::native_draws();
                        self.aside(u, subject);
                        self.cut_cue_draws += crate::base::rng::native_draws() - d;
                        self.cut_note(format!("say {} {} at {:.2}", s.use_, s.subject, s.t));
                    }
                }
            }
            // a cue that ended it (a death, a new floor) or started another
            if !self.cut_is(r#gen) {
                return;
            }
        }
        // the camera
        let shake_k = if self.settings.shake { 1.0 } else { 0.0 };
        let bob_k = self.settings.bob;
        let (shown, w, done, card, dip, bars, veil, colour, t) = {
            let p = self.cut.as_ref().unwrap();
            let done = p.t >= p.end() - 1e-9;
            let card = p.c.then == Then::Card;
            // (a card's end holds its own camera; a far skip's cut is under the veil)
            let w = if done && !card { 0.0 } else { p.weight() };
            let shown = if w <= 0.0 { gameplay } else { Pose::mix(&gameplay, &p.own(self, gameplay.fov, shake_k, bob_k), w) };
            let (veil, colour) = p.veil();
            (shown, w, done, card, p.dip.is_some(), p.bars(), veil, colour, p.t)
        };
        // a death's shake (its sizes from `death_world`, a jitter of its
        // clock), as Settings' camera shake allows
        let (ay, ap, ar) = if card { self.cut_shake } else { (0.0, 0.0, 0.0) };
        let (jy, jp, jr) = super::curve::jitter(t);
        let shown = Pose {
            yaw: shown.yaw + ay * jy * shake_k,
            pitch: shown.pitch + ap * jp * shake_k,
            roll: shown.roll + ar * jr * shake_k,
            ..shown
        };
        self.set_camera_pose(&shown);
        self.cut_veil = Veil { opacity: veil, colour, opening: 0.0 };
        {
            let p = self.cut.as_mut().unwrap();
            p.w = w;
            p.shown = Some(shown);
        }
        if !done {
            return;
        }
        // the end: your hands back (or the card)
        let p = self.cut.take().unwrap();
        self.cut_end = Some(Ended { t, shown, play: gameplay, bars, veil });
        if dip || (!card && veil > 0.0) {
            // opened on your eyes now the hands are back
            self.cut_veil.opacity = if dip { 1.0 } else { veil };
            self.cut_veil.opening = 1.0 / DIP_IN;
        }
        self.cut_note(format!("end {} ({}) at {:.2}", p.c.id, p.how, t));
        if card {
            self.cut_hold = Some(shown);
        }
        for name in &p.c.finally {
            self.cut_event(name, t);
        }
        if card {
            // (the card's own draws: its line, as it always drew them)
            let d = crate::base::rng::native_draws();
            self.open_death_card();
            self.cut_cue_draws += crate::base::rng::native_draws() - d;
        }
    }

    /// The cues of the cutscene playing that came due, in order of time,
    /// each once (counted off as they are taken: nothing after a break-off
    /// or a skip, and at its end everything up to its length: its clock,
    /// summed a step at a time, may stop a hair short of a cue set there)
    fn cut_due(&mut self) -> Vec<Due> {
        let p = self.cut.as_mut().unwrap();
        let until = match (p.out, p.dip) {
            (Some((s, _)), _) | (_, Some(s)) => s.min(p.t),
            _ if p.t >= p.c.length - 1e-9 => p.c.length,
            _ => p.t,
        };
        // (the veil is a function of time, `veil_at`: only counted here)
        while p.c.fade.get(p.next[0]).is_some_and(|f| f.t <= until) {
            p.next[0] += 1;
        }
        let mut due: Vec<(f64, Due)> = vec![];
        while let Some(s) = p.c.sound.get(p.next[1]).filter(|s| s.t <= until) {
            due.push((s.t, Due::Sound(s.clone())));
            p.next[1] += 1;
        }
        while let Some(e) = p.c.event.get(p.next[2]).filter(|e| e.t <= until) {
            due.push((e.t, Due::Event(e.name.clone(), e.t)));
            p.next[2] += 1;
        }
        while let Some(s) = p.c.say.get(p.next[3]).filter(|s| s.t <= until) {
            due.push((s.t, Due::Say(s.clone())));
            p.next[3] += 1;
        }
        due.sort_by(|a, b| a.0.total_cmp(&b.0));
        due.into_iter().map(|(_, d)| d).collect()
    }

    fn cut_event(&mut self, name: &str, t: f64) {
        if let Some(run) = cues::event(name) {
            let d = crate::base::rng::native_draws();
            run(self);
            self.cut_cue_draws += crate::base::rng::native_draws() - d;
            self.cut_note(format!("event {name} at {t:.2}"));
        }
    }

    /// The events left whose effect must hold (`on_skip`), run now in
    /// order, the rest let go: on a skip and a break-off. → false when one
    /// of them ended the cutscene
    fn cut_run_held(&mut self, r#gen: u64) -> bool {
        let rest: Vec<Event> = {
            let p = self.cut.as_mut().unwrap();
            let rest = p.c.event[p.next[2]..].to_vec();
            p.next = [p.c.fade.len(), p.c.sound.len(), p.c.event.len(), p.c.say.len()];
            rest
        };
        for e in rest.iter().filter(|e| e.on_skip) {
            self.cut_event(&e.name, e.t);
            if !self.cut_is(r#gen) {
                return false;
            }
        }
        true
    }

    /// The skip: the end evaluated, never the world fast-forwarded. Every
    /// event not yet run whose effect must hold runs now, in order (its
    /// sounds and asides do not); then back to your eyes, quickly if the
    /// camera is near them, through black if it is far; a cutscene that
    /// ends on a card goes to its end. → false when an event ended it
    fn cut_skip(&mut self, r#gen: u64) -> bool {
        if !self.cut_run_held(r#gen) {
            return false;
        }
        let near = self.cut_near_eyes();
        let card = self.cut.as_ref().unwrap().c.then == Then::Card;
        let p = self.cut.as_mut().unwrap();
        p.how = "skipped";
        let t = p.t;
        if card {
            p.t = p.c.length;
        } else if near {
            p.out = Some((p.t, SKIP_BLEND));
        } else {
            p.dip = Some(p.t);
        }
        self.cut_note(format!(
            "skipped at {t:.2} ({})",
            if card {
                "to its end"
            } else if near {
                "blend"
            } else {
                "dip"
            }
        ));
        true
    }

    /// The camera near enough your eyes to blend back to them quickly
    /// (within `SKIP_NEAR` and `SKIP_NEAR_TURN`, under a veil of at most
    /// `SKIP_NEAR_VEIL`); further, a skip or a break-off dips through black
    fn cut_near_eyes(&self) -> bool {
        let p = self.cut.as_ref().unwrap();
        match p.gameplay {
            Some(g) => {
                let own = Pose::mix(&g, &p.own(self, g.fov, 0.0, 0.0), p.weight());
                let (d, turn) = Pose::apart(&g, &own);
                d < SKIP_NEAR && turn < SKIP_NEAR_TURN && p.veil().0 <= SKIP_NEAR_VEIL
            }
            None => true,
        }
    }

    /// Danger a cutscene that gives your hands back must not play into, and
    /// breaks off at: anything hunting you, anything searching near, the
    /// fog up on the Building; and, to start one, your nerve near its end
    /// (`nerve`: a look's own cost to it never breaks the look off)
    pub fn cutscene_danger(&self, nerve: bool) -> bool {
        let pp = self.player.pos;
        let near = |e: &crate::base::records::Entity| {
            let p = e.pos();
            ((p.x - pp.x).dpowi(2) + (p.z - pp.z).dpowi(2)).sqrt() < DANGER_SEARCH
        };
        if self.entities.iter().any(|e| e.kind != "gone" && (e.state == "hunt" || e.state == "search" && near(e))) {
            return true;
        }
        if self.building.is_some() && self.assembly.stage != crate::world::assembly::Stage::Quiet {
            return true;
        }
        nerve && self.player.sanity < DANGER_NERVE
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
        if acts.contains(&Act::Record) {
            return self.on_action(Act::Record);
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
        if self.is_skip_key(code)
            && let Some(p) = self.cut.as_mut()
        {
            p.skip_down = false;
            p.skip_held = 0.0;
        }
    }

    /// The keys that skip: the use key as bound, Space and Enter
    pub fn is_skip_key(&self, code: &str) -> bool {
        code == "Space" || code == "Enter" || self.settings.keys(crate::session::input::Act::Interact).iter().any(|k| k == code)
    }

    /// End any cutscene now, where it is (a death, a new floor): no event
    /// of its track after now runs, its `finally` does (what it left in the
    /// world is cleared however it ends), and the camera is play's again
    pub fn cutscene_end_now(&mut self, why: &str) {
        if let Some(p) = self.cut.take() {
            self.cut_note(format!("end {} ({why}) at {:.2}", p.c.id, p.t));
            if let Some(g) = p.gameplay {
                self.set_camera_pose(&g);
            }
            for name in &p.c.finally {
                self.cut_event(name, p.t);
            }
        }
        self.cut_veil = Veil::default();
        self.cut_shake = (0.0, 0.0, 0.0);
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
