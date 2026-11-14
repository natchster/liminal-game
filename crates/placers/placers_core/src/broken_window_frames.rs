//! `place_broken_window_frames`
use game::prelude::*;
use placers_kit::place::ri;
use placers_kit::place::wall_candidates;

// Broken window frame: 15% chance per level. The lower sash of an old
// timber double-hung window (Blender family `crate_sash_umbrella_and_things_put_away`), taken out and left
// leaning on a wall, two over two lights: its white paint gone cream and
// peeling to the grey wood, the putty cracked, a few shards of its glass
// still in the putty, the stub of its cord; the rest of the glass on the
// floor in front of it. Now and then a damp streak down the wall behind it,
// and the wet at its foot.
pub(crate) fn place_broken_window_frames(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.85 {
        return;
    } // 15% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = candidates[ri(candidates.len())];
    let shard_mat_col = shard_color_by_weather(g.weather_key());
    let glass = sash_glass_mat(shard_mat_col);
    let sash = group();
    {
        use game::assets::props::prop_part_as_modelled as part;
        let frame = part("Real__sashFrame").unwrap_or_else(|| {
            let g = box_geo!(SASH_W, SASH_H, SASH_T);
            g.translate(0, SASH_H / 2.0, -SASH_T / 2.0);
            g
        });
        sash.add(&mesh(frame, lambert!({ color: 0xffffff, map: &sash_paint_tex() })));
        if let Some(geo) = part("Real__sashPutty") {
            sash.add(&mesh(geo, lambert!({ color: 0x9a927e })));
        }
        if let Some(geo) = part("Real__sashCord") {
            sash.add(&mesh(geo, lambert!({ color: 0x80766a })));
        }
    }
    // 3-5 shards still held in the putty, each at an edge of a light (the
    // edge, where along it, which way it leans and how big: its four draws)
    let shard_count = 3 + (rnd() * 3.0).floor() as i32;
    for _s in 0..shard_count {
        let s_size = 0.04 + rnd() * 0.05;
        let edge = (rnd() * 4.0).floor() as i32;
        let along = rnd();
        let lean = rnd() - 0.5;
        sash.add(&mesh(sash_shard_geo(s_size, edge, along, lean), &glass));
    }
    // leaning on the wall: its foot out from the wall, its top against it
    let lean_a = SASH_LEAN;
    let out = SASH_H * lean_a.dsin() + SASH_T * lean_a.dcos();
    sash.set_rotation_order(EulerOrder::YXZ);
    sash.rotation().x = -lean_a;
    // (the wall point it was hung at, along the wall by the old draw)
    let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.010);
    let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.010);
    let (mx, mz) = if c.dx.abs() == 1 {
        sash.rotation().y = if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 };
        (wall_x, wall_z + (rnd() - 0.5) * 1.0)
    } else {
        sash.rotation().y = if c.dz > 0 { PI } else { 0.0 };
        (wall_x + (rnd() - 0.5) * 1.0, wall_z)
    };
    let back = out - 0.010;
    sash.position().set(mx - c.dx as f64 * back, SASH_T * lean_a.dsin(), mz - c.dz as f64 * back);
    g.world.add(&sash);
    // 30% chance: a damp streak down the wall behind it, from above
    // to the floor, soft and uneven, a tide line at its foot
    let mut has_streak_under_window = false;
    if rnd() < 0.30 {
        let ws_color = by_weather(g.weather_key(), 0x1a2028, 0x1a1408, 0x180a08, 0x1a1610, 0x181410, 0x181410);
        let ws_w = 0.07 + rnd() * 0.05;
        let ws_h = 1.0 + rnd() * 0.2;
        let ws_mat = lambert!({ color: 0xffffff, map: &sash_streak_tex(ws_color), transparent: true, depthWrite: false, polygonOffset: true, polygonOffsetFactor: -2, polygonOffsetUnits: -2 });
        let ws = mesh(plane_geo!(ws_w * 2.2, ws_h), &ws_mat);
        ws.rotation().y = sash.rot().y;
        ws.position().set(mx + c.dx as f64 * 0.006, 1.4 - 0.3 - ws_h / 2.0 + 0.05, mz + c.dz as f64 * 0.006);
        g.world.add(&ws);
        has_streak_under_window = true;
    }
    // 40% chance (if streak present): the wet at its foot, soaked into the floor
    if has_streak_under_window && rnd() < 0.40 {
        let (wp_color, _) = by_weather(
            g.weather_key(),
            (0x1a2830, 0x08101a),
            (0x201808, 0x100804),
            (0x281010, 0x100404),
            (0x181410, 0x080604),
            (0x101820, 0x040810),
            (0x101820, 0x040810),
        );
        let r = 0.13 + rnd() * 0.08;
        let wet = mesh(
            plane_geo!(r * 2.6, r * 1.8),
            lambert!({ color: 0xffffff, map: &sash_wet_tex(wp_color), transparent: true, depthWrite: false, polygonOffset: true, polygonOffsetFactor: -2, polygonOffsetUnits: -2 }),
        );
        wet.rotation().x = -PI / 2.0;
        wet.rotation().z = rnd() * PI * 2.0;
        wet.position().set(mx - c.dx as f64 * 0.16, 0.002, mz - c.dz as f64 * 0.16);
        g.world.add(&wet);
    }
    // 4-7 shards of its glass on the floor in front of it
    let ws_count = 4 + (rnd() * 4.0).floor() as i32;
    for _sh in 0..ws_count {
        let (w, h) = (0.014 + rnd() * 0.026, 0.018 + rnd() * 0.028);
        let wshard = mesh(floor_shard_geo(w, h), &glass);
        wshard.rotation().y = rnd() * PI * 2.0;
        // on the room's side of the wall: half a turn about the way out
        let sh_ang = (rnd() - 0.5) * PI;
        let sh_rad = 0.12 + rnd() * 0.45;
        let (ox, oz) = (-c.dx as f64, -c.dz as f64);
        let (ca, sa) = (sh_ang.dcos(), sh_ang.dsin());
        wshard.position().set(mx + (ox * ca - oz * sa) * sh_rad, 0.0, mz + (ox * sa + oz * ca) * sh_rad);
        g.world.add(&wshard);
    }
}

