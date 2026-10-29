//! `visuals::photo` — the Building's surfaces, made from photographs of
//! Soviet-era places: its panel facade, poured and formwork concrete, the yard concrete of
//! its decks, the granite of its Stair Halls (walls and floor), the factory
//! slab and ribbed ceiling of its Works, and its lit windows; and the
//! photographed details laid over the rooms' painted surfaces (`arch_detail`:
//! woodchip paper, carpet, an acoustic tile's face).
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

/// The details laid over the rooms' painted surfaces (rooms R5): grey height
/// maps, mean ½, each a repeat of `metres` (`engine::ArchDetail`): (name,
/// file, metres a repeat, how much it shades the colour, how much relief)
static DETAILS: [(&str, &[u8], f64, f64, f64); 6] = [
    // woodchip wallpaper: wood chips pressed into the paper (a coarse
    // paper: at a fine one's 3 mm the chips were lost past an arm's length)
    ("paper", texture!("paper.jpg"), 1.2, 0.3, 0.6),
    // carpet: its loops and pile, as photographed (1.7 m a repeat)
    ("carpet", texture!("carpet.jpg"), 1.7, 0.3, 0.5),
    // an acoustic ceiling tile's face, a cell of the grid (`make_ceiling`)
    ("acoustic", texture!("acoustic.jpg"), 4.0 / 6.0, 0.35, 0.5),
    // plaster laid by hand and painted (drywall, painted walls, plaster ceilings)
    ("plaster", texture!("plaster.jpg"), 3.2, 0.22, 0.5),
    // concrete's pores and pits (concrete walls, slab ceilings)
    ("concrete", texture!("concrete.jpg"), 1.23, 0.28, 0.55),
    // a worn concrete floor's skin, pits and scuffs
    ("concrete_floor", texture!("concrete_floor.jpg"), 3.0, 0.28, 0.5),
];

/// The detail over a painted surface of the rooms, by its kind (`surfaces`):
/// a wall's, a floor's or a ceiling's (brick and glazed tile have their
/// pattern painted, and none)
pub fn detail_for(part: &str, kind: &str) -> Option<&'static str> {
    match (part, kind) {
        ("wall", "drywall" | "paint") | ("ceil", "plaster") => Some("plaster"),
        ("wall", "concrete") | ("ceil", "slab") => Some("concrete"),
        ("floor", "concrete") => Some("concrete_floor"),
        ("floor", "office") => Some("carpet"),
        _ => None,
    }
}

/// The detail `name` (`DETAILS`) for a surface: decoded once (as data, no
/// colour space: it is a height), repeating
pub fn arch_detail(name: &str) -> Option<ArchDetail> {
    thread_local! {
        static CACHE: std::cell::RefCell<Vec<(&'static str, Tex)>> = const { std::cell::RefCell::new(Vec::new()) };
    }
    let &(key, bytes, metres, tint, relief) = DETAILS.iter().find(|d| d.0 == name)?;
    let map = CACHE.with(|c| {
        let mut c = c.borrow_mut();
        if let Some((_, t)) = c.iter().find(|(k, _)| *k == key) {
            return t.clone();
        }
        let canvas = Canvas::from_jpeg(bytes).unwrap_or_else(|| panic!("assets/textures/{key}.jpg does not decode"));
        let t = Tex::from_image(canvas.to_image());
        {
            let mut tt = t.t();
            tt.wrap_s = Wrap::Repeat;
            tt.wrap_t = Wrap::Repeat;
            tt.anisotropy = 8;
        }
        c.push((key, t.clone()));
        t
    });
    Some(ArchDetail { map, metres, tint, relief })
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
