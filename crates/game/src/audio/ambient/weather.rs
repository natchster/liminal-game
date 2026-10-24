//! `update_ambient`, part 2: the per-weather ambient beds, the cross-weather
//! rare events R220–R243 and the R237 reality-shift pitch bend. See
//! `audio::ambient` (mod.rs) for the overview.
use crate::audio::ambient::{rand_side, AmbMix};
use crate::base::tables::*;
use crate::prelude::*;

impl Game {
    /// The weather branch: one bed per weather key, `normal` for anything else.
    pub(crate) fn amb_weather_events(&mut self, dt: f64, wk: &str, m: &AmbMix) {
        match wk {
            "wet" => self.amb_wet(dt, m),
            "still" => self.amb_still(dt, m),
            "derelict" => self.amb_derelict(dt, m),
            "electric" => self.amb_electric(dt, m),
            "haunted" => self.amb_haunted(dt, m),
            _ => self.amb_normal(dt, m),
        }
    }

    /// wet: R91/R117/R123 drips, R97/R121 thunder, R118/R119 puddle splash
    fn amb_wet(&mut self, dt: f64, m: &AmbMix) {
        let (eg, ld, etm) = (m.gain, m.depth, m.timer_mul);
        self.wet_drip_timer -= dt;
        if self.wet_drip_timer <= 0.0 {
            // R123/R128: spawn position biased away from camera-forward
            let drip_ang = self.biased_spawn_angle();
            let drip_dist = 2.0 + rnd() * 3.0;
            let dsx = self.player.pos.x + drip_ang.cos() * drip_dist;
            let dsz = self.player.pos.z + drip_ang.sin() * drip_dist;
            // R123 distance falloff
            let dgain = 1.0 / (1.0 + drip_dist * 0.15);
            // R124 stereo pan from spawn position
            let dpan = self.pan_for_world_pos(dsx, dsz);
            // R129 air absorption
            let df_shift = (0.6f64).max(1.0 - drip_dist * 0.08);
            // the "plink": 1400→520 Hz over ~120 ms
            self.blip(1400.0 * df_shift * ld, 0.12, "sine", 0.045 * dgain * eg, Some(520.0 * df_shift * ld), 0.0, dpan);
            self.blip(240.0 * df_shift * ld, 0.30, "sine", 0.018 * dgain * eg, Some(180.0 * df_shift * ld), 0.06, dpan);
            self.wet_drip_timer = (6.0 + rnd() * 12.0) * etm; // R143 sanity-boosted
                                                              // R117/R123: 30% chance of a visible ripple at the drip's spawn
            if rnd() < 0.30 {
                self.ripple_at(dsx, dsz, 0.55, 1.0, 1.5);
            }
            self.log_ambient("wet-drip", Some(dsx), Some(dsz), Some(dpan));
        }
        // R97 distant thunder; R122: audio 150 ms after the R121 flash
        self.wet_thunder_timer -= dt;
        if self.wet_thunder_timer <= 0.0 {
            self.blip(55.0 * ld, 2.5, "sine", 0.045 * eg, Some(30.0 * ld), 0.15, 0.0);
            self.noise_burst(2.2, 90.0 * ld, 0.6, 0.028 * eg, "lowpass", None, 0.30, 0.0);
            self.wet_thunder_timer = (35.0 + rnd() * 55.0) * etm; // R143
            self.log_ambient("wet-thunder", None, None, None);
            // R121 thunder visual: brief scene-wide light flash
            self.thunder_flash_t = 0.9;
        }
        // R118 rare puddle splash: audio + oversized ripple
        self.wet_splash_timer -= dt;
        if self.wet_splash_timer <= 0.0 {
            let spl_ang = self.biased_spawn_angle();
            let spl_dist = 4.0 + rnd() * 4.0; // 4–8 m
            let sx = self.player.pos.x + spl_ang.cos() * spl_dist;
            let sz = self.player.pos.z + spl_ang.sin() * spl_dist;
            let sgain = 1.0 / (1.0 + spl_dist * 0.15);
            let span = self.pan_for_world_pos(sx, sz);
            let sf_shift = (0.55f64).max(1.0 - spl_dist * 0.08);
            // liquid slap — lowpassed noise + quick blip descent
            self.noise_burst(0.18, 500.0 * sf_shift * ld, 1.5, 0.045 * sgain * eg, "lowpass", None, 0.0, span);
            self.blip(320.0 * sf_shift * ld, 0.14, "sine", 0.03 * sgain * eg, Some(90.0 * sf_shift * ld), 0.02, span);
            self.blip(180.0 * sf_shift * ld, 0.20, "sine", 0.020 * sgain * eg, Some(60.0 * sf_shift * ld), 0.10, span);
            // oversized ripple further away (+ R119/R121 splash flash)
            if self.ripple_at(sx, sz, 0.75, 1.8, 1.8) {
                self.splash_flash_light.set_pos(sx, 0.4, sz);
                self.splash_flash_t = 0.30; // 300 ms of blue-white flash
                let splash_battery = self.beam_battery_scale(0.6); // 60% floor at empty
                self.splash_flash_peak = 1.6 * splash_battery;
                self.splash_flash_light.set_intensity(self.splash_flash_peak);
                if self.splash_flash_light.r().light().is_some() {
                    self.splash_flash_light.light().distance = 3.5 * splash_battery;
                }
            }
            self.wet_splash_timer = (55.0 + rnd() * 85.0) * etm; // R143
            self.log_ambient("wet-splash", Some(sx), Some(sz), Some(span));
        }
    }

