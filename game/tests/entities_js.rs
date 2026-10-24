//! The entities section (backrooms.js 61153–62850, game/src/entities*.rs)
//! against the original JS, run by tools/en_fixtures.mjs.
//!
//! The JS side runs with the JS's own no-solids grid versions of bfs / los /
//! circleHitsWall / collideMove / navTileAt / ceilAtWorld; here `GridNav` is a
//! port of the same functions installed through `set_entity_nav`, so the
//! comparison does not depend on world.rs's state. Trigonometry differs from
//! V8's in the last bit, so continuous quantities are compared with a
//! tolerance; draw counts, spawn results and discrete states exactly.
use game::entities::*;
use game::prelude::*;
use game::stairwell::js_hypot2;
use serde_json::Value;
use std::sync::OnceLock;

fn fx() -> &'static Value {
    static V: OnceLock<Value> = OnceLock::new();
    V.get_or_init(|| {
        let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/entities.json");
        let s = std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{}: {e} (run tools/en_fixtures.mjs)", p.display()));
        serde_json::from_str(&s).unwrap()
    })
}
fn f(v: &Value) -> f64 { v.as_f64().unwrap_or_else(|| if v.is_null() { f64::NAN } else if let Some(b) = v.as_bool() { b as i32 as f64 } else { panic!("not a number: {v}") }) }
fn fs(v: &Value) -> Vec<f64> { v.as_array().unwrap().iter().map(f).collect() }
fn close(a: f64, b: f64, tol: f64) -> bool { (a.is_nan() && b.is_nan()) || (a - b).abs() <= tol * (1.0 + a.abs().max(b.abs())) }
#[track_caller]
fn ck(a: f64, b: f64, tol: f64, what: &dyn Fn() -> String) { assert!(close(a, b, tol), "{}: ours {a} JS {b} (diff {:e})", what(), (a - b).abs()); }

// ------------------------------------------------------------ the grid
/// The JS grid versions of the world queries (backrooms.js 1476–1540,
/// 2534–2554, no solids, no barricades) plus the harness's lit / ceiling maps.
struct GridNav { lit: Vec<f64>, ceil: Vec<f32> }
impl EntityNav for GridNav {
    fn bfs(&self, g: &Game, sx: i32, sz: i32) -> Vec<i32> {
        let (w, h) = (g.w, g.h);
        let mut dist = vec![-1i32; (w * h) as usize];
        let (mut sx, mut sz) = (sx, sz);
        if g.is_wall_tile(sx, sz) {
            let mut found = false;
            for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (-1, 1), (1, -1), (-1, -1)] {
                if !g.is_wall_tile(sx + dx, sz + dz) { sx += dx; sz += dz; found = true; break; }
            }
            if !found { return dist; }
        }
        let mut q = vec![g.t_idx(sx, sz)];
        dist[q[0]] = 0;
        let mut qi = 0;
        while qi < q.len() {
            let cur = q[qi];
            let (cx, cz) = ((cur as i32) % w, (cur as i32) / w);
            let d = dist[cur] + 1;
            for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                let (nx, nz) = (cx + dx, cz + dz);
                if g.in_grid(nx, nz) && g.tiles[g.t_idx(nx, nz)] == 0 && dist[g.t_idx(nx, nz)] == -1 {
                    dist[g.t_idx(nx, nz)] = d;
                    q.push(g.t_idx(nx, nz));
                }
            }
            qi += 1;
        }
        dist
    }
    fn los(&self, g: &Game, ax: f64, az: f64, bx: f64, bz: f64) -> bool {
        let (dx, dz) = (bx - ax, bz - az);
        let d = js_hypot2(dx, dz);
        let steps = (d / 0.35).ceil();
        let mut i = 1.0;
        while i < steps {
            let t = i / steps;
            if g.is_wall_tile(g.tx(ax + dx * t), g.tz(az + dz * t)) { return false; }
            i += 1.0;
        }
        true
    }
    fn circle_hits_wall(&self, g: &Game, px: f64, pz: f64, r: f64) -> bool {
        let (x0, x1, z0, z1) = (g.tx(px - r), g.tx(px + r), g.tz(pz - r), g.tz(pz + r));
        for z in z0..=z1 {
            for x in x0..=x1 {
                if !g.is_wall_tile(x, z) { continue; }
                let cx = clamp(px, g.wx(x) - CELL / 2.0, g.wx(x) + CELL / 2.0);
                let cz = clamp(pz, g.wz(z) - CELL / 2.0, g.wz(z) + CELL / 2.0);
                if (px - cx) * (px - cx) + (pz - cz) * (pz - cz) < r * r { return true; }
            }
        }
        false
    }
    fn collide_move(&self, g: &Game, p: &mut Vec3, dx: f64, dz: f64, r: f64) {
        if dx != 0.0 && !self.circle_hits_wall(g, p.x + dx, p.z, r) { p.x += dx; }
        if dz != 0.0 && !self.circle_hits_wall(g, p.x, p.z + dz, r) { p.z += dz; }
    }
    fn nav_tile_at(&self, g: &Game, px: f64, pz: f64) -> (i32, i32) {
        let (x, z) = (g.tx(px), g.tz(pz));
        if !g.is_wall_tile(x, z) { return (x, z); }
        let mut best = None;
        let mut bd = 1e9;
        for dz in -1..=1 {
            for dx in -1..=1 {
                if g.is_wall_tile(x + dx, z + dz) { continue; }
                let d = (g.wx(x + dx) - px).powi(2) + (g.wz(z + dz) - pz).powi(2);
                if d < bd { bd = d; best = Some((x + dx, z + dz)); }
            }
        }
        best.unwrap_or((x, z))
    }
    fn ceil_at_world(&self, g: &Game, px: f64, pz: f64) -> f64 {
        let i = g.t_idx(clamp(g.tx(px) as f64, 0.0, (g.w - 1) as f64), clamp(g.tz(pz) as f64, 0.0, (g.h - 1) as f64));
        self.ceil[i] as f64
    }
    fn region_light_at(&self, g: &Game, x: i32, z: i32) -> f64 {
        let i = z as i64 * g.w as i64 + x as i64;
        if i < 0 { return 0.5; }
        self.lit.get(i as usize).copied().unwrap_or(0.5)
    }
}

