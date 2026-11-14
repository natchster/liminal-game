//! The Stairwell and the Loop glue.
//!
//! Some exits are not doors to the next level but a steel door onto a flight
//! of stairs going down (`modes::stairwell`, inspired by SCP-087), and on the
//! level whose anomaly is the Loop the exit is a plain office door into five
//! identical rooms that come back round (`modes::loop_rooms`, inspired by
//! SCP-970). Both are sub-modes of state 'playing': the level world is hidden,
//! the module runs its own movement, and this glue maps its output onto the
//! game — camera, lamps, fog, sanity, the soundscape, the HUD and the
//! inventory.
//!
//! Test instrumentation: [`trace_begin`] / [`trace_take`] record every call
//! this glue makes into the sound system and the other modules (sfx,
//! soundscape, messages, death, level changes), so the tests can compare
//! them with the recorded reference calls. Off, it costs one thread-local check.
//!
//! CONTRACT: the signatures below are called from other modules; keep them.
pub mod loop_rooms;
pub mod stairwell;

use crate::assets::creatures::{creature_geometry, with_cavity_shade};
use crate::audio::soundscape::{ChildVoice, ReverbPreset, Soundscape, VoiceSlot};
use crate::base::numeric::hypot2;
use crate::modes::loop_rooms::{LoopDeps, LoopEvent, LoopInput, LoopMode, LoopTextures, PickupKind, loop_reverb};
use crate::modes::stairwell::{GoneReason, STAIR, StairDeps, StairEvent, StairInput, Stairwell};
use crate::prelude::*;
use crate::visuals::textures::{WALL_PALETTES, make_wallpaper};

/// One reverb step per lap
pub const LOOP_ZONES: [&str; 7] = ["loop0", "loop1", "loop2", "loop3", "loop4", "loop5", "loop6"];

/// The descent gets longer each time
pub fn stair_flights_for(n: i32) -> i32 {
    40.min(8 + (n as f64 / 3.0).floor() as i32 * 5)
}

/// The child's next phrase (`r` is the frame's `rnd()`, drawn even
/// while the Face is out): it only wails while the Face is out.
pub fn stair_phrase(face_out: bool, r: f64) -> &'static str {
    if face_out {
        "wail"
    } else if r < 0.45 {
        "sob"
    } else if r < 0.62 {
        "please"
    } else if r < 0.8 {
        "help"
    } else if r < 0.92 {
        "downhere"
    } else {
        "wail"
    }
}

// ------------------------------------------------------------ instrumentation
/// One recorded call: a name (string arguments folded in, `ss.footstep:stair`)
/// and the numeric arguments.
pub type TraceEntry = (String, Vec<f64>);
thread_local! {
    static TRACE: RefCell<Option<Vec<TraceEntry>>> = const { RefCell::new(None) };
    static MODES_RNG: RefCell<Option<Rc<dyn Fn() -> f64>>> = const { RefCell::new(None) };
}
/// Start recording the glue's outgoing calls (tests).
pub fn trace_begin() {
    TRACE.with(|t| *t.borrow_mut() = Some(vec![]));
}
/// Stop recording and return what was recorded.
pub fn trace_take() -> Vec<TraceEntry> {
    TRACE.with(|t| t.borrow_mut().take().unwrap_or_default())
}
fn tracing() -> bool {
    TRACE.with(|t| t.borrow().is_some())
}
fn tr(name: &str, args: &[f64]) {
    TRACE.with(|t| {
        if let Some(v) = t.borrow_mut().as_mut() {
            v.push((name.to_string(), args.to_vec()));
        }
    });
}
/// Tests: the `rng` the Stairwell and the Loop are built with (the game passes
/// `native_random`; `None` restores that).
pub fn set_modes_rng(f: Option<Rc<dyn Fn() -> f64>>) {
    MODES_RNG.with(|r| *r.borrow_mut() = f);
}
fn modes_rng() -> Box<dyn FnMut() -> f64> {
    match MODES_RNG.with(|r| r.borrow().clone()) {
        Some(f) => Box::new(move || f()),
        None => Box::new(crate::base::rng::native_rnd),
    }
}

