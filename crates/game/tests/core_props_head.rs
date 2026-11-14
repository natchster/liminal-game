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
    let s = std::fs::read_to_string(p).unwrap_or_else(|e| panic!("{p}: {e} (missing fixture)"));
    match serde_json::from_str::<Value>(&s).unwrap() {
        Value::Array(a) => a,
        _ => panic!("fixture: not an array"),
    }
}
fn f(v: &Value) -> f64 {
    v.as_f64().unwrap_or_else(|| panic!("not a number: {v}"))
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
    assert_eq!(a.len(), b.len(), "{what}: length");
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
        assert!(t.is_none(), "{what}: unexpected map");
        return;
    }
    let t = t.unwrap_or_else(|| panic!("{what}: map missing"));
    let (ws, wt, rp) = {
        let r = t.r();
        (r.wrap_s == Wrap::Repeat, r.wrap_t == Wrap::Repeat, [r.repeat.x, r.repeat.y])
    };
    match rec["tag"].as_str().unwrap() {
        "static" => assert!(t.0.as_ptr() == g.tex.static_tex.0.as_ptr(), "{what}: not static_tex"),
        "rain" => {
            assert!(t.0.as_ptr() != g.rain_streak_tex.0.as_ptr(), "{what}: rain_streak_tex itself, not a clone");
            assert_eq!((t.width(), t.height()), (g.rain_streak_tex.width(), g.rain_streak_tex.height()), "{what}: rain clone size");
        }
        _ => {
            assert_eq!(t.width() as f64, f(&rec["w"]), "{what}: tex width");
            assert_eq!(t.height() as f64, f(&rec["h"]), "{what}: tex height");
        }
    }
    assert_eq!(ws, rec["ws"].as_bool().unwrap(), "{what}: wrap_s");
    assert_eq!(wt, rec["wt"].as_bool().unwrap(), "{what}: wrap_t");
    ckv(&rp, &rec["rp"], 0.0, &format!("{what}: repeat"));
}
#[track_caller]
fn cmp_tree(g: &Game, o: &Obj, rec: &Value, path: &str, ids: &mut Ids) {
    assert_eq!(obj_kind(o), rec["k"].as_str().unwrap(), "{path}: kind");
    ckv(&v3a(&o.pos()), &rec["p"], 1e-12, &format!("{path} position"));
    let q = o.quat();
    ckv(&[q.x, q.y, q.z, q.w], &rec["q"], 1e-12, &format!("{path} quaternion"));
    let s = o.scl();
    if rec.get("s").is_some() {
        ckv(&v3a(&s), &rec["s"], 1e-12, &format!("{path} scale"));
    } else {
        assert_eq!(v3a(&s), [1.0, 1.0, 1.0], "{path} scale");
    }
    assert_eq!(o.visible(), rec.get("v").and_then(|v| v.as_bool()).unwrap_or(true), "{path}: visible");
    let order = format!("{:?}", o.rot().order);
    assert_eq!(order, rec.get("ro").and_then(|v| v.as_str()).unwrap_or("XYZ"), "{path}: rotation order");
    assert_eq!(o.r().user.gameplay, rec.get("gp").is_some(), "{path}: user.gameplay");
    assert_eq!(o.r().user.floats, rec.get("fl").is_some(), "{path}: user.floats");
    if let Some(jg) = rec.get("g") {
        let geo = o.geo();
        assert_eq!(ids.geo(&geo) as f64, f(&jg["gi"]), "{path}: geometry sharing");
        assert_eq!(geo.r().is_prop_part, jg["part"].as_bool().unwrap(), "{path}: prop part");
        let jt = jg["t"].as_array().unwrap();
        assert_eq!(geo_kind(&geo), jt[0].as_str().unwrap(), "{path}: geometry type");
        let jp: Vec<f64> = jt[1..].iter().map(f).collect();
        let rp = geo_params(&geo);
        assert_eq!(rp.len(), jp.len(), "{path}: params");
        for i in 0..rp.len() {
            assert!(close(rp[i], jp[i], 1e-12), "{path}: param {i} got {rp:?} recorded {jp:?}");
        }
        assert_eq!(geo.vertex_count() as f64, f(&jg["vc"]), "{path}: vertex count");
        let ic = geo.r().index.as_ref().map_or(-1.0, |i| i.len() as f64);
        assert_eq!(ic, f(&jg["ic"]), "{path}: index count");
        let ps = attr_sum(&geo.r().position);
        ckv(&ps, &jg["pos"], 1e-9, &format!("{path} position sums"));
        let nr = geo.r().normal.clone();
        if jg["nor"].is_null() {
            assert!(nr.is_empty(), "{path}: unexpected normals");
        } else {
            ckv(&attr_sum(&nr), &jg["nor"], 1e-9, &format!("{path} normal sums"));
        }
        let jm = &rec["m"];
        let m = o.mat();
        assert_eq!(ids.mat(&m) as f64, f(&jm["mi"]), "{path}: material sharing");
        let r = m.r();
        let ty = match r.kind {
            MatKind::Basic => "basic",
            MatKind::Lambert => "lambert",
            MatKind::LineBasic => "line",
            _ => "other",
        };
        assert_eq!(ty, jm["t"].as_str().unwrap(), "{path}: material type");
        assert_eq!(r.color.get_hex() as f64, f(&jm["c"]), "{path}: color");
        if ty == "lambert" {
            assert_eq!(r.emissive.get_hex() as f64, f(&jm["e"]), "{path}: emissive");
        }
        ck(r.opacity, &jm["o"], 1e-12, &format!("{path}: opacity"));
        assert_eq!(r.transparent, jm["tr"].as_bool().unwrap(), "{path}: transparent");
        let side = match r.side {
            Side::Front => 0.0,
            Side::Back => 1.0,
            Side::Double => 2.0,
        };
        assert_eq!(side, f(&jm["sd"]), "{path}: side");
        assert_eq!(r.depth_write, jm["dw"].as_bool().unwrap(), "{path}: depth_write");
        let bl = match r.blending {
            Blending::None => 0.0,
            Blending::Normal => 1.0,
            Blending::Additive => 2.0,
            Blending::Subtractive => 3.0,
            Blending::Multiply => 4.0,
        };
        assert_eq!(bl, f(&jm["bl"]), "{path}: blending");
        let map = r.map.clone();
        drop(r);
        cmp_tex(g, map, &jm["map"], &format!("{path} map"));
    } else {
        assert!(o.try_geo().is_none() || o.is_light(), "{path}: we have geometry, recorded none");
    }
    if let Some(jl) = rec.get("l") {
        let (c, i, d, dc) = {
            let l = o.light();
            (l.color.get_hex(), l.intensity, l.distance, l.decay)
        };
        assert_eq!(c as f64, f(&jl["c"]), "{path}: light color");
        ck(i, &jl["i"], 1e-12, &format!("{path}: light intensity"));
        ck(d, &jl["d"], 1e-12, &format!("{path}: light distance"));
        ck(dc, &jl["dc"], 1e-12, &format!("{path}: light decay"));
    }
    let kids = o.children();
    let jk = rec.get("c").and_then(|c| c.as_array()).map(|a| a.as_slice()).unwrap_or(&[]);
    assert_eq!(kids.len(), jk.len(), "{path}: child count");
    for (i, (k, j)) in kids.iter().zip(jk.iter()).enumerate() {
        cmp_tree(g, k, j, &format!("{path}/{i}"), ids);
    }
}
fn cand_arr(c: &Cand) -> [f64; 3] {
    [c.x as f64, c.z as f64, c.d as f64]
}
#[track_caller]
fn cmp_pickups(a: &[Pickup], rec: &Value, what: &str) {
    let ja = rec.as_array().unwrap();
    assert_eq!(a.len(), ja.len(), "{what}: count");
    for (i, (p, j)) in a.iter().zip(ja.iter()).enumerate() {
        ckv(&v3a(&p.g.pos()), &j["p"], 1e-12, &format!("{what}[{i}] pos"));
        ck(p.base, &j["base"], 1e-12, &format!("{what}[{i}] base"));
        ck(p.phase, &j["phase"], 1e-12, &format!("{what}[{i}] phase"));
        assert_eq!(p.taken, j["taken"].as_bool().unwrap(), "{what}[{i}] taken");
    }
}

