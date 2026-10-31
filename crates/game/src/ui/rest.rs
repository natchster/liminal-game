//! The HUD at rest (`docs/UI.md`, design 5): nothing on the screen but the
//! place until something changes. A meter shows while it is not full (and
//! a moment after), the pockets when their counts change, her thread when
//! it is new, and all of it while the glance key (X) is held; what would
//! help right now (water when you are out of breath, a battery when the
//! light is failing) is offered with its key. The words are the building's
//! voice: `calm` sets the model's capitals in sentence case when painted
//! (the model keeps its text, which recorded fixtures pin).
use crate::prelude::*;
use crate::session::input::{key_label, Act};

/// Seconds a change stays on the screen
pub const SHOW_FOR: f64 = 3.0;
/// ... and a new line of hers
pub const THREAD_FOR: f64 = 14.0;

/// What is showing, as seconds left (0: at rest).
#[derive(Clone, Debug, Default)]
pub struct Rest {
    pub stamina: f64,
    pub sanity: f64,
    pub battery: f64,
    pub pockets: f64,
    pub thread: f64,
    /// the glance key is held: everything, with its labels
    pub glance: bool,
    seen: Option<(i32, i32, i32, usize, String, bool, i32)>,
    last_battery: f64,
    last_sanity: f64,
}

impl Game {
    /// Per frame (from `update_hud`): what changed, and what shows
    pub fn update_rest(&mut self, dt: f64) {
        let p = &self.player;
        let now = (p.bottles, p.batteries, p.papers, self.lore_found.len(), self.thread.clone(), p.flashlight_on, p.coins);
        let glance = self.held(Act::Glance);
        let (stamina, sanity, battery, light) = (p.stamina, p.sanity, p.battery, p.flashlight_on);
        let stam_max = self.body.stam_max;
        let r = &mut self.rest;
        // (the first frame of a run only learns where things stand)
        if r.seen.is_none() {
            r.last_battery = battery;
            r.last_sanity = sanity;
        }
        for t in [&mut r.stamina, &mut r.sanity, &mut r.battery, &mut r.pockets, &mut r.thread] {
            *t = (*t - dt).max(0.0);
        }
        if stamina < stam_max - 0.5 {
            r.stamina = r.stamina.max(1.2);
        }
        // nerve shows when it is going, or low
        if sanity < 60.0 || sanity < r.last_sanity - 0.02 {
            r.sanity = r.sanity.max(SHOW_FOR);
        }
        if light || (battery - r.last_battery).abs() > 0.5 {
            r.battery = r.battery.max(if light { 1.5 } else { SHOW_FOR });
        }
        if let Some(seen) = &r.seen {
            if (seen.0, seen.1, seen.2, seen.3, seen.6) != (now.0, now.1, now.2, now.3, now.6) {
                r.pockets = SHOW_FOR + 1.0;
            }
            if seen.4 != now.4 && !now.4.is_empty() {
                r.thread = THREAD_FOR;
            }
        }
        r.last_battery = battery;
        r.last_sanity = sanity;
        r.seen = Some(now);
        r.glance = glance;
    }
    /// What would help right now, with its key ("G", "Drink almond water")
    pub fn pocket_hint(&self) -> Option<(String, &'static str)> {
        let p = &self.player;
        let key = |a: Act| self.settings.keys(a).first().map(|k| key_label(k)).unwrap_or_default();
        if p.bottles > 0 && p.stamina < 30.0 {
            return Some((key(Act::Drink), "Drink almond water"));
        }
        if p.batteries > 0 && p.battery < 20.0 && (p.flashlight_on || p.battery <= 0.0) {
            return Some((key(Act::Battery), "Change the battery"));
        }
        None
    }
    /// The level card: "Level 4" small at the lower left, the floor's line
    /// beneath it, for a few seconds after you arrive
    pub fn show_level_card(&mut self, line: &str) {
        self.level_card = Some((calm(line), self.now_ms));
    }
    /// The pockets as one quiet line ("Almond water 2 · Batteries 1 · Paper 4")
    pub fn pockets_line(&self) -> String {
        let p = &self.player;
        let mut v = vec![];
        if p.bottles > 0 {
            v.push(format!("Almond water {}", p.bottles));
        }
        if p.batteries > 0 {
            v.push(format!("Batteries {}", p.batteries));
        }
        if p.papers > 0 {
            v.push(format!("Paper {}", p.papers));
        }
        if p.coins > 0 {
            v.push(if p.coins == 1 { "A coin".to_string() } else { format!("Coins {}", p.coins) });
        }
        if !self.lore_found.is_empty() || !self.notebook.is_empty() {
            v.push(format!("Pages {}", self.lore_found.len() + self.notebook.len()));
        }
        // (non-breaking spaces: a laid-out line collapses ordinary runs of them)
        v.join("\u{a0}\u{a0}\u{a0}·\u{a0}\u{a0}\u{a0}")
    }
}