/// A bare game on a fixture grid, with the grid's world queries installed.
fn game_on(grid: &Value) -> Game {
    let mut g = Game::new_bare();
    let n = grid["n"].as_i64().unwrap() as i32;
    g.w = n; g.h = n;
    g.tiles = grid["tiles"].as_array().unwrap().iter().map(|v| v.as_u64().unwrap() as u8).collect();
    let ceil: Vec<f32> = fs(&grid["ceil"]).iter().map(|v| *v as f32).collect();
    let lit = fs(&grid["lit"]);
    g.tile_ceil = ceil.clone();
    set_entity_nav(Some(Rc::new(GridNav { lit, ceil })));
    g.state = "playing".into();
    g.god_mode = true;
    g.now_ms = 12345.0;
    g
}

// ------------------------------------------------------------ tables
#[test]
fn tables_match_js() {
    let j = fx();
    let ours = sm_fingers();
    let js = j["smFingers"].as_array().unwrap();
    assert_eq!(ours.len(), js.len());
    for (a, b) in ours.iter().zip(js) {
        let b = b.as_array().unwrap();
        assert_eq!(a.len(), b.len());
        for (p, q) in a.iter().zip(b) { for k in 0..3 { ck(p[k], f(&q[k]), 1e-14, &|| "SM_FINGERS".into()); } }
    }
    let d = hd_digits();
    for (key, ours) in [("hand", &d.hand), ("foot", &d.foot)] {
        let js = j["hdDigits"][key].as_array().unwrap();
        assert_eq!(ours.len(), js.len());
        for (a, b) in ours.iter().zip(js) {
            let b = b.as_array().unwrap();
            assert_eq!(a.len(), b.len());
            for (p, q) in a.iter().zip(b) { for k in 0..3 { ck(p[k], f(&q[k]), 1e-14, &|| format!("HD_DIGITS.{key}")); } }
        }
    }
    assert_eq!(d.splay.to_vec(), fs(&j["hdDigits"]["splay"]));
    for r in j["grin"].as_array().unwrap() {
        let r = fs(r);
        let o = sm_grin_at(r[0]);
        for k in 0..3 { ck(o[k], r[k + 1], 1e-13, &|| format!("smGrinAt({})", r[0])); }
    }
    for r in j["wrapPi"].as_array().unwrap() { let r = fs(r); ck(wrap_pi(r[0]), r[1], 1e-15, &|| "wrapPi".into()); }
    for r in j["entSmooth"].as_array().unwrap() { let r = fs(r); assert_eq!(ent_smooth(0.2, 1.1, r[0]), r[1], "entSmooth({})", r[0]); }
}

#[test]
fn solve_limb_matches_js() {
    let cases = fx()["solveLimb"].as_array().unwrap();
    assert_eq!(cases.len(), 400);
    for c in cases {
        let c = fs(c);
        let (t1, t2) = solve_limb(c[0], c[1], c[2], c[3], c[4], c[5], c[6]);
        ck(t1, c[7], 1e-12, &|| format!("solveLimb t1 {c:?}"));
        ck(t2, c[8], 1e-12, &|| format!("solveLimb t2 {c:?}"));
    }
}

