//! Module-level canvas textures of chunk p13 (built once, on the native RNG).
#![allow(unused_variables, unused_mut, unused_assignments, unused_imports, dead_code, clippy::all)]
use game::prelude::*;

/// `arr[Math.floor(Math.random() * arr.length)]`
fn pick<'a>(a: &[&'a str]) -> &'a str { a[(rnd() * a.len() as f64).floor() as usize] }

/// `[...arr].sort(() => Math.random() - 0.5)`: V8's TimSort on a short array
/// (count-and-make-run + binary insertion sort), one draw per comparison.
pub fn v8_random_sort<T: Clone>(v: &mut Vec<T>) {
    let n = v.len();
    if n < 2 { return; }
    let cmp = || rnd() - 0.5;
    // CountAndMakeRun(0, n)
    let mut run = 2usize;
    let order = cmp();
    let desc = order < 0.0;
    let mut idx = 2;
    while idx < n {
        let o = cmp();
        if desc { if o >= 0.0 { break; } } else if o < 0.0 { break; }
        run += 1;
        idx += 1;
    }
    if desc { v[0..run].reverse(); }
    // BinaryInsertionSort(0, run, n)
    let mut start = run;
    while start < n {
        let (mut left, mut right) = (0usize, start);
        let pivot = v[start].clone();
        while left < right {
            let mid = left + ((right - left) >> 1);
            let o = cmp();
            if o < 0.0 { right = mid; } else { left = mid + 1; }
        }
        let mut p = start;
        while p > left { v[p] = v[p - 1].clone(); p -= 1; }
        v[left] = pivot;
        start += 1;
    }
}

