//! `place_small_toy_dinos`: Small toy dinosaur: 6% chance per level. A solid PVC tyrannosaurus of the 1980s kind, 18 cm: its body level, the tail's tip on the floor, the mouth open on painted teeth, the eyes dots of paint; moulded in one colour, its skin sculpted in scales. Faded, scuffed, grime in its scales.
use crate::shared::Wear;
use crate::shared::put;
use crate::shared::wear_tex;
use game::prelude::*;
use placers_kit::geometry::put_world;
use placers_kit::noise::mix;
use placers_kit::paint::rgb;
use placers_kit::place::index_of;
use placers_kit::place::pick_color;
use placers_kit::place::pick_open;
use placers_kit::place::salt_at;
use placers_kit::place::scatter_pos;

/// Small toy dinosaur: 6% chance per level. A solid PVC tyrannosaurus
/// of the 1980s kind, 18 cm: its body level, the tail's tip on the floor,
/// the mouth open on painted teeth, the eyes dots of paint; moulded in one
/// colour, its skin sculpted in scales. Faded, scuffed, grime in its scales.
pub(crate) fn place_small_toy_dinos(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let Some((x, z)) = pick_open(g) else { return };
    // the old colours, as PVC faded
    let dino_colors = [0x408040, 0x306030, 0x504020, 0x805040, 0x2050d8];
    let k = index_of(&dino_colors, pick_color(&dino_colors));
    let pvc = rgb([0x5c6c42, 0x46583a, 0x5e4e38, 0x7e5e46, 0x445470][k]);
    let dino = group();
    scatter_pos(g, &dino, x, z, 2.4);
    dino.rotation().y = rnd() * PI * 2.0;
    let p = dino.pos();
    let salt = salt_at(p.x, p.z);
    let w = Wear { base: pvc, alt: mix(pvc, [0.85, 0.82, 0.75], 0.3), rub: 0.0, grime: 0.45, scuff: 0.4, mottle: 0.07, scales: 22 };
    put_world(&dino, "Real__smallTrexBody", &phong!({ map: &wear_tex(salt ^ 61, 256, w), specular: 0x141414, shininess: 14 }), 0.05);
    put(&dino, "Real__smallTrexTeeth", &lambert!({ color: 0xbdb39a }));
    put(&dino, "Real__smallTrexEyes", &lambert!({ color: 0x1c1915 }));
    g.world.add(&dino);
}
