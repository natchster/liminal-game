//! The entities (`game::entities`) against recorded reference data (fixtures
//! in `tests/fixtures/`).
//!
//! The recording used no-solids grid versions of bfs / los /
//! `circle_hits_wall` / `collide_move` / `nav_tile_at` / `ceil_at_world`; here
//! `GridNav` implements the same functions, installed through
//! `set_entity_nav`, so the comparison does not depend on world.rs's state.
//! The recorded trigonometry differs from the platform's in the last bit, so
//! continuous quantities are compared with a tolerance; draw counts, spawn
//! results and discrete states exactly.
//!
//! What is left of it (E7: the old creatures' builds, materials, animation,
//! look rules and update went with them) is what every creature still
//! shares: the steering, the limb solver the anomalies' staff keep, and two
//! small tables (the spawn is the ecology's now, `tests/ecology.rs`). `REGEN=1 cargo test
//! --release -p game --test entities` prunes the fixture to the sections
//! read here; review its diff before committing it.
use game::base::numeric::hypot2;
use game::entities::*;
use game::prelude::*;
use serde_json::Value;
use std::sync::OnceLock;

fn fixture_path() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/entities.json")
}
/// Re-recording instead of comparing (`REGEN=1`)
fn regen() -> bool {
    std::env::var_os("REGEN").is_some()
}
/// `REGEN=1`: the fixture keeps only the sections these tests read.
fn prune() {
    let p = fixture_path();
    let all: Value = serde_json::from_str(&std::fs::read_to_string(&p).unwrap()).unwrap();
    let mut out = serde_json::Map::new();
    for k in ["entSmooth", "solveLimb", "steer", "wrapPi"] {
        out.insert(k.into(), all[k].clone());
    }
    if let Some(steer) = out.get_mut("steer").and_then(|v| v.as_object_mut()) {
        steer.remove("looks");
    }
    std::fs::write(&p, serde_json::to_string(&Value::Object(out)).unwrap()).unwrap();
}

fn fx() -> &'static Value {
    static V: OnceLock<Value> = OnceLock::new();
    V.get_or_init(|| {
        let p = fixture_path();
        let s = std::fs::read_to_string(&p).unwrap_or_else(|e| panic!("{}: {e} (missing fixture)", p.display()));
        serde_json::from_str(&s).unwrap()
    })
}
fn f(v: &Value) -> f64 {
    v.as_f64().unwrap_or_else(|| {
        if v.is_null() {
            f64::NAN
        } else if let Some(b) = v.as_bool() {
            b as i32 as f64
        } else {
            panic!("not a number: {v}")
        }
    })
}
fn fs(v: &Value) -> Vec<f64> {
    v.as_array().unwrap().iter().map(f).collect()
}
fn close(a: f64, b: f64, tol: f64) -> bool {
    (a.is_nan() && b.is_nan()) || (a - b).abs() <= tol * (1.0 + a.abs().max(b.abs()))
}
#[track_caller]
fn ck(a: f64, b: f64, tol: f64, what: &dyn Fn() -> String) {
    assert!(close(a, b, tol), "{}: got {a} vs recorded {b} (diff {:e})", what(), (a - b).abs());
}

