//! A window as it is built (`docs/WINDOWS.md` 1): the Blender parts of batch
//! `w01` placed as modelled, at their real sizes
//! (`docs/research/windows-real.md`), the glass, the blinds and the wear
//! made here. What does not move is merged by material.
//!
//! The window's frame (see `super`): x across (to your right from the
//! room), y up from the floor, z into the room from the back of the cut.
use super::{Blind, Frame, Kind, Light, Plan, Window, SLAB};
use crate::assets::props::prop_part_as_modelled;
use crate::base::rng::SplitMix64;
use crate::prelude::*;

/// A double-hung sash window: its rough opening, frame depth, sill height,
/// and how far apart the windows of a row stand (centre to centre), m
pub const DH_W: f64 = 0.91;
pub const DH_H: f64 = 1.37;
pub const DH_D: f64 = 0.116;
pub const DH_SILL: f64 = 0.80;
pub const DH_PITCH: f64 = 1.50;
/// Its frame's members and its sashes (as `w01.py`)
const JAMB: f64 = 0.022;
const HEAD: f64 = 0.022;
const FRAME_SILL: f64 = 0.040;
const STILE: f64 = 0.050;
const TOP_RAIL: f64 = 0.050;
const MEET: f64 = 0.035;
const BOT_RAIL: f64 = 0.070;
const MUNTIN: f64 = 0.019;
const UP_Z: f64 = 0.0365;
const LO_Z: f64 = 0.0785;
/// The casing round it on the wall face
pub const CASE_W: f64 = 0.070;
pub const CASE_T: f64 = 0.019;
pub const CASE_REVEAL: f64 = 0.006;
/// An office strip window: a light's width (mullion to mullion), its
/// glass's height, the frame's depth, a mullion's face, the sill's height
pub const ST_W: f64 = 1.2;
pub const ST_H: f64 = 1.3;
pub const ST_D: f64 = 0.114;
pub const MULL: f64 = 0.050;
pub const ST_SILL: f64 = 0.82;
/// A convector's depth (it stands 0.80 m high, its stool over it)
const CONV_D: f64 = 0.22;
/// A mini blind's slats: their pitch hanging, and stacked
const SLAT_PITCH: f64 = 0.021;
const SLAT_STACK: f64 = 0.0036;

/// The paint a wood sash window came in (sRGB): off-white, cream, a dark
/// green, a brown (the research's "Georgian white" and the later dark ones)
const DH_PAINTS: [u32; 5] = [0xd6d0be, 0xd6d0be, 0xcbbf9c, 0x2f3b2e, 0x4b3727];
/// Office framing: clear anodised gone dull, bronze anodised
const ST_METALS: [u32; 3] = [0x8b8d8c, 0x8b8d8c, 0x4d3f30];
/// Mini blinds: white and off-white aluminium gone grey with dust
const BLIND_PAINTS: [u32; 3] = [0xd8d6cf, 0xcfcab9, 0xb9b6ad];

/// What a piece is drawn with
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Slot {
    Paint,
    Brass,
    Convector,
    Blind,
    Cord,
    Paper,
    News,
    Tape,
    Dirt,
}

/// Blender part `name` (batch `w01`) as modelled; a small box when the part
/// is missing (a build without the batch)
fn part(name: &str) -> Geo {
    prop_part_as_modelled(name).unwrap_or_else(|| box_geometry(0.02, 0.02, 0.02, 1.0, 1.0, 1.0))
}

fn at(x: f64, y: f64, z: f64) -> Mat4 {
    let mut m = Mat4::IDENTITY;
    m.make_translation(x, y, z);
    m
}

fn at_rs(x: f64, y: f64, z: f64, rx: f64, ry: f64, rz: f64, s: [f64; 3]) -> Mat4 {
    let q = Quat::from_euler(&Euler::new(rx, ry, rz));
    Mat4::compose(&Vec3::new(x, y, z), &q, &Vec3::new(s[0], s[1], s[2]))
}

