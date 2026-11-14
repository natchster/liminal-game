//! `place_tape_rolls`
use crate::shared::salt;
use crate::shared::soft;
use game::prelude::*;
use placers_kit::paint::rgba;
use placers_kit::place::open_tiles;
use placers_kit::place::scatter;

// Tape roll: 10% chance per level. A roll of PVC insulating tape,
// half used (Blender family `paper_and_small_things_on_office_floors`: 19 mm wide on a 1.5 in paperboard core,
// 60 mm across), lying on its face, its loose end peeled up and curled;
// lint and grit stuck to its edges. The old draws are kept call for call.
pub(crate) fn place_tape_rolls(g: &mut Game, _dist: &[i32]) {
    use game::assets::props::prop_part_as_modelled;
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = choice(&open);
    // the old colour draw: white, red, yellow or blue tape
    let tape_colors = [0xf0ece0, 0xa02040, 0xd8b040, 0x2050c0];
    let col = choice(&tape_colors);
    let look = tape_colors.iter().position(|&c| c == col).unwrap_or(0);
    let tape = group();
    scatter(g, &tape, x, z, 2.4);
    let salt = salt(&tape);
    let pvc = PVC[look];
    let vinyl = |map: Tex| phong!({ map: &map, color: 0xffffff, specular: 0x1c1c1c, shininess: 34 });
    let parts = [
        ("Real__tapeRollFaces", vinyl(tape_faces_tex(look, salt))),
        ("Real__tapeRollWrap", vinyl(tape_wrap_tex(look, salt))),
        ("Real__tapeRollCore", lambert!({ map: &tape_core_tex(salt) })),
        ("Real__tapeRollTail", phong!({ color: hex(pvc, 0.9), specular: 0x1c1c1c, shininess: 34 })),
    ];
    for (name, mat) in &parts {
        if let Some(geo) = prop_part_as_modelled(name) {
            tape.add(&mesh(geo, mat));
        }
    }
    g.world.add(&tape);
}

/// An sRGB colour as a hex, darkened by `k`
fn hex(c: (f64, f64, f64), k: f64) -> u32 {
    let q = |v: f64| ((v * k).round().clamp(0.0, 255.0)) as u32;
    (q(c.0) << 16) | (q(c.1) << 8) | q(c.2)
}

// ------------------------------------------------------------ the tape
/// PVC tape gone dull (sRGB): white, red, yellow, blue
const PVC: [(f64, f64, f64); 4] = [(210.0, 204.0, 190.0), (138.0, 38.0, 34.0), (186.0, 150.0, 46.0), (42.0, 66.0, 122.0)];

/// The roll's wound faces (mapped as a disc: the tape from 0.72 of the
/// radius out): the turns' fine rings, grit and lint stuck to the
/// adhesive edges, thickest at the outer turns
fn tape_faces_tex(look: usize, salt: u32) -> Tex {
    canvas_tex_opt(256, 256, false, move |ctx, w, h| {
        let mut r = SplitMix64::new(salt, 0x7a9e);
        let c = PVC[look];
        ctx.set_fill_style(rgba(c, 1.0));
        ctx.fill_rect(0, 0, w, h);
        let (cx, cy) = (w / 2.0, h / 2.0);
        ctx.set_line_width(0.7);
        let mut rr = 92.0;
        while rr < 128.0 {
            let dark = r.next() < 0.5;
            ctx.set_stroke_style(if dark {
                rgba((0.0, 0.0, 0.0), 0.05 + 0.08 * r.next())
            } else {
                rgba((255.0, 250.0, 240.0), 0.04 + 0.06 * r.next())
            });
            ctx.begin_path();
            ctx.arc(cx, cy, rr, 0, 7);
            ctx.stroke();
            rr += 0.8 + r.next() * 1.4;
        }
        // where the roll was put down and picked up: a turn or two darker
        for _ in 0..2 {
            let rr = 96.0 + r.next() * 28.0;
            ctx.set_stroke_style("rgba(30,24,18,0.22)");
            ctx.set_line_width(1.1);
            ctx.begin_path();
            ctx.arc(cx, cy, rr, 0, 7);
            ctx.stroke();
        }
        // grime on the adhesive edges: a band at the outer turns, patchy
        let gr = ctx.create_radial_gradient(cx, cy, 104, cx, cy, 128);
        gr.add_color_stop(0, "rgba(58,48,36,0)");
        gr.add_color_stop(0.6, "rgba(58,48,36,0.25)");
        gr.add_color_stop(1, "rgba(52,42,32,0.62)");
        ctx.set_fill_style(&gr);
        ctx.fill_rect(0, 0, w, h);
        for _ in 0..60 {
            let a = r.next() * PI * 2.0;
            let rr = 96.0 + r.next() * 32.0;
            soft(ctx, (cx + a.cos() * rr, cy + a.sin() * rr), 3.0 + r.next() * 9.0, 0.6, a, (60.0, 50.0, 38.0), 0.15 + 0.3 * r.next());
        }
        // lint: fibres and a hair caught on the glue
        ctx.set_line_width(0.6);
        for _ in 0..14 {
            let a = r.next() * PI * 2.0;
            let rr = 95.0 + r.next() * 30.0;
            let (x, y) = (cx + a.cos() * rr, cy + a.sin() * rr);
            let b = r.next() * PI * 2.0;
            let l = 4.0 + r.next() * 16.0;
            ctx.set_stroke_style(rgba(if r.next() < 0.5 { (70.0, 64.0, 58.0) } else { (190.0, 184.0, 172.0) }, 0.6));
            ctx.begin_path();
            ctx.move_to(x, y);
            ctx.quadratic_curve_to(x + (b + 0.6).cos() * l * 0.6, y + (b + 0.6).sin() * l * 0.6, x + b.cos() * l, y + b.sin() * l);
            ctx.stroke();
        }
        // and the floor's dust over the face it lay on its back on
        for _ in 0..200 {
            let (x, y) = (r.next() * w, r.next() * h);
            ctx.set_fill_style(rgba((96.0, 86.0, 70.0), 0.2 + 0.3 * r.next()));
            ctx.fill_rect(x, y, 1.0, 1.0);
        }
    })
}

