//! `place_family_photos`
use game::prelude::*;
use placers_kit::place::face_wall;
use placers_kit::place::pick;
use placers_kit::place::wall_pos;
use placers_kit::place::wall_tiles;

pub(crate) fn place_family_photos(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    }
    let wt = wall_tiles(g);
    if wt.is_empty() {
        return;
    }
    let (x, z, dx, dz) = pick(&wt);
    let frame_colors: [u32; 4] = [0x604020, 0x8b5a2b, 0x505050, 0xd0a040];
    let frame_mat = lambert!({ color: pick(&frame_colors) });
    let photo_mat = lambert!({ map: family_photo_texture() });
    let fp = group();
    let mount_y = 1.60;
    let (frame_w, frame_h) = (0.24, 0.19);
    let frame = mesh(pg("FamilyPhotos__frame", box_geo!(frame_w, frame_h, 0.014)), &frame_mat);
    frame.position().set(0, mount_y, 0.011);
    fp.add(&frame);
    let photo = mesh(plane_geo!(frame_w - 0.020, frame_h - 0.020), &photo_mat);
    photo.position().set(0, mount_y, 0.019);
    fp.add(&photo);
    fp.rotation().y = face_wall(dx, dz);
    let (px, pz) = wall_pos(g, x, z, dx, dz, 0.015);
    fp.position().set(px, 0, pz);
    fp.rotation().z = (rnd() - 0.5) * 0.04;
    g.world.add(&fp);
}

// Framed family photograph w/ blurred figures (wall):
// 3% chance per level. Sepia-toned photo w/ soft indistinct
// human silhouettes — deeply liminal.
game::lazy_static_tex!(
    family_photo_texture,
    canvas_tex(224, 180, |ctx, w, h| {
        // Sepia background gradient.
        let grad = ctx.create_linear_gradient(0, 0, 0, h);
        grad.add_color_stop(0, "#c8a878");
        grad.add_color_stop(1, "#8a6848");
        ctx.set_fill_style(&grad);
        ctx.fill_rect(0, 0, w, h);
        // Faint radial vignette darkening corners.
        let rg = ctx.create_radial_gradient(w / 2.0, h / 2.0, w * 0.2, w / 2.0, h / 2.0, w * 0.7);
        rg.add_color_stop(0, "rgba(0,0,0,0)");
        rg.add_color_stop(1, "rgba(20,10,0,0.4)");
        ctx.set_fill_style(&rg);
        ctx.fill_rect(0, 0, w, h);
        // Subtle film grain — many tiny dark dots.
        ctx.set_fill_style("rgba(20,10,0,0.15)");
        for _ in 0..240 {
            ctx.begin_path();
            let (ax, ay) = (rnd() * w, rnd() * h);
            ctx.arc(ax, ay, 1, 0, PI * 2.0);
            ctx.fill();
        }
        // Blurred wall/backdrop hint — horizontal darker band mid-height.
        ctx.set_fill_style("rgba(80,50,30,0.25)");
        ctx.fill_rect(0, h * 0.55, w, h * 0.20);
        // 3-4 blurred figures — soft radial gradients w/ elongated form.
        let fig_count = 3 + (rnd() * 2.0).floor() as i32;
        let center_y = h * 0.55;
        for i in 0..fig_count {
            let cx = w * (0.20 + i as f64 / (fig_count as f64 - 0.4) * 0.6);
            let is_child = i == fig_count - 1 && rnd() < 0.6;
            let size = if is_child { 25.0 } else { 40.0 };
            // Head (softer, above shoulders).
            let head_grad = ctx.create_radial_gradient(cx, center_y - size * 0.9, 3, cx, center_y - size * 0.9, size * 0.35);
            head_grad.add_color_stop(0, "rgba(70,45,25,0.85)");
            head_grad.add_color_stop(0.6, "rgba(80,55,35,0.5)");
            head_grad.add_color_stop(1, "rgba(80,55,35,0)");
            ctx.set_fill_style(&head_grad);
            ctx.begin_path();
            ctx.arc(cx, center_y - size * 0.9, size * 0.35, 0, PI * 2.0);
            ctx.fill();
            // Body (larger, tapered — elongated ellipse).
            let body_grad = ctx.create_radial_gradient(cx, center_y - 4.0, 4, cx, center_y - 4.0, size * 0.7);
            body_grad.add_color_stop(0, "rgba(60,35,20,0.85)");
            body_grad.add_color_stop(0.7, "rgba(80,55,35,0.35)");
            body_grad.add_color_stop(1, "rgba(80,55,35,0)");
            ctx.set_fill_style(&body_grad);
            ctx.begin_path();
            ctx.ellipse(cx, center_y - 4.0, size * 0.5, size * 0.85, 0, 0, PI * 2.0);
            ctx.fill();
        }
        // Bottom margin — white/cream border of old photo.
        ctx.set_fill_style("#f0e0c0");
        ctx.fill_rect(0, h - 20.0, w, 20);
        ctx.fill_rect(0, 0, w, 8);
        ctx.fill_rect(0, 0, 8, h);
        ctx.fill_rect(w - 8.0, 0, 8, h);
        // Handwritten caption on bottom margin.
        ctx.set_fill_style("#402010");
        ctx.set_font("11px cursive");
        ctx.set_text_align("center");
        let captions = ["Summer 1976", "The family, 1982", "Home, spring '79", "Us, before."];
        let cap = captions[(rnd() * captions.len() as f64).floor() as usize];
        ctx.fill_text(cap, w / 2.0, h - 6.0);
        // Slight fold/crease diagonal — thin brown line.
        ctx.set_stroke_style("rgba(40,20,0,0.2)");
        ctx.set_line_width(1);
        ctx.begin_path();
        ctx.move_to(w * 0.15, h * 0.10);
        ctx.line_to(w * 0.85, h * 0.90);
        ctx.stroke();
    })
);
