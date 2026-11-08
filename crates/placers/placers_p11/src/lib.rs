//! Prop placers: travel and hallway props — packed suitcases, travel documents,
//! glasses cases, wristwatches, opened wallets, jewelry boxes, diplomas,
//! umbrella stands, boot trays, hat racks and grandfather clocks.
#![allow(unused_variables, unused_mut, unused_assignments, unused_imports, dead_code, clippy::all)]
use game::prelude::*;

mod t07;
use t07::{place_boot_trays, place_umbrella_stands};

/// Every placer in this crate, by its registered name.
pub fn register(r: &mut Vec<(&'static str, PlacerFn)>) {
    r.push(("place_packed_suitcases", place_packed_suitcases));
    r.push(("place_travel_documents", place_travel_documents));
    r.push(("place_glasses_cases", place_glasses_cases));
    r.push(("place_wristwatches", place_wristwatches));
    r.push(("place_opened_wallets", place_opened_wallets));
    r.push(("place_jewelry_boxes", place_jewelry_boxes));
    r.push(("place_formal_diplomas", place_formal_diplomas));
    r.push(("place_umbrella_stands", place_umbrella_stands));
    r.push(("place_boot_trays", place_boot_trays));
    r.push(("place_hat_racks", place_hat_racks));
    r.push(("place_grandfather_clocks", place_grandfather_clocks));
}

// ------------------------------------------------------------------ helpers
/// Every open interior tile (`tiles[t_idx(x, z)] == 0`), row by row.
fn open_tiles(g: &Game) -> Vec<(i32, i32)> {
    let mut v = vec![];
    for z in 1..g.h - 1 {
        for x in 1..g.w - 1 {
            if g.tiles[g.t_idx(x, z)] == 0 {
                v.push((x, z));
            }
        }
    }
    v
}

/// Open tiles with their first wall neighbour (`[[1,0],[-1,0],[0,1],[0,-1]]` order).
fn wall_tiles(g: &Game) -> Vec<(i32, i32, i32, i32)> {
    let mut v = vec![];
    for z in 1..g.h - 1 {
        for x in 1..g.w - 1 {
            if g.tiles[g.t_idx(x, z)] == 0 {
                for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                    if g.tiles[g.t_idx(x + dx, z + dz)] == 1 {
                        v.push((x, z, dx, dz));
                        break;
                    }
                }
            }
        }
    }
    v
}

