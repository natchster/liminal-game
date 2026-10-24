//! p23, first half: reliquary / stained glass / crystal skull / portrait /
//! crucifix / owl / candlestick / skulls / potion / candy corn / treat pail.
use crate::*;
use game::prelude::*;

// R492 reliquary box: 4% chance per level. Small ornate wooden
// box with 4 gem accents + gold trim + optional gem glow.
pub fn place_reliquary_boxes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick_tile(&open);
    let box_mat = lambert!({ color: 0x603820 });
    let gold_mat = lambert!({ color: 0xd8b040 });
    let gem_colors: [u32; 4] = [0xd82040, 0x2080d8, 0x30c060, 0xa020d0];
    let relic = group();
    // Body: rectangular chest.
    let body = mesh(pg("ReliquaryBoxes__body", box_geo!(0.14, 0.06, 0.09)), &box_mat);
    body.position().y = 0.030;
    relic.add(&body);
    // Gold trim strips (top edge + bottom edge).
    for yy in [0.005, 0.055] {
        let trim = mesh(pg("ReliquaryBoxes__trim", box_geo!(0.144, 0.006, 0.094)), &gold_mat);
        trim.position().y = yy;
        relic.add(&trim);
    }
    // 4 corner gems on lid.
    for (gx, gz) in [(-1.0, -1.0), (-1.0, 1.0), (1.0, -1.0), (1.0, 1.0)] {
        let gem_color = pick_u32(&gem_colors);
        let gem_mat = basic_mat!({ color: gem_color, transparent: true, opacity: 0.9 });
        let gem = mesh(sphere_geo!(0.006, 6, 5), &gem_mat);
        gem.position().set(gx * 0.055, 0.061, gz * 0.030);
        relic.add(&gem);
    }
    // Central medallion gem (larger).
    let med_mat = basic_mat!({ color: pick_u32(&gem_colors), transparent: true, opacity: 0.9 });
    let med = mesh(sphere_geo!(0.010, 8, 6), &med_mat);
    med.position().set(0, 0.062, 0);
    relic.add(&med);
    // Faint glow from medallion.
    let glow = point_light!(med_mat.color_v().get_hex(), 0.20, 0.5, 2);
    glow.position().set(0, 0.062, 0);
    relic.add(&glow);
    // Slow pulse.
    let pulse_phase = rnd() * PI * 2.0;
    g.animate(move |_g, _dt, t| {
        let p = (t * 0.8 + pulse_phase).sin() * 0.5 + 0.5;
        glow.set_intensity(0.10 + p * 0.15);
    });
    relic.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    relic.rotation().y = rnd() * PI * 2.0;
    g.world.add(&relic);
}

// R491 stained glass window texture.
game::lazy_static_tex!(
    stained_glass_texture,
    canvas_tex(192, 256, |ctx, w, h| {
        // Dark lead outline background.
        ctx.set_fill_style("#0a0a10");
        ctx.fill_rect(0, 0, w, h);
        // Cell colors palette.
        let palette = ["#a02040", "#d8b040", "#2050d8", "#30a030", "#a020d0", "#f0a020"];
        // 6-8 rows of cells with irregular shapes.
        let (rows, cols) = (8, 5);
        for r in 0..rows {
            for c in 0..cols {
                ctx.set_fill_style(palette[(r + c * 3) % palette.len()]);
                let cx = c as f64 * (w / cols as f64) + 4.0;
                let cy = r as f64 * (h / rows as f64) + 4.0;
                let cw = w / cols as f64 - 8.0;
                let ch = h / rows as f64 - 8.0;
                ctx.begin_path();
                // Irregular hexagon-ish shape.
                ctx.move_to(cx + cw * 0.2, cy);
                ctx.line_to(cx + cw * 0.8, cy);
                ctx.line_to(cx + cw, cy + ch * 0.5);
                ctx.line_to(cx + cw * 0.8, cy + ch);
                ctx.line_to(cx + cw * 0.2, cy + ch);
                ctx.line_to(cx, cy + ch * 0.5);
                ctx.close_path();
                ctx.fill();
            }
        }
        // Central medallion.
        ctx.set_fill_style("#f0d020");
        ctx.begin_path();
        ctx.arc(w / 2.0, h / 2.0, 26, 0, PI * 2.0);
        ctx.fill();
        ctx.set_stroke_style("#0a0a10");
        ctx.set_line_width(3);
        ctx.stroke();
        // Cross inside medallion.
        ctx.set_line_width(4);
        ctx.begin_path();
        ctx.move_to(w / 2.0, h / 2.0 - 18.0);
        ctx.line_to(w / 2.0, h / 2.0 + 18.0);
        ctx.move_to(w / 2.0 - 12.0, h / 2.0);
        ctx.line_to(w / 2.0 + 12.0, h / 2.0);
        ctx.stroke();
    })
);

