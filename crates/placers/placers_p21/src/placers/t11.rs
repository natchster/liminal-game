//! Batch `t11`'s roe buck's head, made real (`docs/PROPS.md` P3; the
//! Blender batch `assets/src/blender/props/t11.py`, its notes
//! `t11.notes.md`): its placer and what only it uses. It keeps the old
//! placer's draws, call for call, and hangs where the old one hung.
use super::*;
use t11_kit::{fbm, fill, mix, put, salt, sstep, vn};

/// The antlers' place on the head (`t11.py` `HEAD_ANTLERS`): the burrs'
/// bases on the pedicles, on top of the skull
const HEAD_ANTLERS: [f64; 3] = [0.0, 0.080_494, 0.161_283];
/// The pair's turn about x: the head's (its nose 25 degrees down,
/// `HEAD_TURN`) less the antlers' lean back off the frontal (`ROE_BETA`,
/// 52 degrees)
const HEAD_ANTLER_TURN: f64 = -0.471_239;

// R505 wall taxidermy deer head: 4% chance per level. A roe buck's head mount (a
// Kopfpraeparat): the hide tanned and set over its form, the neck cut short
// against a carved oak shield, the head turned a little down as if it
// looked ahead, the big ears up and out, glass eyes that catch the light,
// the black nose, the white of its chin and throat; its antlers (a
// six-pointer) on its pedicles. The coat faded and dusty, a few bald spots
// where moths have been.
pub(crate) fn place_deer_head_mounts(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = candidates[(rnd() * candidates.len() as f64).floor() as usize];
    let k = salt(g.wx(c.x) + c.dx as f64, g.wz(c.z) + c.dz as f64, 21);
    let mount = group();
    put(&mount, "Real__roeHeadShield", &phong!({ map: &oak_tex(k, 160, 224), specular: 0x141210, shininess: 16 }));
    put(&mount, "Real__roeHeadHide", &lambert!({ map: &hide_tex(k) }));
    put(&mount, "Real__roeHeadPale", &lambert!({ map: &pale_tex(k), side: DoubleSide }));
    // the nose's leather, painted and glossed by the taxidermist
    put(&mount, "Real__roeHeadNose", &phong!({ color: 0x1c1816, specular: 0x2c2a28, shininess: 38 }));
    // glass eyes: they catch the light, they never make it
    put(&mount, "Real__roeHeadEyes", &phong!({ color: 0x1e140c, specular: 0x9a9a9a, shininess: 120 }));
    let antlers = group();
    antlers.position().set(HEAD_ANTLERS[0], HEAD_ANTLERS[1], HEAD_ANTLERS[2]);
    antlers.rotation().x = HEAD_ANTLER_TURN;
    put(&antlers, "Real__roeAntlers", &lambert!({ map: &antler_tex(k) }));
    put(&antlers, "Real__roeAntlerTips", &phong!({ color: 0xcfc3a8, specular: 0x262420, shininess: 30 }));
    mount.add(&antlers);
    mount_on_wall(g, &mount, &c, 0.005, 2.00, 0.6);
    g.world.add(&mount);
}

