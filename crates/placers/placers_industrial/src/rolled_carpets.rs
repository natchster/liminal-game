//! `place_rolled_carpets`
use game::prelude::*;
use placers_kit::noise::fbm;
use placers_kit::noise::hash;
use placers_kit::noise::vn;
use placers_kit::place::pick;
use placers_kit::place::wall_candidates;
use placers_kit::place::WallCand;

// Rolled carpet: 15% chance per level, 1 against a wall (Blender batch
// `r13b`): an 8 x 10 ft rug rolled pile out and tied twice with jute twine,
// lying along the wall: its pattern woven and faded, dust on its top, the
// spiral of its turns at its ends, its outer end's fringe fallen to the
// floor; now and then a 6 x 9 ft roll on it, sagging over it where it
// bends, and rarely a 5 x 8 ft on that.
pub(crate) fn place_rolled_carpets(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.85 {
        return;
    } // 15% chance
    let cands = wall_candidates(g);
    if cands.is_empty() {
        return;
    }
    let c = pick(&cands);
    let carpet_colors: [u32; 4] = [0x4a2818, 0x38281a, 0x503820, 0x2a1a18];
    let carpet_hex = pick(&carpet_colors);
    let colourway = |hex: u32| carpet_colors.iter().position(|&h| h == hex).unwrap_or(0);
    let wall_x = g.wx(c.x) + c.dx as f64 * (CELL * 0.5 - 0.15);
    let wall_z = g.wz(c.z) + c.dz as f64 * (CELL * 0.5 - 0.15);
    // the rolls in the wall's frame: x along it, z out into the room, the
    // wall 0.15 m behind the first roll's axis
    let (gx, gz) = if c.dx.abs() == 1 { (wall_x, g.wz(c.z)) } else { (g.wx(c.x), wall_z) };
    let yaw = wall_yaw(&c);
    let vary = roll_hash(c.x, c.z, c.dx * 3 + c.dz);
    // a world offset (x, z) in the first roll's frame (along the wall, out)
    let (cy, sy) = (yaw.cos(), yaw.sin());
    let local = |wx: f64, wz: f64| (wx * cy - wz * sy, wx * sy + wz * cy);
    let (roll, top) = rolled_rug(0, colourway(carpet_hex), vary, None, (0.0, 0.0));
    roll.rotation().y = yaw;
    roll.position().set(gx, 0, gz);
    g.world.add(&roll);
    // 25% chance: second smaller rolled carpet stacked on top of the first.
    if rnd() < 0.25 {
        // the carpet material's hex is the picked hex (exact sRGB round trip)
        let second_colors: Vec<u32> = carpet_colors.iter().copied().filter(|&cc| cc != carpet_hex).collect();
        let second_hex = pick(&second_colors);
        let tilt = (rnd() - 0.5) * 0.35;
        // where the old one stood: 2 cm toward the wall, along it by a draw
        let along = (rnd() - 0.5) * 0.3;
        let (w2x, w2z) = if c.dx.abs() == 1 { (c.dx as f64 * 0.02, along) } else { (along, c.dz as f64 * 0.02) };
        let l2 = local(w2x, w2z);
        // lying on the first, turned by `tilt`
        let rest = RollRest { tilt, along: l2.0, across: l2.1, on: top };
        let (roll2, top2) = rolled_rug(1, colourway(second_hex), vary ^ 0x2222, Some(rest), l2);
        roll2.rotation().y = yaw;
        roll2.position().set(gx + w2x, 0, gz + w2z);
        g.world.add(&roll2);
        // 5% chance: 3rd smallest roll perched atop the 2-stack.
        if rnd() < 0.05 {
            let third_colors: Vec<u32> = carpet_colors.iter().copied().filter(|&cc| cc != carpet_hex && cc != second_hex).collect();
            let third_pool: Vec<u32> = if !third_colors.is_empty() { third_colors } else { carpet_colors.to_vec() };
            let third_hex = pick(&third_pool);
            let tilt3 = (rnd() - 0.5) * 0.5;
            // where the old one stood off the second: little across, more along
            let (w3x, w3z) =
                if c.dx.abs() == 1 { ((rnd() - 0.5) * 0.05, (rnd() - 0.5) * 0.3) } else { ((rnd() - 0.5) * 0.3, (rnd() - 0.5) * 0.05) };
            let l3 = local(w3x, w3z);
            let rest = RollRest { tilt: tilt3, along: l2.0 + l3.0, across: top2.z + l3.1, on: top2 };
            let (roll3, _) = rolled_rug(2, colourway(third_hex), vary ^ 0x3333, Some(rest), (l2.0 + l3.0, l2.1 + l3.1));
            roll3.rotation().y = yaw;
            roll3.position().set(gx + w2x + w3x, 0, gz + w2z + w3z);
            g.world.add(&roll3);
        }
    }
}

