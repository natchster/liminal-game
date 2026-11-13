//! Batch `t20`'s music box, made real (`docs/design/PROPS.md` P3; the Blender
//! batch `assets/src/blender/props/t20.py`, its notes `t20.notes.md`): a
//! Swiss child's hand-crank music box of about 1900, its lid open on the
//! works. It keeps the old placer's draws, call for call.
use crate::{pick_color, pick_open_tile, scatter_pos};
use game::prelude::*;
use game::visuals::pictures::{self, Age};
use t20_kit::*;

/// The box (t20.py `MB_*`): its depth, the case's height, the lid's
/// opening (radians), the print inside the lid (m)
const MB_D: f64 = 0.070;
const MB_HB: f64 = 0.032;
const MB_OPEN: f64 = 1.80;
const MB_PRINT: (f64, f64) = (0.088, 0.058);

// Small children's music box: 6% chance per level. A Swiss child's
// hand-crank music box (Blender batch `t20`): a little box of beech,
// stained walnut, mahogany or dark oak, or painted red or yellow ochre (the
// old colour's draw); its lid open, a chromolithograph pasted inside it;
// under a glass the works on their brass bedplate, the pinned cylinder,
// the steel comb, the wheel; the crank on its side with a porcelain knob.
// The stain and paint worn at its corners, the print faded, dust on the
// glass.
pub fn place_small_music_boxes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let Some((x, z)) = pick_open_tile(g) else { return };
    // the old draws, call for call: its finish, where it stands, its turn
    let box_colors = [0xa06030, 0x8a5030, 0x603820, 0xd82060, 0xd8b040];
    let finish = pick_color(&box_colors);
    let bx = group();
    scatter_pos(g, &bx, x, z, 2.4);
    bx.rotation().y = rnd() * PI * 2.0;
    let p = bx.pos();
    let vary = vary(p.x, p.z, bx.rot().y);
    let wood = phong!({ map: &mb_finish_tex(vary, finish), specular: 0x1a1814, shininess: 18 });
    put(&bx, "Real__musicBoxCase", &wood);
    let brass = phong!({ color: 0x6e5832, specular: 0x5a4a2a, shininess: 40 });
    put(&bx, "Real__musicBoxBrass", &brass);
    put(&bx, "Real__musicBoxComb", &phong!({ color: 0x4a4a4c, specular: 0x505058, shininess: 60 }));
    if let Some(m) = put(&bx, "Real__musicBoxGlass", &pictures::glass_mat(vary, 0.6)) {
        m.set_render_order(2);
    }
    put(&bx, "Real__musicBoxKnob", &phong!({ color: 0xc8c0ae, specular: 0x3a3a36, shininess: 50 }));
    // the lid open on its hinges, the print inside it
    let lid = group();
    lid.position().set(0, MB_HB, -MB_D * 0.5);
    lid.rotation().x = -MB_OPEN;
    bx.add(&lid);
    put(&lid, "Real__musicBoxLid", &wood);
    let pics = ["bot_rose", "land_pond", "bot_poppy", "garden_family"];
    let pic = pics[(hash(vary, 5, 5) * pics.len() as f64) as usize % pics.len()];
    put(&lid, "Real__musicBoxPaper", &lambert!({ map: &pictures::print_tex(pic, MB_PRINT, 2400.0, Age::engraving(vary), vary) }));
    g.world.add(&bx);
}

/// The box's finish as its faces are mapped, for the old five colours:
/// beech stained walnut, mahogany or dark oak, or painted (red, yellow
/// ochre) over the wood; worn to the bare beech at its corners and edges,
/// grimed where it was held, dust
fn mb_finish_tex(vary: u32, c: u32) -> Tex {
    let (base, painted) = match c {
        0xa06030 => ([0.36, 0.22, 0.12], false),
        0x8a5030 => ([0.30, 0.14, 0.08], false),
        0x603820 => ([0.17, 0.10, 0.06], false),
        0xd82060 => ([0.48, 0.14, 0.12], true),
        _ => ([0.62, 0.46, 0.20], true),
    };
    canvas_tex_opt(128, 128, false, move |ctx, w, h| {
        let seed = vary ^ 0x3b0;
        fill(ctx, w, h, |u, v| {
            let mut o = if painted {
                let n = fbm(seed, u * 8.0, v * 8.0, 3);
                base.map(|c| c * (0.9 + 0.15 * n))
            } else {
                wood(seed, u, v, base, 10.0, 40.0)
            };
            // worn to the beech at the edges and the corners
            let e = 1.0 - ss(0.02, 0.10, u.min(1.0 - u).min(v).min(1.0 - v));
            let wear = e * ss(0.5, 0.7, fbm(seed ^ 3, u * 20.0, v * 20.0, 3));
            o = mix(o, [0.58, 0.45, 0.30], wear * if painted { 0.8 } else { 0.4 });
            let dust = 0.08 + 0.12 * fbm(seed ^ 5, u * 6.0, v * 6.0, 3);
            opaque(mix(o, [0.43, 0.40, 0.35], dust))
        });
    })
}

/// Batch `t20`'s painting kit (each of its modules has its own copy): the
/// parts placed as modelled, a value of where a thing was put to vary its
/// painters, value noise, and a painter that fills a canvas pixel by pixel
/// on every core (no canvas call and no draw inside the loop)
#[allow(dead_code)]
mod t20_kit {
    use game::assets::props::prop_part_as_modelled;
    use game::prelude::*;