    /// still: R92 wind gust, R101 echo whisper
    fn amb_still(&mut self, dt: f64, m: &AmbMix) {
        let (eg, ld, etm) = (m.gain, m.depth, m.timer_mul);
        self.still_wind_timer -= dt;
        if self.still_wind_timer <= 0.0 {
            // 1.6 s bandpassed noise, slow sweep 220→380 Hz
            self.noise_burst(1.6, 220.0 * ld, 0.6, 0.035 * eg, "bandpass", Some(380.0 * ld), 0.0, 0.0);
            self.still_wind_timer = (10.0 + rnd() * 18.0) * etm; // R143
            self.log_ambient("still-wind", None, None, None);
        }
        // R101 rare echo whisper: a formant burst + its echo 250 ms later
        self.still_echo_timer -= dt;
        if self.still_echo_timer <= 0.0 {
            self.noise_burst(0.7, (800.0 + rnd() * 200.0) * ld, 10.0, 0.018 * eg, "bandpass", Some(1200.0 * ld), 0.0, 0.0);
            self.noise_burst(0.6, (800.0 + rnd() * 200.0) * ld, 10.0, 0.009 * eg, "bandpass", Some(1200.0 * ld), 0.25, 0.0);
            self.still_echo_timer = (90.0 + rnd() * 90.0) * etm; // R143
            self.log_ambient("still-echo", None, None, None);
        }
    }

