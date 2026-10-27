//! Differential tests of `game::world*` against recorded reference data
//! (fixtures in `tests/fixtures/`): the maze, the grid / zone / nav queries,
//! `gen_layout_level`, `build_regions`, `wall_adj_spot`, `biased_spawn_angle`,
//! `pan_for_world_pos`, `level_decay` and the placer themes.
use game::base::rng::{seeded_draws, with_seed};
use game::Game;
use serde_json::Value;

fn fixture(name: &str) -> Value {
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures").join(name);
    serde_json::from_str(&std::fs::read_to_string(p).expect("fixture")).expect("json")
}
fn f(v: &Value) -> f64 {
    v.as_f64().unwrap_or(f64::NAN)
}
fn i(v: &Value) -> i32 {
    v.as_i64().unwrap() as i32
}
fn close(a: f64, b: f64) -> bool {
    a == b || (a - b).abs() <= 1e-12 * a.abs().max(b.abs()).max(1.0)
}
fn tiles_str(g: &Game) -> String {
    g.tiles.iter().map(|t| char::from(b'0' + t)).collect()
}

/// the fixture's wallpaper metadata: 21 palettes × 4 grime levels, then 2 moss (palettes 5, 17)
pub fn set_wallpaper_meta(g: &mut Game) {
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

#[test]
fn maze_matches_reference() {
    let fx = fixture("world_queries.json");
    for c in fx["maze"].as_array().unwrap() {
        let mut g = Game::new_bare();
        let d0 = seeded_draws();
        with_seed(c["seed"].as_u64().unwrap() as u32, || g.gen_maze(i(&c["size"])));
        assert_eq!(seeded_draws() - d0, c["draws"].as_u64().unwrap(), "maze draws {}", c["size"]);
        assert_eq!(g.w, i(&c["w"]));
        assert_eq!(tiles_str(&g), c["tiles"].as_str().unwrap(), "maze tiles {}", c["size"]);
    }
}

#[test]
fn level_queries_match_reference() {
    let fx = fixture("world_queries.json");
    for lv in fx["levels"].as_array().unwrap() {
        let n = i(&lv["n"]);
        let seed = lv["seed"].as_u64().unwrap() as u32;
        let tag = format!("level {n} seed {seed}");
        let mut g = Game::new_bare();
        set_wallpaper_meta(&mut g);
        // (the recorded levels are the rooms' depths: the rooms now begin on level 4)
        g.level_num = n + game::level::building::BUILDING_FLOORS;
        g.level_scp = lv["scp"].as_str().map(|s| s.to_string());
        let d0 = seeded_draws();
        with_seed(seed, || g.gen_layout_level(n, i(&lv["size"])));
        assert_eq!(seeded_draws() - d0, lv["layoutDraws"].as_u64().unwrap(), "{tag}: layout draws");
        assert_eq!((g.w, g.h), (i(&lv["w"]), i(&lv["h"])));
        assert_eq!(tiles_str(&g), lv["tiles"].as_str().unwrap(), "{tag}: tiles");
        assert_eq!(g.solids.as_ref().unwrap().count as u64, lv["solidsCount"].as_u64().unwrap(), "{tag}: solids");
        let tc: Vec<f64> = g.tile_ceil.iter().map(|&c| c as f64).collect();
        assert!(close(tc.iter().sum::<f64>(), f(&lv["tileCeil"][1])), "{tag}: tile_ceil sum");
        // bfs
        for b in lv["bfs"].as_array().unwrap() {
            let d = g.bfs(i(&b["x"]), i(&b["z"]));
            let want: Vec<i32> = b["d"].as_array().unwrap().iter().map(i).collect();
            assert_eq!(d, want, "{tag}: bfs from {} {}", b["x"], b["z"]);
        }
        for r in lv["los"].as_array().unwrap() {
            let r = r.as_array().unwrap();
            assert_eq!(g.los(f(&r[0]), f(&r[1]), f(&r[2]), f(&r[3])), r[4].as_bool().unwrap(), "{tag}: los {:?}", r);
        }
        let circ = |g: &Game, key: &str| {
            for r in lv[key].as_array().unwrap() {
                let r = r.as_array().unwrap();
                assert_eq!(g.circle_hits_wall(f(&r[0]), f(&r[1]), f(&r[2])), r[3].as_bool().unwrap(), "{tag}: {key} {:?}", r);
            }
        };
        circ(&g, "circle");
        g.barricade_tiles = lv["barricades"].as_array().unwrap().iter().map(|v| v.as_u64().unwrap() as usize).collect();
        circ(&g, "circleBar");
        let sol = g.solids.take();
        circ(&g, "circleNoSolids");
        for r in lv["losNoSolids"].as_array().unwrap() {
            let r = r.as_array().unwrap();
            assert_eq!(g.los(f(&r[0]), f(&r[1]), f(&r[2]), f(&r[3])), r[4].as_bool().unwrap(), "{tag}: los (no solids) {:?}", r);
        }
        g.solids = sol;
        g.barricade_tiles.clear();
        for r in lv["alt"].as_array().unwrap() {
            let r = r.as_array().unwrap();
            let blk: Vec<usize> = r[4].as_array().unwrap().iter().map(|v| v.as_u64().unwrap() as usize).collect();
            assert_eq!(g.alt_path_exists(i(&r[0]), i(&r[1]), i(&r[2]), i(&r[3]), &blk), r[5].as_bool().unwrap(), "{tag}: alt {:?}", r);
        }
        for r in lv["collide"].as_array().unwrap() {
            let r = r.as_array().unwrap();
            let mut p = engine::math::Vec3::new(f(&r[0]), 0.0, f(&r[1]));
            g.collide_move(&mut p, f(&r[2]), f(&r[3]), f(&r[4]));
            assert!(p.x == f(&r[5]) && p.z == f(&r[6]), "{tag}: collide {:?} -> {} {}", r, p.x, p.z);
        }
        for r in lv["zone"].as_array().unwrap() {
            let r = r.as_array().unwrap();
            assert_eq!(g.zone_at_tile(i(&r[0]), i(&r[1])).map(|z| z as i64).unwrap_or(-1), r[2].as_i64().unwrap(), "{tag}: zone {:?}", r);
        }
        for r in lv["zoneW"].as_array().unwrap() {
            let r = r.as_array().unwrap();
            assert_eq!(
                g.zone_at_world(f(&r[0]), f(&r[1])).map(|z| z as i64).unwrap_or(-1),
                r[2].as_i64().unwrap(),
                "{tag}: zone_at_world {:?}",
                r
            );
            assert_eq!(g.ceil_at_world(f(&r[0]), f(&r[1])), f(&r[3]), "{tag}: ceil {:?}", r);
        }
        for r in lv["nav"].as_array().unwrap() {
            let r = r.as_array().unwrap();
            assert_eq!(g.nav_tile_at(f(&r[0]), f(&r[1])), (i(&r[2]), i(&r[3])), "{tag}: nav {:?}", r);
        }
        // regions
        let d1 = seeded_draws();
        with_seed(seed ^ 0x5bd1e995, || g.build_regions());
        assert_eq!(seeded_draws() - d1, lv["regionDraws"].as_u64().unwrap(), "{tag}: region draws");
        let rm: Vec<String> = g.region_map.iter().map(|v| v.to_string()).collect();
        assert_eq!(rm.join(","), lv["regionMap"].as_str().unwrap(), "{tag}: region_map");
        let cr: Vec<String> = g.ceil_region.iter().map(|v| v.to_string()).collect();
        assert_eq!(cr.join(","), lv["ceilRegion"].as_str().unwrap(), "{tag}: ceil_region");
        let ri = lv["regionInfo"].as_array().unwrap();
        assert_eq!(g.region_info.len(), ri.len(), "{tag}: region count");
        for (k, (a, b)) in g.region_info.iter().zip(ri).enumerate() {
            let b = b.as_array().unwrap();
            assert_eq!(
                (a.wall as i64, a.ceil as i64, a.floor as i64, a.flags as i64, a.grime as i64),
                (b[0].as_i64().unwrap(), b[1].as_i64().unwrap(), b[2].as_i64().unwrap(), b[3].as_i64().unwrap(), b[4].as_i64().unwrap()),
                "{tag}: region {k}"
            );
            assert_eq!(a.lit, b[5].as_f64(), "{tag}: region {k} lit");
        }
        for r in lv["regionAt"].as_array().unwrap() {
            let r = r.as_array().unwrap();
            let ri = g.region_at(i(&r[0]), i(&r[1]));
            assert_eq!(
                (ri.wall as i64, ri.flags as i64, g.region_flags_at(i(&r[0]), i(&r[1])) as i64),
                (r[2].as_i64().unwrap(), r[3].as_i64().unwrap(), r[4].as_i64().unwrap()),
                "{tag}: region_at {:?}",
                r
            );
        }
        // `wall_adj_spot`
        let d2 = seeded_draws();
        let spots = with_seed(seed + 77, || {
            lv["spots"].as_array().unwrap().iter().map(|r| g.wall_adj_spot(i(&r[0]), i(&r[1]))).collect::<Vec<_>>()
        });
        assert_eq!(seeded_draws() - d2, lv["spotDraws"].as_u64().unwrap(), "{tag}: spot draws");
        for (s, r) in spots.iter().zip(lv["spots"].as_array().unwrap()) {
            match (s, r[2].as_array()) {
                (None, None) => {}
                (Some(s), Some(w)) => {
                    let got = [s.pos.x, s.pos.y, s.pos.z, s.normal.x, s.normal.y, s.normal.z, s.yaw];
                    for k in 0..7 {
                        assert_eq!(got[k], f(&w[k]), "{tag}: spot {:?} [{k}]", r);
                    }
                    assert_eq!(s.normal.x.is_sign_negative() && s.normal.x == 0.0, w[7].as_i64() == Some(1), "{tag}: spot -0 x");
                    assert_eq!(s.normal.z.is_sign_negative() && s.normal.z == 0.0, w[8].as_i64() == Some(1), "{tag}: spot -0 z");
                }
                _ => panic!("{tag}: spot {:?} presence", r),
            }
        }
        let got = with_seed(seed + 99, || {
            lv["angles"]
                .as_array()
                .unwrap()
                .iter()
                .map(|r| {
                    g.player.yaw = r[0].as_f64().unwrap_or(f64::NAN);
                    g.biased_spawn_angle()
                })
                .collect::<Vec<_>>()
        });
        for (a, r) in got.iter().zip(lv["angles"].as_array().unwrap()) {
            assert_eq!(*a, f(&r[1]), "{tag}: angle {:?}", r);
        }
        for r in lv["pans"].as_array().unwrap() {
            let r = r.as_array().unwrap();
            g.player.pos = engine::math::Vec3::new(f(&r[0]), 0.0, f(&r[1]));
            g.player.yaw = f(&r[2]);
            // (sin / cos of the yaw: the platform libm and the recording's may differ in the last bit)
            let p = g.pan_for_world_pos(f(&r[3]), f(&r[4]));
            assert!((p - f(&r[5])).abs() < 1e-13, "{tag}: pan {:?} got {p}", r);
        }
    }
}

/// Rewrites the level records of `world_queries.json` from the current
/// layout generator and regions, keeping every query point:
///
///     cargo test --release -p game --test world_queries -- --ignored rerecord
#[test]
#[ignore = "rewrites the fixture"]
fn rerecord_level_queries() {
    use serde_json::json;
    let p = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/world_queries.json");
    let mut fx = fixture("world_queries.json");
    let num = |v: f64| if v.is_finite() { json!(v) } else { Value::Null };
    for lv in fx["levels"].as_array_mut().unwrap() {
        let n = i(&lv["n"]);
        let seed = lv["seed"].as_u64().unwrap() as u32;
        let mut g = Game::new_bare();
        set_wallpaper_meta(&mut g);
        // (the recorded levels are the rooms' depths: the rooms now begin on level 4)
        g.level_num = n + game::level::building::BUILDING_FLOORS;
        g.level_scp = lv["scp"].as_str().map(|s| s.to_string());
        let d0 = seeded_draws();
        with_seed(seed, || g.gen_layout_level(n, i(&lv["size"])));
        lv["layoutDraws"] = json!(seeded_draws() - d0);
        lv["w"] = json!(g.w);
        lv["h"] = json!(g.h);
        lv["tiles"] = json!(tiles_str(&g));
        lv["solidsCount"] = json!(g.solids.as_ref().unwrap().count);
        lv["tileCeil"] = json!([g.tile_ceil.len(), g.tile_ceil.iter().map(|&c| c as f64).sum::<f64>()]);
        lv["zones"] = json!(g.layout.as_ref().unwrap().zones.len());
        let rows = |lv: &Value, key: &str| lv[key].as_array().unwrap().iter().map(|r| r.as_array().unwrap().clone()).collect::<Vec<_>>();
        let bfs: Vec<Value> =
            lv["bfs"].as_array().unwrap().iter().map(|b| json!({ "x": b["x"], "z": b["z"], "d": g.bfs(i(&b["x"]), i(&b["z"])) })).collect();
        lv["bfs"] = json!(bfs);
        let los = |g: &Game, rs: Vec<Vec<Value>>| -> Value {
            json!(rs
                .into_iter()
                .map(|mut r| {
                    r[4] = json!(g.los(f(&r[0]), f(&r[1]), f(&r[2]), f(&r[3])));
                    r
                })
                .collect::<Vec<_>>())
        };
        let circ = |g: &Game, rs: Vec<Vec<Value>>| -> Value {
            json!(rs
                .into_iter()
                .map(|mut r| {
                    r[3] = json!(g.circle_hits_wall(f(&r[0]), f(&r[1]), f(&r[2])));
                    r
                })
                .collect::<Vec<_>>())
        };
        lv["los"] = los(&g, rows(lv, "los"));
        lv["circle"] = circ(&g, rows(lv, "circle"));
        g.barricade_tiles = lv["barricades"].as_array().unwrap().iter().map(|v| v.as_u64().unwrap() as usize).collect();
        lv["circleBar"] = circ(&g, rows(lv, "circleBar"));
        let sol = g.solids.take();
        lv["circleNoSolids"] = circ(&g, rows(lv, "circleNoSolids"));
        lv["losNoSolids"] = los(&g, rows(lv, "losNoSolids"));
        g.solids = sol;
        g.barricade_tiles.clear();
        let alt: Vec<Value> = rows(lv, "alt")
            .into_iter()
            .map(|mut r| {
                let blk: Vec<usize> = r[4].as_array().unwrap().iter().map(|v| v.as_u64().unwrap() as usize).collect();
                r[5] = json!(g.alt_path_exists(i(&r[0]), i(&r[1]), i(&r[2]), i(&r[3]), &blk));
                json!(r)
            })
            .collect();
        lv["alt"] = json!(alt);
        let collide: Vec<Value> = rows(lv, "collide")
            .into_iter()
            .map(|mut r| {
                let mut p = engine::math::Vec3::new(f(&r[0]), 0.0, f(&r[1]));
                g.collide_move(&mut p, f(&r[2]), f(&r[3]), f(&r[4]));
                r[5] = json!(p.x);
                r[6] = json!(p.z);
                json!(r)
            })
            .collect();
        lv["collide"] = json!(collide);
        let zone: Vec<Value> = rows(lv, "zone")
            .into_iter()
            .map(|mut r| {
                r[2] = json!(g.zone_at_tile(i(&r[0]), i(&r[1])).map(|z| z as i64).unwrap_or(-1));
                json!(r)
            })
            .collect();
        lv["zone"] = json!(zone);
        let zone_w: Vec<Value> = rows(lv, "zoneW")
            .into_iter()
            .map(|mut r| {
                r[2] = json!(g.zone_at_world(f(&r[0]), f(&r[1])).map(|z| z as i64).unwrap_or(-1));
                r[3] = num(g.ceil_at_world(f(&r[0]), f(&r[1])));
                json!(r)
            })
            .collect();
        lv["zoneW"] = json!(zone_w);
        let nav: Vec<Value> = rows(lv, "nav")
            .into_iter()
            .map(|mut r| {
                let (x, z) = g.nav_tile_at(f(&r[0]), f(&r[1]));
                r[2] = json!(x);
                r[3] = json!(z);
                json!(r)
            })
            .collect();
        lv["nav"] = json!(nav);
        let d1 = seeded_draws();
        with_seed(seed ^ 0x5bd1e995, || g.build_regions());
        lv["regionDraws"] = json!(seeded_draws() - d1);
        lv["regionMap"] = json!(g.region_map.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(","));
        lv["ceilRegion"] = json!(g.ceil_region.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(","));
        lv["regionInfo"] =
            json!(g.region_info.iter().map(|a| json!([a.wall, a.ceil, a.floor, a.flags, a.grime, a.lit])).collect::<Vec<_>>());
        let region_at: Vec<Value> = rows(lv, "regionAt")
            .into_iter()
            .map(|r| {
                let (x, z) = (i(&r[0]), i(&r[1]));
                let ri = g.region_at(x, z);
                json!([r[0], r[1], ri.wall, ri.flags, g.region_flags_at(x, z)])
            })
            .collect();
        lv["regionAt"] = json!(region_at);
        let d2 = seeded_draws();
        let pts = rows(lv, "spots");
        let spots = with_seed(seed + 77, || pts.iter().map(|r| g.wall_adj_spot(i(&r[0]), i(&r[1]))).collect::<Vec<_>>());
        lv["spotDraws"] = json!(seeded_draws() - d2);
        let spots: Vec<Value> = pts
            .iter()
            .zip(spots)
            .map(|(r, s)| match s {
                None => json!([r[0], r[1], Value::Null]),
                Some(s) => {
                    let neg = |v: f64| (v == 0.0 && v.is_sign_negative()) as i32;
                    // (zeros unsigned: the sign of a zero normal is in the last two fields)
                    let v = [s.pos.x, s.pos.y, s.pos.z, s.normal.x, s.normal.y, s.normal.z, s.yaw].map(|v| v + 0.0);
                    json!([r[0], r[1], [v[0], v[1], v[2], v[3], v[4], v[5], v[6], neg(s.normal.x), neg(s.normal.z)]])
                }
            })
            .collect();
        lv["spots"] = json!(spots);
        let yaws = rows(lv, "angles");
        let got = with_seed(seed + 99, || {
            yaws.iter()
                .map(|r| {
                    g.player.yaw = r[0].as_f64().unwrap_or(f64::NAN);
                    g.biased_spawn_angle()
                })
                .collect::<Vec<_>>()
        });
        lv["angles"] = json!(yaws.iter().zip(got).map(|(r, a)| json!([r[0], a])).collect::<Vec<_>>());
        let pans: Vec<Value> = rows(lv, "pans")
            .into_iter()
            .map(|mut r| {
                g.player.pos = engine::math::Vec3::new(f(&r[0]), 0.0, f(&r[1]));
                g.player.yaw = f(&r[2]);
                r[5] = json!(g.pan_for_world_pos(f(&r[3]), f(&r[4])) + 0.0);
                json!(r)
            })
            .collect();
        lv["pans"] = json!(pans);
    }
    std::fs::write(&p, serde_json::to_string(&fx).unwrap()).unwrap();
    eprintln!("rewrote {}", p.display());
}

#[test]
fn decay_and_themes_match_reference() {
    let fx = fixture("world_queries.json");
    let mut g = Game::new_bare();
    for r in fx["decay"].as_array().unwrap() {
        let n = r[0].as_i64().map(|v| v as i32);
        if n.is_none() {
            g.level_num = 6 + game::level::building::BUILDING_FLOORS;
        }
        assert_eq!(g.level_decay(n), f(&r[1]), "decay {:?}", r);
    }
    for r in fx["themes"].as_array().unwrap() {
        assert_eq!(g.placer_theme(r[0].as_str().unwrap()), r[1].as_str().unwrap(), "theme of {}", r[0]);
    }
}
