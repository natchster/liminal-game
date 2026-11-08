//! The game's audio section: `audio_init`, the
//! persistent ambience nodes the update loop drives (drone, payphone, vacuum,
//! safe-room hum, room tone, rain hiss, stamina breath), and the `sfx` table.
//!
//! The audio context, master gain, noise buffer and mute flag live on `Game`
//! (`g.ac`, `g.master`, `g.noise_buf`, `g.muted`); the one-shot voices are
//! `g.blip(..)` / `g.noise_burst(..)` in game.rs. Everything else lives in
//! `g.audio`.

use crate::audio::soundscape::{Soundscape, SoundscapeOpts};
use crate::base::game::Game;
use crate::base::rng::{choice, rand2, rnd, with_native};
use engine::prelude::*;
use std::rc::Rc;

/// Every persistent node of the audio graph (built once by `audio_init`).
#[derive(Default)]
pub struct AudioSys {
    /// Entity drone (detuned saws through a lowpass), proximity-driven
    pub drone_gain: Option<AudioNode>,
    /// Payphone ring (440 + 480 Hz), gated from the update loop
    pub phone_gain: Option<AudioNode>,
    /// Silent legacy node (the hum lives in the soundscape now)
    pub hum_gain: Option<AudioNode>,
    /// Robot-vacuum motor + brush noise, proximity-driven
    pub vacuum_gain: Option<AudioNode>,
    /// Safe-room hum bus
    pub safe_room_gain: Option<AudioNode>,
    /// Safe-room tremolo LFO (oscillator; `.frequency` is retuned per level)
    pub safe_room_lfo: Option<AudioNode>,
    /// The 330 Hz tremolo carrier (oscillator)
    pub safe_room_trem_carrier: Option<AudioNode>,
    /// R200 breath layer output gain
    pub stamina_breath_gain: Option<AudioNode>,
    /// Breath cadence LFO (oscillator)
    pub stamina_breath_lfo: Option<AudioNode>,
    /// Breath noise lowpass (`.frequency`, `.Q` driven)
    pub stamina_breath_lp: Option<AudioNode>,
    /// R204 chest-thump sub gain
    pub stamina_breath_sub_gain: Option<AudioNode>,
    /// R204 StereoPanner after the breath gain
    pub stamina_breath_panner: Option<AudioNode>,
    /// Started, never connected (the sweep is done in the update loop)
    pub stamina_breath_pan_lfo: Option<AudioNode>,
    /// R206 90 Hz sub oscillator (per-weather `.frequency`)
    pub stamina_breath_sub_osc: Option<AudioNode>,
    /// R96 always-on hiss floor
    pub room_tone_gain: Option<AudioNode>,
    /// R97 room-tone lowpass (per-weather Hz)
    pub room_tone_lp: Option<AudioNode>,
    /// R120 rain hiss (window-view scaled)
    pub rain_hiss_gain: Option<AudioNode>,
    /// The spatial soundscape (`audio::soundscape`) — `None` before
    /// `audio_init` or when `nosound` is set
    pub soundscape: Option<Rc<Soundscape>>,
    /// Debug switch: run without the soundscape (A/B comparison)
    pub nosound: bool,
}

/// The keys of the sfx table, in declaration order.
pub const SFX_NAMES: [&str; 30] = [
    "step",
    "pickup",
    "door",
    "scream",
    "thump",
    "whisper",
    "tick",
    "thud",
    "clank",
    "moan",
    "doorSlam",
    "distantScream",
    "radioStatic",
    "paperRustle",
    "drip",
    "dialTone",
    "distantFootsteps",
    "glitch",
    "ghostStep",
    "breath",
    "laugh",
    "chime",
    "cry",
    "relay",
    "wetStep",
    "splash",
    "piano",
    "dripHeavy",
    "pageFlip",
    "scrape",
];

impl Game {
    /// Create the AudioContext and the whole persistent graph.
    /// A second call only resumes the context.
    pub fn audio_init(&mut self) {
        if let Some(ac) = &self.ac {
            if ac.state() == "suspended" {
                ac.resume();
            }
            return;
        }
        // the context always opens (a silent clock without a device)
        self.audio_init_with(AudioContext::new());
    }

    /// `audio_init()` on a context the caller made (tests mute it first). No-op
    /// (beyond resuming) if audio is already up.
    pub fn audio_init_with(&mut self, ac: AudioContext) {
        if let Some(ac) = &self.ac {
            if ac.state() == "suspended" {
                ac.resume();
            }
            return;
        }
        // Audio starts from a click, outside any `with_seed` build: its `rnd()`
        // draws (noise buffer, soundscape) come from the native generator. Keep
        // it that way even if a caller happens to be inside a seeded build.
        with_native(|| self.audio_build(ac));
    }

