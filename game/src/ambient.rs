//! backrooms.js "soundscape driver" (62870–66024): floorSurfaceAt, zone
//! entries, updateSoundscape, updateAmbient, updateSanityEffects; plus the
//! flow section's ambience loop / reality glitch / flicker wave / ghost
//! footsteps (70061–70145).
//! CONTRACT: the signatures below are called from other modules; keep them.
//!
//! `updateAmbient` is split into one private function per event family, in
//! the JS order (so the per-frame `Math.random()` draws stay in order):
//!
//! | step | function | file |
//! |---|---|---|
//! | mix factors (`_sanityMag`, `_eventTimerMul`, `_eventGain`, `_levelDepth`) | `amb_mix` | here |
//! | R118/R120 rain-window streaks + rain hiss | `amb_rain_windows` | here |
//! | R117 ripples, R131 scuffs, R119 splash / R121 thunder flash decay | `amb_fx_decay` | here |
//! | R99 room-tone lerp | `amb_room_tone` | here |
//! | per-weather beds (wet / still / derelict / electric / haunted / normal) | `amb_weather_events` | ambient_weather.rs |
//! | cross-weather rare events (knock … clock skip) | `amb_rare_events` | ambient_weather.rs |
//! | prop sounds R272–R584 (fountain … smoke detector) | `amb_props_fixtures` | ambient_props.rs |
//! | outdoor ghosts + R608–R672 (bird … teapot) | `amb_props_outdoor` | ambient_props.rs |
//! | R678–R729 household props (cookie tin … turntable) | `amb_props_household` | ambient_props2.rs |
//! | R732–R767 parlour props (chalkboard … hobby horse) | `amb_props_parlour` | ambient_props2.rs |
//! | R542–R564 + R273 (pay phone … plant, motion sensor) | `amb_props_lobby` | ambient_props3.rs |
//! | R275–R325 rare fixtures (speaker … broken phone) | `amb_props_rare_fixtures` | ambient_props3.rs |
//! | R237 reality-shift pitch bend, R102 surge decay | `amb_shift_bend` | ambient_weather.rs |
use crate::lighting::{RF_DARK, RF_MOSS};
use crate::prelude::*;
use crate::records::*;
use crate::tables::*;

/// `FOG_BASE` (line 70100)
pub const FOG_BASE: f64 = 0.052;
/// `metronomePositions[i].period` when the placer did not record one: the
/// middle of the placer's `1.2 + Math.random() * 0.6` swing period.
pub const METRONOME_PERIOD_DEFAULT: f64 = 1.5;
/// `chairPositions[i].period` when the placer did not record one: the middle
/// of the placer's `5 + Math.random() * 3` rock period.
pub const CHAIR_PERIOD_DEFAULT: f64 = 6.5;
/// `glitchMsgs` (updateRealityGlitch)
pub const GLITCH_MSGS: [&str; 8] = ["HELLO", "YOU ARE HERE", "YOU ARE STILL HERE", "AGAIN", "I'M SORRY", "GO BACK", "IT KNOWS YOUR NAME", "ALMOST"];
/// the on-screen whispers that can accompany a hallucination (updateSanityEffects)
pub const HALLUC_WHISPERS: [&str; 6] = ["IT SAW YOU", "DO NOT TURN", "YOU ARE NOT ALONE", "HELLO AGAIN", "STAY VERY STILL", "I REMEMBER YOU"];
/// `RIPPLE_LIFE` (updateAmbient)
const RIPPLE_LIFE: f64 = 1.5;
/// `SCUFF_LIFE` (updateAmbient)
const SCUFF_LIFE: f64 = 0.7;

/// The per-frame mix factors at the top of `updateAmbient`.
#[derive(Clone, Copy, Debug)]
pub(crate) struct AmbMix {
    /// `_sanityMag` — 0 at ≥ 30 % sanity, 1 at 0 %
    pub sanity_mag: f64,
    /// `_eventTimerMul` — cadence shrink (sanity × descent)
    pub timer_mul: f64,
    /// `_eventGain` — sanity × descent × stamina muffle × cry muffle
    pub gain: f64,
    /// `_levelDepth` — `levelDepthMul()`
    pub depth: f64,
}

/// `list[Math.floor(Math.random() * list.length)]` — one draw; (index, entry)
#[inline]
pub(crate) fn pick_pos(list: &[Vec3]) -> (usize, Vec3) { let i = pick_index(list.len()); (i, list[i]) }