/// Pieces merged into one geometry
fn merge(pieces: &[(Geo, Mat4)]) -> Geo {
    let (mut pos, mut nrm, mut uv, mut idx) = (vec![], vec![], vec![], vec![]);
    for (geo, m) in pieces {
        let g = geo.r();
        let base = (pos.len() / 3) as u32;
        let n = g.position.len() / 3;
        // normals by the inverse transpose (the stool is stretched)
        let mut nm = *m;
        nm.set_position(0.0, 0.0, 0.0);
        nm.invert().transpose();
        for i in 0..n {
            let mut v = Vec3::new(g.position[i * 3] as f64, g.position[i * 3 + 1] as f64, g.position[i * 3 + 2] as f64);
            v.apply_matrix4(m);
            pos.extend_from_slice(&[v.x as f32, v.y as f32, v.z as f32]);
            let mut k = if g.normal.len() >= (i + 1) * 3 {
                Vec3::new(g.normal[i * 3] as f64, g.normal[i * 3 + 1] as f64, g.normal[i * 3 + 2] as f64)
            } else {
                Vec3::new(0.0, 1.0, 0.0)
            };
            k.transform_direction(&nm);
            nrm.extend_from_slice(&[k.x as f32, k.y as f32, k.z as f32]);
            let (u, w) = if g.uv.len() >= (i + 1) * 2 { (g.uv[i * 2], g.uv[i * 2 + 1]) } else { (0.0, 0.0) };
            uv.extend_from_slice(&[u, w]);
        }
        match &g.index {
            Some(ix) => idx.extend(ix.iter().map(|&i| i + base)),
            None => idx.extend((0..n as u32).map(|i| i + base)),
        }
    }
    let geo = Geo::new();
    geo.set_position(pos).set_normal(nrm).set_uv(uv).set_index(idx);
    geo.compute_bounding_box();
    geo.compute_bounding_sphere();
    geo
}

fn pick<T: Copy>(r: &mut SplitMix64, xs: &[T]) -> T {
    xs[((r.next() * xs.len() as f64) as usize).min(xs.len() - 1)]
}

