//! `place_overturned_tricycles`: Overturned tricycle: 4% chance per level. A toddler's steel tricycle (Blender batch `t16`; 66 x 56 x 54 cm standing: a 10 in front wheel with pedals on its crank axle, 7 in rears, spoked, solid rubber tyres; a bent tube frame, a rear step, a moulded saddle, a plated handlebar and vinyl grips) lying on its side as it came to rest, its steering swung. Built lying (the game turns it only about y). Its enamel faded, chipped to the primer and the steel, rust in the chips and running from them; the plating dull and pitted; the rubber grey.
use game::assets::props::prop_part_as_modelled;
use game::prelude::*;
use placers_kit::noise::mix as t16_mix;
use placers_kit::noise::sstep as t16_ss;
use placers_kit::noise::vn as t16_vn;
use placers_kit::place::open_tiles;
use placers_kit::place::pick_hex;
use placers_kit::place::salt_of as t16_salt;

/// Overturned tricycle: 4% chance per level. A toddler's steel
/// tricycle (Blender batch `t16`; 66 x 56 x 54 cm standing: a 10 in front
/// wheel with pedals on its crank axle, 7 in rears, spoked, solid rubber
/// tyres; a bent tube frame, a rear step, a moulded saddle, a plated
/// handlebar and vinyl grips) lying on its side as it came to rest, its
/// steering swung. Built lying (the game turns it only about y). Its enamel
/// faded, chipped to the primer and the steel, rust in the chips and
/// running from them; the plating dull and pitted; the rubber grey.
pub(crate) fn place_overturned_tricycles(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    // the old frame colour: its enamel, faded
    let frame_colors = [0xd82040, 0x2050d8, 0xd8b040, 0x30a030];
    let old = pick_hex(&frame_colors);
    let k = frame_colors.iter().position(|&c| c == old).unwrap_or(0);
    let enamel = [[0.55, 0.17, 0.15], [0.20, 0.28, 0.48], [0.70, 0.55, 0.24], [0.24, 0.42, 0.26]][k];
    let tri = group();
    let px = g.wx(x) + (rnd() - 0.5) * 1.8;
    let pz = g.wz(z) + (rnd() - 0.5) * 1.8;
    let salt = t16_salt(px, pz);
    t16_put(&tri, "Real__trikeFrame", &phong!({ map: &enamel_tex(enamel, salt), specular: 0x161616, shininess: 16 }));
    t16_put(&tri, "Real__trikePlated", &phong!({ map: &plating_tex(salt), specular: 0x3a3a3a, shininess: 36 }));
    t16_put(&tri, "Real__trikeRubber", &lambert!({ color: 0x2e2b28 }));
    t16_put(&tri, "Real__trikeGrips", &lambert!({ color: 0xa49c86 }));
    t16_put(&tri, "Real__trikeSeat", &phong!({ color: 0x5c2a2a, specular: 0x1a1a1a, shininess: 14 }));
    tri.position().set(px, 0, pz);
    tri.rotation().y = rnd() * PI * 2.0;
    g.world.add(&tri);
}

/// The frame's enamel, laid on its bounds' faces: faded and dulled, chipped
/// at random (small and large flakes) to a grey primer and the steel, rust
/// filling the chips and bleeding round them, a rust run below some, grime
fn enamel_tex(enamel: [f64; 3], salt: u32) -> Tex {
    t16_paint(512, 512, move |u, v| {
        let mut c = t16_mix(enamel, [0.62, 0.58, 0.52], 0.25 * t16_ss(0.4, 0.8, t16_fbm(salt, u * 5.0, v * 5.0, 3)));
        // the chips: flakes of two sizes
        let f1 = t16_fbm(salt ^ 0x1f, u * 26.0, v * 26.0, 3);
        let f2 = t16_fbm(salt ^ 0x2f, u * 70.0, v * 70.0, 2);
        let chip = t16_ss(0.66, 0.68, f1).max(t16_ss(0.74, 0.76, f2));
        let halo = t16_ss(0.60, 0.66, f1).max(t16_ss(0.69, 0.74, f2)) * (1.0 - chip);
        let rust_n = t16_fbm(salt ^ 0x3f, u * 40.0, v * 40.0, 3);
        let rust = t16_mix([0.36, 0.20, 0.11], [0.50, 0.29, 0.14], rust_n);
        let primer = [0.42, 0.41, 0.39];
        let bare = t16_mix(primer, rust, t16_ss(0.35, 0.55, rust_n));
        c = t16_mix(c, [c[0] * 0.75 + 0.06, c[1] * 0.62 + 0.04, c[2] * 0.55 + 0.02], halo * 0.8);
        c = t16_mix(c, bare, chip);
        // rust run down from a chip (v down the faces' height)
        let run = t16_ss(0.70, 0.72, t16_fbm(salt ^ 0x1f, u * 26.0, (v - 0.02) * 26.0, 3)) * (1.0 - chip);
        c = t16_mix(c, [0.42, 0.24, 0.13], run * 0.35);
        // grime and dust
        let dirt = t16_ss(0.5, 0.85, t16_fbm(salt ^ 0x4f, u * 8.0, v * 8.0, 3));
        t16_mix(c, [0.30, 0.27, 0.23], 0.45 * dirt)
    })
}

/// The plating (zinc and chrome on the bar, the rims, the spokes): dull
/// grey, pitted, spotted and bloomed with rust
fn plating_tex(salt: u32) -> Tex {
    t16_paint(256, 256, move |u, v| {
        let mut c = t16_mix([0.62, 0.62, 0.61], [0.48, 0.48, 0.47], t16_fbm(salt ^ 0x5f, u * 6.0, v * 6.0, 3));
        let pit = t16_vn(salt ^ 0x6f, u * 150.0, v * 150.0);
        c = t16_mix(c, [0.30, 0.26, 0.22], t16_ss(0.78, 0.9, pit));
        let bloom = t16_ss(0.58, 0.72, t16_fbm(salt ^ 0x7f, u * 18.0, v * 18.0, 3));
        c = t16_mix(c, [0.44, 0.27, 0.15], 0.85 * bloom);
        let dirt = t16_ss(0.55, 0.85, t16_fbm(salt ^ 0x8f, u * 7.0, v * 7.0, 2));
        t16_mix(c, [0.28, 0.26, 0.23], 0.4 * dirt)
    })
}

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
