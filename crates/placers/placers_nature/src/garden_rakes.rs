//! `place_garden_rakes`
use crate::shared::paint_px_rgb as paint_px;
use game::prelude::*;
use placers_kit::geometry::part;
use placers_kit::noise::fbm;
use placers_kit::noise::mix_unclamped;
use placers_kit::noise::sstep;
use placers_kit::paint::mul;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;
use placers_kit::place::salt_of;

// Garden rake: 4% chance per level. A bow rake dropped on the floor
// (Blender batch `t17`): 14 tempered steel tines on a head carried by two
// bows, a steel ferrule, a 54 in ash handle; its enamel chipped to rust, the
// handle grey, dark where it was held. It lies on its tines and its handle's
// end, as a dropped one does.
pub(crate) fn place_garden_rakes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let rake = group();
    rake.position().set(g.wx(x) + (rnd() - 0.5) * 1.8, 0, g.wz(z) + (rnd() - 0.5) * 1.8);
    let p = rake.pos();
    let salt = salt_of(p.x, p.z);
    // the head's enamel, where it lies: green, black, red oxide, or none left
    let paint = [[0.17, 0.26, 0.16], [0.07, 0.07, 0.065], [0.36, 0.13, 0.08], [0.30, 0.18, 0.10]][(salt % 4) as usize];
    if let Some(geo) = part("Real__rakeHandle") {
        rake.add(&mesh(geo, lambert!({ map: &handle_tex(salt) })));
    }
    if let Some(geo) = part("Real__rakeSteel") {
        rake.add(&mesh(geo, lambert!({ map: &steel_tex(paint, salt) })));
    }
    rake.rotation().y = rnd() * PI * 2.0;
    g.world.add(&rake);
}

// =================================================================== rake
/// The ash handle seen along it (u from the head's end to its butt, v
/// across): pale ash gone grey, its grain in long streaks, the varnish worn
/// off, darker and smooth where hands held it, grime round the ferrule
fn handle_tex(salt: u32) -> Tex {
    canvas_tex_opt(1024, 48, false, move |ctx, w, h| {
        paint_px(ctx, w, h, |px, py| {
            let (u, v) = (px / w, py / h);
            let grain = fbm(salt, u * 9.0, v * 14.0, 4);
            let streak = ((v * 40.0 + grain * 6.0 + (u * 3.0).sin() * 0.6).sin() * 0.5 + 0.5).powf(4.0);
            let mut c = mix_unclamped([0.62, 0.53, 0.40], [0.52, 0.48, 0.43], 0.55 + 0.3 * fbm(salt ^ 1, u * 4.0, v * 2.0, 3));
            c = mul(c, 0.9 + 0.12 * grain);
            c = mix_unclamped(c, [0.40, 0.31, 0.21], 0.35 * streak);
            // the hands' places: the lead hand's and the end's
            let grip = sstep(0.40, 0.46, u) * (1.0 - sstep(0.60, 0.66, u)) + sstep(0.84, 0.90, u) * (1.0 - sstep(0.985, 1.0, u));
            c = mix_unclamped(c, [0.36, 0.30, 0.24], 0.45 * grip);
            // grime and rust stain into the wood at the ferrule
            c = mix_unclamped(c, [0.30, 0.22, 0.15], 0.6 * (1.0 - sstep(0.06, 0.12, u)));
            // knocks and scuffs
            let knock = sstep(0.8, 0.85, fbm(salt ^ 9, u * 60.0, v * 3.0, 2));
            mix_unclamped(c, [0.68, 0.62, 0.52], 0.4 * knock)
        });
    })
}

/// The steel (box-mapped): enamel (`paint`) chipped to rust, the rust dark
/// and pitted, bare where the tines wear on the soil
fn steel_tex(paint: [f64; 3], salt: u32) -> Tex {
    canvas_tex_opt(512, 256, false, move |ctx, w, h| {
        paint_px(ctx, w, h, |px, py| {
            let (u, v) = (px / w, py / h);
            let rust_n = fbm(salt ^ 0x77, u * 30.0, v * 30.0, 4);
            let mut rust = mix_unclamped([0.36, 0.20, 0.11], [0.22, 0.13, 0.08], rust_n);
            rust = mix_unclamped(rust, [0.44, 0.28, 0.14], 0.4 * sstep(0.6, 0.75, fbm(salt ^ 0x78, u * 90.0, v * 90.0, 2)));
            let chips = fbm(salt ^ 0x79, u * 22.0, v * 22.0, 4);
            let paint_left = 1.0 - sstep(0.44, 0.56, chips);
            let mut c = mix_unclamped(rust, mul(paint, 0.9 + 0.2 * fbm(salt ^ 0x7a, u * 8.0, v * 8.0, 2)), paint_left);
            // rust blooming through what paint is left
            c = mix_unclamped(c, rust, 0.35 * sstep(0.55, 0.7, fbm(salt ^ 0x7b, u * 50.0, v * 50.0, 3)) * paint_left);
            // dust
            mix_unclamped(c, [0.48, 0.45, 0.40], 0.10)
        });
    })
}