/// `place_props`' first half on every fixture level: draws, locals, tree, records.
#[test]
fn place_props_head_matches_reference() {
    for c in load() {
        let seed = c["case"]["seed"].as_u64().unwrap() as u32;
        let tag = format!("case {seed}");
        let (mut g, dist) = setup(&c);
        let (draws, cx) = with_seed(seed, || {
            let d0 = seeded_draws();
            let cx = g.place_props_head(&dist);
            (seeded_draws() - d0, cx)
        });
        assert_eq!(draws as f64, f(&c["draws"]), "{tag}: seeded draws at the hand-off");
        // the shared locals
        let h = &c["handoff"];
        let jc = h["cands"].as_array().unwrap();
        assert_eq!(cx.cands.len(), jc.len(), "{tag}: cands");
        for (a, b) in cx.cands.iter().zip(jc) {
            ckv(&cand_arr(a), b, 0.0, &format!("{tag}: cand"));
        }
        let jt = h["taken"].as_array().unwrap();
        assert_eq!(cx.taken.len(), jt.len(), "{tag}: taken");
        for (a, b) in cx.taken.iter().zip(jt) {
            ckv(&cand_arr(a), b, 0.0, &format!("{tag}: taken"));
        }
        ckv(&cand_arr(&cx.exit_tile.unwrap()), &h["exitTile"], 0.0, &format!("{tag}: exitTile"));
        // the world
        cmp_tree(&g, &g.world, &c["tree"], &format!("{tag} world"), &mut Ids::default());
        assert_eq!(g.world.child_count() as f64, f(&c["worldChildren"]), "{tag}: world children");
        // interactables, animators
        let ji = c["interactables"].as_array().unwrap();
        assert_eq!(g.interactables.len(), ji.len(), "{tag}: interactables");
        for (i, (it, j)) in g.interactables.iter().zip(ji).enumerate() {
            ckv(&v3a(&it.pos), &j["p"], 1e-12, &format!("{tag}: interactable {i} pos"));
            ck(it.r, &j["r"], 0.0, &format!("{tag}: interactable {i} r"));
            assert_eq!(it.prompt, j["prompt"].as_str().unwrap(), "{tag}: interactable {i} prompt");
            assert_eq!((it.enabled)(&g), j["en"].as_bool().unwrap(), "{tag}: interactable {i} enabled");
        }
        assert_eq!(g.animators.len() as f64, f(&c["animators"]), "{tag}: animators");
        // records
        match (&g.exit_data, c["exitData"].is_null()) {
            (Some(e), false) => {
                ckv(&v3a(&e.pos), &c["exitData"]["pos"], 1e-12, &format!("{tag}: exitData.pos"));
                ckv(&v3a(&e.normal), &c["exitData"]["normal"], 0.0, &format!("{tag}: exitData.normal"));
            }
            (None, true) => {}
            _ => panic!("{tag}: exit_data presence"),
        }
        match (&g.tv_data, c["tvData"].is_null()) {
            (Some(t), false) => {
                let j = &c["tvData"];
                ckv(&v3a(&t.world), &j["world"], 1e-12, &format!("{tag}: tvData.world"));
                ckv(&v3a(&t.normal), &j["normal"], 0.0, &format!("{tag}: tvData.normal"));
                ck(t.acc, &j["acc"], 0.0, &format!("{tag}: tvData.acc"));
                ck(t.light.intensity(), &j["light"], 0.0, &format!("{tag}: tvData.light"));
                assert!(t.light.parent().is_some(), "{tag}: the TV light hangs on the set");
            }
            (None, true) => {}
            _ => panic!("{tag}: tv_data presence"),
        }
        match (&g.clock_data, c["clockData"].is_null()) {
            (Some(k), false) => {
                let j = &c["clockData"];
                ckv(&v3a(&k.world), &j["world"], 1e-12, &format!("{tag}: clockData.world"));
                ckv(&v3a(&k.normal), &j["normal"], 0.0, &format!("{tag}: clockData.normal"));
                assert_eq!(k.tock, j["tock"].as_bool().unwrap());
                assert!(k.sec_pivot.parent().is_some(), "{tag}: second hand pivot on the clock");
            }
            (None, true) => {}
            _ => panic!("{tag}: clock_data presence"),
        }
        match (&g.phone_data, c["phoneData"].is_null()) {
            (Some(p), false) => {
                let j = &c["phoneData"];
                ckv(&v3a(&p.world), &j["world"], 1e-12, &format!("{tag}: phoneData.world"));
                ckv(&v3a(&p.normal), &j["normal"], 0.0, &format!("{tag}: phoneData.normal"));
                assert_eq!((p.ringing, p.answered), (j["ringing"].as_bool().unwrap(), j["answered"].as_bool().unwrap()));
                ck(p.ring_t, &j["ringT"], 0.0, &format!("{tag}: ringT"));
                ckv(&v3a(&p.handset.as_ref().unwrap().pos()), &j["handset"], 1e-12, &format!("{tag}: handset"));
            }
            (None, true) => {}
            _ => panic!("{tag}: phone_data presence"),
        }
        match (&g.man_data, c["manData"].is_null()) {
            (Some(m), false) => {
                ckv(&v3a(&m.g.pos()), &c["manData"]["p"], 1e-12, &format!("{tag}: manData pos"));
                ck(m.f["timer"], &c["manData"]["timer"], 0.0, &format!("{tag}: manData timer"));
            }
            (None, true) => {}
            _ => panic!("{tag}: man_data presence"),
        }
        cmp_pickups(&g.almonds, &c["almonds"], &format!("{tag}: almonds"));
        cmp_pickups(&g.battery_items, &c["batteryItems"], &format!("{tag}: batteryItems"));
        let jd = c["drawerProps"].as_array().unwrap();
        assert_eq!(g.drawer_props.len(), jd.len(), "{tag}: drawer_props");
        let kids = g.world.children();
        for (dp, j) in g.drawer_props.iter().zip(jd) {
            let idx = kids.iter().position(|k| k.ptr_eq(&dp.cabinet)).map_or(-1.0, |i| i as f64);
            assert_eq!(idx, f(&j["cab"]), "{tag}: drawer cabinet");
            let ct: Vec<String> = dp.drawers.iter().map(|d| d.borrow().content_type.clone()).collect();
            let jct: Vec<String> = j["drawers"].as_array().unwrap().iter().map(|v| v.as_str().unwrap().to_string()).collect();
            assert_eq!(ct, jct, "{tag}: drawer contents");
        }
        assert_eq!(g.rain_windows.len() as f64, f(&c["rainWindows"]), "{tag}: rain_windows");
    }
    set_props_hooks(PropsHooks::NONE);
}

