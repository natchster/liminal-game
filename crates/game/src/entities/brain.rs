//! The creatures' brain: what every creature of the rooms shares.
//!
//! A creature is always in one **mode** (`Mode`), and it changes mode only by
//! a rule written in its kind's table (`docs/ENTITIES.md`), each change
//! logged with its reason (`Brain::log`), so what a creature did can always
//! be read back and tested.
//!
//! Each frame it **perceives** (`Percept`): how well it sees you, how loud
//! you are where it stands, whether you are looking at it, whether your lamp
//! is on it, how lit you are. What it perceives fills its **awareness**
//! (0: nothing, 1: it is sure where you are) and its **memory** (where it last
//! saw you, where it last heard something, and when). Awareness drains when
//! nothing feeds it. The thresholds are the same for every kind: from
//! `SUSPECT` it turns to look (the tell), from `SURE` it has you. How fast
//! each sense fills it is the kind's.
//!
//! The director does not move creatures; it gives **orders** (`Order`):
//! come toward where the player is (unseen), keep away, or nothing.

/// Awareness from which a creature stops and looks toward what it noticed.
pub const SUSPECT: f64 = 0.3;
/// Awareness from which a creature is sure where you are.
pub const SURE: f64 = 1.0;
/// Whether awareness `aw` is sure (within a frame's drain of `SURE`: being
/// told where you are sets it to 1, and a silent frame takes a sliver off).
pub fn sure(aw: f64) -> bool {
    aw >= SURE - 0.02
}

/// The most mode changes one creature's log keeps.
pub const LOG_CAP: usize = 256;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Hash)]
pub enum Mode {
    /// At its place, doing what it does there (listening, swaying, waiting)
    #[default]
    Idle,
    /// Walking its ground
    Roam,
    /// Something caught its attention: it stops and turns to it (the tell)
    Alert,
    /// Going to look where the noise or the glimpse was
    Search,
    /// Sent by the director: coming toward where you are, out of sight
    Stalk,
    /// It has you: it comes
    Hunt,
    /// The blow: a short wind-up you can see, then the contact
    Strike,
    /// Driven off by its counter (your light, your gaze, ...): it backs away
    Recoil,
    /// After a close call, sent far off by the director; its senses are dull
    Withdraw,
    /// Holding still because you are watching (the Watcher), or lying in wait
    Hold,
}

impl Mode {
    pub fn name(self) -> &'static str {
        match self {
            Mode::Idle => "idle",
            Mode::Roam => "roam",
            Mode::Alert => "alert",
            Mode::Search => "search",
            Mode::Stalk => "stalk",
            Mode::Hunt => "hunt",
            Mode::Strike => "strike",
            Mode::Recoil => "recoil",
            Mode::Withdraw => "withdraw",
            Mode::Hold => "hold",
        }
    }
    /// Whether the creature is after you (the director's peak, the drone)
    pub fn after_you(self) -> bool {
        matches!(self, Mode::Hunt | Mode::Strike)
    }
    /// The old string state the shared code (director, HUD, tests) reads
    pub fn legacy_state(self) -> &'static str {
        match self {
            Mode::Hunt | Mode::Strike => "hunt",
            // (held in your sight, close: the director feels it as a search)
            Mode::Search | Mode::Alert | Mode::Hold => "search",
            _ => "wander",
        }
    }
}

/// What the director tells a creature.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum Order {
    #[default]
    None,
    /// Come toward the player's part of the level, staying out of sight
    Stalk,
    /// The build-up's last stretch: it is told where the player is (x, z)
    Seek(f64, f64),
    /// Keep away for this many seconds more, senses dulled
    Away(f64),
}

/// What a creature perceives of you this frame.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Percept {
    /// Distance on the floor, m
    pub dist: f64,
    /// A straight line between its eyes and yours that no wall cuts
    pub line: bool,
    /// You are inside its field of view (or right beside it)
    pub in_view: bool,
    /// 0..1: how well it sees you (0 when it cannot)
    pub sight: f64,
    /// 0..1: how loud you are where it stands (0 when it cannot hear you)
    pub sound: f64,
    /// The length of the way from it to you along the level, m
    pub path: f64,
    /// You are looking at it, and could see it
    pub watched: bool,
    /// Your lamp is on and aimed at it
    pub lamp_on_it: bool,
    /// Your lamp is on
    pub lamp: bool,
    /// The light where you stand, 0..1 (the light map)
    pub you_lit: f64,
    /// The light where it stands, 0..1
    pub it_lit: f64,
    /// You are running
    pub running: bool,
    /// You are crouched
    pub crouched: bool,
    /// You are hidden (a hiding place, a shut cell)
    pub hidden: bool,
}

/// What a creature remembers.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Memory {
    /// 0..1: how sure it is where you are
    pub awareness: f64,
    /// Where and when it last saw you
    pub seen: Option<(f64, f64, f64)>,
    /// Where and when it last heard something
    pub heard: Option<(f64, f64, f64)>,
}

/// One change of mode: game time, from, to, and why.
pub type Change = (f64, Mode, Mode, &'static str);

#[derive(Clone, Debug, Default)]
pub struct Brain {
    pub mode: Mode,
    /// Seconds in the current mode
    pub mode_t: f64,
    pub mem: Memory,
    pub order: Order,
    /// The last percept (for the animation and the tests)
    pub last: Percept,
    /// Where it is going (x, z), if anywhere
    pub goal: Option<(f64, f64)>,
    /// Seconds to the next new goal while roaming or searching
    pub goal_t: f64,
    /// A general-purpose timer of the current mode (wind-up, hold, recoil)
    pub t: f64,
    /// How many times its counter has driven it off on this level
    pub repelled: u32,
    pub log: Vec<Change>,
}

impl Brain {
    pub fn new(mode: Mode) -> Brain {
        Brain { mode, ..Default::default() }
    }

    /// Change mode (no-op when already in it), logging why.
    pub fn set(&mut self, to: Mode, why: &'static str, now: f64) {
        if to == self.mode {
            return;
        }
        if self.log.len() >= LOG_CAP {
            self.log.remove(0);
        }
        self.log.push((now, self.mode, to, why));
        self.mode = to;
        self.mode_t = 0.0;
        self.t = 0.0;
    }

    /// Feed awareness: `gain` per second for `dt`, never above `cap`.
    pub fn feed(&mut self, gain: f64, dt: f64, cap: f64) {
        self.mem.awareness = (self.mem.awareness + gain * dt).min(cap.max(self.mem.awareness));
    }

    /// Drain awareness at `rate` per second.
    pub fn drain(&mut self, rate: f64, dt: f64) {
        self.mem.awareness = (self.mem.awareness - rate * dt).max(0.0);
    }

    /// The last change into `mode`, if any.
    pub fn last_into(&self, mode: Mode) -> Option<&Change> {
        self.log.iter().rev().find(|c| c.2 == mode)
    }
}
