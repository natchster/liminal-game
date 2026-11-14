//! `place_wall_phones`
use game::prelude::*;
use placers_kit::place::pick;
use placers_kit::place::wall_candidates;

// Wall phone: 15% chance per level, 1 unit. The office wall set of the
// 2554 class (Blender batch `r12`): a moulded housing on its wall plate, the
// handset hung down its front on the chrome hanger, its coiled cord looping
// under it to the jack; off the hook the handset hangs on its drawn-out cord;
// broken, the housing cracked, the cord cut short, and the burnt board's smoke.
pub(crate) fn place_wall_phones(g: &mut Game, _dist: &[i32]) {
    g.pp.phone_positions.clear(); // reset per level
    g.pp.broken_phone_positions.clear(); // reset per level
    let chance = rnd();
    if chance < 0.85 {
        return;
    } // 15% chance
    let cands = wall_candidates(g);
    if cands.is_empty() {
        return;
    }
    let c = pick(&cands);
    // the set's colour, from what is left of the chance draw (0.85..1): the
    // beige most offices had, the ash grey, black, now and then brown
    let u = (chance - 0.85) / 0.15;
    let shell = if u < 0.45 {
        0xb3a68a
    } else if u < 0.72 {
        0x9b978c
    } else if u < 0.9 {
        0x2e2c2a
    } else {
        0x5c4636
    };
    let phone = group();
    // the parts are modelled with the wall at z = 0: the housing's back on it
    let o = group();
    o.position().z = -0.03;
    phone.add(&o);
    let keys = mesh(wp_part("Real__wallPhoneKeys", 0.05, 0.07, 0.006), lambert!({ color: 0x3b3b3d }));
    o.add(&keys);
    let legends = mesh(
        plane_geo!(0.056, 0.074),
        lambert!({ map: &wp_keys_tex(), transparent: true, depthWrite: false, polygonOffset: true, polygonOffsetFactor: -2, polygonOffsetUnits: -2 }),
    );
    legends.position().set(-0.0125, 0.010, WP_D + 0.0015);
    o.add(&legends);
    let card = mesh(plane_geo!(0.040, 0.011), lambert!({ map: &wp_card_tex() }));
    card.position().set(-0.006, -0.047, WP_D - 0.0017);
    o.add(&card);
    let hook = mesh(wp_part("Real__wallPhoneHook", 0.032, 0.03, 0.02), phong!({ color: 0x575752, specular: 0x3a3a3a, shininess: 22 }));
    o.add(&hook);
    // 5% chance: broken phone with a cracked housing + no handset.
    let is_broken = rnd() < 0.05;
    let mut cracks = [(0.0, 0.0, 0.0, 0.0); 3];
    if is_broken {
        // the old crack planes' draws, now where the housing split: the
        // crack's length, its turn and its place on the face
        for ck in cracks.iter_mut() {
            let w = 0.001 + rnd() * 0.002;
            let turn = rnd() * PI;
            let x = (rnd() - 0.5) * 0.08;
            let y = -0.02 + (rnd() - 0.5) * 0.06;
            *ck = (w, turn, x, y);
        }
    }
    let housing_map = if is_broken { wp_broken_tex(cracks) } else { wp_plastic_tex() };
    let housing = mesh(wp_part("Real__wallPhoneHousing", WP_W, 0.232, WP_D), lambert!({ map: &housing_map, color: shell }));
    o.add(&housing);
    if is_broken {
        // 40% chance: smoke from the burnt board, leaking at the top.
        if rnd() < 0.40 {
            // Per-weather smoke color.
            let smoke_color = smoke_wisp_by_weather(g.weather_key());
            // lit by the room (lambert), so paler than its tint: smoke scatters the light
            let pale = |c: u32, k: u32| ((((c >> k) & 0xff) + 0xd8) / 2) << k;
            let smoke_color = pale(smoke_color, 16) | pale(smoke_color, 8) | pale(smoke_color, 0);
            let smoke_mat = lambert!({ map: &wp_smoke_tex(), color: smoke_color, transparent: true, opacity: 0.6, depthWrite: false, side: DoubleSide });
            // two soft sheets crossed at 45 degrees to the wall, so neither is
            // ever seen edge on from the room
            let wisp = group();
            for turn in [PI / 4.0, -PI / 4.0] {
                let sheet = mesh(plane_geo!(0.09, 0.14), &smoke_mat);
                sheet.rotation().y = turn;
                sheet.position().y = 0.07;
                wisp.add(&sheet);
            }
            let base_y = 0.10 + rnd() * 0.03;
            wisp.position().set((rnd() - 0.5) * 0.04, base_y, 0.02);
            wisp.rotation().z = (rnd() - 0.5) * 0.3;
            phone.add(&wisp);
            // Rise animator — wisp scales up + fades over a 6-10 s cycle.
            let wisp_phase = rnd() * PI * 2.0;
            let wisp_period = 6.0 + rnd() * 4.0; // 6-10 s per rise cycle
            let wisp_base_op = smoke_mat.opacity_v();
            let w2 = wisp.clone();
            let sm = smoke_mat.clone();
            g.animate(move |_g, _dt, t| {
                // Sawtooth: 0 → 1 → 0 → 1 ... over wisp_period.
                let frac = ((t + wisp_phase) / wisp_period) % 1.0;
                let s = 1.0 + frac * 0.6;
                w2.scale().set(s, 1.0 + frac * 0.4, s);
                w2.position().y = base_y + frac * 0.04;
                sm.m().opacity = wisp_base_op * (1.0 - frac);
            });
        }
    }
    // 25% chance handset off-hook; broken phones have no handset at all.
    let off_hook = !is_broken && rnd() < 0.25;
    let skip_handset = is_broken;
    let cord_mat = lambert!({ map: &wp_handset_tex(), color: shell });
    if skip_handset {
        // no handset: the cord cut off a hand's width under the jack
        o.add(&mesh(wp_part("Real__wallPhoneCordCut", 0.02, 0.1, 0.02), &cord_mat));
    } else {
        let handset = mesh(wp_part("Real__wallPhoneHandset", 0.06, 0.21, 0.04), lambert!({ map: &wp_handset_tex(), color: shell }));
        if off_hook {
            // hanging upside down from its drawn-out cord, turned a little
            handset.position().set(0.030, -0.565, 0.012);
            handset.rotation().set(0, 0.30, PI);
            o.add(&mesh(wp_part("Real__wallPhoneCordOff", 0.02, 0.45, 0.02), &cord_mat));
        } else {
            // hung on the hook, the cord looping under the phone
            handset.position().set(-0.026, 0.001, WP_D - 0.004);
            o.add(&mesh(wp_part("Real__wallPhoneCordOn", 0.07, 0.2, 0.05), &cord_mat));
        }
        o.add(&handset);
    }
    let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.03); // body back (z −0.03) on the wall
    let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.03);
    let phone_height = 1.3 + rnd() * 0.2;
    if c.dx.abs() == 1 {
        phone.rotation().y = if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 };
        phone.position().set(wall_x, phone_height, wall_z + (rnd() - 0.5) * 1.0);
    } else {
        phone.rotation().y = if c.dz > 0 { PI } else { 0.0 };
        phone.position().set(wall_x + (rnd() - 0.5) * 1.0, phone_height, wall_z);
    }
    g.world.add(&phone);
    let pp = phone.pos();
    // Track for ring audio events — only on-hook phones can ring.
    if !off_hook {
        g.pp.phone_positions.push(v3(pp.x, phone_height, pp.z));
    }
    // Track broken phones separately for spark-crackle audio.
    if is_broken {
        g.pp.broken_phone_positions.push(v3(pp.x, phone_height, pp.z));
    }
}