// ------------------------------------------------------------ builds
fn geo_id(g: &Geo) -> Vec<String> {
    for n in game::creatures::creature_part_names() {
        if let Some(c) = game::creatures::creature_geometry(n) { if c.ptr_eq(g) { return vec!["part".into(), n.to_string()]; } }
    }
    let wt = ent_assets();
    if wt.wt.body.ptr_eq(g) || (g.vertex_count() == wt.wt.body.vertex_count() && g.r().morph_position.len() == wt.wt.morphs && wt.wt.morphs > 0) { return vec!["part".into(), "wtBody".into()]; }
    match g.params() {
        GeoParams::Sphere { radius, ws, hs, .. } => vec!["sphere".into(), radius.to_string(), (ws as f64).to_string(), (hs as f64).to_string()],
        GeoParams::Plane { width, height, .. } => vec!["plane".into(), width.to_string(), height.to_string()],
        _ => vec![g.type_name().to_string()],
    }
}
fn js_geo_id(v: &Value) -> Vec<String> { v.as_array().unwrap().iter().map(|x| match x { Value::String(s) => s.clone(), Value::Number(n) => n.as_f64().unwrap().to_string(), _ => x.to_string() }).collect() }
fn check_mat(m: &Mat, js: &Value, tex: &dyn Fn(&Tex) -> Option<i64>, what: &str) {
    let r = m.r();
    let ty = match r.kind { MatKind::Basic => "MeshBasicMaterial", MatKind::Lambert => "MeshLambertMaterial", MatKind::Phong => "MeshPhongMaterial", _ => "?" };
    assert_eq!(ty, js["type"].as_str().unwrap(), "{what}: material type");
    assert_eq!(r.color.get_hex() as u64, js["color"].as_u64().unwrap(), "{what}: color");
    if let Some(e) = js["emissive"].as_u64() { assert_eq!(r.emissive.get_hex() as u64, e, "{what}: emissive"); }
    if let Some(e) = js["specular"].as_u64() { assert_eq!(r.specular.get_hex() as u64, e, "{what}: specular"); }
    if let Some(s) = js["shininess"].as_f64() { assert_eq!(r.shininess, s, "{what}: shininess"); }
    assert_eq!(r.vertex_colors, js["vc"].as_bool().unwrap(), "{what}: vertexColors");
    assert_eq!(r.transparent, js["tr"].as_bool().unwrap(), "{what}: transparent");
    assert_eq!(r.opacity, f(&js["op"]), "{what}: opacity");
    assert_eq!(r.depth_write, js["dw"].as_bool().unwrap(), "{what}: depthWrite");
    assert_eq!(r.fog, js["fog"].as_bool().unwrap(), "{what}: fog");
    assert_eq!(r.visible, js["vis"].as_bool().unwrap(), "{what}: visible");
    assert_eq!(r.map.as_ref().and_then(tex), js["map"].as_i64(), "{what}: map");
    assert_eq!(r.bump_map.as_ref().and_then(tex), js["bump"].as_i64(), "{what}: bumpMap");
    if js["bump"].as_i64().is_some() { assert_eq!(r.bump_scale, f(&js["bumpScale"]), "{what}: bumpScale"); }
    let key = match &r.patch {
        Patch::SkinDetail { scale, strength, .. } => { assert_eq!(*strength, r.bump_scale); format!("skinDetail{}", num_str(*scale)) }
        Patch::WatcherSheet { scale, strength, .. } => { assert_eq!((*scale, *strength), (3.1, 0.85)); "watcherSheet".into() }
        Patch::SmilerEye => "smilerEye".into(),
        _ => String::new(),
    };
    assert_eq!(key, js["key"].as_str().unwrap(), "{what}: the shader patch");
}
fn check_tree(o: &Obj, js: &Value, tex: &dyn Fn(&Tex) -> Option<i64>, path: &str) {
    let kind = if matches!(o.r().kind, engine::scene::Kind::Skinned { .. }) { "skinned" } else if o.is_mesh() { "mesh" } else if o.is_bone() { "bone" } else if matches!(o.r().kind, engine::scene::Kind::Group) { "group" } else { "object3d" };
    let path = format!("{path}/{}", o.name());
    assert_eq!(kind, js["k"].as_str().unwrap(), "{path}: kind");
    assert_eq!(o.name(), js["name"].as_str().unwrap(), "{path}: name");
    let (p, q, s) = (o.pos(), o.quat(), o.scl());
    let jp = fs(&js["p"]);
    for (k, v) in [p.x, p.y, p.z].iter().enumerate() { ck(*v, jp[k], 1e-15, &|| format!("{path}: position")); }
    let jq = fs(&js["q"]);
    for (k, v) in [q.x, q.y, q.z, q.w].iter().enumerate() { ck(*v, jq[k], 1e-12, &|| format!("{path}: quaternion {:?} vs {jq:?}", [q.x, q.y, q.z, q.w])); }
    assert_eq!(vec![s.x, s.y, s.z], fs(&js["s"]), "{path}: scale");
    let order = format!("{:?}", o.rot().order);
    assert_eq!(order, js["order"].as_str().unwrap(), "{path}: rotation order");
    {
        let u = &o.r().user;
        match js.get("tail") { Some(t) => assert_eq!(vec![u.get("tailX"), u.get("tailY"), u.get("tailZ")], fs(t), "{path}: tail"), None => assert!(!u.has("tailX"), "{path}: tail") }
        assert_eq!(u.s.get("main").map(|s| s.as_str()), js.get("main").and_then(|v| v.as_str()), "{path}: main");
    }
    if o.is_mesh() {
        assert_eq!(geo_id(&o.geo()), js_geo_id(&js["g"]), "{path}: geometry");
        check_mat(&o.mat(), &js["m"], tex, &path);
        let inf = o.r().morph_influences.clone();
        match js.get("inf") { Some(i) => assert_eq!(inf, fs(i), "{path}: morph influences"), None => assert!(inf.is_empty(), "{path}: influences") }
    }
    if kind == "skinned" {
        let n = o.r();
        let engine::scene::Kind::Skinned { skeleton: Some(sk), bind_matrix, .. } = &n.kind else { panic!("{path}: no skeleton") };
        let names: Vec<String> = sk.bones.iter().map(|b| b.name()).collect();
        let jn: Vec<String> = js["bones"].as_array().unwrap().iter().map(|v| v.as_str().unwrap().to_string()).collect();
        assert_eq!(names, jn, "{path}: skeleton");
        let jb = fs(&js["bind"]);
        for k in 0..16 { ck(bind_matrix.e[k], jb[k], 1e-12, &|| format!("{path}: bindMatrix")); }
        for (i, (inv, ji)) in sk.bone_inverses.iter().zip(js["inv"].as_array().unwrap()).enumerate() {
            let ji = fs(ji);
            for k in 0..16 { ck(inv.e[k], ji[k], 1e-9, &|| format!("{path}: boneInverses[{i}] ({})", names[i])); }
        }
    }
    let kids = o.children();
    let jk = js.get("c").and_then(|v| v.as_array()).map(|v| v.as_slice()).unwrap_or(&[]);
    assert_eq!(kids.len(), jk.len(), "{path}: children");
    for (c, jc) in kids.iter().zip(jk) { check_tree(c, jc, tex, &path); }
}

