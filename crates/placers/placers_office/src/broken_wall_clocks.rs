//! `place_broken_wall_clocks`
use game::prelude::*;
use placers_kit::place::mount_on_wall;
use placers_kit::place::wall_candidates;

// Broken wall clock: 5% chance per level. Clock face frozen at an odd time.
pub(crate) fn place_broken_wall_clocks(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = candidates[(rnd() * candidates.len() as f64).floor() as usize];
    let case_mat = lambert!({ color: 0x2a1810 });
    let face_mat = lambert!({ map: broken_clock_texture(), side: DoubleSide });
    let clock = group();
    let (r, t) = (0.14, 0.020);
    // Case cylinder.
    let case_r = mesh(pg("BrokenWallClocks__case", cyl_geo!(r, r, t, 16)), &case_mat);
    case_r.rotation().x = PI / 2.0;
    clock.add(&case_r);
    // Rim (small torus).
    let rim = mesh(pg("BrokenWallClocks__bezel", torus_geo!(r, 0.008, 6, 16)), &case_mat);
    rim.rotation().x = 0.0;
    rim.position().z = t / 2.0 + 0.001;
    clock.add(&rim);
    // Face.
    let face = mesh(circle_geo!(r - 0.008, 16), &face_mat);
    face.position().z = t / 2.0 + 0.003;
    clock.add(&face);
    mount_on_wall(g, &clock, &c, 0.014, 1.90, 0.6);
    clock.rotation().z = (rnd() - 0.5) * 0.06;
    g.world.add(&clock);
}

game::lazy_static_tex!(
    broken_clock_texture,
    canvas_tex(128, 128, |ctx, w, h| {
        ctx.set_fill_style("#e8dcc0");
        ctx.begin_path();
        ctx.arc(w / 2.0, h / 2.0, w / 2.0 - 4.0, 0, PI * 2.0);
        ctx.fill();
        ctx.set_stroke_style("#181008");
        ctx.set_line_width(3);
        ctx.stroke();
        // Age grime.
        for _ in 0..15 {
            ctx.set_fill_style(format!("rgba(100,80,40,{})", 0.05 + rnd() * 0.15));
            ctx.begin_path();
            ctx.arc(20.0 + rnd() * (w - 40.0), 20.0 + rnd() * (h - 40.0), 4.0 + rnd() * 8.0, 0, PI * 2.0);
            ctx.fill();
        }
        // Hour marks.
        ctx.set_stroke_style("#181008");
        ctx.set_line_width(3);
        for i in 0..12 {
            let a = -PI / 2.0 + (i as f64 / 12.0) * PI * 2.0;
            let (r0, r1) = (44.0, 52.0);
            ctx.begin_path();
            ctx.move_to(w / 2.0 + a.cos() * r0, h / 2.0 + a.sin() * r0);
            ctx.line_to(w / 2.0 + a.cos() * r1, h / 2.0 + a.sin() * r1);
            ctx.stroke();
        }
        // Broken hands at 3:33 (unsettling non-round time).
        ctx.set_line_width(4);
        ctx.begin_path();
        ctx.move_to(w / 2.0, h / 2.0);
        let hour_a = -PI / 2.0 + PI / 2.0 * 1.11; // ~3:33
        ctx.line_to(w / 2.0 + hour_a.cos() * 24.0, h / 2.0 + hour_a.sin() * 24.0);
        ctx.stroke();
        ctx.set_line_width(3);
        ctx.begin_path();
        ctx.move_to(w / 2.0, h / 2.0);
        let min_a = -PI / 2.0 + PI * 1.1; // ~33 min
        ctx.line_to(w / 2.0 + min_a.cos() * 38.0, h / 2.0 + min_a.sin() * 38.0);
        ctx.stroke();
        ctx.begin_path();
        ctx.arc(w / 2.0, h / 2.0, 4, 0, PI * 2.0);
        ctx.set_fill_style("#181008");
        ctx.fill();
        // Big crack across face.
        ctx.set_stroke_style("rgba(0,0,0,0.55)");
        ctx.set_line_width(2);
        ctx.begin_path();
        ctx.move_to(30, 20);
        ctx.line_to(80, 60);
        ctx.line_to(60, 95);
        ctx.line_to(100, 110);
        ctx.stroke();
        // Small branch cracks.
        ctx.set_line_width(1);
        for _ in 0..5 {
            ctx.begin_path();
            ctx.move_to(60.0 + rnd() * 30.0, 40.0 + rnd() * 40.0);
            ctx.line_to(60.0 + rnd() * 40.0, 40.0 + rnd() * 40.0);
            ctx.stroke();
        }
    })
);
