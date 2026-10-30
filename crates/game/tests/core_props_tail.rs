//! `game::core_props::tail` (`place_props`' second half) against recorded
//! reference data (fixtures in `tests/fixtures/`).
//!
//! Each case replays the recorded hand-off state (the shuffled
//! `cands`, `taken`, the seeded stream advanced by the first half's draws,
//! the exit, the pickups, the panels) and runs `place_props_tail` with the
//! test seams: `wall_adj_spot` / `alt_path_exists` are reference stand-ins for
//! the seam, and the builders either replay the recorded builder log (name, arguments, the draw
//! index at the call, the draws the real builder used, `top`) with stand-in
//! groups, or run core_props' real builders and check them against that log
//! (draws used, `top`). Then it compares
//! the draw count, the world subtree (transforms, geometry, materials,
//! textures, lights), interactables, animators and records, and replays the
//! recorded play session (animators, interactables, switch flips, timers) on a
//! second seeded stream, comparing snapshots, messages and the end state.
#[path = "json_common/mod.rs"]
mod common;
use common::*;
use engine::prelude::*;
use game::base::records::*;
use game::base::rng::*;
use game::core_props::seam::*;
use game::core_props::*;
use game::Game;
use std::cell::{Cell, RefCell};
use std::collections::{BTreeSet, HashSet, VecDeque};
use std::rc::Rc;

const TOL: f64 = 1e-9;
/// run-time snapshots: 12 significant digits in the fixture, platform sin/cos
const RTOL: f64 = 2e-9;

