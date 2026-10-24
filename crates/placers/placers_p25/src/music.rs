//! Music, child-music and party-decoration props (R430-R437).
use crate::{buf_geo, open_tiles, pick, set_rot_order};
use game::prelude::*;

// R437 music sheet paper texture: staff lines + notes on cream paper.
game::lazy_static_tex!(
    music_sheet_texture,
    canvas_tex_opt(256, 320, false, |ctx, w, h| {
        // Cream paper background.
        ctx.set_fill_style("#f0ece0");
        ctx.fill_rect(0, 0, w, h);
        // Slight age grime.
        for _ in 0..40 {
            ctx.set_fill_style(&format!("rgba(120,100,60,{})", 0.02 + rnd() * 0.05));
            let cx = rnd() * w;
            let cy = rnd() * h;
            let rx = 8.0 + rnd() * 24.0;
            ctx.begin_path();
            ctx.ellipse(cx, cy, rx, rx * 0.6, rnd() * PI, 0, PI * 2.0);
            ctx.fill();
        }
        // 4 sets of 5 staff lines.
        ctx.set_stroke_style("#301a10");
        ctx.set_line_width(2);
        for gi in 0..4 {
            let gy = 40.0 + gi as f64 * 72.0;
            for i in 0..5 {
                ctx.begin_path();
                ctx.move_to(16, gy + i as f64 * 8.0);
                ctx.line_to(w - 16.0, gy + i as f64 * 8.0);
                ctx.stroke();
            }
            // Random notes: 6-10 filled ovals along the staff.
            ctx.set_fill_style("#181008");
            let n_notes = 6 + (rnd() * 5.0).floor() as i32;
            for n in 0..n_notes {
                let nx = 30.0 + (n as f64 / n_notes as f64) * (w - 60.0) + rnd() * 8.0;
                let ny_off = (rnd() * 9.0).floor() * 4.0 - 4.0;
                let ny = gy + 16.0 + ny_off;
                ctx.begin_path();
                ctx.ellipse(nx, ny, 5, 3.5, -0.4, 0, PI * 2.0);
                ctx.fill();
                // Stem.
                ctx.set_stroke_style("#181008");
                ctx.set_line_width(1.5);
                ctx.begin_path();
                ctx.move_to(nx + 4.0, ny - 1.0);
                ctx.line_to(nx + 4.0, ny - 22.0);
                ctx.stroke();
            }
        }
    })
);

/// R437 music sheet paper: 10% chance per level. Canvas-textured
/// paper with staff lines + notes on floor. Child-music family.
pub fn place_music_sheets(g: &mut Game, _dist: &[i32]) {
    g.pp.paper_positions.clear(); // R487 reset per level (shared with journals)
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let sheet_mat = lambert!({ map: music_sheet_texture(), side: DoubleSide });
    let sheet = mesh(plane_geo!(0.20, 0.25), &sheet_mat);
    sheet.rotation().x = -PI / 2.0;
    sheet.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0.003, g.wz(z) + (rnd() - 0.5) * 2.4);
    sheet.rotation().z = rnd() * PI * 2.0;
    // 30% chance slightly curled (subtle Y tilt).
    if rnd() < 0.30 {
        sheet.rotation().x = -PI / 2.0 + (rnd() - 0.5) * 0.2;
    }
    g.world.add(&sheet);
    // R487 track for paper rustle audio.
    let p = sheet.pos();
    g.pp.paper_positions.push(v3(p.x, 0.01, p.z));
}

/// R437 small conductor's baton: 8% chance per level. Thin white
/// baton with cork tip. Music family (conducting).
pub fn place_conductor_batons(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let shaft_mat = lambert!({ color: 0xf0ece0 });
    let cork_mat = lambert!({ color: 0xb08050 });
    let baton = group();
    // Shaft: thin white tapered cylinder.
    let shaft = mesh(pg("ConductorBatons__shaft", cyl_geo!(0.002, 0.003, 0.30, 6)), &shaft_mat);
    shaft.rotation().z = PI / 2.0;
    shaft.position().set(0.15, 0.010, 0);
    baton.add(&shaft);
    // Cork handle at base.
    let cork = mesh(pg("ConductorBatons__cork", cyl_geo!(0.008, 0.008, 0.030, 8)), &cork_mat);
    cork.rotation().z = PI / 2.0;
    cork.position().set(0.015, 0.010, 0);
    baton.add(&cork);
    baton.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    baton.rotation().y = rnd() * PI * 2.0;
    g.world.add(&baton);
}

