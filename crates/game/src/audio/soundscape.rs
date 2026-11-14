//! soundscape — the synthesized sound of the Backrooms (`audio::soundscape`).
//!
//! Everything is generated from oscillators, noise and filters at runtime (no
//! samples).
//!
//! * ballast hum — fluorescent lamps hum at twice mains frequency (120 Hz)
//!   with a full harmonic series plus a buzzy band of noise chopped at 120 Hz.
//!   One source, placed in 3D at the nearest live fixture, with slow drift and
//!   the occasional painful swell. Silent in blackouts.
//! * reverb — a convolution send whose impulse response is generated per zone
//!   type, crossfaded as you walk between zones.
//! * footsteps — per surface: wet carpet, concrete, tile, office carpet, stair.
//! * voices — formant-synthesized child sobbing and pleading for the
//!   Stairwell, the Face's static screech, heartbeat, knocking.
//! * spatial — HRTF panner slots for entity voices, with an occlusion lowpass
//!   the game drives from line of sight.
//!
//! `Soundscape::new(ac, out, opts)` builds it. `out` is the node everything
//! ends in (the game's master gain). Every method takes
//! `&self` (state is in `Cell`s), so a shared `Rc<Soundscape>` is enough.
//!
//! Default arguments are spelled out by the caller (as with `blip`): each
//! method's doc gives the usual values; `Option<&AudioNode>` destinations take
//! `None` for the method's default bus (`world` or `dry`).

use engine::prelude::*;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;

/// The soundscape's random source (`SoundscapeOpts::rng`, else `rnd()`).
pub type Rng = Rc<dyn Fn() -> f64>;

/// One reverb zone: T60 s, damping (0 bright → 1 dull), wet send level,
/// pre-delay s, early-reflection taps, optional flutter echo period s and
/// optional truncation factor. `flutter` / `truncate` = 0 means absent (both
/// are only ever tested for non-zero).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ReverbPreset {
    pub t60: f64,
    pub damp: f64,
    pub wet: f64,
    pub pre: f64,
    pub er: i32,
    pub flutter: f64,
    pub truncate: f64,
}
impl ReverbPreset {
    /// A preset without flutter or truncation (the `loop_reverb(lap)` shape).
    pub const fn new(t60: f64, damp: f64, wet: f64, pre: f64, er: i32) -> ReverbPreset {
        ReverbPreset { t60, damp, wet, pre, er, flutter: 0.0, truncate: 0.0 }
    }
}
const fn rp(t60: f64, damp: f64, wet: f64, pre: f64, er: i32, flutter: f64, truncate: f64) -> ReverbPreset {
    ReverbPreset { t60, damp, wet, pre, er, flutter, truncate }
}

/// Reverb zone presets by name, in declaration order.
pub const REVERB_PRESETS: [(&str, ReverbPreset); 14] = [
    //                    T60 s, damp, wet, pre-delay s, early taps, flutter, truncate
    ("lobby", rp(0.95, 0.72, 0.16, 0.008, 5, 0.0, 0.0)),
    ("pillars", rp(1.5, 0.62, 0.2, 0.012, 4, 0.0, 0.0)),
    ("office", rp(0.55, 0.8, 0.1, 0.004, 6, 0.0, 0.0)),
    ("store", rp(1.25, 0.45, 0.2, 0.012, 9, 0.0, 0.0)), // a big hard-floored shop hall
    ("maze", rp(0.8, 0.7, 0.14, 0.006, 6, 0.0, 0.0)),
    ("tunnels", rp(1.7, 0.35, 0.26, 0.004, 9, 0.011, 0.0)),
    ("electrical", rp(1.25, 0.45, 0.2, 0.005, 8, 0.009, 0.0)),
    ("hall", rp(3.1, 0.3, 0.3, 0.03, 5, 0.0, 0.0)),
    // the Building's: a long, sparse tail over the void (a well a hundred
    // metres across; a hall of stone with no far wall; a works floor)
    ("well", rp(4.4, 0.4, 0.4, 0.06, 6, 0.0, 0.0)),
    ("halls", rp(5.2, 0.28, 0.44, 0.08, 7, 0.0, 0.0)),
    ("works", rp(4.6, 0.45, 0.38, 0.07, 6, 0.0, 0.0)),
    ("pool", rp(2.7, 0.15, 0.34, 0.045, 3, 0.0, 0.6)),
    ("stairwell", rp(2.3, 0.4, 0.34, 0.012, 7, 0.017, 0.0)),
    ("silent", rp(0.25, 0.95, 0.02, 0.002, 1, 0.0, 0.0)),
];
/// The `REVERB_PRESETS` entry for `name`
pub fn reverb_preset(name: &str) -> Option<&'static ReverbPreset> {
    REVERB_PRESETS.iter().find(|(k, _)| *k == name).map(|(_, p)| p)
}

/// Surface → footstep recipe
pub const FOOTSTEP_SURFACES: [&str; 6] = ["carpet", "wetcarpet", "concrete", "tile", "office", "stair"];

/// `Soundscape::new` options.
#[derive(Clone, Default)]
pub struct SoundscapeOpts {
    /// Random source — `None` = `rnd()` as captured when the soundscape is
    /// made, which in the game is always the native generator (see `Soundscape::new`).
    pub rng: Option<Rng>,
    /// HRTF panning — only `Some(false)` switches the panners to equal-power.
    pub hrtf: Option<bool>,
}

/// child formants, Hz [F1, F2, F3] (`VOWELS`)
const VOWELS: [(&str, [f64; 3]); 9] = [
    ("a", [1000.0, 1650.0, 3400.0]),
    ("e", [620.0, 2500.0, 3500.0]),
    ("i", [380.0, 3000.0, 3700.0]),
    ("o", [620.0, 1100.0, 3300.0]),
    ("u", [430.0, 1150.0, 3200.0]),
    ("ae", [950.0, 2150.0, 3450.0]),
    ("uh", [720.0, 1500.0, 3300.0]),
    ("ee", [360.0, 2900.0, 3800.0]),
    ("er", [520.0, 1650.0, 2050.0]),
];
fn vowel(ph: &str) -> Option<[f64; 3]> {
    VOWELS.iter().find(|(k, _)| *k == ph).map(|(_, v)| *v)
}
const VOWEL_A: [f64; 3] = [1000.0, 1650.0, 3400.0];
const VOWEL_I: [f64; 3] = [380.0, 3000.0, 3700.0];

