//! `place_floor_rugs`
use game::prelude::*;
use placers_kit::noise::fbm;
use placers_kit::noise::hash;
use placers_kit::noise::sstep;
use placers_kit::noise::vn;
use placers_kit::paint::rgb;
use placers_kit::place::open_tiles;
use placers_kit::place::ri;

// Floor rug (Blender batch `r13b`): a Hamadan scatter rug, 2'5" x 4'
// (0.74 x 1.22 m and its fringe), wool pile on cotton, about 6 mm thick:
// its pattern woven and faded, the pile worn down in a path across it, a
// ripple where it has crept, its corners curling and its fringe matted;
// now and then one corner turned right up. Now and then a table lamp on or
// beside it, now and then a newspaper on it.
pub(crate) fn place_floor_rugs(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.85 {
        return;
    } // 15% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = open[ri(open.len())];
    // Per-weather rug tint: a wash over the wool, not a dye
    let tint = rgb(rug_tint_by_weather(g.weather_key()));
    let rug_w = 0.9 + rnd() * 0.4;
    let rug_d = 0.6 + rnd() * 0.3;
    // its size the old one's area, the rug's own proportions (a little
    // smaller or larger, never stretched)
    let k = ((rug_w * rug_d) / (RUG_L * RUG_W)).sqrt().clamp(0.87, 1.13);
    let yaw = rnd() * PI * 2.0;
    let px = g.wx(x) + (rnd() - 0.5) * 1.6;
    let pz = g.wz(z) + (rnd() - 0.5) * 1.6;
    // 25% chance: one corner turned up (the old peel's size, lift, corner)
    let peel = if rnd() < 0.25 {
        let c_size = 0.16 + rnd() * 0.06;
        let lift = 0.5 + rnd() * 0.3;
        let corner = (rnd() * 4.0).floor() as usize;
        Some((c_size, lift, corner.min(3)))
    } else {
        None
    };
    let seed = (((px * 1000.0).round() as i64) as u32).wrapping_mul(0x9e37_79b9)
        ^ (((pz * 1000.0).round() as i64) as u32).wrapping_mul(0x85eb_ca6b);
    let colours = (hash(seed, 3, 7) * 4.0) as usize;
    let wash = tint.map(|t| ((1.0 - (1.0 - t) * 0.5) * 255.0).round() as u32);
    let colour = (wash[0] << 16) | (wash[1] << 8) | wash[2];
    let (face_tex, back_tex) = rug_textures(seed, colours);
    let face_mat = lambert!({ map: &face_tex, color: colour });
    let back_mat = lambert!({ map: &back_tex, color: colour });
    let fringe_mat = lambert!({ color: 0x9c9078, side: DoubleSide });
    let rug = group();
    for (name, mat) in [("Real__rugFace", &face_mat), ("Real__rugBack", &back_mat), ("Real__rugFringe", &fringe_mat)] {
        let Some(geo) = game::assets::props::prop_part_as_modelled(name) else { continue };
        let geo = match peel {
            Some((c_size, lift, corner)) => rug_turned_corner(&geo, c_size / k, lift, corner),
            None => geo,
        };
        rug.add(&mesh(geo, mat));
    }
    if rug.child_count() == 0 {
        let flat = mesh(plane_geo!(RUG_L, RUG_W), &face_mat);
        flat.rotation().x = -PI / 2.0;
        flat.position().y = RUG_T;
        rug.add(&flat);
    }
    rug.scale().set(k, k, k);
    rug.rotation().y = yaw;
    rug.position().set(px, 0, pz);
    g.world.add(&rug);
    // the rug's top under a point (world x, z), or the floor off it
    let (cy, sy) = (yaw.cos(), yaw.sin());
    let on_rug = |wx: f64, wz: f64| {
        let (dx, dz) = (wx - px, wz - pz);
        let (lx, lz) = ((dx * cy - dz * sy) / k, (dx * sy + dz * cy) / k);
        if lx.abs() < RUG_L / 2.0 - 0.02 && lz.abs() < RUG_W / 2.0 - 0.02 {
            RUG_T * k
        } else {
            0.0
        }
    };
    // 25% chance: a table lamp placed on/near rug — reading-
    // corner narrative (inline reproduction).
    if rnd() < 0.25 {
        let base_colors = [0x2a2418, 0x1a1a1e, 0x3a2818];
        let base_color = base_colors[ri(base_colors.len())];
        let shade_colors = [0xa89880, 0x807060, 0x504840];
        let shade_color = shade_colors[ri(shade_colors.len())];
        let base_mat = lambert!({ color: base_color });
        let shade_mat = lambert!({ color: shade_color });
        let lamp = group();
        let base = mesh(pg("TableLamps__base", cyl_geo!(0.10, 0.10, 0.03, 12)), &base_mat);
        base.position().y = 0.015;
        lamp.add(&base);
        let stem = mesh(pg("TableLamps__stem", cyl_geo!(0.010, 0.010, 0.32, 6)), &base_mat);
        stem.position().y = 0.19;
        lamp.add(&stem);
        let shade = mesh(pg("TableLamps__shade", cyl_geo!(0.11, 0.08, 0.14, 8)), &shade_mat);
        shade.position().y = 0.42;
        lamp.add(&shade);
        lamp.rotation().z = (rnd() - 0.5) * 0.08;
        // Place near rug edge (±0.35 m from rug center).
        let l_ang = rnd() * PI * 2.0;
        let l_rad = 0.25 + rnd() * 0.15;
        let (lx, lz) = (px + l_ang.cos() * l_rad, pz + l_ang.sin() * l_rad);
        // standing on the rug where it is on it
        lamp.position().set(lx, on_rug(lx, lz), lz);
        lamp.rotation().y = rnd() * PI * 2.0;
        g.world.add(&lamp);
    }
    // 25% chance: a newspaper laid on top of rug.
    if rnd() < 0.25 {
        let paper_tint = newspaper_tint_by_weather(g.weather_key());
        let rp_mat = lambert!({ map: newspaper_texture(), color: paper_tint, side: DoubleSide });
        let rpaper = mesh(plane_geo!(0.24, 0.16), &rp_mat);
        rpaper.rotation().x = -PI / 2.0;
        rpaper.rotation().z = rnd() * PI * 2.0;
        // Position near rug center with slight offset, lying on its pile
        let (nx, nz) = (px + (rnd() - 0.5) * 0.3, pz + (rnd() - 0.5) * 0.3);
        rpaper.position().set(nx, on_rug(nx, nz) + 0.0025, nz);
        g.world.add(&rpaper);
    }
}

