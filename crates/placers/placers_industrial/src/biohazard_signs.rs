//! `place_biohazard_signs`
use game::prelude::*;
use placers_kit::place::mount_on_wall;
use placers_kit::place::wall_candidates;

// Biohazard sign: 4% chance per level. An ISO 7010 biological hazard
// sign screwed to a wall (family `six_things_hung_on_a_wall`).
pub(crate) fn place_biohazard_signs(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = candidates[(rnd() * candidates.len() as f64).floor() as usize];
    // An A4 rigid PVC safety sign (Blender family `six_things_hung_on_a_wall`): the ISO 7010 W009
    // warning triangle, its trefoil drawn to Baldwin's construction, over
    // BIOHAZARD, screwed to the wall at its corners; its yellow faded, its
    // print scuffed to the white plastic, rust run from its screws, one in
    // three with a screw gone and that corner sprung off the wall
    // (docs/design/PROPS.md)
    use game::assets::props::prop_part_as_modelled;
    let sign = group();
    mount_on_wall(g, &sign, &c, 0.005, 1.70, 0.6);
    let p = sign.pos();
    let vary = bh_vary(p.x, p.z);
    let lifted = vary.is_multiple_of(3);
    // the parts are modelled with the wall's face at z = 0: 5 mm behind the spot
    let body = group();
    body.position().z = -0.005;
    sign.add(&body);
    if let Some(geo) = prop_part_as_modelled(if lifted { "Real__biohazardPlateLifted" } else { "Real__biohazardPlate" }) {
        body.add(&mesh(geo, lambert!({ map: &biohazard_sign_tex(vary) })));
    }
    if let Some(geo) = prop_part_as_modelled(if lifted { "Real__biohazardScrews3" } else { "Real__biohazardScrews" }) {
        body.add(&mesh(geo, phong!({ color: 0x45403a, specular: 0x34322e, shininess: 28 })));
    }
    g.world.add(&sign);
}

/// Something of where a thing was put, to vary what its painters paint
/// (a painter's own stream starts the same on every floor)
fn bh_vary(x: f64, z: f64) -> u32 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for v in [x, z] {
        h ^= v.to_bits();
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    (h ^ (h >> 29) ^ (h >> 47)) as u32
}

/// Is the point (x, y), in the units of Baldwin and Runkle's construction
/// (the symbol's centre at the origin, y up; the outer circles 300 across),
/// inside the biohazard trefoil? Three circles of radius 150 whose centres
/// are 110 out at 90, 210 and 330 degrees, less three of radius 105 whose
/// centres are 150 out, less the centre's hole (60 across), the 10-wide
/// slits from the hole to each inner circle and the 40-wide gaps at each
/// lobe's end; and the ring (radius 100 to 135) where it crosses the inner
/// circles, 10 short of their edges
fn bh_trefoil(x: f64, y: f64) -> bool {
    let r = x.dhypot(y);
    let mut outer = false;
    let mut inner = false;
    let mut ring = false;
    let mut cut = r < 30.0;
    for k in 0..3 {
        let a = PI / 2.0 + k as f64 * PI * 2.0 / 3.0;
        let (ux, uy) = (a.dcos(), a.dsin());
        if (x - ux * 110.0).dhypot(y - uy * 110.0) < 150.0 {
            outer = true;
        }
        let di = (x - ux * 150.0).dhypot(y - uy * 150.0);
        if di < 105.0 {
            inner = true;
        }
        if di < 95.0 && (100.0..=135.0).contains(&r) {
            ring = true;
        }
        let (along, across) = (x * ux + y * uy, (x * uy - y * ux).abs());
        if (0.0..=50.0).contains(&along) && across < 5.0 || (225.0..=275.0).contains(&along) && across < 20.0 {
            cut = true;
        }
    }
    outer && !inner && !cut || ring
}

