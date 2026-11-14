//! `place_wall_aprons`
use crate::shared::kitchen_hash;
use game::prelude::*;
use placers_kit::place::face_wall;
use placers_kit::place::pick;
use placers_kit::place::wall_tiles;

// Hanging wall apron: 5% chance per level (Blender family `coat_hook_broom_apron_and_rugs`): a
// cotton bib apron, 72 x 86 cm, hung by its neck strap on a cast iron hook
// (`r06`'s): the bib's top fallen into a cowl between the strap's ends, the
// waist's corners dropped, the skirt against the wall in folds, its patch
// pocket, its ties hanging; faded, stained where hands were wiped on it.
pub(crate) fn place_wall_aprons(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let wt = wall_tiles(g);
    if wt.is_empty() {
        return;
    }
    let (x, z, dx, dz) = pick(&wt);
    let apron_colors: [u32; 6] = [0xc02020, 0x2060a0, 0x408040, 0xc0a040, 0x606060, 0xd050a0];
    // the old pick, as a real cloth: its dye faded (the blue a butcher's
    // stripe, painted in its map)
    let (cloth, stripe) = match pick(&apron_colors) {
        0xc02020 => (0x8c3d34, false),
        0x2060a0 => (0xffffff, true),
        0x408040 => (0x66705a, false),
        0xc0a040 => (0xa68e5e, false),
        0x606060 => (0x77746c, false),
        _ => (0xa67876, false),
    };
    let vary = kitchen_hash(x, z, 0xa970);
    let apron = group();
    // the hook at 1.85 m, its plate on the wall (8 mm behind the group's
    // origin); the apron as modelled hanging from it
    let hook_y = 1.85;
    let iron = phong!({ color: 0x1c1b1a, specular: 0x1a1a1a, shininess: 22 });
    let hook = mesh(game::assets::props::prop_part_as_modelled("Real__coatHook").unwrap_or_else(|| box_geo!(0.026, 0.12, 0.1)), &iron);
    hook.position().set(0, hook_y, -0.008);
    apron.add(&hook);
    if let Some(geo) = game::assets::props::prop_part_as_modelled("Real__apronCloth") {
        let bb = geo.bounding_box();
        let mat = lambert!({ map: &apron_tex(stripe, vary, [bb.min.x, bb.max.x, bb.min.y, bb.max.y]), color: cloth, side: DoubleSide });
        let m = mesh(geo, &mat);
        m.position().set(0, hook_y, -0.008);
        apron.add(&m);
    }
    apron.rotation().y = face_wall(dx, dz);
    apron.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - 0.008), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - 0.008));
    g.world.add(&apron);
}

