//! `place_trash_cans`
use game::prelude::*;
use placers_kit::place::open_tiles;

// Trash cans: 0-1 per level (25% chance): a round steel office
// wastebasket, its black liner turned over its rim, standing full of
// crumpled paper with a piece or two by it, or knocked over with the paper
// spilled out of its mouth; sometimes a tied bag beside it (Blender batch
// r08: the WB26 class, 343 mm across, 368 mm tall).
pub(crate) fn place_trash_cans(g: &mut Game, _dist: &[i32]) {
    use game::assets::props::prop_part_as_modelled;

    /// Its paint (`paint`, by the weather) as years leave it: mottled,
    /// scuffed and scratched to the steel, rust at its chime, its rim and
    /// down from its seam, grime low (on the can's own wrap: u round it, v
    /// up it).
    fn can_wear_tex(paint: u32) -> Tex {
        canvas_tex(256, 128, move |ctx, w, h| {
            let (r, gg, b) = ((paint >> 16) & 255, (paint >> 8) & 255, paint & 255);
            ctx.set_fill_style(format!("rgb({r},{gg},{b})"));
            ctx.fill_rect(0, 0, w, h);
            for _ in 0..220 {
                let k = 0.9 + rnd() * 0.2;
                ctx.set_fill_style(format!("rgba({:.0},{:.0},{:.0},0.25)", r as f64 * k, gg as f64 * k, b as f64 * k));
                ctx.fill_rect(rnd() * w, rnd() * h, 4.0 + rnd() * 18.0, 2.0 + rnd() * 8.0);
            }
            // grime toward the floor it stood on
            let gr = ctx.create_linear_gradient(0, h, 0, h * 0.6);
            gr.add_color_stop(0, "rgba(40,32,22,0.45)");
            gr.add_color_stop(1, "rgba(40,32,22,0)");
            ctx.set_fill_style(&gr);
            ctx.fill_rect(0, h * 0.6, w, h * 0.4);
            // scratches to the steel and scuffs, short, most of them low
            // where shoes and the cleaner's trolley caught it
            ctx.set_line_width(0.6);
            for _ in 0..60 {
                let (x, y) = (rnd() * w, h * (0.35 + 0.65 * rnd().sqrt()));
                let a = (rnd() - 0.5) * 0.5;
                let l = 2.0 + rnd() * rnd() * 14.0;
                ctx.set_stroke_style(format!("rgba(150,148,140,{:.2})", 0.15 + rnd() * 0.3));
                ctx.begin_path();
                ctx.move_to(x, y);
                ctx.line_to(x + a.dcos() * l, y + a.dsin() * l);
                ctx.stroke();
            }
            // rust: at the chime and the rim where the paint chipped, down from the seam
            let rust = |ctx: &mut Canvas, x: f64, y: f64, s: f64| {
                ctx.set_fill_style(format!(
                    "rgba({:.0},{:.0},{:.0},{:.2})",
                    96.0 + rnd() * 30.0,
                    52.0 + rnd() * 14.0,
                    26.0,
                    0.35 + rnd() * 0.4
                ));
                ctx.begin_path();
                ctx.ellipse(x, y, s, s * (0.5 + rnd()), rnd() * 3.0, 0, 7);
                ctx.fill();
            };
            for _ in 0..70 {
                rust(ctx, rnd() * w, h - rnd() * rnd() * 16.0, 1.0 + rnd() * 3.5);
                rust(ctx, rnd() * w, rnd() * rnd() * 7.0, 0.8 + rnd() * 2.5);
            }
            let seam = w * 0.618;
            for k in 0..14 {
                let y = 4.0 + rnd() * (h - 10.0);
                let l = 6.0 + rnd() * 26.0;
                ctx.set_fill_style(format!("rgba(104,58,30,{:.2})", 0.15 + rnd() * 0.25));
                ctx.fill_rect(seam - 3.0 + rnd() * 6.0 + (k % 2) as f64 * 2.0, y, 1.0 + rnd() * 2.0, l);
            }
        })
    }

    /// The paper spilled from it: white office paper, fragments of print.
    fn litter_tex() -> Tex {
        canvas_tex(128, 128, |ctx, w, h| {
            ctx.set_fill_style("#e2ded2");
            ctx.fill_rect(0, 0, w, h);
            for _ in 0..12 {
                let (cx, cy, a) = (rnd() * w, rnd() * h, rnd() * PI);
                ctx.save();
                ctx.translate(cx, cy);
                ctx.rotate(a);
                for k in 0..6 {
                    let mut x = -24.0 + rnd() * 4.0;
                    while x < 22.0 {
                        let l = 3.0 + rnd() * 9.0;
                        ctx.set_fill_style(format!("rgba(40,38,34,{:.2})", 0.18 + rnd() * 0.22));
                        ctx.fill_rect(x, -16.0 + k as f64 * 6.0, l, 2.0);
                        x += l + 2.0;
                    }
                }
                ctx.restore();
            }
            ctx.set_line_width(0.7);
            for _ in 0..40 {
                let (x, y, a) = (rnd() * w, rnd() * h, rnd() * PI);
                let l = 6.0 + rnd() * 24.0;
                ctx.set_stroke_style(format!("rgba(70,62,50,{:.2})", 0.12 + rnd() * 0.2));
                ctx.begin_path();
                ctx.move_to(x, y);
                ctx.line_to(x + a.dcos() * l, y + a.dsin() * l);
                ctx.stroke();
            }
        })
    }

    if rnd() < 0.75 {
        return;
    } // 25% chance per level
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let pk = open_tiles[(rnd() * open_tiles.len() as f64).floor() as usize];
    let can_color = vent_color_by_weather(g.weather_key());
    let paint = phong!({ map: &can_wear_tex(can_color), color: 0xffffff, specular: 0x262626, shininess: 26 });
    let film_color = trash_bag_color_by_weather(g.weather_key());
    let liner_mat = phong!({ color: film_color, specular: 0x4a4a4a, shininess: 24, side: DoubleSide });
    let paper_mat = lambert!({ map: &litter_tex(), color: 0xffffff });
    let can_group = group();
    // the paper that was in it (2-3 pieces): a can with two left by it stands,
    // full; with three it was knocked over and they spilled out of its mouth
    let spill_count = 2 + (rnd() * 2.0).floor() as i32;
    let tipped = spill_count == 3;
    if let Some(body) = prop_part_as_modelled("Real__wastebasket") {
        let bb = body.bounding_box();
        let (lift, len) = (bb.max.x, bb.max.y);
        let mut parts = vec![(Some(body), &paint), (prop_part_as_modelled("Real__wastebasketLiner"), &liner_mat)];
        if !tipped {
            parts.push((prop_part_as_modelled("Real__wastebasketTrash"), &paper_mat));
        }
        for (geo, mat) in parts {
            let Some(geo) = geo else { continue };
            let m = mesh(geo, mat);
            if tipped {
                // built standing, laid on its side with its mouth toward +x
                m.rotation().z = -PI / 2.0;
                m.position().set(-len / 2.0, lift, 0);
            }
            can_group.add(&m);
        }
    }
    for _ in 0..spill_count {
        // the old litter's sizes: which piece (w), how big (d)
        let bw = 0.06 + rnd() * 0.04;
        let _bh = rnd(); // keeps the old draw (its height)
        let bd = 0.06 + rnd() * 0.04;
        let u = (bw - 0.06) / 0.04;
        let (name, sheet) = if u < 0.45 {
            ("Real__paperBall", false)
        } else if u < 0.82 {
            ("Real__paperWad", false)
        } else {
            ("Real__paperSheet", true)
        };
        let px = 0.30 + rnd() * 0.20;
        let _py = rnd(); // keeps the old draw (its lift)
        let pz = (rnd() - 0.5) * 0.20;
        let _rx = rnd(); // keeps the old draw (its tilt)
        let ry = rnd() * PI * 2.0;
        let _rz = rnd(); // keeps the old draw (its tilt)
        let Some(geo) = prop_part_as_modelled(name) else { continue };
        let sc = mesh(geo, &paper_mat);
        let k = if sheet { 1.0 } else { 0.9 + (bd - 0.06) / 0.04 * 0.2 };
        sc.scale().set(k, k, k);
        // spilled 5 to 25 cm out of its mouth, or thrown at it and missed
        let out = if tipped { px - 0.07 } else { px - 0.02 };
        sc.position().set(out + if sheet { 0.10 } else { 0.0 }, 0, pz);
        sc.rotation().y = ry;
        can_group.add(&sc);
    }
    let cx = g.wx(pk.0) + (rnd() - 0.5) * 1.4;
    let cz = g.wz(pk.1) + (rnd() - 0.5) * 1.4;
    can_group.position().set(cx, 0, cz);
    can_group.rotation().y = rnd() * PI * 2.0;
    g.world.add(&can_group);
    // 30% chance a tied bag beside it
    if rnd() < 0.30 {
        // Per-weather bag film
        let bag_color = trash_bag_color_by_weather(g.weather_key());
        let bag_mat = phong!({ color: bag_color, specular: 0x4a4a4a, shininess: 24 });
        let bag = group();
        if let Some(geo) = prop_part_as_modelled("Real__trashBag") {
            bag.add(&mesh(geo, &bag_mat));
        }
        // the old side lump's turn (keeps the old draws)
        let _rx = rnd();
        let _ry = rnd();
        let _rz = rnd();
        // 0.3-0.7 m from the can (the placement pass keeps it clear of it)
        let b_ang = rnd() * PI * 2.0;
        let b_dist = 0.30 + rnd() * 0.40;
        let cp = can_group.pos();
        bag.position().set(cp.x + b_ang.dcos() * b_dist, 0, cp.z + b_ang.dsin() * b_dist);
        bag.rotation().y = rnd() * PI * 2.0;
        g.world.add(&bag);
    }
}

/// Trash bag colour per weather (else the normal entry)
fn trash_bag_color_by_weather(key: &str) -> u32 {
    by_weather(key, 0x0a0f14, 0x281a0a, 0x1a0808, 0x2a2418, 0x0a0a10, 0x1a1a1e)
}
