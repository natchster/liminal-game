//! Batch `t04`'s prop in this crate, made real (`docs/PROPS.md` P3): the
//! hair dryer dropped on the floor; its placer and what only it uses
//! (`assets/src/blender/props/t04.py`, `t04.notes.md`).
use super::*;
use game::assets::props::prop_part_as_modelled;

/// The Blender part `name`, as modelled, in `grp` with `mat`
fn t04_put(grp: &Obj, name: &str, mat: &Mat) -> Option<Obj> {
    let m = mesh(prop_part_as_modelled(name)?, mat);
    grp.add(&m);
    Some(m)
}

/// Something of where a thing was put, to vary what its painters paint
/// (a painter's own stream starts the same on every floor)
fn t04_vary(x: f64, z: f64, k: f64) -> u32 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for v in [x, z, k] {
        h ^= v.to_bits();
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    (h ^ (h >> 29) ^ (h >> 47)) as u32
}

// ---- painting kit (per pixel, pure: safe for `par_rows`) ----
fn t04_hash(seed: u32, i: i32, j: i32) -> f64 {
    let mut h = seed ^ (i as u32).wrapping_mul(0x27d4_eb2d) ^ (j as u32).wrapping_mul(0x1656_67b1);
    h = (h ^ (h >> 15)).wrapping_mul(0x85eb_ca6b);
    h = (h ^ (h >> 13)).wrapping_mul(0xc2b2_ae35);
    h ^= h >> 16;
    f64::from(h) / 4_294_967_296.0
}

fn t04_vn(seed: u32, x: f64, y: f64) -> f64 {
    let (xf, yf) = (x.floor(), y.floor());
    let (i, j) = (xf as i32, yf as i32);
    let (u, v) = (x - xf, y - yf);
    let (u, v) = (u * u * (3.0 - 2.0 * u), v * v * (3.0 - 2.0 * v));
    let (a, b) = (t04_hash(seed, i, j), t04_hash(seed, i + 1, j));
    let (c, d) = (t04_hash(seed, i, j + 1), t04_hash(seed, i + 1, j + 1));
    a + (b - a) * u + (c - a) * v + (a - b - c + d) * u * v
}

fn t04_fbm(seed: u32, x: f64, y: f64, oct: u32) -> f64 {
    let (mut s, mut amp, mut f, mut n) = (0.0, 0.5, 1.0, 0.0);
    for k in 0..oct {
        s += amp * t04_vn(seed.wrapping_add(k * 7919), x * f, y * f);
        n += amp;
        amp *= 0.5;
        f *= 2.03;
    }
    s / n
}

