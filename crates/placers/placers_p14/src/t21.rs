//! Batch `t21`'s prop in this crate, made real (`docs/design/PROPS.md` P3): the
//! aquarium left for years (`assets/src/blender/props/t21.py`,
//! `t21.notes.md`). Its placer and what only it uses; it keeps the old
//! placer's draws, call for call.
use super::*;
use aq_kit::*;

/// The tank (`t21.py` AQ_*): the glass bottom's top, half the inside, the
/// gravel's depth at the front and how much more at the back; the glass
/// part's foot and height (its map's v); where the air stone stands
const AQ_FLOOR: f64 = 0.606;
const AQ_IN: (f64, f64) = (0.514 / 2.0 - 0.006, 0.267 / 2.0 - 0.006);
const AQ_GRAVEL: (f64, f64) = (0.032, 0.020);
const AQ_GLASS_Y: (f64, f64) = (0.603, 0.918 - 0.603);
const AQ_STONE: (f64, f64, f64) = (-0.200, AQ_FLOOR + 0.070, -0.078);
/// The hood's lamp: its tube (`t21.py` `aq_tube`), where its light is
/// (the old light stood 5 cm from it, in the hood's housing)
const AQ_TUBE: (f64, f64) = (0.600 + 0.321 + 0.020, -0.267 / 2.0 + 0.085);
/// r18's dead plant's stems (`Real__deadPlantStem…`): how far each reaches
/// up its own frame, and the way each leans of itself (r18.py)
const STEM_REACH: f64 = 0.30;
const STEM_LEAN_AZ: [f64; 3] = [0.6397, -0.0035, -0.8885];

/// The gravel's top at z (banked up to the back), a little over it
fn gravel_top(z: f64) -> f64 {
    let t = ((AQ_IN.1 - z) / (2.0 * AQ_IN.1)).clamp(0.0, 1.0);
    AQ_FLOOR + AQ_GRAVEL.0 + AQ_GRAVEL.1 * t
}

