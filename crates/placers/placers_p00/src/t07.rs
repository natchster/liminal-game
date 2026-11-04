//! Batch `t07`'s smoking ashtray, made real (`docs/PROPS.md` P3): its
//! placer and what only it uses.
use game::prelude::*;
use t07_kit::*;

/// The burning cigarette in its rest (as batch `t07` models it: the rest
/// at 45 degrees, the cigarette tipped 10.6 degrees down to the ash)
const CIG_AZ: f64 = PI / 4.0;
const CIG_TILT: f64 = 0.185_005; // 10.6 degrees
const CIG_OUT: f64 = 0.0665;
const CIG_Y0: f64 = 0.0212 + 0.0039 + (0.0665 - 0.0485) * 0.187_174; // the filter end's axis
const CIG_EMBER_AT: f64 = 0.06065; // the ember ring, along it from the filter's end

/// Where the ember burns, in the ashtray's frame
fn ember_at() -> (f64, f64, f64) {
    let (c, s) = (CIG_TILT.cos(), CIG_TILT.sin());
    let r = CIG_OUT - CIG_EMBER_AT * c;
    (r * CIG_AZ.cos(), CIG_Y0 - CIG_EMBER_AT * s, r * CIG_AZ.sin())
}

// Smoking ashtray: 5% chance per level. A round pressed-glass ashtray
// on the floor (batch `t07`: the Libbey 5156 kind, 108 mm, four rests), ash
// and three butts in it, and a cigarette left burning in a rest, its ash
// grown long, a thread of smoke going up from it.
pub(crate) fn place_smoking_ashtrays(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    }
    let mut open_tiles: Vec<(i32, i32)> = vec![];
    for z in 1..g.h - 1 {
        for x in 1..g.w - 1 {
            if g.tiles[g.t_idx(x, z)] == 0 {
                open_tiles.push((x, z));
            }
        }
    }
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = open_tiles[(rnd() * open_tiles.len() as f64).floor() as usize];
    // the old glass colour draw: clear (its edge green), amber, smoke, blue
    let glass_colors = [0xd0d0e0, 0xc0a880, 0x808080, 0x606080];
    let gi = ((rnd() * glass_colors.len() as f64).floor() as usize).min(3);
    let tint = [0x9aa8a0u32, 0x8a6430, 0x55524e, 0x485262][gi];
    let glass = phong!({ color: tint, specular: 0x6a6a68, shininess: 90, transparent: true, opacity: 0.42, depthWrite: false });
    let salt = salt(g.wx(x), g.wz(z), gi as u32);
    let a = group();
    put(&a, "Real__ashtrayGlass", &glass);
    put(&a, "Real__ashtrayAsh", &lambert!({ map: &ash_tex(salt) }));
    let cork = cork_tex();
    put(&a, "Real__ashtrayButtFilters", &lambert!({ map: &cork, color: 0xd8d0c4 }));
    put(&a, "Real__ashtrayButtPapers", &lambert!({ color: 0xa8a296 }));
    put(&a, "Real__ashtrayButtChar", &lambert!({ color: 0x2c2a28 }));
    put(&a, "Real__ashtrayCigFilter", &lambert!({ map: &cork }));
    put(&a, "Real__ashtrayCigPaper", &lambert!({ color: 0xcfcac0 }));
    put(&a, "Real__ashtrayCigAsh", &lambert!({ color: 0x8a8781 }));
    // the ember: a ring of fire under its ash (it glows: a flame)
    let ember_mat = basic_mat!({ color: 0xd2501c });
    put(&a, "Real__ashtrayCigEmber", &ember_mat);
    let (ex, ey, ez) = ember_at();
    // its glow, faint, on the ash and the glass round it
    let ember_light = point_light!(0xff5a20, 0.003, 0.06, 2);
    ember_light.position().set(ex, ey + 0.004, ez);
    a.add(&ember_light);
    // the smoke: a thread going up from the burning end and breaking into
    // curls (two crossed sheets, painted), where the old two wisps were;
    // their draws: a nudge and a phase each
    let smoke_mat = lambert!({ map: &smoke_tex(salt), color: 0xd4d8de, transparent: true, depthWrite: false, side: DoubleSide });
    struct Wisp {
        pivot: Obj,
        phase: f64,
        turn: f64,
    }
    let mut wisps: Vec<Wisp> = vec![];
    for i in 0..2 {
        let nx = (rnd() - 0.5) * 0.01;
        let nz = (rnd() - 0.5) * 0.01;
        let phase = rnd() * PI * 2.0;
        let pivot = group();
        pivot.position().set(ex + nx * 0.2, ey + 0.0035, ez + nz * 0.2);
        let turn = i as f64 * PI / 2.0 + nx * 40.0;
        pivot.rotation().y = turn;
        let sheet = mesh(plane_geo!(0.07, 0.24), &smoke_mat);
        sheet.position().y = 0.12;
        pivot.add(&sheet);
        a.add(&pivot);
        wisps.push(Wisp { pivot, phase, turn });
    }
    let ember_base_intensity = 0.003;
    let ember_phase = rnd() * PI * 2.0;
    g.animate(move |_g, _dt, t| {
        let f = 0.85 + (t * 3.0 + ember_phase).sin() * 0.12 + (t * 7.0 + ember_phase * 1.7).sin() * 0.06;
        ember_light.set_intensity(ember_base_intensity * f);
        let k = (f - 0.67) / 0.36;
        ember_mat.set_color(((0xa0 as f64 + 0x5f as f64 * k) as u32) << 16 | ((0x30 as f64 + 0x30 as f64 * k) as u32) << 8 | 0x14);
        for w in &wisps {
            w.pivot.rotation().y = w.turn + (t * 0.31 + w.phase).sin() * 0.35;
            let s = 1.0 + (t * 0.5 + w.phase).sin() * 0.06;
            w.pivot.scale().set(1.0 + (t * 0.7 + w.phase).cos() * 0.15, s, 1.0);
        }
    });
    a.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    a.rotation().y = rnd() * PI * 2.0;
    g.world.add(&a);
}

