//! `place_child_drawings`
use game::prelude::*;
use placers_kit::place::face_wall;
use placers_kit::place::pick;
use placers_kit::place::wall_pos;
use placers_kit::place::wall_tiles;

pub(crate) fn place_child_drawings(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    }
    let wt = wall_tiles(g);
    if wt.is_empty() {
        return;
    }
    let (x, z, dx, dz) = pick(&wt);
    // Frame — plain wood or thin metal.
    let frame_colors: [u32; 4] = [0x8b5a2b, 0x604020, 0x505050, 0xa07040];
    let frame_mat = lambert!({ color: pick(&frame_colors) });
    let paper_mat = lambert!({ map: child_drawing_texture() });
    let dw = group();
    let mount_y = 1.55;
    let (frame_w, frame_h) = (0.30, 0.24);
    let frame = mesh(pg("ChildDrawings__frame", box_geo!(frame_w, frame_h, 0.012)), &frame_mat);
    frame.position().set(0, mount_y, 0.010);
    dw.add(&frame);
    let paper = mesh(plane_geo!(frame_w - 0.024, frame_h - 0.024), &paper_mat);
    paper.position().set(0, mount_y, 0.017);
    dw.add(&paper);
    dw.rotation().y = face_wall(dx, dz);
    let (px, pz) = wall_pos(g, x, z, dx, dz, 0.015);
    dw.position().set(px, 0, pz);
    dw.rotation().z = (rnd() - 0.5) * 0.05;
    g.world.add(&dw);
}

// Framed child's crayon drawing (wall): 3% chance per level.
// Simple stick-figure family + house on manila paper.
game::lazy_static_tex!(
    child_drawing_texture,
    canvas_tex(240, 192, |ctx, w, h| {
        // Manila/cream paper background.
        ctx.set_fill_style("#f4e8c8");
        ctx.fill_rect(0, 0, w, h);
        // Subtle paper grain — scattered lighter dots.
        ctx.set_fill_style("rgba(220, 200, 160, 0.3)");
        for _ in 0..60 {
            ctx.begin_path();
            let (ax, ay, ar) = (rnd() * w, rnd() * h, 1.0 + rnd() * 1.5);
            ctx.arc(ax, ay, ar, 0, PI * 2.0);
            ctx.fill();
        }
        // Blue sky band top-third — crayon scribbles.
        ctx.set_stroke_style("#5090d0");
        ctx.set_line_width(3);
        for _ in 0..25 {
            ctx.begin_path();
            let sx = rnd() * w;
            let sy = rnd() * h * 0.28;
            ctx.move_to(sx, sy);
            let (lx, ly) = (sx + 20.0 + rnd() * 20.0, sy + (rnd() - 0.5) * 6.0);
            ctx.line_to(lx, ly);
            ctx.stroke();
        }
        // Sun (top-right yellow scribbled circle).
        ctx.set_stroke_style("#f0c040");
        ctx.set_line_width(4);
        let (sun_x, sun_y, sun_r) = (w * 0.82, h * 0.18, 20.0);
        ctx.begin_path();
        ctx.arc(sun_x, sun_y, sun_r, 0, PI * 2.0);
        ctx.stroke();
        // Sun rays.
        for i in 0..8 {
            let a = (i as f64 / 8.0) * PI * 2.0;
            ctx.begin_path();
            ctx.move_to(sun_x + a.dcos() * (sun_r + 3.0), sun_y + a.dsin() * (sun_r + 3.0));
            ctx.line_to(sun_x + a.dcos() * (sun_r + 15.0), sun_y + a.dsin() * (sun_r + 15.0));
            ctx.stroke();
        }
        // Green grass ground line.
        ctx.set_stroke_style("#60a040");
        ctx.set_line_width(5);
        for i in 0..30 {
            let gx = i as f64 * 8.0;
            ctx.begin_path();
            ctx.move_to(gx, h * 0.75);
            ctx.line_to(gx + 4.0, h * 0.68 + rnd() * 8.0);
            ctx.stroke();
        }
        ctx.begin_path();
        ctx.move_to(0, h * 0.75);
        ctx.line_to(w, h * 0.75);
        ctx.stroke();
        // House (center-left) — crayon square + triangle.
        let (hx, hy) = (w * 0.30, h * 0.55);
        ctx.set_stroke_style("#b04020");
        ctx.set_line_width(4);
        ctx.stroke_rect(hx - 30.0, hy, 60, 40);
        ctx.begin_path();
        ctx.move_to(hx - 34.0, hy);
        ctx.line_to(hx, hy - 30.0);
        ctx.line_to(hx + 34.0, hy);
        ctx.stroke();
        // Door.
        ctx.set_stroke_style("#602010");
        ctx.stroke_rect(hx - 8.0, hy + 18.0, 16, 22);
        // Window.
        ctx.set_stroke_style("#4080b0");
        ctx.stroke_rect(hx + 10.0, hy + 8.0, 12, 12);
        // Stick figure family — 3 figures right side.
        struct Fig {
            x: f64,
            head_r: f64,
            body_h: f64,
            color: &'static str,
            leg_y: f64,
        }
        let figs = [
            Fig { x: w * 0.62, head_r: 12.0, body_h: 30.0, color: "#a02040", leg_y: h * 0.75 }, // parent
            Fig { x: w * 0.72, head_r: 12.0, body_h: 30.0, color: "#4060a0", leg_y: h * 0.75 }, // parent
            Fig { x: w * 0.82, head_r: 8.0, body_h: 18.0, color: "#40a040", leg_y: h * 0.75 },  // child
        ];
        for f in &figs {
            ctx.set_stroke_style(f.color);
            ctx.set_line_width(3);
            // Head.
            ctx.begin_path();
            ctx.arc(f.x, f.leg_y - f.body_h - f.head_r, f.head_r, 0, PI * 2.0);
            ctx.stroke();
            // Smile.
            ctx.begin_path();
            ctx.arc(f.x, f.leg_y - f.body_h - f.head_r + 2.0, f.head_r * 0.55, 0.2, PI - 0.2);
            ctx.stroke();
            // Two dots for eyes.
            ctx.begin_path();
            ctx.arc(f.x - f.head_r * 0.4, f.leg_y - f.body_h - f.head_r - 2.0, 1.5, 0, PI * 2.0);
            ctx.arc(f.x + f.head_r * 0.4, f.leg_y - f.body_h - f.head_r - 2.0, 1.5, 0, PI * 2.0);
            ctx.set_fill_style(f.color);
            ctx.fill();
            // Body line.
            ctx.begin_path();
            ctx.move_to(f.x, f.leg_y - f.body_h);
            ctx.line_to(f.x, f.leg_y - 4.0);
            ctx.stroke();
            // Arms — out-and-down.
            ctx.begin_path();
            ctx.move_to(f.x - f.body_h * 0.5, f.leg_y - f.body_h * 0.6);
            ctx.line_to(f.x + f.body_h * 0.5, f.leg_y - f.body_h * 0.6);
            ctx.stroke();
            // Legs.
            ctx.begin_path();
            ctx.move_to(f.x, f.leg_y - 4.0);
            ctx.line_to(f.x - f.body_h * 0.35, f.leg_y + 8.0);
            ctx.move_to(f.x, f.leg_y - 4.0);
            ctx.line_to(f.x + f.body_h * 0.35, f.leg_y + 8.0);
            ctx.stroke();
        }
        // Child's signature — bottom left, wobbly.
        ctx.set_fill_style("#603030");
        ctx.set_font("bold 16px cursive");
        ctx.fill_text("by Emma, age 6", 10, h - 8.0);
    })
);