    /// derelict: R93/R126 HVAC clatter, R100 deep grinder
    fn amb_derelict(&mut self, dt: f64, m: &AmbMix) {
        let (eg, ld, etm) = (m.gain, m.depth, m.timer_mul);
        self.derelict_clatter_timer -= dt;
        if self.derelict_clatter_timer <= 0.0 {
            // 1–3 rapid metallic clanks panned to one side ("ducts on one wall")
            let clanks = to_i32(1.0 + rnd() * 3.0);
            let clatter_pan = (rnd() * 2.0 - 1.0) * 0.65;
            for i in 0..clanks {
                let d = i as f64 * (0.08 + rnd() * 0.10);
                let p = clamp(clatter_pan + rand2(-0.10, 0.10), -0.85, 0.85);
                self.noise_burst(0.05, (3200.0 + rnd() * 800.0) * ld, 4.0, 0.045 * eg, "highpass", None, d, p);
                self.blip((90.0 + rnd() * 30.0) * ld, 0.09, "sine", 0.030 * eg, Some(55.0 * ld), d, p);
            }
            self.derelict_clatter_timer = (12.0 + rnd() * 20.0) * etm; // R143
            self.log_ambient("derelict-clatter", None, None, Some(clatter_pan));
        }
        // R100/R126 rare deep grinder: somewhere a big fan is dying
        self.derelict_grinder_timer -= dt;
        if self.derelict_grinder_timer <= 0.0 {
            let grinder_pan = (rnd() * 2.0 - 1.0) * 0.80;
            self.blip(75.0 * ld, 2.5, "sawtooth", 0.030 * eg, Some(40.0 * ld), 0.0, grinder_pan);
            self.noise_burst(2.2, 140.0 * ld, 0.4, 0.020 * eg, "lowpass", None, 0.20, grinder_pan);
            self.noise_burst(0.35, 220.0 * ld, 1.5, 0.028 * eg, "bandpass", Some(90.0 * ld), 2.10, grinder_pan);
            self.derelict_grinder_timer = (60.0 + rnd() * 90.0) * etm; // R143
            self.log_ambient("derelict-grinder", None, None, Some(grinder_pan));
        }
    }

    /// electric: R94 crackle, R250/R251 socket sparks, R281 junction arcs, R102 surge
    fn amb_electric(&mut self, dt: f64, m: &AmbMix) {
        let (eg, ld, etm) = (m.gain, m.depth, m.timer_mul);
        self.electric_crackle_timer -= dt;
        if self.electric_crackle_timer <= 0.0 {
            // 60 Hz hum + bandpassed crackle + optional snap ("wire in this wall")
            let crackle_pan = (rnd() * 2.0 - 1.0) * 0.60;
            self.blip(60.0 * ld, 0.35, "sawtooth", 0.020 * eg, Some(0.0), 0.0, crackle_pan);
            self.noise_burst(0.14, 4200.0 * ld, 3.0, 0.035 * eg, "bandpass", None, 0.03, crackle_pan);
            if rnd() < 0.5 {
                self.noise_burst(0.05, 5600.0 * ld, 4.0, 0.028 * eg, "highpass", None, 0.20, crackle_pan);
            }
            self.electric_crackle_timer = (8.0 + rnd() * 14.0) * etm; // R143
            self.log_ambient("electric-crackle", None, None, Some(crackle_pan));
        }
        // R250/R251 socket sparks: sound + yellow sprite at the socket
        self.socket_spark_timer -= dt;
        if self.socket_spark_timer <= 0.0 && !self.pp.socket_positions.is_empty() {
            let (_, s) = crate::audio::ambient::pick_pos(&self.pp.socket_positions);
            let (spark_pan, dist2) = self.prop_pan(s, 0.7, 0.8, 6.0); // fade over 6 m
            self.blip(2800.0 + rnd() * 400.0, 0.04, "square", 0.014 * eg * (1.0 - dist2 * 0.5), None, 0.0, spark_pan);
            self.noise_burst(0.05, 6000.0, 3.0, 0.012 * eg * (1.0 - dist2 * 0.5), "highpass", None, 0.02, spark_pan);
            // R251 visual sprite (R131 scuffs pool)
            self.scuff_flash(0xffe860, s.x, s.y, s.z, 0.85 * (1.0 - dist2 * 0.3), 0.06, 0.15);
            self.socket_spark_timer = (12.0 + rnd() * 24.0) * etm; // 12-36 s
            self.log_ambient("socket-spark", Some(s.x), Some(s.z), Some(spark_pan));
        }
        // R281 junction box arc: "loose wire arcing inside box"
        self.junction_arc_timer -= dt;
        if self.junction_arc_timer <= 0.0 && !self.pp.junction_box_positions.is_empty() {
            let (_, jb) = crate::audio::ambient::pick_pos(&self.pp.junction_box_positions);
            let (arc_pan, dist_j) = self.prop_pan(jb, 0.7, 0.8, 6.0);
            let arc_gain = 0.012 * eg * (1.0 - dist_j * 0.5);
            // 3 rapid clicks + a brief high-pass burst
            for n in 0..3 {
                self.blip(2200.0 + n as f64 * 200.0, 0.015, "square", arc_gain, None, n as f64 * 0.020, arc_pan);
            }
            self.noise_burst(0.08, 4500.0, 3.0, arc_gain * 0.7, "highpass", None, 0.06, arc_pan);
            self.junction_arc_timer = (20.0 + rnd() * 25.0) * etm;
            self.log_ambient("junction-arc", Some(jb.x), Some(jb.z), Some(arc_pan));
        }
        // R102/R127 rare flicker cascade: scene-wide panel dim + panned surge
        self.electric_surge_timer -= dt;
        if self.electric_surge_timer <= 0.0 {
            self.electric_surge_t = 0.35; // 350 ms of dimming
            let surge_pan = (rnd() * 2.0 - 1.0) * 0.75;
            self.blip(180.0 * ld, 0.35, "sawtooth", 0.038 * eg, Some(55.0 * ld), 0.0, surge_pan);
            self.noise_burst(0.30, 1400.0 * ld, 2.0, 0.028 * eg, "bandpass", None, 0.02, surge_pan);
            self.noise_burst(0.05, 6200.0 * ld, 5.0, 0.030 * eg, "highpass", None, 0.32, surge_pan);
            self.electric_surge_timer = (60.0 + rnd() * 60.0) * etm; // R143
            self.log_ambient("electric-surge", None, None, Some(surge_pan));
        }
    }

