//! `place_constellation_embroidery`
use game::prelude::*;
use placers_kit::place::face_wall;
use placers_kit::place::pick;
use placers_kit::place::wall_tiles;

pub(crate) fn place_constellation_embroidery(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.98 {
        return;
    }
    let wt = wall_tiles(g);
    if wt.is_empty() {
        return;
    }
    let (x, z, dx, dz) = wt[(rnd() * wt.len() as f64).floor() as usize];
    let frame_colors = [0x8b5a2b, 0x604020, 0xc0a060, 0x2a1810];
    let frame_mat = lambert!({ color: choice(&frame_colors) });
    let emb_mat = lambert!({ map: constellation_embroidery_texture() });
    let ce = group();
    let mount_y = 1.60;
    let (frame_w, frame_h) = (0.22, 0.26);
    let frame = mesh(pg("ConstellationEmbroidery__frame", box_geo!(frame_w, frame_h, 0.014)), &frame_mat);
    frame.position().set(0, mount_y, 0.011);
    ce.add(&frame);
    let emb = mesh(plane_geo!(frame_w - 0.020, frame_h - 0.020), &emb_mat);
    emb.position().set(0, mount_y, 0.019);
    ce.add(&emb);
    ce.rotation().y = face_wall(dx, dz);
    ce.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - 0.015), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - 0.015));
    ce.rotation().z = (rnd() - 0.5) * 0.03;
    g.world.add(&ce);
}

game::lazy_static_tex!(
    constellation_embroidery_texture,
    canvas_tex(200, 240, |ctx, w, h| {
        // Deep blue linen background w/ subtle woven texture.
        ctx.set_fill_style("#101830");
        ctx.fill_rect(0, 0, w, h);
        // Subtle weave (grid of tiny lighter dots for linen texture).
        ctx.set_fill_style("rgba(80,100,140,0.15)");
        let (wi, hi) = (w as i32, h as i32);
        let mut y = 0;
        while y < hi {
            let mut x = 0;
            while x < wi {
                if (x + y) % 6 == 0 {
                    ctx.fill_rect(x, y, 1, 1);
                }
                x += 3;
            }
            y += 3;
        }
        // Title top.
        ctx.set_fill_style("#f0e0a0");
        ctx.set_font("italic bold 14px serif");
        ctx.set_text_align("center");
        let chosen = pick(&["ORION", "URSA MAJOR", "CASSIOPEIA", "CYGNUS", "DRACO", "LEO"]);
        ctx.fill_text(chosen, w / 2.0, 24);
        // 30-50 tiny thread "stars" scattered around.
        ctx.set_fill_style("#f0f0e8");
        let star_count = 30 + (rnd() * 20.0).floor() as i32;
        let mut stars: Vec<(f64, f64)> = vec![];
        for _ in 0..star_count {
            let sx = 20.0 + rnd() * (w - 40.0);
            let sy = 40.0 + rnd() * (h - 80.0);
            let sr = 0.8 + rnd() * 1.2;
            stars.push((sx, sy));
            // Cross-stitch style star.
            ctx.begin_path();
            ctx.arc(sx, sy, sr, 0, PI * 2.0);
            ctx.fill();
            if sr > 1.5 {
                ctx.set_stroke_style("rgba(240,240,232,0.8)");
                ctx.set_line_width(0.5);
                ctx.begin_path();
                ctx.move_to(sx - sr - 1.0, sy);
                ctx.line_to(sx + sr + 1.0, sy);
                ctx.move_to(sx, sy - sr - 1.0);
                ctx.line_to(sx, sy + sr + 1.0);
                ctx.stroke();
            }
        }
        // Constellation lines — connect 5-8 nearby stars.
        let line_count = 5 + (rnd() * 4.0).floor() as i32;
        ctx.set_stroke_style("rgba(240,240,232,0.5)");
        ctx.set_line_width(0.6);
        let mut prev = (rnd() * stars.len() as f64).floor() as usize;
        for _ in 0..line_count {
            // Find a nearby star (identity compare: by index).
            let mut best: Option<usize> = None;
            let mut best_dist = f64::INFINITY;
            for (j, s) in stars.iter().enumerate() {
                if j == prev {
                    continue;
                }
                let d = hypot2(s.0 - stars[prev].0, s.1 - stars[prev].1);
                if d > 8.0 && d < best_dist {
                    best_dist = d;
                    best = Some(j);
                }
            }
            let Some(b) = best else { break };
            // Draw stitched line (dashed for cross-stitch feel).
            ctx.save();
            ctx.set_line_dash(&[2.0, 2.0]);
            ctx.begin_path();
            ctx.move_to(stars[prev].0, stars[prev].1);
            ctx.line_to(stars[b].0, stars[b].1);
            ctx.stroke();
            ctx.restore();
            prev = b;
        }
        // Border stitching — 4-side gold thread line at edges.
        ctx.set_stroke_style("#c0a060");
        ctx.set_line_width(1.5);
        ctx.save();
        ctx.set_line_dash(&[4.0, 2.0]);
        ctx.stroke_rect(10, 10, w - 20.0, h - 20.0);
        ctx.restore();
        // Bottom signature/date.
        ctx.set_fill_style("#c0a060");
        ctx.set_font("italic 8px cursive");
        ctx.set_text_align("center");
        let years = [1978, 1985, 1962, 1993, 1971];
        let who = pick(&["E.M.", "A.R.", "S.C.", "M.L."]);
        let year = years[(rnd() * years.len() as f64).floor() as usize];
        ctx.fill_text(&format!("Stitched by {}, {}", who, year), w / 2.0, h - 12.0);
    })
);
