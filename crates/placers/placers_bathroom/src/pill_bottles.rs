//! `place_pill_bottles`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Opened pill bottle w/ scattered pills (floor): 5% chance per level.
pub(crate) fn place_pill_bottles(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    }
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = open_tiles[(rnd() * open_tiles.len() as f64).floor() as usize];
    // Bottle color pool — amber, orange, dark blue, brown.
    let bottle_colors = [0xc07030, 0xa04010, 0x304060, 0x603010];
    let bottle_mat = lambert!({ color: pick(&bottle_colors), transparent: true, opacity: 0.75 });
    let cap_colors = [0xffffff, 0xa02020, 0x101010, 0xf0e0a0];
    let cap_mat = lambert!({ color: pick(&cap_colors) });
    let label_mat = lambert!({ color: 0xf0e8d0 });
    // Pill colors — variety per bottle.
    let pill_colors = [0xf0f0f0, 0xf0d0a0, 0xa0c0f0, 0xf0c0c0, 0xd0f0d0, 0xf0e0a0, 0x603030, 0x202020];
    let pill_color1 = pick(&pill_colors);
    let pill_color2 = pick(&pill_colors);
    let pill_mat1 = lambert!({ color: pill_color1 });
    let pill_mat2 = lambert!({ color: pill_color2 });
    let pb = group();
    // Main bottle body — squat cylinder.
    let (bottle_r, bottle_h) = (0.020, 0.055);
    let body = mesh(cyl_geo!(bottle_r, bottle_r, bottle_h, 10), &bottle_mat);
    body.position().y = bottle_h / 2.0 + 0.002;
    pb.add(&body);
    // Label wrapping most of body — slightly larger radius, cream colored.
    let label = mesh(pg("PillBottles__label", cyl_geo!(bottle_r + 0.0005, bottle_r + 0.0005, bottle_h * 0.7, 10, 1, true)), &label_mat);
    label.position().y = bottle_h / 2.0 + 0.002;
    pb.add(&label);
    // Thin dark medication text lines on label — 3 horizontal boxes.
    let text_mat = lambert!({ color: 0x101010 });
    for i in 0..3 {
        let text_line = mesh(box_geo!(bottle_r * 1.5, 0.002, 0.001), &text_mat);
        text_line.position().set(0, bottle_h * 0.75 - i as f64 * 0.010, bottle_r + 0.001);
        pb.add(&text_line);
    }
    // Cap — tilted off to the side (open).
    let cap = mesh(pg("PillBottles__cap", cyl_geo!(bottle_r + 0.001, bottle_r + 0.001, 0.010, 10)), &cap_mat);
    // Cap and tabs are tipped on their side together; the group height
    // puts the tilted cap's rim on the floor.
    let cap_tilt = (rnd() - 0.5) * 0.3;
    let cap_g = group();
    cap_g.rotation().z = PI / 2.0 + cap_tilt;
    cap_g.position().set(0.048, (bottle_r + 0.001) * cap_tilt.dcos() + 0.005 * cap_tilt.dsin().abs(), 0.02);
    cap_g.add(&cap);
    pb.add(&cap_g);
    // Cap top (child-proof push tabs).
    for i in 0..4 {
        let ang = (i as f64 / 4.0) * PI * 2.0;
        let tab = mesh(pg("PillBottles__tab", box_geo!(0.002, 0.008, 0.004)), &cap_mat);
        // Radial ribs on the cap's skirt, 45 deg off so no rib is under the tipped cap.
        let t_ang = ang + PI / 4.0;
        tab.position().set(t_ang.dcos() * (bottle_r + 0.0015), 0, t_ang.dsin() * (bottle_r + 0.0015));
        tab.rotation().y = -t_ang;
        // The old per-tab tilt draw is kept (unused) so the later random draws stay the same.
        let _ = rnd();
        cap_g.add(&tab);
    }
    // Scattered pills — 5-12 tiny elongated pill-shaped meshes.
    let pill_count = 5 + (rnd() * 8.0).floor() as i32;
    for _i in 0..pill_count {
        // 50/50 pill type — capsule (2 colors) or tablet (short cylinder).
        let is_capsule = rnd() < 0.5;
        let pill = group();
        if is_capsule {
            // 2-tone capsule: 2 short cylinders + rounded ends.
            let half_len = 0.004;
            let half_a = mesh(pg("PillBottles__half", cyl_geo!(0.002, 0.002, half_len, 6)), &pill_mat1);
            half_a.rotation().z = PI / 2.0;
            half_a.position().set(-half_len / 2.0, 0, 0);
            pill.add(&half_a);
            let half_b = mesh(pg("PillBottles__half", cyl_geo!(0.002, 0.002, half_len, 6)), &pill_mat2);
            half_b.rotation().z = PI / 2.0;
            half_b.position().set(half_len / 2.0, 0, 0);
            pill.add(&half_b);
            // Rounded caps.
            let cap_a = mesh(pg("PillBottles__end", sphere_geo!(0.002, 5, 4)), &pill_mat1);
            cap_a.position().set(-half_len, 0, 0);
            pill.add(&cap_a);
            let cap_b = mesh(pg("PillBottles__end", sphere_geo!(0.002, 5, 4)), &pill_mat2);
            cap_b.position().set(half_len, 0, 0);
            pill.add(&cap_b);
        } else {
            // Round tablet — flat cylinder.
            let tab_mat = if rnd() < 0.5 { &pill_mat1 } else { &pill_mat2 };
            let tablet = mesh(pg("PillBottles__tablet", cyl_geo!(0.003, 0.003, 0.002, 6)), tab_mat);
            pill.add(&tablet);
        }
        // Random scatter position.
        let ang = rnd() * PI * 2.0;
        let rr = 0.02 + rnd() * 0.09;
        pill.position().set(ang.dcos() * rr, 0.002, ang.dsin() * rr);
        pill.rotation().y = rnd() * PI * 2.0;
        pill.rotation().z = (rnd() - 0.5) * 0.5;
        pb.add(&pill);
        // Subtle wobble animator — 30% chance per pill.
        if rnd() < 0.3 {
            let wobble_phase = rnd() * PI * 2.0;
            let base_rot_z = pill.rot().z;
            let base_rot_y = pill.rot().y;
            let pill2 = pill.clone();
            g.animate(move |_g, _dt, t| {
                pill2.rotation().z = base_rot_z + (t * 1.6 + wobble_phase).dsin() * 0.03;
                pill2.rotation().y = base_rot_y + (t * 1.2 + wobble_phase * 1.3).dsin() * 0.02;
            });
        }
    }
    pb.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    pb.rotation().y = rnd() * PI * 2.0;
    g.world.add(&pb);
}
