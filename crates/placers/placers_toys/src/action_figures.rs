//! `place_action_figures`: Plastic action figure: 4% chance per level. A jointed 3 3/4 in (95 mm) soldier of the 1980s kind, no maker's: its O-ring body moulded in the uniform's plastic, the head and hands in flesh plastic, boots, belt and pouches painted black and rubbed through to the plastic, a painted steel-pot helmet; standing on its boots, or fallen on its back. Faded, scuffed, grimed.
use crate::shared::put;
use crate::shared::wear_tex;
use crate::shared::Wear;
use game::prelude::*;
use placers_kit::geometry::put_world;
use placers_kit::paint::rgb;
use placers_kit::place::index_of;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;
use placers_kit::place::salt_at;

/// Plastic action figure: 4% chance per level. A jointed 3 3/4 in
/// (95 mm) soldier of the 1980s kind, no maker's: its O-ring body moulded
/// in the uniform's plastic, the head and hands in flesh plastic, boots,
/// belt and pouches painted black and rubbed through to the plastic, a
/// painted steel-pot helmet; standing on its boots, or fallen on its back.
/// Faded, scuffed, grimed.
pub(crate) fn place_action_figures(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    // the old body colours stand for the uniform's plastic: a flight suit,
    // olive drab, a faded red, grey, desert tan
    let body_colors: [u32; 5] = [0x2060c0, 0x40a040, 0xc02020, 0x606060, 0xd0a040];
    let k = index_of(&body_colors, pick(&body_colors));
    let uniform = [0x3c4a60, 0x545a38, 0x70413a, 0x6a6a64, 0x98865e][k];
    let helmet = [0x4c5040, 0x464c30, 0x4c5040, 0x55574e, 0x8c7a56][k];
    // 60% fallen on its back, else standing (the old draw)
    let pose = if rnd() < 0.60 { "Lie" } else { "Stand" };
    let px = g.wx(x) + (rnd() - 0.5) * 2.4;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.4;
    let turn = rnd() * PI * 2.0;
    let salt = salt_at(px, pz);
    let flesh = rgb(0xc29a7c);
    let plastic = |base: [f64; 3], alt: [f64; 3], rub: f64, grime: f64, scuff: f64, seed: u32| {
        let w = Wear { base, alt, rub, grime, scuff, mottle: 0.05, scales: 0 };
        phong!({ map: &wear_tex(salt ^ seed, 128, w), specular: 0x161616, shininess: 18 })
    };
    let fig = group();
    let size = 0.03;
    let u = rgb(uniform);
    put_world(&fig, &format!("Real__actionFigureBody{pose}"), &plastic(u, u, 0.0, 0.45, 0.5, 1), size);
    put_world(&fig, &format!("Real__actionFigureSkin{pose}"), &plastic(flesh, flesh, 0.0, 0.5, 0.3, 2), size);
    put_world(&fig, &format!("Real__actionFigureKit{pose}"), &plastic(rgb(0x1c1b19), u, 0.08, 0.2, 0.5, 3), size);
    put_world(&fig, &format!("Real__actionFigureHelmet{pose}"), &plastic(rgb(helmet), flesh, 0.04, 0.4, 0.3, 4), size);
    put(&fig, &format!("Real__actionFigureEyes{pose}"), &lambert!({ color: 0x1e1a17 }));
    fig.position().set(px, 0, pz);
    fig.rotation().y = turn;
    g.world.add(&fig);
}
