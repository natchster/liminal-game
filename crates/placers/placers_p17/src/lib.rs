//! Prop placers: servant bell racks, cast-iron door stops, correspondence cards,
//! empty painting outlines, memorial shrines, chess side stands, miniature
//! portrait clusters, dress forms and peacock-feather vases.
#![allow(unused_variables, unused_mut, unused_assignments, unused_imports, dead_code, clippy::all)]
use game::prelude::*;
use std::collections::HashSet;

mod t01;
use t01::place_miniature_portrait_clusters;
// the floor vase and its peacock feathers: batch `t11`'s (`t11.rs`)
mod t11;
use t11::place_peacock_feather_vases;
// where a picture hung: batch `t19`'s (`t19.rs`)
mod t19;
use t19::place_empty_painting_outlines;

// the memorial shrine: batch `t18`'s (`t18.rs`)
mod t18;
use t18::place_memorial_shrines;

/// Every placer in this crate, by its registered name.
pub fn register(r: &mut Vec<(&'static str, PlacerFn)>) {
    r.push(("place_servant_hall_bell_racks", place_servant_hall_bell_racks));
    r.push(("place_cast_iron_door_stops", place_cast_iron_door_stops));
    r.push(("place_correspondence_cards", place_correspondence_cards));
    r.push(("place_empty_painting_outlines", place_empty_painting_outlines));
    r.push(("place_memorial_shrines", place_memorial_shrines));
    r.push(("place_chess_side_stands", place_chess_side_stands));
    r.push(("place_miniature_portrait_clusters", place_miniature_portrait_clusters));
    r.push(("place_dress_forms", place_dress_forms));
    r.push(("place_peacock_feather_vases", place_peacock_feather_vases));
}

// ------------------------------------------------------------------ helpers
/// `{ x, z, dx, dz }`: an open tile with a wall (or the grid edge) on side (dx, dz).
#[derive(Clone, Copy, Debug)]
struct WallCand {
    x: i32,
    z: i32,
    dx: i32,
    dz: i32,
}

/// The candidate scan every wall placer repeats.
fn wall_candidates(g: &Game) -> Vec<WallCand> {
    let mut v = vec![];
    for z in 1..g.h - 1 {
        for x in 1..g.w - 1 {
            if g.tiles[g.t_idx(x, z)] != 0 {
                continue;
            }
            for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let (nx, nz) = (x + dx, z + dz);
                if !g.in_grid(nx, nz) || g.tiles[g.t_idx(nx, nz)] == 1 {
                    v.push(WallCand { x, z, dx, dz });
                }
            }
        }
    }
    v
}

/// Every open tile of the interior, row by row.
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

/// The shared wall-mount tail: face the room, sit `inset` off the wall face,
/// slide along the wall by `(`rnd()` - 0.5) * jitter`.
fn mount_on_wall(g: &Game, o: &Obj, c: &WallCand, inset: f64, center_y: f64, jitter: f64) {
    let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - inset);
    let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - inset);
    if c.dx.abs() == 1 {
        o.rotation().y = if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 };
        let dz = (rnd() - 0.5) * jitter;
        o.position().set(wall_x, center_y, wall_z + dz);
    } else {
        o.rotation().y = if c.dz > 0 { PI } else { 0.0 };
        let dx = (rnd() - 0.5) * jitter;
        o.position().set(wall_x + dx, center_y, wall_z);
    }
}

/// Whether the object's geometry is a generated torus. A fitted prop part
/// (`assets::props`) reports as a plain geometry, so it never matches.
fn is_torus_geometry(o: &Obj) -> bool {
    match o.try_geo() {
        Some(gg) => gg.params().type_name() == "torus" && !gg.r().is_prop_part,
        None => false,
    }
}