#[test]
fn builds_match_js() {
    let j = fx();
    let mut g = game_on(&j["grid"]);
    g.level_num = 3;
    let a = ent_assets();
    let ti = &j["texIndex"];
    let texs = [(game::entities::skin_detail_tex(), "skinDetail"), (a.watcher_cloth_tex.clone(), "cloth"), (a.smiler_face_tex.clone(), "smilerFace"),
        (a.smiler_halo_tex.clone(), "smilerHalo"), (a.hound_face_tex.clone(), "houndFace")];
    let tex = |t: &Tex| texs.iter().find(|(x, _)| x.0.as_ptr() == t.0.as_ptr()).map(|(_, k)| ti[*k].as_i64().unwrap());
    // the texture sizes and repeat flags, in the JS's creation order
    let sizes = j["textures"].as_array().unwrap();
    for (t, k) in &texs {
        let js = &sizes[ti[*k].as_u64().unwrap() as usize];
        assert_eq!((t.width() as f64, t.height() as f64), (f(&js[0]), f(&js[1])), "{k}: size");
        assert_eq!(t.r().wrap_s == Wrap::Repeat, js[2].as_bool().unwrap(), "{k}: repeat");
    }
    assert_eq!(game::entities::skin_detail_tex().r().color_space, ColorSpace::NoColorSpace);
    assert_eq!(a.watcher_cloth_tex.r().color_space, ColorSpace::NoColorSpace);
    let m = &j["mats"];
    for (mat, k) in [(&a.smiler_skin, "smilerSkin"), (&a.hound_skin, "houndSkin"), (&a.watcher_sheet, "watcherSheet"), (&a.hound_hair, "houndHair"),
        (&a.smiler_tooth, "smilerTooth"), (&a.smiler_maw, "smilerMaw"), (&a.hound_tooth, "houndTooth"), (&a.hound_eye, "houndEye"), (&smiler_eye_material(), "smilerEye")] {
        check_mat(mat, &m[k], &tex, k);
    }
    let d0 = seeded_draws();
    with_seed(4242, || {
        for kind in ["smiler", "hound", "watcher"] {
            let b = &j["builds"][kind];
            let (x, z) = (f(&b["x"]), f(&b["z"]));
            let built = match kind { "hound" => g.make_hound(x, z), "watcher" => g.make_watcher(x, z), _ => g.make_smiler(x, z) };
            check_tree(&built.g, &b["tree"], &tex, kind);
            check_tree(built.face.as_ref().unwrap(), &b["face"], &tex, &format!("{kind}.face"));
            assert_eq!(built.face_y, f(&b["faceY"]));
            assert!(b["inScene"].as_i64().unwrap() >= 0 && g.scene.children().iter().any(|c| c.ptr_eq(built.face.as_ref().unwrap())), "{kind}: face in scene");
            assert!(g.world.children().iter().any(|c| c.ptr_eq(&built.g)), "{kind}: in world");
            assert!(built.parts.contains_key("head"), "{kind}: parts.head");
        }
    });
    assert_eq!(seeded_draws() - d0, j["builds"]["draws"].as_u64().unwrap(), "the builds draw nothing");
}

