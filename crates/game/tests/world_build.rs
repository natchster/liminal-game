//! Whole-`build_world()` differential against recorded reference data
//! (fixtures in `tests/fixtures/`): `gen_layout_level` + `build_world` under
//! one seed with every `themed()` placer replaced by the same probe in the
//! recording and here. Checks the draw count and every trace mark (one per
//! `themed()` pass and per lap), the regions, panels, pool plan, theme /
//! settle / placement stats, the masked tiles each placer saw, and every world
//! child after `settle_props`, `settle_placement`, the architecture meshes,
//! `lay_panels` and instancing.
use engine::prelude::*;
use game::base::rng::{rnd, seeded_draws, with_seed};
use game::Game;
use serde_json::{json, Value};
use std::cell::RefCell;

fn fixture(name: &str) -> Value {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures").join(name);
    serde_json::from_str(&std::fs::read_to_string(p).expect("fixture")).expect("json")
}
fn f(v: &Value) -> f64 {
    v.as_f64().unwrap_or(f64::NAN)
}
fn set_wallpaper_meta(g: &mut Game) {
    let mut k = 0;
    for p in 0..21 {
        for gl in 0..4 {
            let v = &mut g.tex.wallpaper_variants[k];
            v.grime_lvl = gl;
            v.moss = false;
            v.palette_idx = p;
            k += 1;
        }
    }
    for (j, p) in [5usize, 17].iter().enumerate() {
        let v = &mut g.tex.wallpaper_variants[84 + j];
        v.grime_lvl = 3;
        v.moss = true;
        v.palette_idx = *p;
    }
}

const PROBE_SIZE: [[f64; 3]; 4] = [[0.4, 0.3, 0.4], [0.5, 0.4, 0.05], [0.3, 0.1, 0.3], [0.8, 2.3, 0.4]];
const PROBE_Y: [f64; 4] = [0.15, 1.5, 3.02, 1.15];
struct Probe {
    log: Vec<usize>,
    geos: Vec<Geo>,
    mats: Vec<Mat>,
}
thread_local! {
    static PROBE: RefCell<Probe> = RefCell::new(Probe {
        log: vec![],
        geos: PROBE_SIZE.iter().map(|s| box_geo!(s[0], s[1], s[2])).collect(),
        mats: [0x884422, 0x226688, 0xcccccc, 0x553311].iter().map(|&c| lambert!({ color: c })).collect(),
    });
}
/// the probe placer (identical to the one the fixtures were recorded with)
fn probe(g: &mut Game, _dist: &[i32]) {
    let masked = g.tiles.iter().filter(|&&t| t == 2).count();
    let open: Vec<usize> = (0..g.tiles.len()).filter(|&i| g.tiles[i] == 0).collect();
    PROBE.with(|p| p.borrow_mut().log.push(masked));
    let r = rnd();
    if open.is_empty() || r >= 0.36 {
        return;
    }
    let w = g.w;
    let pick = || {
        let t = open[(rnd() * open.len() as f64).floor() as usize] as i32;
        (t % w, t / w)
    };
    let (geos, mats) = PROBE.with(|p| {
        let p = p.borrow();
        (p.geos.clone(), p.mats.clone())
    });
    if r < 0.3 {
        let (x, z) = pick();
        let kind = (rnd() * 4.0).floor() as usize;
        let m = mesh(&geos[kind], &mats[kind]);
        let spot = if kind == 0 { None } else { g.wall_adj_spot(x, z) };
        if let Some(s) = spot {
            let mut p = s.pos;
            p.add_scaled_vector(&s.normal, PROBE_SIZE[kind][2] / 2.0);
            m.position().copy(&p);
            m.rotation().y = s.yaw;
        } else {
            let px = g.wx(x) + (rnd() - 0.5) * 3.0;
            let pz = g.wz(z) + (rnd() - 0.5) * 3.0;
            m.position().set(px, 0, pz);
            m.rotation().y = rnd() * 6.0;
        }
        m.position().y = PROBE_Y[kind];
        g.world.add(&m);
        if r < 0.04 {
            g.animate(|_, _, _| {});
        }
    } else if r < 0.34 {
        let gr = group();
        for _ in 0..2 {
            let (x, z) = pick();
            let m = mesh(&geos[0], &mats[0]);
            let px = g.wx(x) + (rnd() - 0.5) * 3.6;
            let pz = g.wz(z) + (rnd() - 0.5) * 3.6;
            m.position().set(px, 0.15, pz);
            gr.add(&m);
        }
        g.world.add(&gr);
    } else {
        let (x, z) = pick();
        let l = point_light!(0xffeecc, 1, 6, 2);
        l.position().set(g.wx(x), 2.2, g.wz(z));
        g.world.add(&l);
    }
}

