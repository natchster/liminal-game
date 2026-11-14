//! `place_wall_handprints`
use game::prelude::*;
use placers_kit::noise::fbm;
use placers_kit::noise::hash as hash01;
use placers_kit::noise::smin as smin_hand;
use placers_kit::noise::sstep;
use placers_kit::noise::vn as vnoise;
use placers_kit::place::pick;
use placers_kit::place::wall_candidates as wall_cands;

// Wall handprints: very rare (5% chance per level). The grime a hand
// left on a wall (`assets/src/blender/props/r11a.notes.md`): someone leaned
// on it at a hand's height, or put a hand out going down a corridor in the
// dark; dirt, soot and the skin's oil came off where the hand pressed (the
// finger pads, the pads under the fingers, the heel of the hand), not where
// it arched (the hollow of the palm, the creases), and smeared as it slid
// away. Soft, partial, uneven; a dragged hand leaves its fingers' tracks
// thinning out along the wall. Painted on transparent planes the size of the
// old ones.
pub(crate) fn place_wall_handprints(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.95 {
        return;
    } // 5% chance
    let cands = wall_cands(g);
    if cands.is_empty() {
        return;
    }
    let c = pick(&cands);
    let grime = grime_by_weather(g.weather_key());
    let vary = hand_vary(c.x * 4 + c.dx, c.z * 4 + c.dz);
    // which hand it was (the second print, beside it, the other one)
    let right = vary & 1 == 0;
    // 12% chance the print is actually an extreme drag streak.
    let is_drag = rnd() < 0.12;
    if is_drag {
        let drag_mat = hand_mat(hand_drag_tex(right, grime, vary));
        let drag = mesh(plane_geo!(0.60, 0.30), &drag_mat);
        let wall_xd = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.008);
        let wall_zd = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.008);
        let drag_height = 1.0 + rnd() * 0.5;
        let wall_j = (rnd() - 0.5) * 0.8;
        let (drag_pos_x, drag_pos_z);
        if c.dx.abs() == 1 {
            drag.rotation().y = if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 };
            drag_pos_x = wall_xd;
            drag_pos_z = wall_zd + wall_j;
            drag.position().set(drag_pos_x, drag_height, drag_pos_z);
        } else {
            drag.rotation().y = if c.dz > 0 { PI } else { 0.0 };
            drag_pos_x = wall_xd + wall_j;
            drag_pos_z = wall_zd;
            drag.position().set(drag_pos_x, drag_height, drag_pos_z);
        }
        drag.rotation().z = (rnd() - 0.5) * 0.15;
        g.world.add(&drag);
        // 20% chance a full handprint at the drag's start edge.
        if rnd() < 0.20 {
            let start_hand = mesh(plane_geo!(0.15, 0.22), &hand_mat(hand_print_tex(right, grime, vary ^ 0x5a)));
            let dr = drag.rot();
            start_hand.rotation().y = dr.y;
            start_hand.rotation().z = dr.z;
            // Offset toward the drag's left (leftmost = start of drag texture).
            let right_vec_x = (dr.y + PI / 2.0).sin();
            let right_vec_z = (dr.y + PI / 2.0).cos();
            start_hand.position().set(drag_pos_x - right_vec_x * 0.28, drag_height, drag_pos_z - right_vec_z * 0.28);
            g.world.add(&start_hand);
        }
        return;
    }
    // 20% chance downward-facing print (someone leaned) vs default upward.
    let is_downward = rnd() < 0.20;
    let make_hand = |tilt_z: f64, right: bool, k: u32| -> Obj {
        let h = mesh(plane_geo!(0.15, 0.22), &hand_mat(hand_print_tex(right, grime, vary ^ k)));
        // Downward: turned a half turn, so the fingers point down.
        if is_downward {
            h.rotation().z = PI + tilt_z;
        } else {
            h.rotation().z = tilt_z;
        }
        h
    };
    // Handprint wall-stick fix: 15 mm off the wall face; jitter only where the wall continues.
    let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.015);
    let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.015);
    let hand_height = if is_downward { 0.7 + rnd() * 0.4 } else { 1.1 + rnd() * 0.6 };
    let wall_y = if c.dz > 0 { PI } else { 0.0 };
    let rot_y = if c.dx.abs() == 1 {
        if c.dx > 0 {
            -PI / 2.0
        } else {
            PI / 2.0
        }
    } else {
        wall_y
    };
    let (wall_tx, wall_tz) = (c.x + c.dx, c.z + c.dz);
    let mut min_j = -CELL * 0.42;
    let mut max_j = CELL * 0.42;
    if c.dx.abs() == 1 {
        if !g.in_grid(wall_tx, wall_tz - 1) || g.tiles[g.t_idx(wall_tx, wall_tz - 1)] != 1 {
            min_j = -0.45;
        }
        if !g.in_grid(wall_tx, wall_tz + 1) || g.tiles[g.t_idx(wall_tx, wall_tz + 1)] != 1 {
            max_j = 0.45;
        }
    } else {
        if !g.in_grid(wall_tx - 1, wall_tz) || g.tiles[g.t_idx(wall_tx - 1, wall_tz)] != 1 {
            min_j = -0.45;
        }
        if !g.in_grid(wall_tx + 1, wall_tz) || g.tiles[g.t_idx(wall_tx + 1, wall_tz)] != 1 {
            max_j = 0.45;
        }
    }
    let jitter = min_j + rnd() * (max_j - min_j);
    let pos_x = if c.dx.abs() == 1 { wall_x } else { wall_x + jitter };
    let pos_z = if c.dx.abs() == 1 { wall_z + jitter } else { wall_z };
    let tilt_z = (rnd() - 0.5) * 0.4;
    let hand = make_hand(tilt_z, right, 0);
    hand.rotation().y = rot_y;
    hand.position().set(pos_x, hand_height, pos_z);
    g.world.add(&hand);
    // 15% chance second handprint 0.3-0.5 m along the wall: the other hand.
    if rnd() < 0.15 {
        let spacing = 0.3 + rnd() * 0.2;
        let dir = if rnd() < 0.5 { 1.0 } else { -1.0 };
        // Clamp second-print offset so combined jitter+spacing stays on wall.
        let combined = clamp(jitter + dir * spacing, min_j, max_j);
        let tilt_z2 = (rnd() - 0.5) * 0.4;
        let hand2 = make_hand(tilt_z2, !right, 1);
        hand2.rotation().y = rot_y;
        if c.dx.abs() == 1 {
            hand2.position().set(pos_x, hand_height, wall_z + combined);
        } else {
            hand2.position().set(wall_x + combined, hand_height, pos_z);
        }
        g.world.add(&hand2);
    } else if rnd() < 0.05 && !is_downward {
        // 5% vertical pair variant — one high (~1.7 m), one low (~0.9 m).
        let tilt_z3 = (rnd() - 0.5) * 0.4;
        let high_hand = make_hand(tilt_z3, !right, 2);
        high_hand.rotation().y = rot_y;
        high_hand.position().set(pos_x, 1.7, pos_z);
        g.world.add(&high_hand);
        // Original hand also at 0.9 m for pair — override position.
        hand.position().y = 0.9;
    }
    // 30% chance a stranger's chalk or a balled-up sheet near the handprint.
    if rnd() < 0.30 {
        if rnd() < 0.5 {
            // a date chalked beside it, someone's note of when they saw it
            // (laid in upright strips under 8 cm wide, each where the old
            // stroke was: settling finds the same wall face for them)
            let chalk_spacing = (if rnd() < 0.5 { -1.0 } else { 1.0 }) * (0.30 + rnd() * 0.20);
            let chalk_h = hand_height + (rnd() - 0.5) * 0.3;
            let tex = chalk_date_tex(vary);
            let chalk_mat = lambert!({
                map: &tex,
                color: 0xdcd8ce,
                transparent: true,
                depthWrite: false,
                polygonOffset: true,
                polygonOffsetFactor: -2,
                polygonOffsetUnits: -2
            });
            let (cx, cz) = if c.dx.abs() == 1 { (pos_x, pos_z + chalk_spacing) } else { (pos_x + chalk_spacing, pos_z) };
            let mark = group();
            let n = (CHALK_DATE_W / 0.07).ceil() as usize;
            let sw = CHALK_DATE_W / n as f64;
            for k in 0..n {
                let geo = plane_geo!(sw, CHALK_DATE_H);
                let (u0, u1) = (k as f32 / n as f32, (k + 1) as f32 / n as f32);
                geo.set_uv(vec![u0, 1.0, u1, 1.0, u0, 0.0, u1, 0.0]);
                geo.translate((k as f64 + 0.5) * sw - CHALK_DATE_W / 2.0, 0, 0);
                let strip = mesh(geo, &chalk_mat);
                strip.rotation().y = rot_y;
                strip.position().set(cx, chalk_h, cz);
                mark.add(&strip);
            }
            g.world.add(&mark);
        } else {
            // a sheet balled up and dropped (batch r08's), its old size picking
            // a tight ball or a loose wad
            let sz = 0.06 + rnd() * 0.05;
            let u = (sz - 0.06) / 0.05;
            let name = if u < 0.5 { "Real__paperBall" } else { "Real__paperWad" };
            let geo = game::assets::props::prop_part_as_modelled(name).unwrap_or_else(|| box_geo!(sz, 0.04, sz));
            let scrap = mesh(geo, &scrap_ball_mat());
            scrap.scale().set_scalar(0.9 + 0.2 * (u * 2.0).fract());
            scrap.position().set(pos_x + (rnd() - 0.5) * 0.3, 0.0, pos_z + (rnd() - 0.5) * 0.3);
            // Nudge slightly away from wall to avoid clipping.
            if c.dx.abs() == 1 {
                scrap.position().x -= c.dx as f64 * 0.2;
            } else {
                scrap.position().z -= c.dz as f64 * 0.2;
            }
            scrap.rotation().y = rnd() * PI * 2.0;
            g.world.add(&scrap);
        }
    }
    // 5% chance the handprint FADES cyclically.
    if rnd() < 0.05 {
        let fade_mat = hand.mat();
        let base_opacity = fade_mat.opacity_v();
        let fade_phase = rnd() * PI * 2.0;
        g.animate(move |_g, _dt, t| {
            // 30 s period: opacity oscillates base_opacity ↔ 0.15.
            let frac = ((t * (PI / 15.0) + fade_phase).sin() + 1.0) * 0.5;
            fade_mat.m().opacity = 0.15 + (base_opacity - 0.15) * frac;
        });
    }
}

