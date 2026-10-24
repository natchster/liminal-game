//! updateAmbient, part 3: prop sounds R272–R584 (lines 63613–64180) and the
//! outdoor ghosts + R608–R672 props (64181–64409). Each event: its cadence
//! timer, one `Math.random()` pick from the prop's `*Positions` list, pan and
//! distance falloff from the player's facing (`prop_pan`), the sound, the
//! next cadence, the log entry. "Gated" props are audible only within a
//! radius; their cadence resets either way. See ambient.rs for the overview.
use crate::ambient::{pick_pos, AmbMix};
use crate::prelude::*;

impl Game {
    /// R272–R584: fountain … smoke detector (lines 63613–64180).
    pub(crate) fn amb_props_fixtures(&mut self, dt: f64, m: &AmbMix) {
        let (eg, ld, etm) = (m.gain, m.depth, m.timer_mul);
        // R272 fountain drip: two-tap high blip + low tail
        self.fountain_drip_timer -= dt;
        if self.fountain_drip_timer <= 0.0 && !self.pp.fountain_positions.is_empty() {
            let (_, f) = pick_pos(&self.pp.fountain_positions);
            let (pan, dn) = self.prop_pan(f, 0.6, 0.8, 8.0);
            self.blip(1600.0 * ld, 0.06, "sine", 0.016 * eg * (1.0 - dn * 0.5), Some(800.0 * ld), 0.0, pan);
            self.blip(180.0 * ld, 0.10, "sine", 0.008 * eg * (1.0 - dn * 0.5), Some(100.0 * ld), 0.05, pan);
            self.fountain_drip_timer = (25.0 + rnd() * 30.0) * etm;
            self.log_ambient("fountain-drip", Some(f.x), Some(f.z), Some(pan));
        }
        // R272 radiator hiss: bandpass noise sweep
        self.radiator_hiss_timer -= dt;
        if self.radiator_hiss_timer <= 0.0 && !self.pp.radiator_positions.is_empty() {
            let (_, r2) = pick_pos(&self.pp.radiator_positions);
            let (pan, dn) = self.prop_pan(r2, 0.6, 0.8, 8.0);
            self.noise_burst(0.5, 1200.0 * ld, 1.5, 0.014 * eg * (1.0 - dn * 0.5), "bandpass", Some(400.0 * ld), 0.0, pan);
            self.radiator_hiss_timer = (40.0 + rnd() * 40.0) * etm;
            self.log_ambient("radiator-hiss", Some(r2.x), Some(r2.z), Some(pan));
        }
        // R273 emergency glow hum: 60 Hz mains
        self.glow_hum_timer -= dt;
        if self.glow_hum_timer <= 0.0 && !self.pp.glow_light_positions.is_empty() {
            let (_, g) = pick_pos(&self.pp.glow_light_positions);
            let (pan, dn) = self.prop_pan(g, 0.7, 0.8, 6.0);
            self.blip(60.0, 0.35, "sine", 0.012 * eg * (1.0 - dn * 0.5), None, 0.0, pan);
            self.blip(120.0, 0.35, "sine", 0.007 * eg * (1.0 - dn * 0.5), None, 0.0, pan);
            self.glow_hum_timer = (20.0 + rnd() * 25.0) * etm;
            self.log_ambient("glow-hum", Some(g.x), Some(g.z), Some(pan));
        }
        // R479 cauldron bubble: a soft "blooip"
        self.cauldron_bubble_timer -= dt;
        if self.cauldron_bubble_timer <= 0.0 && !self.pp.cauldron_positions.is_empty() {
            let (_, c) = pick_pos(&self.pp.cauldron_positions);
            let (pan, dn) = self.prop_pan(c, 0.6, 0.8, 8.0);
            let g0 = 0.014 * eg * (1.0 - dn * 0.5);
            self.blip(160.0 * ld, 0.14, "sine", g0, Some(300.0 * ld), 0.0, pan);
            self.blip(240.0 * ld, 0.10, "sine", g0 * 0.6, Some(120.0 * ld), 0.08, pan);
            self.noise_burst(0.08, 500.0 * ld, 3.0, g0 * 0.5, "bandpass", None, 0.02, pan);
            self.cauldron_bubble_timer = (12.0 + rnd() * 13.0) * etm;
            self.log_ambient("cauldron-bubble", Some(c.x), Some(c.z), Some(pan));
        }
        // R479 metronome tick, at half the visible swing period
        self.metronome_tick_timer -= dt;
        if self.metronome_tick_timer <= 0.0 && !self.pp.metronome_positions.is_empty() {
            let (i, mp) = pick_pos(&self.pp.metronome_positions);
            let (pan, dn) = self.prop_pan(mp, 0.7, 0.8, 6.0);
            let tick_g = 0.020 * eg * (1.0 - dn * 0.5);
            self.blip(2400.0, 0.02, "square", tick_g, None, 0.0, pan);
            self.noise_burst(0.03, 3800.0, 4.0, tick_g * 0.5, "highpass", None, 0.005, pan);
            self.metronome_tick_timer = (self.metronome_period(i) * 0.5) * etm;
            self.log_ambient("metronome-tick", Some(mp.x), Some(mp.z), Some(pan));
        }
        // R480 phonograph vinyl crackle: dust pops on a record still playing
        self.phono_crackle_timer -= dt;
        if self.phono_crackle_timer <= 0.0 && !self.pp.phonograph_positions.is_empty() {
            let (_, p) = pick_pos(&self.pp.phonograph_positions);
            let (pan, dn) = self.prop_pan(p, 0.6, 0.8, 8.0);
            let cg = 0.012 * eg * (1.0 - dn * 0.5);
            for i in 0..4 {
                self.noise_burst(0.015, 5000.0 + rnd() * 2000.0, 5.0, cg * (0.6 + rnd() * 0.4), "highpass", None, i as f64 * 0.06, pan);
            }
            self.phono_crackle_timer = (8.0 + rnd() * 8.0) * etm;
            self.log_ambient("phono-crackle", Some(p.x), Some(p.z), Some(pan));
        }
        // R480 hanging brass lantern rope creak
        self.lantern_creak_timer -= dt;
        if self.lantern_creak_timer <= 0.0 && !self.pp.brass_lantern_positions.is_empty() {
            let (_, l) = pick_pos(&self.pp.brass_lantern_positions);
            let (pan, dn) = self.prop_pan(l, 0.6, 0.8, 8.0);
            let cg2 = 0.014 * eg * (1.0 - dn * 0.5);
            self.blip(140.0, 0.5, "sawtooth", cg2, Some(90.0), 0.0, pan);
            self.noise_burst(0.35, 220.0, 2.0, cg2 * 0.4, "bandpass", Some(160.0), 0.05, pan);
            self.lantern_creak_timer = (18.0 + rnd() * 22.0) * etm;
            self.log_ambient("lantern-creak", Some(l.x), Some(l.z), Some(pan));
        }
        // R481 sewing machine whirr + stitches
        self.sewing_whirr_timer -= dt;
        if self.sewing_whirr_timer <= 0.0 && !self.pp.sewing_positions.is_empty() {
            let (_, s2) = pick_pos(&self.pp.sewing_positions);
            let (pan, dn) = self.prop_pan(s2, 0.6, 0.8, 7.0);
            let sg = 0.012 * eg * (1.0 - dn * 0.5);
            self.blip(220.0, 0.5, "sawtooth", sg, Some(200.0), 0.0, pan);
            for i in 0..3 {
                self.blip(1800.0, 0.02, "square", sg * 0.6, None, 0.08 + i as f64 * 0.08, pan);
            }
            self.sewing_whirr_timer = (15.0 + rnd() * 15.0) * etm;
            self.log_ambient("sewing-whirr", Some(s2.x), Some(s2.z), Some(pan));
        }
        // R481 pocket watch tick-tock (< 3 m)
        self.watch_tick_timer -= dt;
        if self.watch_tick_timer <= 0.0 && !self.pp.pocket_watch_positions.is_empty() {
            let (_, w2) = pick_pos(&self.pp.pocket_watch_positions);
            if self.prop_d2(w2) < 9.0 {
                let (pan, dn) = self.prop_pan(w2, 0.7, 0.8, 3.0);
                let wg = 0.010 * eg * (1.0 - dn * 0.7);
                self.blip(3200.0, 0.012, "square", wg, None, 0.0, pan);
                self.blip(2800.0, 0.012, "square", wg * 0.9, None, 0.5, pan);
                self.log_ambient("watch-tick", Some(w2.x), Some(w2.z), Some(pan));
            }
            self.watch_tick_timer = (3.0 + rnd() * 3.0) * etm;
        }
        // R482 bat squeak
        self.bat_squeak_timer -= dt;
        if self.bat_squeak_timer <= 0.0 && !self.pp.bat_positions.is_empty() {
            let (_, b) = pick_pos(&self.pp.bat_positions);
            let (pan, dn) = self.prop_pan(b, 0.7, 0.8, 8.0);
            let bg = 0.012 * eg * (1.0 - dn * 0.5);
            self.blip(6000.0, 0.04, "sine", bg, Some(4000.0), 0.0, pan);
            self.blip(5000.0, 0.03, "sine", bg * 0.7, Some(3500.0), 0.05, pan);
            self.bat_squeak_timer = (25.0 + rnd() * 30.0) * etm;
            self.log_ambient("bat-squeak", Some(b.x), Some(b.z), Some(pan));
        }
        // R482 spinning globe wood creak
        self.globe_creak_timer -= dt;
        if self.globe_creak_timer <= 0.0 && !self.pp.globe_positions.is_empty() {
            let (_, g2) = pick_pos(&self.pp.globe_positions);
            let (pan, dn) = self.prop_pan(g2, 0.6, 0.8, 7.0);
            let gg = 0.010 * eg * (1.0 - dn * 0.5);
            self.blip(180.0, 0.4, "sawtooth", gg, Some(130.0), 0.0, pan);
            self.noise_burst(0.25, 340.0, 3.0, gg * 0.4, "bandpass", Some(260.0), 0.02, pan);
            self.globe_creak_timer = (22.0 + rnd() * 23.0) * etm;
            self.log_ambient("globe-creak", Some(g2.x), Some(g2.z), Some(pan));
        }
        // R483 hourglass sand whoosh (< 3 m)
        self.hourglass_whoosh_timer -= dt;
        if self.hourglass_whoosh_timer <= 0.0 && !self.pp.hourglass_positions.is_empty() {
            let (_, hg2) = pick_pos(&self.pp.hourglass_positions);
            if self.prop_d2(hg2) < 9.0 {
                let (pan, dn) = self.prop_pan(hg2, 0.7, 0.8, 3.0);
                let hg2g = 0.008 * eg * (1.0 - dn * 0.7);
                self.noise_burst(0.6, 3000.0, 4.0, hg2g, "bandpass", Some(3200.0), 0.0, pan);
                self.log_ambient("hourglass-whoosh", Some(hg2.x), Some(hg2.z), Some(pan));
            }
            self.hourglass_whoosh_timer = (4.0 + rnd() * 4.0) * etm;
        }
        // R485 ghost sheet whisper
        self.ghost_whisper_timer -= dt;
        if self.ghost_whisper_timer <= 0.0 && !self.pp.ghost_positions.is_empty() {
            let (_, gh) = pick_pos(&self.pp.ghost_positions);
            let (pan, dn) = self.prop_pan(gh, 0.7, 0.8, 9.0);
            let ghg = 0.014 * eg * (1.0 - dn * 0.5);
            self.noise_burst(0.7, 800.0, 1.2, ghg, "bandpass", Some(400.0), 0.0, pan);
            self.blip(90.0, 0.6, "sine", ghg * 0.3, Some(60.0), 0.1, pan);
            self.ghost_whisper_timer = (30.0 + rnd() * 40.0) * etm;
            self.log_ambient("ghost-whisper", Some(gh.x), Some(gh.z), Some(pan));
        }
        // R486 chain jangle (< 4 m)
        self.chain_jangle_timer -= dt;
        if self.chain_jangle_timer <= 0.0 && !self.pp.chain_positions.is_empty() {
            let (_, ch) = pick_pos(&self.pp.chain_positions);
            if self.prop_d2(ch) < 16.0 {
                let (pan, dn) = self.prop_pan(ch, 0.7, 0.8, 4.0);
                let chg = 0.014 * eg * (1.0 - dn * 0.6);
                self.noise_burst(0.03, 4200.0, 8.0, chg, "bandpass", Some(3600.0), 0.0, pan);
                self.noise_burst(0.03, 3800.0, 8.0, chg * 0.7, "bandpass", Some(3200.0), 0.08 + rnd() * 0.05, pan);
                self.log_ambient("chain-jangle", Some(ch.x), Some(ch.z), Some(pan));
            }
            self.chain_jangle_timer = (8.0 + rnd() * 10.0) * etm;
        }
        // R487 paper rustle near music sheets / journals (< 5 m)
        self.paper_rustle_timer -= dt;
        if self.paper_rustle_timer <= 0.0 && !self.pp.paper_positions.is_empty() {
            let (_, pp) = pick_pos(&self.pp.paper_positions);
            if self.prop_d2(pp) < 25.0 {
                let (pan, dn) = self.prop_pan(pp, 0.7, 0.8, 5.0);
                let ppg = 0.010 * eg * (1.0 - dn * 0.6);
                self.noise_burst(0.08, 4000.0, 3.0, ppg, "highpass", Some(5000.0), 0.0, pan);
                self.noise_burst(0.06, 5000.0, 3.0, ppg * 0.7, "highpass", Some(6000.0), 0.05, pan);
                self.log_ambient("paper-rustle", Some(pp.x), Some(pp.z), Some(pan));
            }
            self.paper_rustle_timer = (20.0 + rnd() * 25.0) * etm;
        }
        // R488 toy piano random note — "who pressed it?" (< 6 m)
        self.piano_chord_timer -= dt;
        if self.piano_chord_timer <= 0.0 && !self.pp.piano_positions.is_empty() {
            let (_, pn) = pick_pos(&self.pp.piano_positions);
            if self.prop_d2(pn) < 36.0 {
                let (pan, dn) = self.prop_pan(pn, 0.7, 0.8, 6.0);
                let png = 0.020 * eg * (1.0 - dn * 0.5);
                const NOTES: [f64; 8] = [261.63, 293.66, 329.63, 349.23, 392.00, 440.00, 493.88, 523.25];
                let f = NOTES[pick_index(NOTES.len())];
                self.blip(f, 0.4, "sine", png, None, 0.0, pan);
                self.blip(f * 2.0, 0.3, "sine", png * 0.3, None, 0.005, pan);
                self.log_ambient("piano-note", Some(pn.x), Some(pn.z), Some(pan));
            }
            self.piano_chord_timer = (40.0 + rnd() * 50.0) * etm;
        }
        // R497 rocking chair creak, one per rock (< 6 m)
        self.chair_creak_timer -= dt;
        if self.chair_creak_timer <= 0.0 && !self.pp.chair_positions.is_empty() {
            let (i, ch) = pick_pos(&self.pp.chair_positions);
            if self.prop_d2(ch) < 36.0 {
                let (pan, dn) = self.prop_pan(ch, 0.7, 0.8, 6.0);
                let chg = 0.014 * eg * (1.0 - dn * 0.5);
                self.blip(200.0, 0.45, "sawtooth", chg, Some(140.0), 0.0, pan);
                self.noise_burst(0.30, 380.0, 3.0, chg * 0.4, "bandpass", Some(260.0), 0.05, pan);
                self.log_ambient("chair-creak", Some(ch.x), Some(ch.z), Some(pan));
            }
            // next creak at the rock period (one creak per rock)
            self.chair_creak_timer = self.chair_period(i) * etm;
        }
        // R504 CRT TV static hiss (< 4 m)
        self.crt_hiss_timer -= dt;
        if self.crt_hiss_timer <= 0.0 && !self.pp.crt_positions.is_empty() {
            let (_, t2) = pick_pos(&self.pp.crt_positions);
            if self.prop_d2(t2) < 16.0 {
                let (pan, dn) = self.prop_pan(t2, 0.7, 0.8, 4.0);
                let tg = 0.014 * eg * (1.0 - dn * 0.6);
                self.noise_burst(0.4, 5000.0, 0.5, tg, "highpass", None, 0.0, pan);
                self.blip(60.0, 0.4, "sine", tg * 0.3, None, 0.01, pan);
                self.log_ambient("crt-hiss", Some(t2.x), Some(t2.z), Some(pan));
            }
            self.crt_hiss_timer = (4.0 + rnd() * 6.0) * etm;
        }
        // R508 exit sign buzz (< 5 m)
        self.exit_buzz_timer -= dt;
        if self.exit_buzz_timer <= 0.0 && !self.pp.exit_sign_positions.is_empty() {
            let (_, e2) = pick_pos(&self.pp.exit_sign_positions);
            if self.prop_d2(e2) < 25.0 {
                let (pan, dn) = self.prop_pan(e2, 0.7, 0.8, 5.0);
                let egn = 0.010 * eg * (1.0 - dn * 0.5);
                self.blip(120.0, 0.5, "sawtooth", egn, None, 0.0, pan);
                self.blip(240.0, 0.5, "sine", egn * 0.4, None, 0.0, pan);
                self.log_ambient("exit-buzz", Some(e2.x), Some(e2.z), Some(pan));
            }
            self.exit_buzz_timer = (6.0 + rnd() * 6.0) * etm;
        }
        // R520 puddle drip (< 5 m)
        self.puddle_drip_timer -= dt;
        if self.puddle_drip_timer <= 0.0 && !self.pp.puddle_positions.is_empty() {
            let (_, p2) = pick_pos(&self.pp.puddle_positions);
            if self.prop_d2(p2) < 25.0 {
                let (pan, dn) = self.prop_pan(p2, 0.6, 0.8, 5.0);
                let pg = 0.014 * eg * (1.0 - dn * 0.5);
                self.blip(1400.0 * ld, 0.05, "sine", pg, Some(700.0 * ld), 0.0, pan);
                self.blip(140.0 * ld, 0.12, "sine", pg * 0.6, Some(80.0 * ld), 0.03, pan);
                self.log_ambient("puddle-drip", Some(p2.x), Some(p2.z), Some(pan));
            }
            self.puddle_drip_timer = (8.0 + rnd() * 8.0) * etm;
        }
        // R524 chandelier crystal chime (< 6 m)
        self.chandelier_chime_timer -= dt;
        if self.chandelier_chime_timer <= 0.0 && !self.pp.chandelier_positions.is_empty() {
            let (_, ch2) = pick_pos(&self.pp.chandelier_positions);
            if self.prop_d2(ch2) < 36.0 {
                let (pan, dn) = self.prop_pan(ch2, 0.7, 0.8, 6.0);
                let cg2 = 0.010 * eg * (1.0 - dn * 0.5);
                let base_f = 2100.0 + rnd() * 400.0;
                self.blip(base_f, 0.6, "sine", cg2, None, 0.0, pan);
                self.blip(base_f * 1.25, 0.5, "sine", cg2 * 0.7, None, 0.05, pan);
                self.blip(base_f * 1.5, 0.4, "sine", cg2 * 0.5, None, 0.10, pan);
                self.log_ambient("chandelier-chime", Some(ch2.x), Some(ch2.z), Some(pan));
            }
            self.chandelier_chime_timer = (20.0 + rnd() * 20.0) * etm;
        }
        // R524 cassette tape hiss (< 3 m)
        self.cassette_hiss_timer -= dt;
        if self.cassette_hiss_timer <= 0.0 && !self.pp.cassette_positions.is_empty() {
            let (_, cs) = pick_pos(&self.pp.cassette_positions);
            if self.prop_d2(cs) < 9.0 {
                let (pan, dn) = self.prop_pan(cs, 0.7, 0.8, 3.0);
                let csg = 0.008 * eg * (1.0 - dn * 0.6);
                self.noise_burst(0.5, 3500.0, 0.6, csg, "highpass", None, 0.0, pan);
                self.log_ambient("cassette-hiss", Some(cs.x), Some(cs.z), Some(pan));
            }
            self.cassette_hiss_timer = (10.0 + rnd() * 10.0) * etm;
        }
        // R525 teacup porcelain clink (< 3.5 m)
        self.teacup_clink_timer -= dt;
        if self.teacup_clink_timer <= 0.0 && !self.pp.teacup_positions.is_empty() {
            let (_, tc) = pick_pos(&self.pp.teacup_positions);
            if self.prop_d2(tc) < 12.25 {
                let (pan, dn) = self.prop_pan(tc, 0.7, 0.8, 3.5);
                let tcg = 0.008 * eg * (1.0 - dn * 0.6);
                self.blip(3200.0, 0.03, "sine", tcg, None, 0.0, pan);
                self.blip(4200.0, 0.02, "sine", tcg * 0.6, None, 0.02, pan);
                self.log_ambient("teacup-clink", Some(tc.x), Some(tc.z), Some(pan));
            }
            self.teacup_clink_timer = (25.0 + rnd() * 25.0) * etm;
        }
        // R531 locker slam: sudden metal clang
        self.locker_slam_timer -= dt;
        if self.locker_slam_timer <= 0.0 && !self.pp.locker_positions.is_empty() {
            let (_, lk) = pick_pos(&self.pp.locker_positions);
            let (pan, dn) = self.prop_pan(lk, 0.7, 0.8, 10.0);
            let lkg = 0.028 * eg * (1.0 - dn * 0.5);
            self.blip(90.0, 0.06, "sine", lkg, Some(55.0), 0.0, pan);
            self.noise_burst(0.15, 2200.0, 5.0, lkg * 0.6, "bandpass", None, 0.03, pan);
            self.noise_burst(0.10, 4200.0, 6.0, lkg * 0.4, "bandpass", None, 0.10, pan);
            self.locker_slam_timer = (60.0 + rnd() * 60.0) * etm;
            self.log_ambient("locker-slam", Some(lk.x), Some(lk.z), Some(pan));
        }
        // R534 bulletin board flutter (< 5 m)
        self.bulletin_flutter_timer -= dt;
        if self.bulletin_flutter_timer <= 0.0 && !self.pp.bulletin_positions.is_empty() {
            let (_, bb) = pick_pos(&self.pp.bulletin_positions);
            if self.prop_d2(bb) < 25.0 {
                let (pan, dn) = self.prop_pan(bb, 0.7, 0.8, 5.0);
                let bbg = 0.010 * eg * (1.0 - dn * 0.5);
                for i in 0..3 {
                    self.noise_burst(0.06, 4500.0, 3.0, bbg * (0.6 + rnd() * 0.4), "highpass", None, i as f64 * 0.05, pan);
                }
                self.log_ambient("bulletin-flutter", Some(bb.x), Some(bb.z), Some(pan));
            }
            self.bulletin_flutter_timer = (15.0 + rnd() * 15.0) * etm;
        }
        // R541 fire alarm pull beep (< 7 m)
        self.alarm_beep_timer -= dt;
        if self.alarm_beep_timer <= 0.0 && !self.pp.alarm_pull_positions.is_empty() {
            let (_, al) = pick_pos(&self.pp.alarm_pull_positions);
            if self.prop_d2(al) < 49.0 {
                let (pan, dn) = self.prop_pan(al, 0.7, 0.8, 7.0);
                let alg = 0.014 * eg * (1.0 - dn * 0.5);
                self.blip(3200.0, 0.06, "square", alg, None, 0.0, pan);
                self.blip(3200.0, 0.06, "square", alg, None, 0.10, pan);
                self.log_ambient("alarm-beep", Some(al.x), Some(al.z), Some(pan));
            }
            self.alarm_beep_timer = (35.0 + rnd() * 35.0) * etm;
        }
        // R568 ceiling fan whoosh (< 5 m, nearly under it)
        self.fan_whoosh_timer -= dt;
        if self.fan_whoosh_timer <= 0.0 && !self.pp.fan_positions.is_empty() {
            let (_, fnp) = pick_pos(&self.pp.fan_positions);
            if self.prop_d2(fnp) < 25.0 {
                let (pan, dn) = self.prop_pan(fnp, 0.5, 0.7, 5.0);
                let fng = 0.012 * eg * (1.0 - dn * 0.7);
                self.noise_burst(0.9, 260.0, 1.2, fng, "bandpass", Some(220.0), 0.0, pan);
                self.log_ambient("fan-whoosh", Some(fnp.x), Some(fnp.z), Some(pan));
            }
            self.fan_whoosh_timer = (8.0 + rnd() * 6.0) * etm;
        }
        // R569 fan bearing squeak (< 6 m)
        self.fan_squeak_timer -= dt;
        if self.fan_squeak_timer <= 0.0 && !self.pp.fan_positions.is_empty() {
            let (_, fs) = pick_pos(&self.pp.fan_positions);
            if self.prop_d2(fs) < 36.0 {
                let (pan, dn) = self.prop_pan(fs, 0.6, 0.7, 6.0);
                let fsg = 0.011 * eg * (1.0 - dn * 0.6);
                self.blip(2600.0, 0.05, "sine", fsg, Some(2300.0), 0.0, pan);
                self.log_ambient("fan-squeak", Some(fs.x), Some(fs.z), Some(pan));
            }
            self.fan_squeak_timer = (25.0 + rnd() * 30.0) * etm;
        }
        // R584 smoke detector low-battery chirp (< 8 m)
        self.detector_chirp_timer -= dt;
        if self.detector_chirp_timer <= 0.0 && !self.pp.smoke_detector_positions.is_empty() {
            let (_, sd) = pick_pos(&self.pp.smoke_detector_positions);
            if self.prop_d2(sd) < 64.0 {
                let (pan, dn) = self.prop_pan(sd, 0.6, 0.7, 8.0);
                let sdg = 0.013 * eg * (1.0 - dn * 0.5);
                self.blip(3800.0, 0.08, "sine", sdg, None, 0.0, pan);
                self.log_ambient("detector-chirp", Some(sd.x), Some(sd.z), Some(pan));
            }
            self.detector_chirp_timer = (90.0 + rnd() * 110.0) * etm;
        }
    }

