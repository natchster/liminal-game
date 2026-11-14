//! `place_claw_marks`
use crate::shared::mark_tex;
use crate::shared::vary_by;
use game::prelude::*;
use placers_kit::noise::fbm;
use placers_kit::noise::hash;
use placers_kit::noise::sstep;
use placers_kit::noise::vn;
use placers_kit::paint::mark_mat;
use placers_kit::place::mount_on_wall;
use placers_kit::place::wall_candidates;
use placers_kit::wet_floor::Px;

// Wall claw marks: 4% chance per level. Something dragged its nails down
// the wall (`docs/research/props-remodelling/floor_marks_and_dust.md`): three or four gouges
// side by side a finger apart, dug in hard where they start and lifting
// toward their ends, the nails skipping now and then. On paper or paint over
// plasterboard each tore the surface: the paint and the face paper ripped
// back in a ragged lip (its brown inner plies at the edge), the white gypsum
// beneath gouged out and greyed with the years, a strip of paper rolled up
// where a nail left off, gypsum dust fallen down the wall under them. On
// concrete or brick, pale scored furrows with chipped edges and their grit;
// on glazed tile, fine scratches through the glaze, flakes chipped out along
// them. Painted on a transparent plane the size of the old one.
pub(crate) fn place_claw_marks(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.96 {
        return;
    } // 4% chance
    let candidates = wall_candidates(g);
    if candidates.is_empty() {
        return;
    }
    let c = candidates[(rnd() * candidates.len() as f64).floor() as usize];
    let surface = mark_wall_kind(g, c.x, c.z);
    let mat = claw_marks_mat(0.50, 0.70, surface, vary_by(c.x * 4 + c.dx, c.z * 4 + c.dz, 0x516));
    let decal = mesh(plane_geo!(0.50, 0.70), &mat);
    mount_on_wall(g, &decal, &c, 0.008, 1.4, 0.5);
    g.world.add(&decal);
}

/// What the wall facing tile (x, z) is: 0 paper or paint on plasterboard, 1
/// concrete or brick, 2 glazed tile
fn mark_wall_kind(g: &Game, x: i32, z: i32) -> u8 {
    if g.region_flags_at(x, z) & game::visuals::lighting::RF_MOSS != 0 {
        return 0;
    }
    match g.zone_ref(x, z).map(|zn| zn.wall_mat) {
        Some("concrete" | "brick") => 1,
        Some("tile") => 2,
        _ => 0,
    }
}

/// One gouge of a claw mark: its centreline (m, x across the plane, y down
/// from its top), how wide it is along it (half width, m) and how deep (0..1)
struct Gouge {
    pts: Vec<(f64, f64)>,
    half: Vec<f64>,
    depth: Vec<f64>,
}

