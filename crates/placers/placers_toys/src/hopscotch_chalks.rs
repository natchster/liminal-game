//! `place_hopscotch_chalks`
use game::prelude::*;
use placers_kit::place::pick_open_tile;

pub(crate) fn place_hopscotch_chalks(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let Some((x, z)) = pick_open_tile(g) else { return };
    let decal_mat = basic_mat!({ map: hopscotch_texture(), transparent: true, opacity: 0.75, depthWrite: false });
    let decal = mesh(plane_geo!(1.2, 2.4), &decal_mat);
    decal.rotation().x = -PI / 2.0;
    let px = g.wx(x) + (rnd() - 0.5) * 1.5;
    let pz = g.wz(z) + (rnd() - 0.5) * 1.5;
    decal.position().set(px, 0.003, pz);
    decal.rotation().z = rnd() * PI * 2.0;
    g.world.add(&decal);
}

// Hopscotch chalk decal: 4% chance per level. Canvas
// texture with numbered chalk squares on floor. Games family.
game::lazy_static_tex!(hopscotch_texture, {
    let t = canvas_tex_opt(256, 512, false, |ctx, w, h| {
        ctx.set_fill_style("rgba(0,0,0,0)");
        ctx.clear_rect(0, 0, w, h);
        let chalk_color = "#d0d0d0";
        ctx.set_stroke_style(chalk_color);
        ctx.set_fill_style(chalk_color);
        ctx.set_line_width(4);
        // 9 squares in hopscotch layout (1, 2-3, 4, 5-6, 7, 8-9).
        let cell_w = 60.0;
        let cx = w / 2.0;
        let layout: [(i32, f64); 6] = [
            (1, 40.0),
            (2, 40.0 + cell_w + 8.0),
            (1, 40.0 + 2.0 * (cell_w + 8.0)),
            (2, 40.0 + 3.0 * (cell_w + 8.0)),
            (1, 40.0 + 4.0 * (cell_w + 8.0)),
            (2, 40.0 + 5.0 * (cell_w + 8.0)),
        ];
        ctx.set_font("bold 32px sans-serif");
        ctx.set_text_align("center");
        ctx.set_text_baseline("middle");
        let mut num = 1;
        for &(count, ry) in layout.iter() {
            for i in 0..count {
                let rx = cx + (i as f64 - (count - 1) as f64 / 2.0) * (cell_w + 4.0) - cell_w / 2.0;
                // Rough chalk box.
                ctx.begin_path();
                ctx.set_stroke_style(&format!("rgba(220,220,220,{})", 0.5 + rnd() * 0.4));
                ctx.rect(rx, ry, cell_w, cell_w);
                ctx.stroke();
                // Number.
                ctx.set_fill_style(&format!("rgba(220,220,220,{})", 0.5 + rnd() * 0.4));
                ctx.fill_text(&num.to_string(), rx + cell_w / 2.0, ry + cell_w / 2.0);
                num += 1;
            }
        }
    });
    // TODO: this texture is meant to premultiply alpha on upload; it is
    // currently uploaded straight-alpha.
    t
});
