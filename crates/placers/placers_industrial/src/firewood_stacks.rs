//! `place_firewood_stacks`
use game::prelude::*;
use placers_kit::baking::fbm as r16_fbm;
use placers_kit::baking::mix as r16_mix;
use placers_kit::baking::noise as r16_noise;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// A stack of firewood (Blender batch `r16`): 3-5 splits of stove wood,
// 300 mm long (a quarter of a big round, half a small one, a slab off a big
// one, a branch round), a bottom row and one or two laid in its grooves as
// the old stack was; their bark (oak, ash, elm or birch: the old bark
// colours), the faces they were split along gone grey, their sawn ends'
// rings round the pith, checked by drying. 5% chance per level.
pub(crate) fn place_firewood_stacks(g: &mut Game, _dist: &[i32]) {
    use game::assets::props::prop_part_as_modelled;
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    // the old bark colours: oak, ash, elm, birch
    let bark_colors: [u32; 4] = [0x6b3f1f, 0x805030, 0x503020, 0x9a6f4b];
    let bark_color = pick(&bark_colors);
    let species = bark_colors.iter().position(|&c| c == bark_color).unwrap_or(0);
    let count = 3 + (rnd() * 3.0).floor() as i32;
    // the old stack's loop: one draw a log (its turn); now the grain's seed
    let layers = 2i32.min((count as f64 / 2.0).floor() as i32);
    let bottom_row = ((count + 1) as f64 / 2.0).ceil() as i32;
    let mut idx = 0;
    let mut salt: u32 = 0x9e37_79b9 ^ count as u32;
    'rows: for layer in 0..=layers {
        let row_count = bottom_row - layer;
        if row_count <= 0 {
            break;
        }
        for _ in 0..row_count {
            let turn = (rnd() - 0.5) * 0.1;
            salt = salt.rotate_left(7) ^ (turn * 1.0e7) as i64 as u32;
            idx += 1;
            if idx >= count {
                break 'rows;
            }
        }
    }
    let n = count.clamp(3, 5) as usize;
    let stack = group();
    let mats = [
        lambert!({ map: &wood_bark_tex(species, salt) }),
        lambert!({ map: &wood_split_tex(species, salt) }),
        lambert!({ map: &wood_end_tex(species, salt, WOOD_PITHS[n - 3][0], false) }),
        lambert!({ map: &wood_end_tex(species, salt ^ 0x55, WOOD_PITHS[n - 3][1], true) }),
    ];
    for (k, part) in ["Bark", "Split", "EndL", "EndR"].iter().enumerate() {
        if let Some(geo) = prop_part_as_modelled(&format!("Real__woodStack{n}{part}")) {
            stack.add(&mesh(geo, &mats[k]));
        }
    }
    stack.position().set(g.wx(x) + (rnd() - 0.5) * 1.8, 0, g.wz(z) + (rnd() - 0.5) * 1.8);
    stack.rotation().y = rnd() * PI * 2.0;
    g.world.add(&stack);
}

/// Each stack's piths on its two sawn ends, in its end parts' texture
/// coordinates (u, v), and the bark's radius about each (in u and in v):
/// printed by domestic/kitchen_and_hearth_left_behind.py (`_print_piths`) from the stacks as modelled
type WoodPiths = &'static [(f64, f64, f64, f64)];

const WOOD_PITHS: [[WoodPiths; 2]; 3] = [
    [
        &[(0.0055, 0.0258, 0.4243, 0.7347), (0.7389, 0.0258, 0.2586, 0.4478), (0.4235, 0.7248, 0.1697, 0.2939)],
        &[(0.9943, 0.0088, 0.4355, 0.7218), (0.2787, 0.0088, 0.2655, 0.4400), (0.5191, 0.6956, 0.1742, 0.2887)],
    ],
    [
        &[
            (0.2930, 0.0111, 0.2904, 0.4692),
            (0.1308, 0.0111, 0.4701, 0.7596),
            (0.8206, 0.0111, 0.1770, 0.2860),
            (0.1485, 0.5248, 0.2904, 0.4692),
        ],
        &[
            (0.7007, 0.0166, 0.2948, 0.4647),
            (0.8432, 0.0166, 0.4774, 0.7523),
            (0.1846, 0.0166, 0.1797, 0.2832),
            (0.8263, 0.5254, 0.2948, 0.4647),
        ],
    ],
    [
        &[
            (0.1849, 0.0213, 0.1769, 0.3314),
            (0.8716, 0.0213, 0.4700, 0.8802),
            (0.6964, 0.0213, 0.2903, 0.5436),
            (0.3737, 0.6553, 0.1161, 0.2175),
            (0.6888, 0.6590, 0.1769, 0.3314),
        ],
        &[
            (0.8206, 0.0135, 0.1771, 0.3313),
            (0.1686, 0.0135, 0.4704, 0.8801),
            (0.2960, 0.0135, 0.2906, 0.5436),
            (0.6333, 0.6475, 0.1162, 0.2174),
            (0.3105, 0.6512, 0.1771, 0.3313),
        ],
    ],
];