// ------------------------------------------------------------ reference stand-ins for the seam
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
        yaw: (-dxf).atan2(-dzf),
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
fn setup(case: &Json, real_builders: bool) -> Setup {
    game::assets::props::clear_prop_cache();
    let spec = case.get("spec");
    let mut g = Game::new_bare();
    g.w = case.get("W").i() as i32;
    g.h = case.get("H").i() as i32;
    g.tiles = case.get("tiles").arr().iter().map(|v| v.i() as u8).collect();
    g.level_weather = weather(spec.get("weather").s());
    // (the recorded levels are the rooms' depths: the rooms now begin on level 4)
    g.level_num = spec.get("level").i() as i32 + game::level::building::BUILDING_FLOORS;
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
    let log: Rc<RefCell<VecDeque<Logged>>> = Rc::new(RefCell::new(
        case.get("builders")
            .arr()
            .iter()
            // the old framed window is not built here any more (`window_zone_nodes`)
            .filter(|b| b.get("name").s() != OLD_WINDOW)
            .map(|b| Logged {
                name: b.get("name").s().to_string(),
                args: b.get("args").fs(),
                at: b.get("at").i() as u64,
                used: b.get("used").i() as u64,
                top: b.get("top").f(),
            })
            .collect(),
    ));
    let base = Rc::new(Cell::new(0u64));
    let lore = case.get("lore");
    set_props_hooks(PropsHooks { wall_adj_spot: Some(wall_adj_spot_rec), ..PropsHooks::NONE });
    let mut seam = TailSeam {
        alt_path_exists: Some(Box::new(alt_path_rec)),
        build: None,
        lore: Some(TailLore {
            scp: lore.get("scp").arr().iter().map(|s| if s.is_null() { None } else { Some(s.s().to_string()) }).collect(),
            arcs: lore
                .get("arcs")
                .arr()
                .iter()
                .map(|a| (a.at(0).arr().iter().map(|i| i.i() as usize).collect(), a.at(1).i() as u32))
                .collect(),
        }),
    };
    {
        let (log2, base2) = (log.clone(), base.clone());
        seam.build = Some(Box::new(move |g: &mut Game, cx: &mut PropsCtx, b: &TailBuild| {
            let e = log2.borrow_mut().pop_front().unwrap_or_else(|| panic!("unexpected builder call {}", b.name()));
            assert_eq!(b.name(), e.name, "builder order");
            match b {
                TailBuild::BrokenMirror(yaw) => assert_close(*yaw, e.args[0], TOL, "build_broken_mirror(yaw)"),
                TailBuild::Elevator(f) => assert_eq!(*f as i64, e.args[0] as i64, "build_elevator(functional)"),
                _ => assert!(e.args.is_empty(), "{} takes no arguments", e.name),
            }
            let d0 = seeded_draws();
            assert_eq!(d0 - base2.get(), e.at, "draws before {}", e.name);
            let built = if real_builders {
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
            };
            built.g.set_name(&format!("Builder:{}", e.name));
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
        ref o => panic!("unexpected geometry {:?}", o),
    };
    v.sort_by(|a, b| a.0.cmp(b.0));
    v.into_iter().map(|(k, x)| (k.to_string(), x)).collect()
}
#[track_caller]
fn cmp_geo(geo: &Geo, j: &Json, path: &str) {
    assert_eq!(geo.type_name(), j.get("type").s(), "{path}: geometry type");
    let params = geo_params(&geo.params());
    let jp = j.get("params").arr();
    assert_eq!(params.len(), jp.len(), "{path}: geometry params {:?}", params);
    for (k, (name, v)) in params.iter().enumerate() {
        assert_eq!(name, jp[k].at(0).s(), "{path}: param name");
        assert_close(*v, jp[k].at(1).f(), TOL, &format!("{path}: param {name}"));
    }
    let r = geo.r();
    assert_eq!(r.position.len() / 3, j.get("n").i() as usize, "{path}: vertex count");
    assert_eq!(r.index.as_ref().map(|i| i.len()).unwrap_or(0), j.get("idx").i() as usize, "{path}: index count");
    let (mut s, mut sa) = (0.0f64, 0.0f64);
    for (i, &v) in r.position.iter().enumerate() {
        s += v as f64;
        sa += (v as f64).abs() * ((i % 7) + 1) as f64;
    }
    assert_close(s, j.get("s").f(), 1e-7, &format!("{path}: position sum"));
    assert_close(sa, j.get("sa").f(), 1e-7, &format!("{path}: weighted position sum"));
}
/// the texture size a recorded texture implies (canvas w×h; `handwritten_tex` 1024×220 / `text_tex` 256×128 defaults)
fn tex_size(j: &Json) -> (f64, f64) {
    match j.get("kind").s() {
        "canvas" => (j.get("w").f(), j.get("h").f()),
        "hand" => {
            let o = j.get("opts");
            (if o.has("w") { o.get("w").f() } else { 1024.0 }, if o.has("h") { o.get("h").f() } else { 220.0 })
        }
        "text" => {
            let o = j.get("opts");
            (if o.has("w") { o.get("w").f() } else { 256.0 }, if o.has("h") { o.get("h").f() } else { 128.0 })
        }
        k => panic!("texture kind {k}"),
    }
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
    let side = match r.side {
        Side::Front => 0,
        Side::Back => 1,
        Side::Double => 2,
    };
    assert_eq!(side, j.get("side").i(), "{path}: side");
    let bl = match r.blending {
        Blending::None => 0,
        Blending::Normal => 1,
        Blending::Additive => 2,
        Blending::Subtractive => 3,
        Blending::Multiply => 4,
    };
    assert_eq!(bl, j.get("bl").i(), "{path}: blending");
    let jm = j.get("map");
    match (&r.map, jm.is_null()) {
        (None, true) => {}
        (Some(t), false) => {
            let (w, h) = tex_size(jm);
            assert_eq!((t.width() as f64, t.height() as f64), (w, h), "{path}: map size ({})", jm.get("kind").s());
            let tt = t.r();
            assert_eq!(tt.wrap_s == Wrap::Repeat, jm.get("ws").b(), "{path}: map wrap_s");
            assert_eq!(tt.wrap_t == Wrap::Repeat, jm.get("wt").b(), "{path}: map wrap_t");
            assert_close(tt.repeat.x, jm.get("rep").at(0).f(), TOL, &format!("{path}: map repeat.x"));
            assert_close(tt.repeat.y, jm.get("rep").at(1).f(), TOL, &format!("{path}: map repeat.y"));
        }
        (a, _) => panic!("{path}: map presence got {} recorded {}", a.is_some(), !jm.is_null()),
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
/// Compare an object with its recorded run-time tree node.
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
        let geo = o.geo();
        if geo.vertex_count() == 625 {
            let sum: f64 = geo.r().position.iter().map(|&v| v as f64).sum();
            // the recorded sum adds the f32 positions left to right in f64, like this
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
        (a, _) => panic!("{what}: pool_bounds got {:?}", a),
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
        assert_close(fv(&c.f, "hintCooldown"), jc.get("hint").f(), tol, &format!("{what}: candle {i} hintCooldown"));
        assert_eq!(c.b.get("wasNear").copied().unwrap_or(false), jc.get("near").b(), "{what}: candle {i} wasNear");
        assert_close(fv(&c.f, "fizzleStreak"), jc.get("streak").f(), tol, &format!("{what}: candle {i} fizzleStreak"));
        assert_close(fv(&c.f, "wickGlowT"), jc.get("wick").f(), tol, &format!("{what}: candle {i} wickGlowT"));
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

// ------------------------------------------------------------ the play session
fn use_interactable(g: &mut Game, i: usize) {
    let mut f = std::mem::replace(&mut g.interactables[i].on_use, Box::new(|_| {}));
    f(g);
    g.interactables[i].on_use = f;
}
/// The old window zone's builder: the windows are only chosen in the tail
/// now, on the tiles it took and with its draws (`core_props::window`), and
/// finished after the props settle, so its windows and their lights are not
/// in the world the tail builds.
const OLD_WINDOW: &str = "build_window";

/// The recorded world children that were the old window zone's: each window
/// (a builder node) and the point light hung before it
fn window_zone_nodes(case: &Json) -> HashSet<usize> {
    let tree = case.get("build").get("tree").arr();
    let mut out = HashSet::new();
    for (i, t) in tree.iter().enumerate() {
        if t.get("k").s() == "builder" && t.get("b").s() == OLD_WINDOW {
            out.insert(i);
            if tree.get(i + 1).is_some_and(|l| l.get("k").s() == "plight" && l.get("l").at(0).i() == 0x8090a8) {
                out.insert(i + 1);
            }
        }
    }
    out
}

/// A recorded list of world children without the old window zone's
fn without<'a>(jt: &'a [Json], drop: &HashSet<usize>) -> Vec<&'a Json> {
    jt.iter().enumerate().filter(|(i, _)| !drop.contains(i)).map(|(_, j)| j).collect()
}

fn run_case(case: &Json, real_builders: bool) {
    let drop = window_zone_nodes(case);
    let name = format!("{}{}", case.get("name").s(), if real_builders { " (real builders)" } else { "" });
    let spec = case.get("spec");
    let Setup { mut g, mut cx, base, log } = setup(case, real_builders);
    let h = case.get("handoff");
    let seed = spec.get("seed").i() as u32;
    with_seed(seed, || {
        for _ in 0..h.get("draws").i() {
            rnd();
        }
        base.set(seeded_draws());
        g.place_props_tail(&mut cx, &[]);
        assert_eq!(seeded_draws() - base.get(), case.get("tailDraws").i() as u64, "{name}: draws of the second half");
    });
    assert!(log.borrow().is_empty(), "{name}: builder calls left: {:?}", log.borrow());
    // ---- the build
    let b = case.get("build");
    let kids = g.world.children();
    let jt = without(b.get("tree").arr(), &drop);
    assert_eq!(kids.len(), jt.len(), "{name}: world children");
    for (i, o) in kids.iter().enumerate() {
        cmp_tree(o, jt[i], &format!("{name}: world/{i}"));
    }
    cmp_interactables(&g, b.get("interactables"), TOL, true, &format!("{name} build"));
    assert_eq!(g.animators.len() as i64, b.get("animators").i(), "{name}: animators");
    cmp_records(&g, b.get("records"), TOL, &format!("{name} build"));
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
    let jframes = case.get("sim").get("frames").arr();
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
            g.player.pos = Vec3::new(x1 + (k * 6.283).sin() * w * 1.2, 0.0, z1 + (1.0 - (k * 6.283).cos()) * hh * 0.9);
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
                let rec = &jframes[snap_i];
                assert_eq!(rec.get("f").i(), f);
                let st = rec.get("state");
                let what = format!("{name} frame {f}");
                let kids = g.world.children();
                let jt = without(st.get("tree").arr(), &drop);
                assert_eq!(kids.len(), jt.len(), "{what}: world children");
                for (i, o) in kids.iter().enumerate() {
                    cmp_rtree(o, jt[i], &format!("{what}: world/{i}"));
                }
                cmp_records(&g, st.get("records"), RTOL, &what);
                cmp_interactables(&g, st.get("interactables"), RTOL, false, &what);
                snap_i += 1;
            }
        }
    });
    assert_eq!(snap_i, snaps.len(), "{name}: snapshots compared");
    let sim_j = case.get("sim");
    assert_eq!(seeded_draws() - d0, sim_j.get("draws").i() as u64, "{name}: run-time draws");
    let jm = sim_j.get("msgs").arr();
    assert_eq!(msgs.len(), jm.len(), "{name}: messages {:?}", msgs);
    for (i, m) in msgs.iter().enumerate() {
        let e = &jm[i];
        assert_eq!((m.0, m.1, m.2.as_str()), (e.at(0).i(), e.at(1).s(), e.at(2).s()), "{name}: message {i}");
        assert_close(m.3, e.at(3).f(), 0.0, &format!("{name}: message {i} duration"));
    }
    // ---- the end state
    let fin = case.get("final");
    let what = format!("{name} final");
    let kids = g.world.children();
    let jt = without(fin.get("tree").arr(), &drop);
    assert_eq!(kids.len(), jt.len(), "{what}: world children");
    for (i, o) in kids.iter().enumerate() {
        cmp_rtree(o, jt[i], &format!("{what}: world/{i}"));
    }
    let added: Vec<Obj> = kids.iter().filter(|o| !build_set.contains(*o)).cloned().collect();
    let ja = fin.get("added").arr();
    assert_eq!(added.len(), ja.len(), "{what}: objects added by the session");
    for (i, o) in added.iter().enumerate() {
        cmp_tree(o, &ja[i], &format!("{what}: added/{i}"));
    }
    cmp_records(&g, fin.get("records"), RTOL, &what);
    let off: Vec<i64> = g.panels.iter().map(|p| p.switched_off as i64).collect();
    assert_eq!(off, fin.get("panelsOff").arr().iter().map(|v| v.i()).collect::<Vec<_>>(), "{what}: panels switched off");
    let fp = case.get("finalPlayer");
    assert_close(g.player.sanity, fp.get("sanity").f(), TOL, "sanity");
    assert_close(g.player.battery, fp.get("battery").f(), TOL, "battery");
    let fg = case.get("finalGlobals");
    assert_close(g.shake_t, fg.get("shakeT").f(), 0.0, "shakeT");
    assert_close(g.camera_dip_t, fg.get("cameraDipT").f(), 0.0, "cameraDipT");
    let tl = fg.get("tallies");
    assert_eq!(
        [g.candle_blowout_tally, g.candle_relight_tally, g.candle_fizzle_tally],
        [tl.at(0).f(), tl.at(1).f(), tl.at(2).f()],
        "{what}: candle tallies"
    );
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
    // the working elevator leaves the level (flow.rs)
    if let Some(i) = g.interactables.iter().position(|it| it.prompt == "[E] CALL THE ELEVATOR") {
        assert!((g.interactables[i].enabled)(&g));
        use_interactable(&mut g, i);
        assert_eq!(g.state, "transition", "{name}: elevator → next_level_via_elevator");
    }
    set_tail_seam(None);
    set_props_hooks(PropsHooks::NONE);
}