// ------------------------------------------------------------------ line 25756
game::lazy_static_tex!(propaganda_poster_texture, canvas_tex(180, 260, |ctx, w, h| {
    // Bold vibrant background. (bg, accent, slogan, sub)
    let variants: [(&str, &str, &str, &str); 6] = [
        ("#a02020", "#f0e0c0", "DUTY CALLS", "DO YOUR PART"),
        ("#204080", "#f0e040", "STAY STRONG", "UNITED WE STAND"),
        ("#402010", "#e0d090", "CONSERVE", "FOOD WINS WARS"),
        ("#808040", "#a02020", "BUY BONDS", "SUPPORT OUR TROOPS"),
        ("#603030", "#f0e0a0", "REMEMBER", "NEVER FORGET"),
        ("#204020", "#f0e0c0", "VICTORY", "FOR FREEDOM"),
    ];
    let v = variants[(rnd() * variants.len() as f64).floor() as usize];
    ctx.set_fill_style(v.0);
    ctx.fill_rect(0, 0, w, h);
    // Decorative border stripe.
    ctx.set_stroke_style(v.1);
    ctx.set_line_width(6);
    ctx.stroke_rect(8, 8, w - 16.0, h - 16.0);
    // Bold main slogan top.
    ctx.set_fill_style(v.1);
    ctx.set_font("bold 28px sans-serif");
    ctx.set_text_align("center");
    ctx.fill_text(v.2, w / 2.0, 50);
    // Center silhouette — 3 variants of imagery.
    let mid_y = h * 0.30;
    let img_type = (rnd() * 3.0).floor() as i32;
    if img_type == 0 {
        // Silhouette figures — 3 people looking up/marching.
        ctx.set_fill_style("#101010");
        for i in 0..3 {
            let fx = w * 0.30 + i as f64 * w * 0.20;
            ctx.begin_path();
            ctx.arc(fx, mid_y + 30.0, 10, 0, PI * 2.0);
            ctx.fill();
            ctx.fill_rect(fx - 12.0, mid_y + 40.0, 24, 50);
            ctx.fill_rect(fx - 20.0, mid_y + 45.0, 8, 30);
            ctx.fill_rect(fx + 12.0, mid_y + 45.0, 8, 30);
            ctx.fill_rect(fx - 10.0, mid_y + 90.0, 8, 30);
            ctx.fill_rect(fx + 2.0, mid_y + 90.0, 8, 30);
        }
    } else if img_type == 1 {
        // Fist symbol — bold clenched fist silhouette.
        ctx.set_fill_style("#101010");
        ctx.begin_path();
        let (cx, cy) = (w / 2.0, mid_y + 60.0);
        ctx.arc(cx, cy, 30, 0, PI * 2.0);
        ctx.fill();
        ctx.fill_rect(cx - 15.0, cy + 25.0, 30, 40);
        for i in 0..4 {
            ctx.begin_path();
            ctx.arc(cx - 20.0 + i as f64 * 12.0, cy - 12.0, 6, 0, PI * 2.0);
            ctx.fill();
        }
    } else {
        // Rising sun w/ rays + text banner.
        ctx.set_fill_style("#f0c040");
        ctx.begin_path();
        ctx.arc(w / 2.0, mid_y + 60.0, 35, 0, PI * 2.0);
        ctx.fill();
        ctx.set_stroke_style("#f0c040");
        ctx.set_line_width(4);
        for i in 0..8 {
            let ang = (i as f64 / 8.0) * PI * 2.0;
            ctx.begin_path();
            ctx.move_to(w / 2.0 + ang.cos() * 40.0, mid_y + 60.0 + ang.sin() * 40.0);
            ctx.line_to(w / 2.0 + ang.cos() * 70.0, mid_y + 60.0 + ang.sin() * 70.0);
            ctx.stroke();
        }
        // Silhouette dark bird.
        ctx.set_fill_style("#101010");
        ctx.begin_path();
        ctx.move_to(w / 2.0 - 20.0, mid_y + 60.0);
        ctx.line_to(w / 2.0, mid_y + 45.0);
        ctx.line_to(w / 2.0 + 20.0, mid_y + 60.0);
        ctx.line_to(w / 2.0, mid_y + 55.0);
        ctx.close_path();
        ctx.fill();
    }
    // Sub-slogan (smaller, below image).
    ctx.set_fill_style(v.1);
    ctx.set_font("bold 14px sans-serif");
    ctx.fill_text(v.3, w / 2.0, h - 55.0);
    // Year + attribution at bottom.
    ctx.set_font("9px serif");
    ctx.fill_text(pick(&["1942", "1943", "1944", "1917"]), w / 2.0, h - 24.0);
    ctx.set_font("7px serif");
    ctx.fill_text(pick(&["OFFICE OF WAR INFORMATION", "WAR PRODUCTION BOARD", "WAR DEPARTMENT"]), w / 2.0, h - 14.0);
}));

