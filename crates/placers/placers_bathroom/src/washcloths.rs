//! `place_washcloths`
use game::prelude::*;
use placers_kit::noise::fbm;
use placers_kit::noise::hash;
use placers_kit::noise::mix;
use placers_kit::noise::sstep;
use placers_kit::noise::vn;
use placers_kit::paint::opaque;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Dropped washcloth: 8% chance per level.
pub(crate) fn place_washcloths(g: &mut Game, _dist: &[i32]) {
    // A cotton terry washcloth (batch `r18`), dropped: folded over on itself,
    // its corner curled up off it, lying limp; the pile, the flat-woven band
    // and the hem; gone grey with dirt, spotted with mildew, a stain dried
    // in it (docs/research/props-remodelling/washroom_janitor_classroom_and_plant.md)
    use game::assets::props::prop_part_as_modelled;
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = pick(&open_tiles);
    // the old colour draw: white, peach, blue, coral, sage, grey, each faded
    let cloth_colors = [0xffffff, 0xf0d8c0, 0x80a0c0, 0xe08070, 0xa0c0a0, 0xd0d0d0];
    let c = pick(&cloth_colors);
    let kind = cloth_colors.iter().position(|&k| k == c).unwrap_or(0);
    let salt = (x as u32).wrapping_mul(0x9e37_79b9) ^ (z as u32).wrapping_mul(0x85eb_ca6b) ^ (kind as u32).rotate_left(5);
    let cloth = group();
    for (name, band) in [("Real__washclothTerry", false), ("Real__washclothBorder", true)] {
        if let Some(geo) = prop_part_as_modelled(name) {
            let bb = geo.bounding_box();
            let tex = tex(kind, band, [bb.min.x, bb.min.z, bb.max.x, bb.max.z], salt);
            cloth.add(&mesh(geo, &lambert!({ map: &tex })));
        }
    }
    cloth.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    cloth.rotation().y = rnd() * PI * 2.0;
    g.world.add(&cloth);
}

/// Terry faded: white gone to cream, peach, blue, coral, sage, grey (sRGB)
const COTTON: [[f64; 3]; 6] =
    [[0.8, 0.78, 0.71], [0.8, 0.66, 0.55], [0.43, 0.53, 0.62], [0.7, 0.45, 0.39], [0.53, 0.59, 0.49], [0.6, 0.6, 0.57]];

/// Fills the canvas pixel by pixel on every core: `f(u, v)` (u right,
/// v up, both 0..1, as the texture is laid) gives straight sRGB and alpha
fn fill(ctx: &mut Canvas, w: f64, h: f64, f: impl Fn(f64, f64) -> [f64; 4] + Sync) {
    let (iw, _) = (w as usize, h as usize);
    let mut img = ctx.create_image_data(w, h);
    par_rows(&mut img.data, iw * 4, |j, out| {
        for i in 0..iw {
            let c = f((i as f64 + 0.5) / w, 1.0 - (j as f64 + 0.5) / h);
            for k in 0..4 {
                out[i * 4 + k] = (c[k].clamp(0.0, 1.0) * 255.0).round() as u8;
            }
        }
    });
    ctx.put_image_data(&img, 0, 0);
}

/// The cloth seen from above (`bb` its part's x and z bounds): the loops
/// of the pile (or the flat band's weave), the dirt it took up from the
/// floor in blotches, mildew in dark clusters, one stain dried with a
/// tide line, grit caught in it
fn tex(kind: usize, band: bool, bb: [f64; 4], salt: u32) -> Tex {
    canvas_tex_opt(256, 192, false, move |ctx, w, h| {
        let seed = salt ^ 0xc107;
        let col = COTTON[kind.min(5)];
        let (sx, sz) = (0.02 * (hash(seed, 1, 1) - 0.5), 0.02 * (hash(seed, 2, 2) - 0.5));
        fill(ctx, w, h, |u, v| {
            let x = bb[0] + u * (bb[2] - bb[0]);
            let z = bb[1] + (1.0 - v) * (bb[3] - bb[1]);
            let mut c = col;
            if band {
                let weave = 0.5 + 0.5 * (x * 2400.0).sin() * (z * 2400.0).cos();
                c = mix(c, [col[0] * 0.86, col[1] * 0.86, col[2] * 0.86], 0.3 + 0.25 * weave);
            } else {
                // the loops: a texel's worth each, light tops and dark between
                let loopy = hash(seed ^ 3, (x * 1300.0) as i32, (z * 1300.0) as i32);
                let clump = vn(seed ^ 4, x * 500.0, z * 500.0);
                c = mix(
                    [col[0] * 0.7, col[1] * 0.7, col[2] * 0.7],
                    [(col[0] * 1.08).min(1.0), (col[1] * 1.08).min(1.0), (col[2] * 1.08).min(1.0)],
                    loopy * 0.6 + clump * 0.4,
                );
            }
            // dirt taken up from the floor, greyer where it lay
            let dirt = fbm(seed ^ 5, x * 18.0, z * 18.0, 4);
            c = mix(c, [0.36, 0.33, 0.28], 0.18 + (dirt - 0.45).max(0.0) * 1.2);
            // mildew: clusters of dark specks
            let cluster = sstep(0.58, 0.72, fbm(seed ^ 6, x * 9.0, z * 9.0, 3));
            let speck = (vn(seed ^ 7, x * 700.0, z * 700.0) - 0.7).max(0.0) * 3.3;
            c = mix(c, [0.16, 0.17, 0.14], (cluster * speck).min(1.0) * 0.85);
            // a stain: something brown dried in it, darker at its edge
            let d = ((x - 0.03 - sx) / 0.055).hypot((z + 0.05 - sz) / 0.04) + 0.8 * (fbm(seed ^ 8, x * 14.0, z * 14.0, 4) - 0.5);
            c = mix(c, [0.5, 0.4, 0.28], sstep(1.0, 0.6, d) * 0.28 * (0.6 + 0.4 * fbm(seed ^ 10, x * 60.0, z * 60.0, 2)));
            c = mix(c, [0.4, 0.3, 0.2], (-((d - 0.92) / 0.08).powi(2)).exp() * 0.22);
            opaque(c)
        });
    })
}
