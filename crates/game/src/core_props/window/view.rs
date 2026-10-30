//! What a window looks out on (`docs/WINDOWS.md` 2): a farm track running
//! away from the glass into fog (`visuals::photo::window_view_tex`), laid on
//! a ground disc and a dome round a point just outside the glass
//! (`Patch::WindowView`), the same from every window; a streetlamp close
//! outside, drawn as itself beyond the glass (`Material::portal`), its light
//! pooled on the ground out there and thrown in through the glass onto the
//! floor (a decal: the shape of the window, its bars' shadows, its blind's)
//! and nowhere else (a window lights no wall). The deeper the floor, the
//! later it is.
use super::{Kind, Window, SLAB};
use crate::prelude::*;

/// The ground outside lies this far under the floor, m; the photograph was
/// taken this high over it
pub const GROUND: f64 = -0.40;
pub const CAMERA_H: f64 = 1.70;
/// The lamp: its pole stands this far out from the glass and this far to
/// the side, its head over this far out and this high over the floor, m
pub const LAMP_OUT: f64 = 4.5;
pub const LAMP_SIDE: f64 = 1.3;
pub const HEAD_OUT: f64 = 2.9;
pub const HEAD_Y: f64 = GROUND + 4.95;
/// Sodium light (linear)
pub const SODIUM: [f64; 3] = [1.0, 0.42, 0.10];

/// How much light is left out there on floor `depth` of the rooms (2 the
/// first below the Lobby): the exposure and how far into the night
pub fn light_left(depth: i32) -> (f64, f64) {
    let d = (depth - 2).max(0) as f64;
    let night = (d / 5.0).clamp(0.0, 1.0);
    (0.60 * 0.80f64.powf(d), night)
}

/// The lamp's side of the track for window `w` (by its seed)
fn lamp_side(w: &Window) -> f64 {
    if w.seed & 1 == 0 {
        LAMP_SIDE
    } else {
        -LAMP_SIDE
    }
}

/// The lamp's head (the window's frame), as low as it hangs
pub fn lamp_head_local(w: &Window) -> Vec3 {
    Vec3::new(lamp_side(w), HEAD_Y - w.lamp_drop, -HEAD_OUT)
}

