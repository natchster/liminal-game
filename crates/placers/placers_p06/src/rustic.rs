//! Rustic cabin props: antlers, signs, trophy fish, cuckoo clock, lantern,
//! cast iron, firewood, wagon wheel, map, bellows.
use crate::{face_yaw, open_tiles, pick, wall_tiles};
use game::prelude::*;

// the antlers and the trout: batch `t11`'s (`rustic/t11.rs`)
mod t11;
pub use t11::{place_mounted_antlers, place_mounted_fish};

// Fertilizer spikes: made real by batch `t06` (`t06.rs`)
pub use crate::t06::place_fertilizer_spikes;

// Rustic wooden wall sign: 4% chance per level. Three pine boards
// nailed to two battens, its name burned in, hung by a cord (batch r13a).
pub fn place_rustic_signs(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let walls = wall_tiles(g);
    if walls.is_empty() {
        return;
    }
    let (x, z, dx, dz) = pick(&walls);
    // A hand-made cabin sign (Blender batch `r13a`): three pine boards a
    // foot long nailed to two battens, a name burned into them with a
    // poker, gone grey and dry; hung by a jute cord from two screw eyes over
    // a nail, a little askew (the old tilt's draw).
    // Nothing on it says anything about the place (docs/design/PROPS.md)
    use game::assets::props::prop_part_as_modelled;
    let sign = group();
    let mount_y = 1.75;
    let tilt = (rnd() - 0.5) * 0.06;
    sign.rotation().y = face_yaw(dx, dz);
    sign.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - 0.013), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - 0.013));
    let p = sign.pos();
    let vary = rs_vary(p.x, p.z, tilt);
    // askew as the old one hung (turned about the same point, so it hangs
    // where it did); the parts are modelled with the wall's face at z = 0:
    // 13 mm behind the spot, the boards' middle at the old panel's height
    sign.rotation().z = tilt;
    let body = group();
    body.position().set(0, mount_y, -0.013);
    sign.add(&body);
    let wood = lambert!({ map: &rustic_board_tex(vary) });
    let steel = phong!({ color: 0x2c2a28, specular: 0x262422, shininess: 20 });
    let parts: [(&str, Mat); 6] = [
        ("Real__rusticSignBoards", wood),
        ("Real__rusticSignBattens", lambert!({ color: 0x6a5a48 })),
        ("Real__rusticSignNails", steel.clone()),
        ("Real__rusticSignEyes", phong!({ color: 0x3a3630, specular: 0x302c28, shininess: 30 })),
        ("Real__rusticSignCord", lambert!({ color: 0x7c6a4c })),
        ("Real__rusticSignNail", steel),
    ];
    for (name, mat) in parts {
        if let Some(geo) = prop_part_as_modelled(name) {
            body.add(&mesh(geo, &mat));
        }
    }
    g.world.add(&sign);
}

use placers_kit::place::vary as rs_vary;

/// What is burned into it: a family's name, a cabin's, a welcome
const RS_WORDS: [(&str, &str); 6] = [
    ("THE HENDERSONS", "EST. 1979"),
    ("WELCOME", "TO THE CABIN"),
    ("GONE FISHING", ""),
    ("BEAR CREEK", "CABIN"),
    ("THE OLSENS", "1984"),
    ("HOME SWEET HOME", ""),
];

