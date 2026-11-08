//! Constants and per-weather tables used by this crate's placers (the ones not
//! in `game::placers::shared`).
#![allow(unused_variables, unused_mut, unused_assignments, unused_imports, dead_code, clippy::all)]
use game::prelude::*;

/// Book cover pool per weather; other keys get `[0x6a2020, 0x1a3040, 0x203020, 0x4a3820, 0x1a1a2a]`
/// (still / normal have no entry, so they take the baseline pool).
pub fn book_cover_pool(key: &str) -> &'static [u32] {
    match key {
        "wet" => &[0x1a3040, 0x203040, 0x2a3438],      // cool blue-gray covers
        "electric" => &[0x4a3820, 0x604828, 0x503820], // warm brown-gold
        "haunted" => &[0x6a2020, 0x4a1010, 0x300808],  // dark red only
        "derelict" => &[0x4a3820, 0x3a3018, 0x584030], // aged warm brown
        _ => &[0x6a2020, 0x1a3040, 0x203020, 0x4a3820, 0x1a1a2a],
    }
}

/// Dead plant pot pool per weather; other keys get `[0x6a4028, 0x483028, 0x503820]`
pub fn dead_plant_pot_pool(key: &str) -> &'static [u32] {
    match key {
        "wet" => &[0x3a4048, 0x2a3038, 0x4a5058],      // damp gray-blue clay
        "electric" => &[0x4a3020, 0x603828, 0x6a4028], // scorched terracotta
        "haunted" => &[0x3a1010, 0x2a0808, 0x4a1010],  // bloody dark red clay
        "derelict" => &[0x5a4830, 0x483828, 0x584030], // aged tan clay
        _ => &[0x6a4028, 0x483028, 0x503820],
    }
}

/// Traffic cone per weather (else the normal entry)
pub fn traffic_cone_by_weather(key: &str) -> u32 {
    by_weather(key, 0xa04010, 0xf07038, 0x903030, 0x8a4830, 0xd85018, 0xd85018)
}
/// Rug tint per weather (else the normal entry)
pub fn rug_tint_by_weather(key: &str) -> u32 {
    by_weather(key, 0x707880, 0xd8a860, 0xa04040, 0xa89070, 0xd8c8b0, 0xffffff)
}
/// Ceiling debris per weather (else the normal entry)
pub fn ceiling_debris_by_weather(key: &str) -> u32 {
    by_weather(key, 0x8890a0, 0xa89880, 0xa88880, 0x988878, 0xa89890, 0xa89890)
}
/// The water-puddle spec `(color, emissive)` per weather (else normal).
pub fn puddle_spec_by_weather(key: &str) -> (u32, u32) {
    by_weather(
        key,
        (0x1a2830, 0x08101a),
        (0x201808, 0x100804),
        (0x281010, 0x100404),
        (0x181410, 0x080604),
        (0x101820, 0x040810),
        (0x101820, 0x040810),
    )
}
/// The wall-streak colour per weather (else normal).
pub fn streak_color_by_weather(key: &str) -> u32 {
    by_weather(key, 0x1a2028, 0x1a1408, 0x180a08, 0x1a1610, 0x181410, 0x181410)
}

// Floor rug remnant canvas — subtle 3-color grid pattern.
// Built once, reused across all instances.
game::lazy_static_tex!(
    rug_texture,
    canvas_tex_opt(96, 64, false, |ctx, w, h| {
        // Base warm rug tone.
        ctx.set_fill_style("#4a2a20");
        ctx.fill_rect(0, 0, w, h);
        // Slightly lighter horizontal bands (subtle stripes).
        ctx.set_fill_style("#5a3428");
        let mut y = 8.0;
        while y < h {
            ctx.fill_rect(0, y, w, 4);
            y += 16.0;
        }
        // Darker edge border.
        ctx.set_fill_style("#302018");
        ctx.fill_rect(0, 0, w, 3);
        ctx.fill_rect(0, h - 3.0, w, 3);
        ctx.fill_rect(0, 0, 3, h);
        ctx.fill_rect(w - 3.0, 0, 3, h);
        // Wear speckle.
        for _ in 0..40 {
            ctx.set_fill_style(&format!("rgba(0,0,0,{})", 0.05 + rnd() * 0.15));
            let (x, y) = (rnd() * w, rnd() * h);
            ctx.fill_rect(x, y, 1, 1);
        }
    })
);

// Folded newspaper on floor: small rectangular plane with dark
// column-lines suggesting print.
game::lazy_static_tex!(
    newspaper_texture_2,
    canvas_tex_opt(96, 64, false, |ctx, w, h| {
        // Cream paper bg.
        ctx.set_fill_style("#c8bc9e");
        ctx.fill_rect(0, 0, w, h);
        // Dark headline bar at top.
        ctx.set_fill_style("#2a2015");
        ctx.fill_rect(4, 4, w - 8.0, 8);
        // 3 columns of tiny "text" lines.
        ctx.set_fill_style("#3a2c1a");
        let col_w = (w - 12.0) / 3.0;
        for col in 0..3 {
            let cx = 4.0 + col as f64 * (col_w + 2.0);
            let mut ly = 18.0;
            while ly < h - 4.0 {
                if rnd() < 0.75 {
                    let lw = col_w * (0.35 + rnd() * 0.65);
                    ctx.fill_rect(cx, ly, lw, 1);
                }
                ly += 3.0;
            }
        }
        // Grunge speckle.
        for _ in 0..20 {
            ctx.set_fill_style(&format!("rgba(0,0,0,{})", 0.06 + rnd() * 0.1));
            let (x, y) = (rnd() * w, rnd() * h);
            ctx.fill_rect(x, y, 1, 1);
        }
    })
);
