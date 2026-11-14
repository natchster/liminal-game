//! `place_small_black_cats`
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

// Small black cat figurine: 6% chance per level. A cast resin black
// cat (15 cm), sitting, its tail round its feet, glazed black, its eyes
// painted yellow-green with a black slit; the glaze dusty, chipped to the
// grey resin at its ears and paws.
pub(crate) fn place_small_black_cats(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick_tile(&open);
    let px = g.wx(x) + (rnd() - 0.5) * 2.4;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.4;
    let turn = rnd() * PI * 2.0;
    let salt = salt(px, pz, 2);
    let cat = group();
    put(&cat, "Real__catBody", &phong!({ map: &glaze_tex(salt), specular: 0x3a3a3a, shininess: 45 }));
    // the paint catches the light; it never makes its own
    put(&cat, "Real__catEyes", &phong!({ color: 0x9a9238, specular: 0x5a5a5a, shininess: 70 }));
    put(&cat, "Real__catPupils", &phong!({ color: 0x1c1b1a, specular: 0x5a5a5a, shininess: 70 }));
    cat.position().set(px, 0, pz);
    cat.rotation().y = turn;
    g.world.add(&cat);
}

/// Black glaze on cast resin (laid on it from every side): not quite even,
/// dust settled in it, chips through to the grey resin, a few scratches
fn glaze_tex(salt: u32) -> Tex {
    canvas_tex_opt(256, 256, false, move |ctx, w, h| {
        let seed = salt ^ 0xca7;
        fill(ctx, w, h, |u, v| {
            let n = fbm(seed, u * 8.0, v * 8.0, 4);
            let mut c = mix([0.15, 0.145, 0.14], [0.20, 0.19, 0.18], n);
            let dust = vn(seed ^ 3, u * 160.0, v * 160.0);
            if dust > 0.84 {
                c = mix(c, [0.32, 0.31, 0.29], (dust - 0.84) * 3.0);
            }
            // a few chips through to the resin
            let chip = vn(seed ^ 5, u * 22.0, v * 22.0) + 0.10 * vn(seed ^ 7, u * 140.0, v * 140.0);
            if chip > 0.985 {
                c = mix(c, [0.50, 0.48, 0.44], sstep(0.985, 1.01, chip));
            }
            [c[0], c[1], c[2], 1.0]
        });
    })
}
