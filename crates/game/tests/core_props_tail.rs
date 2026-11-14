//! `game::core_props::tail` (`place_props`' second half) against its
//! recorded runs (`tests/fixtures/core_props_tail.json`).
//!
//! Each case replays its hand-off state (the shuffled `cands`, `taken`, the
//! seeded stream advanced by the first half's draws, the exit, the pickups,
//! the panels) on a bare game and runs `place_props_tail` with the test
//! seams: `wall_adj_spot` / `alt_path_exists` answer from the tiles alone,
//! and the builders either replay the recorded builder log (name, arguments,
//! the draw index at the call, the draws the builder used, `top`) with
//! stand-in groups, or run core_props' real builders and check them against
//! that log. Then it compares the draw count, the world subtree (transforms,
//! geometry, materials, textures, lights), interactables, animators and
//! records, and plays a scripted session (animators, interactables, switch
//! flips, timers) on a second seeded stream, comparing snapshots, messages
//! and the end state.
//!
//! The recordings are this game's own. After an intended change to the
//! second half, re-record them from the current code, every case's inputs
//! kept (its tiles, seed, level, weather, hand-off state and session
//! script), through the steps the tests take:
//!   cargo test --release -p game --test core_props_tail -- --ignored rerecord
//! then review the diff (each changed value) and say why in the commit.
#[path = "json_common/mod.rs"]
mod common;
use common::*;
use engine::prelude::*;
use game::Game;
use game::base::records::*;
use game::base::rng::*;
use game::core_props::seam::*;
use game::core_props::*;
use serde_json::{Value, json};
use std::cell::{Cell, RefCell};
use std::collections::{BTreeSet, HashSet, VecDeque};
use std::rc::Rc;

const FIXTURE: &str = "core_props_tail.json";
const TOL: f64 = 1e-9;
/// run-time snapshots (after hundreds of frames of animators)
const RTOL: f64 = 2e-9;

// ------------------------------------------------------------ stand-ins for the seam
// The cases replay on a bare game (its tiles, no walls built), so these two
// answer from the tiles alone.
/// `wall_adj_spot(x, z)` — one `choice` draw
fn wall_adj_spot_rec(g: &Game, x: i32, z: i32) -> Option<WallSpot> {
    let dirs: Vec<(i32, i32)> = [(1, 0), (-1, 0), (0, 1), (0, -1)].into_iter().filter(|&(dx, dz)| g.is_wall_tile(x + dx, z + dz)).collect();
    if dirs.is_empty() {
        return None;
    }
    let (dx, dz) = choice(&dirs);
    let (dxf, dzf) = (dx as f64, dz as f64);
    Some(WallSpot {
        pos: Vec3::new(g.wx(x) + dxf * (CELL / 2.0 - 0.06), 0.0, g.wz(z) + dzf * (CELL / 2.0 - 0.06)),
        normal: Vec3::new(-dxf, 0.0, -dzf),
        yaw: (-dxf).datan2(-dzf),
    })
}
const CELL: f64 = 4.0;
/// `alt_path_exists(sx, sz, ex, ez, extra_blocked)`
fn alt_path_rec(g: &Game, sx: i32, sz: i32, ex: i32, ez: i32, blocked: &[usize]) -> bool {
    if g.is_wall_tile(sx, sz) || g.is_wall_tile(ex, ez) {
        return false;
    }
    let bs: HashSet<usize> = blocked.iter().copied().collect();
    let mut visited = vec![false; (g.w * g.h) as usize];
    let mut q = vec![(sx, sz)];
    visited[g.t_idx(sx, sz)] = true;
    let mut qi = 0;
    while qi < q.len() {
        let (cx, cz) = q[qi];
        qi += 1;
        if cx == ex && cz == ez {
            return true;
        }
        for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let (nx, nz) = (cx + dx, cz + dz);
            if !g.in_grid(nx, nz) {
                continue;
            }
            let idx = g.t_idx(nx, nz);
            if visited[idx] || g.is_wall_tile(nx, nz) || bs.contains(&idx) {
                continue;
            }
            visited[idx] = true;
            q.push((nx, nz));
        }
    }
    false
}

/// one recorded `build_x(...)` call
#[derive(Clone, Debug)]
struct Logged {
    name: String,
    args: Vec<f64>,
    at: u64,
    used: u64,
    top: f64,
}

/// How a case's builders run
#[derive(Clone, Copy, PartialEq)]
enum Builders {
    /// stand-in groups replaying the recorded log (its draws, its `top`)
    StandIn,
    /// core_props' real builders, checked against the recorded log
    Real,
    /// the real builders, their calls logged (`rerecord`)
    Record,
}

fn weather(key: &str) -> &'static game::base::state::Weather {
    game::base::state::WEATHER_BIASES.iter().find(|w| w.key == key).unwrap()
}
fn v3j(j: &Json) -> Vec3 {
    Vec3::new(j.at(0).f(), j.at(1).f(), j.at(2).f())
}
fn cand(j: &Json) -> Cand {
    Cand { x: j.at(0).i() as i32, z: j.at(1).i() as i32, d: j.at(2).i() as i32 }
}