fn mat_name(g: &Game, m: &Mat) -> String {
    for (k, a) in &g.arch_mats {
        if a.ptr_eq(m) {
            return k.clone();
        }
    }
    for (i, a) in g.floor_mats.iter().enumerate() {
        if a.ptr_eq(m) {
            return format!("carpet:{i}");
        }
    }
    for (i, a) in g.ceil_mats.iter().enumerate() {
        if a.ptr_eq(m) {
            return format!("tiles:{i}");
        }
    }
    if g.mats.metal.ptr_eq(m) {
        return "metal".into();
    }
    let r = m.r();
    format!("{}:{}", game::world::instancing::material_type(r.kind), r.color.get_hex_string())
}

/// `gen_layout_level` + `build_world` of level `n` under `seed`, every themed
/// placer replaced by the probe. Returns the game, its trace marks (draws
/// since the start) and draw count.
fn build_case(n: i32, seed: u32, scp: Option<&str>) -> (Game, Vec<(String, u64)>, u64) {
    let mut g = Game::new_bare();
    g.test_stub_core_placers = true; // the recording stubs `place_props` / `place_store` / `place_house`
    g.test_keep_prop_looks = true; // and has the probe props' materials as placed
    set_wallpaper_meta(&mut g);
    for name in game::world::tables::THEMED_ORDER {
        g.placers.push((name, probe));
    }
    g.init_world();
    // (the recorded levels are the rooms' depths: the rooms now begin on level 4)
    g.level_num = n + game::level::building::BUILDING_FLOORS;
    g.level_scp = scp.map(|s| s.to_string());
    g.rand_trace.on = true;
    PROBE.with(|p| p.borrow_mut().log.clear());
    let d0 = seeded_draws();
    with_seed(seed, || {
        g.trace_mark("start");
        g.gen_layout_level(n, (31 + n * 4).min(53));
        g.trace_mark("layout");
        g.build_world();
        g.trace_mark("world");
    });
    let marks: Vec<(String, u64)> = g.rand_trace.marks.iter().map(|(l, k)| (l.clone(), k - d0)).collect();
    let draws = seeded_draws() - d0;
    (g, marks, draws)
}

