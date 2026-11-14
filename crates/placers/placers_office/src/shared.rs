//! What several office placers share.
use game::assets::props::prop_part_as_modelled;
use game::base::rng::SplitMix64;
use game::prelude::*;
use game::visuals::pictures;
use game::visuals::pictures::Age;
use game::visuals::pictures::Finish;
use game::visuals::pictures::Outline;
use placers_kit::paint::rgba;

/// A custom geometry: positions + index, vertex normals computed.
pub(crate) fn tri_geo(verts: Vec<f32>, idx: Vec<u32>) -> Geo {
    let geom = Geo::new();
    geom.set_position(verts);
    geom.set_index(idx);
    geom.compute_vertex_normals();
    geom
}

/// A picture hung flush from the sawtooth on its back over a nail 16 mm
/// below its top (`h` its height), the wall `wall` behind the spot, askew by
/// `tilt`: the body its parts go in (z = 0 the wall)
pub(crate) fn on_nail(o: &Obj, mount_y: f64, wall: f64, h: f64, tilt: f64) -> Obj {
    let nail = group();
    nail.position().set(0, mount_y + h / 2.0 - 0.016, -wall);
    nail.rotation().z = tilt;
    o.add(&nail);
    let body = group();
    body.position().y = -(h / 2.0 - 0.016);
    nail.add(&body);
    body
}

/// A salt from where a placed group lies
pub(crate) fn salt(o: &Obj) -> u32 {
    let p = o.pos();
    ((p.x * 977.0).floor() as i64 ^ ((p.z * 1181.0).floor() as i64).rotate_left(13)) as u32
}

/// A soft round spot: `r` px, `k` times as tall as wide, turned `turn`
/// (the context squashed, so the fade is never cut by an edge)
pub(crate) fn soft(ctx: &mut Canvas, at: (f64, f64), r: f64, k: f64, turn: f64, c: (f64, f64, f64), a: f64) {
    ctx.save();
    ctx.translate(at.0, at.1);
    ctx.rotate(turn);
    ctx.scale(1, k);
    let gr = ctx.create_radial_gradient(0, 0, 0, 0, 0, r);
    gr.add_color_stop(0, &rgba(c, a));
    gr.add_color_stop(1, &rgba(c, 0.0));
    ctx.set_fill_style(&gr);
    ctx.begin_path();
    ctx.arc(0, 0, r, 0, 7);
    ctx.fill();
    ctx.restore();
}

/// Fills the canvas pixel by pixel on every core: `f(u, v)` (u right, v up,
/// both 0..1, as the texture is laid) gives straight sRGB, 0..1
pub(crate) fn t10_fill(ctx: &mut Canvas, w: f64, h: f64, f: impl Fn(f64, f64) -> [f64; 3] + Sync) {
    let iw = w as usize;
    let mut img = ctx.create_image_data(w, h);
    par_rows(&mut img.data, iw * 4, |j, out| {
        for i in 0..iw {
            let c = f((i as f64 + 0.5) / w, 1.0 - (j as f64 + 0.5) / h);
            for k in 0..3 {
                out[i * 4 + k] = (c[k].clamp(0.0, 1.0) * 255.0).round() as u8;
            }
            out[i * 4 + 3] = 255;
        }
    });
    ctx.put_image_data(&img, 0, 0);
}