/// Smoke wisp per weather
fn smoke_wisp_by_weather(key: &str) -> u32 {
    by_weather(key, 0x505860, 0x707068, 0x704040, 0x604838, 0x606058, 0x606058)
}

/// The housing's width and depth (Blender batch `r12`, `wallPhoneHousing`).
const WP_W: f64 = 0.123;

const WP_D: f64 = 0.058;

/// A Blender part of batch `r12` as modelled (a scale of one), or a box
/// standing in for it when the props are switched off.
fn wp_part(name: &str, w: f64, h: f64, d: f64) -> Geo {
    game::assets::props::prop_part_as_modelled(name).unwrap_or_else(|| box_geo!(w, h, d))
}

/// Moulded ABS after decades on an office wall, laid over the set's colour
/// by the housing's box projection (the whole canvas is its face): yellowed
/// from the top, grey with hands round the earpiece's seat and the keys, the
/// plate's edge darker where the wall's dust settled, scuffs and scratches.
fn paint_wp_plastic(ctx: &mut Canvas, w: f64, h: f64) {
    ctx.set_fill_style("#f4f1ea");
    ctx.fill_rect(0, 0, w, h);
    // the moulding's faint mottle
    for _ in 0..500 {
        let a = 0.025 + rnd() * 0.04;
        ctx.set_fill_style(format!("rgba({},{},{},{a})", 90 + (rnd() * 60.0) as i32, 82, 70));
        ctx.fill_rect(rnd() * w, rnd() * h, 1.0 + rnd() * 2.0, 1.0 + rnd() * 2.0);
    }
    // yellowed: the light and the smoke of the room, strongest at the top
    let gy = ctx.create_linear_gradient(0, 0, 0, h);
    gy.add_color_stop(0, "rgba(200,170,90,0.30)");
    gy.add_color_stop(0.6, "rgba(200,170,90,0.10)");
    gy.add_color_stop(1, "rgba(200,170,90,0.04)");
    ctx.set_fill_style(gy);
    ctx.fill_rect(0, 0, w, h);
    // hands: the earpiece's seat (top left), round the keys, the mouthpiece's rest
    for &(cx, cy, r, a) in &[(0.29, 0.16, 0.30, 0.30), (0.40, 0.45, 0.32, 0.20), (0.30, 0.83, 0.26, 0.22), (0.55, 0.30, 0.22, 0.14)] {
        ctx.save();
        ctx.translate(cx * w, cy * h);
        ctx.scale(1.0, 1.3);
        let gr = ctx.create_radial_gradient(0, 0, 0, 0, 0, r * w);
        gr.add_color_stop(0, &format!("rgba(60,48,36,{a})"));
        gr.add_color_stop(1, "rgba(60,48,36,0)");
        ctx.set_fill_style(gr);
        ctx.begin_path();
        ctx.arc(0, 0, r * w, 0, 7);
        ctx.fill();
        ctx.restore();
    }
    // dust on the top edge, grime along the foot
    let gd = ctx.create_linear_gradient(0, 0, 0, h);
    gd.add_color_stop(0, "rgba(120,110,95,0.30)");
    gd.add_color_stop(0.05, "rgba(120,110,95,0)");
    gd.add_color_stop(0.92, "rgba(50,42,34,0)");
    gd.add_color_stop(1, "rgba(50,42,34,0.30)");
    ctx.set_fill_style(gd);
    ctx.fill_rect(0, 0, w, h);
    // scuffs and fine scratches
    ctx.set_line_cap("round");
    for _ in 0..26 {
        let (x, y, a, l) = (rnd() * w, rnd() * h, rnd() * 6.3, 3.0 + rnd() * 16.0);
        ctx.set_line_width(0.5 + rnd() * 0.8);
        ctx.set_stroke_style(format!("rgba(70,60,50,{})", 0.10 + rnd() * 0.22));
        ctx.begin_path();
        ctx.move_to(x, y);
        ctx.line_to(x + a.cos() * l, y + a.sin() * l);
        ctx.stroke();
    }
    for _ in 0..8 {
        ctx.set_fill_style(format!("rgba(40,34,28,{})", 0.12 + rnd() * 0.15));
        ctx.begin_path();
        ctx.ellipse(rnd() * w, rnd() * h, 1.5 + rnd() * 4.0, 0.8 + rnd() * 1.5, rnd() * 3.0, 0, 7);
        ctx.fill();
    }
}

