//! game::anomalies* (backrooms.js 1581–2530) against the original JS, run by
//! tools/an_fixtures.mjs on the same layouts, seeds and scripted inputs.
//!
//! The fixtures come in two variants for the helpers other modules own
//! (world.rs: bfs / los / circleHitsWall / navTileAt; entities.rs: the
//! steering and skinBody): `stub` (what those stubs do now) and `real` (the
//! original functions). Each test probes which state the ports are in and
//! compares against the matching variant; if the two modules disagree (one
//! ported, the other not) the run-dependent comparisons are skipped with a note.
use engine::prelude::*;
use game::anomalies_rig::*;
use game::prelude::*;
use game::records::*;
use game::stairwell::js_hypot2;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::OnceLock;

fn fx() -> &'static Value {
    static F: OnceLock<Value> = OnceLock::new();
    F.get_or_init(|| {
        let p = format!("{}/tests/fixtures/anomalies.json", env!("CARGO_MANIFEST_DIR"));
        serde_json::from_str(&std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{p}: {e} (run `node tools/an_fixtures.mjs`)"))).unwrap()
    })
}
fn f(v: &Value) -> f64 { if v.is_null() { f64::NAN } else { v.as_f64().unwrap_or_else(|| panic!("not a number: {v}")) } }
fn arr(v: &Value) -> &Vec<Value> { v.as_array().unwrap_or_else(|| panic!("not an array: {v}")) }
fn s(v: &Value) -> &str { v.as_str().unwrap_or_else(|| panic!("not a string: {v}")) }
const TOL: f64 = 1e-9;
fn close(a: f64, b: f64, tol: f64) -> bool {
    if a.is_nan() || b.is_nan() { return a.is_nan() && b.is_nan(); }
    (a - b).abs() <= tol * 1f64.max(a.abs()).max(b.abs())
}
#[track_caller]
fn ac(a: f64, b: &Value, tol: f64, w: &str) { let b = f(b); assert!(close(a, b, tol), "{w}: rust {a} vs js {b} (diff {:e})", (a - b).abs()); }
#[track_caller]
fn aopt(a: Option<f64>, b: &Value, tol: f64, w: &str) {
    match a { None => assert!(b.is_null(), "{w}: rust undefined vs js {b}"), Some(a) => ac(a, b, tol, w) }
}

// ------------------------------------------------------------ the variant
fn world_is_stub() -> bool {
    let mut g = Game::new_bare();
    g.w = 5; g.h = 5; g.tiles = vec![0; 25];
    g.bfs(2, 2)[12] != 0
}
fn entities_is_stub() -> bool { game::entities::solve_limb(0.0, 0.0, -0.5, 0.0, 0.3, 0.3, 1.0) == (0.0, 0.0) }
fn skin_is_stub() -> bool {
    let g = group();
    human_rig(&g, &HUMAN_RIG_STAFF, None);
    let body = game::creatures::creature_geometry("stBody").unwrap();
    game::entities::skin_body(&g, &body, &lambert!()).is_none()
}
/// "stub" | "real", or None when the ports disagree
fn variant() -> Option<&'static str> {
    match (world_is_stub(), entities_is_stub(), skin_is_stub()) {
        (true, true, true) => Some("stub"),
        (false, false, false) => Some("real"),
        m => { eprintln!("[anomalies_js] world/entities/skin stubs = {m:?}: mixed port state, run comparisons skipped"); None }
    }
}

