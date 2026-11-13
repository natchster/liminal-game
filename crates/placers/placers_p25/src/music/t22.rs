//! Batch `t22`'s maracas, made real (`docs/design/PROPS.md` P3, "toys stay toys";
//! the Blender batch `assets/src/blender/props/t22.py`, its notes
//! `t22.notes.md`): a child's pair of turned beech maracas, their heads
//! painted, faded and chipped. It keeps the old placer's draws, call for
//! call.
use crate::{open_tiles, pick};
use game::prelude::*;

/// A maraca end to end (`t22.py` `MAR_L`), metres: its middle is where the
/// game turns it
const MAR_L: f64 = 0.133;

/// Small maracas: 8% chance per level. A child's pair of maracas
/// (133 mm, turned beech: a hollow head of 45 mm painted in one lacquer
/// colour, a bare handle and knob), dropped side by side: lying on the
/// head and the knob as a maraca lies, the paint faded, chipped to the
/// wood where it knocked the floor, the handles greyed where hands held
/// them. Latin-percussion child-music family.
pub fn place_small_maracas(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let bulb_colors = [0xd82060, 0x2050d8, 0xd8b820, 0x30a030, 0xa020a0];
    let bulb_color = pick(&bulb_colors);
    // the old colour: the head's lacquer (faded: red, blue, yellow, green,
    // purple)
    let paint = match bulb_colors.iter().position(|&c| c == bulb_color).unwrap_or(0) {
        0 => [0.64, 0.17, 0.20],
        1 => [0.18, 0.30, 0.60],
        2 => [0.84, 0.68, 0.22],
        3 => [0.24, 0.50, 0.26],
        _ => [0.46, 0.21, 0.46],
    };
    let props = game::assets::props::prop_part_as_modelled;
    let (head, handle) = (props("Real__maracaHead"), props("Real__maracaHandle"));
    let head_mat = phong!({ map: &head_tex(paint), specular: 0x2a2a2a, shininess: 30 });
    let wood = lambert!({ map: &beech_tex() });
    let pair = group();
    for i in 0..2 {
        // the old tipped-over draw: whether this one lies the same way as
        // the first (heads together) or head to tail
        let same_way = rnd() < 0.30;
        let along = (rnd() - 0.5) * 0.02;
        let across = (rnd() - 0.5) * 0.02;
        let turn = rnd();
        let maraca = group();
        for (geo, mat) in [(&head, &head_mat), (&handle, &wood)] {
            if let Some(geo) = geo {
                let m = mesh(geo.clone(), mat);
                m.position().x = -MAR_L / 2.0;
                maraca.add(&m);
            }
        }
        // side by side, a little askew (the old turn's draw)
        let flip = i == 1 && !same_way;
        maraca.position().set(along * 2.0, 0.0, if i == 0 { -0.052 } else { 0.052 } + across);
        maraca.rotation().y = (turn - 0.5) * 0.16 + if flip { PI } else { 0.0 };
        pair.add(&maraca);
    }
    pair.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    pair.rotation().y = rnd() * PI * 2.0;
    g.world.add(&pair);
}

/// The head's lacquer (laid by its own box): one colour, faded and uneven
/// where the light reached it, chipped to the pale beech where it knocked
/// the floor (at its widest), grime on its underside
fn head_tex(paint: [f64; 3]) -> Tex {
    canvas_tex(128, 128, move |ctx, w, h| {
        fill(ctx, w, h, |u, v| {
            let n = fbm(434, u * 6.0, v * 6.0, 3);
            let c = mix(paint, [paint[0] * 0.75, paint[1] * 0.75, paint[2] * 0.75], n);
            let c = mix(c, [0.60, 0.57, 0.52], 0.15 + 0.12 * n);
            // chips where it knocked the floor: a few ragged ones round its widest
            let ragged = 0.75 * vn(435, u * 9.0, v * 9.0) + 0.25 * vn(438, u * 40.0, v * 40.0);
            let chip = ragged * (0.55 + 0.45 * (1.0 - (v - 0.5).abs() * 2.0));
            let c = if chip > 0.70 { mix(c, [0.76, 0.64, 0.48], sstep(0.70, 0.74, chip)) } else { c };
            let c = mix(c, [0.28, 0.24, 0.20], 0.35 * (1.0 - sstep(0.0, 0.3, v)));
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// Bare beech (laid by its own box, the grain along x): pale, its grain
/// fine, gone grey with handling and dust
fn beech_tex() -> Tex {
    thread_local! { static T: std::cell::OnceCell<Tex> = const { std::cell::OnceCell::new() }; }
    T.with(|c| {
        c.get_or_init(|| {
            canvas_tex(128, 64, |ctx, w, h| {
                fill(ctx, w, h, |u, v| {
                    let grain = vn(436, u * 3.0, v * 40.0);
                    let fleck = vn(437, u * 60.0, v * 60.0);
                    let c = mix([0.74, 0.62, 0.46], [0.60, 0.48, 0.34], grain);
                    let c = if fleck > 0.85 { mix(c, [0.52, 0.40, 0.28], (fleck - 0.85) * 5.0) } else { c };
                    let c = mix(c, [0.50, 0.47, 0.42], 0.25);
                    [c[0], c[1], c[2], 1.0]
                });
            })
        })
        .clone()
    })
}

fn hash(seed: u32, i: i32, j: i32) -> f64 {
    let mut h = seed ^ (i as u32).wrapping_mul(0x27d4_eb2d) ^ (j as u32).wrapping_mul(0x1656_67b1);
    h = (h ^ (h >> 15)).wrapping_mul(0x85eb_ca6b);
    h = (h ^ (h >> 13)).wrapping_mul(0xc2b2_ae35);
    h ^= h >> 16;
    f64::from(h) / 4_294_967_296.0
}
fn vn(seed: u32, x: f64, y: f64) -> f64 {
    let (xf, yf) = (x.floor(), y.floor());
    let (i, j) = (xf as i32, yf as i32);
    let (u, v) = (x - xf, y - yf);
    let (u, v) = (u * u * (3.0 - 2.0 * u), v * v * (3.0 - 2.0 * v));
    let (a, b) = (hash(seed, i, j), hash(seed, i + 1, j));
    let (c, d) = (hash(seed, i, j + 1), hash(seed, i + 1, j + 1));
    a + (b - a) * u + (c - a) * v + (a - b - c + d) * u * v
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
fn sstep(e0: f64, e1: f64, x: f64) -> f64 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}
fn mix(a: [f64; 3], b: [f64; 3], t: f64) -> [f64; 3] {
    let t = t.clamp(0.0, 1.0);
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
}
/// Fills the canvas pixel by pixel on every core: `f(u, v)` (u right, v up,
/// both 0..1, as the texture is laid) gives straight sRGB and alpha
fn fill(ctx: &mut Canvas, w: f64, h: f64, f: impl Fn(f64, f64) -> [f64; 4] + Sync) {
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
