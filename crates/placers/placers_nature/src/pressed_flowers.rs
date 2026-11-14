//! `place_pressed_flowers`
use game::prelude::*;
use placers_kit::place::face_wall;
use placers_kit::place::pick;
use placers_kit::place::wall_tiles;

pub(crate) fn place_pressed_flowers(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    } // 3% chance
    let walls = wall_tiles(g);
    if walls.is_empty() {
        return;
    }
    let (x, z, dx, dz) = pick(&walls);
    let frame_mat = lambert!({ color: 0xc0a0a0 });
    let glass_mat = basic_mat!({ color: 0xe8f0f8, transparent: true, opacity: 0.25 });
    let paper_mat = lambert!({ map: pressed_flower_texture() });
    let flower = group();
    let mount_y = 1.60;
    // Frame.
    let frame = mesh(pg("PressedFlowers__frame", box_geo!(0.18, 0.24, 0.018)), &frame_mat);
    frame.position().y = mount_y;
    flower.add(&frame);
    // Paper backing.
    let paper = mesh(plane_geo!(0.15, 0.21), &paper_mat);
    // On the mount at the bottom of the shadow box's well (front z 0.0015).
    paper.position().set(0, mount_y, 0.002);
    flower.add(&paper);
    // Glass front (thin translucent): inside the box, just behind the lip
    // of the sight edge (0.0062), not 4 mm in front of the frame.
    let glass = mesh(plane_geo!(0.15, 0.21), &glass_mat);
    glass.position().set(0, mount_y, 0.0058);
    flower.add(&glass);
    flower.rotation().z = (rnd() - 0.5) * 0.04;
    flower.rotation().y = face_wall(dx, dz);
    flower.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - 0.015), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - 0.015));
    g.world.add(&flower);
}

// Framed pressed flower: 3% chance per level. Small shadow-box w/ pressed
// flower silhouette.
game::lazy_static_tex!(
    pressed_flower_texture,
    canvas_tex(96, 128, |ctx, w, h| {
        // Off-white paper.
        ctx.set_fill_style("#f0e8d0");
        ctx.fill_rect(0, 0, w, h);
        let (cx, cy) = (w / 2.0, h / 2.0);
        // Stem line.
        ctx.set_stroke_style("#405020");
        ctx.set_line_width(2);
        ctx.begin_path();
        ctx.move_to(cx, h - 10.0);
        ctx.line_to(cx + 2.0, cy);
        ctx.stroke();
        // 2 leaves along stem.
        ctx.set_fill_style("#506030");
        for i in 0..2 {
            ctx.begin_path();
            ctx.ellipse(
                cx + (if i == 0 { -8.0 } else { 10.0 }),
                h * 0.65 + i as f64 * 12.0,
                8,
                3,
                if i == 0 { -0.4 } else { 0.4 },
                0,
                PI * 2.0,
            );
            ctx.fill();
        }
        // Flower head — 6 pressed-flat petals.
        let petal_colors = ["#c04060", "#d060c0", "#f0c020", "#c08040", "#c060c0"];
        let petal_color = petal_colors[(rnd() * petal_colors.len() as f64).floor() as usize];
        ctx.set_fill_style(format!("{petal_color}d0")); // alpha for pressed look
        for p in 0..6 {
            let a = (p as f64 / 6.0) * PI * 2.0;
            ctx.begin_path();
            ctx.ellipse(cx + a.dcos() * 12.0, cy + a.dsin() * 12.0, 10, 6, a, 0, PI * 2.0);
            ctx.fill();
        }
        // Center dot.
        ctx.set_fill_style("#d0a020");
        ctx.begin_path();
        ctx.arc(cx, cy, 5, 0, PI * 2.0);
        ctx.fill();
    })
);
