//! `place_exit_signs`
use game::prelude::*;
use placers_kit::place::pick;
use placers_kit::place::wall_candidates;

// Exit signs: 15% chance per level, at most 1 sign, mounted high
// (2.4-2.7 m). Faded green glow, optionally flickering.
pub(crate) fn place_exit_signs(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.85 {
        return;
    } // 15% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = pick(&candidates);
    // Frame: dark box behind the glowing plane.
    let frame_mat = lambert!({ color: 0x181812 });
    let frame = mesh(pg("ExitSigns__housing", box_geo!(0.42, 0.20, 0.04)), &frame_mat);
    // Glowing plane in front. Per-sign material for animator flicker.
    let glow_mat = basic_mat!({ map: exit_sign_texture_2(), transparent: true, opacity: 0.85, side: DoubleSide });
    // Flicker animator: 30% chance the sign flickers — stable at 0.85,
    // briefly dips to 0.15 for 40 ms at random 3-8 s intervals.
    if rnd() < 0.30 {
        let mut flicker_t = 3.0 + rnd() * 5.0;
        let mut dip_t = 0.0;
        let gm = glow_mat.clone();
        g.animate(move |_g, dt, _t| {
            if dip_t > 0.0 {
                dip_t -= dt;
                gm.m().opacity = if dip_t > 0.0 { 0.15 } else { 0.85 };
            } else {
                flicker_t -= dt;
                if flicker_t <= 0.0 {
                    dip_t = 0.04; // 40 ms brief dip
                    flicker_t = 3.0 + rnd() * 5.0;
                }
            }
        });
    }
    let glow = mesh(plane_geo!(0.40, 0.18), &glow_mat);
    // housing back on the wall (4 cm deep); glow 1 mm in front of its face
    let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.02);
    let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.02);
    let sign_height = 2.4 + rnd() * 0.3;
    if c.dx.abs() == 1 {
        let rot = if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 };
        frame.rotation().y = rot;
        glow.rotation().y = rot;
        let jit_z = (rnd() - 0.5) * 1.2;
        frame.position().set(wall_x, sign_height, wall_z + jit_z);
        glow.position().set(wall_x - c.dx as f64 * 0.021, sign_height, wall_z + jit_z);
    } else {
        let rot = if c.dz > 0 { PI } else { 0.0 };
        frame.rotation().y = rot;
        glow.rotation().y = rot;
        let jit_x = (rnd() - 0.5) * 1.2;
        frame.position().set(wall_x + jit_x, sign_height, wall_z);
        glow.position().set(wall_x + jit_x, sign_height, wall_z - c.dz as f64 * 0.021);
    }
    g.world.add(&frame);
    g.world.add(&glow);
}

// Exit sign canvas texture — bright green EXIT text on dark background.
game::lazy_static_tex!(
    exit_sign_texture_2,
    canvas_tex_opt(96, 40, false, |ctx, w, h| {
        ctx.set_fill_style("#1a1a12");
        ctx.fill_rect(0, 0, w, h);
        ctx.set_fill_style("#40c860");
        ctx.set_font("bold 28px sans-serif");
        ctx.set_text_align("center");
        ctx.set_text_baseline("middle");
        ctx.fill_text("EXIT", w / 2.0, h / 2.0 + 2.0);
        // Slight scan-line grain.
        let mut y = 0.0;
        while y < h {
            ctx.set_fill_style("rgba(0,0,0,0.06)");
            ctx.fill_rect(0, y, w, 1);
            y += 2.0;
        }
    })
);
