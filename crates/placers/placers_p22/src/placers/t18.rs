//! Batch `t18`'s hanging censer, made real (`docs/PROPS.md` P3, "the party
//! and the shrine"): a brass thurible of the church kind hung from a swag
//! hook in the ceiling by a length of lamp chain, cold
//! (`assets/src/blender/props/t18.py`, `t18.notes.md`). Its placer keeps the
//! old one's draws, call for call.
use crate::{open_tiles, scatter_on_floor};
use game::prelude::*;

/// The thurible (`t18.py`, m): its finger ring's top wire above the bowl's
/// rim, and the ring's offset; the lamp chain's pair of links (its pitch,
/// and from the lower link's lowest wire to the upper one's top); the hook's
/// lowest wire below the ceiling
const RING_TOP: f64 = 0.3622;
const RING_X: f64 = -0.004;
const PAIR_PITCH: f64 = 0.0296;
const PAIR_TOP: f64 = 0.0322;
const HOOK_LOW: f64 = 0.035;
/// A link resting on a wire: the lower one's wire centre this far below the
/// upper one's (their wires' radii: the lamp chain's 1.3 mm, the finger
/// ring's 2.2 mm, the hook's 2 mm)
const ON_RING: f64 = 0.0035;
const ON_HOOK: f64 = 0.0033;

// Hanging censer: 4% chance per level. A brass thurible (a bowl 108 mm
// across on its foot, a pierced domed lid, three chains to a chain head)
// hung by its finger ring from a brass swag hook screwed into the ceiling,
// on as many links of lamp chain as bring its bowl where the old one hung;
// the brass gone brown, its chains darker; cold, no ember in it (the old
// ember glow is gone). It sways a little about the hook, as the old one
// swayed.
pub(crate) fn place_hanging_censers(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let chain_len = 0.5 + rnd() * 0.5;
    // the old ember glow had no draw; the sway's two
    let sway_phase = rnd() * PI * 2.0;
    let sway_period = 10.0 + rnd() * 6.0;
    let censer = group();
    scatter_on_floor(g, &censer, x, z, 1.8);
    let p = censer.pos();
    let salt = salt(p.x, p.z, 0xc35);
    let brass = phong!({ map: &tarnish_tex(salt), color: 0x8a6a34, specular: 0x6a5430, shininess: 30 });
    let chain_brass = phong!({ map: &tarnish_tex(salt ^ 0x51), color: 0x6c5230, specular: 0x4a3c24, shininess: 24 });
    // the hook, fixed to the ceiling; all below it swings about its J
    let hook = group();
    hook.position().y = WALL_H;
    censer.add(&hook);
    put(&hook, "Real__censerHook", &brass);
    let swing = group();
    swing.position().y = -HOOK_LOW;
    hook.add(&swing);
    // the lamp chain: pairs of links down from the hook, as many as bring
    // the bowl's rim where the old bowl hung (WALL_H - chain_len - 0.05)
    let pairs = (1.0 + ((chain_len - 0.3726) / PAIR_PITCH).round()).max(1.0) as usize;
    let top = ON_HOOK - PAIR_TOP;
    for i in 0..pairs {
        if let Some(m) = put(&swing, "Real__censerDropLinks", &chain_brass) {
            m.position().y = top - i as f64 * PAIR_PITCH;
        }
    }
    let ring_y = top - (pairs - 1) as f64 * PAIR_PITCH + ON_RING;
    let thurible = group();
    thurible.position().set(-RING_X, ring_y - RING_TOP, 0);
    swing.add(&thurible);
    put(&thurible, "Real__censerBowl", &brass);
    put(&thurible, "Real__censerLid", &brass);
    put(&thurible, "Real__censerChains", &chain_brass);
    // Slow sway, about the hook.
    {
        let swing = swing.clone();
        g.animate(move |_g, _dt, t| {
            swing.rotation().z = (t * (PI * 2.0 / sway_period) + sway_phase).sin() * 0.03;
        });
    }
    g.world.add(&censer);
}

/// Brass gone dull (grey on the brass's colour: how bright): brown tarnish
/// in patches, darker spots, brighter where hands took it
fn tarnish_tex(salt: u32) -> Tex {
    canvas_tex_opt(64, 64, false, move |ctx, w, h| {
        let seed = salt ^ 0xb7c;
        let iw = w as usize;
        let mut img = ctx.create_image_data(w, h);
        par_rows(&mut img.data, iw * 4, |j, out| {
            for i in 0..iw {
                let (u, v) = ((i as f64 + 0.5) / w, (j as f64 + 0.5) / h);
                let n = fbm(seed, u * 6.0, v * 6.0);
                let t = ((n - 0.4) / 0.35).clamp(0.0, 1.0);
                let t = t * t * (3.0 - 2.0 * t);
                let mut c = [0.42 + 0.43 * t, 0.38 + 0.44 * t, 0.33 + 0.43 * t];
                let spot = vn(seed ^ 3, u * 40.0, v * 40.0);
                if spot > 0.72 {
                    let k = ((spot - 0.72) * 3.0).min(1.0);
                    c = [c[0] + (0.26 - c[0]) * k, c[1] + (0.24 - c[1]) * k, c[2] + (0.21 - c[2]) * k];
                }
                for k in 0..3 {
                    out[i * 4 + k] = (c[k].clamp(0.0, 1.0) * 255.0).round() as u8;
                }
                out[i * 4 + 3] = 255;
            }
        });
        ctx.put_image_data(&img, 0, 0);
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

fn fbm(seed: u32, x: f64, y: f64) -> f64 {
    let (mut s, mut amp, mut f, mut n) = (0.0, 0.5, 1.0, 0.0);
    for k in 0..3 {
        s += amp * vn(seed.wrapping_add(k * 7919), x * f, y * f);
        n += amp;
        amp *= 0.5;
        f *= 2.03;
    }
    s / n
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
