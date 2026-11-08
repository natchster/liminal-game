//! Batch `r18`'s props in this crate, made real (`docs/PROPS.md` P2): its
//! placers and what only they use.
use super::*;

// Mop bucket: 6% chance per level. Plastic bucket with a mop in it.
pub(crate) fn place_mop_buckets(g: &mut Game, _dist: &[i32]) {
    // A 26 qt commercial mop bucket (batch `r18`): a moulded polyethylene tub
    // on four castors, a side-press wringer hooked over its end, a cotton wet
    // mop left standing in it, its handle leaning on the wringer; the water
    // gone grey with a scum on it, or dried to a crust and a ring
    // (r18.notes.md)
    use game::assets::props::prop_part_as_modelled;
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    // the old colour draw: blue, grey or red (the cleaners' colour code)
    let bucket_colors = [0x2050c0, 0x304050, 0xa02040];
    let c = pick_hex(&bucket_colors);
    let kind = bucket_colors.iter().position(|&k| k == c).unwrap_or(0);
    // water left in it (60%)
    let water = rnd() < 0.60;
    let salt = (x as u32).wrapping_mul(0x9e37_79b9) ^ (z as u32).wrapping_mul(0x85eb_ca6b) ^ kind as u32;
    let col = r18_mop::PLASTIC[kind];
    let bucket = group();
    let put = |name: &str, mat: &Mat| {
        if let Some(geo) = prop_part_as_modelled(name) {
            bucket.add(&mesh(geo, mat));
        }
    };
    let tub_tex = r18_mop::tub_tex(col, salt);
    put("Real__mopBucketTub", &phong!({ map: &tub_tex, specular: 0x161616, shininess: 16 }));
    put("Real__mopBucketInside", &phong!({ map: &r18_mop::inside_tex(col, water, salt), specular: 0x121212, shininess: 14 }));
    put("Real__mopBucketFloor", &lambert!({ map: &r18_mop::floor_tex(col, salt) }));
    put("Real__mopBucketWringer", &phong!({ map: &tub_tex, specular: 0x161616, shininess: 16 }));
    put("Real__mopBucketPlate", &phong!({ color: 0x5b5a56, specular: 0x141414, shininess: 14 }));
    put("Real__mopBucketLever", &phong!({ color: 0x75746f, specular: 0x5a5a56, shininess: 40 }));
    put("Real__mopBucketGrip", &phong!({ color: 0x1c1b1a, specular: 0x161616, shininess: 20 }));
    put("Real__mopBucketCastorForks", &phong!({ color: 0x7a7872, specular: 0x484844, shininess: 34 }));
    put("Real__mopBucketWheels", &lambert!({ color: 0x3d3b38 }));
    put("Real__mopBucketMopHandle", &phong!({ color: 0x8a6e4e, specular: 0x1c1a16, shininess: 22 }));
    put("Real__mopBucketMopClamp", &phong!({ color: 0x2c2b29, specular: 0x1a1a1a, shininess: 18 }));
    put("Real__mopBucketMopStrings", &lambert!({ map: &r18_mop::strings_tex(water, salt) }));
    if water {
        put(
            "Real__mopBucketWater",
            &phong!({ map: &r18_mop::water_tex(salt), specular: 0x3c3c3a, shininess: 70, transparent: true, opacity: 0.9, depthWrite: false }),
        );
    }
    scatter_on_floor(g, &bucket, x, z, 1.6);
    g.world.add(&bucket);
}

/// The mop bucket of `place_mop_buckets` (batch `r18`): its plastic and what
/// is painted on it.
mod r18_mop {
    use super::*;

    /// Its own stream starts the same on every floor: a placed thing's salt
    /// moves where it starts, so the strokes differ from floor to floor
    fn skip(salt: u32) {
        for _ in 0..(salt % 61) {
            let _ = rnd();
        }
    }

    /// Polyethylene gone dull: blue, grey, red (sRGB 0..1)
    pub(super) const PLASTIC: [[f64; 3]; 3] = [[0.2, 0.31, 0.5], [0.4, 0.41, 0.41], [0.55, 0.18, 0.16]];

