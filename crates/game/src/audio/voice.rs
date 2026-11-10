//! Voice lines (`docs/VOICES.md`): spoken lines made by text-to-speech
//! models from a persona's voice (`cargo xtask voice`), embedded as 16-bit
//! WAVs (`assets/voice/<persona>/<line>.wav`) with an index of what each is
//! for (`assets/voice/lines.tsv`, written by `cargo xtask voice write`).
//!
//! The first persona is you: the cry for help (`H`, `Game::cry_for_help`).
//! Which line a cry is chosen from a stream of its own (the run's seed, the
//! level and how many times you have cried), never the per-frame stream: a
//! cry draws nothing that later frames see (`CLAUDE.md`, invariant 1).

use crate::base::rng::SplitMix64;

/// A line the game can say, as the index has it
#[derive(Clone, Debug, PartialEq)]
pub struct Line {
    pub persona: String,
    pub id: String,
    /// Where the game says it (`cry`, or one of the asides' uses)
    pub use_: String,
    /// What it is said about within its use (a place, a kind of thing), or
    /// empty: said about anything of its use
    pub subject: String,
    /// The first and last level it may be said on
    pub first: i32,
    pub last: i32,
    /// 0 calm … 1 a scream, within its emotion
    pub intensity: f64,
    pub emotion: String,
    pub text: String,
}

static INDEX: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/voice/lines.tsv"));

macro_rules! wav {
    ($p:literal, $id:literal) => {
        ($p, $id, include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/voice/", $p, "/", $id, ".wav")) as &[u8])
    };
}

/// Every embedded line's WAV (a test holds this list and the index together)
pub static WAVS: &[(&str, &str, &[u8])] = &[
    wav!("you", "answer_me"),
    wav!("you", "can_anyone_hear"),
    wav!("you", "get_me_out"),
    wav!("you", "hello_anyone"),
    wav!("you", "help_me"),
    wav!("you", "help_scream"),
    wav!("you", "i_cant"),
    wav!("you", "im_here"),
    wav!("you", "please_help_hoarse"),
    wav!("you", "somebody_help"),
    wav!("you_aside", "aftermath_breathe"),
    wav!("you_aside", "aftermath_gone"),
    wav!("you_aside", "aftermath_heart"),
    wav!("you_aside", "answered_hatch"),
    wav!("you_aside", "answered_note"),
    wav!("you_aside", "arrive_deep_how_far"),
    wav!("you_aside", "arrive_deep_nobody"),
    wav!("you_aside", "arrive_deep_tired"),
    wav!("you_aside", "arrive_halls_should"),
    wav!("you_aside", "arrive_halls_walls"),
    wav!("you_aside", "arrive_house_lives"),
    wav!("you_aside", "arrive_lobby_hum"),
    wav!("you_aside", "arrive_lobby_warm"),
    wav!("you_aside", "arrive_lobby_yellow"),
    wav!("you_aside", "arrive_okay"),
    wav!("you_aside", "arrive_rooms_again"),
    wav!("you_aside", "arrive_rooms_hate"),
    wav!("you_aside", "arrive_rooms_last"),
    wav!("you_aside", "arrive_showroom_music"),
    wav!("you_aside", "arrive_stairwell"),
    wav!("you_aside", "arrive_well_keeps_going"),
    wav!("you_aside", "arrive_well_look"),
    wav!("you_aside", "arrive_well_real"),
    wav!("you_aside", "arrive_where_now"),
    wav!("you_aside", "arrive_works_ceiling"),
    wav!("you_aside", "arrive_works_for"),
    wav!("you_aside", "dark_lamp"),
    wav!("you_aside", "dark_so"),
    wav!("you_aside", "found_whose"),
    wav!("you_aside", "held_clear_legs"),
    wav!("you_aside", "held_knock_door"),
    wav!("you_aside", "knocked_breathing"),
    wav!("you_aside", "knocked_get_up"),
    wav!("you_aside", "nerve_lose"),
    wav!("you_aside", "nerve_together"),
    wav!("you_aside", "sighted_corner"),
    wav!("you_aside", "sighted_hound"),
    wav!("you_aside", "sighted_oh_no"),
    wav!("you_aside", "sighted_please_no"),
    wav!("you_aside", "sighted_reader"),
    wav!("you_aside", "siren_for"),
    wav!("you_aside", "siren_is"),
    wav!("you_aside", "startle_bang"),
    wav!("you_aside", "startle_dont_like"),
    wav!("you_aside", "startle_laugh"),
    wav!("you_aside", "startle_moan"),
    wav!("you_aside", "startle_scream"),
    wav!("you_aside", "startle_steps"),
    wav!("you_aside", "startle_what"),
    wav!("you_aside", "uncanny_candle"),
    wav!("you_aside", "uncanny_figure"),
    wav!("you_aside", "uncanny_flap"),
    wav!("you_aside", "uncanny_hole"),
    wav!("you_aside", "uncanny_knock_back"),
    wav!("you_aside", "uncanny_lift"),
    wav!("you_aside", "uncanny_pane"),
];

/// The lines of the index, in its order
pub fn lines() -> Vec<Line> {
    INDEX
        .lines()
        .skip(1)
        .filter_map(|r| {
            let f: Vec<&str> = r.split('\t').collect();
            (f.len() == 9).then(|| Line {
                persona: f[0].into(),
                id: f[1].into(),
                use_: f[2].into(),
                subject: f[3].into(),
                first: f[4].parse().unwrap_or(1),
                last: f[5].parse().unwrap_or(99),
                intensity: f[6].parse().unwrap_or(0.5),
                emotion: f[7].into(),
                text: f[8].into(),
            })
        })
        .collect()
}

/// The game's index, read once (the voice director looks in it often)
pub fn index() -> &'static [Line] {
    static INDEX_LINES: std::sync::OnceLock<Vec<Line>> = std::sync::OnceLock::new();
    INDEX_LINES.get_or_init(lines)
}

