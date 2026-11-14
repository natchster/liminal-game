//! `place_wristwatches`
use game::prelude::*;
use placers_kit::noise::vn;
use placers_kit::place::open_tiles;
use placers_kit::place::seed_of;

// Watch on wristband (floor): 7% chance per level. A 38 mm quartz
// wristwatch (Blender batch r14b; the Timex Weekender class) lying on its
// back, unbuckled, its two-piece strap lying out from the lugs, its tip
// curled: the case plated, worn through at the edges; a domed acrylic
// crystal, scratched; a printed dial, its hands where they stopped.
pub(crate) fn place_wristwatches(g: &mut Game, _dist: &[i32]) {
    use game::assets::props::prop_part_as_modelled;
    g.pp.wristwatch_positions = vec![]; // reset per level
    if rnd() < 0.93 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    // the old colour draws, call for call: the strap, the case, the dial
    let band_colors = [0x101010, 0x603020, 0xa0a0a8, 0xc0a060, 0x201050, 0x804a20];
    let band_c = choice(&band_colors);
    let band = band_colors.iter().position(|&c| c == band_c).unwrap_or(0);
    let case_colors = [0xc0a060, 0xa0a0a8, 0xe0d8c0, 0x606060];
    let case_c = choice(&case_colors);
    let case_k = case_colors.iter().position(|&c| c == case_c).unwrap_or(0);
    let face_colors = [0xf0e8d0, 0x101010, 0xf0f0f0, 0x2060a0];
    let face_c = choice(&face_colors);
    let dial = face_colors.iter().position(|&c| c == face_c).unwrap_or(0);
    // where the hands stopped (the old hands' draws): the hour from its own
    // draw, carried on by the minutes; the second hand
    let hour_ang = rnd() * PI * 2.0;
    let min_ang = rnd() * PI * 2.0;
    let sec_ang = rnd() * PI * 2.0;
    // Stepped-tick second hand — 40% chance ("still ticking").
    let is_ticking = rnd() < 0.4;
    let px = g.wx(x) + (rnd() - 0.5) * 2.4;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.4;
    let yaw = rnd() * PI * 2.0;
    let frac = |a: f64| ((a + PI / 2.0) / (PI * 2.0)).rem_euclid(1.0);
    let minutes = frac(min_ang);
    let hours = ((frac(hour_ang) * 12.0).floor() + minutes) / 12.0;
    let seconds = frac(sec_ang);
    let seed = seed_of(&[px, pz]);
    let wg = group();
    // the case: plating over brass (dark, its colour in its highlight)
    let (col, spec, shin) =
        [(0x4a3c22, 0x8c7448, 54.0), (0x404042, 0x8e8e90, 70.0), (0x58564f, 0x6c6a64, 26.0), (0x1e1e20, 0x46464a, 40.0)][case_k];
    let metal = phong!({ color: col, specular: spec, shininess: shin });
    {
        let mut mm = metal.m();
        mm.map = Some(plating_tex(case_k, seed));
    }
    if let Some(geo) = prop_part_as_modelled("Real__watchCase") {
        wg.add(&mesh(geo, &metal));
    }
    if let Some(geo) = prop_part_as_modelled("Real__watchStrap") {
        let bb = geo.bounding_box();
        let (z0, z1) = (bb.min.z, bb.max.z);
        let strap = mesh(geo, strap_mat(band, z0, z1, seed));
        wg.add(&strap);
    }
    // the dial, printed, under its hands
    let face = mesh(circle_geo!(0.0158, 48), lambert!({ map: &dial_tex(dial, seed) }));
    face.rotation().x = -PI / 2.0;
    face.position().y = 0.0061;
    wg.add(&face);
    let dark = dial == 1 || dial == 3;
    let hand_mat = if dark { lambert!({ color: 0xcfc8b2 }) } else { lambert!({ color: 0x161514 }) };
    let hand = |name: &str, y: f64, frac: f64, mat: &Mat| {
        let pivot = group();
        if let Some(geo) = prop_part_as_modelled(name) {
            pivot.add(&mesh(geo, mat));
        }
        pivot.position().y = y;
        pivot.rotation().y = -frac * PI * 2.0;
        wg.add(&pivot);
        pivot
    };
    hand("Real__watchHour", 0.00618, hours, &hand_mat);
    hand("Real__watchMinute", 0.00636, minutes, &hand_mat);
    let second = hand("Real__watchSecond", 0.00654, seconds, &lambert!({ color: if dark { 0xa0402c } else { 0x7a2218 } }));
    if let Some(geo) = prop_part_as_modelled("Real__watchCrystal") {
        let crystal = mesh(geo, crystal_mat(&crystal_dirt_tex(seed)));
        crystal.set_render_order(2);
        wg.add(&crystal);
    }
    if is_ticking {
        // still going: the second hand steps once a second
        let mut ang = -seconds * PI * 2.0;
        let mut timer = 0.0;
        g.animate(move |_g, dt, _t| {
            timer += dt;
            if timer >= 1.0 {
                timer -= 1.0;
                ang -= PI * 2.0 / 60.0;
                second.rotation().y = ang;
            }
        });
    }
    wg.user().set("isTicking", is_ticking);
    wg.position().set(px, 0, pz);
    wg.rotation().y = yaw;
    g.world.add(&wg);
    // Register position only for ticking watches.
    if is_ticking {
        let p = wg.pos();
        g.pp.wristwatch_positions.push(v3(p.x, 0, p.z));
    }
}

