//! `place_torn_photographs`
use game::prelude::*;
use placers_kit::place::open_tiles;

pub(crate) fn place_torn_photographs(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = choice(&open);
    let photo_mat = lambert!({ map: torn_photo_texture(), side: DoubleSide, transparent: true, alphaTest: 0.1 });
    let photo = mesh(plane_geo!(0.10, 0.13), &photo_mat);
    photo.rotation().x = -PI / 2.0;
    let px = g.wx(x) + (rnd() - 0.5) * 2.4;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.4;
    photo.position().set(px, 0.003, pz);
    photo.rotation().z = rnd() * PI * 2.0;
    g.world.add(&photo);
}

// Torn photograph: 8% chance per level. Half-photo w/
// jagged tear edge (canvas texture).
game::lazy_static_tex!(
    torn_photo_texture,
    canvas_tex_opt(128, 160, false, |ctx, w, h| {
        // Sepia photo.
        let grad = ctx.create_radial_gradient(w / 2.0, h / 2.0, 20, w / 2.0, h / 2.0, 100);
        grad.add_color_stop(0, "#c0a080");
        grad.add_color_stop(1, "#503020");
        ctx.set_fill_style(&grad);
        ctx.fill_rect(0, 0, w, h);
        // Silhouette figure.
        ctx.set_fill_style("#201810");
        ctx.begin_path();
        ctx.arc(w / 2.0, h * 0.4, 20, 0, PI * 2.0);
        ctx.fill();
        ctx.begin_path();
        ctx.move_to(w / 2.0 - 25.0, h);
        ctx.line_to(w / 2.0 + 25.0, h);
        ctx.line_to(w / 2.0 + 15.0, h * 0.55);
        ctx.line_to(w / 2.0 - 15.0, h * 0.55);
        ctx.close_path();
        ctx.fill();
        // Tear edge (right side): make right portion transparent.
        ctx.set_global_composite_operation("destination-out");
        ctx.set_fill_style("#000");
        ctx.begin_path();
        ctx.move_to(w * 0.6, 0);
        // Jagged tear.
        let mut y = 0.0;
        while y < h {
            ctx.line_to(w * (0.55 + rnd() * 0.10), y);
            y += 8.0;
        }
        ctx.line_to(w, h);
        ctx.line_to(w, 0);
        ctx.close_path();
        ctx.fill();
        ctx.set_global_composite_operation("source-over");
    })
);
