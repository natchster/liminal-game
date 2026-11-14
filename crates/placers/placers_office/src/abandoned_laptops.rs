//! `place_abandoned_laptops`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Abandoned laptop: 12% chance per level, 1 laptop on the floor.
pub(crate) fn place_abandoned_laptops(g: &mut Game, _dist: &[i32]) {
    g.pp.laptop_positions.clear(); // reset per level
    if rnd() < 0.88 {
        return;
    } // 12% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let pk = pick(&open);
    let body_mat = lambert!({ color: 0x1a1a1e });
    let logo_mat = basic_mat!({ color: 0x4a4a52 });
    let laptop = group();
    let base = mesh(pg("AbandonedLaptops__base", box_geo!(0.30, 0.02, 0.22)), &body_mat);
    base.position().y = 0.01;
    laptop.add(&base);
    // 8% chance the laptop is open with a faint blue screen glow.
    let is_open = rnd() < 0.08;
    if is_open {
        // Keyboard overlay on top face of base.
        let keyboard_mat = lambert!({ map: laptop_keyboard_texture() });
        let keyboard = mesh(plane_geo!(0.28, 0.20), &keyboard_mat);
        keyboard.rotation().x = -PI / 2.0;
        keyboard.position().set(0, 0.021, 0);
        laptop.add(&keyboard);
        // Lid rotated up ~105° from closed position; hinged at back.
        let lid_group = group();
        let lid_open = mesh(pg("AbandonedLaptops__lid", box_geo!(0.30, 0.015, 0.22)), &body_mat);
        // Lid modelled closed, lying flat forward of the hinge (z 0…0.22).
        lid_open.position().set(0, 0.0075, 0.11);
        lid_group.add(&lid_open);
        // 50% chance a garbled-text canvas variant, else faint blue.
        let use_garbled = rnd() < 0.50;
        let screen_mat = if use_garbled {
            let texs = laptop_screen_textures();
            let g_tex = texs[(rnd() * texs.len() as f64).floor() as usize].clone();
            basic_mat!({ map: g_tex })
        } else {
            basic_mat!({ color: 0x2a4890 })
        };
        let screen = mesh(plane_geo!(0.26, 0.18), &screen_mat);
        // on the lid's inner face (its underside while closed), parallel to the slab
        screen.rotation().x = PI / 2.0;
        screen.position().set(0, -0.001, 0.11);
        lid_group.add(&screen);
        // 30% of garbled screens: brief periodic tint flicker.
        if use_garbled && rnd() < 0.30 {
            let flicker_phase = rnd() * PI * 2.0;
            let flicker_period = 1.2 + rnd() * 1.8; // 1.2–3.0 s
            let sm = screen_mat.clone();
            g.animate(move |_g, _dt, t| {
                let phase = (t / flicker_period) * PI * 2.0 + flicker_phase;
                let raw = phase.cos();
                let dip = if raw > 0.85 { (raw - 0.85) / 0.15 } else { 0.0 };
                let bright = 1.0 - dip * 0.55; // 100% down to 45% during dips
                sm.m().color.set_rgb(bright, bright, bright);
            });
        }
        lid_group.position().set(0, 0.02, -0.10); // hinge at back edge
        lid_group.rotation().x = -PI / 1.7; // ~105° open
        laptop.add(&lid_group);
        // Point light with slight blue tint from screen forward.
        let base_int = 0.35;
        let screen_light = point_light!(0x4060b0, base_int, 1.5, 2);
        screen_light.position().set(0, 0.15, 0.10);
        laptop.add(&screen_light);
        // 30% chance screen flicker — subtle intensity dip pattern.
        if rnd() < 0.30 {
            let mut flicker_t = 2.0 + rnd() * 4.0;
            let mut dip_t = 0.0;
            let sl = screen_light.clone();
            g.animate(move |_g, dt, _t| {
                if dip_t > 0.0 {
                    dip_t -= dt;
                    sl.set_intensity(if dip_t > 0.0 { base_int * 0.30 } else { base_int });
                } else {
                    flicker_t -= dt;
                    if flicker_t <= 0.0 {
                        dip_t = 0.05; // 50 ms flicker dip
                        flicker_t = 2.0 + rnd() * 4.0;
                    }
                }
            });
        }
    } else {
        // Closed variant: the stacked lid.
        let lid = mesh(pg("AbandonedLaptops__lid", box_geo!(0.30, 0.015, 0.22)), &body_mat);
        lid.position().y = 0.028;
        laptop.add(&lid);
        let logo = mesh(plane_geo!(0.04, 0.04), &logo_mat);
        logo.rotation().x = -PI / 2.0;
        logo.position().set(0, 0.036, 0);
        laptop.add(&logo);
    }
    // Corner bias (near walls) for realistic "set down against wall" placement.
    let wall_bias = rnd() < 0.60;
    let ox = if wall_bias { (if rnd() < 0.5 { -1.0 } else { 1.0 }) * (1.0 + rnd() * 0.5) } else { (rnd() - 0.5) * 2.2 };
    let oz = if wall_bias { (if rnd() < 0.5 { -1.0 } else { 1.0 }) * (1.0 + rnd() * 0.5) } else { (rnd() - 0.5) * 2.2 };
    laptop.position().x = g.wx(pk.0) + ox;
    laptop.position().z = g.wz(pk.1) + oz;
    laptop.rotation().y = rnd() * PI * 2.0;
    g.world.add(&laptop);
    // Track laptop position for extension cord co-loc.
    let lp = laptop.pos();
    g.pp.laptop_positions.push(v3(lp.x, 0.02, lp.z));
    // 20% chance a briefcase spawns adjacent to the laptop.
    if rnd() < 0.20 {
        let leather_mat = lambert!({ color: 0x1e1610 });
        let clip_mat = lambert!({ color: 0x8a7a48 });
        let handle_mat = lambert!({ color: 0x281a12 });
        let bc = group();
        let bc_body = mesh(pg("Briefcases__body", box_geo!(0.36, 0.10, 0.28)), &leather_mat);
        bc_body.position().y = 0.05;
        bc.add(&bc_body);
        let bc_seam = mesh(pg("Briefcases__rim", box_geo!(0.36, 0.008, 0.28)), &handle_mat);
        bc_seam.position().y = 0.10;
        bc.add(&bc_seam);
        for cs in [-1.0, 1.0] {
            let clasp = mesh(pg("Briefcases__clasp", box_geo!(0.03, 0.01, 0.02)), &clip_mat);
            clasp.position().set(cs * 0.10, 0.106, 0.14);
            bc.add(&clasp);
        }
        let bc_handle_arch = mesh(pg("Briefcases__handle", torus_geo!(0.05, 0.005, 4, 12, PI)), &handle_mat);
        bc_handle_arch.rotation().x = -PI / 2.0;
        bc_handle_arch.position().set(0, 0.109, 0); // lying on the rim's top (y 0.104)
        bc.add(&bc_handle_arch);
        // Position 0.35-0.55 m from laptop.
        let bc_ang = rnd() * PI * 2.0;
        let bc_dist = 0.35 + rnd() * 0.20;
        bc.position().set(lp.x + bc_ang.cos() * bc_dist, 0, lp.z + bc_ang.sin() * bc_dist);
        bc.rotation().y = rnd() * PI * 2.0;
        g.world.add(&bc);
    }
    // 25% chance a coffee cup on floor near laptop.
    if rnd() < 0.25 {
        let vc_mat = lambert!({ color: 0xece0d0 });
        let vcup = group();
        let vcup_body = mesh(pg("AbandonedLaptops__cup", cyl_geo!(0.038, 0.028, 0.09, 10)), &vc_mat);
        vcup_body.position().y = 0.045;
        vcup.add(&vcup_body);
        let vcoffee = mesh(circle_geo!(0.032, 10), lambert!({ color: 0x2a1810 }));
        vcoffee.rotation().x = -PI / 2.0;
        vcoffee.position().y = 0.091;
        vcup.add(&vcoffee);
        let cup_ang = rnd() * PI * 2.0;
        let cup_dist = 0.30 + rnd() * 0.25;
        vcup.position().set(lp.x + cup_ang.cos() * cup_dist, 0, lp.z + cup_ang.sin() * cup_dist);
        vcup.rotation().y = rnd() * PI * 2.0;
        g.world.add(&vcup);
    }
}