// ------------------------------------------------------------ tree compare
/// Material / geometry identity: a bijection between the JS numbering and the Rust handles.
#[derive(Default)]
struct Ids { m_js: HashMap<i64, u64>, m_rs: HashMap<u64, i64>, g_js: HashMap<i64, u64>, g_rs: HashMap<u64, i64> }
impl Ids {
    fn same(js: &mut HashMap<i64, u64>, rs: &mut HashMap<u64, i64>, j: i64, r: u64, w: &str) {
        let a = *js.entry(j).or_insert(r);
        let b = *rs.entry(r).or_insert(j);
        assert!(a == r && b == j, "{w}: sharing differs (js #{j}, rust id {r})");
    }
}
fn kind_name(o: &Obj) -> &'static str {
    match &o.r().kind {
        Kind::Group => "group", Kind::Object3D => "object3d", Kind::Mesh { .. } => "mesh", Kind::Instanced { .. } => "inst", Kind::Skinned { .. } => "skinned",
        Kind::Bone => "bone", Kind::PointLight(_) => "pointLight", _ => "other",
    }
}
fn geo_params(g: &Geo) -> Vec<Value> {
    let n = |v: f64| serde_json::json!(v);
    match g.params() {
        GeoParams::Box { width, height, depth, .. } => vec!["box".into(), n(width), n(height), n(depth)],
        GeoParams::Cylinder { radius_top, radius_bottom, height, radial, height_seg, open_ended, theta_start, theta_length } =>
            vec!["cyl".into(), n(radius_top), n(radius_bottom), n(height), n(radial as f64), n(height_seg as f64), Value::Bool(open_ended), n(theta_start), n(theta_length)],
        GeoParams::Plane { width, height, .. } => vec!["plane".into(), n(width), n(height)],
        GeoParams::Capsule { radius, length, cap_segments, radial } => vec!["capsule".into(), n(radius), n(length), n(cap_segments as f64), n(radial as f64)],
        p => vec![p.type_name().into()],
    }
}
fn mat_type(m: &Mat) -> &'static str {
    match m.r().kind { MatKind::Basic => "MeshBasicMaterial", MatKind::Lambert => "MeshLambertMaterial", MatKind::Phong => "MeshPhongMaterial", _ => "other" }
}
fn cmp_mat(m: &Mat, j: &Value, w: &str) {
    assert_eq!(mat_type(m), s(&j["type"]), "{w}: material type");
    let r = m.r();
    assert_eq!(r.color.get_hex() as i64, j["color"].as_i64().unwrap(), "{w}: color");
    if !j["emissive"].is_null() { assert_eq!(r.emissive.get_hex() as i64, j["emissive"].as_i64().unwrap(), "{w}: emissive"); }
    if !j["ei"].is_null() { ac(r.emissive_intensity, &j["ei"], TOL, &format!("{w}: emissiveIntensity")); }
    match (&r.map, &j["map"]) {
        (None, Value::Null) => {}
        (Some(t), Value::Array(sz)) => assert_eq!((t.width() as f64, t.height() as f64), (f(&sz[0]), f(&sz[1])), "{w}: map size"),
        (a, b) => panic!("{w}: map {:?} vs {b}", a.is_some()),
    }
    assert_eq!(r.transparent, j["tr"].as_bool().unwrap(), "{w}: transparent");
    ac(r.opacity, &j["op"], TOL, &format!("{w}: opacity"));
    assert_eq!(r.depth_write, j["dw"].as_bool().unwrap(), "{w}: depthWrite");
    let side = match r.side { Side::Front => 0, Side::Back => 1, Side::Double => 2 };
    assert_eq!(side, j["side"].as_i64().unwrap(), "{w}: side");
    assert_eq!(r.polygon_offset, j["po"].as_bool().unwrap(), "{w}: polygonOffset");
    ac(r.polygon_offset_factor, &j["pof"], TOL, &format!("{w}: polygonOffsetFactor"));
    assert_eq!(r.vertex_colors, j["vc"].as_bool().unwrap(), "{w}: vertexColors");
    if !j["spec"].is_null() { assert_eq!(r.specular.get_hex() as i64, j["spec"].as_i64().unwrap(), "{w}: specular"); }
    if !j["shin"].is_null() && r.kind == MatKind::Phong { ac(r.shininess, &j["shin"], TOL, &format!("{w}: shininess")); }
}
/// `skip_inst`: leave instanced meshes out (comparing a real-bfs world with the stub fixture)
fn cmp_tree(o: &Obj, j: &Value, mats: &Value, ids: &mut Ids, w: &str) {
    assert_eq!(kind_name(o), s(&j["k"]), "{w}: kind");
    let (p, r, sc) = (o.pos(), o.rot(), o.scl());
    let jp = arr(&j["p"]); let jr = arr(&j["r"]);
    for k in 0..3 { ac([p.x, p.y, p.z][k], &jp[k], TOL, &format!("{w}: position[{k}]")); ac([r.x, r.y, r.z][k], &jr[k], TOL, &format!("{w}: rotation[{k}]")); }
    assert_eq!(o.name(), j.get("n").and_then(|v| v.as_str()).unwrap_or(""), "{w}: name");
    match j.get("s") { Some(js) => { for k in 0..3 { ac([sc.x, sc.y, sc.z][k], &js[k], TOL, &format!("{w}: scale[{k}]")); } } None => assert_eq!((sc.x, sc.y, sc.z), (1.0, 1.0, 1.0), "{w}: scale") }
    assert_eq!(o.visible(), j.get("v").map_or(true, |v| v.as_bool().unwrap()), "{w}: visible");
    assert_eq!(o.r().render_order as i64, j.get("ro").map_or(0, |v| v.as_i64().unwrap()), "{w}: renderOrder");
    let empty = serde_json::json!({});
    let ud = j.get("ud").unwrap_or(&empty);
    match (game::store::hide_spot(o), ud.get("hide")) {
        (None, None) => {}
        (Some(h), Some(jh)) => {
            ac(h.eye, &jh[0], TOL, &format!("{w}: hide eye")); ac(h.r, &jh[1], TOL, &format!("{w}: hide r"));
            assert_eq!(h.kind, s(&jh[2]), "{w}: hide kind");
            match h.front { Some(fr) => ac(fr, &jh[3], TOL, &format!("{w}: hide front")), None => assert!(jh[3].is_null(), "{w}: hide front") }
        }
        (a, b) => panic!("{w}: hide {a:?} vs {b:?}"),
    }
    assert_eq!(o.r().user.gameplay, ud.get("gameplay").is_some(), "{w}: userData.gameplay");
    assert_eq!(o.r().user.arch, ud.get("arch").is_some(), "{w}: userData.arch");
    assert_eq!(o.r().user.s.get("main").map(|x| x.as_str()), ud.get("main").and_then(|v| v.as_str()), "{w}: userData.main");
    match ud.get("tail") {
        Some(t) => { let u = &o.r().user; for (k, key) in ["tail_x", "tail_y", "tail_z"].iter().enumerate() { ac(u.get(key), &t[k], TOL, &format!("{w}: tail[{k}]")); } }
        None => assert!(!o.r().user.has("tail_x"), "{w}: unexpected tail"),
    }
    if o.is_mesh() {
        let geo = o.geo();
        Ids::same(&mut ids.g_js, &mut ids.g_rs, j["gid"].as_i64().unwrap(), geo.id(), &format!("{w}: geometry"));
        let gp = geo_params(&geo);
        let jg = arr(&j["g"]);
        assert_eq!(gp.len(), jg.len(), "{w}: geometry {gp:?} vs {jg:?}");
        for (a, b) in gp.iter().zip(jg) { match (a, b) { (Value::Number(x), Value::Number(y)) => ac(x.as_f64().unwrap(), &Value::Number(y.clone()), TOL, &format!("{w}: geometry param")), _ => assert_eq!(a, b, "{w}: geometry") } }
        if let Some(bb) = j.get("bb") {
            let b = geo.compute_bounding_box();
            let v = [b.min.x, b.min.y, b.min.z, b.max.x, b.max.y, b.max.z];
            for k in 0..6 { ac(v[k], &bb[k], 1e-6, &format!("{w}: bbox[{k}]")); }
        }
        let m = o.mat();
        let jm = j["m"].as_i64().unwrap();
        Ids::same(&mut ids.m_js, &mut ids.m_rs, jm, m.id(), &format!("{w}: material"));
        cmp_mat(&m, &mats[jm as usize], &format!("{w}: material #{jm}"));
    }
    if let Kind::Instanced { inst, .. } = &o.r().kind {
        assert_eq!(inst.count as i64, j["count"].as_i64().unwrap(), "{w}: count");
        for (i, jm) in arr(&j["im"]).iter().enumerate() {
            for k in 0..16 { ac(inst.matrices[i].e[k] as f32 as f64, &jm[k], 1e-7, &format!("{w}: instance {i}[{k}]")); }
        }
    }
    if let Kind::Skinned { skeleton, .. } = &o.r().kind {
        let names: Vec<String> = skeleton.as_ref().map(|s| s.bones.iter().map(|b| b.name()).collect()).unwrap_or_default();
        let jn: Vec<String> = j["bones"].as_array().map(|a| a.iter().map(|v| s(v).to_string()).collect()).unwrap_or_default();
        assert_eq!(names, jn, "{w}: skeleton");
    }
    if o.is_point_light() {
        let l = o.r().light().cloned().unwrap();
        let jl = arr(&j["light"]);
        assert_eq!(l.color.get_hex() as i64, jl[0].as_i64().unwrap(), "{w}: light colour");
        ac(l.intensity, &jl[1], TOL, &format!("{w}: intensity")); ac(l.distance, &jl[2], TOL, &format!("{w}: distance")); ac(l.decay, &jl[3], TOL, &format!("{w}: decay"));
    }
    let kids = o.children();
    let jc: &[Value] = j.get("c").and_then(|c| c.as_array()).map(|a| a.as_slice()).unwrap_or(&[]);
    assert_eq!(kids.len(), jc.len(), "{w}: children");
    for (i, (c, jc)) in kids.iter().zip(jc).enumerate() { cmp_tree(c, jc, mats, ids, &format!("{w}/{i}")); }
}
fn bones_of(root: &Obj) -> Vec<[f64; 6]> {
    let mut out = vec![];
    root.traverse(&mut |o| { if o.is_bone() { let (p, r) = (o.pos(), o.rot()); out.push([p.x, p.y, p.z, r.x, r.y, r.z]); } });
    out
}
fn cmp_bones(root: &Obj, j: &Value, tol: f64, w: &str) {
    let b = bones_of(root);
    let jb = arr(j);
    assert_eq!(b.len(), jb.len(), "{w}: bone count");
    for (i, (x, y)) in b.iter().zip(jb).enumerate() { for k in 0..6 { ac(x[k], &y[k], tol, &format!("{w}: bone {i}[{k}]")); } }
}
const GAIT_KEYS: [&str; 11] = ["gSeed", "gX", "gZ", "gYaw", "gSpd", "gTurn", "gPh", "gHip", "gLook", "gLookP", "headYaw"];
const LEG_KEYS: [&str; 12] = ["fz", "ay", "az", "ph", "w", "y0", "z0", "p0", "ty", "tz", "tph", "ttoe"];
fn cmp_gait(e: &Entity, j: &Value, tol: f64, w: &str) {
    for k in GAIT_KEYS { aopt(e.num.get(k).copied(), &j[k], tol, &format!("{w}: {k}")); }
    for (li, jl) in arr(&j["legs"]).iter().enumerate() {
        assert_eq!(e.fl(&format!("legs.{li}.down")), jl["down"].as_bool().unwrap(), "{w}: leg {li} down");
        assert_eq!(e.fl(&format!("legs.{li}.sw")), jl["sw"].as_bool().unwrap(), "{w}: leg {li} sw");
        for k in LEG_KEYS { aopt(e.num.get(&format!("legs.{li}.{k}")).copied(), &jl[k], tol, &format!("{w}: leg {li} {k}")); }
    }
}
/// `entFrame(e)` of the harness
fn cmp_ent_frame(e: &Entity, j: &Value, tol: f64, w: &str) {
    let j = arr(j);
    assert_eq!(e.kind, s(&j[0]), "{w}: type");
    let p = e.pos();
    ac(p.x, &j[1], tol, &format!("{w}: x")); ac(p.z, &j[2], tol, &format!("{w}: z")); ac(e.yaw, &j[3], tol, &format!("{w}: yaw"));
    assert_eq!(e.state, s(&j[4]), "{w}: state");
    ac(e.lose_t, &j[5], tol, &format!("{w}: loseT")); ac(e.w_timer, &j[6], tol, &format!("{w}: wTimer")); ac(e.step_acc, &j[7], tol, &format!("{w}: stepAcc"));
    for (i, k) in [(8, "lookK"), (9, "huntK"), (10, "solid"), (11, "agit"), (12, "touchCD")] { aopt(e.num.get(k).copied(), &j[i], tol, &format!("{w}: {k}")); }
    ac(e.dying, &j[13], tol, &format!("{w}: dying")); ac(e.g.scl().x, &j[14], tol, &format!("{w}: scale"));
    match (&e.w_target, &j[15]) { (None, Value::Null) => {}, (Some((x, z)), Value::Array(t)) => assert_eq!((*x as i64, *z as i64), (t[0].as_i64().unwrap(), t[1].as_i64().unwrap()), "{w}: wTarget"), (a, b) => panic!("{w}: wTarget {a:?} vs {b}") }
    assert_eq!(e.w_field.is_some(), j[16].as_bool().unwrap(), "{w}: wField");
    assert_eq!(e.parts.get("edge").map(|o| o.visible()), j[17].as_bool(), "{w}: edge visible");
    assert_eq!(e.g.parent().is_some(), j[18].as_bool().unwrap(), "{w}: in world");
}
fn cmp_entity_rec(g: &Game, e: &Entity, j: &Value, w: &str) {
    assert_eq!(e.kind, s(&j["type"]), "{w}: type");
    ac(e.pos().x, &j["x"], TOL, &format!("{w}: x")); ac(e.pos().z, &j["z"], TOL, &format!("{w}: z")); ac(e.yaw, &j["yaw"], TOL, &format!("{w}: yaw"));
    assert_eq!(e.state, s(&j["state"]), "{w}: state");
    ac(e.speed, &j["speed"], TOL, &format!("{w}: speed")); ac(e.face_y, &j["faceY"], TOL, &format!("{w}: faceY")); ac(e.eye_y_offset, &j["eyeYOffset"], TOL, &format!("{w}: eyeYOffset"));
    for (v, k) in [(e.w_timer, "wTimer"), (e.lose_t, "loseT"), (e.step_acc, "stepAcc"), (e.freeze, "freeze"), (e.dying, "dying"), (e.vx, "vx"), (e.vz, "vz")] { ac(v, &j[k], TOL, &format!("{w}: {k}")); }
    assert!(j["wTarget"].is_null() && e.w_target.is_none() && !j["hasField"].as_bool().unwrap() && e.w_field.is_none(), "{w}: fresh wander state");
    let jn = j["num"].as_object().unwrap();
    for (k, v) in jn { aopt(e.num.get(k).copied(), v, TOL, &format!("{w}: num {k}")); }
    for k in ["lookK", "huntK", "solid", "agit", "phase", "touchCD"] { assert_eq!(e.num.contains_key(k), jn.contains_key(k), "{w}: has {k}"); }
    ac(e.g.scl().x, &j["scale"], TOL, &format!("{w}: scale"));
    assert_eq!(e.g.parent().map(|p| p == g.world).unwrap_or(false), j["inWorld"].as_bool().unwrap(), "{w}: in world");
    assert_eq!(e.face.as_ref().and_then(|f| f.parent()).map(|p| p == g.scene).unwrap_or(false), j["faceInScene"].as_bool().unwrap(), "{w}: face in scene");
    assert_eq!(e.parts.get("edge").map(|o| o.visible()), j["edgeVisible"].as_bool(), "{w}: edge");
}