    /// haunted: R94 whisper, R102 distant footsteps
    fn amb_haunted(&mut self, dt: f64, m: &AmbMix) {
        let (eg, ld, etm) = (m.gain, m.depth, m.timer_mul);
        self.haunted_whisper_timer -= dt;
        if self.haunted_whisper_timer <= 0.0 {
            // voice-like formant from "the other side of the room" (±0.6–0.85)
            let whisper_pan = rand_side(0.60, 0.25);
            self.noise_burst(
                0.9,
                (500.0 + rnd() * 200.0) * ld,
                8.0,
                0.028 * eg,
                "bandpass",
                Some((1400.0 + rnd() * 300.0) * ld),
                0.0,
                whisper_pan,
            );
            // occasional soft breath through a low-pass
            if rnd() < 0.4 {
                self.noise_burst(0.5, 200.0, 1.5, 0.020 * eg, "lowpass", None, 0.20, whisper_pan);
            }
            self.haunted_whisper_timer = (14.0 + rnd() * 22.0) * etm; // R143
            self.log_ambient("haunted-whisper", None, None, Some(whisper_pan));
        }
        // R102/R125 rare distant footstep: someone walking in the next room
        self.haunted_step_timer -= dt;
        if self.haunted_step_timer <= 0.0 {
            let steps = to_i32(2.0 + rnd() * 3.0);
            let pan_cur = rand_side(0.65, 0.20);
            let pan_drift = (if rnd() < 0.5 { -1.0 } else { 1.0 }) * 0.15;
            for i in 0..steps {
                let d = i as f64 * (0.55 + rnd() * 0.15);
                let p = clamp(pan_cur + pan_drift * i as f64, -0.8, 0.8);
                // muffled low thud: sine + brief noise pad
                self.blip(110.0 * ld, 0.14, "sine", 0.026 * eg, Some(65.0 * ld), d, p);
                self.noise_burst(0.08, 180.0 * ld, 0.8, 0.014 * eg, "lowpass", None, d + 0.02, p);
            }
            self.haunted_step_timer = (80.0 + rnd() * 80.0) * etm; // R143
            self.log_ambient("haunted-step", None, None, Some(pan_cur));
        }
    }