// Packed suitcase w/ hanging tag (floor): 4% chance per level.
fn place_packed_suitcases(g: &mut Game, _dist: &[i32]) {
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

// Passport + boarding pass (floor): 6% chance per level.
game::lazy_static_tex!(
    passport_cover_texture,
    canvas_tex(96, 128, |ctx, w, h| {
        // Solid dark cover base (blue/red/green).
        let covers = ["#1a2b5c", "#5c1a20", "#204028", "#402015"];
        ctx.set_fill_style(choice(&covers));
        ctx.fill_rect(0, 0, w, h);
        // Central emblem — gold circle w/ crossed lines (abstract crest).
        ctx.set_fill_style("#c0a060");
        ctx.begin_path();
        ctx.arc(w / 2.0, h * 0.4, 16, 0, PI * 2.0);
        ctx.stroke();
        ctx.set_stroke_style("#c0a060");
        ctx.set_line_width(2);
        ctx.begin_path();
        ctx.arc(w / 2.0, h * 0.4, 16, 0, PI * 2.0);
        ctx.stroke();
        // Cross pattern inside emblem.
        ctx.begin_path();
        ctx.move_to(w / 2.0 - 8.0, h * 0.4);
        ctx.line_to(w / 2.0 + 8.0, h * 0.4);
        ctx.move_to(w / 2.0, h * 0.4 - 8.0);
        ctx.line_to(w / 2.0, h * 0.4 + 8.0);
        ctx.stroke();
        // Top word: "PASSPORT".
        ctx.set_fill_style("#c0a060");
        ctx.set_font("bold 8px serif");
        ctx.set_text_align("center");
        ctx.fill_text("PASSPORT", w / 2.0, 20);
        // Country/organization line below title.
        ctx.set_font("6px serif");
        let countries = ["UNITED STATES", "REPUBLIC", "FEDERAL", "COMMONWEALTH"];
        ctx.fill_text(choice(&countries), w / 2.0, 30);
        // Below emblem: line for placeholder.
        ctx.set_stroke_style("rgba(192,160,96,0.5)");
        ctx.set_line_width(1);
        ctx.begin_path();
        ctx.move_to(w / 2.0 - 30.0, h * 0.7);
        ctx.line_to(w / 2.0 + 30.0, h * 0.7);
        ctx.stroke();
    })
);

game::lazy_static_tex!(
    boarding_pass_texture,
    canvas_tex(200, 100, |ctx, w, h| {
        // Off-white/cream boarding-pass paper.
        ctx.set_fill_style("#f0e8d0");
        ctx.fill_rect(0, 0, w, h);
        // Airline color band on left (colored strip).
        let airline_colors = ["#a02020", "#2060a0", "#c0a040", "#408060"];
        ctx.set_fill_style(choice(&airline_colors));
        ctx.fill_rect(0, 0, 30, h);
        // Airline name (vertical text-like bar).
        ctx.set_fill_style("#f0e8d0");
        ctx.set_font("bold 7px sans-serif");
        ctx.save();
        ctx.translate(15, h / 2.0);
        ctx.rotate(-PI / 2.0);
        ctx.set_text_align("center");
        let airlines = ["SKY AIR", "PACIFIC", "ATLANTIS", "NORTHERN"];
        ctx.fill_text(choice(&airlines), 0, 0);
        ctx.restore();
        // BOARDING PASS header.
        ctx.set_fill_style("#101010");
        ctx.set_font("bold 10px sans-serif");
        ctx.set_text_align("left");
        ctx.fill_text("BOARDING PASS", 38, 14);
        // Passenger name section.
        ctx.set_font("7px sans-serif");
        ctx.fill_text("PASSENGER", 38, 26);
        ctx.set_font("bold 10px monospace");
        let names = ["SMITH/JOHN", "LEE/SARAH M", "CHEN/DAVID R", "GARCIA/ANNA"];
        ctx.fill_text(choice(&names), 38, 38);
        // Flight details rows.
        ctx.set_font("7px sans-serif");
        ctx.fill_text("FROM", 38, 52);
        ctx.fill_text("TO", 90, 52);
        ctx.fill_text("FLIGHT", 138, 52);
        ctx.fill_text("SEAT", 172, 52);
        ctx.set_font("bold 10px monospace");
        let codes = ["JFK", "ORD", "LAX", "ATL", "DFW", "MIA", "SEA", "BOS"];
        ctx.fill_text(choice(&codes), 38, 64);
        ctx.fill_text(choice(&codes), 90, 64);
        let flight = format!("SA{}", 100 + (rnd() * 900.0).floor() as i64);
        ctx.fill_text(&flight, 138, 64);
        let seat_row = (rnd() * 30.0).floor() as i64 + 1;
        let seat_letter = ["A", "B", "C", "D", "E", "F"][(rnd() * 6.0).floor() as usize];
        ctx.fill_text(&format!("{}{}", seat_row, seat_letter), 172, 64);
        // Date + time.
        ctx.set_font("7px sans-serif");
        ctx.fill_text("DATE", 38, 78);
        ctx.fill_text("DEPART", 90, 78);
        ctx.fill_text("GATE", 148, 78);
        ctx.set_font("bold 9px monospace");
        ctx.fill_text("14 MAR", 38, 90);
        let dep_h = (rnd() * 12.0).floor() as i64 + 6;
        let dep_m = ((rnd() * 6.0).floor() as i64) * 10;
        ctx.fill_text(&format!("{}:{}0", dep_h, dep_m), 90, 90);
        let gate_l = ["A", "B", "C", "D"][(rnd() * 4.0).floor() as usize];
        let gate_n = (rnd() * 30.0).floor() as i64 + 1;
        ctx.fill_text(&format!("{}{}", gate_l, gate_n), 148, 90);
        // Barcode strip on the right — thin dark lines.
        ctx.set_fill_style("#101010");
        for i in 0..40 {
            let bar_w = 0.5 + rnd() * 1.5;
            ctx.fill_rect(180.0 + i as f64 * 1.2, 24, bar_w, 40);
        }
    })
);

fn place_travel_documents(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let td = group();
    // Passport — small box w/ cover texture on top.
    let passport_mat = lambert!({ map: passport_cover_texture() });
    let passport_side_mat = lambert!({ color: 0x102030 });
    let passport_body = mesh(pg("TravelDocuments__passport", box_geo!(0.075, 0.006, 0.100)), &passport_side_mat);
    passport_body.position().y = 0.003;
    td.add(&passport_body);
    // Cover face plane on top.
    let passport_cover = mesh(plane_geo!(0.075, 0.100), &passport_mat);
    passport_cover.rotation().x = -PI / 2.0;
    passport_cover.position().y = 0.0062;
    td.add(&passport_cover);
    // Boarding pass — thin ticket lying beside/on top of passport.
    let pass_mat = lambert!({ map: boarding_pass_texture() });
    let pass_back = lambert!({ color: 0xe8dcc8 });
    let boarding = group();
    let bp_body = mesh(box_geo!(0.16, 0.001, 0.075), &pass_back);
    bp_body.position().y = 0.0005;
    boarding.add(&bp_body);
    let bp_face = mesh(plane_geo!(0.16, 0.075), &pass_mat);
    bp_face.rotation().x = -PI / 2.0;
    bp_face.position().y = 0.0011;
    boarding.add(&bp_face);
    // On the floor, its end tucked under the passport's edge.
    boarding.position().set(0.09, 0, 0.02);
    boarding.rotation().y = -0.4;
    td.add(&boarding);
    // 50% chance: a currency bill peeking out from passport.
    if rnd() < 0.5 {
        let bill_mat = lambert!({ color: 0x60805a });
        let bill = mesh(pg("TravelDocuments__bill", box_geo!(0.05, 0.001, 0.030)), &bill_mat);
        // Resting on the passport cover (top 0.0062).
        bill.position().set(0.030, 0.0068, -0.04);
        bill.rotation().y = 0.3;
        td.add(&bill);
        // Portrait blob on bill (darker oval).
        let portrait_mat = lambert!({ color: 0x304a2a });
        let portrait = mesh(circle_geo!(0.007, 6), &portrait_mat);
        portrait.rotation().x = -PI / 2.0;
        portrait.position().set(0.030, 0.0074, -0.04);
        portrait.rotation().z = 0.3;
        td.add(&portrait);
    }
    // 40% chance: small pen lying nearby.
    if rnd() < 0.4 {
        let pen_colors = [0x101010, 0x2060a0, 0x603030];
        let pen_mat = lambert!({ color: choice(&pen_colors) });
        let pen = mesh(pg("TravelDocuments__pen", cyl_geo!(0.003, 0.003, 0.10, 6)), &pen_mat);
        // On the floor beside the passport. The barrel's narrow -Y end (the
        // writing end) points along -X, then everything is yawed -0.4.
        pen.rotation().z = -PI / 2.0;
        pen.position().set(-0.088, 0.003, -0.06);
        pen.rotation().y = -0.4;
        td.add(&pen);
        let pen_tip = mesh(pg("TravelDocuments__penTip", cone_geo!(0.002, 0.008, 5)), &pen_mat);
        // Apex out along the pen axis, base 0.5 mm into the writing end.
        pen_tip.rotation().z = PI / 2.0;
        pen_tip.rotation().y = -0.4;
        pen_tip.position().set(-0.088 - 0.0535 * 0.4f64.cos(), 0.003, -0.06 - 0.0535 * 0.4f64.sin());
        td.add(&pen_tip);
    }
    let px = g.wx(x) + (rnd() - 0.5) * 2.2;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.2;
    td.position().set(px, 0, pz);
    td.rotation().y = rnd() * PI * 2.0;
    g.world.add(&td);
}

// Eyeglasses case opened w/ cleaning cloth (floor): 6% chance per level.
fn place_glasses_cases(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let case_colors = [0x101010, 0x603020, 0x2060a0, 0x804a20, 0xa02040];
    let case_mat = lambert!({ color: choice(&case_colors) });
    let lining_mat = lambert!({ color: 0xd0c0a0 });
    let cloth_colors = [0xa0d0e0, 0xf0f0f0, 0xe0c0d0, 0xd0f0a0, 0xf0e0a0];
    let cloth_mat = lambert!({ color: choice(&cloth_colors) });
    let gc = group();
    // Bottom half of case — flat with velvet lining inside.
    let (case_w, case_d, case_h) = (0.15, 0.07, 0.020);
    let bottom_half = mesh(pg("GlassesCases__bottom", box_geo!(case_w, case_h, case_d)), &case_mat);
    bottom_half.position().y = case_h / 2.0 + 0.002;
    gc.add(&bottom_half);
    // Lining inside bottom half — soft cream disc.
    let lining = mesh(pg("GlassesCases__lining", box_geo!(case_w - 0.010, 0.003, case_d - 0.010)), &lining_mat);
    lining.position().y = case_h + 0.003;
    gc.add(&lining);
    // Top half tilted open — hinged upward at back, swinging about the
    // hinge line on the base's back top edge.
    let lid_pivot = group();
    lid_pivot.position().set(0, case_h + 0.002, -case_d * 0.5);
    lid_pivot.rotation().x = -PI / 3.0;
    gc.add(&lid_pivot);
    let top_half = mesh(pg("GlassesCases__lid", box_geo!(case_w, case_h * 0.7, case_d)), &case_mat);
    top_half.position().set(0, case_h * 0.35, case_d * 0.5);
    lid_pivot.add(&top_half);
    // Interior of top half (lining visible): in the lid's hollow.
    let top_lining = mesh(pg("GlassesCases__lining", box_geo!(case_w - 0.010, 0.003, case_d - 0.010)), &lining_mat);
    top_lining.position().set(0, case_h * 0.35 + 0.0033, case_d * 0.5); // hollow ceiling at lid-local 0.0048
    lid_pivot.add(&top_lining);
    // Hinge line (small dark cylinder), on the hinge axis.
    let hinge = mesh(pg("GlassesCases__hinge", cyl_geo!(0.003, 0.003, case_w * 0.85, 5)), lambert!({ color: 0x606060 }));
    hinge.rotation().z = PI / 2.0;
    hinge.position().set(0, case_h + 0.002, -case_d * 0.5);
    gc.add(&hinge);
    // Front snap clasp (tiny dark box).
    let clasp = mesh(pg("GlassesCases__clasp", box_geo!(0.006, 0.004, 0.002)), lambert!({ color: 0xc0a060 }));
    clasp.position().set(0, case_h * 0.6, case_d * 0.5 + 0.002);
    gc.add(&clasp);
    // Cleaning cloth — small square folded lying beside/inside.
    let cloth = mesh(pg("GlassesCases__cloth", box_geo!(0.05, 0.002, 0.05)), &cloth_mat);
    cloth.position().set(case_w * 0.5 + 0.04, 0.005, case_d * 0.1);
    cloth.rotation().y = 0.3;
    cloth.rotation().z = 0.15;
    gc.add(&cloth);
    // Second folded cloth corner for wrinkled effect.
    let cloth2 = mesh(pg("GlassesCases__clothFold", box_geo!(0.03, 0.003, 0.03)), &cloth_mat);
    cloth2.position().set(case_w * 0.5 + 0.03, 0.008, case_d * 0.14);
    cloth2.rotation().y = 0.4;
    cloth2.rotation().z = -0.2;
    gc.add(&cloth2);
    // 50% chance: folded eyeglasses inside case lining.
    if rnd() < 0.5 {
        let frame_colors = [0x101010, 0x604020, 0xa02040, 0xc0a040];
        let glass_frame_mat = lambert!({ color: choice(&frame_colors) });
        let lens_mat = basic_mat!({ color: 0xf0f0f8, transparent: true, opacity: 0.4 });
        // 2 lens rings side-by-side inside case.
        for side in [-1.0f64, 1.0] {
            let ring = mesh(pg("GlassesCases__rim", torus_geo!(0.012, 0.0015, 3, 10)), &glass_frame_mat);
            ring.rotation().x = PI / 2.0;
            ring.position().set(side * 0.015, case_h + 0.007, 0);
            gc.add(&ring);
            // Lens disc.
            let lens = mesh(circle_geo!(0.011, 10), &lens_mat);
            lens.rotation().x = -PI / 2.0;
            lens.position().set(side * 0.015, case_h + 0.008, 0);
            gc.add(&lens);
        }
        // Bridge.
        let bridge = mesh(pg("GlassesCases__bridge", box_geo!(0.008, 0.002, 0.002)), &glass_frame_mat);
        bridge.position().set(0, case_h + 0.008, -0.004);
        gc.add(&bridge);
        // Folded temples: each starts at its rim's outer top corner and folds
        // inward across the rims, lying on them.
        for side in [-1.0f64, 1.0] {
            let temple = mesh(pg("GlassesCases__temple", cyl_geo!(0.001, 0.001, 0.028, 3)), &glass_frame_mat);
            temple.rotation().z = side * PI / 2.0;
            temple.rotation().y = side * 0.2;
            temple.position().set(side * (0.026 - 0.014 * 0.2f64.cos()), case_h + 0.0095, -0.004 + 0.014 * 0.2f64.sin());
            gc.add(&temple);
        }
    }
    let px = g.wx(x) + (rnd() - 0.5) * 2.2;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.2;
    gc.position().set(px, 0, pz);
    gc.rotation().y = rnd() * PI * 2.0;
    g.world.add(&gc);
}

// Watch on wristband (floor): 7% chance per level. A 38 mm quartz
// wristwatch (Blender batch r14b; the Timex Weekender class) lying on its
// back, unbuckled, its two-piece strap lying out from the lugs, its tip
// curled: the case plated, worn through at the edges; a domed acrylic
// crystal, scratched; a printed dial, its hands where they stopped.
fn place_wristwatches(g: &mut Game, _dist: &[i32]) {
    use game::assets::props::prop_part_as_modelled;
    g.pp.wristwatch_positions = vec![]; // reset per level
    if rnd() < 0.93 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    // the old colour draws, call for call: the strap, the case, the dial
    let band_colors = [0x101010, 0x603020, 0xa0a0a8, 0xc0a060, 0x201050, 0x804a20];
    let band_c = choice(&band_colors);
    let band = band_colors.iter().position(|&c| c == band_c).unwrap_or(0);
    let case_colors = [0xc0a060, 0xa0a0a8, 0xe0d8c0, 0x606060];
    let case_c = choice(&case_colors);
    let case_k = case_colors.iter().position(|&c| c == case_c).unwrap_or(0);
    let face_colors = [0xf0e8d0, 0x101010, 0xf0f0f0, 0x2060a0];
    let face_c = choice(&face_colors);
    let dial = face_colors.iter().position(|&c| c == face_c).unwrap_or(0);
    // where the hands stopped (the old hands' draws): the hour from its own
    // draw, carried on by the minutes; the second hand
    let hour_ang = rnd() * PI * 2.0;
    let min_ang = rnd() * PI * 2.0;
    let sec_ang = rnd() * PI * 2.0;
    // Stepped-tick second hand — 40% chance ("still ticking").
    let is_ticking = rnd() < 0.4;
    let px = g.wx(x) + (rnd() - 0.5) * 2.4;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.4;
    let yaw = rnd() * PI * 2.0;
    let frac = |a: f64| ((a + PI / 2.0) / (PI * 2.0)).rem_euclid(1.0);
    let minutes = frac(min_ang);
    let hours = ((frac(hour_ang) * 12.0).floor() + minutes) / 12.0;
    let seconds = frac(sec_ang);
    let seed = r14b_kit::seed_of(&[px, pz]);
    let wg = group();
    // the case: plating over brass (dark, its colour in its highlight)
    let (col, spec, shin) =
        [(0x4a3c22, 0x8c7448, 54.0), (0x404042, 0x8e8e90, 70.0), (0x58564f, 0x6c6a64, 26.0), (0x1e1e20, 0x46464a, 40.0)][case_k];
    let metal = phong!({ color: col, specular: spec, shininess: shin });
    {
        let mut mm = metal.m();
        mm.map = Some(r14b_kit::plating_tex(case_k, seed));
    }
    if let Some(geo) = prop_part_as_modelled("Real__watchCase") {
        wg.add(&mesh(geo, &metal));
    }
    if let Some(geo) = prop_part_as_modelled("Real__watchStrap") {
        let bb = geo.bounding_box();
        let (z0, z1) = (bb.min.z, bb.max.z);
        let strap = mesh(geo, r14b_kit::strap_mat(band, z0, z1, seed));
        wg.add(&strap);
    }
    // the dial, printed, under its hands
    let face = mesh(circle_geo!(0.0158, 48), lambert!({ map: &r14b_kit::dial_tex(dial, seed) }));
    face.rotation().x = -PI / 2.0;
    face.position().y = 0.0061;
    wg.add(&face);
    let dark = dial == 1 || dial == 3;
    let hand_mat = if dark { lambert!({ color: 0xcfc8b2 }) } else { lambert!({ color: 0x161514 }) };
    let hand = |name: &str, y: f64, frac: f64, mat: &Mat| {
        let pivot = group();
        if let Some(geo) = prop_part_as_modelled(name) {
            pivot.add(&mesh(geo, mat));
        }
        pivot.position().y = y;
        pivot.rotation().y = -frac * PI * 2.0;
        wg.add(&pivot);
        pivot
    };
    hand("Real__watchHour", 0.00618, hours, &hand_mat);
    hand("Real__watchMinute", 0.00636, minutes, &hand_mat);
    let second = hand("Real__watchSecond", 0.00654, seconds, &lambert!({ color: if dark { 0xa0402c } else { 0x7a2218 } }));
    if let Some(geo) = prop_part_as_modelled("Real__watchCrystal") {
        let crystal = mesh(geo, r14b_kit::crystal_mat(&r14b_kit::crystal_dirt_tex(seed)));
        crystal.set_render_order(2);
        wg.add(&crystal);
    }
    if is_ticking {
        // still going: the second hand steps once a second
        let mut ang = -seconds * PI * 2.0;
        let mut timer = 0.0;
        g.animate(move |_g, dt, _t| {
            timer += dt;
            if timer >= 1.0 {
                timer -= 1.0;
                ang -= PI * 2.0 / 60.0;
                second.rotation().y = ang;
            }
        });
    }
    wg.user().set("isTicking", is_ticking);
    wg.position().set(px, 0, pz);
    wg.rotation().y = yaw;
    g.world.add(&wg);
    // Register position only for ticking watches.
    if is_ticking {
        let p = wg.pos();
        g.pp.wristwatch_positions.push(v3(p.x, 0, p.z));
    }
}

// Opened wallet w/ credit cards fanned (floor): 5% chance per level.
fn place_opened_wallets(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let leather_colors = [0x603020, 0x201010, 0x8b5a2b, 0x404040, 0x502010, 0x101010];
    let leather_mat = lambert!({ color: choice(&leather_colors) });
    let wg = group();
    // Bi-fold layout — laid flat open with visible interior card slots.
    let (half_w, half_d) = (0.055, 0.09);
    let left_half = mesh(pg("OpenedWallets__leftHalf", box_geo!(half_w, 0.005, half_d)), &leather_mat);
    left_half.position().set(-half_w / 2.0, 0.0025, 0);
    wg.add(&left_half);
    let right_half = mesh(pg("OpenedWallets__rightHalf", box_geo!(half_w, 0.005, half_d)), &leather_mat);
    right_half.position().set(half_w / 2.0, 0.0025, 0);
    wg.add(&right_half);
    // Spine bump.
    let spine = mesh(pg("OpenedWallets__spine", box_geo!(0.004, 0.007, half_d)), &leather_mat);
    spine.position().set(0, 0.004, 0);
    wg.add(&spine);
    // Card slot dividers visible on left half — 3 horizontal thin ridges.
    for i in 0..3 {
        let slot_rim = mesh(pg("OpenedWallets__slotLip", box_geo!(half_w * 0.85, 0.0015, 0.003)), &leather_mat);
        slot_rim.position().set(-half_w / 2.0, 0.006, -half_d * 0.30 + i as f64 * half_d * 0.20);
        wg.add(&slot_rim);
    }
    // Cash slot fold on right half (horizontal seam).
    let cash_slot = mesh(pg("OpenedWallets__cashPocket", box_geo!(half_w * 0.85, 0.0015, half_d * 0.85)), &leather_mat);
    cash_slot.position().set(half_w / 2.0, 0.006, 0);
    wg.add(&cash_slot);
    // 3-5 credit cards fanned out from left slots.
    let card_colors = [
        0x2060a0, // blue
        0x606060, // silver
        0xa08040, // gold
        0x101010, // black
        0xa02040, // red
        0x408060, // green
        0xc0a060, // rose gold
    ];
    let card_count = 3 + (rnd() * 3.0).floor() as i32;
    // Fan pivot near left half's edge.
    let fan_pivot_x = -half_w / 2.0 + half_w * 0.35;
    let fan_pivot_z = 0.0;
    let fan_spread = 1.3; // Total angular spread across all cards.
    for i in 0..card_count {
        let card_mat = lambert!({ color: choice(&card_colors) });
        let card = group();
        // Card body.
        let card_body = mesh(pg("OpenedWallets__card", box_geo!(0.085, 0.001, 0.055)), &card_mat);
        card_body.position().y = 0.0005;
        card.add(&card_body);
        // Chip square top-left of card (gold rectangle).
        let chip_mat = lambert!({ color: 0xd8b060 });
        let chip = mesh(pg("OpenedWallets__chip", box_geo!(0.010, 0.0015, 0.008)), &chip_mat);
        chip.position().set(-0.025, 0.0012, -0.015);
        card.add(&chip);
        // Bank/network logo indicator (small colored circle).
        let logo_colors = [0xa02040, 0x2060a0, 0xd0a040, 0x606060];
        let logo_mat = lambert!({ color: choice(&logo_colors) });
        let logo = mesh(circle_geo!(0.005, 6), &logo_mat);
        logo.rotation().x = -PI / 2.0;
        logo.position().set(0.030, 0.0013, -0.015);
        card.add(&logo);
        // Card number embossed (thin light bar).
        let emboss_mat = lambert!({ color: 0xf0e8d0 });
        let emboss = mesh(pg("OpenedWallets__emboss", box_geo!(0.060, 0.0012, 0.003)), &emboss_mat);
        emboss.position().set(0, 0.0013, 0.010);
        card.add(&emboss);
        // Position fan-style — angular spread + slight radial offset.
        let ang_t = (i as f64 / (card_count - 1) as f64) - 0.5; // -0.5..0.5
        let card_angle = ang_t * fan_spread;
        let card_radius = 0.04 + i as f64 * 0.005; // Progressive outward stack.
        card.position().set(
            fan_pivot_x + (card_angle - PI / 2.0).cos() * card_radius,
            0.006 + i as f64 * 0.0012,
            fan_pivot_z + (card_angle - PI / 2.0).sin() * card_radius,
        );
        card.rotation().y = card_angle;
        wg.add(&card);
    }
    // 60% chance: single cash bill peeking from right slot.
    if rnd() < 0.6 {
        let bill_mat = lambert!({ color: 0x60805a });
        let bill = mesh(pg("OpenedWallets__bill", box_geo!(0.045, 0.001, 0.025)), &bill_mat);
        bill.position().set(half_w / 2.0 + 0.02, 0.008, half_d * 0.20);
        bill.rotation().y = -0.3;
        wg.add(&bill);
        // Portrait blob.
        let portrait = mesh(circle_geo!(0.006, 5), lambert!({ color: 0x304a2a }));
        portrait.rotation().x = -PI / 2.0;
        portrait.position().set(half_w / 2.0 + 0.02, 0.0086, half_d * 0.20);
        portrait.rotation().z = -0.3;
        wg.add(&portrait);
    }
    // Photo slot on inside w/ tiny blur figure — 40% chance.
    if rnd() < 0.4 {
        let photo_frame_mat = lambert!({ color: 0xf0e8c0 });
        let photo_frame = mesh(box_geo!(half_w * 0.6, 0.001, half_d * 0.4), &photo_frame_mat);
        photo_frame.position().set(-half_w / 2.0, 0.008, -half_d * 0.05);
        wg.add(&photo_frame);
        // Tiny sepia figure blob.
        let fig_mat = lambert!({ color: 0x604030 });
        let fig = mesh(circle_geo!(half_w * 0.15, 5), &fig_mat);
        fig.rotation().x = -PI / 2.0;
        fig.position().set(-half_w / 2.0, 0.0085, -half_d * 0.05);
        wg.add(&fig);
    }
    let px = g.wx(x) + (rnd() - 0.5) * 2.2;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.2;
    wg.position().set(px, 0, pz);
    wg.rotation().y = rnd() * PI * 2.0;
    g.world.add(&wg);
}

// Opened jewelry box w/ spilled contents (floor): 4% chance per level.
fn place_jewelry_boxes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let box_colors = [0x603020, 0x201010, 0x101020, 0xa02040, 0x804a20];
    let box_mat = lambert!({ color: choice(&box_colors) });
    let lining_colors = [0xa02020, 0x2060a0, 0x101010, 0x804060, 0x606060];
    let lining_mat = lambert!({ color: choice(&lining_colors) });
    let gold_mat = lambert!({ color: 0xc0a060 });
    let silver_mat = lambert!({ color: 0xd0d0d8 });
    let jb = group();
    // Bottom half — hinged box (larger than the glasses case).
    let (box_w, box_d, box_h) = (0.16, 0.11, 0.05);
    let bottom = mesh(pg("JewelryBoxes__base", box_geo!(box_w, box_h, box_d)), &box_mat);
    bottom.position().y = box_h / 2.0 + 0.002;
    jb.add(&bottom);
    // Interior lining (velvet — colored lining pool).
    let inside_lining = mesh(pg("JewelryBoxes__pad", box_geo!(box_w - 0.012, 0.004, box_d - 0.012)), &lining_mat);
    inside_lining.position().y = box_h + 0.002;
    jb.add(&inside_lining);
    // Compartment dividers on inside (small ridges).
    for dx in [-box_w * 0.25, box_w * 0.25] {
        let divider = mesh(pg("JewelryBoxes__divider", box_geo!(0.002, 0.006, box_d - 0.014)), &box_mat);
        divider.position().set(dx, box_h + 0.005, 0);
        jb.add(&divider);
    }
    let horiz_divider = mesh(pg("JewelryBoxes__crossDivider", box_geo!(box_w - 0.014, 0.006, 0.002)), &box_mat);
    horiz_divider.position().set(0, box_h + 0.005, 0);
    jb.add(&horiz_divider);
    // Top half tilted open — swings about the hinge line on the base's back top edge.
    let lid_pivot = group();
    lid_pivot.position().set(0, box_h + 0.002, -box_d * 0.5);
    lid_pivot.rotation().x = -PI / 3.0;
    jb.add(&lid_pivot);
    let top = mesh(pg("JewelryBoxes__lid", box_geo!(box_w, box_h * 0.5, box_d)), &box_mat);
    top.position().set(0, box_h * 0.25, box_d * 0.5);
    lid_pivot.add(&top);
    // Interior lining top: against the ceiling of the lid's hollow underside.
    let top_lining = mesh(pg("JewelryBoxes__lidLining", box_geo!(box_w - 0.012, 0.003, box_d - 0.012)), &lining_mat);
    top_lining.position().set(0, box_h * 0.25 + 0.0008, box_d * 0.5);
    lid_pivot.add(&top_lining);
    // Small dark hinge, on the hinge axis.
    let hinge = mesh(pg("JewelryBoxes__hinge", cyl_geo!(0.003, 0.003, box_w * 0.85, 5)), lambert!({ color: 0x606060 }));
    hinge.rotation().z = PI / 2.0;
    hinge.position().set(0, box_h + 0.002, -box_d * 0.5);
    jb.add(&hinge);
    // Small gold clasp on front.
    let clasp = mesh(pg("JewelryBoxes__clasp", box_geo!(0.010, 0.006, 0.003)), &gold_mat);
    clasp.position().set(0, box_h * 0.7, box_d * 0.5 + 0.002);
    jb.add(&clasp);
    // 2-3 rings (small tori).
    let ring_count = 2 + (rnd() * 2.0).floor() as i32;
    for i in 0..ring_count {
        let is_gold = rnd() < 0.5;
        let ring_mat2 = if is_gold { &gold_mat } else { &silver_mat };
        let ring = mesh(pg("JewelryBoxes__ring", torus_geo!(0.010, 0.002, 3, 10)), ring_mat2);
        ring.rotation().x = PI / 2.0 + (rnd() - 0.5) * 0.6;
        ring.rotation().z = (rnd() - 0.5) * 0.6;
        let rx = -box_w * 0.3 + (rnd() - 0.5) * 0.04;
        let rz = -box_d * 0.2 + (rnd() - 0.5) * 0.04;
        ring.position().set(rx, box_h + 0.008, rz);
        jb.add(&ring);
        // 40% ring w/ stone.
        if rnd() < 0.4 {
            let stone_colors = [0xa02020, 0x2060a0, 0x408060, 0xc0a0d0, 0xf0f0f0];
            let stone_mat = lambert!({ color: choice(&stone_colors) });
            let stone = mesh(pg("JewelryBoxes__stone", octa_geo!(0.004, 0)), &stone_mat);
            stone.position().copy(&ring.pos());
            stone.position().y += 0.004;
            jb.add(&stone);
        }
    }
    // Necklace — 8-segment chain trailing out of box.
    let necklace_colors = [0xc0a060, 0xd0d0d8, 0xf0e0a0];
    let necklace_mat = lambert!({ color: choice(&necklace_colors) });
    // Starts on the pad in the +X compartment, drapes over the +X rim and
    // falls down the outside onto the floor.
    let chain_start = v3(box_w * 0.30, box_h + 0.006, box_d * 0.15);
    let chain_end = v3(box_w * 0.85, 0.0013, box_d * 0.30);
    let chain_ys = [box_h + 0.006, box_h + 0.006, box_h + 0.0045, 0.030, 0.010, 0.0025, 0.0013, 0.0013];
    let chain_segs = 8;
    let mut prev = chain_start;
    for i in 0..chain_segs {
        let t = (i + 1) as f64 / chain_segs as f64;
        let arc_amt = (t * PI).sin() * 0.02;
        let cx = chain_start.x + (chain_end.x - chain_start.x) * t;
        let cy = chain_ys[i];
        let cz = chain_start.z + (chain_end.z - chain_start.z) * t + (t * PI * 2.0).cos() * arc_amt * 0.6;
        let next = v3(cx, cy, cz);
        let dir_len = next.distance_to(&prev);
        if dir_len > 0.001 {
            let seg = mesh(pg("JewelryBoxes__chain", cyl_geo!(0.0012, 0.0012, dir_len, 3)), &necklace_mat);
            let mut mid = prev;
            mid.add(&next).multiply_scalar(0.5);
            seg.position().copy(&mid);
            let mut v = next;
            v.sub(&prev).normalize();
            seg.quaternion().set_from_unit_vectors(&v3(0, 1, 0), &v);
            jb.add(&seg);
        }
        prev = next;
    }
    // Pendant at chain end (small teardrop / gem).
    let pendant_colors = [0xa02020, 0x2060a0, 0x408060, 0xc0a0d0, 0xf0f0f0];
    let pendant_mat = lambert!({ color: choice(&pendant_colors) });
    let pendant = mesh(pg("JewelryBoxes__pendant", octa_geo!(0.006, 0)), &pendant_mat);
    pendant.position().copy(&prev);
    pendant.position().y += 0.005;
    jb.add(&pendant);
    // 60% chance: pair of earrings scattered nearby.
    if rnd() < 0.6 {
        for i in 0..2 {
            let earring_mat = &necklace_mat;
            let earring = mesh(pg("JewelryBoxes__earring", torus_geo!(0.005, 0.0008, 3, 6)), earring_mat);
            let ear_tilt = (rnd() - 0.5) * 0.4;
            earring.rotation().x = PI / 2.0 + ear_tilt;
            let ex = -box_w * 0.4 - rnd() * 0.05;
            let ey = 0.005 * ear_tilt.sin().abs() + 0.0008; // low edge on the floor
            let ez = box_d * 0.4 + (rnd() - 0.5) * 0.06;
            earring.position().set(ex, ey, ez);
            jb.add(&earring);
            // Small stone dangling below (resting on the floor).
            let ear_stone = mesh(pg("JewelryBoxes__earStone", octa_geo!(0.003, 0)), &pendant_mat);
            ear_stone.position().copy(&earring.pos());
            ear_stone.position().y = 0.003;
            ear_stone.position().z += 0.004;
            jb.add(&ear_stone);
        }
    }
    let px = g.wx(x) + (rnd() - 0.5) * 2.0;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.0;
    jb.position().set(px, 0, pz);
    jb.rotation().y = rnd() * PI * 2.0;
    g.world.add(&jb);
}

