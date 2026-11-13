//! The visual playtest (`docs/engineering/VISUAL_PLAYTEST.md`). The tour (VP1): every
//! stand is one a body can stand on, at an edge it cannot pass, facing it;
//! the tour is the same twice; it is spread over the level; it keeps to an
//! area and a budget; it looks up, down, along and crouched. Its renders and
//! pixel detectors (VP2, `backrooms --tour`): a hole and a fight planted are
//! found, the clean view and the void over a parapet are not. Its geometric
//! detectors (VP3): a box round the eye, a flat-coloured box and the fighting
//! pair are each named in `findings.tsv`.
//!
//!   cargo test --release -p backrooms --test visual_tour -- --nocapture   # stands and views per floor
mod common;

use game::debug::sight::FLAG_PX;
use game::debug::tour::{TourOpts, BODY};
use game::prelude::{CROUCH_EYE, EYE};
use game::Game;

fn floor(seed: u32, level: i32) -> Game {
    game::base::rng::seed_native((u64::from(seed) << 8) | level as u64);
    let mut g = common::new_game();
    g.run_seed = seed;
    g.init_level(level);
    g.state = "playing".into();
    g
}

/// A Building floor and a floor of the rooms
const FLOORS: [(u32, i32); 2] = [(4242, 1), (4242, 5)];

#[test]
fn every_stand_is_at_an_edge_a_body_can_reach() {
    for (seed, level) in FLOORS {
        let g = floor(seed, level);
        let stands = g.tour_stands(None);
        assert!(stands.len() > 40, "L{level}: only {} stands", stands.len());
        let mut kinds = std::collections::BTreeMap::new();
        for s in &stands {
            assert!(g.standable(s.x, s.z), "L{level}: a stand a body cannot stand on: {s:?}");
            // a step further toward the edge is not one a body can take,
            // and the edge is a body's radius away, not across the room
            let (fx, fz) = (-s.yaw.sin(), -s.yaw.cos());
            assert!(
                g.circle_hits_wall(s.x + fx * 0.05, s.z + fz * 0.05, BODY) || !g.standable(s.x + fx * 0.05, s.z + fz * 0.05),
                "L{level}: {s:?}"
            );
            let reach = if s.corner { 0.05 } else { 0.03 };
            assert!(!g.standable(s.x + fx * reach, s.z + fz * reach), "L{level}: a stand not at its edge: {s:?}");
            // the floor under it is the floor its eye is put over
            assert!((g.floor_y_at(s.x, s.z) - s.floor).abs() < 0.31, "L{level}: {s:?}");
            *kinds.entry(s.edge).or_insert(0) += 1;
        }
        println!("L{level}: {} stands {kinds:?}", stands.len());
        if level <= 3 {
            assert!(kinds.contains_key("void"), "L{level}: the Building has edges over the void: {kinds:?}");
        }
        assert!(kinds.contains_key("wall") || kinds.contains_key("solid"), "L{level}: {kinds:?}");
        assert!(stands.iter().any(|s| s.corner), "L{level}: no corners");
    }
}

