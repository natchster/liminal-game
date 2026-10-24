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