// Framed diploma (wall).
game::lazy_static_tex!(
    diploma_texture_2,
    canvas_tex(200, 260, |ctx, w, h| {
        // Parchment background — aged cream.
        ctx.set_fill_style("#f0e4c8");
        ctx.fill_rect(0, 0, w, h);
        // Aging patches.
        ctx.set_fill_style("rgba(160,120,80,0.12)");
        for _ in 0..40 {
            ctx.begin_path();
            let ax = rnd() * w;
            let ay = rnd() * h;
            let ar = 6.0 + rnd() * 10.0;
            ctx.arc(ax, ay, ar, 0, PI * 2.0);
            ctx.fill();
        }
        // Ornate border — double frame lines + corner flourishes.
        ctx.set_stroke_style("#604020");
        ctx.set_line_width(3);
        ctx.stroke_rect(10, 10, w - 20.0, h - 20.0);
        ctx.set_line_width(1);
        ctx.stroke_rect(16, 16, w - 32.0, h - 32.0);
        // 4 corner ornaments (small filled diamonds).
        ctx.set_fill_style("#604020");
        let corner_xs = [16.0, w - 16.0];
        let corner_ys = [16.0, h - 16.0];
        for cx in corner_xs {
            for cy in corner_ys {
                ctx.begin_path();
                ctx.move_to(cx, cy - 6.0);
                ctx.line_to(cx + 6.0, cy);
                ctx.line_to(cx, cy + 6.0);
                ctx.line_to(cx - 6.0, cy);
                ctx.close_path();
                ctx.fill();
            }
        }
        // Top header — institution name.
        ctx.set_fill_style("#302010");
        ctx.set_font("bold 14px serif");
        ctx.set_text_align("center");
        let institutions = ["UNIVERSITY OF ARTS", "STATE COLLEGE", "ROYAL INSTITUTE", "ACADEMY OF SCIENCES", "MEMORIAL UNIVERSITY"];
        ctx.fill_text(choice(&institutions), w / 2.0, 40);
        // "This certifies that" line.
        ctx.set_font("italic 10px serif");
        ctx.fill_text("This is to certify that", w / 2.0, 68);
        // Recipient name — large cursive.
        ctx.set_font("bold italic 22px cursive");
        ctx.set_fill_style("#101010");
        let names = [
            "Sarah Michelle Anderson",
            "David James Chen",
            "Emma Rose Williams",
            "Michael Thomas Garcia",
            "Anna Grace Rodriguez",
            "James David Lee",
        ];
        ctx.fill_text(choice(&names), w / 2.0, 96);
        // "has completed" line.
        ctx.set_font("italic 10px serif");
        ctx.set_fill_style("#302010");
        ctx.fill_text("has satisfactorily completed the requirements for", w / 2.0, 118);
        // Degree name.
        ctx.set_font("bold 14px serif");
        let degrees = ["Bachelor of Arts", "Master of Science", "Doctor of Philosophy", "Bachelor of Science", "Master of Fine Arts"];
        ctx.fill_text(choice(&degrees), w / 2.0, 140);
        // Field of study.
        ctx.set_font("italic 12px serif");
        let fields = ["in Mathematics", "in English Literature", "in Chemistry", "in History", "in Music Performance", "in Biology"];
        ctx.fill_text(choice(&fields), w / 2.0, 158);
        // Date line.
        ctx.set_font("10px serif");
        let yr = 1965 + (rnd() * 30.0).floor() as i64;
        ctx.fill_text(&format!("Awarded this day, {}", yr), w / 2.0, 184);
        // Simulated signature lines — 2 short lines w/ wavy scripts + labels.
        ctx.set_stroke_style("rgba(30,20,10,0.85)");
        ctx.set_line_width(1.2);
        for sig in 0..2 {
            let sy = 208.0 + sig as f64 * 22.0;
            let sx = w * (0.25 + sig as f64 * 0.4);
            ctx.begin_path();
            ctx.move_to(sx - 25.0, sy);
            ctx.quadratic_curve_to(sx - 15.0, sy - 5.0, sx - 5.0, sy + 2.0);
            ctx.quadratic_curve_to(sx + 5.0, sy - 3.0, sx + 25.0, sy + 1.0);
            ctx.stroke();
            ctx.set_stroke_style("#302010");
            ctx.set_line_width(0.5);
            ctx.begin_path();
            ctx.move_to(sx - 30.0, sy + 6.0);
            ctx.line_to(sx + 30.0, sy + 6.0);
            ctx.stroke();
            ctx.set_fill_style("#302010");
            ctx.set_font("7px serif");
            ctx.fill_text(if sig == 0 { "Dean" } else { "President" }, sx, sy + 14.0);
            ctx.set_stroke_style("rgba(30,20,10,0.85)");
            ctx.set_line_width(1.2);
        }
        // Wax seal — red circle w/ inner detail.
        ctx.set_fill_style("#a02020");
        ctx.begin_path();
        ctx.arc(w - 40.0, h - 40.0, 18, 0, PI * 2.0);
        ctx.fill();
        // Seal star pattern.
        ctx.set_stroke_style("#602010");
        ctx.set_line_width(1);
        ctx.begin_path();
        for i in 0..5 {
            let ang = (i as f64 / 5.0) * PI * 2.0 - PI / 2.0;
            let (r1, r2) = (12.0, 5.0);
            let outer = if i % 2 == 0 { r1 } else { r2 };
            let px = w - 40.0 + ang.cos() * outer;
            let py = h - 40.0 + ang.sin() * outer;
            if i == 0 {
                ctx.move_to(px, py);
            } else {
                ctx.line_to(px, py);
            }
        }
        ctx.stroke();
    })
);

