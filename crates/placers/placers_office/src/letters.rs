//! `place_letters`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

// Crumpled letter w/ opening greeting (floor): 5% chance per level.
pub(crate) fn place_letters(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    }
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = open_tiles[(rnd() * open_tiles.len() as f64).floor() as usize];
    let paper_mat = lambert!({ map: letter_texture() });
    let l = group();
    // Main sheet.
    let (sheet_w, sheet_d) = (0.14, 0.18);
    let sheet = mesh(pg("Letters__sheet", box_geo!(sheet_w, 0.002, sheet_d)), &paper_mat);
    sheet.position().y = 0.001;
    l.add(&sheet);
    // Reused crumpled-paper wrinkle pattern — 2 tilted plane bulges.
    let fold1 = mesh(pg("Letters__fold", box_geo!(sheet_w * 0.5, 0.003, sheet_d * 0.35)), &paper_mat);
    fold1.position().set(sheet_w * 0.25, 0.019, -sheet_d * 0.25); // lowest corner on the sheet
    fold1.rotation().z = 0.30;
    fold1.rotation().x = 0.15;
    l.add(&fold1);
    let fold2 = mesh(pg("Letters__scrap", box_geo!(sheet_w * 0.35, 0.003, sheet_d * 0.30)), &paper_mat);
    fold2.position().set(-sheet_w * 0.20, 0.011, sheet_d * 0.25);
    fold2.rotation().z = -0.20;
    fold2.rotation().x = -0.10;
    l.add(&fold2);
    // Optional envelope nearby (60%).
    if rnd() < 0.6 {
        let env_colors = [0xf0e8d0, 0xf0d8c0, 0xe8dcc0];
        let env_mat = lambert!({ color: pick(&env_colors) });
        // Torn-open envelope: base + tilted flap + edge tear.
        let env = group();
        let env_base = mesh(pg("Letters__envelope", box_geo!(0.10, 0.002, 0.06)), &env_mat);
        env_base.position().y = 0.001;
        env.add(&env_base);
        // Flap tilted up (opened): hinge edge on the envelope's back edge.
        let flap = mesh(pg("Letters__flap", box_geo!(0.10, 0.002, 0.04)), &env_mat);
        flap.position().set(0, 0.0165, -0.043);
        flap.rotation().x = 0.8;
        env.add(&flap);
        // Small red stamp.
        let stamp_mat = lambert!({ color: 0xa02040 });
        let stamp = mesh(pg("Letters__stamp", box_geo!(0.012, 0.003, 0.015)), &stamp_mat);
        stamp.position().set(0.037, 0.0035, 0.020);
        env.add(&stamp);
        env.position().set(0.16, 0, 0.02);
        env.rotation().y = (rnd() - 0.5) * 0.6;
        l.add(&env);
    }
    l.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    l.rotation().y = rnd() * PI * 2.0;
    g.world.add(&l);
}

// Crumpled letter w/ opening greeting: handwritten letter with
// "Dear X..." visible + signature.
game::lazy_static_tex!(
    letter_texture,
    canvas_tex(180, 220, |ctx, w, h| {
        // Aged paper background.
        ctx.set_fill_style("#f4ecd0");
        ctx.fill_rect(0, 0, w, h);
        // Faint blue-ruled lines.
        ctx.set_stroke_style("rgba(120,140,180,0.28)");
        ctx.set_line_width(0.7);
        let mut y = 24.0;
        while y < h - 10.0 {
            ctx.begin_path();
            ctx.move_to(8, y);
            ctx.line_to(w - 8.0, y);
            ctx.stroke();
            y += 14.0;
        }
        // Red left margin line.
        ctx.set_stroke_style("rgba(200,60,60,0.35)");
        ctx.set_line_width(1);
        ctx.begin_path();
        ctx.move_to(18, 8);
        ctx.line_to(18, h - 8.0);
        ctx.stroke();
        // Date top right.
        let dates = ["October 14, 1987", "March 22, 1974", "July 4, 1962", "December 18, 1993"];
        ctx.set_fill_style("#302010");
        ctx.set_font("italic 11px cursive");
        ctx.set_text_align("right");
        ctx.fill_text(dates[(rnd() * dates.len() as f64).floor() as usize], w - 12.0, 20);
        // Greeting: "Dear X,"
        let names = ["Sarah", "Michael", "Emma", "James", "Rose", "Thomas", "Anna", "David"];
        let greeting = format!("Dear {},", names[(rnd() * names.len() as f64).floor() as usize]);
        ctx.set_text_align("left");
        ctx.set_font("italic 16px cursive");
        ctx.fill_text(&greeting, 24, 44);
        // Body — 6-8 wavy lines of cursive script.
        ctx.set_stroke_style("rgba(50,35,20,0.75)");
        ctx.set_line_width(1.2);
        for i in 0..8 {
            let y = 66.0 + i as f64 * 14.0;
            if y > h - 40.0 {
                break;
            }
            // Random line length (some short as if paragraph endings).
            let line_len = 100.0 + rnd() * 40.0;
            // Wavy cursive simulation — many quadratic segments.
            ctx.begin_path();
            ctx.move_to(24, y);
            let mut cx = 24.0;
            while cx < 24.0 + line_len {
                let seg_len = 6.0 + rnd() * 8.0;
                let nx = cx + seg_len;
                let dy = (rnd() - 0.5) * 4.0;
                ctx.quadratic_curve_to(cx + seg_len / 2.0, y + dy * 1.5, nx, y + dy);
                cx = nx;
            }
            ctx.stroke();
        }
        // Sign-off "Love," on its own line + signature.
        ctx.set_font("italic 15px cursive");
        ctx.set_fill_style("#302010");
        let signoffs = ["Love always,", "Yours,", "With love,", "Missing you,"];
        ctx.fill_text(signoffs[(rnd() * signoffs.len() as f64).floor() as usize], 24, h - 40.0);
        // Signature — big wavy scribble.
        ctx.set_stroke_style("rgba(30,20,10,0.85)");
        ctx.set_line_width(1.6);
        ctx.begin_path();
        ctx.move_to(30, h - 18.0);
        ctx.quadratic_curve_to(45, h - 30.0, 60, h - 20.0);
        ctx.quadratic_curve_to(75, h - 10.0, 90, h - 22.0);
        ctx.quadratic_curve_to(105, h - 34.0, 120, h - 18.0);
        ctx.stroke();
        // Fold crease horizontal.
        ctx.set_stroke_style("rgba(120,90,60,0.30)");
        ctx.set_line_width(1.5);
        ctx.begin_path();
        ctx.move_to(0, h * 0.5);
        ctx.line_to(w, h * 0.5);
        ctx.stroke();
    })
);
