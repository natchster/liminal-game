//! What several core placers share.
use game::prelude::*;
use placers_kit::noise::hash;
use placers_kit::noise::sstep;

/// A soft round spot at `at`: `r` px, `k` times as tall as it is wide, turned
/// `turn`, its colour fading out from the middle (the context squashed, so
/// the fade is never cut by an edge).
pub(crate) fn soft(ctx: &mut Canvas, at: (f64, f64), r: f64, k: f64, turn: f64, rgba: (u8, u8, u8, f64)) {
    let (cr, cg, cb, a) = rgba;
    ctx.save();
    ctx.translate(at.0, at.1);
    ctx.rotate(turn);
    ctx.scale(1, k);
    let gr = ctx.create_radial_gradient(0, 0, 0, 0, 0, r);
    gr.add_color_stop(0, &format!("rgba({cr},{cg},{cb},{a:.3})"));
    gr.add_color_stop(1, &format!("rgba({cr},{cg},{cb},0)"));
    ctx.set_fill_style(&gr);
    ctx.begin_path();
    ctx.arc(0, 0, r, 0, 7);
    ctx.fill();
    ctx.restore();
}

/// A transparent mark `w × h` px painted by `paint` on a stream of its own,
/// varied by `vary` (a painter's own stream starts the same on every floor:
/// `vary` comes from where the mark was put); where it is clear its colour is
/// `bleed` (sRGB), so its smaller mips keep the mark's colour at its soft
/// edges instead of darkening them
pub(crate) fn mark_tex<F: FnOnce(&mut Canvas, f64, f64)>(w: f64, h: f64, bleed: [f64; 3], vary: u32, paint: F) -> Tex {
    let tex = canvas_tex_opt(w, h, false, move |ctx: &mut Canvas, cw: f64, ch: f64| {
        let seed = (rnd() * 4.0e9) as u32 ^ vary;
        with_own_stream(seed, || paint(ctx, cw, ch))
    });
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

/// A number that differs from one placing to the next (from where it was
/// put), to vary what its painter draws: a painter's own stream starts the
/// same on every floor
pub(crate) fn vary_by(a: i32, b: i32, c: i32) -> u32 {
    (hash(0x7a3c_51d9 ^ c as u32, a, b) * 4.0e9) as u32
}