/// A line's WAV
pub fn wav(persona: &str, id: &str) -> Option<&'static [u8]> {
    WAVS.iter().find(|(p, i, _)| *p == persona && *i == id).map(|w| w.2)
}

/// A mono WAV's samples and rate (16-bit PCM, or 32-bit float)
pub fn decode(bytes: &[u8]) -> Option<(f64, Vec<f32>)> {
    let u16le = |b: &[u8], i: usize| u16::from_le_bytes([b[i], b[i + 1]]);
    let u32le = |b: &[u8], i: usize| u32::from_le_bytes([b[i], b[i + 1], b[i + 2], b[i + 3]]);
    if bytes.len() < 12 || &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return None;
    }
    let (mut fmt, mut rate, mut bits, mut channels) = (0u16, 0u32, 0u16, 0u16);
    let mut i = 12;
    while i + 8 <= bytes.len() {
        let (id, len) = (&bytes[i..i + 4], u32le(bytes, i + 4) as usize);
        let body = i + 8;
        if body + len > bytes.len() {
            return None;
        }
        if id == b"fmt " && len >= 16 {
            fmt = u16le(bytes, body);
            channels = u16le(bytes, body + 2);
            rate = u32le(bytes, body + 4);
            bits = u16le(bytes, body + 14);
        } else if id == b"data" {
            let d = &bytes[body..body + len];
            let ch = channels.max(1) as usize;
            let samples: Vec<f32> = match (fmt, bits) {
                (1, 16) => d.chunks_exact(2 * ch).map(|c| i16::from_le_bytes([c[0], c[1]]) as f32 / 32768.0).collect(),
                (3, 32) => d.chunks_exact(4 * ch).map(|c| f32::from_le_bytes([c[0], c[1], c[2], c[3]])).collect(),
                _ => return None,
            };
            return (rate > 0).then_some((rate as f64, samples));
        }
        i = body + len + (len & 1);
    }
    None
}

