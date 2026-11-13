//! Batch `t06`'s props in this crate, made real (`docs/design/PROPS.md` P3): the
//! garden shed's things (Blender batch `t06`; what each real thing is and
//! where its size comes from: `assets/src/blender/props/t06.notes.md`): a
//! bag of potting soil split at a corner, a canvas garden apron dropped on
//! the floor, a seed house's catalogue framed, a cast concrete bird bath
//! gone dry, a garden gnome, a few houseplant fertilizer spikes.
//! Each placer keeps its old draws call for call (the comments at each say
//! what a draw now chooses, or that it is kept for its place in the stream).
use crate::{face_yaw, open_tiles, pick, wall_tiles};
use game::assets::props::{prop_part_as_modelled, prop_part_info};
use game::prelude::*;
use game::visuals::pictures::{self, Finish, Outline, Window};

// ---------------------------------------------------------------- the kit
/// Adds the Blender part `name` as modelled to `grp` (nothing when there is
/// no such part)
fn put(grp: &Obj, name: &str, mat: &Mat) -> Option<Obj> {
    let geo = prop_part_as_modelled(name)?;
    let m = mesh(geo, mat);
    grp.add(&m);
    Some(m)
}

/// Adds part `name` with its texture coordinates made by `uv` (a copy)
fn put_uv(grp: &Obj, name: &str, mat: &Mat, uv: impl Fn(f64, f64, f64, [f64; 3]) -> (f64, f64)) -> Option<Obj> {
    let geo = prop_part_as_modelled(name)?.duplicate();
    {
        let mut gg = geo.g();
        let n = gg.position.len() / 3;
        let mut out = Vec::with_capacity(n * 2);
        for i in 0..n {
            let p = (f64::from(gg.position[i * 3]), f64::from(gg.position[i * 3 + 1]), f64::from(gg.position[i * 3 + 2]));
            let nn = [f64::from(gg.normal[i * 3]), f64::from(gg.normal[i * 3 + 1]), f64::from(gg.normal[i * 3 + 2])];
            let (u, v) = uv(p.0, p.1, p.2, nn);
            out.push(u as f32);
            out.push(v as f32);
        }
        gg.uv = out;
        gg.touch();
    }
    let m = mesh(geo, mat);
    grp.add(&m);
    Some(m)
}

/// From above: u along x over [x0, x1], v = 1 at z0 down to 0 at z1
fn top(x0: f64, x1: f64, z0: f64, z1: f64) -> impl Fn(f64, f64, f64, [f64; 3]) -> (f64, f64) {
    move |x, _y, z, _n| ((x - x0) / (x1 - x0), 1.0 - (z - z0) / (z1 - z0))
}

/// Laid on at its real scale (`size` m a repeat) from the side each vertex
/// faces most
fn world(size: f64) -> impl Fn(f64, f64, f64, [f64; 3]) -> (f64, f64) {
    move |x, y, z, n| {
        let (ax, ay, az) = (n[0].abs(), n[1].abs(), n[2].abs());
        let (u, v) = if ay >= ax && ay >= az {
            (x, z)
        } else if ax >= az {
            (z, y)
        } else {
            (x, y)
        };
        (u / size, v / size)
    }
}