/// The game at the hand-off point, the seam installed.
struct Setup {
    g: Game,
    cx: PropsCtx,
    base: Rc<Cell<u64>>,
    log: Rc<RefCell<VecDeque<Logged>>>,
}
fn setup(case: &Json, builders: Builders) -> Setup {
    game::assets::props::clear_prop_cache();
    let spec = case.get("spec");
    let mut g = Game::new_bare();
    g.w = case.get("W").i() as i32;
    g.h = case.get("H").i() as i32;
    g.tiles = case.get("tiles").arr().iter().map(|v| v.i() as u8).collect();
    g.level_weather = weather(spec.get("weather").s());
    g.level_num = spec.get("level").i() as i32;
    g.level_scp = if spec.get("scp").is_null() { None } else { Some(spec.get("scp").s().to_string()) };
    g.lore_found = spec.get("loreFound").arr().iter().map(|v| v.i() as usize).collect::<BTreeSet<_>>();
    g.pool_plan = if case.get("poolPlan").is_null() {
        None
    } else {
        let p = case.get("poolPlan");
        Some(game::world::PoolPlan { x: p.at(0).i() as i32, z: p.at(1).i() as i32, pw: p.at(2).i() as i32, ph: p.at(3).i() as i32 })
    };
    g.panels = case.get("panels").arr().iter().map(|p| Panel { wx: p.at(0).f(), wz: p.at(1).f(), ..Default::default() }).collect();
    g.state = "playing".into();
    let (sx, sz) = (g.wx(1), g.wz(1));
    g.spawn_pos = Vec3::new(sx, 0.0, sz);
    g.player.pos = Vec3::new(sx, 0.0, sz);
    let h = case.get("handoff");
    if !h.get("exit").is_null() {
        g.exit_data = Some(ExitData { pos: v3j(h.get("exit")), ..Default::default() });
    }
    let pickup = |j: &Json| {
        let o = group();
        o.set_pos_v(&v3j(j.get("p")));
        Pickup { g: o, taken: j.get("taken").b(), base: j.get("base").f(), phase: j.get("phase").f() }
    };
    g.almonds = h.get("almonds").arr().iter().map(pickup).collect();
    g.battery_items = h.get("batteries").arr().iter().map(pickup).collect();
    let cx = PropsCtx {
        cands: h.get("cands").arr().iter().map(cand).collect(),
        taken: h.get("taken").arr().iter().map(cand).collect(),
        exit_tile: None,
    };
    let recorded = || {
        case.get("builders")
            .arr()
            .iter()
            .map(|b| Logged {
                name: b.get("name").s().to_string(),
                args: b.get("args").fs(),
                at: b.get("at").i() as u64,
                used: b.get("used").i() as u64,
                top: b.get("top").f(),
            })
            .collect()
    };
    let log: Rc<RefCell<VecDeque<Logged>>> = Rc::new(RefCell::new(if builders == Builders::Record { VecDeque::new() } else { recorded() }));
    let base = Rc::new(Cell::new(0u64));
    set_props_hooks(PropsHooks { wall_adj_spot: Some(wall_adj_spot_rec), ..PropsHooks::NONE });
    let mut seam = TailSeam { alt_path_exists: Some(Box::new(alt_path_rec)), build: None, lore: None };
    {
        let (log2, base2) = (log.clone(), base.clone());
        seam.build = Some(Box::new(move |g: &mut Game, cx: &mut PropsCtx, b: &TailBuild| {
            let args = match b {
                TailBuild::BrokenMirror(yaw) => vec![*yaw],
                TailBuild::Elevator(f) => vec![if *f { 1.0 } else { 0.0 }],
                _ => vec![],
            };
            let d0 = seeded_draws();
            let built = if builders == Builders::Record {
                let built = b.call(g, cx);
                let used = seeded_draws() - d0;
                log2.borrow_mut().push_back(Logged { name: b.name().into(), args, at: d0 - base2.get(), used, top: built.top });
                built
            } else {
                let e = log2.borrow_mut().pop_front().unwrap_or_else(|| panic!("unexpected builder call {}", b.name()));
                assert_eq!(b.name(), e.name, "builder order");
                assert_eq!(args.len(), e.args.len(), "{}: arguments", e.name);
                for (a, r) in args.iter().zip(&e.args) {
                    assert_close(*a, *r, TOL, &format!("{}: argument", e.name));
                }
                assert_eq!(d0 - base2.get(), e.at, "draws before {}", e.name);
                if builders == Builders::Real {
                    // core_props' builder, checked against the recorded call
                    let built = b.call(g, cx);
                    assert_eq!(seeded_draws() - d0, e.used, "draws of {}", e.name);
                    assert_close(built.top, e.top, TOL, &format!("{}.top", e.name));
                    built
                } else {
                    for _ in 0..e.used {
                        rnd();
                    }
                    Built::new(group(), e.top)
                }
            };
            built.g.set_name(&format!("Builder:{}", b.name()));
            built
        }));
    }
    set_tail_seam(Some(seam));
    Setup { g, cx, base, log }
}

