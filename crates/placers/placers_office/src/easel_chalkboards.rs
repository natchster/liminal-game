//! `place_easel_chalkboards`
use game::prelude::*;
use placers_kit::place::open_tiles;

pub(crate) fn place_easel_chalkboards(g: &mut Game, _dist: &[i32]) {
    g.pp.chalkboard_positions.clear(); // reset per level
    if rnd() < 0.975 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    let wood_mat = lambert!({ color: 0x8b5a2b });
    let _board_back_mat = lambert!({ color: 0x1a3020 });
    let chalk_face_mat = lambert!({ map: chalk_content_texture() });
    let ec = group();
    // Easel — 3 legs meeting at top (A-frame style w/ back support).
    let easel_h = 1.30;
    let leg_fl = mesh(pg("EaselChalkboards__leg", cyl_geo!(0.012, 0.014, easel_h, 5)), &wood_mat);
    leg_fl.rotation().z = -0.12;
    leg_fl.rotation().x = -0.06;
    leg_fl.position().set(-0.12, easel_h / 2.0 - 0.005, 0.040);
    ec.add(&leg_fl);
    let leg_fr = mesh(pg("EaselChalkboards__leg", cyl_geo!(0.012, 0.014, easel_h, 5)), &wood_mat);
    leg_fr.rotation().z = 0.12;
    leg_fr.rotation().x = -0.06;
    leg_fr.position().set(0.12, easel_h / 2.0 - 0.005, 0.040);
    ec.add(&leg_fr);
    let leg_back = mesh(pg("EaselChalkboards__leg", cyl_geo!(0.012, 0.014, easel_h, 5)), &wood_mat);
    leg_back.rotation().x = 0.20;
    leg_back.position().set(0, easel_h / 2.0 - 0.012, -0.12); // top meets the front legs, foot on the floor
    ec.add(&leg_back);
    // Cross-brace.
    let brace = mesh(pg("EaselChalkboards__brace", cyl_geo!(0.005, 0.005, 0.30, 4)), &wood_mat);
    brace.rotation().z = PI / 2.0;
    brace.position().set(0, easel_h * 0.35, 0.04);
    ec.add(&brace);
    // Chalkboard — rectangular flat panel resting against front legs.
    let (board_w, board_h, board_t) = (0.44, 0.32, 0.014);
    let board_back = mesh(pg("EaselChalkboards__board", box_geo!(board_w, board_h, board_t)), &wood_mat);
    board_back.position().set(0, easel_h * 0.66, 0.05);
    board_back.rotation().x = -0.08;
    ec.add(&board_back);
    let board_face = mesh(plane_geo!(board_w - 0.020, board_h - 0.020), &chalk_face_mat);
    board_face.position().set(0, easel_h * 0.66, 0.05 + board_t / 2.0 + 0.001);
    board_face.rotation().x = -0.08;
    ec.add(&board_face);
    // Chalk tray at bottom of board.
    let chalk_tray = mesh(pg("EaselChalkboards__tray", box_geo!(board_w, 0.014, 0.030)), &wood_mat);
    chalk_tray.position().set(0, easel_h * 0.66 - board_h * 0.55, 0.05 + board_t / 2.0 + 0.008);
    chalk_tray.rotation().x = -0.08;
    ec.add(&chalk_tray);
    // 2-3 chalk sticks on tray + optional eraser block.
    let chalk_colors = [0xf0f0e8, 0xf0d0d0, 0xd0d0f0, 0xf0e0a0];
    let chalk_count = 2 + (rnd() * 2.0).floor() as i32;
    for i in 0..chalk_count {
        let chalk_mat = lambert!({ color: choice(&chalk_colors) });
        let chalk = mesh(pg("EaselChalkboards__chalk", cyl_geo!(0.005, 0.005, 0.055, 6)), &chalk_mat);
        chalk.rotation().z = PI / 2.0;
        let base_chalk_y = 0.05 + (rnd() - 0.5) * 0.1;
        chalk.rotation().y = base_chalk_y;
        chalk.position().set(
            -board_w * 0.3 + i as f64 * 0.06 + (rnd() - 0.5) * 0.010,
            easel_h * 0.66 - board_h * 0.55 + 0.014,
            0.05 + board_t / 2.0 + 0.010,
        );
        ec.add(&chalk);
        // Subtle chalk-stick roll animator — 30% chance per chalk.
        if rnd() < 0.3 {
            let roll_phase = rnd() * PI * 2.0;
            let c2 = chalk.clone();
            g.animate(move |_g, _dt, t| {
                c2.rotation().x = (t * 0.9 + roll_phase).dsin() * 0.08;
            });
        }
    }
    // 60% chance: felt eraser block.
    if rnd() < 0.6 {
        let eraser_mat = lambert!({ color: 0x604020 });
        let eraser = mesh(pg("EaselChalkboards__eraser", box_geo!(0.05, 0.010, 0.020)), &eraser_mat);
        eraser.position().set(board_w * 0.30, easel_h * 0.66 - board_h * 0.55 + 0.014, 0.05 + board_t / 2.0 + 0.010);
        ec.add(&eraser);
        let felt_mat = lambert!({ color: 0x805030 });
        let felt = mesh(pg("EaselChalkboards__felt", box_geo!(0.048, 0.003, 0.018)), &felt_mat);
        felt.position().set(board_w * 0.30, easel_h * 0.66 - board_h * 0.55 + 0.020, 0.05 + board_t / 2.0 + 0.010);
        ec.add(&felt);
    }
    ec.position().set(g.wx(x) + (rnd() - 0.5) * 1.2, 0, g.wz(z) + (rnd() - 0.5) * 1.2);
    ec.rotation().y = rnd() * PI * 2.0;
    g.world.add(&ec);
    let p = ec.pos();
    g.pp.chalkboard_positions.push(v3(p.x, 0, p.z));
}

