//! `place_restroom_signs`
use game::prelude::*;
use placers_kit::place::pick;
use placers_kit::place::wall_candidates;

// Restroom signs: 20% chance per level, 1 wall-mounted pictogram.
pub(crate) fn place_restroom_signs(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.80 {
        return;
    } // 20% chance
    let cands = wall_candidates(g);
    if cands.is_empty() {
        return;
    }
    let c = pick(&cands);
    let texs = restroom_sign_textures();
    let tex = texs[(rnd() * texs.len() as f64).floor() as usize].clone();
    let mat = lambert!({ map: tex, transparent: true, side: DoubleSide });
    let sign = mesh(plane_geo!(0.14, 0.20), &mat);
    let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.012);
    let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.012);
    let sign_height = 1.85 + rnd() * 0.2; // 1.85-2.05 m
    if c.dx.abs() == 1 {
        sign.rotation().y = if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 };
        sign.position().set(wall_x, sign_height, wall_z + (rnd() - 0.5) * 1.2);
    } else {
        sign.rotation().y = if c.dz > 0 { PI } else { 0.0 };
        sign.position().set(wall_x + (rnd() - 0.5) * 1.2, sign_height, wall_z);
    }
    g.world.add(&sign);
}

/// Restroom sign textures (M / W pictograms)
fn restroom_sign_textures() -> Vec<Tex> {
    thread_local! { static V: std::cell::OnceCell<Vec<Tex>> = const { std::cell::OnceCell::new() }; }
    V.with(|c| {
        c.get_or_init(|| {
            with_native(|| {
                vec![
                    canvas_tex_opt(64, 96, false, |ctx, w, h| {
                        ctx.set_fill_style("#484842");
                        ctx.fill_rect(0, 0, w, h);
                        ctx.set_fill_style("#f0e8d8");
                        // Simplified figure: circle head + trapezoidal body.
                        ctx.begin_path();
                        ctx.arc(w / 2.0, h / 2.0 - 22.0, 8, 0, PI * 2.0);
                        ctx.fill();
                        ctx.begin_path();
                        ctx.move_to(w / 2.0 - 16.0, h / 2.0 - 8.0);
                        ctx.line_to(w / 2.0 + 16.0, h / 2.0 - 8.0);
                        ctx.line_to(w / 2.0 + 12.0, h / 2.0 + 24.0);
                        ctx.line_to(w / 2.0 - 12.0, h / 2.0 + 24.0);
                        ctx.close_path();
                        ctx.fill();
                    }),
                    canvas_tex_opt(64, 96, false, |ctx, w, h| {
                        ctx.set_fill_style("#484842");
                        ctx.fill_rect(0, 0, w, h);
                        ctx.set_fill_style("#f0e8d8");
                        ctx.begin_path();
                        ctx.arc(w / 2.0, h / 2.0 - 22.0, 8, 0, PI * 2.0);
                        ctx.fill();
                        // Skirted figure: triangle body.
                        ctx.begin_path();
                        ctx.move_to(w / 2.0 - 14.0, h / 2.0 - 8.0);
                        ctx.line_to(w / 2.0 + 14.0, h / 2.0 - 8.0);
                        ctx.line_to(w / 2.0 + 20.0, h / 2.0 + 24.0);
                        ctx.line_to(w / 2.0 - 20.0, h / 2.0 + 24.0);
                        ctx.close_path();
                        ctx.fill();
                    }),
                ]
            })
        })
        .clone()
    })
}