game::lazy_static_tex!(
    scrap_ball_tex,
    canvas_tex(128, 128, |ctx, w, h| {
        ctx.set_fill_style("#ddd8cb");
        ctx.fill_rect(0, 0, w, h);
        for _ in 0..90 {
            let (x, y) = (rnd() * w, rnd() * h);
            let v = 200.0 + rnd() * 34.0;
            ctx.set_fill_style(format!("rgba({v:.0},{:.0},{:.0},0.2)", v - 4.0, v - 14.0));
            ctx.fill_rect(x, y, 2.0 + rnd() * 7.0, 2.0 + rnd() * 7.0);
        }
        // patches of type, each a facet turned its own way
        for _ in 0..10 {
            ctx.save();
            ctx.translate(rnd() * w, rnd() * h);
            ctx.rotate(rnd() * PI * 2.0);
            ctx.set_fill_style(format!("rgba(40,38,36,{:.2})", 0.25 + rnd() * 0.25));
            for l in 0..4 {
                let mut xx = -18.0;
                while xx < 18.0 {
                    let ww = 2.0 + rnd() * 6.0;
                    ctx.fill_rect(xx, l as f64 * 4.0 - 8.0, ww, 1.6);
                    xx += ww + 1.5;
                }
            }
            ctx.restore();
        }
        ctx.set_line_width(0.7);
        for _ in 0..30 {
            let (x, y) = (rnd() * w, rnd() * h);
            let a = rnd() * PI;
            let l = 8.0 + rnd() * 30.0;
            ctx.set_stroke_style(format!("rgba(70,64,52,{:.2})", 0.1 + rnd() * 0.12));
            ctx.begin_path();
            ctx.move_to(x, y);
            ctx.line_to(x + a.cos() * l, y + a.sin() * l);
            ctx.stroke();
        }
    })
);

