//! Batch `t17`'s garden things in this crate, made real (`docs/PROPS.md`
//! P3; `assets/src/blender/props/t17.py`, `t17.notes.md`): the gardening
//! gloves (a pair of cotton jersey gloves with knitted wrists, dropped one
//! across the other, draped by Blender's cloth) and the garden rake (a bow
//! rake: a tempered steel head on two bows, an ash handle). Each placer keeps
//! the old one's draws, call for call.
use crate::{open_tiles, pick};
use game::assets::props::{prop_part_as_modelled, prop_part_info};
use game::prelude::*;

/// The Blender part `name` as modelled (its own frame, a scale of one)
fn part(name: &str) -> Option<Geo> {
    prop_part_as_modelled(name)
}

/// The bounds (frame) part `name` was modelled in
fn frame(name: &str) -> ([f64; 3], [f64; 3]) {
    prop_part_info(name).map(|m| (m.fmin, m.fmax)).unwrap_or(([-0.05; 3], [0.05; 3]))
}

/// A hash of an integer lattice point, in [0, 1)
fn hash(seed: u32, i: i32, j: i32) -> f64 {
    let mut h = seed ^ (i as u32).wrapping_mul(0x27d4_eb2d) ^ (j as u32).wrapping_mul(0x1656_67b1);
    h = (h ^ (h >> 15)).wrapping_mul(0x85eb_ca6b);
    h = (h ^ (h >> 13)).wrapping_mul(0xc2b2_ae35);
    h ^= h >> 16;
    f64::from(h) / 4_294_967_296.0
}

/// Value noise in [0, 1]
fn vn(seed: u32, x: f64, y: f64) -> f64 {
    let (xf, yf) = (x.floor(), y.floor());
    let (i, j) = (xf as i32, yf as i32);
    let (u, v) = (x - xf, y - yf);
    let (u, v) = (u * u * (3.0 - 2.0 * u), v * v * (3.0 - 2.0 * v));
    let (a, b) = (hash(seed, i, j), hash(seed, i + 1, j));
    let (c, d) = (hash(seed, i, j + 1), hash(seed, i + 1, j + 1));
    a + (b - a) * u + (c - a) * v + (a - b - c + d) * u * v
}

/// Fractal value noise of `oct` octaves, in [0, 1]
fn fbm(seed: u32, x: f64, y: f64, oct: u32) -> f64 {
    let (mut s, mut amp, mut f, mut n) = (0.0, 0.5, 1.0, 0.0);
    for k in 0..oct {
        s += amp * vn(seed.wrapping_add(k * 7919), x * f, y * f);
        n += amp;
        amp *= 0.5;
        f *= 2.03;
    }
    s / n
}

fn sstep(e0: f64, e1: f64, x: f64) -> f64 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn mix(a: [f64; 3], b: [f64; 3], t: f64) -> [f64; 3] {
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
}

fn scale3(a: [f64; 3], k: f64) -> [f64; 3] {
    [a[0] * k, a[1] * k, a[2] * k]
}

/// Fill the canvas pixel by pixel: `f(x, y)` (pixel centres) gives a
/// straight sRGB colour. Pure (no draws, no canvas calls), so its rows are
/// filled on every core
fn paint_px(ctx: &mut Canvas, w: f64, h: f64, f: impl Fn(f64, f64) -> [f64; 3] + Sync) {
    let n = w as usize;
    let mut img = ctx.create_image_data(w, h);
    par_rows(&mut img.data, n * 4, |j, out| {
        for i in 0..n {
            let c = f(i as f64 + 0.5, j as f64 + 0.5);
            for k in 0..3 {
                out[i * 4 + k] = (c[k].clamp(0.0, 1.0) * 255.0).round() as u8;
            }
            out[i * 4 + 3] = 255;
        }
    });
    ctx.put_image_data(&img, 0, 0);
}

/// A salt from where a thing lies (no draw): a painter's own stream starts
/// the same on every floor, so what it paints for each thing mixes this in
fn salt_of(x: f64, z: f64) -> u32 {
    let (a, b) = ((x * 1000.0).round() as i64 as u64, (z * 1000.0).round() as i64 as u64);
    let mut h = a.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ b.wrapping_mul(0xC2B2_AE3D_27D4_EB4F);
    h ^= h >> 29;
    (h ^ (h >> 32)) as u32
}

