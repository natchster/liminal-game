//! `place_vintage_ads`
use game::prelude::*;
use placers_kit::place::face_wall;
use placers_kit::place::pick;
use placers_kit::place::wall_tiles;

pub(crate) fn place_vintage_ads(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    }
    let wt = wall_tiles(g);
    if wt.is_empty() {
        return;
    }
    let (x, z, dx, dz) = wt[(rnd() * wt.len() as f64).floor() as usize];
    let frame_colors = [0x604020, 0x8b5a2b, 0x101010, 0xc0a060];
    let frame_mat = lambert!({ color: choice(&frame_colors) });
    let ad_mat = lambert!({ map: vintage_ad_texture() });
    let va = group();
    let mount_y = 1.60;
    let (frame_w, frame_h) = (0.20, 0.28);
    let frame = mesh(pg("VintageAds__frame", box_geo!(frame_w, frame_h, 0.012)), &frame_mat);
    frame.position().set(0, mount_y, 0.010);
    va.add(&frame);
    let ad = mesh(plane_geo!(frame_w - 0.014, frame_h - 0.014), &ad_mat);
    ad.position().set(0, mount_y, 0.017);
    va.add(&ad);
    va.rotation().y = face_wall(dx, dz);
    va.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - 0.015), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - 0.015));
    va.rotation().z = (rnd() - 0.5) * 0.03;
    g.world.add(&va);
}

