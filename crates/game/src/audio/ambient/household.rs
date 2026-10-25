//! `update_ambient`, part 4: household props R678–R729 and parlour props
//! R732–R767. Same shape as fixtures.rs: all of these are proximity-gated.
//! See `audio::ambient` (mod.rs) for the overview.
use crate::audio::ambient::AmbMix;
use crate::prelude::*;

impl Game {
    /// R678–R729: cookie tin … turntable.
    pub(crate) fn amb_props_household(&mut self, dt: f64, m: &AmbMix) {
        let eg = m.gain;
        let etm = m.timer_mul;
        // R678 cookie tin lid rattle (< 3.5 m)
        if let Some(h) = self.prop_due(|g| &mut g.cookie_tin_rattle_timer, |g| &g.pp.cookie_tin_positions, 3.5, dt, m) {
            let ctg = 0.009 * eg * h.g;
            self.blip(1800.0, 0.03, "square", ctg * 0.7, Some(1600.0), 0.0, h.pan);
            self.blip(2100.0, 0.025, "square", ctg * 0.5, Some(1900.0), 0.05, h.pan);
            self.noise_burst(0.18, 2400.0, 0.9, ctg * 0.35, "bandpass", Some(1200.0), 0.08, h.pan);
            self.cookie_tin_rattle_timer = (45.0 + rnd() * 45.0) * etm;
            self.log_ambient("cookie-tin-rattle", Some(h.p.x), Some(h.p.z), Some(h.pan));
        }
        // R681 kitchen wall clock single tick (< 3 m)
        if let Some(h) = self.prop_due(|g| &mut g.kitchen_clock_tick_timer, |g| &g.pp.kitchen_clock_positions, 3.0, dt, m) {
            let kcg = 0.006 * eg * h.g;
            self.blip(4200.0, 0.02, "square", kcg, Some(3800.0), 0.0, h.pan);
            self.kitchen_clock_tick_timer = (60.0 + rnd() * 60.0) * etm;
            self.log_ambient("kitchen-clock-tick", Some(h.p.x), Some(h.p.z), Some(h.pan));
        }
        // R694 off-hook phone dial-tone hum, US 350 + 440 Hz (< 3.5 m)
        if let Some(h) = self.prop_due(|g| &mut g.off_hook_phone_hum_timer, |g| &g.pp.off_hook_phone_positions, 3.5, dt, m) {
            let ohg = 0.009 * eg * h.g;
            self.blip(350.0, 1.8, "sine", ohg * 0.7, None, 0.0, h.pan);
            self.blip(440.0, 1.8, "sine", ohg * 0.7, None, 0.0, h.pan);
            self.off_hook_phone_hum_timer = (15.0 + rnd() * 15.0) * etm;
            self.log_ambient("dial-tone-hum", Some(h.p.x), Some(h.p.z), Some(h.pan));
        }
        // R695 hairdryer motor hum (< 4 m)
        if let Some(h) = self.prop_due(|g| &mut g.hairdryer_hum_timer, |g| &g.pp.hairdryer_positions, 4.0, dt, m) {
            let hdg = 0.008 * eg * h.g;
            self.noise_burst(1.8, 900.0, 1.0, hdg, "bandpass", Some(500.0), 0.0, h.pan);
            self.blip(720.0, 1.8, "sine", hdg * 0.4, None, 0.0, h.pan);
            self.hairdryer_hum_timer = (10.0 + rnd() * 10.0) * etm;
            self.log_ambient("hairdryer-hum", Some(h.p.x), Some(h.p.z), Some(h.pan));
        }
        // R699 sewing machine stitch clicks at ~4 Hz (< 4 m)
        if let Some(h) = self.prop_due(|g| &mut g.sewing_machine_click_timer, |g| &g.pp.sewing_machine_positions, 4.0, dt, m) {
            let smg = 0.008 * eg * h.g;
            let click_count = 4 + (rnd() * 3.0).floor() as i32;
            for i in 0..click_count {
                let i = i as f64;
                self.blip(2400.0, 0.02, "square", smg, Some(2200.0), i * 0.24, h.pan);
                self.blip(1600.0, 0.015, "square", smg * 0.5, None, i * 0.24 + 0.008, h.pan);
            }
            self.sewing_machine_click_timer = (8.0 + rnd() * 10.0) * etm;
            self.log_ambient("sewing-machine-click", Some(h.p.x), Some(h.p.z), Some(h.pan));
        }
        // R701 herb sieve brushed-metal scrape (< 3 m)
        if let Some(h) = self.prop_due(|g| &mut g.herb_sieve_scrape_timer, |g| &g.pp.herb_sieve_positions, 3.0, dt, m) {
            let hsg = 0.008 * eg * h.g;
            self.noise_burst(0.28, 3200.0, 1.2, hsg, "bandpass", Some(1600.0), 0.0, h.pan);
            self.blip(4800.0, 0.015, "square", hsg * 0.5, Some(4400.0), 0.0, h.pan);
            self.herb_sieve_scrape_timer = (40.0 + rnd() * 40.0) * etm;
            self.log_ambient("herb-sieve-scrape", Some(h.p.x), Some(h.p.z), Some(h.pan));
        }
        // R706 vacuum cleaner motor whine (< 4.5 m)
        if let Some(h) = self.prop_due(|g| &mut g.vacuum_whine_timer, |g| &g.pp.vacuum_positions, 4.5, dt, m) {
            let vcg = 0.010 * eg * h.g;
            self.noise_burst(1.5, 350.0, 1.6, vcg, "bandpass", Some(180.0), 0.0, h.pan);
            self.blip(140.0, 1.5, "sine", vcg * 0.5, None, 0.0, h.pan);
            self.vacuum_whine_timer = (30.0 + rnd() * 30.0) * etm;
            self.log_ambient("vacuum-whine", Some(h.p.x), Some(h.p.z), Some(h.pan));
        }
        // R707 mop bucket water slosh (< 3 m)
        if let Some(h) = self.prop_due(|g| &mut g.mop_bucket_slosh_timer, |g| &g.pp.mop_bucket_positions, 3.0, dt, m) {
            let mbg = 0.008 * eg * h.g;
            self.noise_burst(0.45, 550.0, 0.7, mbg, "bandpass", Some(400.0), 0.0, h.pan);
            self.blip(180.0, 0.35, "sine", mbg * 0.3, Some(140.0), 0.20, h.pan);
            self.mop_bucket_slosh_timer = (50.0 + rnd() * 50.0) * etm;
            self.log_ambient("mop-bucket-slosh", Some(h.p.x), Some(h.p.z), Some(h.pan));
        }
        // R712 wristwatch quartz tick (< 2 m)
        if let Some(h) = self.prop_due(|g| &mut g.wristwatch_tick_timer, |g| &g.pp.wristwatch_positions, 2.0, dt, m) {
            let wwg = 0.005 * eg * h.g;
            self.blip(3600.0, 0.012, "square", wwg, Some(3200.0), 0.0, h.pan);
            self.wristwatch_tick_timer = (3.0 + rnd() * 4.0) * etm;
            self.log_ambient("wristwatch-tick", Some(h.p.x), Some(h.p.z), Some(h.pan));
        }
        // R712 hot iron steam hiss (< 3.5 m)
        if let Some(h) = self.prop_due(|g| &mut g.iron_hiss_timer, |g| &g.pp.hot_iron_positions, 3.5, dt, m) {
            let hig = 0.007 * eg * h.g;
            self.noise_burst(0.85, 4200.0, 1.0, hig, "highpass", None, 0.0, h.pan);
            self.iron_hiss_timer = (20.0 + rnd() * 20.0) * etm;
            self.log_ambient("iron-hiss", Some(h.p.x), Some(h.p.z), Some(h.pan));
        }
        // R717 grandfather clock tick-tock (< 4 m)
        if let Some(h) = self.prop_due(|g| &mut g.grandfather_clock_tick_timer, |g| &g.pp.grandfather_clock_positions, 4.0, dt, m) {
            let gcg = 0.009 * eg * h.g;
            self.blip(220.0, 0.06, "square", gcg, Some(200.0), 0.0, h.pan);
            self.blip(180.0, 0.06, "square", gcg, Some(160.0), 1.0, h.pan);
            self.grandfather_clock_tick_timer = (2.0 + rnd()) * etm;
            self.log_ambient("grandfather-clock-tick", Some(h.p.x), Some(h.p.z), Some(h.pan));
        }
        // R718 rocking chair wood creak (< 3.5 m)
        if let Some(h) = self.prop_due(|g| &mut g.rocking_chair_creak_timer, |g| &g.pp.rocking_chair_positions, 3.5, dt, m) {
            let rcg = 0.007 * eg * h.g;
            self.noise_burst(0.35, 280.0, 1.2, rcg, "bandpass", Some(160.0), 0.0, h.pan);
            self.blip(120.0, 0.25, "sine", rcg * 0.4, Some(90.0), 0.10, h.pan);
            self.rocking_chair_creak_timer = (4.0 + rnd() * 4.0) * etm;
            self.log_ambient("rocker-creak", Some(h.p.x), Some(h.p.z), Some(h.pan));
        }
        // R718 tall lamp filament buzz (< 2.5 m)
        if let Some(h) = self.prop_due(|g| &mut g.tall_lamp_buzz_timer, |g| &g.pp.tall_lamp_positions, 2.5, dt, m) {
            let tlg = 0.005 * eg * h.g;
            self.blip(60.0, 1.2, "sine", tlg, None, 0.0, h.pan);
            self.blip(120.0, 1.2, "sine", tlg * 0.4, None, 0.0, h.pan);
            self.tall_lamp_buzz_timer = (12.0 + rnd() * 13.0) * etm;
            self.log_ambient("lamp-buzz", Some(h.p.x), Some(h.p.z), Some(h.pan));
        }
        // R721 hanging colander metallic tap (< 3.5 m)
        if let Some(h) = self.prop_due(|g| &mut g.colander_tap_timer, |g| &g.pp.colander_positions, 3.5, dt, m) {
            let clg = 0.007 * eg * h.g;
            self.blip(2800.0, 0.025, "square", clg, Some(2500.0), 0.0, h.pan);
            self.blip(2200.0, 0.020, "square", clg * 0.5, Some(2000.0), 0.03, h.pan);
            self.noise_burst(0.10, 3000.0, 1.4, clg * 0.3, "bandpass", Some(2400.0), 0.05, h.pan);
            self.colander_tap_timer = (30.0 + rnd() * 30.0) * etm;
            self.log_ambient("colander-tap", Some(h.p.x), Some(h.p.z), Some(h.pan));
        }
        // R723 pet water bowl slosh (< 2.5 m)
        if let Some(h) = self.prop_due(|g| &mut g.pet_bowl_slosh_timer, |g| &g.pp.pet_bowl_positions, 2.5, dt, m) {
            let pbg = 0.006 * eg * h.g;
            self.noise_burst(0.30, 750.0, 0.9, pbg, "bandpass", Some(550.0), 0.0, h.pan);
            self.blip(320.0, 0.20, "sine", pbg * 0.3, Some(260.0), 0.15, h.pan);
            self.pet_bowl_slosh_timer = (40.0 + rnd() * 40.0) * etm;
            self.log_ambient("pet-bowl-slosh", Some(h.p.x), Some(h.p.z), Some(h.pan));
        }
        // R725 music box C-E-G-A-G melody (< 3 m)
        if let Some(h) = self.prop_due(|g| &mut g.music_box_melody_timer, |g| &g.pp.music_box_positions, 3.0, dt, m) {
            let mbg = 0.006 * eg * h.g;
            const NOTES: [f64; 5] = [1046.0, 1319.0, 1568.0, 1760.0, 1568.0];
            for i in 0..5 {
                self.blip(NOTES[i], 0.18, "sine", mbg, Some(NOTES[i] * 0.995), i as f64 * 0.30, h.pan);
            }
            self.music_box_melody_timer = (20.0 + rnd() * 20.0) * etm;
            self.log_ambient("music-box-melody", Some(h.p.x), Some(h.p.z), Some(h.pan));
        }
        // R726 grandfather clock hourly chime (reuses the R717 positions, < 5 m)
        if let Some(h) = self.prop_due(|g| &mut g.grandfather_chime_timer, |g| &g.pp.grandfather_clock_positions, 5.0, dt, m) {
            let chimeg = 0.012 * eg * h.g;
            let strike_count = 2 + (rnd() * 3.0).floor() as i32;
            for i in 0..strike_count {
                let base_freq = 165.0 + (i % 2) as f64 * 15.0; // ~E3 alternating
                let i = i as f64;
                self.blip(base_freq, 0.90, "sine", chimeg * 0.7, Some(base_freq * 0.90), i * 1.4, h.pan);
                self.blip(base_freq * 2.0, 0.60, "sine", chimeg * 0.3, Some(base_freq * 1.85), i * 1.4 + 0.02, h.pan);
            }
            self.grandfather_chime_timer = (180.0 + rnd() * 180.0) * etm;
            self.log_ambient("grandfather-chime", Some(h.p.x), Some(h.p.z), Some(h.pan));
        }
        // R729 turntable vinyl hiss (< 3 m)
        if let Some(h) = self.prop_due(|g| &mut g.turntable_hiss_timer, |g| &g.pp.turntable_positions, 3.0, dt, m) {
            let ttg = 0.006 * eg * h.g;
            self.noise_burst(1.4, 5200.0, 0.8, ttg, "bandpass", Some(4200.0), 0.0, h.pan);
            // occasional pop tick
            if rnd() < 0.6 {
                self.blip(4600.0, 0.01, "square", ttg * 0.7, Some(4200.0), 0.3 + rnd() * 0.6, h.pan);
            }
            self.turntable_hiss_timer = (6.0 + rnd() * 8.0) * etm;
            self.log_ambient("turntable-hiss", Some(h.p.x), Some(h.p.z), Some(h.pan));
        }
    }

