//! Batch `t22`'s candy corn, made real (`docs/design/PROPS.md` P3, "the party and
//! the shrine"; the Blender batch `assets/src/blender/props/t22.py`, its
//! notes `t22.notes.md`): a handful of candy corn spilled from a bag and
//! left. It keeps the old placer's draws, call for call.
use crate::{open_tiles, pick_tile};
use game::prelude::*;
use std::cell::OnceCell;

/// A piece from its base to its tip (`t22.py` `CORN_L`), metres
const CORN_L: f64 = 0.019;
/// The pieces (`Real__candyCornA..C`): whole, its tip snapped off, gone
/// soft and slumped
const CORNS: [&str; 3] = ["Real__candyCornA", "Real__candyCornB", "Real__candyCornC"];

// Candy corn scatter: 8% chance per level. 8-14 pieces of candy corn
// spilled from a bag (19 mm, three bands of a sugar sweet, yellow at its
// base, orange, a white tip, glazed with confectioner's wax), each lying
// on a broad face; years on, a few stale and bloomed white and dusty, a
// tip snapped off, one gone soft in some summer's heat and slumped.
// Halloween family.
pub fn place_candy_corn_scatters(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick_tile(&open);
    let Some(kit) = corn_kit() else { return };
    let grp = group();
    let n_pieces = 8 + (rnd() * 7.0).floor() as i32;
    // Two by two, a piece and the next are the same (the odd last one too):
    // none is the only one of its kind, so every piece is drawn instanced,
    // as the old ones were
    let mut kind: Option<(usize, bool)> = None;
    for i in 0..n_pieces {
        let cx = (rnd() - 0.5) * 0.30;
        let cz = (rnd() - 0.5) * 0.25;
        let yaw = rnd() * PI * 2.0;
        // which piece and how stale: from where the first of two fell (no draw)
        if (i % 2 == 0 && i + 1 < n_pieces) || kind.is_none() {
            let s = salt(cx * 100.0, cz * 100.0, 465);
            let k = match s % 9 {
                0 => 1,
                1 => 2,
                _ => 0,
            };
            kind = Some((k, (s >> 5) % 4 == 0));
        }
        let (k, stale) = kind.unwrap_or((0, false));
        let candy = mesh(kit.corns[k].clone(), &kit.mats[usize::from(stale)]);
        candy.position().set(cx, 0.0, cz);
        candy.rotation().y = yaw;
        grp.add(&candy);
    }
    grp.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    grp.rotation().y = rnd() * PI * 2.0;
    g.world.add(&grp);
}

/// What every piece shares: the three pieces (their bands laid along each)
/// and the two looks (still glazed, stale)
struct CornKit {
    corns: Vec<Geo>,
    mats: [Mat; 2],
}

fn corn_kit() -> Option<std::rc::Rc<CornKit>> {
    thread_local! { static K: OnceCell<Option<std::rc::Rc<CornKit>>> = const { OnceCell::new() }; }
    K.with(|c| {
        c.get_or_init(|| {
            let mut corns = vec![];
            for name in CORNS {
                corns.push(along_x(name, CORN_L)?);
            }
            Some(std::rc::Rc::new(CornKit {
                corns,
                mats: [phong!({ map: &corn_tex(false), specular: 0x262420, shininess: 34 }), lambert!({ map: &corn_tex(true) })],
            }))
        })
        .clone()
    })
}

/// Its bands along it from the base (u) and across its face (v): the
/// yellow base, the orange, the white tip, each boundary a little wavy as
/// the layers were poured; faded, the floor's grime underneath; `stale`
/// bloomed white and dull, dusty
fn corn_tex(stale: bool) -> Tex {
    canvas_tex_opt(128, 32, false, move |ctx, w, h| {
        fill(ctx, w, h, |u, v| {
            let n = fbm(3 + u32::from(stale), u * 9.0, v * 4.0, 3);
            let b1 = 0.46 + 0.03 * (vn(7, v * 5.0, 0.0) - 0.5);
            let b2 = 0.79 + 0.03 * (vn(9, v * 5.0, 1.0) - 0.5);
            let yellow = [0.90, 0.68, 0.16];
            let orange = [0.86, 0.42, 0.12];
            let white = [0.93, 0.89, 0.80];
            let c = if u < b1 {
                mix(yellow, orange, sstep(b1 - 0.02, b1 + 0.01, u))
            } else if u < b2 {
                mix(orange, white, sstep(b2 - 0.02, b2 + 0.01, u))
            } else {
                white
            };
            let c = mix(c, [c[0] * 0.82, c[1] * 0.80, c[2] * 0.78], n);
            let c = if stale { mix(mix(c, [0.86, 0.82, 0.72], 0.32 + 0.2 * n), [0.50, 0.47, 0.42], 0.15) } else { c };
            // the side it lies on: the floor's grime
            let c = mix(c, [0.32, 0.27, 0.20], 0.35 * (1.0 - sstep(0.0, 0.35, v)));
            [c[0], c[1], c[2], 1.0]
        });
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
/// A salt from where a thing is (no draw)
fn salt(x: f64, z: f64, k: u32) -> u32 {
    ((x * 7919.0).abs() as u32) ^ ((z * 104_729.0).abs() as u32).rotate_left(13) ^ k.wrapping_mul(0x9e37_79b9)
}
/// The part `name` with its texture laid along x: u from its least x (the
/// base) over `span` metres, v up it (0 on the floor). A copy of the part,
/// made once and shared
fn along_x(name: &str, span: f64) -> Option<Geo> {
    let geo = game::assets::props::prop_part_as_modelled(name)?.duplicate();
    {
        let mut gg = geo.g();
        let n = gg.position.len() / 3;
        let (mut lo, mut hi) = ([f32::MAX; 3], [f32::MIN; 3]);
        for i in 0..n {
            for k in 0..3 {
                lo[k] = lo[k].min(gg.position[i * 3 + k]);
                hi[k] = hi[k].max(gg.position[i * 3 + k]);
            }
        }
        let ry = f64::from(hi[1] - lo[1]).max(1e-6);
        let mut uv = vec![0f32; n * 2];
        for i in 0..n {
            let x = f64::from(gg.position[i * 3] - lo[0]);
            let y = f64::from(gg.position[i * 3 + 1] - lo[1]);
            uv[i * 2] = (x / span) as f32;
            uv[i * 2 + 1] = (y / ry).clamp(0.0, 1.0) as f32;
        }
        gg.uv = uv;
        gg.touch();
    }
    Some(geo)
}