// ------------------------------------------------------------ comparisons
fn order_name(o: EulerOrder) -> &'static str {
    match o {
        EulerOrder::XYZ => "XYZ",
        EulerOrder::YXZ => "YXZ",
        EulerOrder::ZXY => "ZXY",
        EulerOrder::ZYX => "ZYX",
        EulerOrder::YZX => "YZX",
        EulerOrder::XZY => "XZY",
    }
}
fn kind_of(o: &Obj) -> String {
    if o.name().starts_with("Builder:") {
        "builder".into()
    } else if o.is_mesh() {
        "mesh".into()
    } else if o.is_point_light() {
        "plight".into()
    } else if o.is_group() {
        "group".into()
    } else {
        "other".into()
    }
}
/// The geometry's parameters, as (recorded key, value) sorted by key
fn geo_params(p: &GeoParams) -> Vec<(String, f64)> {
    let b = |v: bool| if v { 1.0 } else { 0.0 };
    let mut v: Vec<(&str, f64)> = match *p {
        GeoParams::Box { width, height, depth, ws, hs, ds } => vec![
            ("width", width),
            ("height", height),
            ("depth", depth),
            ("widthSegments", ws as f64),
            ("heightSegments", hs as f64),
            ("depthSegments", ds as f64),
        ],
        GeoParams::Cylinder { radius_top, radius_bottom, height, radial, height_seg, open_ended, theta_start, theta_length } => vec![
            ("radiusTop", radius_top),
            ("radiusBottom", radius_bottom),
            ("height", height),
            ("radialSegments", radial as f64),
            ("heightSegments", height_seg as f64),
            ("openEnded", b(open_ended)),
            ("thetaStart", theta_start),
            ("thetaLength", theta_length),
        ],
        GeoParams::Sphere { radius, ws, hs, phi_start, phi_length, theta_start, theta_length } => vec![
            ("radius", radius),
            ("widthSegments", ws as f64),
            ("heightSegments", hs as f64),
            ("phiStart", phi_start),
            ("phiLength", phi_length),
            ("thetaStart", theta_start),
            ("thetaLength", theta_length),
        ],
        GeoParams::Plane { width, height, ws, hs } => {
            vec![("width", width), ("height", height), ("widthSegments", ws as f64), ("heightSegments", hs as f64)]
        }
        GeoParams::Circle { radius, segments, theta_start, theta_length } => {
            vec![("radius", radius), ("segments", segments as f64), ("thetaStart", theta_start), ("thetaLength", theta_length)]
        }
        GeoParams::Torus { radius, tube, radial, tubular, arc } => {
            vec![("radius", radius), ("tube", tube), ("radialSegments", radial as f64), ("tubularSegments", tubular as f64), ("arc", arc)]
        }
        GeoParams::Buffer => vec![],
        ref o => panic!("unexpected geometry {o:?}"),
    };
    v.sort_by(|a, b| a.0.cmp(b.0));
    v.into_iter().map(|(k, x)| (k.to_string(), x)).collect()
}
#[track_caller]
fn cmp_geo(geo: &Geo, j: &Json, path: &str) {
    assert_eq!(geo.type_name(), j.get("type").s(), "{path}: geometry type");
    let params = geo_params(&geo.params());
    let jp = j.get("params").arr();
    assert_eq!(params.len(), jp.len(), "{path}: geometry params {params:?}");
    for (k, (name, v)) in params.iter().enumerate() {
        assert_eq!(name, jp[k].at(0).s(), "{path}: param name");
        assert_close(*v, jp[k].at(1).f(), TOL, &format!("{path}: param {name}"));
    }
    let r = geo.r();
    assert_eq!(r.position.len() / 3, j.get("n").i() as usize, "{path}: vertex count");
    assert_eq!(r.index.as_ref().map_or(0, |i| i.len()), j.get("idx").i() as usize, "{path}: index count");
    let (mut s, mut sa) = (0.0f64, 0.0f64);
    for (i, &v) in r.position.iter().enumerate() {
        s += v as f64;
        sa += (v as f64).abs() * ((i % 7) + 1) as f64;
    }
    assert_close(s, j.get("s").f(), 1e-7, &format!("{path}: position sum"));
    assert_close(sa, j.get("sa").f(), 1e-7, &format!("{path}: weighted position sum"));
}
#[track_caller]
fn cmp_mat(m: &Mat, j: &Json, path: &str) {
    let r = m.r();
    assert_eq!(mat_type(m), j.get("type").s(), "{path}: material type");
    assert_eq!(r.color.get_hex() as i64, j.get("color").i(), "{path}: color");
    if j.get("type").s() == "lambert" {
        assert_eq!(r.emissive.get_hex() as i64, j.get("emissive").i(), "{path}: emissive");
    }
    assert_close(r.opacity, j.get("op").f(), TOL, &format!("{path}: opacity"));
    assert_eq!(r.transparent, j.get("tr").b(), "{path}: transparent");
    assert_eq!(r.depth_write, j.get("dw").b(), "{path}: depth_write");
    assert_eq!(side_code(r.side), j.get("side").i(), "{path}: side");
    assert_eq!(blend_code(r.blending), j.get("bl").i(), "{path}: blending");
    let jm = j.get("map");
    match (&r.map, jm.is_null()) {
        (None, true) => {}
        (Some(t), false) => {
            assert_eq!((t.width() as f64, t.height() as f64), (jm.get("w").f(), jm.get("h").f()), "{path}: map size");
            let tt = t.r();
            assert_eq!(tt.wrap_s == Wrap::Repeat, jm.get("ws").b(), "{path}: map wrap_s");
            assert_eq!(tt.wrap_t == Wrap::Repeat, jm.get("wt").b(), "{path}: map wrap_t");
            assert_close(tt.repeat.x, jm.get("rep").at(0).f(), TOL, &format!("{path}: map repeat.x"));
            assert_close(tt.repeat.y, jm.get("rep").at(1).f(), TOL, &format!("{path}: map repeat.y"));
        }
        (a, _) => panic!("{path}: map presence got {} recorded {}", a.is_some(), !jm.is_null()),
    }
}
fn side_code(s: Side) -> i64 {
    match s {
        Side::Front => 0,
        Side::Back => 1,
        Side::Double => 2,
    }
}
fn blend_code(b: Blending) -> i64 {
    match b {
        Blending::None => 0,
        Blending::Normal => 1,
        Blending::Additive => 2,
        Blending::Subtractive => 3,
        Blending::Multiply => 4,
    }
}
/// Compare an object with its recorded tree node.
#[track_caller]
fn cmp_tree(o: &Obj, j: &Json, path: &str) {
    let k = kind_of(o);
    assert_eq!(k, j.get("k").s(), "{path}: kind");
    assert_vec_close(&[o.pos().x, o.pos().y, o.pos().z], &j.get("p").fs()[..], TOL, &format!("{path}: position"));
    let r = o.rot();
    let jr = j.get("r");
    assert_vec_close(&[r.x, r.y, r.z], &[jr.at(0).f(), jr.at(1).f(), jr.at(2).f()], 1e-8, &format!("{path}: rotation"));
    assert_eq!(order_name(r.order), jr.at(3).s(), "{path}: rotation order");
    let s = o.scl();
    assert_vec_close(&[s.x, s.y, s.z], &j.get("s").fs()[..], TOL, &format!("{path}: scale"));
    assert_eq!(o.visible(), j.get("v").b(), "{path}: visible");
    if k == "builder" {
        assert_eq!(&o.name()["Builder:".len()..], j.get("b").s(), "{path}: builder");
        return;
    }
    if k == "mesh" {
        cmp_geo(&o.geo(), j.get("g"), path);
        cmp_mat(&o.mat(), j.get("m"), path);
    }
    if k == "plight" {
        let l = j.get("l");
        let ld = o.light().clone();
        assert_eq!(ld.color.get_hex() as i64, l.at(0).i(), "{path}: light color");
        assert_close(ld.intensity, l.at(1).f(), TOL, &format!("{path}: light intensity"));
        assert_close(ld.distance, l.at(2).f(), TOL, &format!("{path}: light distance"));
        assert_close(ld.decay, l.at(3).f(), TOL, &format!("{path}: light decay"));
    }
    let kids = o.children();
    let jk: &[Json] = if j.has("c") { j.get("c").arr() } else { &[] };
    assert_eq!(kids.len(), jk.len(), "{path}: child count");
    for (i, c) in kids.iter().enumerate() {
        cmp_tree(c, &jk[i], &format!("{path}/{i}"));
    }
}
/// The sum of the positions of the pool's waving water (625 vertices), added
/// left to right in f64
fn water_sum(geo: &Geo) -> Option<f64> {
    (geo.vertex_count() == 625).then(|| geo.r().position.iter().map(|&v| v as f64).sum())
}
/// Compare an object with its recorded run-time tree node: `[kind,
/// transform, visible, a, b, c, children]`, where a mesh's a, b, c are its
/// opacity, emissive and water sum, a light's a its intensity.
#[track_caller]
fn cmp_rtree(o: &Obj, j: &Json, path: &str) {
    let k = kind_of(o);
    assert_eq!(k, j.at(0).s(), "{path}: kind");
    let (p, r, s) = (o.pos(), o.rot(), o.scl());
    assert_vec_close(&[p.x, p.y, p.z, r.x, r.y, r.z, s.x, s.y, s.z], &j.at(1).fs()[..], RTOL, &format!("{path}: transform"));
    assert_eq!(o.visible(), j.at(2).b(), "{path}: visible");
    if k == "mesh" {
        let m = o.mat();
        assert_close(m.opacity_v(), j.at(3).f(), RTOL, &format!("{path}: opacity"));
        if m.r().kind == MatKind::Lambert {
            assert_eq!(m.emissive_v().get_hex() as i64, j.at(4).i(), "{path}: emissive");
        }
        if let Some(sum) = water_sum(&o.geo()) {
            assert_close(sum, j.at(5).f(), 1e-6, &format!("{path}: water surface sum"));
        }
    }
    if k == "plight" {
        assert_close(o.intensity(), j.at(3).f(), RTOL, &format!("{path}: intensity"));
    }
    if k == "builder" {
        return;
    }
    let kids = o.children();
    let jk: &[Json] = if j.len() > 6 { j.at(6).arr() } else { &[] };
    assert_eq!(kids.len(), jk.len(), "{path}: child count");
    for (i, c) in kids.iter().enumerate() {
        cmp_rtree(c, &jk[i], &format!("{path}/{i}"));
    }
}
fn fv(m: &std::collections::HashMap<String, f64>, k: &str) -> f64 {
    m.get(k).copied().unwrap_or(0.0)
}
/// Compare the records with the recorded state's records.
#[track_caller]
fn cmp_records(g: &Game, j: &Json, tol: f64, what: &str) {
    let sw = j.get("switches").arr();
    assert_eq!(g.light_switches.len(), sw.len(), "{what}: switch count");
    for (i, s) in g.light_switches.iter().enumerate() {
        let rec = &sw[i];
        assert_close(s.x, rec.get("x").f(), tol, &format!("{what}: switch {i} x"));
        assert_close(s.z, rec.get("z").f(), tol, &format!("{what}: switch {i} z"));
        assert_eq!(s.on, rec.get("on").b(), "{what}: switch {i} on");
        assert_eq!(s.faulty, rec.get("faulty").b(), "{what}: switch {i} faulty");
        assert_eq!(
            s.panels.iter().map(|&p| p as i64).collect::<Vec<_>>(),
            rec.get("panels").arr().iter().map(|p| p.i()).collect::<Vec<_>>(),
            "{what}: switch {i} panels"
        );
        let label = s.g.as_ref().and_then(|o| o.r().user.s.get("label").cloned()).unwrap_or_default();
        assert_eq!(label, rec.get("label").s(), "{what}: switch {i} label");
        assert_close(s.objs["rocker"].rot().x, rec.get("rocker").f(), tol, &format!("{what}: switch {i} rocker"));
        assert_close(fv(&s.f, "motionCooldown"), rec.get("mc").f(), tol, &format!("{what}: switch {i} motionCooldown"));
    }
    let pp = j.get("papers").arr();
    assert_eq!(g.papers.len(), pp.len(), "{what}: papers");
    for (i, p) in g.papers.iter().enumerate() {
        assert_vec_close(&[p.g.pos().x, p.g.pos().y, p.g.pos().z], &pp[i].get("p").fs()[..], tol, &format!("{what}: paper {i}"));
        assert_eq!(p.taken, pp[i].get("taken").b());
    }
    let nn = j.get("notes").arr();
    assert_eq!(g.lore_notes.len(), nn.len(), "{what}: lore notes");
    for (i, n) in g.lore_notes.iter().enumerate() {
        assert_vec_close(&[n.g.pos().x, n.g.pos().y, n.g.pos().z], &nn[i].get("p").fs()[..], tol, &format!("{what}: note {i}"));
        assert_eq!(n.id as i64, nn[i].get("id").i(), "{what}: note {i} id");
    }
    match (&g.pool_bounds, j.get("pool")) {
        (None, p) if p.is_null() => {}
        (Some(b), p) if !p.is_null() => {
            assert_vec_close(&[b.min_x, b.max_x, b.min_z, b.max_z, b.water_y], &p.fs()[..5], tol, &format!("{what}: pool_bounds"));
            assert_eq!(
                b.tiles.iter().map(|&t| t as i64).collect::<Vec<_>>(),
                p.arr()[5..].iter().map(|t| t.i()).collect::<Vec<_>>(),
                "{what}: pool tiles"
            );
        }
        (a, _) => panic!("{what}: pool_bounds got {a:?}"),
    }
    assert_eq!(g.pool_plan.is_some(), j.get("poolPlanLeft").b(), "{what}: pool_plan consumed");
    let vj = j.get("vacuum");
    match &g.vacuum_bot {
        None => assert!(vj.is_null(), "{what}: vacuum missing"),
        Some(v) => {
            let p = v.g.pos();
            assert_vec_close(&[p.x, p.y, p.z], &vj.get("p").fs()[..], tol, &format!("{what}: vacuum pos"));
            assert_close(v.dir, vj.get("yaw").f(), tol, &format!("{what}: vacuum yaw"));
            assert_close(fv(&v.f, "speed"), vj.get("speed").f(), tol, "vacuum speed");
            assert_close(v.timer, vj.get("pick").f(), tol, "vacuum pickTimer");
            assert_close(fv(&v.f, "spinPhase"), vj.get("spin").f(), tol, "vacuum spinPhase");
            assert_close(fv(&v.f, "chaseT"), vj.get("chase").f(), tol, "vacuum chaseT");
            assert_close(v.follow_t, vj.get("follow").f(), tol, "vacuum followT");
            assert_close(fv(&v.f, "stunT"), vj.get("stun").f(), tol, "vacuum stunT");
            assert!(v.objs.contains_key("homeBtn"));
        }
    }
    let sj = j.get("safe");
    match &g.safe_room {
        None => assert!(sj.is_null(), "{what}: safe room missing"),
        Some(s) => assert_vec_close(&[s.x, s.z, s.r, s.discovered as i32 as f64], &sj.fs()[..], tol, &format!("{what}: safe room")),
    }
    let bj = j.get("broken");
    match &g.broken_elevator {
        None => assert!(bj.is_null(), "{what}: broken elevator missing"),
        Some(b) => {
            assert_vec_close(&[b.world.x, b.world.y, b.world.z], &bj.get("world").fs()[..], tol, "brokenElevator.world");
            assert_vec_close(&[b.normal.x, b.normal.y, b.normal.z], &bj.get("normal").fs()[..], tol, "brokenElevator.normal");
            assert_close(b.next_scrape, bj.get("next").f(), tol, "brokenElevator.nextScrape");
            assert_eq!(b.peered, bj.get("peered").b(), "{what}: broken_elevator.peered");
            assert_close(b.light.intensity(), bj.get("li").f(), tol, "brokenElevator.light");
            assert!(b.warn_bar.is_some());
        }
    }
    let hj = j.get("holes").arr();
    assert_eq!(g.floor_holes.len(), hj.len(), "{what}: floor holes");
    for (i, h) in g.floor_holes.iter().enumerate() {
        assert_vec_close(&[h.x, h.z, h.r], &hj[i].fs()[..], tol, &format!("{what}: hole {i}"));
    }
    assert_eq!(
        g.barricade_tiles.iter().map(|&t| t as i64).collect::<Vec<_>>(),
        j.get("barricades").arr().iter().map(|t| t.i()).collect::<Vec<_>>(),
        "{what}: barricade_tiles"
    );
    let cj = j.get("candles").arr();
    assert_eq!(g.haunted_candles.len(), cj.len(), "{what}: candles");
    for (i, c) in g.haunted_candles.iter().enumerate() {
        let jc = &cj[i];
        assert_eq!(!c.lit, jc.get("out").b(), "{what}: candle {i} extinguished");
        assert_close(fv(&c.f, "fadeT"), jc.get("fadeT").f(), tol, &format!("{what}: candle {i} fadeT"));
    }
    let pk = j.get("pickups").arr();
    let all: Vec<&Pickup> = g.almonds.iter().chain(g.battery_items.iter()).collect();
    assert_eq!(all.len(), pk.len(), "{what}: pickups");
    for (i, p) in all.iter().enumerate() {
        let (pp, r) = (p.g.pos(), p.g.rot());
        assert_vec_close(&[pp.x, pp.y, pp.z, r.y, r.z], &pk[i].fs()[..], tol, &format!("{what}: pickup {i}"));
    }
}
#[track_caller]
fn cmp_interactables(g: &Game, j: &Json, tol: f64, full: bool, what: &str) {
    let ji = j.arr();
    assert_eq!(g.interactables.len(), ji.len(), "{what}: interactable count");
    for (i, it) in g.interactables.iter().enumerate() {
        let e = &ji[i];
        assert_vec_close(&[it.pos.x, it.pos.y, it.pos.z], &e.get("p").fs()[..], tol, &format!("{what}: interactable {i} pos"));
        assert_eq!((it.enabled)(g), e.get("en").b(), "{what}: interactable {i} enabled");
        if full {
            assert_close(it.r, e.get("r").f(), 0.0, &format!("{what}: interactable {i} r"));
            assert_eq!(it.prompt, e.get("prompt").s(), "{what}: interactable {i} prompt");
        }
    }
}