/// The three boards' faces as the part's front is mapped (its bounds:
/// 0.308 x 0.146 m): pine gone grey and dry, each board its own grain
/// (the growth rings' lines, a knot or two), the gaps between them dark, the
/// name burned in with a poker (charred, soft at its edges, deeper where
/// the poker stopped), weathered lighter where hands and sun reached it,
/// grime down from the nails
fn rustic_board_tex(vary: u32) -> Tex {
    canvas_tex_opt(616, 292, false, move |ctx, w, h| {
        let seed = (rnd() * 4.0e9) as u32 ^ vary;
        with_own_stream(seed, || {
            let words = RS_WORDS[(vary as usize >> 3) % RS_WORDS.len()];
            ctx.set_fill_style("#8a7a62");
            ctx.fill_rect(0, 0, w, h);
            // the three boards: (top, bottom) in px; the gaps between dark
            let bh = h * 0.047 / 0.146;
            let gap = h * 0.0025 / 0.146;
            let boards = [(0.0, bh), (bh + gap, 2.0 * bh + gap), (2.0 * (bh + gap), h)];
            for (bi, &(y0, y1)) in boards.iter().enumerate() {
                ctx.save();
                ctx.begin_path();
                ctx.rect(0, y0, w, y1 - y0);
                ctx.clip();
                // each board its own grey-brown
                let k = 0.9 + 0.2 * rnd();
                ctx.set_fill_style(format!("rgb({:.0},{:.0},{:.0})", 138.0 * k, 122.0 * k, 98.0 * k));
                ctx.fill_rect(0, y0, w, y1 - y0);
                // the rings' lines, flat-sawn: long lines along the board, swelling
                // into a cathedral where the saw cut across the rings
                let (cx, sw) = (w * (0.15 + 0.7 * rnd()), w * (0.12 + 0.2 * rnd()));
                let rings = 18 + (rnd() * 10.0) as usize;
                // nested: each ring's line inside the last, the same wander
                let up = if rnd() < 0.5 { 1.0 } else { -1.0 };
                let (a1, f1, p1) = (0.6 + rnd() * 1.4, 0.004 + rnd() * 0.01, rnd() * 6.3);
                for r in 0..rings {
                    let t = (r as f64 + 0.3 * rnd()) / rings as f64;
                    let yb = y0 - 4.0 + (y1 - y0 + 8.0) * if up > 0.0 { t } else { 1.0 - t };
                    let lift = -up * (y1 - y0) * 0.8 * (1.0 - t);
                    let late = rnd() < 0.5;
                    ctx.set_stroke_style(format!("rgba(64,50,34,{:.2})", if late { 0.25 + 0.2 * rnd() } else { 0.1 + 0.12 * rnd() }));
                    ctx.set_line_width(if late { 1.2 + rnd() * 1.6 } else { 0.5 + rnd() * 0.8 });
                    ctx.begin_path();
                    let n = 40;
                    for k in 0..=n {
                        let xx = w * k as f64 / n as f64;
                        let yy = yb + lift * (-((xx - cx) / sw).powi(2)).exp() + a1 * (xx * f1 + p1).sin();
                        if k == 0 {
                            ctx.move_to(xx, yy);
                        } else {
                            ctx.line_to(xx, yy);
                        }
                    }
                    ctx.stroke();
                }
                // a knot now and then
                if rnd() < 0.6 {
                    let (kx, ky) = (w * (0.1 + 0.8 * rnd()), y0 + (y1 - y0) * (0.3 + 0.4 * rnd()));
                    let g = ctx.create_radial_gradient(kx, ky, 0, kx, ky, 7.0);
                    g.add_color_stop(0, "rgba(52,36,24,0.9)");
                    g.add_color_stop(0.6, "rgba(70,50,32,0.6)");
                    g.add_color_stop(1, "rgba(70,50,32,0)");
                    ctx.set_fill_style(&g);
                    ctx.begin_path();
                    ctx.ellipse(kx, ky, 9.0, 5.0, 0, 0, PI * 2.0);
                    ctx.fill();
                }
                // checks: fine splits along the grain at its ends
                for _ in 0..3 {
                    let y = y0 + (y1 - y0) * (0.2 + 0.6 * rnd());
                    let x0 = if rnd() < 0.5 { 0.0 } else { w - 40.0 - rnd() * 40.0 };
                    ctx.set_stroke_style("rgba(30,24,18,0.55)");
                    ctx.set_line_width(0.8);
                    ctx.begin_path();
                    ctx.move_to(x0, y);
                    ctx.line_to(x0 + 20.0 + rnd() * 40.0, y + (rnd() - 0.5) * 2.0);
                    ctx.stroke();
                }
                // the sawn ends and edges darker, weathered
                let ge = ctx.create_linear_gradient(0, y0, 0, y1);
                ge.add_color_stop(0, "rgba(40,32,24,0.35)");
                ge.add_color_stop(0.12, "rgba(40,32,24,0)");
                ge.add_color_stop(0.88, "rgba(40,32,24,0)");
                ge.add_color_stop(1, "rgba(40,32,24,0.35)");
                ctx.set_fill_style(&ge);
                ctx.fill_rect(0, y0, w, y1 - y0);
                let _ = bi;
                ctx.restore();
            }
            ctx.set_fill_style("rgba(24,18,12,0.9)");
            for &(_, y1) in &boards[..2] {
                ctx.fill_rect(0, y1, w, gap);
            }
            // the name, burned in: the poker's char soft round each stroke, its
            // middle darkest, unevenly (it was done by hand)
            let lines: Vec<&str> = [words.0, words.1].into_iter().filter(|s| !s.is_empty()).collect();
            let big = if lines.len() == 1 { 52.0 } else { 46.0 };
            let ys: Vec<(f64, f64)> =
                if lines.len() == 1 { vec![(h * 0.5 + big * 0.36, big)] } else { vec![(h * 0.43, big), (h * 0.79, 36.0)] };
            ctx.set_text_align("center");
            for (li, (line, &(y, size))) in lines.iter().zip(ys.iter()).enumerate() {
                let font = format!("bold {size:.0}px Copperplate");
                ctx.set_font(&font);
                let fit = ctx.measure_text(line).width;
                let sx = ((w * 0.86) / fit).min(1.0);
                ctx.save();
                ctx.translate(w / 2.0, y);
                ctx.scale(sx, 1.0);
                ctx.rotate((rnd() - 0.5) * 0.012);
                // the scorch round the letters
                for k in 0..10 {
                    let a = k as f64 / 10.0 * PI * 2.0;
                    let r = 1.6 + 0.6 * rnd();
                    ctx.set_fill_style("rgba(82,44,16,0.12)");
                    ctx.fill_text(line, a.cos() * r, a.sin() * r);
                }
                ctx.set_fill_style("rgba(48,26,10,0.85)");
                ctx.fill_text(line, 0, 0);
                ctx.set_fill_style("rgba(20,12,6,0.55)");
                ctx.fill_text(line, 0.5, 0.6);
                ctx.restore();
                let _ = li;
            }
            // the char uneven: the wood's grain shows lighter through it
            for _ in 0..500 {
                let (x, y) = (rnd() * w, rnd() * h);
                ctx.set_fill_style(format!("rgba(150,132,104,{:.2})", 0.05 + 0.1 * rnd()));
                ctx.fill_rect(x, y, 2.0 + rnd() * 8.0, 0.8);
            }
            // weathered: lighter in patches, dust on the boards' top edges
            for _ in 0..10 {
                let (x, y, r) = (rnd() * w, rnd() * h, 30.0 + rnd() * 70.0);
                let g = ctx.create_radial_gradient(x, y, 0, x, y, r);
                g.add_color_stop(0, &format!("rgba(178,168,150,{:.2})", 0.06 + 0.08 * rnd()));
                g.add_color_stop(1, "rgba(178,168,150,0)");
                ctx.set_fill_style(&g);
                ctx.fill_rect(0, 0, w, h);
            }
            // rust run down from each nail
            for &by in &[bh * 0.5, bh * 1.5 + gap, bh * 2.5 + 2.0 * gap] {
                for &bx in &[w * (0.5 - 0.105 / 0.308), w * (0.5 + 0.105 / 0.308)] {
                    let g = ctx.create_linear_gradient(bx, by, bx, by + 22.0);
                    g.add_color_stop(0, "rgba(84,46,22,0.45)");
                    g.add_color_stop(1, "rgba(84,46,22,0)");
                    ctx.set_fill_style(&g);
                    ctx.fill_rect(bx - 2.0, by, 4.0, 22.0);
                }
            }
        })
    })
}

