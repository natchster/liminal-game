//! Batch `t20`'s abacus, made real (`docs/design/PROPS.md` P3; the Blender batch
//! `assets/src/blender/props/t20.py`, its notes `t20.notes.md`): a small
//! Chinese abacus (suanpan) of 13 rods lying on its back on the floor. It
//! keeps the old placer's draws, call for call.
use super::*;
use t20_kit::*;

// Small vintage abacus: 6% chance per level. A suanpan (Blender batch
// `t20`): hardwood rails strapped with brass at the corners, bamboo rods,
// two beads above the beam and five below on each; cleared, or with a sum
// left on it. The wood dark and dull, the beads polished by fingers, dust
// in the frame's corners.
pub(super) fn place_small_abacuses(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = pick(&open_tiles);
    // the old draws, call for call: where it lies and how it is turned
    let px = g.wx(x) + (rnd() - 0.5) * 2.0;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.0;
    let turn = rnd() * PI * 2.0;
    let abacus = group();
    abacus.position().set(px, 0, pz);
    abacus.rotation().y = turn;
    let vary = vary(px, pz, turn);
    put(&abacus, "Real__abacusFrame", &phong!({ map: &ab_wood_tex(vary), specular: 0x1c1814, shininess: 20 }));
    put(&abacus, "Real__abacusRods", &lambert!({ color: 0x7a6440 }));
    put(&abacus, "Real__abacusBrass", &phong!({ color: 0x5e4a28, specular: 0x4a3c22, shininess: 30 }));
    // cleared, or with the last sum left on it (where it lies, no draw)
    let beads = if hash(vary, 1, 2) < 0.5 { "Real__abacusBeadsA" } else { "Real__abacusBeadsB" };
    put(&abacus, beads, &phong!({ map: &ab_bead_tex(vary), specular: 0x2a2018, shininess: 34 }));
    g.world.add(&abacus);
}

/// The frame's hardwood (u along the rails): near black-brown, its edges
/// rubbed to the red of the wood, dust lodged along the rails' feet
fn ab_wood_tex(vary: u32) -> Tex {
    canvas_tex_opt(256, 128, false, move |ctx, w, h| {
        let seed = vary ^ 0xab;
        fill(ctx, w, h, |u, v| {
            let mut o = wood(seed, u, v, [0.13, 0.07, 0.045], 14.0, 40.0);
            let rub = ss(0.6, 0.8, fbm(seed ^ 3, u * 12.0, v * 6.0, 3));
            o = mix(o, [0.26, 0.13, 0.08], rub * 0.4);
            let dust = 0.06 + 0.14 * fbm(seed ^ 5, u * 9.0, v * 9.0, 3);
            opaque(mix(o, [0.42, 0.39, 0.34], dust))
        });
    })
}

/// The beads' wood (as their bounds are mapped): red-brown, polished, each
/// a little different
fn ab_bead_tex(vary: u32) -> Tex {
    canvas_tex_opt(128, 64, false, move |ctx, w, h| {
        let seed = vary ^ 0xbe;
        fill(ctx, w, h, |u, v| {
            let o = wood(seed, u, v, [0.24, 0.11, 0.06], 30.0, 60.0);
            let each = 0.85 + 0.25 * vn(seed ^ 1, u * 16.0, v * 10.0);
            opaque(mix(o.map(|c| c * each), [0.40, 0.37, 0.32], 0.06))
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