    fn audio_build(&mut self, ac: AudioContext) {
        let master = ac.create_gain();
        master.gain().set_value(if self.muted { 0.0 } else { 0.9 * self.settings.volume });
        master.connect(&ac.destination());

        let sr = ac.sample_rate();
        let noise_buf = ac.create_buffer(1, sr as usize, sr);
        {
            let mut nd = noise_buf.channel_mut(0);
            for v in nd.iter_mut() {
                *v = (rnd() * 2.0 - 1.0) as f32;
            }
        }

        // fluorescent hum now lives in the soundscape (a 120 Hz ballast series
        // placed at the nearest live fixture); `hum_gain` stays as a silent node for
        // anything that still references it
        let hum_gain = ac.create_gain();
        hum_gain.gain().set_value(0.0);
        hum_gain.connect(&master);
        // debug: `nosound` runs the game without the soundscape (A/B in the bench)
        let soundscape = if self.audio.nosound { None } else { Some(Rc::new(Soundscape::new(&ac, &master, SoundscapeOpts::default()))) };
        // entity drone: detuned saws through a lowpass, gain driven by proximity
        let lp = ac.create_biquad_filter();
        lp.set_type("lowpass");
        lp.frequency().set_value(240.0);
        let drone_gain = ac.create_gain();
        drone_gain.gain().set_value(0.0);
        lp.connect(&drone_gain);
        drone_gain.connect(&master);
        for f in [47.0, 47.65] {
            let o = ac.create_oscillator();
            o.set_type("sawtooth");
            o.frequency().set_value(f);
            o.connect(&lp);
            o.start(0.0);
        }
        // payphone ring: classic dual tone, gated from the update loop
        let phone_gain = ac.create_gain();
        phone_gain.gain().set_value(0.0);
        phone_gain.connect(&master);
        for f in [440.0, 480.0] {
            let o = ac.create_oscillator();
            o.set_type("sine");
            o.frequency().set_value(f);
            o.connect(&phone_gain);
            o.start(0.0);
        }
        // robot-vacuum hum: soothing low motor whine + gentle noise, gated by proximity
        let vacuum_gain = ac.create_gain();
        vacuum_gain.gain().set_value(0.0);
        vacuum_gain.connect(&master);
        let vlp = ac.create_biquad_filter();
        vlp.set_type("lowpass");
        vlp.frequency().set_value(500.0);
        vlp.connect(&vacuum_gain);
        // motor tone (two slightly detuned triangles at ~180Hz) — round and warm
        for f in [178.0, 182.0, 356.0] {
            let o = ac.create_oscillator();
            o.set_type("triangle");
            o.frequency().set_value(f);
            let g = ac.create_gain();
            g.gain().set_value(if f > 300.0 { 0.12 } else { 0.5 });
            o.connect(&g);
            g.connect(&vlp);
            o.start(0.0);
        }
        // brush noise (soft filtered noise for the "carpet swishing" texture)
        let brush_noise = ac.create_buffer_source();
        brush_noise.set_buffer(&noise_buf);
        brush_noise.set_loop(true);
        let brush_filt = ac.create_biquad_filter();
        brush_filt.set_type("bandpass");
        brush_filt.frequency().set_value(900.0);
        brush_filt.q().set_value(0.6);
        let brush_g = ac.create_gain();
        brush_g.gain().set_value(0.5);
        brush_noise.connect(&brush_filt);
        brush_filt.connect(&brush_g);
        brush_g.connect(&vacuum_gain);
        brush_noise.start(0.0);
        // safe-room hum: warm low sine (~110 Hz) + subtle harmonic + tremolo LFO.
        // R91: enriched with sub-bass (~55 Hz) + gently detuned choir triangle
        // pair (~220±3 Hz) under the tremolo for a fuller, more oppressive body.
        // Gain driven by `update_player` from proximity to the safe room.
        let safe_room_gain = ac.create_gain();
        safe_room_gain.gain().set_value(0.0);
        safe_room_gain.connect(&master);
        for f in [110.0, 220.0] {
            let o = ac.create_oscillator();
            o.set_type("sine");
            o.frequency().set_value(f);
            let og = ac.create_gain();
            og.gain().set_value(if f == 110.0 { 0.6 } else { 0.15 });
            o.connect(&og);
            og.connect(&safe_room_gain);
            o.start(0.0);
        }
        // Sub-bass fundamental — felt more than heard; the "room breathes" layer.
        {
            let sub = ac.create_oscillator();
            sub.set_type("sine");
            sub.frequency().set_value(55.0);
            let sub_g = ac.create_gain();
            sub_g.gain().set_value(0.35);
            sub.connect(&sub_g);
            sub_g.connect(&safe_room_gain);
            sub.start(0.0);
        }
        // Detuned choir triangle pair at ~220 Hz — subtle beating creates the
        // "not-quite-still" feel that matches the R23 haunted-liminal vibe.
        for detune in [-3.0, 3.0] {
            let c = ac.create_oscillator();
            c.set_type("triangle");
            c.frequency().set_value(220.0 + detune);
            let cg = ac.create_gain();
            cg.gain().set_value(0.06);
            c.connect(&cg);
            cg.connect(&safe_room_gain);
            c.start(0.0);
        }
        // gentle tremolo (LFO amplitude modulation of a mix stage)
        let trem = ac.create_gain();
        trem.gain().set_value(0.0);
        let safe_room_lfo = ac.create_oscillator();
        safe_room_lfo.set_type("sine");
        safe_room_lfo.frequency().set_value(0.35);
        let lfo_depth = ac.create_gain();
        lfo_depth.gain().set_value(0.10);
        safe_room_lfo.connect(&lfo_depth);
        lfo_depth.connect_param(&trem.gain());
        safe_room_lfo.start(0.0);
        let safe_room_trem_carrier = ac.create_oscillator();
        safe_room_trem_carrier.set_type("sine");
        safe_room_trem_carrier.frequency().set_value(330.0);
        safe_room_trem_carrier.connect(&trem);
        trem.connect(&safe_room_gain);
        safe_room_trem_carrier.start(0.0);
        // R96/R97 always-running room-tone hiss floor. Filter frequency +
        // gain reconfigured per-weather in `init_level` — wet = darker/warmer,
        // still = brighter/thinner, etc. All weathers get some tone so audio
        // never feels dead between events.
        let room_tone_gain = ac.create_gain();
        room_tone_gain.gain().set_value(0.0);
        room_tone_gain.connect(&master);
        let rt_src = ac.create_buffer_source();
        rt_src.set_buffer(&noise_buf);
        rt_src.set_loop(true);
        let room_tone_lp = ac.create_biquad_filter();
        room_tone_lp.set_type("lowpass");
        room_tone_lp.frequency().set_value(900.0);
        room_tone_lp.q().set_value(0.4);
        rt_src.connect(&room_tone_lp);
        room_tone_lp.connect(&room_tone_gain);
        rt_src.start(0.0);
        // R120 continuous rain hiss — always-running loop, bandpassed to a
        // "distant static rain" band. Gain is set only when the player is
        // facing a rain-window on a wet-weather level (see `update_ambient`).
        let rain_hiss_gain = ac.create_gain();
        rain_hiss_gain.gain().set_value(0.0);
        rain_hiss_gain.connect(&master);
        let rh_src = ac.create_buffer_source();
        rh_src.set_buffer(&noise_buf);
        rh_src.set_loop(true);
        let rh_bp = ac.create_biquad_filter();
        rh_bp.set_type("bandpass");
        rh_bp.frequency().set_value(2200.0);
        rh_bp.q().set_value(0.7);
        rh_src.connect(&rh_bp);
        rh_bp.connect(&rain_hiss_gain);
        rh_src.start(0.0);
        // R200 stamina breath layer: always-running lowpass noise gated by a
        // slow LFO (~0.35 Hz base = ~21 bpm exhausted breathing). Output gain
        // driven from update loop based on fatigue. Signal chain:
        //   noise → lp(220 Hz) → `br_trem` (LFO-modulated) → `stamina_breath_gain` → master
        // R204: `stamina_breath_gain` feeds a stereo panner (`stamina_breath_panner`),
        // swept ±0.2 at peak fatigue from the update loop. The pan LFO is
        // started but never connected. Sub-oscillator (90 Hz sine, LFO-gated)
        // adds chest-thump weight to complement the airway whistle.
        let stamina_breath_gain = ac.create_gain();
        stamina_breath_gain.gain().set_value(0.0);
        // (stereo panners are always available on this context)
        let stamina_breath_panner = ac.create_stereo_panner();
        stamina_breath_panner.pan().set_value(0.0);
        stamina_breath_gain.connect(&stamina_breath_panner);
        stamina_breath_panner.connect(&master);
        let stamina_breath_pan_lfo = ac.create_oscillator();
        stamina_breath_pan_lfo.set_type("sine");
        stamina_breath_pan_lfo.frequency().set_value(0.12);
        stamina_breath_pan_lfo.start(0.0);
        let br_src = ac.create_buffer_source();
        br_src.set_buffer(&noise_buf);
        br_src.set_loop(true);
        let stamina_breath_lp = ac.create_biquad_filter();
        stamina_breath_lp.set_type("lowpass");
        stamina_breath_lp.frequency().set_value(220.0);
        stamina_breath_lp.q().set_value(0.4);
        let br_trem = ac.create_gain();
        br_trem.gain().set_value(0.5); // baseline mid-point
        let stamina_breath_lfo = ac.create_oscillator();
        stamina_breath_lfo.set_type("sine");
        stamina_breath_lfo.frequency().set_value(0.35);
        let br_lfo_depth = ac.create_gain();
        br_lfo_depth.gain().set_value(0.5); // modulate ±0.5 around 0.5 = [0, 1]
        stamina_breath_lfo.connect(&br_lfo_depth);
        br_lfo_depth.connect_param(&br_trem.gain());
        stamina_breath_lfo.start(0.0);
        br_src.connect(&stamina_breath_lp);
        stamina_breath_lp.connect(&br_trem);
        br_trem.connect(&stamina_breath_gain);
        br_src.start(0.0);
        // R204 chest-thump sub-oscillator: 90 Hz sine, gated by its own gain
        // stage that shares the R200 tremolo LFO. Gain driven from the update
        // loop as breath magnitude × 0.020.
        let stamina_breath_sub_gain = ac.create_gain();
        stamina_breath_sub_gain.gain().set_value(0.0);
        stamina_breath_sub_gain.connect(&stamina_breath_gain);
        let stamina_breath_sub_osc = ac.create_oscillator();
        stamina_breath_sub_osc.set_type("sine");
        stamina_breath_sub_osc.frequency().set_value(90.0);
        let sub_trem = ac.create_gain();
        sub_trem.gain().set_value(0.5);
        let sub_lfo_depth = ac.create_gain();
        sub_lfo_depth.gain().set_value(0.5);
        stamina_breath_lfo.connect(&sub_lfo_depth);
        sub_lfo_depth.connect_param(&sub_trem.gain());
        stamina_breath_sub_osc.connect(&sub_trem);
        sub_trem.connect(&stamina_breath_sub_gain);
        stamina_breath_sub_osc.start(0.0);

        let nosound = self.audio.nosound;
        self.audio = AudioSys {
            drone_gain: Some(drone_gain),
            phone_gain: Some(phone_gain),
            hum_gain: Some(hum_gain),
            vacuum_gain: Some(vacuum_gain),
            safe_room_gain: Some(safe_room_gain),
            safe_room_lfo: Some(safe_room_lfo),
            safe_room_trem_carrier: Some(safe_room_trem_carrier),
            stamina_breath_gain: Some(stamina_breath_gain),
            stamina_breath_lfo: Some(stamina_breath_lfo),
            stamina_breath_lp: Some(stamina_breath_lp),
            stamina_breath_sub_gain: Some(stamina_breath_sub_gain),
            stamina_breath_panner: Some(stamina_breath_panner),
            stamina_breath_pan_lfo: Some(stamina_breath_pan_lfo),
            stamina_breath_sub_osc: Some(stamina_breath_sub_osc),
            room_tone_gain: Some(room_tone_gain),
            room_tone_lp: Some(room_tone_lp),
            rain_hiss_gain: Some(rain_hiss_gain),
            soundscape,
            nosound,
        };
        self.noise_buf = Some(noise_buf);
        self.master = Some(master);
        self.ac = Some(ac);
    }

