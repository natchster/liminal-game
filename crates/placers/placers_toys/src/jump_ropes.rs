//! `place_jump_ropes`: Jump rope: 5% chance per level. A child's skipping rope (Blender family `a_play_areas_toys`): 2.3 m of 8 mm braided cotton between two turned beech handles (120 x 25 mm), wound round a hand once and dropped: three slack loops come apart and cross each other, the handles out of the coil side by side. The dye faded and grimed, the lacquer of the handles worn to grey wood where they were held.
use game::assets::props::prop_part_as_modelled;
use game::prelude::*;
use placers_kit::noise::mix;
use placers_kit::noise::sstep;
use placers_kit::noise::vn;
use placers_kit::place::jitter;
use placers_kit::place::pick_open;
use placers_kit::place::salt_of;

/// Jump rope: 5% chance per level. A child's skipping rope (Blender
/// family `a_play_areas_toys`): 2.3 m of 8 mm braided cotton between two turned beech
/// handles (120 x 25 mm), wound round a hand once and dropped: three slack
/// loops come apart and cross each other, the handles out of the coil side
/// by side. The dye faded and grimed, the lacquer of the handles worn to
/// grey wood where they were held.
pub(crate) fn place_jump_ropes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    // the old rope colour: the cotton's dye, faded
    let rope_colors = [0xc04020, 0x2060c0, 0x40a040, 0xf0e020, 0xd050a0];
    let old = choice(&rope_colors);
    let k = rope_colors.iter().position(|&c| c == old).unwrap_or(0);
    let dye = [[0.55, 0.30, 0.24], [0.30, 0.37, 0.50], [0.36, 0.45, 0.31], [0.70, 0.62, 0.38], [0.64, 0.44, 0.49]][k];
    let rope = group();
    let (px, pz) = jitter(g, x, z, 2.4);
    let salt = salt_of(px, pz);
    put(&rope, "Real__jumpRopeCord", &lambert!({ map: &cotton_tex(dye, salt) }));
    put(&rope, "Real__jumpRopeHandles", &lambert!({ map: &beech_tex(salt) }));
    rope.position().set(px, 0, pz);
    rope.rotation().y = rnd() * PI * 2.0;
    g.world.add(&rope);
}

/// The cotton, laid on the cord from above (its bounds): the dye mottled
/// where it faded and where it was wet, the braid's fine light and dark,
/// grey grime and floor dust in patches
fn cotton_tex(dye: [f64; 3], salt: u32) -> Tex {
    paint(128, 128, move |u, v| {
        let n = fbm(salt, u * 7.0, v * 7.0, 3);
        let braid = vn(salt ^ 0x51, u * 160.0, v * 160.0);
        let mut c = mix(dye, [0.62, 0.60, 0.55], 0.25 * sstep(0.45, 0.75, n));
        c = mix(c, [0.0, 0.0, 0.0], 0.10 + 0.18 * braid);
        let grime = sstep(0.5, 0.8, fbm(salt ^ 0x77, u * 4.0, v * 4.0, 3));
        mix(c, [0.30, 0.28, 0.25], 0.45 * grime)
    })
}

/// Beech under a worn lacquer: its fine grain, grey where hands wore the
/// lacquer off, dark grime in the turnings
fn beech_tex(salt: u32) -> Tex {
    paint(128, 128, move |u, v| {
        let grain = fbm(salt ^ 0x3c, u * 6.0, v * 90.0, 3);
        let mut c = mix([0.58, 0.47, 0.36], [0.47, 0.37, 0.27], grain);
        let worn = sstep(0.55, 0.75, fbm(salt ^ 0x19, u * 5.0, v * 5.0, 3));
        c = mix(c, [0.55, 0.52, 0.47], 0.55 * worn);
        mix(c, [0.24, 0.20, 0.16], 0.35 * sstep(0.6, 0.85, fbm(salt ^ 0x2b, u * 12.0, v * 12.0, 2)))
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