/// Laptop screen textures (4 variants: 3 garbled tints + BSOD)
fn laptop_screen_textures() -> Vec<Tex> {
    thread_local! { static V: std::cell::OnceCell<Vec<Tex>> = const { std::cell::OnceCell::new() }; }
    V.with(|c| {
        c.get_or_init(|| {
            with_native(|| {
                let glyphs: Vec<char> = "!@#$%^&*()_+-=[]{}|;:,.<>?/\\~`0123456789ABCDEF#*.".chars().collect();
                // (bg, fg, line, bsod)
                let variants: [(&str, &str, &str, bool); 4] = [
                    ("#0a1830", "#a0c8f0", "rgba(160,200,240,0.10)", false),
                    ("#1a1040", "#c0a0f0", "rgba(190,160,240,0.10)", false),
                    ("#301820", "#f0a0a0", "rgba(240,160,160,0.10)", false),
                    // BSOD variant — classic blue with :( face, error, fake trace.
                    ("#183090", "#f0f0f0", "rgba(255,255,255,0.05)", true),
                ];
                variants
                    .iter()
                    .map(|&(bg, fg, line, bsod)| {
                        canvas_tex_opt(128, 96, false, |ctx, w, h| {
                            ctx.set_fill_style(bg);
                            ctx.fill_rect(0, 0, w, h);
                            // Horizontal scan lines.
                            ctx.set_fill_style(line);
                            let mut y = 0.0;
                            while y < h {
                                ctx.fill_rect(0, y, w, 1);
                                y += 2.0;
                            }
                            if bsod {
                                // BSOD: sad face + error code + fake stack lines.
                                ctx.set_fill_style(fg);
                                ctx.set_font("bold 20px sans-serif");
                                ctx.set_text_align("left");
                                ctx.set_text_baseline("top");
                                ctx.fill_text(":(", 8, 4);
                                ctx.set_font("7px monospace");
                                ctx.fill_text("SYSTEM_FAILURE", 8, 32);
                                ctx.fill_text("0x000000C4", 8, 42);
                                let trace_lines = ["ntoskrnl.exe+0x1c", "kernel32+0x28", "user32.dll+0x40", "hal.dll+0x0c"];
                                for (i, tl) in trace_lines.iter().enumerate() {
                                    ctx.fill_text(tl, 8, 56.0 + i as f64 * 10.0);
                                }
                            } else {
                                // Random glyphs in a rough grid.
                                ctx.set_fill_style(fg);
                                ctx.set_font("9px monospace");
                                ctx.set_text_baseline("top");
                                let mut y = 4.0;
                                while y < h - 6.0 {
                                    let mut x = 4.0;
                                    while x < w - 6.0 {
                                        if rnd() < 0.55 {
                                            let gch = glyphs[(rnd() * glyphs.len() as f64).floor() as usize];
                                            ctx.set_global_alpha(0.5 + rnd() * 0.5);
                                            ctx.fill_text(&gch.to_string(), x, y);
                                        }
                                        x += 8.0;
                                    }
                                    y += 10.0;
                                }
                                ctx.set_global_alpha(1);
                                // A few brighter runs (dense clusters) — reads as "corrupted output".
                                for _i in 0..3 {
                                    let cy = 8.0 + rnd() * (h - 20.0);
                                    let cx = 4.0 + rnd() * (w - 40.0);
                                    ctx.set_fill_style(fg);
                                    // the loop bound draws a fresh `rnd()` every iteration
                                    let mut gi = 0;
                                    while (gi as f64) < 4.0 + rnd() * 4.0 {
                                        let gch = glyphs[(rnd() * glyphs.len() as f64).floor() as usize];
                                        ctx.fill_text(&gch.to_string(), cx + gi as f64 * 7.0, cy);
                                        gi += 1;
                                    }
                                }
                            }
                        })
                    })
                    .collect()
            })
        })
        .clone()
    })
}

