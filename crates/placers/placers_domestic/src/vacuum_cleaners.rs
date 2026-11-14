//! `place_vacuum_cleaners`
use game::prelude::*;
use placers_kit::geometry::put;
use placers_kit::noise::fbm;
use placers_kit::noise::mix;
use placers_kit::noise::sstep;
use placers_kit::noise::vn;
use placers_kit::paint::fill;
use placers_kit::place::open_tiles;
use placers_kit::place::pick;
use placers_kit::place::salt;

// Vacuum cleaner unattended (floor): 3% chance per level.
// Track positions for optional motor whine audio. An upright of the
// Hoover Convertible kind (family `kitchen_hall_and_floor_things`: 45 x 13 x 12 in), stood with its
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

/// The vacuum's cord where it hangs free of its lower hook (family `kitchen_hall_and_floor_things`:
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
