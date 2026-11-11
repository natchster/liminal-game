//! Your asides (`docs/VOICES.md`, *Your asides*): what you say to yourself,
//! under your breath, on arriving, at a sudden noise, at what comes and at
//! what the place does back. A director of their own decides, after the
//! pacing director's pattern and Valve's dynamic dialog
//! (`docs/research/inner-voice.md`): an event (a use, a subject) is weighed
//! by the run's seed, once for its scope; an answered one waits for its
//! moment and is said then, or is owed to the next event like it; its line
//! is dealt from the subject's and the use's general lines, never one said
//! before in the run.
//!
//! The rules (each held by a test here, or named beside it):
//! 1. react, never explain (the lines themselves: `voice::tests`)
//! 2. once is a reaction, twice a habit (`key`, `Scope`, `chance`, `choose`)
//! 3. different every run (`temperament`, `roll`, `deck_order`: the run's
//!    seed on streams of their own; `AsideMemory` kept with the run, saved
//!    as each line is said)
//! 4. hold your breath (`Game::holding_breath`, the pacing director's;
//!    `Use::held_by`)
//! 5. one at a time, and room between (`Game::aside_held`, `Use::moment`)
//! 6. heard by nothing but you (nothing here reaches a creature, a draw of
//!    the per-frame stream, the light or the screen)
//! 7. only for a person playing (`AWAY`)
//!
//! Its times are its own clock (`Asides::clock`): time played, which a
//! pause or a menu does not run.

use super::voice::Line;
use crate::base::game::Game;
use crate::base::numeric::hypot2;
use crate::base::rng::SplitMix64;

/// Whose lines your asides are: you, under your breath
pub const PERSONA: &str = "you_aside";
/// The level of an aside in the mix: its WAV is at -20 LUFS (the cry's at
/// -16), and it is played lower again, under your breath
pub const ASIDE_GAIN: f64 = 0.7;
/// At least this long between two asides, s
pub const GAP: f64 = 20.0;
/// ... or this, for an urgent one after a lesser (`Use::urgent`), s
pub const GAP_URGENT: f64 = 8.0;
/// Neither moved, nor looked round, nor touched a key for this long, s:
/// nobody is playing (90: long enough to sit sealed in a cell, watching
/// the door; review A)
pub const AWAY: f64 = 90.0;
/// You have moved this far, or turned this much, since: you are playing
const MOVED: f64 = 0.5;
const TURNED: f64 = 0.05;
/// How far from where a floor began you walk before its arrival is weighed, m
pub const ARRIVAL_WALK: f64 = 2.0;
/// ... and not before this long after it began, s: once its card (4 s) and
/// its line (from 1.7 s, for 7 s: `scp_arrival`) have gone, so a floor is
/// never described twice at once (`lore-keeper`, A4)
pub const ARRIVAL_AFTER: f64 = 9.0;
/// ... and not after this long: a floor's arrival is its first minute's
/// played (walked later, its moment has gone), s
pub const ARRIVAL_LATEST: f64 = 60.0;
/// An answered reaction that could not be said (held, late, a better one
/// waiting) is owed to the next event like it, this many times at most
pub const OWED_TRIES: u32 = 3;
/// What each line already said of a use leaves of its chance, whatever
/// its subject (the subject's own decay is `Use::decay`)
pub const USE_FADE: f64 = 0.85;
/// How long a voice line of yours takes to fade when it is cut off, s
pub const CUT_FADE: f64 = 0.15;
/// The most events one run's log keeps
pub const LOG_CAP: usize = 2048;
/// Your nerve is near its end under this share of it (where its heartbeat
/// and whispers begin, `update_sanity_effects`), and back from it over the
/// other
pub const NERVE_LOW: (f64, f64) = (0.28, 0.4);

const ROLL_SALT: u64 = 0xA51D_E5C0_7E11_0B5D;
const TEMPER_SALT: u64 = 0x7E39_E2A1_5F0C_C0DE;
const DECK_SALT: u64 = 0xDEC4_0F11_9E5A_31D7;

/// What an aside answers: the game's events, each one of the line files' uses
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Use {
    /// The first steps on a floor
    Arrive,
    /// A sudden sound near you
    Startle,
    /// The first sight of a kind of creature
    Sighted,
    /// The first touch of one throws you down
    Knocked,
    /// The relief after a close call
    Aftermath,
    /// The self-assembly's siren
    Siren,
    /// Shut in a cell while it is up: a knock, the all-clear
    Held,
    /// A resident's door answers
    Answered,
    /// The place does something back
    Uncanny,
    /// Someone else's camp
    Found,
    /// Your nerve near its end
    Nerve,
    /// The lamp's battery runs out
    Dark,
}

pub const USES: [Use; 12] = [
    Use::Arrive,
    Use::Startle,
    Use::Sighted,
    Use::Knocked,
    Use::Aftermath,
    Use::Siren,
    Use::Held,
    Use::Answered,
    Use::Uncanny,
    Use::Found,
    Use::Nerve,
    Use::Dark,
];

/// How long an event of a use is remembered as weighed (rule 2): every
/// kind of event has its word at most once a run (the user's ask: never a
/// voice on a later event of the same kind); a floor's arrival once a floor
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Scope {
    /// once a run for each subject (a kind of sound, of creature, of thing;
    /// the blow, the relief, your nerve, the lamp)
    Run,
    /// once a floor for each subject (an arrival)
    Level,
}

