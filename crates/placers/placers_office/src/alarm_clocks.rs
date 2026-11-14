//! `place_alarm_clocks`
use game::prelude::*;
use placers_kit::noise::vn;
use placers_kit::place::open_tiles;
use placers_kit::place::ri;
use placers_kit::place::seed_of;

// Twin-bell alarm clock: 12% chance per level. A wind-up twin-bell
// alarm clock (Blender batch r14b: a 104 mm drum of painted steel on two
// splayed feet, two bells and their hammer, a carrying handle, the wind keys
// at its back), its printed dial yellowed behind a domed glass, its hands
// stopped where it ran down; the paint chipped to rust.
pub(crate) fn place_alarm_clocks(g: &mut Game, _dist: &[i32]) {
    use game::assets::props::prop_part_as_modelled;
    g.pp.alarm_clock_positions = vec![]; // reset per level
    if rnd() < 0.88 {
        return;
    } // 12% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[ri(open.len())];
    let clock = group();
    clock.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    clock.rotation().y = rnd() * PI * 2.0;
    // which clock, and when it stopped, from where it stands (no draw)
    let p = clock.pos();
    let seed = seed_of(&[p.x, p.z]);
    let paint = (seed % 5) as usize;
    let (h, m) = (f64::from((seed >> 4) % 12), f64::from((seed >> 8) % 60));
    let alarm = f64::from((seed >> 14) % 48) / 4.0;
    const Y: f64 = 0.0600;
    const DIAL: f64 = 0.0145;
    let col = [0xd2c8b0, 0x6e2a24, 0x1c1b1a, 0x3e5a48, 0x9aa0a2][paint];
    let case_mat = phong!({ color: col, specular: 0x2a2a2a, shininess: 24 });
    case_mat.m().map = Some(enamel_tex(seed));
    if let Some(geo) = prop_part_as_modelled("Real__alarmCase") {
        clock.add(&mesh(geo, &case_mat));
    }
    let chrome = phong!({ color: 0x5c5c5a, specular: 0xa4a4a0, shininess: 64 });
    chrome.m().map = Some(pitted_tex(seed));
    if let Some(geo) = prop_part_as_modelled("Real__alarmChrome") {
        clock.add(&mesh(geo, &chrome));
    }
    let dial = mesh(circle_geo!(0.0466, 64), lambert!({ map: &dial_tex(seed) }));
    dial.position().set(0, Y, DIAL);
    clock.add(&dial);
    let black = lambert!({ color: 0x1a1816 });
    let hand = |name: &str, z: f64, turn: f64, mat: &Mat| {
        if let Some(geo) = prop_part_as_modelled(name) {
            let pivot = group();
            pivot.add(&mesh(geo, mat));
            pivot.position().set(0, Y, z);
            pivot.rotation().z = -turn * PI * 2.0;
            clock.add(&pivot);
        }
    };
    hand("Real__alarmSet", DIAL + 0.0004, alarm / 12.0, &lambert!({ color: 0x7a2a1e }));
    hand("Real__alarmHour", DIAL + 0.0010, (h + m / 60.0) / 12.0, &black);
    hand("Real__alarmMinute", DIAL + 0.0017, m / 60.0, &black);
    if let Some(geo) = prop_part_as_modelled("Real__alarmGlass") {
        let glass = mesh(geo, glass_mat(&glass_dirt_tex(seed)));
        glass.set_render_order(2);
        clock.add(&glass);
    }
    g.world.add(&clock);
    // Track alarm clock position for rare ring audio events.
    let cp = clock.pos();
    g.pp.alarm_clock_positions.push(v3(cp.x, 0.08, cp.z));
}

