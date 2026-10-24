//! Module-level canvas textures of chunk p10 (built lazily on the native RNG).
use game::prelude::*;

// ------------------------------------------------------------------ line 21071
// R698 framed hospital birth announcement: pastel-bordered card w/ baby
// footprints + hospital info + name + weight/length + time.
game::lazy_static_tex!(birth_announcement_texture, canvas_tex(200, 220, |ctx, w, h| {
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
            let toe_y = fy - 14.0 - (if toe == 0 { 3.0 } else if toe == 4 { 2.0 } else { 4.0 });
            let toe_r = if toe == 0 { 2.2 } else if toe == 4 { 1.4 } else { 1.8 };
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
}));

// ------------------------------------------------------------------ line 21290
// R700 framed vintage postcard: beach / mountain / cityscape front + stamp.
game::lazy_static_tex!(postcard_texture, canvas_tex(240, 160, |ctx, w, h| {
    // Aged card background — cream/tan.
    ctx.set_fill_style("#f0e0c0");
    ctx.fill_rect(0, 0, w, h);
    // Variant: beach, mountain, or cityscape.
    let variant = (rnd() * 3.0).floor() as usize;
    if variant == 0 {
        // Beach scene. Sky gradient.
        let sky = ctx.create_linear_gradient(0, 0, 0, h * 0.6);
        sky.add_color_stop(0, "#a0d0e0");
        sky.add_color_stop(1, "#f0e0a0");
        ctx.set_fill_style(&sky);
        ctx.fill_rect(0, 0, w, h * 0.6);
        // Sun.
        ctx.set_fill_style("#f0d060");
        ctx.begin_path();
        ctx.arc(w * 0.78, h * 0.25, 20, 0, PI * 2.0);
        ctx.fill();
        // Ocean band.
        let ocean = ctx.create_linear_gradient(0, h * 0.5, 0, h * 0.75);
        ocean.add_color_stop(0, "#4080a0");
        ocean.add_color_stop(1, "#2060a0");
        ctx.set_fill_style(&ocean);
        ctx.fill_rect(0, h * 0.5, w, h * 0.25);
        // Wave lines.
        ctx.set_stroke_style("rgba(255,255,255,0.4)");
        ctx.set_line_width(1);
        for i in 0..8 {
            let y = h * 0.55 + i as f64 * 5.0;
            ctx.begin_path();
            ctx.move_to(0, y);
            let mut x = 0.0;
            while x < w {
                ctx.line_to(x + 4.0, y - 1.0);
                ctx.line_to(x + 8.0, y);
                x += 8.0;
            }
            ctx.stroke();
        }
        // Beach sand.
        ctx.set_fill_style("#e0d090");
        ctx.fill_rect(0, h * 0.75, w, h * 0.20);
        // Palm tree.
        ctx.set_stroke_style("#603020");
        ctx.set_line_width(4);
        ctx.begin_path();
        ctx.move_to(w * 0.15, h * 0.90);
        ctx.quadratic_curve_to(w * 0.14, h * 0.65, w * 0.18, h * 0.45);
        ctx.stroke();
        ctx.set_fill_style("#40803a");
        for i in 0..5 {
            let ang = (i as f64 / 4.0) * PI - PI / 6.0;
            ctx.begin_path();
            ctx.ellipse(w * 0.18, h * 0.45, 22, 5, ang, 0, PI * 2.0);
            ctx.fill();
        }
    } else if variant == 1 {
        // Mountain scene. Sky gradient.
        let sky = ctx.create_linear_gradient(0, 0, 0, h * 0.7);
        sky.add_color_stop(0, "#c0d0e8");
        sky.add_color_stop(1, "#f0e8d0");
        ctx.set_fill_style(&sky);
        ctx.fill_rect(0, 0, w, h * 0.7);
        // Distant mountain silhouettes.
        ctx.set_fill_style("#8090a0");
        ctx.begin_path();
        ctx.move_to(0, h * 0.5);
        ctx.line_to(w * 0.15, h * 0.30);
        ctx.line_to(w * 0.30, h * 0.42);
        ctx.line_to(w * 0.50, h * 0.25);
        ctx.line_to(w * 0.70, h * 0.40);
        ctx.line_to(w * 0.85, h * 0.28);
        ctx.line_to(w, h * 0.45);
        ctx.line_to(w, h * 0.7);
        ctx.line_to(0, h * 0.7);
        ctx.close_path();
        ctx.fill();
        // Foreground darker mountains.
        ctx.set_fill_style("#40506a");
        ctx.begin_path();
        ctx.move_to(0, h * 0.7);
        ctx.line_to(w * 0.20, h * 0.45);
        ctx.line_to(w * 0.40, h * 0.55);
        ctx.line_to(w * 0.60, h * 0.42);
        ctx.line_to(w * 0.80, h * 0.55);
        ctx.line_to(w, h * 0.48);
        ctx.line_to(w, h);
        ctx.line_to(0, h);
        ctx.close_path();
        ctx.fill();
        // Snow caps on peaks (small white triangles).
        ctx.set_fill_style("#f8f8f8");
        ctx.begin_path();
        ctx.move_to(w * 0.15, h * 0.32);
        ctx.line_to(w * 0.20, h * 0.35);
        ctx.line_to(w * 0.24, h * 0.34);
        ctx.close_path();
        ctx.fill();
        ctx.begin_path();
        ctx.move_to(w * 0.50, h * 0.27);
        ctx.line_to(w * 0.56, h * 0.32);
        ctx.line_to(w * 0.60, h * 0.30);
        ctx.close_path();
        ctx.fill();
        // Foreground trees.
        ctx.set_fill_style("#204028");
        for i in 0..6 {
            let tx = 20.0 + i as f64 * 40.0 + (rnd() - 0.5) * 20.0;
            let ty = h * 0.82 + (rnd() - 0.5) * 8.0;
            ctx.begin_path();
            ctx.move_to(tx, ty);
            ctx.line_to(tx - 8.0, ty + 20.0);
            ctx.line_to(tx + 8.0, ty + 20.0);
            ctx.close_path();
            ctx.fill();
        }
    } else {
        // Cityscape. Sky gradient.
        let sky = ctx.create_linear_gradient(0, 0, 0, h * 0.6);
        sky.add_color_stop(0, "#e0a060");
        sky.add_color_stop(1, "#f0d090");
        ctx.set_fill_style(&sky);
        ctx.fill_rect(0, 0, w, h * 0.6);
        // Sun.
        ctx.set_fill_style("#f0e0a0");
        ctx.begin_path();
        ctx.arc(w * 0.70, h * 0.30, 16, 0, PI * 2.0);
        ctx.fill();
        // Silhouette buildings: (x, w, h).
        ctx.set_fill_style("#202030");
        let bldgs: [(f64, f64, f64); 5] = [
            (20.0, 30.0, h * 0.45),
            (60.0, 25.0, h * 0.55),
            (95.0, 45.0, h * 0.35),
            (150.0, 20.0, h * 0.65),
            (180.0, 40.0, h * 0.42),
        ];
        for b in &bldgs {
            ctx.fill_rect(b.0, h - b.2 - h * 0.05, b.1, b.2);
        }
        // Building windows — scattered yellow lit dots.
        ctx.set_fill_style("#f0e090");
        for _ in 0..40 {
            let b = bldgs[(rnd() * bldgs.len() as f64).floor() as usize];
            let wx2 = b.0 + 2.0 + rnd() * (b.1 - 4.0);
            let wy = h - b.2 + 4.0 + rnd() * (b.2 - 8.0);
            if wy < h - h * 0.05 {
                ctx.fill_rect(wx2, wy, 2, 3);
            }
        }
        // Street.
        ctx.set_fill_style("#101014");
        ctx.fill_rect(0, h - h * 0.05, w, h * 0.05);
    }
    // Border margin (matte white).
    ctx.set_stroke_style("rgba(240,220,180,0.9)");
    ctx.set_line_width(6);
    ctx.stroke_rect(3, 3, w - 6.0, h - 6.0);
    // Postcard caption text at bottom — location name.
    ctx.set_fill_style("#603020");
    ctx.set_font("italic bold 16px cursive");
    ctx.set_text_align("center");
    let captions = [
        ["Greetings from Miami", "Sunny California", "Beautiful Bahamas", "Wish You Were Here"],
        ["Rocky Mountains", "Colorado '82", "Big Sky Country", "The Great Divide"],
        ["New York, N.Y.", "Chicago at Dusk", "San Francisco '75", "Boston Skyline"],
    ][variant];
    ctx.fill_text(captions[(rnd() * 4.0).floor() as usize], w / 2.0, h - 14.0);
    // Small stamp corner top-right — dark rectangle w/ inner detail.
    ctx.set_fill_style("#a02040");
    ctx.fill_rect(w - 32.0, 6, 22, 26);
    ctx.set_stroke_style("#f0e0c0");
    ctx.stroke_rect(w - 32.0, 6, 22, 26);
    ctx.set_fill_style("#f0e0c0");
    ctx.set_font("bold 8px serif");
    ctx.set_text_align("center");
    ctx.fill_text("30\u{00a2}", w - 21.0, 22);
    // Postmark circle overlay (subtle).
    ctx.set_stroke_style("rgba(50,30,20,0.35)");
    ctx.set_line_width(1);
    ctx.begin_path();
    ctx.arc(w - 45.0, 20, 14, 0, PI * 2.0);
    ctx.stroke();
}));

