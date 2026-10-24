//! updateAmbient, part 4: household props R678–R729 (lines 64410–64829) and
//! parlour props R732–R767 (64830–65160). Same shape as ambient_props.rs:
//! all of these are proximity-gated. See ambient.rs for the overview.
use crate::ambient::{pick_pos, AmbMix};
use crate::prelude::*;

impl Game {
    /// R678–R729: cookie tin … turntable (lines 64410–64829).
    pub(crate) fn amb_props_household(&mut self, dt: f64, m: &AmbMix) {
        let eg = m.gain;
        let etm = m.timer_mul;
        // R678 cookie tin lid rattle (< 3.5 m)
        self.cookie_tin_rattle_timer -= dt;
        if self.cookie_tin_rattle_timer <= 0.0 && !self.pp.cookie_tin_positions.is_empty() {
            let (_, ct) = pick_pos(&self.pp.cookie_tin_positions);
            if self.prop_d2(ct) < 12.25 {
                let (pan, dn) = self.prop_pan(ct, 0.55, 0.7, 3.5);
                let ctg = 0.009 * eg * (1.0 - dn * 0.55);
                self.blip(1800.0, 0.03, "square", ctg * 0.7, Some(1600.0), 0.0, pan);
                self.blip(2100.0, 0.025, "square", ctg * 0.5, Some(1900.0), 0.05, pan);
                self.noise_burst(0.18, 2400.0, 0.9, ctg * 0.35, "bandpass", Some(1200.0), 0.08, pan);
                self.log_ambient("cookie-tin-rattle", Some(ct.x), Some(ct.z), Some(pan));
            }
            self.cookie_tin_rattle_timer = (45.0 + rnd() * 45.0) * etm;
        }
        // R681 kitchen wall clock single tick (< 3 m)
        self.kitchen_clock_tick_timer -= dt;
        if self.kitchen_clock_tick_timer <= 0.0 && !self.pp.kitchen_clock_positions.is_empty() {
            let (_, kc) = pick_pos(&self.pp.kitchen_clock_positions);
            if self.prop_d2(kc) < 9.0 {
                let (pan, dn) = self.prop_pan(kc, 0.5, 0.65, 3.0);
                let kcg = 0.006 * eg * (1.0 - dn * 0.6);
                self.blip(4200.0, 0.02, "square", kcg, Some(3800.0), 0.0, pan);
                self.log_ambient("kitchen-clock-tick", Some(kc.x), Some(kc.z), Some(pan));
            }
            self.kitchen_clock_tick_timer = (60.0 + rnd() * 60.0) * etm;
        }
        // R694 off-hook phone dial-tone hum, US 350 + 440 Hz (< 3.5 m)
        self.off_hook_phone_hum_timer -= dt;
        if self.off_hook_phone_hum_timer <= 0.0 && !self.pp.off_hook_phone_positions.is_empty() {
            let (_, oh) = pick_pos(&self.pp.off_hook_phone_positions);
            if self.prop_d2(oh) < 12.25 {
                let (pan, dn) = self.prop_pan(oh, 0.55, 0.7, 3.5);
                let ohg = 0.009 * eg * (1.0 - dn * 0.6);
                self.blip(350.0, 1.8, "sine", ohg * 0.7, None, 0.0, pan);
                self.blip(440.0, 1.8, "sine", ohg * 0.7, None, 0.0, pan);
                self.log_ambient("dial-tone-hum", Some(oh.x), Some(oh.z), Some(pan));
            }
            self.off_hook_phone_hum_timer = (15.0 + rnd() * 15.0) * etm;
        }
        // R695 hairdryer motor hum (< 4 m)
        self.hairdryer_hum_timer -= dt;
        if self.hairdryer_hum_timer <= 0.0 && !self.pp.hairdryer_positions.is_empty() {
            let (_, hd) = pick_pos(&self.pp.hairdryer_positions);
            if self.prop_d2(hd) < 16.0 {
                let (pan, dn) = self.prop_pan(hd, 0.5, 0.65, 4.0);
                let hdg = 0.008 * eg * (1.0 - dn * 0.55);
                self.noise_burst(1.8, 900.0, 1.0, hdg, "bandpass", Some(500.0), 0.0, pan);
                self.blip(720.0, 1.8, "sine", hdg * 0.4, None, 0.0, pan);
                self.log_ambient("hairdryer-hum", Some(hd.x), Some(hd.z), Some(pan));
            }
            self.hairdryer_hum_timer = (10.0 + rnd() * 10.0) * etm;
        }
        // R699 sewing machine stitch clicks at ~4 Hz (< 4 m)
        self.sewing_machine_click_timer -= dt;
        if self.sewing_machine_click_timer <= 0.0 && !self.pp.sewing_machine_positions.is_empty() {
            let (_, sm) = pick_pos(&self.pp.sewing_machine_positions);
            if self.prop_d2(sm) < 16.0 {
                let (pan, dn) = self.prop_pan(sm, 0.5, 0.65, 4.0);
                let smg = 0.008 * eg * (1.0 - dn * 0.55);
                let click_count = 4 + (rnd() * 3.0).floor() as i32;
                for i in 0..click_count {
                    let i = i as f64;
                    self.blip(2400.0, 0.02, "square", smg, Some(2200.0), i * 0.24, pan);
                    self.blip(1600.0, 0.015, "square", smg * 0.5, None, i * 0.24 + 0.008, pan);
                }
                self.log_ambient("sewing-machine-click", Some(sm.x), Some(sm.z), Some(pan));
            }
            self.sewing_machine_click_timer = (8.0 + rnd() * 10.0) * etm;
        }
        // R701 herb sieve brushed-metal scrape (< 3 m)
        self.herb_sieve_scrape_timer -= dt;
        if self.herb_sieve_scrape_timer <= 0.0 && !self.pp.herb_sieve_positions.is_empty() {
            let (_, hs) = pick_pos(&self.pp.herb_sieve_positions);
            if self.prop_d2(hs) < 9.0 {
                let (pan, dn) = self.prop_pan(hs, 0.5, 0.65, 3.0);
                let hsg = 0.008 * eg * (1.0 - dn * 0.6);
                self.noise_burst(0.28, 3200.0, 1.2, hsg, "bandpass", Some(1600.0), 0.0, pan);
                self.blip(4800.0, 0.015, "square", hsg * 0.5, Some(4400.0), 0.0, pan);
                self.log_ambient("herb-sieve-scrape", Some(hs.x), Some(hs.z), Some(pan));
            }
            self.herb_sieve_scrape_timer = (40.0 + rnd() * 40.0) * etm;
        }
        // R706 vacuum cleaner motor whine (< 4.5 m)
        self.vacuum_whine_timer -= dt;
        if self.vacuum_whine_timer <= 0.0 && !self.pp.vacuum_positions.is_empty() {
            let (_, vc) = pick_pos(&self.pp.vacuum_positions);
            if self.prop_d2(vc) < 20.25 {
                let (pan, dn) = self.prop_pan(vc, 0.5, 0.65, 4.5);
                let vcg = 0.010 * eg * (1.0 - dn * 0.55);
                self.noise_burst(1.5, 350.0, 1.6, vcg, "bandpass", Some(180.0), 0.0, pan);
                self.blip(140.0, 1.5, "sine", vcg * 0.5, None, 0.0, pan);
                self.log_ambient("vacuum-whine", Some(vc.x), Some(vc.z), Some(pan));
            }
            self.vacuum_whine_timer = (30.0 + rnd() * 30.0) * etm;
        }
        // R707 mop bucket water slosh (< 3 m)
        self.mop_bucket_slosh_timer -= dt;
        if self.mop_bucket_slosh_timer <= 0.0 && !self.pp.mop_bucket_positions.is_empty() {
            let (_, mb) = pick_pos(&self.pp.mop_bucket_positions);
            if self.prop_d2(mb) < 9.0 {
                let (pan, dn) = self.prop_pan(mb, 0.55, 0.65, 3.0);
                let mbg = 0.008 * eg * (1.0 - dn * 0.6);
                self.noise_burst(0.45, 550.0, 0.7, mbg, "bandpass", Some(400.0), 0.0, pan);
                self.blip(180.0, 0.35, "sine", mbg * 0.3, Some(140.0), 0.20, pan);
                self.log_ambient("mop-bucket-slosh", Some(mb.x), Some(mb.z), Some(pan));
            }
            self.mop_bucket_slosh_timer = (50.0 + rnd() * 50.0) * etm;
        }
        // R712 wristwatch quartz tick (< 2 m)
        self.wristwatch_tick_timer -= dt;
        if self.wristwatch_tick_timer <= 0.0 && !self.pp.wristwatch_positions.is_empty() {
            let (_, ww) = pick_pos(&self.pp.wristwatch_positions);
            if self.prop_d2(ww) < 4.0 {
                let (pan, dn) = self.prop_pan(ww, 0.4, 0.55, 2.0);
                let wwg = 0.005 * eg * (1.0 - dn * 0.7);
                self.blip(3600.0, 0.012, "square", wwg, Some(3200.0), 0.0, pan);
                self.log_ambient("wristwatch-tick", Some(ww.x), Some(ww.z), Some(pan));
            }
            self.wristwatch_tick_timer = (3.0 + rnd() * 4.0) * etm;
        }
        // R712 hot iron steam hiss (< 3.5 m)
        self.iron_hiss_timer -= dt;
        if self.iron_hiss_timer <= 0.0 && !self.pp.hot_iron_positions.is_empty() {
            let (_, hi) = pick_pos(&self.pp.hot_iron_positions);
            if self.prop_d2(hi) < 12.25 {
                let (pan, dn) = self.prop_pan(hi, 0.5, 0.65, 3.5);
                let hig = 0.007 * eg * (1.0 - dn * 0.6);
                self.noise_burst(0.85, 4200.0, 1.0, hig, "highpass", None, 0.0, pan);
                self.log_ambient("iron-hiss", Some(hi.x), Some(hi.z), Some(pan));
            }
            self.iron_hiss_timer = (20.0 + rnd() * 20.0) * etm;
        }
        // R717 grandfather clock tick-tock (< 4 m)
        self.grandfather_clock_tick_timer -= dt;
        if self.grandfather_clock_tick_timer <= 0.0 && !self.pp.grandfather_clock_positions.is_empty() {
            let (_, gc) = pick_pos(&self.pp.grandfather_clock_positions);
            if self.prop_d2(gc) < 16.0 {
                let (pan, dn) = self.prop_pan(gc, 0.5, 0.6, 4.0);
                let gcg = 0.009 * eg * (1.0 - dn * 0.6);
                self.blip(220.0, 0.06, "square", gcg, Some(200.0), 0.0, pan);
                self.blip(180.0, 0.06, "square", gcg, Some(160.0), 1.0, pan);
                self.log_ambient("grandfather-clock-tick", Some(gc.x), Some(gc.z), Some(pan));
            }
            self.grandfather_clock_tick_timer = (2.0 + rnd()) * etm;
        }
        // R718 rocking chair wood creak (< 3.5 m)
        self.rocking_chair_creak_timer -= dt;
        if self.rocking_chair_creak_timer <= 0.0 && !self.pp.rocking_chair_positions.is_empty() {
            let (_, rc) = pick_pos(&self.pp.rocking_chair_positions);
            if self.prop_d2(rc) < 12.25 {
                let (pan, dn) = self.prop_pan(rc, 0.5, 0.65, 3.5);
                let rcg = 0.007 * eg * (1.0 - dn * 0.6);
                self.noise_burst(0.35, 280.0, 1.2, rcg, "bandpass", Some(160.0), 0.0, pan);
                self.blip(120.0, 0.25, "sine", rcg * 0.4, Some(90.0), 0.10, pan);
                self.log_ambient("rocker-creak", Some(rc.x), Some(rc.z), Some(pan));
            }
            self.rocking_chair_creak_timer = (4.0 + rnd() * 4.0) * etm;
        }
        // R718 tall lamp filament buzz (< 2.5 m)
        self.tall_lamp_buzz_timer -= dt;
        if self.tall_lamp_buzz_timer <= 0.0 && !self.pp.tall_lamp_positions.is_empty() {
            let (_, tl) = pick_pos(&self.pp.tall_lamp_positions);
            if self.prop_d2(tl) < 6.25 {
                let (pan, dn) = self.prop_pan(tl, 0.45, 0.55, 2.5);
                let tlg = 0.005 * eg * (1.0 - dn * 0.7);
                self.blip(60.0, 1.2, "sine", tlg, None, 0.0, pan);
                self.blip(120.0, 1.2, "sine", tlg * 0.4, None, 0.0, pan);
                self.log_ambient("lamp-buzz", Some(tl.x), Some(tl.z), Some(pan));
            }
            self.tall_lamp_buzz_timer = (12.0 + rnd() * 13.0) * etm;
        }
        // R721 hanging colander metallic tap (< 3.5 m)
        self.colander_tap_timer -= dt;
        if self.colander_tap_timer <= 0.0 && !self.pp.colander_positions.is_empty() {
            let (_, cl) = pick_pos(&self.pp.colander_positions);
            if self.prop_d2(cl) < 12.25 {
                let (pan, dn) = self.prop_pan(cl, 0.55, 0.7, 3.5);
                let clg = 0.007 * eg * (1.0 - dn * 0.6);
                self.blip(2800.0, 0.025, "square", clg, Some(2500.0), 0.0, pan);
                self.blip(2200.0, 0.020, "square", clg * 0.5, Some(2000.0), 0.03, pan);
                self.noise_burst(0.10, 3000.0, 1.4, clg * 0.3, "bandpass", Some(2400.0), 0.05, pan);
                self.log_ambient("colander-tap", Some(cl.x), Some(cl.z), Some(pan));
            }
            self.colander_tap_timer = (30.0 + rnd() * 30.0) * etm;
        }
        // R723 pet water bowl slosh (< 2.5 m)
        self.pet_bowl_slosh_timer -= dt;
        if self.pet_bowl_slosh_timer <= 0.0 && !self.pp.pet_bowl_positions.is_empty() {
            let (_, pb) = pick_pos(&self.pp.pet_bowl_positions);
            if self.prop_d2(pb) < 6.25 {
                let (pan, dn) = self.prop_pan(pb, 0.5, 0.6, 2.5);
                let pbg = 0.006 * eg * (1.0 - dn * 0.65);
                self.noise_burst(0.30, 750.0, 0.9, pbg, "bandpass", Some(550.0), 0.0, pan);
                self.blip(320.0, 0.20, "sine", pbg * 0.3, Some(260.0), 0.15, pan);
                self.log_ambient("pet-bowl-slosh", Some(pb.x), Some(pb.z), Some(pan));
            }
            self.pet_bowl_slosh_timer = (40.0 + rnd() * 40.0) * etm;
        }
        // R725 music box C-E-G-A-G melody (< 3 m)
        self.music_box_melody_timer -= dt;
        if self.music_box_melody_timer <= 0.0 && !self.pp.music_box_positions.is_empty() {
            let (_, mb) = pick_pos(&self.pp.music_box_positions);
            if self.prop_d2(mb) < 9.0 {
                let (pan, dn) = self.prop_pan(mb, 0.5, 0.65, 3.0);
                let mbg = 0.006 * eg * (1.0 - dn * 0.65);
                const NOTES: [f64; 5] = [1046.0, 1319.0, 1568.0, 1760.0, 1568.0];
                for i in 0..5 {
                    self.blip(NOTES[i], 0.18, "sine", mbg, Some(NOTES[i] * 0.995), i as f64 * 0.30, pan);
                }
                self.log_ambient("music-box-melody", Some(mb.x), Some(mb.z), Some(pan));
            }
            self.music_box_melody_timer = (20.0 + rnd() * 20.0) * etm;
        }
        // R726 grandfather clock hourly chime (reuses the R717 positions, < 5 m)
        self.grandfather_chime_timer -= dt;
        if self.grandfather_chime_timer <= 0.0 && !self.pp.grandfather_clock_positions.is_empty() {
            let (_, gc) = pick_pos(&self.pp.grandfather_clock_positions);
            if self.prop_d2(gc) < 25.0 {
                let (pan, dn) = self.prop_pan(gc, 0.4, 0.55, 5.0);
                let chimeg = 0.012 * eg * (1.0 - dn * 0.55);
                let strike_count = 2 + (rnd() * 3.0).floor() as i32;
                for i in 0..strike_count {
                    let base_freq = 165.0 + (i % 2) as f64 * 15.0;   // ~E3 alternating
                    let i = i as f64;
                    self.blip(base_freq, 0.90, "sine", chimeg * 0.7, Some(base_freq * 0.90), i * 1.4, pan);
                    self.blip(base_freq * 2.0, 0.60, "sine", chimeg * 0.3, Some(base_freq * 1.85), i * 1.4 + 0.02, pan);
                }
                self.log_ambient("grandfather-chime", Some(gc.x), Some(gc.z), Some(pan));
            }
            self.grandfather_chime_timer = (180.0 + rnd() * 180.0) * etm;
        }
        // R729 turntable vinyl hiss (< 3 m)
        self.turntable_hiss_timer -= dt;
        if self.turntable_hiss_timer <= 0.0 && !self.pp.turntable_positions.is_empty() {
            let (_, tt) = pick_pos(&self.pp.turntable_positions);
            if self.prop_d2(tt) < 9.0 {
                let (pan, dn) = self.prop_pan(tt, 0.5, 0.6, 3.0);
                let ttg = 0.006 * eg * (1.0 - dn * 0.65);
                self.noise_burst(1.4, 5200.0, 0.8, ttg, "bandpass", Some(4200.0), 0.0, pan);
                // occasional pop tick
                if rnd() < 0.6 {
                    self.blip(4600.0, 0.01, "square", ttg * 0.7, Some(4200.0), 0.3 + rnd() * 0.6, pan);
                }
                self.log_ambient("turntable-hiss", Some(tt.x), Some(tt.z), Some(pan));
            }
            self.turntable_hiss_timer = (6.0 + rnd() * 8.0) * etm;
        }
    }