/// R436 small xylophone: 6% chance per level. Toy xylophone with
/// 7 rainbow bars on wood frame + beater. Child-music family.
pub fn place_small_xylophones(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let rainbow = [0xd82030, 0xe07020, 0xd8b820, 0x30a030, 0x2080c8, 0x4030c0, 0x8020a0];
    let frame_mat = lambert!({ color: 0x8a6540 });
    let beater_mat = lambert!({ color: 0x2a2418 });
    let xylo = group();
    let n_bars = 7;
    let frame_w = 0.24;
    let frame_d = 0.10;
    // Trapezoidal frame: 2 side rails.
    for i in 0..2 {
        let s = if i == 0 { -1.0 } else { 1.0 };
        let rail = mesh(pg("SmallXylophones__rail", box_geo!(frame_w, 0.010, 0.010)), &frame_mat);
        // converge toward the short bars so every bar end rests on a rail
        rail.position().set(0, 0.005, s * 0.032);
        rail.rotation().y = s * (0.003f64 / 0.035).atan();
        xylo.add(&rail);
    }
    // 7 bars descending in width (like real xylophone).
    for i in 0..n_bars {
        let bar_len = frame_d - 0.010 - i as f64 * 0.006;
        let bar = mesh(pg("SmallXylophones__bar", box_geo!(0.024, 0.006, bar_len)), lambert!({ color: rainbow[i] }));
        bar.position().set(-frame_w / 2.0 + 0.015 + i as f64 * ((frame_w - 0.03) / (n_bars - 1) as f64), 0.013, 0);
        xylo.add(&bar);
    }
    // Beater: wood shaft + ball head laid nearby.
    let shaft = mesh(pg("SmallXylophones__shaft", cyl_geo!(0.003, 0.003, 0.08, 5)), &frame_mat);
    shaft.rotation().z = PI / 2.0;
    shaft.position().set(0.06, 0.004, 0.09);
    xylo.add(&shaft);
    let head = mesh(pg("SmallXylophones__head", sphere_geo!(0.008, 8, 6)), &beater_mat);
    head.position().set(0.10, 0.008, 0.09);
    xylo.add(&head);
    xylo.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    xylo.rotation().y = rnd() * PI * 2.0;
    g.world.add(&xylo);
}

/// R436 small kazoo: 8% chance per level. Short cylinder with
/// membrane disc. Child-music family.
pub fn place_small_kazoos(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let kaz_colors = [0xd82060, 0x2050d8, 0xd8b820, 0x30a030, 0xa020a0];
    let body_mat = lambert!({ color: pick(&kaz_colors) });
    let membrane_mat = lambert!({ color: 0xe8dcc0 });
    let kazoo = group();
    // Body: horizontal tapered cylinder.
    let body = mesh(pg("SmallKazoos__body", cyl_geo!(0.014, 0.010, 0.09, 8)), &body_mat);
    body.rotation().z = PI / 2.0;
    body.position().set(0.045, 0.014, 0);
    kazoo.add(&body);
    // Membrane resonator: small cylinder rising from mid-top.
    let res_body = mesh(pg("SmallKazoos__resonator", cyl_geo!(0.010, 0.010, 0.012, 8)), &body_mat);
    res_body.position().set(0.045, 0.032, 0);
    kazoo.add(&res_body);
    // Membrane disc on top of resonator.
    let membrane = mesh(circle_geo!(0.008, 8), &membrane_mat);
    membrane.rotation().x = -PI / 2.0;
    membrane.position().set(0.045, 0.039, 0);
    kazoo.add(&membrane);
    kazoo.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    kazoo.rotation().y = rnd() * PI * 2.0;
    g.world.add(&kazoo);
}