fn place_formal_diplomas(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    }
    let walls = wall_tiles(g);
    if walls.is_empty() {
        return;
    }
    let (x, z, dx, dz) = walls[(rnd() * walls.len() as f64).floor() as usize];
    let frame_colors = [0x604020, 0xd0a040, 0x202020, 0x8b5a2b];
    let frame_mat = lambert!({ color: choice(&frame_colors) });
    let dip_mat = lambert!({ map: diploma_texture_2() });
    let dp = group();
    let mount_y = 1.55;
    let (frame_w, frame_h) = (0.22, 0.28);
    let frame = mesh(pg("FormalDiplomas__frame", box_geo!(frame_w, frame_h, 0.014)), &frame_mat);
    frame.position().set(0, mount_y, 0.011);
    dp.add(&frame);
    // Gold inner mat.
    let gold_mat = lambert!({ color: 0xc0a060 });
    let mat = mesh(pg("FormalDiplomas__mat", box_geo!(frame_w - 0.012, frame_h - 0.012, 0.002)), &gold_mat);
    mat.position().set(0, mount_y, 0.018);
    dp.add(&mat);
    let dip = mesh(plane_geo!(frame_w - 0.030, frame_h - 0.030), &dip_mat);
    dip.position().set(0, mount_y, 0.020);
    dp.add(&dip);
    dp.rotation().y = (-(dx as f64)).atan2(-(dz as f64));
    let px = g.wx(x) + dx as f64 * (CELL * 0.5 - 0.015);
    let pz = g.wz(z) + dz as f64 * (CELL * 0.5 - 0.015);
    dp.position().set(px, 0, pz);
    dp.rotation().z = (rnd() - 0.5) * 0.03;
    g.world.add(&dp);
}

