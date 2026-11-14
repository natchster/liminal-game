//! An audio node graph rendered on CoreAudio (via cpal): oscillators, buffer and
//! constant sources, gains, biquad filters, wave shapers, convolvers, delays,
//! stereo and 3D panners, compressors and analysers.
//!
//! The main thread builds the graph through lightweight handles
//! (`ctx.create_gain()`, `node.connect(&other)`, `param.set_target_at_time(..)`),
//! and commands flow to the render thread, which processes 128-frame quanta with
//! scheduled parameter automation. A node is released once its last handle is
//! dropped and it can no longer make sound.
#![expect(clippy::new_without_default, reason = "a context opens the output device: a Default would hide that")]
#![expect(
    clippy::disallowed_methods,
    reason = "the sound's synthesis: its values reach the speakers and nothing the game decides (engine::dmath)"
)]

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

pub const QUANTUM: usize = 128;

// =============================================================== commands
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum OscType {
    Sine,
    Square,
    Sawtooth,
    Triangle,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FilterType {
    Lowpass,
    Highpass,
    Bandpass,
    Lowshelf,
    Highshelf,
    Peaking,
    Notch,
    Allpass,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum DistanceModel {
    Linear,
    Inverse,
    Exponential,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PanningModel {
    EqualPower,
    Hrtf,
}

#[derive(Clone, Debug)]
enum Kind {
    Destination,
    Gain,
    Oscillator,
    Biquad,
    BufferSource,
    StereoPanner,
    WaveShaper,
    Convolver,
    Panner,
    Delay(f64),
    Compressor,
    ConstantSource,
    Analyser,
}

#[derive(Clone, Copy, Debug)]
enum Ev {
    Set(f64, f64),
    Linear(f64, f64),
    Expo(f64, f64),
    Target(f64, f64, f64),
}
impl Ev {
    fn time(&self) -> f64 {
        match *self {
            Ev::Set(_, t) | Ev::Linear(_, t) | Ev::Expo(_, t) | Ev::Target(_, t, _) => t,
        }
    }
}

enum Cmd {
    Create(u32, Kind),
    Connect(u32, u32, Option<u8>),
    Disconnect(u32, Option<u32>),
    Release(u32),
    SetValue(u32, u8, f64),
    Event(u32, u8, Ev),
    Cancel(u32, u8, f64),
    Start(u32, f64, f64, Option<f64>),
    Stop(u32, f64),
    OscType(u32, OscType),
    FilterType(u32, FilterType),
    Buffer(u32, Option<Arc<BufferData>>),
    Loop(u32, bool, f64, f64),
    Curve(u32, Option<Arc<Vec<f32>>>),
    Normalize(u32, bool),
    PannerCfg(u32, PanningModel, DistanceModel, f64, f64, f64),
    Compressor(u32, [f64; 5]),
}

/// Listener params live on node 0 (the destination) as params 10..=18.
const LISTENER_NODE: u32 = 0;

// =============================================================== shared state
struct Shared {
    cmds: Mutex<Vec<Cmd>>,
    frames: AtomicU64,
    sample_rate: f64,
}

struct CtxInner {
    shared: Arc<Shared>,
    next_id: Cell<u32>,
    stream: RefCell<Option<cpal::Stream>>,
    muted: Cell<bool>,
    /// An offline context's engine, rendered on demand on this thread
    offline: RefCell<Option<Engine>>,
    /// On the silent clock: nothing renders the graph, so what is sent to
    /// it is dropped here, unqueued
    silent: Cell<bool>,
}
impl CtxInner {
    fn new(shared: Arc<Shared>, offline: Option<Engine>) -> CtxInner {
        CtxInner {
            shared,
            next_id: Cell::new(1),
            stream: RefCell::new(None),
            muted: Cell::new(false),
            offline: RefCell::new(offline),
            silent: Cell::new(false),
        }
    }
    fn send(&self, c: Cmd) {
        if !self.silent.get() {
            self.shared.cmds.lock().unwrap().push(c);
        }
    }
    /// Run on the silent clock (no device, or none may be opened): the same
    /// time, nothing rendered, nothing heard
    fn run_silent(&self) {
        self.silent.set(true);
        silent_clock(&self.shared);
    }
}

#[derive(Clone)]
pub struct AudioContext {
    inner: Rc<CtxInner>,
    dest: AudioNode,
    listener: AudioListener,
}

struct NodeInner {
    id: u32,
    ctx: Rc<CtxInner>,
    params: RefCell<HashMap<u8, f64>>,
}
impl Drop for NodeInner {
    fn drop(&mut self) {
        if self.id != LISTENER_NODE {
            self.ctx.send(Cmd::Release(self.id));
        }
    }
}

/// Any `AudioNode` (gain, oscillator, filter, ...). Clone = another handle to the same node.
#[derive(Clone)]
pub struct AudioNode(Rc<NodeInner>);

/// An `AudioParam` of a node.
#[derive(Clone)]
pub struct AudioParam {
    node: AudioNode,
    idx: u8,
    default: f64,
}

/// (planar f32 channels).
#[derive(Clone)]
pub struct AudioBuffer(Rc<RefCell<BufferData>>);
#[derive(Clone, Debug)]
pub struct BufferData {
    pub channels: Vec<Vec<f32>>,
    pub sample_rate: f64,
}

#[derive(Clone)]
pub struct AudioListener {
    node: AudioNode,
}

// param indices
const P_GAIN: u8 = 0;
const P_FREQ: u8 = 0;
const P_DETUNE: u8 = 1;
const P_Q: u8 = 2;
const P_FGAIN: u8 = 3;
const P_RATE: u8 = 0;
const P_PAN: u8 = 0;
const P_PX: u8 = 0;
const P_PY: u8 = 1;
const P_PZ: u8 = 2;
const P_OX: u8 = 3;
const P_OY: u8 = 4;
const P_OZ: u8 = 5;
const P_DELAY: u8 = 0;
const P_OFFSET: u8 = 0;
const L_PX: u8 = 10;
const L_PY: u8 = 11;
const L_PZ: u8 = 12;
const L_FX: u8 = 13;
const L_FY: u8 = 14;
const L_FZ: u8 = 15;
const L_UX: u8 = 16;
const L_UY: u8 = 17;
const L_UZ: u8 = 18;

/// Whether `AudioContext::new` may open the output device. Off until the
/// windowed game, a person playing, turns it on (`set_device_output`, in
/// `main` just before the window opens), so whatever else runs (a
/// headless mode, a tool, a test, a mode added later) is silent without
/// having to remember to be
static DEVICE_OUTPUT: AtomicBool = AtomicBool::new(false);

/// Let `AudioContext::new` open the output device, or not (then it runs on a
/// silent clock: the same time, nothing rendered, nothing heard). Only the
/// windowed game turns it on (`tests/structure.rs` holds that)
pub fn set_device_output(on: bool) {
    DEVICE_OUTPUT.store(on, Ordering::Relaxed);
}

/// The output device may be opened: turned on by the windowed game, not
/// asked for silence (`BACKROOMS_SILENT` set: `cargo xtask` sets it for
/// everything it runs), and not a test binary (cargo puts those in
/// `…/deps/<name>-<16 hex digits>`): a test, a bot's playthrough or a check
/// never plays through the speakers
pub fn device_output() -> bool {
    DEVICE_OUTPUT.load(Ordering::Relaxed) && std::env::var_os("BACKROOMS_SILENT").is_none() && !is_test_binary()
}

fn is_test_binary() -> bool {
    let Some(exe) = std::env::current_exe().ok().or_else(|| std::env::args_os().next().map(Into::into)) else { return false };
    let in_deps = exe.parent().and_then(|p| p.file_name()).is_some_and(|d| d == "deps");
    let hashed = exe
        .file_name()
        .and_then(|n| n.to_str())
        .and_then(|n| n.rsplit_once('-'))
        .is_some_and(|(_, h)| h.len() == 16 && h.bytes().all(|b| b.is_ascii_hexdigit()));
    in_deps && hashed
}

impl AudioContext {
    /// Open the default output device. Falls back to a silent clock if there
    /// is none, or if the device may not be opened (`device_output`).
    pub fn new() -> AudioContext {
        use cpal::traits::{DeviceTrait, HostTrait};
        if !device_output() {
            return AudioContext::new_silent();
        }
        let host = cpal::default_host();
        let dev = host.default_output_device();
        let cfg = dev.as_ref().and_then(|d| d.default_output_config().ok());
        AudioContext::on_output(dev.zip(cfg))
    }
    /// A context playing on `out` (a device and its configuration); on the
    /// silent clock when there is none or its stream cannot be opened
    fn on_output(out: Option<(cpal::Device, cpal::SupportedStreamConfig)>) -> AudioContext {
        use cpal::traits::{DeviceTrait, StreamTrait};
        let sr = out.as_ref().map_or(48000.0, |(_, c)| f64::from(c.sample_rate()));
        let shared = Arc::new(Shared { cmds: Mutex::new(vec![]), frames: AtomicU64::new(0), sample_rate: sr });
        let inner = Rc::new(CtxInner::new(shared.clone(), None));
        if let Some((dev, cfg)) = out {
            let ch = cfg.channels() as usize;
            let engine = Engine::new(sr, shared.clone());
            // at the device's own rate (on macOS, `default_output_config` is
            // its current stream format, so opening it changes no other app's)
            let mut sc = cfg.config();
            sc.buffer_size = cpal::BufferSize::Fixed(512);
            let build = |sc: cpal::StreamConfig, mut engine: Engine| {
                dev.build_output_stream(sc, move |data: &mut [f32], _| engine.fill(data, ch), |e| eprintln!("audio: {e}"), None)
            };
            // (a buffer size the device refuses: its own instead)
            let stream = build(sc, engine.clone_fresh()).or_else(|_| build(cfg.config(), engine.clone_fresh()));
            match stream {
                Ok(s) => {
                    let _ = s.play();
                    *inner.stream.borrow_mut() = Some(s);
                }
                Err(e) => {
                    eprintln!("audio: no output stream ({e}); running silent");
                    inner.run_silent();
                }
            }
        } else {
            // (with no clock its time stood at 0 and what was sent piled up)
            eprintln!("audio: no output device; running silent");
            inner.run_silent();
        }
        AudioContext::with_inner(inner)
    }
    /// A context that plays nothing and keeps real time: its clock moves on
    /// a thread of its own, as with no device, and its graph is never
    /// rendered, since nothing hears it (tests, headless runs): what is sent
    /// to it is dropped
    pub fn new_silent() -> AudioContext {
        let shared = Arc::new(Shared { cmds: Mutex::new(vec![]), frames: AtomicU64::new(0), sample_rate: 48000.0 });
        let inner = CtxInner::new(shared, None);
        inner.run_silent();
        AudioContext::with_inner(Rc::new(inner))
    }
    /// Whether this context sounds through an output device
    pub fn plays_on_device(&self) -> bool {
        self.inner.stream.borrow().is_some()
    }
    /// A context that plays nothing: its graph is rendered on this thread,
    /// on demand, by `render_offline` (recording a headless run, tests). Its
    /// clock only moves as it is rendered.
    pub fn new_offline(sample_rate: f64) -> AudioContext {
        let shared = Arc::new(Shared { cmds: Mutex::new(vec![]), frames: AtomicU64::new(0), sample_rate });
        let engine = Engine::new(sample_rate, shared.clone());
        AudioContext::with_inner(Rc::new(CtxInner::new(shared, Some(engine))))
    }
    fn with_inner(inner: Rc<CtxInner>) -> AudioContext {
        let dest = AudioNode(Rc::new(NodeInner { id: LISTENER_NODE, ctx: inner.clone(), params: RefCell::new(HashMap::new()) }));
        let listener = AudioListener { node: dest.clone() };
        AudioContext { inner, dest, listener }
    }
    /// The next `frames` sample frames of an offline context, as interleaved
    /// stereo (empty for a context playing on a device).
    pub fn render_offline(&self, frames: usize) -> Vec<f32> {
        let mut e = self.inner.offline.borrow_mut();
        let Some(e) = e.as_mut() else { return vec![] };
        let mut out = vec![0f32; frames * 2];
        e.fill(&mut out, 2);
        out
    }
    /// How many nodes an offline context's graph holds, as of what it has
    /// rendered (None for one playing on a device or the silent clock): a
    /// node is freed once its handles are dropped and it can sound no more
    /// (`Engine::collect`), so a count that only grows is a handle kept
    pub fn offline_nodes(&self) -> Option<usize> {
        self.inner.offline.borrow().as_ref().map(|e| e.nodes.len())
    }
    pub fn sample_rate(&self) -> f64 {
        self.inner.shared.sample_rate
    }
    /// Seconds of audio rendered so far (the scheduling clock).
    pub fn current_time(&self) -> f64 {
        self.inner.shared.frames.load(Ordering::Relaxed) as f64 / self.sample_rate()
    }
    pub fn destination(&self) -> AudioNode {
        self.dest.clone()
    }
    pub fn listener(&self) -> AudioListener {
        self.listener.clone()
    }
    fn make(&self, k: Kind) -> AudioNode {
        let id = self.inner.next_id.get();
        self.inner.next_id.set(id + 1);
        self.inner.send(Cmd::Create(id, k));
        AudioNode(Rc::new(NodeInner { id, ctx: self.inner.clone(), params: RefCell::new(HashMap::new()) }))
    }
    pub fn create_gain(&self) -> AudioNode {
        self.make(Kind::Gain)
    }
    pub fn create_oscillator(&self) -> AudioNode {
        self.make(Kind::Oscillator)
    }
    pub fn create_biquad_filter(&self) -> AudioNode {
        self.make(Kind::Biquad)
    }
    pub fn create_buffer_source(&self) -> AudioNode {
        self.make(Kind::BufferSource)
    }
    pub fn create_stereo_panner(&self) -> AudioNode {
        self.make(Kind::StereoPanner)
    }
    pub fn create_wave_shaper(&self) -> AudioNode {
        self.make(Kind::WaveShaper)
    }
    pub fn create_convolver(&self) -> AudioNode {
        self.make(Kind::Convolver)
    }
    pub fn create_panner(&self) -> AudioNode {
        self.make(Kind::Panner)
    }
    pub fn create_delay(&self, max: f64) -> AudioNode {
        self.make(Kind::Delay(max))
    }
    pub fn create_dynamics_compressor(&self) -> AudioNode {
        self.make(Kind::Compressor)
    }
    pub fn create_constant_source(&self) -> AudioNode {
        self.make(Kind::ConstantSource)
    }
    pub fn create_analyser(&self) -> AudioNode {
        self.make(Kind::Analyser)
    }
    pub fn create_buffer(&self, channels: usize, length: usize, sample_rate: f64) -> AudioBuffer {
        AudioBuffer(Rc::new(RefCell::new(BufferData { channels: vec![vec![0.0; length]; channels.max(1)], sample_rate })))
    }
    /// Master mute (the M key): silences the device output without touching the graph.
    pub fn set_muted(&self, m: bool) {
        self.inner.muted.set(m);
        self.inner.send(Cmd::SetValue(LISTENER_NODE, 99, if m { 0.0 } else { 1.0 }));
    }
    pub fn muted(&self) -> bool {
        self.inner.muted.get()
    }
    /// The native context always runs.
    pub fn resume(&self) {}
    pub fn state(&self) -> &'static str {
        "running"
    }
}

impl AudioBuffer {
    pub fn length(&self) -> usize {
        self.0.borrow().channels[0].len()
    }
    pub fn number_of_channels(&self) -> usize {
        self.0.borrow().channels.len()
    }
    pub fn sample_rate(&self) -> f64 {
        self.0.borrow().sample_rate
    }
    pub fn duration(&self) -> f64 {
        self.length() as f64 / self.sample_rate()
    }
    /// One channel's samples, for writing.
    pub fn channel_mut(&self, ch: usize) -> std::cell::RefMut<'_, Vec<f32>> {
        std::cell::RefMut::map(self.0.borrow_mut(), |b| &mut b.channels[ch])
    }
    pub fn channel(&self, ch: usize) -> std::cell::Ref<'_, Vec<f32>> {
        std::cell::Ref::map(self.0.borrow(), |b| &b.channels[ch])
    }
    fn snapshot(&self) -> Arc<BufferData> {
        Arc::new(self.0.borrow().clone())
    }
}

