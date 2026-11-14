//! `place_small_compasses`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Small vintage compass: 8% chance per level.
pub(crate) fn place_small_compasses(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = pick(&open_tiles);
    let brass_mat = lambert!({ color: 0xc89040 });
    let face_mat = lambert!({ map: compass_face_texture(), side: DoubleSide });
    let compass = group();
    // Case (disc).
    let case_ = mesh(pg("SmallCompasses__case", cyl_geo!(0.036, 0.036, 0.012, 14)), &brass_mat);
    case_.position().y = 0.006;
    compass.add(&case_);
    // Face on top.
    let face = mesh(circle_geo!(0.030, 14), &face_mat);
    face.rotation().x = -PI / 2.0;
    face.position().y = 0.0125;
    compass.add(&face);
    // Hinge: barrel along Z (the lid's pivot axis) at the case's top edge.
    let hinge = mesh(pg("SmallCompasses__hinge", cyl_geo!(0.005, 0.005, 0.014, 6)), &brass_mat);
    hinge.rotation().x = PI / 2.0;
    hinge.position().set(0.039, 0.011, 0);
    compass.add(&hinge);
    // Cover leaning open: the closed lid swung 120 deg about the hinge.
    let cover = mesh(pg("SmallCompasses__cover", cyl_geo!(0.036, 0.036, 0.006, 14)), &brass_mat);
    cover.position().set(0.039 + 0.0230, 0.011 + 0.0318, 0);
    cover.rotation().z = -2.0 * PI / 3.0;
    compass.add(&cover);
    compass.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    compass.rotation().y = rnd() * PI * 2.0;
    g.world.add(&compass);
}

// Compass face texture.
game::lazy_static_tex!(
    compass_face_texture,
    canvas_tex_opt(128, 128, false, |ctx, w, h| {
        ctx.set_fill_style("#f0ece0");
        ctx.begin_path();
        ctx.arc(w / 2.0, h / 2.0, w / 2.0 - 4.0, 0, PI * 2.0);
        ctx.fill();
        ctx.set_stroke_style("#181008");
        ctx.set_line_width(2);
        ctx.stroke();
        // Cardinal N/E/S/W.
        ctx.set_fill_style("#181008");
        ctx.set_font("bold 16px serif");
        ctx.set_text_align("center");
        ctx.set_text_baseline("middle");
        ctx.fill_text("N", w / 2.0, 14);
        ctx.fill_text("E", w - 14.0, h / 2.0);
        ctx.fill_text("S", w / 2.0, h - 14.0);
        ctx.fill_text("W", 14, h / 2.0);
        // Cardinal marks (short lines).
        ctx.set_line_width(3);
        for i in 0..16 {
            let a = -PI / 2.0 + (i as f64 / 16.0) * PI * 2.0;
            ctx.begin_path();
            ctx.move_to(w / 2.0 + a.cos() * 40.0, h / 2.0 + a.sin() * 40.0);
            ctx.line_to(w / 2.0 + a.cos() * 48.0, h / 2.0 + a.sin() * 48.0);
            ctx.stroke();
        }
        // Needle (red-tip pointing north).
        ctx.set_fill_style("#d82040");
        ctx.begin_path();
        ctx.move_to(w / 2.0, h / 2.0 - 30.0);
        ctx.line_to(w / 2.0 - 4.0, h / 2.0);
        ctx.line_to(w / 2.0 + 4.0, h / 2.0);
        ctx.close_path();
        ctx.fill();
        ctx.set_fill_style("#404040");
        ctx.begin_path();
        ctx.move_to(w / 2.0, h / 2.0 + 30.0);
        ctx.line_to(w / 2.0 - 4.0, h / 2.0);
        ctx.line_to(w / 2.0 + 4.0, h / 2.0);
        ctx.close_path();
        ctx.fill();
        // Center pin.
        ctx.set_fill_style("#181008");
        ctx.begin_path();
        ctx.arc(w / 2.0, h / 2.0, 4, 0, PI * 2.0);
        ctx.fill();
    })
);
