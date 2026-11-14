//! Painting a canvas texture: per-pixel fills, soft blots, colours as the canvas takes them, decals and their materials.
use crate::noise::sstep;
use game::prelude::*;

/// Where a texture is clear, its colour becomes `rgb` (sRGB 0-1), so its
/// smaller mips keep the thing's colour at a cut-out edge instead of black
pub fn bleed(tex: &Tex, rgb: [f64; 3]) {
    let img = tex.image();
    let mut im = img.borrow_mut();
    for px in im.data.as_chunks_mut::<4>().0 {
        let k = sstep(0.0, 0.5, f64::from(px[3]) / 255.0);
        for (c, b) in px.iter_mut().zip(rgb) {
            *c = (b * 255.0 + (f64::from(*c) - b * 255.0) * k).round().clamp(0.0, 255.0) as u8;
        }
    }
    im.version += 1;
}

/// A colour (0-1) and an alpha as the canvas's `rgba(r,g,b,a)`, each channel clamped
pub fn css(c: [f64; 3], a: f64) -> String {
    let k = |v: f64| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
    format!("rgba({},{},{},{:.3})", k(c[0]), k(c[1]), k(c[2]), a.clamp(0.0, 1.0))
}

/// A transparent mark `tw` x `th` px across a plane `pw` x `ph` m: each
/// pixel `px(x, y, aa)` (m from the plane's top left, y down; `aa` a
/// pixel's size) in straight sRGB and alpha. Where it is clear its colour
/// is `bleed`, so a mark's edge does not filter toward black.
pub fn decal_tex(tw: usize, th: usize, pw: f64, ph: f64, bleed: [f64; 3], px: impl Fn(f64, f64, f64) -> [f64; 4] + Sync) -> Tex {
    let tex = canvas_tex_opt(tw as f64, th as f64, false, move |ctx, cw, ch| {
        let (sx, sy) = (pw / cw, ph / ch);
        let aa = sx.max(sy) * 1.1;
        let mut img = ctx.create_image_data(cw, ch);
        par_rows(&mut img.data, tw * 4, |j, out| {
            let y = (j as f64 + 0.5) * sy;
            for i in 0..tw {
                let x = (i as f64 + 0.5) * sx;
                let p = px(x, y, aa);
                for (c, v) in p.iter().enumerate() {
                    out[i * 4 + c] = (v.clamp(0.0, 1.0) * 255.0).round() as u8;
                }
            }
        });
        ctx.put_image_data(&img, 0, 0);
    });
    // what is clear (it came back black through the canvas) takes the
    // bleed colour, and what is nearly clear goes toward it
    {
        let img = tex.image();
        let mut im = img.borrow_mut();
        for p in im.data.as_chunks_mut::<4>().0 {
            let t = sstep(0.0, 0.06, f64::from(p[3]) / 255.0);
            for (c, b) in p.iter_mut().zip(bleed) {
                *c = (b * 255.0 + (f64::from(*c) - b * 255.0) * t).round().clamp(0.0, 255.0) as u8;
            }
        }
        im.version += 1;
    }
    tex
}

/// Fills the canvas pixel by pixel on every core: `f(u, v)` (u right, v
/// up, both 0..1, as the texture is laid) gives straight sRGB and alpha
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

/// `c` at alpha `a` over straight colour and alpha `dst`
pub fn lay(dst: &mut [f64; 4], c: [f64; 3], a: f64) {
    let a = a.clamp(0.0, 1.0);
    if a <= 0.0 {
        return;
    }
    let out = a + dst[3] * (1.0 - a);
    if out > 1e-6 {
        for k in 0..3 {
            dst[k] = (c[k] * a + dst[k] * dst[3] * (1.0 - a)) / out;
        }
    }
    dst[3] = out;
}

/// The decal material of a mark painted in `tex`, lit as the floor under it
pub fn mark_mat(tex: &Tex) -> Mat {
    lambert!({ map: tex, transparent: true, depthWrite: false, polygonOffset: true, polygonOffsetFactor: -2, polygonOffsetUnits: -2 })
}

/// A colour (or any three numbers) scaled by `k`
pub fn mul(a: [f64; 3], k: f64) -> [f64; 3] {
    [a[0] * k, a[1] * k, a[2] * k]
}

