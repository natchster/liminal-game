//! `place_reliquary_boxes`
use game::prelude::*;
use game::visuals::pictures;
use placers_kit::geometry::put;
use placers_kit::noise::fbm;
use placers_kit::noise::mix;
use placers_kit::noise::sstep;
use placers_kit::noise::vn;
use placers_kit::paint::fill;
use placers_kit::place::open_tiles;
use placers_kit::place::pick_hex as pick_u32;
use placers_kit::place::pick_tile;
use placers_kit::place::salt;

// Reliquary box: 4% chance per level. A small reliquary casket of
// gilt brass of the later 19th century, 14 x 9 cm on its four bun feet:
// a stepped plinth, a glazed body between turned columns, a cornice, a
// hipped lid with its cresting and a cross; paste stones in collets on the
// lid (the old gems' colours, as old glass is: no glow). Inside, on a
// faded velvet cushion, a fragment of bone, its paper label and the red
// wax seal of its authentication. The gilding rubbed to the brass on what
// a hand touches, tarnish in the rest, dust over all of it.
pub(crate) fn place_reliquary_boxes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick_tile(&open);
    let gem_colors: [u32; 4] = [0xd82040, 0x2080d8, 0x30c060, 0xa020d0];
    // the four corner gems' picks, then the medallion's: the stones' pastes
    let corners: Vec<u32> = (0..4).map(|_| pick_u32(&gem_colors)).collect();
    let middle = pick_u32(&gem_colors);
    // the old glow's pulse phase (no light: paste does not shine)
    let _ = rnd();
    let px = g.wx(x) + (rnd() - 0.5) * 2.4;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.4;
    let turn = rnd() * PI * 2.0;
    let k = salt(px, pz, 19);
    let relic = group();
    put(&relic, "Real__reliquaryBrass", &phong!({ map: &gilt_tex(k), specular: 0x7a6030, shininess: 44 }));
    put(&relic, "Real__reliquaryCushion", &lambert!({ map: &velvet_tex(k) }));
    put(&relic, "Real__reliquaryRelic", &lambert!({ map: &relic_bone_tex(k) }));
    put(&relic, "Real__reliquaryLabel", &lambert!({ map: &label_tex(k), side: DoubleSide }));
    put(&relic, "Real__reliquarySeal", &phong!({ color: 0x5a1612, specular: 0x2a1a18, shininess: 24 }));
    let stones = [(-STONE_X, -1.0), (-STONE_X, 1.0), (STONE_X, -1.0), (STONE_X, 1.0)];
    for (i, &(sx, side)) in stones.iter().enumerate() {
        stone(&relic, "Real__reliquaryStone", paste(corners[i]), sx, side);
    }
    stone(&relic, "Real__reliquaryStoneBig", paste(middle), 0.0, 1.0);
    if let Some(m) = put(&relic, "Real__reliquaryGlass", &pictures::glass_mat(k, 0.35 + 0.25 * (k % 89) as f32 / 89.0)) {
        m.set_render_order(2);
    }
    relic.position().set(px, 0, pz);
    relic.rotation().y = turn;
    g.world.add(&relic);
}

/// Where the paste stones sit on the lid's slopes, as `t19.py` sets them
/// (`stone_place`): (x, y, z) of each and the slope's turn about x
const STONE_Y: f64 = 0.080;

const STONE_Z: f64 = 0.0255;

const STONE_TILT: f64 = 0.555_494;

const STONE_X: f64 = 0.034;

/// A paste stone for the old gem colour: foil-backed coloured glass, dark
/// in its body, its colour in its highlight
fn paste(old: u32) -> Mat {
    let (c, s) = match old {
        0xd82040 => (0x4c1418, 0x7a3a3a),
        0x2080d8 => (0x18223e, 0x3a4a6a),
        0x30c060 => (0x183a24, 0x3a5a44),
        _ => (0x2e1a3e, 0x52406a),
    };
    phong!({ color: c, specular: s, shininess: 90 })
}

/// A stone stood in its collet on the lid's slope (front `side` 1, back -1)
fn stone(grp: &Obj, name: &str, mat: Mat, x: f64, side: f64) {
    if let Some(m) = put(grp, name, &mat) {
        m.position().set(x, STONE_Y + 0.0011 * STONE_TILT.cos(), side * (STONE_Z + 0.0011 * STONE_TILT.sin()));
        m.rotation().x = side * STONE_TILT;
    }
}

