//! `place_small_brooms`
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

// Small broom: 8% chance per level. A decorative witch's besom
// (90 cm): a crooked hazel stick, its bark on, birch twigs bound round its
// end with two wraps of jute twine, lying on the floor; the twine dusty,
// the twigs dry and brittle, some broken.
pub(crate) fn place_small_brooms(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick_tile(&open);
    // the old fourteen twigs' draws: each its length and its lift; their
    // lengths' mean sizes the besom a little (+-4%)
    let mut len = 0.0;
    for _ in 0..14 {
        len += 0.11 + rnd() * 0.03;
        let _ = rnd();
    }
    let k = (len / 14.0) / 0.125;
    let px = g.wx(x) + (rnd() - 0.5) * 1.6;
    let pz = g.wz(z) + (rnd() - 0.5) * 1.6;
    let turn = rnd() * PI * 2.0;
    let salt = salt(px, pz, 3);
    let broom = group();
    let besom = group();
    // modelled from the handle's end along +x, as the old one lay
    besom.scale().set_scalar(k);
    broom.add(&besom);
    put(&besom, "Real__besomHandle", &lambert!({ map: &bark_tex(salt) }));
    put(&besom, "Real__besomTwigs", &lambert!({ color: 0x55402f }));
    put(&besom, "Real__besomTwine", &lambert!({ color: 0x8a7754 }));
    broom.position().set(px, 0, pz);
    broom.rotation().y = turn;
    g.world.add(&broom);
}

/// A hazel stick's bark (laid along it: u along the stick): grey-brown,
/// streaked along its length, its lenticels pale dashes, darker where
/// hands held it
fn bark_tex(salt: u32) -> Tex {
    canvas_tex_opt(1024, 64, false, move |ctx, w, h| {
        let seed = salt ^ 0xba7;
        fill(ctx, w, h, |u, v| {
            let n = fbm(seed, u * 40.0, v * 3.0, 4);
            let mut c = mix([0.40, 0.34, 0.27], [0.52, 0.45, 0.36], n);
            let streak = fbm(seed ^ 3, u * 6.0, v * 30.0, 3);
            c = mix(c, [0.30, 0.25, 0.20], sstep(0.55, 0.75, streak) * 0.5);
            let l = vn(seed ^ 5, u * 900.0, v * 26.0);
            if l > 0.86 {
                c = mix(c, [0.66, 0.60, 0.50], sstep(0.86, 0.92, l) * 0.8);
            }
            // where it was held, a third of the way along
            c = mix(c, [0.28, 0.24, 0.20], (1.0 - ((u - 0.25) / 0.12).abs()).max(0.0) * 0.35);
            [c[0], c[1], c[2], 1.0]
        });
    })
}
