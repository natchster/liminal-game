//! `update_ambient`, part 3: the fixtures' prop sounds and the trinkets'.
//! Each event: its cadence timer, the nearest prop of its `*_positions` list
//! within the sound's audible radius (`prop_due`: level, pan from the true
//! direction, muffled without line of sight), the sound, the next cadence,
//! the log entry. With no prop in earshot nothing plays and the timer looks
//! again in a second or two. See `audio::ambient` (mod.rs) for the overview.
use crate::audio::ambient::AmbMix;
use crate::prelude::*;

impl Game {
    /// Fountain … smoke detector.
    pub(crate) fn amb_props_fixtures(&mut self, dt: f64, m: &AmbMix) {
        let (eg, ld, etm) = (m.gain, m.depth, m.timer_mul);
        // Fountain drip: two-tap high blip + low tail (< 12 m)
        if let Some(h) = self.prop_due(|g| &mut g.fountain_drip_timer, |g| &g.pp.fountain_positions, 12.0, dt, m) {
            self.blip(1600.0 * ld, 0.06, "sine", 0.016 * eg * h.g, Some(800.0 * ld), 0.0, h.pan);
            self.blip(180.0 * ld, 0.10, "sine", 0.008 * eg * h.g, Some(100.0 * ld), 0.05, h.pan);
            self.fountain_drip_timer = (25.0 + rnd() * 30.0) * etm;
            self.log_ambient("fountain-drip", Some(h.p.x), Some(h.p.z), Some(h.pan));
        }
        // Radiator hiss: bandpass noise sweep (< 10 m)
        if let Some(h) = self.prop_due(|g| &mut g.radiator_hiss_timer, |g| &g.pp.radiator_positions, 10.0, dt, m) {
            self.noise_burst(0.5, 1200.0 * ld, 1.5, 0.014 * eg * h.g, "bandpass", Some(400.0 * ld), 0.0, h.pan);
            self.radiator_hiss_timer = (40.0 + rnd() * 40.0) * etm;
            self.log_ambient("radiator-hiss", Some(h.p.x), Some(h.p.z), Some(h.pan));
        }
        // Emergency glow hum: 60 Hz mains (< 8 m)
        if let Some(h) = self.prop_due(|g| &mut g.glow_hum_timer, |g| &g.pp.glow_light_positions, 8.0, dt, m) {
            self.blip(60.0, 0.35, "sine", 0.012 * eg * h.g, None, 0.0, h.pan);
            self.blip(120.0, 0.35, "sine", 0.007 * eg * h.g, None, 0.0, h.pan);
            self.glow_hum_timer = (20.0 + rnd() * 25.0) * etm;
            self.log_ambient("glow-hum", Some(h.p.x), Some(h.p.z), Some(h.pan));
        }
        // Cauldron bubble: a soft "blooip" (< 10 m)
        if let Some(h) = self.prop_due(|g| &mut g.cauldron_bubble_timer, |g| &g.pp.cauldron_positions, 10.0, dt, m) {
            let g0 = 0.014 * eg * h.g;
            self.blip(160.0 * ld, 0.14, "sine", g0, Some(300.0 * ld), 0.0, h.pan);
            self.blip(240.0 * ld, 0.10, "sine", g0 * 0.6, Some(120.0 * ld), 0.08, h.pan);
            self.noise_burst(0.08, 500.0 * ld, 3.0, g0 * 0.5, "bandpass", None, 0.02, h.pan);
            self.cauldron_bubble_timer = (12.0 + rnd() * 13.0) * etm;
            self.log_ambient("cauldron-bubble", Some(h.p.x), Some(h.p.z), Some(h.pan));
        }
        // Metronome tick, at half the visible swing period (< 8 m)
        if let Some(h) = self.prop_due(|g| &mut g.metronome_tick_timer, |g| &g.pp.metronome_positions, 8.0, dt, m) {
            let tick_g = 0.020 * eg * h.g;
            self.blip(2400.0, 0.02, "square", tick_g, None, 0.0, h.pan);
            self.noise_burst(0.03, 3800.0, 4.0, tick_g * 0.5, "highpass", None, 0.005, h.pan);
            self.metronome_tick_timer = (self.metronome_period(h.i) * 0.5) * etm;
            self.log_ambient("metronome-tick", Some(h.p.x), Some(h.p.z), Some(h.pan));
        }
        // Phonograph vinyl crackle: dust pops on a record still playing (< 10 m)
        if let Some(h) = self.prop_due(|g| &mut g.phono_crackle_timer, |g| &g.pp.phonograph_positions, 10.0, dt, m) {
            let cg = 0.012 * eg * h.g;
            for i in 0..4 {
                self.noise_burst(0.015, 5000.0 + rnd() * 2000.0, 5.0, cg * (0.6 + rnd() * 0.4), "highpass", None, i as f64 * 0.06, h.pan);
            }
            self.phono_crackle_timer = (8.0 + rnd() * 8.0) * etm;
            self.log_ambient("phono-crackle", Some(h.p.x), Some(h.p.z), Some(h.pan));
        }
        // Hanging brass lantern rope creak (< 10 m)
        if let Some(h) = self.prop_due(|g| &mut g.lantern_creak_timer, |g| &g.pp.brass_lantern_positions, 10.0, dt, m) {
            let cg2 = 0.014 * eg * h.g;
            self.blip(140.0, 0.5, "sawtooth", cg2, Some(90.0), 0.0, h.pan);
            self.noise_burst(0.35, 220.0, 2.0, cg2 * 0.4, "bandpass", Some(160.0), 0.05, h.pan);
            self.lantern_creak_timer = (18.0 + rnd() * 22.0) * etm;
            self.log_ambient("lantern-creak", Some(h.p.x), Some(h.p.z), Some(h.pan));
        }
        // Sewing machine whirr + stitches (< 10 m)
        if let Some(h) = self.prop_due(|g| &mut g.sewing_whirr_timer, |g| &g.pp.sewing_positions, 10.0, dt, m) {
            let sg = 0.012 * eg * h.g;
            self.blip(220.0, 0.5, "sawtooth", sg, Some(200.0), 0.0, h.pan);
            for i in 0..3 {
                self.blip(1800.0, 0.02, "square", sg * 0.6, None, 0.08 + i as f64 * 0.08, h.pan);
            }
            self.sewing_whirr_timer = (15.0 + rnd() * 15.0) * etm;
            self.log_ambient("sewing-whirr", Some(h.p.x), Some(h.p.z), Some(h.pan));
        }
        // Pocket watch tick-tock (< 3 m)
        if let Some(h) = self.prop_due(|g| &mut g.watch_tick_timer, |g| &g.pp.pocket_watch_positions, 3.0, dt, m) {
            let wg = 0.010 * eg * h.g;
            self.blip(3200.0, 0.012, "square", wg, None, 0.0, h.pan);
            self.blip(2800.0, 0.012, "square", wg * 0.9, None, 0.5, h.pan);
            self.watch_tick_timer = (3.0 + rnd() * 3.0) * etm;
            self.log_ambient("watch-tick", Some(h.p.x), Some(h.p.z), Some(h.pan));
        }
        // Bat squeak (< 14 m)
        if let Some(h) = self.prop_due(|g| &mut g.bat_squeak_timer, |g| &g.pp.bat_positions, 14.0, dt, m) {
            let bg = 0.012 * eg * h.g;
            self.blip(6000.0, 0.04, "sine", bg, Some(4000.0), 0.0, h.pan);
            self.blip(5000.0, 0.03, "sine", bg * 0.7, Some(3500.0), 0.05, h.pan);
            self.bat_squeak_timer = (25.0 + rnd() * 30.0) * etm;
            self.log_ambient("bat-squeak", Some(h.p.x), Some(h.p.z), Some(h.pan));
        }
        // Spinning globe wood creak (< 8 m)
        if let Some(h) = self.prop_due(|g| &mut g.globe_creak_timer, |g| &g.pp.globe_positions, 8.0, dt, m) {
            let gg = 0.010 * eg * h.g;
            self.blip(180.0, 0.4, "sawtooth", gg, Some(130.0), 0.0, h.pan);
            self.noise_burst(0.25, 340.0, 3.0, gg * 0.4, "bandpass", Some(260.0), 0.02, h.pan);
            self.globe_creak_timer = (22.0 + rnd() * 23.0) * etm;
            self.log_ambient("globe-creak", Some(h.p.x), Some(h.p.z), Some(h.pan));
        }
        // Hourglass sand whoosh (< 3 m)
        if let Some(h) = self.prop_due(|g| &mut g.hourglass_whoosh_timer, |g| &g.pp.hourglass_positions, 3.0, dt, m) {
            self.noise_burst(0.6, 3000.0, 4.0, 0.008 * eg * h.g, "bandpass", Some(3200.0), 0.0, h.pan);
            self.hourglass_whoosh_timer = (4.0 + rnd() * 4.0) * etm;
            self.log_ambient("hourglass-whoosh", Some(h.p.x), Some(h.p.z), Some(h.pan));
        }
        // Ghost sheet whisper (< 12 m)
        if let Some(h) = self.prop_due(|g| &mut g.ghost_whisper_timer, |g| &g.pp.ghost_positions, 12.0, dt, m) {
            let ghg = 0.014 * eg * h.g;
            self.noise_burst(0.7, 800.0, 1.2, ghg, "bandpass", Some(400.0), 0.0, h.pan);
            self.blip(90.0, 0.6, "sine", ghg * 0.3, Some(60.0), 0.1, h.pan);
            self.ghost_whisper_timer = (30.0 + rnd() * 40.0) * etm;
            self.log_ambient("ghost-whisper", Some(h.p.x), Some(h.p.z), Some(h.pan));
        }
        // Chain jangle (< 4 m)
        if let Some(h) = self.prop_due(|g| &mut g.chain_jangle_timer, |g| &g.pp.chain_positions, 4.0, dt, m) {
            let chg = 0.014 * eg * h.g;
            self.noise_burst(0.03, 4200.0, 8.0, chg, "bandpass", Some(3600.0), 0.0, h.pan);
            self.noise_burst(0.03, 3800.0, 8.0, chg * 0.7, "bandpass", Some(3200.0), 0.08 + rnd() * 0.05, h.pan);
            self.chain_jangle_timer = (8.0 + rnd() * 10.0) * etm;
            self.log_ambient("chain-jangle", Some(h.p.x), Some(h.p.z), Some(h.pan));
        }
        // Paper rustle near music sheets / journals (< 5 m)
        if let Some(h) = self.prop_due(|g| &mut g.paper_rustle_timer, |g| &g.pp.paper_positions, 5.0, dt, m) {
            let ppg = 0.010 * eg * h.g;
            self.noise_burst(0.08, 4000.0, 3.0, ppg, "highpass", Some(5000.0), 0.0, h.pan);
            self.noise_burst(0.06, 5000.0, 3.0, ppg * 0.7, "highpass", Some(6000.0), 0.05, h.pan);
            self.paper_rustle_timer = (20.0 + rnd() * 25.0) * etm;
            self.log_ambient("paper-rustle", Some(h.p.x), Some(h.p.z), Some(h.pan));
        }
        // Toy piano random note — "who pressed it?" (< 6 m)
        if let Some(h) = self.prop_due(|g| &mut g.piano_chord_timer, |g| &g.pp.piano_positions, 6.0, dt, m) {
            let png = 0.020 * eg * h.g;
            const NOTES: [f64; 8] = [261.63, 293.66, 329.63, 349.23, 392.00, 440.00, 493.88, 523.25];
            let f = NOTES[pick_index(NOTES.len())];
            self.blip(f, 0.4, "sine", png, None, 0.0, h.pan);
            self.blip(f * 2.0, 0.3, "sine", png * 0.3, None, 0.005, h.pan);
            self.piano_chord_timer = (40.0 + rnd() * 50.0) * etm;
            self.log_ambient("piano-note", Some(h.p.x), Some(h.p.z), Some(h.pan));
        }
        // Rocking chair creak, one per rock (< 6 m)
        if let Some(h) = self.prop_due(|g| &mut g.chair_creak_timer, |g| &g.pp.chair_positions, 6.0, dt, m) {
            let chg = 0.014 * eg * h.g;
            self.blip(200.0, 0.45, "sawtooth", chg, Some(140.0), 0.0, h.pan);
            self.noise_burst(0.30, 380.0, 3.0, chg * 0.4, "bandpass", Some(260.0), 0.05, h.pan);
            // next creak at the rock period (one creak per rock)
            self.chair_creak_timer = self.chair_period(h.i) * etm;
            self.log_ambient("chair-creak", Some(h.p.x), Some(h.p.z), Some(h.pan));
        }
        // CRT TV static hiss (< 4 m)
        if let Some(h) = self.prop_due(|g| &mut g.crt_hiss_timer, |g| &g.pp.crt_positions, 4.0, dt, m) {
            let tg = 0.014 * eg * h.g;
            self.noise_burst(0.4, 5000.0, 0.5, tg, "highpass", None, 0.0, h.pan);
            self.blip(60.0, 0.4, "sine", tg * 0.3, None, 0.01, h.pan);
            self.crt_hiss_timer = (4.0 + rnd() * 6.0) * etm;
            self.log_ambient("crt-hiss", Some(h.p.x), Some(h.p.z), Some(h.pan));
        }
        // Exit sign buzz (< 5 m)
        if let Some(h) = self.prop_due(|g| &mut g.exit_buzz_timer, |g| &g.pp.exit_sign_positions, 5.0, dt, m) {
            let egn = 0.010 * eg * h.g;
            self.blip(120.0, 0.5, "sawtooth", egn, None, 0.0, h.pan);
            self.blip(240.0, 0.5, "sine", egn * 0.4, None, 0.0, h.pan);
            self.exit_buzz_timer = (6.0 + rnd() * 6.0) * etm;
            self.log_ambient("exit-buzz", Some(h.p.x), Some(h.p.z), Some(h.pan));
        }
        // Puddle drip (< 5 m)
        if let Some(h) = self.prop_due(|g| &mut g.puddle_drip_timer, |g| &g.pp.puddle_positions, 5.0, dt, m) {
            let pg = 0.014 * eg * h.g;
            self.blip(1400.0 * ld, 0.05, "sine", pg, Some(700.0 * ld), 0.0, h.pan);
            self.blip(140.0 * ld, 0.12, "sine", pg * 0.6, Some(80.0 * ld), 0.03, h.pan);
            self.puddle_drip_timer = (8.0 + rnd() * 8.0) * etm;
            self.log_ambient("puddle-drip", Some(h.p.x), Some(h.p.z), Some(h.pan));
        }
        // Chandelier crystal chime (< 6 m)
        if let Some(h) = self.prop_due(|g| &mut g.chandelier_chime_timer, |g| &g.pp.chandelier_positions, 6.0, dt, m) {
            let cg2 = 0.010 * eg * h.g;
            let base_f = 2100.0 + rnd() * 400.0;
            self.blip(base_f, 0.6, "sine", cg2, None, 0.0, h.pan);
            self.blip(base_f * 1.25, 0.5, "sine", cg2 * 0.7, None, 0.05, h.pan);
            self.blip(base_f * 1.5, 0.4, "sine", cg2 * 0.5, None, 0.10, h.pan);
            self.chandelier_chime_timer = (20.0 + rnd() * 20.0) * etm;
            self.log_ambient("chandelier-chime", Some(h.p.x), Some(h.p.z), Some(h.pan));
        }
        // Cassette tape hiss (< 3 m)
        if let Some(h) = self.prop_due(|g| &mut g.cassette_hiss_timer, |g| &g.pp.cassette_positions, 3.0, dt, m) {
            self.noise_burst(0.5, 3500.0, 0.6, 0.008 * eg * h.g, "highpass", None, 0.0, h.pan);
            self.cassette_hiss_timer = (10.0 + rnd() * 10.0) * etm;
            self.log_ambient("cassette-hiss", Some(h.p.x), Some(h.p.z), Some(h.pan));
        }
        // Teacup porcelain clink (< 3.5 m)
        if let Some(h) = self.prop_due(|g| &mut g.teacup_clink_timer, |g| &g.pp.teacup_positions, 3.5, dt, m) {
            let tcg = 0.008 * eg * h.g;
            self.blip(3200.0, 0.03, "sine", tcg, None, 0.0, h.pan);
            self.blip(4200.0, 0.02, "sine", tcg * 0.6, None, 0.02, h.pan);
            self.teacup_clink_timer = (25.0 + rnd() * 25.0) * etm;
            self.log_ambient("teacup-clink", Some(h.p.x), Some(h.p.z), Some(h.pan));
        }
        // Locker slam: sudden metal clang, carries (< 22 m)
        if let Some(h) = self.prop_due(|g| &mut g.locker_slam_timer, |g| &g.pp.locker_positions, 22.0, dt, m) {
            let lkg = 0.028 * eg * h.g;
            self.blip(90.0, 0.06, "sine", lkg, Some(55.0), 0.0, h.pan);
            self.noise_burst(0.15, 2200.0, 5.0, lkg * 0.6, "bandpass", None, 0.03, h.pan);
            self.noise_burst(0.10, 4200.0, 6.0, lkg * 0.4, "bandpass", None, 0.10, h.pan);
            self.locker_slam_timer = (60.0 + rnd() * 60.0) * etm;
            self.log_ambient("locker-slam", Some(h.p.x), Some(h.p.z), Some(h.pan));
        }
        // Bulletin board flutter (< 5 m)
        if let Some(h) = self.prop_due(|g| &mut g.bulletin_flutter_timer, |g| &g.pp.bulletin_positions, 5.0, dt, m) {
            let bbg = 0.010 * eg * h.g;
            for i in 0..3 {
                self.noise_burst(0.06, 4500.0, 3.0, bbg * (0.6 + rnd() * 0.4), "highpass", None, i as f64 * 0.05, h.pan);
            }
            self.bulletin_flutter_timer = (15.0 + rnd() * 15.0) * etm;
            self.log_ambient("bulletin-flutter", Some(h.p.x), Some(h.p.z), Some(h.pan));
        }
        // Fire alarm pull beep (< 7 m)
        if let Some(h) = self.prop_due(|g| &mut g.alarm_beep_timer, |g| &g.pp.alarm_pull_positions, 7.0, dt, m) {
            let alg = 0.014 * eg * h.g;
            self.blip(3200.0, 0.06, "square", alg, None, 0.0, h.pan);
            self.blip(3200.0, 0.06, "square", alg, None, 0.10, h.pan);
            self.alarm_beep_timer = (35.0 + rnd() * 35.0) * etm;
            self.log_ambient("alarm-beep", Some(h.p.x), Some(h.p.z), Some(h.pan));
        }
        // Ceiling fan whoosh (< 5 m, nearly under it)
        if let Some(h) = self.prop_due(|g| &mut g.fan_whoosh_timer, |g| &g.pp.fan_positions, 5.0, dt, m) {
            self.noise_burst(0.9, 260.0, 1.2, 0.012 * eg * h.g, "bandpass", Some(220.0), 0.0, h.pan);
            self.fan_whoosh_timer = (8.0 + rnd() * 6.0) * etm;
            self.log_ambient("fan-whoosh", Some(h.p.x), Some(h.p.z), Some(h.pan));
        }
        // Fan bearing squeak (< 6 m)
        if let Some(h) = self.prop_due(|g| &mut g.fan_squeak_timer, |g| &g.pp.fan_positions, 6.0, dt, m) {
            self.blip(2600.0, 0.05, "sine", 0.011 * eg * h.g, Some(2300.0), 0.0, h.pan);
            self.fan_squeak_timer = (25.0 + rnd() * 30.0) * etm;
            self.log_ambient("fan-squeak", Some(h.p.x), Some(h.p.z), Some(h.pan));
        }
        // Smoke detector low-battery chirp (< 8 m)
        if let Some(h) = self.prop_due(|g| &mut g.detector_chirp_timer, |g| &g.pp.smoke_detector_positions, 8.0, dt, m) {
            self.blip(3800.0, 0.08, "sine", 0.013 * eg * h.g, None, 0.0, h.pan);
            self.detector_chirp_timer = (90.0 + rnd() * 110.0) * etm;
            self.log_ambient("detector-chirp", Some(h.p.x), Some(h.p.z), Some(h.pan));
        }
    }

