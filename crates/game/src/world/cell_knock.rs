//! What knocks at your cell (`docs/MEGASTRUCTURE.md`, G6). Sealed in a cell
//! while the self-assembly's fog is up, something knocks at the door every
//! few seconds (`assembly::KNOCK`). What you do is a choice:
//!
//! - **Stay silent**, and after a few knocks the knocking stops: it goes,
//!   and does not come back while the fog is up.
//! - **Answer it** (`[E] KNOCK`, inside the cell at its door: your own three
//!   raps, the knock of `world::knock`), and a moment later it tries the
//!   wheel: the wheel turns from outside, a pull at a time, heard and seen
//!   at the door. Unless you hold it (`[E] HOLD THE WHEEL`: a press holds it
//!   for `GRIP`, and E held keeps it held, so held or pressed again and
//!   again), the door gives after `GIVE` seconds of turning: the bolts drawn,
//!   the leaf swung in, the cell no longer sealed, and the fog's harm
//!   (`assembly::DRAIN`, `WADE_DRAIN`, `LAST`) reaches you inside as it does
//!   outside wherever its surface is over you. Held, the wheel strains and
//!   the leaf shudders in its frame, and after a while it lets go and goes.
//!
//! A door that gave stays open while the fog is up (its seal is broken: the
//! wheel will not shut it until the all-clear). A door left open, or opened
//! while it is up, lets the fog in the same way (`assembly::in_fog` exempts
//! only a sealed cell). While something is at the door, the door is not
//! opened from inside: its verb is the knock, then the hold.
//!
//! What it chooses (how many knocks it gives unanswered before it goes, how
//! long after your answer it starts on the wheel, how long it tries) comes
//! from a stream of its own, seeded from the floor's seed with a salt, the
//! assembly's count and the cell (`VISIT_SALT`): no draw of the build, and
//! none of the per-frame stream at a floor's start. The knocks' cadence and
//! your raps' rhythm are drawn from the per-frame stream when they come, as
//! before. Its sounds play from the door (`play_from`).
use crate::prelude::*;
use crate::session::input::Act;
use crate::world::assembly::{KNOCK, WHEEL_SHUT};

/// The salt of the stream what knocks at a cell chooses from
pub const VISIT_SALT: u64 = 0xCE11_D002_0000_0006;
/// How many knocks it gives unanswered before it goes (both counted)
pub const PATIENCE: (u32, u32) = (2, 4);
/// From your answer to the wheel's first turn, s
pub const WHEEL_AFTER: (f64, f64) = (0.9, 1.6);
/// How long it tries the wheel, s (longer than `GIVE`: let go, it gives)
pub const TRIES: (f64, f64) = (6.0, 10.0);
/// Seconds of turning, not held, that make the door give
pub const GIVE: f64 = 2.5;
/// A press holds the wheel this long, s (E held keeps it held)
pub const GRIP: f64 = 0.6;
/// How far a wheel turns to shut or open its door, radians (two turns and
/// a half; what it turns from outside before the door gives, too)
pub const WHEEL_TURNS: f64 = 5.0 * PI;
/// How far a door that gave stands open, radians (into the room; past the
/// half radian a shut door is taken for)
pub const GAVE_OPEN: f64 = 0.9;
/// A pull at the wheel heard every this many seconds while it tries
const PULL: f64 = 0.8;

/// Where what knocks at the cell is
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Visit {
    /// nothing at the door yet this time
    #[default]
    Away,
    /// knocking now and then: it can be answered
    Knocking,
    /// answered: trying the wheel
    Trying,
    /// it went (unanswered, let go of the wheel, or the fog went down):
    /// nothing more this time
    Gone,
    /// the door gave: open until the fog is gone
    Gave,
}

/// What knocks at the cell, this time the fog is up (reset at each siren)
#[derive(Clone, Debug, Default)]
pub struct AtTheDoor {
    pub visit: Visit,
    /// the cell it is at (`BuildingPlan::cells` index)
    pub cell: Option<usize>,
    /// its knocks so far, and how many it gives unanswered before it goes
    pub knocks: u32,
    pub patience: u32,
    /// seconds from your answer to the wheel's first turn, and how long it
    /// tries after that
    pub wait: f64,
    pub tries: f64,
    /// seconds since the wheel's first turn (below 0: not yet)
    pub t: f64,
    /// how far it has turned the wheel toward the door giving, 0..1
    pub give: f64,
    /// seconds your hold on the wheel lasts
    pub grip: f64,
    /// seconds of it held, all told (for the log and the tests)
    pub held_for: f64,
    /// seconds to the next pull's sound
    pub pull_t: f64,
}