/// Gilt brass gone dull: the gold rubbed through to the brass in patches,
/// tarnish brown-black where it was never rubbed, dust
fn gilt_tex(salt: u32) -> Tex {
    canvas_tex_opt(160, 160, false, move |ctx, w, h| {
        let seed = salt ^ 0x61;
        fill(ctx, w, h, |u, v| {
            // the gilding: a dark gold in the diffuse (its colour is in the highlight)
            let mut c = [0.28, 0.21, 0.10];
            // rubbed through to the paler, greener brass in patches
            let n = fbm(seed, u * 9.0, v * 9.0, 3);
            c = mix(c, [0.32, 0.28, 0.15], sstep(0.58, 0.72, n) * 0.6);
            // tarnish in specks and blotches
            let t = fbm(seed ^ 5, u * 22.0, v * 22.0, 2);
            c = mix(c, [0.14, 0.10, 0.06], sstep(0.55, 0.85, t) * 0.55);
            // the chased diaper: a lattice of lozenges, a quatrefoil in
            // each, the tarnish dark in the grooves
            let (a, b) = ((u + v) * 6.0, (u - v) * 6.0);
            let (fa, fb) = (a - a.floor() - 0.5, b - b.floor() - 0.5);
            let lattice = (0.5 - fa.abs()).min(0.5 - fb.abs());
            let rr = (fa * fa + fb * fb).sqrt();
            let foil = ((fa.atan2(fb) * 2.0).cos() * 0.05 + 0.20 - rr).abs();
            let groove = (1.0 - sstep(0.02, 0.07, lattice)) + (1.0 - sstep(0.015, 0.045, foil));
            c = mix(c, [0.09, 0.07, 0.04], groove.min(1.0) * 0.7);
            let d = vn(seed ^ 9, u * 60.0, v * 60.0);
            c = mix(c, [0.44, 0.42, 0.38], sstep(0.75, 0.95, d) * 0.3 + 0.05);
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// Silk velvet once crimson, faded toward rose where the light came
/// through the glass, its pile crushed paler where the relic lay, dust
fn velvet_tex(salt: u32) -> Tex {
    canvas_tex_opt(64, 64, false, move |ctx, w, h| {
        let seed = salt ^ 0x77;
        fill(ctx, w, h, |u, v| {
            let mut c = [0.36, 0.11, 0.11];
            let f = fbm(seed, u * 4.0, v * 4.0, 3);
            c = mix(c, [0.46, 0.25, 0.23], sstep(0.4, 0.8, f) * 0.8);
            let pile = vn(seed ^ 3, u * 50.0, v * 9.0);
            c = mix(c, [0.52, 0.32, 0.29], sstep(0.6, 0.9, pile) * 0.35);
            let d = fbm(seed ^ 7, u * 20.0, v * 20.0, 2);
            c = mix(c, [0.48, 0.44, 0.40], sstep(0.5, 0.8, d) * 0.3 + 0.05);
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// Old bone: ivory gone brown, darker in its pores and at its breaks
fn relic_bone_tex(salt: u32) -> Tex {
    canvas_tex_opt(32, 32, false, move |ctx, w, h| {
        let seed = salt ^ 0x91;
        fill(ctx, w, h, |u, v| {
            let mut c = [0.62, 0.55, 0.42];
            c = mix(c, [0.42, 0.34, 0.24], sstep(0.4, 0.8, fbm(seed, u * 5.0, v * 5.0, 3)) * 0.8);
            c = mix(c, [0.28, 0.22, 0.15], sstep(0.75, 0.9, vn(seed ^ 3, u * 30.0, v * 30.0)) * 0.6);
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// The relic's label: a strip of rag paper, its name written in an old
/// hand in iron-gall ink gone brown
fn label_tex(salt: u32) -> Tex {
    canvas_tex_opt(120, 34, false, move |ctx, w, h| {
        let seed = salt ^ 0xa3;
        fill(ctx, w, h, |u, v| {
            let mut c = [0.80, 0.74, 0.60];
            c = mix(c, [0.62, 0.52, 0.36], sstep(0.45, 0.85, fbm(seed, u * 6.0, v * 2.0, 3)) * 0.6);
            let edge = u.min(1.0 - u) * 3.5;
            c = mix(c, [0.55, 0.45, 0.32], (1.0 - sstep(0.0, 0.08, edge)) * 0.5);
            [c[0], c[1], c[2], 1.0]
        });
        ctx.set_fill_style("rgba(74,50,30,0.85)");
        ctx.set_font("italic 15px serif");
        ctx.set_text_align("center");
        let names = ["Ex oss. S. Fortunati M.", "Ex oss. S. Vitalis M.", "Ex oss. S. Clementis M."];
        ctx.fill_text_max(names[(salt % 3) as usize], w / 2.0, h * 0.68, w - 10.0);
    })
}