/// The wall-hung panel placement shared by the window / portrait / crucifix:
/// returns nothing, sets rotation.y and position (one draw for the slide).
fn hang_on_wall(g: &Game, o: &Obj, c: &WallCand, inset: f64, center_y: f64) {
    let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - inset);
    let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - inset);
    if c.dx.abs() == 1 {
        o.rotation().y = if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 };
        o.position().set(wall_x, center_y, wall_z + (rnd() - 0.5) * 0.6);
    } else {
        o.rotation().y = if c.dz > 0 { PI } else { 0.0 };
        o.position().set(wall_x + (rnd() - 0.5) * 0.6, center_y, wall_z);
    }
}

// R491 stained glass window: 4% chance per level. Wall-mounted
// glass fragment with canvas texture + colored inner light.
pub fn place_stained_glass_windows(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let candidates = wall_cands(g);
    if candidates.is_empty() {
        return;
    }
    let c = candidates[(rnd() * candidates.len() as f64).floor() as usize];
    let frame_mat = lambert!({ color: 0x2a2418 });
    let glass_mat = basic_mat!({ map: stained_glass_texture(), side: DoubleSide });
    let window = group();
    let (w, h, t, fw) = (0.30, 0.44, 0.012, 0.020);
    // Dark iron frame.
    for (bar_w, bar_h, bx, by) in [
        (w + fw * 2.0, fw, 0.0, h / 2.0 + fw / 2.0),
        (w + fw * 2.0, fw, 0.0, -h / 2.0 - fw / 2.0),
        (fw, h, -w / 2.0 - fw / 2.0, 0.0),
        (fw, h, w / 2.0 + fw / 2.0, 0.0),
    ] {
        let bar = mesh(pg("StainedGlassWindows__bar", box_geo!(bar_w, bar_h, t)), &frame_mat);
        bar.position().set(bx, by, 0);
        window.add(&bar);
    }
    // Glass panel.
    let glass = mesh(plane_geo!(w, h), &glass_mat);
    glass.position().set(0, 0, t / 2.0 + 0.001);
    window.add(&glass);
    let center_y = 1.70;
    hang_on_wall(g, &window, &c, 0.010, center_y);
    g.world.add(&window);
    // Colored inner light casting from window position.
    let lit_colors: [u32; 3] = [0xf0d020, 0xd8b040, 0xa02040];
    let lit_color = pick_u32(&lit_colors);
    let w_light = point_light!(lit_color, 0.20, 1.2, 2);
    let wp = window.pos();
    let ry = window.rot().y;
    w_light.position().set(wp.x + ry.sin() * 0.15, center_y, wp.z + ry.cos() * 0.15);
    g.world.add(&w_light);
}

// R491 crystal skull: 3% chance per level. Translucent
// bone-crystal skull. Deeply liminal mystical prop.
pub fn place_crystal_skulls(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    } // 3% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick_tile(&open);
    let crystal_mat = basic_mat!({ color: 0xe0f0f0, transparent: true, opacity: 0.35 });
    let socket_mat = basic_mat!({ color: 0xa0b0e0, transparent: true, opacity: 0.6 });
    let skull = group();
    // Cranium.
    let cranium = mesh(sphere_geo!(0.045, 14, 12), &crystal_mat);
    cranium.scale().set(1, 1, 1.1);
    cranium.position().y = 0.045;
    skull.add(&cranium);
    // Jaw.
    let jaw = mesh(box_geo!(0.055, 0.014, 0.040), &crystal_mat);
    jaw.position().set(0, 0.014, 0.010);
    skull.add(&jaw);
    // Sockets.
    for s in [-1.0, 1.0] {
        let socket = mesh(circle_geo!(0.010, 8), &socket_mat);
        socket.position().set(s * 0.014, 0.052, 0.040);
        skull.add(&socket);
    }
    // Faint inner glow point light.
    let glow = point_light!(0xa0e0f0, 0.30, 0.6, 2);
    glow.position().y = 0.045;
    skull.add(&glow);
    // Slow pulsing.
    let pulse_phase = rnd() * PI * 2.0;
    g.animate(move |_g, _dt, t| {
        let p = (t * 0.7 + pulse_phase).sin() * 0.5 + 0.5;
        glow.set_intensity(0.15 + p * 0.20);
    });
    skull.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    skull.rotation().y = rnd() * PI * 2.0;
    g.world.add(&skull);
}