/// The animators over a scripted walk, then every interactable used once.
#[test]
fn place_props_head_runtime_matches_reference() {
    for c in load() {
        let seed = c["case"]["seed"].as_u64().unwrap() as u32;
        let tag = format!("case {seed}");
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
                if k == 650
                    && let Some(p) = g.phone_data.as_mut()
                {
                    p.ringing = true;
                }
                g.perf_t = t;
                g.run_animators(dt, t);
                if k % 30 == 29 {
                    let j = &rec_sim[si];
                    si += 1;
                    let w = format!("{tag} frame {k}");
                    assert_eq!((seeded_draws() - d0) as f64, f(&j["draws"]), "{w}: draws");
                    if let Some(m) = &g.man_data {
                        ckv(&v3a(&m.g.pos()), &j["man"]["p"], 1e-9, &format!("{w} mannequin pos"));
                        ck(m.g.rot().y, &j["man"]["ry"], 1e-9, &format!("{w} mannequin yaw"));
                        ck(m.f["timer"], &j["man"]["timer"], 1e-9, &format!("{w} mannequin timer"));
                    } else {
                        assert!(j["man"].is_null(), "{w}: mannequin presence");
                    }
                    if let Some(p) = &g.phone_data {
                        ckv(&v3a(&p.handset.as_ref().unwrap().pos()), &j["handset"], 1e-9, &format!("{w} handset"));
                    }
                    let floats: Vec<[f64; 3]> =
                        g.world.children().iter().filter(|o| o.r().user.floats).map(|o| v3a(&o.child(0).unwrap().pos())).collect();
                    let jf = j["floats"].as_array().unwrap();
                    assert_eq!(floats.len(), jf.len(), "{w}: floats");
                    for (a, b) in floats.iter().zip(jf) {
                        ckv(a, b, 1e-9, &format!("{w} balloon"));
                    }
                }
            }
            assert_eq!(si, rec_sim.len(), "{tag}: sim samples");
            assert_eq!((seeded_draws() - d0) as f64, f(&c["simDraws"]), "{tag}: sim draws");
            // every enabled interactable, in order
            g.perf_t = 50.0;
            let ju = c["uses"].as_array().unwrap();
            assert_eq!(g.interactables.len(), ju.len(), "{tag}: uses");
            for i in 0..g.interactables.len() {
                let en = (g.interactables[i].enabled)(&g);
                let j = &ju[i];
                assert_eq!(en, !j.is_null(), "{tag}: interactable {i} enabled before use");
                if !en {
                    continue;
                }
                g.msg_text.clear();
                let mut fu = std::mem::replace(&mut g.interactables[i].on_use, Box::new(|_| {}));
                fu(&mut g);
                g.interactables[i].on_use = fu;
                let w = format!("{} use {} ({})", tag, i, g.interactables[i].prompt);
                assert_eq!((seeded_draws() - d0) as f64, f(&j["draws"]), "{w}: draws");
                assert_eq!(g.msg_text, j["msg"].as_str().unwrap(), "{w}: message");
                ck(g.player.sanity, &j["sanity"], 0.0, &format!("{w}: sanity"));
                assert_eq!(g.ui.flash.style("opacity").unwrap_or(""), j["flash"].as_str().unwrap(), "{w}: flash");
            }
            let ja = &c["after"];
            assert_eq!((seeded_draws() - d0) as f64, f(&ja["draws"]), "{tag}: draws after the uses");
        });
        let ja = &c["after"];
        cmp_pickups(&g.almonds, &ja["almonds"], &format!("{tag}: almonds after"));
        cmp_pickups(&g.battery_items, &ja["batteryItems"], &format!("{tag}: batteryItems after"));
        let jp = ja["papers"].as_array().unwrap();
        assert_eq!(g.papers.len(), jp.len(), "{tag}: papers");
        for (p, j) in g.papers.iter().zip(jp) {
            ckv(&v3a(&p.g.pos()), &j["p"], 1e-12, &format!("{tag}: paper pos"));
            assert_eq!(p.taken, j["taken"].as_bool().unwrap());
            assert_eq!(p.g.child_count() as f64, f(&j["n"]), "{tag}: paper sheets");
        }
        let jd = ja["drawers"].as_array().unwrap();
        for (dp, j) in g.drawer_props.iter().zip(jd) {
            for (d, jj) in dp.drawers.iter().zip(j.as_array().unwrap()) {
                let d = d.borrow();
                assert_eq!(d.opened, jj[0].as_bool().unwrap(), "{tag}: drawer opened");
                ckv(&v3a(&d.mesh.pos()), &jj[1], 1e-12, &format!("{tag}: drawer slide"));
            }
        }
        assert_eq!(g.world.child_count() as f64, f(&ja["worldChildren"]), "{tag}: world children after");
        let je = ja["entities"].as_array().unwrap();
        for (e, j) in g.entities.iter().zip(je) {
            assert_eq!(e.state, j[0].as_str().unwrap(), "{tag}: entity state");
            ck(e.lose_t, &j[1], 0.0, &format!("{tag}: entity loseT"));
        }
        match (&g.phone_data, ja["phone"].is_null()) {
            (Some(p), false) => {
                let j = &ja["phone"];
                assert_eq!((p.answered, p.ringing), (j["answered"].as_bool().unwrap(), j["ringing"].as_bool().unwrap()), "{tag}: phone");
                match (p.f.get("hintUntil"), j["hintUntil"].as_f64()) {
                    (Some(a), Some(b)) => assert_eq!(*a, b, "{tag}: hint_until"),
                    (None, None) => {}
                    (a, b) => panic!("{tag}: hint_until got {a:?} recorded {b:?}"),
                }
            }
            (None, true) => {}
            _ => panic!("{tag}: phone presence"),
        }
        let jt: Vec<f64> = fs(&ja["timeouts"]);
        let rt: Vec<f64> = g.timeouts.iter().map(|t| t.at_ms - g.now_ms).collect();
        assert_eq!(rt, jt, "{tag}: timeouts");
        let jen = ja["enabled"].as_array().unwrap();
        for (i, (it, j)) in g.interactables.iter().zip(jen).enumerate() {
            assert_eq!((it.enabled)(&g), j.as_bool().unwrap(), "{tag}: enabled after, {i}");
        }
        let added = g.world.children()[world0..].to_vec();
        let jadd = ja["added"].as_array().unwrap();
        assert_eq!(added.len(), jadd.len(), "{tag}: added objects");
        for (i, (o, j)) in added.iter().zip(jadd).enumerate() {
            cmp_tree(&g, o, j, &format!("{tag} added {i}"), &mut Ids::default());
        }
        g.now_ms += 1000.0;
        g.run_scheduled();
        assert_eq!(g.ui.flash.style("opacity").unwrap_or(""), ja["flashAfterTimeouts"].as_str().unwrap(), "{tag}: flash after timeouts");
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
        assert!(g.exit_data.is_some(), "seed {seed}: no exit");
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
    let s = std::fs::read_to_string(p).unwrap_or_else(|e| panic!("{p}: {e} (missing fixture)"));
    let cases: Vec<Value> = serde_json::from_str(&s).unwrap();
    let mut g = Game::new_bare();
    let mut n = 0;
    for c in &cases {
        let name = c["name"].as_str().unwrap();
        let args = c["args"].as_array().unwrap();
        let seed = c["seed"].as_u64().unwrap() as u32;
        let tag = format!("{name}({args:?}) seed {seed}");
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
                "build_elevator" => build_elevator(&mut g, &mut cx, args[0].as_bool().unwrap()),
                "build_paper_pile" => build_paper_pile(&mut g, &mut cx),
                "build_floor_hole" => build_floor_hole(&mut g, &mut cx),
                "make_leaf_shape" => Built::new(make_leaf_shape(f(&args[0]), f(&args[1]), f(&args[2]) as u32, f(&args[3])), 0.0),
                o => panic!("unknown builder {o}"),
            };
            (seeded_draws() - d0, b)
        });
        assert_eq!(draws as f64, f(&c["draws"]), "{tag}: draws");
        ck(built.top, &c["top"], 0.0, &format!("{tag}: top"));
        cmp_tree(&g, &built.g, &c["tree"], &tag, &mut Ids::default());
        if c["drawers"].is_null() {
            assert!(built.drawers.is_empty(), "{tag}: unexpected drawers");
        } else {
            let jd = c["drawers"].as_array().unwrap();
            assert_eq!(built.drawers.len(), jd.len(), "{tag}: drawers");
            let kids = built.g.children();
            for (d, j) in built.drawers.iter().zip(jd) {
                let d = d.borrow();
                assert_eq!(d.content_type, j[0].as_str().unwrap(), "{tag}: drawer content");
                assert!(!d.opened && !d.content_taken);
                let idx = kids.iter().position(|k| k.ptr_eq(&d.mesh)).map_or(-1.0, |i| i as f64);
                assert_eq!(idx, f(&j[2]), "{tag}: drawer group index");
            }
        }
        assert_eq!((g.rain_windows.len() - rw) as f64, f(&c["rain"]), "{tag}: rain_windows");
        if c["rain"].as_i64().unwrap() > 0 {
            let last = g.rain_windows.last().unwrap();
            let idx = built.g.children().iter().position(|k| k.ptr_eq(last)).map_or(-1.0, |i| i as f64);
            assert_eq!(idx, f(&c["rainIdx"]), "{tag}: rain overlay");
        }
        n += 1;
    }
    // every recorded builder case
    assert_eq!(n, 147);
}

