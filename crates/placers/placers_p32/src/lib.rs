//! Prop placers: wall vents, dropped shoes, wall sockets and wall stains.
#![allow(unused_variables, unused_mut, unused_assignments, unused_imports, dead_code, clippy::all)]
use game::prelude::*;

/// Every placer in this crate, by its registered name.
pub fn register(r: &mut Vec<(&'static str, PlacerFn)>) {
    r.push(("place_wall_vents", place_wall_vents));
    r.push(("place_dropped_shoes", place_dropped_shoes));
    r.push(("place_wall_sockets", place_wall_sockets));
    r.push(("place_wall_stains", place_wall_stains));
}

/// A wall facing: open tile (x, z) with a wall (or the grid edge) at (x+dx, z+dz).
#[derive(Clone, Copy)]
struct Cand {
    x: i32,
    z: i32,
    dx: i32,
    dz: i32,
}

/// The wall-facing candidate scan every wall placer repeats.
fn wall_candidates(g: &Game) -> Vec<Cand> {
    let mut candidates = vec![];
    for z in 1..g.h - 1 {
        for x in 1..g.w - 1 {
            if g.tiles[g.t_idx(x, z)] != 0 {
                continue;
            }
            for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let (nx, nz) = (x + dx, z + dz);
                // off the grid or a wall tile
                if g.is_wall_tile(nx, nz) {
                    candidates.push(Cand { x, z, dx, dz });
                }
            }
        }
    }
    candidates
}

/// In-place Fisher-Yates: for i from n-1 down to 1, swap with j = floor(rnd() * (i+1)).
fn shuffle<T>(v: &mut [T]) {
    if v.len() < 2 {
        return;
    }
    for i in (1..v.len()).rev() {
        let j = (rnd() * (i + 1) as f64).floor() as usize;
        v.swap(i, j);
    }
}

/// Shoe colour per weather (else the normal entry)
fn shoe_color_by_weather(key: &str) -> u32 {
    by_weather(key, 0x2a2018, 0x3c2818, 0x1a0f0a, 0x4a3020, 0x4a3a28, 0x3c2818)
}

// Wall vents: 0-2 rusted grille boxes per level on random walls
// at head height (1.6-2.1 m).
fn place_wall_vents(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.45 {
        return;
    } // 45% chance NO vents this level
    let count = 1 + (rnd() * 2.0).floor() as usize; // 1-2
    let mut candidates = wall_candidates(g);
    shuffle(&mut candidates);
    let frame_color = vent_color_by_weather(g.weather_key());
    let frame_mat = lambert!({ color: frame_color });
    let opening_mat = basic_mat!({ color: 0x080604 });
    let mut i = 0;
    while i < count && i < candidates.len() {
        let c = candidates[i];
        let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.03);
        let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.03);
        let mid_height = 1.6 + rnd() * 0.5; // 1.6 - 2.1 m
                                            // Vent frame (metallic box, proud of wall).
        let frame = mesh(pg("WallVents__grille", box_geo!(0.30, 0.20, 0.04)), &frame_mat);
        // Recessed opening (darker inset).
        let opening = mesh(plane_geo!(0.24, 0.14), &opening_mat);
        if c.dx.abs() == 1 {
            let rot = if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 };
            frame.rotation().y = rot;
            opening.rotation().y = rot;
            frame.position().set(wall_x, mid_height, wall_z);
            // Position opening just proud of the frame face.
            opening.position().set(wall_x + c.dx as f64 * 0.021, mid_height, wall_z);
        } else {
            let rot = if c.dz > 0 { PI } else { 0.0 };
            frame.rotation().y = rot;
            opening.rotation().y = rot;
            frame.position().set(wall_x, mid_height, wall_z);
            opening.position().set(wall_x, mid_height, wall_z + c.dz as f64 * 0.021);
        }
        g.world.add(&frame);
        g.world.add(&opening);
        i += 1;
    }
}