impl Use {
    /// Its name in the line files (`use`)
    pub fn name(self) -> &'static str {
        match self {
            Use::Arrive => "arrive",
            Use::Startle => "startle",
            Use::Sighted => "sighted",
            Use::Knocked => "knocked",
            Use::Aftermath => "aftermath",
            Use::Siren => "siren",
            Use::Held => "held",
            Use::Answered => "answered",
            Use::Uncanny => "uncanny",
            Use::Found => "found",
            Use::Nerve => "nerve",
            Use::Dark => "dark",
        }
    }
    pub fn from_name(s: &str) -> Option<Use> {
        USES.iter().copied().find(|u| u.name() == s)
    }
    /// How long an event of it is remembered as weighed
    pub fn scope(self) -> Scope {
        match self {
            Use::Arrive => Scope::Level,
            _ => Scope::Run,
        }
    }
    /// The subjects the game posts for it (`Game::aside` refuses any other:
    /// a misspelt subject would otherwise be a key of its own and fall back
    /// to the general lines); the line files' subjects are held to these
    /// (`voice::tests`). Each use's are added as the game comes to post it.
    pub fn subjects(self) -> &'static [&'static str] {
        match self {
            // (and the Stairwell, entered from its floor; not the Loop, where
            // a word would point at its door)
            Use::Arrive => &[
                "well",
                "stair_halls",
                "works",
                "lobby",
                "rooms",
                "deep",
                "showroom",
                "house",
                "stairwell_floor",
                "loop_floor",
                "stairwell",
            ],
            // (the steps of what is coming are a kind of their own: a word for
            // the place's steps does not spend theirs)
            Use::Startle => &["bang", "scream", "steps", "laugh", "moan", "creature_steps"],
            // the kinds a first sight can be (never the Watcher: at a glance
            // it is one of the sheets, and a word would say which)
            Use::Sighted => &["hound", "smiler", "corner", "reader", "glass"],
            // the self-assembly's siren: the first of a run (once a run, as
            // every kind)
            Use::Siren => &["first"],
            // sealed in a cell while it is up: a knock at the door, the all-clear
            Use::Held => &["knock", "clear"],
            // a resident's door answers your knock: a thing pushed out, a note
            Use::Answered => &["hatch", "note"],
            // the place does something back: a marked door knocks back, the
            // figure across the void turns its back; the hole's touch, a dead
            // machine's flap and the print on its glass, a lift's car going
            // by, a candle's two notes, a window's pane ticking
            Use::Uncanny => &["knock_back", "figure", "hole", "flap", "lift", "candle", "pane"],
            // someone else's camp, the first time you come near one
            Use::Found => &["camp"],
            _ => &[],
        }
    }
    /// The chance its first event is answered, before the run's temperament
    /// and what has been said (`chance`)
    pub fn chance(self) -> f64 {
        match self {
            Use::Arrive | Use::Knocked | Use::Siren => 0.9,
            Use::Sighted | Use::Uncanny => 0.85,
            Use::Aftermath | Use::Answered | Use::Found | Use::Dark => 0.8,
            Use::Startle | Use::Held => 0.7,
            Use::Nerve => 0.6,
        }
    }
    /// What each line already said about the same subject leaves of the
    /// chance: a place's arrival fades slowly (the rooms are many floors),
    /// the rest by half (rule 2)
    pub fn decay(self) -> f64 {
        match self {
            Use::Arrive => 0.7,
            _ => 0.5,
        }
    }
    /// A better one waiting replaces a lesser one (higher is better)
    pub fn priority(self) -> u8 {
        match self {
            Use::Knocked => 5,
            Use::Sighted | Use::Uncanny => 4,
            Use::Aftermath | Use::Startle | Use::Siren | Use::Held => 3,
            Use::Answered | Use::Found | Use::Dark => 2,
            Use::Arrive | Use::Nerve => 1,
        }
    }
    /// Said `GAP_URGENT` after a lesser aside, not `GAP`
    pub fn urgent(self) -> bool {
        matches!(self, Use::Knocked | Use::Sighted | Use::Uncanny)
    }
    /// A blow's grunt is not chosen: it cuts in over a line of yours that
    /// is playing, and keeps no gap
    pub fn cuts_in(self) -> bool {
        matches!(self, Use::Knocked)
    }
    /// Whether a reason to hold your breath (`Game::holding_breath`) holds
    /// it back: a blow's grunt nothing holds; the siren and a knock at your
    /// cell are what the place's hush is about, and only your cry holds them
    pub fn held_by(self, why: &str) -> bool {
        match self {
            Use::Knocked => false,
            Use::Siren | Use::Held => why != "hushed",
            _ => true,
        }
    }
    /// Its moment: the least and most seconds from the event to the line,
    /// and how long after the event it may still be said (rule 5)
    pub fn moment(self) -> (f64, f64, f64) {
        match self {
            Use::Startle => (0.35, 0.9, 2.0),
            Use::Sighted => (0.4, 1.0, 2.5),
            Use::Knocked => (1.2, 1.8, 4.0),
            // the relief: said once the breath is let go, while it lasts
            Use::Aftermath => (1.5, 3.5, 14.0),
            Use::Arrive => (1.0, 3.0, 15.0),
            Use::Nerve | Use::Dark => (0.8, 2.0, 8.0),
            _ => (0.5, 1.2, 4.0),
        }
    }
}

/// What a run remembers of your asides (saved with it, and as each line is
/// said): what was said and about what, the events weighed and owed, how
/// many of each use there have been
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AsideMemory {
    /// The lines said this run, in order (none twice: rule 2)
    pub said: Vec<String>,
    /// How many lines were said about each `use:subject`
    pub about: std::collections::BTreeMap<String, u32>,
    /// The keys done with (`key`): said, or weighed and not answered, or
    /// owed too many times; a later event like it is not weighed (rule 2)
    pub weighed: Vec<String>,
    /// The keys answered but not yet said (held, late, a better one
    /// waiting), and how many tries: the next event like it is answered
    /// again without a new roll
    pub owed: std::collections::BTreeMap<String, u32>,
    /// How many events of each use there have been this run: each one's
    /// roll is its own (`roll`)
    pub events: std::collections::BTreeMap<String, u32>,
}

impl AsideMemory {
    /// As the save keeps it
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({ "said": self.said, "about": self.about, "weighed": self.weighed, "owed": self.owed, "events": self.events })
    }
    /// From the save (anything missing or malformed: as a new run's)
    pub fn from_json(v: Option<&serde_json::Value>) -> AsideMemory {
        let Some(v) = v else { return AsideMemory::default() };
        let strs = |k: &str| -> Vec<String> {
            v.get(k).and_then(|a| a.as_array()).map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default()
        };
        let counts = |k: &str, known: &dyn Fn(&str) -> bool| -> std::collections::BTreeMap<String, u32> {
            v.get(k)
                .and_then(|h| h.as_object())
                .map(|h| {
                    h.iter()
                        .filter(|(k, _)| known(k))
                        .filter_map(|(k, n)| n.as_u64().map(|n| (k.clone(), n.min(u32::MAX as u64) as u32)))
                        .collect()
                })
                .unwrap_or_default()
        };
        let of_a_use = |k: &str| k.split(':').next().and_then(Use::from_name).is_some();
        AsideMemory {
            said: strs("said"),
            about: counts("about", &of_a_use),
            weighed: strs("weighed"),
            owed: counts("owed", &of_a_use),
            events: counts("events", &|k| Use::from_name(k).is_some()),
        }
    }
    /// How many lines of `u` were said, about anything
    pub fn said_of(&self, u: Use) -> u32 {
        let p = format!("{}:", u.name());
        self.about.iter().filter(|(k, _)| k.starts_with(&p)).map(|(_, n)| n).sum()
    }
}

/// A reaction waiting for its moment
#[derive(Clone, Debug, PartialEq)]
pub struct Pending {
    pub use_: Use,
    pub subject: String,
    /// Its once-a-run (or a-floor) key, owed until it is said
    pub key: String,
    /// Said from this time (the director's clock, s) ...
    pub at: f64,
    /// ... and not after this
    pub until: f64,
}