/// The soundscape as the glue calls it: each call is recorded with the very
/// arguments it is made with (defaults spelled out by the callers).
struct Ss(Rc<Soundscape>);
impl Ss {
    fn set_zone(&self, name: &str, fade: f64) {
        if tracing() {
            tr(&format!("ss.set_zone:{name}"), &[fade]);
        }
        self.0.set_zone(name, fade);
    }
    fn define_zone(&self, name: &str, p: ReverbPreset) {
        if tracing() {
            tr(&format!("ss.define_zone:{name}"), &[p.t60, p.damp, p.wet, p.pre, p.er as f64]);
        }
        self.0.define_zone(name, p);
    }
    fn hum_silence(&self) {
        tr("hum.silence", &[]);
        self.0.hum.silence();
    }
    fn hum_update(&self, dt: f64, x: f64, y: f64, z: f64, strength: f64) {
        tr("hum.update", &[dt, x, y, z, strength]);
        self.0.hum.update(dt, x, y, z, strength);
    }
    fn hum_detune(&self, c: f64, tc: f64) {
        tr("hum.detune", &[c, tc]);
        self.0.hum.detune(c, tc);
    }
    fn voice_slot(&self, ref_dist: f64) -> VoiceSlot {
        tr("ss.voice_slot", &[ref_dist]);
        self.0.voice_slot(ref_dist)
    }
    fn create_child_voice(&self, dest: &AudioNode) -> ChildVoice {
        tr("ss.create_child_voice", &[]);
        self.0.create_child_voice(Some(dest))
    }
    fn set_listener(&self, x: f64, y: f64, z: f64, fx: f64, fy: f64, fz: f64) {
        tr("ss.set_listener", &[x, y, z, fx, fy, fz]);
        self.0.set_listener(x, y, z, fx, fy, fz);
    }
    fn footstep(&self, surface: &str, weight: f64) {
        if tracing() {
            tr(&format!("ss.footstep:{surface}"), &[weight]);
        }
        self.0.footstep(surface, weight, None);
    }
    fn screech(&self, intensity: f64, dur: f64) {
        tr("ss.screech", &[intensity, dur]);
        self.0.screech(intensity, dur);
    }
    fn heartbeat(&self, bpm: f64, beats: i32, wrong: f64) {
        tr("ss.heartbeat", &[bpm, beats as f64, wrong]);
        self.0.heartbeat(bpm, beats, wrong, None);
    }
    fn knock(&self, count: i32, x: f64, y: f64, z: f64, gain: f64) {
        tr("ss.knock", &[count as f64, x, y, z, gain]);
        self.0.knock(count, x, y, z, gain);
    }
    /// (recorded with t0 relative to `now`)
    fn burst(&self, now: f64, dest: &AudioNode, t0: f64, dur: f64, ty: &str, f0: f64, q: f64, gain: f64) {
        if tracing() {
            tr(&format!("ss.burst:{ty}"), &[t0 - now, dur, f0, q, gain]);
        }
        self.0.burst(dest, t0, dur, ty, f0, q, gain, None, 0.002);
    }
    /// (recorded with t0 relative to `now`)
    fn tone(&self, now: f64, dest: &AudioNode, t0: f64, dur: f64, freq: f64, gain: f64, ty: &str, f1: Option<f64>) {
        if tracing() {
            tr(&format!("ss.tone:{ty}"), &[t0 - now, dur, freq, gain, f1.unwrap_or(f64::NAN)]);
        }
        self.0.tone(dest, t0, dur, freq, gain, ty, f1, 0.003);
    }
}
fn voice_set_level(v: &ChildVoice, level: f64, tc: f64) {
    tr("voice.set_level", &[level, tc]);
    v.set_level(level, tc);
}
/// `voice.say(name, t0, pitch, loud)` → the phrase's length (recorded with t0 relative to `now`)
fn voice_say(v: &ChildVoice, now: f64, name: &str, t0: f64, pitch: f64, loud: f64) -> f64 {
    let dur = v.say(name, Some(t0), pitch, loud);
    if tracing() {
        tr(&format!("voice.say:{name}"), &[t0 - now, pitch, loud, dur]);
    }
    dur
}
fn slot_at(s: &VoiceSlot, x: f64, y: f64, z: f64) {
    tr("slot.at", &[x, y, z]);
    s.at(x, y, z);
}
fn slot_occlude(s: &VoiceSlot, amount: f64) {
    tr("slot.occlude", &[amount]);
    s.occlude(amount);
}

/// A number as a flag: true unless it is 0 or NaN
#[inline]
fn truthy(v: f64) -> bool {
    v != 0.0 && !v.is_nan()
}

impl Game {
    // ============================================================ the Stairwell
    /// Start-up work of the Stairwell section: build the Stairwell (the
    /// Face's shell painted, a little extra cavity shade) and add it to the
    /// scene, hidden. Called from `boot()` between the HUD and the flow sections.
    pub fn init_modes(&mut self) {
        let stair_face_geo = creature_geometry("stairFace");
        // hollows are painted in; a little extra
        let mut deps = StairDeps::game(stair_face_geo.map(|g| with_cavity_shade(&g, 0.3, None)));
        deps.rng = modes_rng();
        let stairwell = Stairwell::new(deps);
        self.scene.add(&stairwell.group);
        self.stairwell = Some(stairwell);
        self.in_stairwell = false;
        self.stair_audio = Default::default();
        self.stair_voice = None;
        self.stair_slot = None;
        self.stair_near_exit = false;
        // the Loop's state: built on entry
        self.loop_rooms = None;
        self.loop_paper = vec![];
        self.in_loop = false;
        self.loop_near_exit = false;
        self.loop_audio = Default::default();
    }