/// The sash (`r09`): its width, height and thickness, m
const SASH_W: f64 = 0.60;

const SASH_H: f64 = 0.70;

const SASH_T: f64 = 0.044;

/// How far it leans back against the wall, radians
const SASH_LEAN: f64 = 0.21;

/// Window glass: thin, a little green at its edges, dusty; it catches the light
fn sash_glass_mat(tint: u32) -> Mat {
    // (the weather's tint of the shards, darkened: glass shows what is behind it)
    let ch = |sh: u32| ((tint >> sh) & 255) * 7 / 10;
    let color = (ch(16) << 16) | (ch(8) << 8) | ch(0);
    phong!({ color: color, specular: 0x6a7472, shininess: 90, transparent: true, opacity: 0.24, depthWrite: false, side: DoubleSide })
}

/// A shard still in the putty at `edge` (0 top, 1 bottom, 2 left, 3 right)
/// of a light: its foot buried in the rebate, its point into the light,
/// jagged; 3 mm of glass on the glass's plane
fn sash_shard_geo(size: f64, edge: i32, along: f64, lean: f64) -> Geo {
    // the four lights' sight openings (`r09`'s `sash_lights`)
    let (x0, x1, y0, y1) = (-SASH_W / 2.0 + 0.050, SASH_W / 2.0 - 0.050, 0.080, SASH_H - 0.050);
    let (xm, ym, hb) = ((x0 + x1) / 2.0, (y0 + y1) / 2.0, 0.011);
    let half = (along >= 0.5) as usize;
    // the outer edge of a light, or (leaning far) its edge on the glazing bar
    let bar = lean.abs() > 0.3;
    // (the edge as a line: its start, its direction along it, inward)
    let (sx, sy, tx, ty, nx, ny, len) = match (edge, bar) {
        (0, false) => (if half == 0 { x0 } else { xm + hb }, y1, 1.0, 0.0, 0.0, -1.0, xm - hb - x0),
        (0, true) => (if half == 0 { x0 } else { xm + hb }, ym - hb, 1.0, 0.0, 0.0, -1.0, xm - hb - x0),
        (1, false) => (if half == 0 { x0 } else { xm + hb }, y0, 1.0, 0.0, 0.0, 1.0, xm - hb - x0),
        (1, true) => (if half == 0 { x0 } else { xm + hb }, ym + hb, 1.0, 0.0, 0.0, 1.0, xm - hb - x0),
        (2, false) => (x0, if half == 0 { y0 } else { ym + hb }, 0.0, 1.0, 1.0, 0.0, ym - hb - y0),
        (2, true) => (xm + hb, if half == 0 { y0 } else { ym + hb }, 0.0, 1.0, 1.0, 0.0, ym - hb - y0),
        (_, false) => (x1, if half == 0 { y0 } else { ym + hb }, 0.0, 1.0, -1.0, 0.0, ym - hb - y0),
        (_, true) => (xm - hb, if half == 0 { y0 } else { ym + hb }, 0.0, 1.0, -1.0, 0.0, ym - hb - y0),
    };
    let f = 0.12 + 0.76 * ((along * 2.0).fract());
    let (bw, depth) = (size * (0.7 + 0.8 * lean.abs()), size * (1.0 + 0.6 * (0.5 - lean.abs())));
    // its outline in (along the edge, into the light), from under the putty
    let reb = 0.009;
    let a0 = (f * len - bw / 2.0).max(0.004);
    let a1 = (a0 + bw).min(len - 0.004);
    let tip = (a0 + a1) / 2.0 + lean * size * 1.2;
    let pts = [
        (a0, -reb),
        (a1, -reb),
        (a1 - bw * 0.08, depth * 0.35),
        (a1 - bw * (0.25 + lean * 0.3), depth * 0.62),
        (tip, depth),
        (a0 + bw * 0.12, depth * 0.48),
    ];
    let z0 = -0.0125;
    let to_sash = |a: f64, d: f64| (sx + tx * a + nx * d, sy + ty * a + ny * d);
    let p2: Vec<(f64, f64)> = pts.iter().map(|&(a, d)| to_sash(a, d)).collect();
    slab_geo(&p2, z0, z0 + 0.003)
}