    /// normal / unknown weather: R104 very rare "the game noticed" chime
    fn amb_normal(&mut self, dt: f64, m: &AmbMix) {
        let (eg, ld, etm) = (m.gain, m.depth, m.timer_mul);
        self.normal_notice_timer -= dt;
        if self.normal_notice_timer <= 0.0 {
            self.blip(1760.0 * ld, 0.7, "sine", 0.020 * eg, Some(1320.0 * ld), 0.0, 0.0);
            self.blip(2640.0 * ld, 0.5, "sine", 0.012 * eg, Some(1980.0 * ld), 0.12, 0.0);
            self.normal_notice_timer = (120.0 + rnd() * 120.0) * etm; // R143
            self.log_ambient("normal-chime", None, None, None);
        }
    }

    /// Count a rare event in `rare_event_tally[cause][wk]` (empty `wk` counts as
    /// `"unknown"`) and set the R228/R236 fire stamps
    fn rare_fired(&mut self, cause: &str, wk: &str) {
        let key = if wk.is_empty() { "unknown" } else { wk };
        let e = self.rare_event_tally.entry(cause.to_string()).or_default();
        *e.entry(key.to_string()).or_insert(0.0) += 1.0;
        if self.ac.is_some() {
            self.last_rare_fire_t = self.ac_time();
            let t = self.last_rare_fire_t;
            let r = &mut self.last_rare_fire_by_cause;
            match cause {
                "knock" => r.knock = t,
                "door" => r.door = t,
                "ding" => r.ding = t,
                "voice" => r.voice = t,
                "static" => r.r#static = t,
                "shift" => r.shift = t,
                "pattern" => r.pattern = t,
                _ => r.skip = t,
            }
        }
    }

