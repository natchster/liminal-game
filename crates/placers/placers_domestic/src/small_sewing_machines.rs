//! `place_small_sewing_machines`
use game::prelude::*;
use placers_kit::baking;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// A portable sewing machine on the floor (Blender family `kitchen_and_hearth_left_behind`): a
// black-japanned machine of the Singer 221 class (378 x 184 x 248 mm), its
// gold decals and its maker's name (HALDANE, no real maker) rubbed thin, its
// bright parts tarnished, a spool of thread still on its pin; its handwheel
// turning slowly by itself. 4% chance per level.
pub(crate) fn place_small_sewing_machines(g: &mut Game, _dist: &[i32]) {
    use game::assets::props::prop_part_as_modelled;
    g.pp.sewing_positions.clear(); // reset per level
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = pick(&open_tiles);
    let sewing = group();
    let japan = phong!({ map: &sew_japan_tex(), color: 0xffffff, specular: 0x3a3a3a, shininess: 38 });
    let bright = phong!({ map: &sew_bright_tex(), color: 0xffffff, specular: 0x8a8a88, shininess: 64 });
    for (name, mat) in [("Real__sewBed", &japan), ("Real__sewArm", &japan), ("Real__sewSteel", &bright)] {
        if let Some(geo) = prop_part_as_modelled(name) {
            sewing.add(&mesh(geo, mat));
        }
    }
    // the handwheel and its knob on their axle (x), so it can turn
    let (ax, ay, az) = SEW_AXLE;
    let wheel = group();
    wheel.position().set(ax, ay, az);
    for (name, mat) in [("Real__sewWheel", &japan), ("Real__sewWheelKnob", &bright)] {
        if let Some(geo) = prop_part_as_modelled(name) {
            let m = mesh(geo, mat);
            m.position().set(-ax, -ay, -az);
            wheel.add(&m);
        }
    }
    sewing.add(&wheel);
    // the gold decals, laid on its faces: the maker's name on the arm,
    // scrolls on the bed's front corners, the badge on the pillar
    let decal = |tex: &Tex| lambert!({ map: tex, color: 0xffffff, transparent: true, depthWrite: false, polygonOffset: true, polygonOffsetFactor: -2, polygonOffsetUnits: -2 });
    let name = mesh(plane_geo!(0.140, 0.034), decal(&sew_name_tex()));
    name.position().set(-0.030, 0.2078, SEW_ARM_FRONT + 0.0004);
    sewing.add(&name);
    for side in [-1.0, 1.0] {
        let scroll = mesh(plane_geo!(0.070, 0.030), decal(&sew_scroll_tex()));
        scroll.rotation().x = -PI / 2.0;
        scroll.scale().x = side;
        scroll.position().set(0.022 + side * 0.058, SEW_TOP + 0.0004, 0.031);
        sewing.add(&scroll);
    }
    let badge = mesh(plane_geo!(0.030, 0.030), decal(&sew_badge_tex()));
    badge.position().set(0.101, 0.142, SEW_ARM_FRONT + 0.0004);
    sewing.add(&badge);
    // a spool of thread on the pin (its colour from where it stands)
    let (px, pz) = (g.wx(x), g.wz(z));
    let thread = [0xd8d2c0, 0x2a2a2c, 0x5a3a2a, 0x2c3448, 0x8a8478][((px * 3.7 + pz * 5.3).abs() as usize) % 5];
    if let Some(geo) = prop_part_as_modelled("Real__sewSpool") {
        sewing.add(&mesh(geo, lambert!({ map: &sew_thread_tex(), color: thread })));
    }
    // the wheel turns slowly by itself (the old rate's draw)
    let rot_rate = 0.3 + rnd() * 0.3;
    let wheel2 = wheel.clone();
    g.animate(move |_g, _dt, t| {
        wheel2.rotation().x = -t * rot_rate;
    });
    sewing.position().set(px + (rnd() - 0.5) * 1.8, 0, pz + (rnd() - 0.5) * 1.8);
    sewing.rotation().y = rnd() * PI * 2.0;
    g.world.add(&sewing);
    // Track for whirr audio.
    let sp = sewing.pos();
    g.pp.sewing_positions.push(v3(sp.x, 0.10, sp.z));
}

