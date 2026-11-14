//! `place_gardening_gloves`
use crate::shared::paint_px_rgb as paint_px;
use game::prelude::*;
use placers_kit::geometry::frame;
use placers_kit::geometry::part;
use placers_kit::noise::fbm;
use placers_kit::noise::mix_unclamped;
use placers_kit::noise::sstep;
use placers_kit::paint::mul;
use placers_kit::paint::rgb;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;
use placers_kit::place::salt_of;

// Gardening gloves (pair): 7% chance per level. A pair of cotton jersey
// gardening gloves with knitted wrists (Blender family `a_garden_and_a_wood_indoors`, draped by its
// cloth): the right lying back up, the left dropped across it palm up, its
// PVC grip dots half worn away; soil dried into both.
pub(crate) fn place_gardening_gloves(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.93 {
        return;
    } // 7% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let glove_colors: [u32; 5] = [0x408040, 0xa04020, 0x808030, 0x604020, 0xc0a060];
    // the old colour's draw: the jersey's dye (sage green, brick red, olive,
    // brown, oatmeal), faded
    let col = pick(&glove_colors);
    const JERSEY: [u32; 5] = [0x7a8c6c, 0x9a5644, 0x8a8660, 0x866e56, 0xb8aa8e];
    const RIB: [u32; 5] = [0x56644e, 0x723e30, 0x646244, 0x5c4a38, 0x948670];
    let k = glove_colors.iter().position(|&c| c == col).unwrap_or(0);
    let pair = group();
    let (px, pz) = (g.wx(x) + (rnd() - 0.5) * 2.2, g.wz(z) + (rnd() - 0.5) * 2.2);
    let salt = salt_of(px, pz);
    for (name, dots, tips_low) in [("Real__gloveRBody", false, false), ("Real__gloveLBody", true, true)] {
        if let Some(geo) = part(name) {
            let (lo, hi) = frame(name);
            let tex = glove_tex(rgb(JERSEY[k]), hi[0] - lo[0], hi[2] - lo[2], dots, tips_low, salt ^ name.len() as u32);
            pair.add(&mesh(geo, lambert!({ map: &tex })));
        }
    }
    for name in ["Real__gloveRCuff", "Real__gloveLCuff"] {
        if let Some(geo) = part(name) {
            let (lo, hi) = frame(name);
            let tex = cuff_tex(rgb(RIB[k]), hi[0] - lo[0], hi[2] - lo[2], salt ^ 0x5c);
            pair.add(&mesh(geo, lambert!({ map: &tex })));
        }
    }
    pair.position().set(px, 0, pz);
    pair.rotation().y = rnd() * PI * 2.0;
    g.world.add(&pair);
}

// ================================================================= gloves
/// A glove's jersey seen from above over its part's bounds (`w` x `d` m):
/// the cotton's colour faded and greyed, its fibres, soil dried into it in
/// patches and ground into its fingers' end of it (`tips_low`: the end at
/// low v), worn thin and darker where it rubbed; `dots`: the palm's PVC
/// grip dots in rows, half worn away
fn glove_tex(base: [f64; 3], w: f64, d: f64, dots: bool, tips_low: bool, salt: u32) -> Tex {
    let ppm = 1500.0;
    let (pw, ph) = ((w * ppm).round().max(16.0), (d * ppm).round().max(16.0));
    canvas_tex_opt(pw, ph, false, move |ctx, cw, chh| {
        paint_px(ctx, cw, chh, |px, py| {
            let (x, z) = (px / ppm, py / ppm);
            let (u, v) = (px / cw, py / chh);
            // the knit: fine wales along it, a fleck of fibre
            let wale = 0.5 + 0.5 * (x * 2.0 * PI / 0.0011).dsin();
            let fib = fbm(salt, x * 900.0, z * 300.0, 2);
            let mut c = mul(base, 0.9 + 0.06 * wale + 0.12 * (fib - 0.5));
            // faded unevenly
            c = mix_unclamped(c, [0.55, 0.53, 0.48], 0.18 * fbm(salt ^ 7, x * 30.0, z * 30.0, 3));
            if dots {
                // PVC dots, 2.4 mm, staggered rows 5.5 mm apart, worn away
                // in patches where it gripped
                let (gx, gz) = (x / 0.0055, z / 0.0055);
                let row = gz.round();
                let off = if (row as i64) % 2 == 0 { 0.0 } else { 0.5 };
                let (dx, dz) = ((gx - off - (gx - off).round()) * 0.0055, (gz - row) * 0.0055);
                let r = dx.dhypot(dz);
                let worn = sstep(0.40, 0.62, fbm(salt ^ 0x33, x * 40.0, z * 40.0, 3));
                let dot = (1.0 - sstep(0.0010, 0.0013, r)) * (1.0 - 0.92 * worn);
                c = mix_unclamped(c, [0.10, 0.10, 0.09], dot * 0.9);
            }
            // soil: dried into it in patches, ground into the fingers' end
            let tips = if tips_low { 1.0 - v } else { v };
            let soil_n = fbm(salt ^ 0x51, x * 45.0, z * 45.0, 4);
            let soil =
                sstep(0.55, 0.7, soil_n + 0.25 * sstep(0.55, 0.95, tips)) * (0.55 + 0.25 * fbm(salt ^ 0x52, x * 200.0, z * 200.0, 2));
            c = mix_unclamped(c, [0.27, 0.21, 0.15], soil);
            // grime overall, darker toward its edge where it rubbed
            c = mix_unclamped(c, mul(c, 0.78), 0.4 * sstep(0.62, 0.8, fbm(salt ^ 0x61, u * 6.0, v * 6.0, 3)));
            c
        });
    })
}

/// The knitted wrist seen from above: ribs along the glove (columns), its
/// edge's turned-in welt, soil and wear
fn cuff_tex(base: [f64; 3], w: f64, d: f64, salt: u32) -> Tex {
    let ppm = 1200.0;
    let (pw, ph) = ((w * ppm).round().max(16.0), (d * ppm).round().max(16.0));
    canvas_tex_opt(pw, ph, false, move |ctx, cw, chh| {
        paint_px(ctx, cw, chh, |px, py| {
            let (x, z) = (px / ppm, py / ppm);
            // 1x1 rib, 2.4 mm a rib, both ways (the cuff lies every way)
            let rib = (0.5 + 0.5 * (x * 2.0 * PI / 0.0024).dsin()).max(0.5 + 0.5 * (z * 2.0 * PI / 0.0024).dsin() * 0.0);
            let mut c = mul(base, 0.8 + 0.25 * rib);
            c = mix_unclamped(c, [0.52, 0.50, 0.46], 0.15 * fbm(salt ^ 3, x * 40.0, z * 40.0, 3));
            let soil = sstep(0.62, 0.75, fbm(salt ^ 0x71, x * 50.0, z * 50.0, 4)) * 0.6;
            mix_unclamped(c, [0.28, 0.22, 0.16], soil)
        });
    })
}
