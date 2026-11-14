//! `place_rustic_signs`
use game::prelude::*;
use placers_kit::place::face_wall;
use placers_kit::place::pick;
use placers_kit::place::vary as rs_vary;
use placers_kit::place::wall_tiles;

// Rustic wooden wall sign: 4% chance per level. Three pine boards
// nailed to two battens, its name burned in, hung by a cord (batch r13a).
pub(crate) fn place_rustic_signs(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let walls = wall_tiles(g);
    if walls.is_empty() {
        return;
    }
    let (x, z, dx, dz) = pick(&walls);
    // A hand-made cabin sign (Blender batch `r13a`): three pine boards a
    // foot long nailed to two battens, a name burned into them with a
    // poker, gone grey and dry; hung by a jute cord from two screw eyes over
    // a nail, a little askew (the old tilt's draw).
    // Nothing on it says anything about the place (docs/design/PROPS.md)
    use game::assets::props::prop_part_as_modelled;
    let sign = group();
    let mount_y = 1.75;
    let tilt = (rnd() - 0.5) * 0.06;
    sign.rotation().y = face_wall(dx, dz);
    sign.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - 0.013), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - 0.013));
    let p = sign.pos();
    let vary = rs_vary(p.x, p.z, tilt);
    // askew as the old one hung (turned about the same point, so it hangs
    // where it did); the parts are modelled with the wall's face at z = 0:
    // 13 mm behind the spot, the boards' middle at the old panel's height
    sign.rotation().z = tilt;
    let body = group();
    body.position().set(0, mount_y, -0.013);
    sign.add(&body);
    let wood = lambert!({ map: &rustic_board_tex(vary) });
    let steel = phong!({ color: 0x2c2a28, specular: 0x262422, shininess: 20 });
    let parts: [(&str, Mat); 6] = [
        ("Real__rusticSignBoards", wood),
        ("Real__rusticSignBattens", lambert!({ color: 0x6a5a48 })),
        ("Real__rusticSignNails", steel.clone()),
        ("Real__rusticSignEyes", phong!({ color: 0x3a3630, specular: 0x302c28, shininess: 30 })),
        ("Real__rusticSignCord", lambert!({ color: 0x7c6a4c })),
        ("Real__rusticSignNail", steel),
    ];
    for (name, mat) in parts {
        if let Some(geo) = prop_part_as_modelled(name) {
            body.add(&mesh(geo, &mat));
        }
    }
    g.world.add(&sign);
}

/// What is burned into it: a family's name, a cabin's, a welcome
const RS_WORDS: [(&str, &str); 6] = [
    ("THE HENDERSONS", "EST. 1979"),
    ("WELCOME", "TO THE CABIN"),
    ("GONE FISHING", ""),
    ("BEAR CREEK", "CABIN"),
    ("THE OLSENS", "1984"),
    ("HOME SWEET HOME", ""),
];

