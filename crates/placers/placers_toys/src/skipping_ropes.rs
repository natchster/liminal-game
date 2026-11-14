//! `place_skipping_ropes`: Skipping rope: 8% chance per level. A PVC "licorice" skipping rope (Blender family `a_play_areas_toys`): 2.4 m of 4.8 mm solid PVC through two moulded polypropylene handles (114 mm), dropped with its handles together: the rope springs out in one loop that crossed itself. The PVC faded and dulled, grey with dust; the handles scuffed.
use game::assets::props::prop_part_as_modelled;
use game::prelude::*;
use placers_kit::noise::mix;
use placers_kit::noise::sstep;
use placers_kit::noise::vn;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;
use placers_kit::place::salt_of;

/// Skipping rope: 8% chance per level. A PVC "licorice" skipping rope
/// (Blender family `a_play_areas_toys`): 2.4 m of 4.8 mm solid PVC through two moulded
/// polypropylene handles (114 mm), dropped with its handles together: the
/// rope springs out in one loop that crossed itself. The PVC faded and
/// dulled, grey with dust; the handles scuffed.
pub(crate) fn place_skipping_ropes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    // the old rope colour: the PVC's, faded; its handles moulded to match
    let rope_colors = [0xd82020, 0x2050d8, 0xd8b820, 0xf0f0f0];
    let old = pick(&rope_colors);
    let k = rope_colors.iter().position(|&c| c == old).unwrap_or(0);
    let pvc = [0x8a3c34, 0x3c5084, 0xae9a4c, 0xb8b4a8][k];
    let handles = [0x8e7c4a, 0x7a4642, 0x44506c, 0x7a4642][k];
    let rope = group();
    let px = g.wx(x) + (rnd() - 0.5) * 2.2;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.2;
    let salt = salt_of(px, pz);
    let dust = dust_tex(salt);
    put(&rope, "Real__licoriceRopeCord", &phong!({ color: pvc, map: &dust, specular: 0x262626, shininess: 34 }));
    put(&rope, "Real__licoriceRopeHandles", &phong!({ color: handles, map: &dust, specular: 0x1c1c1c, shininess: 22 }));
    rope.position().set(px, 0, pz);
    rope.rotation().y = rnd() * PI * 2.0;
    g.world.add(&rope);
}

/// Dust and grime over the plastic, laid on from above (a multiplier of
/// the plastic's colour): grey film in patches, darker where it lay in dirt
fn dust_tex(salt: u32) -> Tex {
    paint(128, 128, move |u, v| {
        let film = sstep(0.4, 0.75, fbm(salt, u * 6.0, v * 6.0, 3));
        let dirt = sstep(0.6, 0.85, fbm(salt ^ 0x5d, u * 11.0, v * 11.0, 2));
        let speck = vn(salt ^ 0x33, u * 120.0, v * 120.0);
        let c = mix([0.92, 0.91, 0.90], [1.0, 0.99, 0.97], film);
        let c = mix(c, [0.70, 0.68, 0.65], 0.6 * dirt);
        mix(c, [0.80, 0.79, 0.77], 0.3 * sstep(0.7, 0.95, speck))
    })
}

/// The part `name` of family `a_play_areas_toys` as modelled, added to `grp` in `mat`
fn put(grp: &Obj, name: &str, mat: &Mat) {
    if let Some(geo) = prop_part_as_modelled(name) {
        grp.add(&mesh(geo, mat.clone()));
    }
}

/// A texture painted pixel by pixel (sRGB) from `f(u, v)` (v down the
/// image), on every core: no canvas call and no draw inside
fn paint(w: usize, h: usize, f: impl Fn(f64, f64) -> [f64; 3] + Sync + Send + 'static) -> Tex {
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
