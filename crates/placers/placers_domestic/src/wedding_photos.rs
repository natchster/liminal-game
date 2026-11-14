//! `place_wedding_photos`
use game::prelude::*;
use placers_kit::place::face_wall;
use placers_kit::place::pick;
use placers_kit::place::wall_pos;
use placers_kit::place::wall_tiles;

pub(crate) fn place_wedding_photos(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.975 {
        return;
    }
    let wt = wall_tiles(g);
    if wt.is_empty() {
        return;
    }
    let (x, z, dx, dz) = pick(&wt);
    let frame_colors: [u32; 4] = [0xd0a040, 0x604020, 0x808080, 0xf0e0c0];
    let frame_mat = lambert!({ color: pick(&frame_colors) });
    let photo_mat = lambert!({ map: wedding_photo_texture() });
    let wp = group();
    let mount_y = 1.62;
    let (frame_w, frame_h) = (0.20, 0.24);
    let frame = mesh(pg("WeddingPhotos__frame", box_geo!(frame_w, frame_h, 0.016)), &frame_mat);
    frame.position().set(0, mount_y, 0.012);
    wp.add(&frame);
    let photo = mesh(plane_geo!(frame_w - 0.020, frame_h - 0.020), &photo_mat);
    photo.position().set(0, mount_y, 0.021);
    wp.add(&photo);
    wp.rotation().y = face_wall(dx, dz);
    let (px, pz) = wall_pos(g, x, z, dx, dz, 0.015);
    wp.position().set(px, 0, pz);
    wp.rotation().z = (rnd() - 0.5) * 0.04;
    g.world.add(&wp);
}

