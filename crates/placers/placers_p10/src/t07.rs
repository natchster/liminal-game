//! Batch `t07`'s wall coat rack and upright vacuum cleaner, made real
//! (`docs/design/PROPS.md` P3): their placers and what only they use.
use super::*;
use t07_kit::*;

/// The wall's face in a wall prop's frame (its group stands 15 mm out)
const WALL_Z: f64 = -0.015;
/// A hook's cradle, where a loop rests: its top, out from the wall (batch
/// `t07`'s brass hooks, as r06's: r06's coat and r13b's scarf hang there)
const CRADLE: (f64, f64) = (-0.0005, 0.077);

/// A copy of `geo` pushed out from the wall below its top: a coat hung
/// over the one under it (its loop on its own hook, its body lying on the
/// other's)
fn pushed(geo: &Geo, push: f64) -> Geo {
    let out = geo.duplicate();
    {
        let mut gg = out.g();
        for i in 0..gg.position.len() / 3 {
            let y = gg.position[i * 3 + 1] as f64;
            gg.position[i * 3 + 2] += (push * sstep(0.0, -0.32, y)) as f32;
        }
        gg.touch();
    }
    out.compute_bounding_box();
    out.compute_bounding_sphere();
    out
}

// Wall coat rack w/ 2-3 hanging coats: 3% chance per level. A 24 in
// board of stained hardwood screwed to the wall, five cast brass hooks on
// it (batch `t07`), two or three wool overcoats hung by their loops (r06's,
// one over the next where the hooks are close), now and then a knitted
// scarf thrown over a hook (r13b's).
pub(crate) fn place_coat_racks(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.97 {
        return;
    }
    let wall_tiles = wall_tiles(g);
    if wall_tiles.is_empty() {
        return;
    }
    let (x, z, dx, dz) = wall_tiles[(rnd() * wall_tiles.len() as f64).floor() as usize];
    // the old wood draw: a stain (mid oak, walnut, golden oak, dark)
    let wood_colors = [0x8b5a2b, 0x604020, 0xa07040, 0x504030];
    let wc = pick(&wood_colors);
    let wi = wood_colors.iter().position(|&c| c == wc).unwrap_or(0);
    let salt = salt(g.wx(x), g.wz(z), wi as u32);
    let stain = [[0.40, 0.26, 0.15], [0.26, 0.17, 0.11], [0.50, 0.36, 0.22], [0.24, 0.19, 0.14]][wi];
    let rack = group();
    let mount_y = 1.75;
    // the wall's frame: the board's back on the face, its middle at mount_y
    let wall = group();
    wall.position().set(0, mount_y, WALL_Z);
    put(&wall, "Real__coatRackBoard", &lambert!({ map: &board_tex(stain, salt) }));
    put(&wall, "Real__coatRackScrews", &phong!({ color: 0x5c5850, specular: 0x3a3a36, shininess: 30 }));
    put(&wall, "Real__coatRackPegs", &phong!({ color: 0x6a5632, specular: 0x5a4a2c, shininess: 34 }));
    rack.add(&wall);
    let peg_positions: Vec<f64> = (0..5).map(|i| -0.24 + i as f64 * 0.12).collect();
    // the old coat colours (rich, saturated) as wool after years
    let coat_colors = [0x603030, 0x101020, 0x408060, 0x201010, 0xa02040, 0x203060, 0x604020, 0x2a2a2a];
    let wools = [0x5a3a36u32, 0x2f3442, 0x4a5444, 0x3a2e2a, 0x5e2e32, 0x3e4858, 0x7a6249, 0x464648];
    let wool_map = wool_tex(salt);
    let horn = phong!({ color: 0x2a2018, specular: 0x202020, shininess: 40 });
    // Hang 2-3 coats on random pegs (deduplicated).
    let coat_count = 2 + (rnd() * 2.0).floor() as i32;
    let mut used_pegs: Vec<usize> = vec![]; // a Set
    for _c in 0..coat_count {
        let mut peg_idx;
        let mut attempts = 0;
        loop {
            peg_idx = (rnd() * peg_positions.len() as f64).floor() as usize;
            attempts += 1;
            if !(used_pegs.contains(&peg_idx) && attempts < 10) {
                break;
            }
        }
        if used_pegs.contains(&peg_idx) {
            break;
        }
        let px = peg_positions[peg_idx];
        // over how many coats hung before it, near enough to lie on
        let layer = used_pegs.iter().filter(|&&j| (1..4).contains(&j) && (peg_positions[j] - px).abs() < 0.45).count();
        used_pegs.push(peg_idx);
        let cc = pick(&coat_colors);
        let ci = coat_colors.iter().position(|&c| c == cc).unwrap_or(0);
        let coat = group();
        if (1..4).contains(&peg_idx) {
            // an overcoat, its loop on the hook's cradle, slid out a little
            // over another's
            coat.position().set(px, CRADLE.0, CRADLE.1 + 0.004 * layer as f64);
            let push = 0.06 * layer as f64;
            if let Some(geo) = game::assets::props::prop_part_as_modelled("Real__coatHung") {
                let geo = if layer > 0 { pushed(&geo, push) } else { geo };
                coat.add(&mesh(geo, lambert!({ map: &wool_map, color: wools[ci], side: DoubleSide })));
            }
            if let Some(geo) = game::assets::props::prop_part_as_modelled("Real__hookCoatButtons") {
                let geo = if layer > 0 { pushed(&geo, push) } else { geo };
                coat.add(&mesh(geo, &horn));
            }
        } else {
            // on an end hook, where a coat would stand out past the board
            // (and a wall's end), a scarf thrown over it
            coat.position().set(px, CRADLE.0, CRADLE.1);
            if let Some(geo) = game::assets::props::prop_part_as_modelled("Real__scarfHung") {
                let s = mesh(geo, lambert!({ map: &knit_tex(salt ^ ci as u32), color: wools[ci], side: DoubleSide }));
                s.position().set(0, 0.0465, -0.058);
                coat.add(&s);
            }
        }
        wall.add(&coat);
        // Subtle sway animator — 35% chance per coat.
        if rnd() < 0.35 {
            let sway_phase = rnd() * PI * 2.0;
            let sway_phase2 = rnd() * PI * 2.0;
            let coat2 = coat.clone();
            g.animate(move |_g, _dt, t| {
                coat2.rotation().x = (t * 0.5 + sway_phase).sin() * 0.015;
                coat2.rotation().z = (t * 0.4 + sway_phase2).sin() * 0.010;
            });
        }
    }
    // Optional: a scarf thrown over one empty hook (40% if empty pegs exist).
    if used_pegs.len() < 5 && rnd() < 0.4 {
        let mut scarf_peg_idx;
        loop {
            scarf_peg_idx = (rnd() * peg_positions.len() as f64).floor() as usize;
            if !used_pegs.contains(&scarf_peg_idx) {
                break;
            }
        }
        let scarf_px = peg_positions[scarf_peg_idx];
        let scarf_colors = [0xa02040, 0x408060, 0xd0a040, 0x203060, 0xf0e0a0];
        let sc = pick(&scarf_colors);
        let wool = [0x6a3034u32, 0x4a5a48, 0x8a7244, 0x3c4656, 0x9a9282][scarf_colors.iter().position(|&c| c == sc).unwrap_or(0)];
        if let Some(geo) = game::assets::props::prop_part_as_modelled("Real__scarfHung") {
            // its frame is r06's hook's: the arm's top at (0, -0.0465, 0.058)
            let s = mesh(geo, lambert!({ map: &knit_tex(salt ^ 0x5ca), color: wool, side: DoubleSide }));
            s.position().set(scarf_px, CRADLE.0 + 0.0465, CRADLE.1 - 0.058);
            wall.add(&s);
        }
    }
    rack.rotation().y = (-(dx as f64)).atan2(-(dz as f64));
    rack.position().set(g.wx(x) + dx as f64 * (CELL * 0.5 - 0.015), 0, g.wz(z) + dz as f64 * (CELL * 0.5 - 0.015));
    g.world.add(&rack);
}