/// Claw marks gouged into a wall of `surface` (`mark_wall_kind`), painted on
/// a plane `pw × ph` m
fn claw_marks_mat(pw: f64, ph: f64, surface: u8, vary: u32) -> Mat {
    let (tw, th) = ((pw / 0.001).round(), (ph / 0.001).round());
    // the colours: what lies under the surface, its torn edge, the dust
    let (core, edge, dust) = match surface {
        1 => ([0.8, 0.78, 0.74], [0.84, 0.82, 0.78], [0.82, 0.81, 0.77]),
        2 => ([0.82, 0.8, 0.76], [0.9, 0.89, 0.86], [0.82, 0.81, 0.78]),
        _ => ([0.86, 0.84, 0.79], [0.82, 0.78, 0.68], [0.86, 0.85, 0.81]),
    };
    // the face paper's brown inner plies, where the tear goes through them
    let plies = [0.6, 0.5, 0.37];
    let tex = mark_tex(tw, th, core, vary, move |ctx, cw, ch| {
        let seed = (rnd() * 4.0e9) as u32;
        {
            let s = pw / cw;
            let aa = s * 1.1;
            // the hand: three or four nails a finger apart, dragged down at a lean
            let n = if rnd() < 0.7 { 4 } else { 3 };
            let spacing = 0.02 + 0.008 * rnd();
            let lean = (0.1 + 0.35 * rnd()) * if rnd() < 0.5 { -1.0 } else { 1.0 };
            let len = ph * (0.52 + 0.18 * rnd());
            let bend = (rnd() - 0.5) * 0.25;
            let fan = 0.1 + 0.25 * rnd();
            let (dx, dy) = (lean.sin(), lean.cos());
            let (nx, ny) = (dy, -dx);
            let top = (pw / 2.0 - dx * len / 2.0, (ph - dy * len) / 2.0 - 0.02);
            // each finger's reach, where it starts and how hard it pressed
            let reach = [0.82, 1.0, 0.96, 0.72];
            let stagger = [0.016, 0.0, 0.006, 0.03];
            let press = [0.75, 1.0, 0.92, 0.6];
            let (wide, deep) = match surface {
                1 => (0.0048, 0.6),
                2 => (0.002, 0.5),
                _ => (0.0085, 1.0),
            };
            let gouges: Vec<Gouge> = (0..n)
                .map(|k| {
                    let f = if n == 3 { k + 1 } else { k };
                    let off = (k as f64 - (n as f64 - 1.0) / 2.0) * spacing;
                    let gs = seed.wrapping_add(31 * k as u32);
                    let (l0, l1) = (stagger[f] + 0.01 * rnd(), len * reach[f] * (0.9 + 0.15 * rnd()));
                    let hw = wide * (0.8 + 0.4 * rnd()) * (0.6 + 0.4 * press[f]);
                    let m = 72;
                    let mut g = Gouge { pts: vec![], half: vec![], depth: vec![] };
                    for i in 0..=m {
                        let t = i as f64 / m as f64;
                        let a = l0 + (l1 - l0) * t;
                        let o = off * (1.0 + fan * t) + bend * len * (t * t - t) + (fbm(gs, t * 6.0, 0.5, 2) - 0.5) * 0.004;
                        g.pts.push((top.0 + dx * a + nx * o, top.1 + dy * a + ny * o));
                        // dug in at once, lifting slowly; chattering, skipping
                        let mut e = sstep(0.0, 0.04, t) * (1.0 - t).powf(0.6) * (0.8 + 0.2 * vn(gs ^ 0x5, t * 40.0, 0.5));
                        if vn(gs ^ 0x6, t * 9.0, 0.5) > 0.82 {
                            e *= 0.3;
                        }
                        let e = e * press[f];
                        g.half.push(hw * e.sqrt().max(0.0));
                        g.depth.push(e * deep);
                    }
                    g
                })
                .collect();
            // strips of face paper rolled up where some nails left off (paper only)
            let curls: Vec<(f64, f64, f64, f64)> = if surface == 0 {
                gouges
                    .iter()
                    .filter(|_| rnd() < 0.6)
                    .map(|g| {
                        let k = g.pts.len() * (5 + (rnd() * 4.0) as usize) / 10;
                        let (p, q) = (g.pts[k], g.pts[(k + 4).min(g.pts.len() - 1)]);
                        (p.0, p.1, (q.1 - p.1).atan2(q.0 - p.0), 0.007 + 0.01 * rnd())
                    })
                    .collect()
            } else {
                vec![]
            };
            // each gouge's bounds, with the reach of its dust below it
            let bounds: Vec<[f64; 4]> = gouges
                .iter()
                .map(|g| {
                    let (mut b0, mut b1, mut b2, mut b3) = (f64::INFINITY, f64::INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY);
                    for &(px, py) in &g.pts {
                        b0 = b0.min(px);
                        b1 = b1.min(py);
                        b2 = b2.max(px);
                        b3 = b3.max(py);
                    }
                    [b0 - 0.045, b1 - 0.045, b2 + 0.045, b3 + 0.065]
                })
                .collect();
            // the light falls from above and a little to one side
            let (lx, ly) = (0.35, -0.94);
            let nn = cw as usize;
            let mut img = ctx.create_image_data(cw, ch);
            par_rows(&mut img.data, nn * 4, |j, out| {
                let y = (j as f64 + 0.5) * s;
                for i in 0..nn {
                    let x = (i as f64 + 0.5) * s;
                    let mut px = Px::CLEAR;
                    // the nearest gouge: how far across it (and to which side),
                    // how wide and deep it is there, its heading, how far along
                    let mut best = (f64::INFINITY, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
                    let mut dust_v = 0.0f64;
                    for (g, bb) in gouges.iter().zip(&bounds) {
                        if x < bb[0] || y < bb[1] || x > bb[2] || y > bb[3] {
                            continue;
                        }
                        for k in 0..g.pts.len() - 1 {
                            let (a, b) = (g.pts[k], g.pts[k + 1]);
                            if (x - a.0).abs() > 0.04 || (y - a.1).abs() > 0.06 {
                                continue;
                            }
                            let (ex, ey) = (b.0 - a.0, b.1 - a.1);
                            let l2 = ex * ex + ey * ey;
                            let t = (((x - a.0) * ex + (y - a.1) * ey) / l2).clamp(0.0, 1.0);
                            let (qx, qy) = (a.0 + ex * t, a.1 + ey * t);
                            let d = (x - qx).hypot(y - qy);
                            let hw = g.half[k] + (g.half[k + 1] - g.half[k]) * t;
                            let dp = g.depth[k] + (g.depth[k + 1] - g.depth[k]) * t;
                            let side = if (x - qx) * ey - (y - qy) * ex > 0.0 { 1.0 } else { -1.0 };
                            if d - hw < best.0 - best.1 {
                                let l = l2.sqrt();
                                best = (d, hw, dp, side, ex / l, ey / l, (k as f64 + t) * l);
                            }
                            // the dust that fell: under the gouge, straight down the wall
                            if y > qy && (x - qx).abs() < hw * 2.0 + 0.002 {
                                let fall = y - qy;
                                dust_v = dust_v.max(dp * (-fall / 0.035).exp() * sstep(hw * 2.0 + 0.002, 0.0, (x - qx).abs()));
                            }
                        }
                    }
                    let (d, hw, dp, side, tx, ty, along) = best;
                    // gypsum dust fallen below, streaked by the wall's grain
                    if dust_v > 0.0 {
                        let streak = 0.4 + 0.6 * vn(seed ^ 0x2d, x * 900.0, y * 50.0);
                        px.lay(dust, dust_v * streak * 0.4);
                    }
                    if hw > 2e-4 {
                        // a ragged edge: the surface torn, not cut
                        let rag = (fbm(seed ^ 0x11, x * 600.0, y * 600.0, 2) - 0.5) * (hw * 0.7 + 0.001);
                        let e = d - hw - rag;
                        // the light across the gouge: + from its +n side (n = (ty, -tx))
                        let nl = ty * lx - tx * ly;
                        // the torn surface round it: paint and paper lifted and
                        // split back from the tear, pale where its core shows
                        let fringe = hw * 0.55 + 0.0015 * (0.5 + vn(seed ^ 0x19, along * 120.0, side));
                        if e > 0.0 && e < fringe && surface != 2 {
                            let k = sstep(fringe, fringe * 0.5, e) * (0.55 + 0.45 * vn(seed ^ 0x1a, x * 1500.0, y * 1500.0));
                            px.lay(edge, k * 0.85 * dp.sqrt().min(1.0));
                        }
                        if surface == 2 && e > 0.0 && e < 0.0015 {
                            // flakes of glaze chipped out along it
                            let chip = (hash(seed ^ 0x33, (x / 0.0012) as i32, (y / 0.0012) as i32) > 0.55) as u8 as f64;
                            px.lay(edge, chip * 0.7 * dp);
                        }
                        // the lifted lip's shadow, on the side away from the light
                        if e > fringe * 0.4 && side * nl < 0.0 {
                            px.lay([0.07, 0.06, 0.05], 0.5 * dp * (-nl * side).min(1.0) * (-(e - fringe * 0.4) / 0.0012).exp());
                        }
                        if e <= aa {
                            // inside: the core, scored along by the nail; the wall
                            // facing the light lit, the other in the lip's shadow,
                            // darker down its middle where it is deepest
                            let u = (d / (hw + rag).max(1e-4)).min(1.0);
                            let lit = -side * nl * u;
                            // the lip's shadow reaches only a little way in
                            let shade = if lit < 0.0 { 1.0 + 0.55 * lit * sstep(0.45, 0.9, u) } else { 1.0 + 0.15 * lit };
                            let deepest = 1.0 - 0.18 * dp * (1.0 - u * u);
                            let scored = 0.9 + 0.18 * vn(seed ^ 0x44, (d * side) * 2500.0, along * 30.0);
                            let grime = 0.9 + 0.1 * fbm(seed ^ 0x45, x * 300.0, y * 300.0, 2);
                            let k = (shade * deepest * scored * grime).clamp(0.35, 1.15);
                            // the brown plies of the face paper at the tear's edge
                            let base = if surface == 0 && u > 0.78 { plies } else { core };
                            let c = [base[0] * k, base[1] * k, base[2] * k];
                            px.lay(c, sstep(aa, -aa, e) * (0.7 + 0.3 * dp.sqrt()));
                        }
                    }
                    // a strip of paper rolled up from the tear
                    for &(cx, cy, a, l) in &curls {
                        let (ux, uy) = ((x - cx) * a.cos() + (y - cy) * a.sin(), -(x - cx) * a.sin() + (y - cy) * a.cos());
                        let q = (ux / l).hypot(uy / (l * 0.3));
                        if q < 1.6 {
                            // its shadow on the wall beside it, then the curl, lit on top
                            let sq = ((ux - 0.0006) / l).hypot((uy - 0.0011) / (l * 0.34));
                            px.lay([0.06, 0.05, 0.04], 0.18 * sstep(1.15, 0.85, sq));
                            let shade = 0.82 + 0.25 * (-uy / (l * 0.3)).clamp(-1.0, 1.0);
                            px.lay([edge[0] * shade, edge[1] * shade, edge[2] * shade], sstep(1.0, 0.85, q));
                        }
                    }
                    // nothing at the plane's edges
                    px.a *= sstep(0.0, 0.01, x.min(pw - x).min(y).min(ph - y));
                    let c = px.rgba();
                    for (k, v) in c.iter().enumerate() {
                        out[i * 4 + k] = (v.clamp(0.0, 1.0) * 255.0).round() as u8;
                    }
                }
            });
            ctx.put_image_data(&img, 0, 0);
        }
    });
    mark_mat(&tex)
}