/// Bark by species (sRGB): its ridges, its fissures
const WOOD_BARK: [([f64; 3], [f64; 3]); 4] = [
    ([104.0, 96.0, 84.0], [44.0, 38.0, 32.0]),   // oak: grey-brown ridges, deep dark fissures
    ([128.0, 124.0, 112.0], [70.0, 66.0, 58.0]), // ash: grey, a diamond net of shallow fissures
    ([92.0, 74.0, 58.0], [40.0, 30.0, 24.0]),    // elm: brown, corky
    ([196.0, 192.0, 180.0], [42.0, 38.0, 34.0]), // birch: white, black lenticels and cracks
];

/// The wood itself, fresh-split and weathered (sRGB)
const WOOD_GRAIN: [([f64; 3], [f64; 3]); 4] = [
    ([170.0, 138.0, 100.0], [128.0, 122.0, 112.0]),
    ([186.0, 168.0, 132.0], [138.0, 132.0, 122.0]),
    ([150.0, 112.0, 80.0], [118.0, 108.0, 96.0]),
    ([198.0, 180.0, 146.0], [146.0, 140.0, 128.0]),
];

/// The bark round every log (the box's texture coordinates over the whole
/// stack: u along the logs): ridges along the grain between fissures (birch
/// across it: lenticels and peeling), lichen and dirt here and there
fn wood_bark_tex(species: usize, salt: u32) -> Tex {
    canvas_tex_opt(512, 256, false, move |ctx, w, h| {
        let seed = salt ^ 0xba4c;
        let (ridge, fissure) = WOOD_BARK[species];
        let n = w as usize;
        let mut img = ctx.create_image_data(w, h);
        par_rows(&mut img.data, n * 4, |j, out| {
            let y = j as f64;
            for i in 0..n {
                let x = i as f64;
                let mut c = ridge;
                if species == 3 {
                    // birch: white, dark lenticels across the grain (short
                    // lines along v... the logs run along u), black cracks
                    let len = r16_noise(seed, x / 2.0, y / 14.0);
                    let crack = (r16_fbm(seed ^ 1, x / 40.0, y / 6.0) - 0.62).max(0.0) * 4.0;
                    let dirt = r16_fbm(seed ^ 2, x / 60.0, y / 60.0);
                    r16_mix(&mut c, fissure, ((len - 0.8).max(0.0) * 6.0).min(0.9) + crack);
                    r16_mix(&mut c, [150.0, 140.0, 120.0], (dirt - 0.4).max(0.0));
                } else {
                    // ridges along u, fissures between, net-like for ash
                    let stretch = if species == 1 { 9.0 } else { 18.0 };
                    let f = r16_fbm(seed, x / stretch / 4.0, y / 4.5);
                    let fis = (1.0 - ((f - 0.5).abs() * 4.0).min(1.0)).powf(2.0);
                    let tone = r16_noise(seed ^ 3, x / 3.0, y / 3.0);
                    r16_mix(&mut c, fissure, fis * 0.9);
                    for k in 0..3 {
                        c[k] *= 0.85 + 0.3 * tone;
                    }
                }
                // lichen, grey-green, in patches; dirt where it lay
                let lichen = (r16_fbm(seed ^ 4, x / 30.0, y / 30.0) - 0.68).max(0.0) * 3.5;
                r16_mix(&mut c, [130.0, 136.0, 110.0], lichen * 0.6);
                out[i * 4..i * 4 + 4].copy_from_slice(&[
                    c[0].clamp(0.0, 255.0) as u8,
                    c[1].clamp(0.0, 255.0) as u8,
                    c[2].clamp(0.0, 255.0) as u8,
                    255,
                ]);
            }
        });
        ctx.put_image_data(&img, 0, 0);
    })
}

/// The faces it was split along (u along the logs): torn fibres along the
/// grain, the wood gone grey where the air had it, darker in the tears,
/// a few dark streaks of rot and dirt
fn wood_split_tex(species: usize, salt: u32) -> Tex {
    canvas_tex_opt(512, 256, false, move |ctx, w, h| {
        let seed = salt ^ 0x5b17;
        let (fresh, grey) = WOOD_GRAIN[species];
        let n = w as usize;
        let mut img = ctx.create_image_data(w, h);
        par_rows(&mut img.data, n * 4, |j, out| {
            let y = j as f64;
            for i in 0..n {
                let x = i as f64;
                let fib = r16_noise(seed, x / 40.0, y / 1.3) * 0.5 + r16_noise(seed ^ 1, x / 12.0, y / 0.7) * 0.5;
                let weather = 0.55 + 0.4 * r16_fbm(seed ^ 2, x / 50.0, y / 20.0);
                let mut c = fresh;
                r16_mix(&mut c, grey, weather);
                for k in 0..3 {
                    c[k] *= 0.78 + 0.36 * fib;
                }
                let tear = (r16_noise(seed ^ 3, x / 25.0, y / 2.0) - 0.78).max(0.0) * 4.0;
                r16_mix(&mut c, [60.0, 50.0, 42.0], tear * 0.7);
                let rot = (r16_fbm(seed ^ 4, x / 70.0, y / 10.0) - 0.7).max(0.0) * 3.0;
                r16_mix(&mut c, [74.0, 60.0, 50.0], rot * 0.6);
                out[i * 4..i * 4 + 4].copy_from_slice(&[
                    c[0].clamp(0.0, 255.0) as u8,
                    c[1].clamp(0.0, 255.0) as u8,
                    c[2].clamp(0.0, 255.0) as u8,
                    255,
                ]);
            }
        });
        ctx.put_image_data(&img, 0, 0);
    })
}