/// The rug of batch `r13b`: its body's length and width (m, its fringe and
/// flat-woven ends beyond), its pile's height
const RUG_L: f64 = 1.22;

const RUG_W: f64 = 0.74;

const RUG_T: f64 = 0.006;

/// Its flat-woven band at each end, and its knots' pitch
const RUG_KILIM: f64 = 0.02;

const RUG_KNOT: f64 = 0.0045;

/// A copy of a rug part with one corner turned up: past a fold `reach` m
/// in from corner `corner` (0..4: -x+z, +x+z, -x-z, +x-z) along its
/// diagonal, the rug rolls up over a curl that turns it `lift` + 0.35 rad
fn rug_turned_corner(geo: &Geo, reach: f64, lift: f64, corner: usize) -> Geo {
    let out = geo.duplicate();
    let (hx, hz) = (RUG_L / 2.0 + RUG_KILIM, RUG_W / 2.0);
    let (cx, cz) = [(-hx, hz), (hx, hz), (-hx, -hz), (hx, -hz)][corner];
    // the diagonal inward from the corner, and the fold across it
    let len = (cx * cx + cz * cz).sqrt();
    let (ux, uz) = (cx / len, cz / len);
    let fold = (cx * ux + cz * uz) - reach * 0.75;
    let span = reach * 0.75;
    let turn = lift + 0.35;
    let rho = span / turn;
    {
        let mut gm = out.g();
        let pos = &mut gm.position;
        for v in pos.chunks_exact_mut(3) {
            let (x, y, z) = (v[0] as f64, v[1] as f64, v[2] as f64);
            let t = x * ux + z * uz - fold;
            if t <= 0.0 {
                continue;
            }
            let a = t / rho;
            let r = rho - y;
            let (t2, y2) = (r * a.sin(), rho - r * a.cos());
            let dt = t2 - t;
            v[0] = (x + ux * dt) as f32;
            v[1] = y2 as f32;
            v[2] = (z + uz * dt) as f32;
        }
    }
    out.compute_vertex_normals();
    out.compute_bounding_box();
    out.compute_bounding_sphere();
    out
}

