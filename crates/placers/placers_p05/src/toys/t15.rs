//! Batch `t15`'s toys in this crate, made real (`docs/PROPS.md` P3, "toys
//! stay toys"; `assets/src/blender/props/t15.py`, `t15.notes.md`): the
//! jointed soldier figure, the PVC stegosaurus and brontosaurus, the baby
//! rattle. Their placers and what only they use; each keeps the old
//! placer's draws, call for call (the comments at each say what a draw now
//! chooses).
use crate::{open_tiles, pick};
use game::assets::props::prop_part_as_modelled;
use game::prelude::*;

/// The index of `c` in `cs` (the old colour a draw picked)
fn index_of(cs: &[u32], c: u32) -> usize {
    cs.iter().position(|&x| x == c).unwrap_or(0)
}

/// Plastic action figure: 4% chance per level. A jointed 3 3/4 in
/// (95 mm) soldier of the 1980s kind, no maker's: its O-ring body moulded
/// in the uniform's plastic, the head and hands in flesh plastic, boots,
/// belt and pouches painted black and rubbed through to the plastic, a
/// painted steel-pot helmet; standing on its boots, or fallen on its back.
/// Faded, scuffed, grimed.
pub fn place_action_figures(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    // the old body colours stand for the uniform's plastic: a flight suit,
    // olive drab, a faded red, grey, desert tan
    let body_colors: [u32; 5] = [0x2060c0, 0x40a040, 0xc02020, 0x606060, 0xd0a040];
    let k = index_of(&body_colors, pick(&body_colors));
    let uniform = [0x3c4a60, 0x545a38, 0x70413a, 0x6a6a64, 0x98865e][k];
    let helmet = [0x4c5040, 0x464c30, 0x4c5040, 0x55574e, 0x8c7a56][k];
    // 60% fallen on its back, else standing (the old draw)
    let pose = if rnd() < 0.60 { "Lie" } else { "Stand" };
    let px = g.wx(x) + (rnd() - 0.5) * 2.4;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.4;
    let turn = rnd() * PI * 2.0;
    let salt = salt_at(px, pz);
    let flesh = rgb(0xc29a7c);
    let plastic = |base: [f64; 3], alt: [f64; 3], rub: f64, grime: f64, scuff: f64, seed: u32| {
        let w = Wear { base, alt, rub, grime, scuff, mottle: 0.05, scales: 0 };
        phong!({ map: &wear_tex(salt ^ seed, 128, w), specular: 0x161616, shininess: 18 })
    };
    let fig = group();
    let size = 0.03;
    let u = rgb(uniform);
    put_world(&fig, &format!("Real__actionFigureBody{pose}"), &plastic(u, u, 0.0, 0.45, 0.5, 1), size);
    put_world(&fig, &format!("Real__actionFigureSkin{pose}"), &plastic(flesh, flesh, 0.0, 0.5, 0.3, 2), size);
    put_world(&fig, &format!("Real__actionFigureKit{pose}"), &plastic(rgb(0x1c1b19), u, 0.08, 0.2, 0.5, 3), size);
    put_world(&fig, &format!("Real__actionFigureHelmet{pose}"), &plastic(rgb(helmet), flesh, 0.04, 0.4, 0.3, 4), size);
    put(&fig, &format!("Real__actionFigureEyes{pose}"), &lambert!({ color: 0x1e1a17 }));
    fig.position().set(px, 0, pz);
    fig.rotation().y = turn;
    g.world.add(&fig);
}