    // ---- painting kit (per-pixel, pure: safe for `par_rows`) ----
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
    /// The two nearest distances to a jittered lattice (cells of 1)
    fn worley(seed: u32, x: f64, y: f64) -> (f64, f64) {
        let (mut d1, mut d2) = (9.0f64, 9.0f64);
        for dj in -1..=1 {
            for di in -1..=1 {
                let (ii, jj) = (x.floor() as i32 + di, y.floor() as i32 + dj);
                let fx = ii as f64 + hash(seed, ii, jj);
                let fy = jj as f64 + hash(seed ^ 77, ii, jj);
                let d = (fx - x).hypot(fy - y);
                if d < d1 {
                    d2 = d1;
                    d1 = d;
                } else if d < d2 {
                    d2 = d;
                }
            }
        }
        (d1, d2)
    }
    /// Fills the canvas pixel by pixel on every core: `f(u, v)` (u right,
    /// v up, both 0..1, as the texture is laid) gives straight sRGB and alpha
    fn fill(ctx: &mut Canvas, w: f64, h: f64, f: impl Fn(f64, f64) -> [f64; 4] + Sync) {
        let (iw, _) = (w as usize, h as usize);
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
    fn rgb(c: [f64; 3]) -> [f64; 4] {
        [c[0], c[1], c[2], 1.0]
    }
    fn css(c: [f64; 3], a: f64) -> String {
        format!("rgba({},{},{},{:.3})", (c[0] * 255.0) as u8, (c[1] * 255.0) as u8, (c[2] * 255.0) as u8, a)
    }

    /// The tub and the wringer outside (laid on each side as a box is):
    /// grime up from the floor, dirty water run down from the rim and dried
    /// in streaks, scuffs and the black of other things it hit
    pub(super) fn tub_tex(col: [f64; 3], salt: u32) -> Tex {
        canvas_tex_opt(256, 256, false, move |ctx, w, h| {
            skip(salt);
            let seed = salt ^ 0x70b;
            fill(ctx, w, h, |u, v| {
                let mut c = mix(col, [col[0] * 0.85, col[1] * 0.85, col[2] * 0.85], fbm(seed, u * 9.0, v * 9.0, 3));
                c = mix(c, [0.32, 0.3, 0.26], 0.15 + 0.2 * fbm(seed ^ 1, u * 4.0, v * 4.0, 3));
                let foot = sstep(0.35, 0.0, v) * (0.6 + 0.4 * fbm(seed ^ 2, u * 30.0, v * 40.0, 2));
                c = mix(c, [0.18, 0.16, 0.13], foot * 0.75);
                // the runs: narrow, from the rim down, fading
                let run = (fbm(seed ^ 3, u * 70.0, 0.5, 2) - 0.6).max(0.0) * 3.5;
                let reach = 0.25 + 0.5 * fbm(seed ^ 4, u * 20.0, 0.5, 1);
                let runs = run * sstep(1.0 - reach, 0.95, v) * (0.6 + 0.4 * vn(seed ^ 5, u * 300.0, v * 20.0));
                c = mix(c, [0.25, 0.24, 0.2], runs * 0.6);
                rgb(c)
            });
            for k in 0..120 {
                let (x, y) = (rnd() * w, h * (0.25 + 0.75 * rnd()));
                let a = (rnd() - 0.5) * 0.8;
                let l = 4.0 + rnd() * 20.0;
                ctx.set_stroke_style(
                    if k % 3 == 0 {
                        css([0.12, 0.11, 0.1], 0.4)
                    } else {
                        css([(col[0] + 0.2).min(1.0), (col[1] + 0.2).min(1.0), (col[2] + 0.2).min(1.0)], 0.3)
                    }
                    .as_str(),
                );
                ctx.set_line_width(0.6 + rnd() * 1.2);
                ctx.begin_path();
                ctx.move_to(x, y);
                ctx.line_to(x + a.cos() * l, y + a.sin() * l);
                ctx.stroke();
            }
        })
    }

    /// The tub's inside: grime, the tide where the water stood (with water,
    /// a scum ring just over it; dried, a hard ring and the film below)
    pub(super) fn inside_tex(col: [f64; 3], water: bool, salt: u32) -> Tex {
        canvas_tex_opt(256, 256, false, move |ctx, w, h| {
            let seed = salt ^ 0x1d5;
            // the water's level in the inside part's height (0.114 .. 0.362)
            let lv = (0.262 - 0.114) / (0.362 - 0.114) + if water { 0.0 } else { -0.1 - 0.15 * hash(seed, 3, 4) };
            fill(ctx, w, h, |u, v| {
                let mut c = [col[0] * 0.8, col[1] * 0.8, col[2] * 0.8];
                c = mix(c, [0.28, 0.26, 0.22], 0.25 + 0.2 * fbm(seed, u * 8.0, v * 8.0, 3));
                let d = v - lv - 0.01 * (fbm(seed ^ 2, u * 20.0, 0.5, 2) - 0.5);
                let below = sstep(0.004, -0.01, d);
                c = mix(c, [0.22, 0.2, 0.16], below * (0.55 + 0.25 * fbm(seed ^ 4, u * 40.0, v * 40.0, 2)));
                c = mix(c, [0.14, 0.12, 0.09], (-(d / 0.006).powi(2)).exp() * 0.7);
                c = mix(c, [0.5, 0.48, 0.42], (-((d - 0.012) / 0.008).powi(2)).exp() * 0.3);
                rgb(c)
            });
        })
    }

    /// The tub's bottom: silt and grit dried in it, a hair or a fibre
    pub(super) fn floor_tex(col: [f64; 3], salt: u32) -> Tex {
        canvas_tex_opt(128, 128, false, move |ctx, w, h| {
            let seed = salt ^ 0xf10;
            fill(ctx, w, h, |u, v| {
                let n = fbm(seed, u * 12.0, v * 12.0, 4);
                let mut c = mix([col[0] * 0.5, col[1] * 0.5, col[2] * 0.5], [0.24, 0.21, 0.17], 0.7);
                c = mix(c, [0.33, 0.3, 0.25], sstep(0.45, 0.6, n));
                let grit = (vn(seed ^ 1, u * 300.0, v * 300.0) - 0.8).max(0.0) * 4.0;
                c = mix(c, [0.12, 0.1, 0.08], grit);
                rgb(c)
            });
        })
    }

    /// The water left standing: grey-brown, a scum of dust and soap on it in
    /// drifts, lint and a hair floating, clearer where nothing gathered
    pub(super) fn water_tex(salt: u32) -> Tex {
        canvas_tex_opt(128, 128, false, move |ctx, w, h| {
            skip(salt);
            let seed = salt ^ 0x3a7e;
            fill(ctx, w, h, |u, v| {
                let n = fbm(seed, u * 6.0, v * 6.0, 4);
                let mut c = mix([0.16, 0.15, 0.13], [0.22, 0.21, 0.18], n);
                let scum = sstep(0.5, 0.68, fbm(seed ^ 1, u * 10.0, v * 10.0, 4));
                c = mix(c, [0.42, 0.4, 0.36], scum * 0.55);
                rgb(c)
            });
            ctx.set_line_width(0.7);
            for _ in 0..14 {
                let (x, y) = (rnd() * w, rnd() * h);
                let a = rnd() * PI * 2.0;
                ctx.set_stroke_style(css([0.5, 0.48, 0.44], 0.5).as_str());
                ctx.begin_path();
                ctx.move_to(x, y);
                ctx.quadratic_curve_to(
                    x + 6.0 * a.cos() + 4.0 * rnd(),
                    y + 6.0 * a.sin(),
                    x + 12.0 * a.cos(),
                    y + 12.0 * a.sin() + 3.0 * rnd(),
                );
                ctx.stroke();
            }
        })
    }

    /// The mop's cotton: off-white gone grey-brown, darker toward its ends
    /// and where it lay in the water, the yarns' twist in streaks
    pub(super) fn strings_tex(water: bool, salt: u32) -> Tex {
        canvas_tex_opt(128, 128, false, move |ctx, w, h| {
            let seed = salt ^ 0x5717;
            fill(ctx, w, h, |u, v| {
                let n = fbm(seed, u * 30.0, v * 30.0, 3);
                let streak = vn(seed ^ 1, u * 90.0, v * 8.0);
                let mut c = mix([0.5, 0.47, 0.4], [0.62, 0.59, 0.52], n);
                c = mix(c, [0.36, 0.33, 0.27], streak * 0.35);
                let low = sstep(0.6, 0.0, v);
                c = mix(c, if water { [0.22, 0.2, 0.16] } else { [0.34, 0.31, 0.26] }, low * 0.75);
                rgb(c)
            });
        })
    }
}
