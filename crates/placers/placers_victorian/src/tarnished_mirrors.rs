//! `place_tarnished_mirrors`
use crate::shared::wall_candidates;
use game::prelude::*;
use placers_kit::place::ri;

pub(crate) fn place_tarnished_mirrors(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.975 {
        return;
    }
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let (cx, cz, cdx, cdz) = candidates[ri(candidates.len())];
    // Ornate gilded frame — gold w/ tarnish tinge.
    let frame_colors: [u32; 4] = [0xa08040, 0x8b6f30, 0x907040, 0x604828];
    let frame_mat = lambert!({ color: frame_colors[ri(frame_colors.len())], emissive: 0x1a1408 });
    let mirror_mat = lambert!({ map: tarnished_mirror_texture(), side: DoubleSide });
    let tm = group();
    // Frame outer bars.
    let (mw, mh, mt, mfw) = (0.40, 0.52, 0.020, 0.036);
    for (bar_w, bar_h, bx, by) in [
        (mw + mfw * 2.0, mfw, 0.0, mh / 2.0 + mfw / 2.0),
        (mw + mfw * 2.0, mfw, 0.0, -mh / 2.0 - mfw / 2.0),
        (mfw, mh, -mw / 2.0 - mfw / 2.0, 0.0),
        (mfw, mh, mw / 2.0 + mfw / 2.0, 0.0),
    ] {
        let bar = mesh(pg("TarnishedMirrors__bar", box_geo!(bar_w, bar_h, mt)), &frame_mat);
        bar.position().set(bx, by, 0);
        tm.add(&bar);
    }
    // Ornate corner decorations.
    for (cx_or, cy_or) in [
        (-mw / 2.0 - mfw / 2.0, mh / 2.0 + mfw / 2.0),
        (mw / 2.0 + mfw / 2.0, mh / 2.0 + mfw / 2.0),
        (-mw / 2.0 - mfw / 2.0, -mh / 2.0 - mfw / 2.0),
        (mw / 2.0 + mfw / 2.0, -mh / 2.0 - mfw / 2.0),
    ] {
        let corner_sphere = mesh(pg("TarnishedMirrors__rosette", sphere_geo!(0.020, 8, 6)), &frame_mat);
        corner_sphere.position().set(cx_or, cy_or, mt / 2.0);
        tm.add(&corner_sphere);
    }
    // Ornate top pediment.
    let pediment = mesh(pg("TarnishedMirrors__pediment", box_geo!(mw * 0.6, 0.025, mt * 0.7)), &frame_mat);
    pediment.position().set(0, mh / 2.0 + mfw + 0.012, 0);
    tm.add(&pediment);
    // Central decorative rosette on pediment.
    let rosette = mesh(pg("TarnishedMirrors__rosette", sphere_geo!(0.014, 8, 6)), &frame_mat);
    rosette.position().set(0, mh / 2.0 + mfw + 0.025, mt / 2.0);
    tm.add(&rosette);
    // Bottom pendant.
    let pendant = mesh(pg("TarnishedMirrors__pendant", sphere_geo!(0.012, 6, 5)), &frame_mat);
    pendant.position().set(0, -mh / 2.0 - mfw - 0.010, mt / 2.0);
    tm.add(&pendant);
    // Mirror surface w/ tarnish + crack texture.
    let surface = mesh(plane_geo!(mw, mh), &mirror_mat);
    surface.position().set(0, 0, mt / 2.0 + 0.001);
    tm.add(&surface);
    let wall_x = g.wx(cx) + cdx as f64 * (CELL * 0.5 - 0.014);
    let wall_z = g.wz(cz) + cdz as f64 * (CELL * 0.5 - 0.014);
    let center_y = 1.65;
    if cdx.abs() == 1 {
        tm.rotation().y = if cdx > 0 { -PI / 2.0 } else { PI / 2.0 };
        tm.position().set(wall_x, center_y, wall_z + (rnd() - 0.5) * 0.4);
    } else {
        tm.rotation().y = if cdz > 0 { PI } else { 0.0 };
        tm.position().set(wall_x + (rnd() - 0.5) * 0.4, center_y, wall_z);
    }
    tm.rotation().z = (rnd() - 0.5) * 0.05;
    g.world.add(&tm);
}

