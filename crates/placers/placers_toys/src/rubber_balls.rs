//! `place_rubber_balls`: Small rubber ball: 10% chance per level. A 4 in hollow rubber ball of two moulded halves, each its own colour, joined at a seam (Blender batch r17); years on the floor have let the air out of it, so it sags flat where it rests, and faded, scuffed and grimed it.
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

/// Small rubber ball: 10% chance per level. A 4 in hollow rubber ball
/// of two moulded halves, each its own colour, joined at a seam (Blender
/// batch r17); years on the floor have let the air out of it, so it sags
/// flat where it rests, and faded, scuffed and grimed it.
pub(crate) fn place_rubber_balls(g: &mut Game, _dist: &[i32]) {
    use game::assets::props::prop_part_as_modelled;
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    // the old colours stand for the two halves' rubber, faded
    let ball_colors_a = [0xd82020, 0x2050d8, 0xd8b820, 0x30a030];
    let ball_colors_b = [0xf0f0f0, 0x181818, 0xd8b820, 0x8020d8];
    let c_a = pick(&ball_colors_a);
    let c_b = pick(&ball_colors_b);
    let ka = ball_colors_a.iter().position(|&c| c == c_a).unwrap_or(0);
    let kb = ball_colors_b.iter().position(|&c| c == c_b).unwrap_or(0);
    let rubber_a: [u32; 4] = [0x8e3a30, 0x3b5478, 0xb09048, 0x4d6d45];
    let rubber_b: [u32; 4] = [0xbcb5a3, 0x2c2a28, 0xa98a46, 0x5c4a68];
    let wear = rubber_ball_tex();
    let ball = group();
    for (name, col) in [("Real__rubberBallHalfA", rubber_a[ka]), ("Real__rubberBallHalfB", rubber_b[kb])] {
        if let Some(geo) = prop_part_as_modelled(name) {
            ball.add(&mesh(geo, phong!({ color: col, map: &wear, specular: 0x1c1c1c, shininess: 14 })));
        }
    }
    ball.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    ball.rotation().y = rnd() * PI * 2.0;
    g.world.add(&ball);
}

/// The rubber's wear (multiplies its colour): faded to a chalky bloom in
/// patches, grime rolled into it, scuffs where it hit things, specks of grit
fn rubber_ball_tex() -> Tex {
    canvas_tex(128, 128, |ctx, w, h| {
        ctx.set_fill_style("#e4dfd4");
        ctx.fill_rect(0, 0, w, h);
        // the bloom: rubber goes chalky and pale where the light found it
        for _ in 0..12 {
            let (x, y) = (rnd() * w, rnd() * h);
            let rr = 14.0 + rnd() * 30.0;
            let gr = ctx.create_radial_gradient(x, y, 0, x, y, rr);
            gr.add_color_stop(0, "rgba(255,252,244,0.35)");
            gr.add_color_stop(1, "rgba(255,252,244,0)");
            ctx.set_fill_style(gr);
            ctx.fill_rect(x - rr, y - rr, rr * 2.0, rr * 2.0);
        }
        // grime rolled in: soft dark patches
        for _ in 0..16 {
            let (x, y) = (rnd() * w, rnd() * h);
            let rr = 8.0 + rnd() * 22.0;
            let gr = ctx.create_radial_gradient(x, y, 0, x, y, rr);
            gr.add_color_stop(0, "rgba(64,56,44,0.22)");
            gr.add_color_stop(1, "rgba(64,56,44,0)");
            ctx.set_fill_style(gr);
            ctx.fill_rect(x - rr, y - rr, rr * 2.0, rr * 2.0);
        }
        // scuffs: short pale smears and dark scrapes
        for _ in 0..26 {
            let (x, y) = (rnd() * w, rnd() * h);
            let a = rnd() * PI;
            let l = 3.0 + rnd() * 9.0;
            ctx.set_line_width(1.0 + rnd() * 1.6);
            ctx.set_stroke_style(if rnd() < 0.6 { "rgba(250,246,236,0.35)" } else { "rgba(40,34,28,0.35)" });
            ctx.begin_path();
            ctx.move_to(x, y);
            ctx.line_to(x + a.cos() * l, y + a.sin() * l);
            ctx.stroke();
        }
        for _ in 0..90 {
            ctx.set_fill_style(format!("rgba(52,46,38,{:.2})", 0.25 + rnd() * 0.35));
            ctx.fill_rect(rnd() * w, rnd() * h, 1, 1);
        }
    })
}