// ---------------------------------------------------------------- re-recording
// The fixtures above are recordings of this game. When a core prop is meant
// to change (as the vending machine did, docs/design/VENDING.md V1), they are
// re-recorded from the current code with every case's inputs kept (its seed,
// level, tiles, distances), through the same steps the tests take:
//   cargo test --release -p game --test core_props_head -- --ignored rerecord
// then review the diff (`git diff --stat`, and the changed values).

fn ser_tex(g: &Game, t: Option<Tex>) -> Value {
    let Some(t) = t else { return Value::Null };
    let rain_clone = {
        let (a, b) = (t.r(), g.rain_streak_tex.r());
        std::rc::Rc::ptr_eq(&a.image, &b.image) && t.0.as_ptr() != g.rain_streak_tex.0.as_ptr()
    };
    let tag = if t.0.as_ptr() == g.tex.static_tex.0.as_ptr() {
        "static"
    } else if rain_clone {
        "rain"
    } else {
        "canvas"
    };
    let r = t.r();
    serde_json::json!({"tag": tag, "w": t.width(), "h": t.height(), "ws": r.wrap_s == Wrap::Repeat, "wt": r.wrap_t == Wrap::Repeat, "rp": [r.repeat.x, r.repeat.y]})
}

/// The inverse of `cmp_tree`
fn ser_tree(g: &Game, o: &Obj, ids: &mut Ids) -> Value {
    let mut m = serde_json::Map::new();
    m.insert("k".into(), obj_kind(o).into());
    m.insert("p".into(), serde_json::json!(v3a(&o.pos())));
    let q = o.quat();
    m.insert("q".into(), serde_json::json!([q.x, q.y, q.z, q.w]));
    let s = v3a(&o.scl());
    if s != [1.0, 1.0, 1.0] {
        m.insert("s".into(), serde_json::json!(s));
    }
    if !o.visible() {
        m.insert("v".into(), false.into());
    }
    let order = format!("{:?}", o.rot().order);
    if order != "XYZ" {
        m.insert("ro".into(), order.into());
    }
    if o.r().user.gameplay {
        m.insert("gp".into(), true.into());
    }
    if o.r().user.floats {
        m.insert("fl".into(), true.into());
    }
    if o.try_geo().is_some() && !o.is_light() {
        let geo = o.geo();
        let mut t = vec![Value::from(geo_kind(&geo))];
        t.extend(geo_params(&geo).into_iter().map(Value::from));
        let ic = geo.r().index.as_ref().map_or(-1.0, |i| i.len() as f64);
        let nr = geo.r().normal.clone();
        m.insert(
            "g".into(),
            serde_json::json!({
                "gi": ids.geo(&geo), "part": geo.r().is_prop_part, "t": t, "vc": geo.vertex_count(), "ic": ic,
                "pos": attr_sum(&geo.r().position), "nor": if nr.is_empty() { Value::Null } else { serde_json::json!(attr_sum(&nr)) },
            }),
        );
        let mat = o.mat();
        let mi = ids.mat(&mat);
        let r = mat.r();
        let ty = match r.kind {
            MatKind::Basic => "basic",
            MatKind::Lambert => "lambert",
            MatKind::LineBasic => "line",
            _ => "other",
        };
        let side = match r.side {
            Side::Front => 0,
            Side::Back => 1,
            Side::Double => 2,
        };
        let bl = match r.blending {
            Blending::None => 0,
            Blending::Normal => 1,
            Blending::Additive => 2,
            Blending::Subtractive => 3,
            Blending::Multiply => 4,
        };
        let mut jm = serde_json::json!({"mi": mi, "t": ty, "c": r.color.get_hex(), "o": r.opacity, "tr": r.transparent, "sd": side, "dw": r.depth_write, "bl": bl});
        jm["e"] = if ty == "lambert" { r.emissive.get_hex() as i64 } else { -1 }.into();
        let map = r.map.clone();
        drop(r);
        jm["map"] = ser_tex(g, map);
        m.insert("m".into(), jm);
    }
    if o.is_light() {
        let l = o.light();
        m.insert("l".into(), serde_json::json!({"c": l.color.get_hex(), "i": l.intensity, "d": l.distance, "dc": l.decay}));
    }
    let kids: Vec<Value> = o.children().iter().map(|k| ser_tree(g, k, ids)).collect();
    if !kids.is_empty() {
        m.insert("c".into(), kids.into());
    }
    Value::Object(m)
}
fn ser_pickups(a: &[Pickup]) -> Value {
    a.iter().map(|p| serde_json::json!({"p": v3a(&p.g.pos()), "base": p.base, "phase": p.phase, "taken": p.taken})).collect()
}