/// Build the row `p`, fitted to `f`, with its cuts; its own stream `r`
/// decides its paint, glass, blinds and wear. The view beyond it, its lamp
/// and the light it throws are `view::finish`'s.
pub fn build(g: &mut Game, p: &Plan, f: &Frame, cuts: Vec<[f64; 6]>, r: &mut SplitMix64) -> Window {
    let root = group();
    root.set_pos(f.origin.x, f.origin.y, f.origin.z);
    root.rotation().y = f.yaw;
    let mut pieces: Vec<(Slot, Geo, Mat4)> = vec![];
    let mut put = |slot: Slot, geo: Geo, m: Mat4| pieces.push((slot, geo, m));
    let mut lights: Vec<([f64; 4], f64)> = vec![];
    let mut milky: Vec<bool> = vec![];
    let mut two_lights = false;
    let depth = f.depth;
    // the frame sits at the back of the cut, just in front of the view
    let back = SLAB + 0.004 - 0.019;
    let (paint, metal) = match p.kind {
        Kind::DoubleHung => (pick(r, &DH_PAINTS), 0xb08d57u32),
        Kind::Strip => (pick(r, &ST_METALS), pick(r, &ST_METALS)),
    };
    match p.kind {
        Kind::DoubleHung => {
            let two = r.next() < 0.5;
            two_lights = two;
            for i in 0..f.n {
                let cx = (i as f64 - (f.n - 1) as f64 / 2.0) * DH_PITCH;
                let y = f.sill;
                put(Slot::Paint, part("Window__dhFrame"), at(cx, y, back));
                put(Slot::Paint, part(if two { "Window__dhUpper2" } else { "Window__dhUpper1" }), at(cx, y, back));
                put(Slot::Paint, part(if two { "Window__dhLower2" } else { "Window__dhLower1" }), at(cx, y, back));
                put(Slot::Brass, part("Window__dhLock"), at(cx, y, back));
                for sx in [-1.0, 1.0] {
                    put(Slot::Brass, part("Window__dhLift"), at(cx + sx * 0.22, y, back));
                }
                // the casing and the apron on the wall face, the stool across the reveal
                put(Slot::Paint, part("Window__dhCasing"), at(cx, y, depth + CASE_T / 2.0));
                put(Slot::Paint, part("Window__apron"), at(cx, y - 0.022, depth + 0.008));
                let front = depth + CASE_T + 0.03;
                let from = back + DH_D - 0.02;
                put(Slot::Paint, part("Window__stool"), at_rs(cx, y - 0.021, front, 0.0, 0.0, 0.0, [1.0, 1.0, (front - from).max(0.05)]));
                // the glass: the upper sash's light and the lower's (inside each sash)
                let iw = DH_W - 2.0 * JAMB - 2.0 * STILE;
                let ih = DH_H - HEAD - FRAME_SILL;
                let up0 = FRAME_SILL + ih / 2.0 - MEET / 2.0;
                lights.push(([cx - iw / 2.0, cx + iw / 2.0, y + up0 + MEET, y + DH_H - HEAD - TOP_RAIL], back + UP_Z));
                lights
                    .push(([cx - iw / 2.0, cx + iw / 2.0, y + FRAME_SILL + BOT_RAIL, y + FRAME_SILL + ih / 2.0 - MEET / 2.0], back + LO_Z));
                milky.extend([false, false]);
                let _ = MUNTIN;
            }
        }
        Kind::Strip => {
            let n = f.n;
            let w = n as f64 * ST_W + MULL;
            let y = f.sill;
            for k in 0..=n {
                let x = -w / 2.0 + MULL / 2.0 + k as f64 * ST_W;
                put(Slot::Paint, part("Window__stMullion"), at(x, y + MULL, back));
            }
            for k in 0..n {
                let cx = -w / 2.0 + MULL + (k as f64 + 0.5) * ST_W - MULL / 2.0;
                // the sill rail, and the head rail turned over (its pocket down)
                put(Slot::Paint, part("Window__stRail"), at(cx, y, back));
                put(Slot::Paint, part("Window__stRail"), at_rs(cx, y + ST_H + 2.0 * MULL, back + ST_D, 0.0, PI, PI, [1.0, 1.0, 1.0]));
                put(Slot::Convector, part("Window__convector"), at(cx, 0.0, depth));
                let gw = ST_W - MULL;
                lights.push(([cx - gw / 2.0, cx + gw / 2.0, y + MULL, y + MULL + ST_H], back + ST_D / 2.0));
                // now and then a unit whose seal failed: milky between its panes
                milky.push(r.next() < 0.18);
            }
            // the stool over the convectors, from the frame to past their front
            let front = depth + CONV_D + 0.025;
            let from = back + ST_D - 0.01;
            put(
                Slot::Convector,
                part("Window__stool"),
                at_rs(
                    0.0,
                    y - 0.021,
                    front,
                    0.0,
                    0.0,
                    0.0,
                    [(w + 0.06) / (DH_W + 2.0 * (CASE_REVEAL + CASE_W) + 0.05), 1.0, (front - from).max(0.05)],
                ),
            );
        }
    }
    let _ = metal;
    // ---- what each light has: a blind, paper taped over it, or nothing
    let mut lights_out: Vec<Light> = vec![];
    let blind_paint = pick(r, &BLIND_PAINTS);
    let has_blinds = r.next() < if p.kind == Kind::Strip { 0.9 } else { 0.65 };
    let mut glass_objs = vec![];
    for (k, (rect, gz)) in lights.iter().enumerate() {
        // a double-hung window's blind hangs over both its sashes: one for the pair
        let first_of_pair = p.kind == Kind::Strip || k % 2 == 0;
        let papered = (p.kind == Kind::Strip || k % 2 == 1) && r.next() < 0.05;
        let blind = if has_blinds && first_of_pair && !papered {
            let roll = r.next();
            let down = if roll < 0.42 {
                0.0
            } else if roll < 0.88 {
                0.15 + 0.55 * r.next()
            } else {
                1.0
            };
            let open = if down >= 1.0 && r.next() < 0.6 { 0.05 } else { 0.35 + 0.6 * r.next() };
            Some(Blind { down, open, skew: (r.next() - 0.5) * 0.06 * (down > 0.0 && down < 1.0) as i32 as f64 })
        } else {
            None
        };
        // the glass: dirt from its own part of the map, the room in it
        let glass = glass_pane(rect, *gz, milky[k], r);
        root.add(&glass);
        glass_objs.push(glass.clone());
        // rain on it on wet floors (the weather fades it in)
        let rain = rain_overlay(g, rect, *gz + 0.003);
        root.add(&rain);
        g.rain_windows.push(rain);
        let muntin = p.kind == Kind::DoubleHung && two_lights;
        lights_out.push(Light { rect: *rect, glass_z: *gz, glass, blind, papered, muntin, patch: vec![] });
    }
    // a double-hung window's blind covers both its lights: the lower takes the pair's
    if p.kind == Kind::DoubleHung {
        for k in (1..lights_out.len()).step_by(2) {
            let b = lights_out[k - 1].blind;
            if !lights_out[k].papered {
                lights_out[k].blind = b;
            }
        }
    }
    // ---- the blinds, the paper and the wear
    for k in 0..lights_out.len() {
        let l = &lights_out[k];
        if p.kind == Kind::DoubleHung && k % 2 == 1 {
            continue;
        }
        if let Some(b) = l.blind {
            // over the whole opening: a double-hung window's both lights
            let (x0, x1) = (l.rect[0] - 0.012, l.rect[1] + 0.012);
            let top = if p.kind == Kind::DoubleHung { f.sill + DH_H - HEAD } else { l.rect[3] };
            let bottom = if p.kind == Kind::DoubleHung { f.sill + FRAME_SILL } else { l.rect[2] };
            let inside = depth - (back + if p.kind == Kind::Strip { ST_D } else { DH_D }) > 0.06;
            let bz = if inside { back + if p.kind == Kind::Strip { ST_D } else { DH_D } + 0.03 } else { depth + CASE_T + 0.025 };
            blind(&mut put, x0, x1, top, bottom, bz, &b);
        }
    }
    for l in lights_out.iter().filter(|l| l.papered) {
        paper_over(&mut put, &l.rect, l.glass_z + 0.012, r);
    }
    // dead flies along the sill, now and then
    if r.next() < 0.5 {
        let l = &lights_out[if p.kind == Kind::DoubleHung { 1 } else { 0 }];
        let n = 3 + (r.next() * 9.0) as usize;
        for _ in 0..n {
            let x = l.rect[0] + r.next() * (l.rect[1] - l.rect[0]);
            let z = l.glass_z + 0.02 + r.next() * 0.05;
            let fly = box_geometry(0.006, 0.0025, 0.004, 1.0, 1.0, 1.0);
            put(
                Slot::Dirt,
                fly,
                at_rs(x, f.sill + 0.002 + if p.kind == Kind::Strip { MULL } else { 0.0 }, z, 0.0, r.next() * PI, 0.0, [1.0, 1.0, 1.0]),
            );
        }
    }
    // ---- one mesh a material
    let mats = materials(paint, blind_paint, r);
    let mut slots: Vec<Slot> = pieces.iter().map(|p| p.0).collect();
    slots.sort();
    slots.dedup();
    for s in slots {
        let list: Vec<(Geo, Mat4)> = pieces.iter().filter(|p| p.0 == s).map(|p| (p.1.clone(), p.2)).collect();
        let o = mesh(merge(&list), mats(s));
        root.add(&o);
    }
    g.world.add(&root);
    let portal = std::rc::Rc::new(std::cell::RefCell::new(PortalU::default()));
    Window {
        kind: p.kind,
        root,
        normal: f.normal,
        right: f.right,
        depth,
        sill: f.sill,
        head: f.head,
        lights: lights_out,
        cuts,
        view: Mat::basic(),
        portal,
        lamp: Vec3::default(),
        hum: None,
        seed: p.seed,
    }
}