/// The dials (sRGB): aged cream, black, white gone grey, navy; and the
/// ink printed on each
const DIALS: [(&str, &str, &str); 4] = [
    ("#d6ccb0", "rgba(26,24,22,0.92)", "rgba(150,40,30,0.85)"),
    ("#1d1c1b", "rgba(222,214,196,0.9)", "rgba(196,92,60,0.85)"),
    ("#d2cfc6", "rgba(26,24,22,0.92)", "rgba(150,40,30,0.85)"),
    ("#29384e", "rgba(222,216,200,0.9)", "rgba(206,120,80,0.85)"),
];

/// A watch's dial seen from above (12 at the top): the minute track, the
/// hours in a plain grotesque, the 24-hour ring small inside them, the
/// maker's name (none real) and QUARTZ; dulled toward its rim, freckled
/// where damp got under the crystal
fn dial_tex(k: usize, seed: u32) -> Tex {
    canvas_tex_opt(256, 256, false, move |ctx, w, h| {
        let (ground, ink, red) = DIALS[k];
        let (cx, cy, r) = (w / 2.0, h / 2.0, w / 2.0);
        ctx.set_fill_style(ground);
        ctx.fill_rect(0, 0, w, h);
        let g0 = ctx.create_radial_gradient(cx, cy, r * 0.55, cx, cy, r);
        g0.add_color_stop(0, "rgba(120,100,70,0)");
        g0.add_color_stop(1, "rgba(120,100,70,0.22)");
        ctx.set_fill_style(g0);
        ctx.fill_rect(0, 0, w, h);
        ctx.set_stroke_style(ink);
        ctx.set_line_cap("butt");
        for i in 0..60 {
            let a = i as f64 / 60.0 * PI * 2.0;
            let five = i % 5 == 0;
            let (r0, r1) = if five { (0.86, 0.97) } else { (0.92, 0.97) };
            ctx.set_line_width(if five { 3.2 } else { 1.2 });
            ctx.begin_path();
            ctx.move_to(cx + a.sin() * r * r0, cy - a.cos() * r * r0);
            ctx.line_to(cx + a.sin() * r * r1, cy - a.cos() * r * r1);
            ctx.stroke();
        }
        ctx.set_fill_style(ink);
        ctx.set_text_align("center");
        ctx.set_text_baseline("middle");
        ctx.set_font("bold 30px Helvetica Neue");
        for i in 1..=12 {
            let a = i as f64 / 12.0 * PI * 2.0;
            ctx.fill_text(&i.to_string(), cx + a.sin() * r * 0.70, cy - a.cos() * r * 0.70 + 1.0);
        }
        ctx.set_fill_style(red);
        ctx.set_font("bold 11px Helvetica Neue");
        for i in 13..=24 {
            let a = (i - 12) as f64 / 12.0 * PI * 2.0;
            ctx.fill_text(&i.to_string(), cx + a.sin() * r * 0.47, cy - a.cos() * r * 0.47);
        }
        ctx.set_fill_style(ink);
        ctx.set_font("bold 12px Futura");
        ctx.fill_text("HALDEN", cx, cy - r * 0.26);
        ctx.set_font("8px Helvetica Neue");
        ctx.fill_text("QUARTZ", cx, cy + r * 0.26);
        ctx.fill_text("WATER RESISTANT", cx, cy + r * 0.36);
        // freckles where damp got in, and the print dulled
        let mut rr = SplitMix64::new(seed, 0xd1a1);
        let n = 30 + (rr.next() * 50.0) as usize;
        for _ in 0..n {
            let a = rr.range(0.0, PI * 2.0);
            let d = r * rr.range(0.2, 0.98).sqrt();
            ctx.set_fill_style(format!("rgba(110,78,40,{:.2})", rr.range(0.15, 0.5)));
            ctx.begin_path();
            ctx.arc(cx + a.cos() * d, cy + a.sin() * d, rr.range(0.6, 2.2), 0, 7);
            ctx.fill();
        }
        if rr.next() < 0.5 {
            // a tide mark low on the dial, where water stood once
            let y = cy + r * rr.range(0.35, 0.6);
            let gr = ctx.create_linear_gradient(0, y - 18.0, 0, h);
            gr.add_color_stop(0, "rgba(120,90,50,0)");
            gr.add_color_stop(0.15, "rgba(120,90,50,0.22)");
            gr.add_color_stop(1, "rgba(120,90,50,0.10)");
            ctx.set_fill_style(gr);
            ctx.fill_rect(0, y - 18.0, w, h - y + 18.0);
        }
    })
}