/// The ash in the bowl, seen from above (u along x, v along -z): pale grey
/// fluff, darker where it burnt out, flecks of tobacco, a white bloom
fn ash_tex(salt: u32) -> Tex {
    canvas_tex_opt(128, 128, false, move |ctx, w, h| {
        let seed = salt ^ 0xa5;
        fill(ctx, w, h, |u, v| {
            let n = fbm(seed, u * 9.0, v * 9.0, 4);
            let mut c = mix([0.62, 0.61, 0.58], [0.40, 0.39, 0.37], n * 1.4 - 0.2);
            let speck = vn(seed ^ 3, u * 90.0, v * 90.0);
            if speck > 0.86 {
                c = mix(c, [0.16, 0.13, 0.10], (speck - 0.86) * 7.0);
            } else if speck < 0.08 {
                c = mix(c, [0.82, 0.81, 0.78], (0.08 - speck) * 9.0);
            }
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// Cork tipping paper: its printed mottle, a stain from the lips at one end
fn cork_tex() -> Tex {
    thread_local! { static T: std::cell::OnceCell<Tex> = const { std::cell::OnceCell::new() }; }
    T.with(|c| {
        c.get_or_init(|| {
            canvas_tex(64, 64, |ctx, w, h| {
                fill(ctx, w, h, |u, v| {
                    let n = fbm(77, u * 14.0, v * 14.0, 3);
                    let c = mix([0.66, 0.54, 0.40], [0.50, 0.38, 0.26], n * 1.6 - 0.3);
                    let fleck = vn(91, u * 48.0, v * 48.0);
                    let c = if fleck > 0.8 { mix(c, [0.80, 0.74, 0.62], (fleck - 0.8) * 5.0) } else { c };
                    [c[0], c[1], c[2], 1.0]
                });
            })
        })
        .clone()
    })
}

/// A thread of smoke off a cigarette left burning: straight and thin from
/// the burning end, then wavering, widening and curling as it rises, gone
/// by the top (transparent round it)
fn smoke_tex(salt: u32) -> Tex {
    canvas_tex_opt(64, 256, false, move |ctx, w, h| {
        for _ in 0..(salt % 53) {
            let _ = rnd();
        }
        ctx.clear_rect(0, 0, w, h);
        ctx.set_line_cap("round");
        let base = rnd() * 6.3;
        for k in 0..7 {
            let ph = base + k as f64 * 0.35 + rnd() * 0.3;
            let fr = 6.0 + rnd() * 3.0;
            let drift = (rnd() - 0.5) * 10.0;
            let (mut px, mut py) = (w / 2.0, h);
            let n = 48;
            for s in 1..=n {
                let t = s as f64 / n as f64;
                let amp = 0.5 + 15.0 * t.powf(1.3);
                let x = w / 2.0 + amp * (t * fr + ph).sin() + drift * t * t + 3.0 * t * (t * 23.0 + ph * 2.0).sin() * t;
                let y = h * (1.0 - t);
                let a = 0.30 * (1.0 - t).powf(1.7) * (t / 0.05).min(1.0);
                ctx.set_stroke_style(&format!("rgba(236,240,246,{a:.3})"));
                ctx.set_line_width(0.7 + 4.5 * t);
                ctx.begin_path();
                ctx.move_to(px, py);
                ctx.line_to(x, y);
                ctx.stroke();
                px = x;
                py = y;
            }
        }
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
