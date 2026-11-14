//! `place_spray_bottles`
use game::prelude::*;
use placers_kit::geometry::put;
use placers_kit::noise::fbm;
use placers_kit::noise::hash;
use placers_kit::noise::sstep;
use placers_kit::paint::fill as t04_fill;
use placers_kit::place::open_tiles;
use placers_kit::place::scatter_xz_turned;
use placers_kit::place::vary as t04_vary;

// Spray bottle: 8% chance per level. A 16 oz HDPE cylinder round of
// cleaner with a 28/400 trigger sprayer (Blender batch `t04`): the natural
// plastic gone yellow, the liquid left in it seen through it (the old four:
// a green all-purpose cleaner, a blue glass cleaner, a yellow degreaser, a
// red bathroom cleaner), its printed label (an invented janitorial supplier)
// scuffed and run with drips, grime where hands held it. Standing, or (the
// old 30%) knocked over on its side, the liquid lying along it.
pub(crate) fn place_spray_bottles(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = choice(&open);
    let liquid_colors = [0x30d040, 0x2050d8, 0xf0d020, 0xd82040];
    let liquid_color = choice(&liquid_colors);
    let kind = liquid_colors.iter().position(|&c| c == liquid_color).unwrap_or(0);
    let bottle = group();
    // 30% chance tipped.
    let tipped = rnd() < 0.30;
    if tipped {
        // on its side, its sprayer's nozzle along the floor
        bottle.rotation().z = PI / 2.0;
        bottle.position().y = SB_R;
    }
    scatter_xz_turned(g, &bottle, x, z, 2.0);
    let p = bottle.pos();
    let vary = t04_vary(p.x, p.z, kind as f64);
    put(&bottle, "Real__sprayBottleBody", &lambert!({ map: &sb_body_tex(vary, kind, tipped) }));
    put(&bottle, "Real__sprayBottleHead", &lambert!({ map: &sb_head_tex(vary) }));
    put(&bottle, "Real__sprayBottleNozzle", &lambert!({ color: 0x2a2928 }));
    g.world.add(&bottle);
}

/// The bottle's radius, its height to the neck's top (`bathroom/washroom_fittings.py` SB_*)
const SB_R: f64 = 0.0309;

/// What each of the old liquids is: its name on the label, its colour seen
/// through the bottle (sRGB), its label's band
const SB_KINDS: [(&str, [f64; 3], &str); 4] = [
    ("ALL-PURPOSE CLEANER", [0.36, 0.52, 0.30], "#3f6a3a"),
    ("GLASS CLEANER", [0.30, 0.44, 0.62], "#33557e"),
    ("DEGREASER", [0.72, 0.58, 0.24], "#a8812a"),
    ("BATHROOM CLEANER", [0.66, 0.30, 0.34], "#8a3440"),
];