#[test]
fn the_tour_is_the_same_twice_and_spread_over_the_level() {
    for (seed, level) in FLOORS {
        let o = TourOpts { budget: 120, area: None };
        let a = floor(seed, level).tour_views(&o);
        let b = floor(seed, level).tour_views(&o);
        assert_eq!(a, b, "L{level}: a tour must not depend on anything but the level");
        assert!(a.len() <= o.budget && a.len() > o.budget - 7, "L{level}: {} views for a budget of {}", a.len(), o.budget);
        // spread: the chosen stands reach most of the way across every stand's extent
        let g = floor(seed, level);
        let all = g.tour_stands(None);
        let span = |f: &dyn Fn(f64, f64) -> f64, xs: &mut dyn Iterator<Item = (f64, f64)>| {
            let v: Vec<f64> = xs.map(|(x, z)| f(x, z)).collect();
            v.iter().cloned().fold(f64::MIN, f64::max) - v.iter().cloned().fold(f64::MAX, f64::min)
        };
        for (axis, f) in [("x", &(|x: f64, _z: f64| x) as &dyn Fn(f64, f64) -> f64), ("z", &|_x: f64, z: f64| z)] {
            let whole = span(f, &mut all.iter().map(|s| (s.x, s.z)));
            let toured = span(f, &mut a.iter().map(|v| (v.x, v.z)));
            assert!(toured > 0.8 * whole, "L{level}: the tour spans {toured:.1} m of {whole:.1} m in {axis}");
        }
        // and every look is there, crouched at a crouching eye
        for look in ["at", "left", "right", "up", "down", "crouch"] {
            assert!(a.iter().any(|v| v.look == look), "L{level}: no {look} view");
        }
        for v in &a {
            let floor = g.floor_y_at(v.x, v.z);
            let eye = if v.look == "crouch" { CROUCH_EYE } else { EYE };
            assert!((v.y - floor - eye).abs() < 0.31, "L{level}: {} at eye {:.2} over {floor:.2}", v.name, v.y);
            assert!(v.pitch.abs() < 1.2, "{}", v.name);
        }
        let names: std::collections::BTreeSet<&str> = a.iter().map(|v| v.name.as_str()).collect();
        assert_eq!(names.len(), a.len(), "L{level}: two views with one name");
        println!("L{level}: {} views; first {:?}", a.len(), a[0]);
    }
}

#[test]
fn an_area_keeps_the_tour_inside_it() {
    let (seed, level) = FLOORS[0];
    let g = floor(seed, level);
    // the middle quarter of the level's stands
    let all = g.tour_stands(None);
    let (mut xs, mut zs): (Vec<f64>, Vec<f64>) = all.iter().map(|s| (s.x, s.z)).unzip();
    xs.sort_by(f64::total_cmp);
    zs.sort_by(f64::total_cmp);
    let area = [xs[xs.len() / 4], zs[zs.len() / 4], xs[xs.len() * 3 / 4], zs[zs.len() * 3 / 4]];
    let views = g.tour_views(&TourOpts { budget: 60, area: Some(area) });
    assert!(!views.is_empty(), "no views inside {area:?}");
    for v in &views {
        assert!(v.x >= area[0] && v.x <= area[2] && v.z >= area[1] && v.z <= area[3], "{} outside {area:?}", v.name);
    }
    // nothing at all for an area off the level
    assert!(g.tour_views(&TourOpts { budget: 60, area: Some([1e4, 1e4, 1e4 + 8.0, 1e4 + 8.0]) }).is_empty());
    // a corner beside an area's edge stays out of it (review A: a corner
    // stood 5 cm outside this one, on level 5)
    let g5 = floor(4242, 5);
    let edge = [67.78, 69.78, 73.73, 77.78];
    let stands = g5.tour_stands(Some(edge));
    assert!(!stands.is_empty());
    for s in &stands {
        assert!(s.x >= edge[0] && s.x <= edge[2] && s.z >= edge[1] && s.z <= edge[3], "{s:?} outside {edge:?}");
    }
}

/// A view's row of `views.tsv`
#[derive(Debug)]
struct Row {
    name: String,
    look: String,
    edge: String,
    background: usize,
    gaps: usize,
    flicker: usize,
}