/// The voice director's state (`Game::asides`)
#[derive(Clone, Debug)]
pub struct Asides {
    /// What the run remembers (saved with it)
    pub memory: AsideMemory,
    pub pending: Option<Pending>,
    /// The director's clock: time played (it runs only while the director
    /// does, never in a pause or a menu), s
    pub clock: f64,
    /// When the last aside was said (the clock), and its use's priority
    pub last: Option<(f64, u8)>,
    /// Until when a line of your voice (an aside or your cry) still sounds
    /// (game time, `perf_t`: what the speakers do), set by `play_voice_line`
    pub voice_until: f64,
    /// When you last moved, looked round or touched a key (the clock), and
    /// where and how you stood then
    pub active_at: f64,
    pub anchor: Option<(f64, f64, f64, f64)>,
    /// This floor's arrival, waiting for your first steps: where it began,
    /// what it is said about, and when it began (the clock)
    pub arrival: Option<(f64, f64, String, f64)>,
    /// Which floor (or Stairwell, or Loop) this is, counted: a reaction
    /// posted later (`aside_later`) is let go if it is no longer
    pub floor: u32,
    /// When it began (the clock): nothing but a blow's grunt while its card
    /// and line are up (`ARRIVAL_AFTER`)
    pub floor_began: f64,
    /// Your nerve is near its end (`NERVE_LOW`): an event as it gets there
    pub nerve_low: bool,
    /// What happened, by the clock: `wait:`, `pass:`, `skip:`, `drop:`,
    /// `say:` (use, subject, and the line or the reason). For tests and run
    /// analysis; capped at `LOG_CAP`
    pub log: Vec<(f64, String)>,
    /// Lines in place of the game's index (tests)
    pub lines: Option<Vec<Line>>,
}

impl Default for Asides {
    fn default() -> Self {
        Asides {
            memory: AsideMemory::default(),
            pending: None,
            clock: 0.0,
            last: None,
            voice_until: 0.0,
            // nobody has played yet
            active_at: f64::NEG_INFINITY,
            anchor: None,
            arrival: None,
            floor: 0,
            floor_began: f64::NEG_INFINITY,
            nerve_low: false,
            log: vec![],
            lines: None,
        }
    }
}

/// How much you talk this run: every chance times this, 0.65-1.0, from the
/// run's seed (rule 3)
pub fn temperament(run_seed: u32) -> f64 {
    SplitMix64::new(run_seed, TEMPER_SALT).range(0.65, 1.0)
}

/// The chance an event of `u` is answered: its own, times the run's
/// temperament, times its decay for each line already said about the same
/// subject, and `USE_FADE` for each line of the use said about anything
/// (rule 2: a new place, late in a run, still has its word)
pub fn chance(u: Use, run_seed: u32, said_about: u32, said_of_use: u32) -> f64 {
    u.chance() * temperament(run_seed) * u.decay().powi(said_about as i32) * USE_FADE.powi(said_of_use as i32)
}

/// SplitMix64's finaliser: a salt mixed through, so that runs whose seeds
/// differ in a few bits do not share rolls (review A: `seed ^ salt ^ n`
/// gave `roll(u, s, n) == roll(u, s ^ k, n ^ k)`)
fn mix(mut z: u64) -> u64 {
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// The `n`th event of `u` in the run: its roll (answered when under the
/// chance) and where in its moment it is said (0-1), from a stream of its
/// own: the run's seed, the use, the count (rule 3; never the per-frame
/// stream, `CLAUDE.md` invariant 1)
pub fn roll(u: Use, run_seed: u32, n: u32) -> (f64, f64) {
    let mut r = SplitMix64::new(run_seed, mix(ROLL_SALT ^ ((u as u64 + 1) << 40) ^ n as u64));
    (r.next(), r.next())
}

/// A line's place in the run's order: each run deals the lines afresh (rule 3)
fn deck_order(run_seed: u32, id: &str) -> u64 {
    // FNV-1a of the id: the same id, the same place in a run
    let h = id.bytes().fold(0xCBF2_9CE4_8422_2325u64, |h, b| (h ^ b as u64).wrapping_mul(0x0100_0000_01B3));
    (SplitMix64::new(run_seed, DECK_SALT ^ h).next() * (1u64 << 53) as f64) as u64
}

/// The key an event of `u` is remembered by, for its scope: once a run for
/// a subject (or a use with none), once a floor for an arrival (rule 2)
pub fn key(u: Use, subject: &str, level: i32) -> String {
    match u.scope() {
        Scope::Run => format!("{}:{subject}", u.name()),
        Scope::Level => format!("{}:{subject}:{level}", u.name()),
    }
}

/// How long a level's signature sounds, s: a startle's word waits for it
/// (a slam's crack and rumble, a scream's fall, four paces, five blips, a
/// moan's fall)
pub fn startle_after(sig: &str) -> f64 {
    match sig {
        "doorSlam" => 0.9,
        "thud" | "clank" => 0.5,
        "distantScream" => 1.6,
        "distantFootsteps" => 2.6,
        "laugh" => 0.7,
        "moan" => 1.8,
        _ => 0.5,
    }
}

/// The line an event is answered with: among `all`, the persona's lines
/// of the use for the level and not said this run, the subject's own and
/// the use's general ones dealt together, the first in the run's order (so
/// a kind is answered differently from run to run, and no general line is
/// left unsaid behind a subject's own: review A); none when the use is
/// spent (rules 2, 3, 5)
pub fn choose(all: &[Line], u: Use, subject: &str, level: i32, run_seed: u32, said: &[String]) -> Option<Line> {
    all.iter()
        .filter(|l| l.persona == PERSONA && l.use_ == u.name() && l.first <= level && level <= l.last && !said.contains(&l.id))
        .filter(|l| l.subject.is_empty() || (!subject.is_empty() && l.subject == subject))
        .min_by_key(|l| deck_order(run_seed, &l.id))
        .cloned()
}

/// The kind of sudden sound a level's signature is, for a startle (its
/// subject): a bang, a scream, steps, a laugh, a moan; the rest (a drip,
/// paper, a radio, a phone's tone, a chime, a piano's one note) startle
/// nobody
pub fn startle_of(sig: &str) -> Option<&'static str> {
    match sig {
        "doorSlam" | "thud" | "clank" => Some("bang"),
        "distantScream" => Some("scream"),
        "distantFootsteps" => Some("steps"),
        "laugh" => Some("laugh"),
        "moan" => Some("moan"),
        _ => None,
    }
}

impl Game {
    fn aside_note(&mut self, what: String) {
        if self.asides.log.len() < LOG_CAP {
            let t = self.asides.clock;
            self.asides.log.push((t, what));
        }
    }

    fn aside_lines(&self) -> &[Line] {
        match &self.asides.lines {
            Some(l) => l,
            None => super::voice::index(),
        }
    }

