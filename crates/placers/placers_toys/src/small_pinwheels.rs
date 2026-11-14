//! `place_small_pinwheels`: Small pinwheel: 10% chance per level. A paper party pinwheel (Blender batch r17): a 10 cm square of printed paper cut and curled into four vanes, pinned through a bead into a 28 cm birch dowel, dropped and lying face up on the floor (never standing on its stick's point): the dowel on the floor, the wheel over its end, the curls standing up, the paper faded, water-marked and grimed. Three in ten stir on the pin in a draught.
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

/// Small pinwheel: 10% chance per level. A paper party pinwheel
/// (Blender batch r17): a 10 cm square of printed paper cut and curled into
/// four vanes, pinned through a bead into a 28 cm birch dowel, dropped and
/// lying face up on the floor (never standing on its stick's point): the
/// dowel on the floor, the wheel over its end, the curls standing up, the
/// paper faded, water-marked and grimed. Three in ten stir on the pin in a
/// draught.
pub(crate) fn place_small_pinwheels(g: &mut Game, _dist: &[i32]) {
    use game::assets::props::prop_part_as_modelled;
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    // the four vanes' papers: the old red, blue, yellow and green, faded
    let papers: [u32; 4] = [0xa4473a, 0x3e5a88, 0xbf9c44, 0x4b7449];
    let pinwheel = group();
    if let Some(geo) = prop_part_as_modelled("Real__pinwheelStick") {
        pinwheel.add(&mesh(geo, lambert!({ color: 0xffffff, map: &pinwheel_dowel_tex() })));
    }
    // the wheel turns on the pin about the hub (the parts' origin)
    let wheel = group();
    if let Some(geo) = prop_part_as_modelled("Real__pinwheelVane") {
        for (k, &paper) in papers.iter().enumerate() {
            let vane = mesh(geo.clone(), lambert!({ color: 0xffffff, map: &pinwheel_paper_tex(paper), side: DoubleSide }));
            vane.rotation().y = k as f64 * PI / 2.0;
            wheel.add(&vane);
        }
    }
    if let Some(geo) = prop_part_as_modelled("Real__pinwheelPin") {
        wheel.add(&mesh(geo, phong!({ color: 0xcfc8b8, specular: 0x262626, shininess: 40 })));
    }
    pinwheel.add(&wheel);
    pinwheel.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    pinwheel.rotation().y = rnd() * PI * 2.0;
    // (the old lean, within ±0.075): how far the wheel has turned on its pin
    // from the dowel, within ±20 degrees of the dowel lying under a side's
    // middle (where the sheet clears it; a drooping corner would not)
    let lean = (rnd() - 0.5) * 0.15;
    wheel.rotation().y = lean / 0.075 * 0.35;
    g.world.add(&pinwheel);
    // 30% chance: a draught along the floor stirs it on its pin, slowly
    // (a wheel lying on its corners cannot turn round), period 4-8 s.
    if rnd() < 0.30 {
        let spin_period = 4.0 + rnd() * 4.0;
        let spin_dir = if rnd() < 0.5 { -1.0 } else { 1.0 };
        let wg = wheel.clone();
        let base = wheel.rotation().y;
        g.animate(move |_g, _dt, t| {
            wg.rotation().y = base + spin_dir * 0.07 * (t * PI * 2.0 / spin_period).sin();
        });
    }
}