// An aquarium left for years (Blender batch `t21`): a standard 10 gallon
// tank on a low particleboard cabinet stand, its laminate peeled and its
// foot swollen where water ran down it. Nothing lives in it. Either long
// dry (60% of the old ones had fish and kept their water: a little green
// water is left in those), its glass ringed with the white lines of every
// level the water fell through, its gravel crusted, a dead plant's stems
// lying on it and their dry leaves. The air pump is still running (its hum
// is the old one's): where there is water, its bubbles still rise from the
// stone. The hood's lamp is still lit where the old one's light was on.
pub(super) fn place_aquariums(g: &mut Game, _dist: &[i32]) {
    g.pp.aquarium_positions.clear(); // reset per level
    if rnd() < 0.975 {
        return;
    }
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[(rnd() * open.len() as f64).floor() as usize];
    // the old draws, call for call. The plants (2-3, each a colour and three
    // blades: height, x, z, lean): a dead stem each, its blades' places and
    // turns its fallen leaves
    let plant_colors = [0x408030, 0x60a040, 0x306020];
    let plant_count = 2 + (rnd() * 2.0).floor() as i32;
    let mut plants: Vec<(usize, [[f64; 4]; 3])> = vec![];
    for _ in 0..plant_count {
        let c = choice(&plant_colors);
        let kind = plant_colors.iter().position(|&k| k == c).unwrap_or(0);
        let mut blades = [[0.0; 4]; 3];
        for b in blades.iter_mut() {
            let bh = rnd();
            let bx = rnd();
            let bz = rnd();
            let br = rnd();
            *b = [bh, bx, bz, br];
        }
        plants.push((kind, blades));
    }
    // the fish (60%): none now; their draw says the water stayed, the first
    // one's height how much of it
    let fish_colors = [0xf0a020, 0xa02040, 0x2060a0, 0x60a0d0, 0xf0e040];
    let mut water: Option<f64> = None;
    let mut ripple_phase = 0.0;
    if rnd() < 0.6 {
        let fish_count = 2 + (rnd() * 3.0).floor() as i32;
        for i in 0..fish_count {
            let _ = choice(&fish_colors); // keeps the old draw (its colour)
            let _ = rnd(); // keeps the old draw (where it swam: x)
            let fy = rnd();
            let _ = rnd(); // z
            let _ = rnd(); // its heading
            let phase = rnd();
            if i == 0 {
                water = Some(0.03 + 0.08 * fy);
                ripple_phase = phase * PI * 2.0;
            }
        }
    }
    // the bubbles (4-6): each its size, its place by the stone, its height,
    // offset and speed (the old phase kept)
    let bubble_count = 4 + (rnd() * 3.0).floor() as i32;
    let mut bubbles_d: Vec<[f64; 6]> = vec![];
    for _ in 0..bubble_count {
        let r = rnd();
        let bx = rnd();
        let bz = rnd();
        let by = rnd();
        let offset = rnd();
        let speed = rnd();
        let _ = rnd(); // keeps the old draw (its phase)
        bubbles_d.push([r, bx, bz, by, offset, speed]);
    }
    let lit = rnd() < 0.6;
    let px = g.wx(x) + (rnd() - 0.5) * 1.0;
    let pz = g.wz(z) + (rnd() - 0.5) * 1.0;
    let turn = rnd() * PI * 2.0;
    let salt = salt(px, pz, 0xa9);
    let level = water.map(|d| AQ_FLOOR + AQ_GRAVEL.0 + AQ_GRAVEL.1 * 0.5 + d);
    let aq = group();
    put(&aq, "Real__aquariumStand", &lambert!({ map: &laminate_tex(salt, false) }));
    put(&aq, "Real__aquariumStandTop", &lambert!({ map: &laminate_tex(salt ^ 0x55, true) }));
    let black = phong!({ color: 0x1e1e1e, specular: 0x161616, shininess: 24 });
    put(&aq, "Real__aquariumKnobs", &black);
    put(&aq, "Real__aquariumTrim", &black);
    put(&aq, "Real__aquariumGravel", &lambert!({ map: &gravel_tex(salt, level.is_some()) }));
    put(&aq, "Real__aquariumAirline", &lambert!({ color: if level.is_some() { 0x5c5a48 } else { 0x8a8470 } }));
    put(&aq, "Real__aquariumAirstone", &lambert!({ color: if level.is_some() { 0x5c6248 } else { 0x9a958a } }));
    // the dead plant's stems lying over on the gravel, its leaves fallen
    let wet = level.is_some();
    let stem_mat = lambert!({ color: if wet { 0x4c4a36 } else { 0x6f665a } });
    let leaf_mat = lambert!({ map: &leaf_tex(salt, wet), side: DoubleSide });
    for (i, (kind, blades)) in plants.iter().enumerate() {
        let k = *kind;
        let [bh, bx, bz, br] = blades[0];
        let sx = -AQ_IN.0 * 0.8 + i as f64 * (AQ_IN.0 * 1.6 / (plant_count as f64 - 0.5)) + (bx - 0.5) * 0.010;
        let sz = (bz - 0.5) * (2.0 * AQ_IN.1 - 0.040);
        let scale = 0.88 + 0.12 * bh;
        let reach = STEM_REACH * scale;
        // over toward the side with more room, a little off the long way
        let toward = if sx < 0.0 { 1.0 } else { -1.0 };
        let room = AQ_IN.0 - toward * sx - 0.015;
        let lift = 0.24 - (gravel_top(sz) - AQ_FLOOR);
        let lean_lo = (lift / reach).clamp(-1.0, 1.0).acos() + 0.05;
        let lean_hi = (room / reach).clamp(-1.0, 1.0).asin();
        let lean = (lean_lo + (lean_hi - lean_lo).max(0.0) * 0.5).max(lean_lo).min(1.45);
        let az = if toward > 0.0 { 0.0 } else { PI } + (br - 0.5) * 0.5;
        let dir = v3(az.cos() * lean.sin(), lean.cos(), az.sin() * lean.sin());
        let stem = group();
        stem.quaternion().set_from_unit_vectors(&v3(0, 1, 0), &dir);
        stem.position().set(sx, gravel_top(sz) - 0.008, sz);
        stem.scale().set_scalar(scale);
        // the stem's own lean turned across the way it fell
        let inner = group();
        inner.rotation().y = STEM_LEAN_AZ[k] - PI / 2.0;
        stem.add(&inner);
        put(&inner, ["Real__deadPlantStemA", "Real__deadPlantStemB", "Real__deadPlantStemC"][k], &stem_mat);
        aq.add(&stem);
        for b in &blades[1..] {
            let [lh, lx, lz, lr] = *b;
            let fx = (lx - 0.5) * (2.0 * AQ_IN.0 - 0.06);
            let fz = (lz - 0.5) * (2.0 * AQ_IN.1 - 0.06);
            if let Some(m) = put(&aq, "Real__deadPlantFallen", &leaf_mat) {
                m.position().set(fx, gravel_top(fz) + 0.001, fz);
                m.rotation().y = lr * PI * 2.0;
                m.scale().set_scalar(0.85 + 0.25 * lh);
            }
        }
    }
    // what water is left: green, still, a film on it; the pump's bubbles
    // rising through it from the stone and breaking at its face
    let mut bubble_objs: Vec<(Obj, f64, f64, f64)> = vec![];
    let mut surface: Option<Obj> = None;
    if let Some(top) = level {
        let (iw, id) = (2.0 * AQ_IN.0 - 0.001, 2.0 * AQ_IN.1 - 0.001);
        let body =
            mesh(box_geo!(iw, top - AQ_FLOOR, id), lambert!({ color: 0x3a4526, transparent: true, opacity: 0.78, depthWrite: false }));
        body.position().set(0, (AQ_FLOOR + top) / 2.0, 0);
        body.set_render_order(1);
        aq.add(&body);
        let film = mesh(plane_geo!(iw, id), lambert!({ map: &scum_tex(salt), transparent: true, depthWrite: false }));
        film.rotation().x = -PI / 2.0;
        film.position().y = top + 0.0005;
        film.set_render_order(2);
        aq.add(&film);
        surface = Some(film);
        let rise = top - AQ_STONE.1;
        if rise > 0.02 {
            let bubble_mat =
                phong!({ color: 0xc8d0c0, specular: 0x8a8a8a, shininess: 120, transparent: true, opacity: 0.5, depthWrite: false });
            for d in &bubbles_d {
                let [r, bx, bz, by, offset, speed] = *d;
                let b = mesh(sphere_geo!(0.0015 + r * 0.0015, 6, 4), &bubble_mat);
                b.position().set(AQ_STONE.0 + (bx - 0.5) * 0.016, AQ_STONE.1 + by * rise, AQ_STONE.2 + (bz - 0.5) * 0.016);
                b.set_render_order(2);
                aq.add(&b);
                // an air stone's bubbles rise at 20-33 cm/s
                bubble_objs.push((b, offset * rise, 0.20 + speed * 0.13, rise));
            }
        }
    }
    // the tank: its glass, ringed by the levels the water fell through
    if let Some(m) = put(&aq, "Real__aquariumGlass", &tank_glass_mat(&tank_glass_tex(salt, level))) {
        m.set_render_order(3);
    }
    // the fish's old sway (when they were drawn): the water's face stirring
    // where the bubbles break
    if let Some(film) = surface {
        let y0 = film.pos().y;
        g.animate(move |_g, _dt, t| {
            film.position().y = y0 + 0.0006 * (t * 3.1 + ripple_phase).sin();
        });
    }
    // the bubbles' rise (the old animator, always there)
    g.animate(move |_g, dt, _t| {
        for (b, off, speed, rise) in bubble_objs.iter_mut() {
            *off = (*off + dt * *speed) % *rise;
            b.position().y = AQ_STONE.1 + *off;
        }
    });
    // the hood, its lamp still lit (the old 60%): the old light at its tube
    if lit {
        put(&aq, "Real__aquariumHood", &lambert!({ map: &hood_tex(salt), color: 0x2a2a2a }));
        put(&aq, "Real__aquariumTube", &basic_mat!({ color: 0xdfe9ec }));
        let light = point_light!(0xd0e0ea, 0.30, 1.8, 2);
        light.position().set(0, AQ_TUBE.0, AQ_TUBE.1);
        aq.add(&light);
    }
    aq.position().set(px, 0, pz);
    aq.rotation().y = turn;
    g.world.add(&aq);
    let p = aq.pos();
    g.pp.aquarium_positions.push(v3(p.x, 0, p.z));
}

