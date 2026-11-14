//! `place_fireplaces`
use game::assets::props::prop_part_as_modelled;
use game::prelude::*;
use placers_kit::geometry::put as t10_put;
use placers_kit::noise::fbm as t10_fbm;
use placers_kit::noise::hash as t10_hash;
use placers_kit::noise::mix_unclamped as t10_mix;
use placers_kit::noise::sstep as t10_ss;
use placers_kit::noise::vn as t10_vn;
use placers_kit::paint::fill as t10_fill;
use placers_kit::paint::opaque as t10_opaque;
use placers_kit::place::face_wall;
use placers_kit::place::vary as t10_vary;
use placers_kit::place::wall_tiles;

// Fireplace: 2.5% chance per level. A Victorian bedroom fireplace
// against the wall (Blender batch `t10`): a painted pine mantelpiece on its
// plinths, its corbels, frieze and shelf, the paint gone yellow and chipped
// through its coats to the wood; a cast-iron arched register insert, its
// black-lead gone dull and rusting, its canopy, basket and fret; the
// firebrick behind sooted black; a slate hearth; ash in the basket and
// fallen through it, the charred ends of a fire, the soot of years climbed
// the wall above the shelf. Cold, or (55%, as the old one) someone has lit
// it: a low wood fire burning in the basket, embers glowing under the logs
// and in their cracks, tongues of flame, its light and its crackle.
pub(crate) fn place_fireplaces(g: &mut Game, _dist: &[i32]) {
    g.pp.lit_fireplace_positions.clear(); // reset per level
    if rnd() < 0.975 {
        return;
    }
    let wt = wall_tiles(g);
    if wt.is_empty() {
        return;
    }
    let (x, z, dx, dz) = wt[(rnd() * wt.len() as f64).floor() as usize];
    // the old draws, call for call: how many logs and where each lay,
    // whether it is lit, each flame's colour (now how tall it burns) and
    // place, the flicker's phases
    let log_count = 3 + (rnd() * 2.0).floor() as i32;
    let log_x: Vec<f64> = (0..log_count).map(|_| (rnd() - 0.5) * 0.02).collect();
    let is_lit = rnd() < 0.55;
    let mut flames: Vec<(usize, f64, f64)> = vec![];
    let mut phases = (0.0, 0.0);
    if is_lit {
        let flame_colors = [0xffe040, 0xffa020, 0xff6010];
        for _ in 0..5 {
            let c = choice(&flame_colors);
            let fx = (rnd() - 0.5) * 0.06;
            let fz = (rnd() - 0.5) * 0.02;
            flames.push((flame_colors.iter().position(|&k| k == c).unwrap_or(0), fx, fz));
        }
        let flicker_phase = rnd() * PI * 2.0;
        let flicker_phase2 = rnd() * PI * 2.0;
        phases = (flicker_phase, flicker_phase2);
    }
    let fp = group();
    // Back (local z 0) against the wall, front towards the room.
    fp.rotation().y = face_wall(dx, dz);
    fp.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - 0.005), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - 0.005));
    let p = fp.pos();
    let vary = t10_vary(p.x, p.z, log_count as f64);
    // the parts are modelled with the wall at z = 0: 5 mm behind the spot
    let body = group();
    body.position().z = -0.005;
    fp.add(&body);
    t10_put(&body, "Real__fireplaceMantel", &lambert!({ map: &fp_paint_tex(vary) }));
    let iron = phong!({ map: &fp_iron_tex(vary, 0.25), specular: 0x262626, shininess: 22 });
    t10_put(&body, "Real__fireplaceInsert", &iron);
    t10_put(&body, "Real__fireplaceGrate", &phong!({ map: &fp_iron_tex(vary ^ 0x9, 0.55), specular: 0x1a1a1a, shininess: 16 }));
    t10_put(&body, "Real__fireplaceFirebox", &lambert!({ map: &fp_soot_tex(vary) }));
    t10_put(&body, "Real__fireplaceHearth", &lambert!({ map: &fp_slate_tex(vary) }));
    t10_put(&body, "Real__fireplaceAsh", &lambert!({ map: &fp_ash_tex(), color: 0x9c988f }));
    // the charred ends, lying across the basket on the ash; burning from
    // below when the fire is lit
    // (lit, the fire's own light falls on them close: their char darker)
    let char_mat = lambert!({ map: &fp_char_tex(), color: if is_lit { 0x4c4c4c } else { 0xffffff } });
    let glow_mat = basic_mat!({ map: &fp_glow_tex(), transparent: true, depthWrite: false });
    const SLOTS: [(f64, f64); 4] = [(-0.024, 0.0), (0.030, 0.0), (0.004, 1.0), (-0.060, 0.0)];
    for (i, &jx) in log_x.iter().enumerate() {
        let (name, r) = FP_LOGS[i];
        let (sz, layer) = SLOTS[i];
        let log = group();
        log.position().set(jx * 3.0, FP_BASKET_Y + 0.018 + r + layer * 0.050, FP_LOG_Z + sz);
        log.rotation().y = (t10_hash(vary, i as i32, 1) - 0.5) * 0.35;
        log.rotation().x = (t10_hash(vary, i as i32, 2) - 0.5) * 0.6;
        t10_put(&log, name, &char_mat);
        if is_lit {
            if let Some(m) = t10_put(&log, &format!("{name}Glow"), &glow_mat) {
                m.set_render_order(2);
            }
        }
        body.add(&log);
    }
    if is_lit {
        t10_put(&body, "Real__fireplaceEmbers", &basic_mat!({ map: &fp_ember_tex() }));
        // the flames: each one's colour says how tall it burns, its place
        // where it rises among the logs; each scales from its root
        let flame_mat = basic_mat!({ map: &fp_flame_tex(), transparent: true, depthWrite: false, blending: AdditiveBlending });
        let flame_group = group();
        for (i, &(tall, fx, fz)) in flames.iter().enumerate() {
            let root = group();
            root.position().set((i as f64 - 2.0) * 0.058 + fx * 1.5, FP_BASKET_Y + 0.065, FP_LOG_Z + 0.012 + fz * 2.0);
            root.rotation().y = (t10_hash(vary, i as i32, 9) - 0.5) * 1.2;
            // its tongue, and a smaller one licking up beside it
            const TONGUES: [&str; 3] = ["Real__fireplaceFlameA", "Real__fireplaceFlameB", "Real__fireplaceFlameC"];
            for (k, (dx, turn)) in [(0.0, 0.0), (0.017, PI)].into_iter().enumerate() {
                if let Some(m) = t10_put(&root, TONGUES[(tall + k).min(2)], &flame_mat) {
                    m.position().x = dx;
                    m.rotation().y = turn;
                    m.set_render_order(3);
                }
            }
            flame_group.add(&root);
        }
        body.add(&flame_group);
        // PointLight — bright warm orange, at the fire.
        let fire_light = point_light!(0xff8030, 0.85, 3.0, 2);
        fire_light.position().set(0, FP_BASKET_Y + 0.15, FP_LOG_Z + 0.08);
        body.add(&fire_light);
        // Flame flicker animator — multi-freq for organic feel.
        let (flicker_phase, flicker_phase2) = phases;
        let base_intensity = 0.85;
        let (fl2, fg2) = (fire_light.clone(), flame_group.clone());
        g.animate(move |_g, _dt, t| {
            let f = 0.85 + (t * 7.0 + flicker_phase).sin() * 0.10 + (t * 13.0 + flicker_phase2).sin() * 0.05;
            fl2.set_intensity(base_intensity * f);
            // each flame's height flickers from its root
            for (i, flame) in fg2.children().iter().enumerate() {
                let i = i as f64;
                let local_flick = 0.85 + (t * (8.0 + i) + flicker_phase * i).sin() * 0.15;
                flame.scale().set(0.92 + 0.1 * local_flick, local_flick, 0.92 + 0.1 * local_flick);
            }
        });
    }
    // the soot of years on the wall above the shelf
    let soot = mesh(
        plane_geo!(0.90, 0.80),
        lambert!({ map: &fp_ghost_tex(vary), transparent: true, depthWrite: false, polygonOffset: true, polygonOffsetFactor: -2, polygonOffsetUnits: -2 }),
    );
    soot.position().set(0, FP_SHELF_TOP + 0.40 - 0.02, 0.002);
    body.add(&soot);
    g.world.add(&fp);
    // Register only lit fireplaces for crackle audio.
    if is_lit {
        let p = fp.pos();
        g.pp.lit_fireplace_positions.push(v3(p.x, 0, p.z));
    }
}