// Umbrella stand, boot tray: batch `t07` (t07.rs).

// Wall hat rack w/ 2-3 hats (wall): 3% chance per level.
fn place_hat_racks(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    }
    let walls = wall_tiles(g);
    if walls.is_empty() {
        return;
    }
    let (x, z, dx, dz) = walls[(rnd() * walls.len() as f64).floor() as usize];
    let wood_colors = [0x8b5a2b, 0x604020, 0xa07040, 0x504030];
    let wood_mat = lambert!({ color: choice(&wood_colors) });
    let brass_mat = lambert!({ color: 0xc0a060 });
    let rack = group();
    let mount_y = 1.85;
    // Horizontal rail.
    let rail_w = 0.50;
    let rail = mesh(pg("HatRacks__rail", box_geo!(rail_w, 0.020, 0.030)), &wood_mat);
    rail.position().set(0, mount_y, 0.015);
    rack.add(&rail);
    // 4 pegs.
    let mut peg_xs: Vec<f64> = vec![];
    for i in 0..4 {
        let px = -rail_w * 0.375 + i as f64 * (rail_w * 0.25);
        peg_xs.push(px);
        let peg = mesh(pg("HatRacks__peg", cyl_geo!(0.004, 0.004, 0.028, 5)), &brass_mat);
        peg.rotation().x = PI / 2.0;
        peg.position().set(px, mount_y, 0.034);
        rack.add(&peg);
        let knob = mesh(pg("HatRacks__knob", sphere_geo!(0.006, 5, 4)), &brass_mat);
        knob.position().set(px, mount_y - 0.002, 0.048);
        rack.add(&knob);
    }
    // Hang 2-3 hats on random pegs (dedup).
    let hat_colors = [0x101010, 0x604020, 0x804a20, 0x201050, 0xa02040, 0x606060, 0xf0e0d0];
    let hat_count = 2 + (rnd() * 2.0).floor() as i32;
    let mut used_pegs: Vec<usize> = vec![];
    for _h in 0..hat_count {
        let mut peg_idx;
        let mut attempts = 0;
        loop {
            peg_idx = (rnd() * peg_xs.len() as f64).floor() as usize;
            attempts += 1;
            if !(used_pegs.contains(&peg_idx) && attempts < 10) {
                break;
            }
        }
        if used_pegs.contains(&peg_idx) {
            break;
        }
        used_pegs.push(peg_idx);
        let px = peg_xs[peg_idx];
        let hat_mat = lambert!({ color: choice(&hat_colors) });
        // Hat type: 50/50 fedora or ball cap.
        let is_fedora = rnd() < 0.5;
        let hat = group();
        if is_fedora {
            // Fedora — brim + crown + optional band.
            let brim = mesh(pg("HatRacks__brim", cyl_geo!(0.075, 0.075, 0.008, 12)), &hat_mat);
            brim.position().y = 0.0;
            hat.add(&brim);
            let crown = mesh(pg("HatRacks__crown", cyl_geo!(0.045, 0.048, 0.055, 10)), &hat_mat);
            crown.position().y = 0.032;
            hat.add(&crown);
            // Pinch/dent — small dark box on top.
            let dent = mesh(pg("HatRacks__dent", box_geo!(0.010, 0.005, 0.045)), lambert!({ color: 0x0a0a0a }));
            dent.position().y = 0.057;
            hat.add(&dent);
            // Band — thin torus ring around base of crown.
            let band_mat = lambert!({ color: 0x1a1a1a });
            let band = mesh(pg("HatRacks__band", torus_geo!(0.047, 0.005, 3, 12)), &band_mat);
            band.rotation().x = PI / 2.0;
            band.position().y = 0.010;
            hat.add(&band);
        } else {
            // Baseball cap — dome + brim.
            let dome = mesh(pg("HatRacks__capDome", sphere_geo!(0.050, 8, 6, 0, PI * 2.0, 0, PI / 2.0)), &hat_mat);
            dome.position().y = 0.008;
            hat.add(&dome);
            // Brim — flat half-disc projecting forward.
            let brim = mesh(pg("HatRacks__capBrim", cyl_geo!(0.055, 0.055, 0.006, 12, 1, false, 0, PI)), &hat_mat);
            brim.rotation().y = -PI / 2.0; // half-disc's round edge (+X) turned to face out (+Z), lying flat
            brim.position().set(0, 0.006, 0.035);
            hat.add(&brim);
            // Small button on top.
            let btn = mesh(pg("HatRacks__button", sphere_geo!(0.005, 5, 4)), &hat_mat);
            btn.position().y = 0.055;
            hat.add(&btn);
        }
        // Crown hooked over the peg: clear of the rail, the knob inside the crown
        // touching its inner top (brim clear of the wall).
        hat.position().set(px, mount_y - (if is_fedora { 0.056 } else { 0.042 }), 0.078);
        hat.rotation().y = (rnd() - 0.5) * 0.15;
        // For fedora, tilt slightly forward. For cap, no tilt.
        if is_fedora {
            hat.rotation().x = 0.15;
        }
        rack.add(&hat);
    }
    rack.rotation().y = (-(dx as f64)).atan2(-(dz as f64));
    let px = g.wx(x) + dx as f64 * (CELL * 0.5 - 0.015);
    let pz = g.wz(z) + dz as f64 * (CELL * 0.5 - 0.015);
    rack.position().set(px, 0, pz);
    g.world.add(&rack);
}