#[test]
fn build_world_matches_reference() {
    let fx = fixture("world_build.json");
    let mut worst = 0f64;
    let mut worst_cull = 0f64;
    for b in fx["builds"].as_array().unwrap() {
        let n = b["n"].as_i64().unwrap() as i32;
        let seed = b["seed"].as_u64().unwrap() as u32;
        let tag = format!("level {n} {:?}", b["scp"].as_str());
        let (g, marks, draws) = build_case(n, seed, b["scp"].as_str());
        // marks first: they say where a divergence starts
        let want: Vec<(String, u64)> =
            b["marks"].as_array().unwrap().iter().map(|m| (m[0].as_str().unwrap().to_string(), m[1].as_u64().unwrap())).collect();
        for (k, (a, w)) in marks.iter().zip(&want).enumerate() {
            assert_eq!(a, w, "{tag}: trace mark {k}");
        }
        assert_eq!(marks.len(), want.len(), "{tag}: mark count");
        assert_eq!(draws, b["draws"].as_u64().unwrap(), "{tag}: draws");
        let log = PROBE.with(|p| p.borrow().log.clone());
        let wl: Vec<usize> = b["probeLog"].as_array().unwrap().iter().map(|v| v.as_u64().unwrap() as usize).collect();
        assert_eq!(log, wl, "{tag}: masked tiles per placer run");
        let tiles: String = g.tiles.iter().map(|t| char::from(b'0' + t)).collect();
        assert_eq!(tiles, b["tiles"].as_str().unwrap(), "{tag}: tiles");
        let tc: Vec<f64> = g.tile_ceil.iter().map(|&c| c as f64).collect();
        let wtc: Vec<f64> = b["tileCeil"].as_array().unwrap().iter().map(f).collect();
        assert_eq!(tc, wtc, "{tag}: tile_ceil");
        assert_eq!(g.solids.as_ref().unwrap().count as u64, b["solidsCount"].as_u64().unwrap(), "{tag}: solids");
        for (k, (a, r)) in g.region_info.iter().zip(b["regionInfo"].as_array().unwrap()).enumerate() {
            assert_eq!(
                (a.wall as i64, a.ceil as i64, a.floor as i64, a.flags as i64, a.grime as i64, a.lit),
                (
                    r[0].as_i64().unwrap(),
                    r[1].as_i64().unwrap(),
                    r[2].as_i64().unwrap(),
                    r[3].as_i64().unwrap(),
                    r[4].as_i64().unwrap(),
                    r[5].as_f64()
                ),
                "{tag}: region {k}"
            );
        }
        let wp = b["panels"].as_array().unwrap();
        assert_eq!(g.panels.len(), wp.len(), "{tag}: panel count");
        for (k, (p, w)) in g.panels.iter().zip(wp).enumerate() {
            let got = [
                p.wx,
                p.wz,
                p.bright,
                p.seed,
                p.rot,
                p.lfo_freq,
                p.lfo_phase,
                p.y,
                p.h_boost,
                p.f.get("ceil").copied().unwrap_or(f64::NAN),
            ];
            for j in 0..10 {
                assert_eq!(got[j], f(&w[j]), "{tag}: panel {k} field {j}");
            }
        }
        let pp = g.pool_plan.map(|p| vec![p.x as i64, p.z as i64, p.pw as i64, p.ph as i64]);
        assert_eq!(pp, b["poolPlan"].as_array().map(|a| a.iter().map(|v| v.as_i64().unwrap()).collect()), "{tag}: pool plan");
        assert_eq!(
            [g.spawn_pos.x, g.spawn_pos.y, g.spawn_pos.z],
            [f(&b["spawn"][0]), f(&b["spawn"][1]), f(&b["spawn"][2])],
            "{tag}: spawn"
        );
        let ts = &b["themeStats"];
        assert_eq!(
            (g.theme_stats_ran as u64, g.theme_stats_skipped as u64),
            (ts[0].as_u64().unwrap(), ts[1].as_u64().unwrap()),
            "{tag}: theme stats"
        );
        for e in ts[2].as_array().unwrap() {
            let got = g.theme_stats_by_theme.get(e[0].as_str().unwrap()).copied().unwrap_or((0, 0));
            assert_eq!((got.0 as u64, got.1 as u64), (e[1].as_u64().unwrap(), e[2].as_u64().unwrap()), "{tag}: theme {}", e[0]);
        }
        assert_eq!(g.theme_stats_by_theme.len(), ts[2].as_array().unwrap().len(), "{tag}: themes seen");
        for (k, v) in b["settle"].as_object().unwrap() {
            assert_eq!(g.settle_stats.get(k).copied(), v.as_f64(), "{tag}: settle {k}");
        }
        for (k, v) in b["placement"].as_object().unwrap() {
            assert_eq!(g.placement_stats.get(k).copied(), v.as_f64(), "{tag}: placement {k}");
        }
        assert_eq!(g.build_profile.get("frozen").copied(), b["frozen"].as_f64(), "{tag}: frozen");
        for (k, v) in b["instancing"].as_object().unwrap() {
            assert_eq!(g.build_profile.get(&format!("instancing.{k}")).copied(), v.as_f64(), "{tag}: instancing {k}");
        }
        // the world, child by child
        let kids = g.world.children();
        let wk = b["children"].as_array().unwrap();
        assert_eq!(kids.len(), wk.len(), "{tag}: world children");
        for (ci, (o, w)) in kids.iter().zip(wk).enumerate() {
            let k = if o.is_instanced() {
                "inst"
            } else if o.is_mesh() {
                "mesh"
            } else if o.is_light() {
                "light"
            } else if o.is_group() {
                "group"
            } else {
                "obj"
            };
            assert_eq!(k, w["k"].as_str().unwrap(), "{tag}: child {ci} kind");
            let u = o.r().user.clone();
            assert_eq!(u.placer.as_deref(), w["placer"].as_str(), "{tag}: child {ci} placer");
            assert_eq!(
                (u.static_prop, u.arch, u.instanced),
                (w["sp"].as_bool().unwrap(), w["arch"].as_bool().unwrap(), w["ins"].as_bool().unwrap()),
                "{tag}: child {ci} flags"
            );
            assert_eq!(o.visible(), w["vis"].as_bool().unwrap(), "{tag}: child {ci} visible");
            assert_eq!(o.child_count() as u64, w["nc"].as_u64().unwrap(), "{tag}: child {ci} children");
            if o.is_mesh() {
                assert_eq!(mat_name(&g, &o.mat()), w["mat"].as_str().unwrap(), "{tag}: child {ci} material");
                let geo = o.geo();
                let gd = &w["geo"];
                match gd[0].as_str().unwrap() {
                    "box" => {
                        let p = geo.params();
                        assert_eq!([p.width(), p.height(), p.depth()], [f(&gd[1]), f(&gd[2]), f(&gd[3])], "{tag}: child {ci} box");
                    }
                    "plane" => {
                        let p = geo.params();
                        assert_eq!([p.width(), p.height()], [f(&gd[1]), f(&gd[2])], "{tag}: child {ci} plane");
                    }
                    t => panic!("{tag}: child {ci} geometry {t}"),
                }
            }
            if o.is_instanced() {
                let cnt = o.instance_count();
                assert_eq!(cnt as u64, w["count"].as_u64().unwrap(), "{tag}: child {ci} count");
                let (mut s, mut ws) = (0f64, 0f64);
                for i in 0..cnt {
                    let e = o.get_matrix_at(i).e;
                    for j in 0..16 {
                        let v = e[j] as f32 as f64;
                        s += v;
                        ws += v * (((i * 16 + j) % 7) + 1) as f64;
                    }
                }
                let d = ((s - f(&w["sum"])).abs()).max((ws - f(&w["wsum"])).abs());
                worst = worst.max(d);
                assert!(d < 1e-3, "{tag}: child {ci} instance matrices sum {s} vs {} / {ws} vs {}", w["sum"], w["wsum"]);
                let first = o.get_matrix_at(0).e;
                let last = o.get_matrix_at(cnt - 1).e;
                for j in 0..16 {
                    assert!((first[j] as f32 as f64 - f(&w["first"][j])).abs() < 1e-5, "{tag}: child {ci} first matrix [{j}]");
                    assert!((last[j] as f32 as f64 - f(&w["last"][j])).abs() < 1e-5, "{tag}: child {ci} last matrix [{j}]");
                }
            } else {
                let p = o.pos();
                let q = o.quat();
                for (j, (a, e)) in [p.x, p.y, p.z].iter().zip(w["p"].as_array().unwrap()).enumerate() {
                    worst = worst.max((a - f(e)).abs());
                    assert!((a - f(e)).abs() < 1e-9, "{tag}: child {ci} position [{j}] {a} vs {e}");
                }
                for (j, (a, e)) in [q.x, q.y, q.z, q.w].iter().zip(w["q"].as_array().unwrap()).enumerate() {
                    assert!((a - f(e)).abs() < 1e-9, "{tag}: child {ci} quaternion [{j}]");
                }
                for (c, e) in o.children().iter().zip(w["c"].as_array().unwrap()) {
                    let cp = c.pos();
                    for (a, e) in [cp.x, cp.y, cp.z].iter().zip(e.as_array().unwrap()) {
                        worst = worst.max((a - f(e)).abs());
                        assert!((a - f(e)).abs() < 1e-9, "{tag}: child {ci} grandchild position");
                    }
                }
            }
        }
        let wc = b["cull"].as_array().unwrap();
        assert_eq!(g.prop_cull.list.len(), wc.len(), "{tag}: cull list");
        for (k, ((o, r), w)) in g.prop_cull.list.iter().zip(wc).enumerate() {
            let idx = kids.iter().position(|c| c.ptr_eq(o)).map(|i| i as i64).unwrap_or(-1);
            assert_eq!(idx, w[0].as_i64().unwrap(), "{tag}: cull {k} object");
            let (x, z) = g.prop_cull_xz[k];
            for (a, e) in [x, z, *r].iter().zip(&w.as_array().unwrap()[1..]) {
                worst_cull = worst_cull.max((a - f(e)).abs());
                assert!((a - f(e)).abs() < 1e-4, "{tag}: cull {k} sphere {a} vs {e}");
            }
        }
        eprintln!("{tag}: {} draws, {} marks, {} children, {} panels — ok", draws, marks.len(), kids.len(), g.panels.len());
    }
    eprintln!("worst |Δ| {worst:e} (positions), {worst_cull:e} (cull spheres)");
}

