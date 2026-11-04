//! Batch `t07`'s opened suitcase, made real (`docs/PROPS.md` P3): its
//! placer and what only it uses (the suitcase and the clothes are r02's).
use super::*;
use t07_kit::{fill, mix, salt};

/// r02's case: its hinge line (height, back) and the lining's floor inside
const HINGE_Y: f64 = 0.112;
const HINGE_Z: f64 = -0.209;
const FLOOR: f64 = 0.0063;
/// How far its lid stands open, held by its check straps
const OPEN: f64 = PI / 2.0 + 0.21;

// R507 opened suitcase: 5% chance per level. An old hard-shell suitcase
// left open on the floor (r02's: a 1960s moulded case, 23 x 16 x 7 in, its
// lid stood up on its straps), gone through: what was packed in it pushed
// about, four to six things, a cardigan pulled half out over its rim.
pub(crate) fn place_opened_suitcases(g: &mut Game, _dist: &[i32]) {
    use game::assets::props::prop_part_as_modelled;
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    // the old leather draw: the shell's vinyl (saddle, oxblood, tan, black), dulled
    let leather_colors = [0x603820, 0x502010, 0x8a5030, 0x201a10];
    let lc = pick_hex(&leather_colors);
    let li = leather_colors.iter().position(|&c| c == lc).unwrap_or(0);
    let shell = lambert!({ color: [0x6e5038u32, 0x5a3029, 0x86684a, 0x2e2a26][li] });
    // the old cloths' colours: each thing's own palette, by the pick
    let cloth_colors = [0xd82040, 0x2050d8, 0xd8b040, 0x30a030, 0xa020a0, 0xf0ece0, 0x2a2418];
    const PALETTES: [[u32; 4]; 5] = [
        [0x38383a, 0x2a2e3a, 0x4a3c2e, 0x5e5a54], // trousers: charcoal, navy, brown, grey
        [0x9a8e76, 0x34463a, 0x5a2c2c, 0x6a6864], // jumper: oatmeal, bottle green, burgundy, grey
        [0xc4beb0, 0x9eaab4, 0xb8ae98, 0xbca49c], // shirt: white, pale blue, cream, faded pink
        [0x2c2a2a, 0x625e5a, 0x2a3040, 0x4a3c30], // socks
        [0x76623e, 0x5a2a2c, 0x2c3446, 0x56626a], // cardigan: mustard, maroon, navy, grey-blue
    ];
    // 4-6 things in it, each drawn as the old cloth was: colour, three
    // sizes, where (x, z), its turn and tilt
    let n_clothes = 4 + (rnd() * 3.0).floor() as i32;
    let mut things = vec![];
    for _i in 0..n_clothes {
        let cc = pick_hex(&cloth_colors);
        let k = cloth_colors.iter().position(|&c| c == cc).unwrap_or(0) % 4;
        let (sw, _sh, sd) = (rnd(), rnd(), rnd());
        let (px, pz) = (rnd() - 0.5, rnd() - 0.5);
        let (yaw, tilt) = (rnd(), rnd() - 0.5);
        things.push((k, sw, sd, px, pz, yaw, tilt));
    }
    let salt = salt(g.wx(x), g.wz(z), li as u32);
    let suitcase = group();
    let put_at = |parent: &Obj, name: &str, mat: &Mat, x: f64, y: f64, z: f64, yaw: f64| {
        if let Some(geo) = prop_part_as_modelled(name) {
            let m = mesh(geo, mat);
            m.position().set(x, y, z);
            m.rotation().y = yaw;
            parent.add(&m);
        }
    };
    let lining = lambert!({ map: &lining_tex(salt) });
    let trim = phong!({ color: 0x8a8a86, specular: 0x6a6a66, shininess: 50 });
    let grip = lambert!({ color: 0x2a231e });
    for (name, mat) in [
        ("Real__suitcaseBase", &shell),
        ("Real__suitcaseBaseLining", &lining),
        ("Real__suitcaseBaseTrim", &trim),
        ("Real__suitcaseHandle", &grip),
    ] {
        put_at(&suitcase, name, mat, 0.0, 0.0, 0.0, 0.0);
    }
    let lid = group();
    lid.position().set(0, HINGE_Y, HINGE_Z);
    lid.rotation().x = -OPEN;
    for (name, mat) in [("Real__suitcaseLid", &shell), ("Real__suitcaseLidLining", &lining), ("Real__suitcaseLidTrim", &trim)] {
        put_at(&lid, name, mat, 0.0, -HINGE_Y, -HINGE_Z, 0.0);
    }
    suitcase.add(&lid);
    // what is left in it, each about where it was packed and pushed about
    // by a hand going through it: trousers folded at the back, the jumper
    // at the front, a shirt on the trousers, socks in a corner; a fifth, a
    // cardigan pulled out over the rim; a sixth, another jumper on the first
    const SLOTS: [(&str, usize, f64, f64, f64); 6] = [
        ("Real__suitcaseTrousers", 0, -0.07, -0.085, 0.0),
        ("Real__suitcaseSweater", 1, 0.12, 0.085, 0.0),
        ("Real__suitcaseShirt", 2, -0.11, -0.08, 0.050),
        ("Real__suitcaseSocks", 3, -0.205, 0.12, 0.0),
        ("Real__suitcaseSpill", 4, 0.0, 0.0, 0.0),
        ("Real__suitcaseSweater", 4, 0.11, 0.08, 0.075),
    ];
    let flip = if things[0].6 > 0.0 { -1.0 } else { 1.0 };
    for (i, &(k, sw, sd, px, pz, yaw, _)) in things.iter().enumerate() {
        let (name, pal, sx, sz, sy) = SLOTS[i.min(5)];
        let cloth = lambert!({ color: PALETTES[pal][k] });
        if i == 4 {
            put_at(&suitcase, name, &cloth, px * 0.08, 0.0, 0.0, 0.0);
            continue;
        }
        let turn = (yaw - 0.5) * 0.16 + (sw - 0.5) * 0.04 + if i == 3 { 0.5 } else { 0.0 };
        put_at(&suitcase, name, &cloth, flip * (sx + px * 0.02), FLOOR + sy, sz + pz * 0.02 + (sd - 0.5) * 0.01, flip * turn);
    }
    scatter_on_floor(g, &suitcase, x, z, 1.8);
    g.world.add(&suitcase);
}

