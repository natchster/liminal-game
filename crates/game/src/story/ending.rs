//! The bottom of her story. Level 13 (`notebook::LAST_LEVEL`) holds her
//! last camp: her lamp long cold, her shoes set side by side, the last
//! page, and no trail on from it. Leaving that floor the first time (its
//! door, a lift, a hole) ends her story: a card says what you carry down
//! with you, you take what is left of her paper, and the descent goes on
//! without her (no camps and no trails below the last camp).
use super::notebook;
use crate::prelude::*;

/// Sheets of her paper you take at the end
pub const HER_PAPER: i32 = 12;

impl Game {
    /// Whether leaving this floor ends her story (the last camp's floor,
    /// the first time).
    pub fn arc_ends_here(&self) -> bool {
        self.level_num == notebook::LAST_LEVEL && !self.arc_done
    }

    /// The end of her story: the hum's long chord, black, and the card.
    pub fn begin_ending(&mut self) {
        if self.state != "playing" {
            return;
        }
        self.state = "ending".into();
        self.director_note("ending");
        self.sfx_door();
        if self.ac.is_some() {
            for (i, f) in [220.0, 277.2, 329.6, 440.0].into_iter().enumerate() {
                self.blip(f, 3.2 - i as f64 * 0.3, "sine", 0.05 - i as f64 * 0.008, None, 0.4 + i as f64 * 0.35, 0.0);
            }
        }
        self.ui.fade.set_style("background", "#050403");
        self.ui.fade.set_opacity(1.0);
        self.schedule(1800.0, |g| {
            if g.state != "ending" {
                return;
            }
            g.want_pointer_lock = Some(false);
            let (title, body) = g.ending_card();
            // the card alone: not the controls and the tips the overlay keeps
            g.ui.overlay_controls.set_html("");
            g.show_overlay(&title, &body, "PRESS E TO GO ON");
        });
    }

    /// The card at the end of her story: what you carry down with you.
    pub fn ending_card(&self) -> (String, String) {
        let book = notebook::notebook(self.run_seed);
        let pages = book.iter().filter(|(_, p)| self.notebook.contains(p.id)).count();
        let all = if pages == book.len() { ", every one" } else { "" };
        let deaths = match self.death_count as u32 {
            0 => "and you never died on the way down".to_string(),
            1 => "and you died once on the way down".to_string(),
            n => format!("and you died {n} times on the way down"),
        };
        let body = format!(
            "Her paper ends here.\n\nYou have {pages} of her {} pages{all}, {} of the torn notes, {deaths}.\n\nYou take what is left of her paper. From here down, the trail is yours.",
            book.len(),
            self.lore_found.len(),
        );
        ("THE LAST CAMP".into(), body)
    }

    /// Go on down from the end of her story.
    pub fn end_ending(&mut self) {
        if self.state != "ending" {
            return;
        }
        self.arc_done = true;
        self.player.papers += HER_PAPER;
        self.hide_overlay();
        let controls = crate::ui::model::UiModel::new().overlay_controls.html;
        self.ui.overlay_controls.set_html(&controls);
        self.state = "transition".into();
        self.schedule(400.0, |g| {
            g.init_level(g.level_num + 1);
            g.save_game();
            g.state = "playing".into();
            let title = g.level_title();
            g.show_msg(&format!("LEVEL {} — {}", g.level_num, title), 2.6);
            g.ui.fade.set_opacity(0.0);
            g.want_pointer_lock = Some(true);
        });
    }
}