// Framed black-and-white wedding photo (wall):
// 2.5% chance per level. Vintage formal wedding portrait
// w/ soft blurred figures.
game::lazy_static_tex!(
    wedding_photo_texture,
    canvas_tex(200, 240, |ctx, w, h| {
        // B&W gradient background — light gray on top, darker below.
        let grad = ctx.create_linear_gradient(0, 0, 0, h);
        grad.add_color_stop(0, "#e0e0e0");
        grad.add_color_stop(1, "#909090");
        ctx.set_fill_style(&grad);
        ctx.fill_rect(0, 0, w, h);
        // Radial vignette darkening corners.
        let rg = ctx.create_radial_gradient(w / 2.0, h * 0.4, 20, w / 2.0, h * 0.4, w * 0.8);
        rg.add_color_stop(0, "rgba(0,0,0,0)");
        rg.add_color_stop(1, "rgba(0,0,0,0.4)");
        ctx.set_fill_style(&rg);
        ctx.fill_rect(0, 0, w, h);
        // Fine film grain.
        ctx.set_fill_style("rgba(0,0,0,0.15)");
        for _ in 0..300 {
            ctx.begin_path();
            let (ax, ay) = (rnd() * w, rnd() * h);
            ctx.arc(ax, ay, 1, 0, PI * 2.0);
            ctx.fill();
        }
        // 2 formal figures center — bride (left) + groom (right).
        // Bride: wider silhouette with veil trailing down.
        let (bride_x, bride_y) = (w * 0.36, h * 0.55);
        // Veil area (large soft aura above and around bride's head/shoulders).
        let veil_grad = ctx.create_radial_gradient(bride_x, bride_y - 50.0, 5, bride_x, bride_y - 50.0, 60);
        veil_grad.add_color_stop(0, "rgba(230,230,230,0.7)");
        veil_grad.add_color_stop(1, "rgba(230,230,230,0)");
        ctx.set_fill_style(&veil_grad);
        ctx.begin_path();
        ctx.arc(bride_x, bride_y - 50.0, 60, 0, PI * 2.0);
        ctx.fill();
        // Bride head.
        let bride_head = ctx.create_radial_gradient(bride_x, bride_y - 55.0, 3, bride_x, bride_y - 55.0, 22);
        bride_head.add_color_stop(0, "rgba(60,55,50,0.9)");
        bride_head.add_color_stop(0.55, "rgba(90,80,70,0.5)");
        bride_head.add_color_stop(1, "rgba(90,80,70,0)");
        ctx.set_fill_style(&bride_head);
        ctx.begin_path();
        ctx.arc(bride_x, bride_y - 55.0, 22, 0, PI * 2.0);
        ctx.fill();
        // Bride dress (wide flowing white).
        let bride_body = ctx.create_radial_gradient(bride_x, bride_y + 25.0, 8, bride_x, bride_y + 25.0, 60);
        bride_body.add_color_stop(0, "rgba(210,210,210,0.85)");
        bride_body.add_color_stop(0.55, "rgba(180,180,180,0.5)");
        bride_body.add_color_stop(1, "rgba(180,180,180,0)");
        ctx.set_fill_style(&bride_body);
        ctx.begin_path();
        ctx.ellipse(bride_x, bride_y + 25.0, 44, 65, 0, 0, PI * 2.0);
        ctx.fill();
        // Groom (right) — dark suit.
        let (groom_x, groom_y) = (w * 0.62, h * 0.55);
        // Groom head.
        let groom_head = ctx.create_radial_gradient(groom_x, groom_y - 50.0, 3, groom_x, groom_y - 50.0, 20);
        groom_head.add_color_stop(0, "rgba(60,50,45,0.9)");
        groom_head.add_color_stop(0.6, "rgba(80,65,55,0.5)");
        groom_head.add_color_stop(1, "rgba(80,65,55,0)");
        ctx.set_fill_style(&groom_head);
        ctx.begin_path();
        ctx.arc(groom_x, groom_y - 50.0, 20, 0, PI * 2.0);
        ctx.fill();
        // Groom suit (dark suit — narrower).
        let groom_body = ctx.create_radial_gradient(groom_x, groom_y + 15.0, 5, groom_x, groom_y + 15.0, 45);
        groom_body.add_color_stop(0, "rgba(30,25,25,0.9)");
        groom_body.add_color_stop(0.6, "rgba(50,45,40,0.55)");
        groom_body.add_color_stop(1, "rgba(50,45,40,0)");
        ctx.set_fill_style(&groom_body);
        ctx.begin_path();
        ctx.ellipse(groom_x, groom_y + 15.0, 26, 55, 0, 0, PI * 2.0);
        ctx.fill();
        // Bouquet — subtle lighter blob at bride's waist.
        let bouquet = ctx.create_radial_gradient(bride_x + 10.0, bride_y + 5.0, 3, bride_x + 10.0, bride_y + 5.0, 14);
        bouquet.add_color_stop(0, "rgba(200,200,200,0.7)");
        bouquet.add_color_stop(1, "rgba(200,200,200,0)");
        ctx.set_fill_style(&bouquet);
        ctx.begin_path();
        ctx.arc(bride_x + 10.0, bride_y + 5.0, 14, 0, PI * 2.0);
        ctx.fill();
        // White border margins (matte).
        ctx.set_fill_style("#f0ece0");
        ctx.fill_rect(0, 0, w, 12);
        ctx.fill_rect(0, h - 28.0, w, 28);
        ctx.fill_rect(0, 0, 12, h);
        ctx.fill_rect(w - 12.0, 0, 12, h);
        // Handwritten caption bottom — date/dedication.
        let captions = ["June 12, 1958", "Our Wedding Day, '62", "Charles & Marie, 1971", "Forever, Sept. '49"];
        let cap = captions[(rnd() * captions.len() as f64).floor() as usize];
        ctx.set_fill_style("#402020");
        ctx.set_font("italic 12px cursive");
        ctx.set_text_align("center");
        ctx.fill_text(cap, w / 2.0, h - 10.0);
        // Small age crease at one corner.
        ctx.set_stroke_style("rgba(60,50,40,0.15)");
        ctx.set_line_width(1);
        ctx.begin_path();
        ctx.move_to(w * 0.85, h * 0.05);
        ctx.line_to(w, h * 0.15);
        ctx.stroke();
    })
);