/// The machine as modelled (domestic/kitchen_and_hearth_left_behind.py): the handwheel's axle, the bed's top,
/// the arm and pillar's front face
const SEW_AXLE: (f64, f64, f64) = (0.1405, 0.1960, -0.0310);

const SEW_TOP: f64 = 0.038;

const SEW_ARM_FRONT: f64 = -0.002;

fn sew_noise(seed: u32, x: f64, y: f64) -> f64 {
    let (xf, yf) = (x.floor(), y.floor());
    let (fx, fy) = (x - xf, y - yf);
    let (sx, sy) = (fx * fx * (3.0 - 2.0 * fx), fy * fy * (3.0 - 2.0 * fy));
    let (xi, yi) = (xf as i32, yf as i32);
    let (a, b) = (baking::hash(seed, xi, yi), baking::hash(seed, xi + 1, yi));
    let (c, d) = (baking::hash(seed, xi, yi + 1), baking::hash(seed, xi + 1, yi + 1));
    a + (b - a) * sx + (c - a) * sy + (a - b - c + d) * sx * sy
}

/// Black japanning, years on: its gloss dulled by dust in blotches, fine
/// scratches, chips to the grey metal at edges and knocks (one texture for
/// every face, whatever way it is mapped: nothing in it has a direction)
fn sew_japan_tex() -> Tex {
    thread_local! { static T: std::cell::OnceCell<Tex> = const { std::cell::OnceCell::new() }; }
    T.with(|c| {
        c.get_or_init(|| {
            canvas_tex_opt(256, 256, false, |ctx, w, h| {
                let n = w as usize;
                let mut img = ctx.create_image_data(w, h);
                par_rows(&mut img.data, n * 4, |j, out| {
                    for i in 0..n {
                        let (x, y) = (i as f64, j as f64);
                        let d = sew_noise(41, x / 24.0, y / 24.0) * 0.6 + sew_noise(42, x / 6.0, y / 6.0) * 0.4;
                        // the dust of years on the black, heavier in blotches
                        let g0 = 32.0 + 36.0 * (d - 0.4).max(0.0);
                        out[i * 4..i * 4 + 4].copy_from_slice(&[(g0 + 3.0) as u8, (g0 + 2.0) as u8, g0 as u8, 255]);
                    }
                });
                ctx.put_image_data(&img, 0, 0);
                ctx.set_line_width(0.6);
                for _ in 0..40 {
                    let (x, y) = (rnd() * w, rnd() * h);
                    let a = rnd() * PI;
                    let l = 3.0 + rnd() * 16.0;
                    ctx.set_stroke_style("rgba(74,72,68,0.35)");
                    ctx.begin_path();
                    ctx.move_to(x, y);
                    ctx.line_to(x + a.dcos() * l, y + a.dsin() * l);
                    ctx.stroke();
                }
                for _ in 0..10 {
                    let (x, y) = (rnd() * w, rnd() * h);
                    let r = 0.6 + rnd() * 1.4;
                    ctx.set_fill_style("rgba(96,94,90,0.8)");
                    ctx.begin_path();
                    ctx.ellipse(x, y, r, r * (0.5 + rnd() * 0.5), rnd() * PI, 0, PI * 2.0);
                    ctx.fill();
                }
            })
        })
        .clone()
    })
}

