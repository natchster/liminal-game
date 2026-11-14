//! `place_wall_brooms`
use crate::shared::kitchen_hash;
use game::prelude::*;
use placers_kit::place::face_wall;
use placers_kit::place::pick;
use placers_kit::place::wall_tiles;

// Wall broom (hanging): 4% chance per level (Blender batch `r13b`): a
// hand-made round hearth broom, 0.71 m, hung on a nail by a jute loop through
// the hole in its handle's top, its head's back against the wall: broomcorn
// bound round a hardwood stick with twine, its ends splayed, worn short on
// one side and bent over from standing on it, straws sticking out; the
// handle dark where hands held it. (No wider along the wall than the old
// one and spanning the same heights, so it meets the wall's face where the
// old one did.)
pub(crate) fn place_wall_brooms(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let wt = wall_tiles(g);
    if wt.is_empty() {
        return;
    }
    let (x, z, dx, dz) = pick(&wt);
    // which broom this is comes from where it hangs (no draws)
    let vary = kitchen_hash(x, z, 0x6b70);
    let wood = [0x8a6a48, 0x7a5c3c, 0x9a7a54, 0x6e5238][(vary % 4) as usize];
    let twine = [0x8a4436, 0x4e5e3e, 0x9a8a68][((vary >> 4) % 3) as usize];
    let handle_mat = lambert!({ map: &broom_handle_tex(wood, vary) });
    let corn_mat = lambert!({ map: &broom_corn_tex(vary) });
    let steel = phong!({ color: 0x4a4641, specular: 0x262626, shininess: 26 });
    let twine_mat = lambert!({ color: twine });
    let cord_mat = lambert!({ color: 0x7a6a4c });
    let broom = group();
    // the nail, so the broom spans the old one's 1.14-1.85 m; the parts as
    // modelled about where it enters the wall, 10 mm behind the group's origin
    let nail_y = 1.8415;
    for (name, mat) in [
        ("Real__broomHandle", &handle_mat),
        ("Real__broomCorn", &corn_mat),
        ("Real__broomNail", &steel),
        ("Real__broomTwine", &twine_mat),
        ("Real__broomCord", &cord_mat),
    ] {
        if let Some(geo) = game::assets::props::prop_part_as_modelled(name) {
            let m = mesh(geo, mat);
            m.position().set(0, nail_y, -0.010);
            broom.add(&m);
        }
    }
    broom.rotation().y = face_wall(dx, dz);
    broom.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - 0.010), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - 0.010));
    g.world.add(&broom);
}

/// The broom's handle round its length (u round it, v up it): a hardwood
/// stick oiled long ago, its grain and a knot, gone dark at the top where it
/// was held, grimed toward the broomcorn
fn broom_handle_tex(wood: u32, vary: u32) -> Tex {
    canvas_tex(32, 128, move |ctx, w, h| {
        let seed = (rnd() * 4.0e9) as u32 ^ vary;
        with_own_stream(seed, || {
            ctx.set_fill_style(format!("#{wood:06x}"));
            ctx.fill_rect(0, 0, w, h);
            // the grain, up the stick
            for _ in 0..24 {
                let x = rnd() * w;
                ctx.set_stroke_style(format!("rgba(40,26,14,{})", 0.10 + rnd() * 0.2));
                ctx.set_line_width(0.5 + rnd());
                ctx.begin_path();
                ctx.move_to(x, 0);
                ctx.line_to(x + (rnd() - 0.5) * 3.0, h);
                ctx.stroke();
            }
            // a knot or two in it
            for _ in 0..2 {
                let (x, y) = (rnd() * w, rnd() * h);
                ctx.set_fill_style("rgba(50,32,18,0.45)");
                ctx.begin_path();
                ctx.ellipse(x, y, 1.5, 3.0, 0, 0, 7);
                ctx.fill();
            }
            // the top, where hands held it: dark and greasy
            let gr = ctx.create_linear_gradient(0, 0, 0, h * 0.45);
            gr.add_color_stop(0, "rgba(30,20,12,0.40)");
            gr.add_color_stop(1, "rgba(30,20,12,0)");
            ctx.set_fill_style(gr);
            ctx.fill_rect(0, 0, w, h * 0.45);
            // grime toward its foot
            let gr = ctx.create_linear_gradient(0, h * 0.7, 0, h);
            gr.add_color_stop(0, "rgba(30,24,18,0)");
            gr.add_color_stop(1, "rgba(30,24,18,0.35)");
            ctx.set_fill_style(gr);
            ctx.fill_rect(0, h * 0.7, w, h * 0.3);
        });
    })
}

/// The broomcorn seen from the front (u across, v up the head): straws side
/// by side in their own shades of straw, green-gold to brown, the sweep's
/// ends gone dark with what it swept, dust in it
fn broom_corn_tex(vary: u32) -> Tex {
    canvas_tex(64, 192, move |ctx, w, h| {
        let seed = (rnd() * 4.0e9) as u32 ^ vary;
        with_own_stream(seed, || {
            ctx.set_fill_style("#a48a5a");
            ctx.fill_rect(0, 0, w, h);
            for _ in 0..300 {
                let x = rnd() * w;
                let (y0, y1) = (rnd() * h * 0.5, h * (0.55 + 0.45 * rnd()));
                let k = rnd();
                let (r, gg, b) = if k < 0.55 {
                    (196.0, 170.0, 112.0)
                } else if k < 0.75 {
                    (168.0, 158.0, 98.0)
                } else if k < 0.92 {
                    (150.0, 112.0, 70.0)
                } else {
                    (112.0, 84.0, 56.0)
                };
                let s = 0.8 + 0.3 * rnd();
                ctx.set_stroke_style(format!("rgba({},{},{},{})", (r * s) as i32, (gg * s) as i32, (b * s) as i32, 0.5 + 0.5 * rnd()));
                ctx.set_line_width(0.6 + rnd() * 1.1);
                ctx.begin_path();
                ctx.move_to(x, y0);
                ctx.line_to(x + (rnd() - 0.5) * 3.0, y1);
                ctx.stroke();
            }
            // the gaps between straws
            for _ in 0..90 {
                ctx.set_stroke_style(format!("rgba(40,30,18,{})", 0.15 + rnd() * 0.25));
                ctx.set_line_width(0.5);
                let x = rnd() * w;
                ctx.begin_path();
                ctx.move_to(x, rnd() * h);
                ctx.line_to(x + (rnd() - 0.5) * 2.0, rnd() * h);
                ctx.stroke();
            }
            // the sweep's ends: dirt worked into them
            let gr = ctx.create_linear_gradient(0, h * 0.62, 0, h);
            gr.add_color_stop(0, "rgba(58,48,36,0)");
            gr.add_color_stop(0.7, "rgba(58,48,36,0.45)");
            gr.add_color_stop(1, "rgba(48,40,30,0.7)");
            ctx.set_fill_style(gr);
            ctx.fill_rect(0, h * 0.62, w, h * 0.38);
            // dust caught in it
            for _ in 0..60 {
                ctx.set_fill_style(format!("rgba(170,160,140,{})", 0.10 + rnd() * 0.15));
                ctx.fill_rect(rnd() * w, rnd() * h, 2.0 + rnd() * 6.0, 1.0 + rnd() * 3.0);
            }
        });
    })
}