/// The materials of a window: its paint, its metals, its blinds, the paper
fn materials(paint: u32, blind_paint: u32, r: &mut SplitMix64) -> impl Fn(Slot) -> Mat {
    let wear = 0.45 + 0.4 * r.next();
    let paint_m = phong!({ color: paint, specular: 0x1a1a1a, shininess: 18 });
    paint_m.m().wear = wear;
    let brass = phong!({ color: 0x8c6a3c, specular: 0x5a4a30, shininess: 50 });
    let conv = phong!({ color: 0xbdb5a0, specular: 0x202020, shininess: 22 });
    conv.m().wear = 0.7;
    let blind = phong!({ color: blind_paint, specular: 0x2a2a2a, shininess: 30, side: DoubleSide });
    blind.m().wear = 0.5;
    let cord = lambert!({ color: 0xcfc9b8 });
    let paper = lambert!({ color: 0xe6e1d3, side: DoubleSide });
    paper.m().wear = 0.35;
    let news = lambert!({ map: newsprint_tex(), side: DoubleSide });
    news.m().wear = 0.3;
    let tape = phong!({ color: 0xd3c7a4, specular: 0x303030, shininess: 40, side: DoubleSide });
    let dirt = lambert!({ color: 0x14110e });
    move |s| match s {
        Slot::Paint => paint_m.clone(),
        Slot::Brass => brass.clone(),
        Slot::Convector => conv.clone(),
        Slot::Blind => blind.clone(),
        Slot::Cord => cord.clone(),
        Slot::Paper => paper.clone(),
        Slot::News => news.clone(),
        Slot::Tape => tape.clone(),
        Slot::Dirt => dirt.clone(),
    }
}

