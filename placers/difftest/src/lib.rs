//! Differential test of the ported prop placers against backrooms.js.
//!
//! `placer-diff <crate>` runs every placer a crate registers on synthetic
//! levels (several sizes, every weather, many seeds), each run wrapped in
//! `with_seed(seed)`, and prints one line per case: the number of seeded draws
//! and hashes of a canonical description of what the placer built (scene
//! graph, `*Positions` lists, interactables), of the scene after animator
//! frames and after every interactable was used. `tools/placer-diff` runs the
//! original JS on the same cases, builds the same description and compares.
//! See tools/placer-diff/README.md. The format here must stay byte-identical
//! with `canon.mjs` there.
#![allow(clippy::all)]

use engine::geometry::GeoParams;
use engine::material::{Blending, MatKind, Side};
use engine::scene::Kind;
use engine::texture::Wrap;
use game::prelude::*;
use std::collections::BTreeSet;
use std::fmt::Write as _;

include!(concat!(env!("OUT_DIR"), "/pp_lists.rs"));

pub type Register = fn(&mut Vec<(&'static str, PlacerFn)>);

/// Every placer crate.
pub fn crates() -> Vec<(&'static str, Register)> {
    vec![
        ("p00", placers_p00::register), ("p01", placers_p01::register), ("p02", placers_p02::register), ("p03", placers_p03::register),
        ("p04", placers_p04::register), ("p05", placers_p05::register), ("p06", placers_p06::register), ("p07", placers_p07::register),
        ("p08", placers_p08::register), ("p09", placers_p09::register), ("p10", placers_p10::register), ("p11", placers_p11::register),
        ("p12", placers_p12::register), ("p13", placers_p13::register), ("p14", placers_p14::register), ("p15", placers_p15::register),
        ("p16", placers_p16::register), ("p17", placers_p17::register), ("p18", placers_p18::register), ("p19", placers_p19::register),
        ("p20", placers_p20::register), ("p21", placers_p21::register), ("p22", placers_p22::register), ("p23", placers_p23::register),
        ("p24", placers_p24::register), ("p25", placers_p25::register), ("p26", placers_p26::register), ("p27", placers_p27::register),
        ("p28", placers_p28::register), ("p29", placers_p29::register), ("p30", placers_p30::register), ("p31", placers_p31::register),
        ("p32", placers_p32::register),
    ]
}

// ------------------------------------------------------------------ canonical text

/// A number quantised to 1e-5 (`Math.floor(v * 1e5 + 0.5)` in JS), marked with `~`
/// so the comparison knows which integers may differ by rounding noise.
pub fn q(v: f64) -> String {
    if v.is_nan() { return "NaN".into(); }
    if v.is_infinite() { return if v > 0.0 { "Inf".into() } else { "-Inf".into() }; }
    let r = (v * 1e5 + 0.5).floor();
    format!("~{}", r as i64)
}
fn hex(c: &Color) -> String { format!("{:06x}", c.get_hex()) }
fn esc(s: &str) -> String { s.replace('\\', "\\\\").replace('"', "\\\"").replace('\n', "\\n") }
fn b(v: bool) -> u8 { v as u8 }

fn sum_attr(a: &[f32]) -> (f64, f64) {
    let (mut s, mut w) = (0.0f64, 0.0f64);
    for (i, v) in a.iter().enumerate() { let v = *v as f64; s += v; w += v * ((i % 7) + 1) as f64; }
    (s, w)
}

fn geo_s(geo: &Geo, out: &mut String) {
    let gg = geo.r();
    let (ty, p) = match &gg.params {
        GeoParams::Buffer => ("Buffer", String::new()),
        GeoParams::Box { width, height, depth, ws, hs, ds } => ("Box", format!("{},{},{},{},{},{}", q(*width), q(*height), q(*depth), ws, hs, ds)),
        GeoParams::Cylinder { radius_top, radius_bottom, height, radial, height_seg, open_ended, theta_start, theta_length } =>
            ("Cylinder", format!("{},{},{},{},{},{},{},{}", q(*radius_top), q(*radius_bottom), q(*height), radial, height_seg, b(*open_ended), q(*theta_start), q(*theta_length))),
        GeoParams::Cone { radius, height, radial, height_seg, open_ended, theta_start, theta_length } =>
            ("Cone", format!("{},{},{},{},{},{},{}", q(*radius), q(*height), radial, height_seg, b(*open_ended), q(*theta_start), q(*theta_length))),
        GeoParams::Sphere { radius, ws, hs, phi_start, phi_length, theta_start, theta_length } =>
            ("Sphere", format!("{},{},{},{},{},{},{}", q(*radius), ws, hs, q(*phi_start), q(*phi_length), q(*theta_start), q(*theta_length))),
        GeoParams::Plane { width, height, ws, hs } => ("Plane", format!("{},{},{},{}", q(*width), q(*height), ws, hs)),
        GeoParams::Circle { radius, segments, theta_start, theta_length } => ("Circle", format!("{},{},{},{}", q(*radius), segments, q(*theta_start), q(*theta_length))),
        GeoParams::Torus { radius, tube, radial, tubular, arc } => ("Torus", format!("{},{},{},{},{}", q(*radius), q(*tube), radial, tubular, q(*arc))),
        GeoParams::Capsule { radius, length, cap_segments, radial } => ("Capsule", format!("{},{},{},{}", q(*radius), q(*length), cap_segments, radial)),
        GeoParams::Octahedron { radius, detail } => ("Octahedron", format!("{},{}", q(*radius), detail)),
        GeoParams::Icosahedron { radius, detail } => ("Icosahedron", format!("{},{}", q(*radius), detail)),
        GeoParams::Dodecahedron { radius, detail } => ("Dodecahedron", format!("{},{}", q(*radius), detail)),
        GeoParams::Tetrahedron { radius, detail } => ("Tetrahedron", format!("{},{}", q(*radius), detail)),
        GeoParams::Ring { inner, outer, theta_seg, phi_seg, theta_start, theta_length } =>
            ("Ring", format!("{},{},{},{},{},{}", q(*inner), q(*outer), theta_seg, phi_seg, q(*theta_start), q(*theta_length))),
        GeoParams::Lathe { segments, phi_start, phi_length } => ("Lathe", format!("{},{},{}", segments, q(*phi_start), q(*phi_length))),
        GeoParams::Shape { curve_segments } => ("Shape", format!("{}", curve_segments)),
    };
    let (ps, pw) = sum_attr(&gg.position);
    let _ = write!(out, "{}({}) v{} i{} P{} X{}", ty, p, gg.position.len() / 3, gg.index.as_ref().map(|i| i.len()).unwrap_or(0), q(ps), q(pw));
    for (tag, a) in [("N", &gg.normal), ("U", &gg.uv), ("C", &gg.color)] {
        if a.is_empty() { let _ = write!(out, " {}-", tag); } else { let _ = write!(out, " {}{}", tag, q(sum_attr(a).0)); }
    }
    let _ = write!(out, " gr{}", gg.groups.len());
}

fn wrap_c(w: Wrap) -> char { match w { Wrap::Clamp => 'C', Wrap::Repeat => 'R', Wrap::Mirror => 'M' } }

fn mat_s(m: &Mat, out: &mut String) {
    let mm = m.r();
    let kind = match mm.kind {
        MatKind::Basic => "Basic", MatKind::Lambert => "Lambert", MatKind::Phong => "Phong", MatKind::Standard => "Standard",
        MatKind::Points => "Points", MatKind::Sprite => "Sprite", MatKind::LineBasic => "Line", MatKind::Depth => "Depth",
    };
    let has_em = matches!(mm.kind, MatKind::Lambert | MatKind::Phong | MatKind::Standard);
    let side = match mm.side { Side::Front => 0, Side::Back => 1, Side::Double => 2 };
    let blend = match mm.blending { Blending::None => 0, Blending::Normal => 1, Blending::Additive => 2, Blending::Subtractive => 3, Blending::Multiply => 4 };
    let _ = write!(out, "{} c{}", kind, hex(&mm.color));
    if has_em { let _ = write!(out, " e{} ei{}", hex(&mm.emissive), q(mm.emissive_intensity)); }
    let _ = write!(out, " o{} t{} s{} dw{} dt{} b{} at{} vc{} fog{} vis{}", q(mm.opacity), b(mm.transparent), side, b(mm.depth_write), b(mm.depth_test),
        blend, q(mm.alpha_test), b(mm.vertex_colors), b(mm.fog), b(mm.visible));
    if mm.polygon_offset { let _ = write!(out, " po{},{}", q(mm.polygon_offset_factor), q(mm.polygon_offset_units)); }
    match &mm.map {
        Some(t) => { let t = t.r(); let _ = write!(out, " map{},{},{},{},{},{}{}", q(t.repeat.x), q(t.repeat.y), q(t.offset.x), q(t.offset.y), q(t.rotation), wrap_c(t.wrap_s), wrap_c(t.wrap_t)); }
        None => out.push_str(" map-"),
    }
    if matches!(mm.kind, MatKind::Basic | MatKind::Lambert | MatKind::Phong | MatKind::Standard) {
        let _ = write!(out, " am{}", b(mm.alpha_map.is_some()));
    }
    if matches!(mm.kind, MatKind::Lambert | MatKind::Phong | MatKind::Standard) {
        let _ = write!(out, " bm{} em{}", b(mm.bump_map.is_some()), b(mm.emissive_map.is_some()));
    }
    if mm.kind == MatKind::Phong { let _ = write!(out, " sp{} sh{}", hex(&mm.specular), q(mm.shininess)); }
    if mm.kind == MatKind::Points { let _ = write!(out, " sz{} sa{}", q(mm.size), b(mm.size_attenuation)); }
    if mm.kind == MatKind::Sprite { let _ = write!(out, " rot{}", q(mm.rotation)); }
    if mm.wireframe { out.push_str(" wf"); }
}

fn obj_s(o: &Obj, depth: usize, out: &mut String) {
    let (p, r, qt, s) = (o.pos(), o.rot(), o.quat(), o.scl());
    {
        let n = o.r();
        let kind = match &n.kind {
            Kind::Object3D | Kind::Group => "Group", Kind::Scene => "Scene", Kind::Mesh { .. } => "Mesh", Kind::Instanced { .. } => "Inst",
            Kind::Skinned { .. } => "Skin", Kind::Points { .. } => "Points",
            Kind::Line { segments, looped, .. } => if *segments { "LineSeg" } else if *looped { "LineLoop" } else { "Line" },
            Kind::Sprite { .. } => "Sprite", Kind::PointLight(_) => "PointL", Kind::SpotLight(_) => "SpotL", Kind::AmbientLight(_) => "AmbL",
            Kind::HemisphereLight(_) => "HemiL", Kind::DirectionalLight(_) => "DirL", Kind::Bone => "Bone", Kind::Camera(_) => "Cam",
        };
        for _ in 0..depth { out.push(' '); }
        out.push_str(kind);
        if !n.name.is_empty() { let _ = write!(out, " n={}", n.name.replace(' ', "_")); }
        let _ = write!(out, " p{},{},{} r{},{},{},{:?} q{},{},{},{} s{},{},{}", q(p.x), q(p.y), q(p.z), q(r.x), q(r.y), q(r.z), r.order,
            q(qt.x), q(qt.y), q(qt.z), q(qt.w), q(s.x), q(s.y), q(s.z));
        if !n.visible { out.push_str(" hidden"); }
        if n.render_order != 0 { let _ = write!(out, " ro{}", n.render_order); }
        if !n.frustum_culled { out.push_str(" nofc"); }
        if !n.matrix_auto_update { out.push_str(" mau0"); }
        match &n.kind {
            Kind::Mesh { geo, mats } | Kind::Skinned { geo, mats, .. } | Kind::Instanced { geo, mats, .. } => {
                out.push_str(" | "); geo_s(geo, out);
                for m in mats { out.push_str(" | "); mat_s(m, out); }
                if let Kind::Instanced { inst, .. } = &n.kind {
                    let (mut sm, mut wm) = (0.0f64, 0.0f64);
                    for (k, m) in inst.matrices.iter().take(inst.count).enumerate() {
                        for (j, e) in m.e.iter().enumerate() { let v = (*e as f32) as f64; sm += v; wm += v * (((k * 16 + j) % 7) + 1) as f64; }
                    }
                    let _ = write!(out, " | inst n{} M{} W{} ic{}", inst.count, q(sm), q(wm), b(inst.colors.is_some()));
                }
            }
            Kind::Points { geo, mat } | Kind::Line { geo, mat, .. } => { out.push_str(" | "); geo_s(geo, out); out.push_str(" | "); mat_s(mat, out); }
            Kind::Sprite { mat, .. } => { out.push_str(" | "); mat_s(mat, out); }
            Kind::PointLight(l) | Kind::SpotLight(l) | Kind::AmbientLight(l) | Kind::HemisphereLight(l) | Kind::DirectionalLight(l) => {
                let _ = write!(out, " | L c{} i{}", hex(&l.color), q(l.intensity));
                if matches!(n.kind, Kind::PointLight(_) | Kind::SpotLight(_)) { let _ = write!(out, " d{} k{}", q(l.distance), q(l.decay)); }
                if matches!(n.kind, Kind::SpotLight(_)) { let _ = write!(out, " a{} pe{}", q(l.angle), q(l.penumbra)); }
                if matches!(n.kind, Kind::HemisphereLight(_)) { let _ = write!(out, " gc{}", hex(&l.ground_color)); }
                if matches!(n.kind, Kind::SpotLight(_) | Kind::DirectionalLight(_)) {
                    if let Some(t) = &l.target { let tp = t.pos(); let _ = write!(out, " tgt{},{},{}", q(tp.x), q(tp.y), q(tp.z)); }
                }
            }
            _ => {}
        }
    }
    out.push('\n');
    for c in o.children() { obj_s(&c, depth + 1, out); }
}

/// The world's children, one line per object (children indented).
pub fn scene_s(g: &Game, out: &mut String) {
    for c in g.world.children() { obj_s(&c, 0, out); }
}
fn pp_s(g: &Game, out: &mut String) {
    for (name, v) in pp_lists(&g.pp) {
        if v.is_empty() { continue; }
        let _ = write!(out, "PP {}", name);
        for p in v.iter() { let _ = write!(out, " {},{},{}", q(p.x), q(p.y), q(p.z)); }
        out.push('\n');
    }
    // extra per-entry fields (JS `{ x, y, z, period }`), kept in parallel lists
    for (list, field, v) in pp_extras(&g.pp) {
        if v.is_empty() { continue; }
        let _ = write!(out, "PPX {}.{}", list, field);
        for x in v.iter() { let _ = write!(out, " {}", q(*x)); }
        out.push('\n');
    }
}
fn inter_s(g: &Game, out: &mut String) {
    for it in &g.interactables {
        let en = (it.enabled)(g);
        let _ = writeln!(out, "I {},{},{} r{} \"{}\" en{}", q(it.pos.x), q(it.pos.y), q(it.pos.z), q(it.r), esc(&it.prompt), b(en));
    }
}
/// `gameplayR1State` and any tiles the placer changed.
fn state_s(g: &Game, lv: &Level, out: &mut String) {
    if let Some(r) = &g.gameplay_r1_state {
        let _ = writeln!(out, "R1 {},{},{},{} {},{},{},{} on{}", q(r.switch_x), q(r.switch_z), r.switch_tile.0, r.switch_tile.1,
            q(r.target_x), q(r.target_z), r.target_tile.0, r.target_tile.1, b(r.on));
    }
    let changed: Vec<String> = g.tiles.iter().enumerate().filter(|(i, t)| lv.tiles.get(*i) != Some(*t)).map(|(i, t)| format!("{}={}", i, t)).collect();
    if !changed.is_empty() || g.tiles.len() != lv.tiles.len() { let _ = writeln!(out, "TILES {} {}", g.tiles.len(), changed.join(",")); }
}
fn player_s(g: &Game, out: &mut String) {
    let p = &g.player;
    let _ = writeln!(out, "PL san{} bat{} stam{} bottles{} batteries{} papers{} boost{} fl{} msg\"{}\"", q(p.sanity), q(p.battery), q(p.stamina),
        p.bottles, p.batteries, p.papers, q(p.speed_boost_until), b(p.flashlight_on), esc(&g.msg_text));
}

/// 64-bit hash (two 32-bit FNV-1a variants over UTF-16 code units), as 16 hex digits.
pub fn hash(s: &str) -> String {
    let (mut a, mut c) = (0x811c9dc5u32, 0x01000193u32 ^ 0x5bd1e995);
    for u in s.encode_utf16() {
        a ^= u as u32; a = a.wrapping_mul(0x01000193);
        c = (c ^ (u as u32)).wrapping_mul(0x5bd1e995); c ^= c >> 13;
    }
    format!("{:08x}{:08x}", a, c)
}

pub fn json_str(s: &str) -> String {
    let mut o = String::with_capacity(s.len() + 2);
    o.push('"');
    for ch in s.chars() {
        match ch {
            '"' => o.push_str("\\\""), '\\' => o.push_str("\\\\"), '\n' => o.push_str("\\n"), '\r' => o.push_str("\\r"), '\t' => o.push_str("\\t"),
            c if (c as u32) < 0x20 => { let _ = write!(o, "\\u{:04x}", c as u32); }
            c => o.push(c),
        }
    }
    o.push('"');
    o
}

// ------------------------------------------------------------------ cases

pub struct Opts {
    pub sizes: Vec<i32>,
    pub seeds: u32,
    pub only: Option<BTreeSet<String>>,
    pub detail: BTreeSet<u64>,
    pub uses: bool,
    /// extra grids (e.g. real archgen layouts written by tools/placer-diff/levels.mjs)
    pub levels_file: Option<String>,
}
impl Default for Opts {
    fn default() -> Self { Opts { sizes: vec![21, 29, 37], seeds: 12, only: None, detail: BTreeSet::new(), uses: true, levels_file: None } }
}

/// Level file: one grid per line, `w h tiles dist` (tiles: one digit per tile,
/// row-major; dist: comma-separated BFS distances).
fn read_levels(path: &str) -> Vec<Level> {
    let txt = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("{path}: {e}"));
    let mut v = vec![];
    for line in txt.lines() {
        let f: Vec<&str> = line.split_whitespace().collect();
        if f.len() < 4 { continue; }
        let (w, h): (i32, i32) = (f[0].parse().unwrap(), f[1].parse().unwrap());
        let tiles: Vec<u8> = f[2].bytes().map(|c| c - b'0').collect();
        let dist: Vec<i32> = f[3].split(',').map(|d| d.parse().unwrap()).collect();
        assert!(tiles.len() == (w * h) as usize && dist.len() == tiles.len(), "bad level line");
        v.push(Level { w, h, tiles, dist });
    }
    v
}

/// Animator frames: (dt, t). The scene is described after frame 1 and after the last.
pub const FRAMES: [(f64, f64); 6] = [(1.0 / 60.0, 5.0), (0.5, 7.25), (2.0, 31.5), (9.0, 40.5), (0.02, 40.52), (16.0, 56.52)];
/// Frames after which the scene is described.
pub const FRAME_DUMPS: [usize; 3] = [0, 2, 5];
pub const USE_FRAMES: [(f64, f64); 2] = [(0.1, 60.0), (0.1, 60.1)];

pub fn case_seed(s: u32, li: u32, wi: u32) -> u32 { s.wrapping_mul(2654435761) ^ (li << 24) ^ (wi << 20) ^ 0x0123_4567 }

struct Level { w: i32, h: i32, tiles: Vec<u8>, dist: Vec<i32> }

/// Odd seeds start with every `*Positions` list holding two positions (as if
/// earlier placers of the level had registered props), even seeds with none.
fn prefill(g: &mut Game, lv: &Level, li: u32, s: u32) {
    if s % 2 == 0 { return; }
    let mut open = vec![];
    for z in 0..lv.h { for x in 0..lv.w { if lv.tiles[(z * lv.w + x) as usize] == 0 { open.push((x, z)); } } }
    if open.is_empty() { return; }
    let (w, h) = (lv.w as f64, lv.h as f64);
    for (j, (_n, list)) in pp_lists_mut(&mut g.pp).into_iter().enumerate() {
        for k in 0..2u64 {
            let (x, z) = open[((j as u64 * 7919 + k * 104729 + s as u64 * 31 + li as u64 * 17) % open.len() as u64) as usize];
            list.push(v3((x as f64 - w / 2.0 + 0.5) * CELL, 0.25 + 0.5 * k as f64, (z as f64 - h / 2.0 + 0.5) * CELL));
        }
    }
}

fn reset(g: &mut Game, lv: &Level, li: u32, s: u32) {
    g.w = lv.w; g.h = lv.h; g.tiles = lv.tiles.clone();
    g.world = group(); g.animators.clear(); g.interactables.clear(); g.pp.reset();
    prefill(g, lv, li, s);
    g.panels.clear(); g.almonds.clear(); g.battery_items.clear(); g.floor_holes.clear(); g.barricade_tiles.clear(); g.pool_bounds = None;
    g.papers.clear(); g.paper_markers.clear(); g.lore_notes.clear(); g.safe_room = None; g.broken_elevator = None;
    g.light_switches.clear(); g.faulty_trip_count = 0.0; g.escalation_toast_fired = false; g.haunted_candles.clear();
    g.drawer_props.clear(); g.vacuum_bot = None; g.tv_data = None; g.phone_data = None; g.clock_data = None; g.exit_data = None; g.man_data = None;
    g.player = Player::default();
    g.player.pos = v3(g.wx(1), 0.0, g.wz(1));
    g.msg_text.clear();
    g.perf_t = 0.0;
    g.now_ms = 0.0;
    g.timeouts.clear();
}

/// Run one case; returns ([draws; 3], [text; 3]) for the build, frames and uses sections.
fn run_case(g: &mut Game, lv: &Level, li: u32, s: u32, f: PlacerFn, seed: u32, uses: bool) -> ([u64; 3], [String; 3]) {
    reset(g, lv, li, s);
    g.current_seed = seed;
    let mut texts: [String; 3] = Default::default();
    let mut draws = [0u64; 3];
    let d0 = seeded_draws();
    with_seed(seed, || f(g, &lv.dist));
    draws[0] = seeded_draws() - d0;
    {
        let s = &mut texts[0];
        let _ = writeln!(s, "#build anim{} inter{}", g.animators.len(), g.interactables.len());
        scene_s(g, s); pp_s(g, s); inter_s(g, s); state_s(g, lv, s);
    }
    let d1 = seeded_draws();
    with_seed(seed ^ 0xA5A5_A5A5, || {
        for (k, &(dt, t)) in FRAMES.iter().enumerate() {
            g.perf_t = t;
            g.run_animators(dt, t);
            if FRAME_DUMPS.contains(&k) { let _ = writeln!(texts[1], "#frame{}", k + 1); scene_s(g, &mut texts[1]); }
        }
    });
    draws[1] = seeded_draws() - d1;
    if uses {
        let d2 = seeded_draws();
        // two rounds (toggles switch back), each: use every enabled interactable, then one frame
        for (round, &(dt, t)) in USE_FRAMES.iter().enumerate() {
            with_seed(seed ^ 0x3C3C_3C3C ^ round as u32, || {
                let n = g.interactables.len();
                for i in 0..n {
                    if i >= g.interactables.len() { break; }
                    if !(g.interactables[i].enabled)(g) { continue; }
                    let mut fu = std::mem::replace(&mut g.interactables[i].on_use, Box::new(|_| {}));
                    fu(g);
                    if i < g.interactables.len() { g.interactables[i].on_use = fu; }
                }
                g.perf_t = t;
                g.run_animators(dt, t);
            });
            let s = &mut texts[2];
            let _ = writeln!(s, "#use{}", round + 1);
            scene_s(g, s); inter_s(g, s); player_s(g, s); state_s(g, lv, s);
        }
        draws[2] = seeded_draws() - d2;
        let s = &mut texts[2];
        let _ = writeln!(s, "TO {}", g.timeouts.len());
        if !g.timeouts.is_empty() {
            with_seed(seed ^ 0x7777_7777, || { g.now_ms += 1e7; g.run_timeouts(); });
            let s = &mut texts[2];
            let _ = writeln!(s, "#timeouts");
            scene_s(g, s); inter_s(g, s); player_s(g, s);
        }
    }
    (draws, texts)
}

/// Run every case of one crate, writing the protocol to `out` (see README).
pub fn run_crate(crate_id: &str, opts: &Opts, out: &mut dyn std::io::Write) -> std::io::Result<()> {
    let reg = crates().into_iter().find(|(id, _)| *id == crate_id).map(|(_, r)| r).expect("unknown crate");
    let mut list = vec![];
    reg(&mut list);
    if let Some(only) = &opts.only { list.retain(|(n, _)| only.contains(*n)); }
    // grids
    let mut levels = vec![];
    for (li, &size) in opts.sizes.iter().enumerate() {
        let (g, dist) = Game::test_level(li as u32 * 7919 + 13, size);
        levels.push(Level { w: g.w, h: g.h, tiles: g.tiles.clone(), dist });
    }
    if let Some(p) = &opts.levels_file { levels.extend(read_levels(p)); }
    let weathers: Vec<&'static str> = game::state::WEATHER_BIASES.iter().map(|w| w.key).collect();
    // header
    let mut h = String::new();
    let _ = write!(h, "{{\"crate\":{},\"seeds\":{},\"uses\":{},\"placers\":[{}],\"weathers\":[{}],\"pp\":[{}],\"ppx\":[{}],\"frames\":[{}],\"frameDumps\":[{}],\"useFrames\":[{}],\"levels\":[",
        json_str(crate_id), opts.seeds, opts.uses,
        list.iter().map(|(n, _)| json_str(n)).collect::<Vec<_>>().join(","),
        weathers.iter().map(|w| json_str(w)).collect::<Vec<_>>().join(","),
        pp_lists(&game::positions::PropPositions::default()).iter().map(|(n, _)| json_str(n)).collect::<Vec<_>>().join(","),
        pp_extras(&game::positions::PropPositions::default()).iter().map(|(l, f, _)| format!("[{},{}]", json_str(l), json_str(f))).collect::<Vec<_>>().join(","),
        FRAMES.iter().map(|(dt, t)| format!("[{:?},{:?}]", dt, t)).collect::<Vec<_>>().join(","),
        FRAME_DUMPS.iter().map(|k| k.to_string()).collect::<Vec<_>>().join(","),
        USE_FRAMES.iter().map(|(dt, t)| format!("[{:?},{:?}]", dt, t)).collect::<Vec<_>>().join(","));
    for (i, lv) in levels.iter().enumerate() {
        if i > 0 { h.push(','); }
        let _ = write!(h, "{{\"w\":{},\"h\":{},\"tiles\":\"{}\",\"dist\":[{}]}}", lv.w, lv.h,
            lv.tiles.iter().map(|t| char::from(b'0' + *t)).collect::<String>(),
            lv.dist.iter().map(|d| d.to_string()).collect::<Vec<_>>().join(","));
    }
    h.push_str("]}");
    writeln!(out, "H\t{}", h)?;
    // one persistent game for every case (like the browser session)
    let (mut g, _) = Game::test_level(13, 21);
    let mut idx: u64 = 0;
    for (pi, (name, f)) in list.iter().enumerate() {
        for (li, lv) in levels.iter().enumerate() {
            for (wi, _w) in weathers.iter().enumerate() {
                g.level_weather = &game::state::WEATHER_BIASES[wi];
                for s in 0..opts.seeds {
                    let seed = case_seed(s, li as u32, wi as u32);
                    let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| run_case(&mut g, lv, li as u32, s, *f, seed, opts.uses)));
                    match r {
                        Ok((d, t)) => {
                            writeln!(out, "C\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}", idx, pi, li, wi, s, seed, d[0], hash(&t[0]), d[1], hash(&t[1]), d[2], hash(&t[2]))?;
                            if opts.detail.contains(&idx) { writeln!(out, "D\t{}\t{}\t{}\t{}", idx, json_str(&t[0]), json_str(&t[1]), json_str(&t[2]))?; }
                        }
                        Err(e) => {
                            let msg = e.downcast_ref::<String>().cloned().or_else(|| e.downcast_ref::<&str>().map(|s| s.to_string())).unwrap_or_default();
                            writeln!(out, "P\t{}\t{}\t{}\t{}\t{}\t{}\t{}", idx, pi, li, wi, s, seed, json_str(&msg))?;
                            // a panic can leave the game half-built: start over
                            let (g2, _) = Game::test_level(13, 21);
                            g = g2;
                            g.level_weather = &game::state::WEATHER_BIASES[wi];
                        }
                    }
                    idx += 1;
                }
            }
        }
        let _ = name;
    }
    writeln!(out, "E\t{}", idx)?;
    Ok(())
}
