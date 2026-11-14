//! `place_wall_watches`
use game::prelude::*;
use placers_kit::place::mount_slid_ref as wall_mount;
use placers_kit::place::ri as pick;
use placers_kit::place::wall_candidates;

// Small wall watch: 12% chance per level. Tiny 8-cm circle
// on wall — differs from the 20-cm analog clock and the digital one.
pub(crate) fn place_wall_watches(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.88 {
        return;
    } // 12% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = candidates[pick(candidates.len())];
    let watch_mat = lambert!({ map: wall_watch_texture(), transparent: true, side: DoubleSide });
    let watch = mesh(circle_geo!(0.04, 16), &watch_mat);
    let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.008);
    let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.008);
    let watch_height = 1.6 + rnd() * 0.30;
    wall_mount(&watch, &c, wall_x, wall_z, watch_height, 1.0);
    g.world.add(&watch);
    // 10% chance: cracked-glass overlay atop watch face.
    if rnd() < 0.10 {
        let wc_mat = basic_mat!({ map: cracked_glass_texture(), transparent: true, depthWrite: false, side: DoubleSide });
        let wcrack = mesh(plane_geo!(0.08, 0.08), &wc_mat);
        let wp = watch.pos();
        let wr = watch.rot();
        wcrack.position().copy(&wp);
        wcrack.rotation().copy(&wr);
        wcrack.position().x -= c.dx as f64 * 0.001;
        wcrack.position().z -= c.dz as f64 * 0.001;
        g.world.add(&wcrack);
    }
}

// Tiny analog watch texture — small wristwatch-style face on
// wall. Very small circle with minimal ticks.
game::lazy_static_tex!(
    wall_watch_texture,
    canvas_tex_opt(64, 64, false, |ctx, w, h| {
        let (cx, cy) = (w / 2.0, h / 2.0);
        ctx.set_fill_style("#d8d0c0");
        ctx.begin_path();
        ctx.arc(cx, cy, 26, 0, PI * 2.0);
        ctx.fill();
        ctx.set_stroke_style("#2a2018");
        ctx.set_line_width(2);
        ctx.begin_path();
        ctx.arc(cx, cy, 26, 0, PI * 2.0);
        ctx.stroke();
        // 4 tick marks at 12/3/6/9.
        for i in 0..4 {
            let a = (i as f64 / 4.0) * PI * 2.0 - PI / 2.0;
            ctx.set_line_width(1.5);
            ctx.begin_path();
            ctx.move_to(cx + a.cos() * 20.0, cy + a.sin() * 20.0);
            ctx.line_to(cx + a.cos() * 24.0, cy + a.sin() * 24.0);
            ctx.stroke();
        }
        // Frozen at 10:10 for classic watch-face look.
        ctx.set_stroke_style("#1a1408");
        ctx.set_line_width(2);
        // Hour hand pointing 10.
        let h_ang = ((10.0f64 % 12.0) / 12.0) * PI * 2.0 - PI / 2.0;
        ctx.begin_path();
        ctx.move_to(cx, cy);
        ctx.line_to(cx + h_ang.cos() * 12.0, cy + h_ang.sin() * 12.0);
        ctx.stroke();
        // Minute hand pointing 2.
        let m_ang = (10.0 / 60.0) * PI * 2.0 - PI / 2.0;
        ctx.begin_path();
        ctx.move_to(cx, cy);
        ctx.line_to(cx + m_ang.cos() * 18.0, cy + m_ang.sin() * 18.0);
        ctx.stroke();
        ctx.set_fill_style("#1a1408");
        ctx.begin_path();
        ctx.arc(cx, cy, 2, 0, PI * 2.0);
        ctx.fill();
    })
);
