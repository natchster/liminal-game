//! `place_hanging_colanders`
use crate::shared::NAIL_R;
use crate::shared::NAIL_RISE;
use crate::shared::WALL_Z;
use game::prelude::*;
use placers_kit::geometry::put;
use placers_kit::noise::fbm;
use placers_kit::noise::mix;
use placers_kit::noise::sstep;
use placers_kit::noise::vn;
use placers_kit::paint::css;
use placers_kit::paint::fill;
use placers_kit::paint::soft;
use placers_kit::place::face_wall;
use placers_kit::place::pick;
use placers_kit::place::salt;
use placers_kit::place::wall_pos;
use placers_kit::place::wall_tiles;

// Hanging colander: 4% chance per level. A two-handled colander (batch
// `t07`: 24 cm, stamped, on a foot ring; graniteware enamel, stainless or
// tin-lined copper) hung by an ear's hole on a nail in the wall, its rim to
// the wall, its bottom out.
pub(crate) fn place_hanging_colanders(g: &mut Game, _dist: &[i32]) {
    g.pp.colander_positions.clear(); // reset per level
    if rnd() < 0.96 {
        return;
    }
    let wt = wall_tiles(g);
    if wt.is_empty() {
        return;
    }
    let (x, z, dx, dz) = pick(&wt);
    // the old metal draw: brushed steel (stainless), copper (tin-lined),
    // dark iron (enamelled: graniteware)
    let metal_colors: [u32; 3] = [0xa0a0a8, 0xc08050, 0x505050];
    let m = pick(&metal_colors);
    let kind = metal_colors.iter().position(|&c| c == m).unwrap_or(0);
    // the old 14 holes' draws (where and how high each was): its knocks
    // and wear now
    let mut wear = [0.0f64; 28];
    for w in wear.iter_mut() {
        *w = rnd();
    }
    let salt = salt(g.wx(x), g.wz(z), (wear[0] * 1e6) as u32 ^ kind as u32);
    let col = group();
    let nail_y = 1.86;
    let steel = phong!({ color: 0x4e4c48, specular: 0x3a3a38, shininess: 30 });
    if let Some(n) = put(&col, "Real__colanderNail", &steel) {
        n.position().set(0, nail_y, WALL_Z);
    }
    // hung by its ear: turned so the ear is up and the rim to the wall;
    // the hole's top on the nail's top, where it crosses it
    let hang = group();
    let hole_z = WALL_Z + 0.001 + COL_RIM_TOP - COL_EAR_HOLE.1;
    let hole_y = nail_y + (hole_z - WALL_Z) * NAIL_RISE + NAIL_R - COL_HOLE_R;
    hang.rotation().set(-PI / 2.0, PI / 2.0, 0);
    hang.position().set(0, hole_y - COL_EAR_HOLE.0, WALL_Z + 0.001 + COL_RIM_TOP);
    let (spec, shin) = [(0x404040, 30.0), (0x4a3a30, 26.0), (0x4a4a4a, 60.0)][kind];
    if let Some(geo) = game::assets::props::prop_part_as_modelled("Real__colanderBowl") {
        let bb = geo.bounding_box();
        let tex = colander_tex(kind, salt, Some((bb.min, bb.max)), &wear);
        hang.add(&mesh(geo, phong!({ map: &tex, specular: spec, shininess: shin, alphaTest: 0.5, side: DoubleSide })));
    }
    put(&hang, "Real__colanderRim", &phong!({ map: &colander_tex(kind, salt, None, &wear), specular: spec, shininess: shin }));
    col.add(&hang);
    col.rotation().y = face_wall(dx, dz);
    let (px, pz) = wall_pos(g, x, z, dx, dz, 0.015);
    col.position().set(px, 0, pz);
    g.world.add(&col);
    let p = col.pos();
    g.pp.colander_positions.push(v3(p.x, 0, p.z));
}

/// The colander's bowl as family `kitchen_hall_and_floor_things` models it (radius, height), from the
/// middle of its bottom to under its rim (`COL_PROF`)
const COL_PROF: [(f64, f64); 10] = [
    (0.000, 0.000),
    (0.030, 0.000),
    (0.052, 0.002),
    (0.068, 0.008),
    (0.082, 0.018),
    (0.094, 0.032),
    (0.103, 0.048),
    (0.110, 0.065),
    (0.1145, 0.082),
    (0.1170, 0.0955),
];

/// An ear's hole: out from the axis, its height (standing)
const COL_EAR_HOLE: (f64, f64) = (0.1385, 0.0915);

const COL_HOLE_R: f64 = 0.0068;

/// The rim's top (standing), which hangs against the wall
const COL_RIM_TOP: f64 = 0.0999;

/// The point of the bowl's profile `s` m from the middle of its bottom
fn col_prof_at(s: f64) -> (f64, f64) {
    let mut left = s.max(0.0);
    for w in COL_PROF.windows(2) {
        let (a, b) = (w[0], w[1]);
        let l = (b.0 - a.0).hypot(b.1 - a.1);
        if left <= l {
            let t = left / l;
            return (a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t);
        }
        left -= l;
    }
    COL_PROF[COL_PROF.len() - 1]
}