/// R435 small recorder: 8% chance per level. Beige recorder flute
/// with mouthpiece + 5 finger holes. Child-music family.
pub fn place_small_recorders(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let body_colors = [0xf0e0c8, 0xe8d0a8, 0xd8b890, 0x201a12];
    let body_mat = lambert!({ color: pick(&body_colors) });
    let hole_mat = lambert!({ color: 0x100a08 });
    let recorder = group();
    // Body: horizontal cylinder.
    let body_len = 0.24;
    let body = mesh(pg("SmallRecorders__body", cyl_geo!(0.012, 0.012, body_len, 8)), &body_mat);
    body.rotation().z = PI / 2.0;
    body.position().set(body_len / 2.0, 0.015, 0);
    recorder.add(&body);
    // Mouthpiece: slight flare at near end.
    let mouth = mesh(pg("SmallRecorders__beak", cyl_geo!(0.014, 0.010, 0.020, 8)), &body_mat);
    mouth.rotation().z = PI / 2.0;
    mouth.position().set(-0.005, 0.015, 0);
    recorder.add(&mouth);
    // Bell flare at far end.
    let bell = mesh(pg("SmallRecorders__foot", cyl_geo!(0.016, 0.012, 0.020, 8)), &body_mat);
    bell.rotation().z = PI / 2.0;
    bell.position().set(body_len - 0.005, 0.015, 0);
    recorder.add(&bell);
    // 5 finger holes on top.
    for i in 0..5 {
        let hole = mesh(circle_geo!(0.003, 8), &hole_mat);
        hole.rotation().x = -PI / 2.0;
        hole.position().set(0.06 + i as f64 * 0.028, 0.0255, 0); // in the body's finger-hole recesses
        recorder.add(&hole);
    }
    recorder.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    recorder.rotation().y = rnd() * PI * 2.0;
    g.world.add(&recorder);
}

/// R435 small percussion triangle: 6% chance per level. Bent metal
/// triangle + beater rod. Child-music family.
pub fn place_small_triangles(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let metal_mat = lambert!({ color: 0xc0c8d0 });
    let wood_mat = lambert!({ color: 0x8a6540 });
    let tri = group();
    // Triangle is 3 metal rod segments forming triangle silhouette laid flat.
    let side = 0.10;
    let half_h = side * 3f64.sqrt() / 4.0;
    // (x, z) of each corner (y is 0)
    let vertices = [(-side / 2.0, -half_h), (side / 2.0, -half_h), (0.0, half_h)];
    for i in 0..3 {
        let (x0, z0) = vertices[i];
        let (x1, z1) = vertices[(i + 1) % 3];
        // Skip one side to leave the open-triangle gap.
        if i == 2 {
            continue;
        }
        let seg_len = hypot2(x1 - x0, z1 - z0);
        let seg = mesh(pg("SmallTriangles__rod", cyl_geo!(0.003, 0.003, seg_len, 5)), &metal_mat);
        seg.position().set((x0 + x1) / 2.0, 0.003, (z0 + z1) / 2.0);
        seg.rotation().x = PI / 2.0;
        seg.rotation().z = (-(x1 - x0)).atan2(z1 - z0); // rod axis along (dx, 0, dz)
        tri.add(&seg);
    }
    // Third side with a small gap
    let (x2a, z2a) = vertices[2];
    let (vx0, vz0) = vertices[0];
    let (dx, dz) = (vx0 - x2a, vz0 - z2a);
    let seg_len = hypot2(dx, dz) * 0.90;
    let seg3 = mesh(pg("SmallTriangles__rod", cyl_geo!(0.003, 0.003, seg_len, 5)), &metal_mat);
    seg3.position().set(x2a + dx * 0.45, 0.003, z2a + dz * 0.45);
    seg3.rotation().x = PI / 2.0;
    seg3.rotation().z = (-dx).atan2(dz); // rod axis along (dx, 0, dz)
    tri.add(&seg3);
    // Beater rod: thin wood cylinder laid nearby.
    let beater = mesh(pg("SmallTriangles__beater", cyl_geo!(0.003, 0.003, 0.08, 5)), &wood_mat);
    beater.rotation().z = PI / 2.0;
    beater.position().set(0.09, 0.003, 0.02);
    beater.rotation().y = rnd() * 0.6 - 0.3;
    tri.add(&beater);
    tri.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    tri.rotation().y = rnd() * PI * 2.0;
    g.world.add(&tri);
}

