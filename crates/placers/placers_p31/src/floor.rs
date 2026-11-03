//! Floor / ceiling props of this crate.
use crate::{open_tiles, pick, shuffle, trash_bag_color_by_weather};
use game::prelude::*;

// R265 clipboards: 0-2 dropped clipboards per level (30% chance).
pub fn place_clipboards(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.70 {
        return;
    } // 30% chance
    let count = 1 + (rnd() * 2.0).floor() as usize;
    let mut open_tiles = open_tiles(g);
    shuffle(&mut open_tiles);
    let board_mat = lambert!({ color: 0x9a7048 });
    let paper_mat = lambert!({ color: 0xece0d0 });
    let clip_mat = lambert!({ color: 0x606060 });
    let mut i = 0;
    while i < count && i < open_tiles.len() {
        let (x, z) = open_tiles[i];
        let clipboard = group();
        // Board: 0.20 × 0.28 × 0.008 flat rectangle.
        let board = mesh(pg("Clipboards__board", box_geo!(0.20, 0.008, 0.28)), &board_mat);
        board.position().y = 0.004;
        clipboard.add(&board);
        // Paper on top (slightly smaller).
        let paper = mesh(pg("Clipboards__paper", plane_geo!(0.18, 0.24)), &paper_mat);
        paper.rotation().x = -PI / 2.0;
        paper.position().y = 0.009;
        clipboard.add(&paper);
        // Wire clip at top edge.
        let clip = mesh(pg("Clipboards__clip", box_geo!(0.06, 0.012, 0.02)), &clip_mat);
        clip.position().set(0, 0.014, -0.11);
        clipboard.add(&clip);
        let ox = (rnd() - 0.5) * 2.4;
        let oz = (rnd() - 0.5) * 2.4;
        clipboard.position().x = g.wx(x) + ox;
        clipboard.position().z = g.wz(z) + oz;
        clipboard.rotation().y = rnd() * PI * 2.0;
        g.world.add(&clipboard);
        // R266 30% chance a pen is dropped next to the clipboard.
        if rnd() < 0.30 {
            let pen_mat = lambert!({ color: 0x1a1a1a });
            let pen = mesh(pg("Clipboards__pen", cyl_geo!(0.004, 0.004, 0.11, 6)), &pen_mat);
            pen.rotation().z = PI / 2.0;
            pen.rotation().y = rnd() * PI * 2.0;
            // Position within 0.20 m of clipboard.
            let p_dx = (rnd() - 0.5) * 0.4;
            let p_dz = (rnd() - 0.5) * 0.4;
            let cp = clipboard.pos();
            pen.position().set(cp.x + p_dx, 0.004, cp.z + p_dz);
            g.world.add(&pen);
        }
        i += 1;
    }
}

// R263 cardboard boxes: 0-3 boxes per level (30% chance).
pub fn place_cardboard_boxes(g: &mut Game, _dist: &[i32]) {
    g.pp.cardboard_box_positions.clear(); // R287 reset per level
    if rnd() < 0.70 {
        return;
    } // 30% chance
    let count = 1 + (rnd() * 3.0).floor() as usize;
    let mut open_tiles = open_tiles(g);
    shuffle(&mut open_tiles);
    let box_mat = lambert!({ color: 0x9a7248 });
    let tape_mat = lambert!({ color: 0xd8ccb0 });
    let mut i = 0;
    while i < count && i < open_tiles.len() {
        let (x, z) = open_tiles[i];
        // Box dimensions vary — small shipping box scale.
        let sx = 0.30 + rnd() * 0.20;
        let sy = 0.22 + rnd() * 0.15;
        let sz = 0.30 + rnd() * 0.20;
        let bx = group();
        let body = mesh(pg("CardboardBoxes__body", box_geo!(sx, sy, sz)), &box_mat);
        body.position().y = sy / 2.0;
        bx.add(&body);
        // Packing tape stripe across top (30% chance).
        if rnd() < 0.30 {
            let tape = mesh(plane_geo!(sx * 0.9, 0.04), &tape_mat);
            tape.rotation().x = -PI / 2.0;
            tape.position().y = sy + 0.001;
            bx.add(&tape);
        }
        // 25% chance tipped on side.
        if rnd() < 0.25 {
            bx.rotation().z = PI / 2.0 * (if rnd() < 0.5 { 1.0 } else { -1.0 });
            bx.position().y = sx / 2.0; // approximate rest height
        }
        let ox = (rnd() - 0.5) * 2.2;
        let oz = (rnd() - 0.5) * 2.2;
        bx.position().x = g.wx(x) + ox;
        bx.position().z = g.wz(z) + oz;
        bx.rotation().y = rnd() * PI * 2.0;
        g.world.add(&bx);
        // R287 track for shifting-audio events.
        let p = bx.pos();
        g.pp.cardboard_box_positions.push(v3(p.x, sy / 2.0, p.z));
        i += 1;
    }
}

