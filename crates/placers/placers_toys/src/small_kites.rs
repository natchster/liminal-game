//! `place_small_kites`: Small kite: 8% chance per level. A child's paper diamond kite (Blender family `a_play_areas_toys`): printed paper (600 x 500 mm) folded over a perimeter string on a 5 mm pine spine and cross spar, a tail of three tissue bows tied on a string, a two legged bridle and a length of its line; dropped face up on its spars, the paper sagging between them, crumpled, torn open at its lower left, the bowstring snapped. The print faded and water-stained, a shoe's tread on it, dust.
use game::assets::props::prop_part_as_modelled;
use game::prelude::*;
use placers_kit::noise::hash;
use placers_kit::noise::mix;
use placers_kit::noise::sstep;
use placers_kit::noise::vn;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;
use placers_kit::place::salt_of;

/// Small kite: 8% chance per level. A child's paper diamond kite
/// (Blender family `a_play_areas_toys`): printed paper (600 x 500 mm) folded over a
/// perimeter string on a 5 mm pine spine and cross spar, a tail of three
/// tissue bows tied on a string, a two legged bridle and a length of its
/// line; dropped face up on its spars, the paper sagging between them,
/// crumpled, torn open at its lower left, the bowstring snapped. The print
/// faded and water-stained, a shoe's tread on it, dust.
pub(crate) fn place_small_kites(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    } // 8% chance
    let open = open_tiles(g);
    if open.is_empty() {
        return;
    }
    let (x, z) = pick(&open);
    // the old colours: the paper's, and the tissue of the bows (its print)
    let kite_colors = [0xd82020, 0x2050d8, 0xd8b820, 0x30a030, 0xd050a0];
    let faded = [[0.60, 0.30, 0.25], [0.30, 0.38, 0.55], [0.74, 0.64, 0.36], [0.36, 0.49, 0.32], [0.66, 0.44, 0.50]];
    let ck = pick(&kite_colors);
    let cb = pick(&kite_colors);
    let (ik, ib) = (kite_colors.iter().position(|&c| c == ck).unwrap_or(0), kite_colors.iter().position(|&c| c == cb).unwrap_or(0));
    let kite = group();
    // the three bows, each tied a little askew on the tail (the old bows' turns)
    let tissue = faded[ib];
    let bow_mat = lambert!({ color: hex([tissue[0] * 1.06, tissue[1] * 1.06, tissue[2] * 1.06]), side: DoubleSide });
    for &(bx, bz, way) in &BOWS {
        let askew = (rnd() - 0.5) * 0.9;
        if let Some(geo) = prop_part_as_modelled("Real__kiteBow") {
            let bow = mesh(geo, &bow_mat);
            bow.position().set(bx, 0, bz);
            bow.rotation().y = way + askew;
            kite.add(&bow);
        }
    }
    let px = g.wx(x) + (rnd() - 0.5) * 2.0;
    let pz = g.wz(z) + (rnd() - 0.5) * 2.0;
    let salt = salt_of(px, pz);
    let ink = if ib == ik { [0.16, 0.15, 0.16] } else { [faded[ib][0] * 0.8, faded[ib][1] * 0.8, faded[ib][2] * 0.8] };
    put(&kite, "Real__kiteSail", &lambert!({ map: &sail_tex(faded[ik], ink, salt), side: DoubleSide }));
    put(&kite, "Real__kiteSpars", &lambert!({ color: 0x9e8660 }));
    put(&kite, "Real__kiteString", &lambert!({ color: 0x98928a }));
    kite.position().set(px, 0, pz);
    kite.rotation().y = rnd() * PI * 2.0;
    g.world.add(&kite);
}

/// Where the tail's three bows are tied, and the way the tail runs there
/// (rad about y from +z), as the tail was laid (`toys/a_play_areas_toys.py` prints them)
const BOWS: [(f64, f64, f64); 3] = [(-0.0325, -0.5078, 2.8957), (-0.0073, -0.6946, 2.7716), (0.0705, -0.8578, 2.6857)];