/// The fireplace (t10.py `FP_*`): the basket's floor (y) and the depth (z)
/// the logs lie at, its shelf's top (y)
const FP_BASKET_Y: f64 = 0.130;

const FP_LOG_Z: f64 = 0.080;

const FP_SHELF_TOP: f64 = 1.100;

/// The charred ends left in the basket: (part, radius)
const FP_LOGS: [(&str, f64); 4] =
    [("Real__fireplaceLogA", 0.040), ("Real__fireplaceLogB", 0.034), ("Real__fireplaceLogC", 0.030), ("Real__fireplaceLogD", 0.026)];

/// The mantelpiece's paint as its front is mapped (u across its 960 mm, v
/// up its 1100): cream gloss gone yellow and crazed, chipped through to an
/// older green coat and to the pine, grimed low down, soot over the
/// opening's head
fn fp_paint_tex(vary: u32) -> Tex {
    canvas_tex_opt(256, 296, false, move |ctx, w, h| {
        let seed = vary ^ 0xfa1;
        t10_fill(ctx, w, h, |u, v| {
            let (x, y) = (u * 0.96, v * 1.10);
            let m = t10_fbm(seed, x * 6.0, y * 6.0, 4);
            let mut o = [0.66, 0.61, 0.50].map(|c| c * (0.9 + 0.14 * m));
            // crazing: a fine net of dark lines
            let cr = t10_vn(seed ^ 3, x * 180.0, y * 180.0);
            if (cr - 0.5).abs() < 0.03 {
                o = o.map(|c| c * 0.82);
            }
            // chips: through the cream to the green, deeper to the pine
            let ch = t10_fbm(seed ^ 5, x * 70.0, y * 70.0, 3)
                + 0.10 * (1.0 - t10_ss(0.0, 0.25, y))
                + 0.06 * (1.0 - t10_ss(0.0, 0.05, (u - 0.5).abs().min(0.5 - (u - 0.5).abs())));
            if ch > 0.71 {
                o = [0.36, 0.38, 0.32];
            }
            if ch > 0.76 {
                o = [0.48, 0.34, 0.21].map(|c| c * (0.85 + 0.2 * m));
            }
            // grime: kicked at its foot, handled at the shelf's edge
            o = o.map(|c| c * (1.0 - 0.35 * (1.0 - t10_ss(0.0, 0.2, y))));
            // the soot that crept out over the opening's head onto the frieze
            let sx = (u - 0.5).abs() * 0.96;
            let soot = (1.0 - t10_ss(0.18, 0.36, sx)) * t10_ss(0.80, 0.90, y) * (1.0 - t10_ss(1.02, 1.08, y));
            o = t10_mix(o, [0.10, 0.09, 0.08], soot * (0.55 + 0.3 * m));
            t10_opaque(t10_mix(o, [0.47, 0.45, 0.40], 0.10))
        });
    })
}

