//! Batch `t09`'s shrine things, made real (`docs/PROPS.md` P3, "the party
//! and the shrine"): what a person brought and set out, never a working
//! occult device (`assets/src/blender/props/t09.py`, `t09.notes.md`): the
//! cast glass skull, the carved owl, the brass candlestick and its candle
//! burnt down, the resin skulls stacked. Their placers and what only they
//! use; each keeps the old placer's draws, call for call.
use crate::{open_tiles, pick_tile, pick_u32};
use game::prelude::*;
use t09_kit::*;

// R491 crystal skull: 3% chance per level. A "crystal skull" as they are
// sold: a cast glass ornament, 10.5 cm deep, its jaw fused shut and its base
// ground flat, standing on the floor; the room in its glass by Fresnel, dust
// and a hand's smears on it. It catches the light; it never makes any.
pub fn place_crystal_skulls(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    } // 3% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick_tile(&open);
    // the old glow's pulse phase (no light: glass does not shine)
    let _ = rnd();
    let px = g.wx(x) + (rnd() - 0.5) * 2.4;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.4;
    let turn = rnd() * PI * 2.0;
    let skull = group();
    if let Some(m) = put(&skull, "Real__crystalSkull", &glass_mat(&glass_dust_tex(salt(px, pz, 1)))) {
        m.set_render_order(2);
    }
    skull.position().set(px, 0, pz);
    skull.rotation().y = turn;
    g.world.add(&skull);
}

/// Cast glass as the windows' and the wine glass's (`Patch::Glass`): the
/// room seen in it by Fresnel, bright at its edges, clear square on; thick
/// enough to grey what is seen through it a little; its dust lit as dust
fn glass_mat(map: &Tex) -> Mat {
    let m = phong!({ color: 0xffffff, specular: 0x9a9a9a, shininess: 220, transparent: true, depthWrite: false });
    {
        let mut mm = m.m();
        mm.map = Some(map.clone());
        mm.patch = Patch::Glass;
        mm.look_settled = true;
        mm.name = "crystal_skull".into();
    }
    {
        let pu = m.r().patch_uniforms.clone();
        let f = &mut pu.borrow_mut().f;
        // the room in it, and nothing lit behind it
        f[0] = 0.45;
        // no mist
        f[4] = 0.5;
        f[5] = 0.6;
        f[6] = 0.3;
        f[7] = 0.0;
        // dust: how much of the map's grey, its colour (linear)
        f[8] = 0.6;
        f[9] = 0.20;
        f[10] = 0.19;
        f[11] = 0.17;
        // the cast's body: grey through 8 cm of glass full of seeds
        f[12] = 0.22;
        // a solid's two faces and its inside: brighter than a pane square on
        f[13] = 0.14;
    }
    m
}

/// Dust on glass that stood on a floor (grey: how much): a haze, heavier
/// over the crown where it settles, smears where it was handled
fn glass_dust_tex(salt: u32) -> Tex {
    canvas_tex_opt(128, 128, false, move |ctx, w, h| {
        let seed = salt ^ 0x5c1;
        fill(ctx, w, h, |u, v| {
            let d = fbm(seed, u * 7.0, v * 7.0, 3) * 0.6 + vn(seed ^ 3, u * 29.0, v * 29.0) * 0.4;
            let smear = (fbm(seed ^ 5, u * 3.0, v * 11.0, 2) - 0.58).max(0.0) * 1.6;
            let top = sstep(0.55, 1.0, v) * 0.15;
            let g = (0.16 + 0.42 * d + smear + top).min(1.0);
            [g, g, g, 1.0]
        });
    })
}

// R489 small owl figurine: 5% chance per level. A carved wooden owl (15 cm),
// the souvenir kind carved from one block and stained dark: a horned owl
// standing on a rock carved with it, its feathers cut in rows of chisel
// scallops (the stain lying dark in the cuts), its glass eyes amber with
// the pupil painted behind; the crown and the tufts rubbed lighter by
// hands, dust on it.
pub fn place_small_owls(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick_tile(&open);
    // the old body colour: the stain
    let stains: [u32; 3] = [0x3a2818, 0x201410, 0x5a4030];
    let stain = pick_u32(&stains);
    let px = g.wx(x) + (rnd() - 0.5) * 2.4;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.4;
    let turn = rnd() * PI * 2.0;
    let owl = group();
    put(&owl, "Real__owlWood", &lambert!({ map: &owl_wood_tex(salt(px, pz, 2), stain) }));
    // glass eyes: they catch the light and never make it
    put(&owl, "Real__owlEyes", &phong!({ color: 0x6a4a14, specular: 0x7a7a7a, shininess: 90 }));
    put(&owl, "Real__owlPupils", &phong!({ color: 0x141210, specular: 0x6a6a6a, shininess: 90 }));
    owl.position().set(px, 0, pz);
    owl.rotation().y = turn;
    g.world.add(&owl);
}

