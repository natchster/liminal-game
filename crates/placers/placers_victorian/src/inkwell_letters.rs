//! `place_inkwell_letters`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::ri;

// Fountain pen + inkwell + partially-written letter (floor): 4% chance.
pub(crate) fn place_inkwell_letters(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[ri(open.len())];
    let iw = group();
    // Letter sheet on floor with partial-writing texture.
    let paper_mat = lambert!({ map: partial_letter_texture() });
    let (sheet_w, sheet_d) = (0.16, 0.20);
    let sheet = mesh(pg("InkwellLetters__sheet", box_geo!(sheet_w, 0.002, sheet_d)), &paper_mat);
    sheet.position().y = 0.001;
    sheet.rotation().y = (rnd() - 0.5) * 0.3;
    iw.add(&sheet);
    // Fountain pen laid diagonally across letter.
    let pen_colors: [u32; 5] = [0x1a1a2a, 0x2a1a10, 0x501010, 0x2a3a4a, 0x604020];
    let pen_mat = lambert!({ color: pen_colors[ri(pen_colors.len())] });
    let gold_mat = lambert!({ color: 0xd4b060 });
    let silver_mat = lambert!({ color: 0xc0c0c8 });
    let pen = group();
    let barrel_len = 0.11;
    let barrel = mesh(pg("InkwellLetters__barrel", cyl_geo!(0.006, 0.005, barrel_len, 8)), &pen_mat);
    barrel.rotation().z = PI / 2.0;
    pen.add(&barrel);
    // Metal cap section.
    let use_gold = rnd() < 0.5;
    let trim_mat = if use_gold { gold_mat.clone() } else { silver_mat.clone() };
    let cap_sec = mesh(pg("InkwellLetters__capband", cyl_geo!(0.0065, 0.0065, 0.020, 8)), &trim_mat);
    cap_sec.rotation().z = PI / 2.0;
    cap_sec.position().x = -barrel_len * 0.45;
    pen.add(&cap_sec);
    // Clip on cap.
    let clip = mesh(pg("InkwellLetters__clip", box_geo!(0.016, 0.003, 0.003)), &trim_mat);
    clip.position().set(-barrel_len * 0.45, 0.007, 0);
    pen.add(&clip);
    // Nib section — tapered gold tip.
    let nib = mesh(pg("InkwellLetters__nib", cyl_geo!(0.0015, 0.004, 0.014, 8)), &trim_mat);
    nib.rotation().z = -PI / 2.0;
    nib.position().x = barrel_len * 0.55;
    pen.add(&nib);
    // Tiny black ink drop hanging from nib tip.
    let ink_drop = mesh(pg("InkwellLetters__drop", sphere_geo!(0.002, 5, 4)), lambert!({ color: 0x080810 }));
    let ink_drop_base_y = -0.001;
    ink_drop.position().set(barrel_len * 0.62, ink_drop_base_y, 0);
    pen.add(&ink_drop);
    // Ink drop grow+drop animator.
    let drop_cycle = 6.0 + rnd() * 4.0;
    let drop_phase = rnd() * drop_cycle;
    g.animate(move |_g, _dt, t| {
        let local = ((t + drop_phase) % drop_cycle) / drop_cycle;
        if local < 0.85 {
            let s = 1.0 + local * 0.9;
            ink_drop.scale().set(s, s + local * 0.4, s);
            ink_drop.position().y = ink_drop_base_y - local * 0.0008;
            ink_drop.set_visible(true);
        } else if local < 0.92 {
            let fall_local = (local - 0.85) / 0.07;
            ink_drop.position().y = ink_drop_base_y - 0.001 - fall_local * 0.010;
            ink_drop.scale().set_scalar(1.9 * (1.0 - fall_local * 0.5));
        } else {
            ink_drop.set_visible(false);
        }
    });
    // Position pen laid diagonally across letter.
    pen.position().set(sheet_w * 0.15, 0.008, sheet_d * 0.10);
    pen.rotation().y = -PI / 3.0 + (rnd() - 0.5) * 0.4;
    iw.add(&pen);
    // Inkwell — small glass jar with dark liquid + cap.
    let inkwell = group();
    let ink_glass_mat = lambert!({ color: 0x505860, transparent: true, opacity: 0.55 });
    let jar_r = 0.024;
    let jar = mesh(cyl_geo!(jar_r, jar_r * 0.95, 0.028, 12), &ink_glass_mat);
    jar.position().y = 0.014;
    inkwell.add(&jar);
    // Ink inside — black / blue-black / sepia-brown.
    let ink_colors: [u32; 3] = [0x080810, 0x0a1428, 0x1a1005];
    let ink_color = ink_colors[ri(ink_colors.len())];
    let ink_mat = lambert!({ color: ink_color, transparent: true, opacity: 0.95 });
    let ink_vol = mesh(cyl_geo!(jar_r - 0.003, jar_r - 0.003, 0.020, 10), &ink_mat);
    ink_vol.position().y = 0.011;
    inkwell.add(&ink_vol);
    let ink_surf = mesh(circle_geo!(jar_r - 0.003, 12), &ink_mat);
    ink_surf.rotation().x = -PI / 2.0;
    ink_surf.position().y = 0.021;
    inkwell.add(&ink_surf);
    // Neck ring (top of jar).
    let neck = mesh(pg("InkwellLetters__neck", torus_geo!(jar_r, 0.002, 4, 12)), &trim_mat);
    neck.rotation().x = PI / 2.0;
    neck.position().y = 0.028;
    inkwell.add(&neck);
    // Optional open cap on floor beside — 50% chance.
    if rnd() < 0.5 {
        let cap = mesh(pg("InkwellLetters__cap", cyl_geo!(jar_r + 0.001, jar_r + 0.001, 0.008, 12)), &trim_mat);
        cap.rotation().z = (rnd() - 0.5) * 0.3;
        let crz = cap.rot().z;
        cap.position().set(-0.030, 0.004 + (jar_r + 0.001) * crz.sin().abs(), -0.043);
        inkwell.add(&cap);
    }
    // Beside the letter, clear of its left edge.
    inkwell.position().set(-sheet_w * 0.5 - jar_r - 0.014, 0, sheet_d * 0.15);
    iw.add(&inkwell);
    // 30% chance: small ink smudge stain on the paper.
    if rnd() < 0.3 {
        let smudge_mat = lambert!({ color: ink_color, transparent: true, opacity: 0.5 });
        let smudge = mesh(circle_geo!(0.008, 5), &smudge_mat);
        smudge.rotation().x = -PI / 2.0;
        smudge.position().set(sheet_w * 0.30, 0.003, -sheet_d * 0.15);
        iw.add(&smudge);
    }
    // 25% chance: second blank sheet of stationery beside.
    if rnd() < 0.25 {
        let blank_mat = lambert!({ color: 0xf4ecd0 });
        let blank = mesh(pg("InkwellLetters__sheet", box_geo!(sheet_w, 0.002, sheet_d)), &blank_mat);
        blank.position().set(sheet_w + 0.02, 0.001, 0);
        blank.rotation().y = (rnd() - 0.5) * 0.3;
        iw.add(&blank);
    }
    iw.position().set(g.wx(x) + (rnd() - 0.5) * 1.8, 0, g.wz(z) + (rnd() - 0.5) * 1.8);
    iw.rotation().y = rnd() * PI * 2.0;
    g.world.add(&iw);
}

