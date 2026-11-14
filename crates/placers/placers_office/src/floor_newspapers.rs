//! `place_floor_newspapers`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::ri as pick;

// Folded newspaper on floor: 20% chance per level, 1 paper.
pub(crate) fn place_floor_newspapers(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.80 {
        return;
    } // 20% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[pick(open.len())];
    // Per-weather aged tint.
    let paper_tint = newspaper_tint_by_weather(g.weather_key());
    let paper_mat = lambert!({ map: newspaper_texture_2(), color: paper_tint, side: DoubleSide });
    let paper = mesh(plane_geo!(0.24, 0.16), &paper_mat);
    paper.rotation().x = -PI / 2.0;
    paper.rotation().z = rnd() * PI * 2.0;
    paper.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0.004, g.wz(z) + (rnd() - 0.5) * 2.4);
    g.world.add(&paper);
}

// Folded newspaper on floor: small rectangular plane with dark
// column-lines suggesting print.
game::lazy_static_tex!(
    newspaper_texture_2,
    canvas_tex_opt(96, 64, false, |ctx, w, h| {
        // Cream paper bg.
        ctx.set_fill_style("#c8bc9e");
        ctx.fill_rect(0, 0, w, h);
        // Dark headline bar at top.
        ctx.set_fill_style("#2a2015");
        ctx.fill_rect(4, 4, w - 8.0, 8);
        // 3 columns of tiny "text" lines.
        ctx.set_fill_style("#3a2c1a");
        let col_w = (w - 12.0) / 3.0;
        for col in 0..3 {
            let cx = 4.0 + col as f64 * (col_w + 2.0);
            let mut ly = 18.0;
            while ly < h - 4.0 {
                if rnd() < 0.75 {
                    let lw = col_w * (0.35 + rnd() * 0.65);
                    ctx.fill_rect(cx, ly, lw, 1);
                }
                ly += 3.0;
            }
        }
        // Grunge speckle.
        for _ in 0..20 {
            ctx.set_fill_style(&format!("rgba(0,0,0,{})", 0.06 + rnd() * 0.1));
            let (x, y) = (rnd() * w, rnd() * h);
            ctx.fill_rect(x, y, 1, 1);
        }
    })
);
