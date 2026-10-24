//! Differential tests of `place_props`' first half (`game::core_props`, its
//! builders) against recorded reference data (fixtures in `tests/fixtures/`).
//!
//! Per synthetic level: the seeded draw count at the hand-off to the second
//! half, the shared locals, the whole world tree (kinds, transforms, geometry
//! parameters and vertex sums, materials, textures, lights, shared-resource
//! identity), interactables, animators, the gameplay records; then a scripted
//! run of the animators and a use of every interactable.
use engine::material::{Blending, MatKind, Side};
use engine::scene::Kind;
use engine::texture::Wrap;
use game::base::records::*;
use game::core_props::*;
use game::prelude::*;
use serde_json::Value;

fn load() -> Vec<Value> {
    let p = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/core_props_head.json");
    let s = std::fs::read_to_string(p).unwrap_or_else(|e| panic!("{}: {} (missing fixture)", p, e));
    match serde_json::from_str::<Value>(&s).unwrap() {
        Value::Array(a) => a,
        _ => panic!("fixture: not an array"),
    }
}
fn f(v: &Value) -> f64 {
    v.as_f64().unwrap_or_else(|| panic!("not a number: {}", v))
}
fn fs(v: &Value) -> Vec<f64> {
    v.as_array().unwrap().iter().map(f).collect()
}
fn close(a: f64, b: f64, tol: f64) -> bool {
    if a.is_nan() || b.is_nan() {
        return a.is_nan() && b.is_nan();
    }
    (a - b).abs() <= tol * 1f64.max(a.abs()).max(b.abs())
}
#[track_caller]
fn ck(a: f64, b: &Value, tol: f64, what: &str) {
    let b = f(b);
    assert!(close(a, b, tol), "{}: got {} vs recorded {} (diff {:e})", what, a, b, (a - b).abs());
}
#[track_caller]
fn ckv(a: &[f64], b: &Value, tol: f64, what: &str) {
    let b = fs(b);
    assert_eq!(a.len(), b.len(), "{}: length", what);
    for i in 0..a.len() {
        assert!(close(a[i], b[i], tol), "{}[{}]: got {} vs recorded {} ({:?} vs {:?})", what, i, a[i], b[i], a, b);
    }
}
fn v3a(v: &Vec3) -> [f64; 3] {
    [v.x, v.y, v.z]
}

// ---------------------------------------------------------------- reference stand-ins for the seam
thread_local! { static TO_STAIRS: std::cell::Cell<bool> = const { std::cell::Cell::new(false) }; }
/// `wall_adj_spot`, with signed zeros kept on purpose (`-dx` of 0 is -0)
fn rec_wall_adj_spot(g: &Game, x: i32, z: i32) -> Option<WallSpot> {
    let dirs: Vec<(i32, i32)> = [(1, 0), (-1, 0), (0, 1), (0, -1)].into_iter().filter(|&(dx, dz)| g.is_wall_tile(x + dx, z + dz)).collect();
    if dirs.is_empty() {
        return None;
    }
    let (dx, dz) = choice(&dirs);
    let (dx, dz) = (dx as f64, dz as f64);
    Some(WallSpot {
        pos: Vec3::new(g.wx(x) + dx * (CELL / 2.0 - 0.06), 0.0, g.wz(z) + dz * (CELL / 2.0 - 0.06)),
        normal: Vec3::new(-dx, 0.0, -dz),
        yaw: (-dx).atan2(-dz),
    })
}
/// `circle_hits_wall` without solids
fn rec_circle_hits_wall(g: &Game, px: f64, pz: f64, r: f64) -> bool {
    let (x0, x1, z0, z1) = (g.tx(px - r), g.tx(px + r), g.tz(pz - r), g.tz(pz + r));
    for z in z0..=z1 {
        for x in x0..=x1 {
            let blocked = g.is_wall_tile(x, z) || (g.in_grid(x, z) && g.barricade_tiles.contains(&g.t_idx(x, z)));
            if !blocked {
                continue;
            }
            let cx = clamp(px, g.wx(x) - CELL / 2.0, g.wx(x) + CELL / 2.0);
            let cz = clamp(pz, g.wz(z) - CELL / 2.0, g.wz(z) + CELL / 2.0);
            if (px - cx).powi(2) + (pz - cz).powi(2) < r * r {
                return true;
            }
        }
    }
    false
}
fn rec_level_has_stairwell(_g: &Game, _n: i32) -> bool {
    TO_STAIRS.with(|c| c.get())
}
fn install_hooks(to_stairs: bool) {
    TO_STAIRS.with(|c| c.set(to_stairs));
    set_props_hooks(PropsHooks {
        wall_adj_spot: Some(rec_wall_adj_spot),
        level_has_stairwell: Some(rec_level_has_stairwell),
        circle_hits_wall: Some(rec_circle_hits_wall),
    });
}

