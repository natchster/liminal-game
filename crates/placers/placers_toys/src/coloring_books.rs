//! `place_coloring_books`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick_hex;
use placers_kit::place::scatter_on_floor;

// Open coloring book: 6% chance per level. Cover + open spread + crayon.
pub(crate) fn place_coloring_books(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let cover_mat = lambert!({ color: 0xd82060 });
    let page_mat = lambert!({ map: coloring_texture(), side: DoubleSide });
    let book = group();
    // Cover under.
    let cover = mesh(pg("ColoringBooks__cover", box_geo!(0.22, 0.003, 0.14)), &cover_mat);
    cover.position().y = 0.002;
    book.add(&cover);
    // Pages.
    let pages = mesh(plane_geo!(0.20, 0.13), &page_mat);
    pages.rotation().x = -PI / 2.0;
    pages.position().y = 0.005;
    book.add(&pages);
    // Crayon nearby.
    let crayon_colors = [0xd82040, 0x2050d8, 0xd8b040];
    let crayon_mat = lambert!({ color: pick_hex(&crayon_colors) });
    let crayon = mesh(pg("ColoringBooks__crayon", cyl_geo!(0.004, 0.004, 0.05, 6)), &crayon_mat);
    crayon.rotation().z = PI / 2.0;
    crayon.position().set(0.08, 0.004, -0.05);
    crayon.rotation().y = 0.5;
    book.add(&crayon);
    scatter_on_floor(g, &book, x, z, 2.0);
    g.world.add(&book);
}

game::lazy_static_tex!(
    coloring_texture,
    canvas_tex(320, 200, |ctx, w, h| {
        ctx.set_fill_style("#f0e8d0");
        ctx.fill_rect(0, 0, w, h);
        // Center spine crease.
        ctx.set_stroke_style("rgba(80,60,40,0.4)");
        ctx.set_line_width(1);
        ctx.begin_path();
        ctx.move_to(w / 2.0, 10);
        ctx.line_to(w / 2.0, h - 10.0);
        ctx.stroke();
        // Line drawing (house on left side).
        ctx.set_stroke_style("#181008");
        ctx.set_line_width(2);
        // House square.
        ctx.stroke_rect(30, 80, 100, 80);
        // Roof triangle.
        ctx.begin_path();
        ctx.move_to(30, 80);
        ctx.line_to(80, 30);
        ctx.line_to(130, 80);
        ctx.stroke();
        // Door.
        ctx.stroke_rect(60, 120, 30, 40);
        // 2 windows.
        ctx.stroke_rect(35, 90, 20, 20);
        ctx.stroke_rect(105, 90, 20, 20);
        // Kid scribble in bright colors.
        let kid_colors = ["#d82040", "#2050d8", "#d8b040", "#30a030"];
        for i in 0..15 {
            ctx.set_stroke_style(kid_colors[i % kid_colors.len()]);
            ctx.set_line_width(3);
            ctx.begin_path();
            let mut x = 40.0 + rnd() * 100.0;
            let mut y = 90.0 + rnd() * 60.0;
            ctx.move_to(x, y);
            for _ in 0..5 {
                x += (rnd() - 0.5) * 20.0;
                y += (rnd() - 0.5) * 20.0;
                ctx.line_to(x, y);
            }
            ctx.stroke();
        }
        // Right page: line drawing of tree.
        ctx.set_stroke_style("#181008");
        ctx.set_line_width(2);
        ctx.stroke_rect(220, 130, 20, 40); // trunk
        ctx.begin_path();
        ctx.arc(230, 100, 40, 0, PI * 2.0); // canopy
        ctx.stroke();
        // Scribble green on tree.
        ctx.set_stroke_style("#30a030");
        ctx.set_line_width(4);
        for _ in 0..8 {
            ctx.begin_path();
            ctx.move_to(200.0 + rnd() * 60.0, 80.0 + rnd() * 40.0);
            ctx.line_to(200.0 + rnd() * 60.0, 80.0 + rnd() * 40.0);
            ctx.stroke();
        }
    })
);
