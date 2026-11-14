//! `place_small_skeleton_hands`
use game::prelude::*;
use placers_kit::geometry::put;
use placers_kit::noise::fbm;
use placers_kit::noise::mix;
use placers_kit::noise::sstep;
use placers_kit::noise::vn;
use placers_kit::paint::fill;
use placers_kit::place::open_tiles;
use placers_kit::place::pick_tile;
use placers_kit::place::salt;

// Small skeleton hand: 5% chance per level. A life-size moulded plastic
// skeleton hand (18 cm), the shop kind, lying palm down, its fingers a
// little curled; the plastic gone ivory, grimed in its joints, dusty.
pub(crate) fn place_small_skeleton_hands(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick_tile(&open);
    let px = g.wx(x) + (rnd() - 0.5) * 2.4;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.4;
    let turn = rnd() * PI * 2.0;
    let salt = salt(px, pz, 5);
    let hand = group();
    put(&hand, "Real__skelHand", &phong!({ map: &bone_plastic_tex(salt), specular: 0x262422, shininess: 14 }));
    hand.position().set(px, 0, pz);
    hand.rotation().y = turn;
    g.world.add(&hand);
}

/// Moulded polystyrene gone ivory (laid on it from above and its sides):
/// yellowed unevenly, grime in patches, dust
fn bone_plastic_tex(salt: u32) -> Tex {
    canvas_tex_opt(256, 256, false, move |ctx, w, h| {
        let seed = salt ^ 0xb0e;
        fill(ctx, w, h, |u, v| {
            let n = fbm(seed, u * 7.0, v * 7.0, 4);
            let mut c = mix([0.80, 0.76, 0.65], [0.72, 0.66, 0.52], n);
            let grime = sstep(0.55, 0.8, fbm(seed ^ 3, u * 14.0, v * 14.0, 3));
            c = mix(c, [0.46, 0.41, 0.32], grime * 0.55);
            let dust = vn(seed ^ 5, u * 180.0, v * 180.0);
            if dust > 0.8 {
                c = mix(c, [0.55, 0.52, 0.47], (dust - 0.8) * 2.5);
            }
            [c[0], c[1], c[2], 1.0]
        });
    })
}