/// syllable scripts: (phoneme, duration s, f0 multiplier); '~' is a breath (`PHRASES`)
pub const PHRASES: [(&str, &[(&str, f64, f64)]); 5] = [
    (
        "sob",
        &[
            ("~", 0.28, 1.0),
            ("h", 0.06, 1.0),
            ("a", 0.55, 1.2),
            ("uh", 0.35, 0.92),
            ("~", 0.2, 1.0),
            ("h", 0.05, 1.0),
            ("a", 0.4, 1.12),
            ("o", 0.5, 0.85),
        ],
    ),
    (
        "please",
        &[
            ("p", 0.04, 1.0),
            ("l", 0.07, 1.0),
            ("ee", 0.5, 1.15),
            ("s", 0.16, 1.0),
            ("~", 0.25, 1.0),
            ("p", 0.04, 1.0),
            ("l", 0.06, 1.0),
            ("ee", 0.7, 1.05),
            ("s", 0.2, 1.0),
        ],
    ),
    (
        "help",
        &[
            ("h", 0.09, 1.0),
            ("e", 0.3, 1.18),
            ("l", 0.08, 1.0),
            ("p", 0.05, 1.0),
            ("~", 0.35, 1.0),
            ("h", 0.1, 1.0),
            ("e", 0.5, 1.06),
            ("uh", 0.25, 0.9),
            ("l", 0.07, 1.0),
            ("p", 0.06, 1.0),
        ],
    ),
    (
        "downhere",
        &[
            ("d", 0.04, 1.0),
            ("a", 0.22, 1.05),
            ("u", 0.2, 0.98),
            ("n", 0.08, 1.0),
            ("~", 0.08, 1.0),
            ("h", 0.08, 1.0),
            ("ee", 0.3, 1.1),
            ("er", 0.55, 0.9),
        ],
    ),
    (
        "wail",
        &[
            ("h", 0.08, 1.0),
            ("a", 1.3, 1.3),
            ("uh", 0.6, 0.8),
            ("~", 0.35, 1.0),
            ("h", 0.05, 1.0),
            ("i", 0.4, 1.25),
            ("~", 0.2, 1.0),
            ("h", 0.05, 1.0),
            ("i", 0.3, 1.2),
        ],
    ),
];
fn phrase(name: &str) -> &'static [(&'static str, f64, f64)] {
    PHRASES.iter().find(|(k, _)| *k == name).map_or(PHRASES[0].1, |(_, s)| *s)
}

// ================================================================ helpers
/// Uniform in [a, b) on the soundscape's rng
#[inline]
fn rand_r(rng: &Rng, a: f64, b: f64) -> f64 {
    a + rng() * (b - a)
}

fn noise_src(ac: &AudioContext, noise: &AudioBuffer, looping: bool) -> AudioNode {
    let s = ac.create_buffer_source();
    s.set_buffer(noise);
    s.set_loop(looping);
    s
}

/// `make_panner(ref_dist, rolloff, max_dist)`: HRTF (or equal-power) panner
/// with the inverse distance model
fn make_panner_impl(ac: &AudioContext, hrtf: bool, ref_dist: f64, rolloff: f64, max_dist: f64) -> AudioNode {
    let p = ac.create_panner();
    // panning model / distance model 'inverse' / ref distance / max distance / rolloff
    p.set_panner(if hrtf { "HRTF" } else { "equalpower" }, "inverse", ref_dist, max_dist, rolloff);
    p
}
/// `place_panner(p, x, y, z, tc)`: glide the panner's `position_x/y/z` params
/// to (x, y, z) with time constant `tc`
fn place_panner_impl(ac: &AudioContext, p: &AudioNode, x: f64, y: f64, z: f64, tc: f64) {
    let t = ac.current_time();
    p.position_x().set_target_at_time(x, t, tc);
    p.position_y().set_target_at_time(y, t, tc);
    p.position_z().set_target_at_time(z, t, tc);
}

/// `burst(dest, t0, dur, type, f0, q, gain, f1, attack)` — enveloped noise through a filter
fn burst_impl(
    ac: &AudioContext,
    rng: &Rng,
    noise: &AudioBuffer,
    dest: &AudioNode,
    t0: f64,
    dur: f64,
    ty: &str,
    f0: f64,
    q: f64,
    gain: f64,
    f1: Option<f64>,
    attack: f64,
) {
    let s = noise_src(ac, noise, false);
    let f = ac.create_biquad_filter();
    f.set_type(ty);
    f.frequency().set_value_at_time(f0, t0);
    f.q().set_value(q);
    if let Some(f1) = f1
        && f1 != 0.0
    {
        f.frequency().exponential_ramp_to_value_at_time(f1.max(20.0), t0 + dur);
    }
    let g = ac.create_gain();
    g.gain().set_value_at_time(0.0001, t0);
    g.gain().exponential_ramp_to_value_at_time(gain, t0 + attack);
    g.gain().exponential_ramp_to_value_at_time(0.0001, t0 + dur);
    s.connect(&f);
    f.connect(&g);
    g.connect(dest);
    s.start_at(t0, rng() * 1.5, None);
    s.stop(t0 + dur + 0.05);
}
/// `tone(dest, t0, dur, freq, gain, type, f1, attack)` — an enveloped
/// oscillator, optionally gliding to `f1`
fn tone_impl(ac: &AudioContext, dest: &AudioNode, t0: f64, dur: f64, freq: f64, gain: f64, ty: &str, f1: Option<f64>, attack: f64) {
    let o = ac.create_oscillator();
    o.set_type(ty);
    o.frequency().set_value_at_time(freq, t0);
    if let Some(f1) = f1
        && f1 != 0.0
    {
        o.frequency().exponential_ramp_to_value_at_time(f1.max(10.0), t0 + dur);
    }
    let g = ac.create_gain();
    g.gain().set_value_at_time(0.0001, t0);
    g.gain().exponential_ramp_to_value_at_time(gain, t0 + attack);
    g.gain().exponential_ramp_to_value_at_time(0.0001, t0 + dur);
    o.connect(&g);
    g.connect(dest);
    o.start(t0);
    o.stop(t0 + dur + 0.05);
}

/// `note(dest, t0, dur, freq, gain, type, f1, attack, release)` — an
/// oscillator held at `gain` from its attack to its release, optionally
/// gliding to `f1`: a sung or hummed note (`tone` falls away over its whole
/// length, a pluck: as a hum, it was heard only at each note's start)
fn note_impl(
    ac: &AudioContext,
    dest: &AudioNode,
    t0: f64,
    dur: f64,
    freq: f64,
    gain: f64,
    ty: &str,
    f1: Option<f64>,
    attack: f64,
    release: f64,
) {
    let o = ac.create_oscillator();
    o.set_type(ty);
    o.frequency().set_value_at_time(freq, t0);
    if let Some(f1) = f1.filter(|f| *f != 0.0) {
        o.frequency().exponential_ramp_to_value_at_time(f1.max(10.0), t0 + dur);
    }
    let (attack, release) = (attack.min(dur * 0.5), release.min(dur * 0.5));
    let g = ac.create_gain();
    g.gain().set_value_at_time(0.0001, t0);
    g.gain().exponential_ramp_to_value_at_time(gain, t0 + attack);
    g.gain().set_value_at_time(gain, t0 + dur - release);
    g.gain().exponential_ramp_to_value_at_time(0.0001, t0 + dur);
    o.connect(&g);
    g.connect(dest);
    o.start(t0);
    o.stop(t0 + dur + 0.05);
}

