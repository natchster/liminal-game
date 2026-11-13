//! Batch `t15`'s spinning top, made real (`docs/design/PROPS.md` P3, "toys stay
//! toys"; `assets/src/blender/props/t15.py`, `t15.notes.md`). Its placer and
//! what only it uses; it keeps the old placer's draws, call for call.
use crate::{pick_color, pick_open_tile, scatter_xz};
use game::assets::props::prop_part_as_modelled;
use game::prelude::*;

/// Small spinning top: 5% chance per level. A turned beech twirl top
/// (51 mm across), painted on the lathe in three colours with thin black
/// lines between, the stem and tip bare and lacquered; it has stopped and
/// fallen over, lying on its shoulder and its tip (never standing on its
/// point). The paint chipped to the wood at its rim, faded, grimed.
pub fn place_small_spinning_tops(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let Some((x, z)) = pick_open_tile(g) else { return };
    // the old cap colour: the first of its three paints (the next two
    // follow it round the old rainbow)
    let rainbow = [0xd82030, 0xe07020, 0xd8b820, 0x30a030, 0x2080c8, 0x4030c0];
    let cap = pick_color(&rainbow);
    let ka = rainbow.iter().position(|&c| c == cap).unwrap_or(0);
    // the old 60% spinning and its rate: it has stopped now (the draws kept)
    if rnd() < 0.60 {
        let _ = rnd();
    }
    let top = group();
    scatter_xz(g, &top, x, z, 2.4);
    let p = top.pos();
    let salt = salt_at(p.x, p.z);
    // its heading from where it lies (the old top had none to draw)
    top.rotation().y = f64::from(salt % 3600) / 3600.0 * PI * 2.0;
    let paints = [0x8c3a2e, 0x9e6032, 0xa88c3e, 0x4e6c3e, 0x3c5c7c, 0x504672];
    let wood = rgb(0xa88a62);
    let chipped = |c: u32, seed: u32| {
        let base = rgb(c);
        let w = Wear { base, alt: wood, rub: 0.1, grime: 0.35, scuff: 0.35, mottle: 0.06, scales: 0 };
        phong!({ map: &wear_tex(salt ^ seed, 128, w), specular: 0x2a2a2a, shininess: 34 })
    };
    let bare = Wear { base: wood, alt: mix(wood, [0.3, 0.22, 0.15], 0.5), rub: 0.06, grime: 0.45, scuff: 0.3, mottle: 0.1, scales: 0 };
    let wood_tex = paint_tile(128, move |u, v| {
        // beech's fine grain and its flecks under the lacquer
        let gr = vnp(salt ^ 71, u * 4.0, v * 60.0, 4, 60);
        mul(wear_at(salt ^ 72, u, v, &bare), 0.92 + 0.12 * gr)
    });
    put_world(&top, "Real__spinTopWood", &phong!({ map: &wood_tex, specular: 0x2a2a2a, shininess: 30 }), 0.03);
    put_world(&top, "Real__spinTopPaintA", &chipped(paints[ka], 73), 0.03);
    put_world(&top, "Real__spinTopPaintB", &chipped(paints[(ka + 2) % 6], 74), 0.03);
    put_world(&top, "Real__spinTopPaintC", &chipped(paints[(ka + 4) % 6], 75), 0.03);
    put_world(&top, "Real__spinTopLines", &chipped(0x1c1a18, 76), 0.03);
    g.world.add(&top);
}
// ------------------------------------------------------------- the paint ---

/// Part `name` of batch `t15` as modelled, added to `grp` in `mat`
fn put(grp: &Obj, name: &str, mat: &Mat) {
    if let Some(geo) = prop_part_as_modelled(name) {
        grp.add(&mesh(geo, mat.clone()));
    }
}

use placers_kit::geometry::put_world;

use placers_kit::place::salt_at;

use placers_kit::paint::rgb;

use placers_kit::noise::sstep as ss;

use placers_kit::noise::mix;

use placers_kit::paint::mul;

use placers_kit::noise::hash;

use placers_kit::noise::vnp;

use placers_kit::noise::fbmp;

use placers_kit::noise::cell_edge;

/// How a moulded or painted thing has worn on a floor for years
#[derive(Clone, Copy)]
struct Wear {
    /// its colour, and what shows where it is rubbed through (the plastic
    /// under paint, or plastic whitened by stress)
    base: [f64; 3],
    alt: [f64; 3],
    /// how much of it is rubbed through, grimed, scuffed, mottled (faded
    /// unevenly); its sculpted scales (cells across a tile; 0 none)
    rub: f64,
    grime: f64,
    scuff: f64,
    mottle: f64,
    scales: i32,
}

/// The worn surface at (u, v) of a repeating tile
fn wear_at(seed: u32, u: f64, v: f64, w: &Wear) -> [f64; 3] {
    let m = fbmp(seed, u, v, 3, 4);
    let mut c = mul(w.base, 1.0 + w.mottle * (m - 0.5) * 2.4);
    if w.scales > 0 {
        let e = cell_edge(seed ^ 0x51ab, u, v, w.scales);
        c = mul(c, 1.0 - 0.2 * (1.0 - ss(0.0, 0.14, e)));
    }
    if w.rub > 0.0 {
        // rubbed through in small patches: `rub` is about the share of the
        // surface (this fractal noise sits near 0.49, its spread about 0.15)
        let r = fbmp(seed ^ 0x2b3c, u, v, 12, 3);
        let t = 0.8 - 0.9 * w.rub;
        c = mix(c, w.alt, ss(t, t + 0.025, r));
    }
    // grime settled in soft patches
    let gm = fbmp(seed ^ 0x77e1, u, v, 3, 4);
    c = mix(c, [0.15, 0.13, 0.10], w.grime * 0.7 * ss(0.5, 0.8, gm));
    if w.scuff > 0.0 {
        // fine scratches in patches: thin lines along a stretched noise
        let patch = ss(0.5, 0.7, fbmp(seed ^ 0x9137, u, v, 3, 2));
        let l1 = (vnp(seed ^ 0x4421, u * 40.0, v * 5.0, 40, 5) - 0.5).abs();
        let l2 = (vnp(seed ^ 0x1d77, u * 6.0, v * 34.0, 6, 34) - 0.5).abs();
        let line = (1.0 - ss(0.0, 0.025, l1)).max(1.0 - ss(0.0, 0.025, l2));
        c = mix(c, mix(w.base, [0.86, 0.84, 0.8], 0.45), w.scuff * patch * line * 0.8);
    }
    // dust in the grain, and grit
    let d = vnp(seed ^ 0xd057, u * 90.0, v * 90.0, 90, 90);
    if d > 0.8 {
        c = mix(c, [0.55, 0.52, 0.47], (d - 0.8) * 2.2);
    }
    c
}

use placers_kit::paint::paint_tile;

/// A worn surface's tile
fn wear_tex(seed: u32, px: u32, w: Wear) -> Tex {
    paint_tile(px, move |u, v| wear_at(seed, u, v, &w))
}