/// A salt from where a thing stands (no draw): a painter's own stream
/// starts the same on every floor, so what it paints mixes this in
fn salt_of(x: f64, z: f64) -> u32 {
    let (a, b) = ((x * 1000.0).round() as i64 as u64, (z * 1000.0).round() as i64 as u64);
    let mut h = a.wrapping_mul(0x9E37_79B9_7F4A_7C15) ^ b.wrapping_mul(0xC2B2_AE3D_27D4_EB4F);
    h ^= h >> 29;
    (h ^ (h >> 32)) as u32
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

/// fbm that repeats every `p` (whole numbers) in x and y
fn fbm_tile(seed: u32, x: f64, y: f64, p: f64, oct: u32) -> f64 {
    let (u, v) = (x / p, y / p);
    let a = fbm(seed, x, y, oct);
    let b = fbm(seed, x - p, y, oct);
    let c = fbm(seed, x, y - p, oct);
    let d = fbm(seed, x - p, y - p, oct);
    let (fu, fv) = (u.fract(), v.fract());
    (a * (1.0 - fu) + b * fu) * (1.0 - fv) + (c * (1.0 - fu) + d * fu) * fv
}

fn sstep(e0: f64, e1: f64, x: f64) -> f64 {
    let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

fn mix(a: [f64; 3], b: [f64; 3], t: f64) -> [f64; 3] {
    let t = t.clamp(0.0, 1.0);
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
}

fn css(c: [f64; 3], a: f64) -> String {
    format!("rgba({:.0},{:.0},{:.0},{:.3})", c[0] * 255.0, c[1] * 255.0, c[2] * 255.0, a.clamp(0.0, 1.0))
}

fn hex(c: u32) -> [f64; 3] {
    [f64::from((c >> 16) & 255) / 255.0, f64::from((c >> 8) & 255) / 255.0, f64::from(c & 255) / 255.0]
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

/// What is painted already, changed pixel by pixel on every core: `f(u, v,
/// rgb)` (as [`fill`]) gives the new colour
fn over(ctx: &mut Canvas, w: f64, h: f64, f: impl Fn(f64, f64, [f64; 3]) -> [f64; 3] + Sync) {
    let iw = w as usize;
    let mut img = ctx.get_image_data(0, 0, w, h);
    par_rows(&mut img.data, iw * 4, |j, out| {
        for i in 0..iw {
            let c = [f64::from(out[i * 4]) / 255.0, f64::from(out[i * 4 + 1]) / 255.0, f64::from(out[i * 4 + 2]) / 255.0];
            let o = f((i as f64 + 0.5) / w, 1.0 - (j as f64 + 0.5) / h, c);
            for k in 0..3 {
                out[i * 4 + k] = (o[k].clamp(0.0, 1.0) * 255.0).round() as u8;
            }
            out[i * 4 + 3] = 255;
        }
    });
    ctx.put_image_data(&img, 0, 0);
}

/// A soft round blob, squashed by the context (never an ellipse: no rim)
#[allow(clippy::too_many_arguments)]
fn blob(ctx: &mut Canvas, x: f64, y: f64, r: f64, k: f64, turn: f64, c: [f64; 3], a: f64) {
    ctx.save();
    ctx.translate(x, y);
    ctx.rotate(turn);
    ctx.scale(1.0, k);
    let gr = ctx.create_radial_gradient(0, 0, 0, 0, 0, r);
    gr.add_color_stop(0, &css(c, a));
    gr.add_color_stop(1, &css(c, 0.0));
    ctx.set_fill_style(&gr);
    ctx.begin_path();
    ctx.arc(0, 0, r, 0, 7);
    ctx.fill();
    ctx.restore();
}

/// Paint over cast concrete, faded, chipped to the grey at its edges and
/// here and there, grimy toward the foot (v 0) and in its hollows: for the
/// gnome's parts, laid on by the part's own box
fn painted_tex(paint: [f64; 3], chips: f64, salt: u32) -> Tex {
    canvas_tex_opt(256, 256, false, move |ctx, w, h| {
        let seed = salt ^ 0x6e0e;
        fill(ctx, w, h, |u, v| {
            let n = fbm(seed, u * 9.0, v * 9.0, 4);
            let mut c = paint.map(|k| k * (0.9 + 0.2 * n));
            // faded where the light reached it
            c = mix(c, [0.62, 0.6, 0.56], 0.12 + 0.1 * fbm(seed ^ 3, u * 3.0, v * 3.0, 2));
            // chips: the paint gone to the grey cement, a dark rim round each
            let ch = fbm(seed ^ 5, u * 46.0, v * 46.0, 4) + 0.1 * sstep(0.12, 0.0, v.min(u).min(1.0 - u).min(1.0 - v));
            let edge = sstep(0.71 - chips * 0.05, 0.725 - chips * 0.05, ch);
            let rim = sstep(0.68 - chips * 0.05, 0.71 - chips * 0.05, ch) - edge;
            c = mix(c, c.map(|k| k * 0.6), rim * 0.7);
            c = mix(c, [0.5, 0.49, 0.46], edge);
            // grime: toward the foot and in specks
            c = mix(c, [0.2, 0.18, 0.15], sstep(0.35, 0.0, v) * 0.35 + (vn(seed ^ 7, u * 160.0, v * 160.0) - 0.82).max(0.0) * 2.5);
            [c[0], c[1], c[2], 1.0]
        });
    })
}

// ============================================================ soil bags
/// The bag's film (`t06.py` soil_bag): x across, z along, the seals' ends
const BAG_X: f64 = 0.1525;
const BAG_Z: f64 = 0.215;

/// The bag's print, laid on from above over the film (its top at -z):
/// BRIARWOOD (an invented brand) POTTING SOIL on a dark green film, a
/// photograph of flowering pot plants, the net contents; faded, scuffed,
/// dusty, creased, soil smeared out of the split at its +x, +z corner
fn soil_print_tex(salt: u32) -> Tex {
    canvas_tex_opt(320, 452, false, move |ctx, w, h| {
        let seed = salt ^ 0x5011;
        let film = hex(0x27402a);
        ctx.set_fill_style(&css(film, 1.0));
        ctx.fill_rect(0, 0, w, h);
        // the seals: crimped, a band of the print's green
        ctx.set_fill_style(&css(hex(0x1f3322), 1.0));
        ctx.fill_rect(0, 0, w, 24);
        ctx.fill_rect(0, h - 24.0, w, 24);
        ctx.set_stroke_style("rgba(255,255,255,0.07)");
        ctx.set_line_width(1);
        for k in 0..5 {
            for y0 in [4.0, h - 20.0] {
                ctx.begin_path();
                ctx.move_to(0, y0 + k as f64 * 3.5);
                ctx.line_to(w, y0 + k as f64 * 3.5);
                ctx.stroke();
            }
        }
        // the brand on a cream band
        ctx.set_fill_style(&css(hex(0xe6dec6), 1.0));
        ctx.fill_rect(0, 42, w, 50);
        ctx.set_fill_style(&css(hex(0x2a472c), 1.0));
        ctx.set_text_align("center");
        ctx.set_text_baseline("alphabetic");
        ctx.set_font("bold 36px Rockwell");
        ctx.fill_text_max("BRIARWOOD", w / 2.0, 80, w - 60.0);
        // a sprig each side of it
        for sx in [-1.0, 1.0] {
            let cx = w / 2.0 + sx * 136.0;
            ctx.set_stroke_style(&css(hex(0x2a472c), 1.0));
            ctx.set_line_width(2);
            ctx.begin_path();
            ctx.move_to(cx, 84);
            ctx.line_to(cx, 52);
            ctx.stroke();
            for k in 0..3 {
                blob(ctx, cx + sx * 5.0, 58.0 + k as f64 * 8.0, 6.0, 0.45, sx * 0.6, hex(0x2a472c), 1.0);
                blob(ctx, cx - sx * 5.0, 62.0 + k as f64 * 8.0, 6.0, 0.45, -sx * 0.6, hex(0x2a472c), 1.0);
            }
        }
        ctx.set_fill_style(&css(hex(0xf0e9d4), 1.0));
        ctx.set_font("bold 62px Futura");
        ctx.fill_text_max("POTTING", w / 2.0, 156, w - 22.0);
        ctx.fill_text_max("SOIL", w / 2.0, 214, w - 22.0);
        ctx.set_fill_style(&css(hex(0xc8bf62), 1.0));
        ctx.set_font("bold 15px Futura");
        ctx.fill_text_max("ALL PURPOSE  \u{b7}  INDOOR & OUTDOOR PLANTS", w / 2.0, 238, w - 34.0);
        // the photograph: pot plants in flower on a bench, in a greenhouse
        let (px, py, pw, ph) = (32.0, 250.0, w - 64.0, 132.0);
        ctx.save();
        ctx.begin_path();
        ctx.rect(px, py, pw, ph);
        ctx.clip();
        let gr = ctx.create_linear_gradient(0, py, 0, py + ph);
        gr.add_color_stop(0, &css(hex(0xb7c49a), 1.0));
        gr.add_color_stop(0.6, &css(hex(0x5d7a4a), 1.0));
        gr.add_color_stop(1, &css(hex(0x6a5444), 1.0));
        ctx.set_fill_style(&gr);
        ctx.fill_rect(px, py, pw, ph);
        let mut r = SplitMix64::new(7, 0x50f0);
        // the greenhouse behind, out of focus
        for _ in 0..40 {
            let (x, y) = (px + r.next() * pw, py + r.next() * ph * 0.6);
            blob(ctx, x, y, r.range(8.0, 22.0), 1.0, 0.0, mix(hex(0x6f8c56), hex(0xd7dcb4), r.next()), 0.35);
        }
        // three pots on the bench, their plants and flowers
        for (k, cx) in [px + 52.0, px + 128.0, px + 202.0].into_iter().enumerate() {
            let base = py + ph - 8.0;
            let (tw, bw, pot_h) = (34.0, 24.0, 30.0);
            let pg = ctx.create_linear_gradient(cx - tw, 0, cx + tw, 0);
            pg.add_color_stop(0, &css(hex(0x7a3e24), 1.0));
            pg.add_color_stop(0.4, &css(hex(0xc0704a), 1.0));
            pg.add_color_stop(1, &css(hex(0x6a3420), 1.0));
            ctx.set_fill_style(&pg);
            ctx.begin_path();
            ctx.move_to(cx - tw, base - pot_h);
            ctx.line_to(cx + tw, base - pot_h);
            ctx.line_to(cx + bw, base);
            ctx.line_to(cx - bw, base);
            ctx.close_path();
            ctx.fill();
            ctx.fill_rect(cx - tw - 3.0, base - pot_h - 6.0, (tw + 3.0) * 2.0, 7);
            // leaves
            for _ in 0..26 {
                let a = r.range(-2.9, -0.25);
                let d = r.range(6.0, 40.0);
                blob(
                    ctx,
                    cx + a.cos() * d,
                    base - pot_h - 6.0 + a.sin() * d * 0.75,
                    r.range(6.0, 11.0),
                    0.6,
                    a,
                    mix(hex(0x2c4a22), hex(0x6f9a46), r.next()),
                    0.85,
                );
            }
            // the flowers: geranium heads, petunias
            let fl = [hex(0xc0283a), hex(0xd8a0c0), hex(0xe8e0d0)][k];
            for _ in 0..9 {
                let a = r.range(-2.7, -0.4);
                let d = r.range(14.0, 38.0);
                let (fx, fy) = (cx + a.cos() * d, base - pot_h - 10.0 + a.sin() * d * 0.8);
                for _ in 0..5 {
                    blob(
                        ctx,
                        fx + r.range(-5.0, 5.0),
                        fy + r.range(-4.0, 4.0),
                        r.range(3.0, 5.5),
                        1.0,
                        0.0,
                        mix(fl, [1.0, 1.0, 1.0], r.next() * 0.3),
                        0.9,
                    );
                }
            }
        }
        ctx.restore();
        // a thin white keyline round the photograph
        ctx.set_stroke_style("rgba(240,233,212,0.9)");
        ctx.set_line_width(2);
        ctx.stroke_rect(px, py, pw, ph);
        // the starburst
        ctx.set_fill_style(&css(hex(0xd9b640), 1.0));
        ctx.begin_path();
        let (sx, sy) = (w - 58.0, 262.0);
        for k in 0..32 {
            let rr = if k % 2 == 0 { 36.0 } else { 29.0 };
            let a = k as f64 / 32.0 * PI * 2.0;
            if k == 0 {
                ctx.move_to(sx + a.cos() * rr, sy + a.sin() * rr);
            } else {
                ctx.line_to(sx + a.cos() * rr, sy + a.sin() * rr);
            }
        }
        ctx.close_path();
        ctx.fill();
        ctx.set_fill_style(&css(hex(0x2a472c), 1.0));
        ctx.set_font("bold 12px Futura");
        ctx.fill_text_max("ENRICHED", sx, sy - 2.0, 56);
        ctx.set_font("10px Futura");
        ctx.fill_text_max("WITH PEAT", sx, sy + 11.0, 56);
        ctx.set_fill_style(&css(hex(0xf0e9d4), 1.0));
        ctx.set_font("bold 22px Futura");
        ctx.fill_text_max("8 DRY QT (8.8 L)", w / 2.0, 412, w - 50.0);
        // worn: scuffs and creases where the film folded and was dragged
        let mut s = SplitMix64::new(salt, 0x5cf);
        ctx.set_line_cap("round");
        for _ in 0..70 {
            let (x, y) = (s.next() * w, s.next() * h);
            let a = s.range(-0.6, 0.6) + if s.next() < 0.5 { 0.0 } else { PI / 2.0 };
            let l = s.range(6.0, 40.0);
            ctx.set_stroke_style(&format!("rgba(220,215,200,{:.2})", s.range(0.05, 0.22)));
            ctx.set_line_width(s.range(0.6, 1.8));
            ctx.begin_path();
            ctx.move_to(x, y);
            ctx.line_to(x + a.cos() * l, y + a.sin() * l);
            ctx.stroke();
        }
        for _ in 0..7 {
            let (x, y) = (s.next() * w, s.range(30.0, h - 30.0));
            let a = s.range(0.0, PI);
            let l = s.range(40.0, 140.0);
            for (st, off) in [("rgba(0,0,0,0.25)", 1.2), ("rgba(255,255,240,0.18)", -0.6)] {
                ctx.set_stroke_style(st);
                ctx.set_line_width(1.4);
                ctx.begin_path();
                ctx.move_to(x + off, y + off);
                ctx.quadratic_curve_to(
                    x + a.cos() * l * 0.5 + s.range(-8.0, 8.0),
                    y + a.sin() * l * 0.5,
                    x + a.cos() * l + off,
                    y + a.sin() * l + off,
                );
                ctx.stroke();
            }
        }
        // the split at the bottom right corner: soil smeared out of it
        for _ in 0..30 {
            blob(
                ctx,
                w - s.range(4.0, 60.0),
                h - s.range(4.0, 46.0),
                s.range(4.0, 14.0),
                s.range(0.4, 1.0),
                s.range(0.0, 3.0),
                hex(0x2a1f17),
                s.range(0.3, 0.8),
            );
        }
        ctx.set_stroke_style("rgba(12,9,6,0.95)");
        ctx.set_line_width(3);
        ctx.begin_path();
        ctx.move_to(w - 52.0, h - 26.0);
        ctx.line_to(w - 30.0, h - 30.0);
        ctx.line_to(w - 14.0, h - 22.0);
        ctx.line_to(w - 4.0, h - 34.0);
        ctx.stroke();
        // faded by years of light, dust settled in a film, darker toward the
        // floor at its edges
        over(ctx, w, h, |u, v, c| {
            let n = fbm(seed, u * 6.0, v * 8.0, 3);
            let mut o = mix(c, [0.48, 0.5, 0.44], 0.22 + 0.12 * n);
            o = mix(o, [0.56, 0.53, 0.47], 0.1 + 0.18 * fbm(seed ^ 9, u * 14.0, v * 18.0, 3));
            let speck = (vn(seed ^ 4, u * 260.0, v * 360.0) - 0.84).max(0.0) * 3.0;
            o = mix(o, [0.7, 0.67, 0.6], speck);
            let edge = sstep(0.06, 0.0, u.min(1.0 - u));
            mix(o, [0.16, 0.13, 0.1], edge * 0.4)
        });
    })
}

/// Soil out of a bag, from above over the heap (200 x 165 mm): dark peat,
/// crumbs and fibres of bark, the white grains of perlite; dried paler on
/// top in patches
fn soil_tex(salt: u32) -> Tex {
    canvas_tex_opt(256, 256, false, move |ctx, w, h| {
        let seed = salt ^ 0x0e17;
        fill(ctx, w, h, |u, v| {
            // peat: near black-brown, dried a dusty grey-brown in patches
            let n = fbm_tile(seed, u * 8.0, v * 8.0, 8.0, 4);
            let mut c = mix([0.1, 0.08, 0.065], [0.27, 0.22, 0.18], n);
            let dry = fbm_tile(seed ^ 5, u * 3.0, v * 3.0, 3.0, 2);
            c = mix(c, [0.34, 0.3, 0.26], sstep(0.55, 0.75, dry) * 0.45);
            let bark = (vn(seed ^ 2, u * 40.0, v * 40.0) - 0.75).max(0.0) * 3.0;
            c = mix(c, [0.3, 0.18, 0.1], bark);
            let perl = (vn(seed ^ 3, u * 70.0, v * 70.0) - 0.92).max(0.0) * 9.0;
            c = mix(c, [0.74, 0.73, 0.68], perl.min(1.0));
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// Soil scattered on the floor round the spilled heap (its middle at the
/// split): fine soil in a soft drift, crumbs flung further, clear beyond
fn soil_scatter_tex(salt: u32) -> Tex {
    canvas_tex_opt(256, 224, false, move |ctx, w, h| {
        ctx.clear_rect(0, 0, w, h);
        let mut r = SplitMix64::new(salt, 0x5ca7);
        // the drift: soft blobs thinning out from the split (at the top left)
        for _ in 0..60 {
            let a = r.range(-0.2, 1.7);
            let d = r.next().powf(0.7) * 0.75;
            let (x, y) = (w * 0.3 + a.cos() * d * w * 0.6, h * 0.25 + a.sin() * d * h * 0.7);
            blob(
                ctx,
                x,
                y,
                r.range(8.0, 26.0) * (1.0 - 0.6 * d),
                r.range(0.5, 1.0),
                r.range(0.0, 3.0),
                hex(0x1e1914),
                r.range(0.15, 0.4) * (1.0 - d),
            );
        }
        // crumbs and bits of bark, perlite
        for _ in 0..140 {
            let a = r.range(-0.4, 1.9);
            let d = r.next().powf(0.6);
            let (x, y) = (w * 0.3 + a.cos() * d * w * 0.65, h * 0.25 + a.sin() * d * h * 0.75);
            // a grain of perlite now and then, dulled with the dust
            let c = if r.next() < 0.05 { [0.62, 0.6, 0.56] } else { mix(hex(0x161210), hex(0x3a2e24), r.next()) };
            ctx.set_fill_style(&css(c, r.range(0.6, 0.95)));
            ctx.begin_path();
            ctx.arc(x, y, r.range(0.6, 2.2), 0, 7);
            ctx.fill();
        }
    })
}

// Bag of soil: 6% chance per level.
pub fn place_soil_bags(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    // An 8 dry quart bag of potting soil lying on its back where it was put
    // down, slumped, faded and dusty, split at a corner where soil has run
    // out onto the floor (Blender batch `t06`)
    let bag = group();
    // where it lies (the old draws) and how it is turned (the old draw)
    bag.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    bag.rotation().y = rnd() * PI * 2.0;
    let p = bag.pos();
    let salt = salt_of(p.x, p.z);
    let print = phong!({ map: &soil_print_tex(salt), specular: 0x1c1c1a, shininess: 18 });
    put_uv(&bag, "Real__soilBag", &print, top(-BAG_X, BAG_X, -BAG_Z, BAG_Z));
    put_uv(&bag, "Real__soilBagSeals", &print, top(-BAG_X, BAG_X, -BAG_Z, BAG_Z));
    // the soil run out of the split: peat is matte (no sheen to grey it
    // under a lamp), its map laid on from above over the heap
    put_uv(&bag, "Real__soilBagSpill", &phong!({ map: &soil_tex(salt), specular: 0x0c0c0c, shininess: 6 }), top(0.02, 0.22, 0.15, 0.315));
    // the fine soil and crumbs scattered round the heap
    let scatter = mesh(
        plane_geo!(0.34, 0.3),
        lambert!({ map: &soil_scatter_tex(salt), transparent: true, depthWrite: false, polygonOffset: true, polygonOffsetFactor: -2, polygonOffsetUnits: -2 }),
    );
    scatter.rotation().x = -PI / 2.0;
    scatter.position().set(0.175, 0.0012, 0.25);
    bag.add(&scatter);
    g.world.add(&bag);
}

// ========================================================= garden aprons
/// The canvas of the apron, laid on from above over its heap: the duck's
/// colour gone pale where the light fell and with washing, its weave,
/// soil worked into it in smears and the pockets' mouths, a water stain
fn apron_tex(base: [f64; 3], salt: u32) -> Tex {
    canvas_tex_opt(384, 384, false, move |ctx, w, h| {
        let seed = salt ^ 0xa960;
        let mut r = SplitMix64::new(salt, 0xa7);
        let stains: Vec<(f64, f64, f64)> = (0..7).map(|_| (r.next(), r.next(), r.range(0.03, 0.1))).collect();
        let ring = (r.range(0.25, 0.75), r.range(0.25, 0.75), r.range(0.08, 0.16));
        fill(ctx, w, h, |u, v| {
            let (x, y) = (u * 0.34, v * 0.34);
            // the weave: 18 threads a cm, a fine cross of light and dark
            let wv = ((x * 1800.0 * PI).sin() * (y * 1700.0 * PI).sin()) * 0.5;
            let n = fbm(seed, u * 20.0, v * 20.0, 4);
            let mut c = base.map(|k| k * (0.9 + 0.16 * n + 0.04 * wv));
            // faded in patches
            c = mix(c, [0.66, 0.62, 0.54], 0.12 + 0.2 * fbm(seed ^ 1, u * 4.0, v * 4.0, 3));
            // soil: smears with ragged edges
            let mut dirt = 0.0f64;
            for &(sx, sy, sr) in &stains {
                let d = ((u - sx).hypot(v - sy) / sr) + 0.5 * (fbm(seed ^ 2, u * 30.0, v * 30.0, 3) - 0.5);
                dirt = dirt.max(sstep(1.0, 0.4, d));
            }
            c = mix(c, [0.24, 0.19, 0.14], dirt * 0.55);
            // a dried water stain: its tide line darker than its middle
            let d = (u - ring.0).hypot(v - ring.1) / ring.2 + 0.25 * (fbm(seed ^ 3, u * 12.0, v * 12.0, 2) - 0.5);
            c = mix(c, c.map(|k| k * 0.82), sstep(1.0, 0.95, d) * 0.6 + sstep(0.9, 1.0, d) * sstep(1.06, 1.0, d) * 0.5);
            // dust
            c = mix(c, [0.62, 0.6, 0.55], 0.08 + (vn(seed ^ 4, u * 300.0, v * 300.0) - 0.8).max(0.0) * 2.0);
            [c[0], c[1], c[2], 1.0]
        });
    })
}

// Garden apron: 5% chance per level.
pub fn place_garden_aprons(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    // A gardener's waist apron of cotton duck, dropped on the floor in a
    // heap, one end folded back over it, its ties loose (Blender batch `t06`,
    // draped by Blender's cloth). The old colour's draw chooses its canvas:
    // olive drab, a brown duck, khaki, the natural, a faded denim blue
    let apron_colors: [u32; 5] = [0x408040, 0x604040, 0x808030, 0xa07040, 0x506080];
    let canvas = match pick(&apron_colors) {
        0x408040 => 0x5f6448,
        0x604040 => 0x6e5640,
        0x808030 => 0x8a7e5c,
        0xa07040 => 0xb0a282,
        _ => 0x56667a,
    };
    let apron = group();
    // where it lies and how it is turned (the old draws)
    apron.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    apron.rotation().y = rnd() * PI * 2.0;
    let p = apron.pos();
    let salt = salt_of(p.x, p.z);
    let cloth = lambert!({ map: &apron_tex(hex(canvas), salt), side: DoubleSide });
    let lay = || top(-0.17, 0.17, -0.17, 0.17);
    put_uv(&apron, "Real__apronGardenCloth", &cloth, lay());
    put_uv(&apron, "Real__apronGardenPocket", &cloth, lay());
    // the ties: cotton twill tape dyed to match, as dirty as the rest
    put_uv(&apron, "Real__apronGardenTies", &cloth, lay());
    g.world.add(&apron);
}

// ========================================================== seed catalogs
/// The seed catalogue's frame: its face from the outer edge to the sight
/// (d, z), as `seed_cat_frame` in `t06.py`
static SC_FACE: [(f64, f64); 8] = [
    (0.0, 0.0145),
    (0.0012, 0.0168),
    (0.0040, 0.0182),
    (0.0080, 0.0187),
    (0.0120, 0.0180),
    (0.0150, 0.0160),
    (0.0178, 0.0122),
    (0.020, 0.0112),
];
const SC_SIZE: (f64, f64) = (0.220, 0.300);
const SC_MAT: (f64, f64) = (0.195, 0.275);

/// The cover of a seed house's catalogue of 1937 (the house invented),
/// printed in colour like the chromolithographed covers of the time:
/// tomatoes on the vine, a cabbage, carrots and pea pods under the title;
/// the paper gone yellow, the inks faded, spotted with foxing
fn seed_cover_tex(vary: u32) -> Tex {
    canvas_tex_opt(456, 687, false, move |ctx, w, h| {
        let seed = vary ^ 0x5eed;
        let paper = hex(0xab9a72);
        ctx.set_fill_style(&css(paper, 1.0));
        ctx.fill_rect(0, 0, w, h);
        let ink = hex(0x1e281a);
        let red = hex(0x96281e);
        // the border: a double rule and corner squares
        ctx.set_stroke_style(&css(ink, 1.0));
        ctx.set_line_width(3);
        ctx.stroke_rect(30, 30, w - 60.0, h - 60.0);
        ctx.set_line_width(1);
        ctx.stroke_rect(37, 37, w - 74.0, h - 74.0);
        // the house and the title
        ctx.set_fill_style(&css(ink, 1.0));
        ctx.set_text_align("center");
        ctx.set_text_baseline("alphabetic");
        ctx.set_font("bold 22px Copperplate");
        ctx.fill_text_max("WARBECK'S", w / 2.0, 92, w - 120.0);
        ctx.set_fill_style(&css(red, 1.0));
        ctx.set_font("bold 54px Times New Roman");
        ctx.fill_text_max("Garden Book", w / 2.0, 146, w - 100.0);
        ctx.set_fill_style(&css(ink, 1.0));
        ctx.set_font("italic 20px Times New Roman");
        ctx.fill_text_max("Vegetable \u{b7} Flower \u{b7} Field Seeds", w / 2.0, 176, w - 110.0);
        // the year in a banner
        ctx.set_fill_style(&css(hex(0x3f6a3a), 1.0));
        ctx.fill_rect(w / 2.0 - 60.0, 188, 120, 34);
        ctx.set_fill_style(&css(paper, 1.0));
        ctx.set_font("bold 28px Times New Roman");
        ctx.fill_text_max("1937", w / 2.0, 215, 110);
        // the picture: a still life of the garden's yield
        let mut r = SplitMix64::new(11, 0xc0f);
        let outline = |ctx: &mut Canvas| {
            ctx.set_stroke_style("rgba(52,32,20,0.7)");
            ctx.set_line_width(1.2);
            ctx.stroke();
        };
        // the cabbage, back right
        let (cx, cy) = (w * 0.66, 360.0);
        for k in 0..14 {
            let a = k as f64 / 14.0 * PI * 2.0;
            let rr = 70.0 - (k % 3) as f64 * 8.0;
            let (lx, ly) = (cx + a.cos() * rr * 0.5, cy + a.sin() * rr * 0.4);
            let gr = ctx.create_radial_gradient(lx - 10.0, ly - 12.0, 4, lx, ly, 52);
            gr.add_color_stop(0, &css(hex(0x94b06a), 1.0));
            gr.add_color_stop(1, &css(hex(0x34582a), 1.0));
            ctx.set_fill_style(&gr);
            ctx.begin_path();
            ctx.ellipse(lx, ly, 46, 34, a, 0, 7);
            ctx.fill();
            outline(ctx);
        }
        let gr = ctx.create_radial_gradient(cx - 12.0, cy - 16.0, 4, cx, cy, 48);
        gr.add_color_stop(0, &css(hex(0xc2d090), 1.0));
        gr.add_color_stop(1, &css(hex(0x6a8a4a), 1.0));
        ctx.set_fill_style(&gr);
        ctx.begin_path();
        ctx.arc(cx, cy, 44, 0, 7);
        ctx.fill();
        outline(ctx);
        ctx.set_stroke_style("rgba(240,245,220,0.55)");
        ctx.set_line_width(1.5);
        for k in 0..9 {
            let a = k as f64 / 9.0 * PI * 2.0 + 0.3;
            ctx.begin_path();
            ctx.move_to(cx + a.cos() * 8.0, cy + a.sin() * 8.0);
            ctx.quadratic_curve_to(
                cx + (a + 0.3).cos() * 40.0,
                cy + (a + 0.3).sin() * 34.0,
                cx + (a + 0.2).cos() * 66.0,
                cy + (a + 0.2).sin() * 52.0,
            );
            ctx.stroke();
        }
        // carrots across the front, their tops feathery
        for k in 0..4 {
            let (x0, y0) = (90.0 + k as f64 * 26.0, 470.0 + k as f64 * 10.0);
            let (x1, y1) = (x0 + 190.0, y0 + 40.0 - k as f64 * 6.0);
            let (nx, ny) = (-(y1 - y0), x1 - x0);
            let l = nx.hypot(ny);
            let (nx, ny) = (nx / l * 13.0, ny / l * 13.0);
            let gr = ctx.create_linear_gradient(x0 + nx, y0 + ny, x0 - nx, y0 - ny);
            gr.add_color_stop(0, &css(hex(0xd8842e), 1.0));
            gr.add_color_stop(1, &css(hex(0x94461a), 1.0));
            ctx.set_fill_style(&gr);
            ctx.begin_path();
            ctx.move_to(x0 + nx, y0 + ny);
            ctx.quadratic_curve_to((x0 + x1) / 2.0 + nx * 0.7, (y0 + y1) / 2.0 + ny * 0.7, x1, y1);
            ctx.quadratic_curve_to((x0 + x1) / 2.0 - nx * 0.7, (y0 + y1) / 2.0 - ny * 0.7, x0 - nx, y0 - ny);
            ctx.close_path();
            ctx.fill();
            outline(ctx);
            ctx.set_stroke_style("rgba(120,60,20,0.5)");
            ctx.set_line_width(1);
            for j in 1..8 {
                let t = j as f64 / 9.0;
                let (mx, my) = (x0 + (x1 - x0) * t, y0 + (y1 - y0) * t);
                let s = 1.0 - t;
                ctx.begin_path();
                ctx.move_to(mx + nx * 0.6 * s, my + ny * 0.6 * s);
                ctx.line_to(mx + nx * 0.2 * s, my + ny * 0.2 * s);
                ctx.stroke();
            }
            ctx.set_stroke_style(&css(hex(0x4a7a34), 1.0));
            ctx.set_line_width(1.6);
            for j in 0..7 {
                let a = PI + 0.9 + j as f64 * 0.12 + r.range(-0.05, 0.05);
                ctx.begin_path();
                ctx.move_to(x0, y0);
                ctx.quadratic_curve_to(x0 + a.cos() * 30.0, y0 + a.sin() * 30.0 - 10.0, x0 + a.cos() * 60.0, y0 + a.sin() * 56.0);
                ctx.stroke();
            }
        }
        // tomatoes on the vine, left
        for (k, (tx, ty, tr)) in
            [(130.0, 330.0, 40.0), (190.0, 352.0, 34.0), (112.0, 400.0, 36.0), (174.0, 418.0, 30.0)].into_iter().enumerate()
        {
            let gr = ctx.create_radial_gradient(tx - tr * 0.35, ty - tr * 0.4, tr * 0.1, tx, ty, tr);
            gr.add_color_stop(0, &css(hex(0xd8806a), 1.0));
            gr.add_color_stop(0.35, &css(hex(0xb8341f), 1.0));
            gr.add_color_stop(1, &css(hex(0x641410), 1.0));
            ctx.set_fill_style(&gr);
            ctx.begin_path();
            ctx.ellipse(tx, ty, tr, tr * 0.86, 0.1 * k as f64, 0, 7);
            ctx.fill();
            outline(ctx);
            // the calyx
            for j in 0..5 {
                let a = j as f64 / 5.0 * PI * 2.0;
                blob(ctx, tx + a.cos() * 7.0, ty - tr * 0.8 + a.sin() * 3.0, 7.0, 0.35, a, hex(0x3f6a2e), 1.0);
            }
            blob(ctx, tx - tr * 0.4, ty - tr * 0.45, tr * 0.25, 0.6, -0.5, [1.0, 0.95, 0.9], 0.6);
        }
        // pea pods
        for k in 0..3 {
            let (x0, y0) = (250.0 + k as f64 * 20.0, 520.0 - k as f64 * 14.0);
            ctx.set_fill_style(&css(mix(hex(0x6a9a46), hex(0x9cc06a), k as f64 / 3.0), 1.0));
            ctx.begin_path();
            ctx.move_to(x0, y0);
            ctx.quadratic_curve_to(x0 + 50.0, y0 - 26.0, x0 + 110.0, y0 - 8.0);
            ctx.quadratic_curve_to(x0 + 54.0, y0 + 6.0, x0, y0);
            ctx.fill();
            outline(ctx);
            for j in 0..5 {
                let t = 0.2 + j as f64 * 0.14;
                blob(ctx, x0 + 110.0 * t, y0 - 12.0 * (1.0 - (2.0 * t - 1.0).powi(2)) - 2.0, 6.0, 1.0, 0.0, hex(0xc2dc8c), 0.5);
            }
        }
        // the house's name at the foot
        ctx.set_fill_style(&css(ink, 1.0));
        ctx.set_font("bold 17px Copperplate");
        ctx.fill_text_max("WARBECK SEED CO.", w / 2.0, 598, w - 120.0);
        ctx.set_font("italic 15px Times New Roman");
        ctx.fill_text_max("Columbus, Ohio \u{b7} Established 1884", w / 2.0, 622, w - 120.0);
        // aged: the paper yellowed and browned toward its edges, the inks
        // faded toward it, foxing, a tide line where it once got wet
        let mut s = SplitMix64::new(vary, 0xf0c5);
        let fox: Vec<(f64, f64, f64)> = (0..(s.range(18.0, 40.0) as usize)).map(|_| (s.next(), s.next(), s.range(0.002, 0.009))).collect();
        let tide = (s.range(0.0, 1.0), s.range(0.0, 0.3), s.range(0.18, 0.35));
        over(ctx, w, h, move |u, v, c| {
            let n = fbm(seed, u * 7.0, v * 10.0, 4);
            let mut o = mix(c, paper, 0.08 + 0.08 * n);
            o = mix(o, [0.64, 0.5, 0.3], 0.12 + 0.1 * n);
            let edge = sstep(0.09, 0.0, u.min(1.0 - u).min(v.min(1.0 - v)));
            o = mix(o, [0.5, 0.37, 0.22], edge * 0.6);
            for &(fx, fy, fr) in &fox {
                let d = (u - fx).hypot((v - fy) * 1.5) / fr;
                o = mix(o, [0.58, 0.42, 0.26], sstep(1.0, 0.3, d) * 0.6);
            }
            let d = (u - tide.0).hypot(v - tide.1) / tide.2 + 0.2 * (fbm(seed ^ 6, u * 9.0, v * 9.0, 2) - 0.5);
            o = mix(o, o.map(|k| k * 0.86), sstep(1.0, 0.96, d) * 0.4 + sstep(0.93, 1.0, d) * sstep(1.04, 1.0, d) * 0.5);
            o
        });
    })
}

// Framed vintage seed catalogue: 3% chance per level.
pub fn place_seed_catalogs(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    } // 3% chance
    let walls = wall_tiles(g);
    if walls.is_empty() {
        return;
    }
    let (x, z, dx, dz) = pick(&walls);
    // A seed catalogue's cover of 1937 (9 x 6 in) framed in a stained pine
    // moulding with a cream mat and glass (Blender batch `t06`), its paper
    // gone yellow and foxed, the glass dusty
    let catalog = group();
    let mount_y = 1.65;
    let body = group();
    // the frame's back on the wall (0.013 behind the old box's middle)
    body.position().set(0, mount_y, -0.013);
    catalog.add(&body);
    // askew on its nail (the old draw)
    catalog.rotation().z = (rnd() - 0.5) * 0.05;
    catalog.rotation().y = face_yaw(dx, dz);
    catalog.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - 0.013), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - 0.013));
    let p = catalog.pos();
    let vary = pictures::vary_of(p.x, p.z, 0.47);
    put(
        &body,
        "Real__seedCatFrame",
        &lambert!({ map: &pictures::moulding_tex(SC_SIZE, &SC_FACE, Outline::Rect, Finish::Wood([0x6a, 0x48, 0x2c]), 1100.0, vary) }),
    );
    put(
        &body,
        "Real__seedCatMat",
        &lambert!({ map: &pictures::mat_tex(SC_MAT, Window::Rect(0.136, 0.212, 0.0), [0xb8, 0xaa, 0x88], vary, None) }),
    );
    put(&body, "Real__seedCatCover", &lambert!({ map: &seed_cover_tex(vary) }));
    put(&body, "Real__seedCatBack", &lambert!({ color: 0x4a3c2e }));
    // the glass: a little dust, the room faint in it
    let glass = pictures::glass_mat(vary, 0.05);
    glass.r().patch_uniforms.borrow_mut().f[0] = 0.08;
    put(&body, "Real__seedCatGlass", &glass);
    g.world.add(&catalog);
}

// ============================================================ bird baths
/// The bird bath's basin as modelled (`t06.py`): its dish's lowest point
/// inside and the dish's section near its middle (r, y)
const BB_DISH_Y: f64 = 0.470;
/// The dry dish's water: a puddle 9 mm deep at its middle
const BB_WATER_Y: f64 = BB_DISH_Y + 0.009;
const BB_WATER_R: f64 = 0.093;

/// Cast concrete round and up a pedestal or the basin's outside (u round,
/// v up): its aggregate, weathered grey, lichen and black algae in
/// patches, dark streaks run down from the basin, green toward the foot
fn concrete_tex(salt: u32, streaks: bool) -> Tex {
    canvas_tex_opt(384, 256, false, move |ctx, w, h| {
        let seed = salt ^ 0xc0c0;
        fill(ctx, w, h, |u, v| {
            // u wraps: the noise blends across the seam
            let wrap = |sd: u32, fu: f64, fv: f64, o: u32| fbm(sd, u * fu, v * fv, o) * (1.0 - u) + fbm(sd, (u - 1.0) * fu, v * fv, o) * u;
            let n = wrap(seed, 24.0, 16.0, 4);
            let mut c = mix([0.38, 0.37, 0.34], [0.54, 0.53, 0.49], n);
            // the aggregate and the pits where air was trapped, at the scale
            // the map holds (2 mm a pixel round the column)
            let agg = (vn(seed ^ 2, u * 160.0, v * 110.0) - 0.7).max(0.0) * 2.0;
            c = mix(c, [0.3, 0.28, 0.25], agg);
            let pit = (vn(seed ^ 3, u * 190.0, v * 125.0) - 0.9).max(0.0) * 5.0;
            c = mix(c, [0.16, 0.15, 0.14], pit.min(1.0));
            // lichen: pale grey-green rosettes; black algae in the damp
            let li = wrap(seed ^ 4, 18.0, 12.0, 3);
            c = mix(c, [0.62, 0.64, 0.52], sstep(0.62, 0.7, li) * 0.7);
            let al = wrap(seed ^ 5, 10.0, 8.0, 3);
            c = mix(c, [0.12, 0.13, 0.1], sstep(0.6, 0.72, al) * 0.6 * (0.4 + 0.6 * (1.0 - v)));
            if streaks {
                // run-off streaks down from the top
                let st = wrap(seed ^ 6, 40.0, 1.5, 2);
                c = mix(c, c.map(|k| k * 0.62), sstep(0.55, 0.7, st) * sstep(0.2, 1.0, v));
            }
            // green toward the foot where it stood damp
            c = mix(c, [0.24, 0.27, 0.18], sstep(0.25, 0.0, v) * 0.45);
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// The basin from above (its 410 mm): grey concrete, the dried crust of
/// green-black algae over its floor, tide lines where the water stood and
/// fell, a dark film round the puddle still in it, dust on its rim
fn bowl_tex(salt: u32) -> Tex {
    canvas_tex_opt(512, 512, false, move |ctx, w, h| {
        let seed = salt ^ 0xb0b1;
        fill(ctx, w, h, |u, v| {
            let (x, y) = (u - 0.5, v - 0.5);
            let r = x.hypot(y) * 0.41; // m from the middle
            let n = fbm(seed, u * 30.0, v * 30.0, 4);
            let mut c = mix([0.38, 0.37, 0.34], [0.52, 0.51, 0.47], n);
            let agg = (vn(seed ^ 2, u * 220.0, v * 220.0) - 0.7).max(0.0) * 2.0;
            c = mix(c, [0.32, 0.3, 0.27], agg);
            // the algae's crust over the dish's floor, ragged at its edge
            let crust = sstep(0.16, 0.12, r + 0.02 * (fbm(seed ^ 3, u * 14.0, v * 14.0, 3) - 0.5));
            let ac = mix([0.16, 0.18, 0.12], [0.3, 0.33, 0.22], fbm(seed ^ 4, u * 60.0, v * 60.0, 3));
            c = mix(c, ac, crust * 0.85);
            // tide lines where the water stood and dried: wavering, broken,
            // strong here and gone there, a paler crust just outside each
            let (ca, sa) = (y.atan2(x).cos(), y.atan2(x).sin());
            for (k, tr) in [0.121, 0.142, 0.163].into_iter().enumerate() {
                let sd = seed ^ (10 + k as u32);
                let wob = 0.009 * (fbm(sd, ca * 2.5 + 7.0, sa * 2.5, 3) - 0.5) + 0.003 * (fbm(sd ^ 1, u * 22.0, v * 22.0, 2) - 0.5);
                let d = r - tr - wob;
                let on = sstep(0.4, 0.62, fbm(sd ^ 2, ca * 1.8 + 3.0, sa * 1.8 + 3.0, 3));
                c = mix(c, [0.22, 0.21, 0.17], sstep(0.003, 0.0, d.abs()) * on * (0.45 - 0.1 * k as f64));
                c = mix(c, [0.6, 0.58, 0.52], sstep(0.004, 0.0, (d - 0.0035).abs()) * on * 0.2);
            }
            // the rim: dusty, droppings long dried
            c = mix(c, [0.62, 0.6, 0.55], sstep(0.18, 0.2, r) * 0.25);
            let drop = (vn(seed ^ 7, u * 120.0, v * 120.0) - 0.9).max(0.0) * 8.0 * sstep(0.175, 0.19, r);
            c = mix(c, [0.78, 0.76, 0.7], drop.min(1.0) * 0.8);
            [c[0], c[1], c[2], 1.0]
        });
    })
}

// ============================================================ dead leaves
/// Batch `t05`'s dead leaves (`t05.py` LEAVES; `placers_p03`'s `t05.rs`
/// paints them so, and its painter is copied here): the part, the blade's
/// length, its envelope's half width, oak or beech
const LEAVES: [(&str, f64, f64, bool); 5] = [
    ("Real__leafBeechA", 0.072, 0.0225, false),
    ("Real__leafBeechB", 0.062, 0.0195, false),
    ("Real__leafBeechC", 0.056, 0.0180, false),
    ("Real__leafOakA", 0.086, 0.0265, true),
    ("Real__leafOakB", 0.070, 0.0220, true),
];

/// Each leaf's colour (sRGB): beech russet, beech brown, beech pale tan, oak
/// brown, oak dark (wet once)
const LEAF_RGB: [[f64; 3]; 5] = [[0.56, 0.37, 0.21], [0.48, 0.34, 0.22], [0.63, 0.49, 0.33], [0.54, 0.42, 0.28], [0.42, 0.32, 0.22]];

/// The bounds (frame) part `name` was modelled in
fn frame(name: &str) -> ([f64; 3], [f64; 3]) {
    prop_part_info(name).map(|m| (m.fmin, m.fmax)).unwrap_or(([-0.05; 3], [0.05; 3]))
}

/// The half width of leaf `k`'s blade at a fraction `a` of its length, on
/// side `side` (±1): beech ovate with a wavy margin, oak lobed, each a little
/// ragged, and one tear (as `t05` cuts them)
fn leaf_half(k: usize, a: f64, side: f64) -> f64 {
    let (_, _, w, oak) = LEAVES[k];
    if !(0.0..=1.0).contains(&a) {
        return 0.0;
    }
    let seed = 0x1ea0 + k as u32 * 17;
    let mut hw = if oak {
        let env = (PI * a.powf(0.62)).sin().max(0.0).powf(0.75) * (1.0 - 0.15 * (1.0 - a)) + 0.18 * (-((a - 0.03) / 0.05).powi(2)).exp();
        let t = (a * 5.2 + 0.2 + 0.08 * side).fract();
        let lobe = if a > 0.1 { 0.58 + 0.42 * (1.0 - (2.0 * t - 1.0).powi(2)).max(0.0).sqrt() } else { 1.0 };
        w * env * lobe
    } else {
        let env = (PI * a.powf(1.15)).sin().max(0.0).powf(0.8);
        w * env * (1.0 - 0.04 * (0.5 + 0.5 * (2.0 * PI * a * 8.5).cos()))
    } - 0.0006;
    hw *= 1.0 - 0.07 * sstep(0.55, 0.8, fbm(seed, a * 18.0, side * 3.0, 3));
    let ta = 0.35 + 0.3 * hash(seed, 1, 0);
    if side > 0.0 && k % 2 == 0 {
        let d = ((a - ta) / 0.025).abs();
        hw *= 1.0 - 0.7 * (1.0 - d).max(0.0);
    }
    hw.max(0.0)
}

/// Leaf `k` seen from above in its part's frame (as `t05` paints it): the
/// blade's outline cut out, holes eaten through, mottled, darker at the
/// dried margin, fungal spots, the veins paler; made once a thread
fn leaf_tex(k: usize) -> Tex {
    thread_local! { static T: RefCell<Vec<Option<Tex>>> = RefCell::new(vec![None; 5]); }
    let k = k.min(4);
    if let Some(t) = T.with(|t| t.borrow()[k].clone()) {
        return t;
    }
    let (name, l, _, oak) = LEAVES[k];
    let (lo, hi) = frame(name);
    let (ex, ez) = (hi[0] - lo[0], hi[2] - lo[2]);
    let th = (256.0 * ez / ex).round().max(32.0);
    let base = LEAF_RGB[k];
    let seed = 0x1ea7 + k as u32 * 31;
    let tex = canvas_tex_opt(256, th, false, move |ctx, w, h| {
        let px_m = ex / w;
        // pixel rows top down (z from lo[2]); `fill` gives v up
        fill(ctx, w, h, |u, v| {
            let x = l - (hi[0] - (lo[0] + u * ex));
            let z = lo[2] + (1.0 - v) * ez;
            let a = x / l;
            if x < 0.0004 {
                // the petiole
                let c = mix([0.34, 0.24, 0.15], [0.26, 0.18, 0.12], vn(seed, u * w * 0.3, v * h * 0.3));
                return [c[0], c[1], c[2], 1.0];
            }
            let side = if z > 0.0 { 1.0 } else { -1.0 };
            let hw = leaf_half(k, a, side);
            let edge = (hw - z.abs()) / px_m;
            let inside = (edge + 0.5).clamp(0.0, 1.0);
            let hole_n = fbm(seed ^ 0x40, x * 260.0, z * 260.0, 3);
            let hole = sstep(0.77, 0.79, hole_n);
            let skel = sstep(0.70, 0.77, hole_n) * (1.0 - hole);
            let m = fbm(seed ^ 0x11, x * 120.0, z * 120.0, 4);
            let mut c = mix(base, mix(base, [0.30, 0.22, 0.15], 0.6), sstep(0.45, 0.8, m));
            let margin = 1.0 - sstep(0.0, 0.004, hw - z.abs());
            c = mix(c, [0.30, 0.23, 0.17], 0.5 * margin);
            let sp = fbm(seed ^ 0x77, x * 300.0, z * 300.0, 2);
            c = mix(c, [0.24, 0.18, 0.13], 0.55 * sstep(0.76, 0.79, sp));
            let mid_w = 0.0007 - 0.0005 * a;
            let midrib = 1.0 - sstep(mid_w * 0.5, mid_w, z.abs());
            let (pairs, slope) = if oak { (6.0, 0.62) } else { (8.0, 0.9) };
            let q = (x - z.abs() * slope) / l * pairs - 0.35;
            let dq = (q - q.round()).abs() * l / pairs;
            let lateral =
                (1.0 - sstep(0.00018, 0.00042, dq)) * sstep(0.0, 0.02, q / pairs) * (1.0 - sstep(0.85, 1.0, z.abs() / hw.max(1e-6)));
            let net = (1.0 - sstep(0.0, 0.08, (vn(seed ^ 0x9, x * 900.0, z * 900.0) - 0.5).abs())) * 0.25;
            let vein = midrib.max(lateral * 0.8).max(net * 0.4);
            c = mix(c, mix(c, [0.78, 0.66, 0.50], 0.45), vein);
            c = mix(c, [0.62, 0.55, 0.45], 0.5 * skel);
            [c[0], c[1], c[2], inside * (1.0 - hole)]
        });
    });
    // where it is clear its colour is the leaf's, so its smaller mips keep
    // the leaf's colour at the cut edge instead of black
    {
        let img = tex.image();
        let mut im = img.borrow_mut();
        for px in im.data.chunks_exact_mut(4) {
            let a = sstep(0.0, 0.5, f64::from(px[3]) / 255.0);
            for (c, b) in px.iter_mut().zip(base) {
                *c = (b * 255.0 + (f64::from(*c) - b * 255.0) * a).round().clamp(0.0, 255.0) as u8;
            }
        }
        im.version += 1;
    }
    T.with(|t| t.borrow_mut()[k] = Some(tex.clone()));
    tex
}

/// Dead leaf `k` (one of `t05`'s), its back up or not, a shade darker or
/// lighter; its lowest point at the origin
fn dead_leaf(k: usize, back_up: bool, shade: f64) -> Obj {
    let leaf = group();
    let name = LEAVES[k.min(4)].0;
    if let Some(geo) = prop_part_as_modelled(name) {
        let tint = ((shade.clamp(0.0, 1.0) * 255.0) as u32) * 0x010101;
        let m = mesh(geo, lambert!({ map: &leaf_tex(k), color: tint, alphaTest: 0.5, side: DoubleSide }));
        if back_up {
            m.rotation().x = PI;
            m.position().y = frame(name).1[1];
        }
        leaf.add(&m);
    }
    leaf
}

/// The dish inside the basin (`t06.py` `_bowl_profile`): its height over
/// its lowest point and its slope at `r` m from the middle
fn bb_dish(r: f64) -> (f64, f64) {
    const P: [(f64, f64); 5] = [(0.0, 0.0), (0.07, 0.004), (0.12, 0.014), (0.16, 0.030), (0.18, 0.038)];
    for w in P.windows(2) {
        let ((r0, y0), (r1, y1)) = (w[0], w[1]);
        if r <= r1 {
            let s = (y1 - y0) / (r1 - r0);
            return (y0 + (r.max(r0) - r0) * s, s);
        }
    }
    (0.038, 0.4)
}

// Bird bath: 3% chance per level.
pub fn place_bird_baths(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    } // 3% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    // A small cast concrete pedestal bird bath (Blender batch `t06`), long
    // dry: lichen and black algae on it, its basin crusted, tide lines where
    // the water stood and fell, a last puddle of dark still water, dead
    // leaves blown in (`t05`'s beech and oak) in it and at its foot
    let bath = group();
    // where it stands and how it is turned (the old draws)
    bath.position().set(g.wx(x) + (rnd() - 0.5) * 1.6, 0, g.wz(z) + (rnd() - 0.5) * 1.6);
    bath.rotation().y = rnd() * PI * 2.0;
    let p = bath.pos();
    let salt = salt_of(p.x, p.z);
    put(&bath, "Real__birdBathPedestal", &lambert!({ map: &concrete_tex(salt, true) }));
    put(&bath, "Real__birdBathBowlOut", &lambert!({ map: &concrete_tex(salt ^ 0x77, false) }));
    // the basin inside: weathered concrete is matte (little sheen under a lamp)
    put_uv(
        &bath,
        "Real__birdBathBowlIn",
        &phong!({ map: &bowl_tex(salt), specular: 0x0e0e0e, shininess: 8 }),
        top(-0.205, 0.205, -0.205, 0.205),
    );
    // the water: stagnant, tea dark, its surface still (the lamp's
    // reflection in it small and sharp)
    let water = mesh(
        circle_geo!(BB_WATER_R, 40),
        phong!({ color: 0x2a2920, specular: 0x181816, shininess: 400, transparent: true, opacity: 0.92 }),
    );
    water.rotation().x = -PI / 2.0;
    water.position().y = BB_WATER_Y;
    bath.add(&water);
    // the dead leaves (which of `t05`'s, which side up, how dark: from
    // where it stands, no draw): one afloat, two dried on the dish's slope
    // (along it, lying on it), two on the floor at its foot
    let turn0 = f64::from(salt % 1000) / 1000.0 * PI * 2.0;
    let pickk = |i: u32| ((salt >> (3 * i)) % 5) as usize;
    let back = |i: u32| (salt >> (16 + i)) & 3 == 0;
    // afloat, wet and dark, near the middle
    let afloat = dead_leaf(pickk(0), back(0), 0.62);
    let (fx, fz) = (0.03 * turn0.cos(), 0.03 * turn0.sin());
    afloat.position().set(fx, BB_WATER_Y + 0.0004, fz);
    afloat.rotation().y = turn0;
    bath.add(&afloat);
    for (i, (r, a, turn)) in [(0.118, 2.2, 0.25), (0.106, 4.3, -0.35)].into_iter().enumerate() {
        let (h, slope) = bb_dish(r);
        let phi = turn0 + a;
        // the pivot's +z out along the slope, tipped up with it
        let pivot = group();
        pivot.set_rotation_order(EulerOrder::YXZ);
        pivot.position().set(r * phi.sin(), BB_DISH_Y + h + 0.0025, r * phi.cos());
        pivot.rotation().set(-slope.atan(), phi, 0.0);
        let l = dead_leaf(pickk(1 + i as u32), back(1 + i as u32), 0.82 + 0.1 * i as f64);
        l.rotation().y = turn;
        pivot.add(&l);
        bath.add(&pivot);
    }
    for (i, (r, a)) in [(0.19, 1.1), (0.235, 3.5)].into_iter().enumerate() {
        let phi = turn0 + a;
        let l = dead_leaf(pickk(3 + i as u32), back(3 + i as u32), 0.9 + 0.08 * i as f64);
        l.position().set(r * phi.sin(), 0.0, r * phi.cos());
        l.rotation().y = phi + 1.3 + i as f64 * 2.0;
        bath.add(&l);
    }
    g.world.add(&bath);
    // the old ripple's phase (its draw, kept), and its animation: the leaf
    // afloat drifts and turns a little on the still water in a draught
    let phase = rnd() * PI * 2.0;
    g.animate(move |_g, _dt, t| {
        let mut q = afloat.position();
        q.x = fx + (t * 0.07 + phase).cos() * 0.006;
        q.z = fz + (t * 0.05 + phase).sin() * 0.005;
        drop(q);
        afloat.rotation().y = turn0 + (t * 0.04 + phase).sin() * 0.4;
    });
}

// ============================================================ garden gnomes
// Garden gnome: 3% chance per level.
pub fn place_garden_gnomes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    } // 3% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    // A 12 in garden gnome of painted cast concrete standing on his rock
    // (Blender batch `t06`), the paint faded, chipped to the grey at every
    // edge, grimy at his foot. The old draw of his coat's colour chooses it:
    // a faded blue, a green, a brick red, a brown
    let body_colors: [u32; 4] = [0x2060a0, 0x408040, 0xa04040, 0x604020];
    let coat = pick(&body_colors);
    let (coat, trousers) = match coat {
        0x2060a0 => (0x4a6480, 0x5a4a38),
        0x408040 => (0x50684a, 0x5a4a38),
        0xa04040 => (0x7e4238, 0x46505a),
        _ => (0x664c36, 0x46505a),
    };
    let gnome = group();
    // where he stands and how he is turned (the old draws)
    gnome.position().set(g.wx(x) + (rnd() - 0.5) * 1.8, 0, g.wz(z) + (rnd() - 0.5) * 1.8);
    gnome.rotation().y = rnd() * PI * 2.0;
    let p = gnome.pos();
    let salt = salt_of(p.x, p.z);
    // he faces -z, as the old one did
    let body = group();
    body.rotation().y = PI;
    gnome.add(&body);
    let paint = |c: u32, chips: f64, k: u32| lambert!({ map: &painted_tex(hex(c), chips, salt ^ k) });
    put(&body, "Real__gnomeBase", &paint(0x6c6e5c, 1.2, 1));
    put(&body, "Real__gnomeBoots", &paint(0x2e2822, 1.0, 2));
    put(&body, "Real__gnomeTrousers", &paint(trousers, 0.6, 3));
    let coat_mat = paint(coat, 0.8, 4);
    put(&body, "Real__gnomeCoat", &coat_mat);
    put(&body, "Real__gnomeSleeves", &coat_mat);
    put(&body, "Real__gnomeBelt", &paint(0x2a2622, 0.8, 5));
    put(&body, "Real__gnomeBuckle", &paint(0x9a8448, 1.2, 6));
    put(&body, "Real__gnomeSkin", &paint(0xc49a80, 0.7, 7));
    put(&body, "Real__gnomeBeard", &paint(0xc8c2b2, 0.5, 8));
    put(&body, "Real__gnomeHat", &paint(0x8c3a30, 1.0, 9));
    put(&body, "Real__gnomeEyes", &phong!({ color: 0x1e1a18, specular: 0x3a3a38, shininess: 60 }));
    g.world.add(&gnome);
}

// ======================================================= fertilizer spikes
/// A spike's compressed granules: chalky, the granules showing, crumbled
/// darker where damp got at it (a repeat every 2 cm)
fn spike_tex(base: [f64; 3]) -> Tex {
    canvas_tex(64, 64, move |ctx, w, h| {
        fill(ctx, w, h, |u, v| {
            let n = fbm_tile(0x5b1e, u * 8.0, v * 8.0, 8.0, 3);
            let mut c = base.map(|k| k * (0.84 + 0.3 * n));
            let gr = (vn(0x5b1f, u * 32.0, v * 32.0) - 0.6).max(0.0) * 1.5;
            c = mix(c, c.map(|k| k * 0.7), gr);
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// The spike as modelled (`t06.py` SPIKE_L): its length, where its middle
/// is from its head, where a snapped one broke (0.58 of it)
const SPIKE_L: f64 = 0.052;
const SPIKE_MID: f64 = 0.0275;
const SPIKE_SNAP: f64 = 0.58 * SPIKE_L;

/// A spike (or what is left of it whole) lying in the cluster: its head
/// end and far end on the floor (x, z), the end it pivots on, the sine of
/// its tilt, its length
struct Laid {
    a0: (f64, f64),
    a1: (f64, f64),
    pivot: (f64, f64),
    sin: f64,
    len: f64,
}

impl Laid {
    /// The height of its top at `u` (0 the head end, 1 the far end): its
    /// thickness there (the octagon across its flats, tapering to the
    /// point) over its lift off the floor
    fn top(&self, u: f64) -> f64 {
        let t = u * self.len / SPIKE_L;
        let thick = if t < 0.58 { 0.0079 - 0.0009 * t } else { 0.0074 * (1.0 - 0.72 * ((t - 0.58) / 0.42).powf(1.15)) };
        let p = (self.a0.0 + (self.a1.0 - self.a0.0) * u, self.a0.1 + (self.a1.1 - self.a0.1) * u);
        let lift = (p.0 - self.pivot.0).hypot(p.1 - self.pivot.1) * self.sin;
        lift + thick
    }
}

/// Where segments a0-a1 and b0-b1 cross: the fraction along each
fn seg_cross(a0: (f64, f64), a1: (f64, f64), b0: (f64, f64), b1: (f64, f64)) -> Option<(f64, f64)> {
    let (rx, ry) = (a1.0 - a0.0, a1.1 - a0.1);
    let (sx, sy) = (b1.0 - b0.0, b1.1 - b0.1);
    let den = rx * sy - ry * sx;
    if den.abs() < 1e-12 {
        return None;
    }
    let (qx, qy) = (b0.0 - a0.0, b0.1 - a0.1);
    let t = (qx * sy - qy * sx) / den;
    let u = (qx * ry - qy * rx) / den;
    ((0.0..=1.0).contains(&t) && (0.0..=1.0).contains(&u)).then_some((t, u))
}

// Fertilizer spike cluster: 8% chance per level.
pub fn place_fertilizer_spikes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    // Houseplant fertilizer spikes out of one pack (Blender batch `t06`):
    // the first spike's colour draw chooses the pack's (the white houseplant
    // spikes, a pale grey-green, a pinkish grey, a cream); each later
    // spike's colour draw now chooses whether it lies whole, snapped in two
    // or crumbled
    let spike_colors: [u32; 4] = [0x408040, 0x2060c0, 0xc02020, 0xf0e020];
    let cluster = group();
    let count = 3 + (rnd() * 3.0).floor() as i32;
    let mut pack: Option<Mat> = None;
    let mut down: Vec<Laid> = vec![];
    for i in 0..count {
        let c = pick(&spike_colors);
        let kind = spike_colors.iter().position(|&k| k == c).unwrap_or(0);
        let mat = pack
            .get_or_insert_with(|| {
                let tone = match c {
                    0x408040 => 0xa4ac94,
                    0x2060c0 => 0xd0ccc0,
                    0xc02020 => 0xa89086,
                    _ => 0xc8bea6,
                };
                lambert!({ map: &spike_tex(hex(tone)) })
            })
            .clone();
        let whole = i == 0 || kind < 2;
        // where it lies and how it is turned (the old draws)
        let (sx, sz) = ((rnd() - 0.5) * 0.12, (rnd() - 0.5) * 0.12);
        let turn = rnd() * PI * 2.0;
        // its length along the floor (the spike's middle at the group's
        // origin): the head end, the far end of what lies whole
        let (x0, x1) = (-SPIKE_MID, if whole { SPIKE_L - SPIKE_MID } else { SPIKE_SNAP - SPIKE_MID });
        let dir = (turn.cos(), -turn.sin());
        let a0 = (sx + dir.0 * x0, sz + dir.1 * x0);
        let a1 = (sx + dir.0 * x1, sz + dir.1 * x1);
        // where it crosses one already down it rests on it: it pivots on its
        // end farther from the crossing, raised over the other's top there
        let mut best: Option<(f64, bool)> = None; // (sin of its tilt, pivot at the head)
        for d in &down {
            if let Some((t, u)) = seg_cross(a0, a1, d.a0, d.a1) {
                let len = x1 - x0;
                let at = t * len; // from the head end
                let top = d.top(u);
                let (dist, head) = if at > len / 2.0 { (at, true) } else { (len - at, false) };
                let k = (top / dist.max(0.004)).min(0.42);
                if best.is_none_or(|(b, _)| k > b) {
                    best = Some((k, head));
                }
            }
        }
        let spike = group();
        spike.position().set(sx, 0, sz);
        spike.rotation().y = turn;
        let tilt = group();
        spike.add(&tilt);
        let (sin, head) = best.unwrap_or((0.0, true));
        let pivot = if head { x0 } else { x1 };
        tilt.position().x = pivot;
        tilt.rotation().z = if head { sin.asin() } else { -sin.asin() };
        let inner = group();
        inner.position().x = -SPIKE_MID - pivot;
        tilt.add(&inner);
        // what broke off lies on the floor
        let apart = group();
        apart.position().x = -SPIKE_MID;
        spike.add(&apart);
        let size = 0.02;
        if whole {
            put_uv(&inner, "Real__fertSpike", &mat, world(size));
        } else if kind == 2 {
            put_uv(&inner, "Real__fertSpikeHead", &mat, world(size));
            put_uv(&apart, "Real__fertSpikeTip", &mat, world(size));
        } else {
            put_uv(&inner, "Real__fertSpikeHead", &mat, world(size));
            put_uv(&apart, "Real__fertCrumbs", &mat, world(size));
        }
        down.push(Laid { a0, a1, pivot: if head { a0 } else { a1 }, sin, len: x1 - x0 });
        cluster.add(&spike);
    }
    cluster.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    g.world.add(&cluster);
}
