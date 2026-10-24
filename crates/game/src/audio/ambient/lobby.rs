//! `update_ambient`, part 5: lobby props R542–R564 + the R273 motion sensor
//! and the rare "it still works" fixtures R275–R325. See `audio::ambient`
//! (mod.rs) for the overview.
use crate::audio::ambient::{pick_pos, AmbMix};
use crate::prelude::*;

impl Game {
    /// R542–R564 pay phone … plant, then the R273 motion-sensor click.
    pub(crate) fn amb_props_lobby(&mut self, dt: f64, m: &AmbMix) {
        let eg = m.gain;
        let etm = m.timer_mul;
        // R542 pay phone ring: classic 2-tone double burst
        self.pay_phone_ring_timer -= dt;
        if self.pay_phone_ring_timer <= 0.0 && !self.pp.pay_phone_positions.is_empty() {
            let (_, pp) = pick_pos(&self.pp.pay_phone_positions);
            let (pan, dn) = self.prop_pan(pp, 0.7, 0.8, 12.0);
            let ppg = 0.020 * eg * (1.0 - dn * 0.5);
            self.blip(950.0, 0.30, "sine", ppg, None, 0.0, pan);
            self.blip(1250.0, 0.30, "sine", ppg, None, 0.0, pan);
            self.blip(950.0, 0.30, "sine", ppg, None, 0.55, pan);
            self.blip(1250.0, 0.30, "sine", ppg, None, 0.55, pan);
            self.pay_phone_ring_timer = (50.0 + rnd() * 50.0) * etm;
            self.log_ambient("payphone-ring", Some(pp.x), Some(pp.z), Some(pan));
        }
        // R542 transistor radio AM static (< 4 m)
        self.radio_static_timer -= dt;
        if self.radio_static_timer <= 0.0 && !self.pp.radio_positions.is_empty() {
            let (_, r3) = pick_pos(&self.pp.radio_positions);
            if self.prop_d2(r3) < 16.0 {
                let (pan, dn) = self.prop_pan(r3, 0.6, 0.8, 4.0);
                let rg = 0.010 * eg * (1.0 - dn * 0.6);
                self.noise_burst(0.3, 3800.0, 0.6, rg, "highpass", None, 0.0, pan);
                self.blip(360.0, 0.20, "sine", rg * 0.3, Some(340.0), 0.05, pan);
                self.log_ambient("radio-static", Some(r3.x), Some(r3.z), Some(pan));
            }
            self.radio_static_timer = (10.0 + rnd() * 12.0) * etm;
        }
        // R543 keychain jangle (< 2 m)
        self.keychain_jangle_timer -= dt;
        if self.keychain_jangle_timer <= 0.0 && !self.pp.keychain_positions.is_empty() {
            let (_, kc) = pick_pos(&self.pp.keychain_positions);
            if self.prop_d2(kc) < 4.0 {
                let (pan, dn) = self.prop_pan(kc, 0.7, 0.8, 2.0);
                let kcg = 0.008 * eg * (1.0 - dn * 0.7);
                for i in 0..3 {
                    self.noise_burst(0.015, 3800.0 + rnd() * 600.0, 6.0, kcg * (0.6 + rnd() * 0.4), "bandpass", None, i as f64 * 0.02, pan);
                }
                self.log_ambient("keychain-jangle", Some(kc.x), Some(kc.z), Some(pan));
            }
            self.keychain_jangle_timer = (20.0 + rnd() * 25.0) * etm;
        }
        // R561 elevator button beep (< 6 m)
        self.elevator_beep_timer -= dt;
        if self.elevator_beep_timer <= 0.0 && !self.pp.elevator_panel_positions.is_empty() {
            let (_, ep) = pick_pos(&self.pp.elevator_panel_positions);
            if self.prop_d2(ep) < 36.0 {
                let (pan, dn) = self.prop_pan(ep, 0.7, 0.8, 6.0);
                let epg = 0.012 * eg * (1.0 - dn * 0.5);
                self.blip(1200.0, 0.15, "sine", epg, Some(900.0), 0.0, pan);
                self.log_ambient("elevator-beep", Some(ep.x), Some(ep.z), Some(pan));
            }
            self.elevator_beep_timer = (30.0 + rnd() * 30.0) * etm;
        }
        // R564 plant leaf rustle (< 3 m)
        self.plant_rustle_timer -= dt;
        if self.plant_rustle_timer <= 0.0 && !self.pp.plant_positions.is_empty() {
            let (_, pl) = pick_pos(&self.pp.plant_positions);
            if self.prop_d2(pl) < 9.0 {
                let (pan, dn) = self.prop_pan(pl, 0.7, 0.8, 3.0);
                let plg = 0.008 * eg * (1.0 - dn * 0.6);
                self.noise_burst(0.15, 800.0, 0.8, plg, "lowpass", None, 0.0, pan);
                self.log_ambient("plant-rustle", Some(pl.x), Some(pl.z), Some(pan));
            }
            self.plant_rustle_timer = (20.0 + rnd() * 20.0) * etm;
        }
        // R273 motion sensor click within 2 m, 5 s global cooldown
        if !self.pp.motion_sensor_positions.is_empty() && self.perf_t - self.motion_sensor_last_click_t > 5.0 {
            let sensors = self.pp.motion_sensor_positions.clone();
            for ms in sensors {
                if self.prop_d2(ms) < 4.0 {
                    // 2 m radius
                    let (pan, _) = self.prop_pan(ms, 0.5, 0.7, 1.0);
                    self.noise_burst(0.02, 5000.0, 4.0, 0.008 * eg, "highpass", None, 0.0, pan);
                    self.motion_sensor_last_click_t = self.perf_t;
                    self.log_ambient("motion-click", Some(ms.x), Some(ms.z), Some(pan));
                    break;
                }
            }
        }
    }