// ------------------------------------------------------------ spawnEntities (seeded: exact)
#[test]
fn spawn_matches_js() {
    for c in fx()["spawn"].as_array().unwrap() {
        let n = c["n"].as_i64().unwrap() as i32;
        // the same grid as the fixture's GRID_A(n)
        let grid = grid_a(n);
        let mut g = game_on(&grid);
        g.level_num = c["level"].as_i64().unwrap() as i32;
        let dist = g.e_bfs_pub(1, 1);
        assert_eq!(dist.iter().map(|v| *v as f64).collect::<Vec<_>>(), fs(&c["dist"]), "bfs(1, 1) on grid {n}");
        let d0 = seeded_draws();
        with_seed(c["seed"].as_u64().unwrap() as u32, || g.spawn_entities(c["count"].as_i64().unwrap() as i32, &dist));
        assert_eq!(seeded_draws() - d0, c["draws"].as_u64().unwrap(), "draws (level {}, count {})", c["level"], c["count"]);
        let js = c["ents"].as_array().unwrap();
        assert_eq!(g.entities.len(), js.len());
        for (e, je) in g.entities.iter().zip(js) {
            assert_eq!(e.kind, je["type"].as_str().unwrap());
            assert_eq!(e.pos().x, f(&je["x"])); assert_eq!(e.pos().z, f(&je["z"]));
            assert_eq!(e.yaw, f(&je["yaw"]), "yaw");
            assert_eq!(e.speed, f(&je["speed"])); assert_eq!(e.freeze, f(&je["freeze"]));
            assert_eq!(e.eye_y_offset, f(&je["eyeY"])); assert_eq!(e.face_y, f(&je["faceY"]));
        }
        // their faces in the scene, their bodies in the world
        assert_eq!(g.scene.children().len(), g.entities.len());
    }
}

/// `GRID_A(n)` of the harness
fn grid_a(n: i32) -> Value {
    let mut tiles = vec![]; let mut ceil = vec![]; let mut lit = vec![];
    for z in 0..n {
        for x in 0..n {
            let wall = x == 0 || z == 0 || x == n - 1 || z == n - 1 || (x % 6 == 3 && z % 6 == 3) || (x == 9 && z > 2 && z < n - 3 && z != 8);
            tiles.push(if wall { 1 } else { 0 });
            ceil.push(if (x + 2 * z) % 7 == 0 { 2.4f32 as f64 } else { 3.1f32 as f64 });
            lit.push(((x * 7 + z * 3) % 5) as f64 / 4.0);
        }
    }
    serde_json::json!({ "n": n, "tiles": tiles, "ceil": ceil, "lit": lit })
}

// ------------------------------------------------------------ animateEntity
fn pose(o: &Obj) -> Vec<Vec<f64>> {
    let mut out = vec![];
    o.traverse(&mut |c| { let (q, p, s) = (c.quat(), c.pos(), c.scl()); out.push(vec![q.x, q.y, q.z, q.w, p.x, p.y, p.z, s.x, s.y, s.z]); });
    out
}