impl AudioNode {
    pub fn id(&self) -> u32 {
        self.0.id
    }
    fn send(&self, c: Cmd) {
        self.0.ctx.send(c);
    }
    /// Connect this node's output to `to`'s input.
    pub fn connect(&self, to: &AudioNode) -> &Self {
        self.send(Cmd::Connect(self.0.id, to.0.id, None));
        self
    }
    /// Connect this node's output to a parameter (audio-rate modulation).
    pub fn connect_param(&self, p: &AudioParam) -> &Self {
        self.send(Cmd::Connect(self.0.id, p.node.0.id, Some(p.idx)));
        self
    }
    /// Disconnect every output.
    pub fn disconnect(&self) {
        self.send(Cmd::Disconnect(self.0.id, None));
    }
    /// Disconnect the output to `to`.
    pub fn disconnect_from(&self, to: &AudioNode) {
        self.send(Cmd::Disconnect(self.0.id, Some(to.0.id)));
    }
    fn param(&self, idx: u8, default: f64) -> AudioParam {
        AudioParam { node: self.clone(), idx, default }
    }
    // ---- params
    pub fn gain(&self) -> AudioParam {
        self.param(P_GAIN, 1.0)
    }
    pub fn frequency(&self) -> AudioParam {
        self.param(P_FREQ, 440.0)
    }
    pub fn detune(&self) -> AudioParam {
        self.param(P_DETUNE, 0.0)
    }
    #[expect(non_snake_case, reason = "Web Audio's name for a filter's Q")]
    pub fn Q(&self) -> AudioParam {
        self.param(P_Q, 1.0)
    }
    pub fn q(&self) -> AudioParam {
        self.param(P_Q, 1.0)
    }
    /// Biquad filter gain (shelf / peaking, dB)
    pub fn filter_gain(&self) -> AudioParam {
        self.param(P_FGAIN, 0.0)
    }
    pub fn playback_rate(&self) -> AudioParam {
        self.param(P_RATE, 1.0)
    }
    /// Buffer source detune (cents)
    pub fn source_detune(&self) -> AudioParam {
        self.param(P_DETUNE, 0.0)
    }
    pub fn pan(&self) -> AudioParam {
        self.param(P_PAN, 0.0)
    }
    pub fn position_x(&self) -> AudioParam {
        self.param(P_PX, 0.0)
    }
    pub fn position_y(&self) -> AudioParam {
        self.param(P_PY, 0.0)
    }
    pub fn position_z(&self) -> AudioParam {
        self.param(P_PZ, 0.0)
    }
    pub fn orientation_x(&self) -> AudioParam {
        self.param(P_OX, 1.0)
    }
    pub fn orientation_y(&self) -> AudioParam {
        self.param(P_OY, 0.0)
    }
    pub fn orientation_z(&self) -> AudioParam {
        self.param(P_OZ, 0.0)
    }
    pub fn delay_time(&self) -> AudioParam {
        self.param(P_DELAY, 0.0)
    }
    pub fn offset(&self) -> AudioParam {
        self.param(P_OFFSET, 1.0)
    }
    /// Set a panner's position.
    pub fn set_position(&self, x: f64, y: f64, z: f64) {
        self.position_x().set_value(x);
        self.position_y().set_value(y);
        self.position_z().set_value(z);
    }
    // ---- settings
    /// Oscillator waveform (`"sine"`, ...) or filter type (`"lowpass"`, ...).
    pub fn set_type(&self, t: &str) {
        let osc = match t {
            "sine" => Some(OscType::Sine),
            "square" => Some(OscType::Square),
            "sawtooth" => Some(OscType::Sawtooth),
            "triangle" => Some(OscType::Triangle),
            _ => None,
        };
        if let Some(o) = osc {
            self.send(Cmd::OscType(self.0.id, o));
            return;
        }
        let f = match t {
            "lowpass" => FilterType::Lowpass,
            "highpass" => FilterType::Highpass,
            "bandpass" => FilterType::Bandpass,
            "lowshelf" => FilterType::Lowshelf,
            "highshelf" => FilterType::Highshelf,
            "peaking" => FilterType::Peaking,
            "notch" => FilterType::Notch,
            "allpass" => FilterType::Allpass,
            _ => return,
        };
        self.send(Cmd::FilterType(self.0.id, f));
    }
    /// The buffer a buffer source plays, or a convolver's impulse response.
    pub fn set_buffer(&self, b: &AudioBuffer) {
        self.send(Cmd::Buffer(self.0.id, Some(b.snapshot())));
    }
    pub fn clear_buffer(&self) {
        self.send(Cmd::Buffer(self.0.id, None));
    }
    /// Loop a buffer source over the whole buffer (`set_loop_range` takes start / end
    /// in seconds; 0, 0 = whole buffer).
    pub fn set_loop(&self, on: bool) {
        self.set_loop_range(on, 0.0, 0.0);
    }
    pub fn set_loop_range(&self, on: bool, start: f64, end: f64) {
        self.send(Cmd::Loop(self.0.id, on, start, end));
    }
    /// The wave shaper's transfer curve.
    pub fn set_curve(&self, c: Vec<f32>) {
        self.send(Cmd::Curve(self.0.id, Some(Arc::new(c))));
    }
    pub fn set_oversample(&self, _o: &str) {}
    /// Whether the convolver normalizes its impulse response.
    pub fn set_normalize(&self, n: bool) {
        self.send(Cmd::Normalize(self.0.id, n));
    }
    /// Panner settings (panning model, distance model, reference distance, max distance, rolloff factor).
    pub fn set_panner(&self, model: &str, distance: &str, ref_d: f64, max_d: f64, rolloff: f64) {
        let pm = if model == "HRTF" { PanningModel::Hrtf } else { PanningModel::EqualPower };
        let dm = match distance {
            "linear" => DistanceModel::Linear,
            "exponential" => DistanceModel::Exponential,
            _ => DistanceModel::Inverse,
        };
        self.send(Cmd::PannerCfg(self.0.id, pm, dm, ref_d, max_d, rolloff));
    }
    /// Compressor settings (threshold, knee, ratio, attack, release)
    pub fn set_compressor(&self, threshold: f64, knee: f64, ratio: f64, attack: f64, release: f64) {
        self.send(Cmd::Compressor(self.0.id, [threshold, knee, ratio, attack, release]));
    }
    // ---- scheduling
    pub fn start(&self, when: f64) {
        self.send(Cmd::Start(self.0.id, when, 0.0, None));
    }
    /// `start(when, offset, duration)`
    pub fn start_at(&self, when: f64, offset: f64, duration: Option<f64>) {
        self.send(Cmd::Start(self.0.id, when, offset, duration));
    }
    pub fn stop(&self, when: f64) {
        self.send(Cmd::Stop(self.0.id, when));
    }
}