/// The second half alone: stand-in builders replaying the recorded builder log.
#[test]
fn tail_matches_reference() {
    let fx = Json::load("cp2_tail.json");
    let cases = fx.get("cases").arr();
    assert!(cases.len() >= 20);
    for c in cases {
        run_case(c, false);
    }
}
/// The same with core_props' real builders (their draws and `top` must match the recorded log).
#[test]
fn tail_matches_reference_with_real_builders() {
    let fx = Json::load("cp2_tail.json");
    for c in fx.get("cases").arr() {
        run_case(c, true);
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

/// The lore tables the second half reads (through the seam in the other
/// tests) agree with the recorded tables: `LORE_ENTRIES[i].scp`, `lore_arc_for`, the arc
/// colours, `lore_note_boosted`. Skipped while hud.rs's tables are stubs.
#[test]
fn lore_tables_match_reference() {
    let g0 = Game::new_bare();
    if g0.lore_entry_count() == 0 {
        eprintln!("LORE_ENTRIES empty here: skipped");
        return;
    }
    let fx = Json::load("cp2_tail.json");
    let lore = fx.get("cases").at(0).get("lore");
    let scp = lore.get("scp").arr();
    // the recorded notes stand as they were; those after them are the
    // Building's, added with its floors (`docs/MEGASTRUCTURE.md`, B6)
    assert!(g0.lore_entry_count() >= scp.len());
    for (i, s) in scp.iter().enumerate() {
        assert_eq!(g0.lore_entry_scp(i), if s.is_null() { None } else { Some(s.s().to_string()) }, "LORE_ENTRIES[{i}].scp");
    }
    for i in scp.len()..g0.lore_entry_count() {
        assert_eq!(g0.lore_entry_scp(i).as_deref(), Some("building"), "LORE_ENTRIES[{i}] is the Building's");
    }
    let arcs = lore.get("arcs").arr();
    for (a, arc) in arcs.iter().enumerate() {
        assert_eq!(g0.lore_arc_color(a) as i64, arc.at(1).i(), "LORE_ARCS[{a}].color");
    }
    for found in [vec![], vec![7usize, 3], vec![36, 2, 9, 14]] {
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