    /// The Stairwell is one of the run's anomalies (`level::scp`)
    pub fn level_has_stairwell(&self, n: i32) -> bool {
        self.scp_on_level(n).as_deref() == Some("stairwell")
    }

    /// `soundscape`, recording what the glue asks of it
    fn modes_ss(&self) -> Option<Ss> {
        self.soundscape().map(Ss)
    }

    /// `show_msg(text, dur)` as the glue calls it (recorded)
    fn modes_msg(&mut self, text: &str, dur: f64) {
        if tracing() {
            tr(&format!("msg:{text}"), &[dur]);
        }
        self.show_msg(text, dur);
    }

    pub fn enter_stairwell(&mut self) {
        if self.state != "playing" || self.in_stairwell {
            return;
        }
        self.state = "transition".into();
        tr("sfx.door_slam", &[]);
        self.sfx_door_slam();
        self.ui.fade.set_style("background", "#000");
        self.ui.fade.set_opacity(1.0);
        self.schedule(900.0, |g| g.enter_stairwell_now());
    }
    /// the body of `enter_stairwell`'s 900 ms timeout
    fn enter_stairwell_now(&mut self) {
        self.in_stairwell = true;
        self.asides_leave_floor();
        // your asides: once its screen has gone, if you are still on its stairs
        self.aside_later(6000.0, crate::audio::asides::Use::Arrive, "stairwell");
        self.world.set_visible(false);
        for e in &self.entities {
            if let Some(f) = &e.face {
                f.set_visible(false);
            }
        }
        for l in &self.pooled {
            l.set_intensity(0);
        }
        self.halluc_shape.set_visible(false);
        self.amb_light.set_intensity(0.015);
        self.hemi_light.set_intensity(0.02);
        self.fog_color.set_rgb(0, 0, 0);
        self.background.set_rgb(0, 0, 0);
        self.fog_density = 0.16;
        self.player_light.light().distance = 2.4;
        self.player_light.set_intensity(0.5);
        // the lamp never shows more than nine steps, however strong it is
        self.flashlight.light().distance = STAIR::LIGHT_STEPS as f64 * hypot2(STAIR::RUN, STAIR::RISE) + 0.4;
        if self.player.battery > 0.0 {
            self.player.flashlight_on = true;
        } // (`toggle_flashlight` is a no-op mid-transition)
        let flights = stair_flights_for(self.level_num);
        if let Some(s) = self.stairwell.as_mut() {
            s.enter(flights);
            s.group.set_visible(true);
        }
        self.ui.body.add_class("in-stairwell");
        if let Some(ss) = self.modes_ss() {
            ss.set_zone("stairwell", 0.4);
            ss.hum_silence();
            let slot = ss.voice_slot(4.0);
            let voice = ss.create_child_voice(&slot.input);
            voice_set_level(&voice, 0.55, 2.5);
            slot_occlude(&slot, 0.55); // through concrete, far below
            self.stair_slot = Some(slot);
            self.stair_voice = Some(voice);
        }
        self.stair_audio.next_phrase = 2.5;
        self.stair_audio.next_knock = 45.0;
        self.stair_audio.step_acc = 0.0;
        self.state = "playing".into();
        self.ui.fade.set_opacity(0.0);
        self.modes_msg("THE DOOR LOCKS BEHIND YOU. SOMEONE IS CRYING, FAR BELOW.", 4.0);
    }

    /// Leave the stairwell mode without changing level (death / restart / load)
    pub fn leave_stairwell_mode(&mut self) {
        if !self.in_stairwell {
            return;
        }
        self.in_stairwell = false;
        if let Some(s) = &self.stairwell {
            s.group.set_visible(false);
            s.face.set_visible(false);
        }
        self.world.set_visible(true);
        self.flashlight.light().distance = 28.0;
        self.player_light.light().distance = 7.0;
        self.player_light.set_intensity(2.4);
        self.ui.body.remove_class("in-stairwell");
        if let Some(v) = self.stair_voice.take() {
            tr("voice.stop", &[]);
            v.stop();
        }
        self.stair_near_exit = false;
        self.ui.prompt.set_text("");
    }

    /// The door at the bottom: on to the next level
    pub fn exit_stairwell(&mut self) {
        if !self.in_stairwell || self.state != "playing" {
            return;
        }
        self.state = "transition".into();
        tr("sfx.door", &[]);
        self.sfx_door();
        self.ui.fade.set_style("background", "#000");
        self.ui.fade.set_opacity(1.0);
        self.schedule(1100.0, |g| {
            g.leave_stairwell_mode();
            let n = g.level_num + 1;
            tr("init_level", &[n as f64]);
            g.init_level(n);
            tr("save_game", &[]);
            g.save_game();
            g.state = "playing".into();
            let title = g.level_title();
            // (the recordings trace the message; the screen shows the level card)
            if tracing() {
                tr(&format!("msg:LEVEL {} — {}", g.level_num, title), &[3.0]);
            }
            g.show_level_card(&title);
            g.ui.fade.set_opacity(0.0);
        });
    }