impl AudioParam {
    fn send(&self, c: Cmd) {
        self.node.0.ctx.send(c);
    }
    fn now(&self) -> f64 {
        self.node.0.ctx.shared.frames.load(Ordering::Relaxed) as f64 / self.node.0.ctx.shared.sample_rate
    }
    fn remember(&self, v: f64) {
        self.node.0.params.borrow_mut().insert(self.idx, v);
    }
    /// The current value (the last value set or scheduled from this side)
    pub fn value(&self) -> f64 {
        self.node.0.params.borrow().get(&self.idx).copied().unwrap_or(self.default)
    }
    /// Set the value now (drops events scheduled after now).
    pub fn set_value(&self, v: f64) {
        if !v.is_finite() {
            return;
        }
        self.remember(v);
        self.send(Cmd::SetValue(self.node.0.id, self.idx, v));
    }
    pub fn set_value_at_time(&self, v: f64, t: f64) -> &Self {
        if v.is_finite() && t.is_finite() {
            if t <= self.now() + 0.001 {
                self.remember(v);
            }
            self.send(Cmd::Event(self.node.0.id, self.idx, Ev::Set(v, t.max(0.0))));
        }
        self
    }
    pub fn linear_ramp_to_value_at_time(&self, v: f64, t: f64) -> &Self {
        if v.is_finite() && t.is_finite() {
            self.remember(v);
            self.send(Cmd::Event(self.node.0.id, self.idx, Ev::Linear(v, t.max(0.0))));
        }
        self
    }
    pub fn exponential_ramp_to_value_at_time(&self, v: f64, t: f64) -> &Self {
        if v.is_finite() && t.is_finite() && v != 0.0 {
            self.remember(v);
            self.send(Cmd::Event(self.node.0.id, self.idx, Ev::Expo(v, t.max(0.0))));
        }
        self
    }
    pub fn set_target_at_time(&self, v: f64, t: f64, tc: f64) -> &Self {
        if v.is_finite() && t.is_finite() && tc.is_finite() {
            self.remember(v);
            self.send(Cmd::Event(self.node.0.id, self.idx, Ev::Target(v, t.max(0.0), tc.max(1e-6))));
        }
        self
    }
    pub fn cancel_scheduled_values(&self, t: f64) -> &Self {
        self.send(Cmd::Cancel(self.node.0.id, self.idx, t));
        self
    }
    pub fn cancel_and_hold_at_time(&self, t: f64) -> &Self {
        self.cancel_scheduled_values(t)
    }
}

impl AudioListener {
    fn p(&self, i: u8, d: f64) -> AudioParam {
        AudioParam { node: self.node.clone(), idx: i, default: d }
    }
    pub fn position_x(&self) -> AudioParam {
        self.p(L_PX, 0.0)
    }
    pub fn position_y(&self) -> AudioParam {
        self.p(L_PY, 0.0)
    }
    pub fn position_z(&self) -> AudioParam {
        self.p(L_PZ, 0.0)
    }
    pub fn forward_x(&self) -> AudioParam {
        self.p(L_FX, 0.0)
    }
    pub fn forward_y(&self) -> AudioParam {
        self.p(L_FY, 0.0)
    }
    pub fn forward_z(&self) -> AudioParam {
        self.p(L_FZ, -1.0)
    }
    pub fn up_x(&self) -> AudioParam {
        self.p(L_UX, 0.0)
    }
    pub fn up_y(&self) -> AudioParam {
        self.p(L_UY, 1.0)
    }
    pub fn up_z(&self) -> AudioParam {
        self.p(L_UZ, 0.0)
    }
    pub fn set_position(&self, x: f64, y: f64, z: f64) {
        self.position_x().set_value(x);
        self.position_y().set_value(y);
        self.position_z().set_value(z);
    }
    pub fn set_orientation(&self, fx: f64, fy: f64, fz: f64, ux: f64, uy: f64, uz: f64) {
        self.forward_x().set_value(fx);
        self.forward_y().set_value(fy);
        self.forward_z().set_value(fz);
        self.up_x().set_value(ux);
        self.up_y().set_value(uy);
        self.up_z().set_value(uz);
    }
}

// =============================================================== render side
#[derive(Clone)]
struct Param {
    value: f64,
    events: Vec<Ev>,
    last_t: f64,
    last_v: f64,
}
impl Param {
    fn new(v: f64) -> Param {
        Param { value: v, events: vec![], last_t: 0.0, last_v: v }
    }
    fn insert(&mut self, e: Ev) {
        let t = e.time();
        let pos = self.events.iter().position(|x| x.time() > t).unwrap_or(self.events.len());
        self.events.insert(pos, e);
    }
    /// Value at time t (seconds), advancing past finished events. Call with non-decreasing t.
    fn at(&mut self, t: f64) -> f64 {
        loop {
            let Some(&e) = self.events.first() else {
                return self.value;
            };
            match e {
                Ev::Set(v, et) => {
                    if t < et {
                        return self.value;
                    }
                    self.value = v;
                    self.last_t = et;
                    self.last_v = v;
                    self.events.remove(0);
                }
                Ev::Linear(v1, t1) | Ev::Expo(v1, t1) => {
                    let (t0, v0) = (self.last_t, self.last_v);
                    if t >= t1 {
                        self.value = v1;
                        self.last_t = t1;
                        self.last_v = v1;
                        self.events.remove(0);
                        continue;
                    }
                    if t < t0 {
                        return self.value;
                    }
                    let f = if t1 > t0 { (t - t0) / (t1 - t0) } else { 1.0 };
                    let v = match e {
                        Ev::Linear(..) => v0 + (v1 - v0) * f,
                        _ => {
                            if v0 * v1 > 0.0 {
                                v0 * (v1 / v0).powf(f)
                            } else {
                                v0
                            }
                        }
                    };
                    self.value = v;
                    return v;
                }
                Ev::Target(target, t0, tc) => {
                    if t < t0 {
                        return self.value;
                    }
                    // the approach starts from the value held at t0
                    if self.last_t < t0 {
                        self.last_t = t0;
                        self.last_v = self.value;
                    }
                    // a following event ends the approach
                    if let Some(next) = self.events.get(1) {
                        let nt = next.time();
                        if t >= nt {
                            self.value = target + (self.last_v - target) * (-(nt - self.last_t).max(0.0) / tc).exp();
                            self.last_t = nt;
                            self.last_v = self.value;
                            self.events.remove(0);
                            continue;
                        }
                    }
                    let v = target + (self.last_v - target) * (-(t - self.last_t) / tc).exp();
                    self.value = v;
                    // converged: drop the event
                    if (v - target).abs() < 1e-7 * (1.0 + target.abs()) && self.events.len() == 1 {
                        self.value = target;
                        self.last_v = target;
                        self.last_t = t;
                        self.events.remove(0);
                    }
                    return v;
                }
            }
        }
    }
    fn set_now(&mut self, v: f64, t: f64) {
        // `set_value`: like `set_value_at_time(v, now)`
        self.events.retain(|e| e.time() > t);
        self.value = v;
        self.last_v = v;
        self.last_t = t;
    }
    fn cancel(&mut self, t: f64) {
        self.events.retain(|e| e.time() < t);
    }
    fn idle(&self) -> bool {
        self.events.is_empty()
    }
}