// ------------------------------------------------------------ recording (the inverse of each comparison)
/// The inverse of `cmp_geo`
fn ser_geo(geo: &Geo) -> Value {
    let r = geo.r();
    let (mut s, mut sa) = (0.0f64, 0.0f64);
    for (i, &v) in r.position.iter().enumerate() {
        s += v as f64;
        sa += (v as f64).abs() * ((i % 7) + 1) as f64;
    }
    let params: Vec<Value> = geo_params(&geo.params()).into_iter().map(|(k, v)| json!([k, v])).collect();
    json!({
        "type": geo.type_name(), "params": params, "n": r.position.len() / 3,
        "idx": r.index.as_ref().map_or(0, |i| i.len()), "s": s, "sa": sa,
    })
}
/// The inverse of `cmp_mat`
fn ser_mat(m: &Mat) -> Value {
    let r = m.r();
    let mut j = json!({
        "type": mat_type(m), "color": r.color.get_hex(), "op": r.opacity, "tr": r.transparent, "dw": r.depth_write,
        "side": side_code(r.side), "bl": blend_code(r.blending),
    });
    if r.kind == MatKind::Lambert {
        j["emissive"] = json!(r.emissive.get_hex());
    }
    j["map"] = match &r.map {
        None => Value::Null,
        Some(t) => {
            let tt = t.r();
            json!({
                "w": t.width(), "h": t.height(), "ws": tt.wrap_s == Wrap::Repeat, "wt": tt.wrap_t == Wrap::Repeat,
                "rep": [tt.repeat.x, tt.repeat.y],
            })
        }
    };
    j
}
/// The inverse of `cmp_tree`
fn ser_tree(o: &Obj) -> Value {
    let k = kind_of(o);
    let (p, r, s) = (o.pos(), o.rot(), o.scl());
    let mut j = json!({
        "k": k, "p": [p.x, p.y, p.z], "r": [r.x, r.y, r.z, order_name(r.order)], "s": [s.x, s.y, s.z], "v": o.visible(),
    });
    if k == "builder" {
        j["b"] = json!(&o.name()["Builder:".len()..]);
        return j;
    }
    if k == "mesh" {
        j["g"] = ser_geo(&o.geo());
        j["m"] = ser_mat(&o.mat());
    }
    if k == "plight" {
        let l = o.light().clone();
        j["l"] = json!([l.color.get_hex(), l.intensity, l.distance, l.decay]);
    }
    let kids: Vec<Value> = o.children().iter().map(ser_tree).collect();
    if !kids.is_empty() {
        j["c"] = Value::Array(kids);
    }
    j
}
/// The inverse of `cmp_rtree`
fn ser_rtree(o: &Obj) -> Value {
    let k = kind_of(o);
    let (p, r, s) = (o.pos(), o.rot(), o.scl());
    let mut v = vec![json!(k), json!([p.x, p.y, p.z, r.x, r.y, r.z, s.x, s.y, s.z]), json!(o.visible())];
    if k == "builder" {
        return Value::Array(v);
    }
    let abc = if k == "mesh" {
        let m = o.mat();
        let em = if m.r().kind == MatKind::Lambert { m.emissive_v().get_hex() as f64 } else { 0.0 };
        [m.opacity_v(), em, water_sum(&o.geo()).unwrap_or(0.0)]
    } else if k == "plight" {
        [o.intensity(), 0.0, 0.0]
    } else {
        [0.0; 3]
    };
    v.extend(abc.iter().map(|x| json!(x)));
    let kids: Vec<Value> = o.children().iter().map(ser_rtree).collect();
    if !kids.is_empty() {
        v.push(Value::Array(kids));
    }
    Value::Array(v)
}
fn v3v(v: &Vec3) -> Value {
    json!([v.x, v.y, v.z])
}
/// The inverse of `cmp_records`
fn ser_records(g: &Game) -> Value {
    let switches: Vec<Value> = g
        .light_switches
        .iter()
        .map(|s| {
            let label = s.g.as_ref().and_then(|o| o.r().user.s.get("label").cloned()).unwrap_or_default();
            json!({
                "x": s.x, "z": s.z, "on": s.on, "faulty": s.faulty, "panels": s.panels, "label": label,
                "rocker": s.objs["rocker"].rot().x, "mc": fv(&s.f, "motionCooldown"),
            })
        })
        .collect();
    let pool = match &g.pool_bounds {
        None => Value::Null,
        Some(b) => {
            let mut v = vec![json!(b.min_x), json!(b.max_x), json!(b.min_z), json!(b.max_z), json!(b.water_y)];
            v.extend(b.tiles.iter().map(|t| json!(t)));
            Value::Array(v)
        }
    };
    let vacuum = g.vacuum_bot.as_ref().map_or(Value::Null, |v| {
        json!({
            "p": v3v(&v.g.pos()), "yaw": v.dir, "speed": fv(&v.f, "speed"), "pick": v.timer, "spin": fv(&v.f, "spinPhase"),
            "chase": fv(&v.f, "chaseT"), "follow": v.follow_t, "stun": fv(&v.f, "stunT"),
        })
    });
    let broken = g.broken_elevator.as_ref().map_or(Value::Null, |b| {
        json!({
            "world": v3v(&b.world), "normal": v3v(&b.normal), "next": b.next_scrape, "peered": b.peered, "li": b.light.intensity(),
        })
    });
    let candles: Vec<Value> = g.haunted_candles.iter().map(|c| json!({ "out": !c.lit, "fadeT": fv(&c.f, "fadeT") })).collect();
    let pickups: Vec<Value> = g
        .almonds
        .iter()
        .chain(g.battery_items.iter())
        .map(|p| {
            let (pp, r) = (p.g.pos(), p.g.rot());
            json!([pp.x, pp.y, pp.z, r.y, r.z])
        })
        .collect();
    json!({
        "switches": switches,
        "papers": g.papers.iter().map(|p| json!({"p": v3v(&p.g.pos()), "taken": p.taken})).collect::<Vec<_>>(),
        "notes": g.lore_notes.iter().map(|n| json!({"p": v3v(&n.g.pos()), "id": n.id})).collect::<Vec<_>>(),
        "pool": pool,
        "poolPlanLeft": g.pool_plan.is_some(),
        "vacuum": vacuum,
        "safe": g.safe_room.as_ref().map_or(Value::Null, |s| json!([s.x, s.z, s.r, s.discovered as i32])),
        "broken": broken,
        "holes": g.floor_holes.iter().map(|h| json!([h.x, h.z, h.r])).collect::<Vec<_>>(),
        "barricades": g.barricade_tiles,
        "candles": candles,
        "pickups": pickups,
    })
}
/// The inverse of `cmp_interactables`
fn ser_interactables(g: &Game) -> Value {
    g.interactables.iter().map(|it| json!({"p": v3v(&it.pos), "en": (it.enabled)(g), "r": it.r, "prompt": it.prompt})).collect()
}
/// The lore tables the second half reads: each entry's anomaly (`None`
/// for the pool's), and the arcs (their ids in order, their colour)
fn ser_lore(g: &Game) -> Value {
    let scp: Vec<Value> = (0..g.lore_entry_count()).map(|i| g.lore_entry_scp(i).map_or(Value::Null, Value::from)).collect();
    let arcs: Vec<Value> = game::ui::hud::LORE_ARCS.iter().map(|a| json!([a.ids, a.color])).collect();
    json!({ "scp": scp, "arcs": arcs })
}