/// The texture coordinate the game gives a point of a part modelled in a
/// 'sphere' frame with these bounds (`assets::props::uv_for`)
fn sphere_uv(p: (f64, f64, f64), lo: &Vec3, hi: &Vec3) -> (f64, f64) {
    let f = |v: f64, a: f64, b: f64| ((v - a) / (b - a).max(1e-9)) * 2.0 - 1.0;
    let (cx, cy, cz) = (f(p.0, lo.x, hi.x), f(p.1, lo.y, hi.y), f(p.2, lo.z, hi.z));
    let u = (cz.atan2(-cx) / (2.0 * PI) + 1.0) % 1.0;
    let l = (cx * cx + cy * cy + cz * cz).sqrt().max(1e-9);
    let v = 1.0 - (cy / l).clamp(-1.0, 1.0).acos() / PI;
    (u, v)
}

/// The colander's metal: graniteware (blue-grey enamel speckled white,
/// chipped to the black iron and rust), stainless gone dull (water spots,
/// a film low in the bowl), or copper tarnished brown (verdigris in specks).
/// With `holes` (the bowl's bounds) it is the bowl's map, laid round its
/// axis, its holes cut out in rings as they were punched; else the rim's
fn colander_tex(kind: usize, salt: u32, holes: Option<(Vec3, Vec3)>, wear: &[f64; 28]) -> Tex {
    let chips: Vec<(f64, f64, f64)> = (0..14).map(|i| (wear[i * 2], wear[i * 2 + 1], 0.004 + 0.01 * wear[(i * 5 + 3) % 28])).collect();
    let (w, h) = if holes.is_some() { (1024, 512) } else { (256, 256) };
    canvas_tex_opt(w, h, false, move |ctx, w, h| {
        let seed = salt ^ 0xc01;
        fill(ctx, w, h, |u, v| {
            let n = fbm(seed, u * 24.0, v * 12.0, 4);
            let c = match kind {
                0 => {
                    let streak = vn(seed ^ 5, u * 900.0, v * 6.0);
                    let mut c = mix([0.50, 0.51, 0.52], [0.38, 0.39, 0.40], n + (streak - 0.5) * 0.3);
                    let spot = vn(seed ^ 7, u * 60.0, v * 40.0);
                    if spot > 0.82 {
                        c = mix(c, [0.62, 0.62, 0.60], (spot - 0.82) * 4.0);
                    }
                    mix(c, [0.30, 0.29, 0.26], sstep(0.25, 0.0, v) * 0.5)
                }
                1 => {
                    let mut c = mix([0.42, 0.26, 0.17], [0.27, 0.16, 0.11], n * 1.3 - 0.1);
                    let vg = vn(seed ^ 9, u * 80.0, v * 50.0);
                    if vg > 0.88 {
                        c = mix(c, [0.30, 0.40, 0.34], (vg - 0.88) * 6.0);
                    }
                    c
                }
                _ => {
                    let mut c = mix([0.18, 0.21, 0.25], [0.13, 0.15, 0.18], n);
                    let sp = vn(seed ^ 11, u * 420.0, v * 210.0);
                    if sp > 0.78 {
                        c = mix(c, [0.70, 0.72, 0.73], (sp - 0.78) * 5.0);
                    }
                    c
                }
            };
            // grime and dust, a film where it was handled
            let c = mix(c, [0.28, 0.26, 0.22], fbm(seed ^ 13, u * 6.0, v * 6.0, 3) * 0.35);
            [c[0], c[1], c[2], 1.0]
        });
        // chipped (the enamel to the iron, rust round it), knocked bright
        // (the metals), at the old holes' places
        for &(cu, cv, r) in &chips {
            let at = (cu * w, (1.0 - cv * 0.75) * h);
            let rr = r * w * 1.2;
            if kind == 2 {
                soft(ctx, at, rr * 1.8, 0.7, cu * 6.0, [0.36, 0.22, 0.12], 0.6);
                ctx.set_fill_style(css([0.07, 0.06, 0.06], 1.0));
                ctx.begin_path();
                ctx.ellipse(at.0, at.1, rr, rr * 0.7, cv * 3.0, 0, 7);
                ctx.fill();
            } else {
                soft(ctx, at, rr * 1.4, 0.5, cu * 6.0, if kind == 0 { [0.62, 0.63, 0.63] } else { [0.55, 0.36, 0.24] }, 0.35);
            }
        }
        let Some((lo, hi)) = holes else { return };
        // the holes: rings round the bowl, 4.8 mm, about 10 mm apart, the
        // band under the rim left whole
        ctx.set_global_composite_operation("destination-out");
        ctx.set_fill_style("rgba(0,0,0,1)");
        let hr = 0.0024;
        let mut s = 0.011;
        let mut ring = 0;
        while s < 0.136 {
            let (r, y) = col_prof_at(s);
            let n = ((2.0 * PI * r / 0.0105).floor() as i32).max(1);
            let (r2, y2) = col_prof_at(s + hr);
            for j in 0..n {
                let a = (j as f64 + 0.5 * (ring % 2) as f64) / n as f64 * 2.0 * PI;
                let (u, v) = sphere_uv((r * a.cos(), y, r * a.sin()), &lo, &hi);
                let (_, v2) = sphere_uv((r2 * a.cos(), y2, r2 * a.sin()), &lo, &hi);
                let (rx, ry) = (hr / r / (2.0 * PI) * w, ((v2 - v).abs() * h).max(0.8));
                for du in [-1.0, 0.0, 1.0] {
                    let cx = (u + du) * w;
                    if cx < -rx || cx > w + rx {
                        continue;
                    }
                    ctx.begin_path();
                    ctx.ellipse(cx, (1.0 - v) * h, rx, ry, 0, 0, 7);
                    ctx.fill();
                }
            }
            s += 0.0105;
            ring += 1;
        }
        ctx.set_global_composite_operation("source-over");
    })
}