/// R434 paper garland: 7% chance per level. Chain of circular
/// paper cutouts strung on catenary between two anchor points.
pub fn place_paper_garlands(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.93 {
        return;
    } // 7% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let g_colors = [0xd82060, 0x2050d8, 0xd8b820, 0x30a030, 0xa020a0, 0xd050a0, 0xf0ece4];
    let string_mat = lambert!({ color: 0xf0e8d8 });
    let garland = group();
    let span_len = 1.4 + rnd() * 1.0;
    let sag = 0.20;
    let anchor_y = WALL_H - 0.06;
    let n_dots = 10 + (rnd() * 6.0).floor() as i32;
    for i in 0..n_dots {
        let t_frac = (i as f64 + 0.5) / n_dots as f64;
        let x_pos = (t_frac - 0.5) * span_len;
        let sag_y = anchor_y - sag * (1.0 - ((t_frac - 0.5) * 2.0).powi(2));
        let cut_mat = lambert!({ color: pick(&g_colors), side: DoubleSide });
        // Alternate circle / square / diamond cutout shapes.
        let shape_choice = i % 3;
        let cut;
        if shape_choice == 0 {
            cut = mesh(circle_geo!(0.030, 12), &cut_mat);
        } else if shape_choice == 1 {
            cut = mesh(plane_geo!(0.048, 0.048), &cut_mat);
        } else {
            cut = mesh(plane_geo!(0.048, 0.048), &cut_mat);
            cut.rotation().z = PI / 4.0;
        }
        cut.position().set(x_pos, sag_y - 0.020, 0);
        garland.add(&cut);
    }
    // Catenary string segments between successive cutouts.
    // i = -1 and i = n_dots - 1 are the end strings, from the outer cutouts
    // up to the ceiling at the span ends, so the garland hangs from something.
    for i in -1..n_dots {
        let t0 = 0f64.max((i as f64 + 0.5) / n_dots as f64);
        let t1 = 1f64.min((i as f64 + 1.5) / n_dots as f64);
        let x0 = (t0 - 0.5) * span_len;
        let x1 = (t1 - 0.5) * span_len;
        let y0 = if t0 == 0.0 { WALL_H } else { anchor_y - sag * (1.0 - ((t0 - 0.5) * 2.0).powi(2)) };
        let y1 = if t1 == 1.0 { WALL_H } else { anchor_y - sag * (1.0 - ((t1 - 0.5) * 2.0).powi(2)) };
        let seg_len = hypot2(x1 - x0, y1 - y0);
        let seg = mesh(pg("PaperGarlands__string", cyl_geo!(0.0015, 0.0015, seg_len, 3)), &string_mat);
        seg.position().set((x0 + x1) / 2.0, (y0 + y1) / 2.0, 0);
        seg.rotation().z = -(x1 - x0).atan2(y1 - y0);
        garland.add(&seg);
    }
    garland.position().set(g.wx(x) + (rnd() - 0.5) * 1.5, 0, g.wz(z) + (rnd() - 0.5) * 1.5);
    garland.rotation().y = rnd() * PI * 2.0;
    g.world.add(&garland);
}

/// R434 small maracas: 8% chance per level. Pair of maracas
/// (bulb + handle) on floor. Latin-percussion child-music family.
pub fn place_small_maracas(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let bulb_colors = [0xd82060, 0x2050d8, 0xd8b820, 0x30a030, 0xa020a0];
    let handle_mat = lambert!({ color: 0x8a6540 });
    let bulb_color = pick(&bulb_colors);
    let bulb_mat = lambert!({ color: bulb_color });
    let pair = group();
    for i in 0..2 {
        let maraca = group();
        // Bulb: slightly oblate sphere. Upright = standing head-down on the bulb, neck up.
        let bulb = mesh(pg("SmallMaracas__bulb", sphere_geo!(0.028, 10, 8)), &bulb_mat);
        bulb.scale().y = 0.85;
        bulb.rotation().x = PI;
        bulb.position().y = 0.0238;
        maraca.add(&bulb);
        // Handle: rises out of the bulb's neck (collar 4 mm inside it), knob up.
        let handle = mesh(pg("SmallMaracas__handle", cyl_geo!(0.007, 0.006, 0.08, 6)), &handle_mat);
        handle.rotation().x = PI;
        handle.position().y = 0.0836;
        maraca.add(&handle);
        // 30% chance tipped on side: rests on the bulb and the handle's knob
        // end, so it lies tilted 0.22 rad.
        if rnd() < 0.30 {
            maraca.rotation().z = PI / 2.0 + 0.222;
            maraca.position().y = 0.0333;
        }
        // Position side by side.
        maraca.position().x = (if i == 0 { -0.030 } else { 0.030 }) + (rnd() - 0.5) * 0.02;
        maraca.position().z = (rnd() - 0.5) * 0.02;
        maraca.rotation().y = rnd() * PI * 2.0;
        pair.add(&maraca);
    }
    pair.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    pair.rotation().y = rnd() * PI * 2.0;
    g.world.add(&pair);
}