#[derive(Clone)]
struct Buf {
    ch: usize,
    d: [[f32; QUANTUM]; 2],
}
impl Buf {
    fn silent() -> Buf {
        Buf { ch: 1, d: [[0.0; QUANTUM]; 2] }
    }
    fn clear(&mut self) {
        self.ch = 1;
        self.d = [[0.0; QUANTUM]; 2];
    }
    fn mix(&mut self, o: &Buf) {
        if o.ch == 2 && self.ch == 1 {
            self.d[1] = self.d[0];
            self.ch = 2;
        }
        if self.ch == 2 {
            if o.ch == 2 {
                for i in 0..QUANTUM {
                    self.d[0][i] += o.d[0][i];
                    self.d[1][i] += o.d[1][i];
                }
            } else {
                for i in 0..QUANTUM {
                    self.d[0][i] += o.d[0][i];
                    self.d[1][i] += o.d[0][i];
                }
            }
        } else {
            for i in 0..QUANTUM {
                self.d[0][i] += o.d[0][i];
            }
        }
    }
    fn mono(&self) -> [f32; QUANTUM] {
        if self.ch == 1 {
            self.d[0]
        } else {
            let mut m = [0.0; QUANTUM];
            for i in 0..QUANTUM {
                m[i] = 0.5 * (self.d[0][i] + self.d[1][i]);
            }
            m
        }
    }
}

struct Biquad {
    b0: f64,
    b1: f64,
    b2: f64,
    a1: f64,
    a2: f64,
    z: [[f64; 4]; 2],
}

#[derive(Default)]
struct Conv {
    ir: Vec<Vec<f32>>,   // per IR channel, normalized
    head: Vec<Vec<f32>>, // first HEAD taps (direct)
    hist: Vec<Vec<f32>>, // input history (per input channel), ring of HEAD
    hpos: usize,
    stages: Vec<Vec<Stage>>, // per output channel
    in_ch: usize,
}
const HEAD: usize = 128;
struct Stage {
    b: usize,
    parts: Vec<Vec<rustfft::num_complex::Complex<f32>>>,
    fdl: Vec<Vec<rustfft::num_complex::Complex<f32>>>,
    fdl_pos: usize,
    inbuf: Vec<f32>, // 2B: previous + current block
    fill: usize,
    out: Vec<f32>, // B samples of output for the current block
    out_pos: usize,
    delay_blocks: usize,
    fft: Arc<dyn rustfft::Fft<f32>>,
    ifft: Arc<dyn rustfft::Fft<f32>>,
}
impl Stage {
    fn new(h: &[f32], b: usize, offset: usize, len: usize, planner: &mut rustfft::FftPlanner<f32>) -> Stage {
        use rustfft::num_complex::Complex;
        let fft = planner.plan_fft_forward(2 * b);
        let ifft = planner.plan_fft_inverse(2 * b);
        let mut parts = vec![];
        let mut o = offset;
        while o < offset + len && o < h.len() {
            let mut v = vec![Complex::new(0.0, 0.0); 2 * b];
            for k in 0..b {
                if o + k < h.len() && o + k < offset + len {
                    v[k] = Complex::new(h[o + k], 0.0);
                }
            }
            fft.process(&mut v);
            parts.push(v);
            o += b;
        }
        let n = parts.len().max(1);
        let delay_blocks = offset / b;
        Stage {
            b,
            fdl: vec![vec![Complex::new(0.0, 0.0); 2 * b]; n + delay_blocks],
            fdl_pos: 0,
            parts,
            inbuf: vec![0.0; 2 * b],
            fill: 0,
            out: vec![0.0; b],
            out_pos: 0,
            delay_blocks,
            fft,
            ifft,
        }
    }
    /// push one input sample, return this stage's output sample
    #[inline]
    fn tick(&mut self, x: f32) -> f32 {
        let y = self.out[self.out_pos];
        self.out_pos += 1;
        self.inbuf[self.b + self.fill] = x;
        self.fill += 1;
        if self.fill == self.b {
            self.block();
        }
        y
    }
    fn block(&mut self) {
        use rustfft::num_complex::Complex;
        let b = self.b;
        let n = self.fdl.len();
        // spectrum of [previous, current]
        let slot = self.fdl_pos;
        for i in 0..2 * b {
            self.fdl[slot][i] = Complex::new(self.inbuf[i], 0.0);
        }
        self.fft.process(&mut self.fdl[slot]);
        // shift input
        self.inbuf.copy_within(b..2 * b, 0);
        self.fill = 0;
        // output for the next block: Σ_p H_p · X_{cur - (delay-1) - p}
        let mut acc = vec![Complex::new(0.0f32, 0.0); 2 * b];
        for (p, hp) in self.parts.iter().enumerate() {
            let back = (self.delay_blocks.saturating_sub(1)) + p;
            if back >= n {
                break;
            }
            let idx = (slot + n - back) % n;
            let x = &self.fdl[idx];
            for i in 0..2 * b {
                acc[i] += hp[i] * x[i];
            }
        }
        self.ifft.process(&mut acc);
        let scale = 1.0 / (2 * b) as f32;
        for i in 0..b {
            self.out[i] = acc[b + i].re * scale;
        }
        self.out_pos = 0;
        self.fdl_pos = (slot + 1) % n;
    }
}
impl Conv {
    fn build(&mut self, data: &BufferData, normalize: bool) {
        let mut ir: Vec<Vec<f32>> = data.channels.clone();
        if normalize {
            let (mut power, mut n) = (0.0f64, 0usize);
            for c in &ir {
                for &s in c {
                    power += (s as f64) * (s as f64);
                }
                n += c.len();
            }
            let mut p = (power / n.max(1) as f64).sqrt();
            if !p.is_finite() || p < 0.000125 {
                p = 0.000125;
            }
            let scale = (1.0 / p) * 0.00125 * (44100.0 / data.sample_rate.max(1.0));
            for c in ir.iter_mut() {
                for s in c.iter_mut() {
                    *s = (*s as f64 * scale) as f32;
                }
            }
        }
        let mut planner = rustfft::FftPlanner::<f32>::new();
        self.head = ir.iter().map(|c| c.iter().take(HEAD).copied().collect()).collect();
        self.stages = ir
            .iter()
            .map(|c| {
                let mut st = vec![];
                if c.len() > HEAD {
                    st.push(Stage::new(c, 128, 128, 2048 - 128, &mut planner));
                }
                if c.len() > 2048 {
                    st.push(Stage::new(c, 2048, 2048, c.len() - 2048, &mut planner));
                }
                st
            })
            .collect();
        self.ir = ir;
        self.hist = vec![vec![0.0; HEAD]; 2];
        self.hpos = 0;
    }
    fn process(&mut self, input: &Buf, out: &mut Buf) {
        if self.ir.is_empty() {
            out.clear();
            return;
        }
        let irc = self.ir.len().min(2);
        out.ch = 2;
        let mono_in = input.ch == 1 || irc == 1;
        let m = input.mono();
        for i in 0..QUANTUM {
            self.hpos = (self.hpos + 1) % HEAD;
            let (xl, xr) = if mono_in { (m[i], m[i]) } else { (input.d[0][i], input.d[1][i]) };
            self.hist[0][self.hpos] = xl;
            self.hist[1][self.hpos] = xr;
            for c in 0..2 {
                let ic = if irc == 1 { 0 } else { c };
                let h = &self.head[ic];
                let hist = &self.hist[if mono_in { 0 } else { c }];
                let mut y = 0.0f32;
                let mut p = self.hpos;
                for &hk in h.iter() {
                    y += hk * hist[p];
                    p = if p == 0 { HEAD - 1 } else { p - 1 };
                }
                let x = if mono_in {
                    xl
                } else if c == 0 {
                    xl
                } else {
                    xr
                };
                if irc == 1 && c == 1 {
                    out.d[1][i] = out.d[0][i];
                    continue;
                }
                for st in self.stages[ic].iter_mut() {
                    y += st.tick(x);
                }
                out.d[c][i] = y;
            }
        }
        if irc == 1 {
            out.d[1] = out.d[0];
        }
    }
}

