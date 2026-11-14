//! `place_childrens_books`: Children's book: 10% chance per level.
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

/// Children's book: 10% chance per level.
pub(crate) fn place_childrens_books(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let cover_mat = lambert!({ map: child_book_texture(), side: DoubleSide });
    let pages_mat = lambert!({ color: 0xd8ccb0 });
    let cbook = group();
    // Slightly smaller than the adult book (10 × 14 cm).
    let cover = mesh(box_geo!(0.10, 0.005, 0.14), &cover_mat);
    cover.position().y = 0.020;
    cbook.add(&cover);
    let pages = mesh(pg("ChildrensBooks__pages", box_geo!(0.095, 0.016, 0.135)), &pages_mat);
    pages.position().y = 0.010;
    cbook.add(&pages);
    let back = mesh(box_geo!(0.10, 0.005, 0.14), &cover_mat);
    back.position().y = -0.001;
    cbook.add(&back);
    cbook.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0.003, g.wz(z) + (rnd() - 0.5) * 2.4);
    cbook.rotation().y = rnd() * PI * 2.0;
    cbook.rotation().x = (rnd() - 0.5) * 0.12;
    g.world.add(&cbook);
}

// Children's book texture — colorful cartoon cover simplification
// (bright bg + big geometric shape center).
game::lazy_static_tex!(
    child_book_texture,
    canvas_tex_opt(96, 128, false, |ctx, w, h| {
        let bg_colors = ["#d8b0e0", "#a0d8f0", "#f0d0a0", "#c0e8a0"];
        ctx.set_fill_style(pick(&bg_colors));
        ctx.fill_rect(0, 0, w, h);
        // Large central colorful shape (circle, star, or triangle).
        let shape_colors = ["#d82020", "#2050d8", "#d8b820", "#30a030", "#8020d8"];
        ctx.set_fill_style(pick(&shape_colors));
        let shape = (rnd() * 3.0).floor() as i32;
        let (cx, cy) = (w / 2.0, h * 0.4);
        if shape == 0 {
            // Circle.
            ctx.begin_path();
            ctx.arc(cx, cy, 30, 0, PI * 2.0);
            ctx.fill();
        } else if shape == 1 {
            // Triangle.
            ctx.begin_path();
            ctx.move_to(cx, cy - 30.0);
            ctx.line_to(cx - 30.0, cy + 25.0);
            ctx.line_to(cx + 30.0, cy + 25.0);
            ctx.close_path();
            ctx.fill();
        } else {
            // Star (rough 5-point via 10 vertices).
            ctx.begin_path();
            for i in 0..10 {
                let a = -PI / 2.0 + (i as f64 / 10.0) * PI * 2.0;
                let r = if i % 2 == 0 { 30.0 } else { 14.0 };
                let px = cx + a.dcos() * r;
                let py = cy + a.dsin() * r;
                if i == 0 {
                    ctx.move_to(px, py);
                } else {
                    ctx.line_to(px, py);
                }
            }
            ctx.close_path();
            ctx.fill();
        }
        // Fake title text: 2 dark horizontal bars near bottom.
        ctx.set_fill_style("#3a2c1a");
        ctx.fill_rect(15, h - 30.0, w - 30.0, 4);
        ctx.fill_rect(20, h - 20.0, w - 40.0, 4);
    })
);
