//! Batch `t15`'s dog toys, made real (`docs/PROPS.md` P3, "toys stay toys";
//! `assets/src/blender/props/t15.py`, `t15.notes.md`). Their placer and what
//! only it uses; it keeps the old placer's draws, call for call.
use crate::open_tiles;
use game::assets::props::prop_part_as_modelled;
use game::prelude::*;

/// R722 pet toy (floor): 7% chance per level. A dog's squeaky bone (hollow
/// moulded vinyl, 15 cm, its squeaker's pip and its mould seam) or its
/// rubber ball (solid natural rubber, 76 mm, its seam, some with a band of
/// a second colour); chewed: tooth dents and holes, scraped, grimed where
/// the mouth held it, dusty.
pub(crate) fn place_pet_toys(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.93 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let pt = group();
    let is_bone = rnd() < 0.55;
    let (base, band) = if is_bone {
        // the old bone colours, as vinyl yellowed
        let bone_colors = [0xf0e0c0, 0xd0c0a0, 0xf0d8b0, 0xe0d0a0];
        let c = choice(&bone_colors);
        let k = bone_colors.iter().position(|&b| b == c).unwrap_or(0);
        // the old tooth marks (their count, places and tilts) are chewed
        // into the vinyl now: their draws kept
        let mark_count = 2 + (rnd() * 2.0).floor() as i32;
        for _ in 0..mark_count {
            let _ = (rnd(), rnd(), rnd());
        }
        // the old roll: it lies flat on its knobs (the draw kept)
        let _ = rnd();
        ([0xc8b89a, 0xa89a80, 0xc6ae8e, 0xb8a884][k], false)
    } else {
        // the old ball colours, as rubber faded
        let ball_colors = [0xa02040, 0x2060a0, 0xf0d040, 0x40a030, 0xf07020, 0xa040a0];
        let c = choice(&ball_colors);
        let k = ball_colors.iter().position(|&b| b == c).unwrap_or(0);
        // the old bite marks (their count and places) are pocked into the
        // rubber now: their draws kept
        let mark_count = 2 + (rnd() * 3.0).floor() as i32;
        for _ in 0..mark_count {
            let _ = (rnd(), rnd());
        }
        // the old white stripe: a band moulded round it in off-white
        let band = rnd() < 0.4;
        ([0x7c3a3e, 0x3a5470, 0xa8924a, 0x4e6a3c, 0x9c5e34, 0x6a466a][k], band)
    };
    let px = g.wx(x) + (rnd() - 0.5) * 2.4;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.4;
    pt.position().set(px, 0, pz);
    pt.rotation().y = rnd() * PI * 2.0;
    let salt = salt_at(px, pz);
    let b = rgb(base);
    if is_bone {
        let tex = paint_tile(128, move |u, v| {
            let w = Wear { base: b, alt: mix(b, [0.35, 0.3, 0.24], 0.5), rub: 0.08, grime: 0.5, scuff: 0.7, mottle: 0.08, scales: 0 };
            let mut c = wear_at(salt ^ 91, u, v, &w);
            // tooth scrapes: short dark nicks
            let n = (vnp(salt ^ 93, u * 30.0, v * 8.0, 30, 8) - 0.5).abs();
            c = mix(c, mul(b, 0.45), (1.0 - ss(0.0, 0.012, n)) * ss(0.45, 0.6, fbmp(salt ^ 95, u, v, 4, 2)));
            c
        });
        let mat = phong!({ map: &tex, specular: 0x262626, shininess: 26 });
        put_world(&pt, "Real__petBone", &mat, 0.06);
        put_world(&pt, "Real__petBoneSeam", &mat, 0.06);
    } else {
        let w = Wear { base: b, alt: mix(b, [0.8, 0.78, 0.72], 0.4), rub: 0.05, grime: 0.5, scuff: 0.6, mottle: 0.1, scales: 0 };
        put_world(&pt, "Real__petBall", &phong!({ map: &wear_tex(salt ^ 97, 128, w), specular: 0x161616, shininess: 10 }), 0.06);
        if band {
            let off = rgb(0xc4bdae);
            let w = Wear { base: off, alt: mix(off, [0.4, 0.36, 0.3], 0.5), rub: 0.06, grime: 0.5, scuff: 0.5, mottle: 0.1, scales: 0 };
            put_world(&pt, "Real__petBallBand", &phong!({ map: &wear_tex(salt ^ 98, 64, w), specular: 0x161616, shininess: 10 }), 0.06);
        }
    }
    g.world.add(&pt);
}
// ------------------------------------------------------------- the paint ---

