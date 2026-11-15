//! What several nature placers share.
use game::prelude::*;
use placers_kit::paint::mark_mat;

/// `rgba(…)` of an sRGB colour (0-1) at alpha `a`
pub(crate) fn css(c: [f64; 3], a: f64) -> String {
    format!("rgba({:.0},{:.0},{:.0},{:.3})", c[0] * 255.0, c[1] * 255.0, c[2] * 255.0, a.clamp(0.0, 1.0))
}

/// A soft round blob, squashed by the context (never an ellipse: no rim)
pub(crate) fn blob(ctx: &mut Canvas, x: f64, y: f64, r: f64, k: f64, turn: f64, c: [f64; 3], a: f64) {
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

/// A painted mark lying flat on the floor, `w` × `d` m, its middle at (x, z)
pub(crate) fn floor_mark(tex: &Tex, w: f64, d: f64, x: f64, z: f64, turn: f64) -> Obj {
    let m = mesh(plane_geo!(w, d), mark_mat(tex));
    m.rotation().x = -PI / 2.0;
    m.rotation().z = turn;
    m.position().set(x, 0.0015, z);
    m
}