/// A colour with full alpha, as a fill's pixel
pub fn opaque(c: [f64; 3]) -> [f64; 4] {
    [c[0], c[1], c[2], 1.0]
}

/// Fill the canvas pixel by pixel: `f(x, y)` (pixel centres) gives a
/// straight sRGB colour and alpha. Pure (no draws, no canvas calls), so its
/// rows are filled on every core
pub fn paint_px(ctx: &mut Canvas, w: f64, h: f64, f: impl Fn(f64, f64) -> [f64; 4] + Sync) {
    let n = w as usize;
    let mut img = ctx.create_image_data(w, h);
    par_rows(&mut img.data, n * 4, |j, out| {
        for i in 0..n {
            let c = f(i as f64 + 0.5, j as f64 + 0.5);
            for k in 0..4 {
                out[i * 4 + k] = (c[k].clamp(0.0, 1.0) * 255.0).round() as u8;
            }
        }
    });
    ctx.put_image_data(&img, 0, 0);
}

/// A repeating `px` x `px` tile painted pixel by pixel by `f(u, v)`
pub fn paint_tile<F: Fn(f64, f64) -> [f64; 3] + Sync>(px: u32, f: F) -> Tex {
    canvas_tex_opt(px, px, true, move |ctx, w, h| {
        let w_ = w as usize;
        let mut img = ctx.create_image_data(w, h);
        par_rows(&mut img.data, w_ * 4, |j, out| {
            for i in 0..w_ {
                let c = f((i as f64 + 0.5) / w, (j as f64 + 0.5) / h);
                for k in 0..3 {
                    out[i * 4 + k] = (c[k].clamp(0.0, 1.0) * 255.0).round() as u8;
                }
                out[i * 4 + 3] = 255;
            }
        });
        ctx.put_image_data(&img, 0, 0);
    })
}

/// A colour (0xrrggbb) as fractions
pub fn rgb(c: u32) -> [f64; 3] {
    [f64::from((c >> 16) & 255) / 255.0, f64::from((c >> 8) & 255) / 255.0, f64::from(c & 255) / 255.0]
}

/// A colour in 0-255 and an alpha as the canvas's `rgba(r,g,b,a)`
pub fn rgba(c: (f64, f64, f64), a: f64) -> String {
    format!("rgba({:.0},{:.0},{:.0},{a:.3})", c.0, c.1, c.2)
}

/// A soft spot at `at`, `r` px, `k` times as tall as wide, turned `turn`,
/// fading out from its middle (the context squashed, so no edge cuts it)
pub fn soft(ctx: &mut Canvas, at: (f64, f64), r: f64, k: f64, turn: f64, c: [f64; 3], a: f64) {
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

/// Paints the canvas pixel by pixel: `f(i, j)` gives pixel (i, j)'s straight
/// sRGB and alpha (0..1)
pub fn paint_px_indexed(ctx: &mut Canvas, w: f64, h: f64, mut f: impl FnMut(usize, usize) -> [f64; 4]) {
    let (iw, ih) = (w as usize, h as usize);
    let mut img = ctx.create_image_data(w, h);
    for j in 0..ih {
        for i in 0..iw {
            let c = f(i, j);
            let k = (j * iw + i) * 4;
            for (n, v) in c.iter().enumerate() {
                img.data[k + n] = (v.clamp(0.0, 1.0) * 255.0).round() as u8;
            }
        }
    }
    ctx.put_image_data(&img, 0, 0);
}

/// A transparent mark `w × h` px painted by `paint` (on its own stream);
/// where it is clear its colour is `bleed` (sRGB), so its smaller mips keep
/// the mark's colour at its soft edges instead of darkening them
pub fn mark_tex<F: FnOnce(&mut Canvas, f64, f64)>(w: f64, h: f64, bleed: [f64; 3], paint: F) -> Tex {
    let tex = canvas_tex_opt(w, h, false, paint);
    {
        let img = tex.image();
        let mut im = img.borrow_mut();
        for px in im.data.as_chunks_mut::<4>().0 {
            let k = sstep(0.0, 0.3, f64::from(px[3]) / 255.0);
            for (c, b) in px.iter_mut().zip(bleed) {
                *c = (b * 255.0 + (f64::from(*c) - b * 255.0) * k).round().clamp(0.0, 255.0) as u8;
            }
        }
        im.version += 1;
    }
    tex
}
