//! `place_pet_portraits`
use game::prelude::*;
use placers_kit::place::wall_mount;
use placers_kit::place::wall_tiles;

pub(crate) fn place_pet_portraits(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.975 {
        return;
    }
    let walls = wall_tiles(g);
    if walls.is_empty() {
        return;
    }
    let (x, z, dx, dz) = walls[(rnd() * walls.len() as f64).floor() as usize];
    let frame_colors = [0x604020, 0xd0a040, 0x8b5a2b, 0x2a1a10];
    let frame_mat = lambert!({ color: choice(&frame_colors) });
    let portrait_mat = lambert!({ map: pet_portrait_texture() });
    let pp = group();
    let mount_y = 1.60;
    let (frame_w, frame_h) = (0.22, 0.24);
    let frame = mesh(pg("PetPortraits__frame", box_geo!(frame_w, frame_h, 0.014)), &frame_mat);
    frame.position().set(0, mount_y, 0.011);
    pp.add(&frame);
    let portrait = mesh(plane_geo!(frame_w - 0.020, frame_h - 0.020), &portrait_mat);
    portrait.position().set(0, mount_y, 0.019);
    pp.add(&portrait);
    wall_mount(g, &pp, x, z, dx, dz, 0.015);
    pp.rotation().z = (rnd() - 0.5) * 0.03;
    g.world.add(&pp);
}