// Dropped shoes: very rare small worn-shoe mesh on floor.
fn place_dropped_shoes(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.75 {
        return;
    } // 25% chance per level
    let mut open_tiles: Vec<(i32, i32)> = vec![];
    for z in 1..g.h - 1 {
        for x in 1..g.w - 1 {
            if g.tiles[g.t_idx(x, z)] == 0 {
                open_tiles.push((x, z));
            }
        }
    }
    if open_tiles.is_empty() {
        return;
    }
    // Variant: 60% solo, 30% pair (matched), 10% trail.
    let variant_roll = rnd();
    let variant = if variant_roll < 0.60 {
        "solo"
    } else if variant_roll < 0.90 {
        "pair"
    } else {
        "trail"
    };
    let shoe_color = shoe_color_by_weather(g.weather_key());
    let sole_mat = lambert!({ color: 0x1a1210 });
    let upper_mat = lambert!({ color: shoe_color });
    let make_shoe = || {
        let shoe = group();
        let sole = mesh(pg("DroppedShoes__sole", box_geo!(0.28, 0.03, 0.11)), &sole_mat);
        sole.position().y = 0.015;
        shoe.add(&sole);
        let upper = mesh(pg("DroppedShoes__upper", box_geo!(0.24, 0.08, 0.10)), &upper_mat);
        upper.position().y = 0.055;
        upper.position().x = -0.02; // heel-forward asymmetry
        shoe.add(&upper);
        let toe = mesh(pg("DroppedShoes__toe", box_geo!(0.08, 0.05, 0.08)), &upper_mat);
        toe.position().set(0.10, 0.045, 0); // toe cap ends flush with the sole's front (x 0.14)
        shoe.add(&toe);
        shoe
    };
    let pick = open_tiles[(rnd() * open_tiles.len() as f64).floor() as usize];
    let (base_x, base_z) = (g.wx(pick.0), g.wz(pick.1));
    if variant == "solo" {
        let shoe = make_shoe();
        let px = base_x + (rnd() - 0.5) * 2.0;
        let pz = base_z + (rnd() - 0.5) * 2.0;
        shoe.position().set(px, 0, pz);
        shoe.rotation().y = rnd() * PI * 2.0;
        g.world.add(&shoe);
    } else if variant == "pair" {
        // Two shoes side-by-side, both facing same direction.
        let rot = rnd() * PI * 2.0;
        let (cx, cz) = ((rot + PI / 2.0).cos(), (rot + PI / 2.0).sin());
        let cx2 = base_x + (rnd() - 0.5) * 1.6;
        let cz2 = base_z + (rnd() - 0.5) * 1.6;
        for s in 0..2 {
            let off = (if s == 0 { -1.0 } else { 1.0 }) * 0.09; // 18 cm apart
            let shoe = make_shoe();
            shoe.position().set(cx2 + cx * off, 0, cz2 + cz * off);
            shoe.rotation().y = rot;
            g.world.add(&shoe);
        }
    } else {
        // Trail: 2-3 shoes in line pointing same direction — "ran".
        let trail_count = 2 + (rnd() * 2.0).floor() as i32;
        let dir = rnd() * PI * 2.0;
        let (dx, dz) = (dir.cos(), dir.sin());
        let spacing = 0.55 + rnd() * 0.25;
        for s in 0..trail_count {
            let shoe = make_shoe();
            let s = s as f64;
            shoe.position().set(base_x + dx * s * spacing, 0, base_z + dz * s * spacing);
            shoe.rotation().y = dir + (rnd() - 0.5) * 0.3;
            g.world.add(&shoe);
        }
    }
}