/// A bare game on the fixture's level.
fn setup(c: &Value) -> (Game, Vec<i32>) {
    let cs = &c["case"];
    install_hooks(cs["toStairs"].as_bool().unwrap());
    game::assets::props::clear_prop_cache();
    let mut g = Game::new_bare();
    g.w = c["W"].as_i64().unwrap() as i32;
    g.h = c["H"].as_i64().unwrap() as i32;
    g.tiles = c["tiles"].as_array().unwrap().iter().map(|v| v.as_u64().unwrap() as u8).collect();
    let dist: Vec<i32> = c["dist"].as_array().unwrap().iter().map(|v| v.as_i64().unwrap() as i32).collect();
    g.level_num = cs["levelNum"].as_i64().unwrap() as i32;
    g.level_scp = cs["levelScp"].as_str().map(|s| s.to_string());
    for (k, st, lt) in [("smiler", "wander", 3.0), ("watcher", "wander", 2.0), ("hound", "hunt", 5.0)] {
        let mut e = Entity::new(k, group());
        e.state = st.into();
        e.lose_t = lt;
        g.entities.push(e);
    }
    (g, dist)
}

// ---------------------------------------------------------------- tree comparison
#[derive(Default)]
struct Ids {
    geos: Vec<Geo>,
    mats: Vec<Mat>,
}
impl Ids {
    fn geo(&mut self, g: &Geo) -> usize {
        if let Some(i) = self.geos.iter().position(|x| x.ptr_eq(g)) {
            i
        } else {
            self.geos.push(g.clone());
            self.geos.len() - 1
        }
    }
    fn mat(&mut self, m: &Mat) -> usize {
        if let Some(i) = self.mats.iter().position(|x| x.ptr_eq(m)) {
            i
        } else {
            self.mats.push(m.clone());
            self.mats.len() - 1
        }
    }
}
fn geo_params(g: &Geo) -> Vec<f64> {
    use engine::geometry::GeoParams as P;
    let b = |v: bool| if v { 1.0 } else { 0.0 };
    match g.params() {
        P::Box { width, height, depth, ws, hs, ds } => vec![width, height, depth, ws as f64, hs as f64, ds as f64],
        P::Cone { radius, height, radial, height_seg, open_ended, theta_start, theta_length } => {
            vec![radius, height, radial as f64, height_seg as f64, b(open_ended), theta_start, theta_length]
        }
        P::Cylinder { radius_top, radius_bottom, height, radial, height_seg, open_ended, theta_start, theta_length } => {
            vec![radius_top, radius_bottom, height, radial as f64, height_seg as f64, b(open_ended), theta_start, theta_length]
        }
        P::Sphere { radius, ws, hs, phi_start, phi_length, theta_start, theta_length } => {
            vec![radius, ws as f64, hs as f64, phi_start, phi_length, theta_start, theta_length]
        }
        P::Plane { width, height, ws, hs } => vec![width, height, ws as f64, hs as f64],
        P::Circle { radius, segments, theta_start, theta_length } => vec![radius, segments as f64, theta_start, theta_length],
        P::Torus { radius, tube, radial, tubular, arc } => vec![radius, tube, radial as f64, tubular as f64, arc],
        P::Capsule { radius, length, cap_segments, radial } => vec![radius, length, cap_segments as f64, radial as f64],
        P::Shape { curve_segments } => vec![curve_segments as f64],
        _ => vec![],
    }
}
fn geo_kind(g: &Geo) -> &'static str {
    use engine::geometry::GeoParams as P;
    match g.params() {
        P::Box { .. } => "box",
        P::Cone { .. } => "cone",
        P::Cylinder { .. } => "cyl",
        P::Sphere { .. } => "sphere",
        P::Plane { .. } => "plane",
        P::Circle { .. } => "circle",
        P::Torus { .. } => "torus",
        P::Capsule { .. } => "capsule",
        P::Shape { .. } => "shape",
        _ => "buffer",
    }
}
fn attr_sum(a: &[f32]) -> [f64; 3] {
    let (mut t, mut ta) = (0.0f64, 0.0f64);
    for (i, &v) in a.iter().enumerate() {
        t += v as f64;
        ta += (v as f64).abs() * ((i % 7) + 1) as f64;
    }
    [a.len() as f64, t, ta]
}
fn obj_kind(o: &Obj) -> &'static str {
    if o.is_point_light() {
        return "point";
    }
    if matches!(o.r().kind, Kind::Line { .. }) {
        return "line";
    }
    if o.is_mesh() {
        return "mesh";
    }
    if o.is_group() {
        return "group";
    }
    "other"
}
#[track_caller]
fn cmp_tex(g: &Game, t: Option<Tex>, rec: &Value, what: &str) {
    if rec.is_null() {
        assert!(t.is_none(), "{}: unexpected map", what);
        return;
    }
    let t = t.unwrap_or_else(|| panic!("{}: map missing", what));
    let (ws, wt, rp) = {
        let r = t.r();
        (r.wrap_s == Wrap::Repeat, r.wrap_t == Wrap::Repeat, [r.repeat.x, r.repeat.y])
    };
    match rec["tag"].as_str().unwrap() {
        "static" => assert!(t.0.as_ptr() == g.tex.static_tex.0.as_ptr(), "{}: not static_tex", what),
        "rain" => {
            assert!(t.0.as_ptr() != g.rain_streak_tex.0.as_ptr(), "{}: rain_streak_tex itself, not a clone", what);
            assert_eq!((t.width(), t.height()), (g.rain_streak_tex.width(), g.rain_streak_tex.height()), "{}: rain clone size", what);
        }
        _ => {
            assert_eq!(t.width() as f64, f(&rec["w"]), "{}: tex width", what);
            assert_eq!(t.height() as f64, f(&rec["h"]), "{}: tex height", what);
        }
    }
    assert_eq!(ws, rec["ws"].as_bool().unwrap(), "{}: wrap_s", what);
    assert_eq!(wt, rec["wt"].as_bool().unwrap(), "{}: wrap_t", what);
    ckv(&rp, &rec["rp"], 0.0, &format!("{}: repeat", what));
}
#[track_caller]
fn cmp_tree(g: &Game, o: &Obj, rec: &Value, path: &str, ids: &mut Ids) {
    assert_eq!(obj_kind(o), rec["k"].as_str().unwrap(), "{}: kind", path);
    ckv(&v3a(&o.pos()), &rec["p"], 1e-12, &format!("{} position", path));
    let q = o.quat();
    ckv(&[q.x, q.y, q.z, q.w], &rec["q"], 1e-12, &format!("{} quaternion", path));
    let s = o.scl();
    if rec.get("s").is_some() {
        ckv(&v3a(&s), &rec["s"], 1e-12, &format!("{} scale", path));
    } else {
        assert_eq!(v3a(&s), [1.0, 1.0, 1.0], "{} scale", path);
    }
    assert_eq!(o.visible(), rec.get("v").and_then(|v| v.as_bool()).unwrap_or(true), "{}: visible", path);
    let order = format!("{:?}", o.rot().order);
    assert_eq!(order, rec.get("ro").and_then(|v| v.as_str()).unwrap_or("XYZ"), "{}: rotation order", path);
    assert_eq!(o.r().user.gameplay, rec.get("gp").is_some(), "{}: user.gameplay", path);
    assert_eq!(o.r().user.floats, rec.get("fl").is_some(), "{}: user.floats", path);
    if let Some(jg) = rec.get("g") {
        let geo = o.geo();
        assert_eq!(ids.geo(&geo) as f64, f(&jg["gi"]), "{}: geometry sharing", path);
        assert_eq!(geo.r().is_prop_part, jg["part"].as_bool().unwrap(), "{}: prop part", path);
        let jt = jg["t"].as_array().unwrap();
        assert_eq!(geo_kind(&geo), jt[0].as_str().unwrap(), "{}: geometry type", path);
        let jp: Vec<f64> = jt[1..].iter().map(f).collect();
        let rp = geo_params(&geo);
        assert_eq!(rp.len(), jp.len(), "{}: params", path);
        for i in 0..rp.len() {
            assert!(close(rp[i], jp[i], 1e-12), "{}: param {} got {:?} recorded {:?}", path, i, rp, jp);
        }
        assert_eq!(geo.vertex_count() as f64, f(&jg["vc"]), "{}: vertex count", path);
        let ic = geo.r().index.as_ref().map(|i| i.len() as f64).unwrap_or(-1.0);
        assert_eq!(ic, f(&jg["ic"]), "{}: index count", path);
        let ps = attr_sum(&geo.r().position);
        ckv(&ps, &jg["pos"], 1e-9, &format!("{} position sums", path));
        let nr = geo.r().normal.clone();
        if jg["nor"].is_null() {
            assert!(nr.is_empty(), "{}: unexpected normals", path);
        } else {
            ckv(&attr_sum(&nr), &jg["nor"], 1e-9, &format!("{} normal sums", path));
        }
        let jm = &rec["m"];
        let m = o.mat();
        assert_eq!(ids.mat(&m) as f64, f(&jm["mi"]), "{}: material sharing", path);
        let r = m.r();
        let ty = match r.kind {
            MatKind::Basic => "basic",
            MatKind::Lambert => "lambert",
            MatKind::LineBasic => "line",
            _ => "other",
        };
        assert_eq!(ty, jm["t"].as_str().unwrap(), "{}: material type", path);
        assert_eq!(r.color.get_hex() as f64, f(&jm["c"]), "{}: color", path);
        if ty == "lambert" {
            assert_eq!(r.emissive.get_hex() as f64, f(&jm["e"]), "{}: emissive", path);
        }
        ck(r.opacity, &jm["o"], 1e-12, &format!("{}: opacity", path));
        assert_eq!(r.transparent, jm["tr"].as_bool().unwrap(), "{}: transparent", path);
        let side = match r.side {
            Side::Front => 0.0,
            Side::Back => 1.0,
            Side::Double => 2.0,
        };
        assert_eq!(side, f(&jm["sd"]), "{}: side", path);
        assert_eq!(r.depth_write, jm["dw"].as_bool().unwrap(), "{}: depth_write", path);
        let bl = match r.blending {
            Blending::None => 0.0,
            Blending::Normal => 1.0,
            Blending::Additive => 2.0,
            Blending::Subtractive => 3.0,
            Blending::Multiply => 4.0,
        };
        assert_eq!(bl, f(&jm["bl"]), "{}: blending", path);
        let map = r.map.clone();
        drop(r);
        cmp_tex(g, map, &jm["map"], &format!("{} map", path));
    } else {
        assert!(o.try_geo().is_none() || o.is_light(), "{}: we have geometry, recorded none", path);
    }
    if let Some(jl) = rec.get("l") {
        let (c, i, d, dc) = {
            let l = o.light();
            (l.color.get_hex(), l.intensity, l.distance, l.decay)
        };
        assert_eq!(c as f64, f(&jl["c"]), "{}: light color", path);
        ck(i, &jl["i"], 1e-12, &format!("{}: light intensity", path));
        ck(d, &jl["d"], 1e-12, &format!("{}: light distance", path));
        ck(dc, &jl["dc"], 1e-12, &format!("{}: light decay", path));
    }
    let kids = o.children();
    let jk = rec.get("c").and_then(|c| c.as_array()).map(|a| a.as_slice()).unwrap_or(&[]);
    assert_eq!(kids.len(), jk.len(), "{}: child count", path);
    for (i, (k, j)) in kids.iter().zip(jk.iter()).enumerate() {
        cmp_tree(g, k, j, &format!("{}/{}", path, i), ids);
    }
}
fn cand_arr(c: &Cand) -> [f64; 3] {
    [c.x as f64, c.z as f64, c.d as f64]
}
#[track_caller]
fn cmp_pickups(a: &[Pickup], rec: &Value, what: &str) {
    let ja = rec.as_array().unwrap();
    assert_eq!(a.len(), ja.len(), "{}: count", what);
    for (i, (p, j)) in a.iter().zip(ja.iter()).enumerate() {
        ckv(&v3a(&p.g.pos()), &j["p"], 1e-12, &format!("{}[{}] pos", what, i));
        ck(p.base, &j["base"], 1e-12, &format!("{}[{}] base", what, i));
        ck(p.phase, &j["phase"], 1e-12, &format!("{}[{}] phase", what, i));
        assert_eq!(p.taken, j["taken"].as_bool().unwrap(), "{}[{}] taken", what, i);
    }
}

