//! `place_framed_recipes`
use game::prelude::*;
use placers_kit::place::face_wall;
use placers_kit::place::pick;
use placers_kit::place::wall_tiles;

pub(crate) fn place_framed_recipes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let wt = wall_tiles(g);
    if wt.is_empty() {
        return;
    }
    let (x, z, dx, dz) = pick(&wt);
    let frame_mat = lambert!({ color: 0x604020 });
    let paper_mat = lambert!({ map: recipe_texture() });
    let rec = group();
    let mount_y = 1.65;
    let frame = mesh(pg("FramedRecipes__frame", box_geo!(0.20, 0.26, 0.014)), &frame_mat);
    frame.position().y = mount_y;
    rec.add(&frame);
    let paper = mesh(plane_geo!(0.16, 0.22), &paper_mat);
    paper.position().set(0, mount_y, 0.002); // in the frame's rebate (floor z 0.001), behind the sight edge
    rec.add(&paper);
    rec.rotation().z = (rnd() - 0.5) * 0.06;
    rec.rotation().y = face_wall(dx, dz);
    rec.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - 0.013), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - 0.013));
    g.world.add(&rec);
}

// Framed hand-drawn recipe: 4% chance per level.
game::lazy_static_tex!(
    recipe_texture,
    canvas_tex(112, 152, |ctx, w, h| {
        ctx.set_fill_style("#f8f0d8");
        ctx.fill_rect(0, 0, w, h);
        ctx.set_stroke_style("#804020");
        ctx.set_line_width(1);
        ctx.stroke_rect(6, 6, w - 12.0, h - 12.0);
        ctx.set_fill_style("#40200a");
        ctx.fill_rect(14, 14, w - 28.0, 6);
        ctx.set_fill_style("#c02020");
        ctx.begin_path();
        ctx.arc(w * 0.5, 32, 4, 0, PI * 2.0);
        ctx.fill();
        ctx.set_fill_style("#40301a");
        for i in 0..8 {
            let y = 46.0 + i as f64 * 11.0;
            let start_x = 14.0 + if i % 2 == 0 { 0.0 } else { 6.0 };
            let w2 = 40.0 + rnd() * (w - 60.0 - start_x);
            ctx.fill_rect(start_x, y, w2, 1.6);
        }
        ctx.set_fill_style("#40200a");
        ctx.fill_rect(w * 0.5, h - 22.0, w * 0.35, 1.8);
    })
);
