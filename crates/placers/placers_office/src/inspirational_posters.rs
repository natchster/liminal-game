//! `place_inspirational_posters`
use game::prelude::*;
use placers_kit::place::mount_on_wall;
use placers_kit::place::wall_candidates;

pub(crate) fn place_inspirational_posters(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = candidates[(rnd() * candidates.len() as f64).floor() as usize];
    let frame_mat = lambert!({ color: 0x181410 });
    let poster_mat = lambert!({ map: poster_texture(), side: DoubleSide });
    let poster = group();
    let (w, h, t, fw) = (0.32, 0.40, 0.010, 0.014);
    for (bar_w, bar_h, bx, by) in [
        (w + fw * 2.0, fw, 0.0, h / 2.0 + fw / 2.0),
        (w + fw * 2.0, fw, 0.0, -h / 2.0 - fw / 2.0),
        (fw, h, -w / 2.0 - fw / 2.0, 0.0),
        (fw, h, w / 2.0 + fw / 2.0, 0.0),
    ] {
        let bar = mesh(pg("InspirationalPosters__bar", box_geo!(bar_w, bar_h, t)), &frame_mat);
        bar.position().set(bx, by, 0);
        poster.add(&bar);
    }
    let canvas = mesh(plane_geo!(w, h), &poster_mat);
    canvas.position().set(0, 0, t / 2.0 + 0.001);
    poster.add(&canvas);
    mount_on_wall(g, &poster, &c, 0.007, 1.65, 0.5);
    g.world.add(&poster);
}

// Inspirational poster texture.
game::lazy_static_tex!(
    poster_texture,
    canvas_tex(256, 320, |ctx, w, h| {
        // Sky gradient background.
        let grad = ctx.create_linear_gradient(0, 0, 0, h * 0.65);
        grad.add_color_stop(0, "#80c0e0");
        grad.add_color_stop(1, "#f0e0a0");
        ctx.set_fill_style(&grad);
        ctx.fill_rect(0, 0, w, h);
        // Mountain silhouettes.
        ctx.set_fill_style("#403050");
        ctx.begin_path();
        ctx.move_to(0, h * 0.5);
        ctx.line_to(w * 0.2, h * 0.30);
        ctx.line_to(w * 0.35, h * 0.45);
        ctx.line_to(w * 0.55, h * 0.25);
        ctx.line_to(w * 0.75, h * 0.40);
        ctx.line_to(w * 0.9, h * 0.30);
        ctx.line_to(w, h * 0.5);
        ctx.line_to(w, h * 0.65);
        ctx.line_to(0, h * 0.65);
        ctx.close_path();
        ctx.fill();
        // Text area (dark bar at bottom).
        ctx.set_fill_style("#181410");
        ctx.fill_rect(0, h * 0.65, w, h * 0.35);
        // Motivational word.
        ctx.set_fill_style("#f0ece0");
        ctx.set_font("bold 42px serif");
        ctx.set_text_align("center");
        ctx.set_text_baseline("middle");
        let words = ["BELIEVE", "DREAM", "CONQUER", "IMAGINE", "STRIVE"];
        let word = words[(rnd() * words.len() as f64).floor() as usize];
        ctx.fill_text(word, w / 2.0, h * 0.78);
        // Subtitle.
        ctx.set_font("italic 14px serif");
        ctx.fill_text("The only limit is you.", w / 2.0, h * 0.92);
    })
);