// ------------------------------------------------------------ the play session
fn use_interactable(g: &mut Game, i: usize) {
    let mut f = std::mem::replace(&mut g.interactables[i].on_use, Box::new(|_| {}));
    f(g);
    g.interactables[i].on_use = f;
}
// ------------------------------------------------------------ a case
/// Runs case `case`: compares what the second half builds and does with its
/// recording, or with `Builders::Record` returns the new recording's
/// outputs (what the game builds and does now).
fn run_case(case: &Json, builders: Builders) -> Option<serde_json::Map<String, Value>> {
    let record = builders == Builders::Record;
    let name = format!("{}{}", case.get("name").s(), if builders == Builders::Real { " (real builders)" } else { "" });
    let spec = case.get("spec");
    let Setup { mut g, mut cx, base, log } = setup(case, builders);
    let h = case.get("handoff");
    let seed = spec.get("seed").i() as u32;
    let mut out = serde_json::Map::new();
    let tail_draws = with_seed(seed, || {
        for _ in 0..h.get("draws").i() {
            rnd();
        }
        base.set(seeded_draws());
        g.place_props_tail(&mut cx, &[]);
        seeded_draws() - base.get()
    });
    if record {
        out.insert("tailDraws".into(), json!(tail_draws));
        let calls: Vec<Value> =
            log.borrow().iter().map(|e| json!({"name": e.name, "args": e.args, "at": e.at, "used": e.used, "top": e.top})).collect();
        out.insert("builders".into(), Value::Array(calls));
    } else {
        assert_eq!(tail_draws, case.get("tailDraws").i() as u64, "{name}: draws of the second half");
        assert!(log.borrow().is_empty(), "{name}: builder calls left: {:?}", log.borrow());
    }
    // ---- the build
    let kids = g.world.children();
    if record {
        out.insert(
            "build".into(),
            json!({
                "tree": kids.iter().map(ser_tree).collect::<Vec<_>>(), "interactables": ser_interactables(&g),
                "animators": g.animators.len(), "records": ser_records(&g),
            }),
        );
    } else {
        let b = case.get("build");
        let jt = b.get("tree").arr();
        assert_eq!(kids.len(), jt.len(), "{name}: world children");
        for (i, o) in kids.iter().enumerate() {
            cmp_tree(o, &jt[i], &format!("{name}: world/{i}"));
        }
        cmp_interactables(&g, b.get("interactables"), TOL, true, &format!("{name} build"));
        assert_eq!(g.animators.len() as i64, b.get("animators").i(), "{name}: animators");
        cmp_records(&g, b.get("records"), TOL, &format!("{name} build"));
    }
    let build_set: HashSet<Obj> = kids.iter().cloned().collect();
    // ---- the session
    let sim = spec.get("sim");
    let frames = sim.get("frames").i();
    let uses: HashSet<i64> = sim.get("use").arr().iter().map(|v| v.i()).collect();
    let flips: HashSet<i64> = sim.get("flip").arr().iter().map(|v| v.i()).collect();
    let snaps: Vec<i64> = sim.get("snap").arr().iter().map(|v| v.i()).collect();
    let stick = sim.get("stick");
    let stick_pos = case.get("stickPos");
    let low = sim.get("lowBattery").i();
    let (w, hh) = (g.w as f64, g.h as f64);
    let (x1, z1) = (g.wx(1), g.wz(1));
    let jframes: &[Json] = if record { &[] } else { case.get("sim").get("frames").arr() };
    let mut frames_out: Vec<Value> = vec![];
    let mut snap_i = 0;
    let mut msgs: Vec<(i64, &str, String, f64)> = vec![];
    let runtime_seed = spec.get("runtimeSeed").i() as u32;
    let d0 = seeded_draws();
    with_seed(runtime_seed, || {
        let mut t = 10.0;
        for f in 0..frames {
            let dt = if f % 7 == 3 { 0.05 } else { 1.0 / 60.0 };
            t += dt;
            g.now_ms += dt * 1000.0;
            let k = f as f64 / frames as f64;
            g.player.pos = Vec3::new(x1 + (k * 6.283).dsin() * w * 1.2, 0.0, z1 + (1.0 - (k * 6.283).dcos()) * hh * 0.9);
            if !stick.is_null() && !stick_pos.is_null() && f >= stick.at(0).i() && f < stick.at(1).i() {
                g.player.pos = Vec3::new(stick_pos.at(0).f() + 0.3, 0.0, stick_pos.at(2).f() + 0.2);
            }
            g.player.flashlight_on = (f as f64) > frames as f64 / 3.0;
            if f == low {
                g.player.battery = 2.5;
            }
            if f == low + 40 {
                g.player.battery = 60.0;
            }
            let pp = g.player.pos;
            g.camera.set_pos(pp.x, 1.62, pp.z);
            let mut phase = |g: &mut Game, ph: &'static str, run: &mut dyn FnMut(&mut Game)| {
                g.msg_text.clear();
                run(g);
                if !g.msg_text.is_empty() {
                    msgs.push((f, ph, g.msg_text.clone(), g.msg_t));
                }
            };
            phase(&mut g, "timer", &mut |g| g.run_scheduled());
            phase(&mut g, "anim", &mut |g| g.run_animators(dt, t));
            phase(&mut g, "use", &mut |g| {
                if !uses.contains(&f) {
                    return;
                }
                let mut i = 0;
                while i < g.interactables.len() {
                    if g.interactables[i].prompt != "[E] CALL THE ELEVATOR" && (g.interactables[i].enabled)(g) {
                        use_interactable(g, i);
                    }
                    i += 1;
                }
            });
            phase(&mut g, "flip", &mut |g| {
                if !flips.contains(&f) {
                    return;
                }
                for i in 0..g.light_switches.len() {
                    g.flip_light_switch(i, if i % 2 == 1 { "motion" } else { "hand" });
                }
            });
            if snap_i < snaps.len() && snaps[snap_i] == f {
                let kids = g.world.children();
                if record {
                    frames_out.push(json!({"f": f, "state": {
                        "tree": kids.iter().map(ser_rtree).collect::<Vec<_>>(), "records": ser_records(&g),
                        "interactables": ser_interactables(&g),
                    }}));
                } else {
                    let rec = &jframes[snap_i];
                    assert_eq!(rec.get("f").i(), f);
                    let st = rec.get("state");
                    let what = format!("{name} frame {f}");
                    let jt = st.get("tree").arr();
                    assert_eq!(kids.len(), jt.len(), "{what}: world children");
                    for (i, o) in kids.iter().enumerate() {
                        cmp_rtree(o, &jt[i], &format!("{what}: world/{i}"));
                    }
                    cmp_records(&g, st.get("records"), RTOL, &what);
                    cmp_interactables(&g, st.get("interactables"), RTOL, false, &what);
                }
                snap_i += 1;
            }
        }
    });
    assert_eq!(snap_i, snaps.len(), "{name}: snapshots taken");
    let sim_draws = seeded_draws() - d0;
    if record {
        let jm: Vec<Value> = msgs.iter().map(|m| json!([m.0, m.1, m.2, m.3])).collect();
        out.insert("sim".into(), json!({"frames": frames_out, "draws": sim_draws, "msgs": jm}));
    } else {
        let sim_j = case.get("sim");
        assert_eq!(sim_draws, sim_j.get("draws").i() as u64, "{name}: run-time draws");
        let jm = sim_j.get("msgs").arr();
        assert_eq!(msgs.len(), jm.len(), "{name}: messages {msgs:?}");
        for (i, m) in msgs.iter().enumerate() {
            let e = &jm[i];
            assert_eq!((m.0, m.1, m.2.as_str()), (e.at(0).i(), e.at(1).s(), e.at(2).s()), "{name}: message {i}");
            assert_close(m.3, e.at(3).f(), 0.0, &format!("{name}: message {i} duration"));
        }
    }
    // ---- the end state
    let kids = g.world.children();
    let added: Vec<Obj> = kids.iter().filter(|o| !build_set.contains(*o)).cloned().collect();
    let off: Vec<i64> = g.panels.iter().map(|p| p.switched_off as i64).collect();
    let tallies = [g.candle_blowout_tally];
    if record {
        out.insert(
            "final".into(),
            json!({
                "tree": kids.iter().map(ser_rtree).collect::<Vec<_>>(), "added": added.iter().map(ser_tree).collect::<Vec<_>>(),
                "records": ser_records(&g), "panelsOff": off,
            }),
        );
        out.insert("finalPlayer".into(), json!({"sanity": g.player.sanity, "battery": g.player.battery}));
        let mut flash = serde_json::Map::new();
        if let Some(b) = g.ui.flash.style("background") {
            flash.insert("background".into(), json!(b));
        }
        if let Some(o) = g.ui.flash.opacity() {
            flash.insert("opacity".into(), json!(o));
        }
        let mut aberration = serde_json::Map::new();
        if let Some(o) = g.ui.aberration.opacity() {
            aberration.insert("opacity".into(), json!(o));
        }
        out.insert(
            "finalGlobals".into(),
            json!({
                "shakeT": g.shake_t, "cameraDipT": g.camera_dip_t, "tallies": tallies, "timers": g.timeouts.len(),
                "flash": flash, "aberration": aberration,
            }),
        );
    } else {
        let fin = case.get("final");
        let what = format!("{name} final");
        let jt = fin.get("tree").arr();
        assert_eq!(kids.len(), jt.len(), "{what}: world children");
        for (i, o) in kids.iter().enumerate() {
            cmp_rtree(o, &jt[i], &format!("{what}: world/{i}"));
        }
        let ja = fin.get("added").arr();
        assert_eq!(added.len(), ja.len(), "{what}: objects added by the session");
        for (i, o) in added.iter().enumerate() {
            cmp_tree(o, &ja[i], &format!("{what}: added/{i}"));
        }
        cmp_records(&g, fin.get("records"), RTOL, &what);
        assert_eq!(off, fin.get("panelsOff").arr().iter().map(|v| v.i()).collect::<Vec<_>>(), "{what}: panels switched off");
        let fp = case.get("finalPlayer");
        assert_close(g.player.sanity, fp.get("sanity").f(), TOL, "sanity");
        assert_close(g.player.battery, fp.get("battery").f(), TOL, "battery");
        let fg = case.get("finalGlobals");
        assert_close(g.shake_t, fg.get("shakeT").f(), 0.0, "shakeT");
        assert_close(g.camera_dip_t, fg.get("cameraDipT").f(), 0.0, "cameraDipT");
        let tl = fg.get("tallies");
        assert_eq!(tallies, [tl.at(0).f()], "{what}: candle tallies");
        assert_eq!(g.timeouts.len() as i64, fg.get("timers").i(), "{what}: pending timers");
        let fl = fg.get("flash");
        assert_eq!(
            g.ui.flash.style("background").map(|s| s.to_string()),
            if fl.has("background") { Some(fl.get("background").s().to_string()) } else { None },
            "{what}: #flash background"
        );
        assert_eq!(g.ui.flash.opacity(), if fl.has("opacity") { Some(fl.get("opacity").f()) } else { None }, "{what}: #flash opacity");
        let ab = fg.get("aberration");
        match g.ui.aberration.opacity() {
            Some(v) => assert_close(v, ab.get("opacity").f(), 1e-12, "aberration opacity"),
            None => assert!(!ab.has("opacity"), "{what}: aberration opacity"),
        }
    }
    // the working elevator leaves the level (flow.rs)
    if let Some(i) = g.interactables.iter().position(|it| it.prompt == "[E] CALL THE ELEVATOR") {
        assert!((g.interactables[i].enabled)(&g));
        use_interactable(&mut g, i);
        assert_eq!(g.state, "transition", "{name}: elevator → next_level_via_elevator");
    }
    set_tail_seam(None);
    set_props_hooks(PropsHooks::NONE);
    record.then_some(out)
}

