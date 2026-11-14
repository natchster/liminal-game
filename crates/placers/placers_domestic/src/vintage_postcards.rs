//! `place_vintage_postcards`
use game::prelude::*;
use placers_kit::place::pick;
use placers_kit::place::wall_tiles;

// Framed vintage postcard (wall): 3% chance per level.
pub(crate) fn place_vintage_postcards(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    }
    let wall_tiles = wall_tiles(g);
    if wall_tiles.is_empty() {
        return;
    }
    let (x, z, dx, dz) = wall_tiles[(rnd() * wall_tiles.len() as f64).floor() as usize];
    let frame_colors = [0x8b5a2b, 0x604020, 0xd0a040, 0x505050];
    let frame_mat = lambert!({ color: pick(&frame_colors) });
    let card_mat = lambert!({ map: postcard_texture() });
    let vp = group();
    let mount_y = 1.55;
    let (frame_w, frame_h) = (0.24, 0.17);
    let frame = mesh(pg("VintagePostcards__frame", box_geo!(frame_w, frame_h, 0.012)), &frame_mat);
    frame.position().set(0, mount_y, 0.010);
    vp.add(&frame);
    let card = mesh(plane_geo!(frame_w - 0.018, frame_h - 0.018), &card_mat);
    card.position().set(0, mount_y, 0.017);
    vp.add(&card);
    vp.rotation().y = (-(dx as f64)).atan2(-(dz as f64));
    vp.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - 0.015), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - 0.015));
    vp.rotation().z = (rnd() - 0.5) * 0.04;
    g.world.add(&vp);
}

// Framed vintage postcard: beach / mountain / cityscape front + stamp.
game::lazy_static_tex!(
    postcard_texture,
    canvas_tex(240, 160, |ctx, w, h| {
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
            let bldgs: [(f64, f64, f64); 5] =
                [(20.0, 30.0, h * 0.45), (60.0, 25.0, h * 0.55), (95.0, 45.0, h * 0.35), (150.0, 20.0, h * 0.65), (180.0, 40.0, h * 0.42)];
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
    })
);
