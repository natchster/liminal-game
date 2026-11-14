//! `place_caution_tapes`
use game::prelude::*;
use placers_kit::place::wall_candidates;

// Caution tape strip: 5% chance per level. Striped tape on wall.
pub(crate) fn place_caution_tapes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = candidates[(rnd() * candidates.len() as f64).floor() as usize];
    let mat = lambert!({ map: caution_tape_texture(), side: DoubleSide });
    let tape = mesh(plane_geo!(2.2, 0.05), &mat);
    let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.005);
    let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.005);
    let center_y = 1.1;
    if c.dx.abs() == 1 {
        tape.rotation().y = if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 };
        tape.position().set(wall_x, center_y, wall_z);
    } else {
        tape.rotation().y = if c.dz > 0 { PI } else { 0.0 };
        tape.position().set(wall_x, center_y, wall_z);
    }
    // Small tilt for organic hang.
    tape.rotation().z = (rnd() - 0.5) * 0.08;
    g.world.add(&tape);
}

game::lazy_static_tex!(
    caution_tape_texture,
    canvas_tex(512, 48, |ctx, w, h| {
        ctx.set_fill_style("#e8b820");
        ctx.fill_rect(0, 0, w, h);
        ctx.set_fill_style("#181008");
        // Diagonal black stripes.
        ctx.save();
        ctx.translate(0, 0);
        let mut i = -60.0;
        while i < w {
            ctx.save();
            ctx.begin_path();
            ctx.move_to(i, 0);
            ctx.line_to(i + 20.0, 0);
            ctx.line_to(i + 20.0 + 24.0, h);
            ctx.line_to(i + 24.0, h);
            ctx.close_path();
            ctx.fill();
            ctx.restore();
            i += 30.0;
        }
        ctx.restore();
        // "CAUTION" text overlay.
        ctx.set_fill_style("#e8b820");
        ctx.fill_rect(w * 0.35, 8, w * 0.30, 32);
        ctx.set_fill_style("#181008");
        ctx.set_font("bold 22px sans-serif");
        ctx.set_text_align("center");
        ctx.set_text_baseline("middle");
        ctx.fill_text("CAUTION", w / 2.0, h / 2.0);
    })
);