    /// The movement keys as the two sub-modes read them: (crouch, mx, mz, sprint key).
    fn modes_move_keys(&self) -> (bool, f64, f64, bool) {
        let want_crouch = self.wants_crouch();
        let (mut mx, mut mz) = (0.0, 0.0);
        if self.held(crate::session::input::Act::Forward) {
            mz += 1.0;
        }
        if self.held(crate::session::input::Act::Back) {
            mz -= 1.0;
        }
        if self.held(crate::session::input::Act::Left) {
            mx -= 1.0;
        }
        if self.held(crate::session::input::Act::Right) {
            mx += 1.0;
        }
        if self.touch_input.active {
            mx = self.touch_input.x;
            mz = self.touch_input.z;
        }
        let sprint_key = self.wants_sprint();
        (want_crouch, mx, mz, sprint_key)
    }

    pub fn update_stairwell(&mut self, dt: f64) {
        if self.stairwell.is_none() {
            return;
        } // (built at start-up in the game)
        self.ll_on = 0.0; // the Stairwell lights itself; the level's light map is not for it
        let (want_crouch, mx, mz, sprint_key) = self.modes_move_keys();
        // a run by a press lasts until you stop
        if mx == 0.0 && mz == 0.0 {
            self.input.run_latched = false;
        }
        self.player.crouch = want_crouch;
        let moving = mx != 0.0 || mz != 0.0;
        let sprinting = !self.player.crouch && sprint_key && self.sprint_ready() && moving;
        let l = {
            let h = hypot2(mx, mz);
            if truthy(h) { h } else { 1.0 }
        };
        // stairs are slower than floors; running them is loud and costs more
        let speed = if sprinting {
            3.4
        } else if self.player.crouch {
            1.2
        } else {
            2.1
        };
        if sprinting {
            self.player.stamina = 0f64.max(self.player.stamina - STAM_DRAIN * 1.3 * dt);
        } else {
            self.player.stamina = STAM_MAX.min(self.player.stamina + STAM_REGEN * 0.8 * dt);
        }
        let inp = StairInput::with_camera(mz / l, mx / l, speed, self.player.yaw, self.player.pitch, &self.camera);
        let (out, gp) = {
            let s = self.stairwell.as_mut().unwrap();
            let out = s.update(dt, &inp);
            (out, s.group.pos())
        };
        // camera: eye over the tread, a small dip on every step
        if moving {
            self.player.bob_phase += dt * speed * 2.4;
            self.player.bob_amp = lerp(self.player.bob_amp, if sprinting { 0.06 } else { 0.035 }, dt * 6.0);
        } else {
            self.player.bob_amp = lerp(self.player.bob_amp, 0.0, dt * 6.0);
        }
        let bob = self.player.bob_phase.dsin().abs() * self.player.bob_amp * self.settings.bob;
        let eye_drop = if self.player.crouch { EYE - CROUCH_EYE } else { 0.0 };
        // (Settings: head bob and shake scale; the draw stays)
        let shake_k = if self.settings.shake { 1.0 } else { 0.0 };
        let shake = if self.shake_t > 0.0 { rand2(-1.0, 1.0) * self.shake_t * 0.05 * shake_k } else { 0.0 };
        self.camera.position().set(gp.x + out.x, gp.y + out.y - eye_drop + bob + shake, gp.z + out.z);
        self.camera.rotation().set(self.player.pitch, self.player.yaw, 0);
        if self.shake_t > 0.0 {
            self.shake_t = 0f64.max(self.shake_t - dt);
        }
        let cp = self.camera.pos();
        self.player.pos.set(cp.x, EYE, cp.z);
        // flashlight: battery as usual, reach capped
        if self.player.flashlight_on {
            self.player.battery = 0f64.max(self.player.battery - BAT_DRAIN * 0.3 * dt);
            if self.player.battery <= 0.0 {
                self.player.flashlight_on = false;
                self.modes_msg("BATTERY DEAD", 1.6);
            }
        }
        let fi = if self.player.flashlight_on { 3.2 * self.beam_battery_scale(0.5) } else { 0.0 };
        self.flashlight.set_intensity(fi);
        self.flashlight.position().copy(&cp);
        let fwd = self.player.fwd();
        let pc = self.player.pitch.dcos();
        self.flashlight_target.position().set(cp.x + fwd.x * pc * 5.0, cp.y + self.player.pitch.dsin() * 5.0, cp.z + fwd.z * pc * 5.0);
        self.player_light.position().copy(&cp);
        // the dark wears on you; the lamp keeps it off
        if !self.player.flashlight_on {
            self.player.sanity = 0f64.max(self.player.sanity - dt * 1.4);
        }
        let ss = self.modes_ss();
        // footsteps on concrete treads
        if moving {
            self.stair_audio.step_acc += dt * speed;
            if self.stair_audio.step_acc > (if sprinting { 0.75 } else { 0.62 }) {
                self.stair_audio.step_acc = 0.0;
                if let (Some(ss), false) = (&ss, self.player.crouch) {
                    ss.footstep("stair", if sprinting { 1.0 } else { 0.55 });
                }
            }
        }
        // the child: the same distance below you at every depth, and only ever
        // louder while the Face is out
        if let Some(ss) = &ss {
            let fdir = self.player.fwd();
            ss.set_listener(cp.x, cp.y, cp.z, fdir.x, self.player.pitch.dsin(), fdir.z);
            let (vp, face_out, landing, flights) = {
                let s = self.stairwell.as_ref().unwrap();
                (s.voice_pos(), s.st.face.is_some(), s.landing(), s.flights())
            };
            if let Some(slot) = &self.stair_slot {
                slot_at(slot, gp.x + vp.x, gp.y + vp.y, gp.z + vp.z);
            }
            self.stair_audio.next_phrase -= dt;
            let idle = self.stair_voice.as_ref().is_some_and(|v| !v.busy());
            if self.stair_audio.next_phrase <= 0.0 && idle {
                let r = rnd();
                let phrase = stair_phrase(face_out, r);
                let now = self.ac_time();
                let dur = voice_say(self.stair_voice.as_ref().unwrap(), now, phrase, now + 0.05, rand2(0.95, 1.06), 1.0);
                self.stair_audio.next_phrase = dur + rand2(1.2, 4.5);
            }
            if let Some(v) = &self.stair_voice {
                voice_set_level(v, if face_out { 1.05 } else { 0.55 }, if face_out { 0.3 } else { 2.0 });
            }
            // knocking on the door far above, once you are deep
            self.stair_audio.next_knock -= dt;
            if self.stair_audio.next_knock <= 0.0 && landing as f64 > flights as f64 * 0.4 {
                self.stair_audio.next_knock = rand2(30.0, 60.0);
                ss.knock(rand2(3.0, 7.0).floor() as i32, gp.x, gp.y + 6.0, gp.z - 1.5, 0.35);
            }
            if face_out {
                self.stair_audio.heart_t -= dt;
                if self.stair_audio.heart_t <= 0.0 {
                    self.stair_audio.heart_t = 3.2;
                    ss.heartbeat(96.0, 4, if flights > 20 { 1.0 } else { 0.4 });
                }
            }
        }
        for ev in &out.events {
            match ev {
                StairEvent::FaceSeen { .. } => {
                    self.player.sanity = 0f64.max(self.player.sanity - 4.0);
                }
                StairEvent::FaceJerk { n, .. } => {
                    self.shake_t = 0.5;
                    self.ui.flash.set_opacity(0.35);
                    self.schedule(120.0, |g| g.ui.flash.set_opacity(0.0));
                    if let Some(ss) = &ss {
                        ss.screech(if *n == 1 { 0.55 } else { 0.85 }, if *n == 1 { 0.6 } else { 0.9 });
                    }
                    self.player.sanity = 0f64.max(self.player.sanity - if *n == 1 { 10.0 } else { 18.0 });
                }
                StairEvent::FaceGone { reason } => {
                    if *reason == GoneReason::Approached
                        && let Some(ss) = &ss
                    {
                        ss.screech(0.4, 0.4);
                    }
                    if *reason == GoneReason::Approached {
                        self.player.sanity = 0f64.max(self.player.sanity - 8.0);
                    }
                }
                StairEvent::FaceBehind => {
                    self.shake_t = 0.9;
                    self.ui.flash.set_opacity(0.7);
                    self.schedule(180.0, |g| g.ui.flash.set_opacity(0.0));
                    if let Some(ss) = &ss {
                        ss.screech(1.0, 1.1);
                    }
                    self.player.sanity = 0f64.max(self.player.sanity - 38.0);
                }
                StairEvent::FaceAppear | StairEvent::FaceBehindArmed => {}
            }
        }
        if self.player.sanity <= 0.0 {
            tr("on_death:sanity", &[]);
            self.on_death("sanity");
            return;
        }
        // HUD: a camcorder counter in the corner
        {
            let s = self.stairwell.as_ref().unwrap();
            let rec = if truthy((self.perf_t * 1.5).floor() % 2.0) { "●" } else { "○" };
            let t = format!("{} REC   LANDING {} / {}   DEPTH {} M", rec, s.landing(), s.flights(), fmt_fixed(s.depth_m(), 0));
            self.ui.stair_hud.set_text(&t);
        }
        self.stair_near_exit = out.near_exit;
        self.ui.prompt.set_text(if self.stair_near_exit { "[E] OPEN THE DOOR" } else { "" });
    }

