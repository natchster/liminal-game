//! `place_cookie_tins`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

pub(crate) fn place_cookie_tins(g: &mut Game, _dist: &[i32]) {
    g.pp.cookie_tin_positions.clear(); // reset per level
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let tin_colors: [u32; 4] = [0xa04020, 0x408060, 0x2060a0, 0x604020];
    let tin_mat = lambert!({ color: pick(&tin_colors) });
    let lid_mat = lambert!({ map: cookie_tin_texture() });
    let tin = group();
    let body = mesh(pg("CookieTins__body", cyl_geo!(0.075, 0.075, 0.055, 16)), &tin_mat);
    body.position().y = 0.028;
    tin.add(&body);
    let lid = mesh(circle_geo!(0.075, 16), &lid_mat);
    lid.rotation().x = -PI / 2.0;
    lid.position().y = 0.056;
    tin.add(&lid);
    let rim = mesh(pg("CookieTins__rim", torus_geo!(0.075, 0.003, 4, 16)), &tin_mat);
    rim.rotation().x = PI / 2.0;
    rim.position().y = 0.056;
    tin.add(&rim);
    // Subtle lid Y-jitter — 35% chance. Pairs w/ the tin's rattle audio:
    // when the tin "rattles", visually the lid is also settling.
    if rnd() < 0.35 {
        let lid_base_y = 0.056;
        let jitter_phase = rnd() * PI * 2.0;
        let jitter_phase2 = rnd() * PI * 2.0;
        let lid2 = lid.clone();
        g.animate(move |_g, _dt, t| {
            // Multi-freq combined sine, very small amplitude (~0.6 mm).
            lid2.position().y = lid_base_y + (t * 2.7 + jitter_phase).sin() * 0.0006 + (t * 4.3 + jitter_phase2).sin() * 0.0004;
        });
    }
    tin.position().set(g.wx(x) + (rnd() - 0.5) * 2.2, 0, g.wz(z) + (rnd() - 0.5) * 2.2);
    tin.rotation().y = rnd() * PI * 2.0;
    g.world.add(&tin);
    let p = tin.pos();
    g.pp.cookie_tin_positions.push(v3(p.x, 0, p.z));
}

// Cookie tin box: 6% chance per level.
game::lazy_static_tex!(
    cookie_tin_texture,
    canvas_tex(160, 160, |ctx, w, h| {
        ctx.set_fill_style("#e8d8a8");
        ctx.fill_rect(0, 0, w, h);
        let (cx, cy) = (w / 2.0, h / 2.0);
        ctx.set_stroke_style("#804020");
        ctx.set_line_width(3);
        ctx.begin_path();
        ctx.arc(cx, cy, w * 0.42, 0, PI * 2.0);
        ctx.stroke();
        ctx.set_fill_style("#c04040");
        for i in 0..8 {
            let a = (i as f64 / 8.0) * PI * 2.0;
            ctx.begin_path();
            ctx.arc(cx + a.cos() * w * 0.36, cy + a.sin() * w * 0.36, 6, 0, PI * 2.0);
            ctx.fill();
        }
        ctx.set_fill_style("#804020");
        ctx.begin_path();
        ctx.arc(cx, cy, 20, 0, PI * 2.0);
        ctx.fill();
        ctx.set_fill_style("#f0e0a0");
        ctx.begin_path();
        for i in 0..10 {
            let a = (i as f64 / 10.0) * PI * 2.0 - PI / 2.0;
            let r = if i % 2 == 0 { 14.0 } else { 6.0 };
            let px = cx + a.cos() * r;
            let py = cy + a.sin() * r;
            if i == 0 {
                ctx.move_to(px, py);
            } else {
                ctx.line_to(px, py);
            }
        }
        ctx.close_path();
        ctx.fill();
    })
);
