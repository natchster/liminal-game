//! `place_open_diaries`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Open diary w/ pen on page (floor): 5% chance per level.
pub(crate) fn place_open_diaries(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    }
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = open_tiles[(rnd() * open_tiles.len() as f64).floor() as usize];
    // Diary cover colors.
    let cover_colors = [0x603020, 0x201050, 0x804020, 0x2a4030, 0x505050];
    let cover_mat = lambert!({ color: pick(&cover_colors) });
    let page_mat = lambert!({ map: diary_page_texture() });
    let d = group();
    let (book_w, book_d) = (0.11, 0.15);
    // Both pages laid flat (open flat on floor).
    let page_base = mesh(pg("OpenDiaries__pages", box_geo!(book_w * 2.0, 0.006, book_d)), &page_mat);
    page_base.position().y = 0.008;
    d.add(&page_base);
    // Spine slightly raised.
    let spine = mesh(pg("OpenDiaries__spine", cyl_geo!(0.003, 0.003, book_d, 4)), &cover_mat);
    spine.rotation().x = PI / 2.0;
    spine.position().set(0, 0.013, 0);
    d.add(&spine);
    // Left cover flap (thin plate under left page).
    let left_cover = mesh(pg("OpenDiaries__cover", box_geo!(book_w * 1.05, 0.003, book_d * 1.05)), &cover_mat);
    left_cover.position().set(-book_w * 0.5, 0.003, 0);
    d.add(&left_cover);
    // Right cover flap.
    let right_cover = mesh(pg("OpenDiaries__cover", box_geo!(book_w * 1.05, 0.003, book_d * 1.05)), &cover_mat);
    right_cover.position().set(book_w * 0.5, 0.003, 0);
    d.add(&right_cover);
    // Ribbon bookmark hanging out — thin colored strip.
    let ribbon_colors = [0xa02040, 0x2060a0, 0xc0a040, 0x603030];
    let ribbon_mat = lambert!({ color: pick(&ribbon_colors) });
    let ribbon = mesh(pg("OpenDiaries__ribbon", box_geo!(0.008, 0.002, 0.06)), &ribbon_mat);
    // Draped from the page top over the edge down to the floor.
    ribbon.rotation().x = 0.184;
    ribbon.position().set(0.02, 0.0065, book_d * 0.5 + 0.0245);
    d.add(&ribbon);
    // Pen resting diagonally on page — cylinder + tip cone + button.
    let pen_colors = [0x101010, 0x2060a0, 0x603030, 0xc0a040];
    let pen_mat = lambert!({ color: pick(&pen_colors) });
    let pen = group();
    let pen_body = mesh(pg("OpenDiaries__pen", cyl_geo!(0.004, 0.004, 0.10, 6)), &pen_mat);
    pen_body.rotation().z = PI / 2.0;
    pen.add(&pen_body);
    let pen_tip = mesh(pg("OpenDiaries__tip", cone_geo!(0.003, 0.010, 5)), &pen_mat);
    pen_tip.rotation().z = PI / 2.0; // apex outward (-x), base on the barrel end
    pen_tip.position().x = -0.055;
    pen.add(&pen_tip);
    let pen_button = mesh(pg("OpenDiaries__button", cyl_geo!(0.005, 0.005, 0.006, 5)), &pen_mat);
    pen_button.rotation().z = PI / 2.0;
    pen_button.position().x = 0.053;
    pen.add(&pen_button);
    let pen_clip = mesh(pg("OpenDiaries__clip", box_geo!(0.020, 0.001, 0.004)), &pen_mat);
    pen_clip.position().set(0.020, 0.005, 0);
    pen.add(&pen_clip);
    pen.position().set(book_w * 0.35, 0.016, book_d * 0.15);
    pen.rotation().y = -0.6;
    d.add(&pen);
    d.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    d.rotation().y = rnd() * PI * 2.0;
    g.world.add(&d);
}

