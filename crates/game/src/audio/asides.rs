//! Your asides (`docs/VOICES.md`, *Your asides*): what you say to yourself,
//! under your breath, on arriving, at a sudden noise, at what comes and at
//! what the place does back. A director of their own decides, after the
//! pacing director's pattern and Valve's dynamic dialog
//! (`docs/research/inner-voice.md`): an event (a use, a subject) is weighed
//! once, by the run's seed; a reaction waits for its moment and is said then
//! or never; its line is the subject's before the use's general ones, and
//! never one said before in the run.
//!
//! The rules (each held by a test here, or named beside it):
//! 1. react, never explain (the lines themselves: `voice::tests`)
//! 2. once is a reaction, twice a habit (`key`, `chance`, `choose`)
//! 3. different every run (`temperament`, `roll`, `deck_order`: the run's
//!    seed on streams of their own; `AsideMemory` kept with the run)
//! 4. hold your breath (`Game::holding_breath`)
//! 5. one at a time, and room between (`Game::aside_held`, `Use::moment`)
//! 6. heard by nothing but you (nothing here reaches a creature, a draw of
//!    the per-frame stream, the light or the screen)
//! 7. only for a person playing (`AWAY`)

use super::voice::Line;
use crate::base::game::Game;
use crate::base::numeric::hypot2;
use crate::base::rng::SplitMix64;
use crate::entities::director::Phase;

/// Whose lines your asides are: you, under your breath
pub const PERSONA: &str = "you_aside";
/// The level of an aside in the mix: its WAV is at -20 LUFS (the cry's at
/// -16), and it is played lower again, under your breath
pub const ASIDE_GAIN: f64 = 0.7;
/// At least this long between two asides, s
pub const GAP: f64 = 20.0;
/// ... or this, for an urgent one after a lesser (`Use::urgent`), s
pub const GAP_URGENT: f64 = 8.0;
/// Neither moved nor looked round for this long, s: nobody is playing
pub const AWAY: f64 = 30.0;
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
/// (walked later, its moment has gone), s
pub const ARRIVAL_LATEST: f64 = 60.0;
/// No aside this soon after your cry, s: you listen for what answers (the
/// place answers in 0.6-0.9 s, `Game::cry_for_help`)
pub const AFTER_CRY: f64 = 3.0;
/// You hold your breath while something searches this near, m
pub const SEARCH_NEAR: f64 = 25.0;
/// ... and while the place holds its own this much (`amb_hush`: something
/// coming, a search near, a silent machine, the fog up): silence is a
/// warning, and you keep it (PILLARS 3; `lore-keeper`, A4)
pub const HUSHED: f64 = 0.3;
/// ... or the one sent in a build-up is this near, m
pub const STALKER_NEAR: f64 = 20.0;
/// The most events one run's log keeps
pub const LOG_CAP: usize = 2048;

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
    /// What each of its lines already said this run leaves of the chance:
    /// a floor's arrival fades slowly (there are many floors), the rest by
    /// half (rule 2)
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
    /// A blow's grunt is not chosen: it is said even while you hold your breath
    pub fn involuntary(self) -> bool {
        matches!(self, Use::Knocked)
    }
    /// Its moment: the least and most seconds from the event to the line,
    /// and how long after the event it may still be said (rule 5: in its
    /// moment or never)
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
    pub fn from_name(s: &str) -> Option<Use> {
        USES.iter().copied().find(|u| u.name() == s)
    }
}

/// What a run remembers of your asides (saved with it): what was said, the
/// events already weighed, how many of each use there have been
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AsideMemory {
    /// The lines said this run, in order (none twice: rule 2)
    pub said: Vec<String>,
    /// The once-a-run events already weighed (`key`): a later one like it
    /// is not weighed again, said or not (rule 2)
    pub weighed: Vec<String>,
    /// How many events of each use there have been this run: each one's
    /// roll is its own (`roll`)
    pub events: std::collections::BTreeMap<String, u32>,
}

