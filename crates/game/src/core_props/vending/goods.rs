//! The goods' shapes (`docs/design/VENDING.md`): a bag of crisps is a pillow of
//! foil sealed flat at its top and bottom, crimped; a bar is the same,
//! slimmer, standing on its end in a narrow coil; a pastry is a flat pouch;
//! almond water is a ribbed plastic bottle, nothing printed on it; batteries
//! are a blister card. Each is made with UVs 0..1 over its printed face, so
//! its wrapper comes from its cell of the goods' atlas (`paint::designs`).
//! Each stands with its bottom at y = 0, centred on x and z, its face
//! toward +z (the glass).
use crate::prelude::*;

/// A sealed pouch `w` wide, `h` tall, bulging to `d` thick in the middle,
/// flat and crimped for `seal` of its height at each end. `slack` (0..1)
/// lets it sag: an opened-out top, a dent. `turned`: the print runs up the
/// pouch (a bar standing on its end) instead of across it. `fine`: the
/// goods at the front of a coil; those behind are coarser.
pub fn pouch(w: f64, h: f64, d: f64, seal: f64, slack: f64, turned: bool, seed: f64, fine: bool) -> Geo {
    let (nu, nv) = if fine { (8usize, 11usize) } else { (4usize, 6usize) };
    let (mut pos, mut uv, mut idx) = (vec![], vec![], vec![]);
    for side in [1.0f64, -1.0] {
        let base = (pos.len() / 3) as u32;
        for j in 0..=nv {
            let t = j as f64 / nv as f64;
            for i in 0..=nu {
                let s = i as f64 / nu as f64;
                let a = 2.0 * s - 1.0;
                // full across the middle, flat at the seals
                let env = smoothstep(t, seal * 0.6, seal * 2.6) * smoothstep(1.0 - t, seal * 0.6, seal * 2.6);
                let across = (1.0 - a * a).max(0.0).powf(0.55);
                let sag = 1.0 - slack * 0.35 * (t * PI).sin() * (0.5 + 0.5 * (s * 7.0 + seed).sin());
                let mut z = side * d / 2.0 * across * env * sag;
                // the crimp's ridges
                if t < seal || t > 1.0 - seal {
                    z += side * 0.0007 * ((t * h) * 900.0).sin();
                }
                // foil wrinkles
                z += side * 0.0012 * env * ((s * 9.0 + seed * 3.1).sin() * (t * 7.0 + seed).cos());
                let flare = 1.0 + 0.05 * (1.0 - env);
                let x = a * w / 2.0 * flare;
                pos.extend_from_slice(&[x as f32, (t * h) as f32, z as f32]);
                // the back reads mirrored (its own side of the print)
                let su = if side > 0.0 { s } else { 1.0 - s };
                let (u, v) = if turned { (t, 1.0 - su) } else { (su, t) };
                uv.extend_from_slice(&[u as f32, v as f32]);
            }
        }
        let row = (nu + 1) as u32;
        for j in 0..nv as u32 {
            for i in 0..nu as u32 {
                let (a, b, c, e) = (base + j * row + i, base + j * row + i + 1, base + (j + 1) * row + i + 1, base + (j + 1) * row + i);
                if side > 0.0 {
                    idx.extend_from_slice(&[a, b, c, a, c, e]);
                } else {
                    idx.extend_from_slice(&[a, c, b, a, e, c]);
                }
            }
        }
    }
    let g = Geo::new();
    g.set_position(pos).set_uv(uv).set_index(idx);
    g.compute_vertex_normals();
    g
}

/// A box with its +z face carrying the whole cell (a blister card)
pub fn card(w: f64, h: f64, d: f64) -> Geo {
    let g = box_geometry(w, h, d, 1.0, 1.0, 1.0);
    g.translate(0.0, h / 2.0, 0.0);
    g
}

/// The almond water's bottle: a ribbed body, its shoulder and neck, as a
/// lathe (its own material), and its cap; `(body, cap)`, the bottle 0.205 m
/// tall. Nothing is printed on it, as on the bottles found on the floors.
pub fn bottle() -> (Geo, Geo) {
    let mut prof = vec![Vec2::new(0.0, 0.0), Vec2::new(0.026, 0.0), Vec2::new(0.0315, 0.006)];
    // the ribbed lower body
    for k in 0..6 {
        let y = 0.012 + k as f64 * 0.012;
        prof.push(Vec2::new(0.0322, y));
        prof.push(Vec2::new(0.0300, y + 0.006));
    }
    prof.extend_from_slice(&[
        Vec2::new(0.0322, 0.090),
        Vec2::new(0.0322, 0.140),
        Vec2::new(0.0290, 0.158),
        Vec2::new(0.0200, 0.175),
        Vec2::new(0.0140, 0.185),
        Vec2::new(0.0135, 0.192),
    ]);
    let body = lathe_geometry(&prof, 18.0, 0.0, PI * 2.0);
    let cap = cylinder_geometry(0.0150, 0.0150, 0.016, 14.0, 1.0, false, 0.0, PI * 2.0);
    cap.translate(0.0, 0.197, 0.0);
    (body, cap)
}