/// `make_ir(p)` as plain data: two channels of `len` samples, computed in f64
/// and stored as f32, drawing from `rng` (bit-reproducible for a given `rng`
/// sequence).
pub fn make_ir_data(p: &ReverbPreset, sr: f64, rng: &dyn Fn() -> f64) -> Vec<Vec<f32>> {
    let rand = |a: f64, b: f64| a + rng() * (b - a);
    let trunc = if p.truncate != 0.0 { p.truncate } else { 1.15 };
    let len = ((sr * (4f64).min(p.t60 * trunc)).floor() as i64).max(1) as usize;
    let mut out = Vec::with_capacity(2);
    for ch in 0..2 {
        let mut d = vec![0f32; len];
        let pre = (p.pre * sr).floor() as usize;
        let mut lp = 0.0f64;
        for i in pre..len {
            let t = (i - pre) as f64 / sr;
            let env = (-6.9 * t / p.t60).exp();
            // high frequencies die faster: a one-pole lowpass whose smoothing
            // grows with time (damp sets how fast the room gets dull)
            let a = (0.97f64).min(0.05 + p.damp * 0.85 * (1f64).min(t / (p.t60 * 0.5)));
            lp = lp * a + (rng() * 2.0 - 1.0) * (1.0 - a);
            d[i] = (lp * env * (1.0 + a * 2.2)) as f32;
        }
        // early reflections: a few discrete taps (walls close by)
        for k in 0..p.er {
            let ti = pre + (sr * rand(0.003, 0.045)).floor() as usize;
            if ti < len {
                let sign = if rng() < 0.5 { -1.0 } else { 1.0 };
                d[ti] = (d[ti] as f64 + sign * rand(0.35, 0.8) * (-(k as f64) * 0.35).exp()) as f32;
            }
        }
        // flutter echo between parallel walls (tunnels, the stairwell)
        if p.flutter != 0.0 {
            let step = (p.flutter * sr * (if ch == 1 { 1.03 } else { 1.0 })).floor() as usize;
            let (mut i, mut k) = (pre + step, 1i32);
            while i < len && k < 40 {
                d[i] = (d[i] as f64 + 0.5 * (-(k as f64) * 0.14).exp() * (if k % 2 == 1 { 1.0 } else { -1.0 })) as f32;
                i += step;
                k += 1;
            }
        }
        // truncation: the Poolrooms' reverb stops too early, as if cut
        if p.truncate != 0.0 {
            let cut = (len as f64 * 0.82).floor() as usize;
            for i in cut..len {
                d[i] = (d[i] as f64 * (0f64).max(1.0 - (i - cut) as f64 / (len as f64 * 0.02))) as f32;
            }
        }
        out.push(d);
    }
    out
}

// ================================================================ voice slot
/// Past this distance (m) a voice slot gets no quieter (the panner clamps it)
pub const VOICE_MAX_DIST: f64 = 60.0;

/// A spatial voice slot: source → occlusion lowpass → gain → panner → world
/// (`voice_slot(ref_dist)`'s return value).
#[derive(Clone)]
pub struct VoiceSlot {
    ac: AudioContext,
    /// Connect sources here (the occlusion lowpass)
    pub input: AudioNode,
    /// Level after the occlusion lowpass
    pub gain: AudioNode,
    /// The 3D panner (into `world`)
    pub panner: AudioNode,
}
impl VoiceSlot {
    fn new(ac: &AudioContext, hrtf: bool, world: &AudioNode, ref_dist: f64) -> VoiceSlot {
        let occ = ac.create_biquad_filter();
        occ.set_type("lowpass");
        occ.frequency().set_value(18000.0);
        occ.q().set_value(0.5);
        let g = ac.create_gain();
        g.gain().set_value(1.0);
        let pan = make_panner_impl(ac, hrtf, ref_dist, 1.2, VOICE_MAX_DIST);
        occ.connect(&g);
        g.connect(&pan);
        pan.connect(world);
        VoiceSlot { ac: ac.clone(), input: occ, gain: g, panner: pan }
    }
    /// Move the source to (x, y, z) (50 ms glide)
    pub fn at(&self, x: f64, y: f64, z: f64) {
        place_panner_impl(&self.ac, &self.panner, x, y, z, 0.05);
    }
    /// Put a fresh slot at (x, y, z) with occlusion `amount` at once, for a
    /// one-shot that starts now (`at` / `occlude` glide from where it was)
    pub fn place(&self, x: f64, y: f64, z: f64, amount: f64) {
        self.panner.position_x().set_value(x);
        self.panner.position_y().set_value(y);
        self.panner.position_z().set_value(z);
        self.input.frequency().set_value(18000.0 * (0.03f64).powf(amount));
        self.gain.gain().set_value(1.0 - amount * 0.45);
    }
    /// Occlusion `amount` — 0 clear line of sight … 1 behind walls
    pub fn occlude(&self, amount: f64) {
        let t = self.ac.current_time();
        self.input.frequency().set_target_at_time(18000.0 * (0.03f64).powf(amount), t, 0.08);
        self.gain.gain().set_target_at_time(1.0 - amount * 0.45, t, 0.08);
    }
}