/// The strap (seen from above, its length down the canvas from z1 at
/// the top to z0): leather's grain, creased and darkened at the hole it
/// was worn on and where the wrist sweated; scuffed pale; or a woven
/// webbing's ribs
fn strap_mat(band: usize, z0: f64, z1: f64, seed: u32) -> Mat {
    // black, dark brown, grey webbing, tan, navy webbing, brown (sRGB)
    let col: [(f64, f64, f64); 6] =
        [(30.0, 28.0, 26.0), (64.0, 42.0, 28.0), (92.0, 92.0, 90.0), (118.0, 86.0, 56.0), (38.0, 42.0, 62.0), (88.0, 56.0, 34.0)];
    let webbing = band == 2 || band == 4;
    let c = col[band];
    let tex = canvas_tex_opt(64, 512, false, move |ctx, w, h| {
        let vy = |z: f64| (1.0 - (z - z0) / (z1 - z0)) * h;
        let n = w as usize;
        let mut img = ctx.create_image_data(w, h);
        let s = seed;
        par_rows(&mut img.data, n * 4, |j, out| {
            for i in 0..n {
                let (x, y) = (i as f64, j as f64);
                let k = if webbing {
                    let rib = (x * 1.9).sin() * 0.5 + 0.5;
                    0.86 + 0.12 * rib + 0.06 * (vn(s, x * 0.3, y * 0.05) - 0.5)
                } else {
                    let grain = vn(s, x * 0.9, y * 0.9) * 0.5 + vn(s ^ 5, x * 0.25, y * 0.25) * 0.5;
                    0.84 + 0.22 * grain
                };
                let px = |v: f64| (v * k).clamp(0.0, 255.0) as u8;
                out[i * 4..i * 4 + 4].copy_from_slice(&[px(c.0), px(c.1), px(c.2), 255]);
            }
        });
        ctx.put_image_data(&img, 0, 0);
        // where it was worn: the hole it was buckled on, creased across
        // and darkened, the sweat band behind the case on both pieces
        let worn = vy(0.0905);
        for (y, a) in [(worn, 0.35), (vy(0.032), 0.22), (vy(-0.030), 0.22)] {
            let gr = ctx.create_linear_gradient(0, y - 20.0, 0, y + 20.0);
            gr.add_color_stop(0, "rgba(10,8,6,0)");
            gr.add_color_stop(0.5, &format!("rgba(10,8,6,{a})"));
            gr.add_color_stop(1, "rgba(10,8,6,0)");
            ctx.set_fill_style(gr);
            ctx.fill_rect(0, y - 20.0, w, 40);
        }
        let mut r = SplitMix64::new(seed, 0x57a9);
        ctx.set_line_width(1.0);
        for k in 0..5 {
            let y = worn + (k as f64 - 2.0) * 3.0 + r.range(-1.0, 1.0);
            ctx.set_stroke_style(if webbing { "rgba(20,18,16,0.25)" } else { "rgba(14,10,8,0.45)" });
            ctx.begin_path();
            ctx.move_to(0, y);
            ctx.bezier_curve_to(w * 0.3, y + r.range(-2.0, 2.0), w * 0.7, y + r.range(-2.0, 2.0), w, y + r.range(-1.0, 1.0));
            ctx.stroke();
        }
        // scuffs, pale, and the finish rubbed off in patches
        for _ in 0..40 {
            let (x, y) = (r.range(0.0, w), r.range(0.0, h));
            ctx.set_fill_style(format!("rgba(200,184,160,{:.2})", r.range(0.05, 0.18)));
            ctx.begin_path();
            ctx.ellipse(x, y, r.range(1.0, 5.0), r.range(0.6, 2.0), r.range(0.0, 3.0), 0, 7);
            ctx.fill();
        }
    });
    if webbing {
        lambert!({ map: &tex })
    } else {
        phong!({ map: &tex, specular: 0x161412, shininess: 18 })
    }
}