// ------------------------------------------------------------ the games
fn bare() -> Game {
    let mut g = Game::new_bare();
    game::props::set_props_enabled(false);
    g.state = "playing".into();
    g.god_mode = true;
    g.player.pos = v3(0, EYE, 0);
    g.init_anomalies();
    g
}
/// the fixture's level, regenerated by archgen (and checked against the JS layout)
fn level_game(rec: &Value, scp: &'static str) -> Game {
    let (seed, size, level) = (rec["seed"].as_u64().unwrap() as u32, rec["size"].as_i64().unwrap() as i32, rec["level"].as_i64().unwrap() as i32);
    let mut m = Mulberry32::new(seed);
    let layout = game::archgen::generate_layout(&game::archgen::LayoutOpts { size, level, cell: CELL, scp: Some(scp) }, &mut || m.next());
    let jl = &rec["layout"];
    assert_eq!((layout.w as i64, layout.h as i64), (jl["W"].as_i64().unwrap(), jl["H"].as_i64().unwrap()), "layout size");
    assert_eq!(layout.dominant, s(&jl["dominant"]));
    assert_eq!(layout.tiles.iter().map(|&t| t as i64).collect::<Vec<_>>(), arr(&jl["tiles"]).iter().map(|v| v.as_i64().unwrap()).collect::<Vec<_>>(), "tiles");
    assert_eq!(layout.zone_of.iter().map(|&t| t as i64).collect::<Vec<_>>(), arr(&jl["zoneOf"]).iter().map(|v| v.as_i64().unwrap()).collect::<Vec<_>>(), "zoneOf");
    assert_eq!(layout.zones.iter().map(|z| z.arch.to_string()).collect::<Vec<_>>(), arr(&jl["zones"]).iter().map(|v| s(v).to_string()).collect::<Vec<_>>(), "zones");
    let mut g = bare();
    g.w = layout.w; g.h = layout.h; g.tiles = layout.tiles.clone(); g.has_layout = true;
    g.layout = Some(layout);
    g.level_num = level;
    g.level_scp = Some(scp.to_string());
    let ex = arr(&rec["exit"]);
    let (ex, ez) = (ex[0].as_i64().unwrap() as i32, ex[1].as_i64().unwrap() as i32);
    g.exit_data = Some(ExitData { pos: v3(g.wx(ex), 0, g.wz(ez)), ..Default::default() });
    g
}
/// the spawn's BFS distances (as the harness computes them)
fn spawn_dist(g: &Game, rec: &Value) -> Vec<i32> {
    if let Some(d) = rec.get("dist") { return arr(d).iter().map(|v| v.as_i64().unwrap() as i32).collect(); }
    let sp = arr(&rec["spawn"]);
    let (sx, sz) = (sp[0].as_i64().unwrap() as i32, sp[1].as_i64().unwrap() as i32);
    let (w, h) = (g.w, g.h);
    let mut d = vec![-1; (w * h) as usize];
    let mut q = std::collections::VecDeque::new();
    d[(sz * w + sx) as usize] = 0; q.push_back((sx, sz));
    while let Some((x, z)) = q.pop_front() {
        let dd = d[(z * w + x) as usize];
        for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
            let (nx, nz) = (x + dx, z + dz);
            if nx < 0 || nz < 0 || nx >= w || nz >= h { continue; }
            let i = (nz * w + nx) as usize;
            if g.tiles[i] == 0 && d[i] == -1 { d[i] = dd + 1; q.push_back((nx, nz)); }
        }
    }
    d
}
fn use_interactable(g: &mut Game, i: usize) {
    let mut fun = std::mem::replace(&mut g.interactables[i].on_use, Box::new(|_| {}));
    fun(g);
    g.interactables[i].on_use = fun;
}
fn record<'a>(list: &'a Value, variant: &str, seed: i64) -> &'a Value {
    arr(list).iter().find(|r| r["variant"] == variant && r["seed"].as_i64() == Some(seed)).unwrap()
}