/// `place_props`' first half on every fixture level: draws, locals, tree, records.
#[test]
fn place_props_head_matches_reference() {
    for c in load() {
        let seed = c["case"]["seed"].as_u64().unwrap() as u32;
        let tag = format!("case {}", seed);
        let (mut g, dist) = setup(&c);
        let (draws, cx) = with_seed(seed, || {
            let d0 = seeded_draws();
            let cx = g.place_props_head(&dist);
            (seeded_draws() - d0, cx)
        });
        assert_eq!(draws as f64, f(&c["draws"]), "{}: seeded draws at the hand-off", tag);
        // the shared locals
        let h = &c["handoff"];
        let jc = h["cands"].as_array().unwrap();
        assert_eq!(cx.cands.len(), jc.len(), "{}: cands", tag);
        for (a, b) in cx.cands.iter().zip(jc) {
            ckv(&cand_arr(a), b, 0.0, &format!("{}: cand", tag));
        }
        let jt = h["taken"].as_array().unwrap();
        assert_eq!(cx.taken.len(), jt.len(), "{}: taken", tag);
        for (a, b) in cx.taken.iter().zip(jt) {
            ckv(&cand_arr(a), b, 0.0, &format!("{}: taken", tag));
        }
        ckv(&cand_arr(&cx.exit_tile.unwrap()), &h["exitTile"], 0.0, &format!("{}: exitTile", tag));
        // the world
        cmp_tree(&g, &g.world, &c["tree"], &format!("{} world", tag), &mut Ids::default());
        assert_eq!(g.world.child_count() as f64, f(&c["worldChildren"]), "{}: world children", tag);
        // interactables, animators
        let ji = c["interactables"].as_array().unwrap();
        assert_eq!(g.interactables.len(), ji.len(), "{}: interactables", tag);
        for (i, (it, j)) in g.interactables.iter().zip(ji).enumerate() {
            ckv(&v3a(&it.pos), &j["p"], 1e-12, &format!("{}: interactable {} pos", tag, i));
            ck(it.r, &j["r"], 0.0, &format!("{}: interactable {} r", tag, i));
            assert_eq!(it.prompt, j["prompt"].as_str().unwrap(), "{}: interactable {} prompt", tag, i);
            assert_eq!((it.enabled)(&g), j["en"].as_bool().unwrap(), "{}: interactable {} enabled", tag, i);
        }
        assert_eq!(g.animators.len() as f64, f(&c["animators"]), "{}: animators", tag);
        // records
        match (&g.exit_data, c["exitData"].is_null()) {
            (Some(e), false) => {
                ckv(&v3a(&e.pos), &c["exitData"]["pos"], 1e-12, &format!("{}: exitData.pos", tag));
                ckv(&v3a(&e.normal), &c["exitData"]["normal"], 0.0, &format!("{}: exitData.normal", tag));
            }
            (None, true) => {}
            _ => panic!("{}: exit_data presence", tag),
        }
        match (&g.tv_data, c["tvData"].is_null()) {
            (Some(t), false) => {
                let j = &c["tvData"];
                ckv(&v3a(&t.world), &j["world"], 1e-12, &format!("{}: tvData.world", tag));
                ckv(&v3a(&t.normal), &j["normal"], 0.0, &format!("{}: tvData.normal", tag));
                ck(t.acc, &j["acc"], 0.0, &format!("{}: tvData.acc", tag));
                ck(t.light.intensity(), &j["light"], 0.0, &format!("{}: tvData.light", tag));
                assert!(t.light.parent().is_some(), "{}: the TV light hangs on the set", tag);
            }
            (None, true) => {}
            _ => panic!("{}: tv_data presence", tag),
        }
        match (&g.clock_data, c["clockData"].is_null()) {
            (Some(k), false) => {
                let j = &c["clockData"];
                ckv(&v3a(&k.world), &j["world"], 1e-12, &format!("{}: clockData.world", tag));
                ckv(&v3a(&k.normal), &j["normal"], 0.0, &format!("{}: clockData.normal", tag));
                assert_eq!(k.tock, j["tock"].as_bool().unwrap());
                assert!(k.sec_pivot.parent().is_some(), "{}: second hand pivot on the clock", tag);
            }
            (None, true) => {}
            _ => panic!("{}: clock_data presence", tag),
        }
        match (&g.phone_data, c["phoneData"].is_null()) {
            (Some(p), false) => {
                let j = &c["phoneData"];
                ckv(&v3a(&p.world), &j["world"], 1e-12, &format!("{}: phoneData.world", tag));
                ckv(&v3a(&p.normal), &j["normal"], 0.0, &format!("{}: phoneData.normal", tag));
                assert_eq!((p.ringing, p.answered), (j["ringing"].as_bool().unwrap(), j["answered"].as_bool().unwrap()));
                ck(p.ring_t, &j["ringT"], 0.0, &format!("{}: ringT", tag));
                ckv(&v3a(&p.handset.as_ref().unwrap().pos()), &j["handset"], 1e-12, &format!("{}: handset", tag));
            }
            (None, true) => {}
            _ => panic!("{}: phone_data presence", tag),
        }
        match (&g.man_data, c["manData"].is_null()) {
            (Some(m), false) => {
                ckv(&v3a(&m.g.pos()), &c["manData"]["p"], 1e-12, &format!("{}: manData pos", tag));
                ck(m.f["timer"], &c["manData"]["timer"], 0.0, &format!("{}: manData timer", tag));
            }
            (None, true) => {}
            _ => panic!("{}: man_data presence", tag),
        }
        cmp_pickups(&g.almonds, &c["almonds"], &format!("{}: almonds", tag));
        cmp_pickups(&g.battery_items, &c["batteryItems"], &format!("{}: batteryItems", tag));
        let jd = c["drawerProps"].as_array().unwrap();
        assert_eq!(g.drawer_props.len(), jd.len(), "{}: drawer_props", tag);
        let kids = g.world.children();
        for (dp, j) in g.drawer_props.iter().zip(jd) {
            let idx = kids.iter().position(|k| k.ptr_eq(&dp.cabinet)).map(|i| i as f64).unwrap_or(-1.0);
            assert_eq!(idx, f(&j["cab"]), "{}: drawer cabinet", tag);
            let ct: Vec<String> = dp.drawers.iter().map(|d| d.borrow().content_type.clone()).collect();
            let jct: Vec<String> = j["drawers"].as_array().unwrap().iter().map(|v| v.as_str().unwrap().to_string()).collect();
            assert_eq!(ct, jct, "{}: drawer contents", tag);
        }
        assert_eq!(g.rain_windows.len() as f64, f(&c["rainWindows"]), "{}: rain_windows", tag);
    }
    set_props_hooks(PropsHooks::NONE);
}

