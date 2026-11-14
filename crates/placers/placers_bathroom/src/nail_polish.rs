//! `place_nail_polish`
use game::assets::props::prop_part_as_modelled;
use game::prelude::*;
use placers_kit::geometry::put;
use placers_kit::place::jitter;
use placers_kit::place::pick_open;

// Nail polish bottle: 8% chance per level. A 13.5 ml bottle of nail
// polish (batch `r18`'s parts): heavy-bottomed square glass, a white screw cap
// with its brush, the lacquer dried down in it (pooled along its side when it
// fell over wet), a dried drip down the glass; no maker's name. The old
// colour draw picks the lacquer, dried dark.
pub(crate) fn place_nail_polish(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let polish_colors = [0xc02040, 0xd050a0, 0x804060, 0x40a0c0, 0x808080, 0x202020, 0xe0d040];
    let polish_color = choice(&polish_colors);
    let kind = polish_colors.iter().position(|&k| k == polish_color).unwrap_or(0);
    let lacquer = if kind == 4 {
        // a silver: the shimmer catches the light
        phong!({ color: LACQUER[kind], specular: 0x8a8a8a, shininess: 60 })
    } else {
        phong!({ color: LACQUER[kind], specular: 0x4a4a4a, shininess: 90 })
    };
    let polish = group();
    put(&polish, "Real__nailPolishCap", &phong!({ color: 0xd6d3cb, specular: 0x3a3a3a, shininess: 60 }));
    put(&polish, "Real__nailPolishBrush", &lambert!({ color: 0xcfcab9 }));
    put(&polish, "Real__nailPolishDrip", &lacquer);
    let (px, pz) = jitter(g, x, z, 2.4);
    polish.position().set(px, 0, pz);
    polish.rotation().y = rnd() * PI * 2.0;
    // 25% chance tipped over (it fell while the lacquer was wet)
    let tipped = rnd() < 0.25;
    put(&polish, if tipped { "Real__nailPolishLacquerLying" } else { "Real__nailPolishLacquer" }, &lacquer);
    // the glass over what is in it; its thick bottom less clear, the solid
    // the floor holds up
    put(
        &polish,
        "Real__nailPolishGlassBase",
        &phong!({ color: 0x7f8a82, specular: 0xc8c8c8, shininess: 110, transparent: true, opacity: 0.62 }),
    );
    put(
        &polish,
        "Real__nailPolishGlass",
        &phong!({ color: 0xd2d6d0, specular: 0xc8c8c8, shininess: 110, transparent: true, opacity: 0.42 }),
    );
    if tipped {
        polish.rotation().z = PI / 2.0;
        polish.position().y = 0.0125;
    }
    g.world.add(&polish);
}

/// The lacquers dried dark in the glass (sRGB): red, pink, plum, teal, silver,
/// black, yellow
const LACQUER: [u32; 7] = [0x8a1c28, 0x8e2a56, 0x5a2642, 0x1f5660, 0x6c6c70, 0x18191c, 0x8e7e26];