// Grandfather clock w/ pendulum (floor near wall): 2% chance per level.
fn place_grandfather_clocks(g: &mut Game, _dist: &[i32]) {
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

#[cfg(test)]
mod tests {
    #[test]
    fn smoke() {
        let mut list = vec![];
        super::register(&mut list);
        game::debug::testkit::smoke_placers(&list);
    }
}

/// Batch r14b's kit (the wristwatch): a seed from where it lies, hashed
/// noise, and what is painted: the dial, the strap's leather or webbing,
/// the plating worn through, the crystal's scratches.
mod r14b_kit {
    use game::prelude::*;

    /// A seed from values already fixed (a position): no draws
    pub fn seed_of(v: &[f64]) -> u32 {
        let mut x: u64 = 0xcbf2_9ce4_8422_2325;
        for a in v {
            for b in ((a * 1.0e4).round() as i64).to_le_bytes() {
                x ^= u64::from(b);
                x = x.wrapping_mul(0x0100_0000_01b3);
            }
        }
        ((x ^ (x >> 32)) as u32) | 1
    }

    fn h01(seed: u32, i: i32, j: i32) -> f64 {
        let mut h = seed ^ (i as u32).wrapping_mul(0x27d4_eb2d) ^ (j as u32).wrapping_mul(0x1656_67b1);
        h = (h ^ (h >> 15)).wrapping_mul(0x85eb_ca6b);
        h = (h ^ (h >> 13)).wrapping_mul(0xc2b2_ae35);
        h ^= h >> 16;
        f64::from(h) / 4_294_967_296.0
    }

    /// Smooth value noise in [0, 1]
    fn vnoise(seed: u32, x: f64, y: f64) -> f64 {
        let (xf, yf) = (x.floor(), y.floor());
        let (i, j) = (xf as i32, yf as i32);
        let (u, v) = (x - xf, y - yf);
        let (u, v) = (u * u * (3.0 - 2.0 * u), v * v * (3.0 - 2.0 * v));
        let (a, b) = (h01(seed, i, j), h01(seed, i + 1, j));
        let (c, d) = (h01(seed, i, j + 1), h01(seed, i + 1, j + 1));
        a + (b - a) * u + (c - a) * v + (a - b - c + d) * u * v
    }

    /// The dials (sRGB): aged cream, black, white gone grey, navy; and the
    /// ink printed on each
    const DIALS: [(&str, &str, &str); 4] = [
        ("#d6ccb0", "rgba(26,24,22,0.92)", "rgba(150,40,30,0.85)"),
        ("#1d1c1b", "rgba(222,214,196,0.9)", "rgba(196,92,60,0.85)"),
        ("#d2cfc6", "rgba(26,24,22,0.92)", "rgba(150,40,30,0.85)"),
        ("#29384e", "rgba(222,216,200,0.9)", "rgba(206,120,80,0.85)"),
    ];

    /// A watch's dial seen from above (12 at the top): the minute track, the
    /// hours in a plain grotesque, the 24-hour ring small inside them, the
    /// maker's name (none real) and QUARTZ; dulled toward its rim, freckled
    /// where damp got under the crystal
    pub fn dial_tex(k: usize, seed: u32) -> Tex {
        canvas_tex_opt(256, 256, false, move |ctx, w, h| {
            let (ground, ink, red) = DIALS[k];
            let (cx, cy, r) = (w / 2.0, h / 2.0, w / 2.0);
            ctx.set_fill_style(ground);
            ctx.fill_rect(0, 0, w, h);
            let g0 = ctx.create_radial_gradient(cx, cy, r * 0.55, cx, cy, r);
            g0.add_color_stop(0, "rgba(120,100,70,0)");
            g0.add_color_stop(1, "rgba(120,100,70,0.22)");
            ctx.set_fill_style(g0);
            ctx.fill_rect(0, 0, w, h);
            ctx.set_stroke_style(ink);
            ctx.set_line_cap("butt");
            for i in 0..60 {
                let a = i as f64 / 60.0 * PI * 2.0;
                let five = i % 5 == 0;
                let (r0, r1) = if five { (0.86, 0.97) } else { (0.92, 0.97) };
                ctx.set_line_width(if five { 3.2 } else { 1.2 });
                ctx.begin_path();
                ctx.move_to(cx + a.sin() * r * r0, cy - a.cos() * r * r0);
                ctx.line_to(cx + a.sin() * r * r1, cy - a.cos() * r * r1);
                ctx.stroke();
            }
            ctx.set_fill_style(ink);
            ctx.set_text_align("center");
            ctx.set_text_baseline("middle");
            ctx.set_font("bold 30px Helvetica Neue");
            for i in 1..=12 {
                let a = i as f64 / 12.0 * PI * 2.0;
                ctx.fill_text(&i.to_string(), cx + a.sin() * r * 0.70, cy - a.cos() * r * 0.70 + 1.0);
            }
            ctx.set_fill_style(red);
            ctx.set_font("bold 11px Helvetica Neue");
            for i in 13..=24 {
                let a = (i - 12) as f64 / 12.0 * PI * 2.0;
                ctx.fill_text(&i.to_string(), cx + a.sin() * r * 0.47, cy - a.cos() * r * 0.47);
            }
            ctx.set_fill_style(ink);
            ctx.set_font("bold 12px Futura");
            ctx.fill_text("HALDEN", cx, cy - r * 0.26);
            ctx.set_font("8px Helvetica Neue");
            ctx.fill_text("QUARTZ", cx, cy + r * 0.26);
            ctx.fill_text("WATER RESISTANT", cx, cy + r * 0.36);
            // freckles where damp got in, and the print dulled
            let mut rr = SplitMix64::new(seed, 0xd1a1);
            let n = 30 + (rr.next() * 50.0) as usize;
            for _ in 0..n {
                let a = rr.range(0.0, PI * 2.0);
                let d = r * rr.range(0.2, 0.98).sqrt();
                ctx.set_fill_style(&format!("rgba(110,78,40,{:.2})", rr.range(0.15, 0.5)));
                ctx.begin_path();
                ctx.arc(cx + a.cos() * d, cy + a.sin() * d, rr.range(0.6, 2.2), 0, 7);
                ctx.fill();
            }
            if rr.next() < 0.5 {
                // a tide mark low on the dial, where water stood once
                let y = cy + r * rr.range(0.35, 0.6);
                let gr = ctx.create_linear_gradient(0, y - 18.0, 0, h);
                gr.add_color_stop(0, "rgba(120,90,50,0)");
                gr.add_color_stop(0.15, "rgba(120,90,50,0.22)");
                gr.add_color_stop(1, "rgba(120,90,50,0.10)");
                ctx.set_fill_style(gr);
                ctx.fill_rect(0, y - 18.0, w, h - y + 18.0);
            }
        })
    }

    /// The strap (seen from above, its length down the canvas from z1 at
    /// the top to z0): leather's grain, creased and darkened at the hole it
    /// was worn on and where the wrist sweated; scuffed pale; or a woven
    /// webbing's ribs
    pub fn strap_mat(band: usize, z0: f64, z1: f64, seed: u32) -> Mat {
        // black, dark brown, grey webbing, tan, navy webbing, brown (sRGB)
        let col: [(f64, f64, f64); 6] =
            [(30.0, 28.0, 26.0), (64.0, 42.0, 28.0), (92.0, 92.0, 90.0), (118.0, 86.0, 56.0), (38.0, 42.0, 62.0), (88.0, 56.0, 34.0)];
        let webbing = band == 2 || band == 4;
        let c = col[band];
        let tex = canvas_tex_opt(64, 512, false, move |ctx, w, h| {
            let vy = |z: f64| (1.0 - (z - z0) / (z1 - z0)) * h;
            let n = w as usize;
            let mut img = ctx.create_image_data(w, h);
            let s = seed;
            par_rows(&mut img.data, n * 4, |j, out| {
                for i in 0..n {
                    let (x, y) = (i as f64, j as f64);
                    let k = if webbing {
                        let rib = (x * 1.9).sin() * 0.5 + 0.5;
                        0.86 + 0.12 * rib + 0.06 * (vnoise(s, x * 0.3, y * 0.05) - 0.5)
                    } else {
                        let grain = vnoise(s, x * 0.9, y * 0.9) * 0.5 + vnoise(s ^ 5, x * 0.25, y * 0.25) * 0.5;
                        0.84 + 0.22 * grain
                    };
                    let px = |v: f64| (v * k).clamp(0.0, 255.0) as u8;
                    out[i * 4..i * 4 + 4].copy_from_slice(&[px(c.0), px(c.1), px(c.2), 255]);
                }
            });
            ctx.put_image_data(&img, 0, 0);
            // where it was worn: the hole it was buckled on, creased across
            // and darkened, the sweat band behind the case on both pieces
            let worn = vy(0.0905);
            for (y, a) in [(worn, 0.35), (vy(0.032), 0.22), (vy(-0.030), 0.22)] {
                let gr = ctx.create_linear_gradient(0, y - 20.0, 0, y + 20.0);
                gr.add_color_stop(0, "rgba(10,8,6,0)");
                gr.add_color_stop(0.5, &format!("rgba(10,8,6,{a})"));
                gr.add_color_stop(1, "rgba(10,8,6,0)");
                ctx.set_fill_style(gr);
                ctx.fill_rect(0, y - 20.0, w, 40);
            }
            let mut r = SplitMix64::new(seed, 0x57a9);
            ctx.set_line_width(1.0);
            for k in 0..5 {
                let y = worn + (k as f64 - 2.0) * 3.0 + r.range(-1.0, 1.0);
                ctx.set_stroke_style(if webbing { "rgba(20,18,16,0.25)" } else { "rgba(14,10,8,0.45)" });
                ctx.begin_path();
                ctx.move_to(0, y);
                ctx.bezier_curve_to(w * 0.3, y + r.range(-2.0, 2.0), w * 0.7, y + r.range(-2.0, 2.0), w, y + r.range(-1.0, 1.0));
                ctx.stroke();
            }
            // scuffs, pale, and the finish rubbed off in patches
            for _ in 0..40 {
                let (x, y) = (r.range(0.0, w), r.range(0.0, h));
                ctx.set_fill_style(&format!("rgba(200,184,160,{:.2})", r.range(0.05, 0.18)));
                ctx.begin_path();
                ctx.ellipse(x, y, r.range(1.0, 5.0), r.range(0.6, 2.0), r.range(0.0, 3.0), 0, 7);
                ctx.fill();
            }
        });
        if webbing {
            lambert!({ map: &tex })
        } else {
            phong!({ map: &tex, specular: 0x161412, shininess: 18 })
        }
    }

    /// The plating rubbed through at the edges and lugs to the brass under
    /// it, and grime in the joints (multiplied over the metal's colour)
    pub fn plating_tex(k: usize, seed: u32) -> Tex {
        canvas_tex_opt(64, 64, false, move |ctx, w, h| {
            ctx.set_fill_style("#ffffff");
            ctx.fill_rect(0, 0, w, h);
            let mut r = SplitMix64::new(seed, 0x91a7 + k as u64);
            let brass = if k == 0 { "rgba(170,120,60,0.55)" } else { "rgba(150,110,60,0.45)" };
            for _ in 0..26 {
                let (x, y) = (r.range(0.0, w), r.range(0.0, h));
                ctx.set_fill_style(brass);
                ctx.begin_path();
                ctx.ellipse(x, y, r.range(1.0, 4.0), r.range(0.5, 2.0), r.range(0.0, 3.0), 0, 7);
                ctx.fill();
            }
            for _ in 0..30 {
                let (x, y) = (r.range(0.0, w), r.range(0.0, h));
                ctx.set_fill_style(&format!("rgba(40,34,26,{:.2})", r.range(0.2, 0.5)));
                ctx.fill_rect(x, y, r.range(1.0, 3.0), r.range(1.0, 2.0));
            }
        })
    }

    /// The domed acrylic: polished once, now hazed and scratched (grey:
    /// how much)
    pub fn crystal_dirt_tex(seed: u32) -> Tex {
        canvas_tex_opt(128, 128, false, move |ctx, w, h| {
            let n = w as usize;
            let mut img = ctx.create_image_data(w, h);
            let s = seed;
            par_rows(&mut img.data, n * 4, |j, out| {
                for i in 0..n {
                    let (x, y) = (i as f64 / w, j as f64 / h);
                    let d = vnoise(s, x * 5.0, y * 5.0) * 0.6 + vnoise(s ^ 3, x * 19.0, y * 19.0) * 0.4;
                    let v = ((0.16 + 0.30 * d) * 255.0) as u8;
                    out[i * 4..i * 4 + 4].copy_from_slice(&[v, v, v, 255]);
                }
            });
            ctx.put_image_data(&img, 0, 0);
            let mut r = SplitMix64::new(seed, 0xc4a5);
            ctx.set_line_cap("round");
            for _ in 0..60 {
                let (x0, y0) = (r.range(0.0, w), r.range(0.0, h));
                let a = r.range(0.0, PI);
                let l = r.range(4.0, 26.0);
                ctx.set_stroke_style(&format!("rgba(240,240,240,{:.2})", r.range(0.3, 0.75)));
                ctx.set_line_width(r.range(0.5, 1.0));
                ctx.begin_path();
                ctx.move_to(x0, y0);
                ctx.line_to(x0 + a.cos() * l, y0 + a.sin() * l);
                ctx.stroke();
            }
        })
    }

    /// The crystal as the windows' glass (`Patch::Glass`): the room seen in
    /// it by Fresnel, its haze from the map's grey
    pub fn crystal_mat(map: &Tex) -> Mat {
        let m = phong!({ color: 0xffffff, specular: 0x9a9a9a, shininess: 220, transparent: true, depthWrite: false });
        {
            let mut mm = m.m();
            mm.map = Some(map.clone());
            mm.patch = Patch::Glass;
            mm.look_settled = true;
            mm.name = "watch_crystal".into();
        }
        {
            let pu = m.r().patch_uniforms.clone();
            let f = &mut pu.borrow_mut().f;
            f[0] = 0.5;
            f[4] = 0.5;
            f[5] = 0.6;
            f[6] = 0.3;
            f[7] = 0.0;
            f[8] = 1.4;
            f[9] = 0.2;
            f[10] = 0.195;
            f[11] = 0.18;
            f[12] = 0.0;
            f[13] = 0.07;
        }
        m
    }
}
