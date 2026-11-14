//! `place_vintage_typewriters`
use game::prelude::*;
use placers_kit::place::open_tiles;

pub(crate) fn place_vintage_typewriters(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let body_colors = [0x1a1a1a, 0x2a2a2a, 0x603030, 0x101020, 0x604040];
    let body_mat = lambert!({ color: choice(&body_colors) });
    let paper_mat = lambert!({ color: 0xf0e8d0 });
    let key_mat = lambert!({ color: 0xf0f0e8 });
    let metal_mat = lambert!({ color: 0xc0c0c8 });
    let ink_mat = lambert!({ color: 0x1a1a1a });
    let tw = group();
    // Main body.
    let (body_w, body_d, body_h) = (0.30, 0.22, 0.10);
    let body = mesh(pg("VintageTypewriters__body", box_geo!(body_w, body_h, body_d)), &body_mat);
    body.position().y = body_h / 2.0 + 0.002;
    tw.add(&body);
    // Front slope (angled key deck).
    let deck = mesh(pg("VintageTypewriters__deck", box_geo!(body_w * 0.9, 0.024, body_d * 0.5)), &body_mat);
    deck.position().set(0, body_h * 0.4, body_d * 0.30);
    deck.rotation().x = 0.35;
    tw.add(&deck);
    // 4 rows × 10 keys (round white keys).
    for row in 0..4 {
        for col in 0..10 {
            let key_x = -body_w * 0.4 + col as f64 * (body_w * 0.08);
            let key_z = body_d * 0.20 + row as f64 * 0.020;
            let key_y = body_h * 0.6 - row as f64 * 0.010;
            let key = mesh(pg("VintageTypewriters__key", cyl_geo!(0.007, 0.007, 0.005, 8)), &key_mat);
            key.position().set(key_x, key_y, key_z);
            key.rotation().x = 0.35;
            tw.add(&key);
            // Tiny dark letter dot.
            let letter = mesh(circle_geo!(0.002, 5), &ink_mat);
            letter.rotation().x = -PI / 2.0 + 0.35;
            letter.position().set(key_x, key_y + 0.003, key_z);
            tw.add(&letter);
        }
    }
    // Space bar.
    let space_bar = mesh(pg("VintageTypewriters__spacebar", box_geo!(body_w * 0.6, 0.005, 0.018)), &key_mat);
    space_bar.rotation().x = 0.35;
    space_bar.position().set(0, 0.021, 0.118); // in front of (and below) key row 3
    tw.add(&space_bar);
    // Platen (roller) at back.
    let platen = mesh(pg("VintageTypewriters__platen", cyl_geo!(0.025, 0.025, body_w * 0.7, 12)), &metal_mat);
    platen.rotation().z = PI / 2.0;
    platen.position().set(0, body_h + 0.020, -body_d * 0.15);
    tw.add(&platen);
    // 2 platen end knobs.
    for side in [-1.0, 1.0] {
        let knob = mesh(pg("VintageTypewriters__knob", cyl_geo!(0.014, 0.014, 0.012, 8)), &metal_mat);
        knob.rotation().z = PI / 2.0;
        knob.position().set(side * body_w * 0.37, body_h + 0.020, -body_d * 0.15);
        tw.add(&knob);
    }
    // Paper sheet.
    let (paper_w, paper_h) = (body_w * 0.75, 0.18);
    let paper = mesh(plane_geo!(paper_w, paper_h), &paper_mat);
    paper.position().set(0, body_h + 0.020 + paper_h / 2.0, -body_d * 0.14);
    paper.rotation().x = -0.08; // Slight back-lean.
    tw.add(&paper);
    // Paper back face.
    let paper_back = mesh(plane_geo!(paper_w, paper_h), &paper_mat);
    paper_back.position().set(0, body_h + 0.020 + paper_h / 2.0, -body_d * 0.14 - 0.001);
    paper_back.rotation().x = -0.08;
    paper_back.rotation().y = PI;
    tw.add(&paper_back);
    // 5-7 rows of partial typed text.
    let line_count = 5 + (rnd() * 3.0).floor() as i32;
    for i in 0..line_count {
        let line_len = paper_w * (0.35 + rnd() * 0.55);
        let is_last = i == line_count - 1;
        // Last line is much shorter (interrupted mid-word).
        let actual_len = if is_last { paper_w * (0.15 + rnd() * 0.25) } else { line_len };
        let line = mesh(box_geo!(actual_len, 0.002, 0.001), &ink_mat);
        line.rotation().x = -0.08;
        let y_off = paper_h * 0.5 - 0.020 - i as f64 * 0.020;
        line.position().set(
            -paper_w / 2.0 + actual_len / 2.0 + 0.010,
            body_h + 0.020 + paper_h / 2.0 + y_off * (-0.08f64).cos(),
            -body_d * 0.14 - 0.002 + y_off * (-0.08f64).sin(),
        );
        tw.add(&line);
    }
    // Typebar hint.
    let typebar = mesh(pg("VintageTypewriters__typebar", box_geo!(0.003, 0.030, 0.003)), &metal_mat);
    typebar.rotation().x = 0.6;
    typebar.position().set(0, body_h + 0.010, -body_d * 0.05);
    tw.add(&typebar);
    // Carriage return lever on right.
    let lever = mesh(pg("VintageTypewriters__lever", cyl_geo!(0.002, 0.002, 0.030, 4)), &metal_mat);
    lever.rotation().z = 0.5;
    lever.position().set(body_w * 0.42, body_h + 0.030, -body_d * 0.15);
    tw.add(&lever);
    tw.position().set(g.wx(x) + (rnd() - 0.5) * 1.6, 0, g.wz(z) + (rnd() - 0.5) * 1.6);
    tw.rotation().y = rnd() * PI * 2.0;
    g.world.add(&tw);
}
