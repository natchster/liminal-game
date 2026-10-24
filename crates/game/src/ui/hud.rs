//! The HUD: journal, debug panel, grain, messages, lore notes and arcs,
//! medals, the menu overlay, `update_hud`, `try_interact`. Draws through the
//! element model (`crate::ui::model`) + `crate::ui::draw`.
//! CONTRACT: the signatures below are called from other modules; keep them.
use crate::base::tables;
use crate::prelude::*;

/// `whisper_for(arc, id)` result. `glow` is the arc colour as a CSS hex string
/// (the message's text colour); the rest of the arc's `WhisperGlow` is in the
/// `glow_*` fields, and `glow_shadow` is the ready `text_shadow` string built
/// from it.
#[derive(Clone, Debug, Default)]
pub struct Whisper {
    pub duration: f64,
    pub glow: Option<String>,
    pub opacity: f64,
    pub text: Option<String>,
    // ---- the rest of the arc's `WhisperGlow`
    pub glow_alpha: f64,
    pub glow_blur: f64,
    pub glow_rgb: [u32; 3],
    /// `0 0 {blur}px rgba(r, g, b, alpha)` with blur to 1 and alpha to 2 decimals
    pub glow_shadow: Option<String>,
}

/// `whisper_glow_for(arc)` result
#[derive(Clone, Debug, Default)]
pub struct WhisperGlow {
    pub hex: String,
    pub alpha: f64,
    pub blur: f64,
    pub r: u32,
    pub g: u32,
    pub b: u32,
}

// ------------------------------------------------------------ data tables

pub const JOURNAL_EPILOGUE_BASE: &str = "You have read every page. There are more doors than notes. The hum is a hymn — you understand it now. Keep walking. Someone has to be the one who remembers.";

/// (key, check, line)
pub static EPILOGUE_MEDAL_LINES: [(&str, fn(&Game) -> bool, &str); 5] = [
    ("archivist", |g| g.lore_archivist_shown, "Every page you found is now inside you."),
    ("survivor", |g| g.survivor_shown, "Ten floors down and still walking — the hum could not wear you thin."),
    ("storyteller", |g| g.storyteller_shown, "The threads of every story are yours now; you closed the last loop."),
    ("ironwill", |g| g.ironwill_shown, "You never fell. The dark never touched you long enough."),
    ("completionist", |g| g.completionist_shown, "And you did all of it. The pages remember you back now. The hum knows your name."),
];

/// Pick a fresh line each time so the world feels less repetitive
pub static TEXT_POOLS: [(&str, &[&str]); 11] = [
    ("levelSubtitle", &[
        "THE HUM GROWS LOUDER", "DEEPER STILL", "THE WALLS ARE THINNER HERE",
        "SOMETHING WATCHES", "THE LIGHTS ARE DIMMER", "YOU ARE NOT ALONE",
        "THE GEOMETRY IS WRONG", "THE AIR TASTES OF COPPER",
        "YOU DO NOT REMEMBER ENTERING", "IT SMELLS OF WET CARPET",
        "THE HALLWAYS REPEAT", "YOU HEAR YOUR OWN FOOTSTEPS BEHIND YOU",
    ]),
    ("deathTitle", &[
        "IT FOUND YOU", "THE HUM STOPPED", "YOU BLINKED", "YOU LOOKED TOO LONG",
        "IT WAS ALREADY INSIDE", "THE FACE APPROACHED", "YOU FELT COLD BREATH",
        "THE LIGHTS WENT OUT", "YOU HEARD THE WHISPER TOO LATE", "IT LEARNED YOUR NAME",
    ]),
    ("deathBodyBits", &[
        "The hum never stops.", "The wallpaper watches.", "The lights do not blink for you.",
        "Someone else will find your bottle.", "It remembers you now.",
        "The exit door was there. It was.", "You should have crouched.",
        "You should not have run.", "The clock is still wrong.",
        "The mannequin has moved again.",
    ]),
    ("deathHint", &[
        "PRESS R TO WAKE UP", "PRESS R TO TRY AGAIN", "PRESS R TO RETURN",
        "PRESS R — IT WILL NOT MATTER", "PRESS R", "PRESS R IF YOU MUST",
    ]),
    ("pauseSub", &[
        "The fluorescent lights hum indifferently.",
        "Nothing moves. Nothing ever moves when you look.",
        "Somewhere, a phone starts ringing. Then stops.",
        "The wallpaper pattern shifts when you are not watching.",
        "Your breath is the loudest thing in the room.",
        "A door closes, three hallways over.",
    ]),
    ("almondPickup", &[
        "ALMOND WATER — BOTTLED, UNOPENED", "ALMOND WATER — IT SLOSHES WHEN YOU WALK",
        "ALMOND WATER — SEALED, STILL COLD", "ALMOND WATER — SOMEONE LEFT IT FOR YOU",
        "ALMOND WATER — YOU POCKET IT FOR LATER",
    ]),
    ("batteryPickup", &[
        "BATTERY — WARM IN YOUR HAND", "BATTERY — STILL HALF FULL",
        "BATTERY — SOMEONE MEANT TO COME BACK FOR IT", "BATTERY — YOU POCKET THE CELL",
    ]),
    ("almondDrink", &[
        "YOU DRINK — IT TASTES OF PENNIES", "YOU DRINK — YOUR LEGS REMEMBER",
        "YOU DRINK — THE ACHE BACKS OFF", "YOU DRINK — TOO SWEET, AND WELCOME",
    ]),
    ("batterySwap", &[
        "CELL SWAPPED — THE BEAM STEADIES", "CELL SWAPPED — LIGHT AGAIN",
        "CELL SWAPPED — THE FILAMENT CATCHES", "CELL SWAPPED — WARMTH IN THE HOUSING",
    ]),
    ("objectives", &[
        "FIND THE EXIT DOOR", "FIND A WAY DOWN", "FIND THE NEXT LEVEL",
        "KEEP MOVING", "STAY OUT OF THE DARK", "DO NOT ANSWER THE PHONE",
    ]),
    ("overlaySubs", &[
        "You noclipped out of reality. Mono-yellow wallpaper, the hum of fluorescent lights, endless empty rooms. Find the exit door before it finds you.",
        "You do not remember opening the door. You do not remember it being here yesterday. The wallpaper has always been this color.",
        "The lights hum in a language you almost recognize. Somewhere in these rooms, there is a way down. Somewhere in these rooms, something is walking.",
    ]),
];
/// The `TEXT_POOLS` lines for `key`
pub fn text_pool(key: &str) -> Option<&'static [&'static str]> {
    TEXT_POOLS.iter().find(|(k, _)| *k == key).map(|(_, v)| *v)
}

/// One of `LORE_ENTRIES`: a torn paper the player can pick up. `scp` is the
/// anomaly whose level it only lies on (the Stairwell notes 36–41 all carry
/// `"stairwell"`).
#[derive(Clone, Copy, Debug)]
pub struct LoreEntry {
    pub id: usize,
    pub title: &'static str,
    pub body: &'static str,
    pub scp: Option<&'static str>,
}