// R758 servant's-hall bell rack (wall): 2.5% chance per level.
fn place_servant_hall_bell_racks(g: &mut Game, _dist: &[i32]) {
    g.pp.bell_rack_positions.clear(); // R761 reset per level
    if rnd() < 0.975 {
        return;
    }
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = candidates[(rnd() * candidates.len() as f64).floor() as usize];
    let wood_colors = [0x2a1810, 0x402015, 0x604020];
    let wood_mat = lambert!({ color: choice(&wood_colors) });
    let brass_mat = lambert!({ color: 0xa07030, emissive: 0x1a1005 });
    let br = group();
    // Wooden mounting board.
    let bell_count = 6 + (rnd() * 3.0).floor() as i32; // 6, 7, or 8
    let bwidth = bell_count as f64 * 0.070 + 0.030;
    let bheight = 0.28;
    let board = mesh(pg("ServantHallBellRacks__board", box_geo!(bwidth, bheight, 0.020)), &wood_mat);
    br.add(&board);
    // Ornamental top trim (back onto the wall, 9 mm past the board face).
    let top_trim = mesh(pg("ServantHallBellRacks__topTrim", box_geo!(bwidth + 0.020, 0.020, 0.030)), &wood_mat);
    top_trim.position().set(0, bheight / 2.0 + 0.010, 0.004);
    br.add(&top_trim);
    // Bottom trim.
    let bot_trim = mesh(pg("ServantHallBellRacks__botTrim", box_geo!(bwidth + 0.020, 0.015, 0.030)), &wood_mat);
    bot_trim.position().set(0, -bheight / 2.0 - 0.008, 0.004);
    br.add(&bot_trim);
    // Title strip at top centre.
    let title_plate = mesh(pg("ServantHallBellRacks__title", box_geo!(bwidth * 0.4, 0.020, 0.001)), &brass_mat);
    title_plate.position().set(0, bheight / 2.0 - 0.020, 0.011);
    br.add(&title_plate);
    // 6-8 small brass bells in a row + numbered labels.
    let _room_names = ["DRAWING", "LIBRARY", "DINING", "BILLIARDS", "STUDY", "PARLOUR", "MASTER", "NURSERY"];
    for i in 0..bell_count {
        let bx = -bwidth / 2.0 + 0.030 + i as f64 * 0.070;
        // Spring wire: six 60° arcs overlapping into one 210° curl in the
        // plane across the board (YZ).
        let spring_seg = 6;
        for s in 0..spring_seg {
            let spring = mesh(pg("ServantHallBellRacks__spring", torus_geo!(0.010, 0.0008, 3, 5, PI / 3.0)), &brass_mat);
            spring.rotation().y = -PI / 2.0;
            spring.rotation().z = -PI / 3.0 + s as f64 * (PI / 6.0);
            spring.position().set(bx, 0.0540, 0.0195);
            br.add(&spring);
        }
        // Bell body — small dome.
        let small_bell = mesh(pg("ServantHallBellRacks__bell", cyl_geo!(0.014, 0.010, 0.024, 8, 1, true)), &brass_mat);
        small_bell.position().set(bx, 0.030, 0.025);
        br.add(&small_bell);
        // Bell top cap.
        let bell_cap = mesh(pg("ServantHallBellRacks__cap", cyl_geo!(0.010, 0.010, 0.003, 8)), &brass_mat);
        bell_cap.position().set(bx, 0.043, 0.025);
        br.add(&bell_cap);
        // Numbered label plate below bell.
        let label_plate = mesh(pg("ServantHallBellRacks__label", box_geo!(0.050, 0.020, 0.001)), lambert!({ color: 0xf0e8d0 }));
        label_plate.position().set(bx, -0.030, 0.011);
        br.add(&label_plate);
        // Number engraving.
        let num_mark = mesh(box_geo!(0.015, 0.010, 0.0005), lambert!({ color: 0x1a1a1a }));
        num_mark.position().set(bx, -0.030, 0.012);
        br.add(&num_mark);
        // Room label plate below number.
        let room_plate = mesh(pg("ServantHallBellRacks__roomPlate", box_geo!(0.055, 0.016, 0.001)), lambert!({ color: 0xd8c8a8 }));
        room_plate.position().set(bx, -0.060, 0.011);
        br.add(&room_plate);
        // Room name mark.
        let room_mark = mesh(box_geo!(0.040, 0.006, 0.0005), lambert!({ color: 0x2a1810 }));
        room_mark.position().set(bx, -0.060, 0.012);
        br.add(&room_mark);
    }
    mount_on_wall(g, &br, &c, 0.011, 1.80, 0.3);
    br.rotation().z = (rnd() - 0.5) * 0.010;
    g.world.add(&br);
    let p = br.pos();
    g.pp.bell_rack_positions.push(v3(p.x, 0, p.z)); // R761
}