    // ============================================================ the Loop
    /// You come in at lap 2 or 3, fixed by the level seed
    pub fn loop_entry_lap(&self) -> i32 {
        2 + ((seed_for_level(self.run_seed, self.level_num) >> 7) % 2) as i32
    }

    /// A soft door-latch click on every seam, so you can count them
    pub fn loop_latch(&self, delay: f64, gain: f64) {
        if self.ac.is_none() {
            return;
        }
        let Some(ss) = self.modes_ss() else { return };
        let now = self.ac_time();
        let (t, w) = (now + delay, ss.0.world.clone());
        ss.burst(now, &w, t, 0.018, "bandpass", 3100.0, 2.2, 0.09 * gain);
        ss.burst(now, &w, t + 0.035, 0.06, "lowpass", 520.0, 0.9, 0.12 * gain);
        ss.tone(now, &w, t + 0.035, 0.07, 170.0, 0.045 * gain, "sine", Some(95.0));
    }

    pub fn enter_loop(&mut self) {
        if self.state != "playing" || self.in_loop {
            return;
        }
        self.state = "transition".into();
        tr("sfx.door", &[]);
        self.sfx_door();
        self.ui.fade.set_style("background", "#000");
        self.ui.fade.set_opacity(1.0);
        self.schedule(900.0, |g| g.enter_loop_now());
    }
    /// (tools) The Loop on lap `lap`, you `dist` m in front of its seated
    /// figure, looking at it (`--screen loop-figure`).
    pub fn debug_loop_figure(&mut self, lap: i32, dist: f64) {
        self.enter_loop_now();
        if let Some(lr) = self.loop_rooms.as_mut() {
            lr.face_figure(lap, dist);
            self.player.yaw = lr.state().yaw;
        }
        self.update_loop(0.0);
    }
    /// the body of `enter_loop`'s 900 ms timeout
    fn enter_loop_now(&mut self) {
        // plain yellowed paper from the game's own generator (the level's
        // variants may carry a loud motif); both are disposed on the way out
        for t in self.loop_paper.drain(..) {
            t.dispose();
        }
        self.loop_paper = vec![make_wallpaper(&WALL_PALETTES[0], "plain", 0), make_wallpaper(&WALL_PALETTES[0], "plain", 3)];
        let textures = LoopTextures {
            wall: Some(self.loop_paper[0].clone()),
            wall_dirty: Some(self.loop_paper[1].clone()),
            carpet: self.tex.carpet_variants.get(1).cloned(),
            ceiling: self.tex.ceiling_variants.first().cloned(),
            panel: Some(self.tex.light_tex.clone()),
        };
        let mut deps = LoopDeps::game(textures);
        deps.rng = modes_rng();
        let loop_rooms = LoopMode::new(deps);
        self.scene.add(&loop_rooms.group);
        self.loop_rooms = Some(loop_rooms);
        self.in_loop = true;
        self.asides_leave_floor();
        self.world.set_visible(false);
        for e in &self.entities {
            if let Some(f) = &e.face {
                f.set_visible(false);
            }
        }
        self.halluc_shape.set_visible(false);
        let lap = self.loop_entry_lap();
        let yaw = {
            let lr = self.loop_rooms.as_mut().unwrap();
            lr.enter(lap);
            lr.state().yaw
        };
        self.player.yaw = yaw;
        self.player.pitch = 0.0;
        self.ui.body.add_class("in-loop");
        if let Some(ss) = self.modes_ss() {
            for (k, name) in LOOP_ZONES.iter().enumerate() {
                let r = loop_reverb(k as f64);
                ss.define_zone(name, ReverbPreset::new(r.t60, r.damp, r.wet, r.pre, r.er));
            }
        }
        self.loop_audio.step_acc = 0.0;
        self.loop_audio.touch_t = 0.0;
        self.update_loop(0.0); // camera, lamps and fog before the first frame
        tr("renderer.compile", &[]);
        self.want_warm_up = true; // shader warm-up: the app compiles the scene after this frame
        self.state = "playing".into();
        self.ui.fade.set_opacity(0.0);
        self.loop_latch(0.35, 1.0); // the door you came in by, shutting
        self.modes_msg("STRAIGHT AHEAD. IT IS ALWAYS STRAIGHT AHEAD.", 4.0);
    }

