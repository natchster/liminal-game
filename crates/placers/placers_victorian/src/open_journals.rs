//! `place_open_journals`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Small open journal: 6% chance per level, optional quill.
pub(crate) fn place_open_journals(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.94 {
        return;
    } // 6% chance
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = pick(&open_tiles);
    let cover_mat = lambert!({ color: 0x603820 });
    let page_mat = lambert!({ map: journal_texture(), side: DoubleSide });
    let journal = group();
    let (book_w, book_h) = (0.20, 0.14);
    // Cover under pages.
    let cover = mesh(pg("OpenJournals__cover", box_geo!(book_w + 0.008, 0.005, book_h + 0.008)), &cover_mat);
    cover.position().y = 0.0025;
    journal.add(&cover);
    // Pages plane.
    let pages = mesh(plane_geo!(book_w, book_h), &page_mat);
    pages.rotation().x = -PI / 2.0;
    pages.position().y = 0.0055;
    journal.add(&pages);
    // Small quill on top of journal (60% chance).
    if rnd() < 0.60 {
        let quill_mat = lambert!({ color: 0xf0ece0, side: DoubleSide });
        let shaft_mat = lambert!({ color: 0x201a10 });
        // Lies flat on the pages along (cos 30, 0, sin 30), nib (-Y) end first.
        let shaft = mesh(pg("OpenJournals__quill", cyl_geo!(0.001, 0.001, 0.16, 4)), &shaft_mat);
        shaft.rotation().set(0, -PI / 6.0, -PI / 2.0);
        shaft.position().set(0.02, 0.0065, -0.01);
        journal.add(&shaft);
        // Feather plane.
        let feather = mesh(plane_geo!(0.014, 0.08), &quill_mat);
        feather.rotation().x = -PI / 2.0;
        feather.rotation().z = PI / 3.0;
        feather.position().set(0.02 + 0.04 * (PI / 6.0).dcos(), 0.006, -0.01 + 0.04 * (PI / 6.0).dsin());
        journal.add(&feather);
    }
    journal.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    journal.rotation().y = rnd() * PI * 2.0;
    g.world.add(&journal);
    // Track for paper rustle audio (shared with music sheets).
    let jp = journal.pos();
    g.pp.paper_positions.push(v3(jp.x, 0.01, jp.z));
}

// Open journal pages: simulated handwriting on aged paper.
game::lazy_static_tex!(
    journal_texture,
    canvas_tex_opt(384, 256, false, |ctx, w, h| {
        ctx.set_fill_style("#e8dcc0");
        ctx.fill_rect(0, 0, w, h);
        // Slight yellowing at edges.
        let grad = ctx.create_radial_gradient(w / 2.0, h / 2.0, 100, w / 2.0, h / 2.0, 250);
        grad.add_color_stop(0, "rgba(232,220,192,0)");
        grad.add_color_stop(1, "rgba(160,120,80,0.4)");
        ctx.set_fill_style(&grad);
        ctx.fill_rect(0, 0, w, h);
        // Center spine crease line.
        ctx.set_stroke_style("rgba(80,50,30,0.5)");
        ctx.set_line_width(1);
        ctx.begin_path();
        ctx.move_to(w / 2.0, 20);
        ctx.line_to(w / 2.0, h - 20.0);
        ctx.stroke();
        // Simulated handwriting: wavy lines of "words".
        ctx.set_stroke_style("rgba(40,20,10,0.7)");
        ctx.set_line_width(2);
        for side in 0..2 {
            let start_x = if side == 0 { 30.0 } else { w / 2.0 + 20.0 };
            let end_x = if side == 0 { w / 2.0 - 20.0 } else { w - 30.0 };
            for line in 0..12 {
                let y = 30.0 + line as f64 * 18.0;
                let mut x = start_x;
                while x < end_x {
                    let word_w = 20.0 + rnd() * 30.0;
                    ctx.begin_path();
                    ctx.move_to(x, y);
                    // Squiggly word line.
                    ctx.bezier_curve_to(x + word_w / 3.0, y - 3.0, x + word_w * 2.0 / 3.0, y + 3.0, x + word_w, y);
                    ctx.stroke();
                    x += word_w + 8.0;
                }
            }
        }
    })
);
