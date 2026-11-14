//! `place_correspondence_cards`
use game::prelude::*;
use placers_kit::place::mount_on_wall;
use placers_kit::place::wall_candidates;

pub(crate) fn place_correspondence_cards(g: &mut Game, _dist: &[i32]) {
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

// Framed correspondence: telegram / calling card / formal invitation.
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
            ctx.fill_text(&format!("DATE: {date}"), 12, 54);
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
            ctx.fill_text(&format!("— {name2}"), w / 2.0, 184);
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