/// The housing's wear (shared by every set: its colour is the material's).
// the closure is the painter, and its type names the texture's stream
// (`kit::painter_seed`): handing the fn over instead would paint it anew
#[allow(clippy::redundant_closure)]
fn wp_plastic_tex() -> Tex {
    thread_local! { static T: std::cell::OnceCell<Tex> = const { std::cell::OnceCell::new() }; }
    T.with(|c| c.get_or_init(|| canvas_tex_opt(128, 240, false, |ctx, w, h| paint_wp_plastic(ctx, w, h))).clone())
}

/// A broken set's housing: the same wear, and the three cracks where it was
/// struck (length, turn and place from the old crack planes' draws), each a
/// dark split with the plastic stress-whitened along it.
fn wp_broken_tex(cracks: [(f64, f64, f64, f64); 3]) -> Tex {
    canvas_tex_opt(128, 240, false, move |ctx, w, h| {
        paint_wp_plastic(ctx, w, h);
        // the face's metres to the canvas: x over the housing's 0.123 m, y over 0.232 m
        let to = |x: f64, y: f64| ((x / 0.123 + 0.5) * w, (0.5 - y / 0.232) * h);
        for &(cw, turn, x, y) in &cracks {
            // onto the flat of the face, right of the keys, where a blow lands
            let (px, py) = to(0.034 + x * 0.3, y * 1.7 + 0.02);
            let len = (0.040 + (cw - 0.001) / 0.002 * 0.035) / 0.123 * w;
            let (dx, dy) = (turn.cos(), turn.sin());
            let mut pts = vec![];
            let n = 9;
            for k in 0..=n {
                let t = k as f64 / n as f64 - 0.5;
                let j = if k == 0 || k == n { 0.0 } else { (rnd() - 0.5) * 3.0 };
                pts.push((px + dx * t * len - dy * j, py + dy * t * len + dx * j));
            }
            // the whitened plastic either side, then the split
            for (lw, col) in [(3.2, "rgba(255,250,235,0.35)"), (1.1, "rgba(28,22,18,0.85)")] {
                ctx.set_line_width(lw);
                ctx.set_line_join("round");
                ctx.set_stroke_style(col);
                ctx.begin_path();
                ctx.move_to(pts[0].0, pts[0].1);
                for p in &pts[1..] {
                    ctx.line_to(p.0, p.1);
                }
                ctx.stroke();
            }
            // a branch off its middle
            let (bx, by) = pts[n / 2];
            let bt = turn + 0.7 + rnd() * 0.6;
            ctx.set_line_width(0.8);
            ctx.begin_path();
            ctx.move_to(bx, by);
            ctx.line_to(bx + bt.cos() * len * 0.3, by + bt.sin() * len * 0.3);
            ctx.stroke();
        }
    })
}