/// `backrooms --tour` on one floor (the real renders), its rows
fn toured(name: &str, level: i32, extra: &[&str]) -> Vec<Row> {
    let dir = format!("{}/visual_tour/{name}", env!("CARGO_TARGET_TMPDIR"));
    let _ = std::fs::remove_dir_all(&dir);
    let l = level.to_string();
    let mut args = vec!["--tour", &dir, "--seed", "4242", "--levels", &l, "--width", "480", "--height", "300"];
    args.extend_from_slice(extra);
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_backrooms")).args(&args).output().expect("run backrooms --tour");
    assert!(out.status.success(), "{}\n{}", String::from_utf8_lossy(&out.stdout), String::from_utf8_lossy(&out.stderr));
    println!("{name}: {}", String::from_utf8_lossy(&out.stdout).trim());
    let tsv = std::fs::read_to_string(format!("{dir}/L{level}/views.tsv")).expect("views.tsv");
    let mut lines = tsv.lines();
    let head: Vec<&str> = lines.next().unwrap().split('\t').collect();
    let col = |n: &str| head.iter().position(|h| *h == n).unwrap_or_else(|| panic!("no column {n}"));
    lines
        .map(|line| {
            let c: Vec<&str> = line.split('\t').collect();
            let num = |n: &str| c[col(n)].parse::<usize>().unwrap();
            Row {
                name: c[col("name")].into(),
                look: c[col("look")].into(),
                edge: c[col("edge")].into(),
                background: num("background"),
                gaps: num("gaps"),
                flicker: num("flicker"),
            }
        })
        .collect()
}

/// The detectors' known answers (`docs/engineering/VISUAL_PLAYTEST.md` VP2): a hole
/// cut where the tour's first stand faces a wall is found in the view facing
/// it, and two squares fighting for one depth before its eye are found where
/// it sees them at a grazing angle (looking up past them: face on, their
/// fight is thin lines that move like an edge); the same views of the floor
/// as built are clean
#[test]
fn a_planted_hole_and_fight_are_found_and_the_clean_views_are_not() {
    let clean = toured("clean", 5, &["--tour-budget", "6"]);
    let hole = toured("hole", 5, &["--tour-budget", "6", "--tour-plant", "hole"]);
    let fight = toured("fight", 5, &["--tour-budget", "6", "--tour-plant", "fight"]);
    let view = |rows: &[Row], look: &str| rows.iter().find(|r| r.look == look).map(|r| (r.name.clone(), r.gaps, r.flicker)).unwrap();
    for look in ["at", "up"] {
        let (c, h, f) = (view(&clean, look), view(&hole, look), view(&fight, look));
        println!("{look}: clean {c:?} hole {h:?} fight {f:?}");
        assert!(c.1 < FLAG_PX && c.2 < FLAG_PX, "the clean view: {c:?}");
    }
    assert!(view(&hole, "at").1 >= FLAG_PX, "the hole is not found: {:?}", view(&hole, "at"));
    assert!(view(&fight, "up").2 >= FLAG_PX, "the fight is not found: {:?}", view(&fight, "up"));
}

/// Looking over a parapet into the void, the background shows (nothing is
/// drawn down there within the far plane) and it is no gap: on the Works,
/// every view over the void with the background in it is clean of gaps
#[test]
fn the_void_over_a_parapet_is_not_a_gap() {
    let rows = toured("void", 3, &["--tour-budget", "120"]);
    let open: Vec<&Row> = rows.iter().filter(|r| r.edge == "void" && r.background > 1000).collect();
    println!("{} views over the void with the background in them", open.len());
    assert!(open.len() >= 3, "{rows:?}");
    for r in &open {
        assert!(r.gaps < FLAG_PX, "the void is not a hole: {r:?}");
    }
}

/// `findings.tsv` of a tour run: (kind, label, views, flagged)
fn findings(name: &str, level: i32) -> Vec<(String, String, usize, bool)> {
    let p = format!("{}/visual_tour/{name}/L{level}/findings.tsv", env!("CARGO_TARGET_TMPDIR"));
    std::fs::read_to_string(&p)
        .unwrap_or_else(|e| panic!("{p}: {e}"))
        .lines()
        .skip(1)
        .map(|l| {
            let c: Vec<&str> = l.split('\t').collect();
            (c[0].to_string(), c[1].to_string(), c[3].parse().unwrap(), c[9] == "true")
        })
        .collect()
}