/// The owl's wood, laid on it from the front and the sides (u across, v up
/// its 134 mm): the stain over lime's pale grain, darker in the chisel cuts
/// of the feathers' rows (long feathers low on the sides of the body, short
/// ones on the breast), the facial disc cut in fine rays round the eyes, the
/// rock below cut in flat facets; rubbed light where hands held it, dust
fn owl_wood_tex(salt: u32, stain: u32) -> Tex {
    let s = [f64::from((stain >> 16) & 255) / 255.0, f64::from((stain >> 8) & 255) / 255.0, f64::from(stain & 255) / 255.0];
    canvas_tex_opt(256, 256, false, move |ctx, w, h| {
        let seed = salt ^ 0x0a1;
        // the stain over the pale wood: a third of the wood's own colour shows
        let base = mix(s, [0.62, 0.50, 0.36], 0.30);
        fill(ctx, w, h, |u, v| {
            let mm = v * 134.0;
            // the grain, along the block (up)
            let grain = fbm(seed, u * 70.0, v * 5.0, 3);
            let mut c = mix(base, mix(base, [0.0, 0.0, 0.0], 0.45), sstep(0.45, 0.75, grain) * 0.5);
            c = mix(c, [0.70, 0.58, 0.42], sstep(0.62, 0.8, fbm(seed ^ 3, u * 9.0, v * 9.0, 3)) * 0.12);
            let cut = if mm < 21.0 {
                // the rock: flat facets, each its own shade
                let f = vn(seed ^ 5, u * 14.0, v * 30.0);
                c = mix(c, [0.0, 0.0, 0.0], (f - 0.5) * 0.4);
                0.0
            } else if mm > 92.0 && mm < 120.0 && (u - 0.5).abs() < 0.32 {
                // the facial disc: fine rays from each eye
                let ex = if u < 0.5 { 0.373 } else { 0.627 };
                let (dx, dy) = ((u - ex) * 98.0, mm - 104.5);
                let r = dx.hypot(dy);
                let a = dy.atan2(dx);
                let ray = ((a * 24.0).sin().abs()).powf(8.0) * sstep(8.5, 11.0, r) * (1.0 - sstep(14.0, 17.0, r));
                ray * 0.25
            } else {
                // the feathers: scallop rows, larger and longer toward the
                // sides and low on the body
                let low = 1.0 - sstep(40.0, 70.0, mm);
                let side = sstep(0.2, 0.42, (u - 0.5).abs());
                let k = 1.0 + 0.6 * side.max(low);
                let (cw, ch) = (8.0 * k / 98.0, 6.5 * k * (1.0 + 0.5 * side) / 134.0);
                let row = (v / ch).floor();
                let off = if (row as i64) % 2 == 0 { 0.0 } else { cw * 0.5 };
                let cu = ((u + off) / cw).fract() - 0.5;
                let cv = (v / ch).fract();
                let edge = 0.12 + 1.6 * cu * cu;
                let line = 1.0 - sstep(0.0, 0.09, (cv - edge).abs());
                // the cut's face, just above the line, catches the light
                let face = sstep(edge, edge + 0.08, cv) * (1.0 - sstep(edge + 0.1, edge + 0.4, cv));
                c = mix(c, [0.72, 0.60, 0.45], face * 0.10);
                line * 0.75 * (1.0 - sstep(118.0, 126.0, mm))
            };
            c = mix(c, mix(s, [0.0, 0.0, 0.0], 0.55), cut);
            // rubbed: the crown, the tufts, the rock's front edge
            let rub = sstep(118.0, 132.0, mm) * 0.35 + (1.0 - sstep(16.0, 21.0, mm)) * sstep(12.0, 16.0, mm) * 0.2;
            c = mix(c, [0.66, 0.53, 0.38], rub * (0.6 + 0.4 * vn(seed ^ 7, u * 20.0, v * 20.0)));
            // dust in its pores
            let dust = vn(seed ^ 9, u * 180.0, v * 180.0);
            if dust > 0.8 {
                c = mix(c, [0.46, 0.44, 0.40], (dust - 0.8) * 2.0);
            }
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// The candle's stub (`t09.py`): the flame's root over the socket's floor
/// (m), the taper burnt down to 70 mm (A) or to a 38 mm stub (B)
const STICK_FLAME_ROOT: [f64; 2] = [0.133 + 0.069, 0.133 + 0.037];

// R489 brass candlestick with wax drips: 6% chance per level. A turned and
// cast brass candlestick (15.5 cm) of the common kind, its brass gone dull
// and brown, polished only where hands took it; its taper (a 7/8 in dinner
// candle) burnt down, the wax run over the socket's rim and onto the pan
// and the foot, the wick charred. Lit as the old one was: its flame a
// flame, its light the old light.
pub fn place_brass_candlesticks(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick_tile(&open);
    // the old three drips' lengths: the first says how far the taper has
    // burnt (a long stub or a short one)
    let short = rnd() >= 0.5;
    let _ = rnd();
    let _ = rnd();
    // the old wax pool's eight radii (the wax is the candle's own runs now)
    for _ in 0..8 {
        let _ = rnd();
    }
    let lit = rnd() < 0.60;
    let flicker = if lit { Some((rnd() * PI * 2.0, 6.0 + rnd() * 4.0)) } else { None };
    let px = g.wx(x) + (rnd() - 0.5) * 2.4;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.4;
    let turn = rnd() * PI * 2.0;
    let k = if short { "B" } else { "A" };
    let salt = salt(px, pz, 3);
    let stick = group();
    put(
        &stick,
        "Real__brassStickBody",
        &phong!({ map: &tarnish_tex(salt), color: 0x8a6a34, specular: 0x6a5430, shininess: 30 }),
    );
    put(&stick, &format!("Real__brassStickCandle{k}"), &lambert!({ map: &wax_tex(salt) }));
    put(&stick, &format!("Real__brassStickWick{k}"), &lambert!({ color: 0x1c1915 }));
    if let Some((flicker_phase, flicker_rate)) = flicker {
        let root = STICK_FLAME_ROOT[usize::from(short)];
        // the flame scales from its root as it flickers
        let flame = group();
        flame.position().y = root;
        if let Some(m) = put(
            &flame,
            &format!("Real__brassStickFlame{k}"),
            &basic_mat!({ map: &flame_tex(), transparent: true, depthWrite: false }),
        ) {
            m.position().y = -root;
            m.set_render_order(3);
        }
        stick.add(&flame);
        let glow = point_light!(0xffb060, 0.35, 0.9, 2);
        glow.position().y = root + 0.010;
        stick.add(&glow);
        g.animate(move |_g, _dt, t| {
            let p = t * flicker_rate + flicker_phase;
            let wobble = p.sin() * 0.5 + (p * 1.7).sin() * 0.3;
            glow.set_intensity(0.35 + wobble * 0.08);
            flame.scale().y = 1.0 + wobble * 0.12;
        });
    }
    stick.position().set(px, 0, pz);
    stick.rotation().y = turn;
    g.world.add(&stick);
}

/// Brass gone dull (grey on the brass's colour: how bright): brown tarnish
/// in patches, darker spots, a polished streak where it was held
fn tarnish_tex(salt: u32) -> Tex {
    canvas_tex_opt(64, 64, false, move |ctx, w, h| {
        let seed = salt ^ 0xb7a;
        fill(ctx, w, h, |u, v| {
            let n = fbm(seed, u * 6.0, v * 6.0, 3);
            let mut c = mix([0.42, 0.38, 0.33], [0.85, 0.82, 0.76], sstep(0.4, 0.75, n));
            let spot = vn(seed ^ 3, u * 40.0, v * 40.0);
            if spot > 0.72 {
                c = mix(c, [0.26, 0.24, 0.21], (spot - 0.72) * 3.0);
            }
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// The candle's wax (laid round it, v up): ivory gone grey with dust, the
/// runs a shade clearer, soot near its top
fn wax_tex(salt: u32) -> Tex {
    canvas_tex_opt(64, 128, false, move |ctx, w, h| {
        let seed = salt ^ 0x3a5;
        fill(ctx, w, h, |u, v| {
            let n = fbm(seed, u * 5.0, v * 12.0, 3);
            let mut c = mix([0.86, 0.82, 0.72], [0.78, 0.74, 0.64], n);
            c = mix(c, [0.40, 0.37, 0.33], sstep(0.88, 1.0, v) * 0.35);
            let dust = vn(seed ^ 5, u * 90.0, v * 90.0);
            if dust > 0.8 {
                c = mix(c, [0.62, 0.60, 0.55], (dust - 0.8) * 2.5);
            }
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// A candle flame (laid round it, v up its 27 mm): a dim blue root, the
/// bright yellow-white body, orange toward its tip, soft at its edge
fn flame_tex() -> Tex {
    thread_local! { static T: std::cell::OnceCell<Tex> = const { std::cell::OnceCell::new() }; }
    T.with(|c| {
        c.get_or_init(|| {
            canvas_tex_opt(16, 64, false, |ctx, w, h| {
                fill(ctx, w, h, |_u, v| {
                    let root = 1.0 - sstep(0.04, 0.2, v);
                    let tip = sstep(0.35, 1.0, v);
                    let c = mix([1.0, 0.86, 0.56], [1.0, 0.56, 0.22], tip);
                    let c = mix(c, [0.36, 0.40, 0.80], root);
                    let a = (0.85 - 0.5 * root) * (1.0 - 0.55 * sstep(0.7, 1.0, v));
                    [c[0], c[1], c[2], a]
                });
            })
        })
        .clone()
    })
}

/// The stack (`t09.py`, measured on the model): where a skull sits on one
/// (a pair) or across two side by side (three), and how far apart the two
/// below stand
const STACK_ON_ONE: f64 = 0.201;
const STACK_ON_TWO: f64 = 0.207;
const STACK_APART: f64 = 0.077;

// R488 skull stack cluster: 4% chance per level. Life-size cast resin skulls
// (the 19.6 cm kind sold as replicas), aged: an ivory resin under a brown
// wash that lies dark in the orbits, the nose and the ears, the teeth a
// yellower resin. Two, one set on the other's crown; or three, a pair side
// by side and one across them; each a little askew.
pub fn place_skull_stacks(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick_tile(&open);
    let n_skulls = 2 + (rnd() * 2.0).floor() as i32;
    // each skull's old offset, turn and (above the first) tilt: here how far
    // each is set off its place (a few mm), turned and tilted
    let mut each = vec![];
    for i in 0..n_skulls {
        let ox = (rnd() - 0.5) * 0.04;
        let oz = (rnd() - 0.5) * 0.04;
        let yaw = rnd() * PI * 2.0;
        let roll = if i > 0 { (rnd() - 0.5) * 0.4 } else { 0.0 };
        each.push((ox * 0.15, oz * 0.15, (yaw / (PI * 2.0) - 0.5) * 0.5, roll * 0.3));
    }
    let px = g.wx(x) + (rnd() - 0.5) * 2.0;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.0;
    let turn = rnd() * PI * 2.0;
    let places: &[(f64, f64)] = if n_skulls == 2 { &[(0.0, 0.0), (0.0, STACK_ON_ONE)] } else { &[(-STACK_APART, 0.0), (STACK_APART, 0.0), (0.0, STACK_ON_TWO)] };
    let bone = lambert!({ map: &bone_tex(salt(px, pz, 4)) });
    let wash = lambert!({ color: 0x3e3226 });
    let teeth = lambert!({ color: 0xbfae88 });
    let cluster = group();
    for (&(sx, sy), &(ox, oz, yaw, roll)) in places.iter().zip(each.iter()) {
        let skull = group();
        put(&skull, "Real__skullStackBone", &bone);
        put(&skull, "Real__skullStackHollows", &wash);
        put(&skull, "Real__skullStackTeeth", &teeth);
        // a tilted one sits a little higher on its rim
        skull.position().set(sx + ox, sy + roll.abs() * 0.06, oz);
        set_rot_yz(&skull, yaw, roll);
        cluster.add(&skull);
    }
    cluster.position().set(px, 0, pz);
    cluster.rotation().y = turn;
    g.world.add(&cluster);
}

fn set_rot_yz(o: &Obj, yaw: f64, roll: f64) {
    o.rotation().y = yaw;
    o.rotation().z = roll;
}

/// Aged resin (laid on the skull from every side): ivory under a brown
/// wash, the wash pooled darker in blotches and along the sutures' lines,
/// a little rubbed off on what stands out, dust
fn bone_tex(salt: u32) -> Tex {
    canvas_tex_opt(256, 256, false, move |ctx, w, h| {
        let seed = salt ^ 0x5c0;
        fill(ctx, w, h, |u, v| {
            let n = fbm(seed, u * 5.0, v * 5.0, 4);
            let mut c = mix([0.80, 0.74, 0.60], [0.66, 0.58, 0.44], n);
            // the wash: a brown that pooled in patches
            let wash = sstep(0.52, 0.72, fbm(seed ^ 3, u * 9.0, v * 9.0, 4));
            c = mix(c, [0.48, 0.39, 0.28], wash * 0.55);
            // the sutures: thin wandering lines the wash ran into
            let sut = (fbm(seed ^ 5, u * 4.0, v * 4.0, 3) - 0.5).abs();
            c = mix(c, [0.52, 0.43, 0.32], (1.0 - sstep(0.002, 0.008, sut)) * 0.35);
            // pores and specks
            let p = vn(seed ^ 7, u * 200.0, v * 200.0);
            if p > 0.84 {
                c = mix(c, [0.42, 0.36, 0.28], (p - 0.84) * 3.0);
            }
            let dust = vn(seed ^ 9, u * 140.0, v * 140.0);
            if dust > 0.86 {
                c = mix(c, [0.70, 0.68, 0.64], (dust - 0.86) * 3.0);
            }
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// The painting kit of batch `t09`'s painters (as batch `t08`'s): value
/// noise and a per-pixel fill (pure, so `par_rows` may run it on every
/// core), a salt from where a thing is, a part placed.
#[allow(dead_code)]
mod t09_kit {
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
