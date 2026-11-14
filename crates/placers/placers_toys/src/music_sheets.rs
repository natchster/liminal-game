//! `place_music_sheets`: Music sheet paper: 10% chance per level. Canvas-textured paper with staff lines + notes on floor. Child-music family.
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

/// Music sheet paper: 10% chance per level. Canvas-textured
/// paper with staff lines + notes on floor. Child-music family.
pub(crate) fn place_music_sheets(g: &mut Game, _dist: &[i32]) {
    g.pp.paper_positions.clear(); // reset per level (shared with journals)
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let sheet_mat = lambert!({ map: music_sheet_texture(), side: DoubleSide });
    let sheet = mesh(plane_geo!(0.20, 0.25), &sheet_mat);
    sheet.rotation().x = -PI / 2.0;
    sheet.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0.003, g.wz(z) + (rnd() - 0.5) * 2.4);
    sheet.rotation().z = rnd() * PI * 2.0;
    // 30% chance slightly curled (subtle Y tilt).
    if rnd() < 0.30 {
        sheet.rotation().x = -PI / 2.0 + (rnd() - 0.5) * 0.2;
    }
    g.world.add(&sheet);
    // Track for paper rustle audio.
    let p = sheet.pos();
    g.pp.paper_positions.push(v3(p.x, 0.01, p.z));
}

// Music sheet paper texture: staff lines + notes on cream paper.
game::lazy_static_tex!(
    music_sheet_texture,
    canvas_tex_opt(256, 320, false, |ctx, w, h| {
        // Cream paper background.
        ctx.set_fill_style("#f0ece0");
        ctx.fill_rect(0, 0, w, h);
        // Slight age grime.
        for _ in 0..40 {
            ctx.set_fill_style(format!("rgba(120,100,60,{})", 0.02 + rnd() * 0.05));
            let cx = rnd() * w;
            let cy = rnd() * h;
            let rx = 8.0 + rnd() * 24.0;
            ctx.begin_path();
            ctx.ellipse(cx, cy, rx, rx * 0.6, rnd() * PI, 0, PI * 2.0);
            ctx.fill();
        }
        // 4 sets of 5 staff lines.
        ctx.set_stroke_style("#301a10");
        ctx.set_line_width(2);
        for gi in 0..4 {
            let gy = 40.0 + gi as f64 * 72.0;
            for i in 0..5 {
                ctx.begin_path();
                ctx.move_to(16, gy + i as f64 * 8.0);
                ctx.line_to(w - 16.0, gy + i as f64 * 8.0);
                ctx.stroke();
            }
            // Random notes: 6-10 filled ovals along the staff.
            ctx.set_fill_style("#181008");
            let n_notes = 6 + (rnd() * 5.0).floor() as i32;
            for n in 0..n_notes {
                let nx = 30.0 + (n as f64 / n_notes as f64) * (w - 60.0) + rnd() * 8.0;
                let ny_off = (rnd() * 9.0).floor() * 4.0 - 4.0;
                let ny = gy + 16.0 + ny_off;
                ctx.begin_path();
                ctx.ellipse(nx, ny, 5, 3.5, -0.4, 0, PI * 2.0);
                ctx.fill();
                // Stem.
                ctx.set_stroke_style("#181008");
                ctx.set_line_width(1.5);
                ctx.begin_path();
                ctx.move_to(nx + 4.0, ny - 1.0);
                ctx.line_to(nx + 4.0, ny - 22.0);
                ctx.stroke();
            }
        }
    })
);
