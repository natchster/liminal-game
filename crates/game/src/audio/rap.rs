//! A rap on a steel door, part by part: one table that both knocks read,
//! `sys::sfx_knock` (yours at a sealed door, a resident's raps back) and
//! `Soundscape::knock` (what knocks at your cell; the Stairwell's knocking
//! far above). Each had built its rap by hand, and they drifted: milestone
//! 12 retuned one, and 13 had to give the other what 12 gave it (code
//! review 4). Each keeps its own draws, in its own order, and its own
//! routing; what a rap is made of is here, in the order it is played.
//!
//! A rap is the knuckles' thud and the leaf's body under it, and above the
//! lamps' ballast (its 120 Hz series stops at 1.08 kHz) the knuckle's hard
//! click on the steel and the leaf's lowest rings, so a knock is heard as a
//! knock beside a lamp too (`.claude/rules/audio.md`; `tests/listen.rs`,
//! `a_knock_is_heard_beside_a_lamp`, `what_knocks_is_heard_going`).

/// One part of a rap
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Part {
    /// noise through a filter of this type (its `freq` and `q`), or a sine
    /// (`None`) at `freq`
    pub noise: Option<&'static str>,
    /// Hz: the filter's, or the sine's
    pub freq: f64,
    /// the sine's glide down to (Hz), if it falls
    pub to: Option<f64>,
    /// the filter's Q (a sine has none)
    pub q: f64,
    /// its share of the knock's gain
    pub share: f64,
    /// how long it sounds (s)
    pub dur: f64,
}

/// The knuckles' thud on the steel, lowpassed
const THUD: Part = Part { noise: Some("lowpass"), freq: 380.0, to: None, q: 1.4, share: 1.0, dur: 0.12 };
/// The leaf's body under it, a sine falling
const BODY: Part = Part { noise: None, freq: 95.0, to: Some(58.0), q: 0.0, share: 0.6, dur: 0.14 };
/// The knuckle's hard click on the steel
const CLICK: Part = Part { noise: Some("bandpass"), freq: 2300.0, to: None, q: 1.1, share: 0.6, dur: 0.035 };
/// The leaf's lowest ring, and its second
const RING: Part = Part { noise: None, freq: 1310.0, to: None, q: 0.0, share: 0.09, dur: 0.22 };
const RING2: Part = Part { noise: None, freq: 2090.0, to: None, q: 0.0, share: 0.05, dur: 0.14 };

/// Your knuckles on a sealed door (`sfx_knock`)
pub const NEAR: [Part; 5] = [THUD, BODY, CLICK, RING, RING2];
/// A rap from the other side of a door (`sfx_knock`, behind): duller, its
/// click lower, its second ring fainter
pub const BEHIND: [Part; 5] =
    [Part { freq: 260.0, ..THUD }, Part { freq: 82.0, ..BODY }, Part { freq: 1700.0, ..CLICK }, RING, Part { share: 0.03, ..RING2 }];
/// What knocks at a cell, heard from inside it, and the Stairwell's
/// knocking far above (`Soundscape::knock`): the knuckles' thud as yours,
/// the click as from behind a door and softer, the leaf's one ring
pub const OUTSIDE: [Part; 4] =
    [THUD, Part { to: Some(60.0), ..BODY }, Part { freq: 1700.0, share: 0.5, ..CLICK }, Part { share: 0.08, ..RING }];

#[cfg(test)]
mod tests {
    use super::*;

    /// Every rap is heard beside a lamp: a part above the ballast's series
    /// (1.08 kHz) of a share a knock is heard by, a noise first (the thud),
    /// and the parts of each in the same order (so the same draws: only a
    /// noise draws)
    #[test]
    fn every_rap_carries_over_the_hum() {
        for (name, rap) in [("near", &NEAR[..]), ("behind", &BEHIND[..]), ("outside", &OUTSIDE[..])] {
            assert!(rap.iter().any(|p| p.noise.is_some() && p.freq >= 1200.0 && p.share >= 0.5), "{name}: a click over the hum");
            assert!(rap.iter().any(|p| p.noise.is_none() && p.freq >= 1200.0), "{name}: a ring over the hum");
            let kinds: Vec<bool> = rap.iter().map(|p| p.noise.is_some()).collect();
            assert_eq!(kinds[..4], [true, false, true, false], "{name}: thud, body, click, ring");
        }
    }
}