// ================================================================== tests
#[test]
fn scp_on_level_with_hash_overrides() {
    for c in arr(&fx()["scp"]) {
        let mut g = Game::new_bare();
        g.location_hash = s(&c["hash"]).to_string();
        g.run_seed = c["runSeed"].as_u64().unwrap() as u32;
        for (i, want) in arr(&c["levels"]).iter().enumerate() {
            let n = i as i32 + 1;
            assert_eq!(g.scp_on_level(n).as_deref(), want.as_str(), "hash {:?} seed {} level {n}", g.location_hash, g.run_seed);
        }
        assert_eq!(g.level_has_stairwell(3), g.scp_on_level(3).as_deref() == Some("stairwell"));
    }
    assert_eq!(game::anomalies::js_to_number(" 3 "), 3.0);
    assert_eq!(game::anomalies::js_to_number("0x3"), 3.0);
    assert!(game::anomalies::js_to_number("3x").is_nan());
    assert_eq!(game::anomalies::js_to_number(""), 0.0);
    assert_eq!(game::anomalies::decode_uri_component("store%403%2Chouse"), "store@3,house");
}

#[test]
fn human_rig_trees_fingers_and_noise() {
    let r = &fx()["rig"];
    for name in ["staff", "shadow"] {
        let rig = human_rig_named(name).unwrap();
        for (si, sgn) in [1.0, -1.0].iter().enumerate() {
            let pts = human_finger_pts(rig, *sgn);
            let j = arr(&r["fingers"][name][si]);
            assert_eq!(pts.len(), j.len());
            for (a, b) in pts.iter().zip(j) { for (p, q) in a.iter().zip(arr(b)) { for k in 0..3 { ac(p[k], &q[k], 1e-12, &format!("{name} fingers")); } } }
        }
    }
    for c in arr(&r["rigs"]) {
        let name = s(&c["name"]);
        let rigid = c["rigid"].as_bool().unwrap();
        let g = group();
        let mat = lambert!({ color: 0xc9a23a });
        let p = human_rig(&g, human_rig_named(name).unwrap(), if rigid { Some(&mat) } else { None });
        let w = format!("humanRig({name}, rigid {rigid})");
        cmp_tree(&g, &c["tree"], &c["mats"], &mut Ids::default(), &w);
        for (l, jl) in p.legs.iter().zip(arr(&c["legs"])) {
            assert_eq!([l.hip.name(), l.knee.name(), l.ankle.name(), l.toe.name()].to_vec(), (0..4).map(|k| s(&jl[k]).to_string()).collect::<Vec<_>>(), "{w}: leg");
            ac(l.side, &jl[4], 0.0, &w);
        }
        for (a, ja) in p.arms.iter().zip(arr(&c["arms"])) {
            assert_eq!([a.clav.name(), a.shoulder.name(), a.elbow.name(), a.wrist.name()].to_vec(), (0..4).map(|k| s(&ja[k]).to_string()).collect::<Vec<_>>(), "{w}: arm");
            let jf: Vec<Vec<String>> = arr(&ja[4]).iter().map(|c| arr(c).iter().map(|b| s(b).to_string()).collect()).collect();
            assert_eq!(a.fingers.iter().map(|c| c.iter().map(|b| b.name()).collect::<Vec<_>>()).collect::<Vec<_>>(), jf, "{w}: fingers");
        }
        // the parts map round-trips
        let m = p.to_map();
        let back = RigParts::from_map(&m).unwrap();
        assert!(back.pelvis == p.pelvis && back.arms[1].fingers[4][2] == p.arms[1].fingers[4][2] && m["torso"] == p.pelvis);
    }
    for row in arr(&r["gaitNoise"]) { ac(gait_noise(f(&row[0])), &row[1], 1e-12, "gaitNoise"); }
    let g = bare();
    for (m, j, edge) in [(&g.shadow_mat, &r["shadowMats"][0], false), (&g.shadow_edge_mat, &r["shadowMats"][1], true)] {
        assert_eq!(mat_type(m), s(&j["type"]));
        assert_eq!(m.r().color.get_hex() as i64, j["color"].as_i64().unwrap());
        assert_eq!(m.r().transparent, j["tr"].as_bool().unwrap());
        ac(m.r().opacity, &j["op"], TOL, "shadow opacity");
        assert_eq!(m.r().depth_write, j["dw"].as_bool().unwrap());
        assert_eq!(m.r().side == Side::Back, j["side"].as_i64() == Some(1));
        assert_eq!(m.r().patch, Patch::Shadow { edge });
        assert_eq!(s(&j["key"]), if edge { "shadowEdge" } else { "shadowBody" });
    }
    // one SHADOW_U for both
    assert!(std::rc::Rc::ptr_eq(&g.shadow_mat.r().patch_uniforms, &g.shadow_edge_mat.r().patch_uniforms));
    assert_eq!(g.shadow_mat.r().patch_uniforms.borrow().f[1], 1.0);
}