/// The dial (12 at the top): printed on cream-painted tin, the minute
/// ring of dots, the hours in a bold geometric face, a luminous dot at
/// each hour gone brown, the alarm ring's fine track, the maker's name
/// (none real), UNADJUSTED and the one jewel; yellowed toward the rim,
/// a tide line where damp stood, fly specks, the paint crazed
fn dial_tex(seed: u32) -> Tex {
    canvas_tex_opt(512, 512, false, move |ctx, w, h| {
        let (cx, cy, r) = (w / 2.0, h / 2.0, w / 2.0);
        ctx.set_fill_style("#d9cfb4");
        ctx.fill_rect(0, 0, w, h);
        let g0 = ctx.create_radial_gradient(cx, cy, r * 0.5, cx, cy, r);
        g0.add_color_stop(0, "rgba(170,140,90,0)");
        g0.add_color_stop(1, "rgba(150,120,72,0.34)");
        ctx.set_fill_style(g0);
        ctx.fill_rect(0, 0, w, h);
        let ink = "rgba(26,22,18,0.92)";
        ctx.set_stroke_style(ink);
        ctx.set_line_width(2.5);
        ctx.begin_path();
        ctx.arc(cx, cy, r * 0.94, 0, 7);
        ctx.stroke();
        ctx.set_line_width(1.2);
        ctx.begin_path();
        ctx.arc(cx, cy, r * 0.87, 0, 7);
        ctx.stroke();
        ctx.set_fill_style(ink);
        for k in 0..60 {
            let a = k as f64 / 60.0 * PI * 2.0;
            let rr = if k % 5 == 0 { 4.6 } else { 2.2 };
            ctx.begin_path();
            ctx.arc(cx + a.sin() * r * 0.905, cy - a.cos() * r * 0.905, rr, 0, 7);
            ctx.fill();
        }
        ctx.set_text_align("center");
        ctx.set_text_baseline("middle");
        ctx.set_font("bold 70px Futura");
        for k in 1..=12 {
            let a = k as f64 / 12.0 * PI * 2.0;
            ctx.fill_text(&k.to_string(), cx + a.sin() * r * 0.70, cy - a.cos() * r * 0.70 + 3.0);
        }
        // the luminous dots inside the numerals, gone brown
        ctx.set_fill_style("rgba(150,128,80,0.9)");
        for k in 0..12 {
            let a = k as f64 / 12.0 * PI * 2.0;
            ctx.begin_path();
            ctx.arc(cx + a.sin() * r * 0.52, cy - a.cos() * r * 0.52, 6.0, 0, 7);
            ctx.fill();
        }
        ctx.set_fill_style(ink);
        ctx.set_font("bold 24px Futura");
        ctx.fill_text("HALDEN", cx, cy - r * 0.30);
        ctx.set_font("15px Helvetica Neue");
        ctx.fill_text("UNADJUSTED", cx, cy + r * 0.26);
        ctx.fill_text("ONE JEWEL", cx, cy + r * 0.34);
        // the paint crazed in fine cracks, the yellowing, a tide line
        let mut rr = SplitMix64::new(seed, 0xa1a7);
        ctx.set_line_width(0.8);
        for _ in 0..70 {
            let (mut x, mut y) = (rr.range(0.0, w), rr.range(0.0, h));
            ctx.set_stroke_style(&format!("rgba(90,70,40,{:.2})", rr.range(0.15, 0.35)));
            ctx.begin_path();
            ctx.move_to(x, y);
            for _ in 0..4 {
                x += rr.range(-14.0, 14.0);
                y += rr.range(-14.0, 14.0);
                ctx.line_to(x, y);
            }
            ctx.stroke();
        }
        let ty = cy + r * rr.range(0.45, 0.7);
        let gr = ctx.create_linear_gradient(0, ty - 30.0, 0, h);
        gr.add_color_stop(0, "rgba(130,96,50,0)");
        gr.add_color_stop(0.12, "rgba(130,96,50,0.3)");
        gr.add_color_stop(0.2, "rgba(130,96,50,0.12)");
        gr.add_color_stop(1, "rgba(120,90,50,0.18)");
        ctx.set_fill_style(gr);
        ctx.fill_rect(0, ty - 30.0, w, h - ty + 30.0);
        for _ in 0..90 {
            ctx.set_fill_style(&format!("rgba(50,38,25,{:.2})", rr.range(0.3, 0.8)));
            ctx.begin_path();
            ctx.arc(rr.range(0.0, w), rr.range(0.0, h), rr.range(0.8, 2.6), 0, 7);
            ctx.fill();
        }
    })
}