/// A light's glass: a pane over `rect` at `z`, its dirt a part of the map of
/// its own, a failed unit milky
fn glass_pane(rect: &[f64; 4], z: f64, milky: bool, r: &mut SplitMix64) -> Obj {
    let (w, h) = (rect[1] - rect[0], rect[3] - rect[2]);
    let geo = plane_geometry(w, h, 1.0, 1.0);
    let map = crate::visuals::photo::glass_dirt_tex(r.next(), r.next(), 0.55 + 0.25 * r.next());
    let m = phong!({ color: 0xffffff, specular: 0x2a2a2a, shininess: 220, transparent: true, depthWrite: false });
    {
        let mut mm = m.m();
        mm.map = Some(map);
        mm.patch = Patch::Glass;
        mm.look_settled = true;
        mm.name = "window_glass".into();
    }
    {
        let pu = m.r().patch_uniforms.clone();
        let f = &mut pu.borrow_mut().f;
        // the room in it; the view's light behind it (set with the view)
        f[0] = 0.18;
        // no mist
        f[4] = 0.5;
        f[5] = 0.6;
        f[6] = 0.3;
        f[7] = 0.0;
        // dirt: how much of the map's grey, its colour (linear)
        f[8] = 0.12 + 0.20 * r.next() as f32;
        f[9] = 0.20;
        f[10] = 0.18;
        f[11] = 0.15;
        f[12] = if milky { 0.22 + 0.15 * r.next() as f32 } else { 0.0 };
        f[13] = 0.08;
    }
    let o = mesh(geo, &m);
    o.set_pos((rect[0] + rect[1]) / 2.0, (rect[2] + rect[3]) / 2.0, z);
    o.set_render_order(2);
    o
}

/// The rain running down a light on wet floors (`rain_windows`: the weather
/// fades it in and scrolls it)
fn rain_overlay(g: &Game, rect: &[f64; 4], z: f64) -> Obj {
    let m = basic_mat!({ map: g.rain_streak_tex.duplicate(), transparent: true, opacity: 0, blending: NormalBlending, depthWrite: false });
    if let Some(t) = m.map_v() {
        let mut tt = t.t();
        tt.wrap_s = Wrap::Repeat;
        tt.wrap_t = Wrap::Repeat;
        drop(tt);
        t.needs_update();
    }
    m.m().look_settled = true;
    let o = mesh(plane_geometry(rect[1] - rect[0], rect[3] - rect[2], 1.0, 1.0), &m);
    o.set_pos((rect[0] + rect[1]) / 2.0, (rect[2] + rect[3]) / 2.0, z);
    o.set_render_order(3);
    o.set_visible(false);
    o
}