/// `place_props_head_matches_reference`'s outputs for case `c`
fn record_head(c: &mut Value) {
    let seed = c["case"]["seed"].as_u64().unwrap() as u32;
    let (mut g, dist) = setup(c);
    let (draws, cx) = with_seed(seed, || {
        let d0 = seeded_draws();
        let cx = g.place_props_head(&dist);
        (seeded_draws() - d0, cx)
    });
    c["draws"] = draws.into();
    c["handoff"] = serde_json::json!({
        "cands": cx.cands.iter().map(cand_arr).collect::<Vec<_>>(),
        "taken": cx.taken.iter().map(cand_arr).collect::<Vec<_>>(),
        "exitTile": cand_arr(&cx.exit_tile.unwrap()),
    });
    c["tree"] = ser_tree(&g, &g.world, &mut Ids::default());
    c["worldChildren"] = g.world.child_count().into();
    c["interactables"] = g
        .interactables
        .iter()
        .map(|it| serde_json::json!({"p": v3a(&it.pos), "r": it.r, "prompt": it.prompt, "en": (it.enabled)(&g)}))
        .collect();
    c["animators"] = g.animators.len().into();
    c["exitData"] = g.exit_data.as_ref().map_or(Value::Null, |e| serde_json::json!({"pos": v3a(&e.pos), "normal": v3a(&e.normal)}));
    c["tvData"] = g.tv_data.as_ref().map_or(
        Value::Null,
        |t| serde_json::json!({"world": v3a(&t.world), "normal": v3a(&t.normal), "acc": t.acc, "light": t.light.intensity()}),
    );
    c["clockData"] = g
        .clock_data
        .as_ref()
        .map_or(Value::Null, |k| serde_json::json!({"world": v3a(&k.world), "normal": v3a(&k.normal), "tock": k.tock, "acc": k.acc}));
    c["phoneData"] = g
        .phone_data
        .as_ref()
        .map_or(Value::Null, |p| {
            serde_json::json!({"world": v3a(&p.world), "normal": v3a(&p.normal), "ringing": p.ringing, "answered": p.answered, "ringT": p.ring_t, "handset": v3a(&p.handset.as_ref().unwrap().pos())})
        });
    c["manData"] = g.man_data.as_ref().map_or(Value::Null, |m| serde_json::json!({"p": v3a(&m.g.pos()), "timer": m.f["timer"]}));
    c["almonds"] = ser_pickups(&g.almonds);
    c["batteryItems"] = ser_pickups(&g.battery_items);
    let kids = g.world.children();
    c["drawerProps"] = g
        .drawer_props
        .iter()
        .map(|dp| {
            let cab = kids.iter().position(|k| k.ptr_eq(&dp.cabinet)).map_or(-1.0, |i| i as f64);
            serde_json::json!({"cab": cab, "drawers": dp.drawers.iter().map(|d| d.borrow().content_type.clone()).collect::<Vec<_>>()})
        })
        .collect();
    c["rainWindows"] = g.rain_windows.len().into();
    set_props_hooks(PropsHooks::NONE);
}

