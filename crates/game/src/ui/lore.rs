//! Lore-table accessors used by player.rs, `place_props` and flow.rs, over the
//! HUD module's `LORE_ENTRIES` / `LORE_ARCS`, and `draw_static()`.
use crate::prelude::*;
use crate::ui::hud::{LORE_ARCS, LORE_ENTRIES};

impl Game {
    /// Number of lore entries
    pub fn lore_entries_len(&self) -> usize {
        LORE_ENTRIES.len()
    }
    /// Number of lore entries (same as `lore_entries_len`)
    pub fn lore_entry_count(&self) -> usize {
        LORE_ENTRIES.len()
    }
    /// Title of lore entry `id` (empty when out of range)
    pub fn lore_entry_title(&self, id: usize) -> String {
        LORE_ENTRIES.get(id).map(|e| e.title.to_string()).unwrap_or_default()
    }
    /// `LORE_ARCS` as (name, entry ids), in order
    pub fn lore_arc_list(&self) -> Vec<(String, Vec<usize>)> {
        LORE_ARCS.iter().map(|a| (a.name.to_string(), a.ids.to_vec())).collect()
    }
    /// Refill the TV static canvas and mark its texture for re-upload
    pub fn draw_static(&mut self) {
        self.tex.update_static();
    }
}

impl Game {
    /// The SCP-style tag of lore entry `id`, if any
    pub fn lore_entry_scp(&self, id: usize) -> Option<String> {
        crate::ui::hud::LORE_ENTRIES.get(id).and_then(|e| e.scp).map(|s| s.to_string())
    }
    /// Colour of lore arc `arc`
    pub fn lore_arc_color(&self, arc: usize) -> u32 {
        crate::ui::hud::LORE_ARCS[arc].color
    }
}

/// The salt of the stream that fits a level's notes to the save.
const FIT_SALT: u64 = 0x10AE_F17E_D0C5_0001;

impl Game {
    /// Fit the notes of a level just built to what this run has read.
    ///
    /// `init_level` builds every level as a new run would (nothing read), so
    /// a level is a function of its run seed and number alone and a retry
    /// is the same place. Here, after the build, a note already read is
    /// swapped for one not yet read (chosen on a stream of its own, so the
    /// build's draws never move), a note continuing a story already begun
    /// is written in that story's ink, and a note left read lies with its
    /// corner turned down.
    pub fn fit_lore_notes(&mut self) {
        let n = self.lore_entry_count();
        let placed: std::collections::BTreeSet<usize> = self.lore_notes.iter().map(|x| x.id).collect();
        let free = |g: &Game, i: usize| !g.lore_found.contains(&i) && g.lore_entry_scp(i).is_none() && !placed.contains(&i);
        let mut unread: Vec<usize> = (0..n).filter(|&i| free(self, i)).collect();
        let mut r = SplitMix64::new(self.current_seed, FIT_SALT);
        for i in (1..unread.len()).rev() {
            let j = (r.next() * (i + 1) as f64) as usize;
            unread.swap(i, j);
        }
        for k in 0..self.lore_notes.len() {
            let id = self.lore_notes[k].id;
            if self.lore_found.contains(&id) && self.lore_entry_scp(id).is_none() {
                if let Some(new) = unread.pop() {
                    self.lore_notes[k].id = new;
                }
            }
            let (id, g) = (self.lore_notes[k].id, self.lore_notes[k].g.clone());
            if self.lore_found.contains(&id) {
                Game::turn_down_note(&g);
                continue;
            }
            let Some(arc) = self.lore_arc_for(id).filter(|_| self.lore_note_boosted(id)) else { continue };
            // written in the ink of the story it continues
            Game::ink_note(&g, self.lore_arc_color(arc));
        }
    }
}
