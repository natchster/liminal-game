//! `place_small_toy_microphones`: Small toy microphone: 8% chance per level. A child's toy microphone, 14 cm: a round grille moulded as a mesh in grey plastic and painted silver, a dark band round it, a coloured handle with grip rings and a slide switch; lying on its grille and its butt. The silver rubbed through to the grey, the handle scuffed and grimed.
use crate::shared::put;
use crate::shared::wear_at;
use crate::shared::wear_tex;
use crate::shared::Wear;
use game::assets::props::prop_part_as_modelled;
use game::prelude::*;
use placers_kit::geometry::put_world;
use placers_kit::noise::hash;
use placers_kit::noise::mix;
use placers_kit::noise::sstep;
use placers_kit::paint::paint_tile;
use placers_kit::paint::rgb;
use placers_kit::place::index_of;
use placers_kit::place::pick_color;
use placers_kit::place::pick_open;
use placers_kit::place::salt_at;
use placers_kit::place::scatter_pos;

/// Small toy microphone: 8% chance per level. A child's toy
/// microphone, 14 cm: a round grille moulded as a mesh in grey plastic and
/// painted silver, a dark band round it, a coloured handle with grip rings
/// and a slide switch; lying on its grille and its butt. The silver rubbed
/// through to the grey, the handle scuffed and grimed.
pub(crate) fn place_small_toy_microphones(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let Some((x, z)) = pick_open(g) else { return };
    // the old handle colours, as moulded plastic faded
    let handle_colors = [0xd82040, 0x2050d8, 0xd8b040, 0x30a030, 0xa020a0, 0x181008];
    let k = index_of(&handle_colors, pick_color(&handle_colors));
    let plastic = rgb([0x8a3636, 0x3a527a, 0xa88c48, 0x4e6c44, 0x6a406a, 0x2e2a26][k]);
    let mic = group();
    scatter_pos(g, &mic, x, z, 2.4);
    mic.rotation().y = rnd() * PI * 2.0;
    let p = mic.pos();
    let salt = salt_at(p.x, p.z);
    let grille_tex = paint_tile(128, move |u, v| {
        let w = Wear { base: rgb(0x9c9c98), alt: rgb(0x46484a), rub: 0.15, grime: 0.5, scuff: 0.2, mottle: 0.08, scales: 0 };
        let c = wear_at(salt ^ 51, u, v, &w);
        // the mesh's holes, about 2 mm apart, each a little off its place
        // (so the sides the texture is laid from meet without a seam)
        let (gx, gy) = (u * 10.0, v * 10.0);
        let (ci, cj) = (gx.floor() as i32, gy.floor() as i32);
        let jx = (hash(salt ^ 55, ci.rem_euclid(10), cj.rem_euclid(10)) - 0.5) * 0.3;
        let jy = (hash(salt ^ 57, ci.rem_euclid(10), cj.rem_euclid(10)) - 0.5) * 0.3;
        let (fx, fy) = (gx.fract() - 0.5 - jx, gy.fract() - 0.5 - jy);
        let hole = 1.0 - sstep(0.24, 0.32, (fx * fx + fy * fy).sqrt());
        mix(c, [0.05, 0.05, 0.05], hole * 0.92)
    });
    put_world(&mic, "Real__toyMicGrille", &phong!({ map: &grille_tex, specular: 0x4a4a4a, shininess: 40 }), 0.02);
    put(&mic, "Real__toyMicBand", &phong!({ color: 0x2a2826, specular: 0x202020, shininess: 30 }));
    let w = Wear { base: plastic, alt: mix(plastic, [0.82, 0.8, 0.76], 0.35), rub: 0.0, grime: 0.5, scuff: 0.5, mottle: 0.06, scales: 0 };
    put_world(&mic, "Real__toyMicHandle", &phong!({ map: &wear_tex(salt ^ 52, 128, w), specular: 0x222222, shininess: 26 }), 0.04);
    put(&mic, "Real__toyMicKnob", &phong!({ color: 0x24221f, specular: 0x202020, shininess: 20 }));
    g.world.add(&mic);
}