// R261 coffee cups: 0-3 small paper cups per level (30% chance).
pub fn place_coffee_cups(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.70 {
        return;
    } // 30% chance
    let count = 1 + (rnd() * 3.0).floor() as usize;
    let mut open_tiles = open_tiles(g);
    shuffle(&mut open_tiles);
    // Cup colors: white paper, brown paper, styrofoam, disposable red.
    let cup_colors = [0xece0d0, 0x8a6a48, 0xf0eee4, 0xa03018];
    let mut i = 0;
    while i < count && i < open_tiles.len() {
        let (x, z) = open_tiles[i];
        let cup = group();
        let cup_color = pick(&cup_colors);
        let cup_mat = lambert!({ color: cup_color });
        // Tapered cylinder (top-heavy for takeaway coffee cup look).
        let body = mesh(pg("CoffeeCups__cup", cyl_geo!(0.038, 0.028, 0.09, 10)), &cup_mat);
        body.position().y = 0.045;
        cup.add(&body);
        // Coffee inside (dark disc at top when standing).
        let coffee = mesh(circle_geo!(0.032, 10), lambert!({ color: 0x2a1810 }));
        coffee.rotation().x = -PI / 2.0;
        coffee.position().y = 0.091;
        cup.add(&coffee);
        // 40% chance the cup is knocked over.
        let knocked_over = rnd() < 0.40;
        if knocked_over {
            cup.rotation().z = PI / 2.0;
            cup.position().y = 0.038;
        }
        let ox = (rnd() - 0.5) * 2.6;
        let oz = (rnd() - 0.5) * 2.6;
        cup.position().x = g.wx(x) + ox;
        cup.position().z = g.wz(z) + oz;
        cup.rotation().y = rnd() * PI * 2.0;
        g.world.add(&cup);
        // R262 coffee spill: knocked-over cups 60% chance spill brown stain.
        if knocked_over && rnd() < 0.60 {
            let r = 0.10 + rnd() * 0.06;
            let spill_geo = circle_geo!(r, 8);
            spill_geo.scale(1.6, 0.7, 1); // stretched away from cup mouth
            let spill_mat = basic_mat!({ color: 0x2a1810, transparent: true, opacity: 0.65, depthWrite: false });
            let spill = mesh(spill_geo, &spill_mat);
            let cup_ry = cup.rot().y;
            spill.rotation().x = -PI / 2.0;
            spill.rotation().z = cup_ry; // aligned with cup direction
                                         // Position stretched in cup's forward direction.
            let fx = cup_ry.sin();
            let fz = cup_ry.cos();
            let cp = cup.pos();
            spill.position().set(cp.x + fx * 0.10, 0.005, cp.z + fz * 0.10);
            g.world.add(&spill);
        }
        i += 1;
    }
}