fn rgb(c: u32) -> [f64; 3] {
    [f64::from((c >> 16) & 255) / 255.0, f64::from((c >> 8) & 255) / 255.0, f64::from(c & 255) / 255.0]
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
            let wale = 0.5 + 0.5 * (x * 2.0 * PI / 0.0011).sin();
            let fib = fbm(salt, x * 900.0, z * 300.0, 2);
            let mut c = scale3(base, 0.9 + 0.06 * wale + 0.12 * (fib - 0.5));
            // faded unevenly
            c = mix(c, [0.55, 0.53, 0.48], 0.18 * fbm(salt ^ 7, x * 30.0, z * 30.0, 3));
            if dots {
                // PVC dots, 2.6 mm, staggered rows 4.5 mm apart
                let (gx, gz) = (x / 0.0045, z / 0.0045);
                let row = gz.round();
                let off = if (row as i64) % 2 == 0 { 0.0 } else { 0.5 };
                let (dx, dz) = ((gx - off - (gx - off).round()) * 0.0045, (gz - row) * 0.0045);
                let r = dx.hypot(dz);
                let worn = sstep(0.45, 0.75, fbm(salt ^ 0x33, x * 60.0, z * 60.0, 3));
                let dot = (1.0 - sstep(0.0011, 0.0014, r)) * (1.0 - 0.85 * worn);
                c = mix(c, [0.10, 0.10, 0.09], dot * 0.9);
            }
            // soil: dried into it in patches, ground into the fingers' end
            let tips = if tips_low { 1.0 - v } else { v };
            let soil_n = fbm(salt ^ 0x51, x * 45.0, z * 45.0, 4);
            let soil =
                sstep(0.55, 0.7, soil_n + 0.25 * sstep(0.55, 0.95, tips)) * (0.55 + 0.25 * fbm(salt ^ 0x52, x * 200.0, z * 200.0, 2));
            c = mix(c, [0.27, 0.21, 0.15], soil);
            // grime overall, darker toward its edge where it rubbed
            c = mix(c, scale3(c, 0.78), 0.4 * sstep(0.62, 0.8, fbm(salt ^ 0x61, u * 6.0, v * 6.0, 3)));
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
            let rib = (0.5 + 0.5 * (x * 2.0 * PI / 0.0024).sin()).max(0.5 + 0.5 * (z * 2.0 * PI / 0.0024).sin() * 0.0);
            let mut c = scale3(base, 0.8 + 0.25 * rib);
            c = mix(c, [0.52, 0.50, 0.46], 0.15 * fbm(salt ^ 3, x * 40.0, z * 40.0, 3));
            let soil = sstep(0.62, 0.75, fbm(salt ^ 0x71, x * 50.0, z * 50.0, 4)) * 0.6;
            mix(c, [0.28, 0.22, 0.16], soil)
        });
    })
}

// R641 gardening gloves (pair): 7% chance per level. A pair of cotton jersey
// gardening gloves with knitted wrists (Blender batch `t17`, draped by its
// cloth): the right lying back up, the left dropped across it palm up, its
// PVC grip dots half worn away; soil dried into both.
pub fn place_gardening_gloves(g: &mut Game, _dist: &[i32]) {
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
    const JERSEY: [u32; 5] = [0x667a5c, 0x8a4a38, 0x7a7650, 0x6a5440, 0xb0a084];
    const RIB: [u32; 5] = [0x4c5a46, 0x6a3a2e, 0x5a583c, 0x4e3e30, 0x8a7c64];
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
            let mut c = mix([0.62, 0.53, 0.40], [0.52, 0.48, 0.43], 0.55 + 0.3 * fbm(salt ^ 1, u * 4.0, v * 2.0, 3));
            c = scale3(c, 0.9 + 0.12 * grain);
            c = mix(c, [0.40, 0.31, 0.21], 0.35 * streak);
            // the hands' places: the lead hand's and the end's
            let grip = sstep(0.40, 0.46, u) * (1.0 - sstep(0.60, 0.66, u)) + sstep(0.84, 0.90, u) * (1.0 - sstep(0.985, 1.0, u));
            c = mix(c, [0.36, 0.30, 0.24], 0.45 * grip);
            // grime and rust stain into the wood at the ferrule
            c = mix(c, [0.30, 0.22, 0.15], 0.6 * (1.0 - sstep(0.06, 0.12, u)));
            // knocks and scuffs
            let knock = sstep(0.8, 0.85, fbm(salt ^ 9, u * 60.0, v * 3.0, 2));
            mix(c, [0.68, 0.62, 0.52], 0.4 * knock)
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
            let mut rust = mix([0.36, 0.20, 0.11], [0.22, 0.13, 0.08], rust_n);
            rust = mix(rust, [0.44, 0.28, 0.14], 0.4 * sstep(0.6, 0.75, fbm(salt ^ 0x78, u * 90.0, v * 90.0, 2)));
            let chips = fbm(salt ^ 0x79, u * 22.0, v * 22.0, 4);
            let paint_left = 1.0 - sstep(0.44, 0.56, chips);
            let mut c = mix(rust, scale3(paint, 0.9 + 0.2 * fbm(salt ^ 0x7a, u * 8.0, v * 8.0, 2)), paint_left);
            // rust blooming through what paint is left
            c = mix(c, rust, 0.35 * sstep(0.55, 0.7, fbm(salt ^ 0x7b, u * 50.0, v * 50.0, 3)) * paint_left);
            // dust
            mix(c, [0.48, 0.45, 0.40], 0.10)
        });
    })
}

// R644 garden rake: 4% chance per level. A bow rake dropped on the floor
// (Blender batch `t17`): 14 tempered steel tines on a head carried by two
// bows, a steel ferrule, a 54 in ash handle; its enamel chipped to rust, the
// handle grey, dark where it was held. It lies on its tines and its handle's
// end, as a dropped one does.
pub fn place_garden_rakes(g: &mut Game, _dist: &[i32]) {
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