/// Words a sign writes in capitals, and so does the screen
const SIGN_WORDS: [&str; 3] = ["EXIT", "STAIRS", "TV"];

/// The model's capitals set as the building would write them: a word in
/// capitals is set in lower case ("ALMOND WATER — SEALED" → "Almond water
/// — sealed"), each sentence starts with a capital, a key in brackets is
/// just its name ("[TAB]" → "Tab"), and initials ("M."), the pronoun I and
/// the words of signs keep theirs. Mixed-case words are left as written.
pub fn calm(s: &str) -> String {
    // the brackets of key names
    let mut t = String::new();
    let mut rest = s;
    while let Some(i) = rest.find('[') {
        let Some(j) = rest[i..].find(']') else { break };
        t.push_str(&rest[..i]);
        let k = &rest[i + 1..i + j];
        t.push_str(&if k.chars().count() <= 1 { k.to_string() } else { key_word(k) });
        rest = &rest[i + j + 1..];
    }
    t.push_str(rest);
    // word by word
    let chars: Vec<char> = t.chars().collect();
    let mut out: Vec<char> = Vec::with_capacity(chars.len());
    let mut initials = vec![]; // where a dot ends an initial (not a sentence)
    let mut i = 0;
    while i < chars.len() {
        if !chars[i].is_alphabetic() {
            out.push(chars[i]);
            i += 1;
            continue;
        }
        let mut j = i;
        while j < chars.len() && (chars[j].is_alphabetic() || chars[j] == '\'' || chars[j] == '\u{2019}') {
            j += 1;
        }
        let w: String = chars[i..j].iter().collect();
        let letters: Vec<char> = w.chars().filter(|c| c.is_alphabetic()).collect();
        let shouting = letters.iter().all(|c| c.is_uppercase());
        let initial = letters.len() == 1 && chars.get(j) == Some(&'.');
        if initial {
            initials.push(out.len() + w.chars().count());
        }
        let lw = w.to_lowercase();
        if !shouting || initial || SIGN_WORDS.contains(&w.as_str()) {
            out.extend(w.chars());
        } else if lw == "i" || lw.starts_with("i'") || lw.starts_with("i\u{2019}") {
            out.push('I');
            out.extend(lw.chars().skip(1));
        } else {
            out.extend(lw.chars());
        }
        i = j;
    }
    // a capital to start each sentence
    let mut cap = true;
    for k in 0..out.len() {
        if out[k].is_alphabetic() {
            if cap {
                let u: Vec<char> = out[k].to_uppercase().collect();
                if u.len() == 1 {
                    out[k] = u[0];
                }
            }
            cap = false;
        } else if out[k].is_numeric() {
            cap = false;
        } else if matches!(out[k], '.' | '!' | '?') && out.get(k + 1) == Some(&' ') && !initials.contains(&k) {
            cap = true;
        }
    }
    out.into_iter().collect()
}

/// A key name in brackets as a word: "TAB" → "Tab", "F3" → "F3"
fn key_word(k: &str) -> String {
    let mut c = k.chars();
    match c.next() {
        Some(f) if k.chars().all(|x| x.is_alphabetic()) => f.to_uppercase().chain(c.flat_map(|x| x.to_lowercase())).collect(),
        _ => k.to_string(),
    }
}

/// A prompt as the screen gives it: the bound key apart, the words calm
/// ("[E] OPEN THE DOOR" → ("E", "Open the door"))
pub fn prompt_parts(prompt: &str, key: &str) -> (String, String) {
    let words = match prompt.strip_prefix('[').and_then(|r| r.split_once(']')) {
        Some((_, rest)) => rest.trim_start(),
        None => prompt,
    };
    (key.to_string(), calm(words).split_whitespace().collect::<Vec<_>>().join(" "))
}