#[test]
fn animate_matches_js() {
    let j = fx();
    for c in j["anim"].as_array().unwrap() {
        let kind = c["type"].as_str().unwrap();
        let s = c["seed"].as_u64().unwrap() as u32;
        let mut g = game_on(&grid_a(23));
        g.level_num = 4;
        let d0 = seeded_draws();
        with_seed(s, || {
            let ei = g.add_entity(kind, 11, 11, 0);
            for (k, fr) in c["frames"].as_array().unwrap().iter().enumerate() {
                let inp = &fr["inp"];
                let (dt, t) = (f(&inp["dt"]), f(&inp["t"]));
                {
                    let e = &mut g.entities[ei];
                    e.g.position().x = f(&inp["ex"]); e.g.position().z = f(&inp["ez"]);
                    e.yaw = f(&inp["yaw"]); e.state = inp["state"].as_str().unwrap().into();
                    e.set_n("holdT", f(&inp["holdT"])); e.set_n("lungeT", f(&inp["lungeT"])); e.freeze = f(&inp["freeze"]);
                    match f(&inp["observed"]) as i32 { -1 => { e.flag.remove("observed"); } v => { e.set_fl("observed", v == 1); e.observed = v == 1; } }
                }
                g.player.pos = v3(f(&inp["px"]), 0.0, f(&inp["pz"]));
                g.player.yaw = f(&inp["pyaw"]);
                let p = g.entities[ei].pos();
                let d = js_hypot2(p.x - g.player.pos.x, p.z - g.player.pos.z);
                assert_eq!(d, f(&inp["d"]));
                g.animate_entity(ei, dt, t, d);
                let what = |x: &str| format!("{kind} seed {s} frame {k}: {x}");
                assert_eq!(seeded_draws() - d0, fr["draws"].as_u64().unwrap(), "{}", what("draws"));
                let Some(jp) = fr.get("pose") else { continue };
                let ours = pose(&g.entities[ei].g);
                let jp = jp.as_array().unwrap();
                assert_eq!(ours.len(), jp.len(), "{}", what("nodes"));
                for (ni, (a, b)) in ours.iter().zip(jp).enumerate() {
                    let b = fs(b);
                    for q in 0..10 { ck(a[q], b[q], 2e-7, &|| what(&format!("node {ni} component {q}: {a:?} vs {b:?}"))); }
                }
                let e = &g.entities[ei];
                for (key, v) in fr["num"].as_object().unwrap() {
                    let ours = e.num.get(key).copied().unwrap_or_else(|| panic!("{}", what(&format!("e.{key} missing"))));
                    ck(ours, f(v), 1e-8, &|| what(&format!("e.{key}")));
                }
                for key in e.num.keys() {
                    if ["holdT", "lungeT"].contains(&key.as_str()) { continue; }
                    assert!(fr["num"].get(key).is_some(), "{}", what(&format!("e.{key} is not set in JS")));
                }
                if let Some(pt) = fr.get("poseTaken") { assert_eq!(e.fl("poseTaken"), pt.as_bool().unwrap(), "{}", what("poseTaken")); }
                if let Some(tic) = fr.get("tic") {
                    let id = e.g.id();
                    let tc = g.ent_ext[&id].tic.unwrap();
                    let ours: Vec<f64> = tc.p.iter().chain(&tc.v).chain(&tc.g).copied().chain([tc.next]).collect();
                    for (q, (a, b)) in ours.iter().zip(fs(tic)).enumerate() { ck(*a, b, 1e-8, &|| what(&format!("tic {q}"))); }
                }
                let rig = g.entity_rig(ei).unwrap();
                let (halo, far, eye, sheet) = match &*rig {
                    Rig::Smiler(p) => (Some(p.halo.clone()), Some(p.far.clone()), Some(p.eye_mat.clone()), None),
                    Rig::Hound(p) => (None, Some(p.far.clone()), None, None),
                    Rig::Watcher(p) => (None, None, None, Some(p.sheet.clone())),
                };
                for (m, key) in [(halo, "halo"), (far, "far")] {
                    if let Some(m) = m {
                        let jv = fs(&fr[key]);
                        ck(m.opacity_v(), jv[0], 1e-8, &|| what(key));
                        assert_eq!(m.r().visible, jv[1] != 0.0, "{}", what(&format!("{key}.visible")));
                    }
                }
                if let Some(m) = eye { ck(m.emissive_v().r, f(&fr["eye"]), 1e-8, &|| what("eye emissive")); }
                if let Some(sh) = sheet {
                    let inf = sh.r().morph_influences.clone();
                    let jv = fs(&fr["inf"]);
                    assert_eq!(inf.len(), jv.len());
                    for (a, b) in inf.iter().zip(&jv) { ck(*a, *b, 1e-8, &|| what("influences")); }
                    // the colours follow the influences (three's morphcolor_vertex, on the CPU)
                    let a = ent_assets();
                    let gg = sh.geo();
                    let col = &gg.r().color;
                    for i in (0..col.len()).step_by(997) {
                        let mut want = a.wt.base_color[i];
                        for (kk, mc) in a.wt.morph_color.iter().enumerate() { if inf[kk] != 0.0 { want += mc[i] * inf[kk] as f32; } }
                        assert_eq!(col[i], want, "{}", what("blended colour"));
                    }
                }
                ck(g.entities[ei].g.rot().y, f(&fr["gry"]), 1e-9, &|| what("g.rotation.y"));
                ck(g.entities[ei].g.scl().y, f(&fr["gsy"]), 1e-12, &|| what("g.scale.y"));
            }
        });
    }
}

