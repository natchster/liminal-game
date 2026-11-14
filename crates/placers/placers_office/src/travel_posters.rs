//! `place_travel_posters`
use game::prelude::*;
use placers_kit::place::wall_mount;
use placers_kit::place::wall_tiles;

pub(crate) fn place_travel_posters(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.975 {
        return;
    }
    let walls = wall_tiles(g);
    if walls.is_empty() {
        return;
    }
    let (x, z, dx, dz) = walls[(rnd() * walls.len() as f64).floor() as usize];
    let frame_colors = [0x604020, 0x101010, 0xc0a060, 0x8b5a2b];
    let frame_mat = lambert!({ color: choice(&frame_colors) });
    let poster_mat = lambert!({ map: travel_poster_texture() });
    let tp = group();
    let mount_y = 1.60;
    let (frame_w, frame_h) = (0.20, 0.28);
    let frame = mesh(pg("TravelPosters__frame", box_geo!(frame_w, frame_h, 0.012)), &frame_mat);
    frame.position().set(0, mount_y, 0.010);
    tp.add(&frame);
    let poster = mesh(plane_geo!(frame_w - 0.014, frame_h - 0.014), &poster_mat);
    poster.position().set(0, mount_y, 0.017);
    tp.add(&poster);
    wall_mount(g, &tp, x, z, dx, dz, 0.015);
    tp.rotation().z = (rnd() - 0.5) * 0.03;
    g.world.add(&tp);
}

