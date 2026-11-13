//! Batch `t04`'s props in this crate, made real (`docs/design/PROPS.md` P3): the
//! towel bar with its towel and the fire alarm's strobe; their placers and
//! what only they use (`assets/src/blender/props/t04.py`, `t04.notes.md`).
use super::*;
use game::assets::props::prop_part_as_modelled;

use placers_kit::geometry::put as t04_put;

use placers_kit::place::vary as t04_vary;

use placers_kit::noise::hash as t04_hash;

use placers_kit::noise::vn as t04_vn;

use placers_kit::noise::fbm as t04_fbm;

use placers_kit::noise::sstep as t04_ss;

use placers_kit::paint::fill as t04_fill;

// ============================================================ towel bar

// Wall towel bar: 6% chance per level. A 12 in towel bar of the hardware
// store's kind (Blender batch `t04`): a 3/4 in chromed tube on two posts and
// their rosettes, the plating dulled; over it a cotton terry hand towel
// folded in half along its length, hung and left (draped by Blender's
// cloth), gone grey and stiff, its dobby border across each end, mildew
// specked low down where it stayed damp. Its colour the old one's: cream,
// pink, blue or green, faded.
pub(super) fn place_hand_towel_bars(g: &mut Game, _dist: &[i32]) {
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
    t04_put(&o, "Real__towelBarMetal", &phong!({ map: &tb_chrome_tex(vary), color: 0x8c8e90, specular: 0x6a6c6e, shininess: 64 }));
    t04_put(&o, "Real__towelBarTowel", &lambert!({ map: &tb_terry_tex(vary, kind) }));
    g.world.add(&bar);
}

/// The bar's chrome after years in a washroom's damp: dulled, a bloom of
/// tarnish, pits (a grey the chrome's colour multiplies)
fn tb_chrome_tex(vary: u32) -> Tex {
    canvas_tex_opt(64, 64, false, move |ctx, w, h| {
        let seed = vary ^ 0x7b1;
        t04_fill(ctx, w, h, |u, v| {
            let n = t04_fbm(seed, u * 5.0, v * 5.0, 3);
            let tarnish = t04_ss(0.55, 0.78, t04_fbm(seed ^ 0x9, u * 3.0, v * 3.0, 3)) * 0.3;
            let pit = if t04_hash(seed ^ 0x2, (u * 64.0) as i32, (v * 64.0) as i32) < 0.015 { 0.4 } else { 0.0 };
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
        t04_fill(ctx, w, h, |u, v| {
            // in metres from the towel's ends (its lowest point at v 0)
            let y = v * TW_SPAN;
            let x = u * 0.224;
            // the pile: loops and the pits between them
            let loops = t04_vn(seed, x * 900.0, y * 820.0);
            let pits = t04_vn(seed ^ 0x51, x * 1600.0 + 0.5, y * 1500.0);
            let mut shade = 0.86 + 0.16 * loops - 0.12 * t04_ss(0.62, 0.9, pits);
            // the dobby border: a flat weave band, ribbed across
            let bd = (y - 0.055).abs();
            let border = 1.0 - t04_ss(0.016, 0.018, bd);
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
                shade = 0.88 + 0.06 * rib - 0.12 * (1.0 - t04_ss(0.0, 0.004, y)) - 0.10 * (-((y - 0.012) / 0.0008).powi(2)).exp();
            }
            // the years: grey through, dingier and darker low down where
            // hands were dried on it, blotched
            let n = t04_fbm(seed ^ 0x23, x * 14.0, y * 14.0, 4);
            let low = 1.0 - t04_ss(0.0, 0.22, y);
            let dirt = 0.10 + 0.14 * n + 0.18 * low * (0.5 + n);
            let grey = [0.50, 0.48, 0.44];
            let mut c = [0.0; 3];
            for k in 0..3 {
                let b = base[k] * 0.78 + grey[k] * 0.22;
                c[k] = b * shade * (1.0 - dirt);
            }
            // a tide line where it dried after a wetting
            let tide = t04_fbm(seed ^ 0x61, x * 9.0, y * 9.0, 3);
            let ring = (-((tide - 0.62) / 0.012).powi(2)).exp() * (1.0 - t04_ss(0.1, 0.25, y)) * 0.25;
            for (k, t) in c.iter_mut().zip([0.45, 0.38, 0.28]) {
                *k = *k * (1.0 - ring) + t * ring * 0.8;
            }
            // mildew: grey-black specks in clusters, where it stayed damp
            let cl = t04_ss(0.58, 0.72, t04_fbm(seed ^ 0x3a, x * 20.0, y * 20.0, 3)) * (0.3 + low);
            if t04_hash(seed ^ 0x3b, (u * 224.0) as i32, (v * 336.0) as i32) < 0.10 * cl {
                for (k, m) in c.iter_mut().zip([0.16, 0.17, 0.15]) {
                    *k = *k * 0.35 + m * 0.65;
                }
            }
            [c[0], c[1], c[2], 1.0]
        });
    })
}

// ============================================================ the strobe

/// The strobe's lens, its middle (x, y) and front (`t04.py` ST_*); where
/// FIRE is printed under it
const ST_LENS_FRONT: f64 = 0.0296;
const ST_FIRE: [f64; 3] = [0.0, -0.031, 0.0213];

// Emergency strobe: 3% chance per level. A fire alarm's wall strobe of
// the compact kind (Blender batch `t04`, the Edwards Genesis G1RF-VM): a red
// moulded housing faded and dusty, FIRE in white under a clear prismatic
// lens, the xenon tube and its reflector behind it. Now and then it fires:
// a white flash that lights the room (the old light, its timing and its
// draws), the tube and the lens lit with it.
pub(super) fn place_emergency_strobes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    } // 3% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = choice(&candidates);
    let strobe = group();
    // the parts are modelled with the wall at z = 0: the group's origin is
    // 0.5 mm off it
    let o = group();
    o.position().z = -0.0005;
    strobe.add(&o);
    t04_put(&o, "Real__strobeHousing", &lambert!({ map: &st_housing_tex() }));
    let reflector = phong!({ color: 0xb4b4b0, specular: 0x5a5a58, shininess: 40, side: DoubleSide });
    t04_put(&o, "Real__strobeReflector", &reflector);
    let tube_m = lambert!({ color: 0xc8c8c4 });
    tube_m.set_emissive(0xeef0ff);
    tube_m.set_emissive_intensity(0.0);
    t04_put(&o, "Real__strobeTube", &tube_m);
    let lens_m = phong!({ color: 0xd4d8d6, specular: 0x5a5a5a, shininess: 90, transparent: true, opacity: 0.32, depthWrite: false });
    lens_m.set_emissive(0xeef0ff);
    lens_m.set_emissive_intensity(0.0);
    if let Some(l) = t04_put(&o, "Real__strobeLens", &lens_m) {
        l.set_render_order(2);
    }
    let print = mesh(
        plane_geo!(0.046, 0.023),
        lambert!({ map: &st_fire_tex(), transparent: true, depthWrite: false, polygonOffset: true, polygonOffsetFactor: -2, polygonOffsetUnits: -2 }),
    );
    print.position().set(ST_FIRE[0], ST_FIRE[1], ST_FIRE[2]);
    o.add(&print);
    // the flash: a xenon tube's white (the old light's strength and reach)
    let flash = point_light!(0xf2f4ff, 0.0, 1.5, 2.0);
    flash.position().set(0, 0.026, ST_LENS_FRONT + 0.004);
    strobe.add(&flash);
    // Flash animator (occasional bright pop).
    let mut next_flash = 8.0 + rnd() * 20.0;
    {
        let flash = flash.clone();
        let (tube_m, lens_m) = (tube_m.clone(), lens_m.clone());
        g.animate(move |_g, _dt, t| {
            if t >= next_flash {
                // Bright pop.
                flash.set_intensity(1.5);
                next_flash = t + 4.0 + rnd() * 8.0;
            } else {
                // Rapid decay.
                let v = flash.intensity() * 0.85;
                flash.set_intensity(v);
            }
            // the tube and the lens lit with it
            let k = flash.intensity() / 1.5;
            tube_m.set_emissive_intensity(k * 2.5);
            lens_m.set_emissive_intensity(k * 0.5);
        });
    }
    // group origin = the plate's back (local z 0): 0.5 mm off the wall
    let (wall_x, wall_z) = wall_xz(g, c, 0.0005);
    mount(&strobe, c, wall_x, wall_z, 2.30, Some(0.6));
    g.world.add(&strobe);
}