/// The animators over a scripted walk, then every interactable used once.
#[test]
fn place_props_head_runtime_matches_reference() {
    for c in load() {
        let seed = c["case"]["seed"].as_u64().unwrap() as u32;
        let tag = format!("case {}", seed);
        let (mut g, dist) = setup(&c);
        with_seed(seed, || {
            g.place_props_head(&dist);
        });
        let world0 = g.world.child_count();
        with_seed(seed ^ 0x5eed, || {
            let d0 = seeded_draws();
            let man = g.man_data.as_ref().map(|m| m.g.clone());
            let phone_world = g.phone_data.as_ref().map(|p| p.world);
            let rec_sim = c["sim"].as_array().unwrap();
            let mut si = 0;
            for k in 0..720 {
                let dt = if k < 600 { 0.05 } else { 1.0 / 60.0 };
                let t = if k < 600 { 1.0 + k as f64 * 0.05 } else { 31.0 + (k - 600) as f64 / 60.0 };
                if let (Some(m), true) = (&man, k < 600) {
                    let a = k as f64 * 0.03;
                    let rr = 3.0 + (k % 90) as f64 / 10.0;
                    let mp = m.pos();
                    g.player.pos = Vec3::new(mp.x + a.cos() * rr, 1.62, mp.z + a.sin() * rr);
                    g.player.yaw = if (k / 40) % 2 == 1 { a } else { a + PI };
                } else if let Some(pw) = phone_world {
                    g.player.pos = Vec3::new(pw.x + 1.0, 1.62, pw.z + 1.0);
                }
                if k == 650 {
                    if let Some(p) = g.phone_data.as_mut() {
                        p.ringing = true;
                    }
                }
                g.perf_t = t;
                g.run_animators(dt, t);
                if k % 30 == 29 {
                    let j = &rec_sim[si];
                    si += 1;
                    let w = format!("{} frame {}", tag, k);
                    assert_eq!((seeded_draws() - d0) as f64, f(&j["draws"]), "{}: draws", w);
                    if let Some(m) = &g.man_data {
                        ckv(&v3a(&m.g.pos()), &j["man"]["p"], 1e-9, &format!("{} mannequin pos", w));
                        ck(m.g.rot().y, &j["man"]["ry"], 1e-9, &format!("{} mannequin yaw", w));
                        ck(m.f["timer"], &j["man"]["timer"], 1e-9, &format!("{} mannequin timer", w));
                    } else {
                        assert!(j["man"].is_null(), "{}: mannequin presence", w);
                    }
                    if let Some(p) = &g.phone_data {
                        ckv(&v3a(&p.handset.as_ref().unwrap().pos()), &j["handset"], 1e-9, &format!("{} handset", w));
                    }
                    let floats: Vec<[f64; 3]> =
                        g.world.children().iter().filter(|o| o.r().user.floats).map(|o| v3a(&o.child(0).unwrap().pos())).collect();
                    let jf = j["floats"].as_array().unwrap();
                    assert_eq!(floats.len(), jf.len(), "{}: floats", w);
                    for (a, b) in floats.iter().zip(jf) {
                        ckv(a, b, 1e-9, &format!("{} balloon", w));
                    }
                }
            }
            assert_eq!(si, rec_sim.len(), "{}: sim samples", tag);
            assert_eq!((seeded_draws() - d0) as f64, f(&c["simDraws"]), "{}: sim draws", tag);
            // every enabled interactable, in order
            g.perf_t = 50.0;
            let ju = c["uses"].as_array().unwrap();
            assert_eq!(g.interactables.len(), ju.len(), "{}: uses", tag);
            for i in 0..g.interactables.len() {
                let en = (g.interactables[i].enabled)(&g);
                let j = &ju[i];
                assert_eq!(en, !j.is_null(), "{}: interactable {} enabled before use", tag, i);
                if !en {
                    continue;
                }
                g.msg_text.clear();
                let mut fu = std::mem::replace(&mut g.interactables[i].on_use, Box::new(|_| {}));
                fu(&mut g);
                g.interactables[i].on_use = fu;
                let w = format!("{} use {} ({})", tag, i, g.interactables[i].prompt);
                assert_eq!((seeded_draws() - d0) as f64, f(&j["draws"]), "{}: draws", w);
                assert_eq!(g.msg_text, j["msg"].as_str().unwrap(), "{}: message", w);
                ck(g.player.sanity, &j["sanity"], 0.0, &format!("{}: sanity", w));
                assert_eq!(g.ui.flash.style("opacity").unwrap_or(""), j["flash"].as_str().unwrap(), "{}: flash", w);
            }
            let ja = &c["after"];
            assert_eq!((seeded_draws() - d0) as f64, f(&ja["draws"]), "{}: draws after the uses", tag);
        });
        let ja = &c["after"];
        cmp_pickups(&g.almonds, &ja["almonds"], &format!("{}: almonds after", tag));
        cmp_pickups(&g.battery_items, &ja["batteryItems"], &format!("{}: batteryItems after", tag));
        let jp = ja["papers"].as_array().unwrap();
        assert_eq!(g.papers.len(), jp.len(), "{}: papers", tag);
        for (p, j) in g.papers.iter().zip(jp) {
            ckv(&v3a(&p.g.pos()), &j["p"], 1e-12, &format!("{}: paper pos", tag));
            assert_eq!(p.taken, j["taken"].as_bool().unwrap());
            assert_eq!(p.g.child_count() as f64, f(&j["n"]), "{}: paper sheets", tag);
        }
        let jd = ja["drawers"].as_array().unwrap();
        for (dp, j) in g.drawer_props.iter().zip(jd) {
            for (d, jj) in dp.drawers.iter().zip(j.as_array().unwrap()) {
                let d = d.borrow();
                assert_eq!(d.opened, jj[0].as_bool().unwrap(), "{}: drawer opened", tag);
                ckv(&v3a(&d.mesh.pos()), &jj[1], 1e-12, &format!("{}: drawer slide", tag));
            }
        }
        assert_eq!(g.world.child_count() as f64, f(&ja["worldChildren"]), "{}: world children after", tag);
        let je = ja["entities"].as_array().unwrap();
        for (e, j) in g.entities.iter().zip(je) {
            assert_eq!(e.state, j[0].as_str().unwrap(), "{}: entity state", tag);
            ck(e.lose_t, &j[1], 0.0, &format!("{}: entity loseT", tag));
        }
        match (&g.phone_data, ja["phone"].is_null()) {
            (Some(p), false) => {
                let j = &ja["phone"];
                assert_eq!((p.answered, p.ringing), (j["answered"].as_bool().unwrap(), j["ringing"].as_bool().unwrap()), "{}: phone", tag);
                match (p.f.get("hintUntil"), j["hintUntil"].as_f64()) {
                    (Some(a), Some(b)) => assert_eq!(*a, b, "{}: hint_until", tag),
                    (None, None) => {}
                    (a, b) => panic!("{}: hint_until got {:?} recorded {:?}", tag, a, b),
                }
            }
            (None, true) => {}
            _ => panic!("{}: phone presence", tag),
        }
        let jt: Vec<f64> = fs(&ja["timeouts"]);
        let rt: Vec<f64> = g.timeouts.iter().map(|t| t.at_ms - g.now_ms).collect();
        assert_eq!(rt, jt, "{}: timeouts", tag);
        let jen = ja["enabled"].as_array().unwrap();
        for (i, (it, j)) in g.interactables.iter().zip(jen).enumerate() {
            assert_eq!((it.enabled)(&g), j.as_bool().unwrap(), "{}: enabled after, {}", tag, i);
        }
        let added = g.world.children()[world0..].to_vec();
        let jadd = ja["added"].as_array().unwrap();
        assert_eq!(added.len(), jadd.len(), "{}: added objects", tag);
        for (i, (o, j)) in added.iter().zip(jadd).enumerate() {
            cmp_tree(&g, o, j, &format!("{} added {}", tag, i), &mut Ids::default());
        }
        g.now_ms += 1000.0;
        g.run_scheduled();
        assert_eq!(g.ui.flash.style("opacity").unwrap_or(""), ja["flashAfterTimeouts"].as_str().unwrap(), "{}: flash after timeouts", tag);
    }
    set_props_hooks(PropsHooks::NONE);
}