/// The chalked date's plane (m): as tall as the old stroke, so its middle is
/// where the old one's was
const CHALK_DATE_H: f64 = 0.12;

const CHALK_DATE_W: f64 = 0.13;

/// The grime a hand leaves on a wall in each weather (sRGB): dirt and the
/// skin's oil, mud where it is wet, soot where things burnt or sparked, dust
/// where everything is dust, and a rust-dark brown where the floor is wrong
fn grime_by_weather(key: &str) -> [f64; 3] {
    by_weather(key, [0.26, 0.21, 0.15], [0.15, 0.14, 0.13], [0.27, 0.15, 0.12], [0.2, 0.18, 0.16], [0.36, 0.34, 0.31], [0.24, 0.2, 0.16])
}

/// A number that differs from one placing to the next (from where it was
/// put), to vary what the painters draw: a painter's own stream starts the
/// same on every floor
fn hand_vary(a: i32, b: i32) -> u32 {
    (hash01(0x2981_6c35, a, b) * 4.0e9) as u32
}

/// The decal material of a mark painted in `tex`, lit as the wall under it
fn hand_mat(tex: Tex) -> Mat {
    lambert!({ map: &tex, transparent: true, depthWrite: false, polygonOffset: true, polygonOffsetFactor: -2, polygonOffsetUnits: -2 })
}

