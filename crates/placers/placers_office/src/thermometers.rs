//! `place_thermometers`
use game::prelude::*;
use placers_kit::place::face_wall;
use placers_kit::place::pick;
use placers_kit::place::wall_tiles;

pub(crate) fn place_thermometers(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let walls = wall_tiles(g);
    if walls.is_empty() {
        return;
    }
    let (x, z, dx, dz) = pick(&walls);
    let back_mat = lambert!({ color: 0x604020 });
    let face_mat = lambert!({ map: therm_texture() });
    let bulb_mat = lambert!({ color: 0xc02020 });
    let therm = group();
    let mount_y = 1.55;
    // Wooden backboard.
    let back = mesh(pg("Thermometers__board", box_geo!(0.030, 0.32, 0.008)), &back_mat);
    back.position().y = mount_y;
    therm.add(&back);
    // Face plate w/ scale texture.
    let face = mesh(plane_geo!(0.024, 0.28), &face_mat);
    face.position().set(0, mount_y, 0.006);
    therm.add(&face);
    // Bulb at bottom (red sphere).
    let bulb = mesh(pg("Thermometers__bulb", sphere_geo!(0.010, 8, 6)), &bulb_mat);
    bulb.position().set(0, mount_y - 0.15, 0.007);
    therm.add(&bulb);
    therm.rotation().y = face_wall(dx, dz);
    therm.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - 0.006), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - 0.006));
    g.world.add(&therm);
}

// Wall thermometer strip: 4% chance per level. Long thin wall device w/
// liquid column + tick marks.
game::lazy_static_tex!(
    therm_texture,
    canvas_tex(28, 200, |ctx, w, h| {
        // Cream background.
        ctx.set_fill_style("#f0e8d0");
        ctx.fill_rect(0, 0, w, h);
        // Central red liquid column (partial fill).
        let fill_h = h * (0.4 + rnd() * 0.3);
        ctx.set_fill_style("#c02020");
        ctx.fill_rect(w * 0.42, h - fill_h, w * 0.16, fill_h);
        // Tick marks along right side.
        ctx.set_stroke_style("#202020");
        ctx.set_line_width(1);
        for i in 0..20 {
            let y = 10.0 + i as f64 * ((h - 20.0) / 19.0);
            let long = i % 5 == 0;
            ctx.begin_path();
            ctx.move_to(w * 0.62, y);
            ctx.line_to(w * (if long { 0.90 } else { 0.78 }), y);
            ctx.stroke();
        }
    })
);
