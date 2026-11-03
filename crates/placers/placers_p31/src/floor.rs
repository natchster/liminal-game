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

// R254 ceiling water stains: 0-3 dark irregular circles on the ceiling.
pub fn place_ceiling_stains(g: &mut Game, _dist: &[i32]) {
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
        // Ceiling stains larger than floor: 0.5 - 1.2 m radius.
        let radius = 0.5 + rnd() * 0.7;
        let geo = circle_geo!(radius, 10);
        // Ellipse variant for organic look.
        if rnd() < 0.4 {
            geo.scale(1.4, 0.8, 1);
        }
        let mat = basic_mat!({
            color: spec.color,
            transparent: true,
            opacity: 0.75f64.min(spec.opacity_base + 0.15) * (0.7 + rnd() * 0.5),
            depthWrite: false,
            side: DoubleSide,
        });
        let stain = mesh(geo, &mat);
        stain.rotation().x = PI / 2.0; // face down
        stain.rotation().z = rnd() * PI * 2.0;
        let ox = (rnd() - 0.5) * 1.8;
        let oz = (rnd() - 0.5) * 1.8;
        stain.position().set(g.wx(x) + ox, WALL_H - 0.005, g.wz(z) + oz); // just under ceiling
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
