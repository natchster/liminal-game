//! `place_diplomas`
use game::prelude::*;
use placers_kit::place::pick_wall;
use placers_kit::place::wall_mount;

// Framed diploma: 3% chance per level. Ornate wall certificate.
pub(crate) fn place_diplomas(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    }
    let Some((x, z, dx, dz)) = pick_wall(g) else { return };
    let frame_mat = lambert!({ color: 0xd0a040 });
    let paper_mat = lambert!({ map: diploma_texture() });
    let diploma = group();
    let mount_y = 1.65;
    let frame = mesh(pg("Diplomas__frame", box_geo!(0.26, 0.33, 0.020)), &frame_mat);
    frame.position().y = mount_y;
    diploma.add(&frame);
    let inner = mesh(pg("Diplomas__moulding", box_geo!(0.23, 0.30, 0.010)), lambert!({ color: 0xa07830 }));
    inner.position().set(0, mount_y, 0.013);
    diploma.add(&inner);
    let paper = mesh(plane_geo!(0.21, 0.28), &paper_mat);
    paper.position().set(0, mount_y, 0.019);
    diploma.add(&paper);
    diploma.rotation().z = (rnd() - 0.5) * 0.05; // slight tilt
    wall_mount(g, &diploma, x, z, dx, dz, 0.015);
    g.world.add(&diploma);
}

// `diploma_texture`: parchment, double border, title, script lines, seal, signature.
game::lazy_static_tex!(
    diploma_texture,
    canvas_tex(160, 200, |ctx, w, h| {
        // Cream parchment.
        ctx.set_fill_style("#f0e8c8");
        ctx.fill_rect(0, 0, w, h);
        // Decorative border (double-line rectangle).
        ctx.set_stroke_style("#604020");
        ctx.set_line_width(2);
        ctx.stroke_rect(8, 8, w - 16.0, h - 16.0);
        ctx.stroke_rect(14, 14, w - 28.0, h - 28.0);
        // Header title bar.
        ctx.set_fill_style("#302010");
        ctx.set_font("bold 14px serif");
        ctx.set_text_align("center");
        ctx.fill_text("CERTIFICATE", w / 2.0, 40);
        // 6 script-line placeholders.
        ctx.set_fill_style("#403020");
        for i in 0..6 {
            let y = 60.0 + i as f64 * 14.0;
            let line_w = w * (0.5 + rnd() * 0.3);
            ctx.fill_rect((w - line_w) / 2.0, y, line_w, 2);
        }
        // Circle seal (red).
        ctx.set_fill_style("#a02020");
        ctx.begin_path();
        ctx.arc(w * 0.25, h * 0.85, 14, 0, PI * 2.0);
        ctx.fill();
        // Signature line.
        ctx.set_stroke_style("#403020");
        ctx.set_line_width(1);
        ctx.begin_path();
        ctx.move_to(w * 0.5, h * 0.85);
        ctx.line_to(w * 0.9, h * 0.85);
        ctx.stroke();
    })
);