pub static LORE_ENTRIES: [LoreEntry; 52] = [
    LoreEntry { id: 0, title: "Day 1", body: "I noclipped through the wall of the storage room. My phone shows the wrong date. I can hear humming everywhere.", scp: None },
    LoreEntry { id: 1, title: "Almond Water", body: "The bottles just... appear. Don't ask where. Drink them. They keep the ache away.", scp: None },
    LoreEntry { id: 2, title: "The Hum", body: "The lights hum at a hundred and twenty hertz, twice the mains. Or they're supposed to. Sometimes it swells until your teeth ache. Sometimes it stops. Never look up when it stops.", scp: None },
    LoreEntry { id: 3, title: "M.'s Rule 1", body: "If you turn a corner and see something waiting, close your eyes. Count to ten. It should be gone. Should be.", scp: None },
    LoreEntry { id: 4, title: "M.'s Rule 2", body: "Never answer the phone. I don't care if it's your mother's voice. She's not on level three.", scp: None },
    LoreEntry { id: 5, title: "M.'s Rule 3", body: "The clocks lie. All of them. Trust the hum instead — it means you're still alive.", scp: None },
    LoreEntry { id: 6, title: "Torn Ledger", body: "...paid in almond water, twelve cases... signed by the man in the yellow coat... he was here yesterday...", scp: None },
    LoreEntry { id: 7, title: "Elias, Day 47", body: "Made it to level three today. The plants down here are wet even where there is no water. I keep hearing children laughing.", scp: None },
    LoreEntry { id: 8, title: "Elias, Day 219", body: "There is a pool on this floor. Do not drink from it. The balls in it move on their own.", scp: None },
    LoreEntry { id: 9, title: "The Smilers", body: "Their faces are the only bright thing. Hold its eyes and back away, slowly. It creeps while you watch it. Turn your back, or run, and it comes all at once. Your lamp draws them.", scp: None },
    LoreEntry { id: 10, title: "The Hounds", body: "The four-legged ones don't blink either. They hear footsteps from rooms away. If one runs at you, stare it down — it stops for a breath or two. Not every time. Then it growls.", scp: None },
    LoreEntry { id: 11, title: "The Watcher", body: "You cannot outrun what refuses to move. Just don't look away too long. Especially not when it's near.", scp: None },
    LoreEntry { id: 12, title: "Corporate Memo", body: "All personnel are reminded that the loading dock is not a restroom. Signed, Management. — date illegible.", scp: None },
    LoreEntry { id: 13, title: "Layer Notes", body: "There are floors here that were never built. The elevators know. Sometimes they take you to them.", scp: None },
    LoreEntry { id: 14, title: "Fluorescent Prayer", body: "Blessed be the hum. Blessed be the light. Blessed be the sixty cycles that keep the dark away.", scp: None },
    LoreEntry { id: 15, title: "The Vending Machine", body: "Insert coin, get nothing. Insert nothing, get coin. Do not attempt the reverse. It costs more than you have.", scp: None },
    LoreEntry { id: 16, title: "Paper Trail", body: "I've been dropping paper for weeks. Sometimes I find my own trail. Sometimes I find someone else's, going the opposite way.", scp: None },
    LoreEntry { id: 17, title: "The Mannequin", body: "It was three rooms away. I turned. Now it is here. I did not walk. Neither did it.", scp: None },
    LoreEntry { id: 18, title: "Warning to New Arrivals", body: "If you see the door marked EXIT and there is no green light above it — it is a lie. Walk past. Walk fast.", scp: None },
    LoreEntry { id: 19, title: "Grocery List", body: "eggs, milk, batteries, batteries, batteries, more batteries — the light means safe means alive.", scp: None },
    LoreEntry { id: 20, title: "The Pool Room", body: "There should not be a pool here. There is a pool here. My reflection is not there. I am afraid to swim.", scp: None },
    LoreEntry { id: 21, title: "The Robot", body: "It cleans the carpet. It never stops. I have seen it three levels down, cleaning the same square. Do not step in its way — it does not stop.", scp: None },
    LoreEntry { id: 22, title: "Handwritten Map", body: "This map is wrong. The map is always wrong. Only the door is correct. Only ever the door.", scp: None },
    LoreEntry { id: 23, title: "Faces in the Mirror", body: "Do not look into any mirror longer than four seconds. Mine smiled first, once. I have not smiled since.", scp: None },
    LoreEntry { id: 24, title: "Elias, Final Note", body: "I have been on level three for a very long time now. I do not remember what I look like. Please, if you find this, remember me.", scp: None },
    LoreEntry { id: 25, title: "The Guides", body: "If you cry out loud enough, something answers. Sometimes it shows you the way. Sometimes it just comes.", scp: None },
    LoreEntry { id: 26, title: "Battery Prayer", body: "The battery is the covenant. The flashlight is the promise. The dark is the deep. Amen.", scp: None },
    LoreEntry { id: 27, title: "Windows", body: "The windows here do not open. What they show does not exist. Do not stare too long or it will begin to stare back.", scp: None },
    LoreEntry { id: 28, title: "The Hole", body: "There is a hole in the floor. It leads down. I did not put it there. I did not step into it. Someone did.", scp: None },
    LoreEntry { id: 29, title: "Signed: A Friend", body: "You are not the first. You will not be the last. But you might be the one who finds the door. Keep looking. Please.", scp: None },
    LoreEntry { id: 30, title: "Survey: The Lobby", body: "Rooms, only rooms. Thin walls standing in the middle of nothing, half of them with gaps where a door should be. Wet carpet. The tubes over every other tile. There is nothing here. That is the thing that is here.", scp: None },
    LoreEntry { id: 31, title: "Survey: The Habitable Zone", body: "Concrete, seven metres up, lamps hanging on rods. Pillars in rows. Puddles with no source. When the lights go, walk — do not run — to the narrow halls. They rarely go in there.", scp: None },
    LoreEntry { id: 32, title: "Survey: Utility Halls", body: "Shoulder-wide corridors in raw concrete, the walls bulging in at you. Low ceiling. The hum is worse down here and the echo answers a beat late. Keep moving. Don't touch the pipes.", scp: None },
    LoreEntry { id: 33, title: "Survey: Electrical Station", body: "Old brick, older wiring. Something crackles behind every wall. Someone has written RUN in chalk and I have been trying to decide whether it was advice.", scp: None },
    LoreEntry { id: 34, title: "Survey: Abandoned Office", body: "Low ceilings, carpet tiles, a door in every wall and a room behind every door. Cubicle walls to your chest. You can see over them. So can anything else.", scp: None },
    LoreEntry { id: 35, title: "Blackout Zones", body: "Where the lights are dead there is no hum. No echo either. Your own breathing is the loudest thing in the world. Don't stay. Whatever lives in the silence likes it that way.", scp: None },
    LoreEntry { id: 36, title: "The Steel Door", body: "Some doors say STAIRS and look like a janitor's closet. Behind them: concrete steps going down, thirteen to a flight, a half-round landing, turn, thirteen more. The door does not open again from that side.", scp: Some("stairwell") },
    LoreEntry { id: 37, title: "Stairwell, Landing 9", body: "Brought the big lamp. Doesn't matter. It shows nine steps and then the dark swallows the rest. I can see about a flight and a half. Somebody is crying down there.", scp: Some("stairwell") },
    LoreEntry { id: 38, title: "Stairwell, Landing 30", body: "The crying never gets closer. I have gone down thirty landings and it is exactly as far away as it was at the top. It says please. It says down here. It is not getting closer. I am not getting closer.", scp: Some("stairwell") },
    LoreEntry { id: 39, title: "Stairwell, Landing 51", body: "There was a face on the landing below me. No mouth. No nostrils. Nothing in the eyes. It did not come up the steps. It was just nearer every time I looked. I did not walk towards it. Don't walk towards it.", scp: Some("stairwell") },
    LoreEntry { id: 40, title: "Blue Lights", body: "We left blue markers every ten landings on the way down. Most of them are dead now. If you find one still blinking, you are further than we got.", scp: Some("stairwell") },
    LoreEntry { id: 41, title: "Do Not Turn Back", body: "It never chases. That is the rule and the rule is a lie. The moment you turn to climb back up, it is behind you. Close enough to touch. Keep going down. The door is at the bottom.", scp: Some("stairwell") },
    LoreEntry { id: 42, title: "Receipt, Back Side", body: "Bought nothing. Walked for three days. The arrows on the floor only ever point one way, and only while it is bright.", scp: Some("store") },
    LoreEntry { id: 43, title: "Staff Only", body: "They have no faces and they let you browse. They always let you browse. Until the chime.", scp: Some("store") },
    LoreEntry { id: 44, title: "Display Bed", body: "When it went dark I got under one of the display beds and held my breath. Something in a yellow shirt stood beside it for a long, long time.", scp: Some("store") },
    LoreEntry { id: 45, title: "Wrong House", body: "The kitchen was from one house and the stairs were from another. Somebody took these rooms. Somebody is still taking them.", scp: Some("house") },
    LoreEntry { id: 46, title: "Keep It Dark", body: "Stay out of the light. I know how that sounds. In the dark they are only smoke.", scp: Some("house") },
    LoreEntry { id: 47, title: "Where the Lamps Go", body: "Every lamp in this place ends up in the same room in the middle. I think it keeps them. I think it needs them.", scp: Some("house") },
    LoreEntry { id: 48, title: "The Front Door", body: "Whatever you do, close the front door behind you. It will not close while the middle room is lit.", scp: Some("house") },
    LoreEntry { id: 49, title: "Count the Doors", body: "Count the doors going in. Then count them back out. I stopped counting and now I don't know which way is out.", scp: Some("loop") },
    LoreEntry { id: 50, title: "The Plant", body: "I left a marker by the plant. Next time round the plant was on the other side of the room. The marker wasn't.", scp: Some("loop") },
    LoreEntry { id: 51, title: "The First Room", body: "There was a door in the first room. There was. I keep going round to check and the rooms keep getting worse.", scp: Some("loop") },
];

/// `SCP_OMENS`: what the player notices on arriving at an anomaly's level
pub fn scp_omen(key: &str) -> Option<&'static str> {
    Some(match key {
        "stairwell" => "Far below you, somebody is crying.",
        "store" => "A two-note chime, somewhere overhead. The air smells of sawdust and cinnamon.",
        "house" => "None of these rooms remember the same house.",
        "loop" => "Somewhere a door clicks shut. Then again. Then again.",
        _ => return None,
    })
}

/// One of `LORE_ARCS`: grouped lore ids that form a story.
#[derive(Clone, Copy, Debug)]
pub struct LoreArc {
    pub name: &'static str,
    pub ids: &'static [usize],
    pub color: u32,
}
/// The lore arcs, in order. `color` tints boosted (continuation) notes.
pub static LORE_ARCS: [LoreArc; 5] = [
    LoreArc { name: "ELIAS", ids: &[7, 8, 24], color: 0x60a0ff },     // cool blue
    LoreArc { name: "M.'S RULES", ids: &[3, 4, 5], color: 0xffb060 }, // warm amber
    LoreArc { name: "THE ENTITIES", ids: &[9, 10, 11], color: 0xff5060 }, // red
    LoreArc { name: "THE RITUAL", ids: &[2, 14, 26], color: 0xa0f090 }, // soft green
    LoreArc { name: "THE STAIRWELL", ids: &[36, 38, 39, 41], color: 0xd8d2c4 }, // the Face's pallor
];

/// Candle blow-out / relight milestone thresholds (copies of the world
/// build's values; only the debug panel reads them)
const BLOWOUT_MILESTONE_AT: f64 = 25.0;
const RELIGHT_MILESTONE_AT: f64 = 15.0;

/// the grain canvas size (R769: 480×300)
pub const GRAIN_W: u32 = 480;
pub const GRAIN_H: u32 = 300;

fn hex6(c: u32) -> String {
    format!("#{:06x}", c)
}
/// `s` padded with spaces to `n` characters
fn pad_end(s: &str, n: usize) -> String {
    let l = s.chars().count();
    if l >= n {
        s.to_string()
    } else {
        format!("{}{}", s, " ".repeat(n - l))
    }
}
fn yn(b: bool) -> &'static str {
    if b {
        "y"
    } else {
        "n"
    }
}
fn fx(v: f64, d: usize) -> String {
    fmt_fixed(v, d)
}
fn ns(v: f64) -> String {
    fmt_num(v)
}
fn sum_vals(m: Option<&std::collections::BTreeMap<String, f64>>) -> f64 {
    m.map(|m| m.values().sum()).unwrap_or(0.0)
}

/// a journal entry id: a lore id, or the sentinels -1 (epilogue), -2 (index), -3 (medals)
pub type JournalId = i64;

impl Game {
    // ------------------------------------------------------------ startup
    /// Startup work (grain canvas, journal, medals)
    pub fn init_hud(&mut self) {
        // R769 grain resolution bump: 220×140 was pixelated on large screens.
        self.grain_canvas = Some(Rc::new(RefCell::new(Canvas::new(GRAIN_W, GRAIN_H))));
        self.hud_ui = crate::ui::draw::HudUi::default();
    }