#[test]
fn human_gait_scripted_walk() {
    for c in arr(&fx()["gait"]) {
        let name = s(&c["name"]);
        let rig = human_rig_named(name).unwrap();
        let mut g = bare();
        g.w = 21; g.h = 21; g.tiles = vec![0; 441];
        let seed = if name == "staff" { 101 } else { 202 };
        let script = arr(&c["script"]);
        let snaps = arr(&c["snaps"]);
        let mut si = 0;
        with_seed(seed, || {
            let base = seeded_draws();
            let b = if name == "staff" { g.make_staff(10, 10) } else { g.make_shadow(10, 10) };
            let mut e = Entity::new(name, b.g.clone());
            e.parts = b.parts.to_map();
            g.entities.push(e);
            for (k, fr) in script.iter().enumerate() {
                let p = g.entities[0].g.clone();
                p.position().x = f(&fr["x"]); p.position().z = f(&fr["z"]);
                g.entities[0].yaw = f(&fr["yaw"]);
                g.player.pos = v3(f(&fr["px"]), f(&fr["py"]), f(&fr["pz"]));
                let j = &fr["st"];
                let st = GaitStyle { hunt: f(&j["hunt"]), glide: f(&j["glide"]), reach: f(&j["reach"]), look: f(&j["look"]), shiver: f(&j["shiver"]), tilt: f(&j["tilt"]), breath: f(&j["breath"]) };
                g.human_gait(0, f(&fr["dt"]), f(&fr["t"]), rig, &st);
                if si < snaps.len() && snaps[si]["k"].as_i64() == Some(k as i64) {
                    let w = format!("{name} gait frame {k}");
                    cmp_gait(&g.entities[0], &snaps[si]["gait"], 1e-7, &w);
                    cmp_bones(&b.g, &snaps[si]["bones"], 1e-7, &w);
                    si += 1;
                }
            }
            assert_eq!((seeded_draws() - base) as i64, c["draws"].as_i64().unwrap(), "{name}: gait draws");
        });
        assert_eq!(si, snaps.len(), "{name}: snapshots checked");
    }
}

#[test]
fn place_store_spawn_staff_and_run() {
    let Some(var) = variant() else { return };
    let store = &fx()["store"];
    for seed in [11i64, 12, 13] {
        let rec = record(store, var, seed);
        let stub_rec = record(store, "stub", seed);
        let mut g = level_game(rec, "store");
        for _ in 0..24 { g.panels.push(Panel::default()); }
        let dist = spawn_dist(&g, rec);
        let w = format!("store seed {seed} ({var})");
        with_seed((seed * 7 + 1) as u32, || {
            let base = seeded_draws();
            g.store_reset();
            g.place_store(&dist);
            assert_eq!((seeded_draws() - base) as i64, rec["placeDraws"].as_i64().unwrap(), "{w}: placeStore draws");
            // the world: everything but the arrows against the stub dump, the arrows against this variant's
            let kids = g.world.children();
            let sim = rec.get("frames").is_some();
            let jw = arr(&stub_rec["world"]);
            let plain: Vec<&Obj> = kids.iter().filter(|o| kind_name(o) != "inst").collect();
            assert_eq!(plain.len(), jw.len(), "{w}: world children");
            let mut ids = Ids::default();
            for (i, (o, j)) in plain.iter().zip(jw).enumerate() {
                if sim { cmp_tree(o, j, &stub_rec["mats"], &mut ids, &format!("{w}: world {i}")); }
                else {
                    assert_eq!(kind_name(o), s(&j["k"]), "{w}: world {i} kind");
                    let (p, r) = (o.pos(), o.rot());
                    for k in 0..3 { ac([p.x, p.y, p.z][k], &j["p"][k], TOL, &format!("{w}: world {i} p")); ac([r.x, r.y, r.z][k], &j["r"][k], TOL, &format!("{w}: world {i} r")); }
                    assert_eq!(o.child_count() as i64, j["nc"].as_i64().unwrap(), "{w}: world {i} parts");
                }
            }
            let insts: Vec<(usize, &Obj)> = kids.iter().enumerate().filter(|(_, o)| kind_name(o) == "inst").collect();
            let ji: Vec<&Value> = if var == "real" { arr(&rec["world"]).iter().collect() } else { vec![] };
            assert_eq!(insts.len(), ji.len(), "{w}: arrows");
            for ((i, o), j) in insts.iter().zip(ji) {
                assert_eq!(*i as i64, j["i"].as_i64().unwrap(), "{w}: arrows' slot");
                if sim { cmp_tree(o, j, &rec["mats"], &mut Ids::default(), &format!("{w}: arrows")); }
            }
            let js = &rec["store"];
            assert!(g.store_state.on);
            assert_eq!(g.store_state.board_text, s(&js["boardText"]));
            let jh = arr(&js["hides"]);
            assert_eq!(g.store_hide_spots.len(), jh.len(), "{w}: hides");
            assert_eq!(g.store_state.hides.len(), jh.len());
            for (h, j) in g.store_hide_spots.iter().zip(jh) {
                assert_eq!(h.kind, s(&j[0]));
                for (k, v) in [h.obj.pos().x, h.obj.pos().z, h.obj.rot().y, h.eye, h.r].iter().enumerate() { ac(*v, &j[k + 1], TOL, &format!("{w}: hide")); }
                match h.front { Some(fr) => ac(fr, &j[6], TOL, "front"), None => assert!(j[6].is_null()) }
            }
            // the staff
            let c0 = g.world.child_count();
            let scene0 = g.scene.child_count();
            let n = 4 + 2.min(g.level_num / 5);
            g.spawn_staff(n, &dist);
            assert_eq!((seeded_draws() - base) as i64, rec["spawnDraws"].as_i64().unwrap(), "{w}: spawnStaff draws");
            assert_eq!((g.world.child_count() - c0) as i64, rec["worldAfterSpawn"].as_i64().unwrap());
            assert_eq!((g.scene.child_count() - scene0) as i64, g.entities.len() as i64, "{w}: a face per staff");
            let je = arr(&rec["entities"]);
            assert_eq!(g.entities.len(), je.len(), "{w}: staff");
            for (i, j) in je.iter().enumerate() { cmp_entity_rec(&g, &g.entities[i], j, &format!("{w}: staff {i}")); }
            if sim {
                let mut ids = Ids::default();
                for (i, j) in arr(&rec["staffTrees"]).iter().enumerate() { cmp_tree(&g.entities[i].g, j, &rec["mats"], &mut ids, &format!("{w}: staff tree {i}")); }
            }
        });
        let Some(frames) = rec.get("frames") else { continue };
        // a run through day, closing, night and the next morning
        with_seed((seed * 13 + 5) as u32, || {
            let base = seeded_draws();
            let (mut field_t, mut t) = (0.0, 0.0);
            for (k, fr) in arr(frames).iter().enumerate() {
                let inp = arr(&fr["input"]);
                let dt = f(&inp[5]);
                let (px, pz) = (f(&inp[0]), f(&inp[1]));
                g.player.pos = v3(px, EYE, pz);
                g.player.crouch = inp[2].as_bool().unwrap(); g.player.flashlight_on = inp[3].as_bool().unwrap(); g.player.audible_range = f(&inp[4]);
                field_t -= dt;
                if field_t <= 0.0 { field_t = 0.35; let (tx, tz) = g.nav_tile_at(px, pz); g.player_field = Some(Rc::new(g.bfs(tx, tz))); }
                t += dt;
                for i in 0..g.entities.len() {
                    let yaw = g.entities[i].yaw; g.entities[i].g.rotation().y = yaw;
                    let p = g.entities[i].pos();
                    let d = js_hypot2(p.x - px, p.z - pz);
                    if g.entities[i].freeze > 0.0 { g.entities[i].freeze -= dt; continue; }
                    if g.entities[i].kind == "staff" && g.update_staff(i, dt, t, d) { break; }
                }
                g.update_scp(dt);
                let w = format!("{w} frame {k}");
                assert_eq!((seeded_draws() - base) as i64, fr["draws"].as_i64().unwrap(), "{w}: draws");
                let Some(st) = fr.get("store") else { continue };
                let st = arr(st);
                let ss = &g.store_state;
                ac(ss.t, &st[0], TOL, &format!("{w}: t")); assert_eq!(ss.phase, s(&st[1]), "{w}: phase"); ac(ss.cycle, &st[2], 0.0, &w);
                ac(ss.lights_out, &st[3], TOL, &format!("{w}: lightsOut")); assert_eq!(ss.hidden, st[4].as_bool().unwrap(), "{w}: hidden");
                assert_eq!(ss.board_text, s(&st[5]), "{w}: board"); aopt(g.player.hide_eye, &st[6], TOL, &format!("{w}: hideEye"));
                assert_eq!(ss.panels_off, st[7].as_bool().unwrap(), "{w}: panelsOff"); ac(g.shake_t, &st[8], TOL, &format!("{w}: shakeT"));
                ac(g.player.sanity, &st[9], TOL, &format!("{w}: sanity")); ac(g.god_deaths, &st[10], 0.0, &format!("{w}: deaths"));
                assert_eq!(g.dom.crouch_ind.text, s(&st[11]), "{w}: crouch indicator");
                ac(ss.arrow_mat.as_ref().unwrap().r().color.r, &st[12], TOL, &format!("{w}: arrows"));
                for (i, je) in arr(&fr["ents"]).iter().enumerate() { cmp_ent_frame(&g.entities[i], je, 1e-7, &format!("{w}: staff {i}")); }
                if let Some(ga) = fr.get("gait") {
                    for (i, j) in arr(ga).iter().enumerate() { cmp_gait(&g.entities[i], j, 1e-6, &format!("{w}: staff {i}")); }
                    for (i, j) in arr(&fr["bones"]).iter().enumerate() { cmp_bones(&g.entities[i].g, j, 1e-6, &format!("{w}: staff {i}")); }
                }
            }
        });
        for (p, j) in g.panels.iter().zip(arr(&rec["panels"])) { ac(p.flicker, &j[0], TOL, "panel flicker"); assert_eq!(p.switched_off, j[1].as_bool().unwrap()); }
        let msgs = arr(&rec["msgs"]);
        if let Some(last) = msgs.last() { assert_eq!(g.msg_text, s(&last[0]), "{w}: last message"); }
    }
}

