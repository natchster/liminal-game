//! `place_hand_towel_bars`
use game::prelude::*;
use placers_kit::geometry::put;
use placers_kit::noise::fbm;
use placers_kit::noise::hash;
use placers_kit::noise::sstep;
use placers_kit::noise::vn;
use placers_kit::paint::fill;
use placers_kit::place::mount;
use placers_kit::place::vary as t04_vary;
use placers_kit::place::wall_candidates;
use placers_kit::place::wall_xz;

// Wall towel bar: 6% chance per level. A 12 in towel bar of the hardware
// store's kind (Blender batch `t04`): a 3/4 in chromed tube on two posts and
// their rosettes, the plating dulled; over it a cotton terry hand towel
// folded in half along its length, hung and left (draped by Blender's
// cloth), gone grey and stiff, its dobby border across each end, mildew
// specked low down where it stayed damp. Its colour the old one's: cream,
// pink, blue or green, faded.
pub(crate) fn place_hand_towel_bars(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = choice(&candidates);
    let towel_colors = [0xf0ece0, 0xf0c0d0, 0xd0e0f0, 0xd0e0d0];
    let towel_color = choice(&towel_colors);
    let kind = towel_colors.iter().position(|&k| k == towel_color).unwrap_or(0);
    let bar = group();
    let (wall_x, wall_z) = wall_xz(g, c, 0.010);
    mount(&bar, c, wall_x, wall_z, 1.15, None);
    let p = bar.pos();
    let vary = t04_vary(p.x, p.z, kind as f64);
    // the parts are modelled with the wall at z = 0: 10 mm behind the spot
    let o = group();
    o.position().z = -0.010;
    bar.add(&o);
    put(&o, "Real__towelBarMetal", &phong!({ map: &tb_chrome_tex(vary), color: 0x8c8e90, specular: 0x6a6c6e, shininess: 64 }));
    put(&o, "Real__towelBarTowel", &lambert!({ map: &tb_terry_tex(vary, kind) }));
    g.world.add(&bar);
}

/// The bar's chrome after years in a washroom's damp: dulled, a bloom of
/// tarnish, pits (a grey the chrome's colour multiplies)
fn tb_chrome_tex(vary: u32) -> Tex {
    canvas_tex_opt(64, 64, false, move |ctx, w, h| {
        let seed = vary ^ 0x7b1;
        fill(ctx, w, h, |u, v| {
            let n = fbm(seed, u * 5.0, v * 5.0, 3);
            let tarnish = sstep(0.55, 0.78, fbm(seed ^ 0x9, u * 3.0, v * 3.0, 3)) * 0.3;
            let pit = if hash(seed ^ 0x2, (u * 64.0) as i32, (v * 64.0) as i32) < 0.015 { 0.4 } else { 0.0 };
            let g0 = 0.84 + 0.1 * n - tarnish - pit;
            [g0, g0 * 0.99, g0 * 0.96, 1.0]
        });
    })
}

/// The towel part's bounds (`Real__towelBarTowel`): how far down it hangs
/// below its top, for where its ends are in the map's v
const TW_SPAN: f64 = 0.3365;

/// The hand towel's terry (the part's box faces: its front laid across u,
/// v up): the loops' pile, the dobby border woven across near each end and
/// the hem at the end, the colour gone grey and dingy, darker where hands
/// dried on it, mildew specks low down, a tide mark or two where it dried
fn tb_terry_tex(vary: u32, kind: usize) -> Tex {
    canvas_tex_opt(224, 336, false, move |ctx, w, h| {
        let seed = vary ^ 0x7e11;
        let base = [[0.79, 0.76, 0.68], [0.74, 0.60, 0.60], [0.60, 0.65, 0.70], [0.62, 0.67, 0.59]][kind.min(3)];
        fill(ctx, w, h, |u, v| {
            // in metres from the towel's ends (its lowest point at v 0)
            let y = v * TW_SPAN;
            let x = u * 0.224;
            // the pile: loops and the pits between them
            let loops = vn(seed, x * 900.0, y * 820.0);
            let pits = vn(seed ^ 0x51, x * 1600.0 + 0.5, y * 1500.0);
            let mut shade = 0.86 + 0.16 * loops - 0.12 * sstep(0.62, 0.9, pits);
            // the dobby border: a flat weave band, ribbed across
            let bd = (y - 0.055).abs();
            let border = 1.0 - sstep(0.016, 0.018, bd);
            if border > 0.0 {
                // flat, its weft ribbed across, a corded line at each edge
                let rib = 0.5 + 0.5 * (y * 2.0 * PI * 420.0).sin();
                let cord = (-((bd - 0.0155) / 0.0012).powi(2)).exp();
                let flat = 1.04 + 0.08 * rib - 0.22 * cord;
                shade = shade * (1.0 - border) + flat * border;
            }
            // the hem: the end turned under and stitched
            if y < 0.020 {
                let rib = 0.5 + 0.5 * (y * 2.0 * PI * 380.0).sin();
                shade = 0.88 + 0.06 * rib - 0.12 * (1.0 - sstep(0.0, 0.004, y)) - 0.10 * (-((y - 0.012) / 0.0008).powi(2)).exp();
            }
            // the years: grey through, dingier and darker low down where
            // hands were dried on it, blotched
            let n = fbm(seed ^ 0x23, x * 14.0, y * 14.0, 4);
            let low = 1.0 - sstep(0.0, 0.22, y);
            let dirt = 0.10 + 0.14 * n + 0.18 * low * (0.5 + n);
            let grey = [0.50, 0.48, 0.44];
            let mut c = [0.0; 3];
            for k in 0..3 {
                let b = base[k] * 0.78 + grey[k] * 0.22;
                c[k] = b * shade * (1.0 - dirt);
            }
            // a tide line where it dried after a wetting
            let tide = fbm(seed ^ 0x61, x * 9.0, y * 9.0, 3);
            let ring = (-((tide - 0.62) / 0.012).powi(2)).exp() * (1.0 - sstep(0.1, 0.25, y)) * 0.25;
            for (k, t) in c.iter_mut().zip([0.45, 0.38, 0.28]) {
                *k = *k * (1.0 - ring) + t * ring * 0.8;
            }
            // mildew: grey-black specks in clusters, where it stayed damp
            let cl = sstep(0.58, 0.72, fbm(seed ^ 0x3a, x * 20.0, y * 20.0, 3)) * (0.3 + low);
            if hash(seed ^ 0x3b, (u * 224.0) as i32, (v * 336.0) as i32) < 0.10 * cl {
                for (k, m) in c.iter_mut().zip([0.16, 0.17, 0.15]) {
                    *k = *k * 0.35 + m * 0.65;
                }
            }
            [c[0], c[1], c[2], 1.0]
        });
    })
}