/// A rug's colours (sRGB 0..1): its field, the field's figures, the main
/// border's ground and its figures, the guard stripes, the outlines and an
/// accent; four colourings, as Hamadan's villages dyed them
#[derive(Clone, Copy)]
struct RugColours {
    field: [f64; 3],
    figure: [f64; 3],
    border: [f64; 3],
    bfigure: [f64; 3],
    guard: [f64; 3],
    line: [f64; 3],
    accent: [f64; 3],
}

fn rug_colours(k: usize) -> RugColours {
    let (red, navy, ivory, camel, brown, ochre, green, rust) = (
        [0.44, 0.16, 0.12],
        [0.15, 0.18, 0.26],
        [0.64, 0.58, 0.47],
        [0.55, 0.43, 0.30],
        [0.26, 0.17, 0.12],
        [0.58, 0.44, 0.22],
        [0.28, 0.33, 0.24],
        [0.52, 0.25, 0.14],
    );
    let dark = [0.14, 0.11, 0.09];
    match k % 4 {
        0 => RugColours { field: red, figure: navy, border: ivory, bfigure: red, guard: ochre, line: dark, accent: green },
        1 => RugColours { field: navy, figure: ivory, border: red, bfigure: navy, guard: ivory, line: dark, accent: ochre },
        2 => RugColours { field: camel, figure: red, border: navy, bfigure: camel, guard: red, line: brown, accent: green },
        _ => RugColours { field: brown, figure: rust, border: navy, bfigure: ivory, guard: ochre, line: dark, accent: red },
    }
}