#[test]
fn place_house_spawn_shadows_and_run() {
    let Some(var) = variant() else { return };
    for seed in [21i64, 22] {
        let rec = record(&fx()["house"], var, seed);
        let mut g = level_game(rec, "house");
        let dist = spawn_dist(&g, rec);
        let w = format!("house seed {seed} ({var})");
        with_seed((seed * 7 + 3) as u32, || {
            let base = seeded_draws();
            g.house_reset();
            g.place_house(&dist);
            assert_eq!((seeded_draws() - base) as i64, rec["placeDraws"].as_i64().unwrap(), "{w}: placeHouse draws");
            let kids = g.world.children();
            let jw = arr(&rec["world"]);
            assert_eq!(kids.len(), jw.len(), "{w}: world");
            let mut ids = Ids::default();
            for (i, (o, j)) in kids.iter().zip(jw).enumerate() { cmp_tree(o, j, &rec["mats"], &mut ids, &format!("{w}: world {i}")); }
            let (nx, nz) = g.house_state.nest.unwrap();
            ac(nx, &rec["house"]["nest"][0], TOL, "nest x"); ac(nz, &rec["house"]["nest"][1], TOL, "nest z");
            assert_eq!(g.house_state.lamps.len() as i64, rec["house"]["lamps"].as_i64().unwrap());
            let ji = arr(&rec["interactables"]);
            assert_eq!(g.interactables.len(), ji.len());
            for (it, j) in g.interactables.iter().zip(ji) {
                for (k, v) in [it.pos.x, it.pos.y, it.pos.z, it.r].iter().enumerate() { ac(*v, &j[k], TOL, &format!("{w}: interactable")); }
                assert_eq!(it.prompt, s(&j[4]));
            }
            let c0 = g.world.child_count();
            g.spawn_shadows(3 + if g.level_num >= 8 { 1 } else { 0 }, &dist);
            assert_eq!((seeded_draws() - base) as i64, rec["spawnDraws"].as_i64().unwrap(), "{w}: spawnShadows draws");
            assert_eq!((g.world.child_count() - c0) as i64, rec["worldAfterSpawn"].as_i64().unwrap());
            for (i, j) in arr(&rec["entities"]).iter().enumerate() { cmp_entity_rec(&g, &g.entities[i], j, &format!("{w}: shadow {i}")); }
            for (i, j) in arr(&rec["shadowTrees"]).iter().enumerate() { cmp_tree(&g.entities[i].g, j, &rec["mats"], &mut ids, &format!("{w}: shadow tree {i}")); }
        });
        // in the dark and lit, by the nest, the lamps going out one by one
        with_seed((seed * 13 + 9) as u32, || {
            let base = seeded_draws();
            let (mut field_t, mut t) = (0.0, 0.0);
            for (k, fr) in arr(&rec["frames"]).iter().enumerate() {
                let inp = arr(&fr["input"]);
                let dt = f(&inp[3]);
                let (px, pz) = (f(&inp[0]), f(&inp[1]));
                g.player.pos = v3(px, EYE, pz);
                g.player.flashlight_on = inp[2].as_bool().unwrap();
                if k >= 400 && k % 30 == 0 {
                    let i = (0..g.interactables.len()).find(|&i| (g.interactables[i].enabled)(&g));
                    assert_eq!(i.map_or(-1, |i| i as i64), fr["used"].as_i64().unwrap(), "frame {k}: lamp");
                    if let Some(i) = i { use_interactable(&mut g, i); }
                }
                field_t -= dt;
                if field_t <= 0.0 { field_t = 0.35; let (tx, tz) = g.nav_tile_at(px, pz); g.player_field = Some(Rc::new(g.bfs(tx, tz))); }
                t += dt;
                for i in 0..g.entities.len() {
                    let yaw = g.entities[i].yaw; g.entities[i].g.rotation().y = yaw;
                    let p = g.entities[i].pos();
                    let d = js_hypot2(p.x - px, p.z - pz);
                    let kind = g.entities[i].kind.clone();
                    if (kind == "staff" || kind == "shadow") && g.entities[i].freeze > 0.0 { g.entities[i].freeze -= dt; continue; }
                    if kind == "shadow" && g.update_shadow(i, dt, t, d) { break; }
                }
                g.update_scp(dt);
                let w = format!("{w} frame {k}");
                // The "real" fixtures run the ported world/entities on JS-recorded inputs.
                // Math.sin/cos/atan2 differ from the platform libm in the last bit (and
                // between V8 versions), so after a few hundred frames a shadow can cross
                // the kill radius a frame apart: compare everything strictly up to frame
                // 450, then only the house state the scripted lamp use drives.
                let strict = var != "real" || k < 450;
                if strict { assert_eq!((seeded_draws() - base) as i64, fr["draws"].as_i64().unwrap(), "{w}: draws"); }
                let Some(h) = fr.get("house") else { continue };
                let h = arr(h);
                let hs = &g.house_state;
                assert_eq!(hs.out, h[0].as_bool().unwrap(), "{w}: out");
                assert_eq!(hs.lamps.iter().filter(|l| !l.out).count() as i64, h[1].as_i64().unwrap(), "{w}: lamps lit");
                ac(hs.nest_light.as_ref().unwrap().intensity(), &h[2], TOL, &format!("{w}: nest light"));
                ac(g.shadow_u_time, &h[3], TOL, &format!("{w}: uTime")); ac(g.shadow_u_smoke, &h[4], 1e-7, &format!("{w}: uSmoke"));
                let pu = g.shadow_mat.r().patch_uniforms.borrow().f;
                assert_eq!((pu[0], pu[1]), (g.shadow_u_time as f32, g.shadow_u_smoke as f32), "{w}: SHADOW_U uniforms");
                ac(g.shadow_mat.opacity_v(), &h[5], 1e-7, &format!("{w}: body opacity")); ac(g.shadow_edge_mat.opacity_v(), &h[6], 1e-7, &format!("{w}: edge opacity"));
                assert_eq!(g.player_lit_for_shadows(), h[10].as_bool().unwrap(), "{w}: lit");
                if !strict { continue; }
                ac(g.player.sanity, &h[7], 1e-9, &format!("{w}: sanity")); ac(g.god_deaths, &h[8], 0.0, &format!("{w}: deaths"));
                aopt(g.dom.flash.opacity(), &h[9], TOL, &format!("{w}: flash"));
                for (i, je) in arr(&fr["ents"]).iter().enumerate() { cmp_ent_frame(&g.entities[i], je, 1e-7, &format!("{w}: shadow {i}")); }
                if let Some(ga) = fr.get("gait") {
                    let live: Vec<usize> = (0..g.entities.len()).filter(|&i| g.entities[i].kind != "gone").collect();
                    for (j, &i) in arr(ga).iter().zip(&live) { cmp_gait(&g.entities[i], j, 1e-6, &format!("{w}: shadow {i}")); }
                    for (i, j) in arr(&fr["bones"]).iter().enumerate() { cmp_bones(&g.entities[i].g, j, 1e-6, &format!("{w}: shadow {i}")); }
                }
            }
        });
        for (l, j) in g.house_state.lamps.iter().zip(arr(&rec["lampsEnd"])) {
            assert_eq!(l.out, j[0].as_bool().unwrap());
            let m = l.objs["shade"].mat();
            ac(m.r().emissive_intensity, &j[1], TOL, "shade glow");
            for k in 0..3 { ac([m.r().color.r, m.r().color.g, m.r().color.b][k], &j[k + 2], 1e-9, "shade colour"); }
        }
        if let Some(last) = arr(&rec["msgs"]).last() { assert_eq!(g.msg_text, s(&last[0]), "{w}: last message"); }
    }
}