/// The geometric detectors' known answers (`docs/engineering/VISUAL_PLAYTEST.md` VP3):
/// a box put round the eye of the tour's first stand is a clip (and a body
/// inside it); a flat-coloured box beside it is plain; the two fighting
/// squares are a coplanar pair; none of these is in the floor as built
#[test]
fn a_box_round_the_eye_a_flat_box_and_a_coplanar_pair_are_named() {
    let clean = toured("things_clean", 5, &["--tour-budget", "6"]);
    let _ = toured("things_inside", 5, &["--tour-budget", "6", "--tour-plant", "inside"]);
    let _ = toured("things_flat", 5, &["--tour-budget", "6", "--tour-plant", "flat"]);
    let _ = toured("things_fight", 5, &["--tour-budget", "6", "--tour-plant", "fight"]);
    let has = |run: &str, kind: &str, label: &str| findings(run, 5).iter().any(|f| f.0 == kind && f.1.contains(label));
    assert!(has("things_inside", "clip", "tour_plant_inside"), "{:?}", findings("things_inside", 5));
    assert!(has("things_inside", "body", "tour_plant_inside"), "{:?}", findings("things_inside", 5));
    assert!(has("things_flat", "plain", "tour_plant_flat"), "{:?}", findings("things_flat", 5));
    assert!(has("things_fight", "coplanar", "tour_plant_fight_0"), "{:?}", findings("things_fight", 5));
    // the near plane's face reaching into a box before the eye is a clip; a
    // ring round the eye (in its box, its hole round the eye) is not
    let _ = toured("things_nearface", 5, &["--tour-budget", "6", "--tour-plant", "nearface"]);
    let _ = toured("things_ring", 5, &["--tour-budget", "6", "--tour-plant", "ring"]);
    assert!(has("things_nearface", "clip", "tour_plant_nearface"), "{:?}", findings("things_nearface", 5));
    assert!(
        !findings("things_ring", 5).iter().any(|f| f.1.contains("tour_plant_ring") && matches!(f.0.as_str(), "clip" | "body")),
        "{:?}",
        findings("things_ring", 5)
    );
    // the floor as built: no clip, no body and nothing plain at the first stand
    assert!(
        findings("things_clean", 5).iter().all(|f| !matches!(f.0.as_str(), "clip" | "body" | "plain")),
        "{:?}",
        findings("things_clean", 5)
    );
    assert_eq!(clean.len(), 6);
}

/// The things are named as what they were placed as, the static instancing
/// notwithstanding (review A: its meshes were all "level"), and a coplanar
/// finding lies within reach of a stand (a long part's far end is not)
#[test]
fn things_keep_their_names_and_coplanar_findings_stand_near_the_tour() {
    let g = floor(4242, 5);
    let parts = game::debug::things::parts(&g);
    let instanced: Vec<_> = parts.iter().filter(|p| p.obj.is_instanced()).collect();
    assert!(instanced.iter().any(|p| p.label != "level"), "no instanced part keeps its thing's name");
    let rows = toured("coplanar_reach", 1, &["--tour-budget", "60"]);
    assert!(!rows.is_empty());
    let dir = format!("{}/visual_tour/coplanar_reach/L1", env!("CARGO_TARGET_TMPDIR"));
    let views = std::fs::read_to_string(format!("{dir}/views.tsv")).unwrap();
    let eyes: Vec<(f64, f64, f64)> = views
        .lines()
        .skip(1)
        .map(|l| {
            let cam = l.split('\t').nth(4).unwrap().split_once(':').unwrap().1;
            let n: Vec<f64> = cam.split(',').map(|x| x.parse().unwrap()).collect();
            (n[0], n[1], n[2])
        })
        .collect();
    let found = std::fs::read_to_string(format!("{dir}/findings.tsv")).unwrap();
    let mut n = 0;
    for l in found.lines().skip(1).filter(|l| l.starts_with("coplanar")) {
        let c: Vec<&str> = l.split('\t').collect();
        let at: Vec<f64> = c[5..8].iter().map(|x| x.parse().unwrap()).collect();
        let near =
            eyes.iter().map(|e| ((e.0 - at[0]).powi(2) + (e.1 - at[1]).powi(2) + (e.2 - at[2]).powi(2)).sqrt()).fold(f64::MAX, f64::min);
        // within reach in the plan (6 m), and a storey and a half up or down
        assert!(near < game::debug::things::COPLANAR_REACH + 9.0, "{l}: {near:.1} m from every stand");
        n += 1;
    }
    println!("{n} coplanar findings, each near the tour");
}

