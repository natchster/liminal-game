//! What knocks at your cell (`docs/MEGASTRUCTURE.md`, G6). Sealed in a cell
//! while the self-assembly's fog rises or stands at its crest, something
//! knocks at the door every few seconds (`assembly::KNOCK`). What you do is
//! a choice:
//!
//! - **Stay silent**, and after a few knocks the knocking stops: it goes,
//!   heard going from the door, and does not come back while the fog is up.
//! - **Answer it** (`[E] KNOCK`, inside the cell at its door: your own three
//!   raps, the knock of `world::knock`), and it knocks no more (its raps
//!   still to come hushed), and a moment later it tries the wheel: the wheel turns from outside, a pull at a time, heard and seen
//!   at the door. Its first pull takes up the wheel's play (`SLACK`, `PLAY`)
//!   before it turns the screw. Unless you hold it (`[E] HOLD THE WHEEL`: a
//!   press holds it for `GRIP`, and E held keeps it held, so held or pressed
//!   again and again), the door gives after `GIVE` seconds of turning past
//!   the play: the bolts drawn, the leaf swung in, the cell no longer
//!   sealed, and the fog's harm (`assembly::DRAIN`, `WADE_DRAIN`, `LAST`)
//!   reaches you inside as it does outside wherever its surface is over you.
//!   Held, the wheel strains and the leaf shudders in its frame, and after a
//!   while it lets go (heard: the spokes slipping from its hold) and goes.
//!
//! What the screen shows follows what is heard and seen at the door. The
//! hold shows from the wheel's first pull, not from your answer: between
//! the two nothing at the door is yours to use, and E there holds nothing
//! (only the early E of `session::input`, `INTERACT_BUFFER`, lands on the
//! hold if the pull comes within it, as it lands on anything). A hand on
//! the wheel while its first pull is still in the play holds it before it
//! has turned the screw at all (the wheel moved only through its play):
//! `SLACK` is longer than a reaction to the squeal.
//!
//! However it goes (unanswered, let go of the wheel, the fog starting down,
//! or the door opened under it), it is heard going from the door: its hold
//! on the wheel slipping if it had it, and then not steps (nothing walks
//! the Building) but something wet drawn off the door and away, in breaths
//! (`GOING_BREATHS`), each fainter and lower than the last. Knocking, it
//! goes only once its last knock has rung out (`KNOCK_RINGS`): until then
//! it is at the door as it was (`leaving`), `[E] KNOCK` shown and its knock
//! to be answered, so the screen shows its going no sooner than it is
//! heard. It is at the door until it has gone (`Going`), and the door is
//! not opened from inside until then. The knocking itself carries above
//! the cell's own lamp (`Soundscape::knock`: the knuckle's click and the
//! leaf's ring), so `[E] KNOCK` shows what is heard too.
//!
//! A door that gave stays open while the fog is up (its seal is broken: the
//! wheel will not shut it until the all-clear). A door left open, or opened
//! while it is up, lets the fog in the same way (`assembly::in_fog` exempts
//! only a sealed cell). While something is at the door, the door is not
//! opened from inside: its verb is the knock, then the hold once the wheel
//! turns; between them, and while it goes, none.
//!
//! What it chooses (how many knocks it gives unanswered before it goes, how
//! long after your answer it starts on the wheel, how long it tries, and
//! the pace of its going and its sound) comes from a stream of its
//! own, seeded from the floor's seed with a salt, the assembly's count and
//! the cell (`VISIT_SALT`): no draw of the build, and none of the per-frame
//! stream at a floor's start. The knocks' cadence and your raps' rhythm are
//! drawn from the per-frame stream when they come, as before. Its sounds
//! play from the door (`play_from`).
use crate::prelude::*;
use crate::session::input::Act;
use crate::world::assembly::{KNOCK, WHEEL_SHUT};