#[test]
fn exit_door_title_omen_and_store_phases() {
    let m = &fx()["misc"];
    for c in arr(&m["door"]) {
        let mut g = bare();
        g.level_scp = c["scp"].as_str().map(|x| x.to_string());
        g.house_state.on = c["houseOn"].as_bool().unwrap();
        let w = format!("useExitDoor {:?} toStairs {} houseOn {}", g.level_scp, c["toStairs"], c["houseOn"]);
        let draws = with_seed(5, || { let b = seeded_draws(); g.use_exit_door(c["toStairs"].as_bool().unwrap()); seeded_draws() - b });
        let modes: Vec<&str> = arr(&c["modes"]).iter().map(s).collect();
        let msgs = arr(&c["msgs"]);
        match modes.as_slice() {
            ["next"] => assert_eq!(g.state, "transition", "{w}"),
            [] => { assert_eq!(g.msg_text, s(&msgs[0][0]), "{w}"); assert_eq!(draws as i64, c["draws"].as_i64().unwrap(), "{w}: draws"); assert_eq!(g.state, "playing"); }
            // enterStairwell / enterLoop (the ported glue): a transition too, but through
            // a black fade, where nextLevel fades to the next level's weather colour
            _ => {
                assert!(g.msg_text.is_empty(), "{w}");
                if g.state == "transition" { assert_eq!(g.dom.fade.style("background"), Some("#000"), "{w}: not the next level"); }
            }
        }
    }
    for t in arr(&m["title"]) {
        let mut g = Game::new_bare();
        let dominant: &'static str = Box::leak(s(&t[0]).to_string().into_boxed_str());
        g.layout = Some(game::archgen::Layout { w: 21, h: 21, cell: CELL, tiles: vec![0; 441], zone_of: vec![0; 441], tile_flags: vec![0; 441], zones: vec![], dominant, level: 1, scp: None });
        assert_eq!(g.level_title(), s(&t[1]), "levelTitle {dominant}");
    }
    // showOmen
    let mut g = bare();
    let o = &m["omen"];
    g.show_omen("THE LIGHTS ARE ON IN THE SHOP", 1.7, 7.0);
    ac(g.dom.omen.opacity().unwrap(), &o["opacity"], 0.0, "omen opacity");
    let mut at: Vec<f64> = g.timeouts.iter().map(|t| t.at_ms).collect();
    at.sort_by(|a, b| a.partial_cmp(b).unwrap());
    assert_eq!(at.len(), 2);
    for (a, b) in at.iter().zip(arr(&o["timers"])) { ac(*a, b, 1e-12, "omen timer"); }
    g.now_ms = at[0]; g.run_timeouts();
    assert_eq!(g.dom.omen.text, s(&o["after1"][0])); ac(g.dom.omen.opacity().unwrap(), &o["after1"][1], 0.0, "omen shown");
    g.now_ms = at[1]; g.run_timeouts();
    ac(g.dom.omen.opacity().unwrap(), &o["after2"], 0.0, "omen gone");
    // a second omen cancels the first one's timers
    g.show_omen("A", 1.0, 1.0); g.show_omen("B", 1.0, 1.0);
    assert_eq!(g.timeouts.len(), 2);
    g.show_omen("", 1.0, 1.0);
    assert_eq!(g.timeouts.len(), 2, "an empty omen does nothing");
    // onStorePhase
    let mut g = bare();
    for _ in 0..30 { g.panels.push(Panel::default()); }
    let mut a = Entity::new("staff", group()); a.state = "hunt".into(); a.lose_t = 3.0;
    let mut b = Entity::new("smiler", group()); b.state = "hunt".into(); b.lose_t = 3.0;
    g.entities.push(a); g.entities.push(b);
    with_seed(77, || {
        let base = seeded_draws();
        for j in arr(&m["phases"]) {
            let to = s(&j["to"]);
            g.on_store_phase("x", to);
            assert_eq!((seeded_draws() - base) as i64, j["draws"].as_i64().unwrap(), "{to}: draws");
            for (p, jp) in g.panels.iter().zip(arr(&j["panels"])) { ac(p.flicker, &jp[0], TOL, "flicker"); assert_eq!(p.switched_off, jp[1].as_bool().unwrap()); }
            assert_eq!(g.store_state.panels_off, j["panelsOff"].as_bool().unwrap());
            ac(g.shake_t, &j["shakeT"], 0.0, "shakeT");
            if let Some(last) = arr(&j["msgs"]).last() { assert_eq!(g.msg_text, s(&last[0])); }
            for (e, je) in g.entities.iter().zip(arr(&j["ents"])) { assert_eq!(e.state, s(&je[0])); ac(e.lose_t, &je[1], 0.0, "loseT"); }
        }
    });
}

