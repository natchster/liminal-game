//! Zone record access for the ambient and soundscape code.
use crate::prelude::*;

impl Game {
    /// The fields of zone `zi` (an index as `zone_at_tile` / `zone_at_world`
    /// return). `None` without a layout (or for a bad index).
    pub fn zone_fields(&self, zi: usize) -> Option<ZoneFields> {
        let z = self.layout.as_ref()?.zones.get(zi)?;
        Some(ZoneFields { arch: z.arch, floor_mat: z.floor_mat, title: z.title, lore: z.lore })
    }
}

/// The fields of an archgen zone record the soundscape driver reads.
#[derive(Clone, Copy, Debug)]
pub struct ZoneFields {
    /// Archetype key (reverb zone name, caption key)
    pub arch: &'static str,
    /// Floor material ('carpet' | 'concrete' | 'tile' | 'office')
    pub floor_mat: &'static str,
    /// Display title (e.g. "THE LOBBY")
    pub title: &'static str,
    /// Lore line (e.g. "Level 0")
    pub lore: &'static str,
}
