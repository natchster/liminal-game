//! `place_small_chain_links`
use game::prelude::*;
use placers_kit::geometry::put;
use placers_kit::noise::fbm;
use placers_kit::noise::mix;
use placers_kit::noise::sstep;
use placers_kit::noise::vn;
use placers_kit::paint::fill;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;
use placers_kit::place::salt;

// Small chain links: 8% chance per level. A short length of 3/8 in
// proof coil chain (5 to 8 welded links of 9.9 mm wire, 55 mm long) lying
// where it was dropped, every other link flat and the ones between on edge,
// leaning; galvanized gone dull with white rust, bare steel rusted, or
// black oiled steel blooming rust, by the old colour.
pub(crate) fn place_small_chain_links(g: &mut Game, _dist: &[i32]) {
    g.pp.chain_positions.clear(); // reset per level
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = pick(&open_tiles);
    let chain_colors = [0x606870, 0x808088, 0x40484c];
    let finish: u32 = pick(&chain_colors);
    let n_links = 5 + (rnd() * 4.0).floor() as i32;
    let px = g.wx(x) + (rnd() - 0.5) * 2.0;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.0;
    let turn = rnd() * PI * 2.0;
    let salt = salt(px, pz, 0xc4a);
    let mat = match finish {
        // hot-dip galvanized gone dull grey, white rust in its hollows
        0x808088 => phong!({ map: &chain_tex(salt, 0), specular: 0x262626, shininess: 18 }),
        // self-coloured steel, rusted
        0x606870 => lambert!({ map: &chain_tex(salt, 1) }),
        // black oiled, rust blooming through
        _ => phong!({ map: &chain_tex(salt, 2), specular: 0x1c1c1c, shininess: 14 }),
    };
    let chain = group();
    put(&chain, &format!("Real__chainLinks{}", n_links.clamp(5, 8)), &mat);
    chain.position().set(px, 0, pz);
    chain.rotation().y = turn;
    g.world.add(&chain);
    // Track for jangle audio.
    let cp = chain.pos();
    g.pp.chain_positions.push(v3(cp.x, 0.02, cp.z));
}

/// The chain's steel (box-mapped over the whole length): `kind` 0
/// galvanized, 1 rusted bare steel, 2 black oiled
fn chain_tex(salt: u32, kind: u32) -> Tex {
    canvas_tex_opt(64, 64, false, move |ctx, w, h| {
        let seed = salt ^ 0x3c3 ^ kind.wrapping_mul(0x51);
        fill(ctx, w, h, |u, v| {
            let n = fbm(seed, u * 7.0, v * 7.0, 3);
            let s = vn(seed ^ 4, u * 31.0, v * 31.0);
            let rust = [0.42, 0.22, 0.11];
            let c = match kind {
                0 => {
                    let mut c = mix([0.58, 0.58, 0.56], [0.47, 0.48, 0.47], n);
                    c = mix(c, [0.76, 0.75, 0.71], sstep(0.62, 0.8, s) * 0.7);
                    mix(c, rust, sstep(0.68, 0.85, n) * 0.5)
                }
                1 => {
                    let c = mix([0.44, 0.26, 0.15], [0.32, 0.19, 0.11], n);
                    mix(c, [0.36, 0.35, 0.33], sstep(0.55, 0.8, s) * 0.35)
                }
                _ => {
                    let c = mix([0.12, 0.12, 0.11], [0.17, 0.16, 0.15], n);
                    mix(c, rust, sstep(0.6, 0.82, fbm(seed ^ 6, u * 12.0, v * 12.0, 2)) * 0.8)
                }
            };
            [c[0], c[1], c[2], 1.0]
        });
    })
}