// Framed vintage travel poster (wall).
game::lazy_static_tex!(
    travel_poster_texture,
    canvas_tex(180, 260, |ctx, w, h| {
        // Base color varies per destination.
        struct V {
            bg: &'static str,
            accent: &'static str,
            name: &'static str,
            tag: &'static str,
            scene: &'static str,
        }
        let variants = [
            V { bg: "#3080b0", accent: "#f0c040", name: "PARIS", tag: "City of Light", scene: "eiffel" },
            V { bg: "#e0a060", accent: "#a02040", name: "MOROCCO", tag: "Ancient Sands", scene: "desert" },
            V { bg: "#4090b0", accent: "#f0e0c0", name: "AMALFI", tag: "Coastal Beauty", scene: "sea" },
            V { bg: "#a04030", accent: "#e0d090", name: "GRAND CANYON", tag: "Timeless Wonder", scene: "canyon" },
            V { bg: "#204080", accent: "#f0e040", name: "NEW YORK", tag: "The Big Apple", scene: "skyline" },
            V { bg: "#606030", accent: "#c04030", name: "KYOTO", tag: "Cherry Blossoms", scene: "blossom" },
        ];
        let v = &variants[(rnd() * variants.len() as f64).floor() as usize];
        // Solid color background.
        ctx.set_fill_style(v.bg);
        ctx.fill_rect(0, 0, w, h);
        // Scene illustration in middle third.
        let mid_y = h * 0.35;
        let scene_h = h * 0.40;
        if v.scene == "eiffel" {
            // Eiffel tower silhouette.
            ctx.set_fill_style("#101020");
            ctx.begin_path();
            ctx.move_to(w / 2.0, mid_y);
            ctx.line_to(w / 2.0 - 30.0, mid_y + scene_h);
            ctx.line_to(w / 2.0 - 25.0, mid_y + scene_h);
            ctx.line_to(w / 2.0 - 10.0, mid_y + scene_h * 0.7);
            ctx.line_to(w / 2.0 + 10.0, mid_y + scene_h * 0.7);
            ctx.line_to(w / 2.0 + 25.0, mid_y + scene_h);
            ctx.line_to(w / 2.0 + 30.0, mid_y + scene_h);
            ctx.close_path();
            ctx.fill();
            // Cross-brace lines.
            ctx.set_stroke_style("#101020");
            ctx.set_line_width(2);
            for i in 0..4 {
                let y = mid_y + scene_h * (0.2 + i as f64 * 0.20);
                let spread = 10.0 + i as f64 * 5.0;
                ctx.begin_path();
                ctx.move_to(w / 2.0 - spread, y);
                ctx.line_to(w / 2.0 + spread, y);
                ctx.stroke();
            }
        } else if v.scene == "desert" {
            // Sand dunes + palm.
            ctx.set_fill_style("#d0a040");
            ctx.begin_path();
            ctx.move_to(0, mid_y + scene_h);
            ctx.line_to(0, mid_y + scene_h * 0.65);
            ctx.quadratic_curve_to(w * 0.3, mid_y + scene_h * 0.55, w * 0.55, mid_y + scene_h * 0.75);
            ctx.quadratic_curve_to(w * 0.8, mid_y + scene_h * 0.90, w, mid_y + scene_h * 0.7);
            ctx.line_to(w, mid_y + scene_h);
            ctx.close_path();
            ctx.fill();
            // Simple pyramid silhouettes.
            ctx.set_fill_style("#803020");
            ctx.begin_path();
            ctx.move_to(w * 0.35, mid_y + scene_h * 0.75);
            ctx.line_to(w * 0.45, mid_y + scene_h * 0.35);
            ctx.line_to(w * 0.55, mid_y + scene_h * 0.75);
            ctx.close_path();
            ctx.fill();
            ctx.begin_path();
            ctx.move_to(w * 0.55, mid_y + scene_h * 0.80);
            ctx.line_to(w * 0.62, mid_y + scene_h * 0.50);
            ctx.line_to(w * 0.68, mid_y + scene_h * 0.80);
            ctx.close_path();
            ctx.fill();
            // Sun.
            ctx.set_fill_style("#f0e040");
            ctx.begin_path();
            ctx.arc(w * 0.75, mid_y + scene_h * 0.25, 20, 0, PI * 2.0);
            ctx.fill();
        } else if v.scene == "sea" {
            // Sea + boat + cliff-side houses.
            ctx.set_fill_style("#204060");
            ctx.fill_rect(0, mid_y + scene_h * 0.5, w, scene_h * 0.5);
            // Wave lines.
            ctx.set_stroke_style("#f0e0c0");
            ctx.set_line_width(1);
            for i in 0..6 {
                let y = mid_y + scene_h * 0.55 + i as f64 * 6.0;
                ctx.begin_path();
                ctx.move_to(0, y);
                let mut x = 0.0;
                while x < w {
                    ctx.line_to(x + 5.0, y - 2.0);
                    ctx.line_to(x + 10.0, y);
                    x += 10.0;
                }
                ctx.stroke();
            }
            // Cliff-side houses (stacked white cubes on left).
            ctx.set_fill_style("#f0e0c0");
            for i in 0..4 {
                let fi = i as f64;
                ctx.fill_rect(4.0 + fi * 10.0, mid_y + scene_h * (0.20 + fi * 0.10), 24, 20);
                // Small window.
                ctx.set_fill_style("#4090b0");
                ctx.fill_rect(12.0 + fi * 10.0, mid_y + scene_h * (0.20 + fi * 0.10) + 5.0, 6, 8);
                ctx.set_fill_style("#f0e0c0");
            }
            // Small sailboat.
            ctx.set_fill_style("#a02040");
            ctx.begin_path();
            ctx.move_to(w * 0.7, mid_y + scene_h * 0.75);
            ctx.line_to(w * 0.7, mid_y + scene_h * 0.50);
            ctx.line_to(w * 0.85, mid_y + scene_h * 0.75);
            ctx.close_path();
            ctx.fill();
            ctx.set_fill_style("#101020");
            ctx.fill_rect(w * 0.68, mid_y + scene_h * 0.75, 20, 5);
        } else if v.scene == "canyon" {
            // Layered canyon walls.
            ctx.set_fill_style("#803020");
            ctx.begin_path();
            ctx.move_to(0, mid_y + scene_h * 0.4);
            ctx.line_to(w * 0.2, mid_y + scene_h * 0.55);
            ctx.line_to(w * 0.35, mid_y + scene_h * 0.42);
            ctx.line_to(w * 0.55, mid_y + scene_h * 0.60);
            ctx.line_to(w * 0.75, mid_y + scene_h * 0.48);
            ctx.line_to(w, mid_y + scene_h * 0.55);
            ctx.line_to(w, mid_y + scene_h);
            ctx.line_to(0, mid_y + scene_h);
            ctx.close_path();
            ctx.fill();
            ctx.set_fill_style("#602010");
            ctx.begin_path();
            ctx.move_to(0, mid_y + scene_h * 0.7);
            ctx.line_to(w * 0.3, mid_y + scene_h * 0.85);
            ctx.line_to(w * 0.6, mid_y + scene_h * 0.7);
            ctx.line_to(w, mid_y + scene_h * 0.85);
            ctx.line_to(w, mid_y + scene_h);
            ctx.line_to(0, mid_y + scene_h);
            ctx.close_path();
            ctx.fill();
            // Sun rays.
            ctx.set_stroke_style("#f0e0a0");
            ctx.set_line_width(2);
            for i in 0..5 {
                let ang = (i as f64 / 4.0) * PI - PI / 2.0;
                ctx.begin_path();
                ctx.move_to(w * 0.7, mid_y);
                ctx.line_to(w * 0.7 + ang.dcos() * 60.0, mid_y + ang.dsin() * 60.0);
                ctx.stroke();
            }
        } else if v.scene == "skyline" {
            // Building silhouettes.
            ctx.set_fill_style("#101020");
            struct B {
                x: f64,
                w: f64,
                h: f64,
            }
            let buildings = [
                B { x: 10.0, w: 22.0, h: scene_h * 0.65 },
                B { x: 34.0, w: 18.0, h: scene_h * 0.80 },
                B { x: 54.0, w: 26.0, h: scene_h * 0.55 },
                B { x: 82.0, w: 20.0, h: scene_h * 0.90 },
                B { x: 104.0, w: 24.0, h: scene_h * 0.70 },
                B { x: 130.0, w: 20.0, h: scene_h * 0.85 },
                B { x: 152.0, w: 20.0, h: scene_h * 0.60 },
            ];
            for b in &buildings {
                ctx.fill_rect(b.x, mid_y + scene_h - b.h, b.w, b.h);
            }
            // Yellow window dots.
            ctx.set_fill_style("#f0e040");
            for _ in 0..40 {
                let b = &buildings[(rnd() * buildings.len() as f64).floor() as usize];
                let wy = mid_y + scene_h - b.h + 5.0 + rnd() * (b.h - 10.0);
                let wxp = b.x + 3.0 + rnd() * (b.w - 6.0);
                ctx.fill_rect(wxp, wy, 2, 3);
            }
        } else if v.scene == "blossom" {
            // Cherry blossom scene. Tree trunk.
            ctx.set_fill_style("#402010");
            ctx.fill_rect(w / 2.0 - 5.0, mid_y + scene_h * 0.65, 10, scene_h * 0.35);
            // 3 curved branches.
            ctx.set_stroke_style("#402010");
            ctx.set_line_width(4);
            for angle_d in [-1.0f64, 0.5, 1.0] {
                ctx.begin_path();
                ctx.move_to(w / 2.0, mid_y + scene_h * 0.65);
                ctx.quadratic_curve_to(w / 2.0 + angle_d * 30.0, mid_y + scene_h * 0.4, w / 2.0 + angle_d * 50.0, mid_y + scene_h * 0.3);
                ctx.stroke();
            }
            // Blossom clouds.
            ctx.set_fill_style("#f0c0d0");
            for (bx, by) in
                [(w / 2.0 - 40.0, mid_y + scene_h * 0.25), (w / 2.0 + 40.0, mid_y + scene_h * 0.25), (w / 2.0, mid_y + scene_h * 0.1)]
            {
                ctx.begin_path();
                ctx.arc(bx, by, 25, 0, PI * 2.0);
                ctx.fill();
            }
            // Tiny blossom dots.
            ctx.set_fill_style("#f0d0e0");
            for _ in 0..20 {
                let bx = w * 0.2 + rnd() * w * 0.6;
                let by = mid_y + rnd() * scene_h * 0.5;
                ctx.begin_path();
                ctx.arc(bx, by, 2, 0, PI * 2.0);
                ctx.fill();
            }
        }
        // Top banner — destination name in large accent.
        ctx.set_fill_style(v.accent);
        ctx.set_font("bold 26px serif");
        ctx.set_text_align("center");
        ctx.fill_text(v.name, w / 2.0, 40);
        // Tagline below name.
        ctx.set_font("italic 11px serif");
        ctx.set_fill_style(v.accent);
        ctx.fill_text(v.tag, w / 2.0, 60);
        // Bottom "Fly Now!" or similar tagline.
        ctx.set_font("bold 12px serif");
        ctx.set_fill_style(v.accent);
        let bottoms = ["VISIT TODAY", "FLY NOW", "BOOK YOUR TRIP", "DISCOVER", "YOUR JOURNEY AWAITS"];
        ctx.fill_text(choice(&bottoms), w / 2.0, h - 20.0);
        // Small airline tag bottom-right.
        ctx.set_font("bold 9px serif");
        ctx.set_text_align("right");
        let airlines = ["TWA", "PAN AM", "TAA", "BOAC"];
        ctx.fill_text(choice(&airlines), w - 8.0, h - 6.0);
    })
);