/// The sign's face as its plate's front is mapped (210 x 297 mm): signal
/// yellow, the ISO 7010 W009 triangle (its black band 5.3% of its side, its
/// outer corners rounded) and the trefoil in it, BIOHAZARD under it in a
/// bold grotesque; faded by years of light, the print scuffed through to
/// the white PVC, grime, rust run down from the screws, dust on its top
fn biohazard_sign_tex(vary: u32) -> Tex {
    let ppm = 2000.0;
    canvas_tex_opt(420, 594, false, move |ctx, w, h| {
        let seed = (rnd() * 4.0e9) as u32 ^ vary;
        with_own_stream(seed, || {
            // faded signal yellow, the ink's black gone a little brown
            let fade = 0.25 + 0.2 * rnd();
            let yel = [234.0 + 8.0 * fade, 172.0 + 40.0 * fade, 30.0 + 90.0 * fade];
            let ink = [34.0 + 30.0 * fade, 30.0 + 26.0 * fade, 26.0 + 18.0 * fade];
            let rgb = |c: [f64; 3]| format!("rgb({:.0},{:.0},{:.0})", c[0], c[1], c[2]);
            ctx.set_fill_style(rgb(yel));
            ctx.fill_rect(0, 0, w, h);
            // the triangle: side 180 mm, its apex 12 mm from the top
            let s = 0.180 * ppm;
            let th = s * 3f64.sqrt() / 2.0;
            let (ax, ay) = (w / 2.0, 0.013 * ppm);
            let cr = s * 0.027;
            let tri = |ctx: &mut Canvas, k: f64| {
                // the triangle shrunk about its centroid by `k` px (perpendicular)
                let (gx, gy) = (ax, ay + th * 2.0 / 3.0);
                let f = 1.0 - k / (th / 3.0);
                let pts = [(ax, ay), (ax + s / 2.0, ay + th), (ax - s / 2.0, ay + th)];
                ctx.begin_path();
                for (i, &(x, y)) in pts.iter().enumerate() {
                    let (x, y) = (gx + (x - gx) * f, gy + (y - gy) * f);
                    if i == 0 {
                        ctx.move_to(x, y);
                    } else {
                        ctx.line_to(x, y);
                    }
                }
                ctx.close_path();
            };
            ctx.set_fill_style(rgb(ink));
            ctx.set_stroke_style(rgb(ink));
            ctx.set_line_join("round");
            ctx.set_line_width(cr * 2.0);
            tri(ctx, cr);
            ctx.fill();
            ctx.stroke();
            ctx.set_fill_style(rgb(yel));
            tri(ctx, s * 0.0533);
            ctx.fill();
            // BIOHAZARD
            ctx.set_fill_style(rgb(ink));
            ctx.set_text_align("center");
            ctx.set_font(&format!("bold {:.0}px Helvetica", 0.036 * ppm));
            ctx.fill_text_max("BIOHAZARD", w / 2.0, 0.232 * ppm, w - 0.03 * ppm);
            // the trefoil, each pixel tested against its construction (4 x 4 samples)
            let sym_h = th * 0.458;
            let u = sym_h / 465.0; // px a construction unit
            let (cx, cy) = (ax, ay + th * 0.665);
            let (x0, x1) = ((cx - 250.0 * u).floor().max(0.0) as usize, ((cx + 250.0 * u).ceil() as usize).min(w as usize));
            let (y0, y1) = ((cy - 265.0 * u).floor().max(0.0) as usize, ((cy + 210.0 * u).ceil() as usize).min(h as usize));
            let (bw, bh) = (x1 - x0, y1 - y0);
            let mut img = ctx.get_image_data(x0, y0, bw, bh);
            par_rows(&mut img.data, bw * 4, |j, out| {
                for i in 0..bw {
                    let mut n = 0;
                    for sj in 0..4 {
                        for si in 0..4 {
                            let px = (x0 + i) as f64 + (si as f64 + 0.5) / 4.0;
                            let py = (y0 + j) as f64 + (sj as f64 + 0.5) / 4.0;
                            if bh_trefoil((px - cx) / u, (cy - py) / u) {
                                n += 1;
                            }
                        }
                    }
                    let a = n as f64 / 16.0;
                    for k in 0..3 {
                        let v = f64::from(out[i * 4 + k]);
                        out[i * 4 + k] = (v + (ink[k] - v) * a).round() as u8;
                    }
                }
            });
            ctx.put_image_data(&img, x0, y0);
            // faded more where the light fell: toward its top
            let gf = ctx.create_linear_gradient(0, 0, 0, h);
            gf.add_color_stop(0, "rgba(246,236,200,0.16)");
            gf.add_color_stop(1, "rgba(246,236,200,0.04)");
            ctx.set_fill_style(&gf);
            ctx.fill_rect(0, 0, w, h);
            // grime: a film, heavier low down and round the edges, in patches
            for _ in 0..18 {
                let (x, y, r) = (rnd() * w, h * rnd().dpowf(0.6), 40.0 + rnd() * 120.0);
                let g = ctx.create_radial_gradient(x, y, 0, x, y, r);
                g.add_color_stop(0, &format!("rgba(70,60,44,{:.3})", 0.04 + 0.07 * rnd()));
                g.add_color_stop(1, "rgba(70,60,44,0)");
                ctx.set_fill_style(&g);
                ctx.fill_rect(0, 0, w, h);
            }
            let ge = ctx.create_linear_gradient(0, h * 0.75, 0, h);
            ge.add_color_stop(0, "rgba(60,50,38,0)");
            ge.add_color_stop(1, "rgba(60,50,38,0.22)");
            ctx.set_fill_style(&ge);
            ctx.fill_rect(0, 0, w, h);
            // scuffs: fine scratches through the print to the white plastic,
            // most on its lower half where things brushed past
            ctx.set_line_cap("round");
            for _ in 0..140 {
                let (x, y) = (rnd() * w, h * (1.0 - rnd().dpowf(1.6)));
                let a = (rnd() - 0.5) * 1.2 + if rnd() < 0.5 { 0.0 } else { PI / 2.0 };
                let l = 3.0 + rnd() * rnd() * 40.0;
                ctx.set_stroke_style(format!("rgba(232,228,214,{:.2})", 0.25 + 0.45 * rnd()));
                ctx.set_line_width(0.4 + rnd() * 0.9);
                ctx.begin_path();
                ctx.move_to(x, y);
                ctx.quadratic_curve_to(
                    x + a.dcos() * l * 0.5 + (rnd() - 0.5) * 3.0,
                    y + a.dsin() * l * 0.5,
                    x + a.dcos() * l,
                    y + a.dsin() * l,
                );
                ctx.stroke();
            }
            // a few scrapes where the ink came off in a patch
            for _ in 0..4 {
                let (x, y) = (rnd() * w, h * (0.3 + 0.7 * rnd()));
                for _ in 0..12 {
                    ctx.set_fill_style(format!("rgba(226,222,206,{:.2})", 0.3 + 0.4 * rnd()));
                    ctx.begin_path();
                    ctx.ellipse(
                        x + (rnd() - 0.5) * 16.0,
                        y + (rnd() - 0.5) * 8.0,
                        1.0 + rnd() * 4.0,
                        0.5 + rnd() * 1.5,
                        rnd() * 0.5,
                        0,
                        PI * 2.0,
                    );
                    ctx.fill();
                }
            }
            // rust run from each screw (15 mm in from each side)
            for (sx, sy) in [(0.015, 0.015), (0.195, 0.015), (0.015, 0.282), (0.195, 0.282)] {
                let (x, y) = (sx * ppm, sy * ppm);
                let len = 20.0 + rnd() * 60.0;
                let g = ctx.create_linear_gradient(x, y, x, y + len);
                g.add_color_stop(0, &format!("rgba(104,52,22,{:.2})", 0.35 + 0.3 * rnd()));
                g.add_color_stop(1, "rgba(104,52,22,0)");
                ctx.set_fill_style(&g);
                ctx.begin_path();
                ctx.move_to(x - 4.0, y);
                ctx.line_to(x + 4.0, y);
                ctx.line_to(x + 1.5 + (rnd() - 0.5) * 3.0, y + len);
                ctx.line_to(x - 1.5 + (rnd() - 0.5) * 3.0, y + len);
                ctx.close_path();
                ctx.fill();
                let stops = [(0.0, "rgba(90,44,18,0.5)".to_string()), (1.0, "rgba(90,44,18,0)".to_string())];
                let gr = ctx.create_radial_gradient(x, y, 0, x, y, 12.0);
                for (at, c) in &stops {
                    gr.add_color_stop(*at, c);
                }
                ctx.set_fill_style(&gr);
                ctx.fill_rect(x - 12.0, y - 12.0, 24.0, 24.0);
            }
            // dust along its top edge
            let gd = ctx.create_linear_gradient(0, 0, 0, 14);
            gd.add_color_stop(0, "rgba(150,144,132,0.5)");
            gd.add_color_stop(1, "rgba(150,144,132,0)");
            ctx.set_fill_style(&gd);
            ctx.fill_rect(0, 0, w, 14);
        })
    })
}
