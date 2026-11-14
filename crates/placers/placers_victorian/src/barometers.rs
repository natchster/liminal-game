//! `place_barometers`
use game::prelude::*;
use placers_kit::place::pick_wall;
use placers_kit::place::wall_mount;

// Wall barometer gauge: 4% chance per level. Needle at random angle.
pub(crate) fn place_barometers(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    }
    let Some((x, z, dx, dz)) = pick_wall(g) else { return };
    let rim_mat = lambert!({ color: 0x503018 });
    let face_mat = lambert!({ map: barometer_texture() });
    let needle_mat = lambert!({ color: 0x201810 });
    let gauge = group();
    let mount_y = 1.85;
    let rim = mesh(pg("Barometers__rim", cyl_geo!(0.11, 0.11, 0.030, 16)), &rim_mat);
    rim.rotation().x = PI / 2.0;
    rim.position().y = mount_y;
    gauge.add(&rim);
    let face = mesh(circle_geo!(0.100, 20), &face_mat);
    face.position().set(0, mount_y, 0.016);
    gauge.add(&face);
    // Needle (fixed at random angle within the 270 degree range).
    let needle = mesh(pg_own("Barometers__needle", box_geo!(0.005, 0.070, 0.003)), &needle_mat);
    needle.position().set(0, mount_y, 0.019);
    needle.geo().translate(0, 0.030, 0);
    needle.rotation().z = -PI * 1.25 + rnd() * PI * 1.5;
    gauge.add(&needle);
    let center = mesh(circle_geo!(0.008, 8), &needle_mat);
    center.position().set(0, mount_y, 0.020);
    gauge.add(&center);
    wall_mount(g, &gauge, x, z, dx, dz, 0.015);
    g.world.add(&gauge);
}

// `barometer_texture`: 270 degree arc of ticks + RAIN / CHANGE / FAIR labels.
game::lazy_static_tex!(
    barometer_texture,
    canvas_tex(160, 160, |ctx, w, h| {
        ctx.set_fill_style("#e0d8c0");
        ctx.fill_rect(0, 0, w, h);
        let (cx, cy) = (w / 2.0, h / 2.0);
        ctx.set_stroke_style("#302818");
        ctx.set_line_width(2);
        for i in 0..=24 {
            let t = i as f64 / 24.0;
            let a = -PI * 1.25 + t * PI * 1.5; // 270 degree arc.
            let r1 = w * 0.42;
            let r2 = if i % 4 == 0 { w * 0.33 } else { w * 0.38 };
            ctx.begin_path();
            ctx.move_to(cx + a.dcos() * r1, cy + a.dsin() * r1);
            ctx.line_to(cx + a.dcos() * r2, cy + a.dsin() * r2);
            ctx.stroke();
        }
        // Labels: RAIN CHANGE FAIR.
        ctx.set_fill_style("#302818");
        ctx.set_font("bold 12px sans-serif");
        ctx.set_text_align("center");
        ctx.fill_text("RAIN", cx - w * 0.30, cy);
        ctx.fill_text("FAIR", cx + w * 0.30, cy);
        ctx.fill_text("CHANGE", cx, cy - h * 0.28);
    })
);