/// `(Math.random() < 0.5 ? -1 : 1) * (a + Math.random() * b)` — two draws
#[inline]
pub(crate) fn rand_side(a: f64, b: f64) -> f64 { let s = if rnd() < 0.5 { -1.0 } else { 1.0 }; s * (a + rnd() * b) }

impl Game {
    // ------------------------------------------------------------ helpers
    /// `player.yaw || 0`
    #[inline]
    pub(crate) fn yaw0(&self) -> f64 { let y = self.player.yaw; if y.is_nan() { 0.0 } else { y } }

    /// The prop-sound geometry every `*Positions` event repeats:
    /// `side = dx·cos(yaw) + dz·(−sin(yaw))`, `pan = clamp(side·k, −lim, lim)`,
    /// `dn = min(1, |d| / div)`. Returns (pan, dn).
    #[inline]
    pub(crate) fn prop_pan(&self, p: Vec3, k: f64, lim: f64, div: f64) -> (f64, f64) {
        let dx = p.x - self.player.pos.x;
        let dz = p.z - self.player.pos.z;
        let yaw = self.yaw0();
        let (rx, rz) = (yaw.cos(), -yaw.sin());
        let side = dx * rx + dz * rz;
        let dn = (1f64).min((dx * dx + dz * dz).sqrt() / div);
        (clamp(side * k, -lim, lim), dn)
    }
    /// squared horizontal distance from the player
    #[inline]
    pub(crate) fn prop_d2(&self, p: Vec3) -> f64 {
        let dx = p.x - self.player.pos.x;
        let dz = p.z - self.player.pos.z;
        dx * dx + dz * dz
    }
    /// `scuffs.find(sp => sp.t <= 0)` dressed as a short spark / splash / bubble flash
    pub(crate) fn scuff_flash(&mut self, hex: u32, x: f64, y: f64, z: f64, opacity: f64, scale: f64, life: f64) {
        let Some(i) = self.scuffs.iter().position(|s| s.t <= 0.0) else { return };
        let o = self.scuffs[i].obj.clone();
        if let Some(m) = o.try_mat() { m.m().color.set_hex(hex); }
        o.set_pos(x, y, z);
        o.set_visible(true);
        if let Some(m) = o.try_mat() { m.set_opacity(opacity); }
        o.scale().set_scalar(scale);
        self.scuffs[i].t = life;
    }
    /// `ripples.find(r => r.t <= 0)` placed at (x, 0.015, z); false when the pool is busy
    pub(crate) fn ripple_at(&mut self, x: f64, z: f64, opacity: f64, scale: f64, life: f64) -> bool {
        let Some(i) = self.ripples.iter().position(|r| r.t <= 0.0) else { return false };
        let o = self.ripples[i].obj.clone();
        o.set_pos(x, 0.015, z);
        o.set_visible(true);
        if let Some(m) = o.try_mat() { m.set_opacity(opacity); }
        o.scale().set(scale, scale, 1);
        self.ripples[i].t = life;
        true
    }

    // ------------------------------------------------------------ module load
    /// module-load work of the soundscape driver and of the flow section's
    /// ambience loop / glitch / flicker-wave timers (lines 62882–62885,
    /// 70061–70128): `ambienceLoop()`, `glitchNextT`, `flickerWaveNextT`.
    pub fn init_ambient(&mut self) {
        // line 62882 / 62884
        self.hummer_t = 0.0;
        self.hum_panel = None;
        self.last_zone_arch = None;
        self.zone_toast_t = 0.0;
        self.zones_seen.clear();
        // line 70066
        self.ambience_loop();
        // line 70070
        self.glitch_t = 0.0;
        self.glitch_next_t = 60.0 + rnd() * 90.0;
        // line 70105
        self.flicker_wave_next_t = 45.0 + rnd() * 60.0;
        // line 70128
        self.ghost_step_t = 0.0;
    }