/// `c.dx > 0 ? -PI/2 : PI/2` / `c.dz > 0 ? PI : 0`
fn wall_yaw(c: &WallCand) -> f64 {
    if c.dx.abs() == 1 {
        if c.dx > 0 {
            -PI / 2.0
        } else {
            PI / 2.0
        }
    } else if c.dz > 0 {
        PI
    } else {
        0.0
    }
}

/// The rolled rugs of batch `r13b` (8 x 10, 6 x 9, 5 x 8 ft): the roll's
/// length, its outer radius, the rug's thickness, and the whole rug's
/// length (rolled up across its width)
const ROLLS: [(f64, f64, f64, f64); 3] = [(2.44, 0.120, 0.0125, 3.05), (1.83, 0.090, 0.0105, 2.74), (1.52, 0.070, 0.0085, 2.44)];

/// Where its outer end stops round it (0 the floor, a quarter turn the room
/// side), and how it is squashed by its own weight (down, across)
const ROLL_EDGE: f64 = 52.0 * PI / 180.0;

const ROLL_OVAL: (f64, f64) = (0.95, 1.03);

/// Its knots' pitch
const ROLL_KNOT: f64 = 0.0045;

/// Where a roll lies on the one under it: its turn and its offsets along
/// and across the wall (from the first roll's axis), and the roll under it
#[derive(Clone, Copy)]
struct RollRest {
    tilt: f64,
    along: f64,
    across: f64,
    on: RollTop,
}

/// What a roll offers one laid on it: its axis's height and how far it is
/// out from the wall at its middle, and its radius up to its top
#[derive(Clone, Copy)]
struct RollTop {
    y: f64,
    z: f64,
    r: f64,
}

/// A roll's axis height above the floor it rests on (as `r13b` builds it)
fn roll_axis_y(k: usize) -> f64 {
    let (_, r, t, _) = ROLLS[k];
    (r - t * (1.0 - ROLL_EDGE / (2.0 * PI))) * ROLL_OVAL.0 - 0.0034
}

/// Its axis's bow off straight at `s` m along it (as `r13b` builds it)
fn roll_bow(k: usize, s: f64) -> f64 {
    let u = 2.0 * s / ROLLS[k].0;
    0.012 * (1.0 - u * u) - 0.004 * (u * 2.6).sin()
}

/// A hash of a tile and a salt (no draw)
fn roll_hash(x: i32, z: i32, salt: i32) -> u32 {
    let mut h = (salt as u32).wrapping_mul(0x9e37_79b9) ^ (x as u32).wrapping_mul(0x27d4_eb2d) ^ (z as u32).wrapping_mul(0x1656_67b1);
    h = (h ^ (h >> 15)).wrapping_mul(0x85eb_ca6b);
    h = (h ^ (h >> 13)).wrapping_mul(0xc2b2_ae35);
    h ^ (h >> 16)
}