/// Baby rattle: 6% chance per level. A hard plastic baby rattle of the
/// 1960s, 12 cm: a fluted pastel ball moulded in two halves (the beads loose
/// in it), a cream handle with the ring a small hand holds, a cap over the
/// mould's gate; lying on the ball and the ring. Its pastel faded, a few
/// printed stars rubbed half away, the cream gone yellow, grimed.
pub fn place_baby_rattles(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    // the old pastels, faded
    let rattle_colors: [u32; 4] = [0xf0c0c0, 0xc0c0f0, 0xf0e0a0, 0xa0e0c0];
    let k = index_of(&rattle_colors, pick(&rattle_colors));
    let pastel = rgb([0xc9a29c, 0x9fa4c2, 0xcab98c, 0x8fb8a2][k]);
    let px = g.wx(x) + (rnd() - 0.5) * 2.4;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.4;
    let turn = rnd() * PI * 2.0;
    let salt = salt_at(px, pz);
    let rattle = group();
    let ball = Wear { base: pastel, alt: mix(pastel, [0.92, 0.9, 0.86], 0.4), rub: 0.0, grime: 0.4, scuff: 0.45, mottle: 0.07, scales: 0 };
    let ball_tex = paint_tile(128, move |u, v| {
        let mut c = wear_at(salt ^ 11, u, v, &ball);
        // the printed stars, rubbed half away (two to a tile, 9 mm across)
        for k in 0..2 {
            let (cx, cy) = (0.25 + 0.5 * f64::from(k), 0.3 + 0.4 * f64::from(k));
            let ink = star(u - cx, v - cy, 0.075, 0.4 * f64::from(k)) * ss(0.3, 0.42, fbmp(salt ^ 13, u, v, 6, 3));
            c = mix(c, [0.93, 0.91, 0.86], ink * 0.85);
        }
        c
    });
    put_world(&rattle, "Real__babyRattleBall", &phong!({ map: &ball_tex, specular: 0x262626, shininess: 34 }), 0.06);
    let cream = rgb(0xd2c7aa);
    let handle = Wear { base: cream, alt: mix(cream, [0.6, 0.5, 0.3], 0.3), rub: 0.0, grime: 0.5, scuff: 0.4, mottle: 0.08, scales: 0 };
    put_world(
        &rattle,
        "Real__babyRattleHandle",
        &phong!({ map: &wear_tex(salt ^ 12, 128, handle), specular: 0x222222, shininess: 30 }),
        0.05,
    );
    rattle.position().set(px, 0, pz);
    rattle.rotation().y = turn;
    g.world.add(&rattle);
}

/// Toy dinosaur: 4% chance per level. A solid PVC dinosaur of the
/// 1980s kind, sold loose from a tub: a stegosaurus (13 cm, its plates and
/// tail spikes moulded with it) or a brontosaurus (16 cm, its neck raised);
/// moulded in one colour, its skin sculpted in scales, the eyes dots of
/// paint. Faded, scuffed, grime in its scales.
pub fn place_toy_dinosaurs(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    // the old colours, as PVC faded: green, blue, rust, mustard, purple
    let dino_colors: [u32; 5] = [0x408040, 0x2060c0, 0xa04020, 0xc0a040, 0x804080];
    let k = index_of(&dino_colors, pick(&dino_colors));
    let pvc = rgb([0x5b6a44, 0x46566e, 0x7c4c36, 0x9a8650, 0x5e4a62][k]);
    // the old back plates (50%): a stegosaurus; else a brontosaurus
    let stego = rnd() < 0.5;
    let px = g.wx(x) + (rnd() - 0.5) * 2.4;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.4;
    let turn = rnd() * PI * 2.0;
    let salt = salt_at(px, pz);
    let w = Wear { base: pvc, alt: mix(pvc, [0.85, 0.82, 0.75], 0.3), rub: 0.0, grime: 0.45, scuff: 0.4, mottle: 0.07, scales: 22 };
    let skin = phong!({ map: &wear_tex(salt ^ 21, 256, w), specular: 0x141414, shininess: 14 });
    let eyes = lambert!({ color: 0x1c1915 });
    let dino = group();
    if stego {
        put_world(&dino, "Real__toyStegoBody", &skin, 0.05);
        put_world(&dino, "Real__toyStegoPlates", &skin, 0.05);
        put(&dino, "Real__toyStegoEyes", &eyes);
    } else {
        put_world(&dino, "Real__toyBrontoBody", &skin, 0.05);
        put(&dino, "Real__toyBrontoEyes", &eyes);
    }
    dino.position().set(px, 0, pz);
    dino.rotation().y = turn;
    g.world.add(&dino);
}
/// How much of a five-pointed star (`r` to its points, turned `turn`) covers
/// the point (x, y) from its centre: 1 inside, 0 outside, soft at its edge
fn star(x: f64, y: f64, r: f64, turn: f64) -> f64 {
    let sector = PI * 2.0 / 5.0;
    let d = (x * x + y * y).sqrt();
    let mut phi = (y.atan2(x) + turn).rem_euclid(sector);
    if phi > sector / 2.0 {
        phi = sector - phi;
    }
    // the edge from a point (r, 0) to the notch beside it (0.4 r at 36 degrees)
    let (p0x, p0y) = (r, 0.0);
    let (p1x, p1y) = (0.4 * r * (PI / 5.0).cos(), 0.4 * r * (PI / 5.0).sin());
    let (px, py) = (d * phi.cos(), d * phi.sin());
    let (ex, ey) = (p1x - p0x, p1y - p0y);
    let side = (ex * (py - p0y) - ey * (px - p0x)) / (ex * ex + ey * ey).sqrt();
    ss(-0.004, 0.004, side)
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
        // surface (this fractal noise sits near 0.49, its spread about 0.15)
        let r = fbmp(seed ^ 0x2b3c, u, v, 12, 3);
        let t = 0.8 - 0.9 * w.rub;
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