/// The lining: rayon gone the colour of weak tea, its stripe woven in,
/// tide marks where it got wet, grubby toward its corners and its rim
fn lining_tex(salt: u32) -> Tex {
    canvas_tex_opt(256, 256, false, move |ctx, w, h| {
        let seed = salt ^ 0x11e;
        fill(ctx, w, h, |u, v| {
            let stripe = if ((u * w) as i32 % 6) < 2 { 0.9 } else { 1.0 };
            let n = t07_kit::fbm(seed, u * 8.0, v * 8.0, 4);
            let mut c = mix([0.54, 0.49, 0.44], [0.44, 0.39, 0.34], n);
            c = [c[0] * stripe, c[1] * stripe, c[2] * stripe];
            for (cu, cv, r) in [(0.3, 0.6, 0.18), (0.72, 0.35, 0.12)] {
                let d = (u - cu).hypot(v - cv) / r + (t07_kit::fbm(seed ^ 3, u * 9.0, v * 9.0, 3) - 0.5) * 0.4;
                c = mix(c, [0.40, 0.33, 0.26], (1.0 - ((d - 1.0).abs() / 0.05)).max(0.0) * 0.5);
            }
            let edge = t07_kit::sstep(0.12, 0.0, u.min(1.0 - u).min(v.min(1.0 - v)));
            c = mix(c, [0.24, 0.19, 0.14], edge * 0.4);
            [c[0], c[1], c[2], 1.0]
        });
    })
}
/// The painting kit of batch `t07`'s painters: value noise and a per-pixel
/// fill (pure, so `par_rows` may run it on every core), soft spots.
#[allow(dead_code)]
mod t07_kit {
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
    /// sRGB 0..1 of a hex colour
    pub(super) fn rgb(c: u32) -> [f64; 3] {
        [((c >> 16) & 255) as f64 / 255.0, ((c >> 8) & 255) as f64 / 255.0, (c & 255) as f64 / 255.0]
    }
    pub(super) fn css(c: [f64; 3], a: f64) -> String {
        let k = |v: f64| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
        format!("rgba({},{},{},{:.3})", k(c[0]), k(c[1]), k(c[2]), a.clamp(0.0, 1.0))
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
    /// A soft spot at `at`, `r` px, `k` times as tall as wide, turned `turn`,
    /// fading out from its middle (the context squashed, so no edge cuts it)
    pub(super) fn soft(ctx: &mut Canvas, at: (f64, f64), r: f64, k: f64, turn: f64, c: [f64; 3], a: f64) {
        ctx.save();
        ctx.translate(at.0, at.1);
        ctx.rotate(turn);
        ctx.scale(1, k);
        let gr = ctx.create_radial_gradient(0, 0, 0, 0, 0, r);
        gr.add_color_stop(0, &css(c, a));
        gr.add_color_stop(1, &css(c, 0.0));
        ctx.set_fill_style(&gr);
        ctx.begin_path();
        ctx.arc(0, 0, r, 0, 7);
        ctx.fill();
        ctx.restore();
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