// Cuckoo clock: 3% chance per level. Chalet body w/ peaked roof + face +
// pinecone weights.
pub fn place_cuckoo_clocks(g: &mut Game, _dist: &[i32]) {
    g.pp.cuckoo_clock_positions.clear(); // reset per level
    if rnd() < 0.97 {
        return;
    } // 3% chance
    let walls = wall_tiles(g);
    if walls.is_empty() {
        return;
    }
    let (x, z, dx, dz) = pick(&walls);
    let chalet_mat = lambert!({ color: 0x8b5a2b });
    let roof_mat = lambert!({ color: 0x604020 });
    let face_mat = lambert!({ color: 0xf0e8c8 });
    let hand_mat = lambert!({ color: 0x101010 });
    let cone_mat = lambert!({ color: 0x504020 });
    let cuckoo = group();
    let mount_y = 1.75;
    // Chalet body (main rectangle).
    let body = mesh(pg("CuckooClocks__body", box_geo!(0.22, 0.20, 0.06)), &chalet_mat);
    body.position().y = mount_y;
    cuckoo.add(&body);
    // Peaked roof (triangular prism via cone with 3 sides).
    let roof = mesh(pg("CuckooClocks__roof", cone_geo!(0.14, 0.10, 3)), &roof_mat);
    roof.rotation().y = PI / 2.0;
    roof.position().y = mount_y + 0.158; // gable wall's foot (part y −0.058) on the case top (+0.10)
    cuckoo.add(&roof);
    // Round clock face.
    let face = mesh(circle_geo!(0.055, 12), &face_mat);
    face.position().set(0, mount_y + 0.020, 0.032);
    cuckoo.add(&face);
    // Face rim.
    let rim = mesh(pg("CuckooClocks__rim", torus_geo!(0.055, 0.004, 4, 12)), &roof_mat);
    rim.rotation().x = 0.0;
    rim.position().set(0, mount_y + 0.020, 0.033);
    cuckoo.add(&rim);
    // Hour + minute hands.
    let hour_h = mesh(pg_own("CuckooClocks__hour", box_geo!(0.004, 0.030, 0.003)), &hand_mat);
    hour_h.position().set(0, mount_y + 0.035, 0.036);
    hour_h.geo().translate(0, -0.005, 0);
    hour_h.rotation().z = rnd() * PI * 2.0;
    cuckoo.add(&hour_h);
    let minute_h = mesh(pg_own("CuckooClocks__minute", box_geo!(0.003, 0.040, 0.003)), &hand_mat);
    minute_h.position().set(0, mount_y + 0.035, 0.037);
    minute_h.geo().translate(0, -0.010, 0);
    minute_h.rotation().z = rnd() * PI * 2.0;
    cuckoo.add(&minute_h);
    // 2 pinecone weights hanging below body (chain + cone).
    for side in [-1.0, 1.0] {
        let chain = mesh(pg("CuckooClocks__chain", cyl_geo!(0.001, 0.001, 0.12, 3)), &roof_mat);
        chain.position().set(side * 0.070, mount_y - 0.16, 0.020);
        cuckoo.add(&chain);
        let cone = mesh(pg("CuckooClocks__cone", cone_geo!(0.010, 0.024, 6)), &cone_mat);
        cone.rotation().x = PI;
        cone.position().set(side * 0.070, mount_y - 0.231, 0.020); // eye (top) on the chain's end (−0.22)
        cuckoo.add(&cone);
    }
    cuckoo.rotation().y = face_yaw(dx, dz);
    cuckoo.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - 0.030), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - 0.030));
    g.world.add(&cuckoo);
    let p = cuckoo.pos();
    g.pp.cuckoo_clock_positions.push(v3(p.x, 0, p.z)); // track for cuckoo call
}

// Metal candle lantern: 4% chance per level. Frame + glass panels +
// candle + handle.
pub fn place_candle_lanterns(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let frame_mat = lambert!({ color: 0x303030 });
    let glass_mat = basic_mat!({ color: 0xe8e0c0, transparent: true, opacity: 0.35 });
    let candle_mat = lambert!({ color: 0xf0e8d0 });
    let is_lit = rnd() < 0.4;
    let flame_mat = basic_mat!({ color: 0xffa040 });
    let lantern = group();
    // Base disc.
    let base = mesh(pg("CandleLanterns__base", cyl_geo!(0.045, 0.045, 0.010, 6)), &frame_mat);
    base.position().y = 0.005;
    lantern.add(&base);
    // 4 corner posts.
    for i in 0..4 {
        let a = (i as f64 / 4.0) * PI * 2.0 + PI / 4.0;
        let post = mesh(pg("CandleLanterns__post", cyl_geo!(0.003, 0.003, 0.14, 4)), &frame_mat);
        post.position().set(a.cos() * 0.032, 0.075, a.sin() * 0.032);
        lantern.add(&post);
    }
    // 4 glass panels (planes between posts).
    for i in 0..4 {
        let a = (i as f64 / 4.0) * PI * 2.0;
        let panel = mesh(plane_geo!(0.045, 0.12), &glass_mat);
        // on the line between two diagonal posts (0.032 · cos 45° from the centre)
        panel.position().set(a.cos() * 0.032 * std::f64::consts::FRAC_1_SQRT_2, 0.075, a.sin() * 0.032 * std::f64::consts::FRAC_1_SQRT_2);
        panel.rotation().y = -a + PI / 2.0;
        lantern.add(&panel);
    }
    // Top cap.
    let top = mesh(pg("CandleLanterns__top", cyl_geo!(0.045, 0.045, 0.010, 6)), &frame_mat);
    top.position().y = 0.145;
    lantern.add(&top);
    // Candle inside.
    let candle = mesh(pg("CandleLanterns__candle", cyl_geo!(0.012, 0.012, 0.060, 8)), &candle_mat);
    candle.position().y = 0.045;
    candle.scale().y = 0.070 / 0.060; // from the plinth top (0.010) up to the flame (0.081)
    lantern.add(&candle);
    if is_lit {
        // Wick + flame.
        let flame = mesh(cone_geo!(0.005, 0.014, 5), &flame_mat);
        flame.position().y = 0.088;
        lantern.add(&flame);
        // Light source.
        let light = point_light!(0xffa040, 0.18, 1.5);
        light.position().y = 0.088;
        lantern.add(&light);
        // Flame flicker animator (rapid noise-like intensity + scale wobble).
        let phase = rnd() * PI * 2.0;
        g.animate(move |_g, _dt, t| {
            // Rapid flicker via combined sines (like turbulent flame).
            let f = 0.7 + 0.15 * (t * 7.0 + phase).sin() + 0.15 * (t * 11.0 + phase * 1.7).sin();
            flame.scale().set_scalar(f);
            light.set_intensity(0.18 * f);
        });
    }
    // Handle loop on top (torus arch).
    let handle = mesh(pg("CandleLanterns__handle", torus_geo!(0.020, 0.002, 3, 8, PI)), &frame_mat);
    handle.rotation().y = PI / 2.0;
    handle.position().y = 0.149; // hooked ends in the cap's ears (y 0.145–0.150), not 5 mm above
    lantern.add(&handle);
    lantern.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    lantern.rotation().y = rnd() * PI * 2.0;
    g.world.add(&lantern);
}