    /// An event your voice may answer (rules 2, 3). An event with no line
    /// left for it here is not weighed (nothing plays, and nothing is
    /// spent); one already done with for its scope is not either; one owed
    /// is answered again without a new roll; the rest are rolled by the
    /// run's seed (the first arrival of a run always answered). Answered,
    /// the reaction waits for its moment (`update_asides`), unless a better
    /// one is waiting
    pub fn aside(&mut self, u: Use, subject: &str) {
        let what = format!("{}:{subject}", u.name());
        if !subject.is_empty() && !u.subjects().contains(&subject) {
            self.aside_note(format!("skip:{what}:unknown subject"));
            return;
        }
        if choose(self.aside_lines(), u, subject, self.level_num, self.run_seed, &self.asides.memory.said).is_none() {
            return;
        }
        let clock = self.asides.clock;
        let k = key(u, subject, self.level_num);
        if self.asides.memory.weighed.contains(&k) {
            return;
        }
        let owed = self.asides.memory.owed.get(&k).copied();
        let d = match owed {
            Some(tries) => {
                // answered before and not said: answered again, a few times,
                // without a roll (so it moves no later roll of the run)
                if tries >= OWED_TRIES {
                    self.asides.memory.owed.remove(&k);
                    self.asides.memory.weighed.push(k.clone());
                    self.aside_note(format!("drop:{what}:owed too long"));
                    return;
                }
                self.asides.memory.owed.insert(k.clone(), tries + 1);
                0.5
            }
            None => {
                let n = {
                    let e = self.asides.memory.events.entry(u.name().to_string()).or_insert(0);
                    *e += 1;
                    *e - 1
                };
                let (x, d) = roll(u, self.run_seed, n);
                let said_of_use = self.asides.memory.said_of(u);
                let said_about = self.asides.memory.about.get(&what).copied().unwrap_or(0);
                // the first arrival of a run is always answered: your voice is
                // met; an arrival fades by its place alone (a new place, late
                // in a run, keeps its word: review B)
                let p = match u {
                    Use::Arrive if said_of_use == 0 => 1.0,
                    Use::Arrive => chance(u, self.run_seed, said_about, 0),
                    _ => chance(u, self.run_seed, said_about, said_of_use),
                };
                if x >= p {
                    self.asides.memory.weighed.push(k);
                    self.aside_note(format!("pass:{what}"));
                    return;
                }
                self.asides.memory.owed.insert(k.clone(), 1);
                d
            }
        };
        if let Some(p) = &self.asides.pending {
            if p.use_.priority() >= u.priority() {
                self.aside_note(format!("skip:{what}:busy"));
                return;
            }
        }
        let (lo, hi, win) = u.moment();
        self.asides.pending =
            Some(Pending { use_: u, subject: subject.to_string(), key: k, at: clock + lo + d * (hi - lo), until: clock + win });
        self.aside_note(format!("wait:{what}"));
    }

    /// A floor begins: its arrival waits for your first steps, its card and
    /// its line to go, and nothing else waiting; nothing waiting from
    /// before is said here
    pub fn asides_arrive(&mut self, subject: &str) {
        self.asides.pending = None;
        self.asides.floor += 1;
        self.asides.floor_began = self.asides.clock;
        let p = &self.player;
        self.asides.anchor = Some((p.pos.x, p.pos.z, p.yaw, p.pitch));
        self.asides.arrival = Some((p.pos.x, p.pos.z, subject.to_string(), self.asides.clock));
    }

    /// You go into the Stairwell or the Loop: nothing waiting from the floor
    /// is said in there, nor the floor's arrival
    pub fn asides_leave_floor(&mut self) {
        self.asides.pending = None;
        self.asides.arrival = None;
        self.asides.floor += 1;
        self.asides.floor_began = self.asides.clock;
        let p = &self.player;
        self.asides.anchor = Some((p.pos.x, p.pos.z, p.yaw, p.pitch));
    }

    /// An event your voice may answer, posted `ms` from now: after the sound
    /// it answers has finished (a word over a sound hides what it tells).
    /// Let go if by then the floor has changed (the next floor, the
    /// Stairwell, the Loop, a retry) or you are dead (review B)
    pub fn aside_later(&mut self, ms: f64, u: Use, subject: &'static str) {
        let floor = self.asides.floor;
        self.schedule(ms, move |g| {
            if g.asides.floor == floor && g.state != "dead" {
                g.aside(u, subject);
            }
        });
    }

    /// Your nerve as it goes (`update_sanity_effects`): getting near its
    /// end is an event for your asides (once a floor), back over `NERVE_LOW`'s
    /// second mark you may get there again
    pub fn asides_nerve(&mut self, share: f64) {
        if !self.asides.nerve_low && share < NERVE_LOW.0 {
            self.asides.nerve_low = true;
            self.aside(Use::Nerve, "");
        } else if self.asides.nerve_low && share > NERVE_LOW.1 {
            self.asides.nerve_low = false;
        }
    }