/// The three boards' faces as the part's front is mapped (its bounds:
/// 0.308 x 0.146 m): pine gone grey and dry, each board its own grain
/// (the growth rings' lines, a knot or two), the gaps between them dark, the
/// name burned in with a poker (charred, soft at its edges, deeper where
/// the poker stopped), weathered lighter where hands and sun reached it,
/// grime down from the nails
fn rustic_board_tex(vary: u32) -> Tex {
    canvas_tex_opt(616, 292, false, move |ctx, w, h| {
        let seed = (rnd() * 4.0e9) as u32 ^ vary;
        with_own_stream(seed, || {
            let words = RS_WORDS[(vary as usize >> 3) % RS_WORDS.len()];
            ctx.set_fill_style("#8a7a62");
            ctx.fill_rect(0, 0, w, h);
            // the three boards: (top, bottom) in px; the gaps between dark
            let bh = h * 0.047 / 0.146;
            let gap = h * 0.0025 / 0.146;
            let boards = [(0.0, bh), (bh + gap, 2.0 * bh + gap), (2.0 * (bh + gap), h)];
            for (bi, &(y0, y1)) in boards.iter().enumerate() {
                ctx.save();
                ctx.begin_path();
                ctx.rect(0, y0, w, y1 - y0);
                ctx.clip();
                // each board its own grey-brown
                let k = 0.9 + 0.2 * rnd();
                ctx.set_fill_style(format!("rgb({:.0},{:.0},{:.0})", 138.0 * k, 122.0 * k, 98.0 * k));
                ctx.fill_rect(0, y0, w, y1 - y0);
                // the rings' lines, flat-sawn: long lines along the board, swelling
                // into a cathedral where the saw cut across the rings
                let (cx, sw) = (w * (0.15 + 0.7 * rnd()), w * (0.12 + 0.2 * rnd()));
                let rings = 18 + (rnd() * 10.0) as usize;
                // nested: each ring's line inside the last, the same wander
                let up = if rnd() < 0.5 { 1.0 } else { -1.0 };
                let (a1, f1, p1) = (0.6 + rnd() * 1.4, 0.004 + rnd() * 0.01, rnd() * 6.3);
                for r in 0..rings {
                    let t = (r as f64 + 0.3 * rnd()) / rings as f64;
                    let yb = y0 - 4.0 + (y1 - y0 + 8.0) * if up > 0.0 { t } else { 1.0 - t };
                    let lift = -up * (y1 - y0) * 0.8 * (1.0 - t);
                    let late = rnd() < 0.5;
                    ctx.set_stroke_style(format!("rgba(64,50,34,{:.2})", if late { 0.25 + 0.2 * rnd() } else { 0.1 + 0.12 * rnd() }));
                    ctx.set_line_width(if late { 1.2 + rnd() * 1.6 } else { 0.5 + rnd() * 0.8 });
                    ctx.begin_path();
                    let n = 40;
                    for k in 0..=n {
                        let xx = w * k as f64 / n as f64;
                        let yy = yb + lift * (-((xx - cx) / sw).powi(2)).exp() + a1 * (xx * f1 + p1).sin();
                        if k == 0 {
                            ctx.move_to(xx, yy);
                        } else {
                            ctx.line_to(xx, yy);
                        }
                    }
                    ctx.stroke();
                }
                // a knot now and then
                if rnd() < 0.6 {
                    let (kx, ky) = (w * (0.1 + 0.8 * rnd()), y0 + (y1 - y0) * (0.3 + 0.4 * rnd()));
                    let g = ctx.create_radial_gradient(kx, ky, 0, kx, ky, 7.0);
                    g.add_color_stop(0, "rgba(52,36,24,0.9)");
                    g.add_color_stop(0.6, "rgba(70,50,32,0.6)");
                    g.add_color_stop(1, "rgba(70,50,32,0)");
                    ctx.set_fill_style(&g);
                    ctx.begin_path();
                    ctx.ellipse(kx, ky, 9.0, 5.0, 0, 0, PI * 2.0);
                    ctx.fill();
                }
                // checks: fine splits along the grain at its ends
                for _ in 0..3 {
                    let y = y0 + (y1 - y0) * (0.2 + 0.6 * rnd());
                    let x0 = if rnd() < 0.5 { 0.0 } else { w - 40.0 - rnd() * 40.0 };
                    ctx.set_stroke_style("rgba(30,24,18,0.55)");
                    ctx.set_line_width(0.8);
                    ctx.begin_path();
                    ctx.move_to(x0, y);
                    ctx.line_to(x0 + 20.0 + rnd() * 40.0, y + (rnd() - 0.5) * 2.0);
                    ctx.stroke();
                }
                // the sawn ends and edges darker, weathered
                let ge = ctx.create_linear_gradient(0, y0, 0, y1);
                ge.add_color_stop(0, "rgba(40,32,24,0.35)");
                ge.add_color_stop(0.12, "rgba(40,32,24,0)");
                ge.add_color_stop(0.88, "rgba(40,32,24,0)");
                ge.add_color_stop(1, "rgba(40,32,24,0.35)");
                ctx.set_fill_style(&ge);
                ctx.fill_rect(0, y0, w, y1 - y0);
                let _ = bi;
                ctx.restore();
            }
            ctx.set_fill_style("rgba(24,18,12,0.9)");
            for &(_, y1) in &boards[..2] {
                ctx.fill_rect(0, y1, w, gap);
            }
            // the name, burned in: the poker's char soft round each stroke, its
            // middle darkest, unevenly (it was done by hand)
            let lines: Vec<&str> = [words.0, words.1].into_iter().filter(|s| !s.is_empty()).collect();
            let big = if lines.len() == 1 { 52.0 } else { 46.0 };
            let ys: Vec<(f64, f64)> =
                if lines.len() == 1 { vec![(h * 0.5 + big * 0.36, big)] } else { vec![(h * 0.43, big), (h * 0.79, 36.0)] };
            ctx.set_text_align("center");
            for (li, (line, &(y, size))) in lines.iter().zip(ys.iter()).enumerate() {
                let font = format!("bold {size:.0}px Copperplate");
                ctx.set_font(&font);
                let fit = ctx.measure_text(line).width;
                let sx = ((w * 0.86) / fit).min(1.0);
                ctx.save();
                ctx.translate(w / 2.0, y);
                ctx.scale(sx, 1.0);
                ctx.rotate((rnd() - 0.5) * 0.012);
                // the scorch round the letters
                for k in 0..10 {
                    let a = k as f64 / 10.0 * PI * 2.0;
                    let r = 1.6 + 0.6 * rnd();
                    ctx.set_fill_style("rgba(82,44,16,0.12)");
                    ctx.fill_text(line, a.cos() * r, a.sin() * r);
                }
                ctx.set_fill_style("rgba(48,26,10,0.85)");
                ctx.fill_text(line, 0, 0);
                ctx.set_fill_style("rgba(20,12,6,0.55)");
                ctx.fill_text(line, 0.5, 0.6);
                ctx.restore();
                let _ = li;
            }
            // the char uneven: the wood's grain shows lighter through it
            for _ in 0..500 {
                let (x, y) = (rnd() * w, rnd() * h);
                ctx.set_fill_style(format!("rgba(150,132,104,{:.2})", 0.05 + 0.1 * rnd()));
                ctx.fill_rect(x, y, 2.0 + rnd() * 8.0, 0.8);
            }
            // weathered: lighter in patches, dust on the boards' top edges
            for _ in 0..10 {
                let (x, y, r) = (rnd() * w, rnd() * h, 30.0 + rnd() * 70.0);
                let g = ctx.create_radial_gradient(x, y, 0, x, y, r);
                g.add_color_stop(0, &format!("rgba(178,168,150,{:.2})", 0.06 + 0.08 * rnd()));
                g.add_color_stop(1, "rgba(178,168,150,0)");
                ctx.set_fill_style(&g);
                ctx.fill_rect(0, 0, w, h);
            }
            // rust run down from each nail
            for &by in &[bh * 0.5, bh * 1.5 + gap, bh * 2.5 + 2.0 * gap] {
                for &bx in &[w * (0.5 - 0.105 / 0.308), w * (0.5 + 0.105 / 0.308)] {
                    let g = ctx.create_linear_gradient(bx, by, bx, by + 22.0);
                    g.add_color_stop(0, "rgba(84,46,22,0.45)");
                    g.add_color_stop(1, "rgba(84,46,22,0)");
                    ctx.set_fill_style(&g);
                    ctx.fill_rect(bx - 2.0, by, 4.0, 22.0);
                }
            }
        })
    })
}