// Laptop keyboard canvas texture.
game::lazy_static_tex!(
    laptop_keyboard_texture,
    canvas_tex_opt(256, 128, false, |ctx, w, h| {
        ctx.set_fill_style("#151518");
        ctx.fill_rect(0, 0, w, h);
        let (cols, rows) = (13.0, 5.0);
        let (key_w, key_h) = (15.0, 15.0);
        let margin_x = (w - cols * key_w - (cols - 1.0) * 3.0) / 2.0;
        let margin_y = (h - rows * key_h - (rows - 1.0) * 3.0) / 2.0;
        ctx.set_fill_style("#2a2a2e");
        for r in 0..5 {
            for c in 0..13 {
                let x = margin_x + c as f64 * (key_w + 3.0);
                let y = margin_y + r as f64 * (key_h + 3.0);
                ctx.fill_rect(x, y, key_w, key_h);
                ctx.set_fill_style("#3a3a3e");
                ctx.fill_rect(x + 1.0, y + 1.0, key_w - 2.0, key_h - 2.0);
                ctx.set_fill_style("#2a2a2e");
            }
        }
        // Trackpad below keyboard.
        let (tp_w, tp_h) = (90.0, 30.0);
        let tp_x = (w - tp_w) / 2.0;
        let tp_y = h - tp_h - 5.0;
        ctx.set_fill_style("#1a1a1e");
        ctx.fill_rect(tp_x, tp_y, tp_w, tp_h);
        ctx.set_stroke_style("#2a2a2e");
        ctx.set_line_width(1);
        ctx.stroke_rect(tp_x, tp_y, tp_w, tp_h);
    })
);