// R257 trash cans: 0-1 per level (25% chance): a round steel office
// wastebasket, its black liner turned over its rim, standing full of
// crumpled paper with a piece or two by it, or knocked over with the paper
// spilled out of its mouth; sometimes a tied bag beside it (Blender batch
// r08: the WB26 class, 343 mm across, 368 mm tall).
pub fn place_trash_cans(g: &mut Game, _dist: &[i32]) {
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
                ctx.line_to(x + a.cos() * l, y + a.sin() * l);
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
                ctx.line_to(x + a.cos() * l, y + a.sin() * l);
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
    // R296 30% chance a tied bag beside it
    if rnd() < 0.30 {
        // R306 per-weather bag film
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
        bag.position().set(cp.x + b_ang.cos() * b_dist, 0, cp.z + b_ang.sin() * b_dist);
        bag.rotation().y = rnd() * PI * 2.0;
        g.world.add(&bag);
    }
}

// R257 motion sensors: 30% chance per level, 1-2 ceiling-mounted domes with LED.
pub fn place_motion_sensors(g: &mut Game, _dist: &[i32]) {
    g.pp.motion_sensor_positions.clear(); // R273 reset per level
    if rnd() < 0.70 {
        return;
    } // 30% chance
    let count = 1 + (rnd() * 2.0).floor() as usize;
    let mut open_tiles = open_tiles(g);
    shuffle(&mut open_tiles);
    let dome_mat = lambert!({ color: 0xe0d8c8 });
    // Per-LED material created inline for R258 blink animation.
    let mut i = 0;
    while i < count && i < open_tiles.len() {
        let (x, z) = open_tiles[i];
        let sensor = group();
        // Base disc (fits flush against ceiling).
        let base = mesh(pg("MotionSensors__base", cyl_geo!(0.08, 0.08, 0.02, 10)), &dome_mat);
        base.position().y = 0.0;
        sensor.add(&base);
        // Dome (hemisphere).
        let dome = mesh(pg("MotionSensors__lens", sphere_geo!(0.06, 10, 6, 0, PI * 2.0, 0, PI / 2.0)), &dome_mat);
        dome.position().y = -0.01;
        dome.rotation().x = PI; // flip so it points down
        sensor.add(&dome);
        // R258/R259 blinking LED, colour per weather.
        let led_color = led_color_by_weather(g.weather_key());
        let led_mat_own = basic_mat!({ color: led_color, transparent: true, opacity: 1 });
        let led = mesh(box_geo!(0.008, 0.008, 0.008), &led_mat_own);
        led.position().set(0.04, -0.06, 0);
        sensor.add(&led);
        let blink_phase = rnd() * PI * 2.0;
        g.animate(move |_g, _dt, t| {
            // 2 s period sin wave: opacity = 0.7 + 0.3 * sin.
            led_mat_own.m().opacity = 0.7 + 0.3 * (t * PI + blink_phase).sin();
        });
        let ox = (rnd() - 0.5) * 1.4;
        let oz = (rnd() - 0.5) * 1.4;
        sensor.position().set(g.wx(x) + ox, WALL_H - 0.01, g.wz(z) + oz); // base top on the ceiling
        g.world.add(&sensor);
        // R273 track for proximity-click audio events.
        let p = sensor.pos();
        g.pp.motion_sensor_positions.push(v3(p.x, p.y, p.z));
        i += 1;
    }
}