/// The enamel worn (multiplied over the paint): dulled in a mottle,
/// chipped to the dark steel and rusted in the chips, dust settled
fn enamel_tex(seed: u32) -> Tex {
    canvas_tex_opt(128, 128, false, move |ctx, w, h| {
        let n = w as usize;
        let mut img = ctx.create_image_data(w, h);
        par_rows(&mut img.data, n * 4, |j, out| {
            for i in 0..n {
                let (x, y) = (i as f64 / w, j as f64 / h);
                let m = vn(seed, x * 8.0, y * 8.0) * 0.6 + vn(seed ^ 0x6d, x * 30.0, y * 30.0) * 0.4;
                let v = (205.0 + 50.0 * m).min(255.0) as u8;
                out[i * 4..i * 4 + 4].copy_from_slice(&[v, v, (f64::from(v) * 0.97) as u8, 255]);
            }
        });
        ctx.put_image_data(&img, 0, 0);
        let mut r = SplitMix64::new(seed, 0xe7a3);
        // chips knocked off to the steel, small and ragged, rusted
        for _ in 0..14 {
            let (x, y) = (r.range(0.0, w), r.range(0.0, h));
            let rad = r.range(0.8, 2.6);
            ctx.set_fill_style("rgba(70,56,44,0.85)");
            ctx.begin_path();
            for k in 0..7 {
                let a = k as f64 / 7.0 * PI * 2.0;
                let rr = rad * r.range(0.5, 1.3);
                if k == 0 {
                    ctx.move_to(x + a.cos() * rr, y + a.sin() * rr);
                } else {
                    ctx.line_to(x + a.cos() * rr, y + a.sin() * rr);
                }
            }
            ctx.close_path();
            ctx.fill();
            ctx.set_fill_style("rgba(128,70,34,0.55)");
            ctx.begin_path();
            ctx.arc(x, y, rad * 0.45, 0, 7);
            ctx.fill();
        }
        // rust bled from the chips in faint runs
        for _ in 0..6 {
            let (x, y) = (r.range(0.0, w), r.range(0.0, h));
            let gr = ctx.create_linear_gradient(x, y, x, y + 14.0);
            gr.add_color_stop(0, "rgba(120,70,36,0.35)");
            gr.add_color_stop(1, "rgba(120,70,36,0)");
            ctx.set_fill_style(gr);
            ctx.fill_rect(x - 1.0, y, 2.0, 14);
        }
    })
}

/// The plating pitted and dulled (multiplied over the chrome)
fn pitted_tex(seed: u32) -> Tex {
    canvas_tex_opt(64, 64, false, move |ctx, w, h| {
        ctx.set_fill_style("#ffffff");
        ctx.fill_rect(0, 0, w, h);
        let mut r = SplitMix64::new(seed, 0x9e11);
        for _ in 0..18 {
            let (x, y, rr) = (r.range(0.0, w), r.range(0.0, h), r.range(3.0, 10.0));
            let gr = ctx.create_radial_gradient(x, y, 0, x, y, rr);
            gr.add_color_stop(0, "rgba(90,80,64,0.5)");
            gr.add_color_stop(1, "rgba(90,80,64,0)");
            ctx.set_fill_style(gr);
            ctx.fill_rect(x - rr, y - rr, rr * 2.0, rr * 2.0);
        }
        for _ in 0..60 {
            ctx.set_fill_style(&format!("rgba(96,56,30,{:.2})", r.range(0.3, 0.8)));
            ctx.fill_rect(r.range(0.0, w), r.range(0.0, h), r.range(0.6, 1.5), r.range(0.6, 1.5));
        }
    })
}

/// Dust on the glass (grey: how much), heavier toward its top
fn glass_dirt_tex(seed: u32) -> Tex {
    canvas_tex_opt(128, 128, false, move |ctx, w, h| {
        let n = w as usize;
        let mut img = ctx.create_image_data(w, h);
        par_rows(&mut img.data, n * 4, |j, out| {
            for i in 0..n {
                let (x, y) = (i as f64 / w, j as f64 / h);
                let d = vn(seed, x * 6.0, y * 6.0) * 0.6 + vn(seed ^ 0x19, x * 24.0, y * 24.0) * 0.4;
                let top = (1.0 - y) * 0.25;
                let v = ((0.2 + 0.35 * d + top) * 255.0).min(255.0) as u8;
                out[i * 4..i * 4 + 4].copy_from_slice(&[v, v, v, 255]);
            }
        });
        ctx.put_image_data(&img, 0, 0);
    })
}

/// The glass (`Patch::Glass`): the room seen in it by Fresnel, its dust
/// from the map's grey
fn glass_mat(map: &Tex) -> Mat {
    let m = phong!({ color: 0xffffff, specular: 0x9a9a9a, shininess: 200, transparent: true, depthWrite: false });
    {
        let mut mm = m.m();
        mm.map = Some(map.clone());
        mm.patch = Patch::Glass;
        mm.look_settled = true;
        mm.name = "alarm_glass".into();
    }
    {
        let pu = m.r().patch_uniforms.clone();
        let f = &mut pu.borrow_mut().f;
        f[0] = 0.45;
        f[4] = 0.5;
        f[5] = 0.6;
        f[6] = 0.3;
        f[7] = 0.0;
        f[8] = 1.2;
        f[9] = 0.2;
        f[10] = 0.19;
        f[11] = 0.17;
        f[12] = 0.0;
        f[13] = 0.07;
    }
    m
}
