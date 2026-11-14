//! Replays (`docs/engineering/REPLAYS.md`): what a player did, frame by
//! frame (each frame's interval in clock ticks and its input), from a
//! start a fresh game can stand at (a fresh process, or a floor's anchor),
//! with the state kept every 60 frames, so a replay proves the game repeats
//! and names the first frame where it does not (Factorio's per-tick check).
//! The file is `OCREPLAY` (`file`); `Recorder` makes one from play,
//! `Player` plays one back from any of its segments.
pub mod file;

use crate::prelude::*;
pub use file::{HASH_EVERY, Quantizer, ReplayFile, Segment, Start};
use serde_json::{Value, json};
use std::collections::BTreeMap;

/// One input the window gives the game, in the order it came.
#[derive(Clone, Debug, PartialEq)]
pub enum InputEvent {
    Down(String),
    Up(String),
    /// mouse motion while the pointer is locked
    Look(f64, f64),
    /// a click on the HUD element of this name (`hud_hit_test`)
    Click(String),
    Lock(bool),
    Blur,
    /// the mouse wheel
    Wheel(f64),
    /// the pointer moved onto a menu line (`menu_hover`), by its id
    Hover(String),
    /// the window's view changed: its width, height and scale factor
    View(f64, f64, f64),
}

/// One frame's input.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FrameInput {
    pub events: Vec<InputEvent>,
}

impl FrameInput {
    pub fn down(&mut self, k: &str) {
        self.events.push(InputEvent::Down(k.into()));
    }
    pub fn up(&mut self, k: &str) {
        self.events.push(InputEvent::Up(k.into()));
    }
    pub fn look(&mut self, dx: f64, dy: f64) {
        self.events.push(InputEvent::Look(dx, dy));
    }
    pub fn to_json(&self) -> Value {
        Value::Array(
            self.events
                .iter()
                .map(|e| match e {
                    InputEvent::Down(k) => json!(["d", k]),
                    InputEvent::Up(k) => json!(["u", k]),
                    InputEvent::Look(x, y) => json!(["l", x, y]),
                    InputEvent::Click(h) => json!(["c", h]),
                    InputEvent::Lock(on) => json!(["k", on]),
                    InputEvent::Blur => json!(["b"]),
                    InputEvent::Wheel(d) => json!(["w", d]),
                    InputEvent::Hover(h) => json!(["h", h]),
                    InputEvent::View(w, h, s) => json!(["v", w, h, s]),
                })
                .collect(),
        )
    }
    pub fn from_json(v: &Value) -> FrameInput {
        let events = v
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|e| {
                let s = |i: usize| e[i].as_str().unwrap_or("").to_string();
                Some(match e[0].as_str()? {
                    "d" => InputEvent::Down(s(1)),
                    "u" => InputEvent::Up(s(1)),
                    "l" => InputEvent::Look(e[1].as_f64()?, e[2].as_f64()?),
                    "c" => InputEvent::Click(s(1)),
                    "k" => InputEvent::Lock(e[1].as_bool()?),
                    "b" => InputEvent::Blur,
                    "w" => InputEvent::Wheel(e[1].as_f64()?),
                    "h" => InputEvent::Hover(s(1)),
                    "v" => InputEvent::View(e[1].as_f64()?, e[2].as_f64()?, e[3].as_f64()?),
                    _ => return None,
                })
            })
            .collect();
        FrameInput { events }
    }
}

/// Records play as the window feeds it (`docs/engineering/REPLAYS.md`):
/// `push` each input event as it reaches the game and `end_frame` after
/// each `Game::frame`. It keeps every frame, the state after every
/// `HASH_EVERY`th, a new segment at each floor play enters (its anchor,
/// `Game::entered`), and for playtests a telemetry row a second with the
/// director's log.
///
/// It is the game's log of play, always kept: until a recording begins,
/// only the current floor's segment (from the start a replay of it would
/// stand at), so a recording can begin at any moment and its replay
/// reaches that moment by simulating from the segment's start. While a
/// recording is made (`start_recording` to `take_recording`) it keeps every
/// segment from the one its in-point is in.
pub struct Recorder {
    pub segments: Vec<Segment>,
    pending: FrameInput,
    frame: u64,
    rows: Vec<Value>,
    events: Vec<Value>,
    next_row: f64,
    seen: (i32, usize),
    /// the in-point of the recording being made: the first frame it shows
    from: Option<u64>,
}