struct NodeState {
    kind: Kind,
    params: Vec<Param>,
    inputs: Vec<u32>,
    param_inputs: Vec<(u32, u8)>,
    out: Buf,
    released: bool,
    // sources
    start: Option<f64>,
    stop: Option<f64>,
    ended: bool,
    phase: f64,
    osc: OscType,
    // buffer source
    buffer: Option<Arc<BufferData>>,
    looping: bool,
    loop_start: f64,
    loop_end: f64,
    pos: f64,
    offset: f64,
    duration: Option<f64>,
    // filter
    ftype: FilterType,
    bq: Biquad,
    // shaper
    curve: Option<Arc<Vec<f32>>>,
    // convolver
    conv: Option<Box<Conv>>,
    normalize: bool,
    // panner
    pm: PanningModel,
    dm: DistanceModel,
    ref_d: f64,
    max_d: f64,
    rolloff: f64,
    itd: [f32; 64],
    itd_pos: usize,
    shadow: [f64; 2],
    pan_smooth: [f64; 3],
    // delay
    dline: Vec<f32>,
    dpos: usize,
    // compressor
    comp: [f64; 5],
    env: f64,
}

impl NodeState {
    fn new(kind: Kind) -> NodeState {
        let params = match kind {
            Kind::Gain => vec![Param::new(1.0)],
            Kind::Oscillator => vec![Param::new(440.0), Param::new(0.0)],
            Kind::Biquad => vec![Param::new(350.0), Param::new(0.0), Param::new(1.0), Param::new(0.0)],
            Kind::BufferSource => vec![Param::new(1.0), Param::new(0.0)],
            Kind::StereoPanner => vec![Param::new(0.0)],
            Kind::Panner => vec![Param::new(0.0), Param::new(0.0), Param::new(0.0), Param::new(1.0), Param::new(0.0), Param::new(0.0)],
            Kind::Delay(_) => vec![Param::new(0.0)],
            Kind::ConstantSource => vec![Param::new(1.0)],
            Kind::Destination => {
                let mut v = vec![Param::new(0.0); 19];
                v[L_FZ as usize] = Param::new(-1.0);
                v[L_UY as usize] = Param::new(1.0);
                v.push(Param::new(1.0)); // 19: master (mute)
                v
            }
            _ => vec![],
        };
        let dline = if let Kind::Delay(m) = kind { vec![0.0; (m.max(0.01) * 48000.0 * 2.0) as usize + QUANTUM] } else { vec![] };
        NodeState {
            kind,
            params,
            inputs: vec![],
            param_inputs: vec![],
            out: Buf::silent(),
            released: false,
            start: None,
            stop: None,
            ended: false,
            phase: 0.0,
            osc: OscType::Sine,
            buffer: None,
            looping: false,
            loop_start: 0.0,
            loop_end: 0.0,
            pos: 0.0,
            offset: 0.0,
            duration: None,
            ftype: FilterType::Lowpass,
            bq: Biquad { b0: 1.0, b1: 0.0, b2: 0.0, a1: 0.0, a2: 0.0, z: [[0.0; 4]; 2] },
            curve: None,
            conv: None,
            normalize: true,
            pm: PanningModel::EqualPower,
            dm: DistanceModel::Inverse,
            ref_d: 1.0,
            max_d: 10000.0,
            rolloff: 1.0,
            itd: [0.0; 64],
            itd_pos: 0,
            shadow: [0.0; 2],
            pan_smooth: [0.0, 1.0, 0.0],
            dline,
            dpos: 0,
            comp: [-24.0, 30.0, 12.0, 0.003, 0.25],
            env: 0.0,
        }
    }
    fn is_source(&self) -> bool {
        matches!(self.kind, Kind::Oscillator | Kind::BufferSource | Kind::ConstantSource)
    }
}

/// The silent clock (`CtxInner::run_silent`): with no device, the clock
/// moves on a thread of its own so schedules still complete, as a device
/// would ask for frames (10 ms at a time, the clock at the end of the last
/// whole quantum begun, as `fill` leaves it). Nothing is heard and nothing
/// reads the graph back, so nothing renders it, and what the game sends is
/// dropped before it is queued (`CtxInner::send`). Rendering it, on the
/// creature pack's many one-shots, cost the headless bench's update and GPU
/// their tail (`docs/engineering/PERF.md`, "The update's tail"). The thread ends with
/// its context.
fn silent_clock(shared: &Arc<Shared>) {
    let (step, q) = ((shared.sample_rate * 0.01) as u64, QUANTUM as u64);
    let shared = Arc::downgrade(shared);
    std::thread::spawn(move || {
        let mut played = 0u64;
        loop {
            std::thread::sleep(std::time::Duration::from_millis(10));
            let Some(shared) = shared.upgrade() else { break };
            played += step;
            shared.frames.store(played.div_ceil(q) * q, Ordering::Relaxed);
        }
    });
}

struct Engine {
    sr: f64,
    shared: Arc<Shared>,
    nodes: HashMap<u32, NodeState>,
    order: Vec<u32>,
    order_dirty: bool,
    frame: u64,
    ring: Vec<f32>,
    ring_pos: usize,
    master: f32,
    scratch: Vec<Buf>,
}