/// The handset's and the cord's wear: grimy where it is held, rubbed shiny
/// and dark at the grip, the cups' rims scuffed.
fn wp_handset_tex() -> Tex {
    thread_local! { static T: std::cell::OnceCell<Tex> = const { std::cell::OnceCell::new() }; }
    T.with(|c| {
        c.get_or_init(|| {
            canvas_tex_opt(64, 160, false, |ctx, w, h| {
                ctx.set_fill_style("#f2eee6");
                ctx.fill_rect(0, 0, w, h);
                for _ in 0..240 {
                    ctx.set_fill_style(format!("rgba(80,70,60,{})", 0.03 + rnd() * 0.05));
                    ctx.fill_rect(rnd() * w, rnd() * h, 1.0 + rnd() * 2.0, 1.0 + rnd() * 2.0);
                }
                let gr = ctx.create_linear_gradient(0, 0, 0, h);
                gr.add_color_stop(0, "rgba(60,48,36,0.12)");
                gr.add_color_stop(0.35, "rgba(60,48,36,0.32)");
                gr.add_color_stop(0.65, "rgba(60,48,36,0.32)");
                gr.add_color_stop(1, "rgba(60,48,36,0.14)");
                ctx.set_fill_style(gr);
                ctx.fill_rect(0, 0, w, h);
                for _ in 0..12 {
                    let (x, y, a, l) = (rnd() * w, rnd() * h, rnd() * 6.3, 3.0 + rnd() * 10.0);
                    ctx.set_line_width(0.6);
                    ctx.set_stroke_style(format!("rgba(60,50,40,{})", 0.15 + rnd() * 0.2));
                    ctx.begin_path();
                    ctx.move_to(x, y);
                    ctx.line_to(x + a.cos() * l, y + a.sin() * l);
                    ctx.stroke();
                }
            })
        })
        .clone()
    })
}

