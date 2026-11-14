//! `place_small_owls`
use game::prelude::*;
use placers_kit::geometry::put;
use placers_kit::noise::fbm;
use placers_kit::noise::mix;
use placers_kit::noise::sstep;
use placers_kit::noise::vn;
use placers_kit::paint::fill;
use placers_kit::place::open_tiles;
use placers_kit::place::pick_hex;
use placers_kit::place::pick_tile;
use placers_kit::place::salt;

// Small owl figurine: 5% chance per level. A carved wooden owl (15 cm),
// the souvenir kind carved from one block and stained dark: a horned owl
// standing on a rock carved with it, its feathers cut in rows of chisel
// scallops (the stain lying dark in the cuts), its glass eyes amber with
// the pupil painted behind; the crown and the tufts rubbed lighter by
// hands, dust on it.
pub(crate) fn place_small_owls(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick_tile(&open);
    // the old body colour: the stain
    let stains: [u32; 3] = [0x3a2818, 0x201410, 0x5a4030];
    let stain = pick_hex(&stains);
    let px = g.wx(x) + (rnd() - 0.5) * 2.4;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.4;
    let turn = rnd() * PI * 2.0;
    let owl = group();
    put(&owl, "Real__owlWood", &lambert!({ map: &owl_wood_tex(salt(px, pz, 2), stain) }));
    // glass eyes: they catch the light and never make it
    put(&owl, "Real__owlEyes", &phong!({ color: 0x6a4a14, specular: 0x7a7a7a, shininess: 90 }));
    put(&owl, "Real__owlPupils", &phong!({ color: 0x141210, specular: 0x6a6a6a, shininess: 90 }));
    owl.position().set(px, 0, pz);
    owl.rotation().y = turn;
    g.world.add(&owl);
}

/// The owl's wood, laid on it from the front and the sides (u across, v up
/// its 134 mm): the stain over lime's pale grain, darker in the chisel cuts
/// of the feathers' rows (long feathers low on the sides of the body, short
/// ones on the breast), the facial disc cut in fine rays round the eyes, the
/// rock below cut in flat facets; rubbed light where hands held it, dust
fn owl_wood_tex(salt: u32, stain: u32) -> Tex {
    let s = [f64::from((stain >> 16) & 255) / 255.0, f64::from((stain >> 8) & 255) / 255.0, f64::from(stain & 255) / 255.0];
    canvas_tex_opt(256, 256, false, move |ctx, w, h| {
        let seed = salt ^ 0x0a1;
        // the stain over the pale wood: a third of the wood's own colour shows
        let base = mix(s, [0.62, 0.50, 0.36], 0.30);
        fill(ctx, w, h, |u, v| {
            let mm = v * 134.0;
            // the grain, along the block (up)
            let grain = fbm(seed, u * 70.0, v * 5.0, 3);
            let mut c = mix(base, mix(base, [0.0, 0.0, 0.0], 0.45), sstep(0.45, 0.75, grain) * 0.5);
            c = mix(c, [0.70, 0.58, 0.42], sstep(0.62, 0.8, fbm(seed ^ 3, u * 9.0, v * 9.0, 3)) * 0.12);
            let cut = if mm < 21.0 {
                // the rock: flat facets, each its own shade
                let f = vn(seed ^ 5, u * 14.0, v * 30.0);
                c = mix(c, [0.0, 0.0, 0.0], (f - 0.5) * 0.4);
                0.0
            } else if mm > 92.0 && mm < 120.0 && (u - 0.5).abs() < 0.32 {
                // the facial disc: fine rays from each eye
                let ex = if u < 0.5 { 0.373 } else { 0.627 };
                let (dx, dy) = ((u - ex) * 98.0, mm - 104.5);
                let r = dx.hypot(dy);
                let a = dy.atan2(dx);
                let ray = ((a * 24.0).sin().abs()).powf(8.0) * sstep(8.5, 11.0, r) * (1.0 - sstep(14.0, 17.0, r));
                ray * 0.25
            } else {
                // the feathers: scallop rows, larger and longer toward the
                // sides and low on the body
                let low = 1.0 - sstep(40.0, 70.0, mm);
                let side = sstep(0.2, 0.42, (u - 0.5).abs());
                let k = 1.0 + 0.6 * side.max(low);
                let (cw, ch) = (8.0 * k / 98.0, 6.5 * k * (1.0 + 0.5 * side) / 134.0);
                let row = (v / ch).floor();
                let off = if (row as i64) % 2 == 0 { 0.0 } else { cw * 0.5 };
                let cu = ((u + off) / cw).fract() - 0.5;
                let cv = (v / ch).fract();
                let edge = 0.12 + 1.6 * cu * cu;
                let line = 1.0 - sstep(0.0, 0.09, (cv - edge).abs());
                // the cut's face, just above the line, catches the light
                let face = sstep(edge, edge + 0.08, cv) * (1.0 - sstep(edge + 0.1, edge + 0.4, cv));
                c = mix(c, [0.72, 0.60, 0.45], face * 0.10);
                line * 0.75 * (1.0 - sstep(118.0, 126.0, mm))
            };
            c = mix(c, mix(s, [0.0, 0.0, 0.0], 0.55), cut);
            // rubbed: the crown, the tufts, the rock's front edge
            let rub = sstep(118.0, 132.0, mm) * 0.35 + (1.0 - sstep(16.0, 21.0, mm)) * sstep(12.0, 16.0, mm) * 0.2;
            c = mix(c, [0.66, 0.53, 0.38], rub * (0.6 + 0.4 * vn(seed ^ 7, u * 20.0, v * 20.0)));
            // dust in its pores
            let dust = vn(seed ^ 9, u * 180.0, v * 180.0);
            if dust > 0.8 {
                c = mix(c, [0.46, 0.44, 0.40], (dust - 0.8) * 2.0);
            }
            [c[0], c[1], c[2], 1.0]
        });
    })
}