impl Recorder {
    /// From a fresh process: the per-frame stream's seed it was made with,
    /// its launch flags, what its storage held before it booted, and the
    /// settings and view it has now.
    pub fn session(native_seed: u64, flags: &str, storage: BTreeMap<String, String>, g: &Game) -> Recorder {
        Self::starting(Start::Session {
            native_seed,
            flags: flags.to_string(),
            storage,
            settings: g.settings.to_json(),
            view: [g.view_width, g.view_height, g.scale_factor],
        })
    }
    /// From a floor's anchor.
    pub fn floor(anchor: crate::session::anchor::Anchor) -> Recorder {
        Self::starting(Start::Floor(anchor))
    }
    fn starting(start: Start) -> Recorder {
        Recorder {
            segments: vec![Segment { start, first_frame: 0, frames: vec![], hashes: vec![] }],
            pending: FrameInput::default(),
            frame: 0,
            rows: vec![],
            events: vec![],
            next_row: 0.0,
            seen: (0, 0),
            from: None,
        }
    }
    /// An input event as it reaches the game, in order.
    pub fn push(&mut self, e: InputEvent) {
        self.pending.events.push(e);
    }
    /// How many frames have ended: the index of the next.
    pub fn frames(&self) -> u64 {
        self.frame
    }
    /// The frame and the state after it, every `HASH_EVERY` frames; a new
    /// segment if it entered a floor (the segments before it let go, unless
    /// a recording needs them); telemetry.
    pub fn end_frame(&mut self, g: &mut Game, ticks: u32) {
        let seg = self.segments.last_mut().expect("a recorder has a segment");
        let mut input = std::mem::take(&mut self.pending);
        // (most frames hold one event, a look: no room kept for more)
        input.events.shrink_to_fit();
        seg.frames.push((ticks, input));
        if (self.frame + 1).is_multiple_of(HASH_EVERY) {
            seg.hashes.push((self.frame, g.state_hash()));
        }
        self.frame += 1;
        if let Some(a) = g.entered.take() {
            self.segments.push(Segment { start: Start::Floor(a), first_frame: self.frame, frames: vec![], hashes: vec![] });
            self.let_go(self.from.unwrap_or(self.frame));
        }
        if g.perf_t >= self.next_row {
            self.next_row = g.perf_t + 1.0;
            let p = &g.player;
            let r4 = |x: f64| (x * 10000.0).round() / 10000.0;
            self.rows.push(json!({
                "f": self.frame - 1, "t": r4(g.perf_t), "level": g.level_num, "seed": g.run_seed, "state": g.state,
                "x": r4(p.pos.x), "z": r4(p.pos.z), "phase": format!("{:?}", g.director.phase),
                "menace": r4(g.director.menace), "sanity": r4(p.sanity), "stamina": r4(p.stamina), "battery": r4(p.battery),
            }));
        }
        if g.level_num != self.seen.0 || g.director.log.len() < self.seen.1 {
            self.seen = (g.level_num, 0);
        }
        for &(t, what) in &g.director.log[self.seen.1..] {
            self.events.push(json!({ "f": self.frame - 1, "t": t, "level": g.level_num, "seed": g.run_seed, "what": what }));
        }
        self.seen.1 = g.director.log.len();
    }
    /// Lets go of the segments that end before frame `keep` and of the
    /// telemetry from before the first segment kept.
    fn let_go(&mut self, keep: u64) {
        let first = self.segments.iter().rposition(|s| s.first_frame <= keep).unwrap_or(0);
        self.segments.drain(..first);
        let from = self.segments[0].first_frame;
        let after = |v: &Value| v["f"].as_u64().is_some_and(|f| f >= from);
        self.rows.retain(after);
        self.events.retain(after);
    }
    /// A recording begins with the next frame (its in-point).
    pub fn start_recording(&mut self) {
        self.from = Some(self.frame);
    }
    /// The in-point of the recording being made, if one is.
    pub fn recording_from(&self) -> Option<u64> {
        self.from
    }
    /// The recording made so far, under `header` (the build, when): the
    /// segments from the one its in-point is in, its in-point and its
    /// out-point (the frames end there) in the header, the telemetry from
    /// the in-point. Without a recording, the whole log.
    pub fn snapshot(&self, mut header: Value) -> ReplayFile {
        let from = self.from.unwrap_or(self.segments[0].first_frame);
        let first = self.segments.iter().rposition(|s| s.first_frame <= from).unwrap_or(0);
        header["in"] = json!(from);
        header["out"] = json!(self.frame);
        let after = |v: &&Value| v["f"].as_u64().is_some_and(|f| f >= from);
        ReplayFile {
            header,
            segments: self.segments[first..].to_vec(),
            telemetry: json!({
                "rows": self.rows.iter().filter(after).collect::<Vec<_>>(),
                "events": self.events.iter().filter(after).collect::<Vec<_>>(),
            }),
        }
    }
    /// The recording being made, ended here (`snapshot`); the log goes on,
    /// letting go of what only the recording needed.
    pub fn take_recording(&mut self, header: Value) -> Option<ReplayFile> {
        self.from?;
        let f = self.snapshot(header);
        self.from = None;
        self.let_go(self.frame);
        Some(f)
    }
    /// About how much memory the log holds, in bytes: its frames, their
    /// events (and the names in them), the kept states and the telemetry.
    pub fn heap_bytes(&self) -> usize {
        use std::mem::size_of;
        let event = |e: &InputEvent| match e {
            InputEvent::Down(s) | InputEvent::Up(s) | InputEvent::Click(s) | InputEvent::Hover(s) => s.capacity(),
            _ => 0,
        };
        let frames: usize = self
            .segments
            .iter()
            .map(|s| {
                s.frames.capacity() * size_of::<(u32, FrameInput)>()
                    + s.frames
                        .iter()
                        .map(|(_, i)| i.events.capacity() * size_of::<InputEvent>() + i.events.iter().map(event).sum::<usize>())
                        .sum::<usize>()
                    + s.hashes.capacity() * size_of::<(u64, StateHash)>()
                    + match &s.start {
                        Start::Floor(a) => a.state.to_string().len(),
                        Start::Session { storage, settings, .. } => {
                            storage.iter().map(|(k, v)| k.len() + v.len()).sum::<usize>() + settings.to_string().len()
                        }
                    }
            })
            .sum();
        // (telemetry as its text: a fair measure of JSON's heap)
        let telemetry: usize = self.rows.iter().chain(&self.events).map(|v| v.to_string().len() * 2).sum();
        frames + telemetry
    }
}