impl Game {
    /// Whether something is knocking at cell `ci` and can be answered
    pub fn cell_knocking(&self, ci: usize) -> bool {
        let d = &self.assembly.door;
        d.visit == Visit::Knocking && d.cell == Some(ci) && self.assembly.sealed == Some(ci)
    }

    /// Whether something is trying cell `ci`'s wheel (its answer, then its pulls)
    pub fn cell_wheel_tried(&self, ci: usize) -> bool {
        let d = &self.assembly.door;
        d.visit == Visit::Trying && d.cell == Some(ci)
    }

    /// Whether something is at cell `ci`'s door (knocking, or at the wheel):
    /// the door is not opened from inside while it is
    pub fn at_the_door(&self, ci: usize) -> bool {
        self.cell_knocking(ci) || self.cell_wheel_tried(ci)
    }

    /// Whether cell `ci`'s door gave and the fog is still up: its seal is
    /// broken, and the wheel will not shut it until the all-clear
    pub fn cell_door_gave(&self, ci: usize) -> bool {
        let d = &self.assembly.door;
        d.visit == Visit::Gave && d.cell == Some(ci) && self.assembly.up()
    }

    /// Where the wheel of cell `ci` is, world (its sounds come from there)
    fn cell_wheel_at(&self, ci: usize) -> Option<Vec3> {
        self.cell_doors.get(ci).map(|d| Vec3::new(d.at.x, d.at.y + 1.05, d.at.z))
    }

    /// What it chooses at cell `ci` this time: its patience, its wait before
    /// the wheel and how long it tries, from the floor's stream of its own
    fn visit_plan(&self, ci: usize) -> (u32, f64, f64) {
        let salt = VISIT_SALT ^ (u64::from(self.assembly.count) << 24) ^ ci as u64;
        let mut r = SplitMix64::new(self.current_seed, salt);
        let span = (PATIENCE.1 - PATIENCE.0 + 1) as f64;
        let patience = PATIENCE.0 + ((r.next() * span) as u32).min(PATIENCE.1 - PATIENCE.0);
        (patience, r.range(WHEEL_AFTER.0, WHEEL_AFTER.1), r.range(TRIES.0, TRIES.1))
    }

    /// It goes: the knocking stops, or it lets go of the wheel (a door it
    /// did not open has its wheel back where it rests shut)
    fn visit_goes(&mut self) {
        self.assembly.door.visit = Visit::Gone;
        if let Some(d) = self.assembly.door.cell.and_then(|ci| self.cell_doors.get_mut(ci)) {
            if d.target < 0.5 {
                d.turn_to = WHEEL_SHUT;
            }
        }
        self.director_note("cell:gone");
    }

    /// One step of what knocks at the cell (the self-assembly's): its knocks
    /// while you are sealed in and the fog is up, its going, its pulls at the
    /// wheel once answered, and the door giving unless you hold it
    pub fn update_at_the_door(&mut self, dt: f64) {
        let up = self.assembly.fog_rising_or_crest();
        let sealed = self.assembly.sealed;
        let visit = self.assembly.door.visit;
        // you opened the door, or the fog is going down: it goes
        if matches!(visit, Visit::Knocking | Visit::Trying) && (!up || sealed != self.assembly.door.cell) {
            self.visit_goes();
            return;
        }
        match visit {
            Visit::Away | Visit::Knocking => {
                let (Some(ci), true) = (sealed, up) else { return };
                self.assembly.knock_t -= dt;
                if self.assembly.knock_t > 0.0 {
                    return;
                }
                if visit == Visit::Away {
                    let (patience, wait, tries) = self.visit_plan(ci);
                    self.assembly.door = AtTheDoor { visit: Visit::Knocking, cell: Some(ci), patience, wait, tries, ..Default::default() };
                    self.director_note("cell:knocks");
                } else if self.assembly.door.knocks >= self.assembly.door.patience {
                    // nobody answered: the knocking stops
                    self.visit_goes();
                    return;
                }
                self.assembly.door.knocks += 1;
                self.assembly.knock_t = rand2(KNOCK.0, KNOCK.1);
                let at = self.cell_door_at(ci);
                if let (Some(ss), Some(at)) = (self.soundscape(), at) {
                    ss.knock(2 + (native_rnd() * 3.0) as i32, at.x, at.y, at.z, 0.45);
                }
            }
            Visit::Trying => self.try_the_wheel(dt),
            Visit::Gone | Visit::Gave => {}
        }
    }