impl AsideMemory {
    /// As the save keeps it
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({ "said": self.said, "weighed": self.weighed, "events": self.events })
    }
    /// From the save (anything missing or malformed: as a new run's)
    pub fn from_json(v: Option<&serde_json::Value>) -> AsideMemory {
        let Some(v) = v else { return AsideMemory::default() };
        let strs = |k: &str| -> Vec<String> {
            v.get(k).and_then(|a| a.as_array()).map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default()
        };
        AsideMemory {
            said: strs("said"),
            weighed: strs("weighed"),
            events: v
                .get("events")
                .and_then(|h| h.as_object())
                .map(|h| {
                    h.iter()
                        .filter(|(k, _)| Use::from_name(k).is_some())
                        .filter_map(|(k, n)| n.as_u64().map(|n| (k.clone(), n.min(u32::MAX as u64) as u32)))
                        .collect()
                })
                .unwrap_or_default(),
        }
    }
}

/// A reaction waiting for its moment
#[derive(Clone, Debug, PartialEq)]
pub struct Pending {
    pub use_: Use,
    pub subject: String,
    /// Said from this time (game time, s) ...
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
    /// When the last aside was said, and its use's priority
    pub last: Option<(f64, u8)>,
    /// Until when a line of your voice (an aside or your cry) still sounds
    pub voice_until: f64,
    /// When you last moved or looked round, and where and how you stood then
    pub active_at: f64,
    pub anchor: Option<(f64, f64, f64, f64)>,
    /// This floor's arrival, waiting for your first steps: where it began,
    /// what it is said about, and the time before which it is not weighed
    pub arrival: Option<(f64, f64, String, f64)>,
    /// What happened, by game time: `wait:`, `pass:`, `skip:`, `drop:`,
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
            last: None,
            voice_until: 0.0,
            // nobody has played yet
            active_at: f64::NEG_INFINITY,
            anchor: None,
            arrival: None,
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
/// temperament, times its decay for each of its lines already said (rule 2)
pub fn chance(u: Use, run_seed: u32, said_of_use: usize) -> f64 {
    u.chance() * temperament(run_seed) * u.decay().powi(said_of_use as i32)
}

/// The `n`th event of `u` in the run: its roll (answered when under the
/// chance) and where in its moment it is said (0-1), from a stream of its
/// own: the run's seed, the use, the count (rule 3; never the per-frame
/// stream, `CLAUDE.md` invariant 1)
pub fn roll(u: Use, run_seed: u32, n: u32) -> (f64, f64) {
    let mut r = SplitMix64::new(run_seed, ROLL_SALT ^ ((u as u64 + 1) << 40) ^ n as u64);
    (r.next(), r.next())
}

/// A line's place in the run's order: each run deals the lines afresh (rule 3)
fn deck_order(run_seed: u32, id: &str) -> u64 {
    // FNV-1a of the id: the same id, the same place in a run
    let h = id.bytes().fold(0xCBF2_9CE4_8422_2325u64, |h, b| (h ^ b as u64).wrapping_mul(0x0100_0000_01B3));
    (SplitMix64::new(run_seed, DECK_SALT ^ h).next() * (1u64 << 53) as f64) as u64
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

/// The once-a-run key of an event: a subject is weighed once a run, a
/// floor's arrival once a floor (a retry does not arrive again); the uses
/// with no subject (a blow, the relief, your nerve, the lamp) are weighed
/// every time, each line said lowering the next chance (rule 2)
pub fn key(u: Use, subject: &str, level: i32) -> Option<String> {
    match u {
        Use::Arrive => Some(format!("arrive:{level}")),
        _ if subject.is_empty() => None,
        _ => Some(format!("{}:{subject}", u.name())),
    }
}

/// The line an event is answered with: among `all`, the persona's lines of
/// the use for the level and not said this run, the subject's own before
/// the use's general ones (the most specific match), the first in the run's
/// order; none when the use is spent (rules 2, 3, 5)
pub fn choose(all: &[Line], u: Use, subject: &str, level: i32, run_seed: u32, said: &[String]) -> Option<Line> {
    let fits = |l: &&Line| l.persona == PERSONA && l.use_ == u.name() && l.first <= level && level <= l.last && !said.contains(&l.id);
    let own: Vec<&Line> = all.iter().filter(fits).filter(|l| !subject.is_empty() && l.subject == subject).collect();
    let pool = if own.is_empty() { all.iter().filter(fits).filter(|l| l.subject.is_empty()).collect() } else { own };
    pool.into_iter().min_by_key(|l| deck_order(run_seed, &l.id)).cloned()
}

impl Game {
    fn aside_note(&mut self, what: String) {
        if self.asides.log.len() < LOG_CAP {
            let t = self.perf_t;
            self.asides.log.push((t, what));
        }
    }

    fn aside_lines(&self) -> &[Line] {
        match &self.asides.lines {
            Some(l) => l,
            None => super::voice::index(),
        }
    }

    /// An event your voice may answer. Weighed once (a subject once a run,
    /// rule 2), by the run's seed (rule 3); answered, the reaction waits for
    /// its moment (`update_asides`), and a better one waiting is not
    /// replaced by a lesser. An event with no line left to say for it here
    /// is not weighed at all (nothing plays, and nothing is spent: a sound
    /// on a floor whose lines are elsewhere keeps its kind for later)
    pub fn aside(&mut self, u: Use, subject: &str) {
        if choose(self.aside_lines(), u, subject, self.level_num, self.run_seed, &self.asides.memory.said).is_none() {
            return;
        }
        if let Some(k) = key(u, subject, self.level_num) {
            if self.asides.memory.weighed.contains(&k) {
                return;
            }
            self.asides.memory.weighed.push(k);
        }
        let n = {
            let e = self.asides.memory.events.entry(u.name().to_string()).or_insert(0);
            *e += 1;
            *e - 1
        };
        let said_of_use = {
            let lines = self.aside_lines();
            self.asides.memory.said.iter().filter(|id| lines.iter().any(|l| &l.id == *id && l.use_ == u.name())).count()
        };
        let what = format!("{}:{subject}", u.name());
        let (x, d) = roll(u, self.run_seed, n);
        // the first floor of a run always has its arrival: your voice is met
        let p = if u == Use::Arrive && n == 0 { 1.0 } else { chance(u, self.run_seed, said_of_use) };
        if x >= p {
            self.aside_note(format!("pass:{what}"));
            return;
        }
        if let Some(p) = &self.asides.pending {
            if p.use_.priority() >= u.priority() {
                self.aside_note(format!("skip:{what}:busy"));
                return;
            }
        }
        let (lo, hi, win) = u.moment();
        let t = self.perf_t;
        self.asides.pending = Some(Pending { use_: u, subject: subject.to_string(), at: t + lo + d * (hi - lo), until: t + win });
        self.aside_note(format!("wait:{what}"));
    }

    /// A floor begins (or the Stairwell, or the Loop): its arrival waits
    /// for your first steps, and for its card and line to have gone;
    /// nothing waiting from before is said here
    pub fn asides_arrive(&mut self, subject: &str) {
        self.asides.pending = None;
        let p = &self.player;
        self.asides.anchor = Some((p.pos.x, p.pos.z, p.yaw, p.pitch));
        self.asides.arrival = Some((p.pos.x, p.pos.z, subject.to_string(), self.perf_t + ARRIVAL_AFTER));
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

    /// Rule 4: why you hold your breath now, if you do: just after your
    /// cry; while something hunts you, or searches near you; while the one
    /// sent in a build-up is near; while the place itself is hushed
    pub fn holding_breath(&self) -> Option<&'static str> {
        let t = self.perf_t;
        if self.cry_last_at.is_some_and(|c| t - c < AFTER_CRY) {
            return Some("cried");
        }
        if self.amb_hush >= HUSHED {
            return Some("hushed");
        }
        let pp = self.player.pos;
        for (i, e) in self.entities.iter().enumerate() {
            if e.kind == "gone" {
                continue;
            }
            let p = e.pos();
            let d = hypot2(p.x - pp.x, p.z - pp.z);
            match e.state.as_str() {
                "hunt" => return Some("hunted"),
                "search" if d < SEARCH_NEAR => return Some("searched"),
                _ if self.director.phase == Phase::Build && self.director.stalker == Some(i) && d < STALKER_NEAR => return Some("stalked"),
                _ => {}
            }
        }
        None
    }

    /// Why a reaction whose moment has come cannot be said now, if it cannot
    fn aside_held(&self, p: &Pending) -> Option<&'static str> {
        let t = self.perf_t;
        if self.state != "playing" {
            return Some("not playing");
        }
        // rule 7
        if t - self.asides.active_at > AWAY {
            return Some("away");
        }
        // rule 5
        if self.asides.voice_until > t {
            return Some("speaking");
        }
        if let Some((at, pr)) = self.asides.last {
            let gap = if p.use_.urgent() && pr < p.use_.priority() { GAP_URGENT } else { GAP };
            if t - at < gap {
                return Some("gap");
            }
        }
        // rule 4
        if !p.use_.involuntary() {
            if let Some(why) = self.holding_breath() {
                return Some(why);
            }
        }
        None
    }

    /// One step of the voice director: whether you are playing (rule 7), a
    /// floor's arrival once you have walked a little, and the reaction
    /// waiting, said in its moment or let go after it (rules 4, 5)
    pub fn update_asides(&mut self, _dt: f64) {
        let t = self.perf_t;
        let (x, z, yaw, pitch) = (self.player.pos.x, self.player.pos.z, self.player.yaw, self.player.pitch);
        let moved = match self.asides.anchor {
            Some((ax, az, ay, ap)) => hypot2(x - ax, z - az) > MOVED || (yaw - ay).abs() > TURNED || (pitch - ap).abs() > TURNED,
            // where you stand to begin with is not a move
            None => {
                self.asides.anchor = Some((x, z, yaw, pitch));
                false
            }
        };
        if moved || !self.keys.is_empty() {
            if moved {
                self.asides.anchor = Some((x, z, yaw, pitch));
            }
            self.asides.active_at = t;
        }
        if let Some((ax, az, subject, from)) = self.asides.arrival.clone() {
            if t > from - ARRIVAL_AFTER + ARRIVAL_LATEST {
                self.asides.arrival = None;
            } else if t >= from && hypot2(x - ax, z - az) > ARRIVAL_WALK {
                self.asides.arrival = None;
                self.aside(Use::Arrive, &subject);
            }
        }
        let Some(p) = self.asides.pending.clone() else { return };
        if t < p.at {
            return;
        }
        // past its moment it is never said (a late frame included)
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

    /// Say the line for `p`: the subject's or the use's, never one said
    /// before (rule 2); remembered whether or not it can be heard (no
    /// audio: the run goes on as if it had been)
    fn say_aside(&mut self, p: &Pending) {
        let what = format!("{}:{}", p.use_.name(), p.subject);
        let Some(line) = choose(self.aside_lines(), p.use_, &p.subject, self.level_num, self.run_seed, &self.asides.memory.said) else {
            self.aside_note(format!("skip:{what}:spent"));
            return;
        };
        let t = self.perf_t;
        self.asides.memory.said.push(line.id.clone());
        self.asides.last = Some((t, p.use_.priority()));
        // (no audio, or no WAV: as long as a short line)
        let secs = self.play_voice_line(&line, ASIDE_GAIN).unwrap_or(1.5);
        self.asides.voice_until = t + secs;
        self.aside_note(format!("say:{what}:{}", line.id));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
            ln("si1", "sighted", "", 1, 99),
            ln("n1", "nerve", "", 1, 99),
            ln("cry", "cry", "", 1, 99),
        ]
    }

    #[test]
    fn the_subjects_lines_come_before_the_uses_general_ones() {
        let d = deck();
        let a = choose(&d, Use::Arrive, "well", 1, 7, &[]).unwrap();
        assert!(a.subject == "well", "{a:?}");
        // the subject's spent: the general ones (none on level 1 here)
        let said = vec!["w1".to_string(), "w2".to_string()];
        assert!(choose(&d, Use::Arrive, "well", 1, 7, &said).is_none());
        // a subject with no lines of its own: the general ones
        let b = choose(&d, Use::Startle, "scream", 5, 7, &[]).unwrap();
        assert!(b.subject.is_empty(), "{b:?}");
        let c = choose(&d, Use::Startle, "bang", 5, 7, &[]).unwrap();
        assert_eq!(c.id, "s1");
        // never another persona's, another use's or another level's
        assert!(choose(&d, Use::Arrive, "", 2, 7, &[]).is_none());
        assert!(choose(&d, Use::Dark, "", 5, 7, &[]).is_none());
    }

    #[test]
    fn a_line_is_never_said_twice_and_a_spent_use_is_silent() {
        let d = deck();
        let mut said: Vec<String> = vec![];
        while let Some(l) = choose(&d, Use::Startle, "", 5, 99, &said) {
            assert!(!said.contains(&l.id), "{} twice", l.id);
            said.push(l.id);
        }
        assert_eq!(said.len(), 4, "every general startle once: {said:?}");
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
    }

    #[test]
    fn the_chance_falls_with_each_line_said() {
        for u in USES {
            let c: Vec<f64> = (0..4).map(|k| chance(u, 4242, k)).collect();
            assert!(c.windows(2).all(|w| w[1] < w[0]), "{u:?}: {c:?}");
            assert!(c[0] <= 1.0 && c[3] < 0.5 * c[0], "{u:?}: {c:?}");
        }
        assert!(key(Use::Startle, "", 5).is_none(), "no subject: weighed every time");
        assert_eq!(key(Use::Startle, "bang", 5).as_deref(), Some("startle:bang"));
        assert_eq!(key(Use::Arrive, "rooms", 5).as_deref(), Some("arrive:5"), "a floor's arrival once a floor");
    }

    /// A bare game on a floor of the rooms, playing, with the deck above
    fn bare(level: i32) -> Game {
        let mut g = Game::new_bare();
        g.state = "playing".into();
        g.run_seed = 4242;
        g.level_num = level;
        g.perf_t = 100.0;
        g.asides.lines = Some(deck());
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

    /// Events of `u` until one is answered (each its own roll); the log's
    /// last word for it
    fn answer(g: &mut Game, u: Use) -> bool {
        for _ in 0..40 {
            g.aside(u, "");
            if g.asides.pending.is_some() {
                return true;
            }
        }
        false
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
        assert!(g.asides.memory.weighed.contains(&"arrive:1".to_string()));
        // a retry of the floor arrives again, and is not weighed again
        g.asides_arrive("well");
        g.player.pos.x += 3.0;
        step(&mut g, ARRIVAL_AFTER + 30.0, true);
        assert_eq!(said(&g).len(), 1, "{:?}", g.asides.log);
    }

    #[test]
    fn an_arrival_walked_after_its_first_minute_is_not_weighed() {
        let mut g = bare(1);
        g.asides_arrive("well");
        step(&mut g, ARRIVAL_LATEST + 1.0, true);
        g.player.pos.x += 3.0;
        step(&mut g, 5.0, true);
        assert!(g.asides.memory.events.is_empty() && said(&g).is_empty(), "{:?}", g.asides.log);
    }

    #[test]
    fn you_keep_the_place_s_silence() {
        let mut g = bare(5);
        step(&mut g, 1.0, true);
        g.amb_hush = HUSHED;
        assert_eq!(g.holding_breath(), Some("hushed"));
        assert!(answer(&mut g, Use::Startle));
        step(&mut g, 3.0, true);
        assert!(said(&g).is_empty() && g.asides.log.last().is_some_and(|(_, w)| w.ends_with(":hushed")), "{:?}", g.asides.log);
        g.amb_hush = HUSHED - 0.05;
        assert_eq!(g.holding_breath(), None);
    }

    #[test]
    fn nothing_is_said_to_an_empty_chair() {
        let mut g = bare(5);
        assert!(answer(&mut g, Use::Startle));
        step(&mut g, 4.0, false);
        assert!(said(&g).is_empty(), "{:?}", g.asides.log);
        assert!(g.asides.log.iter().any(|(_, w)| w.ends_with(":away")), "{:?}", g.asides.log);
    }

    #[test]
    fn a_reaction_is_said_in_its_moment_and_the_next_waits_its_gap() {
        let mut g = bare(5);
        step(&mut g, 1.0, true);
        assert!(answer(&mut g, Use::Startle));
        let (lo, _, win) = Use::Startle.moment();
        let t0 = g.perf_t;
        step(&mut g, win + 0.5, true);
        let at = g.asides.last.expect("said").0;
        assert!(at - t0 >= lo - 1e-9 && at - t0 <= win, "said {:.2} s after", at - t0);
        // another within the gap is let go after its moment
        assert!(answer(&mut g, Use::Startle));
        step(&mut g, win + 0.5, true);
        assert_eq!(said(&g).len(), 1, "{:?}", g.asides.log);
        assert!(g.asides.log.last().is_some_and(|(_, w)| w.ends_with(":gap")), "{:?}", g.asides.log);
        // past the gap, the next is said, and never the same line
        step(&mut g, GAP, true);
        assert!(answer(&mut g, Use::Startle));
        step(&mut g, win + 0.5, true);
        let s = said(&g);
        assert_eq!(s.len(), 2, "{:?}", g.asides.log);
        assert_ne!(s[0], s[1]);
    }

    #[test]
    fn you_hold_your_breath_after_your_cry_and_a_blow_is_not_held() {
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
        // a blow's grunt is said through it
        g.cry_last_at = Some(g.perf_t);
        assert!(answer(&mut g, Use::Knocked));
        step(&mut g, 2.0, true);
        assert!(g.asides.log.last().is_some_and(|(_, w)| w == "say:knocked::k1"), "{:?}", g.asides.log);
    }

    #[test]
    fn an_event_with_no_line_here_is_not_weighed() {
        // the deck's only arrivals for level 2: none; its startles: from level 1
        let mut g = bare(2);
        g.aside(Use::Arrive, "rooms");
        g.aside(Use::Knocked, "");
        assert!(g.asides.memory.weighed.is_empty() && g.asides.memory.events.is_empty(), "{:?}", g.asides.memory);
        g.aside(Use::Startle, "bang");
        assert_eq!(g.asides.memory.weighed, vec!["startle:bang".to_string()]);
    }

    #[test]
    fn a_subject_is_weighed_once_a_run_and_a_better_reaction_replaces_a_lesser() {
        let mut g = bare(5);
        g.aside(Use::Startle, "bang");
        g.aside(Use::Startle, "bang");
        assert_eq!(g.asides.memory.events.get("startle"), Some(&1), "the second bang is not weighed");
        assert_eq!(g.asides.memory.weighed, vec!["startle:bang".to_string()]);
        // a lesser one waiting gives way to a better
        g.asides.pending = Some(Pending { use_: Use::Arrive, subject: "rooms".into(), at: 0.0, until: 1e9 });
        assert!(answer(&mut g, Use::Sighted));
        assert_eq!(g.asides.pending.as_ref().map(|p| p.use_), Some(Use::Sighted));
        // and a lesser one does not replace it
        g.aside(Use::Nerve, "");
        assert_eq!(g.asides.pending.as_ref().map(|p| p.use_), Some(Use::Sighted));
    }

    #[test]
    fn the_memory_goes_through_the_save_and_back() {
        let mut m = AsideMemory::default();
        m.said = vec!["a1".into(), "s3".into()];
        m.weighed = vec!["arrive:4".into(), "startle:bang".into()];
        m.events.insert("startle".into(), 3);
        let back = AsideMemory::from_json(Some(&m.to_json()));
        assert_eq!(back, m);
        assert_eq!(AsideMemory::from_json(None), AsideMemory::default());
        let odd = serde_json::json!({"said": [1, "x"], "events": {"nonsense": 4, "dark": 2}});
        let o = AsideMemory::from_json(Some(&odd));
        assert_eq!(o.said, vec!["x".to_string()]);
        assert_eq!(o.events.len(), 1, "an unknown use is dropped");
    }
}
