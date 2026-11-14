//! `place_floor_kettles`
use game::prelude::*;
use placers_kit::place::open_tiles;
use placers_kit::place::ri;

// A kettle left on the floor (Blender batch `r16`): a 2 l enamel
// stovetop kettle, its bail handle up, its enamel chipped to the black steel
// and rusting at the chips, dented, limescale run white down from its lid
// and spout where it boiled over, scorched brown at its foot; or the same
// kettle in stainless steel, dull and spotted. Still breathing a little
// vapour from its spout now and then. 4% chance per level.
pub(crate) fn place_floor_kettles(g: &mut Game, _dist: &[i32]) {
    use game::assets::props::prop_part_as_modelled;
    if rnd() < 0.96 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[ri(open.len())];
    // the old three colours: cream enamel, oxblood enamel, stainless steel
    let finish = ri(3).min(2);
    // (the old brass knob, 40%): a turned wooden knob and grip, else phenolic
    let wooden = rnd() < 0.4;
    // the old spout's bearing: which way it points
    let spout_angle = rnd() * PI * 2.0;
    let kt = group();
    let body = group();
    body.rotation().y = -spout_angle;
    kt.add(&body);
    let salt = (spout_angle * 1.0e6) as u32;
    let shell = if finish == 2 {
        phong!({ map: &kettle_steel_tex(salt), color: 0xffffff, specular: 0x6a6a6a, shininess: 46 })
    } else {
        phong!({ map: &kettle_enamel_tex(finish, salt), color: 0xffffff, specular: 0x262626, shininess: 30 })
    };
    let spout_mat = if finish == 2 {
        phong!({ map: &kettle_spout_tex(2, salt), color: 0xffffff, specular: 0x6a6a6a, shininess: 46 })
    } else {
        phong!({ map: &kettle_spout_tex(finish, salt), color: 0xffffff, specular: 0x262626, shininess: 30 })
    };
    let grip_mat = if wooden { lambert!({ color: 0x5c4028 }) } else { phong!({ color: 0x161412, specular: 0x262626, shininess: 24 }) };
    let wire_mat = phong!({ color: 0x4e4c48, specular: 0x3a3a3a, shininess: 30 });
    for (name, mat) in
        [("Real__kettleBody", &shell), ("Real__kettleSpout", &spout_mat), ("Real__kettleBail", &wire_mat), ("Real__kettleGrip", &grip_mat)]
    {
        if let Some(geo) = prop_part_as_modelled(name) {
            body.add(&mesh(geo, mat));
        }
    }
    // (60%) vapour from the spout, breathing slowly: soft and lit, three
    // wisps (the old draws: where, its phase; the cycle's length and phase)
    if rnd() < 0.6 {
        let steam_mat =
            lambert!({ map: &kettle_vapour_tex(), color: 0xe6e4de, transparent: true, opacity: 0.18, depthWrite: false, side: DoubleSide });
        struct Wisp {
            mesh: Obj,
            phase: f64,
            base_y: f64,
            base_x: f64,
            base_z: f64,
        }
        let mut steam_wisps: Vec<Wisp> = vec![];
        let (tip_x, tip_y) = KETTLE_SPOUT_TIP;
        for i in 0..3 {
            let wisp = group();
            for turn in [PI / 4.0, -PI / 4.0] {
                let sheet = mesh(plane_geo!(0.035, 0.07), &steam_mat);
                sheet.rotation().y = turn;
                sheet.position().y = 0.03;
                wisp.add(&sheet);
            }
            let px = tip_x + 0.006 + (rnd() - 0.5) * 0.015;
            let py = tip_y + i as f64 * 0.030;
            let pz = (rnd() - 0.5) * 0.015;
            wisp.position().set(px, py, pz);
            let phase = rnd() * PI * 2.0;
            steam_wisps.push(Wisp { mesh: wisp.clone(), phase, base_y: py, base_x: px, base_z: pz });
            body.add(&wisp);
        }
        // Pressure-breathing cycle: 20-30 s slow oscillation.
        let pressure_cycle = 20.0 + rnd() * 10.0;
        let pressure_phase = rnd() * PI * 2.0;
        let sm = steam_mat.clone();
        g.animate(move |_g, _dt, t| {
            let pressure = 0.95 + ((t * (PI * 2.0) / pressure_cycle) + pressure_phase).sin() * 0.40;
            for w in &steam_wisps {
                {
                    let mut p = w.mesh.position();
                    p.y = w.base_y + (t * 0.7 + w.phase).sin() * 0.015 * pressure;
                    p.x = w.base_x + (t * 0.5 + w.phase).sin() * 0.010;
                    p.z = w.base_z + (t * 0.6 + w.phase * 1.3).cos() * 0.010;
                }
                let s = (0.88 + (t * 0.9 + w.phase).sin() * 0.16) * pressure;
                w.mesh.scale().set(s, s * 1.2, s);
            }
            // the wisps share one material
            sm.set_opacity(0.06 + 0.12 * clamp(pressure * 0.7, 0.0, 1.0));
        });
    }
    // 30% chance: a round cork trivet under it (the old mat's colours: a
    // cork board's browns)
    if rnd() < 0.3 {
        let cork = ri(3).min(2);
        if let Some(geo) = prop_part_as_modelled("Real__kettleTrivet") {
            kt.add(&mesh(geo, lambert!({ map: &kettle_cork_tex(cork) })));
        }
        body.position().y = KETTLE_TRIVET_T;
    }
    kt.position().set(g.wx(x) + (rnd() - 0.5) * 1.8, 0, g.wz(z) + (rnd() - 0.5) * 1.8);
    kt.rotation().y = rnd() * PI * 2.0;
    g.world.add(&kt);
}