/// The world of one `world_settle.json` case: `settle_props` once per run
/// (props back at their recorded spots, the low ceilings, with or without
/// the pool), handing `check` the game, the run and the objects after each.
fn settle_case(c: &Value, check: &mut dyn FnMut(&Game, &Value, &[Obj])) {
    let n = c["n"].as_i64().unwrap() as i32;
    let mut g = Game::new_bare();
    // (the recorded levels are the rooms' depths: the rooms now begin on level 4)
    g.level_num = n + game::level::building::BUILDING_FLOORS;
    with_seed(c["seed"].as_u64().unwrap() as u32, || g.gen_layout_level(n, (31 + n * 4).min(53)));
    let lows: Vec<(usize, f32)> =
        c["lows"].as_array().unwrap().iter().map(|l| (l[0].as_u64().unwrap() as usize, f(&l[1]) as f32)).collect();
    let base = g.tile_ceil.clone();
    let mat = lambert!({ color: 0x226688 });
    let mut objs = vec![];
    for e in c["spec"].as_array().unwrap() {
        let kind = e["kind"].as_str().unwrap();
        let o = match kind {
            "light" => point_light!(0xffffff, 1, 5, 2),
            "group" | "grouprot" => {
                let gr = group();
                for k in e["kids"].as_array().unwrap() {
                    let m = mesh(box_geo!(f(&k[3]), f(&k[4]), f(&k[5])), &mat);
                    m.position().set(f(&k[0]), f(&k[1]), f(&k[2]));
                    gr.add(&m);
                }
                gr
            }
            _ => mesh(box_geo!(f(&e["w"]), f(&e["h"]), f(&e["d"])), &mat),
        };
        o.position().set(f(&e["p"][0]), f(&e["p"][1]), f(&e["p"][2]));
        o.rotation().y = f(&e["ry"]);
        g.world.add(&o);
        objs.push(o);
    }
    for p in c["ints"].as_array().unwrap() {
        g.interact(Vec3::new(f(&p[0]), 1.6, f(&p[1])), 1.0, "", |_| true, |_| {});
    }
    let pb = c["pool"].as_array().unwrap();
    let spec = c["spec"].as_array().unwrap();
    for run in c["runs"].as_array().unwrap() {
        for (k, o) in objs.iter().enumerate() {
            o.position().set(f(&spec[k]["p"][0]), f(&spec[k]["p"][1]), f(&spec[k]["p"][2]));
            for (j, ch) in o.children().iter().enumerate() {
                let kd = &spec[k]["kids"][j];
                ch.position().set(f(&kd[0]), f(&kd[1]), f(&kd[2]));
            }
        }
        g.tile_ceil = base.clone();
        for &(t, v) in &lows {
            g.tile_ceil[t] = v;
        }
        g.pool_bounds = if run["pool"].as_bool().unwrap() {
            Some(game::base::records::PoolBounds {
                min_x: f(&pb[0]),
                max_x: f(&pb[1]),
                min_z: f(&pb[2]),
                max_z: f(&pb[3]),
                water_y: 0.08,
                splash_y: 0.1,
                tiles: [0; 4],
            })
        } else {
            None
        };
        g.settle_props(0);
        check(&g, run, &objs);
    }
}