/// The board's face (u along it, v up it): stained hardwood, its grain
/// running along, the varnish worn pale where hands reached for the hooks,
/// grime in the bead and along its foot, dust on its top edge
fn board_tex(stain: [f64; 3], salt: u32) -> Tex {
    canvas_tex_opt(512, 128, false, move |ctx, w, h| {
        let seed = salt ^ 0xb0a;
        fill(ctx, w, h, |u, v| {
            let grain = fbm(seed, u * 6.0, v * 40.0 + fbm(seed ^ 1, u * 3.0, v * 3.0, 2) * 6.0, 4);
            let lines = (vn(seed ^ 2, u * 3.0, v * 160.0) - 0.5).abs();
            let mut c = mix(stain, [stain[0] * 0.62, stain[1] * 0.6, stain[2] * 0.58], grain * 1.3 - 0.2);
            c = mix(c, [stain[0] * 0.5, stain[1] * 0.48, stain[2] * 0.46], sstep(0.12, 0.0, lines) * 0.5);
            // rubbed pale round each hook, where coats were taken down
            let mut rub: f64 = 0.0;
            for k in 0..5 {
                let du = (u - (0.5 + (-0.24 + 0.12 * k as f64) / 0.61)) * 0.61 / 0.03;
                let dv = (v - 0.5) * 0.09 / 0.03;
                rub = rub.max((-(du * du + dv * dv)).exp());
            }
            c = mix(c, mix(stain, [0.62, 0.56, 0.46], 0.4), rub * 0.45);
            c = mix(c, [0.16, 0.13, 0.10], sstep(0.12, 0.0, v) * 0.4 + sstep(0.86, 1.0, v) * 0.15);
            c = mix(c, [0.44, 0.42, 0.38], fbm(seed ^ 7, u * 9.0, v * 4.0, 3) * 0.18);
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// Wool cloth after years on a hook (white, for its colour to tint): a
/// twill, fuzz and pills, faded on the shoulders, dust lying on them
fn wool_tex(salt: u32) -> Tex {
    canvas_tex_opt(256, 256, true, move |ctx, w, h| {
        let seed = salt ^ 0x3001;
        fill(ctx, w, h, |u, v| {
            let tw = ((u * w + v * h) / 3.0).fract();
            let twill = if tw < 0.5 { 0.94 } else { 1.0 };
            let n = fbm(seed, u * 16.0, v * 16.0, 4);
            let pill = vn(seed ^ 9, u * 140.0, v * 140.0);
            let k = twill * (0.86 + 0.14 * n) * if pill > 0.88 { 1.08 } else { 1.0 };
            let dust = sstep(0.75, 1.0, v) * 0.12;
            [k * (1.0 - dust) + dust * 0.9, k * (1.0 - dust) + dust * 0.88, k * (1.0 - dust) + dust * 0.84, 1.0]
        });
    })
}

/// A hand-knitted scarf (white, tinted): ribbed down its length, fuzz and
/// pills, a moth hole
fn knit_tex(salt: u32) -> Tex {
    canvas_tex_opt(128, 256, false, move |ctx, w, h| {
        let seed = salt ^ 0x9e7;
        fill(ctx, w, h, |u, v| {
            let rib = (u * 34.0).fract();
            let stitch = ((v * 120.0 + if rib < 0.5 { 0.0 } else { 0.5 }).fract() - 0.5).abs();
            let k = (if rib < 0.42 { 0.84 } else { 1.0 }) * (0.92 + 0.08 * stitch) * (0.9 + 0.1 * fbm(seed, u * 10.0, v * 20.0, 3));
            [k, k * 0.99, k * 0.97, 1.0]
        });
        for _ in 0..(salt % 7) {
            let _ = rnd();
        }
        ctx.set_fill_style("rgba(10,8,6,0.8)");
        ctx.begin_path();
        ctx.ellipse(rnd() * w, h * (0.3 + 0.6 * rnd()), 1.5 + rnd() * 1.5, 1.0 + rnd(), rnd() * 3.0, 0, 7);
        ctx.fill();
    })
}

/// The vacuum's cord where it hangs free of its lower hook (batch `t07`:
/// `CORD_FREE`, off the hook's cradle)
const VAC_CORD_FREE: (f64, f64, f64) = (-0.010, 0.330, -0.176_609);
const VAC_CORD_R: f64 = 0.0034;

/// A round cord along `pts` (no caps): rings carried along by parallel
/// transport, `sides` round
fn cord_geo(pts: &[Vec3], r: f64, sides: usize) -> Geo {
    let n = pts.len();
    let mut pos: Vec<f32> = Vec::with_capacity(n * sides * 3);
    let mut nrm: Vec<f32> = Vec::with_capacity(n * sides * 3);
    let tan = |i: usize| {
        let mut t = pts[(i + 1).min(n - 1)];
        t.sub(&pts[i.saturating_sub(1)]);
        t.normalize();
        t
    };
    let t0 = tan(0);
    let mut side = if t0.y.abs() < 0.9 { v3(0, 1, 0) } else { v3(1, 0, 0) };
    side.cross(&t0).normalize();
    for i in 0..n {
        let t = tan(i);
        // keep the side square to the path
        let d = side.dot(&t);
        let mut s2 = side;
        s2.sub(&v3(t.x * d, t.y * d, t.z * d)).normalize();
        side = s2;
        let mut b = t;
        b.cross(&side).normalize();
        for k in 0..sides {
            let a = k as f64 / sides as f64 * PI * 2.0;
            let (c, s) = (a.cos(), a.sin());
            let nx = side.x * c + b.x * s;
            let ny = side.y * c + b.y * s;
            let nz = side.z * c + b.z * s;
            pos.extend_from_slice(&[(pts[i].x + nx * r) as f32, (pts[i].y + ny * r) as f32, (pts[i].z + nz * r) as f32]);
            nrm.extend_from_slice(&[nx as f32, ny as f32, nz as f32]);
        }
    }
    let mut idx: Vec<u32> = vec![];
    for i in 0..n - 1 {
        for k in 0..sides {
            let k2 = (k + 1) % sides;
            let (a, b, c, d) = (i * sides + k, i * sides + k2, (i + 1) * sides + k2, (i + 1) * sides + k);
            idx.extend_from_slice(&[a as u32, d as u32, c as u32, a as u32, c as u32, b as u32]);
        }
    }
    let geo = Geo::new();
    let nv = pos.len() / 3;
    geo.set_position(pos);
    geo.set_normal(nrm);
    geo.set_uv(vec![0.0; nv * 2]);
    geo.set_index(idx);
    geo.compute_bounding_box();
    geo.compute_bounding_sphere();
    geo
}

/// Points of a Catmull-Rom curve through `c`, `per` to a span
fn catmull(c: &[Vec3], per: usize) -> Vec<Vec3> {
    let mut out = vec![];
    let n = c.len();
    for i in 0..n - 1 {
        let p0 = c[i.saturating_sub(1)];
        let (p1, p2) = (c[i], c[i + 1]);
        let p3 = c[(i + 2).min(n - 1)];
        for k in 0..per {
            let t = k as f64 / per as f64;
            let (t2, t3) = (t * t, t * t * t);
            let f = |a: f64, b: f64, cc: f64, d: f64| {
                0.5 * (2.0 * b + (-a + cc) * t + (2.0 * a - 5.0 * b + 4.0 * cc - d) * t2 + (-a + 3.0 * b - 3.0 * cc + d) * t3)
            };
            out.push(v3(f(p0.x, p1.x, p2.x, p3.x), f(p0.y, p1.y, p2.y, p3.y), f(p0.z, p1.z, p2.z, p3.z)));
        }
    }
    out.push(c[n - 1]);
    out
}

// Vacuum cleaner unattended (floor): 3% chance per level.
// Track positions for optional motor whine audio. An upright of the
// Hoover Convertible kind (batch `t07`: 45 x 13 x 12 in), stood with its
// handle locked up: the hood on its wheels, its headlight dark, the vinyl
// bumper; the chromed handle; the cloth bag hung from its hook, sagging
// full; the cord wound on its two hooks, its end let down to trail across
// the floor to its plug.
pub(crate) fn place_vacuum_cleaners(g: &mut Game, _dist: &[i32]) {
    g.pp.vacuum_positions = vec![]; // reset per level
    if rnd() < 0.97 {
        return;
    }
    let open_tiles = open_tiles(g);
    if open_tiles.is_empty() {
        return;
    }
    let (x, z) = open_tiles[(rnd() * open_tiles.len() as f64).floor() as usize];
    // the old body colour draw: burgundy, slate blue, grey, black, harvest
    // gold, avocado (an appliance's colours, gone dull)
    let body_colors = [0xa02040, 0x2060a0, 0x606060, 0x101010, 0xd0a040, 0x40a060];
    let bc = pick(&body_colors);
    let bi = body_colors.iter().position(|&c| c == bc).unwrap_or(0);
    let hood_col =
        [[0.36, 0.17, 0.18], [0.23, 0.29, 0.36], [0.42, 0.42, 0.40], [0.15, 0.15, 0.14], [0.52, 0.41, 0.20], [0.33, 0.37, 0.22]][bi];
    let bag_col =
        [[0.52, 0.50, 0.46], [0.42, 0.45, 0.50], [0.54, 0.52, 0.48], [0.36, 0.35, 0.33], [0.58, 0.52, 0.40], [0.48, 0.49, 0.40]][bi];
    let v = group();
    let salt = salt(g.wx(x), g.wz(z), bi as u32);
    put(&v, "Real__vacuumHood", &phong!({ map: &vac_plastic_tex(hood_col, salt), specular: 0x2a2a2a, shininess: 24 }));
    let rubber = lambert!({ color: 0x2a2826 });
    put(&v, "Real__vacuumRubber", &rubber);
    put(&v, "Real__vacuumLens", &phong!({ color: 0x8a8476, specular: 0x5a5a56, shininess: 60 }));
    put(&v, "Real__vacuumMetal", &phong!({ color: 0x5e5e5c, specular: 0x7a7a76, shininess: 60 }));
    put(&v, "Real__vacuumGrip", &phong!({ color: 0x232120, specular: 0x262626, shininess: 18 }));
    put(&v, "Real__vacuumBag", &lambert!({ map: &vac_bag_tex(bag_col, salt), side: DoubleSide }));
    put(&v, "Real__vacuumFillTube", &lambert!({ color: 0x2e2c2a }));
    let cord_mat = lambert!({ color: 0x3a3936 });
    put(&v, "Real__vacuumCord", &cord_mat);
    // the cord's end let down off its hook, trailing over the floor to the
    // plug where it was pulled from the wall (the old end's two draws)
    let cord_end = v3(-0.35 - rnd() * 0.15, 0.0, 0.15 + rnd() * 0.10);
    let fy = VAC_CORD_R;
    let (f0x, f0y, f0z) = VAC_CORD_FREE;
    let ctrl = [
        v3(f0x, f0y, f0z),
        v3(f0x - 0.004, 0.13, f0z - 0.040),
        v3(-0.05, fy, -0.30),
        v3(-0.22 - (cord_end.x + 0.42) * 0.5, fy, -0.30 + (cord_end.z - 0.2) * 0.3),
        v3(cord_end.x - 0.07, fy, -0.06 + (cord_end.z - 0.2) * 0.5),
        v3(cord_end.x + 0.01, fy, cord_end.z - 0.08),
        v3(cord_end.x, 0.006, cord_end.z),
    ];
    // (the curve may swing under the floor where it comes down onto it)
    let pts: Vec<Vec3> = catmull(&ctrl, 10).into_iter().map(|p| v3(p.x, p.y.max(VAC_CORD_R), p.z)).collect();
    v.add(&mesh(cord_geo(&pts, VAC_CORD_R, 6), &cord_mat));
    let plug = group();
    let n = pts.len();
    let (pa, pb) = (pts[n - 2], pts[n - 1]);
    plug.position().set(pb.x, 0, pb.z);
    plug.rotation().y = (pb.x - pa.x).atan2(pb.z - pa.z);
    put(&plug, "Real__vacuumPlug", &lambert!({ color: 0x2c2a28 }));
    put(&plug, "Real__vacuumPlugBlades", &phong!({ color: 0x7a6e58, specular: 0x5a5244, shininess: 40 }));
    v.add(&plug);
    v.position().set(g.wx(x) + (rnd() - 0.5) * 1.4, 0, g.wz(z) + (rnd() - 0.5) * 1.4);
    v.rotation().y = rnd() * PI * 2.0;
    g.world.add(&v);
    // Register position for motor whine audio.
    let p = v.pos();
    g.pp.vacuum_positions.push(v3(p.x, 0, p.z));
}

/// The hood's moulded plastic (laid on it as on a box: its top in plan):
/// gone chalky, dust lying on top, scuffs and the grey of the skirting it
/// was run into along its front and sides
fn vac_plastic_tex(col: [f64; 3], salt: u32) -> Tex {
    canvas_tex_opt(256, 256, false, move |ctx, w, h| {
        let seed = salt ^ 0x7ac;
        fill(ctx, w, h, |u, v| {
            let n = fbm(seed, u * 8.0, v * 8.0, 4);
            let mut c = mix(col, [col[0] * 0.8, col[1] * 0.8, col[2] * 0.8], n);
            c = mix(c, [0.55, 0.53, 0.49], 0.12 + 0.2 * fbm(seed ^ 3, u * 3.0, v * 3.0, 3));
            let scuff = vn(seed ^ 5, u * 60.0, v * 8.0);
            if scuff > 0.8 {
                c = mix(c, [0.62, 0.60, 0.56], (scuff - 0.8) * 2.5);
            }
            c = mix(c, [0.20, 0.19, 0.17], sstep(0.2, 0.0, v.min(1.0 - v)) * 0.35);
            [c[0], c[1], c[2], 1.0]
        });
    })
}

/// The cloth bag (laid on it from the front, u across, v up): a coated
/// twill, two printed bands across it gone faint, the zip's track down its
/// front, grey with the dust through it low down where it sags, dark at
/// its foot where it is clamped
fn vac_bag_tex(col: [f64; 3], salt: u32) -> Tex {
    canvas_tex_opt(256, 512, false, move |ctx, w, h| {
        let seed = salt ^ 0xba9;
        fill(ctx, w, h, |u, v| {
            let tw = ((u * w * 0.8 + v * h * 0.8) / 2.0).fract();
            let n = fbm(seed, u * 6.0, v * 12.0, 4);
            let mut c = mix(col, [col[0] * 0.8, col[1] * 0.8, col[2] * 0.78], n * 1.2 - 0.1);
            c = [
                c[0] * if tw < 0.5 { 0.95 } else { 1.0 },
                c[1] * if tw < 0.5 { 0.95 } else { 1.0 },
                c[2] * if tw < 0.5 { 0.95 } else { 1.0 },
            ];
            // the bands, printed, rubbed thin
            for (b0, b1) in [(0.70, 0.715), (0.73, 0.79)] {
                if v > b0 && v < b1 {
                    c = mix(c, [col[0] * 0.55, col[1] * 0.5, col[2] * 0.5], 0.5 * (0.6 + 0.4 * fbm(seed ^ 7, u * 30.0, v * 30.0, 2)));
                }
            }
            // the zip's track down the front's middle
            let zd = (u - 0.5).abs();
            if zd < 0.012 && v > 0.1 && v < 0.94 {
                let tooth = ((v * h / 2.5).fract() < 0.5) as i32 as f64;
                c = mix(c, [0.24, 0.23, 0.22], 0.6 + 0.2 * tooth);
            }
            // dust through the cloth where it sags, grime at its foot
            c = mix(c, [0.46, 0.45, 0.42], sstep(0.45, 0.15, v) * 0.35 * fbm(seed ^ 9, u * 5.0, v * 5.0, 3));
            c = mix(c, [0.18, 0.17, 0.15], sstep(0.08, 0.0, v) * 0.7);
            [c[0], c[1], c[2], 1.0]
        });
    })
}
/// The painting kit of batch `t07`'s painters: value noise and a per-pixel
/// fill (pure, so `par_rows` may run it on every core), soft spots.
#[allow(dead_code)]
mod t07_kit {
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
    /// sRGB 0..1 of a hex colour
    pub(super) fn rgb(c: u32) -> [f64; 3] {
        [((c >> 16) & 255) as f64 / 255.0, ((c >> 8) & 255) as f64 / 255.0, (c & 255) as f64 / 255.0]
    }
    pub(super) fn css(c: [f64; 3], a: f64) -> String {
        let k = |v: f64| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
        format!("rgba({},{},{},{:.3})", k(c[0]), k(c[1]), k(c[2]), a.clamp(0.0, 1.0))
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
    /// A soft spot at `at`, `r` px, `k` times as tall as wide, turned `turn`,
    /// fading out from its middle (the context squashed, so no edge cuts it)
    pub(super) fn soft(ctx: &mut Canvas, at: (f64, f64), r: f64, k: f64, turn: f64, c: [f64; 3], a: f64) {
        ctx.save();
        ctx.translate(at.0, at.1);
        ctx.rotate(turn);
        ctx.scale(1, k);
        let gr = ctx.create_radial_gradient(0, 0, 0, 0, 0, r);
        gr.add_color_stop(0, &css(c, a));
        gr.add_color_stop(1, &css(c, 0.0));
        ctx.set_fill_style(&gr);
        ctx.begin_path();
        ctx.arc(0, 0, r, 0, 7);
        ctx.fill();
        ctx.restore();
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