// Wall sockets: 0-4 small floor-level outlets on random walls.
fn place_wall_sockets(g: &mut Game, _dist: &[i32]) {
    g.pp.socket_positions.clear(); // reset per level for spark events
    if rnd() < 0.30 {
        return;
    } // 30% chance NO sockets
    let wk = g.weather_key();
    // Electric biome gets more (charge-heavy identity).
    let density_mul = if wk == "electric" { 1.6 } else { 1.0 };
    let count = 6.0f64.min(1.0f64.max(((1.0 + rnd() * 3.0) * density_mul).floor())) as usize;
    let mut candidates = wall_candidates(g);
    shuffle(&mut candidates);
    let socket_mat = lambert!({ color: 0x2a2822 });
    let hole_mat = basic_mat!({ color: 0x000000 });
    let mut i = 0;
    while i < count && i < candidates.len() {
        let c = candidates[i];
        // 0.10 × 0.07 outlet plate.
        let plate = mesh(pg("WallSockets__plate", box_geo!(0.10, 0.07, 0.015)), &socket_mat);
        let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.008);
        let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.008);
        let socket_height = 0.28 + rnd() * 0.15; // 0.28 - 0.43 m
                                                 // Two tiny plug holes.
        let hole1 = mesh(plane_geo!(0.010, 0.020), &hole_mat);
        let hole2 = mesh(plane_geo!(0.010, 0.020), &hole_mat);
        if c.dx.abs() == 1 {
            let rot = if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 };
            plate.rotation().y = rot;
            hole1.rotation().y = rot;
            hole2.rotation().y = rot;
            let pz = wall_z + (rnd() - 0.5) * 1.4;
            plate.position().set(wall_x, socket_height, pz);
            // 4.5 mm behind the plate face: dark bottoms of the modelled slots
            let hx = wall_x - c.dx as f64 * 0.003;
            hole1.position().set(hx, socket_height, plate.pos().z - 0.018);
            hole2.position().set(hx, socket_height, plate.pos().z + 0.018);
        } else {
            let rot = if c.dz > 0 { PI } else { 0.0 };
            plate.rotation().y = rot;
            hole1.rotation().y = rot;
            hole2.rotation().y = rot;
            let px = wall_x + (rnd() - 0.5) * 1.4;
            plate.position().set(px, socket_height, wall_z);
            let hz = wall_z - c.dz as f64 * 0.003;
            hole1.position().set(plate.pos().x - 0.018, socket_height, hz);
            hole2.position().set(plate.pos().x + 0.018, socket_height, hz);
        }
        g.world.add(&plate);
        g.world.add(&hole1);
        g.world.add(&hole2);
        // Track for socket-spark events (electric biome only).
        let p = plate.pos();
        g.pp.socket_positions.push(v3(p.x, socket_height, p.z));
        i += 1;
    }
}