    /// R732–R767: chalkboard … hobby horse (lines 64830–65160).
    pub(crate) fn amb_props_parlour(&mut self, dt: f64, m: &AmbMix) {
        let eg = m.gain;
        let etm = m.timer_mul;
        // R732 chalkboard chalk scratch — the invisible writer (< 3.5 m)
        self.chalkboard_scratch_timer -= dt;
        if self.chalkboard_scratch_timer <= 0.0 && !self.pp.chalkboard_positions.is_empty() {
            let (_, cb) = pick_pos(&self.pp.chalkboard_positions);
            if self.prop_d2(cb) < 12.25 {
                let (pan, dn) = self.prop_pan(cb, 0.55, 0.7, 3.5);
                let cbg = 0.007 * eg * (1.0 - dn * 0.6);
                self.blip(4800.0, 0.06, "square", cbg * 0.6, Some(3800.0), 0.0, pan);
                self.noise_burst(0.12, 5500.0, 1.0, cbg * 0.4, "highpass", None, 0.03, pan);
                self.log_ambient("chalkboard-scratch", Some(cb.x), Some(cb.z), Some(pan));
            }
            self.chalkboard_scratch_timer = (45.0 + rnd() * 45.0) * etm;
        }
        // R735 kitchen calendar page turn (< 3.5 m)
        self.kitchen_calendar_turn_timer -= dt;
        if self.kitchen_calendar_turn_timer <= 0.0 && !self.pp.kitchen_calendar_positions.is_empty() {
            let (_, kc) = pick_pos(&self.pp.kitchen_calendar_positions);
            if self.prop_d2(kc) < 12.25 {
                let (pan, dn) = self.prop_pan(kc, 0.55, 0.7, 3.5);
                let kcg = 0.006 * eg * (1.0 - dn * 0.6);
                self.noise_burst(0.55, 3800.0, 0.7, kcg, "highpass", None, 0.0, pan);
                self.blip(1200.0, 0.20, "triangle", kcg * 0.3, Some(900.0), 0.35, pan);
                self.log_ambient("calendar-turn", Some(kc.x), Some(kc.z), Some(pan));
            }
            self.kitchen_calendar_turn_timer = (90.0 + rnd() * 90.0) * etm;
        }
        // R738 telescope focus-knob clicks (< 3 m)
        self.telescope_click_timer -= dt;
        if self.telescope_click_timer <= 0.0 && !self.pp.telescope_positions.is_empty() {
            let (_, ts) = pick_pos(&self.pp.telescope_positions);
            if self.prop_d2(ts) < 9.0 {
                let (pan, dn) = self.prop_pan(ts, 0.5, 0.6, 3.0);
                let tsg = 0.005 * eg * (1.0 - dn * 0.65);
                let click_count = 2 + (rnd() * 2.0).floor() as i32;
                for i in 0..click_count {
                    self.blip(1800.0, 0.012, "square", tsg, Some(1600.0), i as f64 * 0.14, pan);
                }
                self.log_ambient("telescope-click", Some(ts.x), Some(ts.z), Some(pan));
            }
            self.telescope_click_timer = (20.0 + rnd() * 20.0) * etm;
        }
        // R741 cuckoo clock 2–4 "cuckoo" calls (< 4 m)
        self.cuckoo_call_timer_r741 -= dt;
        if self.cuckoo_call_timer_r741 <= 0.0 && !self.pp.cuckoo_clock_positions_r741.is_empty() {
            let (_, cc) = pick_pos(&self.pp.cuckoo_clock_positions_r741);
            if self.prop_d2(cc) < 16.0 {
                let (pan, dn) = self.prop_pan(cc, 0.5, 0.65, 4.0);
                let ccg = 0.010 * eg * (1.0 - dn * 0.55);
                let call_count = 2 + (rnd() * 3.0).floor() as i32;
                for i in 0..call_count {
                    let i = i as f64;
                    self.blip(660.0, 0.18, "sine", ccg, Some(640.0), i * 0.55, pan);          // "cu" (E5)
                    self.blip(520.0, 0.28, "sine", ccg, Some(500.0), i * 0.55 + 0.22, pan);   // "coo" (C5)
                }
                self.log_ambient("cuckoo-call", Some(cc.x), Some(cc.z), Some(pan));
            }
            self.cuckoo_call_timer_r741 = (120.0 + rnd() * 180.0) * etm;
        }
        // R744 lit fireplace crackle (< 5 m)
        self.fireplace_crackle_timer -= dt;
        if self.fireplace_crackle_timer <= 0.0 && !self.pp.lit_fireplace_positions.is_empty() {
            let (_, fp) = pick_pos(&self.pp.lit_fireplace_positions);
            if self.prop_d2(fp) < 25.0 {
                let (pan, dn) = self.prop_pan(fp, 0.5, 0.6, 5.0);
                let fpg = 0.010 * eg * (1.0 - dn * 0.55);
                let pop_count = 2 + (rnd() * 4.0).floor() as i32;
                for i in 0..pop_count {
                    let i = i as f64;
                    self.noise_burst(0.06, 800.0, 0.8, fpg, "bandpass", Some(500.0), i * 0.15 + rnd() * 0.10, pan);
                    // occasional higher snap
                    if rnd() < 0.4 {
                        self.blip(1400.0, 0.015, "square", fpg * 0.5, Some(1200.0), i * 0.15 + 0.03, pan);
                    }
                }
                self.log_ambient("fireplace-crackle", Some(fp.x), Some(fp.z), Some(pan));
            }
            self.fireplace_crackle_timer = (4.0 + rnd() * 6.0) * etm;
        }
        // R744 aquarium filter hum + bubble pops (< 3 m)
        self.aquarium_hum_timer -= dt;
        if self.aquarium_hum_timer <= 0.0 && !self.pp.aquarium_positions.is_empty() {
            let (_, aq) = pick_pos(&self.pp.aquarium_positions);
            if self.prop_d2(aq) < 9.0 {
                let (pan, dn) = self.prop_pan(aq, 0.5, 0.6, 3.0);
                let aqg = 0.006 * eg * (1.0 - dn * 0.6);
                self.blip(90.0, 1.5, "sine", aqg * 0.5, None, 0.0, pan);
                let pop_count = 2 + (rnd() * 3.0).floor() as i32;
                for i in 0..pop_count {
                    self.blip(4200.0, 0.008, "square", aqg * 0.4, Some(3800.0), i as f64 * 0.20 + rnd() * 0.10, pan);
                }
                self.log_ambient("aquarium-hum", Some(aq.x), Some(aq.z), Some(pan));
            }
            self.aquarium_hum_timer = (8.0 + rnd() * 7.0) * etm;
        }
        // R747 wingback armchair creak — an invisible sitter shifts (< 3.5 m)
        self.wingback_creak_timer -= dt;
        if self.wingback_creak_timer <= 0.0 && !self.pp.wingback_armchair_positions.is_empty() {
            let (_, wb) = pick_pos(&self.pp.wingback_armchair_positions);
            if self.prop_d2(wb) < 12.25 {
                let (pan, dn) = self.prop_pan(wb, 0.6, 0.7, 3.5);
                let wbg = 0.014 * eg * (1.0 - dn * 0.55);
                self.blip(180.0, 0.35, "sawtooth", wbg * 0.5, Some(130.0), 0.0, pan);
                self.blip(320.0, 0.28, "triangle", wbg * 0.35, Some(260.0), 0.08, pan);
                self.noise_burst(0.12, 1200.0, 0.6, wbg * 0.25, "bandpass", Some(800.0), 0.20, pan);
                self.log_ambient("wingback-creak", Some(wb.x), Some(wb.z), Some(pan));
            }
            self.wingback_creak_timer = (60.0 + rnd() * 60.0) * etm;
        }
        // R755 frozen pocket watch still ticking (< 2.5 m)
        self.pocket_watch_tick_timer -= dt;
        if self.pocket_watch_tick_timer <= 0.0 && !self.pp.pocket_watch_positions_r755.is_empty() {
            let (_, pw) = pick_pos(&self.pp.pocket_watch_positions_r755);
            if self.prop_d2(pw) < 6.25 {
                let (pan, dn) = self.prop_pan(pw, 0.5, 0.6, 2.5);
                let pwg = 0.008 * eg * (1.0 - dn * 0.6);
                self.blip(3200.0, 0.012, "square", pwg, Some(2800.0), 0.0, pan);
                self.blip(2400.0, 0.014, "square", pwg * 0.9, Some(2000.0), 0.50, pan);
                self.log_ambient("watch-tick", Some(pw.x), Some(pw.z), Some(pan));
            }
            self.pocket_watch_tick_timer = (3.0 + rnd() * 3.0) * etm;
        }
        // R757 servant's bell pull brass ring (< 6 m)
        self.bell_pull_ring_timer -= dt;
        if self.bell_pull_ring_timer <= 0.0 && !self.pp.bell_pull_positions.is_empty() {
            let (_, bpr) = pick_pos(&self.pp.bell_pull_positions);
            if self.prop_d2(bpr) < 36.0 {
                let (pan, dn) = self.prop_pan(bpr, 0.6, 0.7, 6.0);
                let bpg = 0.014 * eg * (1.0 - dn * 0.55);
                self.blip(1200.0, 0.35, "sine", bpg, None, 0.0, pan);
                self.blip(1800.0, 0.28, "sine", bpg * 0.6, None, 0.02, pan);
                self.noise_burst(0.02, 800.0, 0.7, bpg * 0.4, "bandpass", Some(400.0), 0.0, pan);
                self.blip(900.0, 0.22, "sine", bpg * 0.35, None, 0.15, pan);
                self.log_ambient("bell-pull-ring", Some(bpr.x), Some(bpr.z), Some(pan));
            }
            self.bell_pull_ring_timer = (120.0 + rnd() * 120.0) * etm;
        }
        // R761 servants'-hall bell rack ding (< 5 m)
        self.bell_rack_ding_timer -= dt;
        if self.bell_rack_ding_timer <= 0.0 && !self.pp.bell_rack_positions.is_empty() {
            let (_, brr) = pick_pos(&self.pp.bell_rack_positions);
            if self.prop_d2(brr) < 25.0 {
                let (pan, dn) = self.prop_pan(brr, 0.6, 0.7, 5.0);
                let brg = 0.012 * eg * (1.0 - dn * 0.55);
                self.blip(2600.0, 0.25, "sine", brg, None, 0.0, pan);
                self.blip(3900.0, 0.20, "sine", brg * 0.55, None, 0.010, pan);
                self.blip(2400.0, 0.15, "sine", brg * 0.30, Some(2200.0), 0.08, pan);
                self.blip(2400.0, 0.12, "sine", brg * 0.20, Some(2600.0), 0.18, pan);
                self.log_ambient("bell-rack-ding", Some(brr.x), Some(brr.z), Some(pan));
            }
            self.bell_rack_ding_timer = (90.0 + rnd() * 90.0) * etm;
        }
        // R763 chess piece move — the abandoned game continues (< 3 m)
        self.chess_move_click_timer -= dt;
        if self.chess_move_click_timer <= 0.0 && !self.pp.chess_set_positions.is_empty() {
            let (_, csp) = pick_pos(&self.pp.chess_set_positions);
            if self.prop_d2(csp) < 9.0 {
                let (pan, dn) = self.prop_pan(csp, 0.5, 0.6, 3.0);
                let csg = 0.014 * eg * (1.0 - dn * 0.55);
                let is_full_move = rnd() < 0.5;
                self.noise_burst(0.015, 400.0, 0.6, csg, "bandpass", Some(200.0), 0.0, pan);
                self.blip(220.0, 0.020, "square", csg * 0.6, Some(180.0), 0.005, pan);
                if is_full_move {
                    let delay = 0.6 + rnd() * 0.6;
                    self.noise_burst(0.015, 400.0, 0.6, csg, "bandpass", Some(200.0), delay, pan);
                    self.blip(240.0, 0.022, "square", csg * 0.6, Some(200.0), delay + 0.005, pan);
                }
                self.log_ambient("chess-move", Some(csp.x), Some(csp.z), Some(pan));
            }
            self.chess_move_click_timer = (45.0 + rnd() * 45.0) * etm;
        }
        // R767 wheeled hobby horse roll, decelerating clicks (< 4 m)
        self.hobby_horse_roll_timer -= dt;
        if self.hobby_horse_roll_timer <= 0.0 && !self.pp.hobby_horse_wheeled_positions.is_empty() {
            let (_, hhp) = pick_pos(&self.pp.hobby_horse_wheeled_positions);
            if self.prop_d2(hhp) < 16.0 {
                let (pan, dn) = self.prop_pan(hhp, 0.6, 0.7, 4.0);
                let hhg = 0.010 * eg * (1.0 - dn * 0.55);
                let click_count = 3 + (rnd() * 3.0).floor() as i32;
                for i in 0..click_count {
                    let i = i as f64;
                    let delay = i * (0.10 + i * 0.03);
                    self.noise_burst(0.012, 300.0, 0.5, hhg * (1.0 - i * 0.10), "bandpass", Some(150.0), delay, pan);
                    self.blip(180.0, 0.015, "square", hhg * 0.5 * (1.0 - i * 0.10), Some(150.0), delay + 0.003, pan);
                }
                self.log_ambient("horse-roll", Some(hhp.x), Some(hhp.z), Some(pan));
            }
            self.hobby_horse_roll_timer = (90.0 + rnd() * 90.0) * etm;
        }
    }
}