/// `settle_props` on synthetic props (floor / wall / ceiling / tall / in-wall
/// boxes, lights, groups at the origin, a rotated group), low ceilings,
/// interactables to pin faces near, with and without a pool
#[test]
fn settle_props_matches_reference() {
    let fx = fixture("world_settle.json");
    let mut worst = 0f64;
    for c in fx["cases"].as_array().unwrap() {
        let n = c["n"].as_i64().unwrap() as i32;
        let spec = c["spec"].as_array().unwrap();
        settle_case(c, &mut |g, run, objs| {
            let tag = format!("settle level {n} pool {}", run["pool"]);
            for (k, v) in run["stats"].as_object().unwrap() {
                assert_eq!(g.settle_stats.get(k).copied(), v.as_f64(), "{tag}: {k}");
            }
            assert_eq!(g.solids.as_ref().unwrap().count as u64, run["solids"].as_u64().unwrap(), "{tag}: solids");
            let tc: Vec<f64> = g.tile_ceil.iter().map(|&v| v as f64).collect();
            let wtc: Vec<f64> = run["tileCeil"].as_array().unwrap().iter().map(f).collect();
            assert_eq!(tc, wtc, "{tag}: tile_ceil");
            for (k, (o, w)) in objs.iter().zip(run["pos"].as_array().unwrap()).enumerate() {
                let p = o.pos();
                for (j, (a, e)) in [p.x, p.y, p.z].iter().zip(w.as_array().unwrap()).enumerate() {
                    worst = worst.max((a - f(e)).abs());
                    assert!((a - f(e)).abs() < 1e-9, "{tag}: object {k} ({}) [{j}] {a} vs {e}", spec[k]["kind"]);
                }
                for (ch, cw) in o.children().iter().zip(w[3].as_array().unwrap()) {
                    let cp = ch.pos();
                    for (a, e) in [cp.x, cp.y, cp.z].iter().zip(cw.as_array().unwrap()) {
                        worst = worst.max((a - f(e)).abs());
                        assert!((a - f(e)).abs() < 1e-9, "{tag}: object {k} child");
                    }
                }
            }
            eprintln!("{tag}: ok ({:?})", g.settle_stats);
        });
    }
    eprintln!("settle worst |Δ| {worst:e}");
}