/// Cast iron (as the insert's front is mapped): black-lead gone dull and
/// grey, rust (`rust`: how much) blooming through, most at its foot
fn fp_iron_tex(vary: u32, rust: f64) -> Tex {
    canvas_tex_opt(256, 320, false, move |ctx, w, h| {
        let seed = vary ^ 0x1a0;
        t10_fill(ctx, w, h, |u, v| {
            let n = t10_fbm(seed, u * 10.0, v * 12.0, 4);
            let mut o = [0.075 + 0.05 * n, 0.072 + 0.046 * n, 0.068 + 0.042 * n];
            let r = t10_fbm(seed ^ 3, u * 16.0, v * 20.0, 4) + rust * 0.35 * (1.0 - t10_ss(0.0, 0.35, v)) + rust * 0.2;
            o = t10_mix(o, [0.27, 0.15, 0.08], 0.75 * t10_ss(0.70, 0.86, r));
            let dust = 0.10 + 0.12 * t10_fbm(seed ^ 7, u * 5.0, v * 5.0, 2);
            t10_opaque(t10_mix(o, [0.40, 0.38, 0.34], dust))
        });
    })
}

/// The firebrick behind, sooted: its courses just showing through the black
fn fp_soot_tex(vary: u32) -> Tex {
    canvas_tex_opt(128, 256, false, move |ctx, w, h| {
        let seed = vary ^ 0x5007;
        t10_fill(ctx, w, h, |u, v| {
            let (x, y) = (u * 0.44, v * 0.76);
            let row = (y / 0.065).floor();
            let bx = (x + if row as i32 % 2 == 0 { 0.0 } else { 0.11 }) / 0.22;
            let joint = (y / 0.065).fract() < 0.08 || bx.fract() < 0.04;
            let brick = [0.36, 0.24, 0.17].map(|c| c * (0.8 + 0.3 * t10_hash(seed, bx.floor() as i32, row as i32)));
            let mut o = if joint { [0.20, 0.18, 0.16] } else { brick };
            // the soot: thick everywhere, thickest up the back where the smoke went
            let s = 0.82 + 0.15 * t10_ss(0.1, 0.8, v) * (1.0 - (u - 0.5).abs()) + 0.08 * t10_fbm(seed ^ 3, u * 9.0, v * 9.0, 3);
            o = t10_mix(o, [0.025, 0.022, 0.02], s.min(0.97));
            t10_opaque(o)
        });
    })
}

