//! `place_ceiling_stains`
use game::prelude::*;
use placers_kit::noise::hash;
use placers_kit::place::open_tiles;
use placers_kit::place::shuffle;

// Ceiling water stains: 0-3 a level, where water came through the
// ceiling and dried (`docs/research/props-remodelling/marks_on_the_rooms_surfaces.md`). On a suspended
// ceiling, a ring stain on the tile it dripped onto: brown-yellow tide lines,
// one for each time it was wet and dried, sharp and wavy round a paler middle;
// smaller ones on the tiles round it where it ran over, each tile's own and
// stopped at the grid, whose bars it rusted. On a concrete slab a dark damp
// ring with white salts along its rims; on plaster brown rings. Painted on
// transparent planes just under the ceiling (the helpers are inside, so the
// rest of this file is untouched).
pub(crate) fn place_ceiling_stains(g: &mut Game, _dist: &[i32]) {
    /// Smooth value noise in [0, 1]
    fn vnoise(seed: u32, x: f64, y: f64) -> f64 {
        let (xf, yf) = (x.floor(), y.floor());
        let (i, j) = (xf as i32, yf as i32);
        let (u, v) = (x - xf, y - yf);
        let (u, v) = (u * u * (3.0 - 2.0 * u), v * v * (3.0 - 2.0 * v));
        let (a, b) = (hash(seed, i, j), hash(seed, i + 1, j));
        let (c, d) = (hash(seed, i, j + 1), hash(seed, i + 1, j + 1));
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
            par_rows(&mut img.data, n * 4, |j, out| {
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
            for p in im.data.as_chunks_mut::<4>().0 {
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
        let seed = (hash(0x5eed, (lx * 1000.0) as i32, (lz * 1000.0) as i32) * 4.0e9) as u32;
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
            let (ci, mut cj) = cell_of(lx, lz);
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
                    let h = hash(seed, cell.0, cell.1);
                    // (not into the next 4 m tile: its ceiling may be another height)
                    if (ccx - tx).abs() > CELL / 2.0 || (ccz - tz).abs() > CELL / 2.0 - c / 2.0 {
                        continue;
                    }
                    let spot = if (di, dj) == (0, 0) {
                        Spot { x: leak.0, z: leak.1, r: 0.2 + 0.2 * (radius - 0.5) / 0.7, rims: 3 + (h * 3.0) as usize, k: strength }
                    } else if dn < spread && h < 0.75 && !fitting(cell) {
                        let r = ((spread - dn) * (0.6 + 0.4 * hash(seed ^ 0x77, cell.0, cell.1))).min(0.36);
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
                        stain_mat(c, 192, (ccx, ccz), vec![spot], (ax, turn), 0, (leak.0, leak.1, spread * 0.5), seed),
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
                stain_mat(
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