/// The second half alone: stand-in builders replaying the recorded builder log.
#[test]
fn tail_matches_reference() {
    let fx = Json::load(FIXTURE);
    let cases = fx.get("cases").arr();
    assert!(cases.len() >= 20);
    for c in cases {
        run_case(c, Builders::StandIn);
    }
}
/// The same with core_props' real builders (their draws and `top` must match the recorded log).
#[test]
fn tail_matches_reference_with_real_builders() {
    let fx = Json::load(FIXTURE);
    for c in fx.get("cases").arr() {
        run_case(c, Builders::Real);
    }
}

/// On its own: rocker, the controlled panels, the motion toast.
#[test]
fn flip_switch() {
    let mut g = Game::new_bare();
    g.panels = (0..6).map(|i| Panel { wx: i as f64, ..Default::default() }).collect();
    let rocker = mesh(box_geo!(0.05, 0.08, 0.02), basic_mat!());
    rocker.rotation().x = -0.35;
    let mut sw = LightSwitch { on: true, panels: vec![1, 3, 9], ..Default::default() };
    sw.objs.insert("rocker".into(), rocker.clone());
    g.light_switches.push(sw);
    g.flip_light_switch(0, "player");
    assert!(!g.light_switches[0].on);
    assert_eq!(rocker.rot().x, 0.35);
    assert_eq!(g.panels.iter().map(|p| p.switched_off).collect::<Vec<_>>(), vec![false, true, false, true, false, false]);
    assert_eq!(g.msg_text, "");
    g.flip_light_switch(0, "motion");
    assert!(g.light_switches[0].on);
    assert_eq!(rocker.rot().x, -0.35);
    assert!(g.panels.iter().all(|p| !p.switched_off));
    assert_eq!(g.msg_text, "THE SWITCH FLIPPED ON ITS OWN");
    assert_eq!(g.msg_t, 1.4);
    g.flip_light_switch(5, "motion"); // no such switch: nothing
}