    /// Leave the Loop without changing level (death / restart / load / exit)
    pub fn leave_loop_mode(&mut self) {
        if !self.in_loop {
            return;
        }
        self.in_loop = false;
        if let Some(mut lr) = self.loop_rooms.take() {
            self.scene.remove(&lr.group);
            lr.dispose();
        }
        for t in self.loop_paper.drain(..) {
            t.dispose();
        }
        self.world.set_visible(true);
        self.player_light.light().distance = 7.0;
        self.player_light.set_intensity(2.4);
        self.amb_light.light().color.set_hex(0xa89a70);
        self.hemi_light.light().color.set_hex(0xd0c48c);
        for l in &self.pooled {
            l.light().color.set_rgb(1.0, 0.933, 0.769);
        }
        tr("release_pooled_lights", &[]);
        self.release_pooled_lights();
        if let Some(ss) = self.modes_ss() {
            ss.hum_detune(0.0, 0.1);
        }
        self.ui.body.remove_class("in-loop");
        self.loop_near_exit = false;
        self.ui.prompt.set_text("");
    }

    /// The door in room 1 at lap 0: out, to the next level
    pub fn exit_loop(&mut self) {
        if !self.in_loop || self.state != "playing" {
            return;
        }
        tr("next_level", &[]);
        self.next_level(); // fades, then `init_level` → `leave_loop_mode`
    }