/// The shape of a hand pressed flat, fingers up (m, x across, y up, its
/// middle at the plane's middle; a right hand's thumb on the left, as its
/// print is seen): the signed distance to it, and how hard each part pressed
/// (the finger pads and the pads under them, the heel of the hand; not the
/// hollow of the palm or the creases), 0..1
fn hand_press(x: f64, y: f64, right: bool, spread: f64) -> (f64, f64) {
    let x = if right { x } else { -x };
    // a capsule from a to b, radius r: (distance, how far along 0..1)
    let cap = |a: (f64, f64), b: (f64, f64), r: f64| -> (f64, f64) {
        let (ex, ey) = (b.0 - a.0, b.1 - a.1);
        let t = (((x - a.0) * ex + (y - a.1) * ey) / (ex * ex + ey * ey)).clamp(0.0, 1.0);
        ((x - a.0 - ex * t).hypot(y - a.1 - ey * t) - r, t)
    };
    // the palm: a rounded block narrowing to the wrist
    let (pcx, pcy) = (0.002, -0.042);
    let half_w = 0.04 - 0.008 * ((pcy - y) / 0.05).clamp(0.0, 1.0);
    let qx = (x - pcx).abs() - half_w + 0.014;
    let qy = (y - pcy).abs() - 0.047 + 0.014;
    let palm = qx.max(0.0).hypot(qy.max(0.0)) + qx.max(qy).min(0.0) - 0.014;
    let mut d = palm;
    let mut press = 0.0f64;
    let fingers = [
        ((-0.028, 0.0), 0.069, 0.0094, -0.14),
        ((-0.009, 0.004), 0.078, 0.0098, -0.03),
        ((0.011, 0.002), 0.072, 0.0092, 0.07),
        ((0.029, -0.006), 0.057, 0.0079, 0.18),
    ];
    let blob = |cx: f64, cy: f64, sx: f64, sy: f64| (-(((x - cx) / sx).powi(2) + ((y - cy) / sy).powi(2))).exp();
    if palm < 0.0 {
        // the heel of the hand (the ball of the thumb, and the little
        // finger's side), the pads under the fingers; the hollow between
        // them touched lightly; the creases lighter still
        let heel = blob(-0.017, -0.068, 0.017, 0.021).max(blob(0.022, -0.07, 0.012, 0.022));
        let pads = fingers.iter().map(|&((bx, by), ..)| blob(bx, by - 0.008, 0.009, 0.0075)).fold(0.0f64, f64::max);
        let hollow = 1.0 - 0.6 * blob(0.004, -0.035, 0.018, 0.016);
        let crease = |a: (f64, f64), b: (f64, f64)| 0.35 + 0.65 * sstep(0.0005, 0.0018, cap(a, b, 0.0).0);
        let creases =
            crease((-0.036, -0.058), (0.004, -0.02)) * crease((-0.04, -0.03), (0.035, -0.018)) * crease((-0.02, -0.012), (0.04, -0.008));
        press = (0.32 * hollow).max(0.85 * heel.max(pads)) * creases;
    }
    // the fingers: (base, length, radius, lean); the thumb from the palm's side
    for (k, &((bx, by), len, r, lean)) in fingers.iter().enumerate() {
        let a = lean * (0.6 + 0.8 * spread) + (k as f64 - 1.5) * 0.02 * spread;
        let tip = (bx + a.sin() * len, by + a.cos() * len);
        let (fd, t) = cap((bx, by), tip, r);
        d = smin_hand(d, fd, 0.004);
        if fd < 0.0 {
            // the pad at the tip pressed hardest, the joints' creases lighter
            let joint = |at: f64| 0.55 + 0.45 * sstep(0.0015, 0.005, ((t - at) * len).abs());
            let p = (0.6 + 0.4 * sstep(0.62, 0.8, t)) * joint(0.36) * joint(0.66);
            press = press.max(p * sstep(0.0, -0.002, fd));
        }
    }
    // the thumb: short and broad, its root in the ball of the thumb
    let (td, t) = cap((-0.027, -0.066), (-0.06 - 0.008 * spread, -0.03 + 0.004 * spread), 0.0122);
    d = smin_hand(d, td, 0.008);
    if td < 0.0 {
        press = press.max(
            (0.5 + 0.4 * sstep(0.5, 0.85, t)) * (0.6 + 0.4 * sstep(0.0015, 0.005, ((t - 0.55) * 0.05).abs())) * sstep(0.0, -0.002, td),
        );
    }
    (d, press)
}

