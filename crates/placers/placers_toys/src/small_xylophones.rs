//! `place_small_xylophones`: Small xylophone: 6% chance per level. A child's eight note "xylophone" (Blender family `a_play_areas_toys`): a metallophone of 3 mm aluminium bars enamelled in eight colours, C to C (100 to 68 mm), each pinned through its two nodes over a felt strip on a splayed pine frame (265 x 75 mm), its note's letter printed at its foot; a wooden mallet lying in front. The enamel faded, chipped at the corners and worn to the metal where the mallet struck, the pine grey with handling, dust.
use game::assets::props::prop_part_as_modelled;
use game::prelude::*;
use placers_kit::noise::mix;
use placers_kit::noise::sstep;
use placers_kit::noise::vn;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;
use placers_kit::place::salt_of;

/// Small xylophone: 6% chance per level. A child's eight note
/// "xylophone" (Blender family `a_play_areas_toys`): a metallophone of 3 mm aluminium bars
/// enamelled in eight colours, C to C (100 to 68 mm), each pinned through
/// its two nodes over a felt strip on a splayed pine frame (265 x 75 mm),
/// its note's letter printed at its foot; a wooden mallet lying in front.
/// The enamel faded, chipped at the corners and worn to the metal where
/// the mallet struck, the pine grey with handling, dust.
pub(crate) fn place_small_xylophones(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let xylo = group();
    let px = g.wx(x) + (rnd() - 0.5) * 2.0;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.0;
    let salt = salt_of(px, pz);
    put(&xylo, "Real__xyloFrame", &lambert!({ map: &pine_tex(salt) }));
    put(&xylo, "Real__xyloFelt", &lambert!({ color: 0x4a2a28 }));
    if let Some(geo) = prop_part_as_modelled("Real__xyloBars") {
        // the enamel laid on from above, every face (the bars' sides take
        // their own bar's colour)
        let geo = geo.duplicate();
        {
            let mut gg = geo.g();
            let n = gg.position.len() / 3;
            for i in 0..n {
                let (bx, bz) = (f64::from(gg.position[i * 3]), f64::from(gg.position[i * 3 + 2]));
                gg.uv[i * 2] = ((bx - X0) / (X1 - X0)) as f32;
                gg.uv[i * 2 + 1] = (1.0 - (bz - Z0) / (Z1 - Z0)) as f32;
            }
            gg.touch();
        }
        xylo.add(&mesh(geo, phong!({ map: &enamel_tex(salt), specular: 0x2a2a2a, shininess: 30 })));
    }
    put(&xylo, "Real__xyloPins", &phong!({ color: 0x6a6a68, specular: 0x303030, shininess: 30 }));
    put(&xylo, "Real__xyloMallet", &lambert!({ color: 0x96785a }));
    put(&xylo, "Real__xyloMalletHead", &phong!({ color: 0x3a2e26, specular: 0x1e1e1e, shininess: 20 }));
    xylo.position().set(px, 0, pz);
    xylo.rotation().y = rnd() * PI * 2.0;
    g.world.add(&xylo);
}

/// The bars as built (`toys/a_play_areas_toys.py` XY_*): eight, 28 mm apart along x from the
/// low C, 22 mm wide, 100 mm stepping down to 68 mm long across z
const N_BARS: usize = 8;

const PITCH: f64 = 0.028;

const BAR_W: f64 = 0.022;

/// The bars' bounds in plan (x, z), over which their enamel is laid
const X0: f64 = -0.109;

const X1: f64 = 0.109;

const Z0: f64 = -0.050;

const Z1: f64 = 0.050;

fn bar_len(i: usize) -> f64 {
    0.100 - i as f64 * (0.032 / (N_BARS - 1) as f64)
}

fn bar_x(i: usize) -> f64 {
    -PITCH * (N_BARS - 1) as f64 / 2.0 + i as f64 * PITCH
}

