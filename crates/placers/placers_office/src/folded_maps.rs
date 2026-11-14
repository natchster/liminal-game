//! `place_folded_maps`
use crate::shared::tri_geo;
use game::prelude::*;
use placers_kit::place::jitter;
use placers_kit::place::pick_open;

// Folded map: 5% chance per level. Folded paper square w/ road texture.
pub(crate) fn place_folded_maps(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let map_mat = lambert!({ map: map_texture(), side: DoubleSide });
    let map = group();
    let base = mesh(plane_geo!(0.11, 0.11), &map_mat);
    base.rotation().x = -PI / 2.0;
    base.position().y = 0.003;
    map.add(&base);
    // Small folded corner (raised triangle).
    let fold_geom = tri_geo(vec![0.04, 0.003, 0.03, 0.055, 0.003, 0.055, 0.055, 0.010, 0.03], vec![0, 1, 2]);
    let fold = mesh(fold_geom, &map_mat);
    map.add(&fold);
    let (px, pz) = jitter(g, x, z, 2.4);
    map.position().set(px, 0, pz);
    map.rotation().y = rnd() * PI * 2.0;
    g.world.add(&map);
}

// `map_texture`: beige map, fold creases, winding roads, landmark dots.
game::lazy_static_tex!(
    map_texture,
    canvas_tex(96, 96, |ctx, w, h| {
        // Beige base.
        ctx.set_fill_style("#e8d8a0");
        ctx.fill_rect(0, 0, w, h);
        // Fold crease lines (dark).
        ctx.set_stroke_style("rgba(120, 90, 40, 0.6)");
        ctx.set_line_width(1);
        ctx.begin_path();
        ctx.move_to(w / 3.0, 0);
        ctx.line_to(w / 3.0, h);
        ctx.move_to(2.0 * w / 3.0, 0);
        ctx.line_to(2.0 * w / 3.0, h);
        ctx.move_to(0, h / 3.0);
        ctx.line_to(w, h / 3.0);
        ctx.move_to(0, 2.0 * h / 3.0);
        ctx.line_to(w, 2.0 * h / 3.0);
        ctx.stroke();
        // Road lines (winding).
        ctx.set_stroke_style("#804020");
        ctx.set_line_width(1.5);
        for _i in 0..3 {
            ctx.begin_path();
            ctx.move_to(rnd() * w, 0);
            ctx.bezier_curve_to(w * rnd(), h * 0.3, w * rnd(), h * 0.7, rnd() * w, h);
            ctx.stroke();
        }
        // Small green landmark dots.
        ctx.set_fill_style("#406030");
        for _i in 0..6 {
            ctx.begin_path();
            ctx.arc(rnd() * w, rnd() * h, 2, 0, PI * 2.0);
            ctx.fill();
        }
        // Red star landmark.
        ctx.set_fill_style("#c02020");
        ctx.begin_path();
        ctx.arc(w * 0.5, h * 0.5, 4, 0, PI * 2.0);
        ctx.fill();
    })
);
