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
        if self.state == "playing" {
            self.update_input(dt);
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
        } else if self.state == "menu" {
            // the title is a view of the lobby, live: its lights, its dust, its sound
            let perf_t = self.perf_t;
            self.update_title_view(dt);
            self.update_lights(dt, perf_t);
            self.update_dust(dt);
            self.update_grain(dt);
            if self.ac.is_some() {
                self.update_ambient(dt);
                self.update_soundscape(dt);
            }
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
    /// Key down; `code` names the physical key (`KeyW`, `Space`, ...) or a
    /// mouse button while the pointer is locked (`Mouse0`..`Mouse2`). A
    /// held key fires its action once: the OS key repeat only keeps it held.
    pub fn key_down(&mut self, code: &str, repeat: bool) {
        let fresh = self.keys.insert(code.to_string());
        if repeat && !fresh {
            return;
        }
        self.on_key_down(code);
    }
    /// Key up.
    pub fn key_up(&mut self, code: &str) {
        self.keys.remove(code);
        self.on_key_up(code);
    }
    /// Mouse motion while pointer-locked (relative dx / dy).
    pub fn mouse_move(&mut self, dx: f64, dy: f64) {
        if !self.pointer_locked {
            return;
        }
        let k = 0.0022 * self.settings.look;
        let dy = if self.settings.invert_y { -dy } else { dy };
        self.player.yaw -= dx * k;
        self.player.pitch = clamp(self.player.pitch - dy * k, -1.45, 1.45);
        self.mouse_dx += dx;
        self.mouse_dy += dy;
    }
    /// The mouse wheel (`d` > 0: towards you): turns the notebook's pages.
    pub fn wheel(&mut self, d: f64) {
        if !self.journal_open || d.abs() < 1.0 {
            return;
        }
        if d > 0.0 {
            self.journal_next();
        } else {
            self.journal_prev();
        }
    }
    /// A click in the window. `target` is the element under the cursor (the HUD's
    /// hit test): a menu line ('menu:continue', …), 'journal-prev',
    /// 'journal-next', 'overlay' (anywhere on a card) or '' (the canvas).
    pub fn click(&mut self, target: &str) {
        if self.live_menu().is_some() {
            // Controls waiting for a key: a click is the left button
            if let Some(a) = self.menu.capture.take() {
                self.settings.rebind(a, "Mouse0");
                self.save_settings();
                return self.refresh_menu();
            }
            if self.menu.items.iter().any(|i| i.id == target) {
                return self.menu_choose(target);
            }
        }
        match target {
            "journal-prev" => self.journal_prev(),
            "journal-next" => self.journal_next(),
            "overlay" => {
                if self.state == "paused" {
                    self.resume();
                } else if self.state == "dead" {
                    self.restart();
                } else if self.state == "ending" {
                    self.end_ending();
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
        let was = std::mem::replace(&mut self.pointer_locked, locked);
        let blurred = std::mem::take(&mut self.input.blurred);
        // (only a change counts: losing focus releases the pointer here and
        // the window reports it once more)
        if !locked && was && self.state == "playing" {
            // (Settings can leave the world running when the window is left)
            if self.journal_open || (blurred && !self.settings.focus_pause) {
                return;
            }
            self.pause();
        }
    }
    /// The window lost focus: held keys are dropped and pointer lock is released.
    pub fn blur(&mut self) {
        self.keys.clear();
        self.input.crouch_latched = false;
        self.input.run_latched = false;
        self.input.blurred = true;
        if self.pointer_locked {
            self.pointer_lock_changed(false);
        }
    }
    /// `audio_init()` — the audio module provides it; this forwards
    pub fn audio_init_hook(&mut self) {
        self.audio_init();
    }
}