/// The spout's mouth in the kettle's frame (x, y; domestic/kitchen_and_hearth_left_behind.py `KETTLE_SPOUT`)
const KETTLE_SPOUT_TIP: (f64, f64) = (0.1285, 0.1105);

/// The cork trivet's thickness
const KETTLE_TRIVET_T: f64 = 0.010;

/// The kettle body part's height (its bounds' top: the lid, domestic/kitchen_and_hearth_left_behind.py)
const KETTLE_H: f64 = 0.1367;

fn kettle_hash(seed: u32, x: i32, y: i32) -> f64 {
    let mut h = seed ^ (x as u32).wrapping_mul(0x8da6_b343) ^ (y as u32).wrapping_mul(0xd816_3841);
    h = (h ^ (h >> 13)).wrapping_mul(0xcb1a_b31f);
    h ^= h >> 16;
    h as f64 / 4_294_967_296.0
}

/// Smooth value noise in [0, 1], wrapping every `wrap` cells in x
fn kettle_noise(seed: u32, x: f64, y: f64, wrap: i32) -> f64 {
    let (xf, yf) = (x.floor(), y.floor());
    let (fx, fy) = (x - xf, y - yf);
    let (sx, sy) = (fx * fx * (3.0 - 2.0 * fx), fy * fy * (3.0 - 2.0 * fy));
    let (xi, yi) = (xf as i32, yf as i32);
    let w = |v: i32| if wrap > 0 { v.rem_euclid(wrap) } else { v };
    let (a, b) = (kettle_hash(seed, w(xi), yi), kettle_hash(seed, w(xi + 1), yi));
    let (c, d) = (kettle_hash(seed, w(xi), yi + 1), kettle_hash(seed, w(xi + 1), yi + 1));
    a + (b - a) * sx + (c - a) * sy + (a - b - c + d) * sx * sy
}

/// A chip in enamel at (x, y) px, `r` across: the black ground coat at its
/// edge, the steel in its middle gone to rust, a sharp light rim where the
/// glass broke away (drawn on the canvas, after the pixels)
fn kettle_chip(ctx: &mut Canvas, r: &mut SplitMix64, x: f64, y: f64, rad: f64, deep: bool) {
    let n = 7;
    let pts: Vec<(f64, f64)> = (0..n)
        .map(|k| {
            let a = k as f64 / n as f64 * PI * 2.0 + r.range(-0.3, 0.3);
            let rr = rad * r.range(0.55, 1.15);
            (x + a.cos() * rr, y + a.sin() * rr * 0.8)
        })
        .collect();
    let path = |ctx: &mut Canvas, k: f64| {
        ctx.begin_path();
        for (i, &(px, py)) in pts.iter().enumerate() {
            let (qx, qy) = (x + (px - x) * k, y + (py - y) * k);
            if i == 0 {
                ctx.move_to(qx, qy);
            } else {
                ctx.line_to(qx, qy);
            }
        }
        ctx.close_path();
    };
    ctx.set_fill_style("rgba(206,204,196,0.55)");
    path(ctx, 1.12);
    ctx.fill();
    ctx.set_fill_style("rgb(26,24,24)");
    path(ctx, 1.0);
    ctx.fill();
    if deep {
        ctx.set_fill_style("rgba(98,52,30,0.9)");
        path(ctx, 0.62);
        ctx.fill();
        ctx.set_fill_style("rgba(70,36,22,0.8)");
        path(ctx, 0.3);
        ctx.fill();
    }
}