/// The pixel detectors' known answers on the Building (review A: their only
/// Building test was the void's, so its gap path could find nothing and
/// pass): a hole cut in the Well where the first stand faces a mass is
/// found, and two dim squares fighting there, lit as the Well lights them,
/// flicker (a fight a few levels apart, under an absolute threshold of 48
/// it was never found)
#[test]
fn a_hole_and_a_dark_fight_in_the_well_are_found() {
    let clean = toured("well_clean", 1, &["--tour-budget", "6"]);
    let hole = toured("well_hole", 1, &["--tour-budget", "6", "--tour-plant", "hole"]);
    let fight = toured("well_darkfight", 1, &["--tour-budget", "6", "--tour-plant", "darkfight"]);
    let at = |rows: &[Row], look: &str| rows.iter().find(|r| r.look == look).map(|r| (r.name.clone(), r.gaps, r.flicker)).unwrap();
    println!(
        "clean {:?} {:?}\nhole {:?}\nfight {:?} {:?}",
        at(&clean, "at"),
        at(&clean, "up"),
        at(&hole, "at"),
        at(&fight, "at"),
        at(&fight, "up")
    );
    assert!(at(&clean, "at").1 < FLAG_PX && at(&clean, "at").2 < FLAG_PX && at(&clean, "up").2 < FLAG_PX, "{clean:?}");
    assert!(at(&hole, "at").1 >= FLAG_PX, "the Building's hole is not found: {hole:?}");
    assert!(at(&fight, "up").2 >= FLAG_PX || at(&fight, "at").2 >= FLAG_PX, "the dark fight is not found: {fight:?}");
}

/// The stairs are toured (review A: a walk whose floor changed on the way
/// was left out, so the Stair Halls' flights had no stand), each stand's
/// eye over its own floor; and a tool's eye takes nothing (review A: a
/// view's step of play picked up what lay within a metre, gone for every
/// later view)
#[test]
fn the_stairs_are_toured_and_a_tools_eye_takes_nothing() {
    let g = floor(4242, 2);
    let stands = g.tour_stands(None);
    let on_flight = stands
        .iter()
        .filter(|s| g.ground.get(g.t_idx(g.tx(s.x), g.tz(s.z))).is_some_and(|gr| gr.kind == game::level::archgen::GROUND_STAIR))
        .count();
    println!("{on_flight} of {} stands of the Stair Halls on a flight", stands.len());
    assert!(on_flight > 0, "no stand on a flight");
    // a pickup under a tool's eye stays; under a player it is taken
    let mut found = None;
    for level in [5, 6, 7, 8] {
        let g = floor(4242, level);
        if let Some(i) = (0..g.almonds.len()).find(|&i| !g.almonds[i].taken) {
            found = Some((g, i));
            break;
        }
    }
    let (mut g, i) = found.expect("almond water on one of 4242's levels 5-8");
    let p = g.almonds[i].g.pos();
    g.player.pos = engine::math::Vec3::new(p.x, g.floor_y_at(p.x, p.z) + EYE, p.z);
    g.tool_eye = true;
    g.update_player(1.0 / 60.0);
    assert!(!g.almonds[i].taken, "a tool's eye took the almond water");
    g.tool_eye = false;
    g.update_player(1.0 / 60.0);
    assert!(g.almonds[i].taken, "the test is blind: play did not take it either");
}