    /// The Blender part `name`, as modelled, in `grp` with `mat`
    pub fn put(grp: &Obj, name: &str, mat: &Mat) -> Option<Obj> {
        let m = mesh(prop_part_as_modelled(name)?, mat);
        grp.add(&m);
        Some(m)
    }

    /// Something of where a thing was put, to vary what its painters paint
    /// (a painter's own stream starts the same on every floor)
    pub fn vary(x: f64, z: f64, k: f64) -> u32 {
        let mut h: u64 = 0xcbf2_9ce4_8422_2325;
        for v in [x, z, k] {
            h ^= v.to_bits();
            h = h.wrapping_mul(0x0100_0000_01b3);
        }
        (h ^ (h >> 29) ^ (h >> 47)) as u32
    }

    pub fn hash(seed: u32, i: i32, j: i32) -> f64 {
        let mut h = seed ^ (i as u32).wrapping_mul(0x27d4_eb2d) ^ (j as u32).wrapping_mul(0x1656_67b1);
        h = (h ^ (h >> 15)).wrapping_mul(0x85eb_ca6b);
        h = (h ^ (h >> 13)).wrapping_mul(0xc2b2_ae35);
        h ^= h >> 16;
        f64::from(h) / 4_294_967_296.0
    }

    pub fn vn(seed: u32, x: f64, y: f64) -> f64 {
        let (xf, yf) = (x.floor(), y.floor());
        let (i, j) = (xf as i32, yf as i32);
        let (u, v) = (x - xf, y - yf);
        let (u, v) = (u * u * (3.0 - 2.0 * u), v * v * (3.0 - 2.0 * v));
        let (a, b) = (hash(seed, i, j), hash(seed, i + 1, j));
        let (c, d) = (hash(seed, i, j + 1), hash(seed, i + 1, j + 1));
        a + (b - a) * u + (c - a) * v + (a - b - c + d) * u * v
    }

    pub fn fbm(seed: u32, x: f64, y: f64, oct: u32) -> f64 {
        let (mut s, mut amp, mut f, mut n) = (0.0, 0.5, 1.0, 0.0);
        for k in 0..oct {
            s += amp * vn(seed.wrapping_add(k * 7919), x * f, y * f);
            n += amp;
            amp *= 0.5;
            f *= 2.03;
        }
        s / n
    }

    pub fn ss(e0: f64, e1: f64, x: f64) -> f64 {
        let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    }

    pub fn mix(a: [f64; 3], b: [f64; 3], t: f64) -> [f64; 3] {
        [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
    }

    pub fn opaque(c: [f64; 3]) -> [f64; 4] {
        [c[0], c[1], c[2], 1.0]
    }

    /// Fills the canvas pixel by pixel on every core: `f(u, v)` (u right,
    /// v up, both 0..1) gives straight sRGB and alpha, 0..1
    pub fn fill(ctx: &mut Canvas, w: f64, h: f64, f: impl Fn(f64, f64) -> [f64; 4] + Sync) {
        let iw = w as usize;
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

    /// The distance to the nearest edge of a cell of a Voronoi field (cells
    /// of size 1): cracks in old leather, varnish or enamel
    pub fn crack(seed: u32, x: f64, y: f64) -> f64 {
        let (cx, cy) = (x.floor() as i32, y.floor() as i32);
        let (mut d1, mut d2) = (9.0f64, 9.0f64);
        for dy in -1..=1 {
            for dx in -1..=1 {
                let (ix, iy) = (cx + dx, cy + dy);
                let px = f64::from(ix) + 0.1 + 0.8 * hash(seed, ix, iy);
                let py = f64::from(iy) + 0.1 + 0.8 * hash(seed ^ 0x55, ix, iy);
                let d = ((x - px).powi(2) + (y - py).powi(2)).sqrt();
                if d < d1 {
                    d2 = d1;
                    d1 = d;
                } else if d < d2 {
                    d2 = d;
                }
            }
        }
        (d2 - d1) * 0.5
    }

    /// Wood seen along its grain (u along it): the growth rings' streaks and
    /// the figure, `base` its colour (sRGB)
    pub fn wood(seed: u32, u: f64, v: f64, base: [f64; 3], along: f64, across: f64) -> [f64; 3] {
        let warp = fbm(seed ^ 0x1d, u * along * 0.15, v * across * 0.05, 3);
        let ring = ((v * across + warp * 6.0).sin() * 0.5 + 0.5).powf(3.0);
        let streak = fbm(seed, u * along * 0.2, v * across, 3);
        let fine = vn(seed ^ 3, u * along * 3.0, v * across * 6.0);
        base.map(|c| c * (0.82 + 0.28 * streak + 0.06 * fine - 0.14 * ring))
    }

    /// Tarnished metal's mottle (a multiplier near 1)
    pub fn tarnish(seed: u32, u: f64, v: f64, scale: f64) -> f64 {
        0.8 + 0.3 * fbm(seed, u * scale, v * scale, 3) - 0.18 * ss(0.6, 0.8, fbm(seed ^ 9, u * scale * 0.6, v * scale * 0.6, 3))
    }
}