    // ------------------------------------------------------------ journal
    pub fn journal_sorted_ids(&self) -> Vec<usize> {
        self.lore_found.iter().copied().collect()
    }
    /// → (title, body)
    pub fn current_epilogue(&self) -> (String, String) {
        let mut lines: Vec<&str> = vec![JOURNAL_EPILOGUE_BASE];
        let bonuses: Vec<&str> = EPILOGUE_MEDAL_LINES.iter().filter(|m| (m.1)(self)).map(|m| m.2).collect();
        if !bonuses.is_empty() {
            lines.push("");
            lines.extend(bonuses);
        }
        ("EPILOGUE".into(), lines.join("\n"))
    }
    /// The cover (-2), the medals page (-3) once a
    /// medal is held, the sorted lore ids, then the epilogue (-1) once Archivist.
    pub fn journal_entries_in_order(&self) -> Vec<JournalId> {
        let mut ids: Vec<JournalId> = self.lore_found.iter().map(|&i| i as JournalId).collect();
        if ids.is_empty() {
            return ids;
        }
        ids.insert(0, -2);
        if self.lore_archivist_shown || self.survivor_shown || self.storyteller_shown || self.completionist_shown || self.ironwill_shown {
            ids.insert(1, -3); // right after cover
        }
        if self.lore_archivist_shown {
            ids.push(-1);
        }
        ids
    }
    pub fn medals_page_html(&self) -> String {
        struct Row {
            earned: bool,
            color: &'static str,
            name: &'static str,
            crit: &'static str,
        }
        let mut rows = vec![
            Row { earned: self.lore_archivist_shown, color: "#f8e0a0", name: "ARCHIVIST", crit: "Find every lore note" },
            Row { earned: self.survivor_shown, color: "#a8d8f0", name: "SURVIVOR", crit: "Reach level 10 in a single run" },
            Row { earned: self.storyteller_shown, color: "#d8b0f0", name: "STORYTELLER", crit: "Complete every narrative arc" },
            Row {
                earned: self.completionist_shown,
                color: "#ffedb0",
                name: "COMPLETIONIST",
                crit: "Hold Archivist + Survivor + Storyteller together",
            },
            Row { earned: self.ironwill_shown, color: "#a0f0d8", name: "IRONWILL", crit: "Reach level 5 without dying" },
        ];
        // stable sort: earned medals to the top
        rows.sort_by_key(|r| if r.earned { 0 } else { 1 });
        let mut row_strings: Vec<String> = vec![];
        let mut last_earned: Option<bool> = None;
        for r in &rows {
            if last_earned == Some(true) && !r.earned {
                row_strings.push("  <span style=\"opacity:0.35\">─────────────────────────────────</span>".into());
            }
            let badge = if r.earned {
                format!("<span style=\"color:{c};text-shadow:0 0 4px {c}\">◆</span>", c = r.color)
            } else {
                "<span style=\"color:rgba(120,110,90,0.6)\">◇</span>".to_string()
            };
            let name_span = if r.earned {
                format!("<span style=\"color:{c};text-shadow:0 0 3px {c}\">{n}</span>", c = r.color, n = r.name)
            } else {
                format!("<span style=\"color:rgba(140,130,110,0.6)\">{}</span>", r.name)
            };
            row_strings.push(format!("  {}  {}  —  <span style=\"opacity:0.7\">{}</span>", badge, name_span, r.crit));
            last_earned = Some(r.earned);
        }
        let lines = row_strings.join("\n");
        let total = rows.iter().filter(|r| r.earned).count();
        format!(
            "Medals of this run: <span style=\"color:#f8e0a0\">{}</span> of {}.\n\n{}\n\nUse NEXT to continue browsing.",
            total,
            rows.len(),
            lines
        )
    }
    pub fn journal_cover_text(&self) -> String {
        let lines = LORE_ARCS
            .iter()
            .map(|arc| {
                let found = arc.ids.iter().filter(|id| self.lore_found.contains(id)).count();
                let state = if found == arc.ids.len() { "COMPLETE".to_string() } else { format!("{}/{}", found, arc.ids.len()) };
                let hex = hex6(arc.color);
                format!("  <span style=\"color:{h};text-shadow:0 0 4px {h}\">■</span>  {}  —  {}", arc.name, state, h = hex)
            })
            .collect::<Vec<_>>()
            .join("\n");
        let total_found = self.lore_found.len();
        let total = LORE_ENTRIES.len();
        // ASCII progress bar toward the Archivist medal — 20 cells
        let bar_cells = 20usize;
        let filled = round_half_up((total_found as f64 / total as f64) * bar_cells as f64) as usize;
        let bar = "▮".repeat(filled) + &"▯".repeat(bar_cells - filled.min(bar_cells));
        let arcs = self.lore_arcs_completed.len();
        let bar_color = if total_found == total {
            "#ffe0a0"
        }
        // Archivist
        else if arcs >= 4 {
            "#f8e0a0"
        }
        // Storyteller done
        else if arcs >= 3 {
            "#e8c880"
        }
        // near-done
        else if arcs >= 1 {
            "#d8b070"
        }
        // in progress
        else {
            "#c8bea4"
        }; // no arcs yet
        let bar_html = format!(
            "<span class=\"journal-bar\" style=\"color:{};letter-spacing:1px\">{}</span>  <span style=\"opacity:0.7\">{}/{}</span>",
            bar_color, bar, total_found, total
        );
        format!(
            "You have collected {} of {} notes across {} of {} story arcs.\n\n{}\n\n{}\n\nPress NEXT to browse individual entries.",
            total_found,
            total,
            self.lore_arcs_completed.len(),
            LORE_ARCS.len(),
            bar_html,
            lines
        )
    }
    pub fn journal_render(&mut self) {
        let ids = self.journal_entries_in_order();
        if ids.is_empty() {
            self.ui.journal_entry_title.set_text("NO NOTES YET");
            let body = if self.ui.body.has_class("touch") {
                "Find torn pages scattered on the floors. Tap JOURNAL to browse what you've read."
            } else {
                "Find torn pages scattered on the floors. Press J to browse what you've read."
            };
            self.ui.journal_entry_body.set_text(body);
            self.ui.journal_idx.set_text("0 / 0");
            self.ui.journal_prev.set_attr("disabled", "true");
            self.ui.journal_next.set_attr("disabled", "true");
            return;
        }
        self.journal_cursor = clamp(self.journal_cursor, 0.0, (ids.len() - 1) as f64);
        let ci = self.journal_cursor as usize;
        let cur = ids[ci];
        let (title, body) = match cur {
            -1 => self.current_epilogue(),
            -2 => ("INDEX".to_string(), self.journal_cover_text()),
            -3 => ("MEDALS".to_string(), self.medals_page_html()),
            _ => {
                let e = &LORE_ENTRIES[cur as usize];
                (e.title.to_string(), e.body.to_string())
            }
        };
        // Arc annotation: "ELIAS · 2/3" after the title of an arc's note
        let mut arc_suffix = String::new();
        if cur >= 0 {
            for arc in LORE_ARCS.iter() {
                if let Some(idx) = arc.ids.iter().position(|&i| i as JournalId == cur) {
                    arc_suffix = format!("  ·  {} {}/{}", arc.name, idx + 1, arc.ids.len());
                    break;
                }
            }
        }
        self.ui.journal_entry_title.set_text(&format!("{}{}", title, arc_suffix));
        // Cover + medals pages use HTML (colour spans); lore + epilogue use plain text.
        self.ui.journal_entry_body.remove_class("cover-fade");
        if cur == -2 || cur == -3 {
            self.ui.journal_entry_body.set_html(&body);
            if cur == -2 {
                // re-adding the class restarts the fade even after a previous cover render
                self.ui.journal_entry_body.add_class("cover-fade");
            }
            // medals page audio flourish, only when newly arriving at the page
            if cur == -3 && self.last_rendered_cur != Some(-3) && self.ac.is_some() {
                self.blip(1760.0, 0.9, "sine", 0.09, None, 0.0, 0.0);
                self.blip(2640.0, 1.1, "sine", 0.06, None, 0.08, 0.0);
            }
        } else {
            self.ui.journal_entry_body.set_text(&body);
            // Epilogue tier chord — note count scales with medals held; only on the edge INTO the epilogue.
            if cur == -1 && self.last_rendered_cur != Some(-1) && self.ac.is_some() {
                let medal_count = EPILOGUE_MEDAL_LINES.iter().filter(|m| (m.1)(self)).count();
                let notes = [523.0, 659.0, 784.0, 1046.0, 1319.0, 1568.0]; // C E G C E G
                let base_gain = 0.09 + medal_count as f64 * 0.010;
                let bright_boost = (medal_count as f64 - 3.0).max(0.0) * 0.012;
                let mut i = 0usize;
                while i <= medal_count && i < notes.len() {
                    let gain = base_gain - i as f64 * 0.008 + if i >= 3 { bright_boost } else { 0.0 };
                    self.blip(notes[i], 1.6 + i as f64 * 0.2, "sine", gain, None, i as f64 * 0.10, 0.0);
                    i += 1;
                }
            }
        }
        self.last_rendered_cur = Some(cur);
        let found_label = format!("{}/{} FOUND", self.lore_found.len(), LORE_ENTRIES.len());
        let epilogue_label = if self.lore_archivist_shown { "  ·  EPILOGUE UNLOCKED" } else { "" };
        self.ui.journal_idx.set_text(&format!("{} / {}  ·  {}{}", ci + 1, ids.len(), found_label, epilogue_label));
        self.ui.journal_prev.set_attr("disabled", if ci == 0 { "true" } else { "false" });
        self.ui.journal_next.set_attr("disabled", if ci == ids.len() - 1 { "true" } else { "false" });
    }
    pub fn journal_prev(&mut self) {
        if self.journal_cursor > 0.0 {
            self.journal_cursor -= 1.0;
            self.journal_render();
            // Landing on the cover plays a soft chime whose pitch rises with arc progress
            let ids = self.journal_entries_in_order();
            let landed_on_cover = ids.get(self.journal_cursor as usize) == Some(&-2);
            if landed_on_cover {
                if self.ac.is_some() {
                    let arcs = self.lore_arcs_completed.len() as f64;
                    let base = 1200.0 + arcs * 75.0;
                    let upper_boost = arcs * 0.008;
                    self.blip(base, 1.0, "sine", 0.08, None, 0.0, 0.0);
                    self.blip(base * 1.5, 1.2, "sine", 0.06 + upper_boost, None, 0.10, 0.0);
                    self.blip(base * 2.0, 1.4, "sine", 0.04 + upper_boost, None, 0.22, 0.0);
                }
            } else {
                self.sfx_page_flip();
            }
        }
    }
    pub fn journal_next(&mut self) {
        let ids = self.journal_entries_in_order();
        if (self.journal_cursor as usize) + 1 < ids.len() {
            self.journal_cursor += 1.0;
            self.journal_render();
            self.sfx_page_flip();
        }
    }
    pub fn toggle_journal(&mut self) {
        if self.state != "playing" && self.state != "paused" {
            return;
        }
        self.journal_open = !self.journal_open;
        let open = self.journal_open;
        self.ui.journal.toggle_class("on", open);
        if open {
            self.journal_cursor = (self.journal_entries_in_order().len() as f64 - 1.0).max(0.0); // open on newest / epilogue
            self.journal_render();
            self.want_pointer_lock = Some(false); // release the pointer
                                                  // 20% chance of a paper rustle instead of a page flip on open
            if rnd() < 0.20 {
                self.sfx_paper_rustle();
            } else {
                self.sfx_page_flip();
            }
        } else {
            self.want_pointer_lock = Some(true); // capture the pointer again
        }
    }

