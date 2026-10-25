//! The main loop: start-up (`boot`), `frame()`, and the window's input
//! handlers (key down / up, mouse look, clicks on the overlay and its
//! buttons, pointer-lock changes, focus loss).
use crate::prelude::*;

impl Game {
    /// Start-up: every module's init, in order, then `init_level(1)`
    /// behind the title overlay.
    pub fn boot(&mut self) {
        self.init_textures_hook();
        let dust = self.dust_sprite_hook();
        self.init_world();
        self.init_anomalies();
        self.init_render(&dust);
        self.init_entities();
        self.init_ambient();
        self.init_player();
        self.init_hud();
        self.init_modes(); // the Stairwell (built once at start-up)
        self.init_flow();
        self.init_level(1);
    }
    /// the textures section (merged from the textures module)
    pub fn init_textures_hook(&mut self) {
        self.tex = crate::visuals::textures::Textures::new();
    }
    /// `dust_sprite`
    pub fn dust_sprite_hook(&self) -> Tex {
        self.tex.dust_sprite.clone()
    }
    /// One simulation step (rendering is the app's job).
    /// `real_dt` is the wall-clock time since the last frame.
    pub fn frame(&mut self, real_dt: f64) {
        self.run_scheduled();
        if self.sim_frozen {
            return;
        }
        let real_dt = real_dt.min(0.05);
        let dt = if self.fixed_step_dt != 0.0 { self.fixed_step_dt } else { real_dt };
        self.perf_t += dt;
        self.fps_frames += 1.0;
        self.fps_t += dt;
        if self.fps_t >= 1.0 {
            self.fps = self.fps_frames / self.fps_t;
            self.fps_frames = 0.0;
            self.fps_t = 0.0;
        }
        if self.state == "playing" && self.in_stairwell {
            self.update_stairwell(dt);
            self.update_hud(dt);
            self.update_grain(dt);
        } else if self.state == "playing" && self.in_loop {
            self.update_loop(dt);
            self.update_hud(dt);
            self.update_grain(dt);
        } else if self.state == "playing" {
            let perf_t = self.perf_t;
            self.update_player(dt);
            self.update_entities(dt, perf_t);
            self.update_director(dt);
            self.update_scp(dt);
            self.update_prop_cull(dt);
            self.update_lights(dt, perf_t);
            self.update_hud(dt);
            self.update_dust(dt);
            self.update_grain(dt);
            self.update_reality_glitch(dt);
            self.update_flicker_wave(dt);
            let moving = self.player.bob_amp > 0.02;
            self.update_ghost_footsteps(dt, moving);
            self.update_guide_arrows(dt, perf_t);
            self.update_vacuum_bot(dt);
            self.update_broken_elevator(dt, perf_t);
            self.update_sanity_effects(dt);
            self.update_ambient(dt);
            self.update_soundscape(dt);
            self.update_debug();
        } else if self.state == "dead" {
            let y = EYE + (self.perf_t * 30.0).sin() * 0.01;
            self.camera.position().y = y;
            self.update_death_cam(dt);
            for i in 0..self.entities.len() {
                self.update_face(i);
            }
            self.update_grain(dt);
        }
        self.run_animators_frame();
        self.sync_far_to_fog();
        if self.fixed_step_dt == 0.0 {
            self.quality_governor(real_dt);
        }
        self.mouse_dx = 0.0;
        self.mouse_dy = 0.0;
    }

    /// The animators run inside `update_hud` (see ui/hud.rs); this is a
    /// no-op hook kept for the main loop's order.
    fn run_animators_frame(&mut self) {}

    // ------------------------------------------------------------ input
    /// Key down; `code` names the physical key (`KeyW`, `Space`, ...).
    pub fn key_down(&mut self, code: &str, repeat: bool) {
        // `repeat` is deliberately ignored: a held key re-fires its action with the OS key repeat
        let _ = repeat;
        self.keys.insert(code.to_string());
        self.on_key_down(code);
    }
    /// Key up.
    pub fn key_up(&mut self, code: &str) {
        self.keys.remove(code);
    }
    /// Mouse motion while pointer-locked (relative dx / dy).
    pub fn mouse_move(&mut self, dx: f64, dy: f64) {
        if !self.pointer_locked {
            return;
        }
        self.player.yaw -= dx * 0.0022;
        self.player.pitch = clamp(self.player.pitch - dy * 0.0022, -1.45, 1.45);
        self.mouse_dx += dx;
        self.mouse_dy += dy;
    }
    /// A click in the window. `target` is the element under the cursor (the HUD's
    /// hit test): 'btn-new', 'btn-continue', 'journal-prev', 'journal-next',
    /// 'overlay' (anywhere on the visible menu overlay) or '' (the canvas).
    pub fn click(&mut self, target: &str) {
        match target {
            "btn-new" => self.start_new_game(),
            "btn-continue" => self.continue_game(),
            "journal-prev" => self.journal_prev(),
            "journal-next" => self.journal_next(),
            "overlay" => {
                if self.state == "paused" {
                    self.audio_init_if_needed();
                    self.state = "playing".into();
                    self.hide_overlay();
                    self.want_pointer_lock = Some(true);
                } else if self.state == "dead" {
                    self.restart();
                }
            }
            _ => {
                // the canvas itself: re-lock the pointer while playing
                if self.state == "playing" && !self.pointer_locked && !self.journal_open {
                    self.want_pointer_lock = Some(true);
                }
            }
        }
    }
    /// Pointer lock was gained or lost.
    pub fn pointer_lock_changed(&mut self, locked: bool) {
        self.pointer_locked = locked;
        if !locked && self.state == "playing" {
            if self.journal_open {
                return;
            }
            self.state = "paused".into();
            let sub = self.pick_text("pauseSub");
            self.show_overlay("PAUSED", &sub, "CLICK TO RESUME");
        }
    }
    /// The window lost focus: held keys are dropped and pointer lock is released.
    pub fn blur(&mut self) {
        self.keys.clear();
        if self.pointer_locked {
            self.pointer_lock_changed(false);
        }
    }
    fn audio_init_if_needed(&mut self) {
        self.audio_init_hook();
    }
    /// `audio_init()` — the audio module provides it; this forwards
    pub fn audio_init_hook(&mut self) {
        self.audio_init();
    }
}