/// A hand's print in grime on a plane 0.15 × 0.22 m: pressed unevenly
/// (heavier on one side, sometimes only the fingers or only the heel of the
/// hand), mottled as dirt transfers, slid a little as it came away
fn hand_print_tex(right: bool, grime: [f64; 3], vary: u32) -> Tex {
    hand_tex(120.0, 176.0, grime, vary, move |ctx, w, h| {
        let seed = (rnd() * 4.0e9) as u32;
        let s = 0.15 / w;
        let spread = rnd();
        // how it was pressed: across it (a lean), and which part touched
        let (tx, ty) = ((rnd() - 0.5) * 2.0, (rnd() - 0.5) * 2.0);
        let part = rnd();
        let load = 0.55 + 0.35 * rnd();
        // the slide as it came away: down and to one side, a centimetre or two
        let slide = (0.004 + 0.016 * rnd(), (rnd() - 0.5) * 0.6);
        let mut img = ctx.create_image_data(w, h);
        let n = w as usize;
        par_rows(&mut img.data, n * 4, |j, out| {
            let y = 0.11 - (j as f64 + 0.5) * s;
            for i in 0..n {
                let x = (i as f64 + 0.5) * s - 0.075;
                let mut a = 0.0f64;
                for k in 0..7 {
                    // back along the slide: the print, fainter as it smeared
                    let f = k as f64 / 6.0;
                    let (sx, sy) = (x - slide.1 * slide.0 * f, y + slide.0 * f);
                    let (d, p) = hand_press(sx, sy, right, spread);
                    if d > 0.002 {
                        continue;
                    }
                    let lean = 1.0 + 0.6 * (tx * sx / 0.06 + ty * sy / 0.09);
                    let which = if part < 0.6 {
                        1.0
                    } else if part < 0.85 {
                        // the fingers and the pads under them only
                        sstep(-0.03, -0.01, sy)
                    } else {
                        // the heel of the hand and the fingertips
                        sstep(-0.035, -0.055, sy).max(sstep(0.045, 0.06, sy))
                    };
                    let edge = sstep(0.003, -0.003, d);
                    a = a.max(p * lean.clamp(0.2, 1.4) * which * edge * (1.0 - 0.8 * f) * if k == 0 { 1.0 } else { 0.55 });
                }
                // dirt comes off in specks and patches, not evenly
                let mottle = (0.4 + 0.6 * sstep(0.28, 0.68, fbm(seed ^ 0x9, x * 60.0, y * 60.0, 3)))
                    * (0.65 + 0.35 * fbm(seed, x * 260.0, y * 260.0, 3));
                let speck = hash01(seed ^ 0x77, (x / 0.0011) as i32, (y / 0.0011) as i32);
                let a = (a * load * mottle).min(1.0);
                let a = a * (0.55 + 0.45 * sstep(speck - 0.4, speck + 0.2, a * 1.3));
                let px = [grime[0], grime[1], grime[2], (a * 0.85).min(0.75)];
                for (k, v) in px.iter().enumerate() {
                    out[i * 4 + k] = (v.clamp(0.0, 1.0) * 255.0).round() as u8;
                }
            }
        });
        ctx.put_image_data(&img, 0, 0);
    })
}

/// A hand dragged along a wall, on a plane 0.60 × 0.30 m: from where it was
/// pressed (its left), the palm's smear and its fingers' four tracks running
/// on, wavering, thinning and breaking up as the grime ran out
fn hand_drag_tex(right: bool, grime: [f64; 3], vary: u32) -> Tex {
    hand_tex(384.0, 192.0, grime, vary, move |ctx, w, h| {
        let seed = (rnd() * 4.0e9) as u32;
        let s = 0.6 / w;
        // the tracks: fingers 2 cm apart, each its height, width, reach and wobble
        let mid = (rnd() - 0.5) * 0.03;
        let tracks: Vec<(f64, f64, f64, f64, u32)> = (0..4)
            .map(|k| {
                let y = mid + (k as f64 - 1.5) * 0.02 * if right { 1.0 } else { -1.0 };
                (y, 0.006 + 0.002 * rnd(), 0.45 + 0.4 * rnd(), 0.85 + 0.15 * rnd(), seed.wrapping_add(k * 17))
            })
            .collect();
        let start = -0.24;
        let sag = 0.01 + 0.02 * rnd();
        let mut img = ctx.create_image_data(w, h);
        let n = w as usize;
        par_rows(&mut img.data, n * 4, |j, out| {
            let y = 0.15 - (j as f64 + 0.5) * s;
            for i in 0..n {
                let x = (i as f64 + 0.5) * s - 0.3;
                let run = ((x - start) / 0.5).max(0.0);
                let mut a = 0.0f64;
                // the palm's smear where it began
                let palm = (-(((x - start - 0.03) / 0.065).powi(2) + ((y - mid + 0.03) / 0.04).powi(2))).exp();
                a = a.max(palm * 0.85 * (0.4 + 0.6 * fbm(seed ^ 0x3, x * 60.0, y * 60.0, 3)));
                for &(ty, tw, reach, press, ts) in &tracks {
                    if x < start {
                        continue;
                    }
                    // the track sags as the arm went down, and wavers
                    let yc = ty - sag * run * run + (fbm(ts, run * 6.0, 0.5, 2) - 0.5) * 0.006;
                    let fade = (1.0 - run / reach).max(0.0);
                    let wd = tw * (0.6 + 0.4 * fade);
                    let across = sstep(wd, wd * 0.3, (y - yc).abs());
                    // broken where the finger lifted and the grime gave out
                    let broken = sstep(0.05 + 0.5 * (1.0 - fade), 0.45, vnoise(ts ^ 0x9, run * 30.0, (y - yc) * 300.0));
                    // streaked along by the skin's ridges and grit
                    let streak = 0.6 + 0.4 * vnoise(ts ^ 0x5, x * 25.0, y * 1200.0);
                    a = a.max(across * fade.powf(0.5) * press * broken * streak);
                }
                let speck = hash01(seed ^ 0x77, (x / 0.0011) as i32, (y / 0.0011) as i32);
                let a = a * (0.55 + 0.45 * sstep(speck - 0.4, speck + 0.2, a * 1.3));
                // nothing at the plane's edges
                let a = a * sstep(0.0, 0.01, (0.3 - x.abs()).min(0.15 - y.abs()));
                let px = [grime[0], grime[1], grime[2], (a * 0.95).min(0.85)];
                for (k, v) in px.iter().enumerate() {
                    out[i * 4 + k] = (v.clamp(0.0, 1.0) * 255.0).round() as u8;
                }
            }
        });
        ctx.put_image_data(&img, 0, 0);
    })
}