/// The apron's cloth seen from the front over its bounds `b` (x0, x1, y0,
/// y1 in its frame, the hook at the origin): cotton twill, white to be laid
/// over its colour (or a butcher's navy stripe), faded, grease spotted down
/// the bib, hands wiped at the hips and the pocket's mouth, the hem grimed
fn apron_tex(stripe: bool, vary: u32, b: [f64; 4]) -> Tex {
    canvas_tex(256, 512, move |ctx, w, h| {
        let seed = (rnd() * 4.0e9) as u32 ^ vary;
        with_own_stream(seed, || {
            let (x0, x1, y0, y1) = (b[0], b[1], b[2], b[3]);
            // where a point of the apron (its frame) is on the map
            let at = |x: f64, y: f64| ((x - x0) / (x1 - x0) * w, (1.0 - (y - y0) / (y1 - y0)) * h);
            let px_m = w / (x1 - x0);
            ctx.set_fill_style(if stripe { "#d6d1c4" } else { "#f2f0ea" });
            ctx.fill_rect(0, 0, w, h);
            if stripe {
                // the butcher's stripe: 3 mm navy every 9 mm, down it
                ctx.set_fill_style("#2e3a52");
                let mut x = 0.0;
                while x < w {
                    ctx.fill_rect(x, 0, (0.003 * px_m).max(1.0), h);
                    x += 0.009 * px_m;
                }
            }
            // the twill's diagonal and the weave's unevenness
            ctx.set_line_width(1);
            let mut d = -h;
            while d < w {
                ctx.set_stroke_style(format!("rgba(0,0,0,{})", 0.03 + rnd() * 0.04));
                ctx.begin_path();
                ctx.move_to(d, 0);
                ctx.line_to(d + h * 0.5, h);
                ctx.stroke();
                d += 2.0;
            }
            // grease and old splashes down the bib and the skirt: each a
            // cluster of soft blots, uneven, some run down a little
            for _ in 0..6 {
                let (cx, cy) = ((rnd() - 0.5) * 0.30, -0.42 - rnd() * 0.50);
                let size = 0.006 + rnd() * 0.02;
                let a = 0.10 + rnd() * 0.12;
                for _ in 0..(3 + (rnd() * 4.0) as i32) {
                    let (x, y) = at(cx + (rnd() - 0.5) * size * 2.5, cy + (rnd() - 0.5) * size * 2.0);
                    let r = size * (0.4 + rnd()) * px_m;
                    ctx.save();
                    ctx.translate(x, y);
                    ctx.rotate(rnd() * 3.0);
                    ctx.scale(1, 0.5 + 0.6 * rnd());
                    let gr = ctx.create_radial_gradient(0, 0, 0, 0, 0, r);
                    gr.add_color_stop(0, &format!("rgba(74,56,34,{a})"));
                    gr.add_color_stop(0.7, &format!("rgba(74,56,34,{})", a * 0.7));
                    gr.add_color_stop(1, "rgba(74,56,34,0)");
                    ctx.set_fill_style(gr);
                    ctx.begin_path();
                    ctx.arc(0, 0, r, 0, 7);
                    ctx.fill();
                    ctx.restore();
                }
                if rnd() < 0.5 {
                    // a run down from it
                    let (x, y) = at(cx, cy);
                    let l = (0.02 + rnd() * 0.05) * px_m;
                    let gr = ctx.create_linear_gradient(0, y, 0, y + l);
                    gr.add_color_stop(0, &format!("rgba(74,56,34,{})", a * 0.8));
                    gr.add_color_stop(1, "rgba(74,56,34,0)");
                    ctx.set_fill_style(gr);
                    ctx.fill_rect(x - 1.0, y, 2.0 + rnd() * 1.5, l);
                }
            }
            // hands wiped at the hips and at the pocket's mouth
            for (hx, hy, rx, ry, a) in [(-0.24, -0.70, 0.07, 0.12, 0.30), (0.24, -0.70, 0.07, 0.12, 0.30), (0.0, -0.80, 0.18, 0.03, 0.22)] {
                let (x, y) = at(hx + (rnd() - 0.5) * 0.04, hy + (rnd() - 0.5) * 0.04);
                ctx.save();
                ctx.translate(x, y);
                ctx.scale(1, ry / rx);
                let r = rx * px_m;
                let gr = ctx.create_radial_gradient(0, 0, 0, 0, 0, r);
                gr.add_color_stop(0, &format!("rgba(56,44,32,{a})"));
                gr.add_color_stop(1, "rgba(56,44,32,0)");
                ctx.set_fill_style(gr);
                ctx.begin_path();
                ctx.arc(0, 0, r, 0, 7);
                ctx.fill();
                ctx.restore();
            }
            // the hem and the ties' ends grimed, the top faded paler
            let (_, yh) = at(0.0, -1.10);
            let gr = ctx.create_linear_gradient(0, yh, 0, h);
            gr.add_color_stop(0, "rgba(50,42,34,0)");
            gr.add_color_stop(1, "rgba(50,42,34,0.35)");
            ctx.set_fill_style(gr);
            ctx.fill_rect(0, yh, w, h - yh);
            let (_, yt) = at(0.0, -0.55);
            let gr = ctx.create_linear_gradient(0, 0, 0, yt);
            gr.add_color_stop(0, "rgba(236,232,222,0.22)");
            gr.add_color_stop(1, "rgba(236,232,222,0)");
            ctx.set_fill_style(gr);
            ctx.fill_rect(0, 0, w, yt);
            for _ in 0..400 {
                let v = if rnd() < 0.5 { "255,255,255" } else { "0,0,0" };
                ctx.set_fill_style(format!("rgba({v},{})", 0.03 + rnd() * 0.06));
                ctx.fill_rect(rnd() * w, rnd() * h, 1.0 + rnd() * 2.0, 1.0 + rnd() * 2.0);
            }
        });
    })
}
