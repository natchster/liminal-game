//! Lore-table accessors used by player.rs, placeProps and flow.rs, over the
//! HUD module's `LORE_ENTRIES` / `LORE_ARCS`, and `drawStatic()`.
use crate::hud::{LORE_ARCS, LORE_ENTRIES};
use crate::prelude::*;

impl Game {
    /// `LORE_ENTRIES.length` (line 67827)
    pub fn lore_entries_len(&self) -> usize { LORE_ENTRIES.len() }
    /// `LORE_ENTRIES.length`
    pub fn lore_entry_count(&self) -> usize { LORE_ENTRIES.len() }
    /// `LORE_ENTRIES[id].title`
    pub fn lore_entry_title(&self, id: usize) -> String { LORE_ENTRIES.get(id).map(|e| e.title.to_string()).unwrap_or_default() }
    /// `LORE_ARCS` (line 67917) as (arc.name, arc.ids), in order
    pub fn lore_arc_list(&self) -> Vec<(String, Vec<usize>)> { LORE_ARCS.iter().map(|a| (a.name.to_string(), a.ids.to_vec())).collect() }
    /// `drawStatic()` (textures section, line 1011): refill the TV static canvas, `staticTex.needsUpdate = true`
    pub fn draw_static(&mut self) { self.tex.update_static(); }
}
