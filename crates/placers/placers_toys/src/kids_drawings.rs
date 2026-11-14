//! `place_kids_drawings`
use game::prelude::*;
use placers_kit::place::pick_open;

pub(crate) fn place_kids_drawings(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let Some((x, z)) = pick_open(g) else { return };
    let draw_mat = basic_mat!({ map: kids_drawing_texture(), transparent: true, opacity: 0.75, depthWrite: false });
    let decal = mesh(plane_geo!(1.0, 1.0), &draw_mat);
    decal.rotation().x = -PI / 2.0;
    let px = g.wx(x) + (rnd() - 0.5) * 2.0;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.0;
    decal.position().set(px, 0.004, pz);
    decal.rotation().z = rnd() * PI * 2.0;
    g.world.add(&decal);
}

// Kids drawing decal: 6% chance per level. Canvas texture
// with random scribbles simulating child chalk drawing.
game::lazy_static_tex!(
    kids_drawing_texture,
    canvas_tex_opt(384, 384, false, |ctx, w, h| {
        ctx.clear_rect(0, 0, w, h);
        // Chalk-ish palette.
        let colors = ["#f0ece0", "#d0d0d0", "#d08080", "#80d080", "#8080d0", "#d0d080"];
        ctx.set_line_cap("round");
        ctx.set_line_width(4);
        // 3-6 scribble strokes.
        let n_scribbles = 3 + (rnd() * 4.0).floor() as i32;
        for _s in 0..n_scribbles {
            ctx.set_stroke_style(colors[(rnd() * colors.len() as f64).floor() as usize]);
            ctx.set_global_alpha(0.4 + rnd() * 0.4);
            ctx.begin_path();
            let mut cx = 40.0 + rnd() * (w - 80.0);
            let mut cy = 40.0 + rnd() * (h - 80.0);
            ctx.move_to(cx, cy);
            let n_pts = 6 + (rnd() * 8.0).floor() as i32;
            for _p in 0..n_pts {
                cx += (rnd() - 0.5) * 80.0;
                cy += (rnd() - 0.5) * 80.0;
                ctx.line_to(cx, cy);
            }
            ctx.stroke();
        }
        // A stick figure or sun.
        ctx.set_global_alpha(0.7);
        ctx.set_stroke_style(colors[(rnd() * colors.len() as f64).floor() as usize]);
        ctx.set_line_width(5);
        let cx = w / 2.0 + (rnd() - 0.5) * 80.0;
        let cy = h / 2.0 + (rnd() - 0.5) * 80.0;
        // Circle (head or sun).
        ctx.begin_path();
        ctx.arc(cx, cy, 30, 0, PI * 2.0);
        ctx.stroke();
        // Rays or body lines.
        for i in 0..8 {
            let a = (i as f64 / 8.0) * PI * 2.0;
            ctx.begin_path();
            ctx.move_to(cx + a.dcos() * 32.0, cy + a.dsin() * 32.0);
            ctx.line_to(cx + a.dcos() * 55.0, cy + a.dsin() * 55.0);
            ctx.stroke();
        }
    })
);