// R254 ceiling water stains: 0-3 a level, where water came through the
// ceiling and dried (`assets/src/blender/props/r07.notes.md`). On a suspended
// ceiling, a ring stain on the tile it dripped onto: brown-yellow tide lines,
// one for each time it was wet and dried, sharp and wavy round a paler middle;
// smaller ones on the tiles round it where it ran over, each tile's own and
// stopped at the grid, whose bars it rusted. On a concrete slab a dark damp
// ring with white salts along its rims; on plaster brown rings. Painted on
// transparent planes just under the ceiling (the helpers are inside, so the
// rest of this file is untouched).
pub fn place_ceiling_stains(g: &mut Game, _dist: &[i32]) {
    /// A hash of (seed, i, j) in [0, 1)
    fn hash01(seed: u32, i: i32, j: i32) -> f64 {
        let mut h = seed ^ (i as u32).wrapping_mul(0x27d4_eb2d) ^ (j as u32).wrapping_mul(0x1656_67b1);
        h = (h ^ (h >> 15)).wrapping_mul(0x85eb_ca6b);
        h = (h ^ (h >> 13)).wrapping_mul(0xc2b2_ae35);
        h ^= h >> 16;
        f64::from(h) / 4_294_967_296.0
    }
    /// Smooth value noise in [0, 1]
    fn vnoise(seed: u32, x: f64, y: f64) -> f64 {
        let (xf, yf) = (x.floor(), y.floor());
        let (i, j) = (xf as i32, yf as i32);
        let (u, v) = (x - xf, y - yf);
        let (u, v) = (u * u * (3.0 - 2.0 * u), v * v * (3.0 - 2.0 * v));
        let (a, b) = (hash01(seed, i, j), hash01(seed, i + 1, j));
        let (c, d) = (hash01(seed, i, j + 1), hash01(seed, i + 1, j + 1));
        a + (b - a) * u + (c - a) * v + (a - b - c + d) * u * v
    }
    /// Fractal value noise of `oct` octaves, in [0, 1]
    fn fbm(seed: u32, x: f64, y: f64, oct: u32) -> f64 {
        let (mut s, mut amp, mut f, mut n) = (0.0, 0.5, 1.0, 0.0);
        for k in 0..oct {
            s += amp * vnoise(seed.wrapping_add(k * 7919), x * f, y * f);
            n += amp;
            amp *= 0.5;
            f *= 2.03;
        }
        s / n
    }
    /// A dried rim at signed distance `d` (m, < 0 inside): sharp outside
    /// (over `aa`), what the water left fading inward over `ew`
    fn tide(d: f64, aa: f64, ew: f64) -> f64 {
        (0.5 - d / aa).clamp(0.0, 1.0) * (d.min(0.0) / ew).exp()
    }
    /// `c` at alpha `a` over straight colour and alpha `dst`
    fn lay(dst: &mut [f64; 4], c: [f64; 3], a: f64) {
        let a = a.clamp(0.0, 1.0);
        let out = a + dst[3] * (1.0 - a);
        if out > 1e-6 {
            for k in 0..3 {
                dst[k] = (c[k] * a + dst[k] * dst[3] * (1.0 - a)) / out;
            }
        }
        dst[3] = out;
    }
    /// One stain on a piece of ceiling: its middle (world x, z), its size
    /// (m, along its long axis), how many rims, how strong
    #[derive(Clone, Copy)]
    struct Spot {
        x: f64,
        z: f64,
        r: f64,
        rims: usize,
        k: f64,
    }
    /// The material of the stain(s) `spots` on a square of ceiling `side` m
    /// round world (cx, cz), seen from below (a plane turned to face down,
    /// x along world x, its top along world +z), of `kind` (0 tile, 1 slab,
    /// 2 plaster): each an ellipse `ax` (its axes, the long one 1) turned by
    /// `turn`, its rims lobed and wavy. On a tile only its face inside the
    /// bars' flanges `hs` is stained, the bars rusted near `leak`.
    #[allow(clippy::too_many_arguments)]
    fn stain_mat(
        side: f64,
        px: usize,
        (cx, cz): (f64, f64),
        spots: Vec<Spot>,
        (ax, turn): ((f64, f64), f64),
        kind: u8,
        leak: (f64, f64, f64),
        seed: u32,
    ) -> Mat {
        let (line, fill, salt) = match kind {
            1 => ([0.15, 0.145, 0.13], [0.23, 0.22, 0.2], true),
            _ => ([0.38, 0.26, 0.12], [0.6, 0.45, 0.23], false),
        };
        let hs = if kind == 0 { side / 2.0 - 0.0125 } else { f64::MAX };
        let tex = canvas_tex_opt(px as f64, px as f64, false, move |ctx, cw, _| {
            let s = side / cw;
            let aa = s * 1.1;
            // each spot's rims: how far in, how strong, and how far out it
            // reached each way (256 headings: lobed, wavy)
            let rims: Vec<Vec<(f64, Vec<f64>)>> = spots
                .iter()
                .map(|sp| {
                    (0..sp.rims)
                        .map(|k| {
                            let (o, st) = if k == 0 {
                                (0.0, 1.0)
                            } else {
                                (0.1 + 0.62 * rnd() * (k as f64 / sp.rims as f64).sqrt(), 0.45 + 0.45 * rnd())
                            };
                            let rs = (rnd() * 4.0e9) as u32;
                            let reach = (0..=256)
                                .map(|a| {
                                    let (dv, du) = (a as f64 / 256.0 * PI * 2.0).sin_cos();
                                    let lobe = 1.0
                                        + 0.42 * (fbm(rs, du * 1.6 + 9.0, dv * 1.6 + 9.0, 3) - 0.5)
                                        + 0.12 * (vnoise(rs ^ 0x33, du * 5.0 + 3.0, dv * 5.0 + 3.0) - 0.5)
                                        + 0.04 * (vnoise(rs ^ 0x34, du * 14.0 + 5.0, dv * 14.0 + 5.0) - 0.5);
                                    (1.0 - o) * lobe
                                })
                                .collect();
                            (st, reach)
                        })
                        .collect()
                })
                .collect();
            let (sn, cs) = turn.sin_cos();
            let mut img = ctx.create_image_data(cw, cw);
            let n = cw as usize;
            for j in 0..n {
                for i in 0..n {
                    let (lx, lz) = ((i as f64 + 0.5) * s - side / 2.0, side / 2.0 - (j as f64 + 0.5) * s);
                    let (wx, wz) = (cx + lx, cz + lz);
                    let mut px = [line[0], line[1], line[2], 0.0];
                    if lx.abs() > hs || lz.abs() > hs {
                        // the bar: rust where the leak ran along it
                        let dl = (wx - leak.0).hypot(wz - leak.1);
                        if dl < leak.2 {
                            let r = (vnoise(seed ^ 0x2a7, wx * 90.0, wz * 90.0) - 0.55 + 0.3 * (1.0 - dl / leak.2)).max(0.0) * 2.0;
                            lay(&mut px, [0.4, 0.26, 0.15], r.min(0.6));
                        }
                    } else {
                        let fine = (vnoise(seed ^ 0x51, wx * 220.0, wz * 220.0) - 0.5) * 0.0016
                            + (fbm(seed ^ 0x77, wx * 30.0, wz * 30.0, 2) - 0.5) * 0.006;
                        for (sp, rim) in spots.iter().zip(&rims) {
                            let (dx, dz) = (wx - sp.x, wz - sp.z);
                            let (u, v) = ((dx * cs + dz * sn) / ax.0, (-dx * sn + dz * cs) / ax.1);
                            let rho = u.hypot(v) / sp.r;
                            if rho > 1.4 {
                                continue;
                            }
                            let a = (v.atan2(u) / (PI * 2.0)).rem_euclid(1.0) * 256.0;
                            let (a0, t) = (a.floor() as usize, a.fract());
                            let (mut d0, mut lines) = (0.0, 0.0);
                            for (k, (st, reach)) in rim.iter().enumerate() {
                                let st = *st;
                                let edge = reach[a0] + (reach[a0 + 1] - reach[a0]) * t;
                                let d = (rho - edge) * sp.r + fine;
                                if k == 0 {
                                    d0 = d;
                                }
                                lines = 1.0 - (1.0 - lines) * (1.0 - tide(d, aa, 0.005 + 0.006 * st) * st);
                            }
                            // the middle paler, what it carried left at the rims
                            let inside = (0.5 - d0 / aa).clamp(0.0, 1.0);
                            let pale = 0.55 + 0.45 * (d0 / 0.05).exp();
                            let mott = 0.65 + 0.35 * vnoise(seed ^ 0x3c, wx * 40.0, wz * 40.0);
                            lay(&mut px, fill, inside * if salt { 0.45 } else { 0.68 } * sp.k * pale * mott);
                            lay(&mut px, line, lines * 0.9 * sp.k);
                            if salt {
                                // the salts it left: a soft white bloom along
                                // its rims, in patches, grained
                                let bloom = (fbm(seed ^ 0x98, wx * 18.0, wz * 18.0, 3) - 0.42).max(0.0) * 3.0;
                                let grain = 0.6 + 0.4 * vnoise(seed ^ 0x99, wx * 300.0, wz * 300.0);
                                lay(&mut px, [0.78, 0.77, 0.72], (lines * bloom * grain).min(0.6));
                            }
                        }
                    }
                    let k = (j * n + i) * 4;
                    for (c, v) in px.iter().enumerate() {
                        img.data[k + c] = (v.clamp(0.0, 1.0) * 255.0).round() as u8;
                    }
                }
            }
            ctx.put_image_data(&img, 0, 0);
        });
        // where it is clear, the rims' colour, so its mips keep it at the edges
        {
            let img = tex.image();
            let mut im = img.borrow_mut();
            for p in im.data.chunks_exact_mut(4) {
                let t = (f64::from(p[3]) / 255.0 / 0.3).min(1.0);
                for (c, b) in p.iter_mut().zip(line) {
                    *c = (b * 255.0 + (f64::from(*c) - b * 255.0) * t * t * (3.0 - 2.0 * t)).round().clamp(0.0, 255.0) as u8;
                }
            }
            im.version += 1;
        }
        lambert!({ map: &tex, transparent: true, depthWrite: false, polygonOffset: true, polygonOffsetFactor: -2, polygonOffsetUnits: -2 })
    }

    if rnd() < 0.50 {
        return;
    } // 50% chance
    let spec = floor_stain_by_weather(g.weather_key());
    let count = 1 + (rnd() * 3.0).floor() as usize;
    let mut open_tiles = open_tiles(g);
    shuffle(&mut open_tiles);
    let mut i = 0;
    while i < count && i < open_tiles.len() {
        let (x, z) = open_tiles[i];
        // the old stain's draws: its radius (0.5-1.2 m, now how far the water
        // spread), an ellipse or not (1.4 × 0.8 of it), its opacity (now how
        // much it left), its turn, and where in the tile
        let radius = 0.5 + rnd() * 0.7;
        let ax = if rnd() < 0.4 { (1.0, 0.8 / 1.4) } else { (1.0, 1.0) };
        let strength = (0.75f64.min(spec.opacity_base + 0.15) * (0.7 + rnd() * 0.5) / 0.55).clamp(0.6, 1.3);
        let turn = rnd() * PI * 2.0;
        let ox = (rnd() - 0.5) * 1.8;
        let oz = (rnd() - 0.5) * 1.8;
        let (lx, lz) = (g.wx(x) + ox, g.wz(z) + oz);
        let seed = (hash01(0x5eed, (lx * 1000.0) as i32, (lz * 1000.0) as i32) * 4.0e9) as u32;
        let kind = if g.region_flags_at(x, z) & game::visuals::lighting::RF_MOSS != 0 {
            0
        } else {
            match g.zone_ref(x, z).map(|zn| zn.ceil_mat) {
                Some("slab") => 1,
                Some("plaster") => 2,
                _ => 0,
            }
        };
        let stain = group();
        if kind == 0 {
            // the grid (`visuals::textures::CEIL_CELLS`: 6 tiles a 4 m tile,
            // the bars through its edge along x and half a tile off it along
            // z): the tile it dripped onto (the next one along z if a fitting
            // fills it), and the tiles round it it ran onto
            let c = CELL / 6.0;
            let (tx, tz) = (g.wx(x), g.wz(z));
            let cell_of = |px: f64, pz: f64| (((px - tx) / c).floor() as i32, ((pz - tz - c / 2.0) / c).floor() as i32);
            let centre = |(ci, cj): (i32, i32)| (tx + (ci as f64 + 0.5) * c, tz + (cj as f64 + 1.0) * c);
            let (mut ci, mut cj) = cell_of(lx, lz);
            let fitting = |(ci, cj): (i32, i32)| (ci == -1 || ci == 0) && cj == -1;
            if fitting((ci, cj)) {
                cj += if lz > tz { 1 } else { -1 };
            }
            let (mx, mz) = centre((ci, cj));
            let inset = c / 2.0 - 0.07;
            let leak = (lx.clamp(mx - inset, mx + inset), lz.clamp(mz - inset, mz + inset));
            let spread = radius * 0.9;
            for dj in -2..=2 {
                for di in -2..=2 {
                    let cell = (ci + di, cj + dj);
                    let (ccx, ccz) = centre(cell);
                    // the nearest point of this tile to the leak, and how far
                    let (nx, nz) = (leak.0.clamp(ccx - c / 2.0, ccx + c / 2.0), leak.1.clamp(ccz - c / 2.0, ccz + c / 2.0));
                    let dn = (nx - leak.0).hypot(nz - leak.1);
                    let h = hash01(seed, cell.0, cell.1);
                    // (not into the next 4 m tile: its ceiling may be another height)
                    if (ccx - tx).abs() > CELL / 2.0 || (ccz - tz).abs() > CELL / 2.0 - c / 2.0 {
                        continue;
                    }
                    let spot = if (di, dj) == (0, 0) {
                        Spot { x: leak.0, z: leak.1, r: 0.2 + 0.2 * (radius - 0.5) / 0.7, rims: 3 + (h * 3.0) as usize, k: strength }
                    } else if dn < spread && h < 0.75 && !fitting(cell) {
                        let r = ((spread - dn) * (0.6 + 0.4 * hash01(seed ^ 0x77, cell.0, cell.1))).min(0.36);
                        if r < 0.07 {
                            continue;
                        }
                        // from the edge it came over, a little in
                        let k = (r * 0.3).min(0.06) / dn.max(1e-6);
                        let (sx, sz) = (nx + (nx - leak.0) * k, nz + (nz - leak.1) * k);
                        Spot { x: sx, z: sz, r, rims: 1 + (h * 4.0) as usize % 3, k: strength * 0.85 }
                    } else {
                        continue;
                    };
                    let plane = mesh(
                        plane_geo!(c, c),
                        &stain_mat(c, 192, (ccx, ccz), vec![spot], (ax, turn), 0, (leak.0, leak.1, spread * 0.5), seed),
                    );
                    plane.rotation().x = PI / 2.0;
                    plane.position().set(ccx - mx, 0, ccz - mz);
                    stain.add(&plane);
                }
            }
            stain.position().set(mx, WALL_H - 0.005, mz);
        } else {
            // a slab or plaster: one stain, lobed, where it came through
            let r = radius * 0.55;
            let side = r * 2.7;
            let plane = mesh(
                plane_geo!(side, side),
                &stain_mat(
                    side,
                    448,
                    (lx, lz),
                    vec![Spot { x: lx, z: lz, r, rims: 3 + (seed % 3) as usize, k: strength }],
                    (ax, turn),
                    kind,
                    (lx, lz, 0.0),
                    seed,
                ),
            );
            plane.rotation().x = PI / 2.0;
            stain.add(&plane);
            stain.position().set(lx, WALL_H - 0.005, lz);
        }
        g.world.add(&stain);
        i += 1;
    }
}