// R490 ancient portrait texture: dark oil painting w/ figure silhouette.
game::lazy_static_tex!(
    portrait_texture,
    canvas_tex(192, 256, |ctx, w, h| {
        // Dark background wash.
        let grad = ctx.create_radial_gradient(w / 2.0, h / 3.0, 20, w / 2.0, h / 2.0, 180);
        grad.add_color_stop(0, "#4a3820");
        grad.add_color_stop(1, "#181008");
        ctx.set_fill_style(&grad);
        ctx.fill_rect(0, 0, w, h);
        // Figure silhouette: shoulders + head.
        ctx.set_fill_style("#1a1008");
        // Shoulders (trapezoid).
        ctx.begin_path();
        ctx.move_to(w * 0.15, h * 0.75);
        ctx.line_to(w * 0.85, h * 0.75);
        ctx.line_to(w * 0.75, h * 0.55);
        ctx.line_to(w * 0.25, h * 0.55);
        ctx.close_path();
        ctx.fill();
        // Head (ellipse).
        ctx.begin_path();
        ctx.ellipse(w / 2.0, h * 0.42, w * 0.15, h * 0.15, 0, 0, PI * 2.0);
        ctx.fill();
        // Faint face features.
        ctx.set_stroke_style("rgba(40,25,15,0.7)");
        ctx.set_line_width(1);
        // Eyes (hollow dark spots).
        ctx.set_fill_style("#050300");
        ctx.begin_path();
        ctx.arc(w / 2.0 - 14.0, h * 0.42, 3, 0, PI * 2.0);
        ctx.arc(w / 2.0 + 14.0, h * 0.42, 3, 0, PI * 2.0);
        ctx.fill();
        // Age craquelure.
        ctx.set_stroke_style("rgba(50,35,20,0.4)");
        for _ in 0..30 {
            ctx.begin_path();
            let sx = rnd() * w;
            let sy = rnd() * h;
            ctx.move_to(sx, sy);
            ctx.line_to(sx + rnd() * 30.0 - 15.0, sy + rnd() * 30.0 - 15.0);
            ctx.stroke();
        }
    })
);

// R490 ancient portrait: 4% chance per level. Wall-mounted dark
// oil painting w/ figure silhouette. Vintage/Halloween family.
pub fn place_ancient_portraits(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let candidates = wall_cands(g);
    if candidates.is_empty() {
        return;
    }
    let c = candidates[(rnd() * candidates.len() as f64).floor() as usize];
    let frame_mat = lambert!({ color: 0x603820 });
    let canvas_mat = lambert!({ map: portrait_texture(), side: DoubleSide });
    let portrait = group();
    let (w, h, t, fw) = (0.36, 0.48, 0.020, 0.026);
    // Outer frame (4 boxes).
    for (bar_w, bar_h, bx, by) in [
        (w + fw * 2.0, fw, 0.0, h / 2.0 + fw / 2.0),
        (w + fw * 2.0, fw, 0.0, -h / 2.0 - fw / 2.0),
        (fw, h, -w / 2.0 - fw / 2.0, 0.0),
        (fw, h, w / 2.0 + fw / 2.0, 0.0),
    ] {
        let bar = mesh(pg("AncientPortraits__bar", box_geo!(bar_w, bar_h, t)), &frame_mat);
        bar.position().set(bx, by, 0);
        portrait.add(&bar);
    }
    // Canvas.
    let canvas = mesh(plane_geo!(w, h), &canvas_mat);
    canvas.position().set(0, 0, t / 2.0 + 0.001);
    portrait.add(&canvas);
    // Wall placement.
    hang_on_wall(g, &portrait, &c, 0.014, 1.65);
    // Small tilt for organic hang.
    portrait.rotation().z = (rnd() - 0.5) * 0.05;
    g.world.add(&portrait);
}