// Hanging horseshoe: 5% chance per level. Wall-mounted U-shape iron shoe
// w/ nail holes.
pub fn place_horseshoes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let walls = wall_tiles(g);
    if walls.is_empty() {
        return;
    }
    let (x, z, dx, dz) = pick(&walls);
    let iron_mat = lambert!({ color: 0x504040 });
    let nail_mat = lambert!({ color: 0x707070 });
    let shoe = group();
    let mount_y = 1.75;
    // Main U-shape via 3/4 torus.
    let arch = mesh(pg("Horseshoes__arch", torus_geo!(0.055, 0.010, 6, 14, PI * 1.3)), &iron_mat);
    arch.rotation().z = -PI * 0.15;
    arch.position().set(0, mount_y, 0.005);
    shoe.add(&arch);
    // 6 nail holes (small dark dots around inside of arch).
    for i in 0..6 {
        // along the web's fuller groove (r ≈ 0.058), clear of the heels and the toe clip;
        // 0.6 mm proud of the web face (z 0.0122). The group is already turned to the room.
        let a = -PI * 0.03 + (i as f64 / 5.0) * PI * 1.06;
        let nail = mesh(circle_geo!(0.004, 5), &nail_mat);
        nail.position().set(a.cos() * 0.058, mount_y + a.sin() * 0.058, 0.0128);
        shoe.add(&nail);
    }
    // Small nail-hanger at very top.
    let hanger = mesh(pg("Horseshoes__hanger", cyl_geo!(0.002, 0.002, 0.010, 4)), &nail_mat);
    // driven into the wall (z -0.008), head (+Y) towards the room; the toe's inner
    // edge (r 0.045) rests on its shank
    hanger.rotation().x = PI / 2.0;
    hanger.position().set(0, mount_y + 0.043, -0.004);
    shoe.add(&hanger);
    shoe.rotation().y = face_yaw(dx, dz);
    shoe.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - 0.008), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - 0.008));
    g.world.add(&shoe);
}

// A cast-iron skillet hung on the wall (Blender batch `r16`): a 10 1/4
// in skillet of the Lodge L8SK3 class, hung by the hole in its handle from
// a nail, its bottom against the wall and its cooking face to the room; its
// seasoning gone dull and flaking at the rim, rusted where it was put away
// wet: a tide of orange rust low in the pan where the water stood, rust run
// down from it and blooming at the pour lips and in the hole. 4% chance per
// level.
pub fn place_cast_iron_pans(g: &mut Game, _dist: &[i32]) {
    use game::assets::props::prop_part_as_modelled;
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let walls = wall_tiles(g);
    if walls.is_empty() {
        return;
    }
    let (x, z, dx, dz) = pick(&walls);
    let pan = group();
    // its rust differs floor to floor: from where it hangs (no draw)
    let salt = ((x * 7919 + z * 104_729 + dx * 31 + dz * 17) as u32).wrapping_mul(0x9e37_79b9);
    // the nail's hole 1.93 m up, so the pan's middle is at 1.68 m (the old
    // pan's 1.80 m middle and its handle)
    let lift = PAN_NAIL_Y - PAN_HOLE_Y;
    if let Some(geo) = prop_part_as_modelled("Real__panBody") {
        let body = mesh(geo, phong!({ map: &pan_iron_tex(salt), color: 0xffffff, specular: 0x201e1c, shininess: 18 }));
        body.position().set(0, lift, -0.015);
        pan.add(&body);
    }
    if let Some(geo) = prop_part_as_modelled("Real__panNail") {
        let nail = mesh(geo, phong!({ color: 0x4a3c32, specular: 0x262422, shininess: 18 }));
        nail.position().set(0, lift, -0.015);
        pan.add(&nail);
    }
    pan.rotation().y = face_yaw(dx, dz);
    pan.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - 0.015), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - 0.015));
    g.world.add(&pan);
}

/// The skillet as hung (`Real__panBody`): its bounds (x, y; r16.py), its
/// middle at y = 0, the hanging hole's centre, and where the nail is
const PAN_BOUNDS: [f64; 4] = [-0.1393, -0.1697, 0.1393, 0.2631];
const PAN_HOLE_Y: f64 = 0.2482;
const PAN_NAIL_Y: f64 = 1.93;
/// The cooking face's radius, and the inside of the rim (r16.py `PAN_PROFILE`)
const PAN_FLOOR_R: f64 = 0.0915;
const PAN_RIM_R: f64 = 0.1284;

/// A hash of an integer lattice point
fn r16_hash(seed: u32, x: i32, y: i32) -> f64 {
    let mut h = seed ^ (x as u32).wrapping_mul(0x8da6_b343) ^ (y as u32).wrapping_mul(0xd816_3841);
    h = (h ^ (h >> 13)).wrapping_mul(0xcb1a_b31f);
    h ^= h >> 16;
    h as f64 / 4_294_967_296.0
}
/// Smooth value noise in [0, 1]
fn r16_noise(seed: u32, x: f64, y: f64) -> f64 {
    let (xf, yf) = (x.floor(), y.floor());
    let (fx, fy) = (x - xf, y - yf);
    let (sx, sy) = (fx * fx * (3.0 - 2.0 * fx), fy * fy * (3.0 - 2.0 * fy));
    let (xi, yi) = (xf as i32, yf as i32);
    let (a, b) = (r16_hash(seed, xi, yi), r16_hash(seed, xi + 1, yi));
    let (c, d) = (r16_hash(seed, xi, yi + 1), r16_hash(seed, xi + 1, yi + 1));
    a + (b - a) * sx + (c - a) * sy + (a - b - c + d) * sx * sy
}
/// Three octaves of it
fn r16_fbm(seed: u32, x: f64, y: f64) -> f64 {
    0.55 * r16_noise(seed, x, y) + 0.3 * r16_noise(seed ^ 0x51, x * 2.3, y * 2.3) + 0.15 * r16_noise(seed ^ 0xa2, x * 5.1, y * 5.1)
}
fn r16_mix(c: &mut [f64; 3], to: [f64; 3], k: f64) {
    let k = k.clamp(0.0, 1.0);
    for i in 0..3 {
        c[i] += (to[i] - c[i]) * k;
    }
}