// Open diary page spread: ruled pages, a dated entry, and an entry
// interrupted mid-line with a trailing pen tail.
game::lazy_static_tex!(
    diary_page_texture,
    canvas_tex(240, 200, |ctx, w, h| {
        ctx.set_fill_style("#f0e8d0");
        ctx.fill_rect(0, 0, w, h);
        // Ruled blue lines.
        ctx.set_stroke_style("rgba(120,140,180,0.35)");
        ctx.set_line_width(0.7);
        let mut y = 20.0;
        while y < h - 5.0 {
            ctx.begin_path();
            ctx.move_to(6, y);
            ctx.line_to(w - 6.0, y);
            ctx.stroke();
            y += 12.0;
        }
        // Center gutter shadow (spine).
        let shadow = ctx.create_linear_gradient(w / 2.0 - 12.0, 0, w / 2.0 + 12.0, 0);
        shadow.add_color_stop(0, "rgba(0,0,0,0)");
        shadow.add_color_stop(0.5, "rgba(60,40,20,0.35)");
        shadow.add_color_stop(1, "rgba(0,0,0,0)");
        ctx.set_fill_style(&shadow);
        ctx.fill_rect(w / 2.0 - 12.0, 0, 24, h);
        // Date top-right (day entry).
        ctx.set_fill_style("#302010");
        ctx.set_font("italic 12px cursive");
        ctx.set_text_align("right");
        let dates = ["Oct. 14, 1987", "Mar. 22, 1974", "Jul. 4, 1962", "Dec. 18, 1993"];
        ctx.fill_text(dates[(rnd() * dates.len() as f64).floor() as usize], w - 12.0, 16);
        // Left column entry — cursive strokes (the same pattern, reused).
        ctx.set_stroke_style("rgba(30,20,10,0.85)");
        ctx.set_line_width(1.1);
        for row in 0..12 {
            let y = 32.0 + row as f64 * 12.0;
            if y > h - 10.0 {
                break;
            }
            let line_len = 60.0 + rnd() * 40.0;
            ctx.begin_path();
            ctx.move_to(10, y);
            let mut cx = 10.0;
            while cx < 10.0 + line_len && cx < w / 2.0 - 20.0 {
                let seg_len = 5.0 + rnd() * 7.0;
                let nx = cx + seg_len;
                let dy = (rnd() - 0.5) * 3.0;
                ctx.quadratic_curve_to(cx + seg_len / 2.0, y + dy * 1.3, nx, y + dy);
                cx = nx;
            }
            ctx.stroke();
        }
        // Right column — mostly blank (interrupted mid-entry), just short lines.
        for row in 0..3 {
            let y = 32.0 + row as f64 * 12.0;
            let line_len = 50.0 + rnd() * 30.0;
            ctx.begin_path();
            ctx.move_to(w / 2.0 + 14.0, y);
            let mut cx = w / 2.0 + 14.0;
            while cx < w / 2.0 + 14.0 + line_len && cx < w - 10.0 {
                let seg_len = 5.0 + rnd() * 7.0;
                let nx = cx + seg_len;
                let dy = (rnd() - 0.5) * 3.0;
                ctx.quadratic_curve_to(cx + seg_len / 2.0, y + dy * 1.3, nx, y + dy);
                cx = nx;
            }
            ctx.stroke();
        }
        // A partial line — starts, then abruptly ends w/ a trailing pen tail.
        ctx.begin_path();
        let partial_y = 32.0 + 3.0 * 12.0;
        ctx.move_to(w / 2.0 + 14.0, partial_y);
        let mut px = w / 2.0 + 14.0;
        let partial_len = 30.0;
        while px < w / 2.0 + 14.0 + partial_len {
            let seg_len = 5.0 + rnd() * 7.0;
            let nx = px + seg_len;
            let dy = (rnd() - 0.5) * 3.0;
            ctx.quadratic_curve_to(px + seg_len / 2.0, partial_y + dy * 1.3, nx, partial_y + dy);
            px = nx;
        }
        // Trailing pen-drag tail — long low-angle line.
        ctx.quadratic_curve_to(px + 30.0, partial_y + 10.0, px + 60.0, partial_y + 25.0);
        ctx.stroke();
        // Small ink blob at end of trailing tail.
        ctx.set_fill_style("rgba(30,20,10,0.85)");
        ctx.begin_path();
        ctx.arc(px + 60.0, partial_y + 25.0, 2.5, 0, PI * 2.0);
        ctx.fill();
    })
);
