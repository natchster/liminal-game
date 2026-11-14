//! `place_packed_suitcases`
use game::prelude::*;
use placers_kit::place::open_tiles;

// Packed suitcase w/ hanging tag (floor): 4% chance per level.
pub(crate) fn place_packed_suitcases(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    // Suitcase color pool — classic travel colors.
    let shell_colors = [0x604020, 0x101010, 0xa02040, 0x2060a0, 0x606060, 0x804a20, 0x408060];
    let shell_mat = lambert!({ color: choice(&shell_colors) });
    let metal_mat = lambert!({ color: 0xa0a0a8 });
    let gold_mat = lambert!({ color: 0xc0a060 });
    let s = group();
    // Main body — tall upright rectangular hardshell.
    let (body_w, body_h, body_d) = (0.30, 0.42, 0.14);
    let body = mesh(pg("PackedSuitcases__body", box_geo!(body_w, body_h, body_d)), &shell_mat);
    body.position().y = body_h / 2.0 + 0.010;
    s.add(&body);
    // Central divider line (horizontal seam — where two halves meet).
    let seam = mesh(pg("PackedSuitcases__seam", box_geo!(body_w + 0.005, 0.003, body_d + 0.005)), &gold_mat);
    seam.position().y = body_h * 0.5 + 0.010;
    s.add(&seam);
    // 2 latches on top edge.
    for off in [-body_w * 0.30, body_w * 0.30] {
        let latch = mesh(pg("PackedSuitcases__latch", box_geo!(0.024, 0.010, 0.014)), &gold_mat);
        latch.position().set(off, body_h * 0.5 + 0.010, body_d * 0.5 + 0.007);
        s.add(&latch);
    }
    // 4 corner reinforcement caps (small darker boxes at bottom corners).
    let dark_corner_mat = lambert!({ color: 0x1a1a1a });
    for cx in [-body_w * 0.5, body_w * 0.5] {
        for cz in [-body_d * 0.5, body_d * 0.5] {
            let cap = mesh(pg("PackedSuitcases__corner", box_geo!(0.024, 0.016, 0.024)), &dark_corner_mat);
            // Corner feet: reach the floor under the body (bottom at y 0.01), level with the wheels.
            cap.position().set(cx, 0.008, cz);
            s.add(&cap);
        }
    }
    // Wheels — 2 small dark cylinders at bottom (if roller style).
    let is_roller = rnd() < 0.6;
    if is_roller {
        for side in [-1.0f64, 1.0] {
            let wheel = mesh(pg("PackedSuitcases__wheel", cyl_geo!(0.014, 0.014, 0.010, 8)), &dark_corner_mat);
            wheel.rotation().z = PI / 2.0;
            wheel.position().set(side * body_w * 0.35, 0.014, -body_d * 0.5 - 0.006);
            s.add(&wheel);
        }
        // Retractable telescoping handle bar at top.
        for side in [-1.0f64, 1.0] {
            let shaft = mesh(pg("PackedSuitcases__shaft", cyl_geo!(0.006, 0.006, 0.14, 5)), &metal_mat);
            shaft.position().set(side * body_w * 0.30, body_h + 0.080, 0);
            s.add(&shaft);
        }
        let bar = mesh(pg("PackedSuitcases__bar", cyl_geo!(0.008, 0.008, body_w * 0.65, 6)), &metal_mat);
        bar.rotation().z = PI / 2.0;
        bar.position().set(0, body_h + 0.148, 0);
        s.add(&bar);
    } else {
        // Classic carry handle — arch on top.
        let handle_arch = mesh(pg("PackedSuitcases__carryHandle", torus_geo!(0.040, 0.008, 4, 10, PI)), &gold_mat);
        handle_arch.rotation().z = 0.0;
        handle_arch.rotation().x = -PI / 2.0;
        handle_arch.position().set(0, body_h + 0.010, 0);
        s.add(&handle_arch);
    }
    // Side handle on end (small D-ring style).
    let side_handle = mesh(pg("PackedSuitcases__sideHandle", torus_geo!(0.018, 0.005, 4, 8, PI)), &gold_mat);
    side_handle.rotation().z = PI / 2.0;
    side_handle.rotation().y = -PI / 2.0;
    side_handle.position().set(body_w * 0.5 + 0.005, body_h * 0.5 + 0.010, 0);
    s.add(&side_handle);
    // Luggage tag — small paper tag on string, hanging from top handle.
    let tag_mat = lambert!({ color: 0xf0e8c0 });
    let string_mat = lambert!({ color: 0x404040 });
    let tag = group();
    // String (short vertical cylinder).
    let tag_string = mesh(cyl_geo!(0.0008, 0.0008, 0.04, 3), &string_mat);
    // Runs from the tag's eyelet (y -0.005) up to y 0.04.
    tag_string.scale().y = 1.125;
    tag_string.position().y = 0.0175;
    tag.add(&tag_string);
    // Tag body (small rectangle).
    let tag_body = mesh(pg("PackedSuitcases__tag", box_geo!(0.028, 0.032, 0.002)), &tag_mat);
    tag_body.position().y = -0.020;
    tag.add(&tag_body);
    // Tag hole reinforcement (small dark torus).
    let tag_hole = mesh(pg("PackedSuitcases__eyelet", torus_geo!(0.003, 0.0008, 3, 8)), &dark_corner_mat);
    tag_hole.rotation().x = PI / 2.0;
    tag_hole.position().set(0, -0.005, 0.001);
    tag.add(&tag_hole);
    // Text lines on tag (3 thin dark boxes).
    let text_line_mat = lambert!({ color: 0x101010 });
    for i in 0..3 {
        let line = mesh(box_geo!(0.020, 0.001, 0.001), &text_line_mat);
        line.position().set(0, -0.014 - i as f64 * 0.008, 0.002);
        tag.add(&line);
    }
    // Roller: hangs from the pull bar (string top at the bar, y body_h + 0.148).
    // Carry handle: that handle lies folded flat on the lid, so the tag lies on
    // the lid in front of it, its string running back to the handle's -X foot.
    let tag_x = if is_roller { body_w * 0.15 } else { -0.040 };
    let tag_y = if is_roller { body_h + 0.108 } else { body_h + 0.011 };
    tag.position().set(tag_x, tag_y, if is_roller { 0.006 } else { 0.030 });
    if !is_roller {
        tag.rotation().x = -PI / 2.0;
    }
    tag.rotation().z = (rnd() - 0.5) * 0.3;
    s.add(&tag);
    // Optional shipping/travel sticker labels — 60% chance for 2-3 stickers.
    if rnd() < 0.6 {
        let sticker_colors = [0xa02040, 0x2060a0, 0xd0a040, 0x408060];
        let sticker_count = 2 + (rnd() * 2.0).floor() as i32;
        for i in 0..sticker_count {
            let sticker_mat = lambert!({ color: choice(&sticker_colors) });
            let sw = 0.030 + rnd() * 0.020;
            let sh = 0.020 + rnd() * 0.014;
            let sticker = mesh(box_geo!(sw, sh, 0.001), &sticker_mat);
            let px = (rnd() - 0.5) * body_w * 0.7;
            let py = body_h * 0.5 + (rnd() - 0.5) * body_h * 0.6 + 0.010;
            sticker.position().set(px, py, body_d * 0.5 + 0.001);
            sticker.rotation().z = (rnd() - 0.5) * 0.4;
            s.add(&sticker);
        }
    }
    let px = g.wx(x) + (rnd() - 0.5) * 1.6;
    let pz = g.wz(z) + (rnd() - 0.5) * 1.6;
    s.position().set(px, 0, pz);
    s.rotation().y = rnd() * PI * 2.0;
    g.world.add(&s);
}