/// The iron, seen from the room (the box's texture coordinates: u across,
/// v up over the hung pan's bounds): black seasoning gone dull, worn grey
/// where it was scoured, flaking at the rim; rust where it was left wet: a
/// tide low in the pan where water stood when it lay flat, spots and
/// blooms, rust run down the walls from the lips and the rim, and in the
/// hanging hole; the handle polished dark by hands.
fn pan_iron_tex(salt: u32) -> Tex {
    canvas_tex_opt(256, 384, false, move |ctx, w, h| {
        let seed = salt ^ 0x1a0e;
        let [x0, y0, x1, y1] = PAN_BOUNDS;
        let mut r = SplitMix64::new(salt, 0xfe20);
        // where the water stood: a pool low in the pan (the pan lay flat;
        // it dried from its edges), its centre off the middle
        let pool = (r.range(-0.025, 0.025), r.range(-0.04, -0.015), r.range(0.055, 0.075));
        let n = w as usize;
        let mut img = ctx.create_image_data(w, h);
        par_rows(&mut img.data, n * 4, |j, out| {
            let py = y1 - (j as f64 + 0.5) / h * (y1 - y0);
            for i in 0..n {
                let px = x0 + (i as f64 + 0.5) / w * (x1 - x0);
                let rr = (px * px + py * py).sqrt();
                let f = r16_fbm(seed, px * 40.0, py * 40.0);
                let grain = r16_noise(seed ^ 3, px * 900.0, py * 900.0);
                // the seasoning: near-black, a little brown, its cast grain
                let mut c = [27.0 + 7.0 * f + 5.0 * grain, 25.0 + 6.0 * f + 4.0 * grain, 24.0 + 5.0 * f + 3.0 * grain];
                // dulled grey-brown where a spatula and steel wool went
                let inside = rr < PAN_FLOOR_R + 0.008;
                if inside {
                    let scour = (r16_fbm(seed ^ 5, px * 25.0, py * 25.0) - 0.5).max(0.0) * 1.2;
                    r16_mix(&mut c, [52.0, 48.0, 44.0], scour * 0.35);
                }
                // flaking at the rim: the seasoning lifted to grey iron
                let rim = (1.0 - ((rr - PAN_RIM_R) / 0.010).abs()).max(0.0);
                let flake = ((r16_noise(seed ^ 7, px * 160.0, py * 160.0) - 0.55) * 3.0).max(0.0) * rim;
                r16_mix(&mut c, [62.0, 56.0, 50.0], flake * 0.7);
                // rust where the water stood when it lay flat: a ragged pool,
                // uneven inside (orange powder, dark crust), blooms round it
                let (pxc, pyc, pr) = pool;
                let dpool = (((px - pxc) / 1.2).powi(2) + (py - pyc).powi(2)).sqrt() / pr;
                let ragged = dpool + 0.55 * (r16_fbm(seed ^ 9, px * 22.0, py * 22.0) - 0.5);
                let in_pool = ((1.0 - ragged) * 4.0).clamp(0.0, 1.0) * (0.55 + 0.45 * r16_fbm(seed ^ 17, px * 70.0, py * 70.0));
                let bloom = ((r16_fbm(seed ^ 11, px * 34.0, py * 34.0) - 0.6).max(0.0) * 4.0).min(1.0);
                let spots = ((r16_noise(seed ^ 19, px * 260.0, py * 260.0) - 0.8).max(0.0) * 5.0).min(1.0);
                let wet = if inside { 1.0 } else { 0.25 };
                // rust at the pour lips, where water ran out
                let lip = ((px.abs() - 0.118) / 0.012).clamp(0.0, 1.0) * (1.0 - (py.abs() / 0.02).min(1.0));
                let rust = ((in_pool + bloom * 0.8 + spots * 0.6) * wet + lip * 0.4 + rim * flake * 0.5).min(1.0);
                // rust's own colour: orange-brown powder, dark where it crusted
                let rk = r16_noise(seed ^ 15, px * 300.0, py * 300.0);
                let crust = r16_noise(seed ^ 21, px * 90.0, py * 90.0);
                let rc = [96.0 + 44.0 * rk - 30.0 * crust, 46.0 + 20.0 * rk - 14.0 * crust, 24.0 + 10.0 * rk - 6.0 * crust];
                r16_mix(&mut c, rc, rust * (0.8 + 0.2 * grain));
                // the handle, polished dark by hands, the hole rusted
                if py > 0.14 {
                    let dh = ((py - PAN_HOLE_Y).powi(2) + px * px).sqrt();
                    let hole = (1.0 - (dh - 0.008) / 0.005).clamp(0.0, 1.0);
                    let polish = ((0.23 - py) / 0.06).clamp(0.0, 1.0) * 0.3;
                    r16_mix(&mut c, [19.0, 18.0, 18.0], polish);
                    r16_mix(&mut c, [104.0, 52.0, 26.0], hole * 0.7);
                }
                out[i * 4..i * 4 + 4].copy_from_slice(&[c[0] as u8, c[1] as u8, c[2] as u8, 255]);
            }
        });
        ctx.put_image_data(&img, 0, 0);
    })
}