// R490 wall crucifix: 5% chance per level. Simple wooden cross
// on wall. Halloween/vintage family.
pub fn place_wall_crucifixes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let candidates = wall_cands(g);
    if candidates.is_empty() {
        return;
    }
    let c = candidates[(rnd() * candidates.len() as f64).floor() as usize];
    let cross_colors: [u32; 3] = [0x603820, 0x8a5030, 0x2a1810];
    let cross_mat = lambert!({ color: pick_u32(&cross_colors) });
    let cross = group();
    let (v_len, h_len, t) = (0.28, 0.16, 0.020);
    // Vertical bar.
    let v_bar = mesh(pg("WallCrucifixes__upright", box_geo!(0.026, v_len, t)), &cross_mat);
    v_bar.position().y = 0.0;
    cross.add(&v_bar);
    // Horizontal bar.
    let h_bar = mesh(pg("WallCrucifixes__crossbar", box_geo!(h_len, 0.024, t)), &cross_mat);
    h_bar.position().y = 0.050;
    cross.add(&h_bar);
    // Wall placement.
    hang_on_wall(g, &cross, &c, 0.014, 1.70);
    // Slight tilt.
    cross.rotation().z = (rnd() - 0.5) * 0.08;
    g.world.add(&cross);
}

// R489 small owl figurine: 5% chance per level. Dark owl
// silhouette with big glowing yellow eyes. Halloween family.
pub fn place_small_owls(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick_tile(&open);
    let owl_colors: [u32; 3] = [0x3a2818, 0x201410, 0x5a4030];
    let owl_mat = lambert!({ color: pick_u32(&owl_colors) });
    let eye_mat = basic_mat!({ color: 0xf0d020 });
    let eye_rim_mat = lambert!({ color: 0xe8dcc0 });
    let beak_mat = lambert!({ color: 0xd8b040 });
    let owl = group();
    // Body: elongated ellipsoid.
    let body = mesh(pg("SmallOwls__body", sphere_geo!(0.040, 12, 10)), &owl_mat);
    body.scale().set(1, 1.4, 0.9);
    body.position().y = 0.055;
    owl.add(&body);
    // Head: separate sphere.
    let head = mesh(pg("SmallOwls__head", sphere_geo!(0.033, 10, 8)), &owl_mat);
    head.position().y = 0.115;
    owl.add(&head);
    // 2 eye rings + 2 glowing eye centers.
    for s in [-1.0, 1.0] {
        let rim = mesh(circle_geo!(0.012, 8), &eye_rim_mat);
        rim.position().set(s * 0.014, 0.120, 0.028);
        owl.add(&rim);
        let eye = mesh(sphere_geo!(0.005, 6, 5), &eye_mat);
        eye.position().set(s * 0.014, 0.120, 0.030);
        owl.add(&eye);
    }
    // Beak triangle.
    let beak_geo = tri_geo([-0.004, 0.116, 0.032, 0.004, 0.116, 0.032, 0.0, 0.108, 0.038]);
    let beak = mesh(beak_geo, &beak_mat);
    owl.add(&beak);
    // 2 wing markers on body sides.
    for s in [-1.0, 1.0] {
        let wing_geo = tri_geo([s * 0.024, 0.080, 0.0, s * 0.038, 0.036, 0.0, s * 0.026, 0.024, 0.0]);
        let wing = mesh(wing_geo, &owl_mat);
        owl.add(&wing);
    }
    // 2 tuft ears (small triangles above head).
    for s in [-1.0, 1.0] {
        let tuft = mesh(pg("SmallOwls__tuft", cone_geo!(0.006, 0.012, 3)), &owl_mat);
        // Base 2 mm into the head (r 0.033 at y 0.115), 0.45 rad off the crown,
        // leaning out 0.35 rad, so the tuft stands clear instead of buried.
        tuft.rotation().z = -s * 0.35;
        tuft.position().set(s * 0.0156, 0.1484, 0);
        owl.add(&tuft);
    }
    // Base perch (small disc).
    let perch = mesh(pg("SmallOwls__perch", cyl_geo!(0.024, 0.028, 0.008, 8)), &owl_mat);
    perch.position().y = 0.004;
    owl.add(&perch);
    owl.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    owl.rotation().y = rnd() * PI * 2.0;
    g.world.add(&owl);
}

