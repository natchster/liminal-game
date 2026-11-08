//! Batch `t14`'s jointed teddy bear, made real (`docs/PROPS.md` P3; the
//! Blender batch `assets/src/blender/props/t14.py`, its notes
//! `t14.notes.md`): its placer and what only it uses. It keeps the old
//! placer's draws, call for call.
use crate::{jitter, pick_open};
use game::prelude::*;
use t14_kit::{fbm, fill, hash, mix, put, salt, ss, vn};

/// The head's joint in the body's frame (`t14.py` TEDDY_NECK): the head
/// turns on it
const TEDDY_NECK: (f64, f64, f64) = (0.0, 0.0705, -0.006);

/// The old colours, and the mohair each stands for (sRGB): cinnamon, dark
/// brown, gold, white gone to ivory, a dusty rose faded
const BEAR_COLORS: [u32; 5] = [0x8b5a2b, 0x604020, 0xc0a080, 0xf0e0c0, 0x804060];
const MOHAIR: [u32; 5] = [0x8a6242, 0x4f3828, 0xae8a5c, 0xcbbd9f, 0x957670];

// Small teddy bear: 4% chance per level. A jointed mohair teddy bear
// of the 1950s, 18 cm standing (the German and English makers' kind: mohair
// pile on a woven backing, stuffed with wood wool, jointed at the neck,
// shoulders and hips, glass eyes, the nose and mouth stitched in black
// floss, felt pads, long curved arms, a hump on its back), sitting with its
// legs out in front. The pile rubbed thin to its backing on the muzzle, the
// tummy and the paws, matted and dusty, a pad worn through to the wood
// wool. Now and then its head turns a little on its joint and back.
pub(crate) fn place_teddy_bears(g: &mut Game, _dist: &[i32]) {
    g.pp.plush_positions.clear(); // reset per level (shared across plush classes)
    if rnd() < 0.96 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let col = choice(&BEAR_COLORS);
    let k = BEAR_COLORS.iter().position(|&c| c == col).unwrap_or(0);
    let bear = group();
    let (px, pz) = jitter(g, x, z, 2.2);
    let s = salt(px, pz, 625);
    let fur = lambert!({ map: &mohair_tex(s, MOHAIR[k], false) });
    put(&bear, "Real__teddyBody", &fur);
    let pile = lambert!({ map: &pile_tex(s, MOHAIR[k], false), alphaTest: 0.5 });
    put(&bear, "Real__teddyBodyPile", &pile);
    put(&bear, "Real__teddyPads", &lambert!({ map: &felt_tex(s) }));
    // the head on its joint (the animator turns it)
    let head_group = group();
    head_group.position().set(TEDDY_NECK.0, TEDDY_NECK.1, TEDDY_NECK.2);
    put(&head_group, "Real__teddyHead", &lambert!({ map: &mohair_tex(s ^ 0x77, MOHAIR[k], true) }));
    put(&head_group, "Real__teddyHeadPile", &lambert!({ map: &pile_tex(s ^ 0x77, MOHAIR[k], true), alphaTest: 0.5 }));
    // glass eyes: they catch the light, they never make it
    put(&head_group, "Real__teddyEyes", &phong!({ color: 0x24170e, specular: 0x5a5a5a, shininess: 90 }));
    put(&head_group, "Real__teddyNose", &lambert!({ color: 0x221d1a }));
    bear.add(&head_group);
    bear.position().set(px, 0, pz);
    bear.rotation().y = rnd() * PI * 2.0;
    g.world.add(&bear);
    // Subtle head-turn animator: mostly still; occasionally the head turns 0.3 rad and returns.
    let bear_phase = rnd() * PI * 2.0;
    g.animate(move |_g, _dt, t| {
        let cycle = (t * 0.15 + bear_phase).sin();
        if cycle > 0.85 {
            let local_phase = (cycle - 0.85) / 0.15; // 0..1
            let turn_amount = (local_phase * PI).sin() * 0.3;
            head_group.rotation().y = turn_amount;
        } else {
            head_group.rotation().y = 0.0;
        }
    });
    // Track for plush shuffle audio.
    let p = bear.pos();
    g.pp.plush_positions.push(v3(p.x, 0, p.z));
}