    /// The M key / sound button: sets the mute flag and the master gain
    /// (0 when muted, otherwise 0.9).
    pub fn set_muted(&mut self, m: bool) {
        self.muted = m;
        self.apply_master_gain();
    }

    /// (a cheap handle clone, so callers can keep using `g`)
    pub fn soundscape(&self) -> Option<Rc<Soundscape>> {
        self.audio.soundscape.clone()
    }

    // ================================================================ sfx
    /// Footstep: a short bandpassed noise tick, louder when running
    pub fn sfx_step(&self, run: bool) {
        self.noise_burst(0.07, rand2(450.0, 850.0), 1.1, if run { 0.20 } else { 0.10 }, "bandpass", None, 0.0, 0.0);
    }
    /// Item pickup: falling sine blip over a soft lowpassed puff
    pub fn sfx_pickup(&self) {
        self.blip(300.0, 0.16, "sine", 0.18, Some(140.0), 0.0, 0.0);
        self.noise_burst(0.2, 500.0, 1.0, 0.08, "lowpass", None, 0.0, 0.0);
    }
    /// Door: two-note sine chime (C5, then G5)
    pub fn sfx_door(&self) {
        self.blip(523.0, 0.5, "sine", 0.16, None, 0.0, 0.0);
        self.blip(784.0, 0.8, "sine", 0.13, None, 0.14, 0.0);
    }
    /// Scream: falling noise sweep, sawtooth shriek and a low sine drop
    pub fn sfx_scream(&self) {
        self.noise_burst(1.0, 1400.0, 0.8, 0.7, "bandpass", Some(260.0), 0.0, 0.0);
        self.blip(620.0, 0.9, "sawtooth", 0.35, Some(80.0), 0.0, 0.0);
        self.blip(90.0, 1.2, "sine", 0.5, Some(35.0), 0.0, 0.0);
    }
    /// Low double thump at level `v`
    pub fn sfx_thump(&self, v: f64) {
        self.blip(58.0, 0.12, "sine", 0.55 * v, None, 0.0, 0.0);
        self.blip(52.0, 0.14, "sine", 0.4 * v, None, 0.13, 0.0);
    }
    /// Whisper: three overlapping bandpassed noise breaths
    pub fn sfx_whisper(&self) {
        self.noise_burst(0.35, 700.0, 2.5, 0.13, "bandpass", None, 0.0, 0.0);
        self.noise_burst(0.3, 1150.0, 2.5, 0.11, "bandpass", None, 0.3, 0.0);
        self.noise_burst(0.45, 850.0, 2.5, 0.12, "bandpass", None, 0.62, 0.0);
    }
    /// Short square tick at level `v`
    pub fn sfx_tick(&self, v: f64) {
        self.blip(2100.0, 0.03, "square", 0.05 * v, None, 0.0, 0.0);
    }
    /// Thud: low lowpassed noise hit
    pub fn sfx_thud(&self) {
        self.noise_burst(0.35, rand2(90.0, 140.0), 0.8, 0.25, "lowpass", None, 0.0, 0.0);
    }
    /// Clank: short narrow-band metallic noise
    pub fn sfx_clank(&self) {
        self.noise_burst(0.12, rand2(1800.0, 3200.0), 6.0, 0.10, "bandpass", None, 0.0, 0.0);
    }
    /// Moan: low falling sawtooth
    pub fn sfx_moan(&self) {
        self.blip(rand2(80.0, 110.0), 0.9, "sawtooth", 0.05, Some(60.0), 0.0, 0.0);
    }
    // ---- new ambient horror event sounds ----
    /// Door slam: crack, low sine boom and a rumbling tail
    pub fn sfx_door_slam(&self) {
        self.noise_burst(0.05, 3200.0, 4.0, 0.14, "bandpass", None, 0.0, 0.0);
        self.blip(60.0, 0.4, "sine", 0.35, Some(30.0), 0.0, 0.0);
        self.noise_burst(0.5, 180.0, 1.4, 0.14, "lowpass", Some(90.0), 0.02, 0.0);
    }
    /// Distant scream: long falling noise sweep with a faint sawtooth
    pub fn sfx_distant_scream(&self) {
        self.noise_burst(1.4, 900.0, 1.6, 0.14, "bandpass", Some(220.0), 0.0, 0.0);
        self.blip(340.0, 1.1, "sawtooth", 0.05, Some(90.0), 0.0, 0.0);
    }
    /// Radio static: hiss with two wandering tone fragments
    pub fn sfx_radio_static(&self) {
        self.noise_burst(0.6, 2400.0, 0.6, 0.08, "bandpass", None, 0.0, 0.0);
        self.blip(rand2(300.0, 700.0), 0.15, "square", 0.03, Some(rand2(200.0, 500.0)), 0.15, 0.0);
        self.blip(rand2(300.0, 700.0), 0.2, "sine", 0.02, Some(rand2(200.0, 500.0)), 0.35, 0.0);
    }
    /// Paper rustle: two highpassed noise flicks
    pub fn sfx_paper_rustle(&self) {
        self.noise_burst(0.25, 4200.0, 3.0, 0.06, "highpass", None, 0.0, 0.0);
        self.noise_burst(0.18, 5200.0, 4.0, 0.05, "highpass", None, 0.16, 0.0);
    }
    /// Drip: high plink with a lower ring
    pub fn sfx_drip(&self) {
        self.blip(rand2(900.0, 1400.0), 0.06, "sine", 0.08, None, 0.0, 0.0);
        self.blip(rand2(200.0, 400.0), 0.15, "sine", 0.05, None, 0.03, 0.0);
    }
    /// Dial tone: 350 + 440 Hz for half a second
    pub fn sfx_dial_tone(&self) {
        self.blip(350.0, 0.5, "sine", 0.06, None, 0.0, 0.0);
        self.blip(440.0, 0.5, "sine", 0.06, None, 0.0, 0.0);
    }
    /// The exit's call: air drawn under the door (a narrow whistle rising,
    /// then falling back) and the sign's relay ticking as it fails and
    /// catches. Above the hum's band (its 120 Hz would mask a ballast
    /// buzz), so it carries. Played from the door by `ambient`'s
    /// `amb_exit`; no draws.
    pub fn sfx_exit_call(&self) {
        self.noise_burst(2.2, 900.0, 3.0, 0.06, "bandpass", Some(1300.0), 0.0, 0.0);
        self.noise_burst(1.5, 1300.0, 3.5, 0.045, "bandpass", Some(820.0), 1.0, 0.0);
        for (i, t) in [0.30, 0.44, 0.63].into_iter().enumerate() {
            self.noise_burst(0.025, 2600.0, 5.0, 0.09 - i as f64 * 0.02, "bandpass", None, t, 0.0);
        }
    }
    /// The self-assembly's siren: a mechanical wail rising, held, falling
    /// (played from the Building's loudspeakers)
    pub fn sfx_siren(&self) {
        // the rotor spinning up, held at the top, running down: one voice
        // whose pitch rises, holds and falls under a held envelope
        self.wail("square", 0.03, 230.0, 610.0, 240.0);
        self.wail("sine", 0.075, 230.0, 610.0, 240.0);
    }
    /// One held, gliding voice: from `f0` up to `f1` (2.6 s), held (1.2 s),
    /// down to `f2` (3.4 s), swelling in and dying away
    fn wail(&self, ty: &str, gain: f64, f0: f64, f1: f64, f2: f64) {
        let (Some(ac), Some(master)) = (&self.ac, &self.master) else { return };
        let (gain, pan) = self.routed(gain, 0.0);
        let t0 = ac.current_time();
        let o = ac.create_oscillator();
        o.set_type(ty);
        o.frequency().set_value_at_time(f0, t0);
        o.frequency().exponential_ramp_to_value_at_time(f1, t0 + 2.6);
        o.frequency().set_value_at_time(f1, t0 + 3.8);
        o.frequency().exponential_ramp_to_value_at_time(f2, t0 + 7.2);
        let g = ac.create_gain();
        g.gain().set_value_at_time(0.0001, t0);
        g.gain().linear_ramp_to_value_at_time(gain, t0 + 0.9);
        g.gain().set_value_at_time(gain, t0 + 5.4);
        g.gain().exponential_ramp_to_value_at_time(0.0001, t0 + 7.4);
        o.connect(&g);
        self.one_shot_out(ac, master, &g, pan);
        o.start(t0);
        o.stop(t0 + 7.5);
    }
    /// The all-clear: one long steady tone
    pub fn sfx_all_clear(&self) {
        self.wail("triangle", 0.06, 466.0, 470.0, 462.0);
        self.wail("sine", 0.015, 932.0, 940.0, 925.0);
    }
    /// The fog: a wet, low rush, and something in it like breath
    pub fn sfx_fog(&self) {
        self.noise_burst(3.2, 240.0, 0.8, 0.10, "lowpass", Some(140.0), 0.0, 0.0);
        self.noise_burst(2.0, 900.0, 2.0, 0.025, "bandpass", Some(600.0), 0.6, 0.0);
    }
    /// A hermetic door's wheel turned: the screw's squeal; shut, the bolts home
    pub fn sfx_wheel(&self, shut: bool) {
        self.noise_burst(0.9, 2600.0, 6.0, 0.05, "bandpass", Some(1800.0), 0.0, 0.0);
        self.blip(380.0, 0.8, "sawtooth", 0.015, Some(300.0), 0.0, 0.0);
        if shut {
            self.noise_burst(0.08, 600.0, 1.2, 0.2, "lowpass", None, 0.95, 0.0);
            self.blip(70.0, 0.25, "sine", 0.2, Some(45.0), 0.95, 0.0);
        }
    }
    /// A knock at a sealed door (`world::knock`; play it at the door):
    /// three raps `gaps` seconds apart, knuckles on steel; from the other
    /// side (`behind`) duller and a little softer
    pub fn sfx_knock(&self, gaps: &[f64], behind: bool) {
        let (thud, ring, gain) = if behind { (260.0, 900.0, 0.26) } else { (380.0, 1400.0, 0.32) };
        let mut t = 0.0;
        for k in 0..=gaps.len() {
            self.noise_burst(0.12, thud, 1.4, gain, "lowpass", None, t, 0.0);
            self.blip(if behind { 82.0 } else { 95.0 }, 0.14, "sine", gain * 0.6, Some(58.0), t, 0.0);
            self.noise_burst(0.03, ring, 2.0, gain * 0.25, "bandpass", None, t, 0.0);
            t += gaps.get(k).copied().unwrap_or(0.0);
        }
    }
    /// Someone come to a sealed door from inside: two soft steps, and a bolt
    /// drawn (`world::knock`; play it at the door)
    pub fn sfx_behind_door(&self) {
        for k in 0..2 {
            self.noise_burst(0.1, 300.0, 0.9, 0.16, "lowpass", None, k as f64 * 0.45, 0.0);
        }
        self.noise_burst(0.05, 1600.0, 3.0, 0.14, "bandpass", None, 0.75, 0.0);
        self.blip(120.0, 0.08, "sine", 0.12, Some(80.0), 0.75, 0.0);
    }
    /// The hatch at a sealed door's foot opened or let fall shut; for a note,
    /// the paper slid under the door (`world::knock`; play it at the foot)
    pub fn sfx_hatch(&self, gift: crate::world::knock::Gift, open: bool) {
        if gift.is_note() {
            if open {
                self.noise_burst(0.45, 3800.0, 1.2, 0.09, "highpass", Some(5200.0), 0.0, 0.0);
            }
            return;
        }
        if open {
            self.noise_burst(0.2, 2200.0, 5.0, 0.06, "bandpass", Some(1600.0), 0.0, 0.0);
        }
        // the flap's steel on its frame
        self.noise_burst(0.06, 1500.0, 1.4, 0.3, "bandpass", None, 0.0, 0.0);
        self.blip(150.0, 0.09, "sine", 0.2, Some(90.0), 0.0, 0.0);
    }
    /// The thing pushed out coming to rest on the concrete: a bottle's glass,
    /// a battery's tin, a note's paper (`world::knock`; play it at the foot)
    pub fn sfx_pushed_out(&self, gift: crate::world::knock::Gift) {
        use crate::world::knock::Gift;
        match gift {
            Gift::Water => {
                // glass rolled on concrete, and its ring as it stops
                self.noise_burst(0.4, 1400.0, 1.2, 0.18, "bandpass", Some(900.0), 0.0, 0.0);
                self.blip(2600.0, 0.06, "sine", 0.06, None, 0.05, 0.0);
                self.blip(2450.0, 0.06, "sine", 0.045, None, 0.32, 0.0);
            }
            Gift::Battery => {
                self.noise_burst(0.25, 3200.0, 1.6, 0.12, "bandpass", None, 0.0, 0.0);
                self.blip(1900.0, 0.03, "square", 0.025, None, 0.22, 0.0);
            }
            _ => self.noise_burst(0.12, 4200.0, 1.0, 0.06, "highpass", None, 0.0, 0.0),
        }
    }
    /// Footsteps: four paces at a walk
    pub fn sfx_distant_footsteps(&self) {
        // somebody walking: a heel strike and the body's weight landing, four
        // paces at a walk (where they are makes them distant, not the sound)
        let pace = rand2(0.48, 0.58);
        for i in 0..4 {
            let t = i as f64 * pace;
            self.noise_burst(0.09, rand2(420.0, 620.0), 0.9, 0.16, "lowpass", None, t, 0.0);
            self.blip(72.0, 0.12, "sine", 0.12, Some(46.0), t, 0.0);
        }
    }
    /// Glitch — discordant reality-glitch chord
    pub fn sfx_glitch(&self) {
        self.blip(rand2(55.0, 70.0), 1.6, "sawtooth", 0.10, Some(40.0), 0.0, 0.0);
        self.blip(rand2(83.0, 91.0), 1.6, "sawtooth", 0.08, Some(60.0), 0.0, 0.0);
        self.blip(rand2(118.0, 132.0), 1.6, "sawtooth", 0.06, Some(80.0), 0.0, 0.0);
        self.noise_burst(1.6, 800.0, 0.7, 0.09, "bandpass", Some(220.0), 0.0, 0.0);
        self.blip(3000.0, 0.4, "square", 0.03, Some(200.0), 1.0, 0.0);
    }
    /// Ghost step: a single faint muffled footstep
    pub fn sfx_ghost_step(&self) {
        self.noise_burst(0.09, rand2(280.0, 420.0), 2.0, 0.055, "lowpass", None, 0.0, 0.0);
    }
    /// Breath: one lowpassed noise exhale
    pub fn sfx_breath(&self) {
        self.noise_burst(0.6, 220.0, 1.5, 0.08, "lowpass", None, 0.0, 0.0);
    }
    /// Laugh: five quick falling triangle blips
    pub fn sfx_laugh(&self) {
        for i in 0..5 {
            self.blip(rand2(320.0, 620.0), 0.09, "triangle", 0.03, Some(rand2(250.0, 520.0)), i as f64 * 0.11, 0.0);
        }
    }
    /// Chime — ethereal high chime for very rare event
    pub fn sfx_chime(&self) {
        self.blip(rand2(1200.0, 1800.0), 1.2, "sine", 0.08, None, 0.0, 0.0);
        self.blip(rand2(1800.0, 2400.0), 1.5, "sine", 0.05, None, 0.12, 0.0);
        self.blip(rand2(2400.0, 3200.0), 1.8, "sine", 0.03, None, 0.24, 0.0);
    }
    /// Cry — a male voice shouting "HEEEELP" via formant synthesis.
    ///  0.00–0.09 H (breathy fricative), 0.09–0.75 EEEE (sawtooth through two
    ///  peaking formants F1≈450, F2≈2200), 0.75–0.85 L (F2 → 1100),
    ///  0.85–0.95 P occlusion, 0.95–1.00 P burst.
    pub fn sfx_cry(&self) {
        let (Some(ac), Some(master), Some(noise_buf)) = (&self.ac, &self.master, &self.noise_buf) else { return };
        let t0 = ac.current_time();
        // ---- carrier: male fundamental with slight expressive pitch drop ----
        let osc = ac.create_oscillator();
        osc.set_type("sawtooth");
        osc.frequency().set_value_at_time(155.0, t0 + 0.09); // start of vowel
        osc.frequency().linear_ramp_to_value_at_time(140.0, t0 + 0.55);
        osc.frequency().linear_ramp_to_value_at_time(115.0, t0 + 0.85);
        // ---- formants: two peaking BPF-like filters in series ----
        let f1 = ac.create_biquad_filter();
        f1.set_type("peaking");
        f1.q().set_value(7.0);
        let f2 = ac.create_biquad_filter();
        f2.set_type("peaking");
        f2.q().set_value(8.0);
        f1.frequency().set_value_at_time(450.0, t0 + 0.09);
        f1.filter_gain().set_value(22.0);
        f2.frequency().set_value_at_time(2200.0, t0 + 0.09);
        f2.filter_gain().set_value(20.0);
        // "L" transition: F2 drops toward ~1100 Hz (lateral consonant)
        f2.frequency().linear_ramp_to_value_at_time(1100.0, t0 + 0.82);
        // high-shelf cut to keep it warm
        let hs = ac.create_biquad_filter();
        hs.set_type("highshelf");
        hs.frequency().set_value(3800.0);
        hs.filter_gain().set_value(-12.0);
        // ---- amplitude envelope: H (breath) -> EEEE sustain -> L -> P (silence) ----
        let amp = ac.create_gain();
        amp.gain().set_value_at_time(0.0, t0);
        amp.gain().linear_ramp_to_value_at_time(0.02, t0 + 0.05); // barely audible H onset
        amp.gain().linear_ramp_to_value_at_time(0.55, t0 + 0.20); // vowel attack
        amp.gain().set_value_at_time(0.55, t0 + 0.60);
        amp.gain().linear_ramp_to_value_at_time(0.35, t0 + 0.78); // L dip
        amp.gain().linear_ramp_to_value_at_time(0.00, t0 + 0.90); // P closure
                                                                  // ---- H fricative: bandpassed noise burst before the vowel ----
        let n_h = ac.create_buffer_source();
        n_h.set_buffer(noise_buf);
        let n_hf = ac.create_biquad_filter();
        n_hf.set_type("bandpass");
        n_hf.frequency().set_value(3200.0);
        n_hf.q().set_value(0.9);
        let n_hg = ac.create_gain();
        n_hg.gain().set_value_at_time(0.0, t0);
        n_hg.gain().linear_ramp_to_value_at_time(0.18, t0 + 0.04);
        n_hg.gain().linear_ramp_to_value_at_time(0.0, t0 + 0.10);
        n_h.connect(&n_hf);
        n_hf.connect(&n_hg);
        n_hg.connect(master);
        n_h.start(t0);
        n_h.stop(t0 + 0.15);
        // ---- P burst: sharp low-passed noise pop after the closure ----
        let n_p = ac.create_buffer_source();
        n_p.set_buffer(noise_buf);
        let n_pf = ac.create_biquad_filter();
        n_pf.set_type("lowpass");
        n_pf.frequency().set_value(1500.0);
        let n_pg = ac.create_gain();
        n_pg.gain().set_value_at_time(0.0, t0);
        n_pg.gain().set_value_at_time(0.0, t0 + 0.94);
        n_pg.gain().linear_ramp_to_value_at_time(0.20, t0 + 0.96);
        n_pg.gain().exponential_ramp_to_value_at_time(0.001, t0 + 1.05);
        n_p.connect(&n_pf);
        n_pf.connect(&n_pg);
        n_pg.connect(master);
        n_p.start(t0 + 0.93);
        n_p.stop(t0 + 1.10);
        // ---- wire the voiced part ----
        osc.connect(&f1);
        f1.connect(&f2);
        f2.connect(&hs);
        hs.connect(&amp);
        amp.connect(master);
        osc.start(t0);
        osc.stop(t0 + 1.05);
    }
    /// A panel's relay as its tube stutters: a dry click and the ballast's
    /// short buzz (`show_the_way`'s lights; play it where the panel hangs)
    pub fn sfx_relay(&self) {
        self.blip(1650.0, 0.018, "square", 0.045, None, 0.0, 0.0);
        self.blip(120.0, 0.16, "sawtooth", 0.035, Some(110.0), 0.01, 0.0);
    }
    /// A hiding place opened (`secrets`; play it where it is): a ceiling
    /// tile scraped up out of its grid and grit falling, a grille's screws
    /// giving and its tin, a rag pulled out from behind iron, plaster
    /// crumbling round a hand
    pub fn sfx_secret_open(&self, kind: crate::secrets::PlaceKind) {
        use crate::secrets::PlaceKind as K;
        match kind {
            K::Tile => {
                self.noise_burst(0.3, 1700.0, 1.6, 0.06, "bandpass", Some(1200.0), 0.0, 0.0);
                for k in 0..4 {
                    self.noise_burst(0.04, 3200.0, 2.0, 0.025, "bandpass", None, 0.25 + k as f64 * 0.07, 0.0);
                }
            }
            K::Vent => {
                self.blip(420.0, 0.16, "sawtooth", 0.03, Some(300.0), 0.0, 0.0);
                self.noise_burst(0.08, 2400.0, 3.0, 0.06, "bandpass", None, 0.18, 0.0);
            }
            K::Radiator => {
                self.noise_burst(0.35, 900.0, 0.8, 0.05, "lowpass", None, 0.0, 0.0);
            }
            K::Hole => {
                for k in 0..3 {
                    self.noise_burst(0.07, 1400.0, 1.2, 0.04, "lowpass", None, k as f64 * 0.11, 0.0);
                }
            }
            _ => {
                self.noise_burst(0.12, 1100.0, 1.0, 0.04, "lowpass", None, 0.0, 0.0);
            }
        }
    }
    /// A thing taken (`secrets`; play it where it was): eaten (a soft
    /// chew), drunk (a swallow), put on or pocketed (cloth, paper)
    pub fn sfx_take_thing(&self, t: crate::secrets::Thing) {
        use crate::secrets::Thing as T;
        match t {
            T::PaleMushrooms | T::BrownMushrooms | T::Seeds | T::Jelly | T::Ration | T::Sweets => {
                for k in 0..3 {
                    self.noise_burst(0.05, 700.0, 1.0, 0.04, "lowpass", None, k as f64 * 0.18, 0.0);
                }
            }
            T::Medicine | T::YellowWater => {
                self.blip(180.0, 0.12, "sine", 0.05, Some(120.0), 0.0, 0.0);
                self.blip(160.0, 0.12, "sine", 0.04, Some(110.0), 0.22, 0.0);
            }
            _ => self.noise_burst(0.18, 2600.0, 1.0, 0.035, "bandpass", None, 0.0, 0.0),
        }
    }
    /// A wet bare foot set down: a soft slap and the water's squelch
    /// (`show_the_way`'s prints; play it where the print is)
    pub fn sfx_wet_step(&self) {
        self.noise_burst(0.09, 520.0, 0.9, 0.09, "lowpass", None, 0.0, 0.0);
        self.noise_burst(0.06, 2600.0, 2.2, 0.03, "bandpass", Some(1800.0), 0.03, 0.0);
    }
    /// Splash — stepping into or through water (`loud` for running)
    pub fn sfx_splash(&self, loud: bool) {
        let gain = if loud { 0.24 } else { 0.10 };
        self.noise_burst(0.22, 5200.0, 1.4, gain, "bandpass", None, 0.0, 0.0);
        self.noise_burst(0.18, 900.0, 1.2, gain * 0.6, "lowpass", None, 0.0, 0.0);
        self.blip(rand2(600.0, 900.0), 0.08, "sine", gain * 0.25, None, 0.0, 0.0);
    }
    /// Piano — single hollow piano key from far away
    pub fn sfx_piano(&self) {
        let notes = [220.0, 247.0, 262.0, 294.0, 330.0, 392.0]; // A3 B3 C4 D4 E4 G4
        let f: f64 = choice(&notes);
        self.blip(f, 1.8, "sine", 0.07, None, 0.0, 0.0);
        self.blip(f * 2.0, 1.4, "sine", 0.04, None, 0.02, 0.0); // octave overtone
        self.blip(f * 3.0, 0.9, "triangle", 0.02, None, 0.05, 0.0);
    }
    /// Heavy drip — ceiling drip landing in a puddle
    pub fn sfx_drip_heavy(&self) {
        self.blip(rand2(1400.0, 2000.0), 0.03, "sine", 0.10, None, 0.0, 0.0);
        self.blip(rand2(300.0, 500.0), 0.20, "sine", 0.10, Some(rand2(140.0, 180.0)), 0.02, 0.0);
        self.noise_burst(0.25, 700.0, 0.9, 0.05, "lowpass", None, 0.02, 0.0);
    }
    /// Page flip — a stiff paper page turning nearby
    pub fn sfx_page_flip(&self) {
        self.noise_burst(0.14, 3200.0, 3.0, 0.06, "highpass", None, 0.0, 0.0);
        self.noise_burst(0.10, 5400.0, 3.0, 0.05, "highpass", None, 0.10, 0.0);
        self.noise_burst(0.05, 1800.0, 3.0, 0.09, "bandpass", None, 0.18, 0.0);
    }
    /// Scrape — slow metallic drag (`loudness` defaults to 0.10 in the sfx table)
    pub fn sfx_scrape(&self, loudness: f64) {
        self.noise_burst(0.9, 1600.0, 1.4, loudness * 0.9, "bandpass", Some(420.0), 0.0, 0.0);
        self.noise_burst(0.7, 260.0, 1.1, loudness * 0.6, "lowpass", Some(140.0), 0.05, 0.0);
        self.blip(rand2(80.0, 110.0), 0.7, "sawtooth", loudness * 0.35, Some(55.0), 0.1, 0.0);
    }