/// The salt of the stream what knocks at a cell chooses from
pub const VISIT_SALT: u64 = 0xCE11_D002_0000_0006;
/// How many knocks it gives unanswered before it goes (both counted)
pub const PATIENCE: (u32, u32) = (2, 4);
/// From your answer to the wheel's first turn, s
pub const WHEEL_AFTER: (f64, f64) = (0.9, 1.6);
/// How long it tries the wheel, s (longer than `SLACK` and `GIVE`: let go,
/// it gives)
pub const TRIES: (f64, f64) = (6.0, 10.0);
/// The wheel's play: its first seconds of turning, not held, take up this
/// much before it turns the screw toward giving. A hand on it within them
/// holds it before the door begins to open: longer than a reaction to the
/// first pull's squeal (a simple reaction to a sound or a light is
/// 0.15–0.3 s), with a frame or two to spare (`tests/assembly.rs`,
/// `held_is_safe`)
pub const SLACK: f64 = 0.4;
/// How far the wheel turns in its play, radians (seen: the wheel moves at
/// the first pull, before it turns toward giving)
pub const PLAY: f64 = 0.2;
/// Seconds of turning past the play, not held, that make the door give
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
/// Its going from the door: how many breaths of it (the last 9 dB under the
/// first), and their pace, s (from its own stream)
pub const GOING_BREATHS: usize = 4;
pub const BREATH: (f64, f64) = (0.9, 1.2);
/// Its first breath going, s after it let go of the wheel, and at the
/// least after its last knock's raps are over
pub const GOING_AFTER: f64 = 0.9;
/// A knock's raps at the cell (two to four, the k-th at k times 1–2 s:
/// `Soundscape::knock`) are over this long after it began, s
pub const KNOCK_RINGS: f64 = 6.3;
/// From its last breath to its having gone, s (the breath dies away)
pub const GOING_TAIL: f64 = 1.0;

/// Where what knocks at the cell is
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Visit {
    /// nothing at the door yet this time
    #[default]
    Away,
    /// knocking now and then: it can be answered
    Knocking,
    /// answered: trying the wheel (from its first pull, `t` 0 and on)
    Trying,
    /// going (unanswered, let go of the wheel, the fog starting down, or the
    /// door opened): heard going from the door until it has gone
    Going,
    /// it went: nothing more this time
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
    /// how much of the wheel's play its pulls have taken up, 0..1
    pub play: f64,
    /// how far it has turned the wheel toward the door giving, 0..1
    pub give: f64,
    /// seconds your hold on the wheel lasts
    pub grip: f64,
    /// seconds of it held, all told (for the log and the tests)
    pub held_for: f64,
    /// seconds to the next pull's sound
    pub pull_t: f64,
    /// seconds since its last knock began
    pub since_knock: f64,
    /// knocking no more, seconds until its last knock has rung out and it
    /// is heard going (still at the door, its knock still to be answered)
    pub leaving: Option<f64>,
    /// going: seconds until it has gone
    pub going: f64,
}

impl Game {
    /// Whether something is knocking at cell `ci` and can be answered
    pub fn cell_knocking(&self, ci: usize) -> bool {
        let d = &self.assembly.door;
        d.visit == Visit::Knocking && d.cell == Some(ci) && self.assembly.sealed == Some(ci)
    }

    /// Whether what knocks at cell `ci` was answered and has the wheel or
    /// soon will (from your answer: the moment before its first pull, then
    /// its pulls)
    pub fn cell_answered(&self, ci: usize) -> bool {
        let d = &self.assembly.door;
        d.visit == Visit::Trying && d.cell == Some(ci)
    }

    /// Whether cell `ci`'s wheel is being tried: from its first pull, heard
    /// (its squeal) and seen (the wheel turning in its play, the leaf
    /// straining). The hold's gate: not from your answer, or the screen
    /// would warn before the place does
    pub fn cell_wheel_tried(&self, ci: usize) -> bool {
        self.cell_answered(ci) && self.assembly.door.t >= 0.0
    }

    /// Whether what was at cell `ci`'s door is going (heard going)
    pub fn cell_visit_going(&self, ci: usize) -> bool {
        let d = &self.assembly.door;
        d.visit == Visit::Going && d.cell == Some(ci)
    }

