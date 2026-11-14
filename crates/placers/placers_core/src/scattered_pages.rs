//! `place_scattered_pages`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::scatter;

// Scattered pages: 8% chance per level. 5-10 loose sheets with writing.
pub(crate) fn place_scattered_pages(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let page_mat = lambert!({ map: scattered_page_texture(), side: DoubleSide });
    let grp = group();
    let n_pages = 5 + (rnd() * 6.0).floor() as i32;
    for i in 0..n_pages {
        let page = mesh(plane_geo!(0.14, 0.18), &page_mat);
        page.rotation().x = -PI / 2.0;
        page.position().set((rnd() - 0.5) * 0.7, 0.003 + i as f64 * 0.0004, (rnd() - 0.5) * 0.6);
        page.rotation().z = rnd() * PI * 2.0;
        // 20% chance curled (small tilt).
        if rnd() < 0.20 {
            page.rotation().x = -PI / 2.0 + (rnd() - 0.5) * 0.15;
        }
        grp.add(&page);
    }
    scatter(g, &grp, x, z, 2.0);
    g.world.add(&grp);
}

game::lazy_static_tex!(
    scattered_page_texture,
    canvas_tex_opt(128, 160, false, |ctx, w, h| {
        ctx.set_fill_style("#f0e8d8");
        ctx.fill_rect(0, 0, w, h);
        // Age tint corner.
        let grad = ctx.create_radial_gradient(w / 2.0, h / 2.0, 30, w / 2.0, h / 2.0, 90);
        grad.add_color_stop(0, "rgba(240,232,216,0)");
        grad.add_color_stop(1, "rgba(160,120,80,0.35)");
        ctx.set_fill_style(&grad);
        ctx.fill_rect(0, 0, w, h);
        // 8 lines of squiggly text.
        ctx.set_stroke_style("rgba(40,20,10,0.7)");
        ctx.set_line_width(1.5);
        for line in 0..8 {
            let y = 12.0 + line as f64 * 18.0;
            let mut x = 8.0;
            while x < w - 8.0 {
                let word_w = 12.0 + rnd() * 24.0;
                ctx.begin_path();
                ctx.move_to(x, y);
                ctx.bezier_curve_to(x + word_w / 3.0, y - 2.0, x + word_w * 2.0 / 3.0, y + 2.0, x + word_w, y);
                ctx.stroke();
                x += word_w + 6.0;
            }
        }
    })
);
