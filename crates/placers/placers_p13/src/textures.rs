//! Canvas textures shared by this crate's placers (built once, on the native RNG).
#![allow(unused_variables, unused_mut, unused_assignments, unused_imports, dead_code, clippy::all)]
use game::prelude::*;

/// A uniformly random element (one `rnd()` draw).
fn pick<'a>(a: &[&'a str]) -> &'a str {
    a[(rnd() * a.len() as f64).floor() as usize]
}

/// Shuffle by sorting with a random comparator (`rnd() - 0.5`), using TimSort's
/// short-array path (count-and-make-run, then binary insertion sort) with one
/// draw per comparison. The exact comparison sequence fixes how many draws are
/// consumed, so it is kept on purpose.
pub fn shuffle_by_random_comparator<T: Clone>(v: &mut Vec<T>) {
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
                let py = h - 15.0 - (px * 0.10).sin() * 20.0 - px * 0.15;
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

game::lazy_static_tex!(
    periodic_table_texture,
    canvas_tex(280, 200, |ctx, w, h| {
        // White chart background.
        ctx.set_fill_style("#f8f4e8");
        ctx.fill_rect(0, 0, w, h);
        ctx.set_fill_style("#101010");
        ctx.set_font("bold 12px serif");
        ctx.set_text_align("center");
        ctx.fill_text("PERIODIC TABLE OF THE ELEMENTS", w / 2.0, 15);
        let grid_top = 30.0;
        let grid_bottom = h - 30.0;
        let grid_h = grid_bottom - grid_top;
        let cols = 18usize;
        let rows = 7usize;
        let cell_w = (w - 20.0) / cols as f64;
        let cell_h = grid_h / rows as f64;
        let category_colors = ["#f0c0a0", "#f0d0a0", "#f0e0a0", "#e0f0a0", "#a0f0c0", "#a0e0f0", "#c0a0f0", "#f0a0d0", "#c0c0c0"];
        let symbols: [[&str; 18]; 7] = [
            ["H", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "", "He"],
            ["Li", "Be", "", "", "", "", "", "", "", "", "", "", "B", "C", "N", "O", "F", "Ne"],
            ["Na", "Mg", "", "", "", "", "", "", "", "", "", "", "Al", "Si", "P", "S", "Cl", "Ar"],
            ["K", "Ca", "Sc", "Ti", "V", "Cr", "Mn", "Fe", "Co", "Ni", "Cu", "Zn", "Ga", "Ge", "As", "Se", "Br", "Kr"],
            ["Rb", "Sr", "Y", "Zr", "Nb", "Mo", "Tc", "Ru", "Rh", "Pd", "Ag", "Cd", "In", "Sn", "Sb", "Te", "I", "Xe"],
            ["Cs", "Ba", "*", "Hf", "Ta", "W", "Re", "Os", "Ir", "Pt", "Au", "Hg", "Tl", "Pb", "Bi", "Po", "At", "Rn"],
            ["Fr", "Ra", "**", "", "", "", "", "", "", "", "", "", "", "", "", "", "", ""],
        ];
        let mut atomic_num = 0;
        for r in 0..rows {
            for c in 0..cols {
                let sym = symbols[r][c];
                if sym.is_empty() {
                    continue;
                }
                atomic_num += 1;
                let cx = 10.0 + c as f64 * cell_w;
                let cy = grid_top + r as f64 * cell_h;
                // Category-based color (crude grouping by column).
                let cat_idx = if c < 2 {
                    0
                } else if c < 12 {
                    1 + (r % 8)
                } else {
                    6 + (c - 12) % 3
                };
                ctx.set_fill_style(category_colors[cat_idx % category_colors.len()]);
                ctx.fill_rect(cx, cy, cell_w - 1.0, cell_h - 1.0);
                ctx.set_stroke_style("#404040");
                ctx.set_line_width(0.5);
                ctx.stroke_rect(cx, cy, cell_w - 1.0, cell_h - 1.0);
                ctx.set_fill_style("#101010");
                ctx.set_font("bold 8px sans-serif");
                ctx.set_text_align("center");
                ctx.fill_text(sym, cx + cell_w / 2.0, cy + cell_h * 0.65);
                if sym != "*" && sym != "**" {
                    ctx.set_font("5px sans-serif");
                    ctx.set_text_align("left");
                    ctx.fill_text(&atomic_num.to_string(), cx + 2.0, cy + 6.0);
                }
            }
        }
        ctx.set_fill_style("#606060");
        ctx.set_font("italic 7px serif");
        ctx.set_text_align("right");
        ctx.fill_text("Standard Reference — Chemistry Dept.", w - 8.0, h - 4.0);
    })
);
