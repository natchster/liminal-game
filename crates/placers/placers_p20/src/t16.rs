//! Batch `t16`'s prop in this crate, made real (`docs/design/PROPS.md` P3, "toys
//! stay toys"): the deflated ball, a size 3 football or a size 3 rubber
//! basketball gone soft on a floor (`assets/src/blender/props/t16.py`,
//! `t16.notes.md`). Its placer and what only it uses; it keeps the old
//! placer's draws, call for call.
use super::{open_tiles, scatter};
use game::assets::props::prop_part_as_modelled;
use game::prelude::*;

/// Deflated ball: 7% chance per level. A ball gone soft (Blender batch
/// `t16`): half the time a size 3 football (19 cm; 32 machine-stitched PVC
/// panels, the black pentagons and white hexagons puffed between their
/// seams), else a size 3 rubber basketball (17.8 cm; pebbled, its eight
/// panels parted by black channels). It sags on the floor, its bottom
/// spread flat, its top caved in to a dish with creases from the fold. The
/// white yellowed, the orange gone brown, scuffed, grimed (more for one
/// that lay longer: the old patch count).
pub(super) fn place_deflated_balls(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.93 {
        return;
    } // 7% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = choice(&open);
    let is_soccer = rnd() < 0.5;
    // the old patch count (3 or 4): how long it lay, how grimy
    let n_patches = 3 + (rnd() * 2.0).floor() as i32;
    let grime = if n_patches > 3 { 0.75 } else { 0.5 };
    let ball = group();
    scatter(g, &ball, x, z, 2.0);
    let p = ball.pos();
    let salt = t16_salt(p.x, p.z);
    if is_soccer {
        let white = pvc_tex(salt, grime);
        t16_put(&ball, "Real__deflatedFootballWhite", &phong!({ map: &white, specular: 0x1c1c1c, shininess: 22 }));
        t16_put(&ball, "Real__deflatedFootballBlack", &phong!({ color: 0x8a8682, map: &white, specular: 0x141414, shininess: 18 }));
    } else {
        let skin = pebble_tex(salt, grime);
        t16_put(&ball, "Real__deflatedBasketballSkin", &lambert!({ map: &skin }));
        t16_put(&ball, "Real__deflatedBasketballLines", &lambert!({ color: 0x24211f, side: DoubleSide }));
    }
    g.world.add(&ball);
}

/// The football's PVC (its white panels; the black ones are this times a
/// near black): yellowed, scuffed grey where it was kicked and rolled,
/// grime in patches, dust
fn pvc_tex(salt: u32, grime: f64) -> Tex {
    t16_paint(256, 256, move |u, v| {
        let mut c = [0.83, 0.80, 0.72];
        let yel = t16_fbm(salt, u * 4.0, v * 4.0, 3);
        c = t16_mix(c, [0.76, 0.70, 0.55], 0.5 * t16_ss(0.4, 0.8, yel));
        let scuff = t16_ss(0.62, 0.78, t16_fbm(salt ^ 0x27, u * 14.0, v * 40.0, 3));
        c = t16_mix(c, [0.55, 0.53, 0.50], 0.6 * scuff);
        let dirt = t16_ss(0.45, 0.85, t16_fbm(salt ^ 0x49, u * 6.0, v * 6.0, 3));
        c = t16_mix(c, [0.40, 0.37, 0.33], grime * 0.7 * dirt);
        let speck = t16_vn(salt ^ 0x55, u * 140.0, v * 140.0);
        t16_mix(c, [0.30, 0.28, 0.26], 0.4 * t16_ss(0.82, 0.95, speck))
    })
}

/// The basketball's rubber: pebbled (the gaps between its pebbles darker),
/// its orange gone brown, rubbed smooth and pale in patches, grimed
fn pebble_tex(salt: u32, grime: f64) -> Tex {
    t16_paint(256, 256, move |u, v| {
        let mut c = [0.60, 0.35, 0.22];
        c = t16_mix(c, [0.50, 0.33, 0.24], t16_ss(0.4, 0.8, t16_fbm(salt, u * 4.0, v * 4.0, 3)));
        let peb = t16_vn(salt ^ 0x13, u * 110.0, v * 110.0);
        let worn = t16_ss(0.55, 0.75, t16_fbm(salt ^ 0x31, u * 7.0, v * 7.0, 3));
        c = t16_mix(c, [c[0] * 0.72, c[1] * 0.70, c[2] * 0.68], (1.0 - t16_ss(0.25, 0.45, peb)) * (1.0 - worn));
        c = t16_mix(c, [0.62, 0.52, 0.44], 0.35 * worn);
        let dirt = t16_ss(0.45, 0.85, t16_fbm(salt ^ 0x49, u * 6.0, v * 6.0, 3));
        t16_mix(c, [0.30, 0.26, 0.22], grime * 0.6 * dirt)
    })
}

// ---- what only these use ----

use placers_kit::place::salt_of as t16_salt;

/// The part `name` of batch `t16` as modelled, added to `grp` in `mat`
fn t16_put(grp: &Obj, name: &str, mat: &Mat) {
    if let Some(geo) = prop_part_as_modelled(name) {
        grp.add(&mesh(geo, mat.clone()));
    }
}

/// A texture painted pixel by pixel (sRGB) from `f(u, v)` (v down the
/// image), on every core: no canvas call and no draw inside
fn t16_paint(w: usize, h: usize, f: impl Fn(f64, f64) -> [f64; 3] + Sync + Send + 'static) -> Tex {
    canvas_tex_opt(w as f64, h as f64, false, move |ctx, cw, ch| {
        let mut img = ctx.create_image_data(cw, ch);
        par_rows(&mut img.data, w * 4, |j, out| {
            for i in 0..w {
                let c = f((i as f64 + 0.5) / w as f64, (j as f64 + 0.5) / h as f64);
                for k in 0..3 {
                    out[i * 4 + k] = (c[k].clamp(0.0, 1.0) * 255.0).round() as u8;
                }
                out[i * 4 + 3] = 255;
            }
        });
        ctx.put_image_data(&img, 0, 0);
    })
}

use placers_kit::noise::sstep as t16_ss;

use placers_kit::noise::mix as t16_mix;

use placers_kit::noise::hash as t16_hash;

use placers_kit::noise::vn as t16_vn;

fn t16_fbm(seed: u32, x: f64, y: f64, oct: u32) -> f64 {
    let (mut s, mut amp, mut f, mut n) = (0.0, 0.5, 1.0, 0.0);
    for k in 0..oct {
        s += amp * t16_vn(seed.wrapping_add(k * 7919), x * f, y * f);
        n += amp;
        amp *= 0.5;
        f *= 2.03;
    }
    s / n
}