    /// R220–R243 cross-weather rare events with the R227 stagger.
    pub(crate) fn amb_rare_events(&mut self, dt: f64, wk: &str, m: &AmbMix) {
        let (eg, ld, etm) = (m.gain, m.depth, m.timer_mul);
        // R227 stagger: any rare event mid-flash delays new fires by 1.5 s
        let rare_event_active = self.knock_flash_t > 0.3
            || self.door_opened_flash_t > 0.3
            || self.elevator_ding_flash_t > 0.3
            || self.voice_sample_flash_t > 0.3
            || self.static_burst_flash_t > 0.3
            || self.reality_shift_t > 0.5
            || self.pattern_break_t > 0.5
            || self.clock_skip_t > 0.15;
        let stagger_delay = if rare_event_active { 1.5 } else { 0.0 };
        // R230 sanity → rare event boost (up to 15% shorter timers at 0 sanity)
        let rtm = 1.0 / (1.0 + m.sanity_mag * 0.18);

        // R220/R221 distant knock — once per 4–8 min
        self.distant_knock_timer -= dt;
        if self.distant_knock_timer <= 0.0 && rare_event_active {
            self.distant_knock_timer = stagger_delay;
        }
        if self.distant_knock_timer <= 0.0 {
            let knock_pan = rand_side(0.55, 0.30);
            let kt = knock_timbre_by_weather(wk).or(knock_timbre_by_weather("normal")).unwrap();
            let knock_gain = 0.030 * eg * kt.gain_mul;
            // two-tap knock: distant knuckles on a wall
            self.blip(kt.freq * ld, 0.12, "sine", knock_gain, Some((kt.freq * 0.56) * ld), 0.0, knock_pan);
            self.blip((kt.freq * 0.96) * ld, 0.14, "sine", knock_gain * 0.85, Some((kt.freq * 0.53) * ld), 0.16, knock_pan);
            self.noise_burst(0.08, kt.noise * ld, 0.5, knock_gain * 0.6, "lowpass", None, 0.02, knock_pan);
            self.noise_burst(0.08, kt.noise * ld, 0.5, knock_gain * 0.5, "lowpass", None, 0.18, knock_pan);
            self.distant_knock_timer = (240.0 + rnd() * 240.0) * etm * rtm; // 4–8 min × sanity boost
            self.knock_flash_t = 0.6; // vignette pulse (R220)
            self.rare_fired("knock", wk);
            // R222 panel-tint direction hint on the knock's side
            self.knock_tint_t = 0.7;
            self.knock_tint_pan = knock_pan;
            self.last_knock_pan = knock_pan;
            self.log_ambient("distant-knock", None, None, Some(knock_pan));
        }
        // R223/R224 door opened — 8–15 min: creak + airflow whoosh + sub tail
        self.door_opened_timer -= dt;
        if self.door_opened_timer <= 0.0 && rare_event_active {
            self.door_opened_timer = stagger_delay;
        }
        if self.door_opened_timer <= 0.0 {
            let door_pan = rand_side(0.30, 0.40);
            let dt2 = door_timbre_by_weather(wk).or(door_timbre_by_weather("normal")).unwrap();
            let door_gain = 0.025 * eg * dt2.gain_mul;
            self.blip(dt2.creak_start * ld, 1.2, "sawtooth", door_gain, Some(dt2.creak_end * ld), 0.0, door_pan);
            self.noise_burst(0.8, dt2.whoosh * ld, 0.4, door_gain * 0.7, "lowpass", None, 0.40, door_pan);
            self.blip(60.0 * ld, 0.6, "sine", door_gain * 0.4, Some(dt2.sub_end * ld), 0.50, door_pan);
            self.door_opened_timer = (480.0 + rnd() * 420.0) * etm * rtm; // 8–15 min × sanity boost
            self.door_opened_flash_t = 1.0; // longer vignette pulse than knock
            self.door_dim_pulse_t = 0.5; // R224 "air guttered the lights"
            self.rare_fired("door", wk);
            self.log_ambient("door-opened", None, None, Some(door_pan));
        }
        // R225 elevator ding — 12–25 min: bell, counter-tone, machine rumble
        self.elevator_ding_timer -= dt;
        if self.elevator_ding_timer <= 0.0 && rare_event_active {
            self.elevator_ding_timer = stagger_delay;
        }
        if self.elevator_ding_timer <= 0.0 {
            let ding_pan = rand_side(0.20, 0.30);
            let dt3 = ding_timbre_by_weather(wk).or(ding_timbre_by_weather("normal")).unwrap();
            let ding_gain = 0.028 * eg * dt3.gain_mul;
            self.blip(dt3.high * ld, 1.8, "sine", ding_gain, Some((dt3.high - 15.0) * ld), 0.0, ding_pan);
            self.blip(dt3.low * ld, 1.5, "sine", ding_gain * 0.75, Some((dt3.low - 12.0) * ld), 0.35, ding_pan);
            self.noise_burst(0.9, dt3.rumble * ld, 0.5, ding_gain * 0.5, "lowpass", None, 0.60, ding_pan);
            self.elevator_ding_timer = (720.0 + rnd() * 780.0) * etm * rtm; // 12–25 min × sanity boost
            self.elevator_ding_flash_t = 1.4;
            self.ding_bright_pulse_t = 0.4; // R226 panel-brighten pulse
            self.rare_fired("ding", wk);
            self.log_ambient("elevator-ding", None, None, Some(ding_pan));
        }
        // R228 voice sample — 15–30 min: an almost-word from very far off
        self.voice_sample_timer -= dt;
        if self.voice_sample_timer <= 0.0 && rare_event_active {
            self.voice_sample_timer = stagger_delay;
        }
        if self.voice_sample_timer <= 0.0 {
            let voice_pan = rand_side(0.55, 0.30);
            let vt = voice_timbre_by_weather(wk).or(voice_timbre_by_weather("normal")).unwrap();
            let voice_gain = 0.022 * eg * vt.gain_mul;
            // breath-in, vowel formant sweep, fade-out breath
            self.noise_burst(0.20, vt.breath_lp, 0.4, voice_gain * 0.35, "lowpass", None, 0.0, voice_pan);
            self.noise_burst(0.35, vt.f_start, 3.0, voice_gain * 0.55, "bandpass", Some(vt.f_end), 0.15, voice_pan);
            self.noise_burst(0.25, vt.breath_lp * 0.44, 0.3, voice_gain * 0.25, "lowpass", None, 0.40, voice_pan);
            self.voice_sample_timer = (900.0 + rnd() * 900.0) * etm * rtm; // 15–30 min × sanity boost
            self.voice_sample_flash_t = 0.8;
            self.voice_hue_shift_t = 1.0; // R232 panel hue shift
            self.rare_fired("voice", wk);
            self.log_ambient("voice-sample", None, None, Some(voice_pan));
        }
        // R230 static burst — 7–14 min: hiss, pop clicks, fade tail
        self.static_burst_timer -= dt;
        if self.static_burst_timer <= 0.0 && rare_event_active {
            self.static_burst_timer = stagger_delay;
        }
        if self.static_burst_timer <= 0.0 {
            let static_pan = rand_side(0.40, 0.35);
            let st = static_timbre_by_weather(wk).or(static_timbre_by_weather("normal")).unwrap();
            let static_gain = 0.020 * eg * st.gain_mul;
            self.noise_burst(0.15, st.hp_freq, 1.2, static_gain * 0.8, "highpass", None, 0.0, static_pan);
            let mut i = 0.0;
            while i < st.clicks {
                let d = 0.08 + i * 0.065 + rnd() * 0.020;
                self.blip(st.click_base + rnd() * 300.0, 0.02, "square", static_gain * 0.5, None, d, static_pan);
                i += 1.0;
            }
            self.noise_burst(0.30, st.tail_freq, 0.8, static_gain * 0.4, "bandpass", None, 0.30, static_pan);
            self.static_burst_timer = (420.0 + rnd() * 420.0) * etm * rtm; // 7–14 min
            self.static_burst_flash_t = 0.5;
            self.static_chromatic_t = 0.35; // R232 chromatic aberration punch
            self.rare_fired("static", wk);
            self.log_ambient("static-burst", None, None, Some(static_pan));
        }
        // R237 reality shift — 30–60 min: the ambient bed itself bends (see amb_shift_bend)
        self.reality_shift_timer -= dt;
        if self.reality_shift_timer <= 0.0 && rare_event_active {
            self.reality_shift_timer = stagger_delay;
        }
        if self.reality_shift_timer <= 0.0 {
            self.reality_shift_t = 2.0; // 2 s envelope
            self.reality_shift_timer = (1800.0 + rnd() * 1800.0) * etm * rtm; // 30-60 min
            self.rare_fired("shift", wk);
            self.log_ambient("reality-shift", None, None, None);
            // R239/R240 per-weather chance of a settle chord 5–8 s later
            let settle_spec = settle_chord_by_weather(wk).or(settle_chord_by_weather("normal")).unwrap();
            if rnd() < settle_spec.prob {
                self.settle_timer = 5.0 + rnd() * 3.0;
            }
        }
        // R239/R240 settle recovery: the pending countdown fires a descending chord
        if self.settle_timer > 0.0 {
            self.settle_timer -= dt;
            if self.settle_timer <= 0.0 {
                let settle_pan = rand_side(0.20, 0.30);
                let settle_gain = 0.018 * eg;
                let n = settle_chord_by_weather(wk).or(settle_chord_by_weather("normal")).unwrap().notes;
                self.blip(n[0] * ld, 1.8, "sine", settle_gain, Some((n[0] - 12.0) * ld), 0.0, settle_pan);
                self.blip(n[1] * ld, 1.8, "sine", settle_gain * 0.85, Some((n[1] - 10.0) * ld), 0.40, settle_pan);
                self.blip(n[2] * ld, 2.0, "sine", settle_gain * 0.75, Some((n[2] - 8.0) * ld), 0.80, settle_pan);
                self.settle_visual_t = 2.0; // R241 panel warmth pulse
                self.log_ambient("reality-settle", None, None, Some(settle_pan));
            }
        }
        // R241 pattern break — 60–120 min: every panel LFO breathes in phase
        self.pattern_break_timer -= dt;
        if self.pattern_break_timer <= 0.0 && rare_event_active {
            self.pattern_break_timer = stagger_delay;
        }
        if self.pattern_break_timer <= 0.0 {
            let ps = pattern_sync_by_weather(wk).or(pattern_sync_by_weather("normal")).unwrap();
            self.pattern_break_t = ps.dur; // R242 per-weather duration
            for p in self.panels.iter_mut() {
                p.lfo_phase = 0.0;
            }
            // R242 "the ceiling exhaled together" sub swell
            self.blip(ps.sub_freq * ld, ps.dur, "sine", ps.sub_gain * eg, Some((ps.sub_freq * 0.8) * ld), 0.0, 0.0);
            self.pattern_break_timer = (3600.0 + rnd() * 3600.0) * etm * rtm; // 60-120 min
            self.rare_fired("pattern", wk);
            self.log_ambient("pattern-break", None, None, None);
        }
        if self.pattern_break_t > 0.0 {
            self.pattern_break_t = (0.0f64).max(self.pattern_break_t - dt);
        }
        // R243 clock skip — 90–180 min: panel LFOs freeze 0.5 s, resume offset
        self.clock_skip_timer -= dt;
        if self.clock_skip_timer <= 0.0 && rare_event_active {
            self.clock_skip_timer = stagger_delay;
        }
        if self.clock_skip_timer <= 0.0 {
            self.clock_skip_t = 0.5; // 0.5 s freeze envelope
            for p in self.panels.iter_mut() {
                p.lfo_phase_offset = (rnd() - 0.5) * 0.5;
            }
            // a single soft relay click
            let skip_pan = rand_side(0.15, 0.20);
            self.blip(1200.0, 0.04, "square", 0.010 * eg, None, 0.0, skip_pan);
            self.noise_burst(0.06, 400.0, 1.0, 0.006 * eg, "bandpass", None, 0.02, skip_pan);
            self.clock_skip_timer = (5400.0 + rnd() * 5400.0) * etm * rtm; // 90-180 min
            self.rare_fired("skip", wk);
            self.log_ambient("clock-skip", None, None, Some(skip_pan));
        }
        if self.clock_skip_t > 0.0 {
            self.clock_skip_t = (0.0f64).max(self.clock_skip_t - dt);
        }
    }

