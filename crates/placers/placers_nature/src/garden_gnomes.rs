//! `place_garden_gnomes`
use game::prelude::*;
use placers_kit::garden::put;
use placers_kit::noise::fbm;
use placers_kit::noise::mix;
use placers_kit::noise::sstep;
use placers_kit::noise::vn;
use placers_kit::paint::fill;
use placers_kit::paint::rgb;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;
use placers_kit::place::salt_of;

// ============================================================ garden gnomes
// Garden gnome: 3% chance per level.
pub(crate) fn place_garden_gnomes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    } // 3% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    // A 12 in garden gnome of painted cast concrete standing on his rock
    // (Blender family `garden_things`), the paint faded, chipped to the grey at every
    // edge, grimy at his foot. The old draw of his coat's colour chooses it:
    // a faded blue, a green, a brick red, a brown
    let body_colors: [u32; 4] = [0x2060a0, 0x408040, 0xa04040, 0x604020];
    let coat = pick(&body_colors);
    let (coat, trousers) = match coat {
        0x2060a0 => (0x4a6480, 0x5a4a38),
        0x408040 => (0x50684a, 0x5a4a38),
        0xa04040 => (0x7e4238, 0x46505a),
        _ => (0x664c36, 0x46505a),
    };
    let gnome = group();
    // where he stands and how he is turned (the old draws)
    gnome.position().set(g.wx(x) + (rnd() - 0.5) * 1.8, 0, g.wz(z) + (rnd() - 0.5) * 1.8);
    gnome.rotation().y = rnd() * PI * 2.0;
    let p = gnome.pos();
    let salt = salt_of(p.x, p.z);
    // he faces -z, as the old one did
    let body = group();
    body.rotation().y = PI;
    gnome.add(&body);
    let paint = |c: u32, chips: f64, k: u32| lambert!({ map: &painted_tex(rgb(c), chips, salt ^ k) });
    put(&body, "Real__gnomeBase", &paint(0x6c6e5c, 1.2, 1));
    put(&body, "Real__gnomeBoots", &paint(0x2e2822, 1.0, 2));
    put(&body, "Real__gnomeTrousers", &paint(trousers, 0.6, 3));
    let coat_mat = paint(coat, 0.8, 4);
    put(&body, "Real__gnomeCoat", &coat_mat);
    put(&body, "Real__gnomeSleeves", &coat_mat);
    put(&body, "Real__gnomeBelt", &paint(0x2a2622, 0.8, 5));
    put(&body, "Real__gnomeBuckle", &paint(0x9a8448, 1.2, 6));
    put(&body, "Real__gnomeSkin", &paint(0xc49a80, 0.7, 7));
    put(&body, "Real__gnomeBeard", &paint(0xc8c2b2, 0.5, 8));
    put(&body, "Real__gnomeHat", &paint(0x8c3a30, 1.0, 9));
    put(&body, "Real__gnomeEyes", &phong!({ color: 0x1e1a18, specular: 0x3a3a38, shininess: 60 }));
    g.world.add(&gnome);
}

/// Paint over cast concrete, faded, chipped to the grey at its edges and
/// here and there, grimy toward the foot (v 0) and in its hollows: for the
/// gnome's parts, laid on by the part's own box
fn painted_tex(paint: [f64; 3], chips: f64, salt: u32) -> Tex {
    canvas_tex_opt(256, 256, false, move |ctx, w, h| {
        let seed = salt ^ 0x6e0e;
        fill(ctx, w, h, |u, v| {
            let n = fbm(seed, u * 9.0, v * 9.0, 4);
            let mut c = paint.map(|k| k * (0.9 + 0.2 * n));
            // faded where the light reached it
            c = mix(c, [0.62, 0.6, 0.56], 0.12 + 0.1 * fbm(seed ^ 3, u * 3.0, v * 3.0, 2));
            // chips: the paint gone to the grey cement, a dark rim round each
            let ch = fbm(seed ^ 5, u * 46.0, v * 46.0, 4) + 0.1 * sstep(0.12, 0.0, v.min(u).min(1.0 - u).min(1.0 - v));
            let edge = sstep(0.71 - chips * 0.05, 0.725 - chips * 0.05, ch);
            let rim = sstep(0.68 - chips * 0.05, 0.71 - chips * 0.05, ch) - edge;
            c = mix(c, c.map(|k| k * 0.6), rim * 0.7);
            c = mix(c, [0.5, 0.49, 0.46], edge);
            // grime: toward the foot and in specks
            c = mix(c, [0.2, 0.18, 0.15], sstep(0.35, 0.0, v) * 0.35 + (vn(seed ^ 7, u * 160.0, v * 160.0) - 0.82).max(0.0) * 2.5);
            [c[0], c[1], c[2], 1.0]
        });
    })
}
