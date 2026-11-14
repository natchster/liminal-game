//! `place_single_socks`
use crate::shared::r08_soft;
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::scatter;

// Single sock: 10% chance per level: a crew sock lost and lying limp on
// the floor, grey with dirt (Blender batch r08: the knit body, its heel and
// toe, the two bands knitted round a sports sock's cuff).
pub(crate) fn place_single_socks(g: &mut Game, _dist: &[i32]) {
    use game::assets::props::prop_part_as_modelled;
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = choice(&open);
    // the old colours stand for the socks people lose: a white sports sock
    // (grey heel and toe, navy bands), black, burgundy, navy, grey marl
    let sock_colors = [0xf0ece0, 0x181410, 0xa02040, 0x2050c0, 0xd8b040];
    let (body, heel, bands) = match choice(&sock_colors) {
        0xf0ece0 => (0xb6b1a4, 0x86827a, Some(0x2e3550)),
        0x181410 => (0x2b2927, 0x34312d, None),
        0xa02040 => (0x5a2c30, 0x4a2528, None),
        0x2050c0 => (0x2e3446, 0x262b38, None),
        _ => (0x878480, 0x6a6763, Some(0x4a4844)),
    };
    let knit = sock_knit_tex();
    let sock = group();
    let mut part = |name: &str, color: u32| {
        if let Some(geo) = prop_part_as_modelled(name) {
            sock.add(&mesh(geo, lambert!({ map: &knit, color: color })));
        }
    };
    part("Real__sock", body);
    part("Real__sockHeelToe", heel);
    if let Some(b) = bands {
        part("Real__sockBands", b);
    }
    scatter(g, &sock, x, z, 2.4);
    g.world.add(&sock);
}

/// The sock's cotton as it lies (laid over it from above: u along x, v
/// against z of `r08.py`'s sock): the knit's fine grain, washed-out patches,
/// and the dirt of a floor, heaviest at its toe (u 0.95, v 0.83) and heel
/// (u 0.3, v 0.7) where it was walked in.
fn sock_knit_tex() -> Tex {
    canvas_tex_opt(256, 256, false, |ctx, w, h| {
        ctx.set_fill_style("#e9e6df");
        ctx.fill_rect(0, 0, w, h);
        // the knit: fine wales, a fleck of fibre
        for i in 0..(w as i32) {
            if i % 3 == 0 {
                ctx.set_fill_style("rgba(120,116,108,0.10)");
                ctx.fill_rect(i as f64, 0, 1, h);
            }
        }
        for _ in 0..1600 {
            let v = 150.0 + rnd() * 100.0;
            ctx.set_fill_style(format!("rgba({v:.0},{v:.0},{:.0},0.22)", v - 6.0));
            ctx.fill_rect(rnd() * w, rnd() * h, 1.0, 1.0 + rnd() * 2.0);
        }
        // greyed all over, in washes
        for _ in 0..40 {
            let at = (rnd() * w, rnd() * h);
            r08_soft(ctx, at, 18.0 + rnd() * 40.0, 0.6 + rnd() * 0.4, rnd() * 3.0, (110, 102, 90, 0.08 + rnd() * 0.12));
        }
        // walked in: the toe and the heel
        for (u, v, n) in [(0.95, 0.83, 40), (0.30, 0.70, 30)] {
            for _ in 0..n {
                let at = (u * w + (rnd() - 0.5) * 70.0, v * h + (rnd() - 0.5) * 60.0);
                r08_soft(ctx, at, 8.0 + rnd() * 16.0, 0.6 + rnd() * 0.4, rnd() * 3.0, (74, 64, 50, 0.10 + rnd() * 0.16));
            }
        }
        // a few bits of the floor caught in it
        for _ in 0..60 {
            ctx.set_fill_style(format!("rgba(60,52,40,{:.2})", 0.3 + rnd() * 0.4));
            ctx.fill_rect(rnd() * w, rnd() * h, 1.0 + rnd() * 1.5, 1.0 + rnd() * 1.5);
        }
    })
}