/// Mohair pile (`base`, sRGB) on its woven backing, laid on by the part's
/// box (u across, v up a side): the pile's lie in fine streaks, its tips
/// paler, matted into clumps; rubbed thin to the backing in patches (on
/// the head, its clipped muzzle and brow most), faded, dust in it
fn mohair_tex(salt: u32, base: u32, head: bool) -> Tex {
    let b = [f64::from((base >> 16) & 255) / 255.0, f64::from((base >> 8) & 255) / 255.0, f64::from(base & 255) / 255.0];
    canvas_tex_opt(256, 256, false, move |ctx, w, h| {
        let seed = salt ^ 0x6d0;
        let backing = [0.62, 0.55, 0.44];
        fill(ctx, w, h, |u, v| {
            // the pile: fine streaks along its lie, clumps where it matted
            let streak = fbm(seed, u * 110.0, v * 55.0, 3);
            let clump = fbm(seed ^ 1, u * 26.0, v * 22.0, 3);
            let mut c = mix(b, [b[0] * 0.62, b[1] * 0.60, b[2] * 0.58], ss(0.40, 0.72, streak) * 0.4);
            c = mix(c, [b[0] * 0.66, b[1] * 0.64, b[2] * 0.6], ss(0.55, 0.75, clump) * 0.4);
            let tip = vn(seed ^ 2, u * 190.0, v * 150.0);
            c = mix(c, [b[0] * 1.2 + 0.04, b[1] * 1.17 + 0.04, b[2] * 1.14 + 0.03], ss(0.68, 0.95, tip) * 0.45);
            // faded unevenly
            let fade = fbm(seed ^ 3, u * 3.0, v * 3.0, 3);
            c = mix(c, [0.62, 0.58, 0.52], ss(0.5, 0.8, fade) * 0.25);
            // rubbed to the backing: its weave shows
            let wear = worn(seed, u, v, head);
            let weave = 0.5 + 0.5 * ((u * 300.0).sin() * (v * 300.0).sin());
            let bare = mix(backing, [backing[0] * 0.8, backing[1] * 0.8, backing[2] * 0.8], weave * 0.4);
            c = mix(c, mix(bare, b, 0.35), wear * 0.85);
            // dust and dirt
            let dust = fbm(seed ^ 6, u * 5.0, v * 5.0, 3);
            c = mix(c, [0.46, 0.43, 0.39], ss(0.55, 0.85, dust) * 0.3);
            let speck = vn(seed ^ 7, u * 120.0, v * 120.0);
            if speck > 0.9 {
                c = mix(c, [0.25, 0.22, 0.19], (speck - 0.9) * 5.0);
            }
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// Where the pile is rubbed down to its backing (0 to 1): patches, and on
/// the head its clipped muzzle and the brow over it, rubbed by hands. The
/// pile and its tips' layer read it at the same place
fn worn(seed: u32, u: f64, v: f64, head: bool) -> f64 {
    let mut wear = ss(0.66, 0.74, fbm(seed ^ 4, u * 7.0, v * 7.0, 4));
    if head {
        let (du, dv) = ((u - 0.5) / 0.16, (v - 0.32) / 0.2);
        wear = wear.max((1.0 - (du * du + dv * dv)).max(0.0) * 0.8 * ss(0.3, 0.6, fbm(seed ^ 5, u * 12.0, v * 12.0, 3)));
    }
    wear
}

/// The pile's tips (the stippled layer over the body): fibres' ends in
/// the mohair's colour, paler, about a third of them there and none where
/// it is rubbed bare
fn pile_tex(salt: u32, base: u32, head: bool) -> Tex {
    let b = [f64::from((base >> 16) & 255) / 255.0, f64::from((base >> 8) & 255) / 255.0, f64::from(base & 255) / 255.0];
    canvas_tex_opt(256, 256, false, move |ctx, w, h| {
        let seed = salt ^ 0x6d0;
        fill(ctx, w, h, |u, v| {
            let n = vn(seed ^ 0x31, u * 150.0, v * 120.0);
            let m = fbm(seed ^ 0x32, u * 20.0, v * 20.0, 2);
            let bare = worn(seed, u, v, head);
            let a = if n > 0.62 - 0.12 * (m - 0.5) + bare * 0.5 { 1.0 } else { 0.0 };
            let c = mix([b[0] * 1.08 + 0.02, b[1] * 1.06 + 0.02, b[2] * 1.05 + 0.02], [0.55, 0.52, 0.47], 0.15 + 0.2 * m);
            [c[0], c[1], c[2], a]
        });
    })
}

/// The pads' felt: a beige gone grey-brown, pilled, rubbed shiny at the
/// toes, a hole or two worn through to the wood wool under it
fn felt_tex(salt: u32) -> Tex {
    canvas_tex_opt(64, 64, false, move |ctx, w, h| {
        let seed = salt ^ 0xfe1;
        let holes: Vec<(f64, f64, f64)> = (0..3).map(|i| (hash(seed, i, 1), hash(seed, i, 2), 0.05 + 0.08 * hash(seed, i, 3))).collect();
        fill(ctx, w, h, |u, v| {
            let b = [0.62, 0.55, 0.45];
            let n = fbm(seed, u * 12.0, v * 12.0, 3);
            let mut c = mix(b, [0.46, 0.40, 0.33], ss(0.4, 0.8, n) * 0.6);
            let pill = vn(seed ^ 1, u * 40.0, v * 40.0);
            c = mix(c, [0.70, 0.64, 0.55], ss(0.75, 0.9, pill) * 0.4);
            for &(hx, hy, r) in &holes {
                let d = ((u - hx).powi(2) + (v - hy).powi(2)).sqrt();
                if d < r {
                    // the wood wool: pale straw shavings in the dark
                    let straw = fbm(seed ^ 9, u * 60.0, v * 25.0, 2);
                    c = mix([0.18, 0.14, 0.10], [0.56, 0.46, 0.30], ss(0.45, 0.7, straw));
                }
            }
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// What this batch's painters share in this crate
mod t14_kit {
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
    pub(super) fn ss(e0: f64, e1: f64, x: f64) -> f64 {
        let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    }
    pub(super) fn mix(a: [f64; 3], b: [f64; 3], t: f64) -> [f64; 3] {
        let t = t.clamp(0.0, 1.0);
        [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
    }
    /// Every pixel from where it is (u across, v up), on every core
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
    /// A salt from where a thing is (no draw): its painters differ from
    /// floor to floor
    pub(super) fn salt(x: f64, z: f64, k: u32) -> u32 {
        ((x * 7919.0).abs() as u32) ^ ((z * 104_729.0).abs() as u32).rotate_left(13) ^ k.wrapping_mul(0x9e37_79b9)
    }
    /// The part `name` of a Blender batch as modelled, added to `grp`
    pub(super) fn put(grp: &Obj, name: &str, mat: &Mat) -> Option<Obj> {
        let geo = game::assets::props::prop_part_as_modelled(name)?;
        let m = mesh(geo, mat.clone());
        grp.add(&m);
        Some(m)
    }
}