/// A uniformly random element (one `rnd()` draw).
fn pick<'a>(a: &[&'a str]) -> &'a str {
    a[(rnd() * a.len() as f64).floor() as usize]
}

/// Shuffle by sorting with a random comparator (`rnd() - 0.5`), using TimSort's
/// short-array path (count-and-make-run, then binary insertion sort) with one
/// draw per comparison. The exact comparison sequence fixes how many draws are
/// consumed, so it is kept on purpose.
fn shuffle_by_random_comparator<T: Clone>(v: &mut [T]) {
    let n = v.len();
    if n < 2 {
        return;
    }
    let cmp = || rnd() - 0.5;
    // count the first run over [0, n), reversing it if descending
    let mut run = 2usize;
    let order = cmp();
    let desc = order < 0.0;
    let mut idx = 2;
    while idx < n {
        let o = cmp();
        if desc {
            if o >= 0.0 {
                break;
            }
        } else if o < 0.0 {
            break;
        }
        run += 1;
        idx += 1;
    }
    if desc {
        v[0..run].reverse();
    }
    // binary insertion sort of [run, n) into the sorted prefix
    let mut start = run;
    while start < n {
        let (mut left, mut right) = (0usize, start);
        let pivot = v[start].clone();
        while left < right {
            let mid = left + ((right - left) >> 1);
            let o = cmp();
            if o < 0.0 {
                right = mid;
            } else {
                left = mid + 1;
            }
        }
        let mut p = start;
        while p > left {
            v[p] = v[p - 1].clone();
            p -= 1;
        }
        v[left] = pivot;
        start += 1;
    }
}

game::lazy_static_tex!(
    chalk_content_texture,
    canvas_tex(240, 180, |ctx, w, h| {
        // Dark green chalkboard base.
        ctx.set_fill_style("#1a3020");
        ctx.fill_rect(0, 0, w, h);
        // Subtle chalk-erased haze texture.
        ctx.set_fill_style("rgba(200,200,200,0.05)");
        for _ in 0..20 {
            ctx.fill_rect(rnd() * w, rnd() * h, 30.0 + rnd() * 40.0, 2.0 + rnd() * 3.0);
        }
        // Bimodal content: math equations or written notes.
        let is_math = rnd() < 0.5;
        ctx.set_fill_style("#f0f0e8");
        if is_math {
            ctx.set_font("bold 18px serif");
            ctx.set_text_align("left");
            let equations =
                vec!["a² + b² = c²", "E = mc²", "x = (-b ± √(b²-4ac))/2a", "F = ma", "π ≈ 3.14159", "∫ x dx = x²/2", "sin²θ + cos²θ = 1"];
            // Shuffle + pick 3-4.
            let mut chosen = equations.clone();
            shuffle_by_random_comparator(&mut chosen);
            let take = 3 + (rnd() * 2.0).floor() as usize;
            chosen.truncate(take);
            for (i, e) in chosen.iter().enumerate() {
                ctx.fill_text(e, 20, 40.0 + i as f64 * 32.0);
            }
            // Draw a curve/graph in bottom-right area.
            ctx.set_stroke_style("#f0f0e8");
            ctx.set_line_width(1.5);
            ctx.begin_path();
            ctx.move_to(w * 0.6, h - 15.0);
            let mut px = 0.0;
            while px < w * 0.35 {
                let py = h - 15.0 - (px * 0.10).dsin() * 20.0 - px * 0.15;
                ctx.line_to(w * 0.6 + px, py);
                px += 3.0;
            }
            ctx.stroke();
            // X/Y axes.
            ctx.begin_path();
            ctx.move_to(w * 0.6, h - 15.0);
            ctx.line_to(w * 0.95, h - 15.0);
            ctx.move_to(w * 0.6, h - 15.0);
            ctx.line_to(w * 0.6, h - 70.0);
            ctx.stroke();
        } else {
            // Written notes — cursive/wavy scribble lines.
            ctx.set_font("bold 20px serif");
            ctx.set_text_align("left");
            ctx.fill_text(pick(&["HOMEWORK", "DUE FRIDAY", "REMEMBER:", "CHAPTER 12", "DO NOT ERASE"]), 15, 30);
            // Underline.
            ctx.set_stroke_style("#f0f0e8");
            ctx.set_line_width(2);
            ctx.begin_path();
            ctx.move_to(15, 34);
            ctx.line_to(w * 0.5, 34);
            ctx.stroke();
            ctx.set_line_width(1.5);
            for i in 0..6 {
                let y = 60.0 + i as f64 * 18.0;
                if y > h - 20.0 {
                    break;
                }
                let line_len = w * (0.5 + rnd() * 0.4);
                ctx.begin_path();
                ctx.move_to(15, y);
                let mut cx2 = 15.0;
                while cx2 < 15.0 + line_len {
                    let seg = 6.0 + rnd() * 8.0;
                    let nx = cx2 + seg;
                    let dy = (rnd() - 0.5) * 4.0;
                    ctx.quadratic_curve_to(cx2 + seg / 2.0, y + dy * 1.3, nx, y + dy);
                    cx2 = nx;
                }
                ctx.stroke();
            }
        }
        // Occasional chalk smudges on edges.
        ctx.set_fill_style("rgba(240,240,232,0.20)");
        for _ in 0..8 {
            ctx.begin_path();
            ctx.ellipse(rnd() * w, rnd() * h, 10.0 + rnd() * 10.0, 2, rnd() * PI, 0, PI * 2.0);
            ctx.fill();
        }
    })
);