/// The plating rubbed through at the edges and lugs to the brass under
/// it, and grime in the joints (multiplied over the metal's colour)
fn plating_tex(k: usize, seed: u32) -> Tex {
    canvas_tex_opt(64, 64, false, move |ctx, w, h| {
        ctx.set_fill_style("#ffffff");
        ctx.fill_rect(0, 0, w, h);
        let mut r = SplitMix64::new(seed, 0x91a7 + k as u64);
        let brass = if k == 0 { "rgba(170,120,60,0.55)" } else { "rgba(150,110,60,0.45)" };
        for _ in 0..26 {
            let (x, y) = (r.range(0.0, w), r.range(0.0, h));
            ctx.set_fill_style(brass);
            ctx.begin_path();
            ctx.ellipse(x, y, r.range(1.0, 4.0), r.range(0.5, 2.0), r.range(0.0, 3.0), 0, 7);
            ctx.fill();
        }
        for _ in 0..30 {
            let (x, y) = (r.range(0.0, w), r.range(0.0, h));
            ctx.set_fill_style(format!("rgba(40,34,26,{:.2})", r.range(0.2, 0.5)));
            ctx.fill_rect(x, y, r.range(1.0, 3.0), r.range(1.0, 2.0));
        }
    })
}

/// The domed acrylic: polished once, now hazed and scratched (grey:
/// how much)
fn crystal_dirt_tex(seed: u32) -> Tex {
    canvas_tex_opt(128, 128, false, move |ctx, w, h| {
        let n = w as usize;
        let mut img = ctx.create_image_data(w, h);
        let s = seed;
        par_rows(&mut img.data, n * 4, |j, out| {
            for i in 0..n {
                let (x, y) = (i as f64 / w, j as f64 / h);
                let d = vn(s, x * 5.0, y * 5.0) * 0.6 + vn(s ^ 3, x * 19.0, y * 19.0) * 0.4;
                let v = ((0.16 + 0.30 * d) * 255.0) as u8;
                out[i * 4..i * 4 + 4].copy_from_slice(&[v, v, v, 255]);
            }
        });
        ctx.put_image_data(&img, 0, 0);
        let mut r = SplitMix64::new(seed, 0xc4a5);
        ctx.set_line_cap("round");
        for _ in 0..60 {
            let (x0, y0) = (r.range(0.0, w), r.range(0.0, h));
            let a = r.range(0.0, PI);
            let l = r.range(4.0, 26.0);
            ctx.set_stroke_style(format!("rgba(240,240,240,{:.2})", r.range(0.3, 0.75)));
            ctx.set_line_width(r.range(0.5, 1.0));
            ctx.begin_path();
            ctx.move_to(x0, y0);
            ctx.line_to(x0 + a.cos() * l, y0 + a.sin() * l);
            ctx.stroke();
        }
    })
}

/// The crystal as the windows' glass (`Patch::Glass`): the room seen in
/// it by Fresnel, its haze from the map's grey
fn crystal_mat(map: &Tex) -> Mat {
    let m = phong!({ color: 0xffffff, specular: 0x9a9a9a, shininess: 220, transparent: true, depthWrite: false });
    {
        let mut mm = m.m();
        mm.map = Some(map.clone());
        mm.patch = Patch::Glass;
        mm.look_settled = true;
        mm.name = "watch_crystal".into();
    }
    {
        let pu = m.r().patch_uniforms.clone();
        let f = &mut pu.borrow_mut().f;
        f[0] = 0.5;
        f[4] = 0.5;
        f[5] = 0.6;
        f[6] = 0.3;
        f[7] = 0.0;
        f[8] = 1.4;
        f[9] = 0.2;
        f[10] = 0.195;
        f[11] = 0.18;
        f[12] = 0.0;
        f[13] = 0.07;
    }
    m
}