fn t04_ss(e0: f64, e1: f64, x: f64) -> f64 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Fills the canvas pixel by pixel on every core: `f(u, v)` (u right, v up,
/// both 0..1, as the texture is laid) gives straight sRGB and alpha, 0..1
fn t04_fill(ctx: &mut Canvas, w: f64, h: f64, f: impl Fn(f64, f64) -> [f64; 4] + Sync) {
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

/// A round flex along `pts` (its cross-section `r` across, `r * flat`
/// high): a tube with its frames carried along the path, its ends capped
fn t04_cord_geo(pts: &[[f64; 3]], r: f64, flat: f64, seg: usize) -> Geo {
    let n = pts.len();
    let sub = |a: [f64; 3], b: [f64; 3]| [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
    let norm = |a: [f64; 3]| {
        let l = (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt().max(1e-12);
        [a[0] / l, a[1] / l, a[2] / l]
    };
    let cross = |a: [f64; 3], b: [f64; 3]| [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]];
    let mut pos: Vec<f32> = vec![];
    let mut idx: Vec<u32> = vec![];
    for i in 0..n {
        let t = norm(sub(pts[(i + 1).min(n - 1)], pts[i.saturating_sub(1)]));
        // the side across the floor, and up out of it
        let side = norm(cross([0.0, 1.0, 0.0], t));
        let up = norm(cross(t, side));
        for k in 0..seg {
            let a = PI * 2.0 * k as f64 / seg as f64;
            let (c, s) = (a.cos() * r, a.sin() * r * flat);
            for d in 0..3 {
                pos.push((pts[i][d] + side[d] * c + up[d] * s) as f32);
            }
        }
    }
    for i in 0..n - 1 {
        for k in 0..seg {
            let k2 = (k + 1) % seg;
            let (a, b, c, d) = (i * seg + k, i * seg + k2, (i + 1) * seg + k2, (i + 1) * seg + k);
            idx.extend([a as u32, b as u32, d as u32, b as u32, c as u32, d as u32]);
        }
    }
    // the ends, each a fan
    for (i, flip) in [(0usize, true), (n - 1, false)] {
        let c = (pos.len() / 3) as u32;
        for d in 0..3 {
            pos.push(pts[i][d] as f32);
        }
        for k in 0..seg {
            let (a, b) = ((i * seg + k) as u32, (i * seg + (k + 1) % seg) as u32);
            if flip {
                idx.extend([c, b, a]);
            } else {
                idx.extend([c, a, b]);
            }
        }
    }
    let geo = Geo::new();
    geo.set_position(pos);
    geo.set_index(idx);
    geo.compute_vertex_normals();
    geo
}

/// Where the cord leaves the strain relief, and which way (`t04.py`, printed
/// by `hd_strain`)
const HD_CORD_START: [f64; 3] = [-0.0445, 0.0041, -0.1842];
const HD_CORD_DIR: [f64; 3] = [-0.215, -0.057, -0.975];
/// The cord's half-width and how flat it is (an SPT-2 flex, 18 AWG)
const HD_CORD_R: f64 = 0.0029;

// Hairdryer w/ cord + running hum audio (floor): 5% chance per level.
// An 1875 W pistol-grip hair dryer of the common kind (Blender batch `t04`),
// lying on its side where it fell: its barrel and nozzle, the intake's
// grille furred with lint, the handle with its two slide switches and its
// hang loop, the strain relief, and its 5 ft cord lying slack across the
// floor in loose bends to the moulded plug. Its plastic the old colour, a
// pale pink, burgundy, blue, grey, black or almond, scuffed and dusty.
pub(super) fn place_hairdryers(g: &mut Game, _dist: &[i32]) {
    g.pp.hairdryer_positions = vec![]; // reset per level
    if rnd() < 0.95 {
        return;
    }
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = open_tiles[(rnd() * open_tiles.len() as f64).floor() as usize];
    let body_colors = [0xf0d0e0, 0xa02040, 0x2060a0, 0x505050, 0x101010, 0xe0c088];
    let body_color = pick(&body_colors);
    let kind = body_colors.iter().position(|&c| c == body_color).unwrap_or(0);
    // the old cord's end (how far off and to which side its plug lay)
    let end_x = -0.14 * 0.15 - 0.30 + rnd() * 0.20;
    let end_z = (rnd() - 0.5) * 0.30;
    let hd = group();
    hd.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    hd.rotation().y = rnd() * PI * 2.0;
    let p = hd.pos();
    let vary = t04_vary(p.x, p.z, end_z);
    // the plastic: faded, scuffed, dusty
    let shells =
        [[0.78, 0.66, 0.68], [0.40, 0.12, 0.16], [0.17, 0.28, 0.42], [0.33, 0.33, 0.32], [0.065, 0.064, 0.062], [0.76, 0.68, 0.53]];
    let plastic = phong!({ map: &hd_plastic_tex(vary, shells[kind]), specular: 0x1c1c1c, shininess: 26 });
    t04_put(&hd, "Real__hairdryerBody", &plastic);
    t04_put(&hd, "Real__hairdryerGrille", &lambert!({ map: &hd_lint_tex(vary) }));
    let pale = kind == 0 || kind == 5;
    t04_put(&hd, "Real__hairdryerSwitches", &lambert!({ color: if pale { 0x8a8680 } else { 0xb4b0a6 } }));
    let flex = lambert!({ color: if pale { 0xb8b4aa } else { 0x232322 } });
    t04_put(&hd, "Real__hairdryerStrain", &flex);
    // the cord: from the strain relief, across the floor in slack bends to the plug
    let pts = hd_cord_path(end_x, end_z);
    hd.add(&mesh(t04_cord_geo(&pts, HD_CORD_R, 0.82, 8), &flex));
    let n = pts.len();
    let (a, b) = (pts[n - 2], pts[n - 1]);
    let (tx, tz) = (b[0] - a[0], b[2] - a[2]);
    let tl = (tx * tx + tz * tz).sqrt().max(1e-9);
    let (tx, tz) = (tx / tl, tz / tl);
    let plug = group();
    plug.position().set(b[0] + tx * 0.010, 0, b[2] + tz * 0.010);
    plug.rotation().y = (-tz).atan2(tx);
    t04_put(&plug, "Real__hairdryerPlug", &lambert!({ color: if pale { 0xb0aca2 } else { 0x1e1e1d } }));
    t04_put(&plug, "Real__hairdryerProngs", &phong!({ color: 0x7a7462, specular: 0x5a564a, shininess: 50 }));
    hd.add(&plug);
    g.world.add(&hd);
    // Track dryer position for hum audio.
    let hp = hd.pos();
    g.pp.hairdryer_positions.push(v3(hp.x, 0, hp.z));
}

/// The cord's centre line on the floor: out of the strain relief the way it
/// points, then slack, in bends either side of the line to the plug (how
/// far and which way from the old cord's end), never crossing itself
fn hd_cord_path(end_x: f64, end_z: f64) -> Vec<[f64; 3]> {
    let s = HD_CORD_START;
    let dl = (HD_CORD_DIR[0].powi(2) + HD_CORD_DIR[2].powi(2)).sqrt();
    let dh = [HD_CORD_DIR[0] / dl, HD_CORD_DIR[2] / dl];
    // the line to the plug: turned from the strain relief's heading by the
    // old end's side, its length by how far the old end lay
    let turn = (end_z / 0.15) * 0.6;
    let base = [dh[0] * turn.cos() - dh[1] * turn.sin(), dh[0] * turn.sin() + dh[1] * turn.cos()];
    let perp = [-base[1], base[0]];
    let len = 0.30 + (end_x + 0.321) * 0.9;
    let amp = (0.07 + 0.05 * (end_x + 0.321) / 0.2) * if end_z < 0.0 { 1.0 } else { -1.0 };
    let lead = 0.035;
    let y = HD_CORD_R * 0.82;
    let mut pts = vec![];
    // the first stretch straight on from the strain relief, down to the floor
    for k in 0..4 {
        let t = k as f64 / 4.0;
        pts.push([s[0] + dh[0] * lead * t, s[1] + (y - s[1]) * t, s[2] + dh[1] * lead * t]);
    }
    let o = [s[0] + dh[0] * lead, s[2] + dh[1] * lead];
    let m = 48;
    for k in 0..=m {
        let t = k as f64 / m as f64;
        // bends (a wave and a half) that fade in from the strain relief
        let env = t04_ss(0.0, 0.25, t);
        let side = amp * (t * PI * 3.0).sin() * env;
        // the heading turns from the strain relief's to the line's
        let h = [dh[0] + (base[0] - dh[0]) * env, dh[1] + (base[1] - dh[1]) * env];
        let along = len * t;
        let px = o[0] + h[0] * along * 0.5 + base[0] * along * 0.5 + perp[0] * side;
        let pz = o[1] + h[1] * along * 0.5 + base[1] * along * 0.5 + perp[1] * side;
        pts.push([px, y, pz]);
    }
    pts
}

/// The dryer's plastic (the parts' box faces): its colour, scuffed paler on
/// what rubs, a few scratches, dust
fn hd_plastic_tex(vary: u32, base: [f64; 3]) -> Tex {
    canvas_tex_opt(128, 128, false, move |ctx, w, h| {
        let seed = vary ^ 0x4d71;
        t04_fill(ctx, w, h, |u, v| {
            let n = t04_fbm(seed, u * 8.0, v * 8.0, 3);
            let scuff = t04_ss(0.62, 0.8, t04_fbm(seed ^ 0x3, u * 14.0, v * 5.0, 3)) * 0.18;
            let scratch = if t04_hash(seed ^ 0x8, (u * 128.0) as i32, (v * 32.0) as i32) < 0.02 { 0.2 } else { 0.0 };
            let dust = 0.08 + 0.10 * n;
            let d = [0.55, 0.53, 0.49];
            let mut c = [0.0; 3];
            for k in 0..3 {
                let s = base[k] + (0.62 - base[k]) * (scuff + scratch);
                c[k] = s * (1.0 - dust) + d[k] * dust;
            }
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// The grilles: dark plastic, lint furred over them in grey drifts
fn hd_lint_tex(vary: u32) -> Tex {
    canvas_tex_opt(64, 64, false, move |ctx, w, h| {
        let seed = vary ^ 0x1147;
        t04_fill(ctx, w, h, |u, v| {
            let lint = t04_ss(0.45, 0.7, t04_fbm(seed, u * 6.0, v * 6.0, 4));
            let fib = t04_vn(seed ^ 0x5, u * 64.0, v * 16.0);
            let g0 = 0.09 + (0.42 + 0.12 * fib - 0.09) * lint * 0.8;
            [g0, g0 * 0.98, g0 * 0.95, 1.0]
        });
    })
}
