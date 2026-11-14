//! `place_watercolors`
use game::prelude::*;
use placers_kit::place::face_yaw;
use placers_kit::place::pick;
use placers_kit::place::wall_tiles;

pub(crate) fn place_watercolors(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let walls = wall_tiles(g);
    if walls.is_empty() {
        return;
    }
    let (x, z, dx, dz) = pick(&walls);
    let frame_mat = lambert!({ color: 0xd0a040 });
    let matting_mat = lambert!({ color: 0xf0eddb });
    let paint_mat = lambert!({ map: watercolor_texture() });
    let painting = group();
    let mount_y = 1.65;
    // Gold outer frame.
    let frame = mesh(pg("Watercolors__frame", box_geo!(0.34, 0.26, 0.014)), &frame_mat);
    frame.position().y = mount_y;
    painting.add(&frame);
    // White matting inset.
    let matting = mesh(pg("Watercolors__mat", box_geo!(0.28, 0.20, 0.010)), &matting_mat);
    matting.position().set(0, mount_y, 0.008);
    painting.add(&matting);
    // Watercolor paper surface.
    let art = mesh(plane_geo!(0.22, 0.16), &paint_mat);
    art.position().set(0, mount_y, 0.014);
    painting.add(&art);
    painting.rotation().z = (rnd() - 0.5) * 0.05; // slight tilt
    painting.rotation().y = face_yaw(dx, dz);
    painting.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - 0.015), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - 0.015));
    g.world.add(&painting);
}

// Framed watercolor: 4% chance per level. Ornate frame + pastel abstract
// wash inside.
game::lazy_static_tex!(
    watercolor_texture,
    canvas_tex(128, 96, |ctx, w, h| {
        // Soft off-white paper.
        ctx.set_fill_style("#f0ecdc");
        ctx.fill_rect(0, 0, w, h);
        // 5 large soft blobs of pastel color.
        let pastels = ["#a0c0e8", "#e0b0c0", "#c0d8a8", "#e8d090", "#c0a0d0"];
        for i in 0..8 {
            ctx.set_fill_style(&format!("{}a0", pastels[i % pastels.len()])); // alpha ~62%.
            ctx.begin_path();
            let cx = rnd() * w;
            let cy = rnd() * h;
            let r = 8.0 + rnd() * 24.0;
            ctx.arc(cx, cy, r, 0, PI * 2.0);
            ctx.fill();
        }
        // Thin dark accent lines (brush strokes).
        ctx.set_stroke_style("#403020");
        ctx.set_line_width(1);
        for _i in 0..3 {
            ctx.begin_path();
            let (mx, my) = (rnd() * w, rnd() * h);
            ctx.move_to(mx, my);
            let (c1x, c1y) = (rnd() * w, rnd() * h);
            let (c2x, c2y) = (rnd() * w, rnd() * h);
            let (ex, ey) = (rnd() * w, rnd() * h);
            ctx.bezier_curve_to(c1x, c1y, c2x, c2y, ex, ey);
            ctx.stroke();
        }
    })
);