/// The colour of the knot at (a, b) m on a rug `len` long and `wid` across
/// (a along its length from one end, b across from one side), as a Hamadan
/// village rug is knotted: a narrow reciprocal outer guard; a main border
/// of stepped rosettes between pairs of leaves on its own ground; an inner
/// guard of little chevrons; a field of small stepped stars in staggered
/// rows, with a stepped medallion, its pendants and its corners when
/// `medallion`. Everything stepped on the knots' grid; each knot's dye a
/// little its own, the dye lots (abrash) in bands across it
fn rug_knot(a: f64, b: f64, len: f64, wid: f64, c: &RugColours, medallion: bool, seed: u32) -> [f64; 3] {
    let (ia, ib) = ((a / RUG_KNOT).floor(), (b / RUG_KNOT).floor());
    let (qa, qb) = ((ia + 0.5) * RUG_KNOT, (ib + 0.5) * RUG_KNOT);
    let (ea, eb) = (qa.min(len - qa), qb.min(wid - qb));
    let e = ea.min(eb);
    let t = if ea < eb { qb } else { qa };
    let bw = (wid * 0.12).clamp(0.06, 0.11);
    let (g0, g1) = (0.005, 0.017);
    let l1 = g1 + 0.0045;
    let b1 = l1 + bw;
    let l2 = b1 + 0.0045;
    let g2 = l2 + 0.012;
    let l3 = g2 + 0.0045;
    let mut col = if e < g0 {
        c.line
    } else if e < g1 {
        // the outer guard: a reciprocal of little teeth
        let p = (t / 0.018).fract();
        let q = (e - g0) / (g1 - g0);
        if (p - 0.5).abs() < 0.5 - 0.5 * q {
            c.guard
        } else {
            c.field
        }
    } else if e < l1 || (e >= b1 && e < l2) || (e >= g2 && e < l3) {
        c.line
    } else if e < b1 {
        // the main border: stepped rosettes, a pair of leaves between
        let per = bw * 1.25;
        let ut = (t / per).fract() * per - per / 2.0;
        let uc = (e - l1) - bw / 2.0;
        let (x, y) = (ut.abs(), uc.abs());
        let oct = x.max(y) + 0.45 * x.min(y);
        let r = bw * 0.36;
        if oct < r * 0.28 {
            c.accent
        } else if oct < r * 0.55 {
            c.field
        } else if oct < r * 0.85 {
            c.bfigure
        } else if oct < r {
            c.line
        } else {
            // the leaves: short diagonal strokes either side of the gap
            let lx = x - per * 0.5 + bw * 0.12;
            if (lx + uc * if ut > 0.0 { 1.0 } else { -1.0 }).abs() < 0.006 && lx.abs() < bw * 0.14 && y < bw * 0.3 {
                c.figure
            } else {
                c.border
            }
        }
    } else if e < g2 {
        // the inner guard: little chevrons
        let p = (t / 0.012).fract();
        let q = (e - l2) / (g2 - l2);
        if (p - q).abs() < 0.22 || (p - q + 1.0).abs() < 0.22 {
            c.guard
        } else {
            c.border
        }
    } else {
        // the field: small stepped stars in staggered rows
        let (pa, pb) = (0.072, 0.058);
        let row = (qb / pb).floor();
        let sa = if (row as i64).rem_euclid(2) == 0 { 0.0 } else { pa / 2.0 };
        let fa = ((qa + sa) / pa).fract() * pa - pa / 2.0;
        let fb = (qb / pb).fract() * pb - pb / 2.0;
        let d = fa.abs() + fb.abs();
        let arm = (fa.abs() < 0.003 && fb.abs() < 0.017) || (fb.abs() < 0.003 && fa.abs() < 0.017);
        let mut f = if d < 0.004 {
            c.accent
        } else if d < 0.011 {
            c.figure
        } else if arm {
            c.bfigure
        } else {
            c.field
        };
        if medallion {
            let (da, db) = ((qa - len / 2.0).abs(), (qb - wid / 2.0).abs());
            let (ma, mb) = (len * 0.20, (wid - 2.0 * l3) * 0.34);
            let m = da / ma + db / mb;
            // the pendants on its axis, and the corners' quarter medallions
            let pend = ((da - ma - 0.04).abs() / 0.035 + db / 0.03).max(0.0);
            let (ca, cb) = (qa.min(len - qa) - l3, qb.min(wid - qb) - l3);
            let corner = ca / (len * 0.16) + cb / (wid * 0.20);
            if m < 1.0 {
                f = if m > 0.93 {
                    c.line
                } else if m > 0.80 {
                    c.border
                } else if m < 0.12 {
                    c.accent
                } else if m < 0.22 {
                    c.bfigure
                } else if (m - 0.5).abs() < 0.04 {
                    c.line
                } else if m < 0.5 {
                    c.field
                } else {
                    c.border
                };
            } else if pend < 1.0 {
                f = if pend < 0.4 { c.accent } else { c.border };
            } else if corner < 1.0 {
                f = if corner > 0.9 { c.line } else { c.border };
            }
        }
        f
    };
    // the dye lots, in bands across it, and each knot a little its own
    let lot = 1.0 + 0.14 * (vn(seed ^ 0xab5, qa * 2.2, 0.5) - 0.5);
    let own = 0.93 + 0.14 * hash(seed ^ 0x51ed, ia as i32, ib as i32);
    for v in col.iter_mut() {
        *v *= lot * own;
    }
    col
}

/// How worn the pile is at (x, z) m on the rug (its middle the origin):
/// the path walked across it (the same as its shape's in batch `r13b`)
fn rug_worn(x: f64, z: f64) -> f64 {
    let d = (x * 0.42 + z * 0.91) - 0.05;
    (-(d / 0.16) * (d / 0.16)).exp() * (0.6 + 0.4 * (x * 4.0 + 1.0).sin())
}

