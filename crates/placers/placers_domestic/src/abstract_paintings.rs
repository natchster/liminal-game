//! `place_abstract_paintings`
use game::prelude::*;
use placers_kit::place::wall_mount;
use placers_kit::place::wall_tiles;

pub(crate) fn place_abstract_paintings(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.975 {
        return;
    }
    let walls = wall_tiles(g);
    if walls.is_empty() {
        return;
    }
    let (x, z, dx, dz) = walls[(rnd() * walls.len() as f64).floor() as usize];
    let frame_colors = [0x101010, 0x606060, 0xd0d0d0, 0x604020];
    let frame_mat = lambert!({ color: choice(&frame_colors) });
    let canvas_mat = lambert!({ map: abstract_modernist_texture() });
    let ap = group();
    let mount_y = 1.60;
    let (frame_w, frame_h) = (0.28, 0.28);
    let frame = mesh(pg("AbstractPaintings__frame", box_geo!(frame_w, frame_h, 0.012)), &frame_mat);
    frame.position().set(0, mount_y, 0.010);
    ap.add(&frame);
    let canvas = mesh(plane_geo!(frame_w - 0.020, frame_h - 0.020), &canvas_mat);
    canvas.position().set(0, mount_y, 0.017);
    ap.add(&canvas);
    wall_mount(g, &ap, x, z, dx, dz, 0.015);
    ap.rotation().z = (rnd() - 0.5) * 0.03;
    g.world.add(&ap);
}

// Framed modernist abstract painting (wall).
game::lazy_static_tex!(
    abstract_modernist_texture,
    canvas_tex(220, 220, |ctx, w, h| {
        // Off-white canvas background.
        ctx.set_fill_style("#e8e0d0");
        ctx.fill_rect(0, 0, w, h);
        // Variant: 3 different modernist styles.
        let variant = (rnd() * 3.0).floor() as i32;
        if variant == 0 {
            // Kandinsky-inspired: circles + lines.
            ctx.set_fill_style("#f0c040");
            ctx.begin_path();
            ctx.arc(w * 0.35, h * 0.4, 45, 0, PI * 2.0);
            ctx.fill();
            // Red triangle.
            ctx.set_fill_style("#c03040");
            ctx.begin_path();
            ctx.move_to(w * 0.65, h * 0.3);
            ctx.line_to(w * 0.85, h * 0.7);
            ctx.line_to(w * 0.55, h * 0.7);
            ctx.close_path();
            ctx.fill();
            // Blue rectangle.
            ctx.set_fill_style("#3060a0");
            ctx.fill_rect(w * 0.2, h * 0.65, 100, 30);
            // Black line diagonals.
            ctx.set_stroke_style("#101010");
            ctx.set_line_width(3);
            ctx.begin_path();
            ctx.move_to(0, h * 0.2);
            ctx.line_to(w, h * 0.8);
            ctx.stroke();
            ctx.begin_path();
            ctx.move_to(w * 0.15, h);
            ctx.line_to(w * 0.6, 0);
            ctx.stroke();
            // Small green square.
            ctx.set_fill_style("#40a040");
            ctx.fill_rect(w * 0.05, h * 0.05, 30, 30);
        } else if variant == 1 {
            // Mondrian-inspired: black lines + primary color blocks.
            ctx.set_stroke_style("#101010");
            ctx.set_line_width(6);
            ctx.begin_path();
            ctx.move_to(w * 0.4, 0);
            ctx.line_to(w * 0.4, h);
            ctx.move_to(w * 0.75, 0);
            ctx.line_to(w * 0.75, h);
            ctx.move_to(0, h * 0.35);
            ctx.line_to(w, h * 0.35);
            ctx.move_to(0, h * 0.7);
            ctx.line_to(w, h * 0.7);
            ctx.stroke();
            // Color blocks.
            ctx.set_fill_style("#c02040");
            ctx.fill_rect(w * 0.4 + 3.0, 3, w * 0.35 - 6.0, h * 0.35 - 3.0);
            ctx.set_fill_style("#f0c040");
            ctx.fill_rect(3, h * 0.7 + 3.0, w * 0.4 - 3.0, h * 0.30 - 6.0);
            ctx.set_fill_style("#2060a0");
            ctx.fill_rect(w * 0.75 + 3.0, h * 0.35 + 3.0, w * 0.25 - 6.0, h * 0.35 - 6.0);
        } else {
            // Rothko-inspired: soft color fields.
            let top_grad = ctx.create_linear_gradient(0, 0, 0, h * 0.5);
            top_grad.add_color_stop(0, "#601820");
            top_grad.add_color_stop(1, "#803040");
            ctx.set_fill_style(&top_grad);
            ctx.fill_rect(0, 0, w, h * 0.45);
            // Bottom orange band.
            let bottom_grad = ctx.create_linear_gradient(0, h * 0.55, 0, h);
            bottom_grad.add_color_stop(0, "#d06030");
            bottom_grad.add_color_stop(1, "#a04020");
            ctx.set_fill_style(&bottom_grad);
            ctx.fill_rect(0, h * 0.55, w, h * 0.45);
            // Soft transition band in middle.
            ctx.set_fill_style("rgba(100,50,40,0.3)");
            ctx.fill_rect(0, h * 0.42, w, h * 0.16);
            // Add slight edge fuzz to simulate hand-painting.
            ctx.set_fill_style("rgba(0,0,0,0.10)");
            for _ in 0..120 {
                let fx = rnd() * w;
                let fy = rnd() * h;
                ctx.fill_rect(fx, fy, 2, 2);
            }
        }
        // Optional artist signature bottom-right (small).
        ctx.set_fill_style("#302020");
        ctx.set_font("italic 8px cursive");
        ctx.set_text_align("right");
        let signatures = ["M. Klein '62", "E. Vasquez", "J.M. '78", "A. Rodriguez", "S. Chen"];
        ctx.fill_text(choice(&signatures), w - 8.0, h - 6.0);
    })
);