// R252 overturned chair: very rare — a simple wooden chair lying on its side.
pub fn place_overturned_chair(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.85 {
        return;
    } // 15% chance per level
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let pk = open_tiles[(rnd() * open_tiles.len() as f64).floor() as usize];
    let wood_mat = lambert!({ color: 0x3d2818 });
    let chair = group();
    // Seat (0.42 wide × 0.04 thick × 0.42 deep).
    let seat = mesh(pg("OverturnedChair__seat", box_geo!(0.42, 0.04, 0.42)), &wood_mat);
    seat.position().y = 0.22;
    chair.add(&seat);
    // Backrest (0.42 wide × 0.50 tall × 0.04 thick).
    let back = mesh(pg("OverturnedChair__back", box_geo!(0.42, 0.50, 0.04)), &wood_mat);
    back.position().set(0, 0.47, -0.19);
    chair.add(&back);
    // 4 legs (0.04 × 0.4 × 0.04).
    for (lx, lz) in [(-0.18, -0.18), (0.18, -0.18), (-0.18, 0.18), (0.18, 0.18)] {
        let leg = mesh(pg("OverturnedChair__leg", box_geo!(0.04, 0.40, 0.04)), &wood_mat);
        leg.position().set(lx, 0.20, lz);
        chair.add(&leg);
    }
    // Tip on side: rotate Z 90° so backrest becomes horizontal.
    chair.rotation().z = PI / 2.0 * (if rnd() < 0.5 { 1.0 } else { -1.0 });
    // After tip, base needs to translate — approximate by nudging up.
    let cx = g.wx(pk.0) + (rnd() - 0.5) * 1.2;
    let cz = g.wz(pk.1) + (rnd() - 0.5) * 1.2;
    chair.position().set(cx, 0.21, cz);
    chair.rotation().y = rnd() * PI * 2.0;
    g.world.add(&chair);
}