/// A vane's paper (sRGB `paper`): printed one side with rows of small white
/// dots, faded toward white unevenly, a tide line where it got wet and
/// dried, grey dirt from the floor, a crease, rubbed where it was handled
fn pinwheel_paper_tex(paper: u32) -> Tex {
    canvas_tex(128, 128, move |ctx, w, h| {
        let (r, g, b) = ((paper >> 16) & 255, (paper >> 8) & 255, paper & 255);
        ctx.set_fill_style(format!("rgb({r},{g},{b})"));
        ctx.fill_rect(0, 0, w, h);
        // the print: small dots in offset rows, not quite white (the ink was
        // cheap), some rubbed off
        for row in 0..9 {
            for col in 0..9 {
                let x = (col as f64 + if row % 2 == 0 { 0.25 } else { 0.75 }) * w / 9.0;
                let y = (row as f64 + 0.5) * h / 9.0;
                let a = 0.55 + rnd() * 0.25;
                if rnd() < 0.12 {
                    continue;
                }
                ctx.set_fill_style(format!("rgba(226,220,204,{a:.2})"));
                ctx.begin_path();
                ctx.arc(x, y, 2.6, 0, 7);
                ctx.fill();
            }
        }
        // faded unevenly toward the paper's own white
        for _ in 0..14 {
            let (x, y) = (rnd() * w, rnd() * h);
            let rr = 14.0 + rnd() * 30.0;
            let gr = ctx.create_radial_gradient(x, y, 0, x, y, rr);
            gr.add_color_stop(0, "rgba(232,226,210,0.28)");
            gr.add_color_stop(1, "rgba(232,226,210,0)");
            ctx.set_fill_style(gr);
            ctx.fill_rect(x - rr, y - rr, rr * 2.0, rr * 2.0);
        }
        // a tide line: it got wet and dried, its edge darker
        let (tx, ty, tr) = (rnd() * w, rnd() * h, 26.0 + rnd() * 22.0);
        ctx.set_stroke_style("rgba(92,76,52,0.30)");
        ctx.set_line_width(2.2);
        ctx.begin_path();
        ctx.arc(tx, ty, tr, 0, 7);
        ctx.stroke();
        ctx.set_fill_style("rgba(150,132,96,0.10)");
        ctx.begin_path();
        ctx.arc(tx, ty, tr, 0, 7);
        ctx.fill();
        // floor dirt: grey patches and specks
        for _ in 0..10 {
            let (x, y) = (rnd() * w, rnd() * h);
            let rr = 6.0 + rnd() * 16.0;
            let gr = ctx.create_radial_gradient(x, y, 0, x, y, rr);
            gr.add_color_stop(0, "rgba(70,64,56,0.22)");
            gr.add_color_stop(1, "rgba(70,64,56,0)");
            ctx.set_fill_style(gr);
            ctx.fill_rect(x - rr, y - rr, rr * 2.0, rr * 2.0);
        }
        for _ in 0..60 {
            ctx.set_fill_style(format!("rgba(60,54,46,{:.2})", 0.2 + rnd() * 0.35));
            ctx.fill_rect(rnd() * w, rnd() * h, 1.0 + rnd(), 1.0 + rnd());
        }
        // a crease across it, light on one side, dark on the other
        let (x0, y0) = (0.0, rnd() * h);
        let (x1, y1) = (w, rnd() * h);
        ctx.set_line_width(1.0);
        ctx.set_stroke_style("rgba(240,236,226,0.35)");
        ctx.begin_path();
        ctx.move_to(x0, y0);
        ctx.line_to(x1, y1);
        ctx.stroke();
        ctx.set_stroke_style("rgba(40,34,28,0.30)");
        ctx.begin_path();
        ctx.move_to(x0, y0 + 1.2);
        ctx.line_to(x1, y1 + 1.2);
        ctx.stroke();
    })
}

/// The birch dowel: pale wood gone grey-brown with handling and the floor,
/// its fibres in fine streaks, darker specks
fn pinwheel_dowel_tex() -> Tex {
    canvas_tex(64, 64, |ctx, w, h| {
        ctx.set_fill_style("#9a7d58");
        ctx.fill_rect(0, 0, w, h);
        for _ in 0..90 {
            let (x, y) = (rnd() * w, rnd() * h);
            let light = rnd() < 0.5;
            ctx.set_fill_style(if light { "rgba(196,170,128,0.30)" } else { "rgba(74,58,40,0.28)" });
            if rnd() < 0.5 {
                ctx.fill_rect(x, y, 4.0 + rnd() * 10.0, 1.0);
            } else {
                ctx.fill_rect(x, y, 1.0, 4.0 + rnd() * 10.0);
            }
        }
        for _ in 0..30 {
            ctx.set_fill_style("rgba(50,42,34,0.35)");
            ctx.fill_rect(rnd() * w, rnd() * h, 1, 1);
        }
    })
}