// R489 brass candlestick with wax drips: 6% chance per level.
// Brass base + column + candle w/ visible wax runs + flame.
pub fn place_brass_candlesticks(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick_tile(&open);
    let brass_mat = lambert!({ color: 0xc89040 });
    let wax_mat = lambert!({ color: 0xf0e8d8 });
    let wick_mat = lambert!({ color: 0x2a2418 });
    let stick = group();
    // Base disc.
    let base = mesh(pg("BrassCandlesticks__base", cyl_geo!(0.035, 0.045, 0.010, 10)), &brass_mat);
    base.position().y = 0.005;
    stick.add(&base);
    // Column.
    let col = mesh(pg("BrassCandlesticks__column", cyl_geo!(0.008, 0.010, 0.060, 6)), &brass_mat);
    col.position().y = 0.040;
    stick.add(&col);
    // Bulge on column.
    let bulge = mesh(pg("BrassCandlesticks__knop", sphere_geo!(0.014, 8, 6)), &brass_mat);
    bulge.scale().set(1, 0.6, 1);
    bulge.position().y = 0.040;
    stick.add(&bulge);
    // Candle cup (drip catcher).
    let cup = mesh(pg("BrassCandlesticks__cup", cyl_geo!(0.024, 0.020, 0.006, 8)), &brass_mat);
    cup.position().y = 0.073;
    stick.add(&cup);
    // Candle.
    let candle_len = 0.06;
    let candle = mesh(pg("CandleClusters__body", cyl_geo!(0.008, 0.008, candle_len, 6)), &wax_mat);
    candle.position().y = 0.076 + candle_len / 2.0;
    stick.add(&candle);
    // Wax drips: 3 small dangling cones from candle base to catcher.
    for i in 0..3 {
        let a = (i as f64 / 3.0) * PI * 2.0;
        let drip_len = 0.014 + rnd() * 0.010;
        let drip = mesh(pg("BrassCandlesticks__drip", cone_geo!(0.003, drip_len, 4)), &wax_mat);
        drip.position().set(a.cos() * 0.007, 0.076 + candle_len - drip_len / 2.0 - 0.002, a.sin() * 0.007);
        drip.rotation().x = PI;
        stick.add(&drip);
    }
    // R525 wax pool at candlestick base (dried candle drip accumulation).
    let pool_geo = Geo::new();
    let n_pool_verts = 8u32;
    let mut pool_verts: Vec<f64> = vec![];
    let mut pool_indices: Vec<u32> = vec![];
    // Center vertex.
    pool_verts.extend_from_slice(&[0.0, 0.0002, 0.0]);
    // Rim vertices in an irregular blob.
    for i in 0..n_pool_verts {
        let a = (i as f64 / n_pool_verts as f64) * PI * 2.0;
        let r = 0.028 + rnd() * 0.015;
        pool_verts.extend_from_slice(&[a.cos() * r, 0.0001, a.sin() * r]);
    }
    for i in 0..n_pool_verts {
        pool_indices.extend_from_slice(&[0, i + 1, ((i + 1) % n_pool_verts) + 1]);
    }
    pool_geo.set_position(pool_verts.iter().map(|&v| v as f32).collect());
    pool_geo.set_index(pool_indices);
    pool_geo.compute_vertex_normals();
    let wax_pool = mesh(pool_geo, &wax_mat);
    // On the base's domed top (0.010 at the centre, 0.0088 at r 0.035),
    // shrunk to stay inside the dome, clear of the cove and beaded edge.
    wax_pool.scale().set(0.81, 1, 0.81);
    wax_pool.rotation().x = PI; // the fan winds clockwise from above: flip it so its face points up
    wax_pool.position().set(0, 0.0097, 0);
    stick.add(&wax_pool);
    // Wick.
    let wick = mesh(pg("CandleClusters__wick", cyl_geo!(0.001, 0.001, 0.008, 3)), &wick_mat);
    wick.position().y = 0.076 + candle_len + 0.004;
    stick.add(&wick);
    // Lit? 60% chance.
    if rnd() < 0.60 {
        let flame_mat = basic_mat!({ color: 0xffd070, transparent: true, opacity: 0.85 });
        let flame = mesh(cone_geo!(0.004, 0.012, 5), &flame_mat);
        flame.position().y = 0.076 + candle_len + 0.014;
        stick.add(&flame);
        let glow = point_light!(0xffb060, 0.35, 0.9, 2);
        glow.position().y = 0.076 + candle_len + 0.014;
        stick.add(&glow);
        // Halo.
        let halo_mat = basic_mat!({ color: 0xffb060, transparent: true, opacity: 0.25, depthWrite: false });
        let halo = mesh(sphere_geo!(0.020, 6, 5), &halo_mat);
        halo.position().y = 0.076 + candle_len + 0.014;
        stick.add(&halo);
        // Flicker.
        let flicker_phase = rnd() * PI * 2.0;
        let flicker_rate = 6.0 + rnd() * 4.0;
        g.animate(move |_g, _dt, t| {
            let p = t * flicker_rate + flicker_phase;
            let wobble = p.sin() * 0.5 + (p * 1.7).sin() * 0.3;
            glow.set_intensity(0.35 + wobble * 0.08);
            flame.scale().y = 1.0 + wobble * 0.12;
            halo.scale().set_scalar(1.0 + wobble * 0.15);
        });
    }
    stick.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    stick.rotation().y = rnd() * PI * 2.0;
    g.world.add(&stick);
}