// Partial letter texture: mid-word interruption pattern.
game::lazy_static_tex!(
    partial_letter_texture,
    canvas_tex(180, 220, |ctx, w, h| {
        ctx.set_fill_style("#f4ecd0");
        ctx.fill_rect(0, 0, w, h);
        // Faint blue-ruled lines.
        ctx.set_stroke_style("rgba(120,140,180,0.28)");
        ctx.set_line_width(0.7);
        let mut y = 24.0;
        while y < h - 10.0 {
            ctx.begin_path();
            ctx.move_to(8, y);
            ctx.line_to(w - 8.0, y);
            ctx.stroke();
            y += 14.0;
        }
        // Red left margin.
        ctx.set_stroke_style("rgba(200,60,60,0.35)");
        ctx.set_line_width(1);
        ctx.begin_path();
        ctx.move_to(18, 8);
        ctx.line_to(18, h - 8.0);
        ctx.stroke();
        // Date top right.
        let dates = ["August 3, 1958", "April 17, 1971", "November 9, 1965", "February 24, 1982"];
        ctx.set_fill_style("#302010");
        ctx.set_font("italic 11px cursive");
        ctx.set_text_align("right");
        ctx.fill_text(dates[ri(dates.len())], w - 12.0, 20);
        // Greeting.
        let names = ["Sarah", "Michael", "Emma", "James", "Rose", "Thomas", "Anna", "David"];
        ctx.set_text_align("left");
        ctx.set_font("italic 16px cursive");
        ctx.fill_text(&format!("Dear {},", names[ri(names.len())]), 24, 44);
        // 3-4 lines of wavy cursive — then cut off.
        ctx.set_stroke_style("rgba(50,35,20,0.75)");
        ctx.set_line_width(1.2);
        let lines_filled = 3 + (rnd() * 2.0).floor() as i32;
        let (mut last_x, mut last_y) = (24.0, 66.0);
        for i in 0..lines_filled {
            let y = 66.0 + i as f64 * 14.0;
            let is_last = i == lines_filled - 1;
            let line_len = if is_last { 20.0 + rnd() * 40.0 } else { 100.0 + rnd() * 40.0 };
            ctx.begin_path();
            ctx.move_to(24, y);
            let mut cx = 24.0;
            while cx < 24.0 + line_len {
                let seg_len = 6.0 + rnd() * 8.0;
                let nx = cx + seg_len;
                let dy = (rnd() - 0.5) * 4.0;
                ctx.quadratic_curve_to(cx + seg_len / 2.0, y + dy * 1.5, nx, y + dy);
                cx = nx;
            }
            ctx.stroke();
            if is_last {
                last_x = 24.0 + line_len;
                last_y = y;
            }
        }
        // Big ink blot at cut-off.
        ctx.set_fill_style("rgba(20,15,10,0.92)");
        ctx.begin_path();
        let r = 4.0 + rnd() * 2.0;
        ctx.arc(last_x + 3.0, last_y + 1.0, r, 0, PI * 2.0);
        ctx.fill();
        // Splatter — 3-5 small satellite droplets.
        let splatter_n = 3 + (rnd() * 3.0).floor() as i32;
        for _ in 0..splatter_n {
            let sx = last_x + 3.0 + (rnd() - 0.5) * 18.0;
            let sy = last_y + 1.0 + (rnd() - 0.5) * 12.0;
            ctx.begin_path();
            let r = 0.8 + rnd() * 1.5;
            ctx.arc(sx, sy, r, 0, PI * 2.0);
            ctx.fill();
        }
        // Trailing ink smear from blot.
        ctx.set_stroke_style("rgba(20,15,10,0.5)");
        ctx.set_line_width(2);
        ctx.begin_path();
        ctx.move_to(last_x + 3.0, last_y + 1.0);
        ctx.quadratic_curve_to(last_x + 12.0, last_y + 6.0, last_x + 24.0, last_y + 3.0);
        ctx.stroke();
        // Fold crease.
        ctx.set_stroke_style("rgba(120,90,60,0.30)");
        ctx.set_line_width(1.5);
        ctx.begin_path();
        ctx.move_to(0, h * 0.5);
        ctx.line_to(w, h * 0.5);
        ctx.stroke();
    })
);
