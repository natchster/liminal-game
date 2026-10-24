//! Lore-table accessors the placeProps tail reads (over hud.rs's tables).
use crate::prelude::*;

impl Game {
    /// `LORE_ENTRIES[id].scp`
    pub fn lore_entry_scp(&self, id: usize) -> Option<String> { crate::hud::LORE_ENTRIES.get(id).and_then(|e| e.scp).map(|s| s.to_string()) }
    /// `LORE_ARCS[arc].color`
    pub fn lore_arc_color(&self, arc: usize) -> u32 { crate::hud::LORE_ARCS[arc].color }
}