/// Glass as the windows' and the wine glass's (`Patch::Glass`): the room
/// seen in it by Fresnel; its crust and film (the map's grey) a pale grey
fn tank_glass_mat(map: &Tex) -> Mat {
    let m = phong!({ color: 0xffffff, specular: 0x9a9a9a, shininess: 220, transparent: true, depthWrite: false });
    {
        let mut mm = m.m();
        mm.map = Some(map.clone());
        mm.patch = Patch::Glass;
        mm.look_settled = true;
        mm.name = "aquarium_glass".into();
    }
    {
        let pu = m.r().patch_uniforms.clone();
        let f = &mut pu.borrow_mut().f;
        f[0] = 0.40;
        f[4] = 0.5;
        f[5] = 0.6;
        f[6] = 0.3;
        f[7] = 0.0;
        // the scale and the film: how much of the map's grey, its colour
        f[8] = 0.85;
        f[9] = 0.40;
        f[10] = 0.39;
        f[11] = 0.34;
        f[12] = 0.0;
        f[13] = 0.07;
    }
    m
}

/// The tank's glass (v up its panes): a white line of scale at every level
/// the water stood at as it fell, the film between them heavier lower down,
/// below the water that is left a green-brown film; dust above
fn tank_glass_tex(salt: u32, level: Option<f64>) -> Tex {
    canvas_tex_opt(96, 160, false, move |ctx, w, h| {
        let seed = salt ^ 0x61a5;
        let v_of = |y: f64| (y - AQ_GLASS_Y.0) / AQ_GLASS_Y.1;
        let floor_v = v_of(AQ_FLOOR + AQ_GRAVEL.0 + AQ_GRAVEL.1);
        let full = v_of(0.885);
        let last = level.map(v_of).unwrap_or(floor_v);
        // the levels it stood at: from full down to where it is now
        let lines: Vec<f64> = (0..5)
            .map(|k| {
                let t = (k as f64 + 0.6 * (hash(seed, k, 1) - 0.5)) / 4.0;
                full - (full - last) * t.clamp(0.0, 1.0)
            })
            .collect();
        fill(ctx, w, h, |u, v| {
            let n = fbm(seed, u * 8.0, v * 10.0, 3);
            let mut g = 0.08 + 0.06 * n;
            if v < full {
                // the film the falling water left, heavier lower down
                g += 0.10 + 0.22 * (full - v) / (full - floor_v).max(0.05) * (0.6 + 0.5 * n);
            }
            for (k, &lv) in lines.iter().enumerate() {
                let wob = (vn(seed ^ k as u32, u * 30.0, 0.5) - 0.5) * 0.006;
                let d = (v - lv - wob).abs();
                let crust = 1.0 - sstep(0.001, 0.004 + 0.004 * hash(seed, k as i32, 2), d);
                let broken = sstep(0.3, 0.6, vn(seed ^ 0x44 ^ k as u32, u * 14.0, 0.5));
                g = g.max(crust * (0.45 + 0.30 * n) * (0.4 + 0.6 * broken));
            }
            if let Some(lv) = level {
                if v < v_of(lv) {
                    g = g.max(0.35 + 0.25 * fbm(seed ^ 0x9, u * 14.0, v * 14.0, 3));
                }
            }
            let g = g.clamp(0.0, 1.0);
            [g, g, g, 1.0]
        });
    })
}