// Framed pet portrait (wall).
game::lazy_static_tex!(
    pet_portrait_texture,
    canvas_tex(200, 220, |ctx, w, h| {
        // Muted studio-portrait background — gradient.
        let is_dog = rnd() < 0.5;
        let grad = ctx.create_radial_gradient(w / 2.0, h * 0.4, 20, w / 2.0, h * 0.4, w * 0.7);
        grad.add_color_stop(0, "#8a9080");
        grad.add_color_stop(1, "#404838");
        ctx.set_fill_style(&grad);
        ctx.fill_rect(0, 0, w, h);
        // Vignette.
        let vg = ctx.create_radial_gradient(w / 2.0, h * 0.4, 30, w / 2.0, h * 0.4, w * 0.8);
        vg.add_color_stop(0, "rgba(0,0,0,0)");
        vg.add_color_stop(1, "rgba(0,0,0,0.4)");
        ctx.set_fill_style(&vg);
        ctx.fill_rect(0, 0, w, h);
        // Pet silhouette — soft-focus body.
        let (cx, cy) = (w / 2.0, h * 0.55);
        if is_dog {
            // Dog body — larger elongated ellipse w/ head above.
            let body_colors = ["#805030", "#4a3020", "#c0a070", "#e0c890", "#302015"];
            let body_color = choice(&body_colors);
            let c_aa = format!("{body_color}aa");
            let c_00 = format!("{body_color}00");
            // Body ellipse.
            let body_grad = ctx.create_radial_gradient(cx, cy + 20.0, 8, cx, cy + 20.0, 55);
            body_grad.add_color_stop(0, body_color);
            body_grad.add_color_stop(0.7, &c_aa);
            body_grad.add_color_stop(1, &c_00);
            ctx.set_fill_style(&body_grad);
            ctx.begin_path();
            ctx.ellipse(cx, cy + 20.0, 45, 55, 0, 0, PI * 2.0);
            ctx.fill();
            // Head — round + slightly forward.
            let head_grad = ctx.create_radial_gradient(cx, cy - 40.0, 5, cx, cy - 40.0, 32);
            head_grad.add_color_stop(0, body_color);
            head_grad.add_color_stop(0.7, &c_aa);
            head_grad.add_color_stop(1, &c_00);
            ctx.set_fill_style(&head_grad);
            ctx.begin_path();
            ctx.arc(cx, cy - 40.0, 32, 0, PI * 2.0);
            ctx.fill();
            // 2 ears — floppy or perky.
            ctx.set_fill_style(body_color);
            ctx.begin_path();
            ctx.ellipse(cx - 22.0, cy - 55.0, 8, 18, -0.3, 0, PI * 2.0);
            ctx.fill();
            ctx.begin_path();
            ctx.ellipse(cx + 22.0, cy - 55.0, 8, 18, 0.3, 0, PI * 2.0);
            ctx.fill();
            // Snout suggestion — small darker oval below head center.
            ctx.set_fill_style("rgba(0,0,0,0.35)");
            ctx.begin_path();
            ctx.ellipse(cx, cy - 30.0, 10, 6, 0, 0, PI * 2.0);
            ctx.fill();
            // Small black nose.
            ctx.set_fill_style("#101010");
            ctx.begin_path();
            ctx.arc(cx, cy - 32.0, 3, 0, PI * 2.0);
            ctx.fill();
            // 2 dark eye dots.
            ctx.begin_path();
            ctx.arc(cx - 8.0, cy - 44.0, 2, 0, PI * 2.0);
            ctx.arc(cx + 8.0, cy - 44.0, 2, 0, PI * 2.0);
            ctx.fill();
        } else {
            // Cat body — smaller, more compact ellipse.
            let cat_colors = ["#404040", "#a08050", "#f0e0d0", "#8b5a20", "#606060"];
            let cat_color = choice(&cat_colors);
            let c_aa = format!("{cat_color}aa");
            let c_00 = format!("{cat_color}00");
            // Body — squat oval.
            let body_grad = ctx.create_radial_gradient(cx, cy + 25.0, 8, cx, cy + 25.0, 45);
            body_grad.add_color_stop(0, cat_color);
            body_grad.add_color_stop(0.7, &c_aa);
            body_grad.add_color_stop(1, &c_00);
            ctx.set_fill_style(&body_grad);
            ctx.begin_path();
            ctx.ellipse(cx, cy + 25.0, 38, 45, 0, 0, PI * 2.0);
            ctx.fill();
            // Head — smaller round.
            let head_grad = ctx.create_radial_gradient(cx, cy - 25.0, 5, cx, cy - 25.0, 28);
            head_grad.add_color_stop(0, cat_color);
            head_grad.add_color_stop(0.7, &c_aa);
            head_grad.add_color_stop(1, &c_00);
            ctx.set_fill_style(&head_grad);
            ctx.begin_path();
            ctx.arc(cx, cy - 25.0, 28, 0, PI * 2.0);
            ctx.fill();
            // 2 pointed triangle ears.
            ctx.set_fill_style(cat_color);
            ctx.begin_path();
            ctx.move_to(cx - 24.0, cy - 45.0);
            ctx.line_to(cx - 12.0, cy - 55.0);
            ctx.line_to(cx - 8.0, cy - 40.0);
            ctx.close_path();
            ctx.fill();
            ctx.begin_path();
            ctx.move_to(cx + 24.0, cy - 45.0);
            ctx.line_to(cx + 12.0, cy - 55.0);
            ctx.line_to(cx + 8.0, cy - 40.0);
            ctx.close_path();
            ctx.fill();
            // Snout.
            ctx.set_fill_style("rgba(0,0,0,0.3)");
            ctx.begin_path();
            ctx.ellipse(cx, cy - 18.0, 8, 5, 0, 0, PI * 2.0);
            ctx.fill();
            // Pink nose triangle.
            ctx.set_fill_style("#c06070");
            ctx.begin_path();
            ctx.move_to(cx - 3.0, cy - 22.0);
            ctx.line_to(cx + 3.0, cy - 22.0);
            ctx.line_to(cx, cy - 18.0);
            ctx.close_path();
            ctx.fill();
            // 2 vertical-slit yellow eyes.
            ctx.set_fill_style("#f0e040");
            ctx.begin_path();
            ctx.ellipse(cx - 8.0, cy - 30.0, 3, 5, 0, 0, PI * 2.0);
            ctx.ellipse(cx + 8.0, cy - 30.0, 3, 5, 0, 0, PI * 2.0);
            ctx.fill();
            ctx.set_fill_style("#101010");
            ctx.begin_path();
            ctx.ellipse(cx - 8.0, cy - 30.0, 1, 4, 0, 0, PI * 2.0);
            ctx.ellipse(cx + 8.0, cy - 30.0, 1, 4, 0, 0, PI * 2.0);
            ctx.fill();
            // Whiskers (thin lines).
            ctx.set_stroke_style("rgba(240,240,240,0.6)");
            ctx.set_line_width(0.8);
            for i in [-1.0f64, 1.0] {
                ctx.begin_path();
                ctx.move_to(cx + i * 8.0, cy - 20.0);
                let y1 = cy - 22.0 + rnd() * 4.0;
                ctx.line_to(cx + i * 22.0, y1);
                ctx.move_to(cx + i * 8.0, cy - 18.0);
                let y2 = cy - 16.0 + rnd() * 4.0;
                ctx.line_to(cx + i * 22.0, y2);
                ctx.stroke();
            }
        }
        // Optional collar suggestion — colored band around neck.
        if rnd() < 0.7 {
            let collar_colors = ["#a02040", "#2060a0", "#c0a040", "#a04080"];
            ctx.set_stroke_style(choice(&collar_colors));
            ctx.set_line_width(4);
            ctx.begin_path();
            if is_dog {
                ctx.move_to(cx - 22.0, cy - 15.0);
                ctx.quadratic_curve_to(cx, cy - 10.0, cx + 22.0, cy - 15.0);
            } else {
                ctx.move_to(cx - 18.0, cy - 5.0);
                ctx.quadratic_curve_to(cx, cy, cx + 18.0, cy - 5.0);
            }
            ctx.stroke();
            // Small tag on collar.
            ctx.set_fill_style("#c0a060");
            ctx.begin_path();
            ctx.arc(cx, if is_dog { cy - 5.0 } else { cy + 5.0 }, 4, 0, PI * 2.0);
            ctx.fill();
        }
        // Pet name at bottom.
        let dog_names = ["Buddy", "Max", "Rocky", "Charlie", "Duke", "Bailey"];
        let cat_names = ["Mittens", "Whiskers", "Luna", "Oliver", "Simba", "Nala"];
        let names = if is_dog { dog_names } else { cat_names };
        ctx.set_fill_style("#f0e0c0");
        ctx.set_font("italic bold 16px cursive");
        ctx.set_text_align("center");
        ctx.fill_text(choice(&names), w / 2.0, h - 20.0);
        // Years lived below.
        let y1 = 1970 + (rnd() * 20.0).floor() as i64;
        let y2 = y1 + 8 + (rnd() * 8.0).floor() as i64;
        ctx.set_font("italic 11px cursive");
        ctx.fill_text(&format!("{y1} - {y2}"), w / 2.0, h - 6.0);
    })
);