// ------------------------------------------------------------ the grid
/// Grid versions of the world queries (no solids, no barricades) plus the
/// fixture grid's lit / ceiling maps.
struct GridNav {
    lit: Vec<f64>,
    ceil: Vec<f32>,
}
impl EntityNav for GridNav {
    fn bfs(&self, g: &Game, sx: i32, sz: i32) -> Vec<i32> {
        let (w, h) = (g.w, g.h);
        let mut dist = vec![-1i32; (w * h) as usize];
        let (mut sx, mut sz) = (sx, sz);
        if g.is_wall_tile(sx, sz) {
            let mut found = false;
            for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1), (1, 1), (-1, 1), (1, -1), (-1, -1)] {
                if !g.is_wall_tile(sx + dx, sz + dz) {
                    sx += dx;
                    sz += dz;
                    found = true;
                    break;
                }
            }
            if !found {
                return dist;
            }
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
        let d = hypot2(dx, dz);
        let steps = (d / 0.35).ceil();
        let mut i = 1.0;
        while i < steps {
            let t = i / steps;
            if g.is_wall_tile(g.tx(ax + dx * t), g.tz(az + dz * t)) {
                return false;
            }
            i += 1.0;
        }
        true
    }
    fn circle_hits_wall(&self, g: &Game, px: f64, pz: f64, r: f64) -> bool {
        let (x0, x1, z0, z1) = (g.tx(px - r), g.tx(px + r), g.tz(pz - r), g.tz(pz + r));
        for z in z0..=z1 {
            for x in x0..=x1 {
                if !g.is_wall_tile(x, z) {
                    continue;
                }
                let cx = clamp(px, g.wx(x) - CELL / 2.0, g.wx(x) + CELL / 2.0);
                let cz = clamp(pz, g.wz(z) - CELL / 2.0, g.wz(z) + CELL / 2.0);
                if (px - cx) * (px - cx) + (pz - cz) * (pz - cz) < r * r {
                    return true;
                }
            }
        }
        false
    }
    fn collide_move(&self, g: &Game, p: &mut Vec3, dx: f64, dz: f64, r: f64) {
        if dx != 0.0 && !self.circle_hits_wall(g, p.x + dx, p.z, r) {
            p.x += dx;
        }
        if dz != 0.0 && !self.circle_hits_wall(g, p.x, p.z + dz, r) {
            p.z += dz;
        }
    }
    fn nav_tile_at(&self, g: &Game, px: f64, pz: f64) -> (i32, i32) {
        let (x, z) = (g.tx(px), g.tz(pz));
        if !g.is_wall_tile(x, z) {
            return (x, z);
        }
        let mut best = None;
        let mut bd = 1e9;
        for dz in -1..=1 {
            for dx in -1..=1 {
                if g.is_wall_tile(x + dx, z + dz) {
                    continue;
                }
                let d = (g.wx(x + dx) - px).dpowi(2) + (g.wz(z + dz) - pz).dpowi(2);
                if d < bd {
                    bd = d;
                    best = Some((x + dx, z + dz));
                }
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
        if i < 0 {
            return 0.5;
        }
        self.lit.get(i as usize).copied().unwrap_or(0.5)
    }
}

/// A bare game on a fixture grid, with the grid's world queries installed.
fn game_on(grid: &Value) -> Game {
    let mut g = Game::new_bare();
    let n = grid["n"].as_i64().unwrap() as i32;
    g.w = n;
    g.h = n;
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
fn tables_match_reference() {
    if regen() {
        prune();
        return;
    }
    let j = fx();
    for r in j["wrapPi"].as_array().unwrap() {
        let r = fs(r);
        ck(wrap_pi(r[0]), r[1], 1e-15, &|| "wrapPi".into());
    }
    for r in j["entSmooth"].as_array().unwrap() {
        let r = fs(r);
        assert_eq!(ent_smooth(0.2, 1.1, r[0]), r[1], "ent_smooth({})", r[0]);
    }
}

#[test]
fn solve_limb_matches_reference() {
    let cases = fx()["solveLimb"].as_array().unwrap();
    assert_eq!(cases.len(), 400);
    for c in cases {
        let c = fs(c);
        let (t1, t2) = game::anomalies::rig::solve_limb_ik(c[0], c[1], c[2], c[3], c[4], c[5], c[6] != 0.0);
        ck(t1, c[7], 1e-12, &|| format!("solveLimb t1 {c:?}"));
        ck(t2, c[8], 1e-12, &|| format!("solveLimb t2 {c:?}"));
    }
}

/// The fixtures' `GRID_A(n)` test grid
fn grid_a(n: i32) -> Value {
    let mut tiles = vec![];
    let mut ceil = vec![];
    let mut lit = vec![];
    for z in 0..n {
        for x in 0..n {
            let wall =
                x == 0 || z == 0 || x == n - 1 || z == n - 1 || (x % 6 == 3 && z % 6 == 3) || (x == 9 && z > 2 && z < n - 3 && z != 8);
            tiles.push(if wall { 1 } else { 0 });
            ceil.push(if (x + 2 * z) % 7 == 0 { 2.4f32 as f64 } else { 3.1f32 as f64 });
            lit.push(((x * 7 + z * 3) % 5) as f64 / 4.0);
        }
    }
    serde_json::json!({ "n": n, "tiles": tiles, "ceil": ceil, "lit": lit })
}

// ------------------------------------------------------------ steering
#[test]
fn steering_matches_reference() {
    let j = &fx()["steer"];
    let mut g = game_on(&grid_a(23));
    g.level_num = 2 + game::level::building::BUILDING_FLOORS;
    for r in j["lineWalkable"].as_array().unwrap() {
        let r = fs(r);
        assert_eq!(g.line_walkable(r[0], r[1], r[2], r[3], r[4]), r[5] != 0.0, "line_walkable {r:?}");
    }
    // the grid stand-in agrees with the recorded queries
    let nav = GridNavProbe::new(&g);
    for r in j["queries"].as_array().unwrap() {
        let r = fs(r);
        assert_eq!(nav.los(r[0], r[1], r[2], r[3]), r[5] != 0.0, "los {r:?}");
        assert_eq!(nav.hits(r[0], r[1], r[4]), r[6] != 0.0, "circle_hits_wall {r:?}");
        assert_eq!(nav.tile(r[0], r[1]), (r[7] as i32, r[8] as i32), "nav_tile_at {r:?}");
        assert_eq!(nav.ceil(r[0], r[1]), r[9], "ceil_at_world {r:?}");
        let mut p = v3(r[0], 0.0, r[1]);
        nav.mv(&mut p, (r[2] - r[0]) * 0.1, (r[3] - r[1]) * 0.1, r[4]);
        assert_eq!((p.x, p.z), (r[10], r[11]), "collide_move {r:?}");
    }
    for b in j["bfs"].as_array().unwrap() {
        let (x, z) = (b[0].as_i64().unwrap() as i32, b[1].as_i64().unwrap() as i32);
        assert_eq!(g.e_bfs_pub(x, z).iter().map(|v| *v as f64).collect::<Vec<_>>(), fs(&b[2]), "bfs({x}, {z})");
    }
    // `entity_turn`
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
        g.level_num = 2 + game::level::building::BUILDING_FLOORS;
        let kind = w["type"].as_str().unwrap();
        let (sx, sz, gx, gz) = (
            w["sx"].as_i64().unwrap() as i32,
            w["sz"].as_i64().unwrap() as i32,
            w["gx"].as_i64().unwrap() as i32,
            w["gz"].as_i64().unwrap() as i32,
        );
        let direct = w["direct"].as_i64().unwrap() != 0;
        let speed = f(&w["speed"]);
        with_seed(w["seed"].as_u64().unwrap() as u32, || {
            let ei = g.add_entity(if kind == "staff" || kind == "shadow" { "smiler" } else { kind }, sx, sz, 0);
            g.entities[ei].kind = kind.into(); // staff / shadow steer with their own motion constants
            g.entities[ei].freeze = 0.0;
            let field = Rc::new(g.e_bfs_pub(gx, gz));
            let mut field2 = field.clone();
            for (k, st) in w["steps"].as_array().unwrap().iter().enumerate() {
                let st = fs(st);
                if k == 200 {
                    field2 = Rc::new(g.e_bfs_pub(gx, gz));
                }
                g.entities[ei].set_n("lungeT", st[3]);
                if kind == "hound" {
                    g.entities[ei].state = if st[4] != 0.0 { "hunt".into() } else { "wander".into() };
                }
                let done =
                    g.entity_step_toward(ei, Some(field2.clone()), st[0], speed, if direct { Some(v3(st[1], 0.0, st[2])) } else { None });
                let e = &g.entities[ei];
                let what = |x: &str| format!("walk {kind} {sx},{sz}→{gx},{gz} step {k}: {x}");
                let ours = [e.pos().x, e.pos().z, e.vx, e.vz, e.yaw, e.n("aimX"), e.n("aimZ")];
                for (q, v) in ours.iter().enumerate() {
                    ck(*v, st[5 + q], 1e-7, &|| what(&format!("value {q} ({ours:?})")));
                }
                assert_eq!(e.fl("aimFinal"), st[12] != 0.0, "{}", what("aimFinal"));
                assert_eq!(e.fl("aimOk"), st[13] != 0.0, "{}", what("aimOk"));
                ck(e.n("aimT"), st[14], 1e-12, &|| what("aimT"));
                assert_eq!(done, st[15] != 0.0, "{}", what("done"));
            }
        });
    }
}

/// The grid queries through the same `GridNav` the entities use.
struct GridNavProbe<'a> {
    g: &'a Game,
    n: GridNav,
}
impl<'a> GridNavProbe<'a> {
    fn new(g: &'a Game) -> Self {
        let gr = grid_a(g.w);
        GridNavProbe { g, n: GridNav { lit: fs(&gr["lit"]), ceil: fs(&gr["ceil"]).iter().map(|v| *v as f32).collect() } }
    }
    fn los(&self, a: f64, b: f64, c: f64, d: f64) -> bool {
        self.n.los(self.g, a, b, c, d)
    }
    fn hits(&self, x: f64, z: f64, r: f64) -> bool {
        self.n.circle_hits_wall(self.g, x, z, r)
    }
    fn tile(&self, x: f64, z: f64) -> (i32, i32) {
        self.n.nav_tile_at(self.g, x, z)
    }
    fn ceil(&self, x: f64, z: f64) -> f64 {
        self.n.ceil_at_world(self.g, x, z)
    }
    fn mv(&self, p: &mut Vec3, dx: f64, dz: f64, r: f64) {
        self.n.collide_move(self.g, p, dx, dz, r)
    }
}
trait BfsPub {
    fn e_bfs_pub(&self, x: i32, z: i32) -> Vec<i32>;
}
impl BfsPub for Game {
    fn e_bfs_pub(&self, x: i32, z: i32) -> Vec<i32> {
        let gr = grid_a(self.w);
        GridNav { lit: fs(&gr["lit"]), ceil: vec![] }.bfs(self, x, z)
    }
}