/// The sail's plan (`toys/a_play_areas_toys.py` K_*): its top and bottom tips on the spine, the
/// cross spar's height up it and its half length
const K_TOP: f64 = 0.300;

const K_BOT: f64 = -0.300;

const K_CROSS: f64 = 0.150;

const K_HW: f64 = 0.250;

/// The sail's paper laid on from above (its bounds: x across, z from its
/// bottom tip to its top): the ground colour, a printed star and border in
/// the ink, the hem folded over the string, faded, creased, a tide line
/// where it was wet, a shoe's tread across it, dust
fn sail_tex(paper: [f64; 3], ink: [f64; 3], salt: u32) -> Tex {
    paint(512, 512, move |u, v| {
        let (x, z) = (-K_HW + 2.0 * K_HW * u, K_BOT + (K_TOP - K_BOT) * v);
        // how far in from the edge (m)
        let lim = if z > K_CROSS { K_HW * (K_TOP - z) / (K_TOP - K_CROSS) } else { K_HW * (z - K_BOT) / (K_CROSS - K_BOT) };
        let k = if z > K_CROSS { 0.514 } else { 0.874 };
        let edge = (lim - x.abs()) * k;
        let mut c = paper;
        // the print: a border line 24 mm in, a five pointed star over the middle
        let band = sstep(0.020, 0.022, edge) * (1.0 - sstep(0.030, 0.032, edge));
        let (sx, sz) = (x, z - 0.04);
        let r = sx.hypot(sz);
        let star = 1.0 - sstep(-0.002, 0.002, r - star_edge(sz.atan2(sx), 0.115, 0.045));
        let ring = sstep(0.150, 0.152, r) * (1.0 - sstep(0.158, 0.160, r));
        let printed = band.max(star).max(ring);
        // the print rubbed and faded unevenly
        let fade = fbm(salt ^ 0x21, u * 5.0, v * 5.0, 3);
        c = mix(c, ink, printed * (0.85 - 0.35 * sstep(0.5, 0.8, fade)));
        c = mix(c, [0.80, 0.77, 0.70], 0.30 * sstep(0.45, 0.85, fade));
        // the hem: the paper doubled over the string, darker, grimy
        c = mix(c, [c[0] * 0.78, c[1] * 0.76, c[2] * 0.74], sstep(0.014, 0.010, edge));
        // creases where it was folded and crumpled: straight folds across
        // it, each dark on one side and catching the light on the other
        let (mut dark, mut lit) = (0.0f64, 0.0f64);
        for k in 0..10 {
            let a = hash(salt ^ 0x91, k, 0) * PI;
            let (x0, z0) = (-0.2 + 0.4 * hash(salt ^ 0x92, k, 1), -0.25 + 0.5 * hash(salt ^ 0x93, k, 2));
            let (nx, nz) = (a.cos(), a.sin());
            let d = (x - x0) * nx + (z - z0) * nz;
            let along = (z - z0) * nx - (x - x0) * nz;
            let len = 0.06 + 0.18 * hash(salt ^ 0x94, k, 3);
            let fade = sstep(len, len * 0.5, along.abs());
            if d < 0.0 {
                dark = dark.max(sstep(0.0016, 0.0, -d) * fade);
            } else {
                lit = lit.max(sstep(0.0020, 0.0, d) * fade);
            }
        }
        c = mix(c, [c[0] * 0.66, c[1] * 0.64, c[2] * 0.62], 0.75 * dark);
        c = mix(c, [0.86, 0.84, 0.80], 0.22 * lit);
        // a tide line where water stood on it, and its stain inside
        let wet = fbm(salt ^ 0x63, u * 3.0, v * 3.0, 3) + 0.25 * (1.0 - u);
        let inside = sstep(0.58, 0.60, wet);
        let tide = sstep(0.575, 0.59, wet) * (1.0 - sstep(0.59, 0.61, wet));
        c = mix(c, [c[0] * 0.86, c[1] * 0.80, c[2] * 0.72], 0.5 * inside);
        c = mix(c, [0.36, 0.28, 0.20], 0.55 * tide);
        // a shoe's tread across it: a sole's outline of grime, its bars
        let (tx, tz) = (x - 0.06, z + 0.13);
        let (ca, sa) = (0.6f64.cos(), 0.6f64.sin());
        let (lx, lz) = (tx * ca - tz * sa, tx * sa + tz * ca);
        let sole = (lx / 0.045).powi(2) + (lz / (0.11 + 0.012 * (lz / 0.11))).powi(2);
        let bars = sstep(0.3, 0.6, (lz * 220.0).sin()) * (1.0 - sstep(0.6, 1.0, sole));
        let scuff = sstep(0.35, 0.75, fbm(salt ^ 0x71, u * 30.0, v * 30.0, 2));
        c = mix(c, [0.30, 0.27, 0.24], bars * scuff * 0.55);
        // dust
        let dust = sstep(0.45, 0.9, fbm(salt ^ 0x11, u * 9.0, v * 9.0, 3));
        mix(c, [0.62, 0.60, 0.56], 0.35 * dust)
    })
}