/// The sprayer's white polypropylene gone cream, grimy where fingers were
fn sb_head_tex(vary: u32) -> Tex {
    canvas_tex_opt(64, 64, false, move |ctx, w, h| {
        let seed = vary ^ 0x5eed;
        t04_fill(ctx, w, h, |u, v| {
            let n = fbm(seed, u * 6.0, v * 6.0, 3);
            let grime = sstep(0.5, 0.8, n) * 0.25;
            let c = [0.80 - grime, 0.78 - grime, 0.71 - grime * 1.1];
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// The bottle round about (u from its front round by its right, v up):
/// natural HDPE gone yellow, milky, the liquid left in it seen darker
/// through it (to a level standing, along its lower side lying), its
/// label printed and scuffed, a drip run down from the neck, grime where it
/// was held and dirt round its foot
fn sb_body_tex(vary: u32, kind: usize, tipped: bool) -> Tex {
    canvas_tex_opt(512, 512, false, move |ctx, w, h| {
        let seed = vary ^ 0x5b07;
        let (name, liq, band) = SB_KINDS[kind.min(3)];
        // how much is left: a quarter to two thirds
        let fill = 0.25 + 0.42 * hash(seed, 1, 1);
        // standing: its level up the body; lying: how far round it reaches
        let level = 0.02 + fill * 0.74;
        let reach = (1.0 - 2.0 * fill).clamp(-0.99, 0.99).acos() / (PI * 2.0);
        t04_fill(ctx, w, h, |u, v| {
            let n = fbm(seed ^ 0x3, u * 9.0, v * 9.0, 3);
            let plastic = [0.80 + 0.03 * n, 0.78 + 0.03 * n, 0.70 + 0.02 * n];
            let wet = if tipped {
                // the lower side lies at u 0.75 (the bottle's -x down)
                let d = ((u - 0.75 + 0.5).rem_euclid(1.0) - 0.5).abs();
                1.0 - sstep(reach - 0.006, reach + 0.006, d)
            } else {
                1.0 - sstep(level - 0.004, level + 0.004, v)
            };
            let mut c = plastic;
            for k in 0..3 {
                let seen = liq[k] * 0.62 + plastic[k] * 0.38;
                c[k] = plastic[k] + (seen - plastic[k]) * wet * 0.85;
            }
            // the meniscus, a darker line
            if !tipped {
                let m = (-((v - level) / 0.004).powi(2)).exp() * 0.25;
                for k in &mut c {
                    *k *= 1.0 - m;
                }
            }
            // grime where it was held (round its middle, the back most) and
            // dirt round its foot
            let held = (-((v - 0.45) / 0.18).powi(2)).exp() * sstep(0.45, 0.8, fbm(seed ^ 0x7, u * 14.0, v * 10.0, 3)) * 0.22;
            let foot = (1.0 - sstep(0.0, 0.08, v)) * 0.35;
            for k in &mut c {
                *k *= 1.0 - held - foot * (0.6 + 0.4 * n);
            }
            [c[0], c[1], c[2], 1.0]
        });
        // the label: 76 mm round its front, from 26 to 140 mm up; drawn
        // twice, either side of the seam at its front (u 0)
        let (lw, top, bot) = (0.39 * w, (1.0 - 0.70) * h, (1.0 - 0.13) * h);
        for cx in [0.0, w] {
            ctx.save();
            ctx.translate(cx, 0.0);
            ctx.set_fill_style("#ddd8c9");
            ctx.fill_rect(-lw / 2.0, top, lw, bot - top);
            ctx.set_fill_style(band);
            ctx.fill_rect(-lw / 2.0, top, lw, 46.0);
            ctx.set_fill_style("#efeadc");
            ctx.set_text_align("center");
            ctx.set_text_baseline("alphabetic");
            ctx.set_font("bold 17px Helvetica Neue");
            ctx.fill_text_max("HALDEN", 0.0, top + 22.0, lw - 12.0);
            ctx.set_font("10px Helvetica Neue");
            ctx.fill_text_max("JANITORIAL SUPPLY", 0.0, top + 38.0, lw - 12.0);
            ctx.set_fill_style("#1d1c1a");
            ctx.set_font("bold 23px Helvetica Neue");
            let words: Vec<&str> = name.split(' ').collect();
            let mut y = top + 82.0;
            for wd in &words {
                ctx.fill_text_max(wd, 0.0, y, lw - 14.0);
                y += 25.0;
            }
            ctx.set_font("11px Helvetica Neue");
            ctx.fill_text_max("Ready to use. Spray on, wipe off.", 0.0, y + 6.0, lw - 14.0);
            // the hazard pictogram (GHS07) and its word
            let (px, py, ps) = (-lw / 2.0 + 34.0, bot - 64.0, 19.0);
            ctx.set_fill_style("#e8e3d4");
            ctx.set_stroke_style("#9c2a22");
            ctx.set_line_width(3.0);
            ctx.begin_path();
            ctx.move_to(px, py - ps);
            ctx.line_to(px + ps, py);
            ctx.line_to(px, py + ps);
            ctx.line_to(px - ps, py);
            ctx.close_path();
            ctx.fill();
            ctx.stroke();
            ctx.set_fill_style("#1d1c1a");
            ctx.set_font("bold 22px Helvetica Neue");
            ctx.fill_text("!", px, py + 8.0);
            ctx.set_text_align("left");
            ctx.set_font("bold 12px Helvetica Neue");
            ctx.fill_text("WARNING", px + 26.0, py - 4.0);
            ctx.set_font("9px Helvetica Neue");
            ctx.fill_text_max("Causes eye irritation.", px + 26.0, py + 8.0, lw - 70.0);
            ctx.fill_text_max("Keep out of reach of children.", px + 26.0, py + 19.0, lw - 70.0);
            ctx.set_text_align("center");
            ctx.set_font("bold 11px Helvetica Neue");
            ctx.fill_text_max("16 FL OZ (473 mL)", 0.0, bot - 14.0, lw - 14.0);
            ctx.restore();
        }
        // worn: the label scuffed to the plastic at its edges and where hands
        // held it, a drip run from the neck down its front, dirt
        let mut r = SplitMix64::new(vary, 0x5b08);
        for _ in 0..140 {
            let side = r.next() < 0.5;
            let x = if side { r.range(0.0, lw / 2.0) } else { w - r.range(0.0, lw / 2.0) };
            let edge = (x.min(w - x) - lw / 2.0).abs() < 14.0 || r.next() < 0.3;
            if !edge {
                continue;
            }
            let y = r.range(top, bot);
            ctx.set_fill_style(format!("rgba(198,192,170,{:.2})", r.range(0.3, 0.8)));
            ctx.begin_path();
            ctx.ellipse(x, y, r.range(1.0, 5.0), r.range(0.6, 2.5), r.range(0.0, 3.0), 0, 7);
            ctx.fill();
        }
        let drip_x = if r.next() < 0.5 { r.range(4.0, lw / 2.0 - 10.0) } else { w - r.range(4.0, lw / 2.0 - 10.0) };
        let len = r.range(0.25, 0.6) * h;
        let gr = ctx.create_linear_gradient(0, 0, 0, len);
        gr.add_color_stop(0, &format!("rgba({},{},{},0.30)", (liq[0] * 180.0) as u8, (liq[1] * 180.0) as u8, (liq[2] * 180.0) as u8));
        gr.add_color_stop(1, "rgba(120,110,90,0)");
        ctx.set_fill_style(&gr);
        ctx.fill_rect(drip_x - 3.0, 0.0, 6.0, len);
        ctx.set_fill_style("rgba(90,80,62,0.35)");
        ctx.begin_path();
        ctx.ellipse(drip_x, len, 4.0, 5.0, 0.0, 0, 7);
        ctx.fill();
    })
}