impl Engine {
    fn new(sr: f64, shared: Arc<Shared>) -> Engine {
        let mut nodes = HashMap::new();
        nodes.insert(LISTENER_NODE, NodeState::new(Kind::Destination));
        Engine { sr, shared, nodes, order: vec![], order_dirty: true, frame: 0, ring: vec![], ring_pos: 0, master: 1.0, scratch: vec![] }
    }
    fn clone_fresh(&self) -> Engine {
        Engine::new(self.sr, self.shared.clone())
    }
    fn fill(&mut self, out: &mut [f32], ch: usize) {
        let frames = out.len() / ch.max(1);
        for f in 0..frames {
            if self.ring_pos >= self.ring.len() {
                self.render_quantum();
            }
            let l = self.ring[self.ring_pos] * self.master;
            let r = self.ring[self.ring_pos + 1] * self.master;
            self.ring_pos += 2;
            let base = f * ch;
            if ch >= 2 {
                out[base] = l.clamp(-1.0, 1.0);
                out[base + 1] = r.clamp(-1.0, 1.0);
                for k in 2..ch {
                    out[base + k] = 0.0;
                }
            } else if ch == 1 {
                out[base] = (0.5 * (l + r)).clamp(-1.0, 1.0);
            }
        }
    }
    fn apply_cmds(&mut self) {
        let cmds: Vec<Cmd> = match self.shared.cmds.try_lock() {
            Ok(mut g) => std::mem::take(&mut *g),
            Err(_) => return,
        };
        let now = self.frame as f64 / self.sr;
        for c in cmds {
            match c {
                Cmd::Create(id, k) => {
                    self.nodes.insert(id, NodeState::new(k));
                }
                Cmd::Connect(from, to, p) => {
                    if !self.nodes.contains_key(&from) {
                        continue;
                    }
                    if let Some(n) = self.nodes.get_mut(&to) {
                        match p {
                            None => {
                                if !n.inputs.contains(&from) {
                                    n.inputs.push(from);
                                }
                            }
                            Some(i) => {
                                if !n.param_inputs.contains(&(from, i)) {
                                    n.param_inputs.push((from, i));
                                }
                            }
                        }
                        self.order_dirty = true;
                    }
                }
                Cmd::Disconnect(from, to) => {
                    #[expect(clippy::iter_over_hash_type, reason = "each node only drops an edge: any order leaves the same graph")]
                    for (id, n) in self.nodes.iter_mut() {
                        if to.is_none() || to == Some(*id) {
                            n.inputs.retain(|&x| x != from);
                            n.param_inputs.retain(|&(x, _)| x != from);
                        }
                    }
                    self.order_dirty = true;
                }
                Cmd::Release(id) => {
                    if let Some(n) = self.nodes.get_mut(&id) {
                        n.released = true;
                    }
                }
                Cmd::SetValue(id, p, v) => {
                    if id == LISTENER_NODE && p == 99 {
                        self.master = v as f32;
                        continue;
                    }
                    if let Some(n) = self.nodes.get_mut(&id)
                        && let Some(pp) = n.params.get_mut(p as usize)
                    {
                        pp.set_now(v, now);
                    }
                }
                Cmd::Event(id, p, e) => {
                    if let Some(n) = self.nodes.get_mut(&id)
                        && let Some(pp) = n.params.get_mut(p as usize)
                    {
                        if pp.idle() {
                            pp.last_t = now;
                            pp.last_v = pp.value;
                        }
                        pp.insert(e);
                    }
                }
                Cmd::Cancel(id, p, t) => {
                    if let Some(n) = self.nodes.get_mut(&id)
                        && let Some(pp) = n.params.get_mut(p as usize)
                    {
                        pp.cancel(t);
                    }
                }
                Cmd::Start(id, t, off, dur) => {
                    if let Some(n) = self.nodes.get_mut(&id)
                        && n.start.is_none()
                    {
                        n.start = Some(t);
                        n.offset = off;
                        n.duration = dur;
                    }
                }
                Cmd::Stop(id, t) => {
                    if let Some(n) = self.nodes.get_mut(&id) {
                        n.stop = Some(t);
                    }
                }
                Cmd::OscType(id, t) => {
                    if let Some(n) = self.nodes.get_mut(&id) {
                        n.osc = t;
                    }
                }
                Cmd::FilterType(id, t) => {
                    if let Some(n) = self.nodes.get_mut(&id) {
                        n.ftype = t;
                    }
                }
                Cmd::Buffer(id, b) => {
                    if let Some(n) = self.nodes.get_mut(&id) {
                        if matches!(n.kind, Kind::Convolver) {
                            match &b {
                                Some(d) => {
                                    let mut c = Box::new(Conv::default());
                                    c.build(d, n.normalize);
                                    c.in_ch = 2;
                                    n.conv = Some(c);
                                }
                                None => n.conv = None,
                            }
                        }
                        n.buffer = b;
                    }
                }
                Cmd::Loop(id, on, s, e) => {
                    if let Some(n) = self.nodes.get_mut(&id) {
                        n.looping = on;
                        n.loop_start = s;
                        n.loop_end = e;
                    }
                }
                Cmd::Curve(id, c) => {
                    if let Some(n) = self.nodes.get_mut(&id) {
                        n.curve = c;
                    }
                }
                Cmd::Normalize(id, b) => {
                    if let Some(n) = self.nodes.get_mut(&id) {
                        n.normalize = b;
                    }
                }
                Cmd::PannerCfg(id, pm, dm, r, m, ro) => {
                    if let Some(n) = self.nodes.get_mut(&id) {
                        n.pm = pm;
                        n.dm = dm;
                        n.ref_d = r;
                        n.max_d = m;
                        n.rolloff = ro;
                    }
                }
                Cmd::Compressor(id, c) => {
                    if let Some(n) = self.nodes.get_mut(&id) {
                        n.comp = c;
                    }
                }
            }
        }
    }
    /// Drop released nodes that can no longer make sound.
    fn collect(&mut self) {
        loop {
            let mut dead = vec![];
            #[expect(clippy::iter_over_hash_type, reason = "only gathers the released nodes, which are then removed whatever their order")]
            for (&id, n) in &self.nodes {
                if id == LISTENER_NODE || !n.released {
                    continue;
                }
                let gone = if n.is_source() { n.ended || (n.start.is_none()) } else { n.inputs.is_empty() && n.param_inputs.is_empty() };
                // a released effect with no inputs that is still ringing (convolver tail) keeps going briefly
                if gone {
                    dead.push(id);
                }
            }
            if dead.is_empty() {
                break;
            }
            for id in &dead {
                self.nodes.remove(id);
            }
            #[expect(clippy::iter_over_hash_type, reason = "each node only forgets the removed ones: any order leaves the same graph")]
            for n in self.nodes.values_mut() {
                n.inputs.retain(|x| !dead.contains(x));
                n.param_inputs.retain(|(x, _)| !dead.contains(x));
            }
            self.order_dirty = true;
        }
    }
    fn compute_order(&mut self) {
        // post-order DFS from the destination: every node after its inputs
        let mut order = vec![];
        let mut state: HashMap<u32, u8> = HashMap::new();
        fn visit(id: u32, nodes: &HashMap<u32, NodeState>, st: &mut HashMap<u32, u8>, order: &mut Vec<u32>) {
            match st.get(&id) {
                Some(_) => return,
                None => {
                    st.insert(id, 1);
                }
            }
            if let Some(n) = nodes.get(&id) {
                for &i in &n.inputs {
                    visit(i, nodes, st, order);
                }
                for &(i, _) in &n.param_inputs {
                    visit(i, nodes, st, order);
                }
            }
            st.insert(id, 2);
            order.push(id);
        }
        visit(LISTENER_NODE, &self.nodes, &mut state, &mut order);
        self.order = order;
        self.order_dirty = false;
    }
    fn render_quantum(&mut self) {
        self.apply_cmds();
        if self.frame.is_multiple_of(QUANTUM as u64 * 8) {
            self.collect();
        }
        if self.order_dirty {
            self.compute_order();
        }
        let t0 = self.frame as f64 / self.sr;
        let sr = self.sr;
        let order = std::mem::take(&mut self.order);
        // listener (k-rate)
        let lis = {
            let d = self.nodes.get_mut(&LISTENER_NODE).unwrap();
            let mut v = [0.0; 9];
            for (k, item) in v.iter_mut().enumerate() {
                *item = d.params[L_PX as usize + k].at(t0);
            }
            v
        };
        for &id in &order {
            // gather inputs
            let mut input = Buf::silent();
            let mut param_mod: Vec<(u8, [f32; QUANTUM])> = vec![];
            {
                let n = &self.nodes[&id];
                for &i in &n.inputs {
                    if let Some(src) = self.nodes.get(&i) {
                        input.mix(&src.out);
                    }
                }
                for &(i, p) in &n.param_inputs {
                    if let Some(src) = self.nodes.get(&i) {
                        param_mod.push((p, src.out.mono()));
                    }
                }
            }
            let n = self.nodes.get_mut(&id).unwrap();
            process_node(n, &input, &param_mod, t0, sr, &lis);
        }
        self.order = order;
        let d = &self.nodes[&LISTENER_NODE].out;
        self.ring.clear();
        for i in 0..QUANTUM {
            let (l, r) = if d.ch == 2 { (d.d[0][i], d.d[1][i]) } else { (d.d[0][i], d.d[0][i]) };
            self.ring.push(l);
            self.ring.push(r);
        }
        self.ring_pos = 0;
        self.frame += QUANTUM as u64;
        self.shared.frames.store(self.frame, Ordering::Relaxed);
        let _ = &self.scratch;
    }
}

/// a-rate param values for this quantum, plus connected modulators
fn param_block(n: &mut NodeState, idx: u8, mods: &[(u8, [f32; QUANTUM])], t0: f64, sr: f64, out: &mut [f64; QUANTUM]) {
    let p = &mut n.params[idx as usize];
    if p.idle() {
        out.fill(p.value);
    } else {
        for (i, o) in out.iter_mut().enumerate() {
            *o = p.at(t0 + i as f64 / sr);
        }
    }
    for (pi, m) in mods {
        if *pi == idx {
            for i in 0..QUANTUM {
                out[i] += m[i] as f64;
            }
        }
    }
}
fn param_k(n: &mut NodeState, idx: u8, mods: &[(u8, [f32; QUANTUM])], t0: f64) -> f64 {
    let mut v = n.params[idx as usize].at(t0);
    for (pi, m) in mods {
        if *pi == idx {
            v += m[0] as f64;
        }
    }
    v
}

fn source_window(n: &mut NodeState, t0: f64, sr: f64) -> Option<(usize, usize)> {
    let start = n.start?;
    if n.ended {
        return None;
    }
    let s = ((start - t0) * sr).ceil().max(0.0) as usize;
    if s >= QUANTUM {
        return None;
    }
    let mut e = QUANTUM;
    if let Some(stop) = n.stop {
        let st = ((stop - t0) * sr).ceil();
        if st <= 0.0 {
            n.ended = true;
            return None;
        }
        if (st as usize) < QUANTUM {
            e = st as usize;
            n.ended = true;
        }
    }
    Some((s, e.max(s)))
}

fn poly_blep(t: f64, dt: f64) -> f64 {
    if t < dt {
        let t = t / dt;
        t + t - t * t - 1.0
    } else if t > 1.0 - dt {
        let t = (t - 1.0) / dt;
        t * t + t + t + 1.0
    } else {
        0.0
    }
}

