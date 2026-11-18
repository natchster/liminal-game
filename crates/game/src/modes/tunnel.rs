//! The way in: the Jahamun Tunnel (`docs/design/WAY_IN.md`).
//!
//! A run begun from the title starts here, before the Building: the walkway
//! of the Jahamun Tunnel in Seoul at night (the place: `crates/jahamun`,
//! laid as a ring with no end, `jahamun::ring`). It is a sub-mode of
//! 'playing' like the Stairwell and the Loop: the level world (the first
//! floor, built behind it) hidden, the place's own movement on its own
//! ground, and this glue mapping its camera, fog and sound onto the
//! game's. The escape door under the plate is locked; after a minute or two
//! of walking (the run's own, `change_after`) it changes, where you cannot
//! see it happen, into the door into the Building, her note beside it.
//! Through it: level 1, entered as a floor is entered through a door.
//!
//! Nothing here draws from the level's stream or the per-frame one: the
//! door's time is the run seed's own (`SplitMix64`), the place has no
//! randomness, and the voice (`modes::tunnel_voice`) keeps its own order.
use crate::base::rng::SplitMix64;
use crate::prelude::*;
use crate::story::notebook::DOOR_NOTE;
use jahamun::{DOOR_X, Layout, NOTE_FROM_DOOR, Set};

/// Where your hands are given back: a few bays in, facing on in (feet, the
/// set's frame: the walkway at -x, into the tunnel is -z).
pub const START: [f64; 3] = [-0.85, 0.20, -16.0];
/// The far plane: the ring's reach (`jahamun::ring::VIEW`) and a little.
pub const FAR: f64 = 262.0;
/// The door's walking time: 60-120 s of moving, the run's own.
const CHANGE_SALT: u64 = 0x7A4E_D00E_51A7_0001;
pub const CHANGE_MIN: f64 = 60.0;
pub const CHANGE_MAX: f64 = 120.0;
/// The door changes only while every copy of it is out of sight: behind
/// you, or this far off where you look (a door in a wall seen edge-on).
const UNSEEN_AHEAD: f64 = 40.0;
const UNSEEN_BEHIND: f64 = 8.0;
/// From black to the tunnel, at New descent.
const FADE_UP_MS: f64 = 500.0;
/// How near the wall and along it E reaches the door, or her note.
const REACH_X: f64 = -0.9;
const REACH_DOOR: f64 = 0.75;
const REACH_NOTE: f64 = 0.45;

/// What E would act on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Near {
    Nothing,
    Door,
    Note,
}

/// The way in, while you are in it.
pub struct Tunnel {
    pub set: Set,
    /// your feet (the set's frame)
    pub feet: [f64; 3],
    /// seconds spent moving (the door's clock)
    pub walked: f64,
    /// metres walked into the tunnel (along -z, the ring's carries undone)
    pub along: f64,
    /// how far you have come back toward the arch since you were deepest
    pub back: f64,
    deepest: f64,
    /// walked seconds before the door may change
    pub change_after: f64,
    pub near: Near,
    pub note_taken: bool,
    /// the locked door tried (the voice's)
    pub tried: bool,
    step_acc: f64,
}

/// The walking time before the door changes, for the run `run_seed`.
pub fn change_after(run_seed: u32) -> f64 {
    SplitMix64::new(run_seed, CHANGE_SALT).range(CHANGE_MIN, CHANGE_MAX)
}

impl Tunnel {
    fn new(set: Set, run_seed: u32) -> Tunnel {
        Tunnel {
            set,
            feet: START,
            walked: 0.0,
            along: 0.0,
            back: 0.0,
            deepest: 0.0,
            change_after: change_after(run_seed),
            near: Near::Nothing,
            note_taken: false,
            tried: false,
            step_acc: 0.0,
        }
    }

    /// The nearest copy of the escape door (z, along the bore).
    pub fn nearest_door(&self) -> Option<f64> {
        let z = self.feet[2];
        self.set.doors().min_by(|a, b| (a - z).abs().total_cmp(&(b - z).abs()))
    }