/// A transparent mark `w × h` px painted by `paint` on a stream of its own
/// varied by `vary`; where it is clear its colour is `bleed` (sRGB), so its
/// smaller mips keep its colour at its soft edges
fn hand_tex<F: FnOnce(&mut Canvas, f64, f64)>(w: f64, h: f64, bleed: [f64; 3], vary: u32, paint: F) -> Tex {
    let tex = canvas_tex_opt(w, h, false, move |ctx: &mut Canvas, cw: f64, ch: f64| {
        let seed = (rnd() * 4.0e9) as u32 ^ vary;
        with_own_stream(seed, || paint(ctx, cw, ch))
    });
    {
        let img = tex.image();
        let mut im = img.borrow_mut();
        for px in im.data.chunks_exact_mut(4) {
            let k = sstep(0.0, 0.3, f64::from(px[3]) / 255.0);
            for (c, b) in px.iter_mut().zip(bleed) {
                *c = (b * 255.0 + (f64::from(*c) - b * 255.0) * k).round().clamp(0.0, 255.0) as u8;
            }
        }
        im.version += 1;
    }
    tex
}

/// A date chalked quickly on a wall (day and month, underlined), on a plane
/// `CHALK_DATE_W × CHALK_DATE_H`: blackboard chalk caught on the wall's tooth,
/// broken where it skipped, its dust under it. White (the material greys it:
/// white chalk on a wall, years on)
fn chalk_date_tex(vary: u32) -> Tex {
    let (w, h) = (CHALK_DATE_W / 0.001, CHALK_DATE_H / 0.001);
    hand_tex(w.round(), h.round(), [1.0, 1.0, 1.0], vary ^ 0x302, move |ctx, w, h| {
        let seed = (rnd() * 4.0e9) as u32;
        let day = 1 + (rnd() * 28.0) as u64;
        let month = 1 + (rnd() * 12.0) as u64;
        let text = format!("{day}.{month}");
        // digits 3.5 cm tall (1 mm a px), a quick slant, an underline
        let (dh, dw) = (34.0 + rnd() * 6.0, 20.0 + rnd() * 3.0);
        let slant = 0.08 + 0.1 * rnd();
        let jig = |p: (f64, f64), k: f64| (p.0 + (rnd() - 0.5) * k, p.1 + (rnd() - 0.5) * k);
        let mut lines: Vec<Vec<(f64, f64)>> = vec![];
        let total: f64 = text.chars().map(|c| if c == '.' { dw * 0.45 } else { dw * 1.28 }).sum::<f64>() - dw * 0.28;
        let mut x0 = (w - total) / 2.0;
        let y0 = h * 0.22;
        for ch in text.chars() {
            if ch == '.' {
                let p = (x0 + dw * 0.05, y0 + dh * 0.9);
                lines.push(vec![p, (p.0 + 2.5, p.1 + 3.0), (p.0 + 4.0, p.1 + 5.5)]);
                x0 += dw * 0.45;
                continue;
            }
            for st in hand_chalk_digit(ch.to_digit(10).unwrap_or(0)) {
                lines.push(st.iter().map(|&(u, v)| jig((x0 + u * dw + (1.0 - v) * dh * slant, y0 + v * dh), 1.6)).collect());
            }
            x0 += dw * 1.28;
        }
        let ul = y0 + dh + 9.0;
        lines.push(vec![
            jig(((w - total) / 2.0 - 4.0, ul + 2.0), 1.5),
            jig((w / 2.0, ul), 1.5),
            jig(((w + total) / 2.0 + 6.0, ul - 3.0), 1.5),
        ]);
        let alpha = hand_chalk_raster(&lines, w, h, 6.0, seed);
        let mut img = ctx.create_image_data(w, h);
        for (k, a) in alpha.iter().enumerate() {
            img.data[k * 4..k * 4 + 4].copy_from_slice(&[255, 255, 255, (a.clamp(0.0, 1.0) * 255.0).round() as u8]);
        }
        ctx.put_image_data(&img, 0, 0);
    })
}

