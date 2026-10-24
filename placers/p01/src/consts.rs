//! Module-level per-weather tables of chunk p01 (backrooms.js lines 5352–7201).
//! Each is `TABLE[levelWeather?.key] ?? TABLE.normal`; tables whose entries may
//! be `null` (fall back to a placer's baseline pool) return `Option`.
use game::shared::by_weather;

// line 5564
/// `SHOE_COLOR_BY_WEATHER`
pub fn shoe_color_by_weather(key: &str) -> u32 { by_weather(key, 0x2a2018, 0x3c2818, 0x1a0f0a, 0x4a3020, 0x4a3a28, 0x3c2818) }
/// `UMBRELLA_COLOR_BY_WEATHER`
pub fn umbrella_color_by_weather(key: &str) -> u32 { by_weather(key, 0x0a1218, 0x282010, 0x2a0a0a, 0x3a3020, 0x1a1a2a, 0x1a1a1a) }
/// `SOAP_BAR_COLOR_BY_WEATHER` (normal: null → R335 baseline 4-color pool)
pub fn soap_bar_color_by_weather(key: &str) -> Option<&'static [u32]> {
    by_weather(key,
        Some(&[0xa8d0d8, 0xf0e8d0][..]),
        Some(&[0xe8d0a0, 0xf0e8d0][..]),
        Some(&[0x8a3040, 0x603030][..]),
        Some(&[0x8a7050, 0xa08060][..]),
        Some(&[0xf0e8d0, 0xd8c0d0][..]),
        None)
}
/// `TOOTHBRUSH_COLOR_BY_WEATHER` (normal: null → R328 baseline 4-color pool)
pub fn toothbrush_color_by_weather(key: &str) -> Option<&'static [u32]> {
    by_weather(key,
        Some(&[0x40a0d8, 0xf0f0f0][..]),
        Some(&[0xd0d060, 0xf0f0f0][..]),
        Some(&[0xd06060, 0x8a3030][..]),
        Some(&[0x8a7040, 0x6a5030][..]),
        Some(&[0xf0f0f0, 0x40a0d8][..]),
        None)
}
/// `OIL_STAIN_BY_WEATHER`
pub fn oil_stain_by_weather(key: &str) -> u32 { by_weather(key, 0x0a0c10, 0x0a0805, 0x1a0605, 0x100a08, 0x0a0806, 0x0a0806) }
/// `GOGGLE_LENS_BY_WEATHER`
pub fn goggle_lens_by_weather(key: &str) -> u32 { by_weather(key, 0x6890a8, 0xd8c880, 0xa87070, 0x907050, 0x8ac0d8, 0x8ac0d8) }
/// `PHOTO_TINT_BY_WEATHER`
pub fn photo_tint_by_weather(key: &str) -> u32 { by_weather(key, 0x8ea0b4, 0xd0b070, 0xa06060, 0xa89870, 0xc4b898, 0xffffff) }
/// `POSTER_TINT_BY_WEATHER`
pub fn poster_tint_by_weather(key: &str) -> u32 { by_weather(key, 0xa0b0c0, 0xe0c890, 0xb08080, 0xc0b090, 0xd8ccb0, 0xffffff) }
/// `MIRROR_GLASS_BY_WEATHER`
pub fn mirror_glass_by_weather(key: &str) -> u32 { by_weather(key, 0x3a5060, 0x605020, 0x603030, 0x484038, 0x505860, 0x505860) }
/// `DEAD_PLANT_POT_BY_WEATHER` (still / normal: null → R379 baseline)
pub fn dead_plant_pot_by_weather(key: &str) -> Option<&'static [u32]> {
    by_weather(key,
        Some(&[0x3a4048, 0x2a3038, 0x4a5058][..]),
        Some(&[0x4a3020, 0x603828, 0x6a4028][..]),
        Some(&[0x3a1010, 0x2a0808, 0x4a1010][..]),
        Some(&[0x5a4830, 0x483828, 0x584030][..]),
        None,
        None)
}
/// `BOOK_COVER_BY_WEATHER` (still / normal: null → R380 baseline)
pub fn book_cover_by_weather(key: &str) -> Option<&'static [u32]> {
    by_weather(key,
        Some(&[0x1a3040, 0x203040, 0x2a3438][..]),
        Some(&[0x4a3820, 0x604828, 0x503820][..]),
        Some(&[0x6a2020, 0x4a1010, 0x300808][..]),
        Some(&[0x4a3820, 0x3a3018, 0x584030][..]),
        None,
        None)
}
/// `TRAFFIC_CONE_BY_WEATHER`
pub fn traffic_cone_by_weather(key: &str) -> u32 { by_weather(key, 0xa04010, 0xf07038, 0x903030, 0x8a4830, 0xd85018, 0xd85018) }
/// `RUG_TINT_BY_WEATHER`
pub fn rug_tint_by_weather(key: &str) -> u32 { by_weather(key, 0x707880, 0xd8a860, 0xa04040, 0xa89070, 0xd8c8b0, 0xffffff) }
/// `CEILING_DEBRIS_BY_WEATHER`
pub fn ceiling_debris_by_weather(key: &str) -> u32 { by_weather(key, 0x8890a0, 0xa89880, 0xa88880, 0x988878, 0xa89890, 0xa89890) }
/// `EXTENSION_CORD_BY_WEATHER` (still / normal: null → R343 baseline)
pub fn extension_cord_by_weather(key: &str) -> Option<&'static [u32]> {
    by_weather(key,
        Some(&[0x0a0f18, 0x101820][..]),
        Some(&[0x201808, 0x282010][..]),
        Some(&[0x1a0808, 0x201014][..]),
        Some(&[0x181410, 0x201818][..]),
        None,
        None)
}
/// `SMOKE_WISP_BY_WEATHER`
pub fn smoke_wisp_by_weather(key: &str) -> u32 { by_weather(key, 0x505860, 0x707068, 0x704040, 0x604838, 0x606058, 0x606058) }
/// `MIRROR_FRAME_BY_WEATHER`
pub fn mirror_frame_by_weather(key: &str) -> u32 { by_weather(key, 0x2a3038, 0x2a1a0a, 0x2a1010, 0x2a2418, 0x3a2a18, 0x3a2a18) }
/// `TRASH_BAG_COLOR_BY_WEATHER`
pub fn trash_bag_color_by_weather(key: &str) -> u32 { by_weather(key, 0x0a0f14, 0x281a0a, 0x1a0808, 0x2a2418, 0x0a0a10, 0x1a1a1e) }
/// `PLANT_GLOW_BY_WEATHER`
pub fn plant_glow_by_weather(key: &str) -> u32 { by_weather(key, 0x40a080, 0x80c040, 0x9040c0, 0xa08040, 0xd0e0d0, 0x60a040) }
// line 5898
/// `KEY_COLOR_BY_WEATHER`
pub fn key_color_by_weather(key: &str) -> u32 { by_weather(key, 0x8a7830, 0xc8b850, 0xa88a40, 0x6a5828, 0xd0d0c8, 0xb8a850) }