    /// Answered, it tries the wheel: a pull at a time while you do not hold
    /// it, straining while you do; the door gives after `GIVE` seconds of
    /// turning, and after `tries` seconds it lets go and goes
    fn try_the_wheel(&mut self, dt: f64) {
        let Some(ci) = self.assembly.door.cell else { return };
        // E held, in the cell, keeps the wheel held (a press began it)
        if self.in_cell(ci) && self.held(Act::Interact) && self.assembly.door.grip > 0.0 {
            self.assembly.door.grip = GRIP;
        }
        let d = &mut self.assembly.door;
        let first = d.t < 0.0 && d.t + dt >= 0.0;
        d.t += dt;
        if d.t < 0.0 {
            return;
        }
        if d.t >= d.tries {
            self.visit_goes();
            return;
        }
        let held = d.grip > 0.0;
        d.grip = (d.grip - dt).max(0.0);
        if held {
            d.held_for += dt;
        } else {
            d.give += dt / GIVE;
        }
        d.pull_t -= dt;
        let pull = d.pull_t <= 0.0;
        if pull {
            d.pull_t = PULL;
        }
        let (gives, give) = (d.give >= 1.0, d.give);
        if first {
            self.director_note("cell:wheel");
        }
        if let Some(door) = self.cell_doors.get_mut(ci) {
            door.strain = door.strain.max(if held { 1.0 } else { 0.35 });
            // as far from shut as it has turned toward giving (it gives
            // where an open door's wheel rests)
            door.turn_to = WHEEL_SHUT + WHEEL_TURNS * give;
        }
        let at = self.cell_wheel_at(ci);
        if pull && self.ac.is_some() {
            if let Some(p) = at {
                self.play_from(p.x, p.y, p.z, 3.0, 1.0, 0.0, move |g| g.sfx_wheel_tried(held));
            }
        }
        if gives {
            self.cell_door_gives(ci);
        }
    }

    /// The door of cell `ci` gives: the bolts drawn, the leaf swung in, the
    /// seal broken until the all-clear; the fog comes in
    fn cell_door_gives(&mut self, ci: usize) {
        self.assembly.door.visit = Visit::Gave;
        self.assembly.sealed = None;
        if let Some(d) = self.cell_doors.get_mut(ci) {
            d.target = GAVE_OPEN;
            d.strain = 0.0;
        }
        self.director_note("cell:gave");
        if let (Some(p), true) = (self.cell_wheel_at(ci), self.ac.is_some()) {
            self.play_from(p.x, p.y, p.z, 4.0, 1.0, 0.0, |g| g.sfx_seal_gives());
        }
    }

    /// Knock back at cell `ci`'s door from inside (`[E] KNOCK`): your three
    /// raps on the leaf; if something is knocking, it is answered, and it
    /// will try the wheel
    pub fn answer_the_door(&mut self, ci: usize) {
        let Some(at) = self.cell_door_at(ci) else { return };
        let gaps = self.draw_raps();
        if self.ac.is_some() {
            self.play_from(at.x, at.y, at.z, 3.0, 1.0, 0.0, move |g| g.sfx_knock(&gaps, false));
        }
        if !self.cell_knocking(ci) {
            return;
        }
        let d = &mut self.assembly.door;
        d.visit = Visit::Trying;
        d.t = -d.wait;
        d.give = 0.0;
        d.grip = 0.0;
        d.pull_t = 0.0;
        self.director_note("cell:answered");
    }

    /// Hold cell `ci`'s wheel against it (`[E] HOLD THE WHEEL`): for `GRIP`
    /// seconds, and for as long as E is held after
    pub fn hold_the_wheel(&mut self, ci: usize) {
        if self.cell_wheel_tried(ci) {
            self.assembly.door.grip = GRIP;
        }
    }
}