/// The strokes of a digit in a unit box (u across, v down), as a quick hand
/// draws it
fn hand_chalk_digit(d: u32) -> Vec<Vec<(f64, f64)>> {
    let arc = |cx: f64, cy: f64, rx: f64, ry: f64, a0: f64, a1: f64| -> Vec<(f64, f64)> {
        (0..=12).map(|k| a0 + (a1 - a0) * k as f64 / 12.0).map(|a| (cx + rx * a.cos(), cy + ry * a.sin())).collect()
    };
    match d {
        0 => vec![arc(0.5, 0.5, 0.48, 0.5, -1.4, 4.95)],
        1 => vec![vec![(0.22, 0.2), (0.55, 0.0), (0.55, 1.0)]],
        2 => {
            let mut s = arc(0.5, 0.28, 0.45, 0.28, 3.4, 6.6);
            s.extend([(0.05, 1.0), (1.0, 0.98)]);
            vec![s]
        }
        3 => {
            let mut s = arc(0.45, 0.25, 0.45, 0.25, 3.6, 7.6);
            s.extend(arc(0.45, 0.73, 0.5, 0.27, -1.4, 2.6));
            vec![s]
        }
        4 => vec![vec![(0.7, 1.0), (0.7, 0.0), (0.0, 0.68), (1.0, 0.68)]],
        5 => {
            let mut s = vec![(0.9, 0.0), (0.18, 0.0), (0.12, 0.43)];
            s.extend(arc(0.45, 0.68, 0.5, 0.31, -2.4, 2.7));
            vec![s]
        }
        6 => {
            let mut s = vec![(0.85, 0.05), (0.45, 0.2), (0.08, 0.62)];
            s.extend(arc(0.5, 0.72, 0.43, 0.27, 3.1, 9.4));
            vec![s]
        }
        7 => vec![vec![(0.0, 0.02), (0.95, 0.0), (0.35, 1.0)]],
        8 => {
            let mut s = arc(0.5, 0.25, 0.38, 0.25, 1.6, 7.6);
            s.extend(arc(0.5, 0.74, 0.46, 0.26, -1.6, 4.7));
            vec![s]
        }
        _ => {
            let mut s = arc(0.48, 0.3, 0.44, 0.3, 0.2, 6.6);
            s.extend([(0.9, 0.4), (0.62, 1.0)]);
            vec![s]
        }
    }
}