// ================================================================ ballast hum
/// The ballast hum (`Soundscape::hum`).
pub struct Hum {
    ac: AudioContext,
    rng: Rng,
    /// The hum's spatial voice slot
    pub slot: VoiceSlot,
    sum: AudioNode,
    drift: AudioNode,
    /// every pitched source, so the whole hum can be detuned
    oscs: Vec<AudioNode>,
    level: Cell<f64>,
    spike_t: Cell<f64>,
    next_spike: Cell<f64>,
    drift_t: Cell<f64>,
    cents: Cell<f64>,
}
impl Hum {
    fn new(ac: &AudioContext, rng: &Rng, hrtf: bool, world: &AudioNode, noise: &AudioBuffer) -> Hum {
        let slot = VoiceSlot::new(ac, hrtf, world, 3.5);
        let sum = ac.create_gain();
        sum.gain().set_value(0.0);
        sum.connect(&slot.input);
        let drift = ac.create_gain();
        drift.gain().set_value(1.0);
        drift.connect(&sum);
        // harmonic series of 120 Hz, amplitude ~1/n^1.3, each partial a hair
        // detuned so a room full of fixtures beats slowly against itself
        let mut oscs = vec![];
        for n in 1..=9 {
            let nf = n as f64;
            let o = ac.create_oscillator();
            o.set_type("sine");
            o.frequency().set_value(120.0 * nf + rand_r(rng, -0.25, 0.25) * nf);
            let g = ac.create_gain();
            g.gain().set_value(0.55 / nf.powf(1.3) * (if n % 2 == 1 { 1.0 } else { 0.8 }));
            o.connect(&g);
            g.connect(&drift);
            o.start(0.0);
            oscs.push(o);
        }
        // −20 dB mains fundamental underneath
        {
            let o = ac.create_oscillator();
            o.frequency().set_value(60.0);
            let g = ac.create_gain();
            g.gain().set_value(0.055);
            o.connect(&g);
            g.connect(&drift);
            o.start(0.0);
            oscs.push(o);
        }
        // buzz: a band of noise chopped at 120 Hz (the arc re-striking twice a cycle)
        let bz = noise_src(ac, noise, true);
        let bp = ac.create_biquad_filter();
        bp.set_type("bandpass");
        bp.frequency().set_value(3300.0);
        bp.q().set_value(0.9);
        let chop = ac.create_gain();
        chop.gain().set_value(0.0);
        let lfo = ac.create_oscillator();
        lfo.set_type("square");
        lfo.frequency().set_value(120.0);
        let lfo_depth = ac.create_gain();
        lfo_depth.gain().set_value(0.045);
        lfo.connect(&lfo_depth);
        lfo_depth.connect_param(&chop.gain());
        bz.connect(&bp);
        bp.connect(&chop);
        chop.connect(&drift);
        bz.start(0.0);
        lfo.start(0.0);
        oscs.push(lfo);
        let next_spike = rand_r(rng, 40.0, 110.0);
        Hum {
            ac: ac.clone(),
            rng: rng.clone(),
            slot,
            sum,
            drift,
            oscs,
            level: Cell::new(0.0),
            spike_t: Cell::new(0.0),
            next_spike: Cell::new(next_spike),
            drift_t: Cell::new(0.0),
            cents: Cell::new(0.0),
        }
    }
    /// Per-frame update with the source at (x, y, z); `strength` 0..1 (how
    /// many live fixtures are near and how lit the room is)
    pub fn update(&self, dt: f64, x: f64, y: f64, z: f64, strength: f64) {
        self.slot.at(x, y, z);
        self.drift_t.set(self.drift_t.get() - dt);
        if self.drift_t.get() <= 0.0 {
            self.drift_t.set(rand_r(&self.rng, 1.5, 4.0));
            self.drift.gain().set_target_at_time(rand_r(&self.rng, 0.8, 1.15), self.ac.current_time(), 1.2);
        }
        self.next_spike.set(self.next_spike.get() - dt);
        if self.next_spike.get() <= 0.0 && strength > 0.3 {
            self.spike_t.set(rand_r(&self.rng, 1.2, 3.2));
            self.next_spike.set(rand_r(&self.rng, 60.0, 160.0));
        }
        // present, never so loud that it masks what happens elsewhere
        let mut target = strength * 0.1;
        if self.spike_t.get() > 0.0 {
            self.spike_t.set(self.spike_t.get() - dt);
            target *= 2.6;
        }
        let level = self.level.get() + (target - self.level.get()) * (1f64).min(dt * 3.0);
        self.level.set(level);
        self.sum.gain().set_target_at_time(level, self.ac.current_time(), 0.05);
    }
    /// Whether a painful swell is under way
    pub fn spiking(&self) -> bool {
        self.spike_t.get() > 0.0
    }
    /// Fade the hum out (blackouts)
    pub fn silence(&self) {
        self.level.set(0.0);
        self.sum.gain().set_target_at_time(0.0, self.ac.current_time(), 0.2);
    }
    /// Pitch the whole ballast (partials, sub and the 120 Hz chop) by `c`
    /// cents, with time constant `tc` (usually 0.6)
    pub fn detune(&self, c: f64, tc: f64) {
        if c == self.cents.get() {
            return;
        }
        self.cents.set(c);
        for o in &self.oscs {
            o.detune().set_target_at_time(c, self.ac.current_time(), tc);
        }
    }
    /// The current detune, in cents
    pub fn cents(&self) -> f64 {
        self.cents.get()
    }
}

// ================================================================ a machine's hum
/// A vending machine's hum (`Soundscape::machine_hum`), held from where it
/// stands while it is lit: its tube's magnetic ballast (120 Hz and its
/// series, brighter and smaller than a ceiling's), the buzz of the arc
/// re-striking twice a cycle, the transformer under it at 60 Hz, and the
/// air of its fan. Its level glides; at 0 it is silent.
pub struct MachineHum {
    ac: AudioContext,
    /// Where it is heard from
    pub slot: VoiceSlot,
    sum: AudioNode,
    oscs: Vec<AudioNode>,
    level: Cell<f64>,
}
impl MachineHum {
    fn new(ac: &AudioContext, slot: VoiceSlot, noise: &AudioBuffer) -> MachineHum {
        let sum = ac.create_gain();
        sum.gain().set_value(0.0);
        sum.connect(&slot.input);
        let mut oscs = vec![];
        // the ballast: 120 Hz, a thin series above it (each partial its own weight)
        for (n, w) in [(1.0, 0.55), (2.0, 0.42), (3.0, 0.30), (4.0, 0.18), (5.0, 0.10), (7.0, 0.05)] {
            let o = ac.create_oscillator();
            o.set_type("sine");
            o.frequency().set_value(120.0 * n + 0.07 * n);
            let g = ac.create_gain();
            g.gain().set_value(w);
            o.connect(&g);
            g.connect(&sum);
            o.start(0.0);
            oscs.push(o);
        }
        // the transformer's 60 Hz under it
        {
            let o = ac.create_oscillator();
            o.frequency().set_value(60.0);
            let g = ac.create_gain();
            g.gain().set_value(0.12);
            o.connect(&g);
            g.connect(&sum);
            o.start(0.0);
            oscs.push(o);
        }
        // the buzz: a band of noise chopped at 120 Hz
        let bz = noise_src(ac, noise, true);
        let bp = ac.create_biquad_filter();
        bp.set_type("bandpass");
        bp.frequency().set_value(2600.0);
        bp.q().set_value(1.1);
        let chop = ac.create_gain();
        chop.gain().set_value(0.0);
        let lfo = ac.create_oscillator();
        lfo.set_type("square");
        lfo.frequency().set_value(120.0);
        let depth = ac.create_gain();
        depth.gain().set_value(0.07);
        lfo.connect(&depth);
        depth.connect_param(&chop.gain());
        bz.connect(&bp);
        bp.connect(&chop);
        chop.connect(&sum);
        bz.start(0.0);
        lfo.start(0.0);
        oscs.push(lfo);
        oscs.push(bz);
        // the fan: soft air, low
        let air = noise_src(ac, noise, true);
        let lp = ac.create_biquad_filter();
        lp.set_type("lowpass");
        lp.frequency().set_value(700.0);
        let ag = ac.create_gain();
        ag.gain().set_value(0.10);
        air.connect(&lp);
        lp.connect(&ag);
        ag.connect(&sum);
        air.start(0.0);
        oscs.push(air);
        MachineHum { ac: ac.clone(), slot, sum, oscs, level: Cell::new(0.0) }
    }
    /// A sodium streetlamp's ballast heard through a window's glass
    /// (`core_props::window`): its 120 Hz and a little of its series, the
    /// arc's buzz thin and high, all of it dulled by the glass (lowpassed);
    /// no fan. Its level glides; at 0 it is silent.
    fn lamp(ac: &AudioContext, slot: VoiceSlot, noise: &AudioBuffer) -> MachineHum {
        let sum = ac.create_gain();
        sum.gain().set_value(0.0);
        let glass = ac.create_biquad_filter();
        glass.set_type("lowpass");
        glass.frequency().set_value(900.0);
        glass.q().set_value(0.6);
        sum.connect(&glass);
        glass.connect(&slot.input);
        let mut oscs = vec![];
        for (n, w) in [(1.0, 0.70), (2.0, 0.30), (3.0, 0.14), (4.0, 0.06)] {
            let o = ac.create_oscillator();
            o.set_type("sine");
            o.frequency().set_value(120.0 * n - 0.11 * n);
            let g = ac.create_gain();
            g.gain().set_value(w);
            o.connect(&g);
            g.connect(&sum);
            o.start(0.0);
            oscs.push(o);
        }
        let bz = noise_src(ac, noise, true);
        let bp = ac.create_biquad_filter();
        bp.set_type("bandpass");
        bp.frequency().set_value(1800.0);
        bp.q().set_value(1.4);
        let chop = ac.create_gain();
        chop.gain().set_value(0.0);
        let lfo = ac.create_oscillator();
        lfo.set_type("square");
        lfo.frequency().set_value(120.0);
        let depth = ac.create_gain();
        depth.gain().set_value(0.05);
        lfo.connect(&depth);
        depth.connect_param(&chop.gain());
        bz.connect(&bp);
        bp.connect(&chop);
        chop.connect(&sum);
        bz.start(0.0);
        lfo.start(0.0);
        oscs.push(lfo);
        oscs.push(bz);
        MachineHum { ac: ac.clone(), slot, sum, oscs, level: Cell::new(0.0) }
    }
    /// Per frame: from (x, y, z), at `level` (0 silent … 1 near and lit)
    pub fn update(&self, dt: f64, x: f64, y: f64, z: f64, level: f64) {
        self.slot.at(x, y, z);
        let l = self.level.get() + (level - self.level.get()) * (dt * 6.0).min(1.0);
        self.level.set(l);
        self.sum.gain().set_target_at_time(l * 0.04, self.ac.current_time(), 0.03);
    }
    /// The level it is gliding to (for the tests)
    pub fn level(&self) -> f64 {
        self.level.get()
    }
    /// Quiet at once and for good (the machine is gone with its level)
    pub fn stop(&self) {
        self.sum.gain().set_target_at_time(0.0, self.ac.current_time(), 0.05);
        let t = self.ac.current_time() + 0.5;
        for o in &self.oscs {
            o.stop(t);
        }
    }
}

