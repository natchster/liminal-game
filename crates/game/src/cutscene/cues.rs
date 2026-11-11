//! What a cutscene may name: the sounds it plays from a place and the
//! events it runs, each registered here by name (as Unity's signals keep a
//! timeline from naming gameplay code), and the check that a cutscene names
//! only these (`lint`, part of `bundle_problems`). A new event is a line
//! here and a function on `Game`; what it does must hold on a skip too
//! (`on_skip`), as a skip runs every event not yet run.
use super::model::{Anchor, Cutscene, Look};
use crate::audio::asides::Use;
use crate::base::game::Game;

/// The sounds, by name. Each is played from its cue's place
/// (`Game::play_from`: level by distance, pan by direction, muffled by walls)
pub const SOUNDS: &[(&str, fn(&mut Game))] = &[
    ("breath", |g| g.sfx_breath()),
    ("chime", |g| g.sfx_chime()),
    ("clank", |g| g.sfx_clank()),
    ("slam", |g| g.sfx_door_slam()),
    ("scrape", |g| g.sfx_scrape(0.12)),
    ("scream_far", |g| g.sfx_distant_scream()),
];

/// The events, by name, with what each does
pub const EVENTS: &[(&str, &str, fn(&mut Game))] = &[("mark", "nothing: a mark in the cutscene's log (tests, previews)", |_| {})];

pub fn sound(name: &str) -> Option<fn(&mut Game)> {
    SOUNDS.iter().find(|(n, _)| *n == name).map(|(_, f)| *f)
}

pub fn event(name: &str) -> Option<fn(&mut Game)> {
    EVENTS.iter().find(|(n, _, _)| *n == name).map(|(_, _, f)| *f)
}

/// The subject an aside is posted with, as the asides' director knows it
/// (it takes only its own `&'static` names)
pub fn say_subject(use_: &str, subject: &str) -> Option<(Use, &'static str)> {
    let u = Use::from_name(use_)?;
    if subject.is_empty() {
        return Some((u, ""));
    }
    u.subjects().iter().find(|s| **s == subject).map(|s| (u, *s))
}

/// What a cutscene names that the game does not have
pub fn lint(c: &Cutscene) -> Vec<String> {
    let mut p = vec![];
    for s in &c.sound {
        if sound(&s.name).is_none() {
            let known: Vec<&str> = SOUNDS.iter().map(|(n, _)| *n).collect();
            p.push(format!("sound \"{}\" is not one the game has ({})", s.name, known.join(", ")));
        }
        if s.anchor == Anchor::Killer && c.then != super::model::Then::Card {
            p.push(format!("sound \"{}\": only a death has a killer to play it from", s.name));
        }
    }
    for e in &c.event {
        if event(&e.name).is_none() {
            let known: Vec<&str> = EVENTS.iter().map(|(n, _, _)| *n).collect();
            p.push(format!("event \"{}\" is not one the game has ({}): register it in cutscene::cues", e.name, known.join(", ")));
        }
    }
    for s in &c.say {
        if say_subject(&s.use_, &s.subject).is_none() {
            p.push(format!(
                "say \"{}\" / \"{}\": not a use and subject your asides' director takes (audio::asides::Use::subjects)",
                s.use_, s.subject
            ));
        }
    }
    // the killer only where there is one
    let killer = c.camera.iter().any(|k| k.anchor == Anchor::Killer || k.look == Look::At(Anchor::Killer));
    if killer && c.then != super::model::Then::Card {
        p.push("a camera key names the killer, but only a death has one".into());
    }
    p
}