/// The chalk left on a wall by `lines` (px, each smoothed and pressed
/// heavier in its middle), a stick `width` px wide: how much each pixel of a
/// `w × h` canvas holds (0..1), caught on the wall's tooth, gone where it
/// skipped, its dust fallen below. No draws.
fn hand_chalk_raster(lines: &[Vec<(f64, f64)>], w: f64, h: f64, width: f64, seed: u32) -> Vec<f64> {
    let (n, m) = (w as usize, h as usize);
    let mut segs: Vec<((f64, f64), (f64, f64), f64, f64, u32)> = vec![];
    for (li, l) in lines.iter().enumerate() {
        // smoothed (Chaikin, twice), points every 3 px
        let mut p = l.clone();
        for _ in 0..2 {
            if p.len() < 3 {
                break;
            }
            let mut q = vec![p[0]];
            for k in 0..p.len() - 1 {
                let (a, b) = (p[k], p[k + 1]);
                q.push((a.0 * 0.75 + b.0 * 0.25, a.1 * 0.75 + b.1 * 0.25));
                q.push((a.0 * 0.25 + b.0 * 0.75, a.1 * 0.25 + b.1 * 0.75));
            }
            q.push(p[p.len() - 1]);
            p = q;
        }
        let mut d = vec![];
        for k in 0..p.len().saturating_sub(1) {
            let (a, b) = (p[k], p[k + 1]);
            let c = ((a.0 - b.0).hypot(a.1 - b.1) / 3.0).ceil().max(1.0) as usize;
            for i in 0..c {
                let t = i as f64 / c as f64;
                d.push((a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t));
            }
        }
        if let Some(&e) = p.last() {
            d.push(e);
        }
        let nd = d.len().max(2) - 1;
        // a dab (a full stop) pressed all along; a stroke lifting at its ends
        let dab = nd < 4;
        let press = |k: usize| {
            let t = k as f64 / nd as f64;
            if dab {
                return 0.95;
            }
            (0.8 + 0.2 * (t * 7.0 + li as f64).sin()) * (t / 0.08).clamp(0.35, 1.0) * ((1.0 - t) / 0.12).clamp(0.25, 1.0)
        };
        for k in 0..d.len().saturating_sub(1) {
            segs.push((d[k], d[k + 1], press(k), press(k + 1), seed.wrapping_add(li as u32 * 97)));
        }
    }
    let r = width / 2.0;
    // each 16 px square's segments, so a pixel looks only at those near it
    const CELL: usize = 16;
    let (cw, chh) = (n.div_ceil(CELL), m.div_ceil(CELL));
    let mut cells: Vec<Vec<u32>> = vec![vec![]; cw * chh];
    let cell = |v: f64, hi: usize| ((v / CELL as f64).floor().max(0.0) as usize).min(hi - 1);
    for (k, &(a, b, ..)) in segs.iter().enumerate() {
        let (x0, x1) = (cell(a.0.min(b.0) - r - 1.0, cw), cell(a.0.max(b.0) + r + 1.0, cw));
        let (y0, y1) = (cell(a.1.min(b.1) - r - 1.0, chh), cell(a.1.max(b.1) + r + 1.0, chh));
        for cy in y0..=y1 {
            for cx in x0..=x1 {
                cells[cy * cw + cx].push(k as u32);
            }
        }
    }
    let mut bytes = vec![0u8; n * m * 4];
    par_rows(&mut bytes, n * 4, |j, out| {
        let y = j as f64 + 0.5;
        for i in 0..n {
            let x = i as f64 + 0.5;
            let mut keep = 0.0f64;
            for &si in &cells[(j / CELL) * cw + i / CELL] {
                let (a, b, pa, pb, ls) = segs[si as usize];
                if x < a.0.min(b.0) - r - 1.0 || x > a.0.max(b.0) + r + 1.0 || y < a.1.min(b.1) - r - 1.0 || y > a.1.max(b.1) + r + 1.0 {
                    continue;
                }
                let (ex, ey) = (b.0 - a.0, b.1 - a.1);
                let l2 = (ex * ex + ey * ey).max(1e-9);
                let t = (((x - a.0) * ex + (y - a.1) * ey) / l2).clamp(0.0, 1.0);
                let d = (x - a.0 - ex * t).hypot(y - a.1 - ey * t);
                let rr = r * (0.85 + 0.3 * vnoise(ls ^ 0x5a, x * 0.35, y * 0.35));
                let cover = sstep(rr + 0.8, rr - 0.8, d);
                if cover <= 0.0 {
                    continue;
                }
                let skip = sstep(0.2, 0.5, vnoise(ls ^ 0x77, (a.0 + ex * t) * 0.11, (a.1 + ey * t) * 0.11));
                let press = (pa + (pb - pa) * t) * (0.3 + 0.7 * skip) * (1.0 - 0.45 * (d / rr).powi(2));
                keep = 1.0 - (1.0 - keep) * (1.0 - cover * press);
            }
            let grain = hash01(seed ^ 0x1d, (x / 1.3) as i32, (y / 1.3) as i32);
            let patch = vnoise(seed ^ 0x2e, x * 0.12, y * 0.12);
            let tooth = 0.7 * grain + 0.3 * patch;
            // chalk on a wall holds only on the tooth's peaks
            let a = sstep(tooth - 0.06, tooth + 0.05, keep * 0.82);
            out[i * 4] = (a * 255.0).round() as u8;
            out[i * 4 + 1] = (keep * 255.0).round() as u8;
        }
    });
    let mut out: Vec<f64> = (0..n * m).map(|k| f64::from(bytes[k * 4]) / 255.0).collect();
    // the dust: what fell from each stroke, down the wall under it, fading
    for i in 0..n {
        let mut acc = 0.0f64;
        for j in 0..m {
            let k = j * n + i;
            let held = f64::from(bytes[k * 4 + 1]) / 255.0;
            acc = acc * 0.93 + held * 0.1;
            let speck = hash01(seed ^ 0x3f, i as i32, j as i32);
            let dust = acc * (0.12 + 0.5 * (speck > 0.93) as u8 as f64);
            out[k] = 1.0 - (1.0 - out[k]) * (1.0 - dust.min(0.5));
        }
    }
    out
}

/// Office paper balled up: off-white, fragments of its type turned every way,
/// grime in its creases (laid on the ball by its box faces); its print made
/// once, shared
fn scrap_ball_mat() -> Mat {
    lambert!({ map: &scrap_ball_tex() })
}