    // line 62872
    /// `floorSurfaceAt(px, pz)` — what the floor under a point is made of, for footsteps.
    pub fn floor_surface_at(&self, px: f64, pz: f64) -> &'static str {
        let z = self.zone_at_world(px, pz).and_then(|zi| self.zone_fields(zi));
        let kind = z.as_ref().map(|z| z.floor_mat).unwrap_or("carpet");
        if kind == "carpet" {
            let info = self.region_at(self.tx(px), self.tz(pz));
            let wet = (info.grime >= 2 || (info.flags & RF_MOSS) != 0) || self.weather_key() == "wet";
            return if wet { "wetcarpet" } else { "carpet" };
        }
        kind   // concrete | tile | office
    }

    // line 62886
    /// `noteZoneEntry(Z)` — a quiet caption the first time you walk into a new kind of place
    fn note_zone_entry(&mut self, z: Option<crate::xstubs_amb::ZoneFields>) {
        self.zone_toast_t -= 1.0 / 60.0;
        let Some(z) = z else { return };
        if self.last_zone_arch.as_deref() == Some(z.arch) { return; }
        self.last_zone_arch = Some(z.arch.to_string());
        if self.zones_seen.contains(z.arch) || self.zone_toast_t > 0.0 { return; }
        self.zones_seen.insert(z.arch.to_string());
        self.zone_toast_t = 8.0;
        self.show_msg(&format!("{} — {}", z.title, z.lore), 2.4);
    }

    // line 62895
    /// `updateSoundscape(dt)` — listener, reverb zone, zone captions, the ballast hum
    pub fn update_soundscape(&mut self, dt: f64) {
        let Some(ss) = self.soundscape() else { return };
        let fwd = self.player.fwd();
        let cp = self.camera.pos();
        ss.set_listener(cp.x, cp.y, cp.z, fwd.x, self.player.pitch.sin(), fwd.z);
        let (x, z) = (self.tx(self.player.pos.x), self.tz(self.player.pos.z));
        let dark = self.region_flags_at(x, z) & RF_DARK != 0;
        // Blackout zones are silent in the lore: no hum, almost no room
        let zf = self.zone_at_world(self.player.pos.x, self.player.pos.z).and_then(|zi| self.zone_fields(zi));
        ss.set_zone(if dark { "silent" } else { zf.as_ref().map(|z| z.arch).unwrap_or("lobby") }, 1.2);
        self.note_zone_entry(zf);
        // the hum sits at the nearest live fixture; re-pick a few times a second
        self.hummer_t -= dt;
        if self.hummer_t <= 0.0 {
            self.hummer_t = 0.25;
            let mut best: Option<usize> = None;
            let mut bd = 1e9;
            for (i, p) in self.panels.iter().enumerate() {
                if p.bright < 0.3 { continue; }
                let d = (p.wx - self.player.pos.x).powi(2) + (p.wz - self.player.pos.z).powi(2);
                if d < bd { bd = d; best = Some(i); }
            }
            self.hum_panel = best.map(|p| HumPanel { p, d: bd.sqrt() });
        }
        let hp = match self.hum_panel { Some(hp) if !dark && hp.p < self.panels.len() => hp, _ => { ss.hum.silence(); return; } };
        let lit = self.region_light_at(x, z);
        let strength = clamp((0.35 + 0.65 * lit) * clamp(1.4 - hp.d / 14.0, 0.0, 1.0), 0.0, 1.0);
        let p = &self.panels[hp.p];
        ss.hum.update(dt, p.wx, (if p.y != 0.0 { p.y } else { WALL_H }) - 0.1, p.wz, strength);
    }

    // line 62922
    /// `updateAmbient(dt)` — every ambient event, cadence timer and prop sound
    pub fn update_ambient(&mut self, dt: f64) {
        if self.state != "playing" || self.ac.is_none() { return; }
        let wk = self.weather_key();
        let m = self.amb_mix();
        self.amb_rain_windows(dt, wk);
        self.amb_fx_decay(dt);
        self.amb_room_tone(dt, wk);
        self.amb_weather_events(dt, wk, &m);
        self.amb_rare_events(dt, wk, &m);
        self.amb_props_fixtures(dt, &m);
        self.amb_props_outdoor(dt, &m);
        self.amb_props_household(dt, &m);
        self.amb_props_parlour(dt, &m);
        self.amb_props_lobby(dt, &m);
        self.amb_props_rare_fixtures(dt, &m);
        self.amb_shift_bend(dt);
        // R102/R104 electric surge dim countdown (drives panel dim). Outside the
        // electric branch so a mid-surge weather change still decays the dim.
        if self.electric_surge_t > 0.0 { self.electric_surge_t = (0.0f64).max(self.electric_surge_t - dt); }
    }

    /// The mix factors (lines 62929–62960).
    fn amb_mix(&self) -> AmbMix {
        // R143 sanity ambient boost: below 30% sanity the next-cooldown values
        // shrink (2/3 of nominal at 0%); R144 also boosts event gains.
        let sanity_mag = { let s = self.player.sanity / SAN_MAX; if s >= 0.30 { 0.0 } else { 1.0 - s / 0.30 } };
        // R143/R174: sanity timer shrink + descent timer shrink compose.
        let timer_mul = (1.0 / (1.0 + sanity_mag * 0.5)) * (1.0 / self.descent_boost_mul(0.15));
        // R191 exhaustion muffle: below 20% stamina, up to 15% quieter.
        let stam_mag = { let s = self.player.stamina / 100.0; if s >= 0.20 { 0.0 } else { 1.0 - s / 0.20 } };
        let stam_gain = 1.0 - stam_mag * 0.15;
        // R196 cry-dim audio pair: up to 25% muffle at cryDimLerp = 1.
        let cry_gain = 1.0 - 0.25 * self.cry_dim_lerp;
        // R144/R172/R191/R196 compose multiplicatively.
        let gain = (1.0 + sanity_mag * 0.35) * self.descent_boost_mul(0.25) * stam_gain * cry_gain;
        // R159 level-depth multiplier.
        let depth = self.level_depth_mul();
        AmbMix { sanity_mag, timer_mul, gain, depth }
    }

    /// R118 rain streak overlays + R120 rain hiss (lines 62961–62995).
    fn amb_rain_windows(&mut self, dt: f64, wk: &str) {
        let rain_target = if wk == "wet" { 0.55 } else { 0.0 };
        let mut rain_score: f64 = 0.0;   // R120 max over all rain windows
        let yaw = self.yaw0();
        let (cam_fx, cam_fz) = (-yaw.sin(), -yaw.cos());
        let windows = self.rain_windows.clone();
        for w in &windows {
            let Some(mat) = w.try_mat() else { continue };
            let cur = mat.opacity_v();
            mat.set_opacity(cur + (rain_target - cur) * (1f64).min(dt * 2.0));
            if wk == "wet" {
                if let Some(t) = mat.map_v() { let y = t.t().offset.y; t.t().offset.y = (y - dt * 0.6) % 1.0; }
                // R120: score = clamp(facing dot) × distance falloff.
                let wp = w.world_position();
                let (dx, dz) = (wp.x - self.player.pos.x, wp.z - self.player.pos.z);
                let d = (dx * dx + dz * dz).sqrt();
                if d < 12.0 {
                    let dot = (dx * cam_fx + dz * cam_fz) / (if d != 0.0 { d } else { 1.0 });   // -1..1
                    if dot > 0.0 {
                        let near = 1.0 - d / 12.0;   // 0..1
                        rain_score = rain_score.max(dot * near);
                    }
                }
            }
            w.set_visible(mat.opacity_v() > 0.01);
        }
        // R120 rain hiss gain: only active on wet weather + facing a window.
        self.rain_score_current = rain_score;   // R124 expose to K panel
        if let Some(n) = &self.audio.rain_hiss_gain {
            let target = if wk == "wet" { 0.025 * rain_score } else { 0.0 };
            let v = n.gain().value();
            n.gain().set_value(v + (target - v) * (1f64).min(dt * 3.0));
        }
    }

    /// R117 ripple fade, R131 scuff decay, R119/R120 splash flash and R121
    /// thunder flash decay (lines 62996–63042).
    fn amb_fx_decay(&mut self, dt: f64) {
        // R117 wet-ripple fade (every weather, so a weather change still fades them)
        for r in self.ripples.iter_mut() {
            if r.t <= 0.0 { continue; }
            r.t -= dt;
            let mat = r.obj.try_mat();
            if r.t <= 0.0 { r.obj.set_visible(false); if let Some(m) = &mat { m.set_opacity(0.0); } continue; }
            let p = r.t / RIPPLE_LIFE;   // 1 → 0
            if let Some(m) = &mat { m.set_opacity(0.55 * p); }
            let g = 1.0 + (1.0 - p) * 1.6;   // grow 1× → 2.6× as it fades
            r.obj.scale().set(g, g, 1);
        }
        // R131 dust-scuff decay (derelict footsteps). Rises + fades.
        for s in self.scuffs.iter_mut() {
            if s.t <= 0.0 { continue; }
            s.t -= dt;
            let mat = s.obj.try_mat();
            if s.t <= 0.0 {
                s.obj.set_visible(false);
                if let Some(m) = &mat { m.set_opacity(0.0); }
                continue;
            }
            let p = s.t / SCUFF_LIFE;
            if let Some(m) = &mat { m.set_opacity(0.55 * p); }
            s.obj.scale().set_scalar(0.35 + (1.0 - p) * 0.15);
            s.obj.position().y += dt * 0.25;
        }
        // R119/R120 splash flash decay — t² curve
        if self.splash_flash_t > 0.0 {
            self.splash_flash_t -= dt;
            if self.splash_flash_t <= 0.0 {
                self.splash_flash_light.set_intensity(0.0);
            } else {
                let p = self.splash_flash_t / 0.30;
                self.splash_flash_light.set_intensity(self.splash_flash_peak * p * p);
            }
        }
        // R121 thunder flash decay — t² curve, 0.9 s life, 0.35 peak
        if self.thunder_flash_t > 0.0 {
            self.thunder_flash_t -= dt;
            if self.thunder_flash_t <= 0.0 {
                self.thunder_flash_light.set_intensity(0.0);
            } else {
                let p = self.thunder_flash_t / 0.9;
                self.thunder_flash_light.set_intensity(0.35 * p * p);
            }
        }
    }

    /// R99 room-tone lerp toward the per-weather target (lines 63043–63052).
    fn amb_room_tone(&mut self, dt: f64, wk: &str) {
        let (Some(g), Some(lp)) = (self.audio.room_tone_gain.clone(), self.audio.room_tone_lp.clone()) else { return };
        let rt = room_tone_by_weather(wk).or(room_tone_by_weather("normal")).unwrap();
        let k = (1f64).min(dt * 1.5);
        self.room_tone_cur_gain += (rt.gain - self.room_tone_cur_gain) * k;
        self.room_tone_cur_lp += (rt.lp - self.room_tone_cur_lp) * k;
        g.gain().set_value(self.room_tone_cur_gain);
        lp.frequency().set_value(self.room_tone_cur_lp);
    }

    // line 65839
    /// `updateSanityEffects(dt)` — sanity-crossing chimes and flashes, the
    /// critical-sanity heartbeat, whispers and hallucinated silhouettes
    pub fn update_sanity_effects(&mut self, dt: f64) {
        let s = self.player.sanity / SAN_MAX;
        self.sanity_crossing_chimes(s);
        if s >= 0.28 || self.state != "playing" {
            self.crit_sanity_accum = 0.0;
            self.sanity_beat_t = 0.0;
            self.sanity_whisper_t = 0.0;
            // hide any lingering hallucination when we recover
            if self.halluc_shape.visible() {
                self.halluc_shape.set_visible(false);
                if let Some(m) = self.halluc_shape.try_mat() { m.set_opacity(0.0); }
            }
            return;
        }
        self.crit_sanity_accum += dt;
        self.sanity_heartbeat(dt, s);
        // whispered hallucinations only after 10 s of continuous crit-sanity
        if self.crit_sanity_accum > 10.0 {
            self.sanity_whisper_t -= dt;
            if self.sanity_whisper_t <= 0.0 {
                self.sanity_whisper_t = rand2(6.0, 14.0);
                self.sanity_whisper();
            }
        }
        self.halluc_animate(dt);
    }

    /// R147–R152 recovery / impaired / breaking / mini chimes and their flash
    /// (lines 65840–65905).
    fn sanity_crossing_chimes(&mut self, s: f64) {
        let wk = self.weather_key();
        let playing = self.state == "playing";
        let bat_gain = |g: &Game| 0.7 + 0.3 * clamp(g.player.battery / 100.0, 0.0, 1.0);
        // Recovery chime on the upward crossing back above 55%; R149 impaired
        // chime on the downward crossing.
        let is_impaired = s < 0.55 && playing;
        if self.sanity_was_impaired && !is_impaired && playing {
            if self.ac.is_some() {
                self.sfx_chime();
                let mel = recovery_melody_by_weather(wk).or(recovery_melody_by_weather("normal")).unwrap();
                let mel_gain = bat_gain(self);
                let mel_depth = self.chime_depth_mul();   // R163: sanity chords treated as chime-tier
                self.blip(mel.root * mel_depth, 0.55, "sine", 0.055 * mel_gain, None, 0.0, 0.0);
                self.blip(mel.up * mel_depth, 0.65, "sine", 0.040 * mel_gain, None, 0.30, 0.0);
                self.blip(mel.top * mel_depth, 0.75, "sine", 0.028 * mel_gain, None, 0.55, 0.0);
                // R151 recovery visual flash: brief soft green-white pulse (relief).
                self.chime_flash_t = 0.9; self.chime_flash_life = 0.9; self.chime_flash_peak = 0.18; self.chime_flash_color = Some("rgba(200, 240, 200".into());
                self.sanity_recovery_tally += 1.0;
            }
        } else if !self.sanity_was_impaired && is_impaired {
            if self.ac.is_some() {
                // R149/R150 downward crossing: 3-note descending motif, battery-scaled.
                let imp = impaired_melody_by_weather(wk).or(impaired_melody_by_weather("normal")).unwrap();
                let imp_gain = bat_gain(self);
                let imp_depth = self.chime_depth_mul();
                self.blip(imp.root * imp_depth, 0.45, "sine", 0.040 * imp_gain, None, 0.0, 0.0);
                self.blip(imp.down * imp_depth, 0.60, "sine", 0.030 * imp_gain, None, 0.25, 0.0);
                self.blip(imp.bottom * imp_depth, 0.75, "sine", 0.022 * imp_gain, None, 0.50, 0.0);
                // R151 impaired visual flash: muted violet pulse (dread creeping in).
                self.chime_flash_t = 0.8; self.chime_flash_life = 0.8; self.chime_flash_peak = 0.22; self.chime_flash_color = Some("rgba(160, 120, 180".into());
                self.sanity_impaired_tally += 1.0;
            }
        }
        // R151 deep-breakdown chime below 20%; R152 warm mini-chime back above it.
        let is_breaking = s < 0.20 && playing;
        if self.sanity_was_breaking && !is_breaking && playing {
            if self.ac.is_some() {
                let brk = breaking_melody_by_weather(wk).or(breaking_melody_by_weather("normal")).unwrap();
                // single note two octaves above the breaking chord's `a`
                let bk_gain = bat_gain(self);
                let bk_depth = self.chime_depth_mul();
                self.blip(brk.a * 4.0 * bk_depth, 0.55, "sine", 0.035 * bk_gain, None, 0.0, 0.0);
                self.chime_flash_t = 0.6; self.chime_flash_life = 0.6; self.chime_flash_peak = 0.14; self.chime_flash_color = Some("rgba(220, 240, 210".into());
                self.sanity_mini_tally += 1.0;
            }
        }
        if !self.sanity_was_breaking && is_breaking && playing {
            if self.ac.is_some() {
                let brk = breaking_melody_by_weather(wk).or(breaking_melody_by_weather("normal")).unwrap();
                let brk_gain = bat_gain(self);
                let brk_depth = self.chime_depth_mul();
                self.blip(brk.a * brk_depth, 0.60, "sine", 0.055 * brk_gain, None, 0.0, 0.0);
                self.blip(brk.b * brk_depth, 0.75, "sine", 0.045 * brk_gain, None, 0.30, 0.0);
                self.blip(brk.c * brk_depth, 0.95, "sine", 0.038 * brk_gain, None, 0.65, 0.0);
                self.blip(brk.c * 0.5 * brk_depth, 1.10, "sine", 0.024 * brk_gain, None, 0.80, 0.0);
                // R151 deep-breakdown visual flash: heavy red-black pulse.
                self.chime_flash_t = 1.4; self.chime_flash_life = 1.4; self.chime_flash_peak = 0.42; self.chime_flash_color = Some("rgba(120, 40, 40".into());
                self.sanity_breaking_tally += 1.0;
            }
        }
        self.sanity_was_breaking = is_breaking;
        self.sanity_was_impaired = is_impaired;
    }

    /// The critical-sanity heartbeat: ~1.4 s at 28 % down to ~0.5 s at 0 % (line 65919).
    fn sanity_heartbeat(&mut self, dt: f64, s: f64) {
        self.sanity_beat_t -= dt;
        if self.sanity_beat_t <= 0.0 {
            let pct_below = clamp((0.28 - s) / 0.28, 0.0, 1.0);   // 0 → 1 as sanity falls
            self.sanity_beat_t = lerp(1.4, 0.5, pct_below);
            if self.ac.is_some() { self.sfx_thump(0.65 + pct_below * 0.35); }
        }
    }

    /// One whisper: the sound, the aberration spike and maybe a silhouette
    /// at the edge of vision (lines 65931–65991).
    fn sanity_whisper(&mut self) {
        if self.ac.is_some() { self.sfx_whisper(); }
        // fleeting shape: brief spike of the aberration overlay
        {
            let cur = self.dom.aberration.style("opacity").filter(|s| !s.is_empty()).unwrap_or("0").to_string();
            self.dom.aberration.set_style("transition", "opacity 0.08s");
            self.dom.aberration.set_style("opacity", "0.98");
            self.set_timeout(220.0, move |g| {
                g.dom.aberration.set_style("transition", "opacity 0.4s");
                g.dom.aberration.set_style("opacity", &cur);
            });
        }
        // ~55% chance of an actual visible silhouette, ahead-and-off-axis
        if rnd() < 0.55 {
            let yaw = self.player.yaw;
            let (fx, fz) = (-yaw.sin(), -yaw.cos());
            let side = if rnd() < 0.5 { 1.0 } else { -1.0 };
            let (rx, rz) = (yaw.cos() * side, -yaw.sin() * side);
            let mut placed = false;
            let mut tries = 0;
            while tries < 10 && !placed {
                tries += 1;
                let dist = rand2(4.0, 9.0);
                let off = rand2(1.5, 3.5);
                let px = self.player.pos.x + fx * dist + rx * off;
                let pz = self.player.pos.z + fz * dist + rz * off;
                let (t_x, t_z) = (self.tx(px), self.tz(pz));
                if !self.in_grid(t_x, t_z) || self.is_wall_tile(t_x, t_z) { continue; }
                self.halluc_shape.set_pos(px, 0.95, pz);
                // a random pose per appearance
                if !self.halluc_textures.is_empty() {
                    let tex = choice(&self.halluc_textures);
                    if let Some(m) = self.halluc_shape.try_mat() { m.set_map(&tex); m.needs_update(); }
                } else {
                    rnd();   // `choice([])` still draws (and yields undefined)
                }
                self.halluc_shape.set_visible(true);
                if let Some(m) = self.halluc_shape.try_mat() { m.set_opacity(0.65); }
                self.halluc_visible_t = 0.0;
                placed = true;
                // on-screen whisper chance ramps 0.30 → 0.60 over 10–40 s of crit sanity
                let ramp_t = clamp((self.crit_sanity_accum - 10.0) / 30.0, 0.0, 1.0);
                let whisper_chance = 0.30 + ramp_t * 0.30;
                if rnd() < whisper_chance {
                    let msg = choice(&HALLUC_WHISPERS);
                    self.show_msg(msg, 2.0);
                }
            }
        }
    }

    /// The hallucination's per-frame billboard, mid-appearance pose swap and
    /// fade (lines 65994–66022).
    fn halluc_animate(&mut self, dt: f64) {
        if !self.halluc_shape.visible() { return; }
        let hs = self.halluc_shape.clone();
        self.halluc_visible_t += dt;
        // face the camera each frame (billboard)
        let cp = self.camera.pos();
        hs.look_at(cp.x, hs.pos().y, cp.z);
        // 25% chance for a ~30 ms pose swap in the middle of the appearance
        if self.halluc_visible_t > 0.25 && self.halluc_visible_t < 0.35 && hs.r().user.get("swapped") == 0.0 {
            hs.user().set("swapped", 1);
            if rnd() < 0.25 {
                let current = hs.try_mat().and_then(|m| m.map_v());
                let others: Vec<Tex> = self.halluc_textures.iter().filter(|t| Some(*t) != current.as_ref()).cloned().collect();
                if !others.is_empty() {
                    let t = choice(&others);
                    if let Some(m) = hs.try_mat() { m.set_map(&t); m.needs_update(); }
                    // faint high shimmer with the visual glitch
                    if self.ac.is_some() {
                        self.blip(rand2(3200.0, 3800.0), 0.06, "sine", 0.04, None, 0.0, 0.0);
                        self.blip(rand2(4200.0, 5200.0), 0.05, "sine", 0.03, None, 0.02, 0.0);
                    }
                }
            }
        }
        // fade over 0.7 s then hide
        let fade = clamp(1.0 - self.halluc_visible_t / 0.7, 0.0, 1.0);
        if let Some(m) = hs.try_mat() { m.set_opacity(0.65 * fade); }
        if self.halluc_visible_t > 0.7 {
            hs.set_visible(false);
            hs.user().set("swapped", 0);
        }
    }

    // line 70061
    /// `ambienceLoop()` — every 9–22 s (real time) one weighted random ambient
    /// sound from `biasedAmbient` while playing.
    pub fn ambience_loop(&mut self) {
        self.set_timeout(rand2(9000.0, 22000.0), |g| {
            if g.ac.is_some() && g.state == "playing" {
                let arr: Vec<(f64, &'static str)> = if g.biased_ambient.is_empty() { AMBIENT_EVENTS.to_vec() } else { g.biased_ambient.clone() };
                let name = pick_weighted(&arr);
                g.sfx_by_name(name);
            }
            g.ambience_loop();
        });
    }

    // line 70072
    /// `updateRealityGlitch(dt)` — rare event that thickens fog, spikes the
    /// aberration overlay, plays a discord and shows a message from the walls
    pub fn update_reality_glitch(&mut self, dt: f64) {
        if self.glitch_t > 0.0 {
            self.glitch_t -= dt;
            // during the glitch updateLights dims the panels
            if self.glitch_t <= 0.0 {
                // restore — preserve the level's weather-driven fog density
                self.fog_density = FOG_BASE * self.level_weather.fog_mul;
                let v = clamp((1.0 - self.player.sanity / SAN_MAX) * 0.7, 0.0, 1.0);
                self.dom.aberration.set_opacity(v);
            }
            return;
        }
        self.glitch_next_t -= dt;
        if self.glitch_next_t > 0.0 { return; }
        // trigger glitch
        self.glitch_t = rand2(1.6, 3.2);
        self.glitch_next_t = rand2(60.0, 130.0);
        self.sfx_glitch();
        self.fog_density = FOG_BASE * 3.5;
        self.dom.aberration.set_opacity(0.85);
        // a fake message from the walls
        let msg = choice(&GLITCH_MSGS);
        self.show_msg(msg, 2.0);
        // shakes the player
        self.shake_t = 0.6;
    }

    // line 70106
    /// `updateFlickerWave(dt)` — every 90–180 s a surge ripples flicker
    /// through the panels outward from the player
    pub fn update_flicker_wave(&mut self, dt: f64) {
        self.flicker_wave_next_t -= dt;
        if self.flicker_wave_next_t > 0.0 { return; }
        self.flicker_wave_next_t = rand2(90.0, 180.0);
        if self.panels.is_empty() { return; }
        // low-frequency thump at wave start
        if self.ac.is_some() && self.state == "playing" { self.sfx_thud(); self.noise_burst(0.4, 90.0, 1.2, 0.18, "lowpass", None, 0.0, 0.0); }
        // stagger flicker outward from the player: closer panels flicker sooner
        let (px, pz) = (self.player.pos.x, self.player.pos.z);
        let mut ordered: Vec<(usize, f64, f64, f64)> = self.panels.iter().enumerate()
            .map(|(i, p)| (i, p.wx, p.wz, hypot2(p.wx - px, p.wz - pz))).collect();
        ordered.sort_by(|a, b| a.3.partial_cmp(&b.3).unwrap_or(std::cmp::Ordering::Equal));
        let step = 55.0;   // ms between successive panel flickers
        for (k, (i, wx, wz, _)) in ordered.into_iter().enumerate() {
            self.set_timeout(k as f64 * step, move |g| {
                if g.state != "playing" { return; }
                // force flicker for a short interval — updateLights handles the visuals
                let f = rand2(0.35, 0.9);
                // (JS holds the panel object; a rebuilt level's panels are not it)
                if let Some(p) = g.panels.get_mut(i) { if p.wx == wx && p.wz == wz { p.flicker = f; } }
            });
        }
    }

    // line 70129
    /// `updateGhostFootsteps(dt, moving)` — sometimes a few steps behind you while walking
    pub fn update_ghost_footsteps(&mut self, dt: f64, moving: bool) {
        if !moving { self.ghost_step_t = 0.0; return; }
        self.ghost_step_t -= dt;
        if self.ghost_step_t > 0.0 { return; }
        self.ghost_step_t = rand2(22.0, 55.0);
        // small chance to skip so it's genuinely rare
        if rnd() < 0.6 { return; }
        // 2-4 muffled footsteps out of sync with the player's own steps
        let n = irand(2, 4);
        for i in 0..n {
            let ms = i as f64 * rand2(280.0, 420.0);
            self.set_timeout(ms, |g| { if g.ac.is_some() && g.state == "playing" { g.sfx_ghost_step(); } });
        }
        // faint breath often follows
        if rnd() < 0.35 {
            self.set_timeout(n as f64 * 400.0, |g| { if g.ac.is_some() && g.state == "playing" { g.sfx_breath(); } });
        }
    }

    /// The metronome's swing period (`metronomePositions[i].period`), or the
    /// default when the placer did not record one.
    pub(crate) fn metronome_period(&self, i: usize) -> f64 {
        if self.pp.metronome_periods.len() == self.pp.metronome_positions.len() { self.pp.metronome_periods.get(i).copied().unwrap_or(METRONOME_PERIOD_DEFAULT) } else { METRONOME_PERIOD_DEFAULT }
    }
    /// The rocking chair's rock period (`chairPositions[i].period`), or the
    /// default when the placer did not record one.
    pub(crate) fn chair_period(&self, i: usize) -> f64 {
        if self.pp.chair_periods.len() == self.pp.chair_positions.len() { self.pp.chair_periods.get(i).copied().unwrap_or(CHAIR_PERIOD_DEFAULT) } else { CHAIR_PERIOD_DEFAULT }
    }
}