// R759 cast iron door-stop (floor): 4% chance per level. dog / boot / owl.
fn place_cast_iron_door_stops(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let iron_colors = [0x181818, 0x2a1a10, 0x502010]; // fresh iron, rust-brown, rusted red
    let iron_mat = lambert!({ color: choice(&iron_colors) });
    let ds = group();
    // Base plate.
    let base = mesh(pg("CastIronDoorStops__base", box_geo!(0.11, 0.010, 0.09)), &iron_mat);
    base.position().y = 0.005;
    ds.add(&base);
    let variant = (rnd() * 3.0).floor() as i32;
    if variant == 0 {
        // Dog figurine — spaniel-like silhouette.
        let body = mesh(pg("CastIronDoorStops__dogBody", sphere_geo!(0.030, 8, 6)), &iron_mat);
        body.scale().set(1.5, 1.0, 0.9);
        body.position().set(0, 0.030, 0);
        ds.add(&body);
        let head = mesh(pg("CastIronDoorStops__dogHead", sphere_geo!(0.022, 8, 6)), &iron_mat);
        head.position().set(0.033, 0.045, 0);
        ds.add(&head);
        let snout = mesh(pg("CastIronDoorStops__dogMuzzle", cone_geo!(0.010, 0.018, 6)), &iron_mat);
        snout.rotation().z = -PI / 2.0;
        snout.position().set(0.055, 0.042, 0);
        ds.add(&snout);
        // 2 floppy ears.
        for side in [-1.0, 1.0] {
            let ear = mesh(pg("CastIronDoorStops__dogEar", sphere_geo!(0.008, 6, 5)), &iron_mat);
            ear.scale().set(0.5, 1.4, 0.7);
            ear.position().set(0.030, 0.040, side * 0.016);
            ds.add(&ear);
        }
        // 4 stubby legs.
        for lx in [-0.020, 0.020] {
            for lz in [-0.014, 0.014] {
                let leg = mesh(pg("CastIronDoorStops__dogLeg", cyl_geo!(0.006, 0.006, 0.020, 5)), &iron_mat);
                leg.position().set(lx, 0.020, lz);
                ds.add(&leg);
            }
        }
        // Tail (curled up).
        let tail = mesh(pg("CastIronDoorStops__dogTail", cyl_geo!(0.004, 0.004, 0.020, 4)), &iron_mat);
        tail.position().set(-0.030, 0.050, 0);
        tail.rotation().z = 0.6;
        ds.add(&tail);
        // 2 tiny eyes.
        for side in [-1.0, 1.0] {
            let eye = mesh(sphere_geo!(0.002, 4, 4), lambert!({ color: 0x0a0a0a }));
            eye.position().set(0.045, 0.052, side * 0.008);
            ds.add(&eye);
        }
    } else if variant == 1 {
        // Victorian boot.
        let foot = mesh(pg("CastIronDoorStops__bootFoot", box_geo!(0.075, 0.030, 0.038)), &iron_mat);
        foot.position().set(0, 0.025, 0);
        ds.add(&foot);
        let toe = mesh(pg("CastIronDoorStops__bootToe", sphere_geo!(0.018, 8, 6)), &iron_mat);
        toe.scale().set(1.0, 0.8, 0.9);
        toe.position().set(0.038, 0.0245, 0); // rests on the base plate (top 0.010)
        ds.add(&toe);
        let heel = mesh(pg("CastIronDoorStops__bootHeel", box_geo!(0.018, 0.014, 0.030)), &iron_mat);
        heel.position().set(-0.030, 0.017, 0);
        ds.add(&heel);
        let shin = mesh(pg("CastIronDoorStops__bootShaft", box_geo!(0.038, 0.075, 0.038)), &iron_mat);
        shin.position().set(-0.010, 0.075, 0);
        ds.add(&shin);
        // 5 tiny laces across the shaft's front face (+X, the toe side).
        for i in 0..5 {
            let lace = mesh(pg("CastIronDoorStops__lace", cyl_geo!(0.0015, 0.0015, 0.006, 4)), lambert!({ color: 0x080808 }));
            lace.rotation().x = PI / 2.0;
            lace.position().set(0.0095, 0.050 + i as f64 * 0.010, 0);
            ds.add(&lace);
        }
        // Boot cuff/top opening.
        let cuff = mesh(pg("CastIronDoorStops__bootCuff", torus_geo!(0.015, 0.003, 4, 10)), &iron_mat);
        cuff.rotation().x = PI / 2.0;
        cuff.position().set(-0.010, 0.115, 0);
        ds.add(&cuff);
    } else {
        // Owl figurine.
        let body = mesh(pg("CastIronDoorStops__owlBody", sphere_geo!(0.035, 10, 7)), &iron_mat);
        body.scale().set(1.0, 1.2, 0.9);
        body.position().set(0, 0.040, 0);
        ds.add(&body);
        let head = mesh(pg("CastIronDoorStops__owlHead", sphere_geo!(0.024, 10, 7)), &iron_mat);
        head.position().set(0, 0.080, 0);
        ds.add(&head);
        // 2 large staring owl eyes.
        for side in [-1.0, 1.0] {
            let eye_ring = mesh(pg("CastIronDoorStops__owlEyeRing", torus_geo!(0.008, 0.002, 4, 10)), &iron_mat);
            eye_ring.rotation().y = PI / 2.0;
            eye_ring.position().set(0.020, 0.083, side * 0.009);
            ds.add(&eye_ring);
            let eye_pupil = mesh(sphere_geo!(0.003, 5, 4), lambert!({ color: 0x0a0a0a }));
            eye_pupil.position().set(0.022, 0.083, side * 0.009);
            ds.add(&eye_pupil);
        }
        // Small triangular beak between eyes.
        let beak = mesh(pg("CastIronDoorStops__owlBeak", cone_geo!(0.004, 0.008, 4)), &iron_mat);
        beak.rotation().z = -PI / 2.0;
        beak.position().set(0.025, 0.076, 0);
        ds.add(&beak);
        // 2 wing outlines.
        for side in [-1.0, 1.0] {
            let wing = mesh(pg("CastIronDoorStops__owlWing", sphere_geo!(0.014, 6, 5)), &iron_mat);
            wing.scale().set(0.6, 1.2, 0.5);
            wing.position().set(0, 0.045, side * 0.028);
            ds.add(&wing);
        }
        // 2 small feet.
        for side in [-1.0, 1.0] {
            let foot = mesh(pg("CastIronDoorStops__owlFoot", cyl_geo!(0.005, 0.005, 0.008, 5)), &iron_mat);
            foot.position().set(0, 0.014, side * 0.010);
            ds.add(&foot);
        }
        // Ear tufts.
        for side in [-1.0, 1.0] {
            let tuft = mesh(pg("CastIronDoorStops__owlTuft", cone_geo!(0.004, 0.010, 4)), &iron_mat);
            tuft.position().set(-0.005, 0.098, side * 0.008);
            tuft.rotation().z = side * 0.3;
            ds.add(&tuft);
        }
    }
    let px = g.wx(x) + (rnd() - 0.5) * 2.2;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.2;
    ds.position().set(px, 0, pz);
    ds.rotation().y = rnd() * PI * 2.0;
    g.world.add(&ds);
}

