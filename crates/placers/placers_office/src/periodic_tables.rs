//! `place_periodic_tables`
use game::prelude::*;
use placers_kit::place::face_wall;
use placers_kit::place::wall_tiles;

pub(crate) fn place_periodic_tables(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.98 {
        return;
    }
    let wt = wall_tiles(g);
    if wt.is_empty() {
        return;
    }
    let (x, z, dx, dz) = wt[(rnd() * wt.len() as f64).floor() as usize];
    let frame_colors = [0x604020, 0x8b5a2b, 0x404040, 0x2a2a2a];
    let frame_mat = lambert!({ color: choice(&frame_colors) });
    let chart_mat = lambert!({ map: periodic_table_texture() });
    let pt = group();
    let mount_y = 1.55;
    let (frame_w, frame_h) = (0.36, 0.26);
    let frame = mesh(pg("PeriodicTables__frame", box_geo!(frame_w, frame_h, 0.014)), &frame_mat);
    frame.position().set(0, mount_y, 0.011);
    pt.add(&frame);
    let chart = mesh(plane_geo!(frame_w - 0.016, frame_h - 0.016), &chart_mat);
    chart.position().set(0, mount_y, 0.019);
    pt.add(&chart);
    pt.rotation().y = face_wall(dx, dz);
    pt.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - 0.015), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - 0.015));
    pt.rotation().z = (rnd() - 0.5) * 0.02;
    g.world.add(&pt);
}

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