// R251 cigarette butts: 0-2 small piles per level (2-6 butts per pile).
pub fn place_cigarette_butts(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.55 {
        return;
    } // 45% chance of pile(s)
    let pile_count = 1 + (rnd() * 2.0).floor() as usize; // 1-2 piles
    let mut open_tiles = open_tiles(g);
    shuffle(&mut open_tiles);
    let but_mat = lambert!({ color: 0x3a3028 });
    let filter_mat = lambert!({ color: 0xa0906c });
    let mut p = 0;
    while p < pile_count && p < open_tiles.len() {
        let (x, z) = open_tiles[p];
        // Pile center within cell, biased toward wall corners.
        let corner_bias = rnd() < 0.7;
        let px = if corner_bias { (if rnd() < 0.5 { -1.0 } else { 1.0 }) * (0.9 + rnd() * 0.5) } else { (rnd() - 0.5) * 2.0 };
        let pz = if corner_bias { (if rnd() < 0.5 { -1.0 } else { 1.0 }) * (0.9 + rnd() * 0.5) } else { (rnd() - 0.5) * 2.0 };
        let n_butts = 2 + (rnd() * 5.0).floor() as i32; // 2-6 per pile
        for b in 0..n_butts {
            // Position within ~0.15 m radius of pile center.
            let bx = px + (rnd() - 0.5) * 0.30;
            let bz = pz + (rnd() - 0.5) * 0.30;
            // Two-part butt: dark burnt end + lighter filter end.
            let butt = group();
            let burnt = mesh(pg("CigaretteButts__burnt", box_geo!(0.016, 0.005, 0.005)), &but_mat);
            burnt.position().x = -0.005;
            butt.add(&burnt);
            let filter = mesh(pg("CigaretteButts__filter", box_geo!(0.010, 0.005, 0.005)), &filter_mat);
            filter.position().x = 0.008;
            butt.add(&filter);
            butt.position().set(g.wx(x) + bx, 0.003, g.wz(z) + bz);
            butt.rotation().y = rnd() * PI * 2.0;
            butt.rotation().z = (rnd() - 0.5) * 0.2;
            g.world.add(&butt);
        }
        p += 1;
    }
}