/// A mini blind from `top` (its headrail) toward `bottom` over x0..x1 at z
fn blind(put: &mut impl FnMut(Slot, Geo, Mat4), x0: f64, x1: f64, top: f64, bottom: f64, z: f64, b: &Blind) {
    let w = x1 - x0;
    let cx = (x0 + x1) / 2.0;
    put(Slot::Blind, part("Window__blindHead"), at_rs(cx, top, z, 0.0, 0.0, 0.0, [w, 1.0, 1.0]));
    let total = ((top - bottom - 0.04) / SLAT_PITCH).floor().max(1.0) as usize;
    let hanging = (b.down * total as f64).round() as usize;
    let stacked = total - hanging;
    // the slats stacked under the headrail, then those let down
    let mut y = top - 0.03;
    let tilt = (1.0 - b.open) * 1.35;
    for k in 0..total {
        let let_down = k >= stacked;
        let s = if total > stacked { (k.saturating_sub(stacked)) as f64 / (total - stacked).max(1) as f64 } else { 0.0 };
        let skew = if let_down { b.skew * s } else { 0.0 };
        let rx = if let_down { tilt } else { 0.05 };
        put(Slot::Blind, part("Window__blindSlat"), at_rs(cx, y, z, rx, 0.0, skew / w, [w - 0.006, 1.0, 1.0]));
        y -= if let_down { SLAT_PITCH } else { SLAT_STACK };
    }
    put(Slot::Blind, part("Window__blindBottom"), at_rs(cx, y - 0.008, z, 0.0, 0.0, b.skew / w, [w, 1.0, 1.0]));
    // the ladders and the lift cords, down to the bottom rail
    let len = (top - (y - 0.008)).max(0.02);
    for sx in [-1.0, 1.0] {
        for dz in [-0.012, 0.012] {
            let cord = box_geometry(0.0012, len, 0.0012, 1.0, 1.0, 1.0);
            put(Slot::Cord, cord, at(cx + sx * (w / 2.0 - 0.11), top - len / 2.0, z + dz));
        }
    }
    // the wand at one end, the pull cords at the other
    put(Slot::Blind, part("Window__blindWand"), at(x0 + 0.04, top - 0.02, z + 0.02));
    let pull = (top - bottom) * 0.55;
    put(Slot::Cord, box_geometry(0.0016, pull, 0.0016, 1.0, 1.0, 1.0), at(x1 - 0.035, top - pull / 2.0, z + 0.018));
}

/// Sheets taped over a light from inside (someone covered it): newspaper
/// pages and plain sheets, overlapping, a little crooked, held by strips of
/// tape; now and then one has fallen, and the outside shows through
fn paper_over(put: &mut impl FnMut(Slot, Geo, Mat4), rect: &[f64; 4], z: f64, r: &mut SplitMix64) {
    // a tabloid page, a plain sheet
    let sizes = [(0.29, 0.40), (0.21, 0.297)];
    let mut y = rect[3] + 0.01;
    let mut row = 0;
    while y > rect[2] + 0.02 {
        let (_, sh) = sizes[if r.next() < 0.7 { 0 } else { 1 }];
        let mut x = rect[0] - 0.01 - r.next() * 0.04;
        while x < rect[1] - 0.02 {
            let news = r.next() < 0.72;
            let (pw, ph) = if news { sizes[0] } else { sizes[1] };
            let cx = (x + pw / 2.0).min(rect[1] - pw / 2.0 + 0.015);
            let cy = (y - ph / 2.0).max(rect[2] + ph / 2.0 - 0.015);
            let rot = (r.next() - 0.5) * 0.08;
            let lift = (row as f64 + r.next()) * 0.0007;
            if r.next() > 0.035 {
                let slot = if news { Slot::News } else { Slot::Paper };
                let geo = plane_geometry(pw, ph, 1.0, 1.0);
                if news {
                    // a page of its own of the newsprint (one atlas of four)
                    let k = (r.next() * 4.0) as usize % 4;
                    let uv: Vec<f32> = geo.r().uv.chunks(2).flat_map(|c| [(k as f32 + c[0]) / 4.0, c[1]]).collect();
                    geo.set_uv(uv);
                }
                put(slot, geo, at_rs(cx, cy, z + lift, 0.0, 0.0, rot, [1.0, 1.0, 1.0]));
                // tape over a corner or two, the outer ones most
                for (sx, sy) in [(-1.0, 1.0), (1.0, 1.0), (-1.0, -1.0), (1.0, -1.0)] {
                    let outer = (sx < 0.0 && cx - pw / 2.0 <= rect[0] + 0.02)
                        || (sx > 0.0 && cx + pw / 2.0 >= rect[1] - 0.02)
                        || (sy > 0.0 && row == 0)
                        || (sy < 0.0 && cy - ph / 2.0 <= rect[2] + 0.02);
                    if r.next() < if outer { 0.75 } else { 0.18 } {
                        let t = plane_geometry(0.07 + 0.03 * r.next(), 0.022, 1.0, 1.0);
                        let a = sx * sy * (0.55 + 0.4 * r.next());
                        put(
                            Slot::Tape,
                            t,
                            at_rs(
                                cx + sx * (pw / 2.0 - 0.012),
                                cy + sy * (ph / 2.0 - 0.012),
                                z + lift + 0.002,
                                0.0,
                                0.0,
                                a + rot,
                                [1.0, 1.0, 1.0],
                            ),
                        );
                    }
                }
            }
            x += pw - 0.015;
        }
        y -= sh - 0.02;
        row += 1;
    }
}