// R759 framed correspondence: telegram / calling card / formal invitation.
game::lazy_static_tex!(
    correspondence_texture,
    canvas_tex(180, 240, |ctx, w, h| {
        // Base colour varies by variant — creamy paper for all.
        let variant = (rnd() * 3.0).floor() as usize;
        let base_colors = ["#f0e8c8", "#f4ecd8", "#e8dcc0"];
        ctx.set_fill_style(base_colors[variant]);
        ctx.fill_rect(0, 0, w, h);
        // Age spots.
        ctx.set_fill_style("rgba(120,90,50,0.20)");
        for _ in 0..8 {
            ctx.begin_path();
            let (x, y, r) = (rnd() * w, rnd() * h, 2.0 + rnd() * 5.0);
            ctx.arc(x, y, r, 0, PI * 2.0);
            ctx.fill();
        }
        let names =
            ["Mrs. Ashford", "Sir Charles", "Mr. Whitworth", "Lady Millbank", "Dr. Pendleton", "Miss Emily Rowe", "The Hon. E. Delacroix"];
        let name1 = names[(rnd() * names.len() as f64).floor() as usize];
        let name2 = names[(rnd() * names.len() as f64).floor() as usize];
        if variant == 0 {
            // TELEGRAM — top header + terse block-caps text.
            ctx.set_fill_style("#20180c");
            ctx.set_font("bold 12px monospace");
            ctx.set_text_align("center");
            ctx.fill_text("WESTERN UNION", w / 2.0, 20);
            ctx.set_font("bold 10px monospace");
            ctx.fill_text("T E L E G R A M", w / 2.0, 34);
            // Header divider.
            ctx.set_stroke_style("#20180c");
            ctx.set_line_width(1);
            ctx.begin_path();
            ctx.move_to(10, 40);
            ctx.line_to(w - 10.0, 40);
            ctx.stroke();
            // Date + time.
            ctx.set_font("9px monospace");
            ctx.set_text_align("left");
            let dates = ["14 MAR 1912", "03 SEP 1908", "22 NOV 1897", "17 JUL 1919"];
            let date = dates[(rnd() * dates.len() as f64).floor() as usize];
            ctx.fill_text(&format!("DATE: {}", date), 12, 54);
            ctx.fill_text(&format!("TO: {}", name1.to_uppercase()), 12, 68);
            // Body — 4 short lines of all-caps telegram text (with STOP).
            ctx.set_font("bold 10px monospace");
            let messages: [[&str; 4]; 3] = [
                ["ARRIVING BY TRAIN", "THURSDAY EVENING STOP", "PLEASE MEET AT STATION", "STOP MOST URGENT STOP"],
                ["REGRET INFORM YOU", "FATHER TAKEN ILL STOP", "COME AT ONCE STOP", "MOTHER"],
                ["MATTER SETTLED STOP", "RETURNING TOMORROW STOP", "BRING SIGNED PAPERS", "STOP LOVE STOP"],
            ];
            let msg = messages[(rnd() * messages.len() as f64).floor() as usize];
            for (i, line) in msg.iter().enumerate() {
                ctx.fill_text(line, 12, 90.0 + i as f64 * 16.0);
            }
            // Signature at bottom.
            ctx.set_font("italic 9px monospace");
            ctx.fill_text(&format!("FROM {}", name2.to_uppercase()), 12, h - 30.0);
            // Delivery timestamp (small).
            ctx.set_font("7px monospace");
            ctx.set_fill_style("#603820");
            ctx.fill_text("DELIVERED 4:22 PM", 12, h - 15.0);
        } else if variant == 1 {
            // CALLING CARD — decorative border.
            ctx.set_stroke_style("#403020");
            ctx.set_line_width(1);
            ctx.stroke_rect(12, 20, w - 24.0, h - 40.0);
            ctx.stroke_rect(16, 24, w - 32.0, h - 48.0);
            // Corner flourishes.
            ctx.set_fill_style("#403020");
            for (cx, cy) in [(20.0, 28.0), (w - 20.0, 28.0), (20.0, h - 28.0), (w - 20.0, h - 28.0)] {
                ctx.begin_path();
                ctx.arc(cx, cy, 3, 0, PI * 2.0);
                ctx.fill();
            }
            // Central name (large elegant serif).
            ctx.set_text_align("center");
            ctx.set_fill_style("#20180c");
            ctx.set_font("italic 20px serif");
            ctx.fill_text(name1, w / 2.0, h * 0.42);
            // Small subtitle (address).
            ctx.set_font("italic 10px serif");
            ctx.set_fill_style("#403020");
            let addresses = ["17 Belgrave Square", "42 Pembridge Villas", "9 Cadogan Place", "31 Portman Street", "5 Grosvenor Crescent"];
            let addr = addresses[(rnd() * addresses.len() as f64).floor() as usize];
            ctx.fill_text(addr, w / 2.0, h * 0.58);
            // Bottom flourish line.
            ctx.set_stroke_style("#403020");
            ctx.set_line_width(0.7);
            ctx.begin_path();
            ctx.move_to(w * 0.30, h * 0.68);
            ctx.line_to(w * 0.70, h * 0.68);
            ctx.stroke();
            // Optional turned-corner mark ("in person").
            if rnd() < 0.5 {
                ctx.set_fill_style("rgba(90,70,40,0.6)");
                ctx.begin_path();
                ctx.move_to(w - 30.0, 22);
                ctx.line_to(w - 12.0, 22);
                ctx.line_to(w - 12.0, 40);
                ctx.close_path();
                ctx.fill();
            }
        } else {
            // FORMAL INVITATION — ornate script + wax seal.
            ctx.set_fill_style("#20180c");
            ctx.set_font("italic 14px serif");
            ctx.set_text_align("center");
            ctx.fill_text("The pleasure of", w / 2.0, 40);
            ctx.set_font("italic 12px serif");
            ctx.fill_text("your company is requested", w / 2.0, 58);
            // Name of invitee.
            ctx.set_font("bold italic 15px serif");
            ctx.fill_text(name1, w / 2.0, 90);
            // Event body.
            ctx.set_font("italic 11px serif");
            ctx.set_fill_style("#302010");
            let events = ["at a Ball", "at Supper", "at Afternoon Tea", "at Musical Recital"];
            let ev = events[(rnd() * events.len() as f64).floor() as usize];
            ctx.fill_text(ev, w / 2.0, 116);
            ctx.set_font("10px serif");
            let inv_dates = ["Saturday 14th June", "Thursday 22nd October", "Friday 3rd September"];
            let inv = inv_dates[(rnd() * inv_dates.len() as f64).floor() as usize];
            ctx.fill_text(inv, w / 2.0, 138);
            ctx.set_font("italic 9px serif");
            ctx.fill_text("half past seven o'clock", w / 2.0, 154);
            // Host signature.
            ctx.set_font("italic 11px serif");
            ctx.set_fill_style("#20180c");
            ctx.fill_text(&format!("— {}", name2), w / 2.0, 184);
            // Red wax seal at bottom.
            ctx.set_fill_style("#8a1010");
            ctx.begin_path();
            ctx.arc(w / 2.0, h - 28.0, 14, 0, PI * 2.0);
            ctx.fill();
            // Seal highlight (curved gloss).
            ctx.set_fill_style("rgba(200,60,60,0.4)");
            ctx.begin_path();
            ctx.arc(w / 2.0 - 3.0, h - 32.0, 5, 0, PI * 2.0);
            ctx.fill();
            // Seal impression (initials, small).
            ctx.set_fill_style("#4a0808");
            ctx.set_font("bold 8px serif");
            ctx.fill_text("C.W.", w / 2.0, h - 24.0);
            // RSVP note.
            ctx.set_font("italic 8px serif");
            ctx.set_fill_style("#503020");
            ctx.set_text_align("right");
            ctx.fill_text("R.S.V.P.", w - 12.0, h - 6.0);
        }
    })
);