/// Part `name` of batch `t15` as modelled, added to `grp` in `mat`
fn put(grp: &Obj, name: &str, mat: &Mat) {
    if let Some(geo) = prop_part_as_modelled(name) {
        grp.add(&mesh(geo, mat.clone()));
    }
}

/// Part `name` with its texture laid on at its real scale (`size` m a
/// repeat), from the side each vertex faces most: a worn surface the same
/// grain on every part of the thing
fn put_world(grp: &Obj, name: &str, mat: &Mat, size: f64) {
    let Some(geo) = prop_part_as_modelled(name) else { return };
    let geo = geo.duplicate();
    {
        let mut gg = geo.g();
        let n = gg.position.len() / 3;
        let mut uv = Vec::with_capacity(n * 2);
        for i in 0..n {
            let (x, y, z) = (f64::from(gg.position[i * 3]), f64::from(gg.position[i * 3 + 1]), f64::from(gg.position[i * 3 + 2]));
            let (ax, ay, az) = (gg.normal[i * 3].abs(), gg.normal[i * 3 + 1].abs(), gg.normal[i * 3 + 2].abs());
            let (u, v) = if ay >= ax && ay >= az {
                (x, z)
            } else if ax >= az {
                (z, y)
            } else {
                (x, y)
            };
            uv.push((u / size) as f32);
            uv.push((v / size) as f32);
        }
        gg.uv = uv;
        gg.touch();
    }
    grp.add(&mesh(geo, mat.clone()));
}

/// A salt from where the thing lies (no draw): its wear differs from floor
/// to floor
fn salt_at(x: f64, z: f64) -> u32 {
    ((x * 7919.0).abs() as u32) ^ ((z * 104_729.0).abs() as u32).rotate_left(13) ^ 0x7f4a_7c15
}

fn rgb(c: u32) -> [f64; 3] {
    [f64::from((c >> 16) & 255) / 255.0, f64::from((c >> 8) & 255) / 255.0, f64::from(c & 255) / 255.0]
}

fn ss(e0: f64, e1: f64, x: f64) -> f64 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn mix(a: [f64; 3], b: [f64; 3], t: f64) -> [f64; 3] {
    let t = t.clamp(0.0, 1.0);
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
}

fn mul(a: [f64; 3], k: f64) -> [f64; 3] {
    [a[0] * k, a[1] * k, a[2] * k]
}

fn hash(seed: u32, i: i32, j: i32) -> f64 {
    let mut h = seed ^ (i as u32).wrapping_mul(0x27d4_eb2d) ^ (j as u32).wrapping_mul(0x1656_67b1);
    h = (h ^ (h >> 15)).wrapping_mul(0x85eb_ca6b);
    h = (h ^ (h >> 13)).wrapping_mul(0xc2b2_ae35);
    h ^= h >> 16;
    f64::from(h) / 4_294_967_296.0
}

/// Value noise over a lattice repeating every (px, py) cells
fn vnp(seed: u32, x: f64, y: f64, px: i32, py: i32) -> f64 {
    let (xf, yf) = (x.floor(), y.floor());
    let (i, j) = (xf as i32, yf as i32);
    let (u, v) = (x - xf, y - yf);
    let (u, v) = (u * u * (3.0 - 2.0 * u), v * v * (3.0 - 2.0 * v));
    let (i0, i1, j0, j1) = (i.rem_euclid(px), (i + 1).rem_euclid(px), j.rem_euclid(py), (j + 1).rem_euclid(py));
    let (a, b) = (hash(seed, i0, j0), hash(seed, i1, j0));
    let (c, d) = (hash(seed, i0, j1), hash(seed, i1, j1));
    a + (b - a) * u + (c - a) * v + (a - b - c + d) * u * v
}

/// Fractal noise over the tile (u, v in 0..1), `p` cells across at first
fn fbmp(seed: u32, u: f64, v: f64, p: i32, oct: u32) -> f64 {
    let (mut s, mut amp, mut n) = (0.0, 0.5, 0.0);
    for k in 0..oct {
        let q = p << k;
        s += amp * vnp(seed.wrapping_add(k * 7919), u * f64::from(q), v * f64::from(q), q, q);
        n += amp;
        amp *= 0.5;
    }
    s / n
}