// Tarnished gilded ornate wall mirror texture.
game::lazy_static_tex!(
    tarnished_mirror_texture,
    canvas_tex(256, 320, |ctx, w, h| {
        // Base mirror gradient — dark silver reflection tone.
        let grad = ctx.create_linear_gradient(0, 0, w, h);
        grad.add_color_stop(0, "#3a3a45");
        grad.add_color_stop(0.5, "#5a5a68");
        grad.add_color_stop(1, "#2e2e38");
        ctx.set_fill_style(&grad);
        ctx.fill_rect(0, 0, w, h);
        // Tarnish patches — dark green/brown blotches at corners + edges.
        ctx.set_fill_style("rgba(30,40,20,0.45)");
        for _ in 0..6 {
            let tx = if rnd() < 0.6 { if rnd() < 0.5 { 0.0 } else { w } } else { rnd() * w };
            let ty = if rnd() < 0.6 { if rnd() < 0.5 { 0.0 } else { h } } else { rnd() * h };
            ctx.begin_path();
            let rx = 25.0 + rnd() * 30.0;
            let ry = 20.0 + rnd() * 25.0;
            let rot = rnd() * PI;
            ctx.ellipse(tx, ty, rx, ry, rot, 0, PI * 2.0);
            ctx.fill();
        }
        // Rust-red small spots scattered.
        ctx.set_fill_style("rgba(100,50,20,0.4)");
        for _ in 0..20 {
            ctx.begin_path();
            let (ax, ay, ar) = (rnd() * w, rnd() * h, 2.0 + rnd() * 4.0);
            ctx.arc(ax, ay, ar, 0, PI * 2.0);
            ctx.fill();
        }
        // 3-4 crack lines emanating from a starburst point.
        let ox = w * (0.3 + rnd() * 0.4);
        let oy = h * (0.3 + rnd() * 0.4);
        ctx.set_stroke_style("rgba(15,15,20,0.85)");
        ctx.set_line_width(1.5);
        let crack_count = 3 + (rnd() * 2.0).floor() as i32;
        for i in 0..crack_count {
            let angle = (i as f64 / crack_count as f64) * PI * 2.0 + rnd() * 0.5;
            // Zigzag crack line — up to 5 segments with random angle drift.
            let (mut cx, mut cy) = (ox, oy);
            let mut ang = angle;
            ctx.begin_path();
            ctx.move_to(cx, cy);
            for _ in 0..5 {
                let seg_len = 25.0 + rnd() * 30.0;
                ang += (rnd() - 0.5) * 0.6;
                cx += ang.dcos() * seg_len;
                cy += ang.dsin() * seg_len;
                ctx.line_to(cx, cy);
                if cx < 0.0 || cx > w || cy < 0.0 || cy > h {
                    break;
                }
            }
            ctx.stroke();
            // Secondary hairline branch (30% per crack).
            if rnd() < 0.3 {
                ctx.set_stroke_style("rgba(20,20,30,0.5)");
                ctx.set_line_width(0.7);
                ctx.begin_path();
                let mid_x = ox + angle.dcos() * 40.0;
                let mid_y = oy + angle.dsin() * 40.0;
                ctx.move_to(mid_x, mid_y);
                ctx.line_to(mid_x + (angle + 0.8).dcos() * 30.0, mid_y + (angle + 0.8).dsin() * 30.0);
                ctx.stroke();
                ctx.set_stroke_style("rgba(15,15,20,0.85)");
                ctx.set_line_width(1.5);
            }
        }
        // Small highlight dot at crack origin (impact point).
        ctx.set_fill_style("rgba(240,240,240,0.65)");
        ctx.begin_path();
        ctx.arc(ox, oy, 3, 0, PI * 2.0);
        ctx.fill();
        // Faint glass-warp reflection lines (2 subtle diagonals).
        ctx.set_stroke_style("rgba(200,210,220,0.10)");
        ctx.set_line_width(30);
        ctx.begin_path();
        ctx.move_to(0, h * 0.3);
        ctx.line_to(w, h * 0.2);
        ctx.stroke();
        ctx.begin_path();
        ctx.move_to(0, h * 0.7);
        ctx.line_to(w, h * 0.8);
        ctx.stroke();
    })
);