fn place_correspondence_cards(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    }
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = candidates[(rnd() * candidates.len() as f64).floor() as usize];
    let frame_colors = [0x2a1810, 0x402015, 0x604020, 0x8b6f47];
    let frame_mat = lambert!({ color: choice(&frame_colors) });
    let paper_mat = lambert!({ map: correspondence_texture(), side: DoubleSide });
    let cc = group();
    // Smaller frame (correspondence is letter-sized-and-smaller).
    let (cw, ch, ct, cfw) = (0.20, 0.28, 0.008, 0.014);
    for (bar_w, bar_h, bx, by) in [
        (cw + cfw * 2.0, cfw, 0.0, ch / 2.0 + cfw / 2.0),
        (cw + cfw * 2.0, cfw, 0.0, -ch / 2.0 - cfw / 2.0),
        (cfw, ch, -cw / 2.0 - cfw / 2.0, 0.0),
        (cfw, ch, cw / 2.0 + cfw / 2.0, 0.0),
    ] {
        let bar = mesh(pg("CorrespondenceCards__bar", box_geo!(bar_w, bar_h, ct)), &frame_mat);
        bar.position().set(bx, by, 0);
        cc.add(&bar);
    }
    let canvas = mesh(plane_geo!(cw, ch), &paper_mat);
    canvas.position().set(0, 0, ct / 2.0 + 0.001);
    cc.add(&canvas);
    mount_on_wall(g, &cc, &c, 0.005, 1.55, 0.5);
    cc.rotation().z = (rnd() - 0.5) * 0.030;
    g.world.add(&cc);
}