/// The bars' enamel laid on from above (u along x, v along z): each bar its
/// colour, faded; chipped to the aluminium round its edges and corners, a
/// patch worn to the metal where the mallet struck it, scratches, grime
/// round its pins; its note's letter printed white at its foot, rubbed
fn enamel_tex(salt: u32) -> Tex {
    const COLS: [[f64; 3]; N_BARS] = [
        [0.62, 0.24, 0.21],
        [0.70, 0.42, 0.22],
        [0.76, 0.64, 0.30],
        [0.32, 0.50, 0.32],
        [0.28, 0.48, 0.58],
        [0.25, 0.31, 0.54],
        [0.42, 0.30, 0.50],
        [0.66, 0.38, 0.46],
    ];
    let (w, h) = (512usize, 256usize);
    canvas_tex_opt(w as f64, h as f64, false, move |ctx, cw, ch| {
        let mut img = ctx.create_image_data(cw, ch);
        par_rows(&mut img.data, w * 4, |j, out| {
            for i in 0..w {
                let (u, v) = ((i as f64 + 0.5) / w as f64, (j as f64 + 0.5) / h as f64);
                let (x, z) = (X0 + (X1 - X0) * u, Z0 + (Z1 - Z0) * v);
                let k = (((x - bar_x(0)) / PITCH).round().max(0.0) as usize).min(N_BARS - 1);
                let (dx, dz) = ((x - bar_x(k)).abs(), z.abs());
                // how far in from the bar's edge (m)
                let edge = (BAR_W / 2.0 - dx).min(bar_len(k) / 2.0 - dz);
                let metal = mix([0.66, 0.67, 0.68], [0.52, 0.53, 0.54], fbm(salt ^ 0x15, u * 30.0, v * 15.0, 2));
                let mut c = mix(COLS[k], [0.70, 0.68, 0.64], 0.15 * sstep(0.45, 0.8, fbm(salt, u * 8.0, v * 4.0, 3)));
                let chip_n = fbm(salt ^ 0x25, u * 60.0, v * 30.0, 3);
                let corner = sstep(0.006, 0.0, (BAR_W / 2.0 - dx).max(0.0).hypot((bar_len(k) / 2.0 - dz).max(0.0)) - 0.002);
                let chip = sstep(0.0035, 0.0005, edge).max(corner) * sstep(0.45, 0.6, chip_n);
                // where the mallet struck: the middle of the bar, a little to the player
                let (sx, sz) = ((x - bar_x(k)) / 0.008, (z - 0.004) / 0.011);
                let strike = (-(sx * sx + sz * sz)).exp() * sstep(0.3, 0.6, fbm(salt ^ 0x35, u * 90.0, v * 45.0, 2));
                c = mix(c, metal, chip.max(strike * 0.85));
                let scratch = (fbm(salt ^ 0x45, u * 4.0, v * 70.0, 2) - 0.5).abs();
                c = mix(c, [0.78, 0.77, 0.74], 0.25 * (1.0 - sstep(0.0, 0.006, scratch)));
                // grime round the pins and dust
                let pin = (x - bar_x(k)).hypot(dz - bar_len(k) * (0.5 - 0.224));
                c = mix(c, [0.22, 0.20, 0.18], 0.5 * sstep(0.006, 0.0025, pin));
                let dust = sstep(0.5, 0.85, fbm(salt ^ 0x55, u * 12.0, v * 6.0, 3));
                c = mix(c, [0.55, 0.53, 0.50], 0.2 * dust);
                for q in 0..3 {
                    out[i * 4 + q] = (c[q].clamp(0.0, 1.0) * 255.0).round() as u8;
                }
                out[i * 4 + 3] = 255;
            }
        });
        ctx.put_image_data(&img, 0, 0);
        // the notes' letters at the bars' feet (+z), printed and rubbed
        ctx.set_font("bold 15px sans-serif");
        ctx.set_text_align("center");
        for (k, l) in ["C", "D", "E", "F", "G", "A", "B", "C"].iter().enumerate() {
            let px = (bar_x(k) - X0) / (X1 - X0) * cw;
            let pz = (bar_len(k) / 2.0 - 0.006 - Z0) / (Z1 - Z0) * ch;
            ctx.set_fill_style("rgba(226,222,210,0.72)");
            ctx.fill_text(l, px, pz);
        }
    })
}

/// The pine of the frame laid on its bounds: its grain along the rails,
/// greyed and darkened with handling, dust
fn pine_tex(salt: u32) -> Tex {
    paint(256, 64, move |u, v| {
        let grain = fbm(salt ^ 0x65, u * 3.0, v * 40.0, 3);
        let mut c = mix([0.66, 0.53, 0.36], [0.55, 0.42, 0.28], grain);
        c = mix(c, [0.52, 0.48, 0.42], 0.35 * sstep(0.4, 0.8, fbm(salt ^ 0x75, u * 6.0, v * 3.0, 3)));
        mix(c, [0.30, 0.26, 0.22], 0.3 * sstep(0.6, 0.9, fbm(salt ^ 0x85, u * 14.0, v * 8.0, 2)))
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