// R488 skull stack cluster: 4% chance per level. 2-3 skulls
// stacked/piled. Halloween family.
pub fn place_skull_stacks(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick_tile(&open);
    let bone_mat = lambert!({ color: 0xd8ccb0 });
    let socket_mat = basic_mat!({ color: 0x080604 });
    let cluster = group();
    let n_skulls = 2 + (rnd() * 2.0).floor() as i32;
    for i in 0..n_skulls {
        let skull = group();
        let cranium = mesh(pg("SmallSkulls__cranium", sphere_geo!(0.045, 10, 8)), &bone_mat);
        cranium.scale().set(1, 1, 1.1);
        cranium.position().y = 0.045;
        skull.add(&cranium);
        let jaw = mesh(pg("SmallSkulls__jaw", box_geo!(0.055, 0.014, 0.040)), &bone_mat);
        jaw.position().set(0, 0.014, 0.010);
        skull.add(&jaw);
        // Sockets.
        for s in [-1.0, 1.0] {
            let socket = mesh(circle_geo!(0.010, 8), &socket_mat);
            socket.position().set(s * 0.014, 0.052, 0.040);
            skull.add(&socket);
        }
        // Position stacked with slight offset per level.
        skull.position().set((rnd() - 0.5) * 0.04, i as f64 * 0.075, (rnd() - 0.5) * 0.04);
        skull.rotation().y = rnd() * PI * 2.0;
        if i > 0 {
            skull.rotation().z = (rnd() - 0.5) * 0.4;
        }
        cluster.add(&skull);
    }
    cluster.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    cluster.rotation().y = rnd() * PI * 2.0;
    g.world.add(&cluster);
}