// R762 chess set mid-game on small side stand (floor): 3% chance per level.
fn place_chess_side_stands(g: &mut Game, _dist: &[i32]) {
    g.pp.chess_set_positions.clear(); // R763 reset per level
    if rnd() < 0.97 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let wood_colors = [0x604020, 0x2a1810, 0x8b6f47, 0x502010];
    let wood_mat = lambert!({ color: choice(&wood_colors) });
    let cs = group();
    // Small round side stand.
    let stand_r = 0.19;
    let stand_h = 0.62;
    let stand_top = mesh(pg("ChessSideStands__top", cyl_geo!(stand_r, stand_r, 0.014, 14)), &wood_mat);
    // Widened to r 0.2185 so the board's corners, the snifter and the
    // ashtray all stand on it.
    stand_top.scale().set(1.15, 1, 1.15);
    stand_top.position().y = stand_h;
    cs.add(&stand_top);
    // Central pedestal shaft.
    let pedestal = mesh(pg("ChessSideStands__pedestal", cyl_geo!(0.020, 0.030, stand_h * 0.75, 8)), &wood_mat);
    // Stretched to run from y 0.05 up to the top's underside.
    pedestal.scale().y = (stand_h - 0.007 - 0.05) / (stand_h * 0.75);
    pedestal.position().y = (0.05 + stand_h - 0.007) / 2.0;
    cs.add(&pedestal);
    // 3 splayed legs at base.
    for i in 0..3 {
        let angle = (i as f64 / 3.0) * PI * 2.0;
        let leg_end = 0.14;
        let leg = mesh(pg("ChessSideStands__leg", cyl_geo!(0.010, 0.015, leg_end, 5)), &wood_mat);
        // Splay: foot on its pad at r 0.12, the top leaning 0.6 rad inward.
        let leg_dir = v3(-angle.cos() * 0.6f64.sin(), 0.6f64.cos(), -angle.sin() * 0.6f64.sin());
        leg.quaternion().set_from_unit_vectors(&v3(0, 1, 0), &leg_dir);
        leg.position().set(
            angle.cos() * 0.12 + leg_dir.x * leg_end / 2.0,
            0.008 + leg_dir.y * leg_end / 2.0,
            angle.sin() * 0.12 + leg_dir.z * leg_end / 2.0,
        );
        cs.add(&leg);
    }
    // Small foot pad at bottom of each leg.
    for i in 0..3 {
        let angle = (i as f64 / 3.0) * PI * 2.0;
        let foot = mesh(pg("ChessSideStands__foot", cyl_geo!(0.015, 0.018, 0.008, 8)), &wood_mat);
        foot.position().set(angle.cos() * 0.12, 0.004, angle.sin() * 0.12);
        cs.add(&foot);
    }
    // Chess board on top — 8×8 checkered pattern.
    let board_size = 0.28;
    let cell_size = board_size / 8.0;
    let board_top = stand_h + 0.007;
    let light_square = lambert!({ color: 0xd0b090 });
    let dark_square = lambert!({ color: 0x4a2810 });
    // Board substrate.
    let board = mesh(pg("ChessSideStands__board", box_geo!(board_size + 0.020, 0.010, board_size + 0.020)), &wood_mat);
    board.position().y = board_top + 0.005; // board_top is the stand top's surface
    cs.add(&board);
    // 64 squares.
    for ci in 0..8 {
        for cj in 0..8 {
            let is_light = (ci + cj) % 2 == 0;
            let cell = mesh(plane_geo!(cell_size * 0.99, cell_size * 0.99), if is_light { &light_square } else { &dark_square });
            cell.rotation().x = -PI / 2.0;
            cell.position().set(
                -board_size / 2.0 + (ci as f64 + 0.5) * cell_size,
                board_top + 0.0105,
                -board_size / 2.0 + (cj as f64 + 0.5) * cell_size,
            );
            cs.add(&cell);
        }
    }
    // Mid-game piece configuration.
    let white_piece_mat = lambert!({ color: 0xf0e8d0 });
    let black_piece_mat = lambert!({ color: 0x1a1a10 });
    // Common piece shape (base + body + head). Draws no random numbers.
    let make_piece = |color: &str, ty: &str| -> Obj {
        let gp = group();
        let mat = if color == "white" { &white_piece_mat } else { &black_piece_mat };
        // Base (wide short disc).
        let base = mesh(pg("ChessSideStands__base", cyl_geo!(0.010, 0.011, 0.004, 6)), mat);
        base.position().y = 0.002;
        gp.add(&base);
        if ty == "pawn" {
            let body = mesh(pg("ChessSideStands__pawnBody", cyl_geo!(0.004, 0.008, 0.018, 5)), mat);
            body.position().y = 0.013;
            gp.add(&body);
            let head = mesh(pg("ChessSideStands__pawnHead", sphere_geo!(0.006, 5, 4)), mat);
            head.position().y = 0.026;
            gp.add(&head);
        } else if ty == "rook" {
            let body = mesh(pg("ChessSideStands__rookBody", cyl_geo!(0.008, 0.008, 0.028, 6)), mat);
            body.position().y = 0.018;
            gp.add(&body);
            // Crenellations (4 small teeth on top).
            for i in 0..4 {
                let c_angle = (i as f64 / 4.0) * PI * 2.0;
                let tooth = mesh(pg("ChessSideStands__rookTooth", box_geo!(0.003, 0.004, 0.003)), mat);
                tooth.position().set(c_angle.cos() * 0.006, 0.034, c_angle.sin() * 0.006);
                gp.add(&tooth);
            }
        } else if ty == "knight" {
            let body = mesh(pg("ChessSideStands__knightBody", cyl_geo!(0.006, 0.008, 0.020, 5)), mat);
            body.position().y = 0.014;
            gp.add(&body);
            // Neck/head (angled box).
            let head = mesh(pg("ChessSideStands__knightHead", box_geo!(0.008, 0.014, 0.014)), mat);
            head.position().set(0.002, 0.028, 0);
            head.rotation().z = -0.3;
            gp.add(&head);
        } else if ty == "bishop" {
            let body = mesh(pg("ChessSideStands__bishopBody", cyl_geo!(0.005, 0.008, 0.026, 6)), mat);
            body.position().y = 0.017;
            gp.add(&body);
            let head = mesh(pg("ChessSideStands__bishopHead", sphere_geo!(0.006, 6, 5)), mat);
            head.position().y = 0.033;
            head.scale().y = 1.2;
            gp.add(&head);
        } else if ty == "queen" {
            let body = mesh(pg("ChessSideStands__royalBody", cyl_geo!(0.007, 0.009, 0.030, 6)), mat);
            body.position().y = 0.019;
            gp.add(&body);
            let crown = mesh(pg("ChessSideStands__coronet", cyl_geo!(0.010, 0.008, 0.006, 6)), mat);
            crown.position().y = 0.037;
            gp.add(&crown);
            // 5 tiny points around crown top.
            for i in 0..5 {
                let c_angle = (i as f64 / 5.0) * PI * 2.0;
                let point = mesh(pg("ChessSideStands__point", cone_geo!(0.002, 0.005, 4)), mat);
                point.position().set(c_angle.cos() * 0.007, 0.042, c_angle.sin() * 0.007);
                gp.add(&point);
            }
        } else if ty == "king" {
            let body = mesh(pg("ChessSideStands__royalBody", cyl_geo!(0.008, 0.010, 0.032, 6)), mat);
            body.position().y = 0.020;
            gp.add(&body);
            let crown = mesh(pg("ChessSideStands__coronet", cyl_geo!(0.011, 0.009, 0.006, 6)), mat);
            crown.position().y = 0.040;
            gp.add(&crown);
            // Cross on top.
            let cross_v = mesh(pg("ChessSideStands__crossV", box_geo!(0.002, 0.010, 0.002)), mat);
            cross_v.position().y = 0.049;
            gp.add(&cross_v);
            let cross_h = mesh(pg("ChessSideStands__crossH", box_geo!(0.006, 0.002, 0.002)), mat);
            cross_h.position().y = 0.048;
            gp.add(&cross_h);
        }
        gp
    };
    // 8-12 mid-game pieces at random legal-looking positions.
    let piece_count = 8 + (rnd() * 5.0).floor() as i32;
    let piece_types = ["pawn", "pawn", "pawn", "rook", "knight", "bishop", "queen", "king"];
    let mut used_positions: HashSet<(i32, i32)> = HashSet::new();
    for i in 0..piece_count {
        // Random square (avoid duplicates).
        let (mut ci, mut cj) = (0i32, 0i32);
        let mut attempts = 0;
        loop {
            ci = (rnd() * 8.0).floor() as i32;
            cj = (rnd() * 8.0).floor() as i32;
            attempts += 1;
            if !(used_positions.contains(&(ci, cj)) && attempts < 30) {
                break;
            }
        }
        if attempts >= 30 {
            continue;
        }
        used_positions.insert((ci, cj));
        let color = if rnd() < 0.5 { "white" } else { "black" };
        let ty = piece_types[(rnd() * piece_types.len() as f64).floor() as usize];
        let piece = make_piece(color, ty);
        piece.position().set(
            -board_size / 2.0 + (ci as f64 + 0.5) * cell_size,
            board_top + 0.0106, // base on the squares
            -board_size / 2.0 + (cj as f64 + 0.5) * cell_size,
        );
        piece.rotation().y = rnd() * PI * 2.0;
        cs.add(&piece);
    }
    // Captured pieces to the side of the board (2-4 pieces).
    let captured_count = 2 + (rnd() * 3.0).floor() as i32;
    for i in 0..captured_count {
        let color = if rnd() < 0.5 { "white" } else { "black" };
        let captured = make_piece(color, "pawn");
        // Lay on side, in a row along the board's +X edge.
        captured.rotation().x = PI / 2.0;
        captured.position().set(board_size / 2.0 + 0.0125, board_top + 0.0105, -0.070 + i as f64 * 0.036);
        cs.add(&captured);
    }
    // 30%: small brandy snifter beside board.
    if rnd() < 0.3 {
        let glass_mat = lambert!({ color: 0xf0f0f8, transparent: true, opacity: 0.20 });
        let brandy_mat = lambert!({ color: 0x8b4513, transparent: true, opacity: 0.85 });
        // Standing on the stand top, inside its (widened) edge.
        let foot = mesh(cyl_geo!(0.020, 0.022, 0.003, 10), &glass_mat);
        foot.position().set(-board_size / 2.0 - 0.033, board_top + 0.0015, 0.030);
        cs.add(&foot);
        let stem = mesh(cyl_geo!(0.003, 0.003, 0.010, 6), &glass_mat);
        stem.position().set(-board_size / 2.0 - 0.033, board_top + 0.008, 0.030);
        cs.add(&stem);
        let bowl = mesh(sphere_geo!(0.020, 8, 6), &glass_mat);
        bowl.scale().y = 0.85;
        bowl.position().set(-board_size / 2.0 - 0.033, board_top + 0.030, 0.030);
        cs.add(&bowl);
        let liq = mesh(sphere_geo!(0.017, 8, 5), &brandy_mat);
        liq.scale().y = 0.55;
        liq.position().set(-board_size / 2.0 - 0.033, board_top + 0.025, 0.030);
        cs.add(&liq);
    }
    // 30%: cigar in a small tray on other side.
    if rnd() < 0.3 {
        let tray_mat = lambert!({ color: 0x404040 });
        let tray = mesh(pg("ChessSideStands__tray", cyl_geo!(0.025, 0.025, 0.004, 8)), &tray_mat);
        tray.position().set(-board_size / 2.0 - 0.033, board_top + 0.002, -0.040); // on the stand top
        cs.add(&tray);
        let cigar_mat = lambert!({ color: 0x604020 });
        let cigar = mesh(pg("ChessSideStands__cigar", cyl_geo!(0.005, 0.005, 0.055, 6)), &cigar_mat);
        cigar.rotation().z = PI / 2.0;
        // Across the tray in its two rim notches.
        cigar.position().set(-board_size / 2.0 - 0.032, board_top + 0.0075, -0.040);
        cs.add(&cigar);
        // Small ember tip (red).
        let ember = mesh(sphere_geo!(0.003, 5, 4), basic_mat!({ color: 0xd04010 }));
        ember.position().set(-board_size / 2.0 - 0.0055, board_top + 0.0075, -0.040);
        cs.add(&ember);
    }
    let px = g.wx(x) + (rnd() - 0.5) * 1.6;
    let pz = g.wz(z) + (rnd() - 0.5) * 1.6;
    cs.position().set(px, 0, pz);
    cs.rotation().y = rnd() * PI * 2.0;
    g.world.add(&cs);
    let p = cs.pos();
    g.pp.chess_set_positions.push(v3(p.x, 0, p.z)); // R763
}