/// Under a flight's head a look passes: a flight's solid is a flat slab at
/// its foot, drawn along its slope, and the gap rays follow the slope (the
/// first tour of the Stair Halls' every stand flagged 112 views of the open
/// space under flights as holes; these three among them)
#[test]
fn the_space_under_a_flight_is_no_gap() {
    let rows = toured(
        "under_flights",
        2,
        &[
            "--tour-at",
            "L2_0516_up:20.2000,-12.7800,-15.2000,1.57080,0.69813;L2_0227_crouch:-48.2200,15.4500,-39.2000,-1.57080,-0.17453;L2_0297_at:-44.2200,7.6200,-27.2000,-1.57080,0.00000",
        ],
    );
    assert_eq!(rows.len(), 3);
    for r in &rows {
        assert!(r.background > 1000 && r.gaps < FLAG_PX, "{r:?}");
    }
}

/// A far silhouette's edge is no gap, a crack and a hole still are (the
/// Building's tour, `docs/research/playtests/visual/2026-11-13-building-1-3.md`:
/// every gap flagged was the edge pixel of a mass 30-40 m off, its ray
/// grazing the mass by half a pixel; the gap rays' rim now grows with
/// their length): the record's edges on the Works and the Stair Halls are
/// clean, and the rooms' ceiling crack and the hole over a wall are found
#[test]
fn a_far_edge_is_no_gap_and_a_crack_still_is() {
    let works = toured("far_edge_works", 3, &["--tour-at", "L3_0786_at:44.2200,-4.3800,22.0000,1.57080,0.00000"]);
    let halls = toured("far_edge_halls", 2, &["--tour-at", "L2_1117_up:-47.2000,-7.3800,44.2200,0.00000,0.69813"]);
    for r in works.iter().chain(halls.iter()) {
        assert!(r.background > 1000 && r.gaps < FLAG_PX, "a far edge flagged: {r:?}");
    }
    let rooms = toured(
        "crack_rooms",
        5,
        &["--tour-at", "L5_010_right:37.9800,1.6200,-37.2000,-2.79253,0.00000;L5_011_left:38.8000,1.6200,34.0000,1.22173,0.00000"],
    );
    for r in &rooms {
        assert!(r.gaps >= FLAG_PX * 10, "the rooms' {} is no longer found: {r:?}", if r.look == "at" { "gap" } else { "crack" });
    }
}

/// The gaps detector's rim far off (review B): a gap ray 25 m out over the
/// void, through a pixel the size the tests render (300 rows), into the
/// middle of a walk's floor slab (0.4 m thick: 0.2 m inside each face) still
/// meets the level, so a hole in a far slab is found. The rim grows by
/// `SPREAD` pixels' span with the ray's length; at 2 it was 0.25 m there,
/// deeper than the slab's middle, and no ray through a far slab could flag
#[test]
fn a_gap_ray_still_meets_a_far_slab() {
    use game::level::archgen::{GROUND_FLOOR, GROUND_VOID, SLAB};
    const OUT: f64 = 25.0;
    for level in [2, 3] {
        let g = floor(4242, level);
        let pixel = g.camera.camera().fov.to_radians() / 300.0;
        let tile = |x: i32, z: i32| (x >= 0 && z >= 0 && x < g.w && z < g.h).then(|| g.ground[(z * g.w + x) as usize]);
        let reach = (OUT / game::prelude::CELL).ceil() as i32 + 1;
        let mut tried = 0;
        for z in 0..g.h {
            for x in 0..g.w {
                let Some(t) = tile(x, z).filter(|t| t.kind == GROUND_FLOOR) else { continue };
                for (dx, dz) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
                    if !(1..=reach).all(|k| tile(x + dx * k, z + dz * k).is_some_and(|n| n.kind == GROUND_VOID)) {
                        continue;
                    }
                    let edge = (g.wx(x) + dx as f64 * game::prelude::CELL / 2.0, g.wz(z) + dz as f64 * game::prelude::CELL / 2.0);
                    let eye = (edge.0 + dx as f64 * OUT, t.h0 as f64 - SLAB / 2.0, edge.1 + dz as f64 * OUT);
                    let d = (-dx as f64, 0.0, -dz as f64);
                    // the slab is what the ray meets first, at the walk's edge
                    let Some(clean) = g.ray_meets_level(eye, d, 40.0) else { continue };
                    if (clean - OUT).abs() > 0.1 {
                        continue;
                    }
                    let far = g.ray_meets_level_by(eye, d, 40.0, pixel);
                    assert!(
                        far.is_some_and(|t| t < OUT + 1.0),
                        "L{level}: a gap ray into the slab at ({}, {}) 25 m off meets {far:?}",
                        edge.0,
                        edge.1
                    );
                    tried += 1;
                }
            }
        }
        println!("L{level}: {tried} far slabs met");
        assert!(tried > 0, "L{level}: no walk's edge with 25 m of void before it");
    }
}