    /// Whether no copy of the door can be seen from `eye` looking along
    /// `yaw`: each is behind (and not right beside you) or far off.
    pub fn door_unseen(&self, eye: Vec3, yaw: f64) -> bool {
        let (fx, fz) = (-yaw.dsin(), -yaw.dcos());
        self.set.doors().all(|z| {
            let (vx, vz) = (DOOR_X - eye.x, z - eye.z);
            let d = (vx * vx + vz * vz).sqrt().max(1e-6);
            let facing = (vx * fx + vz * fz) / d;
            if facing > -0.2 { d > UNSEEN_AHEAD } else { d > UNSEEN_BEHIND }
        })
    }
}

impl Game {
    /// Whether a run begun from the title goes through the way in: always,
    /// but for a run on a chosen seed or level (the tools' and the tests'
    /// runs begin on their floor), unless `prologue=1` or `prologue=0` says.
    pub fn way_in_wanted(&self) -> bool {
        let flags = self.parse_flag_params();
        match flags.get("prologue").map(|s| s.trim().to_string()).as_deref() {
            Some("1") => true,
            Some("0") => false,
            _ => !flags.contains_key("seed") && !flags.contains_key("level"),
        }
    }

    /// New descent through the way in: the run made (its first floor
    /// built behind, its card kept for the door; the title's own run, if
    /// the title shows its first floor), the tunnel, up out of black.
    pub fn new_descent_through_the_tunnel(&mut self) {
        // (a launch flag's seed or level makes another place than the title's)
        let flags = self.parse_flag_params();
        let flagged = flags.contains_key("seed") || flags.get("level").is_some_and(|l| l.trim() != "1");
        let same = !flagged && self.title_place.is_some_and(|(seed, level)| seed == self.run_seed && level == 1 && self.level_num == 1);
        let seed = same.then_some(self.run_seed);
        if self.menu.screen.is_some() {
            self.menu_fade = Some((self.menu.clone(), self.now_ms));
        }
        self.start_new_game_with(seed, false);
        self.ui.fade.set_style("background", "#000");
        self.ui.fade.set_opacity(1.0);
        self.enter_tunnel();
        self.schedule(FADE_UP_MS, |g| g.ui.fade.set_opacity(0.0));
    }