    /// R732–R767: chalkboard … hobby horse.
    pub(crate) fn amb_props_parlour(&mut self, dt: f64, m: &AmbMix) {
        let eg = m.gain;
        let etm = m.timer_mul;
        // R732 chalkboard chalk scratch — the invisible writer (< 3.5 m)
        if let Some(h) = self.prop_due(|g| &mut g.chalkboard_scratch_timer, |g| &g.pp.chalkboard_positions, 3.5, dt, m) {
            let cbg = 0.007 * eg * h.g;
            self.blip(4800.0, 0.06, "square", cbg * 0.6, Some(3800.0), 0.0, h.pan);
            self.noise_burst(0.12, 5500.0, 1.0, cbg * 0.4, "highpass", None, 0.03, h.pan);
            self.chalkboard_scratch_timer = (45.0 + rnd() * 45.0) * etm;
            self.log_ambient("chalkboard-scratch", Some(h.p.x), Some(h.p.z), Some(h.pan));
        }
        // R735 kitchen calendar page turn (< 3.5 m)
        if let Some(h) = self.prop_due(|g| &mut g.kitchen_calendar_turn_timer, |g| &g.pp.kitchen_calendar_positions, 3.5, dt, m) {
            let kcg = 0.006 * eg * h.g;
            self.noise_burst(0.55, 3800.0, 0.7, kcg, "highpass", None, 0.0, h.pan);
            self.blip(1200.0, 0.20, "triangle", kcg * 0.3, Some(900.0), 0.35, h.pan);
            self.kitchen_calendar_turn_timer = (90.0 + rnd() * 90.0) * etm;
            self.log_ambient("calendar-turn", Some(h.p.x), Some(h.p.z), Some(h.pan));
        }
        // R738 telescope focus-knob clicks (< 3 m)
        if let Some(h) = self.prop_due(|g| &mut g.telescope_click_timer, |g| &g.pp.telescope_positions, 3.0, dt, m) {
            let tsg = 0.005 * eg * h.g;
            let click_count = 2 + (rnd() * 2.0).floor() as i32;
            for i in 0..click_count {
                self.blip(1800.0, 0.012, "square", tsg, Some(1600.0), i as f64 * 0.14, h.pan);
            }
            self.telescope_click_timer = (20.0 + rnd() * 20.0) * etm;
            self.log_ambient("telescope-click", Some(h.p.x), Some(h.p.z), Some(h.pan));
        }
        // R741 cuckoo clock 2–4 "cuckoo" calls (< 4 m)
        if let Some(h) = self.prop_due(|g| &mut g.cuckoo_call_timer_r741, |g| &g.pp.cuckoo_clock_positions_r741, 4.0, dt, m) {
            let ccg = 0.010 * eg * h.g;
            let call_count = 2 + (rnd() * 3.0).floor() as i32;
            for i in 0..call_count {
                let i = i as f64;
                self.blip(660.0, 0.18, "sine", ccg, Some(640.0), i * 0.55, h.pan); // "cu" (E5)
                self.blip(520.0, 0.28, "sine", ccg, Some(500.0), i * 0.55 + 0.22, h.pan);
                // "coo" (C5)
            }
            self.cuckoo_call_timer_r741 = (120.0 + rnd() * 180.0) * etm;
            self.log_ambient("cuckoo-clock-call", Some(h.p.x), Some(h.p.z), Some(h.pan));
        }
        // R744 lit fireplace crackle (< 5 m)
        if let Some(h) = self.prop_due(|g| &mut g.fireplace_crackle_timer, |g| &g.pp.lit_fireplace_positions, 5.0, dt, m) {
            let fpg = 0.010 * eg * h.g;
            let pop_count = 2 + (rnd() * 4.0).floor() as i32;
            for i in 0..pop_count {
                let i = i as f64;
                self.noise_burst(0.06, 800.0, 0.8, fpg, "bandpass", Some(500.0), i * 0.15 + rnd() * 0.10, h.pan);
                // occasional higher snap
                if rnd() < 0.4 {
                    self.blip(1400.0, 0.015, "square", fpg * 0.5, Some(1200.0), i * 0.15 + 0.03, h.pan);
                }
            }
            self.fireplace_crackle_timer = (4.0 + rnd() * 6.0) * etm;
            self.log_ambient("fireplace-crackle", Some(h.p.x), Some(h.p.z), Some(h.pan));
        }
        // R744 aquarium filter hum + bubble pops (< 3 m)
        if let Some(h) = self.prop_due(|g| &mut g.aquarium_hum_timer, |g| &g.pp.aquarium_positions, 3.0, dt, m) {
            let aqg = 0.006 * eg * h.g;
            self.blip(90.0, 1.5, "sine", aqg * 0.5, None, 0.0, h.pan);
            let pop_count = 2 + (rnd() * 3.0).floor() as i32;
            for i in 0..pop_count {
                self.blip(4200.0, 0.008, "square", aqg * 0.4, Some(3800.0), i as f64 * 0.20 + rnd() * 0.10, h.pan);
            }
            self.aquarium_hum_timer = (8.0 + rnd() * 7.0) * etm;
            self.log_ambient("aquarium-hum", Some(h.p.x), Some(h.p.z), Some(h.pan));
        }
        // R747 wingback armchair creak — an invisible sitter shifts (< 3.5 m)
        if let Some(h) = self.prop_due(|g| &mut g.wingback_creak_timer, |g| &g.pp.wingback_armchair_positions, 3.5, dt, m) {
            let wbg = 0.014 * eg * h.g;
            self.blip(180.0, 0.35, "sawtooth", wbg * 0.5, Some(130.0), 0.0, h.pan);
            self.blip(320.0, 0.28, "triangle", wbg * 0.35, Some(260.0), 0.08, h.pan);
            self.noise_burst(0.12, 1200.0, 0.6, wbg * 0.25, "bandpass", Some(800.0), 0.20, h.pan);
            self.wingback_creak_timer = (60.0 + rnd() * 60.0) * etm;
            self.log_ambient("wingback-creak", Some(h.p.x), Some(h.p.z), Some(h.pan));
        }
        // R755 frozen pocket watch still ticking (< 2.5 m)
        if let Some(h) = self.prop_due(|g| &mut g.pocket_watch_tick_timer, |g| &g.pp.pocket_watch_positions_r755, 2.5, dt, m) {
            let pwg = 0.008 * eg * h.g;
            self.blip(3200.0, 0.012, "square", pwg, Some(2800.0), 0.0, h.pan);
            self.blip(2400.0, 0.014, "square", pwg * 0.9, Some(2000.0), 0.50, h.pan);
            self.pocket_watch_tick_timer = (3.0 + rnd() * 3.0) * etm;
            self.log_ambient("pocket-watch-tick", Some(h.p.x), Some(h.p.z), Some(h.pan));
        }
        // R757 servant's bell pull brass ring (< 6 m)
        if let Some(h) = self.prop_due(|g| &mut g.bell_pull_ring_timer, |g| &g.pp.bell_pull_positions, 6.0, dt, m) {
            let bpg = 0.014 * eg * h.g;
            self.blip(1200.0, 0.35, "sine", bpg, None, 0.0, h.pan);
            self.blip(1800.0, 0.28, "sine", bpg * 0.6, None, 0.02, h.pan);
            self.noise_burst(0.02, 800.0, 0.7, bpg * 0.4, "bandpass", Some(400.0), 0.0, h.pan);
            self.blip(900.0, 0.22, "sine", bpg * 0.35, None, 0.15, h.pan);
            self.bell_pull_ring_timer = (120.0 + rnd() * 120.0) * etm;
            self.log_ambient("bell-pull-ring", Some(h.p.x), Some(h.p.z), Some(h.pan));
        }
        // R761 servants'-hall bell rack ding (< 5 m)
        if let Some(h) = self.prop_due(|g| &mut g.bell_rack_ding_timer, |g| &g.pp.bell_rack_positions, 5.0, dt, m) {
            let brg = 0.012 * eg * h.g;
            self.blip(2600.0, 0.25, "sine", brg, None, 0.0, h.pan);
            self.blip(3900.0, 0.20, "sine", brg * 0.55, None, 0.010, h.pan);
            self.blip(2400.0, 0.15, "sine", brg * 0.30, Some(2200.0), 0.08, h.pan);
            self.blip(2400.0, 0.12, "sine", brg * 0.20, Some(2600.0), 0.18, h.pan);
            self.bell_rack_ding_timer = (90.0 + rnd() * 90.0) * etm;
            self.log_ambient("bell-rack-ding", Some(h.p.x), Some(h.p.z), Some(h.pan));
        }
        // R763 chess piece move — the abandoned game continues (< 3 m)
        if let Some(h) = self.prop_due(|g| &mut g.chess_move_click_timer, |g| &g.pp.chess_set_positions, 3.0, dt, m) {
            let csg = 0.014 * eg * h.g;
            let is_full_move = rnd() < 0.5;
            self.noise_burst(0.015, 400.0, 0.6, csg, "bandpass", Some(200.0), 0.0, h.pan);
            self.blip(220.0, 0.020, "square", csg * 0.6, Some(180.0), 0.005, h.pan);
            if is_full_move {
                let delay = 0.6 + rnd() * 0.6;
                self.noise_burst(0.015, 400.0, 0.6, csg, "bandpass", Some(200.0), delay, h.pan);
                self.blip(240.0, 0.022, "square", csg * 0.6, Some(200.0), delay + 0.005, h.pan);
            }
            self.chess_move_click_timer = (45.0 + rnd() * 45.0) * etm;
            self.log_ambient("chess-move", Some(h.p.x), Some(h.p.z), Some(h.pan));
        }
        // R767 wheeled hobby horse roll, decelerating clicks (< 4 m)
        if let Some(h) = self.prop_due(|g| &mut g.hobby_horse_roll_timer, |g| &g.pp.hobby_horse_wheeled_positions, 4.0, dt, m) {
            let hhg = 0.010 * eg * h.g;
            let click_count = 3 + (rnd() * 3.0).floor() as i32;
            for i in 0..click_count {
                let i = i as f64;
                let delay = i * (0.10 + i * 0.03);
                self.noise_burst(0.012, 300.0, 0.5, hhg * (1.0 - i * 0.10), "bandpass", Some(150.0), delay, h.pan);
                self.blip(180.0, 0.015, "square", hhg * 0.5 * (1.0 - i * 0.10), Some(150.0), delay + 0.003, h.pan);
            }
            self.hobby_horse_roll_timer = (90.0 + rnd() * 90.0) * etm;
            self.log_ambient("horse-roll", Some(h.p.x), Some(h.p.z), Some(h.pan));
        }
    }
}