// ------------------------------------------------------------------ re-recording
//
// After a deliberate change to the world build, re-record the fixtures from
// the current code (the cases' inputs are kept) and review the difference:
//
//     cargo test --release -p game --test world_build -- --ignored rerecord

fn fixture_path(name: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures").join(name)
}
/// A JSON number, or null for NaN / infinities
fn jf(v: f64) -> Value {
    if v.is_finite() {
        json!(v)
    } else {
        Value::Null
    }
}

/// One `world_build.json` record of a built level
fn build_record(g: &Game, n: i32, seed: u32, scp: Option<&str>, marks: &[(String, u64)], draws: u64) -> Value {
    let kids = g.world.children();
    let children: Vec<Value> = kids
        .iter()
        .map(|o| {
            let k = if o.is_instanced() {
                "inst"
            } else if o.is_mesh() {
                "mesh"
            } else if o.is_light() {
                "light"
            } else if o.is_group() {
                "group"
            } else {
                "obj"
            };
            let u = o.r().user.clone();
            let mut c = json!({ "k": k, "placer": u.placer, "sp": u.static_prop, "arch": u.arch, "ins": u.instanced, "vis": o.visible(), "nc": o.child_count() });
            if o.is_mesh() {
                c["mat"] = json!(mat_name(g, &o.mat()));
                let p = o.geo().params();
                c["geo"] = match p.type_name() {
                    "box" => json!(["box", p.width(), p.height(), p.depth()]),
                    "plane" => json!(["plane", p.width(), p.height()]),
                    t => json!([t]),
                };
            }
            if o.is_instanced() {
                let cnt = o.instance_count();
                let (mut s, mut ws) = (0f64, 0f64);
                for i in 0..cnt {
                    let e = o.get_matrix_at(i).e;
                    for j in 0..16 {
                        let v = e[j] as f32 as f64;
                        s += v;
                        ws += v * (((i * 16 + j) % 7) + 1) as f64;
                    }
                }
                let m = |i: usize| o.get_matrix_at(i).e.iter().map(|&v| v as f32 as f64).collect::<Vec<_>>();
                c["count"] = json!(cnt);
                c["sum"] = json!(s);
                c["wsum"] = json!(ws);
                c["first"] = json!(m(0));
                c["last"] = json!(m(cnt - 1));
            } else {
                let p = o.pos();
                let q = o.quat();
                c["p"] = json!([p.x, p.y, p.z]);
                c["q"] = json!([q.x, q.y, q.z, q.w]);
                c["c"] = json!(o
                    .children()
                    .iter()
                    .map(|ch| {
                        let cp = ch.pos();
                        json!([cp.x, cp.y, cp.z])
                    })
                    .collect::<Vec<_>>());
            }
            c
        })
        .collect();
    let cull: Vec<Value> = g
        .prop_cull
        .list
        .iter()
        .enumerate()
        .map(|(k, (o, r))| {
            let idx = kids.iter().position(|c| c.ptr_eq(o)).map(|i| i as i64).unwrap_or(-1);
            let (x, z) = g.prop_cull_xz[k];
            json!([idx, x, z, r])
        })
        .collect();
    let panels: Vec<Value> = g
        .panels
        .iter()
        .map(|p| {
            json!([
                p.wx,
                p.wz,
                p.bright,
                p.seed,
                p.rot,
                p.lfo_freq,
                p.lfo_phase,
                p.y,
                p.h_boost,
                jf(p.f.get("ceil").copied().unwrap_or(f64::NAN))
            ])
        })
        .collect();
    let by_theme: Vec<Value> = g.theme_stats_by_theme.iter().map(|(k, v)| json!([k, v.0, v.1])).collect();
    let stats = |m: &std::collections::BTreeMap<String, f64>| -> Value {
        json!(m.iter().map(|(k, v)| (k.clone(), jf(*v))).collect::<serde_json::Map<_, _>>())
    };
    let prof = |k: &str| g.build_profile.get(k).copied().map(jf).unwrap_or(Value::Null);
    json!({
        "n": n, "seed": seed, "scp": scp, "draws": draws,
        "marks": marks.iter().map(|(l, k)| json!([l, k])).collect::<Vec<_>>(),
        "tiles": g.tiles.iter().map(|t| char::from(b'0' + t)).collect::<String>(),
        "tileCeil": g.tile_ceil.iter().map(|&c| c as f64).collect::<Vec<_>>(),
        "solidsCount": g.solids.as_ref().unwrap().count,
        "regionInfo": g.region_info.iter().map(|a| json!([a.wall, a.ceil, a.floor, a.flags, a.grime, a.lit])).collect::<Vec<_>>(),
        "panels": panels,
        "poolPlan": g.pool_plan.map(|p| json!([p.x, p.z, p.pw, p.ph])),
        "spawn": [g.spawn_pos.x, g.spawn_pos.y, g.spawn_pos.z],
        "themeStats": [g.theme_stats_ran, g.theme_stats_skipped, by_theme],
        "settle": stats(&g.settle_stats),
        "placement": stats(&g.placement_stats),
        "probeLog": PROBE.with(|p| p.borrow().log.clone()),
        "frozen": prof("frozen"),
        "instancing": {
            "meshes": prof("instancing.meshes"), "instanced": prof("instancing.instanced"),
            "buckets": prof("instancing.buckets"), "removedGroups": prof("instancing.removedGroups"),
        },
        "children": children,
        "cull": cull,
    })
}