/// A shard on the floor, w by h, lying flat: an irregular five-sided piece
fn floor_shard_geo(w: f64, h: f64) -> Geo {
    let k = (w * 977.0 + h * 613.0).fract();
    let pts = [
        (-w / 2.0, -h / 2.0 + h * 0.1 * k),
        (w * (0.2 + 0.25 * k), -h / 2.0),
        (w / 2.0, h * (0.1 - 0.2 * k)),
        (w * 0.1, h / 2.0),
        (-w / 2.0 + w * 0.15 * k, h * 0.2),
    ];
    // (x across, its other side along z, from the floor up 2.5 mm)
    let g = slab_geo(&pts, 0.0, 0.0025);
    g.rotate_x(-PI / 2.0);
    g.translate(0, 0, 0);
    g
}

/// A flat convex piece: the outline (x, y) between z0 and z1, its edges
fn slab_geo(pts: &[(f64, f64)], z0: f64, z1: f64) -> Geo {
    let n = pts.len();
    let mut pos = vec![];
    let mut idx = vec![];
    for &z in &[z0, z1] {
        for &(x, y) in pts {
            pos.extend_from_slice(&[x as f32, y as f32, z as f32]);
        }
    }
    for i in 1..n - 1 {
        idx.extend_from_slice(&[0, (i + 1) as u32, i as u32]);
        idx.extend_from_slice(&[n as u32, (n + i) as u32, (n + i + 1) as u32]);
    }
    let base = pos.len() as u32 / 3;
    for i in 0..n {
        let j = (i + 1) % n;
        let (a, b) = (pts[i], pts[j]);
        for &(x, y, z) in &[(a.0, a.1, z0), (b.0, b.1, z0), (b.0, b.1, z1), (a.0, a.1, z1)] {
            pos.extend_from_slice(&[x as f32, y as f32, z as f32]);
        }
        let q = base + i as u32 * 4;
        idx.extend_from_slice(&[q, q + 1, q + 2, q, q + 2, q + 3]);
    }
    let g = Geo::new();
    g.set_position(pos);
    g.set_index(idx);
    g.compute_vertex_normals();
    g
}