    /// R237/R238 shift envelope decay + pitch bend of the always-running
    /// safe-room LFO, tremolo carrier and room-tone lowpass.
    pub(crate) fn amb_shift_bend(&mut self, dt: f64) {
        if self.reality_shift_t <= 0.0 {
            return;
        }
        self.reality_shift_t = (0.0f64).max(self.reality_shift_t - dt);
        let env = self.reality_shift_t / 2.0; // 1 → 0 over 2 s
        let wobble_phase = ((2.0 - self.reality_shift_t) * PI * 1.2).sin();
        let wk = self.weather_key();
        let shift_spec = shift_bend_by_weather(wk).or(shift_bend_by_weather("normal")).unwrap();
        let bend_mul = 1.0 - shift_spec.bend * env * env * wobble_phase;
        let level_d = self.level_depth_mul();
        if let Some(lfo) = &self.audio.safe_room_lfo {
            let lfo_jit = ((self.current_seed & 0xff) as f64 / 255.0 - 0.5) * 0.20;
            let lfo_weather_mul = lfo_weather_mul(wk).unwrap_or(1.0);
            let lfo_nominal = (0.35 + lfo_jit) * lfo_weather_mul * level_d;
            lfo.frequency().set_value(lfo_nominal * bend_mul);
        }
        // R238 the tremolo carrier + room-tone LP pitch with it
        if let Some(car) = &self.audio.safe_room_trem_carrier {
            let car_jit = (((self.current_seed >> 8) & 0xff) as f64 / 255.0 - 0.5) * 40.0;
            let car_weather_off = carrier_weather_detune(wk).unwrap_or(0.0);
            let car_nominal = (330.0 + car_jit + car_weather_off) * level_d;
            car.frequency().set_value(car_nominal * bend_mul);
        }
        if let Some(lp) = &self.audio.room_tone_lp {
            let rt_nominal = room_tone_by_weather(wk).or(room_tone_by_weather("normal")).unwrap().lp;
            lp.frequency().set_value(rt_nominal * bend_mul);
        }
    }
}