/// Rolled rug `k` (0..3: 8 x 10, 6 x 9, 5 x 8 ft) in colouring `colours`,
/// as a group in its wall's frame (x along the wall, y up, z into the room,
/// the wall at z = -0.15): laid down along the wall, or, when `rest`, laid
/// on the roll under it, turned, its middle on that one's top and its ends
/// sagging down its sides (and kept off the wall); its group's origin at
/// `origin` (along, out) in that frame. Returns it and what it offers a roll
/// laid on it
fn rolled_rug(k: usize, colours: usize, vary: u32, rest: Option<RollRest>, origin: (f64, f64)) -> (Obj, RollTop) {
    let key = ["A", "B", "C"][k];
    let (len, r, t, _) = ROLLS[k];
    let axis0 = roll_axis_y(k);
    let r_top = (r - t * (PI - ROLL_EDGE) / (2.0 * PI)) * ROLL_OVAL.0;
    let pal = roll_colours(colours);
    let line = pal.line.map(|v| ((v * 1.25 + 0.03).min(1.0) * 255.0).round() as u32);
    let pile = lambert!({ map: &roll_pile_tex(k, colours, vary) });
    let ends = lambert!({ color: line[0] << 16 | line[1] << 8 | line[2] });
    let back = lambert!({ color: 0x8c826e });
    let fringe = lambert!({ color: 0x968a72, side: DoubleSide });
    let twine = lambert!({ color: 0x6a583e });
    let wall_z = -0.15;
    let (tc, ts) = rest.map(|q| (q.tilt.cos(), q.tilt.sin())).unwrap_or((1.0, 0.0));
    // a point of the roll as modelled (standing: its axis +y, its floor
    // side +x at x = 0, the room +z) to the wall's frame, and where its
    // axis lies there
    let place = |sx: f64, sy: f64, sz: f64| -> [f64; 3] {
        let (px, py, pz) = (sy, -sx, sz);
        let Some(q) = rest else { return [px, py, pz] };
        let az = roll_bow(k, px);
        let azg = q.across - px * ts + az * tc;
        let ax = q.along + px * tc + az * ts;
        // off the wall: its side no nearer the wall than it can lie
        let shift = (wall_z + r * ROLL_OVAL.1 + 0.004 - azg).max(0.0);
        // on the roll under it: its axis as high as the two touching allow
        // (the first roll's axis bows; the second's is taken at its middle)
        let under = if k == 1 { roll_bow(0, ax) } else { q.on.z };
        let d = azg + shift - under;
        let reach = q.on.r + axis0 - 0.004;
        let ay = (q.on.y + (reach * reach - d * d).max(0.0).sqrt()).max(axis0);
        [q.along + px * tc + pz * ts, py - axis0 + ay, q.across - px * ts + pz * tc + shift]
    };
    let roll = group();
    for (part, mat) in [("Pile", &pile), ("Ends", &ends), ("Back", &back), ("Fringe", &fringe), ("Twine", &twine)] {
        let name = format!("Real__rugRoll{key}{part}");
        let Some(geo) = game::assets::props::prop_part_as_modelled(&name) else { continue };
        let out = geo.duplicate();
        {
            let mut gm = out.g();
            for v in gm.position.chunks_exact_mut(3) {
                let p = place(v[0] as f64, v[1] as f64, v[2] as f64);
                v[0] = (p[0] - origin.0) as f32;
                v[1] = p[1] as f32;
                v[2] = (p[2] - origin.1) as f32;
            }
            // laid down, the normals turn with it (a sagging one's again below)
            for n in gm.normal.chunks_exact_mut(3) {
                let (nx, ny, nz) = (n[1] as f64, -n[0] as f64, n[2] as f64);
                n[0] = (nx * tc + nz * ts) as f32;
                n[1] = ny as f32;
                n[2] = (-nx * ts + nz * tc) as f32;
            }
        }
        if rest.is_some() {
            out.compute_vertex_normals();
        }
        out.compute_bounding_box();
        out.compute_bounding_sphere();
        roll.add(&mesh(out, mat));
    }
    if roll.child_count() == 0 {
        let m = mesh(cyl_geo!(r, r, len, 12), &pile);
        m.rotation().z = PI / 2.0;
        m.position().y = r;
        roll.add(&m);
    }
    // what it offers one laid on it: its middle's axis
    let mid = place(-axis0, 0.0, roll_bow(k, 0.0));
    (roll, RollTop { y: mid[1], z: mid[2], r: r_top })
}

/// A rug's colours (sRGB 0..1): field, its figures, the main border's
/// ground and figures, the guards, the outlines, an accent (the same four
/// colourings as the floor rug's)
#[derive(Clone, Copy)]
struct RollColours {
    field: [f64; 3],
    figure: [f64; 3],
    border: [f64; 3],
    bfigure: [f64; 3],
    guard: [f64; 3],
    line: [f64; 3],
    accent: [f64; 3],
}