    /// What a floor's arrival is said about: its place
    pub fn arrival_subject(&self) -> &'static str {
        match (self.level_num, self.level_scp.as_deref()) {
            (1, _) => "well",
            (2, _) => "stair_halls",
            (3, _) => "works",
            (4, _) => "lobby",
            (_, Some("store")) => "showroom",
            (_, Some("house")) => "house",
            // a floor whose door leads to the Stairwell or the Loop: its line
            // is theirs (someone crying far below, a door clicking shut
            // again and again), so not the rooms' lines ("Again."): the
            // general ones
            (_, Some("stairwell")) => "stairwell_floor",
            (_, Some("loop")) => "loop_floor",
            (n, _) if n >= 10 => "deep",
            _ => "rooms",
        }
    }

    /// Why a reaction whose moment has come cannot be said now, if it cannot
    fn aside_held(&self, p: &Pending) -> Option<&'static str> {
        if self.state != "playing" {
            return Some("not playing");
        }
        // a cutscene has the camera: said once your hands are back
        if self.hands_taken() {
            return Some("cutscene");
        }
        // rule 7
        if self.asides.clock - self.asides.active_at > AWAY {
            return Some("away");
        }
        // a floor's card and line are up: nothing but a blow's grunt
        if !p.use_.cuts_in() && self.asides.clock - self.asides.floor_began < ARRIVAL_AFTER {
            return Some("card");
        }
        // rule 5 (a blow's grunt cuts in)
        if !p.use_.cuts_in() {
            if self.asides.voice_until > self.perf_t {
                return Some("speaking");
            }
            if let Some((at, pr)) = self.asides.last {
                let gap = if p.use_.urgent() && pr < p.use_.priority() { GAP_URGENT } else { GAP };
                if self.asides.clock - at < gap {
                    return Some("gap");
                }
            }
        }
        // rule 4
        self.holding_breath().filter(|why| p.use_.held_by(why))
    }

    /// One step of the voice director: its clock, whether you are playing
    /// (rule 7), a floor's arrival once you have walked a little, a line of
    /// yours let go when you begin to hold your breath, and the reaction
    /// waiting, said in its moment or let go after it (rules 4, 5)
    pub fn update_asides(&mut self, dt: f64) {
        self.asides.clock += dt;
        let t = self.asides.clock;
        let (x, z, yaw, pitch) = (self.player.pos.x, self.player.pos.z, self.player.yaw, self.player.pitch);
        let moved = match self.asides.anchor {
            Some((ax, az, ay, ap)) => hypot2(x - ax, z - az) > MOVED || (yaw - ay).abs() > TURNED || (pitch - ap).abs() > TURNED,
            // where you stand to begin with is not a move
            None => {
                self.asides.anchor = Some((x, z, yaw, pitch));
                false
            }
        };
        if moved {
            self.asides.anchor = Some((x, z, yaw, pitch));
        }
        if moved || !self.keys.is_empty() || self.mouse_dx != 0.0 || self.mouse_dy != 0.0 {
            self.asides.active_at = t;
        }
        if let Some((ax, az, subject, began)) = self.asides.arrival.clone() {
            if t > began + ARRIVAL_LATEST {
                self.asides.arrival = None;
            } else if t >= began + ARRIVAL_AFTER
                && hypot2(x - ax, z - az) > ARRIVAL_WALK
                && self.asides.pending.is_none()
                && self.asides.last.is_none_or(|(at, _)| t - at >= GAP)
            {
                self.asides.arrival = None;
                self.aside(Use::Arrive, &subject);
            }
        }
        // a line of yours stops when you begin to hold your breath (not the cry)
        if self.asides.voice_until > self.perf_t
            && matches!(self.holding_breath(), Some("hunted" | "searched" | "stalked" | "near" | "faced"))
        {
            self.cut_voice_line();
        }
        let Some(p) = self.asides.pending.clone() else { return };
        if t < p.at {
            return;
        }
        // past its moment it is not said (a late frame included); its key
        // stays owed, for the next event like it
        if t > p.until {
            self.asides.pending = None;
            let why = self.aside_held(&p).unwrap_or("late");
            self.aside_note(format!("drop:{}:{}:{why}", p.use_.name(), p.subject));
            return;
        }
        if self.aside_held(&p).is_none() {
            self.asides.pending = None;
            self.say_aside(&p);
        }
    }

    /// Say the line for `p`: dealt from the subject's and the use's lines,
    /// never one said before (rule 2); remembered whether or not it can be
    /// heard (no audio: the run goes on as if it had been), and saved at
    /// once, so a run left now and continued never says it again
    fn say_aside(&mut self, p: &Pending) {
        let what = format!("{}:{}", p.use_.name(), p.subject);
        let Some(line) = choose(self.aside_lines(), p.use_, &p.subject, self.level_num, self.run_seed, &self.asides.memory.said) else {
            self.aside_note(format!("skip:{what}:spent"));
            return;
        };
        let m = &mut self.asides.memory;
        m.said.push(line.id.clone());
        *m.about.entry(what.clone()).or_insert(0) += 1;
        m.owed.remove(&p.key);
        m.weighed.push(p.key.clone());
        self.asides.last = Some((self.asides.clock, p.use_.priority()));
        if p.use_.cuts_in() {
            self.cut_voice_line();
        }
        // (no audio, or no WAV: as long as a short line)
        if self.play_voice_line(&line, ASIDE_GAIN).is_none() {
            self.asides.voice_until = self.perf_t + 1.5;
        }
        self.aside_note(format!("say:{what}:{}", line.id));
        if self.state == "playing" && self.run_seed != 0 {
            self.save_game();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::director::{AFTER_CRY, HUSHED};

    fn ln(id: &str, u: &str, subject: &str, first: i32, last: i32) -> Line {
        Line {
            persona: PERSONA.into(),
            id: id.into(),
            use_: u.into(),
            subject: subject.into(),
            first,
            last,
            intensity: 0.3,
            emotion: "wary".into(),
            text: id.into(),
        }
    }

    fn deck() -> Vec<Line> {
        vec![
            ln("a1", "arrive", "", 4, 99),
            ln("a2", "arrive", "", 4, 99),
            ln("a3", "arrive", "", 4, 99),
            ln("w1", "arrive", "well", 1, 1),
            ln("w2", "arrive", "well", 1, 1),
            ln("s1", "startle", "bang", 1, 99),
            ln("s2", "startle", "", 1, 99),
            ln("s3", "startle", "", 1, 99),
            ln("s4", "startle", "", 1, 99),
            ln("s5", "startle", "", 1, 99),
            ln("k1", "knocked", "", 4, 99),
            ln("k2", "knocked", "", 4, 99),
            ln("si1", "sighted", "", 1, 99),
            ln("n1", "nerve", "", 1, 99),
            ln("cry", "cry", "", 1, 99),
        ]
    }

    /// A bare game on a floor, playing, with the deck above
    fn bare(level: i32) -> Game {
        let mut g = Game::new_bare();
        g.state = "playing".into();
        g.run_seed = 4242;
        g.level_num = level;
        g.perf_t = 100.0;
        g.asides.lines = Some(deck());
        // well past a floor's card
        g.asides.floor_began = -100.0;
        g
    }

    /// `secs` of frames, looking round (playing) or not
    fn step(g: &mut Game, secs: f64, looking: bool) {
        for _ in 0..(secs * 60.0) as usize {
            g.perf_t += 1.0 / 60.0;
            if looking {
                g.player.yaw += 0.06;
            }
            g.update_asides(1.0 / 60.0);
        }
    }

    fn said(g: &Game) -> Vec<String> {
        g.asides.log.iter().filter(|(_, w)| w.starts_with("say:")).map(|(_, w)| w.clone()).collect()
    }

    /// An event of `u` answered: each of its subjects in turn (each its own
    /// roll); failing that, one not yet done with, owed (answered without a
    /// roll), so a test does not hang on the run's dice
    fn answer(g: &mut Game, u: Use) -> bool {
        let subjects: Vec<&str> = if u.subjects().is_empty() { vec![""] } else { u.subjects().to_vec() };
        let answered = |g: &Game| g.asides.pending.as_ref().is_some_and(|p| p.use_ == u);
        for s in &subjects {
            g.aside(u, s);
            if answered(g) {
                return true;
            }
        }
        for s in &subjects {
            let k = key(u, s, g.level_num);
            g.asides.memory.weighed.retain(|w| *w != k);
            g.asides.memory.owed.insert(k, 1);
            g.aside(u, s);
            if answered(g) {
                return true;
            }
        }
        false
    }

    #[test]
    fn a_subject_is_answered_from_its_own_lines_and_the_uses_general_ones() {
        let d = deck();
        // the Well's own (the general arrivals are the rooms')
        assert_eq!(choose(&d, Use::Arrive, "well", 1, 7, &[]).unwrap().subject, "well");
        let said = vec!["w1".to_string(), "w2".to_string()];
        assert!(choose(&d, Use::Arrive, "well", 1, 7, &said).is_none(), "the Well's spent");
        // a bang: its own line or a general one, by the run's deal
        let firsts: std::collections::BTreeSet<String> = (1..60).map(|s| choose(&d, Use::Startle, "bang", 5, s, &[]).unwrap().id).collect();
        assert!(firsts.contains("s1") && firsts.len() >= 3, "{firsts:?}");
        // a kind with no line of its own: the general ones; no subject: never a subject's
        assert!(choose(&d, Use::Startle, "scream", 5, 7, &[]).unwrap().subject.is_empty());
        let general: Vec<String> = (1..60).map(|s| choose(&d, Use::Startle, "", 5, s, &[]).unwrap().id).collect();
        assert!(!general.contains(&"s1".to_string()));
        // never another persona's, another use's or another level's
        assert!(choose(&d, Use::Arrive, "", 2, 7, &[]).is_none());
        assert!(choose(&d, Use::Dark, "", 5, 7, &[]).is_none());
    }

    #[test]
    fn a_line_is_never_said_twice_and_a_spent_use_is_silent() {
        let d = deck();
        let mut said: Vec<String> = vec![];
        while let Some(l) = choose(&d, Use::Startle, "bang", 5, 99, &said) {
            assert!(!said.contains(&l.id), "{} twice", l.id);
            said.push(l.id);
        }
        assert_eq!(said.len(), 5, "the bang's own and every general startle once: {said:?}");
    }

    #[test]
    fn the_choice_is_the_runs_and_runs_differ() {
        let d = deck();
        let first = |seed| choose(&d, Use::Startle, "", 5, seed, &[]).unwrap().id;
        assert_eq!(first(4242), first(4242), "the same run, the same line");
        let distinct: std::collections::BTreeSet<String> = (1..40).map(first).collect();
        assert!(distinct.len() >= 3, "runs begin with different lines: {distinct:?}");
        let temper: Vec<f64> = (1..40).map(temperament).collect();
        assert!(temper.iter().all(|t| (0.65..1.0).contains(t)));
        let spread = temper.iter().cloned().fold(0.0, f64::max) - temper.iter().cloned().fold(1.0, f64::min);
        assert!(spread > 0.2, "some runs talk more than others: {spread:.2}");
        assert_eq!(roll(Use::Startle, 9, 3), roll(Use::Startle, 9, 3));
        assert_ne!(roll(Use::Startle, 9, 3), roll(Use::Startle, 9, 4));
        assert_ne!(roll(Use::Startle, 9, 3), roll(Use::Sighted, 9, 3));
        // seeds a few bits apart do not share their rolls (review A)
        for k in [1u32, 2, 3, 7] {
            assert_ne!(roll(Use::Startle, 9, 3), roll(Use::Startle, 9 ^ k, 3 ^ k), "k {k}");
        }
    }

    #[test]
    fn the_chance_falls_with_each_line_said_and_the_scopes_keep_their_keys() {
        for u in USES {
            let by_subject: Vec<f64> = (0..4).map(|k| chance(u, 4242, k, k)).collect();
            assert!(by_subject.windows(2).all(|w| w[1] < w[0]), "{u:?}: {by_subject:?}");
            // a new subject keeps most of its chance however much was said of the use
            assert!(chance(u, 4242, 0, 3) > 0.6 * chance(u, 4242, 0, 0), "{u:?}");
            assert!(chance(u, 4242, 3, 3) < 0.5 * chance(u, 4242, 0, 0), "{u:?}");
        }
        assert_eq!(key(Use::Startle, "bang", 5), "startle:bang");
        assert_eq!(key(Use::Arrive, "rooms", 5), "arrive:rooms:5", "a floor's arrival once a floor");
        assert_eq!(key(Use::Knocked, "", 5), "knocked:", "a blow: once a run, as every kind");
    }

    #[test]
    fn every_subject_the_game_posts_is_one_it_knows() {
        for (_, sig) in crate::base::tables::AMBIENT_EVENTS {
            if let Some(s) = startle_of(sig) {
                assert!(Use::Startle.subjects().contains(&s), "{sig}: {s}");
            }
        }
        assert!(Use::Startle.subjects().contains(&"steps"), "the stalker's steps (director)");
        let mut g = bare(1);
        for level in 1..=30 {
            for scp in [None, Some("store"), Some("house"), Some("stairwell"), Some("loop")] {
                g.level_num = level;
                g.level_scp = scp.map(String::from);
                assert!(Use::Arrive.subjects().contains(&g.arrival_subject()), "{level} {scp:?}: {}", g.arrival_subject());
            }
        }
        // and one it does not know is refused, and weighs nothing
        g.level_num = 5;
        g.aside(Use::Startle, "slam");
        assert!(g.asides.memory.events.is_empty() && g.asides.log.last().is_some_and(|(_, w)| w.ends_with("unknown subject")));
    }

    #[test]
    fn an_arrival_waits_for_your_first_steps_and_the_first_is_always_said() {
        let mut g = bare(1);
        g.asides_arrive("well");
        step(&mut g, ARRIVAL_AFTER + 1.0, true);
        assert!(said(&g).is_empty() && g.asides.pending.is_none(), "standing, looking round: no arrival yet {:?}", g.asides.log);
        // walking while the card and the floor's line are up: not yet
        g.asides_arrive("well");
        g.player.pos.x += 3.0;
        step(&mut g, ARRIVAL_AFTER - 0.5, true);
        assert!(g.asides.pending.is_none() && said(&g).is_empty(), "{:?}", g.asides.log);
        step(&mut g, 6.0, true);
        let s = said(&g);
        assert!(s.len() == 1 && s[0].starts_with("say:arrive:well:w"), "{:?}", g.asides.log);
        assert!(g.asides.memory.weighed.contains(&"arrive:well:1".to_string()));
        // a retry of the floor arrives again, and is not weighed again
        g.asides_arrive("well");
        g.player.pos.x += 3.0;
        step(&mut g, ARRIVAL_AFTER + 30.0, true);
        assert_eq!(said(&g).len(), 1, "{:?}", g.asides.log);
    }

    #[test]
    fn the_first_arrival_cannot_be_lost_to_a_held_breath() {
        // the Lobby's arrival comes as you have just cried: held, let go
        let mut g = bare(4);
        g.asides_arrive("lobby");
        g.player.pos.x += 3.0;
        step(&mut g, ARRIVAL_AFTER - 0.1, true);
        g.amb_hush = HUSHED;
        step(&mut g, ARRIVAL_LATEST, true);
        g.amb_hush = 0.0;
        assert!(said(&g).is_empty() && g.asides.log.iter().any(|(_, w)| w.starts_with("drop:arrive:lobby")), "{:?}", g.asides.log);
        // the next floor's is still answered for certain: no arrival said yet
        g.level_num = 5;
        g.asides_arrive("rooms");
        g.player.pos.x += 3.0;
        step(&mut g, ARRIVAL_AFTER + 5.0, true);
        assert_eq!(said(&g).len(), 1, "{:?}", g.asides.log);
    }

    #[test]
    fn an_arrival_walked_after_its_first_minute_played_is_not_weighed_and_a_pause_does_not_count() {
        let mut g = bare(1);
        g.asides_arrive("well");
        step(&mut g, ARRIVAL_LATEST + 1.0, true);
        g.player.pos.x += 3.0;
        step(&mut g, 5.0, true);
        assert!(g.asides.memory.events.is_empty() && said(&g).is_empty(), "{:?}", g.asides.log);
        // paused: the game's time runs, the director's does not
        let mut g = bare(1);
        g.asides_arrive("well");
        step(&mut g, 2.0, true);
        g.perf_t += 300.0;
        g.player.pos.x += 3.0;
        step(&mut g, ARRIVAL_AFTER + 4.0, true);
        assert_eq!(said(&g).len(), 1, "{:?}", g.asides.log);
    }

    #[test]
    fn nothing_is_said_to_an_empty_chair() {
        let mut g = bare(5);
        assert!(answer(&mut g, Use::Startle));
        step(&mut g, 4.0, false);
        assert!(said(&g).is_empty(), "{:?}", g.asides.log);
        assert!(g.asides.log.iter().any(|(_, w)| w.ends_with(":away")), "{:?}", g.asides.log);
        // a key held down is someone playing
        g.asides.clock += AWAY + 1.0;
        g.keys.insert("KeyW".into());
        step(&mut g, 0.1, false);
        assert_eq!(g.asides.active_at, g.asides.clock);
    }

    #[test]
    fn a_reaction_is_said_in_its_moment_and_the_next_waits_its_gap() {
        let mut g = bare(5);
        step(&mut g, 1.0, true);
        assert!(answer(&mut g, Use::Startle));
        let (lo, _, win) = Use::Startle.moment();
        let t0 = g.asides.clock;
        step(&mut g, win + 0.5, true);
        let at = g.asides.last.expect("said").0;
        assert!(at - t0 >= lo - 1e-9 && at - t0 <= win, "said {:.2} s after", at - t0);
        // another within the gap is let go after its moment, and owed
        assert!(answer(&mut g, Use::Startle));
        let owed = g.asides.pending.as_ref().map(|p| p.key.clone()).unwrap();
        step(&mut g, win + 0.5, true);
        assert_eq!(said(&g).len(), 1, "{:?}", g.asides.log);
        assert!(g.asides.log.last().is_some_and(|(_, w)| w.ends_with(":gap")), "{:?}", g.asides.log);
        assert_eq!(g.asides.memory.owed.get(&owed), Some(&1));
        // past the gap, the next event like it is answered without a roll, and said
        step(&mut g, GAP, true);
        let events = g.asides.memory.events.clone();
        let subject = owed.split(':').nth(1).unwrap().to_string();
        g.aside(Use::Startle, &subject);
        assert_eq!(g.asides.memory.events, events, "an owed reaction is not rolled again");
        step(&mut g, win + 0.5, true);
        let s = said(&g);
        assert_eq!(s.len(), 2, "{:?}", g.asides.log);
        assert_ne!(s[0], s[1]);
        assert!(g.asides.memory.owed.is_empty() && g.asides.memory.weighed.contains(&owed));
    }

    #[test]
    fn you_hold_your_breath_after_your_cry_and_a_blow_cuts_in() {
        let mut g = bare(5);
        step(&mut g, 1.0, true);
        g.cry_last_at = Some(g.perf_t);
        assert!(answer(&mut g, Use::Startle));
        step(&mut g, 2.5, true);
        assert!(said(&g).is_empty());
        assert!(g.asides.log.last().is_some_and(|(_, w)| w.ends_with(":cried")), "{:?}", g.asides.log);
        assert_eq!(g.holding_breath(), Some("cried"));
        step(&mut g, AFTER_CRY - 2.5 + 0.1, true);
        assert_eq!(g.holding_breath(), None, "the breath is let go");
        // a line of yours sounding, the gap not kept: a blow's grunt is said
        g.asides.voice_until = g.perf_t + 5.0;
        g.asides.last = Some((g.asides.clock, 3));
        g.cry_last_at = Some(g.perf_t);
        assert!(answer(&mut g, Use::Knocked));
        step(&mut g, 2.0, true);
        assert!(g.asides.log.last().is_some_and(|(_, w)| w.starts_with("say:knocked::k")), "{:?}", g.asides.log);
    }

    #[test]
    fn you_keep_the_places_silence_but_not_in_the_stairwell() {
        let mut g = bare(5);
        step(&mut g, 1.0, true);
        g.amb_hush = HUSHED;
        assert_eq!(g.holding_breath(), Some("hushed"));
        assert!(answer(&mut g, Use::Startle));
        step(&mut g, 3.0, true);
        assert!(said(&g).is_empty() && g.asides.log.last().is_some_and(|(_, w)| w.ends_with(":hushed")), "{:?}", g.asides.log);
        // the siren and a knock at your cell are what the hush is about
        assert!(!Use::Siren.held_by("hushed") && !Use::Held.held_by("hushed") && Use::Held.held_by("cried"));
        g.amb_hush = HUSHED - 0.05;
        assert_eq!(g.holding_breath(), None);
        // in the Stairwell or the Loop the floor's hush and creatures are not there
        g.amb_hush = 0.9;
        g.in_stairwell = true;
        assert_eq!(g.holding_breath(), None);
        g.asides.pending = Some(Pending { use_: Use::Startle, subject: "bang".into(), key: "startle:bang".into(), at: 0.0, until: 1e9 });
        g.asides_leave_floor();
        assert!(g.asides.pending.is_none() && g.asides.arrival.is_none());
    }

    #[test]
    fn every_kind_of_event_has_its_word_at_most_once_a_run() {
        // a blow, said: a later blow has none (the user's ask: no voice on a
        // later event of the same kind)
        let mut g = bare(5);
        step(&mut g, 1.0, true);
        assert!(answer(&mut g, Use::Knocked));
        step(&mut g, 3.0, true);
        assert_eq!(said(&g).len(), 1, "{:?}", g.asides.log);
        let (log, events) = (g.asides.log.len(), g.asides.memory.events.clone());
        step(&mut g, GAP + 1.0, true);
        g.aside(Use::Knocked, "");
        step(&mut g, 3.0, true);
        assert_eq!((g.asides.log.len(), &g.asides.memory.events), (log, &events), "{:?}", g.asides.log);
        // every use but a floor's arrival is once a run
        assert!(USES.iter().all(|u| (u.scope() == Scope::Level) == (*u == Use::Arrive)));
    }

    #[test]
    fn nothing_but_a_blow_while_a_floors_card_is_up() {
        let mut g = bare(5);
        step(&mut g, 1.0, true);
        g.asides.floor_began = g.asides.clock;
        assert!(answer(&mut g, Use::Startle));
        step(&mut g, 3.0, true);
        assert!(said(&g).is_empty() && g.asides.log.last().is_some_and(|(_, w)| w.ends_with(":card")), "{:?}", g.asides.log);
        assert!(answer(&mut g, Use::Knocked));
        step(&mut g, 2.0, true);
        assert!(said(&g).iter().any(|w| w.starts_with("say:knocked:")), "{:?}", g.asides.log);
    }

    #[test]
    fn an_aside_posted_for_later_is_let_go_on_another_floor_or_in_death() {
        let mut g = bare(5);
        g.aside_later(1000.0, Use::Startle, "bang");
        g.asides_arrive("rooms");
        g.now_ms += 1500.0;
        g.run_scheduled();
        assert!(g.asides.memory.events.is_empty(), "posted on the floor before: {:?}", g.asides.memory);
        g.aside_later(1000.0, Use::Startle, "bang");
        g.state = "dead".into();
        g.now_ms += 1500.0;
        g.run_scheduled();
        assert!(g.asides.memory.events.is_empty(), "posted while dead: {:?}", g.asides.memory);
        g.state = "playing".into();
        g.aside_later(1000.0, Use::Startle, "bang");
        g.now_ms += 1500.0;
        g.run_scheduled();
        assert_eq!(g.asides.memory.events.get("startle"), Some(&1), "on its own floor it is posted");
    }

    #[test]
    fn a_subject_is_weighed_once_a_run_and_a_better_reaction_replaces_a_lesser() {
        let mut g = bare(5);
        g.aside(Use::Startle, "bang");
        g.aside(Use::Startle, "bang");
        assert_eq!(g.asides.memory.events.get("startle"), Some(&1), "the second bang is not rolled");
        let k = "startle:bang".to_string();
        assert!(g.asides.memory.weighed.contains(&k) != g.asides.memory.owed.contains_key(&k), "{:?}", g.asides.memory);
        // a lesser one waiting gives way to a better
        g.asides.pending = Some(Pending { use_: Use::Arrive, subject: "rooms".into(), key: "arrive:rooms:5".into(), at: 0.0, until: 1e9 });
        assert!(answer(&mut g, Use::Knocked));
        assert_eq!(g.asides.pending.as_ref().map(|p| p.use_), Some(Use::Knocked));
        // and a lesser one does not replace it
        g.aside(Use::Sighted, "");
        assert_eq!(g.asides.pending.as_ref().map(|p| p.use_), Some(Use::Knocked));
    }

    #[test]
    fn an_event_with_no_line_here_is_not_weighed() {
        // the deck's arrivals for level 2: none; a blow's: from level 4
        let mut g = bare(2);
        g.aside(Use::Arrive, "stair_halls");
        g.aside(Use::Knocked, "");
        assert!(g.asides.memory.weighed.is_empty() && g.asides.memory.events.is_empty(), "{:?}", g.asides.memory);
        g.aside(Use::Startle, "bang");
        assert_eq!(g.asides.memory.events.get("startle"), Some(&1));
    }

    #[test]
    fn your_nerve_near_its_end_is_one_event_until_it_comes_back() {
        let mut g = bare(5);
        g.asides_nerve(0.5);
        assert!(g.asides.memory.events.is_empty());
        g.asides_nerve(NERVE_LOW.0 - 0.01);
        assert_eq!(g.asides.memory.events.get("nerve"), Some(&1));
        // still low, or barely back: nothing more
        g.asides_nerve(0.1);
        g.asides_nerve(NERVE_LOW.1 - 0.01);
        g.asides_nerve(0.2);
        assert_eq!(g.asides.memory.events.get("nerve"), Some(&1));
        assert!(g.asides.nerve_low);
        // back over the second mark, and down again: a second event (weighed
        // once a floor: answered again only if it was owed)
        g.asides_nerve(NERVE_LOW.1 + 0.01);
        assert!(!g.asides.nerve_low);
        g.asides_nerve(0.2);
        assert!(g.asides.nerve_low);
    }

    /// The run's deal reaches every line: for each use and subject the game
    /// posts, on each level, every line it could say there comes first in
    /// some run (none is dead behind another, none always first)
    #[test]
    fn every_line_of_the_game_comes_first_in_some_run() {
        let all = super::super::voice::index();
        for u in USES {
            let subjects: Vec<&str> = if u.subjects().is_empty() { vec![""] } else { u.subjects().to_vec() };
            for s in subjects {
                for level in 1..=14 {
                    let firsts: std::collections::BTreeSet<String> =
                        (1..400).filter_map(|seed| choose(all, u, s, level, seed, &[])).map(|l| l.id).collect();
                    let could: std::collections::BTreeSet<String> = all
                        .iter()
                        .filter(|l| l.persona == PERSONA && l.use_ == u.name() && l.first <= level && level <= l.last)
                        .filter(|l| l.subject.is_empty() || l.subject == s)
                        .map(|l| l.id.clone())
                        .collect();
                    assert_eq!(firsts, could, "{}:{s} on level {level}", u.name());
                }
            }
        }
    }

    #[test]
    fn the_memory_goes_through_the_save_and_back() {
        let mut m = AsideMemory::default();
        m.said = vec!["a1".into(), "s3".into()];
        m.about.insert("arrive:rooms".into(), 1);
        m.about.insert("startle:bang".into(), 1);
        m.weighed = vec!["arrive:rooms:4".into(), "startle:bang".into()];
        m.owed.insert("startle:moan".into(), 2);
        m.events.insert("startle".into(), 3);
        let back = AsideMemory::from_json(Some(&m.to_json()));
        assert_eq!(back, m);
        assert_eq!(back.said_of(Use::Startle), 1);
        assert_eq!(AsideMemory::from_json(None), AsideMemory::default());
        let odd = serde_json::json!({"said": [1, "x"], "events": {"nonsense": 4, "dark": 2}, "owed": {"what:x": 1}});
        let o = AsideMemory::from_json(Some(&odd));
        assert_eq!(o.said, vec!["x".to_string()]);
        assert_eq!(o.events.len(), 1, "an unknown use is dropped");
        assert!(o.owed.is_empty());
    }
}