    /// Props: coin … teapot.
    pub(crate) fn amb_props_trinkets(&mut self, dt: f64, m: &AmbMix) {
        let (eg, etm) = (m.gain, m.timer_mul);
        // Coin cluster tink (< 3.5 m)
        if let Some(h) = self.prop_due(|g| &mut g.coin_tink_timer, |g| &g.pp.coin_positions, 3.5, dt, m) {
            self.blip(4500.0, 0.04, "sine", 0.011 * eg * h.g, Some(4200.0), 0.0, h.pan);
            self.coin_tink_timer = (20.0 + rnd() * 25.0) * etm;
            self.log_ambient("coin-tink", Some(h.p.x), Some(h.p.z), Some(h.pan));
        }
        // Ticking clock tick-tock (< 4 m)
        if let Some(h) = self.prop_due(|g| &mut g.tick_tock_timer, |g| &g.pp.clock_positions, 4.0, dt, m) {
            let ckg = 0.010 * eg * h.g;
            self.blip(2200.0, 0.03, "square", ckg, None, 0.0, h.pan);
            self.blip(1600.0, 0.03, "square", ckg, None, 0.42, h.pan);
            self.tick_tock_timer = (12.0 + rnd() * 13.0) * etm;
            self.log_ambient("tick-tock", Some(h.p.x), Some(h.p.z), Some(h.pan));
        }
        // Cuckoo call near cuckoo clocks (< 6 m)
        if let Some(h) = self.prop_due(|g| &mut g.cuckoo_call_timer, |g| &g.pp.cuckoo_clock_positions, 6.0, dt, m) {
            let ccg = 0.013 * eg * h.g;
            self.blip(700.0, 0.14, "sine", ccg, Some(680.0), 0.0, h.pan);
            self.blip(560.0, 0.16, "sine", ccg, Some(540.0), 0.20, h.pan);
            self.cuckoo_call_timer = (90.0 + rnd() * 130.0) * etm;
            self.log_ambient("cuckoo-call", Some(h.p.x), Some(h.p.z), Some(h.pan));
        }
        // Plush shuffle — the stuffed animal stirred (< 3 m)
        if let Some(h) = self.prop_due(|g| &mut g.plush_shuffle_timer, |g| &g.pp.plush_positions, 3.0, dt, m) {
            self.noise_burst(0.15, 2400.0, 0.8, 0.009 * eg * h.g, "bandpass", Some(1800.0), 0.0, h.pan);
            self.plush_shuffle_timer = (150.0 + rnd() * 170.0) * etm;
            self.log_ambient("plush-shuffle", Some(h.p.x), Some(h.p.z), Some(h.pan));
        }
        // Gumball dispenser rattle (< 4 m)
        if let Some(h) = self.prop_due(|g| &mut g.gumball_rattle_timer, |g| &g.pp.gumball_positions, 4.0, dt, m) {
            let gpg = 0.010 * eg * h.g;
            let click_count = 3 + (rnd() * 3.0).floor() as i32;
            for i in 0..click_count {
                self.blip(3400.0 + rnd() * 800.0, 0.03, "sine", gpg, None, i as f64 * 0.08 + rnd() * 0.05, h.pan);
            }
            self.gumball_rattle_timer = (40.0 + rnd() * 50.0) * etm;
            self.log_ambient("gumball-rattle", Some(h.p.x), Some(h.p.z), Some(h.pan));
        }
        // Teapot whistle — "who's boiling water?" (< 5 m)
        if let Some(h) = self.prop_due(|g| &mut g.teapot_whistle_timer, |g| &g.pp.teapot_positions, 5.0, dt, m) {
            let tpg = 0.010 * eg * h.g;
            self.blip(2200.0, 0.9, "sine", tpg, Some(2800.0), 0.0, h.pan);
            self.noise_burst(0.4, 3200.0, 1.0, tpg * 0.4, "highpass", None, 0.6, h.pan);
            self.teapot_whistle_timer = (200.0 + rnd() * 250.0) * etm;
            self.log_ambient("teapot-whistle", Some(h.p.x), Some(h.p.z), Some(h.pan));
        }
    }
}