// ================================================================ child voice
/// A source-filter voice (`create_child_voice(dest)`'s return value): a buzzy
/// glottal source (band-limited sawtooth at a child's pitch) through three
/// formant bandpasses whose centres move between vowel targets, plus breath
/// noise through the same formants for the h, s, p sounds and the catch
/// between sobs. Words are vowel paths.
pub struct ChildVoice {
    ac: AudioContext,
    rng: Rng,
    /// Output gain (the voice's level)
    pub out: AudioNode,
    src: AudioNode,
    vib: AudioNode,
    voiced: AudioNode,
    breath: AudioNode,
    breath_g: AudioNode,
    fricative: AudioNode,
    fric_bp: AudioNode,
    fric_g: AudioNode,
    formants: [AudioNode; 3],
    busy_until: Cell<f64>,
}
impl ChildVoice {
    fn new(ac: &AudioContext, rng: &Rng, noise: &AudioBuffer, dest: &AudioNode) -> ChildVoice {
        let out = ac.create_gain();
        out.gain().set_value(0.0);
        out.connect(dest);
        let src = ac.create_oscillator();
        src.set_type("sawtooth");
        src.frequency().set_value(380.0);
        let vib = ac.create_oscillator();
        vib.frequency().set_value(6.5);
        let vib_depth = ac.create_gain();
        vib_depth.gain().set_value(9.0);
        vib.connect(&vib_depth);
        vib_depth.connect_param(&src.frequency());
        let src_lp = ac.create_biquad_filter();
        src_lp.set_type("lowpass");
        src_lp.frequency().set_value(4200.0);
        let voiced = ac.create_gain();
        voiced.gain().set_value(0.0);
        src.connect(&src_lp);
        src_lp.connect(&voiced);
        let breath = noise_src(ac, noise, true);
        let breath_hp = ac.create_biquad_filter();
        breath_hp.set_type("highpass");
        breath_hp.frequency().set_value(500.0);
        let breath_g = ac.create_gain();
        breath_g.gain().set_value(0.0);
        breath.connect(&breath_hp);
        breath_hp.connect(&breath_g);
        let fricative = noise_src(ac, noise, true);
        let fric_bp = ac.create_biquad_filter();
        fric_bp.set_type("bandpass");
        fric_bp.frequency().set_value(6500.0);
        fric_bp.q().set_value(1.2);
        let fric_g = ac.create_gain();
        fric_g.gain().set_value(0.0);
        fricative.connect(&fric_bp);
        fric_bp.connect(&fric_g);
        fric_g.connect(&out);
        let mk = |k: usize| {
            let f = ac.create_biquad_filter();
            f.set_type("bandpass");
            f.frequency().set_value(VOWEL_A[k]);
            f.q().set_value([7.0, 11.0, 14.0][k]);
            let g = ac.create_gain();
            g.gain().set_value([1.0, 0.55, 0.22][k]);
            voiced.connect(&f);
            breath_g.connect(&f);
            f.connect(&g);
            g.connect(&out);
            f
        };
        let formants = [mk(0), mk(1), mk(2)];
        src.start(0.0);
        vib.start(0.0);
        breath.start(0.0);
        fricative.start(0.0);
        ChildVoice {
            ac: ac.clone(),
            rng: rng.clone(),
            out,
            src,
            vib,
            voiced,
            breath,
            breath_g,
            fricative,
            fric_bp,
            fric_g,
            formants,
            busy_until: Cell::new(0.0),
        }
    }
    /// Speak the `PHRASES` entry `name` starting at `t0` (`None` = now), with
    /// `pitch` and `loud` usually 1; returns the phrase's length in seconds.
    /// Unknown names say `sob`.
    pub fn say(&self, name: &str, t0: Option<f64>, pitch: f64, loud: f64) -> f64 {
        let t0 = t0.unwrap_or_else(|| self.ac.current_time());
        let script = phrase(name);
        let rng = &self.rng;
        let mut t = t0;
        let f0 = 390.0 * pitch;
        for &(ph, dur, fm) in script {
            let d = dur * rand_r(rng, 0.85, 1.2);
            if let Some(fv) = vowel(ph) {
                for (k, f) in self.formants.iter().enumerate() {
                    f.frequency().set_target_at_time(fv[k] * rand_r(rng, 0.97, 1.04), t, 0.03);
                }
                // pitch: rises into the syllable and sags, trembling
                self.src.frequency().set_target_at_time(f0 * fm * 1.06, t, 0.02);
                self.src.frequency().set_target_at_time(f0 * fm * 0.86, t + d * 0.4, d * 0.5);
                self.voiced.gain().set_target_at_time(0.22 * loud, t, 0.02);
                self.voiced.gain().set_target_at_time(0.0, t + d, 0.03);
                // the catch in a sob: a brief amplitude dip mid-vowel
                if d > 0.35 && rng() < 0.7 {
                    self.voiced.gain().set_target_at_time(0.05 * loud, t + d * 0.55, 0.01);
                    self.voiced.gain().set_target_at_time(0.2 * loud, t + d * 0.62, 0.02);
                }
                self.breath_g.gain().set_target_at_time(0.03 * loud, t, 0.03);
                self.breath_g.gain().set_target_at_time(0.0, t + d, 0.03);
            } else if ph == "~" {
                // the ragged in-breath between sobs
                for (k, f) in self.formants.iter().enumerate() {
                    f.frequency().set_target_at_time(VOWEL_I[k], t, 0.02);
                }
                self.breath_g.gain().set_target_at_time(0.12 * loud, t, 0.02);
                self.breath_g.gain().set_target_at_time(0.0, t + d, 0.04);
            } else if ph == "h" {
                self.breath_g.gain().set_target_at_time(0.1 * loud, t, 0.01);
                self.breath_g.gain().set_target_at_time(0.0, t + d, 0.02);
            } else if ph == "s" {
                self.fric_g.gain().set_target_at_time(0.05 * loud, t, 0.01);
                self.fric_g.gain().set_target_at_time(0.0, t + d, 0.02);
            } else if ph == "p" || ph == "d" {
                self.fric_bp.frequency().set_value_at_time(if ph == "p" { 1500.0 } else { 800.0 }, t);
                self.fric_g.gain().set_target_at_time(0.09 * loud, t, 0.003);
                self.fric_g.gain().set_target_at_time(0.0, t + d, 0.01);
                self.fric_bp.frequency().set_value_at_time(6500.0, t + d + 0.05);
            } else if ph == "l" || ph == "n" {
                let tgt = [400.0, if ph == "l" { 1200.0 } else { 1500.0 }, 2800.0];
                for (k, f) in self.formants.iter().enumerate() {
                    f.frequency().set_target_at_time(tgt[k], t, 0.015);
                }
                self.src.frequency().set_target_at_time(f0 * 0.95, t, 0.02);
                self.voiced.gain().set_target_at_time(0.1 * loud, t, 0.015);
                self.voiced.gain().set_target_at_time(0.0, t + d, 0.02);
            }
            t += d;
        }
        self.busy_until.set(t + 0.1);
        t - t0
    }
    /// Whether a phrase is still playing (plus 0.1 s)
    pub fn busy(&self) -> bool {
        self.ac.current_time() < self.busy_until.get()
    }
    /// Glide the output level to `v` (time constant `tc`, usually 0.4)
    pub fn set_level(&self, v: f64, tc: f64) {
        self.out.gain().set_target_at_time(v, self.ac.current_time(), tc);
    }
    /// Fade out, then stop the sources a second later
    pub fn stop(&self) {
        let now = self.ac.current_time();
        self.out.gain().set_target_at_time(0.0, now, 0.1);
        let now = self.ac.current_time();
        self.src.stop(now + 1.0);
        self.vib.stop(now + 1.0);
        self.breath.stop(now + 1.0);
        self.fricative.stop(now + 1.0);
    }
}