/// The housing's red (the part's box faces: its front laid across u, v up):
/// moulded plastic faded paler toward its top, dust settled on it, a scuff
fn st_housing_tex() -> Tex {
    canvas_tex_opt(48, 80, false, move |ctx, w, h| {
        let seed = 0x5707_e11u32;
        t04_fill(ctx, w, h, |u, v| {
            let n = t04_fbm(seed, u * 4.0, v * 6.0, 3);
            let fade = t04_ss(0.3, 1.0, v) * 0.10;
            let dust = (0.06 + 0.10 * t04_ss(0.7, 1.0, v)) * (0.6 + 0.8 * n);
            let red = [0.55 + 0.05 * n + fade, 0.13 + fade * 0.6, 0.11 + fade * 0.5];
            let d = [0.55, 0.52, 0.46];
            [red[0] * (1.0 - dust) + d[0] * dust, red[1] * (1.0 - dust) + d[1] * dust, red[2] * (1.0 - dust) + d[2] * dust, 1.0]
        });
    })
}

/// FIRE in white under the lens, as the makers print it, worn a little
fn st_fire_tex() -> Tex {
    canvas_tex_opt(128, 64, false, |ctx, w, h| {
        ctx.clear_rect(0, 0, w, h);
        ctx.set_fill_style("#e6e2d8");
        ctx.set_text_align("center");
        ctx.set_text_baseline("middle");
        ctx.set_font("bold 46px Helvetica Neue");
        ctx.fill_text_max("FIRE", w / 2.0, h / 2.0 + 2.0, w - 8.0);
        ctx.set_global_composite_operation("destination-out");
        for _ in 0..30 {
            let (x, y) = (rnd() * w, rnd() * h);
            ctx.set_global_alpha(0.2 + 0.4 * rnd());
            ctx.begin_path();
            ctx.ellipse(x, y, 0.8 + rnd() * 2.0, 0.5 + rnd(), rnd() * 3.0, 0, 7);
            ctx.fill();
        }
        ctx.set_global_alpha(1.0);
        ctx.set_global_composite_operation("source-over");
    })
}
