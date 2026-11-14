//! `place_travel_documents`
use game::prelude::*;
use placers_kit::place::open_tiles;

pub(crate) fn place_travel_documents(g: &mut Game, _dist: &[i32]) {
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
        ctx.fill_text(&format!("{seat_row}{seat_letter}"), 172, 64);
        // Date + time.
        ctx.set_font("7px sans-serif");
        ctx.fill_text("DATE", 38, 78);
        ctx.fill_text("DEPART", 90, 78);
        ctx.fill_text("GATE", 148, 78);
        ctx.set_font("bold 9px monospace");
        ctx.fill_text("14 MAR", 38, 90);
        let dep_h = (rnd() * 12.0).floor() as i64 + 6;
        let dep_m = ((rnd() * 6.0).floor() as i64) * 10;
        ctx.fill_text(&format!("{dep_h}:{dep_m}0"), 90, 90);
        let gate_l = ["A", "B", "C", "D"][(rnd() * 4.0).floor() as usize];
        let gate_n = (rnd() * 30.0).floor() as i64 + 1;
        ctx.fill_text(&format!("{gate_l}{gate_n}"), 148, 90);
        // Barcode strip on the right — thin dark lines.
        ctx.set_fill_style("#101010");
        for i in 0..40 {
            let bar_w = 0.5 + rnd() * 1.5;
            ctx.fill_rect(180.0 + i as f64 * 1.2, 24, bar_w, 40);
        }
    })
);