game::lazy_static_tex!(
    vintage_ad_texture,
    canvas_tex(200, 260, |ctx, w, h| {
        // (name, tag, bg, accent, icon)
        let variants: [(&str, &str, &str, &str, &str); 6] = [
            ("COLA", "REFRESHING & DELICIOUS", "#a02020", "#f0e8c0", "bottle"),
            ("SMOOTHER", "DOCTORS RECOMMEND", "#603030", "#f0e0a0", "cigarette"),
            ("BREAKFAST OF CHAMPIONS", "START YOUR DAY RIGHT", "#f0c040", "#a02020", "cereal"),
            ("FILL 'ER UP", "HAPPY MOTORING", "#204080", "#f0e040", "pump"),
            ("SEE THE USA", "IN YOUR CHEVROLET", "#c04020", "#f0f0e0", "car"),
            ("BEAUTY SLEEP", "FEEL YOUR BEST", "#a06090", "#f0e0f0", "face"),
        ];
        let v = variants[(rnd() * variants.len() as f64).floor() as usize];
        let (name, tag, bg, accent, icon) = v;
        ctx.set_fill_style(bg);
        ctx.fill_rect(0, 0, w, h);
        // Border stripe.
        ctx.set_stroke_style(accent);
        ctx.set_line_width(5);
        ctx.stroke_rect(10, 10, w - 20.0, h - 20.0);
        // Product name.
        ctx.set_fill_style(accent);
        ctx.set_font("bold 26px serif");
        ctx.set_text_align("center");
        ctx.fill_text(name, w / 2.0, 50);
        // Tagline.
        ctx.set_font("italic bold 12px serif");
        ctx.fill_text(tag, w / 2.0, 72);
        // Central product illustration.
        let (cx, cy) = (w / 2.0, h * 0.55);
        ctx.set_fill_style(accent);
        if icon == "bottle" {
            ctx.begin_path();
            ctx.move_to(cx - 15.0, cy - 40.0);
            ctx.line_to(cx - 15.0, cy - 25.0);
            ctx.quadratic_curve_to(cx - 25.0, cy - 15.0, cx - 25.0, cy + 20.0);
            ctx.line_to(cx - 22.0, cy + 45.0);
            ctx.line_to(cx + 22.0, cy + 45.0);
            ctx.line_to(cx + 25.0, cy + 20.0);
            ctx.quadratic_curve_to(cx + 25.0, cy - 15.0, cx + 15.0, cy - 25.0);
            ctx.line_to(cx + 15.0, cy - 40.0);
            ctx.close_path();
            ctx.fill();
            // Bottle cap.
            ctx.set_fill_style(bg);
            ctx.fill_rect(cx - 12.0, cy - 42.0, 24, 6);
            // Label band.
            ctx.set_fill_style(bg);
            ctx.fill_rect(cx - 26.0, cy + 5.0, 52, 20);
        } else if icon == "cigarette" {
            ctx.fill_rect(cx - 32.0, cy - 30.0, 64, 60);
            ctx.set_stroke_style(bg);
            ctx.set_line_width(3);
            ctx.stroke_rect(cx - 32.0, cy - 30.0, 64, 60);
            ctx.set_fill_style(bg);
            ctx.set_font("bold 14px serif");
            ctx.set_text_align("center");
            ctx.fill_text("CAMEL", cx, cy - 5.0);
            ctx.set_font("9px serif");
            ctx.fill_text("20 CLASS A", cx, cy + 10.0);
            // Single cigarette poking out.
            ctx.set_fill_style(accent);
            ctx.fill_rect(cx + 30.0, cy - 40.0, 4, 20);
            ctx.set_fill_style("#a06030");
            ctx.fill_rect(cx + 30.0, cy - 30.0, 4, 5);
        } else if icon == "cereal" {
            ctx.fill_rect(cx - 30.0, cy - 35.0, 60, 70);
            ctx.set_stroke_style(bg);
            ctx.set_line_width(3);
            ctx.stroke_rect(cx - 30.0, cy - 35.0, 60, 70);
            ctx.set_fill_style(bg);
            ctx.set_font("bold 12px serif");
            ctx.set_text_align("center");
            ctx.fill_text("WHEATIES", cx, cy - 10.0);
            ctx.begin_path();
            ctx.arc(cx, cy + 15.0, 12, 0, PI * 2.0);
            ctx.fill();
        } else if icon == "pump" {
            ctx.fill_rect(cx - 20.0, cy - 45.0, 40, 70);
            ctx.set_stroke_style(bg);
            ctx.set_line_width(3);
            ctx.stroke_rect(cx - 20.0, cy - 45.0, 40, 70);
            // Nozzle handle.
            ctx.set_fill_style(accent);
            ctx.fill_rect(cx + 15.0, cy - 20.0, 20, 8);
            // Dial.
            ctx.set_fill_style(bg);
            ctx.begin_path();
            ctx.arc(cx, cy - 25.0, 12, 0, PI * 2.0);
            ctx.fill();
        } else if icon == "car" {
            ctx.begin_path();
            ctx.move_to(cx - 40.0, cy + 10.0);
            ctx.line_to(cx - 30.0, cy - 10.0);
            ctx.line_to(cx - 10.0, cy - 18.0);
            ctx.line_to(cx + 20.0, cy - 18.0);
            ctx.line_to(cx + 35.0, cy - 8.0);
            ctx.line_to(cx + 40.0, cy + 10.0);
            ctx.line_to(cx - 40.0, cy + 10.0);
            ctx.close_path();
            ctx.fill();
            // Wheels.
            ctx.set_fill_style(bg);
            ctx.begin_path();
            ctx.arc(cx - 25.0, cy + 12.0, 8, 0, PI * 2.0);
            ctx.arc(cx + 25.0, cy + 12.0, 8, 0, PI * 2.0);
            ctx.fill();
        } else if icon == "face" {
            ctx.begin_path();
            ctx.arc(cx, cy - 10.0, 25, 0, PI * 2.0);
            ctx.fill();
            // Eyes (2 dots).
            ctx.set_fill_style(bg);
            ctx.begin_path();
            ctx.arc(cx - 8.0, cy - 12.0, 2, 0, PI * 2.0);
            ctx.arc(cx + 8.0, cy - 12.0, 2, 0, PI * 2.0);
            ctx.fill();
            // Smile.
            ctx.set_stroke_style(bg);
            ctx.set_line_width(2);
            ctx.begin_path();
            ctx.arc(cx, cy - 8.0, 8, 0.4, PI - 0.4);
            ctx.stroke();
            // Bottle/product.
            ctx.set_fill_style(accent);
            ctx.fill_rect(cx - 8.0, cy + 25.0, 16, 25);
        }
        // Price/CTA at bottom.
        ctx.set_fill_style(accent);
        ctx.set_font("bold 14px serif");
        ctx.set_text_align("center");
        ctx.fill_text(
            pick(&["NOW ONLY 5¢", "ASK YOUR GROCER", "AVAILABLE EVERYWHERE", "GET YOURS TODAY", "BUY AMERICAN", "ORDER NOW"]),
            w / 2.0,
            h - 25.0,
        );
        // Year — small text bottom.
        ctx.set_font("italic 9px serif");
        ctx.fill_text(pick(&["© 1948", "© 1953", "© 1957", "© 1962", "© 1965"]), w / 2.0, h - 12.0);
    })
);
