//! `place_bottle_caps`
use crate::shared::soft;
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::scatter;

// Bottle caps: 10% chance per level: 8-14 crown caps prised off bottles
// and dropped in one place (Blender batch r08: 29 mm across, their 21 flutes
// crimped, their printed tops faded and scratched to the tin, some bent by
// the opener, some lying on their backs showing the liner).
pub(crate) fn place_bottle_caps(g: &mut Game, _dist: &[i32]) {
    use game::assets::props::prop_part_as_modelled;
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = choice(&open);
    // the old colours stand for five printed caps (none a real brand)
    let cap_colors = [0xa02020, 0x30a030, 0x2050c0, 0xd8b040, 0xc0c0c0];
    let mut crowns: Vec<Option<Mat>> = vec![None; cap_colors.len()];
    let mut tops: Vec<Option<Mat>> = vec![None; cap_colors.len()];
    let liner = lambert!({ color: 0xb2ac9c });
    let grp = group();
    let mut down: Vec<(f64, f64)> = vec![];
    let n_caps = 8 + (rnd() * 7.0).floor() as i32;
    for i in 0..n_caps {
        let col = choice(&cap_colors);
        let k = cap_colors.iter().position(|&c| c == col).unwrap_or(0);
        let px = (rnd() - 0.5) * 0.25;
        let pz = (rnd() - 0.5) * 0.25;
        // 25% on its back (it was edge-on)
        let flipped = rnd() < 0.25;
        let ry = rnd() * PI * 2.0;
        // a third bent by the opener (from where it fell: no new draw)
        let bent = ((px + 0.5) * 53.0).fract() < 0.34;
        // clear of the caps already down
        let (mut qx, mut qz) = (px, pz);
        for _ in 0..4 {
            for &(ax, az) in &down {
                let (dx, dz) = (qx - ax, qz - az);
                let d = (dx * dx + dz * dz).sqrt();
                if d < 0.031 {
                    let (ux, uz) = if d > 1e-6 { (dx / d, dz / d) } else { (1.0, 0.0) };
                    qx += ux * (0.032 - d);
                    qz += uz * (0.032 - d);
                }
            }
        }
        down.push((qx, qz));
        let (crown_n, top_n, liner_n) = if bent {
            ("Real__capCrownBent", "Real__capTopBent", "Real__capLinerBent")
        } else {
            ("Real__capCrown", "Real__capTop", "Real__capLiner")
        };
        let Some(crown_geo) = prop_part_as_modelled(crown_n) else { continue };
        let height = crown_geo.bounding_box().max.y;
        let crown_mat = crowns[k].get_or_insert_with(|| cap_crown_mat(k)).clone();
        let cap = group();
        cap.add(&mesh(crown_geo, &crown_mat));
        if flipped {
            if let Some(geo) = prop_part_as_modelled(liner_n) {
                cap.add(&mesh(geo, &liner));
            }
            cap.rotation().x = PI;
            cap.position().set(qx, height, qz);
        } else {
            if let Some(geo) = prop_part_as_modelled(top_n) {
                let top_mat = tops[k].get_or_insert_with(|| cap_top_mat(k)).clone();
                cap.add(&mesh(geo, &top_mat));
            }
            cap.position().set(qx, 0, qz);
        }
        cap.rotation().y = ry;
        grp.add(&cap);
    }
    scatter(g, &grp, x, z, 2.4);
    g.world.add(&grp);
}

/// The five caps' lacquer (sRGB): red, bottle green, navy, gold, tin.
const CAP_PAINT: [u32; 5] = [0x7e2a24, 0x2f4b33, 0x2b3a5e, 0x9a7c3a, 0x9c9c96];

/// A cap's skirt (round it, u; up it, v): its lacquer, the flutes' crests
/// rubbed to the tin, rust along the cut edge, scratches.
fn cap_crown_mat(k: usize) -> Mat {
    let paint = CAP_PAINT[k];
    let tex = canvas_tex(126, 32, move |ctx, w, h| {
        let (r, g, b) = ((paint >> 16) & 255, (paint >> 8) & 255, paint & 255);
        ctx.set_fill_style(format!("rgb({r},{g},{b})"));
        ctx.fill_rect(0, 0, w, h);
        // 21 flutes: their crests rubbed bright, toward the edge
        for f in 0..21 {
            let x = (f as f64 + 0.5) / 21.0 * w;
            let gr = ctx.create_linear_gradient(0, 0, 0, h);
            gr.add_color_stop(0, "rgba(176,174,166,0)");
            gr.add_color_stop(1, "rgba(176,174,166,0.55)");
            ctx.set_fill_style(&gr);
            ctx.fill_rect(x - 1.0, h * 0.3, 2.0, h * 0.7);
        }
        // rust at the cut edge (it is tinplate: the edge is bare steel)
        for _ in 0..50 {
            ctx.set_fill_style(format!("rgba({:.0},{:.0},30,{:.2})", 100.0 + rnd() * 30.0, 56.0 + rnd() * 14.0, 0.35 + rnd() * 0.45));
            ctx.fill_rect(rnd() * w, h - 1.0 - rnd() * rnd() * 10.0, 1.0 + rnd() * 3.0, 1.0 + rnd() * 2.0);
        }
        ctx.set_line_width(0.7);
        for _ in 0..16 {
            let (x, y) = (rnd() * w, rnd() * h);
            ctx.set_stroke_style(format!("rgba(178,176,168,{:.2})", 0.3 + rnd() * 0.4));
            ctx.begin_path();
            ctx.move_to(x, y);
            ctx.line_to(x + (rnd() - 0.5) * 16.0, y + (rnd() - 0.5) * 6.0);
            ctx.stroke();
        }
    });
    phong!({ map: &tex, color: 0xffffff, specular: 0x484848, shininess: 46 })
}

