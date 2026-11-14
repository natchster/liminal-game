//! What several domestic placers share.
use game::assets::props::prop_part_as_modelled;
use game::prelude::*;
use game::visuals::pictures;
use game::visuals::pictures::Age;
use game::visuals::pictures::Finish;
use game::visuals::pictures::Outline;

/// A hash of a tile and a salt: what a thing placed there is, with no draw
pub(crate) fn kitchen_hash(x: i32, z: i32, salt: u32) -> u32 {
    let mut h = salt ^ (x as u32).wrapping_mul(0x27d4_eb2d) ^ (z as u32).wrapping_mul(0x1656_67b1);
    h = (h ^ (h >> 15)).wrapping_mul(0x85eb_ca6b);
    h = (h ^ (h >> 13)).wrapping_mul(0xc2b2_ae35);
    h ^ (h >> 16)
}

/// A painting's hanging: the hook on the wall at the wire's apex (`apex`
/// above the picture's middle at `mount_y`, the wall `wall` behind the
/// spot), the frame swinging about it by `tilt` and leaning out `lean` at
/// its top from its foot (`h` its height); returns the body the parts go in
pub(crate) fn hang(o: &Obj, mount_y: f64, wall: f64, apex: f64, h: f64, lean: f64, tilt: f64) -> Obj {
    let nail = group();
    nail.position().set(0, mount_y + apex, -wall);
    nail.rotation().z = tilt;
    o.add(&nail);
    if let Some(geo) = prop_part_as_modelled("Real__oldPortraitHook") {
        nail.add(&mesh(geo, phong!({ color: 0x6a5630, specular: 0x5a4a2a, shininess: 36 })));
    }
    let foot = group();
    foot.position().y = -apex - h / 2.0;
    foot.rotation().x = (lean / h).atan();
    nail.add(&foot);
    let body = group();
    body.position().y = h / 2.0;
    foot.add(&body);
    body
}

pub(crate) fn add_parts(body: &Obj, parts: &[(&str, Mat)]) {
    for (name, mat) in parts {
        if let Some(geo) = prop_part_as_modelled(name) {
            body.add(&mesh(geo, mat));
        }
    }
}

/// The wall's face in a wall prop's frame (its group stands 15 mm out)
pub(crate) const WALL_Z: f64 = -0.015;

/// The nail's rise: driven 8 degrees up from square to the wall
pub(crate) const NAIL_RISE: f64 = 0.140_541; // tan(8 degrees)

pub(crate) const NAIL_R: f64 = 0.001_35;

/// The part `name` of a Blender batch, as modelled, added to `grp` in `mat`.
pub(crate) fn add(grp: &Obj, name: &str, mat: &Mat) {
    if let Some(geo) = game::assets::props::prop_part_as_modelled(name) {
        grp.add(&mesh(geo, mat.clone()));
    }
}

pub(crate) fn rgb(c: u32) -> (f64, f64, f64) {
    (((c >> 16) & 255) as f64, ((c >> 8) & 255) as f64, (c & 255) as f64)
}

pub(crate) fn css(c: (f64, f64, f64), a: f64) -> String {
    let k = |v: f64| v.clamp(0.0, 255.0).round();
    format!("rgba({},{},{},{:.3})", k(c.0), k(c.1), k(c.2), a.clamp(0.0, 1.0))
}

pub(crate) fn mix(a: (f64, f64, f64), b: (f64, f64, f64), t: f64) -> (f64, f64, f64) {
    (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t, a.2 + (b.2 - a.2) * t)
}