// ================================================================ soundscape
/// A send's gain automation as scheduled by `set_zone` (a linear ramp), so its
/// current value can be read mid-fade (the automated value, not the last
/// scheduled one).
#[derive(Clone, Copy)]
struct Ramp {
    v0: f64,
    t0: f64,
    v1: f64,
    t1: f64,
}
impl Ramp {
    fn value(&self, t: f64) -> f64 {
        if t >= self.t1 {
            self.v1
        } else if t <= self.t0 {
            self.v0
        } else {
            self.v0 + (self.v1 - self.v0) * (t - self.t0) / (self.t1 - self.t0)
        }
    }
}

/// The soundscape: buses, reverb zones, the ballast hum and the one-shot voices.
pub struct Soundscape {
    ac: AudioContext,
    rng: Rng,
    hrtf: bool,
    sr: f64,
    /// Shared white noise (2 s so loops don't read as a pattern)
    pub noise: AudioBuffer,
    /// Straight to `out`
    pub dry: AudioNode,
    /// World sounds go through here: dry + reverb send
    pub world: AudioNode,
    send_a: AudioNode,
    send_b: AudioNode,
    conv_a: AudioNode,
    conv_b: AudioNode,
    pub wet_out: AudioNode,
    /// true = send / convolver A is the active one
    active_a: Cell<bool>,
    current_preset: RefCell<Option<String>>,
    ir_cache: RefCell<HashMap<String, AudioBuffer>>,
    /// zones defined at runtime (the Loop gives its rooms one more step of
    /// reverb every lap); same fields as REVERB_PRESETS
    extra_zones: RefCell<HashMap<String, ReverbPreset>>,
    ramp_a: Cell<Ramp>,
    ramp_b: Cell<Ramp>,
    /// The ballast hum
    pub hum: Hum,
    shaper: AudioNode,
    pub screech_out: AudioNode,
}

impl Soundscape {
    /// Build the soundscape on `ac`, ending in `out`.
    ///
    /// The default rng is `native_rnd`: the game creates the soundscape once,
    /// in `audio_init` (never inside `with_seed`), so the function it keeps is
    /// the native `rnd()` even while a later level build swaps `rnd()` out.
    pub fn new(ac: &AudioContext, out: &AudioNode, opts: SoundscapeOpts) -> Soundscape {
        let rng: Rng = opts.rng.clone().unwrap_or_else(|| Rc::new(crate::base::rng::native_rnd));
        let hrtf = opts.hrtf != Some(false);
        let sr = ac.sample_rate();

        // shared white noise (2 s so loops don't read as a pattern)
        let noise = ac.create_buffer(1, (sr * 2.0) as usize, sr);
        {
            let mut d = noise.channel_mut(0);
            for v in d.iter_mut() {
                *v = (rng() * 2.0 - 1.0) as f32;
            }
        }

        // -------------------------------------------------------------- buses ---
        let dry = ac.create_gain();
        dry.gain().set_value(1.0);
        dry.connect(out);
        // world sounds go through `world`: dry + reverb send
        let world = ac.create_gain();
        world.gain().set_value(1.0);
        world.connect(&dry);
        let (send_a, send_b) = (ac.create_gain(), ac.create_gain());
        let (conv_a, conv_b) = (ac.create_convolver(), ac.create_convolver());
        conv_a.set_normalize(true);
        conv_b.set_normalize(true);
        world.connect(&send_a);
        world.connect(&send_b);
        send_a.connect(&conv_a);
        send_b.connect(&conv_b);
        let wet_out = ac.create_gain();
        wet_out.gain().set_value(1.0);
        wet_out.connect(out);
        conv_a.connect(&wet_out);
        conv_b.connect(&wet_out);
        send_a.gain().set_value(0.0);
        send_b.gain().set_value(0.0);

        // ------------------------------------------------------- ballast hum ---
        let hum = Hum::new(ac, &rng, hrtf, &world, &noise);

        // The Face: a scream made of radio static — noise ring-modulated by a
        // falling carrier and clipped, the sound the lore's camera feed made.
        let shaper = ac.create_wave_shaper();
        {
            let mut c = vec![0f32; 1024];
            for (i, v) in c.iter_mut().enumerate() {
                let x = i as f64 / 512.0 - 1.0;
                *v = (x * 6.0).tanh() as f32;
            }
            shaper.set_curve(c);
        }
        let screech_out = ac.create_gain();
        screech_out.gain().set_value(1.0);
        shaper.connect(&screech_out);
        screech_out.connect(&dry);

        let z = Ramp { v0: 0.0, t0: 0.0, v1: 0.0, t1: 0.0 };
        Soundscape {
            ac: ac.clone(),
            rng,
            hrtf,
            sr,
            noise,
            dry,
            world,
            send_a,
            send_b,
            conv_a,
            conv_b,
            wet_out,
            active_a: Cell::new(true),
            current_preset: RefCell::new(None),
            ir_cache: RefCell::new(HashMap::new()),
            extra_zones: RefCell::new(HashMap::new()),
            ramp_a: Cell::new(z),
            ramp_b: Cell::new(z),
            hum,
            shaper,
            screech_out,
        }
    }