/// Finish window `i`'s outside: its view, its lamp, the light it throws in
pub fn finish(g: &mut Game, i: usize) {
    let (expo, night) = light_left(g.depth());
    let w = &g.windows[i];
    // ---- the view: one plane an opening at the back of the cut
    let tex = crate::visuals::photo::window_view_tex();
    let view = basic_mat!({ map: &tex, fog: false });
    {
        let mut m = view.m();
        m.patch = Patch::WindowView;
        m.look_settled = true;
        m.name = "window_view".into();
    }
    let o = w.world(0.0, GROUND + CAMERA_H, 0.0);
    let lamp = w.world(lamp_head_local(w).x, lamp_head_local(w).y, -HEAD_OUT);
    {
        let pu = view.r().patch_uniforms.clone();
        let f = &mut pu.borrow_mut().f;
        *f = [
            o.x as f32,
            o.y as f32,
            o.z as f32,
            CAMERA_H as f32,
            -w.normal.x as f32,
            -w.normal.z as f32,
            300.0,
            expo as f32,
            lamp.x as f32,
            lamp.y as f32,
            lamp.z as f32,
            1.0,
            night as f32,
            0.0,
            0.0,
            0.0,
        ];
    }
    let openings: Vec<[f64; 4]> = match w.kind {
        Kind::Strip => {
            let x0 = w.lights.iter().map(|l| l.rect[0]).fold(f64::INFINITY, f64::min);
            let x1 = w.lights.iter().map(|l| l.rect[1]).fold(f64::NEG_INFINITY, f64::max);
            vec![[x0 - 0.06, x1 + 0.06, w.sill, w.head]]
        }
        Kind::DoubleHung => w.lights.chunks(2).map(|p| [p[0].rect[0] - 0.08, p[0].rect[1] + 0.08, w.sill, w.head]).collect(),
    };
    for r in &openings {
        let plane = mesh(plane_geometry(r[1] - r[0], r[3] - r[2], 1.0, 1.0), &view);
        plane.set_pos((r[0] + r[1]) / 2.0, (r[2] + r[3]) / 2.0, 0.0008);
        w.root.add(&plane);
    }
    // ---- what is beyond the glass: drawn through the row's daylight
    let x0 = w.lights.iter().map(|l| l.rect[0]).fold(f64::INFINITY, f64::min);
    let x1 = w.lights.iter().map(|l| l.rect[1]).fold(f64::NEG_INFINITY, f64::max);
    let y0 = w.lights.iter().map(|l| l.rect[2]).fold(f64::INFINITY, f64::min);
    let y1 = w.lights.iter().map(|l| l.rect[3]).fold(f64::NEG_INFINITY, f64::max);
    let c = w.world((x0 + x1) / 2.0, (y0 + y1) / 2.0, 0.0008);
    {
        let sky = sky_light(expo, night);
        let f = &mut w.portal.borrow_mut().f;
        *f = [
            c.x as f32,
            c.y as f32,
            c.z as f32,
            ((x1 - x0) / 2.0) as f32,
            w.right.x as f32,
            0.0,
            w.right.z as f32,
            ((y1 - y0) / 2.0) as f32,
            w.normal.x as f32,
            0.0,
            w.normal.z as f32,
            SLAB as f32,
            sky[0] as f32,
            sky[1] as f32,
            sky[2] as f32,
            // the fog out there (a hundred metres' sight, thinner at night)
            (0.03 * (1.0 - 0.4 * night)) as f32,
        ];
    }
    // ---- the lamp out there
    let lamp_g = group();
    let side = lamp_side(w);
    lamp_g.set_pos(side, GROUND, -LAMP_OUT);
    let portal = w.portal.clone();
    let lit = |m: Mat| {
        {
            let mut mm = m.m();
            mm.portal = Some(portal.clone());
            mm.look_settled = true;
        }
        m
    };
    let steel = lit(phong!({ color: 0x8e908b, specular: 0x303030, shininess: 30 }));
    let housing = lit(phong!({ color: 0x8a8b86, specular: 0x404040, shininess: 40 }));
    let lens = lit(basic_mat!({ color: Color::rgb_linear(1.0, 0.62, 0.30), fog: false }));
    // (in this order: `set_lamp_drop` reads the pole first)
    for (name, m) in [("Window__lampPole", &steel), ("Window__lampHead", &housing), ("Window__lampLens", &lens)] {
        if let Some(geo) = crate::assets::props::prop_part_as_modelled(name) {
            lamp_g.add(&mesh(geo, m));
        }
    }
    w.root.add(&lamp_g);
    let lamp_obj = lamp_g.clone();
    // ---- the glass: the view's light behind it
    let back = view_light(expo, night);
    for l in &w.lights {
        let m = l.glass.mat();
        let pu = m.r().patch_uniforms.clone();
        let f = &mut pu.borrow_mut().f;
        f[1] = back[0] as f32;
        f[2] = back[1] as f32;
        f[3] = back[2] as f32;
    }
    g.windows[i].view = view;
    g.windows[i].lamp = lamp;
    g.windows[i].lamp_obj = Some(lamp_obj);
    // ---- the light it throws in, a light at a time
    rethrow(g, i);
}

/// Throw window `i`'s light in afresh (its lamp moved, a blind let down or
/// drawn up): each light's lit floor and its glow, the old taken away
pub fn rethrow(g: &mut Game, i: usize) {
    let (_, night) = light_left(g.depth());
    for k in 0..g.windows[i].lights.len() {
        if let Some(o) = g.windows[i].lights[k].glow.take() {
            g.world.remove(&o);
        }
        let (patch, obj) = throw_light(g, i, k, night);
        g.windows[i].lights[k].patch = patch;
        if let Some(o) = &obj {
            g.world.add(o);
        }
        g.windows[i].lights[k].glow = obj;
    }
}

/// Hang window `i`'s lamp `drop` m lower: its head and lens down its pole
/// (the pole drawn shorter), its pool and halo in the view with it. The
/// light it throws is not thrown again here (`rethrow`).
pub fn set_lamp_drop(g: &mut Game, i: usize, drop: f64) {
    let w = &mut g.windows[i];
    w.lamp_drop = drop;
    let head = lamp_head_local(w);
    let lamp = w.world(head.x, head.y, head.z);
    w.lamp = lamp;
    if let Some(lg) = &w.lamp_obj {
        let parts = lg.children();
        for (k, o) in parts.iter().enumerate() {
            if k == 0 {
                // the pole, its foot on the ground
                o.scale().set(1.0, ((HEAD_Y - GROUND - drop) / (HEAD_Y - GROUND)).max(0.2), 1.0);
            } else {
                o.set_pos(0.0, -drop, 0.0);
            }
        }
    }
    let pu = w.view.r().patch_uniforms.clone();
    let f = &mut pu.borrow_mut().f;
    f[8] = lamp.x as f32;
    f[9] = lamp.y as f32;
    f[10] = lamp.z as f32;
}