fn process_node(n: &mut NodeState, input: &Buf, mods: &[(u8, [f32; QUANTUM])], t0: f64, sr: f64, lis: &[f64; 9]) {
    let mut out = Buf::silent();
    match n.kind.clone() {
        Kind::Destination => {
            out = input.clone();
        }
        Kind::Gain => {
            let mut g = [0.0; QUANTUM];
            param_block(n, P_GAIN, mods, t0, sr, &mut g);
            out.ch = input.ch;
            for c in 0..input.ch {
                for i in 0..QUANTUM {
                    out.d[c][i] = input.d[c][i] * g[i] as f32;
                }
            }
        }
        Kind::ConstantSource => {
            if let Some((s, e)) = source_window(n, t0, sr) {
                let mut g = [0.0; QUANTUM];
                param_block(n, P_OFFSET, mods, t0, sr, &mut g);
                for i in s..e {
                    out.d[0][i] = g[i] as f32;
                }
            }
        }
        Kind::Oscillator => {
            if let Some((s, e)) = source_window(n, t0, sr) {
                let mut f = [0.0; QUANTUM];
                let mut dt = [0.0; QUANTUM];
                param_block(n, P_FREQ, mods, t0, sr, &mut f);
                param_block(n, P_DETUNE, mods, t0, sr, &mut dt);
                let nyq = sr / 2.0;
                for i in s..e {
                    let freq = (f[i] * (2f64).powf(dt[i] / 1200.0)).clamp(-nyq, nyq);
                    let inc = freq / sr;
                    let ph = n.phase;
                    let v = match n.osc {
                        OscType::Sine => (ph * std::f64::consts::TAU).sin(),
                        OscType::Square => {
                            let mut v = if ph < 0.5 { 1.0 } else { -1.0 };
                            let d = inc.abs().max(1e-9);
                            v += poly_blep(ph, d);
                            v -= poly_blep((ph + 0.5) % 1.0, d);
                            v
                        }
                        OscType::Sawtooth => {
                            let d = inc.abs().max(1e-9);
                            let p2 = (ph + 0.5) % 1.0;
                            2.0 * p2 - 1.0 - poly_blep(p2, d)
                        }
                        OscType::Triangle => 1.0 - 4.0 * ((ph + 0.25) % 1.0 - 0.5).abs(),
                    };
                    out.d[0][i] = v as f32;
                    n.phase = (ph + inc).rem_euclid(1.0);
                }
            }
        }
        Kind::BufferSource => {
            if let (Some(buf), Some((s, e))) = (n.buffer.clone(), source_window(n, t0, sr)) {
                let rate = param_k(n, P_RATE, mods, t0) * (2f64).powf(param_k(n, P_DETUNE, mods, t0) / 1200.0);
                let len = buf.channels[0].len();
                let ratio = buf.sample_rate / sr;
                if n.pos == 0.0 && n.offset > 0.0 {
                    n.pos = n.offset * buf.sample_rate;
                    n.offset = 0.0;
                }
                let (ls, le) = if n.looping {
                    let ls = (n.loop_start * buf.sample_rate).max(0.0);
                    let le = if n.loop_end > n.loop_start { (n.loop_end * buf.sample_rate).min(len as f64) } else { len as f64 };
                    (ls, le)
                } else {
                    (0.0, len as f64)
                };
                out.ch = buf.channels.len().min(2);
                let start_t = n.start.unwrap_or(0.0);
                for i in s..e {
                    if let Some(d) = n.duration
                        && t0 + i as f64 / sr - start_t >= d
                    {
                        n.ended = true;
                        break;
                    }
                    if n.looping {
                        if n.pos >= le {
                            n.pos = ls + (n.pos - le) % (le - ls).max(1.0);
                        }
                    } else if n.pos >= len as f64 || n.pos < 0.0 {
                        n.ended = true;
                        break;
                    }
                    let i0 = n.pos.floor() as usize;
                    let fr = (n.pos - i0 as f64) as f32;
                    for c in 0..out.ch {
                        let ch = &buf.channels[c];
                        let a = ch[i0.min(len - 1)];
                        let b = if i0 + 1 < len {
                            ch[i0 + 1]
                        } else if n.looping {
                            ch[ls as usize % len]
                        } else {
                            0.0
                        };
                        out.d[c][i] = a + (b - a) * fr;
                    }
                    n.pos += rate * ratio;
                }
            } else if n.buffer.is_none() {
                let _ = source_window(n, t0, sr);
            }
        }
        Kind::Biquad => {
            let f = param_k(n, P_FREQ, mods, t0);
            let det = param_k(n, P_DETUNE, mods, t0);
            let q = param_k(n, P_Q, mods, t0);
            let gdb = param_k(n, P_FGAIN, mods, t0);
            set_biquad(&mut n.bq, n.ftype, f * (2f64).powf(det / 1200.0), q, gdb, sr);
            out.ch = input.ch;
            let bq = &mut n.bq;
            for c in 0..input.ch {
                let z = &mut bq.z[c];
                for i in 0..QUANTUM {
                    let x = input.d[c][i] as f64;
                    let y = bq.b0 * x + bq.b1 * z[0] + bq.b2 * z[1] - bq.a1 * z[2] - bq.a2 * z[3];
                    z[1] = z[0];
                    z[0] = x;
                    z[3] = z[2];
                    z[2] = if y.abs() < 1e-30 { 0.0 } else { y };
                    out.d[c][i] = y as f32;
                }
            }
        }
        Kind::StereoPanner => {
            let mut p = [0.0; QUANTUM];
            param_block(n, P_PAN, mods, t0, sr, &mut p);
            out.ch = 2;
            for i in 0..QUANTUM {
                let pan = p[i].clamp(-1.0, 1.0);
                if input.ch == 1 {
                    let x = (pan + 1.0) / 2.0;
                    let (gl, gr) = ((x * std::f64::consts::FRAC_PI_2).cos(), (x * std::f64::consts::FRAC_PI_2).sin());
                    out.d[0][i] = input.d[0][i] * gl as f32;
                    out.d[1][i] = input.d[0][i] * gr as f32;
                } else if pan <= 0.0 {
                    let x = pan + 1.0;
                    let (gl, gr) = ((x * std::f64::consts::FRAC_PI_2).cos(), (x * std::f64::consts::FRAC_PI_2).sin());
                    out.d[0][i] = input.d[0][i] + input.d[1][i] * gl as f32;
                    out.d[1][i] = input.d[1][i] * gr as f32;
                } else {
                    let x = pan;
                    let (gl, gr) = ((x * std::f64::consts::FRAC_PI_2).cos(), (x * std::f64::consts::FRAC_PI_2).sin());
                    out.d[0][i] = input.d[0][i] * gl as f32;
                    out.d[1][i] = input.d[1][i] + input.d[0][i] * gr as f32;
                }
            }
        }
        Kind::WaveShaper => {
            out.ch = input.ch;
            match &n.curve {
                Some(c) if c.len() > 1 => {
                    let nn = c.len();
                    for ch in 0..input.ch {
                        for i in 0..QUANTUM {
                            let x = input.d[ch][i];
                            let v = (nn - 1) as f32 * (x + 1.0) / 2.0;
                            out.d[ch][i] = if v <= 0.0 {
                                c[0]
                            } else if v >= (nn - 1) as f32 {
                                c[nn - 1]
                            } else {
                                let k = v.floor() as usize;
                                let f = v - k as f32;
                                c[k] * (1.0 - f) + c[k + 1] * f
                            };
                        }
                    }
                }
                _ => {
                    out = input.clone();
                }
            }
        }
        Kind::Convolver => {
            if let Some(c) = n.conv.as_mut() {
                c.process(input, &mut out);
            }
        }
        Kind::Delay(_) => {
            let d = (param_k(n, P_DELAY, mods, t0) * sr) as usize;
            let len = n.dline.len() / 2;
            out.ch = input.ch;
            for i in 0..QUANTUM {
                for c in 0..input.ch {
                    n.dline[c * len + n.dpos] = input.d[c][i];
                }
                let rp = (n.dpos + len - d.min(len - 1)) % len;
                for c in 0..input.ch {
                    out.d[c][i] = n.dline[c * len + rp];
                }
                n.dpos = (n.dpos + 1) % len;
            }
        }
        Kind::Compressor => {
            let [thr, knee, ratio, att, rel] = n.comp;
            out.ch = input.ch;
            let ka = (-1.0 / (att.max(1e-4) * sr)).exp();
            let kr = (-1.0 / (rel.max(1e-4) * sr)).exp();
            for i in 0..QUANTUM {
                let lvl = (0..input.ch).map(|c| input.d[c][i].abs()).fold(0.0f32, f32::max) as f64;
                let db = 20.0 * lvl.max(1e-9).log10();
                let over = db - thr;
                let gr = if over <= -knee / 2.0 {
                    0.0
                } else if over >= knee / 2.0 {
                    over * (1.0 - 1.0 / ratio)
                } else {
                    (over + knee / 2.0).powi(2) / (2.0 * knee) * (1.0 - 1.0 / ratio)
                };
                let k = if gr > n.env { ka } else { kr };
                n.env = gr + (n.env - gr) * k;
                let g = (10f64).powf(-n.env / 20.0) as f32;
                for c in 0..input.ch {
                    out.d[c][i] = input.d[c][i] * g;
                }
            }
        }
        Kind::Analyser => {
            out = input.clone();
        }
        Kind::Panner => {
            let px = param_k(n, P_PX, mods, t0);
            let py = param_k(n, P_PY, mods, t0);
            let pz = param_k(n, P_PZ, mods, t0);
            let (lx, ly, lz) = (lis[0], lis[1], lis[2]);
            let (fx, fy, fz) = (lis[3], lis[4], lis[5]);
            let (ux, uy, uz) = (lis[6], lis[7], lis[8]);
            let (dx, dy, dz) = (px - lx, py - ly, pz - lz);
            let dist = (dx * dx + dy * dy + dz * dz).sqrt();
            // distance gain
            let rd = n.ref_d.max(1e-6);
            let gain = match n.dm {
                DistanceModel::Inverse => rd / (rd + n.rolloff * (dist.max(rd) - rd)),
                DistanceModel::Linear => {
                    let d = dist.clamp(rd, n.max_d.max(rd));
                    1.0 - n.rolloff.min(1.0) * (d - rd) / (n.max_d - rd).max(1e-6)
                }
                DistanceModel::Exponential => (dist.max(rd) / rd).powf(-n.rolloff),
            };
            // azimuth (projection into listener space)
            let fl = (fx * fx + fy * fy + fz * fz).sqrt().max(1e-9);
            let (fx, fy, fz) = (fx / fl, fy / fl, fz / fl);
            let (mut rx, mut ry, mut rz) = (fy * uz - fz * uy, fz * ux - fx * uz, fx * uy - fy * ux);
            let rl = (rx * rx + ry * ry + rz * rz).sqrt().max(1e-9);
            rx /= rl;
            ry /= rl;
            rz /= rl;
            let (vux, vuy, vuz) = (ry * fz - rz * fy, rz * fx - rx * fz, rx * fy - ry * fx);
            let mut azimuth = 0.0;
            let mut front = 1.0;
            if dist > 1e-9 {
                let (sx, sy, sz) = (dx / dist, dy / dist, dz / dist);
                let up_proj = sx * vux + sy * vuy + sz * vuz;
                let (prx, pry, prz) = (sx - up_proj * vux, sy - up_proj * vuy, sz - up_proj * vuz);
                let pl = (prx * prx + pry * pry + prz * prz).sqrt().max(1e-9);
                let (prx, pry, prz) = (prx / pl, pry / pl, prz / pl);
                azimuth = (prx * rx + pry * ry + prz * rz).clamp(-1.0, 1.0).asin().to_degrees();
                front = prx * fx + pry * fy + prz * fz;
                if front < 0.0 {
                    azimuth = 180.0 - azimuth;
                }
                if azimuth > 180.0 {
                    azimuth -= 360.0;
                }
            }
            // equal-power pan position from azimuth (front / back folded together)
            let mut az = azimuth;
            if az < -90.0 {
                az = -180.0 - az;
            } else if az > 90.0 {
                az = 180.0 - az;
            }
            let x = (az + 90.0) / 180.0;
            let (mut gl, mut gr) = ((x * std::f64::consts::FRAC_PI_2).cos(), (x * std::f64::consts::FRAC_PI_2).sin());
            // ease the gains to avoid zipper noise
            let s = &mut n.pan_smooth;
            s[0] += (gl - s[0]) * 0.35;
            s[1] += (gr - s[1]) * 0.35;
            s[2] += (gain - s[2]) * 0.35;
            gl = s[0];
            gr = s[1];
            let gain = s[2];
            let m = input.mono();
            out.ch = 2;
            if n.pm == PanningModel::Hrtf {
                // interaural delay + head shadow on the far ear, and a darker sound from behind
                let side = (azimuth.to_radians()).sin();
                let delay = (side.abs() * 0.00066 * sr).min(63.0) as usize;
                let shade = 1.0 - 0.75 * side.abs();
                let back = if front < 0.0 { 0.55 } else { 1.0 };
                let cut = |c: f64| 1.0 - (-std::f64::consts::TAU * c / sr).exp();
                let a_far = cut(1200.0 + 16000.0 * shade * back);
                let a_near = cut(4000.0 + 16000.0 * back);
                for i in 0..QUANTUM {
                    let xin = m[i] * gain as f32;
                    n.itd[n.itd_pos] = xin;
                    let delayed = n.itd[(n.itd_pos + 64 - delay) % 64];
                    n.itd_pos = (n.itd_pos + 1) % 64;
                    let (near, far) = (xin as f64, delayed as f64);
                    let (l_in, r_in, al, ar) = if side >= 0.0 { (far, near, a_far, a_near) } else { (near, far, a_near, a_far) };
                    n.shadow[0] += (l_in - n.shadow[0]) * al;
                    n.shadow[1] += (r_in - n.shadow[1]) * ar;
                    // HRTF keeps more level on the far side than pure equal-power
                    let lg = 0.35 + 0.65 * gl;
                    let rg = 0.35 + 0.65 * gr;
                    out.d[0][i] = (n.shadow[0] * lg * 0.85) as f32;
                    out.d[1][i] = (n.shadow[1] * rg * 0.85) as f32;
                }
            } else {
                for i in 0..QUANTUM {
                    out.d[0][i] = m[i] * (gl * gain) as f32;
                    out.d[1][i] = m[i] * (gr * gain) as f32;
                }
            }
        }
    }
    n.out = out;
}

