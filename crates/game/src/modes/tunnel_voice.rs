//! Your voice on the way in (`docs/design/WAY_IN.md`, *Your voice*;
//! `docs/design/VOICES.md`, *Your voice on the way in*): what you know of
//! her, told to yourself as you walk the tunnel, and a few words at what
//! you pass. The persona `you_tunnel`, its lines `use = "way_in"`, each line
//! said once at most:
//!
//! - `look`: in the look into the tunnel (`tunnel_in`), at times of its
//!   clock; a line whose time a skip has passed is let go;
//! - `walk`: once your hands are back, in order, each after the last has
//!   finished and a pause;
//! - the reactions (`long`, `light`, `back`, `quiet`, `locked`, `keep`,
//!   `door`, `note`, `taken`): once each, when what they answer happens and
//!   nothing else is being said; the next `walk` line waits for them.
//!
//! One at a time; nothing on the screen; nothing hears it; it stops at the
//! door. No draw from any stream: its times are the way in's own clock.
use crate::audio::voice;
use crate::prelude::*;

/// The persona and the use its lines are under.
pub const PERSONA: &str = "you_tunnel";
pub const USE: &str = "way_in";
/// The subjects the way in says lines about (the line files' `subject`).
pub const SUBJECTS: [&str; 11] = ["look", "walk", "long", "light", "back", "quiet", "locked", "keep", "door", "note", "taken"];
/// How loud, in the mix (its WAVs at -19 LUFS; the asides' 0.7)
pub const GAIN: f64 = 0.8;
/// The look's lines, at times of its clock (s): on the street, at the
/// stairs' head, on the stairs, at the arch.
const LOOK_AT: [(&str, f64); 4] = [("look_1", 1.0), ("look_2", 6.8), ("look_3", 12.6), ("look_4", 20.4)];
/// After your hands are back, before the first `walk` line
const FIRST_WALK: f64 = 1.6;
/// Between one line's end and the next `walk` line
const GAP: f64 = 2.4;
/// Between one line's end and a reaction
const REACT_GAP: f64 = 0.6;
/// When the cars behind the glass have stopped (the way in's clock, s):
/// fewer from a minute in, none from here (`modes::tunnel`'s sound)
pub const CARS_STOP: f64 = 95.0;
/// All told and no door yet: this long quiet before `keep`
const KEEP_AFTER: f64 = 16.0;
/// How long a line is taken to last when it cannot be heard (no sound, no
/// WAV): its words at a slow walk's pace
fn unheard_length(text: &str) -> f64 {
    0.6 + 0.36 * text.split_whitespace().count() as f64
}

/// What the way in knows each frame, for the voice.
#[derive(Clone, Copy, Debug, Default)]
pub struct Seen {
    /// the look's clock, while it plays
    pub look: Option<f64>,
    /// your hands are yours (the look over)
    pub hands: bool,
    /// metres walked in, and back toward the arch since your deepest
    pub along: f64,
    pub back: f64,
    /// the way back has begun to come round
    pub closed: bool,
    /// the module under your feet (along the bore, the ring's own index)
    pub module: i64,
    /// the dead bar's module stands there
    pub dead_bar: bool,
    pub tried: bool,
    pub changed: bool,
    /// the open door is in sight, near
    pub door_seen: bool,
    pub near_note: bool,
    pub note_taken: bool,
}

/// The voice's state while you are on the way in.
#[derive(Clone, Debug, Default)]
pub struct TunnelVoice {
    /// the way in's clock (s)
    pub clock: f64,
    /// what was said, in order (ids); what a skip let go
    pub said: Vec<&'static str>,
    pub let_go: Vec<&'static str>,
    /// when what is being said ends (the clock's)
    until: f64,
    /// when your hands came back
    hands_at: Option<f64>,
    next_walk: usize,
    /// the dead bar passed, and the module last under you
    light_passes: u32,
    last_module: i64,
    taken_at: Option<f64>,
}

impl TunnelVoice {
    fn done(&self, id: &str) -> bool {
        self.said.contains(&id) || self.let_go.contains(&id)
    }

    /// The line to say now, if any (and what a skip let go), given what is
    /// seen; the clock moved on by `dt`.
    pub fn next(&mut self, dt: f64, s: &Seen) -> Option<&'static str> {
        self.clock += dt;
        if s.module != self.last_module {
            // (on into a module of the dead bar: a pass; carried back a
            // period, or walking back, is none)
            if s.dead_bar && s.module > self.last_module {
                self.light_passes += 1;
            }
            self.last_module = s.module;
        }
        if s.note_taken && self.taken_at.is_none() {
            self.taken_at = Some(self.clock);
        }
        let quiet = self.clock >= self.until;
        // the look's lines, on its clock; one whose time a skip has passed is let go
        if let Some(lt) = s.look {
            for (id, at) in LOOK_AT {
                if !self.done(id) && lt >= at {
                    if quiet && lt < at + 2.5 {
                        return Some(id);
                    }
                    if lt >= at + 2.5 {
                        self.let_go.push(id);
                    }
                }
            }
            return None;
        }
        if !s.hands {
            return None;
        }
        for (id, _) in LOOK_AT {
            if !self.done(id) {
                self.let_go.push(id);
            }
        }
        let hands_at = *self.hands_at.get_or_insert(self.clock);
        // the reactions, the strongest first, each once, when nothing is said
        let reactions: [(&'static str, bool); 9] = [
            ("taken", self.taken_at.is_some_and(|t| self.clock - t > 1.2)),
            ("note", s.near_note && !s.note_taken),
            ("door", s.door_seen && !s.note_taken),
            ("locked", s.tried && !s.changed),
            ("back", s.closed && s.back > 40.0),
            ("long", s.along >= 485.0),
            ("light", self.light_passes >= 3),
            ("quiet", self.clock >= CARS_STOP + 4.0),
            ("keep", self.next_walk >= WALK.len() && !s.changed && self.clock - self.until > KEEP_AFTER),
        ];
        if self.clock >= self.until + REACT_GAP
            && let Some((id, _)) = reactions.iter().find(|(id, due)| *due && !self.done(id))
        {
            return Some(id);
        }
        // her note taken: the rest is not said
        if s.note_taken {
            return None;
        }
        if self.next_walk < WALK.len() && self.clock >= hands_at + FIRST_WALK && self.clock >= self.until + GAP {
            let id = WALK[self.next_walk];
            return Some(id);
        }
        None
    }

