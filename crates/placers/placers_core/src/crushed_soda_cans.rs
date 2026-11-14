//! `place_crushed_soda_cans`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::scatter_y;

// Crushed soda can: 10% chance per level: a 355 ml aluminium can
// crushed underfoot (Blender batch r08): stamped flat on its end, its printed
// wall folded into rings and the lid with its tab slid off centre, or stepped
// on lying down, pressed flat between its ends; its label a plain one of no
// real brand, faded, scratched and cracked along its folds to the metal.
pub(crate) fn place_crushed_soda_cans(g: &mut Game, _dist: &[i32]) {
    use game::assets::props::prop_part_as_modelled;
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = choice(&open);
    // the old colours stand for five plain labels (cola, club soda,
    // lemon-lime, orange, sparkling water)
    let can_colors = [0xa02040, 0x2050c0, 0x30a030, 0xd8b040, 0xc0c0c0];
    let col = choice(&can_colors);
    let k = can_colors.iter().position(|&c| c == col).unwrap_or(0);
    let can = group();
    scatter_y(g, &can, x, z, 2.4, 0.0);
    // how it was crushed, from where it lies (no new draw)
    let p = can.pos();
    let flat = ((p.x * 7.31 + p.z * 3.17).abs().fract()) < 0.5;
    let label = phong!({ map: &crushed_can_label(k, flat), color: 0xffffff, specular: 0x4a4a4a, shininess: 40 });
    let alu = phong!({ color: 0x8e9092, specular: 0x6a6a6a, shininess: 52 });
    let parts = if flat { ["Real__canFlatBody", "Real__canFlatEnds"] } else { ["Real__canCrushedBody", "Real__canCrushedEnds"] };
    for (name, mat) in parts.iter().zip([&label, &alu]) {
        if let Some(geo) = prop_part_as_modelled(name) {
            can.add(&mesh(geo, mat));
        }
    }
    g.world.add(&can);
}

/// A soft round spot at `at`: `r` px, `k` times as tall as it is wide, turned
/// `turn`, its colour fading out from the middle (the context squashed, so
/// the fade is never cut by an edge).
fn r08_soft(ctx: &mut Canvas, at: (f64, f64), r: f64, k: f64, turn: f64, rgba: (u8, u8, u8, f64)) {
    let (cr, cg, cb, a) = rgba;
    ctx.save();
    ctx.translate(at.0, at.1);
    ctx.rotate(turn);
    ctx.scale(1, k);
    let gr = ctx.create_radial_gradient(0, 0, 0, 0, 0, r);
    gr.add_color_stop(0, &format!("rgba({cr},{cg},{cb},{a:.3})"));
    gr.add_color_stop(1, &format!("rgba({cr},{cg},{cb},0)"));
    ctx.set_fill_style(&gr);
    ctx.begin_path();
    ctx.arc(0, 0, r, 0, 7);
    ctx.fill();
    ctx.restore();
}

