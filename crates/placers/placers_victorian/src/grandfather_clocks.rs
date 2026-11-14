//! `place_grandfather_clocks`
use game::prelude::*;
use placers_kit::place::wall_tiles;

// Grandfather clock w/ pendulum (floor near wall): 2% chance per level.
pub(crate) fn place_grandfather_clocks(g: &mut Game, _dist: &[i32]) {
    g.pp.grandfather_clock_positions = vec![]; // reset per level
    if rnd() < 0.98 {
        return;
    }
    let walls = wall_tiles(g);
    if walls.is_empty() {
        return;
    }
    let (x, z, dx, dz) = walls[(rnd() * walls.len() as f64).floor() as usize];
    let wood_colors = [0x604020, 0x8b5a2b, 0x201010, 0x504030];
    let wood_mat = lambert!({ color: choice(&wood_colors) });
    let gold_mat = lambert!({ color: 0xc0a060 });
    let face_mat = lambert!({ color: 0xf0e8d0 });
    let glass_mat = lambert!({ color: 0xd0d8e0, transparent: true, opacity: 0.15 });
    let hand_mat = lambert!({ color: 0x101010 });
    let gc = group();
    // Total height ~1.85 m, width ~0.35 m, depth ~0.20 m.
    let total_h = 1.85;
    let (body_w, body_d) = (0.32, 0.18);
    // 3 sections: head (clock face), waist (pendulum), base.
    let (base_h, waist_h, head_h) = (0.32, 0.90, 0.63);
    // Base cabinet — bottom section.
    let base = mesh(pg("GrandfatherClocks__base", box_geo!(body_w, base_h, body_d)), &wood_mat);
    base.position().y = base_h / 2.0 + 0.005;
    gc.add(&base);
    // Base decorative trim (small horizontal boxes).
    let base_trim_top = mesh(pg("GrandfatherClocks__baseTrim", box_geo!(body_w + 0.010, 0.020, body_d + 0.010)), &wood_mat);
    base_trim_top.position().y = base_h + 0.010;
    gc.add(&base_trim_top);
    // Waist — pendulum housing. Slightly narrower than base.
    let waist_w = body_w * 0.85;
    let waist = mesh(pg("GrandfatherClocks__waist", box_geo!(waist_w, waist_h, body_d)), &wood_mat);
    waist.position().y = base_h + 0.020 + waist_h / 2.0;
    gc.add(&waist);
    // Waist has a glass door front — larger central rectangle showing pendulum.
    let (waist_glass_w, waist_glass_h) = (waist_w - 0.070, waist_h - 0.120);
    let waist_glass = mesh(box_geo!(waist_glass_w, waist_glass_h, 0.002), &glass_mat);
    waist_glass.position().set(0, base_h + 0.020 + waist_h / 2.0, body_d / 2.0 + 0.001);
    gc.add(&waist_glass);
    // Pendulum inside waist — long rod + heavy disc at bottom.
    let pendulum_group = group();
    let pendulum_len = 0.65;
    let rod = mesh(pg("GrandfatherClocks__rod", cyl_geo!(0.004, 0.004, pendulum_len, 5)), &gold_mat);
    rod.position().y = -pendulum_len / 2.0;
    pendulum_group.add(&rod);
    // Pendulum bob — large disc at bottom.
    let bob = mesh(pg("GrandfatherClocks__bob", cyl_geo!(0.050, 0.050, 0.012, 16)), &gold_mat);
    bob.rotation().x = PI / 2.0;
    bob.position().y = -pendulum_len;
    pendulum_group.add(&bob);
    // Pendulum group pivots at top center of waist.
    let pivot_y = base_h + 0.020 + waist_h - 0.070;
    pendulum_group.position().set(0, pivot_y, body_d / 2.0 - 0.030);
    gc.add(&pendulum_group);
    // Head — top section w/ clock face.
    let head_w = body_w * 0.95;
    let head = mesh(pg("GrandfatherClocks__head", box_geo!(head_w, head_h, body_d)), &wood_mat);
    head.position().y = base_h + 0.020 + waist_h + head_h / 2.0;
    gc.add(&head);
    // Ornate crown top (small trapezoidal piece).
    let crown_top = mesh(pg("GrandfatherClocks__crown", box_geo!(head_w + 0.014, 0.030, body_d + 0.010)), &wood_mat);
    crown_top.position().y = total_h - 0.010;
    gc.add(&crown_top);
    // Small pointed finial on top.
    let finial = mesh(pg("GrandfatherClocks__finial", cone_geo!(0.014, 0.040, 6)), &gold_mat);
    finial.position().y = total_h + 0.040; // on the pediment's crest (hood top 1.87)
    gc.add(&finial);
    // Clock face — round disc set into head.
    let face_r = 0.10;
    let face_height = base_h + 0.020 + waist_h + head_h * 0.55;
    let face = mesh(circle_geo!(face_r, 20), &face_mat);
    face.position().set(0, face_height, body_d / 2.0 + 0.003);
    gc.add(&face);
    // Face rim.
    let face_rim = mesh(pg("GrandfatherClocks__faceRim", torus_geo!(face_r + 0.006, 0.008, 4, 20)), &gold_mat);
    face_rim.rotation().x = 0.0; // the torus already rings the dial in XY
    face_rim.position().set(0, face_height, body_d / 2.0 + 0.004);
    gc.add(&face_rim);
    // 12 Roman-numeral position ticks.
    for i in 0..12 {
        let ang = (i as f64 / 12.0) * PI * 2.0 - PI / 2.0;
        let is_major = i % 3 == 0;
        let tick = mesh(box_geo!(0.005, 0.001, if is_major { 0.012 } else { 0.006 }), &hand_mat);
        tick.position().set(ang.cos() * face_r * 0.85, face_height + ang.sin() * face_r * 0.85, body_d / 2.0 + 0.005);
        tick.rotation().set(PI / 2.0, ang + PI / 2.0, 0); // long axis radial on the dial, thin axis out
        gc.add(&tick);
    }
    // Frozen hands.
    let hour_ang = rnd() * PI * 2.0 - PI / 2.0;
    let min_ang = rnd() * PI * 2.0 - PI / 2.0;
    let hour = mesh(pg("GrandfatherClocks__hour", box_geo!(0.008, 0.002, 0.045)), &hand_mat);
    hour.position().set(hour_ang.cos() * 0.022, face_height + hour_ang.sin() * 0.022, body_d / 2.0 + 0.007);
    hour.rotation().set(PI / 2.0, hour_ang + PI / 2.0, 0); // tip (+Z) radial, flat on the dial
    gc.add(&hour);
    let minute = mesh(pg("GrandfatherClocks__minute", box_geo!(0.005, 0.002, 0.068)), &hand_mat);
    minute.position().set(min_ang.cos() * 0.034, face_height + min_ang.sin() * 0.034, body_d / 2.0 + 0.009);
    minute.rotation().set(PI / 2.0, min_ang + PI / 2.0, 0);
    gc.add(&minute);
    // Center pivot dot.
    let pivot_dot = mesh(circle_geo!(0.006, 6), &hand_mat);
    pivot_dot.position().set(0, face_height, body_d / 2.0 + 0.0105);
    gc.add(&pivot_dot);
    // Pendulum swing animator — 60% chance (mostly working clocks).
    let is_swinging = rnd() < 0.6;
    if is_swinging {
        let swing_phase = rnd() * PI * 2.0;
        let pg2 = pendulum_group.clone();
        g.animate(move |_g, _dt, t| {
            // Classic pendulum swing at ~0.5 Hz w/ ±0.07 rad amp.
            pg2.rotation().z = (t * PI + swing_phase).sin() * 0.07;
        });
    }
    gc.user().set("isSwinging", is_swinging);
    gc.rotation().y = (-(dx as f64)).atan2(-(dz as f64));
    // Position at wall — center against wall.
    let px = g.wx(x) + dx as f64 * (CELL * 0.5 - 0.150);
    let pz = g.wz(z) + dz as f64 * (CELL * 0.5 - 0.150);
    gc.position().set(px, 0, pz);
    g.world.add(&gc);
    // Register position only for swinging clocks.
    if is_swinging {
        let p = gc.pos();
        g.pp.grandfather_clock_positions.push(v3(p.x, 0, p.z));
    }
}