// ------------------------------------------------------------------ line 21786
// R704 bulletin board w/ pinned photos + notes: cork board w/ mixed pinned
// items. (Defined in backrooms.js but never referenced by any placer; kept
// for completeness, built only if something asks for it.)
game::lazy_static_tex!(bulletin_texture, canvas_tex(280, 220, |ctx, w, h| {
    // Cork background — tan w/ speckle noise.
    ctx.set_fill_style("#c0a070");
    ctx.fill_rect(0, 0, w, h);
    // Cork speckle — 400 tiny dots.
    for _ in 0..400 {
        let style = format!("rgba({}, {}, {}, {})", 100.0 + rnd() * 50.0, 70.0 + rnd() * 40.0, 40.0 + rnd() * 30.0, 0.3 + rnd() * 0.3);
        ctx.set_fill_style(&style);
        let (sx, sy) = (rnd() * w, rnd() * h);
        let sr = 0.8 + rnd() * 1.5;
        ctx.begin_path();
        ctx.arc(sx, sy, sr, 0, PI * 2.0);
        ctx.fill();
    }
    // 3-4 pinned photos (mini sepia/color photos w/ blurred figures).
    let photo_count = 3 + (rnd() * 2.0).floor() as i32;
    for _ in 0..photo_count {
        let pw = 40.0 + rnd() * 20.0;
        let ph = 32.0 + rnd() * 16.0;
        let px = 20.0 + rnd() * (w - 40.0 - pw);
        let py = 20.0 + rnd() * (h - 60.0 - ph);
        let angle = (rnd() - 0.5) * 0.3;
        ctx.save();
        ctx.translate(px + pw / 2.0, py + ph / 2.0);
        ctx.rotate(angle);
        // Photo cream border.
        ctx.set_fill_style("#f0e8c8");
        ctx.fill_rect(-pw / 2.0, -ph / 2.0, pw, ph);
        // Photo body — sepia/color.
        let is_sepia = rnd() < 0.5;
        let grad = ctx.create_linear_gradient(0, -ph / 2.0 + 4.0, 0, ph / 2.0 - 4.0);
        if is_sepia {
            grad.add_color_stop(0, "#a08060");
            grad.add_color_stop(1, "#604020");
        } else {
            grad.add_color_stop(0, "#8090a0");
            grad.add_color_stop(1, "#405060");
        }
        ctx.set_fill_style(&grad);
        ctx.fill_rect(-pw / 2.0 + 3.0, -ph / 2.0 + 3.0, pw - 6.0, ph - 6.0);
        // Blurred figure (radial gradient blob).
        let fig_grad = ctx.create_radial_gradient(0, 0, 2, 0, 0, pw.min(ph) * 0.4);
        if is_sepia {
            fig_grad.add_color_stop(0, "rgba(50,30,15,0.85)");
            fig_grad.add_color_stop(1, "rgba(70,50,30,0)");
        } else {
            fig_grad.add_color_stop(0, "rgba(30,25,25,0.85)");
            fig_grad.add_color_stop(1, "rgba(60,55,50,0)");
        }
        ctx.set_fill_style(&fig_grad);
        ctx.begin_path();
        ctx.ellipse(0, 0, pw * 0.3, ph * 0.35, 0, 0, PI * 2.0);
        ctx.fill();
        // Push pin at top — small red circle.
        ctx.set_fill_style("#a02040");
        ctx.begin_path();
        ctx.arc(0, -ph / 2.0 + 2.0, 3, 0, PI * 2.0);
        ctx.fill();
        ctx.restore();
    }
    // 2-3 sticky notes (colored squares w/ tiny text lines).
    let note_count = 2 + (rnd() * 2.0).floor() as i32;
    let note_colors = ["#f0e070", "#f0a0c0", "#a0f0a0", "#a0d0f0"];
    for _ in 0..note_count {
        let nw = 30.0 + rnd() * 20.0;
        let nh = 30.0 + rnd() * 12.0;
        let nx = 20.0 + rnd() * (w - 40.0 - nw);
        let ny = 20.0 + rnd() * (h - 60.0 - nh);
        let angle = (rnd() - 0.5) * 0.25;
        ctx.save();
        ctx.translate(nx + nw / 2.0, ny + nh / 2.0);
        ctx.rotate(angle);
        ctx.set_fill_style(note_colors[(rnd() * note_colors.len() as f64).floor() as usize]);
        ctx.fill_rect(-nw / 2.0, -nh / 2.0, nw, nh);
        // Tiny text lines.
        ctx.set_stroke_style("rgba(60,40,20,0.7)");
        ctx.set_line_width(0.7);
        for li in 0..3 {
            let ly = -nh / 2.0 + 6.0 + li as f64 * 7.0;
            ctx.begin_path();
            ctx.move_to(-nw / 2.0 + 3.0, ly);
            ctx.line_to(-nw / 2.0 + 3.0 + (nw - 8.0) * (0.5 + rnd() * 0.4), ly);
            ctx.stroke();
        }
        // Push pin (blue or yellow).
        ctx.set_fill_style(if rnd() < 0.5 { "#4080c0" } else { "#f0c040" });
        ctx.begin_path();
        ctx.arc(0, -nh / 2.0 + 2.0, 2.5, 0, PI * 2.0);
        ctx.fill();
        ctx.restore();
    }
    // 1 index card at bottom w/ handwriting.
    let (cx, cy) = (w / 2.0, h - 30.0);
    let (cw, ch) = (100.0, 20.0);
    ctx.save();
    ctx.translate(cx, cy);
    ctx.rotate((rnd() - 0.5) * 0.15);
    ctx.set_fill_style("#f0e8d0");
    ctx.fill_rect(-cw / 2.0, -ch / 2.0, cw, ch);
    // Ruled lines.
    ctx.set_stroke_style("rgba(120,140,180,0.4)");
    ctx.set_line_width(0.5);
    ctx.begin_path();
    ctx.move_to(-cw / 2.0 + 3.0, 0);
    ctx.line_to(cw / 2.0 - 3.0, 0);
    ctx.stroke();
    // Wavy handwriting.
    ctx.set_stroke_style("rgba(30,20,10,0.75)");
    ctx.set_line_width(0.9);
    let mut cx2 = -cw / 2.0 + 5.0;
    ctx.begin_path();
    ctx.move_to(cx2, -2);
    while cx2 < cw / 2.0 - 10.0 {
        let seg = 4.0 + rnd() * 5.0;
        let nx = cx2 + seg;
        let dy = (rnd() - 0.5) * 3.0;
        ctx.quadratic_curve_to(cx2 + seg / 2.0, dy * 1.3, nx, dy);
        cx2 = nx;
    }
    ctx.stroke();
    ctx.set_fill_style("#a02040");
    ctx.begin_path();
    ctx.arc(0, -ch / 2.0 + 2.0, 2, 0, PI * 2.0);
    ctx.fill();
    ctx.restore();
}));