/// A soft spot at `at`, `r` px, `k` times as tall as wide, turned `turn`,
/// fading out from its middle (the context squashed, so no edge cuts it).
pub(crate) fn soft(ctx: &mut Canvas, at: (f64, f64), r: f64, k: f64, turn: f64, c: (f64, f64, f64), a: f64) {
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

/// A short wandering line from `at`, `len` px long at angle `ang`.
pub(crate) fn wander(ctx: &mut Canvas, at: (f64, f64), len: f64, ang: f64, c: (f64, f64, f64), a: f64, lw: f64) {
    ctx.set_stroke_style(&css(c, a));
    ctx.set_line_width(lw);
    ctx.begin_path();
    ctx.move_to(at.0, at.1);
    let (mut x, mut y, mut t) = (at.0, at.1, ang);
    for _ in 0..5 {
        t += (rnd() - 0.5) * 0.5;
        x += t.cos() * len / 5.0;
        y += t.sin() * len / 5.0;
        ctx.line_to(x, y);
    }
    ctx.stroke();
}

/// Leather after years, in its colour `base` (sRGB): mottled where it was
/// oiled and dried, its grain, creased, rubbed pale and scuffed (most low
/// on it, v < 0.35: the welt's edge all round, the toe's foot), a film of
/// dust. Painted on a stream of its own varied by `vary`.
pub(crate) fn leather_tex(base: u32, size: u32, vary: u32) -> Tex {
    canvas_tex(size, size, move |ctx, w, h| {
        let seed = (rnd() * 4.0e9) as u32 ^ vary;
        with_own_stream(seed, || paint_leather(ctx, w, h, base))
    })
}

pub(crate) fn paint_leather(ctx: &mut Canvas, w: f64, h: f64, base: u32) {
    let c = rgb(base);
    let s = w / 256.0;
    ctx.set_fill_style(&css(c, 1.0));
    ctx.fill_rect(0, 0, w, h);
    let dark = (c.0 * 0.62, c.1 * 0.58, c.2 * 0.55);
    let pale = mix(c, (196.0, 184.0, 166.0), 0.42);
    for _ in 0..36 {
        let at = (rnd() * w, rnd() * h);
        let (col, a) = if rnd() < 0.55 { (dark, 0.10 + rnd() * 0.14) } else { (pale, 0.06 + rnd() * 0.10) };
        soft(ctx, at, (10.0 + rnd() * 34.0) * s, 0.5 + rnd() * 0.5, rnd() * 3.0, col, a);
    }
    // the grain: pores and a fine pebble
    for _ in 0..(2600.0 * s * s) as i32 {
        let col = if rnd() < 0.7 { dark } else { pale };
        ctx.set_fill_style(&css(col, 0.12 + rnd() * 0.16));
        ctx.fill_rect(rnd() * w, rnd() * h, 1.0, 1.0);
    }
    // creases, each with its pale lip
    for _ in 0..(48.0 * s) as i32 {
        let at = (rnd() * w, rnd() * h);
        let len = (6.0 + rnd() * 24.0) * s;
        let ang = (rnd() - 0.5) * 1.0 + if rnd() < 0.25 { PI / 2.0 } else { 0.0 };
        wander(ctx, at, len, ang, (c.0 * 0.4, c.1 * 0.36, c.2 * 0.34), 0.45, 0.8 * s.max(0.7));
        wander(ctx, (at.0, at.1 + 1.0), len * 0.8, ang, pale, 0.18, 0.7);
    }
    // rubbed pale and scuffed, most of it low: the dye worn through to the
    // crust in ragged streaks along the rub, scratched
    for i in 0..30 {
        let y = if i < 22 { h * (0.66 + rnd() * 0.34) } else { rnd() * h };
        let at = (rnd() * w, y);
        let crust = mix(c, (190.0, 178.0, 160.0), 0.40 + rnd() * 0.2);
        let ang = (rnd() - 0.5) * 0.5;
        for _ in 0..(3 + (rnd() * 5.0) as i32) {
            let a2 = (at.0 + (rnd() - 0.5) * 10.0 * s, at.1 + (rnd() - 0.5) * 4.0 * s);
            wander(
                ctx,
                a2,
                (2.0 + rnd() * 9.0) * s,
                ang + (rnd() - 0.5) * 0.3,
                crust,
                0.18 + rnd() * 0.3,
                (0.5 + rnd() * 0.9) * s.max(0.6),
            );
        }
        soft(ctx, at, (4.0 + rnd() * 8.0) * s, 0.25, ang, crust, 0.10 + rnd() * 0.10);
    }
    // dust, and dried water's tide marks
    for _ in 0..12 {
        soft(ctx, (rnd() * w, rnd() * h), (18.0 + rnd() * 40.0) * s, 0.6, rnd() * 3.0, (150.0, 144.0, 132.0), 0.08 + rnd() * 0.08);
    }
    for _ in 0..3 {
        let (x, y, r) = (rnd() * w, rnd() * h, (10.0 + rnd() * 26.0) * s);
        ctx.set_stroke_style(&css(pale, 0.22));
        ctx.set_line_width(1.2 * s);
        ctx.begin_path();
        ctx.arc(x, y, r, rnd() * 3.0, 3.0 + rnd() * 3.0);
        ctx.stroke();
    }
}

/// A custom geometry from f32 positions + an index, normals computed.
pub(crate) fn buffer_geo(verts: &[f64], index: Vec<u32>) -> Geo {
    let geo = Geo::new();
    geo.set_position(verts.iter().map(|&a| a as f32).collect());
    geo.set_index(index);
    geo.compute_vertex_normals();
    geo
}