// ------------------------------------------------------------ senses and the director
/// A smiler at tile (5, 5) on the open test grid, you 8 m away at (5, 7)
fn senses_game(yaw: f64) -> (Game, usize) {
    let mut g = game_on(&grid_a(23));
    g.level_num = 3 + game::level::building::BUILDING_FLOORS;
    let ei = g.add_entity("smiler", 5, 5, 0);
    let e = &mut g.entities[ei];
    e.freeze = 0.0;
    e.yaw = yaw;
    e.stare_t = 100.0;
    g.player.pos = v3(g.wx(5), 0.0, g.wz(7));
    g.player.audible_range = 0.0;
    (g, ei)
}

// (the old senses' tests, a view cone and wander / search / hunt, went with
// the Smiler to the new brain in E3: no kind on the old AI looks ahead any
// more, and the Watcher has its own rule; the Smiler's are in
// tests/creatures_rules.rs)

#[test]
fn the_director_sends_one_then_backs_off() {
    let (mut g, ei) = senses_game(PI);
    g.player.pos = v3(g.wx(17), 0.0, g.wz(17));
    g.entities[ei].g.set_pos(g.wx(3), 0.0, g.wz(19));
    g.reset_director();
    assert_eq!(g.director.phase, game::entities::director::Phase::Calm);
    assert!(g.director.t >= 45.0 * 0.9 && g.director.t <= 80.0);
    // the quiet ends: the nearest prowler is sent toward you, not onto you
    g.director.t = 0.0;
    g.update_director(1.0 / 60.0);
    g.update_director(1.0 / 60.0);
    assert_eq!(g.director.phase, game::entities::director::Phase::Build);
    assert_eq!(g.director.stalker, Some(ei));
    let e = &g.entities[ei];
    assert!(e.fl("stalk"));
    let (gx, gz) = e.w_target.expect("a goal near you");
    let dg = hypot2(g.wx(gx) - g.player.pos.x, g.wz(gz) - g.player.pos.z);
    assert!((6.0..=16.0).contains(&dg), "goal {dg:.1} m from you");
    // it finds you: a peak; the gauge fills, you break away, it withdraws
    g.entities[ei].state = "hunt".into();
    g.update_director(1.0 / 60.0);
    assert_eq!(g.director.phase, game::entities::director::Phase::Peak);
    g.director.menace = 1.0;
    g.entities[ei].lose_t = 1.0;
    for _ in 0..200 {
        g.update_director(1.0 / 60.0);
    }
    assert_eq!(g.director.phase, game::entities::director::Phase::Relax);
    let e = &g.entities[ei];
    assert_eq!(e.state, "wander");
    assert!(e.n("retreatT") > 0.0 && !e.fl("stalk"), "sent away, and it stays away a while");
}
