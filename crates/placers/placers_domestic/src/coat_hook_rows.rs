//! `place_coat_hook_rows`
use game::prelude::*;
use placers_kit::place::pick;
use placers_kit::place::wall_candidates;
use placers_kit::wall_plates::part;

// Coat hook rows: 20% chance per level, 1 rail of 3-5 hooks (Blender
// batch `r06`): an oak rail screwed to the wall, cast iron double coat-and-
// hat hooks along it, now and then an overcoat still hanging from one.
pub(crate) fn place_coat_hook_rows(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.80 {
        return;
    } // 20% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = pick(&candidates);
    let row = group();
    // the rail on the wall (its back 25 mm behind the row's origin)
    let oak = lambert!({ map: &rail_wood_tex() });
    let rail = mesh(part("Real__hookRail", 0.70, 0.095, 0.019), &oak);
    rail.position().z = -0.025;
    row.add(&rail);
    let steel = phong!({ color: 0x5e5c57, specular: 0x2a2a2a, shininess: 30 });
    for sx in [-0.335, 0.335] {
        let sc = mesh(part("Real__hookScrew", 0.01, 0.01, 0.002), &steel);
        sc.position().set(sx, 0, -0.025 + 0.019);
        row.add(&sc);
    }
    // 3-5 hooks along it
    let hook_count = 3 + (rnd() * 3.0).floor() as i32;
    // 15% chance one hook has a coat still hanging.
    let coat_idx = if rnd() < 0.15 { (rnd() * hook_count as f64).floor() as i32 } else { -1 };
    let coat_colors = [0x2a1810, 0x1a1a2a, 0x2a1a1a, 0x1a2a1a];
    let iron = phong!({ color: 0x1c1b1a, specular: 0x1a1a1a, shininess: 22 });
    for h in 0..hook_count {
        let x_off = -0.30 + (h as f64 / (hook_count - 1) as f64) * 0.60;
        let hook = mesh(part("Real__coatHook", 0.026, 0.13, 0.1), &iron);
        hook.position().set(x_off, 0.0, -0.025 + 0.019);
        row.add(&hook);
        if h == coat_idx {
            // the old palette's pick (the same draw): an overcoat in wool of that hue
            let old = pick(&coat_colors);
            let wool = match old {
                0x2a1810 => 0x7a6249, // camel gone brown
                0x1a1a2a => 0x47506a, // navy
                0x2a1a1a => 0x66403c, // oxblood
                _ => 0x5a5e4a,        // loden
            };
            let coat = mesh(part("Real__coatHung", 0.5, 1.0, 0.2), lambert!({ map: &wool_tex(), color: wool, side: DoubleSide }));
            // hung by its loop on the coat hook's cradle
            coat.position().set(x_off, -0.047, -0.006 + 0.058);
            row.add(&coat);
            let buttons =
                mesh(part("Real__hookCoatButtons", 0.03, 0.4, 0.01), phong!({ color: 0x2a2018, specular: 0x202020, shininess: 40 }));
            buttons.position().copy(&coat.pos());
            row.add(&buttons);
        }
    }
    let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.025);
    let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.025);
    let row_height = 1.7 + rnd() * 0.2; // 1.7-1.9 m
    if c.dx.abs() == 1 {
        row.rotation().y = if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 };
        row.position().set(wall_x, row_height, wall_z + (rnd() - 0.5) * 0.8);
    } else {
        row.rotation().y = if c.dz > 0 { PI } else { 0.0 };
        row.position().set(wall_x + (rnd() - 0.5) * 0.8, row_height, wall_z);
    }
    g.world.add(&row);
}

/// Old oak, its varnish darkened: straight grain along the rail, the rays.
fn rail_wood_tex() -> Tex {
    thread_local! { static T: std::cell::OnceCell<Tex> = const { std::cell::OnceCell::new() }; }
    T.with(|c| {
        c.get_or_init(|| {
            canvas_tex(512, 64, |ctx, w, h| {
                ctx.set_fill_style("#6a4e33");
                ctx.fill_rect(0, 0, w, h);
                for _ in 0..70 {
                    let y = rnd() * h;
                    let a = 0.08 + rnd() * 0.18;
                    ctx.set_stroke_style(format!("rgba(40,26,14,{a})"));
                    ctx.set_line_width(0.6 + rnd() * 1.6);
                    ctx.begin_path();
                    ctx.move_to(0, y);
                    let mut yy = y;
                    let mut x = 0.0;
                    while x < w {
                        x += 16.0;
                        yy += (rnd() - 0.5) * 0.9;
                        ctx.line_to(x, yy);
                    }
                    ctx.stroke();
                }
                // the rays: short pale flecks across the grain
                for _ in 0..60 {
                    ctx.set_fill_style(format!("rgba(170,130,90,{})", 0.08 + rnd() * 0.12));
                    ctx.fill_rect(rnd() * w, rnd() * h, 3.0 + rnd() * 8.0, 1.0);
                }
                // hands' grime along the middle
                let gr = ctx.create_linear_gradient(0, 0, 0, h);
                gr.add_color_stop(0, "rgba(20,14,8,0.15)");
                gr.add_color_stop(0.5, "rgba(20,14,8,0)");
                gr.add_color_stop(1, "rgba(20,14,8,0.2)");
                ctx.set_fill_style(gr);
                ctx.fill_rect(0, 0, w, h);
            })
        })
        .clone()
    })
}

/// Wool cloth: its fine twill, lint and the dust settled on it (grey, laid
/// over the coat's own colour).
fn wool_tex() -> Tex {
    thread_local! { static T: std::cell::OnceCell<Tex> = const { std::cell::OnceCell::new() }; }
    T.with(|c| {
        c.get_or_init(|| {
            canvas_tex(256, 256, |ctx, w, h| {
                ctx.set_fill_style("#f2f2f2");
                ctx.fill_rect(0, 0, w, h);
                ctx.set_line_width(1);
                let mut d = -h;
                while d < w {
                    ctx.set_stroke_style(format!("rgba(0,0,0,{})", 0.06 + rnd() * 0.06));
                    ctx.begin_path();
                    ctx.move_to(d, 0);
                    ctx.line_to(d + h, h);
                    ctx.stroke();
                    d += 3.0;
                }
                for _ in 0..500 {
                    let v = if rnd() < 0.5 { "255,255,255" } else { "0,0,0" };
                    ctx.set_fill_style(format!("rgba({v},{})", 0.05 + rnd() * 0.1));
                    ctx.fill_rect(rnd() * w, rnd() * h, 1.0 + rnd() * 2.0, 1.0 + rnd() * 2.0);
                }
                for _ in 0..30 {
                    let (x, y) = (rnd() * w, rnd() * h);
                    let r = 6.0 + rnd() * 20.0;
                    let gr = ctx.create_radial_gradient(x, y, 0, x, y, r);
                    gr.add_color_stop(0, "rgba(255,250,235,0.10)");
                    gr.add_color_stop(1, "rgba(255,250,235,0)");
                    ctx.set_fill_style(gr);
                    ctx.fill_rect(x - r, y - r, r * 2.0, r * 2.0);
                }
            })
        })
        .clone()
    })
}