// Wooden staved bucket: 4% chance per level. Barrel-style bucket w/
// vertical staves + metal bands.
pub fn place_wooden_buckets(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    let wood_mat = lambert!({ color: 0x8b5a2b });
    let band_mat = lambert!({ color: 0x505050 });
    let bucket = group();
    // Main body (slightly tapered cylinder — barrel shape).
    let body = mesh(pg("WoodenBuckets__body", cyl_geo!(0.080, 0.070, 0.14, 12)), &wood_mat);
    body.position().y = 0.070;
    bucket.add(&body);
    // 12 vertical stave lines (darker thin cylinders on outside).
    for i in 0..12 {
        let a = (i as f64 / 12.0) * PI * 2.0;
        let stave = mesh(cyl_geo!(0.0015, 0.0015, 0.14, 3), lambert!({ color: 0x603818 }));
        // follow the body's taper (r ≈ 0.072 at the foot, 0.081 at the rim)
        stave.rotation().y = -a;
        stave.rotation().z = -0.0655;
        stave.position().set(a.cos() * 0.0770, 0.070, a.sin() * 0.0770);
        bucket.add(&stave);
    }
    // 2 metal bands (top + bottom).
    for i in 0..2 {
        let band_y = 0.020 + i as f64 * 0.100;
        let band = mesh(pg("WoodenBuckets__band", torus_geo!(0.079 - i as f64 * 0.005, 0.004, 4, 16)), &band_mat);
        // sized to the body at this height (it widens upward): the bottom band
        // floated off it and the top one was buried
        let band_s = (0.070 + 0.010 * band_y / 0.14 + 0.0015) / (0.079 - i as f64 * 0.005);
        band.scale().set(band_s, band_s, 1);
        band.rotation().x = PI / 2.0;
        band.position().y = band_y;
        bucket.add(&band);
    }
    // Curved wire handle.
    let handle = mesh(pg("WoodenBuckets__handle", torus_geo!(0.085, 0.003, 4, 12, PI)), &band_mat);
    handle.rotation().y = PI / 2.0;
    handle.position().y = 0.138;
    bucket.add(&handle);
    bucket.position().set(g.wx(x) + (rnd() - 0.5) * 2.0, 0, g.wz(z) + (rnd() - 0.5) * 2.0);
    bucket.rotation().y = rnd() * PI * 2.0;
    g.world.add(&bucket);
}

// Hanging cast iron kettle: 3% chance per level. Wall-hung tea kettle w/
// spout + handle.
pub fn place_cast_iron_kettles(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    } // 3% chance
    let walls = wall_tiles(g);
    if walls.is_empty() {
        return;
    }
    let (x, z, dx, dz) = pick(&walls);
    let iron_mat = lambert!({ color: 0x181818 });
    let kettle = group();
    let mount_y = 1.75;
    // Bulbous body (sphere).
    let body = mesh(pg("CastIronKettles__body", sphere_geo!(0.060, 12, 10)), &iron_mat);
    body.scale().y = 0.85;
    body.position().set(0, mount_y, 0.055);
    kettle.add(&body);
    // Angled spout extending up-forward.
    let spout = mesh(pg("CastIronKettles__spout", cyl_geo!(0.010, 0.014, 0.070, 6)), &iron_mat);
    spout.rotation().z = PI / 3.0; // narrow mouth up and out at -X, wide root in the body
    spout.position().set(-0.055, mount_y + 0.028, 0.060);
    kettle.add(&spout);
    // Lid on top (short disc).
    let lid = mesh(pg("CastIronKettles__lid", cyl_geo!(0.020, 0.024, 0.008, 8)), &iron_mat);
    lid.position().set(0, mount_y + 0.056, 0.055);
    kettle.add(&lid);
    // Lid knob.
    let knob = mesh(pg("CastIronKettles__knob", sphere_geo!(0.005, 5, 4)), &iron_mat);
    knob.position().set(0, mount_y + 0.064, 0.055);
    kettle.add(&knob);
    // Curved handle arch over top.
    let handle = mesh(pg("CastIronKettles__handle", torus_geo!(0.055, 0.004, 4, 10, PI)), &iron_mat);
    handle.rotation().z = 0.0;
    handle.rotation().y = PI / 2.0;
    // ends on the body's shoulders (surface y ≈ +0.020 at 5.5 cm out), arch clears the knob
    handle.position().set(0, mount_y + 0.018, 0.055);
    kettle.add(&handle);
    kettle.rotation().y = face_yaw(dx, dz);
    kettle.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - 0.060), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - 0.060));
    g.world.add(&kettle);
}

// A stack of firewood (Blender batch `r16`): 3-5 splits of stove wood,
// 300 mm long (a quarter of a big round, half a small one, a slab off a big
// one, a branch round), a bottom row and one or two laid in its grooves as
// the old stack was; their bark (oak, ash, elm or birch: the old bark
// colours), the faces they were split along gone grey, their sawn ends'
// rings round the pith, checked by drying. 5% chance per level.
pub fn place_firewood_stacks(g: &mut Game, _dist: &[i32]) {
    use game::assets::props::prop_part_as_modelled;
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    // the old bark colours: oak, ash, elm, birch
    let bark_colors: [u32; 4] = [0x6b3f1f, 0x805030, 0x503020, 0x9a6f4b];
    let bark_color = pick(&bark_colors);
    let species = bark_colors.iter().position(|&c| c == bark_color).unwrap_or(0);
    let count = 3 + (rnd() * 3.0).floor() as i32;
    // the old stack's loop: one draw a log (its turn); now the grain's seed
    let layers = 2i32.min((count as f64 / 2.0).floor() as i32);
    let bottom_row = ((count + 1) as f64 / 2.0).ceil() as i32;
    let mut idx = 0;
    let mut salt: u32 = 0x9e37_79b9 ^ count as u32;
    'rows: for layer in 0..=layers {
        let row_count = bottom_row - layer;
        if row_count <= 0 {
            break;
        }
        for _ in 0..row_count {
            let turn = (rnd() - 0.5) * 0.1;
            salt = salt.rotate_left(7) ^ (turn * 1.0e7) as i64 as u32;
            idx += 1;
            if idx >= count {
                break 'rows;
            }
        }
    }
    let n = count.clamp(3, 5) as usize;
    let stack = group();
    let mats = [
        lambert!({ map: &wood_bark_tex(species, salt) }),
        lambert!({ map: &wood_split_tex(species, salt) }),
        lambert!({ map: &wood_end_tex(species, salt, WOOD_PITHS[n - 3][0], false) }),
        lambert!({ map: &wood_end_tex(species, salt ^ 0x55, WOOD_PITHS[n - 3][1], true) }),
    ];
    for (k, part) in ["Bark", "Split", "EndL", "EndR"].iter().enumerate() {
        if let Some(geo) = prop_part_as_modelled(&format!("Real__woodStack{n}{part}")) {
            stack.add(&mesh(geo, &mats[k]));
        }
    }
    stack.position().set(g.wx(x) + (rnd() - 0.5) * 1.8, 0, g.wz(z) + (rnd() - 0.5) * 1.8);
    stack.rotation().y = rnd() * PI * 2.0;
    g.world.add(&stack);
}