/// Every weather on synthetic levels with stand-in builders, then a long
/// session of animators, interactables, switch flips and timers: no panics.
#[test]
fn smoke() {
    for seed in 0..18u32 {
        let size = 25 + (seed as i32 % 4) * 8;
        let (mut g, dist) = Game::test_level(seed * 7919 + 13, size);
        g.level_weather = &game::base::state::WEATHER_BIASES[(seed as usize) % 6];
        g.level_num = 1 + seed as i32;
        g.level_scp = [None, Some("store"), Some("stairwell"), Some("house"), Some("loop")][(seed % 5) as usize].map(|s| s.to_string());
        g.state = "playing".into();
        if seed % 3 == 0 {
            g.lore_found = (0..20).collect();
        }
        g.panels =
            (0..40).map(|i| Panel { wx: (i % 7) as f64 * 4.0 - 12.0, wz: (i / 7) as f64 * 4.0 - 12.0, ..Default::default() }).collect();
        g.exit_data = Some(ExitData { pos: Vec3::new(g.wx(g.w - 2), 0.0, g.wz(g.h - 2)), ..Default::default() });
        g.spawn_pos = g.player.pos;
        // the pool pre-scan `build_world` does
        'outer: for (pw, ph) in [(3, 3), (4, 3), (3, 4)] {
            for z in 2..=(g.h - 2 - ph) {
                for x in 2..=(g.w - 2 - pw) {
                    if (0..ph).all(|zz| (0..pw).all(|xx| !g.is_wall_tile(x + xx, z + zz))) {
                        g.pool_plan = Some(game::world::PoolPlan { x, z, pw, ph });
                        break 'outer;
                    }
                }
            }
        }
        let mut cands = vec![];
        for z in 1..g.h - 1 {
            for x in 1..g.w - 1 {
                let d = dist[g.t_idx(x, z)];
                if g.tiles[g.t_idx(x, z)] == 0 && d > 2 {
                    cands.push(Cand { x, z, d });
                }
            }
        }
        with_seed(seed + 1, || {
            for i in (1..cands.len()).rev() {
                let j = (rnd() * (i + 1) as f64).floor() as usize;
                cands.swap(i, j);
            }
        });
        let mut cx = PropsCtx { cands, taken: vec![], exit_tile: None };
        set_props_hooks(PropsHooks { wall_adj_spot: Some(wall_adj_spot_rec), ..PropsHooks::NONE });
        set_tail_seam(Some(TailSeam {
            alt_path_exists: Some(Box::new(alt_path_rec)),
            // the real builders on even seeds, cheap stand-ins on odd ones
            build: Some(Box::new(move |g: &mut Game, cx: &mut PropsCtx, b: &TailBuild| {
                if seed % 2 == 0 {
                    b.call(g, cx)
                } else {
                    rnd();
                    Built::new(group(), 0.7)
                }
            })),
            lore: Some(TailLore {
                scp: (0..52)
                    .map(|i| match i {
                        36..=41 => Some("stairwell".to_string()),
                        42..=44 => Some("store".into()),
                        45..=48 => Some("house".into()),
                        49..=51 => Some("loop".into()),
                        _ => None,
                    })
                    .collect(),
                arcs: vec![(vec![7, 8, 24], 0x60a0ff), (vec![3, 4, 5], 0xffb060), (vec![36, 38, 39, 41], 0xd8d2c4)],
            }),
        }));
        with_seed(seed * 31 + 7, || g.place_props_tail(&mut cx, &dist));
        assert!(g.pool_plan.is_none());
        with_seed(seed ^ 0xabcdef, || {
            let mut t = 0.0;
            for f in 0..2400 {
                let dt = if f % 5 == 0 { 0.1 } else { 1.0 / 60.0 };
                t += dt;
                g.now_ms += dt * 1000.0;
                // wander between the interactables so every proximity branch runs
                if !g.interactables.is_empty() {
                    let p = g.interactables[(f / 90) % g.interactables.len()].pos;
                    g.player.pos = Vec3::new(p.x + 0.2, 0.0, p.z);
                }
                g.player.flashlight_on = f % 400 > 150;
                g.player.battery = if (f / 300) % 3 == 1 { 2.0 } else { 80.0 };
                let pp = g.player.pos;
                g.camera.set_pos(pp.x, 1.6, pp.z);
                g.run_scheduled();
                g.run_animators(dt, t);
                if f % 45 == 0 {
                    for i in 0..g.interactables.len() {
                        if g.interactables[i].prompt == "[E] CALL THE ELEVATOR" {
                            continue;
                        }
                        if (g.interactables[i].enabled)(&g) {
                            use_interactable(&mut g, i);
                        }
                    }
                }
                if f % 200 == 7 {
                    for i in 0..g.light_switches.len() {
                        g.flip_light_switch(i, "motion");
                    }
                }
            }
        });
        g.world.update_matrix_world(true);
        set_tail_seam(None);
        set_props_hooks(PropsHooks::NONE);
    }
}

