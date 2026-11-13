//! Batch `t18`'s crystal ball, made real (`docs/design/PROPS.md` P3, "the party
//! and the shrine"): a clear glass ball as they are sold for scrying, on its
//! turned wooden stand (`assets/src/blender/props/t18.py`, `t18.notes.md`).
//! It catches the light; it never makes any. Its placer keeps the old one's
//! draws, call for call.
use crate::{open_tiles, pick_tile, pick_u32};
use game::prelude::*;

// Small crystal ball: 5% chance per level. A glass ball 80 mm across
// in the cupped top of a turned, dark-stained hardwood stand: the room seen
// in its glass by Fresnel, a faint tint in the glass (by the old colour),
// dust settled on its crown and the smears of hands; the stand's stain worn
// through on its edges. No light of its own (the old inner light and its
// spark are gone; their phase's draw is kept).
pub fn place_small_crystal_balls(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick_tile(&open);
    let ball_colors: [u32; 4] = [0xc0e0f0, 0xe0d0f0, 0xd8e8d8, 0xf0e0d0];
    let tint = pick_u32(&ball_colors);
    // the old inner light's shimmer phase (no light: glass does not shine)
    let _ = rnd();
    let px = g.wx(x) + (rnd() - 0.5) * 2.4;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.4;
    let turn = rnd() * PI * 2.0;
    let salt = salt(px, pz, 0xba1);
    let ball = group();
    if let Some(m) = put(&ball, "Real__crystalBallGlass", &ball_glass_mat(&ball_dust_tex(salt), tint)) {
        m.set_render_order(2);
    }
    put(&ball, "Real__crystalBallStand", &phong!({ map: &stand_tex(salt), specular: 0x241c14, shininess: 22 }));
    ball.position().set(px, 0, pz);
    ball.rotation().y = turn;
    g.world.add(&ball);
}

/// Glass as the windows' and t09's skull's (`Patch::Glass`): the room seen
/// in it by Fresnel, bright at its rim, clear through its middle; 80 mm of
/// soda glass greying what is seen through it a little, the old colour a
/// faint tint in it; its dust lit as dust
fn ball_glass_mat(map: &Tex, tint: u32) -> Mat {
    // the old colour, mostly gone: a glass's own faint tint
    let t = |c: u32, s: u32| 0.82 + 0.18 * f64::from((c >> s) & 0xff) / 255.0;
    let col = (((t(tint, 16) * 255.0) as u32) << 16) | (((t(tint, 8) * 255.0) as u32) << 8) | ((t(tint, 0) * 255.0) as u32);
    let m = phong!({ color: col, specular: 0x9a9a9a, shininess: 240, transparent: true, depthWrite: false });
    {
        let mut mm = m.m();
        mm.map = Some(map.clone());
        mm.patch = Patch::Glass;
        mm.look_settled = true;
        mm.name = "crystal_ball".into();
    }
    {
        let pu = m.r().patch_uniforms.clone();
        let f = &mut pu.borrow_mut().f;
        // the room in it, and nothing lit behind it
        f[0] = 0.5;
        // no mist
        f[4] = 0.5;
        f[5] = 0.6;
        f[6] = 0.3;
        f[7] = 0.0;
        // dust: how much of the map's grey, its colour (linear)
        f[8] = 0.55;
        f[9] = 0.20;
        f[10] = 0.19;
        f[11] = 0.17;
        // its body: clear glass, a little grey through 8 cm
        f[12] = 0.18;
        // a solid's two faces and its inside
        f[13] = 0.12;
    }
    m
}

/// Dust on a glass ball that stood on a floor (grey: how much; laid round
/// it, v up): heavier toward its crown where it settles, smears where it
/// was handled round its middle
fn ball_dust_tex(salt: u32) -> Tex {
    canvas_tex_opt(128, 64, false, move |ctx, w, h| {
        let seed = salt ^ 0x5c2;
        fill(ctx, w, h, |u, v| {
            let d = fbm(seed, u * 9.0, v * 5.0, 3) * 0.6 + vn(seed ^ 3, u * 40.0, v * 20.0) * 0.4;
            let smear = (fbm(seed ^ 5, u * 3.0, v * 9.0, 2) - 0.58).max(0.0) * 1.6 * (1.0 - (v - 0.5).abs() * 2.0);
            let top = sstep(0.55, 1.0, v) * 0.35;
            let g = (0.08 + 0.32 * d * (0.4 + v) + smear + top).min(1.0);
            [g, g, g, 1.0]
        });
    })
}

/// The stand's wood (laid round it, v up): walnut stained dark, its grain
/// running round, the stain rubbed through to the paler wood on its rims,
/// dust on its top
fn stand_tex(salt: u32) -> Tex {
    canvas_tex_opt(128, 32, false, move |ctx, w, h| {
        let seed = salt ^ 0x7d1;
        fill(ctx, w, h, |u, v| {
            let grain = vn(seed, u * 60.0, v * 3.0) * 0.6 + fbm(seed ^ 2, u * 9.0, v * 9.0, 2) * 0.4;
            let mut c = mix([0.17, 0.11, 0.07], [0.11, 0.07, 0.045], grain);
            // the rims (the foot's edge and the top's): worn to the wood
            let rim = (1.0 - sstep(0.0, 0.12, v)).max(sstep(0.82, 0.95, v) * (1.0 - sstep(0.95, 1.0, v)));
            c = mix(c, [0.36, 0.25, 0.16], rim * 0.55 * vn(seed ^ 4, u * 30.0, v * 4.0));
            c = mix(c, [0.40, 0.38, 0.35], sstep(0.85, 1.0, v) * 0.25);
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

/// Fills the canvas pixel by pixel on every core: `f(u, v)` (u right, v
/// up, both 0..1, as the texture is laid) gives straight sRGB and alpha
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

/// A salt from where a thing is (no draw): its painters' strokes differ
/// from floor to floor
fn salt(x: f64, z: f64, k: u32) -> u32 {
    ((x * 7919.0).abs() as u32) ^ ((z * 104_729.0).abs() as u32).rotate_left(13) ^ k.wrapping_mul(0x9e37_79b9)
}

/// The part `name` of a Blender batch as modelled, added to `grp` in `mat`
fn put(grp: &Obj, name: &str, mat: &Mat) -> Option<Obj> {
    let geo = game::assets::props::prop_part_as_modelled(name)?;
    let m = mesh(geo, mat.clone());
    grp.add(&m);
    Some(m)
}