/// Each stack's piths on its two sawn ends, in its end parts' texture
/// coordinates (u, v), and the bark's radius about each (in u and in v):
/// printed by r16.py (`_print_piths`) from the stacks as modelled
type WoodPiths = &'static [(f64, f64, f64, f64)];
const WOOD_PITHS: [[WoodPiths; 2]; 3] = [
    [
        &[(0.0055, 0.0258, 0.4243, 0.7347), (0.7389, 0.0258, 0.2586, 0.4478), (0.4235, 0.7248, 0.1697, 0.2939)],
        &[(0.9943, 0.0088, 0.4355, 0.7218), (0.2787, 0.0088, 0.2655, 0.4400), (0.5191, 0.6956, 0.1742, 0.2887)],
    ],
    [
        &[
            (0.2930, 0.0111, 0.2904, 0.4692),
            (0.1308, 0.0111, 0.4701, 0.7596),
            (0.8206, 0.0111, 0.1770, 0.2860),
            (0.1485, 0.5248, 0.2904, 0.4692),
        ],
        &[
            (0.7007, 0.0166, 0.2948, 0.4647),
            (0.8432, 0.0166, 0.4774, 0.7523),
            (0.1846, 0.0166, 0.1797, 0.2832),
            (0.8263, 0.5254, 0.2948, 0.4647),
        ],
    ],
    [
        &[
            (0.1849, 0.0213, 0.1769, 0.3314),
            (0.8716, 0.0213, 0.4700, 0.8802),
            (0.6964, 0.0213, 0.2903, 0.5436),
            (0.3737, 0.6553, 0.1161, 0.2175),
            (0.6888, 0.6590, 0.1769, 0.3314),
        ],
        &[
            (0.8206, 0.0135, 0.1771, 0.3313),
            (0.1686, 0.0135, 0.4704, 0.8801),
            (0.2960, 0.0135, 0.2906, 0.5436),
            (0.6333, 0.6475, 0.1162, 0.2174),
            (0.3105, 0.6512, 0.1771, 0.3313),
        ],
    ],
];

/// Bark by species (sRGB): its ridges, its fissures
const WOOD_BARK: [([f64; 3], [f64; 3]); 4] = [
    ([104.0, 96.0, 84.0], [44.0, 38.0, 32.0]),   // oak: grey-brown ridges, deep dark fissures
    ([128.0, 124.0, 112.0], [70.0, 66.0, 58.0]), // ash: grey, a diamond net of shallow fissures
    ([92.0, 74.0, 58.0], [40.0, 30.0, 24.0]),    // elm: brown, corky
    ([196.0, 192.0, 180.0], [42.0, 38.0, 34.0]), // birch: white, black lenticels and cracks
];
/// The wood itself, fresh-split and weathered (sRGB)
const WOOD_GRAIN: [([f64; 3], [f64; 3]); 4] = [
    ([170.0, 138.0, 100.0], [128.0, 122.0, 112.0]),
    ([186.0, 168.0, 132.0], [138.0, 132.0, 122.0]),
    ([150.0, 112.0, 80.0], [118.0, 108.0, 96.0]),
    ([198.0, 180.0, 146.0], [146.0, 140.0, 128.0]),
];

/// The bark round every log (the box's texture coordinates over the whole
/// stack: u along the logs): ridges along the grain between fissures (birch
/// across it: lenticels and peeling), lichen and dirt here and there
fn wood_bark_tex(species: usize, salt: u32) -> Tex {
    canvas_tex_opt(512, 256, false, move |ctx, w, h| {
        let seed = salt ^ 0xba4c;
        let (ridge, fissure) = WOOD_BARK[species];
        let n = w as usize;
        let mut img = ctx.create_image_data(w, h);
        par_rows(&mut img.data, n * 4, |j, out| {
            let y = j as f64;
            for i in 0..n {
                let x = i as f64;
                let mut c = ridge;
                if species == 3 {
                    // birch: white, dark lenticels across the grain (short
                    // lines along v... the logs run along u), black cracks
                    let len = r16_noise(seed, x / 2.0, y / 14.0);
                    let crack = (r16_fbm(seed ^ 1, x / 40.0, y / 6.0) - 0.62).max(0.0) * 4.0;
                    let dirt = r16_fbm(seed ^ 2, x / 60.0, y / 60.0);
                    r16_mix(&mut c, fissure, ((len - 0.8).max(0.0) * 6.0).min(0.9) + crack);
                    r16_mix(&mut c, [150.0, 140.0, 120.0], (dirt - 0.4).max(0.0));
                } else {
                    // ridges along u, fissures between, net-like for ash
                    let stretch = if species == 1 { 9.0 } else { 18.0 };
                    let f = r16_fbm(seed, x / stretch / 4.0, y / 4.5);
                    let fis = (1.0 - ((f - 0.5).abs() * 4.0).min(1.0)).powf(2.0);
                    let tone = r16_noise(seed ^ 3, x / 3.0, y / 3.0);
                    r16_mix(&mut c, fissure, fis * 0.9);
                    for k in 0..3 {
                        c[k] *= 0.85 + 0.3 * tone;
                    }
                }
                // lichen, grey-green, in patches; dirt where it lay
                let lichen = (r16_fbm(seed ^ 4, x / 30.0, y / 30.0) - 0.68).max(0.0) * 3.5;
                r16_mix(&mut c, [130.0, 136.0, 110.0], lichen * 0.6);
                out[i * 4..i * 4 + 4].copy_from_slice(&[
                    c[0].clamp(0.0, 255.0) as u8,
                    c[1].clamp(0.0, 255.0) as u8,
                    c[2].clamp(0.0, 255.0) as u8,
                    255,
                ]);
            }
        });
        ctx.put_image_data(&img, 0, 0);
    })
}

