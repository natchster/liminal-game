//! `place_toy_dinosaurs`: Toy dinosaur: 4% chance per level. A solid PVC dinosaur of the 1980s kind, sold loose from a tub: a stegosaurus (13 cm, its plates and tail spikes moulded with it) or a brontosaurus (16 cm, its neck raised); moulded in one colour, its skin sculpted in scales, the eyes dots of paint. Faded, scuffed, grime in its scales.
use crate::shared::Wear;
use crate::shared::put;
use crate::shared::wear_tex;
use game::prelude::*;
use placers_kit::geometry::put_world;
use placers_kit::noise::mix;
use placers_kit::paint::rgb;
use placers_kit::place::index_of;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;
use placers_kit::place::salt_at;

/// Toy dinosaur: 4% chance per level. A solid PVC dinosaur of the
/// 1980s kind, sold loose from a tub: a stegosaurus (13 cm, its plates and
/// tail spikes moulded with it) or a brontosaurus (16 cm, its neck raised);
/// moulded in one colour, its skin sculpted in scales, the eyes dots of
/// paint. Faded, scuffed, grime in its scales.
pub(crate) fn place_toy_dinosaurs(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    // the old colours, as PVC faded: green, blue, rust, mustard, purple
    let dino_colors: [u32; 5] = [0x408040, 0x2060c0, 0xa04020, 0xc0a040, 0x804080];
    let k = index_of(&dino_colors, pick(&dino_colors));
    let pvc = rgb([0x5b6a44, 0x46566e, 0x7c4c36, 0x9a8650, 0x5e4a62][k]);
    // the old back plates (50%): a stegosaurus; else a brontosaurus
    let stego = rnd() < 0.5;
    let px = g.wx(x) + (rnd() - 0.5) * 2.4;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.4;
    let turn = rnd() * PI * 2.0;
    let salt = salt_at(px, pz);
    let w = Wear { base: pvc, alt: mix(pvc, [0.85, 0.82, 0.75], 0.3), rub: 0.0, grime: 0.45, scuff: 0.4, mottle: 0.07, scales: 22 };
    let skin = phong!({ map: &wear_tex(salt ^ 21, 256, w), specular: 0x141414, shininess: 14 });
    let eyes = lambert!({ color: 0x1c1915 });
    let dino = group();
    if stego {
        put_world(&dino, "Real__toyStegoBody", &skin, 0.05);
        put_world(&dino, "Real__toyStegoPlates", &skin, 0.05);
        put(&dino, "Real__toyStegoEyes", &eyes);
    } else {
        put_world(&dino, "Real__toyBrontoBody", &skin, 0.05);
        put(&dino, "Real__toyBrontoEyes", &eyes);
    }
    dino.position().set(px, 0, pz);
    dino.rotation().y = turn;
    g.world.add(&dino);
}
