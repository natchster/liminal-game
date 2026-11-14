//! Replays: what a player did, frame by frame, and a hash of the game's state
//! after every frame, so a replay proves the simulation repeats and points at
//! the first frame where it does not (Factorio's per-tick check).
//!
//! A replay starts from the same `(run seed, level)` in a fresh process or
//! thread (the caches and the per-frame random stream are thread-local), with
//! the per-frame randomness seeded first (`base::rng::seed_native`), and feeds
//! the same inputs at the same fixed time step. `crates/backrooms/tests/replay.rs` records a bot
//! run and replays it; the playtest tooling can record people the same way.
use crate::prelude::*;
use serde_json::{Value, json};

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
                    _ => return None,
                })
            })
            .collect();
        FrameInput { events }
    }
}

/// A recorded session: enough to replay it exactly, and what was measured
/// along the way. Written by the game with `--record FILE` (opt-in; the file
/// stays on this machine) and played back with `--replay FILE`.
#[derive(Clone, Debug, Default)]
pub struct Recording {
    /// the per-frame random stream's seed, set before the game was made
    pub native_seed: u64,
    /// per frame: its interval in ticks of the game's clock (from which its
    /// step and the clock come, `Game::advance_clock`), and its input
    pub frames: Vec<(u32, FrameInput)>,
    /// once a second of game time: where the player was and how it was going
    pub rows: Vec<Value>,
    /// the director's log, level by level
    pub events: Vec<Value>,
}

impl Recording {
    pub fn to_json(&self) -> Value {
        json!({
            "version": 2,
            "native_seed": self.native_seed,
            "frames": self.frames.iter().map(|(ticks, i)| json!([ticks, i.to_json()])).collect::<Vec<_>>(),
            "rows": self.rows,
            "events": self.events,
        })
    }
    pub fn from_json(v: &Value) -> Recording {
        Recording {
            native_seed: v["native_seed"].as_u64().unwrap_or(0),
            frames: v["frames"]
                .as_array()
                .into_iter()
                .flatten()
                .map(|f| (f[0].as_u64().map_or(0, |t| t.min(u64::from(u32::MAX)) as u32), FrameInput::from_json(&f[1])))
                .collect(),
            rows: v["rows"].as_array().cloned().unwrap_or_default(),
            events: v["events"].as_array().cloned().unwrap_or_default(),
        }
    }
}

/// Records a session as the window feeds the game: call the input methods as
/// the input arrives and `end_frame` after each `Game::frame`.
#[derive(Default)]
pub struct Recorder {
    pub rec: Recording,
    pending: FrameInput,
    next_row: f64,
    seen: (i32, usize),
}

impl Recorder {
    pub fn new(native_seed: u64) -> Recorder {
        Recorder { rec: Recording { native_seed, ..Default::default() }, ..Default::default() }
    }
    pub fn push(&mut self, e: InputEvent) {
        self.pending.events.push(e);
    }
    /// The frame that just ran: its interval in clock ticks, then telemetry.
    pub fn end_frame(&mut self, g: &Game, ticks: u32) {
        self.rec.frames.push((ticks, std::mem::take(&mut self.pending)));
        if g.perf_t >= self.next_row {
            self.next_row = g.perf_t + 1.0;
            let p = &g.player;
            let r4 = |x: f64| (x * 10000.0).round() / 10000.0;
            self.rec.rows.push(json!({
                "t": r4(g.perf_t), "level": g.level_num, "seed": g.run_seed, "state": g.state,
                "x": r4(p.pos.x), "z": r4(p.pos.z), "phase": format!("{:?}", g.director.phase),
                "menace": r4(g.director.menace), "sanity": r4(p.sanity), "stamina": r4(p.stamina), "battery": r4(p.battery),
            }));
        }
        if g.level_num != self.seen.0 || g.director.log.len() < self.seen.1 {
            self.seen = (g.level_num, 0);
        }
        for &(t, what) in &g.director.log[self.seen.1..] {
            self.rec.events.push(json!({ "t": t, "level": g.level_num, "seed": g.run_seed, "what": what }));
        }
        self.seen.1 = g.director.log.len();
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
            }
        }
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