/// The shelf filters' slope `S` (1: the steepest slope without a resonant peak).
const SHELF_SLOPE: f64 = 1.0;

fn set_biquad(bq: &mut Biquad, t: FilterType, f: f64, q: f64, gain_db: f64, sr: f64) {
    let nyq = sr / 2.0;
    let f = (f / nyq).clamp(0.0, 1.0);
    let w0 = std::f64::consts::PI * f;
    let a = (10f64).powf(gain_db / 40.0);
    let (b0, b1, b2, a0, a1, a2);
    let cw = w0.cos();
    let sw = w0.sin();
    match t {
        FilterType::Lowpass | FilterType::Highpass => {
            if f >= 1.0 {
                if t == FilterType::Lowpass {
                    *bq = Biquad { b0: 1.0, b1: 0.0, b2: 0.0, a1: 0.0, a2: 0.0, z: bq.z };
                } else {
                    *bq = Biquad { b0: 0.0, b1: 0.0, b2: 0.0, a1: 0.0, a2: 0.0, z: bq.z };
                }
                return;
            }
            if f <= 0.0 {
                if t == FilterType::Lowpass {
                    *bq = Biquad { b0: 0.0, b1: 0.0, b2: 0.0, a1: 0.0, a2: 0.0, z: bq.z };
                } else {
                    *bq = Biquad { b0: 1.0, b1: 0.0, b2: 0.0, a1: 0.0, a2: 0.0, z: bq.z };
                }
                return;
            }
            let alpha = sw / 2.0 * (10f64).powf(-q / 20.0);
            if t == FilterType::Lowpass {
                b0 = (1.0 - cw) / 2.0;
                b1 = 1.0 - cw;
                b2 = (1.0 - cw) / 2.0;
            } else {
                b0 = (1.0 + cw) / 2.0;
                b1 = -(1.0 + cw);
                b2 = (1.0 + cw) / 2.0;
            }
            a0 = 1.0 + alpha;
            a1 = -2.0 * cw;
            a2 = 1.0 - alpha;
        }
        FilterType::Bandpass => {
            if f <= 0.0 || f >= 1.0 {
                *bq = Biquad { b0: 0.0, b1: 0.0, b2: 0.0, a1: 0.0, a2: 0.0, z: bq.z };
                return;
            }
            let alpha = sw / (2.0 * q.max(1e-4));
            b0 = alpha;
            b1 = 0.0;
            b2 = -alpha;
            a0 = 1.0 + alpha;
            a1 = -2.0 * cw;
            a2 = 1.0 - alpha;
        }
        FilterType::Notch => {
            let alpha = sw / (2.0 * q.max(1e-4));
            b0 = 1.0;
            b1 = -2.0 * cw;
            b2 = 1.0;
            a0 = 1.0 + alpha;
            a1 = -2.0 * cw;
            a2 = 1.0 - alpha;
        }
        FilterType::Allpass => {
            let alpha = sw / (2.0 * q.max(1e-4));
            b0 = 1.0 - alpha;
            b1 = -2.0 * cw;
            b2 = 1.0 + alpha;
            a0 = 1.0 + alpha;
            a1 = -2.0 * cw;
            a2 = 1.0 - alpha;
        }
        FilterType::Peaking => {
            let alpha = sw / (2.0 * q.max(1e-4));
            b0 = 1.0 + alpha * a;
            b1 = -2.0 * cw;
            b2 = 1.0 - alpha * a;
            a0 = 1.0 + alpha / a;
            a1 = -2.0 * cw;
            a2 = 1.0 - alpha / a;
        }
        FilterType::Lowshelf => {
            let alpha = sw / 2.0 * ((a + 1.0 / a) * (1.0 / SHELF_SLOPE - 1.0) + 2.0).sqrt();
            let k = 2.0 * a.sqrt() * alpha;
            b0 = a * ((a + 1.0) - (a - 1.0) * cw + k);
            b1 = 2.0 * a * ((a - 1.0) - (a + 1.0) * cw);
            b2 = a * ((a + 1.0) - (a - 1.0) * cw - k);
            a0 = (a + 1.0) + (a - 1.0) * cw + k;
            a1 = -2.0 * ((a - 1.0) + (a + 1.0) * cw);
            a2 = (a + 1.0) + (a - 1.0) * cw - k;
        }
        FilterType::Highshelf => {
            let alpha = sw / 2.0 * ((a + 1.0 / a) * (1.0 / SHELF_SLOPE - 1.0) + 2.0).sqrt();
            let k = 2.0 * a.sqrt() * alpha;
            b0 = a * ((a + 1.0) + (a - 1.0) * cw + k);
            b1 = -2.0 * a * ((a - 1.0) + (a + 1.0) * cw);
            b2 = a * ((a + 1.0) + (a - 1.0) * cw - k);
            a0 = (a + 1.0) - (a - 1.0) * cw + k;
            a1 = 2.0 * ((a - 1.0) - (a + 1.0) * cw);
            a2 = (a + 1.0) - (a - 1.0) * cw - k;
        }
    }
    bq.b0 = b0 / a0;
    bq.b1 = b1 / a0;
    bq.b2 = b2 / a0;
    bq.a1 = a1 / a0;
    bq.a2 = a2 / a0;
}

#[cfg(test)]
mod speaker_tests {
    use super::*;

    /// The speakers start off: whatever runs without the window (a headless
    /// mode, a tool, a test) is silent unless it was turned on
    #[test]
    fn the_speakers_start_off() {
        assert!(!DEVICE_OUTPUT.load(Ordering::Relaxed), "DEVICE_OUTPUT must start false; only the windowed game turns it on");
        assert!(!device_output());
        assert!(!AudioContext::new().plays_on_device(), "a context opened the device with the speakers off");
    }

    /// Waits up to 2 s for `f`
    fn within(f: impl Fn() -> bool) -> bool {
        let t0 = std::time::Instant::now();
        while !f() && t0.elapsed().as_secs_f64() < 2.0 {
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        f()
    }

    /// The windowed game with no output device keeps time on the silent
    /// clock (it had none: its time stood at 0 and what it was sent piled up)
    #[test]
    fn with_no_device_the_clock_still_moves() {
        let ac = AudioContext::on_output(None);
        assert!(!ac.plays_on_device());
        assert!(within(|| ac.current_time() > 0.0), "the clock never moved");
        let g = ac.create_gain();
        g.connect(&ac.destination());
        g.gain().set_value(0.5);
        assert!(ac.inner.shared.cmds.lock().unwrap().is_empty(), "commands queued with nothing to render them");
    }

    /// The silent clock queues nothing, and its thread ends with its context
    #[test]
    fn the_silent_clock_drops_what_is_sent_and_ends_with_its_context() {
        let ac = AudioContext::new_silent();
        let o = ac.create_oscillator();
        o.connect(&ac.destination());
        o.start(0.0);
        ac.set_muted(true);
        assert!(ac.inner.shared.cmds.lock().unwrap().is_empty(), "commands queued with nothing to render them");
        assert!(within(|| ac.current_time() > 0.0), "the clock never moved");
        let shared = Arc::downgrade(&ac.inner.shared);
        drop(o);
        drop(ac);
        // (the thread holds it only for a tick, then finds it gone and ends)
        assert!(within(|| shared.upgrade().is_none()), "the clock outlived its context");
    }
}