/// The tour's stands are where play stops a body (review B). The tour tests
/// its own `standable` at its own `BODY`, so a body of the wrong size (it
/// was the robot vacuum's 0.20 m) toured 15 cm past every edge a player
/// reaches and only a reader of its renders noticed. Here the player's own
/// step (`update_player`: `collide_move` with `PLAYER_R`), walking from a
/// metre back toward the edge each sampled stand faces, stops at the stand:
/// no further (the tour's body too thin) and no shorter (too fat), within
/// the tour's step and one of the player's
#[test]
fn the_players_own_step_stops_at_the_tours_stands() {
    const DT: f64 = 1.0 / 500.0;
    for (seed, level) in [(4242, 1), (4242, 2), (4242, 3), (4242, 5)] {
        let mut g = floor(seed, level);
        let stands: Vec<_> = g.tour_stands(None).into_iter().filter(|s| !s.corner).collect();
        let every = (stands.len() / 40).max(1);
        let (mut walked, mut worst) = (0, 0.0f64);
        for s in stands.iter().step_by(every) {
            let d = (-s.yaw.sin(), -s.yaw.cos());
            // a metre of floor behind it, walked by the tour's body too
            if !(0..=50).all(|k| g.standable(s.x - d.0 * k as f64 * 0.02, s.z - d.1 * k as f64 * 0.02)) {
                continue;
            }
            let (x, z) = (s.x - d.0, s.z - d.1);
            g.player.pos = game::prelude::Vec3::new(x, g.floor_y_at(x, z) + EYE, z);
            g.player.yaw = s.yaw;
            g.player.crouch = false;
            g.key_down("KeyW", false);
            let mut stride = 0.0f64;
            let mut still = 0;
            for _ in 0..5000 {
                let before = g.player.pos;
                g.update_player(DT);
                let moved = (g.player.pos.x - before.x).hypot(g.player.pos.z - before.z);
                stride = stride.max(moved);
                still = if moved < 1e-9 { still + 1 } else { 0 };
                if still > 20 {
                    break;
                }
            }
            g.key_up("KeyW");
            let (px, pz) = (g.player.pos.x, g.player.pos.z);
            let along = (px - s.x) * d.0 + (pz - s.z) * d.1;
            let across = (px - s.x) * -d.1 + (pz - s.z) * d.0;
            assert!(stride > 0.0, "L{level}: the player did not move toward {s:?}");
            assert!(
                (-0.001..=0.02 + stride + 0.001).contains(&along) && across.abs() < 0.01,
                "L{level}: play stopped a body {along:.3} m past (across {across:.3}) the tour's stand {s:?} (its step {stride:.4})"
            );
            worst = worst.max(along);
            walked += 1;
        }
        println!("L{level}: {walked} stands walked to, the furthest {worst:.3} m past");
        assert!(walked >= 10, "L{level}: only {walked} stands with a metre of floor behind them");
    }
}