// ------------------------------------------------------------ steering
#[test]
fn steering_matches_js() {
    let j = &fx()["steer"];
    let mut g = game_on(&grid_a(23));
    g.level_num = 2;
    for r in j["lineWalkable"].as_array().unwrap() {
        let r = fs(r);
        assert_eq!(g.line_walkable(r[0], r[1], r[2], r[3], r[4]), r[5] != 0.0, "lineWalkable {r:?}");
    }
    // the grid port agrees with the JS queries
    let nav = GridNavProbe::new(&g);
    for r in j["queries"].as_array().unwrap() {
        let r = fs(r);
        assert_eq!(nav.los(r[0], r[1], r[2], r[3]), r[5] != 0.0, "los {r:?}");
        assert_eq!(nav.hits(r[0], r[1], r[4]), r[6] != 0.0, "circleHitsWall {r:?}");
        assert_eq!(nav.tile(r[0], r[1]), (r[7] as i32, r[8] as i32), "navTileAt {r:?}");
        assert_eq!(nav.ceil(r[0], r[1]), r[9], "ceilAtWorld {r:?}");
        let mut p = v3(r[0], 0.0, r[1]);
        nav.mv(&mut p, (r[2] - r[0]) * 0.1, (r[3] - r[1]) * 0.1, r[4]);
        assert_eq!((p.x, p.z), (r[10], r[11]), "collideMove {r:?}");
    }
    for b in j["bfs"].as_array().unwrap() {
        let (x, z) = (b[0].as_i64().unwrap() as i32, b[1].as_i64().unwrap() as i32);
        assert_eq!(g.e_bfs_pub(x, z).iter().map(|v| *v as f64).collect::<Vec<_>>(), fs(&b[2]), "bfs({x}, {z})");
    }
    // entityTurn
    with_seed(56, || {
        let ei = g.add_entity("smiler", 4, 4, 0);
        for r in j["turns"].as_array().unwrap() {
            let r = fs(r);
            g.entities[ei].yaw = r[0];
            g.entities[ei].kind = ["smiler", "hound", "watcher", "staff", "shadow", "gone"][r[4] as usize].into();
            g.entity_turn(ei, r[1], r[2], if r[3] < 0.0 { None } else { Some(r[3]) });
            ck(g.entities[ei].yaw, r[5], 1e-14, &|| format!("entityTurn {r:?}"));
        }
    });
    // walks
    for w in j["walks"].as_array().unwrap() {
        let mut g = game_on(&grid_a(23));
        g.level_num = 2;
        let kind = w["type"].as_str().unwrap();
        let (sx, sz, gx, gz) = (w["sx"].as_i64().unwrap() as i32, w["sz"].as_i64().unwrap() as i32, w["gx"].as_i64().unwrap() as i32, w["gz"].as_i64().unwrap() as i32);
        let direct = w["direct"].as_i64().unwrap() != 0;
        let speed = f(&w["speed"]);
        with_seed(w["seed"].as_u64().unwrap() as u32, || {
            let ei = g.add_entity(if kind == "staff" || kind == "shadow" { "smiler" } else { kind }, sx, sz, 0);
            g.entities[ei].kind = kind.into();   // staff / shadow steer with their own motion constants
            g.entities[ei].freeze = 0.0;
            let field = Rc::new(g.e_bfs_pub(gx, gz));
            let mut field2 = field.clone();
            for (k, st) in w["steps"].as_array().unwrap().iter().enumerate() {
                let st = fs(st);
                if k == 200 { field2 = Rc::new(g.e_bfs_pub(gx, gz)); }
                g.entities[ei].set_n("lungeT", st[3]);
                if kind == "hound" { g.entities[ei].state = if st[4] != 0.0 { "hunt".into() } else { "wander".into() }; }
                let done = g.entity_step_toward(ei, Some(field2.clone()), st[0], speed, if direct { Some(v3(st[1], 0.0, st[2])) } else { None });
                let e = &g.entities[ei];
                let what = |x: &str| format!("walk {kind} {sx},{sz}→{gx},{gz} step {k}: {x}");
                let ours = [e.pos().x, e.pos().z, e.vx, e.vz, e.yaw, e.n("aimX"), e.n("aimZ")];
                for (q, v) in ours.iter().enumerate() { ck(*v, st[5 + q], 1e-7, &|| what(&format!("value {q} ({ours:?})"))); }
                assert_eq!(e.fl("aimFinal"), st[12] != 0.0, "{}", what("aimFinal"));
                assert_eq!(e.fl("aimOk"), st[13] != 0.0, "{}", what("aimOk"));
                ck(e.n("aimT"), st[14], 1e-12, &|| what("aimT"));
                assert_eq!(done, st[15] != 0.0, "{}", what("done"));
            }
        });
    }
    // look rules
    for l in j["looks"].as_array().unwrap() {
        let mut g = game_on(&grid_a(23));
        g.level_num = 2;
        let kind = l["type"].as_str().unwrap();
        with_seed(l["seed"].as_u64().unwrap() as u32, || {
            let ei = g.add_entity(kind, 11, 11, 0);
            for (k, st) in l["steps"].as_array().unwrap().iter().enumerate() {
                let st = fs(st);
                g.player.pos = v3(st[0], 0.0, st[1]); g.player.yaw = st[2]; g.player.audible_range = st[3];
                let m = g.entity_look_rule(ei, st[4], st[5] != 0.0, 1.0 / 60.0);
                let e = &g.entities[ei];
                let what = |x: &str| format!("look {kind} step {k}: {x}");
                assert_eq!(m, st[6], "{}", what("multiplier"));
                ck(e.n("lungeT"), st[7], 1e-12, &|| what("lungeT"));
                ck(e.n("holdT"), st[8], 1e-12, &|| what("holdT"));
                ck(e.n("stareCd"), st[9], 1e-12, &|| what("stareCd"));
                assert_eq!(e.fl("wasWatched"), st[10] != 0.0, "{}", what("wasWatched"));
            }
        });
    }
}