    pub fn update_loop(&mut self, dt: f64) {
        if self.loop_rooms.is_none() {
            return;
        } // (only ever called while inside)
        self.ll_on = 0.0; // the Loop lights its own rooms (see below)
        let (want_crouch, mx, mz, sprint_key) = self.modes_move_keys();
        // a run by a press lasts until you stop
        if mx == 0.0 && mz == 0.0 {
            self.input.run_latched = false;
        }
        self.player.crouch = want_crouch;
        let moving = mx != 0.0 || mz != 0.0;
        let sprinting = !self.player.crouch && sprint_key && self.sprint_ready() && moving;
        let l = {
            let h = hypot2(mx, mz);
            if h.is_nan() { h } else { 1f64.max(h) }
        };
        let speed = if sprinting {
            SPRINT
        } else if self.player.crouch {
            CROUCH_WALK
        } else {
            WALK
        };
        if sprinting {
            self.player.stamina = 0f64.max(self.player.stamina - STAM_DRAIN * dt);
        } else {
            self.player.stamina = STAM_MAX.min(self.player.stamina + STAM_REGEN * dt);
        }
        let inp = LoopInput { fwd: mz / l, strafe: mx / l, speed, yaw: self.player.yaw };
        let out = self.loop_rooms.as_mut().unwrap().update(dt, &inp);
        // camera: the Loop sits at the world origin
        if moving {
            self.player.bob_phase += dt * speed * 2.1;
            self.player.bob_amp = lerp(
                self.player.bob_amp,
                if sprinting {
                    0.075
                } else if self.player.crouch {
                    0.025
                } else {
                    0.045
                },
                dt * 6.0,
            );
        } else {
            self.player.bob_amp = lerp(self.player.bob_amp, 0.0, dt * 6.0);
        }
        let bob = self.player.bob_phase.dsin().abs() * self.player.bob_amp * self.settings.bob;
        // (Settings: head bob and shake scale; the draw stays)
        let shake_k = if self.settings.shake { 1.0 } else { 0.0 };
        let shake = if self.shake_t > 0.0 { rand2(-1.0, 1.0) * self.shake_t * 0.05 * shake_k } else { 0.0 };
        self.camera.position().set(out.x, (if self.player.crouch { CROUCH_EYE } else { EYE }) + bob + shake, out.z);
        self.camera.rotation().set(self.player.pitch, self.player.yaw, 0);
        if self.shake_t > 0.0 {
            self.shake_t = 0f64.max(self.shake_t - dt);
        }
        self.player.pos.set(out.x, EYE, out.z);
        if self.player.flashlight_on {
            self.player.battery = 0f64.max(self.player.battery - BAT_DRAIN * dt);
            if self.player.battery <= 0.0 {
                self.player.flashlight_on = false;
                self.modes_msg("BATTERY DEAD", 1.6);
            }
        }
        let fi = if self.player.flashlight_on { 3.2 * self.beam_battery_scale(0.5) } else { 0.0 };
        self.flashlight.set_intensity(fi);
        let cp = self.camera.pos();
        self.flashlight.position().copy(&cp);
        let fwd = self.player.fwd();
        let pc = self.player.pitch.dcos();
        self.flashlight_target.position().set(cp.x + fwd.x * pc * 5.0, cp.y + self.player.pitch.dsin() * 5.0, cp.z + fwd.z * pc * 5.0);
        self.player_light.position().copy(&cp);
        // the nearest lamps take the pooled lights; the drift sets fog and sky
        let e = out.env;
        for i in 0..self.pooled.len() {
            let pl = &self.pooled[i];
            if i >= out.light_count {
                pl.set_intensity(0);
                continue;
            }
            let s = out.lights[i];
            pl.position().set(s.x, s.y, s.z);
            if let Some(t) = pl.target() {
                t.position().set(s.x, 0, s.z);
            }
            pl.light().color.set_rgb(s.r, s.g, s.b);
            pl.set_intensity(s.i * 7.5);
            pl.light().distance = 9.0;
        }
        self.fog_color.set_rgb(e.fog_r, e.fog_g, e.fog_b);
        self.background = self.fog_color;
        self.fog_density = e.fog;
        self.amb_light.set_intensity(e.amb);
        self.amb_light.light().color.set_rgb(e.sky_r * 0.8, e.sky_g * 0.75, e.sky_b * 0.6);
        self.hemi_light.set_intensity(e.hemi);
        self.hemi_light.light().color.set_rgb(e.sky_r, e.sky_g, e.sky_b);
        self.hemi_light.light().ground_color.set_rgb(e.ground_r, e.ground_g, e.ground_b);
        self.player_light.light().distance = 6.0;
        self.player_light.set_intensity(e.glow * 2.4);
        // sound: the hum drops and the rooms ring longer every lap; steps on carpet
        let ss = self.modes_ss();
        if let (Some(ss), true) = (&ss, self.ac.is_some()) {
            ss.set_listener(cp.x, cp.y, cp.z, fwd.x, self.player.pitch.dsin(), fwd.z);
            ss.set_zone(LOOP_ZONES[e.reverb.clamp(0, 6) as usize], 1.2);
            let hp = out.lights[0];
            if out.light_count > 0 && e.hum > 0.06 && hp.i > 0.05 {
                ss.hum_update(dt, hp.x, hp.y, hp.z, e.hum * 1f64.min(hp.i));
            } else {
                ss.hum_silence();
            }
            ss.hum_detune(round_half_up(e.hum_cents), 0.6);
            if moving {
                self.loop_audio.step_acc += dt * speed * 2.1;
                if self.loop_audio.step_acc > (if sprinting { 3.4 } else { 2.6 }) {
                    self.loop_audio.step_acc = 0.0;
                    if !self.player.crouch {
                        ss.footstep("office", if sprinting { 1.0 } else { 0.6 });
                    }
                }
            }
        }
        for ev in &out.events {
            match ev {
                LoopEvent::Seam { .. } => self.loop_latch(0.0, 1.0),
                LoopEvent::Pickup { kind, .. } => {
                    if *kind == PickupKind::Almond {
                        self.player.bottles += 1;
                        tr("pick_text:almondPickup", &[]);
                        let t = self.pick_text("almondPickup");
                        self.modes_msg(&t, 1.6);
                        tr("hint_first_use:bottle", &[]);
                        self.hint_first_use("bottle");
                    } else {
                        self.player.batteries += 1;
                        tr("pick_text:batteryPickup", &[]);
                        let t = self.pick_text("batteryPickup");
                        self.modes_msg(&t, 1.6);
                        tr("hint_first_use:battery", &[]);
                        self.hint_first_use("battery");
                    }
                    tr("sfx.pickup", &[]);
                    self.sfx_pickup();
                }
                LoopEvent::FigureSeen { .. } => {
                    self.player.sanity = 0f64.max(self.player.sanity - 3.0);
                    if let Some(ss) = &ss {
                        ss.heartbeat(84.0, 3, 0.2);
                    }
                }
                LoopEvent::FigureNearer { moved, .. } => {
                    self.shake_t = 0.35;
                    self.player.sanity = 0f64.max(self.player.sanity - 12f64.min(3.0 + moved * 1.5));
                    if let Some(ss) = &ss {
                        ss.screech(clamp(0.2 + moved * 0.08, 0.2, 0.65), 0.5);
                    }
                }
                LoopEvent::EntryGone => {}
            }
        }
        // touching it takes your mind, fast
        if out.touch > 0.0 {
            self.player.sanity = 0f64.max(self.player.sanity - 34.0 * out.touch);
            self.shake_t = self.shake_t.max(0.25);
            self.loop_audio.touch_t -= out.touch;
            if self.loop_audio.touch_t <= 0.0 {
                self.loop_audio.touch_t = 0.9;
                if let Some(ss) = &ss {
                    ss.screech(0.6, 0.6);
                }
                self.ui.flash.set_opacity(0.3);
                self.schedule(100.0, |g| g.ui.flash.set_opacity(0.0));
            }
        }
        // the calm laps settle you; past the sixth the mind drains, faster each lap
        self.player.sanity = clamp(self.player.sanity - e.drain * dt, 0.0, SAN_MAX);
        if self.player.sanity <= 0.0 {
            tr("on_death:sanity", &[]);
            self.on_death("sanity");
            return;
        }
        self.loop_near_exit = out.near_exit;
        self.ui.prompt.set_text(if self.loop_near_exit { "[E] OPEN THE EXIT DOOR" } else { "" });
    }
}