/// The lore tables the second half reads (each entry's anomaly, the arcs
/// and their colours) are the ones recorded with the cases: a change to them
/// changes which notes a level lays and how they are inked, and shows here
/// by name. `lore_arc_for` and `lore_note_boosted` agree with the arcs.
#[test]
fn lore_tables_match_reference() {
    let g0 = Game::new_bare();
    let fx = Json::load(FIXTURE);
    let lore = fx.get("lore");
    let scp = lore.get("scp").arr();
    assert_eq!(g0.lore_entry_count(), scp.len(), "lore entries");
    for (i, s) in scp.iter().enumerate() {
        assert_eq!(g0.lore_entry_scp(i), if s.is_null() { None } else { Some(s.s().to_string()) }, "LORE_ENTRIES[{i}].scp");
    }
    let arcs = lore.get("arcs").arr();
    assert_eq!(game::ui::hud::LORE_ARCS.len(), arcs.len(), "lore arcs");
    for (a, arc) in arcs.iter().enumerate() {
        assert_eq!(g0.lore_arc_color(a) as i64, arc.at(1).i(), "LORE_ARCS[{a}].color");
    }
    for found in [vec![], vec![7usize, 3], vec![36, 2, 9, 14], vec![52, 54]] {
        let mut g = Game::new_bare();
        g.lore_found = found.iter().copied().collect();
        for id in 0..scp.len() {
            let rec_arc = arcs.iter().position(|a| a.at(0).arr().iter().any(|v| v.i() as usize == id));
            assert_eq!(g.lore_arc_for(id), rec_arc, "lore_arc_for({id})");
            let rec_boost = arcs.iter().any(|a| {
                let ids: Vec<usize> = a.at(0).arr().iter().map(|v| v.i() as usize).collect();
                match ids.iter().position(|&i| i == id) {
                    Some(k) if k > 0 => ids[..k].iter().any(|i| found.contains(i)),
                    _ => false,
                }
            });
            assert_eq!(g.lore_note_boosted(id), rec_boost, "lore_note_boosted({id})");
        }
    }
}

// ------------------------------------------------------------ re-recording
/// A recorded input as it was read
fn value_of(j: &Json) -> Value {
    match j {
        Json::Null => Value::Null,
        Json::Bool(b) => json!(b),
        Json::Num(n) => json!(n),
        Json::Str(s) => json!(s),
        Json::Arr(v) => Value::Array(v.iter().map(value_of).collect()),
        Json::Obj(v) => Value::Object(v.iter().map(|(k, x)| (k.clone(), value_of(x))).collect()),
    }
}
/// Whole numbers written as integers (the file stays small; every number
/// reads back the same)
fn compact(v: Value) -> Value {
    match v {
        Value::Number(n) => match n.as_f64() {
            Some(x) if n.is_f64() && x.fract() == 0.0 && x.abs() < 1e15 => json!(x as i64),
            _ => Value::Number(n),
        },
        Value::Array(a) => Value::Array(a.into_iter().map(compact).collect()),
        Value::Object(o) => Value::Object(o.into_iter().map(|(k, x)| (k, compact(x))).collect()),
        v => v,
    }
}
/// What a case is: its inputs, kept by a re-record
const INPUTS: &[&str] = &["name", "spec", "W", "H", "tiles", "panels", "poolPlan", "handoff", "stickPos"];

#[test]
#[ignore]
fn rerecord() {
    let fx = Json::load(FIXTURE);
    let mut cases = vec![];
    for c in fx.get("cases").arr() {
        let mut rec: serde_json::Map<String, Value> = INPUTS.iter().map(|k| (k.to_string(), value_of(c.get(k)))).collect();
        rec.extend(run_case(c, Builders::Record).unwrap());
        cases.push(Value::Object(rec));
    }
    let n = cases.len();
    let out = compact(json!({ "lore": ser_lore(&Game::new_bare()), "cases": cases }));
    let p = format!("{}/tests/fixtures/{FIXTURE}", env!("CARGO_MANIFEST_DIR"));
    std::fs::write(&p, serde_json::to_string(&out).unwrap()).unwrap();
    println!("re-recorded {n} cases into {p}");
}