// R764 dress form / tailor's mannequin on tripod stand (floor): 2.5% chance.
fn place_dress_forms(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.975 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    // Torso fabric colour pool — muslin/canvas naturals.
    let torso_colors = [0xe0d8c0, 0xd0c8b0, 0xc8bfa0, 0xb8a898];
    let torso_mat = lambert!({ color: choice(&torso_colors) });
    let wood_mat = lambert!({ color: 0x604020 });
    let brass_mat = lambert!({ color: 0xa07030, emissive: 0x1a1005 });
    let df = group();
    // Tripod stand — 3 splayed wooden legs.
    let leg_h = 0.30;
    for i in 0..3 {
        let angle = (i as f64 / 3.0) * PI * 2.0;
        let leg = mesh(pg("DressForms__leg", cyl_geo!(0.014, 0.020, leg_h, 6)), &wood_mat);
        // Splay: foot on its pad at r 0.19, the top leaning 0.548 rad inward
        // into the hub (stretched 10% to reach it).
        let leg_dir = v3(-angle.cos() * 0.548f64.sin(), 0.548f64.cos(), -angle.sin() * 0.548f64.sin());
        leg.quaternion().set_from_unit_vectors(&v3(0, 1, 0), &leg_dir);
        leg.scale().y = 1.1;
        leg.position().set(
            angle.cos() * 0.19 + leg_dir.x * leg_h * 0.55,
            0.008 + leg_dir.y * leg_h * 0.55,
            angle.sin() * 0.19 + leg_dir.z * leg_h * 0.55,
        );
        df.add(&leg);
        // Foot pad at base.
        let foot = mesh(pg("DressForms__foot", cyl_geo!(0.020, 0.024, 0.008, 8)), &wood_mat);
        foot.position().set(angle.cos() * 0.19, 0.004, angle.sin() * 0.19);
        df.add(&foot);
    }
    // Central tripod hub cap.
    let hub = mesh(pg("DressForms__hub", sphere_geo!(0.024, 8, 6)), &wood_mat);
    hub.position().y = leg_h;
    df.add(&hub);
    // Central vertical pole (up to torso bottom).
    let pole_h = 0.60;
    let pole = mesh(pg("DressForms__pole", cyl_geo!(0.012, 0.012, pole_h, 6)), &wood_mat);
    pole.position().y = leg_h + pole_h / 2.0;
    df.add(&pole);
    // Brass adjustment wheel on pole.
    let wheel = mesh(pg("DressForms__wheel", torus_geo!(0.016, 0.004, 4, 12)), &brass_mat);
    wheel.rotation().x = PI / 2.0; // horizontal round the pole, like the neck wheel
    wheel.position().y = leg_h + pole_h * 0.4;
    df.add(&wheel);
    // Torso form — 3 stacked ellipsoids for bust / waist / hip.
    let torso_base_y = leg_h + pole_h;
    // Hip (widest).
    let hip = mesh(pg("DressForms__hip", sphere_geo!(0.14, 12, 8)), &torso_mat);
    hip.scale().set(1.0, 0.45, 0.85);
    hip.position().y = torso_base_y + 0.06;
    df.add(&hip);
    // Waist (narrowest), stretched from the hip's top to the bust's bottom.
    let waist = mesh(pg("DressForms__waist", sphere_geo!(0.10, 12, 8)), &torso_mat);
    waist.scale().set(1.0, 0.72, 0.85);
    waist.position().y = torso_base_y + 0.199;
    df.add(&waist);
    // Bust (medium, higher).
    let bust = mesh(pg("DressForms__bust", sphere_geo!(0.13, 12, 8)), &torso_mat);
    bust.scale().set(1.0, 0.577, 0.85);
    bust.position().y = torso_base_y + 0.35;
    df.add(&bust);
    // Shoulder narrow top.
    let shoulder = mesh(pg("DressForms__shoulder", cyl_geo!(0.08, 0.11, 0.06, 8)), &torso_mat);
    shoulder.position().y = torso_base_y + 0.46;
    df.add(&shoulder);
    // Neck/collar top (no head).
    let collar = mesh(pg("DressForms__collar", cyl_geo!(0.030, 0.040, 0.030, 8)), &torso_mat);
    collar.position().y = torso_base_y + 0.51;
    df.add(&collar);
    // Brass adjustment wheel at neck top.
    let neck_wheel = mesh(pg("DressForms__wheel", torus_geo!(0.014, 0.003, 4, 12)), &brass_mat);
    neck_wheel.rotation().x = PI / 2.0;
    neck_wheel.position().y = torso_base_y + 0.530;
    df.add(&neck_wheel);
    // Visible seam lines running vertically along torso.
    let seam_mat = lambert!({ color: 0x6a5a48 });
    for seam_angle in [0.0, PI] {
        let seam = mesh(box_geo!(0.002, 0.44, 0.001), &seam_mat);
        seam.position().set(seam_angle.cos() * 0.115, torso_base_y + 0.22, seam_angle.sin() * 0.115);
        df.add(&seam);
    }
    // 40%: pincushion on hip area.
    if rnd() < 0.4 {
        let cushion_mat = lambert!({ color: 0x8b1010 });
        let cushion = mesh(pg("DressForms__cushion", sphere_geo!(0.018, 8, 6)), &cushion_mat);
        cushion.scale().y = 0.75;
        cushion.position().set(0.10, torso_base_y + 0.06, 0.08);
        df.add(&cushion);
        // 4-6 tiny pin heads.
        let pin_count = 4 + (rnd() * 3.0).floor() as i32;
        for i in 0..pin_count {
            let pin_angle = rnd() * PI;
            let pin_shaft = mesh(cyl_geo!(0.0006, 0.0006, 0.015, 3), lambert!({ color: 0xa0a0a8 }));
            let pin_angle_z = pin_angle - PI / 2.0;
            pin_shaft.rotation().z = pin_angle_z;
            pin_shaft.position().set(0.10 + pin_angle.cos() * 0.020, torso_base_y + 0.06 + pin_angle.sin() * 0.015, 0.08);
            df.add(&pin_shaft);
            // Head — coloured bead.
            let bead_colors = [0xf0d040, 0xa02040, 0x2060a0, 0x408060];
            let bead_mat = lambert!({ color: choice(&bead_colors) });
            let bead = mesh(sphere_geo!(0.0018, 5, 4), &bead_mat);
            bead.position().set(0.10 + pin_angle.cos() * 0.028, torso_base_y + 0.06 + pin_angle.sin() * 0.023, 0.08);
            df.add(&bead);
        }
    }
    // 30%: partially draped fabric on torso (unfinished dress).
    if rnd() < 0.3 {
        let fabric_colors = [0x603030, 0x2a4060, 0x604020, 0x503060, 0x604040];
        let fabric_mat = lambert!({ color: choice(&fabric_colors) });
        // Stood up against the bust's front, narrowed to 18 cm, tilted 0.15.
        let drape = mesh(pg("DressForms__drape", box_geo!(0.24, 0.005, 0.15)), &fabric_mat);
        drape.scale().x = 0.75;
        {
            let mut r = drape.rotation();
            r.set(PI / 2.0, 0, 0.15);
            r.order = EulerOrder::ZXY;
        }
        drape.position().set(0, torso_base_y + 0.33, 0.113);
        df.add(&drape);
        // Fabric flowing down side.
        let flow = mesh(pg("DressForms__flow", box_geo!(0.008, 0.22, 0.10)), &fabric_mat);
        flow.position().set(0.095, torso_base_y + 0.14, 0.10);
        flow.rotation().z = 0.12;
        df.add(&flow);
        // 3 pin markings holding fabric in place.
        for i in 0..3 {
            let pin_mark = mesh(sphere_geo!(0.0018, 4, 4), lambert!({ color: 0xa0a0a8 }));
            let pz = 0.1162 + rnd() * 0.001; // heads on the drape's face
            pin_mark.position().set(-0.06 + i as f64 * 0.06, torso_base_y + 0.30, pz);
            df.add(&pin_mark);
        }
        // R767 slow breeze-sway animator on the loose flowing strip.
        let sway_freq = 0.30 + rnd() * 0.25;
        let sway_phase = rnd() * PI * 2.0;
        let flow_base_rot_z = flow.rot().z;
        g.animate(move |_g, _dt, t| {
            flow.rotation().z = flow_base_rot_z + (t * sway_freq + sway_phase).sin() * 0.028;
        });
    }
    let px = g.wx(x) + (rnd() - 0.5) * 1.0;
    let pz = g.wz(z) + (rnd() - 0.5) * 1.0;
    df.position().set(px, 0, pz);
    df.rotation().y = rnd() * PI * 2.0;
    g.world.add(&df);
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
