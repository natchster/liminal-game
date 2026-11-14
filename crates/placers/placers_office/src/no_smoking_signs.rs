//! `place_no_smoking_signs`
use game::prelude::*;
use placers_kit::place::mount_slid_ref as wall_mount;
use placers_kit::place::ri as pick;
use placers_kit::place::wall_candidates;

// Wall no-smoking sign: 15% chance per level. Small round sign
// with pictogram + red slash. Institutional signage.
pub(crate) fn place_no_smoking_signs(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.85 {
        return;
    } // 15% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = candidates[pick(candidates.len())];
    let sign_mat = lambert!({ map: no_smoking_texture(), transparent: true, side: DoubleSide });
    let sign = mesh(circle_geo!(0.09, 20), &sign_mat);
    let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.008);
    let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.008);
    let sign_height = 1.65 + rnd() * 0.20;
    wall_mount(&sign, &c, wall_x, wall_z, sign_height, 1.0);
    sign.rotation().z = (rnd() - 0.5) * 0.05;
    g.world.add(&sign);
}

// No-smoking pictogram texture — circular white bg with dark
// cigarette icon and red slash across.
game::lazy_static_tex!(
    no_smoking_texture,
    canvas_tex_opt(96, 96, false, |ctx, w, h| {
        let (cx, cy) = (w / 2.0, h / 2.0);
        // White circular bg.
        ctx.set_fill_style("#f0f0f0");
        ctx.begin_path();
        ctx.arc(cx, cy, 42, 0, PI * 2.0);
        ctx.fill();
        // Red ring around.
        ctx.set_stroke_style("#c02020");
        ctx.set_line_width(6);
        ctx.begin_path();
        ctx.arc(cx, cy, 39, 0, PI * 2.0);
        ctx.stroke();
        // Cigarette icon: horizontal white rectangle with dark tip + red ember.
        ctx.set_fill_style("#2a2018");
        ctx.fill_rect(cx - 20.0, cy - 3.0, 30, 6);
        // Red ember tip at right.
        ctx.set_fill_style("#f04020");
        ctx.fill_rect(cx + 10.0, cy - 3.0, 8, 6);
        // Smoke wisps (two small curves upward).
        ctx.set_stroke_style("#a0a0a0");
        ctx.set_line_width(2);
        ctx.begin_path();
        ctx.move_to(cx - 15.0, cy - 5.0);
        ctx.quadratic_curve_to(cx - 12.0, cy - 12.0, cx - 15.0, cy - 18.0);
        ctx.stroke();
        ctx.begin_path();
        ctx.move_to(cx - 5.0, cy - 5.0);
        ctx.quadratic_curve_to(cx - 2.0, cy - 12.0, cx - 5.0, cy - 18.0);
        ctx.stroke();
        // Red diagonal slash across everything.
        ctx.set_stroke_style("#c02020");
        ctx.set_line_width(8);
        ctx.begin_path();
        ctx.move_to(cx - 30.0, cy - 30.0);
        ctx.line_to(cx + 30.0, cy + 30.0);
        ctx.stroke();
    })
);
