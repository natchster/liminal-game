//! `place_seed_packets`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

pub(crate) fn place_seed_packets(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let packet_mat = lambert!({ map: seed_packet_texture(), side: DoubleSide });
    let packet = mesh(plane_geo!(0.055, 0.080), &packet_mat);
    packet.rotation().x = -PI / 2.0;
    packet.position().set(g.wx(x) + (rnd() - 0.5) * 2.5, 0.003, g.wz(z) + (rnd() - 0.5) * 2.5);
    packet.rotation().z = rnd() * PI * 2.0;
    g.world.add(&packet);
}

// Seed packet: 8% chance per level. Colored paper envelope w/ printed
// flower image.
game::lazy_static_tex!(
    seed_packet_texture,
    canvas_tex(80, 120, |ctx, w, h| {
        // Random pastel background.
        let bgs = ["#f0e0a0", "#e0b0a0", "#a0c0e0", "#c0e0a0"];
        ctx.set_fill_style(bgs[(rnd() * bgs.len() as f64).floor() as usize]);
        ctx.fill_rect(0, 0, w, h);
        // Border stripe at top.
        ctx.set_fill_style("#404020");
        ctx.fill_rect(0, 0, w, 10);
        // "Seed name" title bar (dark bar).
        ctx.set_fill_style("#202010");
        ctx.fill_rect(6, 16, w - 12.0, 6);
        // Center flower illustration (5 petal circle + center).
        let (cx, cy) = (w / 2.0, h * 0.55);
        let flower_colors = ["#c02020", "#f0c020", "#d060c0", "#c04060", "#e08040"];
        ctx.set_fill_style(flower_colors[(rnd() * flower_colors.len() as f64).floor() as usize]);
        for p in 0..5 {
            let a = (p as f64 / 5.0) * PI * 2.0;
            ctx.begin_path();
            ctx.arc(cx + a.dcos() * 12.0, cy + a.dsin() * 12.0, 8, 0, PI * 2.0);
            ctx.fill();
        }
        // Center yellow dot.
        ctx.set_fill_style("#f0e020");
        ctx.begin_path();
        ctx.arc(cx, cy, 5, 0, PI * 2.0);
        ctx.fill();
        // Bottom info bar.
        ctx.set_fill_style("#404020");
        for i in 0..3 {
            ctx.fill_rect(6, h - 24.0 + i as f64 * 6.0, w - 12.0 - rnd() * 20.0, 2);
        }
    })
);