    /// Whether something is at cell `ci`'s door (knocking, answered, or
    /// going until it has gone): the door is not opened from inside while
    /// it is
    pub fn at_the_door(&self, ci: usize) -> bool {
        self.cell_knocking(ci) || self.cell_answered(ci) || self.cell_visit_going(ci)
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

    /// The stream of its own that what knocks at cell `ci` chooses from this
    /// time (`part` 0: its plan; others: what it chooses as it goes)
    fn visit_stream(&self, ci: usize, part: u64) -> SplitMix64 {
        let salt = VISIT_SALT ^ (u64::from(self.assembly.count) << 24) ^ ci as u64 ^ (part << 56);
        SplitMix64::new(self.current_seed, salt)
    }

    /// What it chooses at cell `ci` this time: its patience, its wait before
    /// the wheel and how long it tries, from the floor's stream of its own
    fn visit_plan(&self, ci: usize) -> (u32, f64, f64) {
        let mut r = self.visit_stream(ci, 0);
        let span = (PATIENCE.1 - PATIENCE.0 + 1) as f64;
        let patience = PATIENCE.0 + ((r.next() * span) as u32).min(PATIENCE.1 - PATIENCE.0);
        (patience, r.range(WHEEL_AFTER.0, WHEEL_AFTER.1), r.range(TRIES.0, TRIES.1))
    }

    /// It goes. Knocking, it goes once its last knock has rung out: until
    /// then it is at the door as it was, knocking no more, and its knock can
    /// still be answered (`leaving`; the screen shows its going no sooner
    /// than it is heard). Then, or at once if it was answered (its raps
    /// were hushed), `visit_heard_going`
    fn visit_goes(&mut self) {
        let d = &mut self.assembly.door;
        if d.visit == Visit::Knocking {
            if d.leaving.is_none() {
                d.leaving = Some(GOING_AFTER.max(KNOCK_RINGS - d.since_knock));
            }
            return;
        }
        self.visit_heard_going();
    }

    /// It is heard going: if it had the wheel, its hold slips from it (a
    /// wheel it had turned turns back to where it rests on a shut door),
    /// and after that, or at once if it was knocking (its last knock rang
    /// out), something wet is drawn off the door and away, breath by
    /// breath. It is at the door until it has gone (`Visit::Going`, then
    /// `Visit::Gone`). The pace of its going and its sound come from its
    /// own stream: no per-frame draw
    fn visit_heard_going(&mut self) {
        let d = &self.assembly.door;
        let Some(ci) = d.cell else {
            self.assembly.door.visit = Visit::Gone;
            return;
        };
        // it had the wheel: its first pull came
        let let_go = d.visit == Visit::Trying && d.t >= 0.0;
        // (back through the screw: not a wheel held within its play)
        let back = let_go && d.give > 0.0;
        // (a pause after it let go or was answered; knocking, it has waited)
        let lead = if d.visit == Visit::Knocking { 0.0 } else { GOING_AFTER };
        let mut r = self.visit_stream(ci, 1);
        let pace = r.range(BREATH.0, BREATH.1);
        // (where each noise starts in its loop: three for the wheel, two a breath)
        let from: Vec<f64> = (0..3 + GOING_BREATHS * 2).map(|_| r.next() * 0.95).collect();
        // (its raps have rung out, or an answer took them: their gains
        // let go, so the knocks' voices are freed, `Engine::collect`)
        self.audio.cell_knocks.clear();
        let d = &mut self.assembly.door;
        d.visit = Visit::Going;
        d.leaving = None;
        d.going = lead + (GOING_BREATHS - 1) as f64 * pace + GOING_TAIL;
        if let Some(door) = self.cell_doors.get_mut(ci) {
            if door.target < 0.5 {
                door.turn_to = WHEEL_SHUT;
            }
        }
        self.director_note("cell:going");
        if self.ac.is_none() {
            return;
        }
        if let (true, Some(p)) = (let_go, self.cell_wheel_at(ci)) {
            let f = [from[0], from[1], from[2]];
            self.play_from(p.x, p.y, p.z, 3.0, 1.0, 0.0, move |g| g.sfx_wheel_let_go(back, f));
        }
        if let Some(at) = self.cell_door_at(ci) {
            self.play_from(at.x, at.y, at.z, 3.0, 1.0, 0.0, move |g| g.sfx_going(lead, pace, &from[3..]));
        }
    }

    /// One step of what knocks at the cell (the self-assembly's): its knocks
    /// while you are sealed in and the fog is up, its going, its pulls at the
    /// wheel once answered, and the door giving unless you hold it
    pub fn update_at_the_door(&mut self, dt: f64) {
        let up = self.assembly.fog_rising_or_crest();
        let sealed = self.assembly.sealed;
        let visit = self.assembly.door.visit;
        self.assembly.door.since_knock += dt;
        // knocking no more, it goes once its last knock has rung out
        if let (Visit::Knocking, Some(left)) = (visit, self.assembly.door.leaving) {
            self.assembly.door.leaving = Some(left - dt);
            if left - dt <= 0.0 {
                self.visit_heard_going();
            }
            return;
        }
        // the door opened under it, or the fog starting down: it goes
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
                    self.audio.cell_knocks.clear();
                    self.director_note("cell:knocks");
                    // your asides: something at the door of the cell you are in
                    self.aside(crate::audio::asides::Use::Held, "knock");
                } else if self.assembly.door.knocks >= self.assembly.door.patience {
                    // nobody answered: the knocking stops
                    self.visit_goes();
                    return;
                }
                self.assembly.door.knocks += 1;
                self.assembly.door.since_knock = 0.0;
                self.assembly.knock_t = rand2(KNOCK.0, KNOCK.1);
                let at = self.cell_door_at(ci);
                if let (Some(ss), Some(at)) = (self.soundscape(), at) {
                    let hush = ss.knock(2 + (native_rnd() * 3.0) as i32, at.x, at.y, at.z, 0.45);
                    self.audio.cell_knocks.push(hush);
                }
            }
            Visit::Trying => self.try_the_wheel(dt),
            // going: gone once its going has died away
            Visit::Going => {
                self.assembly.door.going -= dt;
                if self.assembly.door.going <= 0.0 {
                    self.assembly.door.visit = Visit::Gone;
                    self.director_note("cell:gone");
                }
            }
            Visit::Gone | Visit::Gave => {}
        }
    }

    /// Answered, it tries the wheel: a pull at a time while you do not hold
    /// it, straining while you do; its first `SLACK` seconds of turning take
    /// up the wheel's play, the door gives after `GIVE` seconds more, and
    /// after `tries` seconds it lets go and goes
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
            // the play taken up first, then the screw turned
            let left = SLACK * (1.0 - d.play);
            if left >= dt {
                d.play += dt / SLACK;
            } else {
                d.play = 1.0;
                d.give += (dt - left) / GIVE;
            }
        }
        d.pull_t -= dt;
        let pull = d.pull_t <= 0.0;
        if pull {
            d.pull_t = PULL;
        }
        let (gives, give, play) = (d.give >= 1.0, d.give.min(1.0), d.play);
        if first {
            self.director_note("cell:wheel");
        }
        if let Some(door) = self.cell_doors.get_mut(ci) {
            door.strain = door.strain.max(if held { 1.0 } else { 0.35 });
            // as far from shut as its play and its turning toward giving
            // (it gives where an open door's wheel rests)
            door.turn_to = WHEEL_SHUT + PLAY * play + (WHEEL_TURNS - PLAY) * give;
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
        // it heard you: it knocks no more (its raps still to come hushed,
        // once the one sounding now is done)
        let knocks = std::mem::take(&mut self.audio.cell_knocks);
        if let Some(ac) = &self.ac {
            let t = ac.current_time() + 0.12;
            for k in &knocks {
                k.gain().set_target_at_time(0.0, t, 0.03);
            }
        }
        let d = &mut self.assembly.door;
        d.visit = Visit::Trying;
        d.leaving = None;
        d.t = -d.wait;
        d.play = 0.0;
        d.give = 0.0;
        d.grip = 0.0;
        d.pull_t = 0.0;
        self.director_note("cell:answered");
    }

    /// Hold cell `ci`'s wheel against it (`[E] HOLD THE WHEEL`): for `GRIP`
    /// seconds, and for as long as E is held after. Only once the wheel is
    /// tried (its first pull): before that there is nothing to hold
    pub fn hold_the_wheel(&mut self, ci: usize) {
        if self.cell_wheel_tried(ci) {
            self.assembly.door.grip = GRIP;
        }
    }
}