/// The stand's oak-grain laminate (its doors and sides: u across, v up;
/// its top: u across, v front to back), the print's grain, worn
/// at the edges; on the cabinet, its foot where water stood: the laminate
/// lifted and gone, the particleboard swollen and crumbling, the tide marks
/// of the water above it; on the top, the white rings where the tank and
/// spills stood
fn laminate_tex(salt: u32, top: bool) -> Tex {
    canvas_tex_opt(128, 160, false, move |ctx, w, h| {
        let seed = salt ^ 0x1a31;
        fill(ctx, w, h, |u, v| {
            let (a, b) = if top { (u * 0.55, v * 0.30) } else { (v * 0.60, u * 0.55) };
            // the print of an oak: a broad tone, growth lines a little
            // waved, fine streaks
            let big = fbm(seed, a * 1.5, b * 5.0, 3);
            let warp = fbm(seed ^ 1, a * 3.0, b * 12.0, 3);
            let lines = ((b * 260.0 + warp * 5.0).sin() * 0.5 + 0.5).powf(3.0);
            let fine = fbm(seed ^ 9, a * 10.0, b * 90.0, 2);
            let mut c = mix([0.60, 0.46, 0.30], [0.44, 0.31, 0.18], 0.3 + 0.4 * (big - 0.5) + 0.35 * lines + 0.25 * (fine - 0.5));
            if top {
                // rings of scale where water stood round the tank's foot
                let edge = (u.min(1.0 - u) * 0.55).min(v.min(1.0 - v) * 0.30);
                let wet = 1.0 - sstep(0.010, 0.030, edge);
                let tide = ((edge * 900.0 + fbm(seed ^ 3, u * 8.0, v * 8.0, 2) * 6.0).sin() * 0.5 + 0.5).powf(6.0);
                c = mix(c, [0.70, 0.68, 0.62], wet * tide * 0.6);
                c = mix(c, [0.36, 0.30, 0.22], wet * 0.25);
            } else {
                // the foot: laminate lifted off the swollen chipboard
                let ragged = 0.07 + 0.04 * fbm(seed ^ 0x77, u * 9.0, 0.5, 3);
                if v < ragged {
                    let chip = vn(seed ^ 0x5, u * 320.0, v * 320.0);
                    c = mix([0.55, 0.46, 0.33], [0.36, 0.28, 0.18], chip);
                    c = mix(c, [0.26, 0.21, 0.15], 1.0 - sstep(0.0, 0.04, v));
                } else {
                    // tide marks above it, brown
                    for k in 0..3 {
                        let lv = ragged + 0.03 + 0.035 * k as f64 + 0.01 * fbm(seed ^ k, u * 6.0, 0.3, 2);
                        let d = (v - lv).abs();
                        c = mix(c, [0.30, 0.22, 0.14], (1.0 - sstep(0.0, 0.006, d)) * 0.5);
                    }
                }
            }
            // dust and grime over it
            let bloom = 0.10 + 0.10 * fbm(seed ^ 0x31, u * 3.0, v * 3.0, 2);
            c = mix(c, [0.45, 0.42, 0.37], bloom);
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// Aquarium gravel (its top: u across, v front to back): rounded pebbles
/// of 3-6 mm in river colours; dry, crusted white with dried algae; under
/// water, filmed green-brown
fn gravel_tex(salt: u32, wet: bool) -> Tex {
    canvas_tex_opt(256, 128, false, move |ctx, w, h| {
        let seed = salt ^ 0x9a7e;
        fill(ctx, w, h, |u, v| {
            let (x, y) = (u * 0.50, v * 0.255);
            let s = 220.0;
            let (cx, cy) = ((x * s).floor(), (y * s).floor());
            // the nearest pebble's middle (one in each cell, jittered)
            let mut best = (f64::INFINITY, 0, 0);
            for dj in -1..=1 {
                for di in -1..=1 {
                    let (i, j) = (cx as i32 + di, cy as i32 + dj);
                    let px = (i as f64 + hash(seed, i, j)) / s;
                    let py = (j as f64 + hash(seed ^ 1, i, j)) / s;
                    let d = ((px - x).powi(2) + (py - y).powi(2)).sqrt() * s;
                    if d < best.0 {
                        best = (d, i, j);
                    }
                }
            }
            let (d, i, j) = best;
            let tone = hash(seed ^ 2, i, j);
            let mut c = if tone < 0.4 {
                [0.52, 0.46, 0.38]
            } else if tone < 0.7 {
                [0.40, 0.38, 0.35]
            } else if tone < 0.9 {
                [0.58, 0.52, 0.42]
            } else {
                [0.30, 0.26, 0.22]
            };
            // round: darker toward its rim, a gap between pebbles
            let shade = 1.0 - sstep(0.35, 0.75, d) * 0.55;
            c = c.map(|k| k * shade);
            let n = fbm(seed ^ 4, u * 9.0, v * 5.0, 3);
            if wet {
                c = mix(c, [0.22, 0.25, 0.15], 0.35 + 0.25 * n);
            } else {
                c = mix(c, [0.70, 0.68, 0.62], (n - 0.35).max(0.0) * 0.9);
                c = mix(c, [0.38, 0.36, 0.28], 0.15);
            }
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// The film on still water: a grey-green scum, thicker in patches and at
/// the glass, dust and a few specks on it, a clear patch where the bubbles
/// break (straight sRGB and alpha)
fn scum_tex(salt: u32) -> Tex {
    canvas_tex_opt(128, 64, false, move |ctx, w, h| {
        let seed = salt ^ 0x5c0;
        fill(ctx, w, h, |u, v| {
            let n = fbm(seed, u * 6.0, v * 3.0, 4);
            let edge = 1.0 - sstep(0.0, 0.08, u.min(1.0 - u).min(v.min(1.0 - v) * 0.5));
            // the stone at the back left: the bubbles break over it
            let (bx, by) = ((AQ_STONE.0 + AQ_IN.0) / (2.0 * AQ_IN.0), (AQ_STONE.2 + AQ_IN.1) / (2.0 * AQ_IN.1));
            let clear = 1.0 - sstep(0.04, 0.12, ((u - bx) * 2.0).hypot(v - by));
            let a = ((0.12 + 0.40 * (n - 0.3).max(0.0) + 0.25 * edge) * (1.0 - 0.8 * clear)).clamp(0.0, 0.7);
            let speck = vn(seed ^ 3, u * 120.0, v * 60.0);
            let c = if speck > 0.85 { [0.20, 0.18, 0.14] } else { [0.30, 0.33, 0.22] };
            [c[0], c[1], c[2], a]
        });
    })
}

/// Dust on the black hood (on its colour: how bright), heavier on its top
fn hood_tex(salt: u32) -> Tex {
    canvas_tex_opt(64, 64, false, move |ctx, w, h| {
        let seed = salt ^ 0x40d;
        fill(ctx, w, h, |u, v| {
            let n = fbm(seed, u * 7.0, v * 7.0, 3);
            let g = 0.55 + 0.45 * n;
            [g, g, g, 1.0]
        });
    })
}

/// A dead leaf (r18's fallen pelargonium leaf, as it lies: u across, v
/// along): dry and brown, its veins darker; under water, gone dark and
/// filmed green
fn leaf_tex(salt: u32, wet: bool) -> Tex {
    canvas_tex_opt(32, 32, false, move |ctx, w, h| {
        let seed = salt ^ 0x1eaf;
        fill(ctx, w, h, |u, v| {
            let n = fbm(seed, u * 5.0, v * 5.0, 3);
            let (dx, dy) = (u - 0.5, v - 0.5);
            let ang = dy.atan2(dx);
            let vein = ((ang * 4.0).sin().abs() < 0.08) as i32 as f64 * 0.4;
            let mut c = mix([0.50, 0.38, 0.24], [0.36, 0.26, 0.16], n + vein);
            if wet {
                c = mix(c, [0.18, 0.20, 0.12], 0.6);
            }
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// The painting kit (per pixel, pure: safe for `par_rows` on every core), a
/// salt from where a thing is, a part placed.
mod aq_kit {
    use game::prelude::*;

    pub(super) fn hash(seed: u32, i: i32, j: i32) -> f64 {
        let mut h = seed ^ (i as u32).wrapping_mul(0x27d4_eb2d) ^ (j as u32).wrapping_mul(0x1656_67b1);
        h = (h ^ (h >> 15)).wrapping_mul(0x85eb_ca6b);
        h = (h ^ (h >> 13)).wrapping_mul(0xc2b2_ae35);
        h ^= h >> 16;
        f64::from(h) / 4_294_967_296.0
    }
    pub(super) fn vn(seed: u32, x: f64, y: f64) -> f64 {
        let (xf, yf) = (x.floor(), y.floor());
        let (i, j) = (xf as i32, yf as i32);
        let (u, v) = (x - xf, y - yf);
        let (u, v) = (u * u * (3.0 - 2.0 * u), v * v * (3.0 - 2.0 * v));
        let (a, b) = (hash(seed, i, j), hash(seed, i + 1, j));
        let (c, d) = (hash(seed, i, j + 1), hash(seed, i + 1, j + 1));
        a + (b - a) * u + (c - a) * v + (a - b - c + d) * u * v
    }
    pub(super) fn fbm(seed: u32, x: f64, y: f64, oct: u32) -> f64 {
        let (mut s, mut amp, mut f, mut n) = (0.0, 0.5, 1.0, 0.0);
        for k in 0..oct {
            s += amp * vn(seed.wrapping_add(k * 7919), x * f, y * f);
            n += amp;
            amp *= 0.5;
            f *= 2.03;
        }
        s / n
    }
    pub(super) fn sstep(e0: f64, e1: f64, x: f64) -> f64 {
        let t = ((x - e0) / (e1 - e0)).clamp(0.0, 1.0);
        t * t * (3.0 - 2.0 * t)
    }
    pub(super) fn mix(a: [f64; 3], b: [f64; 3], t: f64) -> [f64; 3] {
        let t = t.clamp(0.0, 1.0);
        [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
    }
    /// Fills the canvas pixel by pixel on every core: `f(u, v)` (u right, v
    /// up, both 0..1, as the texture is laid) gives straight sRGB and alpha
    pub(super) fn fill(ctx: &mut Canvas, w: f64, h: f64, f: impl Fn(f64, f64) -> [f64; 4] + Sync) {
        let iw = w as usize;
        let mut img = ctx.create_image_data(w, h);
        par_rows(&mut img.data, iw * 4, |j, out| {
            for i in 0..iw {
                let c = f((i as f64 + 0.5) / w, 1.0 - (j as f64 + 0.5) / h);
                for k in 0..4 {
                    out[i * 4 + k] = (c[k].clamp(0.0, 1.0) * 255.0).round() as u8;
                }
            }
        });
        ctx.put_image_data(&img, 0, 0);
    }
    /// A salt from where a thing is (no draw): its painters' strokes differ
    /// from floor to floor
    pub(super) fn salt(x: f64, z: f64, k: u32) -> u32 {
        ((x * 7919.0).abs() as u32) ^ ((z * 104_729.0).abs() as u32).rotate_left(13) ^ k.wrapping_mul(0x9e37_79b9)
    }
    /// The part `name` of a Blender batch as modelled, added to `grp` in `mat`
    pub(super) fn put(grp: &Obj, name: &str, mat: &Mat) -> Option<Obj> {
        let geo = game::assets::props::prop_part_as_modelled(name)?;
        let m = mesh(geo, mat.clone());
        grp.add(&m);
        Some(m)
    }
}