/// The crushed can's label, unrolled round its wall: the name of what was in
/// it (no real brand), its band, 12 FL OZ (355 mL), a nutrition panel and a
/// barcode; faded, scuffed, scratched to the aluminium and cracked bare
/// along its folds. Stamped on its end, the whole label round it (u round
/// it, v up the 93 mm of its wall, folded into the crushed part's height);
/// pressed flat, the half of it on top (u along it, the lid at u = 1; v
/// across, its folded edges at v = 0 and 1).
fn crushed_can_label(k: usize, flat: bool) -> Tex {
    // ground, band, ink (sRGB), the name, its face
    const LABELS: [(u32, u32, u32, &str, &str); 5] = [
        (0x8a2428, 0xe2dccc, 0xf0ebe0, "COLA", "bold italic 92px Futura"),
        (0x24365e, 0xc8ccd0, 0xeeeeea, "CLUB SODA", "bold 64px Gill Sans"),
        (0x2c5a34, 0xd8cc6a, 0xf2efe2, "LEMON-LIME", "bold 58px Futura"),
        (0xc0782a, 0xf0e2c0, 0xfaf4e6, "ORANGE", "bold 80px Rockwell"),
        (0xb4b6b6, 0x3a5a86, 0x2c4a74, "SPARKLING WATER", "bold 44px Avenir Next Condensed"),
    ];
    // the crush, as modelled (r08.py `CrushedCan`, `CAN_PH`)
    const PH: [f64; 5] = [1.3, 4.1, 0.6, 2.9, 5.2];
    let (ground, band, ink, name, face) = LABELS[k];
    let (cw, ch) = if flat { (256, 256) } else { (512, 256) };
    canvas_tex(cw, ch, move |ctx, cw, ch| {
        // the label is laid out unrolled, 512 round by 256 up; pressed flat
        // its first half lies along the canvas (its top at the canvas' right)
        let (w, h) = (512.0, 256.0);
        if flat {
            ctx.set_transform(0, 1, -1, 0, cw, 0);
        }
        let hex = |c: u32| format!("#{c:06x}");
        ctx.set_fill_style(hex(ground).as_str());
        ctx.fill_rect(0, 0, w, h);
        // a band round it, waving
        ctx.set_fill_style(hex(band).as_str());
        ctx.begin_path();
        ctx.move_to(0, h * 0.70);
        for i in 0..=32 {
            let x = i as f64 / 32.0 * w;
            ctx.line_to(x, h * 0.70 + (x / w * PI * 4.0).sin() * 10.0);
        }
        ctx.line_to(w, h * 0.82);
        for i in (0..=32).rev() {
            let x = i as f64 / 32.0 * w;
            ctx.line_to(x, h * 0.82 + (x / w * PI * 4.0 + 0.8).sin() * 8.0);
        }
        ctx.close_path();
        ctx.fill();
        // the name, twice round, along the can
        ctx.set_fill_style(hex(ink).as_str());
        ctx.set_text_align("center");
        ctx.set_text_baseline("middle");
        ctx.set_font(face);
        for c in [0.25, 0.75] {
            ctx.fill_text_max(name, w * c, h * 0.42, w * 0.44);
        }
        ctx.set_font("bold 13px Helvetica");
        ctx.fill_text("12 FL OZ (355 mL)", w * 0.25, h * 0.92);
        ctx.fill_text("12 FL OZ (355 mL)", w * 0.75, h * 0.92);
        // the nutrition panel and the barcode, on the back
        let (px, py) = (w * 0.47, h * 0.12);
        ctx.set_fill_style("rgba(236,232,222,0.92)");
        ctx.fill_rect(px, py, 44, 74);
        ctx.set_fill_style("rgba(30,28,26,0.9)");
        ctx.set_font("bold 7px Helvetica");
        ctx.set_text_align("left");
        ctx.fill_text("Nutrition Facts", px + 3.0, py + 6.0);
        ctx.set_font("5px Helvetica");
        for (i, l) in ["Serving size 1 can", "Calories 140", "Total Fat 0g", "Sodium 45mg", "Total Carb. 39g", "Sugars 39g", "Protein 0g"]
            .iter()
            .enumerate()
        {
            ctx.fill_text(l, px + 3.0, py + 15.0 + i as f64 * 7.5);
        }
        ctx.set_fill_style("rgba(236,232,222,0.92)");
        ctx.fill_rect(w * 0.025, h * 0.14, 26, 40);
        ctx.set_fill_style("rgba(20,20,20,0.9)");
        let mut bx = w * 0.025 + 3.0;
        while bx < w * 0.025 + 23.0 {
            let bw = 0.6 + rnd() * 1.6;
            ctx.fill_rect(bx, h * 0.14 + 3.0, bw, 30);
            bx += bw + 0.6 + rnd() * 1.2;
        }
        ctx.set_font("5px Helvetica");
        ctx.fill_text("PLEASE RECYCLE", w * 0.025, h * 0.14 + 48.0);
        // faded: years under the light and the dust
        ctx.set_fill_style("rgba(196,192,182,0.16)");
        ctx.fill_rect(0, 0, w, h);
        for _ in 0..40 {
            let at = (rnd() * w, rnd() * h);
            r08_soft(ctx, at, 14.0 + rnd() * 40.0, 0.5 + rnd() * 0.3, rnd() * 3.0, (110, 100, 84, 0.08 + rnd() * 0.16));
        }
        ctx.reset_transform();
        let (w, h) = (cw, ch);
        if flat {
            // cracked bare along its two folded edges and where it creased
            // going flat by its ends
            ctx.set_stroke_style("rgba(192,194,196,0.7)");
            ctx.set_line_width(2.0);
            for y in [3.0, h - 3.0] {
                ctx.begin_path();
                ctx.move_to(w * 0.18, y + (rnd() - 0.5) * 3.0);
                for i in 1..=12 {
                    ctx.line_to(w * (0.18 + 0.64 * i as f64 / 12.0), y + (rnd() - 0.5) * 3.0);
                }
                ctx.stroke();
            }
            ctx.set_line_width(1.4);
            for _ in 0..10 {
                let x = if rnd() < 0.5 { w * (0.08 + rnd() * 0.2) } else { w * (0.72 + rnd() * 0.2) };
                let y = rnd() * h;
                ctx.begin_path();
                ctx.move_to(x, y);
                ctx.line_to(x + (rnd() - 0.5) * 30.0, y + 20.0 + rnd() * 50.0);
                ctx.stroke();
            }
        }
        // cracked bare where the folds' crests bent it (3 folds, out of step round it)
        ctx.set_line_width(2.2);
        for f in (1..=3).filter(|_| !flat) {
            ctx.set_stroke_style("rgba(192,194,196,0.7)");
            ctx.begin_path();
            for i in 0..=96 {
                let u = i as f64 / 96.0;
                let a = u * PI * 2.0;
                let lag = 0.18 * (a + PH[0]).sin() + 0.08 * (2.0 * a + PH[1]).sin();
                let t = ((f as f64 - lag) / 3.0).clamp(0.0, 1.0);
                let s = ((t - 0.3) / 0.7).clamp(0.0, 1.0);
                let tilt = 0.157 * s * s * (3.0 - 2.0 * s);
                let y = 0.011 * t - 0.033 * a.cos() * tilt.sin() + 0.0012 * (3.0 * a + PH[3]).sin() * (PI * t).sin();
                let v = (y / 0.0162).clamp(0.0, 1.0);
                let (x, yy) = (u * w, (1.0 - v) * h + (rnd() - 0.5) * 2.0);
                if i == 0 {
                    ctx.move_to(x, yy);
                } else {
                    ctx.line_to(x, yy);
                }
            }
            ctx.stroke();
        }
        // scratched to the aluminium, scraped along the floor
        ctx.set_line_width(1.0);
        for _ in 0..110 {
            let (x, y) = (rnd() * w, rnd() * h);
            let a = (rnd() - 0.5) * 0.6;
            let l = 4.0 + rnd() * 50.0;
            ctx.set_stroke_style(format!("rgba(196,198,200,{:.2})", 0.25 + rnd() * 0.5));
            ctx.begin_path();
            ctx.move_to(x, y);
            ctx.line_to(x + a.cos() * l, y + a.sin() * l);
            ctx.stroke();
        }
    })
}
