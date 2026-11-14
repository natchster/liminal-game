//! `place_embroidery_hoops`
use game::prelude::*;
use placers_kit::place::face_wall;
use placers_kit::place::pick;
use placers_kit::place::wall_pos;
use placers_kit::place::wall_tiles;

pub(crate) fn place_embroidery_hoops(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    }
    let wt = wall_tiles(g);
    if wt.is_empty() {
        return;
    }
    let (x, z, dx, dz) = pick(&wt);
    let wood_colors: [u32; 4] = [0x8b5a2b, 0xa07040, 0x604020, 0xb08050];
    let wood_mat = lambert!({ color: pick(&wood_colors) });
    let fabric_mat = lambert!({ map: embroidery_texture() });
    let hoop = group();
    let mount_y = 1.60;
    // Outer wooden ring — flat torus.
    let outer_ring = mesh(pg("EmbroideryHoops__outer", torus_geo!(0.135, 0.010, 5, 24)), &wood_mat);
    outer_ring.position().set(0, mount_y, 0.014);
    hoop.add(&outer_ring);
    // Inner ring (very close to outer, simulates the two-ring clamp).
    let inner_ring = mesh(pg("EmbroideryHoops__inner", torus_geo!(0.125, 0.005, 4, 22)), &wood_mat);
    inner_ring.position().set(0, mount_y, 0.010);
    hoop.add(&inner_ring);
    // Fabric disc w/ embroidery pattern.
    let fabric = mesh(circle_geo!(0.128, 22), &fabric_mat);
    fabric.position().set(0, mount_y, 0.006);
    hoop.add(&fabric);
    // Small tightening screw at top.
    let screw = mesh(pg("EmbroideryHoops__screw", box_geo!(0.020, 0.014, 0.014)), &wood_mat);
    screw.position().set(0, mount_y + 0.14, 0.014);
    hoop.add(&screw);
    // Tiny hanging string above.
    let string_mat = lambert!({ color: 0x604020 });
    let st = mesh(pg("HangingColanders__cord", cyl_geo!(0.001, 0.001, 0.04, 3)), &string_mat);
    st.position().set(0, mount_y + 0.17, 0.008);
    hoop.add(&st);
    hoop.rotation().y = face_wall(dx, dz);
    let (px, pz) = wall_pos(g, x, z, dx, dz, 0.015);
    hoop.position().set(px, 0, pz);
    // Slight tilt.
    hoop.rotation().z = (rnd() - 0.5) * 0.04;
    g.world.add(&hoop);
}

// Framed embroidery hoop (wall): 4% chance per level.
// Wooden hoop stretching fabric w/ visible embroidered pattern.
game::lazy_static_tex!(
    embroidery_texture,
    canvas_tex(192, 192, |ctx, w, h| {
        // Fabric background — off-white/cream.
        ctx.set_fill_style("#f4ecd8");
        ctx.fill_rect(0, 0, w, h);
        // Faint weave texture — subtle vertical + horizontal light lines.
        ctx.set_stroke_style("rgba(200, 190, 170, 0.4)");
        ctx.set_line_width(1);
        let mut i = 0.0;
        while i < w {
            ctx.begin_path();
            ctx.move_to(i, 0);
            ctx.line_to(i, h);
            ctx.stroke();
            i += 3.0;
        }
        let mut j = 0.0;
        while j < h {
            ctx.begin_path();
            ctx.move_to(0, j);
            ctx.line_to(w, j);
            ctx.stroke();
            j += 3.0;
        }
        // Central embroidered flower — random color palette.
        let petal_palette: [[&str; 3]; 4] = [
            ["#a03060", "#f0c080", "#80a040"],
            ["#4060a0", "#f0d0a0", "#608040"],
            ["#804080", "#e0a0c0", "#508050"],
            ["#a04040", "#c0a060", "#406030"],
        ];
        let p = petal_palette[(rnd() * petal_palette.len() as f64).floor() as usize];
        let (cx, cy) = (w / 2.0, h / 2.0);
        // 6 petals — small ellipses arranged around center.
        ctx.set_fill_style(p[0]);
        for i in 0..6 {
            let ang = (i as f64 / 6.0) * PI * 2.0;
            ctx.save();
            ctx.translate(cx + ang.dcos() * 26.0, cy + ang.dsin() * 26.0);
            ctx.rotate(ang);
            ctx.begin_path();
            ctx.ellipse(0, 0, 18, 10, 0, 0, PI * 2.0);
            ctx.fill();
            ctx.restore();
        }
        // Center disc.
        ctx.set_fill_style(p[1]);
        ctx.begin_path();
        ctx.arc(cx, cy, 12, 0, PI * 2.0);
        ctx.fill();
        // Stem — curved leaf below.
        ctx.set_stroke_style(p[2]);
        ctx.set_line_width(4);
        ctx.begin_path();
        ctx.move_to(cx, cy + 12.0);
        ctx.quadratic_curve_to(cx - 20.0, cy + 60.0, cx - 8.0, cy + 84.0);
        ctx.stroke();
        // Small leaf.
        ctx.set_fill_style(p[2]);
        ctx.begin_path();
        ctx.ellipse(cx - 22.0, cy + 50.0, 12, 6, -0.6, 0, PI * 2.0);
        ctx.fill();
        // Frame border — visible stitching around edge.
        ctx.set_stroke_style("rgba(150, 130, 100, 0.6)");
        ctx.set_line_width(1);
        let mut a = 0.0;
        while a < PI * 2.0 {
            let r0 = w.min(h) * 0.44;
            let r1 = r0 - 4.0;
            ctx.begin_path();
            ctx.move_to(cx + a.dcos() * r0, cy + a.dsin() * r0);
            ctx.line_to(cx + a.dcos() * r1, cy + a.dsin() * r1);
            ctx.stroke();
            a += PI / 24.0;
        }
    })
);