    fn now(&self) -> f64 {
        self.ac.current_time()
    }
    /// the rng this soundscape draws from
    pub fn rng(&self) -> Rng {
        self.rng.clone()
    }

    // ------------------------------------------------------------ reverb ---
    /// A stereo impulse response for a zone
    pub fn make_ir(&self, p: &ReverbPreset) -> AudioBuffer {
        let data = make_ir_data(p, self.sr, &*self.rng);
        let buf = self.ac.create_buffer(2, data[0].len(), self.sr);
        for (ch, d) in data.into_iter().enumerate() {
            *buf.channel_mut(ch) = d;
        }
        buf
    }
    /// The cached impulse response of a zone (built by `set_zone`), for inspection.
    pub fn ir_for(&self, name: &str) -> Option<AudioBuffer> {
        self.ir_cache.borrow().get(name).cloned()
    }
    fn preset_for(&self, name: &str) -> Option<ReverbPreset> {
        reverb_preset(name).copied().or_else(|| self.extra_zones.borrow().get(name).copied())
    }
    pub fn define_zone(&self, name: &str, p: ReverbPreset) {
        self.extra_zones.borrow_mut().insert(name.to_string(), p);
        self.ir_cache.borrow_mut().remove(name);
    }
    /// Crossfade the reverb send to a zone's IR
    /// (unknown names fall back to `lobby`).
    pub fn set_zone(&self, name: &str, fade: f64) {
        let key = if self.preset_for(name).is_some() { name } else { "lobby" };
        if self.current_preset.borrow().as_deref() == Some(key) {
            return;
        }
        *self.current_preset.borrow_mut() = Some(key.to_string());
        let p = self.preset_for(key).unwrap();
        if !self.ir_cache.borrow().contains_key(key) {
            let ir = self.make_ir(&p);
            self.ir_cache.borrow_mut().insert(key.to_string(), ir);
        }
        let t = self.now();
        let a = self.active_a.get();
        let (in_conv, in_send, out_send) =
            if a { (&self.conv_b, &self.send_b, &self.send_a) } else { (&self.conv_a, &self.send_a, &self.send_b) };
        let (in_ramp, out_ramp) = if a { (&self.ramp_b, &self.ramp_a) } else { (&self.ramp_a, &self.ramp_b) };
        in_conv.set_buffer(&self.ir_cache.borrow()[key]);
        in_send.gain().cancel_scheduled_values(t);
        out_send.gain().cancel_scheduled_values(t);
        // the send's value mid-fade is the automated one, but the engine only
        // knows what was last scheduled, so track the ramp here
        let vin = in_ramp.get().value(t);
        in_send.gain().set_value_at_time(vin, t);
        in_send.gain().linear_ramp_to_value_at_time(p.wet, t + fade);
        in_ramp.set(Ramp { v0: vin, t0: t, v1: p.wet, t1: t + fade });
        let vout = out_ramp.get().value(t);
        out_send.gain().set_value_at_time(vout, t);
        out_send.gain().linear_ramp_to_value_at_time(0.0, t + fade);
        out_ramp.set(Ramp { v0: vout, t0: t, v1: 0.0, t1: t + fade });
        self.active_a.set(!a);
    }
    /// The current reverb zone
    pub fn zone(&self) -> Option<String> {
        self.current_preset.borrow().clone()
    }

    // ------------------------------------------------------------ spatial ---
    pub fn set_listener(&self, x: f64, y: f64, z: f64, fx: f64, fy: f64, fz: f64) {
        let l = self.ac.listener();
        let t = self.now();
        l.position_x().set_target_at_time(x, t, 0.02);
        l.position_y().set_target_at_time(y, t, 0.02);
        l.position_z().set_target_at_time(z, t, 0.02);
        l.forward_x().set_target_at_time(fx, t, 0.02);
        l.forward_y().set_target_at_time(fy, t, 0.02);
        l.forward_z().set_target_at_time(fz, t, 0.02);
        l.up_x().set_value(0.0);
        l.up_y().set_value(1.0);
        l.up_z().set_value(0.0);
    }
    pub fn make_panner(&self, ref_dist: f64, rolloff: f64, max_dist: f64) -> AudioNode {
        make_panner_impl(&self.ac, self.hrtf, ref_dist, rolloff, max_dist)
    }
    pub fn place_panner(&self, p: &AudioNode, x: f64, y: f64, z: f64, tc: f64) {
        place_panner_impl(&self.ac, p, x, y, z, tc);
    }
    /// Source → occlusion lowpass → gain → panner → world
    pub fn voice_slot(&self, ref_dist: f64) -> VoiceSlot {
        VoiceSlot::new(&self.ac, self.hrtf, &self.world, ref_dist)
    }
    /// A vending machine's hum, held where it stands (`MachineHum`)
    pub fn machine_hum(&self) -> MachineHum {
        MachineHum::new(&self.ac, self.voice_slot(1.2), &self.noise)
    }
    /// A streetlamp outside a window, heard through the glass (`MachineHum::lamp`)
    pub fn lamp_hum(&self) -> MachineHum {
        MachineHum::lamp(&self.ac, self.voice_slot(1.0), &self.noise)
    }

    // ------------------------------------------------------------ one-shots ---
    /// enveloped noise through a filter into a destination
    pub fn burst(&self, dest: &AudioNode, t0: f64, dur: f64, ty: &str, f0: f64, q: f64, gain: f64, f1: Option<f64>, attack: f64) {
        burst_impl(&self.ac, &self.rng, &self.noise, dest, t0, dur, ty, f0, q, gain, f1, attack);
    }
    pub fn tone(&self, dest: &AudioNode, t0: f64, dur: f64, freq: f64, gain: f64, ty: &str, f1: Option<f64>, attack: f64) {
        tone_impl(&self.ac, dest, t0, dur, freq, gain, ty, f1, attack);
    }
    /// A held note (`note_impl`): up over `attack`, held, down over `release`
    pub fn note(&self, dest: &AudioNode, t0: f64, dur: f64, freq: f64, gain: f64, ty: &str, f1: Option<f64>, attack: f64, release: f64) {
        note_impl(&self.ac, dest, t0, dur, freq, gain, ty, f1, attack, release);
    }
    fn rand(&self, a: f64, b: f64) -> f64 {
        rand_r(&self.rng, a, b)
    }