/// A regular five pointed star's edge (outer radius `ro`, inner `ri`, a
/// point toward +z: the kite's head) at the angle `a` from its middle
fn star_edge(a: f64, ro: f64, ri: f64) -> f64 {
    let sector = PI / 5.0;
    // from the nearest point's direction, folded into one half sector
    let f = (a - PI / 2.0).rem_euclid(2.0 * sector);
    let phi = if f > sector { 2.0 * sector - f } else { f };
    // the edge from the point (ro, 0) to the notch (ri at the half sector)
    let (ax, ay) = (ro, 0.0);
    let (bx, by) = (ri * sector.cos(), ri * sector.sin());
    let (ex, ey) = (bx - ax, by - ay);
    let (dx, dy) = (phi.cos(), phi.sin());
    (ax * ey - ay * ex) / (dx * ey - dy * ex)
}

/// A colour as hex from sRGB in 0..1
fn hex(c: [f64; 3]) -> u32 {
    let b = |x: f64| (x.clamp(0.0, 1.0) * 255.0).round() as u32;
    (b(c[0]) << 16) | (b(c[1]) << 8) | b(c[2])
}

/// The part `name` of family `a_play_areas_toys` as modelled, added to `grp` in `mat`
fn put(grp: &Obj, name: &str, mat: &Mat) {
    if let Some(geo) = prop_part_as_modelled(name) {
        grp.add(&mesh(geo, mat.clone()));
    }
}

/// A texture painted pixel by pixel (sRGB) from `f(u, v)` (v down the
/// image), on every core: no canvas call and no draw inside
fn paint(w: usize, h: usize, f: impl Fn(f64, f64) -> [f64; 3] + Sync + Send + 'static) -> Tex {
    canvas_tex_opt(w as f64, h as f64, false, move |ctx, cw, ch| {
        let mut img = ctx.create_image_data(cw, ch);
        par_rows(&mut img.data, w * 4, |j, out| {
            for i in 0..w {
                let c = f((i as f64 + 0.5) / w as f64, (j as f64 + 0.5) / h as f64);
                for k in 0..3 {
                    out[i * 4 + k] = (c[k].clamp(0.0, 1.0) * 255.0).round() as u8;
                }
                out[i * 4 + 3] = 255;
            }
        });
        ctx.put_image_data(&img, 0, 0);
    })
}

fn fbm(seed: u32, x: f64, y: f64, oct: u32) -> f64 {
    let (mut s, mut amp, mut f, mut n) = (0.0, 0.5, 1.0, 0.0);
    for k in 0..oct {
        s += amp * vn(seed.wrapping_add(k * 7919), x * f, y * f);
        n += amp;
        amp *= 0.5;
        f *= 2.03;
    }
    s / n
}