/// Newsprint: four pages side by side, grey columns of type too small to
/// read, a headline's bar, a photograph's block; painted from fixed numbers
/// (the same on every build, whatever was built before)
pub fn newsprint_tex() -> Tex {
    thread_local! { static NEWS: std::cell::RefCell<Option<Tex>> = const { std::cell::RefCell::new(None) }; }
    NEWS.with(|n| {
        n.borrow_mut()
            .get_or_insert_with(|| {
                let (pw, ph) = (256u32, 352u32);
                let mut c = Canvas::new(pw * 4, ph);
                let mut r = SplitMix64::new(0x4E_57_53, 0x9A1E);
                for k in 0..4u32 {
                    let x0 = (k * pw) as f64;
                    let tone = 222.0 + r.next() * 18.0;
                    c.set_fill_style(format!("rgb({},{},{})", tone as u8, (tone - 6.0) as u8, (tone - 22.0) as u8));
                    c.fill_rect(x0, 0, pw, ph);
                    // the masthead or a headline across the top
                    c.set_fill_style("rgba(25,25,25,0.85)");
                    let hh = 14.0 + r.next() * 14.0;
                    c.fill_rect(x0 + 14.0, 14.0, pw as f64 - 28.0 - r.next() * 60.0, hh);
                    // columns of type, and a photograph's grey block
                    let cols = 3 + (r.next() * 2.0) as usize;
                    let cw = (pw as f64 - 28.0 - (cols - 1) as f64 * 8.0) / cols as f64;
                    let photo = (r.next() * cols as f64) as usize;
                    for ci in 0..cols {
                        let cx = x0 + 14.0 + ci as f64 * (cw + 8.0);
                        let mut y = 24.0 + hh;
                        if ci == photo {
                            let ph2 = 70.0 + r.next() * 60.0;
                            c.set_fill_style("rgba(70,68,64,0.8)");
                            c.fill_rect(cx, y, cw, ph2);
                            y += ph2 + 8.0;
                        }
                        while y < ph as f64 - 14.0 {
                            let lw = if r.next() < 0.12 { cw * (0.3 + 0.5 * r.next()) } else { cw };
                            c.set_fill_style(format!("rgba(40,40,40,{:.2})", 0.30 + 0.25 * r.next()));
                            c.fill_rect(cx, y, lw, 2.0);
                            y += 4.0 + if r.next() < 0.06 { 6.0 } else { 0.0 };
                        }
                    }
                }
                let t = Tex::from_image(c.to_image());
                t.t().color_space = ColorSpace::Srgb;
                t
            })
            .clone()
    })
}