/// Rewrites `world_build.json`: the same levels (n, seed, anomaly), built now
#[test]
#[ignore = "rewrites the fixture"]
fn rerecord_world_build() {
    let fx = fixture("world_build.json");
    let mut builds = vec![];
    for b in fx["builds"].as_array().unwrap() {
        let (n, seed, scp) = (b["n"].as_i64().unwrap() as i32, b["seed"].as_u64().unwrap() as u32, b["scp"].as_str());
        let (g, marks, draws) = build_case(n, seed, scp);
        builds.push(build_record(&g, n, seed, scp, &marks, draws));
    }
    std::fs::write(fixture_path("world_build.json"), serde_json::to_string(&json!({ "builds": builds })).unwrap()).unwrap();
}

/// Rewrites the runs of `world_settle.json` (the props, ceilings, pool and
/// interactables of each case are kept)
#[test]
#[ignore = "rewrites the fixture"]
fn rerecord_world_settle() {
    let mut fx = fixture("world_settle.json");
    for c in fx["cases"].as_array_mut().unwrap() {
        let mut runs = vec![];
        settle_case(c, &mut |g, run, objs| {
            let pos: Vec<Value> = objs
                .iter()
                .map(|o| {
                    let p = o.pos();
                    let kids: Vec<Value> = o
                        .children()
                        .iter()
                        .map(|ch| {
                            let cp = ch.pos();
                            json!([cp.x, cp.y, cp.z])
                        })
                        .collect();
                    json!([p.x, p.y, p.z, kids])
                })
                .collect();
            let stats: serde_json::Map<String, Value> = g.settle_stats.iter().map(|(k, v)| (k.clone(), jf(*v))).collect();
            runs.push(json!({
                "pool": run["pool"], "stats": stats, "solids": g.solids.as_ref().unwrap().count,
                "tileCeil": g.tile_ceil.iter().map(|&v| v as f64).collect::<Vec<_>>(), "pos": pos,
            }));
        });
        c["runs"] = json!(runs);
    }
    std::fs::write(fixture_path("world_settle.json"), serde_json::to_string(&fx).unwrap()).unwrap();
}