    /// Weight 0.3 crouch … 1 sprint
    pub fn footstep(&self, surface: &str, weight: f64, dest: Option<&AudioNode>) {
        let dest = dest.unwrap_or(&self.world);
        let (t, w) = (self.now(), weight);
        match surface {
            "wetcarpet" => {
                self.burst(dest, t, 0.09, "lowpass", 520.0, 0.7, 0.16 * w, None, 0.002);
                self.burst(dest, t + 0.02, 0.16, "bandpass", self.rand(900.0, 1300.0), 3.5, 0.05 * w, Some(500.0), 0.02); // squelch
                self.tone(dest, t, 0.08, 70.0, 0.07 * w, "sine", Some(45.0), 0.003);
            }
            "carpet" => {
                self.burst(dest, t, 0.08, "lowpass", 650.0, 0.7, 0.14 * w, None, 0.002);
                self.tone(dest, t, 0.07, 80.0, 0.05 * w, "sine", Some(50.0), 0.003);
            }
            "office" => {
                self.burst(dest, t, 0.06, "lowpass", 900.0, 0.6, 0.1 * w, None, 0.002);
                self.tone(dest, t, 0.05, 95.0, 0.04 * w, "sine", Some(60.0), 0.003);
            }
            "concrete" => {
                self.burst(dest, t, 0.035, "highpass", 1800.0, 0.7, 0.14 * w, None, 0.002); // heel click
                self.burst(dest, t + 0.012, 0.11, "bandpass", self.rand(2600.0, 4200.0), 1.2, 0.05 * w, None, 0.002); // grit
                self.tone(dest, t, 0.06, 110.0, 0.06 * w, "sine", Some(70.0), 0.003);
            }
            "tile" => {
                self.burst(dest, t, 0.03, "highpass", 2500.0, 0.8, 0.16 * w, None, 0.002);
                self.tone(dest, t, 0.09, self.rand(1700.0, 2100.0), 0.012 * w, "sine", None, 0.003); // glaze ring
                self.tone(dest, t, 0.05, 130.0, 0.05 * w, "sine", Some(80.0), 0.003);
            }
            "stair" => {
                self.burst(dest, t, 0.04, "highpass", 1500.0, 0.7, 0.16 * w, None, 0.002);
                self.burst(dest, t + 0.01, 0.2, "bandpass", 700.0, 2.5, 0.05 * w, Some(300.0), 0.002); // hollow landing boom
                self.tone(dest, t, 0.12, 62.0, 0.09 * w, "sine", Some(40.0), 0.003);
            }
            _ => {
                self.burst(dest, t, 0.07, "bandpass", self.rand(450.0, 850.0), 1.1, 0.12 * w, None, 0.002);
            }
        }
    }

    /// The Face: radio static
    /// ring-modulated by a falling carrier and clipped, plus a sub hit.
    pub fn screech(&self, intensity: f64, dur: f64) {
        let ac = &self.ac;
        let t = self.now();
        let s = noise_src(ac, &self.noise, false);
        let ring = ac.create_gain();
        ring.gain().set_value(0.0);
        let car = ac.create_oscillator();
        car.set_type("sawtooth");
        car.frequency().set_value_at_time(1800.0, t);
        car.frequency().exponential_ramp_to_value_at_time(140.0, t + dur);
        car.connect_param(&ring.gain());
        let bp = ac.create_biquad_filter();
        bp.set_type("bandpass");
        bp.frequency().set_value_at_time(2400.0, t);
        bp.q().set_value(0.6);
        bp.frequency().exponential_ramp_to_value_at_time(600.0, t + dur);
        let g = ac.create_gain();
        g.gain().set_value_at_time(0.0001, t);
        g.gain().exponential_ramp_to_value_at_time(0.5 * intensity, t + 0.015);
        g.gain().set_value_at_time(0.5 * intensity, t + dur * 0.6);
        g.gain().exponential_ramp_to_value_at_time(0.0001, t + dur);
        s.connect(&ring);
        ring.connect(&bp);
        bp.connect(&g);
        g.connect(&self.shaper);
        s.start(t);
        s.stop(t + dur + 0.05);
        car.start(t);
        car.stop(t + dur + 0.05);
        // sub hit under it
        self.tone(&self.dry, t, 0.5, 48.0, 0.4 * intensity, "sine", Some(30.0), 0.003);
    }
    /// Lub-dub, not
    /// quite human when `wrong` (extra beat, crackle)
    pub fn heartbeat(&self, bpm: f64, beats: i32, wrong: f64, dest: Option<&AudioNode>) {
        let dest = dest.unwrap_or(&self.dry);
        let (t0, iv) = (self.now(), 60.0 / bpm);
        for b in 0..beats {
            let t = t0 + b as f64 * iv * (1.0 + (if wrong != 0.0 { self.rand(-0.15, 0.2) * wrong } else { 0.0 }));
            self.tone(dest, t, 0.16, 55.0, 0.35, "sine", Some(38.0), 0.008);
            self.tone(dest, t + 0.17, 0.13, 50.0, 0.22, "sine", Some(36.0), 0.008);
            self.burst(dest, t, 0.05, "lowpass", 200.0, 1.0, 0.08, None, 0.002);
            if wrong != 0.0 && (self.rng)() < 0.5 * wrong {
                // cracking, like something settling inside the chest
                for k in 0..4 {
                    self.burst(dest, t + 0.3 + k as f64 * self.rand(0.02, 0.06), 0.02, "highpass", 3000.0, 1.0, 0.08 * wrong, None, 0.002);
                }
            }
        }
    }
    /// Knocking on a steel door somewhere, `count` raps, the k-th at k times
    /// 1–2 s (each drawn on its own, all scheduled at once: what knocks at a
    /// cell waits for the last of them before it goes,
    /// `world::cell_knock::KNOCK_RINGS`). A rap is `audio::rap::OUTSIDE`:
    /// the knuckles' thud and the leaf's body under it, and the knuckle's
    /// click on the steel and the leaf's ring above the lamps' ballast, so
    /// it is heard as a knock beside a lamp too, as the knock of
    /// `world::knock` is (`MILESTONES.md` 12, 13; `tests/listen.rs`,
    /// `what_knocks_is_heard_going`). The raps go through a gain of their
    /// own, returned, so those still to come can be hushed (a knock
    /// answered). Its draws: each rap's time, then its bursts' own
    pub fn knock(&self, count: i32, x: f64, y: f64, z: f64, gain: f64) -> AudioNode {
        let slot = self.voice_slot(2.0);
        slot.at(x, y, z);
        let hush = self.ac.create_gain();
        hush.connect(&slot.input);
        let t0 = self.now();
        for k in 0..count {
            let t = t0 + k as f64 * self.rand(1.0, 2.0);
            for p in &crate::audio::rap::OUTSIDE {
                match p.noise {
                    Some(ty) => self.burst(&hush, t, p.dur, ty, p.freq, p.q, gain * p.share, None, 0.002),
                    None => self.tone(&hush, t, p.dur, p.freq, gain * p.share, "sine", p.to, 0.002),
                }
            }
        }
        hush
    }
    pub fn create_child_voice(&self, dest: Option<&AudioNode>) -> ChildVoice {
        ChildVoice::new(&self.ac, &self.rng, &self.noise, dest.unwrap_or(&self.world))
    }
}