// R466 small skull: 5% chance per level. Bone skull with 2 eye
// sockets + nose gap + teeth. Halloween family.
pub fn place_small_skulls(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick_tile(&open);
    let bone_mat = lambert!({ color: 0xe8dcc0 });
    let socket_mat = basic_mat!({ color: 0x080604 });
    let skull = group();
    // Cranium (main head sphere).
    let cranium = mesh(pg("SmallSkulls__cranium", sphere_geo!(0.045, 12, 10)), &bone_mat);
    cranium.scale().set(1, 1, 1.1);
    cranium.position().y = 0.045;
    skull.add(&cranium);
    // Jaw (smaller flatter cylinder attached below cranium).
    let jaw = mesh(pg("SmallSkulls__jaw", box_geo!(0.055, 0.014, 0.040)), &bone_mat);
    jaw.position().set(0, 0.014, 0.010);
    skull.add(&jaw);
    // 2 eye sockets (dark discs on front).
    for s in [-1.0, 1.0] {
        let socket = mesh(circle_geo!(0.010, 8), &socket_mat);
        socket.position().set(s * 0.014, 0.052, 0.040);
        skull.add(&socket);
    }
    // Nose triangle (small dark).
    let nose_geo = tri_geo([-0.006, 0.038, 0.045, 0.006, 0.038, 0.045, 0.0, 0.024, 0.045]);
    let nose = mesh(nose_geo, &socket_mat);
    skull.add(&nose);
    // Teeth: small bump grid (5 bumps).
    // Set on the jaw's top edge (0.021), following its horseshoe front
    // (z 0.0242 at the chin curving back to ~0.019 at x ±0.014), each tooth
    // turned to face out of the curve.
    for i in 0..5 {
        let tooth = mesh(pg("SmallSkulls__tooth", box_geo!(0.005, 0.006, 0.002)), &bone_mat);
        let tx = -0.014 + i as f64 * 0.007;
        tooth.position().set(tx, 0.0225, 0.0242 - 0.00545 * (tx / 0.0148).powi(2));
        tooth.rotation().y = (2.0 * 0.00545 * tx / (0.0148 * 0.0148)).atan();
        skull.add(&tooth);
    }
    skull.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    skull.rotation().y = rnd() * PI * 2.0;
    // Tilted to the side sometimes.
    if rnd() < 0.30 {
        let sgn = if rnd() < 0.5 { 1.0 } else { -1.0 };
        let tilt = sgn * (PI / 6.0 + rnd() * PI / 4.0);
        skull.rotation().z = tilt;
    }
    g.world.add(&skull);
}

// R466 small potion bottle: 8% chance per level. Glass vial
// with colored liquid + cork stopper. Halloween/mystical family.
pub fn place_small_potions(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick_tile(&open);
    let glass_mat = basic_mat!({ color: 0xc0e0d0, transparent: true, opacity: 0.35 });
    let potion_colors: [u32; 5] = [0x30d040, 0xa020d0, 0xd82040, 0x2050d8, 0xd8b040];
    let potion_color = pick_u32(&potion_colors);
    let potion_mat = basic_mat!({ color: potion_color, transparent: true, opacity: 0.85 });
    let cork_mat = lambert!({ color: 0xa87050 });
    let potion = group();
    // Body: round-shouldered bottle.
    let body = mesh(sphere_geo!(0.024, 10, 8, 0, PI * 2.0, 0.4, PI - 0.4), &glass_mat);
    body.position().y = 0.028;
    potion.add(&body);
    // Neck.
    let neck = mesh(cyl_geo!(0.008, 0.008, 0.020, 6), &glass_mat);
    neck.position().y = 0.056;
    potion.add(&neck);
    // Cork.
    let cork = mesh(pg("SmallPotions__cork", cyl_geo!(0.007, 0.007, 0.010, 6)), &cork_mat);
    cork.position().y = 0.070;
    potion.add(&cork);
    // Colored liquid inside (smaller sphere).
    let liquid = mesh(sphere_geo!(0.018, 8, 6), &potion_mat);
    liquid.position().y = 0.026;
    liquid.scale().y = 0.7;
    potion.add(&liquid);
    // 15% chance tipped.
    if rnd() < 0.15 {
        potion.rotation().z = PI / 2.0;
        potion.position().y = 0.018;
    }
    potion.position().x = g.wx(x) + (rnd() - 0.5) * 2.4;
    potion.position().z = g.wz(z) + (rnd() - 0.5) * 2.4;
    potion.rotation().y = rnd() * PI * 2.0;
    g.world.add(&potion);
}