/// The grid queries through the same `GridNav` the entities use.
struct GridNavProbe<'a> { g: &'a Game, n: GridNav }
impl<'a> GridNavProbe<'a> {
    fn new(g: &'a Game) -> Self { let gr = grid_a(g.w); GridNavProbe { g, n: GridNav { lit: fs(&gr["lit"]), ceil: fs(&gr["ceil"]).iter().map(|v| *v as f32).collect() } } }
    fn los(&self, a: f64, b: f64, c: f64, d: f64) -> bool { self.n.los(self.g, a, b, c, d) }
    fn hits(&self, x: f64, z: f64, r: f64) -> bool { self.n.circle_hits_wall(self.g, x, z, r) }
    fn tile(&self, x: f64, z: f64) -> (i32, i32) { self.n.nav_tile_at(self.g, x, z) }
    fn ceil(&self, x: f64, z: f64) -> f64 { self.n.ceil_at_world(self.g, x, z) }
    fn mv(&self, p: &mut Vec3, dx: f64, dz: f64, r: f64) { self.n.collide_move(self.g, p, dx, dz, r) }
}
trait BfsPub { fn e_bfs_pub(&self, x: i32, z: i32) -> Vec<i32>; }
impl BfsPub for Game { fn e_bfs_pub(&self, x: i32, z: i32) -> Vec<i32> { let gr = grid_a(self.w); GridNav { lit: fs(&gr["lit"]), ceil: vec![] }.bfs(self, x, z) } }

// ------------------------------------------------------------ updateEntities
#[test]
fn update_matches_js() {
    for c in fx()["update"].as_array().unwrap() {
        let mut g = game_on(&c["grid"]);
        g.level_num = c["level"].as_i64().unwrap() as i32;
        let wk = c["weather"].as_str().unwrap();
        g.level_weather = game::state::WEATHER_BIASES.iter().find(|w| w.key == wk).unwrap();
        g.player.pos = v3(g.wx(2), 0.0, g.wz(2));
        g.player.yaw = 0.0;
        g.shake_t = 0.0; g.hum_pulse_phase = 0.0; g.hum_pulse_amp_current = 0.015;
        g.knock_flash_t = 0.5; g.door_opened_flash_t = 0.7; g.elevator_ding_flash_t = 1.0; g.voice_sample_flash_t = 0.6; g.static_burst_flash_t = 0.4;
        g.reality_shift_t = 1.2; g.pattern_break_t = 2.0; g.clock_skip_t = 0.3; g.chime_flash_t = 0.8; g.chime_flash_color = Some("rgba(200, 180, 90".into());
        g.static_chromatic_t = 0.3; g.electric_surge_t = 0.2; g.cry_dim_lerp = 0.5; g.current_seed = 0x12345678;
        let seed = c["seed"].as_u64().unwrap() as u32;
        let d0 = seeded_draws();
        with_seed(seed, || {
            let dist = g.e_bfs_pub(1, 1);
            g.spawn_entities(3, &dist);
            assert_eq!(seeded_draws() - d0, c["spawnDraws"].as_u64().unwrap(), "spawn draws");
            let types: Vec<String> = g.entities.iter().map(|e| e.kind.clone()).collect();
            let jt: Vec<String> = c["types"].as_array().unwrap().iter().map(|v| v.as_str().unwrap().to_string()).collect();
            assert_eq!(types, jt);
            match c["place"].as_array() {
                Some(pl) => for (i, xz) in pl.iter().enumerate() { let (x, z) = (g.wx(f(&xz[0])), g.wz(f(&xz[1]))); g.entities[i].g.set_pos(x, 0.0, z); },
                None => { let (x5, z3) = (g.wx(5), g.wz(3)); g.entities[0].g.set_pos(x5, 0.0, z3); }
            }
            for (k, fr) in c["frames"].as_array().unwrap().iter().enumerate() {
                let inp = fs(&fr["inp"]);
                let (dt, t) = (inp[0], inp[1]);
                g.player.pos = v3(inp[2], 0.0, inp[3]);
                g.player.yaw = inp[4]; g.player.audible_range = inp[5]; g.player.flashlight_on = inp[6] != 0.0;
                if k == 450 { g.player.sanity = 20.0; }
                g.camera.set_pos(inp[2], 1.62, inp[3]);
                g.perf_t = t;
                g.update_entities(dt, t);
                let what = |x: &str| format!("update L{} seed {seed} frame {k}: {x}", c["level"]);
                assert_eq!(seeded_draws() - d0, fr["draws"].as_u64().unwrap(), "{}", what("draws"));
                let jents = fr["ents"].as_array().unwrap();
                assert!(fr["full"].as_i64() == Some(0) || jents.len() == g.entities.len());
                for (i, (e, je)) in g.entities.iter().zip(jents).enumerate() {
                    let je = fs(je);
                    let fp = e.face.as_ref().unwrap().pos();
                    let ours = [e.pos().x, e.pos().z, e.yaw, (e.state == "hunt") as i32 as f64, e.lose_t, e.freeze, e.stare_t, e.w_timer, e.unseen_t, e.vx, e.vz, e.step_acc, fp.x, fp.y, fp.z];
                    for (q, v) in ours.iter().enumerate() { ck(*v, je[q], 1e-6, &|| what(&format!("entity {i} ({}) value {q}: {ours:?}\n vs {je:?}", e.kind))); }
                }
                ck(g.player.sanity, f(&fr["san"]), 1e-8, &|| what("sanity"));
                ck(g.dom.vignette.opacity().unwrap_or(f64::NAN), f(&fr["vig"]), 1e-8, &|| what("vignette opacity"));
                ck(g.heart_t, f(&fr["heart"]), 1e-8, &|| what("heartT"));
                ck(g.shake_t, f(&fr["shake"]), 0.0, &|| what("shakeT"));
                ck(g.god_deaths, f(&fr["deaths"]), 0.0, &|| what("deaths"));
                ck(g.hum_pulse_phase, f(&fr["hum"]), 1e-10, &|| what("humPulsePhase"));
                assert_eq!(g.dom.vignette.style("filter").unwrap_or(""), fr["filter"].as_str().unwrap(), "{}", what("filter"));
                assert_eq!(g.dom.vignette.style("boxShadow").unwrap_or(""), fr["box"].as_str().unwrap(), "{}", what("boxShadow"));
                assert_eq!(g.dom.flash.opacity(), fr["flash"].as_f64(), "{}", what("flash"));
            }
        });
    }
}