/// `place_props_head_runtime_matches_reference`'s outputs for case `c`
fn record_runtime(c: &mut Value) {
    let seed = c["case"]["seed"].as_u64().unwrap() as u32;
    let (mut g, dist) = setup(c);
    with_seed(seed, || {
        g.place_props_head(&dist);
    });
    let world0 = g.world.child_count();
    let mut after = serde_json::Map::new();
    with_seed(seed ^ 0x5eed, || {
        let d0 = seeded_draws();
        let man = g.man_data.as_ref().map(|m| m.g.clone());
        let phone_world = g.phone_data.as_ref().map(|p| p.world);
        let mut sim = vec![];
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
            if k == 650
                && let Some(p) = g.phone_data.as_mut()
            {
                p.ringing = true;
            }
            g.perf_t = t;
            g.run_animators(dt, t);
            if k % 30 == 29 {
                let mut j = serde_json::json!({"k": k, "draws": seeded_draws() - d0});
                j["man"] = g
                    .man_data
                    .as_ref()
                    .map_or(Value::Null, |m| serde_json::json!({"p": v3a(&m.g.pos()), "ry": m.g.rot().y, "timer": m.f["timer"]}));
                if let Some(p) = &g.phone_data {
                    j["handset"] = serde_json::json!(v3a(&p.handset.as_ref().unwrap().pos()));
                }
                let floats: Vec<[f64; 3]> =
                    g.world.children().iter().filter(|o| o.r().user.floats).map(|o| v3a(&o.child(0).unwrap().pos())).collect();
                j["floats"] = serde_json::json!(floats);
                sim.push(j);
            }
        }
        c["sim"] = sim.into();
        c["simDraws"] = (seeded_draws() - d0).into();
        g.perf_t = 50.0;
        let mut uses = vec![];
        for i in 0..g.interactables.len() {
            if !(g.interactables[i].enabled)(&g) {
                uses.push(Value::Null);
                continue;
            }
            g.msg_text.clear();
            let mut fu = std::mem::replace(&mut g.interactables[i].on_use, Box::new(|_| {}));
            fu(&mut g);
            g.interactables[i].on_use = fu;
            uses.push(serde_json::json!({
                "draws": seeded_draws() - d0, "msg": g.msg_text, "sanity": g.player.sanity,
                "flash": g.ui.flash.style("opacity").unwrap_or(""),
            }));
        }
        c["uses"] = uses.into();
        after.insert("draws".into(), (seeded_draws() - d0).into());
    });
    after.insert("almonds".into(), ser_pickups(&g.almonds));
    after.insert("batteryItems".into(), ser_pickups(&g.battery_items));
    after.insert(
        "papers".into(),
        g.papers.iter().map(|p| serde_json::json!({"p": v3a(&p.g.pos()), "taken": p.taken, "n": p.g.child_count()})).collect(),
    );
    after.insert(
        "drawers".into(),
        g.drawer_props
            .iter()
            .map(|dp| dp.drawers.iter().map(|d| serde_json::json!([d.borrow().opened, v3a(&d.borrow().mesh.pos())])).collect::<Value>())
            .collect(),
    );
    after.insert("worldChildren".into(), g.world.child_count().into());
    after.insert("entities".into(), g.entities.iter().map(|e| serde_json::json!([e.state, e.lose_t])).collect());
    after.insert(
        "phone".into(),
        g.phone_data.as_ref().map_or(
            Value::Null,
            |p| serde_json::json!({"answered": p.answered, "ringing": p.ringing, "hintUntil": p.f.get("hintUntil").copied()}),
        ),
    );
    after.insert("timeouts".into(), g.timeouts.iter().map(|t| t.at_ms - g.now_ms).collect());
    after.insert("enabled".into(), g.interactables.iter().map(|it| Value::from((it.enabled)(&g))).collect());
    let added = g.world.children()[world0..].to_vec();
    after.insert("added".into(), added.iter().map(|o| ser_tree(&g, o, &mut Ids::default())).collect());
    g.now_ms += 1000.0;
    g.run_scheduled();
    after.insert("flashAfterTimeouts".into(), g.ui.flash.style("opacity").unwrap_or("").into());
    c["after"] = Value::Object(after);
    set_props_hooks(PropsHooks::NONE);
}