/// Nickel plate tarnished: dull, brown-grey in blotches, rust pricks
fn sew_bright_tex() -> Tex {
    thread_local! { static T: std::cell::OnceCell<Tex> = const { std::cell::OnceCell::new() }; }
    T.with(|c| {
        c.get_or_init(|| {
            canvas_tex_opt(128, 128, false, |ctx, w, h| {
                let n = w as usize;
                let mut img = ctx.create_image_data(w, h);
                par_rows(&mut img.data, n * 4, |j, out| {
                    for i in 0..n {
                        let (x, y) = (i as f64, j as f64);
                        let t = sew_noise(51, x / 14.0, y / 14.0);
                        let p = (sew_noise(52, x / 1.8, y / 1.8) - 0.82).max(0.0) * 5.0;
                        let g0 = 150.0 - 40.0 * t;
                        let c = [g0 + 4.0 * t - 60.0 * p, g0 - 2.0 * t - 80.0 * p, g0 - 10.0 * t - 96.0 * p];
                        out[i * 4..i * 4 + 4].copy_from_slice(&[c[0].max(0.0) as u8, c[1].max(0.0) as u8, c[2].max(0.0) as u8, 255]);
                    }
                });
                ctx.put_image_data(&img, 0, 0);
            })
        })
        .clone()
    })
}

/// Gold leaf rubbed thin: where a decal's gold is worn through to the black
/// (alpha), crazed, more worn where hands and cloth went
fn sew_rub(ctx: &mut Canvas, w: f64, h: f64, seed: u32, wear: f64) {
    let n = w as usize;
    let mut img = ctx.get_image_data(0, 0, w, h);
    for j in 0..h as usize {
        for i in 0..n {
            let a = img.data[(j * n + i) * 4 + 3] as f64;
            if a == 0.0 {
                continue;
            }
            let (x, y) = (i as f64, j as f64);
            let worn = sew_noise(seed, x / 9.0, y / 9.0) * 0.7 + sew_noise(seed ^ 1, x / 2.0, y / 2.0) * 0.3;
            let craze = (1.0 - ((sew_noise(seed ^ 2, x / 3.0, y / 3.0) - 0.5).abs() * 18.0).min(1.0)) * 0.5;
            let keep = (1.0 - ((worn - (1.0 - wear)) * 3.0).max(0.0) - craze).clamp(0.0, 1.0);
            img.data[(j * n + i) * 4 + 3] = (a * keep) as u8;
        }
    }
    ctx.put_image_data(&img, 0, 0);
}

/// The maker's name in gold on the arm, between scrolls (no real maker)
fn sew_name_tex() -> Tex {
    thread_local! { static T: std::cell::OnceCell<Tex> = const { std::cell::OnceCell::new() }; }
    T.with(|c| {
        c.get_or_init(|| {
            canvas_tex_opt(512, 128, false, |ctx, w, h| {
                ctx.clear_rect(0, 0, w, h);
                let gold = "rgba(196,160,84,0.95)";
                ctx.set_fill_style(gold);
                ctx.set_stroke_style(gold);
                ctx.set_text_align("center");
                ctx.set_text_baseline("middle");
                ctx.set_font("italic bold 64px Baskerville");
                ctx.fill_text_max("Haldane", w / 2.0, h * 0.50, w * 0.56);
                // the scrolls either side: S-curves and leaves
                ctx.set_line_width(3.0);
                for side in [-1.0, 1.0] {
                    let x0 = w / 2.0 + side * w * 0.31;
                    ctx.begin_path();
                    ctx.move_to(x0, h * 0.5);
                    ctx.bezier_curve_to(x0 + side * 30.0, h * 0.15, x0 + side * 60.0, h * 0.85, x0 + side * 92.0, h * 0.45);
                    ctx.stroke();
                    for k in 0..4 {
                        let t = 0.2 + k as f64 * 0.2;
                        let (lx, ly) = (x0 + side * 92.0 * t, h * (0.5 + 0.18 * (t * 9.0).dsin()));
                        ctx.begin_path();
                        ctx.ellipse(lx, ly, 9.0, 3.5, side * 0.7 + t, 0, PI * 2.0);
                        ctx.fill();
                    }
                    ctx.begin_path();
                    ctx.arc(x0 + side * 92.0, h * 0.45, 5.0, 0, PI * 2.0);
                    ctx.fill();
                }
                // the thin rule under it
                ctx.set_line_width(2.0);
                ctx.begin_path();
                ctx.move_to(w * 0.22, h * 0.86);
                ctx.line_to(w * 0.78, h * 0.86);
                ctx.stroke();
                sew_rub(ctx, w, h, 61, 0.45);
            })
        })
        .clone()
    })
}

