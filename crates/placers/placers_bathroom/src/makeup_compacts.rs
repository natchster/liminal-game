//! `place_makeup_compacts`
use crate::shared::salt;
use game::prelude::*;
use placers_kit::geometry::put;
use placers_kit::noise::fbm;
use placers_kit::noise::hash;
use placers_kit::noise::line;
use placers_kit::noise::sstep;
use placers_kit::noise::vn;
use placers_kit::paint::fill;
use placers_kit::place::jitter;
use placers_kit::place::pick_open;

// Makeup compact: 8% chance per level. A pressed powder compact for a
// 59 mm pan (Blender family `washroom_shelf_and_dressing_table`): a moulded case, its lid on a three-knuckle
// hinge, the catch at the front; shut, or (the old 40%) open, its mirror gone
// black at the edges and spotted, the powder in its aluminium pan worn into a
// hollow by the puff and cracked where it was dropped. The old colours: a
// rose, black, gold-tone brass, ivory or teal case, scratched and scuffed.
pub(crate) fn place_makeup_compacts(g: &mut Game, _dist: &[i32]) {
    if rnd() < 0.92 {
        return;
    }
    let Some((x, z)) = pick_open(g) else { return };
    let case_colors = [0xd050a0, 0x202020, 0xe0c040, 0xffffff, 0x40a0a0];
    let col = choice(&case_colors);
    let kind = case_colors.iter().position(|&k| k == col).unwrap_or(0);
    let compact = group();
    let is_open = rnd() < 0.4; // 40% open
    let (px, pz) = jitter(g, x, z, 2.4);
    let salt = salt(px, pz);
    let (c, spec, shine) =
        [(0x9a5c6e, 0x3a3a3a, 60), (0x1e1d1f, 0x4a4a4a, 80), (0x6a5228, 0xa08850, 45), (0xd4ccbc, 0x3a3a3a, 50), (0x2f6664, 0x3a3a3a, 60)]
            [kind];
    let case = phong!({ map: &case_tex(kind, salt), color: c, specular: spec, shininess: shine });
    put(&compact, "Real__makeupCompactBase", &case);
    // the lid turns open about the hinge's pin (y, z of `HINGE`, bathroom/washroom_shelf_and_dressing_table.py)
    let pivot = group();
    pivot.position().set(0, HINGE.0, HINGE.1);
    let lid = group();
    lid.position().set(0, -HINGE.0, -HINGE.1);
    pivot.add(&lid);
    compact.add(&pivot);
    put(&lid, "Real__makeupCompactLid", &case);
    if kind != 2 {
        // the plastic ones' gilt trim round the lid, rubbed to the base metal
        put(
            &lid,
            "Real__makeupCompactTrim",
            &phong!({ map: &case_tex(2, salt ^ 0x55), color: 0x6a5228, specular: 0xa08850, shininess: 45 }),
        );
    }
    if is_open {
        // as far as the hinge lets it
        pivot.rotation().x = -1.83;
        if let Some(m) = put(&lid, "Real__makeupCompactMirror", &mirror_mat(&mirror_tex(salt))) {
            m.set_render_order(2);
        }
        put(&compact, "Real__makeupCompactPan", &phong!({ color: 0x5a5c5e, specular: 0x8a8a8a, shininess: 50 }));
        put(&compact, "Real__makeupCompactPowder", &lambert!({ map: &powder_tex(salt) }));
    }
    compact.position().set(px, 0, pz);
    compact.rotation().y = rnd() * PI * 2.0;
    g.world.add(&compact);
}

/// The compact's hinge pin: its height and how far back (bathroom/washroom_shelf_and_dressing_table.py prints it)
const HINGE: (f64, f64) = (0.009, -0.0398);

/// Scratches as straight short strokes from a seeded scatter: the
/// darkness (or lightness) of the nearest at (u, v)
fn scratches(seed: u32, u: f64, v: f64, n: i32, len: f64) -> f64 {
    let mut s: f64 = 0.0;
    for k in 0..n {
        let (cx, cy) = (hash(seed, k, 1), hash(seed, k, 2));
        let a = hash(seed, k, 3) * 3.2;
        let (dx, dy) = (u - cx, v - cy);
        let along = dx * a.dcos() + dy * a.dsin();
        let across = -dx * a.dsin() + dy * a.dcos();
        if along.abs() < len * (0.4 + hash(seed, k, 4)) {
            s = s.max(line(across, 0.004) * (0.4 + 0.6 * hash(seed, k, 5)));
        }
    }
    s
}

/// The case's wear (a grey to multiply its colour by): scuffs, hairline
/// scratches, grime in its seams; the gold-tone one's lid turned in
/// rings and its plating rubbed through to the brass at the edges
fn case_tex(kind: usize, salt: u32) -> Tex {
    canvas_tex_opt(128, 128, false, move |ctx, w, h| {
        let seed = salt ^ 0xc0a7;
        fill(ctx, w, h, |u, v| {
            let n = fbm(seed, u * 7.0, v * 7.0, 3);
            let mut g = 0.84 + 0.12 * n;
            let sc = scratches(seed, u, v, 26, 0.12);
            let r = ((u - 0.5).dpowi(2) + (v - 0.5).dpowi(2)).sqrt();
            if kind == 2 {
                g += 0.06 * (r * 260.0).dsin() * sstep(0.46, 0.2, r);
                g += 0.25 * sstep(0.40, 0.48, r) * sstep(0.5, 0.75, fbm(seed ^ 1, u * 14.0, v * 14.0, 2));
                g += 0.35 * sc;
            } else if kind == 1 {
                // black shows its scratches pale
                g += 0.9 * sc;
            } else {
                g -= 0.25 * sc;
            }
            g -= 0.10 * sstep(0.47, 0.5, r);
            [g, g * 0.99, g * 0.97, 1.0]
        });
    })
}