/// A cap's printed top (seen from above: the disc fills the square): a
/// plain mark of a beer or a soda that never was, faded, rubbed, scratched
/// to the tin, a ring pressed in it by the bottle's lip.
fn cap_top_mat(k: usize) -> Mat {
    let paint = CAP_PAINT[k];
    let tex = canvas_tex_opt(128, 128, false, move |ctx, w, h| {
        let (r, g, b) = ((paint >> 16) & 255, (paint >> 8) & 255, paint & 255);
        ctx.set_fill_style(format!("rgb({r},{g},{b})"));
        ctx.fill_rect(0, 0, w, h);
        let (cx, cy) = (w / 2.0, h / 2.0);
        let ink = if k == 3 || k == 4 { "rgba(40,30,22,0.85)" } else { "rgba(226,220,204,0.85)" };
        ctx.set_fill_style(ink);
        ctx.set_stroke_style(ink);
        ctx.set_text_align("center");
        ctx.set_text_baseline("middle");
        let ring = |ctx: &mut Canvas, rr: f64, lw: f64| {
            ctx.set_line_width(lw);
            ctx.begin_path();
            ctx.arc(cx, cy, rr, 0, 7);
            ctx.stroke();
        };
        match k {
            0 => {
                ring(ctx, 50.0, 3.0);
                ctx.set_font("bold 25px Futura");
                ctx.fill_text_max("LAGER", cx, cy + 6.0, 86);
                // a small star over it
                ctx.begin_path();
                for i in 0..10 {
                    let a = -PI / 2.0 + i as f64 * PI / 5.0;
                    let rr = if i % 2 == 0 { 9.0 } else { 3.8 };
                    let (x, y) = (cx + a.cos() * rr, cy - 22.0 + a.sin() * rr);
                    if i == 0 {
                        ctx.move_to(x, y);
                    } else {
                        ctx.line_to(x, y);
                    }
                }
                ctx.close_path();
                ctx.fill();
            }
            1 => {
                ring(ctx, 52.0, 1.5);
                ring(ctx, 46.0, 1.5);
                ctx.set_font("bold 19px Gill Sans");
                ctx.fill_text_max("PILSNER", cx, cy, 84);
                ctx.set_font("10px Gill Sans");
                ctx.fill_text("BREWED SINCE 1898", cx, cy + 20.0);
            }
            2 => {
                ctx.set_font("bold 30px Futura");
                ctx.fill_text_max("SODA", cx, cy - 4.0, 90);
                ctx.set_line_width(2.5);
                ctx.begin_path();
                ctx.move_to(cx - 34.0, cy + 18.0);
                ctx.bezier_curve_to(cx - 14.0, cy + 10.0, cx + 6.0, cy + 26.0, cx + 34.0, cy + 16.0);
                ctx.stroke();
            }
            3 => {
                // a shield, EXPORT across it
                ctx.set_line_width(2.5);
                ctx.begin_path();
                ctx.move_to(cx - 26.0, cy - 30.0);
                ctx.line_to(cx + 26.0, cy - 30.0);
                ctx.line_to(cx + 26.0, cy + 2.0);
                ctx.quadratic_curve_to(cx + 24.0, cy + 26.0, cx, cy + 36.0);
                ctx.quadratic_curve_to(cx - 24.0, cy + 26.0, cx - 26.0, cy + 2.0);
                ctx.close_path();
                ctx.stroke();
                ctx.set_font("bold 15px Rockwell");
                ctx.fill_text_max("EXPORT", cx, cy - 2.0, 50);
            }
            _ => {
                ring(ctx, 50.0, 2.0);
                for i in 0..24 {
                    let a = i as f64 / 24.0 * PI * 2.0;
                    ctx.begin_path();
                    ctx.arc(cx + a.cos() * 42.0, cy + a.sin() * 42.0, 1.8, 0, 7);
                    ctx.fill();
                }
                ctx.set_font("bold 26px Copperplate");
                ctx.fill_text_max("BEER", cx, cy, 70);
            }
        }
        // faded: the print rubbed thin in patches, the lacquer worn off
        ctx.set_global_alpha(1.0);
        for _ in 0..26 {
            let at = (rnd() * w, rnd() * h);
            soft(ctx, at, 8.0 + rnd() * 16.0, 0.5 + rnd() * 0.4, rnd() * 3.0, (r as u8, g as u8, b as u8, 0.25 + rnd() * 0.45));
        }
        // scratched to the tin
        ctx.set_line_width(0.9);
        for _ in 0..22 {
            let (x, y, a) = (rnd() * w, rnd() * h, rnd() * PI);
            let l = 6.0 + rnd() * 40.0;
            ctx.set_stroke_style(format!("rgba(186,184,176,{:.2})", 0.35 + rnd() * 0.45));
            ctx.begin_path();
            ctx.move_to(x, y);
            ctx.line_to(x + a.cos() * l, y + a.sin() * l);
            ctx.stroke();
        }
        // the ring the bottle's lip pressed into it, a little bright
        ctx.set_stroke_style("rgba(190,186,176,0.25)");
        ctx.set_line_width(2.0);
        ctx.begin_path();
        ctx.arc(cx, cy, 56.0, 0, 7);
        ctx.stroke();
        // dirt
        for _ in 0..30 {
            ctx.set_fill_style(format!("rgba(60,52,40,{:.2})", 0.1 + rnd() * 0.25));
            ctx.fill_rect(rnd() * w, rnd() * h, 1.0 + rnd() * 3.0, 1.0 + rnd() * 3.0);
        }
    });
    phong!({ map: &tex, color: 0xffffff, specular: 0x404040, shininess: 44 })
}