/// The view's sky light on what stands out there (linear)
pub fn sky_light(expo: f64, night: f64) -> [f64; 3] {
    let k = expo * (1.0 - 0.6 * night);
    [0.30 * k, 0.33 * k, 0.40 * k]
}

/// The view's light on the glass from behind: the fog's grey (linear)
fn view_light(expo: f64, night: f64) -> [f64; 3] {
    let k = expo * (1.0 - 0.7 * night);
    [0.05 * k, 0.055 * k, 0.065 * k]
}

/// The floor light `k` of window `i` throws: its lamp's light through the
/// glass and the opening on the wall's face, a cell at a time (the parts
/// another wall stands between dropped), each corner placed where its ray
/// comes down and coloured by what of the glass it came through (the bars,
/// the blind). Returns its lit cells (world x, z) and its decal.
fn throw_light(g: &Game, i: usize, k: usize, night: f64) -> (Vec<[(f64, f64); 4]>, Option<Obj>) {
    let w = &g.windows[i];
    let l = &w.lights[k];
    // (papered over, or its blind down and shut: no light comes in by it)
    if w.covered(k) {
        return (vec![], None);
    }
    let lamp = lamp_head_local(w);
    let r = l.rect;
    // the opening on the wall's face this light shines through
    let (ox0, ox1) = match w.kind {
        Kind::Strip => (r[0] - 0.03, r[1] + 0.03),
        Kind::DoubleHung => (r[0] - 0.05, r[1] + 0.05),
    };
    // a point of the floor (window frame) for a point of the glass (s, t in 0..1)
    let down = |s: f64, t: f64| -> Option<Vec3> {
        let gp = Vec3::new(r[0] + s * (r[1] - r[0]), r[2] + t * (r[3] - r[2]), l.glass_z);
        let dy = lamp.y - gp.y;
        if dy <= 0.01 {
            return None;
        }
        let lam = lamp.y / dy;
        let f = Vec3::new(lamp.x + (gp.x - lamp.x) * lam, 0.0, lamp.z + (gp.z - lamp.z) * lam);
        // through the opening on the wall's face?
        let mu = (w.depth - lamp.z) / (f.z - lamp.z);
        let (cx, cy) = (lamp.x + (f.x - lamp.x) * mu, lamp.y + (f.y - lamp.y) * mu);
        if cx < ox0 || cx > ox1 || cy < w.sill || cy > w.head || f.z < w.depth + 0.02 {
            return None;
        }
        Some(f)
    };
    const N: usize = 18;
    let (mut pos, mut uv, mut idx): (Vec<f32>, Vec<f32>, Vec<u32>) = (vec![], vec![], vec![]);
    let mut col: Vec<f32> = vec![];
    let mut lit: Vec<Vec<Option<Vec3>>> = vec![vec![None; N + 1]; N + 1];
    for (a, row) in lit.iter_mut().enumerate() {
        for (b, cell) in row.iter_mut().enumerate() {
            *cell = down(a as f64 / N as f64, b as f64 / N as f64);
        }
    }
    let s = g.solids.as_ref();
    let from = w.world((r[0] + r[1]) / 2.0, 1.0, w.depth + 0.05);
    let mut cells: Vec<[(f64, f64); 4]> = vec![];
    for a in 0..N {
        for b in 0..N {
            let corners = [lit[a][b], lit[a + 1][b], lit[a + 1][b + 1], lit[a][b + 1]];
            if corners.iter().any(|c| c.is_none()) {
                continue;
            }
            let cs: Vec<Vec3> = corners.iter().map(|c| c.unwrap()).collect();
            // never through another wall: a cell whose middle the window does
            // not see is not lit, unless that middle is under a wall (the
            // wall hides it) and a corner is seen
            if let Some(s) = s {
                let seen = |c: &Vec3| {
                    let p = w.world(c.x, 1.0, c.z);
                    crate::level::archgen::segment_clear(s, g.to_u(from.x), g.to_v(from.z), g.to_u(p.x), g.to_v(p.z), 1.0)
                };
                let mid = Vec3::new((cs[0].x + cs[2].x) / 2.0, 0.0, (cs[0].z + cs[2].z) / 2.0);
                let mw = w.world(mid.x, 1.0, mid.z);
                let under = super::solid_at(s, g.to_u(mw.x), g.to_v(mw.z), 1.0);
                if !(seen(&mid) || under && cs.iter().any(seen)) {
                    continue;
                }
            }
            let quad: Vec<(f64, f64)> = cs
                .iter()
                .map(|c| {
                    let p = w.world(c.x, 0.0, c.z);
                    (p.x, p.z)
                })
                .collect();
            cells.push([quad[0], quad[1], quad[2], quad[3]]);
            let base = (pos.len() / 3) as u32;
            for (j, c) in cs.iter().enumerate() {
                let p = w.world(c.x, c.y + 0.004, c.z);
                pos.extend_from_slice(&[p.x as f32, p.y as f32, p.z as f32]);
                // the lamp's light on the floor: cos over distance squared
                let d = ((c.x - lamp.x).powi(2) + lamp.y.powi(2) + (c.z - lamp.z).powi(2)).sqrt();
                let e = lamp.y / (d * d * d);
                col.extend_from_slice(&[e as f32; 3]);
                let (sa, sb) = [(a, b), (a + 1, b), (a + 1, b + 1), (a, b + 1)][j];
                uv.extend_from_slice(&[sa as f32 / N as f32, sb as f32 / N as f32]);
            }
            idx.extend_from_slice(&[base, base + 2, base + 1, base, base + 3, base + 2]);
        }
    }
    if pos.is_empty() {
        return (cells, None);
    }
    let geo = Geo::new();
    let n = pos.len() / 3;
    // the brightest part of it at 1
    let top = col.iter().cloned().fold(1e-6f32, f32::max);
    let col: Vec<f32> = col.iter().map(|c| c / top).collect();
    geo.set_position(pos).set_normal((0..n).flat_map(|_| [0.0f32, 1.0, 0.0]).collect()).set_uv(uv).set_index(idx);
    geo.set_color(col, 3);
    geo.compute_bounding_box();
    geo.compute_bounding_sphere();
    let mask = shadow_mask(w, k);
    let strength = 0.16 + 0.16 * night;
    let m = basic_mat!({
        map: &mask,
        color: Color::rgb_linear(SODIUM[0] * strength, SODIUM[1] * strength, SODIUM[2] * strength),
        transparent: true,
        blending: AdditiveBlending,
        depthWrite: false,
        fog: false,
        polygonOffset: true,
        polygonOffsetFactor: -2,
        polygonOffsetUnits: -2,
        vertexColors: true,
    });
    {
        let mut mm = m.m();
        mm.look_settled = true;
        mm.name = "window_light".into();
    }
    let o = mesh(geo, &m);
    o.set_render_order(1);
    (cells, Some(o))
}