// R465 candy corn scatter: 8% chance per level. 8-14 tri-color
// candy corn pieces scattered. Halloween family.
pub fn place_candy_corn_scatters(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick_tile(&open);
    let white_mat = lambert!({ color: 0xf0ece0 });
    let orange_mat = lambert!({ color: 0xe07020 });
    let yellow_mat = lambert!({ color: 0xf0d020 });
    let grp = group();
    let n_pieces = 8 + (rnd() * 7.0).floor() as i32;
    for _ in 0..n_pieces {
        let candy = group();
        // 3-tone cone divided into thirds vertically.
        let yellow = mesh(pg("CandyCornScatters__yellow", cyl_geo!(0.006, 0.008, 0.008, 4)), &yellow_mat);
        yellow.position().y = 0.004;
        candy.add(&yellow);
        let orange = mesh(pg("CandyCornScatters__orange", cyl_geo!(0.004, 0.006, 0.008, 4)), &orange_mat);
        orange.position().y = 0.012;
        candy.add(&orange);
        let white = mesh(pg("CandyCornScatters__white", cone_geo!(0.004, 0.008, 4)), &white_mat);
        white.position().y = 0.020;
        candy.add(&white);
        // Tipped on side (candy on floor lying flat): past 90° by the taper so
        // the kernel rests along its side, not on its 8 mm base edge sunk 2 mm
        // into the floor with the axis level at 6 mm.
        candy.rotation().z = PI / 2.0 + 0.28f64.atan();
        candy.position().set((rnd() - 0.5) * 0.30, 0.0082, (rnd() - 0.5) * 0.25);
        candy.rotation().y = rnd() * PI * 2.0;
        grp.add(&candy);
    }
    grp.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    grp.rotation().y = rnd() * PI * 2.0;
    g.world.add(&grp);
}

// R465 small trick-or-treat pail: 6% chance per level. Orange
// plastic pumpkin-shaped candy pail. Halloween family.
pub fn place_small_treat_pails(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick_tile(&open);
    let pump_mat = lambert!({ color: 0xd87020 });
    let stem_mat = lambert!({ color: 0x306018 });
    let handle_mat = lambert!({ color: 0x101010 });
    let face_mat = basic_mat!({ color: 0x181008 });
    let pail = group();
    // Body: flattened sphere (pumpkin shape).
    let body = mesh(pg("SmallTreatPails__body", sphere_geo!(0.070, 14, 10)), &pump_mat);
    body.scale().set(1, 0.75, 1);
    body.position().y = 0.055;
    pail.add(&body);
    // Rim (dark cylinder at top for candy opening).
    let rim = mesh(pg("SmallTreatPails__rim", cyl_geo!(0.048, 0.048, 0.015, 12)), &pump_mat);
    rim.position().y = 0.108;
    pail.add(&rim);
    // Small stem.
    let stem = mesh(pg("SmallPumpkins__stem", cyl_geo!(0.008, 0.010, 0.014, 6)), &stem_mat);
    stem.position().y = 0.122;
    pail.add(&stem);
    // Handle arc (black plastic).
    let handle = mesh(pg("SmallTreatPails__handle", torus_geo!(0.055, 0.004, 4, 12, PI)), &handle_mat);
    // Upright arch over the opening; narrowed so its pivot ends meet the
    // rim's sides (r 0.048) at the rim's mid-height.
    handle.scale().x = 0.052 / 0.055;
    handle.position().y = 0.108;
    pail.add(&handle);
    // Face on body: each decal 1 mm outside the (0.070, 0.0525, 0.070)
    // ellipsoid (the lobed body stays inside it), facing along its normal.
    let on_body = |m: &Obj, px: f64, py: f64| {
        let zs = 0.070 * (1.0 - (px / 0.070).powi(2) - ((py - 0.055) / 0.0525).powi(2)).sqrt();
        let mut n = v3(px / 0.0049, (py - 0.055) / 0.00275625, zs / 0.0049);
        n.normalize();
        m.position().set(px, py, zs).add_scaled_vector(&n, 0.001);
        let mut tgt = m.pos();
        tgt.add(&n);
        m.look_at(tgt.x, tgt.y, tgt.z);
    };
    let eye1 = mesh(circle_geo!(0.010, 3), &face_mat);
    on_body(&eye1, -0.023, 0.072);
    pail.add(&eye1);
    let eye2 = mesh(circle_geo!(0.010, 3), &face_mat);
    on_body(&eye2, 0.023, 0.072);
    pail.add(&eye2);
    let mouth = mesh(plane_geo!(0.040, 0.014), &face_mat);
    on_body(&mouth, 0.0, 0.040);
    pail.add(&mouth);
    pail.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    pail.rotation().y = rnd() * PI * 2.0;
    g.world.add(&pail);
}