/// A cry soon after the last (seconds): it is the loudest he has
pub const CRY_AGAIN: f64 = 45.0;
/// Whose lines a cry is: yours
pub const CRY_PERSONA: &str = "you";
const CRY_SALT: u64 = 0xC4_7F09_2B51_D3E1;

/// The line a cry is, on `level`: the loudest one for the level if you
/// cried less than `CRY_AGAIN` ago; otherwise one of the others for the
/// level, a louder one from the second cry on a floor, never the one said
/// last. Chosen by the run's seed, the level and the cry's number in the
/// run (`nth`), from a stream of its own.
pub fn choose_cry(level: i32, run_seed: u32, nth: u32, on_floor: u32, since_last: Option<f64>, last: Option<&str>) -> Option<Line> {
    let mut here: Vec<Line> =
        lines().into_iter().filter(|l| l.persona == CRY_PERSONA && l.use_ == "cry" && l.first <= level && level <= l.last).collect();
    if here.is_empty() {
        return None;
    }
    here.sort_by(|a, b| a.intensity.total_cmp(&b.intensity).then(a.id.cmp(&b.id)));
    if since_last.is_some_and(|s| s < CRY_AGAIN) {
        return here.last().cloned();
    }
    let top = here.last().map(|l| l.intensity).unwrap_or(1.0);
    let mut pool: Vec<Line> = here.iter().filter(|l| l.intensity < top && last != Some(l.id.as_str())).cloned().collect();
    if on_floor >= 1 && pool.len() > 1 {
        // from the second cry on a floor, the louder half
        let mid = pool[pool.len() / 2].intensity;
        pool.retain(|l| l.intensity >= mid);
    }
    if pool.is_empty() {
        pool = here;
    }
    let mut r = SplitMix64::new(run_seed ^ (level as u32).wrapping_mul(0x9E37_79B9), CRY_SALT.wrapping_add(nth as u64));
    let i = ((r.next() * pool.len() as f64) as usize).min(pool.len() - 1);
    pool.get(i).cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_line_of_the_index_has_its_wav_and_every_wav_its_line() {
        let ls = lines();
        assert!(!ls.is_empty(), "the index is empty");
        for l in &ls {
            let bytes = wav(&l.persona, &l.id).unwrap_or_else(|| panic!("{}/{}: in lines.tsv but not in WAVS", l.persona, l.id));
            let (rate, s) = decode(bytes).unwrap_or_else(|| panic!("{}/{}: not a WAV the game reads", l.persona, l.id));
            assert_eq!(rate, 48000.0, "{}/{}", l.persona, l.id);
            let secs = s.len() as f64 / rate;
            assert!((0.3..8.0).contains(&secs), "{}/{}: {secs:.2} s", l.persona, l.id);
            let peak = s.iter().fold(0f32, |m, v| m.max(v.abs()));
            assert!(peak > 0.1 && peak <= 1.0, "{}/{}: peak {peak}", l.persona, l.id);
        }
        for (p, id, _) in WAVS {
            assert!(ls.iter().any(|l| l.persona == *p && l.id == *id), "{p}/{id}: in WAVS but not in lines.tsv");
        }
    }

    /// Every line of the index is one the game plays: your cry, or one of
    /// your asides' uses (`audio::asides::Use`), said by its persona
    #[test]
    fn every_line_of_the_index_is_one_the_game_plays() {
        use crate::audio::asides::{Use, PERSONA};
        for l in lines() {
            let ok = match l.persona.as_str() {
                CRY_PERSONA => l.use_ == "cry",
                PERSONA => Use::from_name(&l.use_).is_some(),
                _ => false,
            };
            assert!(ok, "{}/{}: use {} is not one the game plays for {}", l.persona, l.id, l.use_, l.persona);
            // a subject is one the game posts for its use (a line about
            // anything else would never be said)
            if let Some(u) = Use::from_name(&l.use_).filter(|_| !l.subject.is_empty()) {
                assert!(
                    u.subjects().contains(&l.subject.as_str()),
                    "{}: subject {} is not one the game posts for {}",
                    l.id,
                    l.subject,
                    l.use_
                );
            }
        }
    }

    /// Rule 1 of your asides: they react and never explain. No line names
    /// what the canon keeps unsaid: a creature, a place by its name, a
    /// level or its number, the way out, the people of the notes, the
    /// self-assembly (`docs/VOICES.md`, *Your asides*; `lore-keeper` reads
    /// the rest)
    #[test]
    fn no_aside_names_what_the_canon_keeps_unsaid() {
        const UNSAID: [&str; 20] = [
            "smiler",
            "hound",
            "watcher",
            "reader",
            "the face",
            "level",
            "exit",
            "lobby",
            "the well",
            "the works",
            "stair hall",
            "poolroom",
            "showroom",
            "borrowed",
            "the loop",
            "stairwell",
            "elias",
            "m.",
            "self-assembly",
            "scp",
        ];
        // `w` at the start of a word of `t` ("m." is not the end of "hum.";
        // "hounds" says "hound")
        let says = |t: &str, w: &str| t.match_indices(w).any(|(i, _)| !t[..i].chars().next_back().is_some_and(|c| c.is_alphabetic()));
        assert!(says("ask m. why", "m.") && !says("that hum.", "m.") && says("the hounds", "hound"));
        for l in lines().into_iter().filter(|l| l.persona == crate::audio::asides::PERSONA) {
            let t = l.text.to_lowercase();
            for w in UNSAID {
                assert!(!says(&t, w), "{}: \"{}\" says {w:?}", l.id, l.text);
            }
            assert!(!t.chars().any(|c| c.is_ascii_digit()), "{}: \"{}\" says a number", l.id, l.text);
            assert!(t.split_whitespace().count() <= 8, "{}: \"{}\" is a speech, not an aside", l.id, l.text);
        }
    }

    #[test]
    fn a_wav_decodes_and_a_broken_one_does_not() {
        let (rate, s) = decode(wav("you", "help_me").unwrap()).unwrap();
        assert_eq!(rate, 48000.0);
        assert!(s.len() > 4800);
        assert!(decode(b"RIFF\0\0\0\0WAVE").is_none());
        assert!(decode(b"not a wav at all").is_none());
    }

    #[test]
    fn every_level_has_a_cry_and_the_choice_is_fixed_by_the_run() {
        for level in 1..=40 {
            let a = choose_cry(level, 4242, 3, 0, None, None).unwrap_or_else(|| panic!("no cry on level {level}"));
            assert!(a.first <= level && level <= a.last);
            assert_eq!(Some(a.clone()), choose_cry(level, 4242, 3, 0, None, None), "the same run, the same cry");
            assert!(a.intensity < 1.0, "a first cry is not the scream");
            // cried again soon after: the loudest of yours for the level
            let again = choose_cry(level, 4242, 4, 1, Some(10.0), Some(&a.id)).unwrap();
            let loudest = lines()
                .into_iter()
                .filter(|l| l.persona == CRY_PERSONA && l.use_ == "cry" && l.first <= level && level <= l.last)
                .map(|l| l.intensity)
                .fold(0.0, f64::max);
            assert_eq!(again.intensity, loudest, "level {level}: {}", again.id);
            assert_eq!(again.persona, CRY_PERSONA);
            // never the one said last
            let next = choose_cry(level, 4242, 4, 1, Some(120.0), Some(&a.id)).unwrap();
            assert_ne!(next.id, a.id, "level {level}");
        }
        // the Building calls out; deep down he is worn out
        let first = |level| choose_cry(level, 7, 0, 0, None, None).unwrap().id;
        assert!(["hello_anyone", "can_anyone_hear", "answer_me", "im_here"].contains(&first(1).as_str()), "{}", first(1));
        let deep: Vec<String> = (0..20).map(|n| choose_cry(12, 7, n, 0, None, None).unwrap().id).collect();
        assert!(deep.iter().all(|id| id != "hello_anyone"), "{deep:?}");
    }
}