/// The distance between the nearest and next nearest of the tile's cell
/// points (`n` cells across, repeating): small along the cells' edges
fn cell_edge(seed: u32, u: f64, v: f64, n: i32) -> f64 {
    let (x, y) = (u * f64::from(n), v * f64::from(n));
    let (i, j) = (x.floor() as i32, y.floor() as i32);
    let (mut d1, mut d2) = (9.0f64, 9.0f64);
    for dj in -1..=1 {
        for di in -1..=1 {
            let (ci, cj) = (i + di, j + dj);
            let (wi, wj) = (ci.rem_euclid(n), cj.rem_euclid(n));
            let fx = f64::from(ci) + 0.15 + 0.7 * hash(seed, wi, wj);
            let fy = f64::from(cj) + 0.15 + 0.7 * hash(seed ^ 0x5bd1, wi, wj);
            let d = ((x - fx).powi(2) + (y - fy).powi(2)).sqrt();
            if d < d1 {
                d2 = d1;
                d1 = d;
            } else if d < d2 {
                d2 = d;
            }
        }
    }
    d2 - d1
}

/// How a moulded or painted thing has worn on a floor for years
#[derive(Clone, Copy)]
struct Wear {
    /// its colour, and what shows where it is rubbed through (the plastic
    /// under paint, or plastic whitened by stress)
    base: [f64; 3],
    alt: [f64; 3],
    /// how much of it is rubbed through, grimed, scuffed, mottled (faded
    /// unevenly); its sculpted scales (cells across a tile; 0 none)
    rub: f64,
    grime: f64,
    scuff: f64,
    mottle: f64,
    scales: i32,
}

/// The worn surface at (u, v) of a repeating tile
fn wear_at(seed: u32, u: f64, v: f64, w: &Wear) -> [f64; 3] {
    let m = fbmp(seed, u, v, 3, 4);
    let mut c = mul(w.base, 1.0 + w.mottle * (m - 0.5) * 2.4);
    if w.scales > 0 {
        let e = cell_edge(seed ^ 0x51ab, u, v, w.scales);
        c = mul(c, 1.0 - 0.2 * (1.0 - ss(0.0, 0.14, e)));
    }
    if w.rub > 0.0 {
        // rubbed through in small patches: `rub` is about the share of the
        // surface (fractal noise sits near 0.5, its spread about 0.12)
        let r = fbmp(seed ^ 0x2b3c, u, v, 7, 4);
        let t = 0.72 - 0.6 * w.rub;
        c = mix(c, w.alt, ss(t, t + 0.025, r));
    }
    // grime settled in soft patches
    let gm = fbmp(seed ^ 0x77e1, u, v, 3, 4);
    c = mix(c, [0.15, 0.13, 0.10], w.grime * 0.7 * ss(0.5, 0.8, gm));
    if w.scuff > 0.0 {
        // fine scratches in patches: thin lines along a stretched noise
        let patch = ss(0.5, 0.7, fbmp(seed ^ 0x9137, u, v, 3, 2));
        let l1 = (vnp(seed ^ 0x4421, u * 40.0, v * 5.0, 40, 5) - 0.5).abs();
        let l2 = (vnp(seed ^ 0x1d77, u * 6.0, v * 34.0, 6, 34) - 0.5).abs();
        let line = (1.0 - ss(0.0, 0.025, l1)).max(1.0 - ss(0.0, 0.025, l2));
        c = mix(c, mix(w.base, [0.86, 0.84, 0.8], 0.45), w.scuff * patch * line * 0.8);
    }
    // dust in the grain, and grit
    let d = vnp(seed ^ 0xd057, u * 90.0, v * 90.0, 90, 90);
    if d > 0.8 {
        c = mix(c, [0.55, 0.52, 0.47], (d - 0.8) * 2.2);
    }
    c
}

/// A repeating `px` x `px` tile painted pixel by pixel by `f(u, v)`
fn paint_tile<F: Fn(f64, f64) -> [f64; 3] + Sync>(px: u32, f: F) -> Tex {
    canvas_tex_opt(px, px, true, move |ctx, w, h| {
        let w_ = w as usize;
        let mut img = ctx.create_image_data(w, h);
        par_rows(&mut img.data, w_ * 4, |j, out| {
            for i in 0..w_ {
                let c = f((i as f64 + 0.5) / w, (j as f64 + 0.5) / h);
                for k in 0..3 {
                    out[i * 4 + k] = (c[k].clamp(0.0, 1.0) * 255.0).round() as u8;
                }
                out[i * 4 + 3] = 255;
            }
        });
        ctx.put_image_data(&img, 0, 0);
    })
}

/// A worn surface's tile
fn wear_tex(seed: u32, px: u32, w: Wear) -> Tex {
    paint_tile(px, move |u, v| wear_at(seed, u, v, &w))
}