/// The slate hearth (top as mapped): its cleft, its grey, ash ground in,
/// a crack across it
fn fp_slate_tex(vary: u32) -> Tex {
    canvas_tex_opt(256, 128, false, move |ctx, w, h| {
        let seed = vary ^ 0x51a7;
        let crack_y = 0.3 + 0.4 * t10_hash(seed, 1, 1);
        t10_fill(ctx, w, h, |u, v| {
            let n = t10_fbm(seed, u * 12.0, v * 4.0, 4);
            let mut o = [0.16, 0.17, 0.18].map(|c| c * (0.85 + 0.3 * n));
            // ash trodden in, heaviest by the fire
            let ash = t10_ss(0.45, 0.75, t10_fbm(seed ^ 3, u * 7.0, v * 7.0, 3)) * (0.4 + 0.6 * v);
            o = t10_mix(o, [0.48, 0.46, 0.43], 0.5 * ash);
            let cy = crack_y + 0.04 * (t10_fbm(seed ^ 5, u * 6.0, 0.0, 3) - 0.5);
            if (v - cy).abs() < 0.006 && u > 0.2 && u < 0.85 {
                o = o.map(|c| c * 0.35);
            }
            t10_opaque(o)
        });
    })
}

/// Wood ash: pale grey, flecked with char
fn fp_ash_tex() -> Tex {
    thread_local! { static T: std::cell::OnceCell<Tex> = const { std::cell::OnceCell::new() }; }
    T.with(|c| {
        c.get_or_init(|| {
            canvas_tex_opt(128, 128, false, |ctx, w, h| {
                t10_fill(ctx, w, h, |u, v| {
                    let n = t10_fbm(0xa5, u * 12.0, v * 12.0, 4);
                    let fleck = if t10_hash(0xa6, (u * 128.0) as i32, (v * 128.0) as i32) > 0.93 { 0.3 } else { 1.0 };
                    let k = (0.38 + 0.16 * n) * fleck;
                    [k, k * 0.98, k * 0.94, 1.0]
                });
            })
        })
        .clone()
    })
}

/// Charcoal: black, crazed into blocks, their edges burnt to white ash
fn fp_char_tex() -> Tex {
    thread_local! { static T: std::cell::OnceCell<Tex> = const { std::cell::OnceCell::new() }; }
    T.with(|c| {
        c.get_or_init(|| {
            canvas_tex_opt(128, 128, false, |ctx, w, h| {
                t10_fill(ctx, w, h, |u, v| {
                    let (gx, gy) = (u * 10.0 + 0.3 * t10_vn(0xc1, u * 8.0, v * 8.0), v * 6.0);
                    let edge = (gx.fract() - 0.5).abs().max((gy.fract() - 0.5).abs());
                    let mut k = 0.035 + 0.03 * t10_vn(0xc2, u * 40.0, v * 40.0);
                    if edge > 0.44 {
                        k = 0.01;
                    } else if edge > 0.38 {
                        k = 0.30;
                    }
                    let ash = t10_ss(0.6, 0.8, t10_fbm(0xc3, u * 6.0, v * 6.0, 3));
                    let k = k + (0.40 - k) * ash * 0.45;
                    [k, k * 0.96, k * 0.92, 1.0]
                });
            })
        })
        .clone()
    })
}