/// A scroll on the bed's corner: a border of curls and leaves (drawn for the
/// right corner; the left is its mirror)
fn sew_scroll_tex() -> Tex {
    thread_local! { static T: std::cell::OnceCell<Tex> = const { std::cell::OnceCell::new() }; }
    T.with(|c| {
        c.get_or_init(|| {
            canvas_tex_opt(256, 112, false, |ctx, w, h| {
                ctx.clear_rect(0, 0, w, h);
                let gold = "rgba(196,160,84,0.9)";
                ctx.set_fill_style(gold);
                ctx.set_stroke_style(gold);
                ctx.set_line_width(3.0);
                ctx.begin_path();
                ctx.move_to(8.0, h * 0.7);
                for k in 0..5 {
                    let x = 8.0 + k as f64 * 46.0;
                    ctx.bezier_curve_to(x + 12.0, h * 0.2, x + 34.0, h * 0.2, x + 46.0, h * 0.7);
                }
                ctx.stroke();
                for k in 0..5 {
                    let x = 31.0 + k as f64 * 46.0;
                    ctx.begin_path();
                    ctx.arc(x, h * 0.55, 7.0, 0, PI * 1.6);
                    ctx.stroke();
                    ctx.begin_path();
                    ctx.ellipse(x + 10.0, h * 0.82, 8.0, 3.0, 0.5, 0, PI * 2.0);
                    ctx.fill();
                }
                ctx.set_line_width(2.0);
                ctx.begin_path();
                ctx.move_to(4.0, h * 0.93);
                ctx.line_to(w - 4.0, h * 0.93);
                ctx.stroke();
                sew_rub(ctx, w, h, 62, 0.6);
            })
        })
        .clone()
    })
}

/// The badge on the pillar: a round enamel plate, a gilt ring, a monogram
fn sew_badge_tex() -> Tex {
    thread_local! { static T: std::cell::OnceCell<Tex> = const { std::cell::OnceCell::new() }; }
    T.with(|c| {
        c.get_or_init(|| {
            canvas_tex_opt(128, 128, false, |ctx, w, h| {
                ctx.clear_rect(0, 0, w, h);
                let (cx, cy) = (w / 2.0, h / 2.0);
                ctx.set_fill_style("rgba(170,140,78,1)");
                ctx.begin_path();
                ctx.arc(cx, cy, 60.0, 0, PI * 2.0);
                ctx.fill();
                ctx.set_fill_style("rgba(92,30,30,1)");
                ctx.begin_path();
                ctx.arc(cx, cy, 50.0, 0, PI * 2.0);
                ctx.fill();
                ctx.set_fill_style("rgba(196,164,92,1)");
                ctx.set_text_align("center");
                ctx.set_text_baseline("middle");
                ctx.set_font("bold 60px Baskerville");
                ctx.fill_text("H", cx, cy + 3.0);
                ctx.set_font("bold 11px Helvetica");
                ctx.fill_text("TRADE MARK", cx, cy + 38.0);
                sew_rub(ctx, w, h, 63, 0.3);
            })
        })
        .clone()
    })
}

/// Thread wound on a spool: fine turns round it, a little dust on the top
fn sew_thread_tex() -> Tex {
    thread_local! { static T: std::cell::OnceCell<Tex> = const { std::cell::OnceCell::new() }; }
    T.with(|c| {
        c.get_or_init(|| {
            canvas_tex_opt(64, 64, false, |ctx, w, h| {
                let n = w as usize;
                let mut img = ctx.create_image_data(w, h);
                par_rows(&mut img.data, n * 4, |j, out| {
                    for i in 0..n {
                        let t = ((j as f64 * 1.7 + i as f64 * 0.05).dsin() * 0.5 + 0.5) * 0.25 + 0.75;
                        let v = (255.0 * t) as u8;
                        out[i * 4..i * 4 + 4].copy_from_slice(&[v, v, v, 255]);
                    }
                });
                ctx.put_image_data(&img, 0, 0);
            })
        })
        .clone()
    })
}
