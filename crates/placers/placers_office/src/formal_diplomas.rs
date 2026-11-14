//! `place_formal_diplomas`
use game::prelude::*;
use placers_kit::place::wall_tiles;

pub(crate) fn place_formal_diplomas(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    }
    let walls = wall_tiles(g);
    if walls.is_empty() {
        return;
    }
    let (x, z, dx, dz) = walls[(rnd() * walls.len() as f64).floor() as usize];
    let frame_colors = [0x604020, 0xd0a040, 0x202020, 0x8b5a2b];
    let frame_mat = lambert!({ color: choice(&frame_colors) });
    let dip_mat = lambert!({ map: diploma_texture_2() });
    let dp = group();
    let mount_y = 1.55;
    let (frame_w, frame_h) = (0.22, 0.28);
    let frame = mesh(pg("FormalDiplomas__frame", box_geo!(frame_w, frame_h, 0.014)), &frame_mat);
    frame.position().set(0, mount_y, 0.011);
    dp.add(&frame);
    // Gold inner mat.
    let gold_mat = lambert!({ color: 0xc0a060 });
    let mat = mesh(pg("FormalDiplomas__mat", box_geo!(frame_w - 0.012, frame_h - 0.012, 0.002)), &gold_mat);
    mat.position().set(0, mount_y, 0.018);
    dp.add(&mat);
    let dip = mesh(plane_geo!(frame_w - 0.030, frame_h - 0.030), &dip_mat);
    dip.position().set(0, mount_y, 0.020);
    dp.add(&dip);
    dp.rotation().y = (-(dx as f64)).atan2(-(dz as f64));
    let px = g.wx(x) + dx as f64 * (CELL * 0.5 - 0.015);
    let pz = g.wz(z) + dz as f64 * (CELL * 0.5 - 0.015);
    dp.position().set(px, 0, pz);
    dp.rotation().z = (rnd() - 0.5) * 0.03;
    g.world.add(&dp);
}

// Framed diploma (wall).
game::lazy_static_tex!(
    diploma_texture_2,
    canvas_tex(200, 260, |ctx, w, h| {
        // Parchment background — aged cream.
        ctx.set_fill_style("#f0e4c8");
        ctx.fill_rect(0, 0, w, h);
        // Aging patches.
        ctx.set_fill_style("rgba(160,120,80,0.12)");
        for _ in 0..40 {
            ctx.begin_path();
            let ax = rnd() * w;
            let ay = rnd() * h;
            let ar = 6.0 + rnd() * 10.0;
            ctx.arc(ax, ay, ar, 0, PI * 2.0);
            ctx.fill();
        }
        // Ornate border — double frame lines + corner flourishes.
        ctx.set_stroke_style("#604020");
        ctx.set_line_width(3);
        ctx.stroke_rect(10, 10, w - 20.0, h - 20.0);
        ctx.set_line_width(1);
        ctx.stroke_rect(16, 16, w - 32.0, h - 32.0);
        // 4 corner ornaments (small filled diamonds).
        ctx.set_fill_style("#604020");
        let corner_xs = [16.0, w - 16.0];
        let corner_ys = [16.0, h - 16.0];
        for cx in corner_xs {
            for cy in corner_ys {
                ctx.begin_path();
                ctx.move_to(cx, cy - 6.0);
                ctx.line_to(cx + 6.0, cy);
                ctx.line_to(cx, cy + 6.0);
                ctx.line_to(cx - 6.0, cy);
                ctx.close_path();
                ctx.fill();
            }
        }
        // Top header — institution name.
        ctx.set_fill_style("#302010");
        ctx.set_font("bold 14px serif");
        ctx.set_text_align("center");
        let institutions = ["UNIVERSITY OF ARTS", "STATE COLLEGE", "ROYAL INSTITUTE", "ACADEMY OF SCIENCES", "MEMORIAL UNIVERSITY"];
        ctx.fill_text(choice(&institutions), w / 2.0, 40);
        // "This certifies that" line.
        ctx.set_font("italic 10px serif");
        ctx.fill_text("This is to certify that", w / 2.0, 68);
        // Recipient name — large cursive.
        ctx.set_font("bold italic 22px cursive");
        ctx.set_fill_style("#101010");
        let names = [
            "Sarah Michelle Anderson",
            "David James Chen",
            "Emma Rose Williams",
            "Michael Thomas Garcia",
            "Anna Grace Rodriguez",
            "James David Lee",
        ];
        ctx.fill_text(choice(&names), w / 2.0, 96);
        // "has completed" line.
        ctx.set_font("italic 10px serif");
        ctx.set_fill_style("#302010");
        ctx.fill_text("has satisfactorily completed the requirements for", w / 2.0, 118);
        // Degree name.
        ctx.set_font("bold 14px serif");
        let degrees = ["Bachelor of Arts", "Master of Science", "Doctor of Philosophy", "Bachelor of Science", "Master of Fine Arts"];
        ctx.fill_text(choice(&degrees), w / 2.0, 140);
        // Field of study.
        ctx.set_font("italic 12px serif");
        let fields = ["in Mathematics", "in English Literature", "in Chemistry", "in History", "in Music Performance", "in Biology"];
        ctx.fill_text(choice(&fields), w / 2.0, 158);
        // Date line.
        ctx.set_font("10px serif");
        let yr = 1965 + (rnd() * 30.0).floor() as i64;
        ctx.fill_text(&format!("Awarded this day, {}", yr), w / 2.0, 184);
        // Simulated signature lines — 2 short lines w/ wavy scripts + labels.
        ctx.set_stroke_style("rgba(30,20,10,0.85)");
        ctx.set_line_width(1.2);
        for sig in 0..2 {
            let sy = 208.0 + sig as f64 * 22.0;
            let sx = w * (0.25 + sig as f64 * 0.4);
            ctx.begin_path();
            ctx.move_to(sx - 25.0, sy);
            ctx.quadratic_curve_to(sx - 15.0, sy - 5.0, sx - 5.0, sy + 2.0);
            ctx.quadratic_curve_to(sx + 5.0, sy - 3.0, sx + 25.0, sy + 1.0);
            ctx.stroke();
            ctx.set_stroke_style("#302010");
            ctx.set_line_width(0.5);
            ctx.begin_path();
            ctx.move_to(sx - 30.0, sy + 6.0);
            ctx.line_to(sx + 30.0, sy + 6.0);
            ctx.stroke();
            ctx.set_fill_style("#302010");
            ctx.set_font("7px serif");
            ctx.fill_text(if sig == 0 { "Dean" } else { "President" }, sx, sy + 14.0);
            ctx.set_stroke_style("rgba(30,20,10,0.85)");
            ctx.set_line_width(1.2);
        }
        // Wax seal — red circle w/ inner detail.
        ctx.set_fill_style("#a02020");
        ctx.begin_path();
        ctx.arc(w - 40.0, h - 40.0, 18, 0, PI * 2.0);
        ctx.fill();
        // Seal star pattern.
        ctx.set_stroke_style("#602010");
        ctx.set_line_width(1);
        ctx.begin_path();
        for i in 0..5 {
            let ang = (i as f64 / 5.0) * PI * 2.0 - PI / 2.0;
            let (r1, r2) = (12.0, 5.0);
            let outer = if i % 2 == 0 { r1 } else { r2 };
            let px = w - 40.0 + ang.cos() * outer;
            let py = h - 40.0 + ang.sin() * outer;
            if i == 0 {
                ctx.move_to(px, py);
            } else {
                ctx.line_to(px, py);
            }
        }
        ctx.stroke();
    })
);
