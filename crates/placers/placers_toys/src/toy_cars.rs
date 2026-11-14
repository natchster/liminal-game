//! `place_toy_cars`: Small toy car: 10% chance per level. A 1:64 die-cast saloon of the 1970s (Blender batch r17), 76 mm long: its zinc body in chipped enamel, clear glazing gone grey, silver-painted bumpers, grille and lamps, thin plastic wheels on steel axles, a black baseplate. A quarter lie on their side.
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;

/// Small toy car: 10% chance per level. A 1:64 die-cast saloon of the
/// 1970s (Blender batch r17), 76 mm long: its zinc body in chipped enamel,
/// clear glazing gone grey, silver-painted bumpers, grille and lamps, thin
/// plastic wheels on steel axles, a black baseplate. A quarter lie on
/// their side.
pub(crate) fn place_toy_cars(g: &mut Game, _dist: &[i32]) {
    use game::assets::props::prop_part_as_modelled;
    if rnd() < 0.90 {
        return;
    } // 10% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    // the old colours stand for the enamels die-cast cars came in, faded
    let body_colors = [0xd82020, 0x2050d8, 0xd8b820, 0x30a030, 0xd0d0d0, 0x181818];
    let col = pick(&body_colors);
    let k = body_colors.iter().position(|&c| c == col).unwrap_or(0);
    let paint = TOY_CAR_PAINT[k];
    let car = group();
    // the car in its own frame (x across, y up, z out of its nose)
    let body = group();
    let mut half_w: f64 = 0.015;
    let parts: [(&str, Mat); 6] = [
        ("Real__toyCarBody", phong!({ color: 0xffffff, map: &toy_car_paint_tex(paint), specular: 0x2c2c2c, shininess: 36 })),
        ("Real__toyCarGlass", phong!({ color: 0x23272a, specular: 0x4a4a4a, shininess: 70 })),
        ("Real__toyCarBright", phong!({ color: 0x7e7d78, specular: 0x5a5a5a, shininess: 48 })),
        ("Real__toyCarTail", lambert!({ color: 0x6e2a24 })),
        ("Real__toyCarBase", lambert!({ color: 0x24221f })),
        ("Real__toyCarTyres", lambert!({ color: 0x1f1e1c })),
    ];
    for (name, mat) in parts {
        if let Some(geo) = prop_part_as_modelled(name) {
            let bb = geo.bounding_box();
            half_w = half_w.max(bb.max.x.abs()).max(bb.min.x.abs());
            body.add(&mesh(geo, mat));
        }
    }
    car.add(&body);
    car.position().set(g.wx(x) + (rnd() - 0.5) * 2.4, 0, g.wz(z) + (rnd() - 0.5) * 2.4);
    car.rotation().y = rnd() * PI * 2.0;
    // 25% chance: car on its side (rolled about its length), resting on
    // its flank.
    if rnd() < 0.25 {
        body.rotation().z = PI / 2.0 * (if rnd() < 0.5 { -1.0 } else { 1.0 });
        body.position().y = half_w;
    }
    g.world.add(&car);
}

/// The toy car's enamels (sRGB): red, blue, mustard, green, silver-grey,
/// black, as years and the light left them.
const TOY_CAR_PAINT: [u32; 6] = [0x8c2f28, 0x2f4878, 0xb08a32, 0x34583a, 0x8e908c, 0x2a2a2a];

/// The body's enamel (sRGB `paint`): a little uneven, dulled, chipped back
/// to the grey zinc in flecks and a few bigger chips (each with its dark
/// edge), fine scratches through it, dust in its hollows
fn toy_car_paint_tex(paint: u32) -> Tex {
    canvas_tex(128, 128, move |ctx, w, h| {
        let (r, g, b) = ((paint >> 16) & 255, (paint >> 8) & 255, paint & 255);
        ctx.set_fill_style(format!("rgb({r},{g},{b})"));
        ctx.fill_rect(0, 0, w, h);
        // dulled unevenly: lighter and darker washes
        for _ in 0..16 {
            let (x, y) = (rnd() * w, rnd() * h);
            let rr = 10.0 + rnd() * 26.0;
            let gr = ctx.create_radial_gradient(x, y, 0, x, y, rr);
            gr.add_color_stop(0, if rnd() < 0.5 { "rgba(200,196,186,0.14)" } else { "rgba(30,26,22,0.14)" });
            gr.add_color_stop(1, "rgba(0,0,0,0)");
            ctx.set_fill_style(gr);
            ctx.fill_rect(x - rr, y - rr, rr * 2.0, rr * 2.0);
        }
        // chips back to the zinc: a few bigger ones, ragged, with a dark
        // edge where the enamel broke; many tiny flecks
        for i in 0..22 {
            let (x, y) = (rnd() * w, rnd() * h);
            let big = i < 4;
            let r0 = if big { 1.6 + rnd() * 2.2 } else { 0.4 + rnd() * 0.6 };
            let n = 7;
            let pts: Vec<(f64, f64)> = (0..n)
                .map(|k| {
                    let a = k as f64 / n as f64 * PI * 2.0;
                    let rr = r0 * (0.5 + rnd() * 0.7);
                    (x + a.cos() * rr * 1.4, y + a.sin() * rr)
                })
                .collect();
            for (grow, style) in [(0.6, "rgba(34,32,30,0.45)"), (0.0, "rgba(146,146,140,0.9)")] {
                ctx.set_fill_style(style);
                ctx.begin_path();
                for (k, &(px, py)) in pts.iter().enumerate() {
                    let (dx, dy) = (px - x, py - y);
                    let d = (dx * dx + dy * dy).sqrt().max(1e-6);
                    let (qx, qy) = (px + dx / d * grow, py + dy / d * grow);
                    if k == 0 {
                        ctx.move_to(qx, qy);
                    } else {
                        ctx.line_to(qx, qy);
                    }
                }
                ctx.close_path();
                ctx.fill();
            }
        }
        // fine scratches, light
        ctx.set_line_width(0.6);
        for _ in 0..22 {
            let (x, y) = (rnd() * w, rnd() * h);
            let a = rnd() * PI;
            let l = 4.0 + rnd() * 14.0;
            ctx.set_stroke_style(format!("rgba(180,178,170,{:.2})", 0.25 + rnd() * 0.35));
            ctx.begin_path();
            ctx.move_to(x, y);
            ctx.line_to(x + a.cos() * l, y + a.sin() * l);
            ctx.stroke();
        }
        // dust and dirt specks
        for _ in 0..70 {
            ctx.set_fill_style(format!("rgba(96,88,74,{:.2})", 0.2 + rnd() * 0.3));
            ctx.fill_rect(rnd() * w, rnd() * h, 1, 1);
        }
    })
}
