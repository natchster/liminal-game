//! `place_cross_stitch_samplers`
use game::prelude::*;
use placers_kit::place::face_wall;
use placers_kit::place::pick;
use placers_kit::place::wall_pos;
use placers_kit::place::wall_tiles;

pub(crate) fn place_cross_stitch_samplers(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    }
    let wt = wall_tiles(g);
    if wt.is_empty() {
        return;
    }
    let (x, z, dx, dz) = pick(&wt);
    let wood_colors: [u32; 4] = [0x8b5a2b, 0x604020, 0xa07040, 0x704a20];
    let frame_mat = lambert!({ color: pick(&wood_colors) });
    let cloth_mat = lambert!({ map: cross_stitch_texture() });
    let sampler = group();
    let mount_y = 1.55;
    // Frame — outer box.
    let (frame_w, frame_h) = (0.28, 0.22);
    let frame = mesh(pg("CrossStitchSamplers__frame", box_geo!(frame_w, frame_h, 0.016)), &frame_mat);
    frame.position().set(0, mount_y, 0.012);
    sampler.add(&frame);
    // Cloth face — slightly inset.
    let cloth = mesh(plane_geo!(frame_w - 0.020, frame_h - 0.020), &cloth_mat);
    cloth.position().set(0, mount_y, 0.021);
    sampler.add(&cloth);
    sampler.rotation().y = face_wall(dx, dz);
    let (px, pz) = wall_pos(g, x, z, dx, dz, 0.015);
    sampler.position().set(px, 0, pz);
    // Slight tilt.
    sampler.rotation().z = (rnd() - 0.5) * 0.03;
    g.world.add(&sampler);
}

// Framed cross-stitch sampler w/ alphabet (wall):
// 3% chance per level. Traditional embroidered alphabet
// on aida cloth in wooden frame.
game::lazy_static_tex!(
    cross_stitch_texture,
    canvas_tex(256, 192, |ctx, w, h| {
        // Aida cloth base — cream/tan.
        ctx.set_fill_style("#ecdfc0");
        ctx.fill_rect(0, 0, w, h);
        // Aida cloth grid — 8×8px cells w/ subtle grid lines.
        ctx.set_stroke_style("rgba(180, 160, 130, 0.35)");
        ctx.set_line_width(1);
        let mut i = 0.0;
        while i <= w {
            ctx.begin_path();
            ctx.move_to(i, 0);
            ctx.line_to(i, h);
            ctx.stroke();
            i += 8.0;
        }
        let mut j = 0.0;
        while j <= h {
            ctx.begin_path();
            ctx.move_to(0, j);
            ctx.line_to(w, j);
            ctx.stroke();
            j += 8.0;
        }
        // Alphabet — 2 rows of 13 chars in cross-stitch style.
        // Uses filled small squares (2×2 blocks) to simulate stitches.
        let row_colors = ["#a03050", "#40608a"]; // top row + bottom row
        let alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZ";
        ctx.set_text_align("center");
        ctx.set_text_baseline("middle");
        ctx.set_font("bold 18px monospace");
        for row in 0..2usize {
            ctx.set_fill_style(row_colors[row]);
            for i in 0..13usize {
                let k = row * 13 + i;
                let ch = &alphabet[k..k + 1];
                let cx = 20.0 + i as f64 * 17.0;
                let cy = 34.0 + row as f64 * 26.0;
                ctx.fill_text(ch, cx, cy);
            }
        }
        // Simple house motif center-bottom.
        let (cx, cy) = (w / 2.0, h * 0.75);
        // House body.
        ctx.set_fill_style("#a06040");
        ctx.fill_rect(cx - 22.0, cy - 4.0, 44, 24);
        // Roof triangle.
        ctx.set_fill_style("#603020");
        ctx.begin_path();
        ctx.move_to(cx - 26.0, cy - 4.0);
        ctx.line_to(cx, cy - 24.0);
        ctx.line_to(cx + 26.0, cy - 4.0);
        ctx.close_path();
        ctx.fill();
        // Door.
        ctx.set_fill_style("#402020");
        ctx.fill_rect(cx - 6.0, cy + 4.0, 12, 16);
        // 2 flanking flowers (small crosses + circle centers).
        for side in [-1.0, 1.0] {
            ctx.set_fill_style("#60a040");
            ctx.fill_rect(cx + side * 44.0 - 2.0, cy + 4.0, 4, 16);
            ctx.set_fill_style("#e04040");
            ctx.begin_path();
            ctx.arc(cx + side * 44.0, cy - 2.0, 6, 0, PI * 2.0);
            ctx.fill();
        }
        // Border stitch line.
        ctx.set_stroke_style("#603020");
        ctx.set_line_width(3);
        ctx.set_line_dash(&[6.0, 4.0]);
        ctx.stroke_rect(6, 6, w - 12.0, h - 12.0);
        ctx.set_line_dash(&[]);
        // Signed initials + year in tiny bottom-right cross-stitch.
        ctx.set_fill_style("#603030");
        ctx.set_font("10px monospace");
        ctx.set_text_align("right");
        ctx.fill_text("E.M. '87", w - 12.0, h - 10.0);
    })
);