/// R433 small trumpet: 6% chance per level. Small brass toy
/// trumpet with bell + tube + 3 valves.
pub fn place_small_trumpets(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let brass_mat = lambert!({ color: 0xc89040 });
    let dark_mat = lambert!({ color: 0x2a2418 });
    let trumpet = group();
    // Main tube: horizontal.
    let tube = mesh(pg("SmallTrumpets__tube", cyl_geo!(0.010, 0.010, 0.18, 8)), &brass_mat);
    tube.rotation().z = PI / 2.0;
    tube.position().set(0.09, 0.020, 0);
    trumpet.add(&tube);
    // Bell: flared cone at far end.
    let bell = mesh(pg("SmallTrumpets__bell", cone_geo!(0.035, 0.06, 12, 1, true)), &brass_mat);
    bell.rotation().z = PI / 2.0; // narrow end on the tube, flaring out to +X
    bell.position().set(0.21, 0.020, 0);
    trumpet.add(&bell);
    // Mouthpiece: small cup at near end.
    let mouth = mesh(pg("SmallTrumpets__mouth", cyl_geo!(0.012, 0.010, 0.012, 8)), &brass_mat);
    mouth.rotation().z = PI / 2.0;
    mouth.position().set(-0.006, 0.020, 0);
    trumpet.add(&mouth);
    // 3 valve caps on top of tube.
    for i in 0..3 {
        let valve = mesh(pg("SmallTrumpets__valve", cyl_geo!(0.008, 0.008, 0.020, 6)), &dark_mat);
        valve.position().set(0.06 + i as f64 * 0.025, 0.038, 0);
        trumpet.add(&valve);
    }
    trumpet.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    trumpet.rotation().y = rnd() * PI * 2.0;
    // Tilt so it rests on the bell rim and the mouthpiece.
    trumpet.rotation().z = 0.023f64.atan2(0.252);
    trumpet.position().y = -0.0069;
    g.world.add(&trumpet);
}

/// R433 hanging balloon bundle: 6% chance per level. 3-4 balloons
/// on independent strings converging at ceiling anchor point.
pub fn place_hanging_balloon_bundles(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let bal_colors = [0xd82060, 0x2050d8, 0xd8b820, 0x30a030, 0xa020a0, 0xd050a0];
    let string_mat = lambert!({ color: 0xf0e8d8 });
    let bundle = group();
    let n_balloons = 3 + (rnd() * 2.0).floor() as i32;
    let anchor_y = WALL_H - 0.05;
    for i in 0..n_balloons {
        let angle = (i as f64 / n_balloons as f64) * PI * 2.0 + rnd() * 0.4;
        let spread = 0.15 + rnd() * 0.08;
        let balloon_y = WALL_H - (0.4 + rnd() * 0.4);
        let bx = angle.cos() * spread;
        let bz = angle.sin() * spread;
        // String from anchor to balloon tie (the knot on top of the neck-up balloon).
        let dx = bx - 0.0;
        let dy = balloon_y + 0.105 - anchor_y;
        let dz = bz - 0.0;
        let str_len = (dx * dx + dy * dy + dz * dz).sqrt();
        let strg = mesh(pg("HangingBalloonBundles__string", cyl_geo!(0.002, 0.002, str_len, 3)), &string_mat);
        strg.position().set(bx / 2.0, (anchor_y + balloon_y + 0.105) / 2.0, bz / 2.0);
        // Rotate cylinder from Y-up to string direction.
        let mut str_dir = v3(dx, dy, dz);
        str_dir.normalize();
        let y_up = v3(0, 1, 0);
        let mut q = Quat::IDENTITY;
        q.set_from_unit_vectors(&y_up, &str_dir);
        strg.quaternion().copy(&q);
        bundle.add(&strg);
        // Balloon body.
        let bal_mat = lambert!({ color: pick(&bal_colors) });
        let bal = mesh(pg("HangingBalloonBundles__balloon", sphere_geo!(0.09, 10, 8)), &bal_mat);
        bal.scale().y = 1.15;
        bal.rotation().z = PI; // neck up, hanging from the knot
        bal.position().set(bx, balloon_y, bz);
        bundle.add(&bal);
        // Balloon tie (small cone), on top of the balloon's neck.
        let tie = mesh(pg("HangingBalloonBundles__tie", cone_geo!(0.008, 0.014, 5)), &bal_mat);
        tie.position().set(bx, balloon_y + 0.100, bz);
        bundle.add(&tie);
    }
    bundle.position().set(g.wx(x) + (rnd() - 0.5) * 1.5, 0, g.wz(z) + (rnd() - 0.5) * 1.5);
    bundle.rotation().y = rnd() * PI * 2.0;
    g.world.add(&bundle);
    // Very subtle whole-bundle sway.
    if rnd() < 0.60 {
        let sway_phase = rnd() * PI * 2.0;
        let sway_period = 10.0 + rnd() * 8.0;
        let base_rot_z = bundle.rot().z;
        let b = bundle.clone();
        g.animate(move |_g, _dt, t| {
            b.rotation().z = base_rot_z + (t * (PI * 2.0 / sway_period) + sway_phase).sin() * 0.02;
        });
    }
}