fn roll_colours(k: usize) -> RollColours {
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
        0 => RollColours { field: red, figure: navy, border: ivory, bfigure: red, guard: ochre, line: dark, accent: green },
        1 => RollColours { field: navy, figure: ivory, border: red, bfigure: navy, guard: ivory, line: dark, accent: ochre },
        2 => RollColours { field: camel, figure: red, border: navy, bfigure: camel, guard: red, line: brown, accent: green },
        _ => RollColours { field: brown, figure: rust, border: navy, bfigure: ivory, guard: ochre, line: dark, accent: red },
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
fn roll_knot(a: f64, b: f64, len: f64, wid: f64, c: &RollColours, medallion: bool, seed: u32) -> [f64; 3] {
    let (ia, ib) = ((a / ROLL_KNOT).floor(), (b / ROLL_KNOT).floor());
    let (qa, qb) = ((ia + 0.5) * ROLL_KNOT, (ib + 0.5) * ROLL_KNOT);
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

/// Roll `k`'s outer turn round it (u round it, v along it): the last turn
/// of the rug's length, its end (the flat-woven band, the end's border)
/// just behind its outer edge, the rest its field; faded, grimed, dust on
/// its top. `vary` from where it lies, so each floor's differs
fn roll_pile_tex(k: usize, colours: usize, vary: u32) -> Tex {
    let (len, r, _, rug_len) = ROLLS[k];
    let c = roll_colours(colours);
    let (w, h) = ([256.0, 192.0, 160.0][k], [640.0, 480.0, 400.0][k]);
    let u_edge = (PI / 2.0 - ROLL_EDGE) / (2.0 * PI);
    canvas_tex(w, h, move |ctx, w, h| {
        let seed = (rnd() * 4.0e9) as u32 ^ vary;
        let mut img = ctx.create_image_data(w, h);
        let (n, m) = (w as usize, h as usize);
        par_rows(&mut img.data, n * 4, |j, out| {
            let b = (1.0 - (j as f64 + 0.5) / m as f64) * len;
            for i in 0..n {
                let u = (i as f64 + 0.5) / n as f64;
                let psi = (u_edge - u).rem_euclid(1.0) * 2.0 * PI;
                let a = psi * r;
                let phi = ROLL_EDGE + psi;
                let mut px = if a < 0.02 {
                    let s = if (0.006..0.009).contains(&a) || (0.013..0.015).contains(&a) { c.field } else { [0.70, 0.65, 0.55] };
                    s.map(|v| v * (0.9 + 0.1 * hash(seed ^ 0x77, i as i32, j as i32)))
                } else {
                    roll_knot(a - 0.02, b, rug_len, len, &c, false, seed)
                };
                let tuft = 0.90 + 0.12 * hash(seed ^ 0x3c, i as i32, j as i32);
                let lum = 0.30 * px[0] + 0.59 * px[1] + 0.11 * px[2];
                let fade = 0.40 + 0.16 * fbm(seed ^ 0x99, a * 2.0, b * 2.0, 2);
                let warm = [lum * 1.05 + 0.05, lum + 0.04, lum * 0.86 + 0.03];
                // the dust lying on its top, thickest along the crown
                let top = (-phi.cos()).max(0.0).powi(2) * (0.55 + 0.45 * fbm(seed ^ 0x41, a * 6.0, b * 3.0, 3));
                let grime = 0.80 + 0.20 * fbm(seed ^ 0x61, a * 5.0, b * 5.0, 3);
                for q in 0..3 {
                    let v = (px[q] * (1.0 - fade) + warm[q] * fade) * tuft * grime;
                    px[q] = v * (1.0 - 0.55 * top) + [0.52, 0.49, 0.43][q] * 0.55 * top;
                }
                for (q, v) in px.iter().enumerate() {
                    out[i * 4 + q] = (v.clamp(0.0, 1.0) * 255.0).round() as u8;
                }
                out[i * 4 + 3] = 255;
            }
        });
        ctx.put_image_data(&img, 0, 0);
    })
}