// ------------------------------------------------------------------ line 26322
game::lazy_static_tex!(anatomy_chart_texture, canvas_tex(180, 280, |ctx, w, h| {
    // Aged cream chart paper.
    ctx.set_fill_style("#f0e8c8");
    ctx.fill_rect(0, 0, w, h);
    // Faint tan patches for aging.
    ctx.set_fill_style("rgba(140,100,60,0.10)");
    for _ in 0..30 {
        ctx.begin_path();
        ctx.arc(rnd() * w, rnd() * h, 8.0 + rnd() * 12.0, 0, PI * 2.0);
        ctx.fill();
    }
    // Title banner top.
    ctx.set_fill_style("#302010");
    ctx.set_font("bold 14px serif");
    ctx.set_text_align("center");
    ctx.fill_text(pick(&["HUMAN ANATOMY", "THE HUMAN FIGURE", "MUSCULAR SYSTEM", "SKELETAL SYSTEM", "THE HUMAN BODY"]), w / 2.0, 22);
    // Sub-title.
    ctx.set_font("italic 9px serif");
    ctx.fill_text("Educational Reference — 1912", w / 2.0, 36);
    // Central human silhouette figure — full body outline.
    let (cx, cy) = (w / 2.0, h * 0.55);
    let fh = h * 0.55;
    // Head circle.
    ctx.set_stroke_style("#402020");
    ctx.set_line_width(1.5);
    ctx.begin_path();
    ctx.arc(cx, cy - fh * 0.4, fh * 0.10, 0, PI * 2.0);
    ctx.stroke();
    // Body torso.
    ctx.begin_path();
    ctx.move_to(cx - fh * 0.16, cy - fh * 0.30);
    ctx.line_to(cx - fh * 0.14, cy);
    ctx.line_to(cx - fh * 0.10, cy + fh * 0.05);
    ctx.line_to(cx + fh * 0.10, cy + fh * 0.05);
    ctx.line_to(cx + fh * 0.14, cy);
    ctx.line_to(cx + fh * 0.16, cy - fh * 0.30);
    ctx.close_path();
    ctx.stroke();
    // Arms.
    ctx.begin_path();
    ctx.move_to(cx - fh * 0.16, cy - fh * 0.28);
    ctx.quadratic_curve_to(cx - fh * 0.28, cy - fh * 0.10, cx - fh * 0.24, cy + fh * 0.10);
    ctx.stroke();
    ctx.begin_path();
    ctx.move_to(cx + fh * 0.16, cy - fh * 0.28);
    ctx.quadratic_curve_to(cx + fh * 0.28, cy - fh * 0.10, cx + fh * 0.24, cy + fh * 0.10);
    ctx.stroke();
    // Legs.
    ctx.begin_path();
    ctx.move_to(cx - fh * 0.10, cy + fh * 0.05);
    ctx.quadratic_curve_to(cx - fh * 0.13, cy + fh * 0.25, cx - fh * 0.10, cy + fh * 0.42);
    ctx.stroke();
    ctx.begin_path();
    ctx.move_to(cx + fh * 0.10, cy + fh * 0.05);
    ctx.quadratic_curve_to(cx + fh * 0.13, cy + fh * 0.25, cx + fh * 0.10, cy + fh * 0.42);
    ctx.stroke();
    // Skeleton hints inside body — ribs.
    ctx.set_stroke_style("rgba(60,30,10,0.5)");
    ctx.set_line_width(0.8);
    for i in 0..5 {
        let rib_y = cy - fh * 0.28 + i as f64 * (fh * 0.06);
        ctx.begin_path();
        ctx.arc(cx, rib_y, fh * 0.12 - i as f64 * 0.002 * fh, 0.4, PI - 0.4);
        ctx.stroke();
    }
    // Spine vertical line.
    ctx.begin_path();
    ctx.move_to(cx, cy - fh * 0.30);
    ctx.line_to(cx, cy + fh * 0.05);
    ctx.stroke();
    // Numbered labels around figure.
    let label_count = 5 + (rnd() * 3.0).floor() as i32;
    ctx.set_stroke_style("#a02020");
    ctx.set_line_width(0.7);
    ctx.set_fill_style("#a02020");
    let terms = ["CRANIUM", "CLAVICLE", "STERNUM", "HUMERUS", "RADIUS", "PATELLA", "FEMUR", "TIBIA", "PELVIS", "RIBS"];
    for i in 0..label_count {
        let is_left = i % 2 == 0;
        let label_x = if is_left { 12.0 } else { w - 12.0 };
        let label_y = 60.0 + i as f64 * 30.0;
        let target_x = cx + if is_left { -fh * 0.05 } else { fh * 0.05 };
        let target_y = cy - fh * 0.25 + i as f64 * (fh * 0.10);
        ctx.begin_path();
        ctx.move_to(label_x, label_y);
        ctx.line_to(target_x, target_y);
        ctx.stroke();
        ctx.begin_path();
        ctx.arc(label_x, label_y, 5, 0, PI * 2.0);
        ctx.fill();
        ctx.set_fill_style("#302010");
        ctx.set_font("bold 7px serif");
        ctx.set_text_align(if is_left { "left" } else { "right" });
        ctx.fill_text(terms[i as usize % terms.len()], label_x + if is_left { 10.0 } else { -10.0 }, label_y + 2.0);
        ctx.set_fill_style("#a02020");
    }
    // Bottom source citation.
    ctx.set_fill_style("#403020");
    ctx.set_font("italic 8px serif");
    ctx.set_text_align("center");
    ctx.fill_text(pick(&["Gray's Anatomy, 8th Ed.", "Netter Atlas", "Encyclopedia Medica", "Anatomica Universalis"]), w / 2.0, h - 12.0);
}));