    /// Into the tunnel: the place built and shown, the floor behind it
    /// hidden, you a few bays in. False (and straight onto the floor, its
    /// card shown) if the place could not be built.
    pub fn enter_tunnel(&mut self) -> bool {
        let set = match Set::build(Layout::Ring) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("the way in: {e}");
                self.way_in_arrive();
                return false;
            }
        };
        // the first floor's line and its arrival are for the door
        self.msg_t = 0.0;
        self.ui.msg.set_opacity(0.0);
        self.asides_leave_floor();
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
        self.scene.add(&set.root);
        let t = Tunnel::new(set, self.run_seed);
        self.player.yaw = 0.0;
        self.player.pitch = 0.0;
        self.player.flashlight_on = false;
        self.player.pos.set(t.feet[0], EYE, t.feet[2]);
        self.tunnel = Some(Box::new(t));
        self.in_tunnel = true;
        if let Some(ss) = self.soundscape() {
            ss.define_zone("tunnel", crate::audio::soundscape::ReverbPreset::new(2.8, 0.42, 0.42, 0.025, 2));
            ss.set_zone("tunnel", 0.1);
        }
        self.update_tunnel(0.0);
        self.tunnel_look();
        self.want_warm_up = true;
        self.state = "playing".into();
        true
    }

    /// Out of the tunnel without going on (a new floor built, a load, the
    /// title): the place let go, the floor's world shown again.
    pub fn leave_tunnel_mode(&mut self) {
        if !self.in_tunnel {
            return;
        }
        self.in_tunnel = false;
        if let Some(t) = self.tunnel.take() {
            self.scene.remove(&t.set.root);
        }
        self.world.set_visible(true);
        self.player_light.light().distance = 7.0;
        self.player_light.set_intensity(2.4);
        self.amb_light.light().color.set_hex(0xa89a70);
        self.hemi_light.light().color.set_hex(0xd0c48c);
        self.release_pooled_lights();
        if let Some(ss) = self.soundscape() {
            ss.hum.silence();
        }
        self.ui.prompt.set_text("");
    }

    /// Through the open door: the Building's first floor, entered as a
    /// floor is entered through a door (`How::Door`: built, saved, its card).
    pub fn exit_tunnel(&mut self) {
        if !self.in_tunnel || self.state != "playing" {
            return;
        }
        self.state = "transition".into();
        self.sfx_door();
        self.ui.fade.set_style("background", "#000");
        self.ui.fade.set_opacity(1.0);
        self.schedule(1400.0, |g| {
            // (the card the run kept for the way in is the door's now)
            g.way_in_card = None;
            g.enter_level(1, crate::session::anchor::How::Door);
        });
    }

    /// E in the tunnel: the door tried, gone through, or her note taken.
    pub fn tunnel_interact(&mut self) {
        let Some(t) = self.tunnel.as_mut() else { return };
        match t.near {
            Near::Note => {
                t.note_taken = true;
                self.take_door_note();
            }
            Near::Door if t.set.door_changed() => self.exit_tunnel(),
            Near::Door => {
                t.tried = true;
                self.sfx_clank();
                self.show_msg("LOCKED", 1.2);
            }
            Near::Nothing => {}
        }
    }

    /// Her note by the door: into the notebook (J), before her pages, its
    /// line the thread. (Nothing is saved: the way in keeps no save.)
    fn take_door_note(&mut self) {
        self.notebook.insert(DOOR_NOTE.id.to_string());
        self.thread = DOOR_NOTE.thread.to_string();
        self.show_thread();
        self.sfx_paper_rustle();
        self.sfx_page_flip();
        let key = self.settings.keys(crate::session::input::Act::Journal).first().map_or_default(|k| crate::session::input::key_label(k));
        self.show_msg(&format!("Her note. {key} to read it."), 2.6);
        self.journal_newest = Some(crate::ui::hud::notebook_journal_id(0));
        if let Some(t) = self.tunnel.as_mut() {
            t.set.take_note();
        }
    }

    /// Once a frame in the tunnel: your movement on its ground, the ring's
    /// carry, the door's time, what E would act on, the camera, the
    /// render's fog and light, the sound.
    pub fn update_tunnel(&mut self, dt: f64) {
        let Some(mut t) = self.tunnel.take() else { return };
        let hands = !self.hands_taken();
        let (want_crouch, mx, mz, sprint_key) = if hands { self.modes_move_keys() } else { (false, 0.0, 0.0, false) };
        if mx == 0.0 && mz == 0.0 {
            self.input.run_latched = false;
        }
        self.player.crouch = want_crouch;
        let moving = mx != 0.0 || mz != 0.0;
        let sprinting = !self.player.crouch && sprint_key && self.sprint_ready() && moving;
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
        if moving && dt > 0.0 {
            let l = hypot2(mx, mz).max(1.0);
            let (s, c) = self.player.yaw.dsin_cos();
            // forward (-sin, -cos), right (cos, -sin)
            let dx = (-s * mz + c * mx) / l * speed * dt;
            let dz = (-c * mz - s * mx) / l * speed * dt;
            let before = t.feet;
            t.feet = t.set.walk.step(t.feet, dx, dz);
            let moved = before[2] - t.feet[2];
            if (t.feet[0] - before[0]).abs() + moved.abs() > 1e-9 {
                t.walked += dt;
            }
            t.along += moved;
            if t.along > t.deepest {
                t.deepest = t.along;
            }
            t.back = t.deepest - t.along;
            if let Some(shift) = t.set.ring.step(t.feet[2]) {
                t.feet[2] += shift;
            }
            self.player.bob_phase += dt * speed * 2.1;
            let amp = if sprinting {
                0.075
            } else if self.player.crouch {
                0.025
            } else {
                0.045
            };
            self.player.bob_amp = lerp(self.player.bob_amp, amp, dt * 6.0);
        } else {
            self.player.bob_amp = lerp(self.player.bob_amp, 0.0, dt * 6.0);
        }
        // play's camera (a cutscene's is mixed over it after)
        let bob = self.player.bob_phase.dsin().abs() * self.player.bob_amp * self.settings.bob;
        let eye = if self.player.crouch { CROUCH_EYE } else { EYE };
        self.camera.position().set(t.feet[0], t.feet[1] + eye + bob, t.feet[2]);
        self.camera.rotation().set(self.player.pitch, self.player.yaw, 0);
        self.player.pos.set(t.feet[0], EYE, t.feet[2]);
        // the door's time: walking, then a moment no copy of it is in sight
        if !t.set.door_changed() && t.walked >= t.change_after {
            let eye_at = Vec3::new(t.feet[0], t.feet[1] + EYE, t.feet[2]);
            if t.door_unseen(eye_at, self.player.yaw) {
                t.set.set_door_changed(true);
            }
        }
        // what E would act on
        t.near = Near::Nothing;
        if hands
            && t.feet[0] < REACH_X
            && let Some(dz) = t.nearest_door()
        {
            let z = t.feet[2];
            if t.set.door_changed() && !t.note_taken && (dz + NOTE_FROM_DOOR - z).abs() < REACH_NOTE {
                t.near = Near::Note;
            } else if (dz - z).abs() < REACH_DOOR {
                t.near = Near::Door;
            }
        }
        self.ui.prompt.set_text(match t.near {
            Near::Note => "[E] TAKE HER NOTE",
            Near::Door if t.set.door_changed() => "[E] GO THROUGH",
            Near::Door => "[E] TRY THE DOOR",
            Near::Nothing => "",
        });
        // the place lights itself: no lamp of yours, no light tied to you
        self.player.flashlight_on = false;
        self.flashlight.set_intensity(0.0);
        self.player_light.set_intensity(0.0);
        self.amb_light.set_intensity(0.0);
        self.hemi_light.set_intensity(0.0);
        // the render's fog and light: the place's (its light map is the fog by place)
        let gl = t.set.globals();
        self.background = gl.background;
        if let Some((c, d)) = gl.fog {
            self.fog_color = c;
            self.fog_density = d;
        }
        self.ll_on = 1.0;
        self.ll_map = gl.ll.map.clone();
        self.ll_grid = gl.ll.grid;
        self.ll_scale = gl.ll.scale;
        self.ll_amb_d = gl.ll.amb_d;
        self.ll_sky_d = gl.ll.sky_d;
        self.ll_gnd_d = gl.ll.gnd_d;
        self.ll_fog_lit = gl.ll.fog_lit;
        self.ll_fog_ratio = gl.ll.fog_ratio;
        // the sound: steps on the paint; the bars' hum from the nearest bar
        if let (Some(ss), true) = (self.soundscape(), self.ac.is_some()) {
            let cp = self.camera.pos();
            let fwd = self.player.fwd();
            ss.set_listener(cp.x, cp.y, cp.z, fwd.x, self.player.pitch.dsin(), fwd.z);
            ss.set_zone("tunnel", 1.0);
            let bar_z = (t.feet[2] / 2.0).round() * 2.0 - 1.0;
            ss.hum.update(dt, 0.05, 1.05, bar_z, 0.55);
            if moving {
                t.step_acc += dt * speed * 2.1;
                if t.step_acc > (if sprinting { 3.4 } else { 2.6 }) {
                    t.step_acc = 0.0;
                    if !self.player.crouch {
                        ss.footstep("concrete", if sprinting { 1.0 } else { 0.6 }, None);
                    }
                }
            }
        }
        self.tunnel = Some(t);
    }

    /// After the camera is set (play's or a cutscene's): the place's look
    /// for the eye (the highlights, the glass, the glows, the other bore).
    pub fn tunnel_look(&mut self) {
        let cp = self.camera.pos();
        if let Some(t) = self.tunnel.as_mut() {
            t.set.update(cp);
        }
        self.camera.camera().far = FAR;
    }

    /// (tools) The way in, you at `feet` facing `yaw`, the door changed or
    /// not (`--screen tunnel`, `tunnel-door`, `tunnel-open`).
    pub fn debug_tunnel(&mut self, feet: [f64; 3], yaw: f64, changed: bool) {
        if !self.in_tunnel && !self.enter_tunnel() {
            return;
        }
        if let Some(t) = self.tunnel.as_mut() {
            t.feet = feet;
            t.set.set_door_changed(changed);
        }
        self.player.yaw = yaw;
        self.player.pitch = 0.0;
        self.update_tunnel(0.0);
        self.tunnel_look();
    }
}