/// The sash's paint (u across it, v up it): white gloss gone cream and
/// grey, crazed, peeled in flakes to the grey wood, most of all on the
/// bottom rail where the water stood; dirt in the corners of the lights
fn sash_paint_tex() -> Tex {
    canvas_tex_opt(256, 256, false, |ctx, w, h| {
        // the paint: white gloss gone cream and grey, uneven
        ctx.set_fill_style("#bfb7a1");
        ctx.fill_rect(0, 0, w, h);
        for _ in 0..40 {
            let (x, y, r) = (rnd() * w, rnd() * h, 10.0 + rnd() * 40.0);
            let gr = ctx.create_radial_gradient(x, y, 0, x, y, r);
            gr.add_color_stop(0, &format!("rgba(150,140,118,{:.3})", 0.1 + rnd() * 0.2));
            gr.add_color_stop(1, "rgba(150,140,118,0)");
            ctx.set_fill_style(&gr);
            ctx.begin_path();
            ctx.arc(x, y, r, 0, 7);
            ctx.fill();
        }
        // crazed: a net of fine cracks
        ctx.set_stroke_style("rgba(70,62,52,0.4)");
        ctx.set_line_width(0.7);
        for _ in 0..260 {
            let (mut x, mut y) = (rnd() * w, rnd() * h);
            ctx.begin_path();
            ctx.move_to(x, y);
            for _ in 0..3 {
                let vert = rnd() < 0.6;
                x += if vert { (rnd() - 0.5) * 3.0 } else { (rnd() - 0.5) * 14.0 };
                y += if vert { (rnd() - 0.5) * 14.0 } else { (rnd() - 0.5) * 3.0 };
                ctx.line_to(x, y);
            }
            ctx.stroke();
        }
        // the bottom rail, where the water stood: the paint mostly gone, the
        // grey wood bare and checked along its grain, islands of paint left
        let (stile, bot, top) = (0.050 / SASH_W, 0.080 / SASH_H, 0.050 / SASH_H);
        let rail_y = (1.0 - bot * 1.15) * h;
        ctx.set_fill_style("#8c8576");
        ctx.fill_rect(0, rail_y, w, h - rail_y);
        for _ in 0..80 {
            ctx.set_fill_style(format!("rgba(64,58,50,{:.3})", 0.15 + rnd() * 0.3));
            ctx.fill_rect(rnd() * w, rail_y + rnd() * (h - rail_y), 10.0 + rnd() * 50.0, 0.7);
        }
        for _ in 0..26 {
            let (x, y) = (rnd() * w, rail_y + rnd() * (h - rail_y));
            let (rx, ry) = (3.0 + rnd() * 12.0, 1.0 + rnd() * 3.0);
            let mut pts = vec![];
            for i in 0..10 {
                let a = i as f64 / 10.0 * PI * 2.0;
                let k = 0.6 + rnd() * 0.5;
                pts.push((x + a.dcos() * rx * k, y + a.dsin() * ry * k));
            }
            for (col, dy) in [("rgba(40,34,28,0.45)", 0.8), ("#c3bba5", 0.0)] {
                ctx.set_fill_style(col);
                ctx.begin_path();
                for (i, &(px, py)) in pts.iter().enumerate() {
                    if i == 0 {
                        ctx.move_to(px, py + dy);
                    } else {
                        ctx.line_to(px, py + dy);
                    }
                }
                ctx.close_path();
                ctx.fill();
            }
        }
        // elsewhere flakes gone along each member's grain (up the stiles and
        // the bar, across the rails), a shadow under the lifted edge
        for _ in 0..70 {
            let (u, v) = (rnd(), bot * 1.15 + rnd() * (1.0 - bot * 1.15));
            let across = v > 1.0 - top || (v - (bot + (1.0 - top)) / 2.0).abs() < 0.02;
            let along_u = across && !(u < stile || u > 1.0 - stile || (u - 0.5).abs() < 0.02);
            let (x, y) = (u * w, (1.0 - v) * h);
            let r = 1.2 + rnd() * 3.5;
            let (ex, ey) = if along_u { (2.8, 0.6) } else { (0.6, 2.8) };
            let mut pts = vec![];
            for i in 0..8 {
                let a = i as f64 / 8.0 * PI * 2.0;
                let rr = r * (0.5 + rnd() * 0.7);
                pts.push((x + a.dcos() * rr * ex, y + a.dsin() * rr * ey));
            }
            for (col, d) in [("rgba(60,52,44,0.35)", 0.8), ("#8f887a", 0.0)] {
                ctx.set_fill_style(col);
                ctx.begin_path();
                for (i, &(px, py)) in pts.iter().enumerate() {
                    if i == 0 {
                        ctx.move_to(px + d, py + d);
                    } else {
                        ctx.line_to(px + d, py + d);
                    }
                }
                ctx.close_path();
                ctx.fill();
            }
        }
        // grime: settled along the bottom rail, run down from the corners
        let gr = ctx.create_linear_gradient(0, h, 0, h * 0.75);
        gr.add_color_stop(0, "rgba(54,46,36,0.5)");
        gr.add_color_stop(1, "rgba(54,46,36,0)");
        ctx.set_fill_style(&gr);
        ctx.fill_rect(0, h * 0.75, w, h * 0.25);
        // mould in the corners of the lights (their sight corners, in the texture)
        let (sx0, sx1) = ((0.050 / SASH_W) * w, (1.0 - 0.050 / SASH_W) * w);
        let (sy0, sy1) = (h - (0.080 / SASH_H) * h, (0.050 / SASH_H) * h);
        let (smx, smy) = ((sx0 + sx1) / 2.0, (sy0 + sy1) / 2.0);
        for (cx, cy) in [(sx0, sy0), (smx, sy0), (sx1, sy0), (sx0, smy), (smx, smy), (sx1, smy), (sx0, sy1), (smx, sy1), (sx1, sy1)] {
            for _ in 0..14 {
                let (x, y, r) = (cx + (rnd() - 0.5) * 18.0, cy + (rnd() - 0.5) * 18.0, 0.6 + rnd() * 2.2);
                ctx.set_fill_style(format!("rgba(36,34,26,{:.3})", 0.2 + rnd() * 0.4));
                ctx.begin_path();
                ctx.arc(x, y, r, 0, 7);
                ctx.fill();
            }
        }
    })
}