// ------------------------------------------------------------------ line 26478
game::lazy_static_tex!(chalk_content_texture, canvas_tex(240, 180, |ctx, w, h| {
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
        let equations = vec!["a² + b² = c²", "E = mc²", "x = (-b ± √(b²-4ac))/2a", "F = ma", "π ≈ 3.14159", "∫ x dx = x²/2", "sin²θ + cos²θ = 1"];
        // Shuffle + pick 3-4.
        let mut chosen = equations.clone();
        v8_random_sort(&mut chosen);
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
            if y > h - 20.0 { break; }
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
}));

// ------------------------------------------------------------------ line 26828
game::lazy_static_tex!(periodic_table_texture, canvas_tex(280, 200, |ctx, w, h| {
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
            if sym.is_empty() { continue; }
            atomic_num += 1;
            let cx = 10.0 + c as f64 * cell_w;
            let cy = grid_top + r as f64 * cell_h;
            // Category-based color (crude grouping by column).
            let cat_idx = if c < 2 { 0 } else if c < 12 { 1 + (r % 8) } else { 6 + (c - 12) % 3 };
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
}));

// ------------------------------------------------------------------ line 27050
game::lazy_static_tex!(framed_map_texture, canvas_tex(240, 180, |ctx, w, h| {
    let variant = (rnd() * 3.0).floor() as i32;
    if variant == 0 {
        // US states — simplified outline w/ inner state divisions.
        ctx.set_fill_style("#e8dcb8");
        ctx.fill_rect(0, 0, w, h);
        ctx.set_fill_style("#a0c0d0");
        ctx.fill_rect(0, 0, w, 20);
        ctx.fill_rect(0, h - 20.0, w, 20);
        ctx.fill_rect(0, 0, 15, h);
        ctx.fill_rect(w - 15.0, 0, 15, h);
        ctx.set_fill_style("#c0d0a0");
        ctx.begin_path();
        ctx.move_to(20, 40);
        ctx.line_to(80, 30);
        ctx.line_to(180, 35);
        ctx.line_to(w - 25.0, 45);
        ctx.line_to(w - 20.0, 80);
        ctx.line_to(w - 30.0, 110);
        ctx.line_to(w - 50.0, 130);
        ctx.line_to(180, 140);
        ctx.line_to(100, 145);
        ctx.line_to(60, 130);
        ctx.line_to(30, 100);
        ctx.line_to(20, 60);
        ctx.close_path();
        ctx.fill();
        ctx.set_stroke_style("rgba(80,60,30,0.6)");
        ctx.set_line_width(0.7);
        for i in 0..8 {
            let x1 = 30.0 + (i as f64 / 7.0) * (w - 60.0);
            ctx.begin_path();
            ctx.move_to(x1, 40);
            ctx.line_to(x1 + (rnd() - 0.5) * 20.0, 140);
            ctx.stroke();
        }
        for i in 0..3 {
            let y = 70.0 + i as f64 * 20.0;
            ctx.begin_path();
            ctx.move_to(30, y);
            ctx.line_to(w - 30.0, y + (rnd() - 0.5) * 10.0);
            ctx.stroke();
        }
        ctx.set_fill_style("#302010");
        ctx.set_font("bold 14px serif");
        ctx.set_text_align("center");
        ctx.fill_text("UNITED STATES", w / 2.0, 15);
        // Compass rose top-right.
        ctx.set_stroke_style("#302010");
        ctx.set_line_width(1);
        ctx.begin_path();
        ctx.move_to(w - 30.0, 30);
        ctx.line_to(w - 30.0, 50);
        ctx.move_to(w - 40.0, 40);
        ctx.line_to(w - 20.0, 40);
        ctx.stroke();
        ctx.set_fill_style("#302010");
        ctx.set_font("bold 6px sans-serif");
        ctx.fill_text("N", w - 30.0, 27);
    } else if variant == 1 {
        // World political — approximate continent silhouettes.
        ctx.set_fill_style("#a0c0d0");
        ctx.fill_rect(0, 0, w, h);
        ctx.set_fill_style("#c0e0a0");
        ctx.begin_path();
        ctx.move_to(30, 50);
        ctx.line_to(70, 40);
        ctx.line_to(80, 80);
        ctx.line_to(70, 130);
        ctx.line_to(50, 150);
        ctx.line_to(30, 100);
        ctx.close_path();
        ctx.fill();
        ctx.begin_path();
        ctx.move_to(100, 40);
        ctx.line_to(140, 35);
        ctx.line_to(150, 65);
        ctx.line_to(140, 100);
        ctx.line_to(120, 145);
        ctx.line_to(105, 130);
        ctx.line_to(100, 90);
        ctx.close_path();
        ctx.fill();
        ctx.begin_path();
        ctx.move_to(160, 40);
        ctx.line_to(230, 50);
        ctx.line_to(210, 80);
        ctx.line_to(200, 100);
        ctx.line_to(180, 90);
        ctx.line_to(155, 70);
        ctx.close_path();
        ctx.fill();
        ctx.begin_path();
        ctx.arc(210, 130, 15, 0, PI * 2.0);
        ctx.fill();
        ctx.set_stroke_style("rgba(60,60,60,0.35)");
        ctx.set_line_width(0.4);
        for i in 0..8 {
            ctx.begin_path();
            ctx.move_to(0, 20 + i * 20);
            ctx.line_to(w, 20 + i * 20);
            ctx.stroke();
        }
        for i in 0..12 {
            ctx.begin_path();
            ctx.move_to(i * 20, 0);
            ctx.line_to(i * 20, h);
            ctx.stroke();
        }
        ctx.set_fill_style("#302010");
        ctx.set_font("bold 14px serif");
        ctx.set_text_align("center");
        ctx.fill_text("WORLD POLITICAL", w / 2.0, 15);
    } else {
        // Star map — dark blue background w/ stars + constellation lines.
        ctx.set_fill_style("#101830");
        ctx.fill_rect(0, 0, w, h);
        for _ in 0..60 {
            let sx = rnd() * w;
            let sy = rnd() * h;
            let sr = rnd() * 1.5 + 0.5;
            ctx.set_fill_style(&format!("rgba(240,240,240,{})", 0.5 + rnd() * 0.5));
            ctx.begin_path();
            ctx.arc(sx, sy, sr, 0, PI * 2.0);
            ctx.fill();
        }
        ctx.set_stroke_style("rgba(200,200,240,0.4)");
        ctx.set_line_width(0.7);
        for _ in 0..3 {
            let start_x = 20.0 + rnd() * (w - 40.0);
            let start_y = 20.0 + rnd() * (h - 40.0);
            ctx.begin_path();
            ctx.move_to(start_x, start_y);
            for _ in 0..5 {
                ctx.line_to(start_x + (rnd() - 0.5) * 60.0, start_y + (rnd() - 0.5) * 60.0);
            }
            ctx.stroke();
        }
        ctx.set_fill_style("#e0e8f0");
        ctx.set_font("bold 12px serif");
        ctx.set_text_align("center");
        ctx.fill_text("CELESTIAL SPHERE", w / 2.0, 15);
        ctx.set_font("italic 7px serif");
        ctx.set_fill_style("rgba(200,200,240,0.6)");
        let names = ["Orion", "Ursa Major", "Cassiopeia", "Draco"];
        for i in 0..3 {
            ctx.fill_text(names[i], 30.0 + rnd() * (w - 60.0), 40.0 + rnd() * (h - 60.0));
        }
    }
}));