/// The rug's face and back, painted at its knots' scale: the face its
/// pattern faded by years of light, the pile worn to the foundation along
/// the path, grime in the worn, a few old stains, the flat-woven ends; the
/// back the same pattern seen through the foundation, flatter and darker,
/// its wefts showing. `seed` from where it lies, so each floor's differs
fn rug_textures(seed: u32, colours: usize) -> (Tex, Tex) {
    let c = rug_colours(colours);
    let (fx0, fz0) = (RUG_L / 2.0 + RUG_KILIM, RUG_W / 2.0);
    // the stains: where, how big
    let stains: Vec<(f64, f64, f64)> = (0..3)
        .map(|k| ((hash(seed, k, 1) - 0.5) * RUG_L * 0.8, (hash(seed, k, 2) - 0.5) * RUG_W * 0.7, 0.03 + 0.06 * hash(seed, k, 3)))
        .collect();
    let moths: Vec<(f64, f64, f64)> = (0..4)
        .map(|k| {
            let side = if hash(seed, k, 5) < 0.5 { -1.0 } else { 1.0 };
            (
                (hash(seed, k, 6) - 0.5) * RUG_L * 0.9,
                side * (RUG_W / 2.0 - 0.03 - 0.08 * hash(seed, k, 7)),
                0.008 + 0.014 * hash(seed, k, 8),
            )
        })
        .collect();
    let face = canvas_tex_opt(512, 320, false, |ctx, w, h| {
        let mut img = ctx.create_image_data(w, h);
        let (n, m) = (w as usize, h as usize);
        par_rows(&mut img.data, n * 4, |j, out| {
            // the box frame's top: u along x, v = 1 - (z's fraction)
            let z = -fz0 + (j as f64 + 0.5) / m as f64 * 2.0 * fz0;
            for i in 0..n {
                let x = -fx0 + (i as f64 + 0.5) / n as f64 * 2.0 * fx0;
                let a = x + RUG_L / 2.0;
                let b = z + RUG_W / 2.0;
                let mut px = if !(0.0..=RUG_L).contains(&a) {
                    // the flat-woven band: ivory with two red stripes
                    let e = if a < 0.0 { -a } else { a - RUG_L };
                    let s = if (0.006..0.009).contains(&e) || (0.013..0.015).contains(&e) { c.field } else { [0.70, 0.65, 0.55] };
                    s.map(|v| v * (0.9 + 0.1 * hash(seed ^ 0x77, i as i32, j as i32)))
                } else {
                    rug_knot(a, b, RUG_L, RUG_W, &c, true, seed)
                };
                // the pile's tufts
                let tuft = 0.90 + 0.12 * hash(seed ^ 0x3c, i as i32, j as i32);
                // faded: the colour gone from it toward a warm grey, unevenly
                let lum = 0.30 * px[0] + 0.59 * px[1] + 0.11 * px[2];
                let fade = 0.40 + 0.16 * fbm(seed ^ 0x99, x * 2.0, z * 2.0, 2);
                let warm = [lum * 1.05 + 0.05, lum + 0.04, lum * 0.86 + 0.03];
                for k in 0..3 {
                    px[k] = (px[k] * (1.0 - fade) + warm[k] * fade) * tuft;
                }
                // worn to the foundation along the path: the colour walked
                // out of it, the knots' rows and the cotton showing, grime
                // ground in
                let wr = rug_worn(x, z) * (0.7 + 0.5 * fbm(seed ^ 0x5, x * 7.0, z * 7.0, 3));
                if wr > 0.02 {
                    let row = ((a / RUG_KNOT).fract() < 0.4) as i32 as f64;
                    let k_ = (wr * 1.25).min(1.0);
                    for q in 0..3 {
                        let bare = [0.52, 0.48, 0.41][q] * (0.80 + 0.20 * row);
                        px[q] = px[q] * (1.0 - 0.8 * k_) + bare * 0.8 * k_;
                        px[q] *= 1.0 - 0.12 * k_;
                    }
                }
                // moth: a few patches eaten to the foundation near its edges
                for &(mx, mz, r) in &moths {
                    let d = ((x - mx).powi(2) + (z - mz).powi(2)).sqrt() / r + 0.5 * (fbm(seed ^ 0x6d, x * 40.0, z * 40.0, 2) - 0.5);
                    if d < 1.0 {
                        let k_ = sstep(1.0, 0.7, d);
                        for (q, v) in px.iter_mut().enumerate() {
                            *v = *v * (1.0 - k_) + [0.52, 0.48, 0.41][q] * k_;
                        }
                    }
                }
                // old stains: dark and brown, soft, their tide rims darker
                for &(sx, sz, r) in &stains {
                    let d =
                        ((x - sx).powi(2) + ((z - sz) * 1.4).powi(2)).sqrt() / r + 0.35 * (fbm(seed ^ 0x2b, x * 12.0, z * 12.0, 3) - 0.5);
                    if d < 1.05 {
                        let k_ = 0.30 * sstep(1.05, 0.7, d) + 0.18 * sstep(0.12, 0.0, (d - 0.95).abs());
                        for (q, v) in px.iter_mut().enumerate() {
                            *v = *v * (1.0 - k_) + [0.20, 0.14, 0.09][q] * k_ * 0.5;
                        }
                    }
                }
                // grime: mottled, heavier toward its edges and ends
                let edge = sstep(0.12, 0.0, (RUG_W / 2.0 - z.abs()).min(RUG_L / 2.0 + RUG_KILIM - x.abs()));
                let grime = 0.80 + 0.20 * fbm(seed ^ 0x61, x * 5.0, z * 5.0, 3) - 0.12 * edge;
                // the dust of years in it, greying it
                let dust = 0.12 + 0.12 * fbm(seed ^ 0x44, x * 3.0, z * 3.0, 3);
                for (q, v) in px.iter_mut().enumerate() {
                    *v = *v * (1.0 - dust) + [0.46, 0.43, 0.38][q] * dust;
                }
                for (q, v) in px.iter().enumerate() {
                    out[i * 4 + q] = ((v * grime).clamp(0.0, 1.0) * 255.0).round() as u8;
                }
                out[i * 4 + 3] = 255;
            }
        });
        ctx.put_image_data(&img, 0, 0);
    });
    let back = canvas_tex_opt(256, 160, false, |ctx, w, h| {
        let mut img = ctx.create_image_data(w, h);
        let (n, m) = (w as usize, h as usize);
        par_rows(&mut img.data, n * 4, |j, out| {
            // the box frame's bottom: u along x, v = z's fraction
            let z = fz0 - (j as f64 + 0.5) / m as f64 * 2.0 * fz0;
            for i in 0..n {
                let x = -fx0 + (i as f64 + 0.5) / n as f64 * 2.0 * fx0;
                let (a, b) = (x + RUG_L / 2.0, z + RUG_W / 2.0);
                let mut px = if (0.0..=RUG_L).contains(&a) { rug_knot(a, b, RUG_L, RUG_W, &c, true, seed) } else { [0.62, 0.58, 0.50] };
                let weft = if (j % 3) == 0 { 0.18 } else { 0.0 };
                for v in px.iter_mut() {
                    *v = (*v * 0.62 + 0.10) * (1.0 - weft) + 0.55 * weft;
                }
                for (q, v) in px.iter().enumerate() {
                    out[i * 4 + q] = (v.clamp(0.0, 1.0) * 255.0).round() as u8;
                }
                out[i * 4 + 3] = 255;
            }
        });
        ctx.put_image_data(&img, 0, 0);
    });
    (face, back)
}

/// Rug tint per weather (else the normal entry)
fn rug_tint_by_weather(key: &str) -> u32 {
    by_weather(key, 0x707880, 0xd8a860, 0xa04040, 0xa89070, 0xd8c8b0, 0xffffff)
}