/// Where the state of a replay first left its recording: the frame and the
/// parts of the state that differ there.
#[derive(Clone, Debug, PartialEq)]
pub struct Divergence {
    pub frame: u64,
    pub parts: Vec<&'static str>,
}

/// Plays a recording back into a game, from one of its segments to its end,
/// checking the state wherever the recording kept it.
pub struct Player {
    pub file: ReplayFile,
    seg: usize,
    i: usize,
    entry: Option<f64>,
    /// the index (in the whole recording) of the next frame to run
    pub frame: u64,
    /// the first frame whose state was not the recording's
    pub diverged: Option<Divergence>,
}

impl Player {
    /// The game a replay of `file` from its segment `seg` starts in. `make`
    /// makes a fresh game with the launch flags given, not yet booted (for a
    /// fresh process it is called right after the per-frame stream is
    /// seeded); it is then booted as the window boots one (`boot_in_view`),
    /// with the storage and view the recording started with.
    pub fn start(file: ReplayFile, seg: usize, make: impl FnOnce(&str) -> Game) -> Result<(Game, Player), String> {
        let s = file.segments.get(seg).ok_or_else(|| format!("the recording has no segment {seg}"))?;
        let first = s.first_frame;
        let (g, entry) = match &s.start {
            Start::Session { native_seed, flags, storage, settings, view } => {
                crate::base::rng::seed_native(*native_seed);
                let mut g = make(flags);
                g.storage.clone_from(storage);
                g.boot_in_view(Some(*view));
                g.settings = crate::session::settings::Settings::from_json(settings);
                (g, None)
            }
            Start::Floor(a) => {
                let mut g = make(file.header["flags"].as_str().unwrap_or_default());
                g.boot_in_view(None);
                let dt = g.start_from_anchor(a)?;
                (g, Some(dt))
            }
        };
        Ok((g, Player { file, seg, i: 0, entry, frame: first, diverged: None }))
    }