/// The outer wrap (u round it, v up it): grime at both edges, scuffs
/// where it rolled, the floor's dust
fn tape_wrap_tex(look: usize, salt: u32) -> Tex {
    canvas_tex_opt(256, 32, false, move |ctx, w, h| {
        let mut r = SplitMix64::new(salt, 0x3c1d);
        ctx.set_fill_style(rgba(PVC[look], 1.0));
        ctx.fill_rect(0, 0, w, h);
        for (y0, y1) in [(0.0, 7.0), (h, h - 7.0)] {
            let gr = ctx.create_linear_gradient(0, y0, 0, y1);
            gr.add_color_stop(0, "rgba(52,42,32,0.55)");
            gr.add_color_stop(1, "rgba(52,42,32,0)");
            ctx.set_fill_style(&gr);
            ctx.fill_rect(0, y0.min(y1), w, 7);
        }
        for _ in 0..30 {
            let (x, y) = (r.next() * w, r.next() * h);
            ctx.set_fill_style(rgba((240.0, 236.0, 226.0), 0.08 + 0.1 * r.next()));
            ctx.fill_rect(x, y, 4.0 + r.next() * 18.0, 0.8);
        }
        for _ in 0..120 {
            let (x, y) = (r.next() * w, r.next() * h);
            ctx.set_fill_style(rgba((90.0, 80.0, 66.0), 0.2 + 0.3 * r.next()));
            ctx.fill_rect(x, y, 1.0, 1.0);
        }
        // the outer turn's cut end, a faint step across the wrap
        ctx.set_fill_style("rgba(20,16,12,0.25)");
        ctx.fill_rect(2, 0, 1.2, h);
    })
}

/// The paperboard core: grey-brown kraft, mottled, its ends dark with
/// handling
fn tape_core_tex(salt: u32) -> Tex {
    canvas_tex_opt(128, 64, false, move |ctx, w, h| {
        let mut r = SplitMix64::new(salt, 0xc02e);
        ctx.set_fill_style("#8e7656");
        ctx.fill_rect(0, 0, w, h);
        for _ in 0..90 {
            let at = (r.next() * w, r.next() * h);
            soft(
                ctx,
                at,
                3.0 + r.next() * 8.0,
                0.5,
                0.0,
                if r.next() < 0.5 { (70.0, 56.0, 40.0) } else { (170.0, 150.0, 120.0) },
                0.15 + 0.2 * r.next(),
            );
        }
        for (y0, y1) in [(0.0, 10.0), (h, h - 10.0)] {
            let gr = ctx.create_linear_gradient(0, y0, 0, y1);
            gr.add_color_stop(0, "rgba(40,32,24,0.6)");
            gr.add_color_stop(1, "rgba(40,32,24,0)");
            ctx.set_fill_style(&gr);
            ctx.fill_rect(0, y0.min(y1), w, 10);
        }
    })
}