/// One end of the stack: each log's sawn end grain, its rings round its
/// pith (the heart darker for oak and elm), checks split out from the pith
/// as it dried, the chainsaw's faint arcs, the bark round it; weathered grey.
/// `piths`: (u, v, the bark's radius in u, in v) for each log.
fn wood_end_tex(species: usize, salt: u32, piths: &'static [(f64, f64, f64, f64)], right: bool) -> Tex {
    canvas_tex_opt(512, 320, false, move |ctx, w, h| {
        let seed = salt ^ if right { 0xe2d1 } else { 0xe2d0 };
        let (fresh, grey) = WOOD_GRAIN[species];
        let (ridge, _) = WOOD_BARK[species];
        let mut r = SplitMix64::new(seed, 0x4e11);
        // each log's checks: angles of the cracks out from its pith, how far
        let checks: Vec<Vec<(f64, f64)>> = piths
            .iter()
            .map(|_| (0..3 + (r.next() * 4.0) as usize).map(|_| (r.range(0.0, PI * 2.0), r.range(0.35, 0.95))).collect())
            .collect();
        let n = w as usize;
        let mut img = ctx.create_image_data(w, h);
        par_rows(&mut img.data, n * 4, |j, out| {
            let v = 1.0 - (j as f64 + 0.5) / h;
            for i in 0..n {
                let u = (i as f64 + 0.5) / w;
                // the nearest pith (in units of its bark's radius)
                let mut best = (f64::INFINITY, 0usize, 0.0, 0.0, 0.0);
                for (k, &(pu, pv, ru, rv)) in piths.iter().enumerate() {
                    let (du, dv) = ((u - pu) / ru, (v - pv) / rv);
                    let d = (du * du + dv * dv).sqrt();
                    if d < best.0 {
                        best = (d, k, du, dv, ru);
                    }
                }
                let (d, k, du, dv, ru) = best;
                // the log's radius in metres (the ends are about 0.35 m across)
                let rad = ru * 0.35;
                let ang = dv.atan2(du);
                let wob = 0.025 * (r16_noise(seed ^ k as u32, ang * 2.5 + 10.0, d * 2.0) - 0.5);
                let rings = ((d + wob) * rad / 0.0032).fract();
                let late = (1.0 - ((rings - 0.82) / 0.12).abs()).max(0.0);
                let mut c = fresh;
                let heart = if species == 0 || species == 2 { (1.0 - d / 0.7).clamp(0.0, 1.0) } else { 0.0 };
                let dark = [c[0] * 0.74, c[1] * 0.66, c[2] * 0.6];
                r16_mix(&mut c, dark, heart.sqrt() * 0.8);
                for q in 0..3 {
                    c[q] *= 1.0 - 0.28 * late;
                }
                // weathered grey, more toward the rim; the pith a dark speck
                r16_mix(&mut c, grey, 0.35 + 0.25 * d.min(1.0));
                if d * rad < 0.0025 {
                    r16_mix(&mut c, [70.0, 56.0, 44.0], 0.7);
                }
                // the bark round it, a band its own thickness
                let band = 1.0 - 0.006 / rad.max(0.01);
                if d > band {
                    r16_mix(&mut c, [ridge[0] * 0.9, ridge[1] * 0.88, ridge[2] * 0.86], ((d - band) / 0.03).min(1.0) * 0.85);
                }
                // checks out from the pith
                for &(a, l) in &checks[k] {
                    let da = ((ang - a + PI).rem_euclid(PI * 2.0) - PI).abs();
                    let wid = 0.0012 / rad.max(0.01) * (1.0 - d / l).max(0.0);
                    if d < l && da * d.max(0.02) < wid {
                        r16_mix(&mut c, [40.0, 32.0, 26.0], 0.85);
                    }
                }
                // the chainsaw's arcs, faint
                let saw = (((u * 3.0 + v * 0.4) * 60.0 + (u * 7.0).sin() * 3.0).fract() - 0.5).abs();
                for q in 0..3 {
                    c[q] *= 0.96 + 0.08 * saw;
                }
                out[i * 4..i * 4 + 4].copy_from_slice(&[
                    c[0].clamp(0.0, 255.0) as u8,
                    c[1].clamp(0.0, 255.0) as u8,
                    c[2].clamp(0.0, 255.0) as u8,
                    255,
                ]);
            }
        });
        ctx.put_image_data(&img, 0, 0);
    })
}