    /// The first frame the recording shows (frames before it are reached
    /// by simulating, unseen): its in-point.
    pub fn in_point(&self) -> u64 {
        self.file.in_point()
    }

    /// Runs the next recorded frame (for a floor's start, first the rest of
    /// the frame it was entered in); false once the recording is done.
    pub fn step(&mut self, g: &mut Game) -> bool {
        if let Some(dt) = self.entry.take() {
            g.frame(dt);
            return true;
        }
        loop {
            let Some(s) = self.file.segments.get(self.seg) else { return false };
            if let Some((ticks, input)) = s.frames.get(self.i) {
                replay_frame(g, *ticks, input);
                if self.diverged.is_none()
                    && let Ok(k) = s.hashes.binary_search_by_key(&self.frame, |h| h.0)
                {
                    let got = g.state_hash();
                    if got != s.hashes[k].1 {
                        self.diverged = Some(Divergence { frame: self.frame, parts: s.hashes[k].1.diff(&got) });
                    }
                }
                self.i += 1;
                self.frame += 1;
                return true;
            }
            // on into the next segment: play crossed into it by itself
            self.seg += 1;
            self.i = 0;
        }
    }

    /// Simulates up to `to` (the next frame to run is then `to`), calling
    /// `progress` with the share done now and then; false if the recording
    /// ends first.
    pub fn seek(&mut self, g: &mut Game, to: u64, mut progress: impl FnMut(f64)) -> bool {
        let (from, n) = (self.frame, to.saturating_sub(self.frame).max(1));
        while self.frame < to || self.entry.is_some() {
            if !self.step(g) {
                return false;
            }
            if (self.frame - from).is_multiple_of(600) {
                progress((self.frame - from) as f64 / n as f64);
            }
        }
        true
    }
}

/// Run one recorded frame: its input, then its interval on the game's clock.
pub fn replay_frame(g: &mut Game, ticks: u32, i: &FrameInput) {
    g.apply_input(i);
    let dt = g.advance_clock(ticks);
    g.frame(dt);
}

/// The state after a frame, hashed by part so a divergence says where it is.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct StateHash {
    pub player: u64,
    pub entities: u64,
    pub director: u64,
    pub world: u64,
}

impl StateHash {
    /// The parts that differ from `other`.
    pub fn diff(&self, other: &StateHash) -> Vec<&'static str> {
        let mut out = vec![];
        for (name, a, b) in [
            ("player", self.player, other.player),
            ("entities", self.entities, other.entities),
            ("director", self.director, other.director),
            ("world", self.world, other.world),
        ] {
            if a != b {
                out.push(name);
            }
        }
        out
    }
}

