//! Per-weather colour tables shared by this crate's placers. Each picks the entry
//! for the level's weather key, else the normal entry; tables whose entries may
//! be absent (the placer keeps its baseline pool) return `Option`.
use game::placers::shared::by_weather;

/// Shoe colour per weather
pub fn shoe_color_by_weather(key: &str) -> u32 {
    by_weather(key, 0x2a2018, 0x3c2818, 0x1a0f0a, 0x4a3020, 0x4a3a28, 0x3c2818)
}
/// Umbrella colour per weather
pub fn umbrella_color_by_weather(key: &str) -> u32 {
    by_weather(key, 0x0a1218, 0x282010, 0x2a0a0a, 0x3a3020, 0x1a1a2a, 0x1a1a1a)
}
/// Soap bar colour per weather (normal: `None` → the 4-colour baseline pool)
pub fn soap_bar_color_by_weather(key: &str) -> Option<&'static [u32]> {
    by_weather(
        key,
        Some(&[0xa8d0d8, 0xf0e8d0][..]),
        Some(&[0xe8d0a0, 0xf0e8d0][..]),
        Some(&[0x8a3040, 0x603030][..]),
        Some(&[0x8a7050, 0xa08060][..]),
        Some(&[0xf0e8d0, 0xd8c0d0][..]),
        None,
    )
}
/// Toothbrush colour per weather (normal: `None` → the 4-colour baseline pool)
pub fn toothbrush_color_by_weather(key: &str) -> Option<&'static [u32]> {
    by_weather(
        key,
        Some(&[0x40a0d8, 0xf0f0f0][..]),
        Some(&[0xd0d060, 0xf0f0f0][..]),
        Some(&[0xd06060, 0x8a3030][..]),
        Some(&[0x8a7040, 0x6a5030][..]),
        Some(&[0xf0f0f0, 0x40a0d8][..]),
        None,
    )
}
/// Oil stain per weather
pub fn oil_stain_by_weather(key: &str) -> u32 {
    by_weather(key, 0x0a0c10, 0x0a0805, 0x1a0605, 0x100a08, 0x0a0806, 0x0a0806)
}
/// Goggle lens per weather
pub fn goggle_lens_by_weather(key: &str) -> u32 {
    by_weather(key, 0x6890a8, 0xd8c880, 0xa87070, 0x907050, 0x8ac0d8, 0x8ac0d8)
}
/// Photo tint per weather
pub fn photo_tint_by_weather(key: &str) -> u32 {
    by_weather(key, 0x8ea0b4, 0xd0b070, 0xa06060, 0xa89870, 0xc4b898, 0xffffff)
}
/// Poster tint per weather
pub fn poster_tint_by_weather(key: &str) -> u32 {
    by_weather(key, 0xa0b0c0, 0xe0c890, 0xb08080, 0xc0b090, 0xd8ccb0, 0xffffff)
}
/// Mirror glass per weather
pub fn mirror_glass_by_weather(key: &str) -> u32 {
    by_weather(key, 0x3a5060, 0x605020, 0x603030, 0x484038, 0x505860, 0x505860)
}
/// Dead plant pot per weather (still / normal: `None` → the baseline)
pub fn dead_plant_pot_by_weather(key: &str) -> Option<&'static [u32]> {
    by_weather(
        key,
        Some(&[0x3a4048, 0x2a3038, 0x4a5058][..]),
        Some(&[0x4a3020, 0x603828, 0x6a4028][..]),
        Some(&[0x3a1010, 0x2a0808, 0x4a1010][..]),
        Some(&[0x5a4830, 0x483828, 0x584030][..]),
        None,
        None,
    )
}
/// Book cover per weather (still / normal: `None` → the baseline)
pub fn book_cover_by_weather(key: &str) -> Option<&'static [u32]> {
    by_weather(
        key,
        Some(&[0x1a3040, 0x203040, 0x2a3438][..]),
        Some(&[0x4a3820, 0x604828, 0x503820][..]),
        Some(&[0x6a2020, 0x4a1010, 0x300808][..]),
        Some(&[0x4a3820, 0x3a3018, 0x584030][..]),
        None,
        None,
    )
}
/// Traffic cone per weather
pub fn traffic_cone_by_weather(key: &str) -> u32 {
    by_weather(key, 0xa04010, 0xf07038, 0x903030, 0x8a4830, 0xd85018, 0xd85018)
}
/// Rug tint per weather
pub fn rug_tint_by_weather(key: &str) -> u32 {
    by_weather(key, 0x707880, 0xd8a860, 0xa04040, 0xa89070, 0xd8c8b0, 0xffffff)
}
/// Ceiling debris per weather
pub fn ceiling_debris_by_weather(key: &str) -> u32 {
    by_weather(key, 0x8890a0, 0xa89880, 0xa88880, 0x988878, 0xa89890, 0xa89890)
}
/// Extension cord per weather (still / normal: `None` → the baseline)
pub fn extension_cord_by_weather(key: &str) -> Option<&'static [u32]> {
    by_weather(
        key,
        Some(&[0x0a0f18, 0x101820][..]),
        Some(&[0x201808, 0x282010][..]),
        Some(&[0x1a0808, 0x201014][..]),
        Some(&[0x181410, 0x201818][..]),
        None,
        None,
    )
}
/// Smoke wisp per weather
pub fn smoke_wisp_by_weather(key: &str) -> u32 {
    by_weather(key, 0x505860, 0x707068, 0x704040, 0x604838, 0x606058, 0x606058)
}
/// Mirror frame per weather
pub fn mirror_frame_by_weather(key: &str) -> u32 {
    by_weather(key, 0x2a3038, 0x2a1a0a, 0x2a1010, 0x2a2418, 0x3a2a18, 0x3a2a18)
}
/// Trash bag colour per weather
pub fn trash_bag_color_by_weather(key: &str) -> u32 {
    by_weather(key, 0x0a0f14, 0x281a0a, 0x1a0808, 0x2a2418, 0x0a0a10, 0x1a1a1e)
}
/// Plant glow per weather
pub fn plant_glow_by_weather(key: &str) -> u32 {
    by_weather(key, 0x40a080, 0x80c040, 0x9040c0, 0xa08040, 0xd0e0d0, 0x60a040)
}
/// Key colour per weather
pub fn key_color_by_weather(key: &str) -> u32 {
    by_weather(key, 0x8a7830, 0xc8b850, 0xa88a40, 0x6a5828, 0xd0d0c8, 0xb8a850)
}