    // (the sfx table, by key)
    /// Play a sound by its sfx-table key (`"doorSlam"`; snake_case
    /// `"door_slam"` also works). Sounds that take an argument get the value
    /// a bare call means: `step` walks, `splash` is quiet, `scrape` uses its
    /// default 0.10, and `thump` / `tick` play at full level. Returns false
    /// for an unknown name.
    pub fn sfx_by_name(&mut self, name: &str) -> bool {
        match name {
            "step" => self.sfx_step(false),
            "pickup" => self.sfx_pickup(),
            "door" => self.sfx_door(),
            "scream" => self.sfx_scream(),
            "thump" => self.sfx_thump(1.0),
            "whisper" => self.sfx_whisper(),
            "tick" => self.sfx_tick(1.0),
            "thud" => self.sfx_thud(),
            "clank" => self.sfx_clank(),
            "moan" => self.sfx_moan(),
            "doorSlam" | "door_slam" => self.sfx_door_slam(),
            "distantScream" | "distant_scream" => self.sfx_distant_scream(),
            "radioStatic" | "radio_static" => self.sfx_radio_static(),
            "paperRustle" | "paper_rustle" => self.sfx_paper_rustle(),
            "drip" => self.sfx_drip(),
            "dialTone" | "dial_tone" => self.sfx_dial_tone(),
            "distantFootsteps" | "distant_footsteps" => self.sfx_distant_footsteps(),
            "glitch" => self.sfx_glitch(),
            "ghostStep" | "ghost_step" => self.sfx_ghost_step(),
            "breath" => self.sfx_breath(),
            "laugh" => self.sfx_laugh(),
            "chime" => self.sfx_chime(),
            "cry" => self.sfx_cry(),
            "relay" => self.sfx_relay(),
            "wetStep" | "wet_step" => self.sfx_wet_step(),
            "splash" => self.sfx_splash(false),
            "piano" => self.sfx_piano(),
            "dripHeavy" | "drip_heavy" => self.sfx_drip_heavy(),
            "pageFlip" | "page_flip" => self.sfx_page_flip(),
            "scrape" => self.sfx_scrape(0.10),
            "exitCall" | "exit_call" => self.sfx_exit_call(),
            _ => return false,
        }
        true
    }
}
