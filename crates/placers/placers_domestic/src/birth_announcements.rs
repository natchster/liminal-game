//! `place_birth_announcements`
use game::prelude::*;
use placers_kit::place::pick;
use placers_kit::place::wall_tiles;

// Framed hospital birth announcement (wall): 2.5% chance per level.
pub(crate) fn place_birth_announcements(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.975 {
        return;
    }
    let wall_tiles = wall_tiles(g);
    if wall_tiles.is_empty() {
        return;
    }
    let (x, z, dx, dz) = wall_tiles[(rnd() * wall_tiles.len() as f64).floor() as usize];
    let frame_colors = [0xf0e0e8, 0xe0e8f0, 0xf0e8d0, 0xd0a040];
    let frame_mat = lambert!({ color: pick(&frame_colors) });
    let card_mat = lambert!({ map: birth_announcement_texture() });
    let ba = group();
    let mount_y = 1.55;
    let (frame_w, frame_h) = (0.20, 0.22);
    let frame = mesh(pg("BirthAnnouncements__frame", box_geo!(frame_w, frame_h, 0.014)), &frame_mat);
    frame.position().set(0, mount_y, 0.011);
    ba.add(&frame);
    let card = mesh(plane_geo!(frame_w - 0.020, frame_h - 0.020), &card_mat);
    card.position().set(0, mount_y, 0.019);
    ba.add(&card);
    ba.rotation().y = (-(dx as f64)).atan2(-(dz as f64));
    ba.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - 0.015), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - 0.015));
    ba.rotation().z = (rnd() - 0.5) * 0.04;
    g.world.add(&ba);
}

// Framed hospital birth announcement: pastel-bordered card w/ baby
// footprints + hospital info + name + weight/length + time.
game::lazy_static_tex!(
    birth_announcement_texture,
    canvas_tex(200, 220, |ctx, w, h| {
        // Pastel gradient background (pink or blue).
        let is_pink = rnd() < 0.5;
        let grad = ctx.create_linear_gradient(0, 0, 0, h);
        if is_pink {
            grad.add_color_stop(0, "#f8dae0");
            grad.add_color_stop(1, "#f0c0c8");
        } else {
            grad.add_color_stop(0, "#dae5f8");
            grad.add_color_stop(1, "#c0d0f0");
        }
        ctx.set_fill_style(&grad);
        ctx.fill_rect(0, 0, w, h);
        // Decorative border stripe.
        ctx.set_stroke_style(if is_pink { "#c07080" } else { "#6080c0" });
        ctx.set_line_width(2);
        ctx.stroke_rect(8, 8, w - 16.0, h - 16.0);
        // Small decorative header text: "IT'S A BOY!" or "IT'S A GIRL!"
        ctx.set_fill_style(if is_pink { "#a02050" } else { "#204080" });
        ctx.set_font("bold 18px serif");
        ctx.set_text_align("center");
        ctx.fill_text(if is_pink { "IT'S A GIRL!" } else { "IT'S A BOY!" }, w / 2.0, 32);
        // Name section — italic large.
        let girl_names = ["Emma Rose", "Anna Marie", "Lily Grace", "Sarah Jane", "Baby Grace"];
        let boy_names = ["James Michael", "Thomas John", "Baby David", "Michael James", "Noah Andrew"];
        let name = (if is_pink { girl_names } else { boy_names })[(rnd() * 5.0).floor() as usize];
        ctx.set_fill_style("#302010");
        ctx.set_font("italic bold 20px cursive");
        ctx.fill_text(name, w / 2.0, 62);
        // Birth details.
        ctx.set_font("12px serif");
        ctx.set_fill_style("#403020");
        let months = ["January", "March", "June", "September", "November"];
        let day = 1 + (rnd() * 28.0).floor() as i64;
        let year = 1970 + (rnd() * 25.0).floor() as i64;
        let hour = 1 + (rnd() * 11.0).floor() as i64;
        let min = (rnd() * 60.0).floor() as i64;
        let ampm = if rnd() < 0.5 { "AM" } else { "PM" };
        ctx.fill_text(&format!("Born {} {}, {}", months[(rnd() * 5.0).floor() as usize], day, year), w / 2.0, 82);
        ctx.fill_text(&format!("at {}:{} {}", hour, pad_start(&min.to_string(), 2, '0'), ampm), w / 2.0, 96);
        let lbs = 6 + (rnd() * 3.0).floor() as i64;
        let oz = (rnd() * 16.0).floor() as i64;
        let inches = 18 + (rnd() * 5.0).floor() as i64;
        ctx.fill_text(&format!("{} lbs {} oz \u{00b7} {} inches", lbs, oz, inches), w / 2.0, 114);
        // 2 baby footprints (side by side) — small oval + 5 tiny toe dots each.
        let print_color = if is_pink { "#c0304a" } else { "#204080" };
        ctx.set_fill_style(print_color);
        for side in 0..2 {
            let fx = w / 2.0 + (side as f64 - 0.5) * 30.0;
            let fy = h * 0.62;
            // Foot oval.
            ctx.begin_path();
            ctx.ellipse(fx, fy, 8, 14, 0, 0, PI * 2.0);
            ctx.fill();
            // 5 tiny toe dots above the oval.
            for toe in 0..5 {
                let toe_x = fx + (toe as f64 - 2.0) * 2.5;
                let toe_y = fy
                    - 14.0
                    - (if toe == 0 {
                        3.0
                    } else if toe == 4 {
                        2.0
                    } else {
                        4.0
                    });
                let toe_r = if toe == 0 {
                    2.2
                } else if toe == 4 {
                    1.4
                } else {
                    1.8
                };
                ctx.begin_path();
                ctx.arc(toe_x, toe_y, toe_r, 0, PI * 2.0);
                ctx.fill();
            }
        }
        // Hospital name at bottom.
        ctx.set_font("italic 10px serif");
        ctx.set_fill_style("#403020");
        let hospitals = ["St. Mary's Hospital", "Mercy General", "Riverside Medical Center", "Sacred Heart Hospital"];
        ctx.fill_text(hospitals[(rnd() * 4.0).floor() as usize], w / 2.0, h - 18.0);
        // Doctor name.
        ctx.set_font("9px serif");
        let doctors = ["Anderson", "Williams", "Chen", "Rodriguez"];
        ctx.fill_text(&format!("Dr. {}, attending", doctors[(rnd() * 4.0).floor() as usize]), w / 2.0, h - 8.0);
    })
);
