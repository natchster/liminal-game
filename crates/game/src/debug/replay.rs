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
use serde_json::{json, Value};

/// One frame's input.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FrameInput {
    /// keys pressed and released this frame (`KeyW`, `ShiftLeft`, ...)
    pub down: Vec<String>,
    pub up: Vec<String>,
    /// mouse motion (pointer locked)
    pub look: (f64, f64),
}

impl FrameInput {
    pub fn to_json(&self) -> Value {
        json!({ "d": self.down, "u": self.up, "l": [self.look.0, self.look.1] })
    }
    pub fn from_json(v: &Value) -> FrameInput {
        let list = |k: &str| v[k].as_array().into_iter().flatten().filter_map(|s| s.as_str().map(String::from)).collect();
        FrameInput { down: list("d"), up: list("u"), look: (v["l"][0].as_f64().unwrap_or(0.0), v["l"][1].as_f64().unwrap_or(0.0)) }
    }
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
    /// Feed one frame's input (as the window would).
    pub fn apply_input(&mut self, i: &FrameInput) {
        for k in &i.down {
            self.key_down(k, false);
        }
        for k in &i.up {
            self.key_up(k);
        }
        if i.look != (0.0, 0.0) {
            self.mouse_move(i.look.0, i.look.1);
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
        h.f(d.stalker.map(|s| s as f64).unwrap_or(-1.0));
        let director = h.0;
        let mut h = Fnv::new();
        h.f(self.level_num as f64);
        h.s(&self.state);
        h.f(self.ambient_log.len() as f64);
        h.f(self.animators.len() as f64);
        let world = h.0;
        StateHash { player, entities, director, world }
    }
}