/// The whole `place_props` (both halves, whatever state the second half is
/// in) on many synthetic levels, weathers and anomalies, then the
/// animators and every interactable: no panics, records consistent.
#[test]
fn place_props_smoke() {
    for seed in 0..10u32 {
        let (mut g, dist) = Game::test_level(seed * 7919 + 13, 21 + (seed as i32 % 4) * 8);
        set_props_hooks(PropsHooks {
            wall_adj_spot: Some(rec_wall_adj_spot),
            level_has_stairwell: Some(rec_level_has_stairwell),
            circle_hits_wall: Some(rec_circle_hits_wall),
        });
        TO_STAIRS.with(|c| c.set(seed % 3 == 0));
        g.level_scp = match seed % 4 {
            1 => Some("loop".into()),
            2 => Some("store".into()),
            _ => None,
        };
        game::assets::props::clear_prop_cache();
        with_seed(seed.wrapping_mul(2654435761), || g.place_props(&dist));
        assert!(g.exit_data.is_some(), "seed {}: no exit", seed);
        assert_eq!(g.drawer_props.iter().map(|d| d.drawers.len()).sum::<usize>() % 3, 0);
        if let Some(p) = g.phone_data.as_mut() {
            p.ringing = true;
        }
        for k in 0..200 {
            let t = k as f64 * 0.1;
            g.perf_t = t;
            g.player.pos = Vec3::new((t * 0.7).sin() * 20.0, 1.62, (t * 0.3).cos() * 20.0);
            g.player.yaw = t;
            g.run_animators(0.1, t);
        }
        g.use_all_interactables();
        g.use_all_interactables();
        for k in 0..20 {
            g.run_animators(0.25, 30.0 + k as f64);
        }
        g.now_ms += 1000.0;
        g.run_scheduled();
        g.world.update_matrix_world(true);
    }
    // the real (unhooked) `wall_adj_spot` / `level_has_stairwell` / `circle_hits_wall` paths
    set_props_hooks(PropsHooks::NONE);
    let (mut g, dist) = Game::test_level(4242, 29);
    with_seed(77, || g.place_props(&dist));
    g.use_all_interactables();
    g.run_animators(0.1, 1.0);
}