/// What is printed on the keys, white on their grey (as the 2500 sets'):
/// the letters over the numeral, OPER over the 0, the star and the square;
/// the most used rubbed thin. The canvas spans the twelve keys, 2 px a mm.
fn wp_keys_tex() -> Tex {
    thread_local! { static T: std::cell::OnceCell<Tex> = const { std::cell::OnceCell::new() }; }
    T.with(|c| {
        c.get_or_init(|| {
            canvas_tex_opt(112, 148, false, |ctx, w, h| {
                ctx.clear_rect(0, 0, w, h);
                let keys = [
                    ("1", ""),
                    ("2", "ABC"),
                    ("3", "DEF"),
                    ("4", "GHI"),
                    ("5", "JKL"),
                    ("6", "MNO"),
                    ("7", "PRS"),
                    ("8", "TUV"),
                    ("9", "WXY"),
                    ("*", ""),
                    ("0", "OPER"),
                    ("#", ""),
                ];
                ctx.set_text_align("center");
                ctx.set_text_baseline("alphabetic");
                for (i, (num, letters)) in keys.iter().enumerate() {
                    let (col, row) = ((i % 3) as f64, (i / 3) as f64);
                    // the key's middle: x -0.030 + col * 0.0175 over the canvas's -0.0405..0.0155
                    let cx = (0.0105 + col * 0.0175) * 2000.0;
                    let cy = (0.007 + row * 0.020) * 2000.0;
                    let worn = rnd();
                    ctx.set_fill_style(format!("rgba(238,236,228,{})", 0.95 - worn * 0.45));
                    if !letters.is_empty() {
                        ctx.set_font("bold 5px Helvetica");
                        ctx.fill_text(letters, cx, cy - 2.0);
                    }
                    let big = if *num == "*" || *num == "#" { "bold 11px Helvetica" } else { "bold 9px Helvetica" };
                    ctx.set_font(big);
                    ctx.fill_text(num, cx, cy + if letters.is_empty() { 5.0 } else { 7.0 });
                }
            })
        })
        .clone()
    })
}

/// The number card behind its window: a printed rule and the extension
/// written in by hand, the card yellowed (555-01xx, the numbers kept for fiction).
fn wp_card_tex() -> Tex {
    thread_local! { static T: std::cell::OnceCell<Tex> = const { std::cell::OnceCell::new() }; }
    T.with(|c| {
        c.get_or_init(|| {
            canvas_tex_opt(160, 44, false, |ctx, w, h| {
                ctx.set_fill_style("#d9d0b8");
                ctx.fill_rect(0, 0, w, h);
                ctx.set_stroke_style("rgba(120,40,30,0.55)");
                ctx.set_line_width(1);
                ctx.stroke_rect(3.5, 3.5, w - 7.0, h - 7.0);
                ctx.set_fill_style("rgba(30,30,60,0.85)");
                ctx.set_font("22px Marker Felt");
                ctx.set_text_align("center");
                ctx.set_text_baseline("middle");
                ctx.fill_text("555-0147", w / 2.0, h / 2.0 + 1.0);
                let gr = ctx.create_linear_gradient(0, 0, w, 0);
                gr.add_color_stop(0, "rgba(90,70,40,0.25)");
                gr.add_color_stop(0.5, "rgba(90,70,40,0)");
                gr.add_color_stop(1, "rgba(90,70,40,0.2)");
                ctx.set_fill_style(gr);
                ctx.fill_rect(0, 0, w, h);
            })
        })
        .clone()
    })
}

/// A wisp of smoke: soft, thicker at its foot, drawn out and breaking up.
fn wp_smoke_tex() -> Tex {
    thread_local! { static T: std::cell::OnceCell<Tex> = const { std::cell::OnceCell::new() }; }
    T.with(|c| {
        c.get_or_init(|| {
            canvas_tex_opt(64, 128, false, |ctx, w, h| {
                ctx.clear_rect(0, 0, w, h);
                for k in 0..40 {
                    let t = k as f64 / 39.0; // 0 at the foot, 1 at the top
                    let x = w / 2.0 + (t * 6.0 + rnd() * 0.8).sin() * w * 0.16 * t;
                    let y = h * (0.97 - t * 0.85);
                    let r = w * (0.14 + t * 0.26);
                    let a = 0.32 * (1.0 - t).powf(0.6);
                    ctx.save();
                    ctx.translate(x, y);
                    ctx.scale(1.0, 1.4);
                    let gr = ctx.create_radial_gradient(0, 0, 0, 0, 0, r);
                    gr.add_color_stop(0, &format!("rgba(255,255,255,{a})"));
                    gr.add_color_stop(1, "rgba(255,255,255,0)");
                    ctx.set_fill_style(gr);
                    ctx.begin_path();
                    ctx.arc(0, 0, r, 0, 7);
                    ctx.fill();
                    ctx.restore();
                }
            })
        })
        .clone()
    })
}
