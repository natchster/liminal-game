//! `place_small_nail_polishes`: Small nail polish bottle: 10% chance per level. Small square glass bottle with brush cap. Dressing-table family.
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

/// Small nail polish bottle: 10% chance per level. Small
/// square glass bottle with brush cap. Dressing-table family.
pub(crate) fn place_small_nail_polishes(g: &mut Game, _dist: &[i32]) {
    // A small bottle of nail polish (batch `r18`): heavy-bottomed square glass,
    // a black screw cap with its brush in the lacquer, the lacquer dried down
    // in it (pooled along its side when it fell over wet), a drip run down
    // the glass and dried, a crust round the neck; no maker's name
    // (r18.notes.md)
    use game::assets::props::prop_part_as_modelled;
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    // the old colour draw: red, plum, blue, gold, green, burgundy, near black
    let polish_colors = [0xd82040, 0x8a3060, 0x2050d8, 0xd8b040, 0x30a030, 0xa02040, 0x101820];
    let polish_color = pick(&polish_colors);
    let kind = polish_colors.iter().position(|&k| k == polish_color).unwrap_or(0);
    let lacquer = phong!({ color: LACQUER[kind], specular: 0x4a4a4a, shininess: 90 });
    let bottle = group();
    let put = |name: &str, mat: &Mat| {
        if let Some(geo) = prop_part_as_modelled(name) {
            bottle.add(&mesh(geo, mat));
        }
    };
    put("Real__nailPolishCap", &phong!({ color: 0x18171a, specular: 0x3a3a3a, shininess: 70 }));
    put("Real__nailPolishBrush", &lambert!({ color: 0xcfcab9 }));
    put("Real__nailPolishDrip", &lacquer);
    // 20% chance tipped on its side (it fell over while the lacquer was wet)
    let tipped = rnd() < 0.20;
    put(if tipped { "Real__nailPolishLacquerLying" } else { "Real__nailPolishLacquer" }, &lacquer);
    // the glass over what is in it; its thick bottom less clear (and solid
    // enough to stand on the floor: the lacquer is not what touches it)
    put("Real__nailPolishGlassBase", &phong!({ color: 0x7f8a82, specular: 0xc8c8c8, shininess: 110, transparent: true, opacity: 0.62 }));
    put("Real__nailPolishGlass", &phong!({ color: 0xd2d6d0, specular: 0xc8c8c8, shininess: 110, transparent: true, opacity: 0.42 }));
    if tipped {
        bottle.rotation().z = PI / 2.0;
        bottle.position().y = 0.0125;
    }
    bottle.position().x = g.wx(x) + (rnd() - 0.5) * 2.4;
    bottle.position().z = g.wz(z) + (rnd() - 0.5) * 2.4;
    bottle.rotation().y = rnd() * PI * 2.0;
    g.world.add(&bottle);
}

/// The lacquers, dried dark in the glass (sRGB hex): red, plum, blue,
/// gold, green, burgundy, near black
const LACQUER: [u32; 7] = [0x8a1c28, 0x5a2642, 0x23386a, 0x8a6a30, 0x2a5034, 0x561a26, 0x18191c];