struct Fnv(u64);
impl Fnv {
    fn new() -> Fnv {
        Fnv(0xcbf2_9ce4_8422_2325)
    }
    fn bytes(&mut self, b: &[u8]) {
        for x in b {
            self.0 ^= *x as u64;
            self.0 = self.0.wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
    fn f(&mut self, v: f64) {
        self.bytes(&v.to_bits().to_le_bytes());
    }
    fn s(&mut self, v: &str) {
        self.bytes(v.as_bytes());
        self.bytes(&[0]);
    }
}

impl Game {
    /// Feed one frame's input, in order (as the window would).
    pub fn apply_input(&mut self, i: &FrameInput) {
        for e in &i.events {
            self.apply_event(e);
        }
    }

    /// One input event: the window's driver feeds play through here, and a
    /// replay feeds what it recorded, so the two read the same.
    pub fn apply_event(&mut self, e: &InputEvent) {
        match e {
            InputEvent::Down(k) => self.key_down(k, false),
            InputEvent::Up(k) => self.key_up(k),
            InputEvent::Look(x, y) => self.mouse_move(*x, *y),
            InputEvent::Click(h) => self.click(h),
            InputEvent::Lock(on) => self.pointer_lock_changed(*on),
            InputEvent::Blur => self.blur(),
            InputEvent::Wheel(d) => self.wheel(*d),
            InputEvent::Hover(h) => {
                if let Some(k) = self.menu.items.iter().position(|i| i.id == h.as_str()) {
                    self.menu.sel = k;
                }
            }
            InputEvent::View(w, h, s) => self.set_view(*w, *h, *s),
        }
    }

    /// Boots a fresh game as the window boots one: its view first (when one
    /// is given), then the boot and the title's sound. A recorded session
    /// starts so, and its replay too (`Player::start`'s `make`).
    pub fn boot_in_view(&mut self, view: Option<[f64; 3]>) {
        if let Some([w, h, scale]) = view {
            self.view_width = w;
            self.view_height = h;
            self.scale_factor = scale;
            self.quality.max_pr = scale.min(2.0);
            self.quality.pr = self.quality.max_pr;
        }
        self.now_ms = 0.0;
        self.boot();
        self.audio_init();
    }

    /// The window's view: its size in points and its scale factor (the
    /// camera's aspect, and so how far it sees, hang on it).
    pub fn set_view(&mut self, w: f64, h: f64, scale: f64) {
        self.view_width = w;
        self.view_height = h;
        self.scale_factor = scale;
        self.quality.max_pr = scale.min(2.0);
        self.quality.pr = self.quality.pr.min(self.quality.max_pr);
        self.resize();
    }

    /// Hash of what a replay must reproduce exactly.
    pub fn state_hash(&self) -> StateHash {
        let p = &self.player;
        let mut h = Fnv::new();
        for v in [p.pos.x, p.pos.y, p.pos.z, p.yaw, p.pitch, p.stamina, p.sanity, p.battery] {
            h.f(v);
        }
        h.bytes(&[p.flashlight_on as u8, p.crouch as u8]);
        let player = h.0;
        let mut h = Fnv::new();
        for e in &self.entities {
            let q = e.pos();
            h.s(&e.kind);
            h.s(&e.state);
            for v in [q.x, q.y, q.z, e.yaw, e.freeze] {
                h.f(v);
            }
        }
        let entities = h.0;
        let d = &self.director;
        let mut h = Fnv::new();
        h.s(&format!("{:?}", d.phase));
        for v in [d.menace, d.tension, d.t] {
            h.f(v);
        }
        h.f(d.stalker.map_or(-1.0, |s| s as f64));
        let director = h.0;
        let mut h = Fnv::new();
        h.f(self.level_num as f64);
        h.s(&self.state);
        h.f(self.ambient_log.len() as f64);
        h.f(self.animators.len() as f64);
        // how much chance was drawn, and the clocks: most divergences show
        // in the first, a step or a time read wrongly in the others
        h.f(crate::base::rng::native_draws() as f64);
        for v in [self.perf_t, self.now_ms] {
            h.f(v);
        }
        let world = h.0;
        StateHash { player, entities, director, world }
    }
}