/// R432 hanging streamers: 8% chance per level. 3-6 vertical
/// paper streamer ribbons hanging from ceiling with sway.
pub fn place_hanging_streamers(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let streamer_colors = [0xd82060, 0x2050d8, 0xd8b820, 0x30a030, 0xa020a0, 0xd050a0];
    let grp = group();
    let n_streamers = 3 + (rnd() * 4.0).floor() as i32;
    for _ in 0..n_streamers {
        let mat = lambert!({ color: pick(&streamer_colors), side: DoubleSide });
        let len = 0.9 + rnd() * 0.8;
        let strip = mesh(plane_geo!(0.020, len), &mat);
        strip.position().set((rnd() - 0.5) * 0.35, WALL_H - len / 2.0 - 0.02, (rnd() - 0.5) * 0.35);
        strip.rotation().y = rnd() * PI * 2.0;
        grp.add(&strip);
        // Subtle sway per strip.
        if rnd() < 0.50 {
            let sway_phase = rnd() * PI * 2.0;
            let sway_period = 6.0 + rnd() * 6.0;
            let base_rot_z = strip.rot().z;
            let s = strip.clone();
            g.animate(move |_g, _dt, t| {
                s.rotation().z = base_rot_z + (t * (PI * 2.0 / sway_period) + sway_phase).sin() * 0.04;
            });
        }
    }
    grp.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    grp.rotation().y = rnd() * PI * 2.0;
    g.world.add(&grp);
}