    /// The impossible outdoor sounds (R600–R642, not tied to props) and the
    /// R608–R672 props: bird … teapot (lines 64181–64409).
    pub(crate) fn amb_props_outdoor(&mut self, dt: f64, m: &AmbMix) {
        let (eg, etm) = (m.gain, m.timer_mul);
        // R600 distant bird chirp — an impossibly-far songbird
        self.bird_chirp_timer -= dt;
        if self.bird_chirp_timer <= 0.0 {
            let pan = (rnd() - 0.5) * 1.2;
            let bg = 0.010 * eg;
            self.blip(2400.0, 0.06, "sine", bg, Some(2200.0), 0.0, pan);
            self.blip(2600.0, 0.05, "sine", bg, None, 0.10, pan);
            self.blip(2100.0, 0.08, "sine", bg, Some(1900.0), 0.19, pan);
            let (x, z) = (self.player.pos.x + rnd() * 20.0 - 10.0, self.player.pos.z + rnd() * 20.0 - 10.0);
            self.log_ambient("bird-chirp", Some(x), Some(z), Some(pan));
            self.bird_chirp_timer = (120.0 + rnd() * 180.0) * etm;
        }
        // R600 distant cricket trill
        self.cricket_chirp_timer -= dt;
        if self.cricket_chirp_timer <= 0.0 {
            let pan = (rnd() - 0.5) * 1.2;
            let cg = 0.008 * eg;
            for i in 0..4 {
                self.blip(4500.0, 0.025, "square", cg, None, i as f64 * 0.06, pan);
            }
            let (x, z) = (self.player.pos.x + rnd() * 20.0 - 10.0, self.player.pos.z + rnd() * 20.0 - 10.0);
            self.log_ambient("cricket-chirp", Some(x), Some(z), Some(pan));
            self.cricket_chirp_timer = (100.0 + rnd() * 120.0) * etm;
        }
        // R601 distant thunder rumble (non-directional)
        self.thunder_rumble_timer -= dt;
        if self.thunder_rumble_timer <= 0.0 {
            let tg = 0.020 * eg;
            self.noise_burst(1.5, 80.0, 0.4, tg, "lowpass", Some(55.0), 0.0, 0.0);
            self.noise_burst(0.4, 300.0, 0.6, tg * 0.5, "bandpass", Some(220.0), 0.3, 0.0);
            let (x, z) = (self.player.pos.x, self.player.pos.z);
            self.log_ambient("thunder-rumble", Some(x), Some(z), Some(0.0));
            self.thunder_rumble_timer = (200.0 + rnd() * 300.0) * etm;
        }
        // R608 coin cluster tink (< 3.5 m)
        self.coin_tink_timer -= dt;
        if self.coin_tink_timer <= 0.0 && !self.pp.coin_positions.is_empty() {
            let (_, ci) = pick_pos(&self.pp.coin_positions);
            if self.prop_d2(ci) < 12.25 {
                let (pan, dn) = self.prop_pan(ci, 0.6, 0.7, 3.5);
                let cig = 0.011 * eg * (1.0 - dn * 0.6);
                self.blip(4500.0, 0.04, "sine", cig, Some(4200.0), 0.0, pan);
                self.log_ambient("coin-tink", Some(ci.x), Some(ci.z), Some(pan));
            }
            self.coin_tink_timer = (20.0 + rnd() * 25.0) * etm;
        }
        // R610 distant wind gust (non-directional)
        self.wind_gust_timer -= dt;
        if self.wind_gust_timer <= 0.0 {
            let wg = 0.016 * eg;
            self.noise_burst(2.0, 180.0, 0.5, wg, "bandpass", Some(320.0), 0.0, 0.0);
            let (x, z) = (self.player.pos.x, self.player.pos.z);
            self.log_ambient("wind-gust", Some(x), Some(z), Some(0.0));
            self.wind_gust_timer = (150.0 + rnd() * 180.0) * etm;
        }
        // R629 distant frog croak
        self.frog_croak_timer -= dt;
        if self.frog_croak_timer <= 0.0 {
            let pan = (rnd() - 0.5) * 1.2;
            let fg = 0.012 * eg;
            self.blip(160.0, 0.14, "sawtooth", fg, Some(140.0), 0.0, pan);
            self.blip(150.0, 0.12, "sawtooth", fg, Some(130.0), 0.20, pan);
            if rnd() < 0.5 { self.blip(155.0, 0.10, "sawtooth", fg, Some(135.0), 0.36, pan); }
            let (x, z) = (self.player.pos.x + rnd() * 20.0 - 10.0, self.player.pos.z + rnd() * 20.0 - 10.0);
            self.log_ambient("frog-croak", Some(x), Some(z), Some(pan));
            self.frog_croak_timer = (180.0 + rnd() * 220.0) * etm;
        }
        // R629 distant owl hoot
        self.owl_hoot_timer -= dt;
        if self.owl_hoot_timer <= 0.0 {
            let pan = (rnd() - 0.5) * 1.2;
            let og = 0.014 * eg;
            self.blip(340.0, 0.35, "sine", og, Some(300.0), 0.0, pan);
            self.blip(280.0, 0.22, "sine", og, Some(260.0), 0.60, pan);
            let (x, z) = (self.player.pos.x + rnd() * 20.0 - 10.0, self.player.pos.z + rnd() * 20.0 - 10.0);
            self.log_ambient("owl-hoot", Some(x), Some(z), Some(pan));
            self.owl_hoot_timer = (200.0 + rnd() * 250.0) * etm;
        }
        // R633 ticking clock tick-tock (< 4 m)
        self.tick_tock_timer -= dt;
        if self.tick_tock_timer <= 0.0 && !self.pp.clock_positions.is_empty() {
            let (_, ck) = pick_pos(&self.pp.clock_positions);
            if self.prop_d2(ck) < 16.0 {
                let (pan, dn) = self.prop_pan(ck, 0.5, 0.6, 4.0);
                let ckg = 0.010 * eg * (1.0 - dn * 0.6);
                self.blip(2200.0, 0.03, "square", ckg, None, 0.0, pan);
                self.blip(1600.0, 0.03, "square", ckg, None, 0.42, pan);
                self.log_ambient("tick-tock", Some(ck.x), Some(ck.z), Some(pan));
            }
            self.tick_tock_timer = (12.0 + rnd() * 13.0) * etm;
        }
        // R636 distant wolf howl
        self.wolf_howl_timer -= dt;
        if self.wolf_howl_timer <= 0.0 {
            let pan = (rnd() - 0.5) * 1.2;
            let whg = 0.014 * eg;
            self.blip(400.0, 1.4, "sine", whg, Some(550.0), 0.0, pan);
            self.blip(550.0, 1.2, "sine", whg, Some(380.0), 1.4, pan);
            let (x, z) = (self.player.pos.x + rnd() * 20.0 - 10.0, self.player.pos.z + rnd() * 20.0 - 10.0);
            self.log_ambient("wolf-howl", Some(x), Some(z), Some(pan));
            self.wolf_howl_timer = (300.0 + rnd() * 300.0) * etm;
        }
        // R639 distant rain patter: hiss + individual drops
        self.rain_patter_timer -= dt;
        if self.rain_patter_timer <= 0.0 {
            let rg = 0.011 * eg;
            self.noise_burst(2.5, 4200.0, 0.6, rg, "highpass", None, 0.0, 0.0);
            let drop_count = 5 + (rnd() * 4.0).floor() as i32;
            for _ in 0..drop_count {
                self.noise_burst(0.02, 5800.0, 3.0, rg * 0.6, "bandpass", None, rnd() * 2.3, (rnd() - 0.5) * 1.0);
            }
            let (x, z) = (self.player.pos.x, self.player.pos.z);
            self.log_ambient("rain-patter", Some(x), Some(z), Some(0.0));
            self.rain_patter_timer = (220.0 + rnd() * 180.0) * etm;
        }
        // R642 leaf rustle: wind through dry leaves
        self.leaf_rustle_timer -= dt;
        if self.leaf_rustle_timer <= 0.0 {
            let lrg = 0.010 * eg;
            let pan = (rnd() - 0.5) * 1.0;
            self.noise_burst(0.6, 3200.0, 1.0, lrg, "bandpass", Some(2400.0), 0.0, pan);
            for _ in 0..3 {
                self.noise_burst(0.03, 6000.0 + rnd() * 2000.0, 2.0, lrg * 0.5, "bandpass", None, 0.05 + rnd() * 0.4, pan);
            }
            let (x, z) = (self.player.pos.x + rnd() * 15.0 - 7.5, self.player.pos.z + rnd() * 15.0 - 7.5);
            self.log_ambient("leaf-rustle", Some(x), Some(z), Some(pan));
            self.leaf_rustle_timer = (60.0 + rnd() * 80.0) * etm;
        }
        // R664 cuckoo call near cuckoo clocks (< 6 m)
        self.cuckoo_call_timer -= dt;
        if self.cuckoo_call_timer <= 0.0 && !self.pp.cuckoo_clock_positions.is_empty() {
            let (_, cc) = pick_pos(&self.pp.cuckoo_clock_positions);
            if self.prop_d2(cc) < 36.0 {
                let (pan, dn) = self.prop_pan(cc, 0.6, 0.7, 6.0);
                let ccg = 0.013 * eg * (1.0 - dn * 0.5);
                self.blip(700.0, 0.14, "sine", ccg, Some(680.0), 0.0, pan);
                self.blip(560.0, 0.16, "sine", ccg, Some(540.0), 0.20, pan);
                self.log_ambient("cuckoo-call", Some(cc.x), Some(cc.z), Some(pan));
            }
            self.cuckoo_call_timer = (90.0 + rnd() * 130.0) * etm;
        }
        // R666 plush shuffle — the stuffed animal stirred (< 3 m)
        self.plush_shuffle_timer -= dt;
        if self.plush_shuffle_timer <= 0.0 && !self.pp.plush_positions.is_empty() {
            let (_, pl) = pick_pos(&self.pp.plush_positions);
            if self.prop_d2(pl) < 9.0 {
                let (pan, dn) = self.prop_pan(pl, 0.5, 0.6, 3.0);
                let plg = 0.009 * eg * (1.0 - dn * 0.6);
                self.noise_burst(0.15, 2400.0, 0.8, plg, "bandpass", Some(1800.0), 0.0, pan);
                self.log_ambient("plush-shuffle", Some(pl.x), Some(pl.z), Some(pan));
            }
            self.plush_shuffle_timer = (150.0 + rnd() * 170.0) * etm;
        }
        // R669 gumball dispenser rattle (< 4 m)
        self.gumball_rattle_timer -= dt;
        if self.gumball_rattle_timer <= 0.0 && !self.pp.gumball_positions.is_empty() {
            let (_, gp) = pick_pos(&self.pp.gumball_positions);
            if self.prop_d2(gp) < 16.0 {
                let (pan, dn) = self.prop_pan(gp, 0.6, 0.7, 4.0);
                let gpg = 0.010 * eg * (1.0 - dn * 0.5);
                let click_count = 3 + (rnd() * 3.0).floor() as i32;
                for i in 0..click_count {
                    self.blip(3400.0 + rnd() * 800.0, 0.03, "sine", gpg, None, i as f64 * 0.08 + rnd() * 0.05, pan);
                }
                self.log_ambient("gumball-rattle", Some(gp.x), Some(gp.z), Some(pan));
            }
            self.gumball_rattle_timer = (40.0 + rnd() * 50.0) * etm;
        }
        // R672 teapot whistle — "who's boiling water?" (< 5 m)
        self.teapot_whistle_timer -= dt;
        if self.teapot_whistle_timer <= 0.0 && !self.pp.teapot_positions.is_empty() {
            let (_, tp) = pick_pos(&self.pp.teapot_positions);
            if self.prop_d2(tp) < 25.0 {
                let (pan, dn) = self.prop_pan(tp, 0.6, 0.7, 5.0);
                let tpg = 0.010 * eg * (1.0 - dn * 0.5);
                self.blip(2200.0, 0.9, "sine", tpg, Some(2800.0), 0.0, pan);
                self.noise_burst(0.4, 3200.0, 1.0, tpg * 0.4, "highpass", None, 0.6, pan);
                self.log_ambient("teapot-whistle", Some(tp.x), Some(tp.z), Some(pan));
            }
            self.teapot_whistle_timer = (200.0 + rnd() * 250.0) * etm;
        }
    }
}