/// A roe's winter coat (u across, v up the head): grizzled grey-brown, the
/// hairs' pale bands flecked through it, lying in streaks, darker over the
/// crown and down the neck's top; faded, dusty, a few bald spots where
/// moths ate the hair to the hide
fn hide_tex(salt: u32) -> Tex {
    canvas_tex_opt(256, 256, false, move |ctx, w, h| {
        let seed = salt ^ 0x1d3;
        fill(ctx, w, h, |u, v| {
            let mut c = [0.42, 0.35, 0.27];
            // darker over the top
            c = mix(c, [0.29, 0.23, 0.17], sstep(0.45, 0.95, v) * 0.7);
            // the hair: fine streaks down, each hair's band paler
            let streak = fbm(seed, u * 140.0, v * 26.0, 3);
            c = mix(c, [0.20, 0.16, 0.12], sstep(0.5, 0.75, streak) * 0.45);
            let fleck = vn(seed ^ 3, u * 220.0, v * 90.0);
            c = mix(c, [0.62, 0.56, 0.47], sstep(0.68, 0.9, fleck) * 0.45);
            // the coat's lie: broad soft waves
            let wave = fbm(seed ^ 5, u * 7.0, v * 7.0, 3);
            c = mix(c, [0.47, 0.40, 0.31], (wave - 0.5).max(0.0) * 0.45);
            // moth-eaten spots: the hair gone to the tanned skin
            let moth = fbm(seed ^ 7, u * 22.0, v * 22.0, 3);
            c = mix(c, [0.54, 0.46, 0.36], sstep(0.73, 0.76, moth) * 0.5);
            // faded and dusty
            c = mix(c, [0.55, 0.52, 0.47], 0.12);
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// The pale hair of the chin and the lower lip, the throat's patch and
/// the ears' insides: an old white gone cream and grey, streaked
fn pale_tex(salt: u32) -> Tex {
    canvas_tex_opt(128, 128, false, move |ctx, w, h| {
        let seed = salt ^ 0x1e7;
        fill(ctx, w, h, |u, v| {
            let mut c = [0.68, 0.65, 0.58];
            let streak = fbm(seed, u * 90.0, v * 20.0, 3);
            c = mix(c, [0.48, 0.44, 0.38], sstep(0.5, 0.8, streak) * 0.5);
            let grime = fbm(seed ^ 3, u * 6.0, v * 6.0, 3);
            c = mix(c, [0.55, 0.50, 0.41], (grime - 0.45).max(0.0) * 1.2);
            // the ears' insides (the top of the part): grey-brown, the white
            // hairs in tufts along them
            let ear = sstep(0.42, 0.5, v);
            let tuft = sstep(0.55, 0.8, fbm(seed ^ 5, u * 40.0, v * 14.0, 2));
            c = mix(c, mix([0.30, 0.26, 0.22], [0.52, 0.49, 0.44], tuft), ear);
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// Quartersawn oak under an old dark stain and varnish (u across, v up the
/// shield), as the antlers' shield (`placers_p06` `rustic/t11.rs`)
fn oak_tex(salt: u32, w: u32, h: u32) -> Tex {
    canvas_tex_opt(w, h, false, move |ctx, w, h| {
        let seed = salt ^ 0x0a4;
        fill(ctx, w, h, |u, v| {
            let base = [0.33, 0.22, 0.13];
            let grain = fbm(seed, u * 46.0 + fbm(seed ^ 13, u * 3.0, v * 6.0, 2) * 4.0, v * 3.0, 3);
            let mut c = mix(base, [0.20, 0.12, 0.07], sstep(0.45, 0.72, grain) * 0.5);
            let ray = vn(seed ^ 3, u * 30.0, v * 160.0);
            c = mix(c, [0.46, 0.33, 0.20], sstep(0.78, 0.92, ray) * sstep(0.45, 0.6, fbm(seed ^ 5, u * 8.0, v * 8.0, 2)) * 0.6);
            let blot = fbm(seed ^ 7, u * 5.0, v * 5.0, 3);
            c = mix(c, [0.40, 0.28, 0.17], (blot - 0.5).max(0.0) * 0.8);
            let edge = (u.min(1.0 - u)).min(v.min(1.0 - v));
            c = mix(c, [0.44, 0.31, 0.19], (1.0 - sstep(0.0, 0.06, edge)) * 0.35);
            let cr = (vn(seed ^ 9, u * 90.0, v * 120.0) - 0.5).abs();
            c = mix(c, [0.10, 0.06, 0.03], (1.0 - sstep(0.0, 0.03, cr)) * 0.35);
            let d = fbm(seed ^ 11, u * 60.0, v * 80.0, 2);
            c = mix(c, [0.48, 0.45, 0.40], sstep(0.55, 0.8, d) * 0.25 + 0.08);
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// A roe's antler (v up the beam), as the antlers' trophy's
fn antler_tex(salt: u32) -> Tex {
    canvas_tex_opt(64, 128, false, move |ctx, w, h| {
        let seed = salt ^ 0x0c7;
        fill(ctx, w, h, |u, v| {
            let low = 1.0 - sstep(0.15, 0.6, v);
            let mut c = mix([0.40, 0.30, 0.20], [0.20, 0.14, 0.09], low);
            let gut = fbm(seed, u * 34.0, v * 3.0, 3);
            c = mix(c, [0.10, 0.07, 0.05], sstep(0.55, 0.75, gut) * 0.6);
            let pearl = vn(seed ^ 3, u * 60.0, v * 110.0);
            c = mix(c, [0.62, 0.54, 0.42], sstep(0.72, 0.9, pearl) * (0.35 + 0.4 * low));
            let d = vn(seed ^ 5, u * 90.0, v * 160.0);
            c = mix(c, [0.55, 0.52, 0.46], sstep(0.85, 0.97, d) * 0.4);
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// The painting kit of batch `t11`'s painters (as batches `t08`'s and
/// `t09`'s): value noise and a per-pixel fill (pure, so `par_rows` may run
/// it on every core), a salt from where a thing is, a part placed.
#[allow(dead_code)]
mod t11_kit {
    use game::prelude::*;

    pub(super) fn hash(seed: u32, i: i32, j: i32) -> f64 {
        let mut h = seed ^ (i as u32).wrapping_mul(0x27d4_eb2d) ^ (j as u32).wrapping_mul(0x1656_67b1);
        h = (h ^ (h >> 15)).wrapping_mul(0x85eb_ca6b);
        h = (h ^ (h >> 13)).wrapping_mul(0xc2b2_ae35);
        h ^= h >> 16;
        f64::from(h) / 4_294_967_296.0
    }
    pub(super) fn vn(seed: u32, x: f64, y: f64) -> f64 {
        let (xf, yf) = (x.floor(), y.floor());
        let (i, j) = (xf as i32, yf as i32);
        let (u, v) = (x - xf, y - yf);
        let (u, v) = (u * u * (3.0 - 2.0 * u), v * v * (3.0 - 2.0 * v));
        let (a, b) = (hash(seed, i, j), hash(seed, i + 1, j));
        let (c, d) = (hash(seed, i, j + 1), hash(seed, i + 1, j + 1));
        a + (b - a) * u + (c - a) * v + (a - b - c + d) * u * v
    }
    pub(super) fn fbm(seed: u32, x: f64, y: f64, oct: u32) -> f64 {
        let (mut s, mut amp, mut f, mut n) = (0.0, 0.5, 1.0, 0.0);
        for k in 0..oct {
            s += amp * vn(seed.wrapping_add(k * 7919), x * f, y * f);
            n += amp;
            amp *= 0.5;
            f *= 2.03;
        }
        s / n
    }
    pub(super) fn sstep(e0: f64, e1: f64, x: f64) -> f64 {
        let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    }
    pub(super) fn mix(a: [f64; 3], b: [f64; 3], t: f64) -> [f64; 3] {
        let t = t.clamp(0.0, 1.0);
        [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
    }
    /// Fills the canvas pixel by pixel on every core: `f(u, v)` (u right, v
    /// up, both 0..1, as the texture is laid) gives straight sRGB and alpha
    pub(super) fn fill(ctx: &mut Canvas, w: f64, h: f64, f: impl Fn(f64, f64) -> [f64; 4] + Sync) {
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
    /// A salt from where a thing is (no draw): its painters' strokes differ
    /// from floor to floor
    pub(super) fn salt(x: f64, z: f64, k: u32) -> u32 {
        ((x * 7919.0).abs() as u32) ^ ((z * 104_729.0).abs() as u32).rotate_left(13) ^ k.wrapping_mul(0x9e37_79b9)
    }
    /// The part `name` of a Blender batch as modelled, added to `grp` in `mat`
    pub(super) fn put(grp: &Obj, name: &str, mat: &Mat) -> Option<Obj> {
        let geo = game::assets::props::prop_part_as_modelled(name)?;
        let m = mesh(geo, mat.clone());
        grp.add(&m);
        Some(m)
    }
}