/// `builders_match_reference`'s outputs for case `c`
fn record_builder(g: &mut Game, c: &mut Value) {
    let name = c["name"].as_str().unwrap().to_string();
    let args = c["args"].as_array().unwrap().clone();
    let seed = c["seed"].as_u64().unwrap() as u32;
    game::assets::props::clear_prop_cache();
    let rw = g.rain_windows.len();
    let mut cx = PropsCtx::default();
    let (draws, built) = with_seed(seed, || {
        let d0 = seeded_draws();
        let b = match name.as_str() {
            "build_filing_cabinet" => build_filing_cabinet(g, &mut cx),
            "build_cardboard_stack" => build_cardboard_stack(g, &mut cx),
            "build_wet_floor_sign" => build_wet_floor_sign(g, &mut cx),
            "build_vending" => build_vending(g, &mut cx),
            "build_locker" => build_locker(g, &mut cx),
            "build_fire_extinguisher" => build_fire_extinguisher(g, &mut cx),
            "build_trash_can" => build_trash_can(g, &mut cx),
            "build_rubber_duck" => build_rubber_duck(g, &mut cx),
            "build_traffic_cone" => build_traffic_cone(g, &mut cx),
            "build_wheelchair" => build_wheelchair(g, &mut cx),
            "build_baby_doll" => build_baby_doll(g, &mut cx),
            "build_suitcase" => build_suitcase(g, &mut cx),
            "build_broken_mirror" => build_broken_mirror(g, &mut cx, f(&args[0])),
            "build_tv_on_floor" => build_tv_on_floor(g, &mut cx),
            "build_book_stack" => build_book_stack(g, &mut cx),
            "build_coat_rack" => build_coat_rack(g, &mut cx),
            "build_fluorescent_tube" => build_fluorescent_tube(g, &mut cx),
            "build_pile" => build_pile(g, &mut cx),
            "build_potted_plant" => build_potted_plant(g, &mut cx),
            "build_floor_plant" => build_floor_plant(g, &mut cx),
            "build_elevator" => build_elevator(g, &mut cx, args[0].as_bool().unwrap()),
            "build_paper_pile" => build_paper_pile(g, &mut cx),
            "build_floor_hole" => build_floor_hole(g, &mut cx),
            "make_leaf_shape" => Built::new(make_leaf_shape(f(&args[0]), f(&args[1]), f(&args[2]) as u32, f(&args[3])), 0.0),
            o => panic!("unknown builder {o}"),
        };
        (seeded_draws() - d0, b)
    });
    c["draws"] = draws.into();
    c["top"] = built.top.into();
    c["tree"] = ser_tree(g, &built.g, &mut Ids::default());
    let kids = built.g.children();
    c["drawers"] = if built.drawers.is_empty() {
        Value::Null
    } else {
        built
            .drawers
            .iter()
            .map(|d| {
                let d = d.borrow();
                let idx = kids.iter().position(|k| k.ptr_eq(&d.mesh)).map_or(-1.0, |i| i as f64);
                serde_json::json!([d.content_type, -1, idx])
            })
            .collect()
    };
    let rain = g.rain_windows.len() - rw;
    c["rain"] = rain.into();
    c["rainIdx"] = if rain > 0 {
        let last = g.rain_windows.last().unwrap();
        kids.iter().position(|k| k.ptr_eq(last)).map_or(-1.0, |i| i as f64).into()
    } else {
        (-1.0).into()
    };
}

/// Builders no longer in the code, whose recorded cases a rerecord drops
const RETIRED: &[&str] = &["build_window"];

#[test]
#[ignore]
fn rerecord() {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures");
    let mut cases = load();
    for c in cases.iter_mut() {
        record_head(c);
        record_runtime(c);
    }
    std::fs::write(format!("{dir}/core_props_head.json"), serde_json::to_string(&cases).unwrap()).unwrap();
    let p = format!("{dir}/core_props_builders.json");
    let mut builders: Vec<Value> = serde_json::from_str(&std::fs::read_to_string(&p).unwrap()).unwrap();
    // builders that are gone (the old framed window: `core_props::window`
    // builds the windows now, on the final walls)
    builders.retain(|c| !RETIRED.contains(&c["name"].as_str().unwrap_or("")));
    let mut g = Game::new_bare();
    for c in builders.iter_mut() {
        record_builder(&mut g, c);
    }
    std::fs::write(&p, serde_json::to_string(&builders).unwrap()).unwrap();
    println!("re-recorded {} head cases and {} builder cases", cases.len(), builders.len());
}
