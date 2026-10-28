//! `visuals::photo` — the Building's surfaces, made from photographs of
//! Soviet-era places: its panel facade, poured and formwork concrete, the yard concrete of
//! its decks, the granite of its Stair Halls (walls and floor), the factory
//! slab and ribbed ceiling of its Works, and its lit windows.
//!
//! The JPEGs in `crates/game/assets/textures/` are built by
//! `assets/src/textures/build.py` from the CC0 photographs it lists (their
//! sources and licence are in `crates/game/assets/MANIFEST.json`). Each is
//! seamless both ways and graded to the canvas surface it replaced. A
//! surface is decoded once, when a floor first asks for it, and takes no
//! random draw (the native stream is left as it is).
//!
//! grime: 0 clean · 1 aged · 2 dirty · 3 ruined: one texture for every
//! level, the material tinted darker (`grime_tint`), so a kind is decoded
//! and uploaded once. The Well's lit windows are photographed windows too
//! (`lit_window_tex`).

use engine::math::srgb_to_linear;
use engine::prelude::*;

macro_rules! texture {
    ($f:literal) => {
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/textures/", $f))
    };
}

/// The photographed surfaces: (kind, file, metres a repeat `[u, v]`).
pub static PHOTO_SURFACES: [(&str, &[u8], [f64; 2]); 8] = [
    // the Well's facade: twelve panels a tile wide and a storey high, each with its window
    ("panel", texture!("panel.jpg"), [16.0, 9.0]),
    // the Stair Halls: rock-faced granite in courses of 0.75 m
    ("ashlar", texture!("ashlar.jpg"), [8.0, 6.0]),
    // the Well's parapets and cells: whitewashed poured concrete
    ("poured", texture!("poured.jpg"), [4.0, 4.0]),
    // the Works' walls, housings and cells: concrete cast against steel forms
    ("formwork", texture!("formwork.jpg"), [4.0, 4.0]),
    // the Well's decks
    ("deck", texture!("deck.jpg"), [8.0, 8.0]),
    // the Stair Halls' floors: granite slabs a metre by half a metre
    ("stone", texture!("stone.jpg"), [4.0, 4.0]),
    // the Works' floors: a factory slab, saw-cut every 4 m
    ("factory", texture!("factory.jpg"), [8.0, 8.0]),
    // the Works' ceiling: ribbed precast slabs
    ("ribbed", texture!("ribbed.jpg"), [4.0, 1.6]),
];

/// The Well's lit windows: four photographed windows lit from inside, side by
/// side (grey light; the material tints it).
static LIT_WINDOWS: &[u8] = texture!("lit.jpg");
/// How many windows `LIT_WINDOWS` holds.
pub const LIT_CELLS: usize = 4;

/// How much of the photograph's light each grime level keeps (as a print:
/// a factor on its sRGB values).
const GRIME_TONE: [f64; 4] = [1.1, 1.0, 0.84, 0.7];

/// The material tint (linear) that darkens a photographed surface to grime
/// `g`: a print's factor on sRGB values is about this one on linear light
pub fn grime_tint(g: i32) -> f64 {
    srgb_to_linear(GRIME_TONE[g.clamp(0, 3) as usize])
}

/// The photographed surface `kind` (None if there is no such surface)
pub fn photo_tex(kind: &str) -> Option<Tex> {
    let (_, bytes, _) = PHOTO_SURFACES.iter().find(|s| s.0 == kind)?;
    let canvas = Canvas::from_jpeg(bytes).unwrap_or_else(|| panic!("assets/textures/{kind}.jpg does not decode"));
    let t = Tex::from_image(canvas.to_image());
    {
        let mut tt = t.t();
        tt.color_space = ColorSpace::Srgb;
        tt.wrap_s = Wrap::Repeat;
        tt.wrap_t = Wrap::Repeat;
        // floors and facades are mostly seen at a slant
        tt.anisotropy = 8;
    }
    Some(t)
}

/// Metres a repeat of the photographed surface `kind`
pub fn photo_scale(kind: &str) -> Option<[f64; 2]> {
    PHOTO_SURFACES.iter().find(|s| s.0 == kind).map(|s| s.2)
}

/// The lit window `cell` (0..LIT_CELLS) of `LIT_WINDOWS`, for a quad over a
/// panel's glass; every cell shares one decoded image (and one upload)
pub fn lit_window_tex(cell: usize) -> Tex {
    thread_local! { static LIT: std::cell::RefCell<Option<Tex>> = const { std::cell::RefCell::new(None) }; }
    let base = LIT.with(|l| {
        l.borrow_mut()
            .get_or_insert_with(|| {
                let t = Tex::from_image(Canvas::from_jpeg(LIT_WINDOWS).expect("assets/textures/lit.jpg does not decode").to_image());
                t.t().color_space = ColorSpace::Srgb;
                t
            })
            .clone()
    });
    let t = base.duplicate();
    t.set_repeat(1.0 / LIT_CELLS as f64, 1.0);
    t.set_offset(cell.min(LIT_CELLS - 1) as f64 / LIT_CELLS as f64, 0.0);
    t
}
