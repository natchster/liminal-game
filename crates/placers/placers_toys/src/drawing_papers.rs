//! `place_drawing_papers`: Drawing paper: 10% chance per level.
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

/// Drawing paper: 10% chance per level.
pub(crate) fn place_drawing_papers(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let paper_mat = lambert!({ map: drawing_paper_texture(), side: DoubleSide });
    let paper = mesh(plane_geo!(0.22, 0.28), &paper_mat);
    paper.rotation().x = -PI / 2.0;
    paper.rotation().z = rnd() * PI * 2.0;
    paper.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0.005, g.wz(z) + (rnd() - 0.5) * 2.4);
    g.world.add(&paper);
}

// Drawing paper texture — cream paper with random crayon-color
// scribble strokes. Built once.
game::lazy_static_tex!(
    drawing_paper_texture,
    canvas_tex_opt(96, 96, false, |ctx, w, h| {
        ctx.set_fill_style("#e8dcc0");
        ctx.fill_rect(0, 0, w, h);
        // Random crayon scribble strokes.
        let scribble_colors = ["#c82020", "#2050c8", "#c8b020", "#30a030", "#8020c8", "#d05038"];
        let strokes = 4 + (rnd() * 4.0).floor() as i32;
        for _ in 0..strokes {
            ctx.set_stroke_style(pick(&scribble_colors));
            ctx.set_line_width(2.0 + rnd() * 2.0);
            ctx.begin_path();
            let start_x = 10.0 + rnd() * (w - 20.0);
            let start_y = 10.0 + rnd() * (h - 20.0);
            ctx.move_to(start_x, start_y);
            // Wiggly line via 4 quadratic curves.
            let (mut cur_x, mut cur_y) = (start_x, start_y);
            for _ in 0..4 {
                let cp_x = cur_x + (rnd() - 0.5) * 30.0;
                let cp_y = cur_y + (rnd() - 0.5) * 30.0;
                let end_x = cur_x + (rnd() - 0.5) * 40.0;
                let end_y = cur_y + (rnd() - 0.5) * 40.0;
                ctx.quadratic_curve_to(cp_x, cp_y, end_x, end_y);
                cur_x = end_x;
                cur_y = end_y;
            }
            ctx.stroke();
        }
        // Slight grunge.
        for _ in 0..15 {
            ctx.set_fill_style(&format!("rgba(0,0,0,{})", 0.05 + rnd() * 0.08));
            ctx.fill_rect(rnd() * w, rnd() * h, 1, 1);
        }
    })
);