/// A damp streak run down a wall: drips from above, soft-edged, darker
/// where it dried at the foot (the tide line); transparent round it
fn sash_streak_tex(col: u32) -> Tex {
    let (r, gg, b) = ((col >> 16) & 255, (col >> 8) & 255, col & 255);
    canvas_tex_opt(64, 256, false, move |ctx, w, h| {
        ctx.clear_rect(0, 0, w, h);
        for _ in 0..9 {
            let x = w * (0.25 + rnd() * 0.5);
            let len = h * (0.5 + rnd() * 0.5);
            let wid = 2.0 + rnd() * 7.0;
            let gr = ctx.create_linear_gradient(0, 0, 0, len);
            gr.add_color_stop(0, &format!("rgba({r},{gg},{b},0)"));
            gr.add_color_stop(0.2, &format!("rgba({r},{gg},{b},{:.3})", 0.15 + rnd() * 0.15));
            gr.add_color_stop(1, &format!("rgba({r},{gg},{b},{:.3})", 0.25 + rnd() * 0.2));
            ctx.set_fill_style(&gr);
            ctx.begin_path();
            ctx.move_to(x - wid * 0.3, 0);
            ctx.quadratic_curve_to(x - wid, len * 0.5, x - wid * 0.4, len);
            ctx.arc(x, len, wid * 0.45, PI, 0);
            ctx.quadratic_curve_to(x + wid, len * 0.5, x + wid * 0.3, 0);
            ctx.close_path();
            ctx.fill();
        }
        // the tide line where it stood and dried
        ctx.save();
        ctx.translate(w / 2.0, h - 8.0);
        ctx.scale(1, 0.25);
        let gr = ctx.create_radial_gradient(0, 0, 0, 0, 0, w * 0.5);
        gr.add_color_stop(0, &format!("rgba({r},{gg},{b},0.35)"));
        gr.add_color_stop(1, &format!("rgba({r},{gg},{b},0)"));
        ctx.set_fill_style(&gr);
        ctx.begin_path();
        ctx.arc(0, 0, w * 0.5, 0, 7);
        ctx.fill();
        ctx.restore();
    })
}

/// The wet soaked into the floor: a soft, uneven dark patch, a darker rim
/// where it is drying
fn sash_wet_tex(col: u32) -> Tex {
    let (r, gg, b) = ((col >> 16) & 255, (col >> 8) & 255, col & 255);
    canvas_tex_opt(128, 128, false, move |ctx, w, h| {
        ctx.clear_rect(0, 0, w, h);
        for k in 0..7 {
            let (x, y) = (w / 2.0 + (rnd() - 0.5) * w * 0.35, h / 2.0 + (rnd() - 0.5) * h * 0.35);
            let rr = w * (0.18 + rnd() * 0.18) * if k == 0 { 1.4 } else { 1.0 };
            let gr = ctx.create_radial_gradient(x, y, 0, x, y, rr);
            gr.add_color_stop(0, &format!("rgba({r},{gg},{b},0.4)"));
            gr.add_color_stop(0.75, &format!("rgba({r},{gg},{b},0.32)"));
            gr.add_color_stop(1, &format!("rgba({r},{gg},{b},0)"));
            ctx.set_fill_style(&gr);
            ctx.begin_path();
            ctx.arc(x, y, rr, 0, 7);
            ctx.fill();
        }
    })
}