/// R432 small party banner: 6% chance per level. Small horizontal
/// paper banner strung between two ceiling anchor points with a
/// gentle catenary sag.
pub fn place_small_party_banners(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let ban_colors = [0xd82060, 0x2050d8, 0xd8b820, 0x30a030, 0xa020a0];
    let ban_mat = lambert!({ color: pick(&ban_colors), side: DoubleSide });
    let string_mat = lambert!({ color: 0xf0e8d8 });
    let banner = group();
    let span_len = 1.6 + rnd() * 0.8;
    let sag = 0.18;
    let anchor_y = WALL_H - 0.05;
    // 5 triangular pennants along the string.
    let n_flags = 5;
    for i in 0..n_flags {
        let t_frac = (i as f64 + 0.5) / n_flags as f64;
        let x_pos = (t_frac - 0.5) * span_len;
        // Catenary sag approximation.
        let sag_y = anchor_y - sag * (1.0 - ((t_frac - 0.5) * 2.0).powi(2));
        let fs = 0.055;
        let flag_geo = buf_geo(&[-fs / 2.0, 0.0, 0.0, fs / 2.0, 0.0, 0.0, 0.0, -fs * 1.4, 0.0], &[0, 1, 2]);
        let flag_mat = lambert!({ color: pick(&ban_colors), side: DoubleSide });
        let flag = mesh(flag_geo, &flag_mat);
        flag.position().set(x_pos, sag_y, 0);
        banner.add(&flag);
    }
    // String segments between flag tops.
    for i in 0..n_flags {
        let t0 = (i as f64 + 0.5) / n_flags as f64;
        let t1 = (i as f64 + 1.5) / n_flags as f64;
        if i == n_flags - 1 {
            break;
        }
        let x0 = (t0 - 0.5) * span_len;
        let x1 = (t1 - 0.5) * span_len;
        let y0 = anchor_y - sag * (1.0 - ((t0 - 0.5) * 2.0).powi(2));
        let y1 = anchor_y - sag * (1.0 - ((t1 - 0.5) * 2.0).powi(2));
        let seg_len = hypot2(x1 - x0, y1 - y0);
        let seg = mesh(pg("SmallPartyBanners__string", cyl_geo!(0.002, 0.002, seg_len, 3)), &string_mat);
        seg.position().set((x0 + x1) / 2.0, (y0 + y1) / 2.0, 0);
        seg.rotation().z = -(x1 - x0).atan2(y1 - y0);
        banner.add(&seg);
    }
    // End string segments to anchor points: tied into the ceiling, each runs
    // to the outer pennant's top.
    let end_anchors = [(-span_len / 2.0, WALL_H), (span_len / 2.0, WALL_H)];
    for &(ex, ey) in end_anchors.iter() {
        let nearest =
            if ex < 0.0 { -span_len / 2.0 + span_len / n_flags as f64 / 2.0 } else { span_len / 2.0 - span_len / n_flags as f64 / 2.0 };
        let nearest_y = anchor_y - sag * (1.0 - ((0.5 / n_flags as f64 - 0.5) * 2.0).powi(2));
        let dx = nearest - ex;
        let dy = nearest_y - ey;
        let seg_len = hypot2(dx, dy);
        if seg_len < 0.01 {
            continue;
        }
        let seg = mesh(pg("SmallPartyBanners__string", cyl_geo!(0.002, 0.002, seg_len, 3)), &string_mat);
        seg.position().set((nearest + ex) / 2.0, (nearest_y + ey) / 2.0, 0);
        seg.rotation().z = -dx.atan2(dy);
        banner.add(&seg);
    }
    banner.position().set(g.wx(x) + (rnd() - 0.5) * 1.5, 0, g.wz(z) + (rnd() - 0.5) * 1.5);
    banner.rotation().y = rnd() * PI * 2.0;
    g.world.add(&banner);
}

/// R431 candle cluster: 8% chance per level. 3-5 R428-style
/// birthday candles arranged in a small ring (cake without cake).
pub fn place_candle_clusters(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let wax_colors = [0xf0e8d8, 0xf0c0d0, 0xd8f0d0, 0xd0d8f0, 0xf0d8a0, 0xe0b0e0];
    let wick_mat = lambert!({ color: 0x2a2418 });
    let n_candles = 3 + (rnd() * 3.0).floor() as i32;
    let ring_radius = 0.04 + rnd() * 0.03;
    let cluster = group();
    for i in 0..n_candles {
        let angle = (i as f64 / n_candles as f64) * PI * 2.0 + rnd() * 0.3;
        let wax_mat = lambert!({ color: pick(&wax_colors) });
        let candle = group();
        let body = mesh(pg("CandleClusters__body", cyl_geo!(0.008, 0.008, 0.06, 8)), &wax_mat);
        body.position().y = 0.03;
        candle.add(&body);
        let wick = mesh(pg("CandleClusters__wick", cyl_geo!(0.001, 0.001, 0.008, 4)), &wick_mat);
        wick.position().y = 0.064;
        candle.add(&wick);
        // 70% lit for cluster (more festive than lone candle).
        if rnd() < 0.70 {
            let flame_mat = basic_mat!({ color: 0xffd070, transparent: true, opacity: 0.85 });
            let flame = mesh(cone_geo!(0.003, 0.010, 5), &flame_mat);
            flame.position().y = 0.074;
            candle.add(&flame);
            let flame_light = point_light!(0xffb060, 0.28, 0.7, 2);
            flame_light.position().y = 0.075;
            candle.add(&flame_light);
            // R483 warm halo sphere.
            let halo_mat = basic_mat!({ color: 0xffb060, transparent: true, opacity: 0.25, depthWrite: false });
            let halo = mesh(sphere_geo!(0.018, 6, 5), &halo_mat);
            halo.position().y = 0.076;
            candle.add(&halo);
            let flicker_phase = rnd() * PI * 2.0;
            let flicker_rate = 6.0 + rnd() * 4.0;
            g.animate(move |_g, _dt, t| {
                let p = t * flicker_rate + flicker_phase;
                let wobble = p.sin() * 0.5 + (p * 1.7).sin() * 0.3;
                flame_light.set_intensity(0.28 + wobble * 0.07);
                flame.scale().y = 1.0 + wobble * 0.14;
                halo.scale().set_scalar(1.0 + wobble * 0.15);
            });
        }
        candle.position().set(angle.cos() * ring_radius, 0, angle.sin() * ring_radius);
        cluster.add(&candle);
    }
    cluster.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    cluster.rotation().y = rnd() * PI * 2.0;
    g.world.add(&cluster);
}