/// What of light `k`'s glass lets the lamp's light through (white) and what
/// does not (the muntin of a two-light sash, the blind's slats, open or
/// shut): u across the glass, v up it
fn shadow_mask(w: &Window, k: usize) -> Tex {
    const S: usize = 64;
    let l = &w.lights[k];
    let r = l.rect;
    let mut data = vec![0u8; S * S * 4];
    // the blind's bottom (window frame y) and how much its slats let by
    let blind = l.blind.filter(|b| b.down > 0.0).map(|b| {
        let (top, bottom) = match w.kind {
            Kind::DoubleHung => (w.sill + super::model::DH_H, w.sill),
            Kind::Strip => (r[3], r[2]),
        };
        (top - b.down * (top - bottom), 0.08 + 0.75 * b.open)
    });
    for j in 0..S {
        let y = r[2] + (j as f64 + 0.5) / S as f64 * (r[3] - r[2]);
        for i in 0..S {
            let u = (i as f64 + 0.5) / S as f64;
            let t = (j as f64 + 0.5) / S as f64;
            // the edges soft (the lamp's head is a hand wide, not a point)
            let soft = |x: f64| ((x / 0.07).clamp(0.0, 1.0)).powf(0.8);
            let mut v: f64 = soft(u) * soft(1.0 - u) * soft(t) * soft(1.0 - t);
            if l.muntin && (u - 0.5).abs() < 0.016 {
                v *= 0.15;
            }
            if let Some((bottom, open)) = blind {
                if y > bottom {
                    // a slat every 21 mm, its shadow as wide as it stands shut
                    let ph = ((y - bottom) / 0.021).fract();
                    v *= if ph < 1.0 - open { 0.12 } else { 1.0 };
                }
            }
            let a = (v * 255.0).round() as u8;
            let o = ((S - 1 - j) * S + i) * 4;
            data[o..o + 4].copy_from_slice(&[a, a, a, 255]);
        }
    }
    let t = Tex::data_rgba(data, S as u32, S as u32);
    {
        let mut tt = t.t();
        tt.mag_filter = Filter::Linear;
        tt.min_filter = Filter::Linear;
    }
    t
}