/// Every builder on its own (several seeds each), including the ones only
/// the second half calls: draws, `top`, the tree, drawers, rain overlays.
#[test]
fn builders_match_reference() {
    let p = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/core_props_builders.json");
    let s = std::fs::read_to_string(p).unwrap_or_else(|e| panic!("{}: {} (missing fixture)", p, e));
    let cases: Vec<Value> = serde_json::from_str(&s).unwrap();
    let mut g = Game::new_bare();
    let mut n = 0;
    for c in &cases {
        let name = c["name"].as_str().unwrap();
        let args = c["args"].as_array().unwrap();
        let seed = c["seed"].as_u64().unwrap() as u32;
        let tag = format!("{}({:?}) seed {}", name, args, seed);
        game::assets::props::clear_prop_cache();
        let rw = g.rain_windows.len();
        let mut cx = PropsCtx::default();
        let (draws, built) = with_seed(seed, || {
            let d0 = seeded_draws();
            let b = match name {
                "build_filing_cabinet" => build_filing_cabinet(&mut g, &mut cx),
                "build_cardboard_stack" => build_cardboard_stack(&mut g, &mut cx),
                "build_wet_floor_sign" => build_wet_floor_sign(&mut g, &mut cx),
                "build_vending" => build_vending(&mut g, &mut cx),
                "build_locker" => build_locker(&mut g, &mut cx),
                "build_fire_extinguisher" => build_fire_extinguisher(&mut g, &mut cx),
                "build_trash_can" => build_trash_can(&mut g, &mut cx),
                "build_rubber_duck" => build_rubber_duck(&mut g, &mut cx),
                "build_traffic_cone" => build_traffic_cone(&mut g, &mut cx),
                "build_wheelchair" => build_wheelchair(&mut g, &mut cx),
                "build_baby_doll" => build_baby_doll(&mut g, &mut cx),
                "build_suitcase" => build_suitcase(&mut g, &mut cx),
                "build_broken_mirror" => build_broken_mirror(&mut g, &mut cx, f(&args[0])),
                "build_tv_on_floor" => build_tv_on_floor(&mut g, &mut cx),
                "build_book_stack" => build_book_stack(&mut g, &mut cx),
                "build_coat_rack" => build_coat_rack(&mut g, &mut cx),
                "build_fluorescent_tube" => build_fluorescent_tube(&mut g, &mut cx),
                "build_pile" => build_pile(&mut g, &mut cx),
                "build_potted_plant" => build_potted_plant(&mut g, &mut cx),
                "build_floor_plant" => build_floor_plant(&mut g, &mut cx),
                "build_window" => build_window(&mut g, &mut cx),
                "build_elevator" => build_elevator(&mut g, &mut cx, args[0].as_bool().unwrap()),
                "build_paper_pile" => build_paper_pile(&mut g, &mut cx),
                "build_floor_hole" => build_floor_hole(&mut g, &mut cx),
                "make_leaf_shape" => Built::new(make_leaf_shape(f(&args[0]), f(&args[1]), f(&args[2]) as u32, f(&args[3])), 0.0),
                o => panic!("unknown builder {}", o),
            };
            (seeded_draws() - d0, b)
        });
        assert_eq!(draws as f64, f(&c["draws"]), "{}: draws", tag);
        ck(built.top, &c["top"], 0.0, &format!("{}: top", tag));
        cmp_tree(&g, &built.g, &c["tree"], &tag, &mut Ids::default());
        if c["drawers"].is_null() {
            assert!(built.drawers.is_empty(), "{}: unexpected drawers", tag);
        } else {
            let jd = c["drawers"].as_array().unwrap();
            assert_eq!(built.drawers.len(), jd.len(), "{}: drawers", tag);
            let kids = built.g.children();
            for (d, j) in built.drawers.iter().zip(jd) {
                let d = d.borrow();
                assert_eq!(d.content_type, j[0].as_str().unwrap(), "{}: drawer content", tag);
                assert!(!d.opened && !d.content_taken);
                let idx = kids.iter().position(|k| k.ptr_eq(&d.mesh)).map(|i| i as f64).unwrap_or(-1.0);
                assert_eq!(idx, f(&j[2]), "{}: drawer group index", tag);
            }
        }
        assert_eq!((g.rain_windows.len() - rw) as f64, f(&c["rain"]), "{}: rain_windows", tag);
        if c["rain"].as_i64().unwrap() > 0 {
            let last = g.rain_windows.last().unwrap();
            let idx = built.g.children().iter().position(|k| k.ptr_eq(last)).map(|i| i as f64).unwrap_or(-1.0);
            assert_eq!(idx, f(&c["rainIdx"]), "{}: rain overlay", tag);
        }
        n += 1;
    }
    assert_eq!(n, 153);
}
