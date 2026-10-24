//! The zone record accessor the ambient port needs (`layout.zones[zi]`).
use crate::prelude::*;

impl Game {
    /// The zone record `Z` that `zoneAtTile` / `zoneAtWorld` return
    /// (`layout.zones[zi]`): `Z.arch`, `Z.floorMat`, `Z.title`, `Z.lore`.
    /// `None` without a layout (or for a bad index).
    pub fn zone_fields(&self, zi: usize) -> Option<ZoneFields> {
        let z = self.layout.as_ref()?.zones.get(zi)?;
        Some(ZoneFields { arch: z.arch, floor_mat: z.floor_mat, title: z.title, lore: z.lore })
    }
}

/// The fields of an archgen zone record the soundscape driver reads.
#[derive(Clone, Copy, Debug)]
pub struct ZoneFields {
    /// `Z.arch` (archetype key: reverb zone name, caption key)
    pub arch: &'static str,
    /// `Z.floorMat` ('carpet' | 'concrete' | 'tile' | 'office')
    pub floor_mat: &'static str,
    /// `Z.title`
    pub title: &'static str,
    /// `Z.lore`
    pub lore: &'static str,
}