/// The faces it was split along (u along the logs): torn fibres along the
/// grain, the wood gone grey where the air had it, darker in the tears,
/// a few dark streaks of rot and dirt
fn wood_split_tex(species: usize, salt: u32) -> Tex {
    canvas_tex_opt(512, 256, false, move |ctx, w, h| {
        let seed = salt ^ 0x5b17;
        let (fresh, grey) = WOOD_GRAIN[species];
        let n = w as usize;
        let mut img = ctx.create_image_data(w, h);
        par_rows(&mut img.data, n * 4, |j, out| {
            let y = j as f64;
            for i in 0..n {
                let x = i as f64;
                let fib = r16_noise(seed, x / 40.0, y / 1.3) * 0.5 + r16_noise(seed ^ 1, x / 12.0, y / 0.7) * 0.5;
                let weather = 0.55 + 0.4 * r16_fbm(seed ^ 2, x / 50.0, y / 20.0);
                let mut c = fresh;
                r16_mix(&mut c, grey, weather);
                for k in 0..3 {
                    c[k] *= 0.78 + 0.36 * fib;
                }
                let tear = (r16_noise(seed ^ 3, x / 25.0, y / 2.0) - 0.78).max(0.0) * 4.0;
                r16_mix(&mut c, [60.0, 50.0, 42.0], tear * 0.7);
                let rot = (r16_fbm(seed ^ 4, x / 70.0, y / 10.0) - 0.7).max(0.0) * 3.0;
                r16_mix(&mut c, [74.0, 60.0, 50.0], rot * 0.6);
                out[i * 4..i * 4 + 4].copy_from_slice(&[
                    c[0].clamp(0.0, 255.0) as u8,
                    c[1].clamp(0.0, 255.0) as u8,
                    c[2].clamp(0.0, 255.0) as u8,
                    255,
                ]);
            }
        });
        ctx.put_image_data(&img, 0, 0);
    })
}

/// One end of the stack: each log's sawn end grain, its rings round its
/// pith (the heart darker for oak and elm), checks split out from the pith
/// as it dried, the chainsaw's faint arcs, the bark round it; weathered grey.
/// `piths`: (u, v, the bark's radius in u, in v) for each log.
fn wood_end_tex(species: usize, salt: u32, piths: &'static [(f64, f64, f64, f64)], right: bool) -> Tex {
    canvas_tex_opt(512, 320, false, move |ctx, w, h| {
        let seed = salt ^ if right { 0xe2d1 } else { 0xe2d0 };
        let (fresh, grey) = WOOD_GRAIN[species];
        let (ridge, _) = WOOD_BARK[species];
        let mut r = SplitMix64::new(seed, 0x4e11);
        // each log's checks: angles of the cracks out from its pith, how far
        let checks: Vec<Vec<(f64, f64)>> = piths
            .iter()
            .map(|_| (0..3 + (r.next() * 4.0) as usize).map(|_| (r.range(0.0, PI * 2.0), r.range(0.35, 0.95))).collect())
            .collect();
        let n = w as usize;
        let mut img = ctx.create_image_data(w, h);
        par_rows(&mut img.data, n * 4, |j, out| {
            let v = 1.0 - (j as f64 + 0.5) / h;
            for i in 0..n {
                let u = (i as f64 + 0.5) / w;
                // the nearest pith (in units of its bark's radius)
                let mut best = (f64::INFINITY, 0usize, 0.0, 0.0, 0.0);
                for (k, &(pu, pv, ru, rv)) in piths.iter().enumerate() {
                    let (du, dv) = ((u - pu) / ru, (v - pv) / rv);
                    let d = (du * du + dv * dv).sqrt();
                    if d < best.0 {
                        best = (d, k, du, dv, ru);
                    }
                }
                let (d, k, du, dv, ru) = best;
                // the log's radius in metres (the ends are about 0.35 m across)
                let rad = ru * 0.35;
                let ang = dv.atan2(du);
                let wob = 0.025 * (r16_noise(seed ^ k as u32, ang * 2.5 + 10.0, d * 2.0) - 0.5);
                let rings = ((d + wob) * rad / 0.0032).fract();
                let late = (1.0 - ((rings - 0.82) / 0.12).abs()).max(0.0);
                let mut c = fresh;
                let heart = if species == 0 || species == 2 { (1.0 - d / 0.7).clamp(0.0, 1.0) } else { 0.0 };
                let dark = [c[0] * 0.74, c[1] * 0.66, c[2] * 0.6];
                r16_mix(&mut c, dark, heart.sqrt() * 0.8);
                for q in 0..3 {
                    c[q] *= 1.0 - 0.28 * late;
                }
                // weathered grey, more toward the rim; the pith a dark speck
                r16_mix(&mut c, grey, 0.35 + 0.25 * d.min(1.0));
                if d * rad < 0.0025 {
                    r16_mix(&mut c, [70.0, 56.0, 44.0], 0.7);
                }
                // the bark round it, a band its own thickness
                let band = 1.0 - 0.006 / rad.max(0.01);
                if d > band {
                    r16_mix(&mut c, [ridge[0] * 0.9, ridge[1] * 0.88, ridge[2] * 0.86], ((d - band) / 0.03).min(1.0) * 0.85);
                }
                // checks out from the pith
                for &(a, l) in &checks[k] {
                    let da = ((ang - a + PI).rem_euclid(PI * 2.0) - PI).abs();
                    let wid = 0.0012 / rad.max(0.01) * (1.0 - d / l).max(0.0);
                    if d < l && da * d.max(0.02) < wid {
                        r16_mix(&mut c, [40.0, 32.0, 26.0], 0.85);
                    }
                }
                // the chainsaw's arcs, faint
                let saw = (((u * 3.0 + v * 0.4) * 60.0 + (u * 7.0).sin() * 3.0).fract() - 0.5).abs();
                for q in 0..3 {
                    c[q] *= 0.96 + 0.08 * saw;
                }
                out[i * 4..i * 4 + 4].copy_from_slice(&[
                    c[0].clamp(0.0, 255.0) as u8,
                    c[1].clamp(0.0, 255.0) as u8,
                    c[2].clamp(0.0, 255.0) as u8,
                    255,
                ]);
            }
        });
        ctx.put_image_data(&img, 0, 0);
    })
}

// Wagon wheel: made real by batch `t19` (`rustic/t19.rs`)
mod t19;
pub use t19::place_wagon_wheels;

// Framed vintage map (Blender batch `t02`).
mod t02;
pub use t02::place_vintage_maps;

// Hearth bellows: batch `t20`'s (`rustic/t20.rs`)
mod t20;
pub use t20::place_bellows;