/// R431 small piñata: 5% chance per level. Hanging star shape from
/// ceiling with string. 6-point paper-mache star silhouette.
pub fn place_small_pinatas(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let pin_colors = [0xd82060, 0x2050d8, 0xd8b820, 0x30a030, 0xa020a0, 0xd050a0];
    let pin_body = lambert!({ color: pick(&pin_colors) });
    let pin_accent = lambert!({ color: pick(&pin_colors) });
    let string_mat = lambert!({ color: 0x8a7050 });
    let pinata = group();
    // String from ceiling down to piñata top.
    let string_len = 0.4 + rnd() * 0.4;
    let string_y = WALL_H - string_len / 2.0 - 0.02;
    let strg = mesh(pg("SmallPinatas__string", cyl_geo!(0.003, 0.003, string_len, 4)), &string_mat);
    strg.position().y = string_y;
    pinata.add(&strg);
    let body_y = WALL_H - string_len - 0.08 - 0.02;
    // Core body: sphere.
    let core = mesh(pg("SmallPinatas__core", sphere_geo!(0.09, 8, 6)), &pin_body);
    core.position().y = body_y;
    pinata.add(&core);
    // 6 cone points around equator for star silhouette.
    for i in 0..6 {
        let angle = (i as f64 / 6.0) * PI * 2.0;
        let point = mesh(pg("SmallPinatas__point", cone_geo!(0.03, 0.08, 5)), &pin_accent);
        point.position().set(angle.cos() * 0.11, body_y, angle.sin() * 0.11);
        point.rotation().z = -PI / 2.0;
        point.rotation().y = -angle;
        pinata.add(&point);
    }
    // Top point and bottom point for depth.
    let top_pt = mesh(pg("SmallPinatas__point", cone_geo!(0.03, 0.06, 5)), &pin_accent);
    top_pt.position().y = body_y + 0.10;
    pinata.add(&top_pt);
    let bot_pt = mesh(pg("SmallPinatas__point", cone_geo!(0.03, 0.06, 5)), &pin_accent);
    bot_pt.position().y = body_y - 0.10;
    bot_pt.rotation().z = PI;
    pinata.add(&bot_pt);
    pinata.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    pinata.rotation().y = rnd() * PI * 2.0;
    g.world.add(&pinata);
    // Very subtle sway.
    if rnd() < 0.60 {
        let sway_phase = rnd() * PI * 2.0;
        let sway_period = 10.0 + rnd() * 8.0;
        let p = pinata.clone();
        g.animate(move |_g, _dt, t| {
            p.rotation().z = (t * (PI * 2.0 / sway_period) + sway_phase).sin() * 0.025;
        });
    }
}

/// R430 small paper cup: 12% chance per level. Tapered paper cup.
/// Party-remnant.
pub fn place_small_paper_cups(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.88 {
        return;
    } // 12% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let cup_colors = [0xf0ece4, 0xf0c0d0, 0xd0d8f0, 0xf0d8a0, 0xd8f0d0, 0xe8d8c8];
    let cup_mat = lambert!({ color: pick(&cup_colors), side: DoubleSide });
    let cup = group();
    // Body: tapered cylinder (wider at top).
    let body = mesh(cyl_geo!(0.035, 0.025, 0.08, 12, 1, true), &cup_mat);
    body.position().y = 0.04;
    cup.add(&body);
    // Bottom disc.
    let bottom = mesh(circle_geo!(0.025, 12), &cup_mat);
    bottom.rotation().x = -PI / 2.0;
    bottom.position().y = 0.001;
    cup.add(&bottom);
    // 25% chance tipped on side (crushed party cup vibe).
    if rnd() < 0.25 {
        cup.rotation().z = PI / 2.0;
        cup.position().y = 0.035;
    }
    cup.position().x = g.wx(x) + (rnd() - 0.5) * 2.4;
    cup.position().z = g.wz(z) + (rnd() - 0.5) * 2.4;
    cup.rotation().y = rnd() * PI * 2.0;
    g.world.add(&cup);
}