    /// `id` said, lasting `len` seconds.
    pub fn said(&mut self, id: &'static str, len: f64) {
        self.said.push(id);
        self.until = self.clock + len;
        if WALK.get(self.next_walk) == Some(&id) {
            self.next_walk += 1;
        }
    }
}

/// The walk's lines, in order.
const WALK: [&str; 13] = [
    "walk_01", "walk_02", "walk_03", "walk_04", "walk_05", "walk_06", "walk_07", "walk_08", "walk_09", "walk_10", "walk_11", "walk_12",
    "walk_13",
];

impl Game {
    /// Say `id` of the way in's voice, if there is such a line (its WAV,
    /// or as long as its words when it cannot be heard): its length.
    pub(crate) fn tunnel_say(&mut self, id: &str) -> f64 {
        let Some(line) = voice::index().iter().find(|l| l.persona == PERSONA && l.id == id).cloned() else {
            return 0.0;
        };
        self.play_voice_line(&line, GAIN).unwrap_or_else(|| unheard_length(&line.text))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seen() -> Seen {
        Seen { hands: true, ..Default::default() }
    }

    /// Run the voice for `secs` at 30 frames a second with `s`, saying
    /// each line for `len` seconds.
    fn run(v: &mut TunnelVoice, s: &Seen, secs: f64, len: f64) {
        for _ in 0..(secs * 30.0) as usize {
            if let Some(id) = v.next(1.0 / 30.0, s) {
                v.said(id, len);
            }
        }
    }

    #[test]
    fn the_look_then_the_walk_in_order_one_at_a_time() {
        let mut v = TunnelVoice::default();
        // the look, on its clock
        for f in 0..(30.0 * 29.0) as usize {
            let s = Seen { look: Some(f as f64 / 30.0), ..Default::default() };
            if let Some(id) = v.next(1.0 / 30.0, &s) {
                v.said(id, 3.5);
            }
        }
        assert_eq!(v.said, ["look_1", "look_2", "look_3", "look_4"]);
        run(&mut v, &seen(), 140.0, 4.0);
        let walk: Vec<&str> = v.said.iter().copied().filter(|i| i.starts_with("walk")).collect();
        assert_eq!(walk, WALK);
        // the cars' stop, said once among them, never over one
        assert_eq!(v.said.iter().filter(|i| **i == "quiet").count(), 1);
        // and then, all told with no door, once: keep walking
        assert_eq!(v.said.iter().filter(|i| **i == "keep").count(), 1);
    }

    #[test]
    fn a_skipped_look_lets_its_lines_go() {
        let mut v = TunnelVoice::default();
        for f in 0..(30.0 * 3.0) as usize {
            let s = Seen { look: Some(f as f64 / 30.0), ..Default::default() };
            if let Some(id) = v.next(1.0 / 30.0, &s) {
                v.said(id, 3.0);
            }
        }
        // skipped at 3 s: hands back at once
        run(&mut v, &seen(), 10.0, 3.0);
        assert_eq!(v.said[0], "look_1");
        assert_eq!(v.let_go, ["look_2", "look_3", "look_4"]);
        assert_eq!(v.said[1], "walk_01");
    }

    #[test]
    fn a_reaction_is_said_once_and_waits_for_the_line_before_it() {
        let mut v = TunnelVoice::default();
        run(&mut v, &seen(), 4.0, 5.0);
        assert_eq!(v.said, ["walk_01"]);
        // the shut door tried while walk_01 is said: it waits, then is said, once
        let s = Seen { tried: true, ..seen() };
        let before = v.clock;
        run(&mut v, &s, 30.0, 2.0);
        let at = v.said.iter().position(|i| *i == "locked").expect("locked said");
        assert_eq!(at, 1, "{:?}", v.said);
        assert!(v.clock > before);
        assert_eq!(v.said.iter().filter(|i| **i == "locked").count(), 1);
        // the dead bar passed three times
        let mut v = TunnelVoice::default();
        for m in 1..=70 {
            let s = Seen { module: m, dead_bar: m % 20 == 4, ..seen() };
            run(&mut v, &s, 0.2, 0.1);
        }
        assert!(v.said.contains(&"light"));
        // her note taken: the walk stops, the last word is said
        let mut v = TunnelVoice::default();
        run(&mut v, &Seen { note_taken: true, ..seen() }, 5.0, 1.0);
        assert_eq!(v.said, ["taken"]);
    }
}