/// The enamel, unrolled round the body (u from +z toward +x: the spout at
/// u = 0.25) and up it (v, over the body and lid's 137 mm): its glaze
/// mottled and dulled, grime and a brown scorch at its foot, limescale run
/// white down from the lid and under the spout, the rim's dark band (cream
/// only), chips at the rim, the chime, round the dent and the spout's seam.
fn kettle_enamel_tex(finish: usize, salt: u32) -> Tex {
    canvas_tex_opt(512, 256, false, move |ctx, w, h| {
        let (base, mottle): ([f64; 3], f64) = if finish == 0 { ([214.0, 206.0, 186.0], 10.0) } else { ([104.0, 44.0, 36.0], 8.0) };
        let seed = salt ^ 0xe7a3;
        let vy = |y: f64| (1.0 - y / KETTLE_H) * h;
        let n = w as usize;
        let mut img = ctx.create_image_data(w, h);
        par_rows(&mut img.data, n * 4, |j, out| {
            let fy = 1.0 - j as f64 / h; // up the kettle
            for i in 0..n {
                let u = i as f64 / w;
                let m = kettle_noise(seed, u * 16.0, fy * 8.0, 16) * 0.6 + kettle_noise(seed ^ 1, u * 64.0, fy * 32.0, 64) * 0.4;
                let mut c = [base[0] + (m - 0.5) * mottle * 2.0, base[1] + (m - 0.5) * mottle * 2.0, base[2] + (m - 0.5) * mottle * 2.0];
                // grime and the stove's scorch, brown, up from the foot
                let low = (1.0 - fy / 0.28).clamp(0.0, 1.0);
                let scorch = (low * low * (0.55 + 0.45 * kettle_noise(seed ^ 2, u * 24.0, fy * 10.0, 24))).min(1.0);
                let sc = [92.0, 64.0, 38.0];
                for k in 0..3 {
                    c[k] += (sc[k] - c[k]) * scorch * 0.75;
                }
                // limescale: chalky runs down from the lid's seat, most under
                // the spout's side, and a crust at the chime
                let du = ((u - 0.25 + 0.5).rem_euclid(1.0) - 0.5).abs();
                let runs = (kettle_noise(seed ^ 3, u * 90.0, fy * 2.0, 90) - 0.55).max(0.0) * 2.6;
                let reach = 0.85 - 0.45 * (1.0 - (du / 0.18).min(1.0));
                let lime = runs * ((fy - reach) / 0.12).clamp(0.0, 1.0) * (0.4 + 0.6 * (1.0 - (du / 0.3).min(1.0)))
                    + 0.5 * (1.0 - (fy / 0.05).min(1.0)) * kettle_noise(seed ^ 4, u * 60.0, fy * 20.0, 60);
                let lime = lime.min(0.8);
                let lc = [222.0, 218.0, 204.0];
                for k in 0..3 {
                    c[k] += (lc[k] - c[k]) * lime;
                }
                // the dust of years, mottled, and the glaze gone dull under it
                let dust = 0.14 + 0.22 * kettle_noise(seed ^ 6, u * 30.0, fy * 14.0, 30) + 0.12 * fy;
                let dc = [150.0, 142.0, 128.0];
                for k in 0..3 {
                    c[k] += (dc[k] - c[k]) * dust;
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
        // cream enamelware's dark band at the lid seat's bead and the lid's rim
        if finish == 0 {
            ctx.set_fill_style("rgba(34,40,58,0.9)");
            ctx.fill_rect(0, vy(0.1250), w, vy(0.1185) - vy(0.1250));
            ctx.fill_rect(0, vy(0.1270), w, vy(0.1250) - vy(0.1270) + 1.0);
        }
        let mut r = SplitMix64::new(salt, 0xc41b);
        // chips: along the rim's bead, the chime, round the dent (u 0.70),
        // the spout's seam (u 0.25, low)
        let spots: [(f64, f64, f64, usize); 4] =
            [(0.5, 0.120, 0.5, 14), (0.5, 0.004, 0.5, 10), (0.70, 0.052, 0.05, 8), (0.25, 0.032, 0.03, 4)];
        for &(uc, yc, spread, count) in &spots {
            for _ in 0..count {
                let u = (uc + r.range(-spread, spread)).rem_euclid(1.0);
                let y = yc + r.range(-0.006, 0.006);
                let rad = r.range(2.0, 7.0);
                kettle_chip(ctx, &mut r, u * w, vy(y), rad, rad > 3.5);
            }
        }
        // lone chips anywhere, from knocks
        for _ in 0..12 {
            let (x, y, rad) = (r.next() * w, vy(r.range(0.02, 0.11)), r.range(1.5, 4.0));
            kettle_chip(ctx, &mut r, x, y, rad, false);
        }
    })
}

/// Stainless steel gone dull: its polish clouded, water spots dried white,
/// limescale run down from the lid and spout, rainbow heat tint and a brown
/// scorch at its foot, scratches round it
fn kettle_steel_tex(salt: u32) -> Tex {
    canvas_tex_opt(512, 256, false, move |ctx, w, h| {
        let seed = salt ^ 0x57ee;
        let n = w as usize;
        let mut img = ctx.create_image_data(w, h);
        par_rows(&mut img.data, n * 4, |j, out| {
            let fy = 1.0 - j as f64 / h;
            for i in 0..n {
                let u = i as f64 / w;
                let m = kettle_noise(seed, u * 10.0, fy * 6.0, 10) * 0.5 + kettle_noise(seed ^ 1, u * 120.0, fy * 4.0, 120) * 0.5;
                let g0 = 118.0 + (m - 0.5) * 26.0;
                let mut c = [g0, g0 + 1.0, g0 + 2.0];
                let low = (1.0 - fy / 0.25).clamp(0.0, 1.0);
                let tint = low * low;
                let sc = [96.0, 74.0, 52.0];
                for k in 0..3 {
                    c[k] += (sc[k] - c[k]) * tint * 0.7;
                }
                let du = ((u - 0.25 + 0.5).rem_euclid(1.0) - 0.5).abs();
                let runs = (kettle_noise(seed ^ 3, u * 90.0, fy * 2.0, 90) - 0.55).max(0.0) * 2.4;
                let spots = (kettle_noise(seed ^ 5, u * 140.0, fy * 70.0, 140) - 0.72).max(0.0) * 3.0;
                let lime = (runs * ((fy - 0.55) / 0.12).clamp(0.0, 1.0) * (1.0 - (du / 0.35).min(1.0)) + spots).min(0.7);
                for k in 0..3 {
                    c[k] += (206.0 - c[k]) * lime;
                }
                out[i * 4..i * 4 + 4].copy_from_slice(&[c[0] as u8, c[1] as u8, c[2] as u8, 255]);
            }
        });
        ctx.put_image_data(&img, 0, 0);
        let mut r = SplitMix64::new(salt, 0x5c2a);
        ctx.set_line_width(0.7);
        for _ in 0..60 {
            let (x, y) = (r.next() * w, r.next() * h);
            let a = r.range(-0.4, 0.4);
            let l = r.range(4.0, 22.0);
            ctx.set_stroke_style(&format!("rgba({},{},{},0.35)", if r.next() < 0.5 { 70 } else { 170 }, 168, 166));
            ctx.begin_path();
            ctx.move_to(x, y);
            ctx.line_to(x + a.cos() * l, y + a.sin() * l);
            ctx.stroke();
        }
    })
}

/// The spout (the box's texture coordinates over its bounds: its mouth at
/// the top right): the body's finish, a limescale crust at the mouth and run
/// down under it, chips at the mouth and the seam
fn kettle_spout_tex(finish: usize, salt: u32) -> Tex {
    canvas_tex_opt(128, 128, false, move |ctx, w, h| {
        let base = match finish {
            0 => "rgb(212,204,184)",
            1 => "rgb(110,40,34)",
            _ => "rgb(120,121,122)",
        };
        ctx.set_fill_style(base);
        ctx.fill_rect(0, 0, w, h);
        let mut r = SplitMix64::new(salt, 0x590f);
        // the mouth's crust, at the part's top and its outer end
        for _ in 0..40 {
            let (x, y) = (w * r.range(0.62, 1.0), h * r.range(0.0, 0.42));
            let rr = r.range(2.0, 9.0);
            let gr = ctx.create_radial_gradient(x, y, 0, x, y, rr);
            gr.add_color_stop(0, "rgba(224,220,206,0.55)");
            gr.add_color_stop(1, "rgba(224,220,206,0)");
            ctx.set_fill_style(gr);
            ctx.fill_rect(x - rr, y - rr, rr * 2.0, rr * 2.0);
        }
        // scorch low on it
        let gl = ctx.create_linear_gradient(0, h, 0, h * 0.6);
        gl.add_color_stop(0, "rgba(80,58,36,0.5)");
        gl.add_color_stop(1, "rgba(80,58,36,0)");
        ctx.set_fill_style(gl);
        ctx.fill_rect(0, 0, w, h);
        if finish < 2 {
            for _ in 0..5 {
                let (x, y) = (w * r.range(0.7, 0.98), h * r.range(0.02, 0.3));
                let rad = r.range(2.0, 5.0);
                kettle_chip(ctx, &mut r, x, y, rad, true);
            }
            for _ in 0..3 {
                let (x, y) = (w * r.range(0.0, 0.2), h * r.range(0.6, 1.0));
                let rad = r.range(2.0, 4.0);
                kettle_chip(ctx, &mut r, x, y, rad, true);
            }
        }
    })
}

/// Cork, its grains: light and dark granules pressed together, stained (the
/// three shades: a natural board, one gone dark with spills, a pale one)
fn kettle_cork_tex(shade: usize) -> Tex {
    canvas_tex_opt(128, 128, false, move |ctx, w, h| {
        let base: [f64; 3] = [[150.0, 112.0, 72.0], [96.0, 68.0, 44.0], [176.0, 146.0, 104.0]][shade];
        let n = w as usize;
        let mut img = ctx.create_image_data(w, h);
        par_rows(&mut img.data, n * 4, |j, out| {
            for i in 0..n {
                let (x, y) = (i as f64, j as f64);
                let g = kettle_noise(31, x / 1.6, y / 1.6, 0) * 0.6 + kettle_noise(32, x / 4.0, y / 4.0, 0) * 0.4;
                let stain = kettle_noise(33, x / 30.0, y / 30.0, 0);
                let k = 0.72 + 0.5 * g - 0.25 * (stain - 0.4).max(0.0);
                out[i * 4..i * 4 + 4].copy_from_slice(&[
                    (base[0] * k).min(255.0) as u8,
                    (base[1] * k).min(255.0) as u8,
                    (base[2] * k).min(255.0) as u8,
                    255,
                ]);
            }
        });
        ctx.put_image_data(&img, 0, 0);
    })
}

/// A wisp of vapour: soft, drawn out upward and thinning
fn kettle_vapour_tex() -> Tex {
    thread_local! { static T: std::cell::OnceCell<Tex> = const { std::cell::OnceCell::new() }; }
    T.with(|c| {
        c.get_or_init(|| {
            canvas_tex_opt(64, 128, false, |ctx, w, h| {
                ctx.clear_rect(0, 0, w, h);
                for k in 0..36 {
                    let t = k as f64 / 35.0;
                    let x = w / 2.0 + (t * 5.0 + rnd() * 0.8).sin() * w * 0.14 * t;
                    let y = h * (0.97 - t * 0.85);
                    let r = w * (0.12 + t * 0.24);
                    let a = 0.28 * (1.0 - t).powf(0.7);
                    ctx.save();
                    ctx.translate(x, y);
                    ctx.scale(1.0, 1.4);
                    let gr = ctx.create_radial_gradient(0, 0, 0, 0, 0, r);
                    gr.add_color_stop(0, &format!("rgba(255,255,255,{a})"));
                    gr.add_color_stop(1, "rgba(255,255,255,0)");
                    ctx.set_fill_style(gr);
                    ctx.begin_path();
                    ctx.arc(0, 0, r, 0, 7);
                    ctx.fill();
                    ctx.restore();
                }
            })
        })
        .clone()
    })
}