/// The soot of years on the wall above the shelf: a dark bloom rising from
/// the shelf's edge, fading up the wall, darkest over the middle
fn fp_ghost_tex(vary: u32) -> Tex {
    canvas_tex_opt(128, 128, false, move |ctx, w, h| {
        let seed = vary ^ 0x6057;
        t10_fill(ctx, w, h, |u, v| {
            let x = (u - 0.5) * 2.0;
            let n = t10_fbm(seed, u * 5.0, v * 3.0, 4);
            let a = (1.0 - t10_ss(0.0, 0.95, v + 0.25 * (n - 0.5))) * (1.0 - t10_ss(0.35, 1.0, x.abs() + 0.2 * v)) * (0.55 + 0.45 * n);
            [0.06, 0.055, 0.05, (a * 0.55).min(0.6)]
        });
    })
}

/// The cracks of a burning log's underside (as it is mapped): a net of
/// orange-hot lines between the char's blocks, clear between them
fn fp_glow_tex() -> Tex {
    thread_local! { static T: std::cell::OnceCell<Tex> = const { std::cell::OnceCell::new() }; }
    T.with(|c| {
        c.get_or_init(|| {
            canvas_tex_opt(128, 128, false, |ctx, w, h| {
                t10_fill(ctx, w, h, |u, v| {
                    let (gx, gy) = (u * 10.0 + 0.3 * t10_vn(0xc1, u * 8.0, v * 8.0), v * 6.0);
                    let edge = (gx.fract() - 0.5).abs().max((gy.fract() - 0.5).abs());
                    let hot = t10_ss(0.36, 0.46, edge) * (0.55 + 0.45 * t10_fbm(0xc4, u * 5.0, v * 5.0, 3));
                    let k = t10_ss(0.3, 0.9, hot);
                    [1.0, 0.42 + 0.3 * k, 0.10 + 0.1 * k, hot]
                });
            })
        })
        .clone()
    })
}

/// Embers: orange-hot cores under a grey-black crust, the crust broken
fn fp_ember_tex() -> Tex {
    thread_local! { static T: std::cell::OnceCell<Tex> = const { std::cell::OnceCell::new() }; }
    T.with(|c| {
        c.get_or_init(|| {
            canvas_tex_opt(128, 128, false, |ctx, w, h| {
                t10_fill(ctx, w, h, |u, v| {
                    let n = t10_fbm(0xe1, u * 14.0, v * 14.0, 4);
                    let crust = t10_ss(0.55, 0.7, t10_fbm(0xe2, u * 22.0, v * 22.0, 3));
                    let hot = [0.95, 0.38 + 0.25 * n, 0.08];
                    let o = t10_mix(hot, [0.12, 0.09, 0.08], crust * 0.85);
                    t10_opaque(o)
                });
            })
        })
        .clone()
    })
}

/// A wood fire's flame (laid round it, v up it): orange at its root, a
/// yellow-white body, red toward its tip, thinning out to nothing; its
/// edges soft
fn fp_flame_tex() -> Tex {
    thread_local! { static T: std::cell::OnceCell<Tex> = const { std::cell::OnceCell::new() }; }
    T.with(|c| {
        c.get_or_init(|| {
            canvas_tex_opt(32, 128, false, |ctx, w, h| {
                t10_fill(ctx, w, h, |u, v| {
                    let root = 1.0 - t10_ss(0.0, 0.18, v);
                    let tip = t10_ss(0.30, 0.95, v);
                    let c = t10_mix([1.0, 0.86, 0.52], [0.95, 0.40, 0.10], tip);
                    let c = t10_mix(c, [0.85, 0.36, 0.08], root * 0.7);
                    let edge = 1.0 - t10_ss(0.22, 0.5, (u - 0.5).abs());
                    let a = (0.9 - 0.4 * root) * (1.0 - t10_ss(0.55, 1.0, v)) * (0.35 + 0.65 * edge);
                    [c[0], c[1], c[2], a]
                });
            })
        })
        .clone()
    })
}