// Wall stains: where water came down a wall from the ceiling's edge
// and dried (`assets/src/blender/props/r07.notes.md`). The wetted patch hangs
// from under the ceiling in lobes, each lobe as far as the water got; the
// edge of every wetting dried as a sharp, wavy, darker tide line (what the
// water carried, left at its rim), the earlier ones inside the last, the
// middle paler; a run or two went on down from the lowest lobes and ended in
// a drop. Brown-yellow on paper and paint, a grey damp patch with a white salt
// crust on concrete and brick, a faint rust on glazed tile; mould speckled in
// soft clusters where it stayed wet. Painted on a transparent plane from the
// ceiling's edge to where the old streak ended (the helpers are inside).
fn place_wall_stains(g: &mut Game, _dist: &[i32]) {
    /// The narrowest a stain is: just under the width (8 cm) past which
    /// `settle_props` probes a wall's face at its sides as well as its middle
    const STAIN_W: f64 = 0.0799;
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
    /// The smooth minimum of two distances, blended over `k`
    fn smin(a: f64, b: f64, k: f64) -> f64 {
        let h = (0.5 + 0.5 * (b - a) / k).clamp(0.0, 1.0);
        b + (a - b) * h - k * h * (1.0 - h)
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
    /// The material of a stain across a plane `pw × ph` m, hanging from the
    /// ceiling's edge (the top of the plane) in tongues, the deepest about
    /// `reach` m down, a run on from it to `foot` m down, `heavy` how much it
    /// left (0.7-1.2), on `surface` (0 paper or paint, 1 masonry, 2 glazed
    /// tile), mouldy where it stayed `damp`
    #[allow(clippy::too_many_arguments)]
    fn stain_mat(pw: f64, ph: f64, reach: f64, foot: f64, heavy: f64, surface: u8, damp: bool) -> Mat {
        // (sRGB) its tide lines, the wash inside them, and how strong each is
        let (line, fill, k_line, k_fill) = match surface {
            1 => ([0.13, 0.12, 0.11], [0.19, 0.18, 0.16], 0.6, 0.5),
            2 => ([0.45, 0.36, 0.25], [0.6, 0.52, 0.41], 0.38, 0.12),
            _ => ([0.3, 0.2, 0.09], [0.5, 0.37, 0.2], 0.9, 0.7),
        };
        let (tw, th) = ((pw / 0.001).round().clamp(64.0, 112.0), (ph / 0.002).round().clamp(320.0, 768.0));
        let tex = canvas_tex_opt(tw, th, false, move |ctx, cw, ch| {
            let seed = (rnd() * 4.0e9) as u32;
            let (sx, sy) = (pw / cw, ph / ch);
            let aa = sx.max(sy) * 1.1;
            let (n, m) = (cw as usize, ch as usize);
            // the lobes it hung in: (middle, half width, depth); the deepest first
            let w = pw * 0.92;
            let lobes: Vec<(f64, f64, f64)> = (0..1 + (rnd() * 2.6) as usize)
                .map(|k| {
                    let c = pw / 2.0 + (rnd() - 0.5) * w * if k == 0 { 0.12 } else { 0.5 };
                    let half = w * if k == 0 { 0.4 + 0.08 * rnd() } else { 0.18 + 0.16 * rnd() };
                    let depth = reach * if k == 0 { 0.75 + 0.25 * rnd() } else { 0.2 + 0.5 * rnd() };
                    (c, half, depth)
                })
                .collect();
            // the band it soaked along under the ceiling, its ends rounded
            let band = 0.04 + 0.06 * rnd();
            // how far down it reached (m) at each column x, with the earlier,
            // shallower wettings inside it: L(x) for every rim
            let rims: Vec<(f64, f64)> = (0..2 + (rnd() * 2.0) as usize)
                .map(|k| if k == 0 { (1.0, 1.0) } else { (0.4 + 0.45 * rnd(), 0.5 + 0.4 * rnd()) })
                .collect();
            let reach_at = |x: f64, k: usize, scale: f64| -> f64 {
                let t = ((x - pw / 2.0) / (w / 2.0)).clamp(-1.0, 1.0);
                let mut l = (band + 0.12 * lobes[0].2) * (1.0 - t * t).max(0.0).powf(0.5);
                // each lobe a tongue, rounded at its foot, joined smoothly to the rest
                for &(c, half, depth) in &lobes {
                    let u = (x - c) / half;
                    if u.abs() < 1.0 {
                        let tongue = depth * (1.0 - u.abs().powf(2.5)).powf(0.6);
                        let kk = 0.03;
                        let hh = (0.5 + 0.5 * (tongue - l) / kk).clamp(0.0, 1.0);
                        l = l + (tongue - l) * hh + kk * hh * (1.0 - hh);
                    }
                }
                let wav = (fbm(seed.wrapping_add(k as u32 * 131), x * 11.0, 0.5, 3) - 0.5) * 0.07 * reach
                    + (vnoise(seed ^ 0x3d1, x * 70.0, k as f64 * 7.0) - 0.5) * 0.008;
                (l * scale + wav * (l / (band + 0.02)).min(1.0)).max(0.0)
            };
            let cols: Vec<Vec<(f64, f64)>> = (0..rims.len())
                .map(|k| {
                    (0..n)
                        .map(|i| {
                            let x = (i as f64 + 0.5) * sx;
                            let (l0, l1) = (reach_at(x - sx, k, rims[k].0), reach_at(x + sx, k, rims[k].0));
                            let l = reach_at(x, k, rims[k].0);
                            // its edge's slope, to measure across it
                            (l, (1.0 + ((l1 - l0) / (2.0 * sx)).powi(2)).sqrt())
                        })
                        .collect()
                })
                .collect();
            // the runs on down from the lowest lobes: (x, from, to, half width, seed)
            let runs: Vec<(f64, f64, f64, f64, u32)> = (0..1 + (rnd() * 1.8) as usize)
                .map(|k| {
                    let (c, half, depth) = lobes[k.min(lobes.len() - 1)];
                    let x = c + (rnd() - 0.5) * half * 0.5;
                    let from = depth * 0.8;
                    let to = if k == 0 { foot - 0.02 * rnd() } else { from + (foot - from) * (0.3 + 0.5 * rnd()) };
                    (x, from, to, 0.004 + 0.004 * rnd(), seed.wrapping_add(900 + k as u32))
                })
                .collect();
            // mould where it stayed wet longest: clusters low in the lobes
            let mould: Vec<(f64, f64, f64)> = if damp || rnd() < 0.2 {
                (0..1 + (rnd() * 2.5) as usize)
                    .map(|_| {
                        let (c, half, depth) = lobes[(rnd() * lobes.len() as f64) as usize % lobes.len()];
                        (c + (rnd() - 0.5) * half, depth * (0.35 + 0.5 * rnd()), 0.015 + 0.025 * rnd())
                    })
                    .collect()
            } else {
                vec![]
            };
            let mut img = ctx.create_image_data(cw, ch);
            let _ = m;
            par_rows(&mut img.data, n * 4, |j, out| {
                let y = (j as f64 + 0.5) * sy;
                for i in 0..n {
                    let x = (i as f64 + 0.5) * sx;
                    let mut px = [line[0], line[1], line[2], 0.0];
                    // the runs
                    let mut d_run = 1.0f64;
                    for &(rx, from, to, hw, rs) in &runs {
                        if y < from - 0.01 || y > to + 0.01 {
                            continue;
                        }
                        let t = ((y - from) / (to - from)).clamp(0.0, 1.0);
                        let xc = rx + (fbm(rs, y * 3.0, 0.5, 3) - 0.5) * 0.03 * t;
                        let hw = hw * (1.5 - 0.7 * t) * (0.45 + 1.0 * fbm(rs ^ 0x9, y * 12.0, 0.5, 2));
                        let along = if y <= to - hw * 2.0 { (x - xc).abs() - hw } else { 1.0 };
                        let drop = (((x - xc) / (hw * 1.4)).hypot((y - (to - hw * 2.0)) / (hw * 2.0)) - 1.0) * hw * 1.4;
                        d_run = d_run.min(smin(along, drop, 0.0015));
                    }
                    let (l, slope) = cols[0][i];
                    if y > l + 0.03 && d_run > 0.01 {
                        continue;
                    }
                    let fine = (vnoise(seed ^ 0x51, x * 260.0, y * 260.0) - 0.5) * 0.0018;
                    // the wetted patch and its rims
                    let d0 = smin((y - l) / slope + fine, d_run + fine, 0.006);
                    // each rim heavier in places, where more was left
                    let along = |k: u32| 0.5 + 0.5 * fbm(seed ^ (0xa1 + k), x * 14.0, y * 3.0, 2);
                    let mut lines = tide(d0, aa, 0.005) * along(0);
                    for (k, &(_, st)) in rims.iter().enumerate().skip(1) {
                        let (lk, sk) = cols[k][i];
                        lines = 1.0 - (1.0 - lines) * (1.0 - tide((y - lk) / sk + fine, aa, 0.004) * st * along(k as u32));
                    }
                    let inside = (0.5 - d0 / aa).clamp(0.0, 1.0);
                    // paler in the middle, darker toward its rims and up where it came in
                    let pale = 0.5 + 0.5 * (d0 / 0.04).exp() + 0.3 * (-y / 0.06).exp();
                    let mott = 0.6 + 0.4 * fbm(seed ^ 0x3c3c, x * 30.0, y * 30.0, 2);
                    lay(&mut px, fill, inside * k_fill * heavy * pale * mott);
                    lay(&mut px, line, lines * k_line * heavy);
                    if surface == 1 {
                        lay(
                            &mut px,
                            [0.78, 0.77, 0.72],
                            (lines * (vnoise(seed ^ 0x99, x * 400.0, y * 400.0) - 0.45).max(0.0) * 1.8).min(0.55),
                        );
                    }
                    // mould: speckles in soft clusters, denser and darker at the middle
                    for &(mx, my, mr) in &mould {
                        let q = (x - mx).hypot((y - my) * 0.8) / mr;
                        if q < 1.6 {
                            let dens = (-q * q * 1.4).exp();
                            // a spot in each 4 mm cell, as likely as the cluster is dense there
                            let (gi, gj) = ((x / 0.004).floor() as i32, (y / 0.004).floor() as i32);
                            let mut dot = 0.0f64;
                            for k in 0..9 {
                                let (ci, cj) = (gi + k % 3 - 1, gj + k / 3 - 1);
                                let h = hash01(seed ^ 0x1234, ci, cj);
                                if h < dens * 0.35 {
                                    let (dx, dy) = (
                                        (ci as f64 + hash01(seed ^ 0x77, ci, cj)) * 0.004 - x,
                                        (cj as f64 + hash01(seed ^ 0x78, ci, cj)) * 0.004 - y,
                                    );
                                    let r = 0.0009 + 0.0012 * hash01(seed ^ 0x79, ci, cj) * dens;
                                    dot = dot.max((1.0 - (dx.hypot(dy) - r) / aa).clamp(0.0, 1.0));
                                }
                            }
                            lay(&mut px, [0.24, 0.24, 0.2], (dot * 0.42 + dens * 0.1).min(0.5) * inside.max(0.3));
                        }
                    }
                    // nothing at the plane's sides
                    px[3] *= ((x.min(pw - x)) / 0.003).clamp(0.0, 1.0);
                    for (c, v) in px.iter().enumerate() {
                        out[i * 4 + c] = (v.clamp(0.0, 1.0) * 255.0).round() as u8;
                    }
                }
            });
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

    let spec = floor_stain_by_weather(g.weather_key());
    let damp = matches!(g.weather_key(), "wet" | "derelict" | "haunted");
    let mut candidates = wall_candidates(g);
    // Density: ~1 per 40 wall-facings × density_mul. Cap 8 to keep sparse.
    let count = 8.0f64.min(0.0f64.max((candidates.len() as f64 * spec.density_mul / 40.0).floor())) as usize;
    shuffle(&mut candidates);
    let mut i = 0;
    while i < count && i < candidates.len() {
        let c = candidates[i];
        // the old streak's draws: its width (4-10 cm, the stain's), its
        // length (0.6-1.6 m below its old top, 15 cm under the ceiling:
        // where its run ends; its patch 8 cm and a sixth of that deep) and its opacity
        // (now how much it left)
        let w = 0.04 + rnd() * 0.06;
        let h = 0.6 + rnd() * 1.0;
        let heavy = ((spec.opacity_base + 0.15) * (0.7 + rnd() * 0.5) / 0.55).clamp(0.7, 1.2);
        let surface = if g.region_flags_at(c.x, c.z) & game::visuals::lighting::RF_MOSS != 0 {
            0
        } else {
            match g.zone_ref(c.x, c.z).map(|zn| zn.wall_mat) {
                Some("concrete" | "brick") => 1,
                Some("tile") => 2,
                _ => 0,
            }
        };
        // its plane's middle where the old one's was, and as wide, or 8 cm
        // where it was narrower (`settle_props` probes the wall's face from
        // there, across the width when it is 8 cm or more: so it finds what
        // it found): from just under the ceiling to as far below the old foot
        let (pw, up) = (w.max(STAIN_W), 0.145);
        let stain = mesh(plane_geo!(pw, h + 2.0 * up), &stain_mat(pw, h + 2.0 * up, 0.08 + h * 0.15, h + up, heavy, surface, damp));
        // on the wall's face, just proud of it
        let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.006);
        let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.006);
        let y = WALL_H - h * 0.5 - 0.15;
        if c.dx.abs() == 1 {
            stain.rotation().y = if c.dx > 0 { -PI / 2.0 } else { PI / 2.0 };
            let z = wall_z + (rnd() - 0.5) * 1.6;
            stain.position().set(wall_x, y, z);
        } else {
            stain.rotation().y = if c.dz > 0 { PI } else { 0.0 };
            let x = wall_x + (rnd() - 0.5) * 1.6;
            stain.position().set(x, y, wall_z);
        }
        g.world.add(&stain);
        i += 1;
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn smoke() {
        let mut list = vec![];
        super::register(&mut list);
        game::debug::testkit::smoke_placers(&list);
    }
}