    // ------------------------------------------------------------ debug panel
    pub fn toggle_debug(&mut self) {
        self.debug_on = !self.debug_on;
        let on = self.debug_on;
        self.ui.debug.toggle_class("on", on);
    }
    /// The [K] debug panel's text, rebuilt from the current state.
    pub fn update_debug(&mut self) {
        if !self.debug_on {
            return;
        }
        let wk = self.weather_key();
        let (px, pz) = (self.player.pos.x, self.player.pos.z);
        let (tile_x, tile_z) = (self.tx(px), self.tz(pz));
        let rid: i64 = if self.in_grid(tile_x, tile_z) {
            self.region_map.get(self.t_idx(tile_x, tile_z)).map(|&r| r as i64).unwrap_or(-1)
        } else {
            -1
        };
        let info = if rid >= 0 { self.region_info.get(rid as usize).cloned() } else { None };
        let yaw = ((self.player.yaw * 180.0 / PI) % 360.0 + 360.0) % 360.0;
        let pitch = self.player.pitch * 180.0 / PI;
        let in_pool = self.pool_bounds.map(|b| px > b.min_x && px < b.max_x && pz > b.min_z && pz < b.max_z).unwrap_or(false);
        let mut flag_list: Vec<&str> = vec![];
        if let Some(i) = &info {
            if i.flags & crate::visuals::lighting::RF_DARK != 0 {
                flag_list.push("DARK");
            }
            if i.flags & crate::visuals::lighting::RF_MOSS != 0 {
                flag_list.push("MOSS");
            }
        }
        let seed_hex = format!("{:08x}", self.current_seed);
        let run_hex = format!("{:08x}", self.run_seed);
        let p = &self.player;
        let mut s = String::with_capacity(8192);
        use std::fmt::Write;
        let _ = write!(
            s,
            "── DEBUG [K] ──\nlevel     {}\nrunSeed   0x{}\nlevelSeed 0x{}\ngrid      {} × {}\nfps       {}\ndeaths    {}\n",
            self.level_num,
            run_hex,
            seed_hex,
            self.w,
            self.h,
            fx(self.fps, 1),
            ns(self.death_count)
        );
        let _ = write!(s, "─ player ─\npos       {}, {}, {}\ntile      {}, {}\nyaw/pitch {}°, {}°\nsanity    {} / {}\nbattery   {} / {}\nstamina   {} / {}\nflash     {}\ncrouch    {}\nin-pool   {}\nbottles   {}   batteries {}   papers {}\n",
            fx(px, 2), fx(p.eye_h, 2), fx(pz, 2), tile_x, tile_z, fx(yaw, 1), fx(pitch, 1),
            fx(p.sanity, 0), ns(SAN_MAX), fx(p.battery, 0), ns(BAT_MAX), fx(p.stamina, 0), ns(STAM_MAX),
            if p.flashlight_on { "ON" } else { "off" }, if p.crouch { "yes" } else { "no" }, if in_pool { "yes" } else { "no" },
            p.bottles, p.batteries, p.papers);
        let _ = write!(
            s,
            "─ crit-sanity ─\ncritAccum {}s   whisperChance {}\nbeat      {}s   whisperT {}s\n",
            fx(self.crit_sanity_accum, 1),
            fx(0.30 + clamp((self.crit_sanity_accum - 10.0) / 30.0, 0.0, 1.0) * 0.30, 2),
            fx(self.sanity_beat_t, 2),
            fx(self.sanity_whisper_t, 1)
        );
        // ─ audio keys ─
        let bh = self.weather_pick(tables::battery_hint_dips, tables::battery_hint_dips("normal").unwrap());
        let lfo = self.audio.safe_room_lfo.as_ref().map(|n| fx(n.frequency().value(), 3)).unwrap_or_else(|| "-".into());
        let carrier = self.audio.safe_room_trem_carrier.as_ref().map(|n| fx(n.frequency().value(), 1)).unwrap_or_else(|| "-".into());
        let det = tables::carrier_weather_detune(wk).unwrap_or(0.0);
        let _ = write!(
            s,
            "─ audio keys ─\nbatteryHint {} / {} Hz\nsafeRoom    LFO={} Hz  carrier={} Hz  wMul={}  cOff={}\n",
            ns(bh[0]),
            ns(bh[1]),
            lfo,
            carrier,
            fx(tables::lfo_weather_mul(wk).unwrap_or(1.0), 2),
            if det >= 0.0 { "+" } else { "" }.to_string() + &ns(det)
        );
        let dash = || "-".to_string();
        let _ = write!(
            s,
            "─ region {} ─\nwall  {}   ceil {}   floor {}\nflags [{}]\n",
            rid,
            info.as_ref().map(|i| i.wall.to_string()).unwrap_or_else(dash),
            info.as_ref().map(|i| i.ceil.to_string()).unwrap_or_else(dash),
            info.as_ref().map(|i| i.floor.to_string()).unwrap_or_else(dash),
            flag_list.join(", ")
        );
        // ─ entities ─
        let ents = self
            .entities
            .iter()
            .enumerate()
            .map(|(i, e)| {
                let ep = e.pos();
                format!("  #{} {} state={} pos={},{}", i, pad_end(&e.kind, 8), pad_end(&e.state, 8), fx(ep.x, 1), fx(ep.z, 1))
            })
            .collect::<Vec<_>>()
            .join("\n");
        let _ = write!(s, "─ entities ({}) ─\n{}\n", self.entities.len(), if ents.is_empty() { "  none".to_string() } else { ents });
        // ─ switches ─
        let sws = self
            .light_switches
            .iter()
            .enumerate()
            .map(|(i, sw)| {
                let tag = if sw.faulty { "FAULT" } else { "ok   " };
                let arm = sw.f.get("armT").copied().unwrap_or(0.0);
                let arm_val = if arm > 0.0 { fx(arm, 2) } else { "-".into() };
                let cd = sw.f.get("motionCooldown").copied().unwrap_or(0.0);
                let cd_val = if cd > 0.0 { fx(cd, 1) } else { "-".into() };
                let near = if sw.b.get("wasInRange").copied().unwrap_or(false) { "IN " } else { "   " };
                // TODO: the placard name has no field on `records::LightSwitch`; it is read from the group's user data (`label`) when the world build sets it
                let label = sw.g.as_ref().and_then(|g| g.r().user.s.get("label").cloned()).unwrap_or_default();
                format!("  #{} {} on={} arm={} cd={} {}\"{}\"", i, tag, yn(sw.on), arm_val, cd_val, near, label)
            })
            .collect::<Vec<_>>()
            .join("\n");
        let _ = write!(
            s,
            "─ switches ({}) trips={} ─\n{}\n",
            self.light_switches.len(),
            ns(self.faulty_trip_count),
            if sws.is_empty() { "  none".to_string() } else { sws }
        );
        let _ = writeln!(
            s,
            "─ mileshown blowout={}({}/{}) relight={}({}/{}) archive={}({}/{}+{}/{}) ─",
            yn(self.blowout_milestone_shown),
            ns(self.candle_blowout_tally),
            ns(BLOWOUT_MILESTONE_AT),
            yn(self.relight_milestone_shown),
            ns(self.candle_relight_tally),
            ns(RELIGHT_MILESTONE_AT),
            yn(self.archive_milestone_shown),
            self.lore_found.len(),
            LORE_ENTRIES.len(),
            self.lore_arcs_completed.len(),
            LORE_ARCS.len()
        );
        // ─ battery cue ─
        {
            let bs = self.beam_battery_scale(0.4);
            let li = self.beam_battery_scale(0.5);
            let di = self.beam_battery_scale(0.6);
            let warm = if self.player.flashlight_on { fx(1.0 - bs, 2) } else { "-".into() };
            let cboost = if self.player.flashlight_on { fx(1.0 + 0.15 * (1.0 - self.beam_battery_scale(0.0)), 2) } else { "1.00".into() };
            let pd_floor = tables::panel_dim_floor_by_weather(wk).or(tables::panel_dim_floor_by_weather("normal")).unwrap();
            let pd_radius = tables::panel_dim_radius_by_weather(wk).or(tables::panel_dim_radius_by_weather("normal")).unwrap();
            let pd_live = self.beam_battery_scale(pd_floor);
            let pd_falloff = tables::panel_dim_falloff_by_weather(wk).or(tables::panel_dim_falloff_by_weather("normal")).unwrap();
            let _ = write!(s, "─ battery cue (R110–R114 dying-beam) ─\n  bScale={}  lightMul={}  distMul={}  warm={}  candleBoost={}\n  panelDim  floor={}  depth={}  radius={}m  falloff={}  live={}\n",
                fx(bs, 2), fx(li, 2), fx(di, 2), warm, cboost, ns(pd_floor), fx(1.0 - pd_floor, 3), ns(pd_radius), ns(pd_falloff), fx(pd_live, 3));
        }
        let (ft, ct) = (self.footstep_fx_tally, self.crouch_fx_tally);
        let _ = write!(s, "─ footstep fx tally (R130–R133/R216) ─\n  walk    wet={} still={} electric={} haunted={} derelict={} normal={}\n  crouch  wet={} still={} electric={} haunted={} derelict={} normal={}\n",
            ns(ft.wet), ns(ft.still), ns(ft.electric), ns(ft.haunted), ns(ft.derelict), ns(ft.normal),
            ns(ct.wet), ns(ct.still), ns(ct.electric), ns(ct.haunted), ns(ct.derelict), ns(ct.normal));
        let rain_gain = self.audio.rain_hiss_gain.as_ref().map(|n| fx(n.gain().value(), 3)).unwrap_or_else(|| "-".into());
        let _ = write!(
            s,
            "─ rain hiss (R120) ─\n  rainScore={}  gain={}  windows={}\n",
            fx(self.rain_score_current, 2),
            rain_gain,
            self.rain_windows.len()
        );
        // ─ ripples ─
        {
            let active: Vec<&crate::base::records::PooledFx> = self.ripples.iter().filter(|r| r.t > 0.0).collect();
            let body = if active.is_empty() {
                "  none active".to_string()
            } else {
                active
                    .iter()
                    .enumerate()
                    .map(|(i, r)| {
                        let rp = r.obj.pos();
                        let (dx, dz) = (rp.x - self.player.pos.x, rp.z - self.player.pos.z);
                        let d = (dx * dx + dz * dz).sqrt();
                        let op = r.obj.mats().first().map(|m| m.opacity_v()).unwrap_or(0.0);
                        format!("  #{} t={}s  d={}m  op={}", i, fx(r.t, 2), fx(d, 1), fx(op, 2))
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            };
            let _ = write!(s, "─ ripples (R117/R118) ─\n{}\n", body);
        }
        // ─ rare-tier timers ─
        {
            let f = |name: &str, t: f64| format!("{}={}{}", name, fx(t, 1), if t < 5.0 { "<" } else { "" });
            let parts = [
                f("thunder", self.wet_thunder_timer),
                f("echo", self.still_echo_timer),
                f("grinder", self.derelict_grinder_timer),
                f("surge", self.electric_surge_timer),
                f("step", self.haunted_step_timer),
                f("chime", self.normal_notice_timer),
                f("knock", self.distant_knock_timer),
                f("door", self.door_opened_timer),
                f("ding", self.elevator_ding_timer),
                f("voice", self.voice_sample_timer),
                f("static", self.static_burst_timer),
                f("shift", self.reality_shift_timer),
                f("patt", self.pattern_break_timer),
                f("skip", self.clock_skip_timer),
            ];
            let _ = write!(s, "─ rare-tier timers (seconds until fire, < = imminent) ─\n  {}\n", parts.join("  "));
        }
        // ─ descent ─
        {
            let ld = self.level_depth_mul();
            let cd = self.chime_depth_mul();
            let dread = (1.0 - ld) * 0.8;
            let fog_mul = 1.0 + (1.0 - cd) * 1.05;
            let dist_descent = 0.5 + 0.5 * ld;
            let light_depth = 0.7 + 0.3 * ld;
            let dust_d = 0.75 + 0.25 * ld;
            let panel_amp_boost = 1.0 + (1.0 - ld) * 0.4;
            let int_desc = 0.65 + 0.35 * ld;
            let _ = write!(s, "─ descent (R164–R168 deep-level effects) ─\n  level={}  vignette=+{}\n  atmos  fogMul={}  fog×={}  bg×={}  dust×{}\n  light  light×{}  int×{}  range×{}  panel×{}  amp×{}\n",
                self.level_num, fx(dread, 3), fx(fog_mul, 3), fx(cd, 3), fx(cd, 3), fx(dust_d, 3),
                fx(light_depth, 3), fx(int_desc, 3), fx(dist_descent, 3), fx(ld, 3), fx(panel_amp_boost, 3));
        }
        {
            let src = self.audio.safe_room_lfo.as_ref().map(|n| fx(n.frequency().value(), 3)).unwrap_or_else(|| "-".into());
            let flame = tables::flame_breath_amp_by_weather(wk).or(tables::flame_breath_amp_by_weather("normal")).unwrap();
            let slope = tables::level_depth_slope_by_weather(wk).or(tables::level_depth_slope_by_weather("normal")).unwrap();
            let _ = write!(s, "─ room breath (R48/R63 LFO drives all) ─\n  src={}Hz  vignette={}  panel={}  flame={}  levelDepth={}  chimeDepth={}  slope={}  descentDread={}  fogMul={}\n",
                src, fx(self.hum_pulse_amp_current, 3), fx(self.panel_lfo_amp_current, 3), fx(flame, 3), fx(self.level_depth_mul(), 3),
                fx(self.chime_depth_mul(), 3), fx(slope, 3), fx((1.0 - self.level_depth_mul()) * 0.8, 3), fx(1.0 + (1.0 - self.chime_depth_mul()) * 1.05, 3));
        }
        // ─ sanity breakdown ─
        {
            let sv = self.player.sanity / SAN_MAX;
            let roll_active = sv < 0.50;
            let hue_active = sv < 0.20;
            let detune_active = sv < 0.20;
            let bright_active = sv < 0.30;
            let san_timer = if sv >= 0.30 { 1.00 } else { 1.0 / (1.0 + (1.0 - sv / 0.30) * 0.5) };
            let dsc_timer = 1.0 / self.descent_boost_mul(0.15);
            let san_gain = if sv >= 0.30 { 1.00 } else { 1.0 + (1.0 - sv / 0.30) * 0.35 };
            let dsc_gain = self.descent_boost_mul(0.25);
            let dread_period =
                tables::sanity_dread_period_ms_by_weather(wk).or(tables::sanity_dread_period_ms_by_weather("normal")).unwrap()
                    / self.descent_freq_boost();
            let _ = write!(s, "─ sanity breakdown (R138–R143) ─\n  s={}%  gates: roll={}  hue={}  detune={}  bright={}  dreadT={}ms\n  muls   sanTimer={}×dscTimer={}  sanGain={}×dscGain={}\n  chimes rec={} imp={} brk={} mini={}\n",
                fx(sv * 100.0, 0), yn(roll_active), yn(hue_active), yn(detune_active), yn(bright_active), fx(dread_period, 0),
                fx(san_timer, 2), fx(dsc_timer, 2), fx(san_gain, 2), fx(dsc_gain, 2),
                ns(self.sanity_recovery_tally), ns(self.sanity_impaired_tally), ns(self.sanity_breaking_tally), ns(self.sanity_mini_tally));
        }
        // ─ player rhythms ─
        {
            let idle = tables::idle_breath_by_weather(wk).or(tables::idle_breath_by_weather("normal")).unwrap();
            let active = if self.player.sanity < 50.0 { format!("y {}", fx(1.0 - self.player.sanity / 50.0, 2)) } else { "n".into() };
            let sv = self.player.sanity / SAN_MAX;
            let panic = if sv < 0.30 { 1.0 - sv / 0.30 } else { 0.0 };
            let stam_crouch_wobble =
                if self.player.crouch && self.player.stamina < 40.0 { fx(1.0 - self.player.stamina / 40.0, 2) } else { "-".into() };
            let row_dip_mag = tables::stamina_dip_mag_by_weather(wk).or(tables::stamina_dip_mag_by_weather("normal")).unwrap();
            let stam_dip = if !self.player.crouch && self.player.stamina < 30.0 {
                fx((1.0 - self.player.stamina / 30.0) * row_dip_mag, 3)
            } else {
                "-".into()
            };
            let _ = write!(s, "─ player rhythms (R135–R178) ─\n  idleAmp={}  idleFreq={}Hz  crouchTilt={}rad  bobDsc={}  bobRate={}\n  sanityAmp={}  sanityFreq={}Hz  active={}\n  panicBreath  freqMul={}x  ampMul={}x  panic={}  stamWobble={}  stamDip={}rad\n",
                fx(idle.amp, 3), fx(idle.freq, 2), fx(self.crouch_tilt_mag_current, 3), fx(self.descent_amp_boost(), 3), fx(2.1 * self.descent_boost_mul(0.15), 3),
                fx(self.sanity_roll_amp_current, 3), fx(self.sanity_roll_freq_current, 2), active,
                fx(1.0 + panic * 0.6, 2), fx(1.0 - panic * 0.30, 2), fx(panic, 2), stam_crouch_wobble, stam_dip);
        }
        // ─ stamina breath ─
        {
            let br = tables::stamina_breath_by_weather(wk).or(tables::stamina_breath_by_weather("normal")).unwrap();
            let a = &self.audio;
            let g = a.stamina_breath_gain.as_ref().map(|n| fx(n.gain().value(), 4)).unwrap_or_else(|| "-".into());
            let f = a.stamina_breath_lfo.as_ref().map(|n| fx(n.frequency().value(), 3)).unwrap_or_else(|| "-".into());
            let lp = a.stamina_breath_lp.as_ref().map(|n| fx(n.frequency().value(), 0)).unwrap_or_else(|| "-".into());
            let q = a.stamina_breath_lp.as_ref().map(|n| fx(n.q().value(), 2)).unwrap_or_else(|| "-".into());
            let sg = a.stamina_breath_sub_gain.as_ref().map(|n| fx(n.gain().value(), 4)).unwrap_or_else(|| "-".into());
            let sf = a.stamina_breath_sub_osc.as_ref().map(|n| fx(n.frequency().value(), 0)).unwrap_or_else(|| "-".into());
            let pn = a.stamina_breath_panner.as_ref().map(|n| fx(n.pan().value(), 2)).unwrap_or_else(|| "-".into());
            let _ = write!(s, "─ stamina breath (R200–R207) ─\n  gain={}  freq={}Hz  lp={}Hz  q={}  sub={}@{}Hz  pan={}\n  peaks[{}]  g={}  rBase={}  rGain={}  sub={}@{}  pan={}@{}\n",
                g, f, lp, q, sg, sf, pn, if wk.is_empty() { "?" } else { wk },
                ns(br.gain), ns(br.rate_base), ns(br.rate_gain), ns(br.sub_gain), ns(br.sub_freq), ns(br.pan_depth), ns(br.pan_rate));
        }
        // ─ rare events ─
        {
            let kt = tables::knock_timbre_by_weather(wk).or(tables::knock_timbre_by_weather("normal")).unwrap();
            let stagger = self.knock_flash_t > 0.3
                || self.door_opened_flash_t > 0.3
                || self.elevator_ding_flash_t > 0.3
                || self.voice_sample_flash_t > 0.3
                || self.static_burst_flash_t > 0.3;
            let age =
                if self.ac.is_some() && self.last_rare_fire_t > 0.0 { fx(self.ac_time() - self.last_rare_fire_t, 1) } else { "-".into() };
            let ta = |k: &str| ns(sum_vals(self.rare_event_tally.get(k)));
            let settle_cd = if self.settle_timer > 0.0 { format!("{}s", fx(self.settle_timer, 1)) } else { "-".into() };
            let door_dim = tables::door_dim_by_weather(wk).or(tables::door_dim_by_weather("normal")).unwrap();
            let ding_br = tables::ding_bright_by_weather(wk).or(tables::ding_bright_by_weather("normal")).unwrap();
            let static_ctr = tables::static_chrom_by_weather(wk).or(tables::static_chrom_by_weather("normal")).unwrap().contrast;
            let voice_hue_r = tables::voice_hue_by_weather(wk).or(tables::voice_hue_by_weather("normal")).unwrap().r;
            let _ = write!(s, "─ rare events (R220–R227) ─\n  knock  cd={}s  flashT={}  tintT={}  lastPan={}\n  timbre  freq={}Hz  noise={}Hz  gainMul={}\n  door   cd={}s  flashT={}  dimT={}\n  ding   cd={}s  flashT={}  brightT={}\n  voice  cd={}s  flashT={}\n  static cd={}s  flashT={}\n  shift  cd={}s  envT={}  settleCd={}\n  patt   cd={}s  envT={}\n  skip   cd={}s  envT={}\n  peaks  doorDim={}  dingBr={}  staticCtr={}  voiceHueR={}\n  meta   stagger={}  lastFireT={}s  age={}s\n  tally  k={}  d={}  b={}  v={}  s={}  sh={}  pt={}  sk={}\n",
                fx(self.distant_knock_timer, 0), fx(self.knock_flash_t, 2), fx(self.knock_tint_t, 2), fx(self.last_knock_pan, 2),
                ns(kt.freq), ns(kt.noise), ns(kt.gain_mul),
                fx(self.door_opened_timer, 0), fx(self.door_opened_flash_t, 2), fx(self.door_dim_pulse_t, 2),
                fx(self.elevator_ding_timer, 0), fx(self.elevator_ding_flash_t, 2), fx(self.ding_bright_pulse_t, 2),
                fx(self.voice_sample_timer, 0), fx(self.voice_sample_flash_t, 2),
                fx(self.static_burst_timer, 0), fx(self.static_burst_flash_t, 2),
                fx(self.reality_shift_timer, 0), fx(self.reality_shift_t, 2), settle_cd,
                fx(self.pattern_break_timer, 0), fx(self.pattern_break_t, 2),
                fx(self.clock_skip_timer, 0), fx(self.clock_skip_t, 2),
                fx(door_dim, 2), fx(ding_br, 2), fx(static_ctr, 2), fx(voice_hue_r, 2),
                yn(stagger), fx(self.last_rare_fire_t, 2), age,
                ta("knock"), ta("door"), ta("ding"), ta("voice"), ta("static"), ta("shift"), ta("pattern"), ta("skip"));
            // R232 full per-weather rare-event tally block
            let evs = [
                ("knock", "k"),
                ("door", "d"),
                ("ding", "b"),
                ("voice", "v"),
                ("static", "s"),
                ("shift", "sh"),
                ("pattern", "pt"),
                ("skip", "sk"),
            ];
            let rows = evs
                .iter()
                .map(|(e, ab)| {
                    let t = self.rare_event_tally.get(*e);
                    let g = |k: &str| ns(t.and_then(|t| t.get(k)).copied().unwrap_or(0.0));
                    format!(
                        "  {}byW  wet={} still={} elec={} haun={} der={} norm={}",
                        ab,
                        g("wet"),
                        g("still"),
                        g("electric"),
                        g("haunted"),
                        g("derelict"),
                        g("normal")
                    )
                })
                .collect::<Vec<_>>()
                .join("\n");
            let _ = writeln!(s, "{}", rows);
        }
        // ─ death ─
        {
            let age = if self.ac.is_some() && self.last_death_t > 0.0 { fx(self.ac_time() - self.last_death_t, 1) } else { "-".into() };
            let cause = self.last_death_cause.clone().unwrap_or_else(|| "-".into());
            let ent_sum = sum_vals(self.death_tally.get("entity"));
            let san_sum = sum_vals(self.death_tally.get("sanity"));
            let _ = write!(
                s,
                "─ death (R208/R209/R211) ─\n  count={}  lastCause={}  lastT={}s  age={}s\n  tally  entity={}  sanity={}\n",
                ns(self.death_count),
                cause,
                fx(self.last_death_t, 2),
                age,
                ns(ent_sum),
                ns(san_sum)
            );
        }
        // ─ cry envelope ─
        {
            let active = yn(self.cry_dim_t > 0.0);
            let remain = if self.cry_dim_t > 0.0 { fx(self.cry_dim_t, 1) } else { "-".into() };
            let cool = if self.cry_cooldown > 0.0 { fx(self.cry_cooldown, 1) } else { "-".into() };
            let vig_peak = tables::cry_vignette_peak_by_weather(wk).or(tables::cry_vignette_peak_by_weather("normal")).unwrap();
            let beam_peak = tables::cry_beam_peak_by_weather(wk).or(tables::cry_beam_peak_by_weather("normal")).unwrap();
            let panel_peak = tables::cry_panel_peak_by_weather(wk).or(tables::cry_panel_peak_by_weather("normal")).unwrap();
            let l = self.cry_dim_lerp;
            let _ = write!(s, "─ cry envelope (R195–R199) ─\n  active={}  remain={}s  cool={}s  lerp={}\n  peaks  beam={}  panel={}  vig={}\n  beam×{}  panel×{}  audio×{}  vig+{}\n",
                active, remain, cool, fx(l, 2), fx(beam_peak, 2), fx(panel_peak, 2), fx(vig_peak, 2),
                fx(1.0 - beam_peak * l, 3), fx(1.0 - panel_peak * l, 3), fx(1.0 - 0.25 * l, 3), fx(l * vig_peak, 3));
        }
        // ─ ambient tally ─
        {
            let body = if self.ambient_tally.is_empty() {
                "  none".to_string()
            } else {
                format!(
                    "  {}",
                    self.ambient_tally.iter().map(|(k, per)| format!("{}={}", k, ns(per.values().sum()))).collect::<Vec<_>>().join("  ")
                )
            };
            let _ = write!(s, "─ ambient tally (R187/R188 session totals) ─\n{}\n", body);
        }
        // ─ ambient log ─ (R127–R130 ASCII compass rose; markers fade over ~30 s)
        {
            let names = ["NW", "N", "NE", "W", "C", "E", "SW", "S", "SE"];
            let mut cells: Vec<char> = vec!['.', '.', '.', '.', 'o', '.', '.', '.', '.'];
            let now = self.clock_ms() / 1000.0;
            let mut positional: Vec<&crate::base::records::AmbientLogEntry> =
                self.ambient_log.iter().filter(|e| e.bearing.as_deref().map(|b| !b.is_empty()).unwrap_or(false)).collect();
            positional.reverse();
            let strength = |m: char| match m {
                '.' => 0,
                '+' => 1,
                '*' => 2,
                _ => 99,
            };
            for e in &positional {
                let age = now - e.at;
                if age >= 30.0 {
                    continue;
                }
                let mut m = if age < 5.0 {
                    '*'
                } else if age < 15.0 {
                    '+'
                } else {
                    '.'
                };
                let d = e.dist.filter(|d| *d != 0.0).unwrap_or(999.0);
                if d >= 10.0 && m == '*' {
                    m = '+';
                }
                if d >= 15.0 && m != '.' {
                    m = '.';
                }
                if let Some(ci) = names.iter().position(|n| Some(*n) == e.bearing.as_deref()) {
                    if strength(m) > strength(cells[ci]) {
                        cells[ci] = m;
                    }
                }
            }
            let last = positional.first();
            let dist_str = last.map(|l| format!(" {}m", fx(l.dist.unwrap_or(f64::NAN), 1))).unwrap_or_default();
            let last_s =
                last.map(|l| format!("{} {}{}", l.tag, l.bearing.clone().unwrap_or_default(), dist_str)).unwrap_or_else(|| "-".into());
            let _ = write!(
                s,
                "─ ambient log ─\n  compass  {}{}{}   last: {}\n           {}{}{}\n           {}{}{}\n",
                cells[0], cells[1], cells[2], last_s, cells[3], cells[4], cells[5], cells[6], cells[7], cells[8]
            );
            let log = if self.ambient_log.is_empty() {
                "  none".to_string()
            } else {
                self.ambient_log
                    .iter()
                    .enumerate()
                    .map(|(i, e)| {
                        let pos = match &e.bearing {
                            Some(b) => format!(" @ {} {}m", b, fx(e.dist.unwrap_or(f64::NAN), 1)),
                            None => String::new(),
                        };
                        let pan = match e.pan {
                            Some(p) => format!(" pan={}{}", if p >= 0.0 { "+" } else { "" }, fx(p, 2)),
                            None => String::new(),
                        };
                        format!("  #{} t-{}s {}{}{}", i, fx(now - e.at, 1), e.tag, pos, pan)
                    })
                    .collect::<Vec<_>>()
                    .join("\n")
            };
            let _ = writeln!(s, "{}", log);
        }
        // ─ candles ─ (the rate cell pushes the R87 rolling delta)
        {
            let rate = {
                let denom = self.candle_fizzle_tally + self.candle_relight_tally;
                if denom == 0.0 {
                    "-".to_string()
                } else {
                    let r = self.candle_fizzle_tally / denom;
                    let target = tables::fizzle_prob_by_weather(wk).or(tables::fizzle_prob_by_weather("normal")).unwrap();
                    let delta = r - target;
                    let avg_delta = self.push_rate_delta(delta).unwrap_or(delta);
                    let drift_now = denom >= tables::RATE_DRIFT_MIN_N && avg_delta.abs() > tables::RATE_DRIFT_THRESHOLD;
                    self.rate_drift_ticks = if drift_now { self.rate_drift_ticks + 1.0 } else { 0.0 };
                    let sign = if avg_delta >= 0.0 { "+" } else { "" };
                    let marker = if self.rate_drift_ticks >= tables::RATE_DRIFT_DEBOUNCE {
                        format!("*!{}{}*", sign, fx(avg_delta, 2))
                    } else {
                        String::new()
                    };
                    fx(r, 2) + &marker
                }
            };
            let tail = tables::tail_beat_specs(wk).or(tables::tail_beat_specs("normal")).unwrap().len();
            let spark = tables::relight_specs(wk).or(tables::relight_specs("normal")).unwrap().len();
            let echo_p = tables::fizzle_echo_prob_by_weather(wk).or(tables::fizzle_echo_prob_by_weather("normal")).unwrap();
            let fiz_p = tables::fizzle_prob_by_weather(wk).or(tables::fizzle_prob_by_weather("normal")).unwrap();
            let _ = writeln!(
                s,
                "─ candles ({}) blowouts={} relights={} fizzles={} rate={}  tail={}  spark={}  echoP={}  fizP={} ─",
                self.haunted_candles.len(),
                ns(self.candle_blowout_tally),
                ns(self.candle_relight_tally),
                ns(self.candle_fizzle_tally),
                rate,
                tail,
                spark,
                fx(echo_p, 2),
                fx(fiz_p, 2)
            );
            let avg = match self.current_rate_avg() {
                None => "-".to_string(),
                Some(a) => {
                    let sign = if a >= 0.0 { "+" } else { "" };
                    // R90: remaining debounce ticks before the marker fires
                    let pre_fire = a.abs() > tables::RATE_DRIFT_THRESHOLD && self.rate_drift_ticks < tables::RATE_DRIFT_DEBOUNCE;
                    let countdown =
                        if pre_fire { format!("({})", ns(tables::RATE_DRIFT_DEBOUNCE - self.rate_drift_ticks)) } else { String::new() };
                    format!("{}{}{}", sign, fx(a, 2), countdown)
                }
            };
            let mm = if self.rate_delta_buf.is_empty() {
                String::new()
            } else {
                let mn = self.rate_delta_buf.iter().cloned().fold(f64::INFINITY, f64::min);
                let mx = self.rate_delta_buf.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
                format!("  max={}{} min={}{}", if mx >= 0.0 { "+" } else { "" }, fx(mx, 2), if mn >= 0.0 { "+" } else { "" }, fx(mn, 2))
            };
            let _ = writeln!(
                s,
                "─ tuning driftT={} minN={} deb={}({}) win={}({}) avg={}{} minOp={} ─",
                ns(tables::RATE_DRIFT_THRESHOLD),
                ns(tables::RATE_DRIFT_MIN_N),
                ns(tables::RATE_DRIFT_DEBOUNCE),
                ns(self.rate_drift_ticks),
                ns(tables::RATE_DELTA_WINDOW),
                self.rate_delta_buf.len(),
                avg,
                mm,
                ns(tables::WHISPER_MIN_OPACITY)
            );
            let rows = self
                .haunted_candles
                .iter()
                .enumerate()
                .map(|(i, c)| {
                    let extinguished = c.b.get("extinguished").copied().unwrap_or(!c.lit);
                    let lit = if extinguished { "OUT" } else { "LIT" };
                    let hc = c.f.get("hintCooldown").copied().unwrap_or(0.0);
                    let cd = if hc > 0.0 { fx(hc, 1) } else { "-".into() };
                    let near = if c.b.get("wasNear").copied().unwrap_or(false) { "IN " } else { "   " };
                    format!("  #{} {} streak={} hintCD={} {}", i, lit, ns(c.f.get("fizzleStreak").copied().unwrap_or(0.0)), cd, near)
                })
                .collect::<Vec<_>>()
                .join("\n");
            let _ = writeln!(s, "{}", if rows.is_empty() { "  none".to_string() } else { rows });
        }
        // ─ notes ─
        {
            let all_read = self.lore_found.len() >= LORE_ENTRIES.len();
            let all_arcs = self.lore_arcs_completed.len() >= LORE_ARCS.len();
            let flag = if all_read && all_arcs {
                "  *ARCHIVE*"
            } else if all_read {
                "  *READ*"
            } else if all_arcs {
                "  *ALL*"
            } else {
                ""
            };
            let _ = writeln!(
                s,
                "─ notes ({}) read={}/{} arcs={}/{}{} ─",
                self.lore_notes.len(),
                self.lore_found.len(),
                LORE_ENTRIES.len(),
                self.lore_arcs_completed.len(),
                LORE_ARCS.len(),
                flag
            );
            let mut rows: Vec<(usize, usize, Option<usize>)> =
                self.lore_notes.iter().enumerate().map(|(i, n)| (i, n.id, self.lore_arc_for(n.id))).collect();
            // R74: sort by arc name (no arc sorts as '~', before every arc), then by id.
            let key = |a: Option<usize>| -> (u8, &'static str) {
                match a {
                    Some(k) => (1, LORE_ARCS[k].name),
                    None => (0, "~"),
                }
            };
            rows.sort_by(|a, b| {
                let (ka, kb) = (key(a.2), key(b.2));
                if ka == kb {
                    a.1.cmp(&b.1)
                } else {
                    ka.cmp(&kb)
                }
            });
            let body = rows
                .iter()
                .map(|&(i, id, arc)| {
                    let n = &self.lore_notes[i];
                    let status = if n.taken {
                        "TKN"
                    } else if self.lore_found.contains(&id) {
                        "RD "
                    } else {
                        "NEW"
                    };
                    let wc = n.g.r().user.f.get("whisperCooldown").copied().unwrap_or(0.0);
                    let wcd = if wc > 0.0 { fx(wc, 1) } else { "-".into() };
                    let near = if n.g.r().user.get("wasNearRead") != 0.0 { "IN " } else { "   " };
                    let wf = self.whisper_for(arc, None);
                    format!(
                        "  #{} id={} {} arc={} whCD={} dur={} op={} {}",
                        i,
                        pad_start(&id.to_string(), 2, '0'),
                        status,
                        pad_end(arc.map(|k| LORE_ARCS[k].name).unwrap_or("-"), 11),
                        wcd,
                        fx(wf.duration, 1),
                        fx(wf.opacity, 2),
                        near
                    )
                })
                .collect::<Vec<_>>()
                .join("\n");
            let _ = writeln!(s, "{}", if body.is_empty() { "  none".to_string() } else { body });
        }
        self.ui.debug.set_text(&s);
    }

    // ------------------------------------------------------------ grain
    pub fn update_grain(&mut self, dt: f64) {
        let Some(grain) = self.grain_canvas.clone() else { return };
        // R769 smooth sanity-driven opacity every frame (not just on refresh)
        let s = self.player.sanity / SAN_MAX;
        let grain_target = 0.08 + (1.0 - s) * 0.28;
        let ab_scalar = (1.0 - s).powf(1.8);
        let aberration_target = clamp(ab_scalar * 1.1, 0.0, 0.95);
        // Lerp toward target at ~2Hz half-life
        let k_lerp = (dt * 4.5).min(1.0);
        self.grain_opacity_cur += (grain_target - self.grain_opacity_cur) * k_lerp;
        self.aberration_opacity_cur += (aberration_target - self.aberration_opacity_cur) * k_lerp;
        let (go, ao) = (self.grain_opacity_cur, self.aberration_opacity_cur);
        self.ui.grain.set_opacity(go);
        self.ui.aberration.set_opacity(ao);
        // Grain texture regenerates ~18fps.
        self.grain_t += dt;
        if self.grain_t < 0.055 {
            return;
        }
        self.grain_t = 0.0;
        {
            let mut ctx = grain.borrow_mut();
            let (w, h) = (ctx.width() as usize, ctx.height() as usize);
            let mut img = ctx.create_image_data(w as f64, h as f64);
            let d = &mut img.data;
            // R769 smoother grain — mix each row ~30% with the row above
            let mut prev_row: Option<Vec<u8>> = None;
            for y in 0..h {
                let row_start = y * w * 4;
                for x in 0..w {
                    let i = row_start + x * 4;
                    let mut v = irand(0, 255) as f64;
                    if let Some(pr) = &prev_row {
                        v = to_i32(v * 0.7 + pr[x] as f64 * 0.3) as f64;
                    }
                    let v = v as u8;
                    d[i] = v;
                    d[i + 1] = v;
                    d[i + 2] = v;
                    d[i + 3] = 255;
                }
                let pr = prev_row.get_or_insert_with(|| vec![0u8; w]);
                for x in 0..w {
                    pr[x] = d[row_start + x * 4];
                }
            }
            ctx.put_image_data(&img, 0, 0);
        }
        // Body-class swap: CSS-filter saturation drop on the 3D view at low sanity
        let want_low = s < 0.55;
        let want_crit = s < 0.28;
        let b = &mut self.ui.body;
        if b.has_class("crit-sanity") != want_crit {
            b.toggle_class("crit-sanity", want_crit);
        }
        if b.has_class("low-sanity") != (want_low && !want_crit) {
            b.toggle_class("low-sanity", want_low && !want_crit);
        }
    }

    // ------------------------------------------------------------ messages
    pub fn show_msg(&mut self, text: &str, dur: f64) {
        self.msg_text = text.to_string();
        self.ui.msg.set_text(text);
        self.ui.msg.set_opacity(1.0);
        self.ui.msg.set_style("color", ""); // R73: clear any prior whisper-flash tint
        self.ui.msg.set_style("textShadow", ""); // R74: clear any prior whisper arc glow
        self.msg_t = dur;
    }
    /// A random line from the `key` text pool (one `choice` draw)
    pub fn pick_text(&self, key: &str) -> String {
        match text_pool(key) {
            Some(p) => choice(p).to_string(),
            None => String::new(),
        }
    }

    // ------------------------------------------------------------ anomalies
    pub fn scp_arrival(&mut self) {
        let Some(scp) = self.level_scp.clone() else { return };
        if let Some(text) = scp_omen(&scp) {
            self.show_omen(text, 1.7, 7.0);
        }
        if self.ac.is_none() {
            return;
        }
        fn wait(g: &mut Game, s: f64, f: impl FnOnce(&mut Game) + 'static) {
            g.schedule(s * 1000.0, move |g| {
                if g.state == "playing" {
                    f(g);
                }
            });
        }
        match scp.as_str() {
            "store" => wait(self, 2.0, |g| {
                g.blip(659.0, 0.9, "sine", 0.035, None, 0.0, 0.0);
                g.blip(523.0, 1.4, "sine", 0.03, None, 0.45, 0.0);
            }),
            "house" => wait(self, 2.4, |g| g.sfx_door()),
            "loop" => {
                for t in [1.8, 3.4, 5.0] {
                    wait(self, t, |g| g.blip(1900.0, 0.03, "square", 0.02, None, 0.0, 0.0));
                }
            }
            "stairwell" => wait(self, 2.2, |g| g.distant_sob()),
            _ => {}
        }
    }
    /// One sob from somewhere far off: a throwaway child voice through the reverb
    pub fn distant_sob(&mut self) {
        let Some(sc) = self.soundscape() else { return };
        let v = sc.create_child_voice(None);
        v.set_level(0.14, 0.3);
        let d = v.say("sob", None, 1.0, 0.6);
        self.schedule((d + 1.2) * 1000.0, move |_| v.stop());
    }

    // ------------------------------------------------------------ lore arcs
    /// Index into LORE_ARCS
    pub fn lore_arc_for(&self, id: usize) -> Option<usize> {
        LORE_ARCS.iter().position(|a| a.ids.contains(&id))
    }
    /// (R78)
    pub fn whisper_duration_for(&self, arc: Option<usize>) -> f64 {
        let w = match arc {
            Some(a) => (LORE_ARCS[a].ids.len() as f64 / 3.0).min(1.0).max(0.5),
            None => 1.0,
        };
        0.6 + 0.6 * w
    }
    /// (R79)
    pub fn whisper_glow_for(&self, arc: Option<usize>) -> Option<WhisperGlow> {
        let a = &LORE_ARCS[arc?];
        let w = (a.ids.len() as f64 / 3.0).min(1.0).max(0.5);
        Some(WhisperGlow {
            hex: hex6(a.color),
            alpha: 0.45 * w,
            blur: 6.0 * w,
            r: (a.color >> 16) & 0xff,
            g: (a.color >> 8) & 0xff,
            b: a.color & 0xff,
        })
    }
    pub fn whisper_for(&self, arc: Option<usize>, id: Option<usize>) -> Whisper {
        let w = match arc {
            Some(a) => (LORE_ARCS[a].ids.len() as f64 / 3.0).min(1.0).max(0.5),
            None => 1.0,
        };
        let glow = self.whisper_glow_for(arc);
        let mut out = Whisper {
            duration: self.whisper_duration_for(arc),
            glow: glow.as_ref().map(|g| g.hex.clone()),
            opacity: tables::WHISPER_MIN_OPACITY.max(0.25 + 0.10 * w), // 0.30 at w=0.5, 0.35 at w=1.0
            text: None,
            glow_alpha: glow.as_ref().map(|g| g.alpha).unwrap_or(0.0),
            glow_blur: glow.as_ref().map(|g| g.blur).unwrap_or(0.0),
            glow_rgb: glow.as_ref().map(|g| [g.r, g.g, g.b]).unwrap_or([0; 3]),
            glow_shadow: glow.as_ref().map(|g| format!("0 0 {}px rgba({}, {}, {}, {})", fx(g.blur, 1), g.r, g.g, g.b, fx(g.alpha, 2))),
        };
        if let Some(id) = id {
            if let Some(entry) = LORE_ENTRIES.get(id) {
                out.text = Some(format!("...  {}  ...", entry.title.to_lowercase()));
            }
        }
        out
    }
    /// Is any earlier id of an arc containing `id` found?
    pub fn lore_note_boosted(&self, id: usize) -> bool {
        for arc in LORE_ARCS.iter() {
            let idx = match arc.ids.iter().position(|&i| i == id) {
                Some(i) => i,
                None => continue,
            };
            if idx == 0 {
                continue;
            } // the first — no boost
            for k in 0..idx {
                if self.lore_found.contains(&arc.ids[k]) {
                    return true;
                }
            }
        }
        false
    }

    // ------------------------------------------------------------ medals
    /// the shared "medal earned" beat: HUD medal on, the bloom flash for 0.9 s
    fn medal_on_and_flash(&mut self, id: &'static str) {
        if let Some(m) = self.ui.by_id(id) {
            m.add_class("on");
        }
        self.ui.achievement_flash.add_class("on");
        self.schedule(900.0, |g| g.ui.achievement_flash.remove_class("on"));
    }
    pub fn fire_storyteller_achievement(&mut self) {
        if self.storyteller_shown {
            return;
        }
        self.storyteller_shown = true;
        self.medal_on_and_flash("storytellerMedal");
        self.show_msg("STORYTELLER — EVERY THREAD IS PULLED", 3.2);
        if self.ac.is_some() {
            // triad rising + one high shimmer note
            self.blip(523.0, 1.6, "sine", 0.10, None, 0.0, 0.0);
            self.blip(659.0, 1.9, "sine", 0.08, None, 0.10, 0.0);
            self.blip(784.0, 2.2, "sine", 0.08, None, 0.20, 0.0);
            self.blip(1976.0, 2.5, "sine", 0.05, None, 0.40, 0.0);
        }
        self.save_game();
        self.check_completionist();
    }
    pub fn refresh_storyteller_medal(&mut self) {
        let on = self.storyteller_shown;
        self.ui.storyteller_medal.toggle_class("on", on);
    }
    pub fn fire_ironwill_achievement(&mut self) {
        if self.ironwill_shown {
            return;
        }
        self.ironwill_shown = true;
        self.medal_on_and_flash("ironwillMedal");
        self.show_msg("IRONWILL — FIVE FLOORS, NO DEATHS", 3.2);
        if self.ac.is_some() {
            // steady triad — grounded, resolute
            self.blip(523.0, 1.7, "sine", 0.10, None, 0.0, 0.0);
            self.blip(659.0, 1.9, "sine", 0.10, None, 0.10, 0.0);
            self.blip(1046.0, 2.2, "sine", 0.07, None, 0.24, 0.0);
        }
        self.save_game();
    }
    pub fn refresh_ironwill_medal(&mut self) {
        let on = self.ironwill_shown;
        self.ui.ironwill_medal.toggle_class("on", on);
    }
    pub fn fire_completionist_achievement(&mut self) {
        if self.completionist_shown {
            return;
        }
        self.completionist_shown = true;
        self.medal_on_and_flash("completionistMedal");
        self.show_msg("COMPLETIONIST — YOU HAVE SEEN IT ALL", 3.6);
        if self.ac.is_some() {
            // rich prestige chord — five notes, both directions
            self.blip(392.0, 1.6, "sine", 0.08, None, 0.0, 0.0);
            self.blip(659.0, 1.9, "sine", 0.09, None, 0.08, 0.0);
            self.blip(880.0, 2.2, "sine", 0.09, None, 0.16, 0.0);
            self.blip(1319.0, 2.5, "sine", 0.08, None, 0.24, 0.0);
            self.blip(1976.0, 2.8, "sine", 0.06, None, 0.32, 0.0);
        }
        self.save_game();
    }
    pub fn refresh_completionist_medal(&mut self) {
        let on = self.completionist_shown;
        self.ui.completionist_medal.toggle_class("on", on);
    }
    /// After a save-load, re-flash each earned medal (staggered)
    pub fn replay_medal_highlight(&mut self) {
        let staggered: [(&'static str, bool, f64); 5] = [
            ("loreMedal", self.lore_archivist_shown, 0.0),
            ("survivorMedal", self.survivor_shown, 180.0),
            ("storytellerMedal", self.storyteller_shown, 360.0),
            ("completionistMedal", self.completionist_shown, 540.0),
            ("ironwillMedal", self.ironwill_shown, 720.0),
        ];
        for (id, shown, delay) in staggered {
            if !shown {
                continue;
            }
            self.schedule(delay, move |g| {
                if let Some(el) = g.ui.by_id(id) {
                    el.remove_class("medal-replay"); // force restart if it was already animating
                    el.add_class("medal-replay");
                }
                // clean up class after animation ends (1.4s)
                g.schedule(1500.0, move |g| {
                    if let Some(el) = g.ui.by_id(id) {
                        el.remove_class("medal-replay");
                    }
                });
            });
        }
    }
    pub fn check_completionist(&mut self) {
        if self.lore_archivist_shown && self.survivor_shown && self.storyteller_shown && !self.completionist_shown {
            // small defer so it doesn't clobber the just-fired medal's toast
            self.schedule(4500.0, |g| g.fire_completionist_achievement());
        }
    }
    pub fn fire_survivor_achievement(&mut self) {
        if self.survivor_shown {
            return;
        }
        self.survivor_shown = true;
        self.medal_on_and_flash("survivorMedal");
        self.show_msg("SURVIVOR — REACHED LEVEL 10", 3.2);
        if self.ac.is_some() {
            // cooler chord than the archivist chime — descending sines
            self.blip(1760.0, 1.8, "sine", 0.10, None, 0.0, 0.0);
            self.blip(1320.0, 2.0, "sine", 0.10, None, 0.10, 0.0);
            self.blip(880.0, 2.2, "sine", 0.10, None, 0.20, 0.0);
            self.blip(440.0, 2.4, "sine", 0.10, None, 0.30, 0.0);
        }
        self.save_game();
        self.check_completionist();
    }
    pub fn refresh_survivor_medal(&mut self) {
        let on = self.survivor_shown;
        self.ui.survivor_medal.toggle_class("on", on);
    }
    pub fn fire_archivist_achievement(&mut self) {
        if self.lore_archivist_shown {
            return;
        }
        self.lore_archivist_shown = true;
        self.medal_on_and_flash("loreMedal");
        self.show_msg("ARCHIVIST — ALL 30 NOTES FOUND", 3.2);
        if self.ac.is_some() {
            // brighter chime chord than the standard `sfx_chime`
            self.blip(880.0, 1.6, "sine", 0.13, None, 0.0, 0.0);
            self.blip(1320.0, 1.9, "sine", 0.10, None, 0.10, 0.0);
            self.blip(1760.0, 2.2, "sine", 0.08, None, 0.20, 0.0);
            self.blip(2640.0, 2.5, "sine", 0.05, None, 0.30, 0.0);
        }
        self.check_completionist();
    }
    pub fn refresh_lore_medal(&mut self) {
        let on = self.lore_archivist_shown;
        self.ui.lore_medal.toggle_class("on", on);
    }

    // ------------------------------------------------------------ overlay
    pub fn show_overlay(&mut self, title: &str, sub: &str, hint: &str) {
        self.ui.overlay_title.set_text(title);
        self.ui.overlay_sub.set_text(sub);
        self.ui.overlay_hint.set_text(hint);
        self.ui.overlay.remove_class("hidden");
        // Buttons only visible on the main menu; pause/death overlays skip them
        let disp = if self.state == "menu" { "flex" } else { "none" };
        self.ui.overlay_buttons.set_display(disp);
    }
    pub fn hide_overlay(&mut self) {
        self.ui.overlay.add_class("hidden");
    }

    // ------------------------------------------------------------ per frame
    /// The meters, counters, compass, toast timer, the
    /// interact prompt, then every prop animator and the TV / phone / clock.
    pub fn update_hud(&mut self, dt: f64) {
        let p = self.player.clone();
        self.ui.staminabar.set_style("width", &format!("{}%", ns(p.stamina)));
        self.ui.stamina.toggle_class("low", p.stamina < 25.0);
        self.ui.sanitybar.set_style("width", &format!("{}%", ns(p.sanity)));
        self.ui.sanity.toggle_class("low", p.sanity < 30.0);
        self.ui.batterybar.set_style("width", &format!("{}%", ns(p.battery)));
        self.ui.battery.toggle_class("low", p.battery < 25.0);
        self.ui.battery.toggle_class("off", !p.flashlight_on);
        let bt = if p.bottles > 0 { format!("ALMOND WATER ×{}  {}", p.bottles, self.item_hint_water) } else { String::new() };
        self.ui.bottles.set_text(&bt);
        let ct = if p.batteries > 0 { format!("BATTERIES ×{}  {}", p.batteries, self.item_hint_cell) } else { String::new() };
        self.ui.batteries.set_text(&ct);
        {
            let paper_txt = if p.papers > 0 { format!("PAPER ×{}  [Q] DROP", p.papers) } else { String::new() };
            let note_txt = if !self.lore_found.is_empty() {
                format!("  ·  NOTES {}/{}  [J]", self.lore_found.len(), LORE_ENTRIES.len())
            } else {
                String::new()
            };
            self.ui.papers.set_text(&(paper_txt + &note_txt));
        }
        self.ui.crouch_ind.toggle_class("on", p.crouch);

        // compass to exit door: arrow rotation is clockwise angle from player-forward
        if let Some(ex) = &self.exit_data {
            let dx = ex.pos.x - p.pos.x;
            let dz = ex.pos.z - p.pos.z;
            let dist_to_exit = dx.hypot(dz);
            let (fx_, fz) = (-p.yaw.sin(), -p.yaw.cos());
            let (rx, rz) = (p.yaw.cos(), -p.yaw.sin());
            let fwd_c = dx * fx_ + dz * fz;
            let right_c = dx * rx + dz * rz;
            let rel = right_c.atan2(fwd_c);
            self.ui.compass_arrow.set_style("transform", &format!("translate(-50%,-100%) rotate({}rad)", ns(rel)));
            // fade out at extreme distance so navigation stays a game
            let mut vis = clamp(1.0 - (dist_to_exit - 6.0) / 40.0, 0.15, 1.0);
            // R769 phone-answer hint: temporary boost to full opacity + pulse.
            if let Some(hint_until) = self.phone_data.as_ref().and_then(|ph| ph.f.get("hintUntil").copied()) {
                if hint_until != 0.0 && self.perf_t < hint_until {
                    let remaining = hint_until - self.perf_t;
                    let pulse = 0.7 + 0.3 * (self.perf_t * 4.0).sin();
                    vis = vis.max(pulse * (remaining / 2.0).min(1.0));
                }
            }
            self.ui.compass_arrow.set_opacity(vis);
        } else {
            self.ui.compass_arrow.set_opacity(0.0);
        }

        if self.msg_t > 0.0 {
            self.msg_t -= dt;
            if self.msg_t <= 0.0 {
                self.ui.msg.set_opacity(0.0);
                self.ui.msg.set_style("color", "");
                self.ui.msg.set_style("textShadow", "");
            }
        }
        // nearest enabled interactable
        let mut prompt = String::new();
        for it in &self.interactables {
            if !(it.enabled)(self) {
                continue;
            }
            if it.pos.distance_to_squared(&self.player.pos) < it.r * it.r {
                prompt = it.prompt.clone();
                break;
            }
        }
        self.ui.prompt.set_text(&prompt);
        // prop animation + proximity devices
        let t = self.perf_t;
        self.run_hud_animators(dt, t);
        if let Some(tv) = self.tv_data.clone() {
            let d2 = tv.world.distance_to_squared(&self.player.pos);
            tv.light.set_intensity(if d2 < 900.0 { rand2(0.0, 2.6) } else { 0.0 });
            let mut redraw = false;
            if let Some(tvm) = self.tv_data.as_mut() {
                tvm.acc += dt;
                if d2 < 900.0 && tvm.acc > 0.09 {
                    tvm.acc = 0.0;
                    redraw = true;
                }
            }
            if redraw {
                self.draw_static();
            }
        }
        if self.phone_data.as_ref().map(|ph| !ph.answered).unwrap_or(false) {
            let pos = self.player.pos;
            let ph = self.phone_data.as_mut().unwrap();
            let d = ph.world.distance_to_squared(&pos).sqrt();
            if d < 9.0 && !ph.ringing {
                ph.ringing = true;
            }
            ph.ring_t += dt;
            let on = ph.ringing && (ph.ring_t % 2.6) < 1.1;
            if let Some(pg) = &self.audio.phone_gain {
                let cur = pg.gain().value();
                pg.gain().set_value(cur + ((if on { clamp(0.14 - d * 0.012, 0.0, 0.14) } else { 0.0 }) - cur) * (dt * 10.0).min(1.0));
            }
        }
        if self.clock_data.is_some() {
            let pos = self.player.pos;
            let mut tick: Option<f64> = None;
            {
                let cd = self.clock_data.as_mut().unwrap();
                cd.sec_pivot.rotation().z -= dt * (PI * 2.0 / 60.0);
                let d = cd.world.distance_to_squared(&pos).sqrt();
                cd.acc += dt;
                if d < 4.0 && cd.acc > 0.55 {
                    cd.acc = 0.0;
                    cd.tock = !cd.tock;
                    tick = Some(clamp(1.0 - d / 4.0, 0.0, 1.0) * if cd.tock { 1.0 } else { 0.7 });
                }
            }
            if let Some(v) = tick {
                self.sfx_tick(v);
            }
        }
    }
    /// Run every prop animator with (dt, t). The list in effect at the start is
    /// the one iterated; if an animator rebuilt the level (`world` and
    /// `animators` replaced), only the new level's list survives.
    fn run_hud_animators(&mut self, dt: f64, t: f64) {
        let world0 = self.world.clone();
        let mut anims = std::mem::take(&mut self.animators);
        for a in anims.iter_mut() {
            a(self, dt, t);
        }
        if self.world.ptr_eq(&world0) {
            anims.append(&mut self.animators);
            self.animators = anims;
        }
    }
    pub fn try_interact(&mut self) {
        if self.state != "playing" {
            return;
        }
        if self.in_stairwell {
            if self.stair_near_exit {
                self.exit_stairwell();
            }
            return;
        }
        if self.in_loop {
            if self.loop_near_exit {
                self.exit_loop();
            }
            return;
        }
        let pos = self.player.pos;
        let hit = self.interactables.iter().position(|it| (it.enabled)(self) && it.pos.distance_to_squared(&pos) < it.r * it.r);
        let Some(i) = hit else { return };
        // `on_use`: take the closure out while it runs (it may rebuild the level);
        // put it back only if its placeholder is still in the list.
        let marker = Rc::new(());
        let m2 = marker.clone();
        let ph: Box<dyn FnMut(&mut Game)> = Box::new(move |_g| {
            let _ = &m2;
        });
        let ph_ptr = &*ph as *const dyn FnMut(&mut Game) as *const u8;
        let mut f = std::mem::replace(&mut self.interactables[i].on_use, ph);
        f(self);
        if Rc::strong_count(&marker) == 2 {
            if let Some(it) = self.interactables.iter_mut().find(|it| (&*it.on_use as *const dyn FnMut(&mut Game) as *const u8) == ph_ptr) {
                it.on_use = f;
            }
        }
    }

    // ------------------------------------------------------------ HUD layers
    /// The HUD's layers over the 3D view this frame (vignette, tints, flash,
    /// grain, fade, the HUD itself, the menu overlay, the journal), for a
    /// window of `w`×`h` CSS px at `dpr` device pixels per CSS px.
    pub fn hud_layers(&mut self, w: f64, h: f64, dpr: f64) -> Vec<crate::ui::model::HudLayer> {
        crate::ui::draw::hud_layers(self, w, h, dpr)
    }
    /// The element under a click at CSS px (x, y): 'btn-new', 'btn-continue',
    /// 'journal-prev', 'journal-next', 'overlay' or '' (the 3D view).
    pub fn hud_hit_test(&self, x: f64, y: f64) -> String {
        crate::ui::draw::hud_hit_test(self, x, y)
    }
}