/// The mirror's glass: silver behind it (`Patch::Glass`, as the wall
/// mirror's: nearly all of the light back), its map where the silver
/// has gone to black
fn mirror_mat(map: &Tex) -> Mat {
    let m = phong!({ color: 0xffffff, specular: 0x2a2a2a, shininess: 3000, transparent: true, depthWrite: false });
    {
        let mut mm = m.m();
        mm.map = Some(map.clone());
        mm.patch = Patch::Glass;
        mm.look_settled = true;
        mm.name = "compact_mirror".into();
    }
    {
        let pu = m.r().patch_uniforms.clone();
        let f = &mut pu.borrow_mut().f;
        f[0] = 0.13;
        f[4] = 0.5;
        f[5] = 0.6;
        f[6] = 0.3;
        f[7] = 0.0;
        f[8] = 1.0;
        f[9] = 0.018;
        f[10] = 0.017;
        f[11] = 0.015;
        f[12] = 0.0;
        f[13] = 0.86;
    }
    m
}

/// Where the mirror's silver has gone (grey: how much): black creeping
/// in from its edge, a few spots and pinholes, a smear of powder
fn mirror_tex(salt: u32) -> Tex {
    canvas_tex_opt(64, 64, false, move |ctx, w, h| {
        let seed = salt ^ 0x3177;
        fill(ctx, w, h, |u, v| {
            let r = ((u - 0.5).dpowi(2) + (v - 0.5).dpowi(2)).sqrt() * 2.0;
            let edge = sstep(0.80, 0.98, r + 0.25 * (fbm(seed, u * 9.0, v * 9.0, 3) - 0.5));
            let spot = sstep(0.90, 0.95, vn(seed ^ 2, u * 26.0, v * 26.0)) * sstep(0.4, 0.7, fbm(seed ^ 8, u * 4.0, v * 4.0, 2));
            let smear = 0.25 * sstep(0.6, 0.75, fbm(seed ^ 4, u * 3.0 + v * 6.0, v * 2.0, 2));
            let g = (0.06 + 0.9 * edge + 0.8 * spot + smear).clamp(0.0, 1.0);
            [g, g, g, 1.0]
        });
    })
}

/// The pressed powder seen from above: its shade (light beige, medium,
/// tan), the hollow the puff wore (greyed and glazed with skin's oil, the
/// pan's metal showing at its middle), cracks from a fall, a fingerprint
fn powder_tex(salt: u32) -> Tex {
    canvas_tex_opt(128, 128, false, move |ctx, w, h| {
        let seed = salt ^ 0x90d3;
        let base = [[0.90, 0.78, 0.66], [0.85, 0.70, 0.57], [0.78, 0.61, 0.48]][(salt >> 9) as usize % 3];
        fill(ctx, w, h, |u, v| {
            let r = ((u - 0.5).dpowi(2) + (v - 0.5).dpowi(2)).sqrt() * 2.0;
            // the hollow, where bathroom/washroom_shelf_and_dressing_table.py wore it (x -4 mm, z +3 mm of 58)
            let dh = ((u - 0.431).dpowi(2) + (v - 0.448).dpowi(2)).sqrt() / 0.30;
            let worn = (-dh * dh * 1.6).dexp();
            let n = fbm(seed, u * 30.0, v * 30.0, 3);
            let mut c = [0.0; 3];
            for k in 0..3 {
                c[k] = base[k] * (0.94 + 0.10 * n) * (1.0 - 0.22 * worn);
            }
            // the pan's metal through the middle of the hollow
            let bare = sstep(0.25, 0.12, dh + 0.08 * (n - 0.5));
            for k in 0..3 {
                c[k] = c[k] * (1.0 - bare) + [0.55, 0.56, 0.57][k] * bare;
            }
            // cracks: lines where the cake broke, from a cell lattice
            let (x, y) = (u * 3.0, v * 3.0);
            let (mut d1, mut d2) = (9.0f64, 9.0f64);
            for dj in -1..=1 {
                for di in -1..=1 {
                    let (ii, jj) = (x.floor() as i32 + di, y.floor() as i32 + dj);
                    let fx = f64::from(ii) + hash(seed, ii, jj);
                    let fy = f64::from(jj) + hash(seed ^ 77, ii, jj);
                    let d = (fx - x).dhypot(fy - y);
                    if d < d1 {
                        d2 = d1;
                        d1 = d;
                    } else if d < d2 {
                        d2 = d;
                    }
                }
            }
            let crack = sstep(0.035, 0.0, d2 - d1) * sstep(0.45, 0.65, fbm(seed ^ 3, u * 2.5, v * 2.5, 2));
            for ch in c.iter_mut() {
                *ch *= 1.0 - 0.55 * crack;
            }
            // the rim of the pan round it
            let rim = sstep(0.93, 0.99, r);
            for k in 0..3 {
                c[k] = c[k] * (1.0 - rim) + 0.5 * rim;
            }
            [c[0], c[1], c[2], 1.0]
        });
    })
}