    /// R275–R325: speaker … broken phone.
    pub(crate) fn amb_props_rare_fixtures(&mut self, dt: f64, m: &AmbMix) {
        let eg = m.gain;
        let etm = m.timer_mul;
        // R275 ceiling speaker PA announcement: ding + garbled voice band + click
        self.speaker_announce_timer -= dt;
        if self.speaker_announce_timer <= 0.0 && !self.pp.speaker_positions.is_empty() {
            let (_, sp) = pick_pos(&self.pp.speaker_positions);
            let (pan, dn) = self.prop_pan(sp, 0.7, 0.8, 8.0);
            let spk_gain = 0.020 * eg * (1.0 - dn * 0.4);
            self.blip(523.0, 0.4, "sine", spk_gain, None, 0.0, pan);
            self.blip(659.0, 0.5, "sine", spk_gain * 0.8, None, 0.20, pan);
            self.noise_burst(1.2, 700.0, 3.0, spk_gain * 0.4, "bandpass", Some(400.0), 0.70, pan);
            self.blip(1200.0, 0.03, "square", spk_gain * 0.5, None, 1.95, pan);
            self.speaker_announce_timer = (180.0 + rnd() * 180.0) * etm; // 3-6 min
            self.log_ambient("speaker-announce", Some(sp.x), Some(sp.z), Some(pan));
        }
        // R276 fire alarm dying-battery beep
        self.fire_alarm_beep_timer -= dt;
        if self.fire_alarm_beep_timer <= 0.0 && !self.pp.fire_alarm_positions.is_empty() {
            let (_, fa) = pick_pos(&self.pp.fire_alarm_positions);
            let (pan, dn) = self.prop_pan(fa, 0.7, 0.8, 7.0);
            let beep_gain = 0.014 * eg * (1.0 - dn * 0.5);
            self.blip(2400.0, 0.08, "square", beep_gain, None, 0.0, pan);
            self.fire_alarm_beep_timer = (300.0 + rnd() * 300.0) * etm; // 5-10 min
            self.log_ambient("alarm-beep", Some(fa.x), Some(fa.z), Some(pan));
        }
        // R278 phone ringing to no one: 450 + 480 Hz, twice
        self.phone_ring_timer -= dt;
        if self.phone_ring_timer <= 0.0 && !self.pp.phone_positions.is_empty() {
            let (_, ph) = pick_pos(&self.pp.phone_positions);
            let (pan, dn) = self.prop_pan(ph, 0.7, 0.8, 8.0);
            let ring_gain = 0.018 * eg * (1.0 - dn * 0.4);
            for ring_rep in 0..2 {
                let t0 = ring_rep as f64 * 0.40;
                self.blip(450.0, 0.2, "sine", ring_gain, None, t0, pan);
                self.blip(480.0, 0.2, "sine", ring_gain * 0.85, None, t0, pan);
            }
            self.phone_ring_timer = (600.0 + rnd() * 600.0) * etm; // 10-20 min
            self.log_ambient("phone-ring", Some(ph.x), Some(ph.z), Some(pan));
        }
        // R280 vending machine compressor throb
        self.vending_hum_timer -= dt;
        if self.vending_hum_timer <= 0.0 && !self.pp.vending_positions.is_empty() {
            let (_, vm) = pick_pos(&self.pp.vending_positions);
            let (pan, dn) = self.prop_pan(vm, 0.6, 0.7, 7.0);
            let hum_gain = 0.016 * eg * (1.0 - dn * 0.5);
            self.blip(30.0, 3.0, "sine", hum_gain, None, 0.0, pan);
            self.blip(60.0, 3.0, "sine", hum_gain * 0.5, None, 0.0, pan);
            self.vending_hum_timer = (720.0 + rnd() * 780.0) * etm; // 12-25 min
            self.log_ambient("vending-hum", Some(vm.x), Some(vm.z), Some(pan));
        }
        // R283/R284 keycard reader phantom read (85% denied, 15% admitted)
        self.keycard_beep_timer -= dt;
        if self.keycard_beep_timer <= 0.0 && !self.pp.keycard_reader_positions.is_empty() {
            let (_, kr) = pick_pos(&self.pp.keycard_reader_positions);
            let (pan, dn) = self.prop_pan(kr, 0.7, 0.8, 6.0);
            let beep_gain = 0.012 * eg * (1.0 - dn * 0.5);
            let is_success = rnd() < 0.15;
            if is_success {
                self.blip(400.0, 0.06, "square", beep_gain, None, 0.0, pan);
                self.blip(800.0, 0.08, "square", beep_gain * 0.9, None, 0.07, pan);
            } else {
                self.blip(800.0, 0.06, "square", beep_gain, None, 0.0, pan);
                self.blip(400.0, 0.08, "square", beep_gain * 0.9, None, 0.07, pan);
            }
            self.keycard_beep_timer = (120.0 + rnd() * 120.0) * etm; // 2-4 min
            self.log_ambient(if is_success { "keycard-beep-ok" } else { "keycard-beep" }, Some(kr.x), Some(kr.z), Some(pan));
        }
        // R287 cardboard box stack settling
        self.cardboard_shift_timer -= dt;
        if self.cardboard_shift_timer <= 0.0 && !self.pp.cardboard_box_positions.is_empty() {
            let (_, cb) = pick_pos(&self.pp.cardboard_box_positions);
            let (pan, dn) = self.prop_pan(cb, 0.6, 0.7, 6.0);
            let shift_gain = 0.012 * eg * (1.0 - dn * 0.5);
            self.noise_burst(0.15, 800.0, 1.5, shift_gain * 0.6, "lowpass", Some(300.0), 0.0, pan);
            self.blip(120.0, 0.08, "sine", shift_gain * 0.5, Some(60.0), 0.10, pan);
            self.cardboard_shift_timer = (180.0 + rnd() * 180.0) * etm; // 3-6 min
            self.log_ambient("cardboard-shift", Some(cb.x), Some(cb.z), Some(pan));
        }
        // R291 living plant rustle — "the leaves moved somehow"
        self.plant_rustle_timer_r291 -= dt;
        if self.plant_rustle_timer_r291 <= 0.0 && !self.pp.living_plant_positions.is_empty() {
            let (_, lp) = pick_pos(&self.pp.living_plant_positions);
            let (pan, dn) = self.prop_pan(lp, 0.6, 0.7, 5.0);
            let rustle_gain = 0.010 * eg * (1.0 - dn * 0.6);
            self.noise_burst(0.20, 2500.0, 1.2, rustle_gain * 0.8, "highpass", Some(1500.0), 0.0, pan);
            self.noise_burst(0.08, 3000.0, 1.5, rustle_gain * 0.5, "highpass", None, 0.15, pan);
            self.plant_rustle_timer_r291 = (240.0 + rnd() * 240.0) * etm; // 4-8 min
            self.log_ambient("plant-rustle", Some(lp.x), Some(lp.z), Some(pan));
        }
        // R307/R314/R315 walkie-talkie static, garbled voice, the answer back
        self.walkie_static_timer -= dt;
        if self.walkie_static_timer <= 0.0 && !self.pp.walkie_talkie_positions.is_empty() {
            let (_, wp) = pick_pos(&self.pp.walkie_talkie_positions);
            let (pan, dn) = self.prop_pan(wp, 0.7, 0.8, 6.0);
            let static_gain = 0.014 * eg * (1.0 - dn * 0.55);
            self.blip(600.0, 0.03, "square", static_gain * 0.4, None, 0.0, pan);
            self.noise_burst(0.22, 2200.0, 1.4, static_gain, "bandpass", Some(1800.0), 0.04, pan);
            self.noise_burst(0.06, 3400.0, 3.0, static_gain * 0.6, "highpass", None, 0.26, pan);
            // 25% a tiny formant flicker suggests a distant voice
            if rnd() < 0.25 {
                self.noise_burst(0.14, 700.0, 8.0, static_gain * 0.5, "bandpass", Some(1100.0), 0.10, pan);
            }
            // R314 15%: extended garbled-voice sample
            if rnd() < 0.15 {
                let syllable_count = 3 + (rnd() * 2.0).floor() as i32;
                for sy in 0..syllable_count {
                    let syl_base = 400.0 + rnd() * 500.0;
                    let syl_end = syl_base * (0.7 + rnd() * 0.6);
                    let syl_dur = 0.10 + rnd() * 0.10;
                    let syl_delay = 0.35 + sy as f64 * (0.14 + rnd() * 0.10);
                    self.noise_burst(syl_dur, syl_base, 10.0, static_gain * 0.55, "bandpass", Some(syl_end), syl_delay, pan);
                }
                // end-of-transmission click
                self.blip(1400.0, 0.02, "square", static_gain * 0.35, None, 0.35 + syllable_count as f64 * 0.24, pan);
                // R315 10%: someone answers back 4–6 s later
                if rnd() < 0.10 {
                    let answer_delay = 4.0 + rnd() * 2.0;
                    let ans_syl_count = 2 + (rnd() * 2.0).floor() as i32;
                    for sy in 0..ans_syl_count {
                        let syl_base = 350.0 + rnd() * 450.0;
                        let syl_end = syl_base * (0.7 + rnd() * 0.6);
                        let syl_dur = 0.10 + rnd() * 0.10;
                        let syl_delay = answer_delay + sy as f64 * (0.16 + rnd() * 0.10);
                        self.noise_burst(syl_dur, syl_base, 10.0, static_gain * 0.28, "bandpass", Some(syl_end), syl_delay, pan);
                    }
                    self.blip(1400.0, 0.02, "square", static_gain * 0.18, None, answer_delay + ans_syl_count as f64 * 0.26, pan);
                    self.log_ambient("walkie-answer", Some(wp.x), Some(wp.z), Some(pan));
                }
            }
            self.walkie_static_timer = (180.0 + rnd() * 180.0) * etm; // 3-6 min
            self.log_ambient("walkie-static", Some(wp.x), Some(wp.z), Some(pan));
        }
        // R308 the frozen wall clock ticks once (sometimes twice)
        self.wall_clock_tick_timer -= dt;
        if self.wall_clock_tick_timer <= 0.0 && !self.pp.wall_clock_positions.is_empty() {
            let (_, wcl) = pick_pos(&self.pp.wall_clock_positions);
            let (pan, dn) = self.prop_pan(wcl, 0.7, 0.8, 6.0);
            let tick_gain = 0.011 * eg * (1.0 - dn * 0.55);
            self.noise_burst(0.03, 900.0, 3.0, tick_gain * 0.9, "lowpass", None, 0.0, pan);
            self.blip(1200.0, 0.02, "square", tick_gain * 0.4, None, 0.005, pan);
            if rnd() < 0.30 {
                self.noise_burst(0.03, 900.0, 3.0, tick_gain * 0.9, "lowpass", None, 0.90, pan);
                self.blip(1200.0, 0.02, "square", tick_gain * 0.4, None, 0.905, pan);
            }
            self.wall_clock_tick_timer = (240.0 + rnd() * 240.0) * etm; // 4-8 min
            self.log_ambient("wallclock-tick", Some(wcl.x), Some(wcl.z), Some(pan));
        }
        // R342 cracked clock wooden groan
        self.cracked_clock_creak_timer -= dt;
        if self.cracked_clock_creak_timer <= 0.0 && !self.pp.cracked_clock_positions.is_empty() {
            let (_, cc) = pick_pos(&self.pp.cracked_clock_positions);
            let (pan, dn) = self.prop_pan(cc, 0.7, 0.8, 6.0);
            let creak_gain = 0.011 * eg * (1.0 - dn * 0.55);
            self.blip(300.0, 0.50, "sawtooth", creak_gain * 0.9, Some(180.0), 0.0, pan);
            self.noise_burst(0.35, 260.0, 1.2, creak_gain * 0.5, "bandpass", Some(200.0), 0.05, pan);
            self.cracked_clock_creak_timer = (180.0 + rnd() * 180.0) * etm; // 3-6 min
            self.log_ambient("cracked-clock-creak", Some(cc.x), Some(cc.z), Some(pan));
        }
        // R353/R354/R355 ceiling drip landing + a splash sprite
        self.ceiling_drip_audio_timer -= dt;
        if self.ceiling_drip_audio_timer <= 0.0 && !self.pp.ceiling_drip_positions.is_empty() {
            let (_, cd) = pick_pos(&self.pp.ceiling_drip_positions);
            let (pan, dn) = self.prop_pan(cd, 0.7, 0.8, 6.0);
            let drip_gain = 0.010 * eg * (1.0 - dn * 0.55);
            self.blip(1600.0, 0.05, "sine", drip_gain, Some(1100.0), 0.0, pan);
            self.noise_burst(0.03, 3600.0, 4.0, drip_gain * 0.35, "highpass", None, 0.005, pan);
            self.ceiling_drip_audio_timer = (30.0 + rnd() * 60.0) * etm; // 30-90 s
            self.log_ambient("ceiling-drip", Some(cd.x), Some(cd.z), Some(pan));
            let splash_color = splash_sprite_by_weather(self.weather_key());
            self.scuff_flash(splash_color, cd.x, cd.y + 0.02, cd.z, 0.65 * (1.0 - dn * 0.3), 0.04, 0.20);
        }
        // R355 hanging ceiling cable plug spark + a yellow sprite
        self.ceiling_cable_spark_timer -= dt;
        if self.ceiling_cable_spark_timer <= 0.0 && !self.pp.ceiling_cable_plug_positions.is_empty() {
            let (_, cp) = pick_pos(&self.pp.ceiling_cable_plug_positions);
            let (pan, dn) = self.prop_pan(cp, 0.7, 0.8, 6.0);
            let cs_gain = 0.011 * eg * (1.0 - dn * 0.55);
            self.blip(60.0, 0.08, "sawtooth", cs_gain * 0.7, Some(0.0), 0.0, pan);
            self.noise_burst(0.04, 5400.0, 4.0, cs_gain, "highpass", None, 0.02, pan);
            self.noise_burst(0.03, 6400.0, 4.0, cs_gain * 0.6, "highpass", None, 0.08, pan);
            self.scuff_flash(0xffe860, cp.x, cp.y, cp.z, 0.85 * (1.0 - dn * 0.3), 0.055, 0.15);
            self.ceiling_cable_spark_timer = (90.0 + rnd() * 150.0) * etm; // 90-240 s
            self.log_ambient("ceiling-cable-spark", Some(cp.x), Some(cp.z), Some(pan));
        }
        // R357 thermostat HVAC relay click
        self.thermostat_click_timer -= dt;
        if self.thermostat_click_timer <= 0.0 && !self.pp.thermostat_positions.is_empty() {
            let (_, tp) = pick_pos(&self.pp.thermostat_positions);
            let (pan, dn) = self.prop_pan(tp, 0.7, 0.8, 6.0);
            let tp_gain = 0.009 * eg * (1.0 - dn * 0.55);
            self.blip(1100.0, 0.02, "square", tp_gain, None, 0.0, pan);
            self.noise_burst(0.03, 2600.0, 3.0, tp_gain * 0.5, "highpass", None, 0.010, pan);
            self.thermostat_click_timer = (120.0 + rnd() * 120.0) * etm; // 2-4 min
            self.log_ambient("thermostat-click", Some(tp.x), Some(tp.z), Some(pan));
        }
        // R390 the handbell on the floor rings
        self.handbell_chime_timer -= dt;
        if self.handbell_chime_timer <= 0.0 && !self.pp.handbell_positions.is_empty() {
            let (_, hb) = pick_pos(&self.pp.handbell_positions);
            let (pan, dn) = self.prop_pan(hb, 0.7, 0.8, 6.0);
            let hb_gain = 0.011 * eg * (1.0 - dn * 0.55);
            self.blip(2400.0, 0.35, "sine", hb_gain, Some(1600.0), 0.0, pan);
            self.blip(4000.0, 0.15, "sine", hb_gain * 0.4, Some(3200.0), 0.01, pan);
            self.handbell_chime_timer = (180.0 + rnd() * 180.0) * etm; // 3-6 min
            self.log_ambient("handbell-chime", Some(hb.x), Some(hb.z), Some(pan));
        }
        // R402 digital clock mains buzz
        self.digital_clock_buzz_timer -= dt;
        if self.digital_clock_buzz_timer <= 0.0 && !self.pp.digital_clock_positions.is_empty() {
            let (_, dc) = pick_pos(&self.pp.digital_clock_positions);
            let (pan, dn) = self.prop_pan(dc, 0.7, 0.8, 6.0);
            let dcb_gain = 0.008 * eg * (1.0 - dn * 0.55);
            self.blip(60.0, 0.8, "sawtooth", dcb_gain, Some(60.0), 0.0, pan);
            self.blip(120.0, 0.8, "sawtooth", dcb_gain * 0.4, Some(120.0), 0.0, pan);
            self.digital_clock_buzz_timer = (240.0 + rnd() * 240.0) * etm; // 4-8 min
            self.log_ambient("digital-clock-buzz", Some(dc.x), Some(dc.z), Some(pan));
        }
        // R406 the alarm clock goes off, impossibly
        self.alarm_ring_timer -= dt;
        if self.alarm_ring_timer <= 0.0 && !self.pp.alarm_clock_positions.is_empty() {
            let (_, acp) = pick_pos(&self.pp.alarm_clock_positions);
            let (pan, dn) = self.prop_pan(acp, 0.7, 0.8, 6.0);
            let ar_gain = 0.014 * eg * (1.0 - dn * 0.55);
            for br in 0..3 {
                let delay = br as f64 * 0.06;
                self.blip(3200.0, 0.04, "sine", ar_gain, Some(2600.0), delay, pan);
                self.blip(4200.0, 0.04, "sine", ar_gain * 0.5, Some(3400.0), delay, pan);
            }
            self.alarm_ring_timer = (360.0 + rnd() * 360.0) * etm; // 6-12 min
            self.log_ambient("alarm-ring", Some(acp.x), Some(acp.z), Some(pan));
        }
        // R366/R367 off-hook rotary phone dial tone (+ a voice on the line)
        self.dial_tone_timer -= dt;
        if self.dial_tone_timer <= 0.0 && !self.pp.off_hook_rotary_positions.is_empty() {
            let (_, oh) = pick_pos(&self.pp.off_hook_rotary_positions);
            let (pan, dn) = self.prop_pan(oh, 0.7, 0.8, 6.0);
            let dt_gain = 0.008 * eg * (1.0 - dn * 0.55);
            self.blip(350.0, 1.2, "sine", dt_gain, Some(350.0), 0.0, pan);
            self.blip(440.0, 1.2, "sine", dt_gain * 0.9, Some(440.0), 0.0, pan);
            if rnd() < 0.15 {
                let syllable_count = 2 + (rnd() * 2.0).floor() as i32;
                for sy in 0..syllable_count {
                    let syl_base = 320.0 + rnd() * 400.0;
                    let syl_end = syl_base * (0.7 + rnd() * 0.6);
                    let syl_dur = 0.10 + rnd() * 0.10;
                    let syl_delay = 0.30 + sy as f64 * (0.16 + rnd() * 0.10);
                    self.noise_burst(syl_dur, syl_base, 10.0, dt_gain * 0.7, "bandpass", Some(syl_end), syl_delay, pan);
                }
                self.log_ambient("rotary-dial-voice", Some(oh.x), Some(oh.z), Some(pan));
            }
            self.dial_tone_timer = (120.0 + rnd() * 120.0) * etm; // 2-4 min
            self.log_ambient("rotary-dial-tone", Some(oh.x), Some(oh.z), Some(pan));
        }
        // R369/R372/R373 floor drain gurgle (+ a voice from below, + a bubble sprite)
        self.drain_gurgle_timer -= dt;
        if self.drain_gurgle_timer <= 0.0 && !self.pp.drain_grate_positions.is_empty() {
            let (_, dg) = pick_pos(&self.pp.drain_grate_positions);
            let (pan, dn) = self.prop_pan(dg, 0.7, 0.8, 6.0);
            let dg_gain = 0.010 * eg * (1.0 - dn * 0.55);
            self.blip(500.0, 0.14, "sine", dg_gain, Some(260.0), 0.0, pan);
            self.noise_burst(0.05, 800.0, 2.5, dg_gain * 0.4, "lowpass", None, 0.02, pan);
            if rnd() < 0.10 {
                for sy in 0..2 {
                    let syl_base = 220.0 + rnd() * 200.0;
                    let syl_end = syl_base * (0.7 + rnd() * 0.5);
                    let syl_dur = 0.14 + rnd() * 0.10;
                    let syl_delay = 0.20 + sy as f64 * (0.20 + rnd() * 0.10);
                    self.noise_burst(syl_dur, syl_base, 12.0, dg_gain * 0.7, "bandpass", Some(syl_end), syl_delay, pan);
                }
                self.log_ambient("drain-voice", Some(dg.x), Some(dg.z), Some(pan));
            }
            self.scuff_flash(0x80a0c0, dg.x, dg.y + 0.015, dg.z, 0.55 * (1.0 - dn * 0.3), 0.035, 0.25);
            self.drain_gurgle_timer = (120.0 + rnd() * 180.0) * etm; // 2-5 min
            self.log_ambient("drain-gurgle", Some(dg.x), Some(dg.z), Some(pan));
        }
        // R319 bathroom faucet drip ("plink... plink...")
        self.faucet_drip_timer -= dt;
        if self.faucet_drip_timer <= 0.0 && !self.pp.bathroom_bundle_positions.is_empty() {
            let (_, bp) = pick_pos(&self.pp.bathroom_bundle_positions);
            let (pan, dn) = self.prop_pan(bp, 0.7, 0.8, 6.0);
            let drip_gain = 0.012 * eg * (1.0 - dn * 0.55);
            self.blip(1400.0, 0.06, "sine", drip_gain, Some(900.0), 0.0, pan);
            self.noise_burst(0.04, 3200.0, 4.0, drip_gain * 0.4, "highpass", None, 0.005, pan);
            if rnd() < 0.40 {
                let gap = 1.5 + rnd() * 1.0;
                self.blip(1400.0, 0.06, "sine", drip_gain, Some(900.0), gap, pan);
                self.noise_burst(0.04, 3200.0, 4.0, drip_gain * 0.4, "highpass", None, gap + 0.005, pan);
            }
            self.faucet_drip_timer = (120.0 + rnd() * 120.0) * etm; // 2-4 min
            self.log_ambient("faucet-drip", Some(bp.x), Some(bp.z), Some(pan));
        }
        // R325/R326 smashed phone keypad shorting (+ 20% a spark sprite)
        self.broken_phone_spark_timer -= dt;
        if self.broken_phone_spark_timer <= 0.0 && !self.pp.broken_phone_positions.is_empty() {
            let (_, bpk) = pick_pos(&self.pp.broken_phone_positions);
            let (pan, dn) = self.prop_pan(bpk, 0.7, 0.8, 6.0);
            let spark_gain = 0.010 * eg * (1.0 - dn * 0.55);
            self.blip(60.0, 0.10, "sawtooth", spark_gain * 0.7, Some(0.0), 0.0, pan);
            self.noise_burst(0.04, 5200.0, 4.0, spark_gain, "highpass", None, 0.02, pan);
            self.noise_burst(0.03, 6200.0, 4.0, spark_gain * 0.7, "highpass", None, 0.10, pan);
            if rnd() < 0.20 {
                self.scuff_flash(0xffe860, bpk.x, bpk.y, bpk.z, 0.80 * (1.0 - dn * 0.3), 0.05, 0.15);
            }
            self.broken_phone_spark_timer = (120.0 + rnd() * 180.0) * etm; // 2-5 min
            self.log_ambient("broken-phone-spark", Some(bpk.x), Some(bpk.z), Some(pan));
        }
    }
}