/// The whole module on the game's own deps, with (muted) audio: placement,
/// spawning, many frames of the store's hours and the house's shadows, every
/// lamp put out, the music, the announcements, the heartbeat; the resets.
#[test]
fn anomalies_smoke() {
    for (scp, seed, size) in [("store", 3u32, 35), ("house", 4, 35), ("store", 5, 45), ("house", 6, 45)] {
        let mut m = Mulberry32::new(seed);
        let layout = game::archgen::generate_layout(&game::archgen::LayoutOpts { size, level: 6, cell: CELL, scp: Some(scp) }, &mut || m.next());
        let mut g = Game::new_bare();
        g.state = "playing".into();
        g.god_mode = true;
        let ac = AudioContext::new(); ac.set_muted(true);
        g.audio_init_with(ac);
        g.init_anomalies();
        g.w = layout.w; g.h = layout.h; g.tiles = layout.tiles.clone(); g.layout = Some(layout);
        g.level_num = 6; g.level_scp = Some(scp.into());
        for _ in 0..12 { g.panels.push(Panel::default()); }
        let mut d = vec![-1; (g.w * g.h) as usize];
        for i in 0..d.len() { if g.tiles[i] == 0 { d[i] = 99; } }
        let open: Vec<usize> = (0..g.tiles.len()).filter(|&i| g.tiles[i] == 0).collect();
        let ex = open[open.len() / 2];
        g.exit_data = Some(ExitData { pos: v3(g.wx(ex as i32 % g.w), 0, g.wz(ex as i32 / g.w)), ..Default::default() });
        with_seed(seed, || {
            g.store_reset(); g.house_reset();
            if scp == "store" { g.place_store(&d); g.spawn_staff(5, &d); } else { g.place_house(&d); g.spawn_shadows(4, &d); }
        });
        assert!(!g.entities.is_empty());
        let mut lamps_used = 0;
        for k in 0..2400 {
            let i = open[(k * 37) % open.len()];
            let (px, pz) = (g.wx(i as i32 % g.w), g.wz(i as i32 / g.w));
            g.player.pos = v3(px, EYE, pz);
            g.player.crouch = k % 7 < 3;
            g.player.flashlight_on = k % 300 > 200;
            if let Some(h) = g.store_hide_spots.get(k % 5) { if k % 11 == 0 { let p = h.obj.pos(); g.player.pos = v3(p.x, EYE, p.z); g.player.crouch = true; } }
            if k % 40 == 0 { g.player_field = Some(Rc::new(g.bfs(g.tx(px), g.tz(pz)))); }
            if scp == "house" && k > 600 && k % 50 == 0 {
                if let Some(i) = (0..g.interactables.len()).find(|&i| (g.interactables[i].enabled)(&g)) { use_interactable(&mut g, i); lamps_used += 1; }
            }
            let dt = if k % 13 == 0 { 0.3 } else { 1.0 / 20.0 };
            let t = k as f64 * dt;
            for i in 0..g.entities.len() {
                let p = g.entities[i].pos();
                let d = js_hypot2(p.x - g.player.pos.x, p.z - g.player.pos.z);
                let caught = match g.entities[i].kind.as_str() { "staff" => g.update_staff(i, dt, t, d), "shadow" => g.update_shadow(i, dt, t, d), _ => false };
                if caught { break; }
            }
            g.update_scp(dt);
            g.now_ms += dt * 1000.0;
            g.run_timeouts();
            for e in &g.entities { assert!(e.pos().x.is_finite() && e.pos().z.is_finite()); }
        }
        if scp == "store" {
            assert!(g.store_state.t > 150.0 && g.store_state.music.is_some() && g.store_music_timer.is_some());
            assert!(g.store_state.board_tex.is_some());
        } else {
            assert_eq!(lamps_used, g.house_state.lamps.len());
            assert!(g.house_state.out);
            assert!(g.entities.iter().all(|e| e.kind == "gone"), "every shadow came apart");
            assert!(g.house_exit_blocked() == false);
        }
        // useExitDoor and the title on the level
        let _ = g.level_title();
        g.use_exit_door(false);
        assert_eq!(g.state, "transition", "the door leads to the next level");
        g.timeouts.clear();   // (initLevel needs the rest of the game)
        g.store_reset(); g.house_reset();
        assert!(g.store_state.music.is_none() && g.store_music_timer.is_none() && g.store_hide_spots.is_empty() && g.player.hide_eye.is_none());
        assert!(!g.house_state.on && g.house_state.lamps.is_empty());
        g.now_ms += 5000.0; g.run_timeouts();
    }
}
